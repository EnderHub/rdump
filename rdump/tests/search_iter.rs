use once_cell::sync::Lazy;
use rdump::{search_iter, SearchOptions};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::tempdir;

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[test]
fn test_search_iter_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let results: Vec<_> = search_iter(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path.file_name().unwrap(), "test.rs");
}

#[test]
fn test_search_iter_early_termination() {
    let dir = tempdir().unwrap();
    for i in 0..10 {
        fs::write(
            dir.path().join(format!("file{i}.rs")),
            format!("fn func{i}() {{}}"),
        )
        .unwrap();
    }

    let results: Vec<_> = search_iter(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap()
    .take(3)
    .collect::<Result<Vec<_>, _>>()
    .unwrap();

    assert_eq!(results.len(), 3);
}

#[test]
fn test_search_iter_invalid_query() {
    let dir = tempdir().unwrap();

    let result = search_iter(
        "invalid::: query syntax",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    );

    assert!(result.is_err());
}

#[test]
fn test_search_iter_nonexistent_root() {
    let result = search_iter(
        "ext:rs",
        SearchOptions {
            root: PathBuf::from("/definitely/missing/path"),
            ..Default::default()
        },
    );

    assert!(result.is_err());
}

#[test]
fn test_search_iter_unknown_preset() {
    let dir = tempdir().unwrap();

    let result = search_iter(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            presets: vec!["nonexistent_preset".to_string()],
            ..Default::default()
        },
    );

    assert!(result.is_err());
}

#[test]
fn test_search_iter_empty_query_uses_presets() {
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

    let results: Vec<_> = search_iter(
        "",
        SearchOptions {
            root: file.parent().unwrap().to_path_buf(),
            presets: vec!["rust".to_string()],
            ..Default::default()
        },
    )
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap();

    std::env::remove_var("RDUMP_TEST_CONFIG_DIR");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path.file_name().unwrap(), "main.rs");
}

#[test]
fn test_search_iter_skip_errors() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let iter = search_iter(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    fs::remove_file(&file).unwrap();

    let results: Vec<_> = iter.filter_map(Result::ok).collect();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_iter_count_only() {
    let dir = tempdir().unwrap();
    for i in 0..5 {
        fs::write(
            dir.path().join(format!("file{i}.rs")),
            format!("fn func{i}() {{}}"),
        )
        .unwrap();
    }

    let count = search_iter(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap()
    .filter_map(Result::ok)
    .count();

    assert_eq!(count, 5);
}
