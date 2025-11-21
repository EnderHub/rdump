# Library API Analysis

## Problem Statement

rdump is currently limited to CLI usage only. Users cannot embed search functionality in other Rust programs. This prevents:

- IDE plugin development
- Custom tooling built on top of rdump
- CI/CD pipeline integration
- Test harness usage
- Programmatic access to search results

### Current Limitations

1. **`run_search()` prints to stdout** - No way to capture results programmatically
2. **No structured return type** - Results are `Vec<(PathBuf, Vec<Range>)>` where `Range` is a tree-sitter type
3. **`SearchArgs` is CLI-coupled** - Contains formatting options (color, output path, format) that don't belong in a library API
4. **No content access** - Results only contain paths and byte ranges, not actual matched content

## Current Architecture Analysis

### Data Flow

```
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

| Component | Location | Purpose |
|-----------|----------|---------|
| `SearchArgs` | lib.rs:196-277 | CLI argument struct |
| `run_search()` | commands/search.rs:21-73 | Orchestrates search and output |
| `perform_search()` | commands/search.rs:77-221 | Core search logic |
| `Evaluator` | evaluator.rs:127-138 | Query evaluation engine |
| `MatchResult` | evaluator.rs:11-18 | Evaluation result (Boolean/Hunks) |
| `FileContext` | evaluator.rs:21-34 | Per-file evaluation context |
| `print_output()` | formatter.rs:237-264 | Output formatting dispatcher |

### Current SearchArgs Structure

```rust
pub struct SearchArgs {
    // Core search parameters (needed for library)
    pub query: Option<String>,
    pub dialect: Option<SqlDialectFlag>,
    pub preset: Vec<String>,
    pub root: PathBuf,
    pub no_ignore: bool,
    pub hidden: bool,
    pub max_depth: Option<usize>,

    // CLI-only parameters (NOT needed for library)
    pub output: Option<PathBuf>,      // Write to file
    pub line_numbers: bool,           // Display option
    pub no_headers: bool,             // Display option
    pub format: Format,               // Output format
    pub color: ColorChoice,           // ANSI colors
    pub context: Option<usize>,       // Context lines
    pub find: bool,                   // Format alias
}
```

### Current Return Type

`perform_search()` returns `Vec<(PathBuf, Vec<Range>)>`:
- `PathBuf` - File path
- `Vec<Range>` - tree-sitter ranges (byte offsets + row/column positions)
  - Empty vector = boolean match (whole file)
  - Non-empty = specific matching code blocks ("hunks")

This is insufficient for library consumers who need:
- Actual matched text content
- Line numbers
- Full file content (for context)
- Metadata about matches

## Proposed Solution

### New Public Types

#### 1. SearchOptions (replaces CLI concerns)

```rust
/// Options for performing a search (library-friendly)
pub struct SearchOptions {
    /// Root directory to search
    pub root: PathBuf,
    /// Named presets to apply
    pub presets: Vec<String>,
    /// Ignore .gitignore rules
    pub no_ignore: bool,
    /// Include hidden files
    pub hidden: bool,
    /// Maximum directory depth
    pub max_depth: Option<usize>,
    /// SQL dialect override for .sql files
    pub sql_dialect: Option<SqlDialect>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
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

#### 2. SearchResult (rich result type)

```rust
/// A single file that matched the search query
pub struct SearchResult {
    /// Path to the matched file
    pub path: PathBuf,
    /// The matches within this file
    pub matches: Vec<Match>,
    /// Full file content (lazily loaded or optional)
    pub content: String,
}

/// A single match within a file
pub struct Match {
    /// Start line (1-indexed)
    pub start_line: usize,
    /// End line (1-indexed, inclusive)
    pub end_line: usize,
    /// Start column (0-indexed)
    pub start_column: usize,
    /// End column (0-indexed)
    pub end_column: usize,
    /// Byte range in file
    pub byte_range: std::ops::Range<usize>,
    /// The matched text
    pub text: String,
}

impl SearchResult {
    /// Returns true if this is a whole-file match (no specific hunks)
    pub fn is_whole_file_match(&self) -> bool {
        self.matches.is_empty()
    }

    /// Get all matched line numbers
    pub fn matched_lines(&self) -> Vec<usize> {
        self.matches
            .iter()
            .flat_map(|m| m.start_line..=m.end_line)
            .collect()
    }
}
```

#### 3. Main API Functions (Streaming-First Design)

For large repositories, streaming is essential. The primary API returns an iterator:

```rust
/// Search for files matching a query (streaming, memory-efficient)
///
/// Returns an iterator that yields results one at a time, loading file
/// content only when each result is consumed. This is the recommended
/// API for large codebases.
///
/// # Example
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
///     println!("{}: {} matches", result.path.display(), result.matches.len());
/// }
/// ```
pub fn search_iter(
    query: &str,
    options: SearchOptions,
) -> Result<impl Iterator<Item = Result<SearchResult>>> {
    // Implementation - yields results lazily
}

/// Search for files matching a query (convenience wrapper)
///
/// Collects all results into a Vec. Use `search_iter` for large codebases
/// to avoid loading all content into memory at once.
///
/// # Example
/// ```rust
/// use rdump::{search, SearchOptions};
///
/// let results = search("ext:rs & func:main", SearchOptions::default())?;
/// println!("Found {} files", results.len());
/// ```
pub fn search(query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
    search_iter(query, options)?.collect()
}
```

### Why Streaming-First?

1. **Memory efficiency** - Only one file's content in memory at a time
2. **Early termination** - Can stop after finding N results without processing all
3. **Pipeline-friendly** - Compose with other iterators (filter, map, take)
4. **Large repo support** - Essential for monorepos with 10K+ files

### Lazy Content Loading Option

For even more efficiency, content can be loaded on-demand:

```rust
pub struct SearchResult {
    pub path: PathBuf,
    pub matches: Vec<Match>,
    content: OnceCell<String>,  // Lazy-loaded
}

impl SearchResult {
    /// Get file content (loads from disk on first access)
    pub fn content(&self) -> Result<&str> {
        self.content.get_or_try_init(|| {
            std::fs::read_to_string(&self.path)
        }).map(|s| s.as_str())
    }

    /// Check if content is already loaded
    pub fn is_content_loaded(&self) -> bool {
        self.content.get().is_some()
    }
}
```

## CLI Compatibility

**The CLI interface remains completely unchanged.** Users will see no difference in commands, flags, or output. This is purely an internal refactoring that enables library usage.

### Architecture Change

**Before (CLI-only):**
```
rdump search "ext:rs"
    ↓
SearchArgs (CLI struct with format, color, output, etc.)
    ↓
run_search()
    ↓
perform_search() → Vec<(PathBuf, Vec<Range>)>
    ↓
formatter::print_output() → stdout
```

**After (CLI + Library):**
```
┌─────────────────────────────────────────────────────────┐
│ CLI Path                                                │
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
│ Library Path                                            │
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

### What Changes Internally

| Component | Change |
|-----------|--------|
| `SearchArgs` | **No change** - stays as CLI argument struct |
| `run_search()` | Converts `SearchArgs` → `SearchOptions`, calls internal function |
| `perform_search()` | Becomes thin wrapper around `perform_search_internal()` |
| `perform_search_internal()` | **New** - core logic accepting `SearchOptions` |
| `search_iter()` | **New** - public library API returning iterator |
| `search()` | **New** - convenience wrapper collecting results |

### Code Example

```rust
// In commands/search.rs

/// CLI entry point (unchanged behavior)
pub fn run_search(mut args: SearchArgs) -> Result<()> {
    // Handle shorthand flags (unchanged)
    if args.no_headers {
        args.format = Format::Cat;
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

    // Use internal search function
    let matching_files = perform_search_internal(query, &options)?;

    // Format and output (unchanged)
    formatter::print_output(...)?;

    Ok(())
}

/// Existing function for backward compatibility
pub fn perform_search(args: &SearchArgs) -> Result<Vec<(PathBuf, Vec<Range>)>> {
    let options = SearchOptions::from(args);
    perform_search_internal(args.query.as_deref().unwrap_or(""), &options)
}
```

### Why This Approach?

1. **Zero breaking changes** - Existing CLI behavior is preserved
2. **Shared code** - Both CLI and library use `perform_search_internal()`
3. **Clear separation** - CLI concerns (color, format) stay in `SearchArgs`
4. **Testability** - Library API can be tested without CLI overhead
5. **Future flexibility** - Can evolve library API without affecting CLI

## Implementation Plan

### Phase 1: Core Types and Refactoring

**Estimated effort: 3-4 hours**

1. Create `SearchOptions` struct in `lib.rs`
2. Create `SearchResult` and `Match` structs in new `lib/types.rs` or in `lib.rs`
3. Refactor `perform_search()` to accept `SearchOptions` instead of `SearchArgs`
4. Add `SearchArgs::to_search_options()` method for CLI compatibility

### Phase 2: Streaming Iterator Implementation

**Estimated effort: 5-6 hours**

1. Create `SearchResultIterator` struct that wraps the internal search pipeline
2. Implement `Iterator<Item = Result<SearchResult>>` trait
3. Lazy content loading - only read file when result is consumed
4. Convert tree-sitter `Range` to user-friendly `Match` struct
5. Handle edge cases (empty hunks = whole file, Unicode, etc.)
6. Implement `search()` convenience wrapper using `.collect()`

### Phase 3: Public API and Exports

**Estimated effort: 2-3 hours**

1. Create `search_iter()` function that returns the streaming iterator
2. Create `search()` convenience function that collects results
3. Export all public types from `lib.rs`
4. Update `run_search()` to use new internal functions
5. Add `OnceCell` for lazy content loading in `SearchResult`

### Phase 4: Documentation and Testing

**Estimated effort: 3-4 hours**

1. Add comprehensive rustdoc for all public types and functions
2. Create integration tests for library API
3. Add examples in `examples/` directory
4. Update README with library usage section

### Phase 5: Async Support

**Estimated effort: 2-3 hours** (Level 1 implementation)

1. Add `async` feature flag with tokio dependencies
2. Create `async_api.rs` module (feature-gated)
3. Implement `search_async()` using spawn_blocking + channel
4. Export async API from `lib.rs`
5. Add async examples and tests

See [Async Implementation Details](#async-implementation-details) for full technical breakdown.

## Detailed Implementation Steps

### Step 1: Create SearchOptions

Location: `rdump/src/lib.rs`

```rust
/// Re-export SqlDialect for library users
pub use predicates::code_aware::SqlDialect;

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub root: PathBuf,
    pub presets: Vec<String>,
    pub no_ignore: bool,
    pub hidden: bool,
    pub max_depth: Option<usize>,
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

### Step 2: Create Result Types

Location: `rdump/src/lib.rs` or new file

```rust
/// A match within a file
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub byte_range: std::ops::Range<usize>,
    pub text: String,
}

/// A file that matched the search query
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub matches: Vec<Match>,
    pub content: String,
}
```

### Step 3: Refactor perform_search

Location: `rdump/src/commands/search.rs`

Create internal function that accepts `SearchOptions`:

```rust
/// Internal search implementation
pub(crate) fn perform_search_internal(
    query: &str,
    options: &SearchOptions,
) -> Result<Vec<(PathBuf, Vec<Range>)>> {
    // Move current perform_search logic here
    // but use SearchOptions instead of SearchArgs
}

/// Existing perform_search for CLI compatibility
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

### Step 4: Create Public search() Function

Location: `rdump/src/lib.rs`

```rust
/// Search for files matching an RQL query
pub fn search(query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
    use commands::search::perform_search_internal;

    let raw_results = perform_search_internal(query, &options)?;

    let mut results = Vec::with_capacity(raw_results.len());
    for (path, ranges) in raw_results {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let matches = ranges_to_matches(&content, &ranges);

        results.push(SearchResult {
            path,
            matches,
            content,
        });
    }

    Ok(results)
}

fn ranges_to_matches(content: &str, ranges: &[tree_sitter::Range]) -> Vec<Match> {
    ranges
        .iter()
        .map(|r| {
            let text = content
                .get(r.start_byte..r.end_byte)
                .unwrap_or("")
                .to_string();

            Match {
                start_line: r.start_point.row + 1,
                end_line: r.end_point.row + 1,
                start_column: r.start_point.column,
                end_column: r.end_point.column,
                byte_range: r.start_byte..r.end_byte,
                text,
            }
        })
        .collect()
}
```

## Effort Estimation Summary

| Phase | Description | Estimated Hours |
|-------|-------------|-----------------|
| Phase 1 | Core types and refactoring | 3-4 hours |
| Phase 2 | Streaming iterator implementation | 5-6 hours |
| Phase 3 | Public API and exports | 2-3 hours |
| Phase 4 | Documentation and testing | 3-4 hours |
| Phase 5 | Async support | 2-3 hours |
| **Total** | Phases 1-5 | **15-20 hours** |

## Risks and Considerations

### 1. Breaking Changes

**Risk:** Modifying internal functions could break CLI behavior.

**Mitigation:**
- Keep existing `SearchArgs` and `run_search()` unchanged externally
- Refactor incrementally with comprehensive test coverage
- Use feature flags if needed for gradual rollout

### 2. Memory Usage

**Risk:** Loading all file contents into memory could be problematic for large result sets.

**Mitigation (built into design):**
- Primary API is streaming (`search_iter()`) - only one file in memory at a time
- `SearchResult.content` uses `OnceCell` for lazy loading
- Convenience `search()` wrapper clearly documented for small result sets
- Users can use `.take(n)` to limit results
- Consider `SearchOptions::max_results` as additional safeguard

### 3. API Stability

**Risk:** Exposing internal types (like tree-sitter's `Range`) could lock in implementation details.

**Mitigation:**
- Create wrapper types (`Match`) that hide implementation
- Only expose stable, user-friendly types
- Mark experimental APIs with `#[doc(hidden)]` initially

### 4. Thread Safety

**Risk:** Library users may want to call `search()` from multiple threads.

**Mitigation:**
- Ensure `SearchOptions` is `Send + Sync`
- `SearchResult` should be fully owned data (no references)
- Document thread safety guarantees

### 5. Error Handling

**Risk:** CLI errors are printed to stderr; library should return errors.

**Mitigation:**
- Create proper error types (`SearchError` enum)
- Remove `eprintln!` calls from library paths
- Return `Result<T, SearchError>` from public API

### 6. Performance

**Risk:** Converting results and loading content adds overhead.

**Mitigation:**
- Benchmark before/after
- Keep internal `perform_search_internal()` for hot paths
- Make content loading lazy/optional
- Use parallel iteration for result transformation

## Testing Strategy

### Unit Tests

1. `SearchOptions::default()` correctness
2. `ranges_to_matches()` conversion
3. `SearchResult::is_whole_file_match()`
4. Edge cases (empty results, Unicode, large files)

### Integration Tests

1. Basic search with different predicates
2. Preset application
3. Directory filtering (hidden, no_ignore)
4. SQL dialect handling
5. Error cases (invalid query, missing root)

### Example Tests

```rust
#[test]
fn test_library_api_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let results = search(
        "ext:rs & func:main",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    ).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, file);
    assert!(!results[0].matches.is_empty());
}

#[test]
fn test_library_api_whole_file_match() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    ).unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].is_whole_file_match());
}
```

## Async Implementation Details

Both sync and async APIs would coexist, with async behind a feature flag:

```rust
// Sync API (primary, no runtime needed)
use rdump::{search_iter, search, SearchOptions};

let results = search_iter("ext:rs", SearchOptions::default())?;
for result in results { ... }

// Async API (for tokio users)
use rdump::{search_async, SearchOptions};

let mut stream = search_async("ext:rs", SearchOptions::default()).await?;
while let Some(result) = stream.next().await { ... }
```

#### Feature Flag Configuration

```toml
# Cargo.toml
[features]
default = []
async = ["tokio", "tokio-stream"]

[dependencies]
tokio = { version = "1", features = ["fs", "sync", "rt"], optional = true }
tokio-stream = { version = "0.1", optional = true }
```

```rust
// In lib.rs
#[cfg(feature = "async")]
mod async_api;

#[cfg(feature = "async")]
pub use async_api::{search_async, search_stream};
```

Users opt-in:
```toml
# User's Cargo.toml
rdump = { version = "0.1", features = ["async"] }
```

#### Where the Sync/Async Split Happens

The split occurs at **I/O boundaries**, not in the core logic:

```
perform_search_internal()
├── config::load_config()              ← File I/O
├── get_candidate_files()              ← Directory walk I/O     ← SPLIT POINT
├── parser::parse_query()              ← CPU (pure, stays sync)
├── validate_ast_predicates()          ← CPU (pure, stays sync)
├── Pre-filter pass (sequential)       ← File metadata I/O      ← SPLIT POINT
└── Main evaluation pass (par_iter)    ← File content I/O + CPU ← SPLIT POINT
```

**Shared core (always sync):**
- Query parsing
- AST validation
- Predicate registry
- `Evaluator::evaluate()` logic
- Result transformation

**I/O layer (sync or async):**
- Directory walking
- File reading
- Metadata access

#### Implementation: Level 1 (Spawn Blocking)

**Chosen approach:** Wrap the sync pipeline with `spawn_blocking` and stream results via channel.

```rust
#[cfg(feature = "async")]
pub async fn search_async(
    query: &str,
    options: SearchOptions,
) -> Result<impl Stream<Item = Result<SearchResult>>> {
    let query = query.to_string();
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::task::spawn_blocking(move || {
        // Entire sync pipeline runs here (keeps rayon parallelism)
        let results = search_iter(&query, options)?;
        for result in results {
            if tx.blocking_send(result).is_err() {
                break;  // Receiver dropped
            }
        }
        Ok::<_, anyhow::Error>(())
    });

    Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
}
```

#### Why Level 1?

| Benefit | Explanation |
|---------|-------------|
| **Keeps rayon** | Battle-tested parallelism for CPU-bound tree-sitter parsing |
| **Simple** | 1-2 hours to implement, minimal code to maintain |
| **Backpressure** | Bounded channel (100 items) prevents memory blowup |
| **Good enough** | rdump is CPU-bound; async I/O has diminishing returns on local disks |

#### Characteristics

- **Thread usage:** 1 blocking thread per search
- **Parallelism:** Full rayon parallelism preserved internally
- **Memory:** Results buffered in channel with backpressure
- **First result latency:** Slight delay until rayon batch starts yielding
- **Best for:** Local SSDs, occasional searches, simple integration

#### When to Reconsider

Level 1 may need upgrading to async I/O (Level 2/3) if:
- Profiling shows I/O is the bottleneck (not CPU)
- Running on network filesystems (S3, NFS) with high latency
- Need 100+ concurrent searches exhausting blocking thread pool

For now, Level 1 covers the majority of use cases with minimal complexity.

#### File Structure

```
src/
├── lib.rs
│   ├── search_iter()        ← Sync, primary
│   ├── search()             ← Sync convenience
│   └── #[cfg(feature = "async")]
│       └── search_async()   ← Async wrapper
├── commands/
│   └── search.rs
│       └── perform_search_internal()  ← Unchanged core
└── async_api.rs             ← Feature-gated async module
    └── search_async()
```

#### When to Use Each API

| API | Runtime | Use Case |
|-----|---------|----------|
| `search_iter()` | None | CLI tools, scripts, default choice |
| `search()` | None | Small result sets, convenience |
| `search_async()` | Tokio | Web services, IDE plugins, async apps |

## Conclusion

Implementing a library API for rdump is a moderate-effort task (15-20 hours for core functionality including async) that provides significant value for users wanting to integrate rdump into their tools and workflows.

The key design decisions are:
1. **Streaming-first** - `search_iter()` is the primary API for memory efficiency on large repos
2. **Dual sync/async APIs** - Feature-gated async support for tokio users
3. **Separate concerns** - `SearchOptions` (library) vs `SearchArgs` (CLI)
4. **Rich result types** - `SearchResult` and `Match` with all needed data
5. **Lazy content loading** - `OnceCell` defers file reads until content is accessed
6. **Backward compatibility** - Existing CLI continues to work unchanged

The streaming approach is essential for monorepos and large codebases where:
- Thousands of files may match
- Full content of all matches would exceed available memory
- Users want early termination (e.g., find first 10 results)
- Results need to be piped through additional processing

The implementation can be done incrementally, starting with the core types and streaming iterator, then async support, followed by optional advanced features (progress callbacks, builder pattern) based on user feedback.
