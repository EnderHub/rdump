use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(name)
}

#[test]
fn test_def_and_func_predicates_ocaml() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "def:greet"])
        .current_dir(fixture("ocaml_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.ml"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "func:add"])
        .current_dir(fixture("ocaml_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.ml"));
}

#[test]
fn test_import_call_and_str_predicates_ocaml() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "str:\"Hello\""])
        .current_dir(fixture("ocaml_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.ml"));
}
