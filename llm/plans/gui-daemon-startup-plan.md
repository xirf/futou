# GUI daemon startup plan

## Scope

Make the selected installation directory control daemon storage and make the GUI
load daemon-backed data automatically after daemon startup.

## Plan

- [x] Pass the configured installation directory to the daemon process.
- [x] Resolve and validate the daemon data directory from its process arguments.
- [x] Wait for daemon readiness before the GUI's first runtime and catalogue load.
- [x] Cover directory resolution and update the architecture map.
- [x] Run formatting, linting, tests, and GUI build checks.

## Work log

- Created the canonical lessons and coding-style documents.
- Root causes confirmed: the daemon ignored GUI settings, and the frontend loaded
  data before the named pipe became ready.
- Added `--data-dir` propagation for GUI starts and Windows login autostart.
- Made initial and manual daemon starts reload runtimes and catalogue after the
  named pipe reports ready.
- Verified with `cargo test -p futou-daemon`, clippy with warnings denied for
  `futou-daemon` and the Tauri manifest, `cargo fmt --all --check`, and
  `bun run build`.
