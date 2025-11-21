use predicates::prelude::*;
mod common;
use common::setup_test_project;

#[test]
fn test_def_finds_struct_in_correct_file() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("def:Cli"); // Query for the Cli struct

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("struct Cli"))
        .stdout(predicate::str::contains("src/lib.rs").not());
}

#[test]
fn test_def_finds_enum_in_correct_file() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("def:Role"); // Query for the Role enum

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("pub enum Role"))
        .stdout(predicate::str::contains("src/main.rs").not());
}

#[test]
fn test_def_with_ext_predicate_and_paths_format() {
    let dir = setup_test_project();
    let root = dir.path();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(root);
    cmd.arg("search").arg("def:User & ext:rs");
    cmd.arg("--format=paths");

    // Normalize path for cross-platform compatibility
    let expected_path_str = format!("src{}lib.rs", std::path::MAIN_SEPARATOR);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(expected_path_str));
}

#[test]
fn test_def_returns_no_matches_for_non_existent_item() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("def:NonExistent");

    // Should succeed with no output
    cmd.assert().success().stdout(predicate::str::is_empty());
}

#[test]
fn test_def_does_not_match_in_non_rust_files() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    // The README.md contains the words "Role" and "User"
    cmd.arg("search").arg("def:Role | def:User");

    // It should ONLY find src/lib.rs, not README.md
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("README.md").not());
}

#[test]
fn test_func_finds_standalone_function() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("func:main");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/lib.rs").not());
}

#[test]
fn test_func_finds_impl_method() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("func:new");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("src/main.rs"));
}

#[test]
fn test_import_finds_use_statement() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search")
        .arg("--format=markdown")
        .arg("import:serde");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("```rs")) // Check for markdown code fence
        .stdout(predicate::str::contains("src/main.rs").not());
}

#[test]
fn test_logical_or_across_files() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("func:main | import:serde");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/lib.rs"));
}

#[test]
fn test_comment_predicate_rust() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:TODO")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/lib.rs").not());
}

#[test]
fn test_str_predicate_rust() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:\"Hello, world!\"")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

#[test]
fn test_type_and_struct_predicates_rust() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("type:UserId & struct:User")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("src/main.rs").not());
}

#[test]
fn test_call_predicate_rust() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:println & ext:rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs")) // The call is in main.rs
        .stdout(predicate::str::contains("src/lib.rs").not()); // The definition is in lib.rs
}

#[test]
fn test_logical_operators_with_hunks() {
    let dir = setup_test_project();
    // Query: find the file that defines the `Cli` struct AND ALSO contains a `TODO` comment.
    // This should only match main.rs
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=hunks")
        .arg("struct:Cli & comment:TODO")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/lib.rs").not());
}

#[test]
fn test_negation_with_hunks() {
    let dir = setup_test_project();
    // Query: find files with `User` struct but NOT containing `TODO`
    // This should only match lib.rs
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=hunks")
        .arg("struct:User & !comment:TODO")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("User"))
        .stdout(predicate::str::contains("src/main.rs").not());
}

#[test]
fn test_and_of_semantic_predicates() {
    let dir = setup_test_project();
    // Query: find files with a `struct` AND a `func`
    // This should only match lib.rs (User struct, new function)
    // and main.rs (Cli struct, main function)
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search");
    cmd.arg("--format=paths");
    cmd.arg("struct:. & func:. & ext:rs");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "Expected exactly 3 files, but found {}: {:?}",
        lines.len(),
        lines
    );
}

#[test]
fn test_func_not_found() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:non_existent_function")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// ADDITIONAL PREDICATE TESTS
// =============================================================================

#[test]
fn test_enum_predicate_rust() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("enum:Role")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("pub enum Role"));
}

#[test]
fn test_trait_predicate_rust() {
    let dir = common::setup_custom_project(&[(
        "traits.rs",
        r#"
pub trait Greet {
    fn greet(&self) -> String;
}

pub struct Person {
    name: String,
}

impl Greet for Person {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name)
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("trait:Greet & struct:Person")
        .assert()
        .success()
        .stdout(predicate::str::contains("traits.rs"));
}

#[test]
fn test_macro_predicate_rust() {
    let dir = common::setup_custom_project(&[(
        "macros.rs",
        r#"
macro_rules! say_hello {
    () => {
        println!("Hello!");
    };
}

macro_rules! create_function {
    ($name:ident) => {
        fn $name() {
            println!("Created function: {}", stringify!($name));
        }
    };
}

fn main() {
    say_hello!();
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("macro:say_hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("macros.rs"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_rust_custom_generic_struct() {
    let dir = common::setup_custom_project(&[(
        "generic.rs",
        r#"
pub struct Container<T> {
    value: T,
}

impl<T> Container<T> {
    pub fn new(value: T) -> Self {
        Container { value }
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Container & func:new")
        .assert()
        .success()
        .stdout(predicate::str::contains("generic.rs"));
}

#[test]
fn test_rust_custom_async_function() {
    let dir = common::setup_custom_project(&[(
        "async_code.rs",
        r#"
use std::future::Future;

async fn fetch_data() -> String {
    "data".to_string()
}

async fn process() {
    let data = fetch_data().await;
    println!("{}", data);
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:fetch_data | func:process")
        .assert()
        .success()
        .stdout(predicate::str::contains("async_code.rs"));
}

#[test]
fn test_rust_custom_result_handling() {
    let dir = common::setup_custom_project(&[(
        "errors.rs",
        r#"
use std::io;

pub enum AppError {
    IoError(io::Error),
    ParseError(String),
}

pub fn read_file(path: &str) -> Result<String, AppError> {
    std::fs::read_to_string(path)
        .map_err(AppError::IoError)
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("enum:AppError & func:read_file")
        .assert()
        .success()
        .stdout(predicate::str::contains("errors.rs"));
}

#[test]
fn test_rust_custom_lifetime_annotations() {
    let dir = common::setup_custom_project(&[(
        "lifetimes.rs",
        r#"
pub struct Ref<'a, T> {
    value: &'a T,
}

impl<'a, T> Ref<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Ref { value }
    }
}

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Ref & func:longest")
        .assert()
        .success()
        .stdout(predicate::str::contains("lifetimes.rs"));
}

#[test]
fn test_rust_custom_derive_macro() {
    let dir = common::setup_custom_project(&[(
        "derive.rs",
        r#"
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Point | enum:Direction")
        .assert()
        .success()
        .stdout(predicate::str::contains("derive.rs"));
}

#[test]
fn test_rust_custom_module_structure() {
    let dir = common::setup_custom_project(&[(
        "modules.rs",
        r#"
mod inner {
    pub struct InnerStruct {
        pub value: i32,
    }

    pub fn inner_function() -> i32 {
        42
    }
}

pub use inner::InnerStruct;

fn main() {
    let s = inner::InnerStruct { value: 10 };
    let v = inner::inner_function();
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:InnerStruct & func:inner_function")
        .assert()
        .success()
        .stdout(predicate::str::contains("modules.rs"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_rust_format_markdown() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("struct:Cli")
        .assert()
        .success()
        .stdout(predicate::str::contains("```rs"));
}

#[test]
fn test_rust_format_paths() {
    let dir = setup_test_project();
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("struct:User")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("lib.rs"));
}

// =============================================================================
// COMPLEX COMBINATION TESTS
// =============================================================================

#[test]
fn test_rust_complex_and_or() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("(struct:Cli | struct:User) & func:new")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/lib.rs"));
}

#[test]
fn test_rust_negation_complex() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:new & !struct:Cli & ext:rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/lib.rs"));
}

#[test]
fn test_rust_ext_filter() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:rs")
        .assert()
        .success()
        .stdout(predicate::str::contains(".md").not());
}
