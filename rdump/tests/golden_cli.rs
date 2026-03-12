use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("rust_project")
}

#[test]
fn summary_output_matches_golden() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(fixture_root())
        .args(["search", "--format", "summary", "ext:rs"]);

    let expected = "./src/lib.rs\tmatches=0\twhole_file_match=true\tcontent_state=loaded\tdiagnostics=0\n./src/macros.rs\tmatches=0\twhole_file_match=true\tcontent_state=loaded\tdiagnostics=0\n./src/main.rs\tmatches=0\twhole_file_match=true\tcontent_state=loaded\tdiagnostics=0\n./src/traits.rs\tmatches=0\twhole_file_match=true\tcontent_state=loaded\tdiagnostics=0\n";

    cmd.assert().success().stdout(expected);
}

#[test]
fn matches_output_contains_expected_shape() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(fixture_root())
        .args(["search", "--format", "matches", "func:main"]);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("File:"))
        .stdout(predicates::str::contains("main"));
}
