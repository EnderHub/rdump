use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::backend::{RealFsSearchBackend, SearchBackend};
use crate::limits::{is_probably_binary, maybe_contains_secret, MAX_FILE_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    ContentSkipped,
    ContentDecodedLossy,
    WalkWarning,
    IgnoreExcluded,
    RootBoundaryExcluded,
    PathResolutionFallback,
    SemanticSkip,
    LanguageSelection,
    SqlDialectTrace,
    DeprecatedQueryAlias,
    SnapshotDrift,
    FormatResolution,
    QueueOverload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchDiagnostic {
    pub level: DiagnosticLevel,
    pub kind: DiagnosticKind,
    pub message: String,
    pub path: Option<PathBuf>,
}

impl SearchDiagnostic {
    pub fn new(
        level: DiagnosticLevel,
        kind: DiagnosticKind,
        message: impl Into<String>,
        path: Option<PathBuf>,
    ) -> Self {
        Self {
            level,
            kind,
            message: message.into(),
            path,
        }
    }

    pub fn content_skipped(
        path: impl Into<PathBuf>,
        reason: ContentSkipReason,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            DiagnosticLevel::Warn,
            DiagnosticKind::ContentSkipped,
            format!("{} ({})", message.into(), reason.as_str()),
            Some(path.into()),
        )
    }

    pub fn walk_warning(path: Option<PathBuf>, message: impl Into<String>) -> Self {
        Self::new(
            DiagnosticLevel::Warn,
            DiagnosticKind::WalkWarning,
            message,
            path,
        )
    }

    pub fn ignore_excluded(
        path: impl Into<PathBuf>,
        source: impl AsRef<str>,
        pattern: impl AsRef<str>,
    ) -> Self {
        Self::new(
            DiagnosticLevel::Info,
            DiagnosticKind::IgnoreExcluded,
            format!(
                "Excluded by {} pattern `{}`",
                source.as_ref(),
                pattern.as_ref()
            ),
            Some(path.into()),
        )
    }

    pub fn root_boundary(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::new(
            DiagnosticLevel::Warn,
            DiagnosticKind::RootBoundaryExcluded,
            message,
            Some(path.into()),
        )
    }

    pub fn path_resolution_fallback(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::new(
            DiagnosticLevel::Warn,
            DiagnosticKind::PathResolutionFallback,
            message,
            Some(path.into()),
        )
    }

    pub fn semantic_skip(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::new(
            DiagnosticLevel::Warn,
            DiagnosticKind::SemanticSkip,
            message,
            Some(path.into()),
        )
    }

    pub fn language_selection(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::new(
            DiagnosticLevel::Info,
            DiagnosticKind::LanguageSelection,
            message,
            Some(path.into()),
        )
    }

    pub fn sql_dialect_trace(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::new(
            DiagnosticLevel::Info,
            DiagnosticKind::SqlDialectTrace,
            message,
            Some(path.into()),
        )
    }

    pub fn deprecated_query_alias(message: impl Into<String>) -> Self {
        Self::new(
            DiagnosticLevel::Warn,
            DiagnosticKind::DeprecatedQueryAlias,
            message,
            None,
        )
    }

    pub fn snapshot_drift(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::new(
            DiagnosticLevel::Warn,
            DiagnosticKind::SnapshotDrift,
            message,
            Some(path.into()),
        )
    }

    pub fn format_resolution(message: impl Into<String>) -> Self {
        Self::new(
            DiagnosticLevel::Warn,
            DiagnosticKind::FormatResolution,
            message,
            None,
        )
    }

    pub fn queue_overload(message: impl Into<String>) -> Self {
        Self::new(
            DiagnosticLevel::Warn,
            DiagnosticKind::QueueOverload,
            message,
            None,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSkipReason {
    TooLarge,
    Binary,
    SecretLike,
}

impl ContentSkipReason {
    pub fn as_str(self) -> &'static str {
        match self {
            ContentSkipReason::TooLarge => "too_large",
            ContentSkipReason::Binary => "binary",
            ContentSkipReason::SecretLike => "secret_like",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContentState {
    Loaded,
    LoadedLossy,
    Skipped { reason: ContentSkipReason },
}

impl ContentState {
    pub fn is_loaded(&self) -> bool {
        matches!(self, ContentState::Loaded | ContentState::LoadedLossy)
    }
}

#[derive(Debug, Clone)]
pub struct LoadedContent {
    pub content: Arc<str>,
    pub state: ContentState,
    pub diagnostics: Vec<SearchDiagnostic>,
}

pub fn load_search_content(path: &Path) -> Result<LoadedContent> {
    load_search_content_with_backend(&RealFsSearchBackend, path, path)
}

pub fn load_search_content_with_backend(
    backend: &dyn SearchBackend,
    resolved_path: &Path,
    display_path: &Path,
) -> Result<LoadedContent> {
    let metadata = backend
        .stat(resolved_path)
        .with_context(|| format!("Failed to read metadata for {}", display_path.display()))?;

    if metadata.size_bytes > MAX_FILE_SIZE {
        return Ok(LoadedContent {
            content: Arc::<str>::from(""),
            state: ContentState::Skipped {
                reason: ContentSkipReason::TooLarge,
            },
            diagnostics: vec![SearchDiagnostic::content_skipped(
                display_path.to_path_buf(),
                ContentSkipReason::TooLarge,
                format!(
                    "Skipping {} because it exceeds the max file size of {} bytes",
                    display_path.display(),
                    MAX_FILE_SIZE
                ),
            )],
        });
    }

    let bytes = backend
        .read_bytes(resolved_path)
        .with_context(|| format!("Failed to read file {}", display_path.display()))?;
    let check_len = bytes.len().min(8192);

    if is_probably_binary(&bytes[..check_len]) {
        return Ok(LoadedContent {
            content: Arc::<str>::from(""),
            state: ContentState::Skipped {
                reason: ContentSkipReason::Binary,
            },
            diagnostics: vec![SearchDiagnostic::content_skipped(
                display_path.to_path_buf(),
                ContentSkipReason::Binary,
                format!("Skipping binary file {}", display_path.display()),
            )],
        });
    }

    match String::from_utf8(bytes) {
        Ok(content) => {
            if maybe_contains_secret(&content) {
                return Ok(LoadedContent {
                    content: Arc::<str>::from(""),
                    state: ContentState::Skipped {
                        reason: ContentSkipReason::SecretLike,
                    },
                    diagnostics: vec![SearchDiagnostic::content_skipped(
                        display_path.to_path_buf(),
                        ContentSkipReason::SecretLike,
                        format!(
                            "Skipping possible secret-containing file {}",
                            display_path.display()
                        ),
                    )],
                });
            }

            Ok(LoadedContent {
                content: Arc::from(content.into_boxed_str()),
                state: ContentState::Loaded,
                diagnostics: Vec::new(),
            })
        }
        Err(err) => {
            let content = String::from_utf8_lossy(err.as_bytes()).into_owned();
            if maybe_contains_secret(&content) {
                return Ok(LoadedContent {
                    content: Arc::<str>::from(""),
                    state: ContentState::Skipped {
                        reason: ContentSkipReason::SecretLike,
                    },
                    diagnostics: vec![SearchDiagnostic::content_skipped(
                        display_path.to_path_buf(),
                        ContentSkipReason::SecretLike,
                        format!(
                            "Skipping possible secret-containing file {}",
                            display_path.display()
                        ),
                    )],
                });
            }

            Ok(LoadedContent {
                content: Arc::from(content.into_boxed_str()),
                state: ContentState::LoadedLossy,
                diagnostics: vec![SearchDiagnostic::new(
                    DiagnosticLevel::Warn,
                    DiagnosticKind::ContentDecodedLossy,
                    format!(
                        "Decoded {} with lossy UTF-8 replacement",
                        display_path.display()
                    ),
                    Some(display_path.to_path_buf()),
                )],
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn loads_utf8_content() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "hello").unwrap();

        let loaded = load_search_content(&path).unwrap();
        assert_eq!(loaded.content.as_ref(), "hello");
        assert_eq!(loaded.state, ContentState::Loaded);
        assert!(loaded.diagnostics.is_empty());
    }

    #[test]
    fn decodes_invalid_utf8_lossily() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, [0x41, 0x42, 0xC3, 0x28, 0x43]).unwrap();

        let loaded = load_search_content(&path).unwrap();
        assert!(loaded.state.is_loaded());
        assert_eq!(loaded.state, ContentState::LoadedLossy);
        assert!(!loaded.diagnostics.is_empty());
    }

    #[test]
    fn skips_binary_files() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.bin");
        std::fs::write(&path, b"abc\x00def").unwrap();

        let loaded = load_search_content(&path).unwrap();
        assert_eq!(
            loaded.state,
            ContentState::Skipped {
                reason: ContentSkipReason::Binary
            }
        );
        assert!(loaded.content.is_empty());
    }

    #[test]
    fn skips_secret_like_files() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("secret.txt");
        std::fs::write(&path, "aws_secret_access_key=123").unwrap();

        let loaded = load_search_content(&path).unwrap();
        assert_eq!(
            loaded.state,
            ContentState::Skipped {
                reason: ContentSkipReason::SecretLike
            }
        );
        assert!(loaded.content.is_empty());
    }
}
