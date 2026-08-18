const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    const exe_mod = b.addModule("sizeof", .{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    const exe = b.addExecutable(.{
        .name = "sizeof",
        .root_module = exe_mod,
    });

    {
        const moduleNames: []const []const u8 = &.{
            "zlob",
            "args",
            "readable_size",
        };

        for (moduleNames) |mod| {
            const dep = b.dependency(mod, .{
                .target = target,
                .optimize = optimize,
            });

            exe.root_module.addImport(mod, dep.module(mod));
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
