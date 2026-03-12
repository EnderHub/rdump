//! Async API for rdump search.
//!
//! Enabled with the `async` feature. Bridges the synchronous iterator to an
//! async stream using `tokio::task::spawn_blocking` and a bounded channel for
//! backpressure.

use crate::{
    search_execution_policy, search_iter, CancelOnDrop, SearchCancellationToken,
    SearchExecutionPolicy, SearchOptions, SearchResult,
};
use anyhow::Result;
use futures::Stream;
use once_cell::sync::Lazy;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::sync::Semaphore;
use tokio_stream::wrappers::ReceiverStream;

static SEARCH_SEMAPHORE: Lazy<Arc<Semaphore>> = Lazy::new(|| {
    Arc::new(Semaphore::new(
        search_execution_policy().max_concurrent_searches,
    ))
});
static ASYNC_SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct SearchAsyncStream {
    inner: ReceiverStream<Result<SearchResult>>,
    _cancel_on_drop: CancelOnDrop,
}

impl Stream for SearchAsyncStream {
    type Item = Result<SearchResult>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        Pin::new(&mut this.inner).poll_next(cx)
    }
}

/// Run an rdump query and stream results asynchronously. Requires the `async`
/// feature.
///
/// The underlying search still uses rayon-powered parallelism; results are
/// forwarded over a bounded channel (capacity 100) to provide backpressure.
/// Dropping the returned stream signals the producer to stop early.
///
/// # Arguments
/// - `query`: RQL query string (e.g., `ext:rs & func:main`)
/// - `options`: Search configuration
///
/// # Returns
/// A stream of `Result<SearchResult>` items.
///
/// # Errors
/// - Invalid query syntax
/// - Unknown preset name
/// - Root directory missing or inaccessible
/// - Failure to spawn the blocking task
///
/// # Examples
/// ```rust
/// # use tempfile::tempdir;
/// use futures::StreamExt;
/// use rdump::{search_async, SearchOptions};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// #   let dir = tempdir()?;
/// #   std::fs::write(dir.path().join("main.rs"), "fn main() {}")?;
/// #   let options = SearchOptions { root: dir.path().to_path_buf(), ..Default::default() };
/// let mut stream = search_async("func:main", SearchOptions::default()).await?;
/// while let Some(result) = stream.next().await {
///     let result = result?;
///     println!("{}", result.path.display());
/// }
/// #   Ok(())
/// # }
/// ```
pub async fn search_async(query: &str, options: SearchOptions) -> Result<SearchAsyncStream> {
    search_async_with_progress(query, options, |_| {}).await
}

pub async fn search_async_with_progress<F>(
    query: &str,
    options: SearchOptions,
    mut progress: F,
) -> Result<SearchAsyncStream>
where
    F: FnMut(rdump_contracts::ProgressEvent) + Send + 'static,
{
    let query = query.to_string();
    let policy = search_execution_policy();
    let (tx, rx) = mpsc::channel(policy.async_channel_capacity);
    let join_tx = tx.clone();
    let cancellation = SearchCancellationToken::new();
    let task_cancellation = cancellation.clone();
    let queue_wait_started = Instant::now();
    let permit = SEARCH_SEMAPHORE
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| anyhow::anyhow!("search limiter is closed"))?;
    let queue_wait_millis = queue_wait_started.elapsed().as_millis() as u64;
    let session_id = format!(
        "async-{}",
        ASYNC_SESSION_COUNTER.fetch_add(1, Ordering::Relaxed)
    );

    let root = options.root.display().to_string();
    let handle = tokio::task::spawn_blocking(move || {
        progress(rdump_contracts::ProgressEvent::Started {
            session_id: session_id.clone(),
            query: query.clone(),
            effective_query: query.clone(),
            root,
            queue_wait_millis,
        });
        let _permit = permit;
        let iter = match search_iter(&query, options) {
            Ok(iter) => iter,
            Err(e) => {
                let _ = tx.blocking_send(Err(e));
                return;
            }
        };
        let stats = iter.stats().clone();
        progress(rdump_contracts::ProgressEvent::Phase {
            session_id: session_id.clone(),
            name: "discover".to_string(),
            completed_items: stats.candidate_files,
            total_items: Some(stats.candidate_files),
        });
        progress(rdump_contracts::ProgressEvent::Phase {
            session_id: session_id.clone(),
            name: "prefilter".to_string(),
            completed_items: stats.prefiltered_files,
            total_items: Some(stats.candidate_files),
        });
        progress(rdump_contracts::ProgressEvent::Phase {
            session_id: session_id.clone(),
            name: "evaluate".to_string(),
            completed_items: stats.evaluated_files,
            total_items: Some(stats.prefiltered_files.max(stats.evaluated_files)),
        });
        progress(rdump_contracts::ProgressEvent::Phase {
            session_id: session_id.clone(),
            name: "materialize".to_string(),
            completed_items: 0,
            total_items: Some(iter.remaining()),
        });

        for (index, result) in iter.enumerate() {
            if should_stop(&task_cancellation, &policy, index) {
                progress(rdump_contracts::ProgressEvent::Finished {
                    session_id: session_id.clone(),
                    returned_files: index,
                    returned_matches: 0,
                    truncated: true,
                });
                break;
            }
            if tx.blocking_send(result).is_err() {
                task_cancellation.cancel();
                break;
            }
            progress(rdump_contracts::ProgressEvent::Phase {
                session_id: session_id.clone(),
                name: "materialize".to_string(),
                completed_items: index + 1,
                total_items: None,
            });
        }
        progress(rdump_contracts::ProgressEvent::Finished {
            session_id,
            returned_files: 0,
            returned_matches: 0,
            truncated: false,
        });
    });

    tokio::spawn(async move {
        if let Err(err) = handle.await {
            let _ = join_tx
                .send(Err(anyhow::anyhow!("search task failed to join: {err}")))
                .await;
        }
    });

    Ok(SearchAsyncStream {
        inner: ReceiverStream::new(rx),
        _cancel_on_drop: CancelOnDrop::new(cancellation),
    })
}

/// Search for files matching a query (async, convenience). Requires the `async`
/// feature.
///
/// Collects all results into a `Vec`. For large result sets, prefer
/// [`search_async`] and stream results to avoid loading all content into
/// memory at once.
///
/// # Arguments
/// - `query`: RQL query string (e.g., `ext:rs & func:main`)
/// - `options`: Search configuration
///
/// # Returns
/// A vector of all matching files with their content loaded.
///
/// # Errors
/// - Invalid query syntax
/// - Unknown preset name
/// - Root directory missing or inaccessible
/// - First per-file error encountered during iteration
///
/// # Examples
/// ```rust
/// # use tempfile::tempdir;
/// use rdump::{search_all_async, SearchOptions};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// #   let dir = tempdir()?;
/// #   std::fs::write(dir.path().join("main.rs"), "fn main() {}")?;
/// #   let options = SearchOptions { root: dir.path().to_path_buf(), ..Default::default() };
/// let results = search_all_async("func:main", SearchOptions::default()).await?;
/// assert!(!results.is_empty());
/// #   Ok(())
/// # }
/// ```
///
/// # Performance Note
/// This collects everything into memory. For large repositories, use
/// [`search_async`] and stream incrementally.
pub async fn search_all_async(query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
    use futures::StreamExt;

    let stream = search_async(query, options).await?;
    stream.collect::<Vec<_>>().await.into_iter().collect()
}

fn should_stop(
    cancellation: &SearchCancellationToken,
    policy: &SearchExecutionPolicy,
    index: usize,
) -> bool {
    index % policy.cancellation_check_interval.max(1) == 0 && cancellation.is_cancelled()
}
