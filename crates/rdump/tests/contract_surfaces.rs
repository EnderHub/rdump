use anyhow::{anyhow, Result};
use rdump::backend::{
    BackendFileType, BackendMetadata, BackendPathIdentity, DiscoveryReport, DiscoveryRequest,
    SearchBackend,
};
use rdump::contracts::{LimitValue, Limits, OutputMode, SearchItem, SearchRequest};
use rdump::{
    execute_search_request, execute_search_request_with_runtime, search, search_iter,
    PathResolution, SearchOptions, SearchRuntime,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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

    fn relative_key<'a>(&'a self, path: &'a Path) -> Result<PathBuf> {
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
                resolution: PathResolution::Canonical,
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
            resolution: PathResolution::Canonical,
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

#[test]
fn custom_backend_runtime_search_materializes_without_host_filesystem() {
    let root = PathBuf::from("/virtual");
    let runtime = SearchRuntime::with_backend(Arc::new(FakeBackend::new(
        root.clone(),
        [
            (
                PathBuf::from("src/lib.rs"),
                b"fn virtual_main() {}\n".to_vec(),
            ),
            (
                PathBuf::from("notes.txt"),
                b"hello from virtual backend\n".to_vec(),
            ),
        ],
    )));

    let results = runtime
        .search(
            "contains:hello",
            &SearchOptions {
                root,
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, PathBuf::from("notes.txt"));
    assert!(results[0].content.contains("virtual backend"));
    assert_eq!(
        results[0]
            .metadata
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.stable_token.as_deref()),
        Some("token:notes.txt")
    );
}

#[test]
fn request_runtime_search_uses_backend_path_metadata_without_host_filesystem() {
    let root = PathBuf::from("/virtual");
    let runtime = SearchRuntime::with_backend(Arc::new(FakeBackend::new(
        root.clone(),
        [(
            PathBuf::from("src/lib.rs"),
            b"fn virtual_main() {}\n".to_vec(),
        )],
    )));

    let response = execute_search_request_with_runtime(
        runtime,
        &SearchRequest {
            query: "ext:rs".to_string(),
            root: Some(root.display().to_string()),
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
        },
    )
    .unwrap();

    assert_eq!(response.results.len(), 1);
    match &response.results[0] {
        SearchItem::Path {
            path,
            file,
            fingerprint,
            metadata,
            ..
        } => {
            assert_eq!(path, "src/lib.rs");
            assert_eq!(file.resolved_path, "/virtual/src/lib.rs");
            assert_eq!(file.root_relative_path.as_deref(), Some("src/lib.rs"));
            assert_eq!(fingerprint, "token:src/lib.rs");
            assert_eq!(metadata.size_bytes, "fn virtual_main() {}\n".len() as u64);
            assert_eq!(metadata.permissions_display, "-rw-r--r--");
        }
        other => panic!("expected path item, got {other:?}"),
    }
}
