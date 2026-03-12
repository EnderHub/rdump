# rdump Deep Assessment: 500 Observations and Improvement Gaps

## Scope

This assessment is based on direct source reads plus `syntact-tracer` analysis over the primary entry points in `rdump` and `rdump-mcp`, especially:

- `rdump/src/lib.rs`
- `rdump/src/commands/search.rs`
- `rdump/src/parser.rs`
- `rdump/src/evaluator.rs`
- `rdump/src/predicates/mod.rs`
- `rdump/src/predicates/code_aware/mod.rs`
- `rdump/src/predicates/code_aware/profiles/mod.rs`
- `rdump/src/formatter.rs`
- `rdump/src/config.rs`
- `rdump/src/async_api.rs`
- `rdump-mcp/src/lib.rs`
- `rdump-mcp/src/search.rs`
- `rdump-mcp/src/languages.rs`
- `rdump-mcp/src/docs.rs`
- `rdump-mcp/src/types.rs`

`syntact-tracer` highlights used here included `build_context`, `trace`, `analyze_symbol_connections`, `find_reverse_dependencies`, and `list_symbols`.

## Executive Summary

The repo already has strong semantic-language coverage, a credible staged evaluation model, and unusually broad tests for a 0.1.x tool. The biggest opportunities are not "add more languages" but "make behavior consistent everywhere."

The highest-value gaps are:

- Content safety policy differs across evaluator, formatter, SDK iterator, and MCP.
- `search_iter` is only lazily loading file content; it is not lazily discovering/evaluating files.
- Tree-sitter queries are compiled per file/predicate call instead of being cached.
- Symlink behavior, canonicalization behavior, and user-facing tests/docs are not aligned.
- MCP language listing is built from an alias-keyed registry and likely emits duplicates.
- MCP docs and architecture docs are hand-maintained and already drift from live code.

## Key Source References

- Content safety split:
  `rdump/src/evaluator.rs:51-90`,
  `rdump/src/formatter.rs:31-62`,
  `rdump/src/lib.rs:425-459`
- Iterator eager/eager split:
  `rdump/src/lib.rs:478-487`,
  `rdump/src/lib.rs:535-587`
- Per-file tree-sitter query compilation:
  `rdump/src/predicates/code_aware/mod.rs:159-166`
- Symlink and canonicalization behavior:
  `rdump/src/commands/search.rs:263-266`,
  `rdump/src/commands/search.rs:315-317`,
  `rdump/src/lib.rs:89-100`,
  `rdump/tests/filesystem_edge_cases.rs:8-30`
- Duplicate/unstable MCP language inventory:
  `rdump/src/predicates/code_aware/profiles/mod.rs:39-85`,
  `rdump-mcp/src/languages.rs:5-10`
- Docs drift examples:
  `docs/architecture.md:177-199`,
  `docs/architecture.md:239-252`,
  `rdump/src/evaluator.rs:22-33`,
  `rdump/src/lib.rs:300-310`

## Repo Metrics

- Rust `src` lines in `rdump/src` and `rdump-mcp/src`: about 10,053.
- Rust test lines in `rdump/tests` and `rdump-mcp/tests`: about 15,287.
- Top-level `rdump/tests` files: 51.
- `code_aware/profiles` Rust files: 24.
- `syntact-tracer` summary for `rdump`: 45 files, 467 symbols, 19 cycles, graph depth 5.
- `syntact-tracer` summary for `rdump-mcp`: 7 files, 128 symbols, 2 cycles, graph depth 3.

## 500 Observations

### Repo Shape and Product Positioning

1. The repo is split into two Rust crates: `rdump` as the engine and `rdump-mcp` as the MCP adapter.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
2. `rdump` is the primary product surface; `rdump-mcp` is intentionally thin and delegates to it.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
3. The implementation footprint is smaller than the test footprint, which signals a quality-conscious project.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
4. The test corpus is large enough to be a product asset, not just a safety net.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
5. The repo structure suggests local developer workflows came first and agent workflows came second.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
6. `rdump` positions itself as both search and content-dump tool, not only a matcher.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
7. The name "rdump" is operationally accurate because many output modes emit content, not just paths.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
8. The architecture narrative in `docs/architecture.md` still maps to the broad design even where details have drifted.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
9. The parse -> evaluate -> format pipeline is easy to understand from both code and docs.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
10. The engine is library-first in spirit even though it still exposes many CLI types publicly.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
11. Public CLI types living in the same crate root as SDK types increase surface area.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
12. The MCP crate shows that the team sees structured programmatic access as important.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
13. There is no separate shared crate for schema or contracts between CLI, SDK, and MCP.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
14. That keeps the workspace simple but increases drift risk between output surfaces.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
15. The repo contains substantial BMAD process artifacts alongside product code.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
16. Those artifacts are helpful historically but can blur the live source of truth.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
17. The presence of stories and QA gates implies disciplined delivery, but not necessarily ongoing doc reconciliation.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
18. `rdump` is versioned `0.1.8`, while `rdump-mcp` is `0.1.0`, which suggests the MCP layer is newer and less mature.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
19. `rdump-mcp` depends on `rdump` by local path, which is appropriate for a workspace-level adapter.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
20. `rdump` still carries placeholder package authorship in `Cargo.toml`.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
21. The README still contains placeholder GitHub URLs, which weakens release-readiness.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
22. The README is much richer than the MCP documentation surface.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
23. There is no dedicated `rdump-mcp/README.md`, so the MCP product story is under-explained.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
24. The repo's strongest documentation is conceptual architecture plus the main README.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
25. The repo's weakest documentation is operational guidance for runtime behavior and MCP deployment.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
26. The source graph density in `rdump` is moderate, not chaotic, but there is meaningful internal coupling.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
27. The source graph density in `rdump-mcp` is low enough to keep that crate easy to reason about.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
28. `syntact-tracer` showing 19 cycles in `rdump` indicates some cross-module entanglement worth monitoring.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
29. `syntact-tracer` showing only 2 cycles in `rdump-mcp` supports the "thin adapter" interpretation.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
30. The engine crate has enough scale now that architectural drift becomes a real maintenance cost.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
31. The project has broad language ambitions without yet having a corresponding benchmark discipline.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
32. The absence of a persistent index means every query is "fresh scan, fresh evaluate."
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
33. That one-shot model is coherent with local CLI use and smaller agent requests.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
34. That same model becomes expensive once repos get large or repeated queries become common.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
35. The broad language coverage is already a differentiator for a small project.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
36. The operational story is not yet as differentiated as the language-coverage story.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
37. The tests reflect pride in semantic coverage across languages.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
38. The docs reflect pride in query expressiveness and layered evaluation.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
39. The next maturity step is consistency, not raw feature count.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
40. The current codebase looks like an engine graduating from "clever prototype" toward "reliable tool."
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
41. The architecture doc explicitly names non-goals like IDE integration and file watching.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
42. The MCP server partially broadens integration goals beyond those original non-goals.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
43. That does not invalidate the non-goals, but it does raise the bar for stable interfaces.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
44. The project is still simple enough that a focused reliability pass would have outsized payoff.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
45. The codebase size is large enough that hand-maintained docs have started to diverge.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
46. The codebase size is still small enough that those divergences are fixable without a rewrite.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
47. The large fixture inventory means the repo carries implicit behavioral knowledge in tests.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
48. Some of that knowledge is not surfaced in product docs.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
49. The repo already feels like a real tool, not just a parser demo.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md
50. The repo is one strong cleanup cycle away from having a much sharper operational story.
   Refs: rdump/Cargo.toml; rdump-mcp/Cargo.toml; rdump/README.md; docs/architecture.md

### Public API, CLI, and Config Surface

51. `SearchOptions` is the clean library-facing search configuration type.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
52. `SearchArgs` is the richer CLI-facing type with formatting and UX concerns mixed in.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
53. `search_iter` is the preferred SDK entry point for large repos.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
54. `search` is just a convenience collector around `search_iter`.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
55. `SearchResultIterator` is lazily loading file content, but it is not lazily discovering files.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
56. `search_iter` still pays the full candidate discovery and evaluation cost before the iterator is returned.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
57. `SearchResult` always contains `path`, `matches`, and full `content`.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
58. That means the SDK currently bakes content loading into its core success type.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
59. `Match` carries byte ranges and extracted text, which is useful for structured consumers.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
60. Line numbering is 1-indexed while columns are 0-indexed, which is explicit but easy to forget.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
61. `SearchResult::is_whole_file_match` cleanly distinguishes metadata-only matches from hunk matches.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
62. `SearchResult::matched_lines`, `match_count`, and `total_lines_matched` are practical helper methods.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
63. The CLI default format is `Hunks`.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
64. `--no-headers` is treated as an alias for `cat` behavior in practice.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
65. `--find` mutates the selected format instead of being its own command or mode enum only.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
66. Color behavior is centralized in `run_search`, which is the right place for it.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
67. There is a commented safeguard in `run_search` around `Cat` color handling, which suggests previous uncertainty.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
68. CLI root defaults to `"."`, which is ergonomic.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
69. `SearchOptions::default()` also uses `"."`, so SDK and CLI are aligned there.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
70. SQL dialect is exposed both as library `SqlDialect` and CLI `SqlDialectFlag`.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
71. The CLI exports `Cli`, `Commands`, `SearchArgs`, `LangArgs`, and `PresetArgs` from the crate root.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
72. `api_exports.rs` intentionally locks those exports in place as public API.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
73. That turns clap structs into semver-sensitive surface for downstream embedders.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
74. Public `commands` access from the crate root increases accidental API exposure further.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
75. `perform_search` in `commands::search` is tested as reachable from outside the crate.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
76. That choice is convenient now but may make internal refactors more expensive later.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
77. The CLI help text for RQL is richer than the MCP reference surface.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
78. The CLI docs enumerate React-specific predicates directly in the clap docs.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
79. The CLI surface supports `component`, `element`, `hook`, `customhook`, and `prop`.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
80. The CLI does not expose the MCP output vocabulary of `summary`, `matches`, or `snippets`.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
81. The SDK also does not expose those output shapes as first-class shared types.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
82. There are effectively separate user stories for CLI output and MCP output.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
83. `SearchArgs.query` is optional so presets can drive a search even with no raw query text.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
84. `perform_search` translates `SearchArgs` into `SearchOptions`.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
85. `run_search` also translates `SearchArgs` into `SearchOptions`.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
86. Those duplicate mappings can drift if new search options are added.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
87. `LangAction` defaults to `List`, which is a good ergonomics choice.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
88. `PresetAction` covers `List`, `Add`, and `Remove`, which is sufficient for a small local config story.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
89. There is no explicit config-inspection command beyond preset management.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
90. There is no "explain this query" or "dry-run planner" CLI mode.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
91. `Format` includes `Hunks`, `Markdown`, `Json`, `Paths`, `Cat`, and `Find`.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
92. Those formats are mostly human-oriented except JSON.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
93. CLI JSON emits whole-file content, not a compact result schema.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
94. CLI and SDK are closer to each other than CLI and MCP are.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
95. `SearchOptions` is `Send` and `Sync`, which is explicitly tested.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
96. `SearchResult` is also `Send` and `Sync`.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
97. `SearchResultIterator` is `Send` but not `Sync`, which is sensible.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
98. The thread-safety tests make it clear the team cares about cross-thread consumption patterns.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
99. Error-handling behavior differs across CLI, SDK, and MCP, which is the single biggest public-surface inconsistency.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs
100. A unified policy layer would likely improve user trust more than any new predicate would.
   Refs: rdump/src/lib.rs; rdump/src/config.rs; rdump/tests/api_exports.rs

### Query Language and Predicate Model

101. The parser uses `pest` with Pratt parsing, which is a solid fit for boolean query syntax.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
102. Empty queries are rejected explicitly unless presets produce a final query.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
103. Implicit `AND` is intentionally unsupported.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
104. That keeps parsing simpler but is a small UX tax for casual users.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
105. Aliases exist for `contains` (`c`) and `matches` (`m`).
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
106. Predicate keys are normalized into a single `PredicateKey` enum.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
107. Unknown keys become `PredicateKey::Other`, then fail during validation.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
108. This separation between parse and validation is simple and effective.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
109. `AstNode` has the expected small shape: predicate, logical op, and not.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
110. The parser tests are extensive enough that core grammar changes should be caught.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
111. Presets are merged into the raw query string before parsing.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
112. Preset expressions are parenthesized, which preserves intended boolean structure.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
113. The final query string is parsed only after preset expansion.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
114. Query parsing currently happens after candidate file discovery, which is backwards for invalid-query fast fail.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
115. Invalid query syntax therefore still pays filesystem-walk cost today.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
116. Unknown predicates also fail only after candidate file discovery today.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
117. Predicate validation is registry-based, not just enum-based.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
118. That means the system can change active predicate sets by swapping registries.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
119. The staged evaluator relies on that registry composition heavily.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
120. Missing predicates are treated as `true` in partial registries to support staged evaluation.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
121. That behavior is clever but subtle enough to deserve explicit design docs.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
122. The `NOT` path has to special-case missing predicate evaluators to keep staged logic sound.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
123. The code does that, which shows good attention to boolean semantics.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
124. The query model supports mixing boolean predicates and hunk-producing predicates in one tree.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
125. That is stronger than many simple semantic tools that force separate query modes.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
126. The combination semantics are expressive enough for real code-navigation workflows.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
127. The parser itself does not validate predicate-value formats like size units or modified units.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
128. That means some invalid values surface only at evaluation time.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
129. A more typed predicate AST could shift more failures to parse time.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
130. The current model favors low parser complexity over strong static validation.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
131. Query order typed by the user does not determine evaluation cost because of staged registries.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
132. That is good from a performance-stability perspective.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
133. It also means the engine could provide planner/explain output independent of user term order.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
134. The architecture doc talks about cost ordering as a principle.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
135. The live implementation achieves some of that through two-pass evaluation, not through AST reordering.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
136. There is no user-visible query planner output today.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
137. There is no canonical normalization API for queries today.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
138. There is no AST pretty-printer in the public API.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
139. There is no lint pass for "this semantic predicate is unsupported for your file types."
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
140. Query capability depends on both predicate key and per-language profile support.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
141. That dependency is not surfaced by the parser itself.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
142. There is no compile-time or query-time warning if a query can never match due to extension/profile mismatch.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
143. The current design assumes "unsupported" should just become non-match, not user-facing error.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
144. That is pragmatic, but it can hide intent mistakes.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
145. The parser and evaluator together already contain enough structure to implement a useful explain mode.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
146. The query language is expressive enough to justify that explain mode.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
147. The parser tests make a good foundation for adding more static validation later.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
148. Some older docs still use `content:` where live code uses `contains:`.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
149. That specific doc drift is risky because it affects query-writing directly.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md
150. The query language is one of the project's strongest assets and should be documented from code, not by hand.
   Refs: rdump/src/parser.rs; rdump/src/commands/search.rs; docs/architecture.md

### Search Pipeline and Evaluation Semantics

151. The live pipeline is candidate walk -> parse -> validate -> metadata pass -> full pass -> output conversion.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
152. Candidate walk before parse is wasted work for invalid queries.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
153. The metadata prefilter is single-threaded today.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
154. The full evaluation pass is parallelized with Rayon.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
155. Results are sorted by path after the parallel pass.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
156. Final result paths are rewritten relative to the original display root.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
157. `safe_canonicalize` ensures walked paths remain under the canonical root.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
158. The walker is configured with `follow_links(true)`.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
159. That means symlinks are followed in discovery.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
160. A filesystem-edge test claims symlinks are not followed by default, which conflicts with the live code.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
161. The likely reason that test still passes is path canonicalization collapsing the symlink path to the target path.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
162. There is no explicit deduplication after canonicalization.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
163. That means a real file reachable by multiple symlink paths may still be processed multiple times.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
164. The current symlink behavior is safe relative to root escape, but not clearly defined relative to duplicates or user expectations.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
165. Default ignore rules are written into a temporary ignore file on every search.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
166. That adds avoidable per-query filesystem work.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
167. The default ignore layer includes common build and VCS directories.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
168. Global ignore and project `.rdumpignore` are layered on top of those defaults.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
169. `--no-ignore` disables all those ignore sources, which is sensible.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
170. `hidden(!hidden)` delegates dotfile behavior to the ignore walker cleanly.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
171. Default max depth is 100, which is generous enough for deep repos.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
172. Root existence is checked via canonicalization and produces a good error message.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
173. The prefilter phase uses an `Evaluator` with a metadata-only registry.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
174. Content and semantic predicates evaluate as effectively neutral during that pass.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
175. This ensures correctness without custom AST rewriting.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
176. It also means semantic-heavy queries can still send large candidate sets into the expensive pass.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
177. The full pass builds a registry that includes content and semantic predicates.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
178. SQL dialect override is threaded into that full registry only where needed.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
179. `perform_search_internal` returns raw `tree_sitter::Range` values, not public `Match` values.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
180. That is a good boundary because it avoids tree-sitter types in the public API.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
181. `search_iter` converts raw ranges into public `Match` values later.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
182. That later conversion depends on rereading the file from disk.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
183. If files change between evaluation and iteration, reported ranges can become stale.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
184. The engine currently assumes a mostly stable filesystem during search.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
185. Errors found during the evaluation pass are captured via a shared `Mutex<Option<anyhow::Error>>`.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
186. That is a simple first-error abort mechanism for parallel code.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
187. Rayon tasks already in flight may still do some extra work before the abort is observed.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
188. `FileContext` canonicalizes both root and path when it is created.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
189. If canonicalization fails there, it silently falls back to the original path/root.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
190. `FileContext` caches content lazily as `Option<String>`.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
191. `FileContext` caches one parse tree plus its language key.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
192. SQL fallback reparses because the language key changes.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
193. `FileContext::get_content` uses `String::from_utf8_lossy`.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
194. That means invalid UTF-8 is tolerated inside evaluator and formatter paths.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
195. Oversized files become empty content inside the evaluator instead of hard errors.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
196. Binary files become empty content inside the evaluator instead of hard errors.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
197. Secret-like files become empty content inside the evaluator instead of hard errors.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
198. That makes content/semantic predicates quietly return false for those files.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
199. Metadata predicates can still match those same files.
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs
200. The content-loading policy is therefore not "these files are excluded"; it is "these files are invisible to content-aware logic."
   Refs: rdump/src/commands/search.rs; rdump/src/evaluator.rs; rdump/src/lib.rs

### Content Safety, Iterator Behavior, and Result Materialization

201. `read_file_content_for_iterator` uses strict UTF-8 conversion rather than lossy conversion.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
202. `read_file_content_for_iterator` does not apply the secret heuristic.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
203. `read_file_content_for_iterator` does apply binary detection and size limits.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
204. This differs from both `FileContext::get_content` and `formatter::read_file_content`.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
205. The SDK iterator can therefore error on invalid UTF-8 where the evaluator would have tolerated the file.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
206. The SDK iterator can therefore return secret-like file contents where the evaluator and formatter would have hidden them.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
207. A metadata-only query like `ext:rs` can produce whole-file SDK results for a secret-like file.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
208. The CLI formatter may later suppress that same file's content at print time.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
209. MCP `Full` output can expose that same file content because it depends on SDK results, not formatter safety logic.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
210. This is the highest-severity cross-surface inconsistency in the repo.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
211. The engine currently has no single source of truth for "is this file readable for content purposes."
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
212. The evaluator path answers that question one way.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
213. The formatter path answers it a second way.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
214. The iterator/SDK path answers it a third way.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
215. Those differences are not currently documented as an intentional policy choice.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
216. `SearchResultIterator` stores raw results in a `Vec`-backed `IntoIter`.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
217. That makes it efficient to implement `remaining`, `size_hint`, and `ExactSizeIterator`.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
218. It also means memory is committed for all raw results up front.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
219. `take(3)` on the iterator saves content-loading work but not walk/evaluate work.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
220. The tests for early termination validate API semantics, not computational savings.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
221. `SearchResultIterator` emits per-item errors instead of failing the whole iterator up front.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
222. That is useful for callers who want partial success.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
223. The CLI path does not currently expose that per-item resilience.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
224. MCP exposes it through `skip_errors`, which defaults to true.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
225. The public API is therefore stronger than the CLI on partial-failure handling.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
226. The public API is weaker than the formatter on content-safety filtering.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
227. Because `SearchResult` always carries content, there is no light-weight result form for metadata-only use.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
228. That forces callers who only want paths to still pay content-read cost during iteration.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
229. The MCP adapter works around that by truncating and reshaping after reading content.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
230. A path-only or metadata-only iterator variant would reduce work for several surfaces.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
231. The current iterator design is simple and ergonomic, which explains why it exists as-is.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
232. The main cost of that simplicity is hidden eager work and inconsistent content policy.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
233. `read_file_content_for_iterator` emits hard errors for missing files discovered after evaluation.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
234. That is correct but could surprise callers expecting snapshot-like behavior.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
235. There is no per-result metadata saying content was omitted due to size/binary/secret policy.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
236. There is no `ContentState` enum in the public API.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
237. Such an enum would make behavior far more explainable.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
238. The current `SearchResult` type is sufficient for demos but thin for operational tooling.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
239. The SDK is already close to supporting richer semantics because the internal states already exist.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
240. Consolidating them would strengthen both SDK and MCP.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
241. `ranges_to_matches` extracts matched text from the reread file content.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
242. If content changes between evaluation and iteration, match text can disagree with the original tree-sitter capture.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
243. That race is acceptable for local CLI use but worth documenting for agent integrations.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
244. The public API does not promise snapshot consistency, which is probably wise.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
245. The line/column mapping from tree-sitter ranges is straightforward and correct.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
246. `Match::first_line` is a small but useful helper for summary UIs.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
247. The iterator is correctly marked as `FusedIterator`.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
248. The iterator is correctly marked as `ExactSizeIterator`.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
249. The iterator shape is clean; the main issue is hidden eagerness, not API ugliness.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs
250. A future truly streaming pipeline could preserve the same iterator API while improving internals.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs

### Tree-sitter Evaluator and Language Profile System

251. `CodeAwareEvaluator` is the single semantic evaluator for all code-aware predicate keys.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
252. Centralizing semantic logic makes cross-language behavior more consistent.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
253. Centralizing semantic logic also makes the module a likely future hotspot.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
254. Language support is chosen by file extension for non-SQL files.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
255. Files without meaningful extensions cannot use semantic predicates even if content would be parseable.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
256. There is no shebang detection for shell-like files.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
257. There is no content-based fallback for ambiguous extensions.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
258. SQL is treated specially because `.sql` files need dialect selection.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
259. SQL dialect auto-detection lowercases the full file contents.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
260. Lowercasing the full file contents allocates another whole-file string.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
261. Auto-detection then runs a few regex and string heuristics over that lowercase content.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
262. The heuristics are intentionally cheap but not exhaustive.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
263. Generic SQL is the fallback if no heuristic triggers.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
264. The chosen SQL profile key is cached in `FileContext`.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
265. That caching is a good micro-optimization.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
266. Failed dialect-specific parse falls back to generic SQL and logs a warning.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
267. Failed non-SQL parse logs a warning and skips the file semantically.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
268. Those warnings go to stderr rather than a structured diagnostics channel.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
269. Tree-sitter queries are compiled inside `CodeAwareEvaluator::evaluate`.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
270. That means the same query is compiled again for every file.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
271. That is a significant avoidable cost on large repos.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
272. `docs/performance-optimizations.md` already identifies precompiled query caching as desirable.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
273. The evaluator clones `content` into a `String` to avoid borrow issues.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
274. That is another acknowledged performance cost.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
275. Query execution depends on `@match` capture names being used consistently across profile query strings.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
276. That convention is simple but fragile if a profile is edited incorrectly.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
277. Import, comment, and string predicates use substring matching on captured text.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
278. That makes them more forgiving than exact identifier predicates.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
279. `hook` and `customhook` accept `.` as wildcard.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
280. Most definition-style predicates also accept `.` as wildcard.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
281. `call` uses substring matching, which is permissive.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
282. That permissiveness may overmatch qualified names or embedded text.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
283. The semantic API does not currently offer case-insensitive or regex-based identifier matching.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
284. The semantic API does not currently offer prefix-only matching.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
285. The profile registry is composed as a `HashMap<&str, LanguageProfile>` keyed by extension or SQL pseudo-key.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
286. Alias extensions are implemented as separate keys that each own another `LanguageProfile`.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
287. C++ alone gets several alias entries in that map.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
288. React gets separate `jsx` and `tsx` entries sharing the same profile shape.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
289. SQL dialects also live as pseudo-language entries in that same map.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
290. `list_language_profiles()` returns `LANGUAGE_PROFILES.values().collect()`.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
291. Returning raw `values()` means alias duplicates remain in the outward-facing list.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
292. `HashMap` iteration order also means that outward-facing list is nondeterministic.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
293. That directly affects MCP `list_languages`.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
294. That also affects any future docs built from `list_language_profiles`.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
295. `describe_language` in the MCP crate linearly scans that same duplicate list.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
296. Duplicate entries can inflate counts and crowd out unique entries in truncated displays.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
297. There is no canonical `LanguageId` separate from aliases today.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
298. The language profile system is functionally rich but presentation-poor.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
299. The profile file-per-language layout is easy to extend.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs
300. The profile registry representation should be split into canonical languages plus aliases before external consumers depend on it more heavily.
   Refs: rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump/src/predicates/mod.rs

### Formatter and Human-readable Output Layer

301. The formatter rereads files instead of consuming `SearchResult` objects directly.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
302. That keeps CLI formatting independent of SDK types but duplicates content-loading policy.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
303. Formatter content loading uses the size limit.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
304. Formatter content loading uses binary detection.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
305. Formatter content loading uses the secret heuristic.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
306. Formatter content loading uses lossy UTF-8 conversion.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
307. Formatter silently skips files it decides not to print, aside from stderr warnings.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
308. A file can match search criteria and still produce no visible CLI content.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
309. That behavior is reasonable for safety, but it needs explicit messaging or metadata.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
310. `print_hunks_format` prints the whole file when the match list is empty.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
311. That makes metadata-only queries useful in CLI mode.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
312. That also increases the stakes of content-safety consistency.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
313. `print_json_format` emits a very simple `{ path, content }` shape.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
314. CLI JSON omits structured match metadata entirely.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
315. MCP provides richer structured outputs than CLI JSON does.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
316. CLI Markdown always uses fenced blocks and does not emit ANSI colors.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
317. Tests explicitly verify that markdown ignores color.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
318. Cat format conditionally highlights content based on color policy and TTY detection.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
319. Syntax highlighting is based on extension-driven syntect grammar selection.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
320. Theme selection is implicit; there is no user-selectable theme option.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
321. `print_find_format` is a useful operator-facing mode with permissions, size, and time.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
322. `print_find_format` uses local time formatting, which is human-friendly but nondeterministic across environments.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
323. Non-Unix permissions are effectively placeholders in `find` output.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
324. `get_contextual_line_ranges` merges overlapping hunk ranges.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
325. That helps readability for dense match clusters.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
326. Hunk printing preserves line endings through `LinesWithEndings`.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
327. MCP snippet construction does not preserve line endings in the same way.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
328. CLI hunk output and MCP snippet output can therefore differ in exact text shape.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
329. `print_output` is a clean single dispatch function over formats.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
330. The formatter file is large enough that splitting by output mode would be reasonable if more modes are added.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
331. The CLI starts printing only after the full search phase completes.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
332. There is no incremental rendering path for path-only or find-only output.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
333. That increases first-result latency on large repos.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
334. The CLI currently optimizes for simplicity over streaming responsiveness.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
335. `FileOutput` is only used for CLI JSON serialization.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
336. There are targeted formatter tests, which is good.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
337. There are no obvious snapshot tests for large multi-file human output.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
338. There are no performance tests around highlighting cost.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
339. Highlighting uses global lazy statics for syntax and theme sets, which is efficient enough.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
340. The formatter exposes a clear seam where CLI and SDK could be brought closer through shared structured outputs.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
341. Output behavior today is coherent inside the CLI itself.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
342. Output behavior today is not coherent across CLI, SDK, and MCP.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
343. If the project keeps both CLI and MCP as first-class channels, output policy should likely live below both.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
344. JSON output being less structured than MCP is an odd inversion.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
345. Many machine consumers will use MCP anyway, but CLI JSON should still be competitive.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
346. There is no CLI `summary` equivalent to MCP `summary`.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
347. There is no CLI `matches` equivalent to MCP `matches`.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
348. There is no MCP equivalent to CLI `markdown`.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
349. There is no shared rendering abstraction between these surfaces.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs
350. Unifying them would reduce duplicated policy and make the tool easier to explain.
   Refs: rdump/src/formatter.rs; rdump/src/lib.rs; rdump-mcp/src/search.rs

### Async API, Runtime Model, and Operational Controls

351. The async API is behind an optional `async` feature, which keeps the default crate lean.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
352. `search_async` uses `spawn_blocking` to bridge the sync iterator into async.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
353. The async channel capacity is fixed at 100.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
354. That provides some backpressure but is not configurable.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
355. Dropping the async stream stops the producer only when the send fails.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
356. There is no explicit cancellation token.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
357. The spawned blocking task handle is discarded.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
358. Join failures after spawn are therefore not surfaced explicitly.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
359. `search_all_async` just collects `search_async` into a `Vec`.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
360. That is ergonomic but not a different execution model.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
361. Async tests cover early termination and concurrent use, which is good.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
362. The async layer still inherits eager raw-result materialization from `search_iter`.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
363. That means the async API is not truly end-to-end streaming.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
364. The async API improves integration ergonomics more than memory behavior.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
365. The core crate has no concurrency limiter for async searches.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
366. Multiple async searches can therefore stack Rayon work with Tokio `spawn_blocking` work.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
367. `rdump-mcp` adds a semaphore on top of the same core engine.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
368. SDK async users and MCP users therefore experience different concurrency policy.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
369. There is no tracing, metrics, or structured timing in the core engine.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
370. There is a `MAX_REGEX_EVAL_DURATION` constant in `limits`.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
371. That constant suggests the team has thought about regex runaway behavior.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
372. There is no comparable timeout policy for tree-sitter query execution.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
373. There is no query budget, file budget, or wall-clock budget in core SDK.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
374. MCP approximates budgets via result and byte limits, not execution-time limits.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
375. There is no first-class statistics object returned by core search functions.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
376. That makes performance reasoning harder for both users and maintainers.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
377. There is no benchmark harness checked into the repo.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
378. The PRD explicitly mentions benchmark gaps.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
379. That means performance claims are still more aspirational than evidenced.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
380. The runtime model is simple and serviceable; the main missing capability is observability.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
381. The repo is mature enough now that observability would likely pay for itself quickly.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
382. Even a minimal timing breakdown for walk, prefilter, full eval, and output would be valuable.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
383. The core engine does not expose candidate counts or prefilter counts.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
384. That makes it hard to know whether a slow query is walk-bound or semantic-bound.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
385. The staged architecture is good enough that such counters would be very meaningful.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
386. Async API users would also benefit from incremental stats and progress.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
387. There is no log level or tracing flag today.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
388. All warnings go straight to stderr.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
389. That is fine for a CLI but awkward for libraries and agent-hosted environments.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs
390. The runtime model currently favors simplicity and portability over introspection and control.
   Refs: rdump/src/async_api.rs; rdump/src/lib.rs; rdump-mcp/src/lib.rs

### MCP Surface and External Contract

391. `rdump-mcp` is correctly designed as an adapter, not a reimplementation.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
392. `RdumpServer` keeps almost no mutable state, which is a good default for MCP tooling.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
393. Search concurrency is bounded by a semaphore whose default is available parallelism.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
394. That is a reasonable guard against too many simultaneous heavy searches.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
395. The search tool delegates to `build_search_request`, then to the SDK iterator, then reshapes results.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
396. MCP defaults output mode to `snippets`, which is a good AI-friendly default.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
397. MCP defaults `skip_errors` to `true`, which is also AI-friendly.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
398. MCP defaults limits to conservative caps, which makes it safer for tool responses.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
399. MCP is therefore safer by default than the raw SDK, even though it inherits the SDK content model.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
400. `normalize_query` requires a non-empty query string.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
401. MCP search therefore does not support the preset-only pattern that the core CLI/SDK support.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
402. That is a contract mismatch between surfaces.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
403. MCP request schema is broader than CLI in some ways because it includes structured limits and output modes.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
404. MCP output modes are `paths`, `matches`, `snippets`, `full`, and `summary`.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
405. CLI output modes are `hunks`, `markdown`, `json`, `paths`, `cat`, and `find`.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
406. Those two vocabularies only partially overlap.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
407. `SearchResponse` stats track returned files, returned matches, returned bytes, and error count.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
408. `SearchResponse` does not report total candidates scanned or total files matched before truncation.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
409. `truncated` and `truncation_reason` are good ergonomic fields.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
410. `estimate_item_bytes` uses approximate payload size, not actual serialized size.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
411. Real JSON payload size can exceed the estimate because of syntax and field names.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
412. `build_snippet` constructs snippets from `result.content.lines()`, which drops original line endings.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
413. `build_snippet` is line-context oriented and easy to understand.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
414. `MatchInfo` truncates text safely at char boundaries.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
415. `Full` output truncates content safely at char boundaries too.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
416. Path-only output still requires the underlying SDK iterator to read content for each result today.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
417. MCP cannot currently avoid that cost because the SDK result type includes content unconditionally.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
418. Errors are returned as strings, not structured typed diagnostics.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
419. That is sufficient for human reading but weak for automated remediation.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
420. `validate_query` only runs the parser.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
421. `validate_query` cannot warn about unsupported language/predicate combinations.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
422. `rql_reference` is hand-written rather than derived from clap docs or parser metadata.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
423. `sdk_reference` is hand-written rather than derived from Rust types or rustdoc.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
424. Those docs are already at risk because code and architecture docs have drifted elsewhere.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
425. `list_resources` only exposes `rdump://docs/rql` and `rdump://docs/sdk`.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
426. There are no MCP resources for language inventory, presets, or search examples.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
427. The server exposes no prompts.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
428. That keeps the MCP surface lean, but also means agent onboarding relies on external docs.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
429. `tool_result` conveniently attaches both text and structured content, which is a solid adapter utility.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs
430. The MCP crate is already useful, but its documentation and schema derivation need hardening before it should be treated as long-term stable.
   Refs: rdump-mcp/src/lib.rs; rdump-mcp/src/search.rs; rdump-mcp/src/languages.rs; rdump-mcp/src/docs.rs; rdump-mcp/src/types.rs

### Test Coverage, Docs Drift, and Process Signals

431. The repo has unusually broad cross-language test coverage for a project of this size.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
432. Dedicated language tests exist for the core semantic predicates across many ecosystems.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
433. There are also targeted tests for async behavior, thread safety, ignore handling, CLI behavior, and library exports.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
434. MCP has real stdio child-process end-to-end tests, which is excellent.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
435. The test suite is a major strategic strength of the project.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
436. The test suite is strongest on semantic correctness, not on operational envelopes.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
437. There are no checked-in microbenchmarks or Criterion benches.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
438. There are no obvious memory-envelope tests for huge repos or huge files beyond size-limit checks.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
439. There are no property-based tests for parser/evaluator invariants.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
440. There are no fuzz tests for malformed queries or malformed source inputs.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
441. There are no doc snapshot tests that compare hand-written references against live types.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
442. The architecture doc still claims `FileContext` uses `OnceCell`, but live code uses `Option`.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
443. The architecture doc shows a richer `SearchResult` than the current public struct.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
444. The architecture doc therefore cannot currently be treated as an exact API reference.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
445. The story artifact for `perform_search_internal` still contains implementation examples that no longer match live code.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
446. The QA gate for that story recorded zero risks, which shows how process artifacts can go stale relative to code evolution.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
447. The README still contains placeholder GitHub badge and release links.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
448. Those placeholders make the project feel less production-ready than the code quality warrants.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
449. The PRD still includes explicit TODOs for benchmarking.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
450. That is honest, but it also confirms that performance claims are not yet backed by stable evidence.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
451. The project has process discipline, but not yet automated doc-discipline.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
452. Some tests rely on `panic!` pattern branches rather than richer assertion context.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
453. That is fine functionally, but improving failure diagnostics would speed maintenance on a large test suite.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
454. Fixtures are plentiful, but there is no obvious canonical manifest of fixture capabilities.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
455. Many per-language tests repeat similar semantic scenarios by hand.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
456. That gives strong coverage today but increases maintenance cost as behaviors evolve.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
457. A generated matrix for common predicate scenarios could reduce repetition substantially.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
458. Hand-written language-specific tests should remain for grammar-specific edge cases.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
459. The source and tests show the team cares about API stability explicitly.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
460. The docs do not yet reflect that same level of explicit stability management.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
461. There is no CI guard for public schema drift in MCP.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
462. There is no CI guard for architecture-doc drift in core.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
463. There is no CI guard for CLI help text vs hand-written RQL docs.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
464. The repo has the raw ingredients for strong release engineering, but not all the automation yet.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
465. The broad test coverage means the next failures are more likely to come from policy mismatch than missing semantic support.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
466. The repo already has enough internal clarity that generated docs are feasible.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
467. The current doc debt is fixable because the core type surfaces are not huge.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
468. The cost of not fixing it will rise as more external consumers adopt MCP or the SDK.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
469. The process artifacts are useful context, but live code must become the generated source of truth.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml
470. A small investment in generated references would eliminate several recurring mismatch classes at once.
   Refs: rdump/tests; rdump-mcp/tests/stdio_e2e.rs; docs/architecture.md; docs/stories/library-api-epic/1.4.create-perform-search-internal.md; docs/qa/gates/1.4-create-perform-search-internal.yml

### Capability, Performance, Architecture, and Operational Improvement Gaps

471. The highest-priority gap is a unified content-safety policy across evaluator, formatter, SDK iterator, and MCP.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
472. Decide explicitly whether secret-like files should be excluded everywhere, exposed everywhere, or exposed only behind opt-in flags.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
473. If safety wins, apply the same secret heuristic in `read_file_content_for_iterator`.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
474. If flexibility wins, expose a typed `content_state` so callers can decide how to handle sensitive or unreadable files.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
475. Parse and validate queries before walking the filesystem to cut wasted work on invalid input.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
476. Deduplicate canonical paths after `safe_canonicalize` so symlink aliases do not cause duplicate work.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
477. Reconcile symlink policy in code, tests, and docs; the current state is internally inconsistent.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
478. Cache compiled tree-sitter queries by language profile and predicate key.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
479. Remove the `content.to_string()` clone in `CodeAwareEvaluator` if possible; it is known overhead.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
480. Consider a truly streaming search pipeline so `search_iter` can produce first results before full completion.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
481. Expose search stats such as candidate count, prefilter count, evaluated count, and timings from the core engine.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
482. Make MCP stats include those same engine counters so agent consumers can reason about cost and truncation.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
483. Introduce a path-only or metadata-only SDK result type to avoid mandatory content reads when callers do not need content.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
484. Align UTF-8 policy across evaluator, formatter, and iterator; today's lossy-vs-strict split is surprising.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
485. Canonicalize the language catalog into stable canonical entries plus aliases before exposing it externally.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
486. Sort language lists deterministically so MCP results are stable and truncation is meaningful.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
487. Allow MCP preset-only searches if that is intended to match core behavior, or document the narrower contract explicitly.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
488. Derive MCP docs from code rather than maintaining `rql_reference` and `sdk_reference` by hand.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
489. Add doc drift tests for `SearchOptions`, `SearchResult`, predicate inventory, and language inventory.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
490. Introduce a benchmark suite covering metadata-only, semantic-only, mixed, and pathological queries.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
491. Add tests specifically for secret-like files across CLI, SDK, and MCP to lock intended behavior.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
492. Add tests for symlink duplicates and canonical-path collapse to make path policy explicit.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
493. Add tests for duplicate or unstable language listings in MCP.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
494. Improve MCP byte-budget enforcement by measuring serialized size or using a safer estimate.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
495. Consider exposing structured diagnostics instead of raw error strings in MCP responses.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
496. Move nested crate-root `limits` into its own module file to reduce root sprawl and clarify ownership.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
497. Split large modules like `formatter.rs` and `code_aware/mod.rs` before the next major feature wave lands.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
498. Clean placeholder metadata in `Cargo.toml` and README before wider adoption, because those rough edges undercut an otherwise strong codebase.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
499. The repo does not need more language breadth first; it needs consistency, observability, and generated truth surfaces.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
500. If the team fixes policy consistency, query caching, path deduplication, and doc generation, rdump will become much stronger for both local operators and AI-agent workflows without needing a major rewrite.
   Refs: rdump/src/evaluator.rs; rdump/src/formatter.rs; rdump/src/lib.rs; rdump/src/commands/search.rs; rdump/src/predicates/code_aware/mod.rs; rdump/src/predicates/code_aware/profiles/mod.rs; rdump-mcp/src/search.rs; rdump-mcp/src/docs.rs; docs/architecture.md
