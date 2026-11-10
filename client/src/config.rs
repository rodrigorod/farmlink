//! Configuration management and FS25 path detection.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::error::Result;

/// Default backend API URL.
pub const DEFAULT_BACKEND_URL: &str = "http://localhost:8088";

/// FarmLink client runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// Absolute path to the local Farming Simulator 2025 mods directory.
    pub mods_path: PathBuf,

    /// Remote FarmLink backend API endpoint.
    pub backend_url: String,

    /// Optional user API token for authenticated downloads/sync.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,

    /// Auto-synchronization flag on server join or manifest change.
    #[serde(default)]
    pub auto_sync: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mods_path: detect_default_fs25_mods_path(),
            backend_url: DEFAULT_BACKEND_URL.to_string(),
            api_token: None,
            auto_sync: false,
        }
    }
}

impl Config {
    /// Loads configuration from standard OS config directory or creates default.
    pub fn load() -> Result<Self> {
        let path = Self::config_file_path();
        if path.exists() {
            debug!("loading config from {}", path.display());
            let content = fs::read_to_string(&path)?;
            let config: Self = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = Self::default();
            // Attempt to persist default configuration
            if let Err(e) = config.save() {
                debug!("could not persist initial default config: {e}");
            }
            Ok(config)
        }
    }

    /// Saves configuration to standard OS config file path.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_file_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        info!("saved configuration to {}", path.display());
        Ok(())
    }

    /// Updates the mods path and saves configuration.
    pub fn set_mods_path(&mut self, new_path: impl AsRef<Path>) -> Result<()> {
        let p = new_path.as_ref().to_path_buf();
        self.mods_path = p;
        self.save()
    }

    /// Returns the standard path to the config file `~/.config/farmlink/config.json`.
    pub fn config_file_path() -> PathBuf {
        let base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base_dir.join("farmlink").join("config.json")
    }

    /// Ensures that the configured mods directory exists on disk.
    pub fn ensure_mods_directory(&self) -> Result<()> {
        if !self.mods_path.exists() {
            fs::create_dir_all(&self.mods_path)?;
            info!("created mods directory at {}", self.mods_path.display());
        }
        Ok(())
    }
}

/// Automatically detects the default Farming Simulator 2025 mods directory
/// based on the host operating system and common Steam Proton paths.
pub fn detect_default_fs25_mods_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(docs) = dirs::document_dir() {
            let win_path = docs
                .join("My Games")
                .join("FarmingSimulator2025")
                .join("mods");
            if win_path.exists() {
                return win_path;
            }
            return docs
                .join("My Games")
                .join("FarmingSimulator2025")
                .join("mods");
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            // 1. Check Steam Proton compatdata for FS25 (Steam AppID 2300320)
            let proton_candidates = [
                home.join(".steam/steam/steamapps/compatdata/2300320/pfx/drive_c/users/steamuser/Documents/My Games/FarmingSimulator2025/mods"),
                home.join(".local/share/Steam/steamapps/compatdata/2300320/pfx/drive_c/users/steamuser/Documents/My Games/FarmingSimulator2025/mods"),
                home.join(".var/app/com.valvesoftware.Steam/.steam/steam/steamapps/compatdata/2300320/pfx/drive_c/users/steamuser/Documents/My Games/FarmingSimulator2025/mods"),
            ];

            for candidate in proton_candidates {
                if candidate.exists() {
                    return candidate;
                }
            }

            // 2. Standard XDG local share location
            let xdg_path = home.join(".local/share/FarmingSimulator2025/mods");
            return xdg_path;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            return home.join("Library/Application Support/FarmingSimulator2025/mods");
        }
    }

    PathBuf::from("mods")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_config_has_valid_fields() {
        let cfg = Config::default();
        assert_eq!(cfg.backend_url, DEFAULT_BACKEND_URL);
        assert!(!cfg.auto_sync);
        assert!(cfg.api_token.is_none());
        assert!(!cfg.mods_path.as_os_str().is_empty());
    }

    #[test]
    fn serialization_roundtrip_works() {
        let cfg = Config {
            auto_sync: true,
            api_token: Some("secret-token-123".to_string()),
            mods_path: PathBuf::from("/custom/mods/path"),
            ..Config::default()
        };

        let serialized = serde_json::to_string(&cfg).expect("serialize");
        let deserialized: Config = serde_json::from_str(&serialized).expect("deserialize");

        assert_eq!(cfg, deserialized);
    }

    #[test]
    fn ensure_mods_directory_creates_dir() {
        let temp = tempdir().expect("tempdir");
        let mods_dir = temp.path().join("sub").join("mods");
        let cfg = Config {
            mods_path: mods_dir.clone(),
            ..Config::default()
        };

        assert!(!mods_dir.exists());
        cfg.ensure_mods_directory().expect("ensure mods directory");
        assert!(mods_dir.exists());
        assert!(mods_dir.is_dir());
    }
}
