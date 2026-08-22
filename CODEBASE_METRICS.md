# SIAR Codebase Metrics & Lines of Code Report

> **Last Updated:** `2026-08-22 22:54:51 UTC`

---

## 1. Summary by Language / File Format

| Language / Format | Files | Code Lines (Non-blank) | Blank Lines | Total Lines |
| :--- | :---: | :---: | :---: | :---: |
| **Rust (`.rs`)** | 162 | 23,289 | 2,294 | 25,583 |
| **Kotlin (`.kt`, `.kts`)** | 20 | 2,889 | 269 | 3,158 |
| **Markdown / Specs (`.md`)** | 42 | 72,526 | 35,220 | 107,746 |
| **Configuration (`.toml`, `.yml`, `.xml`, `.properties`)** | 42 | 1,103 | 87 | 1,190 |
| **Legal, Shell & Config (`LICENSE-*`, `.sh`, `.gitignore`)** | 5 | 742 | 145 | 887 |
| **Total Codebase** | **271** | **100,549** | **38,015** | **138,564** |

---

## 2. Breakdown by Architectural Layer & Crates

### Core Domain, Identity & Cryptography

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [siar-domain](crates/siar-domain) | 15 | 1,565 | 1,756 | Core entities, mailboxes, lifecycles, and IDs |
| [siar-crypto](crates/siar-crypto) | 8 | 845 | 956 | Ed25519, X25519, AES-GCM, ChaCha20-Poly1305 |
| [siar-crypto-mls](crates/siar-crypto-mls) | 5 | 783 | 837 | IETF MLS group end-to-end encryption |
| [siar-identity-multidevice](crates/siar-identity-multidevice) | 8 | 651 | 723 | Root keys, certificates, directory & trust stores |
| **Subtotal** | **36** | **3,844** | **4,272** | |

### Protocols & Extension System

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [siar-protocol](crates/siar-protocol) | 9 | 1,033 | 1,122 | Wire codec, frame definitions, envelope format |
| [siar-protocol-ext](crates/siar-protocol-ext) | 8 | 887 | 987 | Capability negotiation, descriptors, registries |
| **Subtotal** | **17** | **1,920** | **2,109** | |

### Storage, Messaging & UI State

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [siar-storage](crates/siar-storage) | 10 | 1,319 | 1,489 | Stoolap DB, message, contact, outbox, blob repos |
| [siar-messaging](crates/siar-messaging) | 7 | 1,716 | 1,870 | Core messaging engine, ticket exchanges, tickets |
| [siar-ui-state](crates/siar-ui-state) | 10 | 887 | 1,014 | Reactive state models, timelines, composers |
| **Subtotal** | **27** | **3,922** | **4,373** | |

### Mesh Networking, Transports & Routing Policy

| Component / Module | Files | Code Lines | Total Lines | Description |
| :--- | :---: | :---: | :---: | :--- |
| [siar-routing-policy](crates/siar-routing-policy) | 14 | 1,454 | 1,597 | Transport routing policy engine, path scoring, multi-path failover |
| [siar-routing](crates/siar-routing) | 7 | 1,420 | 1,547 | Path scoring, multipath transport router |
| [siar-dtn](crates/siar-dtn) | 5 | 562 | 630 | Store-and-forward bundle delivery & dedup |
| [siar-transport](crates/siar-transport) | 8 | 537 | 602 | Iroh transport abstraction & peer pool |
| [siar-transport-ble](crates/siar-transport-ble) | 5 | 546 | 607 | Pure Rust BLE discovery & frame fragmentation |
| [siar-transport-ble-android](crates/siar-transport-ble-android) | 3 | 229 | 246 | Android BLE GATT JNI bridge |
| [siar-transport-bluetooth-classic](crates/siar-transport-bluetooth-classic) | 4 | 379 | 420 | Bluetooth RFCOMM streaming & JNI bridge |
| [siar-transport-wifi-aware](crates/siar-transport-wifi-aware) | 3 | 190 | 205 | Wi-Fi NAN / Aware JNI bridge |
| [siar-transport-wifi-direct](crates/siar-transport-wifi-direct) | 3 | 193 | 208 | Wi-Fi P2P / Direct JNI bridge |
| [siar-connectivity](crates/siar-connectivity) | 3 | 227 | 248 | Link monitor & rank-ordered interface selection |
| **Subtotal** | **55** | **5,737** | **6,310** | |

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
| [apps/android](apps/android) | 25 | 4,251 | 4,571 | Jetpack Compose Kotlin UI + JNI messaging glue |
| [apps/desktop](apps/desktop) | 5 | 1,427 | 1,509 | Desktop GUI app (Dioxus 0.7 + pure Rust) |
| [apps/emergency-node](apps/emergency-node) | 2 | 824 | 863 | Headless off-grid mesh repeater daemon |
| [apps/cli](apps/cli) | 2 | 804 | 871 | Command-line client for dev/diagnostics |
| **Subtotal** | **34** | **7,306** | **7,814** | |

---

## 3. Architecture Specifications & Documentation

| Document Category | Files | Content Lines | Total Lines | Scope |
| :--- | :---: | :---: | :---: | :--- |
| [sys-arch](sys-arch) | 33 | 70,907 | 105,732 | Complete 33-part system architecture specifications |
| Root Docs & Legal (`README`, `CLA`, `CONTRIBUTING`, `LICENSE-*`) | 11 | 2,158 | 2,685 | Setup guides, system evaluations & dual licensing |
| **Subtotal** | **44** | **73,065** | **108,417** | |
