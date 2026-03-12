# rdump Second-Wave P0/P1 Improvements Checklist

Source: [rdump-500-observations-assessment.md](./rdump-500-observations-assessment.md)

This is a second, additional set of `P0` and `P1` improvements derived from the assessment.
It is intentionally non-overlapping with the first checklist in
[rdump-p0-p1-improvements-checklist.md](./rdump-p0-p1-improvements-checklist.md).

## P0

- [x] Define and publish separate stability and semver expectations for the CLI, SDK, and MCP surfaces so external consumers know which contracts are meant to stay fixed. `(Assessment: obs. 18, 42-43, 459-469)`
- [x] Add an explicit schema or contract version field to MCP tool responses and documentation resources so hosts can detect breaking payload changes safely. `(Assessment: obs. 42-43, 430, 468-469)`
- [x] Replace string-only MCP error reporting with typed error codes and machine-usable remediation fields so agent hosts can react programmatically. `(Assessment: obs. 418-419, 495)`
- [x] Unify partial-failure handling across CLI, SDK, and MCP so skipped-file behavior and first-error behavior are explicit policy choices, not surface-specific quirks. `(Assessment: obs. 99-100, 221-225)`
- [x] Add explicit CLI error-handling modes such as `--skip-errors` and `--fail-fast` so the CLI can match the resilience controls already exposed to SDK and MCP users. `(Assessment: obs. 223-225, 397)`
- [x] Centralize `SearchArgs` to `SearchOptions` translation in one shared helper so new search options cannot drift between CLI code paths. `(Assessment: obs. 84-86)`
- [x] Freeze, deprecate, or move accidental public APIs like crate-root clap types, `commands`, and `perform_search` behind an intentional compatibility plan before they become permanent obligations. `(Assessment: obs. 71-76, 459-460)`
- [x] Add release gating for public export changes so semver-impacting SDK or CLI surface shifts are reviewed intentionally rather than discovered after release. `(Assessment: obs. 71-76, 459-460)`
- [x] Surface `spawn_blocking` panics or join failures from async search back to callers instead of discarding the handle and losing that failure mode. `(Assessment: obs. 357-358)`
- [x] Promote execution-budget controls from env-only tuning into stable CLI, SDK, and MCP request options so time limits are part of the supported contract. `(Assessment: obs. 372-374, 390)`
- [x] Preserve original line endings in MCP snippets, or expose a raw-vs-normalized mode explicitly, so machine consumers do not get silent text-shape changes. `(Assessment: obs. 327-328, 412-413)`
- [x] Add CI guards for CLI help text versus README and RQL reference docs so the query-writing surface cannot drift silently across user entry points. `(Assessment: obs. 77-79, 148-150, 463)`

## P1

- [x] Introduce a shared schema or contracts crate for CLI, SDK, and MCP payload types so cross-surface drift becomes structurally harder. `(Assessment: obs. 13-14, 42-43)`
- [x] Add a dedicated `rdump-mcp/README.md` that explains purpose, tools, resources, limits, and host expectations instead of relying on the main repo README alone. `(Assessment: obs. 22-25, 430)`
- [x] Write an MCP stdio deployment and host-integration guide covering process model, concurrency, limits, and safe defaults for integrators. `(Assessment: obs. 25, 391-430)`
- [x] Add an operator-focused runtime and troubleshooting guide for rdump itself so budgets, warnings, concurrency, and failure modes are documented in one place. `(Assessment: obs. 25, 380-390)`
- [x] Add CLI commands to show config path and merged config state so users can debug preset resolution and local-vs-global overrides without reading files manually. `(Assessment: obs. 89)`
- [x] Add a preset expansion or “effective query” CLI command so users can inspect how saved presets compose before executing a search. `(Assessment: obs. 88-90, 111-113)`
- [x] Add a first-class CLI `explain` subcommand backed by planner output so local users get the same query introspection power already moving into the programmatic surfaces. `(Assessment: obs. 90, 133-146)`
- [x] Expose normalized or effective query strings as first-class SDK and CLI outputs so tooling can reason about what will actually run after preset expansion. `(Assessment: obs. 111-113, 137)`
- [x] Add a public AST pretty-printer or normalizer API so advanced consumers can round-trip, inspect, or lint queries outside of execution. `(Assessment: obs. 137-138, 145-146)`
- [x] Shift more predicate-value validation into parse or plan time, especially for `size:` and `modified:`, so obviously malformed values fail earlier and more consistently. `(Assessment: obs. 127-130)`
- [x] Add query lints for contradictory or impossible combinations, such as mutually exclusive file-type filters, so users get guidance rather than silent zero-result runs. `(Assessment: obs. 139-144)`
- [x] Document the staged evaluator rule that missing predicates are treated as neutral in partial registries because that behavior is subtle and important for maintainers. `(Assessment: obs. 120-123)`
- [x] Generate CLI predicate/help reference material from the live registry metadata rather than maintaining separate prose descriptions by hand. `(Assessment: obs. 77-79, 148-150)`
- [x] Expose shared structured output types in the SDK for `summary`, `matches`, `snippets`, `full`, and `paths` so adapters do not each invent their own result-shaping model. `(Assessment: obs. 80-82, 340-350)`
- [x] Add an SDK result-shaping selector so callers can ask for path-only, summary, or snippet-like views directly instead of always materializing full `SearchResult` objects first. `(Assessment: obs. 80-82, 227-230, 340-350)`
- [x] Add incremental CLI rendering for `paths`, `find`, and `summary`-style outputs so first-result latency improves on large repos even before the whole search completes. `(Assessment: obs. 331-334)`
- [x] Add golden or snapshot tests for large human-readable CLI outputs so formatting regressions become easier to spot than with ad hoc string assertions. `(Assessment: obs. 336-337, 341-343)`
- [x] Add highlighting performance benchmarks and warm-vs-cold tests so syntax coloring cost is measured rather than assumed. `(Assessment: obs. 338-339)`
- [x] Add user-selectable syntax themes or a documented no-highlight profile so operators can tune readability and output behavior intentionally. `(Assessment: obs. 319-320)`
- [x] Add deterministic `find` output modes with UTC or ISO timestamps and portable metadata fields for scripting and agent use. `(Assessment: obs. 321-323)`
- [x] Add public progress callbacks or events for long-running searches so hosts can display progress without scraping stderr or waiting for final completion. `(Assessment: obs. 380-386)`
- [x] Add incremental progress and stat updates to the async API so async consumers can observe long searches instead of only receiving final items. `(Assessment: obs. 362-386)`
- [x] Introduce tree-sitter-specific execution budgets or per-file match caps separate from regex limits so semantic runaway cases have targeted controls. `(Assessment: obs. 370-374)`
- [x] Optimize SQL dialect detection to avoid allocating a full lowercased clone of file content for every `.sql` file. `(Assessment: obs. 259-262)`
- [x] Add an explicit language override for extensionless or ambiguous files so semantic queries can still be forced on known content types. `(Assessment: obs. 254-257)`
- [x] Add case-insensitive, prefix, or regex-style matching modes for semantic identifiers so users are not limited to exact-match or wildcard-only behavior. `(Assessment: obs. 281-284)`
- [x] Add adaptive or parallel metadata prefiltering for very large candidate sets so the cheap pass does not become a sequential bottleneck. `(Assessment: obs. 153-176)`
- [x] Add planner-driven predicate reordering or cost optimization beyond the current fixed staged passes so expensive query mixes can be reduced further. `(Assessment: obs. 131-135)`
- [x] Explore an optional persistent query or index cache for repeated searches in large repositories where fresh full scans are the dominant cost. `(Assessment: obs. 32-34)`
- [x] Add synthetic large-repo and memory-envelope tests so the engine’s operational ceiling is measured, not inferred from small fixtures. `(Assessment: obs. 437-438)`
- [x] Add benchmark regression thresholds to CI or release review so performance gains and regressions are visible over time, not just one-off bench results. `(Assessment: obs. 449-450, 464)`
- [x] Add a fixture manifest or capability map that documents what each fixture is intended to validate so test assets stay maintainable as the suite grows. `(Assessment: obs. 47-48, 454)`
- [x] Improve common test helpers and assertion messages so failures report intent and expected capability more clearly than raw `panic!` branches do today. `(Assessment: obs. 452-453)`
- [x] Add MCP prompts for onboarding and common workflows so agent users can discover safe, idiomatic rdump usage directly from the server surface. `(Assessment: obs. 427-428)`
- [x] Add MCP resources for presets and active config state so agents can inspect local search context without only relying on tool calls. `(Assessment: obs. 425-428)`
- [x] Enrich `server_info` and external docs with capability metadata such as supported outputs, limit defaults, and stability tier so hosts can integrate more defensibly. `(Assessment: obs. 392-430)`
- [x] Add host-integrator telemetry hooks or callbacks below the CLI layer so embedding environments can route events, timings, and warnings into their own observability systems. `(Assessment: obs. 380-390, 429)`
- [x] Extract README examples into executable examples or add README example sync checks so the main documentation stays runnable as the tool evolves. `(Assessment: obs. 24, 47-48, 463)`
