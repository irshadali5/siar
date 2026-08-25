# 22 — Developer & Getting Started Guide

> **Target Audience:** Core Contributors, Integration Engineers, Application Developers  
> **Repository:** [`github.com/irshadali5/siar`](https://github.com/irshadali5/siar)

---

## 1. Prerequisites & Environment Setup

SIAR is developed in modern Rust (2021 Edition). Ensure your toolchain is up to date:

```bash
# 1. Install Rust via rustup (version 1.80+ recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
rustup component add clippy rustfmt

# 2. Clone the repository
git clone https://github.com/irshadali5/siar.git
cd siar
```

---

## 2. Compiling the Workspace

The workspace contains 30 crates and 4 applications configured under a single root [`Cargo.toml`](../Cargo.toml):

```bash
# Check compilation across all crates, targets, and tests
cargo check --workspace --tests

# Build optimized release binaries
cargo build --release --workspace

# Run complete automated test suite (330+ unit & integration tests)
cargo test --workspace
```

---

## 3. Running Workspace Applications

### 1. Desktop Application (Dioxus 0.7 GUI)
```bash
# Launch native desktop interface
cargo run -p apps-desktop
```

### 2. Emergency Mesh Node Daemon
```bash
# Run headless background repeater daemon
cargo run -p apps-emergency-node -- --db-path ./mesh-node.db --port 9000
```

### 3. Command-Line Interface (CLI Diagnostics)
```bash
# Inspect available commands
cargo run -p apps-cli -- --help

# Generate a sovereign root keypair
cargo run -p apps-cli -- keygen --output root-identity.key
```

---

## 4. Android Build & Testing

To compile the Android native library and assemble the APK:

```bash
# Set Android NDK path
export ANDROID_NDK_HOME=/path/to/android-ndk

# Build native JNI shared libraries (.so)
cargo build --target aarch64-linux-android --release -p apps-android-rust-jni-glue

# Assemble Android APK via Gradle
cd apps/android
./gradlew assembleDebug
```

---

## 5. Code Quality & Formatting Standards

Before submitting pull requests, verify adherence to codebase standards:

```bash
# Run clippy lints
cargo clippy --workspace --all-targets -- -D warnings

# Check code formatting
cargo fmt --all -- --check

# Check dependency licenses and vulnerabilities
cargo deny check
```
