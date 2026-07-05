# futou Lessons

## 1. Bundle overwrite
Bundle is written to disk once on first launch but read from disk thereafter.
**Rule:** Always overwrite on startup during dev, or delete `%APPDATA%\.futou\catalogue\bundle.json`.
**File:** `futou-daemon/src/composition_root.rs`

## 2. Zip top-level dir ≠ bin_dir
MariaDB zip extracts to `mariadb-11.4.5-winx64/` → `bin_dir` must be `"mariadb-11.4.5-winx64/bin"`, not `"bin"`.
**Rule:** Unzip one archive, inspect the output tree, verify `bin_dir` before committing.

## 3. Stale state.json
Fixing bundle data doesn't fix old `installation.path` in `%APPDATA%/.futou/state.json`.
**Rule:** After changing `bin_dir` in bundle, user must uninstall+reinstall. Consider auto-migration.

## 4. invoke() without catch = silent failure
`await invoke("runtime_activate", ...)` rejects silently — user sees nothing.
**Rule:** Every `invoke` must be wrapped in `try/catch` with `alert()`.

## 5. Key collisions from runtime name alone
`setOpenMenu(r.runtime)` opens menus on ALL rows with that runtime name.
**Rule:** Compound keys: `${runtime}-${version}`.

## 6. Modal overflow needs inner scroll container
`overflow-y:auto` on modal itself clips content. Use flex column: pinned header + scrollable body.
**Rule:** `<modal class="flex flex-col"><header /><div class="flex-1 overflow-y-auto min-h-0">{items}</div></modal>`.

## 7. Cross-crate deps
Adding `tracing` to `futou-core` means adding it to `futou-core/Cargo.toml`, not just `futou-daemon/Cargo.toml`.
**Rule:** Check Cargo.toml of the crate that USES the import.

## 8. Windows file lock on build
Running `.exe` locks it → `cargo build` fails with "Access is denied".
**Rule:** `Get-Process <name> | Stop-Process -Force; Start-Sleep -Seconds 5; cargo build`

## 9. Tauri: frontend build then cargo build
Tauri embeds `dist/` into the binary. `cargo build -p futou-gui` does NOT rebuild the frontend.
**Rule:** `bun run build` first, then `cargo build` for the Tauri binary.

## 10. Named pipe pool
One pipe instance = one client. Tauri fires concurrent commands.
**Rule:** Pre-create 4 instances, spawn each client, replenish on disconnect.

## 11. Progress callbacks in spawned tasks
`&str` has lifetime issues in `Box<dyn Fn>`. Use `String`.
**Rule:** `Box<dyn Fn(f64, String) + Send + Sync + 'static>`.

## 12. Stale aria2c blocks port
**Rule:** Random port via `TcpListener::bind("127.0.0.1:0")`, PID file, kill previous PID on startup.

## 14. Adding fields to persisted structs = `#[serde(default)]`
Adding `version_dir` to `Installation` broke all existing `state.json` files at runtime.
**Rule:** Every new `serde` field on a persisted type gets `#[serde(default)]`. Old disk state won't have it.
**File:** `futo-core/src/domain/runtime.rs`

## TODO (backlog)
- Bundle `futou-cli.exe` in the installer alongside the GUI and daemon
- Register CLI in system PATH during install
- Log panel: tail server process stdout/stderr live
- .bat shims for postgres/mariadb should include init scripts (mysql_install_db, initdb)
- Config button: for PHP, auto-copy php.ini-development → php.ini if missing
- aria2c resolution: `Path::exists()` on Windows needs `.exe` suffix check
