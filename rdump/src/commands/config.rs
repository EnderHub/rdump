use anyhow::Result;
use serde::Serialize;

use crate::{config, ConfigAction};

#[derive(Serialize)]
struct ConfigValidationOutput {
    valid: bool,
    schema_version: u32,
    preset_count: usize,
    diagnostics: Vec<config::ConfigDiagnostic>,
}

#[derive(Serialize)]
struct ConfigDoctorOutput {
    config_path: Option<String>,
    cwd: Option<String>,
    temp_dir: String,
    schema_version: u32,
    preset_count: usize,
    diagnostics: Vec<config::ConfigDiagnostic>,
    execution_policy: DoctorExecutionPolicy,
    default_limits: rdump_contracts::Limits,
}

#[derive(Serialize)]
struct DoctorExecutionPolicy {
    max_concurrent_searches: usize,
    async_channel_capacity: usize,
    cancel_check_interval: usize,
}

pub fn run_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Path => {
            if let Some(path) = config::global_config_path() {
                println!("{}", path.display());
            }
        }
        ConfigAction::Show => {
            let config = config::load_config()?;
            println!("{}", toml::to_string_pretty(&config)?);
        }
        ConfigAction::Validate { json } => {
            let report = config::validate_config_report()?;
            let payload = ConfigValidationOutput {
                valid: true,
                schema_version: report.merged.schema_version,
                preset_count: report.merged.presets.len(),
                diagnostics: report.diagnostics,
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("Valid");
                println!("schema_version={}", payload.schema_version);
                println!("preset_count={}", payload.preset_count);
                for diagnostic in payload.diagnostics {
                    println!("warning:{} {}", diagnostic.code, diagnostic.message);
                }
            }
        }
        ConfigAction::Doctor { json } => {
            let report = config::load_config_report()?;
            let execution = crate::search_execution_policy();
            let payload = ConfigDoctorOutput {
                config_path: config::global_config_path().map(|path| path.display().to_string()),
                cwd: std::env::current_dir()
                    .ok()
                    .map(|path| path.display().to_string()),
                temp_dir: std::env::temp_dir().display().to_string(),
                schema_version: report.merged.schema_version,
                preset_count: report.merged.presets.len(),
                diagnostics: report.diagnostics,
                execution_policy: DoctorExecutionPolicy {
                    max_concurrent_searches: execution.max_concurrent_searches,
                    async_channel_capacity: execution.async_channel_capacity,
                    cancel_check_interval: execution.cancellation_check_interval,
                },
                default_limits: crate::request::default_limits(),
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("rdump doctor");
                println!(
                    "config_path={}",
                    payload.config_path.as_deref().unwrap_or("<unavailable>")
                );
                println!("schema_version={}", payload.schema_version);
                println!("preset_count={}", payload.preset_count);
                println!("cwd={}", payload.cwd.as_deref().unwrap_or("<unavailable>"));
                println!("temp_dir={}", payload.temp_dir);
                println!(
                    "execution_policy=max_concurrent_searches:{} async_channel_capacity:{} cancel_check_interval:{}",
                    payload.execution_policy.max_concurrent_searches,
                    payload.execution_policy.async_channel_capacity,
                    payload.execution_policy.cancel_check_interval
                );
                println!(
                    "default_limits={}",
                    serde_json::to_string(&payload.default_limits)?
                );
                for diagnostic in payload.diagnostics {
                    println!("warning:{} {}", diagnostic.code, diagnostic.message);
                }
            }
        }
    }

    Ok(())
}
