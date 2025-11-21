use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the C++ language.
pub(super) fn create_cpp_profile() -> LanguageProfile {
    let language = tree_sitter_cpp::LANGUAGE.into();
    let mut queries = HashMap::new();

    // --- Definitions ---
    let class_query = "(class_specifier name: (type_identifier) @match)";
    let struct_query = "(struct_specifier name: (type_identifier) @match)";
    let enum_query = "(enum_specifier name: (type_identifier) @match)";

    // Function / method definitions.
    let func_query = "
    [
        (function_definition
            declarator: (function_declarator
                declarator: (identifier) @match))
        (function_definition
            declarator: (pointer_declarator
                declarator: (function_declarator
                    declarator: (identifier) @match)))
        (function_definition
            declarator: (field_identifier) @match) ; method definitions
    ]
    ";

    let macro_query = "[(preproc_def name: (identifier) @match) (preproc_function_def name: (identifier) @match)]";

    queries.insert(
        PredicateKey::Def,
        [class_query, struct_query, enum_query, func_query].join("\n"),
    );
    queries.insert(PredicateKey::Class, class_query.to_string());
    queries.insert(PredicateKey::Struct, struct_query.to_string());
    queries.insert(PredicateKey::Enum, enum_query.to_string());
    // Type alias queries omitted to keep grammar-compat simple.
    queries.insert(PredicateKey::Func, func_query.to_string());
    queries.insert(PredicateKey::Macro, macro_query.to_string());

    // --- Calls & Imports ---
    queries.insert(
        PredicateKey::Call,
        "(call_expression function: [ (identifier) @match (field_expression field: (field_identifier) @match) (qualified_identifier name: (identifier) @match) ])"
            .to_string(),
    );
    queries.insert(PredicateKey::Import, "(preproc_include) @match".to_string());

    // --- Comments / Strings ---
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(
        PredicateKey::Str,
        "[ (string_literal) @match (raw_string_literal) @match ]".to_string(),
    );

    LanguageProfile {
        name: "C++",
        extensions: vec!["cpp", "cc", "cxx", "hpp", "hh", "hxx"],
        language,
        queries,
    }
}
