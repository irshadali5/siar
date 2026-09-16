# Core System Architecture Part 41 — Censorship Resistance, Bridges, Pluggable Transports & Traffic Obfuscation Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 41  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:**  
- Part 34 — Mixnet, Loopix, Sphinx, Nym & High-Anonymity Transport Architecture  
- Part 35 — Anonymous Mailbox, Offline Receiving & Reply Capability Architecture  
- Part 36 — Sphinx Packet, Cell, Fragmentation & Anonymous Message Framing Architecture  
- Part 37 — Cover Traffic, Traffic Shaping, Timing Obfuscation & Loop Traffic Architecture  
- Part 38 — Native Loopix-Inspired Mixnet Topology, Mix Nodes, Layering & Packet Forwarding Architecture  
- Part 39 — Mixnet Directory, Node Admission, Identity, Sybil Resistance & Topology Governance Architecture  
- Part 40 — Anonymous Attachment Transfer, Rendezvous, Large-File Privacy & High-Bandwidth Anonymous Data Architecture  

**Primary purpose:** define the complete censorship-resistance subsystem for SIAR, including bridge nodes, pluggable transports, transport obfuscation, active-probing resistance, bootstrap under blocking, bridge rotation, regional policy profiles, update survivability, fallback safety, traffic mimicry boundaries, observability, abuse controls, and provider-neutral Rust abstractions.

---

# 1. Purpose

A mixnet can be cryptographically strong and still fail operationally if users cannot reach it.

Censors or restrictive networks may block:

```text
known gateway IPs
known mixnet domains
QUIC
UDP
specific TLS fingerprints
known packet sizes
known protocol handshakes
directory servers
bridge-discovery endpoints
software update endpoints
```

Part 41 addresses network reachability under adversarial filtering.

The governing principle is:

> **Censorship resistance must be an independent access layer that can change transport appearance and entry strategy without weakening SIAR's anonymity, E2EE, or no-downgrade guarantees.**

---

# 2. Architectural Position

```text
SIAR Client
    │
    ▼
Censorship-Resistance Access Layer
    │
    ├── Direct Anonymous Gateway
    ├── Bridge
    ├── Pluggable Transport
    ├── Obfuscated Tunnel
    └── Alternate Bootstrap
    │
    ▼
Gateway / Mixnet Entry
    │
    ▼
Native Mixnet
```

---

# 3. Core Separation

Keep separate:

```text
Access Transport
Mixnet Routing
Application E2EE
```

The access layer only determines how the client reaches an approved anonymous entry point.

---

# 4. Non-Goals

Part 41 does not:

```text
replace the mixnet
replace Sphinx
replace application E2EE
guarantee unblockability
guarantee legality in every jurisdiction
hide all endpoint activity from a global observer
```

---

# 5. Threat Model

Censor capabilities may include:

```text
IP blocking
DNS blocking
SNI filtering
TLS fingerprinting
QUIC blocking
UDP blocking
DPI
active probing
connection reset
traffic-shape classification
protocol fingerprinting
bridge enumeration
directory blocking
update-channel blocking
```

---

# 6. Access Modes

```rust
pub enum AnonymousAccessMode {
    Direct,
    Bridge,
    Obfuscated,
    RestrictedNetwork,
    Automatic,
}
```

---

# 7. Direct

Use ordinary supported anonymous gateway path.

---

# 8. Bridge

Use non-public or semi-private entry node.

---

# 9. Obfuscated

Use pluggable transport that disguises or transforms traffic characteristics.

---

# 10. RestrictedNetwork

Use policy tuned for:

```text
TCP-only
HTTPS-like egress
limited ports
blocked UDP
```

---

# 11. Automatic

Resolver probes approved methods and selects privacy-compatible one.

---

# 12. No Privacy Downgrade

Censorship fallback must not mean:

```text
mixnet blocked
→ direct peer QUIC
```

Hard rule.

---

# 13. Bridge Role

A bridge is an entry proxy that:

```text
accepts client traffic
transforms/forwards it
connects into gateway/mixnet
```

without learning application plaintext.

---

# 14. Bridge Identity

```rust
pub struct BridgeId(pub [u8; 32]);
```

---

# 15. Bridge Descriptor

```rust
pub struct BridgeDescriptor {
    pub bridge_id: BridgeId,
    pub endpoint: BridgeEndpoint,
    pub transports: Vec<PluggableTransportId>,
    pub expires_at: Timestamp,
    pub signature: BridgeDescriptorSignature,
}
```

---

# 16. Bridge Distribution

Do not publish every bridge globally if enumeration resistance is desired.

Possible mechanisms:

```text
invitation-based distribution
rate-limited bridge broker
social distribution
email distribution
QR transfer
trusted contact relay
bundled rotating bootstrap set
```

---

# 17. Bridge Discovery Service

```rust
pub trait BridgeBroker {
    async fn request_bridges(
        &self,
        request: BridgeRequest,
    ) -> Result<Vec<BridgeDescriptor>, BridgeBrokerError>;
}
```

---

# 18. Bridge Enumeration Defense

Broker should avoid:

```text
returning entire bridge pool
```

to one requester.

---

# 19. Bridge Allocation

Return small subset.

---

# 20. Bridge Rotation

Bridge descriptors rotate.

---

# 21. Bridge Epoch

```rust
pub struct BridgeEpoch(pub u64);
```

---

# 22. Rotation Goals

Reduce:

```text
long-term enumeration
static blocking
```

---

# 23. Rotation Cost

Too frequent:

```text
breaks connectivity
increases bootstrap dependency
```

---

# 24. Bridge Pool

Maintain:

```text
current
backup
emergency
```

sets.

---

# 25. Emergency Bootstrap Bridges

Small pre-bundled set.

---

# 26. Bundle Risk

Bundled bridges can be discovered and blocked.

Use only as bootstrap fallback.

---

# 27. Bridge Authentication

Client authenticates bridge identity.

---

# 28. Bridge Must Not MITM E2EE

Hard rule.

---

# 29. Bridge-to-Gateway Forwarding

Bridge forwards opaque anonymous transport session.

---

# 30. Bridge Trust

Bridge may observe:

```text
client IP
connection timing
```

Therefore bridge is not equivalent to mix anonymity.

---

# 31. Bridge Privacy Mitigation

Use:

```text
short-lived bridge assignment
traffic shaping
multiple bridges over time
no user identity at bridge
```

---

# 32. Bridge Logging

Forbidden:

```text
AccountId
DeviceId
conversation data
mailbox IDs
```

---

# 33. Bridge Logs Allowed

Only aggregate:

```text
connections
bytes
errors
resource usage
```

---

# 34. Pluggable Transport Abstraction

```rust
#[async_trait]
pub trait PluggableTransport: Send + Sync {
    async fn connect(
        &self,
        target: BridgeTarget,
        config: TransportConfig,
    ) -> Result<ObfuscatedStream, TransportError>;

    fn capabilities(
        &self,
    ) -> PluggableTransportCapabilities;
}
```

---

# 35. Transport Capabilities

```rust
pub struct PluggableTransportCapabilities {
    pub tcp: bool,
    pub udp: bool,
    pub websocket: bool,
    pub http_like: bool,
    pub active_probe_resistant: bool,
    pub supports_padding: bool,
}
```

---

# 36. Pluggable Transport IDs

```rust
pub struct PluggableTransportId(pub String);
```

---

# 37. Initial Transport Classes

Potential:

```text
Plain QUIC
TCP Tunnel
WebSocket Tunnel
HTTPS-Like Tunnel
Obfuscated TCP
Provider-Native Obfuscation
```

---

# 38. Do Not Tie Architecture to One Obfuscation Scheme

Hard rule.

The access layer must allow transport replacement as censorship techniques evolve.

---

# 39. Provider-Native Support

If Nym/provider supplies censorship-resistance transports, SIAR can integrate through adapter.

---

# 40. Native SIAR Support

Future SIAR-native bridges can expose same trait.

---

# 41. Transport Selection Policy

```rust
pub struct AccessTransportPolicy {
    pub allowed: Vec<PluggableTransportId>,
    pub prefer_udp: bool,
    pub prefer_tcp: bool,
    pub require_probe_resistance: bool,
}
```

---

# 42. Restricted-Network Profile

Example:

```text
TCP 443 only
HTTPS-like
bridge required
```

---

# 43. Enterprise Firewall Profile

Could use:

```text
WebSocket over 443
```

if explicitly allowed.

---

# 44. DPI Resistance

Goal:

```text
avoid trivial protocol classification
```

---

# 45. Traffic Obfuscation Techniques

Architecture may support:

```text
record padding
packet-size normalization
timing jitter
handshake mimicry
stream framing transformations
```

---

# 46. Important Limit

Traffic mimicry can be detected if imperfect.

Do not make absolute claims.

---

# 47. TLS Fingerprint

Transport adapter should avoid unique SIAR-only fingerprints where possible.

---

# 48. ClientHello Fingerprint

Provider-specific.

May require maintained impersonation/mimic strategy.

---

# 49. User-Agent/HTTP Headers

Do not invent static identifying SIAR header.

---

# 50. WebSocket Mode

Should look like ordinary standards-compliant WebSocket where used.

---

# 51. HTTP-Like Mode

Must still authenticate bridge endpoint securely.

---

# 52. Domain Fronting

Can be considered only where technically and contractually supported by upstream infrastructure.

Do not hard-code it as a dependency.

---

# 53. Architecture Boundary

Expose generic:

```text
frontable/indirected HTTPS transport
```

capability rather than binding core design to one CDN technique.

---

# 54. Active Probing

Censor may connect to suspected bridge and inspect behavior.

---

# 55. Probe Resistance

Bridge should not reveal itself to unauthenticated random probe if transport supports secret-based admission.

---

# 56. Bridge Access Secret

```rust
pub struct BridgeAccessToken(SecretBytes);
```

---

# 57. Token Distribution

Delivered with bridge descriptor.

---

# 58. Token Scope

Bound to:

```text
bridge
transport
expiry
```

---

# 59. Probe Response

Unauthenticated probe should receive behavior indistinguishable from ordinary benign endpoint where feasible.

---

# 60. No Dangerous Protocol Confusion

Do not proxy arbitrary attacker traffic blindly.

---

# 61. Authentication Before Expensive Work

Mitigate DoS.

---

# 62. Replay Protection

Access token handshake should prevent replay where protocol supports.

---

# 63. Bridge Access Token Rotation

Short-lived.

---

# 64. Bridge Credential Leak

Rotate/revoke bridge descriptor.

---

# 65. Bridge Revocation

Signed revocation feed.

---

# 66. Bootstrap Under DNS Blocking

Options:

```text
embedded IP bootstrap
DoH/DoT where available
multiple resolvers
bridge descriptor with IP
trusted contact transfer
```

---

# 67. DNS Independence

Critical control-plane endpoints should not rely on one DNS name only.

---

# 68. Bootstrap Under Directory Blocking

Use:

```text
signed cached topology
signed bridge bundle
mirrors
trusted contact transfer
```

---

# 69. Bootstrap Bundle

```rust
pub struct CensorshipBootstrapBundle {
    pub topology_snapshot: Option<TopologySnapshot>,
    pub bridges: Vec<BridgeDescriptor>,
    pub directory_mirrors: Vec<MirrorDescriptor>,
    pub expires_at: Timestamp,
    pub signature: BootstrapSignature,
}
```

---

# 70. Bundle Authenticity

Signed.

---

# 71. Bundle Freshness

Bounded lifetime.

---

# 72. QR Transfer

Trusted peer can share bootstrap bundle via:

```text
QR
NFC
local file
```

---

# 73. Offline Bootstrap

Useful when Internet metadata channels are blocked.

---

# 74. Bootstrap Privacy

Bundle must not contain user identity.

---

# 75. Trusted Contact Bootstrap

Contact can share bridge descriptors.

---

# 76. Social Distribution Risk

Malicious contact can provide fake bridge.

Signature verification prevents trust substitution.

---

# 77. Bridge Broker Privacy

Broker itself can learn that requester needs bridges.

---

# 78. Broker Access

Possible future:

```text
privacy-preserving token
anonymous request
```

---

# 79. Initial Broker

Could be ordinary HTTPS with strict data minimization.

---

# 80. Broker Logging

No account identity if avoidable.

---

# 81. Regional Blocking Profiles

Different censorship environments need different transport preference.

---

# 82. Regional Policy

```rust
pub struct CensorshipProfile {
    pub name: String,
    pub blocked_protocols: Vec<ProtocolClass>,
    pub preferred_transports: Vec<PluggableTransportId>,
    pub bridge_required: bool,
}
```

---

# 83. Do Not Geo-Locate User Precisely

Policy can be user-selected or inferred from connection failures.

---

# 84. Failure-Based Adaptation

If QUIC repeatedly blocked:

```text
try approved TCP/WebSocket transport
```

---

# 85. Hysteresis

Avoid transport flapping.

---

# 86. Access Resolver

```rust
#[async_trait]
pub trait AnonymousAccessResolver {
    async fn resolve(
        &self,
        policy: AnonymousAccessPolicy,
    ) -> Result<ResolvedAnonymousAccess, AccessError>;
}
```

---

# 87. Access Policy

```rust
pub struct AnonymousAccessPolicy {
    pub privacy_mode: PrivacyRoutingMode,
    pub access_mode: AnonymousAccessMode,
    pub allowed_transports: Vec<PluggableTransportId>,
    pub allow_bridges: bool,
}
```

---

# 88. Resolved Access

```rust
pub struct ResolvedAnonymousAccess {
    pub bridge: Option<BridgeDescriptor>,
    pub transport: PluggableTransportId,
    pub gateway: AnonymousProviderId,
}
```

---

# 89. Resolver Invariant

All selected routes still terminate in approved anonymous gateway/mixnet.

---

# 90. No Direct Peer Fallback

Hard rule.

---

# 91. Transport Probe

Client may test:

```text
TCP reachable
QUIC reachable
WebSocket reachable
bridge reachable
```

---

# 92. Probe Privacy

Avoid probing all known bridges.

---

# 93. Probe Budget

Bound:

```text
number of probes
frequency
```

---

# 94. Probe Fingerprinting

Probes themselves can identify SIAR.

Use adapter-specific safe probes.

---

# 95. Active Censor Reaction

Repeated failed transport can trigger fallback.

---

# 96. Transport State

```rust
pub enum AccessPathState {
    Ready,
    Degraded,
    Blocked,
    Probing,
    Unavailable,
}
```

---

# 97. Path Memory

Cache recent successful path.

---

# 98. Cache Privacy

No peer identity involved.

---

# 99. Transport Rotation

Can rotate among approved transports if current becomes blocked.

---

# 100. Bridge Rotation Trigger

```text
block
expiry
revocation
poor health
enumeration suspicion
```

---

# 101. Rotation Randomization

Avoid all clients rotating at same exact time.

---

# 102. Bridge Health

```rust
pub struct BridgeHealth {
    pub state: AccessPathState,
    pub last_success: Option<Timestamp>,
    pub latency_class: HealthLatencyClass,
}
```

---

# 103. Bridge Health Source

Client local observations + broker/observer health.

---

# 104. No Global Exact Usage Counters

Avoid exposing bridge popularity precisely.

---

# 105. Load Balancing

Bridge broker may spread clients.

---

# 106. Load Privacy

Do not assign based on user identity.

---

# 107. Bridge Capacity Class

```rust
pub enum BridgeCapacityClass {
    Small,
    Medium,
    Large,
}
```

---

# 108. Bridge Overload

Return alternate descriptor.

---

# 109. Access Congestion

Do not confuse censorship with congestion.

---

# 110. Failure Classification

```rust
pub enum AccessFailureClass {
    Timeout,
    Reset,
    DnsBlocked,
    QuicBlocked,
    TlsBlocked,
    ProbeRejected,
    BridgeUnavailable,
    Unknown,
}
```

---

# 111. Classification Is Heuristic

Do not claim certainty.

---

# 112. Obfuscation Profile

```rust
pub enum ObfuscationProfile {
    None,
    Minimal,
    Standard,
    Aggressive,
}
```

---

# 113. Aggressive Mode

May increase:

```text
latency
CPU
bandwidth
```

---

# 114. Interaction with Part 37

Access obfuscation sits below traffic shaping.

Conceptually:

```text
SIAR shaped anonymous stream
→ pluggable transport
→ network
```

---

# 115. Do Not Double-Pad Blindly

Coordinate:

```text
cell padding
traffic shaping
transport padding
```

---

# 116. Padding Ownership

Provider capability reports:

```text
transport-level padding available
```

---

# 117. Access Transport Capabilities

```rust
pub struct AccessTransportCapabilities {
    pub supports_padding: bool,
    pub supports_mimicry: bool,
    pub supports_probe_resistance: bool,
    pub supports_tcp_only: bool,
}
```

---

# 118. Packet Size Interaction

Outer transport may fragment.

---

# 119. MTU

Must avoid pathological fragmentation.

---

# 120. Path MTU Discovery

Transport adapter handles.

---

# 121. TCP Head-of-Line

Restricted mode may accept higher latency.

---

# 122. QUIC Blocked

Fallback to TCP-based approved transport.

---

# 123. UDP Blocked

Same.

---

# 124. HTTPS Proxy Environments

Optional support.

---

# 125. Corporate Proxy

Could tunnel through standard proxy if policy permits.

---

# 126. Proxy Authentication

Handled by platform/network settings, not anonymous identity.

---

# 127. Proxy Privacy

Corporate proxy sees client connection.

Still better than direct peer exposure, but privacy implications must be shown.

---

# 128. Captive Portal

Detect carefully.

---

# 129. Captive Portal State

Anonymous access unavailable until portal completed.

---

# 130. No Portal Bypass Tricks

Use OS flow.

---

# 131. Mobile Carrier Blocking

Adaptive transport can switch.

---

# 132. IPv6/IPv4

Bridge descriptors may include both.

---

# 133. NAT64

Transport adapter should support.

---

# 134. DNS64

Avoid assumptions.

---

# 135. Update Survivability

Censor may block software updates.

---

# 136. Update Channels

Use multiple signed distribution paths.

---

# 137. Update Artifact Security

Always verify:

```text
signature
version
hash
```

---

# 138. Alternate Update Sources

Possible:

```text
official mirrors
peer-assisted signed package
manual offline package
```

---

# 139. Peer-Assisted Updates

Only distribute signed official artifact.

---

# 140. No Trust in Peer

Trust package signature.

---

# 141. Bootstrap Data Update

Bridge lists/topology can update independently from app binary.

---

# 142. Signed Dynamic Config

```rust
pub struct SignedCensorshipConfig {
    pub version: u64,
    pub profiles: Vec<CensorshipProfile>,
    pub bridges: Vec<BridgeDescriptor>,
    pub expires_at: Timestamp,
    pub signature: ConfigSignature,
}
```

---

# 143. Config Anti-Rollback

Reject older signed config after newer accepted.

---

# 144. Config Compromise

Separate signing key from app release key if useful.

---

# 145. Offline Config Transfer

QR/file import supported if signed.

---

# 146. Android Lifecycle

Access transport may run in:

```text
foreground service
background service where allowed
```

---

# 147. Android Process Death

Durable anonymous queue remains.

Access path reconnects after restart.

---

# 148. Android Network Switch

Re-resolve access path.

---

# 149. Android VPN Interaction

Respect user VPN.

Do not disable/bypass.

---

# 150. Desktop Daemon

Maintains bridge/transport connection.

---

# 151. Sleep/Resume

Reconnect with randomized backoff.

---

# 152. Backoff Privacy

Avoid global synchronized reconnect storms.

---

# 153. Censorship Telemetry

Sensitive.

---

# 154. Local-First Telemetry

Prefer local diagnostics.

---

# 155. Safe Aggregate Metrics

```text
transport success rate
bridge availability
failure class bucket
reconnect count
```

---

# 156. Forbidden Metrics

No:

```text
exact bridge token
exact user location
contact identity
mailbox identity
route identity
```

---

# 157. Remote Telemetry

Maximum Anonymity:

```text
disabled by default
```

---

# 158. Censorship Report

User may explicitly submit redacted report.

---

# 159. Report Contents

```text
country/region optional
transport attempted
failure class
app version
```

---

# 160. No Precise IP by Default

Hard rule.

---

# 161. Bridge Operator Observability

Can see:

```text
aggregate connections
bytes
CPU
memory
errors
```

---

# 162. Bridge Privacy Logs

No raw client IP retention where operationally avoidable.

---

# 163. Legal/Operational Reality

Some operators may need short-lived abuse/security logs.

Policy must be transparent.

---

# 164. Retention

Minimize and time-bound.

---

# 165. Abuse Risks

Bridges can be abused for:

```text
DoS
open proxying
scanner traffic
resource exhaustion
```

---

# 166. Not an Open Proxy

Hard rule.

Bridge only forwards authenticated SIAR anonymous protocol traffic.

---

# 167. Protocol Whitelist

Bridge accepts only:

```text
authenticated access transport
approved destination
```

---

# 168. Destination Pinning

Bridge forwards only to:

```text
approved SIAR gateway/mixnet endpoints
```

---

# 169. No Arbitrary Internet Egress

Hard rule.

---

# 170. DoS Protection

Use:

```text
token validation
connection limits
rate limits
proof-of-work optionally
```

---

# 171. Proof-of-Work

Optional for abuse control.

Must be battery-aware.

---

# 172. Mobile Fairness

Do not impose heavy proof-of-work on low-end phones by default.

---

# 173. Bridge Access Quotas

```rust
pub struct BridgeQuota {
    pub max_connections: u32,
    pub max_bytes_per_minute: u64,
}
```

---

# 174. Token-Bucket Rate Limit

Suitable.

---

# 175. Per-IP Rate Limit

May help DoS but creates IP processing/logging tradeoffs.

---

# 176. Ephemeral IP State

Prefer short-lived in-memory rate limiting.

---

# 177. No Persistent User Profile

Hard rule.

---

# 178. Active Probe Flood

Authenticate cheaply before expensive crypto where possible.

---

# 179. CPU Budget

Bound handshake workers.

---

# 180. Memory Budget

Bound pending sessions.

---

# 181. Bridge Process Isolation

Separate:

```text
network listener
auth
forwarder
metrics
```

---

# 182. Sandbox

Use OS hardening.

---

# 183. Node Hardening

```text
non-root
seccomp where applicable
read-only filesystem
minimal capabilities
```

---

# 184. Supply Chain

Same strict requirements as mix nodes.

---

# 185. Bridge Software Version

Directory/broker may require minimum version.

---

# 186. Revocation

Compromised bridge removed from broker.

---

# 187. Bridge Rotation After Compromise

Issue new descriptors/tokens.

---

# 188. Transport Plugin Security

Pluggable transports are attack surface.

---

# 189. Plugin Boundary

They should not access:

```text
message plaintext
E2EE keys
mailbox secrets
```

---

# 190. Transport Plugin API

Only opaque byte stream.

---

# 191. Sandboxing

Future dynamic plugins should be sandboxed.

---

# 192. Initial Recommendation

Compile approved transports into trusted binary.

---

# 193. Third-Party Pluggable Transport

Later, signed/sandboxed extension model.

---

# 194. Fallback Chain

Example:

```text
Direct QUIC
→ Bridge QUIC
→ Bridge TCP
→ WebSocket 443
→ HTTPS-like obfuscated
```

All still terminate in approved anonymous entry.

---

# 195. Fallback Policy

```rust
pub struct AccessFallbackPolicy {
    pub ordered: Vec<PluggableTransportId>,
    pub max_attempts: u8,
    pub cooldown: Duration,
}
```

---

# 196. No Infinite Retry

Bound attempts.

---

# 197. Cooldown

Avoid obvious scan behavior.

---

# 198. Parallel Racing

Could race two transports for latency.

---

# 199. Privacy Tradeoff

Parallel attempts expose more endpoints.

---

# 200. Strict Mode

Prefer sequential bounded fallback.

---

# 201. Fast Mode

Could allow small race set.

---

# 202. Bridge Selection

```rust
pub trait BridgeSelector {
    fn select(
        &self,
        bridges: &[BridgeDescriptor],
        policy: &BridgeSelectionPolicy,
    ) -> Result<BridgeDescriptor, BridgeSelectionError>;
}
```

---

# 203. Bridge Selection Inputs

```text
freshness
transport compatibility
health
capacity
operator diversity
```

---

# 204. Do Not Select by Exact User Region Alone

Hard rule.

---

# 205. Bridge Operator Diversity

If multiple bridge hops ever used, operator diversity matters.

---

# 206. Single-Bridge Model

Initial architecture.

---

# 207. Multi-Bridge Chaining

Possible future but may duplicate mixnet complexity.

---

# 208. Do Not Chain Blindly

More hops ≠ automatically better anonymity.

---

# 209. Bridge-to-Mix Handoff

Bridge should not alter Sphinx payload.

---

# 210. Access Encapsulation

```text
Sphinx/provider traffic
→ access transport framing
→ bridge
→ decapsulation
→ gateway
```

---

# 211. Access Framing

Provider-neutral opaque stream.

---

# 212. Framing Padding

Optional.

---

# 213. Active Probe Secret

Can be embedded in handshake.

---

# 214. Token Rotation

Broker issues new token.

---

# 215. Clock Skew

Tokens use bounded validity window.

---

# 216. Replay

Handshake nonce + token scope.

---

# 217. Bridge Descriptor Versioning

```rust
pub struct BridgeDescriptorVersion(pub u16);
```

---

# 218. Unknown Version

Reject safely.

---

# 219. Transport Versioning

Per adapter.

---

# 220. Capability Negotiation

Authenticated.

---

# 221. No Cleartext SIAR Signature

Avoid static magic bytes.

---

# 222. Protocol Fingerprinting

Do not use unique fixed handshake constants externally.

---

# 223. Internal Magic

Allowed only inside encrypted/obfuscated channel.

---

# 224. Cover Traffic Interaction

Part 37 traffic shaping continues through bridge.

---

# 225. Bridge Should Not Strip Timing Protection

Forward promptly/consistently according to access transport.

---

# 226. Additional Bridge Jitter

Possible, but coordinate with mixnet delay.

---

# 227. Avoid Double Randomization Chaos

Policy-driven.

---

# 228. Large File Interaction

Part 40 anonymous bulk provider may need censorship-resistant access too.

---

# 229. Bulk Access

Same pluggable transport framework can wrap:

```text
anonymous rendezvous
bulk relay
```

---

# 230. Throughput Consideration

Some obfuscation modes lower bandwidth.

---

# 231. Bulk Transfer Policy

Could select separate access transport while preserving anonymity requirements.

---

# 232. Directory Mirror Access

Pluggable transport can also reach:

```text
directory mirrors
bridge broker
update mirror
```

---

# 233. Trust

Artifacts still signature-verified.

---

# 234. UI Settings

Suggested:

```text
Network Access
    Automatic
    Direct
    Bridge
    Restricted Network
```

---

# 235. Advanced Settings

```text
Allow TCP fallback
Use bridges
Use aggressive obfuscation
```

---

# 236. Normal User Copy

Example:

```text
Restricted Network mode can use alternate transports and bridges when the anonymous network is blocked.
```

---

# 237. No "Stealth Guaranteed"

Hard rule.

---

# 238. Status UX

```text
Anonymous network connected
Connected through bridge
Restricted-network transport active
```

---

# 239. Failure UX

```text
Anonymous network appears blocked.
Trying alternate access method…
```

---

# 240. Strict Failure

If all anonymous access methods fail:

```text
Anonymous network unavailable.
Messages remain queued.
```

---

# 241. Never Auto-Downgrade to Ordinary Internet

Hard rule.

---

# 242. Diagnostics

Advanced:

```text
access mode
transport
bridge health
last failure class
directory mirror status
```

---

# 243. Diagnostics Redaction

No:

```text
bridge token
raw secret
full bridge pool
user IP
```

---

# 244. Testkit

Need fake censor/network emulator.

---

# 245. Censor Simulator

Capabilities:

```text
block UDP
block IP
block SNI
block TLS fingerprint
reset connections
DPI signature block
active probe
DNS poisoning
```

---

# 246. Unit Tests

Test:

```text
fallback ordering
bridge expiry
token validation
transport selection
anti-rollback config
```

---

# 247. Active Probe Tests

Unauthenticated probe does not reveal bridge service.

---

# 248. Enumeration Tests

Broker does not leak full bridge pool.

---

# 249. DNS Blocking Tests

Client can use signed bootstrap IP/mirror.

---

# 250. Directory Blocking Tests

Cached signed topology + bridge bundle works.

---

# 251. QUIC Blocking Tests

Fallback to approved TCP/WebSocket mode.

---

# 252. TLS Fingerprint Tests

Transport profile can rotate/update.

---

# 253. Bridge Revocation Tests

Client stops using revoked bridge.

---

# 254. Token Replay Tests

Rejected.

---

# 255. Update Blocking Tests

Signed alternate update path works.

---

# 256. Rollback Tests

Old censorship config rejected.

---

# 257. Process Death Tests

Reconnect without losing durable anonymous queue.

---

# 258. Android Tests

```text
mobile carrier
Wi-Fi
VPN
Doze
battery saver
network switch
```

---

# 259. Desktop Tests

```text
proxy
sleep/resume
firewall changes
daemon restart
```

---

# 260. Bulk Transfer Tests

Large anonymous transfer through restricted transport.

---

# 261. Abuse Tests

```text
connection flood
invalid tokens
active probes
resource exhaustion
```

---

# 262. Fuzzing

Fuzz:

```text
bridge descriptor
transport handshake parser
bootstrap bundle
signed config
```

---

# 263. Property Tests

Properties:

```text
strict anonymity never chooses non-anonymous fallback
revoked bridge never selected
expired bridge never selected
untrusted bootstrap bundle never accepted
```

---

# 264. Performance Tests

Measure:

```text
connect latency
throughput
CPU
memory
padding overhead
bridge capacity
```

---

# 265. Censorship Effectiveness Tests

Against synthetic censor only.

Do not treat as guarantee against real-world censor.

---

# 266. Privacy Leak Tests

No user identity in bridge protocol.

---

# 267. Metric Leak Tests

No raw bridge tokens/IPs in telemetry.

---

# 268. Crate Layout

Recommended:

```text
crates/
├── siar-censorship-core/
├── siar-bridge-core/
├── siar-bridge-broker/
├── siar-bridge-client/
├── siar-bridge-server/
├── siar-pluggable-transport/
├── siar-access-resolver/
├── siar-obfuscation/
├── siar-bootstrap-bundle/
├── siar-censorship-config/
├── siar-censorship-observability/
├── siar-censorship-sim/
└── siar-censorship-testkit/
```

---

# 269. `siar-censorship-core`

Owns:

```text
access modes
failure classes
policy
errors
```

---

# 270. `siar-bridge-core`

Bridge IDs/descriptors/tokens.

---

# 271. `siar-bridge-broker`

Bridge distribution.

---

# 272. `siar-bridge-client`

Client bridge connection.

---

# 273. `siar-bridge-server`

Bridge runtime.

---

# 274. `siar-pluggable-transport`

Trait and transport implementations.

---

# 275. `siar-access-resolver`

Selection/fallback/hysteresis.

---

# 276. `siar-obfuscation`

Padding/mimicry helpers.

---

# 277. `siar-bootstrap-bundle`

Signed offline bootstrap.

---

# 278. `siar-censorship-config`

Signed policy/config distribution.

---

# 279. `siar-censorship-observability`

Privacy-safe metrics.

---

# 280. `siar-censorship-sim`

Synthetic censor.

---

# 281. `siar-censorship-testkit`

Fault injection.

---

# 282. Error Taxonomy

```rust
pub enum AccessError {
    DirectBlocked,
    BridgeUnavailable,
    BridgeRevoked,
    BridgeExpired,
    TransportBlocked,
    ProbeFailed,
    DirectoryUnavailable,
    BootstrapInvalid,
    ConfigRollback,
    UnsupportedNetwork,
    ResourceRestricted,
    Internal,
}
```

---

# 283. Security Invariants

Mandatory:

```text
1. Censorship fallback never becomes direct peer communication under strict anonymity.
2. Bridges forward only approved SIAR anonymous traffic and are never open proxies.
3. Bridge descriptors and bootstrap bundles are signed and freshness-checked.
4. Bridge access tokens are secret, scoped, expiring, and never logged.
5. Unauthenticated active probes must not trivially reveal bridge behavior where transport supports probe resistance.
6. Access transport changes do not alter SIAR E2EE or Sphinx payload semantics.
7. Directory/update mirrors are distribution channels only; signatures establish trust.
8. Dynamic censorship config is anti-rollback protected.
9. Transport plugins cannot access message plaintext, E2EE keys, mailbox secrets, or file capability secrets.
10. Failure to reach anonymous network results in queued messages, not privacy downgrade.
11. Normal diagnostics do not expose bridge pools or tokens.
12. Production claims never promise unblockability or undetectability.
```

---

# 284. Initial Production Scope

Implement first:

```text
AnonymousAccessMode
bridge descriptors
bridge broker
single-bridge entry
QUIC direct access
TCP fallback
WebSocket/443 transport
signed bootstrap bundle
signed censorship config
bridge revocation
fallback resolver
hysteresis
privacy-safe diagnostics
synthetic censor testkit
```

Then add:

```text
probe-resistant obfuscation
advanced HTTPS-like mimicry
multiple bridge distribution channels
anonymous bridge broker requests
peer-assisted bootstrap
signed offline update distribution
bulk-transfer censorship adapters
```

---

# 285. Definition of Done

Part 41 is complete when:

- censorship resistance is cleanly separated from mixnet routing and application E2EE
- bridge node role and trust boundaries are defined
- bridge descriptors, epochs, tokens, and rotation are specified
- bridge discovery and enumeration resistance are defined
- pluggable transport abstraction supports multiple access mechanisms
- QUIC/TCP/WebSocket/HTTPS-like access classes are represented
- DPI/active-probing threat model is explicit
- active-probing resistance and secret-based bridge admission are accounted for
- DNS/directory blocking and signed offline bootstrap are handled
- dynamic censorship configuration is signed and anti-rollback protected
- fallback ordering/hysteresis never weakens anonymity
- Android/Desktop lifecycle, VPN/proxy/network-switch behavior are defined
- update survivability and alternate signed artifact distribution are accounted for
- bridge abuse/DoS protection and non-open-proxy invariant are defined
- diagnostics/metrics remain privacy-safe
- censor simulation, active-probe, revocation, rollback, update-blocking, and performance tests are specified
- the architecture remains transport-pluggable so censorship countermeasures can evolve independently

---

# 286. Final Architecture

```text
                       SIAR CLIENT
                           │
                           ▼
                 ACCESS RESOLUTION POLICY
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
      Direct            Bridge          Obfuscated
      QUIC              Access          Transport
         │                 │                 │
         └─────────────────┼─────────────────┘
                           ▼
                  Approved Anonymous Gateway
                           │
                           ▼
                         Mixnet
```

Bootstrap path:

```text
Signed bootstrap bundle
    ↓
Bridge descriptors
    ↓
Pluggable transport
    ↓
Authenticated bridge
    ↓
Anonymous gateway
```

Failure semantics:

```text
Anonymous access blocked
    ↓
Try approved alternate access
    ↓
If all fail
    ↓
Queue message

NOT

fall back to direct peer transport
```

---

# 287. Final Principle

Censorship resistance should make SIAR's anonymous network **reachable through changing network conditions** without changing what the anonymity system fundamentally trusts.

The correct model is:

```text
signed bridge discovery
+
pluggable transports
+
obfuscated access
+
active-probe resistance
+
multi-path bootstrap
+
strict no-downgrade policy
```

not:

```text
anonymous network blocked
→ bypass anonymity
```

This architecture gives SIAR a durable access layer that can evolve as filtering techniques change while preserving the anonymity, E2EE, routing, and metadata-protection guarantees established in Parts 34–40.
