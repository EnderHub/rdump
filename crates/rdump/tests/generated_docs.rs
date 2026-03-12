use rdump::predicates::code_aware::profiles::render_language_profile_reference;
use rdump::support_matrix::render_support_matrix_markdown;

#[test]
fn generated_language_profile_doc_is_in_sync() {
    assert_eq!(
        render_language_profile_reference(),
        include_str!("../../../docs/generated/language-profile-reference.md")
    );
}

#[test]
fn generated_support_matrix_doc_is_in_sync() {
    assert_eq!(
        render_support_matrix_markdown(),
        include_str!("../../../docs/generated/test-support-matrix.md")
    );
}

#[test]
fn generated_predicate_catalog_json_is_in_sync() {
    let expected = serde_json::to_string_pretty(&rdump::request::predicate_catalog())
        .expect("predicate catalog should serialize")
        + "\n";
    assert_eq!(
        expected,
        include_str!("../../../docs/generated/predicate-catalog.json")
    );
}

#[test]
fn generated_language_matrix_json_is_in_sync() {
    let expected = serde_json::to_string_pretty(&rdump::request::language_capability_matrix())
        .expect("language matrix should serialize")
        + "\n";
    assert_eq!(
        expected,
        include_str!("../../../docs/generated/language-matrix.json")
    );
}
