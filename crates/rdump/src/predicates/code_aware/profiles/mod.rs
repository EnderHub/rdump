use crate::parser::PredicateKey;
use once_cell::sync::Lazy;
use std::collections::{BTreeMap, HashMap};

use super::SqlDialect;

mod bash;
mod c;
mod cpp;
mod csharp;
mod css;
mod elixir;
mod go;
mod haskell;
mod html;
mod java;
mod javascript;
mod lua;
mod ocaml;
mod php;
mod python;
mod react; // Add react module
mod ruby;
mod rust;
mod scala;
mod sql;
mod swift;
mod typescript;
mod zig;

/// Defines the tree-sitter queries and metadata for a specific language.
#[derive(Debug)]
pub struct LanguageProfile {
    pub name: &'static str,
    pub extensions: Vec<&'static str>,
    pub(super) language: tree_sitter::Language,
    pub queries: HashMap<PredicateKey, String>,
}

#[derive(Debug, Clone)]
pub struct CanonicalLanguageProfile {
    pub id: &'static str,
    pub profile: &'static LanguageProfile,
    pub aliases: Vec<&'static str>,
}

pub fn support_tier_for_id(id: &str) -> rdump_contracts::LanguageSupportTier {
    match id {
        "html" | "css" | "sql" => rdump_contracts::LanguageSupportTier::Partial,
        "hs" | "ml" | "swift" | "scala" => rdump_contracts::LanguageSupportTier::Experimental,
        _ => rdump_contracts::LanguageSupportTier::Stable,
    }
}

pub(super) static LANGUAGE_PROFILES: Lazy<HashMap<&'static str, LanguageProfile>> =
    Lazy::new(|| {
        let mut m = HashMap::new();
        m.insert("c", c::create_c_profile());
        m.insert("cpp", cpp::create_cpp_profile());
        m.insert("cc", cpp::create_cpp_profile());
        m.insert("cxx", cpp::create_cpp_profile());
        m.insert("hpp", cpp::create_cpp_profile());
        m.insert("hh", cpp::create_cpp_profile());
        m.insert("hxx", cpp::create_cpp_profile());
        m.insert("cs", csharp::create_csharp_profile());
        m.insert("csx", csharp::create_csharp_profile());
        m.insert("php", php::create_php_profile());
        m.insert("rb", ruby::create_ruby_profile());
        m.insert("sh", bash::create_bash_profile());
        m.insert("bash", bash::create_bash_profile());
        m.insert("css", css::create_css_profile());
        m.insert("ex", elixir::create_elixir_profile());
        m.insert("exs", elixir::create_elixir_profile());
        m.insert("html", html::create_html_profile());
        m.insert("lua", lua::create_lua_profile());
        m.insert("ml", ocaml::create_ocaml_profile());
        m.insert("mli", ocaml::create_ocaml_profile());
        m.insert("zig", zig::create_zig_profile());
        m.insert("hs", haskell::create_haskell_profile());
        m.insert("lhs", haskell::create_haskell_profile());
        m.insert("scala", scala::create_scala_profile());
        m.insert("swift", swift::create_swift_profile());
        m.insert("rs", rust::create_rust_profile());
        m.insert("py", python::create_python_profile());
        m.insert("go", go::create_go_profile());
        m.insert("java", java::create_java_profile());
        m.insert("ts", typescript::create_typescript_profile());
        m.insert("js", javascript::create_javascript_profile());
        m.insert("jsx", react::create_react_profile());
        m.insert("tsx", react::create_react_profile());
        m.insert(SqlDialect::Generic.key(), sql::create_generic_profile());
        m.insert(SqlDialect::Postgres.key(), sql::create_postgres_profile());
        m.insert(SqlDialect::Mysql.key(), sql::create_mysql_profile());
        m.insert(SqlDialect::Sqlite.key(), sql::create_sqlite_profile());
        m
    });

/// Returns a list of all configured language profiles.
pub fn list_language_profiles() -> Vec<&'static LanguageProfile> {
    LANGUAGE_PROFILES.values().collect()
}

/// Returns a deduplicated, stably sorted language catalog with aliases.
pub fn list_canonical_language_profiles() -> Vec<CanonicalLanguageProfile> {
    let mut aliases_by_id: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();

    for (alias, profile) in LANGUAGE_PROFILES.iter() {
        let id = canonical_profile_id(alias, profile);
        aliases_by_id.entry(id).or_default().push(*alias);
    }

    let mut catalog: Vec<_> = aliases_by_id
        .into_iter()
        .filter_map(|(id, mut aliases)| {
            aliases.sort_unstable();
            aliases.dedup();
            let profile = LANGUAGE_PROFILES.get(id)?;
            Some(CanonicalLanguageProfile {
                id,
                profile,
                aliases,
            })
        })
        .collect();

    catalog.sort_by(|left, right| {
        left.profile
            .name
            .cmp(right.profile.name)
            .then(left.id.cmp(right.id))
    });
    catalog
}

pub fn find_canonical_language_profile(query: &str) -> Option<CanonicalLanguageProfile> {
    let query = query.to_lowercase();
    list_canonical_language_profiles()
        .into_iter()
        .find(|entry| {
            entry.id == query
                || entry.profile.name.to_lowercase() == query
                || entry.aliases.iter().any(|alias| *alias == query)
                || entry.profile.extensions.iter().any(|ext| *ext == query)
        })
}

pub(super) fn get_profile(key: &str) -> Option<&'static LanguageProfile> {
    LANGUAGE_PROFILES.get(key)
}

pub fn lint_language_profiles() -> Vec<String> {
    let mut issues = Vec::new();

    for entry in list_canonical_language_profiles() {
        for alias in &entry.aliases {
            let Some(alias_profile) = LANGUAGE_PROFILES.get(alias) else {
                issues.push(format!("alias `{alias}` is missing from LANGUAGE_PROFILES"));
                continue;
            };

            if alias_profile.name != entry.profile.name {
                issues.push(format!(
                    "alias `{alias}` drifted from canonical `{}` name: `{}` != `{}`",
                    entry.id, alias_profile.name, entry.profile.name
                ));
            }

            if alias_profile.extensions != entry.profile.extensions {
                issues.push(format!(
                    "alias `{alias}` drifted from canonical `{}` extensions",
                    entry.id
                ));
            }

            if alias_profile.queries.len() != entry.profile.queries.len() {
                issues.push(format!(
                    "alias `{alias}` drifted from canonical `{}` predicate coverage",
                    entry.id
                ));
            }
        }

        for (predicate, query) in &entry.profile.queries {
            if query.trim().is_empty() {
                continue;
            }
            if !query.contains("@match") {
                issues.push(format!(
                    "language `{}` predicate `{}` is missing the required @match capture",
                    entry.id,
                    predicate.as_ref()
                ));
            }

            let captures = query_capture_names(query);
            let mut seen = BTreeMap::<String, usize>::new();
            for capture in captures {
                *seen.entry(capture).or_default() += 1;
            }
            for (capture, count) in seen {
                if count > 1 && capture != "match" && !capture.starts_with('_') {
                    issues.push(format!(
                        "language `{}` predicate `{}` reuses capture role `@{capture}` {} times",
                        entry.id,
                        predicate.as_ref(),
                        count
                    ));
                }
                if !capture
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
                {
                    issues.push(format!(
                        "language `{}` predicate `{}` uses non-canonical capture role `@{capture}`",
                        entry.id,
                        predicate.as_ref()
                    ));
                }
            }
        }
    }

    issues.sort();
    issues.dedup();
    issues
}

pub fn render_language_profile_reference() -> String {
    let mut out = String::from(
        "# Language Semantic Profile Reference\n\nGenerated from live tree-sitter profiles. Capture convention: `@match`.\n\n",
    );

    for entry in list_canonical_language_profiles() {
        let mut predicates: Vec<_> = entry
            .profile
            .queries
            .keys()
            .map(|key| key.as_ref())
            .collect();
        predicates.sort_unstable();
        predicates.dedup();

        out.push_str(&format!(
            "## {} ({})\n\n- Support tier: `{}`\n- Aliases: `{}`\n- Extensions: `{}`\n- Semantic predicates: `{}`\n",
            entry.profile.name,
            entry.id,
            format!("{:?}", support_tier_for_id(entry.id)).to_lowercase(),
            entry.aliases.join(", "),
            entry.profile.extensions.join(", "),
            predicates.join(", "),
        ));

        let caveats = semantic_caveats_for_id(entry.id);
        if caveats.is_empty() {
            out.push_str("- Caveats: none recorded\n");
        } else {
            out.push_str("- Caveats:\n");
            for caveat in caveats {
                out.push_str(&format!("  - {caveat}\n"));
            }
        }

        out.push_str("\n### Matching Rules\n\n");
        let mut query_entries: Vec<_> = entry.profile.queries.iter().collect();
        query_entries.sort_by(|(left, _), (right, _)| left.as_ref().cmp(right.as_ref()));
        for (predicate, _) in query_entries {
            out.push_str(&format!(
                "- `{}`: {}\n",
                predicate.as_ref(),
                predicate_matching_rule(predicate)
            ));
        }
        out.push('\n');
    }

    out
}

pub fn predicate_matching_rule(predicate: &PredicateKey) -> &'static str {
    match predicate {
        PredicateKey::Import | PredicateKey::Comment | PredicateKey::Str | PredicateKey::Call => {
            "Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics."
        }
        PredicateKey::Hook | PredicateKey::CustomHook => {
            "Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode=wildcard` enables shell-style `*` matching."
        }
        _ => {
            "Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior."
        }
    }
}

pub fn semantic_caveats_for_id(id: &str) -> Vec<&'static str> {
    let mut caveats = Vec::new();
    match id {
        "html" => caveats.push("HTML semantic coverage is partial and focuses on structural nodes rather than browser/runtime behavior."),
        "css" => caveats.push("CSS semantic coverage is partial and focuses on selectors and declarations, not cascade resolution."),
        "sql" | "sqlgeneric" | "sqlpostgres" | "sqlmysql" | "sqlsqlite" => caveats.push("SQL dialect selection is heuristic unless overridden; enable strict mode to fail instead of falling back."),
        "jsx" | "tsx" => caveats.push("React-specific predicates are only available on JSX/TSX profiles and remain more permissive than language-core predicates."),
        "hs" | "ml" | "swift" | "scala" => caveats.push("This profile is experimental; expect narrower predicate coverage and fewer regression fixtures."),
        _ => {}
    }
    if matches!(
        support_tier_for_id(id),
        rdump_contracts::LanguageSupportTier::Partial
    ) {
        caveats.push("Support tier is partial; some language constructs may not produce semantic captures yet.");
    }
    caveats
}

fn query_capture_names(query: &str) -> Vec<String> {
    let mut names = Vec::new();
    let bytes = query.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'@' {
            let start = index + 1;
            let mut end = start;
            while end < bytes.len() {
                let ch = bytes[end] as char;
                if ch.is_ascii_lowercase()
                    || ch.is_ascii_uppercase()
                    || ch.is_ascii_digit()
                    || ch == '_'
                {
                    end += 1;
                } else {
                    break;
                }
            }
            if end > start {
                names.push(query[start..end].to_string());
            }
            index = end;
            continue;
        }
        index += 1;
    }
    names
}

fn canonical_profile_id(alias: &'static str, profile: &LanguageProfile) -> &'static str {
    if alias.starts_with("sql") {
        alias
    } else {
        profile.extensions.first().copied().unwrap_or(alias)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_language_profiles_pass_lint() {
        let issues = lint_language_profiles();
        assert!(
            issues.is_empty(),
            "language profile lint found issues:\n{}",
            issues.join("\n")
        );
    }

    #[test]
    fn rendered_language_profile_reference_mentions_capture_convention() {
        let rendered = render_language_profile_reference();
        assert!(rendered.contains("Capture convention: `@match`"));
        assert!(rendered.contains("Rust (rs)"));
    }
}
