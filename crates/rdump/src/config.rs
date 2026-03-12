use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const CONFIG_SCHEMA_VERSION: u32 = 1;

fn default_config_schema_version() -> u32 {
    CONFIG_SCHEMA_VERSION
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigScope {
    Global,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigSource {
    pub scope: ConfigScope,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigDiagnostic {
    pub code: String,
    pub message: String,
    pub path: Option<PathBuf>,
}

impl ConfigDiagnostic {
    fn warning(code: impl Into<String>, message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetSpec {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default, alias = "extends", alias = "presets")]
    pub includes: Vec<String>,
}

impl PresetSpec {
    pub fn normalized(mut self) -> Self {
        self.tags.sort();
        self.tags.dedup();
        self.examples.sort();
        self.examples.dedup();
        self.includes.sort();
        self.includes.dedup();
        self
    }

    pub fn has_query_or_includes(&self) -> bool {
        self.query
            .as_deref()
            .is_some_and(|query| !query.trim().is_empty())
            || !self.includes.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PresetDefinition {
    Query(String),
    Detailed(PresetSpec),
}

impl Default for PresetDefinition {
    fn default() -> Self {
        Self::Detailed(PresetSpec::default())
    }
}

impl PresetDefinition {
    pub fn as_spec(&self) -> PresetSpec {
        match self {
            PresetDefinition::Query(query) => PresetSpec {
                query: Some(query.clone()),
                ..PresetSpec::default()
            },
            PresetDefinition::Detailed(spec) => spec.clone(),
        }
        .normalized()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_config_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub presets: BTreeMap<String, PresetDefinition>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: CONFIG_SCHEMA_VERSION,
            presets: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedPreset {
    pub name: String,
    pub query: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
    pub includes: Vec<String>,
    pub source: Option<ConfigSource>,
    pub contributions: Vec<PresetContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetContribution {
    pub preset: String,
    pub clause: String,
    pub source: Option<ConfigSource>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigReport {
    pub merged: Config,
    pub diagnostics: Vec<ConfigDiagnostic>,
    pub sources: Vec<ConfigSource>,
    pub resolved_presets: BTreeMap<String, ResolvedPreset>,
}

#[derive(Debug, Clone)]
struct LayeredPresetDefinition {
    definition: PresetDefinition,
    source: ConfigSource,
}

/// Returns the path to the configuration file.
/// Prefers a repo-local `.rdump/config.toml` to avoid touching host-global dirs.
/// Can be overridden by the RDUMP_TEST_CONFIG_DIR environment variable for testing.
pub fn global_config_path() -> Option<PathBuf> {
    if let Ok(path_str) = env::var("RDUMP_TEST_CONFIG_DIR") {
        return Some(PathBuf::from(path_str).join("rdump/config.toml"));
    }

    if let Ok(cwd) = env::current_dir() {
        return Some(cwd.join(".rdump/config.toml"));
    }

    dirs::config_dir().map(|p| p.join("rdump/config.toml"))
}

/// Searches for a local `.rdump.toml` in the given directory and its parents.
fn find_local_config(start_dir: &Path) -> Option<PathBuf> {
    for ancestor in start_dir.ancestors() {
        let config_path = ancestor.join(".rdump.toml");
        if config_path.exists() {
            return Some(config_path);
        }
    }
    None
}

pub fn load_config() -> Result<Config> {
    Ok(load_config_report()?.merged)
}

pub fn load_config_report() -> Result<ConfigReport> {
    let current_dir = env::current_dir()?;
    load_config_report_for_dir(&current_dir)
}

pub fn load_config_for_dir(start_dir: &Path) -> Result<Config> {
    Ok(load_config_report_for_dir(start_dir)?.merged)
}

pub fn load_config_report_for_dir(start_dir: &Path) -> Result<ConfigReport> {
    let layers = load_layers_for_dir(start_dir)?;
    let mut merged = Config::default();
    let mut diagnostics = Vec::new();
    let mut layered_presets = BTreeMap::new();
    let mut sources = Vec::new();

    for (config, source) in layers {
        sources.push(source.clone());
        if config.schema_version != CONFIG_SCHEMA_VERSION {
            diagnostics.push(ConfigDiagnostic::warning(
                "schema_version_mismatch",
                format!(
                    "Config schema version {} at {} will be migrated to {} semantics.",
                    config.schema_version,
                    source.path.display(),
                    CONFIG_SCHEMA_VERSION
                ),
                Some(source.path.clone()),
            ));
        }

        for (name, definition) in config.presets {
            layered_presets.insert(
                name,
                LayeredPresetDefinition {
                    definition,
                    source: source.clone(),
                },
            );
        }
    }

    merged.schema_version = CONFIG_SCHEMA_VERSION;
    merged.presets = layered_presets
        .iter()
        .map(|(name, layered)| (name.clone(), layered.definition.clone()))
        .collect();

    let resolved_presets = resolve_all_presets(&layered_presets)?;

    Ok(ConfigReport {
        merged,
        diagnostics,
        sources,
        resolved_presets,
    })
}

pub fn validate_config_report() -> Result<ConfigReport> {
    load_config_report()
}

pub fn validate_config_report_for_dir(start_dir: &Path) -> Result<ConfigReport> {
    load_config_report_for_dir(start_dir)
}

pub fn resolve_named_presets(names: &[String]) -> Result<(String, Vec<PresetContribution>)> {
    let current_dir = env::current_dir()?;
    resolve_named_presets_for_dir(names, &current_dir)
}

pub fn resolve_named_presets_for_dir(
    names: &[String],
    start_dir: &Path,
) -> Result<(String, Vec<PresetContribution>)> {
    let report = load_config_report_for_dir(start_dir)?;
    let mut clauses = Vec::new();
    let mut contributions = Vec::new();

    for name in names {
        let resolved = report
            .resolved_presets
            .get(name)
            .ok_or_else(|| anyhow!("Preset '{name}' not found"))?;
        clauses.push(format!("({})", resolved.query));
        contributions.extend(resolved.contributions.clone());
    }

    Ok((clauses.join(" & "), contributions))
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = global_config_path().ok_or_else(|| anyhow!("Could not determine config path"))?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory at {parent:?}"))?;
    }

    let mut config = config.clone();
    config.schema_version = CONFIG_SCHEMA_VERSION;

    let toml_string = toml::to_string_pretty(&config)?;
    fs::write(&path, toml_string)
        .with_context(|| format!("Failed to write global config to {path:?}"))?;

    println!("Successfully saved config to {path:?}");
    Ok(())
}

fn load_layers_for_dir(start_dir: &Path) -> Result<Vec<(Config, ConfigSource)>> {
    let mut layers = Vec::new();

    if let Some(path) = global_config_path().filter(|path| path.exists()) {
        layers.push((
            parse_config_file(&path)?,
            ConfigSource {
                scope: ConfigScope::Global,
                path,
            },
        ));
    }

    if let Some(path) = find_local_config(start_dir).filter(|path| path.exists()) {
        layers.push((
            parse_config_file(&path)?,
            ConfigSource {
                scope: ConfigScope::Local,
                path,
            },
        ));
    }

    Ok(layers)
}

fn parse_config_file(path: &Path) -> Result<Config> {
    let text =
        fs::read_to_string(path).with_context(|| format!("Failed to read config at {path:?}"))?;
    let config: Config =
        toml::from_str(&text).with_context(|| format!("Invalid TOML in {}", path.display()))?;
    Ok(config)
}

fn resolve_all_presets(
    presets: &BTreeMap<String, LayeredPresetDefinition>,
) -> Result<BTreeMap<String, ResolvedPreset>> {
    let mut resolved = BTreeMap::new();
    let mut visiting = BTreeSet::new();

    for name in presets.keys() {
        resolve_preset(name, presets, &mut visiting, &mut resolved)?;
    }

    Ok(resolved)
}

fn resolve_preset(
    name: &str,
    presets: &BTreeMap<String, LayeredPresetDefinition>,
    visiting: &mut BTreeSet<String>,
    resolved: &mut BTreeMap<String, ResolvedPreset>,
) -> Result<ResolvedPreset> {
    if let Some(cached) = resolved.get(name) {
        return Ok(cached.clone());
    }

    let Some(layered) = presets.get(name) else {
        return Err(anyhow!("Preset '{name}' not found"));
    };

    if !visiting.insert(name.to_string()) {
        let cycle = visiting.iter().cloned().collect::<Vec<_>>().join(" -> ");
        return Err(anyhow!(
            "Preset reference cycle detected: {cycle} -> {name}"
        ));
    }

    let spec = layered.definition.as_spec();
    if !spec.has_query_or_includes() {
        return Err(anyhow!(
            "Preset '{name}' must define a query or include at least one other preset"
        ));
    }

    let mut clauses = Vec::new();
    let mut contributions = Vec::new();

    for include in &spec.includes {
        let child = resolve_preset(include, presets, visiting, resolved).with_context(|| {
            format!("Preset '{name}' references unknown or invalid preset '{include}'")
        })?;
        clauses.push(format!("({})", child.query));
        contributions.extend(child.contributions.clone());
    }

    if let Some(query) = spec
        .query
        .as_deref()
        .filter(|query| !query.trim().is_empty())
    {
        crate::parser::parse_query(query)
            .with_context(|| format!("Preset '{name}' has an invalid query"))?;
        clauses.push(format!("({query})"));
        contributions.push(PresetContribution {
            preset: name.to_string(),
            clause: query.to_string(),
            source: Some(layered.source.clone()),
            description: spec.description.clone(),
            tags: spec.tags.clone(),
            examples: spec.examples.clone(),
        });
    }

    let resolved_preset = ResolvedPreset {
        name: name.to_string(),
        query: clauses.join(" & "),
        description: spec.description.clone(),
        tags: spec.tags.clone(),
        examples: spec.examples.clone(),
        includes: spec.includes.clone(),
        source: Some(layered.source.clone()),
        contributions,
    };

    visiting.remove(name);
    resolved.insert(name.to_string(), resolved_preset.clone());
    Ok(resolved_preset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn test_find_local_config_in_parent() {
        let _lock = ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let root = tempdir().unwrap();
        let sub = root.path().join("sub");
        fs::create_dir(&sub).unwrap();

        let config_path = root.path().join(".rdump.toml");
        fs::File::create(&config_path).unwrap();

        let found_path = find_local_config(&sub).unwrap();
        assert_eq!(found_path, config_path);
    }

    #[test]
    fn test_find_local_config_not_found() {
        let _lock = ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let root = tempdir().unwrap();
        assert!(find_local_config(root.path()).is_none());
    }

    #[test]
    fn test_load_config_merging_and_overriding() {
        let _lock = ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let test_dir = tempdir().unwrap();

        let fake_home_dir = test_dir.path().join("home");
        let global_config_dir = fake_home_dir.join("rdump");
        fs::create_dir_all(&global_config_dir).unwrap();
        let global_config_path = global_config_dir.join("config.toml");
        let mut global_file = fs::File::create(&global_config_path).unwrap();
        writeln!(
            global_file,
            r#"
            schema_version = 1
            [presets]
            rust = "ext:rs"
            docs = "ext:md"
        "#
        )
        .unwrap();

        let project_dir = test_dir.path().join("project");
        fs::create_dir(&project_dir).unwrap();
        let local_config_path = project_dir.join(".rdump.toml");
        let mut local_file = fs::File::create(&local_config_path).unwrap();
        writeln!(
            local_file,
            r#"
            schema_version = 1
            [presets]
            docs = "ext:md | ext:txt"

            [presets.scripts]
            query = "ext:sh"
            description = "Shell scripts"
            tags = ["shell", "ops"]
        "#
        )
        .unwrap();

        env::set_var("RDUMP_TEST_CONFIG_DIR", fake_home_dir.to_str().unwrap());
        let report = load_config_report_for_dir(&project_dir).unwrap();

        assert_eq!(report.merged.presets.len(), 3);
        assert_eq!(
            report.resolved_presets.get("rust").unwrap().query,
            "(ext:rs)"
        );
        assert_eq!(
            report
                .resolved_presets
                .get("scripts")
                .unwrap()
                .description
                .as_deref(),
            Some("Shell scripts")
        );
        assert!(report.diagnostics.is_empty());

        env::remove_var("RDUMP_TEST_CONFIG_DIR");
    }

    #[test]
    fn test_save_config_prefers_repo_local_dir() {
        let _lock = ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = tempdir().unwrap();
        env::set_var("RDUMP_TEST_CONFIG_DIR", temp.path().to_str().unwrap());

        let path = global_config_path().unwrap();
        assert!(path.ends_with("rdump/config.toml"));

        let mut cfg = Config::default();
        cfg.presets.insert(
            "local".to_string(),
            PresetDefinition::Query("ext:rs".to_string()),
        );
        save_config(&cfg).unwrap();

        assert!(path.exists());
        let saved = parse_config_file(&path).unwrap();
        assert_eq!(saved.schema_version, CONFIG_SCHEMA_VERSION);

        env::remove_var("RDUMP_TEST_CONFIG_DIR");
    }

    #[test]
    fn test_resolve_named_presets_reports_unknown_reference() {
        let _lock = ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = tempdir().unwrap();
        fs::write(
            temp.path().join(".rdump.toml"),
            r#"
schema_version = 1
[presets.backend]
includes = ["missing"]
query = "ext:rs"
"#,
        )
        .unwrap();

        let err = resolve_named_presets_for_dir(&["backend".to_string()], temp.path()).unwrap_err();

        assert!(err.to_string().contains("unknown or invalid preset"));
    }

    #[test]
    fn test_load_config_report_warns_on_schema_version_drift() {
        let _lock = ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = tempdir().unwrap();
        fs::write(
            temp.path().join(".rdump.toml"),
            r#"
schema_version = 0
[presets]
rust = "ext:rs"
"#,
        )
        .unwrap();

        let report = load_config_report_for_dir(temp.path()).unwrap();

        assert_eq!(report.merged.schema_version, CONFIG_SCHEMA_VERSION);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].code, "schema_version_mismatch");
    }
}
