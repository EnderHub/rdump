# rdump P0/P1 Improvements Checklist

Source: [rdump-500-observations-assessment.md](./rdump-500-observations-assessment.md)

This checklist is derived from the assessment and limited to `P0` and `P1` work only.

## P0

- [x] Unify content-read policy across evaluator, formatter, SDK iterator, and MCP so the same file is handled the same way everywhere. `(Assessment: obs. 201-215, 471-474)`
- [x] Make secret-like file handling explicit and consistent instead of silently exposing content in one surface and suppressing it in another. `(Assessment: obs. 197-215, 471-474)`
- [x] Align UTF-8 handling across surfaces so invalid UTF-8 is either always lossy or always an error, not both. `(Assessment: obs. 193-206, 484)`
- [x] Parse and validate queries before walking the filesystem so invalid input fails fast. `(Assessment: obs. 114-116, 151-152, 475)`
- [x] Deduplicate canonicalized paths after symlink resolution so the same file cannot be processed multiple times through aliases. `(Assessment: obs. 157-164, 476)`
- [x] Reconcile symlink-follow behavior in code, tests, and docs and make the policy intentional. `(Assessment: obs. 158-164, 477)`
- [x] Cache compiled tree-sitter queries by language profile and predicate key instead of recompiling them per file. `(Assessment: obs. 269-274, 478)`
- [x] Remove the full-content clone in `CodeAwareEvaluator` or replace it with a cached borrowed path so semantic searches stop paying that allocation repeatedly. `(Assessment: obs. 260, 273-275, 479)`
- [x] Add a true path-only or metadata-only SDK path so callers do not have to read full file content just to list matches. `(Assessment: obs. 227-230, 416-417, 483)`
- [x] Make `search_iter` genuinely streaming end to end, or document it more honestly as lazy content loading over eager search results. `(Assessment: obs. 55-56, 216-220, 362-364, 480)`
- [x] Expose engine stats like candidate count, prefilter count, evaluated count, and match count from core search. `(Assessment: obs. 383-385, 407-408, 481-482)`
- [x] Replace stderr-only warnings with structured diagnostics that SDK and MCP consumers can actually reason about. `(Assessment: obs. 268, 388-389, 418-419, 495)`
- [x] Canonicalize the language inventory into unique languages plus aliases so MCP does not build user-facing lists from alias keys. `(Assessment: obs. 285-299, 485)`
- [x] Sort language inventory deterministically before exposing it through MCP. `(Assessment: obs. 292-296, 486)`
- [x] Generate MCP `rql_reference` and `sdk_reference` from live code instead of maintaining them by hand. `(Assessment: obs. 422-424, 441-469, 488)`
- [x] Add tests that lock secret-file behavior across CLI, SDK, and MCP so the policy cannot drift again. `(Assessment: obs. 206-215, 491)`
- [x] Add tests that lock symlink, canonicalization, and duplicate-path behavior explicitly. `(Assessment: obs. 158-164, 492)`
- [x] Add tests that fail on duplicate or unstable MCP language listings. `(Assessment: obs. 291-296, 493)`
- [x] Align MCP preset-only query behavior with core search or explicitly narrow the contract and document it. `(Assessment: obs. 83, 400-402, 487)`
- [x] Add doc-drift checks for architecture, SDK surface, predicate inventory, and language inventory against live code. `(Assessment: obs. 442-469, 489)`

## P1

- [x] Make MCP byte-budget enforcement use serialized-size measurement or a safer approximation than the current rough estimator. `(Assessment: obs. 410-411, 494)`
- [x] Introduce a typed `content_state` or equivalent in `SearchResult` so callers know whether content was loaded, skipped, or sanitized. `(Assessment: obs. 235-238)`
- [x] Preserve and expose the reason content was skipped, such as size, binary detection, secret heuristic, or decode failure. `(Assessment: obs. 195-199, 235-238)`
- [x] Move the nested `limits` module out of `lib.rs` into its own file to reduce crate-root sprawl. `(Assessment: obs. 496)`
- [x] Split `formatter.rs` by output mode before more formats are added. `(Assessment: obs. 330, 497)`
- [x] Split `code_aware/mod.rs` into profile selection, query execution, and cache/policy pieces to reduce hotspot complexity. `(Assessment: obs. 252-253, 298-300, 497)`
- [x] Add a benchmark suite for metadata-only, semantic-only, mixed, and worst-case queries. `(Assessment: obs. 377-379, 437-440, 490)`
- [x] Record timing breakdowns for walk, prefilter, full evaluation, and output shaping. `(Assessment: obs. 369, 375-385)`
- [x] Add a query explain/planner mode that shows how a query will be evaluated and where the expensive work is. `(Assessment: obs. 133-145)`
- [x] Extend validation so it can warn about unsupported language or predicate combinations, not just syntax errors. `(Assessment: obs. 142-145, 420-421)`
- [x] Surface candidate-walk and prefilter discard rates so slow-query analysis becomes practical. `(Assessment: obs. 176, 383-385)`
- [x] Make async channel capacity configurable instead of fixed at 100. `(Assessment: obs. 353-354)`
- [x] Add cooperative cancellation for async and MCP search tasks instead of relying only on broken-channel behavior. `(Assessment: obs. 355-358)`
- [x] Unify concurrency policy between async SDK and MCP so they do not apply different backpressure rules over the same engine. `(Assessment: obs. 365-368)`
- [x] Add execution budgets or timeouts for long-running semantic searches, not just result-size truncation. `(Assessment: obs. 372-374)`
- [x] Introduce a logging or tracing layer so warnings and timings can be routed by host environment instead of always going to stderr. `(Assessment: obs. 268, 387-389)`
- [x] Generate the outward-facing language catalog from canonical profiles plus alias metadata instead of directly from the alias map. `(Assessment: obs. 285-299)`
- [x] Add a stable `LanguageId` model separate from extensions so future docs and APIs stop treating aliases as primaries. `(Assessment: obs. 297-298)`
- [x] Add shebang-based or content-based profile detection for extensionless files where that materially helps. `(Assessment: obs. 255-257)`
- [x] Document snapshot-consistency semantics, because files can change between evaluation and result materialization today. `(Assessment: obs. 183-184, 242-244)`
- [x] Stop creating a temporary ignore file on every search and replace it with a cached or default ignore strategy. `(Assessment: obs. 165-166, 491)`
- [x] Revisit ignore-layer implementation so default ignores, global ignores, and local ignores are cheaper to compose per request. `(Assessment: obs. 167-170, 491)`
- [x] Add a no-content CLI JSON mode or richer structured JSON so machine consumers are not forced into whole-file dumps. `(Assessment: obs. 313-315, 344-345)`
- [x] Bring CLI JSON closer to MCP structured output so the two machine-facing surfaces are not solving the same problem differently. `(Assessment: obs. 313-317, 403-406)`
- [x] Consider adding CLI `summary`, `matches`, and `snippets` output modes or a shared output layer that can serve both CLI and MCP. `(Assessment: obs. 346-348, 403-406)`
- [x] Add MCP resources for languages, examples, and maybe presets so agents can self-orient without only using tool calls. `(Assessment: obs. 425-428)`
- [x] Add MCP schema and contract stability checks in CI so API drift is caught before release. `(Assessment: obs. 461-463)`
- [x] Add property-based and fuzz testing for parser edge cases and malformed source inputs. `(Assessment: obs. 439-440)`
- [x] Reduce repeated language tests by generating a shared behavior matrix where the semantics are meant to be identical. `(Assessment: obs. 455-458)`
- [x] Clean package metadata and README placeholders so the repo's public surface matches the actual implementation quality. `(Assessment: obs. 20-21, 447-448, 498)`
