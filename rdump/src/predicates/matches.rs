use super::PredicateEvaluator;
use crate::evaluator::{FileContext, MatchResult};
use crate::parser::PredicateKey;
use anyhow::Result;
use std::time::Instant;
use regex::RegexBuilder;
use tree_sitter::Range;
use crate::limits::MAX_REGEX_EVAL_DURATION;

pub(super) struct MatchesEvaluator;
impl PredicateEvaluator for MatchesEvaluator {
    fn evaluate(
        &self,
        context: &mut FileContext,
        _key: &PredicateKey,
        value: &str,
    ) -> Result<MatchResult> {
        let content = context.get_content()?;
        let re = RegexBuilder::new(value)
            .size_limit(10 * 1024 * 1024) // Cap regex state to avoid pathological patterns
            .build()?;

        let mut ranges = Vec::new();
        let start = Instant::now();
        for (i, line) in content.lines().enumerate() {
            if i % 100 == 0 && start.elapsed() > MAX_REGEX_EVAL_DURATION {
                anyhow::bail!("Regex evaluation timed out");
            }
            if re.is_match(line) {
                let start_byte = content.lines().take(i).map(|l| l.len() + 1).sum();
                let end_byte = start_byte + line.len();
                ranges.push(Range {
                    start_byte,
                    end_byte,
                    start_point: tree_sitter::Point { row: i, column: 0 },
                    end_point: tree_sitter::Point {
                        row: i,
                        column: line.len(),
                    },
                });
            }
        }
        Ok(MatchResult::Hunks(ranges))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_matches_evaluator() {
        let file = create_temp_file("version = \"0.1.0\"\nauthor = \"test\"");
        let mut context = FileContext::new(file.path().to_path_buf(), PathBuf::from("/"));
        let evaluator = MatchesEvaluator;
        // Simple regex
        assert!(evaluator
            .evaluate(
                &mut context,
                &PredicateKey::Matches,
                r#"version = "[0-9]+\.[0-9]+\.[0-9]+""#
            )
            .unwrap()
            .is_match());
        // Test regex that finds a line
        assert!(evaluator
            .evaluate(&mut context, &PredicateKey::Matches, r#"author = "test""#)
            .unwrap()
            .is_match());
        assert!(!evaluator
            .evaluate(
                &mut context,
                &PredicateKey::Matches,
                r#"^version = "1.0.0"$"#
            )
            .unwrap()
            .is_match());
    }
}
