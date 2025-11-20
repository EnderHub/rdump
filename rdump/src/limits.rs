use std::path::PathBuf;
use std::time::Duration;

/// Maximum file size we will read in bytes (default: 100MB).
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Maximum directory depth we will traverse by default.
pub const DEFAULT_MAX_DEPTH: usize = 100;

/// Maximum time we will spend evaluating a single regex against a file's lines.
pub const MAX_REGEX_EVAL_DURATION: Duration = Duration::from_millis(200);

/// Returns true if the byte slice is likely a binary file.
pub fn is_probably_binary(bytes: &[u8]) -> bool {
    bytes.iter().any(|b| *b == 0)
}

/// Light heuristic to skip obvious secrets before printing them.
pub fn maybe_contains_secret(content: &str) -> bool {
    let lower = content.to_lowercase();
    lower.contains("-----begin private key-----")
        || lower.contains("aws_secret_access_key")
        || lower.contains("aws_access_key_id")
        || lower.contains("secret_key=")
        || lower.contains("secret-key=")
        || lower.contains("authorization: bearer")
        || lower.contains("eyj") // common JWT prefix (base64url '{"typ":"JWT"...}')
        || lower.contains("private_key")
}

/// Canonicalize `path` and ensure it stays under `root`. Returns the canonicalized path.
pub fn safe_canonicalize(path: &PathBuf, root: &PathBuf) -> anyhow::Result<PathBuf> {
    let canonical_root = dunce::canonicalize(root)?;
    let canonical = dunce::canonicalize(path)?;
    if !canonical.starts_with(&canonical_root) {
        anyhow::bail!(
            "Path {} escapes root {}",
            canonical.display(),
            canonical_root.display()
        );
    }
    Ok(canonical)
}
