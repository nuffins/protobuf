# Package containing kernel-independent libraries.
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "common",
    srcs = [
        "lib.rs",
        "proxied.rs",
    ],
    visibility = ["//rust:__subpackages__"],
)

rust_test(
    name = "common_test",
    crate = ":common",
)
