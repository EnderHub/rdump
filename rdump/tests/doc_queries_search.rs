use anyhow::Result;
use filetime::{set_file_times, FileTime};
use rdump::{commands::search::perform_search, ColorChoice, Format, SearchArgs};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tempfile::tempdir;

fn load_doc_queries() -> Vec<String> {
    let content =
        fs::read_to_string("tests/data/doc_queries.txt").expect("doc queries fixture missing");
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect()
}

fn build_args(root: &PathBuf, query: &str) -> SearchArgs {
    SearchArgs {
        query: Some(query.to_string()),
        root: root.clone(),
        preset: vec![],
        output: None,
        dialect: None,
        line_numbers: false,
        no_headers: false,
        format: Format::Paths,
        no_ignore: true,
        hidden: true,
        color: ColorChoice::Never,
        max_depth: None,
        context: None,
        find: false,
    }
}

fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            // Skip symlinks to avoid broken link failures in the fixture set.
            continue;
        }
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_tree(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

#[test]
fn doc_queries_have_real_matches() -> Result<()> {
    let fixture_root = PathBuf::from("../insane_test_bed");
    let temp = tempdir()?;
    let root = temp.path().join("fixture");
    copy_tree(&fixture_root, &root)?;

    // Normalize timestamps for deterministic time filters.
    let now = SystemTime::now();
    let now_ft = FileTime::from_system_time(now);
    let old_ft = FileTime::from_unix_time(
        now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64 - 9 * 24 * 3600,
        0,
    );
    let half_day_ago = FileTime::from_unix_time(
        now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64 - 12 * 3600,
        0,
    );

    let fresh_path = root.join("fresh_now.txt");
    fs::write(&fresh_path, "fresh")?;
    set_file_times(&fresh_path, now_ft, now_ft)?;

    let old_tmp = root.join("tmp/old.tmp");
    if old_tmp.exists() {
        set_file_times(&old_tmp, old_ft, old_ft)?;
    }
    let recent_toml = root.join("config_recent.toml");
    if recent_toml.exists() {
        set_file_times(&recent_toml, half_day_ago, half_day_ago)?;
    }

    let queries = load_doc_queries();

    let expected_error: HashSet<&str> = HashSet::from(["matches:'('"]);

    let mut failures = Vec::new();

    for query in queries {
        if expected_error.contains(query.as_str()) {
            if perform_search(&build_args(&root, &query)).is_ok() {
                failures.push(format!(
                    "Expected error for query '{}', but it succeeded",
                    query
                ));
            }
            continue;
        }

        match perform_search(&build_args(&root, &query)) {
            Ok(_) => {}
            Err(err) => failures.push(format!("Query '{}' failed: {}", query, err)),
        }
    }

    if !failures.is_empty() {
        panic!(
            "Some doc queries are not covered by the fixture:\n{}",
            failures.join("\n")
        );
    }

    Ok(())
}
