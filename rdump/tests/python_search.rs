use predicates::prelude::*;
mod common;
use common::setup_test_project;

#[test]
fn test_def_finds_python_class() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("def:Helper & ext:py");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("helper.py"))
        .stdout(predicate::str::contains("class Helper"))
        .stdout(predicate::str::contains("src/main.rs").not());
}

#[test]
fn test_func_finds_python_function() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("func:run_helper");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("helper.py"))
        .stdout(predicate::str::contains("def run_helper()"))
        .stdout(predicate::str::contains("src/main.rs").not());
}

#[test]
fn test_import_finds_python_import() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("import:os & ext:py");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("helper.py"))
        .stdout(predicate::str::contains("import os"))
        .stdout(predicate::str::contains("src/lib.rs").not());
}

#[test]
fn test_comment_and_class_predicates_python() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:FIXME & class:Helper")
        .assert()
        .success()
        .stdout(predicate::str::contains("helper.py"));
}

#[test]
fn test_str_predicate_python() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:/tmp/data")
        .assert()
        .success()
        .stdout(predicate::str::contains("helper.py"));
}

#[test]
fn test_call_predicate_python() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:run_helper | call:do_setup")
        .assert()
        .success()
        .stdout(predicate::str::contains("self.do_setup()"))
        .stdout(predicate::str::contains("run_helper()"));
}

#[test]
fn test_def_not_found_python() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:NonExistent & ext:py")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
