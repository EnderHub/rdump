use predicates::prelude::*;

mod common;
use common::setup_fixture;

#[test]
fn test_sql_generic_def_and_import() {
    let dir = setup_fixture("sql_generic");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("def:users & ext:sql");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("schema.sql"))
        .stdout(predicate::str::contains("CREATE TABLE users"))
        .stdout(predicate::str::contains("select count(*").not());

    let mut import_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    import_cmd.current_dir(dir.path());
    import_cmd.arg("search").arg("import:users & ext:sql");
    import_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("SELECT count(*) FROM users"));
}

#[test]
fn test_sql_postgres_function_and_call() {
    let dir = setup_fixture("sql_postgres");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("func:calculate_total & ext:sql");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("calculate_total"))
        .stdout(predicate::str::contains("schema.sql"));

    let mut call_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    call_cmd.current_dir(dir.path());
    call_cmd.arg("search").arg("call:calculate_total & ext:sql");
    call_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("SELECT calculate_total"));
}

#[test]
fn test_sql_mysql_dialect_flag_and_call() {
    let dir = setup_fixture("sql_mysql");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search")
        .arg("call:bump_count & ext:sql")
        .arg("--dialect")
        .arg("mysql");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("bump_count"))
        .stdout(predicate::str::contains("SELECT bump_count"));
}

#[test]
fn test_sql_sqlite_comment_and_string() {
    let dir = setup_fixture("sql_sqlite");

    let mut comment_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    comment_cmd.current_dir(dir.path());
    comment_cmd.arg("search").arg("comment:note & ext:sql");
    comment_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("schema.sql"));

    let mut str_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    str_cmd.current_dir(dir.path());
    str_cmd.arg("search").arg("str:sqlite-user & ext:sql");
    str_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("sqlite-user"));
}
