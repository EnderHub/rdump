use crate::{ColorChoice, SearchArgs, SearchOptions, SearchReport, SearchRuntime, SearchStats};
use anyhow::Result;
use rdump_contracts::{ErrorMode, LimitValue, Limits, OutputMode, SearchRequest};
use std::fs::File;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use tree_sitter::Range;

use crate::formatter;

/// The main entry point for the `search` command.
pub fn run_search(mut args: SearchArgs) -> Result<()> {
    if args.no_headers && args.find {
        eprintln!("Warning: --no-headers has no effect with --find.");
    }
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

    let request = search_request_from_args(&args);
    let options = crate::request::search_options_from_request(&request);
    let query = args.query.as_deref().unwrap_or("");

    let use_color = if args.output.is_some() {
        args.color == ColorChoice::Always
    } else {
        match args.color {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => io::stdout().is_terminal(),
        }
    };

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
            let response = crate::request::execute_search_request(&request)?;
            formatter::print_contract_path_items(
                &mut writer,
                &response.results,
                &args.format,
                args.time_format,
            )?;
            maybe_log_contract_diagnostics(&response.diagnostics);
        }
        crate::Format::Summary => {
            let report =
                apply_cli_output_preferences(crate::search_with_stats(query, options)?, &args);
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
            let report =
                apply_cli_output_preferences(crate::search_with_stats(query, options)?, &args);
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
        result.path = apply_cli_path_display(result.file_identity(), args.path_display);
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
    mode: crate::PathDisplayModeFlag,
) -> PathBuf {
    match mode {
        crate::PathDisplayModeFlag::Relative => identity.display_path.clone(),
        crate::PathDisplayModeFlag::Absolute => identity.resolved_path.clone(),
        crate::PathDisplayModeFlag::RootRelative => identity
            .root_relative_path
            .clone()
            .unwrap_or_else(|| identity.display_path.clone()),
    }
}

fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn maybe_log_diagnostics(diagnostics: &[crate::SearchDiagnostic]) {
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

fn maybe_log_contract_diagnostics(diagnostics: &[rdump_contracts::SearchDiagnostic]) {
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
                diagnostic.level, diagnostic.kind, diagnostic.message, path
            );
        } else {
            eprintln!(
                "[{}:{}] {}",
                diagnostic.level, diagnostic.kind, diagnostic.message
            );
        }
    }
}

/// Performs the search logic and returns the matching files and their hunks.
/// This function is separated from `run_search` to be testable.
pub(crate) fn perform_search_internal(
    query: &str,
    options: &SearchOptions,
) -> Result<crate::engine::RawSearchReport> {
    SearchRuntime::real_fs().collect_raw_search(query, options, None)
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
        limits: Some(cli_unbounded_limits()),
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

fn cli_unbounded_limits() -> Limits {
    Limits {
        max_results: LimitValue::Unlimited,
        max_matches_per_file: LimitValue::Unlimited,
        max_bytes_per_file: LimitValue::Unlimited,
        max_total_bytes: LimitValue::Unlimited,
        max_match_bytes: LimitValue::Unlimited,
        max_snippet_bytes: LimitValue::Unlimited,
        max_errors: LimitValue::Unlimited,
    }
}
