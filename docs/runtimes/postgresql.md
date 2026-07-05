# PostgreSQL

PostgreSQL binaries from [EnterpriseDB](https://www.enterprisedb.com/download-postgresql-binaries).

## Versions

| Version | Source |
|---------|--------|
| 17.2 | Windows x64 |
| 16.6 | Windows x64 |

## Starting

```bash
futou start postgresql 17.2
```

On first start, the daemon initializes the data directory with `initdb` (UTF8, no-locale). Uses `pg_ctl start` to launch the server.

Started on port **5432**.

## Stopping

```bash
futou stop postgresql
```

Uses `taskkill /PID` to terminate the process.

## Configuration

The GUI Config button opens `postgresql.conf` if it exists:

```ini
port = 5432
max_connections = 100
shared_buffers = 128MB
```

The config file may be in the `data/` subdirectory.

## CLI Tools

Activate PostgreSQL to get CLI tools in PATH:

```bash
futou use postgresql 17.2
psql -U postgres
```
