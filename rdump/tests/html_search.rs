use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(name)
}

#[test]
fn test_def_predicate_html() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "def:div"])
        .current_dir(fixture("html_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("index.html"));
}

#[test]
fn test_import_and_str_predicates_html() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "import:script"])
        .current_dir(fixture("html_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("index.html"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "str:\"Hello\""])
        .current_dir(fixture("html_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("index.html"));
}
