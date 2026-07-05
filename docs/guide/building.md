# Building from Source

## Prerequisites

- Rust 1.85+ with `stable` toolchain
- Bun 1.3+ (for GUI frontend)
- Git

## Build All Crates

```bash
git clone https://github.com/xirf/futou.git
cd futou
cargo build --release
```

Output binaries in `target/release/`:
- `futou-daemon.exe`
- `futou-cli.exe`

## Build the GUI

```bash
cd futou-gui
bun install
bun run tauri build
```

The Tauri build command first compiles the daemon (`cargo build -p futou-daemon`), then bundles it alongside the GUI. The output is an NSIS installer in `futo-gui/src-tauri/target/release/bundle/`.

## Development

### Daemon + CLI only

```bash
cargo run --bin futou-daemon   # Terminal 1
cargo run --bin futou-cli -- catalogue  # Terminal 2
```

### GUI with hot reload

```bash
cargo run --bin futou-daemon   # Terminal 1
cd futou-gui && bun run tauri dev  # Terminal 2
```

::: tip File Lock
Windows locks running `.exe` files. If `cargo build` fails with "Access is denied", kill the daemon first:
```powershell
taskkill /f /im futou-daemon.exe
cargo build
```
:::

## Running Tests

```bash
cargo test -p futou-core    # Domain logic tests (28 tests)
cargo test --workspace       # All crates
```
