# CLI Reference

The `futou` CLI communicates with the daemon over a Windows named pipe. All commands require the daemon to be running.

## Available Commands

| Command | Description |
|---------|-------------|
| `futou list` | List installed runtimes |
| `futou install <name> <version>` | Install a runtime |
| `futou uninstall <name> <version>` | Remove a runtime (with confirmation) |
| `futou use <name> <version>` | Activate a runtime (add to PATH) |
| `futou deactivate <name>` | Deactivate a runtime (remove from PATH) |
| `futou start <name>` | Start a server process |
| `futou stop <name>` | Stop a server process |
| `futou logs <name>` | Show operation logs for a runtime |
| `futou active` | Show currently active runtimes |
| `futou catalogue` | List available runtimes from the catalogue |
| `futou status` | Show daemon status |
| `futou refresh` | Force-refresh the catalogue from remote |

## Command Details

### `futou list`

Lists all installed runtimes with their version, status, and running state.

```
$ futou list
Installed runtimes:
  php 8.4.23 [active]
  mariadb 11.4.5 [installed] [running]
  deno 2.3.8 [active]
```

### `futou start`

Starts a database or web server process. Version auto-detected if only one is installed.

```bash
futou start mariadb                    # Auto-detects version
futou start mariadb -v 11.4.5          # Explicit version
futou start apache -d D:\projects\myapp  # Web server with document root
```

For Apache/Nginx without `-d`, the CLI prompts for a document root interactively.

### `futou stop`

Stops a running server process.

```bash
futou stop mariadb
futou stop apache
```

### `futou logs`

Shows recent operation logs for a runtime.

```
$ futou logs mariadb
[09:14:25] INFO  mariadb 11.4.5 server started (pid 9042)
[09:22:10] INFO  mariadb server stopped
```

### `futou active`

Shows which runtimes are currently activated (in PATH).

```
$ futou active
Active runtimes:
  php -> 8.4.23
  deno -> 2.3.8
```

### `futou catalogue`

Lists all available runtimes from the catalogue (remote fetch with local cache).

```
$ futou catalogue
Available runtimes:
  PHP (php): 8.4.23, 8.3.32, 8.2.32
  Node.js (nodejs): 22.23.1, 20.20.2
  Deno (deno): 2.3.8, 2.2.11
  ...
```
