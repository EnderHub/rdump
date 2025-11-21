use predicates::prelude::*;
mod common;
use common::setup_fixture;

#[test]
fn test_struct_enum_and_type_predicates_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Server & enum:Status & type:user_id")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.c"))
        .stdout(predicate::str::contains("typedef struct Server"))
        .stdout(predicate::str::contains("typedef enum Status"))
        .stdout(predicate::str::contains("typedef int user_id"));
}

#[test]
fn test_func_and_call_predicates_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add | call:log_message")
        .assert()
        .success()
        .stdout(predicate::str::contains("int add(int a, int b)"))
        .stdout(predicate::str::contains("log_message(\"hello\")"));
}

#[test]
fn test_union_matches_struct_predicate_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Packet")
        .assert()
        .success()
        .stdout(predicate::str::contains("typedef union Packet"));
}

#[test]
fn test_import_and_macro_predicates_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:stdio & macro:MAX_BUFFER")
        .assert()
        .success()
        .stdout(predicate::str::contains("#include <stdio.h>"))
        .stdout(predicate::str::contains("#define MAX_BUFFER 1024"));
}

#[test]
fn test_comment_string_and_call_predicates_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:TODO & str:adding & call:use_util")
        .assert()
        .success()
        .stdout(predicate::str::contains("// TODO: add validation"))
        .stdout(predicate::str::contains("LOG(\"adding\")"))
        .stdout(predicate::str::contains("use_util(total)"));
}

#[test]
fn test_function_like_macro_predicate_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("macro:LOG_INT")
        .assert()
        .success()
        .stdout(predicate::str::contains("#define LOG_INT(x) log_message(#x)"));
}

#[test]
fn test_malformed_c_file_is_skipped_gracefully() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:. & path:bad.c")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_nonexistent_def_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:MissingThing & ext:c")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
