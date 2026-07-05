# Node.js

Node.js binaries from [nodejs.org](https://nodejs.org/dist/).

## Versions

| Version | Source |
|---------|--------|
| 22.23.1 | Current LTS |
| 20.20.2 | Maintenance LTS |

## Activation

```bash
futou use nodejs 22.23.1
```

Creates `.bat` shims for `node.exe`, `npm.cmd`, and `npx.cmd`.

## Included Tools

- **node**: JavaScript runtime
- **npm**: Package manager
- **npx**: Package runner

All three are available in PATH after activation.

## Global Packages

npm global packages install into `%APPDATA%\npm\` by default. This directory is outside futou's management and persists across version switches.
