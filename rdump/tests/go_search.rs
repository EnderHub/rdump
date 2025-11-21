use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_test_project};

// =============================================================================
// BASIC PREDICATE TESTS - Individual predicates
// =============================================================================

#[test]
fn test_struct_predicate_go() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Server & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.go"))
        .stdout(predicate::str::contains("type Server struct"));
}

#[test]
fn test_func_and_call_predicates_go() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:NewServer | call:NewServer")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "func NewServer(addr string) *Server",
        ))
        .stdout(predicate::str::contains("server := NewServer(\":8080\")"));
}

#[test]
fn test_import_and_comment_predicates_go() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:fmt & comment:\"HTTP server\"")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.go"));
}

#[test]
fn test_struct_not_found_go() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:NonExistent & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// FUNC PREDICATE TESTS
// =============================================================================

#[test]
fn test_go_func_main() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("func main()"));
}

#[test]
fn test_go_func_newserver() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:NewServer")
        .assert()
        .success()
        .stdout(predicate::str::contains("func NewServer"));
}

#[test]
fn test_go_func_newserver_exists() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:NewServer & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("NewServer"));
}

#[test]
fn test_go_func_wildcard() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:go")
        .assert()
        .success();
}

// =============================================================================
// STRUCT AND TYPE PREDICATE TESTS
// =============================================================================

#[test]
fn test_go_type_predicate() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("type:Server & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("type Server"));
}

#[test]
fn test_go_struct_with_port() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Server & str::8080")
        .assert()
        .success()
        .stdout(predicate::str::contains("Server"));
}

// =============================================================================
// IMPORT PREDICATE TESTS
// =============================================================================

#[test]
fn test_go_import_fmt() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:fmt & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("import"));
}

#[test]
fn test_go_import_not_found() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:nonexistent & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// CALL PREDICATE TESTS
// =============================================================================

#[test]
fn test_go_call_println() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:Println & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("fmt.Println"));
}

#[test]
fn test_go_call_newserver() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:NewServer")
        .assert()
        .success()
        .stdout(predicate::str::contains("NewServer("));
}

// =============================================================================
// COMMENT PREDICATE TESTS
// =============================================================================

#[test]
fn test_go_comment_http() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:HTTP & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("HTTP server"));
}

#[test]
fn test_go_comment_represents() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:represents & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("represents"));
}

// =============================================================================
// STRING PREDICATE TESTS
// =============================================================================

#[test]
fn test_go_str_port() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str::8080 & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains(":8080"));
}

#[test]
fn test_go_str_not_found() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:nonexistent_string_12345 & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// COMBINATION TESTS - AND operations
// =============================================================================

#[test]
fn test_go_struct_and_func() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Server & func:NewServer")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.go"));
}

#[test]
fn test_go_import_and_call() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:fmt & call:Println")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.go"));
}

#[test]
fn test_go_func_and_comment() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main & comment:HTTP")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.go"));
}

#[test]
fn test_go_struct_and_str() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Server & str::8080")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.go"));
}

#[test]
fn test_go_multiple_and() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Server & func:main & import:fmt")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.go"));
}

// =============================================================================
// COMBINATION TESTS - OR operations
// =============================================================================

#[test]
fn test_go_func_or_struct() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main | struct:Server")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.go"));
}

#[test]
fn test_go_multiple_or() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main | func:NewServer | struct:Server")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.go"));
}

// =============================================================================
// NEGATION TESTS
// =============================================================================

#[test]
fn test_go_struct_not_comment() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Server & !comment:TODO")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.go"));
}

#[test]
fn test_go_func_not_call() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:NewServer & !call:Println & ext:go")
        .assert()
        .success();
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_go_format_paths() {
    let dir = setup_test_project();
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("struct:Server & ext:go")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.go"));
    assert!(!stdout.contains("type Server"));
}

#[test]
fn test_go_format_markdown() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("struct:Server & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("```go"));
}

#[test]
fn test_go_format_hunks() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=hunks")
        .arg("func:main & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.go"));
}

// =============================================================================
// DEF PREDICATE TESTS
// =============================================================================

#[test]
fn test_go_def_struct() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Server & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("Server"));
}

#[test]
fn test_go_def_func() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:NewServer")
        .assert()
        .success()
        .stdout(predicate::str::contains("NewServer"));
}

#[test]
fn test_go_def_regex() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:.*Server & ext:go")
        .assert()
        .success();
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_go_custom_interface() {
    let dir = setup_custom_project(&[
        (
            "service.go",
            r#"package main

type Handler interface {
    Handle(req Request) Response
}

type Request struct {
    Method string
    Path   string
}

type Response struct {
    Status int
    Body   string
}
"#,
        ),
    ]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("interface:Handler")
        .assert()
        .success()
        .stdout(predicate::str::contains("type Handler interface"));
}

#[test]
fn test_go_custom_multiple_structs() {
    let dir = setup_custom_project(&[
        (
            "models.go",
            r#"package main

type User struct {
    ID   int
    Name string
}

type Post struct {
    ID      int
    Title   string
    Content string
}

type Comment struct {
    ID     int
    Text   string
    UserID int
}
"#,
        ),
    ]);

    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:.")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("User"));
    assert!(stdout.contains("Post"));
    assert!(stdout.contains("Comment"));
}

#[test]
fn test_go_custom_method_receiver() {
    let dir = setup_custom_project(&[
        (
            "methods.go",
            r#"package main

type Counter struct {
    value int
}

func (c *Counter) Increment() {
    c.value++
}

func (c Counter) Value() int {
    return c.value
}
"#,
        ),
    ]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:Increment")
        .assert()
        .success()
        .stdout(predicate::str::contains("func (c *Counter) Increment"));
}

#[test]
fn test_go_custom_multiple_imports() {
    let dir = setup_custom_project(&[
        (
            "imports.go",
            r#"package main

import (
    "fmt"
    "net/http"
    "encoding/json"
)

func handler(w http.ResponseWriter, r *http.Request) {
    json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
    fmt.Println("handled")
}
"#,
        ),
    ]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:json & call:Encode")
        .assert()
        .success()
        .stdout(predicate::str::contains("imports.go"));
}

#[test]
fn test_go_custom_constants() {
    let dir = setup_custom_project(&[
        (
            "constants.go",
            r#"package main

const (
    MaxRetries = 3
    Timeout    = 30
)

const Version = "1.0.0"
"#,
        ),
    ]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:1.0.0")
        .assert()
        .success()
        .stdout(predicate::str::contains("constants.go"));
}

#[test]
fn test_go_custom_defer() {
    let dir = setup_custom_project(&[
        (
            "defer.go",
            r#"package main

import "os"

func readFile(path string) error {
    f, err := os.Open(path)
    if err != nil {
        return err
    }
    defer f.Close()
    return nil
}
"#,
        ),
    ]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:Close & import:os")
        .assert()
        .success()
        .stdout(predicate::str::contains("defer.go"));
}

#[test]
fn test_go_custom_goroutine() {
    let dir = setup_custom_project(&[
        (
            "concurrent.go",
            r#"package main

func worker(jobs <-chan int, results chan<- int) {
    for j := range jobs {
        results <- j * 2
    }
}

func main() {
    jobs := make(chan int, 100)
    results := make(chan int, 100)

    go worker(jobs, results)
}
"#,
        ),
    ]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:worker & call:make")
        .assert()
        .success()
        .stdout(predicate::str::contains("concurrent.go"));
}

#[test]
fn test_go_custom_embedded_struct() {
    let dir = setup_custom_project(&[
        (
            "embedded.go",
            r#"package main

type Base struct {
    ID int
}

type Extended struct {
    Base
    Name string
}
"#,
        ),
    ]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Base | struct:Extended")
        .assert()
        .success()
        .stdout(predicate::str::contains("Base"))
        .stdout(predicate::str::contains("Extended"));
}

#[test]
fn test_go_custom_type_alias() {
    let dir = setup_custom_project(&[
        (
            "types.go",
            r#"package main

type UserID int64
type Handler func(int) error
"#,
        ),
    ]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("type:UserID")
        .assert()
        .success()
        .stdout(predicate::str::contains("type UserID"));
}

#[test]
fn test_go_custom_error_handling() {
    let dir = setup_custom_project(&[
        (
            "errors.go",
            r#"package main

import "errors"

var ErrNotFound = errors.New("not found")

func find(id int) error {
    if id <= 0 {
        return ErrNotFound
    }
    return nil
}
"#,
        ),
    ]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:errors & func:find")
        .assert()
        .success()
        .stdout(predicate::str::contains("errors.go"));
}

// =============================================================================
// COMPLEX QUERY TESTS
// =============================================================================

#[test]
fn test_go_complex_and_or() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("(func:main | func:NewServer) & import:fmt")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.go"));
}

#[test]
fn test_go_complex_nested() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("((struct:Server & func:NewServer) | func:main) & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.go"));
}

#[test]
fn test_go_complex_multiple_negations() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & !struct:NonExistent & !comment:TODO & ext:go")
        .assert()
        .success();
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_go_case_sensitive() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:server & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_go_empty_result() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("interface:NonExistent & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_go_ext_filter_excludes_others() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not())
        .stdout(predicate::str::contains(".py").not())
        .stdout(predicate::str::contains(".java").not());
}
