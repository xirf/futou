# futou Lessons

## 1. Bundle overwrite
Bundle is written to disk once on first launch but read from disk thereafter.
**Rule:** Only write `bundle.json` if it doesn't exist. Overwriting it wipes the remote fetch cache.
**File:** `futo-daemon/src/composition_root.rs`

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
**Rule:** `taskkill /f /im futou-daemon.exe; Start-Sleep -Seconds 2; cargo build`

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

## 15. Operator precedence in boolean chains
`data_dir.exists() && x.exists() || y.exists()` — `&&` binds tighter than `||`. `y.exists()` bypasses `data_dir.exists()`.
**Rule:** Always parenthesize mixed `&&`/`||` expressions. Clippy catches some but not all.
**File:** `futo-daemon/src/adapters/process_manager_win.rs`

## 16. `.unwrap()` in RPC handlers = daemon crash
`serde_json::to_value(...).unwrap()` panics if a struct field serializes badly. The daemon is a long-lived process — a panic kills all connected clients.
**Rule:** Wrap serialization in a helper that returns `INTERNAL_ERROR` on failure. Never unwrap in a handler.
**File:** `futo-daemon/src/handler.rs` → `json_success()` helper

## 17. Pipe reads have no default timeout
Named pipe reads block indefinitely. If the daemon crashes mid-response, CLI and GUI hang forever.
**Rule:** `tokio::time::timeout(Duration::from_secs(30), reader.read_line(...))` on all pipe reads.
**File:** `futo-cli/src/pipe_client.rs`, `futo-gui/src-tauri/src/lib.rs`

## 18. Install progress map never pruned
`tokio::spawn` inserts into `HashMap<String, InstallProgress>` on every install. Completed/failed entries stay forever.
**Rule:** Prune `stage == "completed" || stage == "failed"` entries on `daemon.status` polls (or after N minutes).
**File:** `futo-daemon/src/handler.rs`

## 19. Empty `version_dir` on pre-field state
Old installs saved before `version_dir` was added have `version_dir: ""`. Path joins against `""` produce broken paths.
**Rule:** Guard with `if version_dir.is_empty() { &installation.path } else { &installation.version_dir }`.
**File:** `futo-core/src/service/activation_service.rs → start_process`

## 20. Uninstall is irreversible with no confirmation
`cmd_uninstall` / `handle_runtime_uninstall` immediately deletes files + state. One typo deletes a PostgreSQL data dir.
**Rule:** Confirmation prompt. CLI: stdin `"yes"`. GUI: `window.confirm()`.
**File:** `futo-cli/src/main.rs`, `futo-gui/src/App.tsx`

## TODO (backlog)
- Bundle `futou-cli.exe` in the installer alongside the GUI and daemon
- Register CLI in system PATH during install
- Log panel: tail server process stdout/stderr live
- .bat shims for postgres/mariadb should include init scripts (mysql_install_db, initdb)
- Config button: for PHP, auto-copy php.ini-development → php.ini if missing
- aria2c resolution: `Path::exists()` on Windows needs `.exe` suffix check
- Shim cleanup on deactivate: track which shims belong to which runtime+version
- ActivationService: load → find → load → save is a race window (two lock acquisitions)
- Child process handle: dropped immediately after spawn, no health check on PID
- CLI install: `cmd_install` returns "installed successfully" before async install finishes
- Aria2 fallback: warn user when NullDownloader is active (installs won't work)
- Document root validation: check path exists before starting Apache/Nginx
- Graceful shutdown: `httpd.exe -k stop` / `nginx.exe -s quit` instead of `taskkill`
