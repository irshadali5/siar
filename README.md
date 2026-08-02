# Siar (v0.5.0)

Siar is a serverless, cross-platform peer-to-peer (P2P) messenger built on the [Iroh](https://iroh.computer/) networking stack. It provides a secure, fully decentralized alternative to Signal and WhatsApp by eliminating central servers, phone numbers, and centralized identity providers.

> [!WARNING]
> **AI-Assisted Development & Security Notice**
>
> This project has been heavily developed and iterated on with **AI assistance**. While care has been taken in designing the system and architecture, **please review the codebase carefully before using it in sensitive environments**.
>
> We strongly welcome all community contributions, security audits, and bug fixes! Please see [CONTRIBUTING.md](CONTRIBUTING.md) to learn how you can contribute.


## ⚙️ Technical Architecture Overview

### 1. Peer-to-Peer Networking (Iroh & QUIC)

At its core, Siar communicates directly peer-to-peer over **QUIC** (via Iroh). Iroh provides built-in NAT traversal, hole-punching, and relayed fallback connections.

- **Application-Layer Protocol Negotiation (ALPN):** We multiplex entirely different sub-protocols over single QUIC connections:
  - `siar/contact/1`: Contact request/accept signaling.
  - `iroh-messenger/dm/2`: 1:1 direct messaging stream.
  - `iroh-messenger/call/3`: Real-time audio/video call signaling and audio streams.
  - `iroh-messenger/video/2`: Dedicated connection for video frames to avoid stream-acceptance races with audio.
  - `iroh-gossip`: Group chat broadcasting.

### 2. Mesh Networking (LAN & BLE)

When the public internet (or relay discovery) is completely unreachable, Siar automatically falls back to a custom store-and-forward mesh network.

- **Transports:** UDP Broadcast for Local Area Networks (LAN) and Bluetooth Low Energy (BLE) for local proximity.
- **Routing:** Instead of maintaining a complex topology graph, Siar uses a **flooded mesh with TTL (Time-to-Live)**. Every node that sees an unseen `Envelope` re-broadcasts it, decrementing the TTL. This is highly resilient to topology churn (peers joining/leaving rapidly) and works well for phone-scale meshes.

### 3. Identity Model (BIP39 & HKDF)

Identity is strictly decentralized and deterministic.

- A 24-word **BIP39 mnemonic** provides 256 bits of entropy.
- **HKDF-SHA512** expands this seed into multiple isolated keys:
  - `Ed25519` keypair for the Iroh network `EndpointId`.
  - Database encryption keys for local storage.
  - `iroh-docs` Author IDs for publishing to the global registry.
- The mnemonic is never persisted to disk. Only the derived keys are stored securely.

### 4. Decentralized Registry (iroh-docs CRDT)

Since there are no centralized servers to handle "usernames", Siar uses `iroh-docs`—a replicated, eventually-consistent CRDT key-value store.

- **Username Claims:** Users broadcast a claim to a well-known global namespace. Conflicts (race conditions for the same username) are resolved optimistically by timestamp.
- **Conversation Metadata:** `iroh-docs` is also used to sync group chat metadata (members, titles) and DM settings (disappearing message TTLs) asynchronously when peers come back online.

### 5. Media & Real-Time Communications

- **Audio:** Opus codec over ordered QUIC uni streams.
- **Video:** AV1 via software (`rav1e`/`dav1d`) on desktop, and negotiated hardware codecs (H.264/H.265/AV1) on Android via native `AMediaCodec` probing.
- **Codec Negotiation:** Uniquely, encode/decode capabilities are negotiated asymmetrically. A desktop might encode AV1 while an Android peer encodes H.265, optimizing for battery and hardware-decode availability on each side.

### 6. Local Storage Engine (SQLite & SQLCipher)

- Heavy messaging data and binary attachments are strictly kept out of CRDTs. They are stored locally using `rusqlite`.
- **Encryption at Rest:** The SQLite database is encrypted via **SQLCipher**, keyed using a derived HKDF branch of the identity seed.
- **Content-Addressing:** Large files are transferred out-of-band via `iroh-blobs`, which provides BLAKE3-verified streaming and deduplication, keeping the SQLite database small.

### 7. UI and Native Integrations (Dioxus)

The UI is written in Rust using [Dioxus](https://dioxuslabs.com/), rendering to a webview shell.

- **Desktop:** Compiles to macOS, Linux, and Windows natively.
- **Android:** Integrates directly with Android APIs via Kotlin. Features native Foreground Services (`CallForegroundService`, `RelayForegroundService`) to maintain network connections and audio/video pipelines even when the app is backgrounded.

## 🛠️ Building & Running

Ensure you have Rust installed via `rustup`.

### Desktop

```bash
cargo run --release
```

### Android

Siar requires Android build tools.

```bash
dx build --platform android
```

## 📄 License

This project is licensed under the **GNU Affero General Public License v3.0** (AGPL-3.0). See the [LICENSE](LICENSE) file for details.

