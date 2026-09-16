# Core System Architecture Part 77 — Anonymous Network Service Discovery, Naming, Endpoint Resolution, Load Balancing & Traffic Steering Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 77  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 3, 11–18, 38–41, 48–50, 58–60, 62–76  

**Primary purpose:** define SIAR's service discovery, infrastructure naming, endpoint resolution, health-aware load balancing, failover, weighted/randomized selection, regional/provider diversity, traffic steering, anycast/relay selection, stale-cache behavior, tenant/federation isolation, and privacy constraints without creating a global tracking or correlation surface.

---

# 1. Purpose

Distributed anonymous infrastructure must discover and select services such as:

```text
mix gateways
mailbox providers
relays
bulk-transfer providers
directory mirrors
federation gateways
bridges
control-plane endpoints
```

Discovery and traffic steering can easily leak:

```text
where users are
which provider they prefer
which service they contacted
how often they reconnect
```

The governing principle is:

> **Service discovery should reveal only what is needed to find eligible infrastructure, and traffic steering should optimize reliability without creating stable user-to-endpoint affinity or weakening privacy constraints.**

---

# 2. Architectural Position

```text
Service Intent
    │
    ▼
Discovery Policy
    │
    ▼
Signed Service Records
    │
    ▼
Resolver
    │
    ▼
Eligible Endpoint Set
    │
    ▼
Load Balancer / Steering Policy
    │
    ▼
Selected Endpoint(s)
```

---

# 3. Core Separation

Keep distinct:

```text
service identity
provider identity
endpoint identity
host identity
DNS name
network address
routing policy
health status
```

---

# 4. Non-Goals

Part 77 does not create:

```text
one global user-visible infrastructure directory
stable per-user endpoint assignment
DNS-based account discovery
client IP–aware centralized steering by default
```

---

# 5. Service Classes

```rust
pub enum DiscoverableService {
    MixGateway,
    MailboxProvider,
    RealtimeRelay,
    BulkTransferProvider,
    Bridge,
    DirectoryMirror,
    FederationGateway,
    ControlPlaneEndpoint,
}
```

---

# 6. Service Identity

Stable logical service identity.

```rust
pub struct ServiceId(pub [u8; 32]);
```

---

# 7. Endpoint Identity

Rotatable network-serving identity.

```rust
pub struct EndpointId(pub [u8; 32]);
```

---

# 8. Host Identity

Part 71 infrastructure identity.

---

# 9. Hard Rule

ServiceId ≠ EndpointId ≠ HostAttestationId.

---

# 10. Why

Rotation and failover should not collapse all infrastructure identities.

---

# 11. Service Record

```rust
pub struct ServiceRecord {
    pub service_id: ServiceId,
    pub service_type: DiscoverableService,
    pub endpoints: Vec<EndpointDescriptor>,
    pub capabilities: ServiceCapabilities,
    pub valid_until: Timestamp,
    pub signatures: ServiceRecordSignatureBundle,
}
```

---

# 12. Endpoint Descriptor

```rust
pub struct EndpointDescriptor {
    pub endpoint_id: EndpointId,
    pub transport: TransportEndpoint,
    pub region: RegionId,
    pub provider: ProviderId,
    pub health_class: HealthClass,
}
```

---

# 13. Transport Endpoint

```rust
pub enum TransportEndpoint {
    Quic(SocketAddr),
    Tls(SocketAddr),
    OnionLike(OpaqueRouteDescriptor),
    Bridge(OpaqueBridgeDescriptor),
}
```

---

# 14. Public IP Exposure

Only for infrastructure endpoints intended to be public.

---

# 15. Client Identity

Never embedded in service records.

---

# 16. Record Signing

Required for authoritative records.

---

# 17. Signature Authority

Depends on service class.

---

# 18. Directory-Controlled Service Record

Signed by authorized provider/directory authority.

---

# 19. Federation Record

Signed by remote federation domain.

---

# 20. No Unsigned Discovery Record In Production

Hard rule.

---

# 21. Record Lifetime

Short enough for revocation/failover.

---

# 22. But

Long enough for offline/partition resilience.

---

# 23. Validity Window

```rust
pub struct ServiceRecordValidity {
    pub not_before: Timestamp,
    pub not_after: Timestamp,
}
```

---

# 24. Clock Privacy

Part 60.

---

# 25. Resolver

```rust
pub trait ServiceResolver {
    fn resolve(
        &self,
        query: ServiceQuery,
    ) -> Result<ServiceResolution, DiscoveryError>;
}
```

---

# 26. Service Query

```rust
pub struct ServiceQuery {
    pub service: DiscoverableService,
    pub constraints: DiscoveryConstraints,
}
```

---

# 27. Discovery Constraints

```rust
pub struct DiscoveryConstraints {
    pub privacy_mode: PrivacyRoutingMode,
    pub allowed_regions: Option<BTreeSet<RegionId>>,
    pub forbidden_providers: BTreeSet<ProviderId>,
    pub required_capabilities: BTreeSet<ServiceCapability>,
}
```

---

# 28. No User Identity In Query

Hard rule.

---

# 29. Resolver Privacy

Do not send:

```text
account ID
contact ID
conversation ID
```

to discovery infrastructure.

---

# 30. Exact Service Query

Use service class only.

---

# 31. Private Service Discovery

For internal enterprise/federation services.

---

# 32. Public Infrastructure Discovery

For public relays/gateways/providers.

---

# 33. Discovery Sources

```rust
pub enum DiscoverySource {
    SignedDirectory,
    CachedDirectory,
    Dns,
    StaticConfig,
    FederationRecord,
    LocalPeerCache,
}
```

---

# 34. Signed Directory

Preferred for anonymous infrastructure.

---

# 35. Cached Directory

Important for partition tolerance.

---

# 36. DNS

Useful for bootstrap and operator-facing control endpoints.

---

# 37. Static Config

Useful for air-gapped/self-hosted deployments.

---

# 38. Federation Record

Peer/domain-specific.

---

# 39. Local Peer Cache

Short-lived.

---

# 40. Source Precedence

```text
trusted signed directory
> valid cache
> signed federation record
> static policy
> DNS bootstrap
```

---

# 41. DNS Is Not Authority For Cryptographic Identity

Hard rule.

---

# 42. DNS Role

Resolve human-readable bootstrap names to candidate infrastructure.

---

# 43. Then

Validate signed service identity.

---

# 44. No TOFU For Critical Service

Hard rule.

---

# 45. DNSSEC

Can be additional assurance.

---

# 46. But

Still not sole trust root.

---

# 47. DoH/DoT

Optional.

---

# 48. Privacy Caveat

Central resolver can observe lookup patterns.

---

# 49. Recommendation

Use DNS only for coarse bootstrap, then rely on signed cached service descriptors.

---

# 50. No Per-Message DNS Lookup

Hard rule.

---

# 51. Name Classes

```rust
pub enum InfrastructureNameClass {
    HumanBootstrapName,
    ServiceLogicalName,
    EndpointOpaqueName,
}
```

---

# 52. Human Bootstrap Name

Example:

```text
gateway.example.net
```

---

# 53. Service Logical Name

Internal typed identity.

---

# 54. Endpoint Opaque Name

Short-lived/rotatable.

---

# 55. No Global User-Facing Endpoint Naming

Hard rule.

---

# 56. Service Namespace

Domain-scoped.

---

# 57. Federation

Remote domain keeps own namespace.

---

# 58. Tenant Internal Service

Tenant-scoped.

---

# 59. No Flat Global Namespace

Hard rule.

---

# 60. Endpoint Set

Resolver returns eligible candidate set.

---

# 61. Resolution Result

```rust
pub struct ServiceResolution {
    pub service: ServiceId,
    pub endpoints: Vec<ResolvedEndpoint>,
    pub valid_until: Timestamp,
}
```

---

# 62. Resolved Endpoint

```rust
pub struct ResolvedEndpoint {
    pub endpoint: EndpointDescriptor,
    pub trust: EndpointTrustState,
    pub health: HealthClass,
}
```

---

# 63. Trust State

```rust
pub enum EndpointTrustState {
    Trusted,
    Degraded,
    Ineligible,
}
```

---

# 64. Ineligible

Never selected.

---

# 65. Health

Separate from trust.

---

# 66. Healthy but untrusted

Not selectable.

---

# 67. Unhealthy but trusted

Not preferred.

---

# 68. Endpoint Eligibility

```rust
pub trait EndpointEligibilityEvaluator {
    fn eligible(
        &self,
        endpoint: &ResolvedEndpoint,
        policy: &SteeringPolicy,
    ) -> bool;
}
```

---

# 69. Selection Inputs

```text
trust
health
capacity
region
provider diversity
latency class
privacy mode
```

---

# 70. No User Wealth/Reputation

Hard rule.

---

# 71. Load Balancing

Not classic sticky web balancing.

---

# 72. Goals

```text
reliability
capacity utilization
diversity
privacy
```

---

# 73. Selection Policy

```rust
pub enum LoadBalancingStrategy {
    RandomEligible,
    WeightedRandom,
    PowerOfTwoChoices,
    LatencyClassRandom,
    DiversityAware,
}
```

---

# 74. RandomEligible

Strong privacy baseline.

---

# 75. WeightedRandom

Can account for coarse capacity.

---

# 76. PowerOfTwoChoices

Efficient load balancing without global optimization.

---

# 77. LatencyClassRandom

Uses coarse latency buckets.

---

# 78. DiversityAware

Enforces operator/ASN/region diversity.

---

# 79. Hard Rule

No globally stable endpoint affinity by user identity.

---

# 80. Sticky Sessions

Avoid by default.

---

# 81. When Needed

Realtime call relay session may remain sticky for session lifetime.

---

# 82. Session-Scoped Affinity

Allowed.

---

# 83. User-Lifetime Affinity

Forbidden.

---

# 84. Affinity Scope

```rust
pub enum AffinityScope {
    None,
    Session,
    Workflow,
}
```

---

# 85. No Account-Level Affinity

Hard rule.

---

# 86. Weighted Random

Weight by coarse class.

---

# 87. Endpoint Weight

```rust
pub struct EndpointWeight {
    pub capacity_class: CapacityClass,
    pub health_class: HealthClass,
}
```

---

# 88. No Exact Live Utilization To Clients

Hard rule.

---

# 89. Why

Exact utilization leaks operator internals and may create fingerprinting.

---

# 90. Health Classes

```rust
pub enum HealthClass {
    Healthy,
    Degraded,
    Draining,
    Unavailable,
}
```

---

# 91. Health Record

Signed/coarsened.

---

# 92. Draining

No new sessions.

---

# 93. Unavailable

Do not select.

---

# 94. Health Freshness

Bounded.

---

# 95. Stale Health

Treat cautiously.

---

# 96. No Assume Healthy Forever

Hard rule.

---

# 97. Health Source

```rust
pub enum HealthSource {
    LocalProbe,
    DirectorySnapshot,
    ProviderSignedStatus,
    SyntheticProbe,
}
```

---

# 98. Local Probe

Privacy-preserving.

---

# 99. Provider-Signed Status

Useful.

---

# 100. No User Traffic Inspection

Hard rule.

---

# 101. Synthetic Probes

Preferred for active health.

---

# 102. Probe Identity

Infrastructure-only.

---

# 103. No Probe Sharing User Identity

Hard rule.

---

# 104. Load Balancer Trait

```rust
pub trait ServiceLoadBalancer {
    fn select(
        &self,
        candidates: &[ResolvedEndpoint],
        context: &SelectionContext,
    ) -> Result<EndpointSelection, DiscoveryError>;
}
```

---

# 105. Selection Context

```rust
pub struct SelectionContext {
    pub privacy_mode: PrivacyRoutingMode,
    pub required_diversity: DiversityRequirement,
    pub affinity: AffinityScope,
}
```

---

# 106. Endpoint Selection

```rust
pub struct EndpointSelection {
    pub primary: ResolvedEndpoint,
    pub alternates: Vec<ResolvedEndpoint>,
}
```

---

# 107. Alternate Set

Useful for failover.

---

# 108. But

Do not expose huge provider list to client if unnecessary.

---

# 109. Candidate Minimization

Use only enough alternatives.

---

# 110. Diversity

Important.

---

# 111. Diversity Dimensions

```text
operator
region
ASN
cloud provider
jurisdiction
```

---

# 112. Diversity Requirement

```rust
pub struct DiversityRequirement {
    pub distinct_operators: u8,
    pub distinct_regions: u8,
    pub distinct_asns: Option<u8>,
}
```

---

# 113. Mixnet

Requires strong diversity.

---

# 114. Mailbox Replicas

Different failure domains.

---

# 115. Relay Alternates

Different provider/region if practical.

---

# 116. No Same-Operator Path Collapse

Hard rule when privacy mode requires diversity.

---

# 117. Service Discovery & Mixnet

Part 38/39.

---

# 118. Mix Route Selection

Not ordinary load balancing.

---

# 119. Directory gives eligible mix nodes.

---

# 120. Route selector enforces layer/operator diversity.

---

# 121. No Direct Client-to-All-Nodes Probing

Hard rule.

---

# 122. Mailbox Discovery

User receives opaque mailbox provider capability.

---

# 123. Provider rotation

New record.

---

# 124. No global mailbox search.

---

# 125. Relay Discovery

For realtime media.

---

# 126. Strict mode

Relay-only; no direct ICE fallback.

---

# 127. Relay Selection

Can use:

```text
coarse region
latency class
capacity class
```

---

# 128. No exact geolocation required.

---

# 129. Bulk Provider Discovery

Bandwidth/capacity-aware.

---

# 130. Still privacy-constrained.

---

# 131. Bridge Discovery

May be secret/private.

---

# 132. Bridge Descriptor

Sensitive capability.

---

# 133. No public bridge list if censorship threat model says otherwise.

---

# 134. Directory Mirror Discovery

Can use multiple mirrors.

---

# 135. Mirror Diversity

Recommended.

---

# 136. Control Plane Discovery

Infrastructure-only.

---

# 137. Can use internal DNS/service registry.

---

# 138. No mixing with user-plane discovery.

---

# 139. Anycast

Potential for:

```text
public relay ingress
bootstrap endpoint
```

---

# 140. Anycast Benefit

Simple nearest-path routing.

---

# 141. Anycast Risk

```text
route instability
operator centralization
correlation
```

---

# 142. Recommendation

Use only where operationally useful; signed service identity above network address remains authoritative.

---

# 143. No Trust Based On Anycast IP

Hard rule.

---

# 144. Traffic Steering

Broader than load balancing.

---

# 145. Steering Inputs

```text
privacy mode
jurisdiction
capacity
health
failover state
network reachability
```

---

# 146. Steering Policy

```rust
pub struct SteeringPolicy {
    pub privacy_floor: PrivacyRoutingMode,
    pub allowed_regions: BTreeSet<RegionId>,
    pub forbidden_providers: BTreeSet<ProviderId>,
    pub diversity: DiversityRequirement,
    pub failover: FailoverPolicy,
}
```

---

# 147. Failover Policy

```rust
pub enum FailoverPolicy {
    Strict,
    Bounded,
    OperatorApproved,
}
```

---

# 148. Strict

Only pre-approved equivalents.

---

# 149. Bounded

Use approved alternate set.

---

# 150. OperatorApproved

High-risk exceptional failover.

---

# 151. No Failover To Privacy-Weaker Path

Hard rule.

---

# 152. Stale Cache

Important for partitions.

---

# 153. Cache Entry

```rust
pub struct CachedServiceRecord {
    pub record: ServiceRecord,
    pub cached_at: Timestamp,
    pub stale_until: Timestamp,
}
```

---

# 154. Cache Freshness States

```rust
pub enum CacheFreshness {
    Fresh,
    StaleButUsable,
    Expired,
}
```

---

# 155. StaleButUsable

Only if signed record still within offline grace policy.

---

# 156. Expired

Do not use for security-critical changes.

---

# 157. No Infinite Stale Cache

Hard rule.

---

# 158. Offline Grace

Policy-defined.

---

# 159. Revocation Overrides Cache

Hard rule.

---

# 160. Directory Outage

Use cached signed records within grace.

---

# 161. New trust changes

Unavailable.

---

# 162. This is safe degradation.

---

# 163. Endpoint Rotation

Need overlap.

---

# 164. Rotation Flow

```text
publish new
→ overlap
→ prefer new
→ drain old
→ retire old
```

---

# 165. Compromise

Skip overlap where necessary.

---

# 166. Endpoint Revocation

Signed.

---

# 167. Revoked endpoint never selected from cache.

---

# 168. Hard rule.

---

# 169. Service Record Anti-Rollback

Version/epoch monotonic.

---

# 170. Record Epoch

```rust
pub struct ServiceRecordEpoch(pub u64);
```

---

# 171. Highest accepted epoch persisted.

---

# 172. Older record rejected.

---

# 173. No DNS rollback bypass.

---

# 174. Hard rule.

---

# 175. Resolver Cache Key

Service class + trust scope.

---

# 176. No User ID In Cache Key

Hard rule.

---

# 177. Per-Session Selection Randomness

CSPRNG.

---

# 178. No deterministic user hash steering.

---

# 179. Hard rule.

---

# 180. Latency Measurement

Could be local.

---

# 181. Coarse buckets.

---

# 182. Do not upload exact latency fingerprint.

---

# 183. Latency Class

```rust
pub enum LatencyClass {
    Low,
    Medium,
    High,
    Unknown,
}
```

---

# 184. Client local scorer

Preferred.

---

# 185. Central steering

Minimize.

---

# 186. Why

Central steering sees all selection requests.

---

# 187. Recommendation

Server publishes eligible set; client selects locally.

---

# 188. Hard Rule

Client-local selection preferred for privacy-sensitive services.

---

# 189. Server-Side Steering

Allowed for:

```text
control plane
internal infrastructure
enterprise managed traffic
```

---

# 190. User anonymous path

Prefer local choice.

---

# 191. Endpoint Scoring

Avoid precise total ordering if not needed.

---

# 192. Candidate Buckets

Better.

---

# 193. Example

```text
eligible-high
eligible-medium
```

then random choose.

---

# 194. Prevent deterministic fingerprints.

---

# 195. Load Feedback

Provider publishes coarse health/capacity.

---

# 196. No Per-Client Load Token

Hard rule.

---

# 197. Admission Control

Provider can reject new work when saturated.

---

# 198. Retry another eligible endpoint.

---

# 199. Retry Jitter

Required.

---

# 200. No Herding

Hard rule.

---

# 201. Power-of-Two Choices

Client samples two eligible endpoints.

---

# 202. Uses coarse metrics.

---

# 203. Good balance of privacy/performance.

---

# 204. Endpoint Cooldown

After failure.

---

# 205. Cooldown State

Local.

---

# 206. No global blacklist due one client failure.

---

# 207. Hard rule.

---

# 208. Circuit Breaker

Per endpoint/provider.

---

# 209. Local/infrastructure-scoped.

---

# 210. Breaker State

```rust
pub enum EndpointCircuitState {
    Closed,
    Open,
    HalfOpen,
}
```

---

# 211. Open

Temporarily avoid.

---

# 212. HalfOpen

Probe.

---

# 213. Health Aggregation

Server-side only for infrastructure.

---

# 214. User selection can use signed coarse class.

---

# 215. Tenant Isolation

Part 69.

---

# 216. Tenant Internal Service

Discovery namespace tenant-scoped.

---

# 217. Tenant endpoint record

Contains TenantId internally.

---

# 218. No cross-tenant service lookup unless explicit federation/share.

---

# 219. Hard rule.

---

# 220. Federation Discovery

Part 58.

---

# 221. Peer domain publishes federation gateway record.

---

# 222. Bilateral trust validation.

---

# 223. No global federation super-registry.

---

# 224. Hard rule.

---

# 225. Cross-Domain Endpoint

Validated against domain trust root.

---

# 226. No transitive trust.

---

# 227. Service Capability Advertisement

Versioned.

---

# 228. Capability Examples

```text
protocol versions
media relay support
mailbox retention class
PQ profile
```

---

# 229. No Sensitive Internal Software Inventory

Hard rule.

---

# 230. Capability Fingerprinting

Use standardized profiles.

---

# 231. Not arbitrary bitset where avoidable.

---

# 232. Service Capability Profile

```rust
pub enum ServiceCapabilityProfile {
    Baseline,
    Modern,
    HybridCrypto,
    HighCapacity,
}
```

---

# 233. Protocol Negotiation

Still separate.

---

# 234. Discovery only filters gross compatibility.

---

# 235. Endpoint Resolution Pipeline

```text
query
→ fetch records
→ verify signatures
→ anti-rollback check
→ apply legal/privacy filters
→ trust filter
→ health/capacity filter
→ diversity filter
→ local randomized selection
```

---

# 236. Ordering Important.

---

# 237. Privacy/Trust Filter Before Performance

Hard rule.

---

# 238. No "fastest wins" before trust checks.

---

# 239. Region Selection

Coarse.

---

# 240. User may choose:

```text
automatic
preferred region
avoid jurisdiction
```

---

# 241. Automatic Region

Based on network reachability/coarse latency.

---

# 242. No precise GPS requirement.

---

# 243. Hard rule.

---

# 244. Jurisdiction Constraints

Part 55.

---

# 245. Steering must obey.

---

# 246. Availability cannot override residency/forbidden region.

---

# 247. Hard rule.

---

# 248. Load Balancer Security

Potential attacks:

```text
endpoint poisoning
fake health
capacity manipulation
route steering attack
```

---

# 249. Signed Records

Mitigate poisoning.

---

# 250. Health Authenticity

Use signed provider health + local probes.

---

# 251. Capacity Lies

Possible.

---

# 252. Local observed failure can lower local score.

---

# 253. Directory/operator reputation not global user reputation.

---

# 254. Malicious Provider

Can advertise capacity but not force selection.

---

# 255. Randomization/diversity helps.

---

# 256. Steering Attack

Compromised directory might bias routes.

---

# 257. Multi-authority directory snapshots reduce risk.

---

# 258. Client-side diversity verification.

---

# 259. Hard rule.

---

# 260. No Single Directory Response Blindly Trusted In High-Anonymity Mode

Preferred.

---

# 261. Multi-source verification

Possible.

---

# 262. Directory Equivocation

Part 39/53.

---

# 263. Detect via signed snapshots/transparency.

---

# 264. Service Record Transparency

Optional.

---

# 265. No user query logging.

---

# 266. Endpoint Privacy

Do not persist client-endpoint relationship centrally.

---

# 267. Hard rule.

---

# 268. Provider Logs

Minimized.

---

# 269. Load Balancer Logs

Infrastructure aggregate.

---

# 270. No long-lived source-IP affinity logs.

---

# 271. If operationally unavoidable

Short retention.

---

# 272. Strict mode

Avoid.

---

# 273. Observability

Safe metrics:

```text
resolution success
eligible endpoint count
health distribution
selection failures
failover rate
```

---

# 274. Forbidden metrics

No:

```text
user-to-endpoint mapping
contact-related selection
conversation route history
```

---

# 275. Discovery SLO

Examples:

```text
resolution success
directory freshness
failover completion
```

---

# 276. Steering SLO

```text
eligible endpoint selected within target
```

---

# 277. Privacy SLO

```text
0 selections below privacy floor
```

---

# 278. Reliability SLO

```text
failover uses trusted alternate
```

---

# 279. Service Record Storage

Signed cache.

---

# 280. No giant central lookup DB required.

---

# 281. Client Cache

Encrypted local metadata if sensitive.

---

# 282. Cache size bounded.

---

# 283. Eviction

LRU/TTL.

---

# 284. No user correlation in cache key.

---

# 285. Bootstrap

Initial trust problem.

---

# 286. Bootstrap Bundle

Ships:

```text
trusted root fingerprints
initial directory endpoints
protocol version
```

---

# 287. Signed with release artifact.

---

# 288. No live secret.

---

# 289. Bootstrap Endpoint Failure

Multiple alternatives.

---

# 290. No Single Bootstrap Host

Hard rule.

---

# 291. Air-Gapped

Static signed records.

---

# 292. Self-Hosted

Admin-supplied signed records.

---

# 293. No DNS requirement.

---

# 294. Embedded/Edge

Static/local discovery possible.

---

# 295. Nearby discovery

Part 14/15.

---

# 296. Different from infrastructure discovery.

---

# 297. Do not merge proximity IDs with service endpoint IDs.

---

# 298. Hard rule.

---

# 299. Service Mesh

Optional.

---

# 300. Not required.

---

# 301. Service mesh risks metadata/complexity.

---

# 302. Internal control plane may use.

---

# 303. Data plane anonymous services should not depend on mesh.

---

# 304. Sidecar

Avoid mandatory per-packet sidecar.

---

# 305. Why

Performance/metadata/trust.

---

# 306. L4 Load Balancer

Possible.

---

# 307. But cryptographic service identity above it.

---

# 308. Reverse Proxy

Control-plane endpoints only.

---

# 309. No TLS termination that exposes E2EE content.

---

# 310. Load Balancer TLS

Transport termination may occur for public API.

---

# 311. Anonymous app E2EE unaffected.

---

# 312. Relay UDP/QUIC

Prefer direct service termination.

---

# 313. NAT/LB

Need connection-aware.

---

# 314. Anycast relay

Benchmark carefully.

---

# 315. Retry Semantics

Endpoint failure before app commit.

---

# 316. Idempotency protects.

---

# 317. No duplicate side effect due failover.

---

# 318. Hard rule.

---

# 319. Multi-Endpoint Racing

Happy Eyeballs–style possible.

---

# 320. But

Privacy concern: contacting multiple endpoints leaks more.

---

# 321. Strict mode

Avoid parallel racing across many providers.

---

# 322. Standard mode

May race limited alternatives.

---

# 323. Race Policy

```rust
pub enum EndpointRacePolicy {
    Sequential,
    LimitedParallel(u8),
}
```

---

# 324. Default anonymous

Sequential or very limited parallel.

---

# 325. No Wide Fanout.

---

# 326. Connection Reuse

Good for performance.

---

# 327. Privacy tradeoff.

---

# 328. Reuse scope

Provider/session.

---

# 329. No cross-identity reuse if it creates correlation.

---

# 330. Hard rule.

---

# 331. Resolver API

```rust
pub trait EndpointResolver {
    fn resolve_service(
        &self,
        query: &ServiceQuery,
    ) -> Result<ServiceResolution, DiscoveryError>;
}
```

---

# 332. Steering Engine

```rust
pub trait TrafficSteeringEngine {
    fn select(
        &self,
        resolution: &ServiceResolution,
        policy: &SteeringPolicy,
    ) -> Result<EndpointSelection, DiscoveryError>;
}
```

---

# 333. Health Provider

```rust
pub trait EndpointHealthProvider {
    fn health(
        &self,
        endpoint: &EndpointId,
    ) -> Result<HealthClass, DiscoveryError>;
}
```

---

# 334. Cache

```rust
pub trait ServiceRecordCache {
    fn get(
        &self,
        service: &ServiceId,
    ) -> Result<Option<CachedServiceRecord>, DiscoveryError>;

    fn put(
        &self,
        record: &ServiceRecord,
    ) -> Result<(), DiscoveryError>;
}
```

---

# 335. Anti-Rollback Store

```rust
pub trait ServiceRecordEpochStore {
    fn highest_seen(
        &self,
        service: &ServiceId,
    ) -> Result<Option<ServiceRecordEpoch>, DiscoveryError>;
}
```

---

# 336. Resolution Error Taxonomy

```rust
pub enum DiscoveryError {
    NoTrustedRecord,
    SignatureInvalid,
    RecordExpired,
    RecordRolledBack,
    NoEligibleEndpoint,
    DiversityUnsatisfied,
    RegionPolicyViolation,
    HealthUnavailable,
    CacheUnavailable,
    Internal,
}
```

---

# 337. Failure Semantics

No trusted endpoint:

```text
fail closed / queue
```

---

# 338. Not

```text
connect to arbitrary IP
```

---

# 339. Hard rule.

---

# 340. Censorship Mode

Bridge/pluggable transport can expand candidate set.

---

# 341. Still signed/authorized.

---

# 342. No random open proxy.

---

# 343. Offline Mode

Use cached/local/peer mesh services.

---

# 344. Infrastructure discovery may be unavailable.

---

# 345. DTN continues locally.

---

# 346. No forced direct internet fallback.

---

# 347. Testing

Need discovery/steering testkit.

---

# 348. Test Scenarios

```text
expired record
stale cache
revoked endpoint
fake health
region outage
provider outage
directory partition
```

---

# 349. Signature Test

Tampered record rejected.

---

# 350. Rollback Test

Old record rejected.

---

# 351. Revocation Test

Cached revoked endpoint not selected.

---

# 352. Diversity Test

Same-operator candidates rejected when policy requires distinct operators.

---

# 353. Region Policy Test

Forbidden region never selected.

---

# 354. Health Test

Draining endpoint gets no new sessions.

---

# 355. Failover Test

Trusted alternate selected.

---

# 356. Privacy Test

No user ID included in query/cache/topic/log.

---

# 357. Randomization Test

Selection not deterministic by account identity.

---

# 358. Parallel Race Test

Strict mode limits fanout.

---

# 359. Federation Test

Remote endpoint validated against remote domain trust root only.

---

# 360. Tenant Test

Cross-tenant internal service lookup rejected.

---

# 361. DNS Poisoning Test

DNS result alone cannot establish trust.

---

# 362. Directory Equivocation Test

Conflicting signed snapshots detected.

---

# 363. Fuzzing

Fuzz:

```text
service record
endpoint descriptor
capability profile
steering policy
```

---

# 364. Property Tests

Properties:

```text
untrusted endpoint is never selected
revoked record never becomes active through stale cache
privacy floor filter always runs before performance ranking
account identity never changes candidate weight
```

---

# 365. Formal Verification Targets

Strong candidates:

```text
eligibility filtering
diversity constraints
anti-rollback
failover selection
```

---

# 366. Kani Candidate

selection/filter invariants.

---

# 367. TLA+ Candidate

directory update + revocation + partition.

---

# 368. Loom Candidate

concurrent cache refresh + revocation.

---

# 369. Performance

Resolution should be fast.

---

# 370. But

No per-message remote lookup.

---

# 371. Cache hit target high.

---

# 372. Signature verification amortized.

---

# 373. Batch record verification possible.

---

# 374. Load Selection

O(n) over bounded candidate set.

---

# 375. No thousands of candidates per selection.

---

# 376. Candidate set bounded.

---

# 377. Memory

Cache bounded by count/bytes.

---

# 378. Network

Directory snapshots compressed if safe.

---

# 379. Compression side-channel not relevant for public metadata but still validate sizes.

---

# 380. Crate Layout

Recommended:

```text
crates/
├── siar-discovery-core/
├── siar-service-record/
├── siar-directory-resolver/
├── siar-dns-bootstrap/
├── siar-endpoint-health/
├── siar-load-balancer/
├── siar-traffic-steering/
├── siar-discovery-cache/
├── siar-discovery-observability/
└── siar-discovery-testkit/
```

---

# 381. `siar-discovery-core`

Owns:

```text
service IDs
endpoint IDs
queries
errors
```

---

# 382. `siar-service-record`

Signed service descriptor schema/verification.

---

# 383. `siar-directory-resolver`

Signed directory lookup.

---

# 384. `siar-dns-bootstrap`

Bootstrap-only DNS adapter.

---

# 385. `siar-endpoint-health`

Coarse health state/probes.

---

# 386. `siar-load-balancer`

Random/weighted/diversity-aware selection.

---

# 387. `siar-traffic-steering`

Policy/legal/failover constraints.

---

# 388. `siar-discovery-cache`

Signed cache/anti-rollback state.

---

# 389. `siar-discovery-observability`

Privacy-safe aggregate metrics.

---

# 390. `siar-discovery-testkit`

Poisoning/revocation/failover/diversity simulator.

---

# 391. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Service identity, endpoint identity, host identity, and user identity remain distinct.
2. DNS may bootstrap addresses but never establishes critical service cryptographic identity by itself.
3. Public/private service records are signed, versioned, time-bounded, and anti-rollback protected.
4. Trust/privacy/legal eligibility filtering occurs before latency or capacity optimization.
5. Anonymous clients do not send account/contact/conversation identifiers in discovery queries.
6. Endpoint selection is randomized or diversity-aware and never deterministically keyed by user identity.
7. Sticky affinity is limited to session/workflow scope, never permanent account-level routing.
8. Revoked endpoints cannot be resurrected from stale caches or DNS.
9. Failover never selects a privacy-weaker, legally forbidden, or untrusted endpoint merely to restore availability.
10. Tenant and federation service namespaces remain isolated; there is no global infrastructure super-registry.
11. Discovery/load-balancing telemetry never becomes a user-to-endpoint relationship database.
12. User-plane service selection is local where practical so central infrastructure does not observe every routing decision.
```

---

# 392. Initial Production Scope

Implement first:

```text
signed service-record schema
service/endpoint identity separation
signed directory resolver
bounded encrypted cache
anti-rollback epochs
DNS bootstrap only
coarse endpoint health
random/weighted/diversity-aware selection
local client steering
strict privacy/legal filters
endpoint revocation
region/provider failover
tenant/federation namespaces
privacy-safe metrics
discovery/steering testkit
```

Then add:

```text
multi-source directory verification
anycast adapters
advanced capacity-aware randomization
federated service-record transparency
adaptive path selection
formal steering verification
```

---

# 393. Definition of Done

Part 77 is complete when:

- service, endpoint, host, provider, tenant, federation, and user identities are separate
- authoritative service records are signed and anti-rollback protected
- DNS is bootstrap-only for critical infrastructure trust
- clients can operate from cached signed records during outages
- eligibility filters precede performance optimization
- endpoint selection is randomized/diversity-aware and privacy-safe
- no stable account-level affinity exists
- revocation overrides stale caches
- regional/provider failover preserves privacy/legal/trust constraints
- tenant/federation namespaces remain independent
- bridge/private-service discovery can remain capability-scoped
- health/load signals are coarse and non-user-specific
- poisoning/revocation/diversity/partition/fuzz/formal tests are specified

---

# 394. Final Architecture

```text
                     SERVICE INTENT
                           │
                           ▼
                    DISCOVERY POLICY
                           │
                           ▼
                 SIGNED SERVICE RECORDS
                           │
                           ▼
                     TRUST VALIDATION
                           │
                           ▼
             PRIVACY / LEGAL / DIVERSITY FILTER
                           │
                           ▼
                    ELIGIBLE ENDPOINT SET
                           │
                           ▼
               LOCAL RANDOMIZED LOAD BALANCER
                           │
                           ▼
                     SELECTED ENDPOINT
```

Service-discovery safety model:

```text
signed records
+
anti-rollback
+
local selection
+
randomization
+
diversity
+
coarse health
+
strict failover constraints
```

not:

```text
ask a central server which endpoint this user should use every time
```

---

# 395. Final Principle

Service discovery should help SIAR find infrastructure without creating a map of who uses which infrastructure.

The correct model is:

```text
signed eligibility
+
local resolution/cache
+
privacy-first filtering
+
randomized/diversity-aware steering
+
bounded failover
```

This architecture gives SIAR resilient service discovery and traffic steering for gateways, mailboxes, relays, bridges, federation gateways, directory mirrors, and control-plane services while preserving the anonymity and isolation guarantees established across Parts 34–76.
