# Activate & Deactivate

## Activating a Runtime

```bash
futou use <name> <version>
```

Activation does two things:
1. **Creates `.bat` shim files** in the shims directory (`%APPDATA%\.futou\shims\`). Each executable in the runtime's `bin/` directory gets a `.bat` wrapper.
2. **Adds the shims directory to `PATH`** via the Windows registry (`HKCU\Environment\PATH`). A `WM_SETTINGCHANGE` broadcast notifies running processes.

```bash
$ futou use php 8.4.23
php 8.4.23 is now active
```

After activation, `php`, `php-cgi`, and other PHP binaries are available from any terminal.

::: tip Multiple Activations
You can activate multiple runtimes simultaneously. For example, `futou use php 8.4.23` then `futou use nodejs 22.23.1` — both will be available in your PATH.
:::

## Deactivating a Runtime

```bash
futou deactivate <name>
```

Removes the runtime's shims from the shims directory and clears the active entry in state. Does **not** remove the runtime from PATH if other runtimes' shims are still present.

```bash
$ futou deactivate php
php deactivated
```

::: warning Shim Cleanup
Deactivation only removes shims for the specified runtime. Other runtimes' shims remain untouched. The daemon tracks which shims belong to which runtime via `shims.json`.
:::
