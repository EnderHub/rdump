use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_scala() {
    let dir = setup_fixture("scala_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Greeter")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));
}

#[test]
fn test_func_predicate_scala() {
    let dir = setup_fixture("scala_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));
}

#[test]
fn test_import_predicate_scala() {
    let dir = setup_fixture("scala_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:Helper")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));
}

#[test]
fn test_call_predicate_scala() {
    let dir = setup_fixture("scala_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));
}

#[test]
fn test_trait_predicate_scala() {
    let dir = setup_fixture("scala_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("trait:Greets")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));
}

// =============================================================================
// CLASS PREDICATE TESTS
// =============================================================================

#[test]
fn test_scala_class_greeter() {
    let dir = setup_fixture("scala_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Greeter")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_scala_class_and_func() {
    let dir = setup_fixture("scala_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Greeter & func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));
}

#[test]
fn test_scala_or_operations() {
    let dir = setup_fixture("scala_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add | func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.scala"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_scala_custom_trait() {
    let dir = setup_custom_project(&[(
        "traits.scala",
        r#"
trait Printable {
  def print(): Unit
}

class Document(content: String) extends Printable {
  def print(): Unit = {
    println(content)
  }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("trait:Printable & class:Document")
        .assert()
        .success()
        .stdout(predicate::str::contains("traits.scala"));
}

#[test]
fn test_scala_custom_case_class() {
    let dir = setup_custom_project(&[(
        "models.scala",
        r#"
case class Person(name: String, age: Int)

case class Address(street: String, city: String, zip: String)

object PersonUtils {
  def createPerson(name: String, age: Int): Person = {
    Person(name, age)
  }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Person & object:PersonUtils")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.scala"));
}

#[test]
fn test_scala_custom_object() {
    let dir = setup_custom_project(&[(
        "singleton.scala",
        r#"
object Calculator {
  def add(a: Int, b: Int): Int = a + b
  def subtract(a: Int, b: Int): Int = a - b
  def multiply(a: Int, b: Int): Int = a * b
  def divide(a: Int, b: Int): Double = a.toDouble / b
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("object:Calculator & func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("singleton.scala"));
}

#[test]
fn test_scala_custom_sealed_trait() {
    let dir = setup_custom_project(&[(
        "sealed.scala",
        r#"
sealed trait Result[+T]
case class Success[T](value: T) extends Result[T]
case class Failure(error: String) extends Result[Nothing]

object Result {
  def success[T](value: T): Result[T] = Success(value)
  def failure(error: String): Result[Nothing] = Failure(error)
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("trait:Result & class:Success")
        .assert()
        .success()
        .stdout(predicate::str::contains("sealed.scala"));
}

#[test]
fn test_scala_custom_implicit() {
    let dir = setup_custom_project(&[(
        "implicits.scala",
        r#"
object Implicits {
  implicit class StringOps(s: String) {
    def toIntOpt: Option[Int] = {
      try {
        Some(s.toInt)
      } catch {
        case _: NumberFormatException => None
      }
    }
  }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("object:Implicits & class:StringOps")
        .assert()
        .success()
        .stdout(predicate::str::contains("implicits.scala"));
}

#[test]
fn test_scala_custom_pattern_matching() {
    let dir = setup_custom_project(&[(
        "patterns.scala",
        r#"
object Matcher {
  def describe(x: Any): String = x match {
    case i: Int if i > 0 => "positive integer"
    case i: Int => "non-positive integer"
    case s: String => s"string: $s"
    case _ => "unknown"
  }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("object:Matcher & func:describe")
        .assert()
        .success()
        .stdout(predicate::str::contains("patterns.scala"));
}

#[test]
fn test_scala_custom_for_comprehension() {
    let dir = setup_custom_project(&[(
        "comprehensions.scala",
        r#"
object ListOps {
  def combine(xs: List[Int], ys: List[Int]): List[(Int, Int)] = {
    for {
      x <- xs
      y <- ys
    } yield (x, y)
  }

  def filter(xs: List[Int]): List[Int] = {
    for {
      x <- xs
      if x > 0
    } yield x * 2
  }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("object:ListOps & func:combine")
        .assert()
        .success()
        .stdout(predicate::str::contains("comprehensions.scala"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_scala_format_paths() {
    let dir = setup_fixture("scala_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("class:Greeter")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Main.scala"));
}

#[test]
fn test_scala_format_markdown() {
    let dir = setup_fixture("scala_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("```scala"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_scala_not_found() {
    let dir = setup_fixture("scala_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:NonExistent & ext:scala")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_scala_ext_filter() {
    let dir = setup_fixture("scala_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:scala")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
