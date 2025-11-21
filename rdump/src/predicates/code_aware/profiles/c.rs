use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the C language.
pub(super) fn create_c_profile() -> LanguageProfile {
    let language = tree_sitter_c::LANGUAGE.into();
    let mut queries = HashMap::new();

    let func_query = "
    [
        (function_definition
            declarator: (function_declarator
                declarator: (identifier) @match))
        (function_definition
            declarator: (pointer_declarator
                declarator: (function_declarator
                    declarator: (identifier) @match)))
    ]
    ";
    let struct_query = "(struct_specifier name: (type_identifier) @match)";
    let union_query = "(union_specifier name: (type_identifier) @match)";
    let enum_query = "(enum_specifier name: (type_identifier) @match)";
    let type_query = "
    [
        (type_definition declarator: (type_identifier) @match)
        (type_definition declarator: (identifier) @match)
        (type_definition declarator: (pointer_declarator declarator: (identifier) @match))
        (type_definition declarator: (pointer_declarator declarator: (function_declarator declarator: (identifier) @match)))
        (type_definition declarator: (array_declarator declarator: (identifier) @match))
        (type_definition declarator: (function_declarator declarator: (identifier) @match))
    ]
    ";
    let macro_query = "[(preproc_def name: (identifier) @match) (preproc_function_def name: (identifier) @match)]";

    queries.insert(
        PredicateKey::Def,
        [
            func_query,
            struct_query,
            union_query,
            enum_query,
            type_query,
        ]
        .join("\n"),
    );
    queries.insert(PredicateKey::Func, func_query.to_string());
    queries.insert(PredicateKey::Struct, [struct_query, union_query].join("\n"));
    queries.insert(PredicateKey::Enum, enum_query.to_string());
    queries.insert(PredicateKey::Type, type_query.to_string());
    queries.insert(PredicateKey::Macro, macro_query.to_string());

    queries.insert(PredicateKey::Import, "(preproc_include) @match".to_string());

    queries.insert(
        PredicateKey::Call,
        "(call_expression function: [ (identifier) @match (field_expression field: (field_identifier) @match) ])"
            .to_string(),
    );

    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());

    queries.insert(
        PredicateKey::Str,
        "[ (string_literal) @match (system_lib_string) @match ]".to_string(),
    );

    LanguageProfile {
        name: "C",
        extensions: vec!["c", "h"],
        language,
        queries,
    }
}
