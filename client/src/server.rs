//! Local companion HTTP and WebSocket server for real-time web frontend integration.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path as AxumPath, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::downloader::{DownloadProgressEvent, ModDownloader};
use crate::error::Result;
use crate::fs25::{LocalMod, ModScanner};
use crate::launcher::launch_fs25;
use crate::manifest::{compute_manifest_diff, fetch_manifest};
use crate::protocol::{ClientMessage, CompanionState, LocalModSummary, ServerMessage};

/// Shared companion application state.
pub struct AppState {
    pub config: RwLock<Config>,
    pub scanner: Mutex<ModScanner>,
    pub sync_in_progress: AtomicBool,
    pub sync_progress: AtomicU8,
    pub current_sync_mod: RwLock<Option<String>>,
    pub last_sync_at: RwLock<Option<String>>,
    pub http_client: reqwest::Client,
    pub broadcast_tx: broadcast::Sender<ServerMessage>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let (broadcast_tx, _) = broadcast::channel(128);
        Self {
            config: RwLock::new(config),
            scanner: Mutex::new(ModScanner::new()),
            sync_in_progress: AtomicBool::new(false),
            sync_progress: AtomicU8::new(0),
            current_sync_mod: RwLock::new(None),
            last_sync_at: RwLock::new(None),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap_or_default(),
            broadcast_tx,
        }
    }

    /// Builds the current [`CompanionState`] snapshot.
    pub async fn current_companion_state(&self) -> CompanionState {
        let cfg = self.config.read().await;
        let last_sync = self.last_sync_at.read().await.clone();
        let current_mod = self.current_sync_mod.read().await.clone();

        CompanionState {
            is_connected: true,
            version: format!("v{}-rust", env!("CARGO_PKG_VERSION")),
            os: std::env::consts::OS.to_string(),
            mods_path: cfg.mods_path.to_string_lossy().to_string(),
            last_sync_at: last_sync,
            sync_in_progress: self.sync_in_progress.load(Ordering::Relaxed),
            sync_progress: self.sync_progress.load(Ordering::Relaxed),
            current_sync_mod: current_mod,
            simulation_mode: false,
        }
    }
}

/// Creates the Axum router for the companion service.
pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::permissive();

    Router::new()
        .route("/health", get(health_handler))
        .route("/status", get(status_handler))
        .route("/api/mods", get(list_mods_handler))
        .route("/api/sync/{server_id}", post(sync_server_handler))
        .route("/ws", get(ws_upgrade_handler))
        .layer(cors)
        .with_state(state)
}

/// Starts the companion daemon listener.
pub async fn run_server(config: Config) -> Result<()> {
    let port = config.port;
    let state = Arc::new(AppState::new(config));
    let app = create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!(
        "farmLink Rust companion listening on ws://{}/ws and http://{}",
        addr, addr
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "farmlink-companion",
        "version": format!("v{}-rust", env!("CARGO_PKG_VERSION"))
    }))
}

async fn status_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let status = state.current_companion_state().await;
    Json(status)
}

async fn list_mods_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mods_path = state.config.read().await.mods_path.clone();
    let mut scanner = state.scanner.lock().await;

    match scanner.scan_directory(&mods_path) {
        Ok(mods) => {
            let total_size: u64 = mods.iter().map(|m| m.file_size_bytes).sum();
            let summaries: Vec<LocalModSummary> = mods
                .into_iter()
                .map(|m| LocalModSummary {
                    file_name: m.file_name,
                    title: m.title,
                    version: m.version,
                    author: m.author,
                    hash_sha256: m.hash_sha256,
                    file_size_bytes: m.file_size_bytes,
                })
                .collect();

            Json(json!({
                "mods": summaries,
                "total_mods": summaries.len(),
                "total_size_bytes": total_size
            }))
        }
        Err(e) => {
            error!("error scanning mods: {e}");
            Json(json!({ "error": e.to_string(), "mods": [] }))
        }
    }
}

async fn sync_server_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(server_id): AxumPath<String>,
) -> impl IntoResponse {
    tokio::spawn(async move {
        if let Err(e) = perform_sync(state, server_id, None, None).await {
            error!("sync failed: {e}");
        }
    });

    Json(json!({ "status": "sync_started" }))
}

async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = state.broadcast_tx.subscribe();

    // 1. Send initial state on connection
    let initial_state = state.current_companion_state().await;
    if let Ok(json_text) = serde_json::to_string(&ServerMessage::State(initial_state)) {
        let _ = sender.send(Message::Text(json_text.into())).await;
    }

    // Spawn broadcast forwarder to this WebSocket client
    let mut forward_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if let Ok(json_text) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json_text.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming client messages
    let mut recv_task = {
        let state = state.clone();
        tokio::spawn(async move {
            while let Some(Ok(message)) = receiver.next().await {
                if let Message::Text(text) = message {
                    debug!("received client ws text: {text}");
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        handle_client_command(client_msg, &state).await;
                    }
                }
            }
        })
    };

    tokio::select! {
        _ = (&mut forward_task) => recv_task.abort(),
        _ = (&mut recv_task) => forward_task.abort(),
    }
}

async fn handle_client_command(command: ClientMessage, state: &Arc<AppState>) {
    match command {
        ClientMessage::GetState => {
            let s = state.current_companion_state().await;
            let _ = state.broadcast_tx.send(ServerMessage::State(s));
        }
        ClientMessage::Ping => {
            let _ = state.broadcast_tx.send(ServerMessage::Pong);
        }
        ClientMessage::SetModsPath { path } => {
            info!("updating mods path to {path}");
            let mut cfg = state.config.write().await;
            if let Err(e) = cfg.set_mods_path(&path) {
                let _ = state.broadcast_tx.send(ServerMessage::Error {
                    message: format!("cannot update mods path: {e}"),
                });
            } else {
                drop(cfg);
                let updated = state.current_companion_state().await;
                let _ = state.broadcast_tx.send(ServerMessage::State(updated));
            }
        }
        ClientMessage::Scan => {
            let mods_path = state.config.read().await.mods_path.clone();
            let mut scanner = state.scanner.lock().await;
            match scanner.scan_directory(&mods_path) {
                Ok(mods) => {
                    let total_size: u64 = mods.iter().map(|m| m.file_size_bytes).sum();
                    let summaries: Vec<LocalModSummary> = mods
                        .into_iter()
                        .map(|m| LocalModSummary {
                            file_name: m.file_name,
                            title: m.title,
                            version: m.version,
                            author: m.author,
                            hash_sha256: m.hash_sha256,
                            file_size_bytes: m.file_size_bytes,
                        })
                        .collect();

                    let total_mods = summaries.len();
                    let _ = state.broadcast_tx.send(ServerMessage::ScanResult {
                        mods: summaries,
                        total_mods,
                        total_size_bytes: total_size,
                    });
                }
                Err(e) => {
                    let _ = state.broadcast_tx.send(ServerMessage::Error {
                        message: format!("scan failed: {e}"),
                    });
                }
            }
        }
        ClientMessage::SyncServer {
            server_id,
            backend_url,
        } => {
            let state_clone = state.clone();
            tokio::spawn(async move {
                if let Err(e) = perform_sync(state_clone, server_id, backend_url, None).await {
                    error!("sync failed: {e}");
                }
            });
        }
        ClientMessage::SyncMods {
            mod_names,
            server_id,
        } => {
            let target_server = server_id.unwrap_or_else(|| "default".to_string());
            let state_clone = state.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    perform_sync(state_clone, target_server, None, Some(mod_names)).await
                {
                    error!("targeted mod sync failed: {e}");
                }
            });
        }
        ClientMessage::LaunchGame { server_address } => {
            match launch_fs25(server_address.as_deref()) {
                Ok(_) => {
                    let _ = state.broadcast_tx.send(ServerMessage::GameLaunched {
                        success: true,
                        message: "Farming Simulator 25 launched".to_string(),
                    });
                }
                Err(e) => {
                    let _ = state.broadcast_tx.send(ServerMessage::GameLaunched {
                        success: false,
                        message: format!("cannot launch FS25: {e}"),
                    });
                }
            }
        }
    }
}

/// Executes full mod synchronization with remote backend.
pub async fn perform_sync(
    state: Arc<AppState>,
    server_id: String,
    custom_backend_url: Option<String>,
    filter_mod_names: Option<Vec<String>>,
) -> Result<()> {
    // Guard against concurrent sync runs
    if state
        .sync_in_progress
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        warn!("synchronization already in progress, ignoring duplicate request");
        return Ok(());
    }

    state.sync_progress.store(0, Ordering::Relaxed);
    *state.current_sync_mod.write().await = None;

    // Send state update
    let cur_state = state.current_companion_state().await;
    let _ = state.broadcast_tx.send(ServerMessage::State(cur_state));

    let (mods_path, backend_url, api_token) = {
        let cfg = state.config.read().await;
        (
            cfg.mods_path.clone(),
            custom_backend_url.unwrap_or_else(|| cfg.backend_url.clone()),
            cfg.api_token.clone(),
        )
    };

    // 1. Fetch remote manifest
    let manifest = match fetch_manifest(
        &state.http_client,
        &backend_url,
        &server_id,
        api_token.as_deref(),
    )
    .await
    {
        Ok(m) => m,
        Err(err) => {
            state.sync_in_progress.store(false, Ordering::Relaxed);
            let _ = state.broadcast_tx.send(ServerMessage::Error {
                message: format!("failed fetching manifest: {err}"),
            });
            let cur_state = state.current_companion_state().await;
            let _ = state.broadcast_tx.send(ServerMessage::State(cur_state));
            return Err(err);
        }
    };

    // 2. Scan local files
    let local_mods: Vec<LocalMod> = {
        let mut scanner = state.scanner.lock().await;
        scanner.scan_directory(&mods_path).unwrap_or_default()
    };

    // 3. Diff analysis
    let diff = compute_manifest_diff(&manifest, &local_mods);
    let mut mods_to_download = Vec::new();
    mods_to_download.extend(diff.outdated);
    mods_to_download.extend(diff.missing);

    // Apply optional name filter if specified
    if let Some(names) = filter_mod_names {
        mods_to_download.retain(|m| names.iter().any(|n| n.eq_ignore_ascii_case(&m.file_name)));
    }

    let total_count = mods_to_download.len();
    let total_bytes: u64 = mods_to_download
        .iter()
        .map(|m| m.file_size_bytes.max(0) as u64)
        .sum();

    let _ = state.broadcast_tx.send(ServerMessage::SyncStarted {
        total_mods: total_count,
        total_bytes,
    });

    let (progress_tx, mut progress_rx) = mpsc::channel(64);
    let downloader = ModDownloader::new(state.http_client.clone(), 3);

    // Monitor progress events and re-broadcast
    let state_progress_ref = state.clone();
    let monitor_task = tokio::spawn(async move {
        while let Some(event) = progress_rx.recv().await {
            match event {
                DownloadProgressEvent::Started { file_name, .. } => {
                    *state_progress_ref.current_sync_mod.write().await = Some(file_name);
                }
                DownloadProgressEvent::Progress {
                    file_name,
                    mod_name,
                    downloaded_bytes,
                    file_size_bytes,
                    overall_percentage,
                    ..
                } => {
                    let pct_u8 = overall_percentage as u8;
                    state_progress_ref
                        .sync_progress
                        .store(pct_u8, Ordering::Relaxed);

                    let mod_pct = if file_size_bytes > 0 {
                        ((downloaded_bytes as f64 / file_size_bytes as f64) * 100.0) as u8
                    } else {
                        100
                    };

                    let _ = state_progress_ref
                        .broadcast_tx
                        .send(ServerMessage::SyncProgress {
                            mod_name,
                            file_name,
                            progress: mod_pct,
                            overall_progress: pct_u8,
                            downloaded_bytes,
                            total_bytes,
                        });
                }
                DownloadProgressEvent::Completed { .. } => {}
                DownloadProgressEvent::Failed {
                    file_name, error, ..
                } => {
                    warn!("download failed for {file_name}: {error}");
                }
                DownloadProgressEvent::AllDone { .. } => break,
            }
        }
    });

    let download_result = downloader
        .download_mods(mods_to_download, &mods_path, Some(progress_tx))
        .await;

    let _ = monitor_task.await;

    // Conclude synchronization
    let now_str = chrono_or_fallback();
    *state.last_sync_at.write().await = Some(now_str.clone());
    state.sync_in_progress.store(false, Ordering::Relaxed);
    state.sync_progress.store(100, Ordering::Relaxed);
    *state.current_sync_mod.write().await = None;

    let (succeeded, failed, errors) = match download_result {
        Ok(summary) => (summary.succeeded, summary.failed, summary.errors),
        Err(e) => (0, total_count, vec![e.to_string()]),
    };

    let _ = state.broadcast_tx.send(ServerMessage::SyncCompleted {
        synced_count: succeeded,
        failed_count: failed,
        errors,
        last_sync_at: now_str,
    });

    let final_state = state.current_companion_state().await;
    let _ = state.broadcast_tx.send(ServerMessage::State(final_state));

    info!(
        "sync routine completed: {} succeeded, {} failed",
        succeeded, failed
    );
    Ok(())
}

fn chrono_or_fallback() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("timestamp:{}", now)
}
