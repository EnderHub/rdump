use crate::backend::{build_directory_hotspots, DiscoveryRequest, SearchBackend};
use crate::evaluator::{Evaluator, FileContext, MatchResult};
use crate::parser::{self, AstNode, PredicateKey};
use crate::planner::resolve_effective_query;
use crate::predicates::code_aware::CodeAwareSettings;
use crate::predicates::{self, PredicateEvaluator};
use crate::{
    FileSnapshot, PathResolution, RawSearchItem, SearchCancellationToken, SearchDiagnostic,
    SearchOptions, SearchStats,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[allow(dead_code)]
pub(crate) struct RawSearchReport {
    pub(crate) results: Vec<RawSearchItem>,
    pub(crate) stats: SearchStats,
    pub(crate) diagnostics: Vec<SearchDiagnostic>,
}

#[derive(Clone)]
struct CandidateEntry {
    identity: crate::backend::BackendPathIdentity,
    estimated_bytes: usize,
}

pub(crate) struct SearchRawIterator {
    backend: Arc<dyn SearchBackend>,
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
    semantic_telemetry: Arc<crate::predicates::code_aware::SemanticTelemetry>,
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
        backend: Arc<dyn SearchBackend>,
        query: &str,
        options: &SearchOptions,
        cancellation: Option<SearchCancellationToken>,
    ) -> Result<Self> {
        let canonical_root = backend.normalize_root(&options.root)?;
        let query_to_parse = resolve_effective_query(query, options)?;
        let ast = crate::planner::optimize_ast(parser::parse_query(&query_to_parse)?);
        validate_ast_predicates(&ast, &predicates::create_predicate_registry())?;

        let walk_started = Instant::now();
        let discovery = backend.discover(&DiscoveryRequest {
            root: canonical_root.clone(),
            display_root: options.root.clone(),
            no_ignore: options.no_ignore,
            hidden: options.hidden,
            max_depth: options.max_depth,
            ignore_debug: options.ignore_debug,
        })?;

        let candidates: Vec<CandidateEntry> = discovery
            .candidates
            .into_iter()
            .map(|identity| {
                let estimated_bytes = identity.resolved_path.as_os_str().len()
                    + identity.display_path.as_os_str().len()
                    + identity
                        .root_relative_path
                        .as_ref()
                        .map(|path| path.as_os_str().len())
                        .unwrap_or_default()
                    + 64;
                CandidateEntry {
                    identity,
                    estimated_bytes,
                }
            })
            .collect();

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
                    .map(|candidate| candidate.identity.clone())
                    .collect::<Vec<_>>(),
            ),
            walk_millis: walk_started.elapsed().as_millis() as u64,
            ..Default::default()
        };

        let metadata_evaluator = Evaluator::new(
            ast.clone(),
            predicates::create_metadata_predicate_registry(),
        );
        let (query_cache_hits_before, query_cache_misses_before) =
            crate::predicates::code_aware::query_cache_metrics_snapshot();
        let semantic_telemetry =
            Arc::new(crate::predicates::code_aware::SemanticTelemetry::default());
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
        let full_evaluator = Evaluator::new(
            ast,
            predicates::create_predicate_registry_with_settings(code_settings),
        );

        let remaining_candidate_bytes = candidates
            .iter()
            .map(|candidate| candidate.estimated_bytes)
            .sum();
        stats.query_cache_hits = 0;
        stats.query_cache_misses = 0;

        Ok(Self {
            backend,
            candidates,
            next_candidate: 0,
            canonical_root,
            options: options.clone(),
            stats,
            diagnostics: discovery.diagnostics,
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
        self.stats.query_cache_hits = cache_hits_after.saturating_sub(self.query_cache_hits_before);
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

            let mut context = FileContext::with_backend(
                candidate.identity.resolved_path.clone(),
                self.canonical_root.clone(),
                Arc::clone(&self.backend),
            );
            context.display_path = candidate.identity.display_path.clone();

            let prefilter_started = Instant::now();
            let prefilter = self.metadata_evaluator.evaluate(&mut context);
            self.stats.prefilter_millis += prefilter_started.elapsed().as_millis() as u64;
            let prefilter_result = match prefilter {
                Ok(result) => result,
                Err(err) => {
                    self.refresh_runtime_stats();
                    return Some(Err(anyhow!(
                        "Error during pre-filter on {}: {}",
                        candidate.identity.resolved_path.display(),
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
                    context
                        .metadata()
                        .ok()
                        .map(FileSnapshot::from_backend_metadata)
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
                        && candidate.identity.resolution == PathResolution::Fallback
                    {
                        self.refresh_runtime_stats();
                        return Some(Err(anyhow!(
                            "Strict path resolution failed for {}",
                            candidate.identity.resolved_path.display()
                        )));
                    }
                    self.stats.matched_files += 1;
                    self.stats.whole_file_results += 1;
                    let diagnostics = attach_resolution_diagnostics(
                        candidate.identity.display_path.clone(),
                        candidate.identity.resolution,
                        path_diagnostics,
                    );
                    self.refresh_runtime_stats();
                    return Some(Ok(RawSearchItem {
                        backend: Arc::clone(&self.backend),
                        display_path: candidate.identity.display_path,
                        resolved_path: candidate.identity.resolved_path,
                        root_relative_path: candidate.identity.root_relative_path,
                        resolution: candidate.identity.resolution,
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
                        && candidate.identity.resolution == PathResolution::Fallback
                    {
                        self.refresh_runtime_stats();
                        return Some(Err(anyhow!(
                            "Strict path resolution failed for {}",
                            candidate.identity.resolved_path.display()
                        )));
                    }
                    self.stats.matched_files += 1;
                    self.stats.ranged_results += 1;
                    self.stats.matched_ranges += hunks.len();
                    let diagnostics = attach_resolution_diagnostics(
                        candidate.identity.display_path.clone(),
                        candidate.identity.resolution,
                        path_diagnostics,
                    );
                    self.refresh_runtime_stats();
                    return Some(Ok(RawSearchItem {
                        backend: Arc::clone(&self.backend),
                        display_path: candidate.identity.display_path,
                        resolved_path: candidate.identity.resolved_path,
                        root_relative_path: candidate.identity.root_relative_path,
                        resolution: candidate.identity.resolution,
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
                        candidate.identity.resolved_path.display(),
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
    SearchRawIterator::new(
        Arc::new(crate::backend::RealFsSearchBackend),
        query,
        options,
        cancellation,
    )
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
