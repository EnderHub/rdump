//! Async search examples for rdump.
//!
//! Run with: `cargo run --features async --example async_search`

#![cfg(feature = "async")]

use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use rdump::{search_async, SearchOptions, SearchResult};
use std::path::PathBuf;
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    println!("rdump Async API Examples\n");

    example_basic_async().await?;
    example_early_termination().await?;
    example_concurrent_streams().await?;
    example_with_other_async().await?;
    example_error_handling().await?;

    println!("\nAll async examples completed!");
    Ok(())
}

/// Example 1: Basic async streaming
async fn example_basic_async() -> Result<()> {
    heading("Example 1: Basic Async Streaming");

    let mut stream = search_async("ext:rs", default_options()).await?;
    let mut count = 0;
    while let Some(res) = stream.next().await {
        let r = res?;
        count += 1;
        if count <= 3 {
            println!("  - {} ({} bytes)", r.path.display(), r.content.len());
        }
    }
    if count > 3 {
        println!("  ... and {} more", count - 3);
    }
    println!();
    Ok(())
}

/// Example 2: Early termination with .take()
async fn example_early_termination() -> Result<()> {
    heading("Example 2: Early Termination");

    let start = Instant::now();
    let stream = search_async("ext:rs", default_options()).await?;
    let first_two: Vec<SearchResult> = stream.take(2).try_collect().await?;

    println!("Took {} results in {:?}", first_two.len(), start.elapsed());
    for r in &first_two {
        println!("  - {}", r.path.display());
    }
    println!();
    Ok(())
}

/// Example 3: Concurrent stream processing
async fn example_concurrent_streams() -> Result<()> {
    heading("Example 3: Concurrent Streams");

    let opts = default_options();
    let (rust, funcs) = tokio::try_join!(
        collect_paths("ext:rs", opts.clone()),
        collect_paths("func:main", opts.clone())
    )?;

    println!("Rust files: {}", rust.len());
    println!("Files with main(): {}", funcs.len());
    println!();
    Ok(())
}

/// Example 4: Integrate with other async work (simulate I/O + delay)
async fn example_with_other_async() -> Result<()> {
    heading("Example 4: With Other Async Work");

    let mut stream = search_async("ext:rs", default_options()).await?;
    let out_dir = tempfile::tempdir()?;

    while let Some(res) = stream.next().await {
        let r = res?;
        let out_file = out_dir.path().join(
            r.path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );
        tokio::fs::write(&out_file, &r.content).await?;
        // Simulate additional async work
        sleep(Duration::from_millis(5)).await;
    }

    println!("Wrote copies to {}", out_dir.path().display());
    println!();
    Ok(())
}

/// Example 5: Error handling in async context
async fn example_error_handling() -> Result<()> {
    heading("Example 5: Error Handling");

    match search_async("invalid((query", default_options()).await {
        Ok(mut stream) => {
            if let Some(err) = stream.next().await {
                eprintln!("Unexpected: {:?}", err);
            }
        }
        Err(e) => {
            println!("Caught query error early: {e}");
        }
    }

    println!();
    Ok(())
}

fn heading(title: &str) {
    println!("{title}");
    println!("{}", "-".repeat(title.len()));
}

fn default_options() -> SearchOptions {
    SearchOptions {
        root: fixture_root(),
        ..Default::default()
    }
}

fn fixture_root() -> PathBuf {
    PathBuf::from("tests/fixtures/rust_project")
}

async fn collect_paths(query: &str, options: SearchOptions) -> Result<Vec<PathBuf>> {
    let mut stream = search_async(query, options).await?;
    let mut paths = Vec::new();
    while let Some(item) = stream.next().await {
        let r = item?;
        paths.push(r.path);
    }
    Ok(paths)
}
