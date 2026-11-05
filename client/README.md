# FarmLink Companion (Rust Client)

High-performance native companion daemon and CLI built in Rust for Farming Simulator 25 local mod detection, cryptographic verification, and differential synchronization.

---

## Overview

The FarmLink Companion is a lightweight (4.3 MB standalone binary) native client engineered in Rust (2021 Edition). It bridges the gap between the player's local game installation and the FarmLink web platform.

The application operates in two modes:
1. **Background IPC Daemon**: Runs an Axum HTTP and WebSocket server on `127.0.0.1:45823`, allowing the browser dashboard to read local mod inventory, trigger synchronized downloads, and monitor progress without native browser extensions.
2. **Standalone Command-Line Tool (CLI)**: Allows players and headless setups to scan directories, compute cryptographic hashes, synchronize modpacks from remote manifests, and launch Farming Simulator 25 directly from the terminal.

---

## Key Features

- **Persistent SHA-256 Disk Cache**: Uses a fast disk-backed cache (`~/.config/farmlink/cache.json`) indexed by file size and modification timestamp (`mtime`). Unchanged archives bypass re-hashing, allowing scans of dozens of gigabytes in milliseconds.
- **Atomic Temp-Swap Downloads**: Streams downloads into temporary files (`.farmlink_tmp_<nanos>`), computes SHA-256 on the fly, and validates the hash against the server manifest before atomically renaming to `.zip`. Corrupt or truncated files are automatically discarded.
- **In-Memory Streaming XML Parser**: Inspects archive contents without full disk decompression using `quick-xml` to extract `modDesc.xml` metadata (multilingual titles, version, author, descVersion, and multiplayer compatibility).
- **Concurrency Rate Limiting**: Regulates parallel HTTP downloads via `tokio::sync::Semaphore` to maximize network throughput without saturating disk I/O.
- **Real-Time WebSocket IPC**: Streams state updates (`scanning`, `syncing`, `idle`, `error`) and granular byte transfer progress to connected frontend clients using Tokio broadcast and MPSC channels.
- **Cross-Platform FS25 Detection**: Auto-detects Farming Simulator 25 directories across Windows (`%USERPROFILE%/Documents/My Games/FarmingSimulator2025/mods`), Linux standard XDG directories, and Steam Proton prefixes (Steam AppID `2300320`).
- **Game Launcher & URI Protocol**: Launches the game via Steam or direct executable, with built-in support for `farmlink://join?server=...&addr=...` deep-link URIs.

---

## Architecture & Directory Structure

The codebase adheres strictly to Rust best practices, maintaining a clean library and binary split (`proj-lib-main-split`):

```
client/
├── src/
│   ├── fs25/
│   │   ├── mod.rs               # FS25 module definitions
│   │   ├── mod_desc.rs          # Streaming XML parser for modDesc.xml metadata
│   │   └── scanner.rs           # Directory scanner, SHA-256 hashing, and disk cache
│   ├── config.rs                # Multi-OS path detection and persistent JSON configuration
│   ├── downloader.rs            # Async concurrent download manager with atomic placement
│   ├── error.rs                 # Strongly-typed library errors (thiserror)
│   ├── launcher.rs              # Steam and executable launcher, URI protocol handler
│   ├── lib.rs                   # Library crate entrypoint and public interface
│   ├── main.rs                  # CLI command runner (clap + anyhow)
│   ├── manifest.rs              # Remote manifest client and differential comparison engine
│   ├── protocol.rs              # Typed JSON WebSocket IPC message schemas
│   └── server.rs                # Axum HTTP/WebSocket local IPC daemon
├── tests/
│   └── companion_integration_test.rs  # End-to-end HTTP and WebSocket IPC integration tests
├── Cargo.toml                   # Crate manifest, dependencies, and release LTO profile
└── README.md                    # Module documentation
```

### IPC Endpoints (`127.0.0.1:45823`)

| Protocol | Path | Description |
|---|---|---|
| `WebSocket` | `/ws` | Bi-directional JSON event stream (`scan`, `sync`, `progress`, `status`) |
| `GET` | `/health` | Daemon liveness check |
| `GET` | `/status` | Returns companion state, detected FS25 path, and mod counts |
| `GET` | `/api/mods` | Returns scanned local mods with versions and SHA-256 hashes |

---

## Configuration & Environment Variables

The companion stores user settings and cache data in platform-specific configuration directories:

- **Linux**: `~/.config/farmlink/config.json` & `~/.config/farmlink/cache.json`
- **Windows**: `%APPDATA%\farmlink\config.json` & `%APPDATA%\farmlink\cache.json`

### CLI Parameters & Options

| Command / Flag | Default | Description |
|---|---|---|
| `daemon [--port <PORT>]` | `45823` | Start background IPC server for web dashboard integration |
| `scan [--path <PATH>]` | Auto-detected | Scan and display local FS25 mods, versions, and hashes |
| `sync <SERVER_ID> [--backend-url <URL>]` | `http://localhost:8088` | Synchronize local mods folder with a server manifest |
| `status` | — | Display current daemon status and configured paths |
| `config show` | — | Print active configuration |
| `config set-mods-path <PATH>` | — | Explicitly set and persist custom FS25 mods folder |
| `join <ADDRESS>` | — | Launch Farming Simulator 25 and join dedicated server |

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) 1.80 or higher (Cargo package manager)

### 1. Run the Local Daemon

Start the background daemon for integration with the FarmLink web dashboard:

```bash
cargo run -- daemon
```

The daemon listens on `ws://127.0.0.1:45823/ws` and `http://127.0.0.1:45823/status`.

### 2. Standalone CLI Usage

```bash
# Scan local FS25 mods folder
cargo run -- scan

# Sync with a dedicated server
cargo run -- sync b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22 --backend-url http://localhost:8088

# Check configuration
cargo run -- config show

# Launch game to server
cargo run -- join "zielonka.farmlink-servers.eu:10823"
```

### 3. Build Optimized Release Binary

```bash
cargo build --release
```

Produces an optimized standalone binary in `target/release/farmlink-companion` (4.3 MB with fat LTO and stripped debug symbols).

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
