use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the Scala language.
pub(super) fn create_scala_profile() -> LanguageProfile {
    let language = tree_sitter_scala::LANGUAGE.into();
    let mut queries = HashMap::new();

    // --- Definitions ---
    let class_query = "(class_definition name: (identifier) @match)";
    let object_query = "(object_definition name: (identifier) @match)";
    let trait_query = "(trait_definition name: (identifier) @match)";
    let func_query = "(function_definition name: (identifier) @match)";

    queries.insert(
        PredicateKey::Def,
        [class_query, object_query, trait_query, func_query].join("\n"),
    );
    queries.insert(PredicateKey::Class, class_query.to_string());
    queries.insert(PredicateKey::Type, object_query.to_string());
    queries.insert(PredicateKey::Object, object_query.to_string());
    queries.insert(PredicateKey::Trait, trait_query.to_string());
    queries.insert(PredicateKey::Func, func_query.to_string());

    // --- Imports ---
    queries.insert(
        PredicateKey::Import,
        "(import_declaration) @match".to_string(),
    );

    // --- Calls ---
    // Calls: allow any call_expression match; caller filtered by substring later.
    queries.insert(PredicateKey::Call, "(call_expression) @match".to_string());

    // --- Comments / Strings ---
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    // String predicate not yet implemented for Scala grammar; leave empty.
    queries.insert(PredicateKey::Str, "".to_string());

    LanguageProfile {
        name: "Scala",
        extensions: vec!["scala"],
        language,
        queries,
    }
}
