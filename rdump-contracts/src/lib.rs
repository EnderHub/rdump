use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA_VERSION: &str = "rdump.v1";

const fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchStatus {
    FullSuccess,
    PartialSuccess,
    TruncatedSuccess,
    InvalidQuery,
    PolicySuppressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StabilityTier {
    Stable,
    Provisional,
    Deprecated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SurfaceStability {
    pub surface: String,
    pub tier: StabilityTier,
    pub semver_notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LanguageSupportTier {
    Stable,
    Partial,
    Experimental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResultKind {
    WholeFile,
    Ranged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PathResolution {
    Canonical,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FileIdentity {
    pub display_path: String,
    pub resolved_path: String,
    pub root_relative_path: Option<String>,
    pub resolution: PathResolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PathMetadata {
    pub size_bytes: u64,
    pub modified_unix_millis: Option<i64>,
    pub readonly: bool,
    pub permissions_display: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SemanticSkipReason {
    UnsupportedLanguage,
    ParseFailed,
    ContentUnavailable,
    BudgetExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MatchCoordinateSemantics {
    pub line_numbers: String,
    pub columns: String,
    pub byte_ranges: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionProfile {
    Interactive,
    Batch,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Paths,
    Matches,
    Snippets,
    Full,
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SqlDialectOption {
    Generic,
    Postgres,
    Mysql,
    Sqlite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorMode {
    SkipErrors,
    FailFast,
}

impl Default for ErrorMode {
    fn default() -> Self {
        Self::SkipErrors
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SnippetMode {
    Normalized,
    PreserveLineEndings,
}

impl Default for SnippetMode {
    fn default() -> Self {
        Self::PreserveLineEndings
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SemanticMatchMode {
    Exact,
    CaseInsensitive,
    Prefix,
    Regex,
    Wildcard,
}

impl Default for SemanticMatchMode {
    fn default() -> Self {
        Self::Exact
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
pub enum LimitValue {
    Unset,
    Unlimited,
    Value(usize),
}

impl Default for LimitValue {
    fn default() -> Self {
        Self::Unset
    }
}

impl Serialize for LimitValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Value(value) => serializer.serialize_u64(*value as u64),
            Self::Unlimited | Self::Unset => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for LimitValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Option::<usize>::deserialize(deserializer)?;
        Ok(match value {
            Some(value) => Self::Value(value),
            None => Self::Unlimited,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Limits {
    #[serde(default)]
    pub max_results: LimitValue,
    #[serde(default)]
    pub max_matches_per_file: LimitValue,
    #[serde(default)]
    pub max_bytes_per_file: LimitValue,
    #[serde(default)]
    pub max_total_bytes: LimitValue,
    #[serde(default)]
    pub max_match_bytes: LimitValue,
    #[serde(default)]
    pub max_snippet_bytes: LimitValue,
    #[serde(default)]
    pub max_errors: LimitValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PathDisplayMode {
    Relative,
    Absolute,
    RootRelative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LineEndingMode {
    Preserve,
    Normalize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub presets: Vec<String>,
    #[serde(default)]
    pub no_ignore: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub sql_dialect: Option<SqlDialectOption>,
    #[serde(default)]
    pub sql_strict: bool,
    #[serde(default)]
    pub output: Option<OutputMode>,
    #[serde(default)]
    pub limits: Option<Limits>,
    #[serde(default)]
    pub context_lines: Option<usize>,
    #[serde(default)]
    pub error_mode: ErrorMode,
    #[serde(default)]
    pub execution_budget_ms: Option<u64>,
    #[serde(default)]
    pub semantic_budget_ms: Option<u64>,
    #[serde(default)]
    pub max_semantic_matches_per_file: Option<usize>,
    #[serde(default)]
    pub language_override: Option<String>,
    #[serde(default)]
    pub semantic_match_mode: SemanticMatchMode,
    #[serde(default)]
    pub snippet_mode: SnippetMode,
    #[serde(default)]
    pub semantic_strict: bool,
    #[serde(default)]
    pub strict_path_resolution: bool,
    #[serde(default)]
    pub execution_profile: Option<ExecutionProfile>,
    #[serde(default)]
    pub offset: usize,
    #[serde(default)]
    pub continuation_token: Option<String>,
    #[serde(default = "default_true")]
    pub snapshot_drift_detection: bool,
    #[serde(default)]
    pub ignore_debug: bool,
    #[serde(default)]
    pub language_debug: bool,
    #[serde(default)]
    pub sql_trace: bool,
    #[serde(default)]
    pub path_display: Option<PathDisplayMode>,
    #[serde(default)]
    pub line_endings: Option<LineEndingMode>,
    #[serde(default = "default_true")]
    pub include_match_text: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    QuerySyntax,
    QueryValidation,
    Config,
    SearchExecution,
    SearchBudgetExceeded,
    SearchCancelled,
    AsyncJoin,
    UnsupportedLanguage,
    Internal,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ErrorRemediation {
    pub retryable: bool,
    pub suggested_action: Option<String>,
    pub docs_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContractError {
    pub code: ErrorCode,
    pub message: String,
    pub field: Option<String>,
    pub remediation: ErrorRemediation,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SearchStats {
    pub returned_files: usize,
    pub returned_matches: usize,
    pub returned_bytes: usize,
    pub errors: usize,
    pub whole_file_results: usize,
    pub ranged_results: usize,
    pub candidate_files: usize,
    pub prefiltered_files: usize,
    pub evaluated_files: usize,
    pub matched_files: usize,
    pub matched_ranges: usize,
    pub hidden_skipped: usize,
    pub ignore_skipped: usize,
    pub max_depth_skipped: usize,
    pub unreadable_entries: usize,
    pub root_boundary_excluded: usize,
    pub suppressed_too_large: usize,
    pub suppressed_binary: usize,
    pub suppressed_secret_like: usize,
    pub diagnostics: usize,
    pub walk_millis: u64,
    pub prefilter_millis: u64,
    pub evaluate_millis: u64,
    pub materialize_millis: u64,
    pub semantic_parse_failures: usize,
    pub semantic_budget_exhaustions: usize,
    pub query_cache_hits: usize,
    pub query_cache_misses: usize,
    pub tree_cache_hits: usize,
    pub tree_cache_misses: usize,
    pub semaphore_wait_millis: u64,
    pub semantic_parse_failures_by_language: BTreeMap<String, usize>,
    pub directory_hotspots: Vec<DirectoryHotspot>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DirectoryHotspot {
    pub path: String,
    pub candidate_files: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SearchDiagnostic {
    pub level: String,
    pub kind: String,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SearchResponse {
    pub schema_version: String,
    pub schema_reference: String,
    pub status: SearchStatus,
    pub coordinate_semantics: MatchCoordinateSemantics,
    pub query: String,
    pub effective_query: String,
    pub root: String,
    pub output: OutputMode,
    pub error_mode: ErrorMode,
    pub results: Vec<SearchItem>,
    pub stats: SearchStats,
    pub diagnostics: Vec<SearchDiagnostic>,
    pub errors: Vec<ContractError>,
    pub truncated: bool,
    pub truncation_reason: Option<String>,
    pub next_offset: Option<usize>,
    pub continuation_token: Option<String>,
    pub page_size: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SearchItem {
    Path {
        path: String,
        file: FileIdentity,
        fingerprint: String,
        metadata: PathMetadata,
        result_kind: ResultKind,
        item_truncated: bool,
    },
    Summary {
        path: String,
        file: FileIdentity,
        fingerprint: String,
        matches: usize,
        whole_file_match: bool,
        result_kind: ResultKind,
        matches_truncated: bool,
        content_state: Option<String>,
        diagnostic_count: usize,
        semantic_skip_reasons: Vec<SemanticSkipReason>,
        item_truncated: bool,
    },
    Matches {
        path: String,
        file: FileIdentity,
        fingerprint: String,
        matches: Vec<MatchInfo>,
        whole_file_match: bool,
        result_kind: ResultKind,
        matches_truncated: bool,
        content_state: Option<String>,
        diagnostic_count: usize,
        semantic_skip_reasons: Vec<SemanticSkipReason>,
        item_truncated: bool,
    },
    Snippets {
        path: String,
        file: FileIdentity,
        fingerprint: String,
        snippets: Vec<Snippet>,
        whole_file_match: bool,
        result_kind: ResultKind,
        matches_truncated: bool,
        content_state: Option<String>,
        diagnostic_count: usize,
        semantic_skip_reasons: Vec<SemanticSkipReason>,
        item_truncated: bool,
    },
    Full {
        path: String,
        file: FileIdentity,
        fingerprint: String,
        content: String,
        matches: Vec<MatchInfo>,
        content_truncated: bool,
        matches_truncated: bool,
        result_kind: ResultKind,
        content_state: Option<String>,
        diagnostic_count: usize,
        semantic_skip_reasons: Vec<SemanticSkipReason>,
        item_truncated: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MatchInfo {
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub byte_range: [usize; 2],
    pub text: Option<String>,
    pub text_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Snippet {
    pub start_line: usize,
    pub end_line: usize,
    pub match_start_line: usize,
    pub match_end_line: usize,
    pub text: String,
    pub text_truncated: bool,
    pub line_ending: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LanguagePredicates {
    pub metadata: Vec<String>,
    pub content: Vec<String>,
    pub semantic: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LanguageInfo {
    pub id: String,
    pub name: String,
    pub extensions: Vec<String>,
    pub aliases: Vec<String>,
    pub support_tier: LanguageSupportTier,
    pub predicates: LanguagePredicates,
    pub semantic_caveats: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PredicateDescriptor {
    pub name: String,
    pub category: String,
    pub aliases: Vec<String>,
    pub deprecated_aliases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PredicateCatalog {
    pub schema_version: String,
    pub predicates: Vec<PredicateDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LanguageCapabilityMatrix {
    pub schema_version: String,
    pub capture_convention: String,
    pub languages: Vec<LanguageInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RqlReference {
    pub schema_version: String,
    pub operators: Vec<String>,
    pub notes: Vec<String>,
    pub aliases: Vec<String>,
    pub deprecated_predicates: Vec<String>,
    pub metadata_predicates: Vec<String>,
    pub content_predicates: Vec<String>,
    pub semantic_predicates: Vec<String>,
    pub react_predicates: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FieldDoc {
    pub name: String,
    pub description: String,
    pub default: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FunctionDoc {
    pub name: String,
    pub signature: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TypeDoc {
    pub name: String,
    pub description: String,
    pub fields: Vec<FieldDoc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SdkReference {
    pub schema_version: String,
    pub functions: Vec<FunctionDoc>,
    pub types: Vec<TypeDoc>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ValidateQueryResponse {
    pub schema_version: String,
    pub valid: bool,
    pub normalized_query: Option<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<ContractError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CapabilityMetadata {
    pub schema_version: String,
    pub supported_outputs: Vec<OutputMode>,
    pub default_context_lines: usize,
    pub stability: Vec<SurfaceStability>,
    pub default_limits: Limits,
    pub coordinate_semantics: MatchCoordinateSemantics,
    pub execution_profiles: Vec<ExecutionProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ErrorEnvelope {
    pub schema_version: String,
    pub status: SearchStatus,
    pub error: ContractError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProgressEvent {
    Started {
        session_id: String,
        query: String,
        effective_query: String,
        root: String,
        queue_wait_millis: u64,
    },
    Phase {
        session_id: String,
        name: String,
        completed_items: usize,
        total_items: Option<usize>,
    },
    Result {
        session_id: String,
        path: String,
        emitted_results: usize,
    },
    Finished {
        session_id: String,
        returned_files: usize,
        returned_matches: usize,
        truncated: bool,
    },
}
