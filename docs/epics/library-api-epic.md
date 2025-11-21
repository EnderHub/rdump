# Library API - Brownfield Enhancement

## Epic Goal

Enable programmatic access to rdump's search functionality by exposing a streaming-first library API, allowing developers to embed rdump in IDE plugins, custom tooling, CI/CD pipelines, and test harnesses without modifying the existing CLI interface.

---

## Problem Statement

### Current Limitations

rdump is currently limited to CLI usage only. Users cannot embed search functionality in other Rust programs. This prevents:

- **IDE plugin development** - Cannot build VSCode/IntelliJ extensions that use rdump queries
- **Custom tooling** - Cannot create specialized search tools built on rdump's query language
- **CI/CD pipeline integration** - Cannot programmatically search code in build pipelines
- **Test harness usage** - Cannot use rdump in integration tests
- **Programmatic access** - No way to get structured results for further processing

### Technical Blockers

| Blocker | Current State | Impact |
|---------|---------------|--------|
| `run_search()` prints to stdout | No return value | Cannot capture results programmatically |
| No structured return type | `Vec<(PathBuf, Vec<Range>)>` | `Range` is tree-sitter internal type |
| `SearchArgs` is CLI-coupled | Contains `color`, `format`, `output` | Library doesn't need display options |
| No content access | Only byte ranges returned | Must re-read files to get matched text |

### Target Users

1. **Tool developers** building IDE plugins or code analysis tools
2. **DevOps engineers** integrating code search into CI/CD
3. **Researchers** analyzing codebases programmatically
4. **Teams** building custom developer tooling on rdump

---

## Existing System Context

### Current Architecture

```text
SearchArgs (CLI struct)
    ↓
run_search() [commands/search.rs:21-73]
    ├─ Handle shorthand flags
    ├─ perform_search() → Vec<(PathBuf, Vec<Range>)>
    │   ├─ Load config/presets
    │   ├─ get_candidate_files() → directory walk
    │   ├─ Parse query → AST
    │   ├─ Pre-filter pass (metadata predicates only)
    │   └─ Main evaluation pass (content + semantic)
    └─ formatter::print_output() → stdout/file
```

### Key Components

| Component | Location | Purpose | Library Relevance |
|-----------|----------|---------|-------------------|
| `SearchArgs` | lib.rs:196-277 | CLI argument struct | Keep for CLI, create `SearchOptions` for library |
| `run_search()` | commands/search.rs:21-73 | Orchestrates search and output | Refactor to use internal function |
| `perform_search()` | commands/search.rs:77-221 | Core search logic | Extract to `perform_search_internal()` |
| `Evaluator` | evaluator.rs:127-138 | Query evaluation engine | Unchanged - core logic |
| `MatchResult` | evaluator.rs:11-18 | Evaluation result (Boolean/Hunks) | Map to `SearchResult` |
| `FileContext` | evaluator.rs:21-34 | Per-file evaluation context | Unchanged - internal |
| `print_output()` | formatter.rs:237-264 | Output formatting dispatcher | CLI only - not exposed |

### Current SearchArgs Structure

**Location:** `src/lib.rs:196-277`

```rust
pub struct SearchArgs {
    // === Core search parameters (needed for library) ===
    pub query: Option<String>,
    pub dialect: Option<SqlDialectFlag>,
    pub preset: Vec<String>,
    pub root: PathBuf,
    pub no_ignore: bool,
    pub hidden: bool,
    pub max_depth: Option<usize>,

    // === CLI-only parameters (NOT needed for library) ===
    pub output: Option<PathBuf>,      // Write to file
    pub line_numbers: bool,           // Display option
    pub no_headers: bool,             // Display option
    pub format: Format,               // Output format
    pub color: ColorChoice,           // ANSI colors
    pub context: Option<usize>,       // Context lines
    pub find: bool,                   // Format alias
}
```

### Current Return Type Problem

`perform_search()` returns `Vec<(PathBuf, Vec<Range>)>`:
- `PathBuf` - File path
- `Vec<Range>` - tree-sitter ranges (byte offsets + row/column positions)
  - Empty vector = boolean match (whole file)
  - Non-empty = specific matching code blocks ("hunks")

**Why this is insufficient:**

```rust
// Current: Library user gets this
let results: Vec<(PathBuf, Vec<Range>)> = perform_search(&args)?;

// Problem: To get actual text, user must:
for (path, ranges) in results {
    let content = std::fs::read_to_string(&path)?;  // Re-read entire file
    for range in ranges {
        // Deal with tree-sitter internals
        let text = &content[range.start_byte..range.end_byte];
        let line = range.start_point.row + 1;  // 0-indexed to 1-indexed
        // ... manual conversion
    }
}
```

---

## Proposed Solution

### Architecture After Implementation

```text
┌─────────────────────────────────────────────────────────┐
│ CLI Path (unchanged external behavior)                  │
│                                                         │
│ rdump search "ext:rs"                                   │
│     ↓                                                   │
│ SearchArgs (unchanged CLI struct)                       │
│     ↓                                                   │
│ run_search()                                            │
│     ├─ Convert to SearchOptions                         │
│     ├─ Call perform_search_internal()                   │
│     └─ formatter::print_output() → stdout               │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ Library Path (new)                                      │
│                                                         │
│ search_iter("ext:rs", SearchOptions::default())         │
│     ↓                                                   │
│ SearchOptions (library struct, no CLI concerns)         │
│     ↓                                                   │
│ perform_search_internal()                               │
│     ↓                                                   │
│ Iterator<SearchResult> → user's code                    │
└─────────────────────────────────────────────────────────┘
```

### Why Streaming-First Design

1. **Memory efficiency** - Only one file's content in memory at a time
2. **Early termination** - Stop after finding N results without processing all files
3. **Pipeline-friendly** - Compose with iterator adapters (filter, map, take)
4. **Large repo support** - Essential for monorepos with 10K+ files

**Memory comparison:**

| Approach | 1000 files @ 10KB each | Memory Usage |
|----------|------------------------|--------------|
| Collect all | Load all content | ~10 MB |
| Streaming | One file at a time | ~10 KB |

---

## Stories

---

### Story 1: Create SearchOptions Struct

**Estimated: 30 minutes | Dependencies: None**

#### Goal
Create a `SearchOptions` struct that contains only search-relevant parameters, excluding CLI concerns like output formatting and colors.

#### Location
`src/lib.rs`

#### Implementation

```rust
/// Options for performing a search (library-friendly)
///
/// This struct contains only the parameters needed for search logic,
/// excluding CLI-specific concerns like output formatting and colors.
///
/// # Example
///
/// ```rust
/// use rdump::SearchOptions;
/// use std::path::PathBuf;
///
/// let options = SearchOptions {
///     root: PathBuf::from("/path/to/project"),
///     presets: vec!["rust".to_string()],
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Root directory to search from
    pub root: PathBuf,

    /// Named presets to apply (e.g., "rust", "python")
    pub presets: Vec<String>,

    /// If true, ignore .gitignore rules
    pub no_ignore: bool,

    /// If true, include hidden files and directories
    pub hidden: bool,

    /// Maximum directory depth to search
    pub max_depth: Option<usize>,

    /// SQL dialect override for .sql files
    pub sql_dialect: Option<SqlDialect>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            presets: vec![],
            no_ignore: false,
            hidden: false,
            max_depth: None,
            sql_dialect: None,
        }
    }
}
```

#### Re-export SqlDialect

```rust
// In lib.rs - re-export for library users
pub use predicates::code_aware::SqlDialect;
```

#### Acceptance Criteria

**Type Requirements:**
- [ ] Struct derives `Debug`, `Clone`
- [ ] Struct is `Send + Sync` (no interior mutability)
- [ ] All fields use owned types (no lifetimes)

**Functionality:**
- [ ] `SearchOptions::default()` returns:
  - `root`: current directory (".")
  - `presets`: empty vec
  - `no_ignore`: false (respect .gitignore)
  - `hidden`: false (skip hidden files)
  - `max_depth`: None (unlimited)
  - `sql_dialect`: None (auto-detect)

**Documentation:**
- [ ] Struct has module-level rustdoc with example
- [ ] Each field has `///` documentation
- [ ] `SqlDialect` is re-exported from lib.rs

#### Technical Notes

- The struct mirrors the search-relevant fields from `SearchArgs` but excludes:
  - `output: Option<PathBuf>` - CLI output destination
  - `line_numbers: bool` - display option
  - `no_headers: bool` - display option
  - `format: Format` - output format
  - `color: ColorChoice` - ANSI colors
  - `context: Option<usize>` - context lines for display
  - `find: bool` - format alias

---

### Story 2: Create SearchResult Struct

**Estimated: 30 minutes | Dependencies: Story 1**

#### Goal
Create the `SearchResult` struct with helper methods for working with matches. This represents a file that matched the search query.

#### Location
`src/lib.rs`

#### Implementation

```rust
/// A file that matched the search query
///
/// Contains the file path, all matches within the file, and the file content.
/// For whole-file matches (boolean predicates like `ext:rs`), the `matches`
/// vector will be empty.
///
/// # Example
///
/// ```rust
/// let result: SearchResult = /* from search */;
///
/// if result.is_whole_file_match() {
///     println!("Whole file matched: {}", result.path.display());
/// } else {
///     for m in &result.matches {
///         println!("Lines {}-{}: {}", m.start_line, m.end_line, m.text);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Path to the matched file
    pub path: PathBuf,

    /// Matches within this file (empty for whole-file matches)
    pub matches: Vec<Match>,

    /// Full file content
    pub content: String,
}

impl SearchResult {
    /// Returns true if this is a whole-file match (no specific hunks)
    ///
    /// Whole-file matches occur with boolean predicates like `ext:rs` or
    /// `lang:python` that match the entire file rather than specific code blocks.
    pub fn is_whole_file_match(&self) -> bool {
        self.matches.is_empty()
    }

    /// Get all matched line numbers (1-indexed)
    ///
    /// Returns a sorted, deduplicated list of all line numbers that contain matches.
    pub fn matched_lines(&self) -> Vec<usize> {
        let mut lines: Vec<usize> = self.matches
            .iter()
            .flat_map(|m| m.start_line..=m.end_line)
            .collect();
        lines.sort_unstable();
        lines.dedup();
        lines
    }

    /// Get the number of matches in this file
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Get the total number of lines matched
    pub fn total_lines_matched(&self) -> usize {
        self.matched_lines().len()
    }
}
```

#### Acceptance Criteria

**Type Requirements:**
- [ ] Struct derives `Debug`, `Clone`
- [ ] Struct is `Send + Sync`
- [ ] All fields use owned types

**Methods:**
- [ ] `is_whole_file_match()` returns true when `matches.is_empty()`
- [ ] `matched_lines()` returns sorted, deduplicated 1-indexed line numbers
- [ ] `match_count()` returns `matches.len()`
- [ ] `total_lines_matched()` returns count of unique lines

**Documentation:**
- [ ] Struct has rustdoc with example
- [ ] Each field and method has `///` documentation
- [ ] Example shows both whole-file and hunk matching patterns

#### Technical Notes

**Why empty matches = whole-file match:**

When the evaluator returns `MatchResult::Boolean(true)`, it means a boolean predicate matched (like `ext:rs`). There are no specific code hunks - the entire file is the match. We represent this as an empty `matches` vector rather than a single match spanning the whole file because:

1. It's more efficient (don't need to calculate file line count)
2. It clearly distinguishes "whole file matched" from "one big hunk matched"
3. Matches the internal `MatchResult` enum semantics

---

### Story 3: Create Match Struct

**Estimated: 30 minutes | Dependencies: Story 1**

#### Goal
Create the `Match` struct for individual match locations and content. This represents a single code block ("hunk") that matched within a file.

#### Location
`src/lib.rs`

#### Implementation

```rust
/// A single match within a file
///
/// Contains precise location information and the matched text content.
/// Line numbers are 1-indexed (matching editor conventions), while
/// columns and byte ranges are 0-indexed.
///
/// # Example
///
/// ```rust
/// let m: Match = /* from search result */;
///
/// println!("Match at lines {}-{}", m.start_line, m.end_line);
/// println!("Spans {} lines", m.line_count());
///
/// // Get the first line of matched text
/// let first_line = m.text.lines().next().unwrap_or("");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Start line number (1-indexed)
    pub start_line: usize,

    /// End line number (1-indexed, inclusive)
    pub end_line: usize,

    /// Start column (0-indexed byte offset within line)
    pub start_column: usize,

    /// End column (0-indexed byte offset within line)
    pub end_column: usize,

    /// Byte range within the file (0-indexed)
    pub byte_range: std::ops::Range<usize>,

    /// The matched text content
    pub text: String,
}

impl Match {
    /// Returns the number of lines this match spans
    ///
    /// A single-line match returns 1.
    pub fn line_count(&self) -> usize {
        self.end_line - self.start_line + 1
    }

    /// Returns true if this match spans multiple lines
    pub fn is_multiline(&self) -> bool {
        self.start_line != self.end_line
    }

    /// Get the byte length of the match
    pub fn byte_len(&self) -> usize {
        self.byte_range.len()
    }

    /// Get the first line of matched text
    pub fn first_line(&self) -> &str {
        self.text.lines().next().unwrap_or("")
    }
}
```

#### Acceptance Criteria

**Type Requirements:**
- [ ] Struct derives `Debug`, `Clone`, `PartialEq`, `Eq`
- [ ] Struct is `Send + Sync`
- [ ] All fields use owned types

**Field Conventions:**
- [ ] `start_line` and `end_line` are 1-indexed (editor convention)
- [ ] `start_column` and `end_column` are 0-indexed (byte offset)
- [ ] `byte_range` is 0-indexed (file byte offset)
- [ ] `end_line` is inclusive (line 1-3 means lines 1, 2, and 3)

**Methods:**
- [ ] `line_count()` returns `end_line - start_line + 1`
- [ ] `is_multiline()` returns `start_line != end_line`
- [ ] `byte_len()` returns `byte_range.len()`
- [ ] `first_line()` returns first line of text

**Documentation:**
- [ ] Struct and all fields have rustdoc
- [ ] Indexing conventions clearly documented
- [ ] Example shows common usage patterns

#### Technical Notes

**Why 1-indexed lines but 0-indexed columns?**

- Line numbers: 1-indexed matches editor conventions (VSCode, vim, etc.)
- Columns: 0-indexed because they're byte offsets, and tree-sitter uses 0-indexed columns
- Byte ranges: 0-indexed because they're direct file offsets

**Conversion from tree-sitter Range:**

```rust
// tree-sitter Range has 0-indexed rows
let ts_range: tree_sitter::Range = /* from evaluator */;

let match_struct = Match {
    start_line: ts_range.start_point.row + 1,  // 0 → 1 indexed
    end_line: ts_range.end_point.row + 1,      // 0 → 1 indexed
    start_column: ts_range.start_point.column, // stays 0-indexed
    end_column: ts_range.end_point.column,     // stays 0-indexed
    byte_range: ts_range.start_byte..ts_range.end_byte,
    text: content[ts_range.start_byte..ts_range.end_byte].to_string(),
};
```

---

### Story 4: Create perform_search_internal Function

**Estimated: 1 hour | Dependencies: Stories 1-3**

#### Goal
Extract core search logic into a new function that accepts `SearchOptions` instead of CLI-coupled `SearchArgs`. This is the core refactoring that enables the library API.

#### Location
`src/commands/search.rs`

#### Implementation

```rust
use crate::SearchOptions;

/// Internal search implementation accepting library-friendly options
///
/// This is the core search function used by both CLI and library paths.
/// It returns raw results that can be transformed into SearchResult.
pub(crate) fn perform_search_internal(
    query: &str,
    options: &SearchOptions,
) -> Result<Vec<(PathBuf, Vec<Range>)>> {
    // Load configuration from project root
    let config_path = options.root.join(".rdump.toml");
    let config = if config_path.exists() {
        config::load_config(&config_path)?
    } else {
        config::RdumpConfig::default()
    };

    // Resolve presets
    let preset_registry = presets::PresetRegistry::load_default()?;
    let resolved_presets: Vec<_> = options.presets
        .iter()
        .filter_map(|name| preset_registry.get(name))
        .collect();

    // Build directory walker
    let mut walker_builder = WalkBuilder::new(&options.root);
    walker_builder
        .hidden(!options.hidden)
        .git_ignore(!options.no_ignore)
        .git_global(!options.no_ignore)
        .git_exclude(!options.no_ignore);

    if let Some(depth) = options.max_depth {
        walker_builder.max_depth(Some(depth));
    }

    // Get candidate files
    let candidates = get_candidate_files(walker_builder)?;

    // Parse query into AST
    let ast = if query.is_empty() {
        None
    } else {
        Some(parser::parse_query(query)?)
    };

    // Validate AST predicates against available presets
    if let Some(ref ast) = ast {
        validate_ast_predicates(ast, &preset_registry)?;
    }

    // Build evaluator with config and presets
    let evaluator = Evaluator::new(
        ast,
        &config,
        &resolved_presets,
        options.sql_dialect,
    );

    // Pre-filter pass (metadata predicates only - fast)
    let pre_filtered: Vec<_> = candidates
        .into_iter()
        .filter(|path| evaluator.pre_filter(path))
        .collect();

    // Main evaluation pass (content + semantic predicates - parallel)
    let results: Vec<(PathBuf, Vec<Range>)> = pre_filtered
        .into_par_iter()
        .filter_map(|path| {
            match evaluator.evaluate(&path) {
                Ok(MatchResult::Boolean(true)) => Some((path, vec![])),
                Ok(MatchResult::Hunks(ranges)) if !ranges.is_empty() => {
                    Some((path, ranges))
                }
                Ok(_) => None,
                Err(e) => {
                    // For library path, we should collect errors instead
                    // For now, maintain CLI behavior
                    eprintln!("Error evaluating {}: {}", path.display(), e);
                    None
                }
            }
        })
        .collect();

    Ok(results)
}

/// Existing perform_search for backward compatibility
///
/// This wrapper maintains the existing CLI API while using the new internal function.
pub fn perform_search(args: &SearchArgs) -> Result<Vec<(PathBuf, Vec<Range>)>> {
    let options = SearchOptions {
        root: args.root.clone(),
        presets: args.preset.clone(),
        no_ignore: args.no_ignore,
        hidden: args.hidden,
        max_depth: args.max_depth,
        sql_dialect: args.dialect.map(Into::into),
    };
    perform_search_internal(
        args.query.as_deref().unwrap_or(""),
        &options,
    )
}
```

#### SqlDialect Conversion

If `SqlDialectFlag` and `SqlDialect` are different types, add conversion:

```rust
impl From<SqlDialectFlag> for SqlDialect {
    fn from(flag: SqlDialectFlag) -> Self {
        match flag {
            SqlDialectFlag::Postgres => SqlDialect::Postgres,
            SqlDialectFlag::Mysql => SqlDialect::Mysql,
            SqlDialectFlag::Sqlite => SqlDialect::Sqlite,
            SqlDialectFlag::Tsql => SqlDialect::Tsql,
            SqlDialectFlag::Plpgsql => SqlDialect::Plpgsql,
        }
    }
}
```

#### Acceptance Criteria

**Function Signature:**
- [ ] `perform_search_internal` accepts `&str` query and `&SearchOptions`
- [ ] Returns `Result<Vec<(PathBuf, Vec<Range>)>>`
- [ ] Marked `pub(crate)` (internal, not public API)

**Parameter Mapping:**
- [ ] `options.root` → walker root directory
- [ ] `options.presets` → preset resolution
- [ ] `options.no_ignore` → gitignore handling (inverted for walker)
- [ ] `options.hidden` → hidden file inclusion (inverted for walker)
- [ ] `options.max_depth` → directory depth limit
- [ ] `options.sql_dialect` → SQL file parsing

**Backward Compatibility:**
- [ ] Existing `perform_search(&SearchArgs)` still works
- [ ] Returns identical results for same inputs
- [ ] All existing tests pass unchanged

**Code Quality:**
- [ ] No `unwrap()` - use `?` for error propagation
- [ ] Config loading handles missing file gracefully
- [ ] Preset resolution handles unknown presets

#### Technical Notes

**Why `pub(crate)` visibility?**

The function returns raw `(PathBuf, Vec<Range>)` tuples with tree-sitter `Range` types. We don't want to expose tree-sitter internals in the public API. The public `search_iter()` and `search()` functions wrap this and convert to `SearchResult`.

**Error handling consideration:**

The `eprintln!` in the evaluation pass is CLI behavior. For the library path, errors should be collected or returned. This will be addressed in Story 8 (edge cases).

**Performance preservation:**

- Pre-filter pass runs single-threaded (metadata only, fast)
- Main evaluation uses `into_par_iter()` for rayon parallelism
- This matches existing CLI performance characteristics

---

### Story 5: Update run_search for CLI Compatibility

**Estimated: 30 minutes | Dependencies: Story 4**

#### Goal
Update `run_search()` to use the new `perform_search_internal()` while maintaining identical CLI behavior. This ensures no regression in the existing user experience.

#### Location
`src/commands/search.rs`

#### Implementation

```rust
/// CLI entry point for search command
///
/// External behavior remains completely unchanged. Internally uses
/// the new perform_search_internal() function.
pub fn run_search(mut args: SearchArgs) -> Result<()> {
    // Handle shorthand flags (unchanged from original)
    if args.no_headers {
        args.format = Format::Cat;
    }
    if args.find {
        args.format = Format::Find;
    }

    // Convert CLI args to library options
    let options = SearchOptions {
        root: args.root.clone(),
        presets: args.preset.clone(),
        no_ignore: args.no_ignore,
        hidden: args.hidden,
        max_depth: args.max_depth,
        sql_dialect: args.dialect.map(Into::into),
    };

    let query = args.query.as_deref().unwrap_or("");

    // Use new internal search function
    let matching_files = perform_search_internal(query, &options)?;

    // Format and output (unchanged from original)
    let output_writer: Box<dyn Write> = match &args.output {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(std::io::stdout()),
    };

    formatter::print_output(
        output_writer,
        &matching_files,
        &args.format,
        args.color,
        args.line_numbers,
        args.context,
    )?;

    Ok(())
}
```

#### Acceptance Criteria

**CLI Behavior:**
- [ ] Output is byte-for-byte identical for all existing test cases
- [ ] Shorthand flags (`--no-headers`, `--find`) still work
- [ ] Output formatting options unchanged
- [ ] Color output works correctly
- [ ] File output (`--output`) works correctly

**All Existing Tests Pass:**
- [ ] Unit tests for `run_search`
- [ ] Integration tests for CLI
- [ ] Format-specific tests (default, cat, find, json)

**No Performance Regression:**
- [ ] Search time within 5% of original
- [ ] Memory usage comparable

#### Testing Approach

Run the full CLI test suite before and after this change:

```bash
# Run all existing tests
cargo test

# Run CLI integration tests specifically
cargo test --test cli_tests

# Compare output manually for key test cases
rdump search "ext:rs" --format json > before.json
# (after change)
rdump search "ext:rs" --format json > after.json
diff before.json after.json
```

#### Technical Notes

**Why clone args fields?**

`SearchOptions` takes ownership while `SearchArgs` may be borrowed. Cloning is cheap for:
- `PathBuf` - typically small strings
- `Vec<String>` - usually 0-2 presets
- Primitives - Copy types

The alternative (making `SearchOptions` use references) would require lifetime parameters, complicating the public API.

---

### Story 6: Create SearchResultIterator Struct

**Estimated: 45 minutes | Dependencies: Stories 2-3**

#### Goal
Create a streaming iterator that yields `SearchResult` items lazily, loading file content only when each result is consumed. This is the core of the memory-efficient library API.

#### Location
`src/lib.rs`

#### Implementation

```rust
/// Iterator over search results
///
/// Yields `SearchResult` items lazily, loading file content only when
/// each item is consumed. This provides memory efficiency for large
/// result sets.
///
/// # Example
///
/// ```rust
/// let iter = search_iter("ext:rs", SearchOptions::default())?;
///
/// // Process results one at a time
/// for result in iter {
///     let result = result?;
///     println!("{}", result.path.display());
/// }
///
/// // Or take just a few
/// let first_five: Vec<_> = search_iter("ext:rs", options)?
///     .take(5)
///     .collect::<Result<Vec<_>, _>>()?;
/// ```
pub struct SearchResultIterator {
    /// Raw results from perform_search_internal
    inner: std::vec::IntoIter<(PathBuf, Vec<Range>)>,
}

impl SearchResultIterator {
    /// Create a new iterator from raw search results
    pub(crate) fn new(results: Vec<(PathBuf, Vec<Range>)>) -> Self {
        Self {
            inner: results.into_iter(),
        }
    }

    /// Get the number of remaining results
    pub fn remaining(&self) -> usize {
        self.inner.len()
    }
}

impl Iterator for SearchResultIterator {
    type Item = Result<SearchResult>;

    fn next(&mut self) -> Option<Self::Item> {
        let (path, ranges) = self.inner.next()?;

        // Load file content lazily (only when this result is consumed)
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                return Some(Err(anyhow::anyhow!(
                    "Failed to read {}: {}",
                    path.display(),
                    e
                )));
            }
        };

        // Convert tree-sitter ranges to Match structs
        let matches = ranges_to_matches(&content, &ranges);

        Some(Ok(SearchResult {
            path,
            matches,
            content,
        }))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for SearchResultIterator {}

// Optionally implement FusedIterator for efficiency
impl std::iter::FusedIterator for SearchResultIterator {}
```

#### Acceptance Criteria

**Iterator Implementation:**
- [ ] Implements `Iterator<Item = Result<SearchResult>>`
- [ ] Implements `ExactSizeIterator` (knows total count)
- [ ] Implements `FusedIterator` (returns None forever after exhausted)
- [ ] `size_hint()` returns accurate `(remaining, Some(remaining))`

**Lazy Loading:**
- [ ] File content loaded only when `next()` is called
- [ ] Only one file's content in memory at a time
- [ ] Early termination (`.take(n)`) prevents unnecessary file reads

**Error Handling:**
- [ ] File read errors returned as `Err`, not panics
- [ ] Error includes file path for context
- [ ] Iterator continues after error (can skip failed files)

**Helper Methods:**
- [ ] `remaining()` returns count of unprocessed results

#### Technical Notes

**Why lazy loading matters:**

```rust
// Without lazy loading (bad for large results):
let results: Vec<SearchResult> = raw_results
    .into_iter()
    .map(|(path, ranges)| {
        let content = fs::read_to_string(&path)?;  // Loads ALL files upfront
        Ok(SearchResult { path, matches, content })
    })
    .collect()?;

// With lazy loading (memory efficient):
let iter = SearchResultIterator::new(raw_results);
for result in iter.take(10) {  // Only reads 10 files
    // ...
}
```

**Memory comparison:**

| Scenario | Without Lazy | With Lazy |
|----------|-------------|-----------|
| 1000 files, take 5 | ~10 MB | ~50 KB |
| 10000 files, take 10 | ~100 MB | ~100 KB |

---

### Story 7: Implement ranges_to_matches Helper

**Estimated: 30 minutes | Dependencies: Story 3**

#### Goal
Convert tree-sitter `Range` to user-friendly `Match` structs, handling the index conversion and text extraction safely.

#### Location
`src/lib.rs`

#### Implementation

```rust
/// Convert tree-sitter Range to user-friendly Match
///
/// Handles:
/// - 0-indexed to 1-indexed line number conversion
/// - Byte range extraction
/// - Unicode-safe text extraction
fn ranges_to_matches(content: &str, ranges: &[tree_sitter::Range]) -> Vec<Match> {
    ranges
        .iter()
        .filter_map(|r| {
            // Extract text using byte range
            // Use .get() for safe handling of invalid ranges
            let text = content
                .get(r.start_byte..r.end_byte)?
                .to_string();

            Some(Match {
                // Convert 0-indexed rows to 1-indexed line numbers
                start_line: r.start_point.row + 1,
                end_line: r.end_point.row + 1,
                // Columns stay 0-indexed (byte offset within line)
                start_column: r.start_point.column,
                end_column: r.end_point.column,
                // Byte range stays as-is
                byte_range: r.start_byte..r.end_byte,
                text,
            })
        })
        .collect()
}
```

#### Alternative: With Better Error Handling

```rust
/// Convert ranges with detailed error information
fn ranges_to_matches_with_errors(
    content: &str,
    ranges: &[tree_sitter::Range],
) -> (Vec<Match>, Vec<RangeConversionError>) {
    let mut matches = Vec::with_capacity(ranges.len());
    let mut errors = Vec::new();

    for (i, r) in ranges.iter().enumerate() {
        match content.get(r.start_byte..r.end_byte) {
            Some(text) => {
                matches.push(Match {
                    start_line: r.start_point.row + 1,
                    end_line: r.end_point.row + 1,
                    start_column: r.start_point.column,
                    end_column: r.end_point.column,
                    byte_range: r.start_byte..r.end_byte,
                    text: text.to_string(),
                });
            }
            None => {
                errors.push(RangeConversionError {
                    index: i,
                    range: r.clone(),
                    reason: "Invalid byte range for content",
                });
            }
        }
    }

    (matches, errors)
}
```

#### Acceptance Criteria

**Index Conversion:**
- [ ] Converts 0-indexed `row` to 1-indexed `start_line`/`end_line`
- [ ] Preserves 0-indexed `column` values
- [ ] Preserves byte range values

**Text Extraction:**
- [ ] Uses `.get()` for safe Unicode boundary handling
- [ ] Returns None (filtered out) for invalid ranges
- [ ] Handles empty content string

**Edge Cases:**
- [ ] Empty ranges vector returns empty matches
- [ ] Single-character matches work correctly
- [ ] Multi-byte Unicode characters handled correctly

#### Test Cases

```rust
#[test]
fn test_ranges_to_matches_basic() {
    let content = "line one\nline two\nline three";
    let ranges = vec![
        Range {
            start_byte: 0,
            end_byte: 8,
            start_point: Point { row: 0, column: 0 },
            end_point: Point { row: 0, column: 8 },
        },
    ];

    let matches = ranges_to_matches(content, &ranges);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].start_line, 1);  // 0-indexed row 0 → 1-indexed line 1
    assert_eq!(matches[0].text, "line one");
}

#[test]
fn test_ranges_to_matches_unicode() {
    let content = "fn 你好() {}";
    let ranges = vec![
        Range {
            start_byte: 3,
            end_byte: 9,  // 你好 is 6 bytes in UTF-8
            start_point: Point { row: 0, column: 3 },
            end_point: Point { row: 0, column: 9 },
        },
    ];

    let matches = ranges_to_matches(content, &ranges);

    assert_eq!(matches[0].text, "你好");
}

#[test]
fn test_ranges_to_matches_invalid_range() {
    let content = "short";
    let ranges = vec![
        Range {
            start_byte: 0,
            end_byte: 100,  // Beyond content length
            start_point: Point { row: 0, column: 0 },
            end_point: Point { row: 0, column: 100 },
        },
    ];

    let matches = ranges_to_matches(content, &ranges);

    assert_eq!(matches.len(), 0);  // Invalid range filtered out
}
```

#### Technical Notes

**Why `filter_map` instead of `map`?**

Invalid byte ranges (e.g., from corrupted tree-sitter output or race conditions where file changed) are silently dropped rather than causing panics. This makes the library more robust.

**Unicode safety:**

Using `.get()` instead of direct indexing (`content[start..end]`) prevents panics when byte indices fall in the middle of a multi-byte UTF-8 character.

---

### Story 8: Handle Iterator Edge Cases

**Estimated: 45 minutes | Dependencies: Stories 6-7**

#### Goal
Handle all edge cases in the iterator: whole-file matches, Unicode, large files, missing files, and race conditions.

#### Edge Case Implementations

##### 8.1 Whole-File Matches

```rust
// In SearchResultIterator::next()
let (path, ranges) = self.inner.next()?;

let content = std::fs::read_to_string(&path)?;

// Empty ranges = whole-file match (boolean predicate like ext:rs)
let matches = if ranges.is_empty() {
    vec![]  // Empty matches vector signals whole-file match
} else {
    ranges_to_matches(&content, &ranges)
};

// Users check with:
if result.is_whole_file_match() {
    // Entire file matched (e.g., ext:rs, lang:python)
}
```

##### 8.2 Large File Protection

```rust
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

fn read_file_content(path: &Path) -> Result<String> {
    let metadata = std::fs::metadata(path)?;

    if metadata.len() > MAX_FILE_SIZE {
        return Err(anyhow::anyhow!(
            "File {} exceeds maximum size limit ({} bytes > {} bytes)",
            path.display(),
            metadata.len(),
            MAX_FILE_SIZE
        ));
    }

    std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))
}
```

##### 8.3 Missing/Deleted Files

```rust
// In SearchResultIterator::next()
fn next(&mut self) -> Option<Self::Item> {
    let (path, ranges) = self.inner.next()?;

    // File may have been deleted between search and iteration
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Some(Err(anyhow::anyhow!(
                "File no longer exists: {}",
                path.display()
            )));
        }
        Err(e) => {
            return Some(Err(anyhow::anyhow!(
                "Failed to read {}: {}",
                path.display(),
                e
            )));
        }
    };

    // ... rest of conversion
}
```

##### 8.4 Binary File Detection

```rust
fn is_likely_binary(content: &[u8]) -> bool {
    // Check first 8KB for null bytes
    let check_len = content.len().min(8192);
    content[..check_len].contains(&0)
}

// In iterator, optionally skip binary files:
let bytes = std::fs::read(&path)?;
if is_likely_binary(&bytes) {
    return Some(Err(anyhow::anyhow!(
        "Skipping binary file: {}",
        path.display()
    )));
}
let content = String::from_utf8(bytes)
    .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in {}: {}", path.display(), e))?;
```

##### 8.5 Permission Errors

```rust
Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
    return Some(Err(anyhow::anyhow!(
        "Permission denied reading: {}",
        path.display()
    )));
}
```

#### Acceptance Criteria

**Whole-File Matches:**
- [ ] Empty ranges vector results in empty matches
- [ ] `is_whole_file_match()` returns true for empty matches
- [ ] Content is still loaded and available

**File Size Limits:**
- [ ] Files over 10MB return error (configurable)
- [ ] Error message includes file size and limit
- [ ] Small files process normally

**Missing Files:**
- [ ] Deleted files return descriptive error
- [ ] Error includes file path
- [ ] Iterator continues to next file

**Unicode Handling:**
- [ ] Valid UTF-8 files work correctly
- [ ] Invalid UTF-8 returns error with file path
- [ ] Multi-byte characters in matches work correctly

**Permission Errors:**
- [ ] Unreadable files return error
- [ ] Error is descriptive
- [ ] Iterator continues to next file

#### Test Cases

```rust
#[test]
fn test_whole_file_match() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].is_whole_file_match());
    assert!(results[0].matches.is_empty());
    assert!(!results[0].content.is_empty());  // Content still loaded
}

#[test]
fn test_missing_file_error() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let mut iter = search_iter("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).unwrap();

    // Delete file after search but before iteration
    fs::remove_file(&file).unwrap();

    let result = iter.next().unwrap();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no longer exists"));
}

#[test]
fn test_unicode_content() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("unicode.rs");
    fs::write(&file, "fn 你好() { let émoji = '🦀'; }").unwrap();

    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].content.contains("你好"));
    assert!(results[0].content.contains("🦀"));
}

#[test]
fn test_multiline_match() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {\n    println!(\"hello\");\n}").unwrap();

    let results = search("func:main", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).unwrap();

    let m = &results[0].matches[0];
    assert_eq!(m.start_line, 1);
    assert_eq!(m.end_line, 3);
    assert_eq!(m.line_count(), 3);
    assert!(m.is_multiline());
}
```

#### Technical Notes

**Error recovery strategy:**

The iterator returns `Result<SearchResult>` for each item, allowing users to:
1. Fail fast: `.collect::<Result<Vec<_>, _>>()?`
2. Skip errors: `.filter_map(Result::ok)`
3. Log and continue: `.map(|r| r.map_err(|e| eprintln!("{}", e)).ok())`

**Why not skip errors silently?**

Returning errors gives users control. Some want to know about permission issues, others want to skip them. The library shouldn't make that decision.

---

### Story 9: Create search_iter Public Function

**Estimated: 20 minutes | Dependencies: Stories 4, 6**

#### Goal
Create the primary public API function for streaming search. This is the recommended API for large codebases.

#### Location
`src/lib.rs`

#### Implementation

```rust
/// Search for files matching a query (streaming, memory-efficient)
///
/// Returns an iterator that yields results one at a time, loading file
/// content only when each result is consumed. This is the recommended
/// API for large codebases.
///
/// # Arguments
///
/// * `query` - An RQL (rdump query language) query string
/// * `options` - Search configuration options
///
/// # Returns
///
/// An iterator yielding `Result<SearchResult>` for each matching file.
///
/// # Errors
///
/// Returns an error if:
/// - The query syntax is invalid
/// - The root directory doesn't exist
/// - A preset name is not found
///
/// # Example
///
/// ```rust
/// use rdump::{search_iter, SearchOptions};
///
/// let results = search_iter(
///     "ext:rs & func:main",
///     SearchOptions::default(),
/// )?;
///
/// for result in results {
///     let result = result?;
///     println!("{}: {} matches",
///         result.path.display(),
///         result.matches.len()
///     );
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Early Termination
///
/// ```rust
/// // Find first 10 matches only
/// let first_ten: Vec<_> = search_iter("ext:rs", options)?
///     .take(10)
///     .collect::<Result<Vec<_>, _>>()?;
/// ```
///
/// # Error Handling
///
/// ```rust
/// // Skip files that can't be read
/// let results: Vec<_> = search_iter("ext:rs", options)?
///     .filter_map(Result::ok)
///     .collect();
/// ```
pub fn search_iter(
    query: &str,
    options: SearchOptions,
) -> Result<SearchResultIterator> {
    let raw_results = commands::search::perform_search_internal(query, &options)?;
    Ok(SearchResultIterator::new(raw_results))
}
```

#### Acceptance Criteria

**Function Signature:**
- [ ] Public function in `rdump` crate root
- [ ] Takes `&str` query and owned `SearchOptions`
- [ ] Returns `Result<SearchResultIterator>`

**Behavior:**
- [ ] Invalid query returns `Err` immediately
- [ ] Nonexistent root returns `Err` immediately
- [ ] Unknown preset returns `Err` immediately
- [ ] Empty query matches all files (based on presets)

**Documentation:**
- [ ] Rustdoc with full description
- [ ] Documents all error conditions
- [ ] Multiple examples showing common patterns
- [ ] Explains when to use this vs `search()`

#### Technical Notes

**Why `SearchOptions` is owned, not borrowed:**

Taking ownership avoids lifetime parameters in the return type. Since `SearchOptions` is cheap to construct and users typically create it inline, this is ergonomic:

```rust
// Clean API - no lifetime annotations
let iter = search_iter("query", SearchOptions::default())?;

// vs requiring a reference (would need lifetime)
let options = SearchOptions::default();
let iter = search_iter("query", &options)?;  // iter borrows options
// options must outlive iter
```

---

### Story 10: Create search Convenience Function

**Estimated: 15 minutes | Dependencies: Story 9**

#### Goal
Create convenience wrapper that collects all results into a Vec. Best for small result sets.

#### Location
`src/lib.rs`

#### Implementation

```rust
/// Search for files matching a query (convenience wrapper)
///
/// Collects all results into a Vec. Use [`search_iter`] for large codebases
/// to avoid loading all content into memory at once.
///
/// # Arguments
///
/// * `query` - An RQL query string
/// * `options` - Search configuration options
///
/// # Returns
///
/// A vector of all matching files with their content loaded.
///
/// # Example
///
/// ```rust
/// use rdump::{search, SearchOptions};
/// use std::path::PathBuf;
///
/// let results = search(
///     "ext:rs & func:main",
///     SearchOptions {
///         root: PathBuf::from("./src"),
///         ..Default::default()
///     }
/// )?;
///
/// println!("Found {} files", results.len());
///
/// for result in &results {
///     if result.is_whole_file_match() {
///         println!("  {} (whole file)", result.path.display());
///     } else {
///         println!("  {} ({} hunks)",
///             result.path.display(),
///             result.matches.len()
///         );
///     }
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Performance Note
///
/// This loads all matching file contents into memory. For repositories
/// with many matches, consider using [`search_iter`] instead:
///
/// ```rust
/// // Better for large result sets
/// for result in search_iter("ext:rs", options)? {
///     process(result?);
/// }
/// ```
pub fn search(query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
    search_iter(query, options)?.collect()
}
```

#### Acceptance Criteria

**Function Signature:**
- [ ] Public function in `rdump` crate root
- [ ] Takes `&str` query and owned `SearchOptions`
- [ ] Returns `Result<Vec<SearchResult>>`

**Behavior:**
- [ ] Collects all results, including errors
- [ ] First error encountered causes `Err` return
- [ ] Empty result set returns `Ok(vec![])`

**Documentation:**
- [ ] Rustdoc with description and example
- [ ] Performance warning for large result sets
- [ ] Points users to `search_iter` for large codebases

#### Technical Notes

**Why does `.collect()` work?**

`SearchResultIterator` yields `Result<SearchResult>`, and `collect()` on an iterator of `Result<T, E>` into `Result<Vec<T>, E>` is a special impl that short-circuits on the first error. This is usually what users want.

---

### Story 11: Module Organization and Exports

**Estimated: 30 minutes | Dependencies: Stories 1-10**

#### Goal
Export all public types and functions from `lib.rs` with proper organization. This makes the library API discoverable and ensures users can import everything they need from the crate root.

#### Location
`src/lib.rs`

#### Implementation

```rust
// At the top of lib.rs, organize exports clearly

// =============================================================================
// Library API Types
// =============================================================================

/// Options for performing a search (library-friendly)
#[derive(Debug, Clone)]
pub struct SearchOptions { /* ... */ }

/// A file that matched the search query
#[derive(Debug, Clone)]
pub struct SearchResult { /* ... */ }

/// A single match within a file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match { /* ... */ }

/// Iterator over search results
pub struct SearchResultIterator { /* ... */ }

// =============================================================================
// Library API Functions
// =============================================================================

/// Search for files matching a query (streaming, memory-efficient)
pub fn search_iter(query: &str, options: SearchOptions) -> Result<SearchResultIterator> { /* ... */ }

/// Search for files matching a query (convenience wrapper)
pub fn search(query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> { /* ... */ }

// =============================================================================
// Re-exports for Library Users
// =============================================================================

/// Re-export SqlDialect so users don't need to dig into predicates module
pub use predicates::code_aware::SqlDialect;

// =============================================================================
// CLI Types (existing, unchanged)
// =============================================================================

/// CLI argument struct for search command
#[derive(Debug, Clone, Parser)]
pub struct SearchArgs { /* ... unchanged ... */ }

// Other existing CLI types...
```

#### Module Structure Option

Alternatively, organize into submodules:

```rust
// src/lib.rs

// Library API module
mod library_api {
    mod types;
    mod search;

    pub use types::{Match, SearchOptions, SearchResult, SearchResultIterator};
    pub use search::{search, search_iter};
}

// Re-export at crate root for convenience
pub use library_api::{
    search, search_iter,
    Match, SearchOptions, SearchResult, SearchResultIterator,
};

// External type re-export
pub use predicates::code_aware::SqlDialect;

// Keep all existing CLI exports unchanged
pub use args::{SearchArgs, /* other CLI args */};
```

#### Prelude Module (Optional Enhancement)

For even more convenient imports:

```rust
// src/lib.rs

/// Convenient imports for library users
pub mod prelude {
    pub use crate::{
        search, search_iter,
        Match, SearchOptions, SearchResult, SearchResultIterator,
        SqlDialect,
    };
}

// Usage:
// use rdump::prelude::*;
```

#### Verify Exports with Tests

```rust
// In tests/api_exports.rs

#[test]
fn test_all_types_importable() {
    // Verify all public types are importable from crate root
    use rdump::{
        Match,
        SearchOptions,
        SearchResult,
        SearchResultIterator,
        SqlDialect,
    };

    // Verify functions
    use rdump::{search, search_iter};

    // Create instances to verify they're fully usable
    let _options = SearchOptions::default();
}

#[test]
fn test_cli_exports_unchanged() {
    // Verify existing CLI exports still work
    use rdump::SearchArgs;
    // Other CLI types...
}
```

#### Acceptance Criteria

**Type Exports:**
- [ ] `rdump::SearchOptions` is accessible
- [ ] `rdump::SearchResult` is accessible
- [ ] `rdump::Match` is accessible
- [ ] `rdump::SearchResultIterator` is accessible
- [ ] `rdump::SqlDialect` is re-exported

**Function Exports:**
- [ ] `rdump::search_iter` is accessible
- [ ] `rdump::search` is accessible

**Backward Compatibility:**
- [ ] All existing CLI exports unchanged (`SearchArgs`, etc.)
- [ ] No breaking changes to existing public API
- [ ] Existing `perform_search()` still accessible if previously public

**Documentation:**
- [ ] Module-level docs explain library vs CLI usage
- [ ] Each export has rustdoc

**Verification:**
- [ ] Test file verifies all imports work
- [ ] `cargo doc` generates clean docs with all public items

#### Technical Notes

**Why re-export SqlDialect?**

`SqlDialect` lives in `predicates::code_aware::SqlDialect`. Without re-export, users would need:

```rust
// Without re-export (awkward)
use rdump::SearchOptions;
use rdump::predicates::code_aware::SqlDialect;  // Deep path

// With re-export (clean)
use rdump::{SearchOptions, SqlDialect};
```

**Keeping CLI and Library Separate:**

The CLI types (`SearchArgs`, etc.) and library types (`SearchOptions`, etc.) are both exported but serve different purposes:
- CLI types: For building command-line tools on top of rdump
- Library types: For embedding search in other Rust programs

Both are valid use cases, so both are exported.

---

### Story 12: Verify Thread Safety

**Estimated: 20 minutes | Dependencies: Story 11**

#### Goal
Ensure all public types are `Send + Sync` to enable safe use in multi-threaded contexts. This is essential for users who want to share search results across threads or use the library in async runtimes.

#### Location
`tests/thread_safety.rs`

#### Implementation

```rust
//! Thread safety tests for library API types
//!
//! These tests verify that all public types can be safely sent between
//! threads and shared across threads. This is important for:
//! - Async runtimes (tokio, async-std)
//! - Parallel processing with rayon
//! - Multi-threaded applications

use rdump::{Match, SearchOptions, SearchResult, SearchResultIterator};

/// Compile-time assertions for Send trait
#[test]
fn test_types_are_send() {
    fn assert_send<T: Send>() {}

    assert_send::<SearchOptions>();
    assert_send::<SearchResult>();
    assert_send::<Match>();
    assert_send::<SearchResultIterator>();
}

/// Compile-time assertions for Sync trait
#[test]
fn test_types_are_sync() {
    fn assert_sync<T: Sync>() {}

    assert_sync::<SearchOptions>();
    assert_sync::<SearchResult>();
    assert_sync::<Match>();
    // Note: SearchResultIterator may not be Sync (interior mutability)
    // That's okay - iterators are typically used from one thread
}

/// Verify SearchOptions can be cloned and sent to another thread
#[test]
fn test_search_options_across_threads() {
    use std::thread;

    let options = SearchOptions {
        root: std::path::PathBuf::from("/tmp"),
        presets: vec!["rust".to_string()],
        no_ignore: true,
        hidden: false,
        max_depth: Some(5),
        sql_dialect: None,
    };

    let handle = thread::spawn(move || {
        // Use options in another thread
        assert_eq!(options.presets.len(), 1);
        assert_eq!(options.max_depth, Some(5));
        options
    });

    let returned = handle.join().unwrap();
    assert_eq!(returned.root, std::path::PathBuf::from("/tmp"));
}

/// Verify SearchResult can be sent to another thread
#[test]
fn test_search_result_across_threads() {
    use std::thread;

    let result = SearchResult {
        path: std::path::PathBuf::from("test.rs"),
        matches: vec![Match {
            start_line: 1,
            end_line: 1,
            start_column: 0,
            end_column: 10,
            byte_range: 0..10,
            text: "fn main()".to_string(),
        }],
        content: "fn main() {}".to_string(),
    };

    let handle = thread::spawn(move || {
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].start_line, 1);
        result
    });

    let returned = handle.join().unwrap();
    assert_eq!(returned.path.to_str().unwrap(), "test.rs");
}

/// Verify results can be shared via Arc
#[test]
fn test_results_with_arc() {
    use std::sync::Arc;
    use std::thread;

    let result = Arc::new(SearchResult {
        path: std::path::PathBuf::from("shared.rs"),
        matches: vec![],
        content: "// shared content".to_string(),
    });

    let mut handles = vec![];

    for i in 0..3 {
        let result_clone = Arc::clone(&result);
        let handle = thread::spawn(move || {
            // Multiple threads can read the result
            assert_eq!(result_clone.path.to_str().unwrap(), "shared.rs");
            i
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Verify iterator can be moved to another thread
#[test]
fn test_iterator_across_threads() {
    use std::thread;
    use tempfile::tempdir;
    use std::fs;
    use rdump::{search_iter, SearchOptions};

    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let iter = search_iter("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).unwrap();

    // Move iterator to another thread for processing
    let handle = thread::spawn(move || {
        let results: Vec<_> = iter.filter_map(Result::ok).collect();
        results.len()
    });

    let count = handle.join().unwrap();
    assert_eq!(count, 1);
}

/// Verify parallel processing of results with rayon
#[test]
fn test_parallel_result_processing() {
    use rayon::prelude::*;
    use tempfile::tempdir;
    use std::fs;
    use rdump::{search, SearchOptions};

    let dir = tempdir().unwrap();
    for i in 0..5 {
        let file = dir.path().join(format!("file{}.rs", i));
        fs::write(&file, format!("fn func{}() {{}}", i)).unwrap();
    }

    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).unwrap();

    // Process results in parallel with rayon
    let paths: Vec<_> = results
        .par_iter()
        .map(|r| r.path.to_string_lossy().to_string())
        .collect();

    assert_eq!(paths.len(), 5);
}
```

#### Acceptance Criteria

**Compile-Time Checks:**
- [ ] `SearchOptions` implements `Send`
- [ ] `SearchOptions` implements `Sync`
- [ ] `SearchResult` implements `Send`
- [ ] `SearchResult` implements `Sync`
- [ ] `Match` implements `Send`
- [ ] `Match` implements `Sync`
- [ ] `SearchResultIterator` implements `Send`

**Runtime Tests:**
- [ ] `SearchOptions` can be moved to another thread
- [ ] `SearchResult` can be moved to another thread
- [ ] `SearchResult` can be shared via `Arc`
- [ ] `SearchResultIterator` can be moved to another thread
- [ ] Results can be processed in parallel with rayon

**All Tests Pass:**
- [ ] `cargo test --test thread_safety`

#### Technical Notes

**Why Send + Sync matters:**

- **Send**: Type can be transferred to another thread (ownership moves)
- **Sync**: Type can be shared between threads via reference (`&T` is Send)

For async code:
```rust
// This requires SearchResult: Send
tokio::spawn(async move {
    let result = get_result().await;
    process(result);  // result moved here
});
```

For parallel processing:
```rust
// This requires SearchResult: Sync
results.par_iter().for_each(|result| {
    // Shared reference across threads
});
```

**Why SearchResultIterator might not be Sync:**

Iterators typically have interior mutability (they track current position). That's fine because you usually consume an iterator from one thread. It still needs to be `Send` so you can move it to a worker thread.

**Automatic Trait Implementation:**

Rust automatically implements `Send` and `Sync` for types whose fields are all `Send`/`Sync`. Since our structs use:
- `PathBuf` (Send + Sync)
- `String` (Send + Sync)
- `Vec<T>` where T: Send + Sync
- `Option<T>` where T: Send + Sync
- `usize` (Send + Sync)
- `Range<usize>` (Send + Sync)

All our types automatically get `Send + Sync`. The tests verify this is true.

---

### Story 13: Write Core Integration Tests

**Estimated: 1 hour | Dependencies: Stories 9-11**

#### Goal
Write comprehensive integration tests for basic library functionality. These tests verify that the public API works correctly end-to-end.

#### Location
`tests/library_api.rs`

#### Implementation

```rust
//! Integration tests for the rdump library API
//!
//! These tests verify the public search API functions correctly with
//! real file system operations using temporary directories.

use anyhow::Result;
use rdump::{search, search_iter, Match, SearchOptions, SearchResult};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// =============================================================================
// Test Fixtures
// =============================================================================

/// Create a test directory with sample Rust files
fn create_rust_fixtures() -> Result<tempfile::TempDir> {
    let dir = tempdir()?;

    // Simple main.rs
    fs::write(
        dir.path().join("main.rs"),
        r#"fn main() {
    println!("Hello, world!");
}
"#,
    )?;

    // lib.rs with multiple functions
    fs::write(
        dir.path().join("lib.rs"),
        r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#,
    )?;

    // Nested file
    let subdir = dir.path().join("src");
    fs::create_dir(&subdir)?;
    fs::write(
        subdir.join("utils.rs"),
        r#"pub fn helper() -> String {
    "helper".to_string()
}
"#,
    )?;

    Ok(dir)
}

/// Create a test directory with multiple languages
fn create_multi_lang_fixtures() -> Result<tempfile::TempDir> {
    let dir = tempdir()?;

    fs::write(dir.path().join("script.py"), "def main():\n    print('hello')\n")?;
    fs::write(dir.path().join("app.js"), "function main() {\n  console.log('hello');\n}\n")?;
    fs::write(dir.path().join("lib.rs"), "fn main() {}\n")?;

    Ok(dir)
}

// =============================================================================
// Basic Search Tests
// =============================================================================

#[test]
fn test_basic_extension_search() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    // Should find main.rs, lib.rs, and src/utils.rs
    assert_eq!(results.len(), 3);

    // All results should be .rs files
    for result in &results {
        assert!(result.path.extension().unwrap() == "rs");
    }

    Ok(())
}

#[test]
fn test_search_with_no_results() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let results = search("ext:py", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert!(results.is_empty());

    Ok(())
}

#[test]
fn test_search_nonexistent_root() {
    let result = search("ext:rs", SearchOptions {
        root: PathBuf::from("/nonexistent/path/that/does/not/exist"),
        ..Default::default()
    });

    assert!(result.is_err());
}

#[test]
fn test_invalid_query_syntax() {
    let dir = tempdir().unwrap();

    let result = search("invalid((syntax", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    });

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    // Error should mention parsing/syntax issue
    assert!(err.contains("parse") || err.contains("syntax") || err.contains("unexpected"));
}

// =============================================================================
// Semantic Search Tests
// =============================================================================

#[test]
fn test_function_predicate() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let results = search("func:main", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    // Should find main.rs (has fn main)
    assert_eq!(results.len(), 1);
    assert!(results[0].path.file_name().unwrap() == "main.rs");

    // Should have actual matches (not whole-file)
    assert!(!results[0].is_whole_file_match());
    assert!(!results[0].matches.is_empty());

    // Match should be the main function
    let m = &results[0].matches[0];
    assert_eq!(m.start_line, 1);
    assert!(m.text.contains("fn main"));

    Ok(())
}

#[test]
fn test_function_predicate_multiple_matches() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let results = search("func:add | func:subtract", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    // Should find lib.rs (has add and subtract)
    assert_eq!(results.len(), 1);
    assert!(results[0].path.file_name().unwrap() == "lib.rs");

    // Should have two matches
    assert_eq!(results[0].matches.len(), 2);

    Ok(())
}

// =============================================================================
// Compound Query Tests
// =============================================================================

#[test]
fn test_and_query() -> Result<()> {
    let dir = create_rust_fixtures()?;

    // Find .rs files containing 'main' function
    let results = search("ext:rs & func:main", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(results.len(), 1);
    assert!(results[0].path.file_name().unwrap() == "main.rs");

    Ok(())
}

#[test]
fn test_or_query() -> Result<()> {
    let dir = create_multi_lang_fixtures()?;

    // Find Python or JavaScript files
    let results = search("ext:py | ext:js", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(results.len(), 2);

    let extensions: Vec<_> = results
        .iter()
        .map(|r| r.path.extension().unwrap().to_str().unwrap())
        .collect();

    assert!(extensions.contains(&"py"));
    assert!(extensions.contains(&"js"));

    Ok(())
}

#[test]
fn test_not_query() -> Result<()> {
    let dir = create_multi_lang_fixtures()?;

    // Find non-Rust files
    let results = search("!ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(results.len(), 2);

    for result in &results {
        assert_ne!(result.path.extension().unwrap(), "rs");
    }

    Ok(())
}

#[test]
fn test_complex_compound_query() -> Result<()> {
    let dir = create_rust_fixtures()?;

    // Find .rs files with test attribute OR main function
    let results = search("ext:rs & (content:test | func:main)", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    // Should find main.rs and lib.rs (which has #[test])
    assert_eq!(results.len(), 2);

    Ok(())
}

// =============================================================================
// Whole-File Match Tests
// =============================================================================

#[test]
fn test_whole_file_match() -> Result<()> {
    let dir = create_rust_fixtures()?;

    // Extension predicates result in whole-file matches
    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    for result in &results {
        // Extension matches are whole-file (no specific hunks)
        assert!(result.is_whole_file_match());
        assert!(result.matches.is_empty());
        // But content is still loaded
        assert!(!result.content.is_empty());
    }

    Ok(())
}

#[test]
fn test_whole_file_match_content_available() -> Result<()> {
    let dir = tempdir()?;
    let content = "fn specific_content() { 42 }";
    fs::write(dir.path().join("test.rs"), content)?;

    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(results.len(), 1);
    assert!(results[0].is_whole_file_match());
    assert_eq!(results[0].content, content);

    Ok(())
}

// =============================================================================
// Match Struct Tests
// =============================================================================

#[test]
fn test_match_line_numbers_are_one_indexed() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("test.rs"),
        "// line 1\n// line 2\nfn target() {}\n// line 4\n",
    )?;

    let results = search("func:target", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(results.len(), 1);
    let m = &results[0].matches[0];

    // Function is on line 3 (1-indexed)
    assert_eq!(m.start_line, 3);
    assert_eq!(m.end_line, 3);

    Ok(())
}

#[test]
fn test_match_multiline() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("test.rs"),
        r#"fn multiline(
    arg1: i32,
    arg2: i32,
) -> i32 {
    arg1 + arg2
}"#,
    )?;

    let results = search("func:multiline", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(results.len(), 1);
    let m = &results[0].matches[0];

    // Function spans multiple lines
    assert_eq!(m.start_line, 1);
    assert_eq!(m.end_line, 6);
    assert!(m.is_multiline());
    assert_eq!(m.line_count(), 6);

    Ok(())
}

#[test]
fn test_match_byte_range() -> Result<()> {
    let dir = tempdir()?;
    let content = "fn foo() {}";
    fs::write(dir.path().join("test.rs"), content)?;

    let results = search("func:foo", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    let m = &results[0].matches[0];

    // Byte range should extract the function
    assert_eq!(&content[m.byte_range.clone()], m.text);

    Ok(())
}

// =============================================================================
// SearchResult Helper Method Tests
// =============================================================================

#[test]
fn test_matched_lines_helper() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("test.rs"),
        "fn a() {}\nfn b() {}\nfn c() {}\n",
    )?;

    // Search that will match multiple functions
    let results = search("func:a | func:b | func:c", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    let lines = results[0].matched_lines();

    // Should have lines 1, 2, 3
    assert_eq!(lines, vec![1, 2, 3]);

    Ok(())
}

#[test]
fn test_match_count_helper() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("test.rs"),
        "fn a() {}\nfn b() {}\nfn c() {}\n",
    )?;

    let results = search("func:a | func:b", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(results[0].match_count(), 2);

    Ok(())
}

// =============================================================================
// Empty Query Tests
// =============================================================================

#[test]
fn test_empty_query_with_preset() -> Result<()> {
    let dir = create_multi_lang_fixtures()?;

    // Empty query with rust preset should return all Rust files
    let results = search("", SearchOptions {
        root: dir.path().to_path_buf(),
        presets: vec!["rust".to_string()],
        ..Default::default()
    })?;

    // Should find only the .rs file
    assert_eq!(results.len(), 1);
    assert!(results[0].path.extension().unwrap() == "rs");

    Ok(())
}
```

#### Acceptance Criteria

**Test Coverage:**
- [ ] `test_basic_extension_search` - passes
- [ ] `test_search_with_no_results` - passes
- [ ] `test_search_nonexistent_root` - passes
- [ ] `test_invalid_query_syntax` - passes
- [ ] `test_function_predicate` - passes
- [ ] `test_function_predicate_multiple_matches` - passes
- [ ] `test_and_query` - passes
- [ ] `test_or_query` - passes
- [ ] `test_not_query` - passes
- [ ] `test_complex_compound_query` - passes
- [ ] `test_whole_file_match` - passes
- [ ] `test_whole_file_match_content_available` - passes
- [ ] `test_match_line_numbers_are_one_indexed` - passes
- [ ] `test_match_multiline` - passes
- [ ] `test_match_byte_range` - passes
- [ ] `test_matched_lines_helper` - passes
- [ ] `test_match_count_helper` - passes
- [ ] `test_empty_query_with_preset` - passes

**Test Quality:**
- [ ] All tests use `tempdir()` for isolation
- [ ] Tests clean up automatically
- [ ] Tests are deterministic (no timing dependencies)
- [ ] Tests have clear assertions with good error messages

**Running Tests:**
```bash
cargo test --test library_api
```

#### Technical Notes

**Why tempdir?**

Using `tempfile::tempdir()` ensures:
- Tests are isolated from each other
- Cleanup happens automatically when `TempDir` drops
- No pollution of the actual file system
- Tests can run in parallel safely

**Test Dependencies:**

Add to `Cargo.toml`:
```toml
[dev-dependencies]
tempfile = "3"
```

---

### Story 14: Write Advanced Integration Tests

**Estimated: 45 minutes | Dependencies: Story 13**

#### Goal
Write integration tests for advanced features and edge cases including streaming behavior, SearchOptions fields, and error recovery.

#### Location
`tests/library_api.rs` (continue in same file)

#### Implementation

```rust
// =============================================================================
// Streaming Iterator Tests
// =============================================================================

#[test]
fn test_search_iter_basic() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let iter = search_iter("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    let results: Vec<_> = iter.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(results.len(), 3);

    Ok(())
}

#[test]
fn test_search_iter_early_termination() -> Result<()> {
    let dir = tempdir()?;

    // Create 100 files
    for i in 0..100 {
        fs::write(dir.path().join(format!("file{}.rs", i)), "fn main() {}")?;
    }

    let iter = search_iter("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    // Only take first 5 - should not process all 100
    let first_five: Vec<_> = iter.take(5).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(first_five.len(), 5);

    // Note: This test verifies the API works; actual early termination
    // of file reads happens automatically due to lazy loading

    Ok(())
}

#[test]
fn test_search_iter_size_hint() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let iter = search_iter("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    let (lower, upper) = iter.size_hint();

    // Should know exact size
    assert_eq!(lower, 3);
    assert_eq!(upper, Some(3));

    Ok(())
}

#[test]
fn test_search_iter_remaining() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let mut iter = search_iter("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(iter.remaining(), 3);

    iter.next();
    assert_eq!(iter.remaining(), 2);

    iter.next();
    assert_eq!(iter.remaining(), 1);

    iter.next();
    assert_eq!(iter.remaining(), 0);

    Ok(())
}

#[test]
fn test_search_iter_skip_errors() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("good.rs"), "fn main() {}")?;

    let iter = search_iter("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    // Demonstrates how to skip errors and continue
    let results: Vec<_> = iter.filter_map(Result::ok).collect();

    assert_eq!(results.len(), 1);

    Ok(())
}

// =============================================================================
// SearchOptions Field Tests
// =============================================================================

#[test]
fn test_max_depth_option() -> Result<()> {
    let dir = tempdir()?;

    // Create nested structure
    // root/
    //   level1.rs           (depth 1)
    //   sub/
    //     level2.rs         (depth 2)
    //     sub/
    //       level3.rs       (depth 3)

    fs::write(dir.path().join("level1.rs"), "fn main() {}")?;

    let sub = dir.path().join("sub");
    fs::create_dir(&sub)?;
    fs::write(sub.join("level2.rs"), "fn main() {}")?;

    let subsub = sub.join("sub");
    fs::create_dir(&subsub)?;
    fs::write(subsub.join("level3.rs"), "fn main() {}")?;

    // Without depth limit - find all 3
    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;
    assert_eq!(results.len(), 3);

    // With max_depth: 1 - find only level1.rs
    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        max_depth: Some(1),
        ..Default::default()
    })?;
    assert_eq!(results.len(), 1);
    assert!(results[0].path.file_name().unwrap() == "level1.rs");

    // With max_depth: 2 - find level1.rs and level2.rs
    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        max_depth: Some(2),
        ..Default::default()
    })?;
    assert_eq!(results.len(), 2);

    Ok(())
}

#[test]
fn test_hidden_files_option() -> Result<()> {
    let dir = tempdir()?;

    fs::write(dir.path().join("visible.rs"), "fn main() {}")?;
    fs::write(dir.path().join(".hidden.rs"), "fn main() {}")?;

    // Without hidden: true - only visible file
    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        hidden: false,
        ..Default::default()
    })?;
    assert_eq!(results.len(), 1);
    assert!(results[0].path.file_name().unwrap() == "visible.rs");

    // With hidden: true - both files
    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        hidden: true,
        ..Default::default()
    })?;
    assert_eq!(results.len(), 2);

    Ok(())
}

#[test]
fn test_no_ignore_option() -> Result<()> {
    let dir = tempdir()?;

    // Create a .gitignore
    fs::write(dir.path().join(".gitignore"), "ignored.rs\n")?;
    fs::write(dir.path().join("included.rs"), "fn main() {}")?;
    fs::write(dir.path().join("ignored.rs"), "fn main() {}")?;

    // With no_ignore: false (default) - respect .gitignore
    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        no_ignore: false,
        ..Default::default()
    })?;
    assert_eq!(results.len(), 1);
    assert!(results[0].path.file_name().unwrap() == "included.rs");

    // With no_ignore: true - include ignored files
    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        no_ignore: true,
        ..Default::default()
    })?;
    assert_eq!(results.len(), 2);

    Ok(())
}

#[test]
fn test_presets_option() -> Result<()> {
    let dir = create_multi_lang_fixtures()?;

    // Use rust preset - should only find .rs files
    let results = search("", SearchOptions {
        root: dir.path().to_path_buf(),
        presets: vec!["rust".to_string()],
        ..Default::default()
    })?;

    assert_eq!(results.len(), 1);
    assert!(results[0].path.extension().unwrap() == "rs");

    Ok(())
}

#[test]
fn test_multiple_presets() -> Result<()> {
    let dir = create_multi_lang_fixtures()?;

    // Use multiple presets
    let results = search("", SearchOptions {
        root: dir.path().to_path_buf(),
        presets: vec!["rust".to_string(), "python".to_string()],
        ..Default::default()
    })?;

    // Should find .rs and .py files
    assert_eq!(results.len(), 2);

    let extensions: Vec<_> = results
        .iter()
        .map(|r| r.path.extension().unwrap().to_str().unwrap())
        .collect();

    assert!(extensions.contains(&"rs"));
    assert!(extensions.contains(&"py"));

    Ok(())
}

#[test]
fn test_custom_root() -> Result<()> {
    let dir = create_rust_fixtures()?;

    // Search only in the src subdirectory
    let results = search("ext:rs", SearchOptions {
        root: dir.path().join("src"),
        ..Default::default()
    })?;

    // Should only find utils.rs
    assert_eq!(results.len(), 1);
    assert!(results[0].path.file_name().unwrap() == "utils.rs");

    Ok(())
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_unicode_in_file_content() -> Result<()> {
    let dir = tempdir()?;

    // File with Unicode content
    fs::write(
        dir.path().join("unicode.rs"),
        r#"fn 你好() -> &'static str {
    "こんにちは 🦀"
}
"#,
    )?;

    let results = search("func:你好", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(results.len(), 1);
    assert!(results[0].matches[0].text.contains("你好"));

    Ok(())
}

#[test]
fn test_unicode_in_file_path() -> Result<()> {
    let dir = tempdir()?;

    // Create file with Unicode in path
    let path = dir.path().join("código.rs");
    fs::write(&path, "fn main() {}")?;

    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(results.len(), 1);
    assert!(results[0].path.to_string_lossy().contains("código"));

    Ok(())
}

#[test]
fn test_empty_file() -> Result<()> {
    let dir = tempdir()?;

    fs::write(dir.path().join("empty.rs"), "")?;

    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(results.len(), 1);
    assert!(results[0].content.is_empty());

    Ok(())
}

#[test]
fn test_very_long_lines() -> Result<()> {
    let dir = tempdir()?;

    // Create file with very long line
    let long_string = "x".repeat(10000);
    fs::write(
        dir.path().join("long.rs"),
        format!("fn main() {{ let s = \"{}\"; }}", long_string),
    )?;

    let results = search("func:main", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    assert_eq!(results.len(), 1);
    assert!(results[0].matches[0].text.len() > 10000);

    Ok(())
}

#[test]
fn test_symlinks_not_followed_by_default() -> Result<()> {
    // Skip on Windows where symlinks require special permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let dir = tempdir()?;
        let target_dir = tempdir()?;

        // Create file in target directory
        fs::write(target_dir.path().join("target.rs"), "fn main() {}")?;

        // Create symlink in search directory
        symlink(target_dir.path(), dir.path().join("link"))?;

        // By default, symlinks should not be followed
        let results = search("ext:rs", SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        })?;

        // Behavior depends on walker configuration
        // This test documents the current behavior
        println!("Symlink test found {} results", results.len());
    }

    Ok(())
}

#[test]
fn test_binary_file_detection() -> Result<()> {
    let dir = tempdir()?;

    // Create a text file
    fs::write(dir.path().join("text.rs"), "fn main() {}")?;

    // Create a binary file (with null bytes)
    let mut binary_content = vec![0u8; 100];
    binary_content[0] = b'f';
    binary_content[1] = b'n';
    fs::write(dir.path().join("binary.dat"), binary_content)?;

    let results = search("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })?;

    // Should only find text file
    assert_eq!(results.len(), 1);

    Ok(())
}

// =============================================================================
// Concurrent Access Tests
// =============================================================================

#[test]
fn test_results_can_be_accessed_after_tempdir_dropped() {
    // This is important: results should own their data
    let results = {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {}").unwrap();

        search("ext:rs", SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        })
        .unwrap()
        // dir is dropped here
    };

    // Results should still be valid
    assert_eq!(results.len(), 1);
    assert!(results[0].content.contains("fn main"));
}
```

#### Acceptance Criteria

**Streaming Tests:**
- [ ] `test_search_iter_basic` - passes
- [ ] `test_search_iter_early_termination` - passes
- [ ] `test_search_iter_size_hint` - passes
- [ ] `test_search_iter_remaining` - passes
- [ ] `test_search_iter_skip_errors` - passes

**SearchOptions Tests:**
- [ ] `test_max_depth_option` - passes
- [ ] `test_hidden_files_option` - passes
- [ ] `test_no_ignore_option` - passes
- [ ] `test_presets_option` - passes
- [ ] `test_multiple_presets` - passes
- [ ] `test_custom_root` - passes

**Edge Case Tests:**
- [ ] `test_unicode_in_file_content` - passes
- [ ] `test_unicode_in_file_path` - passes
- [ ] `test_empty_file` - passes
- [ ] `test_very_long_lines` - passes
- [ ] `test_symlinks_not_followed_by_default` - passes
- [ ] `test_binary_file_detection` - passes
- [ ] `test_results_can_be_accessed_after_tempdir_dropped` - passes

**Running Tests:**
```bash
cargo test --test library_api
```

---

### Story 15: Create Basic Example Program

**Estimated: 30 minutes | Dependencies: Stories 9-11**

#### Goal
Create a comprehensive example demonstrating library usage patterns for common use cases. This example serves as both documentation and a quick-start guide.

#### Location
`examples/basic_search.rs`

#### Implementation

```rust
//! Basic rdump library usage example
//!
//! Demonstrates common patterns for using the rdump library API.
//!
//! Run with: cargo run --example basic_search

use anyhow::Result;
use rdump::{search, SearchOptions, SearchResult, SqlDialect};
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("rdump Library API Examples\n");

    // Example 1: Simple extension search
    example_extension_search()?;

    // Example 2: Semantic function search
    example_function_search()?;

    // Example 3: Compound queries
    example_compound_query()?;

    // Example 4: Custom search options
    example_custom_options()?;

    // Example 5: Working with results
    example_working_with_results()?;

    println!("\nAll examples completed successfully!");
    Ok(())
}

/// Example 1: Search by file extension
fn example_extension_search() -> Result<()> {
    println!("=== Example 1: Extension Search ===");

    // Find all Rust files in current directory
    let results = search("ext:rs", SearchOptions::default())?;

    println!("Found {} Rust files:", results.len());
    for result in results.iter().take(5) {
        println!("  - {}", result.path.display());
    }
    if results.len() > 5 {
        println!("  ... and {} more", results.len() - 5);
    }

    println!();
    Ok(())
}

/// Example 2: Search for functions by name
fn example_function_search() -> Result<()> {
    println!("=== Example 2: Function Search ===");

    // Find all files containing a 'main' function
    let results = search("func:main", SearchOptions::default())?;

    println!("Found {} files with main function:", results.len());
    for result in &results {
        if result.is_whole_file_match() {
            println!("  {} (whole file match)", result.path.display());
        } else {
            for m in &result.matches {
                println!(
                    "  {}:{} - {}",
                    result.path.display(),
                    m.start_line,
                    m.first_line()
                );
            }
        }
    }

    println!();
    Ok(())
}

/// Example 3: Compound queries with AND/OR
fn example_compound_query() -> Result<()> {
    println!("=== Example 3: Compound Query ===");

    // Find Rust files that contain either 'test' or 'main' function
    let query = "ext:rs & (func:test | func:main)";
    let results = search(query, SearchOptions::default())?;

    println!("Query: {}", query);
    println!("Found {} matching files:", results.len());
    for result in results.iter().take(5) {
        println!("  - {} ({} matches)",
            result.path.display(),
            result.match_count()
        );
    }

    println!();
    Ok(())
}

/// Example 4: Custom search options
fn example_custom_options() -> Result<()> {
    println!("=== Example 4: Custom Options ===");

    // Create custom options for a specific search
    let options = SearchOptions {
        // Search in a specific directory
        root: PathBuf::from("."),

        // Use a preset to limit to specific file types
        presets: vec!["rust".to_string()],

        // Include hidden files (like .github)
        hidden: true,

        // Ignore .gitignore rules
        no_ignore: false,

        // Limit directory depth
        max_depth: Some(3),

        // Use default SQL dialect (auto-detect)
        sql_dialect: None,
    };

    let results = search("", options)?;

    println!("Custom search found {} files", results.len());
    for result in results.iter().take(3) {
        println!("  - {}", result.path.display());
    }

    println!();
    Ok(())
}

/// Example 5: Working with search results
fn example_working_with_results() -> Result<()> {
    println!("=== Example 5: Working with Results ===");

    let results = search("func:new", SearchOptions::default())?;

    if results.is_empty() {
        println!("No results found");
        return Ok(());
    }

    // Get the first result
    let result = &results[0];

    println!("First result: {}", result.path.display());
    println!("  Content length: {} bytes", result.content.len());
    println!("  Is whole file match: {}", result.is_whole_file_match());
    println!("  Match count: {}", result.match_count());

    // If there are specific matches, show details
    if !result.is_whole_file_match() {
        println!("  Matched lines: {:?}", result.matched_lines());

        for (i, m) in result.matches.iter().enumerate() {
            println!("\n  Match {}:", i + 1);
            println!("    Lines: {}-{}", m.start_line, m.end_line);
            println!("    Columns: {}-{}", m.start_column, m.end_column);
            println!("    Bytes: {:?}", m.byte_range);
            println!("    Multiline: {}", m.is_multiline());

            // Show first few characters of matched text
            let preview: String = m.text.chars().take(50).collect();
            if m.text.len() > 50 {
                println!("    Text: {}...", preview);
            } else {
                println!("    Text: {}", preview);
            }
        }
    }

    println!();
    Ok(())
}

// =============================================================================
// Additional utility examples
// =============================================================================

/// Example: Find and count function definitions per file
#[allow(dead_code)]
fn count_functions_per_file() -> Result<()> {
    let results = search("func:*", SearchOptions {
        presets: vec!["rust".to_string()],
        ..Default::default()
    })?;

    let mut files_with_counts: Vec<_> = results
        .iter()
        .map(|r| (r.path.clone(), r.match_count()))
        .collect();

    // Sort by function count descending
    files_with_counts.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Files by function count:");
    for (path, count) in files_with_counts.iter().take(10) {
        println!("  {:>3} functions: {}", count, path.display());
    }

    Ok(())
}

/// Example: Collect all function names in a project
#[allow(dead_code)]
fn collect_function_names() -> Result<Vec<String>> {
    let results = search("func:*", SearchOptions::default())?;

    let mut function_names = Vec::new();

    for result in &results {
        for m in &result.matches {
            // Extract function name from first line
            // This is a simplified example; actual extraction would need parsing
            if let Some(name) = extract_function_name(&m.text) {
                function_names.push(name);
            }
        }
    }

    Ok(function_names)
}

fn extract_function_name(text: &str) -> Option<String> {
    // Simplified extraction - real implementation would use tree-sitter
    let line = text.lines().next()?;
    if line.contains("fn ") {
        let start = line.find("fn ")? + 3;
        let end = line[start..].find('(')?;
        Some(line[start..start + end].trim().to_string())
    } else {
        None
    }
}

/// Example: Filter results programmatically
#[allow(dead_code)]
fn filter_results_example() -> Result<()> {
    let results = search("ext:rs", SearchOptions::default())?;

    // Filter to only files in src directory
    let src_files: Vec<_> = results
        .iter()
        .filter(|r| r.path.to_string_lossy().contains("/src/"))
        .collect();

    // Filter to files with certain content
    let with_unsafe: Vec<_> = results
        .iter()
        .filter(|r| r.content.contains("unsafe"))
        .collect();

    println!("Files in src: {}", src_files.len());
    println!("Files with unsafe: {}", with_unsafe.len());

    Ok(())
}
```

#### Acceptance Criteria

**Compilation:**
- [ ] Example compiles without warnings
- [ ] Example runs successfully
- [ ] Works with `cargo run --example basic_search`

**Content:**
- [ ] Shows extension search pattern
- [ ] Shows function search pattern
- [ ] Shows compound query pattern
- [ ] Shows custom SearchOptions usage
- [ ] Shows how to work with SearchResult and Match structs
- [ ] Includes helpful comments explaining each part

**Output:**
- [ ] Produces readable, formatted output
- [ ] Handles edge cases (empty results, many results)
- [ ] Shows practical information about results

#### Technical Notes

**Why these examples?**

1. **Extension search**: Most common use case
2. **Function search**: Shows semantic search capability
3. **Compound queries**: Shows query language power
4. **Custom options**: Shows all configuration options
5. **Working with results**: Shows how to process results

**Running the example:**

```bash
# From project root
cargo run --example basic_search

# With a specific directory
cd /path/to/project
cargo run --example basic_search --manifest-path /path/to/rdump/Cargo.toml
```

---

### Story 16: Create Streaming Example Program

**Estimated: 30 minutes | Dependencies: Story 9**

#### Goal
Create a comprehensive example demonstrating the streaming API for memory-efficient processing of large codebases. Shows iterator patterns, early termination, error handling, and real-world use cases.

#### Location
`examples/streaming_search.rs`

#### Implementation

```rust
//! Streaming search example for rdump library
//!
//! Demonstrates memory-efficient patterns for processing large codebases
//! using the search_iter() API.
//!
//! Run with: cargo run --example streaming_search

use anyhow::Result;
use rdump::{search_iter, SearchOptions, SearchResult};
use std::io::{self, Write};
use std::time::Instant;

fn main() -> Result<()> {
    println!("rdump Streaming API Examples\n");

    // Example 1: Basic streaming iteration
    example_basic_streaming()?;

    // Example 2: Early termination
    example_early_termination()?;

    // Example 3: Skip errors and continue
    example_skip_errors()?;

    // Example 4: Progress reporting
    example_progress_reporting()?;

    // Example 5: Parallel processing of results
    example_parallel_processing()?;

    // Example 6: Memory-efficient aggregation
    example_memory_efficient_aggregation()?;

    println!("\nAll streaming examples completed!");
    Ok(())
}

/// Example 1: Basic streaming iteration
///
/// Process results one at a time without loading all into memory
fn example_basic_streaming() -> Result<()> {
    println!("=== Example 1: Basic Streaming ===");

    let iter = search_iter("ext:rs", SearchOptions::default())?;

    println!("Processing {} files...", iter.remaining());

    let mut count = 0;
    for result in iter {
        match result {
            Ok(r) => {
                count += 1;
                // Process each file individually
                // Content is loaded only when we reach this point
                if count <= 3 {
                    println!("  {} ({} bytes)", r.path.display(), r.content.len());
                }
            }
            Err(e) => {
                eprintln!("  Error: {}", e);
            }
        }
    }

    if count > 3 {
        println!("  ... and {} more files", count - 3);
    }

    println!();
    Ok(())
}

/// Example 2: Early termination with .take()
///
/// Stop processing after finding N results - remaining files are never read
fn example_early_termination() -> Result<()> {
    println!("=== Example 2: Early Termination ===");

    let iter = search_iter("ext:rs", SearchOptions::default())?;
    let total = iter.remaining();

    println!("Total matching files: {}", total);
    println!("Taking only first 5...\n");

    // Only the first 5 files will have their content read
    let first_five: Vec<SearchResult> = iter
        .take(5)
        .filter_map(Result::ok)
        .collect();

    for result in &first_five {
        println!("  - {}", result.path.display());
    }

    println!("\nProcessed {} of {} files", first_five.len(), total);
    println!("(Remaining {} files were not read from disk)\n", total - first_five.len());

    Ok(())
}

/// Example 3: Skip errors and continue processing
///
/// Demonstrates error handling strategies
fn example_skip_errors() -> Result<()> {
    println!("=== Example 3: Error Handling ===");

    let iter = search_iter("ext:rs", SearchOptions::default())?;

    // Strategy 1: Collect all errors
    let mut successes = Vec::new();
    let mut errors = Vec::new();

    for result in iter {
        match result {
            Ok(r) => successes.push(r),
            Err(e) => errors.push(e),
        }
    }

    println!("Successes: {}, Errors: {}", successes.len(), errors.len());

    // Strategy 2: Skip errors silently (use filter_map)
    let iter2 = search_iter("ext:rs", SearchOptions::default())?;
    let results: Vec<_> = iter2.filter_map(Result::ok).collect();
    println!("Results (skipping errors): {}", results.len());

    // Strategy 3: Fail on first error (use collect::<Result<Vec<_>, _>>())
    let iter3 = search_iter("ext:rs", SearchOptions::default())?;
    match iter3.collect::<Result<Vec<_>, _>>() {
        Ok(results) => println!("All {} files processed successfully", results.len()),
        Err(e) => println!("Failed on: {}", e),
    }

    println!();
    Ok(())
}

/// Example 4: Progress reporting during iteration
fn example_progress_reporting() -> Result<()> {
    println!("=== Example 4: Progress Reporting ===");

    let mut iter = search_iter("ext:rs", SearchOptions::default())?;
    let total = iter.remaining();

    if total == 0 {
        println!("No files to process\n");
        return Ok(());
    }

    let start = Instant::now();
    let mut processed = 0;
    let mut total_bytes = 0;

    print!("Processing: ");
    io::stdout().flush()?;

    while let Some(result) = iter.next() {
        processed += 1;

        if let Ok(r) = result {
            total_bytes += r.content.len();
        }

        // Update progress every 10 files
        if processed % 10 == 0 || processed == total {
            print!("\rProcessing: {}/{} files ({:.1}%)",
                processed, total,
                (processed as f64 / total as f64) * 100.0
            );
            io::stdout().flush()?;
        }
    }

    let elapsed = start.elapsed();
    println!("\n\nCompleted in {:?}", elapsed);
    println!("Total bytes processed: {} KB", total_bytes / 1024);
    println!("Average: {:.2} files/sec\n", processed as f64 / elapsed.as_secs_f64());

    Ok(())
}

/// Example 5: Parallel processing of streamed results
///
/// Collect results then process them in parallel
fn example_parallel_processing() -> Result<()> {
    println!("=== Example 5: Parallel Processing ===");

    use rayon::prelude::*;

    // First, collect results (streaming)
    let results: Vec<_> = search_iter("ext:rs", SearchOptions::default())?
        .filter_map(Result::ok)
        .collect();

    println!("Collected {} files", results.len());

    // Then process in parallel
    let total_lines: usize = results
        .par_iter()
        .map(|r| r.content.lines().count())
        .sum();

    println!("Total lines of code: {}", total_lines);

    // Find file with most lines (parallel)
    if let Some((path, lines)) = results
        .par_iter()
        .map(|r| (&r.path, r.content.lines().count()))
        .max_by_key(|(_, lines)| *lines)
    {
        println!("Largest file: {} ({} lines)", path.display(), lines);
    }

    println!();
    Ok(())
}

/// Example 6: Memory-efficient aggregation
///
/// Aggregate statistics without keeping all results in memory
fn example_memory_efficient_aggregation() -> Result<()> {
    println!("=== Example 6: Memory-Efficient Aggregation ===");

    let iter = search_iter("func:*", SearchOptions {
        presets: vec!["rust".to_string()],
        ..Default::default()
    })?;

    // Track statistics without storing all results
    let mut file_count = 0;
    let mut total_functions = 0;
    let mut max_functions = 0;
    let mut max_functions_file = String::new();
    let mut total_lines = 0;

    for result in iter.filter_map(Result::ok) {
        file_count += 1;
        let func_count = result.match_count();
        total_functions += func_count;

        if func_count > max_functions {
            max_functions = func_count;
            max_functions_file = result.path.display().to_string();
        }

        // Count lines in matched functions
        for m in &result.matches {
            total_lines += m.line_count();
        }

        // result is dropped here - content memory is freed
    }

    println!("Statistics:");
    println!("  Files analyzed: {}", file_count);
    println!("  Total functions: {}", total_functions);
    println!("  Total function lines: {}", total_lines);
    if file_count > 0 {
        println!("  Avg functions/file: {:.1}", total_functions as f64 / file_count as f64);
        println!("  Max functions in file: {} ({})", max_functions, max_functions_file);
    }

    println!();
    Ok(())
}

// =============================================================================
// Additional streaming patterns
// =============================================================================

/// Pattern: Find first match meeting criteria
#[allow(dead_code)]
fn find_first_with_condition() -> Result<Option<SearchResult>> {
    let iter = search_iter("ext:rs", SearchOptions::default())?;

    // Find first file containing "unsafe"
    for result in iter {
        if let Ok(r) = result {
            if r.content.contains("unsafe") {
                return Ok(Some(r));
            }
        }
    }

    Ok(None)
}

/// Pattern: Windowed processing
#[allow(dead_code)]
fn process_in_batches() -> Result<()> {
    let mut iter = search_iter("ext:rs", SearchOptions::default())?;

    let batch_size = 100;
    let mut batch = Vec::with_capacity(batch_size);

    loop {
        // Fill batch
        batch.clear();
        for _ in 0..batch_size {
            match iter.next() {
                Some(Ok(r)) => batch.push(r),
                Some(Err(_)) => continue,
                None => break,
            }
        }

        if batch.is_empty() {
            break;
        }

        // Process batch
        println!("Processing batch of {} files", batch.len());
        // ... batch processing logic ...
    }

    Ok(())
}

/// Pattern: Two-phase processing
#[allow(dead_code)]
fn two_phase_processing() -> Result<()> {
    // Phase 1: Quick scan to collect paths
    let paths: Vec<_> = search_iter("ext:rs", SearchOptions::default())?
        .filter_map(Result::ok)
        .map(|r| r.path.clone())
        .collect();

    println!("Phase 1: Found {} files", paths.len());

    // Phase 2: Detailed analysis (could be done in parallel or on-demand)
    for path in paths.iter().take(10) {
        // Re-read specific files for detailed analysis
        let content = std::fs::read_to_string(path)?;
        println!("Analyzing: {} ({} bytes)", path.display(), content.len());
    }

    Ok(())
}

/// Pattern: Streaming to external system
#[allow(dead_code)]
fn stream_to_external() -> Result<()> {
    let iter = search_iter("ext:rs", SearchOptions::default())?;

    for result in iter.filter_map(Result::ok) {
        // Stream each result to external system immediately
        // without accumulating in memory
        send_to_database(&result)?;
        send_to_search_index(&result)?;
    }

    Ok(())
}

fn send_to_database(_result: &SearchResult) -> Result<()> {
    // Placeholder for database insertion
    Ok(())
}

fn send_to_search_index(_result: &SearchResult) -> Result<()> {
    // Placeholder for search index update
    Ok(())
}
```

#### Acceptance Criteria

**Compilation:**
- [ ] Example compiles without warnings
- [ ] Example runs successfully
- [ ] Works with `cargo run --example streaming_search`

**Content:**
- [ ] Shows basic streaming iteration
- [ ] Shows early termination with `.take()`
- [ ] Shows error handling strategies (collect, skip, fail-fast)
- [ ] Shows progress reporting during iteration
- [ ] Shows parallel processing pattern
- [ ] Shows memory-efficient aggregation

**Code Quality:**
- [ ] Includes helpful comments explaining patterns
- [ ] Shows practical real-world use cases
- [ ] Demonstrates memory efficiency benefits
- [ ] Additional patterns for reference

**Output:**
- [ ] Produces clear, informative output
- [ ] Shows timing and statistics
- [ ] Handles edge cases (empty results)

#### Technical Notes

**Why streaming matters:**

| Scenario | Collect All | Streaming |
|----------|-------------|-----------|
| 10K files, take 10 | Load all 10K | Load only 10 |
| 1K files, count total | ~100 MB | ~10 KB peak |
| Find first match | Load until found | Stop immediately |

**Running the example:**

```bash
# Run in project directory
cargo run --example streaming_search

# Run against a large codebase
cd /path/to/large/project
cargo run --example streaming_search --manifest-path /path/to/rdump/Cargo.toml
```

**Dependencies:**

This example uses rayon for parallel processing:
```toml
[dev-dependencies]
rayon = "1"
```

---

### Story 17: Add Rustdoc Documentation

**Estimated: 45 minutes | Dependencies: Stories 1-11**

#### Goal
Add comprehensive rustdoc to all public types and functions, including module-level documentation, examples, and error documentation. This ensures the library is well-documented for users discovering it through `cargo doc`.

#### Location
`src/lib.rs`

#### Implementation

##### Module-Level Documentation

```rust
//! # rdump - Code Search Library
//!
//! rdump provides a library API for semantic code search using tree-sitter.
//! You can search for files by extension, language, function names, and more
//! using a powerful query language.
//!
//! ## Quick Start
//!
//! ```rust
//! use rdump::{search, SearchOptions};
//!
//! fn main() -> anyhow::Result<()> {
//!     // Find all Rust files with a main function
//!     let results = search("ext:rs & func:main", SearchOptions::default())?;
//!
//!     for result in &results {
//!         println!("{}: {} matches",
//!             result.path.display(),
//!             result.matches.len()
//!         );
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Streaming API
//!
//! For large codebases, use `search_iter()` to process results lazily:
//!
//! ```rust
//! use rdump::{search_iter, SearchOptions};
//!
//! # fn main() -> anyhow::Result<()> {
//! // Find first 10 matching files
//! let first_ten: Vec<_> = search_iter("ext:rs", SearchOptions::default())?
//!     .take(10)
//!     .filter_map(Result::ok)
//!     .collect();
//! # Ok(())
//! # }
//! ```
//!
//! ## Query Language
//!
//! rdump uses RQL (rdump Query Language) for searches:
//!
//! | Predicate | Description | Example |
//! |-----------|-------------|---------|
//! | `ext:` | File extension | `ext:rs` |
//! | `lang:` | Programming language | `lang:python` |
//! | `func:` | Function name | `func:main` |
//! | `class:` | Class name | `class:User` |
//! | `content:` | Text content | `content:TODO` |
//!
//! Combine with operators:
//! - `&` - AND
//! - `|` - OR
//! - `!` - NOT
//! - `()` - Grouping
//!
//! Example: `ext:rs & (func:new | func:default)`
//!
//! ## Feature Flags
//!
//! - `async` - Enable async API with tokio support
//!
//! ```toml
//! [dependencies]
//! rdump = { version = "0.1", features = ["async"] }
//! ```
```

##### Function Documentation Pattern

Each public function should follow this pattern:

```rust
/// Search for files matching a query (streaming, memory-efficient)
///
/// Returns an iterator that yields results one at a time, loading file
/// content only when each result is consumed. This is the recommended
/// API for large codebases.
///
/// # Arguments
///
/// * `query` - An RQL query string (see module docs for syntax)
/// * `options` - Search configuration options
///
/// # Returns
///
/// An iterator yielding `Result<SearchResult>` for each matching file.
///
/// # Errors
///
/// Returns an error if:
/// - The query syntax is invalid
/// - The root directory doesn't exist or isn't accessible
/// - A preset name is not found in the preset registry
///
/// Individual iterator items may also return errors for:
/// - File read failures
/// - Permission errors
/// - Invalid UTF-8 content
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use rdump::{search_iter, SearchOptions};
///
/// # fn main() -> anyhow::Result<()> {
/// let results = search_iter("ext:rs", SearchOptions::default())?;
///
/// for result in results {
///     let result = result?;
///     println!("{}", result.path.display());
/// }
/// # Ok(())
/// # }
/// ```
///
/// Early termination:
///
/// ```rust
/// # use rdump::{search_iter, SearchOptions};
/// # fn main() -> anyhow::Result<()> {
/// // Find first 5 matches only
/// let first_five: Vec<_> = search_iter("ext:rs", SearchOptions::default())?
///     .take(5)
///     .collect::<Result<Vec<_>, _>>()?;
/// # Ok(())
/// # }
/// ```
///
/// Skip errors:
///
/// ```rust
/// # use rdump::{search_iter, SearchOptions};
/// # fn main() -> anyhow::Result<()> {
/// let results: Vec<_> = search_iter("ext:rs", SearchOptions::default())?
///     .filter_map(Result::ok)
///     .collect();
/// # Ok(())
/// # }
/// ```
pub fn search_iter(
    query: &str,
    options: SearchOptions,
) -> Result<SearchResultIterator> {
    // ... implementation
}
```

##### Type Documentation Pattern

```rust
/// Options for performing a search (library-friendly)
///
/// This struct contains only the parameters needed for search logic,
/// excluding CLI-specific concerns like output formatting and colors.
///
/// # Example
///
/// ```rust
/// use rdump::SearchOptions;
/// use std::path::PathBuf;
///
/// let options = SearchOptions {
///     root: PathBuf::from("/path/to/project"),
///     presets: vec!["rust".to_string()],
///     max_depth: Some(5),
///     ..Default::default()
/// };
/// ```
///
/// # Default Values
///
/// | Field | Default |
/// |-------|---------|
/// | `root` | Current directory (`.`) |
/// | `presets` | Empty (no filtering) |
/// | `no_ignore` | `false` (respect .gitignore) |
/// | `hidden` | `false` (skip hidden files) |
/// | `max_depth` | `None` (unlimited) |
/// | `sql_dialect` | `None` (auto-detect) |
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Root directory to search from
    ///
    /// This is the starting point for the directory walk. Only files
    /// under this directory will be searched.
    pub root: PathBuf,

    // ... other fields with similar documentation
}
```

#### Acceptance Criteria

**Documentation Coverage:**
- [ ] Module-level docs with overview and examples
- [ ] `SearchOptions` struct fully documented
- [ ] `SearchResult` struct fully documented
- [ ] `Match` struct fully documented
- [ ] `SearchResultIterator` fully documented
- [ ] `search_iter()` function fully documented
- [ ] `search()` function fully documented
- [ ] `SqlDialect` documented in re-export

**Example Quality:**
- [ ] All examples are complete (can be run as-is)
- [ ] Examples show common use cases
- [ ] Examples demonstrate error handling
- [ ] Examples are tested via `cargo test --doc`

**Documentation Quality:**
- [ ] Errors are documented with `# Errors` sections
- [ ] Return types are documented with `# Returns` sections
- [ ] Arguments are documented with `# Arguments` sections
- [ ] Links between related items work

**Build Verification:**
- [ ] `cargo doc` generates without warnings
- [ ] `cargo test --doc` passes
- [ ] Doc tests don't depend on external state

#### Technical Notes

**Why comprehensive docs matter:**

Users discovering rdump through `cargo doc` or docs.rs should be able to:
1. Understand what the library does
2. See working examples immediately
3. Understand the query language
4. Know about error conditions
5. Choose between `search()` and `search_iter()`

**Testing doc examples:**

```bash
# Build documentation
cargo doc --open

# Run all doc tests
cargo test --doc

# Run specific doc test
cargo test --doc search_iter
```

**Doc test features:**

Use `# ` to hide boilerplate in doc examples:
```rust
/// ```rust
/// # fn main() -> anyhow::Result<()> {
/// let results = search("ext:rs", SearchOptions::default())?;
/// # Ok(())
/// # }
/// ```
```

---

### Story 18: Update README with Library Usage

**Estimated: 30 minutes | Dependencies: Story 17**

#### Goal
Add a comprehensive library usage section to README.md to help users discover and use the library API. This complements the rustdoc and provides a quick-start guide in the project's main documentation.

#### Location
`README.md`

#### Implementation

Add the following section to README.md after the CLI usage section:

```markdown
## Library Usage

rdump can be used as a Rust library in your own projects.

### Installation

Add rdump to your `Cargo.toml`:

```toml
[dependencies]
rdump = "0.1"
```

### Quick Start

```rust
use rdump::{search, SearchOptions};

fn main() -> anyhow::Result<()> {
    // Find all Rust files with a main function
    let results = search("ext:rs & func:main", SearchOptions::default())?;

    println!("Found {} files", results.len());

    for result in &results {
        println!("{}: {} matches",
            result.path.display(),
            result.matches.len()
        );
    }

    Ok(())
}
```

### Streaming API (Memory-Efficient)

For large codebases, use `search_iter()` to process results lazily:

```rust
use rdump::{search_iter, SearchOptions};

fn main() -> anyhow::Result<()> {
    let iter = search_iter("ext:rs", SearchOptions::default())?;

    println!("Processing {} files...", iter.remaining());

    // Only first 10 files are loaded from disk
    for result in iter.take(10) {
        let result = result?;
        println!("{} ({} bytes)",
            result.path.display(),
            result.content.len()
        );
    }

    Ok(())
}
```

### Search Options

Customize your search with `SearchOptions`:

```rust
use rdump::{search, SearchOptions};
use std::path::PathBuf;

let options = SearchOptions {
    // Search in a specific directory
    root: PathBuf::from("./src"),

    // Use presets to filter by language
    presets: vec!["rust".to_string()],

    // Include hidden files
    hidden: true,

    // Ignore .gitignore rules
    no_ignore: false,

    // Limit directory depth
    max_depth: Some(5),

    // SQL dialect for .sql files
    sql_dialect: None,
};

let results = search("func:new", options)?;
```

### Working with Results

```rust
use rdump::{search, SearchOptions};

let results = search("func:main", SearchOptions::default())?;

for result in &results {
    // Check if whole file matched (e.g., ext:rs)
    if result.is_whole_file_match() {
        println!("{}: whole file match", result.path.display());
        continue;
    }

    // Work with specific matches
    for m in &result.matches {
        println!("{}:{}:{}",
            result.path.display(),
            m.start_line,      // 1-indexed line number
            m.first_line()     // First line of matched text
        );

        // Multi-line matches
        if m.is_multiline() {
            println!("  Spans {} lines", m.line_count());
        }
    }

    // Aggregate statistics
    println!("  {} matches, {} lines total",
        result.match_count(),
        result.total_lines_matched()
    );
}
```

### Error Handling

```rust
use rdump::{search_iter, SearchOptions};

// Strategy 1: Fail on first error
let results = search_iter("ext:rs", SearchOptions::default())?
    .collect::<Result<Vec<_>, _>>()?;

// Strategy 2: Skip errors and continue
let results: Vec<_> = search_iter("ext:rs", SearchOptions::default())?
    .filter_map(Result::ok)
    .collect();

// Strategy 3: Collect errors separately
let mut successes = Vec::new();
let mut errors = Vec::new();

for result in search_iter("ext:rs", SearchOptions::default())? {
    match result {
        Ok(r) => successes.push(r),
        Err(e) => errors.push(e),
    }
}
```

### Async Support

Enable the `async` feature for tokio-compatible async functions:

```toml
[dependencies]
rdump = { version = "0.1", features = ["async"] }
```

```rust
use rdump::{search_async, SearchOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut stream = search_async("ext:rs", SearchOptions::default()).await?;

    while let Some(result) = stream.next().await {
        let result = result?;
        println!("{}", result.path.display());
    }

    Ok(())
}
```

### Query Language Reference

rdump uses RQL (rdump Query Language):

| Predicate | Description | Example |
|-----------|-------------|---------|
| `ext:` | File extension | `ext:rs`, `ext:py` |
| `lang:` | Programming language | `lang:rust`, `lang:python` |
| `func:` | Function name | `func:main`, `func:new` |
| `class:` | Class/struct name | `class:User` |
| `content:` | Text content | `content:TODO` |
| `path:` | Path pattern | `path:src/` |

**Operators:**
- `&` or ` ` (space) - AND
- `|` - OR
- `!` - NOT
- `()` - Grouping

**Examples:**
```
ext:rs & func:main           # Rust files with main function
ext:py | ext:js              # Python or JavaScript files
ext:rs & !path:test          # Rust files not in test directories
lang:rust & (func:new | func:default)  # Rust files with new or default
```
```

#### Acceptance Criteria

**Content:**
- [ ] Library usage section added after CLI usage
- [ ] Installation instructions included
- [ ] Quick start example works
- [ ] Streaming API example included
- [ ] SearchOptions fully explained
- [ ] Working with results demonstrated
- [ ] Error handling patterns shown
- [ ] Async support documented
- [ ] Query language reference table

**Quality:**
- [ ] All code examples compile and run
- [ ] Examples are concise but complete
- [ ] Consistent formatting with rest of README
- [ ] No broken links

**Verification:**
- [ ] Examples tested manually
- [ ] README renders correctly on GitHub
- [ ] Links to docs.rs work (after publish)

#### Technical Notes

**Why README documentation matters:**

- First thing users see on GitHub/crates.io
- Should provide immediate value without clicking further
- Complements but doesn't duplicate rustdoc

**Keeping examples in sync:**

Consider extracting examples to `examples/` directory and referencing them in README. This ensures examples are tested:

```bash
# Test README examples
cargo run --example readme_quickstart
cargo run --example readme_streaming
```

---

### Story 19: Async Feature Flag Setup

**Estimated: 20 minutes | Dependencies: None**

#### Goal
Add feature flag configuration for optional async support using tokio. This enables users to integrate rdump into async applications without adding async dependencies for users who don't need them.

#### Location
`Cargo.toml`

#### Implementation

##### Cargo.toml Changes

```toml
[package]
name = "rdump"
version = "0.1.0"
edition = "2021"
description = "Semantic code search using tree-sitter"
license = "MIT"
repository = "https://github.com/your-org/rdump"
keywords = ["search", "code", "tree-sitter", "semantic"]
categories = ["command-line-utilities", "development-tools"]

# ... existing content ...

[features]
default = []

# Enable async API with tokio support
async = ["dep:tokio", "dep:tokio-stream", "dep:futures"]

[dependencies]
# Existing dependencies
anyhow = "1"
rayon = "1"
tree-sitter = "0.20"
ignore = "0.4"
# ... other existing deps ...

# Optional async dependencies
tokio = { version = "1", features = ["fs", "sync", "rt", "macros"], optional = true }
tokio-stream = { version = "0.1", optional = true }
futures = { version = "0.3", optional = true }

[dev-dependencies]
tempfile = "3"
# For async tests
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

##### Feature Gate Usage in Code

In `src/lib.rs`:
```rust
// Conditionally compile async module
#[cfg(feature = "async")]
mod async_api;

#[cfg(feature = "async")]
pub use async_api::{search_async, search_all_async};
```

##### Documentation Update

Add to lib.rs module docs:
```rust
//! ## Feature Flags
//!
//! - **`async`** - Enable async API with tokio support
//!
//!   Adds `search_async()` and `search_all_async()` functions for use in
//!   async applications. Requires tokio runtime.
//!
//!   ```toml
//!   [dependencies]
//!   rdump = { version = "0.1", features = ["async"] }
//!   ```
```

#### Acceptance Criteria

**Cargo.toml:**
- [ ] `async` feature defined
- [ ] All async dependencies are optional
- [ ] Version specifications are compatible
- [ ] Features use `dep:` syntax for clarity

**Build Verification:**
- [ ] `cargo build` succeeds (no async)
- [ ] `cargo build --features async` succeeds
- [ ] `cargo build --all-features` succeeds
- [ ] No warnings about unused dependencies

**Documentation:**
- [ ] Feature documented in lib.rs module docs
- [ ] Cargo.toml has description comment for feature

**Testing:**
- [ ] `cargo test` passes without feature
- [ ] `cargo test --features async` passes
- [ ] CI tests both configurations

#### Technical Notes

**Why optional async?**

Not all users need async support. Making it optional:
1. Reduces compile time for sync-only users
2. Avoids tokio dependency weight (~3MB of deps)
3. Keeps the default API simple
4. Follows Rust ecosystem conventions

**Tokio feature selection:**

We need these tokio features:
- `fs` - for potential async file reading in future
- `sync` - for mpsc channels
- `rt` - for spawn_blocking
- `macros` - for `#[tokio::main]` in examples

**dep: syntax:**

Using `dep:tokio` instead of `"tokio"` in features:
```toml
# Better - explicitly enables dependency
async = ["dep:tokio", "dep:tokio-stream"]

# Older style - less clear
# async = ["tokio", "tokio-stream"]
```

---

### Story 20: Implement search_async Function

**Estimated: 45 minutes | Dependencies: Stories 9, 19**

#### Goal
Create async streaming search function using `spawn_blocking` to bridge sync and async worlds. This preserves rayon's parallelism while providing an async-friendly interface.

#### Location
`src/async_api.rs`

#### Implementation

```rust
//! Async API for rdump search
//!
//! This module provides async versions of the search functions for use
//! in tokio-based applications.

use crate::{search_iter, SearchOptions, SearchResult};
use anyhow::Result;
use futures::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Search for files matching a query (async streaming)
///
/// Returns a Stream that yields results one at a time. The search runs
/// in a blocking thread pool (via `spawn_blocking`) to preserve rayon's
/// parallelism while being async-friendly.
///
/// # Arguments
///
/// * `query` - An RQL query string
/// * `options` - Search configuration options
///
/// # Returns
///
/// A stream yielding `Result<SearchResult>` for each matching file.
///
/// # Errors
///
/// Returns an error if:
/// - The query syntax is invalid
/// - The root directory doesn't exist
/// - The blocking task fails to spawn
///
/// # Backpressure
///
/// The channel has a bounded capacity (100 items). If the consumer is slow,
/// the producer will block, providing natural backpressure.
///
/// # Example
///
/// ```rust
/// use rdump::{search_async, SearchOptions};
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut stream = search_async("ext:rs", SearchOptions::default()).await?;
///
///     while let Some(result) = stream.next().await {
///         let result = result?;
///         println!("{}", result.path.display());
///     }
///
///     Ok(())
/// }
/// ```
///
/// # Early Termination
///
/// Dropping the stream will signal the producer to stop:
///
/// ```rust
/// # use rdump::{search_async, SearchOptions};
/// # use futures::StreamExt;
/// # async fn example() -> anyhow::Result<()> {
/// let stream = search_async("ext:rs", SearchOptions::default()).await?;
///
/// // Take only first 10
/// let first_ten: Vec<_> = stream
///     .take(10)
///     .collect::<Vec<_>>()
///     .await
///     .into_iter()
///     .collect::<Result<Vec<_>, _>>()?;
/// # Ok(())
/// # }
/// ```
pub async fn search_async(
    query: &str,
    options: SearchOptions,
) -> Result<impl Stream<Item = Result<SearchResult>>> {
    // Clone query for move into blocking task
    let query = query.to_string();

    // Bounded channel for backpressure
    let (tx, rx) = mpsc::channel(100);

    // Spawn blocking task for the sync search
    tokio::task::spawn_blocking(move || {
        // Perform the search (uses rayon parallelism internally)
        let iter = match search_iter(&query, options) {
            Ok(iter) => iter,
            Err(e) => {
                // Send error and return
                let _ = tx.blocking_send(Err(e));
                return;
            }
        };

        // Stream results through the channel
        for result in iter {
            // If receiver is dropped, stop sending
            if tx.blocking_send(result).is_err() {
                break;
            }
        }
    });

    Ok(ReceiverStream::new(rx))
}

/// Search for files matching a query (async, convenience)
///
/// Collects all results into a Vec. Use `search_async()` for large
/// result sets to avoid loading all content into memory.
///
/// # Example
///
/// ```rust
/// use rdump::{search_all_async, SearchOptions};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let results = search_all_async("ext:rs", SearchOptions::default()).await?;
///     println!("Found {} files", results.len());
///     Ok(())
/// }
/// ```
pub async fn search_all_async(
    query: &str,
    options: SearchOptions,
) -> Result<Vec<SearchResult>> {
    use futures::StreamExt;

    let stream = search_async(query, options).await?;

    stream.collect::<Vec<_>>().await
        .into_iter()
        .collect()
}
```

#### Alternative Implementation with Better Error Handling

```rust
/// More robust version with explicit error handling for spawn failures
pub async fn search_async_robust(
    query: &str,
    options: SearchOptions,
) -> Result<impl Stream<Item = Result<SearchResult>>> {
    let query = query.to_string();
    let (tx, rx) = mpsc::channel(100);

    let handle = tokio::task::spawn_blocking(move || {
        let iter = search_iter(&query, options)?;

        for result in iter {
            if tx.blocking_send(result).is_err() {
                // Receiver dropped, stop early
                break;
            }
        }

        Ok::<_, anyhow::Error>(())
    });

    // Spawn a task to check for panics in the blocking task
    let error_tx = tx.clone();
    tokio::spawn(async move {
        match handle.await {
            Ok(Ok(())) => {} // Success
            Ok(Err(e)) => {
                // Search error
                let _ = error_tx.send(Err(e)).await;
            }
            Err(e) => {
                // Panic in blocking task
                let _ = error_tx.send(Err(anyhow::anyhow!("Search task panicked: {}", e))).await;
            }
        }
    });

    Ok(ReceiverStream::new(rx))
}
```

#### Acceptance Criteria

**Function Signature:**
- [ ] Returns `Result<impl Stream<Item = Result<SearchResult>>>`
- [ ] Takes `&str` query and owned `SearchOptions`
- [ ] Feature-gated with `#[cfg(feature = "async")]`

**Implementation:**
- [ ] Uses `spawn_blocking` for sync-to-async bridge
- [ ] Bounded channel (100 items) for backpressure
- [ ] Early termination when stream is dropped
- [ ] Error handling for search failures

**Documentation:**
- [ ] Full rustdoc with examples
- [ ] Documents backpressure behavior
- [ ] Documents early termination
- [ ] Shows common usage patterns

**Testing:**
- [ ] Basic streaming works
- [ ] Early termination stops producer
- [ ] Errors are propagated correctly
- [ ] Works with tokio runtime

#### Technical Notes

**Why spawn_blocking?**

The search uses rayon for parallelism, which is incompatible with async. Using `spawn_blocking`:
1. Runs search on tokio's blocking thread pool
2. Preserves rayon's parallel file evaluation
3. Bridges sync results to async stream via channel

**Backpressure mechanism:**

```
Producer (spawn_blocking)  →  Channel (100)  →  Consumer (async)
         ↓                                         ↓
    blocking_send()                             stream.next()
         ↓                                         ↓
    Blocks if full                             Wakes producer
```

If consumer is slow, channel fills up and producer blocks on `blocking_send()`.

**Memory usage:**

- Channel buffer: 100 × (SearchResult size)
- Each SearchResult includes file content
- Bounded channel prevents unbounded memory growth

**Channel size trade-offs:**

| Size | Pro | Con |
|------|-----|-----|
| Small (10) | Lower memory | More blocking |
| Medium (100) | Balanced | Default choice |
| Large (1000) | Less blocking | Higher memory |

---

### Story 21: Implement search_all_async Function

**Estimated: 15 minutes | Dependencies: Story 20**

#### Goal
Create async convenience function that collects all results into a Vec. This complements the streaming `search_async()` for smaller result sets.

**Note:** The implementation is included in Story 20's code. This story focuses on documentation and testing.

#### Location
`src/async_api.rs`

#### Implementation

```rust
/// Search for files matching a query (async, convenience)
///
/// Collects all results into a Vec. Use [`search_async`] for large
/// result sets to avoid loading all content into memory.
///
/// # Arguments
///
/// * `query` - An RQL query string
/// * `options` - Search configuration options
///
/// # Returns
///
/// A vector of all matching files with their content loaded.
///
/// # Errors
///
/// Returns an error if:
/// - The query syntax is invalid
/// - The root directory doesn't exist
/// - Any file in the results fails to read
///
/// # Example
///
/// ```rust
/// use rdump::{search_all_async, SearchOptions};
/// use std::path::PathBuf;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let results = search_all_async(
///         "ext:rs & func:main",
///         SearchOptions {
///             root: PathBuf::from("./src"),
///             ..Default::default()
///         }
///     ).await?;
///
///     println!("Found {} files with main function", results.len());
///
///     for result in &results {
///         println!("  {} ({} bytes)",
///             result.path.display(),
///             result.content.len()
///         );
///     }
///
///     Ok(())
/// }
/// ```
///
/// # Performance Note
///
/// This loads all matching file contents into memory. For repositories
/// with many matches, consider using [`search_async`] with streaming:
///
/// ```rust
/// # use rdump::{search_async, SearchOptions};
/// # use futures::StreamExt;
/// # async fn example() -> anyhow::Result<()> {
/// let mut stream = search_async("ext:rs", SearchOptions::default()).await?;
///
/// while let Some(result) = stream.next().await {
///     process(result?);
/// }
/// # fn process(_: rdump::SearchResult) {}
/// # Ok(())
/// # }
/// ```
pub async fn search_all_async(
    query: &str,
    options: SearchOptions,
) -> Result<Vec<SearchResult>> {
    use futures::StreamExt;

    let stream = search_async(query, options).await?;

    // Collect all results, failing on first error
    stream.collect::<Vec<_>>().await
        .into_iter()
        .collect()
}
```

#### Acceptance Criteria

**Function Signature:**
- [ ] Returns `Result<Vec<SearchResult>>`
- [ ] Takes `&str` query and owned `SearchOptions`
- [ ] Feature-gated with `#[cfg(feature = "async")]`

**Behavior:**
- [ ] Collects all stream results
- [ ] First error causes `Err` return
- [ ] Empty result set returns `Ok(vec![])`

**Documentation:**
- [ ] Full rustdoc with examples
- [ ] Performance warning for large results
- [ ] Cross-reference to `search_async`

**Testing:**
- [ ] Basic collection works
- [ ] Error propagation works
- [ ] Empty results handled

---

### Story 22: Export Async API

**Estimated: 10 minutes | Dependencies: Stories 20-21**

#### Goal
Export async functions from lib.rs under feature gate, ensuring they're only compiled and visible when the `async` feature is enabled.

#### Location
`src/lib.rs`

#### Implementation

```rust
// =============================================================================
// Async API (feature-gated)
// =============================================================================

/// Async search API module (requires `async` feature)
#[cfg(feature = "async")]
mod async_api;

/// Re-export async functions at crate root
#[cfg(feature = "async")]
pub use async_api::{search_async, search_all_async};
```

#### Integration with Existing Exports

Full lib.rs export section:

```rust
// =============================================================================
// Library API - Sync
// =============================================================================

// Types
pub use search_types::{Match, SearchOptions, SearchResult, SearchResultIterator};

// Functions
pub use api::{search, search_iter};

// Re-exports
pub use predicates::code_aware::SqlDialect;

// =============================================================================
// Library API - Async (feature-gated)
// =============================================================================

#[cfg(feature = "async")]
mod async_api;

#[cfg(feature = "async")]
pub use async_api::{search_async, search_all_async};

// =============================================================================
// CLI API (unchanged)
// =============================================================================

pub use args::SearchArgs;
// ... other CLI exports
```

#### Verification Test

```rust
// In tests/api_exports.rs

#[test]
fn test_sync_exports() {
    // Always available
    use rdump::{search, search_iter, SearchOptions, SearchResult, Match};
    let _ = SearchOptions::default();
}

#[test]
#[cfg(feature = "async")]
fn test_async_exports() {
    // Only with async feature
    use rdump::{search_async, search_all_async};
    // Can't easily test without runtime, just verify import works
}

#[test]
fn test_async_not_visible_without_feature() {
    // This would fail to compile if async exports leaked
    // We can't test this directly, but cargo build without
    // the feature should succeed
}
```

#### Acceptance Criteria

**Feature Gating:**
- [ ] `search_async` only exported with `async` feature
- [ ] `search_all_async` only exported with `async` feature
- [ ] `async_api` module only compiled with `async` feature
- [ ] No async types leak without feature

**Build Verification:**
- [ ] `cargo build` succeeds without async imports
- [ ] `cargo build --features async` succeeds with async imports
- [ ] `use rdump::search_async` fails without feature
- [ ] `use rdump::search_async` succeeds with feature

**Documentation:**
- [ ] `cargo doc` shows async functions only when feature enabled
- [ ] Feature requirement noted in function docs

#### Technical Notes

**Why feature gate at module level?**

```rust
// Good - module not compiled without feature
#[cfg(feature = "async")]
mod async_api;

// Bad - module always compiled, just not exported
mod async_api;
#[cfg(feature = "async")]
pub use async_api::*;
```

The first approach:
1. Faster compilation without feature
2. No dead code warnings
3. Dependencies not linked

---

### Story 23: Write Async Integration Tests

**Estimated: 30 minutes | Dependencies: Story 22**

#### Goal
Write comprehensive integration tests for the async API, covering basic functionality, error handling, and edge cases like early stream termination.

#### Location
`tests/async_api.rs`

#### Implementation

```rust
//! Integration tests for async API
//!
//! Run with: cargo test --features async --test async_api

#![cfg(feature = "async")]

use anyhow::Result;
use futures::StreamExt;
use rdump::{search_async, search_all_async, SearchOptions};
use std::fs;
use tempfile::tempdir;

// =============================================================================
// Basic Functionality Tests
// =============================================================================

#[tokio::test]
async fn test_search_async_basic() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("test.rs"), "fn main() {}")?;
    fs::write(dir.path().join("lib.rs"), "pub fn add() {}")?;

    let mut stream = search_async("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).await?;

    let mut count = 0;
    while let Some(result) = stream.next().await {
        let result = result?;
        assert!(result.path.extension().unwrap() == "rs");
        count += 1;
    }

    assert_eq!(count, 2);
    Ok(())
}

#[tokio::test]
async fn test_search_all_async_basic() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("test.rs"), "fn main() {}")?;
    fs::write(dir.path().join("lib.rs"), "pub fn add() {}")?;

    let results = search_all_async("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).await?;

    assert_eq!(results.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_search_async_empty_results() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("test.rs"), "fn main() {}")?;

    let mut stream = search_async("ext:py", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).await?;

    assert!(stream.next().await.is_none());
    Ok(())
}

#[tokio::test]
async fn test_search_all_async_empty_results() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("test.rs"), "fn main() {}")?;

    let results = search_all_async("ext:py", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).await?;

    assert!(results.is_empty());
    Ok(())
}

// =============================================================================
// Stream Behavior Tests
// =============================================================================

#[tokio::test]
async fn test_search_async_early_termination() -> Result<()> {
    let dir = tempdir()?;

    // Create many files
    for i in 0..50 {
        fs::write(dir.path().join(format!("file{}.rs", i)), "fn main() {}")?;
    }

    let stream = search_async("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).await?;

    // Take only first 5
    let first_five: Vec<_> = stream
        .take(5)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(first_five.len(), 5);
    Ok(())
}

#[tokio::test]
async fn test_search_async_drop_stream() -> Result<()> {
    let dir = tempdir()?;

    for i in 0..100 {
        fs::write(dir.path().join(format!("file{}.rs", i)), "fn main() {}")?;
    }

    {
        let mut stream = search_async("ext:rs", SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        }).await?;

        // Read just one
        let _ = stream.next().await;

        // Drop stream - producer should stop
    }

    // Should complete without hanging
    Ok(())
}

#[tokio::test]
async fn test_search_async_collect_pattern() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("test.rs"), "fn main() {}")?;

    // Common pattern: collect stream into vec
    let results: Vec<_> = search_async("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    })
    .await?
    .collect()
    .await;

    // Results are Result<SearchResult>
    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());
    Ok(())
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_search_async_invalid_query() {
    let dir = tempdir().unwrap();

    let result = search_async("invalid((syntax", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).await;

    // Should return error immediately (before stream)
    assert!(result.is_err() || {
        // Or first item is error
        let mut stream = result.unwrap();
        matches!(
            futures::executor::block_on(stream.next()),
            Some(Err(_))
        )
    });
}

#[tokio::test]
async fn test_search_async_nonexistent_root() {
    let result = search_async("ext:rs", SearchOptions {
        root: std::path::PathBuf::from("/nonexistent/path"),
        ..Default::default()
    }).await;

    // Error should propagate
    assert!(result.is_err() || {
        let mut stream = result.unwrap();
        matches!(
            futures::executor::block_on(stream.next()),
            Some(Err(_)) | None
        )
    });
}

#[tokio::test]
async fn test_search_async_skip_errors() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("good.rs"), "fn main() {}")?;

    let stream = search_async("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).await?;

    // Skip errors pattern
    let results: Vec<_> = stream
        .filter_map(|r| async { r.ok() })
        .collect()
        .await;

    assert_eq!(results.len(), 1);
    Ok(())
}

// =============================================================================
// Concurrent Usage Tests
// =============================================================================

#[tokio::test]
async fn test_multiple_concurrent_searches() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("test.rs"), "fn main() {}")?;
    fs::write(dir.path().join("test.py"), "def main(): pass")?;

    let options1 = SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    };
    let options2 = options1.clone();

    // Run two searches concurrently
    let (results1, results2) = tokio::join!(
        search_all_async("ext:rs", options1),
        search_all_async("ext:py", options2),
    );

    assert_eq!(results1?.len(), 1);
    assert_eq!(results2?.len(), 1);
    Ok(())
}

#[tokio::test]
async fn test_search_in_spawn() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("test.rs"), "fn main() {}")?;

    let root = dir.path().to_path_buf();

    let handle = tokio::spawn(async move {
        search_all_async("ext:rs", SearchOptions {
            root,
            ..Default::default()
        }).await
    });

    let results = handle.await??;
    assert_eq!(results.len(), 1);
    Ok(())
}

// =============================================================================
// Feature Tests
// =============================================================================

#[tokio::test]
async fn test_search_async_with_function_predicate() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("test.rs"), "fn main() {}\nfn helper() {}")?;

    let results = search_all_async("func:main", SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    }).await?;

    assert_eq!(results.len(), 1);
    assert!(!results[0].is_whole_file_match());
    assert_eq!(results[0].matches.len(), 1);
    Ok(())
}

#[tokio::test]
async fn test_search_async_with_options() -> Result<()> {
    let dir = tempdir()?;

    // Create nested structure
    let sub = dir.path().join("sub");
    fs::create_dir(&sub)?;
    fs::write(dir.path().join("root.rs"), "fn main() {}")?;
    fs::write(sub.join("nested.rs"), "fn main() {}")?;

    // With max_depth: 1
    let results = search_all_async("ext:rs", SearchOptions {
        root: dir.path().to_path_buf(),
        max_depth: Some(1),
        ..Default::default()
    }).await?;

    assert_eq!(results.len(), 1);
    assert!(results[0].path.file_name().unwrap() == "root.rs");
    Ok(())
}
```

#### Acceptance Criteria

**Test Coverage:**
- [ ] `test_search_async_basic` - passes
- [ ] `test_search_all_async_basic` - passes
- [ ] `test_search_async_empty_results` - passes
- [ ] `test_search_all_async_empty_results` - passes
- [ ] `test_search_async_early_termination` - passes
- [ ] `test_search_async_drop_stream` - passes
- [ ] `test_search_async_collect_pattern` - passes
- [ ] `test_search_async_invalid_query` - passes
- [ ] `test_search_async_nonexistent_root` - passes
- [ ] `test_search_async_skip_errors` - passes
- [ ] `test_multiple_concurrent_searches` - passes
- [ ] `test_search_in_spawn` - passes
- [ ] `test_search_async_with_function_predicate` - passes
- [ ] `test_search_async_with_options` - passes

**Test Quality:**
- [ ] Uses `#[tokio::test]` attribute
- [ ] All tests use tempdir for isolation
- [ ] Tests are independent and can run in parallel
- [ ] Error cases covered

**Running Tests:**
```bash
cargo test --features async --test async_api
```

#### Technical Notes

**Why `#[tokio::test]`?**

The async tests need a runtime. `#[tokio::test]` macro:
1. Creates a new runtime for each test
2. Runs test as async
3. Handles panics correctly

**Testing early termination:**

We can't directly test that the producer stops, but we can verify:
1. Taking fewer items works
2. Dropping stream doesn't hang
3. Memory doesn't grow unboundedly

---

### Story 24: Create Async Example Program

**Estimated: 20 minutes | Dependencies: Story 22**

#### Goal
Create a comprehensive example demonstrating async API usage patterns including stream processing, concurrent searches, and integration with tokio applications.

#### Location
`examples/async_search.rs`

#### Implementation

```rust
//! Async search example for rdump library
//!
//! Demonstrates async API patterns for tokio-based applications.
//!
//! Run with: cargo run --features async --example async_search

use anyhow::Result;
use futures::StreamExt;
use rdump::{search_async, search_all_async, SearchOptions, SearchResult};
use std::path::PathBuf;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("rdump Async API Examples\n");

    // Example 1: Basic async streaming
    example_basic_streaming().await?;

    // Example 2: Collect all results
    example_collect_all().await?;

    // Example 3: Early termination
    example_early_termination().await?;

    // Example 4: Concurrent searches
    example_concurrent_searches().await?;

    // Example 5: Processing with select
    example_with_timeout().await?;

    // Example 6: Aggregation
    example_aggregation().await?;

    println!("\nAll async examples completed!");
    Ok(())
}

/// Example 1: Basic async streaming
async fn example_basic_streaming() -> Result<()> {
    println!("=== Example 1: Basic Streaming ===");

    let mut stream = search_async("ext:rs", SearchOptions::default()).await?;

    let mut count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(r) => {
                count += 1;
                if count <= 3 {
                    println!("  {} ({} bytes)", r.path.display(), r.content.len());
                }
            }
            Err(e) => eprintln!("  Error: {}", e),
        }
    }

    if count > 3 {
        println!("  ... and {} more files", count - 3);
    }

    println!();
    Ok(())
}

/// Example 2: Collect all results at once
async fn example_collect_all() -> Result<()> {
    println!("=== Example 2: Collect All ===");

    let start = Instant::now();

    let results = search_all_async("ext:rs", SearchOptions::default()).await?;

    println!("Found {} files in {:?}", results.len(), start.elapsed());

    for result in results.iter().take(3) {
        println!("  - {}", result.path.display());
    }

    println!();
    Ok(())
}

/// Example 3: Early termination
async fn example_early_termination() -> Result<()> {
    println!("=== Example 3: Early Termination ===");

    let stream = search_async("ext:rs", SearchOptions::default()).await?;

    // Take only first 5 results
    let first_five: Vec<SearchResult> = stream
        .take(5)
        .filter_map(|r| async { r.ok() })
        .collect()
        .await;

    println!("Took first {} results:", first_five.len());
    for result in &first_five {
        println!("  - {}", result.path.display());
    }

    println!();
    Ok(())
}

/// Example 4: Run multiple searches concurrently
async fn example_concurrent_searches() -> Result<()> {
    println!("=== Example 4: Concurrent Searches ===");

    let start = Instant::now();

    // Clone options for each search
    let opts = SearchOptions::default();

    // Run searches concurrently
    let (rust_results, python_results, js_results) = tokio::join!(
        search_all_async("ext:rs", opts.clone()),
        search_all_async("ext:py", opts.clone()),
        search_all_async("ext:js", opts.clone()),
    );

    println!("Completed in {:?}", start.elapsed());
    println!("  Rust files:   {}", rust_results?.len());
    println!("  Python files: {}", python_results?.len());
    println!("  JS files:     {}", js_results?.len());

    println!();
    Ok(())
}

/// Example 5: Processing with timeout using select
async fn example_with_timeout() -> Result<()> {
    println!("=== Example 5: With Timeout ===");

    let mut stream = search_async("ext:rs", SearchOptions::default()).await?;
    let timeout = tokio::time::sleep(std::time::Duration::from_millis(100));
    tokio::pin!(timeout);

    let mut count = 0;

    loop {
        tokio::select! {
            // Process next result
            result = stream.next() => {
                match result {
                    Some(Ok(_)) => count += 1,
                    Some(Err(e)) => eprintln!("Error: {}", e),
                    None => break,
                }
            }
            // Timeout
            _ = &mut timeout => {
                println!("Timeout! Processed {} files", count);
                break;
            }
        }
    }

    println!("Processed {} files before timeout/completion", count);
    println!();
    Ok(())
}

/// Example 6: Async aggregation with streaming
async fn example_aggregation() -> Result<()> {
    println!("=== Example 6: Aggregation ===");

    let stream = search_async("func:*", SearchOptions {
        presets: vec!["rust".to_string()],
        ..Default::default()
    }).await?;

    // Aggregate without collecting all results
    let mut file_count = 0;
    let mut total_functions = 0;
    let mut total_bytes = 0;

    tokio::pin!(stream);

    while let Some(result) = stream.next().await {
        if let Ok(r) = result {
            file_count += 1;
            total_functions += r.match_count();
            total_bytes += r.content.len();
        }
    }

    println!("Statistics:");
    println!("  Files:     {}", file_count);
    println!("  Functions: {}", total_functions);
    println!("  Bytes:     {} KB", total_bytes / 1024);

    println!();
    Ok(())
}

// =============================================================================
// Additional async patterns
// =============================================================================

/// Pattern: Fan-out search to multiple directories
#[allow(dead_code)]
async fn search_multiple_directories(dirs: Vec<PathBuf>) -> Result<Vec<SearchResult>> {
    let handles: Vec<_> = dirs
        .into_iter()
        .map(|dir| {
            tokio::spawn(async move {
                search_all_async("ext:rs", SearchOptions {
                    root: dir,
                    ..Default::default()
                }).await
            })
        })
        .collect();

    let mut all_results = Vec::new();
    for handle in handles {
        let results = handle.await??;
        all_results.extend(results);
    }

    Ok(all_results)
}

/// Pattern: Process results with bounded concurrency
#[allow(dead_code)]
async fn process_with_concurrency_limit() -> Result<()> {
    use futures::stream::StreamExt;

    let stream = search_async("ext:rs", SearchOptions::default()).await?;

    // Process up to 10 files concurrently
    stream
        .map(|result| async move {
            if let Ok(r) = result {
                process_file(&r).await;
            }
        })
        .buffer_unordered(10)  // Max 10 concurrent
        .collect::<Vec<_>>()
        .await;

    Ok(())
}

async fn process_file(result: &SearchResult) {
    // Simulate async processing
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    println!("Processed: {}", result.path.display());
}

/// Pattern: Cancellable search with channel
#[allow(dead_code)]
async fn cancellable_search() -> Result<()> {
    use tokio::sync::oneshot;

    let (cancel_tx, cancel_rx) = oneshot::channel();
    let mut stream = search_async("ext:rs", SearchOptions::default()).await?;

    let search_task = tokio::spawn(async move {
        tokio::select! {
            _ = async {
                while let Some(result) = stream.next().await {
                    if let Ok(r) = result {
                        println!("{}", r.path.display());
                    }
                }
            } => {}
            _ = cancel_rx => {
                println!("Search cancelled!");
            }
        }
    });

    // Cancel after 50ms
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let _ = cancel_tx.send(());

    search_task.await?;
    Ok(())
}

/// Pattern: Stream to async writer
#[allow(dead_code)]
async fn stream_to_file() -> Result<()> {
    use tokio::io::AsyncWriteExt;

    let mut file = tokio::fs::File::create("results.txt").await?;
    let stream = search_async("ext:rs", SearchOptions::default()).await?;

    tokio::pin!(stream);

    while let Some(result) = stream.next().await {
        if let Ok(r) = result {
            file.write_all(format!("{}\n", r.path.display()).as_bytes()).await?;
        }
    }

    file.flush().await?;
    Ok(())
}
```

#### Acceptance Criteria

**Compilation:**
- [ ] Example compiles with `--features async`
- [ ] Example runs successfully
- [ ] Works with `cargo run --features async --example async_search`

**Content:**
- [ ] Basic streaming pattern shown
- [ ] `search_all_async` usage shown
- [ ] Early termination with `.take()` shown
- [ ] Concurrent searches with `tokio::join!` shown
- [ ] Timeout with `tokio::select!` shown
- [ ] Aggregation pattern shown

**Code Quality:**
- [ ] Clear comments explaining patterns
- [ ] Shows practical real-world use cases
- [ ] Demonstrates async-specific patterns (select, join, spawn)
- [ ] Additional patterns for reference

**Output:**
- [ ] Produces clear, formatted output
- [ ] Shows timing information
- [ ] Handles edge cases

#### Technical Notes

**Running the example:**

```bash
# Basic run
cargo run --features async --example async_search

# In a specific directory
cd /path/to/project
cargo run --features async --example async_search --manifest-path /path/to/rdump/Cargo.toml
```

**Why these patterns?**

1. **Basic streaming**: Foundation for all async usage
2. **Collect all**: Simple case for small results
3. **Early termination**: Memory efficiency
4. **Concurrent**: Performance for multiple searches
5. **Timeout**: Integration with other async operations
6. **Aggregation**: Real-world analytics use case

**Dependencies for examples:**

```toml
[dev-dependencies]
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

---

## Compatibility Requirements

- [x] Existing CLI APIs remain unchanged
- [x] Performance impact is minimal
- [x] Backward compatible - existing `perform_search()` still works

---

## Technical Specifications Summary

### New Public Types

| Type | Purpose |
|------|---------|
| `SearchOptions` | Library-friendly search config |
| `SearchResult` | File with matches |
| `Match` | Single match location + text |
| `SearchResultIterator` | Streaming iterator |

### Public Functions

| Function | Use Case |
|----------|----------|
| `search_iter()` | Large codebases (streaming) |
| `search()` | Small result sets |
| `search_async()` | Tokio applications |
| `search_all_async()` | Async convenience |

---

## Risk Mitigation

### Primary Risk: Breaking CLI Behavior
- Incremental refactoring with tests at each step
- All existing CLI tests must pass before merge

### Memory Usage
- Streaming-first design
- `search()` documented for small sets only

---

## Definition of Done

- [ ] All 24 stories completed with acceptance criteria met
- [ ] Existing CLI tests pass unchanged
- [ ] Documentation complete
- [ ] CI passes on all platforms

---

## Effort Summary

| Phase | Stories | Total Time |
|-------|---------|------------|
| Core Types | 1-3 | 1.5 hours |
| Internal Refactor | 4-5 | 1.5 hours |
| Iterator | 6-8 | 2 hours |
| Public API | 9-12 | 1.5 hours |
| Testing | 13-14 | 1.75 hours |
| Examples | 15-16 | 1 hour |
| Documentation | 17-18 | 1.25 hours |
| Async | 19-24 | 2.5 hours |
| **Total** | **24 stories** | **~13 hours** |

---

## References

- **Analysis Document:** `docs/analysis/library-api.md`
- **Current Search Implementation:** `src/commands/search.rs`
- **Library Entry Point:** `src/lib.rs`
