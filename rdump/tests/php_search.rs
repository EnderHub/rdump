use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_php() {
    let dir = setup_fixture("php_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Greeter")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.php"));
}

#[test]
fn test_func_predicate_php() {
    let dir = setup_fixture("php_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.php"));
}

#[test]
fn test_import_predicate_php() {
    let dir = setup_fixture("php_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:Helper")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.php"));
}

#[test]
fn test_call_predicate_php() {
    let dir = setup_fixture("php_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.php"));
}

#[test]
fn test_str_predicate_php() {
    let dir = setup_fixture("php_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.php"));
}

// =============================================================================
// CLASS PREDICATE TESTS
// =============================================================================

#[test]
fn test_php_class_greeter() {
    let dir = setup_fixture("php_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Greeter")
        .assert()
        .success()
        .stdout(predicate::str::contains("class Greeter"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_php_class_and_str() {
    let dir = setup_fixture("php_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Greeter & str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.php"));
}

#[test]
fn test_php_or_operations() {
    let dir = setup_fixture("php_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add | func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.php"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_php_custom_interface() {
    let dir = setup_custom_project(&[(
        "service.php",
        r#"<?php
interface ServiceInterface {
    public function execute();
}

class MyService implements ServiceInterface {
    public function execute() {
        return "done";
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("interface:ServiceInterface & class:MyService")
        .assert()
        .success()
        .stdout(predicate::str::contains("service.php"));
}

#[test]
fn test_php_custom_trait() {
    let dir = setup_custom_project(&[(
        "traits.php",
        r#"<?php
trait Loggable {
    public function log($message) {
        echo $message;
    }
}

class App {
    use Loggable;
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("trait:Loggable & class:App")
        .assert()
        .success()
        .stdout(predicate::str::contains("traits.php"));
}

#[test]
fn test_php_custom_namespace() {
    let dir = setup_custom_project(&[(
        "namespaced.php",
        r#"<?php
namespace App\Services;

class UserService {
    public function find($id) {
        return null;
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:UserService & ext:php")
        .assert()
        .success()
        .stdout(predicate::str::contains("namespaced.php"));
}

#[test]
fn test_php_custom_static_method() {
    let dir = setup_custom_project(&[(
        "utils.php",
        r#"<?php
class Utils {
    public static function format($str) {
        return trim($str);
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Utils & ext:php")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.php"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_php_not_found() {
    let dir = setup_fixture("php_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:NonExistent & ext:php")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
