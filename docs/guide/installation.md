# Installation

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.85+ | `rustup install stable` |
| Bun | 1.3+ | `powershell -c "irm bun.sh/install.ps1 \| iex"` |
| aria2c | 1.37+ | `winget install aria2` or [aria2.github.io](https://aria2.github.io) |

## Building from Source

```bash
git clone https://github.com/xirf/futou.git
cd futou
cargo build --release
```

This produces three binaries:
- `target/release/futou-daemon.exe` — background service
- `target/release/futou-cli.exe` — command-line interface
- `target/release/futou.exe` (alias for CLI)

## Starting the Daemon

The daemon must be running for CLI or GUI to work:

```bash
cargo run --bin futou-daemon
```

It spawns in the background with a system tray icon. The daemon manages downloads, installations, and server processes.

## Installing the GUI

```bash
cd futou-gui
bun install
bun run tauri dev      # Development
bun run tauri build     # Production installer
```

The GUI connects to the daemon via the same named pipe — no separate configuration needed.
