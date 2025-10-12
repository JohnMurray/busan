//! The counter module provides a simple counter data structure.
//!
//! The counter CRDT is represented as ops of either increment or decrement, that are summed
//! and applied to the counter value.

const std = @import("std");
const t = std.testing;

const GCounterMap = std.HashMap(u32, u64, std.hash_map.AutoContext(u32), std.hash_map.default_max_load_percentage);

pub const GCounter = struct {
    id: u32,
    counters: GCounterMap,
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator, id: u32) GCounter {
        return GCounter{
            .id = id,
            .counters = GCounterMap.init(allocator),
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *GCounter) void {
        self.counters.deinit();
    }

    pub fn increment(self: *GCounter, value: u32) !void {
        const current_entry = try self.counters.getOrPut(self.id);
        if (current_entry.found_existing) {
            try self.counters.put(self.id, *current_entry.value_ptr + value);
        } else {
            try self.counters.put(self.id, value);
        }
    }

    pub fn query(self: *GCounter) u64 {
        var total: u64 = 0;
        for (self.counters.values()) |value| {
            total += value;
        }
        return total;
    }

    pub fn merge_in_place(self: *GCounter, other: *GCounter) !void {
        for (other.counters.keys()) |key| {
            const other_value = other.counters.get(key).?;
            const current_entry = try self.counters.getOrPut(key);
            var value: u64 = 0;
            if (current_entry.found_existing) {
                value = *current_entry.value_ptr;
            }

            try self.counters.put(key, @max(value, other_value));
        }
    }

    pub fn merge(self: *GCounter, others: []*GCounter) !GCounter {
        var merged = GCounter.init(self.allocator, self.id);
        for (others) |other| {
            try merged.merge_in_place(other);
        }
        return merged;
    }
};

test "init and deinit an empty gcounter" {
    var gcounter = GCounter.init(t.allocator, 1);
    defer gcounter.deinit();
    try t.expect(gcounter.counters.count() == 0);
}

test "increment a gcounter" {
    var gcounter = GCounter.init(t.allocator, 1);
    defer gcounter.deinit();
    try gcounter.increment(1);
    try t.expect(gcounter.counters.count() == 1);
}

test "increment two gcounters and perform an immutable merge" {
    var gcounter1 = GCounter.init(t.allocator, 1);
    defer gcounter1.deinit();
    var gcounter2 = GCounter.init(t.allocator, 2);
    defer gcounter2.deinit();

    try gcounter1.increment(1);
    try gcounter2.increment(2);

    var merged = try gcounter1.merge(&[_]*GCounter{&gcounter2});
    defer merged.deinit();

    try t.expect(gcounter1.counters.query() == 1);
    try t.expect(gcounter2.counters.query() == 2);
    try t.expect(merged.counters.query() == 3);
}
