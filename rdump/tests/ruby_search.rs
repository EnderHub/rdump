use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(name)
}

#[test]
fn test_def_and_func_predicates_ruby() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "def:Greeter"])
        .current_dir(fixture("ruby_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "func:greet"])
        .current_dir(fixture("ruby_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));
}

#[test]
fn test_import_call_and_str_predicates_ruby() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "import:require_relative"])
        .current_dir(fixture("ruby_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "call:greet"])
        .current_dir(fixture("ruby_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "str:\"Hello\""])
        .current_dir(fixture("ruby_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));
}
