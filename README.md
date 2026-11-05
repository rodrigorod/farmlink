# FarmLink

<p align="center">
  <img src="frontend/public/tractor-icon.svg" width="72" height="72" alt="FarmLink Logo" />
</p>

<p align="center">
  <strong>Fast, cryptographically verified modpack synchronization for Farming Simulator 25.</strong>
</p>

<p align="center">
  <a href="https://github.com/rodrigorod/farmlink/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/rodrigorod/farmlink/ci.yml?branch=main&label=CI&logo=github" alt="CI Status" /></a>
  <a href="https://github.com/rodrigorod/farmlink/releases"><img src="https://img.shields.io/github/v/release/rodrigorod/farmlink?label=release&logo=rust" alt="Release" /></a>
  <img src="https://img.shields.io/badge/go-1.24-00ADD8?logo=go&logoColor=white" alt="Go Version" />
  <img src="https://img.shields.io/badge/rust-2021-DEA584?logo=rust&logoColor=white" alt="Rust Version" />
  <img src="https://img.shields.io/badge/vue-3.5-4FC08D?logo=vuedotjs&logoColor=white" alt="Vue Version" />
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License" />
</p>

---

## Overview

FarmLink synchronizes Farming Simulator 25 dedicated server modpacks with player machines. It replaces manual `.zip` sharing with automated differential syncing, cryptographic SHA-256 verification, and 1-click server joining.

### Core Architecture

```
┌─────────────────────────────────┐       ┌─────────────────────────────────┐
│     FarmLink Web Dashboard      │       │     FarmLink Go Backend API     │
│    (Vue 3, Tailwind, Pinia)     │◄─────►│    (Go 1.24, Postgres, S3)      │
└────────────────┬────────────────┘       └────────────────┬────────────────┘
                 │                                         │
        WebSocket IPC (45823)                       HTTPS / S3 Direct
                 │                                         │
                 ▼                                         ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                     FarmLink Native Companion (Rust)                      │
│            • SHA-256 disk cache     • Atomic temp-swap downloads          │
│            • modDesc.xml parser     • Steam FS25 game launcher            │
└───────────────────────────────────────────────────────────────────────────┘
```

- **Backend (`/backend`)**: Go HTTP API managing authentication, server manifests, and S3 multipart mod uploads. Single-pass streaming buffers files to disk once and cleans up failed uploads automatically.
- **Frontend (`/frontend`)**: Industrial cockpit-style web dashboard for players and server admins. Manages modpack manifests, versions, and communicates with the local Rust daemon over WebSocket.
- **Desktop Companion (`/client`)**: Self-contained 4.3 MB Rust daemon. Scans local mods folder, computes streaming SHA-256 digests with disk caching (`cache.json`), pulls outdated/missing mods concurrently, and verifies integrity before atomically placing them in the game directory.

---

## Quick Start

### 1. Prerequisites
- Docker & Docker Compose
- Go 1.24+ (for local backend development)
- Node.js 20+ & npm (for frontend)
- Rust 1.80+ (for desktop companion)

### 2. Infrastructure (PostgreSQL & MinIO S3)
```bash
cd backend
docker compose up -d
```
Starts PostgreSQL on port `5432` and MinIO on port `9000` (Console on `9001`).

### 3. Backend API
```bash
cd backend
go run ./cmd/server/main.go
```
Listens on `http://localhost:8088`. Database migrations and S3 bucket initialization run automatically on startup.

### 4. Frontend Dashboard
```bash
cd frontend
npm install
npm run dev
```
Accessible at `http://localhost:5173`.

### 5. Native Rust Companion
```bash
cd client
# Run background IPC daemon for web dashboard
cargo run -- daemon

# Or scan local FS25 mods folder directly from CLI
cargo run -- scan

# Or sync with a server manifest from terminal
cargo run -- sync <server-id>
```
Daemon listens locally on `ws://127.0.0.1:45823/ws` and `http://127.0.0.1:45823/status`.

---

## Mod Verification & Sync Flow

1. **Scan**: The companion client scans local `.zip` archives. Unchanged files (matching size and `mtime`) resolve instantaneously via persistent disk cache.
2. **Diff**: Remote manifest is compared with local files. Mods are classified as `synced`, `outdated`, `missing`, or `extra`.
3. **Atomic Download**: Required mods download concurrently to temporary staging files (`.farmlink_tmp_<nanos>`).
4. **Verification**: SHA-256 is validated against the manifest hash. Upon match, the file is atomically renamed to the target `.zip` in the FS25 mods folder. Corrupted downloads are removed automatically.

---

## Testing & Quality Assurance

Each workspace includes dedicated test suites and strict linting rules:

```bash
# Backend tests
cd backend && go test ./...

# Frontend tests & production build check
cd frontend && npm test && npm run build

# Client tests, clippy (zero warnings), and formatting
cd client && cargo test && cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --check
```

---

## CI / CD Workflows

- **`.github/workflows/ci.yml`**: Automatically validates Go tests, Frontend build and Vitest suites, and Rust tests/Clippy on pull requests and pushes to `main`.
- **`.github/workflows/release.yml`**: Builds release binaries for Linux (`x86_64-unknown-linux-gnu`) and Windows (`x86_64-pc-windows-msvc`), generates `SHA256SUMS.txt`, and creates GitHub Releases on version tags (`v*`).

---

## License

MIT License. See [LICENSE](LICENSE) for details.
