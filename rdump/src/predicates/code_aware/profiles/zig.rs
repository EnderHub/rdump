use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the Zig language.
pub(super) fn create_zig_profile() -> LanguageProfile {
    let language = tree_sitter_zig::LANGUAGE.into();
    let mut queries = HashMap::new();

    // Definitions
    let ident = "(identifier) @match";

    queries.insert(PredicateKey::Def, ident.to_string());
    queries.insert(PredicateKey::Func, ident.to_string());
    queries.insert(PredicateKey::Struct, ident.to_string());
    queries.insert(PredicateKey::Enum, ident.to_string());
    queries.insert(PredicateKey::Type, ident.to_string());

    // Imports (@import).
    queries.insert(PredicateKey::Import, "(builtin_identifier) @match".to_string());

    // Calls
    queries.insert(PredicateKey::Call, ident.to_string());

    // Comments / Strings
    queries.insert(PredicateKey::Comment, "(line_comment) @match".to_string());
    queries.insert(
        PredicateKey::Str,
        "[(string) @match (multiline_string) @match]".to_string(),
    );

    LanguageProfile {
        name: "Zig",
        extensions: vec!["zig"],
        language,
        queries,
    }
}
