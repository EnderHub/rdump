use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the OCaml language.
pub(super) fn create_ocaml_profile() -> LanguageProfile {
    let language = tree_sitter_ocaml::LANGUAGE_OCAML.into();
    let mut queries = HashMap::new();

    // Broad identifier capture to stay compatible across grammar versions.
    // Function/value definitions via let-binding pattern.
    let def_query = "(let_binding pattern: (_binding_pattern) @match)";

    queries.insert(PredicateKey::Def, def_query.to_string());
    queries.insert(PredicateKey::Func, def_query.to_string());
    queries.insert(PredicateKey::Type, "(identifier) @match".to_string());

    // `open Module` treated as import; match identifiers.
    queries.insert(PredicateKey::Import, "(identifier) @match".to_string());

    // Calls - approximate by identifiers.
    queries.insert(PredicateKey::Call, "(identifier) @match".to_string());

    // Module definitions in OCaml
    queries.insert(
        PredicateKey::Module,
        "(module_definition) @match".to_string(),
    );

    // Comments / Strings
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(PredicateKey::Str, "(string) @match".to_string());

    LanguageProfile {
        name: "OCaml",
        extensions: vec!["ml", "mli"],
        language,
        queries,
    }
}
