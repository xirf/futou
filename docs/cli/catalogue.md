# Catalogue & Status

## Catalogue

The catalogue is a JSON manifest of available runtimes and versions. It's fetched from the remote URL (`raw.githubusercontent.com/xirf/futou/master/catalogue.json`), cached locally, and falls back to a bundled copy when offline.

### Listing

```bash
$ futou catalogue
Available runtimes:
  Apache (apache): 2.4.66
  Deno (deno): 2.3.8, 2.2.11
  MariaDB (mariadb): 11.4.5, 10.11.11
  ...
```

### Refreshing

```bash
futou refresh
```

Forces a remote re-fetch of the catalogue (8-second timeout). Updates the local cache on success. Falls back through cache → bundled catalogue on failure.

## Status

```bash
$ futou status
Daemon status: {
  "version": "0.1.0",
  "uptime_secs": 423,
  "aria2_running": true,
  "active_tasks": 0
}
```

- **version**: Daemon build version
- **uptime_secs**: Seconds since daemon started
- **aria2_running**: Whether the aria2 downloader is available (false = installs won't work)
- **active_tasks**: Number of ongoing install tasks

## Server Management

Start and stop database/web servers:

```bash
futou start mariadb 11.4.5
futou stop mariadb

futou start apache 2.4.66    # Requires document_root
futou stop apache
```

For Apache/Nginx, the `start` command requires a `document_root` parameter when sent via GUI. The CLI will prompt for it.
