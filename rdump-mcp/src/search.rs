use crate::limits::{resolve_limits, ResolvedLimits, DEFAULT_CONTEXT_LINES};
use crate::types::{
    MatchInfo, OutputMode, SearchArgs, SearchItem, SearchRequest, SearchResponse, SearchStats,
    Snippet,
};
use rdump::{search_iter, SearchOptions};
use std::path::PathBuf;
use turbomcp::prelude::{McpError, McpResult};

pub fn build_search_request(args: SearchArgs) -> McpResult<SearchRequest> {
    let query = normalize_query(args.query)?;
    Ok(SearchRequest {
        query,
        root: args.root,
        presets: args.presets.unwrap_or_default(),
        no_ignore: args.no_ignore.unwrap_or(false),
        hidden: args.hidden.unwrap_or(false),
        max_depth: args.max_depth,
        sql_dialect: args.sql_dialect,
        output: args.output,
        limits: args.limits,
        context_lines: args.context_lines,
        skip_errors: args.skip_errors,
    })
}

pub fn run_search(params: SearchRequest) -> McpResult<SearchResponse> {
    let output = params.output.unwrap_or(OutputMode::Snippets);
    let limits = resolve_limits(params.limits);
    let context_lines = params.context_lines.unwrap_or(DEFAULT_CONTEXT_LINES);
    let skip_errors = params.skip_errors.unwrap_or(true);

    let root = params.root.unwrap_or_else(|| ".".to_string());

    let options = SearchOptions {
        root: PathBuf::from(&root),
        presets: params.presets,
        no_ignore: params.no_ignore,
        hidden: params.hidden,
        max_depth: params.max_depth,
        sql_dialect: params.sql_dialect.map(Into::into),
    };

    let mut results = Vec::new();
    let mut errors = Vec::new();
    let mut errors_truncated = false;
    let mut truncated = false;
    let mut truncation_reason = None;
    let mut returned_matches = 0usize;
    let mut returned_bytes = 0usize;

    let iter = search_iter(&params.query, options)
        .map_err(|e| McpError::Tool(e.to_string()))?;

    for item in iter {
        let result = match item {
            Ok(result) => result,
            Err(err) => {
                if skip_errors {
                    if errors.len() < limits.max_errors {
                        errors.push(err.to_string());
                    } else {
                        errors_truncated = true;
                    }
                    continue;
                }
                return Err(McpError::Tool(err.to_string()));
            }
        };

        if results.len() >= limits.max_results {
            truncated = true;
            truncation_reason = Some("max_results".to_string());
            break;
        }

        let path = result.path.display().to_string();
        let (item, item_match_count) =
            build_item(output, &path, &result, context_lines, &limits);

        let item_bytes = estimate_item_bytes(&item);
        let next_bytes = returned_bytes.saturating_add(item_bytes);
        if next_bytes > limits.max_total_bytes {
            truncated = true;
            truncation_reason = Some("max_total_bytes".to_string());
            break;
        }

        returned_bytes = next_bytes;
        returned_matches += item_match_count;
        results.push(item);
    }

    let stats = SearchStats {
        returned_files: results.len(),
        returned_matches,
        returned_bytes,
        errors: errors.len(),
    };

    Ok(SearchResponse {
        query: params.query,
        root,
        output,
        results,
        stats,
        errors,
        errors_truncated,
        truncated,
        truncation_reason,
    })
}

pub fn format_search_text(response: &SearchResponse) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Query: {}", response.query));
    lines.push(format!("Root: {}", response.root));
    lines.push(format!("Output: {}", output_mode_label(response.output)));
    lines.push(format!(
        "Results: {} files, {} matches, {} bytes",
        response.stats.returned_files, response.stats.returned_matches, response.stats.returned_bytes
    ));

    if response.truncated {
        let reason = response.truncation_reason.as_deref().unwrap_or("unknown");
        lines.push(format!("Truncated: true ({reason})"));
    }

    if !response.errors.is_empty() {
        let suffix = if response.errors_truncated {
            " (errors truncated)"
        } else {
            ""
        };
        lines.push(format!("Errors: {}{}", response.errors.len(), suffix));
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

pub(crate) fn build_item(
    output: OutputMode,
    path: &str,
    result: &rdump::SearchResult,
    context_lines: usize,
    limits: &ResolvedLimits,
) -> (SearchItem, usize) {
    let whole_file_match = result.matches.is_empty();
    let match_limit = limits.max_matches_per_file;
    let matches: Vec<&rdump::Match> = if match_limit == usize::MAX {
        result.matches.iter().collect()
    } else {
        result.matches.iter().take(match_limit).collect()
    };
    let matches_truncated = matches.len() < result.matches.len();

    match output {
        OutputMode::Paths => (
            SearchItem::Path {
                path: path.to_string(),
            },
            0,
        ),
        OutputMode::Summary => {
            let returned_matches = matches.len();
            (
                SearchItem::Summary {
                    path: path.to_string(),
                    matches: returned_matches,
                    whole_file_match,
                    matches_truncated,
                },
                returned_matches,
            )
        }
        OutputMode::Matches => {
            let match_infos: Vec<MatchInfo> = matches
                .iter()
                .map(|m| build_match_info(m, limits.max_match_bytes))
                .collect();

            let returned_matches = match_infos.len();
            (
                SearchItem::Matches {
                    path: path.to_string(),
                    matches: match_infos,
                    whole_file_match,
                    matches_truncated,
                },
                returned_matches,
            )
        }
        OutputMode::Snippets => {
            let snippets: Vec<Snippet> = matches
                .iter()
                .map(|m| build_snippet(result, m, context_lines, limits.max_snippet_bytes))
                .collect();

            let returned_matches = snippets.len();
            (
                SearchItem::Snippets {
                    path: path.to_string(),
                    snippets,
                    whole_file_match,
                    matches_truncated,
                },
                returned_matches,
            )
        }
        OutputMode::Full => {
            let match_infos: Vec<MatchInfo> = matches
                .iter()
                .map(|m| build_match_info(m, limits.max_match_bytes))
                .collect();
            let returned_matches = match_infos.len();

            let (content, content_truncated) =
                truncate_str_bytes(&result.content, limits.max_bytes_per_file);

            (
                SearchItem::Full {
                    path: path.to_string(),
                    content,
                    matches: match_infos,
                    content_truncated,
                    matches_truncated,
                },
                returned_matches,
            )
        }
    }
}

pub(crate) fn build_match_info(m: &rdump::Match, max_bytes: usize) -> MatchInfo {
    let (text, text_truncated) = if max_bytes == usize::MAX {
        (m.text.clone(), false)
    } else {
        truncate_str_bytes(&m.text, max_bytes)
    };

    MatchInfo {
        start_line: m.start_line,
        end_line: m.end_line,
        start_column: m.start_column,
        end_column: m.end_column,
        byte_range: [m.byte_range.start, m.byte_range.end],
        text: Some(text),
        text_truncated,
    }
}

pub(crate) fn build_snippet(
    result: &rdump::SearchResult,
    m: &rdump::Match,
    context_lines: usize,
    max_bytes: usize,
) -> Snippet {
    if result.content.is_empty() {
        return Snippet {
            start_line: m.start_line,
            end_line: m.end_line,
            match_start_line: m.start_line,
            match_end_line: m.end_line,
            text: String::new(),
            text_truncated: false,
        };
    }

    let lines: Vec<&str> = result.content.lines().collect();
    let total_lines = lines.len();
    let mut start_line = m.start_line.saturating_sub(context_lines);
    if start_line == 0 {
        start_line = 1;
    }
    let end_line = (m.end_line + context_lines).min(total_lines);

    let start_idx = start_line.saturating_sub(1);
    let end_idx = end_line.saturating_sub(1);

    let mut text = String::new();
    for line in lines.iter().take(end_idx + 1).skip(start_idx) {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(line);
    }

    let (text, text_truncated) = if max_bytes == usize::MAX {
        (text, false)
    } else {
        truncate_str_bytes(&text, max_bytes)
    };

    Snippet {
        start_line,
        end_line,
        match_start_line: m.start_line,
        match_end_line: m.end_line,
        text,
        text_truncated,
    }
}

pub(crate) fn truncate_str_bytes(input: &str, max_bytes: usize) -> (String, bool) {
    if max_bytes == usize::MAX || input.len() <= max_bytes {
        return (input.to_string(), false);
    }

    let mut end = max_bytes;
    while end > 0 && !input.is_char_boundary(end) {
        end -= 1;
    }

    (input[..end].to_string(), true)
}

pub(crate) fn estimate_item_bytes(item: &SearchItem) -> usize {
    match item {
        SearchItem::Path { path } => path.len(),
        SearchItem::Summary { path, .. } => path.len(),
        SearchItem::Matches { path, matches, .. } => {
            path.len()
                + matches
                    .iter()
                    .map(|m| m.text.as_ref().map_or(0, |text| text.len()))
                    .sum::<usize>()
        }
        SearchItem::Snippets { path, snippets, .. } => {
            path.len() + snippets.iter().map(|snippet| snippet.text.len()).sum::<usize>()
        }
        SearchItem::Full { path, content, .. } => path.len() + content.len(),
    }
}

fn normalize_query(query: Option<String>) -> McpResult<String> {
    let query = query.unwrap_or_default();
    if query.trim().is_empty() {
        return Err(McpError::Tool(
            "query is required (example: contains:hello)".to_string(),
        ));
    }
    Ok(query)
}

fn output_mode_label(output: OutputMode) -> &'static str {
    match output {
        OutputMode::Paths => "paths",
        OutputMode::Matches => "matches",
        OutputMode::Snippets => "snippets",
        OutputMode::Full => "full",
        OutputMode::Summary => "summary",
    }
}

fn item_path(item: &SearchItem) -> &str {
    match item {
        SearchItem::Path { path } => path,
        SearchItem::Summary { path, .. } => path,
        SearchItem::Matches { path, .. } => path,
        SearchItem::Snippets { path, .. } => path,
        SearchItem::Full { path, .. } => path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::ResolvedLimits;
    use crate::types::{LimitValue, Limits, OutputMode, SearchArgs, SearchRequest};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn make_match(content: &str, needle: &str, line: usize) -> rdump::Match {
        let start = content.find(needle).unwrap();
        let end = start + needle.len();
        rdump::Match {
            start_line: line,
            end_line: line,
            start_column: 0,
            end_column: needle.len(),
            byte_range: start..end,
            text: needle.to_string(),
        }
    }

    fn sample_result() -> rdump::SearchResult {
        let content = "zero\none\ntwo";
        let first = make_match(content, "one", 2);
        let second = make_match(content, "two", 3);
        rdump::SearchResult {
            path: PathBuf::from("sample.txt"),
            matches: vec![first, second],
            content: content.to_string(),
        }
    }

    #[test]
    fn truncate_str_bytes_respects_char_boundaries() {
        let input = "a\u{1F600}b";
        let (out, truncated) = truncate_str_bytes(input, 2);
        assert_eq!(out, "a");
        assert!(truncated);
    }

    #[test]
    fn build_snippet_includes_context_lines() {
        let result = sample_result();
        let snippet = build_snippet(&result, &result.matches[0], 1, usize::MAX);
        assert_eq!(snippet.start_line, 1);
        assert_eq!(snippet.end_line, 3);
        assert!(snippet.text.contains("zero\none\ntwo"));
    }

    #[test]
    fn build_item_truncates_matches() {
        let result = sample_result();
        let limits = ResolvedLimits {
            max_results: 10,
            max_matches_per_file: 1,
            max_bytes_per_file: usize::MAX,
            max_total_bytes: usize::MAX,
            max_match_bytes: usize::MAX,
            max_snippet_bytes: usize::MAX,
            max_errors: 10,
        };

        let (item, returned_matches) =
            build_item(OutputMode::Matches, "sample.txt", &result, 0, &limits);
        assert_eq!(returned_matches, 1);
        match item {
            SearchItem::Matches {
                matches_truncated,
                matches,
                ..
            } => {
                assert!(matches_truncated);
                assert_eq!(matches.len(), 1);
            }
            _ => panic!("unexpected item kind"),
        }
    }

    #[test]
    fn build_match_info_truncates_text() {
        let result = sample_result();
        let match_info = build_match_info(&result.matches[0], 1);
        assert!(match_info.text_truncated);
        assert_eq!(match_info.text.as_deref(), Some("o"));
    }

    #[test]
    fn estimate_item_bytes_counts_content() {
        let result = sample_result();
        let limits = ResolvedLimits {
            max_results: 10,
            max_matches_per_file: usize::MAX,
            max_bytes_per_file: usize::MAX,
            max_total_bytes: usize::MAX,
            max_match_bytes: usize::MAX,
            max_snippet_bytes: usize::MAX,
            max_errors: 10,
        };
        let (item, _) = build_item(OutputMode::Full, "sample.txt", &result, 0, &limits);
        assert!(estimate_item_bytes(&item) >= result.content.len());
    }

    #[test]
    fn search_request_finds_content_in_temp_dir() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("sample.txt");
        fs::write(&file_path, "hello world").unwrap();

        let request = SearchRequest {
            query: "contains:hello".to_string(),
            root: Some(dir.path().to_string_lossy().to_string()),
            presets: Vec::new(),
            no_ignore: false,
            hidden: false,
            max_depth: None,
            sql_dialect: None,
            output: Some(OutputMode::Paths),
            limits: Some(Limits::default()),
            context_lines: None,
            skip_errors: Some(true),
        };

        let response = run_search(request).unwrap();
        assert_eq!(response.results.len(), 1);
    }

    #[test]
    fn run_search_invalid_query_returns_error() {
        let dir = tempdir().unwrap();
        let request = SearchRequest {
            query: "invalid((syntax".to_string(),
            root: Some(dir.path().to_string_lossy().to_string()),
            presets: Vec::new(),
            no_ignore: false,
            hidden: false,
            max_depth: None,
            sql_dialect: None,
            output: Some(OutputMode::Paths),
            limits: Some(Limits::default()),
            context_lines: None,
            skip_errors: Some(true),
        };

        let result = run_search(request);
        assert!(result.is_err());
    }

    #[test]
    fn run_search_respects_max_results() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("one.txt"), "hello one").unwrap();
        fs::write(dir.path().join("two.txt"), "hello two").unwrap();

        let mut limits = Limits::default();
        limits.max_results = LimitValue::Value(1);

        let request = SearchRequest {
            query: "contains:hello".to_string(),
            root: Some(dir.path().to_string_lossy().to_string()),
            presets: Vec::new(),
            no_ignore: false,
            hidden: false,
            max_depth: None,
            sql_dialect: None,
            output: Some(OutputMode::Paths),
            limits: Some(limits),
            context_lines: None,
            skip_errors: Some(true),
        };

        let response = run_search(request).unwrap();
        assert_eq!(response.results.len(), 1);
        assert!(response.truncated);
        assert_eq!(response.truncation_reason.as_deref(), Some("max_results"));
    }

    #[test]
    fn run_search_truncates_by_total_bytes() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("sample.txt"), "hello world").unwrap();

        let mut limits = Limits::default();
        limits.max_total_bytes = LimitValue::Value(1);

        let request = SearchRequest {
            query: "contains:hello".to_string(),
            root: Some(dir.path().to_string_lossy().to_string()),
            presets: Vec::new(),
            no_ignore: false,
            hidden: false,
            max_depth: None,
            sql_dialect: None,
            output: Some(OutputMode::Paths),
            limits: Some(limits),
            context_lines: None,
            skip_errors: Some(true),
        };

        let response = run_search(request).unwrap();
        assert!(response.results.is_empty());
        assert!(response.truncated);
        assert_eq!(response.truncation_reason.as_deref(), Some("max_total_bytes"));
    }

    #[test]
    fn run_search_full_output_truncates_content() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("sample.txt"), "hello world hello world").unwrap();

        let mut limits = Limits::default();
        limits.max_bytes_per_file = LimitValue::Value(5);

        let request = SearchRequest {
            query: "contains:hello".to_string(),
            root: Some(dir.path().to_string_lossy().to_string()),
            presets: Vec::new(),
            no_ignore: false,
            hidden: false,
            max_depth: None,
            sql_dialect: None,
            output: Some(OutputMode::Full),
            limits: Some(limits),
            context_lines: None,
            skip_errors: Some(true),
        };

        let response = run_search(request).unwrap();
        let first = response.results.first().expect("search result");
        match first {
            SearchItem::Full {
                content_truncated,
                content,
                ..
            } => {
                assert!(*content_truncated);
                assert!(content.len() <= 5);
            }
            _ => panic!("expected full output"),
        }
    }

    #[test]
    fn build_snippet_truncates_text() {
        let result = sample_result();
        let snippet = build_snippet(&result, &result.matches[0], 1, 4);
        assert!(snippet.text_truncated);
    }

    #[test]
    fn build_search_request_requires_query() {
        let args = SearchArgs {
            query: None,
            root: None,
            presets: None,
            no_ignore: None,
            hidden: None,
            max_depth: None,
            sql_dialect: None,
            output: None,
            limits: None,
            context_lines: None,
            skip_errors: None,
        };
        let result = build_search_request(args);
        assert!(result.is_err());
    }

    #[test]
    fn build_item_summary_counts_matches() {
        let result = sample_result();
        let limits = ResolvedLimits {
            max_results: 10,
            max_matches_per_file: 10,
            max_bytes_per_file: usize::MAX,
            max_total_bytes: usize::MAX,
            max_match_bytes: usize::MAX,
            max_snippet_bytes: usize::MAX,
            max_errors: 10,
        };

        let (item, returned_matches) =
            build_item(OutputMode::Summary, "sample.txt", &result, 0, &limits);
        assert_eq!(returned_matches, 2);
        match item {
            SearchItem::Summary { matches, .. } => assert_eq!(matches, 2),
            _ => panic!("unexpected item kind"),
        }
    }
}
