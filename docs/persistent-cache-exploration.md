# Persistent Cache Exploration

This document records the current second-wave exploration outcome for persistent query and index caching.

## Candidate cache layers

- Query-plan cache keyed by normalized query plus preset set.
- Language-profile and tree-sitter query cache keyed by predicate and profile.
- Optional file-discovery/index cache keyed by root, ignore inputs, and mtimes.

## Recommended scope

- Keep the existing in-process tree-sitter query cache as the default fast path.
- Treat persistent filesystem indexing as optional and opt-in.
- Version any persistent cache by schema version plus rdump crate version to avoid stale reuse across breaking changes.

## Risks

- Root-local caches can become stale when ignore files or symlink layouts change.
- Cross-platform path canonicalization can make cache reuse unsafe.
- Secret-handling policy must be consistent before any persistent content-derived cache is introduced.
