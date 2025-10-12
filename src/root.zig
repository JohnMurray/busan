pub const system = @import("system.zig");
pub const std = @import("std");

pub const ActorSystem = struct {};

pub const Executor = struct {};

pub const ActorCell = struct {};

pub const Mailbox = struct {
    queue: std.PriorityDequeue(Envelope, void, equalPriority),
};

fn equalPriority(_: void, _: Envelope, _: Envelope) std.math.Order {
    return std.math.Order.eq;
}

pub const Envelope = struct {
    message: *void,
};

// Current hack to run all tests on all imports
test {
    @import("std").testing.refAllDecls(@This());
}
