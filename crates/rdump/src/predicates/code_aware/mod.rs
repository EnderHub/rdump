use crate::evaluator::{FileContext, MatchResult};
use crate::parser::PredicateKey;
use crate::predicates::PredicateEvaluator;
use anyhow::Result;
use rdump_contracts::SemanticMatchMode;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

mod cache;
mod execution;
pub mod profiles;
mod selection;

#[derive(Debug, Clone, Default)]
pub struct CodeAwareSettings {
    pub sql_dialect: Option<SqlDialect>,
    pub sql_strict: bool,
    pub semantic_budget_ms: Option<u64>,
    pub max_semantic_matches_per_file: Option<usize>,
    pub language_override: Option<String>,
    pub semantic_match_mode: SemanticMatchMode,
    pub semantic_strict: bool,
    pub language_debug: bool,
    pub sql_trace: bool,
    pub telemetry: Option<Arc<SemanticTelemetry>>,
}

#[derive(Debug, Default)]
pub struct SemanticTelemetry {
    parse_failures_by_language: Mutex<BTreeMap<String, usize>>,
    budget_exhaustions: AtomicUsize,
    unsupported_language_skips: AtomicUsize,
    tree_cache_hits: AtomicUsize,
    tree_cache_misses: AtomicUsize,
}

impl SemanticTelemetry {
    pub fn record_parse_failure(&self, profile_key: &str) {
        let mut guard = self
            .parse_failures_by_language
            .lock()
            .expect("semantic telemetry parse-failure lock poisoned");
        *guard.entry(profile_key.to_string()).or_default() += 1;
    }

    pub fn record_budget_exhaustion(&self) {
        self.budget_exhaustions.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_unsupported_language(&self) {
        self.unsupported_language_skips
            .fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_tree_cache_hit(&self) {
        self.tree_cache_hits.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_tree_cache_miss(&self) {
        self.tree_cache_misses.fetch_add(1, Ordering::SeqCst);
    }

    pub fn parse_failures_by_language(&self) -> BTreeMap<String, usize> {
        self.parse_failures_by_language
            .lock()
            .expect("semantic telemetry parse-failure lock poisoned")
            .clone()
    }

    pub fn total_parse_failures(&self) -> usize {
        self.parse_failures_by_language().values().copied().sum()
    }

    pub fn budget_exhaustions(&self) -> usize {
        self.budget_exhaustions.load(Ordering::SeqCst)
    }

    pub fn tree_cache_hits(&self) -> usize {
        self.tree_cache_hits.load(Ordering::SeqCst)
    }

    pub fn tree_cache_misses(&self) -> usize {
        self.tree_cache_misses.load(Ordering::SeqCst)
    }
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

    fn select_language_profile(
        &self,
        extension: &str,
        context: &mut FileContext,
    ) -> Result<Option<(String, &'static profiles::LanguageProfile)>> {
        selection::select_language_profile(&self.settings, extension, context)
    }

    #[cfg(test)]
    fn select_sql_profile(&self, context: &mut FileContext) -> Result<String> {
        selection::select_sql_profile(&self.settings, context)
    }

    fn compiled_query(
        &self,
        profile_key: &str,
        profile: &'static profiles::LanguageProfile,
        key: &PredicateKey,
        query: &str,
    ) -> Result<std::sync::Arc<tree_sitter::Query>> {
        cache::compiled_query(profile_key, profile, key, query)
    }
}

pub fn query_cache_metrics_snapshot() -> (usize, usize) {
    cache::cache_metrics_snapshot()
}

pub fn detect_sql_dialect_for_debug(content: &str) -> Option<SqlDialect> {
    selection::detect_sql_dialect(content)
}

pub fn detect_sql_dialect_trace_for_debug(content: &str) -> (Option<SqlDialect>, String) {
    selection::detect_sql_dialect_with_trace(content)
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
            if let Some(telemetry) = &self.settings.telemetry {
                telemetry.record_unsupported_language();
            }
            context.push_semantic_skip(
                crate::SemanticSkipReason::UnsupportedLanguage,
                format!(
                    "Skipping semantic evaluation for {} because no supported language profile matched.",
                    context.path.display()
                ),
            );
            return Ok(MatchResult::Boolean(false)); // Not a supported language for this predicate.
        };

        // 2. Resolve the execution profile and tree, including SQL fallback when needed.
        let Some(plan) =
            execution::resolve_execution_plan(context, profile_key, profile, &self.settings)?
        else {
            return Ok(MatchResult::Boolean(false));
        };
        profile = plan.profile;

        // 3. Get content and the tree-sitter query for the execution profile.
        let content = context.get_content_arc()?;
        if !context.content_state()?.is_loaded() {
            context.push_semantic_skip(
                crate::SemanticSkipReason::ContentUnavailable,
                format!(
                    "Skipping semantic evaluation for {} because content was not available after safety checks.",
                    context.path.display()
                ),
            );
            return Ok(MatchResult::Boolean(false));
        }
        let ts_query_str = match profile.queries.get(key) {
            Some(q) if !q.is_empty() => q,
            _ => return Ok(MatchResult::Boolean(false)),
        };
        let query = self.compiled_query(&plan.profile_key, profile, key, ts_query_str)?;

        // 4. Execute the query and build match hunks.
        execution::execute_captures(&plan.tree, &content, &query, key, value, &self.settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluator::FileContext;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_sql_dialect_key() {
        assert_eq!(SqlDialect::Generic.key(), "sql");
        assert_eq!(SqlDialect::Postgres.key(), "sqlpg");
        assert_eq!(SqlDialect::Mysql.key(), "sqlmysql");
        assert_eq!(SqlDialect::Sqlite.key(), "sqlsqlite");
    }

    #[test]
    fn test_detect_sql_dialect_mysql() {
        let content = "DELIMITER //\nCREATE PROCEDURE foo() BEGIN END//";
        assert_eq!(
            selection::detect_sql_dialect(content),
            Some(SqlDialect::Mysql)
        );
    }

    #[test]
    fn test_detect_sql_dialect_sqlite() {
        let content = "CREATE TRIGGER foo BEGIN ATOMIC UPDATE t; END;";
        assert_eq!(
            selection::detect_sql_dialect(content),
            Some(SqlDialect::Sqlite)
        );
    }

    #[test]
    fn test_detect_sql_dialect_postgres() {
        let content = "CREATE FUNCTION foo() RETURNS TABLE (id INT) AS $$ BEGIN END $$";
        assert_eq!(
            selection::detect_sql_dialect(content),
            Some(SqlDialect::Postgres)
        );
    }

    #[test]
    fn test_detect_sql_dialect_postgres_plpgsql() {
        let content = "CREATE FUNCTION foo() language plpgsql AS $$ BEGIN END $$";
        assert_eq!(
            selection::detect_sql_dialect(content),
            Some(SqlDialect::Postgres)
        );
    }

    #[test]
    fn test_detect_sql_dialect_generic() {
        let content = "SELECT * FROM users;";
        assert_eq!(selection::detect_sql_dialect(content), None);
    }

    #[test]
    fn test_code_aware_evaluator_new() {
        let settings = CodeAwareSettings::default();
        let evaluator = CodeAwareEvaluator::new(settings);
        assert!(evaluator.settings.sql_dialect.is_none());
    }

    #[test]
    fn test_code_aware_evaluator_with_dialect() {
        let settings = CodeAwareSettings {
            sql_dialect: Some(SqlDialect::Postgres),
            ..Default::default()
        };
        let evaluator = CodeAwareEvaluator::new(settings);
        assert_eq!(evaluator.settings.sql_dialect, Some(SqlDialect::Postgres));
    }

    #[test]
    fn test_select_language_profile_rust() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        let result = evaluator
            .select_language_profile("rs", &mut context)
            .unwrap();
        assert!(result.is_some());
        let (key, profile) = result.unwrap();
        assert_eq!(key, "rs");
        assert_eq!(profile.name, "Rust");
    }

    #[test]
    fn test_select_language_profile_unsupported() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.xyz");
        fs::write(&file_path, "some content").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        let result = evaluator
            .select_language_profile("xyz", &mut context)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_select_language_profile_via_shebang() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("script");
        fs::write(&file_path, "#!/usr/bin/env python3\nprint('hi')\n").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        let result = evaluator.select_language_profile("", &mut context).unwrap();
        assert!(result.is_some());
        let (key, profile) = result.unwrap();
        assert_eq!(key, "py");
        assert_eq!(profile.name, "Python");
    }

    #[test]
    fn test_select_language_profile_sql_with_dialect() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.sql");
        fs::write(&file_path, "SELECT * FROM users;").unwrap();

        let settings = CodeAwareSettings {
            sql_dialect: Some(SqlDialect::Postgres),
            ..Default::default()
        };
        let evaluator = CodeAwareEvaluator::new(settings);
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        let result = evaluator
            .select_language_profile("sql", &mut context)
            .unwrap();
        assert!(result.is_some());
        let (key, _) = result.unwrap();
        assert_eq!(key, "sqlpg");
    }

    #[test]
    fn test_select_sql_profile_cached() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.sql");
        fs::write(&file_path, "SELECT * FROM users;").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        // Pre-set the cached key
        context.set_sql_profile_key("cached_key");

        let result = evaluator.select_sql_profile(&mut context).unwrap();
        assert_eq!(result, "cached_key");
    }

    #[test]
    fn test_evaluate_unsupported_extension() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.xyz");
        fs::write(&file_path, "some content").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Func, "main")
            .unwrap();
        assert!(!result.is_match());
    }

    #[test]
    fn test_evaluate_unsupported_predicate_for_language() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.css");
        fs::write(&file_path, ".class { color: red; }").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        // CSS doesn't support func predicate
        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Func, "main")
            .unwrap();
        assert!(!result.is_match());
    }

    #[test]
    fn test_evaluate_rust_func() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}\nfn helper() {}").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Func, "main")
            .unwrap();
        assert!(result.is_match());
    }

    #[test]
    fn test_evaluate_rust_func_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Func, "nonexistent")
            .unwrap();
        assert!(!result.is_match());
    }

    #[test]
    fn test_evaluate_import_contains() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "use std::collections::HashMap;").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        // Import predicate uses contains matching
        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Import, "HashMap")
            .unwrap();
        assert!(result.is_match());
    }

    #[test]
    fn test_evaluate_comment_contains() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "// TODO: fix this\nfn main() {}").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Comment, "TODO")
            .unwrap();
        assert!(result.is_match());
    }

    #[test]
    fn test_evaluate_string_contains() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "fn main() { let s = \"hello world\"; }").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Str, "hello")
            .unwrap();
        assert!(result.is_match());
    }

    #[test]
    fn test_evaluate_call_contains() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "fn main() { println!(\"hi\"); }").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Call, "println")
            .unwrap();
        assert!(result.is_match());
    }

    #[test]
    fn test_evaluate_wildcard_match() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "fn any_func() {}").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        // Wildcard "." should match any function
        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Func, ".")
            .unwrap();
        assert!(result.is_match());
    }

    #[test]
    fn test_evaluate_no_extension() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("Makefile");
        fs::write(&file_path, "all:\n\techo hello").unwrap();

        let evaluator = CodeAwareEvaluator::new(CodeAwareSettings::default());
        let mut context = FileContext::new(file_path, dir.path().to_path_buf());

        // No extension means unsupported
        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Func, "main")
            .unwrap();
        assert!(!result.is_match());
    }
}
