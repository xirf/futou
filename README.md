# futou (埠頭) — Environment Manager for Windows

Install, activate, and manage runtimes (PHP, MariaDB, PostgreSQL, etc.) via CLI or GUI.

## Architecture

```
┌──────────┐     ┌──────────────┐     ┌────────────┐
│  futou-cli │────▶│  futou-daemon  │◀────│ futou-gui   │
│  (CLI)    │     │  (long-lived) │     │ (Tauri GUI)│
└──────────┘     └───────┬───────┘     └────────────┘
                         │
              ┌──────────┴──────────┐
              │     futou-core        │  ← domain logic
              │     futou-ipc         │  ← shared types
              └─────────────────────┘
```

Communication between processes uses **Windows Named Pipes** (`\\.\pipe\futou`) with JSON-RPC 2.0.

## Prerequisites

| Tool   | Version  | Notes                                            |
|--------|----------|--------------------------------------------------|
| Rust   | 1.85+    | `rustup install stable`                          |
| Bun    | 1.3+     | `powershell -c "irm bun.sh/install.ps1 | iex"`   |
| aria2c | 1.37+    | `winget install aria2` or [aria2.github.io](https://aria2.github.io) |

## Quick Start

```bash
# 1. Build all Rust crates
cargo build

# 2. Start the daemon (background process)
cargo run --bin futou-daemon

# 3. In another terminal, try the CLI
cargo run --bin futou-cli -- catalogue
cargo run --bin futou-cli -- list

# 4. Install a runtime (example: PHP 8.4)
cargo run --bin futou-cli -- install php 8.4.3

# 5. Activate it (adds to PATH)
cargo run --bin futou-cli -- use php 8.4.3

# 6. Or launch the GUI
cd futou-gui
bun run tauri dev
```

## Frontend (Tauri GUI)

```bash
cd futou-gui

# Development mode (hot reload)
bun run tauri dev

# Production build
bun run tauri build
```

The GUI connects to the running daemon via named pipe and provides:

- **Dashboard** — view installed runtimes, start/stop, switch versions
- **Add Service** — browse catalogue, search, install with progress
- **Log panel** — view service logs

## CLI Reference

```bash
# Catalogue
futou catalogue          # List available runtimes from catalogue

# Runtimes
futou list               # List installed runtimes
futou install <name> <version>   # Install a runtime
futou uninstall <name> <version> # Remove a runtime

# Activation
futou use <name> <version>       # Activate (add to PATH)
futou deactivate <name>          # Deactivate (remove from PATH)

# Status
futou status             # Daemon status

# Refresh
futou refresh            # Refresh catalogue cache
```

## Project Structure

```
futou/
├── Cargo.toml              # Workspace root
├── futou-ipc/               # Shared IPC types (JSON-RPC, catalogue)
├── futou-core/              # Domain logic (services, ports)
├── futou-daemon/            # Long-lived daemon process
│   └── resources/          # Fallback catalogue bundle
├── futou-cli/               # CLI frontend (clap)
└── futou-gui/               # Tauri GUI frontend
    ├── src/                # React app (Dashboard, Catalogue, Settings)
    └── src-tauri/          # Tauri Rust commands (named pipe client)
```

## Configuration

Default data directory: `%USERPROFILE%\.futou\`

```
.futou/
├── state.json              # Installed runtimes & state
├── runtimes/               # Downloaded runtime binaries
│   ├── php/8.4.3/
│   ├── mariadb/11.4.5/
│   └── postgresql/17.2/
├── catalogue/              # Catalogue cache
├── shims/                  # .bat shims and symlinks
└── aria2/                  # aria2 downloads
```

Custom data dir: set `futou_DATA_DIR` environment variable or edit `config.toml`.

## How It Works

1. **Daemon** starts on demand (or manually), spawns aria2c in RPC mode
2. **CLI/GUI** sends JSON-RPC requests to the daemon over `\\.\pipe\futou`
3. **Install flow**: catalogue lookup → aria2 download → extract → register in state
4. **Activation**: creates shim (symlink or .bat) → updates PATH via registry
5. **State** is persisted to `state.json` on every mutation

## Development

```bash
# Check all crates
cargo check

# Check Tauri Rust
cd futou-gui/src-tauri && cargo check -p app

# Frontend type-check and build
cd futou-gui && bun run build

# Full clean build
cargo clean && cargo build
```
