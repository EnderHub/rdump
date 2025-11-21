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

### 8. Library API

**Problem:** rdump can only be used as a CLI tool. Cannot embed search functionality in other Rust programs.

**Current limitations:**
- `run_search()` prints to stdout, doesn't return results
- No way to get `Vec<SearchResult>` programmatically
- Args struct is CLI-coupled (color choice, output path, etc.)

**Solution:** Separate query execution from output formatting.

```rust
// Proposed library API
use rdump::{search, SearchOptions, SearchResult};

let results: Vec<SearchResult> = search(
    "ext:rs & func:main",
    Path::new("."),
    SearchOptions::default(),
)?;

for result in results {
    println!("{}: {} matches", result.path.display(), result.matches.len());
    // Access result.content, result.ranges, etc.
}
```

**Implementation:**
1. Create `SearchResult` struct with path, matches, content
2. Refactor `run_search` to call `search()` then format output
3. Export clean public API without CLI concerns
4. Consider streaming iterator for large result sets

**Use cases:**
- Embed in IDE plugins
- Build custom tooling on top of rdump
- Integration with CI/CD pipelines
- Use in test harnesses

---

## Low Priority

### 9. Interactive Mode (TUI)

**Problem:** Iterating on queries requires re-running commands.

**Solution:** Interactive terminal UI with live results.

```bash
rdump --interactive
```

---

### 10. Language Server Protocol (LSP) Integration

**Problem:** IDE users can't easily use rdump queries.

**Solution:** LSP server that provides rdump as a code action.

---

### 11. Custom Predicate Plugins

**Problem:** Users need domain-specific predicates not built into rdump.

**Solution:** Plugin system for custom predicates.

```bash
rdump --plugin=./my_predicate.wasm "ext:rs & my_check:foo"
```

---

### 12. Diff Mode

**Problem:** Can't easily see what changed between two query runs.

**Solution:** Compare results between git refs or timestamps.

```bash
rdump "ext:rs & func:." --diff=HEAD~5
rdump "ext:rs & func:." --since="2024-01-01"
```

---

### 13. Export Formats

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

## Version Roadmap

### v0.2.0 - Enhanced Usability

- Interactive mode with REPL, history, tab completion (see #9)
- Query builder wizard for guided construction
- Result caching with mtime invalidation (see #3)
- Extended output formats: XML, CSV, custom templates (see #13)

### v0.3.0 - Language Expansion

- New languages: C/C++, Ruby, PHP, Swift, Kotlin
- New predicates: `docstring:`, `decorator:`, `literal:`, `operator:`
- Cross-language analysis with `polyglot:` predicate
- Improved language detection

### v0.4.0 - Analysis Features

- Dependency analysis: `depends:`, `dependents:`
- Code metrics: `complexity:>10`, `lines:>100`, `depth:>5`
- Pattern detection: `pattern:singleton`, `antipattern:godclass`

### v1.0.0 - Production Ready

- Stable CLI and query language (semantic versioning)
- Enterprise features: config profiles, audit logging
- Performance targets: 100K files < 5s, memory < 500MB, startup < 100ms

### Long-Term Vision

- IDE integration (VS Code, JetBrains, Neovim, Emacs)
- Cloud/remote: GitHub/GitLab search, distributed repos
- AI integration: natural language queries, query suggestions

### Not Planned

- File modification (read-only by design)
- Real-time watching (use watchman/fswatch + rdump)
- GUI (CLI-first, IDE plugins instead)
- Network search (local filesystem only)

---

## Contributing

To propose a new feature:
1. Add it to this document with problem statement and proposed solution
2. Open a PR for discussion

Priority is based on:
- User demand
- Impact on core use cases

- Add Kotlin semantic support once a tree-sitter-kotlin crate compatible with tree-sitter 0.25.x exists (or vendor a 0.25-compatible binding). Current crate 0.3.8 depends on tree-sitter 0.22.x and conflicts with our 0.25.8 stack; investigate when a newer release appears or consider vendoring a generated binding. Marked as potentially unmaintained; re-evaluate availability before implementation.
