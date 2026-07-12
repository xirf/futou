# Security Catalogue Hardening Plan

## Checklist

- [x] Validate runtime, version, catalogue paths, and safe uninstall containment.
- [x] Serialize repository mutations and persist state atomically with backup recovery.
- [x] Require SHA-256 and verify signed catalogue snapshots.
- [x] Add catalogue generator and CI validation/update workflows.
- [x] Update architecture documentation and verify the workspace.

## Decisions

- Catalogue discovery runs in maintainer CI, never in the daemon.
- Remote catalogue snapshots use detached Ed25519 signatures.
- Missing or malformed checksums fail closed.
- Python is omitted until a developer-friendly portable distribution is selected.
- Apache is deferred because the current Apache Lounge download path downgrades HTTPS to HTTP.

## Work Log

- Implementation started from the approved P0 security plan.
- Added exact Windows component validation, staging installs, checksum enforcement,
  canonical uninstall identity checks, and atomic duplicate rejection.
- Replaced repository load/mutate/save flows with serialized transactions and
  Windows atomic replacement, including backup recovery tests.
- Added catalogue schema v2, pinned Ed25519 verification, signed cache/bundle
  handling, six-provider generation, and reviewed PostgreSQL/nginx pins.
- Omitted Python by product decision and Apache because its HTTPS download
  endpoint currently redirects to HTTP, violating the fail-closed policy.
- Added Windows CI, scheduled catalogue PR generation, signature gates, and
  frontend build verification.
