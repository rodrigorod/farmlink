//! WebSocket and IPC communication protocol between the web frontend and Rust companion.

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

/// Incoming commands received from the web frontend over WebSocket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Request current companion status.
    GetState,

    /// Request re-scanning of the local mods folder.
    Scan,

    /// Synchronize local mods with a remote server modpack.
    SyncServer {
        server_id: String,
        #[serde(default)]
        backend_url: Option<String>,
    },

    /// Synchronize specific mods by name.
    SyncMods {
        mod_names: Vec<String>,
        #[serde(default)]
        server_id: Option<String>,
    },

    /// Update and persist the local mods directory path.
    SetModsPath { path: String },

    /// Launch Farming Simulator 25 and optionally join a server.
    LaunchGame {
        #[serde(default)]
        server_address: Option<String>,
    },

    /// Keep-alive ping.
    Ping,
}

/// Outgoing events pushed from the companion client to the web frontend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Current state update.
    State(CompanionState),

    /// Directory scan completed.
    ScanResult {
        mods: Vec<LocalModSummary>,
        total_mods: usize,
        total_size_bytes: u64,
    },

    /// Mod synchronization process started.
    SyncStarted { total_mods: usize, total_bytes: u64 },

    /// Real-time progress during download.
    SyncProgress {
        mod_name: String,
        file_name: String,
        progress: u8,
        overall_progress: u8,
        downloaded_bytes: u64,
        total_bytes: u64,
    },

    /// Synchronization concluded.
    SyncCompleted {
        synced_count: usize,
        failed_count: usize,
        errors: Vec<String>,
        last_sync_at: String,
    },

    /// Game process launch status.
    GameLaunched { success: bool, message: String },

    /// Generic error notification.
    Error { message: String },

    /// Keep-alive pong.
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_incoming_client_messages() {
        let json_sync = r#"{"type":"sync_server","server_id":"srv_zielonka_01"}"#;
        let msg: ClientMessage = serde_json::from_str(json_sync).expect("parse sync");
        match msg {
            ClientMessage::SyncServer {
                server_id,
                backend_url,
            } => {
                assert_eq!(server_id, "srv_zielonka_01");
                assert!(backend_url.is_none());
            }
            _ => panic!("wrong variant"),
        }

        let json_set_path = r#"{"type":"set_mods_path","path":"/custom/path"}"#;
        let msg: ClientMessage = serde_json::from_str(json_set_path).expect("parse set path");
        assert_eq!(
            msg,
            ClientMessage::SetModsPath {
                path: "/custom/path".to_string()
            }
        );
    }

    #[test]
    fn serializes_server_messages_cleanly() {
        let state_msg = ServerMessage::State(CompanionState {
            is_connected: true,
            version: "v0.4.2-rust".to_string(),
            os: "linux".to_string(),
            mods_path: "/home/user/mods".to_string(),
            last_sync_at: Some("2026-11-05T12:00:00Z".to_string()),
            sync_in_progress: false,
            sync_progress: 0,
            current_sync_mod: None,
            simulation_mode: false,
        });

        let json = serde_json::to_string(&state_msg).expect("serialize state");
        assert!(json.contains(r#""type":"state""#));
        assert!(json.contains(r#""is_connected":true"#));
        assert!(json.contains(r#""version":"v0.4.2-rust""#));
    }
}
