use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_struct_enum_and_type_predicates_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Server & enum:Status & type:user_id")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.c"))
        .stdout(predicate::str::contains("typedef struct Server"))
        .stdout(predicate::str::contains("typedef enum Status"))
        .stdout(predicate::str::contains("typedef int user_id"));
}

#[test]
fn test_func_and_call_predicates_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add | call:log_message")
        .assert()
        .success()
        .stdout(predicate::str::contains("int add(int a, int b)"))
        .stdout(predicate::str::contains("log_message(\"hello\")"));
}

#[test]
fn test_union_matches_struct_predicate_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Packet")
        .assert()
        .success()
        .stdout(predicate::str::contains("typedef union Packet"));
}

#[test]
fn test_import_and_macro_predicates_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:stdio & macro:MAX_BUFFER")
        .assert()
        .success()
        .stdout(predicate::str::contains("#include <stdio.h>"))
        .stdout(predicate::str::contains("#define MAX_BUFFER 1024"));
}

#[test]
fn test_comment_string_and_call_predicates_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:TODO & str:adding & call:use_util")
        .assert()
        .success()
        .stdout(predicate::str::contains("// TODO: add validation"))
        .stdout(predicate::str::contains("LOG(\"adding\")"))
        .stdout(predicate::str::contains("use_util(total)"));
}

#[test]
fn test_function_like_macro_predicate_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("macro:LOG_INT")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "#define LOG_INT(x) log_message(#x)",
        ));
}

#[test]
fn test_malformed_c_file_is_skipped_gracefully() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:. & path:bad.c")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_nonexistent_def_c() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:MissingThing & ext:c")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// STRUCT PREDICATE TESTS
// =============================================================================

#[test]
fn test_c_struct_server() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Server")
        .assert()
        .success()
        .stdout(predicate::str::contains("typedef struct Server"));
}

#[test]
fn test_c_struct_packet() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Packet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Packet"));
}

// =============================================================================
// ENUM PREDICATE TESTS
// =============================================================================

#[test]
fn test_c_enum_status() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("enum:Status")
        .assert()
        .success()
        .stdout(predicate::str::contains("typedef enum Status"));
}

// =============================================================================
// TYPE PREDICATE TESTS
// =============================================================================

#[test]
fn test_c_type_user_id() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("type:user_id")
        .assert()
        .success()
        .stdout(predicate::str::contains("typedef int user_id"));
}

#[test]
fn test_c_type_wildcard() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("type:. & ext:c")
        .assert()
        .success()
        .stdout(predicate::str::contains("typedef"));
}

// =============================================================================
// FUNC PREDICATE TESTS
// =============================================================================

#[test]
fn test_c_func_main() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("int main(void)"));
}

#[test]
fn test_c_func_add() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("int add(int a, int b)"));
}

#[test]
fn test_c_func_log_message() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:log_message")
        .assert()
        .success()
        .stdout(predicate::str::contains("static void log_message"));
}

// =============================================================================
// MACRO PREDICATE TESTS
// =============================================================================

#[test]
fn test_c_macro_max_buffer() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("macro:MAX_BUFFER")
        .assert()
        .success()
        .stdout(predicate::str::contains("#define MAX_BUFFER 1024"));
}

#[test]
fn test_c_macro_log() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("macro:LOG")
        .assert()
        .success()
        .stdout(predicate::str::contains("#define LOG"));
}

// =============================================================================
// IMPORT PREDICATE TESTS
// =============================================================================

#[test]
fn test_c_import_stdio() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:stdio")
        .assert()
        .success()
        .stdout(predicate::str::contains("#include <stdio.h>"));
}

#[test]
fn test_c_import_util() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:util.h")
        .assert()
        .success()
        .stdout(predicate::str::contains("#include \"util.h\""));
}

// =============================================================================
// CALL PREDICATE TESTS
// =============================================================================

#[test]
fn test_c_call_printf() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:printf")
        .assert()
        .success()
        .stdout(predicate::str::contains("printf"));
}

#[test]
fn test_c_call_use_util() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:use_util")
        .assert()
        .success()
        .stdout(predicate::str::contains("use_util(total)"));
}

// =============================================================================
// COMMENT PREDICATE TESTS
// =============================================================================

#[test]
fn test_c_comment_todo() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:TODO")
        .assert()
        .success()
        .stdout(predicate::str::contains("// TODO"));
}

// =============================================================================
// STRING PREDICATE TESTS
// =============================================================================

#[test]
fn test_c_str_hello() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_c_str_adding() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:adding")
        .assert()
        .success()
        .stdout(predicate::str::contains("adding"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_c_struct_and_func() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Server & func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.c"));
}

#[test]
fn test_c_macro_and_call() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("macro:LOG & call:log_message")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.c"));
}

#[test]
fn test_c_import_and_func() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:stdio & func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.c"));
}

#[test]
fn test_c_multiple_or() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main | func:add | func:log_message")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.c"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_c_format_paths() {
    let dir = setup_fixture("c_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("func:main")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.c"));
}

#[test]
fn test_c_format_markdown() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("```c"));
}

// =============================================================================
// DEF PREDICATE TESTS
// =============================================================================

#[test]
fn test_c_def_matches_multiple() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Server")
        .assert()
        .success()
        .stdout(predicate::str::contains("Server"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_c_custom_multiple_structs() {
    let dir = setup_custom_project(&[(
        "types.c",
        r#"
typedef struct Point {
    int x;
    int y;
} Point;

typedef struct Rectangle {
    Point origin;
    int width;
    int height;
} Rectangle;

typedef struct Circle {
    Point center;
    int radius;
} Circle;
"#,
    )]);

    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:.")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Point"));
    assert!(stdout.contains("Rectangle"));
    assert!(stdout.contains("Circle"));
}

#[test]
fn test_c_custom_simple_typedef() {
    let dir = setup_custom_project(&[(
        "types.c",
        r#"
typedef int user_id;
typedef long timestamp;

int get_user(user_id id) {
    return id;
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("type:user_id & func:get_user")
        .assert()
        .success()
        .stdout(predicate::str::contains("types.c"));
}

#[test]
fn test_c_custom_static_functions() {
    let dir = setup_custom_project(&[(
        "internal.c",
        r#"
static int helper(int x) {
    return x * 2;
}

int public_func(int x) {
    return helper(x) + 1;
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:helper & call:helper")
        .assert()
        .success()
        .stdout(predicate::str::contains("internal.c"));
}

#[test]
fn test_c_custom_include_guard_macro() {
    let dir = setup_custom_project(&[(
        "main.c",
        r#"
#define DEBUG_MODE 1
#define VERSION "1.0"

int main(void) {
    return 0;
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("macro:DEBUG_MODE")
        .assert()
        .success()
        .stdout(predicate::str::contains("#define DEBUG_MODE"));
}

#[test]
fn test_c_custom_variadic_function() {
    let dir = setup_custom_project(&[(
        "logging.c",
        r#"
#include <stdarg.h>
#include <stdio.h>

void log_format(const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);
    vprintf(fmt, args);
    va_end(args);
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:log_format & import:stdarg")
        .assert()
        .success()
        .stdout(predicate::str::contains("logging.c"));
}

// =============================================================================
// NEGATION TESTS
// =============================================================================

#[test]
fn test_c_func_not_struct() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add & !struct:Missing")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.c"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_c_case_sensitive() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:server")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_c_ext_filter() {
    let dir = setup_fixture("c_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:c")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not())
        .stdout(predicate::str::contains(".py").not());
}
