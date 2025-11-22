//! Basic rdump library usage example
//!
//! Demonstrates common patterns for using the rdump library API.
//! Run with: `cargo run --example basic_search`

use anyhow::Result;
use rdump::{search_iter, Match, SearchOptions, SearchResult, SqlDialect};
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("rdump Library API Examples\n");

    example_extension_search()?;
    example_function_search()?;
    example_compound_query()?;
    example_custom_options()?;
    example_working_with_results()?;
    example_utility_patterns()?;

    println!("\nAll examples completed successfully!");
    Ok(())
}

/// Example 1: Search by file extension
fn example_extension_search() -> Result<()> {
    print_heading("Example 1: Extension Search");

    let results = run_query(
        "ext:rs",
        SearchOptions {
            root: fixture_root(),
            ..Default::default()
        },
    )?;
    report_results(&results, 5);

    Ok(())
}

/// Example 2: Search for functions by name
fn example_function_search() -> Result<()> {
    print_heading("Example 2: Function Search (func:main)");

    let results = run_query(
        "func:main",
        SearchOptions {
            root: fixture_root(),
            ..Default::default()
        },
    )?;
    if results.is_empty() {
        println!("No files with a main function.\n");
        return Ok(());
    }

    for result in &results {
        if result.is_whole_file_match() {
            println!("{} (whole file match)", result.path.display());
        } else {
            for m in &result.matches {
                println!(
                    "{}:{} - {}",
                    result.path.display(),
                    m.start_line,
                    m.first_line()
                );
            }
        }
    }
    println!();
    Ok(())
}

/// Example 3: Compound queries with AND/OR
fn example_compound_query() -> Result<()> {
    print_heading("Example 3: Compound Query");

    let query = "ext:rs & (func:test | func:main)";
    let results = run_query(
        query,
        SearchOptions {
            root: fixture_root(),
            ..Default::default()
        },
    )?;

    println!("Query: {query}");
    report_results(&results, 5);
    Ok(())
}

/// Example 4: Custom search options
fn example_custom_options() -> Result<()> {
    print_heading("Example 4: Custom Options");

    let options = SearchOptions {
        root: fixture_root(),                    // search the sample project
        presets: vec![],                         // no presets required
        no_ignore: true,                         // include gitignored files if any
        hidden: true,                            // include hidden files
        max_depth: Some(4),                      // limit traversal depth
        sql_dialect: Some(SqlDialect::Postgres), // demonstrate dialect override
    };

    let results = run_query("ext:rs | ext:sql", options)?;
    report_results(&results, 5);
    Ok(())
}

/// Example 5: Working with SearchResult and Match
fn example_working_with_results() -> Result<()> {
    print_heading("Example 5: Working with Results");

    let results = run_query(
        "func:main",
        SearchOptions {
            root: fixture_root(),
            ..Default::default()
        },
    )?;
    if results.is_empty() {
        println!("No functions found.\n");
        return Ok(());
    }

    let first = &results[0];
    println!("First file: {}", first.path.display());
    println!("  Whole file match? {}", first.is_whole_file_match());
    println!("  Matches: {}", first.match_count());
    println!("  Matched lines: {:?}\n", first.matched_lines());

    if let Some(m) = first.matches.first() {
        describe_match(m);
    }

    Ok(())
}

/// Utility examples: counting, collecting, filtering
fn example_utility_patterns() -> Result<()> {
    print_heading("Utility Patterns");

    let results = run_query(
        "func:main",
        SearchOptions {
            root: fixture_root(),
            ..Default::default()
        },
    )?;

    // Count functions per file
    for result in &results {
        println!(
            "{} => {} functions",
            result.path.display(),
            result.match_count()
        );
    }

    // Collect function signatures (first line of each match)
    let signatures: Vec<String> = results
        .iter()
        .flat_map(|r| r.matches.iter())
        .map(|m| m.first_line().to_string())
        .collect();
    println!("\nCollected {} function signatures", signatures.len());

    // Filter results programmatically (only files with more than one match)
    let multi_match: Vec<&SearchResult> = results.iter().filter(|r| r.match_count() > 1).collect();
    println!("Files with multiple matches: {}", multi_match.len());

    // Show streaming pattern with early termination
    let iter = search_iter(
        "ext:rs",
        SearchOptions {
            root: fixture_root(),
            ..Default::default()
        },
    )?;
    let first_two: Vec<_> = iter.take(2).filter_map(Result::ok).collect();
    println!("Streaming preview (first {} results):", first_two.len());
    for r in first_two {
        println!("  - {}", r.path.display());
    }

    println!();
    Ok(())
}

fn report_results(results: &[SearchResult], sample: usize) {
    println!("Found {} results", results.len());
    for result in results.iter().take(sample) {
        println!(
            "  - {} ({} matches)",
            result.path.display(),
            result.match_count()
        );
    }
    if results.len() > sample {
        println!("  ... and {} more", results.len() - sample);
    }
    println!();
}

fn describe_match(m: &Match) {
    println!("Example match:");
    println!("  Lines: {}-{}", m.start_line, m.end_line);
    println!("  Bytes: {:?}", m.byte_range);
    println!("  First line: {}", m.first_line());
    println!();
}

fn print_heading(title: &str) {
    println!("{}", title);
    println!("{}", "-".repeat(title.len()));
}

fn run_query(query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
    // Use the iterator form so we can gracefully skip per-file errors (e.g., binary files).
    let iter = search_iter(query, options)?;
    Ok(iter.filter_map(Result::ok).collect())
}

fn fixture_root() -> PathBuf {
    PathBuf::from("tests/fixtures/rust_project")
}
