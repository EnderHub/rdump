use crate::types::{ErrorMode, SearchArgs, SearchRequest};
use rdump::contracts::ProgressEvent;
use rdump::request::format_search_text;
use turbomcp::prelude::{McpError, McpResult};

pub fn build_search_request(args: SearchArgs) -> McpResult<SearchRequest> {
    let query = normalize_query(args.query, args.presets.as_ref())?;
    let error_mode = args
        .error_mode
        .or_else(|| {
            args.skip_errors.map(|skip| {
                if skip {
                    ErrorMode::SkipErrors
                } else {
                    ErrorMode::FailFast
                }
            })
        })
        .unwrap_or(ErrorMode::SkipErrors);

    let offset = args.offset.unwrap_or_else(|| {
        args.continuation_token
            .as_deref()
            .and_then(parse_continuation_offset)
            .unwrap_or(0)
    });

    Ok(SearchRequest {
        query,
        root: args.root,
        presets: args.presets.unwrap_or_default(),
        no_ignore: args.no_ignore.unwrap_or(false),
        hidden: args.hidden.unwrap_or(false),
        max_depth: args.max_depth,
        sql_dialect: args.sql_dialect,
        sql_strict: args.sql_strict.unwrap_or(false),
        output: args.output,
        limits: args.limits,
        context_lines: args.context_lines,
        error_mode,
        execution_budget_ms: args.execution_budget_ms,
        semantic_budget_ms: args.semantic_budget_ms,
        max_semantic_matches_per_file: args.max_semantic_matches_per_file,
        language_override: args.language_override,
        semantic_match_mode: args.semantic_match_mode.unwrap_or_default(),
        snippet_mode: args.snippet_mode.unwrap_or_default(),
        semantic_strict: args.semantic_strict.unwrap_or(false),
        strict_path_resolution: args.strict_path_resolution.unwrap_or(false),
        snapshot_drift_detection: args.snapshot_drift_detection.unwrap_or(true),
        ignore_debug: args.ignore_debug.unwrap_or(false),
        language_debug: args.language_debug.unwrap_or(false),
        sql_trace: args.sql_trace.unwrap_or(false),
        execution_profile: args.execution_profile,
        offset,
        continuation_token: args.continuation_token,
        path_display: args.path_display,
        line_endings: args.line_endings,
        include_match_text: args.include_match_text.unwrap_or(true),
    })
}

pub fn run_search(params: SearchRequest) -> McpResult<rdump::contracts::SearchResponse> {
    run_search_with_runtime(params, rdump::SearchRuntime::real_fs())
}

pub fn run_search_with_runtime(
    params: SearchRequest,
    runtime: rdump::SearchRuntime,
) -> McpResult<rdump::contracts::SearchResponse> {
    run_search_with_runtime_and_cancellation(params, runtime, rdump::SearchCancellationToken::new())
}

pub fn run_search_with_cancellation(
    params: SearchRequest,
    cancellation: rdump::SearchCancellationToken,
) -> McpResult<rdump::contracts::SearchResponse> {
    run_search_with_runtime_and_cancellation(params, rdump::SearchRuntime::real_fs(), cancellation)
}

pub fn run_search_with_runtime_and_cancellation(
    params: SearchRequest,
    runtime: rdump::SearchRuntime,
    cancellation: rdump::SearchCancellationToken,
) -> McpResult<rdump::contracts::SearchResponse> {
    run_search_with_runtime_and_cancellation_and_progress(params, runtime, cancellation, |_| {})
}

pub fn run_search_with_cancellation_and_progress<F>(
    params: SearchRequest,
    cancellation: rdump::SearchCancellationToken,
    mut progress: F,
) -> McpResult<rdump::contracts::SearchResponse>
where
    F: FnMut(&ProgressEvent),
{
    run_search_with_runtime_and_cancellation_and_progress(
        params,
        rdump::SearchRuntime::real_fs(),
        cancellation,
        &mut progress,
    )
}

pub fn run_search_with_runtime_and_cancellation_and_progress<F>(
    params: SearchRequest,
    runtime: rdump::SearchRuntime,
    cancellation: rdump::SearchCancellationToken,
    mut progress: F,
) -> McpResult<rdump::contracts::SearchResponse>
where
    F: FnMut(&ProgressEvent),
{
    rdump::request::execute_search_request_with_runtime_and_cancellation(
        runtime,
        &params,
        Some(cancellation),
        "mcp-search",
        |event| progress(event),
    )
    .map_err(|err| McpError::tool_execution_failed("search", err.to_string()))
}

pub fn format_search_response_text(response: &rdump::contracts::SearchResponse) -> String {
    format_search_text(response)
}

fn normalize_query(query: Option<String>, presets: Option<&Vec<String>>) -> McpResult<String> {
    match query {
        Some(query) if !query.trim().is_empty() => Ok(query),
        _ if presets.is_some_and(|presets| !presets.is_empty()) => Ok(String::new()),
        _ => Err(McpError::invalid_request(
            "search requires a non-empty query or at least one preset",
        )),
    }
}

fn parse_continuation_offset(token: &str) -> Option<usize> {
    token.strip_prefix("offset:")?.parse::<usize>().ok()
}
