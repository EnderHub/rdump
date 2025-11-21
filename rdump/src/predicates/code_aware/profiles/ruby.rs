use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the Ruby language.
pub(super) fn create_ruby_profile() -> LanguageProfile {
    let language = tree_sitter_ruby::LANGUAGE.into();
    let mut queries = HashMap::new();

    // --- Definitions ---
    let class_query = "(class name: (constant) @match)";
    let module_query = "(module name: (constant) @match)";
    let func_query = "
    [
      (method name: (identifier) @match)
      (singleton_method name: (identifier) @match)
    ]
    ";

    queries.insert(
        PredicateKey::Def,
        [class_query, module_query, func_query].join("\n"),
    );
    queries.insert(PredicateKey::Class, class_query.to_string());
    queries.insert(PredicateKey::Type, module_query.to_string());
    queries.insert(PredicateKey::Func, func_query.to_string());

    // --- Imports ---
    // Ruby import-like statements (`require`, `require_relative`) are method calls; we approximate by matching identifiers.
    queries.insert(PredicateKey::Import, "(identifier) @match".to_string());

    // --- Calls ---
    queries.insert(
        PredicateKey::Call,
        "(call method: (identifier) @match)".to_string(),
    );

    // --- Comments / Strings ---
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(
        PredicateKey::Str,
        "[ (string) @match (heredoc_body) @match ]".to_string(),
    );

    LanguageProfile {
        name: "Ruby",
        extensions: vec!["rb"],
        language,
        queries,
    }
}
