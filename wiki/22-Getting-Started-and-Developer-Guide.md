# 22 — Developer & Getting Started Guide

> **Target Audience:** Core Contributors, Integration Engineers, Application Developers  
> **Repository:** [`github.com/irshadali5/siar`](https://github.com/irshadali5/siar)

---

## 1. Prerequisites & Environment Setup

SIAR is developed in modern Rust (2021 Edition, pinned to `rust-version = "1.91"`).

### Option A: Hermetic Nix Flake (Recommended)
SIAR includes a hermetic Nix environment (`flake.nix`, `shell.nix`) providing compiler toolchains and all native GUI/audio libraries (GTK3, WebKit2GTK, ALSA, OpenSSL, CMake):

```bash
# Enter the fully provisioned dev environment
nix develop

# Or build binaries directly
nix build .#siar-cli
nix build .#siar-desktop
nix build .#siar-emergency-node

# Run Nix check suite
nix flake check
```

### Option B: Manual Toolchain Setup
```bash
# 1. Install Rust via rustup (Rust 1.91.0 or newer required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
rustup component add clippy rustfmt

# 2. Clone the repository
git clone https://github.com/irshadali5/siar.git
cd siar
```

---

## 2. Compiling the Workspace

The workspace contains **33 crates** and **4 applications** configured under a single root [`Cargo.toml`](../Cargo.toml):

```bash
# Check compilation across all crates, targets, and tests
cargo check --workspace --tests

# Build optimized release binaries
cargo build --release --workspace

# Run complete automated test suite
cargo test --workspace

# Run multi-node end-to-end integration test
cargo test -p siar-messaging --test end_to_end
```

---

## 3. Running Workspace Applications

### 1. Desktop Application (Dioxus 0.7 GUI)
```bash
# Launch native desktop interface
cargo run --bin siar-desktop
```

### 2. Emergency Mesh Node Daemon
```bash
# Run headless background repeater daemon
cargo run --bin siar-emergency-node
```

### 3. Command-Line Interface (Interactive Terminal Messenger)
```bash
# Launch interactive CLI node
cargo run --bin siar-cli
```

---

## 4. Android Build & Testing

To compile the Android native library across ABIs and assemble the APK:

```bash
# Run automated multi-ABI build script
cd apps/android
./build-native.sh

# Assemble Android APK via Gradle
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
