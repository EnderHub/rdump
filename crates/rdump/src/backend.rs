use anyhow::{anyhow, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use crate::engine;
use crate::limits::{safe_canonicalize, DEFAULT_MAX_DEPTH};
use crate::{
    FileIdentity, PathResolution, SearchCancellationToken, SearchDiagnostic, SearchOptions,
    SearchPathIterator, SearchReport, SearchResult, SearchResultIterator,
};

static DEFAULT_IGNORE_SET: Lazy<GlobSet> = Lazy::new(|| {
    let mut builder = GlobSetBuilder::new();
    for pattern in [
        "node_modules/**",
        "target/**",
        "dist/**",
        "build/**",
        ".git/**",
        ".svn/**",
        ".hg/**",
        "**/*.pyc",
        "**/__pycache__/**",
    ] {
        builder.add(Glob::new(pattern).expect("default ignore glob should compile"));
    }
    builder.build().expect("default ignore set should compile")
});

const MAX_EXCLUSION_DIAGNOSTICS: usize = 25;

/// Backend-neutral file kind used in search metadata and snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendFileType {
    File,
    Directory,
    Symlink,
    Other,
}

/// Backend-supplied metadata used by predicates, formatters, and drift checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendMetadata {
    /// File size in bytes.
    pub size_bytes: u64,
    /// Last-modified timestamp when the backend can provide one.
    pub modified_unix_millis: Option<i64>,
    pub readonly: bool,
    /// Human-readable permissions summary from the active backend.
    pub permissions_display: String,
    pub file_type: BackendFileType,
    /// Optional stronger identity token for fingerprints and drift detection.
    pub stable_token: Option<String>,
    pub device_id: Option<u64>,
    pub inode: Option<u64>,
}

impl BackendMetadata {
    pub fn to_path_metadata(&self) -> rdump_contracts::PathMetadata {
        rdump_contracts::PathMetadata {
            size_bytes: self.size_bytes,
            modified_unix_millis: self.modified_unix_millis,
            readonly: self.readonly,
            permissions_display: self.permissions_display.clone(),
        }
    }
}

/// Stable path identity returned by a search backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendPathIdentity {
    /// Caller-facing path projection.
    pub display_path: PathBuf,
    /// Backend-stable identity path.
    ///
    /// For the real filesystem this is usually canonical, but virtual backends
    /// may return a stable non-host path here.
    pub resolved_path: PathBuf,
    /// Path relative to the backend root when available.
    pub root_relative_path: Option<PathBuf>,
    pub resolution: PathResolution,
}

impl BackendPathIdentity {
    pub fn to_file_identity(&self) -> FileIdentity {
        FileIdentity {
            display_path: self.display_path.clone(),
            resolved_path: self.resolved_path.clone(),
            root_relative_path: self.root_relative_path.clone(),
            resolution: self.resolution,
        }
    }
}

/// Search-oriented discovery input passed to [`SearchBackend::discover`].
#[derive(Debug, Clone)]
pub struct DiscoveryRequest {
    pub root: PathBuf,
    pub display_root: PathBuf,
    pub no_ignore: bool,
    pub hidden: bool,
    pub max_depth: Option<usize>,
    pub ignore_debug: bool,
}

/// Candidate file identities and skip counters returned by backend discovery.
#[derive(Debug, Clone, Default)]
pub struct DiscoveryReport {
    pub candidates: Vec<BackendPathIdentity>,
    pub diagnostics: Vec<SearchDiagnostic>,
    pub hidden_skipped: usize,
    pub ignore_skipped: usize,
    pub max_depth_skipped: usize,
    pub unreadable_entries: usize,
    pub root_boundary_excluded: usize,
}

/// Search storage abstraction for real filesystems and virtual workspaces.
///
/// Implementors own discovery, path normalization, metadata reads, and byte
/// reads. The rest of the search engine stays in `rdump`.
pub trait SearchBackend: Send + Sync + fmt::Debug {
    /// Normalizes the caller-supplied root into a backend-stable root path.
    fn normalize_root(&self, root: &Path) -> Result<PathBuf>;
    /// Discovers candidate files plus skip diagnostics for one search session.
    fn discover(&self, request: &DiscoveryRequest) -> Result<DiscoveryReport>;
    /// Normalizes one path into backend identity projections.
    fn normalize_path(
        &self,
        root: &Path,
        display_root: &Path,
        path: &Path,
    ) -> Result<BackendPathIdentity>;
    /// Returns backend metadata for predicates, snapshots, and formatting.
    fn stat(&self, path: &Path) -> Result<BackendMetadata>;
    /// Returns raw file bytes for content loading and semantic evaluation.
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>>;
}

/// Built-in backend that searches the host filesystem.
#[derive(Debug, Default)]
pub struct RealFsSearchBackend;

/// Search session wrapper bound to one [`SearchBackend`] implementation.
#[derive(Debug, Clone)]
pub struct SearchRuntime {
    backend: Arc<dyn SearchBackend>,
}

impl SearchRuntime {
    /// Constructs a runtime backed by the default host filesystem behavior.
    pub fn real_fs() -> Self {
        Self::with_backend(Arc::new(RealFsSearchBackend))
    }

    /// Constructs a runtime from a caller-supplied backend implementation.
    pub fn with_backend(backend: Arc<dyn SearchBackend>) -> Self {
        Self { backend }
    }

    /// Returns the backend bound to this runtime.
    pub fn backend(&self) -> &Arc<dyn SearchBackend> {
        &self.backend
    }

    /// Runs a streaming search that materializes full [`SearchResult`] values.
    pub fn search_iter(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> Result<SearchResultIterator> {
        Ok(SearchResultIterator::from_raw_iter(
            self.search_raw_iter(query, options, None)?,
        ))
    }

    /// Runs a streaming search that emits only matching paths.
    pub fn search_path_iter(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> Result<SearchPathIterator> {
        Ok(SearchPathIterator::from_raw_iter(
            self.search_raw_iter(query, options, None)?,
        ))
    }

    pub(crate) fn search_raw_iter(
        &self,
        query: &str,
        options: &SearchOptions,
        cancellation: Option<SearchCancellationToken>,
    ) -> Result<engine::SearchRawIterator> {
        engine::SearchRawIterator::new(Arc::clone(&self.backend), query, options, cancellation)
    }

    /// Collects results plus engine statistics and diagnostics.
    pub fn search_with_stats(&self, query: &str, options: &SearchOptions) -> Result<SearchReport> {
        collect_search_report(self, query, options, options.error_mode, None)
    }

    /// Collects all results into memory.
    pub fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        Ok(self.search_with_stats(query, options)?.results)
    }

    pub(crate) fn collect_raw_search(
        &self,
        query: &str,
        options: &SearchOptions,
        cancellation: Option<SearchCancellationToken>,
    ) -> Result<engine::RawSearchReport> {
        let mut iter = self.search_raw_iter(query, options, cancellation)?;
        let mut results = Vec::new();
        while let Some(item) = iter.next() {
            results.push(item?);
        }

        Ok(engine::RawSearchReport {
            results,
            stats: iter.stats().clone(),
            diagnostics: iter.diagnostics().to_vec(),
        })
    }
}

impl SearchBackend for RealFsSearchBackend {
    fn normalize_root(&self, root: &Path) -> Result<PathBuf> {
        dunce::canonicalize(root).map_err(|_| {
            anyhow!(
                "root path '{}' does not exist or is not accessible.",
                root.display()
            )
        })
    }

    fn discover(&self, request: &DiscoveryRequest) -> Result<DiscoveryReport> {
        let mut files = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        let mut diagnostics = Vec::new();
        let mut walker_builder = WalkBuilder::new(&request.root);
        let root_unignores = if request.no_ignore {
            None
        } else {
            load_root_unignore_set(&request.root)
        };

        let effective_max_depth = request.max_depth.unwrap_or(DEFAULT_MAX_DEPTH);
        walker_builder
            .hidden(!request.hidden)
            .max_depth(Some(effective_max_depth))
            .follow_links(false);

        if request.no_ignore {
            walker_builder
                .ignore(false)
                .git_ignore(false)
                .git_global(false)
                .git_exclude(false);
        } else {
            if let Some(global_ignore_path) = dirs::config_dir().map(|p| p.join("rdump/ignore")) {
                if global_ignore_path.exists() {
                    if let Some(err) = walker_builder.add_ignore(global_ignore_path) {
                        diagnostics.push(SearchDiagnostic::walk_warning(
                            None,
                            format!("Could not add global ignore file: {err}"),
                        ));
                    }
                }
            }
            walker_builder.add_custom_ignore_filename(".rdumpignore");
        }

        for result in walker_builder.build() {
            match result {
                Ok(entry) => {
                    if entry.file_type().is_some_and(|ft| ft.is_file()) {
                        let original_path = entry.into_path();
                        if !request.no_ignore {
                            let relative = original_path
                                .strip_prefix(&request.root)
                                .unwrap_or(original_path.as_path());
                            let explicitly_unignored = root_unignores
                                .as_ref()
                                .is_some_and(|set| set.is_match(relative));
                            if DEFAULT_IGNORE_SET.is_match(relative) && !explicitly_unignored {
                                continue;
                            }
                        }
                        match safe_canonicalize(&original_path, &request.root) {
                            Ok(canonical_path) => {
                                if seen.insert(canonical_path.clone()) {
                                    files.push(self.normalize_path(
                                        &request.root,
                                        &request.display_root,
                                        &canonical_path,
                                    )?);
                                }
                            }
                            Err(err) => {
                                diagnostics.push(SearchDiagnostic::root_boundary(
                                    original_path,
                                    format!("Skipping path outside root ({err})"),
                                ));
                            }
                        }
                    }
                }
                Err(err) => {
                    diagnostics.push(SearchDiagnostic::walk_warning(
                        None,
                        format!("Could not access entry: {err}"),
                    ));
                }
            }
        }

        let mut report = analyze_discovery_impl(
            &request.root,
            request.no_ignore,
            request.hidden,
            request.max_depth,
            request.ignore_debug,
        );
        report.diagnostics.extend(diagnostics);
        report.candidates = files;
        report.candidates.sort_by(|left, right| {
            left.display_path
                .cmp(&right.display_path)
                .then(left.resolved_path.cmp(&right.resolved_path))
        });
        Ok(report)
    }

    fn normalize_path(
        &self,
        root: &Path,
        display_root: &Path,
        path: &Path,
    ) -> Result<BackendPathIdentity> {
        let resolved_path = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if let Ok(relative) = resolved_path.strip_prefix(root) {
            let relative = relative.to_path_buf();
            Ok(BackendPathIdentity {
                display_path: display_root.join(&relative),
                resolved_path,
                root_relative_path: Some(relative),
                resolution: PathResolution::Canonical,
            })
        } else {
            Ok(BackendPathIdentity {
                display_path: resolved_path.clone(),
                resolved_path,
                root_relative_path: None,
                resolution: PathResolution::Fallback,
            })
        }
    }

    fn stat(&self, path: &Path) -> Result<BackendMetadata> {
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
        Ok(backend_metadata_from_std(&metadata))
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        std::fs::read(path).with_context(|| format!("Failed to read file {}", path.display()))
    }
}

pub fn backend_metadata_from_std(metadata: &std::fs::Metadata) -> BackendMetadata {
    let readonly = metadata.permissions().readonly();
    #[cfg(unix)]
    let permissions_display = crate::formatter::format_mode(metadata.permissions().mode());
    #[cfg(not(unix))]
    let permissions_display = if readonly {
        "readonly".to_string()
    } else {
        "readwrite".to_string()
    };

    BackendMetadata {
        size_bytes: metadata.len(),
        modified_unix_millis: metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis() as i64),
        readonly,
        permissions_display,
        file_type: if metadata.is_file() {
            BackendFileType::File
        } else if metadata.is_dir() {
            BackendFileType::Directory
        } else if metadata.file_type().is_symlink() {
            BackendFileType::Symlink
        } else {
            BackendFileType::Other
        },
        stable_token: None,
        #[cfg(unix)]
        device_id: Some(metadata.dev()),
        #[cfg(not(unix))]
        device_id: None,
        #[cfg(unix)]
        inode: Some(metadata.ino()),
        #[cfg(not(unix))]
        inode: None,
    }
}

pub(crate) fn build_directory_hotspots(
    candidates: &[BackendPathIdentity],
) -> Vec<rdump_contracts::DirectoryHotspot> {
    let mut counts = BTreeMap::<String, usize>::new();
    for candidate in candidates {
        let bucket = candidate
            .root_relative_path
            .as_ref()
            .and_then(|path| path.components().next())
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        *counts.entry(bucket).or_default() += 1;
    }

    let mut hotspots: Vec<_> = counts
        .into_iter()
        .map(
            |(path, candidate_files)| rdump_contracts::DirectoryHotspot {
                path,
                candidate_files,
            },
        )
        .collect();
    hotspots.sort_by(|left, right| {
        right
            .candidate_files
            .cmp(&left.candidate_files)
            .then(left.path.cmp(&right.path))
    });
    hotspots.truncate(10);
    hotspots
}

pub(crate) fn collect_search_report(
    runtime: &SearchRuntime,
    query: &str,
    options: &SearchOptions,
    error_mode: rdump_contracts::ErrorMode,
    cancellation: Option<SearchCancellationToken>,
) -> Result<SearchReport> {
    let force_fail_fast = options.sql_strict || options.semantic_strict;
    let mut iter = runtime.search_raw_iter(query, options, cancellation)?;
    let mut result_diagnostics = Vec::new();
    let mut results = Vec::with_capacity(iter.remaining_hint());
    let materialize_started = std::time::Instant::now();

    while let Some(item) = iter.next() {
        match crate::materialize_raw_search_item(item) {
            Ok(result) => {
                result_diagnostics.extend(result.diagnostics.iter().cloned());
                results.push(result);
            }
            Err(err) => match error_mode {
                rdump_contracts::ErrorMode::SkipErrors => {
                    if force_fail_fast {
                        return Err(err);
                    }
                    result_diagnostics.push(SearchDiagnostic::walk_warning(
                        None,
                        format!("Skipping result after per-file error: {err}"),
                    ));
                }
                rdump_contracts::ErrorMode::FailFast => return Err(err),
            },
        }
    }

    let mut diagnostics = iter.diagnostics().to_vec();
    diagnostics.extend(result_diagnostics);
    let mut stats = iter.stats().clone();
    stats.whole_file_results = results
        .iter()
        .filter(|result| result.is_whole_file_match())
        .count();
    stats.ranged_results = results.len().saturating_sub(stats.whole_file_results);
    stats.suppressed_too_large = results
        .iter()
        .filter(|result| {
            matches!(
                result.content_state,
                crate::ContentState::Skipped {
                    reason: crate::ContentSkipReason::TooLarge
                }
            )
        })
        .count();
    stats.suppressed_binary = results
        .iter()
        .filter(|result| {
            matches!(
                result.content_state,
                crate::ContentState::Skipped {
                    reason: crate::ContentSkipReason::Binary
                }
            )
        })
        .count();
    stats.suppressed_secret_like = results
        .iter()
        .filter(|result| {
            matches!(
                result.content_state,
                crate::ContentState::Skipped {
                    reason: crate::ContentSkipReason::SecretLike
                }
            )
        })
        .count();
    stats.diagnostics = diagnostics.len();
    stats.materialize_millis = materialize_started.elapsed().as_millis() as u64;

    Ok(SearchReport {
        results,
        stats,
        diagnostics,
    })
}

#[derive(Debug, Default)]
struct RootIgnorePatterns {
    source: &'static str,
    patterns: Vec<String>,
    globset: Option<GlobSet>,
    unignore_globset: Option<GlobSet>,
}

fn analyze_discovery_impl(
    root: &Path,
    no_ignore: bool,
    hidden: bool,
    max_depth: Option<usize>,
    ignore_debug: bool,
) -> DiscoveryReport {
    let mut report = DiscoveryReport::default();
    let effective_max_depth = max_depth.unwrap_or(DEFAULT_MAX_DEPTH);
    let root_unignores = if no_ignore {
        None
    } else {
        load_root_unignore_set(root)
    };
    let gitignore = if no_ignore {
        RootIgnorePatterns::default()
    } else {
        load_root_ignore_patterns(root, ".gitignore")
    };
    let rdumpignore = if no_ignore {
        RootIgnorePatterns::default()
    } else {
        load_root_ignore_patterns(root, ".rdumpignore")
    };

    let mut walker_builder = WalkBuilder::new(root);
    walker_builder
        .hidden(false)
        .follow_links(false)
        .max_depth(None)
        .ignore(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false);

    for result in walker_builder.build() {
        match result {
            Ok(entry) => {
                if !entry
                    .file_type()
                    .is_some_and(|file_type| file_type.is_file())
                {
                    continue;
                }
                let path = entry.into_path();
                let relative = path.strip_prefix(root).unwrap_or(path.as_path());

                if relative.components().count() > effective_max_depth {
                    report.max_depth_skipped += 1;
                    continue;
                }

                if !hidden && path_has_hidden_component(relative) {
                    report.hidden_skipped += 1;
                    maybe_record_ignore_debug(
                        &mut report,
                        ignore_debug,
                        &path,
                        "hidden",
                        "path contains hidden component",
                    );
                    continue;
                }

                if !no_ignore {
                    let explicitly_unignored = root_unignores
                        .as_ref()
                        .is_some_and(|set| set.is_match(relative));
                    if DEFAULT_IGNORE_SET.is_match(relative) && !explicitly_unignored {
                        report.ignore_skipped += 1;
                        maybe_record_ignore_debug(
                            &mut report,
                            ignore_debug,
                            &path,
                            "default_ignore",
                            relative.to_string_lossy().as_ref(),
                        );
                        continue;
                    }
                    if let Some(pattern) = gitignore.matching_pattern(relative) {
                        report.ignore_skipped += 1;
                        maybe_record_ignore_debug(
                            &mut report,
                            ignore_debug,
                            &path,
                            gitignore.source,
                            &pattern,
                        );
                        continue;
                    }
                    if let Some(pattern) = rdumpignore.matching_pattern(relative) {
                        report.ignore_skipped += 1;
                        maybe_record_ignore_debug(
                            &mut report,
                            ignore_debug,
                            &path,
                            rdumpignore.source,
                            &pattern,
                        );
                        continue;
                    }
                }

                if safe_canonicalize(&path.to_path_buf(), &root.to_path_buf()).is_err() {
                    report.root_boundary_excluded += 1;
                }
            }
            Err(err) => {
                report.unreadable_entries += 1;
                report.diagnostics.push(SearchDiagnostic::walk_warning(
                    None,
                    format!("Could not access entry: {err}"),
                ));
            }
        }
    }

    report
}

fn maybe_record_ignore_debug(
    report: &mut DiscoveryReport,
    ignore_debug: bool,
    path: &Path,
    source: &str,
    pattern: &str,
) {
    if !ignore_debug || report.diagnostics.len() >= MAX_EXCLUSION_DIAGNOSTICS {
        return;
    }
    report.diagnostics.push(SearchDiagnostic::ignore_excluded(
        path.to_path_buf(),
        source,
        pattern,
    ));
}

fn path_has_hidden_component(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|segment| segment.starts_with('.') && segment != "." && segment != "..")
    })
}

fn load_root_unignore_set(root: &Path) -> Option<GlobSet> {
    let ignore_path = root.join(".rdumpignore");
    let contents = std::fs::read_to_string(ignore_path).ok()?;

    let mut builder = GlobSetBuilder::new();
    let mut added_any = false;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if !line.starts_with('!') || line.len() <= 1 {
            continue;
        }

        let pattern = line[1..].trim();
        if pattern.is_empty() {
            continue;
        }

        let glob_pattern = if pattern.ends_with('/') {
            format!("{pattern}**")
        } else {
            pattern.to_string()
        };

        if let Ok(glob) = Glob::new(&glob_pattern) {
            builder.add(glob);
            added_any = true;
        }
    }

    added_any.then(|| builder.build().ok()).flatten()
}

fn load_root_ignore_patterns(root: &Path, filename: &'static str) -> RootIgnorePatterns {
    let path = root.join(filename);
    let Ok(contents) = std::fs::read_to_string(path) else {
        return RootIgnorePatterns {
            source: filename,
            ..RootIgnorePatterns::default()
        };
    };

    let mut include_builder = GlobSetBuilder::new();
    let mut unignore_builder = GlobSetBuilder::new();
    let mut patterns = Vec::new();
    let mut added_include = false;
    let mut added_unignore = false;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (target, builder, added_any) = if let Some(pattern) = line.strip_prefix('!') {
            (pattern.trim(), &mut unignore_builder, &mut added_unignore)
        } else {
            (line, &mut include_builder, &mut added_include)
        };

        if target.is_empty() {
            continue;
        }

        let glob_pattern = normalize_ignore_pattern(target);
        if let Ok(glob) = Glob::new(&glob_pattern) {
            builder.add(glob);
            *added_any = true;
            if !line.starts_with('!') {
                patterns.push(glob_pattern);
            }
        }
    }

    RootIgnorePatterns {
        source: filename,
        patterns,
        globset: added_include
            .then(|| include_builder.build().ok())
            .flatten(),
        unignore_globset: added_unignore
            .then(|| unignore_builder.build().ok())
            .flatten(),
    }
}

fn normalize_ignore_pattern(pattern: &str) -> String {
    let trimmed = pattern.trim_start_matches("./");
    if trimmed.ends_with('/') {
        return format!("{trimmed}**");
    }
    if trimmed.contains('/') || trimmed.contains('*') || trimmed.starts_with('.') {
        return trimmed.to_string();
    }
    format!("**/{trimmed}")
}

impl RootIgnorePatterns {
    fn matching_pattern(&self, relative: &Path) -> Option<String> {
        if self
            .unignore_globset
            .as_ref()
            .is_some_and(|globset| globset.is_match(relative))
        {
            return None;
        }
        if !self
            .globset
            .as_ref()
            .is_some_and(|globset| globset.is_match(relative))
        {
            return None;
        }

        let relative = relative.to_string_lossy();
        self.patterns.iter().find_map(|pattern| {
            Glob::new(pattern).ok().and_then(|glob| {
                glob.compile_matcher()
                    .is_match(relative.as_ref())
                    .then(|| pattern.clone())
            })
        })
    }
}
