# FarmLink Companion (Rust & Tauri Client)

High-performance native desktop client and IPC backend built with Rust and Tauri v2 for Farming Simulator 25 local mod detection, cryptographic verification, and differential synchronization.

---

## Overview

The FarmLink Companion is a lightweight native desktop client engineered in Rust (2021 Edition) and powered by Tauri v2. It seamlessly bridges the user interface with the local OS and filesystem via native Inter-Process Communication (IPC).

By embedding the companion directly into the native runtime:
- No background HTTP/WebSocket daemon or local listening ports are required.
- The Vue 3 dashboard interacts directly with the Rust engine through secure, typed Tauri IPC invokes and event streams.
- Full cryptographic SHA-256 verification and file streaming happen at native speed with minimal resource footprint.

---

## Key Features

- **Native Tauri v2 IPC**: Zero open network ports on localhost. Frontend communication uses isolated OS-native webview IPC channels.
- **Persistent SHA-256 Disk Cache**: Uses a fast disk-backed cache (`~/.config/farmlink/cache.json`) indexed by file size and modification timestamp (`mtime`). Unchanged archives bypass re-hashing, allowing scans of dozens of gigabytes in milliseconds.
- **Atomic Temp-Swap Downloads**: Streams downloads into temporary files (`.farmlink_tmp_<nanos>`), computes SHA-256 on the fly, and validates the hash against the server manifest before atomically renaming to `.zip`. Corrupt or truncated files are automatically discarded.
- **In-Memory Streaming XML Parser**: Inspects archive contents without full disk decompression using `quick-xml` to extract `modDesc.xml` metadata (multilingual titles, version, author, descVersion, and multiplayer compatibility).
- **Concurrency Rate Limiting**: Regulates parallel HTTP downloads via `tokio::sync::Semaphore` to maximize network throughput without saturating disk I/O.
- **Real-Time Tauri Event Streams**: Emits granular progress events (`sync_progress`, `sync_completed`, `scan_result`) directly to the Pinia state store in the frontend.
- **Cross-Platform FS25 Detection**: Auto-detects Farming Simulator 25 directories across Windows (`%USERPROFILE%/Documents/My Games/FarmingSimulator2025/mods`), Linux standard XDG directories, and Steam Proton prefixes (Steam AppID `2300320`).
- **Game Launcher & URI Protocol**: Launches the game via Steam or direct executable, with built-in support for `farmlink://join?server=...&addr=...` deep-link URIs.

---

## Architecture & Directory Structure

The codebase adheres strictly to Rust best practices, maintaining a clean library and binary split:

```
client/
├── src/
│   ├── fs25/
│   │   ├── mod.rs               # FS25 module definitions
│   │   ├── mod_desc.rs          # Streaming XML parser for modDesc.xml metadata
│   │   └── scanner.rs           # Directory scanner, SHA-256 hashing, and disk cache
│   ├── commands.rs              # Tauri IPC commands and event emitters
│   ├── config.rs                # Multi-OS path detection and persistent JSON configuration
│   ├── downloader.rs            # Async concurrent download manager with atomic placement
│   ├── error.rs                 # Strongly-typed library errors (thiserror)
│   ├── launcher.rs              # Steam and executable launcher, URI protocol handler
│   ├── lib.rs                   # Library crate entrypoint and public interface
│   ├── main.rs                  # Tauri v2 application builder and entrypoint
│   ├── manifest.rs              # Remote manifest client and differential comparison engine
│   └── protocol.rs              # Typed JSON IPC payloads and state structures
├── capabilities/
│   └── default.json             # Tauri v2 security ACL and permission capabilities
├── icons/                       # Multi-resolution application icons
├── tests/
│   └── companion_integration_test.rs  # End-to-end IPC state and mod scanner tests
├── tauri.conf.json              # Tauri v2 configuration manifest
├── Cargo.toml                   # Crate manifest, dependencies, and release LTO profile
└── README.md                    # Module documentation
```

### Tauri IPC Commands

| Command | Arguments | Return Type | Description |
|---|---|---|---|
| `scan_mods` | `mods_path: Option<String>` | `Result<Vec<LocalModSummary>, String>` | Scans directory, extracts versions and SHA-256 hashes |
| `sync_server` | `server_id: String, backend_url?: String, mod_names?: Vec<String>` | `Result<(), String>` | Performs differential download and sync with live event updates |
| `launch_game` | `target: Option<String>` | `Result<GameLaunchResult, String>` | Dispatches game launch via Steam or direct path |
| `set_mods_path` | `path: String` | `Result<String, String>` | Updates and persists custom FS25 mods folder |
| `get_companion_state` | — | `Result<CompanionStatus, String>` | Fetches real-time companion state and OS info |

### Tauri IPC Events Emitted

| Event Name | Payload Schema | Description |
|---|---|---|
| `sync_progress` | `{ file_name, bytes_downloaded, total_bytes, overall_progress }` | Real-time byte progress per mod archive |
| `sync_completed` | `{ synced_count, failed_count, errors, last_sync_at }` | Notification on batch synchronization completion |
| `scan_result` | `{ total_mods, total_size_bytes }` | Summary of scanned local mod library |
| `error` | `{ message }` | Operation error notifications |

---

## Configuration & Cache Paths

The companion stores user settings and cache data in platform-specific configuration directories:

- **Linux**: `~/.config/farmlink/config.json` & `~/.config/farmlink/cache.json`
- **Windows**: `%APPDATA%\farmlink\config.json` & `%APPDATA%\farmlink\cache.json`

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) 1.80 or higher (Cargo package manager)
- [Node.js](https://nodejs.org/) 20+ & npm (for frontend bundling)
- Linux: `libwebkit2gtk-4.1-dev` and `libsoup-3.0-dev`

### Development

Run the frontend development server and Tauri desktop app concurrently:

```bash
# In frontend directory
npm run dev

# In client directory
cargo run
```

### Build Production Binary

```bash
# 1. Build frontend distribution
cd ../frontend && npm run build && cd ../client

# 2. Build optimized release binary
cargo build --release
```

Produces an optimized standalone desktop executable in `target/release/farmlink-companion`.

---

## Testing & Quality Assurance

```bash
# Run unit and integration tests
cargo test

# Run strict Clippy linter checks
cargo clippy --all-targets --all-features -- -D warnings

# Verify source formatting
cargo fmt --check
```
