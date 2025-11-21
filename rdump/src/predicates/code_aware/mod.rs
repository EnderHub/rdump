use crate::evaluator::{FileContext, MatchResult};
use crate::parser::PredicateKey;
use crate::predicates::PredicateEvaluator;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

pub mod profiles;

#[derive(Debug, Clone, Default)]
pub struct CodeAwareSettings {
    pub sql_dialect: Option<SqlDialect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    Generic,
    Postgres,
    Mysql,
    Sqlite,
}

impl SqlDialect {
    pub fn key(&self) -> &'static str {
        match self {
            SqlDialect::Generic => "sql",
            SqlDialect::Postgres => "sqlpg",
            SqlDialect::Mysql => "sqlmysql",
            SqlDialect::Sqlite => "sqlsqlite",
        }
    }
}

/// The evaluator that uses tree-sitter to perform code-aware queries.
#[derive(Debug, Clone)]
pub struct CodeAwareEvaluator {
    settings: CodeAwareSettings,
}

impl CodeAwareEvaluator {
    pub fn new(settings: CodeAwareSettings) -> Self {
        Self { settings }
    }

    fn select_language_profile<'a>(
        &'a self,
        extension: &str,
        context: &mut FileContext,
    ) -> Result<Option<(String, &'a profiles::LanguageProfile)>> {
        // SQL gets a dialect-aware path; everything else uses straight extension mapping.
        if extension.eq_ignore_ascii_case("sql") {
            let selected_key = self.select_sql_profile(context)?;
            if let Some(profile) = profiles::get_profile(&selected_key) {
                return Ok(Some((selected_key, profile)));
            }
            return Ok(None);
        }

        // Non-SQL: a simple direct lookup by extension.
        let key = extension.to_lowercase();
        if let Some(profile) = profiles::get_profile(&key) {
            return Ok(Some((key, profile)));
        }

        Ok(None)
    }

    fn select_sql_profile(&self, context: &mut FileContext) -> Result<String> {
        if let Some(cached) = context.sql_profile_key() {
            return Ok(cached.to_string());
        }

        if let Some(dialect) = &self.settings.sql_dialect {
            let key = dialect.key().to_string();
            context.set_sql_profile_key(&key);
            return Ok(key);
        }

        let content = context.get_content()?;
        let lowered = content.to_lowercase();
        let detected = detect_sql_dialect(&lowered);
        let key = detected.unwrap_or(SqlDialect::Generic).key().to_string();
        context.set_sql_profile_key(&key);
        Ok(key)
    }
}

impl PredicateEvaluator for CodeAwareEvaluator {
    fn evaluate(
        &self,
        context: &mut FileContext,
        key: &PredicateKey,
        value: &str,
    ) -> Result<MatchResult> {
        // 1. Determine the language from the file extension.
        let extension = context
            .path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        let Some((profile_key, mut profile)) = self.select_language_profile(&extension, context)?
        else {
            return Ok(MatchResult::Boolean(false)); // Not a supported language for this predicate.
        };

        // 2. Get the tree-sitter query string for the specific predicate.
        let ts_query_str = match profile.queries.get(key) {
            Some(q) if !q.is_empty() => q,
            _ => return Ok(MatchResult::Boolean(false)), // Not a supported language for this predicate.
        };

        // 3. Get content and lazily get the parsed tree from the file context.
        let content = context.get_content()?.to_string(); // Clone to avoid borrow issues
        let tree = match context.get_tree(&profile_key, profile.language.clone()) {
            Ok(tree) => tree,
            Err(e) => {
                if profile_key != SqlDialect::Generic.key() && profile_key.starts_with("sql") {
                    if let Some(generic_profile) = profiles::get_profile(SqlDialect::Generic.key())
                    {
                        context.set_sql_profile_key(SqlDialect::Generic.key());
                        match context
                            .get_tree(SqlDialect::Generic.key(), generic_profile.language.clone())
                        {
                            Ok(tree) => {
                                profile = generic_profile;
                                tree
                            }
                            Err(fallback_err) => {
                                eprintln!(
                                    "Warning: Failed to parse {} with {} and fallback: {e}; {fallback_err}. Skipping.",
                                    context.path.display(),
                                    profile_key
                                );
                                return Ok(MatchResult::Boolean(false));
                            }
                        }
                    } else {
                        eprintln!(
                            "Warning: Failed to parse {} with {} and no SQL fallback available: {}.",
                            context.path.display(),
                            profile_key,
                            e
                        );
                        return Ok(MatchResult::Boolean(false));
                    }
                } else {
                    eprintln!(
                        "Warning: Failed to parse {}: {}. Skipping.",
                        context.path.display(),
                        e
                    );
                    return Ok(MatchResult::Boolean(false));
                }
            }
        };

        // 4. Compile the tree-sitter query.
        let query = Query::new(&profile.language, ts_query_str)
            .with_context(|| format!("Failed to compile tree-sitter query for key {key:?}"))?;
        let mut cursor = QueryCursor::new();
        let mut ranges = Vec::new();

        // 5. Execute the query and check for a match.
        let mut captures = cursor.matches(&query, tree.root_node(), content.as_bytes());

        while let Some(m) = captures.next() {
            for capture in m.captures {
                // We only care about nodes captured with the name `@match`.
                let capture_name = &query.capture_names()[capture.index as usize];
                if *capture_name != "match" {
                    continue;
                }

                let captured_node = capture.node;
                let captured_text = captured_node.utf8_text(content.as_bytes())?;

                // Use the correct matching strategy based on the predicate type.
                let is_match = match key {
                    // Content-based predicates check for substrings.
                    PredicateKey::Import | PredicateKey::Comment | PredicateKey::Str => {
                        captured_text.contains(value)
                    }
                    // Hook predicates can match any hook (`hook:.`) or a specific one
                    PredicateKey::Hook | PredicateKey::CustomHook => {
                        value == "." || captured_text == value
                    }
                    // Calls: allow substring match since callee may include arguments/qualifiers.
                    PredicateKey::Call => captured_text.contains(value),
                    // Definition-based predicates require an exact match on the identifier, unless a wildcard is used.
                    _ => value == "." || captured_text == value,
                };

                if is_match {
                    ranges.push(captured_node.range());
                }
            }
        }

        Ok(MatchResult::Hunks(ranges))
    }
}

static MYSQL_DELIMITER_RE: Lazy<Regex> = Lazy::new(|| Regex::new("(?i)delimiter\\s+//").unwrap());
static SQLITE_BEGIN_ATOMIC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("(?i)begin\\s+atomic").unwrap());
static POSTGRES_RETURNS_TABLE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("(?i)returns\\s+table").unwrap());

fn detect_sql_dialect(content: &str) -> Option<SqlDialect> {
    if MYSQL_DELIMITER_RE.is_match(content) {
        return Some(SqlDialect::Mysql);
    }
    if SQLITE_BEGIN_ATOMIC_RE.is_match(content) {
        return Some(SqlDialect::Sqlite);
    }
    if POSTGRES_RETURNS_TABLE_RE.is_match(content) || content.contains("language plpgsql") {
        return Some(SqlDialect::Postgres);
    }
    None
}
