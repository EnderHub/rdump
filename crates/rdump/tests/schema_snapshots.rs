use serde_json::Value as JsonValue;
use std::path::PathBuf;

fn normalize_snapshot(value: &mut JsonValue) {
    if let Some(root) = value.get("root").and_then(|entry| entry.as_str()) {
        let root = root.to_string();
        normalize_paths(value, &root);
    }

    if let Some(stats) = value
        .get_mut("stats")
        .and_then(|entry| entry.as_object_mut())
    {
        for key in [
            "walk_millis",
            "prefilter_millis",
            "evaluate_millis",
            "materialize_millis",
            "semaphore_wait_millis",
            "returned_bytes",
        ] {
            stats.insert(key.to_string(), JsonValue::from(0));
        }
    }

    normalize_fingerprints(value);
    normalize_modified_times(value);
}

fn normalize_paths(value: &mut JsonValue, root: &str) {
    match value {
        JsonValue::Object(map) => {
            for entry in map.values_mut() {
                normalize_paths(entry, root);
            }
        }
        JsonValue::Array(values) => {
            for entry in values {
                normalize_paths(entry, root);
            }
        }
        JsonValue::String(text) => {
            if text.starts_with('/') && text.contains(root.trim_start_matches("./")) {
                *text = text.replace(&fixture_root().display().to_string(), "<FIXTURE_ROOT>");
            }
        }
        _ => {}
    }
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mixed_project")
}

fn normalize_modified_times(value: &mut JsonValue) {
    match value {
        JsonValue::Object(map) => {
            if map.contains_key("modified_unix_millis") {
                map.insert("modified_unix_millis".to_string(), JsonValue::from(0));
            }
            for entry in map.values_mut() {
                normalize_modified_times(entry);
            }
        }
        JsonValue::Array(values) => {
            for entry in values {
                normalize_modified_times(entry);
            }
        }
        _ => {}
    }
}

fn normalize_fingerprints(value: &mut JsonValue) {
    match value {
        JsonValue::Object(map) => {
            if map.contains_key("fingerprint") {
                map.insert(
                    "fingerprint".to_string(),
                    JsonValue::String("<FINGERPRINT>".to_string()),
                );
            }
            for entry in map.values_mut() {
                normalize_fingerprints(entry);
            }
        }
        JsonValue::Array(values) => {
            for entry in values {
                normalize_fingerprints(entry);
            }
        }
        _ => {}
    }
}

#[test]
fn cli_full_json_snapshot_matches() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    let fixture_root = fixture_root();
    let output = cmd
        .args([
            "search",
            "--root",
            fixture_root.to_str().unwrap(),
            "--format",
            "json",
            "path:src/main.rs",
        ])
        .output()?;
    assert!(output.status.success());
    let mut json: JsonValue = serde_json::from_slice(&output.stdout)?;
    normalize_snapshot(&mut json);
    let mut expected: JsonValue =
        serde_json::from_str(include_str!(
            "../../../docs/generated/cli-search-full.snapshot.json"
        ))?;
    normalize_snapshot(&mut expected);
    assert_eq!(
        json, expected,
        "normalized CLI JSON output diverged from the full snapshot"
    );
    Ok(())
}

#[test]
fn cli_find_json_snapshot_matches() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    let fixture_root = fixture_root();
    let output = cmd
        .args([
            "search",
            "--root",
            fixture_root.to_str().unwrap(),
            "--find",
            "--format",
            "json",
            "path:src/main.rs",
        ])
        .output()?;
    assert!(output.status.success());
    let mut json: JsonValue = serde_json::from_slice(&output.stdout)?;
    normalize_snapshot(&mut json);
    let mut expected: JsonValue =
        serde_json::from_str(include_str!(
            "../../../docs/generated/cli-search-find.snapshot.json"
        ))?;
    normalize_snapshot(&mut expected);
    assert_eq!(
        json, expected,
        "normalized CLI JSON output diverged from the find snapshot"
    );
    Ok(())
}
