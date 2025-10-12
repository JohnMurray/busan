const std = @import("std");
const builtin = @import("builtin");

pub fn bufferedPrint() !void {
    const stdout_file = std.io.getStdOut().writer();
    var bw = std.io.bufferedWriter(stdout_file);
    const stdout = bw.writer();

    try stdout.print("Run `zig build test` to run the tests.\n", .{});

    try bw.flush(); // Don't forget to flush!
}

pub fn add(a: i32, b: i32) i32 {
    return a + b;
}

test "basic add functionality" {
    try std.testing.expect(add(3, 7) == 10);
}

pub const counter = @import("counter.zig");

// Current hack to run all tests on all imports
test {
    @import("std").testing.refAllDecls(@This());
}
