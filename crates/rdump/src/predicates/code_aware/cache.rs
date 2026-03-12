use super::profiles;
use crate::parser::PredicateKey;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use tree_sitter::Query;

static QUERY_CACHE: Lazy<RwLock<HashMap<(String, PredicateKey), Arc<Query>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static QUERY_CACHE_HITS: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));
static QUERY_CACHE_MISSES: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));

pub(super) fn compiled_query(
    profile_key: &str,
    profile: &'static profiles::LanguageProfile,
    key: &PredicateKey,
    query: &str,
) -> Result<Arc<Query>> {
    let cache_key = (profile_key.to_string(), key.clone());

    if let Some(compiled) = QUERY_CACHE
        .read()
        .expect("query cache read lock poisoned")
        .get(&cache_key)
        .cloned()
    {
        QUERY_CACHE_HITS.fetch_add(1, Ordering::SeqCst);
        return Ok(compiled);
    }
    QUERY_CACHE_MISSES.fetch_add(1, Ordering::SeqCst);

    let compiled = Arc::new(
        Query::new(&profile.language, query)
            .with_context(|| format!("Failed to compile tree-sitter query for key {key:?}"))?,
    );

    let mut cache = QUERY_CACHE
        .write()
        .expect("query cache write lock poisoned");
    Ok(cache
        .entry(cache_key)
        .or_insert_with(|| compiled.clone())
        .clone())
}

pub fn cache_metrics_snapshot() -> (usize, usize) {
    (
        QUERY_CACHE_HITS.load(Ordering::SeqCst),
        QUERY_CACHE_MISSES.load(Ordering::SeqCst),
    )
}
