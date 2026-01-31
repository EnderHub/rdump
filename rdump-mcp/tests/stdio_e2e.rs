use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value as JsonValue};
use tempfile::tempdir;
use turbomcp_client::Client;
use turbomcp_protocol::types::ResourceContent;
use turbomcp_transport::{ChildProcessConfig, ChildProcessTransport};

#[tokio::test]
async fn e2e_search_tool_over_child_process_stdio() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;
    let dir = tempdir()?;
    let file_path = dir.path().join("hello.txt");
    fs::write(&file_path, "hello world")?;
    let tools = client.list_tools().await?;
    assert!(tools.iter().any(|tool| tool.name == "search"));

    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("contains:hello"));
    args.insert(
        "root".to_string(),
        json!(dir.path().to_string_lossy().to_string()),
    );
    args.insert("output".to_string(), json!("paths"));

    let result = client.call_tool("search", Some(args)).await?;
    assert_ne!(result.is_error, Some(true));
    assert!(!result.all_text().is_empty());
    let value = extract_structured_content(&result)?;
    let results = value
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(!results.is_empty());

    Ok(())
}

#[tokio::test]
async fn e2e_language_tools_over_stdio() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;

    let result = client.call_tool("list_languages", None).await?;
    assert_ne!(result.is_error, Some(true));
    assert!(!result.all_text().is_empty());
    assert!(result.structured_content.is_some());

    let mut args = HashMap::new();
    args.insert("language".to_string(), json!("rust"));
    let result = client.call_tool("describe_language", Some(args)).await?;
    assert_ne!(result.is_error, Some(true));
    assert!(!result.all_text().is_empty());
    assert!(result.structured_content.is_some());

    Ok(())
}

#[tokio::test]
async fn e2e_reference_tools_and_resources_over_stdio() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;

    let result = client.call_tool("rql_reference", None).await?;
    assert_ne!(result.is_error, Some(true));
    assert!(!result.all_text().is_empty());
    assert!(result.structured_content.is_some());

    let result = client.call_tool("sdk_reference", None).await?;
    assert_ne!(result.is_error, Some(true));
    assert!(!result.all_text().is_empty());
    assert!(result.structured_content.is_some());

    let resources = client.list_resources().await?;
    let uris: Vec<String> = resources.iter().map(|r| r.uri.clone()).collect();
    assert!(uris.iter().any(|uri| uri == "rdump://docs/rql"));
    assert!(uris.iter().any(|uri| uri == "rdump://docs/sdk"));

    let rql_resource = client.read_resource("rdump://docs/rql").await?;
    let rql_text = collect_resource_text(&rql_resource.contents);
    assert!(rql_text.contains("RQL"));

    Ok(())
}

#[tokio::test]
async fn e2e_list_tools_contains_expected() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;
    let tools = client.list_tools().await?;
    let names: Vec<&str> = tools.iter().map(|tool| tool.name.as_str()).collect();

    for expected in [
        "search",
        "list_languages",
        "describe_language",
        "rql_reference",
        "sdk_reference",
        "validate_query",
    ] {
        assert!(names.contains(&expected));
    }

    Ok(())
}

#[tokio::test]
async fn e2e_search_full_output_truncates_content() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;
    let dir = tempdir()?;
    let file_path = dir.path().join("long.txt");
    fs::write(&file_path, "hello world hello world")?;

    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("contains:hello"));
    args.insert(
        "root".to_string(),
        json!(dir.path().to_string_lossy().to_string()),
    );
    args.insert("output".to_string(), json!("full"));
    args.insert(
        "limits".to_string(),
        json!({
            "max_bytes_per_file": 5
        }),
    );

    let result = client.call_tool("search", Some(args)).await?;
    let structured = extract_structured_content(&result)?;
    let results = structured
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(!results.is_empty());
    let first = &results[0];
    assert_eq!(first.get("kind").and_then(|v| v.as_str()), Some("full"));
    assert_eq!(
        first.get("content_truncated").and_then(|v| v.as_bool()),
        Some(true)
    );

    Ok(())
}

#[tokio::test]
async fn e2e_search_max_results_truncates() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;
    let dir = tempdir()?;
    fs::write(dir.path().join("one.txt"), "hello one")?;
    fs::write(dir.path().join("two.txt"), "hello two")?;

    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("contains:hello"));
    args.insert(
        "root".to_string(),
        json!(dir.path().to_string_lossy().to_string()),
    );
    args.insert("output".to_string(), json!("paths"));
    args.insert(
        "limits".to_string(),
        json!({
            "max_results": 1
        }),
    );

    let result = client.call_tool("search", Some(args)).await?;
    let structured = extract_structured_content(&result)?;
    assert_eq!(structured.get("truncated").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(
        structured
            .get("truncation_reason")
            .and_then(|v| v.as_str()),
        Some("max_results")
    );

    Ok(())
}

#[tokio::test]
async fn e2e_search_invalid_query_returns_error() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;
    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("invalid((syntax"));
    args.insert("output".to_string(), json!("paths"));
    let result = client.call_tool("search", Some(args)).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn e2e_validate_query_tool() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;

    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("contains:hello"));
    let result = client.call_tool("validate_query", Some(args)).await?;
    assert_ne!(result.is_error, Some(true));
    let structured = extract_structured_content(&result)?;
    assert_eq!(structured.get("valid").and_then(|v| v.as_bool()), Some(true));

    Ok(())
}

#[tokio::test]
async fn e2e_search_summary_output() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;
    let dir = tempdir()?;
    fs::write(dir.path().join("hello.txt"), "hello world")?;

    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("contains:hello"));
    args.insert(
        "root".to_string(),
        json!(dir.path().to_string_lossy().to_string()),
    );
    args.insert("output".to_string(), json!("summary"));

    let result = client.call_tool("search", Some(args)).await?;
    assert_ne!(result.is_error, Some(true));
    let structured = extract_structured_content(&result)?;
    let results = structured
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(!results.is_empty());
    let first = &results[0];
    assert_eq!(first.get("kind").and_then(|v| v.as_str()), Some("summary"));

    Ok(())
}

async fn initialized_client() -> Result<Client<ChildProcessTransport>, Box<dyn Error>> {
    let bin = locate_server_binary()?;
    let config = ChildProcessConfig {
        command: bin,
        args: Vec::new(),
        working_directory: None,
        environment: None,
        ..Default::default()
    };

    let transport = ChildProcessTransport::new(config);
    let client = Client::new(transport);
    let _init = client.initialize().await?;
    Ok(client)
}

fn locate_server_binary() -> Result<String, Box<dyn Error>> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_rdump-mcp") {
        return Ok(path);
    }

    let exe = std::env::current_exe()?;
    let target_dir = exe
        .parent()
        .and_then(|p| p.parent())
        .ok_or("failed to resolve target dir")?;
    let mut bin_path = PathBuf::from(target_dir);
    bin_path.push("rdump-mcp");
    if cfg!(windows) {
        bin_path.set_extension("exe");
    }
    Ok(bin_path.to_string_lossy().to_string())
}

fn collect_resource_text(contents: &[ResourceContent]) -> String {
    let mut text = String::new();
    for content in contents {
        if let ResourceContent::Text(inner) = content {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&inner.text);
        }
    }
    text
}

fn extract_structured_content(
    result: &turbomcp_protocol::types::CallToolResult,
) -> Result<JsonValue, Box<dyn Error>> {
    if let Some(value) = result.structured_content.clone() {
        Ok(value)
    } else {
        Err("missing structured content".into())
    }
}
