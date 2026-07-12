# futou Coding Style

## Goals

Code should be easy to locate, read, change, and delete. Prefer direct flows and
small modules over speculative abstractions.

## File and module naming

- Use lowercase `snake_case` for Rust files and modules.
- Name files after one responsibility or domain concept, not generic buckets.
  Prefer `daemon_process.rs` or `settings_store.rs` over `utils.rs`, `helpers.rs`,
  or `manager.rs`.
- Keep the public module name aligned with its filename.
- Put platform suffixes last when implementations differ, such as
  `process_windows.rs`.
- Use `PascalCase.tsx` for React component files and `camelCase.ts` for hooks.
  Component and hook exports must match the filename.
- Tests should live beside small Rust modules. Use `tests/<feature>.rs` only for
  cross-crate or external-behavior integration tests.

## Naming

- Name functions with a verb and their observable result: `load_settings`,
  `resolve_data_dir`, `wait_for_daemon`.
- Name types and modules with domain language already used by the UI and RPC.
- Avoid abbreviations except established terms such as `rpc`, `pid`, and `url`.
- Boolean names start with `is`, `has`, `can`, or `should`.

## Structure

- Keep transport, orchestration, domain logic, and filesystem/process I/O
  separate.
- A function should operate at one level of abstraction. Extract a helper when
  it gives a non-obvious operation a searchable domain name.
- Reuse existing types and dependencies before adding new ones.
- Prefer explicit arguments at process boundaries over duplicated configuration
  readers.

## Errors and documentation

- Return contextual errors at I/O and process boundaries; do not silently ignore
  failures that affect user-visible behavior.
- Follow the project Rustdoc rules for every public symbol and changed contract.
- Inline comments explain only non-obvious constraints or reasons.

## Verification

- Add the smallest test that fails for the reported bug.
- Run `cargo fmt --check`, clippy with warnings denied, and affected tests.
- For GUI changes, also run the frontend build and exercise the startup flow when
  a Windows binary can be launched locally.
