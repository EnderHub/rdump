use super::{profiles, CodeAwareSettings, SqlDialect};
use crate::evaluator::{FileContext, MatchResult};
use crate::parser::PredicateKey;
use anyhow::Result;
use regex::Regex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tree_sitter::{Query, QueryCursor, StreamingIterator, Tree};

pub(super) struct ExecutionPlan {
    pub profile_key: String,
    pub profile: &'static profiles::LanguageProfile,
    pub tree: Tree,
}

pub(super) fn resolve_execution_plan(
    context: &mut FileContext,
    profile_key: String,
    profile: &'static profiles::LanguageProfile,
    settings: &CodeAwareSettings,
) -> Result<Option<ExecutionPlan>> {
    if let Some(telemetry) = &settings.telemetry {
        if context.has_tree_for(&profile_key) {
            telemetry.record_tree_cache_hit();
        } else {
            telemetry.record_tree_cache_miss();
        }
    }
    match context.get_tree(&profile_key, profile.language.clone()) {
        Ok(tree) => {
            if profile_key != SqlDialect::Generic.key()
                && profile_key.starts_with("sql")
                && tree.root_node().has_error()
            {
                return fallback_execution_plan(
                    context,
                    &profile_key,
                    anyhow::anyhow!(
                        "tree-sitter reported syntax errors for dialect-specific SQL parse"
                    ),
                    profile,
                    settings,
                );
            }
            Ok(Some(ExecutionPlan {
                profile_key,
                profile,
                tree: tree.clone(),
            }))
        }
        Err(err) => fallback_execution_plan(context, &profile_key, err, profile, settings),
    }
}

pub(super) fn execute_captures(
    tree: &Tree,
    content: &Arc<str>,
    query: &Query,
    key: &PredicateKey,
    value: &str,
    settings: &CodeAwareSettings,
) -> Result<MatchResult> {
    let mut cursor = QueryCursor::new();
    if let Some(limit) = settings.max_semantic_matches_per_file {
        let limit = limit.max(1).min(u32::MAX as usize) as u32;
        cursor.set_match_limit(limit);
    }
    let mut ranges = Vec::new();
    let source = content.as_bytes();
    let mut captures = cursor.matches(query, tree.root_node(), source);
    let started = Instant::now();
    let semantic_budget = settings.semantic_budget_ms.map(Duration::from_millis);

    while let Some(matched) = captures.next() {
        if semantic_budget.is_some_and(|budget| started.elapsed() > budget) {
            if let Some(telemetry) = &settings.telemetry {
                telemetry.record_budget_exhaustion();
            }
            break;
        }
        for capture in matched.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            if *capture_name != "match" {
                continue;
            }

            let captured_node = capture.node;
            let captured_text = captured_node.utf8_text(source)?;
            if is_capture_match(key, value, captured_text, settings) {
                ranges.push(captured_node.range());
            }
        }
    }

    Ok(MatchResult::Hunks(ranges))
}

fn fallback_execution_plan(
    context: &mut FileContext,
    profile_key: &str,
    err: anyhow::Error,
    profile: &'static profiles::LanguageProfile,
    settings: &CodeAwareSettings,
) -> Result<Option<ExecutionPlan>> {
    if profile_key != SqlDialect::Generic.key() && profile_key.starts_with("sql") {
        if settings.sql_strict {
            if let Some(telemetry) = &settings.telemetry {
                telemetry.record_parse_failure(profile_key);
            }
            context.push_semantic_skip(
                crate::SemanticSkipReason::ParseFailed,
                format!(
                    "Strict SQL mode rejected fallback for {} after {} failed to parse: {}",
                    context.path.display(),
                    profile_key,
                    err
                ),
            );
            return Err(anyhow::anyhow!(
                "Strict SQL mode: failed to parse {} with {}: {}",
                context.path.display(),
                profile_key,
                err
            ));
        }
        if let Some(generic_profile) = profiles::get_profile(SqlDialect::Generic.key()) {
            context.set_sql_profile_key(SqlDialect::Generic.key());
            if let Some(telemetry) = &settings.telemetry {
                if context.has_tree_for(SqlDialect::Generic.key()) {
                    telemetry.record_tree_cache_hit();
                } else {
                    telemetry.record_tree_cache_miss();
                }
            }
            match context.get_tree(SqlDialect::Generic.key(), generic_profile.language.clone()) {
                Ok(tree) => {
                    return Ok(Some(ExecutionPlan {
                        profile_key: SqlDialect::Generic.key().to_string(),
                        profile: generic_profile,
                        tree: tree.clone(),
                    }));
                }
                Err(fallback_err) => {
                    if let Some(telemetry) = &settings.telemetry {
                        telemetry.record_parse_failure(profile_key);
                    }
                    context.push_semantic_skip(
                        crate::SemanticSkipReason::ParseFailed,
                        format!(
                            "Failed to parse {} with {} and generic SQL fallback: {err}; {fallback_err}",
                            context.path.display(),
                            profile_key
                        ),
                    );
                    if settings.semantic_strict {
                        return Err(anyhow::anyhow!(
                            "Strict semantic mode: failed to parse {} with {} and generic SQL fallback: {err}; {fallback_err}",
                            context.path.display(),
                            profile_key
                        ));
                    }
                    return Ok(None);
                }
            }
        }

        if let Some(telemetry) = &settings.telemetry {
            telemetry.record_parse_failure(profile_key);
        }
        context.push_semantic_skip(
            crate::SemanticSkipReason::ParseFailed,
            format!(
                "Failed to parse {} with {} and no SQL fallback was available: {}",
                context.path.display(),
                profile_key,
                err
            ),
        );
        if settings.semantic_strict {
            return Err(anyhow::anyhow!(
                "Strict semantic mode: failed to parse {} with {} and no SQL fallback was available: {}",
                context.path.display(),
                profile_key,
                err
            ));
        }
        return Ok(None);
    }

    if let Some(telemetry) = &settings.telemetry {
        telemetry.record_parse_failure(profile_key);
    }
    context.push_semantic_skip(
        crate::SemanticSkipReason::ParseFailed,
        format!("Failed to parse {}: {}", context.path.display(), err),
    );
    if settings.semantic_strict {
        return Err(anyhow::anyhow!(
            "Strict semantic mode: failed to parse {}: {}",
            context.path.display(),
            err
        ));
    }
    let _ = profile;
    Ok(None)
}

fn is_capture_match(
    key: &PredicateKey,
    value: &str,
    captured_text: &str,
    settings: &CodeAwareSettings,
) -> bool {
    match key {
        PredicateKey::Import | PredicateKey::Comment | PredicateKey::Str => {
            match_text(captured_text, value, settings, true)
        }
        PredicateKey::Hook | PredicateKey::CustomHook => {
            value == "." || match_text(captured_text, value, settings, false)
        }
        PredicateKey::Call => match_text(captured_text, value, settings, true),
        _ => value == "." || match_text(captured_text, value, settings, false),
    }
}

fn match_text(
    captured_text: &str,
    value: &str,
    settings: &CodeAwareSettings,
    default_contains: bool,
) -> bool {
    match settings.semantic_match_mode {
        rdump_contracts::SemanticMatchMode::Exact => {
            if default_contains {
                captured_text.contains(value)
            } else {
                captured_text == value
            }
        }
        rdump_contracts::SemanticMatchMode::CaseInsensitive => {
            if default_contains {
                captured_text
                    .to_ascii_lowercase()
                    .contains(&value.to_ascii_lowercase())
            } else {
                captured_text.eq_ignore_ascii_case(value)
            }
        }
        rdump_contracts::SemanticMatchMode::Prefix => captured_text.starts_with(value),
        rdump_contracts::SemanticMatchMode::Regex => Regex::new(value)
            .map(|regex| regex.is_match(captured_text))
            .unwrap_or(false),
        rdump_contracts::SemanticMatchMode::Wildcard => {
            value == "." || wildcard_match(captured_text, value)
        }
    }
}

fn wildcard_match(captured_text: &str, value: &str) -> bool {
    if value == "*" {
        return true;
    }

    let pattern = regex::escape(value).replace("\\*", ".*");
    Regex::new(&format!("^{pattern}$"))
        .map(|regex| regex.is_match(captured_text))
        .unwrap_or(false)
}
