const std = @import("std");

pub fn greet(name: []const u8) void {
    std.debug.print("Hello {s}\n", .{name});
}

pub fn add(a: i32, b: i32) i32 {
    return a + b;
}

pub fn main() !void {
    greet("world");
    const r = add(1, 2);
    _ = r;
}
