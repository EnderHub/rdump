use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_formatter_merges_overlapping_hunks() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let file_path = root.join("test.txt");

    // Create a file where two matches are close enough that their contexts will overlap.
    let content = "line 1\nline 2 (match 1)\nline 3\nline 4 (match 2)\nline 5\n";
    fs::write(&file_path, content).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(root);
    // Query for "match", with a context of 1 line (-C 1).
    // The context for "match 1" is lines 1-3.
    // The context for "match 2" is lines 3-5.
    // These overlap on line 3 and should be merged.
    cmd.arg("search").arg("contains:match").arg("-C").arg("1");

    // The output should be a single, continuous block from line 1 to 5.
    // Crucially, it should NOT contain the "..." separator that would
    // appear if the hunks were printed separately.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2 (match 1)"))
        .stdout(predicate::str::contains("line 3"))
        .stdout(predicate::str::contains("line 4 (match 2)"))
        .stdout(predicate::str::contains("line 5"))
        .stdout(predicate::str::contains("...").not());
}

#[test]
fn test_cat_and_hunks_preserve_crlf_line_endings() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let file_path = root.join("test.txt");
    fs::write(&file_path, b"line 1\r\nmatch here\r\nline 3\r\n").unwrap();

    let mut cat_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    let cat_output = cat_cmd
        .current_dir(root)
        .args(["search", "contains:match", "--format", "cat"])
        .output()
        .unwrap();
    assert!(cat_output.status.success());
    assert!(cat_output.stdout.windows(2).any(|bytes| bytes == b"\r\n"));

    let mut hunks_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    let hunks_output = hunks_cmd
        .current_dir(root)
        .args(["search", "contains:match", "--format", "hunks"])
        .output()
        .unwrap();
    assert!(hunks_output.status.success());
    assert!(hunks_output.stdout.windows(2).any(|bytes| bytes == b"\r\n"));
}

#[test]
fn test_json_escapes_crlf_line_endings() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let file_path = root.join("test.txt");
    fs::write(&file_path, b"line 1\r\nmatch here\r\nline 3\r\n").unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(root)
        .args(["search", "contains:match", "--format", "json"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\\r\\n"));
}
