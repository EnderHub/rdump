# rdump Feature Proposals

This document tracks proposed features and enhancements for future rdump versions.

---

## High Priority

### 1. Cross-File Relationship Queries

**Problem:** Users often need to find files based on relationships to other files (e.g., "find source files without corresponding tests"). Currently requires inefficient shell loops.

**Use Cases:**
- Find implementation files without tests
- Find modules without documentation
- Find components without stories/snapshots
- Find handlers without corresponding routes

**Proposed Syntax Options:**

```bash
# Option A: --missing flag
rdump "path:src/ & ext:rs & !name:*test*" --missing="name:*{stem}*test*"

# Option B: has_related predicate
rdump "path:src/ & ext:rs & !name:*test* & !has_related:tests/*{stem}*"

# Option C: Dedicated coverage predicate
rdump "path:src/ & ext:rs & !has_test"
```

**Implementation Considerations:**
- Need to define `{stem}` or `{name}` template variables
- Should support custom patterns for different project conventions
- Could cache the "related files" set for efficiency

---

### 2. Automatic Query Optimization

**Problem:** Query performance depends on predicate order, but users write queries in logical order, not optimal order.

**Solution:** Reorder predicates by cost tier before evaluation.

**Details:** See `docs/performance-optimizations.md` section 4.

---

### 3. Query Result Caching

**Problem:** Repeated queries on unchanged files redo all work.

**Solution:** Cache parse trees and match results with file mtime invalidation.

```bash
rdump --cache "ext:rs & func:main"
```

---

## Medium Priority

### 4. Set Operations Between Queries

**Problem:** Can't express "files matching A but not B" without shell tools.

**Proposed Syntax:**

```bash
# Difference
rdump "ext:rs & func:main" --minus "path:test"

# Intersection
rdump "ext:rs & func:main" --intersect "modified:<1d"

# Or using subqueries
rdump "(ext:rs & func:main) - (path:test)"
```

---

### 5. Aggregation and Statistics

**Problem:** Limited ability to analyze codebase metrics.

**Proposed Features:**

```bash
# Count by extension
rdump "func:." --group-by=ext

# Size distribution
rdump "ext:rs" --stats

# Output:
# Files: 234
# Total size: 1.2MB
# Avg size: 5.1KB
# Functions: 1,847
# Structs: 423
```

---

### 6. Watch Mode

**Problem:** Need to re-run queries manually after file changes.

**Solution:** Watch for filesystem changes and re-run query.

```bash
rdump --watch "ext:rs & func:test_" --format=count
```

---

### 7. Configuration File

**Problem:** Complex queries and common options require repetition.

**Solution:** `.rdumprc` or `rdump.toml` config file.

```toml
[defaults]
format = "markdown"
context = 3

[aliases]
rust-funcs = "ext:rs & func:."
recent = "modified:<1d"
no-tests = "!path:test & !name:*test*"

[profiles.rust]
function_query = "(function_item name: (identifier) @name)"
```

---

## Low Priority

### 8. Interactive Mode (TUI)

**Problem:** Iterating on queries requires re-running commands.

**Solution:** Interactive terminal UI with live results.

```bash
rdump --interactive
```

---

### 9. Language Server Protocol (LSP) Integration

**Problem:** IDE users can't easily use rdump queries.

**Solution:** LSP server that provides rdump as a code action.

---

### 10. Custom Predicate Plugins

**Problem:** Users need domain-specific predicates not built into rdump.

**Solution:** Plugin system for custom predicates.

```bash
rdump --plugin=./my_predicate.wasm "ext:rs & my_check:foo"
```

---

### 11. Diff Mode

**Problem:** Can't easily see what changed between two query runs.

**Solution:** Compare results between git refs or timestamps.

```bash
rdump "ext:rs & func:." --diff=HEAD~5
rdump "ext:rs & func:." --since="2024-01-01"
```

---

### 12. Export Formats

**Problem:** Limited integration with other tools.

**Additional Formats:**
- CSV for spreadsheets
- HTML for reports
- SARIF for security tools
- Graphviz DOT for dependency visualization

---

## Completed

- [x] Basic RQL query language
- [x] Tree-sitter integration for semantic search
- [x] Multiple output formats (markdown, json, paths, hunks, cat, find)
- [x] Parallel file processing
- [x] Gitignore support
- [x] Context lines for hunks

---

## Contributing

To propose a new feature:
1. Add it to this document with problem statement and proposed solution
2. Open a PR for discussion

Priority is based on:
- User demand
- Impact on core use cases
