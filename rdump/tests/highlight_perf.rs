use rdump::{
    formatter, Format, Match, SearchReport, SearchResult, SearchResultMetadata, SearchStats,
};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

fn sample_report() -> SearchReport {
    SearchReport {
        results: vec![SearchResult {
            path: PathBuf::from("src/main.rs"),
            matches: vec![Match {
                start_line: 1,
                end_line: 1,
                start_column: 3,
                end_column: 7,
                byte_range: 3..7,
                text: "main".to_string(),
            }],
            content: "fn main() {\n    println!(\"hi\");\n}\n".to_string(),
            content_state: rdump::ContentState::Loaded,
            diagnostics: vec![],
            metadata: SearchResultMetadata::default(),
        }],
        stats: SearchStats::default(),
        diagnostics: vec![],
    }
}

#[test]
#[ignore = "perf harness"]
fn highlight_cold_vs_warm_report_render() {
    let report = sample_report();

    let cold_started = Instant::now();
    let mut cold = Vec::new();
    formatter::print_report_output(
        &mut cold,
        &report,
        &Format::Hunks,
        false,
        false,
        true,
        0,
        true,
        rdump::TimeFormat::Local,
    )
    .unwrap();
    let cold_elapsed = cold_started.elapsed().as_millis();

    let warm_started = Instant::now();
    let mut warm = Vec::new();
    formatter::print_report_output(
        &mut warm,
        &report,
        &Format::Hunks,
        false,
        false,
        true,
        0,
        true,
        rdump::TimeFormat::Local,
    )
    .unwrap();
    let warm_elapsed = warm_started.elapsed().as_millis();

    std::io::stderr()
        .write_all(
            format!("highlight_cold_ms={cold_elapsed} highlight_warm_ms={warm_elapsed}\n")
                .as_bytes(),
        )
        .unwrap();

    if let Ok(value) = std::env::var("RDUMP_PERF_HIGHLIGHT_WARM_MAX_MS") {
        let threshold = value.parse::<u128>().expect("threshold env should parse");
        assert!(
            warm_elapsed <= threshold,
            "RDUMP_PERF_HIGHLIGHT_WARM_MAX_MS exceeded: elapsed={warm_elapsed} threshold={threshold}"
        );
    }
}
