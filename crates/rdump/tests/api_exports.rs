use clap::Parser;
use rdump::commands::search::search_request_from_args;
use rdump::{
    capability_metadata, execute_search_request,
    execute_search_request_with_runtime_and_cancellation,
    execute_search_request_with_runtime_and_progress, repo_language_inventory_with_runtime, search,
    search_iter, search_options_from_request, Cli, ColorChoice, Commands, ConfigAction, ConfigArgs,
    Format, LangAction, LangArgs, Match, PresetAction, PresetArgs, QueryAction, QueryArgs,
    RealFsSearchBackend, SearchArgs, SearchCancellationToken, SearchOptions, SearchResult,
    SearchResultMetadata, SearchRuntime, SqlDialect, SqlDialectFlag,
};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn library_exports_accessible_from_crate_root() {
    let tmp = tempdir().unwrap();
    std::fs::write(tmp.path().join("example.rs"), "fn main() {}\n").unwrap();

    let options = SearchOptions {
        root: tmp.path().to_path_buf(),
        ..Default::default()
    };

    let iter = search_iter("ext:rs", options.clone()).expect("search_iter reachable");
    let (lower, upper) = iter.size_hint();
    assert!(lower <= iter.remaining());
    if let Some(max) = upper {
        assert!(iter.remaining() <= max);
    }
    let results = search("ext:rs", options).expect("search reachable");
    let runtime = SearchRuntime::with_backend(Arc::new(RealFsSearchBackend));
    let runtime_results = runtime
        .search(
            "ext:rs",
            &SearchOptions {
                root: tmp.path().to_path_buf(),
                ..Default::default()
            },
        )
        .expect("runtime search reachable");
    assert_eq!(runtime_results.len(), results.len());
    let inventory = repo_language_inventory_with_runtime(
        &runtime,
        &SearchOptions {
            root: tmp.path().to_path_buf(),
            ..Default::default()
        },
    );
    assert!(inventory.iter().any(|entry| entry.extension == "rs"));

    let sample_match = Match {
        start_line: 1,
        end_line: 1,
        start_column: 0,
        end_column: 2,
        byte_range: 0..2,
        text: "fn".to_string(),
    };
    let sample_result = SearchResult {
        path: PathBuf::from("example.rs"),
        matches: vec![sample_match],
        content: "fn main() {}".to_string(),
        content_state: rdump::ContentState::Loaded,
        diagnostics: vec![],
        metadata: SearchResultMetadata::default(),
    };

    assert!(!sample_result.is_whole_file_match());
    assert_eq!(sample_result.match_count(), 1);
    let _ = results; // ensure collection is usable
    let mut writer = Vec::new();
    rdump::formatter::print_output_with_backend(
        &RealFsSearchBackend,
        &mut writer,
        &[(tmp.path().join("example.rs"), vec![])],
        &Format::Paths,
        false,
        false,
        false,
        0,
        rdump::TimeFormat::Local,
    )
    .expect("backend-aware raw formatter reachable");
    let mut path_writer = Vec::new();
    rdump::formatter::print_path_output_with_backend(
        &RealFsSearchBackend,
        &mut path_writer,
        &[tmp.path().join("example.rs")],
        &Format::Paths,
        rdump::TimeFormat::Local,
    )
    .expect("backend-aware report path formatter reachable");

    let dialects = [
        SqlDialect::Generic,
        SqlDialect::Postgres,
        SqlDialect::Mysql,
        SqlDialect::Sqlite,
    ];
    assert_eq!(dialects[1], SqlDialect::Postgres);
}

#[test]
fn cli_exports_and_flags_remain_public() {
    let cli = Cli::parse_from(["rdump", "search", "ext:rs"]);
    match cli.command {
        Commands::Search(search_args) => {
            assert_eq!(search_args.query.as_deref(), Some("ext:rs"));
        }
        _ => panic!("expected search subcommand"),
    }

    let tmp = tempdir().unwrap();
    std::fs::write(tmp.path().join("example.rs"), "fn main() {}\n").unwrap();

    let args =
        SearchArgs::try_parse_from(["rdump", "ext:rs", "--root", tmp.path().to_str().unwrap()])
            .expect("SearchArgs parseable");
    let request = search_request_from_args(&args);
    let _ = execute_search_request(&request).expect("request execution stays available");
    let request = rdump::contracts::SearchRequest {
        query: "ext:rs".to_string(),
        root: Some(tmp.path().to_string_lossy().to_string()),
        ..Default::default()
    };
    let _ = search_options_from_request(&request);
    let _ = execute_search_request(&request).expect("request execution stays available");
    let runtime = SearchRuntime::with_backend(Arc::new(RealFsSearchBackend));
    let _ = execute_search_request_with_runtime_and_progress(runtime.clone(), &request, |_| {})
        .expect("runtime request execution with progress stays available");
    let _ = execute_search_request_with_runtime_and_cancellation(
        runtime,
        &request,
        Some(SearchCancellationToken::new()),
        "api-exports",
        |_| {},
    )
    .expect("runtime request execution with cancellation stays available");
    let _ = capability_metadata();

    let _format = Format::Hunks;
    let _color = ColorChoice::Auto;
    let _lang_args = LangArgs {
        action: Some(LangAction::List),
    };
    let _query_args = QueryArgs {
        action: QueryAction::Reference { json: false },
    };
    let _config_args = ConfigArgs {
        action: ConfigAction::Show,
    };
    let _preset_args = PresetArgs {
        action: PresetAction::List,
    };

    let dialect: SqlDialect = SqlDialectFlag::Mysql.into();
    assert_eq!(dialect, SqlDialect::Mysql);
}

#[cfg(feature = "async")]
#[test]
fn async_exports_available_when_feature_enabled() {
    use rdump::{
        search_all_async, search_all_async_with_runtime, search_async, search_async_with_runtime,
        search_async_with_runtime_and_progress,
    };
    let _ = search_async;
    let _ = search_all_async;
    let runtime = SearchRuntime::with_backend(Arc::new(RealFsSearchBackend));
    let options = SearchOptions::default();
    let _unused_stream = search_async_with_runtime(runtime.clone(), "ext:rs", options.clone());
    let _unused_collect = search_all_async_with_runtime(runtime.clone(), "ext:rs", options.clone());
    let _unused_progress_stream =
        search_async_with_runtime_and_progress(runtime, "ext:rs", options, |_| {});
}
