use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the Lua language.
pub(super) fn create_lua_profile() -> LanguageProfile {
    let language = tree_sitter_lua::LANGUAGE.into();
    let mut queries = HashMap::new();

    // --- Definitions ---
    let func_query = "
    [
        (function_declaration name: (identifier) @match)
        (function_declaration name: (dot_index_expression field: (identifier) @match))
        (function_declaration name: (method_index_expression method: (identifier) @match))
    ]
    ";
    queries.insert(PredicateKey::Def, func_query.to_string());
    queries.insert(PredicateKey::Func, func_query.to_string());

    // --- Imports (require calls)
    queries.insert(
        PredicateKey::Import,
        "(function_call name: (identifier) @match)".to_string(),
    );

    // --- Calls ---
    queries.insert(
        PredicateKey::Call,
        "(function_call name: (identifier) @match)".to_string(),
    );

    // --- Comments / Strings ---
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(PredicateKey::Str, "(string) @match".to_string());

    LanguageProfile {
        name: "Lua",
        extensions: vec!["lua"],
        language,
        queries,
    }
}
