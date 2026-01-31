pub mod docs;
pub mod languages;
pub mod limits;
pub mod responses;
pub mod search;
pub mod types;

use std::sync::Arc;

use docs::{build_rql_reference, build_sdk_reference, format_rql_reference_text, format_sdk_reference_text};
use languages::{describe_language, format_language_list_text, format_language_text, list_languages};
use responses::tool_result;
use search::{build_search_request, format_search_text, run_search};
use types::{
    DescribeLanguageArgs, LanguageInfo, RqlReference, SearchArgs, SearchResponse, SdkReference,
    ValidateQueryArgs, ValidateQueryResponse,
};
use turbomcp::handlers::utils;
use turbomcp::prelude::{McpError, McpResult};
use turbomcp::schema;
use turbomcp::{
    CallToolRequest, RequestContext, ServerBuilder, ServerError, ToolInputSchema,
};
use turbomcp::turbomcp_protocol::types::{
    ReadResourceResult, ResourceContent, TextResourceContents,
};
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct RdumpServer {
    search_semaphore: Arc<Semaphore>,
}

impl Default for RdumpServer {
    fn default() -> Self {
        Self {
            search_semaphore: Arc::new(Semaphore::new(default_max_concurrent_searches())),
        }
    }
}

impl RdumpServer {
    fn search_tool(&self, args: SearchArgs) -> McpResult<SearchResponse> {
        let request = build_search_request(args)?;
        run_search(request)
    }

    fn list_languages_tool(&self) -> McpResult<Vec<LanguageInfo>> {
        Ok(list_languages())
    }

    fn describe_language_tool(&self, args: DescribeLanguageArgs) -> McpResult<LanguageInfo> {
        describe_language(&args.language)
    }

    fn rql_reference_tool(&self) -> McpResult<RqlReference> {
        Ok(build_rql_reference())
    }

    fn sdk_reference_tool(&self) -> McpResult<SdkReference> {
        Ok(build_sdk_reference())
    }

    fn validate_query_tool(&self, args: ValidateQueryArgs) -> McpResult<ValidateQueryResponse> {
        match rdump::parser::parse_query(&args.query) {
            Ok(_) => Ok(ValidateQueryResponse {
                valid: true,
                error: None,
            }),
            Err(err) => Ok(ValidateQueryResponse {
                valid: false,
                error: Some(err.to_string()),
            }),
        }
    }

    fn rql_doc_text(&self) -> String {
        format_rql_reference_text()
    }

    fn sdk_doc_text(&self) -> String {
        format_sdk_reference_text()
    }

    pub async fn run_custom_stdio(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = ServerBuilder::new()
            .name("rdump")
            .version(env!("CARGO_PKG_VERSION"));

        let server = self.clone();
        builder = builder.tool(
            "search",
            utils::tool_with_schema(
                "search",
                "Search codebases using rdump + RQL. Output modes: paths|matches|snippets|full|summary. Limits accept null for unlimited.",
                tool_schema::<SearchArgs>(),
                move |req: CallToolRequest, _ctx: RequestContext| {
                    let server = server.clone();
                    async move {
                        let args: SearchArgs = parse_args(&req)?;
                        let permit = server
                            .search_semaphore
                            .clone()
                            .acquire_owned()
                            .await
                            .map_err(|_| ServerError::handler("Search limiter is closed"))?;
                        let response = tokio::task::spawn_blocking(move || {
                            let _permit = permit;
                            server.search_tool(args)
                        })
                        .await
                        .map_err(|err| {
                            ServerError::handler(format!("Search task failed to join: {err}"))
                        })?
                        .map_err(to_server_error)?;
                        let text = format_search_text(&response);
                        tool_result(&response, text).map_err(to_server_error)
                    }
                },
            ),
        )?;

        let server = self.clone();
        builder = builder.tool(
            "list_languages",
            utils::tool_with_schema(
                "list_languages",
                "List supported languages and extensions for rdump semantic predicates.",
                ToolInputSchema::empty(),
                move |_req: CallToolRequest, _ctx: RequestContext| {
                    let server = server.clone();
                    async move {
                        let response = server.list_languages_tool().map_err(to_server_error)?;
                        let text = format_language_list_text(&response);
                        tool_result(&response, text).map_err(to_server_error)
                    }
                },
            ),
        )?;

        let server = self.clone();
        builder = builder.tool(
            "validate_query",
            utils::tool_with_schema(
                "validate_query",
                "Validate an RQL query string without executing a search.",
                tool_schema::<ValidateQueryArgs>(),
                move |req: CallToolRequest, _ctx: RequestContext| {
                    let server = server.clone();
                    async move {
                        let args: ValidateQueryArgs = parse_args(&req)?;
                        let response = server.validate_query_tool(args).map_err(to_server_error)?;
                        let text = if response.valid {
                            "Query is valid.".to_string()
                        } else {
                            format!(
                                "Query is invalid: {}",
                                response.error.as_deref().unwrap_or("unknown error")
                            )
                        };
                        tool_result(&response, text).map_err(to_server_error)
                    }
                },
            ),
        )?;

        let server = self.clone();
        builder = builder.tool(
            "describe_language",
            utils::tool_with_schema(
                "describe_language",
                "Describe predicates available for a specific language (name or extension).",
                tool_schema::<DescribeLanguageArgs>(),
                move |req: CallToolRequest, _ctx: RequestContext| {
                    let server = server.clone();
                    async move {
                        let args: DescribeLanguageArgs = parse_args(&req)?;
                        let response = server.describe_language_tool(args).map_err(to_server_error)?;
                        let text = format_language_text(&response);
                        tool_result(&response, text).map_err(to_server_error)
                    }
                },
            ),
        )?;

        let server = self.clone();
        builder = builder.tool(
            "rql_reference",
            utils::tool_with_schema(
                "rql_reference",
                "Reference for rdump Query Language (RQL), predicates, and syntax.",
                ToolInputSchema::empty(),
                move |_req: CallToolRequest, _ctx: RequestContext| {
                    let server = server.clone();
                    async move {
                        let response = server.rql_reference_tool().map_err(to_server_error)?;
                        let text = format_rql_reference_text();
                        tool_result(&response, text).map_err(to_server_error)
                    }
                },
            ),
        )?;

        let server = self.clone();
        builder = builder.tool(
            "sdk_reference",
            utils::tool_with_schema(
                "sdk_reference",
                "Reference for the rdump SDK: SearchOptions, functions, and result types.",
                ToolInputSchema::empty(),
                move |_req: CallToolRequest, _ctx: RequestContext| {
                    let server = server.clone();
                    async move {
                        let response = server.sdk_reference_tool().map_err(to_server_error)?;
                        let text = format_sdk_reference_text();
                        tool_result(&response, text).map_err(to_server_error)
                    }
                },
            ),
        )?;

        let server = self.clone();
        builder = builder.resource(
            "rdump://docs/rql",
            utils::resource("rdump://docs/rql", "RQL Reference", move |_req, _ctx| {
                let server = server.clone();
                async move {
                    Ok(ReadResourceResult {
                        contents: vec![ResourceContent::Text(TextResourceContents {
                            uri: "rdump://docs/rql".to_string(),
                            mime_type: Some("text/plain".to_string()),
                            text: server.rql_doc_text(),
                            meta: None,
                        })],
                        _meta: None,
                    })
                }
            }),
        )?;

        let server = self.clone();
        builder = builder.resource(
            "rdump://docs/sdk",
            utils::resource("rdump://docs/sdk", "SDK Reference", move |_req, _ctx| {
                let server = server.clone();
                async move {
                    Ok(ReadResourceResult {
                        contents: vec![ResourceContent::Text(TextResourceContents {
                            uri: "rdump://docs/sdk".to_string(),
                            mime_type: Some("text/plain".to_string()),
                            text: server.sdk_doc_text(),
                            meta: None,
                        })],
                        _meta: None,
                    })
                }
            }),
        )?;

        let server = builder.build();
        server.run_stdio().await?;
        Ok(())
    }
}

pub async fn run_stdio() -> Result<(), Box<dyn std::error::Error>> {
    RdumpServer::default().run_custom_stdio().await
}

fn default_max_concurrent_searches() -> usize {
    std::env::var("RDUMP_MCP_MAX_CONCURRENT_SEARCHES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|value| value.get())
                .unwrap_or(4)
        })
}

fn tool_schema<T: schemars::JsonSchema>() -> ToolInputSchema {
    let schema_json = schema::generate_schema::<T>();
    serde_json::from_value(schema_json)
        .expect("Generated schema should always be valid ToolInputSchema")
}

fn parse_args<T: serde::de::DeserializeOwned>(
    request: &CallToolRequest,
) -> Result<T, ServerError> {
    let args = request.arguments.clone().unwrap_or_default();
    let value = serde_json::Value::Object(args.into_iter().collect());
    serde_json::from_value(value)
        .map_err(|e| ServerError::handler(format!("Invalid arguments: {e}")))
}

fn to_server_error(error: McpError) -> ServerError {
    match error {
        McpError::Server(server_err) => server_err,
        McpError::Tool(msg)
        | McpError::Resource(msg)
        | McpError::Prompt(msg)
        | McpError::Protocol(msg)
        | McpError::Context(msg)
        | McpError::Network(msg)
        | McpError::InvalidInput(msg)
        | McpError::Schema(msg)
        | McpError::Transport(msg)
        | McpError::Internal(msg)
        | McpError::InvalidRequest(msg) => ServerError::handler(msg),
        McpError::Unauthorized(msg) => ServerError::authorization(msg),
        McpError::Serialization(err) => ServerError::from(err),
    }
}
