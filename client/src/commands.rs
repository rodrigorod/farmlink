//! Tauri IPC command handlers for native frontend communication.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::SystemTime;

use tauri::Emitter;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{info, warn};

use crate::config::Config;
use crate::downloader::{DownloadProgressEvent, ModDownloader};
use crate::fs25::{LocalMod, ModScanner};
use crate::launcher::{launch_fs25, parse_join_url};
use crate::manifest::{compute_manifest_diff, fetch_manifest};
use crate::protocol::{
    CompanionState, LocalModSummary, ScanResultPayload, SyncCompletedPayload, SyncProgressPayload,
};

/// Shared companion application state managed by Tauri.
pub struct AppState {
    pub config: RwLock<Config>,
    pub scanner: Mutex<ModScanner>,
    pub sync_in_progress: AtomicBool,
    pub sync_progress: AtomicU8,
    pub current_sync_mod: RwLock<Option<String>>,
    pub last_sync_at: RwLock<Option<String>>,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(config: Config) -> Self {
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
        }
    }

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

/// Scans local FS25 mods directory and returns metadata summaries.
#[tauri::command]
pub async fn scan_mods(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    mods_path: Option<String>,
) -> Result<Vec<LocalModSummary>, String> {
    let target_path = match mods_path {
        Some(p) if !p.trim().is_empty() => PathBuf::from(p),
        _ => state.config.read().await.mods_path.clone(),
    };

    let mut scanner = state.scanner.lock().await;
    let mods = scanner
        .scan_directory(&target_path)
        .map_err(|e| format!("failed scanning {}: {e}", target_path.display()))?;

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

    let total_size: u64 = summaries.iter().map(|m| m.file_size_bytes).sum();
    let _ = app.emit(
        "scan_result",
        ScanResultPayload {
            mods: summaries.clone(),
            total_mods: summaries.len(),
            total_size_bytes: total_size,
        },
    );

    Ok(summaries)
}

/// Updates and persists the local mods directory path.
#[tauri::command]
pub async fn set_mods_path(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<String, String> {
    let p = PathBuf::from(&path);
    let mut cfg = state.config.write().await;
    cfg.set_mods_path(&p).map_err(|e| e.to_string())?;
    Ok(format!("mods directory configured to {}", p.display()))
}

/// Launches Farming Simulator 25, optionally targeting a dedicated server.
#[tauri::command]
pub async fn launch_game(target: Option<String>) -> Result<String, String> {
    let addr = if let Some(ref t) = target {
        if t.starts_with("farmlink://") {
            parse_join_url(t).and_then(|p| p.address)
        } else {
            Some(t.clone())
        }
    } else {
        None
    };

    info!("launching game targeting: {:?}", addr);
    launch_fs25(addr.as_deref()).map_err(|e| e.to_string())?;
    Ok("Farming Simulator 25 launched successfully".to_string())
}

/// Retrieves the current companion state.
#[tauri::command]
pub async fn get_companion_state(
    state: tauri::State<'_, AppState>,
) -> Result<CompanionState, String> {
    Ok(state.current_companion_state().await)
}

/// Synchronizes mods with a remote dedicated server manifest.
#[tauri::command]
pub async fn sync_server(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    server_id: String,
    backend_url: Option<String>,
    mod_names: Option<Vec<String>>,
) -> Result<SyncCompletedPayload, String> {
    if state
        .sync_in_progress
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        warn!("synchronization already in progress, ignoring duplicate call");
        return Err("synchronization already in progress".to_string());
    }

    state.sync_progress.store(0, Ordering::Relaxed);
    *state.current_sync_mod.write().await = None;

    let (mods_path, backend, api_token) = {
        let cfg = state.config.read().await;
        (
            cfg.mods_path.clone(),
            backend_url.unwrap_or_else(|| cfg.backend_url.clone()),
            cfg.api_token.clone(),
        )
    };

    // 1. Fetch remote manifest
    let manifest = match fetch_manifest(
        &state.http_client,
        &backend,
        &server_id,
        api_token.as_deref(),
    )
    .await
    {
        Ok(m) => m,
        Err(err) => {
            state.sync_in_progress.store(false, Ordering::Relaxed);
            return Err(format!("failed fetching manifest: {err}"));
        }
    };

    // 2. Scan local files
    let local_mods: Vec<LocalMod> = {
        let mut scanner = state.scanner.lock().await;
        scanner.scan_directory(&mods_path).unwrap_or_default()
    };

    // 3. Diff computation
    let diff = compute_manifest_diff(&manifest, &local_mods);
    let mut mods_to_download = Vec::new();
    mods_to_download.extend(diff.outdated);
    mods_to_download.extend(diff.missing);

    if let Some(names) = mod_names {
        if !names.is_empty() {
            mods_to_download.retain(|m| names.iter().any(|n| n.eq_ignore_ascii_case(&m.file_name)));
        }
    }

    let total_count = mods_to_download.len();
    let total_bytes: u64 = mods_to_download
        .iter()
        .map(|m| m.file_size_bytes.max(0) as u64)
        .sum();

    let (progress_tx, mut progress_rx) = mpsc::channel(64);
    let downloader = ModDownloader::new(state.http_client.clone(), 3);

    let app_handle_ref = app.clone();
    let monitor_task = tokio::spawn(async move {
        while let Some(event) = progress_rx.recv().await {
            match event {
                DownloadProgressEvent::Started { file_name, .. } => {
                    let _ = app_handle_ref.emit(
                        "sync_progress",
                        SyncProgressPayload {
                            mod_name: file_name.clone(),
                            file_name,
                            progress: 0,
                            overall_progress: 0,
                            downloaded_bytes: 0,
                            total_bytes,
                        },
                    );
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
                    let mod_pct = if file_size_bytes > 0 {
                        ((downloaded_bytes as f64 / file_size_bytes as f64) * 100.0) as u8
                    } else {
                        100
                    };

                    let _ = app_handle_ref.emit(
                        "sync_progress",
                        SyncProgressPayload {
                            mod_name,
                            file_name,
                            progress: mod_pct,
                            overall_progress: pct_u8,
                            downloaded_bytes,
                            total_bytes,
                        },
                    );
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

    let now_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let now_str = format!("timestamp:{}", now_secs);

    *state.last_sync_at.write().await = Some(now_str.clone());
    state.sync_in_progress.store(false, Ordering::Relaxed);
    state.sync_progress.store(100, Ordering::Relaxed);
    *state.current_sync_mod.write().await = None;

    let (succeeded, failed, errors) = match download_result {
        Ok(summary) => (summary.succeeded, summary.failed, summary.errors),
        Err(e) => (0, total_count, vec![e.to_string()]),
    };

    let completed_payload = SyncCompletedPayload {
        synced_count: succeeded,
        failed_count: failed,
        errors,
        last_sync_at: now_str,
    };

    let _ = app.emit("sync_completed", &completed_payload);
    Ok(completed_payload)
}
