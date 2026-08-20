const std = @import("std");

const SIUnit = enum(u8) {
    b,
    kb,
    mb,
    gb,
    tb,
    pb,
    yb,

    pub fn string(raw: i32) []const u8 {
        return switch (@as(SIUnit, @enumFromInt(raw))) {
            .b => "B",
            .kb => "KB",
            .mb => "MB",
            .gb => "GB",
            .tb => "TB",
            .pb => "PB",
            .yb => "YB",
        };
    }
};

const HumanUnit = enum(u8) {
    bi,
    kib,
    mib,
    gib,
    tib,
    pib,
    yib,

    pub fn string(raw: i32) []const u8 {
        return switch (@as(HumanUnit, @enumFromInt(raw))) {
            .bi => "Bi",
            .kib => "KiB",
            .mib => "MiB",
            .gib => "GiB",
            .tib => "TiB",
            .pib => "PiB",
            .yib => "YiB",
        };
    }
};

fn serialize(a: std.mem.Allocator, raw: i32, val: f64, human: bool) ![]u8 {
    return try std.fmt.allocPrint(a, "{d:.2} {s}\n", .{ val, if (human) HumanUnit.string(raw) else SIUnit.string(raw) });
}

pub fn from(a: std.mem.Allocator, num: u64, humanized: bool) ![]u8 {
    var k: i32 = 0;
    var f: f64 = @floatFromInt(num);
    const dim: f64 = if (humanized) 1024.0 else 1000.0;

    while (f / dim >= 1.0) {
        f /= dim;
        k += 1;
    }

    return serialize(a, k, f, humanized);
}
