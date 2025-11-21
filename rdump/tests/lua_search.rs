use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(name)
}

#[test]
fn test_def_and_func_predicates_lua() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "def:add"])
        .current_dir(fixture("lua_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "func:greet"])
        .current_dir(fixture("lua_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));
}

#[test]
fn test_import_call_and_str_predicates_lua() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "import:require"])
        .current_dir(fixture("lua_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "call:greet"])
        .current_dir(fixture("lua_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "str:\"Hello\""])
        .current_dir(fixture("lua_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));
}
