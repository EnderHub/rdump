use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tree_sitter::Range;

use crate::formatter::shared::{
    display_path_text, format_mode, format_size, format_timestamp, get_contextual_line_ranges,
    print_content_with_style, print_markdown_fenced_content, print_plain_content,
};
use crate::limits::{is_probably_binary, maybe_contains_secret, MAX_FILE_SIZE};
use crate::{Format, TimeFormat};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct FileOutput {
    path: String,
    content: String,
}

pub fn print_output(
    writer: &mut impl Write,
    matching_files: &[(PathBuf, Vec<Range>)],
    format: &Format,
    with_line_numbers: bool,
    no_headers: bool,
    use_color: bool,
    context_lines: usize,
    time_format: TimeFormat,
) -> Result<()> {
    match format {
        Format::Find => print_find_format(writer, matching_files, time_format)?,
        Format::Paths => print_paths_format(writer, matching_files)?,
        Format::Json => print_json_format(writer, matching_files)?,
        Format::Cat => print_cat_format(writer, matching_files, with_line_numbers, use_color)?,
        Format::Markdown => {
            print_markdown_format(writer, matching_files, with_line_numbers, !no_headers)?
        }
        Format::Hunks => print_hunks_format(
            writer,
            matching_files,
            with_line_numbers,
            !no_headers,
            use_color,
            context_lines,
        )?,
        other => anyhow::bail!("raw formatter does not support format {:?}", other),
    }
    Ok(())
}

fn read_file_content(path: &Path) -> Result<Option<String>> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to read metadata for {}", path.display()))?;

    if metadata.len() > MAX_FILE_SIZE {
        eprintln!(
            "Skipping {} (exceeds max file size of {} bytes)",
            path.display(),
            MAX_FILE_SIZE
        );
        return Ok(None);
    }

    let bytes =
        fs::read(path).with_context(|| format!("Failed to read file {}", path.display()))?;

    if is_probably_binary(&bytes) {
        eprintln!("Skipping binary file {}", path.display());
        return Ok(None);
    }

    let content = String::from_utf8_lossy(&bytes).into_owned();

    if maybe_contains_secret(&content) {
        eprintln!(
            "Skipping possible secret-containing file {}",
            path.display()
        );
        return Ok(None);
    }

    Ok(Some(content))
}

fn print_markdown_format(
    writer: &mut impl Write,
    matching_files: &[(PathBuf, Vec<Range>)],
    with_line_numbers: bool,
    with_headers: bool,
) -> Result<()> {
    for (index, (path, _)) in matching_files.iter().enumerate() {
        if with_headers {
            if index > 0 {
                writeln!(writer, "\n---\n")?;
            }
            writeln!(writer, "File: {}", path.display())?;
            writeln!(writer, "---")?;
        }
        let Some(content) = read_file_content(path)? else {
            continue;
        };
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        print_markdown_fenced_content(writer, &content, extension, with_line_numbers, 0)?;
    }
    Ok(())
}

fn print_cat_format(
    writer: &mut impl Write,
    matching_files: &[(PathBuf, Vec<Range>)],
    with_line_numbers: bool,
    use_color: bool,
) -> Result<()> {
    for (path, _) in matching_files {
        let Some(content) = read_file_content(path)? else {
            continue;
        };
        if use_color {
            print_content_with_style(
                writer,
                &content,
                path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                with_line_numbers,
                true,
                0,
            )?;
        } else {
            print_plain_content(writer, &content, with_line_numbers, 0)?;
        }
    }
    Ok(())
}

fn print_json_format(
    writer: &mut impl Write,
    matching_files: &[(PathBuf, Vec<Range>)],
) -> Result<()> {
    let mut outputs = Vec::new();
    for (path, _) in matching_files {
        let Some(content) = read_file_content(path)
            .with_context(|| format!("Failed to read file for final output: {}", path.display()))?
        else {
            continue;
        };
        outputs.push(FileOutput {
            path: display_path_text(path),
            content,
        });
    }
    serde_json::to_writer_pretty(writer, &outputs)?;
    Ok(())
}

fn print_paths_format(
    writer: &mut impl Write,
    matching_files: &[(PathBuf, Vec<Range>)],
) -> Result<()> {
    for (path, _) in matching_files {
        writeln!(writer, "{}", display_path_text(path))?;
    }
    Ok(())
}

fn print_find_format(
    writer: &mut impl Write,
    matching_files: &[(PathBuf, Vec<Range>)],
    time_format: TimeFormat,
) -> Result<()> {
    for (path, _) in matching_files {
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
        let size = metadata.len();
        let modified: DateTime<Local> = DateTime::from(metadata.modified()?);
        let perms = metadata.permissions();
        #[cfg(unix)]
        let mode = perms.mode();
        #[cfg(not(unix))]
        let mode = 0;
        let perms_str = format_mode(mode);
        let size_str = format_size(size);
        let time_str = format_timestamp(modified, time_format);

        writeln!(
            writer,
            "{:<12} {:>8} {} {}",
            perms_str,
            size_str,
            time_str,
            display_path_text(path)
        )?;
    }
    Ok(())
}

fn print_hunks_format(
    writer: &mut impl Write,
    matching_files: &[(PathBuf, Vec<Range>)],
    with_line_numbers: bool,
    with_headers: bool,
    use_color: bool,
    context_lines: usize,
) -> Result<()> {
    for (index, (path, hunks)) in matching_files.iter().enumerate() {
        if with_headers {
            if index > 0 {
                writeln!(writer, "\n---\n")?;
            }
            writeln!(writer, "File: {}", path.display())?;
            writeln!(writer, "---")?;
        }
        let Some(content) = read_file_content(path)? else {
            continue;
        };
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        if hunks.is_empty() {
            print_content_with_style(writer, &content, extension, with_line_numbers, use_color, 0)?;
        } else {
            let lines: Vec<&str> = syntect::util::LinesWithEndings::from(&content).collect();
            let line_ranges = get_contextual_line_ranges(hunks, &lines, context_lines);

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
    }
    Ok(())
}
