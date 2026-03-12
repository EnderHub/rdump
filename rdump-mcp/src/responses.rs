use serde::Serialize;
use turbomcp::prelude::{McpError, McpResult, ToolResult};

use crate::types::{ContractError, ErrorEnvelope};

pub fn tool_result<T: Serialize>(value: &T, text: String) -> McpResult<ToolResult> {
    let structured =
        serde_json::to_value(value).map_err(|err| McpError::serialization(err.to_string()))?;
    let mut result = ToolResult::text(text);
    result.is_error = Some(false);
    result.structured_content = Some(structured);
    Ok(result)
}

pub fn tool_error_result(error: &ContractError, text: String) -> McpResult<ToolResult> {
    let envelope = ErrorEnvelope {
        schema_version: crate::types::SCHEMA_VERSION.to_string(),
        status: match error.code {
            crate::types::ErrorCode::InvalidRequest
            | crate::types::ErrorCode::QuerySyntax
            | crate::types::ErrorCode::QueryValidation => {
                rdump::contracts::SearchStatus::InvalidQuery
            }
            _ => rdump::contracts::SearchStatus::PartialSuccess,
        },
        error: error.clone(),
    };
    let structured =
        serde_json::to_value(envelope).map_err(|err| McpError::serialization(err.to_string()))?;
    let mut result = ToolResult::text(text);
    result.is_error = Some(true);
    result.structured_content = Some(structured);
    Ok(result)
}
