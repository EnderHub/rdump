use crate::planner::repo_language_inventory;
use crate::predicates::code_aware::profiles::{
    find_canonical_language_profile, list_canonical_language_profiles, support_tier_for_id,
};
use crate::request::language_capability_matrix;
use crate::LangAction;
use crate::SearchOptions;
use anyhow::{anyhow, Result};

pub fn run_lang(action: LangAction) -> Result<()> {
    match action {
        LangAction::List => {
            let profiles = list_canonical_language_profiles();
            println!("{:<14} {:<12} {:<14} EXTENSIONS", "ID", "NAME", "TIER");
            println!("────────────────────────────────────────────────────────────");
            for profile in profiles {
                println!(
                    "{:<14} {:<12} {:<14} {}",
                    profile.id,
                    profile.profile.name,
                    format!("{:?}", support_tier_for_id(profile.id)).to_lowercase(),
                    profile.profile.extensions.join(", ")
                );
            }
        }
        LangAction::Describe { language } => {
            let profile = find_canonical_language_profile(&language).ok_or_else(|| {
                anyhow!(
                    "Language '{language}' not supported. Run `rdump lang list` to see available languages."
                )
            })?;

            println!(
                "Predicates for {} ({})",
                profile.profile.name,
                profile.profile.extensions.join(", ")
            );
            println!("Id: {}", profile.id);
            println!(
                "Support tier: {}",
                format!("{:?}", support_tier_for_id(profile.id)).to_lowercase()
            );

            let metadata_preds = ["ext", "name", "path", "in", "size", "modified"];
            let content_preds = ["contains", "matches"];

            println!("\nMETADATA");
            println!("  {}", metadata_preds.join(", "));

            println!("\nCONTENT");
            println!("  {}", content_preds.join(", "));

            let mut semantic_preds: Vec<&str> =
                profile.profile.queries.keys().map(|k| k.as_ref()).collect();
            semantic_preds.sort_unstable();
            if !semantic_preds.is_empty() {
                println!("\nSEMANTIC");
                println!("  {}", semantic_preds.join(", "));
            }

            println!("\nALIASES");
            println!("  {}", profile.aliases.join(", "));
        }
        LangAction::Inventory { root, json } => {
            let options = SearchOptions {
                root,
                ..Default::default()
            };
            let inventory = repo_language_inventory(&options);
            if json {
                println!("{}", serde_json::to_string_pretty(&inventory)?);
            } else {
                println!("{:<12} {:<8} PROFILE", "EXT", "FILES");
                println!("────────────────────────────────────────────");
                for entry in inventory {
                    println!(
                        "{:<12} {:<8} {}",
                        if entry.extension.is_empty() {
                            "<none>"
                        } else {
                            &entry.extension
                        },
                        entry.files,
                        entry.semantic_profile.as_deref().unwrap_or("-")
                    );
                }
            }
        }
        LangAction::Matrix { json } => {
            let matrix = language_capability_matrix();
            if json {
                println!("{}", serde_json::to_string_pretty(&matrix)?);
            } else {
                println!("Capture convention: {}", matrix.capture_convention);
                for language in matrix.languages {
                    println!(
                        "- {} [{}] metadata={} content={} semantic={}",
                        language.name,
                        format!("{:?}", language.support_tier).to_lowercase(),
                        language.predicates.metadata.len(),
                        language.predicates.content.len(),
                        language.predicates.semantic.len(),
                    );
                }
            }
        }
    }
    Ok(())
}
