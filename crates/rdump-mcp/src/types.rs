pub use rdump_contracts::{
    CapabilityMetadata, ContractError, ErrorCode, ErrorEnvelope, ErrorMode, FieldDoc, FunctionDoc,
    LanguageCapabilityMatrix, LanguageInfo, LanguagePredicates, LimitValue, Limits, LineEndingMode,
    MatchInfo, OutputMode, PathDisplayMode, PredicateCatalog, ProgressEvent, RqlReference,
    SdkReference, SearchDiagnostic, SearchItem, SearchRequest, SearchResponse, SearchStats,
    SemanticMatchMode, Snippet, SnippetMode, SqlDialectOption, TypeDoc, ValidateQueryResponse,
    SCHEMA_VERSION,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchArgs {
    pub query: Option<String>,
    pub root: Option<String>,
    pub presets: Option<Vec<String>>,
    pub no_ignore: Option<bool>,
    pub hidden: Option<bool>,
    pub max_depth: Option<usize>,
    pub sql_dialect: Option<SqlDialectOption>,
    pub sql_strict: Option<bool>,
    pub output: Option<OutputMode>,
    pub limits: Option<Limits>,
    pub context_lines: Option<usize>,
    pub error_mode: Option<ErrorMode>,
    pub skip_errors: Option<bool>,
    pub execution_budget_ms: Option<u64>,
    pub semantic_budget_ms: Option<u64>,
    pub max_semantic_matches_per_file: Option<usize>,
    pub language_override: Option<String>,
    pub semantic_match_mode: Option<SemanticMatchMode>,
    pub snippet_mode: Option<SnippetMode>,
    pub semantic_strict: Option<bool>,
    pub strict_path_resolution: Option<bool>,
    pub snapshot_drift_detection: Option<bool>,
    pub ignore_debug: Option<bool>,
    pub language_debug: Option<bool>,
    pub sql_trace: Option<bool>,
    pub execution_profile: Option<rdump_contracts::ExecutionProfile>,
    pub offset: Option<usize>,
    pub continuation_token: Option<String>,
    pub path_display: Option<PathDisplayMode>,
    pub line_endings: Option<LineEndingMode>,
    pub include_match_text: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DescribeLanguageArgs {
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateQueryArgs {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExplainQueryArgs {
    pub query: String,
    #[serde(default)]
    pub presets: Vec<String>,
}
