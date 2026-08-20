const std = @import("std");
const zlob = @import("zlob");
const args = @import("args");
const read = @import("readable.zig");
const log = std.log;

const Args = struct {
    jobs: usize = 1,
    progress: bool = false,
    humanized: bool = false,

    pub const shorthands = .{
        .p = "progress",
        .h = "humanized",
    };
};

const Context = struct {
    size: u64 = 0,
    bar: struct {
        file: ?std.Progress.Node = null,
        dir: ?std.Progress.Node = null,
    } = .{},
};

fn visit(ctx: ?*anyopaque, entry: *const zlob.walk.Entry) zlob.walk.VisitAction {
    const c: *Context = @as(*Context, @ptrCast(@alignCast(ctx.?)));

    if (entry.kind == .file) {
        c.*.size += entry.meta.size;

        if (c.bar.file) |b| {
            b.setName(entry.basename);
            b.completeOne();
        }
    } else if (entry.kind == .directory) {
        if (c.bar.dir) |b| {
            b.setName(entry.basename);
            b.completeOne();
        }
    }
    return .cont;
}

pub fn main(init: std.process.Init) !u8 {
    var arena: std.heap.ArenaAllocator = .init(init.gpa);
    const alloc = arena.allocator();
    defer arena.deinit();

    const opts = args.parseForCurrentProcess(Args, init, .print) catch return 1;
    defer opts.deinit();

    if (opts.positionals.len == 0) {
        log.err("USAGE: sizeof [flags] <root>", .{});
        return 1;
    }

    var ctx = Context{};
    const rootNode = std.Progress.start(init.io, .{});

    if (opts.options.progress) {
        ctx.bar.dir = rootNode.startFmt(0, "Measuring {s}", .{opts.positionals[0]});
        ctx.bar.file = ctx.bar.dir.?.start("", 0);
    }

    (try zlob.walk.run(
        alloc,
        opts.positionals[0],
        .{
            .respect_git = false,
            .skip_git_dir = false,
            .meta = .{
                .size = true,
                .nlink = true,
            },
        },
        .{
            .context = &ctx,
            .visit = visit,
        },
    )).deinit();

    if (ctx.bar.file) |b| {
        b.end();
    }

    if (ctx.bar.dir) |b| b.end();

    const fmt = read.from(alloc, ctx.size, opts.options.humanized) catch "Unknown\n";
    try std.Io.File.stdout().writeStreamingAll(init.io, fmt);
    return 0;
}
