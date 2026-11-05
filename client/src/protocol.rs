//! IPC communication protocol schemas between the web frontend and Rust companion.

use serde::{Deserialize, Serialize};

/// High-level companion runtime state shared with the frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanionState {
    pub is_connected: bool,
    pub version: String,
    pub os: String,
    pub mods_path: String,
    pub last_sync_at: Option<String>,
    pub sync_in_progress: bool,
    pub sync_progress: u8,
    pub current_sync_mod: Option<String>,
    pub simulation_mode: bool,
}

/// Compact representation of a local mod for web UI listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalModSummary {
    pub file_name: String,
    pub title: String,
    pub version: String,
    pub author: String,
    pub hash_sha256: String,
    pub file_size_bytes: u64,
}

/// Event payload for download progress.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncProgressPayload {
    pub mod_name: String,
    pub file_name: String,
    pub progress: u8,
    pub overall_progress: u8,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

/// Event payload when synchronization finishes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncCompletedPayload {
    pub synced_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
    pub last_sync_at: String,
}

/// Event payload when mod directory scan finishes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResultPayload {
    pub mods: Vec<LocalModSummary>,
    pub total_mods: usize,
    pub total_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_companion_state() {
        let state = CompanionState {
            is_connected: true,
            version: "v0.4.2-rust".to_string(),
            os: "linux".to_string(),
            mods_path: "/test/mods".to_string(),
            last_sync_at: None,
            sync_in_progress: false,
            sync_progress: 0,
            current_sync_mod: None,
            simulation_mode: false,
        };

        let json = serde_json::to_string(&state).expect("serialize");
        assert!(json.contains("v0.4.2-rust"));
    }
}
