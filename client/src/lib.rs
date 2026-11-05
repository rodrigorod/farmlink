//! FarmLink Client Library
//!
//! Provides modpack scanning, cryptographic verification, fast asynchronous synchronization,
//! and native Tauri IPC command services for Farming Simulator 25.

pub mod commands;
pub mod config;
pub mod downloader;
pub mod error;
pub mod fs25;
pub mod launcher;
pub mod manifest;
pub mod protocol;

pub use commands::AppState;
pub use config::Config;
pub use downloader::{DownloadProgressEvent, DownloadSummary, ModDownloader};
pub use error::{ClientError, Result};
pub use fs25::{calculate_file_sha256, extract_mod_desc_from_zip, LocalMod, ModDesc, ModScanner};
pub use launcher::{launch_fs25, parse_join_url, JoinUrlParams};
pub use manifest::{
    compute_manifest_diff, fetch_manifest, ManifestDiff, ManifestModDto, ManifestResponse,
};
pub use protocol::{
    CompanionState, LocalModSummary, ScanResultPayload, SyncCompletedPayload, SyncProgressPayload,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
