# SIAR: Secure Interoperable Autonomous Routing & Messaging

[![License: AGPLv3 / Commercial](https://img.shields.io/badge/license-AGPLv3%20%7C%20Commercial-blue.svg)](#license--duality-model)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](#build--compilation-tutorial)
[![Platform Target](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20Android-lightgrey.svg)](#prerequisites--environment-setup)

**SIAR** (*Secure Interoperable Autonomous Routing*) is a zero-infrastructure, multi-transport, offline-first decentralized messaging and delay-tolerant networking (DTN) platform. Designed to operate seamlessly across high-latency, degraded, or completely disconnected environments, SIAR eliminates reliance on central servers, cloud APIs, and traditional cellular/internet infrastructure.

It leverages peer-to-peer (P2P) mesh transports (**Bluetooth LE**, **Bluetooth Classic**, **Wi-Fi Direct**, **Wi-Fi Aware**, **P2P QUIC via Iroh**), strong cryptographic guarantees (**Ed25519**, **X25519**, **ChaCha20-Poly1305**, **MLS 1:1 and Group E2EE**), and pure-Rust zero-copy storage pipelines.

---

## Table of Contents

- [About SIAR](#about-siar)
  - [Core Philosophy](#core-philosophy)
  - [SIAR vs Traditional Messengers](#siar-vs-traditional-messengers)
- [System Architecture](#system-architecture)
  - [High-Level Architecture Diagram](#high-level-architecture-diagram)
  - [Workspace Crate Map](#workspace-crate-map)
- [System Architecture Topics (Specifications Index)](#system-architecture-topics-specifications-index)
- [Prerequisites & Environment Setup](#prerequisites--environment-setup)
- [Build & Compilation Tutorial](#build--compilation-tutorial)
  - [1. Building the Rust Workspace](#1-building-the-rust-workspace)
  - [2. Cross-Compiling Android Native (.so) Libraries](#2-cross-compiling-android-native-so-libraries)
  - [3. Building the Android Application](#3-building-the-android-application)
- [Node Configuration & User Guide](#node-configuration--user-guide)
  - [1. Command-Line Interface (`siar-cli`)](#1-command-line-interface-siar-cli)
  - [2. Desktop Application (`siar-desktop`)](#2-desktop-application-siar-desktop)
  - [3. Headless Emergency Relay Node (`siar-emergency-node`)](#3-headless-emergency-relay-node-siar-emergency-node)
  - [4. Android Messenger App (`apps/android`)](#4-android-messenger-app-appsandroid)
- [Testing & Fuzzing](#testing--fuzzing)
- [License & Contributing](#license--contributing)

---

## About SIAR

### Core Philosophy

1. **Zero Central Dependencies**: Communication must function directly between devices without requiring centralized identity registries, DNS servers, or cloud relays.
2. **Multi-Transport Opportunistic Mesh**: Devices dynamically discover and switch between available physical links—local Wi-Fi Direct, Wi-Fi Aware, BLE, Bluetooth Classic, LAN, and Internet QUIC endpoints—without dropping application-level sessions.
3. **Delay-Tolerant Networking (DTN)**: Messages and attachments automatically store, carry, and forward across nodes in disconnected network partitions until reachability is established.
4. **Cryptographic Autonomy**: Every device self-generates its cryptographic identity (`DeviceIdentity` with Ed25519 signing keys and X25519 key exchange keys). MLS (Messaging Layer Security) provides strong forward secrecy and post-compromise security for 1:1 and group conversations.
5. **Pure-Rust & Native Hardware Acceleration**: High-level abstractions are built in safe, modern Rust (with zero OpenSSL/C runtime dependencies in core crates), while Android platforms utilize zero-copy JNI bridges into `MediaCodec` hardware video/audio encoders.

### SIAR vs Traditional Messengers

| Feature | SIAR | Signal / WhatsApp / Telegram |
| :--- | :--- | :--- |
| **Server Requirement** | **None** (Fully Autonomous P2P / Mesh) | Mandatory Central Cloud Servers |
| **Offline / Disaster Operation** | **Native** (BLE, Wi-Fi Direct/Aware, DTN) | Unusable without Internet |
| **Addressing & Identity** | Self-sovereign Peer Tickets / Endpoint Keys | Phone Numbers / Cloud User IDs |
| **Transport Layer** | Multi-transport adaptive path routing | HTTPS / WebSockets / TCP |
| **Group Security** | OpenMLS (Forward Secrecy + Post-Compromise) | Custom Signal Protocol / Server-managed |
| **Hardware Codec Acceleration** | Native Android `MediaCodec` + pure-Rust DSP | WebRTC / Platform C-libs |

---

## System Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Client Applications
        CLI["apps/cli (Terminal Node)"]
        Desktop["apps/desktop (Slint GUI)"]
        Android["apps/android (Jetpack Compose UI)"]
        Emergency["apps/emergency-node (Headless DTN Relay)"]
    end

    subgraph UI State & Messaging Orchestration
        UIState["crates/siar-ui-state (Framework-Agnostic State)"]
        MessagingService["crates/siar-messaging (MessageService / GroupService)"]
    end

    subgraph Security & Storage Layer
        CryptoMLS["crates/siar-crypto-mls (MLS 1:1 & Group E2EE)"]
        Crypto["crates/siar-crypto (Ed25519 / X25519 / ChaCha20)"]
        Storage["crates/siar-storage (Stoolap SQL Engine)"]
    end

    subgraph Routing & DTN Subsystem
        Routing["crates/siar-routing (PathTable / Link Health Scoring)"]
        DTN["crates/siar-dtn (Store-Carry-Forward Queue)"]
        Connectivity["crates/siar-connectivity (Shared State Engine)"]
    end

    subgraph Multi-Transport Layer
        Transport["crates/siar-transport (Pool / Endpoint Manager)"]
        BLE["crates/siar-transport-ble"]
        WifiDirect["crates/siar-transport-wifi-direct"]
        WifiAware["crates/siar-transport-wifi-aware"]
        BTClassic["crates/siar-transport-bluetooth-classic"]
        IrohP2P["iroh (QUIC P2P / NAT Traversal)"]
    end

    CLI --> UIState
    Desktop --> UIState
    Android --> MessagingService
    Emergency --> DTN

    UIState --> MessagingService
    MessagingService --> CryptoMLS
    MessagingService --> Crypto
    MessagingService --> Storage

    MessagingService --> Routing
    Routing --> DTN
    Routing --> Connectivity
    Routing --> Transport

    Transport --> BLE
    Transport --> WifiDirect
    Transport --> WifiAware
    Transport --> BTClassic
    Transport --> IrohP2P
```

### Workspace Crate Map

SIAR is structured as a modular Rust cargo workspace comprising core domain libraries, hardware bridges, and application binaries:

```
siar/
├── apps/
│   ├── android/                  # Android Native App & JNI Bridges
│   │   ├── app/                  # Jetpack Compose UI (ChatUi, GroupUi, ChatStore)
│   │   ├── messaging-jni/        # Rust JNI cdylib surface (siar-android-messaging)
│   │   ├── rust-jni-glue/        # Shared JNI glue helpers
│   │   └── build-native.sh       # Multi-ABI cargo ndk build automation script
│   ├── cli/                      # Interactive Terminal Messenger & Node
│   ├── desktop/                  # Desktop GUI Application (Slint UI)
│   └── emergency-node/           # Headless Emergency DTN Relay Daemon
├── crates/
│   ├── siar-calls/               # Realtime Media Call Session Protocols
│   ├── siar-connectivity/        # Cross-transport state engine
│   ├── siar-crypto/              # Ed25519/X25519/ChaCha20Poly1305 primitives
│   ├── siar-crypto-mls/          # OpenMLS Group & 1:1 encryption service
│   ├── siar-domain/              # Core domain entities (AccountId, DeviceId, Ticket)
│   ├── siar-dtn/                 # Delay-Tolerant store-carry-forward buffer
│   ├── siar-emergency/           # Priority class queuing & override handler
│   ├── siar-media-android/       # Android MediaCodec JNI hardware surface
│   ├── siar-media-audio/         # Opus audio codec integration (Desktop)
│   ├── siar-media-av1/           # dav1d AV1 video codec integration (Desktop)
│   ├── siar-media-core/          # Frame traits, buffers, and media types
│   ├── siar-media-image/         # Image processing & thumbnail generator
│   ├── siar-messaging/           # MessageService, GroupService, Ticket management
│   ├── siar-protocol/            # Wire frames & Postcard binary codec
│   ├── siar-routing/             # PathTable, Link Health Scoring, classification
│   ├── siar-storage/             # Pure-Rust Stoolap embedded SQL repos
│   ├── siar-testkit/             # Network simulator & mesh test harness
│   ├── siar-transport/           # Transport manager & socket abstraction
│   ├── siar-transport-ble/       # Bluetooth Low Energy protocol implementation
│   ├── siar-transport-ble-android/ # Android BLE Scanner/Advertiser JNI bridge
│   ├── siar-transport-bluetooth-classic/ # BT RFCOMM transport bridge
│   ├── siar-transport-wifi-aware/ # Wi-Fi Aware (NAN) P2P transport bridge
│   ├── siar-transport-wifi-direct/ # Wi-Fi Direct (P2P) transport bridge
│   └── siar-ui-state/            # Framework-agnostic UI state machines
├── sys-arch/                     # 33 Comprehensive System Architecture Specs
└── fuzz/                         # Cargo Fuzz targets for wire/codec parsing
```

---

## System Architecture Topics (Specifications Index)

The `sys-arch/` directory contains detailed design documents defining every architectural subsystem:

| Document | Topic | Description |
| :--- | :--- | :--- |
| [`01-protocol-extension`](sys-arch/01-protocol-extension-system-architecture.md) | **Protocol Extensions** | TLV frame framing and extensible wire header specifications |
| [`02-multi-device-identity`](sys-arch/02-multi-device-identity-architecture.md) | **Multi-Device Identity** | Ed25519 master keys, subkey derivation, device provisioning |
| [`03-transport-routing-policy`](sys-arch/03-transport-routing-policy-engine-architecture.md) | **Routing Engine** | Cost metrics, latency/bandwidth scoring, link quality selection |
| [`04-offline-event-log`](sys-arch/04-offline-event-log-architecture.md) | **Offline Log** | Append-only event log, sequence vectors, state synchronization |
| [`05-robust-file-blob`](sys-arch/05-robust-file-blob-subsystem-architecture.md) | **Blob Subsystem** | Chunking, BLAKE3 content-addressable storage, AES-GCM encryption |
| [`06-dtn-store-carry-forward`](sys-arch/06-dtn-store-carry-forward-architecture.md) | **DTN Core** | Opportunistic epidemic routing, TTL expiry, anti-entropy sync |
| [`07-capability-negotiation`](sys-arch/07-capability-negotiation-architecture.md) | **Capabilities** | Version handshake, media codec negotiation, link parameters |
| [`08-resource-limits`](sys-arch/08-resource-limits-backpressure-architecture.md) | **Backpressure** | Flow control, queue bounds, priority dropping under congestion |
| [`09-crash-recovery`](sys-arch/09-crash-recovery-architecture.md) | **Crash Safety** | WAL recovery, transactional checkpoints, corrupt state isolation |
| [`10-fuzzing-protocol`](sys-arch/10-fuzzing-protocol-test-suite-architecture.md) | **Fuzz Testing** | AFL/libFuzzer strategies for Postcard frames and video blobs |
| [`11-relay-infrastructure`](sys-arch/11-relay-self-hosted-infrastructure-architecture.md) | **Relay Nodes** | Unlinkable token-mailbox relaying and TURN/DERP traversal |
| [`12-multipath-networking`](sys-arch/12-multipath-networking-architecture.md) | **Multipath** | Concurrent multi-socket striping across Wi-Fi + BLE + Cellular |
| [`13-battery-aware`](sys-arch/13-battery-aware-scheduling-architecture.md) | **Power Management** | Battery level polling, BLE duty cycle adjustment, wake lock management |
| [`14-proximity-abstraction`](sys-arch/14-proximity-abstraction-architecture.md) | **Proximity** | Unified API for BLE RSSI, Wi-Fi Aware distance, and LAN discovery |
| [`15-qr-nfc-bootstrap`](sys-arch/15-qr-nfc-bootstrap-pairing-architecture.md) | **Out-of-Band Pairing** | QR code / NFC payload format for PeerTicket exchange |
| [`16-daemon-headless`](sys-arch/16-daemon-headless-runtime-architecture.md) | **Headless Runtime** | UNIX domain socket / IPC control surface for background daemons |
| [`17-emergency-priority`](sys-arch/17-emergency-priority-classes-architecture.md) | **Emergency Priority** | High-priority SOS broadcast preemption over normal traffic |
| [`18-network-diagnostics`](sys-arch/18-network-diagnostics-path-visualization-architecture.md) | **Path Diagnostics** | Route tracing, RTT measurement, graph visualization metrics |
| [`19-c-abi-ffi`](sys-arch/19-c-abi-ffi-architecture.md) | **C/FFI Layer** | Stable C ABI header definitions for iOS/C++ integration |
| [`20-embedded-linux`](sys-arch/20-embedded-linux-node-architecture.md) | **Embedded Linux** | Low-footprint compilation flags for Raspberry Pi / OpenWrt |
| [`25-android-hardware-surface`](sys-arch/25-android-direct-hardware-surface-zero-copy-media-architecture.md) | **Android Hardware Media** | Zero-copy `Surface` / `GraphicBuffer` pipeline into MediaCodec |
| [`28-production-security`](sys-arch/28-production-security-e2ee-key-management-privacy-architecture.md) | **Security & Privacy** | Threat model, identity blinding, MLS key package rotation |

---

## Prerequisites & Environment Setup

### Required Toolchains

1. **Rust Platform**:
   - Rust **1.91.0** or newer (`cargo`, `rustc`).
   - Installation:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     rustup target add x86_64-unknown-linux-gnu aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
     ```

2. **Android Development (for `apps/android`)**:
   - **Android SDK** (API Level 34+ build-tools, `minSdk` 26).
   - **Android NDK** (`r25b` or newer).
   - **`cargo-ndk`**:
     ```bash
     cargo install cargo-ndk
     ```
   - Set environment variables:
     ```bash
     export ANDROID_HOME=$HOME/Android/Sdk
     export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653 # Adjust path to your NDK version
     ```

3. **System Dependencies (Linux GUI / Build Tools)**:
   - On Debian/Ubuntu:
     ```bash
     sudo apt-get update
     sudo apt-get install -y build-essential pkg-config libssl-dev libfontconfig1-dev libx11-dev libasound2-dev
     ```

---

## Build & Compilation Tutorial

### 1. Building the Rust Workspace

To build all core crates, CLI, desktop app, and emergency node:

```bash
# Check compilation across all workspace crates
cargo check --workspace

# Build all binaries in debug mode
cargo build --workspace

# Build optimized release binaries
cargo build --workspace --release
```

The compiled binaries will be located under `target/release/`:
- `target/release/siar-cli`
- `target/release/siar-desktop`
- `target/release/siar-emergency-node`

### 2. Cross-Compiling Android Native (.so) Libraries

The Android app relies on `siar-android-messaging` JNI shared libraries compiled for 4 target ABIs (`arm64-v8a`, `armeabi-v7a`, `x86`, `x86_64`).

Run the automated script:

```bash
cd apps/android
./build-native.sh
```

This script invokes `cargo ndk` for the 7 Android-relevant workspace crates and places the output `.so` files directly in:
`apps/android/app/src/main/jniLibs/<abi>/`

### 3. Building the Android Application

With the `.so` libraries in place, assemble the APK using Gradle:

```bash
cd apps/android

# Build Debug APK
./gradlew assembleDebug

# Install directly on a connected device/emulator
./gradlew installDebug
```

---

## Node Configuration & User Guide

### 1. Command-Line Interface (`siar-cli`)

`siar-cli` provides an interactive terminal interface for managing identities, tickets, direct messaging, and anonymous token mailboxes.

#### Startup & Identity Initialization

```bash
# Start CLI node (automatically initializes persistent identity under OS data directory)
siar-cli
```

#### CLI Operations

```text
SIAR Interactive Messenger
--------------------------
1. Show My Peer Ticket
2. Add Contact Peer Ticket
3. Send 1:1 Direct Text
4. Send Anonymous Mailbox Message
5. Check Token Mailbox
6. Exit
```

1. **Show My Peer Ticket**: Displays your Base64-encoded `PeerTicket`. Share this ticket out-of-band (via QR code or text) with your contact.
2. **Add Contact**: Paste your contact's `PeerTicket`. The CLI decodes the ticket and resolves the peer's public key and endpoint address.
3. **Send Direct Text**: Type a text message. The system checks `PathTable` for active direct routes (`LocalLan`, `InternetDirect`, BLE) and sends the message.
4. **Send Anonymous Mailbox Message**: Delivers a message to a relay node using an unlinkable single-use token token mailbox path.

### 2. Desktop Application (`siar-desktop`)

The desktop app features a modern GUI built with **Slint**.

```bash
cargo run --bin siar-desktop
```

#### Desktop Features:
- **Contact Roster**: Manage peer tickets, online reachability status, and link health.
- **MLS Group Conversations**: Create 1:1 and multi-member MLS groups with forward secrecy.
- **Media Attachments**: Drag-and-drop file sharing with automatic chunking and BLAKE3 content validation.

### 3. Headless Emergency Relay Node (`siar-emergency-node`)

The emergency node runs headless on Linux, Raspberry Pi, or server infrastructure to provide store-carry-forward DTN relaying during disaster scenarios.

```bash
# Launch emergency relay node
cargo run --bin siar-emergency-node
```

#### Configuration & Behavior:
- Automatically listens on all local network interfaces and available Bluetooth/BLE adapters.
- Maintains an in-memory & on-disk DTN store-carry-forward queue for offline messages.
- Periodically pings nearby nodes, updates link quality metrics (`rtt_millis`, `reliability`), and flushes pending queues when a route becomes available.

### 4. Android Messenger App (`apps/android`)

The Android application features a **Jetpack Compose UI** integrated via JNI to the Rust core engine.

#### Setup & Permissions:
1. Launch the app on your device.
2. Grant requested permissions:
   - **Bluetooth & Nearby Devices**: Required for BLE and Bluetooth Classic scanning/advertising (`BLUETOOTH_SCAN`, `BLUETOOTH_CONNECT`, `NEARBY_WIFI_DEVICES`).
   - **Location**: Mandatory on Android 12 and below for physical proximity peer discovery (`ACCESS_FINE_LOCATION`, `ACCESS_COARSE_LOCATION`).

#### Messaging Operations:
- **Chats Tab**: View 1:1 conversations, add contacts via Peer Tickets, send text, and attach media files using the system file picker.
- **Groups Tab**: Create MLS groups, send group invites, and participate in multi-device encrypted group threads.

---

## Testing & Fuzzing

### Unit & Integration Tests

Run the full workspace test suite:

```bash
cargo test --workspace
```

Run mesh network simulation tests in `siar-testkit`:

```bash
cargo test -p siar-testkit
```

### Fuzzing

Fuzz targets are defined under `fuzz/` to test wire decoding and video frame parsing:

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Fuzz frame decoder target
cargo fuzz run decode_frame

# Fuzz blob frame decoder target
cargo fuzz run decode_blob_frame
```

---

## License & Duality Model

SIAR is dual-licensed under the **GNU Affero General Public License Version 3 (AGPLv3)** and the **SIAR Commercial License & Enterprise Exemption Agreement (SIAR-CEEL-1.0)**.

### Why Dual AGPLv3 + Commercial License?

Under permissive licenses (such as MIT or Apache 2.0), commercial corporations and big-tech entities can incorporate open-source mesh routing, cryptographic protocols, and core engine code into proprietary, revenue-generating SaaS platforms and embedded closed-source products without contributing back to the project maintainers or compensating the open-source creators.

To prevent uncompensated enterprise exploitation while preserving 100% freedom for the open-source community:

1. **GNU AGPLv3 (Open Source & Non-Commercial)**:
   - Gratis for non-commercial projects, academic research, open-source developers, emergency responders, and GPL/AGPL-compliant applications.
   - **Section 13 (Network Copyleft Trigger)**: Any cloud service, SaaS application, or network node running SIAR must release its complete corresponding source code under AGPLv3 to all network users.
   - Full text available in [`LICENSE-AGPLv3`](file:///home/irshad/Projects/siar/LICENSE-AGPLv3).

2. **SIAR Commercial License (SIAR-CEEL-1.0)**:
   - Designed for commercial enterprise integration, closed-source application bundling (iOS/Android/Desktop), cloud SaaS deployment, and big-tech infrastructure.
   - **Lifts AGPLv3 Section 13 Network Copyleft**: Allows embedding, static/dynamic linking, and cloud hosting without disclosing proprietary application code or infrastructure backend sources.
   - **Conditioned on Active Recurring Subscription**: Granted in exchange for monthly or annual commercial licensing fees.
   - Full legal contract available in [`LICENSE-COMMERCIAL.md`](file:///home/irshad/Projects/siar/LICENSE-COMMERCIAL.md).

### Commercial Subscription Tiers

| Tier | Revenue / Eligibility | Annual Fee | Exemption Scope |
| :--- | :--- | :--- | :--- |
| **Startup Tier** | Revenue < $2M USD | **$2,400 / yr** ($200/mo) | Closed-source embedding up to 50k devices |
| **Business Tier** | Revenue $2M - $20M USD | **$12,000 / yr** ($1,000/mo) | Closed-source embedding up to 500k devices |
| **Enterprise / Cloud Tier** | Revenue > $20M USD / Big-Tech | **Custom Quote** | Unlimited deployments, cloud SaaS, SLA & dedicated support |

To activate a Commercial Exemption or inquire about Enterprise Licensing, contact `licensing@siar.network`.

---

### Guidelines for Contribution:
1. All contributions are governed by the Contributor License Agreement (CLA) granting the SIAR maintainers dual-licensing rights.
2. Maintain zero-warning clean compilation across all targets (`cargo check --workspace`).
3. Preserve strict boundary isolation: Core crates under `crates/` must remain pure-Rust without mandatory C-library linkages.
4. Ensure platform-specific hardware integration is strictly isolated in dedicated crates (`siar-media-android`, `siar-transport-ble-android`).

---
*Built with Rust, Kotlin, and OpenMLS by the SIAR Open Source Engineering Team.*
