use crate::evaluator::{Evaluator, FileContext, MatchResult};
use crate::limits::{safe_canonicalize, DEFAULT_MAX_DEPTH};
use crate::parser::{self, AstNode, PredicateKey};
use crate::planner::resolve_effective_query;
use crate::predicates::code_aware::CodeAwareSettings;
use crate::predicates::{self, PredicateEvaluator};
use crate::{
    FileSnapshot, PathResolution, RawSearchItem, SearchCancellationToken, SearchDiagnostic,
    SearchOptions, SearchStats,
};
use anyhow::{anyhow, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use once_cell::sync::Lazy;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[allow(dead_code)]
pub(crate) struct RawSearchReport {
    pub(crate) results: Vec<RawSearchItem>,
    pub(crate) stats: SearchStats,
    pub(crate) diagnostics: Vec<SearchDiagnostic>,
}

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

#[derive(Debug, Default)]
pub(crate) struct DiscoveryAnalysis {
    diagnostics: Vec<SearchDiagnostic>,
    hidden_skipped: usize,
    ignore_skipped: usize,
    max_depth_skipped: usize,
    unreadable_entries: usize,
    root_boundary_excluded: usize,
}

#[derive(Debug, Default)]
struct RootIgnorePatterns {
    source: &'static str,
    patterns: Vec<String>,
    globset: Option<GlobSet>,
    unignore_globset: Option<GlobSet>,
}

#[derive(Clone)]
struct CandidateEntry {
    resolved_path: PathBuf,
    display_path: PathBuf,
    root_relative_path: Option<PathBuf>,
    resolution: PathResolution,
    estimated_bytes: usize,
}

pub(crate) struct SearchRawIterator {
    candidates: Vec<CandidateEntry>,
    next_candidate: usize,
    canonical_root: PathBuf,
    options: SearchOptions,
    stats: SearchStats,
    diagnostics: Vec<SearchDiagnostic>,
    started: Instant,
    time_budget: Option<Duration>,
    metadata_evaluator: Evaluator,
    full_evaluator: Evaluator,
    semantic_telemetry: std::sync::Arc<crate::predicates::code_aware::SemanticTelemetry>,
    query_cache_hits_before: usize,
    query_cache_misses_before: usize,
    cancellation: Option<SearchCancellationToken>,
    cancelled: bool,
    remaining_candidate_bytes: usize,
}

impl fmt::Debug for SearchRawIterator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SearchRawIterator")
            .field("next_candidate", &self.next_candidate)
            .field("candidate_files", &self.stats.candidate_files)
            .field("matched_files", &self.stats.matched_files)
            .field("cancelled", &self.cancelled)
            .finish()
    }
}

impl SearchRawIterator {
    pub(crate) fn new(
        query: &str,
        options: &SearchOptions,
        cancellation: Option<SearchCancellationToken>,
    ) -> Result<Self> {
        let display_root = options.root.clone();
        let canonical_root = dunce::canonicalize(&options.root).map_err(|_| {
            anyhow!(
                "root path '{}' does not exist or is not accessible.",
                options.root.display()
            )
        })?;
        let query_to_parse = resolve_effective_query(query, options)?;
        let ast = crate::planner::optimize_ast(parser::parse_query(&query_to_parse)?);
        validate_ast_predicates(&ast, &predicates::create_predicate_registry())?;

        let walk_started = Instant::now();
        let (candidate_files, mut diagnostics) = get_candidate_files(
            &canonical_root,
            options.no_ignore,
            options.hidden,
            options.max_depth,
        )?;
        let discovery = analyze_discovery(
            &canonical_root,
            options.no_ignore,
            options.hidden,
            options.max_depth,
            options.ignore_debug,
        );
        diagnostics.extend(discovery.diagnostics);

        let mut candidates: Vec<CandidateEntry> = candidate_files
            .into_iter()
            .map(|resolved_path| {
                let (display_path, root_relative_path, resolution) =
                    build_file_identity(&resolved_path, &canonical_root, &display_root);
                let estimated_bytes = resolved_path.as_os_str().len()
                    + display_path.as_os_str().len()
                    + root_relative_path
                        .as_ref()
                        .map(|path| path.as_os_str().len())
                        .unwrap_or_default()
                    + 64;
                CandidateEntry {
                    resolved_path,
                    display_path,
                    root_relative_path,
                    resolution,
                    estimated_bytes,
                }
            })
            .collect();
        candidates.sort_by(|left, right| {
            left.display_path
                .cmp(&right.display_path)
                .then(left.resolved_path.cmp(&right.resolved_path))
        });

        let mut stats = SearchStats {
            candidate_files: candidates.len(),
            hidden_skipped: discovery.hidden_skipped,
            ignore_skipped: discovery.ignore_skipped,
            max_depth_skipped: discovery.max_depth_skipped,
            unreadable_entries: discovery.unreadable_entries,
            root_boundary_excluded: discovery.root_boundary_excluded,
            directory_hotspots: build_directory_hotspots(
                &candidates
                    .iter()
                    .map(|candidate| candidate.resolved_path.clone())
                    .collect::<Vec<_>>(),
                &canonical_root,
            ),
            walk_millis: walk_started.elapsed().as_millis() as u64,
            ..Default::default()
        };

        let metadata_evaluator =
            Evaluator::new(ast.clone(), predicates::create_metadata_predicate_registry());
        let (query_cache_hits_before, query_cache_misses_before) =
            crate::predicates::code_aware::query_cache_metrics_snapshot();
        let semantic_telemetry =
            std::sync::Arc::new(crate::predicates::code_aware::SemanticTelemetry::default());
        let mut code_settings = CodeAwareSettings::default();
        if let Some(dialect) = options.sql_dialect {
            code_settings.sql_dialect = Some(dialect);
        }
        code_settings.sql_strict = options.sql_strict;
        code_settings.semantic_budget_ms = options.semantic_budget_ms;
        code_settings.max_semantic_matches_per_file = options.max_semantic_matches_per_file;
        code_settings.language_override = options.language_override.clone();
        code_settings.semantic_match_mode = options.semantic_match_mode;
        code_settings.semantic_strict = options.semantic_strict;
        code_settings.language_debug = options.language_debug;
        code_settings.sql_trace = options.sql_trace;
        code_settings.telemetry = Some(semantic_telemetry.clone());
        let full_evaluator = Evaluator::new(ast, predicates::create_predicate_registry_with_settings(code_settings));

        let remaining_candidate_bytes = candidates
            .iter()
            .map(|candidate| candidate.estimated_bytes)
            .sum();
        stats.query_cache_hits = 0;
        stats.query_cache_misses = 0;

        Ok(Self {
            candidates,
            next_candidate: 0,
            canonical_root,
            options: options.clone(),
            stats,
            diagnostics,
            started: Instant::now(),
            time_budget: search_time_budget(options),
            metadata_evaluator,
            full_evaluator,
            semantic_telemetry,
            query_cache_hits_before,
            query_cache_misses_before,
            cancellation,
            cancelled: false,
            remaining_candidate_bytes,
        })
    }

    pub(crate) fn stats(&self) -> &SearchStats {
        &self.stats
    }

    pub(crate) fn diagnostics(&self) -> &[SearchDiagnostic] {
        &self.diagnostics
    }

    pub(crate) fn remaining_hint(&self) -> usize {
        self.candidates.len().saturating_sub(self.next_candidate)
    }

    pub(crate) fn was_cancelled(&self) -> bool {
        self.cancelled
    }

    pub(crate) fn estimated_state_bytes(&self) -> usize {
        self.remaining_candidate_bytes
            + self
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    diagnostic.message.len()
                        + diagnostic
                            .path
                            .as_ref()
                            .map(|path| path.as_os_str().len())
                            .unwrap_or_default()
                        + 32
                })
                .sum::<usize>()
    }

    fn refresh_runtime_stats(&mut self) {
        self.stats.semantic_parse_failures = self.semantic_telemetry.total_parse_failures();
        self.stats.semantic_budget_exhaustions = self.semantic_telemetry.budget_exhaustions();
        self.stats.semantic_parse_failures_by_language =
            self.semantic_telemetry.parse_failures_by_language();
        self.stats.tree_cache_hits = self.semantic_telemetry.tree_cache_hits();
        self.stats.tree_cache_misses = self.semantic_telemetry.tree_cache_misses();
        let (cache_hits_after, cache_misses_after) =
            crate::predicates::code_aware::query_cache_metrics_snapshot();
        self.stats.query_cache_hits =
            cache_hits_after.saturating_sub(self.query_cache_hits_before);
        self.stats.query_cache_misses =
            cache_misses_after.saturating_sub(self.query_cache_misses_before);
        self.stats.diagnostics = self.diagnostics.len();
    }

    fn next_candidate(&mut self) -> Option<CandidateEntry> {
        let candidate = self.candidates.get(self.next_candidate)?.clone();
        self.next_candidate += 1;
        self.remaining_candidate_bytes = self
            .remaining_candidate_bytes
            .saturating_sub(candidate.estimated_bytes);
        Some(candidate)
    }

    fn should_stop(&self) -> bool {
        self.cancellation
            .as_ref()
            .is_some_and(SearchCancellationToken::is_cancelled)
    }
}

impl Iterator for SearchRawIterator {
    type Item = Result<RawSearchItem>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.should_stop() {
                self.cancelled = true;
                self.refresh_runtime_stats();
                return None;
            }

            let candidate = match self.next_candidate() {
                Some(candidate) => candidate,
                None => {
                    self.refresh_runtime_stats();
                    return None;
                }
            };

            if let Some(err) = budget_error(self.started, self.time_budget) {
                self.refresh_runtime_stats();
                return Some(Err(err));
            }

            let mut context = FileContext::new(
                candidate.resolved_path.clone(),
                self.canonical_root.clone(),
            );

            let prefilter_started = Instant::now();
            let prefilter = self.metadata_evaluator.evaluate(&mut context);
            self.stats.prefilter_millis += prefilter_started.elapsed().as_millis() as u64;
            let prefilter_result = match prefilter {
                Ok(result) => result,
                Err(err) => {
                    self.refresh_runtime_stats();
                    return Some(Err(anyhow!(
                        "Error during pre-filter on {}: {}",
                        candidate.resolved_path.display(),
                        err
                    )));
                }
            };
            if !prefilter_result.is_match() {
                let diagnostics = context.take_diagnostics();
                if !diagnostics.is_empty() {
                    self.diagnostics.extend(diagnostics);
                    self.refresh_runtime_stats();
                }
                continue;
            }

            self.stats.prefiltered_files += 1;

            let evaluate_started = Instant::now();
            let evaluation = self.full_evaluator.evaluate(&mut context);
            self.stats.evaluate_millis += evaluate_started.elapsed().as_millis() as u64;
            self.stats.evaluated_files += 1;

            let path_diagnostics = context.take_diagnostics();
            let semantic_skip_reasons = context.take_semantic_skip_reasons();
            let snapshot = self
                .options
                .snapshot_drift_detection
                .then(|| {
                    std::fs::metadata(&candidate.resolved_path)
                        .ok()
                        .map(|metadata| FileSnapshot::from_metadata(&metadata))
                })
                .flatten();

            match evaluation {
                Ok(MatchResult::Boolean(false)) => {
                    if !path_diagnostics.is_empty() {
                        self.diagnostics.extend(path_diagnostics);
                    }
                    self.refresh_runtime_stats();
                }
                Ok(MatchResult::Boolean(true)) => {
                    if self.options.strict_path_resolution
                        && candidate.resolution == PathResolution::Fallback
                    {
                        self.refresh_runtime_stats();
                        return Some(Err(anyhow!(
                            "Strict path resolution failed for {}",
                            candidate.resolved_path.display()
                        )));
                    }
                    self.stats.matched_files += 1;
                    self.stats.whole_file_results += 1;
                    let diagnostics = attach_resolution_diagnostics(
                        candidate.display_path.clone(),
                        candidate.resolution,
                        path_diagnostics,
                    );
                    self.refresh_runtime_stats();
                    return Some(Ok(RawSearchItem {
                        display_path: candidate.display_path,
                        resolved_path: candidate.resolved_path,
                        root_relative_path: candidate.root_relative_path,
                        resolution: candidate.resolution,
                        ranges: Vec::new(),
                        diagnostics,
                        semantic_skip_reasons,
                        snapshot,
                    }));
                }
                Ok(MatchResult::Hunks(hunks)) => {
                    if hunks.is_empty() {
                        if !path_diagnostics.is_empty() {
                            self.diagnostics.extend(path_diagnostics);
                        }
                        self.refresh_runtime_stats();
                        continue;
                    }
                    if self.options.strict_path_resolution
                        && candidate.resolution == PathResolution::Fallback
                    {
                        self.refresh_runtime_stats();
                        return Some(Err(anyhow!(
                            "Strict path resolution failed for {}",
                            candidate.resolved_path.display()
                        )));
                    }
                    self.stats.matched_files += 1;
                    self.stats.ranged_results += 1;
                    self.stats.matched_ranges += hunks.len();
                    let diagnostics = attach_resolution_diagnostics(
                        candidate.display_path.clone(),
                        candidate.resolution,
                        path_diagnostics,
                    );
                    self.refresh_runtime_stats();
                    return Some(Ok(RawSearchItem {
                        display_path: candidate.display_path,
                        resolved_path: candidate.resolved_path,
                        root_relative_path: candidate.root_relative_path,
                        resolution: candidate.resolution,
                        ranges: hunks,
                        diagnostics,
                        semantic_skip_reasons,
                        snapshot,
                    }));
                }
                Err(err) => {
                    self.refresh_runtime_stats();
                    return Some(Err(anyhow!(
                        "Error evaluating file {}: {}",
                        candidate.resolved_path.display(),
                        err
                    )));
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.remaining_hint()))
    }
}

impl std::iter::FusedIterator for SearchRawIterator {}

pub(crate) fn search_raw_iter(
    query: &str,
    options: &SearchOptions,
    cancellation: Option<SearchCancellationToken>,
) -> Result<SearchRawIterator> {
    SearchRawIterator::new(query, options, cancellation)
}

#[allow(dead_code)]
pub(crate) fn collect_raw_search(
    query: &str,
    options: &SearchOptions,
    cancellation: Option<SearchCancellationToken>,
) -> Result<RawSearchReport> {
    let mut iter = search_raw_iter(query, options, cancellation)?;
    let mut results = Vec::new();
    while let Some(item) = iter.next() {
        results.push(item?);
    }

    Ok(RawSearchReport {
        results,
        stats: iter.stats().clone(),
        diagnostics: iter.diagnostics().to_vec(),
    })
}

pub(crate) fn get_candidate_files(
    root: &PathBuf,
    no_ignore: bool,
    hidden: bool,
    max_depth: Option<usize>,
) -> Result<(Vec<PathBuf>, Vec<SearchDiagnostic>)> {
    let mut files = Vec::new();
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    let mut walker_builder = WalkBuilder::new(root);
    let root_unignores = if no_ignore {
        None
    } else {
        load_root_unignore_set(root)
    };

    let effective_max_depth = max_depth.unwrap_or(DEFAULT_MAX_DEPTH);
    walker_builder
        .hidden(!hidden)
        .max_depth(Some(effective_max_depth))
        .follow_links(false);

    if no_ignore {
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
                    if !no_ignore {
                        let relative = original_path
                            .strip_prefix(root)
                            .unwrap_or(original_path.as_path());
                        let explicitly_unignored = root_unignores
                            .as_ref()
                            .is_some_and(|set| set.is_match(relative));
                        if DEFAULT_IGNORE_SET.is_match(relative) && !explicitly_unignored {
                            continue;
                        }
                    }
                    match safe_canonicalize(&original_path, root) {
                        Ok(canonical_path) => {
                            if seen.insert(canonical_path.clone()) {
                                files.push(canonical_path);
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
    Ok((files, diagnostics))
}

fn analyze_discovery(
    root: &PathBuf,
    no_ignore: bool,
    hidden: bool,
    max_depth: Option<usize>,
    ignore_debug: bool,
) -> DiscoveryAnalysis {
    analyze_discovery_impl(root, no_ignore, hidden, max_depth, ignore_debug)
}

fn analyze_discovery_impl(
    root: &PathBuf,
    no_ignore: bool,
    hidden: bool,
    max_depth: Option<usize>,
    ignore_debug: bool,
) -> DiscoveryAnalysis {
    let mut analysis = DiscoveryAnalysis::default();
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
                if !entry.file_type().is_some_and(|file_type| file_type.is_file()) {
                    continue;
                }
                let path = entry.into_path();
                let relative = path.strip_prefix(root).unwrap_or(path.as_path());

                if relative.components().count() > effective_max_depth {
                    analysis.max_depth_skipped += 1;
                    continue;
                }

                if !hidden && path_has_hidden_component(relative) {
                    analysis.hidden_skipped += 1;
                    maybe_record_ignore_debug(
                        &mut analysis,
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
                        analysis.ignore_skipped += 1;
                        maybe_record_ignore_debug(
                            &mut analysis,
                            ignore_debug,
                            &path,
                            "default_ignore",
                            relative.to_string_lossy().as_ref(),
                        );
                        continue;
                    }
                    if let Some(pattern) = gitignore.matching_pattern(relative) {
                        analysis.ignore_skipped += 1;
                        maybe_record_ignore_debug(
                            &mut analysis,
                            ignore_debug,
                            &path,
                            gitignore.source,
                            &pattern,
                        );
                        continue;
                    }
                    if let Some(pattern) = rdumpignore.matching_pattern(relative) {
                        analysis.ignore_skipped += 1;
                        maybe_record_ignore_debug(
                            &mut analysis,
                            ignore_debug,
                            &path,
                            rdumpignore.source,
                            &pattern,
                        );
                        continue;
                    }
                }

                if safe_canonicalize(&path, root).is_err() {
                    analysis.root_boundary_excluded += 1;
                }
            }
            Err(err) => {
                analysis.unreadable_entries += 1;
                analysis.diagnostics.push(SearchDiagnostic::walk_warning(
                    None,
                    format!("Could not access entry: {err}"),
                ));
            }
        }
    }

    analysis
}

fn maybe_record_ignore_debug(
    analysis: &mut DiscoveryAnalysis,
    ignore_debug: bool,
    path: &Path,
    source: &str,
    pattern: &str,
) {
    if !ignore_debug || analysis.diagnostics.len() >= MAX_EXCLUSION_DIAGNOSTICS {
        return;
    }
    analysis.diagnostics.push(SearchDiagnostic::ignore_excluded(
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

pub(crate) fn build_directory_hotspots(
    candidate_files: &[PathBuf],
    root: &PathBuf,
) -> Vec<rdump_contracts::DirectoryHotspot> {
    let mut counts = BTreeMap::<String, usize>::new();
    for path in candidate_files {
        let relative = path.strip_prefix(root).unwrap_or(path);
        let bucket = relative
            .components()
            .next()
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

fn load_root_unignore_set(root: &PathBuf) -> Option<GlobSet> {
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

fn load_root_ignore_patterns(root: &PathBuf, filename: &'static str) -> RootIgnorePatterns {
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
        globset: added_include.then(|| include_builder.build().ok()).flatten(),
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

fn search_time_budget(options: &SearchOptions) -> Option<Duration> {
    if let Some(value) = options.execution_budget_ms.filter(|value| *value > 0) {
        return Some(Duration::from_millis(value));
    }

    std::env::var("RDUMP_MAX_SEARCH_MILLIS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .map(Duration::from_millis)
}

fn budget_error(started: Instant, budget: Option<Duration>) -> Option<anyhow::Error> {
    let budget = budget?;
    if started.elapsed() <= budget {
        return None;
    }

    Some(anyhow!(
        "Search exceeded configured time budget of {}ms",
        budget.as_millis()
    ))
}

pub(crate) fn build_file_identity(
    path: &PathBuf,
    canonical_root: &PathBuf,
    display_root: &PathBuf,
) -> (PathBuf, Option<PathBuf>, PathResolution) {
    if let Ok(relative) = path.strip_prefix(canonical_root) {
        (
            display_root.join(relative),
            Some(relative.to_path_buf()),
            PathResolution::Canonical,
        )
    } else {
        (path.clone(), None, PathResolution::Fallback)
    }
}

pub(crate) fn attach_resolution_diagnostics(
    display_path: PathBuf,
    resolution: PathResolution,
    mut diagnostics: Vec<SearchDiagnostic>,
) -> Vec<SearchDiagnostic> {
    if resolution == PathResolution::Fallback {
        diagnostics.push(SearchDiagnostic::path_resolution_fallback(
            display_path,
            "Fell back to a non-canonical display path because root-relative rewriting was not possible.",
        ));
    }
    diagnostics
}

pub(crate) fn validate_ast_predicates(
    node: &AstNode,
    registry: &HashMap<PredicateKey, Box<dyn PredicateEvaluator + Send + Sync>>,
) -> Result<()> {
    match node {
        AstNode::Predicate(key, value) => {
            if !registry.contains_key(key) {
                if let PredicateKey::Other(name) = key {
                    return Err(anyhow!("Unknown predicate: '{name}'"));
                }
                return Err(anyhow!("Unknown predicate: '{}'", key.as_ref()));
            }
            predicates::validate_predicate_value(key, value)?;
        }
        AstNode::LogicalOp(_, left, right) => {
            validate_ast_predicates(left, registry)?;
            validate_ast_predicates(right, registry)?;
        }
        AstNode::Not(child) => {
            validate_ast_predicates(child, registry)?;
        }
    }
    Ok(())
}
