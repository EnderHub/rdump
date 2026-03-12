use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_bash() {
    let dir = setup_fixture("bash_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.sh"));
}

#[test]
fn test_func_predicate_bash() {
    let dir = setup_fixture("bash_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.sh"));
}

#[test]
fn test_import_predicate_bash() {
    let dir = setup_fixture("bash_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:source")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.sh"));
}

#[test]
fn test_call_predicate_bash() {
    let dir = setup_fixture("bash_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.sh"));
}

#[test]
fn test_str_predicate_bash() {
    let dir = setup_fixture("bash_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.sh"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_bash_func_and_call() {
    let dir = setup_fixture("bash_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:greet & call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.sh"));
}

#[test]
fn test_bash_or_operations() {
    let dir = setup_fixture("bash_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add | func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.sh"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_bash_custom_variables() {
    let dir = setup_custom_project(&[(
        "vars.sh",
        r#"#!/bin/bash

# Configuration variables
CONFIG_DIR="/etc/myapp"
LOG_FILE="/var/log/myapp.log"

function init_config() {
    mkdir -p "$CONFIG_DIR"
    touch "$LOG_FILE"
}

function log_message() {
    echo "$(date): $1" >> "$LOG_FILE"
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:init_config & func:log_message")
        .assert()
        .success()
        .stdout(predicate::str::contains("vars.sh"));
}

#[test]
fn test_bash_custom_array() {
    let dir = setup_custom_project(&[(
        "arrays.sh",
        r#"#!/bin/bash

declare -a COLORS=("red" "green" "blue")

function print_colors() {
    for color in "${COLORS[@]}"; do
        echo "$color"
    done
}

function add_color() {
    COLORS+=("$1")
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:print_colors | func:add_color")
        .assert()
        .success()
        .stdout(predicate::str::contains("arrays.sh"));
}

#[test]
fn test_bash_custom_conditionals() {
    let dir = setup_custom_project(&[(
        "conditionals.sh",
        r#"#!/bin/bash

function check_file() {
    if [[ -f "$1" ]]; then
        echo "File exists"
        return 0
    else
        echo "File not found"
        return 1
    fi
}

function is_root() {
    if [[ $EUID -eq 0 ]]; then
        return 0
    else
        return 1
    fi
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:check_file & func:is_root")
        .assert()
        .success()
        .stdout(predicate::str::contains("conditionals.sh"));
}

#[test]
fn test_bash_custom_case_statement() {
    let dir = setup_custom_project(&[(
        "cases.sh",
        r#"#!/bin/bash

function handle_option() {
    case "$1" in
        start)
            echo "Starting..."
            ;;
        stop)
            echo "Stopping..."
            ;;
        restart)
            echo "Restarting..."
            ;;
        *)
            echo "Unknown option"
            ;;
    esac
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:handle_option")
        .assert()
        .success()
        .stdout(predicate::str::contains("cases.sh"));
}

#[test]
fn test_bash_custom_trap() {
    let dir = setup_custom_project(&[(
        "traps.sh",
        r#"#!/bin/bash

function cleanup() {
    echo "Cleaning up..."
    rm -f /tmp/myapp.pid
}

trap cleanup EXIT

function main() {
    echo $$ > /tmp/myapp.pid
    echo "Running..."
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:cleanup & func:main")
        .assert()
        .success()
        .stdout(predicate::str::contains("traps.sh"));
}

#[test]
fn test_bash_custom_here_doc() {
    let dir = setup_custom_project(&[(
        "heredoc.sh",
        r#"#!/bin/bash

function print_help() {
    cat << EOF
Usage: $(basename $0) [options]
Options:
    -h    Show help
    -v    Verbose mode
EOF
}

function write_config() {
    cat > config.txt << 'EOF'
key=value
name=test
EOF
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:print_help | func:write_config")
        .assert()
        .success()
        .stdout(predicate::str::contains("heredoc.sh"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_bash_format_paths() {
    let dir = setup_fixture("bash_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("func:greet")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.sh"));
}

#[test]
fn test_bash_format_markdown() {
    let dir = setup_fixture("bash_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("```sh"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_bash_not_found() {
    let dir = setup_fixture("bash_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:nonexistent & ext:sh")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_bash_ext_filter() {
    let dir = setup_fixture("bash_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:sh")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
