use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(name)
}

#[test]
fn test_def_and_func_predicates_scala() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "def:Greeter"])
        .current_dir(fixture("scala_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "func:add"])
        .current_dir(fixture("scala_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));
}

#[test]
fn test_import_call_and_str_predicates_scala() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "import:Helper"])
        .current_dir(fixture("scala_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "call:greet"])
        .current_dir(fixture("scala_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));
}
