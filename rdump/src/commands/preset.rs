use crate::config::{self, Config, PresetDefinition};
use crate::PresetAction;
use anyhow::{anyhow, Result};
use std::fs;

pub fn run_preset(action: PresetAction) -> Result<()> {
    match action {
        PresetAction::List => {
            let report = config::load_config_report()?;
            if report.resolved_presets.is_empty() {
                println!("No presets found.");
            } else {
                println!("Available presets:");
                let max_len = report
                    .resolved_presets
                    .keys()
                    .map(|key| key.len())
                    .max()
                    .unwrap_or(0);
                for (name, preset) in report.resolved_presets {
                    println!("  {name:<max_len$} : {}", preset.query);
                    if let Some(description) = preset.description {
                        println!("    description: {description}");
                    }
                    if !preset.tags.is_empty() {
                        println!("    tags: {}", preset.tags.join(", "));
                    }
                    if !preset.examples.is_empty() {
                        println!("    examples: {}", preset.examples.join(" | "));
                    }
                    if !preset.includes.is_empty() {
                        println!("    includes: {}", preset.includes.join(", "));
                    }
                }
            }
        }
        PresetAction::Add { name, query } => {
            let path = config::global_config_path()
                .ok_or_else(|| anyhow!("Could not determine global config path"))?;

            let mut config = if path.exists() {
                let config_str = fs::read_to_string(&path)?;
                toml::from_str(&config_str)?
            } else {
                Config::default()
            };

            println!("Adding/updating preset '{name}'...");
            config.presets.insert(name, PresetDefinition::Query(query));
            config::save_config(&config)?;
        }
        PresetAction::Remove { name } => {
            let path = config::global_config_path()
                .ok_or_else(|| anyhow!("Could not determine global config path"))?;

            if !path.exists() {
                return Err(anyhow!(
                    "Global config file does not exist. No presets to remove."
                ));
            }

            let mut config: Config = toml::from_str(&fs::read_to_string(&path)?)?;

            if config.presets.remove(&name).is_some() {
                println!("Removing preset '{name}'...");
                config::save_config(&config)?;
            } else {
                return Err(anyhow!("Preset '{name}' not found in global config."));
            }
        }
    }
    Ok(())
}
