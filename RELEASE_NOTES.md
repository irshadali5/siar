# Siar v2 Release Notes

We are thrilled to introduce **Siar v2**, a complete architectural rewrite designed to deliver a truly decentralized, serverless, and privacy-first peer-to-peer (P2P) messaging experience.

Siar has transitioned away from hardcoded or centralized discovery mechanics to a fully P2P stack built on top of [Iroh](https://iroh.computer/). This release brings robust cryptography, multi-platform support, real-time media, and offline capabilities.

## 🚀 Key Features

### 1. Serverless Identity & Naming

* **BIP39 Seed Phrase Identity:** Your identity is generated deterministically from a 24-word seed phrase. There are no accounts, phone numbers, or passwords. Your identity (and its derived Ed25519 endpoints and encryption keys) is entirely self-custodied.
* **Global Unique Usernames:** Usernames are now claimed optimistically in an `iroh-docs` CRDT (Conflict-Free Replicated Data Type) registry. The network collectively manages the phonebook without relying on any central server.

### 2. Peer-to-Peer Networking (QUIC)

* **Direct Connections:** Leveraging Iroh, Siar negotiates direct P2P connections using QUIC, with built-in NAT traversal (hole-punching) and relay fallback.
* **ALPN Multiplexing:** Different types of traffic (contact requests, DMs, real-time calls, video frames) run on isolated application-layer protocols over the same connection.

### 3. Offline Mesh Networking

* **LAN & BLE Support:** When the internet is unreachable, Siar automatically falls back to an offline mesh network.
* **Store-and-Forward Routing:** Uses a highly resilient, flooded mesh with TTL (Time-To-Live) routing over UDP broadcast (LAN) and Bluetooth Low Energy (BLE), allowing proximity-based communication.

### 4. Real-Time Audio & Video Calls

* **Audio:** High-fidelity real-time voice calls using the Opus codec over ordered QUIC streams.
* **Video with Asymmetric Negotiation:** Each peer independently negotiates the best codec for its hardware. For example, a Desktop peer may encode using software AV1 (`rav1e`/`dav1d`), while an Android device leverages native hardware encoding (H.264/H.265/AV1 via `AMediaCodec`).

### 5. Privacy & Storage Security

* **Encrypted at Rest:** All local messaging data and metadata are stored in SQLite (`rusqlite`) and encrypted using **SQLCipher**, keyed natively from your derived identity seed.
* **CRDT Metadata:** While heavy data (messages, images) lives securely in your local database, lightweight conversation metadata (group titles, members, disappearing message TTLs) syncs asynchronously via `iroh-docs` when peers come online.
* **Content-Addressed File Transfers:** File transfers bypass the standard message envelope. They are transferred via `iroh-blobs`, which provides BLAKE3-verified streaming, automatic deduplication, and adaptive `zstd` compression.

### 6. Cross-Platform Native Experience

* **Desktop & Mobile:** The user interface is built in Rust using **Dioxus**, providing a native webview shell that compiles to Linux, macOS, and Windows.
* **Android Native Integrations:** The Android build leverages Kotlin for OS-level integration, notably introducing `CallForegroundService` and `RelayForegroundService` to keep audio/video streams and network connections alive even when the app is backgrounded.

---

## 🛠 Developer & Contributor Updates

Siar v2 introduces major structural changes for developers:
* Project split into an explicit Cargo Workspace (`crates/core`, `crates/ui`, `crates/desktop`, `crates/android`).
* Added a new `DEVELOPER_GUIDE.md` for simplified contributor onboarding.
* Adaptive adaptive content-based `zstd` compression logic added to the core networking layer.
* Comprehensive `ARCHITECTURE.md` overhauled to reflect the state of the new P2P mesh and hardware-accelerated video implementations.

Thank you for supporting decentralized, private communication. Clone the repository, compile via `cargo run --release`, and experience a truly serverless messenger!
