# Siar Developer Onboarding Guide

Welcome to the Siar codebase! This document is designed to get new developers and contributors up to speed with how the project is structured, the critical technologies we use, and how to successfully build and modify the app.

Siar is a fully decentralized, serverless, peer-to-peer (P2P) messenger built on the **Iroh** networking stack and the **Dioxus** cross-platform UI framework.

---

## 1. Project Structure

The project is organized as a Cargo workspace with multiple crates, separating core networking logic from UI and platform-specific wrappers:

- **`crates/core/`**: The brain of the application. Contains all P2P networking, Iroh CRDT synchronization, SQLite database interactions, cryptographic identity management, and the `App` state machine.
- **`crates/ui/`**: The frontend. Written in Rust using Dioxus. It handles rendering the Signal/WhatsApp-like UI, state management, and user interactions.
- **`crates/desktop/`**: The desktop platform wrapper. It binds the `core` and `ui` crates together into a Wry-based webview for Linux, macOS, and Windows.
- **`crates/android/`**: The Android platform wrapper. Contains JNI/Kotlin bridges (like the `CallForegroundService`) to handle native mobile lifecycles.
- **`scripts/`**: Contains utility scripts, notably `build-android.sh` for compiling the Android APK.

---

## 2. Core Technologies & Dependencies

To effectively contribute to Siar, you should be familiar with the following Rust crates and concepts:

### Networking & P2P
- **[Iroh](https://iroh.computer/)**: The absolute core of Siar. We use Iroh for:
  - **Connections**: Direct P2P QUIC connections with built-in NAT traversal (hole punching) and relay fallbacks.
  - **iroh-docs**: A CRDT key-value store. Used to build our completely serverless username registry (optimistic claims) and synchronize conversation metadata (group chat titles, members, DM settings).
  - **iroh-blobs**: Content-addressed blob storage (BLAKE3). Used exclusively for out-of-band file transfers.
  - **iroh-gossip**: Pub/sub broadcasting used for multi-user chat rooms.

### Frontend
- **[Dioxus](https://dioxuslabs.com/)**: A React-like UI framework for Rust. We use its desktop and mobile renderers, which both wrap a webview. You don't need to write HTML/JS, but you will write Dioxus' `rsx!` macros.

### Storage & Cryptography
- **rusqlite & SQLCipher**: We use SQLite for the durable message log and local contact lists. The database is encrypted at rest using SQLCipher (`bundled-sqlcipher-vendored-openssl`).
- **bip39 & ed25519-dalek**: Identity is generated from a 24-word seed phrase. We use HKDF-SHA512 to derive multiple deterministic keys (Iroh endpoint ID, storage encryption keys, etc.) from this single seed.

### Media & Calls
- **Audio**: Opus codec over QUIC streams.
- **Video**: Asymmetric codec negotiation.
  - Desktop uses `rav1e` (encode) and `dav1d` (decode) for AV1.
  - Android probes native hardware via `AMediaCodec` for H.264/H.265/AV1.

---

## 3. The Architecture in a Nutshell

Before modifying the code, understand these constraints and design choices:
1. **No Servers**: Everything is peer-to-peer. Usernames are not guaranteed by a central authority; they are claimed optimistically in an `iroh-docs` registry.
2. **Offline Mesh**: Siar implements a custom flooded mesh over Bluetooth LE and Local Area Network (UDP broadcast) when the internet is unreachable.
3. **Data Split**: Heavy data (messages, images) lives in the local `rusqlite` database. Light metadata that needs to survive offline syncing (e.g., room names) lives in `iroh-docs`.
4. **Android Background Limits**: Standard mobile OSs kill background P2P apps. We use Kotlin Foreground Services (`CallForegroundService`) to keep the network and audio/video pipelines alive while the app is backgrounded.

---

## 4. Building the Project

### Desktop (Linux / macOS / Windows)
The default build target is the desktop webview. It is straightforward:
```bash
# Run the application locally
cargo run --release

# Run with verbose logging to debug Iroh/Core issues
RUST_LOG=info cargo run --release
```

### Android
Building for Android requires the Android NDK, SDK, and Dioxus CLI (`dx`).
We use a wrapper script to handle Gradle and Dioxus interactions:

1. Ensure `dx` is installed (`cargo install dioxus-cli`).
2. Ensure `$ANDROID_SDK_ROOT` and `$ANDROID_NDK_ROOT` are set.
3. Run the build script:
```bash
./scripts/build-android.sh
```
This script will produce a signed release APK (`siar-android-release.apk`) in the root directory.

---

## 5. Adding New Features (A Workflow Example)

If you are adding a new feature (e.g., "Message Reactions"):
1. **Schema Update**: Start in `crates/core/src/store.rs`. Add the SQLite schema migration.
2. **Protocol Update**: Modify `crates/core/src/protocol/message.rs` (the `Envelope`) to carry the reaction over the wire.
3. **Core Logic**: Update `crates/core/src/app.rs` to handle incoming reactions, inserting them into the database and triggering a UI event.
4. **UI Integration**: Finally, go to `crates/ui/src/` (e.g., `chat.rs`) to render the reaction using Dioxus state.

## 6. Testing

- Keep `cargo clippy` happy.
- Run tests in the core crate: `cargo test -p siar-core`
- Note: P2P network testing often requires running two instances on different machines or utilizing distinct data directories to prevent database locking and identity collisions.

Welcome aboard! If you have architectural questions, always refer to `ARCHITECTURE.md` as the source of truth for design decisions.
