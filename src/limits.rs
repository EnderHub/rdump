use std::path::PathBuf;

/// Maximum file size we will read in bytes (default: 100MB).
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Maximum directory depth we will traverse by default.
pub const DEFAULT_MAX_DEPTH: usize = 100;

/// Returns true if the byte slice is likely a binary file.
pub fn is_probably_binary(bytes: &[u8]) -> bool {
    bytes.iter().any(|b| *b == 0)
}

/// Canonicalize `path` and ensure it stays under `root`. Returns the canonicalized path.
pub fn safe_canonicalize(path: &PathBuf, root: &PathBuf) -> anyhow::Result<PathBuf> {
    let canonical = dunce::canonicalize(path)?;
    if !canonical.starts_with(root) {
        anyhow::bail!("Path {} escapes root {}", canonical.display(), root.display());
    }
    Ok(canonical)
}
