use once_cell::sync::Lazy;
use rdump::{search, SearchOptions};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::tempdir;

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[test]
fn test_search_basic_extension() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].path.to_string_lossy().ends_with(".rs"));
}

#[test]
fn test_search_no_results() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.txt");
    fs::write(&file, "some text").unwrap();

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(results.is_empty());
}

#[test]
fn test_search_nonexistent_root() {
    let result = search(
        "ext:rs",
        SearchOptions {
            root: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        },
    );

    assert!(result.is_err());
}

#[test]
fn test_search_invalid_query() {
    let dir = tempdir().unwrap();

    let result = search(
        "invalid::: syntax",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    );

    assert!(result.is_err());
}

#[test]
fn test_search_multiple_files() {
    let dir = tempdir().unwrap();

    for i in 0..5 {
        let file = dir.path().join(format!("file{i}.rs"));
        fs::write(&file, format!("fn func{i}() {{}}")).unwrap();
    }

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(results.len(), 5);
}

#[test]
fn test_search_empty_result_vec() {
    let dir = tempdir().unwrap();

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(results.is_empty());
}

#[test]
fn test_search_first_error_short_circuits() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let options = SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    };

    // Create iterator via search_iter, then collect via search
    let iter = rdump::search_iter("ext:rs", options.clone()).unwrap();
    fs::remove_file(&file).unwrap();
    let result: Result<Vec<_>, _> = iter.collect();
    assert!(result.is_err());
}

#[test]
fn test_search_whole_file_match() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(results[0].is_whole_file_match());
}

#[test]
fn test_search_with_matches() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let results = search(
        "func:main",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(!results[0].is_whole_file_match());
    assert!(!results[0].matches.is_empty());
}

#[test]
fn test_search_empty_query_with_preset() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let dir = tempdir().unwrap();
    let config_root = dir.path().join("config");
    let preset_dir = config_root.join("rdump");
    fs::create_dir_all(&preset_dir).unwrap();
    fs::write(
        preset_dir.join("config.toml"),
        r#"
            [presets]
            rust = "ext:rs"
        "#,
    )
    .unwrap();

    let file = dir.path().join("code").join("main.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "fn main() {}").unwrap();

    std::env::set_var("RDUMP_TEST_CONFIG_DIR", config_root.to_str().unwrap());

    let results = search(
        "",
        SearchOptions {
            root: file.parent().unwrap().to_path_buf(),
            presets: vec!["rust".to_string()],
            ..Default::default()
        },
    )
    .unwrap();

    std::env::remove_var("RDUMP_TEST_CONFIG_DIR");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path.file_name().unwrap(), "main.rs");
}
