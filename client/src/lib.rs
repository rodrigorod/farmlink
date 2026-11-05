//! FarmLink Client Library
//!
//! Provides modpack scanning, cryptographic verification, fast asynchronous synchronization,
//! and local WebSocket / IPC services for Farming Simulator 25.

pub mod config;
pub mod downloader;
pub mod error;
pub mod fs25;
pub mod launcher;
pub mod manifest;
pub mod protocol;
pub mod server;

pub use config::Config;
pub use downloader::{DownloadProgressEvent, DownloadSummary, ModDownloader};
pub use error::{ClientError, Result};
pub use fs25::{calculate_file_sha256, extract_mod_desc_from_zip, LocalMod, ModDesc, ModScanner};
pub use launcher::{launch_fs25, parse_join_url, JoinUrlParams};
pub use manifest::{
    compute_manifest_diff, fetch_manifest, ManifestDiff, ManifestModDto, ManifestResponse,
};
pub use protocol::{ClientMessage, CompanionState, LocalModSummary, ServerMessage};
pub use server::{create_router, run_server, AppState};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
