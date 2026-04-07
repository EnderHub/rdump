use crate::types::{
    ErrorMode, FieldDoc, FunctionDoc, Limits, OutputMode, RqlReference, SdkReference,
    SearchRequest, SearchResponse, TypeDoc,
};
use rdump::predicates::{
    content_predicate_keys, metadata_predicate_keys, react_predicate_keys, semantic_predicate_keys,
};
use rdump_contracts::{LimitValue, SearchStatus};

pub fn build_rql_reference() -> RqlReference {
    let metadata_predicates = metadata_predicate_keys()
        .into_iter()
        .map(|key| key.as_ref().to_string())
        .filter(|name| name != "path_exact")
        .collect();
    let content_predicates = content_predicate_keys()
        .into_iter()
        .map(|key| key.as_ref().to_string())
        .collect();
    let semantic_predicates = semantic_predicate_keys()
        .into_iter()
        .map(|key| key.as_ref().to_string())
        .collect();
    let react_predicates = react_predicate_keys()
        .into_iter()
        .map(|key| key.as_ref().to_string())
        .collect();

    RqlReference {
        schema_version: crate::types::SCHEMA_VERSION.to_string(),
        operators: vec!["AND: &", "OR: |", "NOT: !", "Grouping: ( )"]
            .into_iter()
            .map(String::from)
            .collect(),
        notes: vec![
            "Quote values with spaces using single or double quotes.",
            "Examples: contains:'fn main' or name:'test file.rs'.",
            "Implicit AND is not supported; use '&'.",
            "Use backslashes inside quoted values when you need literal quote characters.",
            "Glob characters remain literal inside quoted predicate values unless the predicate itself is glob-aware.",
            "Unicode is preserved in quoted values; prefer exact quotes instead of shell escaping when possible.",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        aliases: vec![
            "contains -> c".to_string(),
            "matches -> m".to_string(),
        ],
        deprecated_predicates: vec!["content -> contains".to_string()],
        metadata_predicates,
        content_predicates,
        semantic_predicates,
        react_predicates,
        examples: vec![
            "ext:rs & func:main",
            "path:src & (struct:User | enum:UserState)",
            "import:serde & contains:derive",
            "ext:tsx & component:Button & hook:useState",
            "modified:<2d & size:>10kb",
            "contains:\"literal * glob\"",
            "name:'unicode_名前.py'",
            "matches:'^fn\\\\s+main'",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    }
}

pub fn build_sdk_reference() -> SdkReference {
    let functions = vec![
        FunctionDoc {
            name: "search_iter".to_string(),
            signature:
                "search_iter(query: &str, options: SearchOptions) -> Result<SearchResultIterator>"
                    .to_string(),
            description: "Streaming iterator over SearchResult. Preferred for large repos."
                .to_string(),
        },
        FunctionDoc {
            name: "search".to_string(),
            signature: "search(query: &str, options: SearchOptions) -> Result<Vec<SearchResult>>"
                .to_string(),
            description: "Collects all results into memory.".to_string(),
        },
        FunctionDoc {
            name: "search_with_stats".to_string(),
            signature:
                "search_with_stats(query: &str, options: SearchOptions) -> Result<SearchReport>"
                    .to_string(),
            description: "Collects results and returns engine statistics plus diagnostics."
                .to_string(),
        },
        FunctionDoc {
            name: "search_path_iter".to_string(),
            signature:
                "search_path_iter(query: &str, options: SearchOptions) -> Result<SearchPathIterator>"
                    .to_string(),
            description: "Streams matching paths without loading file content.".to_string(),
        },
        FunctionDoc {
            name: "search_paths".to_string(),
            signature: "search_paths(query: &str, options: SearchOptions) -> Result<Vec<PathBuf>>"
                .to_string(),
            description: "Collects matching paths without materializing SearchResult content."
                .to_string(),
        },
        FunctionDoc {
            name: "explain_query".to_string(),
            signature:
                "explain_query(query: &str, options: &SearchOptions) -> Result<QueryExplanation>"
                    .to_string(),
            description: "Explains preset expansion, predicate classes, and evaluation stages."
                .to_string(),
        },
        FunctionDoc {
            name: "SearchRuntime::with_backend".to_string(),
            signature:
                "SearchRuntime::with_backend(backend: Arc<dyn SearchBackend>) -> SearchRuntime"
                    .to_string(),
            description:
                "Constructs a runtime backed by a custom filesystem or virtual-workspace adapter."
                    .to_string(),
        },
        FunctionDoc {
            name: "execute_search_request_with_runtime".to_string(),
            signature:
                "execute_search_request_with_runtime(runtime: SearchRuntime, request: &SearchRequest) -> Result<SearchResponse>"
                    .to_string(),
            description:
                "Runs contract/request searches against a caller-supplied backend runtime."
                    .to_string(),
        },
        FunctionDoc {
            name: "execute_search_request_with_runtime_and_progress".to_string(),
            signature:
                "execute_search_request_with_runtime_and_progress(runtime: SearchRuntime, request: &SearchRequest, progress: impl FnMut(&ProgressEvent)) -> Result<SearchResponse>"
                    .to_string(),
            description:
                "Runs contract/request searches against a caller-supplied backend runtime and emits progress events."
                    .to_string(),
        },
        FunctionDoc {
            name: "execute_search_request_with_runtime_and_cancellation".to_string(),
            signature:
                "execute_search_request_with_runtime_and_cancellation(runtime: SearchRuntime, request: &SearchRequest, cancellation: Option<SearchCancellationToken>, session_id: &str, progress: impl FnMut(&ProgressEvent)) -> Result<SearchResponse>"
                    .to_string(),
            description:
                "Runs contract/request searches against a caller-supplied backend runtime with explicit cancellation and session identity."
                    .to_string(),
        },
        FunctionDoc {
            name: "repo_language_inventory_with_runtime".to_string(),
            signature:
                "repo_language_inventory_with_runtime(runtime: &SearchRuntime, options: &SearchOptions) -> Vec<RepoLanguageCount>"
                    .to_string(),
            description:
                "Builds planner/preflight language inventory against a caller-supplied backend runtime."
                    .to_string(),
        },
        FunctionDoc {
            name: "search_async_with_runtime".to_string(),
            signature:
                "search_async_with_runtime(runtime: SearchRuntime, query: &str, options: SearchOptions) -> Result<SearchAsyncStream>"
                    .to_string(),
            description:
                "Starts async streaming search against a caller-supplied backend runtime."
                    .to_string(),
        },
        FunctionDoc {
            name: "search_async_with_runtime_and_progress".to_string(),
            signature:
                "search_async_with_runtime_and_progress(runtime: SearchRuntime, query: &str, options: SearchOptions, progress: impl FnMut(ProgressEvent) + Send + 'static) -> Result<SearchAsyncStream>"
                    .to_string(),
            description:
                "Starts async streaming search against a caller-supplied backend runtime and emits progress events."
                    .to_string(),
        },
    ];

    let search_options_fields = vec![
        FieldDoc {
            name: "root".to_string(),
            description: "Root directory to search.".to_string(),
            default: "\".\"".to_string(),
        },
        FieldDoc {
            name: "presets".to_string(),
            description: "Named presets to apply (e.g., rust, python).".to_string(),
            default: "[]".to_string(),
        },
        FieldDoc {
            name: "no_ignore".to_string(),
            description: "If true, ignore .gitignore/.rdumpignore.".to_string(),
            default: "false".to_string(),
        },
        FieldDoc {
            name: "hidden".to_string(),
            description: "If true, include hidden files and directories.".to_string(),
            default: "false".to_string(),
        },
        FieldDoc {
            name: "max_depth".to_string(),
            description: "Maximum directory depth.".to_string(),
            default: "None".to_string(),
        },
        FieldDoc {
            name: "sql_dialect".to_string(),
            description: "Override SQL dialect for .sql files.".to_string(),
            default: "None".to_string(),
        },
    ];

    let result_fields = vec![
        FieldDoc {
            name: "path".to_string(),
            description: "Path to matched file.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "matches".to_string(),
            description: "Match hunks (empty for whole-file matches).".to_string(),
            default: "[]".to_string(),
        },
        FieldDoc {
            name: "content".to_string(),
            description: "Full file content.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "content_state".to_string(),
            description: "Whether content was loaded, lossy-decoded, or skipped.".to_string(),
            default: "loaded".to_string(),
        },
        FieldDoc {
            name: "diagnostics".to_string(),
            description: "Per-result warnings collected while loading content.".to_string(),
            default: "[]".to_string(),
        },
    ];

    let match_fields = vec![
        FieldDoc {
            name: "start_line".to_string(),
            description: "1-indexed start line.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "end_line".to_string(),
            description: "1-indexed end line.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "start_column".to_string(),
            description: "0-indexed start column.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "end_column".to_string(),
            description: "0-indexed end column.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "byte_range".to_string(),
            description: "Byte range within file content.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "text".to_string(),
            description: "Matched text (may be shortened).".to_string(),
            default: "".to_string(),
        },
    ];

    let types = vec![
        TypeDoc {
            name: "SearchOptions".to_string(),
            description: "Search configuration for SDK calls.".to_string(),
            fields: search_options_fields,
        },
        TypeDoc {
            name: "SearchRuntime".to_string(),
            description:
                "Backend-bound search session wrapper for real filesystems or virtual workspaces."
                    .to_string(),
            fields: vec![],
        },
        TypeDoc {
            name: "SearchBackend".to_string(),
            description:
                "Trait implemented by real-fs or virtual-workspace adapters that provide discovery, normalization, metadata, and bytes."
                    .to_string(),
            fields: vec![
                FieldDoc {
                    name: "normalize_root(...)".to_string(),
                    description:
                        "Normalizes a caller-supplied root into a backend-stable root path."
                            .to_string(),
                    default: "".to_string(),
                },
                FieldDoc {
                    name: "discover(...)".to_string(),
                    description:
                        "Enumerates candidate files and skip diagnostics for one search session."
                            .to_string(),
                    default: "".to_string(),
                },
                FieldDoc {
                    name: "normalize_path(...)".to_string(),
                    description:
                        "Converts a display or resolved path into backend-normalized identity."
                            .to_string(),
                    default: "".to_string(),
                },
                FieldDoc {
                    name: "stat(...)".to_string(),
                    description:
                        "Returns backend metadata used for predicates, snapshots, and formatters."
                            .to_string(),
                    default: "".to_string(),
                },
                FieldDoc {
                    name: "read_bytes(...)".to_string(),
                    description:
                        "Returns raw file bytes for content loading and semantic evaluation."
                            .to_string(),
                    default: "".to_string(),
                },
            ],
        },
        TypeDoc {
            name: "DiscoveryRequest".to_string(),
            description:
                "Search-oriented discovery input passed to SearchBackend::discover(...)."
                    .to_string(),
            fields: vec![
                FieldDoc {
                    name: "root".to_string(),
                    description: "Normalized backend root for candidate enumeration.".to_string(),
                    default: "".to_string(),
                },
                FieldDoc {
                    name: "display_root".to_string(),
                    description: "Caller-facing root projection used for display paths.".to_string(),
                    default: "".to_string(),
                },
                FieldDoc {
                    name: "no_ignore".to_string(),
                    description: "Disables ignore-file filtering when true.".to_string(),
                    default: "false".to_string(),
                },
                FieldDoc {
                    name: "hidden".to_string(),
                    description: "Includes hidden files when true.".to_string(),
                    default: "false".to_string(),
                },
            ],
        },
        TypeDoc {
            name: "DiscoveryReport".to_string(),
            description:
                "Candidate file identities and skip counters returned by backend discovery."
                    .to_string(),
            fields: vec![
                FieldDoc {
                    name: "candidates".to_string(),
                    description: "Backend-normalized file identities eligible for evaluation."
                        .to_string(),
                    default: "[]".to_string(),
                },
                FieldDoc {
                    name: "diagnostics".to_string(),
                    description: "Discovery-time warnings or errors.".to_string(),
                    default: "[]".to_string(),
                },
            ],
        },
        TypeDoc {
            name: "BackendPathIdentity".to_string(),
            description:
                "Stable backend path identity used for display, resolution, and root-relative projections."
                    .to_string(),
            fields: vec![
                FieldDoc {
                    name: "display_path".to_string(),
                    description: "Caller-facing path projection.".to_string(),
                    default: "".to_string(),
                },
                FieldDoc {
                    name: "resolved_path".to_string(),
                    description: "Backend-stable resolved path, not necessarily a host canonical path."
                        .to_string(),
                    default: "".to_string(),
                },
                FieldDoc {
                    name: "root_relative_path".to_string(),
                    description: "Path relative to the backend root when available.".to_string(),
                    default: "None".to_string(),
                },
            ],
        },
        TypeDoc {
            name: "BackendMetadata".to_string(),
            description:
                "Backend-neutral metadata used by predicates, snapshots, request payloads, and formatters."
                    .to_string(),
            fields: vec![
                FieldDoc {
                    name: "size_bytes".to_string(),
                    description: "File length in bytes.".to_string(),
                    default: "0".to_string(),
                },
                FieldDoc {
                    name: "modified_unix_millis".to_string(),
                    description: "Last-modified timestamp when available.".to_string(),
                    default: "None".to_string(),
                },
                FieldDoc {
                    name: "permissions_display".to_string(),
                    description: "Human-readable permissions string.".to_string(),
                    default: "".to_string(),
                },
                FieldDoc {
                    name: "stable_token".to_string(),
                    description:
                        "Optional backend-supplied stable identity token for drift detection and fingerprints."
                            .to_string(),
                    default: "None".to_string(),
                },
            ],
        },
        TypeDoc {
            name: "SearchResult".to_string(),
            description: "Result for a matched file.".to_string(),
            fields: result_fields,
        },
        TypeDoc {
            name: "SearchReport".to_string(),
            description: "Collected results plus search stats and diagnostics.".to_string(),
            fields: vec![
                FieldDoc {
                    name: "results".to_string(),
                    description: "Collected SearchResult values.".to_string(),
                    default: "[]".to_string(),
                },
                FieldDoc {
                    name: "stats".to_string(),
                    description: "Engine-level counts for discovery and matching.".to_string(),
                    default: "".to_string(),
                },
                FieldDoc {
                    name: "diagnostics".to_string(),
                    description: "Engine and content diagnostics aggregated across the search."
                        .to_string(),
                    default: "[]".to_string(),
                },
            ],
        },
        TypeDoc {
            name: "QueryExplanation".to_string(),
            description: "Planner output for effective queries and predicate staging.".to_string(),
            fields: vec![
                FieldDoc {
                    name: "effective_query".to_string(),
                    description: "Query after preset expansion.".to_string(),
                    default: "".to_string(),
                },
                FieldDoc {
                    name: "estimated_cost".to_string(),
                    description: "Rough low/medium/high cost classification.".to_string(),
                    default: "low".to_string(),
                },
            ],
        },
        TypeDoc {
            name: "Match".to_string(),
            description: "Single match span within a file.".to_string(),
            fields: match_fields,
        },
    ];

    let notes = vec![
        "Use search_iter for large repos to avoid loading all results at once.",
        "Use search_path_iter/search_paths when you only need matching file paths.",
        "Implement SearchBackend in external adapters when you need rdump to search a virtual workspace.",
        "Use explain_query to inspect preset expansion and evaluation stages before running a search.",
        "RQL supports logical operators &, |, ! and parentheses.",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    SdkReference {
        schema_version: crate::types::SCHEMA_VERSION.to_string(),
        functions,
        types,
        notes,
    }
}

pub fn format_rql_reference_text() -> String {
    let reference = build_rql_reference();
    let mut lines = Vec::new();

    lines.push("RQL Operators:".to_string());
    for op in &reference.operators {
        lines.push(format!("- {op}"));
    }

    lines.push("".to_string());
    lines.push("Notes:".to_string());
    for note in &reference.notes {
        lines.push(format!("- {note}"));
    }

    lines.push("".to_string());
    lines.push("Metadata predicates:".to_string());
    lines.push(reference.metadata_predicates.join(", "));

    lines.push("".to_string());
    lines.push("Content predicates:".to_string());
    lines.push(reference.content_predicates.join(", "));

    lines.push("".to_string());
    lines.push("Semantic predicates:".to_string());
    lines.push(reference.semantic_predicates.join(", "));

    lines.push("".to_string());
    lines.push("React predicates:".to_string());
    lines.push(reference.react_predicates.join(", "));

    lines.push("".to_string());
    lines.push("Examples:".to_string());
    for example in &reference.examples {
        lines.push(format!("- {example}"));
    }

    lines.join("\n")
}

pub fn format_sdk_reference_text() -> String {
    let reference = build_sdk_reference();
    let mut lines = Vec::new();

    lines.push("SDK Functions:".to_string());
    for func in &reference.functions {
        lines.push(format!("- {}: {}", func.name, func.signature));
        lines.push(format!("  {}", func.description));
    }

    lines.push("".to_string());
    lines.push("SDK Types:".to_string());
    for ty in &reference.types {
        lines.push(format!("- {}: {}", ty.name, ty.description));
        for field in &ty.fields {
            if field.default.is_empty() {
                lines.push(format!("  - {}: {}", field.name, field.description));
            } else {
                lines.push(format!(
                    "  - {}: {} (default: {})",
                    field.name, field.description, field.default
                ));
            }
        }
    }

    lines.push("".to_string());
    lines.push("Notes:".to_string());
    for note in &reference.notes {
        lines.push(format!("- {note}"));
    }

    lines.join("\n")
}

pub fn format_schema_examples_text() -> String {
    let request = SearchRequest {
        query: "ext:rs & func:main".to_string(),
        root: Some(".".to_string()),
        presets: vec!["rust".to_string()],
        output: Some(OutputMode::Summary),
        limits: Some(Limits {
            max_results: LimitValue::Value(25),
            max_matches_per_file: LimitValue::Value(5),
            max_bytes_per_file: LimitValue::Unlimited,
            max_total_bytes: LimitValue::Value(50_000),
            max_match_bytes: LimitValue::Value(200),
            max_snippet_bytes: LimitValue::Value(2_000),
            max_errors: LimitValue::Value(5),
        }),
        error_mode: ErrorMode::SkipErrors,
        ..Default::default()
    };

    let response = SearchResponse {
        schema_version: crate::types::SCHEMA_VERSION.to_string(),
        schema_reference: "rdump://docs/sdk".to_string(),
        status: SearchStatus::TruncatedSuccess,
        coordinate_semantics: rdump::request::coordinate_semantics(),
        query: request.query.clone(),
        effective_query: "(ext:rs) & (func:main)".to_string(),
        root: ".".to_string(),
        output: OutputMode::Summary,
        error_mode: ErrorMode::SkipErrors,
        results: Vec::new(),
        stats: rdump_contracts::SearchStats::default(),
        diagnostics: Vec::new(),
        errors: Vec::new(),
        truncated: true,
        truncation_reason: Some("max_results".to_string()),
        next_offset: Some(25),
        continuation_token: Some("session:v1:123:25:deadbeefdeadbeef".to_string()),
        page_size: Some(25),
    };

    let tool_call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": request,
            "_meta": {
                "progressToken": "search-7"
            }
        }
    });

    let progress = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/progress",
        "params": {
            "progressToken": "search-7",
            "progress": 12,
            "total": 25,
            "message": "phase `evaluate`"
        }
    });

    format!(
        "Sample SearchRequest:\n{}\n\nSample MCP tools/call:\n{}\n\nSample progress notification:\n{}\n\nSample SearchResponse:\n{}",
        serde_json::to_string_pretty(&request).expect("request example should serialize"),
        serde_json::to_string_pretty(&tool_call).expect("tool call example should serialize"),
        serde_json::to_string_pretty(&progress).expect("progress example should serialize"),
        serde_json::to_string_pretty(&response).expect("response example should serialize"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdump::predicates::{
        content_predicate_keys, metadata_predicate_keys, react_predicate_keys,
        semantic_predicate_keys,
    };

    #[test]
    fn rql_reference_has_examples() {
        let reference = build_rql_reference();
        assert!(!reference.examples.is_empty());
    }

    #[test]
    fn sdk_reference_has_types() {
        let reference = build_sdk_reference();
        assert!(!reference.types.is_empty());
    }

    #[test]
    fn format_rql_reference_text_contains_examples() {
        let text = format_rql_reference_text();
        assert!(text.contains("Examples:"));
    }

    #[test]
    fn sdk_reference_includes_search_options() {
        let reference = build_sdk_reference();
        let has_search_options = reference.types.iter().any(|ty| ty.name == "SearchOptions");
        assert!(has_search_options);
        let has_search_runtime = reference.types.iter().any(|ty| ty.name == "SearchRuntime");
        assert!(has_search_runtime);
        let has_search_backend = reference.types.iter().any(|ty| ty.name == "SearchBackend");
        assert!(has_search_backend);
        let has_backend_metadata = reference
            .types
            .iter()
            .any(|ty| ty.name == "BackendMetadata");
        assert!(has_backend_metadata);
    }

    #[test]
    fn rql_reference_matches_core_predicate_inventory() {
        let reference = build_rql_reference();
        let metadata: Vec<_> = metadata_predicate_keys()
            .into_iter()
            .map(|key| key.as_ref().to_string())
            .filter(|name| name != "path_exact")
            .collect();
        let content: Vec<_> = content_predicate_keys()
            .into_iter()
            .map(|key| key.as_ref().to_string())
            .collect();
        let semantic: Vec<_> = semantic_predicate_keys()
            .into_iter()
            .map(|key| key.as_ref().to_string())
            .collect();
        let react: Vec<_> = react_predicate_keys()
            .into_iter()
            .map(|key| key.as_ref().to_string())
            .collect();

        assert_eq!(reference.metadata_predicates, metadata);
        assert_eq!(reference.content_predicates, content);
        assert_eq!(reference.semantic_predicates, semantic);
        assert_eq!(reference.react_predicates, react);
    }

    #[test]
    fn sdk_reference_includes_current_library_surface() {
        let reference = build_sdk_reference();
        let function_names: Vec<_> = reference
            .functions
            .iter()
            .map(|function| function.name.as_str())
            .collect();

        for expected in [
            "search_iter",
            "search",
            "search_with_stats",
            "search_path_iter",
            "search_paths",
            "explain_query",
            "SearchRuntime::with_backend",
            "execute_search_request_with_runtime",
            "execute_search_request_with_runtime_and_progress",
            "execute_search_request_with_runtime_and_cancellation",
            "repo_language_inventory_with_runtime",
            "search_async_with_runtime",
            "search_async_with_runtime_and_progress",
        ] {
            assert!(function_names.contains(&expected));
        }
    }
}
