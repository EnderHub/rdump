use anyhow::{anyhow, Result};
use serde::Serialize;
use std::path::Path;

use crate::evaluator::{Evaluator, FileContext, MatchResult};
use crate::predicates::code_aware::CodeAwareSettings;
use crate::predicates::code_aware::{
    detect_sql_dialect_for_debug, detect_sql_dialect_trace_for_debug,
};
use crate::predicates::{
    content_predicate_keys, create_metadata_predicate_registry,
    create_predicate_registry_with_settings, metadata_predicate_keys, react_predicate_keys,
    semantic_predicate_keys,
};
use crate::{parser, planner, QueryAction, SearchOptions};

#[derive(Serialize)]
struct FileExplainReport {
    path: String,
    effective_query: String,
    metadata_result: String,
    full_result: String,
    diagnostics: Vec<String>,
    semantic_skip_reasons: Vec<String>,
}

pub fn run_query(action: QueryAction) -> Result<()> {
    match action {
        QueryAction::Explain {
            query,
            preset,
            json,
        } => {
            let options = SearchOptions {
                presets: preset,
                ..Default::default()
            };
            let explanation = crate::explain_query(&query, &options)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&explanation)?);
                return Ok(());
            }
            println!("Original query: {}", explanation.original_query);
            println!("Effective query: {}", explanation.effective_query);
            println!("Normalized query: {}", explanation.normalized_query);
            println!("Simplified query: {}", explanation.simplified_query);
            println!("Estimated cost: {}", explanation.estimated_cost);
            if !explanation.metadata_predicates.is_empty() {
                println!(
                    "Metadata predicates: {}",
                    explanation.metadata_predicates.join(", ")
                );
            }
            if !explanation.content_predicates.is_empty() {
                println!(
                    "Content predicates: {}",
                    explanation.content_predicates.join(", ")
                );
            }
            if !explanation.semantic_predicates.is_empty() {
                println!(
                    "Semantic predicates: {}",
                    explanation.semantic_predicates.join(", ")
                );
            }
            if !explanation.react_predicates.is_empty() {
                println!(
                    "React predicates: {}",
                    explanation.react_predicates.join(", ")
                );
            }
            for stage in explanation.stages {
                println!("Stage {}: {}", stage.name, stage.description);
            }
            if !explanation.preset_contributions.is_empty() {
                println!("Preset provenance:");
                for contribution in explanation.preset_contributions {
                    println!("  - {} => {}", contribution.preset, contribution.clause);
                }
            }
            if !explanation.predicate_plans.is_empty() {
                println!("Predicate plan:");
                for plan in explanation.predicate_plans {
                    println!("  - {} [{}] {}", plan.key, plan.category, plan.raw_value);
                }
            }
            if !explanation.config_diagnostics.is_empty() {
                println!("Config warnings:");
                for diagnostic in explanation.config_diagnostics {
                    println!("  - {}: {}", diagnostic.code, diagnostic.message);
                }
            }
            if !explanation.preflight.warnings.is_empty() {
                println!("Repo preflight:");
                for warning in explanation.preflight.warnings {
                    println!("  - {warning}");
                }
            }
            for note in explanation.notes {
                println!("Note: {note}");
            }
        }
        QueryAction::Effective { query, preset } => {
            let options = SearchOptions {
                presets: preset,
                ..Default::default()
            };
            let query = query.unwrap_or_default();
            let explanation = crate::explain_query(&query, &options)?;
            println!("{}", explanation.effective_query);
        }
        QueryAction::Validate {
            query,
            preset,
            json,
        } => {
            let options = SearchOptions {
                presets: preset,
                ..Default::default()
            };
            let explanation = crate::explain_query(&query, &options)?;
            let lints = planner::lint_query(&query, &options)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "valid": true,
                        "effective_query": explanation.effective_query,
                        "normalized_query": explanation.normalized_query,
                        "simplified_query": explanation.simplified_query,
                        "warnings": lints,
                        "config_diagnostics": explanation.config_diagnostics,
                    }))?
                );
                return Ok(());
            }
            println!("Valid");
            println!("Effective query: {}", explanation.effective_query);
            println!("Normalized query: {}", explanation.normalized_query);
            println!("Simplified query: {}", explanation.simplified_query);
            for lint in lints {
                println!("Lint: {lint}");
            }
        }
        QueryAction::Normalize { query } => {
            println!("{}", planner::simplify_query(&query)?);
        }
        QueryAction::Ast { query } => {
            println!("{}", planner::serialize_query_ast(&query)?);
        }
        QueryAction::Reference { json } => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&crate::request::predicate_catalog())?
                );
            } else {
                let catalog = crate::request::predicate_catalog();
                println!(
                    "Metadata predicates: {}",
                    join_keys(metadata_predicate_keys())
                );
                println!(
                    "Content predicates: {}",
                    join_keys(content_predicate_keys())
                );
                println!(
                    "Semantic predicates: {}",
                    join_keys(semantic_predicate_keys())
                );
                println!("React predicates: {}", join_keys(react_predicate_keys()));
                println!("\nAliases:");
                for descriptor in catalog.predicates.iter().filter(|descriptor| {
                    !descriptor.aliases.is_empty() || !descriptor.deprecated_aliases.is_empty()
                }) {
                    if !descriptor.aliases.is_empty() {
                        println!("- {} => {}", descriptor.name, descriptor.aliases.join(", "));
                    }
                    if !descriptor.deprecated_aliases.is_empty() {
                        println!(
                            "- deprecated {} => {}",
                            descriptor.deprecated_aliases.join(", "),
                            descriptor.name
                        );
                    }
                }
            }
        }
        QueryAction::WhyNoResults {
            query,
            preset,
            root,
        } => {
            let options = SearchOptions {
                root,
                presets: preset,
                ..Default::default()
            };
            let explanation = match crate::explain_query(&query, &options) {
                Ok(explanation) => explanation,
                Err(err) => {
                    println!("Invalid query.");
                    println!("Error: {err}");
                    println!(
                        "Hint: use explicit '&' or '|' operators between predicates, or run `rdump query explain`."
                    );
                    return Ok(());
                }
            };
            let report = match crate::search_with_stats(&query, options) {
                Ok(report) => report,
                Err(err) => {
                    println!("Search failed before results could be analyzed.");
                    println!("Error: {err}");
                    return Ok(());
                }
            };
            if !report.results.is_empty() {
                println!(
                    "Query returned {} result(s); use search or query explain for more detail.",
                    report.results.len()
                );
            } else {
                println!("No results.");
                println!("Effective query: {}", explanation.effective_query);
                println!(
                    "Engine stats: candidates={} prefiltered={} evaluated={} matched_files={}",
                    report.stats.candidate_files,
                    report.stats.prefiltered_files,
                    report.stats.evaluated_files,
                    report.stats.matched_files
                );
                for note in explanation.notes {
                    println!("Note: {note}");
                }
                for diagnostic in &report.diagnostics {
                    println!("Diagnostic: {}", diagnostic.message.replace('\n', " "));
                }
                let unsupported_language_skips = unsupported_language_skip_count(&report);
                if report.stats.candidate_files == 0 {
                    println!("Hint: the root and ignore settings produced zero candidate files.");
                } else if report.stats.prefiltered_files == 0 {
                    println!(
                        "Hint: metadata predicates filtered everything before content or semantic evaluation."
                    );
                } else if unsupported_language_skips > 0 {
                    println!(
                        "Hint: semantic predicates targeted unsupported languages in {} candidate file(s).",
                        unsupported_language_skips
                    );
                } else if report.stats.semantic_parse_failures > 0 {
                    println!(
                        "Hint: semantic parsing failed for {} file(s).",
                        report.stats.semantic_parse_failures
                    );
                } else {
                    println!(
                        "Hint: surviving files did not produce true content or semantic matches."
                    );
                }
            }
        }
        QueryAction::WhyFile {
            query,
            path,
            preset,
            root,
        } => {
            let options = SearchOptions {
                root,
                presets: preset,
                ..Default::default()
            };
            let report = explain_file(&query, &path, &options)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        QueryAction::Dialect { path } => {
            let text = std::fs::read_to_string(&path)
                .map_err(|err| anyhow!("Failed to read {}: {err}", path.display()))?;
            let (_debug_detected, trace) = detect_sql_dialect_trace_for_debug(&text);
            let detected = detect_sql_dialect_for_debug(&text)
                .map(|dialect| dialect.key().to_string())
                .unwrap_or_else(|| "sql".to_string());
            println!("path={}", path.display());
            println!("detected_dialect={detected}");
            println!("reason={trace}");
        }
    }

    Ok(())
}

fn explain_file(query: &str, path: &Path, options: &SearchOptions) -> Result<FileExplainReport> {
    let effective_query = planner::resolve_effective_query(query, options)?;
    let ast = planner::optimize_ast(parser::parse_query(&effective_query)?);

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        options.root.join(path)
    };

    let metadata_registry = create_metadata_predicate_registry();
    let mut metadata_context = FileContext::new(absolute_path.clone(), options.root.clone());
    let metadata_result =
        Evaluator::new(ast.clone(), metadata_registry).evaluate(&mut metadata_context)?;
    let metadata_diagnostics = metadata_context.take_diagnostics();

    let mut code_settings = CodeAwareSettings::default();
    code_settings.sql_dialect = options.sql_dialect;
    code_settings.sql_strict = options.sql_strict;
    code_settings.language_override = options.language_override.clone();
    code_settings.semantic_budget_ms = options.semantic_budget_ms;
    code_settings.max_semantic_matches_per_file = options.max_semantic_matches_per_file;
    code_settings.semantic_match_mode = options.semantic_match_mode;
    code_settings.semantic_strict = options.semantic_strict;
    code_settings.language_debug = options.language_debug;
    code_settings.sql_trace = options.sql_trace;
    let mut full_context = FileContext::new(absolute_path.clone(), options.root.clone());
    let full_result = Evaluator::new(ast, create_predicate_registry_with_settings(code_settings))
        .evaluate(&mut full_context)?;
    let full_diagnostics = full_context.take_diagnostics();
    let semantic_skip_reasons = full_context
        .take_semantic_skip_reasons()
        .into_iter()
        .map(|reason| format!("{reason:?}").to_lowercase())
        .collect();

    Ok(FileExplainReport {
        path: absolute_path.display().to_string(),
        effective_query,
        metadata_result: match_result_label(&metadata_result),
        full_result: match_result_label(&full_result),
        diagnostics: metadata_diagnostics
            .into_iter()
            .chain(full_diagnostics)
            .map(|diagnostic| diagnostic.message)
            .collect(),
        semantic_skip_reasons,
    })
}

fn join_keys(keys: Vec<crate::parser::PredicateKey>) -> String {
    let mut names: Vec<_> = keys
        .into_iter()
        .map(|key| key.as_ref().to_string())
        .collect();
    names.sort();
    names.dedup();
    names.join(", ")
}

fn match_result_label(result: &MatchResult) -> String {
    match result {
        MatchResult::Boolean(value) => format!("boolean:{value}"),
        MatchResult::Hunks(hunks) => format!("hunks:{}", hunks.len()),
    }
}

fn unsupported_language_skip_count(report: &crate::SearchReport) -> usize {
    report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.kind == crate::content::DiagnosticKind::SemanticSkip
                && diagnostic
                    .message
                    .contains("no supported language profile matched")
        })
        .count()
}
