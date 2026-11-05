# FarmLink Backend (Go API)

High-performance Go HTTP REST API managing server manifests, modpack catalogues, S3 storage distribution, and player authentication for Farming Simulator 25.

---

## Overview

The FarmLink Backend acts as the central coordinator and storage gateway for Farming Simulator 25 multiplayer communities. It maintains dedicated server registrations, tracks modpack versions, validates file integrity metadata (SHA-256 hashes and file sizes), and distributes mod files through direct S3 object storage.

The service is built on standard library Go (`net/http`) leveraging Go 1.22+ routing enhancements, backed by a PostgreSQL relational store and an S3-compatible storage backend (MinIO or AWS S3).

---

## Key Features

- **Streaming S3 Uploads**: Handles multi-megabyte mod archive uploads via single-pass streaming to a temporary disk buffer, transferring to S3/MinIO with automatic cleanup on failure to conserve memory.
- **Manifest Engine**: Generates canonical modpack manifests containing SHA-256 cryptographic hashes, file sizes, and download URLs for fast client-side differential synchronization.
- **Zero-Dependency Router**: Uses native Go 1.22+ HTTP pattern matching (`GET /api/v1/servers/{id}/manifest`, `POST /api/v1/servers/{id}/mods/upload`) without external routing frameworks.
- **Embedded Schema Migrations**: Automatically applies PostgreSQL database migrations and initial seed records on service startup.
- **Graceful Shutdown**: Intercepts `SIGINT` and `SIGTERM` signals to allow active requests and database connections to drain within a bounded 10-second timeout.
- **JWT Authentication**: Secures administrative endpoints (server creation, mod uploads, mod toggles) using bcrypt password hashing and HMAC-SHA256 JWT tokens.

---

## Architecture & Directory Structure

```
backend/
├── cmd/
│   └── server/
│       └── main.go              # Application entrypoint, wiring, routes, graceful shutdown
├── internal/
│   ├── config/
│   │   └── config.go            # Environment variable configuration and production guards
│   ├── database/
│   │   ├── db.go                # pgxpool connection lifecycle and schema migration runner
│   │   └── schema.sql           # PostgreSQL DDL and initial demo seeds
│   ├── handlers/
│   │   ├── auth.go              # User registration, login, and JWT middleware
│   │   ├── auth_test.go         # Authentication unit and integration tests
│   │   ├── health.go            # Health check endpoint (/api/v1/health)
│   │   ├── mod.go               # Multipart .zip upload handler with S3 streaming
│   │   ├── mod_test.go          # Mod upload and validation tests
│   │   ├── server.go            # Server listing, manifest generation, mod toggling/removal
│   │   └── server_test.go       # Server and manifest handler tests
│   └── storage/
│       └── s3.go                # MinIO / AWS S3 client wrapper and bucket provisioner
├── Dockerfile                   # Multi-stage production container build
├── docker-compose.yml           # PostgreSQL 16 and MinIO development infrastructure
└── go.mod                       # Go module dependencies (Go 1.24)
```

### API Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/api/v1/health` | Public | Service health probe |
| `POST` | `/api/v1/auth/register` | Public | Register a new user account |
| `POST` | `/api/v1/auth/login` | Public | Authenticate user and receive JWT token |
| `GET` | `/api/v1/servers/{id}/manifest` | Public | Retrieve modpack manifest with SHA-256 hashes |
| `GET` | `/api/v1/servers` | Bearer JWT | List all registered servers |
| `POST` | `/api/v1/servers/{id}/mods/upload` | Bearer JWT | Upload a mod `.zip` archive to server |
| `PATCH` | `/api/v1/servers/{id}/mods/{mod_id}` | Bearer JWT | Toggle a mod active/inactive on a server |
| `DELETE` | `/api/v1/servers/{id}/mods/{mod_id}` | Bearer JWT | Remove a mod from a server modpack |

---

## Configuration & Environment Variables

All settings are configured via environment variables with sensible local defaults:

| Variable | Default | Description |
|---|---|---|
| `PORT` | `8088` | HTTP port the server listens on |
| `DATABASE_URL` | `postgres://postgres:postgres@localhost:5432/farmlink?sslmode=disable` | PostgreSQL connection string |
| `S3_ENDPOINT` | `http://localhost:9000` | S3 / MinIO API endpoint |
| `S3_PUBLIC_URL` | Same as `S3_ENDPOINT` | Publicly accessible URL for mod downloads |
| `S3_ACCESS_KEY` | `minioadmin` | S3 access key ID |
| `S3_SECRET_KEY` | `minioadmin` | S3 secret access key |
| `S3_BUCKET` | `farmlink-mods` | S3 bucket name for mod storage |
| `S3_REGION` | `us-east-1` | S3 bucket region |
| `JWT_SECRET` | `farmlink-dev-secret-key-fs25` | Secret key for JWT signing (fails on startup if default in production) |
| `ENV` | `development` | Runtime environment (`development` or `production`) |

---

## Getting Started

### Prerequisites

- [Go](https://go.dev/) 1.24 or higher
- [Docker](https://www.docker.com/) & Docker Compose (for local PostgreSQL & MinIO)

### 1. Start Local Infrastructure

Start the PostgreSQL database and MinIO S3 storage:

```bash
docker compose up -d
```

- **PostgreSQL**: `localhost:5432` (database: `farmlink`, user: `postgres`, password: `postgres`)
- **MinIO S3 API**: `http://localhost:9000`
- **MinIO Web Console**: `http://localhost:9001` (user: `minioadmin`, password: `minioadmin`)

### 2. Run the Backend Server

```bash
go run ./cmd/server/main.go
```

The server runs database migrations, creates the `farmlink-mods` S3 bucket if missing, and starts listening on `http://localhost:8088`.

### 3. Build Production Binary

```bash
go build -ldflags="-s -w" -o bin/server ./cmd/server
```

---

## Testing & Quality Assurance

Run the test suite including database integration tests and HTTP handler tests:

```bash
# Run all tests
go test -v ./...

# Run static analysis
go vet ./...
```
