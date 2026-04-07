use rdump::contracts::ProgressEvent;
use rdump_mcp::search::run_search_with_runtime_and_cancellation_and_progress;
use rdump_mcp::types::{OutputMode, SearchRequest};
use tempfile::tempdir;

#[test]
fn runtime_aware_search_helper_uses_supplied_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    std::fs::write(dir.path().join("hello.txt"), "hello world")?;

    let request = SearchRequest {
        query: "contains:hello".to_string(),
        root: Some(dir.path().to_string_lossy().to_string()),
        output: Some(OutputMode::Paths),
        ..Default::default()
    };

    let runtime = rdump::SearchRuntime::real_fs();
    let cancellation = rdump::SearchCancellationToken::new();
    let mut phases = Vec::new();
    let response = run_search_with_runtime_and_cancellation_and_progress(
        request,
        runtime,
        cancellation,
        |event| match event {
            ProgressEvent::Started { .. } => phases.push("started".to_string()),
            ProgressEvent::Phase { name, .. } => phases.push(name.clone()),
            ProgressEvent::Finished { .. } => phases.push("finished".to_string()),
            ProgressEvent::Result { .. } => phases.push("result".to_string()),
        },
    )?;

    assert_eq!(response.results.len(), 1);
    let item = response.results.first().expect("path result");
    let rdump::contracts::SearchItem::Path { path, metadata, .. } = item else {
        panic!("expected path result");
    };
    assert!(path.ends_with("hello.txt"));
    assert!(metadata.size_bytes > 0);
    assert!(phases.iter().any(|phase| phase == "started"));
    assert!(phases.iter().any(|phase| phase == "finished"));

    Ok(())
}
