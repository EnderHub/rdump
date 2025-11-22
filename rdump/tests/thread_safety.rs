use rayon::prelude::*;
use rdump::{search, search_iter, Match, SearchOptions, SearchResult, SearchResultIterator};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use tempfile::tempdir;

// Compile-time assertions ----------------------------------------------------

#[test]
fn types_are_send() {
    fn assert_send<T: Send>() {}

    assert_send::<SearchOptions>();
    assert_send::<SearchResult>();
    assert_send::<Match>();
    assert_send::<SearchResultIterator>();
}

#[test]
fn types_are_sync() {
    fn assert_sync<T: Sync>() {}

    assert_sync::<SearchOptions>();
    assert_sync::<SearchResult>();
    assert_sync::<Match>();
    // Iterators are expected to be consumed by a single thread; Sync is not required.
}

// Runtime behaviors ----------------------------------------------------------

#[test]
fn search_options_moves_between_threads() {
    let options = SearchOptions {
        root: PathBuf::from("/tmp"),
        presets: vec!["rust".to_string()],
        no_ignore: true,
        hidden: false,
        max_depth: Some(5),
        sql_dialect: None,
    };

    let handle = thread::spawn(move || {
        assert_eq!(options.presets.len(), 1);
        assert_eq!(options.max_depth, Some(5));
        options
    });

    let returned = handle.join().unwrap();
    assert_eq!(returned.root, PathBuf::from("/tmp"));
}

#[test]
fn search_result_moves_between_threads() {
    let result = SearchResult {
        path: PathBuf::from("test.rs"),
        matches: vec![Match {
            start_line: 1,
            end_line: 1,
            start_column: 0,
            end_column: 10,
            byte_range: 0..10,
            text: "fn main()".to_string(),
        }],
        content: "fn main() {}".to_string(),
    };

    let handle = thread::spawn(move || {
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].start_line, 1);
        result
    });

    let returned = handle.join().unwrap();
    assert_eq!(returned.path.to_str().unwrap(), "test.rs");
}

#[test]
fn search_result_shared_via_arc() {
    let result = Arc::new(SearchResult {
        path: PathBuf::from("shared.rs"),
        matches: vec![],
        content: "// shared content".to_string(),
    });

    let mut handles = vec![];
    for _ in 0..3 {
        let result_clone = Arc::clone(&result);
        handles.push(thread::spawn(move || {
            assert_eq!(result_clone.path.to_str().unwrap(), "shared.rs");
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn iterator_moves_between_threads() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("test.rs"), "fn main() {}\n").unwrap();

    let iter = search_iter(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    let handle = thread::spawn(move || {
        let results: Vec<_> = iter.filter_map(Result::ok).collect();
        results.len()
    });

    let count = handle.join().unwrap();
    assert_eq!(count, 1);
}

#[test]
fn results_process_in_parallel_with_rayon() {
    let dir = tempdir().unwrap();
    for i in 0..5 {
        let file = dir.path().join(format!("file{}.rs", i));
        std::fs::write(&file, format!("fn func{}() {{}}", i)).unwrap();
    }

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .unwrap();

    let paths: Vec<_> = results
        .par_iter()
        .map(|r| r.path.to_string_lossy().to_string())
        .collect();

    assert_eq!(paths.len(), 5);
}
