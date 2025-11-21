use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_css() {
    let dir = setup_fixture("css_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:button")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.css"));
}

#[test]
fn test_import_predicate_css() {
    let dir = setup_fixture("css_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:reset.css")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.css"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_css_def_and_import() {
    let dir = setup_fixture("css_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:button & import:reset.css")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.css"));
}

#[test]
fn test_css_or_operations() {
    let dir = setup_fixture("css_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:button | def:header")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.css"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_css_custom_flexbox() {
    let dir = setup_custom_project(&[(
        "layout.css",
        r#"
.container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
}

.row {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
}

.col {
    flex: 1;
    min-width: 200px;
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:container | def:row")
        .assert()
        .success()
        .stdout(predicate::str::contains("layout.css"));
}

#[test]
fn test_css_custom_grid() {
    let dir = setup_custom_project(&[(
        "grid.css",
        r#"
.grid-container {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    grid-gap: 20px;
}

.grid-item {
    padding: 20px;
    background-color: #f0f0f0;
}

.grid-header {
    grid-column: 1 / -1;
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:grid-container & ext:css")
        .assert()
        .success()
        .stdout(predicate::str::contains("grid.css"));
}

#[test]
fn test_css_custom_media_query() {
    let dir = setup_custom_project(&[(
        "responsive.css",
        r#"
.sidebar {
    width: 300px;
}

@media (max-width: 768px) {
    .sidebar {
        width: 100%;
    }
}

@media (min-width: 1200px) {
    .sidebar {
        width: 400px;
    }
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:sidebar & ext:css")
        .assert()
        .success()
        .stdout(predicate::str::contains("responsive.css"));
}

#[test]
fn test_css_custom_animation() {
    let dir = setup_custom_project(&[(
        "animations.css",
        r#"
@keyframes fadeIn {
    from {
        opacity: 0;
    }
    to {
        opacity: 1;
    }
}

@keyframes slideIn {
    from {
        transform: translateX(-100%);
    }
    to {
        transform: translateX(0);
    }
}

.fade-in {
    animation: fadeIn 0.5s ease-in;
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:fade-in & ext:css")
        .assert()
        .success()
        .stdout(predicate::str::contains("animations.css"));
}

#[test]
fn test_css_custom_variables() {
    let dir = setup_custom_project(&[(
        "variables.css",
        r#"
:root {
    --primary-color: #007bff;
    --secondary-color: #6c757d;
    --font-size-base: 16px;
}

.btn-primary {
    background-color: var(--primary-color);
    color: white;
}

.btn-secondary {
    background-color: var(--secondary-color);
    color: white;
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:btn-primary | def:btn-secondary")
        .assert()
        .success()
        .stdout(predicate::str::contains("variables.css"));
}

#[test]
fn test_css_custom_pseudo() {
    let dir = setup_custom_project(&[(
        "pseudo.css",
        r#"
.link {
    color: blue;
}

.link:hover {
    color: darkblue;
    text-decoration: underline;
}

.link:active {
    color: red;
}

.link::before {
    content: "→ ";
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:link & ext:css")
        .assert()
        .success()
        .stdout(predicate::str::contains("pseudo.css"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_css_format_paths() {
    let dir = setup_fixture("css_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("def:button")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.css"));
}

#[test]
fn test_css_format_markdown() {
    let dir = setup_fixture("css_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("def:button")
        .assert()
        .success()
        .stdout(predicate::str::contains("```css"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_css_not_found() {
    let dir = setup_fixture("css_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:nonexistent & ext:css")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_css_ext_filter() {
    let dir = setup_fixture("css_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:. & ext:css")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
