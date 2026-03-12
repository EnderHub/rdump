use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use turbomcp::prelude::{McpError, McpHandler, McpResult, RequestContext};
use turbomcp::CallToolRequest;
use turbomcp::JsonRpcNotification;
use turbomcp_server::transport::MAX_MESSAGE_SIZE;
use turbomcp_server::{parse_request, route_request, JsonRpcOutgoing};

use crate::RdumpServer;

pub async fn run(handler: RdumpServer) -> McpResult<()> {
    handler.on_initialize().await?;

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut writer = stdout;
    let (response_tx, mut response_rx) = mpsc::channel::<JsonRpcOutgoing>(32);
    let (notification_tx, mut notification_rx) = mpsc::unbounded_channel::<JsonRpcNotification>();

    let result = async {
        let mut line = String::new();
        loop {
            tokio::select! {
                biased;

                Some(notification) = notification_rx.recv() => {
                    write_json_line(&mut writer, &notification).await?;
                }

                Some(response) = response_rx.recv() => {
                    if response.should_send() {
                        write_json_line(&mut writer, &response).await?;
                    }
                }

                read = reader.read_line(&mut line) => {
                    let bytes_read = read.map_err(|err| {
                        McpError::internal(format!("Failed to read stdio line: {err}"))
                    })?;
                    if bytes_read == 0 {
                        break;
                    }

                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        line.clear();
                        continue;
                    }

                    if line.len() > MAX_MESSAGE_SIZE {
                        write_json_line(
                            &mut writer,
                            &JsonRpcOutgoing::error(
                                None,
                                McpError::invalid_request(format!(
                                    "Message exceeds maximum size of {MAX_MESSAGE_SIZE} bytes",
                                )),
                            ),
                        )
                        .await?;
                        line.clear();
                        continue;
                    }

                    match parse_request(trimmed) {
                        Ok(request) => {
                            let handler = handler.clone();
                            let response_tx = response_tx.clone();
                            let notification_tx = notification_tx.clone();
                            let request_id = request.id.as_ref().map(request_id_key);
                            let progress_token = progress_token(&request);
                            let ctx = request_context(&request);

                            tokio::spawn(async move {
                                if let (Some(request_id), Some(progress_token)) =
                                    (request_id.as_ref(), progress_token.as_ref())
                                {
                                    handler.register_progress_sink(
                                        request_id.clone(),
                                        progress_token.clone(),
                                        notification_tx,
                                    );
                                }

                                let response = route_request(&handler, request, &ctx).await;

                                if let Some(request_id) = request_id.as_deref() {
                                    handler.remove_progress_sink(request_id);
                                }

                                let _ = response_tx.send(response).await;
                            });
                        }
                        Err(err) => {
                            write_json_line(&mut writer, &JsonRpcOutgoing::error(None, err)).await?;
                        }
                    }

                    line.clear();
                }
            }
        }

        Ok::<(), McpError>(())
    }
    .await;

    handler.on_shutdown().await?;
    result
}

fn request_context(request: &turbomcp_server::JsonRpcIncoming) -> RequestContext {
    let mut ctx = RequestContext::stdio();
    ctx.request_id = request
        .id
        .as_ref()
        .map(request_id_key)
        .unwrap_or_else(|| "notification".to_string());
    ctx
}

fn request_id_key(id: &serde_json::Value) -> String {
    match id {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Number(value) => value.to_string(),
        other => other.to_string(),
    }
}

fn progress_token(request: &turbomcp_server::JsonRpcIncoming) -> Option<String> {
    if request.method != "tools/call" {
        return None;
    }

    let params = request.params.as_ref()?;
    let call = serde_json::from_value::<CallToolRequest>(params.clone()).ok()?;
    if call.name != "search" {
        return None;
    }

    match call._meta?.get("progressToken")? {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

async fn write_json_line<W, T>(writer: &mut W, value: &T) -> McpResult<()>
where
    W: AsyncWrite + Unpin,
    T: serde::Serialize,
{
    let line = serde_json::to_string(value)
        .map_err(|err| McpError::internal(format!("Failed to serialize stdio payload: {err}")))?;
    writer
        .write_all(line.as_bytes())
        .await
        .map_err(|err| McpError::internal(format!("Failed to write stdio payload: {err}")))?;
    writer
        .write_all(b"\n")
        .await
        .map_err(|err| McpError::internal(format!("Failed to write stdio newline: {err}")))?;
    writer
        .flush()
        .await
        .map_err(|err| McpError::internal(format!("Failed to flush stdio payload: {err}")))?;
    Ok(())
}
