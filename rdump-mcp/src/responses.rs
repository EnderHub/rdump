use serde::Serialize;
use turbomcp::prelude::{McpError, McpResult};
use turbomcp::{CallToolResult, Content, TextContent};

pub fn tool_result<T: Serialize>(value: &T, text: String) -> McpResult<CallToolResult> {
    let structured = serde_json::to_value(value).map_err(McpError::Serialization)?;
    Ok(CallToolResult {
        content: vec![Content::Text(TextContent {
            text,
            annotations: None,
            meta: None,
        })],
        is_error: Some(false),
        structured_content: Some(structured),
        _meta: None,
        task_id: None,
    })
}
