# Operational Targets

These targets are the working SLO-style goals for `rdump`.

## Latency

- First result latency for metadata-only queries: `<= 150 ms` on a warm local repository under 50k files.
- First result latency for mixed metadata + semantic queries: `<= 500 ms` on a warm local repository under 50k files.
- Total latency for default `summary` searches in medium repositories: `<= 2 s`.

## Result shaping

- Default response envelope must stay under `200 KB` unless the caller explicitly raises limits.
- Default per-file content budget stays at `20 KB`.
- Default per-file match budget stays at `20` matches.

## Reliability

- Invalid queries should fail deterministically with stable `invalid_query` status or CLI exit code `2`.
- Partial failures must emit typed diagnostics instead of silently dropping context.
- Truncated result sets exposed to MCP must include a continuation token when additional cached pages remain.
