pub mod docs;
pub mod languages;
pub mod limits;
pub mod responses;
pub mod search;
pub mod stdio;
pub mod types;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::{mpsc::UnboundedSender, Semaphore};
use turbomcp::prelude::*;
use turbomcp::JsonRpcNotification;

use crate::responses::{tool_error_result, tool_result};
use crate::search::{
    build_search_request, format_search_response_text, run_search_with_cancellation_and_progress,
};
use crate::types::{
    DescribeLanguageArgs, ExplainQueryArgs, LimitValue, SearchArgs, ValidateQueryArgs,
    ValidateQueryResponse,
};

static SEARCH_SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);
const SESSION_TOKEN_PREFIX: &str = "session";
const SESSION_TOKEN_VERSION: &str = "v1";

#[derive(Clone)]
struct CachedSearchSession {
    response: rdump::contracts::SearchResponse,
    page_size: usize,
    byte_budget: usize,
    created_at: Instant,
    last_accessed: Instant,
    approx_bytes: usize,
}

#[derive(Default)]
struct SessionCacheMetrics {
    created: AtomicU64,
    evicted_ttl: AtomicU64,
    evicted_capacity: AtomicU64,
    invalid_tokens: AtomicU64,
}

#[derive(Clone)]
struct SearchProgressSink {
    progress_token: String,
    notifications_tx: UnboundedSender<JsonRpcNotification>,
}

#[derive(Clone)]
struct SearchProgressEmitter {
    progress_token: String,
    notifications_tx: UnboundedSender<JsonRpcNotification>,
}

#[derive(Clone)]
pub struct RdumpServer {
    search_semaphore: Arc<Semaphore>,
    search_sessions: Arc<Mutex<HashMap<String, CachedSearchSession>>>,
    progress_sinks: Arc<Mutex<HashMap<String, SearchProgressSink>>>,
    session_metrics: Arc<SessionCacheMetrics>,
    session_ttl: Duration,
    max_cached_sessions: usize,
}

impl Default for RdumpServer {
    fn default() -> Self {
        Self {
            search_semaphore: Arc::new(Semaphore::new(default_max_concurrent_searches())),
            search_sessions: Arc::new(Mutex::new(HashMap::new())),
            progress_sinks: Arc::new(Mutex::new(HashMap::new())),
            session_metrics: Arc::new(SessionCacheMetrics::default()),
            session_ttl: default_session_ttl(),
            max_cached_sessions: default_max_cached_sessions(),
        }
    }
}

impl RdumpServer {
    async fn search_tool(
        &self,
        args: SearchArgs,
        request_id: Option<&str>,
    ) -> McpResult<ToolResult> {
        if let Some(token) = args.continuation_token.as_deref() {
            match parse_session_token(token) {
                Ok((session_id, offset)) => return self.search_session_page(&session_id, offset),
                Err(message) => {
                    self.session_metrics
                        .invalid_tokens
                        .fetch_add(1, Ordering::Relaxed);
                    return Err(McpError::invalid_request(message));
                }
            }
        }

        let resolved_limits = crate::limits::resolve_limits(args.limits.clone());
        let page_size = resolved_limits.max_results;
        let byte_budget = resolved_limits.max_total_bytes;
        let request = match build_search_request(args) {
            Ok(request) => request,
            Err(err) => {
                let error = rdump::request::contract_error(
                    crate::types::ErrorCode::InvalidRequest,
                    err.to_string(),
                    None,
                    false,
                    Some("Provide a non-empty query or at least one preset.".to_string()),
                );
                return tool_error_result(
                    &error,
                    format!("Invalid search request: {}", error.message),
                );
            }
        };
        let queue_wait_started = Instant::now();
        self.prune_expired_sessions();
        let permit = self
            .search_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| McpError::unavailable("Search limiter is closed"))?;
        let cancellation = rdump::SearchCancellationToken::new();
        let task_cancellation = cancellation.clone();
        let mut cancel_on_drop = rdump::CancelOnDrop::new(cancellation);
        let progress = request_id.and_then(|request_id| self.progress_emitter(request_id));

        let response = tokio::task::spawn_blocking(
            move || -> McpResult<(rdump::contracts::SearchResponse, Option<CachedSearchSession>)> {
            let _permit = permit;
            let page_response = run_search_with_cancellation_and_progress(
                request.clone(),
                task_cancellation.clone(),
                |event| {
                    if let Some(progress) = progress.as_ref() {
                        progress.emit(event);
                    }
                },
            )?;
            if !page_response.truncated {
                return Ok((page_response, None));
            }

            let mut full_request = request;
            full_request.offset = 0;
            full_request.continuation_token = None;
            let mut limits = full_request.limits.unwrap_or_default();
            limits.max_results = LimitValue::Unlimited;
            limits.max_total_bytes = LimitValue::Unlimited;
            full_request.limits = Some(limits);

            let full_response = run_search_with_cancellation_and_progress(
                full_request,
                task_cancellation,
                |event| {
                    if let Some(progress) = progress.as_ref() {
                        progress.emit(event);
                    }
                },
            )?;
            let now = Instant::now();
                Ok((
                    page_response,
                    Some(CachedSearchSession {
                        approx_bytes: measure_serialized_bytes(&full_response),
                        response: full_response,
                        page_size,
                        byte_budget,
                        created_at: now,
                        last_accessed: now,
                    }),
                ))
            },
        )
        .await
        .map_err(|err| {
            rdump::request::contract_error(
                crate::types::ErrorCode::AsyncJoin,
                format!("Search task failed to join: {err}"),
                None,
                true,
                Some("Retry the request or lower concurrent search load.".to_string()),
            )
        });
        cancel_on_drop.disarm();

        match response {
            Ok(Ok((mut response, session))) => {
                response.stats.semaphore_wait_millis =
                    queue_wait_started.elapsed().as_millis() as u64;
                self.maybe_attach_queue_overload_diagnostic(&mut response);
                if let Some(session) = session {
                    let session_id = SEARCH_SESSION_COUNTER
                        .fetch_add(1, Ordering::Relaxed)
                        .to_string();
                    response.continuation_token = response
                        .next_offset
                        .map(|offset| format_session_token(&session_id, offset));
                    response.page_size = Some(session.page_size);
                    self.insert_session(session_id, session);
                }
                let text = format_search_response_text(&response);
                tool_result(&response, text)
            }
            Ok(Err(err)) => {
                let error = rdump::request::classify_error_message(&err.to_string());
                tool_error_result(&error, format!("Search failed: {}", error.message))
            }
            Err(error) => tool_error_result(&error, format!("Search failed: {}", error.message)),
        }
    }

    fn search_session_page(&self, session_id: &str, offset: usize) -> McpResult<ToolResult> {
        let cached = self.lookup_session(session_id)?;

        let mut response = paginate_cached_response(session_id, offset, &cached);
        response.stats.semaphore_wait_millis = 0;
        self.maybe_attach_queue_overload_diagnostic(&mut response);
        if response.next_offset.is_none() {
            self.search_sessions
                .lock()
                .expect("search session lock poisoned")
                .remove(session_id);
        }

        let text = format_search_response_text(&response);
        tool_result(&response, text)
    }

    pub(crate) fn register_progress_sink(
        &self,
        request_id: String,
        progress_token: String,
        notifications_tx: UnboundedSender<JsonRpcNotification>,
    ) {
        self.progress_sinks
            .lock()
            .expect("progress sink lock poisoned")
            .insert(
                request_id,
                SearchProgressSink {
                    progress_token,
                    notifications_tx,
                },
            );
    }

    pub(crate) fn remove_progress_sink(&self, request_id: &str) {
        self.progress_sinks
            .lock()
            .expect("progress sink lock poisoned")
            .remove(request_id);
    }

    fn progress_emitter(&self, request_id: &str) -> Option<SearchProgressEmitter> {
        self.progress_sinks
            .lock()
            .expect("progress sink lock poisoned")
            .get(request_id)
            .cloned()
            .map(|sink| SearchProgressEmitter {
                progress_token: sink.progress_token,
                notifications_tx: sink.notifications_tx,
            })
    }

    fn insert_session(&self, session_id: String, session: CachedSearchSession) {
        let mut guard = self
            .search_sessions
            .lock()
            .expect("search session lock poisoned");
        prune_expired_sessions_locked(&mut guard, self.session_ttl, &self.session_metrics);
        while guard.len() >= self.max_cached_sessions {
            if let Some(evicted_id) = least_recently_used_session_id(&guard) {
                guard.remove(&evicted_id);
                self.session_metrics
                    .evicted_capacity
                    .fetch_add(1, Ordering::Relaxed);
            } else {
                break;
            }
        }
        guard.insert(session_id, session);
        self.session_metrics.created.fetch_add(1, Ordering::Relaxed);
    }

    fn lookup_session(&self, session_id: &str) -> McpResult<CachedSearchSession> {
        let mut guard = self
            .search_sessions
            .lock()
            .expect("search session lock poisoned");
        prune_expired_sessions_locked(&mut guard, self.session_ttl, &self.session_metrics);
        let Some(cached) = guard.get_mut(session_id) else {
            return Err(McpError::invalid_request(format!(
                "Unknown continuation session `{session_id}`"
            )));
        };
        if cached.last_accessed.elapsed() > self.session_ttl {
            guard.remove(session_id);
            self.session_metrics
                .evicted_ttl
                .fetch_add(1, Ordering::Relaxed);
            return Err(McpError::invalid_request(format!(
                "Continuation session `{session_id}` expired after {}s of inactivity",
                self.session_ttl.as_secs()
            )));
        }
        cached.last_accessed = Instant::now();
        Ok(cached.clone())
    }

    fn prune_expired_sessions(&self) {
        let mut guard = self
            .search_sessions
            .lock()
            .expect("search session lock poisoned");
        prune_expired_sessions_locked(&mut guard, self.session_ttl, &self.session_metrics);
    }

    fn maybe_attach_queue_overload_diagnostic(
        &self,
        response: &mut rdump::contracts::SearchResponse,
    ) {
        let threshold = queue_overload_threshold_millis();
        if response.stats.semaphore_wait_millis < threshold {
            return;
        }
        response.diagnostics.push(rdump::contracts::SearchDiagnostic {
            level: "warn".to_string(),
            kind: "queue_overload".to_string(),
            message: format!(
                "Search waited {}ms behind the MCP concurrency guard (threshold={}ms). Consider lowering load or raising RDUMP_MCP_MAX_CONCURRENT_SEARCHES.",
                response.stats.semaphore_wait_millis,
                threshold
            ),
            path: None,
        });
    }

    fn session_cache_text(&self) -> String {
        let guard = self
            .search_sessions
            .lock()
            .expect("search session lock poisoned");
        let active_sessions = guard.len();
        let retained_bytes: usize = guard.values().map(|session| session.approx_bytes).sum();
        let oldest_session_age_seconds = guard
            .values()
            .map(|session| session.created_at.elapsed().as_secs())
            .max()
            .unwrap_or(0);
        drop(guard);

        format!(
            "rdump MCP session cache\nactive_sessions={active_sessions}\nretained_bytes={retained_bytes}\noldest_session_age_seconds={oldest_session_age_seconds}\nsession_ttl_seconds={}\nmax_cached_sessions={}\nsessions_created={}\nevicted_ttl={}\nevicted_capacity={}\ninvalid_tokens={}",
            self.session_ttl.as_secs(),
            self.max_cached_sessions,
            self.session_metrics.created.load(Ordering::Relaxed),
            self.session_metrics.evicted_ttl.load(Ordering::Relaxed),
            self.session_metrics.evicted_capacity.load(Ordering::Relaxed),
            self.session_metrics.invalid_tokens.load(Ordering::Relaxed),
        )
    }

    fn list_languages_tool(&self) -> McpResult<ToolResult> {
        let response = crate::languages::list_languages();
        let text = crate::languages::format_language_list_text(&response);
        tool_result(&response, text)
    }

    fn describe_language_tool(&self, args: DescribeLanguageArgs) -> McpResult<ToolResult> {
        match crate::languages::describe_language(&args.language) {
            Ok(response) => {
                let text = crate::languages::format_language_text(&response);
                tool_result(&response, text)
            }
            Err(err) => {
                let error = rdump::request::contract_error(
                    crate::types::ErrorCode::UnsupportedLanguage,
                    err.to_string(),
                    Some("language".to_string()),
                    false,
                    Some(
                        "Call list_languages to discover supported names and aliases.".to_string(),
                    ),
                );
                tool_error_result(&error, format!("Unknown language: {}", error.message))
            }
        }
    }

    fn rql_reference_tool(&self) -> McpResult<ToolResult> {
        let response = crate::docs::build_rql_reference();
        let text = crate::docs::format_rql_reference_text();
        tool_result(&response, text)
    }

    fn sdk_reference_tool(&self) -> McpResult<ToolResult> {
        let response = crate::docs::build_sdk_reference();
        let text = crate::docs::format_sdk_reference_text();
        tool_result(&response, text)
    }

    fn validate_query_tool(&self, args: ValidateQueryArgs) -> McpResult<ToolResult> {
        let response = match rdump::parser::parse_query(&args.query) {
            Ok(_) => {
                let explanation =
                    rdump::explain_query(&args.query, &rdump::SearchOptions::default()).ok();
                ValidateQueryResponse {
                    schema_version: crate::types::SCHEMA_VERSION.to_string(),
                    valid: true,
                    normalized_query: explanation
                        .as_ref()
                        .map(|explanation| explanation.effective_query.clone()),
                    warnings: explanation
                        .map(|explanation| explanation.notes)
                        .unwrap_or_default(),
                    errors: Vec::new(),
                }
            }
            Err(err) => ValidateQueryResponse {
                schema_version: crate::types::SCHEMA_VERSION.to_string(),
                valid: false,
                normalized_query: None,
                warnings: Vec::new(),
                errors: vec![rdump::request::contract_error(
                    crate::types::ErrorCode::QuerySyntax,
                    err.to_string(),
                    Some("query".to_string()),
                    false,
                    Some(
                        "Use explain_query to inspect the normalized query and stages.".to_string(),
                    ),
                )],
            },
        };

        let text = if response.valid {
            if response.warnings.is_empty() {
                "Query is valid.".to_string()
            } else {
                format!(
                    "Query is valid with {} warning(s).",
                    response.warnings.len()
                )
            }
        } else {
            format!(
                "Query is invalid: {}",
                response
                    .errors
                    .first()
                    .map(|error| error.message.as_str())
                    .unwrap_or("unknown error")
            )
        };

        if response.valid {
            tool_result(&response, text)
        } else {
            tool_error_result(response.errors.first().unwrap(), text)
        }
    }

    fn explain_query_tool(&self, args: ExplainQueryArgs) -> McpResult<ToolResult> {
        let options = rdump::SearchOptions {
            presets: args.presets,
            ..Default::default()
        };
        match rdump::explain_query(&args.query, &options) {
            Ok(response) => {
                let text = format!(
                    "Effective query: {}\nEstimated cost: {}\nStages: {}",
                    response.effective_query,
                    response.estimated_cost,
                    response
                        .stages
                        .iter()
                        .map(|stage| stage.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                tool_result(&response, text)
            }
            Err(err) => {
                let error = rdump::request::classify_error(&err);
                tool_error_result(&error, format!("Explain failed: {}", error.message))
            }
        }
    }
}

fn parse_session_token(token: &str) -> Result<(String, usize), String> {
    let mut parts = token.splitn(5, ':');
    let prefix = parts
        .next()
        .ok_or_else(|| "Continuation token is missing a prefix.".to_string())?;
    if prefix != SESSION_TOKEN_PREFIX {
        return Err(format!(
            "Unsupported continuation token prefix `{prefix}`; expected `{SESSION_TOKEN_PREFIX}`."
        ));
    }
    let version = parts
        .next()
        .ok_or_else(|| "Continuation token is missing a version.".to_string())?;
    if version != SESSION_TOKEN_VERSION {
        return Err(format!(
            "Continuation token version `{version}` is not supported by this server. Expected `{SESSION_TOKEN_VERSION}`."
        ));
    }
    let session_id = parts
        .next()
        .ok_or_else(|| "Continuation token is missing a session id.".to_string())?
        .to_string();
    let offset = parts
        .next()
        .ok_or_else(|| "Continuation token is missing an offset.".to_string())?
        .parse::<usize>()
        .map_err(|_| "Continuation token offset is not a valid integer.".to_string())?;
    let checksum = parts
        .next()
        .ok_or_else(|| "Continuation token is missing an integrity checksum.".to_string())?;
    let expected = session_token_checksum(&session_id, offset);
    let parsed = u64::from_str_radix(checksum, 16)
        .map_err(|_| "Continuation token checksum is malformed.".to_string())?;
    if parsed != expected {
        return Err(
            "Continuation token failed integrity validation. It may be stale, tampered, or from a different schema version."
                .to_string(),
        );
    }
    Ok((session_id, offset))
}

fn format_session_token(session_id: &str, offset: usize) -> String {
    format!(
        "{SESSION_TOKEN_PREFIX}:{SESSION_TOKEN_VERSION}:{session_id}:{offset}:{:016x}",
        session_token_checksum(session_id, offset)
    )
}

fn session_token_checksum(session_id: &str, offset: usize) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    rdump::contracts::SCHEMA_VERSION.hash(&mut hasher);
    SESSION_TOKEN_VERSION.hash(&mut hasher);
    session_id.hash(&mut hasher);
    offset.hash(&mut hasher);
    hasher.finish()
}

fn prune_expired_sessions_locked(
    sessions: &mut HashMap<String, CachedSearchSession>,
    session_ttl: Duration,
    metrics: &SessionCacheMetrics,
) {
    let expired: Vec<_> = sessions
        .iter()
        .filter_map(|(session_id, session)| {
            (session.last_accessed.elapsed() > session_ttl).then(|| session_id.clone())
        })
        .collect();
    for session_id in expired {
        sessions.remove(&session_id);
        metrics.evicted_ttl.fetch_add(1, Ordering::Relaxed);
    }
}

fn least_recently_used_session_id(
    sessions: &HashMap<String, CachedSearchSession>,
) -> Option<String> {
    sessions
        .iter()
        .min_by_key(|(_, session)| session.last_accessed)
        .map(|(session_id, _)| session_id.clone())
}

fn paginate_cached_response(
    session_id: &str,
    offset: usize,
    cached: &CachedSearchSession,
) -> rdump::contracts::SearchResponse {
    let mut response = cached.response.clone();
    let total = response.results.len();
    let mut sliced = Vec::new();
    let mut emitted = 0usize;
    let mut returned_bytes = 0usize;
    let mut truncation_reason = None;

    for item in cached.response.results.iter().skip(offset) {
        if emitted >= cached.page_size {
            truncation_reason = Some("max_results".to_string());
            break;
        }

        let item_bytes = measure_serialized_bytes(item);
        let next_bytes = returned_bytes.saturating_add(item_bytes);
        if emitted > 0 && next_bytes > cached.byte_budget {
            truncation_reason = Some("max_total_bytes".to_string());
            break;
        }

        returned_bytes = next_bytes;
        sliced.push(item.clone());
        emitted += 1;
    }

    let next_offset = if offset + emitted < total {
        Some(offset + emitted)
    } else {
        None
    };

    response.results = sliced;
    response.truncated = next_offset.is_some();
    response.truncation_reason = if response.truncated {
        Some(truncation_reason.unwrap_or_else(|| "continuation".to_string()))
    } else {
        None
    };
    response.next_offset = next_offset;
    response.continuation_token = next_offset.map(|next| format_session_token(session_id, next));
    response.page_size = Some(cached.page_size);
    response.status = if response.truncated {
        rdump::contracts::SearchStatus::TruncatedSuccess
    } else {
        cached.response.status
    };
    response
}

fn measure_serialized_bytes<T: Serialize>(value: &T) -> usize {
    serde_json::to_vec(value)
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

impl SearchProgressEmitter {
    fn emit(&self, event: &rdump::contracts::ProgressEvent) {
        let params = match event {
            rdump::contracts::ProgressEvent::Started {
                root,
                queue_wait_millis,
                ..
            } => serde_json::json!({
                "progressToken": self.progress_token,
                "progress": 0,
                "total": 100,
                "message": format!(
                    "started search under `{root}` (queue_wait_ms={queue_wait_millis})"
                ),
            }),
            rdump::contracts::ProgressEvent::Phase {
                name,
                completed_items,
                total_items,
                ..
            } => serde_json::json!({
                "progressToken": self.progress_token,
                "progress": (*completed_items).min(u64::MAX as usize) as u64,
                "total": total_items.map(|value| value.min(u64::MAX as usize) as u64),
                "message": format!("phase `{name}`"),
            }),
            rdump::contracts::ProgressEvent::Result {
                path,
                emitted_results,
                ..
            } => serde_json::json!({
                "progressToken": self.progress_token,
                "progress": (*emitted_results).min(u64::MAX as usize) as u64,
                "message": format!("emitted result for `{path}`"),
            }),
            rdump::contracts::ProgressEvent::Finished {
                returned_files,
                returned_matches,
                truncated,
                ..
            } => {
                let completed = (*returned_files).max(1).min(u64::MAX as usize) as u64;
                serde_json::json!({
                    "progressToken": self.progress_token,
                    "progress": completed,
                    "total": completed,
                    "message": format!(
                        "finished (files={returned_files}, matches={returned_matches}, truncated={truncated})"
                    ),
                })
            }
        };
        let notification =
            JsonRpcNotification::new("notifications/progress".to_string(), Some(params));
        let _ = self.notifications_tx.send(notification);
    }
}

impl McpHandler for RdumpServer {
    fn server_info(&self) -> ServerInfo {
        ServerInfo::new("rdump", env!("CARGO_PKG_VERSION"))
            .with_description("Search codebases using rdump and RQL. Structured outputs are versioned with schema_version=rdump.v1.")
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool::new(
                "search",
                "Search codebases using rdump + RQL. Output modes: paths|matches|snippets|full|summary. Limits accept null for unlimited.",
            )
            .with_schema(tool_schema::<SearchArgs>())
            .read_only(),
            Tool::new(
                "list_languages",
                "List supported languages and extensions for rdump semantic predicates.",
            )
            .with_schema(ToolInputSchema::empty())
            .read_only(),
            Tool::new(
                "validate_query",
                "Validate an RQL query string without executing a search.",
            )
            .with_schema(tool_schema::<ValidateQueryArgs>())
            .read_only(),
            Tool::new(
                "explain_query",
                "Explain preset expansion, predicate classes, and evaluation stages for an RQL query.",
            )
            .with_schema(tool_schema::<ExplainQueryArgs>())
            .read_only(),
            Tool::new(
                "describe_language",
                "Describe predicates available for a specific language (name or extension).",
            )
            .with_schema(tool_schema::<DescribeLanguageArgs>())
            .read_only(),
            Tool::new(
                "rql_reference",
                "Reference for rdump Query Language (RQL), predicates, and syntax.",
            )
            .with_schema(ToolInputSchema::empty())
            .read_only(),
            Tool::new(
                "sdk_reference",
                "Reference for the rdump SDK: SearchOptions, functions, and result types.",
            )
            .with_schema(ToolInputSchema::empty())
            .read_only(),
            Tool::new(
                "capability_metadata",
                "Describe output modes, default limits, stability tiers, and schema version for rdump.",
            )
            .with_schema(ToolInputSchema::empty())
            .read_only(),
            Tool::new(
                "predicate_catalog",
                "Machine-readable predicate catalog including aliases and deprecated spellings.",
            )
            .with_schema(ToolInputSchema::empty())
            .read_only(),
            Tool::new(
                "language_matrix",
                "Machine-readable language capability matrix with support tiers and predicate coverage.",
            )
            .with_schema(ToolInputSchema::empty())
            .read_only(),
        ]
    }

    fn list_resources(&self) -> Vec<Resource> {
        vec![
            Resource::new("rdump://docs/rql", "rql_reference")
                .with_description("RQL reference documentation")
                .with_mime_type("text/plain"),
            Resource::new("rdump://docs/sdk", "sdk_reference")
                .with_description("rdump SDK reference documentation")
                .with_mime_type("text/plain"),
            Resource::new("rdump://docs/languages", "languages")
                .with_description("Supported languages and aliases")
                .with_mime_type("text/plain"),
            Resource::new("rdump://docs/examples", "examples")
                .with_description("Sample RQL queries")
                .with_mime_type("text/plain"),
            Resource::new("rdump://docs/runtime", "runtime")
                .with_description("Runtime troubleshooting and operator guide")
                .with_mime_type("text/plain"),
            Resource::new("rdump://docs/stdio", "stdio_guide")
                .with_description("MCP stdio deployment and integration guide")
                .with_mime_type("text/plain"),
            Resource::new("rdump://docs/session-cache", "session_cache")
                .with_description("Session-cache limits, TTL, and live eviction counters")
                .with_mime_type("text/plain"),
            Resource::new("rdump://docs/schema-examples", "schema_examples")
                .with_description("Sample MCP request and response payloads")
                .with_mime_type("text/plain"),
            Resource::new("rdump://docs/stability", "stability")
                .with_description("CLI, SDK, and MCP stability policy")
                .with_mime_type("text/plain"),
            Resource::new("rdump://docs/capabilities", "capabilities")
                .with_description("Structured capability metadata and stability tiers")
                .with_mime_type("application/json"),
            Resource::new("rdump://docs/predicates", "predicate_catalog")
                .with_description(
                    "Machine-readable predicate catalog with aliases and deprecated spellings",
                )
                .with_mime_type("application/json"),
            Resource::new("rdump://docs/language-matrix", "language_matrix")
                .with_description("Machine-readable language capability matrix with support tiers")
                .with_mime_type("application/json"),
            Resource::new("rdump://config/active", "active_config")
                .with_description("Merged rdump config visible from the current working directory")
                .with_mime_type("application/json"),
            Resource::new("rdump://config/presets", "presets")
                .with_description("Merged preset table from the active config")
                .with_mime_type("application/json"),
        ]
    }

    fn list_prompts(&self) -> Vec<Prompt> {
        vec![
            Prompt::new(
                "onboarding",
                "Orient an agent to rdump capabilities, safety defaults, and query workflow.",
            ),
            Prompt::new(
                "search_workflow",
                "Guide an agent through validate -> explain -> search with conservative defaults.",
            ),
        ]
    }

    fn call_tool<'a>(
        &'a self,
        name: &'a str,
        args: Value,
        ctx: &'a RequestContext,
    ) -> impl std::future::Future<Output = McpResult<ToolResult>> + Send + 'a {
        let name = name.to_string();
        async move {
            match name.as_str() {
                "search" => match parse_args(args) {
                    Ok(args) => self.search_tool(args, Some(ctx.request_id.as_str())).await,
                    Err(err) => {
                        let error = rdump::request::contract_error(
                            crate::types::ErrorCode::InvalidRequest,
                            err.to_string(),
                            None,
                            false,
                            Some(
                                "Check the tool schema for supported arguments and enums."
                                    .to_string(),
                            ),
                        );
                        tool_error_result(&error, format!("Invalid arguments: {}", error.message))
                    }
                },
                "list_languages" => self.list_languages_tool(),
                "validate_query" => match parse_args(args) {
                    Ok(args) => self.validate_query_tool(args),
                    Err(err) => {
                        let error = rdump::request::contract_error(
                            crate::types::ErrorCode::InvalidRequest,
                            err.to_string(),
                            None,
                            false,
                            Some("Provide a JSON object with a `query` string.".to_string()),
                        );
                        tool_error_result(&error, format!("Invalid arguments: {}", error.message))
                    }
                },
                "explain_query" => match parse_args(args) {
                    Ok(args) => self.explain_query_tool(args),
                    Err(err) => {
                        let error = rdump::request::contract_error(
                            crate::types::ErrorCode::InvalidRequest,
                            err.to_string(),
                            None,
                            false,
                            Some(
                                "Provide a JSON object with `query` and optional `presets`."
                                    .to_string(),
                            ),
                        );
                        tool_error_result(&error, format!("Invalid arguments: {}", error.message))
                    }
                },
                "describe_language" => match parse_args(args) {
                    Ok(args) => self.describe_language_tool(args),
                    Err(err) => {
                        let error = rdump::request::contract_error(
                            crate::types::ErrorCode::InvalidRequest,
                            err.to_string(),
                            None,
                            false,
                            Some("Provide a JSON object with a `language` string.".to_string()),
                        );
                        tool_error_result(&error, format!("Invalid arguments: {}", error.message))
                    }
                },
                "rql_reference" => self.rql_reference_tool(),
                "sdk_reference" => self.sdk_reference_tool(),
                "capability_metadata" => tool_result(
                    &rdump::request::capability_metadata(),
                    "rdump capability metadata".to_string(),
                ),
                "predicate_catalog" => tool_result(
                    &rdump::request::predicate_catalog(),
                    "rdump predicate catalog".to_string(),
                ),
                "language_matrix" => tool_result(
                    &rdump::request::language_capability_matrix(),
                    "rdump language capability matrix".to_string(),
                ),
                _ => Err(McpError::tool_not_found(&name)),
            }
        }
    }

    fn read_resource<'a>(
        &'a self,
        uri: &'a str,
        _ctx: &'a RequestContext,
    ) -> impl std::future::Future<Output = McpResult<ResourceResult>> + Send + 'a {
        let uri = uri.to_string();
        async move {
            match uri.as_str() {
                "rdump://docs/rql" => Ok(ResourceResult::text(
                    uri,
                    crate::docs::format_rql_reference_text(),
                )),
                "rdump://docs/sdk" => Ok(ResourceResult::text(
                    uri,
                    crate::docs::format_sdk_reference_text(),
                )),
                "rdump://docs/languages" => Ok(ResourceResult::text(
                    uri,
                    crate::languages::format_language_list_text(&crate::languages::list_languages()),
                )),
                "rdump://docs/examples" => Ok(ResourceResult::text(
                    uri,
                    crate::docs::build_rql_reference().examples.join("\n"),
                )),
                "rdump://docs/runtime" => Ok(ResourceResult::text(
                    uri,
                    include_str!("../../docs/runtime-guide.md").to_string(),
                )),
                "rdump://docs/stdio" => Ok(ResourceResult::text(
                    uri,
                    include_str!("../../docs/mcp-stdio-guide.md").to_string(),
                )),
                "rdump://docs/session-cache" => {
                    Ok(ResourceResult::text(uri, self.session_cache_text()))
                }
                "rdump://docs/schema-examples" => Ok(ResourceResult::text(
                    uri,
                    crate::docs::format_schema_examples_text(),
                )),
                "rdump://docs/stability" => Ok(ResourceResult::text(
                    uri,
                    include_str!("../../docs/stability.md").to_string(),
                )),
                "rdump://docs/capabilities" => Ok(ResourceResult::text(
                    uri,
                    serde_json::to_string_pretty(&rdump::request::capability_metadata())
                        .expect("capability metadata should serialize"),
                )),
                "rdump://docs/predicates" => Ok(ResourceResult::text(
                    uri,
                    serde_json::to_string_pretty(&rdump::request::predicate_catalog())
                        .expect("predicate catalog should serialize"),
                )),
                "rdump://docs/language-matrix" => Ok(ResourceResult::text(
                    uri,
                    serde_json::to_string_pretty(&rdump::request::language_capability_matrix())
                        .expect("language matrix should serialize"),
                )),
                "rdump://config/active" => Ok(ResourceResult::text(
                    uri,
                    serde_json::to_string_pretty(&rdump::config::load_config().unwrap_or_default())
                        .expect("config should serialize"),
                )),
                "rdump://config/presets" => Ok(ResourceResult::text(
                    uri,
                    serde_json::to_string_pretty(
                        &rdump::config::load_config().unwrap_or_default().presets,
                    )
                    .expect("presets should serialize"),
                )),
                _ => Err(McpError::resource_not_found(&uri)),
            }
        }
    }

    fn get_prompt<'a>(
        &'a self,
        name: &'a str,
        _args: Option<Value>,
        _ctx: &'a RequestContext,
    ) -> impl std::future::Future<Output = McpResult<PromptResult>> + Send + 'a {
        let name = name.to_string();
        async move {
            match name.as_str() {
                "onboarding" => Ok(PromptResult::user(
                    "Start with `capability_metadata`, then `list_languages`, `validate_query`, and `explain_query` before a broad search. Prefer `output=summary` or `snippets`, keep default limits unless you need more, and inspect `schema_version` plus `error_mode` in responses."
                )),
                "search_workflow" => Ok(PromptResult::user(
                    "1. Validate the query.\n2. Explain it to confirm preset expansion and cost.\n3. Search with `output=summary` first.\n4. Escalate to `snippets` or `full` only when needed.\n5. Respect `schema_version`, diagnostics, and typed error codes in structured output."
                )),
                _ => Err(McpError::prompt_not_found(&name)),
            }
        }
    }
}

pub async fn run_stdio() -> Result<(), Box<dyn std::error::Error>> {
    crate::stdio::run(RdumpServer::default()).await?;
    Ok(())
}

fn default_max_concurrent_searches() -> usize {
    std::env::var("RDUMP_MCP_MAX_CONCURRENT_SEARCHES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or_else(rdump::default_max_concurrent_searches)
}

fn default_session_ttl() -> Duration {
    std::env::var("RDUMP_MCP_SESSION_TTL_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(15 * 60))
}

fn default_max_cached_sessions() -> usize {
    std::env::var("RDUMP_MCP_MAX_CACHED_SESSIONS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(16)
}

fn queue_overload_threshold_millis() -> u64 {
    std::env::var("RDUMP_MCP_QUEUE_OVERLOAD_MILLIS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(200)
}

fn tool_schema<T: schemars::JsonSchema>() -> ToolInputSchema {
    let schema = schemars::schema_for!(T);
    let value = serde_json::to_value(schema).expect("Generated schema should be serializable");
    ToolInputSchema::from_value(value)
}

fn parse_args<T: DeserializeOwned>(args: Value) -> McpResult<T> {
    let args = match args {
        Value::Null => Value::Object(Default::default()),
        other => other,
    };

    serde_json::from_value(args)
        .map_err(|err| McpError::invalid_params(format!("Invalid arguments: {err}")))
}
