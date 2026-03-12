use crate::types::{LanguageInfo, LanguagePredicates};
use rdump::predicates::code_aware::profiles::{
    find_canonical_language_profile, list_canonical_language_profiles, CanonicalLanguageProfile,
};
use turbomcp::prelude::{McpError, McpResult};

pub fn list_languages() -> Vec<LanguageInfo> {
    list_canonical_language_profiles()
        .into_iter()
        .map(language_info_from_profile)
        .collect()
}

pub fn describe_language(language: &str) -> McpResult<LanguageInfo> {
    let profile = find_canonical_language_profile(language)
        .ok_or_else(|| McpError::invalid_params(format!("Language '{language}' not supported.")))?;

    Ok(language_info_from_profile(profile))
}

pub fn format_language_list_text(languages: &[LanguageInfo]) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Supported languages: {}", languages.len()));

    for lang in languages.iter().take(20) {
        let ext_list = if lang.extensions.is_empty() {
            "no extensions".to_string()
        } else {
            lang.extensions.join(", ")
        };
        let caveat_suffix = if lang.semantic_caveats.is_empty() {
            String::new()
        } else {
            format!(" caveats={}", lang.semantic_caveats.len())
        };
        lines.push(format!("- {} ({}){}", lang.name, ext_list, caveat_suffix));
    }

    if languages.len() > 20 {
        lines.push(format!("... and {} more", languages.len() - 20));
    }

    lines.join("\n")
}

pub fn format_language_text(info: &LanguageInfo) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Id: {}", info.id));
    lines.push(format!("Language: {}", info.name));
    if info.extensions.is_empty() {
        lines.push("Extensions: none".to_string());
    } else {
        lines.push(format!("Extensions: {}", info.extensions.join(", ")));
    }
    if !info.aliases.is_empty() {
        lines.push(format!("Aliases: {}", info.aliases.join(", ")));
    }
    lines.push(format!(
        "Predicates: metadata {}, content {}, semantic {}",
        info.predicates.metadata.len(),
        info.predicates.content.len(),
        info.predicates.semantic.len()
    ));
    if !info.semantic_caveats.is_empty() {
        lines.push("Caveats:".to_string());
        for caveat in &info.semantic_caveats {
            lines.push(format!("- {caveat}"));
        }
    }
    lines.join("\n")
}

fn language_info_from_profile(profile: CanonicalLanguageProfile) -> LanguageInfo {
    let metadata = vec!["ext", "name", "path", "in", "size", "modified"]
        .into_iter()
        .map(String::from)
        .collect();

    let content = vec!["contains", "matches"]
        .into_iter()
        .map(String::from)
        .collect();

    let mut semantic: Vec<String> = profile
        .profile
        .queries
        .keys()
        .map(|k| k.as_ref().to_string())
        .collect();
    semantic.sort();

    LanguageInfo {
        id: profile.id.to_string(),
        name: profile.profile.name.to_string(),
        extensions: profile
            .profile
            .extensions
            .iter()
            .map(|ext| ext.to_string())
            .collect(),
        aliases: profile
            .aliases
            .iter()
            .map(|alias| alias.to_string())
            .collect(),
        support_tier: rdump::predicates::code_aware::profiles::support_tier_for_id(profile.id),
        predicates: LanguagePredicates {
            metadata,
            content,
            semantic,
        },
        semantic_caveats: rdump::predicates::code_aware::profiles::semantic_caveats_for_id(
            profile.id,
        )
        .into_iter()
        .map(str::to_string)
        .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_languages_returns_entries() {
        let languages = list_languages();
        assert!(!languages.is_empty());
        let first = &languages[0];
        assert!(!first.name.is_empty());
    }

    #[test]
    fn describe_language_unknown_returns_error() {
        let result = describe_language("definitely-not-a-language");
        assert!(result.is_err());
    }

    #[test]
    fn describe_language_by_extension() {
        let result = describe_language("rs");
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(!info.name.is_empty());
    }

    #[test]
    fn format_language_list_text_contains_name() {
        let languages = list_languages();
        let text = format_language_list_text(&languages);
        assert!(!text.is_empty());
        assert!(text.contains(&languages[0].name));
    }

    #[test]
    fn format_language_text_includes_extensions() {
        let info = describe_language("rs").unwrap();
        let text = format_language_text(&info);
        assert!(text.contains("Extensions:"));
    }

    #[test]
    fn list_languages_is_deduplicated_and_sorted() {
        let languages = list_languages();
        let mut ids: Vec<_> = languages
            .iter()
            .map(|language| language.id.clone())
            .collect();
        let mut sorted = ids.clone();
        sorted.sort_by(|left, right| {
            let left_name = languages
                .iter()
                .find(|language| language.id == *left)
                .map(|language| language.name.clone())
                .unwrap();
            let right_name = languages
                .iter()
                .find(|language| language.id == *right)
                .map(|language| language.name.clone())
                .unwrap();
            left_name.cmp(&right_name).then(left.cmp(right))
        });
        ids.sort();
        ids.dedup();

        assert_eq!(ids.len(), languages.len());
        assert_eq!(
            sorted,
            languages
                .iter()
                .map(|language| language.id.clone())
                .collect::<Vec<_>>()
        );
    }
}
