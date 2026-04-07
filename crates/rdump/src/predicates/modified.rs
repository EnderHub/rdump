use super::{helpers, PredicateEvaluator};
use crate::evaluator::{FileContext, MatchResult};
use crate::parser::PredicateKey;
use anyhow::Result;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(super) struct ModifiedEvaluator;
impl PredicateEvaluator for ModifiedEvaluator {
    fn evaluate(
        &self,
        context: &mut FileContext,
        _key: &PredicateKey,
        value: &str,
    ) -> Result<MatchResult> {
        let modified_time = context
            .metadata()?
            .modified_unix_millis
            .map(|millis| UNIX_EPOCH + Duration::from_millis(millis as u64))
            .unwrap_or(SystemTime::UNIX_EPOCH);
        Ok(MatchResult::Boolean(helpers::parse_and_compare_time(
            modified_time,
            value,
        )?))
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
    fn test_modified_evaluator() {
        let file = create_temp_file("content");
        let mut context = FileContext::new(file.path().to_path_buf(), PathBuf::from("/"));

        let evaluator = ModifiedEvaluator;
        // File was just created
        assert!(evaluator
            .evaluate(&mut context, &PredicateKey::Modified, ">1m")
            .unwrap()
            .is_match()); // Modified more recently than 1 min ago
        assert!(!evaluator
            .evaluate(&mut context, &PredicateKey::Modified, "<1m")
            .unwrap()
            .is_match()); // Not modified longer than 1 min ago
    }
}
