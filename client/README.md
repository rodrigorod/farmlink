# FarmLink Companion (Client Rust)

Application compagnon native haute performance développée en **Rust (Edition 2021 / Rust 1.99+)** pour **Farming Simulator 25 (FS25)**.

Elle gère la détection locale du dossier des mods, l'inspection cryptographique SHA-256, l'analyse des descripteurs `modDesc.xml`, la synchronisation asynchrone concurrente avec le backend Go / S3, et expose une interface IPC locale en WebSocket (`ws://127.0.0.1:45823/ws`) et HTTP REST pour l'application web frontend Vue.js 3.

---

## 🛠️ Architecture & Bonnes Pratiques Rust

Le projet applique rigoureusement les directives des skills Rust intégrées (`rust-skills`, `rust-patterns`, `rust-testing`, `rust-best-practices`) :

1. **Séparation Bibliothèque / Binaire (`proj-lib-main-split`)** :
   - `src/lib.rs` : Coeur métier, API publique, modules réutilisables.
   - `src/main.rs` : Point d'entrée CLI minimaliste basé sur `clap` et `anyhow`.
2. **Gestion des erreurs typées (`err-thiserror-lib` & `err-anyhow-app`)** :
   - Bibliothèque basée sur `thiserror::Error` (`ClientError`).
   - Binaire CLI utilisant `anyhow::Result` avec enrichissement contextuel (`with_context`).
   - Zéro `unwrap()` dans le code de production (`err-no-unwrap-prod`).
3. **Gestion asynchrone & Concurrence (`async-tokio-fs`, `async-bounded-channel`)** :
   - Moteur basé sur **Tokio** et **Axum 0.8**.
   - Téléchargement concurrent contrôlé par sémaphore (`tokio::sync::Semaphore`).
   - Diffusion d'événements en temps réel via `tokio::sync::broadcast` et `mpsc`.
   - Écriture atomique sécurisée : téléchargement vers un fichier temporaire, validation SHA-256, puis renommage atomique vers `.zip`.
4. **Performance & Optimisation Mémoire (`mem-with-capacity`, `perf-io-buffering`)** :
   - Calcul de hash SHA-256 en flux avec `BufReader` de 64 Ko.
   - Système de cache disque (`~/.config/farmlink/cache.json`) pour éviter de recalculer le hash des archives inchangées (scan de dizaines de gigaoctets en quelques millisecondes).
   - Profil release avec LTO fat, 1 codegen-unit et binaire final compact (4.3 Mo).
5. **Tests & TDD (`rust-testing`)** :
   - Tests unitaires complets dans chaque module (`#[cfg(test)] mod tests`).
   - Tests d'intégration end-to-end dans `tests/companion_integration_test.rs` validant le serveur HTTP, le handshake WebSocket et les commandes.

---

## 📂 Structure des Modules

| Module | Fichier | Description |
|---|---|---|
| **Config** | `src/config.rs` | Détection automatique multi-OS du dossier FS25 (Windows `%USERPROFILE%/Documents/My Games/...`, Linux Proton Steam AppID 2300320, standard XDG), persistance JSON dans `~/.config/farmlink/config.json`. |
| **FS25 ModDesc** | `src/fs25/mod_desc.rs` | Parseur XML streaming (`quick-xml`) pour `modDesc.xml` (titres multilingues EN/FR/DE, version, auteur, descVersion, support multijoueur). |
| **FS25 Scanner** | `src/fs25/scanner.rs` | Scanner d'archives `.zip`, calcul SHA-256 et cache persistant indexé par taille et mtime. |
| **Manifest** | `src/manifest.rs` | Modèles DTO synchronisés avec le backend Go (`/api/v1/servers/:id/manifest`), calcul différentiel (`synced`, `outdated`, `missing`, `extra`). |
| **Downloader** | `src/downloader.rs` | Gestionnaire de téléchargement asynchrone en streaming avec validation cryptographique et rapport de progression. |
| **Protocol** | `src/protocol.rs` | Messages typés JSON échangés via WebSocket (`ClientMessage`, `ServerMessage`, `CompanionState`). |
| **Server** | `src/server.rs` | Démon local Axum avec WebSocket `/ws`, `/health`, `/status`, `/api/mods` et CORS permissif pour le web local. |
| **Launcher** | `src/launcher.rs` | Lancement de Farming Simulator 25 via Steam ou binaire direct, support du protocole URI `farmlink://join?server=...&addr=...`. |

---

## 🚀 Utilisation de la CLI

### 1. Démarrer le démon local (arrière-plan pour le frontend web)
```bash
cargo run -- daemon
# Ou avec un port personnalisé :
cargo run -- daemon --port 45823
```
Le serveur écoute sur :
- **WebSocket** : `ws://127.0.0.1:45823/ws`
- **HTTP Statut** : `http://127.0.0.1:45823/status`
- **HTTP Santé** : `http://127.0.0.1:45823/health`

### 2. Scanner les mods locaux
```bash
cargo run -- scan
# Ou sur un dossier spécifique :
cargo run -- scan --path /chemin/vers/mods
```

Sortie console :
```text
Scanning mods directory: /home/rauxde/.local/share/FarmingSimulator2025/mods

FILE NAME                           TITLE                     VERSION            SIZE SHA-256 HASH
------------------------------------------------------------------------------------------------------------------------------------------------------
FS25_ClaasLexion8900.zip            Claas Lexion 8900 Pack    1.0.2.0        125.9 MB 03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4
FS25_Courseplay.zip                 Courseplay FS25 NextGen   7.4.1.0         14.7 MB c83d6a9956d498b7921a24d550e0468e0d4948f93616238b1d4317135084922f
FS25_JohnDeere_7R.zip               John Deere 7R Series 2024 1.2.0.0         80.6 MB 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08
------------------------------------------------------------------------------------------------------------------------------------------------------
Total: 3 mods verified | Total Size: 221.20 MB (0.22 GB)
```

### 3. Synchroniser directement avec un serveur FarmLink
```bash
cargo run -- sync b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22 --backend-url http://localhost:8088
```

### 4. Consulter l'état et la configuration
```bash
cargo run -- status
cargo run -- config show
cargo run -- config set-mods-path /nouveau/chemin/mods
```

### 5. Rejoindre un serveur (Lancement FS25)
```bash
cargo run -- join "zielonka.farmlink-servers.eu:10823"
# Ou via l'URL personnalisée :
cargo run -- join "farmlink://join?server=srv_01&addr=zielonka.farmlink-servers.eu:10823"
```

---

## 🧪 Tests & Qualité de code

```bash
# Exécution de l'ensemble des tests (unitaires + intégration)
cargo test

# Vérification du respect strict des règles Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Vérification du formatage
cargo fmt --check

# Compilation optimisée en release (binaire autonome de 4.3 Mo)
cargo build --release
```
