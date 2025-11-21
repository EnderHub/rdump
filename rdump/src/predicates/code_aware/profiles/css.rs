use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for CSS.
pub(super) fn create_css_profile() -> LanguageProfile {
    let language = tree_sitter_css::LANGUAGE.into();
    let mut queries = HashMap::new();

    // Selectors treated as defs.
    let selector_query = "(selectors (class_selector (class_name) @match))";
    let id_query = "(selectors (id_selector (id_name) @match))";

    queries.insert(
        PredicateKey::Def,
        [selector_query, id_query].join("\n"),
    );
    queries.insert(PredicateKey::Type, selector_query.to_string());

    // Imports (e.g., @import url(".."))
    queries.insert(PredicateKey::Import, "(import_statement) @match".to_string());

    // Calls not applicable for CSS; leave empty.
    queries.insert(PredicateKey::Call, "".to_string());

    // Comments / Strings
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(
        PredicateKey::Str,
        "[ (string_value) @match (plain_value) @match ]".to_string(),
    );

    LanguageProfile {
        name: "CSS",
        extensions: vec!["css"],
        language,
        queries,
    }
}
