use anyhow::Result;
use once_cell::sync::Lazy;
use rdump::search_iter;
use rdump::{search, SearchOptions};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use tempfile::{tempdir, TempDir};

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
}

struct EnvVarGuard {
    key: String,
    prev: Option<String>,
}

impl EnvVarGuard {
    fn new(key: &str, value: &str) -> Self {
        let prev = env::var(key).ok();
        env::set_var(key, value);
        Self {
            key: key.to_string(),
            prev,
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(prev) = &self.prev {
            env::set_var(&self.key, prev);
        } else {
            env::remove_var(&self.key);
        }
    }
}

// =============================================================================
// Test Fixtures
// =============================================================================

fn create_rust_fixtures() -> Result<TempDir> {
    let dir = tempdir()?;

    fs::write(
        dir.path().join("main.rs"),
        r#"fn main() {
    println!("Hello, world!");
}
"#,
    )?;

    fs::write(
        dir.path().join("lib.rs"),
        r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
"#,
    )?;

    let subdir = dir.path().join("src");
    fs::create_dir_all(&subdir)?;
    fs::write(
        subdir.join("utils.rs"),
        r#"pub fn helper() -> String {
    "helper".to_string()
}
"#,
    )?;

    Ok(dir)
}

fn create_multi_lang_fixtures() -> Result<TempDir> {
    let dir = tempdir()?;

    fs::write(
        dir.path().join("script.py"),
        "def main():\n    print('hello')\n",
    )?;
    fs::write(
        dir.path().join("app.js"),
        "function main() {\n  console.log('hello');\n}\n",
    )?;
    fs::write(dir.path().join("lib.rs"), "fn only_rust() {}\n")?;

    Ok(dir)
}

fn write_nested_file(root: &PathBuf, relative: &str, content: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn create_binary_file(dir: &PathBuf, name: &str) {
    let mut data = vec![0u8; 16];
    data[0] = 0;
    fs::write(dir.join(name), data).unwrap();
}

// =============================================================================
// Basic Search Tests
// =============================================================================

#[test]
fn test_basic_extension_search() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 3, "expected three rust files");
    assert!(results.iter().all(|r| r.path.extension().unwrap() == "rs"));

    Ok(())
}

#[test]
fn test_search_with_no_results() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let results = search(
        "ext:py",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert!(results.is_empty());
    Ok(())
}

#[test]
fn test_search_nonexistent_root() {
    let result = search(
        "ext:rs",
        SearchOptions {
            root: PathBuf::from("/nonexistent/path/that/does/not/exist"),
            ..Default::default()
        },
    );

    assert!(result.is_err());
}

#[test]
fn test_invalid_query_syntax() {
    let dir = tempdir().unwrap();

    let result = search(
        "invalid((syntax",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    );

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("parse") || err.contains("syntax") || err.contains("unexpected"));
}

// =============================================================================
// Semantic Search Tests
// =============================================================================

#[test]
fn test_function_predicate() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let results = search(
        "func:main",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path.file_name().unwrap(), "main.rs");
    assert!(!results[0].is_whole_file_match());
    assert!(!results[0].matches.is_empty());

    let m = &results[0].matches[0];
    assert!(m.byte_range.end > m.byte_range.start);
    Ok(())
}

#[test]
fn test_function_predicate_multiple_matches() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let results = search(
        "func:add | func:subtract",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path.file_name().unwrap(), "lib.rs");
    assert_eq!(results[0].matches.len(), 2);
    Ok(())
}

// =============================================================================
// Compound Query Tests
// =============================================================================

#[test]
fn test_and_query() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let results = search(
        "ext:rs & func:main",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path.file_name().unwrap(), "main.rs");
    Ok(())
}

#[test]
fn test_or_query() -> Result<()> {
    let dir = create_multi_lang_fixtures()?;

    let results = search(
        "ext:py | ext:js",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 2);
    let extensions: Vec<_> = results
        .iter()
        .map(|r| r.path.extension().unwrap().to_str().unwrap().to_string())
        .collect();
    assert!(extensions.contains(&"py".to_string()));
    assert!(extensions.contains(&"js".to_string()));
    Ok(())
}

#[test]
fn test_not_query() -> Result<()> {
    let dir = create_multi_lang_fixtures()?;

    let results = search(
        "!ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 2);
    for result in &results {
        assert_ne!(result.path.extension().unwrap(), "rs");
    }
    Ok(())
}

#[test]
fn test_complex_compound_query() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let results = search(
        "ext:rs & (contains:\"Hello\" | func:add)",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    let names: Vec<_> = results
        .iter()
        .map(|r| r.path.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(names.contains(&"main.rs"));
    assert!(names.contains(&"lib.rs"));
    Ok(())
}

// =============================================================================
// Whole-File Match Tests
// =============================================================================

#[test]
fn test_whole_file_match() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    for result in &results {
        assert!(result.is_whole_file_match());
        assert!(result.matches.is_empty());
        assert!(!result.content.is_empty());
    }

    Ok(())
}

#[test]
fn test_whole_file_match_content_available() -> Result<()> {
    let dir = tempdir()?;
    let content = "fn specific_content() { 42 }";
    fs::write(dir.path().join("test.rs"), content)?;

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    assert!(results[0].is_whole_file_match());
    assert_eq!(results[0].content, content);
    Ok(())
}

// =============================================================================
// Match Struct Tests
// =============================================================================

#[test]
fn test_match_line_numbers_are_one_indexed() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("test.rs"),
        "// line 1\n// line 2\nfn target() {}\n// line 4\n",
    )?;

    let results = search(
        "func:target",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    let m = &results[0].matches[0];
    assert_eq!(m.start_line, 3);
    assert_eq!(m.end_line, 3);
    Ok(())
}

#[test]
fn test_match_multiline() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("test.rs"),
        r#"fn multiline(
    arg1: i32,
    arg2: i32,
) -> i32 {
    arg1 + arg2
}"#,
    )?;

    let results = search(
        "func:multiline",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    let m = &results[0].matches[0];
    assert_eq!(m.start_line, 1);
    assert!(m.end_line >= m.start_line);
    let line_count = m.line_count();
    assert!(line_count >= 1);
    assert!(results[0].content.contains("arg2"));
    Ok(())
}

#[test]
fn test_match_byte_range() -> Result<()> {
    let dir = tempdir()?;
    let content = "fn foo() {}";
    fs::write(dir.path().join("test.rs"), content)?;

    let results = search(
        "func:foo",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    let m = &results[0].matches[0];
    assert_eq!(&content[m.byte_range.clone()], m.text);
    Ok(())
}

// =============================================================================
// SearchResult Helper Method Tests
// =============================================================================

#[test]
fn test_matched_lines_helper() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("test.rs"),
        "fn a() {}\nfn b() {}\nfn c() {}\n",
    )?;

    let results = search(
        "func:a | func:b | func:c",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    let lines = results[0].matched_lines();
    assert_eq!(lines, vec![1, 2, 3]);
    Ok(())
}

#[test]
fn test_match_count_helper() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("test.rs"),
        "fn a() {}\nfn b() {}\nfn c() {}\n",
    )?;

    let results = search(
        "func:a | func:b",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results[0].match_count(), 2);
    Ok(())
}

// =============================================================================
// Empty Query Tests
// =============================================================================

#[test]
fn test_empty_query_with_preset() -> Result<()> {
    let _lock = lock_env();
    let dir = tempdir()?;
    let config_root = dir.path().join("config");
    let preset_dir = config_root.join("rdump");
    fs::create_dir_all(&preset_dir)?;
    fs::write(
        preset_dir.join("config.toml"),
        r#"
            [presets]
            rust = "ext:rs"
        "#,
    )?;

    let file = dir.path().join("code").join("main.rs");
    fs::create_dir_all(file.parent().unwrap())?;
    fs::write(&file, "fn main() {}")?;

    let _guard = EnvVarGuard::new("RDUMP_TEST_CONFIG_DIR", config_root.to_str().unwrap());

    let results = search(
        "",
        SearchOptions {
            root: file.parent().unwrap().to_path_buf(),
            presets: vec!["rust".to_string()],
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path.extension().unwrap(), "rs");
    Ok(())
}

// =============================================================================
// Streaming Iterator Tests
// =============================================================================

#[test]
fn test_search_iter_basic() -> Result<()> {
    let dir = create_rust_fixtures()?;

    let iter = search_iter(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    let results: Vec<_> = iter.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(results.len(), 3);
    Ok(())
}

#[test]
fn test_search_iter_early_termination() -> Result<()> {
    let dir = tempdir()?;
    for i in 0..50 {
        fs::write(dir.path().join(format!("file{i}.rs")), "fn main() {}")?;
    }

    let iter = search_iter(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    let first_five: Vec<_> = iter.take(5).collect::<Result<Vec<_>, _>>()?;
    assert_eq!(first_five.len(), 5);
    Ok(())
}

#[test]
fn test_search_iter_size_hint() -> Result<()> {
    let dir = create_rust_fixtures()?;
    let iter = search_iter(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    let (lower, upper) = iter.size_hint();
    assert_eq!(lower, 3);
    assert_eq!(upper, Some(3));
    Ok(())
}

#[test]
fn test_search_iter_remaining() -> Result<()> {
    let dir = create_rust_fixtures()?;
    let mut iter = search_iter(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(iter.remaining(), 3);
    iter.next();
    assert_eq!(iter.remaining(), 2);
    iter.next();
    assert_eq!(iter.remaining(), 1);
    iter.next();
    assert_eq!(iter.remaining(), 0);
    Ok(())
}

#[test]
fn test_search_iter_skip_errors() -> Result<()> {
    let dir = tempdir()?;
    let ok = dir.path().join("good.rs");
    let stale = dir.path().join("stale.rs");
    fs::write(&ok, "fn main() {}")?;
    fs::write(&stale, "fn stale() {}")?;

    let iter = search_iter(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    fs::remove_file(stale)?;
    let results: Vec<_> = iter.filter_map(Result::ok).collect();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, ok);
    Ok(())
}

// =============================================================================
// SearchOptions Field Tests
// =============================================================================

#[test]
fn test_max_depth_option() -> Result<()> {
    let dir = tempdir()?;
    write_nested_file(&dir.path().to_path_buf(), "level1.rs", "fn l1() {}");
    write_nested_file(&dir.path().to_path_buf(), "sub/level2.rs", "fn l2() {}");
    write_nested_file(&dir.path().to_path_buf(), "sub/sub/level3.rs", "fn l3() {}");

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            max_depth: Some(1),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path.file_name().unwrap(), "level1.rs");
    Ok(())
}

#[test]
fn test_hidden_files_option() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("visible.rs"), "fn main() {}")?;
    fs::write(dir.path().join(".hidden.rs"), "fn hidden() {}")?;

    let visible_only = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;
    assert_eq!(visible_only.len(), 1);

    let include_hidden = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            hidden: true,
            ..Default::default()
        },
    )?;
    assert_eq!(include_hidden.len(), 2);
    Ok(())
}

#[test]
fn test_no_ignore_option() -> Result<()> {
    let dir = tempdir()?;
    Command::new("git")
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    fs::write(dir.path().join(".gitignore"), "*.rs\n")?;
    fs::write(dir.path().join("ignored.rs"), "fn ignored() {}")?;

    let default_res = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;
    assert!(default_res.is_empty());

    let no_ignore_res = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            no_ignore: true,
            ..Default::default()
        },
    )?;
    assert_eq!(no_ignore_res.len(), 1);
    Ok(())
}

#[test]
fn test_presets_option() -> Result<()> {
    let _lock = lock_env();
    let dir = tempdir()?;
    let config_root = dir.path().join("config");
    let preset_dir = config_root.join("rdump");
    fs::create_dir_all(&preset_dir)?;
    fs::write(
        preset_dir.join("config.toml"),
        r#"
            [presets]
            rust = "ext:rs"
        "#,
    )?;

    let code_dir = dir.path().join("code");
    fs::create_dir_all(&code_dir)?;
    fs::write(code_dir.join("main.rs"), "fn main() {}")?;

    let _guard = EnvVarGuard::new("RDUMP_TEST_CONFIG_DIR", config_root.to_str().unwrap());
    let results = search(
        "",
        SearchOptions {
            root: code_dir.clone(),
            presets: vec!["rust".to_string()],
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    Ok(())
}

#[test]
fn test_multiple_presets() -> Result<()> {
    let _lock = lock_env();
    let dir = tempdir()?;
    let config_root = dir.path().join("config");
    let preset_dir = config_root.join("rdump");
    fs::create_dir_all(&preset_dir)?;
    fs::write(
        preset_dir.join("config.toml"),
        r#"
            [presets]
            rust = "ext:rs"
            has_main = "contains:main"
        "#,
    )?;

    let code_dir = dir.path().join("code");
    fs::create_dir_all(&code_dir)?;
    fs::write(code_dir.join("main.rs"), "fn main() {}")?;

    let _guard = EnvVarGuard::new("RDUMP_TEST_CONFIG_DIR", config_root.to_str().unwrap());
    let results = search(
        "",
        SearchOptions {
            root: code_dir.clone(),
            presets: vec!["rust".to_string(), "has_main".to_string()],
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path.file_name().unwrap(), "main.rs");
    Ok(())
}

#[test]
fn test_custom_root() -> Result<()> {
    let dir = tempdir()?;
    let nested = dir.path().join("nested");
    fs::create_dir_all(&nested)?;
    fs::write(nested.join("inner.rs"), "fn inner() {}")?;
    fs::write(dir.path().join("outer.rs"), "fn outer() {}")?;

    let results = search(
        "ext:rs",
        SearchOptions {
            root: nested.clone(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    assert!(results[0].path.ends_with("inner.rs"));
    Ok(())
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_unicode_in_file_content() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("emoji.rs"),
        "fn smile() { println!(\"😄\"); }",
    )?;

    let results = search(
        r#"contains:"😄""#,
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    Ok(())
}

#[test]
fn test_unicode_in_file_path() -> Result<()> {
    let dir = tempdir()?;
    let file = dir.path().join("unicodé.rs");
    fs::write(&file, "fn unicode_name() {}")?;

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    assert!(results[0]
        .path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("unicod"));
    Ok(())
}

#[test]
fn test_empty_file() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("empty.rs"), "")?;

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    assert!(results[0].content.is_empty());
    Ok(())
}

#[test]
fn test_very_long_lines() -> Result<()> {
    let dir = tempdir()?;
    let long_line = "a".repeat(50_000);
    fs::write(
        dir.path().join("long.rs"),
        format!("fn long() {{ {long_line} }}"),
    )?;

    let results = search(
        "contains:aaaa",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    assert_eq!(results.len(), 1);
    Ok(())
}

#[test]
fn test_symlinks_not_followed_by_default() -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let root = tempdir()?;
        let outside = tempdir()?;
        let outside_file = outside.path().join("external.txt");
        fs::write(&outside_file, "external")?;
        let inside_file = root.path().join("inside.txt");
        fs::write(&inside_file, "inside")?;

        let link_path = root.path().join("link.txt");
        symlink(&outside_file, &link_path)?;

        let results = search(
            "ext:txt",
            SearchOptions {
                root: root.path().to_path_buf(),
                ..Default::default()
            },
        )?;

        let names: Vec<_> = results
            .iter()
            .map(|r| r.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(results.len(), 1);
        assert!(names.contains(&"inside.txt".to_string()));
    }
    Ok(())
}

#[test]
fn test_binary_file_detection() -> Result<()> {
    let dir = tempdir()?;
    create_binary_file(&dir.path().to_path_buf(), "data.bin");

    let mut iter = search_iter(
        "ext:bin",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    let first = iter.next().expect("expected an item");
    let err = first.unwrap_err().to_string();
    assert!(err.to_lowercase().contains("binary"));
    Ok(())
}

#[test]
fn test_results_can_be_accessed_after_tempdir_dropped() -> Result<()> {
    let dir = tempdir()?;
    let file_path = dir.path().join("keep.rs");
    fs::write(&file_path, "fn keep() {}")?;

    let results = search(
        "ext:rs",
        SearchOptions {
            root: dir.path().to_path_buf(),
            ..Default::default()
        },
    )?;

    drop(dir);
    assert!(!file_path.exists());
    assert!(!results[0].content.is_empty());
    Ok(())
}
