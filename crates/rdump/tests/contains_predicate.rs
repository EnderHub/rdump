// In rdump/tests/contains_predicate.rs

use predicates::prelude::*;
mod common;
use common::setup_test_project;

#[test]
fn test_contains_simple() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("contains:\"main function\"")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("main function").from_utf8());
}

#[test]
fn test_contains_case_insensitivity() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("contains:\"MAIN FUNCTION\"")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("main function").from_utf8());
}

#[test]
fn test_contains_no_results() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("contains:\"this should not be found\"")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_contains_with_other_predicates() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("contains:\"main function\" and ext:rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("main.go").not());
}
