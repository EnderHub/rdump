use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_swift() {
    let dir = setup_fixture("swift_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:ConsoleGreeter")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));
}

#[test]
fn test_func_predicate_swift() {
    let dir = setup_fixture("swift_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));
}

#[test]
fn test_import_predicate_swift() {
    let dir = setup_fixture("swift_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:Foundation")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));
}

#[test]
fn test_call_predicate_swift() {
    let dir = setup_fixture("swift_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));
}

#[test]
fn test_str_predicate_swift() {
    let dir = setup_fixture("swift_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));
}

// =============================================================================
// CLASS AND STRUCT PREDICATE TESTS
// =============================================================================

#[test]
fn test_swift_class_greeter() {
    let dir = setup_fixture("swift_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:ConsoleGreeter")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_swift_class_and_func() {
    let dir = setup_fixture("swift_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:ConsoleGreeter & func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));
}

#[test]
fn test_swift_or_operations() {
    let dir = setup_fixture("swift_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add | func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.swift"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_swift_custom_protocol() {
    let dir = setup_custom_project(&[(
        "protocols.swift",
        r#"
protocol Drawable {
    func draw()
}

class Circle: Drawable {
    var radius: Double

    init(radius: Double) {
        self.radius = radius
    }

    func draw() {
        print("Drawing circle with radius \(radius)")
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("protocol:Drawable & class:Circle")
        .assert()
        .success()
        .stdout(predicate::str::contains("protocols.swift"));
}

#[test]
fn test_swift_custom_struct() {
    let dir = setup_custom_project(&[(
        "models.swift",
        r#"
struct Point {
    var x: Double
    var y: Double

    func distance(to other: Point) -> Double {
        let dx = x - other.x
        let dy = y - other.y
        return sqrt(dx * dx + dy * dy)
    }
}

struct Rectangle {
    var origin: Point
    var width: Double
    var height: Double
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Point & def:Rectangle")
        .assert()
        .success()
        .stdout(predicate::str::contains("models.swift"));
}

#[test]
fn test_swift_custom_enum() {
    let dir = setup_custom_project(&[(
        "enums.swift",
        r#"
enum Direction {
    case north
    case south
    case east
    case west

    func opposite() -> Direction {
        switch self {
        case .north: return .south
        case .south: return .north
        case .east: return .west
        case .west: return .east
        }
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Direction & ext:swift")
        .assert()
        .success()
        .stdout(predicate::str::contains("enums.swift"));
}

#[test]
fn test_swift_custom_extension() {
    let dir = setup_custom_project(&[(
        "extensions.swift",
        r#"
extension String {
    func reversed() -> String {
        return String(self.reversed())
    }

    var isPalindrome: Bool {
        return self == String(self.reversed())
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:reversed & ext:swift")
        .assert()
        .success()
        .stdout(predicate::str::contains("extensions.swift"));
}

#[test]
fn test_swift_custom_generic() {
    let dir = setup_custom_project(&[(
        "generics.swift",
        r#"
class Stack<Element> {
    private var items: [Element] = []

    func push(_ item: Element) {
        items.append(item)
    }

    func pop() -> Element? {
        return items.popLast()
    }

    var isEmpty: Bool {
        return items.isEmpty
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Stack & func:push")
        .assert()
        .success()
        .stdout(predicate::str::contains("generics.swift"));
}

#[test]
fn test_swift_custom_closure() {
    let dir = setup_custom_project(&[(
        "closures.swift",
        r#"
func performOperation(_ a: Int, _ b: Int, operation: (Int, Int) -> Int) -> Int {
    return operation(a, b)
}

let add = { (a: Int, b: Int) -> Int in
    return a + b
}

let multiply = { (a: Int, b: Int) -> Int in
    return a * b
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:performOperation")
        .assert()
        .success()
        .stdout(predicate::str::contains("closures.swift"));
}

#[test]
fn test_swift_custom_optional() {
    let dir = setup_custom_project(&[(
        "optionals.swift",
        r#"
class Person {
    var name: String
    var address: String?

    init(name: String) {
        self.name = name
    }

    func formattedAddress() -> String {
        guard let addr = address else {
            return "No address"
        }
        return addr
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Person & func:formattedAddress")
        .assert()
        .success()
        .stdout(predicate::str::contains("optionals.swift"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_swift_format_paths() {
    let dir = setup_fixture("swift_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("class:ConsoleGreeter")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.swift"));
}

#[test]
fn test_swift_format_markdown() {
    let dir = setup_fixture("swift_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("```swift"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_swift_not_found() {
    let dir = setup_fixture("swift_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:NonExistent & ext:swift")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_swift_ext_filter() {
    let dir = setup_fixture("swift_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:swift")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
