pub const system = @import("system.zig");

// Current hack to run all tests on all imports
test {
    @import("std").testing.refAllDecls(@This());
}
