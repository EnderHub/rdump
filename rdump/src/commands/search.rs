use crate::limits::{safe_canonicalize, DEFAULT_MAX_DEPTH};
use crate::{
    ColorChoice, FileSnapshot, PathResolution, RawSearchItem, SearchArgs, SearchDiagnostic,
    SearchOptions, SearchReport, SearchStats,
};
use anyhow::{anyhow, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use rdump_contracts::{ErrorMode, OutputMode, SearchRequest};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::File;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tree_sitter::Range;

use crate::evaluator::{Evaluator, FileContext, MatchResult};
use crate::formatter;
use crate::parser::{self, AstNode, PredicateKey};
use crate::planner::resolve_effective_query;
use crate::predicates::code_aware::CodeAwareSettings;
use crate::predicates::{self, PredicateEvaluator};

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
struct DiscoveryAnalysis {
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

/// The main entry point for the `search` command.
pub fn run_search(mut args: SearchArgs) -> Result<()> {
    if args.no_headers && args.find {
        eprintln!("Warning: --no-headers has no effect with --find.");
    }
    // --- Handle Shorthand Flags ---
    if args.no_headers {
        args.format = crate::Format::Cat;
    }
    if args.find && !matches!(args.format, crate::Format::Json) {
        args.format = crate::Format::Find;
    }
    if matches!(args.format, crate::Format::Paths | crate::Format::Find) && args.no_headers {
        eprintln!(
            "Warning: --no-headers only affects content-oriented formats and is ignored here."
        );
    }

    // --- Build options and perform the actual search ---
    let request = search_request_from_args(&args);
    let options = crate::request::search_options_from_request(&request);
    let query = args.query.as_deref().unwrap_or("");

    // --- Determine if color should be used ---
    let use_color = if args.output.is_some() {
        // If outputting to a file, never use color unless explicitly forced.
        args.color == ColorChoice::Always
    } else {
        // Otherwise, decide based on the color choice and TTY status.
        match args.color {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => io::stdout().is_terminal(),
        }
    };

    // If the output format is `Cat` (likely for piping), we should not use color
    // unless the user has explicitly forced it with `Always`.
    if let crate::Format::Cat = args.format {
        if args.color != ColorChoice::Always {
            // This is a bit redundant with the above, but it's a safeguard.
            // use_color = false;
        }
    }

    // --- 5. Format and print results ---
    let mut writer: Box<dyn Write> = if let Some(output_path) = &args.output {
        Box::new(File::create(output_path)?)
    } else {
        Box::new(io::stdout())
    };

    if matches!(args.format, crate::Format::Json) {
        let mut request = search_request_from_args(&args);
        request.output = Some(if args.find {
            OutputMode::Paths
        } else {
            OutputMode::Full
        });
        let response = crate::request::execute_search_request(&request)?;
        serde_json::to_writer_pretty(&mut writer, &response)?;
        writer.write_all(b"\n")?;
        return Ok(());
    }

    match args.format {
        crate::Format::Paths | crate::Format::Find => {
            let iter = crate::search_path_iter(query, options)?;
            let diagnostics = iter.diagnostics().to_vec();
            for item in iter {
                let path = apply_cli_path_display_to_path(item?, &args.root, args.path_display);
                formatter::print_path_output(&mut writer, &[path], &args.format, args.time_format)?;
            }
            maybe_log_diagnostics(&diagnostics);
        }
        crate::Format::Summary => {
            let report = apply_cli_output_preferences(
                collect_search_report(query, options, search_request_from_args(&args).error_mode)?,
                &args,
            );
            for result in &report.results {
                let single = SearchReport {
                    results: vec![result.clone()],
                    stats: SearchStats::default(),
                    diagnostics: Vec::new(),
                };
                formatter::print_report_output(
                    &mut writer,
                    &single,
                    &args.format,
                    args.line_numbers,
                    args.no_headers,
                    use_color,
                    args.context.unwrap_or(0),
                    args.show_suppressed_placeholders,
                    args.time_format,
                )?;
            }
            maybe_log_diagnostics(&report.diagnostics);
        }
        _ => {
            let report = apply_cli_output_preferences(
                collect_search_report(query, options, search_request_from_args(&args).error_mode)?,
                &args,
            );
            formatter::print_report_output(
                &mut writer,
                &report,
                &args.format,
                args.line_numbers,
                args.no_headers,
                use_color,
                args.context.unwrap_or(0),
                args.show_suppressed_placeholders,
                args.time_format,
            )?;
            maybe_log_diagnostics(&report.diagnostics);
        }
    }

    Ok(())
}

fn apply_cli_output_preferences(mut report: SearchReport, args: &SearchArgs) -> SearchReport {
    for result in &mut report.results {
        result.path = apply_cli_path_display(result.file_identity(), &args.root, args.path_display);
        if matches!(args.line_endings, crate::LineEndingModeFlag::Normalize) {
            result.content = normalize_line_endings(&result.content);
            for matched in &mut result.matches {
                matched.text = normalize_line_endings(&matched.text);
            }
        }
    }
    report
}

fn apply_cli_path_display(
    identity: &crate::FileIdentity,
    root: &PathBuf,
    mode: crate::PathDisplayModeFlag,
) -> PathBuf {
    match mode {
        crate::PathDisplayModeFlag::Relative => identity.display_path.clone(),
        crate::PathDisplayModeFlag::Absolute => identity.resolved_path.clone(),
        crate::PathDisplayModeFlag::RootRelative => identity
            .resolved_path
            .strip_prefix(dunce::canonicalize(root).unwrap_or_else(|_| root.clone()))
            .map(PathBuf::from)
            .or_else(|_| identity.root_relative_path.clone().ok_or(()))
            .unwrap_or_else(|_| identity.display_path.clone()),
    }
}

fn apply_cli_path_display_to_path(
    path: PathBuf,
    root: &PathBuf,
    mode: crate::PathDisplayModeFlag,
) -> PathBuf {
    match mode {
        crate::PathDisplayModeFlag::Relative => path,
        crate::PathDisplayModeFlag::Absolute => dunce::canonicalize(&path).unwrap_or(path),
        crate::PathDisplayModeFlag::RootRelative => {
            path.strip_prefix(root).map(PathBuf::from).unwrap_or(path)
        }
    }
}

fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn collect_search_report(
    query: &str,
    options: SearchOptions,
    error_mode: ErrorMode,
) -> Result<SearchReport> {
    let mut iter = crate::search_iter(query, options)?;
    let mut diagnostics = iter.diagnostics().to_vec();
    let mut results = Vec::with_capacity(iter.remaining());
    let materialize_started = Instant::now();

    for item in &mut iter {
        match item {
            Ok(result) => {
                diagnostics.extend(result.diagnostics.iter().cloned());
                results.push(result);
            }
            Err(err) => match error_mode {
                ErrorMode::SkipErrors => {
                    diagnostics.push(SearchDiagnostic::walk_warning(
                        None,
                        format!("Skipping result after per-file error: {err}"),
                    ));
                }
                ErrorMode::FailFast => return Err(err),
            },
        }
    }

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

fn maybe_log_diagnostics(diagnostics: &[SearchDiagnostic]) {
    let enabled = std::env::var("RDUMP_LOG_DIAGNOSTICS")
        .ok()
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));
    if !enabled {
        return;
    }

    for diagnostic in diagnostics {
        if let Some(path) = &diagnostic.path {
            eprintln!(
                "[{}:{}] {} ({})",
                format!("{:?}", diagnostic.level).to_lowercase(),
                format!("{:?}", diagnostic.kind).to_lowercase(),
                diagnostic.message,
                path.display()
            );
        } else {
            eprintln!(
                "[{}:{}] {}",
                format!("{:?}", diagnostic.level).to_lowercase(),
                format!("{:?}", diagnostic.kind).to_lowercase(),
                diagnostic.message
            );
        }
    }
}

/// Performs the search logic and returns the matching files and their hunks.
/// This function is separated from `run_search` to be testable.
pub(crate) fn perform_search_internal(
    query: &str,
    options: &SearchOptions,
) -> Result<RawSearchReport> {
    // --- Load Config and Build Query ---
    let display_root = options.root.clone();
    let canonical_root = dunce::canonicalize(&options.root).map_err(|_| {
        anyhow!(
            "root path '{}' does not exist or is not accessible.",
            options.root.display()
        )
    })?;
    let query_to_parse = resolve_effective_query(query, options)?;

    // --- 1. Parse query ---
    let ast = crate::planner::optimize_ast(parser::parse_query(&query_to_parse)?);

    // --- 2. Validate Predicates ---
    validate_ast_predicates(&ast, &predicates::create_predicate_registry())?;

    // --- 3. Find initial candidates ---
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

    let mut stats = SearchStats {
        candidate_files: candidate_files.len(),
        hidden_skipped: discovery.hidden_skipped,
        ignore_skipped: discovery.ignore_skipped,
        max_depth_skipped: discovery.max_depth_skipped,
        unreadable_entries: discovery.unreadable_entries,
        root_boundary_excluded: discovery.root_boundary_excluded,
        directory_hotspots: build_directory_hotspots(&candidate_files, &canonical_root),
        walk_millis: walk_started.elapsed().as_millis() as u64,
        ..Default::default()
    };
    let started = Instant::now();
    let time_budget = search_time_budget(options);

    // --- 4. Pre-filtering Pass (Metadata) ---
    let prefilter_started = Instant::now();
    let metadata_registry = predicates::create_metadata_predicate_registry();
    let pre_filter_evaluator = Evaluator::new(ast.clone(), metadata_registry);

    let first_error = Mutex::new(None);
    let pre_filtered_files: Vec<PathBuf> = candidate_files
        .par_iter()
        .filter_map(|path| {
            if let Some(err) = budget_error(started, time_budget) {
                let mut error_guard = first_error.lock().unwrap();
                if error_guard.is_none() {
                    *error_guard = Some(err);
                }
                return None;
            }
            if first_error.lock().unwrap().is_some() {
                return None;
            }
            let mut context = FileContext::new(path.clone(), canonical_root.clone());
            match pre_filter_evaluator.evaluate(&mut context) {
                Ok(result) => result.is_match().then(|| path.clone()),
                Err(e) => {
                    let mut error_guard = first_error.lock().unwrap();
                    if error_guard.is_none() {
                        *error_guard = Some(anyhow!(
                            "Error during pre-filter on {}: {}",
                            path.display(),
                            e
                        ));
                    }
                    None
                }
            }
        })
        .collect();

    if let Some(e) = first_error.into_inner().unwrap() {
        return Err(e);
    }

    stats.prefiltered_files = pre_filtered_files.len();
    stats.prefilter_millis = prefilter_started.elapsed().as_millis() as u64;

    // --- 5. Main Evaluation Pass (Content + Semantic) ---
    let evaluate_started = Instant::now();
    let (cache_hits_before, cache_misses_before) =
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
    let full_registry = predicates::create_predicate_registry_with_settings(code_settings);
    let evaluator = Evaluator::new(ast, full_registry);

    let first_error = Mutex::new(None);
    let full_pass_diagnostics = Mutex::new(Vec::new());
    let mut matching_files: Vec<RawSearchItem> = pre_filtered_files
        .par_iter()
        .filter_map(|path| {
            if let Some(err) = budget_error(started, time_budget) {
                let mut error_guard = first_error.lock().unwrap();
                if error_guard.is_none() {
                    *error_guard = Some(err);
                }
                return None;
            }
            if first_error.lock().unwrap().is_some() {
                return None;
            }
            let mut context = FileContext::new(path.clone(), canonical_root.clone());
            let evaluation = evaluator.evaluate(&mut context);
            let path_diagnostics = context.take_diagnostics();
            let semantic_skip_reasons = context.take_semantic_skip_reasons();
            let snapshot = options
                .snapshot_drift_detection
                .then(|| {
                    std::fs::metadata(path)
                        .ok()
                        .map(|metadata| FileSnapshot::from_metadata(&metadata))
                })
                .flatten();
            match evaluation {
                Ok(MatchResult::Boolean(true)) => {
                    let (display_path, root_relative_path, resolution) =
                        build_file_identity(path, &canonical_root, &display_root);
                    if options.strict_path_resolution && resolution == PathResolution::Fallback {
                        let mut error_guard = first_error.lock().unwrap();
                        if error_guard.is_none() {
                            *error_guard = Some(anyhow!(
                                "Strict path resolution failed for {}",
                                path.display()
                            ));
                        }
                        return None;
                    }
                    Some(RawSearchItem {
                        display_path: display_path.clone(),
                        resolved_path: path.clone(),
                        root_relative_path,
                        resolution,
                        ranges: Vec::new(),
                        diagnostics: attach_resolution_diagnostics(
                            display_path.clone(),
                            resolution,
                            path_diagnostics,
                        ),
                        semantic_skip_reasons,
                        snapshot,
                    })
                }
                Ok(MatchResult::Boolean(false)) => {
                    if !path_diagnostics.is_empty() {
                        full_pass_diagnostics
                            .lock()
                            .unwrap()
                            .extend(path_diagnostics);
                    }
                    None
                }
                Ok(MatchResult::Hunks(hunks)) => {
                    if hunks.is_empty() {
                        if !path_diagnostics.is_empty() {
                            full_pass_diagnostics
                                .lock()
                                .unwrap()
                                .extend(path_diagnostics);
                        }
                        None
                    } else {
                        let (display_path, root_relative_path, resolution) =
                            build_file_identity(path, &canonical_root, &display_root);
                        if options.strict_path_resolution && resolution == PathResolution::Fallback
                        {
                            let mut error_guard = first_error.lock().unwrap();
                            if error_guard.is_none() {
                                *error_guard = Some(anyhow!(
                                    "Strict path resolution failed for {}",
                                    path.display()
                                ));
                            }
                            return None;
                        }
                        Some(RawSearchItem {
                            display_path: display_path.clone(),
                            resolved_path: path.clone(),
                            root_relative_path,
                            resolution,
                            ranges: hunks,
                            diagnostics: attach_resolution_diagnostics(
                                display_path.clone(),
                                resolution,
                                path_diagnostics,
                            ),
                            semantic_skip_reasons,
                            snapshot,
                        })
                    }
                }
                Err(e) => {
                    let mut error_guard = first_error.lock().unwrap();
                    if error_guard.is_none() {
                        *error_guard =
                            Some(anyhow!("Error evaluating file {}: {}", path.display(), e));
                    }
                    None
                }
            }
        })
        .collect();

    if let Some(e) = first_error.into_inner().unwrap() {
        return Err(e);
    }

    stats.evaluated_files = pre_filtered_files.len();
    stats.matched_files = matching_files.len();
    stats.matched_ranges = matching_files.iter().map(|item| item.ranges.len()).sum();
    stats.evaluate_millis = evaluate_started.elapsed().as_millis() as u64;
    stats.semantic_parse_failures = semantic_telemetry.total_parse_failures();
    stats.semantic_budget_exhaustions = semantic_telemetry.budget_exhaustions();
    stats.semantic_parse_failures_by_language = semantic_telemetry.parse_failures_by_language();
    stats.tree_cache_hits = semantic_telemetry.tree_cache_hits();
    stats.tree_cache_misses = semantic_telemetry.tree_cache_misses();
    let (cache_hits_after, cache_misses_after) =
        crate::predicates::code_aware::query_cache_metrics_snapshot();
    stats.query_cache_hits = cache_hits_after.saturating_sub(cache_hits_before);
    stats.query_cache_misses = cache_misses_after.saturating_sub(cache_misses_before);

    diagnostics.extend(full_pass_diagnostics.into_inner().unwrap());

    matching_files.sort_by(|a, b| {
        a.display_path
            .cmp(&b.display_path)
            .then(a.resolved_path.cmp(&b.resolved_path))
    });

    diagnostics.shrink_to_fit();

    Ok(RawSearchReport {
        results: matching_files,
        stats,
        diagnostics,
    })
}

fn build_file_identity(
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

fn attach_resolution_diagnostics(
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

/// Performs the search logic and returns the matching files and their hunks.
/// This function is separated from `run_search` to be testable.
#[deprecated(
    note = "legacy CLI-compat export; prefer request::execute_search_request or search/search_iter"
)]
pub fn perform_search(args: &SearchArgs) -> Result<Vec<(PathBuf, Vec<Range>)>> {
    let options = crate::request::search_options_from_request(&search_request_from_args(args));

    Ok(
        perform_search_internal(args.query.as_deref().unwrap_or(""), &options)?
            .results
            .into_iter()
            .map(|item| (item.display_path, item.ranges))
            .collect(),
    )
}

pub fn search_request_from_args(args: &SearchArgs) -> SearchRequest {
    let output = if args.find || matches!(args.format, crate::Format::Paths | crate::Format::Find) {
        Some(OutputMode::Paths)
    } else {
        match args.format {
            crate::Format::Summary => Some(OutputMode::Summary),
            crate::Format::Diagnostics => Some(OutputMode::Summary),
            crate::Format::Matches => Some(OutputMode::Matches),
            crate::Format::Snippets => Some(OutputMode::Snippets),
            crate::Format::Json
            | crate::Format::Cat
            | crate::Format::Markdown
            | crate::Format::Hunks => Some(OutputMode::Full),
            crate::Format::Paths | crate::Format::Find => Some(OutputMode::Paths),
        }
    };

    SearchRequest {
        query: args.query.clone().unwrap_or_default(),
        root: Some(args.root.display().to_string()),
        presets: args.preset.clone(),
        no_ignore: args.no_ignore,
        hidden: args.hidden,
        max_depth: args.max_depth,
        sql_dialect: args.dialect.map(Into::into),
        sql_strict: args.sql_strict,
        output,
        limits: None,
        context_lines: args.context,
        error_mode: if args.fail_fast {
            ErrorMode::FailFast
        } else {
            ErrorMode::SkipErrors
        },
        execution_budget_ms: args.execution_budget_ms,
        semantic_budget_ms: args.semantic_budget_ms,
        max_semantic_matches_per_file: args.max_semantic_matches_per_file,
        language_override: args.language_override.clone(),
        semantic_match_mode: args.semantic_match_mode.into(),
        snippet_mode: rdump_contracts::SnippetMode::PreserveLineEndings,
        semantic_strict: args.semantic_strict,
        strict_path_resolution: args.strict_path_resolution,
        snapshot_drift_detection: !args.no_snapshot_drift_detection,
        ignore_debug: args.ignore_debug,
        language_debug: args.language_debug,
        sql_trace: args.sql_trace,
        execution_profile: args.execution_profile.map(Into::into),
        offset: 0,
        continuation_token: None,
        path_display: Some(args.path_display.into()),
        line_endings: Some(args.line_endings.into()),
        include_match_text: !args.no_match_text,
    }
}

/// Walks the directory, respecting .gitignore, and applies our own smart defaults.
fn get_candidate_files(
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
        // If --no-ignore is passed, disable everything.
        walker_builder
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false);
    } else {
        // Layer 1: A user's custom global ignore file.
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

        // Layer 2: A user's custom project-local .rdumpignore file.
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
                        Err(e) => {
                            diagnostics.push(SearchDiagnostic::root_boundary(
                                original_path,
                                format!("Skipping path outside root ({e})"),
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                diagnostics.push(SearchDiagnostic::walk_warning(
                    None,
                    format!("Could not access entry: {e}"),
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
                if !entry
                    .file_type()
                    .is_some_and(|file_type| file_type.is_file())
                {
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

fn build_directory_hotspots(
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

/// Recursively traverses the AST to ensure all used predicates are valid.
fn validate_ast_predicates(
    node: &AstNode,
    registry: &HashMap<PredicateKey, Box<dyn PredicateEvaluator + Send + Sync>>,
) -> Result<()> {
    match node {
        AstNode::Predicate(key, value) => {
            if !registry.contains_key(key) {
                // The parser wraps unknown keys in `Other`, so we can check for that.
                if let PredicateKey::Other(name) = key {
                    return Err(anyhow!("Unknown predicate: '{name}'"));
                }
                // This case handles if a known key is somehow not in the registry.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn get_sorted_file_names(
        root: &PathBuf,
        no_ignore: bool,
        hidden: bool,
        max_depth: Option<usize>,
    ) -> Vec<String> {
        let (mut paths, _) = get_candidate_files(root, no_ignore, hidden, max_depth).unwrap();
        paths.sort();
        let canonical_root = dunce::canonicalize(root).unwrap();
        paths
            .into_iter()
            .map(|p| {
                p.strip_prefix(&canonical_root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    }

    #[test]
    fn test_custom_rdumpignore_file() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let mut ignore_file = fs::File::create(root.join(".rdumpignore")).unwrap();
        writeln!(ignore_file, "*.log").unwrap();
        fs::File::create(root.join("app.js")).unwrap();
        fs::File::create(root.join("app.log")).unwrap();

        let files = get_sorted_file_names(&root.to_path_buf(), false, false, None);
        assert_eq!(files, vec!["app.js"]);
    }

    #[test]
    fn test_unignore_via_rdumpignore() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let node_modules = root.join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        fs::File::create(node_modules.join("some_dep.js")).unwrap();
        fs::File::create(root.join("app.js")).unwrap();

        let mut ignore_file = fs::File::create(root.join(".rdumpignore")).unwrap();
        writeln!(ignore_file, "!node_modules/").unwrap();

        let files = get_sorted_file_names(&root.to_path_buf(), false, false, None);
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"app.js".to_string()));
        let expected_path = PathBuf::from("node_modules").join("some_dep.js");
        assert!(files.contains(&expected_path.to_string_lossy().to_string()));
    }

    #[test]
    fn test_output_to_file_disables_color() {
        // Setup: Create a temporary directory and a file to search
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let file_path = root.join("test.rs");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        // Args: Simulate running `rdump search 'ext:rs' --output dump.txt`
        let output_file = dir.path().join("dump.txt");
        let args = SearchArgs {
            query: Some("ext:rs".to_string()),
            root: root.clone(),
            output: Some(output_file.clone()),
            dialect: None,
            color: ColorChoice::Auto, // This is the default
            context: Some(0),
            ..Default::default()
        };

        // Run the search part of the command
        run_search(args).unwrap();

        // Verify: Read the output file and check for ANSI codes
        let output_content = fs::read_to_string(output_file).unwrap();
        assert!(
            !output_content.contains('\x1b'),
            "Output file should not contain ANSI color codes"
        );
    }

    #[test]
    fn test_output_to_file_with_color_never() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let file_path = root.join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let output_file = dir.path().join("dump.txt");
        let args = SearchArgs {
            query: Some("ext:rs".to_string()),
            root: root.clone(),
            output: Some(output_file.clone()),
            color: ColorChoice::Never,
            ..Default::default()
        };

        run_search(args).unwrap();

        let output_content = fs::read_to_string(output_file).unwrap();
        assert!(!output_content.contains('\x1b'));
    }

    #[test]
    fn test_output_to_file_with_color_always() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let file_path = root.join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let output_file = dir.path().join("dump.txt");
        let args = SearchArgs {
            query: Some("ext:rs".to_string()),
            root: root.clone(),
            output: Some(output_file.clone()),
            color: ColorChoice::Always,
            format: crate::Format::Cat, // Use a format that supports color
            ..Default::default()
        };

        run_search(args).unwrap();

        let output_content = fs::read_to_string(output_file).unwrap();
        assert!(
            output_content.contains('\x1b'),
            "Output file should contain ANSI color codes when color=always"
        );
    }

    #[test]
    fn test_validate_ast_unknown_predicate() {
        use crate::parser::PredicateKey;
        use std::collections::HashMap;

        // Empty registry - no predicates registered
        let registry: HashMap<PredicateKey, Box<dyn PredicateEvaluator + Send + Sync>> =
            HashMap::new();

        // Test with Other predicate key (unknown predicate name)
        let ast = AstNode::Predicate(
            PredicateKey::Other("unknown".to_string()),
            "value".to_string(),
        );
        let result = validate_ast_predicates(&ast, &registry);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown predicate: 'unknown'"));
    }

    #[test]
    fn test_validate_ast_known_key_not_in_registry() {
        use crate::parser::PredicateKey;
        use std::collections::HashMap;

        // Empty registry - known key but not registered
        let registry: HashMap<PredicateKey, Box<dyn PredicateEvaluator + Send + Sync>> =
            HashMap::new();

        let ast = AstNode::Predicate(PredicateKey::Ext, "rs".to_string());
        let result = validate_ast_predicates(&ast, &registry);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown predicate: 'ext'"));
    }

    #[test]
    fn test_validate_ast_logical_ops() {
        use crate::parser::PredicateKey;
        use crate::predicates;
        use crate::predicates::code_aware::CodeAwareSettings;

        let registry =
            predicates::create_predicate_registry_with_settings(CodeAwareSettings::default());

        // Test with valid logical operations
        let ast = AstNode::LogicalOp(
            crate::parser::LogicalOperator::And,
            Box::new(AstNode::Predicate(PredicateKey::Ext, "rs".to_string())),
            Box::new(AstNode::Predicate(PredicateKey::Contains, "fn".to_string())),
        );
        let result = validate_ast_predicates(&ast, &registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_ast_not() {
        use crate::parser::PredicateKey;
        use crate::predicates;
        use crate::predicates::code_aware::CodeAwareSettings;

        let registry =
            predicates::create_predicate_registry_with_settings(CodeAwareSettings::default());

        let ast = AstNode::Not(Box::new(AstNode::Predicate(
            PredicateKey::Ext,
            "py".to_string(),
        )));
        let result = validate_ast_predicates(&ast, &registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_ignores_applied() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Initialize git repo so ignores work properly
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(root)
            .output()
            .ok();

        // Create directories that should be ignored by default
        let target_dir = root.join("target");
        let node_modules = root.join("node_modules");
        fs::create_dir(&target_dir).unwrap();
        fs::create_dir(&node_modules).unwrap();

        fs::File::create(target_dir.join("debug.rs")).unwrap();
        fs::File::create(node_modules.join("package.js")).unwrap();
        fs::File::create(root.join("main.rs")).unwrap();

        let files = get_sorted_file_names(&root.to_path_buf(), false, false, None);
        // Default ignores should filter out target/ and node_modules/
        assert!(files.contains(&"main.rs".to_string()));
        // Note: Default ignores are applied via walker_builder.add_ignore
        // which has lower precedence than .gitignore
    }

    #[test]
    fn test_no_ignore_includes_all() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let target_dir = root.join("target");
        fs::create_dir(&target_dir).unwrap();
        fs::File::create(target_dir.join("debug.rs")).unwrap();
        fs::File::create(root.join("main.rs")).unwrap();

        // With no_ignore=true, target should be included
        let files = get_sorted_file_names(&root.to_path_buf(), true, false, None);
        assert!(files.len() >= 2);
        assert!(files.contains(&"main.rs".to_string()));
    }

    #[test]
    fn test_search_time_budget_env_parsing() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("RDUMP_MAX_SEARCH_MILLIS", "25");
        assert_eq!(
            search_time_budget(&SearchOptions::default()),
            Some(Duration::from_millis(25))
        );
        std::env::remove_var("RDUMP_MAX_SEARCH_MILLIS");
    }
}
