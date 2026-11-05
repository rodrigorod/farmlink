//! Server manifest representation, API communication, and local differential analysis.

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::error::{ClientError, Result};
use crate::fs25::LocalMod;

/// DTO for a mod entry in the server's modpack manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestModDto {
    pub id: String,
    pub mod_name: String,
    pub file_name: String,
    pub version: String,
    pub hash_sha256: String,
    pub file_size_bytes: i64,
    pub download_url: String,
    pub is_active: bool,
}

/// Full modpack manifest payload returned by FarmLink backend `/api/v1/servers/:id/manifest`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestResponse {
    pub server_id: String,
    pub server_name: String,
    pub modpack_version: String,
    pub generated_at: String,
    pub total_mods: usize,
    pub total_size_bytes: i64,
    pub mods: Vec<ManifestModDto>,
}

/// Mod synchronization state category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModSyncStatus {
    /// Local file has exact cryptographic SHA-256 match.
    Synced,
    /// Local file exists under same name/title but hash or version differs.
    Outdated,
    /// Mod is listed in server manifest but not present in local mods folder.
    Missing,
    /// Local mod is present on disk but not part of this server modpack.
    Extra,
}

/// Differential comparison between a server manifest and local mod files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestDiff {
    /// Mods already present and cryptographically matching.
    pub synced: Vec<ManifestModDto>,

    /// Mods that exist locally but have a different version or hash.
    pub outdated: Vec<ManifestModDto>,

    /// Mods required by the server that are missing locally.
    pub missing: Vec<ManifestModDto>,

    /// Local files not included in this server modpack.
    pub extra: Vec<LocalMod>,

    /// Total download size in bytes required to bring the client up to date.
    pub total_download_bytes: i64,
}

impl ManifestDiff {
    /// Returns true if all server mods are locally synced and no downloads are needed.
    pub fn is_up_to_date(&self) -> bool {
        self.missing.is_empty() && self.outdated.is_empty()
    }

    /// Returns the total number of mods that must be downloaded.
    pub fn required_download_count(&self) -> usize {
        self.missing.len() + self.outdated.len()
    }
}

/// Fetches the manifest for a given server ID from the FarmLink backend.
pub async fn fetch_manifest(
    client: &reqwest::Client,
    backend_url: &str,
    server_id: &str,
    api_token: Option<&str>,
) -> Result<ManifestResponse> {
    let url = format!(
        "{}/api/v1/servers/{}/manifest",
        backend_url.trim_end_matches('/'),
        server_id
    );
    debug!("requesting manifest from {url}");

    let mut req = client.get(&url);
    if let Some(token) = api_token {
        req = req.bearer_auth(token);
    }

    let resp = req.send().await?;
    let status = resp.status();
    if !status.is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(ClientError::ServerApiError {
            status: status.as_u16(),
            message: msg,
        });
    }

    let manifest = resp.json::<ManifestResponse>().await?;
    info!(
        "retrieved manifest for '{}': {} mods (modpack {})",
        manifest.server_name, manifest.total_mods, manifest.modpack_version
    );
    Ok(manifest)
}

/// Compares a server manifest against scanned local mods to determine sync state.
pub fn compute_manifest_diff(manifest: &ManifestResponse, local_mods: &[LocalMod]) -> ManifestDiff {
    let mut synced = Vec::new();
    let mut outdated = Vec::new();
    let mut missing = Vec::new();
    let mut total_download_bytes = 0i64;

    // Track which local mods are matched by manifest entries
    let mut matched_local_indices = std::collections::HashSet::new();

    for remote_mod in &manifest.mods {
        if !remote_mod.is_active {
            continue;
        }

        // Match local mod by file name
        if let Some((idx, local_mod)) = local_mods
            .iter()
            .enumerate()
            .find(|(_, l)| l.file_name.eq_ignore_ascii_case(&remote_mod.file_name))
        {
            matched_local_indices.insert(idx);
            if local_mod.hash_sha256.eq_ignore_ascii_case(&remote_mod.hash_sha256) {
                synced.push(remote_mod.clone());
            } else {
                outdated.push(remote_mod.clone());
                total_download_bytes += remote_mod.file_size_bytes;
            }
        } else {
            missing.push(remote_mod.clone());
            total_download_bytes += remote_mod.file_size_bytes;
        }
    }

    // Local mods not referenced in the manifest are considered extra
    let extra = local_mods
        .iter()
        .enumerate()
        .filter(|(idx, _)| !matched_local_indices.contains(idx))
        .map(|(_, l)| l.clone())
        .collect();

    ManifestDiff {
        synced,
        outdated,
        missing,
        extra,
        total_download_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_manifest() -> ManifestResponse {
        ManifestResponse {
            server_id: "srv-01".to_string(),
            server_name: "Zielonka Farm".to_string(),
            modpack_version: "v2.4".to_string(),
            generated_at: "2026-11-05T12:00:00Z".to_string(),
            total_mods: 3,
            total_size_bytes: 231900000,
            mods: vec![
                ManifestModDto {
                    id: "mod-1".to_string(),
                    mod_name: "John Deere 7R".to_string(),
                    file_name: "FS25_JohnDeere_7R.zip".to_string(),
                    version: "1.2.0.0".to_string(),
                    hash_sha256: "hash_jd_perfect".to_string(),
                    file_size_bytes: 84500000,
                    download_url: "http://storage/jd.zip".to_string(),
                    is_active: true,
                },
                ManifestModDto {
                    id: "mod-2".to_string(),
                    mod_name: "Courseplay".to_string(),
                    file_name: "FS25_Courseplay.zip".to_string(),
                    version: "7.4.1.0".to_string(),
                    hash_sha256: "hash_cp_new".to_string(),
                    file_size_bytes: 15400000,
                    download_url: "http://storage/cp.zip".to_string(),
                    is_active: true,
                },
                ManifestModDto {
                    id: "mod-3".to_string(),
                    mod_name: "Claas Lexion 8900".to_string(),
                    file_name: "FS25_ClaasLexion8900.zip".to_string(),
                    version: "1.0.2.0".to_string(),
                    hash_sha256: "hash_claas_new".to_string(),
                    file_size_bytes: 132000000,
                    download_url: "http://storage/claas.zip".to_string(),
                    is_active: true,
                },
            ],
        }
    }

    #[test]
    fn correctly_identifies_synced_outdated_missing_extra() {
        let manifest = sample_manifest();
        let local_mods = vec![
            // Perfectly synced John Deere
            LocalMod {
                file_name: "FS25_JohnDeere_7R.zip".to_string(),
                title: "John Deere 7R".to_string(),
                version: "1.2.0.0".to_string(),
                author: "GIANTS".to_string(),
                hash_sha256: "hash_jd_perfect".to_string(),
                file_size_bytes: 84500000,
                modified_at: 1000,
                path: PathBuf::from("/mods/FS25_JohnDeere_7R.zip"),
            },
            // Outdated Courseplay (same filename, old hash)
            LocalMod {
                file_name: "FS25_Courseplay.zip".to_string(),
                title: "Courseplay".to_string(),
                version: "7.3.0.0".to_string(),
                author: "CP Team".to_string(),
                hash_sha256: "hash_cp_old".to_string(),
                file_size_bytes: 14000000,
                modified_at: 2000,
                path: PathBuf::from("/mods/FS25_Courseplay.zip"),
            },
            // Extra mod not in server pack
            LocalMod {
                file_name: "FS25_RandomTruck.zip".to_string(),
                title: "Random Truck".to_string(),
                version: "1.0.0.0".to_string(),
                author: "RandomModder".to_string(),
                hash_sha256: "hash_random".to_string(),
                file_size_bytes: 5000000,
                modified_at: 3000,
                path: PathBuf::from("/mods/FS25_RandomTruck.zip"),
            },
        ];

        let diff = compute_manifest_diff(&manifest, &local_mods);

        assert_eq!(diff.synced.len(), 1);
        assert_eq!(diff.synced[0].file_name, "FS25_JohnDeere_7R.zip");

        assert_eq!(diff.outdated.len(), 1);
        assert_eq!(diff.outdated[0].file_name, "FS25_Courseplay.zip");

        assert_eq!(diff.missing.len(), 1);
        assert_eq!(diff.missing[0].file_name, "FS25_ClaasLexion8900.zip");

        assert_eq!(diff.extra.len(), 1);
        assert_eq!(diff.extra[0].file_name, "FS25_RandomTruck.zip");

        assert_eq!(diff.total_download_bytes, 15400000 + 132000000);
        assert_eq!(diff.required_download_count(), 2);
        assert!(!diff.is_up_to_date());
    }
}
