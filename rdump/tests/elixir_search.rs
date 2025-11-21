use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(name)
}

#[test]
fn test_def_and_func_predicates_elixir() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "def:Demo"])
        .current_dir(fixture("elixir_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("demo.ex"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "func:greet"])
        .current_dir(fixture("elixir_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("demo.ex"));
}

#[test]
fn test_call_and_str_predicates_elixir() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "call:greet"])
        .current_dir(fixture("elixir_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("demo.ex"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "str:\"Hello\""])
        .current_dir(fixture("elixir_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("demo.ex"));
}
