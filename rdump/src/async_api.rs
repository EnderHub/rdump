//! Async API for rdump search.
//!
//! Enabled with the `async` feature. Bridges the synchronous iterator to an
//! async stream using `tokio::task::spawn_blocking` and a bounded channel for
//! backpressure.

use crate::{search_iter, SearchOptions, SearchResult};
use anyhow::Result;
use futures::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

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
pub async fn search_async(
    query: &str,
    options: SearchOptions,
) -> Result<impl Stream<Item = Result<SearchResult>>> {
    let query = query.to_string();
    let (tx, rx) = mpsc::channel(100);

    tokio::task::spawn_blocking(move || {
        let iter = match search_iter(&query, options) {
            Ok(iter) => iter,
            Err(e) => {
                let _ = tx.blocking_send(Err(e));
                return;
            }
        };

        for result in iter {
            if tx.blocking_send(result).is_err() {
                break;
            }
        }
    });

    Ok(ReceiverStream::new(rx))
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
