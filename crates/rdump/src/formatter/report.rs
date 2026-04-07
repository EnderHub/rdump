use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use rdump_contracts::SearchItem;
use serde::Serialize;
use std::io::Write;
use std::path::PathBuf;
use syntect::util::LinesWithEndings;

use crate::backend::{RealFsSearchBackend, SearchBackend};
use crate::formatter::shared::{
    content_notice, content_state_label, display_path_text, escape_human_text, format_size,
    format_timestamp, get_contextual_line_ranges_from_matches, print_content_with_style,
    print_markdown_fenced_content, snippet_range_for_match,
};
use crate::{Format, SearchDiagnostic, SearchReport, SearchResult, SearchStats, TimeFormat};

#[derive(Serialize)]
struct JsonSearchOutput<'a> {
    schema_version: &'static str,
    schema_reference: &'static str,
    status: rdump_contracts::SearchStatus,
    coordinate_semantics: rdump_contracts::MatchCoordinateSemantics,
    results: &'a [SearchResult],
    stats: &'a SearchStats,
    diagnostics: &'a [SearchDiagnostic],
}

pub fn print_path_output(
    writer: &mut impl Write,
    paths: &[PathBuf],
    format: &Format,
    time_format: TimeFormat,
) -> Result<()> {
    print_path_output_with_backend(&RealFsSearchBackend, writer, paths, format, time_format)
}

pub fn print_path_output_with_backend(
    backend: &dyn SearchBackend,
    writer: &mut impl Write,
    paths: &[PathBuf],
    format: &Format,
    time_format: TimeFormat,
) -> Result<()> {
    match format {
        Format::Find => print_find_paths(backend, writer, paths, time_format)?,
        Format::Paths => print_paths_only(writer, paths)?,
        other => anyhow::bail!("path-only output does not support format {:?}", other),
    }
    Ok(())
}

pub fn print_contract_path_items(
    writer: &mut impl Write,
    items: &[SearchItem],
    format: &Format,
    time_format: TimeFormat,
) -> Result<()> {
    for item in items {
        let SearchItem::Path { path, metadata, .. } = item else {
            continue;
        };

        match format {
            Format::Paths => {
                writeln!(writer, "{path}")?;
            }
            Format::Find => {
                let size_str = format_size(metadata.size_bytes);
                let time_str = metadata
                    .modified_unix_millis
                    .and_then(|millis| {
                        chrono::TimeZone::timestamp_millis_opt(&Local, millis).single()
                    })
                    .map(|modified| format_timestamp(modified, time_format))
                    .unwrap_or_else(|| "-".to_string());
                writeln!(
                    writer,
                    "{:<12} {:>8} {} {}",
                    metadata.permissions_display, size_str, time_str, path
                )?;
            }
            other => anyhow::bail!("path-only output does not support format {:?}", other),
        }
    }

    Ok(())
}

pub fn print_report_output(
    writer: &mut impl Write,
    report: &SearchReport,
    format: &Format,
    with_line_numbers: bool,
    no_headers: bool,
    use_color: bool,
    context_lines: usize,
    show_suppressed_placeholders: bool,
    time_format: TimeFormat,
) -> Result<()> {
    match format {
        Format::Find => print_find_results(writer, &report.results, time_format)?,
        Format::Paths => {
            let paths: Vec<_> = report
                .results
                .iter()
                .map(|result| result.path.clone())
                .collect();
            print_paths_only(writer, &paths)?
        }
        Format::Json => print_json_report(writer, report)?,
        Format::Cat => print_cat_results(
            writer,
            &report.results,
            with_line_numbers,
            use_color,
            show_suppressed_placeholders,
        )?,
        Format::Markdown => print_markdown_results(
            writer,
            &report.results,
            with_line_numbers,
            !no_headers,
            show_suppressed_placeholders,
        )?,
        Format::Hunks => print_hunks_results(
            writer,
            &report.results,
            with_line_numbers,
            !no_headers,
            use_color,
            context_lines,
            show_suppressed_placeholders,
        )?,
        Format::Summary => {
            print_summary_results(writer, &report.results, show_suppressed_placeholders)?
        }
        Format::Diagnostics => print_diagnostics_results(
            writer,
            &report.results,
            !no_headers,
            show_suppressed_placeholders,
        )?,
        Format::Matches => print_matches_results(
            writer,
            &report.results,
            !no_headers,
            show_suppressed_placeholders,
        )?,
        Format::Snippets => print_snippets_results(
            writer,
            &report.results,
            with_line_numbers,
            !no_headers,
            use_color,
            context_lines,
            show_suppressed_placeholders,
        )?,
    }
    print_report_footer(writer, report, format)?;
    Ok(())
}

fn print_markdown_results(
    writer: &mut impl Write,
    results: &[SearchResult],
    with_line_numbers: bool,
    with_headers: bool,
    show_suppressed_placeholders: bool,
) -> Result<()> {
    for (index, result) in results.iter().enumerate() {
        if with_headers {
            if index > 0 {
                writeln!(writer, "\n---\n")?;
            }
            writeln!(writer, "File: {}", display_path_text(&result.path))?;
            writeln!(writer, "---")?;
        }

        if !result.content_available() {
            if !show_suppressed_placeholders {
                continue;
            }
            writeln!(writer, "{}", content_notice(result))?;
            continue;
        }

        let extension = result
            .path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        print_markdown_fenced_content(writer, &result.content, extension, with_line_numbers, 0)?;
    }
    Ok(())
}

fn print_cat_results(
    writer: &mut impl Write,
    results: &[SearchResult],
    with_line_numbers: bool,
    use_color: bool,
    show_suppressed_placeholders: bool,
) -> Result<()> {
    for result in results {
        if !result.content_available() {
            if !show_suppressed_placeholders {
                continue;
            }
            writeln!(writer, "{}", content_notice(result))?;
            continue;
        }

        print_content_with_style(
            writer,
            &result.content,
            result
                .path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or(""),
            with_line_numbers,
            use_color,
            0,
        )?;
    }
    Ok(())
}

fn print_json_report(writer: &mut impl Write, report: &SearchReport) -> Result<()> {
    let output = JsonSearchOutput {
        schema_version: rdump_contracts::SCHEMA_VERSION,
        schema_reference: "rdump://docs/sdk",
        status: report.status(),
        coordinate_semantics: crate::request::coordinate_semantics(),
        results: &report.results,
        stats: &report.stats,
        diagnostics: &report.diagnostics,
    };
    serde_json::to_writer_pretty(writer, &output)?;
    Ok(())
}

fn print_paths_only(writer: &mut impl Write, paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        writeln!(writer, "{}", display_path_text(path))?;
    }
    Ok(())
}

fn print_find_paths(
    backend: &dyn SearchBackend,
    writer: &mut impl Write,
    paths: &[PathBuf],
    time_format: TimeFormat,
) -> Result<()> {
    for path in paths {
        let metadata = backend
            .stat(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
        let size_str = format_size(metadata.size_bytes);
        let time_str = metadata
            .modified_unix_millis
            .and_then(|millis| chrono::TimeZone::timestamp_millis_opt(&Local, millis).single())
            .map(|modified: DateTime<Local>| format_timestamp(modified, time_format))
            .unwrap_or_else(|| "-".to_string());

        writeln!(
            writer,
            "{:<12} {:>8} {} {}",
            metadata.permissions_display,
            size_str,
            time_str,
            display_path_text(path)
        )?;
    }
    Ok(())
}

fn print_find_results(
    writer: &mut impl Write,
    results: &[SearchResult],
    time_format: TimeFormat,
) -> Result<()> {
    for result in results {
        if let Some(snapshot) = result.metadata.snapshot.as_ref() {
            let size_str = format_size(snapshot.len);
            let time_str = snapshot
                .modified_unix_millis
                .and_then(|millis| chrono::TimeZone::timestamp_millis_opt(&Local, millis).single())
                .map(|modified| format_timestamp(modified, time_format))
                .unwrap_or_else(|| "-".to_string());

            writeln!(
                writer,
                "{:<12} {:>8} {} {}",
                snapshot.permissions_display,
                size_str,
                time_str,
                display_path_text(&result.path)
            )?;
            continue;
        }

        let path = std::slice::from_ref(&result.path);
        print_find_paths(&RealFsSearchBackend, writer, path, time_format)?;
    }
    Ok(())
}

fn print_hunks_results(
    writer: &mut impl Write,
    results: &[SearchResult],
    with_line_numbers: bool,
    with_headers: bool,
    use_color: bool,
    context_lines: usize,
    show_suppressed_placeholders: bool,
) -> Result<()> {
    for (index, result) in results.iter().enumerate() {
        if with_headers {
            if index > 0 {
                writeln!(writer, "\n---\n")?;
            }
            writeln!(writer, "File: {}", display_path_text(&result.path))?;
            writeln!(writer, "---")?;
        }

        if !result.content_available() {
            if !show_suppressed_placeholders {
                continue;
            }
            writeln!(writer, "{}", content_notice(result))?;
            continue;
        }

        let extension = result
            .path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if result.matches.is_empty() {
            print_content_with_style(
                writer,
                &result.content,
                extension,
                with_line_numbers,
                use_color,
                0,
            )?;
            continue;
        }

        let lines: Vec<&str> = LinesWithEndings::from(&result.content).collect();
        let line_ranges =
            get_contextual_line_ranges_from_matches(&result.matches, &lines, context_lines);

        for (range_index, range) in line_ranges.iter().enumerate() {
            if range_index > 0 {
                writeln!(writer, "...")?;
            }
            let hunk_content = lines[range.clone()].join("");
            print_content_with_style(
                writer,
                &hunk_content,
                extension,
                with_line_numbers,
                use_color,
                range.start,
            )?;
        }
    }
    Ok(())
}

fn print_summary_results(
    writer: &mut impl Write,
    results: &[SearchResult],
    show_suppressed_placeholders: bool,
) -> Result<()> {
    for result in results {
        if !show_suppressed_placeholders && !result.content_available() {
            continue;
        }
        writeln!(
            writer,
            "{}\tmatches={}\twhole_file_match={}\tcontent_state={}\tdiagnostics={}",
            display_path_text(&result.path),
            result.match_count(),
            result.is_whole_file_match(),
            content_state_label(&result.content_state),
            result.diagnostics.len()
        )?;
    }
    Ok(())
}

fn print_matches_results(
    writer: &mut impl Write,
    results: &[SearchResult],
    with_headers: bool,
    show_suppressed_placeholders: bool,
) -> Result<()> {
    for (index, result) in results.iter().enumerate() {
        if with_headers {
            if index > 0 {
                writeln!(writer, "\n---\n")?;
            }
            writeln!(writer, "File: {}", display_path_text(&result.path))?;
            writeln!(writer, "---")?;
            writeln!(
                writer,
                "[partial output: match coordinates plus escaped text]"
            )?;
        }

        if result.matches.is_empty() {
            if !show_suppressed_placeholders && !result.content_available() {
                continue;
            }
            if with_headers {
                writeln!(
                    writer,
                    "[whole-file match; content_state={}]",
                    content_state_label(&result.content_state)
                )?;
            } else {
                writeln!(
                    writer,
                    "{}\twhole-file\tcontent_state={}",
                    display_path_text(&result.path),
                    content_state_label(&result.content_state)
                )?;
            }
            continue;
        }

        for matched in &result.matches {
            let text = escape_human_text(&matched.text);
            if with_headers {
                writeln!(
                    writer,
                    "{}:{}-{}:{} {}",
                    matched.start_line,
                    matched.start_column + 1,
                    matched.end_line,
                    matched.end_column + 1,
                    text
                )?;
            } else {
                writeln!(
                    writer,
                    "{}:{}:{}-{}:{} {}",
                    display_path_text(&result.path),
                    matched.start_line,
                    matched.start_column + 1,
                    matched.end_line,
                    matched.end_column + 1,
                    text
                )?;
            }
        }
    }
    Ok(())
}

fn print_diagnostics_results(
    writer: &mut impl Write,
    results: &[SearchResult],
    with_headers: bool,
    show_suppressed_placeholders: bool,
) -> Result<()> {
    for (index, result) in results.iter().enumerate() {
        if !show_suppressed_placeholders && !result.content_available() {
            continue;
        }
        if with_headers {
            if index > 0 {
                writeln!(writer, "\n---\n")?;
            }
            writeln!(writer, "File: {}", display_path_text(&result.path))?;
            writeln!(writer, "---")?;
        }
        writeln!(
            writer,
            "matches={}\twhole_file_match={}\tcontent_state={}",
            result.match_count(),
            result.is_whole_file_match(),
            content_state_label(&result.content_state)
        )?;
        if result.diagnostics.is_empty() {
            writeln!(writer, "diagnostics=0")?;
            continue;
        }
        writeln!(writer, "diagnostics={}", result.diagnostics.len())?;
        for diagnostic in &result.diagnostics {
            writeln!(
                writer,
                "- [{}:{}] {}",
                format!("{:?}", diagnostic.level).to_lowercase(),
                format!("{:?}", diagnostic.kind).to_lowercase(),
                escape_human_text(&diagnostic.message)
            )?;
        }
    }
    Ok(())
}

fn print_snippets_results(
    writer: &mut impl Write,
    results: &[SearchResult],
    with_line_numbers: bool,
    with_headers: bool,
    use_color: bool,
    context_lines: usize,
    show_suppressed_placeholders: bool,
) -> Result<()> {
    for (index, result) in results.iter().enumerate() {
        if with_headers {
            if index > 0 {
                writeln!(writer, "\n---\n")?;
            }
            writeln!(writer, "File: {}", display_path_text(&result.path))?;
            writeln!(writer, "---")?;
            writeln!(writer, "[partial output: contextual snippets only]")?;
        }

        if !result.content_available() {
            if !show_suppressed_placeholders {
                continue;
            }
            writeln!(writer, "{}", content_notice(result))?;
            continue;
        }

        let extension = result
            .path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if result.matches.is_empty() {
            print_content_with_style(
                writer,
                &result.content,
                extension,
                with_line_numbers,
                use_color,
                0,
            )?;
            continue;
        }

        let lines: Vec<&str> = LinesWithEndings::from(&result.content).collect();
        for (match_index, matched) in result.matches.iter().enumerate() {
            if match_index > 0 {
                writeln!(writer, "...")?;
            }

            let range = snippet_range_for_match(matched, lines.len(), context_lines);
            writeln!(writer, "@@ {}-{} @@", range.start + 1, range.end)?;
            let snippet_content = lines[range.clone()].join("");
            print_content_with_style(
                writer,
                &snippet_content,
                extension,
                with_line_numbers,
                use_color,
                range.start,
            )?;
        }
    }
    Ok(())
}

fn print_report_footer(
    writer: &mut impl Write,
    report: &SearchReport,
    format: &Format,
) -> Result<()> {
    if report.results.is_empty() {
        return Ok(());
    }
    let mut segments = Vec::new();
    let suppressed_total = report.stats.suppressed_too_large
        + report.stats.suppressed_binary
        + report.stats.suppressed_secret_like;
    if suppressed_total > 0 {
        segments.push(format!(
            "suppressed too_large={} binary={} secret_like={}",
            report.stats.suppressed_too_large,
            report.stats.suppressed_binary,
            report.stats.suppressed_secret_like
        ));
    }
    if report.stats.hidden_skipped > 0
        || report.stats.ignore_skipped > 0
        || report.stats.max_depth_skipped > 0
        || report.stats.root_boundary_excluded > 0
    {
        segments.push(format!(
            "discovery hidden_skipped={} ignore_skipped={} max_depth_skipped={} root_boundary_excluded={}",
            report.stats.hidden_skipped,
            report.stats.ignore_skipped,
            report.stats.max_depth_skipped,
            report.stats.root_boundary_excluded
        ));
    }
    match format {
        Format::Matches => segments.push("output is partial: match-coordinate view".to_string()),
        Format::Snippets => segments.push("output is partial: contextual snippet view".to_string()),
        Format::Hunks => segments.push("output is partial: matching-hunks view".to_string()),
        _ => {}
    }
    if segments.is_empty() {
        return Ok(());
    }
    writeln!(writer, "\n--")?;
    for segment in segments {
        writeln!(writer, "{segment}")?;
    }
    Ok(())
}
