use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the Elixir language.
pub(super) fn create_elixir_profile() -> LanguageProfile {
    let language = tree_sitter_elixir::LANGUAGE.into();
    let mut queries = HashMap::new();

    // Broad identifier/alias matching to keep queries simple and compatible across grammar versions.
    let ident_query = "[(identifier) @match (alias) @match (atom) @match]";

    queries.insert(PredicateKey::Def, ident_query.to_string());
    queries.insert(PredicateKey::Func, ident_query.to_string());
    queries.insert(PredicateKey::Call, ident_query.to_string());
    queries.insert(PredicateKey::Import, ident_query.to_string());
    queries.insert(PredicateKey::Module, ident_query.to_string());
    queries.insert(PredicateKey::Protocol, ident_query.to_string());

    // Comments / Strings.
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(PredicateKey::Str, "(string) @match".to_string());

    LanguageProfile {
        name: "Elixir",
        extensions: vec!["ex", "exs"],
        language,
        queries,
    }
}
