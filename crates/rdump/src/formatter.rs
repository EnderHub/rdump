#[path = "formatter/raw.rs"]
mod raw;
#[path = "formatter/report.rs"]
mod report;
#[path = "formatter/shared.rs"]
mod shared;

pub use raw::print_output;
pub use report::{print_path_output, print_report_output};
pub(crate) use shared::format_mode;

#[cfg(test)]
pub(crate) use shared::{format_size, get_contextual_line_ranges};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ContentState, Match, SearchReport, SearchResult, SearchResultMetadata, SearchStats,
    };
    use std::io::Write;
    use std::ops::Range;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn create_temp_file_with_content(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    fn sample_report(format_path: &str, content: &str, matches: Vec<Match>) -> SearchReport {
        SearchReport {
            results: vec![SearchResult {
                path: PathBuf::from(format_path),
                matches,
                content: content.to_string(),
                content_state: ContentState::Loaded,
                diagnostics: vec![],
                metadata: SearchResultMetadata::default(),
            }],
            stats: SearchStats::default(),
            diagnostics: vec![],
        }
    }

    #[test]
    fn test_format_plain_cat_with_line_numbers() {
        let file = create_temp_file_with_content("a\nb");
        let paths = vec![(file.path().to_path_buf(), vec![])];
        let mut writer = Vec::new();
        print_output(
            &mut writer,
            &paths,
            &crate::Format::Cat,
            true,
            false,
            false,
            0,
            crate::TimeFormat::Local,
        )
        .unwrap();
        let output = String::from_utf8(writer).unwrap();
        assert_eq!(output, "    1 | a\n    2 | b");
    }

    #[test]
    fn test_format_paths() {
        let file1 = create_temp_file_with_content("a");
        let file2 = create_temp_file_with_content("b");
        let paths = vec![
            (file1.path().to_path_buf(), vec![]),
            (file2.path().to_path_buf(), vec![]),
        ];
        let mut writer = Vec::new();
        print_output(
            &mut writer,
            &paths,
            &crate::Format::Paths,
            false,
            false,
            false,
            0,
            crate::TimeFormat::Local,
        )
        .unwrap();
        let output = String::from_utf8(writer).unwrap();
        let expected = format!("{}\n{}\n", file1.path().display(), file2.path().display());
        assert_eq!(output, expected);
    }

    #[test]
    fn test_format_markdown_with_fences() {
        let file = create_temp_file_with_content("line 1");
        let paths = vec![(file.path().to_path_buf(), vec![])];
        let mut writer = Vec::new();
        print_output(
            &mut writer,
            &paths,
            &crate::Format::Markdown,
            false,
            false,
            false,
            0,
            crate::TimeFormat::Local,
        )
        .unwrap();

        let output = String::from_utf8(writer).unwrap();
        let expected_header = format!("File: {}\n---\n", file.path().display());
        assert!(output.starts_with(&expected_header));
        assert!(output.contains("```\nline 1\n```\n"));
    }

    #[test]
    fn test_format_markdown_with_ansi_color() {
        let file = create_temp_file_with_content("fn main() {}");
        let rs_path = file.path().with_extension("rs");
        std::fs::rename(file.path(), &rs_path).unwrap();

        let paths = vec![(rs_path, vec![])];
        let mut writer = Vec::new();
        print_output(
            &mut writer,
            &paths,
            &crate::Format::Cat,
            false,
            false,
            true,
            0,
            crate::TimeFormat::Local,
        )
        .unwrap();
        let output = String::from_utf8(writer).unwrap();

        assert!(output.contains("\x1b["), "Should contain ANSI escape codes");
        assert!(
            !output.contains("```"),
            "Should not contain markdown fences"
        );
    }

    #[test]
    fn test_format_markdown_ignores_color_flag() {
        let file = create_temp_file_with_content("fn main() {}");
        let paths = vec![(file.path().to_path_buf(), vec![])];
        let mut writer = Vec::new();
        print_output(
            &mut writer,
            &paths,
            &crate::Format::Markdown,
            false,
            false,
            true,
            0,
            crate::TimeFormat::Local,
        )
        .unwrap();

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("```"));
        assert!(!output.contains("\x1b["));
    }

    #[test]
    fn test_format_find() {
        let file = create_temp_file_with_content("hello");
        let paths = vec![(file.path().to_path_buf(), vec![])];
        let mut writer = Vec::new();
        print_output(
            &mut writer,
            &paths,
            &crate::Format::Find,
            false,
            false,
            false,
            0,
            crate::TimeFormat::Local,
        )
        .unwrap();
        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("B"));
        assert!(output.contains(&file.path().display().to_string()));
    }

    #[test]
    fn test_format_size_all_ranges() {
        assert_eq!(format_size(500), "500B");
        assert_eq!(format_size(1024), "1.0K");
        assert_eq!(format_size(1536), "1.5K");
        assert_eq!(format_size(1024 * 1024), "1.0M");
        assert_eq!(format_size(2 * 1024 * 1024), "2.0M");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0G");
        assert_eq!(format_size(3 * 1024 * 1024 * 1024), "3.0G");
    }

    #[test]
    fn test_format_markdown_multiple_files_with_separators() {
        let file1 = create_temp_file_with_content("content 1");
        let file2 = create_temp_file_with_content("content 2");
        let paths = vec![
            (file1.path().to_path_buf(), vec![]),
            (file2.path().to_path_buf(), vec![]),
        ];
        let mut writer = Vec::new();
        print_output(
            &mut writer,
            &paths,
            &crate::Format::Markdown,
            false,
            false,
            false,
            0,
            crate::TimeFormat::Local,
        )
        .unwrap();
        let output = String::from_utf8(writer).unwrap();

        assert!(output.contains("content 1"));
        assert!(output.contains("content 2"));
        assert!(output.contains("File:"));
    }

    #[test]
    fn test_format_cat_with_ansi_and_line_numbers() {
        let file = create_temp_file_with_content("fn main() {}\nlet x = 1;");
        let rs_path = file.path().with_extension("rs");
        std::fs::rename(file.path(), &rs_path).unwrap();

        let paths = vec![(rs_path, vec![])];
        let mut writer = Vec::new();
        print_output(
            &mut writer,
            &paths,
            &crate::Format::Cat,
            true,
            false,
            true,
            0,
            crate::TimeFormat::Local,
        )
        .unwrap();
        let output = String::from_utf8(writer).unwrap();

        assert!(output.contains(" | "));
        assert!(output.contains("\x1b["));
    }

    #[test]
    fn test_get_contextual_line_ranges_empty_hunks() {
        let lines: Vec<&str> = vec!["line 1", "line 2", "line 3"];
        let hunks: Vec<tree_sitter::Range> = vec![];
        let result = get_contextual_line_ranges(&hunks, &lines, 1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_contextual_line_ranges_empty_lines() {
        let lines: Vec<&str> = vec![];
        let hunks = vec![tree_sitter::Range {
            start_byte: 0,
            end_byte: 10,
            start_point: tree_sitter::Point { row: 0, column: 0 },
            end_point: tree_sitter::Point { row: 0, column: 10 },
        }];
        let result = get_contextual_line_ranges(&hunks, &lines, 1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_summary_report_output() {
        let report = sample_report("src/main.rs", "fn main() {}", vec![]);
        let mut writer = Vec::new();
        print_report_output(
            &mut writer,
            &report,
            &crate::Format::Summary,
            false,
            false,
            false,
            0,
            true,
            crate::TimeFormat::Local,
        )
        .unwrap();
        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("src/main.rs"));
        assert!(output.contains("whole_file_match=true"));
    }

    #[test]
    fn test_matches_report_output() {
        let report = sample_report(
            "src/main.rs",
            "fn main() {}",
            vec![Match {
                start_line: 1,
                end_line: 1,
                start_column: 3,
                end_column: 7,
                byte_range: Range { start: 3, end: 7 },
                text: "main".to_string(),
            }],
        );
        let mut writer = Vec::new();
        print_report_output(
            &mut writer,
            &report,
            &crate::Format::Matches,
            false,
            false,
            false,
            0,
            true,
            crate::TimeFormat::Local,
        )
        .unwrap();
        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("File: src/main.rs"));
        assert!(output.contains("1:4-1:8 main"));
    }

    #[test]
    fn test_snippets_report_output() {
        let report = sample_report(
            "src/main.rs",
            "fn main() {}\nlet value = main();",
            vec![Match {
                start_line: 2,
                end_line: 2,
                start_column: 12,
                end_column: 16,
                byte_range: Range { start: 25, end: 29 },
                text: "main".to_string(),
            }],
        );
        let mut writer = Vec::new();
        print_report_output(
            &mut writer,
            &report,
            &crate::Format::Snippets,
            true,
            false,
            false,
            0,
            true,
            crate::TimeFormat::Local,
        )
        .unwrap();
        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("@@ 2-2 @@"));
        assert!(output.contains("2 | let value = main();"));
    }
}
