use rdump::SqlDialect;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Paths,
    Matches,
    Snippets,
    Full,
    Summary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SqlDialectOption {
    Generic,
    Postgres,
    Mysql,
    Sqlite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitValue {
    Unset,
    Unlimited,
    Value(usize),
}

impl Default for LimitValue {
    fn default() -> Self {
        LimitValue::Unset
    }
}

impl Serialize for LimitValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            LimitValue::Value(value) => serializer.serialize_u64(*value as u64),
            LimitValue::Unlimited | LimitValue::Unset => serializer.serialize_none(),
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
            Some(inner) => LimitValue::Value(inner),
            None => LimitValue::Unlimited,
        })
    }
}

impl JsonSchema for LimitValue {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("LimitValue")
    }

    fn json_schema(gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        <Option<usize>>::json_schema(gen)
    }
}

impl From<SqlDialectOption> for SqlDialect {
    fn from(value: SqlDialectOption) -> Self {
        match value {
            SqlDialectOption::Generic => SqlDialect::Generic,
            SqlDialectOption::Postgres => SqlDialect::Postgres,
            SqlDialectOption::Mysql => SqlDialect::Mysql,
            SqlDialectOption::Sqlite => SqlDialect::Sqlite,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct Limits {
    /// Maximum number of files to return. Use null for unlimited.
    #[serde(default)]
    pub max_results: LimitValue,
    /// Maximum matches per file. Use null for unlimited.
    #[serde(default)]
    pub max_matches_per_file: LimitValue,
    /// Maximum bytes of content per file (full output). Use null for unlimited.
    #[serde(default)]
    pub max_bytes_per_file: LimitValue,
    /// Maximum bytes across all returned content. Use null for unlimited.
    #[serde(default)]
    pub max_total_bytes: LimitValue,
    /// Maximum bytes for each match text. Use null for unlimited.
    #[serde(default)]
    pub max_match_bytes: LimitValue,
    /// Maximum bytes for each snippet. Use null for unlimited.
    #[serde(default)]
    pub max_snippet_bytes: LimitValue,
    /// Maximum number of error strings to return. Use null for unlimited.
    #[serde(default)]
    pub max_errors: LimitValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchRequest {
    /// RQL query string (example: "ext:rs & func:main").
    pub query: String,
    /// Root directory to search from (defaults to ".").
    #[serde(default)]
    pub root: Option<String>,
    /// Named presets to apply.
    #[serde(default)]
    pub presets: Vec<String>,
    /// Ignore .gitignore rules if true.
    #[serde(default)]
    pub no_ignore: bool,
    /// Include hidden files and directories.
    #[serde(default)]
    pub hidden: bool,
    /// Maximum directory depth.
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// SQL dialect override for .sql files.
    #[serde(default)]
    pub sql_dialect: Option<SqlDialectOption>,
    /// Output shaping mode (paths, matches, snippets, full, summary).
    #[serde(default)]
    pub output: Option<OutputMode>,
    /// Result and size limits. Omitted fields use defaults.
    #[serde(default)]
    pub limits: Option<Limits>,
    /// Context lines for snippet output.
    #[serde(default)]
    pub context_lines: Option<usize>,
    /// Skip per-file errors instead of failing the request.
    #[serde(default)]
    pub skip_errors: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchArgs {
    /// RQL query string (example: "ext:rs & func:main"). Optional but required to execute.
    pub query: Option<String>,
    /// Root directory to search from (defaults to ".").
    pub root: Option<String>,
    /// Named presets to apply.
    pub presets: Option<Vec<String>>,
    /// Ignore .gitignore rules if true.
    pub no_ignore: Option<bool>,
    /// Include hidden files and directories.
    pub hidden: Option<bool>,
    /// Maximum directory depth.
    pub max_depth: Option<usize>,
    /// SQL dialect override for .sql files.
    pub sql_dialect: Option<SqlDialectOption>,
    /// Output shaping mode (paths, matches, snippets, full, summary).
    pub output: Option<OutputMode>,
    /// Result and size limits. Set any field to null for unlimited.
    pub limits: Option<Limits>,
    /// Context lines for snippet output.
    pub context_lines: Option<usize>,
    /// Skip per-file errors instead of failing the request.
    pub skip_errors: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DescribeLanguageArgs {
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateQueryArgs {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ValidateQueryResponse {
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SearchStats {
    pub returned_files: usize,
    pub returned_matches: usize,
    pub returned_bytes: usize,
    pub errors: usize,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SearchResponse {
    pub query: String,
    pub root: String,
    pub output: OutputMode,
    pub results: Vec<SearchItem>,
    pub stats: SearchStats,
    pub errors: Vec<String>,
    pub errors_truncated: bool,
    pub truncated: bool,
    pub truncation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SearchItem {
    Path { path: String },
    Summary {
        path: String,
        matches: usize,
        whole_file_match: bool,
        matches_truncated: bool,
    },
    Matches {
        path: String,
        matches: Vec<MatchInfo>,
        whole_file_match: bool,
        matches_truncated: bool,
    },
    Snippets {
        path: String,
        snippets: Vec<Snippet>,
        whole_file_match: bool,
        matches_truncated: bool,
    },
    Full {
        path: String,
        content: String,
        matches: Vec<MatchInfo>,
        content_truncated: bool,
        matches_truncated: bool,
    },
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MatchInfo {
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub byte_range: [usize; 2],
    pub text: Option<String>,
    pub text_truncated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Snippet {
    pub start_line: usize,
    pub end_line: usize,
    pub match_start_line: usize,
    pub match_end_line: usize,
    pub text: String,
    pub text_truncated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct LanguagePredicates {
    pub metadata: Vec<String>,
    pub content: Vec<String>,
    pub semantic: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct LanguageInfo {
    pub name: String,
    pub extensions: Vec<String>,
    pub predicates: LanguagePredicates,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RqlReference {
    pub operators: Vec<String>,
    pub notes: Vec<String>,
    pub metadata_predicates: Vec<String>,
    pub content_predicates: Vec<String>,
    pub semantic_predicates: Vec<String>,
    pub react_predicates: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct FieldDoc {
    pub name: String,
    pub description: String,
    pub default: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct FunctionDoc {
    pub name: String,
    pub signature: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TypeDoc {
    pub name: String,
    pub description: String,
    pub fields: Vec<FieldDoc>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SdkReference {
    pub functions: Vec<FunctionDoc>,
    pub types: Vec<TypeDoc>,
    pub notes: Vec<String>,
}
