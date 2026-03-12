use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;

use serde_json::{json, Value as JsonValue};
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::{timeout, Duration};
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

    let result = client.call_tool("search", Some(args), None).await?;
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

    let result = client.call_tool("list_languages", None, None).await?;
    assert_ne!(result.is_error, Some(true));
    assert!(!result.all_text().is_empty());
    assert!(result.structured_content.is_some());

    let mut args = HashMap::new();
    args.insert("language".to_string(), json!("rust"));
    let result = client
        .call_tool("describe_language", Some(args), None)
        .await?;
    assert_ne!(result.is_error, Some(true));
    assert!(!result.all_text().is_empty());
    assert!(result.structured_content.is_some());

    Ok(())
}

#[tokio::test]
async fn e2e_reference_tools_and_resources_over_stdio() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;

    let result = client.call_tool("rql_reference", None, None).await?;
    assert_ne!(result.is_error, Some(true));
    assert!(!result.all_text().is_empty());
    assert!(result.structured_content.is_some());

    let result = client.call_tool("sdk_reference", None, None).await?;
    assert_ne!(result.is_error, Some(true));
    assert!(!result.all_text().is_empty());
    assert!(result.structured_content.is_some());

    let resources = client.list_resources().await?;
    let uris: Vec<String> = resources.iter().map(|r| r.uri.clone()).collect();
    assert!(uris.iter().any(|uri| uri == "rdump://docs/rql"));
    assert!(uris.iter().any(|uri| uri == "rdump://docs/sdk"));
    assert!(uris.iter().any(|uri| uri == "rdump://docs/languages"));
    assert!(uris.iter().any(|uri| uri == "rdump://docs/examples"));
    assert!(uris.iter().any(|uri| uri == "rdump://docs/runtime"));
    assert!(uris.iter().any(|uri| uri == "rdump://docs/stdio"));
    assert!(uris.iter().any(|uri| uri == "rdump://docs/stability"));
    assert!(uris.iter().any(|uri| uri == "rdump://docs/capabilities"));
    assert!(uris.iter().any(|uri| uri == "rdump://docs/predicates"));
    assert!(uris.iter().any(|uri| uri == "rdump://docs/language-matrix"));
    assert!(uris.iter().any(|uri| uri == "rdump://config/active"));
    assert!(uris.iter().any(|uri| uri == "rdump://config/presets"));

    let rql_resource = client.read_resource("rdump://docs/rql").await?;
    let rql_text = collect_resource_text(&rql_resource.contents);
    assert!(rql_text.contains("RQL"));

    let language_resource = client.read_resource("rdump://docs/languages").await?;
    let language_text = collect_resource_text(&language_resource.contents);
    assert!(language_text.contains("Supported languages"));

    let capability_resource = client.read_resource("rdump://docs/capabilities").await?;
    let capability_text = collect_resource_text(&capability_resource.contents);
    assert!(capability_text.contains("schema_version"));

    let predicate_resource = client.read_resource("rdump://docs/predicates").await?;
    let predicate_text = collect_resource_text(&predicate_resource.contents);
    assert!(predicate_text.contains("contains"));

    let matrix_resource = client.read_resource("rdump://docs/language-matrix").await?;
    let matrix_text = collect_resource_text(&matrix_resource.contents);
    assert!(matrix_text.contains("support_tier"));

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
        "explain_query",
        "capability_metadata",
        "predicate_catalog",
        "language_matrix",
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

    let result = client.call_tool("search", Some(args), None).await?;
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

    let result = client.call_tool("search", Some(args), None).await?;
    let structured = extract_structured_content(&result)?;
    assert_eq!(
        structured.get("truncated").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        structured.get("truncation_reason").and_then(|v| v.as_str()),
        Some("max_results")
    );

    Ok(())
}

#[tokio::test]
async fn e2e_search_continuation_token_pages_without_rerun_from_client(
) -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;
    let dir = tempdir()?;
    fs::write(dir.path().join("one.txt"), "hello one")?;
    fs::write(dir.path().join("two.txt"), "hello two")?;
    fs::write(dir.path().join("three.txt"), "hello three")?;

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

    let first = client.call_tool("search", Some(args), None).await?;
    let first_structured = extract_structured_content(&first)?;
    let token = first_structured
        .get("continuation_token")
        .and_then(|value| value.as_str())
        .expect("first page should contain continuation token");

    let mut continuation_args = HashMap::new();
    continuation_args.insert("continuation_token".to_string(), json!(token));
    let second = client
        .call_tool("search", Some(continuation_args), None)
        .await?;
    let second_structured = extract_structured_content(&second)?;
    assert_eq!(
        second_structured
            .get("status")
            .and_then(|value| value.as_str()),
        Some("truncated_success")
    );
    assert!(second_structured.get("results").is_some());
    assert!(second_structured.get("continuation_token").is_some());

    Ok(())
}

#[tokio::test]
async fn e2e_search_invalid_query_returns_error() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;
    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("invalid((syntax"));
    args.insert("output".to_string(), json!("paths"));
    let result = client.call_tool("search", Some(args), None).await?;
    assert_eq!(result.is_error, Some(true));
    let structured = extract_structured_content(&result)?;
    assert_eq!(
        structured.get("status").and_then(|value| value.as_str()),
        Some("invalid_query")
    );
    assert!(structured.get("error").is_some());
    assert!(structured
        .get("error")
        .and_then(|value| value.get("code"))
        .is_some());
    Ok(())
}

#[tokio::test]
async fn e2e_validate_query_tool() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;

    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("contains:hello"));
    let result = client.call_tool("validate_query", Some(args), None).await?;
    assert_ne!(result.is_error, Some(true));
    let structured = extract_structured_content(&result)?;
    assert_eq!(
        structured.get("valid").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        structured
            .get("schema_version")
            .and_then(|value| value.as_str()),
        Some("rdump.v1")
    );

    Ok(())
}

#[tokio::test]
async fn e2e_explain_query_tool() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;

    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("ext:rs & func:main"));
    let result = client.call_tool("explain_query", Some(args), None).await?;
    assert_ne!(result.is_error, Some(true));
    let structured = extract_structured_content(&result)?;
    assert_eq!(
        structured
            .get("effective_query")
            .and_then(|value| value.as_str()),
        Some("ext:rs & func:main")
    );
    assert!(structured.get("stages").is_some());

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

    let result = client.call_tool("search", Some(args), None).await?;
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

#[tokio::test]
async fn e2e_search_snippets_preserve_line_endings() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;
    let dir = tempdir()?;
    fs::write(
        dir.path().join("hello.txt"),
        b"line 1\r\nhello world\r\nline 3\r\n",
    )?;

    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("contains:hello"));
    args.insert(
        "root".to_string(),
        json!(dir.path().to_string_lossy().to_string()),
    );
    args.insert("output".to_string(), json!("snippets"));

    let result = client.call_tool("search", Some(args), None).await?;
    let structured = extract_structured_content(&result)?;
    let snippets = structured
        .get("results")
        .and_then(|value| value.as_array())
        .and_then(|results| results.first())
        .and_then(|first| first.get("snippets"))
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(!snippets.is_empty());
    assert_eq!(
        snippets[0]
            .get("line_ending")
            .and_then(|value| value.as_str()),
        Some("crlf")
    );
    Ok(())
}

#[tokio::test]
async fn e2e_search_repeated_requests_soak() -> Result<(), Box<dyn Error>> {
    let client = initialized_client().await?;
    let dir = tempdir()?;
    for index in 0..8 {
        fs::write(
            dir.path().join(format!("file{index}.txt")),
            format!("hello from file {index}"),
        )?;
    }

    for _ in 0..12 {
        let mut args = HashMap::new();
        args.insert("query".to_string(), json!("contains:hello"));
        args.insert(
            "root".to_string(),
            json!(dir.path().to_string_lossy().to_string()),
        );
        args.insert("output".to_string(), json!("summary"));
        let result = client.call_tool("search", Some(args), None).await?;
        let structured = extract_structured_content(&result)?;
        assert_eq!(
            structured
                .get("schema_version")
                .and_then(|value| value.as_str()),
            Some("rdump.v1")
        );
    }

    Ok(())
}

#[tokio::test]
async fn e2e_search_emits_progress_notifications_when_progress_token_present(
) -> Result<(), Box<dyn Error>> {
    let dir = tempdir()?;
    for index in 0..24 {
        fs::write(
            dir.path().join(format!("file{index}.txt")),
            format!("hello from file {index}"),
        )?;
    }

    let mut child = spawn_raw_server()?;
    let mut stdin = child.stdin.take().ok_or("missing stdin pipe")?;
    let stdout = child.stdout.take().ok_or("missing stdout pipe")?;
    let _stderr = child.stderr.take().ok_or("missing stderr pipe")?;
    let mut reader = BufReader::new(stdout);

    write_json_line(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "clientInfo": {
                    "name": "rdump-progress-test",
                    "version": "1.0.0"
                },
                "capabilities": {}
            }
        }),
    )
    .await?;

    let init = read_json_line(&mut reader).await?;
    assert_eq!(init.get("id"), Some(&json!(1)));
    assert!(init.get("result").is_some());

    write_json_line(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }),
    )
    .await?;

    write_json_line(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "search",
                "arguments": {
                    "query": "contains:hello",
                    "root": dir.path().to_string_lossy().to_string(),
                    "output": "summary"
                },
                "_meta": {
                    "progressToken": "search-progress-1"
                }
            }
        }),
    )
    .await?;

    let mut saw_progress = false;
    let mut final_response = None;
    for _ in 0..32 {
        let message = read_json_line(&mut reader).await?;
        if message.get("method").and_then(|value| value.as_str()) == Some("notifications/progress")
        {
            saw_progress = true;
            assert_eq!(
                message
                    .get("params")
                    .and_then(|value| value.get("progressToken"))
                    .and_then(|value| value.as_str()),
                Some("search-progress-1")
            );
            continue;
        }

        if message.get("id") == Some(&json!(2)) {
            final_response = Some(message);
            break;
        }
    }

    assert!(
        saw_progress,
        "expected at least one MCP progress notification"
    );
    let final_response = final_response.ok_or("missing final tools/call response")?;
    assert!(final_response.get("result").is_some());

    child.kill().await.ok();
    let _ = child.wait().await;
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

fn spawn_raw_server() -> Result<Child, Box<dyn Error>> {
    let bin = locate_server_binary()?;
    let child = Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    Ok(child)
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

async fn write_json_line(stdin: &mut ChildStdin, value: &JsonValue) -> Result<(), Box<dyn Error>> {
    let line = serde_json::to_string(value)?;
    stdin.write_all(line.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;
    Ok(())
}

async fn read_json_line(reader: &mut BufReader<ChildStdout>) -> Result<JsonValue, Box<dyn Error>> {
    loop {
        let mut line = String::new();
        let bytes_read = timeout(Duration::from_secs(10), reader.read_line(&mut line)).await??;
        if bytes_read == 0 {
            return Err("server closed stdout".into());
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        return Ok(serde_json::from_str(trimmed)?);
    }
}
