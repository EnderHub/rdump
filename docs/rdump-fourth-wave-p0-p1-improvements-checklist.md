# rdump Fourth-Wave P0/P1 Improvements Checklist

Source: [rdump-500-observations-assessment.md](./rdump-500-observations-assessment.md)

This is a fourth, additional set of `P0` and `P1` improvements derived from the assessment.
It is intentionally non-overlapping with:

- [rdump-p0-p1-improvements-checklist.md](./rdump-p0-p1-improvements-checklist.md)
- [rdump-second-wave-p0-p1-improvements-checklist.md](./rdump-second-wave-p0-p1-improvements-checklist.md)
- [rdump-third-wave-p0-p1-improvements-checklist.md](./rdump-third-wave-p0-p1-improvements-checklist.md)

## P0

- [x] Split the clap-driven CLI surface behind a dedicated crate or feature so library evolution is no longer pinned to accidental public CLI API baggage. `(Assessment: obs. 51-52, 71-76, 459-460)`
- [x] Add a versioned config schema plus migration warnings so local and global config evolution cannot silently change behavior across releases. `(Assessment: obs. 88-89, 459-460)`
- [x] Add config validation that fails fast on malformed presets, unknown referenced presets, or broken merged config before search execution begins. `(Assessment: obs. 88-89, 111-113)`
- [x] Add a stable machine-readable AST or plan export for queries so tooling can inspect structure without scraping `query explain` prose. `(Assessment: obs. 109-110, 137-145)`
- [x] Add typed predicate-value planning for `size:` and `modified:` so invalid units or relations stop leaking into runtime-only failures. `(Assessment: obs. 127-130)`
- [x] Add a repo-aware preflight that inspects the current root and warns when a semantic query has zero plausible target files before the expensive pass runs. `(Assessment: obs. 139-144, 176)`
- [x] Add an ignore-debug mode that reports which ignore source or pattern excluded a path so discovery failures are explainable. `(Assessment: obs. 167-170, 89)`
- [x] Reuse file metadata between evaluation and materialization so drift reporting and later shaping use the same baseline instead of fresh ad hoc stats. `(Assessment: obs. 181-184, 188-190)`
- [x] Add aggregate content-suppression counters by reason, not just per-result state, so operators can see how much of a search was hidden by policy. `(Assessment: obs. 195-200, 307-309)`
- [x] Strengthen snapshot-drift detection with richer file identity signals where available, rather than relying only on length and modified time. `(Assessment: obs. 183-184, 241-244)`
- [x] Add CI checks that tree-sitter grammar upgrades preserve expected capture behavior for each supported language profile. `(Assessment: obs. 275-276, 299, 431-435)`
- [x] Generate docs for per-predicate semantic matching rules such as exact, substring, and wildcard behavior across language families. `(Assessment: obs. 277-284, 298-300)`
- [x] Add false-positive regression suites for permissive predicates like `call`, `import`, `comment`, and `str` so recall improvements do not silently destroy precision. `(Assessment: obs. 277-282, 431-435)`
- [x] Surface explicit CLI truncation markers for matches, snippets, and full-content output so shortened human output cannot be mistaken for complete output. `(Assessment: obs. 307-309, 409, 414-415)`
- [x] Add TTL and memory limits for cached MCP pagination sessions plus explicit session-expired errors so continuation support does not imply unbounded retained state. `(Assessment: obs. 398-409, 392-394)`
- [x] Add continuation-token versioning and integrity metadata so stale, tampered, or cross-version tokens fail predictably instead of ambiguously. `(Assessment: obs. 398-409, 459-469)`

## P1

- [x] Add a builder or fluent API around `SearchOptions` so callers are less coupled to the raw struct layout as options continue to grow. `(Assessment: obs. 51-69)`
- [x] Make CLI dependencies opt-in for library consumers through feature gating or packaging separation to reduce compile footprint and accidental surface area. `(Assessment: obs. 51-52, 71-76)`
- [x] Add preset descriptions, tags, and examples so preset discovery becomes useful without opening config files directly. `(Assessment: obs. 88-89, 48)`
- [x] Add `config validate` and `config doctor --json` so automation can inspect config health without parsing prose output. `(Assessment: obs. 89, 25, 380-390)`
- [x] Add preset provenance output that shows which preset contributed each clause to the final effective query. `(Assessment: obs. 111-113, 145-146)`
- [x] Add a query simplifier that removes redundant parentheses and duplicate clauses after normalization. `(Assessment: obs. 109-146)`
- [x] Add stable AST serialization for cache keys, editor tooling, and contract tests. `(Assessment: obs. 109-110, 137-138)`
- [x] Expand literal and escaping docs and tests for quotes, backslashes, glob characters, and Unicode in predicate values. `(Assessment: obs. 101-110, 147-150)`
- [x] Add a repo-language inventory command or resource that reports dominant extensions and semantic-capable languages under the current root. `(Assessment: obs. 140-144, 432-435)`
- [x] Add a path-display policy option for CLI and MCP such as `relative`, `absolute`, or `root-relative` so consumers can choose path identity intentionally. `(Assessment: obs. 156, 188-189)`
- [x] Add root-boundary diagnostics that explain when files were excluded by canonical-root safety rules. `(Assessment: obs. 157, 188-189)`
- [x] Add filesystem bucket stats such as hidden-skipped, ignore-skipped, max-depth-skipped, and unreadable-entry counts. `(Assessment: obs. 167-172, 380-385)`
- [x] Add directory-hotspot reporting so users can see which subtrees dominate walk time or candidate volume on slow searches. `(Assessment: obs. 171-176, 380-385)`
- [x] Add opt-in lazy `Match.text` extraction or a no-text mode for machine consumers that only need coordinates. `(Assessment: obs. 57-62, 227-230, 241-245)`
- [x] Add aggregate counts for whole-file matches versus ranged matches in search stats. `(Assessment: obs. 61-62, 381-385)`
- [x] Add stable per-result fingerprints built from file identity and snapshot metadata so paginated UIs and caches can deduplicate safely. `(Assessment: obs. 156, 183-184, 245-246)`
- [x] Add non-SQL language-selection debug output that explains why a file mapped to a specific semantic profile or to none. `(Assessment: obs. 254-257, 298)`
- [x] Add SQL dialect confidence or heuristic-trace reporting so ambiguous `.sql` classifications are easier to trust or override. `(Assessment: obs. 258-263)`
- [x] Add capture-lint rules beyond `@match`, such as unused captures, duplicate roles, or missing capture conventions for comment and string queries. `(Assessment: obs. 275-276, 299-300)`
- [x] Add profile metadata for known semantic caveats so partial or permissive behaviors are documented per language. `(Assessment: obs. 277-284, 298-300)`
- [x] Add cross-language conformance tests for wildcard, substring, and case-sensitivity semantics where behavior is meant to be shared. `(Assessment: obs. 277-284, 455-458)`
- [x] Add negative semantic fixtures for near-miss identifiers to catch precision regressions from permissive matching. `(Assessment: obs. 281-282, 431-435)`
- [x] Add explicit line-ending mode selection across CLI text outputs so reproducible exports can choose `preserve` or `normalize`. `(Assessment: obs. 316-328)`
- [x] Add suppression and truncation summary footers to human outputs with counts by reason. `(Assessment: obs. 307-309, 409)`
- [x] Add deeper TTY and color-policy regression tests for `auto`, `always`, and `never` across multiple formats. `(Assessment: obs. 318-320, 336-339)`
- [x] Add escaping rules for tabs, newlines, control characters, and odd paths in human outputs so copy/paste and scraping are less ambiguous. `(Assessment: obs. 313-315, 321-323, 344-345)`
- [x] Add a CLI format that prints per-file diagnostic summaries alongside results instead of relying on global logs alone. `(Assessment: obs. 307-309, 388-389)`
- [x] Add finer-grained async progress milestones for walk, prefilter, evaluate, and materialize phases rather than only result-count updates. `(Assessment: obs. 380-386)`
- [x] Add overload diagnostics when searches spend too long queued behind concurrency guards, not just raw wait-time fields. `(Assessment: obs. 393-394, 380-390)`
- [x] Add MCP progress or notification support so hosts can observe long-running searches without waiting for the final tool result. `(Assessment: obs. 398-430)`
- [x] Add cached-session eviction metrics and operator docs for MCP pagination memory behavior. `(Assessment: obs. 398-409, 430)`
- [x] Add schema examples and sample request or response payloads to MCP docs and resources so host integrations can validate contracts faster. `(Assessment: obs. 403-430, 466-470)`
- [x] Extend executable example sync checks beyond the README to runtime, stdio, output-parity, and other query-bearing docs. `(Assessment: obs. 24-25, 77-79, 148-150, 463)`
- [x] Add platform-support docs that state which OSes, shells, time semantics, and filesystem behaviors are part of the supported operational contract. `(Assessment: obs. 24-25, 321-323, 431-435, 460)`
