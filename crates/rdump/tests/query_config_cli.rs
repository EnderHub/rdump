use predicates::prelude::*;
use serde_json::Value as JsonValue;
use std::fs;
use tempfile::tempdir;

#[test]
fn query_explain_outputs_effective_query() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.args(["query", "explain", "ext:rs & func:main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Effective query: ext:rs & func:main",
        ))
        .stdout(predicate::str::contains("Estimated cost:"));
    Ok(())
}

#[test]
fn config_path_and_show_work() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join(".rdump.toml"),
        "[presets]\nrust = \"ext:rs\"\n",
    )?;

    let mut path_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    path_cmd.current_dir(dir.path()).args(["config", "path"]);
    path_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(".rdump/config.toml"));

    let mut show_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    show_cmd.current_dir(dir.path()).args(["config", "show"]);
    show_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("[presets]"))
        .stdout(predicate::str::contains("rust = \"ext:rs\""));

    Ok(())
}

#[test]
fn query_reference_json_lists_aliases_and_deprecations() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    let output = cmd.args(["query", "reference", "--json"]).output()?;
    assert!(output.status.success());
    let json: JsonValue = serde_json::from_slice(&output.stdout)?;
    let predicates = json
        .get("predicates")
        .and_then(|value| value.as_array())
        .expect("predicate catalog should contain predicates");
    let contains = predicates
        .iter()
        .find(|entry| entry.get("name").and_then(|value| value.as_str()) == Some("contains"))
        .expect("contains predicate should be present");
    assert_eq!(
        contains
            .get("aliases")
            .and_then(|value| value.as_array())
            .map(|values| values
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>()),
        Some(vec!["c"])
    );
    assert_eq!(
        contains
            .get("deprecated_aliases")
            .and_then(|value| value.as_array())
            .map(|values| values
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>()),
        Some(vec!["content"])
    );
    Ok(())
}

#[test]
fn query_why_no_results_reports_engine_hints() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    fs::write(dir.path().join("main.rs"), "fn main() {}\n")?;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path())
        .args(["query", "why-no-results", "ext:py & func:main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No results."))
        .stdout(predicate::str::contains("Engine stats:"))
        .stdout(predicate::str::contains("Hint:"));
    Ok(())
}

#[test]
fn query_why_no_results_reports_invalid_query_instead_of_bailing(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.args(["query", "why-no-results", "ext:rs func:main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Invalid query."))
        .stdout(predicate::str::contains("explicit '&' or '|' operators"));
    Ok(())
}

#[test]
fn query_why_no_results_reports_unsupported_language_hint() -> Result<(), Box<dyn std::error::Error>>
{
    let dir = tempdir()?;
    fs::write(dir.path().join("hello.txt"), "hello world\n")?;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path())
        .args(["query", "why-no-results", "func:main"]);
    cmd.assert().success().stdout(predicate::str::contains(
        "unsupported languages in 1 candidate file(s)",
    ));
    Ok(())
}

#[test]
fn query_why_file_outputs_diagnostics_json() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    fs::create_dir_all(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), "fn main() {}\n")?;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    let output = cmd
        .current_dir(dir.path())
        .args(["query", "why-file", "ext:rs & func:main", "src/main.rs"])
        .output()?;
    assert!(output.status.success());
    let json: JsonValue = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        json.get("metadata_result").and_then(|value| value.as_str()),
        Some("boolean:true")
    );
    assert!(json
        .get("full_result")
        .and_then(|value| value.as_str())
        .is_some());
    Ok(())
}

#[test]
fn query_dialect_reports_detected_sql_profile() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let sql_path = dir.path().join("schema.sql");
    fs::write(
        &sql_path,
        "DELIMITER //\nCREATE PROCEDURE foo() BEGIN SELECT 1; END//\n",
    )?;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.args(["query", "dialect", sql_path.to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("detected_dialect=sqlmysql"));
    Ok(())
}

#[test]
fn config_doctor_reports_runtime_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path()).args(["config", "doctor"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("rdump doctor"))
        .stdout(predicate::str::contains("execution_policy="))
        .stdout(predicate::str::contains("default_limits="));
    Ok(())
}

#[test]
fn lang_matrix_json_reports_support_tiers() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    let output = cmd.args(["lang", "matrix", "--json"]).output()?;
    assert!(output.status.success());
    let json: JsonValue = serde_json::from_slice(&output.stdout)?;
    let languages = json
        .get("languages")
        .and_then(|value| value.as_array())
        .expect("language matrix should contain languages");
    assert!(languages.iter().any(|entry| {
        entry.get("id").and_then(|value| value.as_str()) == Some("rs")
            && entry.get("support_tier").and_then(|value| value.as_str()) == Some("stable")
    }));
    Ok(())
}
