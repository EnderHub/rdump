use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn cli_help_mentions_new_query_and_config_surfaces() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("query"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("lang"))
        .stdout(predicate::str::contains("preset"));
}

#[test]
fn search_help_mentions_error_modes_and_budgets() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.args(["search", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--skip-errors"))
        .stdout(predicate::str::contains("--fail-fast"))
        .stdout(predicate::str::contains("--execution-budget-ms"))
        .stdout(predicate::str::contains("--semantic-budget-ms"))
        .stdout(predicate::str::contains("--language-override"))
        .stdout(predicate::str::contains("--semantic-match-mode"));
}

#[test]
fn readme_mentions_current_query_and_config_commands() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let readme = manifest_dir.parent().unwrap().join("README.md");
    let text = fs::read_to_string(readme).unwrap();

    assert!(text.contains("rdump query explain") || text.contains("query explain"));
    assert!(text.contains("rdump config show") || text.contains("config show"));
}
