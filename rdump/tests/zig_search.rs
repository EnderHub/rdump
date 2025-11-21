use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(name)
}

#[test]
fn test_def_and_func_predicates_zig() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "def:greet"])
        .current_dir(fixture("zig_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.zig"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "func:add"])
        .current_dir(fixture("zig_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.zig"));
}

#[test]
fn test_import_call_and_str_predicates_zig() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "import:@import"])
        .current_dir(fixture("zig_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.zig"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "call:greet"])
        .current_dir(fixture("zig_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.zig"));

    // String predicate not enabled for Zig yet due to grammar differences.
}
