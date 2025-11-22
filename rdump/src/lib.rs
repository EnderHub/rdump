//! # rdump - Library API Overview
//!
//! `rdump` provides a library-first, semantic code search API. Export groups:
//! - **Library API**: `SearchOptions`, `SearchResult`, `Match`,
//!   `SearchResultIterator`, `search_iter`, and `search`. `SqlDialect` is
//!   re-exported for convenience.
//! - **CLI API**: `Cli`, `Commands`, `SearchArgs`, and related enums remain
//!   public for consumers that embed or extend the CLI.
//!
//! ## Quick Start (collecting results)
//! ```rust
//! # use tempfile::tempdir;
//! use rdump::{search, SearchOptions};
//! # let dir = tempdir().unwrap();
//! # std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
//! # let opts = SearchOptions { root: dir.path().to_path_buf(), ..Default::default() };
//! let results = search("func:main", opts)?;
//! for r in &results {
//!     println!("{} ({} matches)", r.path.display(), r.match_count());
//! }
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Streaming API (memory-efficient)
//! ```rust
//! # use tempfile::tempdir;
//! use rdump::{search_iter, SearchOptions};
//! # let dir = tempdir().unwrap();
//! # std::fs::write(dir.path().join("lib.rs"), "fn helper() {}").unwrap();
//! # let opts = SearchOptions { root: dir.path().to_path_buf(), ..Default::default() };
//! let first_two: Vec<_> = search_iter("ext:rs", opts)?
//!     .take(2)
//!     .filter_map(Result::ok)
//!     .collect();
//! assert!(!first_two.is_empty());
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Query Language (RQL) Basics
//! - Predicates: `ext:rs`, `func:main`, `class:User`, `contains:TODO`
//! - Operators: `&` (AND), `|` (OR), `!` (NOT), parentheses for grouping
//! - Example: `ext:rs & (func:new | func:default)`
//!
//! ## Feature Flags
//! - `async` &mdash; optional Tokio-backed helpers for async runtimes (not required for sync use)
//!   ```toml
//!   [dependencies]
//!   rdump = { version = "0.1", features = ["async"] }
//!   ```

// Declare all our modules
#[cfg(feature = "async")]
mod async_api;
pub mod commands;
pub mod config;
pub mod evaluator;
pub mod formatter;
pub mod limits {
    use std::path::PathBuf;
    use std::time::Duration;

    /// Maximum file size we will read in bytes (default: 10MB).
    pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

    /// Maximum directory depth we will traverse by default.
    pub const DEFAULT_MAX_DEPTH: usize = 100;

    /// Maximum time we will spend evaluating a single regex against a file's lines.
    pub const MAX_REGEX_EVAL_DURATION: Duration = Duration::from_millis(200);

    /// Returns true if the byte slice is likely a binary file.
    pub fn is_probably_binary(bytes: &[u8]) -> bool {
        bytes.contains(&0)
    }

    /// Light heuristic to skip obvious secrets before printing them.
    pub fn maybe_contains_secret(content: &str) -> bool {
        let lower = content.to_lowercase();
        lower.contains("-----begin private key-----")
            || lower.contains("aws_secret_access_key")
            || lower.contains("aws_access_key_id")
            || lower.contains("secret_key=")
            || lower.contains("secret-key=")
            || lower.contains("authorization: bearer")
            || lower.contains("eyj") // common JWT prefix (base64url '{"typ":"JWT"...}')
            || lower.contains("private_key")
    }

    /// Canonicalize `path` and ensure it stays under `root`. Returns the canonicalized path.
    pub fn safe_canonicalize(path: &PathBuf, root: &PathBuf) -> anyhow::Result<PathBuf> {
        let canonical_root = dunce::canonicalize(root)?;
        let canonical = dunce::canonicalize(path)?;
        if !canonical.starts_with(&canonical_root) {
            anyhow::bail!(
                "Path {} escapes root {}",
                canonical.display(),
                canonical_root.display()
            );
        }
        Ok(canonical)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;
        use tempfile::tempdir;

        #[test]
        fn test_is_probably_binary() {
            assert!(is_probably_binary(&[0, 1, 2, 3]));
            assert!(is_probably_binary(b"hello\x00world"));
            assert!(!is_probably_binary(b"hello world"));
            assert!(!is_probably_binary(b"fn main() {}"));
        }

        #[test]
        fn test_maybe_contains_secret_private_key() {
            assert!(maybe_contains_secret("-----BEGIN PRIVATE KEY-----"));
            assert!(maybe_contains_secret(
                "some text with -----begin private key----- in it"
            ));
        }

        #[test]
        fn test_maybe_contains_secret_aws() {
            assert!(maybe_contains_secret("aws_secret_access_key=abcd1234"));
            assert!(maybe_contains_secret("AWS_ACCESS_KEY_ID=AKIA..."));
        }

        #[test]
        fn test_maybe_contains_secret_other() {
            assert!(maybe_contains_secret("secret_key=mykey"));
            assert!(maybe_contains_secret("secret-key=mykey"));
            assert!(maybe_contains_secret("Authorization: Bearer token"));
            assert!(maybe_contains_secret("eyJhbGciOiJIUzI1NiJ9")); // JWT
            assert!(maybe_contains_secret("private_key: xyz"));
        }

        #[test]
        fn test_maybe_contains_secret_safe() {
            assert!(!maybe_contains_secret("fn main() { println!(\"Hello\"); }"));
            assert!(!maybe_contains_secret("SELECT * FROM users;"));
        }

        #[test]
        fn test_safe_canonicalize_within_root() {
            let dir = tempdir().unwrap();
            let root = dir.path().to_path_buf();
            let subdir = root.join("subdir");
            fs::create_dir(&subdir).unwrap();
            let file = subdir.join("test.txt");
            fs::write(&file, "content").unwrap();

            let result = safe_canonicalize(&file, &root);
            assert!(result.is_ok());
            assert!(result
                .unwrap()
                .starts_with(dunce::canonicalize(&root).unwrap()));
        }

        #[test]
        fn test_safe_canonicalize_escapes_root() {
            let dir = tempdir().unwrap();
            let root = dir.path().join("project");
            fs::create_dir(&root).unwrap();

            // Create a file outside the root
            let outside_file = dir.path().join("outside.txt");
            fs::write(&outside_file, "content").unwrap();

            let result = safe_canonicalize(&outside_file, &root);
            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("escapes root"));
        }

        #[test]
        fn test_safe_canonicalize_nonexistent_path() {
            let dir = tempdir().unwrap();
            let root = dir.path().to_path_buf();
            let nonexistent = root.join("nonexistent.txt");

            let result = safe_canonicalize(&nonexistent, &root);
            assert!(result.is_err());
        }
    }
}
pub mod parser;
pub mod predicates;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};

// =============================================================================
// Library API Exports
// =============================================================================

/// SQL dialect used for SQL-aware searches; re-exported so callers can configure
/// dialects without reaching into internal modules.
pub use crate::predicates::code_aware::SqlDialect;
#[cfg(feature = "async")]
pub use async_api::{search_all_async, search_async};

// Bring our command functions into scope
use crate::limits::{is_probably_binary, MAX_FILE_SIZE};
use crate::predicates::code_aware::SqlDialect as CodeSqlDialect;
use commands::{lang::run_lang, preset::run_preset, search::run_search};
use std::io::ErrorKind;
use std::ops::Range;
use tree_sitter::Range as TsRange;

// =============================================================================
// Library API Types
// =============================================================================

/// Options for performing a search (library-friendly).
///
/// Defaults are chosen for safety and ergonomics:
/// - `root`: current directory (`.`)
/// - `presets`: empty (no preset filter)
/// - `no_ignore`: false (respect ignore files)
/// - `hidden`: false (skip hidden files)
/// - `max_depth`: `None` (use default max depth)
/// - `sql_dialect`: `None` (auto-detect)
///
/// This struct contains only the parameters needed for search logic,
/// excluding CLI-specific concerns like output formatting and colors.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Root directory to search from.
    pub root: PathBuf,

    /// Named presets to apply (e.g., "rust", "python").
    pub presets: Vec<String>,

    /// If true, ignore .gitignore rules.
    pub no_ignore: bool,

    /// If true, include hidden files and directories.
    pub hidden: bool,

    /// Maximum directory depth to search.
    pub max_depth: Option<usize>,

    /// SQL dialect override for .sql files.
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

/// A file that matched the search query.
///
/// Contains the file path, all matches within the file, and the file content.
/// For whole-file matches (boolean predicates like `ext:rs`), the `matches`
/// vector will be empty.
///
/// # Example
///
/// ```rust
/// use rdump::{Match, SearchResult};
/// use std::path::PathBuf;
///
/// let whole_file = SearchResult {
///     path: PathBuf::from("src/lib.rs"),
///     matches: vec![],
///     content: String::from("fn main() {}"),
/// };
/// assert!(whole_file.is_whole_file_match());
///
/// let hunked = SearchResult {
///     path: PathBuf::from("src/lib.rs"),
///     matches: vec![Match {
///         start_line: 3,
///         end_line: 4,
///         start_column: 0,
///         end_column: 12,
///         byte_range: 10..34,
///         text: String::from("fn main() {}"),
///     }],
///     content: String::from("fn main() {}"),
/// };
/// assert_eq!(hunked.matched_lines(), vec![3, 4]);
/// assert_eq!(hunked.match_count(), 1);
/// assert_eq!(hunked.total_lines_matched(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Path to the matched file.
    pub path: PathBuf,

    /// Matches within this file (empty for whole-file matches).
    pub matches: Vec<Match>,

    /// Full file content.
    pub content: String,
}

impl SearchResult {
    /// Returns true if this is a whole-file match (no specific hunks).
    pub fn is_whole_file_match(&self) -> bool {
        self.matches.is_empty()
    }

    /// Get all matched line numbers (1-indexed), sorted and deduplicated.
    pub fn matched_lines(&self) -> Vec<usize> {
        let mut lines: Vec<usize> = self
            .matches
            .iter()
            .flat_map(|m| m.start_line..=m.end_line)
            .collect();
        lines.sort_unstable();
        lines.dedup();
        lines
    }

    /// Get the number of matches in this file.
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Get the total number of unique lines matched.
    pub fn total_lines_matched(&self) -> usize {
        self.matched_lines().len()
    }
}

/// A single match within a file.
///
/// Line numbers are 1-indexed (editor convention). Columns and byte ranges are
/// 0-indexed because they are byte offsets from the start of the line/file.
///
/// # Example
///
/// ```rust
/// use rdump::Match;
///
/// let m = Match {
///     start_line: 3,
///     end_line: 4,
///     start_column: 0,
///     end_column: 12,
///     byte_range: 10..34,
///     text: String::from("fn main() {}\nprintln!(\"hi\");"),
/// };
/// assert_eq!(m.line_count(), 2);
/// assert!(m.is_multiline());
/// assert_eq!(m.byte_len(), 24);
/// assert_eq!(m.first_line(), "fn main() {}");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Starting line (1-indexed) of the match.
    pub start_line: usize,
    /// Ending line (1-indexed) of the match.
    pub end_line: usize,
    /// Starting column (0-indexed) of the match within `start_line`.
    pub start_column: usize,
    /// Ending column (0-indexed) of the match within `end_line`.
    pub end_column: usize,
    /// Byte range of the match within the file content.
    pub byte_range: Range<usize>,
    /// The matched text (may be shortened for large hunks).
    pub text: String,
}

impl Match {
    /// Returns the number of lines this match spans (inclusive).
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }

    /// Returns true if the match spans more than one line.
    pub fn is_multiline(&self) -> bool {
        self.start_line != self.end_line
    }

    /// Returns the byte length of this match.
    pub fn byte_len(&self) -> usize {
        self.byte_range.len()
    }

    /// Returns the first line of the matched text (or empty string).
    pub fn first_line(&self) -> &str {
        self.text.lines().next().unwrap_or("")
    }
}

/// Converts tree-sitter ranges into user-facing `Match` structs.
fn ranges_to_matches(content: &str, ranges: &[TsRange]) -> Vec<Match> {
    if content.is_empty() {
        return Vec::new();
    }

    ranges
        .iter()
        .filter_map(|range| {
            let text = content.get(range.start_byte..range.end_byte)?;

            Some(Match {
                start_line: range.start_point.row + 1,
                end_line: range.end_point.row + 1,
                start_column: range.start_point.column,
                end_column: range.end_point.column,
                byte_range: range.start_byte..range.end_byte,
                text: text.to_string(),
            })
        })
        .collect()
}

fn read_file_content_for_iterator(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => anyhow::anyhow!("File no longer exists: {}", path.display()),
        ErrorKind::PermissionDenied => {
            anyhow::anyhow!("Permission denied reading: {}", path.display())
        }
        _ => anyhow::anyhow!("Failed to read {}: {}", path.display(), e),
    })?;

    if metadata.len() > MAX_FILE_SIZE {
        bail!(
            "File {} exceeds maximum size limit ({} bytes > {} bytes)",
            path.display(),
            metadata.len(),
            MAX_FILE_SIZE
        );
    }

    let bytes = fs::read(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => anyhow::anyhow!("File no longer exists: {}", path.display()),
        ErrorKind::PermissionDenied => {
            anyhow::anyhow!("Permission denied reading: {}", path.display())
        }
        _ => anyhow::anyhow!("Failed to read {}: {}", path.display(), e),
    })?;

    let check_len = bytes.len().min(8192);
    if is_probably_binary(&bytes[..check_len]) {
        bail!("Skipping binary file: {}", path.display());
    }

    let content = String::from_utf8(bytes)
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in {}: {}", path.display(), e))?;

    Ok(content)
}

// =============================================================================
// Library API Iterators
// =============================================================================

/// Iterator over search results, lazily loading file content as items are
/// pulled. Constructed via [`search_iter`].
///
/// # Examples
/// ```no_run
/// use rdump::{search_iter, SearchOptions};
///
/// let iter = search_iter("ext:rs", SearchOptions::default())?;
/// assert!(iter.remaining() >= 0);
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct SearchResultIterator {
    /// Raw results from `perform_search_internal`.
    inner: std::vec::IntoIter<(PathBuf, Vec<TsRange>)>,
}

impl SearchResultIterator {
    /// Create a new iterator from raw search results.
    pub(crate) fn new(results: Vec<(PathBuf, Vec<TsRange>)>) -> Self {
        Self {
            inner: results.into_iter(),
        }
    }

    /// Get the number of remaining results without advancing the iterator.
    pub fn remaining(&self) -> usize {
        self.inner.len()
    }
}

impl Iterator for SearchResultIterator {
    type Item = Result<SearchResult>;

    fn next(&mut self) -> Option<Self::Item> {
        let (path, ranges) = self.inner.next()?;

        let content = match read_file_content_for_iterator(&path) {
            Ok(c) => c,
            Err(e) => return Some(Err(e)),
        };

        let matches = if ranges.is_empty() {
            Vec::new()
        } else {
            ranges_to_matches(&content, &ranges)
        };

        Some(Ok(SearchResult {
            path,
            matches,
            content,
        }))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.inner.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for SearchResultIterator {}

impl std::iter::FusedIterator for SearchResultIterator {}

// =============================================================================
// Library API Functions
// =============================================================================

/// Run an rdump query and stream results lazily.
///
/// This is the preferred API for large codebases: it parses the query, resolves
/// presets, validates the root, and returns an iterator that loads file content
/// only when each item is consumed.
///
/// # Arguments
/// - `query`: RQL string (e.g., `ext:rs & func:main`). Empty string is allowed
///   when presets are provided; otherwise an error is returned.
/// - `options`: Search configuration (owned to keep the iterator lifetime-simple).
///
/// # Returns
/// An iterator yielding `Result<SearchResult>` for each matching file.
///
/// # Errors
/// - Invalid query syntax (parse/validation failure)
/// - Unknown preset name
/// - Root directory missing or inaccessible
/// - Empty query with no presets
///
/// # Examples
/// Basic usage:
/// ```
/// use rdump::{search_iter, SearchOptions};
///
/// let iter = search_iter("ext:rs & func:main", SearchOptions::default())?;
/// for result in iter {
///     let result = result?;
///     println!("{}: {} matches", result.path.display(), result.matches.len());
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// Early termination:
/// ```
/// use rdump::{search_iter, SearchOptions};
/// let first_two: Vec<_> = search_iter("ext:rs", SearchOptions::default())?
///     .take(2)
///     .collect::<Result<Vec<_>, _>>()?;
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// Skipping per-file errors:
/// ```
/// use rdump::{search_iter, SearchOptions};
/// let ok_results: Vec<_> = search_iter("ext:rs", SearchOptions::default())?
///     .filter_map(Result::ok)
///     .collect();
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn search_iter(query: &str, options: SearchOptions) -> Result<SearchResultIterator> {
    let raw_results = commands::search::perform_search_internal(query, &options)?;
    Ok(SearchResultIterator::new(raw_results))
}

/// Run an rdump query and collect all results into memory.
///
/// Convenience wrapper around [`search_iter`]. Suitable for small/medium result
/// sets; for large codebases prefer `search_iter` to avoid loading all content
/// at once.
///
/// # Arguments
/// - `query`: RQL string (e.g., `ext:rs & func:main`)
/// - `options`: Search configuration
///
/// # Returns
/// A vector of matching results. Short-circuits on the first error from the
/// iterator.
///
/// # Errors
/// - Invalid query syntax
/// - Unknown preset name
/// - Root directory missing or inaccessible
/// - First per-file error encountered during iteration
///
/// # Examples
/// Basic usage:
/// ```no_run
/// use rdump::{search, SearchOptions};
///
/// let results = search("ext:rs", SearchOptions::default())?;
/// println!("Found {} files", results.len());
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// Performance note:
/// ```no_run
/// for result in rdump::search_iter("ext:rs", rdump::SearchOptions::default())? {
///     let result = result?;
///     // Prefer streaming for very large result sets to avoid loading all content at once.
///     println!("{}", result.path.display());
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn search(query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
    search_iter(query, options)?.collect()
}

// =============================================================================
// CLI API
// =============================================================================

// These structs and enums define the public API of our CLI.
// They need to be public so the `commands` modules can use them.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "A fast, expressive, code-aware tool to find and dump file contents."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for files using a query (default command).
    #[command(visible_alias = "s")]
    Search(SearchArgs),
    /// List supported languages and their available predicates.
    #[command(visible_alias = "l")]
    Lang(LangArgs),
    /// Manage saved presets.
    #[command(visible_alias = "p")]
    Preset(PresetArgs),
}

#[derive(Debug, Clone, ValueEnum, Default, PartialEq)]
pub enum ColorChoice {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, ValueEnum, Copy)]
pub enum SqlDialectFlag {
    Generic,
    Postgres,
    Mysql,
    Sqlite,
}

impl From<SqlDialectFlag> for CodeSqlDialect {
    fn from(value: SqlDialectFlag) -> Self {
        match value {
            SqlDialectFlag::Generic => CodeSqlDialect::Generic,
            SqlDialectFlag::Postgres => CodeSqlDialect::Postgres,
            SqlDialectFlag::Mysql => CodeSqlDialect::Mysql,
            SqlDialectFlag::Sqlite => CodeSqlDialect::Sqlite,
        }
    }
}

#[derive(Parser, Debug, Default)]
pub struct SearchArgs {
    /// The query string to search for, using rdump Query Language (RQL).
    ///
    /// RQL supports logical operators (&, |, !), parentheses, and key:value predicates.
    /// Values with spaces must be quoted (e.g., contains:'fn main').
    ///
    /// METADATA PREDICATES:
    /// ```text
    /// ext:<str>               - File extension (e.g., "rs", "toml")
    /// name:<glob>             - File name glob pattern (e.g., "test_*.rs")
    /// path:<str>              - Substring in the full file path
    /// in:<path>               - Directory path to search within
    /// size:[>|<]<num>[kb|mb]  - File size (e.g., ">10kb")
    /// modified:[>|<]<num>[h|d|w] - Modified time (e.g., "<2d")
    /// ```
    ///
    /// CONTENT PREDICATES:
    /// ```text
    /// contains:<str>          - Literal string a file contains
    /// matches:<regex>         - Regular expression a file's content matches
    /// ```
    ///
    /// CODE-AWARE PREDICATES for supported languages:
    /// ```text
    /// def:<str>               - A generic definition (class, struct, enum, etc.)
    /// func:<str>              - A function or method
    /// import:<str>            - An import or use statement
    /// call:<str>              - A function or method call site
    /// ```
    ///
    /// GRANULAR DEFINITIONS:
    /// ```text
    /// class:<str>             - A class definition
    /// struct:<str>            - A struct definition
    /// enum:<str>              - An enum definition
    /// interface:<str>         - An interface definition
    /// trait:<str>             - A trait definition
    /// type:<str>              - A type alias
    /// impl:<str>              - An implementation block (e.g., `impl User`)
    /// macro:<str>             - A macro definition
    /// ```
    ///
    /// SYNTACTIC CONTENT:
    /// ```text
    /// comment:<str>           - Text inside a comment (e.g., "TODO", "FIXME")
    /// str:<str>               - Text inside a string literal
    /// ```
    ///
    /// REACT-SPECIFIC PREDICATES (.jsx, .tsx):
    /// ```text
    /// component:<str>         - A React component definition
    /// element:<str>           - A JSX element/tag (e.g., `div`, `MyComponent`)
    /// hook:<str>              - A React hook call (e.g., `useState`, `useEffect`)
    /// customhook:<str>        - A custom hook definition (e.g., `useAuth`)
    /// prop:<str>              - A prop being passed to a JSX element
    /// ```
    #[arg(verbatim_doc_comment, name = "QUERY")]
    pub query: Option<String>,
    /// Force the SQL dialect to use for .sql files (overrides auto-detection).
    #[arg(long, value_enum, ignore_case = true)]
    pub dialect: Option<SqlDialectFlag>,
    #[arg(long, short)]
    pub preset: Vec<String>,
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    #[arg(short, long)]
    pub line_numbers: bool,
    #[arg(long, help = "Alias for --format=cat, useful for piping")]
    pub no_headers: bool,
    #[arg(long, value_enum, default_value_t = Format::Hunks)]
    pub format: Format,
    #[arg(long)]
    pub no_ignore: bool,
    #[arg(long)]
    pub hidden: bool,
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto, help = "When to use syntax highlighting")]
    pub color: ColorChoice,
    #[arg(long)]
    pub max_depth: Option<usize>,
    #[arg(
        long,
        short = 'C',
        value_name = "LINES",
        help = "Show LINES of context around matches for --format=hunks"
    )]
    pub context: Option<usize>,

    /// List files with metadata instead of dumping content. Alias for --format=find
    #[arg(long)]
    pub find: bool,
}

#[derive(Parser, Debug)]
pub struct LangArgs {
    #[command(subcommand)]
    pub action: Option<LangAction>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum LangAction {
    /// List all supported languages.
    List,
    /// Describe the predicates available for a specific language.
    Describe { language: String },
}

#[derive(Parser, Debug)]
pub struct PresetArgs {
    #[command(subcommand)]
    pub action: PresetAction,
}

#[derive(Subcommand, Debug, Clone)]
pub enum PresetAction {
    /// List all available presets.
    List,
    /// Add or update a preset in the global config file.
    Add {
        #[arg(required = true)]
        name: String,
        #[arg(required = true)]
        query: String,
    },
    /// Remove a preset from the global config file.
    Remove {
        #[arg(required = true)]
        name: String,
    },
}

#[derive(Debug, Clone, ValueEnum, Default, PartialEq)]
pub enum Format {
    /// Show only the specific code blocks ("hunks") that match a semantic query
    #[default]
    Hunks,
    /// Human-readable markdown with file headers
    Markdown,
    /// Machine-readable JSON
    Json,
    /// A simple list of matching file paths
    Paths,
    /// Raw concatenated file content, for piping
    Cat,
    /// `ls`-like output with file metadata
    Find,
}

// This is the function that will be called from main.rs
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search(args) => run_search(args),
        Commands::Lang(args) => {
            // Default to `list` if no subcommand is given for `lang`
            let action = args.action.unwrap_or(LangAction::List);
            run_lang(action)
        }
        Commands::Preset(args) => run_preset(args.action),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_sql_dialect_flag_conversion_generic() {
        let flag = SqlDialectFlag::Generic;
        let dialect: CodeSqlDialect = flag.into();
        assert_eq!(dialect, CodeSqlDialect::Generic);
    }

    #[test]
    fn test_sql_dialect_flag_conversion_postgres() {
        let flag = SqlDialectFlag::Postgres;
        let dialect: CodeSqlDialect = flag.into();
        assert_eq!(dialect, CodeSqlDialect::Postgres);
    }

    #[test]
    fn test_sql_dialect_flag_conversion_mysql() {
        let flag = SqlDialectFlag::Mysql;
        let dialect: CodeSqlDialect = flag.into();
        assert_eq!(dialect, CodeSqlDialect::Mysql);
    }

    #[test]
    fn test_sql_dialect_flag_conversion_sqlite() {
        let flag = SqlDialectFlag::Sqlite;
        let dialect: CodeSqlDialect = flag.into();
        assert_eq!(dialect, CodeSqlDialect::Sqlite);
    }

    #[test]
    fn test_color_choice_default() {
        let choice = ColorChoice::default();
        assert_eq!(choice, ColorChoice::Auto);
    }

    #[test]
    fn test_format_default() {
        let format = Format::default();
        assert_eq!(format, Format::Hunks);
    }

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.root, PathBuf::from("."));
        assert!(options.presets.is_empty());
        assert!(!options.no_ignore);
        assert!(!options.hidden);
        assert!(options.max_depth.is_none());
        assert!(options.sql_dialect.is_none());
    }

    #[test]
    fn test_search_options_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<SearchOptions>();
        assert_sync::<SearchOptions>();
    }

    #[test]
    fn test_is_whole_file_match_empty() {
        let result = SearchResult {
            path: PathBuf::from("test.rs"),
            matches: vec![],
            content: "fn main() {}".to_string(),
        };
        assert!(result.is_whole_file_match());
    }

    #[test]
    fn test_is_whole_file_match_with_matches() {
        let result = SearchResult {
            path: PathBuf::from("test.rs"),
            matches: vec![Match {
                start_line: 1,
                end_line: 1,
                start_column: 0,
                end_column: 12,
                byte_range: 0..12,
                text: "fn main() {}".to_string(),
            }],
            content: "fn main() {}".to_string(),
        };
        assert!(!result.is_whole_file_match());
    }

    #[test]
    fn test_matched_lines_single_match() {
        let result = SearchResult {
            path: PathBuf::from("test.rs"),
            matches: vec![Match {
                start_line: 5,
                end_line: 7,
                start_column: 0,
                end_column: 1,
                byte_range: 0..100,
                text: "...".to_string(),
            }],
            content: "...".to_string(),
        };
        assert_eq!(result.matched_lines(), vec![5, 6, 7]);
    }

    #[test]
    fn test_matched_lines_overlapping() {
        let result = SearchResult {
            path: PathBuf::from("test.rs"),
            matches: vec![
                Match {
                    start_line: 1,
                    end_line: 3,
                    start_column: 0,
                    end_column: 1,
                    byte_range: 0..50,
                    text: "...".to_string(),
                },
                Match {
                    start_line: 2,
                    end_line: 4,
                    start_column: 0,
                    end_column: 1,
                    byte_range: 25..100,
                    text: "...".to_string(),
                },
            ],
            content: "...".to_string(),
        };
        assert_eq!(result.matched_lines(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_matched_lines_empty_matches() {
        let result = SearchResult {
            path: PathBuf::from("test.rs"),
            matches: vec![],
            content: "...".to_string(),
        };
        assert_eq!(result.matched_lines(), Vec::<usize>::new());
    }

    #[test]
    fn test_match_count() {
        let result = SearchResult {
            path: PathBuf::from("test.rs"),
            matches: vec![
                Match {
                    start_line: 1,
                    end_line: 1,
                    start_column: 0,
                    end_column: 5,
                    byte_range: 0..5,
                    text: "fn a()".to_string(),
                },
                Match {
                    start_line: 3,
                    end_line: 3,
                    start_column: 0,
                    end_column: 5,
                    byte_range: 10..15,
                    text: "fn b()".to_string(),
                },
            ],
            content: "...".to_string(),
        };
        assert_eq!(result.match_count(), 2);
    }

    #[test]
    fn test_total_lines_matched() {
        let result = SearchResult {
            path: PathBuf::from("test.rs"),
            matches: vec![
                Match {
                    start_line: 1,
                    end_line: 3,
                    start_column: 0,
                    end_column: 1,
                    byte_range: 0..50,
                    text: "...".to_string(),
                },
                Match {
                    start_line: 5,
                    end_line: 5,
                    start_column: 0,
                    end_column: 1,
                    byte_range: 60..70,
                    text: "...".to_string(),
                },
            ],
            content: "...".to_string(),
        };
        assert_eq!(result.total_lines_matched(), 4);
    }

    #[test]
    fn test_search_result_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SearchResult>();
    }

    #[test]
    fn test_match_line_count_single_line() {
        let m = Match {
            start_line: 5,
            end_line: 5,
            start_column: 0,
            end_column: 10,
            byte_range: 0..10,
            text: "fn main()".to_string(),
        };
        assert_eq!(m.line_count(), 1);
    }

    #[test]
    fn test_match_line_count_multi_line() {
        let m = Match {
            start_line: 1,
            end_line: 10,
            start_column: 0,
            end_column: 1,
            byte_range: 0..100,
            text: "...".to_string(),
        };
        assert_eq!(m.line_count(), 10);
    }

    #[test]
    fn test_match_is_multiline() {
        let single = Match {
            start_line: 3,
            end_line: 3,
            start_column: 0,
            end_column: 5,
            byte_range: 0..5,
            text: "hello".to_string(),
        };
        let multi = Match {
            start_line: 3,
            end_line: 5,
            start_column: 0,
            end_column: 5,
            byte_range: 0..50,
            text: "line1\nline2\nline3".to_string(),
        };
        assert!(!single.is_multiline());
        assert!(multi.is_multiline());
    }

    #[test]
    fn test_match_byte_len() {
        let m = Match {
            start_line: 1,
            end_line: 1,
            start_column: 0,
            end_column: 12,
            byte_range: 10..22,
            text: "fn main() {}".to_string(),
        };
        assert_eq!(m.byte_len(), 12);
    }

    #[test]
    fn test_match_first_line() {
        let single = Match {
            start_line: 1,
            end_line: 1,
            start_column: 0,
            end_column: 12,
            byte_range: 0..12,
            text: "fn main() {}".to_string(),
        };
        let multi = Match {
            start_line: 1,
            end_line: 3,
            start_column: 0,
            end_column: 1,
            byte_range: 0..30,
            text: "fn main() {\n    println!(\"hi\");\n}".to_string(),
        };
        let empty = Match {
            start_line: 1,
            end_line: 1,
            start_column: 0,
            end_column: 0,
            byte_range: 0..0,
            text: "".to_string(),
        };
        assert_eq!(single.first_line(), "fn main() {}");
        assert_eq!(multi.first_line(), "fn main() {");
        assert_eq!(empty.first_line(), "");
    }

    #[test]
    fn test_match_send_sync_and_eq() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Match>();

        let m1 = Match {
            start_line: 1,
            end_line: 1,
            start_column: 0,
            end_column: 5,
            byte_range: 0..5,
            text: "hello".to_string(),
        };
        let m2 = Match {
            start_line: 1,
            end_line: 1,
            start_column: 0,
            end_column: 5,
            byte_range: 0..5,
            text: "hello".to_string(),
        };
        let m3 = Match {
            start_line: 2,
            end_line: 2,
            start_column: 0,
            end_column: 5,
            byte_range: 0..5,
            text: "hello".to_string(),
        };
        assert_eq!(m1, m2);
        assert_ne!(m1, m3);
    }

    #[test]
    fn test_ranges_to_matches_bounds_and_conversion() {
        let content = "line1\nline2\nline3";
        let ranges = vec![TsRange {
            start_byte: 0,
            end_byte: 5,
            start_point: tree_sitter::Point::new(0, 0),
            end_point: tree_sitter::Point::new(0, 5),
        }];

        let matches = ranges_to_matches(content, &ranges);
        assert_eq!(matches.len(), 1);
        let m = &matches[0];
        assert_eq!(m.start_line, 1);
        assert_eq!(m.end_line, 1);
        assert_eq!(m.start_column, 0);
        assert_eq!(m.end_column, 5);
        assert_eq!(m.byte_range, 0..5);
        assert_eq!(m.text, "line1");
    }

    #[test]
    fn test_ranges_to_matches_filters_invalid_ranges() {
        let content = "short";
        let ranges = vec![TsRange {
            start_byte: 0,
            end_byte: 100,
            start_point: tree_sitter::Point::new(0, 0),
            end_point: tree_sitter::Point::new(0, 100),
        }];

        let matches = ranges_to_matches(content, &ranges);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_ranges_to_matches_unicode_boundaries() {
        let content = "fn 你好() {}";
        let ranges = vec![TsRange {
            start_byte: 3,
            end_byte: 9,
            start_point: tree_sitter::Point::new(0, 3),
            end_point: tree_sitter::Point::new(0, 9),
        }];

        let matches = ranges_to_matches(content, &ranges);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "你好");
        assert_eq!(matches[0].start_line, 1);
        assert_eq!(matches[0].end_line, 1);
    }

    #[test]
    fn test_ranges_to_matches_empty_content() {
        let content = "";
        let ranges = vec![TsRange {
            start_byte: 0,
            end_byte: 0,
            start_point: tree_sitter::Point::new(0, 0),
            end_point: tree_sitter::Point::new(0, 0),
        }];

        let matches = ranges_to_matches(content, &ranges);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_ranges_to_matches_empty_ranges() {
        let content = "some content";
        let ranges: Vec<TsRange> = vec![];

        let matches = ranges_to_matches(content, &ranges);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_ranges_to_matches_single_character() {
        let content = "x";
        let ranges = vec![TsRange {
            start_byte: 0,
            end_byte: 1,
            start_point: tree_sitter::Point::new(0, 0),
            end_point: tree_sitter::Point::new(0, 1),
        }];

        let matches = ranges_to_matches(content, &ranges);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "x");
        assert_eq!(matches[0].byte_range, 0..1);
        assert_eq!(matches[0].start_column, 0);
        assert_eq!(matches[0].end_column, 1);
    }

    #[test]
    fn test_ranges_to_matches_multiline_and_columns() {
        let content = "fn main() {\n    println!(\"hello\");\n}";
        let ranges = vec![TsRange {
            start_byte: 0,
            end_byte: content.len(),
            start_point: tree_sitter::Point::new(0, 0),
            end_point: tree_sitter::Point::new(2, 1),
        }];

        let matches = ranges_to_matches(content, &ranges);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].start_line, 1);
        assert_eq!(matches[0].end_line, 3);
        assert_eq!(matches[0].start_column, 0);
        assert_eq!(matches[0].end_column, 1);
        assert_eq!(matches[0].byte_range, 0..content.len());
    }

    #[test]
    fn test_search_result_iterator_size_hint_and_remaining() {
        let results = vec![
            (PathBuf::from("file1.txt"), vec![]),
            (PathBuf::from("file2.txt"), vec![]),
        ];
        let mut iter = SearchResultIterator::new(results);

        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.remaining(), 2);
        iter.next();
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.remaining(), 1);
    }

    #[test]
    fn test_search_result_iterator_error_contains_path_and_continues() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_ok = dir.path().join("ok.txt");
        fs::write(&file_ok, "ok").unwrap();
        let missing = dir.path().join("missing.txt");

        let results = vec![(file_ok.clone(), vec![]), (missing.clone(), vec![])];
        let mut iter = SearchResultIterator::new(results);

        let first = iter.next().unwrap();
        assert!(first.is_ok());

        let second = iter.next().unwrap();
        assert!(second.is_err());
        let msg = second.unwrap_err().to_string();
        assert!(msg.contains("missing.txt"));

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_search_result_iterator_take_stops_early() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        for i in 0..3 {
            let path = dir.path().join(format!("file{i}.txt"));
            fs::write(&path, format!("content{i}")).unwrap();
        }

        let results: Vec<_> = (0..3)
            .map(|i| (dir.path().join(format!("file{i}.txt")), vec![]))
            .collect();

        let taken: Vec<_> = SearchResultIterator::new(results).take(1).collect();
        assert_eq!(taken.len(), 1);
        assert!(taken[0].is_ok());
    }

    #[test]
    fn test_iterator_whole_file_match_loads_content() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("file.txt");
        fs::write(&file, "hello world").unwrap();

        let mut iter = SearchResultIterator::new(vec![(file.clone(), vec![])]);
        let result = iter.next().unwrap().unwrap();

        assert!(result.is_whole_file_match());
        assert!(result.matches.is_empty());
        assert_eq!(result.content, "hello world");
    }

    #[test]
    fn test_iterator_large_file_error() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("large.txt");
        let bytes = vec![b'x'; (MAX_FILE_SIZE + 1) as usize];
        fs::write(&file, &bytes).unwrap();

        let mut iter = SearchResultIterator::new(vec![(file.clone(), vec![])]);
        let err = iter.next().unwrap().unwrap_err().to_string();

        assert!(err.contains("exceeds maximum size limit"));
        assert!(err.contains(file.to_string_lossy().as_ref()));
    }

    #[test]
    fn test_iterator_missing_file_error() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("missing.txt");
        fs::write(&file, "content").unwrap();

        let mut iter = SearchResultIterator::new(vec![(file.clone(), vec![])]);
        fs::remove_file(&file).unwrap();

        let err = iter.next().unwrap().unwrap_err().to_string();
        assert!(err.contains("no longer exists"));
        assert!(err.contains("missing.txt"));
    }

    #[test]
    fn test_iterator_permission_error() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("restricted.txt");
        fs::write(&file, "content").unwrap();
        let permissions = fs::metadata(&file).unwrap().permissions();
        fs::set_permissions(&file, PermissionsExt::from_mode(0o000)).unwrap();

        let mut iter = SearchResultIterator::new(vec![(file.clone(), vec![])]);
        let err = iter.next().unwrap().unwrap_err().to_string();
        assert!(err.contains("Permission denied reading"));

        fs::set_permissions(&file, permissions).unwrap();
    }

    #[test]
    fn test_iterator_binary_file_error() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("binary.bin");
        fs::write(&file, b"\x00\x01\x02binary").unwrap();

        let mut iter = SearchResultIterator::new(vec![(file.clone(), vec![])]);
        let err = iter.next().unwrap().unwrap_err().to_string();
        assert!(err.contains("binary file"));
    }

    #[test]
    fn test_iterator_invalid_utf8_error() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("invalid.txt");
        fs::write(&file, vec![0xff, 0xfe]).unwrap();

        let mut iter = SearchResultIterator::new(vec![(file.clone(), vec![])]);
        let err = iter.next().unwrap().unwrap_err().to_string();
        assert!(err.contains("Invalid UTF-8"));
        assert!(err.contains("invalid.txt"));
    }

    #[test]
    fn test_iterator_unicode_matches_multibyte() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("unicode.txt");
        let content = "fn 你好() {}";
        fs::write(&file, content).unwrap();
        let ranges = vec![TsRange {
            start_byte: 3,
            end_byte: 9,
            start_point: tree_sitter::Point::new(0, 3),
            end_point: tree_sitter::Point::new(0, 9),
        }];

        let mut iter = SearchResultIterator::new(vec![(file, ranges)]);
        let result = iter.next().unwrap().unwrap();

        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].text, "你好");
        assert_eq!(result.matches[0].start_line, 1);
        assert_eq!(result.matches[0].end_line, 1);
    }

    #[test]
    fn test_iterator_continues_after_error() {
        let dir = tempdir().unwrap();
        let existing = dir.path().join("good.txt");
        let missing = dir.path().join("missing.txt");
        fs::write(&existing, "ok").unwrap();

        let mut iter =
            SearchResultIterator::new(vec![(missing.clone(), vec![]), (existing.clone(), vec![])]);
        let first = iter.next().unwrap();
        let second = iter.next().unwrap();

        assert!(first.is_err());
        assert!(second.is_ok());
    }
}
