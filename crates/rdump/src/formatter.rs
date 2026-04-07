#[path = "formatter/raw.rs"]
mod raw;
#[path = "formatter/report.rs"]
mod report;
#[path = "formatter/shared.rs"]
mod shared;

pub use raw::{print_output, print_output_with_backend};
pub use report::{
    print_contract_path_items, print_path_output, print_path_output_with_backend,
    print_report_output,
};
pub(crate) use shared::format_mode;

#[cfg(test)]
pub(crate) use shared::get_contextual_line_ranges;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{
        BackendFileType, BackendMetadata, BackendPathIdentity, DiscoveryReport, DiscoveryRequest,
        SearchBackend,
    };
    use crate::formatter::shared::format_size;
    use crate::{
        ContentState, Match, SearchReport, SearchResult, SearchResultMetadata, SearchStats,
    };
    use anyhow::{anyhow, Result};
    use std::collections::BTreeMap;
    use std::io::Write;
    use std::ops::Range;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    #[derive(Debug)]
    struct FakeBackend {
        root: PathBuf,
        files: BTreeMap<PathBuf, BackendMetadata>,
    }

    impl FakeBackend {
        fn new(root: PathBuf, files: impl IntoIterator<Item = PathBuf>) -> Self {
            let files = files
                .into_iter()
                .map(|relative| {
                    (
                        relative.clone(),
                        BackendMetadata {
                            size_bytes: 12,
                            modified_unix_millis: Some(1_700_000_000_000),
                            readonly: false,
                            permissions_display: "-rw-r--r--".to_string(),
                            file_type: BackendFileType::File,
                            stable_token: Some(format!("token:{}", relative.display())),
                            device_id: None,
                            inode: None,
                        },
                    )
                })
                .collect();
            Self { root, files }
        }

        fn relative_key(&self, path: &Path) -> Result<PathBuf> {
            if let Ok(relative) = path.strip_prefix(&self.root) {
                return Ok(relative.to_path_buf());
            }
            if self.files.contains_key(path) {
                return Ok(path.to_path_buf());
            }
            Err(anyhow!("unknown virtual path {}", path.display()))
        }
    }

    impl SearchBackend for FakeBackend {
        fn normalize_root(&self, root: &Path) -> Result<PathBuf> {
            if root == self.root {
                Ok(self.root.clone())
            } else {
                Err(anyhow!("unexpected root {}", root.display()))
            }
        }

        fn discover(&self, request: &DiscoveryRequest) -> Result<DiscoveryReport> {
            if request.root != self.root {
                return Err(anyhow!("unexpected root {}", request.root.display()));
            }
            let candidates = self
                .files
                .keys()
                .cloned()
                .map(|relative| BackendPathIdentity {
                    display_path: relative.clone(),
                    resolved_path: self.root.join(&relative),
                    root_relative_path: Some(relative),
                    resolution: crate::PathResolution::Canonical,
                })
                .collect();
            Ok(DiscoveryReport {
                candidates,
                ..Default::default()
            })
        }

        fn normalize_path(
            &self,
            root: &Path,
            _display_root: &Path,
            path: &Path,
        ) -> Result<BackendPathIdentity> {
            if root != self.root {
                return Err(anyhow!("unexpected root {}", root.display()));
            }
            let relative = self.relative_key(path)?;
            Ok(BackendPathIdentity {
                display_path: relative.clone(),
                resolved_path: self.root.join(&relative),
                root_relative_path: Some(relative),
                resolution: crate::PathResolution::Canonical,
            })
        }

        fn stat(&self, path: &Path) -> Result<BackendMetadata> {
            let relative = self.relative_key(path)?;
            self.files
                .get(&relative)
                .cloned()
                .ok_or_else(|| anyhow!("missing metadata for {}", path.display()))
        }

        fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
            let _ = self.relative_key(path)?;
            Ok(b"placeholder".to_vec())
        }
    }

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
    fn test_report_path_output_with_backend() {
        let backend = Arc::new(FakeBackend::new(
            PathBuf::from("/virtual"),
            [PathBuf::from("src/lib.rs")],
        ));
        let mut writer = Vec::new();
        print_path_output_with_backend(
            backend.as_ref(),
            &mut writer,
            &[PathBuf::from("/virtual/src/lib.rs")],
            &crate::Format::Find,
            crate::TimeFormat::Local,
        )
        .unwrap();

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("-rw-r--r--"));
        assert!(output.contains("/virtual/src/lib.rs"));
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
