use clap::Parser;
use rdump::commands::search::perform_search;
use rdump::{
    search, search_iter, Cli, ColorChoice, Commands, Format, LangAction, LangArgs, Match,
    PresetAction, PresetArgs, SearchArgs, SearchOptions, SearchResult, SqlDialect, SqlDialectFlag,
};
use std::path::PathBuf;
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
    assert_eq!(iter.remaining(), lower);
    if let Some(max) = upper {
        assert!(lower <= max);
    }
    let results = search("ext:rs", options).expect("search reachable");

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
    };

    assert!(!sample_result.is_whole_file_match());
    assert_eq!(sample_result.match_count(), 1);
    let _ = results; // ensure collection is usable

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
    let _ = perform_search(&args).expect("perform_search stays available");

    let _format = Format::Hunks;
    let _color = ColorChoice::Auto;
    let _lang_args = LangArgs {
        action: Some(LangAction::List),
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
    use rdump::{search_all_async, search_async};
    let _ = search_async;
    let _ = search_all_async;
}
