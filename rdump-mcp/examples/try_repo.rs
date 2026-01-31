use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use serde_json::{json, Value as JsonValue};
use turbomcp_client::Client;
use turbomcp_transport::{ChildProcessConfig, ChildProcessTransport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let root = resolve_repo_root()?;
    let client = initialized_client().await?;

    let cases = vec![
        SearchCase {
            label: "Docs lookup (summary)",
            query: "name:AGENTS.md | name:README.md",
            output: "summary",
            limits: json!({ "max_results": 5 }),
            context_lines: None,
            sanitize_content: true,
        },
        SearchCase {
            label: "Full AGENTS.md (full)",
            query: "name:AGENTS.md",
            output: "full",
            limits: json!({ "max_results": 1, "max_bytes_per_file": null, "max_total_bytes": null }),
            context_lines: None,
            sanitize_content: true,
        },
        SearchCase {
            label: "Validate query mentions (snippets)",
            query: "contains:validate_query",
            output: "snippets",
            limits: json!({ "max_results": 2, "max_snippet_bytes": 160 }),
            context_lines: Some(1),
            sanitize_content: true,
        },
        SearchCase {
            label: "Rdump MCP rust files (paths)",
            query: "path:rdump-mcp & ext:rs",
            output: "paths",
            limits: json!({ "max_results": 5 }),
            context_lines: None,
            sanitize_content: true,
        },
    ];

    for case in cases {
        let mut args = HashMap::new();
        args.insert("query".to_string(), json!(case.query));
        args.insert("root".to_string(), json!(root.to_string_lossy().to_string()));
        args.insert("output".to_string(), json!(case.output));
        args.insert("limits".to_string(), case.limits.clone());
        if let Some(lines) = case.context_lines {
            args.insert("context_lines".to_string(), json!(lines));
        }

        let result = client.call_tool("search", Some(args)).await?;
        let structured = result
            .structured_content
            .ok_or("missing structured content")?;

        println!("\n== {} ==", case.label);
        println!("Query: {}", case.query);
        println!("Response:");
        let displayed = if case.sanitize_content {
            sanitize_full_content(&structured, 400)
        } else {
            structured
        };
        println!("{}", pretty_json(&displayed));
    }

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

fn resolve_repo_root() -> Result<PathBuf, Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .ok_or("failed to resolve repo root")?
        .to_path_buf();
    Ok(root)
}

struct SearchCase {
    label: &'static str,
    query: &'static str,
    output: &'static str,
    limits: JsonValue,
    context_lines: Option<usize>,
    sanitize_content: bool,
}

fn pretty_json(value: &JsonValue) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

fn sanitize_full_content(value: &JsonValue, preview_len: usize) -> JsonValue {
    let mut cloned = value.clone();
    let results = cloned
        .get_mut("results")
        .and_then(|v| v.as_array_mut());

    if let Some(items) = results {
        for item in items.iter_mut() {
            let kind = item.get("kind").and_then(|v| v.as_str());
            if kind != Some("full") {
                continue;
            }
            if let Some(content) = item.get("content").and_then(|v| v.as_str()) {
                let length = content.len();
                let preview = content.chars().take(preview_len).collect::<String>();
                let obj = item.as_object_mut();
                if let Some(obj) = obj {
                    obj.insert("content_length".to_string(), json!(length));
                    obj.insert("content_preview".to_string(), json!(preview));
                    obj.insert("content_preview_truncated".to_string(), json!(length > preview_len));
                    obj.remove("content");
                }
            }
        }
    }

    cloned
}
