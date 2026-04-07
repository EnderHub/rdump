use super::PredicateEvaluator;
use crate::evaluator::{FileContext, MatchResult};
use crate::parser::PredicateKey;
use anyhow::Result;
use globset::Glob;
use std::path::PathBuf;

pub(super) struct PathEvaluator;

impl PredicateEvaluator for PathEvaluator {
    fn evaluate(
        &self,
        context: &mut FileContext,
        key: &PredicateKey,
        value: &str,
    ) -> Result<MatchResult> {
        if let PredicateKey::PathExact = key {
            let mut expected = PathBuf::from(value);
            if expected.is_relative() {
                expected = context.root_path().join(expected);
            }
            let normalized_expected = context
                .normalized_path(&expected)
                .map(|identity| identity.resolved_path)
                .unwrap_or(expected);

            return Ok(MatchResult::Boolean(
                context.resolved_path() == normalized_expected.as_path(),
            ));
        }

        let relative_path = context
            .root_relative_path()
            .map(PathBuf::from)
            .unwrap_or_else(|| context.resolved_path().to_path_buf());

        let path_str = relative_path.to_string_lossy();
        let absolute_path_str = context.resolved_path().to_string_lossy();
        let value_path = std::path::Path::new(value);
        let use_absolute = value_path.is_absolute();

        if value.contains('*') || value.contains('?') || value.contains('[') || value.contains('{')
        {
            // Convert glob-style pattern to a regex
            let glob = Glob::new(value)?.compile_matcher();
            let target = if use_absolute {
                absolute_path_str.as_ref()
            } else {
                path_str.as_ref()
            };
            Ok(MatchResult::Boolean(glob.is_match(target)))
        } else {
            // Fallback to simple substring search for non-glob patterns
            let target = if use_absolute {
                absolute_path_str.as_ref()
            } else {
                path_str.as_ref()
            };
            Ok(MatchResult::Boolean(target.contains(value)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_evaluator_contains() {
        let mut context = FileContext::new(
            PathBuf::from("/home/user/project/src/main.rs"),
            PathBuf::from("/"),
        );
        let evaluator = PathEvaluator;
        assert!(evaluator
            .evaluate(&mut context, &PredicateKey::Path, "project/src")
            .unwrap()
            .is_match());
        assert!(evaluator
            .evaluate(&mut context, &PredicateKey::Path, "/home/user")
            .unwrap()
            .is_match());
        assert!(!evaluator
            .evaluate(&mut context, &PredicateKey::Path, "project/lib")
            .unwrap()
            .is_match());
        assert!(evaluator
            .evaluate(&mut context, &PredicateKey::Path, "main.rs")
            .unwrap()
            .is_match());
    }

    #[test]
    fn test_path_evaluator_wildcard() {
        let mut context = FileContext::new(
            PathBuf::from("/home/user/project/src/main.rs"),
            PathBuf::from("/"),
        );
        let evaluator = PathEvaluator;

        // This should match because ** crosses directory boundaries
        assert!(evaluator
            .evaluate(&mut context, &PredicateKey::Path, "**/main.rs")
            .unwrap()
            .is_match());
        // This should also match
        assert!(evaluator
            .evaluate(
                &mut context,
                &PredicateKey::Path,
                "/home/user/project/src/*.rs"
            )
            .unwrap()
            .is_match());
        // This SHOULD match because a glob without a separator matches the file name.
        assert!(evaluator
            .evaluate(&mut context, &PredicateKey::Path, "*.rs")
            .unwrap()
            .is_match());
        // This should match
        assert!(evaluator
            .evaluate(&mut context, &PredicateKey::Path, "**/*.rs")
            .unwrap()
            .is_match());
        assert!(!evaluator
            .evaluate(&mut context, &PredicateKey::Path, "**/*.ts")
            .unwrap()
            .is_match());
    }

    #[test]
    fn test_empty_path_query() {
        let mut context = FileContext::new(
            PathBuf::from("/home/user/project/src/main.rs"),
            PathBuf::from("/"),
        );
        let evaluator = PathEvaluator;

        // Empty string should match everything with `contains`
        assert!(evaluator
            .evaluate(&mut context, &PredicateKey::Path, "")
            .unwrap()
            .is_match());
    }

    #[test]
    fn test_path_exact_matches_absolute_and_relative() {
        let file_path = PathBuf::from("/home/user/project/src/main.rs");
        let root = PathBuf::from("/home/user/project");
        let mut context = FileContext::new(file_path.clone(), root.clone());
        let evaluator = PathEvaluator;

        // Absolute match should succeed
        assert!(evaluator
            .evaluate(
                &mut context,
                &PredicateKey::PathExact,
                "/home/user/project/src/main.rs"
            )
            .unwrap()
            .is_match());

        // Relative to root should also succeed
        let mut context_relative = FileContext::new(file_path, root);
        assert!(evaluator
            .evaluate(
                &mut context_relative,
                &PredicateKey::PathExact,
                "src/main.rs"
            )
            .unwrap()
            .is_match());
    }

    #[test]
    fn test_path_strip_prefix_failure() {
        // Create a context where the path doesn't share prefix with root
        // This is a synthetic test case to exercise the or_else fallback
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        // Use the canonical path
        let canonical_path = dunce::canonicalize(&file_path).unwrap();
        let root = dir.path().to_path_buf();

        let mut context = FileContext::new(canonical_path, root);
        let evaluator = PathEvaluator;

        // Test with a pattern that should match
        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Path, "test.rs")
            .unwrap();
        assert!(result.is_match());
    }

    #[test]
    fn test_path_absolute_glob_pattern() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        let file_path = src_dir.join("main.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let canonical_path = dunce::canonicalize(&file_path).unwrap();
        let canonical_root = dunce::canonicalize(dir.path()).unwrap();

        let mut context = FileContext::new(canonical_path.clone(), canonical_root.clone());
        let evaluator = PathEvaluator;

        // Test absolute glob pattern
        let pattern = format!("{}/**/*.rs", canonical_root.display());
        let result = evaluator
            .evaluate(&mut context, &PredicateKey::Path, &pattern)
            .unwrap();
        assert!(result.is_match());
    }

    #[test]
    fn test_path_non_glob_absolute() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let canonical_path = dunce::canonicalize(&file_path).unwrap();
        let canonical_root = dunce::canonicalize(dir.path()).unwrap();

        let mut context = FileContext::new(canonical_path.clone(), canonical_root);
        let evaluator = PathEvaluator;

        // Test absolute non-glob pattern
        let result = evaluator
            .evaluate(
                &mut context,
                &PredicateKey::Path,
                canonical_path.to_str().unwrap(),
            )
            .unwrap();
        assert!(result.is_match());
    }
}
