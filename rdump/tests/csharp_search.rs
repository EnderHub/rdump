use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(name)
}

#[test]
fn test_def_and_func_predicates_csharp() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "def:Greeter"])
        .current_dir(fixture("csharp_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "func:Main"])
        .current_dir(fixture("csharp_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));
}

#[test]
fn test_import_call_and_str_predicates_csharp() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "import:System"])
        .current_dir(fixture("csharp_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "call:Greet"])
        .current_dir(fixture("csharp_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--", "search", "str:\"Hello\""])
        .current_dir(fixture("csharp_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));
}
