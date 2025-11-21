use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_csharp() {
    let dir = setup_fixture("csharp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Greeter")
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));
}

#[test]
fn test_func_predicate_csharp() {
    let dir = setup_fixture("csharp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:Main")
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));
}

#[test]
fn test_import_predicate_csharp() {
    let dir = setup_fixture("csharp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:System")
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));
}

#[test]
fn test_call_predicate_csharp() {
    let dir = setup_fixture("csharp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:Greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));
}

#[test]
fn test_str_predicate_csharp() {
    let dir = setup_fixture("csharp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));
}

// =============================================================================
// CLASS PREDICATE TESTS
// =============================================================================

#[test]
fn test_csharp_class_greeter() {
    let dir = setup_fixture("csharp_project");
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
fn test_csharp_class_and_func() {
    let dir = setup_fixture("csharp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Greeter & func:Greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));
}

#[test]
fn test_csharp_import_and_call() {
    let dir = setup_fixture("csharp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:System & call:WriteLine")
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));
}

#[test]
fn test_csharp_or_operations() {
    let dir = setup_fixture("csharp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:Main | func:Greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Program.cs"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_csharp_format_paths() {
    let dir = setup_fixture("csharp_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("class:Greeter")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Program.cs"));
}

#[test]
fn test_csharp_format_markdown() {
    let dir = setup_fixture("csharp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("func:Main")
        .assert()
        .success()
        .stdout(predicate::str::contains("```cs"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_csharp_custom_interface() {
    let dir = setup_custom_project(&[(
        "Service.cs",
        r#"
namespace App;

public interface IService {
    void Execute();
    string GetName();
}

public class ServiceImpl : IService {
    public void Execute() { }
    public string GetName() => "Service";
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("interface:IService & class:ServiceImpl")
        .assert()
        .success()
        .stdout(predicate::str::contains("Service.cs"));
}

#[test]
fn test_csharp_custom_async_method() {
    let dir = setup_custom_project(&[(
        "Async.cs",
        r#"
using System.Threading.Tasks;

public class AsyncService {
    public async Task<string> FetchDataAsync() {
        await Task.Delay(100);
        return "data";
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:FetchDataAsync & import:Threading")
        .assert()
        .success()
        .stdout(predicate::str::contains("Async.cs"));
}

#[test]
fn test_csharp_custom_generic_class() {
    let dir = setup_custom_project(&[(
        "Generic.cs",
        r#"
using System.Collections.Generic;

public class Repository<T> {
    private List<T> items = new List<T>();

    public void Add(T item) {
        items.Add(item);
    }

    public T Get(int index) {
        return items[index];
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Repository & func:Add")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generic.cs"));
}

#[test]
fn test_csharp_custom_properties() {
    let dir = setup_custom_project(&[(
        "Model.cs",
        r#"
public class User {
    public int Id { get; set; }
    public string Name { get; set; }
    public string Email { get; private set; }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:User")
        .assert()
        .success()
        .stdout(predicate::str::contains("Model.cs"));
}

#[test]
fn test_csharp_custom_linq() {
    let dir = setup_custom_project(&[(
        "Query.cs",
        r#"
using System.Linq;
using System.Collections.Generic;

public class QueryService {
    public List<int> FilterEven(List<int> numbers) {
        return numbers.Where(n => n % 2 == 0).ToList();
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:Linq & func:FilterEven")
        .assert()
        .success()
        .stdout(predicate::str::contains("Query.cs"));
}

#[test]
fn test_csharp_custom_enum() {
    let dir = setup_custom_project(&[(
        "Enums.cs",
        r#"
public enum Status {
    Pending,
    Active,
    Completed
}

public enum Priority {
    Low = 1,
    Medium = 2,
    High = 3
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("enum:Status | enum:Priority")
        .assert()
        .success()
        .stdout(predicate::str::contains("Enums.cs"));
}

#[test]
fn test_csharp_custom_static_class() {
    let dir = setup_custom_project(&[(
        "Utils.cs",
        r#"
public static class StringUtils {
    public static string Reverse(string s) {
        char[] arr = s.ToCharArray();
        Array.Reverse(arr);
        return new string(arr);
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:StringUtils & func:Reverse")
        .assert()
        .success()
        .stdout(predicate::str::contains("Utils.cs"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_csharp_not_found() {
    let dir = setup_fixture("csharp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:NonExistent & ext:cs")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_csharp_ext_filter() {
    let dir = setup_fixture("csharp_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:cs")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
