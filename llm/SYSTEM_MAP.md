# SYSTEM\_MAP — futou

Windows runtime environment manager. CLI + GUI + daemon over named-pipe JSON-RPC.

```
┌──────────┐     ┌──────────────┐     ┌────────────┐
│ futo-cli  │────→│  futo-daemon  │←────│  futo-gui   │
│ (clap)    │     │  (long-lived)  │     │ (Tauri v2)  │
└──────────┘     └───────┬───────┘     └────────────┘
                         │  \\.\pipe\futou (JSON-RPC 2.0)
              ┌──────────┴──────────┐
              │     futo-core        │  domain + services + port traits
              │     futo-ipc         │  shared message types + catalogue schema
              └─────────────────────┘
```

## Crate Map

### futo-ipc (`futo-ipc/`) — Shared Types
- **Purpose:** JSON-RPC 2.0 message types + catalogue data model. Zero logic, zero deps beyond serde.
- **Key files:**
  - `messages.rs` — `RpcRequest`, `RpcResponse`, `RpcNotification`, plus domain param/result structs for every RPC method
  - `catalogue.rs` — `CatalogueManifest`, `VersionEntry` (what the daemon fetches), `BundledCatalogue` (legacy, unused), `Config`
  - `error_codes.rs` — JSON-RPC standard codes + custom app codes (-32000..-32009)
- **Consumed by:** `futo-core`, `futo-daemon`, `futo-cli`, `futo-gui/src-tauri`

### futo-core (`futo-core/`) — Domain Logic
- **Purpose:** Hexagonal ports-and-adapters. Domain types, port traits, service orchestration. No I/O.
- **Key files:**
  - `domain/runtime.rs` — `RuntimeName`, `Version`, `Installation`, `DaemonState` (all `serde`)
  - `ports/` — 7 traits: `Downloader`, `Extractor`, `PathManager`, `ProcessManager`, `ShimManager`, `CatalogueSource`, `RuntimeRepository`
  - `service/` — `InstallService`, `ActivationService`, `CatalogueService`, `EnvService`
- **Deps:** `futo-ipc`, `serde`, `thiserror`, `async-trait`, `chrono`, `sha2`, `tracing`
- **Tests:** 28 unit tests covering domain serialization, catalogue sorting, checksum verification

### futo-daemon (`futo-daemon/`) — Background Process
- **Purpose:** Long-lived Windows daemon. Downloads, installs, activates runtimes. Exposes JSON-RPC over named pipe.
- **Key files:**
  - `main.rs` — entry: `dirs::data_dir()\.futou`, composition root, pipe server + tray spawn, shutdown signal
  - `composition_root.rs` — `AppContext` assembly: aria2c resolution, adapter wiring, orphan PID cleanup
  - `handler.rs` — 14 RPC method handlers dispatching to core services
  - `pipe_server.rs` — 4-instance named pipe pool, accept → read JSON-RPC → dispatch → respond
  - `tray_manager.rs` — system tray icon with "Exit" menu (bare function, no wrapper struct)
  - `operation_log.rs` — in-memory ring buffer (500 entries, drain oldest 100)
  - `adapters/` — platform impls: `Aria2Downloader`, `NullDownloader`, `AsyncExtractor`, `FsRepository`, `RemoteCatalogueSource`, `WindowsPathManager`, `WindowsProcessManager`, `WindowsShimManager`
  - `resources/bundle.json` — bundled catalogue fallback (included at build via `include_str!`)
- **Deps:** `futo-ipc`, `futo-core`, `tokio` (full), `reqwest`, `tray-icon`, `zip`, `tar`, `flate2`, `winreg`, `which`, etc.

### futo-cli (`futo-cli/`) — Terminal Frontend
- **Purpose:** clap-powered CLI that connects to daemon via named pipe, sends JSON-RPC, prints results.
- **Key files:**
  - `main.rs` — 8 subcommands: list, install, uninstall, use, deactivate, catalogue, status, refresh
  - `pipe_client.rs` — `PipeClient` wrapping `tokio::net::windows::named_pipe`, JSON-RPC send/recv with notification broadcast
- **Deps:** `futo-ipc`, `clap`, `serde`, `serde_json`, `tokio` (full)
- **Note:** `#![cfg(windows)]` — Windows-only. 30s pipe read timeout.

### catalogue-generator (`catalogue-generator/`) — Maintainer Tool
- **Purpose:** Builds schema-v2 catalogue snapshots from official provider metadata, validates pinned artifacts, and creates detached Ed25519 signatures.
- **Runtime discovery:** PHP, Node.js, MariaDB, and Deno use structured upstream APIs; PostgreSQL and Nginx are retained as reviewed pins until stable discovery adapters exist.
- **Trust boundary:** The signing secret lives only in `CATALOGUE_SIGNING_KEY`; the daemon embeds the corresponding public key.
- **CI:** Pull requests validate committed snapshots, while the scheduled/manual workflow proposes signed catalogue updates for maintainer review.

### futo-gui (`futo-gui/`) — Desktop Frontend (Tauri v2)
- **Purpose:** React 19 + Tailwind v4 + Tauri v2. Connects to daemon via named pipe (thin Rust proxy layer).
- **Key files:**
  - `src/App.tsx` — single-page app root: header, service list, log panel, add-service modal, settings toggle
  - `src/components/` — `Header`, `ServiceRow`, `ServiceList`, `LogPanel`, `AddServiceModal`, `CatalogueItem`, `SettingsPanel`, `WindowControls`
  - `src/hooks/` — `useRuntimes`, `useCatalogue`, `useInstall`
  - `src/i18n/` — `en.ts`, `id.ts`, context provider + `useTranslation()` hook
  - `src-tauri/src/lib.rs` — 20 Tauri commands → thin proxy to named pipe RPC
- **Deps:** React 19, Tailwind v4, lucide-react, `@tauri-apps/api`
- **Features:** frameless window, custom title bar, EN/ID localization, install location + autostart settings

## Data Flow

### Install flow
```
CLI/GUI → runtime.install RPC → handler → InstallService::install()
  → CatalogueSource::fetch_version_urls()  (look up URL)
  → Downloader::download()                 (aria2c HTTP or NullDownloader)
  → verify_checksum()                      (SHA256)
  → Extractor::extract()                   (zip/tar.gz via spawn_blocking)
  → RuntimeRepository::add_installation()  (state.json)
```

### Start flow (DB)
```
CLI/GUI → runtime.start RPC → handler → ActivationService::start_process()
  → ProcessManager::init_data_dir()        (mariadb-install-db / initdb)
  → ProcessManager::start_server()         (mariadbd / pg_ctl → PID)
  → RuntimeRepository::save()              (persist PID)
```

### Start flow (Web server)
```
CLI/GUI → runtime.start RPC (with document_root) → handler → ActivationService
  → ProcessManager::start_server(doc_root)
    → generate_apache_config() / generate_nginx_config()
    → httpd.exe -f conf/httpd.conf / nginx.exe -c conf/nginx.conf
```

### Catalogue fetch
```
Daemon startup → RemoteCatalogueSource::load_manifest()
  → verify cache.json + cache.json.sig
  → fetch and verify remote catalogue.json + detached signature
  → fallback to signed bundle.json
```

Catalogue schema v2 requires an HTTPS artifact URL, lowercase SHA-256, provider
provenance, and an explicit official/third-party trust level. Unsigned, malformed,
or checksum-free snapshots fail closed.

## Key Config Paths

| Path | Purpose |
|------|---------|
| `%APPDATA%\.futou\` | GUI settings and default daemon data dir |
| Configured install location | Daemon data root passed as `--data-dir`; contains runtimes, state, cache, and shims |
| `config.toml` | Daemon config (created on first run, not read back yet) |
| `state.json` | Persisted installations, active versions, PIDs |
| `settings.json` | GUI settings (install_dir, written by SettingsPanel) |
| `catalogue\bundle.json` | Bundled fallback catalogue |
| `catalogue\cache.json` | Remote fetch cache |
| `shims\` | `.bat` shim files for active runtimes |
| `runtimes\<name>\<version>\` | Extracted runtime binaries |
| `aria2\` | aria2c download dir + PID file |

GUI startup launches the daemon with the configured data root, waits until the
named pipe answers `daemon.status`, then loads runtimes and catalogue.

## Web Server Runtime Handling

Runtimes tagged as web servers (`apache`, `nginx`) receive special treatment:
- `StartServerParams.document_root` is required (validated at start)
- `ProcessManager::start_server` generates config files from templates:
  - Apache: `httpd.conf` with ServerRoot, Listen 80, DocumentRoot, Directory block
  - Nginx: `nginx.conf` with worker_processes, events, http → server block
- Config is regenerated on every start (stateless, picks up changes)
- Document root path is stored in `localStorage` per runtime for convenience
- `bin_dir` points to the executable directory (Apache: `Apache24/bin`, Nginx: `nginx-1.27.4`)
- Graceful shutdown not yet implemented — uses `taskkill /PID` for all runtimes

## Port Allocation

| Runtime | Default Port |
|---------|-------------|
| MariaDB/MySQL | 3306 |
| PostgreSQL | 5432 |
| Apache | 80 |
| Nginx | 80 |

## RPC Methods (14)

| Method | Params | Result |
|--------|--------|--------|
| `catalogue.list` | — | `{ runtimes: [{ name, display_name, versions }] }` |
| `catalogue.refresh` | — | `{ updated: bool }` |
| `runtime.install` | `{ runtime, version }` | `{ task_id }` (async) |
| `runtime.install.status` | `{ task_id }` | `InstallProgress` |
| `runtime.uninstall` | `{ runtime, version }` | `{ runtime, version, status }` |
| `runtime.list` | — | `{ installed: [InstalledRuntime] }` |
| `runtime.activate` | `{ runtime, version }` | `{ runtime, version, status }` |
| `runtime.deactivate` | `{ runtime }` | `{ runtime, status }` |
| `runtime.start` | `{ runtime, version, document_root? }` | `{ runtime, version, pid }` |
| `runtime.stop` | `{ runtime }` | `{ runtime, status }` |
| `runtime.logs` | `{ runtime }` | `{ entries: [LogEntry] }` |
| `runtime.active` | — | `{ active: { runtime: version } }` |
| `daemon.status` | — | `{ version, uptime_secs, aria2_running, active_tasks }` |
| `daemon.shutdown` | — | `{ status }` |

## RC Patch Status

### ✅ Fixed
- data_initialized operator precedence (parens)
- bundle.json cache overwrite (exists check)
- pipe read timeout (30s in CLI + GUI)
- handler .unwrap() panics (json_success helper)
- progress map stale entries (prune on daemon.status)
- empty version_dir guard (fallback to installation.path)
- uninstall confirmation (CLI stdin + GUI confirm)

### ❌ Remaining
- activation_service race condition (double lock on load→save)
- Child process health check after spawn
- Shim cleanup tracking per runtime+version
- CLI async install feedback (returns before install completes)
- Aria2 NullDownloader user warning
- Document root path validation
- Graceful shutdown for Apache/Nginx (taskkill → httpd -k stop / nginx -s quit)
