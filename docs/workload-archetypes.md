# Workload Archetypes

`rdump` is tuned against three recurring workloads.

## Local Operator Grep Replacement

- Shape: quick metadata/content filters from a terminal
- Preferred outputs: `summary`, `matches`, `hunks`, `find`
- Key metrics: first-result latency, stable ordering, low startup cost

## AI Agent Snippet Lookup

- Shape: repeated narrow searches over the same checkout
- Preferred outputs: CLI `json`, MCP `search output=snippets|summary`
- Key metrics: continuation support, schema stability, explicit truncation metadata, safety placeholders

## Large Monorepo Semantic Search

- Shape: language-aware queries on wide trees with mixed file types
- Preferred outputs: `summary` first, then `snippets` or `full`
- Key metrics: candidate pruning rate, semantic parse-failure counts, query cache metrics, time-budget behavior

## Benchmark coverage

- `rdump/tests/perf_smoke.rs` covers metadata-only, path-only, semantic-only, mixed, and synthetic large-repo shapes.
- `rdump/tests/highlight_perf.rs` covers cold vs warm formatter highlighting cost.
