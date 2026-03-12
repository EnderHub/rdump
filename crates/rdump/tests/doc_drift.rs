use std::fs;
use std::path::PathBuf;

#[test]
fn architecture_doc_mentions_current_public_surface() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let architecture = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs")
        .join("architecture.md");
    let text = fs::read_to_string(architecture).unwrap();

    for expected in [
        "search_with_stats",
        "search_path_iter",
        "search_paths",
        "explain_query",
        "ContentState",
        "SearchDiagnostic",
    ] {
        assert!(
            text.contains(expected),
            "architecture doc is missing current API marker: {expected}"
        );
    }
}

#[test]
fn operational_docs_track_current_output_and_platform_contracts() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let docs_dir = manifest_dir.parent().unwrap().parent().unwrap().join("docs");

    let output_parity = fs::read_to_string(docs_dir.join("output-parity.md")).unwrap();
    for expected in ["summary", "matches", "snippets", "full"] {
        assert!(
            output_parity.contains(expected),
            "output-parity.md is missing `{expected}`"
        );
    }

    let runtime = fs::read_to_string(docs_dir.join("runtime-guide.md")).unwrap();
    for expected in ["search_with_stats", "search_path_iter", "schema_version"] {
        assert!(
            runtime.contains(expected),
            "runtime-guide.md is missing `{expected}`"
        );
    }

    let platform = fs::read_to_string(docs_dir.join("cross-platform-semantics.md")).unwrap();
    for expected in ["root_relative_path", "permissions_display", "line endings"] {
        assert!(
            platform.contains(expected),
            "cross-platform-semantics.md is missing `{expected}`"
        );
    }
}
