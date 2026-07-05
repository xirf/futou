# What is Futou?

Futou (埠頭, "pier" in Japanese) is a **Windows environment manager** for developers. Install, activate, switch, and manage multiple versions of runtimes — all from a clean CLI or desktop GUI.

## Why Futou?

Existing tools like XAMPP or Laragon lock you into one version of each runtime. Changing PHP versions means reinstalling the entire stack. Futou gives you:

- **Multiple versions side-by-side**: PHP 8.2, 8.3, and 8.4 installed simultaneously
- **One-command switching**: `futou use php 8.4.23` — your PATH updates instantly
- **Database servers**: MariaDB and PostgreSQL managed as services with proper init and shutdown
- **Web servers**: Apache and Nginx with per-project document roots
- **No bloat**: CLI + optional GUI. No bundled cruft. Fresh downloads from official sources.

## How It Works

```
CLI or GUI  →  Named Pipe (JSON-RPC)  →  Daemon  →  aria2c download  →  Extract  →  Activate
```

1. A long-lived **daemon** runs in the background (system tray)
2. **CLI** or **GUI** sends JSON-RPC commands over `\\.\pipe\futou`
3. Daemon downloads runtimes via **aria2c**, verifies checksums, extracts archives
4. **Activation** creates `.bat` shims and updates `PATH` via Windows registry

## Available Runtimes

| Category | Runtimes |
|----------|----------|
| **Languages** | PHP, Node.js, Python, Deno |
| **Databases** | MariaDB, PostgreSQL |
| **Web Servers** | Apache, Nginx |

## Quick Start

```bash
# Install a runtime
futou install php 8.4.23

# Activate it (adds to PATH)
futou use php 8.4.23

# Start a database
futou start mariadb 11.4.5

# Launch a web server
futou start apache 2.4.66
```

Or use the **GUI** (`cargo run --bin futou-daemon` + Tauri app) for point-and-click management with progress bars and logs.
