# Catalogue generator

Maintainer-only tool for producing and signing Futou's Windows catalogue.

```powershell
cargo run -p catalogue-generator -- validate catalogue.json
cargo run -p catalogue-generator -- verify catalogue.json catalogue.json.sig futo-daemon/resources/catalogue-public-key.hex
cargo run -p catalogue-generator -- generate catalogue.json
$env:CATALOGUE_SIGNING_KEY = '<64 hex characters encoding a 32-byte Ed25519 secret>'
cargo run -p catalogue-generator -- sign catalogue.json catalogue.json.sig
cargo run -p catalogue-generator -- keygen futo-daemon/resources/catalogue-public-key.hex
```

`generate` discovers PHP, Node.js, MariaDB, and Deno from structured official
endpoints, then merges the reviewed PostgreSQL and nginx pins. It writes output
only after the complete schema validates. The signing key is supplied only
through the CI secret store.

## Signing key rotation

`CATALOGUE_SIGNING_KEY` is exactly 64 hexadecimal characters encoding the
32-byte Ed25519 secret. Detached signatures are 128 lowercase hexadecimal
characters. Never store the private key in this repository.

To rotate it, generate a new Ed25519 key offline, update the pinned public key
in `futo-daemon/resources/catalogue-public-key.hex`, update the Actions secret,
and sign both catalogue copies. Release the daemon with the new public key
before publishing catalogues signed only by that key.

PostgreSQL and nginx are maintained in `providers-pinned.json`; their hashes
are computed during reviewed ingestion because the publishers do not expose a
machine-readable digest feed. Apache Lounge is temporarily omitted: its HTTPS
artifact and checksum endpoints redirect to HTTP, which violates Futou's
fail-closed HTTPS policy. Add it only after end-to-end HTTPS is available or
the ingestion pipeline verifies a trusted publisher signature.
