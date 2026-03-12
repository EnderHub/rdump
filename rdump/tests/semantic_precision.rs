mod common;

use common::{setup_custom_project, setup_fixture};
use rdump::contracts::SemanticMatchMode;
use rdump::{search_with_stats, SearchOptions};

fn total_match_count(report: &rdump::SearchReport) -> usize {
    report
        .results
        .iter()
        .map(|result| result.match_count())
        .sum()
}

#[test]
fn semantic_case_insensitive_is_shared_across_rust_and_python() {
    let dir = setup_custom_project(&[
        ("src/main.rs", "fn RunTask() {}\nfn RunTaskExtra() {}\n"),
        (
            "helpers.py",
            "def RunTask():\n    pass\n\ndef RunTaskExtra():\n    pass\n",
        ),
    ]);

    let report = search_with_stats(
        "func:runtask",
        SearchOptions {
            root: dir.path().to_path_buf(),
            semantic_match_mode: SemanticMatchMode::CaseInsensitive,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(report.results.len(), 2);
    assert!(report
        .results
        .iter()
        .all(|result| result.match_count() == 1));
    assert!(report.results.iter().all(|result| {
        result
            .matches
            .iter()
            .all(|matched| matched.text.contains("RunTask"))
    }));
}

#[test]
fn semantic_wildcard_is_shared_across_rust_and_python() {
    let dir = setup_custom_project(&[
        ("src/main.rs", "fn RunTask() {}\nfn RunTaskExtra() {}\n"),
        (
            "helpers.py",
            "def RunTask():\n    pass\n\ndef RunTaskExtra():\n    pass\n",
        ),
    ]);

    let report = search_with_stats(
        "func:RunTask*",
        SearchOptions {
            root: dir.path().to_path_buf(),
            semantic_match_mode: SemanticMatchMode::Wildcard,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(report.results.len(), 2);
    assert_eq!(
        report
            .results
            .iter()
            .map(|result| result.match_count())
            .sum::<usize>(),
        4
    );
}

#[test]
fn exact_semantic_matches_do_not_capture_near_miss_identifiers() {
    let dir = setup_custom_project(&[
        ("src/main.rs", "fn RunTask() {}\nfn RunTaskExtra() {}\n"),
        (
            "helpers.py",
            "def RunTask():\n    pass\n\ndef RunTaskExtra():\n    pass\n",
        ),
    ]);

    let report = search_with_stats(
        "func:RunTask",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(report.results.len(), 2);
    assert!(report
        .results
        .iter()
        .all(|result| result.match_count() == 1));
    assert!(report.results.iter().all(|result| {
        result
            .matches
            .iter()
            .all(|matched| !matched.text.contains("RunTaskExtra"))
    }));
}

#[test]
fn language_debug_reports_profile_selection_and_unsupported_extensions() {
    let dir = setup_custom_project(&[
        ("src/main.rs", "fn main() {}\n"),
        ("notes.txt", "plain text\n"),
    ]);

    let report = search_with_stats(
        "func:main",
        SearchOptions {
            root: dir.path().to_path_buf(),
            language_debug: true,
            ..Default::default()
        },
    )
    .unwrap();

    let messages: Vec<_> = report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(messages
        .iter()
        .any(|message| message.contains("Selected semantic profile `rs`")));
    assert!(messages
        .iter()
        .any(|message| message.contains("No semantic profile matched extension `.txt`")));
}

#[test]
fn sql_trace_reports_heuristic_reasoning() {
    let dir = setup_fixture("sql_mysql");

    let report = search_with_stats(
        "call:bump_count & ext:sql",
        SearchOptions {
            root: dir.path().to_path_buf(),
            sql_trace: true,
            ..Default::default()
        },
    )
    .unwrap();

    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("Detected `sqlmysql`")
            && diagnostic.message.contains("DELIMITER //")
    }));
}

#[test]
fn call_query_does_not_match_same_text_in_defs_comments_or_strings() {
    let dir = setup_custom_project(&[(
        "src/main.rs",
        r#"use helper::target;
fn target() {}

fn main() {
    // target in a comment should not count as a call
    let label = "target in a string should not count as a call";
    target();
    println!("{label}");
}
"#,
    )]);

    let report = search_with_stats(
        "call:target",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(report.results.len(), 1);
    assert_eq!(total_match_count(&report), 1);
}

#[test]
fn import_query_does_not_match_same_text_in_calls_comments_or_strings() {
    let dir = setup_custom_project(&[(
        "src/main.rs",
        r#"use helper::Thing;

fn main() {
    // helper should not count as an import here
    let label = "helper should not count as an import here";
    helper();
}

fn helper() {}
"#,
    )]);

    let report = search_with_stats(
        "import:helper",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(report.results.len(), 1);
    assert_eq!(total_match_count(&report), 1);
    assert!(report.results[0].matches[0]
        .text
        .contains("use helper::Thing"));
}

#[test]
fn comment_query_does_not_match_same_text_in_strings_or_identifiers() {
    let dir = setup_custom_project(&[(
        "src/main.rs",
        r#"fn todo_marker() {}

fn main() {
    // TODO: comment hit
    let label = "TODO in a string should not count as a comment";
    todo_marker();
}
"#,
    )]);

    let report = search_with_stats(
        "comment:TODO",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(report.results.len(), 1);
    assert_eq!(total_match_count(&report), 1);
    assert!(report.results[0].matches[0]
        .text
        .contains("// TODO: comment hit"));
}

#[test]
fn string_query_does_not_match_same_text_in_comments_or_calls() {
    let dir = setup_custom_project(&[(
        "src/main.rs",
        r#"fn target() {}

fn main() {
    // target in a comment should not count as a string
    let label = "target in a string should count";
    target();
    println!("{label}");
}
"#,
    )]);

    let report = search_with_stats(
        "str:\"target in a string should count\"",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(report.results.len(), 1);
    assert_eq!(total_match_count(&report), 1);
    assert!(report.results[0].matches[0]
        .text
        .contains("\"target in a string should count\""));
}
