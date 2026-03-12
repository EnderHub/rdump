use rdump::{search, search_paths, SearchOptions};
use std::fs;
use std::time::Instant;
use tempfile::tempdir;

fn build_fixture(file_count: usize) -> tempfile::TempDir {
    let dir = tempdir().unwrap();

    for index in 0..file_count {
        let path = dir.path().join(format!("file{index}.rs"));
        fs::write(
            path,
            format!(
                r#"
pub struct User{index} {{
    id: usize,
}}

impl User{index} {{
    pub fn new() -> Self {{
        println!("user {index}");
        Self {{ id: {index} }}
    }}
}}
"#
            ),
        )
        .unwrap();
    }

    dir
}

fn build_large_fixture(file_count: usize, bytes_per_file: usize) -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    let payload = "x".repeat(bytes_per_file.max(1));

    for index in 0..file_count {
        let path = dir.path().join(format!("large{index}.rs"));
        fs::write(
            path,
            format!("pub fn item_{index}() -> &'static str {{ \"{payload}\" }}\n"),
        )
        .unwrap();
    }

    dir
}

fn assert_under_env_threshold(env_key: &str, elapsed_ms: u128) {
    if let Ok(value) = std::env::var(env_key) {
        let threshold = value.parse::<u128>().expect("threshold env should parse");
        assert!(
            elapsed_ms <= threshold,
            "{env_key} exceeded: elapsed={elapsed_ms} threshold={threshold}"
        );
    }
}

#[test]
#[ignore = "perf harness"]
fn perf_metadata_only_query() {
    let dir = build_fixture(250);
    let started = Instant::now();
    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    eprintln!(
        "metadata_only: files={} elapsed_ms={}",
        results.len(),
        started.elapsed().as_millis()
    );
    assert_under_env_threshold("RDUMP_PERF_METADATA_MAX_MS", started.elapsed().as_millis());
}

#[test]
#[ignore = "perf harness"]
fn perf_semantic_only_query() {
    let dir = build_fixture(250);
    let started = Instant::now();
    let results = search(
        "func:new",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    eprintln!(
        "semantic_only: files={} elapsed_ms={}",
        results.len(),
        started.elapsed().as_millis()
    );
    assert_under_env_threshold("RDUMP_PERF_SEMANTIC_MAX_MS", started.elapsed().as_millis());
}

#[test]
#[ignore = "perf harness"]
fn perf_mixed_query() {
    let dir = build_fixture(250);
    let started = Instant::now();
    let results = search(
        "ext:rs & contains:println & func:new",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    eprintln!(
        "mixed_query: files={} elapsed_ms={}",
        results.len(),
        started.elapsed().as_millis()
    );
    assert_under_env_threshold("RDUMP_PERF_MIXED_MAX_MS", started.elapsed().as_millis());
}

#[test]
#[ignore = "perf harness"]
fn perf_path_only_query() {
    let dir = build_fixture(250);
    let started = Instant::now();
    let paths = search_paths(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    eprintln!(
        "path_only: files={} elapsed_ms={}",
        paths.len(),
        started.elapsed().as_millis()
    );
    assert_under_env_threshold("RDUMP_PERF_PATH_MAX_MS", started.elapsed().as_millis());
}

#[test]
#[ignore = "scale harness"]
fn synthetic_large_repo_summary() {
    let dir = build_large_fixture(2_000, 512);
    let started = Instant::now();
    let results = search(
        "ext:rs & func:item_1999",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    eprintln!(
        "large_repo_summary: files={} elapsed_ms={}",
        results.len(),
        started.elapsed().as_millis()
    );
    assert_under_env_threshold(
        "RDUMP_PERF_LARGE_SUMMARY_MAX_MS",
        started.elapsed().as_millis(),
    );
}

#[test]
#[ignore = "scale harness"]
fn synthetic_large_repo_path_only() {
    let dir = build_large_fixture(5_000, 64);
    let started = Instant::now();
    let paths = search_paths(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    eprintln!(
        "large_repo_path_only: files={} elapsed_ms={}",
        paths.len(),
        started.elapsed().as_millis()
    );
    assert_under_env_threshold(
        "RDUMP_PERF_LARGE_PATH_MAX_MS",
        started.elapsed().as_millis(),
    );
}
