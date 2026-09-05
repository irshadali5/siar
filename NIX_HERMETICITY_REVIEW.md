# Nix hermeticity/reproducibility review — 2026-09-01

Static review only — this sandbox has no `nix` binary, so nothing below was
actually built or hash-compared. Treat the fixes as "should be correct on
inspection," and run `nix build .#siar-cli --rebuild` yourself to confirm.

## Fixed

1. **`rust-toolchain.toml` pinned `channel = "stable"`.** This is not a
   pinned input — rust-overlay resolves "stable" against whatever is
   current, so the exact rustc patch version a build gets can change over
   time even with `flake.lock` untouched. Two people building the same
   commit six months apart could get different compilers. Fixed to
   `channel = "1.91.0"`, matching `workspace.package.rust-version = "1.91"`
   in `Cargo.toml`. **You should confirm `1.91.0` is the exact patch you
   want** (bump both files together going forward).

2. **`RUSTFLAGS = "--remap-path-prefix=${src}=/build/siar"` was a no-op.**
   `${src}` is the `/nix/store/<hash>-source` path, known at *eval* time.
   But stdenv's default `unpackPhase` copies a directory `src` into the
   sandbox as `./source` under `$NIX_BUILD_TOP` before compiling — rustc
   only ever sees `$NIX_BUILD_TOP/source/...` in `file!()`/debug-info
   output, never the literal nix store path. So the remap flag was present
   but never matched anything; it did not make the build any more
   path-independent than not setting it. Moved the remap into a `preBuild`
   hook where `$NIX_BUILD_TOP` is actually known at build time.

3. **No test that the "bit-for-bit reproducible" claim is actually true.**
   The flake's own source-filtering comment claims hermetic/bit-for-bit
   reproducibility, but nothing in CI ever verified it — a green build only
   proves the derivation succeeds once, not that it's deterministic. Added
   `nix build .#siar-cli --rebuild` to `nix.yml`, which forces a second
   from-scratch build and fails the job if the output differs by a single
   byte.

4. **Offline guarantee was cargo-flag-only.** `cargoExtraArgs = "--offline"`
   stops cargo's own network calls but not a rogue `build.rs`, and doesn't
   stop cargo from silently rewriting `Cargo.lock` if something's missing.
   Added `CARGO_NET_OFFLINE = "true"` and `--frozen` so a lockfile drift is
   a hard build failure instead of a silent network fetch or silent
   relock.

## Not fixed — flagged for you to decide

- **`Cargo.lock` wasn't in this upload** (only in your local checkout per
  earlier session notes), so I could not check it for `git+` dependencies.
  Crane vendors both registry and git deps into a fixed-output derivation
  keyed on the lockfile hash, which is hermetic in the Nix sense (network
  allowed only inside a hash-checked FOD) — but if any git dependency uses
  a branch ref instead of a pinned rev/tag in `Cargo.toml`, `cargo update`
  could later resolve a different commit. Worth a quick
  `grep -A2 'source = "git' Cargo.lock` pass.

- **`nixpkgs.url` tracks the `nixos-unstable` branch, not a specific
  tag/rev.** `flake.lock` pins the actual commit for any given checkout,
  so this doesn't break reproducibility for a fixed lock file — but
  `nix flake update` will pull a different nixpkgs each time it's run,
  which is a maintenance/drift concern more than a hermeticity bug. Leave
  as-is unless you want tighter release-to-release stability, in which
  case point at a tagged release branch instead (e.g. `nixos-25.05`).

- **Android is entirely outside the Nix closure.** `apps/android`'s
  Gradle/Kotlin/JNI build (`build-native.sh`, `build.gradle.kts`) has no
  Nix derivation at all — `*.sh` files are explicitly excluded from
  `sourceFilter`. This is consistent with ROADMAP.md's Tier-3 note that
  only the Rust glue (`rust-jni-glue`, `messaging-jni`) is
  compile-verified, and real device/emulator/Kotlin behavior is out of
  reach either way — but if "true hermeticity" is meant to eventually
  cover the Android app build too, that's a much larger separate project
  (`android-nixpkgs` + a Gradle-hermeticity story), not a fix that belongs
  in this pass.

- **`cargo-deny`'s advisory-database check needs network** (it's a `check`,
  not a package build, so it runs outside the sandboxed FOD model) —
  unrelated to build reproducibility, just noting it's not offline.

## Not reviewed

`apps/desktop`'s runtime dynamic-linking behavior, actual byte-for-byte
diffing of two real build outputs, and the Darwin build path (no macOS
sandbox available here) — all need to be checked on real hardware/CI, not
in this text-only review.
