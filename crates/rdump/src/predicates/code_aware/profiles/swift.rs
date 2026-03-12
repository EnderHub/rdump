use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the Swift language.
pub(super) fn create_swift_profile() -> LanguageProfile {
    let language = tree_sitter_swift::LANGUAGE.into();
    let mut queries = HashMap::new();

    // --- Definitions ---
    let class_query = "(class_declaration name: (type_identifier) @match)";

    let func_query = "(function_declaration name: (simple_identifier) @match)";
    let call_query = "(call_expression) @match";

    let protocol_query = "(protocol_declaration name: (type_identifier) @match)";

    queries.insert(
        PredicateKey::Def,
        [class_query, func_query, protocol_query].join("\n"),
    );
    queries.insert(PredicateKey::Class, class_query.to_string());
    queries.insert(PredicateKey::Protocol, protocol_query.to_string());
    // Other type-level predicates (struct/enum/extension) omitted due to grammar tokens-only nodes.
    queries.insert(PredicateKey::Func, func_query.to_string());

    // --- Imports ---
    queries.insert(
        PredicateKey::Import,
        "(import_declaration) @match".to_string(),
    );

    // --- Calls ---
    queries.insert(PredicateKey::Call, call_query.to_string());

    // --- Comments / Strings ---
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(
        PredicateKey::Str,
        "[ (line_string_literal) @match (multi_line_string_literal) @match (raw_string_literal) @match ]"
            .to_string(),
    );

    LanguageProfile {
        name: "Swift",
        extensions: vec!["swift"],
        language,
        queries,
    }
}
