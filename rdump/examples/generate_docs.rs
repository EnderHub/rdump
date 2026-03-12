use anyhow::{bail, Result};

fn main() -> Result<()> {
    let Some(kind) = std::env::args().nth(1) else {
        bail!("usage: cargo run --example generate_docs -- <predicate-catalog|language-matrix|language-profiles|support-matrix>");
    };

    match kind.as_str() {
        "predicate-catalog" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&rdump::request::predicate_catalog())?
            );
        }
        "language-matrix" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&rdump::request::language_capability_matrix())?
            );
        }
        "language-profiles" => {
            print!(
                "{}",
                rdump::predicates::code_aware::profiles::render_language_profile_reference()
            );
        }
        "support-matrix" => {
            print!(
                "{}",
                rdump::support_matrix::render_support_matrix_markdown()
            );
        }
        other => bail!("unknown doc kind `{other}`"),
    }

    Ok(())
}
