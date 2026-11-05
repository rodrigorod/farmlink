# FarmLink Dashboard (Frontend)

Cockpit-style web dashboard built with Vue 3, TypeScript, and Tailwind CSS for managing Farming Simulator 25 servers and synchronizing modpacks via the local companion.

---

## Overview

The FarmLink Dashboard provides an industrial, tactile web interface for players and server administrators. It connects to the central Go backend for server listings and modpack manifests, while simultaneously communicating with the local Rust companion daemon over WebSocket (`ws://127.0.0.1:45823/ws`).

Through this hybrid web-and-local IPC architecture, players can inspect server modpacks, review differential sync statuses (`synced`, `outdated`, `missing`, `extra`), trigger cryptographic verification, track live download progress, and launch Farming Simulator 25 directly from their browser.

---

## Key Features

- **Local Companion WebSocket Bridge**: Connects directly to `ws://127.0.0.1:45823/ws` to query local game path status, initiate mod folder scans, and receive real-time byte-level download updates.
- **Differential Modpack View**: Displays detailed comparisons between local `.zip` files and remote server manifests with status badges (`synced`, `outdated`, `missing`, `extra`).
- **Admin Modpack Manager**: Provides drag-and-drop `.zip` upload functionality, active/inactive mod toggling, and mod removal from dedicated server modpacks.
- **Cockpit-Grade Dark Interface**: High-contrast, accessibility-focused UI designed for quick scanning, built using Tailwind CSS and Lucide icons.
- **Client-Side State Management**: Centralized reactive state powered by Pinia with dedicated stores for authentication, server manifests, local companion IPC, and toasts.
- **Type-Safe Routing & Guards**: Vue Router configuration with authentication navigation guards and clean URL state synchronization.

---

## Architecture & Directory Structure

```
frontend/
├── public/
│   └── tractor-icon.svg         # FarmLink tractor icon
├── src/
│   ├── assets/
│   │   └── main.css             # Tailwind CSS base and utility styling
│   ├── components/
│   │   ├── layout/              # Navigation bar, header, and container layouts
│   │   └── ui/                  # Reusable UI primitives (buttons, badges, modals, cards)
│   ├── lib/
│   │   ├── api.ts               # HTTP client with JWT interceptor and error handling
│   │   ├── utils.ts             # Formatting utilities (file sizes, dates, hashes)
│   │   └── utils.test.ts        # Formatting and utility unit tests
│   ├── router/
│   │   └── index.ts             # Route definitions and authentication navigation guards
│   ├── stores/
│   │   ├── auth.ts              # Authentication state, login/registration, and persistent JWT
│   │   ├── companion.ts         # WebSocket IPC client for local Rust companion daemon
│   │   ├── servers.ts           # Server catalog and manifest state management
│   │   ├── stores.test.ts       # Pinia store integration tests
│   │   └── toast.ts             # Notification toast queue
│   ├── types/
│   │   └── index.ts             # Shared TypeScript interfaces (Server, Mod, Manifest, IPC)
│   ├── views/
│   │   ├── AuthView.vue         # Player login and account registration form
│   │   ├── DashboardView.vue    # Server list cockpit with companion connection badge
│   │   ├── ServerAdminView.vue  # Server management and drag-and-drop modpack upload
│   │   ├── ServerDetailView.vue # Detailed server view, differential mod list, and sync CTA
│   │   └── SettingsView.vue     # Local companion configuration and game path editor
│   ├── App.vue                  # Root application component with toast container
│   └── main.ts                  # Application bootstrap and Pinia/Router plugin registration
├── index.html                   # HTML entrypoint
├── package.json                 # Node.js project manifest and scripts
├── tailwind.config.js           # Tailwind theme configuration
├── tsconfig.json                # TypeScript compiler settings
└── vite.config.ts               # Vite bundler configuration
```

### Application Views

| Path | View | Access | Description |
|---|---|---|---|
| `/` | `DashboardView` | Public | Server browser with connection status and companion badge |
| `/servers/:id` | `ServerDetailView` | Public | Manifest differential comparison and 1-click sync/launch |
| `/servers/:id/admin` | `ServerAdminView` | Authenticated | Drag-and-drop mod upload, toggle active, and mod deletion |
| `/settings` | `SettingsView` | Public | Companion settings, detected FS25 path, and IPC diagnostic |
| `/auth` | `AuthView` | Public | Login and registration tabs |

---

## Configuration & Environment Variables

Environment variables are defined in `.env` or passed at build/runtime with the `VITE_` prefix:

| Variable | Default | Description |
|---|---|---|
| `VITE_API_BASE_URL` | `http://localhost:8088` | Base URL of the FarmLink Go backend API |
| `VITE_COMPANION_WS_URL` | `ws://127.0.0.1:45823/ws` | WebSocket URL for the local Rust companion daemon |

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 20.x or higher
- [npm](https://www.npmjs.com/) 10.x or higher

### 1. Install Dependencies

```bash
npm install
```

### 2. Run Development Server

```bash
npm run dev
```

The application is available at `http://localhost:5173`. Hot module replacement (HMR) is enabled.

### 3. Build for Production

```bash
npm run build
```

Performs full type-checking via `vue-tsc` and compiles optimized static assets into the `dist/` directory.

### 4. Preview Production Build

```bash
npm run preview
```

---

## Testing & Quality Assurance

```bash
# Run unit and store tests with Vitest
npm test

# Run TypeScript type check
npx vue-tsc --noEmit
```
