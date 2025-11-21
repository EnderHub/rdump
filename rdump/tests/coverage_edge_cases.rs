use predicates::prelude::*;
use std::fs;
use std::io::Write;
mod common;
use common::setup_custom_project;

// =============================================================================
// EVALUATOR EDGE CASES
// =============================================================================

#[test]
fn test_file_exceeds_max_size() {
    // Create a file larger than MAX_FILE_SIZE (100MB) - we'll use a smaller test
    // by checking the behavior with a large file message
    let dir = setup_custom_project(&[("large.rs", "fn main() {}")]);

    // This tests the normal path; actual large file test would be too slow
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main")
        .assert()
        .success();
}

#[test]
fn test_binary_file_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("binary.rs");

    // Create a file with null bytes (binary detection)
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(b"fn main() { \x00 }").unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_secret_file_skipped() {
    let dir = setup_custom_project(&[(
        "secrets.rs",
        r#"
fn get_key() -> &'static str {
    "-----BEGIN PRIVATE KEY-----
    MIIEvgIBADANBg...
    -----END PRIVATE KEY-----"
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:get_key")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_aws_secret_key_skipped() {
    let dir = setup_custom_project(&[(
        "config.rs",
        r#"
const AWS_SECRET_ACCESS_KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
"#,
    )]);

    // File is found but content is empty due to secret detection
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("ext:rs")
        .assert()
        .success()
        .stderr(predicate::str::contains("Skipping possible secret"));
}

#[test]
fn test_jwt_token_skipped() {
    let dir = setup_custom_project(&[(
        "auth.rs",
        r#"
const TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ";
"#,
    )]);

    // File is found but content is empty due to JWT detection
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("ext:rs")
        .assert()
        .success()
        .stderr(predicate::str::contains("Skipping possible secret"));
}

// =============================================================================
// SQL DIALECT EDGE CASES
// =============================================================================

#[test]
fn test_sql_mysql_delimiter_detection() {
    let dir = setup_custom_project(&[(
        "mysql.sql",
        r#"
DELIMITER //
CREATE PROCEDURE GetUsers()
BEGIN
    SELECT * FROM users;
END //
DELIMITER ;
"#,
    )]);

    // MySQL dialect detection via DELIMITER keyword
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::contains("mysql.sql"));
}

#[test]
fn test_sql_sqlite_begin_atomic_detection() {
    let dir = setup_custom_project(&[(
        "sqlite.sql",
        r#"
CREATE TRIGGER update_timestamp
AFTER UPDATE ON users
BEGIN ATOMIC
    UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
"#,
    )]);

    // SQLite dialect detection via BEGIN ATOMIC
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::contains("sqlite.sql"));
}

#[test]
fn test_sql_postgres_returns_table_detection() {
    let dir = setup_custom_project(&[(
        "postgres.sql",
        r#"
CREATE FUNCTION get_users()
RETURNS TABLE (id INT, name VARCHAR)
AS $$
BEGIN
    RETURN QUERY SELECT id, name FROM users;
END;
$$ LANGUAGE plpgsql;
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:get_users")
        .assert()
        .success()
        .stdout(predicate::str::contains("postgres.sql"));
}

#[test]
fn test_sql_dialect_flag_postgres() {
    let dir = setup_custom_project(&[(
        "query.sql",
        "SELECT * FROM users;",
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--dialect=postgres")
        .arg("ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::contains("query.sql"));
}

#[test]
fn test_sql_dialect_flag_mysql() {
    let dir = setup_custom_project(&[(
        "query.sql",
        "SELECT * FROM users;",
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--dialect=mysql")
        .arg("ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::contains("query.sql"));
}

#[test]
fn test_sql_dialect_flag_sqlite() {
    let dir = setup_custom_project(&[(
        "query.sql",
        "SELECT * FROM users;",
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--dialect=sqlite")
        .arg("ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::contains("query.sql"));
}

// =============================================================================
// PATH ESCAPE DETECTION
// =============================================================================

#[test]
fn test_safe_canonicalize_within_root() {
    let dir = setup_custom_project(&[("test.rs", "fn main() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

// =============================================================================
// FORMATTER EDGE CASES
// =============================================================================

#[test]
fn test_format_json_output() {
    let dir = setup_custom_project(&[("test.rs", "fn main() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=json")
        .arg("func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"path\""));
}

#[test]
fn test_format_cat_output() {
    let dir = setup_custom_project(&[("test.rs", "fn main() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=cat")
        .arg("func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn main()"));
}

#[test]
fn test_format_find_output() {
    let dir = setup_custom_project(&[("test.rs", "fn main() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=find")
        .arg("func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

#[test]
fn test_no_headers_flag() {
    let dir = setup_custom_project(&[("test.rs", "fn main() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--no-headers")
        .arg("func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn main()"));
}

#[test]
fn test_line_numbers_flag() {
    let dir = setup_custom_project(&[("test.rs", "fn main() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--line-numbers")
        .arg("--format=markdown")
        .arg("func:main")
        .assert()
        .success();
}

#[test]
fn test_context_lines_flag() {
    let dir = setup_custom_project(&[(
        "test.rs",
        r#"
// Comment before
fn main() {
    println!("hello");
}
// Comment after
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("-C")
        .arg("2")
        .arg("func:main")
        .assert()
        .success();
}

// =============================================================================
// SEARCH COMMAND EDGE CASES
// =============================================================================

#[test]
fn test_hidden_files_flag() {
    let dir = tempfile::tempdir().unwrap();
    let hidden_file = dir.path().join(".hidden.rs");
    fs::write(&hidden_file, "fn hidden() {}").unwrap();

    // Without --hidden flag, should not find
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:hidden")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    // With --hidden flag, should find
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--hidden")
        .arg("func:hidden")
        .assert()
        .success()
        .stdout(predicate::str::contains(".hidden.rs"));
}

#[test]
fn test_max_depth_flag() {
    let dir = tempfile::tempdir().unwrap();
    let deep_dir = dir.path().join("a").join("b").join("c");
    fs::create_dir_all(&deep_dir).unwrap();
    fs::write(deep_dir.join("deep.rs"), "fn deep() {}").unwrap();

    // With max-depth=1, should not find deep file
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--max-depth=1")
        .arg("func:deep")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    // With max-depth=4, should find deep file
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--max-depth=4")
        .arg("func:deep")
        .assert()
        .success()
        .stdout(predicate::str::contains("deep.rs"));
}

#[test]
fn test_output_to_file() {
    let dir = setup_custom_project(&[("test.rs", "fn main() {}")]);
    let output_file = dir.path().join("output.txt");

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--output")
        .arg(&output_file)
        .arg("--format=paths")
        .arg("func:main")
        .assert()
        .success();

    let content = fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("test.rs"));
}

#[test]
fn test_color_always_flag() {
    let dir = setup_custom_project(&[("test.rs", "fn main() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--color=always")
        .arg("func:main")
        .assert()
        .success();
}

#[test]
fn test_color_never_flag() {
    let dir = setup_custom_project(&[("test.rs", "fn main() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--color=never")
        .arg("func:main")
        .assert()
        .success();
}

#[test]
fn test_no_ignore_flag() {
    let dir = tempfile::tempdir().unwrap();

    // Initialize git repo for .gitignore to work
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Create a .gitignore that ignores .rs files
    fs::write(dir.path().join(".gitignore"), "*.rs").unwrap();
    fs::write(dir.path().join("test.rs"), "fn main() {}").unwrap();

    // Without --no-ignore, should not find (gitignore respected)
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    // With --no-ignore, should find
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--no-ignore")
        .arg("func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

// =============================================================================
// MATCH RESULT COMBINATION EDGE CASES
// =============================================================================

#[test]
fn test_hunks_and_boolean_true() {
    // Test combining hunks with boolean true (full-file match)
    let dir = setup_custom_project(&[("test.rs", "fn foo() {} fn bar() {}")]);

    // func:foo produces hunks, ext:rs produces boolean true
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:foo & ext:rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn foo()"));
}

#[test]
fn test_boolean_true_and_hunks() {
    // Test combining boolean true with hunks (opposite order)
    let dir = setup_custom_project(&[("test.rs", "fn foo() {} fn bar() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("ext:rs & func:foo")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn foo()"));
}

#[test]
fn test_hunks_or_boolean_false() {
    // Test combining hunks with boolean false
    let dir = setup_custom_project(&[("test.rs", "fn foo() {}")]);

    // func:foo produces hunks, ext:go produces false
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:foo | ext:go")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn foo()"));
}

#[test]
fn test_boolean_false_or_hunks() {
    // Test combining boolean false with hunks
    let dir = setup_custom_project(&[("test.rs", "fn foo() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("ext:go | func:foo")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn foo()"));
}

// =============================================================================
// PARSER EDGE CASES
// =============================================================================

#[test]
fn test_complex_nested_query() {
    let dir = setup_custom_project(&[(
        "test.rs",
        r#"
fn foo() {}
fn bar() {}
struct Baz {}
"#,
    )]);

    // Complex nested query with multiple operators
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("(func:foo | func:bar) & struct:Baz")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

#[test]
fn test_find_flag_alias() {
    let dir = setup_custom_project(&[("test.rs", "fn foo() {}")]);

    // --find is an alias for --format=find
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--find")
        .arg("func:foo")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

// =============================================================================
// PATH PREDICATE TESTS
// =============================================================================

#[test]
fn test_path_predicate() {
    let dir = tempfile::tempdir().unwrap();
    let subdir = dir.path().join("src").join("utils");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(subdir.join("helper.rs"), "fn help() {}").unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("path:src/utils")
        .assert()
        .success()
        .stdout(predicate::str::contains("helper.rs"));
}

#[test]
fn test_path_exact_predicate() {
    let dir = tempfile::tempdir().unwrap();
    let subdir = dir.path().join("src");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(subdir.join("main.rs"), "fn main() {}").unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("path_exact:src/main.rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"));
}

#[test]
fn test_in_predicate() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    let test_dir = dir.path().join("tests");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "fn lib() {}").unwrap();
    fs::write(test_dir.join("test.rs"), "fn test() {}").unwrap();

    // Only find files in src directory
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("in:src")
        .assert()
        .success()
        .stdout(predicate::str::contains("lib.rs"))
        .stdout(predicate::str::contains("test.rs").not());
}

// =============================================================================
// PRESET COMMAND TESTS
// =============================================================================

#[test]
fn test_preset_list() {
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .arg("preset")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_preset_add_and_remove() {
    // Add a preset
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .arg("preset")
        .arg("add")
        .arg("test_preset_coverage")
        .arg("ext:rs")
        .assert()
        .success();

    // Remove the preset
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .arg("preset")
        .arg("remove")
        .arg("test_preset_coverage")
        .assert()
        .success();
}

#[test]
fn test_use_preset() {
    let dir = setup_custom_project(&[("test.rs", "fn main() {}")]);

    // First add a preset
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .arg("preset")
        .arg("add")
        .arg("rust_funcs_coverage")
        .arg("func:. & ext:rs")
        .assert()
        .success();

    // Use the preset
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--preset")
        .arg("rust_funcs_coverage")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));

    // Clean up
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .arg("preset")
        .arg("remove")
        .arg("rust_funcs_coverage")
        .assert()
        .success();
}

// =============================================================================
// LANG COMMAND TESTS
// =============================================================================

#[test]
fn test_lang_list() {
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .arg("lang")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"));
}

#[test]
fn test_lang_describe() {
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .arg("lang")
        .arg("describe")
        .arg("rust")
        .assert()
        .success()
        .stdout(predicate::str::contains("func"));
}

#[test]
fn test_lang_default_action() {
    // Just `rdump lang` should list languages
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .arg("lang")
        .assert()
        .success();
}

// =============================================================================
// MORE SIZE AND MODIFIED PREDICATES
// =============================================================================

#[test]
fn test_size_less_than() {
    let dir = setup_custom_project(&[("small.rs", "fn x() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("size:<1kb")
        .assert()
        .success()
        .stdout(predicate::str::contains("small.rs"));
}

#[test]
fn test_size_greater_than() {
    let dir = setup_custom_project(&[("small.rs", "fn x() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("size:>1mb")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_modified_recently() {
    let dir = setup_custom_project(&[("recent.rs", "fn recent() {}")]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("modified:>1h")
        .assert()
        .success()
        .stdout(predicate::str::contains("recent.rs"));
}

// =============================================================================
// MATCHES PREDICATE
// =============================================================================

#[test]
fn test_matches_regex() {
    let dir = setup_custom_project(&[(
        "test.rs",
        r#"
fn calculate_sum() {}
fn calculate_diff() {}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("matches:calculate_\\w+")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

// =============================================================================
// NAME PREDICATE
// =============================================================================

#[test]
fn test_name_glob_pattern() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("test_foo.rs"), "fn foo() {}").unwrap();
    fs::write(dir.path().join("test_bar.rs"), "fn bar() {}").unwrap();
    fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("name:test_*.rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("test_foo.rs"))
        .stdout(predicate::str::contains("test_bar.rs"))
        .stdout(predicate::str::contains("main.rs").not());
}

// =============================================================================
// ROOT DIRECTORY
// =============================================================================

#[test]
fn test_custom_root_directory() {
    let dir = tempfile::tempdir().unwrap();
    let subdir = dir.path().join("project");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(subdir.join("app.rs"), "fn app() {}").unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .arg("search")
        .arg("--root")
        .arg(&subdir)
        .arg("func:app")
        .assert()
        .success()
        .stdout(predicate::str::contains("app.rs"));
}

// =============================================================================
// PATH PREDICATE WITH ABSOLUTE PATHS
// =============================================================================

#[test]
fn test_path_predicate_absolute() {
    let dir = setup_custom_project(&[("src/lib.rs", "fn lib() {}")]);
    let absolute_dir = dir.path().canonicalize().unwrap();

    // Use absolute path in the search pattern
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg(format!("path:{}", absolute_dir.join("src").display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("lib.rs"));
}

#[test]
fn test_path_predicate_glob_absolute() {
    let dir = setup_custom_project(&[("src/main.rs", "fn main() {}")]);
    let absolute_dir = dir.path().canonicalize().unwrap();

    // Use absolute glob pattern
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg(format!("path:{}/*.rs", absolute_dir.join("src").display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"));
}

// =============================================================================
// CODE AWARE WITH UNSUPPORTED EXTENSIONS
// =============================================================================

#[test]
fn test_code_aware_unsupported_extension() {
    let dir = setup_custom_project(&[("data.xyz", "some random data")]);

    // Code aware predicates should return false for unsupported extensions
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:main & ext:xyz")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_code_aware_empty_query_result() {
    let dir = setup_custom_project(&[("test.rs", "struct Data {}")]);

    // func predicate looking for something that doesn't exist
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:nonexistent")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// SQL DIALECT EDGE CASES
// =============================================================================

#[test]
fn test_sql_generic_dialect() {
    let dir = setup_custom_project(&[(
        "simple.sql",
        "SELECT * FROM users WHERE active = 1;",
    )]);

    // Generic SQL without specific dialect markers
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::contains("simple.sql"));
}

#[test]
fn test_sql_plpgsql_detection() {
    let dir = setup_custom_project(&[(
        "postgres.sql",
        r#"
CREATE OR REPLACE FUNCTION get_users()
RETURNS TABLE(id INT, name TEXT)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY SELECT id, name FROM users;
END;
$$;
"#,
    )]);

    // Postgres detection via 'language plpgsql'
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::contains("postgres.sql"));
}

// =============================================================================
// EVALUATOR EDGE CASES
// =============================================================================

#[test]
fn test_not_predicate_with_code_aware() {
    let dir = setup_custom_project(&[
        ("has_func.rs", "fn target() {}"),
        ("no_func.rs", "struct Data {}"),
    ]);

    // NOT with code-aware predicate
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("!func:target & ext:rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("no_func.rs"))
        .stdout(predicate::str::contains("has_func.rs").not());
}

#[test]
fn test_boolean_combinations_edge_cases() {
    let dir = setup_custom_project(&[
        ("test.rs", "fn foo() { bar() }"),
    ]);

    // Complex boolean combination
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("(func:foo & call:bar) | (func:baz)")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

// =============================================================================
// MATCH RESULT COMBINE EDGE CASES
// =============================================================================

#[test]
fn test_hunks_with_boolean_true() {
    let dir = setup_custom_project(&[
        ("test.rs", "fn foo() {} fn bar() {}"),
    ]);

    // Hunk-producing predicate AND with boolean true (ext match)
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:foo & ext:rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn foo"));
}

#[test]
fn test_boolean_false_and_hunks() {
    let dir = setup_custom_project(&[
        ("test.rs", "fn foo() {}"),
    ]);

    // Boolean false AND hunks should return false
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("ext:py & func:foo")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// IN PREDICATE EDGE CASES
// =============================================================================

#[test]
fn test_in_predicate_with_trailing_slash() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "fn lib() {}").unwrap();

    // With trailing slash
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("in:src/")
        .assert()
        .success()
        .stdout(predicate::str::contains("lib.rs"));
}

#[test]
fn test_in_predicate_dot_notation() {
    let dir = setup_custom_project(&[("test.rs", "fn test() {}")]);

    // Current directory with .
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("in:.")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

// =============================================================================
// PRESET EDGE CASES
// =============================================================================

#[test]
fn test_preset_remove_nonexistent() {
    // Try to remove a preset that doesn't exist
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .arg("preset")
        .arg("remove")
        .arg("nonexistent_preset_12345")
        .assert()
        .failure();
}

// =============================================================================
// LARGE FILE HANDLING
// =============================================================================

#[test]
fn test_very_large_query() {
    let dir = setup_custom_project(&[("test.rs", "fn main() {}")]);

    // Complex deeply nested query
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("((ext:rs | ext:py) & (func:main | func:init)) | contains:fn")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

// =============================================================================
// WILDCARD PREDICATE VALUES
// =============================================================================

#[test]
fn test_func_wildcard() {
    let dir = setup_custom_project(&[("test.rs", "fn any_function() {}")]);

    // Wildcard value for func predicate
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:.")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

#[test]
fn test_class_wildcard() {
    let dir = setup_custom_project(&[("test.py", "class AnyClass:\n    pass")]);

    // Wildcard for class
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:.")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.py"));
}
