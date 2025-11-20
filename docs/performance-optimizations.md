# Performance Optimizations

This document outlines performance optimization opportunities for rdump, prioritized by impact and effort.

## Current Architecture

rdump uses a well-designed two-pass architecture:
1. **Metadata pre-filter** (single-threaded) - fast checks on extension, name, size, modified time
2. **Content evaluation** (Rayon parallel) - tree-sitter parsing, content matching

This is fundamentally sound. The optimizations below are targeted improvements, not architectural changes.

---

## Critical Optimizations

### 1. Pre-compile Tree-sitter Queries

**Location:** `src/predicates/code_aware/mod.rs:53`

**Problem:** `Query::new()` is called for every file evaluated. Query compilation is expensive - for 1000 files with 5 code-aware predicates, this means 5000 query compilations.

**Current code:**
```rust
let query = Query::new(&language, query_string)?;
```

**Solution:** Pre-compile queries at startup using `Lazy<Query>` in each profile module.

```rust
// In profiles/rust.rs
use once_cell::sync::Lazy;
use tree_sitter::Query;

static FUNCTION_QUERY: Lazy<Query> = Lazy::new(|| {
    Query::new(
        &tree_sitter_rust::language(),
        "(function_item name: (identifier) @name)"
    ).expect("Invalid query")
});

// In code_aware/mod.rs - use reference instead of creating new
pub fn get_query(profile: &str, predicate: &str) -> Option<&'static Query> {
    // Return pre-compiled query reference
}
```

**Impact:** 40-60% faster on code-aware searches
**Effort:** Medium (2-3 hours)

---

### 2. Parallel Directory Walking

**Location:** `src/commands/search.rs:256`

**Problem:** Uses sequential `WalkBuilder::build()` which doesn't utilize multiple cores during directory traversal.

**Current code:**
```rust
for entry in WalkBuilder::new(path).build() {
    // collect candidates
}
```

**Solution:** Use `WalkBuilder::build_parallel()` with thread-local accumulation.

```rust
use std::sync::Mutex;

let candidates = Mutex::new(Vec::new());

WalkBuilder::new(path)
    .build_parallel()
    .run(|| {
        let candidates = &candidates;
        Box::new(move |entry| {
            if let Ok(entry) = entry {
                if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    candidates.lock().unwrap().push(entry.into_path());
                }
            }
            ignore::WalkState::Continue
        })
    });

let candidates = candidates.into_inner().unwrap();
```

**Impact:** 30-50% faster on large directories (10k+ files)
**Effort:** Medium (1-2 hours)

---

### 3. Remove Unnecessary Content Clone

**Location:** `src/predicates/code_aware/mod.rs:39`

**Problem:** Clones entire file content string unnecessarily.

**Current code:**
```rust
let source = context.content()?.to_string();
```

**Solution:** Use the `&str` reference directly.

```rust
let source = context.content()?;
// tree_sitter::Parser::parse() accepts &str
```

**Impact:** 20-30% reduction in allocations
**Effort:** Low (5 minutes)

---

### 4. Automatic Query Predicate Reordering

**Location:** `src/parser.rs` or new `src/optimizer.rs`

**Problem:** Query performance depends on predicate order, but users write queries in logical order, not optimal order. For example:

```bash
# User writes (logical: "find functions named main in Rust files")
rdump "func:main & ext:rs"

# But optimal is (filter 99% of files first, then parse remaining)
rdump "ext:rs & func:main"
```

The semantic predicate `func:main` costs 500-2000 units (parse + tree query), while `ext:rs` costs 1-5 units (string comparison). Without optimization, the user's query parses ALL files, then filters by extension.

**Solution:** Reorder predicates by cost tier before evaluation.

```rust
// src/optimizer.rs
use crate::ast::{Expr, Predicate};

/// Predicate cost tiers (lower = cheaper = evaluate first)
fn predicate_cost(pred: &Predicate) -> u32 {
    match pred {
        // Tier 1: Immediate from path (1-5)
        Predicate::Ext(_) | Predicate::Name(_) => 1,

        // Tier 2: Path matching (5-10)
        Predicate::Path(_) | Predicate::PathExact(_) | Predicate::In(_) => 5,

        // Tier 3: Stat syscall (10-20)
        Predicate::Size(_) | Predicate::Modified(_) => 10,

        // Tier 4: File read + search (100-500)
        Predicate::Contains(_) | Predicate::Matches(_) => 100,

        // Tier 5: Parse + tree query (500-2000)
        Predicate::Func(_) | Predicate::Struct(_) | Predicate::Def(_) |
        Predicate::Import(_) | Predicate::Call(_) => 500,

        // Tier 6: Parse + full tree scan (1000-3000)
        Predicate::Comment(_) | Predicate::Str(_) => 1000,
    }
}

/// Optimize a query expression by reordering predicates
pub fn optimize(expr: Expr) -> Expr {
    match expr {
        Expr::And(mut predicates) => {
            // Sort by cost ascending - cheapest first for short-circuit
            predicates.sort_by_key(|p| predicate_cost(p));
            Expr::And(predicates)
        }
        Expr::Or(mut predicates) => {
            // Sort by cost ascending - cheapest first might satisfy early
            predicates.sort_by_key(|p| predicate_cost(p));
            Expr::Or(predicates)
        }
        Expr::Not(inner) => Expr::Not(Box::new(optimize(*inner))),
        other => other,
    }
}

// In main.rs or search.rs
let parsed_query = parse_query(&query_string)?;
let optimized_query = optimize(parsed_query);
evaluate(optimized_query, files)
```

**Additional features:**

```rust
// Optional: Show optimization in verbose mode
if args.verbose {
    eprintln!("Original: {}", original_query);
    eprintln!("Optimized: {}", optimized_query);
}

// Optional: Disable with --no-optimize flag
let query = if args.no_optimize {
    parsed_query
} else {
    optimize(parsed_query)
};
```

**Why this matters:**

| Query | Without Optimizer | With Optimizer |
|-------|-------------------|----------------|
| `func:main & ext:rs` | Parse 10,000 files | Parse ~200 .rs files |
| `contains:TODO & size:<1000` | Read 10,000 files | Stat first, read small files |
| `import:react & ext:tsx & path:components` | Parse all, then filter | Filter by path+ext, parse few |

**Impact:** 10-100x faster for poorly-ordered queries (very common)
**Effort:** Low (1-2 hours)

---

## Medium-Priority Optimizations

### 5. Cache Byte-to-Line Offset Mapping

**Location:** `src/evaluator.rs`

**Problem:** Converting byte offsets to line numbers is O(n) per range. Multiple ranges in one file causes O(n*m) work.

**Solution:** Build line offset table once per file.

```rust
struct LineOffsets {
    offsets: Vec<usize>, // byte offset of each line start
}

impl LineOffsets {
    fn from_content(content: &str) -> Self {
        let mut offsets = vec![0];
        for (i, c) in content.char_indices() {
            if c == '\n' {
                offsets.push(i + 1);
            }
        }
        Self { offsets }
    }

    fn byte_to_line(&self, byte: usize) -> usize {
        self.offsets.partition_point(|&o| o <= byte)
    }
}
```

**Impact:** O(n^2) to O(n) for files with many matches
**Effort:** Low (1 hour)

---

### 6. Reduce PathBuf Cloning in Parallel Phase

**Location:** `src/commands/search.rs:164`

**Problem:** PathBuf is cloned when moved into the parallel iterator closure.

**Solution:** Use `Arc<PathBuf>` or pass by reference where possible.

```rust
let matching_files: Vec<_> = pre_filtered_files
    .par_iter()  // Use par_iter() not par_iter().cloned()
    .filter_map(|path| {
        // work with &PathBuf instead of PathBuf
    })
    .collect();
```

**Impact:** Reduced allocation pressure
**Effort:** Low (30 minutes)

---

## Low-Priority Optimizations

### 7. Memory-map Large Files

**Problem:** Large files (>1MB) are read entirely into memory.

**Solution:** Use `memmap2` for files above a threshold.

```rust
use memmap2::Mmap;

fn read_content(path: &Path) -> Result<Cow<str>> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > 1_000_000 {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        // Convert to str...
    } else {
        fs::read_to_string(path).map(Cow::Owned)
    }
}
```

**Impact:** Reduced memory pressure for large files
**Effort:** Medium (1-2 hours)

---

### 8. Reuse Tree-sitter Parser

**Location:** `src/predicates/code_aware/mod.rs`

**Problem:** Parser may be created per-file (verify current implementation).

**Solution:** Use thread-local parser instances.

```rust
thread_local! {
    static PARSER: RefCell<Parser> = RefCell::new(Parser::new());
}

fn parse_file(content: &str, language: Language) -> Option<Tree> {
    PARSER.with(|parser| {
        let mut parser = parser.borrow_mut();
        parser.set_language(&language).ok()?;
        parser.parse(content, None)
    })
}
```

**Impact:** Minor improvement
**Effort:** Low (30 minutes)

---

### 9. Handle Very Large Files (>100MB)

**Location:** `src/evaluator.rs`

**Problem:** Files over 100MB are read entirely into memory via `fs::read()`. This causes:
- High memory pressure (multiple large files in parallel = OOM risk)
- Slow initial load time
- Tree-sitter must parse entire file regardless

**Current behavior:**
```rust
// evaluator.rs:41 - reads entire file into memory
let bytes = fs::read(&self.path)?;
```

**Solution options:**

1. **Skip by default, opt-in flag:**
```rust
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB

if metadata.len() > MAX_FILE_SIZE {
    if !args.include_large_files {
        return Ok(false); // Skip file
    }
    warn!("Processing large file: {} ({}MB)", path, metadata.len() / 1_000_000);
}
```

2. **Streaming for content predicates (contains/matches):**
```rust
// For contains: predicate on very large files
fn streaming_contains(path: &Path, needle: &str) -> Result<bool> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(64 * 1024, file);

    // Use memchr or aho-corasick for streaming search
    // Note: Can't use tree-sitter with streaming
}
```

3. **Sampling for code-aware predicates:**
```rust
// Parse only first N bytes for structure detection
const SAMPLE_SIZE: usize = 1_000_000; // 1MB sample

fn sample_parse(path: &Path) -> Result<Option<Tree>> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; SAMPLE_SIZE];
    let bytes_read = file.read(&mut buffer)?;
    // Parse sample - may miss definitions later in file
}
```

**Recommended approach:** Option 1 (skip by default) is safest. Most 100MB+ files are:
- Generated code (node_modules, vendor)
- Binary files misidentified
- Data files (logs, dumps)

Add `--include-large-files` flag and `size:<100mb` predicate guidance.

**Impact:** Prevents OOM, faster scans on repos with large files
**Effort:** Low (1 hour) for skip approach, Medium (3-4 hours) for streaming

---

## What's Already Optimal

- **Two-pass filtering** - metadata checks before content reading
- **Lazy content loading** - files only read when needed
- **Rayon parallelism** - correct choice for CPU-bound work
- **Short-circuit evaluation** - AND/OR predicates exit early (effectiveness depends on predicate order; see optimization #4)
- **Compiled regex caching** - regex patterns compiled once

---

## Implementation Roadmap

### Phase 1: Quick Wins (1-2 days)
- [ ] Automatic query predicate reordering (10-100x for poorly-ordered queries)
- [ ] Remove content `.to_string()` clone
- [ ] Cache byte-to-line offset mapping
- [ ] Reduce PathBuf cloning

**Expected improvement: 25-50% (much higher for suboptimal queries)**

### Phase 2: Medium Effort (2-3 days)
- [ ] Parallel directory walking
- [ ] Thread-local parser reuse
- [ ] Memory-map large files

**Expected improvement: +15%**

### Phase 3: High Impact (3-5 days)
- [ ] Pre-compile all tree-sitter queries
- [ ] Refactor profile system for static queries

**Expected improvement: +20%**

### Total Expected Improvement: 50-80% (10-100x for poorly-ordered queries)

---

## Benchmarking

Before implementing, establish baselines:

```bash
# Simple search
hyperfine 'rdump "ext:rs"' --warmup 3

# Content search
hyperfine 'rdump "ext:rs & contains:fn"' --warmup 3

# Code-aware search (most impacted by optimizations)
hyperfine 'rdump "ext:rs & func:main"' --warmup 3

# Large directory
hyperfine 'rdump "ext:rs" /large/codebase' --warmup 3
```

Compare before/after for each optimization to validate impact.

---

## Notes

- All optimizations are backward-compatible
- No API changes required
- Consider feature-gating expensive optimizations (e.g., mmap) if they add dependencies
- Profile with `cargo flamegraph` to identify actual bottlenecks before optimizing
