use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_zig() {
    let dir = setup_fixture("zig_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.zig"));
}

#[test]
fn test_func_predicate_zig() {
    let dir = setup_fixture("zig_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.zig"));
}

#[test]
fn test_import_predicate_zig() {
    let dir = setup_fixture("zig_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:@import")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.zig"));
}

#[test]
fn test_call_predicate_zig() {
    let dir = setup_fixture("zig_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.zig"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_zig_func_and_call() {
    let dir = setup_fixture("zig_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:greet & call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.zig"));
}

#[test]
fn test_zig_or_operations() {
    let dir = setup_fixture("zig_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add | func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.zig"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_zig_custom_struct() {
    let dir = setup_custom_project(&[(
        "structs.zig",
        r#"
const std = @import("std");

const Point = struct {
    x: f64,
    y: f64,

    pub fn distance(self: Point, other: Point) f64 {
        const dx = other.x - self.x;
        const dy = other.y - self.y;
        return std.math.sqrt(dx * dx + dy * dy);
    }

    pub fn add(self: Point, other: Point) Point {
        return Point{
            .x = self.x + other.x,
            .y = self.y + other.y,
        };
    }
};
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("struct:Point & func:distance")
        .assert()
        .success()
        .stdout(predicate::str::contains("structs.zig"));
}

#[test]
fn test_zig_custom_enum() {
    let dir = setup_custom_project(&[(
        "enums.zig",
        r#"
const Color = enum {
    red,
    green,
    blue,

    pub fn toRgb(self: Color) [3]u8 {
        return switch (self) {
            .red => [_]u8{ 255, 0, 0 },
            .green => [_]u8{ 0, 255, 0 },
            .blue => [_]u8{ 0, 0, 255 },
        };
    }
};
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("enum:Color & func:toRgb")
        .assert()
        .success()
        .stdout(predicate::str::contains("enums.zig"));
}

#[test]
fn test_zig_custom_error_handling() {
    let dir = setup_custom_project(&[(
        "errors.zig",
        r#"
const std = @import("std");

const ParseError = error{
    InvalidCharacter,
    Overflow,
};

fn parseNumber(str: []const u8) ParseError!u32 {
    var result: u32 = 0;
    for (str) |c| {
        if (c < '0' or c > '9') return error.InvalidCharacter;
        result = result * 10 + (c - '0');
    }
    return result;
}

fn safeDivide(a: u32, b: u32) !u32 {
    if (b == 0) return error.Overflow;
    return a / b;
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:parseNumber | func:safeDivide")
        .assert()
        .success()
        .stdout(predicate::str::contains("errors.zig"));
}

#[test]
fn test_zig_custom_allocator() {
    let dir = setup_custom_project(&[(
        "allocator.zig",
        r#"
const std = @import("std");

pub fn createList(allocator: std.mem.Allocator) !std.ArrayList(u32) {
    var list = std.ArrayList(u32).init(allocator);
    try list.append(1);
    try list.append(2);
    try list.append(3);
    return list;
}

pub fn freeList(list: *std.ArrayList(u32)) void {
    list.deinit();
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:createList & func:freeList")
        .assert()
        .success()
        .stdout(predicate::str::contains("allocator.zig"));
}

#[test]
fn test_zig_custom_comptime() {
    let dir = setup_custom_project(&[(
        "comptime.zig",
        r#"
fn fibonacci(comptime n: u32) u32 {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

const fib10 = fibonacci(10);

fn GenericMax(comptime T: type) type {
    return struct {
        pub fn max(a: T, b: T) T {
            return if (a > b) a else b;
        }
    };
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:fibonacci | func:GenericMax")
        .assert()
        .success()
        .stdout(predicate::str::contains("comptime.zig"));
}

#[test]
fn test_zig_custom_optional() {
    let dir = setup_custom_project(&[(
        "optionals.zig",
        r#"
fn findIndex(slice: []const u32, value: u32) ?usize {
    for (slice, 0..) |item, index| {
        if (item == value) return index;
    }
    return null;
}

fn getOrDefault(optional: ?u32, default: u32) u32 {
    return optional orelse default;
}
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:findIndex & func:getOrDefault")
        .assert()
        .success()
        .stdout(predicate::str::contains("optionals.zig"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_zig_format_paths() {
    let dir = setup_fixture("zig_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("func:greet")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.zig"));
}

#[test]
fn test_zig_format_markdown() {
    let dir = setup_fixture("zig_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("```zig"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_zig_not_found() {
    let dir = setup_fixture("zig_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:nonexistent & ext:zig")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_zig_ext_filter() {
    let dir = setup_fixture("zig_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:zig")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
