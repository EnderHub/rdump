use predicates::prelude::*;
mod common;
use common::setup_test_project;

#[test]
fn test_def_finds_javascript_class() {
    let dir = setup_test_project();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("def:OldLogger");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("logger.js"))
        .stdout(predicate::str::contains("export class OldLogger"))
        .stdout(predicate::str::contains("log_utils.ts").not());
}

#[test]
fn test_def_finds_typescript_interface_and_type() {
    let dir = setup_test_project();
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path())
        .arg("search")
        .arg("def:ILog | def:LogLevel");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("log_utils.ts"))
        .stdout(predicate::str::contains("interface ILog"))
        .stdout(predicate::str::contains(
            r#"type LogLevel = "info" | "warn" | "error";"#,
        ));
}

#[test]
fn test_func_finds_typescript_function() {
    let dir = setup_test_project();
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path())
        .arg("search")
        .arg("func:createLog");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("log_utils.ts"))
        .stdout(predicate::str::contains("export function createLog"));
}

#[test]
fn test_import_finds_typescript_import() {
    let dir = setup_test_project();
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path())
        .arg("search")
        .arg("import:path & ext:ts");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("log_utils.ts"))
        .stdout(predicate::str::contains("import * as path from 'path';"));
}

#[test]
fn test_call_predicate_javascript() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:log & ext:js")
        .assert()
        .success()
        .stdout(predicate::str::contains("logger.js"))
        .stdout(predicate::str::contains("logger.log(\"init\");"));
}

#[test]
fn test_call_predicate_typescript() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:log & ext:ts")
        .assert()
        .success()
        .stdout(predicate::str::contains("log_utils.ts"))
        .stdout(predicate::str::contains("console.log(newLog);"));
}

#[test]
fn test_comment_predicate_typescript() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("comment:REVIEW")
        .assert()
        .success()
        .stdout(predicate::str::contains("log_utils.ts"));
}

#[test]
fn test_str_predicate_javascript() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:logging:")
        .assert()
        .success()
        .stdout(predicate::str::contains("logger.js"));
}

#[test]
fn test_interface_and_type_predicates_typescript() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("interface:ILog & type:LogLevel")
        .assert()
        .success()
        .stdout(predicate::str::contains("log_utils.ts"));
}

#[test]
fn test_def_not_found_js_ts() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:NonExistent & (ext:js | ext:ts)")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_js_custom_class() {
    let dir = common::setup_custom_project(&[(
        "person.js",
        r#"
class Person {
    constructor(name, age) {
        this.name = name;
        this.age = age;
    }

    greet() {
        return `Hello, ${this.name}!`;
    }

    isAdult() {
        return this.age >= 18;
    }
}

module.exports = Person;
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Person & func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("person.js"));
}

#[test]
fn test_js_custom_arrow_functions() {
    let dir = common::setup_custom_project(&[(
        "utils.js",
        r#"
const add = (a, b) => a + b;

const multiply = (a, b) => {
    return a * b;
};

const greet = name => `Hello, ${name}`;

const compose = (...fns) => x => fns.reduceRight((acc, fn) => fn(acc), x);

module.exports = { add, multiply, greet, compose };
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello & ext:js")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.js"));
}

#[test]
fn test_js_custom_async_await() {
    let dir = common::setup_custom_project(&[(
        "api.js",
        r#"
async function fetchData(url) {
    const response = await fetch(url);
    return response.json();
}

async function processData(id) {
    try {
        const data = await fetchData(`/api/${id}`);
        return data;
    } catch (error) {
        console.error(error);
        throw error;
    }
}

module.exports = { fetchData, processData };
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:fetchData & func:processData")
        .assert()
        .success()
        .stdout(predicate::str::contains("api.js"));
}

#[test]
fn test_ts_custom_interface() {
    let dir = common::setup_custom_project(&[(
        "types.ts",
        r#"
interface User {
    id: number;
    name: string;
    email: string;
}

interface CreateUserInput {
    name: string;
    email: string;
}

function createUser(input: CreateUserInput): User {
    return {
        id: Date.now(),
        ...input,
    };
}

export { User, CreateUserInput, createUser };
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("interface:User & func:createUser")
        .assert()
        .success()
        .stdout(predicate::str::contains("types.ts"));
}

#[test]
fn test_ts_custom_generic() {
    let dir = common::setup_custom_project(&[(
        "generic.ts",
        r#"
class Stack<T> {
    private items: T[] = [];

    push(item: T): void {
        this.items.push(item);
    }

    pop(): T | undefined {
        return this.items.pop();
    }

    peek(): T | undefined {
        return this.items[this.items.length - 1];
    }

    isEmpty(): boolean {
        return this.items.length === 0;
    }
}

export default Stack;
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Stack & func:push")
        .assert()
        .success()
        .stdout(predicate::str::contains("generic.ts"));
}

#[test]
fn test_ts_custom_enum() {
    let dir = common::setup_custom_project(&[(
        "enums.ts",
        r#"
enum Direction {
    Up = "UP",
    Down = "DOWN",
    Left = "LEFT",
    Right = "RIGHT",
}

enum Status {
    Pending,
    Active,
    Completed,
}

function move(direction: Direction): void {
    console.log(`Moving ${direction}`);
}

export { Direction, Status, move };
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("enum:Direction | enum:Status")
        .assert()
        .success()
        .stdout(predicate::str::contains("enums.ts"));
}

#[test]
fn test_ts_custom_type_alias() {
    let dir = common::setup_custom_project(&[(
        "aliases.ts",
        r#"
type ID = string | number;

type Point = {
    x: number;
    y: number;
};

type Callback<T> = (value: T) => void;

type Result<T, E> = { ok: true; value: T } | { ok: false; error: E };

function processResult<T, E>(result: Result<T, E>): void {
    if (result.ok) {
        console.log(result.value);
    } else {
        console.error(result.error);
    }
}

export { ID, Point, Callback, Result, processResult };
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("type:Point & func:processResult")
        .assert()
        .success()
        .stdout(predicate::str::contains("aliases.ts"));
}

#[test]
fn test_js_custom_module() {
    let dir = common::setup_custom_project(&[(
        "module.js",
        r#"
const CONFIG = {
    apiUrl: 'https://api.example.com',
    timeout: 5000,
};

function get(endpoint) {
    return fetch(`${CONFIG.apiUrl}${endpoint}`);
}

function post(endpoint, data) {
    return fetch(`${CONFIG.apiUrl}${endpoint}`, {
        method: 'POST',
        body: JSON.stringify(data),
    });
}

module.exports = { CONFIG, get, post };
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:get & func:post")
        .assert()
        .success()
        .stdout(predicate::str::contains("module.js"));
}

#[test]
fn test_ts_custom_decorator() {
    let dir = common::setup_custom_project(&[(
        "decorators.ts",
        r#"
function log(target: any, key: string, descriptor: PropertyDescriptor) {
    const original = descriptor.value;
    descriptor.value = function(...args: any[]) {
        console.log(`Calling ${key} with`, args);
        return original.apply(this, args);
    };
    return descriptor;
}

class Calculator {
    @log
    add(a: number, b: number): number {
        return a + b;
    }

    @log
    multiply(a: number, b: number): number {
        return a * b;
    }
}

export { log, Calculator };
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:Calculator & func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("decorators.ts"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_js_format_paths() {
    let dir = setup_test_project();
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("class:OldLogger")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("logger.js"));
}

#[test]
fn test_ts_format_markdown() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("func:createLog")
        .assert()
        .success()
        .stdout(predicate::str::contains("```ts"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_js_ts_complex_query() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("(class:OldLogger | interface:ILog) & ext:js")
        .assert()
        .success()
        .stdout(predicate::str::contains("logger.js"));
}

#[test]
fn test_js_ts_negation() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:createLog & !class:OldLogger")
        .assert()
        .success()
        .stdout(predicate::str::contains("log_utils.ts"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_js_ext_filter() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("class:. & ext:js")
        .assert()
        .success()
        .stdout(predicate::str::contains(".ts").not());
}

#[test]
fn test_ts_ext_filter() {
    let dir = setup_test_project();
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("interface:. & ext:ts")
        .assert()
        .success()
        .stdout(predicate::str::contains(".js").not());
}
