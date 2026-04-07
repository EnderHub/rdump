#![cfg(feature = "async")]

use anyhow::{anyhow, Result};
use futures::{future, StreamExt};
use rdump::backend::{
    BackendFileType, BackendMetadata, BackendPathIdentity, DiscoveryReport, DiscoveryRequest,
    SearchBackend,
};
use rdump::{
    search_all_async, search_all_async_with_runtime, search_async, search_async_with_runtime,
    search_async_with_runtime_and_progress, SearchOptions, SearchRuntime,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use tempfile::tempdir;

#[derive(Debug)]
struct FakeBackend {
    root: PathBuf,
    files: BTreeMap<PathBuf, (Vec<u8>, BackendMetadata)>,
}

impl FakeBackend {
    fn new(root: PathBuf, files: impl IntoIterator<Item = (PathBuf, Vec<u8>)>) -> Self {
        let files = files
            .into_iter()
            .map(|(relative, bytes)| {
                let metadata = BackendMetadata {
                    size_bytes: bytes.len() as u64,
                    modified_unix_millis: Some(1_700_000_000_000),
                    readonly: false,
                    permissions_display: "-rw-r--r--".to_string(),
                    file_type: BackendFileType::File,
                    stable_token: Some(format!("token:{}", relative.display())),
                    device_id: None,
                    inode: None,
                };
                (relative, (bytes, metadata))
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
                resolution: rdump::PathResolution::Canonical,
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
            resolution: rdump::PathResolution::Canonical,
        })
    }

    fn stat(&self, path: &Path) -> Result<BackendMetadata> {
        let relative = self.relative_key(path)?;
        self.files
            .get(&relative)
            .map(|(_, metadata)| metadata.clone())
            .ok_or_else(|| anyhow!("missing metadata for {}", path.display()))
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        let relative = self.relative_key(path)?;
        self.files
            .get(&relative)
            .map(|(bytes, _)| bytes.clone())
            .ok_or_else(|| anyhow!("missing content for {}", path.display()))
    }
}

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

#[tokio::test]
async fn test_search_async_with_runtime_streaming() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

    let mut stream = search_async_with_runtime(
        SearchRuntime::real_fs(),
        "func:main",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let result = stream.next().await.unwrap().unwrap();
    assert!(result.path.ends_with("main.rs"));
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn test_search_all_async_with_runtime_collects() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

    let results = search_all_async_with_runtime(
        SearchRuntime::real_fs(),
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
async fn test_search_all_async_with_custom_runtime_collects_virtual_results() {
    let root = PathBuf::from("/virtual");
    let runtime = SearchRuntime::with_backend(Arc::new(FakeBackend::new(
        root.clone(),
        [
            (PathBuf::from("src/lib.rs"), b"fn helper() {}\n".to_vec()),
            (
                PathBuf::from("notes.txt"),
                b"hello from async backend\n".to_vec(),
            ),
        ],
    )));

    let results = search_all_async_with_runtime(
        runtime,
        "contains:hello",
        SearchOptions {
            root,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, PathBuf::from("notes.txt"));
    assert!(results[0].content.contains("async backend"));
    assert_eq!(
        results[0]
            .metadata
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.stable_token.as_deref()),
        Some("token:notes.txt")
    );
}

#[tokio::test]
async fn test_search_async_with_custom_runtime_emits_progress() {
    let root = PathBuf::from("/virtual");
    let runtime = SearchRuntime::with_backend(Arc::new(FakeBackend::new(
        root.clone(),
        [(PathBuf::from("notes.txt"), b"hello progress\n".to_vec())],
    )));
    let events = Arc::new(Mutex::new(Vec::new()));
    let captured = events.clone();

    let mut stream = search_async_with_runtime_and_progress(
        runtime,
        "contains:hello",
        SearchOptions {
            root,
            ..Default::default()
        },
        move |event| {
            captured.lock().unwrap().push(event);
        },
    )
    .await
    .unwrap();

    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.path, PathBuf::from("notes.txt"));
    assert!(stream.next().await.is_none());

    let events = events.lock().unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, rdump::contracts::ProgressEvent::Started { .. })));
    assert!(events.iter().any(|event| matches!(
        event,
        rdump::contracts::ProgressEvent::Phase { name, .. } if name == "materialize"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        rdump::contracts::ProgressEvent::Finished { returned_files, .. } if *returned_files == 1
    )));
}
