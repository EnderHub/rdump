# Changelog

## Unreleased

### Contract surface

- Added stable search statuses, coordinate semantics, result kinds, file identity metadata, and semantic skip reasons across SDK and MCP payloads.
- Added machine-readable predicate and language-capability resources plus generated docs under `docs/generated/`.
- Added structured path metadata for machine-readable `find`/path outputs.
- Added MCP continuation tokens backed by cached server-side pages instead of client reruns.

### CLI

- Added `query reference --json`, `query why-no-results`, `query why-file`, `query dialect`, `lang matrix --json`, and `config doctor`.
- Added stable CLI invalid-query exit code `2`.
- Added JSON envelopes with explicit schema version, status, and truncation fields.

### Testing and operations

- Added generated-doc drift checks, shared support-matrix generation, and additional MCP soak coverage.
- Added release regeneration and smoke-check guidance for generated docs and binary UX.
