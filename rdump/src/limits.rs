use std::path::PathBuf;
use std::time::Duration;

/// Maximum file size we will read in bytes (default: 10MB).
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum directory depth we will traverse by default.
pub const DEFAULT_MAX_DEPTH: usize = 100;

/// Maximum time we will spend evaluating a single regex against a file's lines.
pub const MAX_REGEX_EVAL_DURATION: Duration = Duration::from_millis(200);

/// Returns true if the byte slice is likely a binary file.
pub fn is_probably_binary(bytes: &[u8]) -> bool {
    bytes.contains(&0)
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
        || lower.contains("eyj")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_is_probably_binary() {
        assert!(is_probably_binary(&[0, 1, 2, 3]));
        assert!(is_probably_binary(b"hello\x00world"));
        assert!(!is_probably_binary(b"hello world"));
        assert!(!is_probably_binary(b"fn main() {}"));
    }

    #[test]
    fn test_maybe_contains_secret_private_key() {
        assert!(maybe_contains_secret("-----BEGIN PRIVATE KEY-----"));
        assert!(maybe_contains_secret(
            "some text with -----begin private key----- in it"
        ));
    }

    #[test]
    fn test_maybe_contains_secret_aws() {
        assert!(maybe_contains_secret("aws_secret_access_key=abcd1234"));
        assert!(maybe_contains_secret("AWS_ACCESS_KEY_ID=AKIA..."));
    }

    #[test]
    fn test_maybe_contains_secret_other() {
        assert!(maybe_contains_secret("secret_key=mykey"));
        assert!(maybe_contains_secret("secret-key=mykey"));
        assert!(maybe_contains_secret("Authorization: Bearer token"));
        assert!(maybe_contains_secret("eyJhbGciOiJIUzI1NiJ9"));
        assert!(maybe_contains_secret("private_key: xyz"));
    }

    #[test]
    fn test_maybe_contains_secret_safe() {
        assert!(!maybe_contains_secret("fn main() { println!(\"Hello\"); }"));
        assert!(!maybe_contains_secret("SELECT * FROM users;"));
    }

    #[test]
    fn test_safe_canonicalize_within_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let subdir = root.join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file = subdir.join("test.txt");
        fs::write(&file, "content").unwrap();

        let result = safe_canonicalize(&file, &root);
        assert!(result.is_ok());
        assert!(result
            .unwrap()
            .starts_with(dunce::canonicalize(&root).unwrap()));
    }

    #[test]
    fn test_safe_canonicalize_escapes_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("project");
        fs::create_dir(&root).unwrap();

        let outside_file = dir.path().join("outside.txt");
        fs::write(&outside_file, "content").unwrap();

        let result = safe_canonicalize(&outside_file, &root);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("escapes root"));
    }

    #[test]
    fn test_safe_canonicalize_nonexistent_path() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let nonexistent = root.join("nonexistent.txt");

        let result = safe_canonicalize(&nonexistent, &root);
        assert!(result.is_err());
    }
}
