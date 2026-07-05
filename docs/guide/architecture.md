# Architecture

## Overview

```
┌──────────┐     ┌──────────────┐     ┌────────────┐
│ futo-cli  │────→│  futo-daemon  │←────│  futo-gui   │
│ (clap)    │     │  (long-lived)  │     │ (Tauri v2)  │
└──────────┘     └───────┬───────┘     └────────────┘
                         │  \\.\pipe\futou (JSON-RPC 2.0)
              ┌──────────┴──────────┐
              │     futo-core        │  domain + services + port traits
              │     futo-ipc         │  shared message types + catalogue
              └─────────────────────┘
```

## Crate Map

### futo-ipc — Shared Types
JSON-RPC 2.0 message types, catalogue data model, error codes. Zero logic. Consumed by every other crate.

### futo-core — Domain Logic
Hexagonal ports-and-adapters design. Domain types (`RuntimeName`, `Version`, `DaemonState`), 7 port traits (`Downloader`, `Extractor`, `ProcessManager`, etc.), and 4 service orchestrators (`InstallService`, `ActivationService`, `CatalogueService`, `EnvService`). No I/O — pure logic.

### futo-daemon — Background Process
Long-lived Windows service. Manages downloads via aria2c, extracts archives, persists state to `state.json`, exposes 14 JSON-RPC methods over a named pipe pool (4 concurrent instances). System tray icon for quick shutdown.

### futo-cli — Terminal Frontend
clap-powered CLI with 8 subcommands. Connects to daemon via Windows named pipe, sends JSON-RPC, prints results. 30-second timeout on all pipe reads.

### futo-gui — Desktop Frontend
React 19 + Tailwind v4 + Tauri v2. Thin Rust proxy layer that forwards Tauri IPC commands to the daemon's named pipe. Features frameless window, EN/ID localization, settings panel, document root prompt for web servers.

## Data Flow

### Install
```
CLI/GUI → runtime.install RPC → handler → InstallService
  → CatalogueSource.fetch_version_urls()
  → Downloader.download() (aria2c)
  → verify_checksum() (SHA256)
  → Extractor.extract()
  → RuntimeRepository.save() (state.json)
```

### Activate
```
CLI/GUI → runtime.activate RPC → handler → ActivationService
  → ShimManager.create_shims() (.bat files)
  → PathManager.add_to_path() (Windows registry)
  → RuntimeRepository.save()
```

### Start Server
```
CLI/GUI → runtime.start RPC → handler → ActivationService
  → ProcessManager.init_data_dir()
  → ProcessManager.start_server()
  → RuntimeRepository.save() (persist PID)
```

## State

All state lives at `%APPDATA%\.futou\`:

| File | Purpose |
|------|---------|
| `state.json` | Installed runtimes, active versions, running PIDs |
| `settings.json` | GUI settings (language, install location) |
| `runtimes/` | Extracted runtime binaries |
| `catalogue/` | Cached catalogue from remote URL |
| `shims/` | `.bat` shim files for activated runtimes |
| `aria2/` | aria2c downloads and PID file |
