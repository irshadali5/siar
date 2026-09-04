# Part 14 — Proximity Abstraction Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 14 of 24  
**Primary language:** Rust  
**Primary goals:** unify nearby discovery and local short-range connectivity across BLE, Bluetooth Classic, Wi‑Fi Direct, Wi‑Fi Aware, LAN, QR/NFC bootstrap, and future proximity technologies while preserving privacy, battery efficiency, transport neutrality, and reuse across messaging, files, DTN, emergency, ERP, and custom products.

---

# 1. Purpose

The platform needs one neutral model for "nearby devices" even though every OS exposes different APIs.

Potential proximity mechanisms include:

```text
Bluetooth LE
Bluetooth Classic
Wi-Fi Direct
Wi-Fi Aware
LAN / mDNS-style discovery
QR bootstrap
NFC bootstrap
device hotspot / local network
future UWB or other proximity radios
```

The core must not become polluted with Android callbacks, Linux D-Bus objects, Windows Bluetooth handles, Apple framework types, or device-specific assumptions.

Correct architecture:

```text
Platform APIs
      ↓
Platform Proximity Adapters
      ↓
Proximity Core
      ↓
Nearby Peer / Path Candidates
      ↓
Routing / DTN / Files / Pairing / UI
```

Fundamental rule:

> **Upper layers reason about nearby peers and local path opportunities, not raw radio APIs.**

---

# 2. Architectural Position

```text
Android / iOS / Linux / Windows / macOS
               ↓
      Proximity Platform Adapter
               ↓
        comm-proximity
        ├── discovery
        ├── advertising
        ├── observations
        ├── privacy tokens
        ├── session bootstrap
        ├── path escalation
        └── diagnostics
               ↓
   ┌───────────┼───────────┐
   │           │           │
Routing       DTN        Pairing
   │           │           │
Files       Emergency     UI
```

---

# 3. Nearby Does Not Mean Trusted

A device being nearby does not prove:

```text
identity
ownership
authorization
contact relationship
trust
```

A proximity observation only means:

```text
a potentially reachable endpoint exists nearby
```

Identity binding happens later through Part 02.

---

# 4. Nearby Peer Model

```rust
pub struct NearbyPeer {
    pub observation_id: ObservationId,
    pub ephemeral_id: EphemeralPeerId,
    pub observed_via: ProximityTransportKind,
    pub signal: Option<SignalInfo>,
    pub capabilities: ProximityCapabilityHint,
    pub last_seen: Timestamp,
}
```

Never expose raw MAC address as platform identity.

---

# 5. Proximity Transport Kinds

```rust
pub enum ProximityTransportKind {
    BluetoothLe,
    BluetoothClassic,
    WifiDirect,
    WifiAware,
    Lan,
    QrBootstrap,
    NfcBootstrap,
    Future(u16),
}
```

QR and NFC are bootstrap methods rather than continuous bulk transports, but they fit the same discovery/handshake entry model.

---

# 6. Observation Identity

```rust
pub struct ObservationId([u8; 16]);
```

An `ObservationId` identifies a local observation instance.

It is not:

```text
AccountId
DeviceId
transport identity
```

---

# 7. Ephemeral Peer Identity

Nearby advertisements use rotating identifiers.

```rust
pub struct EphemeralPeerId([u8; 16]);
```

Goals:

```text
reduce passive tracking
avoid exposing stable user identity
allow temporary session correlation
support secure later binding
```

---

# 8. Never Broadcast Stable Identity by Default

Do not advertise:

```text
AccountId
DeviceId
email
phone number
display name
device name
```

over BLE/LAN beacons.

Use short-lived opaque tokens.

---

# 9. Ephemeral Rotation

Rotate based on:

```text
time window
session boundary
privacy policy
pairing mode
```

The exact period should balance:

```text
privacy
reconnect continuity
battery use
radio overhead
```

---

# 10. Identity Binding

Once a secure session forms:

```text
EphemeralPeerId
      ↓
authenticated session
      ↓
DeviceId
      ↓
AccountId
```

Only then may upper layers merge observations into known-device state.

---

# 11. Discovery and Connection Must Be Separate

Discovery:

```text
peer observed
```

Connection:

```text
usable transport session established
```

Authentication:

```text
peer identity verified
```

These are distinct phases.

---

# 12. Discovery State Machine

```text
Idle
 ↓
Scanning
 ↓
Observed
 ↓
Updated
 ↓
Expired/Lost
```

Connection state:

```text
Candidate
 ↓
Connecting
 ↓
TransportReady
 ↓
Authenticated
 ↓
UsablePath
```

---

# 13. Observation Expiry

Nearby peers must age out.

```rust
pub struct ObservationTtl(pub Duration);
```

A peer not seen recently must not stay permanently listed as nearby.

---

# 14. Signal Model

```rust
pub struct SignalInfo {
    pub strength: SignalStrengthClass,
    pub raw_rssi: Option<i16>,
}
```

---

# 15. Signal Strength Class

```rust
pub enum SignalStrengthClass {
    VeryWeak,
    Weak,
    Medium,
    Strong,
    VeryStrong,
    Unknown,
}
```

Do not convert RSSI into fake exact physical distance.

---

# 16. Capability Hints

Advertisements may expose tiny coarse hints:

```rust
pub struct ProximityCapabilityHint {
    pub supports_pairing: bool,
    pub supports_dtn: bool,
    pub supports_wifi_upgrade: bool,
    pub gateway_hint: bool,
}
```

Detailed capability negotiation belongs to Part 07.

---

# 17. Advertisement Payload

Keep it tiny.

Example:

```rust
pub struct ProximityAdvertisementV1 {
    pub protocol_version: u8,
    pub ephemeral_id: [u8; 16],
    pub capability_bits: u16,
    pub short_token: Option<[u8; 8]>,
}
```

Do not include:

```text
certificate chains
full bundle inventory
full capability matrix
stable account identifiers
```

---

# 18. Progressive Disclosure

```text
Advertisement
      ↓
Nearby transport
      ↓
Secure handshake
      ↓
Identity binding
      ↓
Capability negotiation
      ↓
Feature use
```

This improves privacy and reduces radio overhead.

---

# 19. Discovery API

```rust
pub trait ProximityDiscovery {
    async fn start_scan(
        &self,
        policy: ScanPolicy,
    ) -> Result<ScanHandle, ProximityError>;

    fn events(&self) -> ProximityEventStream;
}
```

---

# 20. Proximity Events

```rust
pub enum ProximityEvent {
    Observed(NearbyPeer),
    Updated(NearbyPeer),
    Lost(ObservationId),
    PathHint(ProximityPathHint),
    PermissionChanged(ProximityPermissionState),
    Error(ProximityError),
}
```

---

# 21. Discovery Purpose

```rust
pub enum DiscoveryPurpose {
    General,
    Pairing,
    SendFile,
    Dtn,
    Emergency,
}
```

Purpose allows policy to tune aggressiveness.

---

# 22. Scan Modes

```rust
pub enum ScanMode {
    Passive,
    Balanced,
    Active,
    Emergency,
}
```

---

# 23. Passive Scan

Use for:

```text
background
battery saver
long-lived low-power DTN presence
```

Characteristics:

```text
low duty cycle
no expensive Wi-Fi setup
minimal wakeups
```

---

# 24. Balanced Scan

Default foreground/background compromise.

---

# 25. Active Scan

Use when user explicitly opens:

```text
Nearby
Pair device
Send nearby
```

Higher duty cycle is temporarily acceptable.

---

# 26. Emergency Scan

Emergency mode can enable:

```text
higher BLE duty cycle
local Wi-Fi discovery
DTN peer search
gateway search
```

but still respects hard resource limits.

---

# 27. Advertising API

```rust
pub trait ProximityAdvertiser {
    async fn advertise(
        &self,
        policy: AdvertisePolicy,
    ) -> Result<AdvertiseHandle, ProximityError>;
}
```

---

# 28. Visibility Modes

```rust
pub enum VisibilityPolicy {
    Hidden,
    PairingMode,
    ContactsOrAuthorized,
    DtnRelay,
    Emergency,
}
```

"Contacts only" does not mean broadcasting contact identity. It means continuing secure interaction only after authorization proof.

---

# 29. Pairing Visibility

Pairing mode should:

```text
be explicit
be temporary
expire automatically
```

Do not keep devices permanently discoverable.

---

# 30. Bluetooth LE Role

BLE is best suited for:

```text
discovery
control signaling
small messages
DTN encounter bootstrap
pairing
Wi-Fi upgrade negotiation
```

It is not the preferred path for:

```text
large files
video attachments
realtime high-rate media
```

---

# 31. Bluetooth Classic Role

Bluetooth Classic may provide:

```text
moderate local transfer
legacy device support
fallback data transport
```

where platform APIs support it.

---

# 32. Wi-Fi Direct Role

Use for:

```text
large nearby file transfers
high-throughput local sessions
no-router environments
```

It has higher setup cost and should be activated selectively.

---

# 33. Wi-Fi Aware Role

Where available:

```text
nearby service discovery
peer-aware data path
router-independent communication
```

Especially useful on supported Android devices.

---

# 34. LAN Role

If peers share a router/local network:

```text
LAN direct
```

is often ideal:

```text
low latency
high bandwidth
no Internet dependency
low setup cost
```

---

# 35. LAN Discovery Privacy

Do not publish user identity through service names.

Use opaque ephemeral service instances and authenticate later.

---

# 36. Avoid Blind Subnet Scanning

Default LAN discovery should not scan every IP in the subnet.

Prefer:

```text
service discovery
known-peer probing
signed endpoint hints
```

Reasons:

```text
battery
network noise
security software
latency
privacy
```

---

# 37. Existing-Path Preference

If an authenticated local path already exists:

```text
reuse it
```

before activating another radio.

---

# 38. Transport Escalation

Recommended model:

```text
BLE discovers peer
       ↓
secure handshake
       ↓
operation estimated
       ↓
small data → stay on BLE/existing path
large data → negotiate Wi-Fi upgrade
```

---

# 39. Tiny-Payload Rule

Do not create Wi-Fi Direct merely to send:

```text
1 KB message
small receipt
tiny DTN bundle
```

---

# 40. Large-File Rule

For large nearby file:

```text
BLE discovery
      ↓
Wi-Fi Direct/Aware or LAN
      ↓
chunked file transfer
```

---

# 41. DTN Encounter Flow

```text
BLE/LAN observation
      ↓
DTN capability hint
      ↓
secure session
      ↓
inventory reconciliation
      ↓
bundle transfer
      ↓
optional Wi-Fi upgrade
```

Part 06 owns forwarding logic.

---

# 42. Routing Integration

Part 14 emits path candidates.

Part 03 decides whether they are useful.

Example candidates:

```text
BluetoothLe
BluetoothClassic
WifiDirect
WifiAware
Lan
```

---

# 43. Routing Inputs

Each proximity path provides:

```text
capacity class
setup cost
energy cost
health
current availability
```

---

# 44. Path Hint

```rust
pub struct ProximityPathHint {
    pub observation_id: ObservationId,
    pub transport: ProximityTransportKind,
    pub expected_capacity: CapacityClass,
    pub setup_cost: SetupCostClass,
}
```

---

# 45. Capacity Class

```rust
pub enum CapacityClass {
    Tiny,
    Low,
    Medium,
    High,
    VeryHigh,
}
```

---

# 46. Setup Cost

```rust
pub enum SetupCostClass {
    VeryLow,
    Low,
    Medium,
    High,
}
```

---

# 47. Multipath Integration

Part 12 can compose:

```text
BLE control + Wi-Fi data
LAN + Iroh
Wi-Fi Direct + relay
```

Proximity reports local paths; multipath decides composition.

---

# 48. File Integration

Part 05 should receive:

```text
authenticated DeviceId
usable path candidates
```

not raw BLE handles.

---

# 49. Pairing Integration

Part 15 uses Part 14 for:

```text
QR discovery
NFC bootstrap
BLE discovery
nearby session establishment
```

Identity authorization remains Part 02.

---

# 50. Battery Integration

Part 13 determines:

```text
scan duty cycle
advertisement duty cycle
Wi-Fi escalation
background behavior
```

---

# 51. Resource Integration

Part 08 limits:

```text
tracked nearby peers
handshakes
scan jobs
Wi-Fi setup jobs
memory
radio operations
```

---

# 52. Nearby Peer Table

Must be bounded.

```text
max observations
TTL
LRU
rate limits
```

Never retain an unbounded radio-observation history.

---

# 53. Multi-Transport Observation

Before identity binding:

```text
BLE observation
LAN observation
Wi-Fi observation
```

remain separate unless cryptographically linked.

---

# 54. Post-Authentication Merge

After all observations authenticate to same `DeviceId`:

```text
DeviceId
 ├── BLE path
 ├── LAN path
 └── Wi-Fi path
```

Routing now sees one peer with multiple candidate paths.

---

# 55. Do Not Fingerprint to Merge

Avoid correlating anonymous nearby observations by:

```text
radio fingerprints
device quirks
timing patterns
```

just to guess they are the same device.

Privacy is more important.

---

# 56. Discovery Flood Protection

Hostile environments may send thousands of fake advertisements.

Enforce:

```text
max observations
events/sec
handshakes/sec
metadata bytes
```

---

# 57. Cheap Validation First

Processing order:

```text
length check
version check
basic structure
rate limit
then expensive crypto/session work
```

---

# 58. Handshake Escalation

Do not establish secure session with every observed peer automatically.

Trigger based on:

```text
user intent
trusted token match
DTN policy
emergency policy
```

---

# 59. Connection Intent

```rust
pub enum ConnectionIntent {
    Pair,
    Message,
    FileTransfer,
    DtnRelay,
    Emergency,
}
```

---

# 60. Proximity Session

```rust
pub struct ProximitySession {
    pub session_id: SessionId,
    pub transport: ProximityTransportKind,
    pub ephemeral_peer: EphemeralPeerId,
    pub authenticated_peer: Option<DeviceId>,
}
```

---

# 61. Secure Session Rule

No private application protocol should trust the transport alone.

After connection:

```text
run normal cryptographic identity handshake
```

---

# 62. Physical Proximity Is Not Authentication

Signal strength does not prove identity.

Being physically nearby does not prove authorization.

---

# 63. Relay/Distance Attacks

Attackers may relay radio interactions.

Sensitive pairing must use:

```text
QR secret
numeric comparison
NFC touch
authenticated transcript
```

Part 15 handles this.

---

# 64. Gateway Hints

A nearby node may advertise:

```text
gateway available
```

This is a hint only.

Routing/DTN verifies actual connectivity before relying on it.

---

# 65. DTN Relay Hint

Advertisement may say:

```text
DTN supported
```

but never publish full bundle inventory.

---

# 66. Wi-Fi Upgrade Hint

Nearby peer can advertise:

```text
Wi-Fi upgrade available
```

Detailed negotiation occurs later.

---

# 67. Android Adapter Responsibilities

Kotlin/platform side:

```text
BLE scanning
BLE advertising
Bluetooth Classic APIs
Wi-Fi Aware APIs
Wi-Fi Direct APIs
permissions
OS lifecycle
network handles
```

Rust side:

```text
discovery policy
privacy tokens
peer model
transport escalation
routing integration
DTN integration
identity binding
```

---

# 68. Kotlin Boundary

Do not place:

```text
DTN forwarding policy
pairing business logic
routing scoring
file semantics
```

in Kotlin.

Keep the policy core in Rust.

---

# 69. Android Permission Model

Adapter should normalize current permission state into:

```rust
pub enum ProximityPermissionState {
    Granted,
    Denied,
    Restricted,
    NeedsUserAction,
    Unknown,
}
```

---

# 70. Permission-Aware Behavior

If permission unavailable:

```text
stop attempts
surface typed state
wait for user/platform change
```

Do not retry continuously.

---

# 71. Android Background Restrictions

Adapter reports:

```text
foreground
background allowed
background restricted
```

Part 13 adjusts scan/advertising policy.

---

# 72. iOS Adapter

Map available Apple proximity frameworks into the same neutral model.

Do not pretend unsupported technologies exist.

---

# 73. Desktop Adapters

Provide:

```text
Linux
Windows
macOS
```

implementations behind the same trait.

LAN discovery should remain first-class even if Bluetooth capabilities differ.

---

# 74. Linux Adapter

Linux-specific details such as:

```text
BlueZ
D-Bus
network interfaces
```

must not leak into core crates.

---

# 75. Windows Adapter

Windows native proximity APIs remain inside platform crate.

---

# 76. macOS Adapter

macOS native APIs remain inside platform crate.

Final behavior must be tested on macOS hardware/CI.

---

# 77. QR Bootstrap

QR provides intentional local bootstrap.

Typical flow:

```text
display QR
      ↓
scan
      ↓
bootstrap secret/token
      ↓
authenticate
      ↓
select best network path
```

---

# 78. NFC Bootstrap

NFC can exchange:

```text
ephemeral key
session token
pairing challenge
```

then move actual communication to another transport.

---

# 79. QR/NFC Are Not Bulk Transports

Do not send large file data through NFC/QR.

They bootstrap trust/session discovery.

---

# 80. QR/NFC Path Upgrade

```text
QR/NFC
   ↓
secure session identity
   ↓
LAN / Wi-Fi / Iroh / BLE
```

---

# 81. Discovery Scope

A scan can be filtered by purpose.

Examples:

```text
Pairing → pairing advertisements only
DTN → DTN-capable peers
File share → nearby authenticated/file-capable candidates
```

---

# 82. Power-Aware Scanning

Part 13 may choose:

```text
Foreground Active
Background Passive
Saver Rare
Emergency Aggressive-but-bounded
```

---

# 83. Scan Window

Foreground manual scan should be finite unless user keeps the screen active.

---

# 84. Advertisement Expiry

Pairing visibility expires quickly.

DTN/emergency visibility follows separate policy.

---

# 85. Router-Isolated LAN

Some Wi-Fi networks prevent peer-to-peer LAN traffic.

The system must detect:

```text
candidate exists
connection fails
```

then routing falls back.

---

# 86. Captive Portal

Local LAN may still work while Internet is blocked.

This is a useful local-first path.

---

# 87. Airplane Mode

Airplane mode does not necessarily mean all proximity transports are unavailable.

Bluetooth/Wi-Fi may be manually re-enabled.

Use actual adapter state.

---

# 88. Platform State Invalidation

Events:

```text
Bluetooth off
Wi-Fi off
permission revoked
network lost
```

must invalidate affected candidates.

---

# 89. Observation Persistence

Live proximity observations are ephemeral.

After process restart:

```text
rescan
```

Do not restore stale "nearby now" state.

---

# 90. Persistent Hints

It may be useful to retain:

```text
last successful local path type
known peer supports proximity
```

as hints only.

---

# 91. Event Log Boundary

Do not permanently journal:

```text
peer observed
peer lost
RSSI changed
```

Meaningful domain transitions such as:

```text
DevicePaired
BundleForwarded
```

belong to their own domain logs.

---

# 92. UI Model

Normal UI should expose:

```text
Nearby
Known nearby devices
Pairing requests
Emergency relay available
```

not platform-specific radio jargon.

---

# 93. Pre-Authentication UI

Before identity:

```text
Nearby device
Strong signal
File-capable
```

Do not show guessed personal names.

---

# 94. Post-Authentication UI

After secure identity:

```text
Bob's Phone
```

if product policy allows displaying the known device name.

---

# 95. Nearby File UX

```text
Share nearby
 ↓
discover
 ↓
select peer
 ↓
authenticate
 ↓
choose best path
 ↓
transfer
```

---

# 96. Emergency UX

DTN proximity can work automatically.

User should not need to manually select every relay.

---

# 97. Privacy Threats

Protect against:

```text
stable-radio tracking
device-name leakage
nearby graph collection
fingerprinting
pairing spam
gateway spoofing
```

---

# 98. Telemetry Privacy

Never export by default:

```text
raw MAC
EphemeralPeerId
nearby peer graph
exact RSSI history
```

---

# 99. Metrics

Safe aggregate metrics:

```text
scan duration
observation count
successful secure nearby sessions
Wi-Fi upgrades
permission failures
```

---

# 100. Diagnostics

Advanced local diagnostics:

```text
BLE: scanning
LAN: available
Wi-Fi Aware: unsupported
Wi-Fi Direct: idle
Observed peers: 3
Authenticated local peers: 1
```

---

# 101. Abuse Protection

Rate-limit:

```text
advertisements
pairing attempts
secure handshakes
Wi-Fi upgrade requests
DTN encounter requests
```

---

# 102. Pairing Spam

Repeated unknown pairing requests:

```text
rate limit
temporary ignore
```

Avoid repeated user prompts.

---

# 103. Gateway Spoofing

Gateway hints only improve ranking after verification.

---

# 104. Authority Spoofing

An unknown nearby device cannot become a trusted emergency authority merely by setting a capability bit.

Authority is cryptographically verified later.

---

# 105. Protocol Versioning

Nearby advertisement includes small version.

Unknown incompatible major:

```text
ignore safely
```

or use explicit compatible fallback.

---

# 106. Parser Safety

Advertisement parser must be:

```text
bounded
non-allocating or minimally allocating
panic-free
```

---

# 107. Fuzzing

Part 10 should fuzz:

```text
advertisement parser
LAN discovery record
capability bitset
bootstrap token
proximity control frames
```

---

# 108. Flood Test

Inject massive fake advertisements.

Expected:

```text
bounded observations
bounded handshakes
bounded memory
```

---

# 109. Expiry Test

Peer disappears.

Expected:

```text
observation expires
candidate removed
```

---

# 110. Rotation Test

Ephemeral ID changes.

Expected:

```text
old anonymous observation not automatically linked
```

unless authenticated continuity proves identity.

---

# 111. Multi-Transport Merge Test

Same DeviceId authenticates through BLE and LAN.

Expected:

```text
one logical peer
multiple path candidates
```

---

# 112. Permission Test

Permission revoked during scan.

Expected:

```text
scan stops
state becomes NeedsUserAction/Denied
no retry storm
```

---

# 113. Tiny Message Test

BLE + Wi-Fi upgrade available.

Send 1 KB.

Expected:

```text
no unnecessary Wi-Fi Direct setup
```

---

# 114. Large File Test

Same peer, large file.

Expected:

```text
Wi-Fi upgrade attempted if policy allows
```

---

# 115. DTN Encounter Test

BLE-only environment.

Expected:

```text
DTN inventory and small critical bundle exchange
```

---

# 116. LAN Offline Test

Two devices on router with no Internet.

Expected:

```text
local discovery
authenticated local communication
```

---

# 117. AP Isolation Test

Discovery hint exists but direct LAN blocked.

Expected:

```text
path marked failed
routing fallback
```

---

# 118. Battery Test

Saver mode:

```text
lower duty cycle
```

Emergency mode:

```text
higher bounded duty cycle
```

---

# 119. Process Restart Test

After restart:

```text
no stale nearby observations
fresh scan according to current power policy
```

---

# 120. Platform Trait

```rust
pub trait ProximityPlatform: Send + Sync {
    async fn scan(
        &self,
        request: PlatformScanRequest,
    ) -> Result<PlatformScanHandle, ProximityError>;

    async fn advertise(
        &self,
        request: PlatformAdvertiseRequest,
    ) -> Result<PlatformAdvertiseHandle, ProximityError>;

    async fn connect(
        &self,
        candidate: PlatformProximityCandidate,
    ) -> Result<PlatformProximityConnection, ProximityError>;
}
```

---

# 121. Suggested Crate Structure

```text
crates/comm-proximity/
├── src/
│   ├── lib.rs
│   ├── peer.rs
│   ├── observation.rs
│   ├── advertisement.rs
│   ├── discovery.rs
│   ├── advertise.rs
│   ├── session.rs
│   ├── transport.rs
│   ├── privacy.rs
│   ├── escalation.rs
│   ├── policy.rs
│   ├── diagnostics.rs
│   └── error.rs
└── Cargo.toml
```

Platform crates:

```text
comm-proximity-android
comm-proximity-ios
comm-proximity-linux
comm-proximity-windows
comm-proximity-macos
```

---

# 122. Android Adapter Structure

```text
comm-proximity-android/
├── ble/
├── bluetooth_classic/
├── wifi_aware/
├── wifi_direct/
├── permissions/
├── lifecycle/
└── bridge/
```

Kotlin bridge should remain as small as practical.

---

# 123. Error Model

```rust
pub enum ProximityError {
    Unsupported,
    PermissionDenied,
    Disabled,
    BackgroundRestricted,
    ResourceDenied,
    MalformedAdvertisement,
    ConnectionFailed,
    AuthenticationFailed,
    Timeout,
    Cancelled,
    Platform,
}
```

---

# 124. Retry Semantics

```text
Timeout → retryable
Bluetooth off → user/platform action
Permission denied → user action
Malformed advertisement → drop
Resource denied → defer
```

---

# 125. Public API

```rust
let scan = proximity
    .scan(DiscoveryPurpose::FileTransfer)
    .await?;

while let Some(peer) = scan.next().await {
    // Safe nearby peer model
}
```

Connect:

```rust
let session = proximity
    .connect(
        observation_id,
        ConnectionIntent::FileTransfer,
    )
    .await?;
```

---

# 126. Application Boundary

Upper-level apps should request:

```text
find nearby peers
connect nearby
send nearby
```

never:

```text
start BLE scanner directly
```

---

# 127. Initial Production Scope

Implement first:

```text
NearbyPeer model
ObservationId
EphemeralPeerId
BLE discovery/advertising
LAN discovery
Android Wi-Fi Direct/Aware adapter
scan policies
observation expiry
secure-session handoff
routing integration
DTN integration
battery integration
permission handling
```

Defer initially:

```text
UWB
precise distance estimation
cross-radio fingerprint correlation
custom mesh routing inside proximity core
```

---

# 128. Implementation Phases

## Phase 1 — Core Model

```text
NearbyPeer
ObservationId
EphemeralPeerId
ProximityEvent
```

## Phase 2 — BLE + LAN

```text
scan
advertise
TTL
```

## Phase 3 — Secure Session Handoff

```text
connect
authenticate
identity bind
multi-path merge
```

## Phase 4 — Wi-Fi Escalation

```text
Wi-Fi Direct
Wi-Fi Aware
large-transfer upgrade
```

## Phase 5 — DTN

```text
encounter discovery
relay hints
gateway hints
```

## Phase 6 — Battery + Permissions

```text
duty cycle
background restrictions
Android permissions
```

## Phase 7 — Pairing Hooks

```text
QR
NFC
temporary pairing visibility
```

## Phase 8 — Hardening

```text
fuzzing
flood testing
privacy tests
real-device tests
```

---

# 129. Definition of Done

Part 14 is complete when:

- upper layers can discover nearby peers without platform-specific APIs
- observations are distinct from trusted identities
- stable AccountId/DeviceId values are not broadcast by default
- ephemeral IDs rotate
- nearby observations expire automatically
- BLE, Wi-Fi, and LAN map into one neutral model
- routing receives local path candidates
- DTN receives encounter notifications
- large nearby transfers can upgrade from BLE discovery to Wi-Fi
- tiny payloads do not trigger expensive Wi-Fi setup
- battery policy controls scan/advertising duty cycle
- permission/background restrictions are typed
- multiple authenticated observations of the same DeviceId merge correctly
- hostile advertisements cannot cause unbounded memory or handshake work
- QR/NFC bootstrap into the same secure identity/session model
- normal UI can present a single "Nearby" concept
- fuzz, flood, permission, privacy, transport-upgrade, and real-device tests exist

---

# 130. Relationship to Earlier Parts

Part 14 builds on:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
03 — Transport & Routing Policy Engine
04 — Offline Event Log
05 — Robust File / Blob Subsystem
06 — DTN / Store-Carry-Forward
07 — Capability Negotiation
08 — Resource Limits & Backpressure
09 — Crash Recovery
10 — Fuzzing & Protocol Test Suite
11 — Relay / Self-Hosted Infrastructure
12 — Multipath Networking
13 — Battery-Aware Scheduling
```

It directly supports:

```text
15 — QR / NFC Bootstrap Pairing
16 — Daemon & Headless Runtime
17 — Emergency Priority Architecture
18 — Network Diagnostics & Path Visualization
20 — Embedded Linux Node
22 — Third-Party Protocol Extensions
24 — Plugin / Module Ecosystem
```

---

# 131. Final Principle

The proximity layer should make this flow possible:

```text
User opens Nearby.

The runtime may use:
BLE
LAN
Wi-Fi Aware
Wi-Fi Direct

The UI simply shows:
Nearby devices.

A large file is selected:
BLE discovers the peer.
The peer is authenticated.
Wi-Fi Direct becomes available.
The transfer upgrades automatically.

Later Internet disappears:
the same proximity layer discovers DTN peers
and enables store-carry-forward encounters.
```

No messaging, file, DTN, or UI code should need to understand Android BLE callbacks, Linux D-Bus, Windows Bluetooth APIs, or platform-specific Wi-Fi sessions.

The proximity abstraction turns all of those mechanisms into:

```text
NearbyPeer
+
ProximitySession
+
PathCandidate
```

That is what keeps the wider platform privacy-preserving, local-first, battery-aware, portable, and reusable.
