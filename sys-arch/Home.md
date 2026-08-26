# SIAR System Architecture & Technical Wiki

> **SIAR**: Secure, Interoperable, Autonomous & Resilient Peer-to-Peer Communication Platform.
> Built in Rust for mobile, desktop, headless daemons, and embedded Linux nodes.

Welcome to the **SIAR System Architecture Wiki**. This knowledge portal contains **60 comprehensive technical specifications** covering all aspects of SIAR's low-level cryptographic protocols, delay-tolerant mesh networking, distributed storage, audio DSP pipelines, cross-platform client architecture, and user experience design.

---

## ⚡ Quick Navigation Matrix

| Domain | Parts | Key Focus Areas |
| :--- | :--- | :--- |
| **I. Foundation & Extensions** | [01](01-protocol-extension-system-architecture.md), [07](07-capability-negotiation-architecture.md), [21](21-third-party-protocol-extensions-architecture.md), [22](22-wasm-compatible-components-architecture.md), [23](23-external-interoperability-suite-architecture.md), [24](24-plugin-module-ecosystem-architecture.md) | Extensibility, WASM plugins, capability negotiation, interop suites |
| **II. Identity & Security** | [02](02-multi-device-identity-architecture.md), [28](28-production-security-e2ee-key-management-privacy-architecture.md) | Multi-device key trees, MLS / Double Ratchet, forward secrecy, audit logs |
| **III. Mesh & Transports** | [03](03-transport-routing-policy-engine-architecture.md), [11](11-relay-self-hosted-infrastructure-architecture.md), [12](12-multipath-networking-architecture.md), [13](13-battery-aware-scheduling-architecture.md), [14](14-proximity-abstraction-architecture.md), [15](15-qr-nfc-bootstrap-pairing-architecture.md) | Iroh QUIC, BLE, Wi-Fi Direct, LoRa, relays, multipath link bonding |
| **IV. DTN & Emergency Mesh** | [06](06-dtn-store-carry-forward-architecture.md), [17](17-emergency-priority-classes-architecture.md) | Store-carry-forward bundle routing, hop limits, SOS priority mesh |
| **V. Storage & Event Log** | [04](04-offline-event-log-architecture.md), [05](05-robust-file-blob-subsystem-architecture.md), [09](09-crash-recovery-architecture.md), [32](32-search-indexing-local-knowledge-privacy-architecture.md), [33](33-backup-restore-export-import-archival-portability-architecture.md) | Append-only outbox, chunked blob sync, SQLite/Redb, Tantivy search |
| **VI. Realtime Media & Audio** | [25](25-android-direct-hardware-surface-zero-copy-media-architecture.md), [26](26-rust-first-audio-dsp-resampling-aec-ns-agc-architecture.md), [29](29-realtime-calls-media-session-protocol-architecture.md) | Low-latency audio/video calls, zero-copy surfaces, Rust DSP (AEC, NS, AGC) |
| **VII. Ephemeral & Presence** | [30](30-presence-availability-typing-read-receipts-ephemeral-state-architecture.md), [31](31-notifications-push-background-delivery-lifecycle-architecture.md) | Ephemeral presence, typing indicators, unified push wake & background delivery |
| **VIII. Native Daemons & FFI** | [16](16-daemon-headless-runtime-architecture.md), [19](19-c-abi-ffi-architecture.md), [20](20-embedded-linux-node-architecture.md), [27](27-rust-driven-android-native-build-packaging-automation.md) | Headless daemons, C-ABI bindings, embedded Linux, Android NDK packaging |
| **IX. Testing & Diagnostics** | [08](08-resource-limits-backpressure-architecture.md), [10](10-fuzzing-protocol-test-suite-architecture.md), [18](18-network-diagnostics-path-visualization-architecture.md) | Backpressure queues, AFL++/libFuzzer harnesses, live path visualizer |
| **X. UI/UX Architecture** | [UI/UX 01](ui-ux-01-product-foundation-cross-platform-interaction-architecture.md) – [UI/UX 27](ui-ux-27-ui-testing-screenshot-interaction-release-quality-gates-architecture.md) | Desktop Dioxus, Android Jetpack Compose, design tokens, responsive layouts |

---

## 🎯 Role-Based Reading Paths

Depending on your engineering focus, we recommend following these reading paths:

### 🛡️ 1. Cryptography & Security Architects
1. [02 — Multi-Device Identity & Trust Model](02-multi-device-identity-architecture.md)
2. [28 — Production Security, E2EE, Key Management & Privacy](28-production-security-e2ee-key-management-privacy-architecture.md)
3. [15 — Out-of-Band QR & NFC Bootstrap Pairing](15-qr-nfc-bootstrap-pairing-architecture.md)
4. [UI/UX 15 — Security Center, Devices, Keys & Recovery](ui-ux-15-security-center-devices-keys-recovery-architecture.md)

### 📡 2. Mesh Networking & Protocol Engineers
1. [01 — Protocol Extension System Architecture](01-protocol-extension-system-architecture.md)
2. [03 — Transport & Routing Policy Engine](03-transport-routing-policy-engine-architecture.md)
3. [06 — DTN Store-Carry-Forward & Bundle Forwarding](06-dtn-store-carry-forward-architecture.md)
4. [12 — Multipath Networking & Link Aggregation](12-multipath-networking-architecture.md)
5. [14 — Proximity Abstraction (BLE, Wi-Fi Direct, LoRa)](14-proximity-abstraction-architecture.md)
6. [17 — Emergency Priority Classes & Critical Alert Mesh](17-emergency-priority-classes-architecture.md)

### 💾 3. Storage, Database & Sync Engineers
1. [04 — Offline Event Log & Sync Engine](04-offline-event-log-architecture.md)
2. [05 — Robust File & Blob Subsystem](05-robust-file-blob-subsystem-architecture.md)
3. [09 — Crash Recovery & State Integrity](09-crash-recovery-architecture.md)
4. [32 — Search, Indexing & Local Privacy Retrieval](32-search-indexing-local-knowledge-privacy-architecture.md)
5. [33 — Backup, Restore, Export & Long-Term Archival](33-backup-restore-export-import-archival-portability-architecture.md)

### 📞 4. Realtime Media & Audio DSP Engineers
1. [29 — Realtime Calls & Media Session Protocol](29-realtime-calls-media-session-protocol-architecture.md)
2. [26 — Rust-First Audio DSP (AEC, NS, AGC, Resampling)](26-rust-first-audio-dsp-resampling-aec-ns-agc-architecture.md)
3. [25 — Android Direct Surface Zero-Copy Media](25-android-direct-hardware-surface-zero-copy-media-architecture.md)
4. [UI/UX 07 — Calls & Realtime Media UX](ui-ux-07-calls-realtime-media-architecture.md)

### 📱 5. UI/UX & Frontend Engineers (Desktop / Mobile)
1. [UI/UX 01 — Product UX Foundation & Cross-Platform Interaction](ui-ux-01-product-foundation-cross-platform-interaction-architecture.md)
2. [UI/UX 02 — Desktop Dioxus App Shell & Navigation](ui-ux-02-desktop-dioxus-app-shell-navigation-window-architecture.md)
3. [UI/UX 03 — Android Jetpack Compose App Shell & Lifecycle](ui-ux-03-android-jetpack-compose-app-shell-navigation-lifecycle-architecture.md)
4. [UI/UX 22 — Design System, Tokens, Typography & Motion](ui-ux-22-design-system-tokens-typography-icons-motion-architecture.md)
5. [UI/UX 23 — Responsive & Adaptive Layouts](ui-ux-23-responsive-adaptive-desktop-tablet-foldable-phone-layout-architecture.md)

---

## 💡 Reading & Keyboard Shortcuts

This interactive wiki is powered by **mdBook**. You can use the following shortcuts from any page:

- <kbd>s</kbd> or <kbd>/</kbd> — Focus the **Full-Text Search Bar**
- <kbd>←</kbd> / <kbd>→</kbd> — Navigate to **Previous / Next Chapter**
- <kbd>t</kbd> — Open the **Theme Selector** (Navy, Coal, Ayu, Rust, Light)
- <kbd>c</kbd> — Copy Code Blocks
- Toggle sidebar button in the top left for clean reading on mobile and tablet devices.

---

## 🏛️ Core Architectural Tenets

```
   ┌─────────────────────────────────────────────────────────┐
   │                     SIAR Core Engine                    │
   ├───────────────────┬───────────────────┬─────────────────┤
   │  Zero Monolith    │  Offline-First    │ Zero Trust      │
   │  WASM & Plugins   │  DTN Store-Carry  │ Multi-Device    │
   │  Extensible ABI   │  Append-Only Log  │ End-to-End Enc  │
   └───────────────────┴───────────────────┴─────────────────┘
```

1. **Transport Neutrality**: Iroh QUIC as primary transport, with pluggable support for BLE, Wi-Fi Direct, LoRa, TCP relays, and custom radio layers.
2. **Zero Centralization Requirement**: Every node can act as an autonomous router, storage node, or relay without depending on centralized servers.
3. **Rust Memory & Concurrency Safety**: The entire core engine and DSP pipeline is implemented in pure, zero-cost-abstraction Rust.
