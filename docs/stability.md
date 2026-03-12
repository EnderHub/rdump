# rdump Stability Policy

## CLI

- Command names, top-level flags, and machine-oriented output modes follow semver.
- Human-readable prose and examples may evolve between minor releases.
- Scripted consumers should prefer explicit formats and timestamp flags over default prose.

## SDK

- `search`, `search_iter`, `search_with_stats`, `search_path_iter`, `search_paths`, and request/contract execution APIs are semver-governed.
- CLI-facing clap structs remain available for compatibility, but new programmatic integrations should prefer `rdump_contracts::SearchRequest` plus `execute_search_request`.

## MCP

- Structured payloads are guarded by `schema_version`.
- New fields may be added compatibly.
- Breaking payload changes require a schema version change and release-note callout.
