# rdump MCP stdio Integration Guide

## Process model

- Launch `rdump-mcp` as a long-lived stdio subprocess.
- Requests are independent and safe to pipeline within the configured concurrency limit.
- Search work runs in blocking tasks and uses cancellation tokens when the host drops the request.

## Safe defaults

- Start with `validate_query`, then `explain_query`, then `search`.
- Prefer `output=summary` or `output=paths` first.
- Keep default limits unless a host explicitly needs more output.
- Treat `schema_version` as the contract key for structured payloads.
- Follow `continuation_token` exactly as returned; tokens are versioned and integrity-checked.

## Concurrency and limits

- `RDUMP_MCP_MAX_CONCURRENT_SEARCHES` controls concurrent searches.
- `RDUMP_MCP_SESSION_TTL_SECONDS` bounds cached continuation-session lifetime.
- `RDUMP_MCP_MAX_CACHED_SESSIONS` bounds retained pagination sessions in memory.
- `execution_budget_ms` bounds total search runtime.
- `semantic_budget_ms` and `max_semantic_matches_per_file` bound semantic evaluation.
- `error_mode` chooses `skip_errors` vs `fail_fast`.

## Integration notes

- Resources expose docs, capabilities, active config, and presets.
- `rdump://docs/session-cache` exposes live session-cache limits and eviction counters for operators.
- `rdump://docs/schema-examples` exposes sample request and response payloads for contract validation.
- Hosts that include `_meta.progressToken` on `tools/call` requests for `search` receive `notifications/progress` updates before the final tool result.
- Prompts expose onboarding and conservative search workflow guidance.
- Tool results can be returned with `is_error=true` and a typed error payload; do not assume protocol-level failure for all tool errors.
