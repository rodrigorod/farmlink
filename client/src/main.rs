//! FarmLink Companion CLI executable.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use farmlink_client::config::Config;
use farmlink_client::downloader::DownloadProgressEvent;
use farmlink_client::fs25::ModScanner;
use farmlink_client::launcher::{launch_fs25, parse_join_url};
use farmlink_client::manifest::{compute_manifest_diff, fetch_manifest};
use farmlink_client::server::run_server;

#[derive(Parser)]
#[command(
    name = "farmlink-companion",
    author = "FarmLink Team",
    version = farmlink_client::VERSION,
    about = "Native high-performance companion app for Farming Simulator 25 mod synchronization"
)]
struct Cli {
    /// Logging verbosity level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info", global = true)]
    log_level: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the local companion daemon (WebSocket & HTTP IPC on port 45823)
    Daemon {
        /// Port to bind the companion service
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// Scan local FS25 mods directory and display detected mods
    Scan {
        /// Custom path to scan instead of configured directory
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Synchronize local mods directly with a FarmLink server manifest
    Sync {
        /// Server ID or slug to synchronize
        server_id: String,

        /// Custom backend API URL
        #[arg(short, long)]
        backend_url: Option<String>,
    },

    /// Display companion status, detected paths, and environment
    Status,

    /// Manage persistent configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Launch Farming Simulator 25, optionally connecting directly to a server
    Join {
        /// Server address (IP:port) or custom farmlink:// URL
        target: String,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Update the Farming Simulator 25 mods folder path
    SetModsPath { path: PathBuf },

    /// Update the FarmLink backend API endpoint
    SetBackend { url: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize structured logging
    let level = match cli.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("setting default tracing subscriber")?;

    let mut config = Config::load().context("loading configuration")?;

    match cli.command.unwrap_or(Commands::Daemon { port: None }) {
        Commands::Daemon { port } => {
            if let Some(p) = port {
                config.port = p;
            }
            config.ensure_mods_directory().ok();

            println!("============================================================");
            println!("  FarmLink Companion v{} (Rust)", farmlink_client::VERSION);
            println!("  Mods Directory: {}", config.mods_path.display());
            println!("  WebSocket IPC : ws://127.0.0.1:{}/ws", config.port);
            println!("  HTTP Status   : http://127.0.0.1:{}/status", config.port);
            println!("============================================================");

            run_server(config)
                .await
                .context("running companion server")?;
        }

        Commands::Scan { path } => {
            let target_path = path.unwrap_or(config.mods_path);
            println!("Scanning mods directory: {}\n", target_path.display());

            let mut scanner = ModScanner::new();
            let mods = scanner
                .scan_directory(&target_path)
                .context("scanning directory")?;

            if mods.is_empty() {
                println!("No .zip mods found in {}", target_path.display());
                return Ok(());
            }

            println!(
                "{:<35} {:<25} {:<12} {:>10} {:<64}",
                "FILE NAME", "TITLE", "VERSION", "SIZE", "SHA-256 HASH"
            );
            println!("{}", "-".repeat(150));

            let mut total_size = 0u64;
            for m in &mods {
                total_size += m.file_size_bytes;
                let size_mb = (m.file_size_bytes as f64) / (1024.0 * 1024.0);
                println!(
                    "{:<35} {:<25} {:<12} {:>8.1} MB {:<64}",
                    truncate_str(&m.file_name, 34),
                    truncate_str(&m.title, 24),
                    truncate_str(&m.version, 11),
                    size_mb,
                    m.hash_sha256
                );
            }

            println!("{}", "-".repeat(150));
            let total_mb = (total_size as f64) / (1024.0 * 1024.0);
            println!(
                "Total: {} mods verified | Total Size: {:.2} MB ({:.2} GB)\n",
                mods.len(),
                total_mb,
                total_mb / 1024.0
            );
        }

        Commands::Sync {
            server_id,
            backend_url,
        } => {
            let backend = backend_url.unwrap_or_else(|| config.backend_url.clone());
            println!(
                "Synchronizing with server '{}' via {}\n",
                server_id, backend
            );

            let http_client = reqwest::Client::new();
            let manifest = fetch_manifest(
                &http_client,
                &backend,
                &server_id,
                config.api_token.as_deref(),
            )
            .await
            .context("fetching server manifest")?;

            let mut scanner = ModScanner::new();
            let local_mods = scanner
                .scan_directory(&config.mods_path)
                .context("scanning local mods")?;

            let diff = compute_manifest_diff(&manifest, &local_mods);

            println!(
                "Modpack: {} ({})",
                manifest.server_name, manifest.modpack_version
            );
            println!("  [OK] Already Synced : {}", diff.synced.len());
            println!("  [!]  Outdated Mods  : {}", diff.outdated.len());
            println!("  [-]  Missing Mods   : {}", diff.missing.len());
            println!("  [+]  Extra Mods     : {}", diff.extra.len());
            println!(
                "  Download Required   : {:.2} MB\n",
                (diff.total_download_bytes as f64) / (1024.0 * 1024.0)
            );

            if diff.is_up_to_date() {
                println!("All server mods are cryptographically up to date! Ready to farm.");
                return Ok(());
            }

            let mut to_download = Vec::new();
            to_download.extend(diff.outdated);
            to_download.extend(diff.missing);

            let (tx, mut rx) = tokio::sync::mpsc::channel(32);
            let downloader = farmlink_client::downloader::ModDownloader::new(http_client, 3);
            let mods_path = config.mods_path.clone();

            let dl_task = tokio::spawn(async move {
                downloader
                    .download_mods(to_download, &mods_path, Some(tx))
                    .await
            });

            while let Some(event) = rx.recv().await {
                match event {
                    DownloadProgressEvent::Started { file_name, .. } => {
                        println!(" -> Downloading: {}", file_name);
                    }
                    DownloadProgressEvent::Progress {
                        file_name,
                        overall_percentage,
                        downloaded_bytes,
                        file_size_bytes,
                        ..
                    } => {
                        let cur_mb = (downloaded_bytes as f64) / (1024.0 * 1024.0);
                        let total_mb = (file_size_bytes as f64) / (1024.0 * 1024.0);
                        print!(
                            "\r    [{:<3.0}%] {} ({:.1} / {:.1} MB)       ",
                            overall_percentage, file_name, cur_mb, total_mb
                        );
                        use std::io::Write;
                        let _ = std::io::stdout().flush();
                    }
                    DownloadProgressEvent::Completed { file_name, .. } => {
                        println!("\r    [100%] {} verified and installed!    ", file_name);
                    }
                    DownloadProgressEvent::Failed {
                        file_name, error, ..
                    } => {
                        println!(
                            "\r    [ERR ] {} failed: {}                 ",
                            file_name, error
                        );
                    }
                    DownloadProgressEvent::AllDone { succeeded, failed } => {
                        println!(
                            "\nDownload session ended: {} succeeded, {} failed",
                            succeeded, failed
                        );
                    }
                }
            }

            let summary = dl_task.await??;
            if summary.failed > 0 {
                eprintln!("\nWarning: {} downloads failed.", summary.failed);
            } else {
                println!("\nAll mods synchronized successfully!");
            }
        }

        Commands::Status => {
            println!("FarmLink Companion Status");
            println!("-------------------------");
            println!("Version       : v{}-rust", farmlink_client::VERSION);
            println!("OS            : {}", std::env::consts::OS);
            println!("Mods Path     : {}", config.mods_path.display());
            println!("Path Exists?  : {}", config.mods_path.exists());
            println!("Backend URL   : {}", config.backend_url);
            println!("IPC Port      : {}", config.port);
            println!("Auto-Sync     : {}", config.auto_sync);
            println!("Config File   : {}", Config::config_file_path().display());
        }

        Commands::Config { action } => match action {
            ConfigAction::Show => {
                let json = serde_json::to_string_pretty(&config)?;
                println!("{json}");
            }
            ConfigAction::SetModsPath { path } => {
                config.set_mods_path(&path).context("saving config")?;
                println!("Updated mods path to: {}", path.display());
            }
            ConfigAction::SetBackend { url } => {
                config.backend_url = url.clone();
                config.save().context("saving config")?;
                println!("Updated backend URL to: {}", url);
            }
        },

        Commands::Join { target } => {
            let addr = if target.starts_with("farmlink://") {
                let params = parse_join_url(&target);
                params.and_then(|p| p.address)
            } else {
                Some(target)
            };

            info!("joining server: {:?}", addr);
            launch_fs25(addr.as_deref()).context("launching Farming Simulator 25")?;
            println!("Farming Simulator 25 launched with connection parameters.");
        }
    }

    Ok(())
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
