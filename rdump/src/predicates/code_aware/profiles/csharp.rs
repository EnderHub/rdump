use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the C# language.
pub(super) fn create_csharp_profile() -> LanguageProfile {
    let language = tree_sitter_c_sharp::LANGUAGE.into();
    let mut queries = HashMap::new();

    // --- Definitions ---
    let class_query = "(class_declaration name: (identifier) @match)";
    let struct_query = "(struct_declaration name: (identifier) @match)";
    let enum_query = "(enum_declaration name: (identifier) @match)";
    let interface_query = "(interface_declaration name: (identifier) @match)";
    let namespace_query = "(namespace_declaration name: (identifier) @match)";

    let func_query = "
    [
        (method_declaration name: (identifier) @match)
        (local_function_statement name: (identifier) @match)
    ]
    ";

    queries.insert(
        PredicateKey::Def,
        [
            class_query,
            struct_query,
            enum_query,
            interface_query,
            namespace_query,
            func_query,
        ]
        .join("\n"),
    );
    queries.insert(PredicateKey::Class, class_query.to_string());
    queries.insert(PredicateKey::Struct, struct_query.to_string());
    queries.insert(PredicateKey::Enum, enum_query.to_string());
    queries.insert(PredicateKey::Interface, interface_query.to_string());
    queries.insert(PredicateKey::Type, namespace_query.to_string());
    queries.insert(PredicateKey::Func, func_query.to_string());

    // --- Imports ---
    queries.insert(PredicateKey::Import, "(using_directive) @match".to_string());

    // --- Calls ---
    queries.insert(
        PredicateKey::Call,
        "(invocation_expression function: [ (identifier) @match (member_access_expression name: (identifier) @match) ])"
            .to_string(),
    );

    // --- Comments / Strings ---
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(
        PredicateKey::Str,
        "[ (string_literal) @match (verbatim_string_literal) @match (interpolated_string_expression) @match ]"
            .to_string(),
    );

    LanguageProfile {
        name: "C#",
        extensions: vec!["cs", "csx"],
        language,
        queries,
    }
}
