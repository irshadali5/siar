# Siar (v2)

A serverless, cross-platform, peer-to-peer (P2P) messenger built on [Iroh](https://iroh.computer/). Siar provides a modern, Signal/WhatsApp-style messaging experience with true decentralization. Your identity belongs to you via a 24-word seed phrase—no central servers, no phone number requirements.

## 🚀 Features

- **True Decentralization**: No central servers. Everything operates peer-to-peer using Iroh.
- **Seed Phrase Identity**: Generate a 24-word BIP39 mnemonic that deterministically derives all your encryption keys. Never lose your identity.
- **Unique Usernames**: Globally-searchable, unique usernames built on top of a decentralized `iroh-docs` registry.
- **Cross-Platform by Default**: One codebase that compiles to Linux, macOS, Windows (via Dioxus desktop) and Android/iOS.
- **Adaptive Compression**: Messages and files are compressed adaptively via `zstd` only when it provides a size benefit.
- **Secure Local Storage**: Full encryption at rest. Chat history is stored locally using an encrypted `rusqlite` database via SQLCipher.
- **Modern UI**: A responsive, Signal/WhatsApp-style interface built with Dioxus.

## 🏗️ System & Architecture

Siar is designed from the ground up to operate securely in a trustless environment without sacrificing user experience.

### 1. Identity Model (BIP39 + HKDF)
Instead of relying on centralized accounts, Siar uses a 24-word BIP39 seed phrase (256-bit entropy).
- **Why we chose this:** To empower true ownership. A single seed deterministically derives all keys needed (Ed25519 identity keys for Iroh, database encryption keys, and `iroh-docs` author keys) via HKDF-SHA512. The phrase is never saved to disk—only the derived keys are securely persisted locally. You can easily recover your account on a new device with the phrase.

### 2. Networking & P2P Engine (Iroh)
All network communications (messaging, file transfers, discovery) are powered by [Iroh](https://iroh.computer/).
- **Why we chose Iroh:** Iroh provides excellent NAT traversal, hole punching, and secure QUIC connections out-of-the-box. Instead of reinventing custom networking logic, Iroh ensures secure, encrypted, and reliable P2P communication.
- **Messaging:** 1:1 DMs use Direct Iroh connections (via custom ALPNs like `siar/contact/1` and DMs).
- **File Transfers:** Large files skip the DM envelope and are securely streamed using `iroh-blobs`, which provides BLAKE3-verified streaming and deduplication.

### 3. Decentralized Registry (iroh-docs)
Usernames and conversation metadata (group chat titles, DM settings) are synced using `iroh-docs`.
- **Why we chose this:** We needed a way to resolve unique usernames without a central server. `iroh-docs` functions as a replicated, eventually-consistent CRDT key-value store. It allows users to claim a username in a globally synced phonebook. Conflict resolution is handled gracefully without relying on a central authority or a heavy blockchain consensus mechanism.

### 4. Local Storage Engine (rusqlite + SQLCipher)
Messages, file metadata, and contacts are stored in a local SQLite database (`rusqlite`).
- **Why we chose rusqlite over redb:** A chat application’s access patterns (filtering by conversation, paginating by timestamp) are perfectly suited for SQL relational queries. Furthermore, `rusqlite` bundles easily across all target platforms (Desktop & Mobile). 
- **Security:** We use SQLCipher to ensure total encryption at rest. If a device is compromised, chat history remains secure. 
- **Data Segregation:** We explicitly split data storage. Heavy conversation logs stay strictly local (SQLite), while metadata that needs syncing across peers lives in `iroh-docs`.

### 5. UI Framework (Dioxus)
The user interface is built in Rust using [Dioxus](https://dioxuslabs.com/).
- **Why we chose Dioxus:** It allows us to build a rich, responsive frontend using the same language (Rust) as our backend logic. The `desktop` and `mobile` renderers wrap a webview shell, meaning our UI code runs natively across Linux, macOS, Windows, Android, and iOS without maintaining separate forks.

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
*(iOS support requires macOS and Xcode).*
