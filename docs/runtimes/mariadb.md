# MariaDB

MariaDB binaries from [archive.mariadb.org](https://archive.mariadb.org/).

## Versions

| Version | Source |
|---------|--------|
| 11.4.5 | winx64 |
| 10.11.11 | winx64 |

## Starting

```bash
futou start mariadb 11.4.5
```

On first start, the daemon initializes the data directory with `mariadb-install-db`. Subsequent starts skip initialization.

Started on port **3306**.

## Stopping

```bash
futou stop mariadb
```

Uses `taskkill /PID` to terminate the process.

## Configuration

The GUI Config button opens `my.ini` if it exists in the data directory. Common settings:

```ini
[mysqld]
port=3306
datadir=C:/Users/.../data
max_connections=100
```

::: warning Data Directory
The data directory is at `%APPDATA%\.futou\runtimes\mariadb\<version>\data\`. Uninstalling the runtime **permanently deletes** all databases.
:::

## CLI Tools

Activate MariaDB to get CLI tools in PATH:

```bash
futou use mariadb 11.4.5
mysql -u root
```
