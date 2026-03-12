use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_cpp() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Point")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}

#[test]
fn test_func_predicate_cpp() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}

#[test]
fn test_import_predicate_cpp() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:util.hpp")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}

#[test]
fn test_call_predicate_cpp() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}

#[test]
fn test_str_predicate_cpp() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}

// =============================================================================
// CLASS AND STRUCT PREDICATE TESTS
// =============================================================================

#[test]
fn test_cpp_struct_point() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Point")
        .assert()
        .success()
        .stdout(predicate::str::contains("struct Point"));
}

#[test]
fn test_cpp_class_greeter() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Greeter")
        .assert()
        .success()
        .stdout(predicate::str::contains("class Greeter"));
}

// =============================================================================
// FUNC PREDICATE TESTS
// =============================================================================

#[test]
fn test_cpp_func_main() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("int main()"));
}

#[test]
fn test_cpp_func_add_namespace() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("add"));
}

// =============================================================================
// IMPORT PREDICATE TESTS
// =============================================================================

#[test]
fn test_cpp_import_iostream() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:iostream")
        .assert()
        .success()
        .stdout(predicate::str::contains("#include <iostream>"));
}

// =============================================================================
// CALL PREDICATE TESTS
// =============================================================================

#[test]
fn test_cpp_call_add() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("add("));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_cpp_class_and_str() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Greeter & str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}

#[test]
fn test_cpp_struct_and_func() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Point & func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}

#[test]
fn test_cpp_import_and_call() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:iostream & call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}

#[test]
fn test_cpp_multiple_or() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main | func:add | func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_cpp_format_paths() {
    let dir = setup_fixture("cpp_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("class:Greeter")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.cpp"));
}

#[test]
fn test_cpp_format_markdown() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("class:Greeter")
        .assert()
        .success()
        .stdout(predicate::str::contains("```cpp"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_cpp_custom_template_class() {
    let dir = setup_custom_project(&[(
        "container.cpp",
        r#"
template<typename T>
class Container {
public:
    Container(T val) : value(val) {}
    T get() const { return value; }
    void set(T val) { value = val; }
private:
    T value;
};
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Container")
        .assert()
        .success()
        .stdout(predicate::str::contains("container.cpp"));
}

#[test]
fn test_cpp_custom_inheritance() {
    let dir = setup_custom_project(&[(
        "shapes.cpp",
        r#"
class Shape {
public:
    virtual double area() const = 0;
    virtual ~Shape() = default;
};

class Circle : public Shape {
public:
    Circle(double r) : radius(r) {}
    double area() const override {
        return 3.14159 * radius * radius;
    }
private:
    double radius;
};
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Circle | class:Shape")
        .assert()
        .success()
        .stdout(predicate::str::contains("shapes.cpp"));
}

#[test]
fn test_cpp_custom_namespace() {
    let dir = setup_custom_project(&[(
        "utils.cpp",
        r#"
namespace utils {
    int add(int a, int b) {
        return a + b;
    }

    namespace math {
        double square(double x) {
            return x * x;
        }
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:square | func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.cpp"));
}

#[test]
fn test_cpp_custom_smart_pointers() {
    let dir = setup_custom_project(&[(
        "memory.cpp",
        r#"
#include <memory>

class Resource {
public:
    Resource() = default;
    void use() {}
};

std::unique_ptr<Resource> create() {
    return std::make_unique<Resource>();
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:memory & class:Resource")
        .assert()
        .success()
        .stdout(predicate::str::contains("memory.cpp"));
}

#[test]
fn test_cpp_custom_lambda() {
    let dir = setup_custom_project(&[(
        "lambda.cpp",
        r#"
#include <algorithm>
#include <vector>

void process(std::vector<int>& v) {
    std::sort(v.begin(), v.end(), [](int a, int b) {
        return a > b;
    });
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:process & import:algorithm")
        .assert()
        .success()
        .stdout(predicate::str::contains("lambda.cpp"));
}

#[test]
fn test_cpp_custom_operator_overload() {
    let dir = setup_custom_project(&[(
        "vector.cpp",
        r#"
struct Vector2D {
    double x, y;

    Vector2D operator+(const Vector2D& other) const {
        return {x + other.x, y + other.y};
    }

    Vector2D operator*(double scalar) const {
        return {x * scalar, y * scalar};
    }
};
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Vector2D")
        .assert()
        .success()
        .stdout(predicate::str::contains("struct Vector2D"));
}

#[test]
fn test_cpp_custom_constexpr() {
    let dir = setup_custom_project(&[(
        "compile_time.cpp",
        r#"
constexpr int factorial(int n) {
    return (n <= 1) ? 1 : n * factorial(n - 1);
}

constexpr int result = factorial(5);
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:factorial")
        .assert()
        .success()
        .stdout(predicate::str::contains("constexpr int factorial"));
}

#[test]
fn test_cpp_custom_enum_class() {
    let dir = setup_custom_project(&[(
        "enums.cpp",
        r#"
enum class Color {
    Red,
    Green,
    Blue
};

enum class Status : int {
    Ok = 0,
    Error = 1
};
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("enum:Color | enum:Status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Color"))
        .stdout(predicate::str::contains("Status"));
}

#[test]
fn test_cpp_custom_raii() {
    let dir = setup_custom_project(&[(
        "raii.cpp",
        r#"
#include <fstream>

class FileHandler {
public:
    FileHandler(const char* path) : file(path) {}
    ~FileHandler() { if(file.is_open()) file.close(); }
    void write(const char* data) { file << data; }
private:
    std::ofstream file;
};
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:FileHandler & import:fstream")
        .assert()
        .success()
        .stdout(predicate::str::contains("raii.cpp"));
}

// =============================================================================
// NEGATION TESTS
// =============================================================================

#[test]
fn test_cpp_class_not_comment() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Greeter & !comment:TODO")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.cpp"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_cpp_case_sensitive() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:greeter")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_cpp_not_found() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:NonExistent & ext:cpp")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_cpp_ext_filter() {
    let dir = setup_fixture("cpp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:cpp")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not())
        .stdout(predicate::str::contains(".py").not());
}
