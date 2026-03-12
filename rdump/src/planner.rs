use anyhow::{anyhow, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::config::{self, ConfigDiagnostic, PresetContribution};
use crate::parser::{self, AstNode, PredicateKey};
use crate::predicates::helpers::{parse_modified_predicate, parse_size_predicate};
use crate::predicates::{
    content_predicate_keys, metadata_predicate_keys, react_predicate_keys, semantic_predicate_keys,
};
use crate::SearchOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStage {
    pub name: String,
    pub description: String,
    pub predicates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLanguageCount {
    pub extension: String,
    pub files: usize,
    pub semantic_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPreflight {
    pub root: String,
    pub semantic_predicates_present: bool,
    pub semantic_candidate_files: usize,
    pub dominant_extensions: Vec<RepoLanguageCount>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StableAstNode {
    Predicate { key: String, value: String },
    Not { child: Box<StableAstNode> },
    And { children: Vec<StableAstNode> },
    Or { children: Vec<StableAstNode> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PredicateValuePlan {
    Size {
        operator: String,
        numeric_value: f64,
        unit: String,
        target_size_bytes: u64,
    },
    Modified {
        operator: String,
        value: serde_json::Value,
    },
    Literal {
        normalized: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredicatePlan {
    pub key: String,
    pub category: String,
    pub raw_value: String,
    pub value_plan: PredicateValuePlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExplanation {
    pub original_query: String,
    pub effective_query: String,
    pub normalized_query: String,
    pub simplified_query: String,
    pub stable_ast: StableAstNode,
    pub stable_ast_json: String,
    pub metadata_predicates: Vec<String>,
    pub content_predicates: Vec<String>,
    pub semantic_predicates: Vec<String>,
    pub react_predicates: Vec<String>,
    pub predicate_plans: Vec<PredicatePlan>,
    pub stages: Vec<QueryStage>,
    pub estimated_cost: String,
    pub notes: Vec<String>,
    pub preset_contributions: Vec<PresetContribution>,
    pub config_diagnostics: Vec<ConfigDiagnostic>,
    pub preflight: QueryPreflight,
}

#[derive(Debug, Clone)]
pub struct EffectiveQuery {
    pub effective_query: String,
    pub preset_contributions: Vec<PresetContribution>,
    pub config_diagnostics: Vec<ConfigDiagnostic>,
}

static DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    "node_modules/**",
    "target/**",
    "dist/**",
    "build/**",
    ".git/**",
    ".svn/**",
    ".hg/**",
    "**/*.pyc",
    "**/__pycache__/**",
];

pub fn explain_query(query: &str, options: &SearchOptions) -> Result<QueryExplanation> {
    let effective = resolve_effective_query_details(query, options)?;
    let ast = parser::parse_query(&effective.effective_query)?;
    let normalized_query = ast.to_canonical_string();
    let simplified_ast = simplify_ast(ast);
    let simplified_query = simplified_ast.to_canonical_string();
    let optimized_ast = optimize_ast(simplified_ast.clone());
    let lint_warnings = lint_query_ast(&optimized_ast, query, &effective.effective_query);
    let predicate_plans = predicate_plans(&optimized_ast)?;

    let mut keys = Vec::new();
    collect_predicates(&optimized_ast, &mut keys);

    let metadata = classify_keys(&keys, &metadata_predicate_keys());
    let content = classify_keys(&keys, &content_predicate_keys());
    let semantic = classify_keys(&keys, &semantic_predicate_keys());
    let react = classify_keys(&keys, &react_predicate_keys());

    let estimated_cost = if !react.is_empty() || !semantic.is_empty() {
        "high"
    } else if !content.is_empty() {
        "medium"
    } else {
        "low"
    };

    let mut notes = Vec::new();
    if !metadata.is_empty() {
        notes.push(
            "Metadata predicates run during candidate prefiltering before content evaluation."
                .to_string(),
        );
    }
    if !content.is_empty() {
        notes.push("Content predicates require reading candidate file text.".to_string());
    }
    if !semantic.is_empty() || !react.is_empty() {
        notes.push(
            "Semantic predicates use tree-sitter and are limited to supported language profiles."
                .to_string(),
        );
        if !metadata.iter().any(|key| key == "ext") {
            notes.push(
                "Add ext: or presets to narrow semantic evaluation and reduce scan cost."
                    .to_string(),
            );
        }
    }
    if effective.effective_query != query {
        notes.push("Presets expanded into the effective query shown here.".to_string());
    }
    if !effective.config_diagnostics.is_empty() {
        notes.push(format!(
            "Config emitted {} migration or compatibility warning(s).",
            effective.config_diagnostics.len()
        ));
    }
    notes.extend(lint_warnings);
    notes.sort();
    notes.dedup();

    let preflight = repo_preflight(&optimized_ast, options);
    if !preflight.warnings.is_empty() {
        notes.extend(preflight.warnings.clone());
        notes.sort();
        notes.dedup();
    }

    let stages = vec![
        QueryStage {
            name: "discover".to_string(),
            description: "Walk candidate files under the root with ignore rules applied."
                .to_string(),
            predicates: vec![],
        },
        QueryStage {
            name: "prefilter".to_string(),
            description: "Evaluate metadata predicates to cut down the file set early.".to_string(),
            predicates: metadata.clone(),
        },
        QueryStage {
            name: "evaluate".to_string(),
            description: "Run content and semantic predicates on surviving files.".to_string(),
            predicates: [content.clone(), semantic.clone(), react.clone()].concat(),
        },
        QueryStage {
            name: "materialize".to_string(),
            description: "Convert matches into SearchResult or path-only output.".to_string(),
            predicates: vec![],
        },
    ];

    let stable_ast = stable_ast(&simplified_ast);
    let stable_ast_json =
        serde_json::to_string_pretty(&stable_ast).expect("stable query AST should serialize");

    Ok(QueryExplanation {
        original_query: query.to_string(),
        effective_query: effective.effective_query,
        normalized_query,
        simplified_query,
        stable_ast,
        stable_ast_json,
        metadata_predicates: metadata,
        content_predicates: content,
        semantic_predicates: semantic,
        react_predicates: react,
        predicate_plans,
        stages,
        estimated_cost: estimated_cost.to_string(),
        notes,
        preset_contributions: effective.preset_contributions,
        config_diagnostics: effective.config_diagnostics,
        preflight,
    })
}

pub fn lint_query(query: &str, options: &SearchOptions) -> Result<Vec<String>> {
    let effective = resolve_effective_query_details(query, options)?;
    let ast = parser::parse_query(&effective.effective_query)?;
    let optimized = optimize_ast(simplify_ast(ast));
    Ok(lint_query_ast(
        &optimized,
        query,
        &effective.effective_query,
    ))
}

pub fn simplify_query(query: &str) -> Result<String> {
    let ast = parser::parse_query(query)?;
    Ok(simplify_ast(ast).to_canonical_string())
}

pub fn serialize_query_ast(query: &str) -> Result<String> {
    let ast = parser::parse_query(query)?;
    let stable = stable_ast(&simplify_ast(ast));
    Ok(serde_json::to_string_pretty(&stable)?)
}

pub fn optimize_ast(ast: AstNode) -> AstNode {
    match ast {
        AstNode::LogicalOp(crate::parser::LogicalOperator::And, _, _) => {
            let mut nodes = Vec::new();
            collect_same_operator(&ast, crate::parser::LogicalOperator::And, &mut nodes);
            let mut nodes: Vec<AstNode> = nodes.into_iter().map(optimize_ast).collect();
            nodes.sort_by_key(predicate_cost);
            rebuild_chain(crate::parser::LogicalOperator::And, nodes)
        }
        AstNode::LogicalOp(op, left, right) => AstNode::LogicalOp(
            op,
            Box::new(optimize_ast(*left)),
            Box::new(optimize_ast(*right)),
        ),
        AstNode::Not(inner) => AstNode::Not(Box::new(optimize_ast(*inner))),
        predicate @ AstNode::Predicate(_, _) => predicate,
    }
}

pub fn simplify_ast(ast: AstNode) -> AstNode {
    match ast {
        AstNode::Predicate(_, _) => ast,
        AstNode::Not(inner) => match simplify_ast(*inner) {
            AstNode::Not(grandchild) => *grandchild,
            other => AstNode::Not(Box::new(other)),
        },
        AstNode::LogicalOp(op, left, right) => {
            let left = simplify_ast(*left);
            let right = simplify_ast(*right);
            let mut nodes = Vec::new();
            collect_same_operator(&left, op.clone(), &mut nodes);
            collect_same_operator(&right, op.clone(), &mut nodes);

            let mut deduped = Vec::new();
            let mut seen = BTreeSet::new();
            for node in nodes {
                let key = node.to_canonical_string();
                if seen.insert(key) {
                    deduped.push(node);
                }
            }

            rebuild_chain(op, deduped)
        }
    }
}

pub(crate) fn resolve_effective_query(query: &str, options: &SearchOptions) -> Result<String> {
    Ok(resolve_effective_query_details(query, options)?.effective_query)
}

pub fn resolve_effective_query_details(
    query: &str,
    options: &SearchOptions,
) -> Result<EffectiveQuery> {
    let report = config::load_config_report_for_dir(&options.root)?;
    let mut final_query = if query.trim().is_empty() {
        None
    } else {
        Some(query.to_string())
    };
    let mut preset_contributions = Vec::new();

    if !options.presets.is_empty() {
        let mut preset_queries = Vec::new();
        for preset_name in &options.presets {
            let resolved = report
                .resolved_presets
                .get(preset_name)
                .ok_or_else(|| anyhow!("Preset '{preset_name}' not found"))?;
            preset_queries.push(format!("({})", resolved.query));
            preset_contributions.extend(resolved.contributions.clone());
        }
        let all_presets = preset_queries.join(" & ");

        if let Some(query) = final_query {
            final_query = Some(format!("({all_presets}) & ({query})"));
        } else {
            final_query = Some(all_presets);
        }
    }

    let effective_query = final_query
        .ok_or_else(|| anyhow!("Empty query. Please provide a query or use a preset."))?;
    if effective_query.trim().is_empty() {
        return Err(anyhow!("Empty query."));
    }

    Ok(EffectiveQuery {
        effective_query,
        preset_contributions,
        config_diagnostics: report.diagnostics,
    })
}

pub fn repo_language_inventory(options: &SearchOptions) -> Vec<RepoLanguageCount> {
    let root = dunce::canonicalize(&options.root).unwrap_or_else(|_| options.root.clone());
    let default_ignore = default_ignore_set();
    let mut builder = WalkBuilder::new(&root);
    builder
        .hidden(!options.hidden)
        .follow_links(false)
        .max_depth(options.max_depth)
        .ignore(!options.no_ignore)
        .git_ignore(!options.no_ignore)
        .git_global(!options.no_ignore)
        .git_exclude(!options.no_ignore);
    if !options.no_ignore {
        builder.add_custom_ignore_filename(".rdumpignore");
    }

    let mut counts = BTreeMap::<String, usize>::new();
    for entry in builder.build().flatten() {
        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            continue;
        }
        let path = entry.path();
        let relative = path.strip_prefix(&root).unwrap_or(path);
        if !options.no_ignore && default_ignore.is_match(relative) {
            continue;
        }
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        *counts.entry(extension).or_default() += 1;
    }

    counts
        .into_iter()
        .map(|(extension, files)| {
            let semantic_profile =
                crate::predicates::code_aware::profiles::find_canonical_language_profile(
                    &extension,
                )
                .map(|profile| profile.id.to_string());
            RepoLanguageCount {
                extension,
                files,
                semantic_profile,
            }
        })
        .collect()
}

fn predicate_plans(ast: &AstNode) -> Result<Vec<PredicatePlan>> {
    let mut plans = Vec::new();
    collect_predicate_plans(ast, &mut plans)?;
    plans.sort_by(|left, right| {
        left.key
            .cmp(&right.key)
            .then(left.raw_value.cmp(&right.raw_value))
    });
    Ok(plans)
}

fn collect_predicate_plans(node: &AstNode, plans: &mut Vec<PredicatePlan>) -> Result<()> {
    match node {
        AstNode::Predicate(key, value) => {
            let category = if metadata_predicate_keys().contains(key) {
                "metadata"
            } else if content_predicate_keys().contains(key) {
                "content"
            } else if react_predicate_keys().contains(key) {
                "react"
            } else {
                "semantic"
            };
            let value_plan = match key {
                PredicateKey::Size => {
                    let parsed = parse_size_predicate(value)?;
                    PredicateValuePlan::Size {
                        operator: parsed.operator.as_str().to_string(),
                        numeric_value: parsed.numeric_value,
                        unit: parsed.unit,
                        target_size_bytes: parsed.target_size_bytes,
                    }
                }
                PredicateKey::Modified => {
                    let parsed = parse_modified_predicate(value)?;
                    PredicateValuePlan::Modified {
                        operator: parsed.operator.as_str().to_string(),
                        value: serde_json::to_value(parsed.value)
                            .expect("parsed modified predicate should serialize"),
                    }
                }
                _ => PredicateValuePlan::Literal {
                    normalized: value.to_string(),
                },
            };

            plans.push(PredicatePlan {
                key: key.as_ref().to_string(),
                category: category.to_string(),
                raw_value: value.clone(),
                value_plan,
            });
        }
        AstNode::LogicalOp(_, left, right) => {
            collect_predicate_plans(left, plans)?;
            collect_predicate_plans(right, plans)?;
        }
        AstNode::Not(inner) => collect_predicate_plans(inner, plans)?,
    }

    Ok(())
}

fn collect_predicates(node: &AstNode, keys: &mut Vec<PredicateKey>) {
    match node {
        AstNode::Predicate(key, _) => keys.push(key.clone()),
        AstNode::LogicalOp(_, left, right) => {
            collect_predicates(left, keys);
            collect_predicates(right, keys);
        }
        AstNode::Not(inner) => collect_predicates(inner, keys),
    }
}

fn classify_keys(keys: &[PredicateKey], members: &[PredicateKey]) -> Vec<String> {
    let mut names: Vec<String> = keys
        .iter()
        .filter(|key| members.contains(key))
        .map(|key| key.as_ref().to_string())
        .collect();
    names.sort();
    names.dedup();
    names
}

fn lint_query_ast(ast: &AstNode, original_query: &str, effective_query: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    if contains_deprecated_content_alias(original_query)
        || contains_deprecated_content_alias(effective_query)
    {
        warnings.push(
            "Deprecated predicate alias `content:` was used. Prefer `contains:`; the alias is accepted for compatibility but may be removed in a future major release."
                .to_string(),
        );
    }
    lint_and_groups(ast, &mut warnings);
    warnings.sort();
    warnings.dedup();
    warnings
}

fn lint_and_groups(node: &AstNode, warnings: &mut Vec<String>) {
    match node {
        AstNode::LogicalOp(crate::parser::LogicalOperator::And, _, _) => {
            let mut group = Vec::new();
            collect_and_predicates(node, &mut group);
            lint_conjunction(&group, warnings);
        }
        AstNode::LogicalOp(_, left, right) => {
            lint_and_groups(left, warnings);
            lint_and_groups(right, warnings);
        }
        AstNode::Not(inner) => lint_and_groups(inner, warnings),
        AstNode::Predicate(_, _) => {}
    }
}

fn collect_and_predicates<'a>(
    node: &'a AstNode,
    predicates: &mut Vec<(&'a PredicateKey, &'a str)>,
) {
    match node {
        AstNode::LogicalOp(crate::parser::LogicalOperator::And, left, right) => {
            collect_and_predicates(left, predicates);
            collect_and_predicates(right, predicates);
        }
        AstNode::Predicate(key, value) => predicates.push((key, value.as_str())),
        _ => {}
    }
}

fn lint_conjunction(predicates: &[(&PredicateKey, &str)], warnings: &mut Vec<String>) {
    let mut exts = BTreeSet::new();
    let mut exact_path_names = HashSet::new();
    for (key, value) in predicates {
        match key {
            PredicateKey::Ext => {
                exts.insert((*value).to_string());
            }
            PredicateKey::Name => {
                exact_path_names.insert((*value).to_string());
            }
            _ => {}
        }
    }

    if exts.len() > 1 {
        warnings.push(format!(
            "Conjunction contains multiple ext: predicates ({}) which likely yields zero results unless files have compound extensions.",
            exts.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    if exact_path_names.len() > 1 {
        warnings.push(
            "Conjunction contains multiple name: predicates. If they are exact names this is likely impossible."
                .to_string(),
        );
    }
    let has_semantic = predicates.iter().any(|(key, _)| {
        semantic_predicate_keys().contains(key) || react_predicate_keys().contains(key)
    });
    let has_narrowing_metadata = predicates.iter().any(|(key, _)| {
        matches!(
            key,
            PredicateKey::Ext
                | PredicateKey::Path
                | PredicateKey::PathExact
                | PredicateKey::In
                | PredicateKey::Name
        )
    });
    if has_semantic && !has_narrowing_metadata {
        warnings.push(
            "Semantic predicates are present without ext:/path:/in:/name: narrowing filters. Add a metadata predicate or preset to reduce fan-out."
                .to_string(),
        );
    }
}

fn contains_deprecated_content_alias(query: &str) -> bool {
    let lower = query.to_ascii_lowercase();
    lower.contains("content:")
}

fn collect_same_operator(
    node: &AstNode,
    operator: crate::parser::LogicalOperator,
    nodes: &mut Vec<AstNode>,
) {
    match node {
        AstNode::LogicalOp(op, left, right) if *op == operator => {
            collect_same_operator(left, operator.clone(), nodes);
            collect_same_operator(right, operator, nodes);
        }
        other => nodes.push(other.clone()),
    }
}

fn rebuild_chain(operator: crate::parser::LogicalOperator, mut nodes: Vec<AstNode>) -> AstNode {
    if nodes.is_empty() {
        return AstNode::Predicate(PredicateKey::Other("true".to_string()), "true".to_string());
    }
    if nodes.len() == 1 {
        return nodes.pop().unwrap();
    }

    let first = nodes.remove(0);
    nodes.into_iter().fold(first, |left, right| {
        AstNode::LogicalOp(operator.clone(), Box::new(left), Box::new(right))
    })
}

fn predicate_cost(node: &AstNode) -> usize {
    match node {
        AstNode::Predicate(key, _) if metadata_predicate_keys().contains(key) => 0,
        AstNode::Predicate(key, _) if content_predicate_keys().contains(key) => 10,
        AstNode::Predicate(key, _) if semantic_predicate_keys().contains(key) => 20,
        AstNode::Predicate(key, _) if react_predicate_keys().contains(key) => 25,
        AstNode::Predicate(_, _) => 30,
        AstNode::Not(inner) => predicate_cost(inner) + 5,
        AstNode::LogicalOp(crate::parser::LogicalOperator::Or, _, _) => 40,
        AstNode::LogicalOp(crate::parser::LogicalOperator::And, _, _) => 15,
    }
}

fn stable_ast(node: &AstNode) -> StableAstNode {
    match node {
        AstNode::Predicate(key, value) => StableAstNode::Predicate {
            key: key.as_ref().to_string(),
            value: value.clone(),
        },
        AstNode::Not(inner) => StableAstNode::Not {
            child: Box::new(stable_ast(inner)),
        },
        AstNode::LogicalOp(crate::parser::LogicalOperator::And, _, _) => {
            let mut children = Vec::new();
            collect_stable_children(node, crate::parser::LogicalOperator::And, &mut children);
            StableAstNode::And { children }
        }
        AstNode::LogicalOp(crate::parser::LogicalOperator::Or, _, _) => {
            let mut children = Vec::new();
            collect_stable_children(node, crate::parser::LogicalOperator::Or, &mut children);
            StableAstNode::Or { children }
        }
    }
}

fn collect_stable_children(
    node: &AstNode,
    operator: crate::parser::LogicalOperator,
    children: &mut Vec<StableAstNode>,
) {
    match node {
        AstNode::LogicalOp(op, left, right) if *op == operator => {
            collect_stable_children(left, operator.clone(), children);
            collect_stable_children(right, operator, children);
        }
        other => children.push(stable_ast(other)),
    }
}

fn repo_preflight(ast: &AstNode, options: &SearchOptions) -> QueryPreflight {
    let semantic_predicates_present = contains_semantic_predicates(ast);
    let inventory = repo_language_inventory(options);
    let semantic_candidate_files = inventory
        .iter()
        .filter(|entry| entry.semantic_profile.is_some())
        .map(|entry| entry.files)
        .sum();
    let dominant_extensions = inventory.into_iter().take(10).collect::<Vec<_>>();
    let mut warnings = Vec::new();

    if semantic_predicates_present && semantic_candidate_files == 0 {
        warnings.push(
            "Semantic predicates are present but the current root contains zero plausible semantic target files after ignore filtering."
                .to_string(),
        );
    }

    QueryPreflight {
        root: options.root.display().to_string(),
        semantic_predicates_present,
        semantic_candidate_files,
        dominant_extensions,
        warnings,
    }
}

fn contains_semantic_predicates(ast: &AstNode) -> bool {
    match ast {
        AstNode::Predicate(key, _) => {
            semantic_predicate_keys().contains(key) || react_predicate_keys().contains(key)
        }
        AstNode::LogicalOp(_, left, right) => {
            contains_semantic_predicates(left) || contains_semantic_predicates(right)
        }
        AstNode::Not(inner) => contains_semantic_predicates(inner),
    }
}

fn default_ignore_set() -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in DEFAULT_IGNORE_PATTERNS {
        builder.add(Glob::new(pattern).expect("default ignore glob should compile"));
    }
    builder.build().expect("default ignore set should compile")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explain_query_classifies_predicates() {
        let options = SearchOptions::default();
        let explanation = explain_query("ext:rs & contains:main & func:main", &options).unwrap();

        assert_eq!(explanation.metadata_predicates, vec!["ext"]);
        assert_eq!(explanation.content_predicates, vec!["contains"]);
        assert_eq!(explanation.semantic_predicates, vec!["func"]);
        assert_eq!(explanation.estimated_cost, "high");
        assert!(explanation.stable_ast_json.contains("\"kind\": \"and\""));
    }

    #[test]
    fn simplify_query_removes_duplicate_clauses() {
        let simplified = simplify_query("ext:rs & func:main & ext:rs").unwrap();
        assert_eq!(simplified, "ext:rs & func:main");
    }

    #[test]
    fn serialize_query_ast_is_machine_readable() {
        let serialized = serialize_query_ast("ext:rs | func:main").unwrap();
        assert!(serialized.contains("\"kind\": \"or\""));
        assert!(serialized.contains("\"key\": \"ext\""));
    }
}
