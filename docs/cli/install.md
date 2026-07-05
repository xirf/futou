# Install & Uninstall

## Installing a Runtime

```bash
futou install <name> <version>
```

Downloads the runtime archive via aria2c, verifies the SHA256 checksum, extracts it, and registers it in the daemon state.

```bash
$ futou install php 8.4.23
Looking up version in catalogue 0.0%
Downloading 35.0%
Verifying checksum 70.0%
Extracting 85.0%
Registering installation 95.0%
php 8.4.23 installed successfully
```

The CLI polls the daemon every 600ms for progress updates, showing a progress bar with percentage.

### Supported Archive Types

- `.zip` (most runtimes)
- `.tar.gz` (some Linux-native runtimes)

## Uninstalling a Runtime

```bash
futou uninstall <name> <version>
```

::: danger Data Loss
Uninstalling deletes the runtime directory and all associated data. For databases (MariaDB, PostgreSQL), this **permanently deletes** your data directory. The CLI prompts for confirmation:

```
Warning: This will delete all data for mariadb 11.4.5!
Type 'yes' to confirm:
```
:::
