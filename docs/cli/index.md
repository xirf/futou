# CLI Reference

The `futou` CLI communicates with the daemon over a Windows named pipe. All commands require the daemon to be running.

## Available Commands

| Command | Description |
|---------|-------------|
| `futou list` | List installed runtimes |
| `futou catalogue` | List available runtimes from the catalogue |
| `futou install <name> <version>` | Install a runtime |
| `futou uninstall <name> <version>` | Remove a runtime (with confirmation) |
| `futou use <name> <version>` | Activate a runtime (add to PATH) |
| `futou deactivate <name>` | Deactivate a runtime (remove from PATH) |
| `futou status` | Show daemon status |
| `futou refresh` | Force-refresh the catalogue from remote |

## Command Details

### `futou list`

Lists all installed runtimes with their version, status, and path.

```
$ futou list
Installed runtimes:
  php 8.4.23 [active]
  mariadb 11.4.5 [installed]
  deno 2.3.8 [active]
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
