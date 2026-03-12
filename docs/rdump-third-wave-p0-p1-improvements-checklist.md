# rdump Third-Wave P0/P1 Improvements Checklist

Source: [rdump-500-observations-assessment.md](./rdump-500-observations-assessment.md)

This is a third, additional set of `P0` and `P1` improvements derived from the assessment.
It is intentionally non-overlapping with:

- [rdump-p0-p1-improvements-checklist.md](./rdump-p0-p1-improvements-checklist.md)
- [rdump-second-wave-p0-p1-improvements-checklist.md](./rdump-second-wave-p0-p1-improvements-checklist.md)

## P0

- [x] Define stable match-coordinate semantics including line base, column base, and byte-vs-char offset rules in every machine-facing contract. `(Assessment: obs. 59-60, 245)`
- [x] Expose an explicit `match_kind` or `result_kind` so metadata-only whole-file matches are distinguishable from range-based content matches across CLI JSON, SDK, and MCP. `(Assessment: obs. 61, 310-312, 344-349)`
- [x] Expose both display path and resolved path, or explicitly standardize one as the contract path identity, so root-relative rewriting and canonicalization do not stay ambiguous for machine consumers. `(Assessment: obs. 156, 188-189)`
- [x] Surface canonicalization fallback as diagnostics or strict-mode errors instead of silently reverting to original paths when canonicalization fails. `(Assessment: obs. 188-189)`
- [x] Add a strict semantic-evaluation mode that reports per-file parse failures as surfaced statuses instead of silently converting them into non-matches. `(Assessment: obs. 143-144, 266-267)`
- [x] Add CI validation that every language-profile query uses required capture conventions such as `@match` and that declared predicate support stays internally consistent. `(Assessment: obs. 275-276, 299)`
- [x] Add explicit per-item truncation fields in MCP outputs so hosts can tell which files, matches, or snippets were shortened, not just that the overall response truncated. `(Assessment: obs. 409, 414-415)`
- [x] Add MCP continuation or pagination support for truncated result sets so agent workflows can safely retrieve the remainder without rerunning from scratch. `(Assessment: obs. 398-409)`
- [x] Surface "content suppressed for safety" indicators in human outputs so matching files do not disappear silently when formatter policy suppresses their contents. `(Assessment: obs. 307-312)`
- [x] Add stable ordering guarantees and tests for result lists across CLI, SDK, and MCP, including tie-breaking rules where path order is not enough. `(Assessment: obs. 155-156, 407-409)`
- [x] Add a deprecation-backed compatibility layer for legacy query spellings like `content:` so older docs and scripts fail predictably with migration guidance. `(Assessment: obs. 107-108, 148-150)`
- [x] Add stable search-status codes for full success, partial success, truncated success, invalid query, and policy-suppressed output across CLI exit behavior, SDK responses, and MCP tool envelopes. `(Assessment: obs. 99-100, 223-225, 398-419)`
- [x] Add per-file semantic-skip reason metadata such as `unsupported_language`, `parse_failed`, `content_unavailable`, or `budget_exhausted` so false negatives are inspectable. `(Assessment: obs. 198-200, 266-267, 372-374)`
- [x] Add release-time schema snapshots for CLI JSON and MCP structured outputs so machine-readable envelopes cannot drift unnoticed between versions. `(Assessment: obs. 93-94, 403-406, 461-463)`
- [x] Add cross-platform contract tests for `find` permissions, timestamps, and path fields so the current non-Unix placeholder behavior cannot surprise automation later. `(Assessment: obs. 321-323)`

## P1

- [x] Add parser error hints specifically for missing explicit `AND` so casual users get actionable recovery guidance instead of only generic parse failures. `(Assessment: obs. 103-104, 110)`
- [x] Add a registry-driven alias and help command that lists predicate aliases like `contains` and `c`, plus deprecated spellings, from live metadata. `(Assessment: obs. 105-108, 148-150)`
- [x] Add a capability-matrix report or resource for language-by-predicate support so users can see what semantic queries are actually available before running them. `(Assessment: obs. 140-144, 298-300, 432)`
- [x] Add per-language support tiers such as `stable`, `experimental`, or `partial` so the broad language catalog becomes easier to trust operationally. `(Assessment: obs. 35-39, 299, 431-432)`
- [x] Add a query advisor that warns when semantic-heavy queries lack narrowing metadata filters and are likely to fan out expensively. `(Assessment: obs. 131-135, 176, 384-385)`
- [x] Add file-level explain traces that show why a specific file matched or failed during metadata and full evaluation passes. `(Assessment: obs. 173-177, 198-200, 241-245)`
- [x] Add a "why no results?" report that distinguishes invalid query, unsupported language, all files filtered, and zero true semantic matches. `(Assessment: obs. 139-144, 198-200, 383-385)`
- [x] Add profile-lint tooling that checks alias profiles for drift against their canonical language definitions instead of assuming duplicated profile entries stay aligned. `(Assessment: obs. 286-288, 299-300)`
- [x] Add generated docs showing capture conventions and supported semantic predicates for each language profile so contributors do not have to infer them from query strings. `(Assessment: obs. 275-276, 298-300, 466-470)`
- [x] Add a strict SQL mode that errors when dialect-specific parsing fails instead of silently falling back to generic SQL semantics. `(Assessment: obs. 258-266)`
- [x] Add a dialect-detection report or debug output that explains why a SQL file was classified as PostgreSQL, MySQL, SQLite, or generic SQL. `(Assessment: obs. 258-263, 264-266)`
- [x] Add semantic parse-failure counters by language profile to search stats so flaky or weak profile coverage is visible over time. `(Assessment: obs. 266-268, 383-385)`
- [x] Add parse-tree cache hit and miss metrics for semantic evaluation so the value of parse reuse can be measured rather than assumed. `(Assessment: obs. 191-192, 264-265, 381-385)`
- [x] Add structured fallback reasons when `search_iter` rereads changed or missing files so callers can distinguish snapshot drift from ordinary I/O failures. `(Assessment: obs. 182-184, 233-244)`
- [x] Add an optional snapshot-ish mode that records file metadata at evaluation time and emits drift warnings when iteration later sees changed content. `(Assessment: obs. 183-184, 241-244)`
- [x] Add a machine-readable CLI JSON envelope version and schema reference separate from the human-oriented output formats. `(Assessment: obs. 92-94, 313-315, 344-345)`
- [x] Add a structured `find` JSON mode that exposes size, mtime, permissions, and path metadata without requiring consumers to scrape the human `find` output. `(Assessment: obs. 321-323, 344-345)`
- [x] Add explicit header and format-resolution rules, plus user-facing warnings, for ambiguous combinations such as `--find` with `--no-headers`. `(Assessment: obs. 64-67, 91-92)`
- [x] Add a CLI flag or config option to show placeholder entries when content was suppressed, rather than omitting the file entirely from `cat`, `hunks`, or `markdown` output. `(Assessment: obs. 307-312)`
- [x] Add output-parity docs that map each CLI and MCP mode to its intended audience, truncation behavior, and content-safety policy. `(Assessment: obs. 82, 342-350, 404-406)`
- [x] Add per-format regression tests for line-ending fidelity, especially comparing hunks, `cat`, CLI JSON, and MCP snippets. `(Assessment: obs. 326-328, 412-413)`
- [x] Add a stable `FileIdentity` helper type in the SDK for display path, resolved path, and root relationship so adapters stop reconstructing those semantics ad hoc. `(Assessment: obs. 156, 188-189, 245-246)`
- [x] Add an async search session ID or correlation token that can tie together progress, diagnostics, and final stats in host integrations. `(Assessment: obs. 380-390, 429)`
- [x] Add overload and queue metrics around async and MCP concurrency guards so hosts can tell when searches are waiting versus actively executing. `(Assessment: obs. 353-368, 393-394)`
- [x] Add preset execution-policy profiles such as `interactive`, `batch`, and `agent` that bundle sensible limits, truncation, and error-handling defaults. `(Assessment: obs. 36, 390, 396-399)`
- [x] Add a CLI or MCP "doctor" self-check command that validates environment assumptions like config presence, writable temp area, active limits, and concurrency defaults. `(Assessment: obs. 25, 165-166, 380-390, 430)`
- [x] Add stdio soak tests that exercise large responses, truncation, and repeated requests so host-integration regressions are caught beyond today's happy-path end-to-end cases. `(Assessment: obs. 398-415, 434-436)`
- [x] Add release smoke tests for published binary UX, not just crate-internal library behavior, so packaging and real CLI ergonomics are validated before release. `(Assessment: obs. 24-25, 431-435, 464)`
- [x] Add a generated support matrix from tests that shows which semantic scenarios are covered per language and which are intentionally language-specific. `(Assessment: obs. 432, 455-458)`
- [x] Add release automation or a checklist step that regenerates all code-derived docs and MCP resources before publish. `(Assessment: obs. 451, 464, 466-470)`
- [x] Add changelog entries specifically for contract-surface changes in CLI JSON, SDK result types, and MCP tools or resources. `(Assessment: obs. 459-460, 464, 468-470)`
- [x] Add operational SLO-style targets for first-result latency, total latency, and result-size caps so future optimization work has explicit goals. `(Assessment: obs. 31-36, 379-390, 499-500)`
- [x] Add workload-archetype docs and benchmarks for "local operator grep replacement", "AI agent snippet lookup", and "large monorepo semantic search" so tuning follows real usage modes. `(Assessment: obs. 33-36, 377-379, 499-500)`
- [x] Add cross-platform docs and tests for hidden-file, ignore, and permission semantics so operator expectations remain portable across environments. `(Assessment: obs. 167-170, 321-323)`
- [x] Add a machine-readable "supported predicates" resource in MCP and CLI so IDEs, wrappers, and agents can build query UIs without scraping prose docs. `(Assessment: obs. 77-79, 117-119, 422-424)`
