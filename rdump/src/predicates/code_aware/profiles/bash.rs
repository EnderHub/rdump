use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for shell scripts (Bash).
pub(super) fn create_bash_profile() -> LanguageProfile {
    let language = tree_sitter_bash::LANGUAGE.into();
    let mut queries = HashMap::new();

    // --- Definitions ---
    let func_query = "(function_definition name: (word) @match)";
    queries.insert(PredicateKey::Def, func_query.to_string());
    queries.insert(PredicateKey::Func, func_query.to_string());

    // --- Imports (source) ---
    queries.insert(PredicateKey::Import, "(command) @match".to_string());

    // --- Calls ---
    queries.insert(PredicateKey::Call, "(command) @match".to_string());

    // --- Comments / Strings ---
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(
        PredicateKey::Str,
        "[ (string) @match (raw_string) @match ]".to_string(),
    );

    LanguageProfile {
        name: "Bash",
        extensions: vec!["sh", "bash"],
        language,
        queries,
    }
}
