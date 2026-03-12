use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchExecutionPolicy {
    pub max_concurrent_searches: usize,
    pub async_channel_capacity: usize,
    pub cancellation_check_interval: usize,
}

impl Default for SearchExecutionPolicy {
    fn default() -> Self {
        Self {
            max_concurrent_searches: parse_positive_env("RDUMP_MAX_CONCURRENT_SEARCHES")
                .unwrap_or_else(default_parallelism),
            async_channel_capacity: parse_positive_env("RDUMP_SEARCH_CHANNEL_CAPACITY")
                .or_else(|| parse_positive_env("RDUMP_ASYNC_CHANNEL_CAPACITY"))
                .unwrap_or(100),
            cancellation_check_interval: parse_positive_env("RDUMP_CANCEL_CHECK_INTERVAL")
                .unwrap_or(1),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchCancellationToken {
    inner: Arc<AtomicBool>,
}

impl SearchCancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.inner.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }
}

#[derive(Debug)]
pub struct CancelOnDrop {
    token: SearchCancellationToken,
    armed: bool,
}

impl CancelOnDrop {
    pub fn new(token: SearchCancellationToken) -> Self {
        Self { token, armed: true }
    }

    pub fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if self.armed {
            self.token.cancel();
        }
    }
}

static EXECUTION_POLICY: Lazy<SearchExecutionPolicy> = Lazy::new(SearchExecutionPolicy::default);

pub fn search_execution_policy() -> SearchExecutionPolicy {
    EXECUTION_POLICY.clone()
}

pub fn default_max_concurrent_searches() -> usize {
    EXECUTION_POLICY.max_concurrent_searches
}

fn parse_positive_env(key: &str) -> Option<usize> {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
}

fn default_parallelism() -> usize {
    std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_token_defaults_to_not_cancelled() {
        let token = SearchCancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn search_execution_policy_has_positive_values() {
        let policy = search_execution_policy();
        assert!(policy.max_concurrent_searches > 0);
        assert!(policy.async_channel_capacity > 0);
        assert!(policy.cancellation_check_interval > 0);
    }
}
