# SIAR Codebase Metrics & Lines of Code Report

> **Last Updated:** `2026-08-25 07:26:46 UTC`

---

## 1. Summary by Language / File Format

| Language / Format | Files | Code Lines (Non-blank) | Blank Lines | Total Lines |
| :--- | :---: | :---: | :---: | :---: |
| **Rust (`.rs`)** | 195 | 26,715 | 2,673 | 29,388 |
| **Kotlin (`.kt`, `.kts`)** | 20 | 2,889 | 269 | 3,158 |
| **Markdown / Specs (`.md`)** | 52 | 80,614 | 38,947 | 119,561 |
| **Configuration (`.toml`, `.yml`, `.xml`, `.properties`)** | 48 | 1,177 | 92 | 1,269 |
| **Legal, Shell & Config (`LICENSE-*`, `.sh`, `.gitignore`)** | 5 | 743 | 145 | 888 |
| **Total Codebase** | **320** | **112,138** | **42,126** | **154,264** |

---

## 2. Breakdown by Architectural Layer & Crates

### Core Domain, Identity & Cryptography

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [siar-domain](crates/siar-domain) | 15 | 1,565 | 1,756 | Core entities, mailboxes, lifecycles, and IDs |
| [siar-crypto](crates/siar-crypto) | 8 | 845 | 956 | Ed25519, X25519, AES-GCM, ChaCha20-Poly1305 |
| [siar-crypto-mls](crates/siar-crypto-mls) | 5 | 783 | 837 | IETF MLS group end-to-end encryption |
| [siar-identity-multidevice](crates/siar-identity-multidevice) | 14 | 1,454 | 1,602 | Root keys, certificates, directory, pairing handshakes, SAS codes & revocations |
| **Subtotal** | **42** | **4,647** | **5,151** | |

### Protocols & Extension System

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [siar-protocol](crates/siar-protocol) | 9 | 1,033 | 1,122 | Wire codec, frame definitions, envelope format |
| [siar-protocol-ext](crates/siar-protocol-ext) | 12 | 1,336 | 1,484 | Capability negotiation, frame headers, backpressure & fair scheduling |
| **Subtotal** | **21** | **2,369** | **2,606** | |

### Offline Event Log & Robust Blob Subsystem

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [siar-event-log](crates/siar-event-log) | 7 | 508 | 575 | Offline append-only event log, version envelopes, causal gap detection |
| [siar-blob-manifest](crates/siar-blob-manifest) | 11 | 932 | 1,043 | Chunked file transfers, BLAKE3 Merkle tree verification, ChaCha20-Poly1305 AEAD, resumable bitmaps |
| **Subtotal** | **18** | **1,440** | **1,618** | |

### Storage, Messaging & UI State

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [siar-storage](crates/siar-storage) | 10 | 1,319 | 1,489 | Stoolap DB, message, contact, outbox, blob repos |
| [siar-messaging](crates/siar-messaging) | 7 | 1,716 | 1,870 | Core messaging engine, ticket exchanges, tickets |
| [siar-ui-state](crates/siar-ui-state) | 10 | 887 | 1,014 | Reactive state models, timelines, composers |
| **Subtotal** | **27** | **3,922** | **4,373** | |

### Mesh Networking, DTN & Routing Policy

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [siar-routing-policy](crates/siar-routing-policy) | 14 | 1,454 | 1,597 | Transport routing policy engine, path scoring, multi-path failover |
| [siar-dtn-bundle](crates/siar-dtn-bundle) | 8 | 786 | 868 | Store-carry-forward DTN bundle engine, Spray-and-Wait & Epidemic replication |
| [siar-dtn](crates/siar-dtn) | 5 | 562 | 630 | Store-and-forward bundle delivery & dedup |
| [siar-routing](crates/siar-routing) | 7 | 1,420 | 1,547 | Path scoring, multipath transport router |
| [siar-transport](crates/siar-transport) | 8 | 537 | 602 | Iroh transport abstraction & peer pool |
| [siar-transport-ble](crates/siar-transport-ble) | 5 | 546 | 607 | Pure Rust BLE discovery & frame fragmentation |
| [siar-transport-ble-android](crates/siar-transport-ble-android) | 3 | 229 | 246 | Android BLE GATT JNI bridge |
| [siar-transport-bluetooth-classic](crates/siar-transport-bluetooth-classic) | 4 | 379 | 420 | Bluetooth RFCOMM streaming & JNI bridge |
| [siar-transport-wifi-aware](crates/siar-transport-wifi-aware) | 3 | 190 | 205 | Wi-Fi NAN / Aware JNI bridge |
| [siar-transport-wifi-direct](crates/siar-transport-wifi-direct) | 3 | 193 | 208 | Wi-Fi P2P / Direct JNI bridge |
| [siar-connectivity](crates/siar-connectivity) | 3 | 227 | 248 | Link monitor & rank-ordered interface selection |
| **Subtotal** | **63** | **6,523** | **7,178** | |

### Realtime Calls, Media & Codecs

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [siar-calls](crates/siar-calls) | 8 | 759 | 833 | Signaling state machine & adaptive jitter buffer |
| [siar-media-core](crates/siar-media-core) | 9 | 664 | 740 | Audio/video frame types & codec interfaces |
| [siar-media-audio](crates/siar-media-audio) | 5 | 538 | 603 | Opus audio encoder/decoder, capture/playback |
| [siar-media-av1](crates/siar-media-av1) | 4 | 515 | 570 | Software rav1e/dav1d AV1 encoder/decoder |
| [siar-media-image](crates/siar-media-image) | 6 | 393 | 440 | Pure-Rust image compression & thumbnailing |
| [siar-media-android](crates/siar-media-android) | 3 | 504 | 546 | Android MediaCodec hardware JNI glue |
| [platform-android-media](platform/android/media) | 5 | 500 | 564 | Android Kotlin hardware surface & codec bridge |
| **Subtotal** | **40** | **3,873** | **4,296** | |

### Emergency, Diagnostics & Testing

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [siar-emergency](crates/siar-emergency) | 6 | 325 | 355 | SOS signaling, battery-aware duty cycling |
| [siar-testkit](crates/siar-testkit) | 3 | 275 | 310 | Simulated multi-hop network environment |
| [fuzz](fuzz) | 3 | 50 | 58 | Protocol & blob decode fuzzing targets |
| **Subtotal** | **12** | **650** | **723** | |

### Applications & Client Frontends

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [apps/android](apps/android) | 28 | 4,267 | 4,587 | Jetpack Compose Kotlin UI + JNI messaging glue |
| [apps/desktop](apps/desktop) | 5 | 1,427 | 1,509 | Desktop GUI app (Dioxus 0.7 + pure Rust) |
| [apps/emergency-node](apps/emergency-node) | 2 | 824 | 863 | Headless off-grid mesh repeater daemon |
| [apps/cli](apps/cli) | 2 | 804 | 871 | Command-line client for dev/diagnostics |
| **Subtotal** | **37** | **7,322** | **7,830** | |

---

## 3. Architecture Specifications & Documentation

| Document Category | Files | Content Lines | Total Lines | Scope |
| :--- | :---: | :---: | :---: | :--- |
| [sys-arch](sys-arch) | 54 | 107,933 | 162,725 | Complete 33-part system architecture specifications & 21 UI/UX specs |
| [docs](docs) | 6 | 805 | 982 | Master integration plan, UI/UX architecture, off-grid guide, video & image codecs |
| Root Docs & Legal (`README`, `CLA`, `CONTRIBUTING`, `LICENSE-*`) | 11 | 2,510 | 3,112 | System evaluation, status, legal agreements & licenses |
| **Subtotal** | **71** | **111,248** | **166,819** | |
