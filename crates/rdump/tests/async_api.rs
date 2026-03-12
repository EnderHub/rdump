#![cfg(feature = "async")]

use futures::{future, StreamExt};
use rdump::{search_all_async, search_async, SearchOptions};
use tempfile::tempdir;

#[tokio::test]
async fn test_search_async_basic_streaming() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

    let mut stream = search_async(
        "func:main",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let mut count = 0;
    while let Some(item) = stream.next().await {
        let res = item.unwrap();
        assert!(res.path.ends_with("main.rs"));
        count += 1;
    }
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_search_async_early_termination() {
    let dir = tempdir().unwrap();
    for i in 0..5 {
        std::fs::write(dir.path().join(format!("file{i}.rs")), "fn main() {}").unwrap();
    }

    let mut stream = search_async(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let first = stream.next().await.unwrap().unwrap();
    assert!(first.path.extension().unwrap() == "rs");
    // Drop stream here; producer should stop without panicking.
    drop(stream);
}

#[tokio::test]
async fn test_search_async_propagates_error() {
    let dir = tempdir().unwrap();
    let mut stream = search_async(
        "invalid((query",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let err = stream.next().await.unwrap().unwrap_err();
    let msg = err.to_string().to_lowercase();
    assert!(msg.contains("invalid") || msg.contains("parse") || msg.contains("syntax"));
}

#[tokio::test]
async fn test_search_async_empty_results() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("none.rs"), "fn nothing() {}").unwrap();

    let mut stream = search_async(
        "ext:py",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn test_search_async_drop_stream() {
    let dir = tempdir().unwrap();
    for i in 0..3 {
        std::fs::write(dir.path().join(format!("file{i}.rs")), "fn main() {}").unwrap();
    }

    {
        let mut stream = search_async(
            "ext:rs",
            SearchOptions {
                root: dir.path().to_path_buf(),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        // Consume one item then drop stream; producer should stop without hanging.
        let _ = stream.next().await;
    }
}

#[tokio::test]
async fn test_search_async_concurrent_processing() {
    let dir = tempdir().unwrap();
    for i in 0..4 {
        std::fs::write(dir.path().join(format!("file{i}.rs")), "fn main() {}").unwrap();
    }

    let opts = SearchOptions {
        root: dir.path().to_path_buf(),
        ..Default::default()
    };

    let (count_a, count_b) = future::join(
        async {
            let mut s = search_async("ext:rs", opts.clone()).await.unwrap();
            let mut c = 0;
            while let Some(item) = s.next().await {
                item.unwrap();
                c += 1;
            }
            c
        },
        async {
            let mut s = search_async("func:main", opts.clone()).await.unwrap();
            let mut c = 0;
            while let Some(item) = s.next().await {
                item.unwrap();
                c += 1;
            }
            c
        },
    )
    .await;

    assert_eq!(count_a, 4);
    assert_eq!(count_b, 4);
}

#[tokio::test]
async fn test_search_all_async_collects() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

    let results = search_all_async(
        "func:main",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].path.ends_with("main.rs"));
}

#[tokio::test]
async fn test_search_all_async_empty() {
    let dir = tempdir().unwrap();
    let results = search_all_async(
        "func:main",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert!(results.is_empty());
}

#[tokio::test]
async fn test_search_all_async_error() {
    let dir = tempdir().unwrap();
    let err = search_all_async(
        "invalid((query",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .await
    .unwrap_err();

    let msg = err.to_string().to_lowercase();
    assert!(msg.contains("invalid") || msg.contains("parse") || msg.contains("syntax"));
}
