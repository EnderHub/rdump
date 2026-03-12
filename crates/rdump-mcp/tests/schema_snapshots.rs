use rdump_mcp::responses::tool_error_result;

#[test]
fn mcp_error_envelope_snapshot_matches() -> Result<(), Box<dyn std::error::Error>> {
    let error = rdump::request::contract_error(
        rdump::contracts::ErrorCode::QueryValidation,
        "Invalid query syntax: expected value".to_string(),
        Some("query".to_string()),
        false,
        Some("Use `rdump query explain` to inspect the normalized query.".to_string()),
    );
    let result = tool_error_result(&error, "Query is invalid".to_string())?;
    let structured = result
        .structured_content
        .expect("tool error should include structured content");
    let rendered = serde_json::to_string_pretty(&structured)? + "\n";
    assert_eq!(
        rendered,
        include_str!("../../../docs/generated/mcp-error-envelope.snapshot.json")
    );
    Ok(())
}
