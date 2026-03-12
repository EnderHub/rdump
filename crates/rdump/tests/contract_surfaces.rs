use rdump::contracts::{LimitValue, Limits, OutputMode, SearchItem, SearchRequest};
use rdump::{execute_search_request, search, search_iter, SearchOptions};
use std::fs;
use tempfile::tempdir;

#[test]
fn sdk_search_results_are_stably_sorted_by_path() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("zeta.rs"), "fn zeta() {}\n").unwrap();
    fs::create_dir_all(dir.path().join("nested")).unwrap();
    fs::write(dir.path().join("nested/alpha.rs"), "fn alpha() {}\n").unwrap();
    fs::write(dir.path().join("beta.rs"), "fn beta() {}\n").unwrap();

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    let paths: Vec<_> = results
        .iter()
        .map(|result| result.path.display().to_string())
        .collect();
    assert!(paths[0].ends_with("beta.rs"));
    assert!(paths[1].ends_with("nested/alpha.rs"));
    assert!(paths[2].ends_with("zeta.rs"));
}

#[test]
fn request_path_output_is_sorted_and_includes_metadata() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("zeta.rs"), "fn zeta() {}\n").unwrap();
    fs::create_dir_all(dir.path().join("nested")).unwrap();
    fs::write(dir.path().join("nested/alpha.rs"), "fn alpha() {}\n").unwrap();
    fs::write(dir.path().join("beta.rs"), "fn beta() {}\n").unwrap();

    let response = execute_search_request(&SearchRequest {
        query: "ext:rs".to_string(),
        root: Some(dir.path().to_string_lossy().to_string()),
        output: Some(OutputMode::Paths),
        limits: Some(Limits {
            max_results: LimitValue::Unlimited,
            max_matches_per_file: LimitValue::Unlimited,
            max_bytes_per_file: LimitValue::Unlimited,
            max_total_bytes: LimitValue::Unlimited,
            max_match_bytes: LimitValue::Unlimited,
            max_snippet_bytes: LimitValue::Unlimited,
            max_errors: LimitValue::Unlimited,
        }),
        ..Default::default()
    })
    .unwrap();

    let paths: Vec<_> = response
        .results
        .iter()
        .map(|item| match item {
            SearchItem::Path { path, .. } => path.clone(),
            other => panic!("expected path item, got {other:?}"),
        })
        .collect();
    assert!(paths[0].ends_with("beta.rs"));
    assert!(paths[1].ends_with("nested/alpha.rs"));
    assert!(paths[2].ends_with("zeta.rs"));

    for item in response.results {
        match item {
            SearchItem::Path { metadata, file, .. } => {
                assert!(metadata.size_bytes > 0);
                assert!(!metadata.permissions_display.is_empty());
                assert!(metadata.modified_unix_millis.is_some());
                assert!(!file.resolved_path.is_empty());
                assert!(file.root_relative_path.is_some());
                #[cfg(unix)]
                assert_eq!(metadata.permissions_display.len(), 10);
                #[cfg(not(unix))]
                assert!(
                    metadata.permissions_display == "readonly"
                        || metadata.permissions_display == "readwrite"
                );
            }
            other => panic!("expected path item, got {other:?}"),
        }
    }
}

#[test]
fn contract_search_stats_expose_tree_cache_metrics() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();

    let response = execute_search_request(&SearchRequest {
        query: "func:main | func:helper".to_string(),
        root: Some(dir.path().to_string_lossy().to_string()),
        output: Some(OutputMode::Summary),
        limits: Some(Limits {
            max_results: LimitValue::Unlimited,
            max_matches_per_file: LimitValue::Unlimited,
            max_bytes_per_file: LimitValue::Unlimited,
            max_total_bytes: LimitValue::Unlimited,
            max_match_bytes: LimitValue::Unlimited,
            max_snippet_bytes: LimitValue::Unlimited,
            max_errors: LimitValue::Unlimited,
        }),
        ..Default::default()
    })
    .unwrap();

    assert!(response.stats.tree_cache_misses > 0);
    assert!(response.stats.tree_cache_hits > 0);
}

#[test]
fn search_iter_can_disable_snapshot_drift_detection() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("main.rs");
    fs::write(&path, "fn main() {}\n").unwrap();

    let mut iter = search_iter(
        "func:main",
        SearchOptions {
            root: dir.path().to_path_buf(),
            snapshot_drift_detection: false,
            ..Default::default()
        },
    )
    .unwrap();

    fs::write(&path, "fn main() { println!(\"changed\"); }\n").unwrap();
    let result = iter.next().unwrap().unwrap();
    assert!(result.metadata.snapshot.is_none());
    assert!(!result.metadata.snapshot_drift);
    assert!(!result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == rdump::content::DiagnosticKind::SnapshotDrift));
}
