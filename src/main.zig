const std = @import("std");
const zlob = @import("zlob");
const args = @import("args");
const read = @import("readable");
const log = std.log;

const Args = struct {
    jobs: usize = 1,
    progress: bool = false,
};

fn visit(ctx: ?*anyopaque, entry: *const zlob.walk.Entry) zlob.walk.VisitAction {
    if (entry.kind == .file) {
        const trueSize: *read.ReadableSize = @as(*read.ReadableSize, @ptrCast(@alignCast(ctx.?)));
        trueSize.*.bytes += entry.meta.size;
    }

    return .cont;
}

pub fn main(init: std.process.Init) !u8 {
    var alloc: std.heap.ArenaAllocator = .init(init.gpa);
    defer alloc.deinit();

    const opts = args.parseForCurrentProcess(Args, init, .print) catch return 1;
    defer opts.deinit();

    if (opts.positionals.len == 0) {
        log.err("USAGE: sizeof [flags] <root>", .{});
        return 1;
    }

    var size = try alloc.allocator().alloc(read.ReadableSize, 1);
    size[0].bytes = 0;

    const r = try zlob.walk.run(
        alloc.allocator(),
        opts.positionals[0],
        .{
            .respect_git = false,
            .skip_git_dir = false,
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

    var buff: [128]u8 = undefined;
    const fmt = try size[0].formatInto(&buff);

    log.info("{s}", .{fmt});
    return 0;
}
