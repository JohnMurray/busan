const std = @import("std");
const builtin = @import("builtin");

pub fn main() void {
    const test_functions = builtin.test_functions;
    var passed: usize = 0;
    var failed: usize = 0;

    for (test_functions) |test_fn| {
        std.debug.print("Running test: {s} ... ", .{test_fn.name});

        if (test_fn.func()) |_| {
            std.debug.print("✓ PASSED\n", .{});
            passed += 1;
        } else |err| {
            std.debug.print("✗ FAILED: {any}\n", .{err});
            failed += 1;
        }
    }

    std.debug.print("\n", .{});
    std.debug.print("Test Summary:\n", .{});
    std.debug.print("  Passed: {d}\n", .{passed});
    std.debug.print("  Failed: {d}\n", .{failed});
    std.debug.print("  Total:  {d}\n", .{test_functions.len});

    if (failed > 0) {
        std.process.exit(1);
    }
}
