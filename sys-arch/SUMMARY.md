# SIAR System Architecture & Technical Wiki

[Architecture Portal & Reading Guide](Home.md)

---

# Part I: Protocol Foundation & Extension Platform
- [01 — Protocol Extension System Architecture](01-protocol-extension-system-architecture.md)
- [07 — Capability Negotiation & Protocol Upgrades](07-capability-negotiation-architecture.md)
- [21 — Third-Party Protocol Extensions](21-third-party-protocol-extensions-architecture.md)
- [22 — WebAssembly (WASM) Compatible Components](22-wasm-compatible-components-architecture.md)
- [23 — External Interoperability & Conformance Suite](23-external-interoperability-suite-architecture.md)
- [24 — Dynamic Plugin & Module Ecosystem](24-plugin-module-ecosystem-architecture.md)

# Part II: Identity, Trust & Cryptography
- [02 — Multi-Device Identity & Trust Model](02-multi-device-identity-architecture.md)
- [28 — Production Security, E2EE, Key Management & Privacy](28-production-security-e2ee-key-management-privacy-architecture.md)

# Part III: Mesh Networking, Transports & Hardware
- [03 — Transport & Routing Policy Engine](03-transport-routing-policy-engine-architecture.md)
- [11 — Self-Hosted Relay & Coordination Infrastructure](11-relay-self-hosted-infrastructure-architecture.md)
- [12 — Multipath Networking & Link Aggregation](12-multipath-networking-architecture.md)
- [13 — Battery-Aware Scheduling & Power Profiles](13-battery-aware-scheduling-architecture.md)
- [14 — Proximity Abstraction (BLE, Wi-Fi Direct, LoRa)](14-proximity-abstraction-architecture.md)
- [15 — Out-of-Band QR & NFC Bootstrap Pairing](15-qr-nfc-bootstrap-pairing-architecture.md)

# Part IV: Delay-Tolerant Networking & Emergency Mesh
- [06 — DTN Store-Carry-Forward & Bundle Forwarding](06-dtn-store-carry-forward-architecture.md)
- [17 — Emergency Priority Classes & Critical Alert Mesh](17-emergency-priority-classes-architecture.md)

# Part V: Storage, Outbox & Data Lifecycle
- [04 — Offline Event Log & Sync Engine](04-offline-event-log-architecture.md)
- [05 — Robust File & Blob Subsystem (Chunking & Resumption)](05-robust-file-blob-subsystem-architecture.md)
- [09 — Crash Recovery & State Integrity](09-crash-recovery-architecture.md)
- [32 — Search, Indexing & Local Privacy Retrieval](32-search-indexing-local-knowledge-privacy-architecture.md)
- [33 — Backup, Restore, Export & Long-Term Archival](33-backup-restore-export-import-archival-portability-architecture.md)

# Part VI: Realtime Media, Calling & Audio DSP
- [25 — Android Direct Surface Zero-Copy Media](25-android-direct-hardware-surface-zero-copy-media-architecture.md)
- [26 — Rust-First Audio DSP (AEC, NS, AGC, Resampling)](26-rust-first-audio-dsp-resampling-aec-ns-agc-architecture.md)
- [29 — Realtime Calls & Media Session Protocol](29-realtime-calls-media-session-protocol-architecture.md)

# Part VII: Ephemeral State, Presence & Notifications
- [30 — Presence, Availability, Typing & Read Receipts](30-presence-availability-typing-read-receipts-ephemeral-state-architecture.md)
- [31 — Notifications, Push Wake & Background Delivery](31-notifications-push-background-delivery-lifecycle-architecture.md)

# Part VIII: Native Runtimes, Daemons & Embedded Linux
- [16 — Headless Daemon & Background Runtime](16-daemon-headless-runtime-architecture.md)
- [19 — C-ABI FFI & Native Language Interop](19-c-abi-ffi-architecture.md)
- [20 — Embedded Linux Node & Resource-Constrained Targets](20-embedded-linux-node-architecture.md)
- [27 — Rust-Driven Android Native Build & Packaging Automation](27-rust-driven-android-native-build-packaging-automation.md)

# Part IX: Reliability, Testing, Fuzzing & Diagnostics
- [08 — Resource Limits, Quotas & Backpressure](08-resource-limits-backpressure-architecture.md)
- [10 — Fuzzing & Protocol Test Suite](10-fuzzing-protocol-test-suite-architecture.md)
- [18 — Network Diagnostics & Path Visualization](18-network-diagnostics-path-visualization-architecture.md)

---

# Part X: UI/UX Client Architecture (Cross-Platform)
- [UI/UX 01 — Product UX Foundation & Cross-Platform Interaction](ui-ux-01-product-foundation-cross-platform-interaction-architecture.md)
- [UI/UX 02 — Desktop Dioxus App Shell & Navigation](ui-ux-02-desktop-dioxus-app-shell-navigation-window-architecture.md)
- [UI/UX 03 — Android Jetpack Compose App Shell & Lifecycle](ui-ux-03-android-jetpack-compose-app-shell-navigation-lifecycle-architecture.md)
- [UI/UX 04 — Conversation List & Inbox UX](ui-ux-04-conversation-list-inbox-architecture.md)
- [UI/UX 05 — Conversation Message Timeline UX](ui-ux-05-conversation-message-timeline-architecture.md)
- [UI/UX 06 — Message Composer, Attachments & Voice Notes](ui-ux-06-message-composer-attachments-voice-notes-drafts-architecture.md)
- [UI/UX 07 — Calls & Realtime Media UX](ui-ux-07-calls-realtime-media-architecture.md)
- [UI/UX 08 — Contacts, Requests, Verification & Identity](ui-ux-08-contacts-requests-verification-identity-architecture.md)
- [UI/UX 09 — Groups, Membership & Roles UX](ui-ux-09-groups-membership-roles-architecture.md)
- [UI/UX 10 — Files, Media Gallery & Transfer UX](ui-ux-10-files-media-gallery-transfer-architecture.md)
- [UI/UX 11 — Search & Local Knowledge Retrieval UX](ui-ux-11-search-local-knowledge-retrieval-architecture.md)
- [UI/UX 12 — Nearby, QR/NFC Pairing & Device Linking UX](ui-ux-12-nearby-qr-nfc-pairing-device-linking-architecture.md)
- [UI/UX 13 — Notifications, Background & Incoming Call UX](ui-ux-13-notifications-background-incoming-call-architecture.md)
- [UI/UX 14 — Presence, Typing, Receipts & Status UX](ui-ux-14-presence-typing-receipts-status-architecture.md)
- [UI/UX 15 — Security Center, Devices, Keys & Recovery UX](ui-ux-15-security-center-devices-keys-recovery-architecture.md)
- [UI/UX 16 — Backup, Restore, Export & Migration UX](ui-ux-16-backup-restore-export-migration-architecture.md)
- [UI/UX 17 — Emergency / SOS / Offline Mesh UX](ui-ux-17-emergency-sos-offline-mesh-architecture.md)
- [UI/UX 18 — Settings, Privacy, Notifications & Data Controls](ui-ux-18-settings-privacy-notifications-data-controls-architecture.md)
- [UI/UX 19 — Plugin & Module Ecosystem UX](ui-ux-19-plugin-module-ecosystem-architecture.md)
- [UI/UX 20 — Diagnostics, Network Paths & Developer UX](ui-ux-20-diagnostics-network-paths-advanced-developer-architecture.md)
- [UI/UX 21 — Accessibility & Inclusive Interaction](ui-ux-21-accessibility-inclusive-interaction-architecture.md)
- [UI/UX 22 — Design System, Tokens, Typography & Motion](ui-ux-22-design-system-tokens-typography-icons-motion-architecture.md)
- [UI/UX 23 — Responsive & Adaptive Layouts (Desktop/Tablet/Foldable/Phone)](ui-ux-23-responsive-adaptive-desktop-tablet-foldable-phone-layout-architecture.md)
- [UI/UX 24 — Error, Loading, Empty & Degraded-State UX](ui-ux-24-error-loading-empty-offline-degraded-state-architecture.md)
- [UI/UX 25 — Onboarding, First Run & Permission Education](ui-ux-25-onboarding-first-run-permission-education-architecture.md)
- [UI/UX 26 — Virtualization & Large-Data UI Performance](ui-ux-26-performance-virtualization-large-data-ui-architecture.md)
- [UI/UX 27 — UI Testing, Screenshots & Release Quality Gates](ui-ux-27-ui-testing-screenshot-interaction-release-quality-gates-architecture.md)
