# Part 27 — Rust-Driven Android Native Build & Packaging Automation Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 27 of 27 additional production-hardening parts  
**Primary language:** Rust  
**Android build hosts:** Rust `xtask` + Cargo + Android NDK + Gradle/AGP as packaging/signing backend  
**Primary goals:** one-command reproducible Android builds, automatic Rust `.so` generation for all required ABIs, deterministic staging into Android packaging, Dioxus integration, version consistency, signing safety, Play Store AAB/APK production, CI automation, symbols/SBOM/checksums, failure-proof release gates, and elimination of manual `jniLibs` workflows

---

# 1. Purpose

The previous Android architecture assumes a native Rust communication/media core.

A weak build workflow often looks like:

```text
manually run cargo-ndk
copy libsomething.so into jniLibs
run Gradle
hope versions match
```

This is acceptable during prototyping but not for production.

Manual native-library staging creates risks:

```text
wrong ABI
stale .so
debug library in release build
missing architecture
version mismatch
forgotten rebuild
wrong NDK
wrong Cargo features
unsigned or incorrectly signed artifact
non-reproducible CI
```

The production architecture must turn Android building into a deterministic pipeline.

The core rule is:

> **Rust `xtask` owns build intent and orchestration; Cargo builds native Rust, the Android NDK supplies target toolchains, and Gradle/AGP packages/signs Android artifacts.**

Gradle is still required because Android's application packaging/signing ecosystem is Gradle/AGP-centric, but Gradle should not become the source of truth for Rust build policy.

---

# 2. Desired Developer Experience

Development:

```text
cargo xtask android run
```

Release candidate:

```text
cargo xtask android build --release
```

Play Store:

```text
cargo xtask android bundle --release
```

Signed APK:

```text
cargo xtask android apk --release
```

Everything required should happen automatically:

```text
toolchain validation
target setup
Rust build
native library collection
JNI staging
Gradle packaging
artifact validation
```

---

# 3. Architectural Position

```text
Developer / CI
      │
      ▼
   cargo xtask
      │
      ├── validate environment
      ├── select Android ABIs
      ├── invoke Cargo/NDK
      ├── stage .so artifacts
      ├── generate build metadata
      ├── invoke Gradle
      ├── verify APK/AAB
      └── collect release artifacts
               │
               ▼
         Android Gradle Project
               │
               ▼
           APK / AAB
```

---

# 4. Responsibilities

## `xtask` owns

```text
build profile
Rust feature selection
target ABI matrix
toolchain discovery
native build invocation
artifact staging
build metadata
release checks
Gradle invocation
artifact collection
verification
```

## Cargo owns

```text
Rust compilation
dependency resolution
Rust features
Cargo.lock
```

## Android NDK owns

```text
Android C ABI toolchain
linker
sysroot
platform native libraries
```

## Gradle / AGP owns

```text
Android resources
AndroidManifest
DEX/Kotlin/Java
APK/AAB packaging
signing integration
Play Store bundle format
```

---

# 5. Why Not Replace Gradle Entirely

Attempting to replace Gradle for official Android packaging would add enormous maintenance burden around:

```text
AAB
manifest merging
resources
R8
APK signing
bundle metadata
Play Store conventions
Android SDK changes
```

That offers little architectural benefit.

The Rust-centric solution is:

```text
Rust controls Gradle
```

rather than:

```text
Rust replaces Android packaging infrastructure
```

---

# 6. Workspace Layout

Recommended:

```text
/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── xtask/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── android/
│       │   ├── mod.rs
│       │   ├── build.rs
│       │   ├── toolchain.rs
│       │   ├── targets.rs
│       │   ├── stage.rs
│       │   ├── gradle.rs
│       │   ├── signing.rs
│       │   ├── metadata.rs
│       │   ├── verify.rs
│       │   ├── symbols.rs
│       │   └── release.rs
│       └── util/
├── apps/
│   └── android/
│       ├── build.gradle.kts
│       ├── settings.gradle.kts
│       ├── gradle.properties
│       └── app/
│           ├── build.gradle.kts
│           ├── src/main/
│           │   ├── AndroidManifest.xml
│           │   ├── java/ or kotlin/
│           │   ├── res/
│           │   └── jniLibs/
│           └── proguard-rules.pro
└── crates/
    └── ...
```

---

# 7. `xtask` Pattern

Use a workspace-local Rust binary:

```text
cargo xtask ...
```

`xtask` is not shipped to users.

It exists for:

```text
build
test
package
release
CI
developer tooling
```

---

# 8. `cargo xtask`

Root Cargo configuration can expose alias:

```toml
[alias]
xtask = "run --package xtask --"
```

Then:

```text
cargo xtask android build
```

---

# 9. Android Command Tree

Suggested CLI:

```text
cargo xtask android doctor
cargo xtask android build
cargo xtask android run
cargo xtask android test
cargo xtask android apk
cargo xtask android bundle
cargo xtask android symbols
cargo xtask android clean
cargo xtask android release
```

---

# 10. `doctor`

Before builds, validate:

```text
Java
Android SDK
Android NDK
Gradle wrapper
adb
Rust toolchain
Rust Android targets
signing config if release
```

---

# 11. Fail Early

Bad:

```text
compile for 15 minutes
then discover NDK missing
```

Good:

```text
doctor detects missing dependency immediately
```

---

# 12. Toolchain Manifest

Pin required versions in one machine-readable file.

Example:

```ron
(
    android: (
        min_sdk: 26,
        target_sdk: 36,
        compile_sdk: 36,
        ndk: "28.0.x",
    ),
)
```

Exact versions should follow current product requirements at implementation time.

---

# 13. Version Source of Truth

Avoid defining versions independently in:

```text
Cargo
Gradle
CI
docs
release script
```

Use one source of truth and generate/validate downstream configuration.

---

# 14. Build Metadata Crate

Optional:

```text
crates/build-metadata/
```

contains:

```text
product version
protocol version
ABI version
git commit
build channel
```

---

# 15. App Version

Maintain:

```text
semver-like app version
Android versionName
Android versionCode
```

through one release metadata path.

---

# 16. Version Code

Android requires monotonically increasing integer.

`xtask` should validate:

```text
versionCode > previous release
```

in release pipeline where previous metadata available.

---

# 17. Supported Android ABIs

Recommended initial release:

```text
arm64-v8a
x86_64
```

Possible additional:

```text
armeabi-v7a
```

only if real user/device requirements justify it.

---

# 18. Rust Target Mapping

Example:

```text
arm64-v8a
→ aarch64-linux-android

x86_64
→ x86_64-linux-android

armeabi-v7a
→ armv7-linux-androideabi
```

---

# 19. ABI Mapping Type

```rust
pub enum AndroidAbi {
    Arm64V8a,
    X86_64,
    ArmeabiV7a,
}
```

---

# 20. Central Target Mapping

One Rust function defines:

```text
Android ABI
Rust target
jniLibs directory
NDK linker target
```

Do not duplicate mappings in shell scripts.

---

# 21. Native Library

Example output:

```text
libcomm_android.so
```

or product-specific name.

---

# 22. Library Crate

Android native entry crate:

```text
crates/comm-android-ffi/
```

Cargo:

```toml
[lib]
crate-type = ["cdylib"]
```

---

# 23. One Android Native Entry Library

Prefer one primary JNI/native shared library.

Benefits:

```text
simpler loading
fewer packaging issues
one ABI boundary
```

Internal Rust crates link statically into it.

---

# 24. Multiple `.so` Libraries

Only when needed for:

```text
third-party native dependency
special codec library
platform separation
```

Track explicitly.

---

# 25. Build Flow

For each ABI:

```text
select target
 ↓
configure NDK compiler/linker
 ↓
cargo build
 ↓
locate resulting .so
 ↓
strip/copy symbols according to profile
 ↓
stage into jniLibs/<abi>/
```

---

# 26. No Manual Copy

`xtask` must own:

```text
jniLibs staging
```

Developer should never manually move `.so`.

---

# 27. Staging Directory

Use generated directory:

```text
target/android-staging/
```

Then either:

```text
copy/sync to app/src/main/jniLibs
```

or configure Gradle to consume the generated directory directly.

---

# 28. Prefer Generated Native Directory

Better:

```text
Gradle jniLibs.srcDirs += target/android-staging/jniLibs
```

This avoids modifying source tree during build.

---

# 29. Source Tree Cleanliness

Do not commit compiled:

```text
.so
```

into Git.

---

# 30. Gradle Native Input

Generated:

```text
target/android-staging/jniLibs/arm64-v8a/libcomm_android.so
target/android-staging/jniLibs/x86_64/libcomm_android.so
```

---

# 31. Stale Library Protection

Before native build:

```text
clear staging for selected build profile
```

After build:

```text
verify every expected ABI exists
```

---

# 32. Build Fingerprint

For each `.so`, record:

```text
Rust target
Cargo profile
features
commit
digest
```

---

# 33. Native Artifact Manifest

```ron
(
    profile: "release",
    git: "...",
    artifacts: [
        (
            abi: "arm64-v8a",
            sha256: "...",
        ),
    ],
)
```

---

# 34. Gradle Preflight Check

Gradle packaging should verify expected native manifest exists.

This prevents standalone Gradle release from accidentally packaging stale libraries.

---

# 35. Gradle Guard Task

Create task:

```text
verifyRustNativeArtifacts
```

that fails if:

```text
missing manifest
wrong profile
missing ABI
version mismatch
```

---

# 36. Gradle Dependency

Ensure:

```text
preBuild
```

depends on:

```text
verifyRustNativeArtifacts
```

---

# 37. Optional Gradle → xtask Trigger

Could make Gradle invoke Rust automatically.

But recommended source of truth is:

```text
xtask → Cargo → Gradle
```

rather than recursive:

```text
Gradle → xtask → Gradle
```

---

# 38. Avoid Build Cycles

Choose one top-level orchestrator:

```text
xtask
```

---

# 39. Development Escape Hatch

A developer may run Gradle directly for UI-only work.

Then Gradle should:

```text
reuse last valid debug native artifact
```

or fail with clear message.

---

# 40. Automatic Debug Native Build

Option:

```text
./gradlew assembleDebug
```

can call a small Rust build preparation script/task.

But keep release build strictly through `xtask`.

---

# 41. Cargo-NDK

`cargo-ndk` can be used as implementation detail if it remains useful and maintained.

Architecture should not depend semantically on it.

---

# 42. Direct Cargo + NDK

`xtask` can alternatively configure:

```text
CC
AR
CARGO_TARGET_*_LINKER
```

and invoke Cargo directly.

---

# 43. Build Tool Abstraction

```rust
pub trait AndroidNativeBuilder {
    fn build(&self, request: NativeBuildRequest) -> Result<NativeArtifacts, BuildError>;
}
```

Implementations:

```text
CargoNdkBuilder
DirectNdkBuilder
```

---

# 44. Default

Use the simplest robust approach.

If `cargo-ndk` works reliably:

```text
use it
```

No need to reimplement its functionality just for Rust purity.

---

# 45. "Rust-Driven" Meaning

The orchestration logic is Rust.

It can invoke:

```text
cargo
cargo-ndk
gradlew
adb
bundletool
```

as external tools.

---

# 46. No Shell-Script Core

Avoid large:

```text
build_android.sh
release_android.sh
```

containing business logic.

Tiny wrapper scripts are acceptable.

---

# 47. Process Invocation

Use Rust:

```rust
std::process::Command
```

or ergonomic command crate.

---

# 48. Command Logging

Print:

```text
tool
arguments sanitized
target
duration
result
```

Never print signing secrets.

---

# 49. Structured Build Errors

```rust
pub enum AndroidBuildError {
    MissingSdk,
    MissingNdk,
    UnsupportedNdk,
    MissingRustTarget,
    CargoFailed,
    MissingNativeArtifact,
    GradleFailed,
    SigningMisconfigured,
    ArtifactVerificationFailed,
}
```

---

# 50. Build Profiles

Support:

```rust
pub enum AndroidBuildProfile {
    Debug,
    Dev,
    Profile,
    Release,
}
```

---

# 51. Debug

```text
fast compile
debug symbols
logging
no optimization or low optimization
```

---

# 52. Dev

Useful compromise:

```text
some optimization
debug assertions
developer diagnostics
```

---

# 53. Profile

Performance testing:

```text
release optimization
symbols retained
profiling hooks
```

---

# 54. Release

```text
full optimization
production features
release signing
strict checks
```

---

# 55. Rust Cargo Profiles

Map Android profiles to:

```text
Cargo profile
Gradle build type
feature set
```

---

# 56. Release Feature Freeze

Explicitly define release features.

Example:

```text
android
dioxus
iroh
files
dtn
calls
android-hw-video
audio-dsp
```

Do not rely on accidental default feature expansion.

---

# 57. Feature Manifest

```ron
(
    release_features: [
        "android",
        "iroh",
        "calls",
    ],
)
```

---

# 58. Debug-Only Features

Examples:

```text
network inspector
unsafe developer plugin mode
verbose tracing
```

must not accidentally ship in release.

---

# 59. Release Guard

`xtask` checks forbidden features.

---

# 60. NDK Discovery

Look in:

```text
ANDROID_NDK_HOME
ANDROID_SDK_ROOT/ndk/<version>
```

according to policy.

---

# 61. Do Not Guess Arbitrarily

If multiple NDKs installed:

```text
select pinned version
```

or fail with clear instructions.

---

# 62. SDK Discovery

Validate:

```text
platform
build-tools
platform-tools
```

required versions.

---

# 63. Java Discovery

Gradle/AGP needs compatible JDK.

`doctor` validates.

---

# 64. Rust Target Installation

`doctor` can report:

```text
rustup target add aarch64-linux-android
```

Optionally `xtask setup` installs automatically.

---

# 65. Setup Command

```text
cargo xtask android setup
```

Can:

```text
check SDK
check NDK
install Rust targets
create local generated config
```

Do not silently install large Android SDK components without explicit user action.

---

# 66. Local Config

Machine-specific paths belong in:

```text
local.properties
```

or environment.

Do not commit them.

---

# 67. Generated Gradle Properties

`xtask` may generate:

```text
target/android-staging/build.properties
```

with:

```text
native staging path
build commit
feature hash
```

---

# 68. Dioxus Integration

The Android app may be Dioxus-based.

`xtask` should treat Dioxus build output as part of app build graph.

---

# 69. Dioxus CLI

If Dioxus tooling is required:

```text
xtask invokes it
```

or directly builds appropriate Rust crate.

Keep one top-level command.

---

# 70. Rust Native Core vs Dioxus App

Possible architecture:

```text
Dioxus application Rust
+
Android platform shell
+
native communication/media core
```

Avoid compiling duplicate copies of core into multiple shared libraries.

---

# 71. Native Symbol Ownership

Ensure JNI symbols exist only in intended library.

---

# 72. JNI Registration

Prefer:

```text
JNI_OnLoad
+
explicit native method registration
```

or stable generated bindings.

Avoid fragile name-based JNI where possible.

---

# 73. Native Library Load

Android shell loads:

```kotlin
System.loadLibrary("comm_android")
```

This tiny Kotlin line is fine.

---

# 74. Load Validation

Expose:

```text
native build ID
ABI version
SDK version
```

and compare with Android app build metadata at startup.

---

# 75. Native/App Version Handshake

Kotlin/Rust bridge:

```text
expected native ABI = X
actual native ABI = Y
```

Mismatch:

```text
fail clearly
```

---

# 76. Why Version Handshake

Protects against:

```text
stale jniLibs
partial install
development mismatch
```

---

# 77. Build ID

Generate:

```text
git commit
dirty flag
timestamp optional
feature digest
```

---

# 78. Reproducibility and Timestamp

For reproducible builds, avoid embedding current wall-clock timestamp unless explicitly outside reproducible binary.

Use:

```text
SOURCE_DATE_EPOCH
```

where appropriate.

---

# 79. Dirty Tree Policy

Release build should fail if Git tree dirty by default.

Override only explicitly.

---

# 80. Release Tag

Release command can require:

```text
version tag
```

matches metadata.

---

# 81. Cargo.lock

Release requires committed:

```text
Cargo.lock
```

---

# 82. Gradle Dependency Locking

Enable dependency verification/locking where practical.

---

# 83. NDK Version Pinning

CI and developers use same NDK major/minor.

---

# 84. Gradle Wrapper

Commit:

```text
gradlew
gradle-wrapper.properties
```

Use wrapper, not system Gradle.

---

# 85. AGP Pinning

Pin Android Gradle Plugin version.

Upgrade deliberately.

---

# 86. Kotlin Version Pinning

If Kotlin is used, pin version.

---

# 87. Build Reproducibility

Pin:

```text
Rust toolchain
Cargo.lock
NDK
SDK level
Gradle wrapper
AGP
Kotlin
Java major
```

---

# 88. Build Environment Manifest

`xtask doctor --json` can emit machine-readable tool versions.

---

# 89. CI Build Container

For Linux CI, use controlled image containing:

```text
JDK
Android SDK
NDK
Rust
```

or install deterministically.

---

# 90. Cache Strategy

Cache:

```text
Cargo registry
Cargo git
Rust target
Gradle cache
NDK downloads
```

Do not cache:

```text
final release staging blindly
```

without fingerprint validation.

---

# 91. ABI Parallelism

ABIs can build in parallel if machine memory allows.

---

# 92. Resource-Aware Build Parallelism

`xtask` can cap:

```text
ABI jobs
Cargo jobs
```

for CI.

---

# 93. Artifact Staging Atomicity

Build into temporary directory:

```text
target/android-staging.tmp
```

Then rename to active staging after success.

---

# 94. Partial Build Protection

If x86_64 build fails after arm64 succeeds:

```text
do not leave mixed release staging
```

---

# 95. Native Digest Verification

Before Gradle:

```text
hash every .so
```

and record manifest.

---

# 96. ELF Validation

On Linux host, inspect:

```text
architecture
SONAME
needed libraries
```

using suitable tools.

---

# 97. Android API Compatibility

Ensure native library does not depend on symbols above configured minSdk unexpectedly.

---

# 98. Native Dependency Audit

Check `DT_NEEDED`.

Unexpected:

```text
libfoo.so
```

not packaged:

```text
fail build
```

---

# 99. Strip Strategy

Release:

```text
strip runtime symbols
```

but preserve separate debug symbols.

---

# 100. Symbols

Collect:

```text
unstripped .so
symbol files
mapping files
```

for crash diagnosis.

---

# 101. Native Debug Symbols

Store outside public APK/AAB release artifact where appropriate.

---

# 102. Play Console Symbols

Generate/upload compatible native debug symbol archive as part of release workflow if needed.

---

# 103. R8 / ProGuard

If Kotlin/Java surface is small:

```text
R8 still enabled for release
```

Keep JNI-required classes/methods from being removed when necessary.

---

# 104. JNI Keep Rules

Generate/maintain explicit rules.

---

# 105. Resource Shrinking

Can enable for release if safe.

---

# 106. APK vs AAB

Official Play Store:

```text
AAB
```

Direct site download:

```text
APK
```

---

# 107. Build Commands

```text
cargo xtask android bundle --release
cargo xtask android apk --release
```

---

# 108. Universal APK

For website distribution, choose:

```text
universal APK
```

or separate ABI APKs.

---

# 109. Separate ABI APKs

Smaller downloads but more release files.

---

# 110. Recommended Direct Distribution

Initially:

```text
one universal signed APK
```

for simplicity.

Later optionally:

```text
per-ABI APKs
```

---

# 111. Play Bundle

AAB allows Play to generate device-specific splits.

---

# 112. Signing Architecture

Never hard-code:

```text
keystore password
key password
```

in source.

---

# 113. Signing Inputs

Read from:

```text
environment
CI secret store
local secure configuration
```

---

# 114. Local Release Signing

Possible:

```text
ANDROID_KEYSTORE_PATH
ANDROID_KEYSTORE_PASSWORD
ANDROID_KEY_ALIAS
ANDROID_KEY_PASSWORD
```

Environment variable names can be product-specific.

---

# 115. Do Not Print Secrets

Command logs must redact:

```text
passwords
tokens
key paths if sensitive
```

---

# 116. Play App Signing

Prefer Play App Signing for Play distribution if product chooses it.

Local/CI uses upload key.

---

# 117. Direct APK Signing

Website APK needs product signing key.

Protect carefully.

---

# 118. Signing Separation

Potential:

```text
Play upload key
direct-distribution signing key
```

depending strategy.

---

# 119. Reproducibility vs Signing

Signing may make byte-for-byte reproducibility harder.

Keep:

```text
unsigned deterministic artifact
+
signed distribution artifact
```

where useful.

---

# 120. Signing Validation

After build:

```text
verify APK signature
```

using official Android tooling.

---

# 121. AAB Validation

Use:

```text
bundletool
```

or official tooling for structural validation.

---

# 122. Package Name

Validate release application ID.

Prevent accidental:

```text
.dev
.debug
```

package release.

---

# 123. Release Manifest Validation

Check:

```text
INTERNET
BLUETOOTH
camera
microphone
foreground service types
```

against actual features.

---

# 124. Permission Audit

Release pipeline should produce permission report.

Unexpected dangerous permission:

```text
fail or require explicit approval
```

---

# 125. Android Exported Components

Validate:

```text
activities
services
receivers
providers
```

for unnecessary exported access.

---

# 126. Network Security Config

Validate production config.

Do not ship debug trust-all certificates.

---

# 127. Debuggable Flag

Release APK must be:

```text
debuggable=false
```

---

# 128. Backup Rules

Validate Android backup behavior for:

```text
identity keys
sensitive state
```

---

# 129. Native Key Storage

Part 19/identity secure-store config must align with release.

---

# 130. Build Channel

```rust
pub enum ReleaseChannel {
    Dev,
    Alpha,
    Beta,
    Stable,
}
```

---

# 131. Channel Metadata

Embed:

```text
channel
version
commit
```

for diagnostics.

---

# 132. Package Suffix

Dev/beta may use:

```text
applicationIdSuffix
```

allowing side-by-side install.

---

# 133. Release Candidate

```text
cargo xtask android release --channel beta
```

---

# 134. Test Before Package

Release pipeline must run:

```text
cargo fmt --check
cargo clippy
unit tests
protocol tests
Android native tests where available
```

before signing.

---

# 135. Release Gate Ordering

```text
lint
 ↓
unit
 ↓
Rust build
 ↓
native artifact verify
 ↓
Gradle tests
 ↓
package
 ↓
package verify
 ↓
sign
 ↓
final verify
```

---

# 136. Android Unit Tests

Run:

```text
Gradle unit tests
```

for Kotlin/platform bridge.

---

# 137. Instrumentation Tests

On emulator/device:

```text
JNI load
native ABI handshake
basic runtime start
permissions
```

---

# 138. Codec Smoke Tests

For media release:

```text
hardware codec capability
audio native stream
```

at least on hardware lab/nightly.

---

# 139. Emulator Limits

Emulator is useful for:

```text
JNI
basic lifecycle
x86_64
```

not final hardware media validation.

---

# 140. Device Farm

Optional future CI:

```text
real Android devices
```

for release candidate validation.

---

# 141. `run`

```text
cargo xtask android run
```

Flow:

```text
build selected ABI
assemble debug
adb install
adb launch
```

---

# 142. Device ABI Detection

If one device connected:

```text
adb shell getprop ro.product.cpu.abi
```

Then build only that ABI for faster dev.

---

# 143. Emulator Detection

x86_64 emulator:

```text
build x86_64 only
```

---

# 144. Multi-Device Run

Allow:

```text
--device SERIAL
```

---

# 145. Logcat

Optional:

```text
cargo xtask android run --logcat
```

filters Rust/app logs.

---

# 146. Rust Backtrace

Debug builds can enable native backtraces.

---

# 147. Native Crash Symbols

`xtask` can symbolize tombstone/backtrace using stored symbols.

---

# 148. `android symbols`

Command:

```text
cargo xtask android symbols <crash-file>
```

maps native addresses where possible.

---

# 149. Release Artifact Directory

```text
dist/android/<version>/
```

Contents:

```text
app-release.aab
app-release.apk
checksums.txt
build-manifest.json
native-symbols.zip
sbom.*
licenses.*
```

---

# 150. Artifact Naming

Include:

```text
product
version
channel
architecture if split
```

---

# 151. Checksums

Generate:

```text
SHA-256
```

for website release.

---

# 152. SBOM

Generate software bill of materials for:

```text
Rust dependencies
Gradle/Kotlin dependencies
native dependencies
```

---

# 153. License Report

Aggregate:

```text
Cargo license metadata
Android dependencies
```

for release compliance.

---

# 154. Provenance

Build manifest should include:

```text
Git commit
Rust version
NDK version
Gradle version
AGP version
JDK version
features
ABIs
artifact hashes
```

---

# 155. Build Manifest

Example:

```json
{
  "version": "1.0.0",
  "commit": "...",
  "abis": ["arm64-v8a", "x86_64"],
  "rust": "...",
  "ndk": "...",
  "artifacts": {}
}
```

JSON is appropriate for external CI/release tooling.

---

# 156. Rebuild Comparison

Two builds can compare manifests.

---

# 157. Release Upload

`xtask` may prepare artifacts.

Actual upload can be separate.

---

# 158. Website Release

Could provide:

```text
APK
checksum
signature metadata
release notes
```

---

# 159. Play Store Upload

If later automated:

```text
use official Play publishing API/tool
```

behind explicit release command.

Do not embed credentials into repository.

---

# 160. Upload Separation

Safer initial architecture:

```text
build/sign
then
manual Play Console upload
```

until automated deployment is mature.

---

# 161. CI Job Split

Example:

```text
android-check
android-native-arm64
android-native-x86_64
android-package
android-release
```

---

# 162. PR CI

Build:

```text
debug x86_64
arm64 compile check
```

to save time.

---

# 163. Main Branch CI

Build all release ABIs.

---

# 164. Release CI

Full:

```text
tests
AAB
APK
symbols
SBOM
signatures
```

---

# 165. Matrix Builds

Each ABI can be separate CI job.

Final packaging job downloads verified `.so` artifacts.

---

# 166. Cross-Job Artifact Integrity

Package job verifies digest manifest from native jobs.

---

# 167. Avoid Mixing Commits

Every artifact embeds same:

```text
Git SHA
```

Packaging fails if native artifacts came from another commit.

---

# 168. Feature Hash

Native artifact includes/exports:

```text
feature configuration hash
```

Optional strong protection.

---

# 169. Build Cache Poisoning

Do not trust cache merely because filename matches.

Cargo handles dependency fingerprints; release staging additionally checks build metadata.

---

# 170. Clean Build Release

Official release should support:

```text
--clean
```

or run from clean isolated CI checkout.

---

# 171. Hermetic Direction

Long-term:

```text
containerized/pinned toolchain
```

for reproducible Android releases.

---

# 172. Nix/Devcontainer Optional

Can use:

```text
Nix
container
mise/asdf
```

for developer setup, but not required architecture.

Rust `doctor` remains canonical validator.

---

# 173. Windows/macOS Build Hosts

Android build can run from:

```text
Linux
Windows
macOS
```

if toolchains supported.

Official CI can standardize on Linux.

---

# 174. Path Handling

`xtask` must use Rust `PathBuf`.

No Unix-only path string assumptions.

---

# 175. Gradle Wrapper Invocation

Linux/macOS:

```text
./gradlew
```

Windows:

```text
gradlew.bat
```

Abstract in Rust.

---

# 176. Environment Sanitization

Pass only required environment into release subprocess where practical.

---

# 177. Secrets Isolation

Signing secrets only provided to signing/package phase.

Not to ordinary compile/test jobs.

---

# 178. CI Secret Boundary

Unsigned release build can run in untrusted build job.

Signing runs in protected environment.

---

# 179. Two-Stage Release

```text
Stage A:
build unsigned verified artifact

Stage B:
sign/package final artifact
```

---

# 180. Native Code Signing

Android signs APK/AAB package as a whole.

Individual `.so` usually rely on package integrity.

Still hash them in build manifest.

---

# 181. JNI ABI Compatibility

Part 19 ABI version is validated at build/startup.

---

# 182. Kotlin Stub Generation

If JNI wrappers are generated:

```text
xtask generate bindings
```

before compile.

---

# 183. Binding Drift

CI checks generated Kotlin/JNI header/binding files are up to date.

---

# 184. Cbindgen/UniFFI

If used for other bindings:

```text
xtask orchestrates generation
```

but Android JNI may remain custom.

---

# 185. Generated Files Policy

Choose:

```text
committed generated API
or
build-generated
```

and enforce consistently.

---

# 186. Recommended JNI Generation

Keep generated bindings reproducible and diffable.

For release:

```text
regenerate
compare
fail if unexpected dirty diff
```

---

# 187. Android Resources

`xtask` does not replace Android resource processing.

Gradle handles:

```text
XML
icons
strings
manifest merge
```

---

# 188. Dioxus Assets

If Dioxus assets require generation/bundling:

```text
xtask prepares them before Gradle
```

---

# 189. Asset Fingerprinting

Use generated asset manifest.

---

# 190. Native Asset Separation

Do not hide `.so` inside generic assets.

Use `jniLibs`.

---

# 191. Split Install Testing

For AAB, use bundletool to generate/install local device APK set.

---

# 192. Bundle Test

```text
AAB
 ↓
bundletool
 ↓
.apks
 ↓
install
```

CI/nightly can validate.

---

# 193. Universal APK from AAB

Can generate for testing, but direct website APK can also come from Gradle APK task.

---

# 194. MinSdk Enforcement

Rust native code must respect minSdk.

---

# 195. API-Level Compile Flags

NDK target includes API level:

```text
aarch64-linux-android26
```

example.

`xtask` derives from one minSdk source.

---

# 196. ABI-Specific API Level

Avoid accidental inconsistent API levels across targets.

---

# 197. 64-Bit Policy

Play Store requires appropriate 64-bit support for native apps.

Arm64 must be first-class.

---

# 198. 32-Bit Support Decision

Do not carry `armeabi-v7a` forever by default.

It increases:

```text
build time
test matrix
download size
maintenance
```

Enable only if user base requires.

---

# 199. Native Library Size

Track per ABI.

Release guard can warn on major regression.

---

# 200. APK/AAB Size Budget

Track:

```text
total
native
assets
DEX
```

---

# 201. Size Regression Report

Example:

```text
+12 MB native due to new codec
```

---

# 202. AV1 Software Codec Impact

Software AV1 can significantly increase binary size.

Feature/config should make size visible.

---

# 203. Hardware Codec Dependencies

Android hardware codec path generally uses system APIs and does not require shipping codec libraries.

This helps size.

---

# 204. Release Architecture Choice

If software AV1 library is huge:

```text
consider dynamic feature only if Android architecture permits and complexity justified
```

but start simple.

---

# 205. Debug Symbols Size

Do not mistake stripped APK size with symbol archive size.

---

# 206. Build Timing

`xtask` reports durations:

```text
Rust per ABI
Gradle
packaging
tests
```

---

# 207. Performance Optimization

Use incremental builds for dev.

Use clean reproducible environment for release.

---

# 208. Build Cancellation

If one ABI fails:

```text
cancel remaining packaging
```

---

# 209. Error Output

Preserve:

```text
Cargo stderr
Gradle stderr
```

with clear phase header.

---

# 210. No Hidden Retry

Build failures should not be automatically retried repeatedly unless known transient network download phase.

---

# 211. Offline Build

After dependencies/toolchains cached, support:

```text
cargo --offline
Gradle offline
```

for reproducibility/air-gapped use.

---

# 212. Vendoring

Optional enterprise:

```text
cargo vendor
Gradle dependency mirror
```

---

# 213. Supply Chain

Release pipeline should audit:

```text
Cargo.lock
Gradle lockfiles
checksums
SBOM
```

---

# 214. Cargo Audit

Security/dependency scanning can be part of release gate.

Tool choice may evolve.

---

# 215. Gradle Dependency Verification

Enable checksums/signatures where supported.

---

# 216. Forbidden Licenses

Optional project policy can fail release based on dependency license.

---

# 217. Signing Key Rotation

Release tooling must allow new keystore/upload key configuration without code changes.

---

# 218. Key Backup

Signing keys require secure backup/operational process.

Outside source repository.

---

# 219. Debug Signing

Use standard debug key only for dev.

Never website-distribute debug-signed build.

---

# 220. Release Environment Marker

At runtime diagnostics:

```text
release_signed
debug
dev
```

---

# 221. Build Integrity Check at Startup

Optional:

Rust JNI can compare:

```text
app build version
native build version
```

and emit fatal error on mismatch.

---

# 222. Crash Symbol Server

Future self-hosted system can store:

```text
version → symbols
```

for crash reports.

---

# 223. Privacy

Build tooling contains no user data.

Crash symbols are code metadata only.

---

# 224. Android Build Testkit

Create:

```text
xtask tests
```

for:

```text
ABI mappings
version generation
staging
manifest parsing
artifact validation
```

---

# 225. Unit Test Toolchain Discovery

Mock filesystem/environment.

Do not require real NDK for every `xtask` unit test.

---

# 226. Integration Test

CI runs real:

```text
NDK build
Gradle package
```

---

# 227. Stale Artifact Test

Put old `.so` in staging.

Run build.

Expected:

```text
removed/replaced
```

---

# 228. Missing ABI Test

Delete x86_64 library before package.

Expected:

```text
packaging blocked
```

---

# 229. Wrong Profile Test

Stage debug library.

Attempt release package.

Expected:

```text
blocked
```

---

# 230. Wrong Commit Test

Native manifest Git SHA differs.

Expected:

```text
blocked
```

---

# 231. ABI Handshake Test

Package app with intentionally mismatched native ABI version.

Expected:

```text
startup fails clearly
```

---

# 232. Signing Test

Release pipeline without signing secrets:

```text
fail before final signing
```

or produce explicit unsigned artifact only if requested.

---

# 233. Permission Regression Test

Unexpected Android permission appears.

Release guard detects.

---

# 234. Debuggable Test

Inspect APK.

Assert release is not debuggable.

---

# 235. Native Dependency Test

Inspect `.so`.

Assert only approved Android/system libraries are required.

---

# 236. Symbol Test

Ensure symbol archive corresponds to exact native digest.

---

# 237. Reproducibility Test

Build unsigned artifact twice in controlled environment.

Compare:

```text
manifest
native hashes
```

and byte-level outputs where practical.

---

# 238. Release Command

Example:

```text
cargo xtask android release \
  --version 1.4.0 \
  --channel stable \
  --apk \
  --aab
```

---

# 239. Release Flow

```text
validate clean tree
 ↓
validate versions
 ↓
doctor
 ↓
lint/test
 ↓
build all ABIs
 ↓
verify native
 ↓
Gradle release
 ↓
verify package
 ↓
sign
 ↓
verify signature
 ↓
symbols
 ↓
SBOM/licenses
 ↓
checksums
 ↓
dist/
```

---

# 240. Release Confirmation

Before signing stable release, `xtask` can require:

```text
--confirm-release
```

in local mode.

CI can use noninteractive protected job.

---

# 241. Build State Machine

```rust
pub enum AndroidReleaseStage {
    Validate,
    RustBuild,
    NativeVerify,
    AndroidBuild,
    PackageVerify,
    Signing,
    ReleaseMetadata,
    Complete,
}
```

---

# 242. State Logging

Print:

```text
[3/8] NativeVerify
```

for clear build UX.

---

# 243. Dry Run

```text
cargo xtask android release --dry-run
```

Shows:

```text
targets
features
versions
signing mode
commands
```

without signing/upload.

---

# 244. Configuration Precedence

Recommended:

```text
CLI
>
explicit environment
>
project config
>
defaults
```

Document clearly.

---

# 245. Config File

Example:

```text
build/android.ron
```

---

# 246. Example Config

```ron
(
    min_sdk: 26,
    abis: [
        "arm64-v8a",
        "x86_64",
    ],
    native_lib: "comm_android",
    release_features: [
        "android",
        "calls",
        "dtn",
    ],
)
```

---

# 247. No Secrets in RON

Do not store:

```text
keystore password
Play API token
```

in project config.

---

# 248. Build Plugin Architecture

If other projects reuse the communication platform, package build helpers as:

```text
comm-android-build
```

crate.

---

# 249. Reusable Builder Crate

Provides:

```text
ABI mapping
artifact manifest
Gradle handoff
JNI verification
```

External products can reuse it.

---

# 250. Product-Specific xtask

Each product can wrap:

```text
comm-android-build
```

with its own:

```text
package name
features
signing
assets
```

---

# 251. Why This Matters for Reusability

The P2P platform is intended to be reused by other applications.

A reusable Android SDK should not require users to memorize manual NDK commands.

---

# 252. Consumer Build Integration

External product could:

```rust
AndroidBuild::new()
    .native_crate("comm-android-ffi")
    .abis([...])
    .gradle_project("android/")
    .build()?;
```

---

# 253. C ABI SDK Distribution Alternative

Part 19 can also distribute prebuilt Android:

```text
AAR
```

so non-Rust consumers do not compile Rust at all.

---

# 254. Source Integration vs Prebuilt SDK

Two modes:

```text
Source build:
xtask compiles Rust

SDK build:
Gradle consumes official AAR/native libs
```

---

# 255. First-Party App

Use source build for maximum control.

---

# 256. Third-Party SDK Consumer

Prefer:

```text
AAR
```

where appropriate.

---

# 257. AAR Build

`xtask` can produce:

```text
comm-sdk.aar
```

containing:

```text
JNI bridge
native .so per ABI
minimal Kotlin API
```

---

# 258. AAR Verification

Inspect:

```text
classes.jar
jni/
manifest
```

and ABI metadata.

---

# 259. Maven Publication

Future SDK distribution:

```text
Maven repository
```

for Android consumers.

---

# 260. Maven Publishing

Gradle handles publishing artifact format.

`xtask` orchestrates version/validation.

---

# 261. SDK Release Separation

App release and SDK release can be separate commands.

```text
cargo xtask android release-app
cargo xtask android release-sdk
```

---

# 262. Binary Compatibility

Part 19 C ABI version is embedded in AAR metadata.

---

# 263. SDK Consumer Safety

At runtime, wrapper validates ABI.

---

# 264. Documentation Generation

Release can generate:

```text
BUILD_INFO.md
```

from manifest for audit.

---

# 265. Build Troubleshooting

`doctor` should produce actionable output:

```text
NDK 27 found, project requires 28.x
```

not generic failure.

---

# 266. No Automatic Global Mutation

Do not silently modify:

```text
user shell config
global Java
system SDK
```

---

# 267. Local Tool Bootstrap

If project chooses, provide:

```text
tools/
```

download cache.

But user consent is needed for large downloads.

---

# 268. CI Parity

Developer and CI should invoke same:

```text
cargo xtask android ...
```

not maintain separate CI shell implementation.

---

# 269. GitHub/GitLab/etc.

CI config should be thin:

```text
setup cache
setup secrets
cargo xtask android release
```

---

# 270. Forgeyard Integration

If the project later uses its own CI/CD platform, Forgeyard should call the same `xtask`.

The build contract remains tool-neutral.

---

# 271. Release Notes

`xtask` may generate template from Git history, but release notes remain product-level concern.

---

# 272. Artifact Upload Is Separate Capability

Do not couple building tightly to:

```text
GitHub Releases
Play Store
website
```

Adapters can be added.

---

# 273. Build Security Invariants

1. Release `.so` files are never manually staged.
2. All selected ABIs must be present before packaging.
3. Native artifacts must match release profile.
4. Native artifacts and Android wrapper must come from same build/commit.
5. Release feature set is explicit.
6. Android/FFI ABI versions are checked.
7. Signing secrets never enter source control.
8. Release package is verified after signing.
9. Debuggable/developer features cannot silently ship.
10. Gradle cannot package stale release native artifacts without detection.
11. Build tool versions are pinned/validated.
12. final artifacts include checksums/build provenance.
13. CI and local development use the same orchestrator.
14. failed partial ABI builds never become active staging.
15. build caches cannot override artifact fingerprint validation.

---

# 274. Initial Production Scope

Implement first:

```text
xtask Android CLI
doctor
arm64-v8a
x86_64
cargo-ndk/direct NDK build adapter
automatic staging
native artifact manifest
Gradle verification task
debug/release profiles
APK
AAB
runtime native/app ABI handshake
release signing
checksums
symbols
```

Then:

```text
SBOM
license report
AAR SDK
device-run helper
bundletool verification
CI matrix
permission audit
```

Defer initially:

```text
automatic Play Store publishing
complex remote build service
all possible Android ABIs
custom replacement for Gradle/AGP
```

---

# 275. Implementation Phases

## Phase 1 — `xtask`

```text
CLI
config
doctor
tool discovery
```

## Phase 2 — Native Build

```text
ABI matrix
Cargo/NDK
artifact collection
```

## Phase 3 — Staging

```text
generated jniLibs
artifact manifest
stale protection
```

## Phase 4 — Gradle Orchestration

```text
assemble
bundle
verification task
```

## Phase 5 — Runtime Compatibility

```text
build ID
ABI version handshake
```

## Phase 6 — Release

```text
signing
APK
AAB
checksums
symbols
```

## Phase 7 — CI

```text
matrix
cache
artifact digest
protected signing
```

## Phase 8 — SDK

```text
AAR
Maven
third-party consumers
```

## Phase 9 — Hardening

```text
stale artifact tests
wrong profile tests
permission regression
reproducibility
supply-chain checks
```

---

# 276. Definition of Done

Part 27 is complete when:

- a developer can build Android with one `cargo xtask` command
- Rust native libraries are automatically built for all selected ABIs
- no manual `jniLibs` copy is required
- generated native staging is outside source control
- Gradle refuses stale/missing/wrong-profile native libraries
- arm64-v8a is a first-class release ABI
- x86_64 emulator/developer builds work
- the exact NDK/toolchain version is validated
- Rust target and Android ABI mapping is centralized
- debug/dev/profile/release modes map consistently across Cargo and Gradle
- release features are explicit and validated
- app/native ABI versions handshake at runtime
- AAB is produced for Play Store distribution
- signed APK is produced for direct website distribution
- release signing secrets remain outside repository/logs
- package signatures are verified after build
- symbols are archived for native crash diagnosis
- release artifacts receive checksums and provenance metadata
- CI and local builds execute the same Rust orchestrator
- partial or mixed-commit native artifacts cannot be packaged
- automated tests cover stale artifacts, missing ABIs, wrong profiles, JNI mismatch, and release package validation

---

# 277. Relationship to Earlier Parts

Part 27 operationalizes:

```text
19 — C ABI / FFI
20 — Embedded/build portability principles
25 — Android Hardware Surface / Zero-Copy Video
26 — Rust-First Audio DSP / Native Audio
```

It also packages the Android implementation of:

```text
Iroh P2P networking
Dioxus UI
DTN
Bluetooth/Wi-Fi communication
hardware video codecs
Opus audio
emergency functionality
plugin/extension runtime where enabled
```

---

# 278. Final Architecture

```text
                        DEVELOPER / CI
                              │
                              ▼
                       cargo xtask
                              │
             ┌────────────────┼─────────────────┐
             │                │                 │
          Doctor        Rust/NDK Build      Release Policy
             │                │                 │
             │       ┌────────┼────────┐        │
             │       │                 │        │
             │   arm64-v8a         x86_64      │
             │       │                 │        │
             │       └────────┬────────┘        │
             │                │                 │
             │         Native Artifact          │
             │            Manifest              │
             │                │                 │
             └────────────────┼─────────────────┘
                              │
                      Generated jniLibs
                              │
                              ▼
                        Gradle / AGP
                    ┌─────────┴─────────┐
                    │                   │
                   APK                 AAB
                    │                   │
              Website release      Play Store
```

The intended developer experience becomes:

```text
cargo xtask android run
```

for daily work and:

```text
cargo xtask android release --apk --aab
```

for official release.

No step should require:

```text
"remember to run cargo-ndk first"
"copy this .so into jniLibs"
"make sure it is the release one"
```

Those are build-system responsibilities.

---

# 279. Final Principle

The Android build system should be Rust-driven without pretending Android packaging is not an Android concern.

The correct division is:

```text
Rust xtask:
    owns intent
    owns orchestration
    owns validation
    owns native compilation
    owns release policy

Cargo + NDK:
    compile Rust for Android

Gradle / AGP:
    package and sign Android applications

CI:
    invokes the same xtask
```

This gives the project a predictable one-command Android pipeline while preserving compatibility with the official Android ecosystem.

The result is not merely:

```text
"the app can be built"
```

but:

```text
the exact native code,
for the exact ABIs,
with the exact features,
from the exact source revision,
is automatically packaged,
verified,
signed,
and made ready for Play Store and direct distribution.
```

That is the production role of Part 27.
