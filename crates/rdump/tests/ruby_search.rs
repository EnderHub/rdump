use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_ruby() {
    let dir = setup_fixture("ruby_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Greeter")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));
}

#[test]
fn test_func_predicate_ruby() {
    let dir = setup_fixture("ruby_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));
}

#[test]
fn test_import_predicate_ruby() {
    let dir = setup_fixture("ruby_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:require")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));
}

#[test]
fn test_call_predicate_ruby() {
    let dir = setup_fixture("ruby_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));
}

#[test]
fn test_str_predicate_ruby() {
    let dir = setup_fixture("ruby_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));
}

// =============================================================================
// CLASS PREDICATE TESTS
// =============================================================================

#[test]
fn test_ruby_class_greeter() {
    let dir = setup_fixture("ruby_project");
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
fn test_ruby_class_and_func() {
    let dir = setup_fixture("ruby_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Greeter & func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));
}

#[test]
fn test_ruby_or_operations() {
    let dir = setup_fixture("ruby_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:greet | class:Greeter")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rb"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_ruby_custom_module() {
    let dir = setup_custom_project(&[(
        "utils.rb",
        r#"
module StringUtils
  def self.reverse(str)
    str.reverse
  end
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("module:StringUtils & func:reverse")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.rb"));
}

#[test]
fn test_ruby_custom_attr_accessor() {
    let dir = setup_custom_project(&[(
        "model.rb",
        r#"
class User
  attr_accessor :name, :email

  def initialize(name, email)
    @name = name
    @email = email
  end
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:User & func:initialize")
        .assert()
        .success()
        .stdout(predicate::str::contains("model.rb"));
}

#[test]
fn test_ruby_custom_block() {
    let dir = setup_custom_project(&[(
        "blocks.rb",
        r#"
class Collection
  def each(&block)
    @items.each(&block)
  end

  def map(&block)
    @items.map(&block)
  end
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Collection & func:each")
        .assert()
        .success()
        .stdout(predicate::str::contains("blocks.rb"));
}

#[test]
fn test_ruby_custom_inheritance() {
    let dir = setup_custom_project(&[(
        "inheritance.rb",
        r#"
class Animal
  def speak
    raise NotImplementedError
  end
end

class Dog < Animal
  def speak
    "Woof!"
  end
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Dog | class:Animal")
        .assert()
        .success()
        .stdout(predicate::str::contains("inheritance.rb"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_ruby_not_found() {
    let dir = setup_fixture("ruby_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:NonExistent & ext:rb")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
