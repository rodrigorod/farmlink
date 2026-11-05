//! Local FS25 mod directory scanner, fast hashing, and caching.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::error::{ClientError, Result};
use crate::fs25::mod_desc::{parse_mod_desc, ModDesc};

/// Maximum allowed bytes to read from `modDesc.xml` to prevent zip-bombs (2 MiB).
const MAX_XML_READ_BYTES: u64 = 2 * 1024 * 1024;

/// Represents a detected local Farming Simulator 25 mod file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalMod {
    /// File name on disk (e.g. "FS25_JohnDeere_7R.zip").
    pub file_name: String,

    /// Title parsed from `modDesc.xml`.
    pub title: String,

    /// Version string (e.g. "1.2.0.0").
    pub version: String,

    /// Author or team credited in `modDesc.xml`.
    pub author: String,

    /// Lowercase hexadecimal SHA-256 cryptographic digest.
    pub hash_sha256: String,

    /// Exact size of the `.zip` archive in bytes.
    pub file_size_bytes: u64,

    /// Last modification timestamp (seconds since UNIX epoch).
    pub modified_at: u64,

    /// Full canonical path on disk.
    pub path: PathBuf,
}

/// Disk cache entry to avoid re-reading and re-hashing unchanged mod files.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    file_size_bytes: u64,
    modified_at: u64,
    hash_sha256: String,
    title: String,
    version: String,
    author: String,
}

/// Scanner with persistent cache for fast directory inspection.
pub struct ModScanner {
    cache: HashMap<String, CacheEntry>,
    cache_path: PathBuf,
}

impl ModScanner {
    /// Initializes a new scanner with cached metadata if available.
    pub fn new() -> Self {
        let cache_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("farmlink")
            .join("cache.json");

        let cache = Self::load_cache(&cache_path).unwrap_or_default();
        Self { cache, cache_path }
    }

    /// Loads the cache file from disk if valid.
    fn load_cache(path: &Path) -> Option<HashMap<String, CacheEntry>> {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(parsed) = serde_json::from_str(&content) {
                    return Some(parsed);
                }
            }
        }
        None
    }

    /// Persists cache entries to disk.
    pub fn save_cache(&self) {
        if let Some(parent) = self.cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(serialized) = serde_json::to_string(&self.cache) {
            let _ = fs::write(&self.cache_path, serialized);
        }
    }

    /// Scans the specified directory, reading all `.zip` mod packages.
    pub fn scan_directory(&mut self, mods_dir: &Path) -> Result<Vec<LocalMod>> {
        if !mods_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(mods_dir)?;
        let mut results = Vec::new();
        let mut cache_modified = false;

        for entry_res in entries {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    warn!("error reading directory entry: {err}");
                    continue;
                }
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.to_lowercase().ends_with(".zip") {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(err) => {
                    warn!("cannot read metadata for {}: {err}", path.display());
                    continue;
                }
            };

            let file_size_bytes = metadata.len();
            let modified_at = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            // Check cache hit
            if let Some(cached) = self.cache.get(&file_name) {
                if cached.file_size_bytes == file_size_bytes && cached.modified_at == modified_at {
                    debug!("cache hit for {}", file_name);
                    results.push(LocalMod {
                        file_name: file_name.clone(),
                        title: cached.title.clone(),
                        version: cached.version.clone(),
                        author: cached.author.clone(),
                        hash_sha256: cached.hash_sha256.clone(),
                        file_size_bytes,
                        modified_at,
                        path,
                    });
                    continue;
                }
            }

            // Cache miss: process archive
            debug!("hashing and parsing new or modified mod: {}", file_name);
            match Self::process_mod_file(&path) {
                Ok((hash_sha256, mod_desc)) => {
                    let local_mod = LocalMod {
                        file_name: file_name.clone(),
                        title: if mod_desc.title.is_empty() {
                            file_name.trim_end_matches(".zip").to_string()
                        } else {
                            mod_desc.title.clone()
                        },
                        version: mod_desc.version.clone(),
                        author: mod_desc.author.clone(),
                        hash_sha256: hash_sha256.clone(),
                        file_size_bytes,
                        modified_at,
                        path,
                    };

                    self.cache.insert(
                        file_name,
                        CacheEntry {
                            file_size_bytes,
                            modified_at,
                            hash_sha256,
                            title: local_mod.title.clone(),
                            version: local_mod.version.clone(),
                            author: local_mod.author.clone(),
                        },
                    );
                    cache_modified = true;
                    results.push(local_mod);
                }
                Err(err) => {
                    warn!("skipping unreadable mod {}: {err}", path.display());
                }
            }
        }

        if cache_modified {
            self.save_cache();
        }

        // Sort alphabetically by file name for deterministic output
        results.sort_by(|a, b| a.file_name.cmp(&b.file_name));
        info!("scanned {} mods in {}", results.len(), mods_dir.display());
        Ok(results)
    }

    /// Computes the SHA-256 hash of a file and inspects its `modDesc.xml`.
    pub fn process_mod_file(path: &Path) -> Result<(String, ModDesc)> {
        let hash_sha256 = calculate_file_sha256(path)?;
        let mod_desc = extract_mod_desc_from_zip(path)?;
        Ok((hash_sha256, mod_desc))
    }
}

impl Default for ModScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes the SHA-256 digest of a local file using a 64KB buffer for optimal I/O throughput.
pub fn calculate_file_sha256(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Inspects a `.zip` file, finds `modDesc.xml` case-insensitively, and parses its contents.
pub fn extract_mod_desc_from_zip(path: &Path) -> Result<ModDesc> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Locate modDesc.xml index
    let mut target_index = None;
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_lowercase();
            // Check if top-level or inside root dir
            if name == "moddesc.xml" || name.ends_with("/moddesc.xml") {
                target_index = Some(i);
                break;
            }
        }
    }

    let index = target_index.ok_or_else(|| {
        ClientError::ModDescNotFound(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        )
    })?;

    let mut entry = archive.by_index(index)?;
    let mut xml_bytes = Vec::new();
    entry
        .by_ref()
        .take(MAX_XML_READ_BYTES)
        .read_to_end(&mut xml_bytes)?;

    let xml_str = String::from_utf8_lossy(&xml_bytes);
    parse_mod_desc(&xml_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::{SimpleFileOptions, ZipWriter};

    fn create_dummy_mod_zip(dir: &Path, file_name: &str, title: &str, version: &str) -> PathBuf {
        let zip_path = dir.join(file_name);
        let file = File::create(&zip_path).expect("create zip file");
        let mut zip = ZipWriter::new(file);

        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        zip.start_file("modDesc.xml", options).expect("start file");
        let xml = format!(
            r#"<modDesc descVersion="90">
                <author>Tester</author>
                <version>{version}</version>
                <title><en>{title}</en></title>
            </modDesc>"#
        );
        zip.write_all(xml.as_bytes()).expect("write xml");
        zip.finish().expect("finish zip");

        zip_path
    }

    #[test]
    fn calculates_sha256_correctly() {
        let temp = tempdir().expect("tempdir");
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, b"hello farmlink").expect("write test file");

        let hash = calculate_file_sha256(&file_path).expect("hash");
        // sha256("hello farmlink") = c7ee0c6a319aa6cd9368cae666500426dcd44bbb28f8e669aaa63de0325db4d8
        assert_eq!(
            hash,
            "c7ee0c6a319aa6cd9368cae666500426dcd44bbb28f8e669aaa63de0325db4d8"
        );
    }

    #[test]
    fn scans_directory_and_extracts_mods() {
        let temp = tempdir().expect("tempdir");
        create_dummy_mod_zip(temp.path(), "FS25_Tractor.zip", "Super Tractor", "1.0.0.0");
        create_dummy_mod_zip(
            temp.path(),
            "FS25_Harvester.zip",
            "Giant Combine",
            "2.1.0.0",
        );

        // Create a non-zip file to verify it's ignored
        fs::write(temp.path().join("readme.txt"), b"ignore me").expect("write readme");

        let mut scanner = ModScanner::new();
        let mods = scanner.scan_directory(temp.path()).expect("scan");

        assert_eq!(mods.len(), 2);
        assert_eq!(mods[0].file_name, "FS25_Harvester.zip");
        assert_eq!(mods[0].title, "Giant Combine");
        assert_eq!(mods[0].version, "2.1.0.0");

        assert_eq!(mods[1].file_name, "FS25_Tractor.zip");
        assert_eq!(mods[1].title, "Super Tractor");
        assert_eq!(mods[1].version, "1.0.0.0");
    }
}
