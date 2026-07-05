# Deno

Deno binaries from [deno.land](https://deno.land/).

## Versions

| Version | Source |
|---------|--------|
| 2.3.8 | Latest |
| 2.2.11 | Previous stable |

## Activation

```bash
futou use deno 2.3.8
```

Creates a `.bat` shim for `deno.exe`. Deno is a single-binary runtime — no `node_modules`, no `package.json`.

## Key Features

- **TypeScript natively**: no build step, no tsconfig
- **Secure by default**: `--allow-net`, `--allow-read`, etc.
- **Built-in tooling**: formatter, linter, test runner, bundler

```bash
deno run main.ts
deno run --allow-net server.ts
deno test
deno fmt
```

## npm Compatibility

Deno 2.x supports npm packages directly:

```bash
deno add npm:express
```
