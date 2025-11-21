use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the PHP language.
pub(super) fn create_php_profile() -> LanguageProfile {
    let language = tree_sitter_php::LANGUAGE_PHP.into();
    let mut queries = HashMap::new();

    // --- Definitions ---
    let class_query = "(class_declaration name: (name) @match)";
    let interface_query = "(interface_declaration name: (name) @match)";
    let trait_query = "(trait_declaration name: (name) @match)";
    let func_query = "(function_definition name: (name) @match)";

    queries.insert(
        PredicateKey::Def,
        [class_query, interface_query, trait_query, func_query].join("\n"),
    );
    queries.insert(PredicateKey::Class, class_query.to_string());
    queries.insert(PredicateKey::Interface, interface_query.to_string());
    queries.insert(PredicateKey::Trait, trait_query.to_string());
    queries.insert(PredicateKey::Func, func_query.to_string());

    // --- Imports ---
    queries.insert(PredicateKey::Import, "(qualified_name) @match".to_string());

    // --- Calls ---
    queries.insert(
        PredicateKey::Call,
        "
        [
          (function_call_expression) @match
          (member_call_expression) @match
          (scoped_call_expression) @match
        ]
        "
        .to_string(),
    );

    // --- Comments / Strings ---
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(
        PredicateKey::Str,
        "[ (string) @match (encapsed_string) @match (heredoc) @match ]".to_string(),
    );

    LanguageProfile {
        name: "PHP",
        extensions: vec!["php", "phtml"],
        language,
        queries,
    }
}
