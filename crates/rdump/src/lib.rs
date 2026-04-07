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
pub mod backend;
pub mod commands;
pub mod config;
pub mod content;
mod engine;
pub mod evaluator;
mod execution;
pub mod formatter;
pub mod limits;
pub mod parser;
pub mod planner;
pub mod predicates;
pub mod request;
pub mod support_matrix;

use anyhow::Result;
#[cfg(feature = "cli")]
use clap::{Parser, Subcommand, ValueEnum};
pub use rdump_contracts as contracts;
use rdump_contracts::{ErrorMode, ExecutionProfile, SemanticMatchMode, SnippetMode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
#[cfg(unix)]
use std::path::PathBuf;
use std::sync::Arc;

// =============================================================================
// Library API Exports
// =============================================================================

pub use crate::planner::{
    explain_query, explain_query_with_runtime, repo_language_inventory,
    repo_language_inventory_with_runtime, serialize_query_ast, simplify_query, PredicatePlan,
    QueryExplanation, QueryPreflight, QueryStage, RepoLanguageCount, StableAstNode,
};
/// SQL dialect used for SQL-aware searches; re-exported so callers can configure
/// dialects without reaching into internal modules.
pub use crate::predicates::code_aware::SqlDialect;
#[cfg(feature = "async")]
pub use async_api::{
    search_all_async, search_all_async_with_runtime, search_async, search_async_with_progress,
    search_async_with_runtime, search_async_with_runtime_and_progress,
};
pub use backend::{
    BackendFileType, BackendMetadata, BackendPathIdentity, DiscoveryReport, DiscoveryRequest,
    RealFsSearchBackend, SearchBackend, SearchRuntime,
};
pub use execution::{
    default_max_concurrent_searches, search_execution_policy, CancelOnDrop,
    SearchCancellationToken, SearchExecutionPolicy,
};
pub use request::{
    capability_metadata, classify_error as classify_contract_error, contract_error,
    default_limits as default_contract_limits, execute_search_request,
    execute_search_request_with_progress, execute_search_request_with_progress_and_cancellation,
    execute_search_request_with_runtime, execute_search_request_with_runtime_and_cancellation,
    execute_search_request_with_runtime_and_progress,
    format_search_text as format_contract_search_text, search_options_from_request,
};

// Bring our command functions into scope
pub use crate::content::{ContentSkipReason, ContentState, SearchDiagnostic};
use crate::predicates::code_aware::SqlDialect as CodeSqlDialect;
#[cfg(feature = "cli")]
use commands::{config::run_config, query::run_query};
#[cfg(feature = "cli")]
use commands::{lang::run_lang, preset::run_preset, search::run_search};
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

    /// If true, fail immediately when a dialect-specific SQL parser fails instead of falling back to generic SQL.
    pub sql_strict: bool,

    /// How per-file materialization failures should be handled by higher-level helpers.
    pub error_mode: ErrorMode,

    /// End-to-end search time budget in milliseconds.
    pub execution_budget_ms: Option<u64>,

    /// Per-file semantic evaluation budget in milliseconds.
    pub semantic_budget_ms: Option<u64>,

    /// Maximum semantic captures to retain for a single file.
    pub max_semantic_matches_per_file: Option<usize>,

    /// Override language selection for extensionless or ambiguous files.
    pub language_override: Option<String>,

    /// Matching behavior for semantic identifier-like predicates.
    pub semantic_match_mode: SemanticMatchMode,

    /// Snippet shaping policy for line endings.
    pub snippet_mode: SnippetMode,

    /// If true, semantic parse failures are surfaced as hard errors.
    pub semantic_strict: bool,

    /// If true, path canonicalization fallback becomes a hard error.
    pub strict_path_resolution: bool,

    /// If true, record file metadata during evaluation and emit drift diagnostics during later materialization.
    pub snapshot_drift_detection: bool,

    /// Bundled operational defaults for different execution environments.
    pub execution_profile: Option<ExecutionProfile>,

    /// If true, emit best-effort diagnostics describing why paths were excluded by ignore handling.
    pub ignore_debug: bool,

    /// If true, emit diagnostics describing semantic language-profile selection.
    pub language_debug: bool,

    /// If true, emit SQL dialect heuristic traces for `.sql` files.
    pub sql_trace: bool,
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
            sql_strict: false,
            error_mode: ErrorMode::SkipErrors,
            execution_budget_ms: None,
            semantic_budget_ms: None,
            max_semantic_matches_per_file: None,
            language_override: None,
            semantic_match_mode: SemanticMatchMode::Exact,
            snippet_mode: SnippetMode::PreserveLineEndings,
            semantic_strict: false,
            strict_path_resolution: false,
            snapshot_drift_detection: true,
            execution_profile: None,
            ignore_debug: false,
            language_debug: false,
            sql_trace: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchOptionsBuilder {
    options: SearchOptions,
}

impl SearchOptions {
    pub fn builder() -> SearchOptionsBuilder {
        SearchOptionsBuilder {
            options: SearchOptions::default(),
        }
    }
}

impl SearchOptionsBuilder {
    pub fn root(mut self, root: impl Into<PathBuf>) -> Self {
        self.options.root = root.into();
        self
    }

    pub fn presets(mut self, presets: Vec<String>) -> Self {
        self.options.presets = presets;
        self
    }

    pub fn no_ignore(mut self, no_ignore: bool) -> Self {
        self.options.no_ignore = no_ignore;
        self
    }

    pub fn hidden(mut self, hidden: bool) -> Self {
        self.options.hidden = hidden;
        self
    }

    pub fn max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.options.max_depth = max_depth;
        self
    }

    pub fn sql_dialect(mut self, sql_dialect: Option<SqlDialect>) -> Self {
        self.options.sql_dialect = sql_dialect;
        self
    }

    pub fn sql_strict(mut self, sql_strict: bool) -> Self {
        self.options.sql_strict = sql_strict;
        self
    }

    pub fn error_mode(mut self, error_mode: ErrorMode) -> Self {
        self.options.error_mode = error_mode;
        self
    }

    pub fn execution_budget_ms(mut self, execution_budget_ms: Option<u64>) -> Self {
        self.options.execution_budget_ms = execution_budget_ms;
        self
    }

    pub fn semantic_budget_ms(mut self, semantic_budget_ms: Option<u64>) -> Self {
        self.options.semantic_budget_ms = semantic_budget_ms;
        self
    }

    pub fn max_semantic_matches_per_file(
        mut self,
        max_semantic_matches_per_file: Option<usize>,
    ) -> Self {
        self.options.max_semantic_matches_per_file = max_semantic_matches_per_file;
        self
    }

    pub fn language_override(mut self, language_override: Option<String>) -> Self {
        self.options.language_override = language_override;
        self
    }

    pub fn semantic_match_mode(mut self, semantic_match_mode: SemanticMatchMode) -> Self {
        self.options.semantic_match_mode = semantic_match_mode;
        self
    }

    pub fn snippet_mode(mut self, snippet_mode: SnippetMode) -> Self {
        self.options.snippet_mode = snippet_mode;
        self
    }

    pub fn semantic_strict(mut self, semantic_strict: bool) -> Self {
        self.options.semantic_strict = semantic_strict;
        self
    }

    pub fn strict_path_resolution(mut self, strict_path_resolution: bool) -> Self {
        self.options.strict_path_resolution = strict_path_resolution;
        self
    }

    pub fn snapshot_drift_detection(mut self, snapshot_drift_detection: bool) -> Self {
        self.options.snapshot_drift_detection = snapshot_drift_detection;
        self
    }

    pub fn execution_profile(mut self, execution_profile: Option<ExecutionProfile>) -> Self {
        self.options.execution_profile = execution_profile;
        self
    }

    pub fn ignore_debug(mut self, ignore_debug: bool) -> Self {
        self.options.ignore_debug = ignore_debug;
        self
    }

    pub fn language_debug(mut self, language_debug: bool) -> Self {
        self.options.language_debug = language_debug;
        self
    }

    pub fn sql_trace(mut self, sql_trace: bool) -> Self {
        self.options.sql_trace = sql_trace;
        self
    }

    pub fn build(self) -> SearchOptions {
        self.options
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ResultKind {
    #[default]
    WholeFile,
    Ranged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PathResolution {
    #[default]
    Canonical,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FileIdentity {
    pub display_path: PathBuf,
    pub resolved_path: PathBuf,
    pub root_relative_path: Option<PathBuf>,
    pub resolution: PathResolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SemanticSkipReason {
    UnsupportedLanguage,
    ParseFailed,
    ContentUnavailable,
    BudgetExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FileSnapshot {
    pub len: u64,
    pub modified_unix_millis: Option<i64>,
    pub readonly: bool,
    pub permissions_display: String,
    pub stable_token: Option<String>,
    pub device_id: Option<u64>,
    pub inode: Option<u64>,
}

impl FileSnapshot {
    /// Builds a snapshot from backend-neutral metadata.
    ///
    /// Adapter authors should prefer this constructor so snapshot identity and
    /// drift checks stay decoupled from `std::fs::Metadata`.
    pub fn from_backend_metadata(metadata: &crate::backend::BackendMetadata) -> Self {
        Self {
            len: metadata.size_bytes,
            modified_unix_millis: metadata.modified_unix_millis,
            readonly: metadata.readonly,
            permissions_display: metadata.permissions_display.clone(),
            stable_token: metadata.stable_token.clone(),
            device_id: metadata.device_id,
            inode: metadata.inode,
        }
    }

    #[deprecated(
        since = "0.1.10",
        note = "prefer FileSnapshot::from_backend_metadata(...) so snapshot creation stays backend-neutral"
    )]
    pub fn from_metadata(metadata: &std::fs::Metadata) -> Self {
        Self::from_backend_metadata(&crate::backend::backend_metadata_from_std(metadata))
    }

    pub fn to_path_metadata(&self) -> rdump_contracts::PathMetadata {
        rdump_contracts::PathMetadata {
            size_bytes: self.len,
            modified_unix_millis: self.modified_unix_millis,
            readonly: self.readonly,
            permissions_display: self.permissions_display.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SearchResultMetadata {
    pub file: FileIdentity,
    pub fingerprint: String,
    pub result_kind: ResultKind,
    pub semantic_skip_reasons: Vec<SemanticSkipReason>,
    pub snapshot: Option<FileSnapshot>,
    pub snapshot_drift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchMaterializationFailureKind {
    ContentReadFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchMaterializationError {
    pub path: PathBuf,
    pub kind: SearchMaterializationFailureKind,
    pub snapshot_drift: bool,
    pub message: String,
}

impl std::fmt::Display for SearchMaterializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to materialize {} ({:?}, snapshot_drift={}): {}",
            self.path.display(),
            self.kind,
            self.snapshot_drift,
            self.message
        )
    }
}

impl std::error::Error for SearchMaterializationError {}

/// A file that matched the search query.
///
/// Contains the file path, all matches within the file, and the file content.
/// For whole-file matches (boolean predicates like `ext:rs`), the `matches`
/// vector will be empty.
///
/// Snapshot note: discovery/evaluation and content materialization are not a
/// single filesystem snapshot. If a file changes between those stages, the
/// content and diagnostics reflect the later read.
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
///     content_state: rdump::ContentState::Loaded,
///     diagnostics: vec![],
///     metadata: rdump::SearchResultMetadata::default(),
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
///     content_state: rdump::ContentState::Loaded,
///     diagnostics: vec![],
///     metadata: rdump::SearchResultMetadata::default(),
/// };
/// assert_eq!(hunked.matched_lines(), vec![3, 4]);
/// assert_eq!(hunked.match_count(), 1);
/// assert_eq!(hunked.total_lines_matched(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Path to the matched file.
    pub path: PathBuf,

    /// Matches within this file (empty for whole-file matches).
    pub matches: Vec<Match>,

    /// Full file content.
    pub content: String,

    /// How the content field was produced for this file.
    pub content_state: ContentState,

    /// Structured warnings collected while loading or shaping this result.
    pub diagnostics: Vec<SearchDiagnostic>,

    /// Stable machine-facing metadata for path identity, match semantics, and snapshot context.
    pub metadata: SearchResultMetadata,
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

    /// Returns true if the content field contains user-visible file text.
    pub fn content_available(&self) -> bool {
        self.content_state.is_loaded()
    }

    pub fn result_kind(&self) -> ResultKind {
        self.metadata.result_kind
    }

    pub fn file_identity(&self) -> &FileIdentity {
        &self.metadata.file
    }

    pub fn semantic_skip_reasons(&self) -> &[SemanticSkipReason] {
        &self.metadata.semantic_skip_reasons
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Summary statistics produced by the core search engine.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchStats {
    pub whole_file_results: usize,
    pub ranged_results: usize,
    pub candidate_files: usize,
    pub prefiltered_files: usize,
    pub evaluated_files: usize,
    pub matched_files: usize,
    pub matched_ranges: usize,
    pub hidden_skipped: usize,
    pub ignore_skipped: usize,
    pub max_depth_skipped: usize,
    pub unreadable_entries: usize,
    pub root_boundary_excluded: usize,
    pub suppressed_too_large: usize,
    pub suppressed_binary: usize,
    pub suppressed_secret_like: usize,
    pub diagnostics: usize,
    pub walk_millis: u64,
    pub prefilter_millis: u64,
    pub evaluate_millis: u64,
    pub materialize_millis: u64,
    pub semantic_parse_failures: usize,
    pub semantic_budget_exhaustions: usize,
    pub query_cache_hits: usize,
    pub query_cache_misses: usize,
    pub tree_cache_hits: usize,
    pub tree_cache_misses: usize,
    pub semaphore_wait_millis: u64,
    pub semantic_parse_failures_by_language: BTreeMap<String, usize>,
    pub directory_hotspots: Vec<rdump_contracts::DirectoryHotspot>,
}

/// Collected search results plus engine-level statistics and diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchReport {
    pub results: Vec<SearchResult>,
    pub stats: SearchStats,
    pub diagnostics: Vec<SearchDiagnostic>,
}

impl SearchReport {
    pub fn status(&self) -> rdump_contracts::SearchStatus {
        if !self.results.is_empty()
            && self
                .results
                .iter()
                .all(|result| !result.content_available())
        {
            return rdump_contracts::SearchStatus::PolicySuppressed;
        }
        if self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == crate::content::DiagnosticKind::ContentSkipped)
        {
            return rdump_contracts::SearchStatus::PartialSuccess;
        }
        rdump_contracts::SearchStatus::FullSuccess
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

pub(crate) fn materialize_raw_search_item(raw: Result<RawSearchItem>) -> Result<SearchResult> {
    let raw = raw?;
    let loaded = match crate::content::load_search_content_with_backend(
        raw.backend.as_ref(),
        &raw.resolved_path,
        &raw.display_path,
    ) {
        Ok(c) => c,
        Err(e) => {
            let snapshot_drift = raw.snapshot.is_some();
            let message = if snapshot_drift {
                format!(
                    "Materialization for {} may have drifted since evaluation: {e}",
                    raw.resolved_path.display()
                )
            } else {
                e.to_string()
            };
            return Err(SearchMaterializationError {
                path: raw.display_path,
                kind: SearchMaterializationFailureKind::ContentReadFailed,
                snapshot_drift,
                message,
            }
            .into());
        }
    };

    let content = loaded.content.as_ref().to_string();
    let matches = if raw.ranges.is_empty() || !loaded.state.is_loaded() {
        Vec::new()
    } else {
        ranges_to_matches(&content, &raw.ranges)
    };

    let mut diagnostics = raw.diagnostics;
    diagnostics.extend(loaded.diagnostics);
    let mut snapshot_drift = false;
    if let Some(snapshot) = &raw.snapshot {
        if let Ok(metadata) = raw.backend.stat(&raw.resolved_path) {
            let current_snapshot = FileSnapshot::from_backend_metadata(&metadata);
            if current_snapshot != *snapshot {
                snapshot_drift = true;
                diagnostics.push(SearchDiagnostic::snapshot_drift(
                    raw.display_path.clone(),
                    "File identity or metadata changed between evaluation and content materialization.",
                ));
            }
        }
    }

    let result_kind = if matches.is_empty() {
        ResultKind::WholeFile
    } else {
        ResultKind::Ranged
    };

    Ok(SearchResult {
        path: raw.display_path.clone(),
        matches,
        content,
        content_state: loaded.state,
        diagnostics,
        metadata: SearchResultMetadata {
            fingerprint: result_fingerprint(
                &raw.display_path,
                &raw.resolved_path,
                result_kind,
                raw.snapshot.as_ref(),
                &raw.semantic_skip_reasons,
            ),
            file: FileIdentity {
                display_path: raw.display_path,
                resolved_path: raw.resolved_path,
                root_relative_path: raw.root_relative_path,
                resolution: raw.resolution,
            },
            result_kind,
            semantic_skip_reasons: raw.semantic_skip_reasons,
            snapshot: raw.snapshot,
            snapshot_drift,
        },
    })
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
#[derive(Debug)]
pub struct SearchResultIterator {
    inner: SearchResultIteratorInner,
}

#[derive(Debug)]
enum SearchResultIteratorInner {
    #[cfg_attr(not(test), allow(dead_code))]
    Buffered {
        inner: std::vec::IntoIter<RawSearchItem>,
        stats: SearchStats,
        diagnostics: Vec<SearchDiagnostic>,
    },
    Raw(engine::SearchRawIterator),
}

#[derive(Debug, Clone)]
pub(crate) struct RawSearchItem {
    pub backend: Arc<dyn crate::backend::SearchBackend>,
    pub display_path: PathBuf,
    pub resolved_path: PathBuf,
    pub root_relative_path: Option<PathBuf>,
    pub resolution: PathResolution,
    pub ranges: Vec<TsRange>,
    pub diagnostics: Vec<SearchDiagnostic>,
    pub semantic_skip_reasons: Vec<SemanticSkipReason>,
    pub snapshot: Option<FileSnapshot>,
}

impl SearchResultIterator {
    /// Create a new iterator from raw search results.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn new(
        results: Vec<RawSearchItem>,
        stats: SearchStats,
        diagnostics: Vec<SearchDiagnostic>,
    ) -> Self {
        Self {
            inner: SearchResultIteratorInner::Buffered {
                inner: results.into_iter(),
                stats,
                diagnostics,
            },
        }
    }

    pub(crate) fn from_raw_iter(iter: engine::SearchRawIterator) -> Self {
        Self {
            inner: SearchResultIteratorInner::Raw(iter),
        }
    }

    /// Get an upper bound on the remaining results without advancing the iterator.
    pub fn remaining(&self) -> usize {
        match &self.inner {
            SearchResultIteratorInner::Buffered { inner, .. } => inner.len(),
            SearchResultIteratorInner::Raw(iter) => iter.remaining_hint(),
        }
    }

    /// Returns search-engine statistics gathered before result materialization.
    pub fn stats(&self) -> &SearchStats {
        match &self.inner {
            SearchResultIteratorInner::Buffered { stats, .. } => stats,
            SearchResultIteratorInner::Raw(iter) => iter.stats(),
        }
    }

    /// Returns engine-level diagnostics gathered before result materialization.
    pub fn diagnostics(&self) -> &[SearchDiagnostic] {
        match &self.inner {
            SearchResultIteratorInner::Buffered { diagnostics, .. } => diagnostics,
            SearchResultIteratorInner::Raw(iter) => iter.diagnostics(),
        }
    }

    #[cfg_attr(not(feature = "async"), allow(dead_code))]
    pub(crate) fn was_cancelled(&self) -> bool {
        match &self.inner {
            SearchResultIteratorInner::Buffered { .. } => false,
            SearchResultIteratorInner::Raw(iter) => iter.was_cancelled(),
        }
    }

    #[cfg(test)]
    pub(crate) fn buffered_raw_items_mut(&mut self) -> Option<&mut [RawSearchItem]> {
        match &mut self.inner {
            SearchResultIteratorInner::Buffered { inner, .. } => Some(inner.as_mut_slice()),
            SearchResultIteratorInner::Raw(_) => None,
        }
    }
}

impl Iterator for SearchResultIterator {
    type Item = Result<SearchResult>;

    fn next(&mut self) -> Option<Self::Item> {
        let raw = match &mut self.inner {
            SearchResultIteratorInner::Buffered { inner, .. } => inner.next().map(Ok)?,
            SearchResultIteratorInner::Raw(iter) => iter.next()?,
        };
        Some(materialize_raw_search_item(raw))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            SearchResultIteratorInner::Buffered { inner, .. } => {
                let len = inner.len();
                (len, Some(len))
            }
            SearchResultIteratorInner::Raw(iter) => iter.size_hint(),
        }
    }
}

fn result_fingerprint(
    display_path: &PathBuf,
    resolved_path: &PathBuf,
    result_kind: ResultKind,
    snapshot: Option<&FileSnapshot>,
    semantic_skip_reasons: &[SemanticSkipReason],
) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    display_path.hash(&mut hasher);
    resolved_path.hash(&mut hasher);
    result_kind.hash(&mut hasher);
    if let Some(snapshot) = snapshot {
        snapshot.len.hash(&mut hasher);
        snapshot.modified_unix_millis.hash(&mut hasher);
        snapshot.readonly.hash(&mut hasher);
        snapshot.permissions_display.hash(&mut hasher);
        snapshot.stable_token.hash(&mut hasher);
        snapshot.device_id.hash(&mut hasher);
        snapshot.inode.hash(&mut hasher);
    }
    for reason in semantic_skip_reasons {
        reason.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

impl std::iter::FusedIterator for SearchResultIterator {}

/// Iterator over matching file paths without content materialization.
#[derive(Debug)]
pub struct SearchPathIterator {
    inner: SearchPathIteratorInner,
}

#[derive(Debug)]
enum SearchPathIteratorInner {
    #[allow(dead_code)]
    Buffered {
        inner: std::vec::IntoIter<PathBuf>,
        stats: SearchStats,
        diagnostics: Vec<SearchDiagnostic>,
    },
    Raw(engine::SearchRawIterator),
}

impl SearchPathIterator {
    #[allow(dead_code)]
    pub(crate) fn new(
        paths: Vec<PathBuf>,
        stats: SearchStats,
        diagnostics: Vec<SearchDiagnostic>,
    ) -> Self {
        Self {
            inner: SearchPathIteratorInner::Buffered {
                inner: paths.into_iter(),
                stats,
                diagnostics,
            },
        }
    }

    pub(crate) fn from_raw_iter(iter: engine::SearchRawIterator) -> Self {
        Self {
            inner: SearchPathIteratorInner::Raw(iter),
        }
    }

    pub fn remaining(&self) -> usize {
        match &self.inner {
            SearchPathIteratorInner::Buffered { inner, .. } => inner.len(),
            SearchPathIteratorInner::Raw(iter) => iter.remaining_hint(),
        }
    }

    pub fn stats(&self) -> &SearchStats {
        match &self.inner {
            SearchPathIteratorInner::Buffered { stats, .. } => stats,
            SearchPathIteratorInner::Raw(iter) => iter.stats(),
        }
    }

    pub fn diagnostics(&self) -> &[SearchDiagnostic] {
        match &self.inner {
            SearchPathIteratorInner::Buffered { diagnostics, .. } => diagnostics,
            SearchPathIteratorInner::Raw(iter) => iter.diagnostics(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn was_cancelled(&self) -> bool {
        match &self.inner {
            SearchPathIteratorInner::Buffered { .. } => false,
            SearchPathIteratorInner::Raw(iter) => iter.was_cancelled(),
        }
    }
}

impl Iterator for SearchPathIterator {
    type Item = Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            SearchPathIteratorInner::Buffered { inner, .. } => inner.next().map(Ok),
            SearchPathIteratorInner::Raw(iter) => match iter.next()? {
                Ok(raw) => Some(Ok(raw.display_path)),
                Err(err) => Some(Err(err)),
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            SearchPathIteratorInner::Buffered { inner, .. } => {
                let len = inner.len();
                (len, Some(len))
            }
            SearchPathIteratorInner::Raw(iter) => iter.size_hint(),
        }
    }
}

impl std::iter::FusedIterator for SearchPathIterator {}

// =============================================================================
// Library API Functions
// =============================================================================

/// Run an rdump query and stream results lazily.
///
/// This is the preferred API for large codebases: it parses the query, resolves
/// presets, validates the root, and returns an iterator that walks candidates,
/// evaluates predicates, and loads file content incrementally as items are
/// consumed.
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
    SearchRuntime::real_fs().search_iter(query, &options)
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
    SearchRuntime::real_fs().search(query, &options)
}

/// Run an rdump query and return results plus engine-level statistics.
pub fn search_with_stats(query: &str, options: SearchOptions) -> Result<SearchReport> {
    SearchRuntime::real_fs().search_with_stats(query, &options)
}

/// Run an rdump query and stream only matching paths.
pub fn search_path_iter(query: &str, options: SearchOptions) -> Result<SearchPathIterator> {
    SearchRuntime::real_fs().search_path_iter(query, &options)
}

/// Run an rdump query and collect only matching paths.
pub fn search_paths(query: &str, options: SearchOptions) -> Result<Vec<PathBuf>> {
    search_path_iter(query, options)?.collect()
}

// =============================================================================
// CLI API
// =============================================================================

// These structs and enums define the public API of our CLI.
// They need to be public so the `commands` modules can use them.
#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
#[cfg_attr(
    feature = "cli",
    command(
        version,
        about = "A fast, expressive, code-aware tool to find and dump file contents."
    )
)]
pub struct Cli {
    #[cfg_attr(feature = "cli", command(subcommand))]
    pub command: Commands,
}

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Subcommand))]
pub enum Commands {
    /// Search for files using a query (default command).
    #[cfg_attr(feature = "cli", command(visible_alias = "s"))]
    Search(SearchArgs),
    /// Inspect query expansion, validation, and normalization.
    #[cfg_attr(feature = "cli", command(visible_alias = "q"))]
    Query(QueryArgs),
    /// Inspect config resolution and merged config state.
    #[cfg_attr(feature = "cli", command(visible_alias = "cfg"))]
    Config(ConfigArgs),
    /// List supported languages and their available predicates.
    #[cfg_attr(feature = "cli", command(visible_alias = "l"))]
    Lang(LangArgs),
    /// Manage saved presets.
    #[cfg_attr(feature = "cli", command(visible_alias = "p"))]
    Preset(PresetArgs),
}

#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum ColorChoice {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum TimeFormat {
    #[default]
    Local,
    Utc,
    Iso,
    Unix,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum SqlDialectFlag {
    Generic,
    Postgres,
    Mysql,
    Sqlite,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum SemanticMatchModeFlag {
    #[default]
    Exact,
    CaseInsensitive,
    Prefix,
    Regex,
    Wildcard,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum ExecutionProfileFlag {
    Interactive,
    Batch,
    Agent,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum PathDisplayModeFlag {
    #[default]
    Relative,
    Absolute,
    RootRelative,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum LineEndingModeFlag {
    #[default]
    Preserve,
    Normalize,
}

impl From<SemanticMatchModeFlag> for rdump_contracts::SemanticMatchMode {
    fn from(value: SemanticMatchModeFlag) -> Self {
        match value {
            SemanticMatchModeFlag::Exact => rdump_contracts::SemanticMatchMode::Exact,
            SemanticMatchModeFlag::CaseInsensitive => {
                rdump_contracts::SemanticMatchMode::CaseInsensitive
            }
            SemanticMatchModeFlag::Prefix => rdump_contracts::SemanticMatchMode::Prefix,
            SemanticMatchModeFlag::Regex => rdump_contracts::SemanticMatchMode::Regex,
            SemanticMatchModeFlag::Wildcard => rdump_contracts::SemanticMatchMode::Wildcard,
        }
    }
}

impl From<ExecutionProfileFlag> for rdump_contracts::ExecutionProfile {
    fn from(value: ExecutionProfileFlag) -> Self {
        match value {
            ExecutionProfileFlag::Interactive => rdump_contracts::ExecutionProfile::Interactive,
            ExecutionProfileFlag::Batch => rdump_contracts::ExecutionProfile::Batch,
            ExecutionProfileFlag::Agent => rdump_contracts::ExecutionProfile::Agent,
        }
    }
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

impl From<SqlDialectFlag> for rdump_contracts::SqlDialectOption {
    fn from(value: SqlDialectFlag) -> Self {
        match value {
            SqlDialectFlag::Generic => rdump_contracts::SqlDialectOption::Generic,
            SqlDialectFlag::Postgres => rdump_contracts::SqlDialectOption::Postgres,
            SqlDialectFlag::Mysql => rdump_contracts::SqlDialectOption::Mysql,
            SqlDialectFlag::Sqlite => rdump_contracts::SqlDialectOption::Sqlite,
        }
    }
}

impl From<PathDisplayModeFlag> for rdump_contracts::PathDisplayMode {
    fn from(value: PathDisplayModeFlag) -> Self {
        match value {
            PathDisplayModeFlag::Relative => rdump_contracts::PathDisplayMode::Relative,
            PathDisplayModeFlag::Absolute => rdump_contracts::PathDisplayMode::Absolute,
            PathDisplayModeFlag::RootRelative => rdump_contracts::PathDisplayMode::RootRelative,
        }
    }
}

impl From<LineEndingModeFlag> for rdump_contracts::LineEndingMode {
    fn from(value: LineEndingModeFlag) -> Self {
        match value {
            LineEndingModeFlag::Preserve => rdump_contracts::LineEndingMode::Preserve,
            LineEndingModeFlag::Normalize => rdump_contracts::LineEndingMode::Normalize,
        }
    }
}

impl From<rdump_contracts::SqlDialectOption> for CodeSqlDialect {
    fn from(value: rdump_contracts::SqlDialectOption) -> Self {
        match value {
            rdump_contracts::SqlDialectOption::Generic => CodeSqlDialect::Generic,
            rdump_contracts::SqlDialectOption::Postgres => CodeSqlDialect::Postgres,
            rdump_contracts::SqlDialectOption::Mysql => CodeSqlDialect::Mysql,
            rdump_contracts::SqlDialectOption::Sqlite => CodeSqlDialect::Sqlite,
        }
    }
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "cli", derive(Parser))]
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
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment, name = "QUERY"))]
    pub query: Option<String>,
    /// Force the SQL dialect to use for .sql files (overrides auto-detection).
    #[cfg_attr(feature = "cli", arg(long, value_enum, ignore_case = true))]
    pub dialect: Option<SqlDialectFlag>,
    #[cfg_attr(feature = "cli", arg(long))]
    pub sql_strict: bool,
    #[cfg_attr(feature = "cli", arg(long, short))]
    pub preset: Vec<String>,
    #[cfg_attr(feature = "cli", arg(short, long, default_value = "."))]
    pub root: PathBuf,
    #[cfg_attr(feature = "cli", arg(short, long))]
    pub output: Option<PathBuf>,
    #[cfg_attr(feature = "cli", arg(short, long))]
    pub line_numbers: bool,
    #[cfg_attr(
        feature = "cli",
        arg(long, help = "Alias for --format=cat, useful for piping")
    )]
    pub no_headers: bool,
    #[cfg_attr(feature = "cli", arg(long, value_enum, default_value_t = Format::Hunks))]
    pub format: Format,
    #[cfg_attr(feature = "cli", arg(long))]
    pub no_ignore: bool,
    #[cfg_attr(feature = "cli", arg(long))]
    pub hidden: bool,
    #[cfg_attr(
        feature = "cli",
        arg(long, value_enum, default_value_t = ColorChoice::Auto, help = "When to use syntax highlighting")
    )]
    pub color: ColorChoice,
    #[cfg_attr(feature = "cli", arg(long, value_enum, default_value_t = TimeFormat::Local))]
    pub time_format: TimeFormat,
    #[cfg_attr(feature = "cli", arg(long))]
    pub max_depth: Option<usize>,
    #[cfg_attr(feature = "cli", arg(long, conflicts_with = "fail_fast"))]
    pub skip_errors: bool,
    #[cfg_attr(feature = "cli", arg(long, conflicts_with = "skip_errors"))]
    pub fail_fast: bool,
    #[cfg_attr(feature = "cli", arg(long, value_name = "MILLIS"))]
    pub execution_budget_ms: Option<u64>,
    #[cfg_attr(feature = "cli", arg(long, value_name = "MILLIS"))]
    pub semantic_budget_ms: Option<u64>,
    #[cfg_attr(feature = "cli", arg(long, value_name = "COUNT"))]
    pub max_semantic_matches_per_file: Option<usize>,
    #[cfg_attr(feature = "cli", arg(long, value_name = "LANG"))]
    pub language_override: Option<String>,
    #[cfg_attr(
        feature = "cli",
        arg(long, value_enum, default_value_t = SemanticMatchModeFlag::Exact)
    )]
    pub semantic_match_mode: SemanticMatchModeFlag,
    #[cfg_attr(feature = "cli", arg(long))]
    pub semantic_strict: bool,
    #[cfg_attr(feature = "cli", arg(long))]
    pub strict_path_resolution: bool,
    #[cfg_attr(feature = "cli", arg(long = "no-snapshot-drift-detection"))]
    pub no_snapshot_drift_detection: bool,
    #[cfg_attr(feature = "cli", arg(long))]
    pub ignore_debug: bool,
    #[cfg_attr(feature = "cli", arg(long))]
    pub language_debug: bool,
    #[cfg_attr(feature = "cli", arg(long))]
    pub sql_trace: bool,
    #[cfg_attr(feature = "cli", arg(long, value_enum))]
    pub execution_profile: Option<ExecutionProfileFlag>,
    #[cfg_attr(feature = "cli", arg(long, default_value_t = true))]
    pub show_suppressed_placeholders: bool,
    #[cfg_attr(feature = "cli", arg(long, value_enum, default_value_t = PathDisplayModeFlag::Relative))]
    pub path_display: PathDisplayModeFlag,
    #[cfg_attr(feature = "cli", arg(long, value_enum, default_value_t = LineEndingModeFlag::Preserve))]
    pub line_endings: LineEndingModeFlag,
    #[cfg_attr(feature = "cli", arg(long))]
    pub no_match_text: bool,
    #[cfg_attr(
        feature = "cli",
        arg(
            long,
            short = 'C',
            value_name = "LINES",
            help = "Show LINES of context around matches for --format=hunks"
        )
    )]
    pub context: Option<usize>,

    /// List files with metadata instead of dumping content. Alias for --format=find
    #[cfg_attr(feature = "cli", arg(long))]
    pub find: bool,
}

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct LangArgs {
    #[cfg_attr(feature = "cli", command(subcommand))]
    pub action: Option<LangAction>,
}

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct QueryArgs {
    #[cfg_attr(feature = "cli", command(subcommand))]
    pub action: QueryAction,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(Subcommand))]
pub enum QueryAction {
    /// Explain preset expansion, lints, and planned stages for a query.
    Explain {
        #[cfg_attr(feature = "cli", arg(name = "QUERY"))]
        query: String,
        #[cfg_attr(feature = "cli", arg(long, short))]
        preset: Vec<String>,
        #[cfg_attr(feature = "cli", arg(long))]
        json: bool,
    },
    /// Print the effective query after preset expansion.
    Effective {
        #[cfg_attr(feature = "cli", arg(name = "QUERY"))]
        query: Option<String>,
        #[cfg_attr(feature = "cli", arg(long, short))]
        preset: Vec<String>,
    },
    /// Validate a query and print lints or validation errors.
    Validate {
        #[cfg_attr(feature = "cli", arg(name = "QUERY"))]
        query: String,
        #[cfg_attr(feature = "cli", arg(long, short))]
        preset: Vec<String>,
        #[cfg_attr(feature = "cli", arg(long))]
        json: bool,
    },
    /// Normalize a query from its AST back into a canonical string form.
    Normalize {
        #[cfg_attr(feature = "cli", arg(name = "QUERY"))]
        query: String,
    },
    /// Print the stable serialized AST for a query.
    Ast {
        #[cfg_attr(feature = "cli", arg(name = "QUERY"))]
        query: String,
    },
    /// Print predicate reference material generated from the live predicate registry.
    Reference {
        #[cfg_attr(feature = "cli", arg(long))]
        json: bool,
    },
    /// Explain why a query returned zero results under the current root and presets.
    WhyNoResults {
        #[cfg_attr(feature = "cli", arg(name = "QUERY"))]
        query: String,
        #[cfg_attr(feature = "cli", arg(long, short))]
        preset: Vec<String>,
        #[cfg_attr(feature = "cli", arg(long, default_value = "."))]
        root: PathBuf,
    },
    /// Explain how one file behaved across metadata and semantic evaluation stages.
    WhyFile {
        #[cfg_attr(feature = "cli", arg(name = "QUERY"))]
        query: String,
        #[cfg_attr(feature = "cli", arg(name = "PATH"))]
        path: PathBuf,
        #[cfg_attr(feature = "cli", arg(long, short))]
        preset: Vec<String>,
        #[cfg_attr(feature = "cli", arg(long, default_value = "."))]
        root: PathBuf,
    },
    /// Explain SQL dialect detection for a specific .sql file.
    Dialect {
        #[cfg_attr(feature = "cli", arg(name = "PATH"))]
        path: PathBuf,
    },
}

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct ConfigArgs {
    #[cfg_attr(feature = "cli", command(subcommand))]
    pub action: ConfigAction,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(Subcommand))]
pub enum ConfigAction {
    /// Print the preferred global config path.
    Path,
    /// Print the merged config visible from the current working directory.
    Show,
    /// Validate the merged config and preset graph before executing a search.
    Validate {
        #[cfg_attr(feature = "cli", arg(long))]
        json: bool,
    },
    /// Validate runtime assumptions, config visibility, and execution defaults.
    Doctor {
        #[cfg_attr(feature = "cli", arg(long))]
        json: bool,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(Subcommand))]
pub enum LangAction {
    /// List all supported languages.
    List,
    /// Describe the predicates available for a specific language.
    Describe { language: String },
    /// Inventory the current root by extension and semantic-capable language coverage.
    Inventory {
        #[cfg_attr(feature = "cli", arg(long, default_value = "."))]
        root: PathBuf,
        #[cfg_attr(feature = "cli", arg(long))]
        json: bool,
    },
    /// Print a generated language-by-predicate support matrix.
    Matrix {
        #[cfg_attr(feature = "cli", arg(long))]
        json: bool,
    },
}

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct PresetArgs {
    #[cfg_attr(feature = "cli", command(subcommand))]
    pub action: PresetAction,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(Subcommand))]
pub enum PresetAction {
    /// List all available presets.
    List,
    /// Add or update a preset in the global config file.
    Add {
        #[cfg_attr(feature = "cli", arg(required = true))]
        name: String,
        #[cfg_attr(feature = "cli", arg(required = true))]
        query: String,
    },
    /// Remove a preset from the global config file.
    Remove {
        #[cfg_attr(feature = "cli", arg(required = true))]
        name: String,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum Format {
    /// Show only the specific code blocks ("hunks") that match a semantic query
    #[default]
    Hunks,
    /// One line per file with match counts and content state
    Summary,
    /// One line per file with diagnostics and content policy details
    Diagnostics,
    /// One line per match with line/column locations
    Matches,
    /// Context snippets around each match
    Snippets,
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
#[cfg(feature = "cli")]
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search(args) => run_search(args),
        Commands::Query(args) => run_query(args.action),
        Commands::Config(args) => run_config(args.action),
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
    use crate::limits::MAX_FILE_SIZE;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    fn sample_result(path: &str, content: &str, matches: Vec<Match>) -> SearchResult {
        SearchResult {
            path: PathBuf::from(path),
            matches,
            content: content.to_string(),
            content_state: ContentState::Loaded,
            diagnostics: vec![],
            metadata: SearchResultMetadata::default(),
        }
    }

    fn empty_stats() -> SearchStats {
        SearchStats::default()
    }

    fn raw_item(path: PathBuf, ranges: Vec<TsRange>) -> RawSearchItem {
        RawSearchItem {
            backend: Arc::new(crate::backend::RealFsSearchBackend),
            display_path: path.clone(),
            resolved_path: path,
            root_relative_path: None,
            resolution: PathResolution::Canonical,
            ranges,
            diagnostics: vec![],
            semantic_skip_reasons: vec![],
            snapshot: None,
        }
    }

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
        let result = sample_result("test.rs", "fn main() {}", vec![]);
        assert!(result.is_whole_file_match());
    }

    #[test]
    fn test_is_whole_file_match_with_matches() {
        let result = sample_result(
            "test.rs",
            "fn main() {}",
            vec![Match {
                start_line: 1,
                end_line: 1,
                start_column: 0,
                end_column: 12,
                byte_range: 0..12,
                text: "fn main() {}".to_string(),
            }],
        );
        assert!(!result.is_whole_file_match());
    }

    #[test]
    fn test_matched_lines_single_match() {
        let result = sample_result(
            "test.rs",
            "...",
            vec![Match {
                start_line: 5,
                end_line: 7,
                start_column: 0,
                end_column: 1,
                byte_range: 0..100,
                text: "...".to_string(),
            }],
        );
        assert_eq!(result.matched_lines(), vec![5, 6, 7]);
    }

    #[test]
    fn test_matched_lines_overlapping() {
        let result = sample_result(
            "test.rs",
            "...",
            vec![
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
        );
        assert_eq!(result.matched_lines(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_matched_lines_empty_matches() {
        let result = sample_result("test.rs", "...", vec![]);
        assert_eq!(result.matched_lines(), Vec::<usize>::new());
    }

    #[test]
    fn test_match_count() {
        let result = sample_result(
            "test.rs",
            "...",
            vec![
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
        );
        assert_eq!(result.match_count(), 2);
    }

    #[test]
    fn test_total_lines_matched() {
        let result = sample_result(
            "test.rs",
            "...",
            vec![
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
        );
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
            raw_item(PathBuf::from("file1.txt"), vec![]),
            raw_item(PathBuf::from("file2.txt"), vec![]),
        ];
        let mut iter = SearchResultIterator::new(results, empty_stats(), vec![]);

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

        let results = vec![
            raw_item(file_ok.clone(), vec![]),
            raw_item(missing.clone(), vec![]),
        ];
        let mut iter = SearchResultIterator::new(results, empty_stats(), vec![]);

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
            .map(|i| raw_item(dir.path().join(format!("file{i}.txt")), vec![]))
            .collect();

        let taken: Vec<_> = SearchResultIterator::new(results, empty_stats(), vec![])
            .take(1)
            .collect();
        assert_eq!(taken.len(), 1);
        assert!(taken[0].is_ok());
    }

    #[test]
    fn test_iterator_whole_file_match_loads_content() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("file.txt");
        fs::write(&file, "hello world").unwrap();

        let mut iter =
            SearchResultIterator::new(vec![raw_item(file.clone(), vec![])], empty_stats(), vec![]);
        let result = iter.next().unwrap().unwrap();

        assert!(result.is_whole_file_match());
        assert!(result.matches.is_empty());
        assert_eq!(result.content, "hello world");
        assert_eq!(result.content_state, ContentState::Loaded);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_iterator_large_file_skips_content() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("large.txt");
        let bytes = vec![b'x'; (MAX_FILE_SIZE + 1) as usize];
        fs::write(&file, &bytes).unwrap();

        let mut iter =
            SearchResultIterator::new(vec![raw_item(file.clone(), vec![])], empty_stats(), vec![]);
        let result = iter.next().unwrap().unwrap();

        assert_eq!(
            result.content_state,
            ContentState::Skipped {
                reason: ContentSkipReason::TooLarge
            }
        );
        assert!(result.content.is_empty());
        assert_eq!(result.match_count(), 0);
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_iterator_missing_file_error() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("missing.txt");
        fs::write(&file, "content").unwrap();

        let mut iter =
            SearchResultIterator::new(vec![raw_item(file.clone(), vec![])], empty_stats(), vec![]);
        if let Some(raw) = iter
            .buffered_raw_items_mut()
            .and_then(|items| items.first_mut())
        {
            raw.snapshot = Some(FileSnapshot {
                len: 7,
                modified_unix_millis: Some(0),
                ..Default::default()
            });
        }
        fs::remove_file(&file).unwrap();

        let err = iter.next().unwrap().unwrap_err();
        let materialization = err
            .downcast_ref::<SearchMaterializationError>()
            .expect("iterator errors should carry typed materialization context");
        assert_eq!(
            materialization.kind,
            SearchMaterializationFailureKind::ContentReadFailed
        );
        assert!(materialization.snapshot_drift);
        assert!(materialization.message.contains("missing.txt"));
    }

    #[test]
    fn test_iterator_permission_error() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("restricted.txt");
        fs::write(&file, "content").unwrap();
        let permissions = fs::metadata(&file).unwrap().permissions();
        fs::set_permissions(&file, PermissionsExt::from_mode(0o000)).unwrap();

        let mut iter =
            SearchResultIterator::new(vec![raw_item(file.clone(), vec![])], empty_stats(), vec![]);
        let err = iter.next().unwrap().unwrap_err().to_string();
        assert!(err.contains("restricted.txt"));
        assert!(
            err.contains("Permission denied")
                || err.contains("Operation not permitted")
                || err.contains("Failed to read file")
        );

        fs::set_permissions(&file, permissions).unwrap();
    }

    #[test]
    fn test_file_snapshot_from_backend_metadata_preserves_backend_identity_fields() {
        let snapshot = FileSnapshot::from_backend_metadata(&crate::backend::BackendMetadata {
            size_bytes: 42,
            modified_unix_millis: Some(1234),
            readonly: true,
            permissions_display: "r--r--r--".to_string(),
            file_type: crate::backend::BackendFileType::File,
            stable_token: Some("token:abc".to_string()),
            device_id: Some(7),
            inode: Some(9),
        });

        assert_eq!(snapshot.len, 42);
        assert_eq!(snapshot.modified_unix_millis, Some(1234));
        assert!(snapshot.readonly);
        assert_eq!(snapshot.permissions_display, "r--r--r--");
        assert_eq!(snapshot.stable_token.as_deref(), Some("token:abc"));
        assert_eq!(snapshot.device_id, Some(7));
        assert_eq!(snapshot.inode, Some(9));
    }

    #[test]
    fn test_iterator_binary_file_skips_content() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("binary.bin");
        fs::write(&file, b"\x00\x01\x02binary").unwrap();

        let mut iter =
            SearchResultIterator::new(vec![raw_item(file.clone(), vec![])], empty_stats(), vec![]);
        let result = iter.next().unwrap().unwrap();

        assert_eq!(
            result.content_state,
            ContentState::Skipped {
                reason: ContentSkipReason::Binary
            }
        );
        assert!(result.content.is_empty());
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.message.to_lowercase().contains("binary")));
    }

    #[test]
    fn test_iterator_invalid_utf8_is_lossy() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("invalid.txt");
        fs::write(&file, vec![0xff, 0xfe]).unwrap();

        let mut iter =
            SearchResultIterator::new(vec![raw_item(file.clone(), vec![])], empty_stats(), vec![]);
        let result = iter.next().unwrap().unwrap();

        assert_eq!(result.content_state, ContentState::LoadedLossy);
        assert!(result.content.contains('\u{FFFD}'));
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.kind == crate::content::DiagnosticKind::ContentDecodedLossy));
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

        let mut iter =
            SearchResultIterator::new(vec![raw_item(file, ranges)], empty_stats(), vec![]);
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

        let mut iter = SearchResultIterator::new(
            vec![
                raw_item(missing.clone(), vec![]),
                raw_item(existing.clone(), vec![]),
            ],
            empty_stats(),
            vec![],
        );
        let first = iter.next().unwrap();
        let second = iter.next().unwrap();

        assert!(first.is_err());
        assert!(second.is_ok());
    }
}
