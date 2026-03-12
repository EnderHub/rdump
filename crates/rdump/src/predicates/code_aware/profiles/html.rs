use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for HTML.
pub(super) fn create_html_profile() -> LanguageProfile {
    let language = tree_sitter_html::LANGUAGE.into();
    let mut queries = HashMap::new();

    // --- Elements as definitions ---
    let element_query = "(start_tag (tag_name) @match)";
    queries.insert(PredicateKey::Def, element_query.to_string());

    // Import-like tags (scripts/stylesheets)
    let import_query = "
    [
      (start_tag (tag_name) @match (#match? @match \"^(script|link)$\"))
    ]
    ";
    queries.insert(PredicateKey::Import, import_query.to_string());

    // Calls: not applicable; leave empty so predicate returns false
    queries.insert(PredicateKey::Call, "".to_string());

    // Comments / Strings
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(PredicateKey::Str, "(text) @match".to_string());

    LanguageProfile {
        name: "HTML",
        extensions: vec!["html", "htm"],
        language,
        queries,
    }
}
