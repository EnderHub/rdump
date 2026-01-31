use crate::types::{LimitValue, Limits};

pub const DEFAULT_MAX_RESULTS: usize = 50;
pub const DEFAULT_MAX_MATCHES_PER_FILE: usize = 20;
pub const DEFAULT_MAX_BYTES_PER_FILE: usize = 20_000;
pub const DEFAULT_MAX_TOTAL_BYTES: usize = 200_000;
pub const DEFAULT_MAX_MATCH_BYTES: usize = 200;
pub const DEFAULT_MAX_SNIPPET_BYTES: usize = 2_000;
pub const DEFAULT_MAX_ERRORS: usize = 10;
pub const DEFAULT_CONTEXT_LINES: usize = 2;

pub struct ResolvedLimits {
    pub max_results: usize,
    pub max_matches_per_file: usize,
    pub max_bytes_per_file: usize,
    pub max_total_bytes: usize,
    pub max_match_bytes: usize,
    pub max_snippet_bytes: usize,
    pub max_errors: usize,
}

pub fn resolve_limits(limits: Option<Limits>) -> ResolvedLimits {
    let limits = limits.unwrap_or_default();
    ResolvedLimits {
        max_results: resolve_limit(limits.max_results, DEFAULT_MAX_RESULTS),
        max_matches_per_file: resolve_limit(limits.max_matches_per_file, DEFAULT_MAX_MATCHES_PER_FILE),
        max_bytes_per_file: resolve_limit(limits.max_bytes_per_file, DEFAULT_MAX_BYTES_PER_FILE),
        max_total_bytes: resolve_limit(limits.max_total_bytes, DEFAULT_MAX_TOTAL_BYTES),
        max_match_bytes: resolve_limit(limits.max_match_bytes, DEFAULT_MAX_MATCH_BYTES),
        max_snippet_bytes: resolve_limit(limits.max_snippet_bytes, DEFAULT_MAX_SNIPPET_BYTES),
        max_errors: resolve_limit(limits.max_errors, DEFAULT_MAX_ERRORS),
    }
}

pub fn resolve_limit(value: LimitValue, default: usize) -> usize {
    match value {
        LimitValue::Value(value) => value,
        LimitValue::Unlimited => usize::MAX,
        LimitValue::Unset => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_limits_defaults() {
        let resolved = resolve_limits(None);
        assert_eq!(resolved.max_results, DEFAULT_MAX_RESULTS);
        assert_eq!(resolved.max_matches_per_file, DEFAULT_MAX_MATCHES_PER_FILE);
        assert_eq!(resolved.max_bytes_per_file, DEFAULT_MAX_BYTES_PER_FILE);
        assert_eq!(resolved.max_total_bytes, DEFAULT_MAX_TOTAL_BYTES);
        assert_eq!(resolved.max_match_bytes, DEFAULT_MAX_MATCH_BYTES);
        assert_eq!(resolved.max_snippet_bytes, DEFAULT_MAX_SNIPPET_BYTES);
        assert_eq!(resolved.max_errors, DEFAULT_MAX_ERRORS);
    }

    #[test]
    fn resolve_limits_null_values_become_unlimited() {
        let limits = Limits {
            max_results: LimitValue::Unlimited,
            max_matches_per_file: LimitValue::Unlimited,
            max_bytes_per_file: LimitValue::Unlimited,
            max_total_bytes: LimitValue::Unlimited,
            max_match_bytes: LimitValue::Unlimited,
            max_snippet_bytes: LimitValue::Unlimited,
            max_errors: LimitValue::Unlimited,
        };
        let resolved = resolve_limits(Some(limits));
        assert_eq!(resolved.max_results, usize::MAX);
        assert_eq!(resolved.max_matches_per_file, usize::MAX);
        assert_eq!(resolved.max_bytes_per_file, usize::MAX);
        assert_eq!(resolved.max_total_bytes, usize::MAX);
        assert_eq!(resolved.max_match_bytes, usize::MAX);
        assert_eq!(resolved.max_snippet_bytes, usize::MAX);
        assert_eq!(resolved.max_errors, usize::MAX);
    }

    #[test]
    fn resolve_limits_zero_values_remain_zero() {
        let limits = Limits {
            max_results: LimitValue::Value(0),
            max_matches_per_file: LimitValue::Value(0),
            max_bytes_per_file: LimitValue::Value(0),
            max_total_bytes: LimitValue::Value(0),
            max_match_bytes: LimitValue::Value(0),
            max_snippet_bytes: LimitValue::Value(0),
            max_errors: LimitValue::Value(0),
        };
        let resolved = resolve_limits(Some(limits));
        assert_eq!(resolved.max_results, 0);
        assert_eq!(resolved.max_matches_per_file, 0);
        assert_eq!(resolved.max_bytes_per_file, 0);
        assert_eq!(resolved.max_total_bytes, 0);
        assert_eq!(resolved.max_match_bytes, 0);
        assert_eq!(resolved.max_snippet_bytes, 0);
        assert_eq!(resolved.max_errors, 0);
    }
}
