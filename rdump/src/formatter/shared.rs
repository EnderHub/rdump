use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use once_cell::sync::Lazy;
use std::io::Write;
use std::ops::Range as StdRange;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
use tree_sitter::Range;

use crate::{ContentState, SearchResult, TimeFormat};

pub(crate) static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
pub(crate) static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

pub(crate) fn print_content_with_style(
    writer: &mut impl Write,
    content: &str,
    extension: &str,
    with_line_numbers: bool,
    use_color: bool,
    start_line_number: usize,
) -> Result<()> {
    if use_color {
        print_highlighted_content(
            writer,
            content,
            extension,
            with_line_numbers,
            start_line_number,
        )
    } else {
        print_plain_content(writer, content, with_line_numbers, start_line_number)
    }
}

pub(crate) fn content_notice(result: &SearchResult) -> String {
    match &result.content_state {
        ContentState::Loaded => String::new(),
        ContentState::LoadedLossy => "[content decoded with lossy UTF-8 replacement]".to_string(),
        ContentState::Skipped { reason } => format!("[content unavailable: {}]", reason.as_str()),
    }
}

pub(crate) fn escape_human_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{{{:x}}}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

pub(crate) fn display_path_text(path: &Path) -> String {
    escape_human_text(&path.display().to_string())
}

pub(crate) fn content_state_label(state: &ContentState) -> String {
    match state {
        ContentState::Loaded => "loaded".to_string(),
        ContentState::LoadedLossy => "loaded_lossy".to_string(),
        ContentState::Skipped { reason } => format!("skipped:{}", reason.as_str()),
    }
}

pub(crate) fn get_contextual_line_ranges(
    hunks: &[Range],
    lines: &[&str],
    context_lines: usize,
) -> Vec<StdRange<usize>> {
    if hunks.is_empty() || lines.is_empty() {
        return vec![];
    }

    let mut line_ranges = Vec::new();
    for hunk in hunks {
        let start_line = hunk.start_point.row;
        let end_line = hunk.end_point.row;

        let context_start = start_line.saturating_sub(context_lines);
        let context_end = (end_line + context_lines).min(lines.len() - 1);

        if context_end >= context_start {
            line_ranges.push(context_start..context_end + 1);
        }
    }

    merge_line_ranges(line_ranges)
}

pub(crate) fn get_contextual_line_ranges_from_matches(
    matches: &[crate::Match],
    lines: &[&str],
    context_lines: usize,
) -> Vec<StdRange<usize>> {
    if matches.is_empty() || lines.is_empty() {
        return vec![];
    }

    let mut line_ranges = Vec::new();
    for matched in matches {
        let start_line = matched.start_line.saturating_sub(1);
        let end_line = matched.end_line.saturating_sub(1);

        let context_start = start_line.saturating_sub(context_lines);
        let context_end = (end_line + context_lines).min(lines.len() - 1);

        if context_end >= context_start {
            line_ranges.push(context_start..context_end + 1);
        }
    }

    merge_line_ranges(line_ranges)
}

pub(crate) fn snippet_range_for_match(
    matched: &crate::Match,
    total_lines: usize,
    context_lines: usize,
) -> StdRange<usize> {
    let start_line = matched.start_line.saturating_sub(1);
    let end_line = matched.end_line.saturating_sub(1);

    let context_start = start_line.saturating_sub(context_lines);
    let context_end = (end_line + context_lines).min(total_lines.saturating_sub(1));

    context_start..context_end + 1
}

pub(crate) fn print_highlighted_content(
    writer: &mut impl Write,
    content: &str,
    extension: &str,
    with_line_numbers: bool,
    start_line_number: usize,
) -> Result<()> {
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let theme = &THEME_SET.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);

    for (i, line) in LinesWithEndings::from(content).enumerate() {
        if with_line_numbers {
            write!(writer, "{: >5} | ", start_line_number + i + 1)?;
        }
        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &SYNTAX_SET)?;
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        write!(writer, "{escaped}")?;
    }
    write!(writer, "\x1b[0m")?;
    Ok(())
}

pub(crate) fn print_plain_content(
    writer: &mut impl Write,
    content: &str,
    with_line_numbers: bool,
    start_line_number: usize,
) -> Result<()> {
    for (i, line) in LinesWithEndings::from(content).enumerate() {
        if with_line_numbers {
            write!(writer, "{: >5} | {}", start_line_number + i + 1, line)?;
        } else {
            write!(writer, "{line}")?;
        }
    }
    Ok(())
}

pub(crate) fn print_markdown_fenced_content(
    writer: &mut impl Write,
    content: &str,
    extension: &str,
    with_line_numbers: bool,
    start_line_number: usize,
) -> Result<()> {
    writeln!(writer, "```{extension}")?;
    print_plain_content(writer, content, with_line_numbers, start_line_number)?;
    if !content.is_empty() && !content.ends_with('\n') && !content.ends_with('\r') {
        writeln!(writer)?;
    }
    writeln!(writer, "```")?;
    Ok(())
}

pub(crate) fn format_mode(mode: u32) -> String {
    #[cfg(unix)]
    {
        let user_r = if mode & 0o400 != 0 { 'r' } else { '-' };
        let user_w = if mode & 0o200 != 0 { 'w' } else { '-' };
        let user_x = if mode & 0o100 != 0 { 'x' } else { '-' };
        let group_r = if mode & 0o040 != 0 { 'r' } else { '-' };
        let group_w = if mode & 0o020 != 0 { 'w' } else { '-' };
        let group_x = if mode & 0o010 != 0 { 'x' } else { '-' };
        let other_r = if mode & 0o004 != 0 { 'r' } else { '-' };
        let other_w = if mode & 0o002 != 0 { 'w' } else { '-' };
        let other_x = if mode & 0o001 != 0 { 'x' } else { '-' };
        format!("-{user_r}{user_w}{user_x}{group_r}{group_w}{group_x}{other_r}{other_w}{other_x}")
    }
    #[cfg(not(unix))]
    {
        if mode & 0o200 != 0 {
            "-rw-------"
        } else {
            "-r--------"
        }
        .to_string()
    }
}

pub(crate) fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{bytes}B")
    }
}

pub(crate) fn format_timestamp(timestamp: DateTime<Local>, format: TimeFormat) -> String {
    match format {
        TimeFormat::Local => timestamp.format("%b %d %H:%M").to_string(),
        TimeFormat::Utc => timestamp
            .with_timezone(&Utc)
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        TimeFormat::Iso => timestamp.to_rfc3339(),
        TimeFormat::Unix => timestamp.timestamp().to_string(),
    }
}

fn merge_line_ranges(mut line_ranges: Vec<StdRange<usize>>) -> Vec<StdRange<usize>> {
    line_ranges.sort_by_key(|range| range.start);

    let mut merged_ranges = Vec::new();
    let mut iter = line_ranges.into_iter();
    if let Some(mut current) = iter.next() {
        for next in iter {
            if next.start <= current.end {
                current.end = current.end.max(next.end);
            } else {
                merged_ranges.push(current);
                current = next;
            }
        }
        merged_ranges.push(current);
    }

    merged_ranges
}
