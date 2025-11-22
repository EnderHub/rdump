//! Streaming search example for rdump library
//!
//! Demonstrates memory-efficient patterns for processing large codebases
//! using the `search_iter` API.
//! Run with: `cargo run --example streaming_search`

use anyhow::Result;
use rayon::prelude::*;
use rdump::{search_iter, SearchOptions, SearchResult};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<()> {
    println!("rdump Streaming API Examples\n");

    example_basic_streaming()?;
    example_early_termination()?;
    example_skip_errors()?;
    example_progress_reporting()?;
    example_parallel_processing()?;
    example_memory_efficient_aggregation()?;
    example_additional_patterns()?;

    println!("\nAll streaming examples completed!");
    Ok(())
}

fn example_basic_streaming() -> Result<()> {
    heading("Example 1: Basic Streaming");
    let iter = search_iter("ext:rs", default_options())?;

    let total = iter.remaining();
    println!("Processing {} files lazily...", total);
    let mut seen = 0;
    for result in iter {
        let r = result?;
        seen += 1;
        if seen <= 3 {
            println!("  - {} ({} bytes)", r.path.display(), r.content.len());
        }
    }
    if seen > 3 {
        println!("  ...and {} more", seen - 3);
    }
    println!();
    Ok(())
}

fn example_early_termination() -> Result<()> {
    heading("Example 2: Early Termination (.take())");
    let iter = search_iter("ext:rs", default_options())?;
    let total = iter.remaining();
    let first_two: Vec<SearchResult> = iter.take(2).filter_map(Result::ok).collect();

    for r in &first_two {
        println!("  - {}", r.path.display());
    }
    println!(
        "Processed {} of {} results (rest never read from disk)\n",
        first_two.len(),
        total
    );
    Ok(())
}

fn example_skip_errors() -> Result<()> {
    heading("Example 3: Error Handling (skip/collect)");
    let iter = search_iter("ext:rs", default_options())?;

    let mut successes = Vec::new();
    let mut errors = Vec::new();
    for res in iter {
        match res {
            Ok(r) => successes.push(r),
            Err(e) => errors.push(e.to_string()),
        }
    }

    println!("Successes: {}", successes.len());
    if !errors.is_empty() {
        println!("Errors:");
        for e in &errors {
            println!("  - {e}");
        }
    } else {
        println!("No errors encountered (still demonstrates pattern).");
    }
    println!();
    Ok(())
}

fn example_progress_reporting() -> Result<()> {
    heading("Example 4: Progress Reporting");
    let iter = search_iter("ext:rs", default_options())?;
    let total = iter.remaining().max(1); // avoid divide-by-zero

    let mut processed = 0usize;
    let start = Instant::now();
    for res in iter {
        if res.is_ok() {
            processed += 1;
            if processed % 2 == 0 || processed == total {
                let pct = (processed as f64 / total as f64) * 100.0;
                println!("  processed {processed}/{total} ({pct:.1}%)");
            }
        }
    }
    println!("Completed in {:.2?}\n", start.elapsed());
    Ok(())
}

fn example_parallel_processing() -> Result<()> {
    heading("Example 5: Parallel Processing (rayon)");
    let iter = search_iter("ext:rs", default_options())?;
    let results: Vec<_> = iter.filter_map(Result::ok).collect();

    let total_matches: usize = results.par_iter().map(|r| r.match_count()).sum();
    println!(
        "Processed {} files in parallel; total matches: {}",
        results.len(),
        total_matches
    );
    println!();
    Ok(())
}

fn example_memory_efficient_aggregation() -> Result<()> {
    heading("Example 6: Memory-Efficient Aggregation");
    let iter = search_iter("ext:rs", default_options())?;

    let mut file_count = 0usize;
    let mut byte_total = 0usize;
    for res in iter {
        let r = res?;
        file_count += 1;
        byte_total += r.content.len();
    }

    println!("Files: {}", file_count);
    println!("Total bytes (approx): {}", byte_total);
    println!();
    Ok(())
}

fn example_additional_patterns() -> Result<()> {
    heading("Additional Patterns");
    let iter = search_iter("ext:rs", default_options())?;

    // Find first file with more than one match
    let first_multi = iter.filter_map(Result::ok).find(|r| r.match_count() > 1);
    if let Some(r) = first_multi {
        println!(
            "First with >1 match: {} ({} matches)",
            r.path.display(),
            r.match_count()
        );
    } else {
        println!("No file with multiple matches found.");
    }

    // Batch processing (windowed)
    let iter = search_iter("ext:rs", default_options())?;
    let mut batch = Vec::with_capacity(2);
    for res in iter {
        if let Ok(r) = res {
            batch.push(r);
            if batch.len() == 2 {
                println!("Batch of 2:");
                for item in &batch {
                    println!("  {}", item.path.display());
                }
                batch.clear();
            }
        }
    }
    if !batch.is_empty() {
        println!("Final partial batch of {} items", batch.len());
    }

    // Two-phase processing: collect light metadata, defer content-heavy work
    let iter = search_iter("ext:rs", default_options())?;
    let light: Vec<PathBuf> = iter
        .filter_map(Result::ok)
        .map(|r| r.path.clone())
        .collect();
    println!("\nCollected {} paths for phase two.\n", light.len());

    Ok(())
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

fn heading(title: &str) {
    println!("{title}");
    println!("{}", "-".repeat(title.len()));
}
