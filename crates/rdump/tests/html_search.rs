use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_html() {
    let dir = setup_fixture("html_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:div")
        .assert()
        .success()
        .stdout(predicate::str::contains("index.html"));
}

#[test]
fn test_import_predicate_html() {
    let dir = setup_fixture("html_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:script")
        .assert()
        .success()
        .stdout(predicate::str::contains("index.html"));
}

#[test]
fn test_str_predicate_html() {
    let dir = setup_fixture("html_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("index.html"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_html_def_and_str() {
    let dir = setup_fixture("html_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:div & str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("index.html"));
}

#[test]
fn test_html_or_operations() {
    let dir = setup_fixture("html_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:div | def:span")
        .assert()
        .success()
        .stdout(predicate::str::contains("index.html"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_html_custom_form() {
    let dir = setup_custom_project(&[(
        "form.html",
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Contact Form</title>
</head>
<body>
    <form id="contact-form" action="/submit" method="POST">
        <label for="name">Name:</label>
        <input type="text" id="name" name="name" required>

        <label for="email">Email:</label>
        <input type="email" id="email" name="email" required>

        <button type="submit">Submit</button>
    </form>
</body>
</html>
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:form & ext:html")
        .assert()
        .success()
        .stdout(predicate::str::contains("form.html"));
}

#[test]
fn test_html_custom_table() {
    let dir = setup_custom_project(&[(
        "table.html",
        r#"<!DOCTYPE html>
<html>
<body>
    <table>
        <thead>
            <tr>
                <th>Name</th>
                <th>Age</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>John</td>
                <td>30</td>
            </tr>
            <tr>
                <td>Jane</td>
                <td>25</td>
            </tr>
        </tbody>
    </table>
</body>
</html>
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:table & ext:html")
        .assert()
        .success()
        .stdout(predicate::str::contains("table.html"));
}

#[test]
fn test_html_custom_semantic() {
    let dir = setup_custom_project(&[(
        "semantic.html",
        r#"<!DOCTYPE html>
<html>
<body>
    <header>
        <nav>
            <ul>
                <li><a href="/">Home</a></li>
                <li><a href="/about">About</a></li>
            </ul>
        </nav>
    </header>
    <main>
        <article>
            <h1>Article Title</h1>
            <p>Article content here.</p>
        </article>
    </main>
    <footer>
        <p>Copyright 2024</p>
    </footer>
</body>
</html>
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:header | def:footer")
        .assert()
        .success()
        .stdout(predicate::str::contains("semantic.html"));
}

#[test]
fn test_html_custom_meta() {
    let dir = setup_custom_project(&[(
        "meta.html",
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="A sample page">
    <title>Page Title</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <h1>Hello World</h1>
</body>
</html>
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:meta & ext:html")
        .assert()
        .success()
        .stdout(predicate::str::contains("meta.html"));
}

#[test]
fn test_html_custom_script() {
    let dir = setup_custom_project(&[(
        "script.html",
        r#"<!DOCTYPE html>
<html>
<head>
    <script src="app.js" defer></script>
</head>
<body>
    <button id="btn">Click me</button>
    <script>
        document.getElementById('btn').onclick = function() {
            alert('Clicked!');
        };
    </script>
</body>
</html>
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:script & ext:html")
        .assert()
        .success()
        .stdout(predicate::str::contains("script.html"));
}

#[test]
fn test_html_custom_list() {
    let dir = setup_custom_project(&[(
        "list.html",
        r#"<!DOCTYPE html>
<html>
<body>
    <h2>Unordered List</h2>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
    </ul>

    <h2>Ordered List</h2>
    <ol>
        <li>First</li>
        <li>Second</li>
        <li>Third</li>
    </ol>
</body>
</html>
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:ul | def:ol")
        .assert()
        .success()
        .stdout(predicate::str::contains("list.html"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_html_format_paths() {
    let dir = setup_fixture("html_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("def:div")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("index.html"));
}

#[test]
fn test_html_format_markdown() {
    let dir = setup_fixture("html_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("def:div")
        .assert()
        .success()
        .stdout(predicate::str::contains("```html"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_html_not_found() {
    let dir = setup_fixture("html_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:nonexistent & ext:html")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_html_ext_filter() {
    let dir = setup_fixture("html_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:. & ext:html")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
