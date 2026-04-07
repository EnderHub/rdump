use anyhow::Result;
use rdump_contracts::{
    CapabilityMetadata, ContractError, ErrorCode, ErrorEnvelope, ErrorMode, ErrorRemediation,
    ExecutionProfile, FileIdentity as ContractFileIdentity, LanguageCapabilityMatrix, LimitValue,
    Limits, LineEndingMode, MatchCoordinateSemantics, MatchInfo, OutputMode, PathDisplayMode,
    PathMetadata, PredicateCatalog, PredicateDescriptor, ProgressEvent,
    ResultKind as ContractResultKind, SearchDiagnostic as ContractDiagnostic, SearchItem,
    SearchRequest, SearchResponse, SearchStats as ContractStats, SearchStatus,
    SemanticSkipReason as ContractSemanticSkipReason, Snippet, SnippetMode, StabilityTier,
    SurfaceStability, SCHEMA_VERSION,
};
#[cfg(unix)]
use std::path::PathBuf;

use crate::{
    materialize_raw_search_item, SearchCancellationToken, SearchDiagnostic, SearchOptions,
    SearchResult, SearchRuntime,
};

pub const DEFAULT_MAX_RESULTS: usize = 50;
pub const DEFAULT_MAX_MATCHES_PER_FILE: usize = 20;
pub const DEFAULT_MAX_BYTES_PER_FILE: usize = 20_000;
pub const DEFAULT_MAX_TOTAL_BYTES: usize = 200_000;
pub const DEFAULT_MAX_MATCH_BYTES: usize = 200;
pub const DEFAULT_MAX_SNIPPET_BYTES: usize = 2_000;
pub const DEFAULT_MAX_ERRORS: usize = 10;
pub const DEFAULT_CONTEXT_LINES: usize = 2;

#[derive(Debug)]
pub struct ResolvedLimits {
    pub max_results: usize,
    pub max_matches_per_file: usize,
    pub max_bytes_per_file: usize,
    pub max_total_bytes: usize,
    pub max_match_bytes: usize,
    pub max_snippet_bytes: usize,
    pub max_errors: usize,
}

pub fn default_limits() -> Limits {
    Limits {
        max_results: LimitValue::Value(DEFAULT_MAX_RESULTS),
        max_matches_per_file: LimitValue::Value(DEFAULT_MAX_MATCHES_PER_FILE),
        max_bytes_per_file: LimitValue::Value(DEFAULT_MAX_BYTES_PER_FILE),
        max_total_bytes: LimitValue::Value(DEFAULT_MAX_TOTAL_BYTES),
        max_match_bytes: LimitValue::Value(DEFAULT_MAX_MATCH_BYTES),
        max_snippet_bytes: LimitValue::Value(DEFAULT_MAX_SNIPPET_BYTES),
        max_errors: LimitValue::Value(DEFAULT_MAX_ERRORS),
    }
}

pub fn capability_metadata() -> CapabilityMetadata {
    CapabilityMetadata {
        schema_version: SCHEMA_VERSION.to_string(),
        supported_outputs: vec![
            OutputMode::Paths,
            OutputMode::Matches,
            OutputMode::Snippets,
            OutputMode::Full,
            OutputMode::Summary,
        ],
        default_context_lines: DEFAULT_CONTEXT_LINES,
        stability: vec![
            SurfaceStability {
                surface: "cli".to_string(),
                tier: StabilityTier::Stable,
                semver_notes:
                    "Command names and default output flags follow semver. Human-readable help and prose may evolve."
                        .to_string(),
            },
            SurfaceStability {
                surface: "sdk".to_string(),
                tier: StabilityTier::Stable,
                semver_notes:
                    "Core search APIs are semver-stable. Legacy CLI-facing exports remain available but are compatibility-bound."
                        .to_string(),
            },
            SurfaceStability {
                surface: "mcp".to_string(),
                tier: StabilityTier::Provisional,
                semver_notes:
                    "Structured payloads are versioned by schema_version. Hosts should gate behavior on that field."
                        .to_string(),
            },
        ],
        default_limits: default_limits(),
        coordinate_semantics: coordinate_semantics(),
        execution_profiles: vec![
            ExecutionProfile::Interactive,
            ExecutionProfile::Batch,
            ExecutionProfile::Agent,
        ],
    }
}

pub fn search_options_from_request(request: &SearchRequest) -> SearchOptions {
    let mut options = SearchOptions {
        root: PathBuf::from(request.root.as_deref().unwrap_or(".")),
        presets: request.presets.clone(),
        no_ignore: request.no_ignore,
        hidden: request.hidden,
        max_depth: request.max_depth,
        sql_dialect: request.sql_dialect.map(Into::into),
        sql_strict: request.sql_strict,
        error_mode: request.error_mode,
        execution_budget_ms: request.execution_budget_ms,
        semantic_budget_ms: request.semantic_budget_ms,
        max_semantic_matches_per_file: request.max_semantic_matches_per_file,
        language_override: request.language_override.clone(),
        semantic_match_mode: request.semantic_match_mode,
        snippet_mode: request.snippet_mode,
        semantic_strict: request.semantic_strict,
        strict_path_resolution: request.strict_path_resolution,
        snapshot_drift_detection: request.snapshot_drift_detection,
        execution_profile: request.execution_profile,
        ignore_debug: request.ignore_debug,
        language_debug: request.language_debug,
        sql_trace: request.sql_trace,
    };
    apply_execution_profile(request, &mut options);
    options
}

#[derive(Debug)]
struct PendingSearchItem {
    item: SearchItem,
    path: String,
    diagnostics: Vec<ContractDiagnostic>,
    match_count: usize,
    approx_bytes: usize,
}

#[derive(Debug)]
pub struct SearchRequestPager {
    runtime: SearchRuntime,
    session_id: String,
    request: SearchRequest,
    output: OutputMode,
    limits: ResolvedLimits,
    context_lines: usize,
    root: String,
    effective_query: String,
    line_endings: LineEndingMode,
    raw_iter: crate::engine::SearchRawIterator,
    pending: Option<PendingSearchItem>,
    current_offset: usize,
    reported_engine_diagnostics: usize,
    started: bool,
    finished: bool,
}

impl SearchRequestPager {
    pub fn new(
        request: &SearchRequest,
        session_id: impl Into<String>,
        cancellation: Option<SearchCancellationToken>,
    ) -> Result<Self> {
        Self::with_runtime(SearchRuntime::real_fs(), request, session_id, cancellation)
    }

    pub fn with_runtime(
        runtime: SearchRuntime,
        request: &SearchRequest,
        session_id: impl Into<String>,
        cancellation: Option<SearchCancellationToken>,
    ) -> Result<Self> {
        let output = request.output.unwrap_or(OutputMode::Snippets);
        let limits = resolve_limits(request.limits.clone());
        let context_lines = request.context_lines.unwrap_or(DEFAULT_CONTEXT_LINES);
        let root = request.root.clone().unwrap_or_else(|| ".".to_string());
        let options = search_options_from_request(request);
        let explanation = crate::explain_query_with_runtime(&runtime, &request.query, &options)?;
        let raw_iter = runtime.search_raw_iter(&request.query, &options, cancellation)?;

        let mut pager = Self {
            runtime,
            session_id: session_id.into(),
            request: request.clone(),
            output,
            limits,
            context_lines,
            root,
            effective_query: explanation.effective_query,
            line_endings: request.line_endings.unwrap_or(LineEndingMode::Preserve),
            raw_iter,
            pending: None,
            current_offset: 0,
            reported_engine_diagnostics: 0,
            started: false,
            finished: false,
        };
        pager.skip_to_offset(request.offset)?;
        Ok(pager)
    }

    pub fn current_offset(&self) -> usize {
        self.current_offset
    }

    pub fn is_finished(&self) -> bool {
        self.finished && self.pending.is_none()
    }

    pub fn estimated_state_bytes(&self) -> usize {
        self.raw_iter.estimated_state_bytes()
            + self
                .pending
                .as_ref()
                .map(|item| item.approx_bytes)
                .unwrap_or(0)
            + self.session_id.len()
            + self.request.query.len()
            + self.root.len()
            + self.effective_query.len()
            + 256
    }

    pub fn next_page<F>(&mut self, mut progress: F) -> Result<SearchResponse>
    where
        F: FnMut(&ProgressEvent),
    {
        self.emit_start_progress(&mut progress);

        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut page_diagnostics = Vec::new();
        let mut truncated = false;
        let mut truncation_reason = None;
        let mut returned_matches = 0usize;
        let mut returned_bytes = 0usize;

        progress(&ProgressEvent::Phase {
            session_id: self.session_id.clone(),
            name: "materialize".to_string(),
            completed_items: self.current_offset,
            total_items: None,
        });

        loop {
            if results.len() >= self.limits.max_results {
                truncated = !self.is_finished();
                if truncated {
                    truncation_reason = Some("max_results".to_string());
                }
                break;
            }

            let entry = match self.poll_item() {
                Some(Ok(entry)) => entry,
                Some(Err(err)) => {
                    handle_error(&mut errors, &self.limits, self.request.error_mode, err)?;
                    continue;
                }
                None => break,
            };

            let next_bytes = returned_bytes.saturating_add(entry.approx_bytes);
            if !results.is_empty() && next_bytes > self.limits.max_total_bytes {
                self.pending = Some(entry);
                truncated = true;
                truncation_reason = Some("max_total_bytes".to_string());
                break;
            }

            returned_bytes = next_bytes;
            returned_matches += entry.match_count;
            page_diagnostics.extend(entry.diagnostics);
            results.push(entry.item);
            self.current_offset += 1;

            progress(&ProgressEvent::Result {
                session_id: self.session_id.clone(),
                path: entry.path,
                emitted_results: results.len(),
            });
        }

        if self.raw_iter.was_cancelled() {
            self.finished = true;
            truncated = true;
            truncation_reason.get_or_insert_with(|| "cancelled".to_string());
        }

        let mut diagnostics = self.take_engine_diagnostics();
        diagnostics.append(&mut page_diagnostics);

        let (
            whole_file_results,
            ranged_results,
            suppressed_too_large,
            suppressed_binary,
            suppressed_secret_like,
        ) = summarize_results_for_stats(&results);

        let engine_stats = self.raw_iter.stats().clone();
        let next_offset = if truncated && (!self.is_finished() || !results.is_empty()) {
            Some(self.current_offset)
        } else {
            None
        };

        let stats = ContractStats {
            returned_files: results.len(),
            returned_matches,
            returned_bytes,
            errors: errors.len(),
            whole_file_results,
            ranged_results,
            candidate_files: engine_stats.candidate_files,
            prefiltered_files: engine_stats.prefiltered_files,
            evaluated_files: engine_stats.evaluated_files,
            matched_files: engine_stats.matched_files,
            matched_ranges: engine_stats.matched_ranges,
            hidden_skipped: engine_stats.hidden_skipped,
            ignore_skipped: engine_stats.ignore_skipped,
            max_depth_skipped: engine_stats.max_depth_skipped,
            unreadable_entries: engine_stats.unreadable_entries,
            root_boundary_excluded: engine_stats.root_boundary_excluded,
            suppressed_too_large,
            suppressed_binary,
            suppressed_secret_like,
            diagnostics: diagnostics.len(),
            walk_millis: engine_stats.walk_millis,
            prefilter_millis: engine_stats.prefilter_millis,
            evaluate_millis: engine_stats.evaluate_millis,
            materialize_millis: engine_stats.materialize_millis,
            semantic_parse_failures: engine_stats.semantic_parse_failures,
            semantic_budget_exhaustions: engine_stats.semantic_budget_exhaustions,
            query_cache_hits: engine_stats.query_cache_hits,
            query_cache_misses: engine_stats.query_cache_misses,
            tree_cache_hits: engine_stats.tree_cache_hits,
            tree_cache_misses: engine_stats.tree_cache_misses,
            semaphore_wait_millis: engine_stats.semaphore_wait_millis,
            semantic_parse_failures_by_language: engine_stats.semantic_parse_failures_by_language,
            directory_hotspots: engine_stats.directory_hotspots,
        };

        progress(&ProgressEvent::Finished {
            session_id: self.session_id.clone(),
            returned_files: stats.returned_files,
            returned_matches: stats.returned_matches,
            truncated,
        });

        Ok(SearchResponse {
            schema_version: SCHEMA_VERSION.to_string(),
            schema_reference: "rdump://docs/sdk".to_string(),
            status: derive_status(&results, &errors, truncated),
            coordinate_semantics: coordinate_semantics(),
            query: self.request.query.clone(),
            effective_query: self.effective_query.clone(),
            root: self.root.clone(),
            output: self.output,
            error_mode: self.request.error_mode,
            results,
            stats,
            diagnostics,
            errors,
            truncated,
            truncation_reason,
            next_offset,
            continuation_token: next_offset.map(|offset| format!("offset:{offset}")),
            page_size: Some(self.limits.max_results),
        })
    }

    fn emit_start_progress<F>(&mut self, progress: &mut F)
    where
        F: FnMut(&ProgressEvent),
    {
        if self.started {
            return;
        }
        self.started = true;
        progress(&ProgressEvent::Started {
            session_id: self.session_id.clone(),
            query: self.request.query.clone(),
            effective_query: self.effective_query.clone(),
            root: self.root.clone(),
            queue_wait_millis: 0,
        });
        emit_engine_phase_progress(progress, self.raw_iter.stats(), &self.session_id);
    }

    fn skip_to_offset(&mut self, offset: usize) -> Result<()> {
        while self.current_offset < offset {
            match self.poll_item() {
                Some(Ok(_)) => {
                    self.current_offset += 1;
                }
                Some(Err(err)) => {
                    if self.request.error_mode == ErrorMode::FailFast {
                        return Err(err);
                    }
                }
                None => break,
            }
        }
        Ok(())
    }

    fn poll_item(&mut self) -> Option<Result<PendingSearchItem>> {
        if let Some(item) = self.pending.take() {
            return Some(Ok(item));
        }
        if self.finished {
            return None;
        }

        let raw = match self.raw_iter.next() {
            Some(raw) => raw,
            None => {
                self.finished = true;
                return None;
            }
        };

        Some(self.shape_item(raw))
    }

    fn shape_item(&self, raw: Result<crate::RawSearchItem>) -> Result<PendingSearchItem> {
        match self.output {
            OutputMode::Paths => {
                let raw = raw?;
                let file = crate::FileIdentity {
                    display_path: raw.display_path.clone(),
                    resolved_path: raw.resolved_path.clone(),
                    root_relative_path: raw.root_relative_path.clone(),
                    resolution: raw.resolution,
                };
                let backend_metadata = if raw.snapshot.is_none() {
                    Some(self.runtime.backend().stat(&raw.resolved_path)?)
                } else {
                    None
                };
                let item = SearchItem::Path {
                    path: render_contract_path(&file, self.request.path_display),
                    file: map_file_identity(&file),
                    fingerprint: raw
                        .snapshot
                        .as_ref()
                        .map(|snapshot| {
                            snapshot.stable_token.clone().unwrap_or_else(|| {
                                format!(
                                    "{}:{}:{}:{}:{}",
                                    snapshot.len,
                                    snapshot.modified_unix_millis.unwrap_or_default(),
                                    snapshot.readonly,
                                    snapshot.device_id.unwrap_or_default(),
                                    snapshot.inode.unwrap_or_default()
                                )
                            })
                        })
                        .or_else(|| {
                            backend_metadata.as_ref().map(|metadata| {
                                backend_path_fingerprint(metadata, &raw.resolved_path)
                            })
                        })
                        .unwrap_or_else(|| raw.resolved_path.display().to_string()),
                    metadata: if let Some(snapshot) = raw.snapshot.as_ref() {
                        snapshot.to_path_metadata()
                    } else if let Some(metadata) = backend_metadata.as_ref() {
                        metadata.to_path_metadata()
                    } else {
                        path_metadata(self.runtime.backend().as_ref(), &raw.resolved_path)?
                    },
                    result_kind: if raw.ranges.is_empty() {
                        ContractResultKind::WholeFile
                    } else {
                        ContractResultKind::Ranged
                    },
                    item_truncated: false,
                };
                let path = item_path(&item).to_string();
                let diagnostics = raw
                    .diagnostics
                    .iter()
                    .map(map_diagnostic)
                    .collect::<Vec<_>>();
                Ok(PendingSearchItem {
                    approx_bytes: estimate_search_item_bytes(&item),
                    item,
                    path,
                    diagnostics,
                    match_count: 0,
                })
            }
            _ => {
                let result = materialize_raw_search_item(raw)?;
                let path = result.path.display().to_string();
                let diagnostics = result
                    .diagnostics
                    .iter()
                    .map(map_diagnostic)
                    .collect::<Vec<_>>();
                let (item, match_count) = build_item(
                    self.runtime.backend().as_ref(),
                    self.output,
                    &result,
                    self.context_lines,
                    self.request.snippet_mode,
                    &self.limits,
                    self.request.path_display,
                    self.line_endings,
                    self.request.include_match_text,
                )?;
                Ok(PendingSearchItem {
                    approx_bytes: estimate_search_item_bytes(&item),
                    item,
                    path,
                    diagnostics,
                    match_count,
                })
            }
        }
    }

    fn take_engine_diagnostics(&mut self) -> Vec<ContractDiagnostic> {
        let diagnostics = self.raw_iter.diagnostics();
        let new_diagnostics = diagnostics[self.reported_engine_diagnostics..]
            .iter()
            .map(map_diagnostic)
            .collect();
        self.reported_engine_diagnostics = diagnostics.len();
        new_diagnostics
    }
}

pub fn execute_search_request(request: &SearchRequest) -> Result<SearchResponse> {
    execute_search_request_with_progress(request, |_| {})
}

pub fn execute_search_request_with_runtime(
    runtime: SearchRuntime,
    request: &SearchRequest,
) -> Result<SearchResponse> {
    execute_search_request_with_runtime_and_progress(runtime, request, |_| {})
}

pub fn execute_search_request_with_progress<F>(
    request: &SearchRequest,
    mut progress: F,
) -> Result<SearchResponse>
where
    F: FnMut(&ProgressEvent),
{
    execute_search_request_with_runtime_and_progress(
        SearchRuntime::real_fs(),
        request,
        &mut progress,
    )
}

pub fn execute_search_request_with_runtime_and_progress<F>(
    runtime: SearchRuntime,
    request: &SearchRequest,
    mut progress: F,
) -> Result<SearchResponse>
where
    F: FnMut(&ProgressEvent),
{
    execute_search_request_with_runtime_and_cancellation(
        runtime,
        request,
        None,
        "sync-request",
        &mut progress,
    )
}

pub fn execute_search_request_with_progress_and_cancellation<F>(
    request: &SearchRequest,
    cancellation: Option<SearchCancellationToken>,
    session_id: &str,
    mut progress: F,
) -> Result<SearchResponse>
where
    F: FnMut(&ProgressEvent),
{
    execute_search_request_with_runtime_and_cancellation(
        SearchRuntime::real_fs(),
        request,
        cancellation,
        session_id,
        &mut progress,
    )
}

pub fn execute_search_request_with_runtime_and_cancellation<F>(
    runtime: SearchRuntime,
    request: &SearchRequest,
    cancellation: Option<SearchCancellationToken>,
    session_id: &str,
    mut progress: F,
) -> Result<SearchResponse>
where
    F: FnMut(&ProgressEvent),
{
    let mut pager =
        SearchRequestPager::with_runtime(runtime, request, session_id.to_string(), cancellation)?;
    pager.next_page(&mut progress)
}

pub fn format_search_text(response: &SearchResponse) -> String {
    let mut lines = vec![
        format!("Query: {}", response.query),
        format!("Effective query: {}", response.effective_query),
        format!("Root: {}", response.root),
        format!("Output: {}", output_mode_label(response.output)),
        format!(
            "Results: {} files, {} matches, {} bytes",
            response.stats.returned_files,
            response.stats.returned_matches,
            response.stats.returned_bytes
        ),
        format!(
            "Engine: {} candidates, {} prefiltered, {} evaluated, {} matched files",
            response.stats.candidate_files,
            response.stats.prefiltered_files,
            response.stats.evaluated_files,
            response.stats.matched_files
        ),
        format!(
            "Discovery buckets: hidden_skipped={} ignore_skipped={} max_depth_skipped={} root_boundary_excluded={} unreadable_entries={}",
            response.stats.hidden_skipped,
            response.stats.ignore_skipped,
            response.stats.max_depth_skipped,
            response.stats.root_boundary_excluded,
            response.stats.unreadable_entries
        ),
        format!(
            "Timing: walk={}ms prefilter={}ms evaluate={}ms materialize={}ms",
            response.stats.walk_millis,
            response.stats.prefilter_millis,
            response.stats.evaluate_millis,
            response.stats.materialize_millis
        ),
    ];

    if response.stats.suppressed_too_large > 0
        || response.stats.suppressed_binary > 0
        || response.stats.suppressed_secret_like > 0
    {
        lines.push(format!(
            "Suppressed content: too_large={} binary={} secret_like={}",
            response.stats.suppressed_too_large,
            response.stats.suppressed_binary,
            response.stats.suppressed_secret_like
        ));
    }
    if !response.stats.directory_hotspots.is_empty() {
        lines.push(format!(
            "Hotspots: {}",
            response
                .stats
                .directory_hotspots
                .iter()
                .take(5)
                .map(|entry| format!("{} ({})", entry.path, entry.candidate_files))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if response.truncated {
        lines.push(format!(
            "Truncated: true ({})",
            response.truncation_reason.as_deref().unwrap_or("unknown")
        ));
    }

    if !response.errors.is_empty() {
        lines.push(format!("Errors: {}", response.errors.len()));
    }
    if !response.diagnostics.is_empty() {
        lines.push(format!("Diagnostics: {}", response.diagnostics.len()));
    }
    if !response.results.is_empty() {
        lines.push("Top results:".to_string());
        for path in response.results.iter().take(10).map(item_path) {
            lines.push(format!("- {path}"));
        }
        if response.results.len() > 10 {
            lines.push(format!("... and {} more", response.results.len() - 10));
        }
    }

    lines.join("\n")
}

fn emit_engine_phase_progress<F>(progress: &mut F, stats: &crate::SearchStats, session_id: &str)
where
    F: FnMut(&ProgressEvent),
{
    progress(&ProgressEvent::Phase {
        session_id: session_id.to_string(),
        name: "discover".to_string(),
        completed_items: stats.candidate_files,
        total_items: Some(stats.candidate_files),
    });
    progress(&ProgressEvent::Phase {
        session_id: session_id.to_string(),
        name: "prefilter".to_string(),
        completed_items: stats.prefiltered_files,
        total_items: Some(stats.candidate_files),
    });
    progress(&ProgressEvent::Phase {
        session_id: session_id.to_string(),
        name: "evaluate".to_string(),
        completed_items: stats.evaluated_files,
        total_items: Some(stats.prefiltered_files.max(stats.evaluated_files)),
    });
}

fn handle_error(
    errors: &mut Vec<ContractError>,
    limits: &ResolvedLimits,
    error_mode: ErrorMode,
    err: anyhow::Error,
) -> Result<()> {
    match error_mode {
        ErrorMode::SkipErrors => {
            if errors.len() < limits.max_errors {
                errors.push(contract_error(
                    ErrorCode::SearchExecution,
                    err.to_string(),
                    None,
                    false,
                    Some("Retry with fail-fast to stop on the first per-file error.".to_string()),
                ));
            }
            Ok(())
        }
        ErrorMode::FailFast => Err(err),
    }
}

fn build_item(
    backend: &dyn crate::backend::SearchBackend,
    output: OutputMode,
    result: &SearchResult,
    context_lines: usize,
    snippet_mode: SnippetMode,
    limits: &ResolvedLimits,
    path_display: Option<PathDisplayMode>,
    line_endings: LineEndingMode,
    include_match_text: bool,
) -> Result<(SearchItem, usize)> {
    let file = map_file_identity(result.file_identity());
    let path = render_contract_path(result.file_identity(), path_display);
    let fingerprint = result.metadata.fingerprint.clone();
    let result_kind = map_result_kind(result.result_kind());
    let semantic_skip_reasons = result
        .semantic_skip_reasons()
        .iter()
        .copied()
        .map(map_semantic_skip_reason)
        .collect::<Vec<_>>();
    match output {
        OutputMode::Summary => Ok((
            SearchItem::Summary {
                path,
                file,
                fingerprint,
                matches: result.match_count(),
                whole_file_match: result.is_whole_file_match(),
                result_kind,
                matches_truncated: result.match_count() > limits.max_matches_per_file,
                content_state: Some(content_state_label(&result.content_state)),
                diagnostic_count: result.diagnostics.len(),
                semantic_skip_reasons,
                item_truncated: false,
            },
            result.match_count(),
        )),
        OutputMode::Matches => {
            let matches = shape_matches(result, limits, include_match_text);
            let match_count = matches.len();
            Ok((
                SearchItem::Matches {
                    path,
                    file,
                    fingerprint,
                    matches,
                    whole_file_match: result.is_whole_file_match(),
                    result_kind,
                    matches_truncated: result.match_count() > limits.max_matches_per_file,
                    content_state: Some(content_state_label(&result.content_state)),
                    diagnostic_count: result.diagnostics.len(),
                    semantic_skip_reasons,
                    item_truncated: result.match_count() > limits.max_matches_per_file,
                },
                match_count,
            ))
        }
        OutputMode::Snippets => {
            let snippets =
                shape_snippets(result, context_lines, snippet_mode, limits, line_endings);
            let match_count = snippets.len();
            Ok((
                SearchItem::Snippets {
                    path,
                    file,
                    fingerprint,
                    snippets,
                    whole_file_match: result.is_whole_file_match(),
                    result_kind,
                    matches_truncated: result.match_count() > limits.max_matches_per_file,
                    content_state: Some(content_state_label(&result.content_state)),
                    diagnostic_count: result.diagnostics.len(),
                    semantic_skip_reasons,
                    item_truncated: result.match_count() > limits.max_matches_per_file,
                },
                match_count,
            ))
        }
        OutputMode::Full => {
            let matches = shape_matches(result, limits, include_match_text);
            let match_count = matches.len();
            let rendered_content = apply_line_endings(&result.content, line_endings);
            let content_truncated = rendered_content.len() > limits.max_bytes_per_file;
            let content = truncate_string(&rendered_content, limits.max_bytes_per_file);
            Ok((
                SearchItem::Full {
                    path,
                    file,
                    fingerprint,
                    content,
                    matches,
                    content_truncated,
                    matches_truncated: result.match_count() > limits.max_matches_per_file,
                    result_kind,
                    content_state: Some(content_state_label(&result.content_state)),
                    diagnostic_count: result.diagnostics.len(),
                    semantic_skip_reasons,
                    item_truncated: content_truncated
                        || result.match_count() > limits.max_matches_per_file,
                },
                match_count,
            ))
        }
        OutputMode::Paths => Ok((
            SearchItem::Path {
                path,
                file,
                fingerprint,
                metadata: if let Some(snapshot) = result.metadata.snapshot.as_ref() {
                    snapshot.to_path_metadata()
                } else {
                    path_metadata(backend, &result.file_identity().resolved_path)?
                },
                result_kind,
                item_truncated: false,
            },
            0,
        )),
    }
}

fn shape_matches(
    result: &SearchResult,
    limits: &ResolvedLimits,
    include_match_text: bool,
) -> Vec<MatchInfo> {
    result
        .matches
        .iter()
        .take(limits.max_matches_per_file)
        .map(|matched| {
            let text_truncated = matched.text.len() > limits.max_match_bytes;
            MatchInfo {
                start_line: matched.start_line,
                end_line: matched.end_line,
                start_column: matched.start_column,
                end_column: matched.end_column,
                byte_range: [matched.byte_range.start, matched.byte_range.end],
                text: if !include_match_text || matched.text.is_empty() {
                    None
                } else {
                    Some(truncate_string(&matched.text, limits.max_match_bytes))
                },
                text_truncated,
            }
        })
        .collect()
}

fn shape_snippets(
    result: &SearchResult,
    context_lines: usize,
    snippet_mode: SnippetMode,
    limits: &ResolvedLimits,
    line_endings: LineEndingMode,
) -> Vec<Snippet> {
    if !result.content_available() || result.matches.is_empty() {
        return Vec::new();
    }

    let lines = split_lines_with_endings(&result.content);
    result
        .matches
        .iter()
        .take(limits.max_matches_per_file)
        .map(|matched| {
            let range = snippet_range(matched, lines.len(), context_lines);
            let text = lines[range.clone()].concat();
            let line_ending = detect_line_ending(&text);
            let text_truncated = text.len() > limits.max_snippet_bytes;
            let text = if result.content.is_empty() {
                String::new()
            } else {
                let shaped = match snippet_mode {
                    SnippetMode::Normalized => text.replace("\r\n", "\n").replace('\r', "\n"),
                    SnippetMode::PreserveLineEndings => text,
                };
                truncate_string(
                    &apply_line_endings(&shaped, line_endings),
                    limits.max_snippet_bytes,
                )
            };
            Snippet {
                start_line: range.start + 1,
                end_line: range.end,
                match_start_line: matched.start_line,
                match_end_line: matched.end_line,
                text,
                text_truncated,
                line_ending,
            }
        })
        .collect()
}

fn snippet_range(
    matched: &crate::Match,
    total_lines: usize,
    context_lines: usize,
) -> std::ops::Range<usize> {
    let start_line = matched.start_line.saturating_sub(1);
    let end_line = matched.end_line.saturating_sub(1);
    let context_start = start_line.saturating_sub(context_lines);
    let context_end = (end_line + context_lines).min(total_lines.saturating_sub(1));
    context_start..context_end + 1
}

fn split_lines_with_endings(content: &str) -> Vec<String> {
    if content.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut start = 0;
    let bytes = content.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\n' {
            lines.push(content[start..=index].to_string());
            index += 1;
            start = index;
            continue;
        }
        index += 1;
    }
    if start < content.len() {
        lines.push(content[start..].to_string());
    }
    lines
}

fn detect_line_ending(text: &str) -> String {
    if text.contains("\r\n") {
        "crlf".to_string()
    } else if text.contains('\n') {
        "lf".to_string()
    } else if text.contains('\r') {
        "cr".to_string()
    } else {
        "none".to_string()
    }
}

fn render_contract_path(
    identity: &crate::FileIdentity,
    path_display: Option<PathDisplayMode>,
) -> String {
    match path_display.unwrap_or(PathDisplayMode::Relative) {
        PathDisplayMode::Relative => identity.display_path.display().to_string(),
        PathDisplayMode::Absolute => identity.resolved_path.display().to_string(),
        PathDisplayMode::RootRelative => identity
            .root_relative_path
            .as_ref()
            .unwrap_or(&identity.display_path)
            .display()
            .to_string(),
    }
}

fn apply_line_endings(text: &str, mode: LineEndingMode) -> String {
    match mode {
        LineEndingMode::Preserve => text.to_string(),
        LineEndingMode::Normalize => text.replace("\r\n", "\n").replace('\r', "\n"),
    }
}

fn summarize_results_for_stats(results: &[SearchItem]) -> (usize, usize, usize, usize, usize) {
    let mut whole_file_results = 0usize;
    let mut ranged_results = 0usize;
    let mut suppressed_too_large = 0usize;
    let mut suppressed_binary = 0usize;
    let mut suppressed_secret_like = 0usize;

    for result in results {
        let (whole_file_match, content_state) = match result {
            SearchItem::Path { .. } => (true, None),
            SearchItem::Summary {
                whole_file_match,
                content_state,
                ..
            }
            | SearchItem::Matches {
                whole_file_match,
                content_state,
                ..
            }
            | SearchItem::Snippets {
                whole_file_match,
                content_state,
                ..
            } => (*whole_file_match, content_state.as_deref()),
            SearchItem::Full {
                result_kind,
                content_state,
                ..
            } => (
                matches!(result_kind, rdump_contracts::ResultKind::WholeFile),
                content_state.as_deref(),
            ),
        };

        if whole_file_match {
            whole_file_results += 1;
        } else {
            ranged_results += 1;
        }

        match content_state {
            Some(state) if state == "skipped:too_large" => suppressed_too_large += 1,
            Some(state) if state == "skipped:binary" => suppressed_binary += 1,
            Some(state) if state == "skipped:secret_like" => suppressed_secret_like += 1,
            _ => {}
        }
    }

    (
        whole_file_results,
        ranged_results,
        suppressed_too_large,
        suppressed_binary,
        suppressed_secret_like,
    )
}

fn content_state_label(state: &crate::ContentState) -> String {
    match state {
        crate::ContentState::Loaded => "loaded".to_string(),
        crate::ContentState::LoadedLossy => "loaded_lossy".to_string(),
        crate::ContentState::Skipped { reason } => format!("skipped:{}", reason.as_str()),
    }
}

fn resolve_limits(limits: Option<Limits>) -> ResolvedLimits {
    let limits = limits.unwrap_or_else(default_limits);
    ResolvedLimits {
        max_results: resolve_limit(limits.max_results, DEFAULT_MAX_RESULTS),
        max_matches_per_file: resolve_limit(
            limits.max_matches_per_file,
            DEFAULT_MAX_MATCHES_PER_FILE,
        ),
        max_bytes_per_file: resolve_limit(limits.max_bytes_per_file, DEFAULT_MAX_BYTES_PER_FILE),
        max_total_bytes: resolve_limit(limits.max_total_bytes, DEFAULT_MAX_TOTAL_BYTES),
        max_match_bytes: resolve_limit(limits.max_match_bytes, DEFAULT_MAX_MATCH_BYTES),
        max_snippet_bytes: resolve_limit(limits.max_snippet_bytes, DEFAULT_MAX_SNIPPET_BYTES),
        max_errors: resolve_limit(limits.max_errors, DEFAULT_MAX_ERRORS),
    }
}

fn resolve_limit(value: LimitValue, default: usize) -> usize {
    match value {
        LimitValue::Value(value) => value,
        LimitValue::Unlimited => usize::MAX,
        LimitValue::Unset => default,
    }
}

fn truncate_string(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }

    let mut boundary = max_bytes;
    while !text.is_char_boundary(boundary) && boundary > 0 {
        boundary -= 1;
    }
    text[..boundary].to_string()
}

fn map_diagnostic(diagnostic: &SearchDiagnostic) -> ContractDiagnostic {
    ContractDiagnostic {
        level: format!("{:?}", diagnostic.level).to_lowercase(),
        kind: format!("{:?}", diagnostic.kind).to_lowercase(),
        message: diagnostic.message.clone(),
        path: diagnostic
            .path
            .as_ref()
            .map(|path| path.display().to_string()),
    }
}

pub(crate) fn estimate_search_item_bytes(item: &SearchItem) -> usize {
    const SCALAR_BYTES: usize = 8;

    fn string_bytes(value: &str) -> usize {
        value.len() + 2
    }

    fn option_string_bytes(value: Option<&String>) -> usize {
        value.map(|value| string_bytes(value)).unwrap_or(1)
    }

    fn file_identity_bytes(file: &ContractFileIdentity) -> usize {
        string_bytes(&file.display_path)
            + string_bytes(&file.resolved_path)
            + option_string_bytes(file.root_relative_path.as_ref())
            + SCALAR_BYTES
    }

    fn path_metadata_bytes(metadata: &PathMetadata) -> usize {
        string_bytes(&metadata.permissions_display)
            + metadata
                .modified_unix_millis
                .map(|_| SCALAR_BYTES)
                .unwrap_or(1)
            + SCALAR_BYTES * 3
    }

    fn match_info_bytes(info: &MatchInfo) -> usize {
        option_string_bytes(info.text.as_ref()) + SCALAR_BYTES * 7
    }

    fn snippet_bytes(snippet: &Snippet) -> usize {
        string_bytes(&snippet.text) + string_bytes(&snippet.line_ending) + SCALAR_BYTES * 4
    }

    match item {
        SearchItem::Path {
            path,
            file,
            fingerprint,
            metadata,
            ..
        } => {
            string_bytes(path)
                + file_identity_bytes(file)
                + string_bytes(fingerprint)
                + path_metadata_bytes(metadata)
                + SCALAR_BYTES * 4
        }
        SearchItem::Summary {
            path,
            file,
            fingerprint,
            content_state,
            semantic_skip_reasons,
            ..
        } => {
            string_bytes(path)
                + file_identity_bytes(file)
                + string_bytes(fingerprint)
                + option_string_bytes(content_state.as_ref())
                + semantic_skip_reasons.len() * SCALAR_BYTES
                + SCALAR_BYTES * 8
        }
        SearchItem::Matches {
            path,
            file,
            fingerprint,
            matches,
            content_state,
            semantic_skip_reasons,
            ..
        } => {
            string_bytes(path)
                + file_identity_bytes(file)
                + string_bytes(fingerprint)
                + matches.iter().map(match_info_bytes).sum::<usize>()
                + option_string_bytes(content_state.as_ref())
                + semantic_skip_reasons.len() * SCALAR_BYTES
                + SCALAR_BYTES * 8
        }
        SearchItem::Snippets {
            path,
            file,
            fingerprint,
            snippets,
            content_state,
            semantic_skip_reasons,
            ..
        } => {
            string_bytes(path)
                + file_identity_bytes(file)
                + string_bytes(fingerprint)
                + snippets.iter().map(snippet_bytes).sum::<usize>()
                + option_string_bytes(content_state.as_ref())
                + semantic_skip_reasons.len() * SCALAR_BYTES
                + SCALAR_BYTES * 8
        }
        SearchItem::Full {
            path,
            file,
            fingerprint,
            content,
            matches,
            content_state,
            semantic_skip_reasons,
            ..
        } => {
            string_bytes(path)
                + file_identity_bytes(file)
                + string_bytes(fingerprint)
                + string_bytes(content)
                + matches.iter().map(match_info_bytes).sum::<usize>()
                + option_string_bytes(content_state.as_ref())
                + semantic_skip_reasons.len() * SCALAR_BYTES
                + SCALAR_BYTES * 10
        }
    }
}

pub fn coordinate_semantics() -> MatchCoordinateSemantics {
    MatchCoordinateSemantics {
        line_numbers: "1-indexed lines".to_string(),
        columns: "0-indexed byte offsets within the line".to_string(),
        byte_ranges: "0-indexed byte offsets within the file".to_string(),
    }
}

fn derive_status(
    results: &[SearchItem],
    errors: &[ContractError],
    truncated: bool,
) -> SearchStatus {
    if truncated {
        return SearchStatus::TruncatedSuccess;
    }
    if !errors.is_empty() {
        return SearchStatus::PartialSuccess;
    }
    if !results.is_empty() && results.iter().all(item_is_policy_suppressed) {
        return SearchStatus::PolicySuppressed;
    }
    SearchStatus::FullSuccess
}

fn item_is_policy_suppressed(item: &SearchItem) -> bool {
    match item {
        SearchItem::Summary { content_state, .. }
        | SearchItem::Matches { content_state, .. }
        | SearchItem::Snippets { content_state, .. }
        | SearchItem::Full { content_state, .. } => content_state
            .as_deref()
            .is_some_and(|state| state.starts_with("skipped:")),
        SearchItem::Path { .. } => false,
    }
}

fn map_file_identity(identity: &crate::FileIdentity) -> ContractFileIdentity {
    ContractFileIdentity {
        display_path: identity.display_path.display().to_string(),
        resolved_path: identity.resolved_path.display().to_string(),
        root_relative_path: identity
            .root_relative_path
            .as_ref()
            .map(|path| path.display().to_string()),
        resolution: match identity.resolution {
            crate::PathResolution::Canonical => rdump_contracts::PathResolution::Canonical,
            crate::PathResolution::Fallback => rdump_contracts::PathResolution::Fallback,
        },
    }
}

fn map_result_kind(kind: crate::ResultKind) -> ContractResultKind {
    match kind {
        crate::ResultKind::WholeFile => ContractResultKind::WholeFile,
        crate::ResultKind::Ranged => ContractResultKind::Ranged,
    }
}

fn map_semantic_skip_reason(reason: crate::SemanticSkipReason) -> ContractSemanticSkipReason {
    match reason {
        crate::SemanticSkipReason::UnsupportedLanguage => {
            ContractSemanticSkipReason::UnsupportedLanguage
        }
        crate::SemanticSkipReason::ParseFailed => ContractSemanticSkipReason::ParseFailed,
        crate::SemanticSkipReason::ContentUnavailable => {
            ContractSemanticSkipReason::ContentUnavailable
        }
        crate::SemanticSkipReason::BudgetExhausted => ContractSemanticSkipReason::BudgetExhausted,
    }
}

fn apply_execution_profile(request: &SearchRequest, options: &mut SearchOptions) {
    match request.execution_profile {
        Some(ExecutionProfile::Interactive) => {}
        Some(ExecutionProfile::Batch) => {
            options.error_mode = ErrorMode::FailFast;
            options.execution_budget_ms.get_or_insert(60_000);
            options.semantic_budget_ms.get_or_insert(5_000);
        }
        Some(ExecutionProfile::Agent) => {
            options.error_mode = ErrorMode::SkipErrors;
            options.execution_budget_ms.get_or_insert(15_000);
            options.semantic_budget_ms.get_or_insert(1_500);
            options.max_semantic_matches_per_file.get_or_insert(25);
        }
        None => {}
    }
}

pub fn predicate_catalog() -> PredicateCatalog {
    let mut predicates = vec![
        PredicateDescriptor {
            name: "contains".to_string(),
            category: "content".to_string(),
            aliases: vec!["c".to_string()],
            deprecated_aliases: vec!["content".to_string()],
        },
        PredicateDescriptor {
            name: "matches".to_string(),
            category: "content".to_string(),
            aliases: vec!["m".to_string()],
            deprecated_aliases: Vec::new(),
        },
    ];
    for name in [
        "ext",
        "name",
        "path",
        "path_exact",
        "in",
        "size",
        "modified",
    ] {
        predicates.push(PredicateDescriptor {
            name: name.to_string(),
            category: "metadata".to_string(),
            aliases: Vec::new(),
            deprecated_aliases: Vec::new(),
        });
    }
    for name in [
        "def",
        "func",
        "import",
        "class",
        "struct",
        "enum",
        "interface",
        "trait",
        "type",
        "impl",
        "macro",
        "module",
        "object",
        "protocol",
        "comment",
        "str",
        "call",
        "component",
        "element",
        "hook",
        "customhook",
        "prop",
    ] {
        predicates.push(PredicateDescriptor {
            name: name.to_string(),
            category: "semantic".to_string(),
            aliases: Vec::new(),
            deprecated_aliases: Vec::new(),
        });
    }
    predicates.sort_by(|left, right| left.name.cmp(&right.name));
    PredicateCatalog {
        schema_version: SCHEMA_VERSION.to_string(),
        predicates,
    }
}

fn path_metadata(
    backend: &dyn crate::backend::SearchBackend,
    path: &std::path::Path,
) -> Result<PathMetadata> {
    Ok(backend.stat(path)?.to_path_metadata())
}

fn backend_path_fingerprint(
    metadata: &crate::backend::BackendMetadata,
    _resolved_path: &std::path::Path,
) -> String {
    metadata.stable_token.clone().unwrap_or_else(|| {
        format!(
            "{}:{}:{}:{}:{}",
            metadata.size_bytes,
            metadata.modified_unix_millis.unwrap_or_default(),
            metadata.readonly,
            metadata.device_id.unwrap_or_default(),
            metadata.inode.unwrap_or_default()
        )
    })
}

pub fn language_capability_matrix() -> LanguageCapabilityMatrix {
    LanguageCapabilityMatrix {
        schema_version: SCHEMA_VERSION.to_string(),
        capture_convention:
            "Language-profile queries that emit hunks must expose tree-sitter captures named @match."
                .to_string(),
        languages: crate::predicates::code_aware::profiles::list_canonical_language_profiles()
            .into_iter()
            .map(|profile| rdump_contracts::LanguageInfo {
                id: profile.id.to_string(),
                name: profile.profile.name.to_string(),
                extensions: profile
                    .profile
                    .extensions
                    .iter()
                    .map(|extension| extension.to_string())
                    .collect(),
                aliases: profile
                    .aliases
                    .iter()
                    .map(|alias| alias.to_string())
                    .collect(),
                support_tier: crate::predicates::code_aware::profiles::support_tier_for_id(
                    profile.id,
                ),
                predicates: rdump_contracts::LanguagePredicates {
                    metadata: vec![
                        "ext".to_string(),
                        "name".to_string(),
                        "path".to_string(),
                        "in".to_string(),
                        "size".to_string(),
                        "modified".to_string(),
                    ],
                    content: vec!["contains".to_string(), "matches".to_string()],
                    semantic: {
                        let mut semantic: Vec<String> = profile
                            .profile
                            .queries
                            .keys()
                            .map(|key| key.as_ref().to_string())
                            .collect();
                        semantic.sort();
                        semantic
                    },
                },
                semantic_caveats: crate::predicates::code_aware::profiles::semantic_caveats_for_id(
                    profile.id,
                )
                .into_iter()
                .map(str::to_string)
                .collect(),
            })
            .collect(),
    }
}

fn output_mode_label(mode: OutputMode) -> &'static str {
    match mode {
        OutputMode::Paths => "paths",
        OutputMode::Matches => "matches",
        OutputMode::Snippets => "snippets",
        OutputMode::Full => "full",
        OutputMode::Summary => "summary",
    }
}

fn item_path(item: &SearchItem) -> &str {
    match item {
        SearchItem::Path { path, .. }
        | SearchItem::Summary { path, .. }
        | SearchItem::Matches { path, .. }
        | SearchItem::Snippets { path, .. }
        | SearchItem::Full { path, .. } => path,
    }
}

pub fn contract_error(
    code: ErrorCode,
    message: impl Into<String>,
    field: Option<String>,
    retryable: bool,
    suggested_action: Option<String>,
) -> ContractError {
    ContractError {
        code,
        message: message.into(),
        field,
        remediation: ErrorRemediation {
            retryable,
            suggested_action,
            docs_uri: Some("rdump://docs/runtime".to_string()),
        },
    }
}

pub fn error_envelope(error: ContractError, status: SearchStatus) -> ErrorEnvelope {
    ErrorEnvelope {
        schema_version: SCHEMA_VERSION.to_string(),
        status,
        error,
    }
}

pub fn classify_error_message(message: &str) -> ContractError {
    let lower = message.to_ascii_lowercase();
    if lower.contains("query") || lower.contains("predicate") || lower.contains("syntax") {
        return contract_error(
            ErrorCode::QueryValidation,
            message.to_string(),
            None,
            false,
            Some("Run `rdump query explain` or `validate_query` to inspect the query.".to_string()),
        );
    }
    if lower.contains("time budget") {
        return contract_error(
            ErrorCode::SearchBudgetExceeded,
            message.to_string(),
            None,
            true,
            Some(
                "Increase execution_budget_ms or narrow the query with ext:/preset filters."
                    .to_string(),
            ),
        );
    }
    if lower.contains("cancel") {
        return contract_error(
            ErrorCode::SearchCancelled,
            message.to_string(),
            None,
            true,
            Some("Retry the search or increase the host timeout.".to_string()),
        );
    }
    contract_error(
        ErrorCode::SearchExecution,
        message.to_string(),
        None,
        false,
        Some("Review diagnostics and try --fail-fast for the first failing file.".to_string()),
    )
}

pub fn classify_error(err: &anyhow::Error) -> ContractError {
    classify_error_message(&err.to_string())
}
