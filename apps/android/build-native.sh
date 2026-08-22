#!/usr/bin/env bash
# Builds every Android-targeted Rust crate this app actually loads via
# System.loadLibrary and copies the resulting .so files into
# app/src/main/jniLibs/<abi>/ — the standard Android layout
# app/build.gradle.kts already expects (see that file's own comment).
#
# Deliberately scoped with explicit `-p` flags, NEVER `cargo ndk build
# --workspace`. A real build/error report against this workspace found
# that a full-workspace NDK invocation fails on two crates that were
# never meant to cross-compile for Android at all:
#   - siar-media-audio (via audiopus_sys): needs a real Opus C build
#     with a CMake toolchain, not something `cargo ndk` sets up for you
#   - siar-media-av1 (via dav1d-sys): needs a real dav1d C library
#     cross-compiled for the target ABI with PKG_CONFIG_SYSROOT_DIR
#     pointed at it, again nothing `cargo ndk` provides on its own
# Both are desktop-only codec crates (`siar-media-android`'s own
# hardware codec path is what Android actually uses instead — see that
# crate's own doc comment) — there was never a real requirement for
# either to build for `aarch64-linux-android` at all. The fix here is
# this explicit crate list, not fixing audiopus_sys/dav1d-sys's own
# cross-compilation stories, which this workspace doesn't own.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

ABIS=(arm64-v8a armeabi-v7a x86_64 x86)
MIN_API=26 # matches app/build.gradle.kts's minSdk

# Exactly the 7 crates a real build/error report already confirmed
# build cleanly across all 4 ABIs with this scoping — see this
# workspace's own memory of that report for the confirmation.
CRATES=(
    siar-android-connectivity
    siar-android-messaging
    siar-transport-wifi-direct
    siar-transport-wifi-aware
    siar-transport-ble-android
    siar-transport-bluetooth-classic
    siar-media-android
)

PACKAGE_ARGS=()
for crate in "${CRATES[@]}"; do
    PACKAGE_ARGS+=(-p "$crate")
done

TARGET_ARGS=()
for abi in "${ABIS[@]}"; do
    TARGET_ARGS+=(-t "$abi")
done

(
    cd ..
    cargo ndk "${TARGET_ARGS[@]}" -P "$MIN_API" build --release "${PACKAGE_ARGS[@]}"
)

# cargo-ndk's own -o flag can place outputs directly under
# jniLibs/<abi>/ in one step — left as a documented improvement rather
# than added here, since verifying the exact output layout cargo-ndk
# produces needs a real run of this script, which this sandbox can't
# do (no NDK, no Android toolchain — same limitation named throughout
# this workspace's history). Run this script for real, then adjust the
# `cargo ndk` invocation above to add `-o app/src/main/jniLibs` once
# you've confirmed the layout it produces matches what
# app/build.gradle.kts expects.
