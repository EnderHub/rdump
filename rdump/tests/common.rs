#![allow(dead_code)] // allow dead code for this common helper module

use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use tempfile::TempDir;

pub struct CliSearchCase<'a> {
    pub fixture: &'a str,
    pub query: &'a str,
    pub expected: &'a [&'a str],
    pub absent: &'a [&'a str],
}

/// Get the path to the fixtures directory
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Copy a directory recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Setup a test project from fixtures
///
/// Available fixtures:
/// - "rust_project" - Standard Rust project structure
/// - "c_project" - C sources exercising tree-sitter predicates
/// - "python_project" - Python with various patterns
/// - "react_project" - React/TypeScript components
/// - "mixed_project" - Multiple languages
/// - "edge_cases" - Unusual file names, encodings
/// - "malformed" - Syntax errors, binary files
pub fn setup_fixture(fixture_name: &str) -> TempDir {
    let dir = tempdir().unwrap();
    let fixture_path = fixtures_dir().join(fixture_name);

    if !fixture_path.exists() {
        let manifest = fixtures_dir().join("MANIFEST.md");
        panic!(
            "Fixture '{}' not found at {:?}. See {:?} for the fixture capability map.",
            fixture_name, fixture_path, manifest
        );
    }

    copy_dir_recursive(&fixture_path, dir.path()).unwrap();
    dir
}

/// Setup the default test project (mixed_project) for backward compatibility
/// This maintains the same structure as the previous setup_test_project()
pub fn setup_test_project() -> TempDir {
    setup_fixture("mixed_project")
}

/// Setup a Rust-only test project
pub fn setup_rust_project() -> TempDir {
    setup_fixture("rust_project")
}

/// Setup a Python-only test project
pub fn setup_python_project() -> TempDir {
    setup_fixture("python_project")
}

/// Setup a React/TypeScript test project
pub fn setup_react_project() -> TempDir {
    setup_fixture("react_project")
}

/// Setup edge case files for testing unusual scenarios
pub fn setup_edge_cases() -> TempDir {
    setup_fixture("edge_cases")
}

/// Setup malformed files for error handling tests
pub fn setup_malformed_files() -> TempDir {
    setup_fixture("malformed")
}

/// Create an empty test directory
pub fn setup_empty_project() -> TempDir {
    tempdir().unwrap()
}

/// Create a test project with custom files
///
/// # Example
/// ```
/// let dir = setup_custom_project(&[
///     ("src/main.rs", "fn main() {}"),
///     ("README.md", "# Test"),
/// ]);
/// ```
pub fn setup_custom_project(files: &[(&str, &str)]) -> TempDir {
    let dir = tempdir().unwrap();

    for (path, content) in files {
        let file_path = dir.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&file_path, content).unwrap();
    }

    dir
}

pub fn assert_cli_search_case(case: &CliSearchCase<'_>) {
    let dir = setup_fixture(case.fixture);
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path()).arg("search").arg(case.query);

    let mut assertion = cmd.assert();
    assertion = assertion.success();

    for needle in case.expected {
        assertion = assertion.stdout(predicates::str::contains(*needle));
    }

    for needle in case.absent {
        assertion = assertion.stdout(predicates::str::contains(*needle).not());
    }
}

pub fn assert_cli_search_matrix(cases: &[CliSearchCase<'_>]) {
    for case in cases {
        assert_cli_search_case(case);
    }
}

pub fn assert_public_support_matrix(cases: &[rdump::support_matrix::SupportMatrixCase]) {
    for case in cases {
        assert_cli_search_case(&CliSearchCase {
            fixture: case.fixture,
            query: case.query,
            expected: case.expected,
            absent: case.absent,
        });
    }
}
