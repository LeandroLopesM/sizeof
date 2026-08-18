const std = @import("std");
const zlob = @import("zlob");
const args = @import("args");
const read = @import("readable");
const log = std.log;

const Args = struct {
    jobs: usize = 1,
    progress: bool = false,
};

pub fn usage(exec_name: []const u8) u8 {
    log.err("USAGE: {s}: [options] <root>", .{exec_name});

    return 1;
}

fn visit(ctx: ?*anyopaque, entry: *const zlob.walk.Entry) zlob.walk.VisitAction {
    if (entry.kind == .file) {
        const trueSize: *u64 = @as(*u64, @ptrCast(@alignCast(ctx.?)));
        trueSize.* += entry.meta.size;
    }

    return .cont;
}

pub fn main(init: std.process.Init) !u8 {
    var alloc: std.heap.ArenaAllocator = .init(init.gpa);
    defer alloc.deinit();

    const opts = args.parseForCurrentProcess(Args, init, .print) catch return 1;
    defer opts.deinit();

    if (opts.positionals.len == 0) {
        return usage(opts.executable_name orelse "sizeof");
    }

    var size = try alloc.allocator().alloc(u64, 1);
    size[0] = 0;

    const r = try zlob.walk.run(
        alloc.allocator(),
        opts.positionals[0],
        .{
            .meta = .{
                .size = true,
            },
        },
        .{
            .context = &size[0],
            .visit = visit,
        },
    );
    defer r.deinit();

    var buff: [64]u8 = undefined;
    const tmp: read.ReadableSize = .{ .bytes = size[0] };
    const fmt = try tmp.formatInto(&buff);

    log.info("{s}", .{fmt});
    return 0;
}
