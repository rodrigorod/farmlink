//! Asynchronous streaming downloader with cryptographic verification and atomic file updates.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Semaphore};
use tracing::{error, info};

use crate::error::{ClientError, Result};
use crate::manifest::ManifestModDto;

/// Maximum concurrent active downloads.
pub const DEFAULT_MAX_CONCURRENT_DOWNLOADS: usize = 3;

/// Progress event emitted during synchronization.
#[derive(Debug, Clone)]
pub enum DownloadProgressEvent {
    Started {
        file_name: String,
        mod_name: String,
        file_size_bytes: i64,
    },
    Progress {
        file_name: String,
        mod_name: String,
        downloaded_bytes: u64,
        file_size_bytes: i64,
        overall_downloaded_bytes: u64,
        overall_total_bytes: u64,
        overall_percentage: f64,
    },
    Completed {
        file_name: String,
        mod_name: String,
    },
    Failed {
        file_name: String,
        mod_name: String,
        error: String,
    },
    AllDone {
        succeeded: usize,
        failed: usize,
    },
}

/// Downloader engine for batch acquiring and verifying mod files.
pub struct ModDownloader {
    client: reqwest::Client,
    max_concurrency: usize,
}

impl ModDownloader {
    pub fn new(client: reqwest::Client, max_concurrency: usize) -> Self {
        Self {
            client,
            max_concurrency: if max_concurrency == 0 {
                DEFAULT_MAX_CONCURRENT_DOWNLOADS
            } else {
                max_concurrency
            },
        }
    }

    /// Downloads a list of mods into `target_dir` emitting progress events.
    pub async fn download_mods(
        &self,
        mods_to_download: Vec<ManifestModDto>,
        target_dir: &Path,
        progress_tx: Option<mpsc::Sender<DownloadProgressEvent>>,
    ) -> Result<DownloadSummary> {
        let overall_total_bytes: u64 = mods_to_download
            .iter()
            .map(|m| m.file_size_bytes.max(0) as u64)
            .sum();

        let overall_downloaded_bytes = Arc::new(AtomicU64::new(0));
        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));

        let mut tasks = Vec::new();

        for mod_dto in mods_to_download {
            let client = self.client.clone();
            let sem = semaphore.clone();
            let target_path = target_dir.join(&mod_dto.file_name);
            let overall_bytes = overall_downloaded_bytes.clone();
            let tx = progress_tx.clone();

            tasks.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.map_err(|e| {
                    ClientError::SyncFailed(format!("semaphore acquisition error: {e}"))
                })?;

                if let Some(ref sender) = tx {
                    let _ = sender
                        .send(DownloadProgressEvent::Started {
                            file_name: mod_dto.file_name.clone(),
                            mod_name: mod_dto.mod_name.clone(),
                            file_size_bytes: mod_dto.file_size_bytes,
                        })
                        .await;
                }

                let result = download_single_mod_verified(
                    &client,
                    &mod_dto,
                    &target_path,
                    overall_bytes,
                    overall_total_bytes,
                    tx.as_ref(),
                )
                .await;

                match &result {
                    Ok(_) => {
                        if let Some(ref sender) = tx {
                            let _ = sender
                                .send(DownloadProgressEvent::Completed {
                                    file_name: mod_dto.file_name.clone(),
                                    mod_name: mod_dto.mod_name.clone(),
                                })
                                .await;
                        }
                    }
                    Err(err) => {
                        error!("failed downloading {}: {err}", mod_dto.file_name);
                        if let Some(ref sender) = tx {
                            let _ = sender
                                .send(DownloadProgressEvent::Failed {
                                    file_name: mod_dto.file_name.clone(),
                                    mod_name: mod_dto.mod_name.clone(),
                                    error: err.to_string(),
                                })
                                .await;
                        }
                    }
                }

                result
            }));
        }

        let mut succeeded = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for task in tasks {
            match task.await {
                Ok(Ok(_)) => succeeded += 1,
                Ok(Err(e)) => {
                    failed += 1;
                    errors.push(e.to_string());
                }
                Err(join_err) => {
                    failed += 1;
                    errors.push(format!("task join error: {join_err}"));
                }
            }
        }

        if let Some(ref sender) = progress_tx {
            let _ = sender
                .send(DownloadProgressEvent::AllDone { succeeded, failed })
                .await;
        }

        info!(
            "download finished: {} succeeded, {} failed (total bytes: {})",
            succeeded, failed, overall_total_bytes
        );

        Ok(DownloadSummary {
            succeeded,
            failed,
            total_bytes: overall_total_bytes,
            errors,
        })
    }
}

/// Execution summary of batch downloads.
#[derive(Debug, Clone)]
pub struct DownloadSummary {
    pub succeeded: usize,
    pub failed: usize,
    pub total_bytes: u64,
    pub errors: Vec<String>,
}

/// Downloads a single mod file into a temporary file, computes its SHA-256 digest,
/// validates the hash against the manifest, and atomically renames to the destination.
async fn download_single_mod_verified(
    client: &reqwest::Client,
    mod_dto: &ManifestModDto,
    destination: &Path,
    overall_downloaded: Arc<AtomicU64>,
    overall_total: u64,
    progress_tx: Option<&mpsc::Sender<DownloadProgressEvent>>,
) -> Result<()> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_destination = destination.with_extension(format!("farmlink_tmp_{nonce}"));

    info!(
        "starting download for '{}' from {}",
        mod_dto.file_name, mod_dto.download_url
    );

    let res = async {
        let response = client.get(&mod_dto.download_url).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(ClientError::ServerApiError {
                status: status.as_u16(),
                message: format!("HTTP {} downloading mod", status),
            });
        }

        let mut stream = response.bytes_stream();
        let mut file = File::create(&temp_destination).await?;
        let mut hasher = Sha256::new();
        let mut downloaded_bytes = 0u64;

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res?;
            file.write_all(&chunk).await?;
            hasher.update(&chunk);

            let chunk_len = chunk.len() as u64;
            downloaded_bytes += chunk_len;
            let cur_overall = overall_downloaded.fetch_add(chunk_len, Ordering::Relaxed) + chunk_len;

            if let Some(tx) = progress_tx {
                let pct = if overall_total > 0 {
                    ((cur_overall as f64) / (overall_total as f64) * 100.0).clamp(0.0, 100.0)
                } else {
                    100.0
                };

                let _ = tx
                    .send(DownloadProgressEvent::Progress {
                        file_name: mod_dto.file_name.clone(),
                        mod_name: mod_dto.mod_name.clone(),
                        downloaded_bytes,
                        file_size_bytes: mod_dto.file_size_bytes,
                        overall_downloaded_bytes: cur_overall,
                        overall_total_bytes: overall_total,
                        overall_percentage: pct,
                    })
                    .await;
            }
        }

        file.flush().await?;
        drop(file);

        // Cryptographic validation
        let computed_hash = hex::encode(hasher.finalize());
        if !computed_hash.eq_ignore_ascii_case(&mod_dto.hash_sha256) {
            return Err(ClientError::HashMismatch {
                file: mod_dto.file_name.clone(),
                expected: mod_dto.hash_sha256.clone(),
                actual: computed_hash,
            });
        }

        // Atomic move to target filename
        fs::rename(&temp_destination, destination).await?;
        info!(
            "successfully downloaded and verified '{}' (SHA256: {})",
            mod_dto.file_name, computed_hash
        );

        Ok(())
    }
    .await;

    if res.is_err() {
        let _ = fs::remove_file(&temp_destination).await;
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn handles_empty_mod_list_gracefully() {
        let client = reqwest::Client::new();
        let downloader = ModDownloader::new(client, 2);
        let temp = tempdir().expect("tempdir");

        let summary = downloader
            .download_mods(vec![], temp.path(), None)
            .await
            .expect("empty downloads");

        assert_eq!(summary.succeeded, 0);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.total_bytes, 0);
    }
}
