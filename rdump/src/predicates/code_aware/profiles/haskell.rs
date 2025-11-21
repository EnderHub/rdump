use super::LanguageProfile;
use crate::parser::PredicateKey;
use std::collections::HashMap;

/// Creates the profile for the Haskell language.
pub(super) fn create_haskell_profile() -> LanguageProfile {
    let language = tree_sitter_haskell::LANGUAGE.into();
    let mut queries = HashMap::new();

    // Definitions: use variable nodes to stay compatible across grammar versions.
    let var_query = "(variable) @match";
    queries.insert(PredicateKey::Def, var_query.to_string());
    queries.insert(PredicateKey::Func, var_query.to_string());
    queries.insert(PredicateKey::Type, var_query.to_string());

    // Imports: match variable nodes inside import statements (approximate).
    queries.insert(PredicateKey::Import, var_query.to_string());

    // Calls: approximate with variables.
    queries.insert(PredicateKey::Call, var_query.to_string());

    // Comments / Strings.
    queries.insert(PredicateKey::Comment, "(comment) @match".to_string());
    queries.insert(PredicateKey::Str, "(string) @match".to_string());

    LanguageProfile {
        name: "Haskell",
        extensions: vec!["hs", "lhs"],
        language,
        queries,
    }
}
