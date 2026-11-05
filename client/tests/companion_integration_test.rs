//! Integration tests for FarmLink native companion state, scanning, and launcher.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::tempdir;
use zip::write::{SimpleFileOptions, ZipWriter};

use farmlink_client::commands::AppState;
use farmlink_client::config::Config;
use farmlink_client::fs25::ModScanner;
use farmlink_client::launcher::{launch_fs25, parse_join_url};

fn create_sample_mod(dir: &Path, file_name: &str, title: &str, version: &str) -> PathBuf {
    let zip_path = dir.join(file_name);
    let file = File::create(&zip_path).expect("create zip");
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("modDesc.xml", options)
        .expect("start modDesc.xml");
    let xml = format!(
        r#"<modDesc descVersion="90">
            <author>GIANTS Software</author>
            <version>{version}</version>
            <title><en>{title}</en></title>
        </modDesc>"#
    );
    zip.write_all(xml.as_bytes()).expect("write xml");
    zip.finish().expect("finish zip");
    zip_path
}

#[tokio::test]
async fn test_companion_app_state_and_config() {
    let temp = tempdir().expect("tempdir");
    let config = Config {
        mods_path: temp.path().to_path_buf(),
        ..Config::default()
    };

    let state = AppState::new(config);
    let companion_state = state.current_companion_state().await;

    assert!(companion_state.is_connected);
    assert_eq!(companion_state.mods_path, temp.path().to_string_lossy());
    assert!(!companion_state.sync_in_progress);
    assert_eq!(companion_state.sync_progress, 0);
}

#[tokio::test]
async fn test_mod_scanning_integration() {
    let temp = tempdir().expect("tempdir");
    create_sample_mod(
        temp.path(),
        "FS25_JohnDeere_7R.zip",
        "John Deere 7R Series 2024",
        "1.2.0.0",
    );
    create_sample_mod(
        temp.path(),
        "FS25_Courseplay.zip",
        "Courseplay FS25",
        "7.4.1.0",
    );

    let mut scanner = ModScanner::new();
    let mods = scanner.scan_directory(temp.path()).expect("scan directory");

    assert_eq!(mods.len(), 2);
    assert_eq!(mods[0].file_name, "FS25_Courseplay.zip");
    assert_eq!(mods[0].title, "Courseplay FS25");
    assert_eq!(mods[0].version, "7.4.1.0");

    assert_eq!(mods[1].file_name, "FS25_JohnDeere_7R.zip");
    assert_eq!(mods[1].title, "John Deere 7R Series 2024");
    assert_eq!(mods[1].version, "1.2.0.0");
}

#[test]
fn test_launcher_uri_parsing_and_dispatch() {
    let uri = "farmlink://join?server=srv_zielonka&addr=zielonka.farmlink-servers.eu%3A10823";
    let parsed = parse_join_url(uri).expect("parsed uri");

    assert_eq!(parsed.server_id.as_deref(), Some("srv_zielonka"));
    assert_eq!(
        parsed.address.as_deref(),
        Some("zielonka.farmlink-servers.eu:10823")
    );

    // Verify launcher accepts formatted server address without panic
    let result = launch_fs25(parsed.address.as_deref());
    assert!(result.is_ok());
}
