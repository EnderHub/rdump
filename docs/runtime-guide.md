# rdump Runtime Guide

## Runtime controls

- `execution_budget_ms` or `--execution-budget-ms` bounds total search time.
- `semantic_budget_ms` or `--semantic-budget-ms` bounds semantic work per file.
- `error_mode=skip_errors` continues past per-file materialization failures.
- `error_mode=fail_fast` stops on the first per-file failure.
- `RDUMP_MAX_CONCURRENT_SEARCHES` and `RDUMP_MCP_MAX_CONCURRENT_SEARCHES` tune concurrency.

## Troubleshooting

- Use `rdump query explain` or the MCP `explain_query` tool before broad semantic searches.
- Prefer `output=summary` or `output=paths` first on large repositories.
- Inspect typed diagnostics, typed errors, and `schema_version` before retrying with larger limits.
- Use explicit language overrides for extensionless or ambiguous files.
- If searches are truncated, increase limits or narrow the query with `ext:` or presets.

## Partial failures

- CLI, SDK request execution, and MCP all support explicit error modes.
- Skipped files surface as typed diagnostics or typed `errors` entries rather than silent drops.
- During metadata-only evaluation, predicates absent from the active registry are treated as neutral and deferred to later stages.

## Library and request surfaces

- `search_with_stats` is the easiest SDK entry point when you need results plus engine stats and diagnostics.
- `search_path_iter` is the path-only SDK surface when a caller does not need file text.
- `SearchRuntime::with_backend(...)` lets SDK, MCP, or adapter code bind searches to a custom `SearchBackend` instead of the default real filesystem.
- `execute_search_request_with_runtime(...)` and `repo_language_inventory_with_runtime(...)` are the backend-aware request and planner entry points.
- `search_async_with_runtime(...)` and `search_async_with_runtime_and_progress(...)` mirror the same runtime seam for Tokio callers.
- Request/response payloads are versioned by `schema_version`; pin automation to that field rather than prose output.
