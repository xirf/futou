# Runtimes

Futou manages three categories of runtimes:

## Languages

Runtimes that provide a CLI binary and are activated via PATH shims.

| Runtime | Versions | Source | Activation |
|---------|----------|--------|------------|
| PHP | 8.2, 8.3, 8.4 | windows.php.net | `futou use php 8.4.23` |
| Node.js | 20, 22 | nodejs.org | `futou use nodejs 22.23.1` |
| Python | 3.12, 3.13 | python.org (embeddable) | `futou use python 3.13.3` |
| Deno | 2.2, 2.3 | deno.land | `futou use deno 2.3.8` |

## Databases

Runtimes that run as server processes with data directory initialization.

| Runtime | Versions | Default Port | Init Command |
|---------|----------|-------------|--------------|
| MariaDB | 10.11, 11.4 | 3306 | `mariadb-install-db` |
| PostgreSQL | 16.6, 17.2 | 5432 | `initdb` |

## Web Servers

Runtimes that serve HTTP with generated configuration files.

| Runtime | Versions | Default Port | Config |
|---------|----------|-------------|--------|
| Apache | 2.4.66 | 80 | `httpd.conf` (auto-generated) |
| Nginx | 1.27.4 | 80 | `nginx.conf` (auto-generated) |

## Installation Flow

All runtimes follow the same install pipeline:

1. **Catalogue lookup** — find download URL and checksum
2. **Download** — aria2c with progress tracking
3. **Verify** — SHA256 checksum
4. **Extract** — zip or tar.gz to `%APPDATA%\.futou\runtimes\<name>\<version>\`
5. **Register** — update `state.json`

## Activation vs Starting

- **Activation** (`use`/`activate`): Makes runtime binaries available in PATH. Applies to languages and CLI tools.
- **Starting** (`start`): Launches a server process in the background. Applies to databases and web servers.

You can activate a database (to get its CLI tools) and start it (to run the server) independently.
