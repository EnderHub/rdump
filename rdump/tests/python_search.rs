use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture, setup_test_project};

// =============================================================================
// BASIC PREDICATE TESTS - Individual predicates
// =============================================================================

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

// =============================================================================
// PYTHON PROJECT FIXTURE TESTS - Using dedicated Python fixtures
// =============================================================================

#[test]
fn test_python_class_in_python_project() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:User")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"))
        .stdout(predicate::str::contains("class User"));
}

#[test]
fn test_python_class_userservice() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:UserService")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"))
        .stdout(predicate::str::contains("class UserService"));
}

#[test]
fn test_python_class_configloader() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:ConfigLoader")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"))
        .stdout(predicate::str::contains("class ConfigLoader"));
}

#[test]
fn test_python_func_validate_email() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:validate_email")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"))
        .stdout(predicate::str::contains("def validate_email"));
}

#[test]
fn test_python_func_format_name() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:format_name")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"));
}

#[test]
fn test_python_func_add_user() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add_user")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"))
        .stdout(predicate::str::contains("def add_user"));
}

#[test]
fn test_python_async_func_fetch_user() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:fetch_user")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"))
        .stdout(predicate::str::contains("async def fetch_user"));
}

#[test]
fn test_python_func_create_admin() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:create_admin")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"));
}

#[test]
fn test_python_import_re() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:re")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"));
}

#[test]
fn test_python_import_typing() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:typing")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"));
}

#[test]
fn test_python_import_dataclasses() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:dataclasses")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"));
}

#[test]
fn test_python_from_import() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:dataclass")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"));
}

#[test]
fn test_python_comment_todo() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:TODO")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"));
}

#[test]
fn test_python_comment_hack() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:HACK")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"));
}

#[test]
fn test_python_str_email_pattern() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:@")
        .assert()
        .success()
        .stdout(predicate::str::contains(".py"));
}

#[test]
fn test_python_str_admin() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:admin")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"));
}

#[test]
fn test_python_call_append() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:append")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"));
}

#[test]
fn test_python_call_match() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:match")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"));
}

// =============================================================================
// COMBINATION TESTS - AND operations
// =============================================================================

#[test]
fn test_python_class_and_func() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:User & func:add_user")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"));
}

#[test]
fn test_python_import_and_class() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:dataclass & class:User")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"));
}

#[test]
fn test_python_comment_and_func() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:TODO & func:fetch_user")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"));
}

#[test]
fn test_python_func_and_call() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:validate_email & call:match")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"));
}

#[test]
fn test_python_class_and_comment() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:ConfigLoader & comment:HACK")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"));
}

#[test]
fn test_python_multiple_and() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:UserService & func:add_user & import:List")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"));
}

// =============================================================================
// COMBINATION TESTS - OR operations
// =============================================================================

#[test]
fn test_python_class_or_func() {
    let dir = setup_fixture("python_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("class:User | func:validate_email")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("models.py") || stdout.contains("utils.py"));
}

#[test]
fn test_python_multiple_or() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:validate_email | func:format_name | func:create_admin")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py").or(predicate::str::contains("models.py")));
}

#[test]
fn test_python_import_or() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("import:re | import:dataclasses")
        .assert()
        .success();
}

// =============================================================================
// NEGATION TESTS
// =============================================================================

#[test]
fn test_python_class_not_func() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:ConfigLoader & !func:add_user")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"));
}

#[test]
fn test_python_func_not_comment() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:validate_email & !comment:TODO")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"));
}

#[test]
fn test_python_import_not_class() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:re & !class:User")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_python_format_paths() {
    let dir = setup_fixture("python_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("class:User")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("models.py"));
    assert!(!stdout.contains("class User"));
}

#[test]
fn test_python_format_markdown() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("class:User")
        .assert()
        .success()
        .stdout(predicate::str::contains("```py"));
}

#[test]
fn test_python_format_hunks() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=hunks")
        .arg("class:UserService")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.py"));
}

// =============================================================================
// REGEX PATTERN TESTS
// =============================================================================

#[test]
fn test_python_func_add_user_direct() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add_user")
        .assert()
        .success()
        .stdout(predicate::str::contains(".py"));
}

#[test]
fn test_python_class_userservice_direct() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:UserService")
        .assert()
        .success()
        .stdout(predicate::str::contains("UserService"));
}

#[test]
fn test_python_func_regex_wildcard() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:.")
        .assert()
        .success();
}

// =============================================================================
// EXTENSION FILTER TESTS
// =============================================================================

#[test]
fn test_python_ext_filter() {
    let dir = setup_test_project();
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("func:. & ext:py")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    for line in stdout.lines() {
        assert!(line.ends_with(".py") || line.is_empty());
    }
}

#[test]
fn test_python_ext_excludes_other_langs() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:py")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not())
        .stdout(predicate::str::contains(".go").not())
        .stdout(predicate::str::contains(".java").not());
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_python_no_match_returns_empty() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:NonExistentClass123 & ext:py")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_python_case_sensitive_class() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:user")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_python_dunder_method() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:__init__")
        .assert()
        .success()
        .stdout(predicate::str::contains("__init__"));
}

#[test]
fn test_python_decorated_class() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:User & str:dataclass")
        .assert()
        .success();
}

// =============================================================================
// CUSTOM PROJECT TESTS - Testing specific patterns
// =============================================================================

#[test]
fn test_python_custom_multiple_classes() {
    let dir = setup_custom_project(&[(
        "app.py",
        r#"
class First:
    pass

class Second:
    pass

class Third:
    pass
"#,
    )]);

    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:.")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("First"));
    assert!(stdout.contains("Second"));
    assert!(stdout.contains("Third"));
}

#[test]
fn test_python_custom_nested_functions() {
    let dir = setup_custom_project(&[(
        "nested.py",
        r#"
def outer():
    def inner():
        pass
    return inner
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:outer")
        .assert()
        .success()
        .stdout(predicate::str::contains("def outer"));
}

#[test]
fn test_python_custom_lambda_not_matched_as_func() {
    let dir = setup_custom_project(&[(
        "lambdas.py",
        r#"
double = lambda x: x * 2
"#,
    )]);

    // func: should not match lambdas
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:double")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_python_custom_multiline_string() {
    let dir = setup_custom_project(&[(
        "docs.py",
        r#"
docstring = """
This is a multiline
docstring with special content
"""
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:multiline")
        .assert()
        .success()
        .stdout(predicate::str::contains("docs.py"));
}

#[test]
fn test_python_custom_f_string() {
    let dir = setup_custom_project(&[(
        "formatted.py",
        r#"
name = "world"
message = f"Hello {name}!"
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("formatted.py"));
}

#[test]
fn test_python_custom_relative_import() {
    let dir = setup_custom_project(&[(
        "pkg/module.py",
        r#"
from . import sibling
from ..parent import something
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:sibling")
        .assert()
        .success()
        .stdout(predicate::str::contains("module.py"));
}

#[test]
fn test_python_custom_starred_import() {
    let dir = setup_custom_project(&[(
        "wildcard.py",
        r#"
from os.path import *
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:os.path")
        .assert()
        .success()
        .stdout(predicate::str::contains("wildcard.py"));
}

#[test]
fn test_python_custom_method_call_chain() {
    let dir = setup_custom_project(&[(
        "chains.py",
        r#"
result = obj.method1().method2().method3()
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:method2")
        .assert()
        .success()
        .stdout(predicate::str::contains("chains.py"));
}

#[test]
fn test_python_custom_static_method() {
    let dir = setup_custom_project(&[(
        "static.py",
        r#"
class Utils:
    @staticmethod
    def helper():
        pass

    @classmethod
    def factory(cls):
        pass
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:helper & class:Utils")
        .assert()
        .success()
        .stdout(predicate::str::contains("static.py"));
}

#[test]
fn test_python_custom_async_comprehension() {
    let dir = setup_custom_project(&[(
        "async_comp.py",
        r#"
async def gather_data():
    results = [await fetch(x) async for x in sources]
    return results
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:gather_data & call:fetch")
        .assert()
        .success()
        .stdout(predicate::str::contains("async_comp.py"));
}

// =============================================================================
// COMPLEX QUERY TESTS
// =============================================================================

#[test]
fn test_python_complex_and_or() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("(class:User | class:ConfigLoader) & import:typing")
        .assert()
        .success();
}

#[test]
fn test_python_complex_nested_parens() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("((func:validate_email | func:format_name) & import:re)")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.py"));
}

#[test]
fn test_python_complex_multiple_negations() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & !class:User & !comment:TODO")
        .assert()
        .success();
}

// =============================================================================
// DEF PREDICATE COMPREHENSIVE TESTS
// =============================================================================

#[test]
fn test_python_def_matches_class_and_func() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:User")
        .assert()
        .success()
        .stdout(predicate::str::contains("class User"));
}

#[test]
fn test_python_def_function() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:validate_email")
        .assert()
        .success()
        .stdout(predicate::str::contains("def validate_email"));
}

#[test]
fn test_python_def_configloader() {
    let dir = setup_fixture("python_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:ConfigLoader")
        .assert()
        .success()
        .stdout(predicate::str::contains("ConfigLoader"));
}

// =============================================================================
// COUNT AND STATISTICS TESTS
// =============================================================================

#[test]
fn test_python_multiple_files_matched() {
    let dir = setup_fixture("python_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("func:.")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let file_count = stdout.lines().filter(|l| !l.is_empty()).count();
    assert!(file_count >= 2, "Expected at least 2 files with functions");
}

#[test]
fn test_python_all_classes_count() {
    let dir = setup_fixture("python_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("class:.")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let file_count = stdout.lines().filter(|l| !l.is_empty()).count();
    assert!(file_count >= 2, "Expected at least 2 files with classes");
}
