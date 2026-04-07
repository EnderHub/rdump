use rdump::predicates::{
    content_predicate_keys, metadata_predicate_keys, react_predicate_keys, semantic_predicate_keys,
};
use std::fs;
use std::path::PathBuf;

#[test]
fn rql_reference_tracks_live_predicate_registry() {
    let reference = rdump_mcp::docs::build_rql_reference();

    for key in metadata_predicate_keys() {
        if key.as_ref() == "path_exact" {
            continue;
        }
        assert!(reference
            .metadata_predicates
            .contains(&key.as_ref().to_string()));
    }
    for key in content_predicate_keys() {
        assert!(reference
            .content_predicates
            .contains(&key.as_ref().to_string()));
    }
    for key in semantic_predicate_keys() {
        assert!(reference
            .semantic_predicates
            .contains(&key.as_ref().to_string()));
    }
    for key in react_predicate_keys() {
        assert!(reference
            .react_predicates
            .contains(&key.as_ref().to_string()));
    }
}

#[test]
fn readme_mentions_current_mcp_contract_markers() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let readme = manifest_dir.join("README.md");
    let text = fs::read_to_string(readme).unwrap();

    assert!(text.contains("schema_version"));
    assert!(text.contains("error_mode"));
    assert!(text.contains("capability_metadata"));
}

#[test]
fn mcp_docs_track_runtime_and_session_resources() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let docs_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs");

    let stdio = fs::read_to_string(docs_dir.join("mcp-stdio-guide.md")).unwrap();
    for expected in [
        "continuation_token",
        "rdump://docs/session-cache",
        "rdump://docs/schema-examples",
    ] {
        assert!(
            stdio.contains(expected),
            "mcp-stdio-guide.md is missing `{expected}`"
        );
    }

    let runtime = fs::read_to_string(docs_dir.join("runtime-guide.md")).unwrap();
    for expected in ["output=summary", "schema_version", "diagnostics"] {
        assert!(
            runtime.contains(expected),
            "runtime-guide.md is missing `{expected}`"
        );
    }
}
