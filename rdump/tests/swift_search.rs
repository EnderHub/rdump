use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(name)
}

#[test]
fn test_def_and_func_predicates_swift() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "def:ConsoleGreeter"])
        .current_dir(fixture("swift_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "func:add"])
        .current_dir(fixture("swift_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));
}

#[test]
fn test_import_call_and_str_predicates_swift() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "import:Foundation"])
        .current_dir(fixture("swift_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "call:greet"])
        .current_dir(fixture("swift_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "str:\"Hello\""])
        .current_dir(fixture("swift_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));
}
