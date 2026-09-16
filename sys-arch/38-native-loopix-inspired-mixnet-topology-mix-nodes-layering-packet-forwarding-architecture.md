# Core System Architecture Part 38 — Native Loopix-Inspired Mixnet Topology, Mix Nodes, Layering & Packet Forwarding Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 38  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:**  
- Part 34 — Mixnet, Loopix, Sphinx, Nym & High-Anonymity Transport Architecture  
- Part 35 — Anonymous Mailbox, Offline Receiving & Reply Capability Architecture  
- Part 36 — Sphinx Packet, Cell, Fragmentation & Anonymous Message Framing Architecture  
- Part 37 — Cover Traffic, Traffic Shaping, Timing Obfuscation & Loop Traffic Architecture  

**Primary purpose:** define the complete SIAR-native mixnet network plane, including client/gateway/mix/mailbox roles, stratified topology, route construction, per-hop packet forwarding, delay queues, node selection boundaries, topology snapshots, node lifecycle, congestion, failover, resource isolation, deployment patterns, observability, simulation, and Rust crate/service boundaries.

---

# 1. Purpose

Parts 34–37 defined the client-side anonymity architecture:

```text
privacy modes
anonymous transport abstraction
mailboxes
Sphinx-style framing
fixed-size cells
cover traffic
loop traffic
randomized emission
```

Part 38 defines the **network that those clients can use natively** if SIAR eventually runs its own mixnet.

The governing principle is:

> **A native SIAR mixnet must be layered, stateless where practical, privacy-preserving by construction, operationally observable without exposing routes, and independently evolvable from the application layer.**

---

# 2. Architectural Position

```text
SIAR Client
   │
   ▼
Entry / Gateway
   │
   ▼
Mix Layer 1
   │
   ▼
Mix Layer 2
   │
   ▼
Mix Layer 3
   │
   ▼
Exit / Mailbox / Recipient Gateway
   │
   ▼
Recipient
```

---

# 3. Why Stratified Layers

A stratified topology provides:

```text
clear path structure
bounded route length
simpler node selection
operational isolation
easier health measurement
reduced arbitrary-path complexity
```

---

# 4. Recommended Initial Topology

```text
Client
→ Gateway
→ Layer 1 Mix
→ Layer 2 Mix
→ Layer 3 Mix
→ Destination Gateway/Mailbox
```

---

# 5. Role Taxonomy

```rust
pub enum MixnetNodeRole {
    Gateway,
    MixLayer1,
    MixLayer2,
    MixLayer3,
    MailboxGateway,
    DirectoryObserver,
}
```

---

# 6. Client Role

Client owns:

```text
route request
Sphinx packet construction
traffic shaping
cover traffic
loop traffic
mailbox interaction
```

---

# 7. Gateway Role

Gateway bridges:

```text
client session
↔
mixnet packet ingress
```

Responsibilities:

```text
accept authenticated client transport
rate limit
validate packet envelope
queue packets
forward to first mix layer
```

---

# 8. Gateway Must Not Learn Final Destination

Hard goal.

---

# 9. Gateway Must Not Learn Full Route

Only:

```text
next hop
```

from Sphinx processing.

---

# 10. Mix Node Role

Each mix node:

```text
receives packet
processes one Sphinx layer
learns next hop only
applies/observes delay metadata as protocol requires
queues packet
forwards packet
```

---

# 11. Mailbox Gateway Role

Mailbox gateway handles:

```text
anonymous mailbox deposit
anonymous mailbox fetch
bounded retention
ACK
quota
```

without learning application plaintext.

---

# 12. Directory Observer

Optional control-plane role.

Collects signed node metadata and health signals.

Does not handle user packets.

---

# 13. Data Plane vs Control Plane

Strictly separate:

```text
Data Plane:
    packets
    delays
    forwarding
    mailboxes

Control Plane:
    node registration
    topology snapshots
    health
    revocation
    policy
```

---

# 14. Data Plane Statelessness

Mix nodes should be as stateless as possible except:

```text
delay queue
replay cache
connection state
health counters
```

---

# 15. No Conversation State in Mix Nodes

Hard rule.

---

# 16. No User Account State in Mix Nodes

Hard rule.

---

# 17. Topology Shape

Initial:

```text
N1 nodes in Layer 1
N2 nodes in Layer 2
N3 nodes in Layer 3
```

---

# 18. Route Length

Fixed route length preferred initially:

```text
Gateway + 3 mixes + destination gateway
```

---

# 19. Why Fixed Length

Reduces:

```text
route-length fingerprinting
complexity
compatibility risk
```

---

# 20. Variable Length

Possible later, but only if privacy analysis justifies it.

---

# 21. Node Identity

Each node needs a stable cryptographic identity for control-plane trust.

```rust
pub struct MixNodeId(pub [u8; 32]);
```

---

# 22. Node Identity Is Not User Identity

Separate key domains.

---

# 23. Node Key Types

Potential:

```text
identity signing key
transport key
Sphinx processing key
control-plane signing key
```

Key reuse should be minimized.

---

# 24. Node Descriptor

```rust
pub struct MixNodeDescriptor {
    pub node_id: MixNodeId,
    pub role: MixnetNodeRole,
    pub layer: Option<u8>,
    pub endpoint: NodeEndpoint,
    pub keys: MixNodePublicKeys,
    pub capabilities: MixNodeCapabilities,
    pub epoch: TopologyEpoch,
}
```

---

# 25. Node Endpoint

Contains network endpoint necessary for routing.

Do not embed operator PII.

---

# 26. Node Capabilities

```rust
pub struct MixNodeCapabilities {
    pub protocol_versions: Vec<MixnetProtocolVersion>,
    pub max_packet_size: usize,
    pub supports_quic: bool,
    pub supports_tcp_fallback: bool,
    pub supports_ipv6: bool,
}
```

---

# 27. Transport Between Nodes

Recommended:

```text
QUIC
```

for:

```text
multiplexing
congestion control
connection reuse
modern crypto
```

---

# 28. TCP Fallback

Optional for censorship/network compatibility.

---

# 29. Node-to-Node Transport Security

Transport security is separate from Sphinx anonymity.

---

# 30. Sphinx Still Required

Because TLS/QUIC alone reveals route path to intermediaries.

---

# 31. Layer Assignment

A node belongs to one mix layer per topology epoch.

---

# 32. Epoch-Based Topology

```rust
pub struct TopologyEpoch(pub u64);
```

---

# 33. Why Epochs

Allows:

```text
rotation
node joins/leaves
layer reshuffling
revocation
version upgrades
```

---

# 34. Epoch Duration

Policy-controlled.

Example:

```text
hours to days
```

depending network size and stability.

---

# 35. Layer Reshuffling

Nodes may change layer between epochs.

---

# 36. Privacy Benefit

Reduces long-term structural predictability.

---

# 37. Operational Cost

Frequent reshuffling causes:

```text
route churn
cache invalidation
connection churn
```

Need balance.

---

# 38. Topology Snapshot

Clients use signed snapshot:

```rust
pub struct TopologySnapshot {
    pub epoch: TopologyEpoch,
    pub layers: Vec<Vec<MixNodeDescriptor>>,
    pub gateways: Vec<MixNodeDescriptor>,
    pub mailboxes: Vec<MixNodeDescriptor>,
    pub signature: TopologySignature,
}
```

---

# 39. Snapshot Authenticity

Must be authenticated.

---

# 40. Snapshot Freshness

Clients reject overly stale snapshot.

---

# 41. Stale Snapshot UX

```text
Anonymous network topology is out of date.
```

Do not silently use untrusted nodes.

---

# 42. Topology Distribution

Possible:

```text
directory service
multiple signed mirrors
gossip among trusted control-plane nodes
```

---

# 43. Single Directory Risk

A single central directory is a central trust point.

---

# 44. Initial Deployment

May still use one authoritative directory while architecture remains ready for federation/multi-signature later.

---

# 45. Route Construction

Client selects one node from each layer.

```text
Gateway
→ L1
→ L2
→ L3
→ Destination
```

---

# 46. Route Selection Inputs

```text
topology snapshot
node eligibility
operator diversity
network diversity
region diversity
health
capacity
policy
```

---

# 47. Route Selection Must Avoid Bias

Poor selection can create anonymity weaknesses.

---

# 48. Selection API

```rust
pub trait MixRouteSelector {
    fn select_route(
        &self,
        snapshot: &TopologySnapshot,
        policy: &RouteSelectionPolicy,
        rng: &mut dyn CryptoRngCore,
    ) -> Result<MixRoute, RouteSelectionError>;
}
```

---

# 49. Route

```rust
pub struct MixRoute {
    pub gateway: MixNodeId,
    pub layers: [MixNodeId; 3],
    pub destination: MixDestination,
}
```

---

# 50. Operator Diversity

Avoid route where same operator controls multiple hops.

---

# 51. Operator Identity

Control plane needs operator grouping metadata.

---

# 52. Privacy

Operator metadata is network governance data, not user data.

---

# 53. ASN Diversity

Avoid all hops in same ASN where possible.

---

# 54. Geographic Diversity

Optional.

Avoid overfitting to geography.

---

# 55. Region Leakage

Do not force route based on user's exact location.

---

# 56. Latency vs Diversity

Need policy tradeoff.

---

# 57. Route Selection Policy

```rust
pub struct RouteSelectionPolicy {
    pub require_operator_diversity: bool,
    pub prefer_asn_diversity: bool,
    pub prefer_region_diversity: bool,
    pub health_floor: NodeHealthClass,
}
```

---

# 58. Maximum Anonymity

Stricter diversity requirements.

---

# 59. Interactive Anonymous Mode

May allow more latency-aware ranking after hard privacy constraints.

---

# 60. Hard Constraint First

Never:

```text
lowest latency
```

before:

```text
operator diversity
eligible layer
security validity
```

---

# 61. Route Reuse

Frequent reuse can increase correlation.

---

# 62. Route Rotation

Rotate routes periodically or per message batch.

---

# 63. Excessive Rotation Cost

Too much rotation causes:

```text
connection churn
latency
resource usage
```

---

# 64. Route Reuse Policy

```rust
pub enum RouteReusePolicy {
    PerPacket,
    ShortWindow,
    PerSession,
}
```

---

# 65. Initial Recommendation

```text
ShortWindow
```

for balance.

---

# 66. Sphinx Packet Construction

Client constructs layered packet:

```text
Hop 1 instructions
Hop 2 instructions
Hop 3 instructions
Destination instructions
```

encrypted so each hop reveals only its own layer.

---

# 67. Per-Hop Processing

Conceptually:

```text
receive packet
→ validate replay tag
→ unwrap one layer
→ learn delay/next hop
→ enqueue
→ forward transformed packet
```

---

# 68. Node Must Not Know Previous Full Path

Only transport peer and packet-level info.

---

# 69. Node Must Not Know Future Full Path

Only next hop.

---

# 70. Delay Queue

Core privacy mechanism.

---

# 71. Delay Metadata

Provider/protocol determines how delay is encoded.

---

# 72. Delay Queue Entry

```rust
pub struct DelayedPacket {
    pub release_at: Instant,
    pub next_hop: NodeEndpoint,
    pub packet: ProviderPacket,
}
```

---

# 73. Queue Ordering

Min-heap / timer wheel.

---

# 74. High Scale

Timer wheel may outperform heap at scale.

Benchmark.

---

# 75. Delay Queue Bounds

Must enforce:

```text
max packets
max bytes
max delay
```

---

# 76. Queue Full

Apply backpressure/drop policy carefully.

---

# 77. Cover vs Real Indistinguishability

Node should not know whether payload is:

```text
real
cover
loop
```

where protocol permits.

---

# 78. Drop Policy

Node should not prioritize by hidden application class.

---

# 79. Packet Expiry

Expired packet may be dropped.

---

# 80. Replay Cache

Each node tracks provider-defined replay tags.

---

# 81. Replay Cache Requirements

```text
bounded
time-scoped
fast lookup
privacy-safe
```

---

# 82. Bloom/Cuckoo Filters

Possible for high-volume replay detection.

Need false-positive analysis.

---

# 83. False Positive Risk

Can cause legitimate packet drop.

---

# 84. Initial Recommendation

Use exact bounded structure until scale requires probabilistic structure.

---

# 85. Connection Pool

Maintain node-to-node persistent connections.

---

# 86. Connection Churn

Avoid connect-per-packet.

---

# 87. Multiplexing

QUIC streams/datagrams as protocol permits.

---

# 88. Packet Transport Mode

Consider:

```text
QUIC streams
QUIC datagrams
```

depending reliability needs.

---

# 89. Reliability

Mixnet forwarding may use hop-level reliable transport.

---

# 90. End-to-End Delivery

Still governed by mailbox/application ACK.

---

# 91. Hop ACK

Should not become application-visible.

---

# 92. Packet Retry

At hop level, retry carefully to avoid duplication.

---

# 93. Duplicate Safety

Sphinx/replay layer and application dedup protect downstream.

---

# 94. Node Health

Need local and control-plane health.

---

# 95. Health Signals

```text
uptime
packet processing success
queue depth
send latency
connection health
CPU
memory
```

---

# 96. Privacy-Safe Health

No per-user/per-route labels.

---

# 97. Node Health Class

```rust
pub enum NodeHealthClass {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
```

---

# 98. Health Reporting

Aggregate.

---

# 99. Health Freshness

Timestamped by epoch/interval.

---

# 100. Route Eligibility

Unhealthy node excluded.

---

# 101. Capacity Advertisement

Node may advertise:

```text
max packet rate
available queue class
supported versions
```

---

# 102. Trust Boundary

Self-reported capacity is not fully trusted.

---

# 103. Independent Measurement

Future observers can verify availability/performance.

---

# 104. Congestion

Node-level congestion must not collapse network.

---

# 105. Congestion States

```text
normal
elevated
severe
```

---

# 106. Congestion Handling

```text
backpressure upstream
shed cover first only if protocol permits
reject new ingress
protect memory
```

---

# 107. Mix Node Cannot Know Real vs Cover

Therefore shedding should be queue-level/protocol-level, not semantic.

---

# 108. Gateway May Distinguish Client Session Priority?

Avoid unless privacy model explicitly permits.

---

# 109. Global Congestion Signal

Could be control-plane aggregate.

---

# 110. Client Reaction

Reduce:

```text
attachment traffic
cover rate
```

without privacy downgrade.

---

# 111. Node Resource Isolation

Separate pools:

```text
network I/O
packet crypto
delay queue
control plane
metrics
```

---

# 112. No Unbounded Tokio Tasks

Hard rule.

---

# 113. Worker Pools

Bounded.

---

# 114. CPU Isolation

Packet crypto cannot starve control plane.

---

# 115. Memory Isolation

Delay queue has hard memory ceiling.

---

# 116. Disk Use

Mix nodes should avoid disk for transient packet queues where possible.

---

# 117. Crash Semantics

Transient packets in volatile queue may be lost.

Application/mailbox retry handles loss.

---

# 118. Persistent Queue

Could improve reliability but creates:

```text
disk I/O
forensic metadata
complexity
```

---

# 119. Initial Recommendation

Use memory-only transient mix delay queues.

---

# 120. Gateway Queue Durability

Separate decision.

Client has durable outbox.

---

# 121. Mailbox Durability

Mailbox storage is durable by design.

---

# 122. Node Startup

```text
load keys
load signed config
validate topology epoch
start listeners
connect control plane
announce ready
```

---

# 123. Node Shutdown

```text
stop ingress
drain bounded queue
close connections
publish unavailable
```

---

# 124. Graceful Drain

Bound by timeout.

---

# 125. Rolling Upgrade

Topology epochs help.

---

# 126. Version Compatibility

Node descriptor advertises supported protocol versions.

---

# 127. Mixed-Version Epoch

Allowed only if client route can find compatible path end-to-end.

---

# 128. Upgrade Strategy

```text
introduce new version
dual support
shift majority
retire old
```

---

# 129. Emergency Revocation

Compromised node:

```text
remove from next topology snapshot
```

---

# 130. Mid-Epoch Revocation

Need urgent revocation list.

---

# 131. Revocation Feed

Signed control-plane artifact.

---

# 132. Client Behavior

Reject revoked node even if current snapshot includes it.

---

# 133. Connection Handling

Drop connection to revoked node.

---

# 134. Route Rebuild

Immediately select new route.

---

# 135. No Silent Weakening

If topology too small to satisfy diversity constraints:

```text
Anonymous route unavailable
```

instead of weaker route under strict mode.

---

# 136. Minimum Network Size

Need policy:

```text
minimum eligible nodes per layer
minimum operator diversity
```

---

# 137. Bootstrap Network

Early network may be small.

---

# 138. Development Mode

Can relax thresholds explicitly.

---

# 139. Production Mode

Must enforce minimum anonymity set.

---

# 140. Anonymity Set Health

Expose coarse:

```text
Healthy
Reduced
Insufficient
```

---

# 141. Do Not Show Exact User Count as Privacy Guarantee

Network size alone is not anonymity proof.

---

# 142. Gateway Selection

Client chooses gateway separately.

---

# 143. Gateway Persistence

Could remain stable for a short window.

---

# 144. Gateway Rotation

Reduces long-term linkage.

---

# 145. Gateway Tradeoff

Frequent rotation increases connection cost.

---

# 146. Destination Gateway

Mailbox may be tied to designated gateway/provider.

---

# 147. Mailbox Distribution

Future:

```text
multiple mailbox gateways
```

for redundancy.

---

# 148. Loop Traffic Routes

Loop packets:

```text
client
→ gateway
→ mixes
→ same client's return gateway/mailbox
```

---

# 149. Loop Route Independence

Do not always reuse exact real-message route.

---

# 150. Loop Health

Can estimate:

```text
network-wide latency bucket
packet loss
```

---

# 151. Control Plane Architecture

Recommended modules:

```text
directory
topology builder
revocation
health collector
epoch signer
```

---

# 152. Topology Builder

Inputs:

```text
eligible nodes
role capabilities
operator groups
health
capacity
policy
```

Outputs:

```text
signed epoch topology
```

---

# 153. Layer Assignment Algorithm

Must be deterministic or auditable from signed inputs where possible.

---

# 154. Random Assignment

Use verifiable randomness if later decentralizing.

---

# 155. Initial Centralized Assignment

Acceptable for early deployment, clearly documented as trust assumption.

---

# 156. Operator Metadata

Node registration should include:

```text
operator ID
ASN
region
capacity class
```

with validation where feasible.

---

# 157. Registration Abuse

Malicious operator can lie.

Later parts must address Sybil resistance and attestation.

---

# 158. Separation of Concerns

Part 38 defines topology plumbing.

Part 39+ should handle:

```text
node admission
Sybil resistance
directory trust
governance
```

separately.

---

# 159. Node Discovery

Clients obtain:

```text
topology snapshot
revocation list
directory signatures
```

---

# 160. Bootstrap Trust

App ships with:

```text
directory trust anchors
```

or obtains them through secure update.

---

# 161. Trust Anchor Rotation

Signed transition.

---

# 162. No TOFU for Production Directory Root

Prefer explicit trust anchor.

---

# 163. Multi-Directory Future

Possible:

```text
threshold signatures
multiple directory authorities
```

---

# 164. Topology Cache

Client caches last valid snapshot.

---

# 165. Cache Use

Allowed during short directory outage if:

```text
not expired
not revoked
```

---

# 166. Expired Cache

Strict mode:

```text
stop new anonymous routes
```

---

# 167. Existing Inflight Packets

May complete.

---

# 168. Route Selection Cache

Do not cache indefinitely.

---

# 169. Packet Forwarding Pipeline

```text
socket receive
→ frame length check
→ packet parse
→ replay check
→ Sphinx unwrap
→ delay extraction
→ next-hop resolve
→ enqueue
→ release
→ transport send
```

---

# 170. Parser Bounds

Reject oversized packet before allocation.

---

# 171. Constant Packet Size

Recommended per protocol profile.

---

# 172. Packet Profile

```rust
pub struct MixnetPacketProfile {
    pub version: MixnetProtocolVersion,
    pub packet_size: usize,
    pub max_delay: Duration,
}
```

---

# 173. Protocol Version

```rust
pub struct MixnetProtocolVersion(pub u16);
```

---

# 174. Forwarding Context

```rust
pub struct ForwardingContext {
    pub epoch: TopologyEpoch,
    pub node_id: MixNodeId,
    pub now: Instant,
}
```

---

# 175. Next-Hop Resolution

Uses:

```text
topology snapshot
local connection map
```

---

# 176. Unknown Next Hop

Drop packet and record aggregate error.

---

# 177. Do Not Route Arbitrarily Outside Topology

Hard rule.

---

# 178. Packet Loop Detection

Sphinx path/replay semantics should prevent abuse.

---

# 179. TTL / Hop Count

Packet structure must enforce bounded hops.

---

# 180. Delay Abuse

Malicious sender should not request unbounded delay.

---

# 181. Max Delay

Enforced by node profile.

---

# 182. Invalid Delay

Clamp or reject according to protocol.

Prefer reject for malformed authenticated packet.

---

# 183. Clock

Delay queue uses monotonic clock.

---

# 184. No Wall-Clock Dependency

For forwarding timing.

---

# 185. Node Observability

Metrics:

```text
packets received
packets forwarded
packets dropped
queue depth
queue wait time
connection health
CPU
memory
```

---

# 186. Privacy Constraint

No:

```text
source user
destination user
route ID
frame ID
mailbox ID
```

labels.

---

# 187. Packet Counters

Aggregate by:

```text
role
epoch
protocol version
```

---

# 188. Tracing

Production distributed tracing is dangerous for anonymity.

---

# 189. No Per-Packet Cross-Node Trace ID

Hard rule.

---

# 190. Local Debug Trace

Only in isolated test/dev environment.

---

# 191. Production Logs

Minimal.

---

# 192. Error Logging

Use:

```text
error class
counter
node-local timestamp
```

without packet identifiers.

---

# 193. Support Bundle

Node operators can export:

```text
aggregate metrics
config version
protocol version
health
```

---

# 194. Abuse Detection

Need coarse local defenses:

```text
connection flood
invalid packet flood
queue flood
replay flood
```

---

# 195. Rate Limiting

Gateway may rate-limit client ingress.

---

# 196. Mix Node Rate Limiting

Per transport peer, not per end user.

---

# 197. Peer Identity

Node-to-node authenticated transport identity.

---

# 198. DoS Isolation

Bad upstream node should not consume all resources.

---

# 199. Per-Peer Quota

```rust
pub struct PeerQuota {
    pub max_inflight_packets: u32,
    pub max_bytes_per_second: u64,
}
```

---

# 200. Abuse Quarantine

Temporarily isolate misbehaving peer connection.

---

# 201. Control Plane Alert

Aggregate abuse event.

---

# 202. Network Partition

Topology may split.

---

# 203. Partition Behavior

Nodes continue forwarding where next hop reachable.

---

# 204. Client Route Failure

Client retries new route from fresh topology/health.

---

# 205. Mailbox Availability

Independent of some mix failures if alternate routes exist.

---

# 206. Multi-Region Deployment

Recommended eventually.

---

# 207. Region Layout

Each layer should include nodes across multiple regions/operators.

---

# 208. Avoid Region-Specific Layer

Bad:

```text
Layer 1 = Europe
Layer 2 = US
```

creates structural leakage.

---

# 209. Better

Each layer is geographically diverse.

---

# 210. Edge Gateways

Gateways can be closer to clients.

---

# 211. Mix Layers

Should prioritize diversity over proximity.

---

# 212. Mailboxes

Can be regionally replicated only with privacy analysis.

---

# 213. Anycast

Potential for gateways, but identity/routing implications must be studied.

---

# 214. Deployment Modes

```rust
pub enum MixnetDeploymentMode {
    DevelopmentSingleHost,
    TestCluster,
    RegionalCluster,
    MultiRegionProduction,
}
```

---

# 215. Development Single Host

All roles on one machine for testing only.

---

# 216. Test Cluster

Separate processes/containers.

---

# 217. Regional Cluster

Multiple nodes/operators within regions.

---

# 218. Multi-Region Production

Multiple independent operators and ASNs.

---

# 219. Containerization

Useful for node deployment.

---

# 220. Isolation

One process per node role recommended.

---

# 221. Host Co-Location

Multiple logical nodes on same physical host reduce real diversity.

---

# 222. Production Rule

Do not count co-located nodes as independent anonymity hops.

---

# 223. Operator Diversity Metadata

Must reflect physical/administrative reality as much as possible.

---

# 224. Cloud Concentration

Avoid all nodes on one cloud provider.

---

# 225. Cloud Diversity

Policy can encourage:

```text
different providers
different ASNs
different regions
```

---

# 226. Bare Metal

Optional.

---

# 227. Home Nodes

Potential later.

Need reliability/Sybil controls.

---

# 228. Node Configuration

Human-readable RON:

```ron
(
    role: MixLayer2,
    listen: "0.0.0.0:4433",
    protocol_versions: [1],
    max_queue_packets: 100000,
    max_queue_bytes: 1073741824,
)
```

---

# 229. Secrets

Node private keys not stored directly in RON.

---

# 230. Secret Store

Use:

```text
OS secret store
encrypted key file
HSM
TPM
```

as deployment permits.

---

# 231. Key Rotation

Node keys rotate with signed transition.

---

# 232. Identity Rotation

More sensitive.

May require new node ID and topology registration.

---

# 233. Transport Cert Rotation

Routine.

---

# 234. Sphinx Key Rotation

Epoch-aligned where possible.

---

# 235. Graceful Key Overlap

Bounded overlap between old/new keys.

---

# 236. Compromise Response

```text
revoke node
rotate affected keys
publish emergency revocation
rebuild topology
```

---

# 237. Node Software Supply Chain

Require:

```text
pinned dependencies
SBOM
cargo-deny
cargo-audit
signed builds
reproducible-build effort
```

---

# 238. Reproducibility

Important for independent operators.

---

# 239. Binary Attestation

Future.

---

# 240. Remote Attestation

Not required initially; can create central trust complexity.

---

# 241. Rust Crate Layout

Recommended:

```text
crates/
├── siar-mixnet-core/
├── siar-mixnet-node/
├── siar-mixnet-gateway/
├── siar-mixnet-forwarder/
├── siar-mixnet-delay/
├── siar-mixnet-topology/
├── siar-mixnet-route/
├── siar-mixnet-directory-client/
├── siar-mixnet-mailbox-node/
├── siar-mixnet-observability/
├── siar-mixnet-sim/
└── siar-mixnet-testkit/
```

---

# 242. `siar-mixnet-core`

Owns:

```text
IDs
versions
roles
descriptors
shared errors
```

---

# 243. `siar-mixnet-node`

Common node runtime.

---

# 244. `siar-mixnet-gateway`

Client ingress/egress.

---

# 245. `siar-mixnet-forwarder`

Sphinx unwrap + next-hop forwarding.

---

# 246. `siar-mixnet-delay`

Delay queue implementation.

---

# 247. `siar-mixnet-topology`

Snapshots/epochs/layer representation.

---

# 248. `siar-mixnet-route`

Client-side route construction.

---

# 249. `siar-mixnet-directory-client`

Snapshot/revocation retrieval.

---

# 250. `siar-mixnet-mailbox-node`

Mailbox gateway integration.

---

# 251. `siar-mixnet-observability`

Privacy-safe metrics/logs.

---

# 252. `siar-mixnet-sim`

Discrete-event network simulator.

---

# 253. `siar-mixnet-testkit`

Fake nodes/topologies/fault injection.

---

# 254. Node Runtime API

```rust
#[async_trait]
pub trait MixNodeRuntime {
    async fn start(&self) -> Result<(), MixNodeError>;
    async fn shutdown(&self) -> Result<(), MixNodeError>;
    fn health(&self) -> NodeHealthSnapshot;
}
```

---

# 255. Packet Processor API

```rust
pub trait MixPacketProcessor {
    fn process(
        &self,
        packet: ProviderPacket,
        context: ForwardingContext,
    ) -> Result<ForwardingDecision, MixPacketError>;
}
```

---

# 256. Forwarding Decision

```rust
pub struct ForwardingDecision {
    pub next_hop: NodeEndpoint,
    pub delay: Duration,
    pub transformed_packet: ProviderPacket,
}
```

---

# 257. Delay Queue API

```rust
#[async_trait]
pub trait MixDelayQueue {
    async fn enqueue(
        &self,
        packet: DelayedPacket,
    ) -> Result<(), DelayQueueError>;

    async fn next_ready(
        &self,
    ) -> Result<DelayedPacket, DelayQueueError>;
}
```

---

# 258. Topology Provider API

```rust
pub trait TopologyProvider {
    fn current(&self) -> Arc<TopologySnapshot>;
    fn revocations(&self) -> Arc<RevocationSet>;
}
```

---

# 259. Route Selector API

```rust
pub trait RouteSelector {
    fn build(
        &self,
        topology: &TopologySnapshot,
        policy: &RouteSelectionPolicy,
    ) -> Result<MixRoute, RouteSelectionError>;
}
```

---

# 260. Health Snapshot

```rust
pub struct NodeHealthSnapshot {
    pub class: NodeHealthClass,
    pub queue_depth: u32,
    pub queue_bytes: u64,
    pub connection_count: u32,
    pub packet_rate: u64,
}
```

---

# 261. Simulation

Need native mixnet simulator before real deployment.

---

# 262. Simulation Inputs

```text
node count per layer
latency matrix
packet rate
delay distribution
node failure rate
operator grouping
capacity
```

---

# 263. Simulation Outputs

```text
delivery latency
queue growth
route diversity
node utilization
failure recovery
traffic concentration
```

---

# 264. Privacy Simulation

Can measure proxies:

```text
route overlap
operator overlap
timing correlation
anonymity set size
```

but not prove anonymity.

---

# 265. Failure Simulation

Inject:

```text
node crash
layer outage
gateway outage
network partition
high latency
packet loss
congestion
```

---

# 266. Route Diversity Test

Ensure selector avoids same operator across multiple hops.

---

# 267. Layer Exhaustion Test

Too few nodes:

```text
strict mode refuses route
```

---

# 268. Replay Test

Duplicate packet rejected.

---

# 269. Queue Exhaustion Test

No unbounded memory.

---

# 270. Delay Test

Release time obeyed within tolerance.

---

# 271. Clock Test

Monotonic behavior.

---

# 272. Topology Epoch Test

Old/new epoch coexistence bounded.

---

# 273. Revocation Test

Compromised node immediately excluded.

---

# 274. Version Upgrade Test

Mixed-version epoch compatibility.

---

# 275. Route Rotation Test

Short-window reuse then rotation.

---

# 276. Gateway Rotation Test

Client can move gateway without identity corruption.

---

# 277. Multi-Region Test

Route crosses independent operators/providers.

---

# 278. Co-Location Test

Selector does not count same physical/operator domain as diverse.

---

# 279. DoS Test

Bad peer cannot exhaust node.

---

# 280. Backpressure Test

Upstream slows when queue saturated.

---

# 281. Observability Leak Test

No route/user IDs in metrics.

---

# 282. Crash Test

Node crash loses only transient packets.

---

# 283. Mailbox Durability Test

Mailbox data survives independently.

---

# 284. Performance Tests

Measure:

```text
packets/sec
crypto CPU
queue latency
memory
connections
```

---

# 285. Scale Targets

Set benchmark tiers:

```text
1k pkt/s
10k pkt/s
100k pkt/s
```

depending hardware.

---

# 286. Horizontal Scaling

Mix nodes scale by adding more nodes to layers.

---

# 287. Vertical Scaling

Larger nodes increase throughput but reduce decentralization if overused.

---

# 288. Capacity Balancing

Topology builder can assign selection weights.

---

# 289. Weighted Selection

Must not create dominant nodes.

---

# 290. Maximum Weight

Cap selection probability.

---

# 291. Hotspot Avoidance

Use:

```text
capacity-aware
privacy-capped
```

weights.

---

# 292. Operator Concentration Limit

Topology policy may enforce max fraction per operator.

---

# 293. Cloud Concentration Limit

Likewise per cloud/ASN.

---

# 294. Node Join

Flow:

```text
generate identity
register
validate
wait eligibility
assign layer next epoch
```

---

# 295. Node Leave

Graceful:

```text
announce drain
finish epoch
remove next epoch
```

---

# 296. Unexpected Leave

Health marks unhealthy.

---

# 297. Churn

Topology must tolerate node churn without constant epoch rebuild.

---

# 298. Churn Threshold

Batch changes into next epoch unless urgent revocation.

---

# 299. Directory Availability

Clients can use cached valid snapshot.

---

# 300. Directory Outage

Does not immediately kill running routes.

---

# 301. Emergency Revocation Overrides Cache

Signed revocation must be fetched independently where possible.

---

# 302. Node Time Sync

Wall clock useful for control plane, but forwarding uses monotonic.

---

# 303. NTP Trust

Do not make packet security depend entirely on NTP correctness.

---

# 304. Epoch Timing

Allow bounded skew.

---

# 305. Security Threat Model

Adversaries include:

```text
malicious gateway
malicious mix node
colluding mix nodes
malicious mailbox
malicious directory
malicious operator
network observer
DoS attacker
compromised node key
```

---

# 306. Single Malicious Mix

Should not reveal full route.

---

# 307. Colluding First/Last

Can increase correlation risk.

---

# 308. Operator Diversity

Primary mitigation.

---

# 309. Global Observer

Requires cover/timing defenses from Part 37.

---

# 310. Directory Attack

Could bias routes by controlling topology.

---

# 311. Mitigation Roadmap

Later:

```text
multiple directory authorities
verifiable topology
auditable assignment
Sybil resistance
```

---

# 312. Node Fingerprinting

Standardize:

```text
packet sizes
protocol behavior
software versions where practical
```

---

# 313. Version Fingerprinting

Mixed versions can identify nodes.

---

# 314. Upgrade Window

Keep bounded.

---

# 315. Endpoint Censorship

Nodes may be blocked.

Later architecture should add:

```text
bridges
domain fronting alternatives where lawful/appropriate
obfuscation
```

---

# 316. Part 38 Boundary

Censorship resistance is not fully designed here.

---

# 317. Governance Boundary

Who is allowed to run nodes and how they are trusted is not fully designed here.

---

# 318. Economics Boundary

Incentives/tokenomics are explicitly outside Part 38.

---

# 319. Production Rollout Phases

Phase 1:

```text
single-operator private testnet
```

---

# 320. Phase 2

```text
multi-node staging
separate physical hosts
```

---

# 321. Phase 3

```text
multi-region internal production
```

---

# 322. Phase 4

```text
independent operators
```

after Sybil/admission architecture exists.

---

# 323. Development Testnet

Can run:

```text
1 gateway
3 mix nodes
1 mailbox
1 directory
```

---

# 324. Minimum Meaningful Staging

Prefer:

```text
2+ nodes per layer
multiple operators/hosts
```

for route testing.

---

# 325. Production Minimum

Must be policy-defined after privacy analysis.

---

# 326. No Fake Anonymity Claims

A 3-node single-host testnet is not "maximum anonymity".

---

# 327. User UX

Native provider appears as:

```text
SIAR Anonymous Network
```

not raw node list.

---

# 328. Advanced Diagnostics

Could show:

```text
network status
topology epoch
anonymity set health
provider version
```

---

# 329. Do Not Show Route Hops

Hard rule.

---

# 330. Node Operator UX

Separate admin/ops console.

---

# 331. Operator Console

Shows:

```text
health
queue
connections
version
resource usage
epoch
```

not user routing data.

---

# 332. Alerting

Examples:

```text
queue saturation
packet drop spike
key expiry
topology mismatch
revocation lag
```

---

# 333. SLOs

Node SLOs:

```text
availability
packet processing success
queue bound
forwarding latency
control-plane freshness
```

---

# 334. Privacy SLO

Examples:

```text
no route-identifier logging
no user identifiers in node telemetry
```

---

# 335. Audit

Periodic privacy review.

---

# 336. Formal Verification Future

Potential targets:

```text
route selector invariants
packet parser
replay state machine
topology validation
```

---

# 337. Security Invariants

Mandatory:

```text
1. Mix nodes never store conversation/account state.
2. Each hop learns only next-hop information required for forwarding.
3. Route length is fixed in the initial production profile.
4. Node selection enforces layer, health, revocation, and diversity constraints before latency optimization.
5. Same operator must not occupy multiple hops in strict mode.
6. Delay queues are bounded by packets, bytes, and time.
7. Replay caches are bounded.
8. Node telemetry contains no user/route/frame/mailbox identifiers.
9. Production tracing never uses cross-node per-packet trace IDs.
10. Compromised/revoked nodes are excluded immediately from new routes.
11. Cached topology cannot be used past security-defined freshness.
12. Co-located nodes do not count as independent anonymity hops.
13. Testnet topology is never presented as production anonymity.
```

---

# 338. Initial Production Scope

Implement first:

```text
3-layer stratified topology
gateway role
mix node role
mailbox gateway role
epoch snapshots
signed topology
client route selector
operator-diversity constraint
QUIC node transport
Sphinx per-hop processing
bounded delay queue
replay cache
health/capacity reporting
revocation feed
privacy-safe metrics
simulator/testkit
```

Defer:

```text
fully decentralized directory
public node admission
economic incentives
home-node participation
bridges/obfuscation
threshold directory authorities
formal anonymity proofs
```

---

# 339. Definition of Done

Part 38 is complete when:

- gateway, three mix layers, mailbox gateway, and control-plane roles are defined
- data plane and control plane are cleanly separated
- mix nodes remain free of application/account/conversation state
- topology is epoch-based and signed
- clients build fixed-length routes through one node per layer
- route selection supports operator/ASN/region diversity
- route reuse/rotation policy is explicit
- per-hop Sphinx processing learns only the next hop
- delay queues, replay caches, packet parsing, and connection pools are bounded
- QUIC is the primary node-to-node transport with optional fallback
- node health, congestion, capacity, and revocation are integrated
- node lifecycle, rolling upgrade, key rotation, and emergency revocation are defined
- privacy-safe observability forbids per-packet cross-node tracing
- multi-region and operator-diverse deployment principles are defined
- testnet/staging/production rollout phases are explicit
- simulator, fault injection, DoS, route diversity, replay, queue, revocation, and performance tests are specified
- clear boundaries remain for future Sybil resistance, directory decentralization, governance, and censorship resistance

---

# 340. Final Architecture

```text
                            CONTROL PLANE
        ┌───────────────────────────────────────────┐
        │ Directory │ Topology │ Health │ Revocation │
        └──────────────────────┬────────────────────┘
                               │
                        Signed Epoch Snapshot
                               │
                               ▼
                            CLIENT
                               │
                         Route Selection
                               │
                               ▼
                            GATEWAY
                               │
                               ▼
                         MIX LAYER 1
                               │
                         Delay / Forward
                               │
                               ▼
                         MIX LAYER 2
                               │
                         Delay / Forward
                               │
                               ▼
                         MIX LAYER 3
                               │
                         Delay / Forward
                               │
                               ▼
                     MAILBOX / DESTINATION
```

Per-hop path:

```text
Receive
  ↓
Bounds Check
  ↓
Replay Check
  ↓
Sphinx Unwrap
  ↓
Read Next Hop + Delay
  ↓
Bounded Delay Queue
  ↓
Forward
```

---

# 341. Final Principle

A native SIAR mixnet should not be a generic mesh of anonymous relays.

The correct model is:

```text
signed epoch topology
+
stratified mix layers
+
client-selected privacy-aware routes
+
per-hop Sphinx processing
+
bounded delay queues
+
operator diversity
+
privacy-safe observability
```

not:

```text
pick random relays
→ hope the route is anonymous
```

This architecture gives SIAR a concrete path toward its own Loopix-inspired anonymity network while keeping routing, node operation, privacy policy, and application messaging cleanly separated.
