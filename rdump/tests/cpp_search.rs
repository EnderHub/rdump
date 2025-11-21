use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(name)
}

#[test]
fn test_def_and_func_predicates_cpp() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "def:Point"])
        .current_dir(fixture("cpp_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "func:add"])
        .current_dir(fixture("cpp_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}

#[test]
fn test_import_call_and_comment_predicates_cpp() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "import:util.hpp"])
        .current_dir(fixture("cpp_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "call:greet"])
        .current_dir(fixture("cpp_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "str:\"Hello\""])
        .current_dir(fixture("cpp_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}
