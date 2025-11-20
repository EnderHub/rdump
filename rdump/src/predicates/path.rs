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
                expected = context.root.join(expected);
            }

            let normalized_expected = expected.canonicalize().unwrap_or_else(|_| expected.clone());
            let normalized_actual = context
                .path
                .canonicalize()
                .unwrap_or_else(|_| context.path.clone());

            return Ok(MatchResult::Boolean(
                normalized_actual == normalized_expected,
            ));
        }

        let path_str = context.path.to_string_lossy();

        if value.contains('*') || value.contains('?') || value.contains('[') || value.contains('{')
        {
            // Convert glob-style pattern to a regex
            let glob = Glob::new(value)?.compile_matcher();
            Ok(MatchResult::Boolean(glob.is_match(path_str.as_ref())))
        } else {
            // Fallback to simple substring search for non-glob patterns
            Ok(MatchResult::Boolean(path_str.contains(value)))
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
}
