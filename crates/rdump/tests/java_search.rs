use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_test_project};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_class_predicate_java() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Application & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"))
        .stdout(predicate::str::contains("public class Application"));
}

#[test]
fn test_func_and_call_predicates_java() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main & call:println")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"));
}

#[test]
fn test_import_and_comment_predicates_java() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:ArrayList & comment:HACK")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"));
}

#[test]
fn test_str_predicate_java() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:\"Hello from Java!\"")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"));
}

#[test]
fn test_class_not_found_java() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:NonExistent & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// FUNC PREDICATE TESTS
// =============================================================================

#[test]
fn test_java_func_main() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("public static void main"));
}

#[test]
fn test_java_func_regex() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:.*main.* & ext:java")
        .assert()
        .success();
}

// =============================================================================
// CLASS PREDICATE TESTS
// =============================================================================

#[test]
fn test_java_class_wildcard() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:. & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application"));
}

#[test]
fn test_java_class_application_name() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Application & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application"));
}

// =============================================================================
// IMPORT PREDICATE TESTS
// =============================================================================

#[test]
fn test_java_import_arraylist() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:ArrayList & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("import java.util.ArrayList"));
}

#[test]
fn test_java_import_not_found() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:NonExistent & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// CALL PREDICATE TESTS
// =============================================================================

#[test]
fn test_java_call_println() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:println & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("println"));
}

// =============================================================================
// COMMENT PREDICATE TESTS
// =============================================================================

#[test]
fn test_java_comment_hack() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:HACK & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("HACK"));
}

#[test]
fn test_java_comment_main() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:Main & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main application"));
}

// =============================================================================
// STRING PREDICATE TESTS
// =============================================================================

#[test]
fn test_java_str_hello() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_java_class_and_func() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Application & func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"));
}

#[test]
fn test_java_import_and_call() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:ArrayList & call:println")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"));
}

#[test]
fn test_java_multiple_and() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Application & func:main & import:ArrayList")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"));
}

#[test]
fn test_java_or_operations() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main | class:Application")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"));
}

// =============================================================================
// NEGATION TESTS
// =============================================================================

#[test]
fn test_java_class_not_comment() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Application & !comment:TODO")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_java_format_paths() {
    let dir = setup_test_project();
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("class:Application & ext:java")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Application.java"));
    assert!(!stdout.contains("public class"));
}

#[test]
fn test_java_format_markdown() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("class:Application & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("```java"));
}

#[test]
fn test_java_format_hunks() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=hunks")
        .arg("func:main & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"));
}

// =============================================================================
// DEF PREDICATE TESTS
// =============================================================================

#[test]
fn test_java_def_class() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Application & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_java_custom_interface() {
    let dir = setup_custom_project(&[(
        "Service.java",
        r#"package com.example;

public interface Service {
    void execute();
    String getName();
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("interface:Service")
        .assert()
        .success()
        .stdout(predicate::str::contains("public interface Service"));
}

#[test]
fn test_java_custom_multiple_classes() {
    let dir = setup_custom_project(&[(
        "Models.java",
        r#"package com.example;

class User {
    private int id;
    private String name;
}

class Product {
    private int id;
    private double price;
}

class Order {
    private int id;
    private User user;
}
"#,
    )]);

    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:.")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("User"));
    assert!(stdout.contains("Product"));
    assert!(stdout.contains("Order"));
}

#[test]
fn test_java_custom_methods() {
    let dir = setup_custom_project(&[(
        "Calculator.java",
        r#"package com.example;

public class Calculator {
    public int add(int a, int b) {
        return a + b;
    }

    public int subtract(int a, int b) {
        return a - b;
    }

    public int multiply(int a, int b) {
        return a * b;
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add & class:Calculator")
        .assert()
        .success()
        .stdout(predicate::str::contains("public int add"));
}

#[test]
fn test_java_custom_annotations() {
    let dir = setup_custom_project(&[(
        "Controller.java",
        r#"package com.example;

import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api")
public class Controller {
    @GetMapping("/users")
    public String getUsers() {
        return "users";
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:springframework & class:Controller")
        .assert()
        .success()
        .stdout(predicate::str::contains("Controller.java"));
}

#[test]
fn test_java_custom_generics() {
    let dir = setup_custom_project(&[(
        "Repository.java",
        r#"package com.example;

import java.util.List;
import java.util.Optional;

public class Repository<T> {
    public Optional<T> findById(int id) {
        return Optional.empty();
    }

    public List<T> findAll() {
        return List.of();
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Repository & import:Optional")
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository.java"));
}

#[test]
fn test_java_custom_enum() {
    let dir = setup_custom_project(&[(
        "Status.java",
        r#"package com.example;

public enum Status {
    PENDING,
    ACTIVE,
    COMPLETED,
    CANCELLED
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("enum:Status")
        .assert()
        .success()
        .stdout(predicate::str::contains("enum Status"));
}

#[test]
fn test_java_custom_abstract_class() {
    let dir = setup_custom_project(&[(
        "BaseEntity.java",
        r#"package com.example;

public abstract class BaseEntity {
    protected int id;

    public abstract void validate();

    public int getId() {
        return id;
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:BaseEntity & func:validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("abstract class BaseEntity"));
}

#[test]
fn test_java_custom_static_methods() {
    let dir = setup_custom_project(&[(
        "Utils.java",
        r#"package com.example;

public class Utils {
    public static String format(String s) {
        return s.trim().toLowerCase();
    }

    public static int parse(String s) {
        return Integer.parseInt(s);
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:format | func:parse")
        .assert()
        .success()
        .stdout(predicate::str::contains("Utils.java"));
}

#[test]
fn test_java_custom_constructor() {
    let dir = setup_custom_project(&[(
        "Person.java",
        r#"package com.example;

public class Person {
    private String name;
    private int age;

    public Person(String name, int age) {
        this.name = name;
        this.age = age;
    }

    public String getName() {
        return name;
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Person & func:getName")
        .assert()
        .success()
        .stdout(predicate::str::contains("Person.java"));
}

#[test]
fn test_java_custom_exception() {
    let dir = setup_custom_project(&[(
        "CustomException.java",
        r#"package com.example;

public class CustomException extends RuntimeException {
    public CustomException(String message) {
        super(message);
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:CustomException")
        .assert()
        .success()
        .stdout(predicate::str::contains("extends RuntimeException"));
}

#[test]
fn test_java_custom_lambda() {
    let dir = setup_custom_project(&[(
        "Lambda.java",
        r#"package com.example;

import java.util.List;
import java.util.stream.Collectors;

public class Lambda {
    public List<String> filter(List<String> items) {
        return items.stream()
            .filter(s -> s.length() > 3)
            .collect(Collectors.toList());
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:stream & call:filter")
        .assert()
        .success()
        .stdout(predicate::str::contains("Lambda.java"));
}

#[test]
fn test_java_custom_inner_class() {
    let dir = setup_custom_project(&[(
        "Outer.java",
        r#"package com.example;

public class Outer {
    private int value;

    public class Inner {
        public int getValue() {
            return value;
        }
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Outer | class:Inner")
        .assert()
        .success()
        .stdout(predicate::str::contains("class Outer"))
        .stdout(predicate::str::contains("class Inner"));
}

// =============================================================================
// COMPLEX QUERY TESTS
// =============================================================================

#[test]
fn test_java_complex_and_or() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("(class:Application | func:main) & import:ArrayList")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"));
}

#[test]
fn test_java_complex_nested() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("((class:Application & func:main) | comment:HACK) & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains("Application.java"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_java_case_sensitive() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:application & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_java_ext_filter() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:java")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not())
        .stdout(predicate::str::contains(".py").not())
        .stdout(predicate::str::contains(".go").not());
}
