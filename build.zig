const std = @import("std");

const DepTuple = struct {
    tn: []const u8,
    in: []const u8,
};

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    const exe_mod = b.createModule(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{},
    });

    const exe = b.addExecutable(.{
        .name = "sizeof",
        .root_module = exe_mod,
    });

    {
        const moduleNames: []const DepTuple = &.{
            .{ .tn = "zlob", .in = "zlob" },
            .{ .tn = "args", .in = "args" },
            .{ .tn = "readable", .in = "readable_size" },
        };

        for (moduleNames) |mod| {
            const dep = b.dependency(mod.tn, .{
                .target = target,
                .optimize = optimize,
            });

            exe.root_module.addImport(mod.tn, dep.module(mod.in));
        }
    }

    b.installArtifact(exe);

    const run_step = b.step("run", "Run the app");
    const run_cmd = b.addRunArtifact(exe);

    run_step.dependOn(&run_cmd.step);
    run_cmd.step.dependOn(b.getInstallStep());

    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const exe_check = b.addExecutable(.{
        .name = "sizeof",
        .root_module = exe_mod,
    });
    const check = b.step("check", "Check if this compiles");
    check.dependOn(&exe_check.step);
}
