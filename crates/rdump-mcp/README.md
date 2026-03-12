# rdump-mcp

`rdump-mcp` exposes `rdump` search, language metadata, and query-inspection capabilities over the Model Context Protocol.

## Tools

- `search`
- `validate_query`
- `explain_query`
- `list_languages`
- `describe_language`
- `rql_reference`
- `sdk_reference`
- `capability_metadata`

## Resources

- `rdump://docs/rql`
- `rdump://docs/sdk`
- `rdump://docs/runtime`
- `rdump://docs/capabilities`
- `rdump://docs/languages`
- `rdump://docs/examples`
- `rdump://config/active`
- `rdump://config/presets`

## Contracts

- Structured tool outputs include `schema_version`.
- Tool errors use typed error payloads with `code`, `message`, and remediation hints.
- Requests accept `error_mode` so hosts can choose `skip_errors` or `fail_fast`.
- Hosts should branch on `schema_version` instead of assuming payload stability across major changes.

## Limits and defaults

- Default output is `snippets`.
- Default limits come from `capability_metadata`.
- Error handling defaults to `skip_errors`.
- Prefer `summary` or `paths` for broad repo scans before escalating to `full`.

## Host expectations

- The server is stdio-based and stateless across requests.
- Search concurrency is bounded; tune with `RDUMP_MCP_MAX_CONCURRENT_SEARCHES`.
- Hosts should respect typed diagnostics, truncation flags, and typed errors instead of scraping plain text.
