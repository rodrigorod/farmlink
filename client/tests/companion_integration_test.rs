//! End-to-end integration tests for the FarmLink companion daemon and IPC services.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tempfile::tempdir;
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use zip::write::{SimpleFileOptions, ZipWriter};

use farmlink_client::config::Config;
use farmlink_client::protocol::{ClientMessage, ServerMessage};
use farmlink_client::server::{create_router, AppState};

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
async fn test_http_health_and_status_endpoints() {
    let temp = tempdir().expect("tempdir");
    let config = Config {
        mods_path: temp.path().to_path_buf(),
        ..Config::default()
    };

    let state = Arc::new(AppState::new(config));
    let app = create_router(state);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve app");
    });

    let client = reqwest::Client::new();

    // 1. Test /health
    let health_resp = client
        .get(format!("http://{addr}/health"))
        .send()
        .await
        .expect("get /health");

    assert_eq!(health_resp.status(), 200);
    let health_json: serde_json::Value = health_resp.json().await.expect("parse json");
    assert_eq!(health_json["status"], "ok");
    assert_eq!(health_json["service"], "farmlink-companion");

    // 2. Test /status
    let status_resp = client
        .get(format!("http://{addr}/status"))
        .send()
        .await
        .expect("get /status");

    assert_eq!(status_resp.status(), 200);
    let status_json: serde_json::Value = status_resp.json().await.expect("parse json");
    assert_eq!(status_json["is_connected"], true);
    assert_eq!(
        status_json["mods_path"].as_str().unwrap(),
        temp.path().to_str().unwrap()
    );
}

#[tokio::test]
async fn test_websocket_lifecycle_and_scan_command() {
    let temp = tempdir().expect("tempdir");
    create_sample_mod(
        temp.path(),
        "FS25_JohnDeere_7R.zip",
        "John Deere 7R",
        "1.2.0.0",
    );
    create_sample_mod(temp.path(), "FS25_Courseplay.zip", "Courseplay", "7.4.1.0");

    let config = Config {
        mods_path: temp.path().to_path_buf(),
        ..Config::default()
    };

    let state = Arc::new(AppState::new(config));
    let app = create_router(state);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve app");
    });

    let ws_url = format!("ws://{addr}/ws");
    let (ws_stream, _) = connect_async(&ws_url).await.expect("connect websocket");
    let (mut write, mut read) = ws_stream.split();

    // 1. Expect initial State message pushed on connect
    let first_msg = read.next().await.expect("initial msg").expect("ws read");
    let initial_text = first_msg.to_text().expect("to text");
    let server_msg: ServerMessage = serde_json::from_str(initial_text).expect("parse state");

    match server_msg {
        ServerMessage::State(s) => {
            assert!(s.is_connected);
            assert_eq!(s.mods_path, temp.path().to_string_lossy());
        }
        _ => panic!("expected ServerMessage::State, got {:?}", server_msg),
    }

    // 2. Send Ping and expect Pong
    let ping_cmd = serde_json::to_string(&ClientMessage::Ping).expect("serialize ping");
    write
        .send(Message::Text(ping_cmd.into()))
        .await
        .expect("send ping");

    let pong_msg = read.next().await.expect("pong msg").expect("ws read");
    let pong_server_msg: ServerMessage =
        serde_json::from_str(pong_msg.to_text().expect("text")).expect("parse pong");
    assert_eq!(pong_server_msg, ServerMessage::Pong);

    // 3. Send Scan command and expect ScanResult
    let scan_cmd = serde_json::to_string(&ClientMessage::Scan).expect("serialize scan");
    write
        .send(Message::Text(scan_cmd.into()))
        .await
        .expect("send scan");

    let scan_msg = read
        .next()
        .await
        .expect("scan result msg")
        .expect("ws read");
    let scan_server_msg: ServerMessage =
        serde_json::from_str(scan_msg.to_text().expect("text")).expect("parse scan");

    match scan_server_msg {
        ServerMessage::ScanResult {
            mods, total_mods, ..
        } => {
            assert_eq!(total_mods, 2);
            assert_eq!(mods.len(), 2);
            assert_eq!(mods[0].file_name, "FS25_Courseplay.zip");
            assert_eq!(mods[0].version, "7.4.1.0");
            assert_eq!(mods[1].file_name, "FS25_JohnDeere_7R.zip");
            assert_eq!(mods[1].version, "1.2.0.0");
        }
        _ => panic!(
            "expected ServerMessage::ScanResult, got {:?}",
            scan_server_msg
        ),
    }
}
