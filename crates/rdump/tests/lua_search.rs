use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_lua() {
    let dir = setup_fixture("lua_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));
}

#[test]
fn test_func_predicate_lua() {
    let dir = setup_fixture("lua_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));
}

#[test]
fn test_import_predicate_lua() {
    let dir = setup_fixture("lua_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:require")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));
}

#[test]
fn test_call_predicate_lua() {
    let dir = setup_fixture("lua_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));
}

#[test]
fn test_str_predicate_lua() {
    let dir = setup_fixture("lua_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_lua_func_and_call() {
    let dir = setup_fixture("lua_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:greet & call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));
}

#[test]
fn test_lua_or_operations() {
    let dir = setup_fixture("lua_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add | func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.lua"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_lua_custom_table() {
    let dir = setup_custom_project(&[(
        "tables.lua",
        r#"
local Person = {}
Person.__index = Person

function Person.new(name, age)
    local self = setmetatable({}, Person)
    self.name = name
    self.age = age
    return self
end

function Person:greet()
    return "Hello, " .. self.name
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:new & func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("tables.lua"));
}

#[test]
fn test_lua_custom_module() {
    let dir = setup_custom_project(&[(
        "utils.lua",
        r#"
local M = {}

function M.split(str, sep)
    local result = {}
    for match in (str .. sep):gmatch("(.-)" .. sep) do
        table.insert(result, match)
    end
    return result
end

function M.join(arr, sep)
    return table.concat(arr, sep)
end

return M
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:split & func:join")
        .assert()
        .success()
        .stdout(predicate::str::contains("utils.lua"));
}

#[test]
fn test_lua_custom_iterator() {
    let dir = setup_custom_project(&[(
        "iterators.lua",
        r#"
function range(from, to, step)
    step = step or 1
    return function(_, last)
        local next = last + step
        if next <= to then
            return next
        end
    end, nil, from - step
end

function map(tbl, func)
    local result = {}
    for i, v in ipairs(tbl) do
        result[i] = func(v)
    end
    return result
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:range | func:map")
        .assert()
        .success()
        .stdout(predicate::str::contains("iterators.lua"));
}

#[test]
fn test_lua_custom_coroutine() {
    let dir = setup_custom_project(&[(
        "coroutines.lua",
        r#"
function producer()
    return coroutine.create(function()
        for i = 1, 5 do
            coroutine.yield(i)
        end
    end)
end

function consumer(prod)
    while true do
        local status, value = coroutine.resume(prod)
        if not status then break end
        print(value)
    end
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:producer & func:consumer")
        .assert()
        .success()
        .stdout(predicate::str::contains("coroutines.lua"));
}

#[test]
fn test_lua_custom_metatables() {
    let dir = setup_custom_project(&[(
        "metatables.lua",
        r#"
local Vector = {}
Vector.__index = Vector

function Vector.new(x, y)
    return setmetatable({x = x, y = y}, Vector)
end

function Vector.__add(a, b)
    return Vector.new(a.x + b.x, a.y + b.y)
end

function Vector.__tostring(v)
    return "(" .. v.x .. ", " .. v.y .. ")"
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:new & ext:lua")
        .assert()
        .success()
        .stdout(predicate::str::contains("metatables.lua"));
}

#[test]
fn test_lua_custom_error_handling() {
    let dir = setup_custom_project(&[(
        "errors.lua",
        r#"
function safe_divide(a, b)
    if b == 0 then
        error("Division by zero")
    end
    return a / b
end

function try_divide(a, b)
    local status, result = pcall(safe_divide, a, b)
    if status then
        return result
    else
        return nil, result
    end
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:safe_divide & func:try_divide")
        .assert()
        .success()
        .stdout(predicate::str::contains("errors.lua"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_lua_format_paths() {
    let dir = setup_fixture("lua_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("func:greet")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.lua"));
}

#[test]
fn test_lua_format_markdown() {
    let dir = setup_fixture("lua_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("```lua"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_lua_not_found() {
    let dir = setup_fixture("lua_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:nonexistent & ext:lua")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_lua_ext_filter() {
    let dir = setup_fixture("lua_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:lua")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
