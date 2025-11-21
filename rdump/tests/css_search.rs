use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(name)
}

#[test]
fn test_def_predicates_css() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "def:button"])
        .current_dir(fixture("css_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.css"));
}

#[test]
fn test_import_and_str_predicates_css() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "import:reset.css"])
        .current_dir(fixture("css_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.css"));

    // String predicate not available for CSS profile yet.
}
