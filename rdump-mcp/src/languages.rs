use crate::types::{LanguageInfo, LanguagePredicates};
use rdump::predicates::code_aware::profiles::{list_language_profiles, LanguageProfile};
use turbomcp::prelude::{McpError, McpResult};

pub fn list_languages() -> Vec<LanguageInfo> {
    list_language_profiles()
        .into_iter()
        .map(language_info_from_profile)
        .collect()
}

pub fn describe_language(language: &str) -> McpResult<LanguageInfo> {
    let lang_lower = language.to_lowercase();
    let profiles = list_language_profiles();
    let profile = profiles
        .into_iter()
        .find(|p| p.name.to_lowercase() == lang_lower || p.extensions.contains(&lang_lower.as_str()))
        .ok_or_else(|| McpError::Tool(format!("Language '{language}' not supported.")))?;

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
        lines.push(format!("- {} ({})", lang.name, ext_list));
    }

    if languages.len() > 20 {
        lines.push(format!("... and {} more", languages.len() - 20));
    }

    lines.join("\n")
}

pub fn format_language_text(info: &LanguageInfo) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Language: {}", info.name));
    if info.extensions.is_empty() {
        lines.push("Extensions: none".to_string());
    } else {
        lines.push(format!("Extensions: {}", info.extensions.join(", ")));
    }
    lines.push(format!(
        "Predicates: metadata {}, content {}, semantic {}",
        info.predicates.metadata.len(),
        info.predicates.content.len(),
        info.predicates.semantic.len()
    ));
    lines.join("\n")
}

fn language_info_from_profile(profile: &LanguageProfile) -> LanguageInfo {
    let metadata = vec!["ext", "name", "path", "in", "size", "modified"]
        .into_iter()
        .map(String::from)
        .collect();

    let content = vec!["contains", "matches"]
        .into_iter()
        .map(String::from)
        .collect();

    let mut semantic: Vec<String> = profile
        .queries
        .keys()
        .map(|k| k.as_ref().to_string())
        .collect();
    semantic.sort();

    LanguageInfo {
        name: profile.name.to_string(),
        extensions: profile.extensions.iter().map(|ext| ext.to_string()).collect(),
        predicates: LanguagePredicates {
            metadata,
            content,
            semantic,
        },
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
}
