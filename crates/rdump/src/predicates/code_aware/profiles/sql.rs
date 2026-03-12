use super::LanguageProfile;
use crate::parser::PredicateKey;
use crate::predicates::code_aware::SqlDialect;
use std::collections::HashMap;

fn base_sql_queries() -> HashMap<PredicateKey, String> {
    let mut queries = HashMap::new();

    let table_query = "(create_table (object_reference name: (identifier) @match))";
    let view_query = "(create_view (object_reference name: (identifier) @match))";
    let mat_view_query = "(create_materialized_view (object_reference name: (identifier) @match))";
    let function_query = "(create_function (object_reference name: (identifier) @match))";

    queries.insert(
        PredicateKey::Def,
        [table_query, view_query, mat_view_query, function_query].join("\n"),
    );
    queries.insert(PredicateKey::Func, function_query.to_string());

    queries.insert(
        PredicateKey::Import,
        "(object_reference name: (identifier) @match)".to_string(),
    );

    queries.insert(
        PredicateKey::Call,
        "(invocation (object_reference name: (identifier) @match))".to_string(),
    );
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(PredicateKey::Str, "(literal) @match".to_string());

    queries
}

fn sql_language() -> tree_sitter::Language {
    // tree-sitter-sequel exports LANGUAGE as a LanguageFn; convert to a Language
    tree_sitter_sequel::LANGUAGE.into()
}

fn extensions_for_dialect(dialect: SqlDialect) -> Vec<&'static str> {
    match dialect {
        SqlDialect::Generic => vec!["sql"],
        SqlDialect::Postgres => vec!["psql", "pgsql"],
        SqlDialect::Mysql => vec!["mysql"],
        SqlDialect::Sqlite => vec!["sqlite"],
    }
}

fn make_profile(dialect: SqlDialect, display_name: &'static str) -> LanguageProfile {
    LanguageProfile {
        name: display_name,
        extensions: extensions_for_dialect(dialect),
        language: sql_language(),
        queries: base_sql_queries(),
    }
}

pub(super) fn create_generic_profile() -> LanguageProfile {
    make_profile(SqlDialect::Generic, "SQL (Generic)")
}

pub(super) fn create_postgres_profile() -> LanguageProfile {
    make_profile(SqlDialect::Postgres, "SQL (Postgres)")
}

pub(super) fn create_mysql_profile() -> LanguageProfile {
    make_profile(SqlDialect::Mysql, "SQL (MySQL)")
}

pub(super) fn create_sqlite_profile() -> LanguageProfile {
    make_profile(SqlDialect::Sqlite, "SQL (SQLite)")
}
