# Core System Architecture Part 78 — Anonymous Network Edge Gateway, Ingress/Egress, Reverse Proxy, API Gateway, WAF & Boundary Enforcement Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 78  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 11, 28, 41, 47–53, 55, 58, 61–77  

**Primary purpose:** define SIAR's edge and boundary architecture, including public/private ingress, reverse proxies, API gateways, protocol demultiplexing, egress enforcement, WAF-like controls, request normalization, rate limiting, abuse prevention, TLS termination, tenant/federation isolation, zero-trust service entry, boundary logging, failover, and privacy-preserving enforcement.

---

# 1. Purpose

The edge is where untrusted networks meet trusted services.

A badly designed edge can become:

```text
a universal traffic observer
a secret-termination choke point
a single compromise domain
a cross-tenant routing oracle
a metadata warehouse
```

The governing principle is:

> **SIAR edge infrastructure must enforce boundaries without becoming a central point that can observe, correlate, or decrypt more than the protocol strictly requires.**

---

# 2. Architectural Position

```text
Internet / Federation / Private Network
                │
                ▼
        Edge Boundary Layer
                │
     ┌──────────┼──────────┐
     │          │          │
   L4 Gate    L7/API     Egress Gate
     │          │          │
     └──────────┼──────────┘
                ▼
        Internal Service Mesh
                │
                ▼
             SIAR Core
```

---

# 3. Core Separation

Keep distinct:

```text
ingress
egress
reverse proxy
API gateway
WAF
service gateway
federation gateway
anonymous data plane
control plane
```

---

# 4. Non-Goals

Part 78 does not create:

```text
one universal TLS termination layer
deep inspection of E2EE payloads
a single global API gateway for all protocols
user-level outbound traffic logging
```

---

# 5. Boundary Classes

```rust
pub enum BoundaryClass {
    PublicIngress,
    PrivateIngress,
    FederationIngress,
    ControlPlaneIngress,
    PublicEgress,
    RestrictedEgress,
}
```

---

# 6. Public Ingress

Internet-facing services.

Examples:

```text
bootstrap API
relay ingress
mailbox deposit/fetch
public directory mirrors
```

---

# 7. Private Ingress

Internal/admin/operator services.

---

# 8. Federation Ingress

Cross-domain peering.

---

# 9. Control Plane Ingress

Highly restricted admin/deployment/config surface.

---

# 10. Public Egress

Outbound to approved external providers.

---

# 11. Restricted Egress

Outbound denied by default unless policy allows.

---

# 12. Hard Rule

Anonymous client payload paths do not share the same trust boundary as admin/control-plane ingress.

---

# 13. Edge Node Roles

```rust
pub enum EdgeRole {
    L4Ingress,
    ApiGateway,
    ReverseProxy,
    EgressGateway,
    FederationGateway,
    WafEnforcer,
}
```

---

# 14. Role Separation

Can co-reside in small deployments.

---

# 15. But

Critical deployments should separate high-trust roles.

---

# 16. Edge Trust Zone

```rust
pub struct EdgeTrustZoneId(pub [u8; 32]);
```

---

# 17. No User Identity

Hard rule.

---

# 18. L4 Ingress

Operates at transport layer.

---

# 19. Good For

```text
QUIC
TCP/TLS pass-through
relay traffic
mixnet ingress
```

---

# 20. L4 Advantage

Less application metadata exposure.

---

# 21. L4 Design Goal

Pass encrypted traffic with minimal termination.

---

# 22. L7/API Gateway

Operates on application protocol.

---

# 23. Good For

```text
public REST/gRPC-like API
control plane
webhook endpoints
external integration APIs
```

---

# 24. Hard Rule

Do not force anonymous encrypted transport through L7 inspection merely for operational convenience.

---

# 25. Reverse Proxy

Routes requests to service backends.

---

# 26. Reverse Proxy Responsibilities

```text
TLS policy
backend selection
header normalization
connection limits
```

---

# 27. Reverse Proxy Non-Responsibilities

```text
business authorization
tenant policy decisions
crypto root authority
```

---

# 28. API Gateway

Responsible for:

```text
authentication
capability checks
rate limits
schema validation
version routing
```

---

# 29. Business Authorization

Still belongs to application layer.

---

# 30. Hard Rule

API gateway authorization cannot replace domain-level authorization.

---

# 31. WAF

Use narrowly.

---

# 32. WAF Scope

Protect:

```text
HTTP APIs
admin portals
webhooks
```

---

# 33. WAF Cannot Inspect

```text
E2EE message content
encrypted mixnet cells
opaque mailbox ciphertext
```

---

# 34. Hard Rule

WAF failure must not justify decrypting protected payloads.

---

# 35. Edge Policy

```rust
pub struct EdgePolicy {
    pub boundary: BoundaryClass,
    pub allowed_protocols: BTreeSet<ProtocolId>,
    pub max_connections: u64,
    pub request_limits: RequestLimitPolicy,
}
```

---

# 36. Protocol Allowlist

Explicit.

---

# 37. No Open Port By Default

Hard rule.

---

# 38. Ingress Port Matrix

Machine-readable.

---

# 39. Example

```text
443/QUIC → relay/mailbox
443/TLS  → public API
admin    → private network only
```

---

# 40. No Admin Port On Public Interface

Hard rule.

---

# 41. Protocol Demultiplexing

May use:

```text
ALPN
SNI
port
magic/version prefix
```

---

# 42. ALPN

Preferred for secure multiplexing where applicable.

---

# 43. SNI Privacy

Can reveal service hostname.

---

# 44. Recommendation

Use coarse service names or encrypted client hello where supported by ecosystem.

---

# 45. No Per-User SNI

Hard rule.

---

# 46. Endpoint Identity

Part 77.

---

# 47. Edge Address ≠ Service Identity

Hard rule.

---

# 48. TLS Termination Models

```rust
pub enum TlsTerminationMode {
    PassThrough,
    EdgeTerminate,
    ServiceTerminate,
    MutualTlsTerminate,
}
```

---

# 49. PassThrough

Best privacy for opaque traffic.

---

# 50. EdgeTerminate

Useful for web/API services.

---

# 51. ServiceTerminate

Good for sensitive service-specific endpoints.

---

# 52. MutualTlsTerminate

Control-plane/federation use.

---

# 53. Hard Rule

E2EE application payload remains E2EE regardless of transport termination.

---

# 54. TLS Private Keys

Scoped per service/domain.

---

# 55. No Shared Wildcard Key Across All Critical Services

Hard rule.

---

# 56. Certificate Automation

Allowed.

---

# 57. But

Certificate issuance authority separate from application root authority.

---

# 58. TLS Certificate Rotation

Automatic where safe.

---

# 59. Control Plane mTLS

Recommended.

---

# 60. Federation mTLS

Possible in addition to signed federation identities.

---

# 61. No TLS Certificate = Governance Trust

Hard rule.

---

# 62. Request Normalization

Critical for L7 APIs.

---

# 63. Normalize

```text
method
path
headers
content length
encoding
```

---

# 64. Reject Ambiguity

Hard rule.

---

# 65. Request Smuggling

Must defend.

---

# 66. Conflicting Length Headers

Reject.

---

# 67. Duplicate Sensitive Headers

Reject/normalize deterministically.

---

# 68. Header Size Bounds

Mandatory.

---

# 69. Body Size Bounds

Mandatory.

---

# 70. URL/Path Length Bounds

Mandatory.

---

# 71. Parser Differential

Avoid multiple parsers interpreting differently.

---

# 72. Canonical Request Model

```rust
pub struct CanonicalRequest {
    pub method: Method,
    pub path: NormalizedPath,
    pub headers: CanonicalHeaders,
    pub body_len: usize,
}
```

---

# 73. No Raw Header Map Into Domain Layer

Preferred.

---

# 74. API Schema Validation

Strict.

---

# 75. Unknown Required Field

Reject safely.

---

# 76. Oversized Payload

Reject before buffering.

---

# 77. Streaming Upload

Use bounded streaming.

---

# 78. No Full Multi-GB Buffer In Edge

Hard rule.

---

# 79. Request Authentication

Depends on API class.

---

# 80. Public Anonymous API

May use capability/token/proof.

---

# 81. Private Admin API

mTLS + strong operator auth.

---

# 82. Federation API

domain-scoped authenticated peer identity.

---

# 83. Webhook

signature + replay protection.

---

# 84. No Cookie-Based Auth For Headless Critical APIs

Preferred hard rule.

---

# 85. Capability Security

Part 68.

---

# 86. Edge verifies coarse capability.

---

# 87. Application re-validates business authorization.

---

# 88. No Edge-Only Trust

Hard rule.

---

# 89. Rate Limiting

Boundary-scoped.

---

# 90. Rate Limit Inputs

Use:

```text
service capability
token
tenant
coarse source class
```

---

# 91. Avoid

```text
global user identity
```

---

# 92. Anonymous Rate Limiting

Can use:

```text
proof tokens
prepaid credits
coarse source network
```

---

# 93. No Permanent IP Reputation

Hard rule for anonymous services.

---

# 94. IP-Based Limits

May be coarse/temporary.

---

# 95. NAT Fairness

Consider.

---

# 96. Rate Limit Type

```rust
pub struct EdgeRateLimit {
    pub max_requests: u64,
    pub window: Duration,
    pub burst: u64,
}
```

---

# 97. Token Bucket

Useful.

---

# 98. Leaky Bucket

Possible.

---

# 99. Per-Connection Limits

Required.

---

# 100. Global Service Limits

Required.

---

# 101. Tenant Limits

Part 69.

---

# 102. No Tenant A Starves Tenant B

Hard rule.

---

# 103. Abuse Resistance

Edge handles:

```text
connection flood
request flood
oversized body
slowloris
malformed frame
```

---

# 104. Connection Admission

Bounded.

---

# 105. Handshake Budget

Bounded.

---

# 106. Slowloris Defense

Idle/read deadlines.

---

# 107. Header Deadline

Required.

---

# 108. Request Deadline

Per endpoint.

---

# 109. Long-Lived Streaming

Separate policy.

---

# 110. Relay Sessions

Long-lived but bandwidth-limited.

---

# 111. No Generic Long Timeout

Hard rule.

---

# 112. Connection State Budget

```rust
pub struct ConnectionBudget {
    pub max_active: u64,
    pub max_handshakes_per_second: u64,
    pub max_idle: Duration,
}
```

---

# 113. SYN/UDP Flood

Host/network mitigation layer.

---

# 114. DDoS Provider

Optional external service.

---

# 115. Privacy Caveat

Third-party DDoS proxy can observe metadata.

---

# 116. Hard Rule

High-anonymity path must not silently route through metadata-heavy DDoS proxy without policy approval.

---

# 117. Anycast DDoS Front

Useful for public control/API endpoints.

---

# 118. Less suitable for sensitive anonymous paths unless explicitly designed.

---

# 119. WAF Rule Classes

```rust
pub enum WafRuleClass {
    ProtocolViolation,
    KnownExploitPattern,
    RequestSmuggling,
    HeaderAbuse,
    PathTraversal,
    InjectionPattern,
}
```

---

# 120. Injection Rules

Only for plaintext APIs.

---

# 121. No Pattern Matching On Encrypted User Content

Hard rule.

---

# 122. False Positive Policy

Fail closed for clearly malformed protocol.

---

# 123. High-risk heuristic rule

Observe/limit before hard block where possible.

---

# 124. No Opaque ML WAF Requirement

Hard rule.

---

# 125. Explainable rules preferred.

---

# 126. API Gateway Version Routing

Routes by explicit API version.

---

# 127. No Silent Version Rewrite

Hard rule.

---

# 128. Deprecated Version

Can reject with migration guidance.

---

# 129. Protocol Gateway

May multiplex:

```text
Postcard binary API
JSON external API
webhook JSON
```

---

# 130. Internal Binary

Postcard.

---

# 131. External Interop

JSON only where needed.

---

# 132. No JSON Requirement On Internal Hot Path

Hard rule.

---

# 133. Federation Boundary

Part 58.

---

# 134. Federation Gateway Responsibilities

```text
authenticate domain
validate peering agreement
enforce service scope
reproject metadata
rate limit
```

---

# 135. Federation Gateway Must Not

```text
merge identity namespaces
accept transitive trust
```

---

# 136. Hard Rule

No raw internal event bus bridging across federation.

---

# 137. Tenant Boundary

Part 69.

---

# 138. API Gateway carries explicit TenantContext.

---

# 139. No Tenant Inferred Solely From Host Header

Hard rule.

---

# 140. Tenant Routing

Host/path/token can help, but signed/authorized tenant context required.

---

# 141. Cross-Tenant Request

Explicit capability only.

---

# 142. Edge Cache

Use carefully.

---

# 143. Good Cache Candidates

```text
public static metadata
signed service records
public release docs
```

---

# 144. Bad Cache Candidates

```text
private user responses
identity lookups
message content
```

---

# 145. No Shared Cache Of Sensitive Tenant/User Data

Hard rule.

---

# 146. Cache Key

Must include tenant/scope when applicable.

---

# 147. Cache Poisoning

Signatures/schema validation help.

---

# 148. Edge Compression

Potential.

---

# 149. Compression Side Channel

Avoid mixing secrets with attacker-controlled reflection.

---

# 150. Public static responses

Fine.

---

# 151. Sensitive API

Disable or isolate compression contexts.

---

# 152. Hard Rule

No dynamic cross-user compression dictionary for sensitive responses.

---

# 153. Egress Gateway

Controls outbound service traffic.

---

# 154. Default Policy

```text
deny
```

---

# 155. Allow Explicit Destinations

Hard rule.

---

# 156. Egress Policy

```rust
pub struct EgressPolicy {
    pub service: ServiceId,
    pub allowed_destinations: BTreeSet<EgressDestination>,
    pub allowed_protocols: BTreeSet<ProtocolId>,
}
```

---

# 157. Egress Destination

```rust
pub enum EgressDestination {
    Domain(DomainName),
    IpNet(IpNetwork),
    FederationPeer(FederationDomainId),
    Provider(ProviderId),
}
```

---

# 158. No Wildcard Internet Egress For Critical Services

Hard rule.

---

# 159. Examples

Mail service may need:

```text
SMTP relay
DNS
```

---

# 160. Core mix node may need:

```text
directory
peers
```

---

# 161. Database service

No internet egress.

---

# 162. Egress DNS

Controlled resolver.

---

# 163. DNS Rebinding

Defend.

---

# 164. Resolve then enforce destination class.

---

# 165. No Domain Allowlist Without IP Revalidation

Hard rule.

---

# 166. Egress Proxy

Can enforce:

```text
destination
method
TLS policy
rate
```

---

# 167. External HTTP

Part 68 integration broker may use egress gateway.

---

# 168. Plugin Egress

Never direct.

---

# 169. Hard rule.

---

# 170. Credential Exfiltration Prevention

Egress gateway can block unknown destinations.

---

# 171. No Secret In URL Query

Preferred.

---

# 172. Authorization Header

Scoped.

---

# 173. TLS Pinning

Use cautiously.

---

# 174. Better

Trust roots + service identity.

---

# 175. Pin rotation complexity.

---

# 176. Egress Logging

Minimal.

---

# 177. Log:

```text
service class
destination class
result
```

---

# 178. Avoid:

```text
user identifier
full URL path/query
payload
```

---

# 179. Hard Rule

No user-content egress logs.

---

# 180. Data Loss Prevention

Traditional DLP unsuitable for E2EE content.

---

# 181. Do not decrypt to scan.

---

# 182. For admin/public APIs

Can scan structured metadata if policy permits.

---

# 183. Edge Authorization Layers

```text
network allow
→ transport auth
→ capability auth
→ application auth
```

---

# 184. Defense In Depth

Yes.

---

# 185. But

No duplicate inconsistent policy engines.

---

# 186. Canonical Policy Source

Part 61.

---

# 187. Edge receives compiled subset.

---

# 188. Edge Policy Bundle

```rust
pub struct EdgePolicyBundle {
    pub version: RuntimePolicyVersion,
    pub ingress: Vec<IngressRule>,
    pub egress: Vec<EgressRule>,
    pub signatures: PolicySignatureBundle,
}
```

---

# 189. Signed.

---

# 190. Anti-Rollback.

---

# 191. No Local Weakening

Hard rule.

---

# 192. Emergency Edge Policy

Can:

```text
block malicious IP range
disable compromised endpoint
reduce rate
```

---

# 193. Cannot:

```text
disable E2EE
route around anonymity policy
```

---

# 194. Hard rule.

---

# 195. Edge Deployment

Part 62.

---

# 196. Prefer stateless edge.

---

# 197. Why

Easy replacement.

---

# 198. Stateful Edge

Avoid except:

```text
rate counters
connection state
short-lived token cache
```

---

# 199. No Durable User Profile At Edge

Hard rule.

---

# 200. Edge Node Storage

Ephemeral/minimal.

---

# 201. Secrets

TLS keys, scoped credentials.

---

# 202. Secret Store/HSM

Part 61/72.

---

# 203. Edge Compromise

Should not reveal user content if E2EE.

---

# 204. Blast Radius

Scoped.

---

# 205. No Universal Edge Secret

Hard rule.

---

# 206. Edge Identity

Service-scoped.

---

# 207. Edge Attestation

Part 71 optional/required by node class.

---

# 208. Public Edge Host

Can be attested internally.

---

# 209. No exposing host attestation ID to users.

---

# 210. Reverse Proxy Backend Selection

Part 77.

---

# 211. Uses signed service discovery.

---

# 212. No Hardcoded Backend IPs In Code

Preferred.

---

# 213. Backend Health

Coarse.

---

# 214. Draining respected.

---

# 215. Failover

Only to eligible backend.

---

# 216. No Cross-Region Illegal Failover

Hard rule.

---

# 217. Connection Draining

Graceful.

---

# 218. QUIC Migration

Protocol-dependent.

---

# 219. Existing Sessions

May continue on draining endpoint.

---

# 220. New Sessions

Denied.

---

# 221. Edge Load Balancing

Random/weighted.

---

# 222. No account-sticky hash.

---

# 223. Hard rule.

---

# 224. Source IP Hashing

Avoid for anonymous user routing.

---

# 225. Why

Creates stable affinity and correlation.

---

# 226. Session-local affinity

Allowed.

---

# 227. Header Rewriting

Minimal.

---

# 228. Remove dangerous hop-by-hop headers.

---

# 229. Add only necessary internal context.

---

# 230. No Injected Global User Correlation Header

Hard rule.

---

# 231. Request ID

Scoped/random.

---

# 232. Not stable across sessions.

---

# 233. Trace Context

Infrastructure-scoped.

---

# 234. No user identity.

---

# 235. Proxy Protocol

Can pass source network info internally.

---

# 236. Use only where required.

---

# 237. Strict privacy mode

Avoid forwarding source IP deeper than edge if unnecessary.

---

# 238. Hard rule.

---

# 239. Source IP Retention

Short.

---

# 240. Edge may need for DDoS/rate limits.

---

# 241. But

Do not propagate to unrelated services.

---

# 242. Forwarded-For

Avoid broad propagation.

---

# 243. Prefer opaque connection class.

---

# 244. Geolocation

Not required.

---

# 245. Coarse region from network may be used locally if needed.

---

# 246. No precise geolocation collection.

---

# 247. Hard rule.

---

# 248. Privacy-Preserving Rate Limits

Future options:

```text
anonymous tokens
credits
proof systems
```

---

# 249. Initial

Coarse IP/service token limits acceptable with short retention.

---

# 250. Abuse Reporting

Part 46.

---

# 251. Edge can surface abuse counters.

---

# 252. No user content.

---

# 253. Egress Identity Separation

Different service outbound identities.

---

# 254. No Shared NAT Identity Required

Can share network egress but service auth remains separate.

---

# 255. High-risk services

Dedicated egress path.

---

# 256. Data Residency

Part 55.

---

# 257. Egress cannot send tenant data to forbidden jurisdiction/provider.

---

# 258. Hard rule.

---

# 259. Tenant Egress Policy

May be stricter.

---

# 260. User Stronger Privacy

Can be stricter still.

---

# 261. Control Plane Boundary

Most restrictive.

---

# 262. Requirements:

```text
private reachability
mTLS
MFA/operator auth
rate limit
typed admin API
```

---

# 263. No Browser-Exposed Raw Admin API Publicly

Preferred hard rule.

---

# 264. Admin UI

Behind private access boundary.

---

# 265. Bastion

Optional.

---

# 266. Better

Zero-trust private access proxy.

---

# 267. But

No dependence on one SaaS for emergency access.

---

# 268. Break-Glass

Part 61/70.

---

# 269. Works when primary edge unavailable.

---

# 270. Federation Boundary

Dedicated.

---

# 271. Federation gateway identity separate from public API edge.

---

# 272. No mixing user public API and federation admin channel if avoidable.

---

# 273. Hard rule.

---

# 274. Mailbox Edge

Can enforce:

```text
deposit size
fetch rate
token validity
```

---

# 275. Cannot inspect ciphertext content.

---

# 276. Relay Edge

Can enforce:

```text
session admission
bandwidth
packet rate
```

---

# 277. Cannot transcode E2EE media.

---

# 278. Mixnet Edge

Can enforce:

```text
cell framing
size
rate
```

---

# 279. Cannot inspect inner route/message.

---

# 280. Bridge Edge

Obfuscation layer.

---

# 281. Must not log client identity unnecessarily.

---

# 282. Bulk Edge

Can enforce:

```text
chunk size
quota
token
```

---

# 283. Object bytes encrypted.

---

# 284. Edge Failure Modes

```text
proxy crash
policy mismatch
WAF false positive
certificate expiry
egress block
backend discovery failure
```

---

# 285. Proxy Crash

Stateless replacement.

---

# 286. Policy Mismatch

Fail closed for security-sensitive rules.

---

# 287. WAF False Positive

Scoped bypass through signed policy update, not local ad-hoc disable.

---

# 288. Certificate Expiry

Automated rotation/alert.

---

# 289. Egress Block

Service fails safely.

---

# 290. Discovery Failure

Use cached signed endpoints.

---

# 291. No Direct Arbitrary Backend Fallback

Hard rule.

---

# 292. Edge HA

Multiple nodes.

---

# 293. Avoid single regional choke point.

---

# 294. Anycast or DNS load balancing optional.

---

# 295. Edge State

Minimal to make failover easy.

---

# 296. Rate Counter Replication

Can be approximate.

---

# 297. Security-critical revocation

Strongly consistent source.

---

# 298. No Need Consensus For Every Request

Hard rule.

---

# 299. Edge Config Distribution

Part 61.

---

# 300. Signed/pull model.

---

# 301. Config Hot Reload

Safe.

---

# 302. If invalid new policy

retain last valid.

---

# 303. No drop to permissive default.

---

# 304. Hard rule.

---

# 305. Edge Observability

Safe metrics:

```text
request rate
connection count
error class
rate-limit hits
backend health
egress denial count
```

---

# 306. Forbidden metrics

No:

```text
message content
contact IDs
conversation IDs
raw long-lived client IP history
```

---

# 307. Logging

Structured.

---

# 308. Request path

Can be normalized to route template.

---

# 309. No full query string.

---

# 310. Headers

Allowlist only.

---

# 311. Authorization header

Never log.

---

# 312. Cookies

Never log.

---

# 313. Payload

Never log by default.

---

# 314. Hard rule.

---

# 315. Sampling

Aggregate.

---

# 316. Support Bundle

Part 65.

---

# 317. Edge logs redacted.

---

# 318. Privacy Class

```rust
pub enum EdgeTelemetryClass {
    SafeAggregate,
    SensitiveAggregate,
    LocalOnly,
    Forbidden,
}
```

---

# 319. DDoS telemetry

Sensitive aggregate.

---

# 320. User-identifying request trace

Forbidden.

---

# 321. Edge SLOs

Examples:

```text
availability
connection success
request p99
backend failover
```

---

# 322. Security SLO

```text
0 unauthorized admin ingress
```

---

# 323. Privacy SLO

```text
0 plaintext E2EE payload inspection
0 user-correlation header propagation
```

---

# 324. Egress SLO

```text
0 unauthorized external destinations
```

---

# 325. Policy SLO

```text
100% edge nodes on signed current/allowed policy
```

---

# 326. Testing

Need edge testkit.

---

# 327. Test Scenarios

```text
request smuggling
oversized headers
slowloris
TLS downgrade
backend failover
egress violation
```

---

# 328. Request Smuggling Test

Ambiguous framing rejected.

---

# 329. Header Abuse Test

Bounds enforced.

---

# 330. Body Streaming Test

No unbounded buffering.

---

# 331. Slowloris Test

Idle timeout works.

---

# 332. TLS Test

Weak protocols/ciphers rejected per policy.

---

# 333. Certificate Rotation Test

No outage beyond target.

---

# 334. mTLS Test

Unknown peer rejected.

---

# 335. Federation Test

Wrong domain identity rejected.

---

# 336. Tenant Test

Wrong tenant context rejected.

---

# 337. Egress Allowlist Test

Unknown destination denied.

---

# 338. DNS Rebinding Test

Resolved destination revalidated.

---

# 339. Plugin Egress Test

No direct bypass.

---

# 340. WAF Test

Encrypted payload passes opaque.

---

# 341. Privacy Test

No user IDs in logs/headers.

---

# 342. Source IP Test

Not propagated beyond required edge scope.

---

# 343. Policy Rollback Test

Old permissive bundle rejected.

---

# 344. Failover Test

Only eligible backend selected.

---

# 345. Edge Compromise Test

Blast radius limited.

---

# 346. Fuzzing

Fuzz:

```text
HTTP parser adapters
binary ingress frames
edge policy
egress destination parser
```

---

# 347. Property Tests

Properties:

```text
forbidden egress destination is never allowed
admin ingress is never reachable from public boundary
privacy-sensitive protocol never routes through L7 inspection path
tenant context cannot be changed by untrusted header alone
```

---

# 348. Formal Verification Targets

Strong candidates:

```text
policy precedence
ingress/egress rule evaluation
tenant boundary routing
failover eligibility
```

---

# 349. Kani Candidate

rule evaluation/filter ordering.

---

# 350. TLA+ Candidate

edge failover + policy update + backend revocation.

---

# 351. Loom Candidate

concurrent hot reload + connection acceptance.

---

# 352. Performance

Edge is hot path.

---

# 353. Budget

```text
low added latency
bounded allocation
connection reuse
```

---

# 354. No Per-Request Database Lookup

Hard rule for common ingress.

---

# 355. Cache signed policy locally.

---

# 356. Auth Token Verification

Local where possible.

---

# 357. Remote introspection

Avoid hot path dependency.

---

# 358. Rate Counter

In-memory/local approximate if acceptable.

---

# 359. High-risk auth

Application confirms.

---

# 360. Zero-Copy

Use where safe for L4 forwarding.

---

# 361. But

Do not compromise validation.

---

# 362. io_uring

Optional optimization.

---

# 363. QUIC

First-class.

---

# 364. HTTP/2/3

For API gateway where supported.

---

# 365. HTTP/1.1

Support only if needed.

---

# 366. Request Smuggling Complexity

Lower if protocol stack simpler.

---

# 367. Rust Frameworks

Potential:

```text
hyper
axum
tower
quinn
rustls
```

---

# 368. Pure Rust Preference

Yes where mature.

---

# 369. Native Kernel/OS

Needed for:

```text
socket
firewall
eBPF/seccomp
```

---

# 370. Firewall

Default deny.

---

# 371. nftables/iptables adapter.

---

# 372. eBPF

Optional enforcement/observability.

---

# 373. No Mandatory eBPF.

---

# 374. Kernel-Level Egress

Defense in depth.

---

# 375. App-Level Egress Broker

Primary semantic policy.

---

# 376. Network Policy

Part 62 deployment.

---

# 377. Kubernetes NetworkPolicy

Optional adapter.

---

# 378. systemd/Podman/firewall

Equivalent policy.

---

# 379. No K8s dependency.

---

# 380. Crate Layout

Recommended:

```text
crates/
├── siar-edge-core/
├── siar-ingress/
├── siar-egress/
├── siar-api-gateway/
├── siar-reverse-proxy/
├── siar-waf/
├── siar-boundary-policy/
├── siar-federation-gateway/
├── siar-edge-observability/
└── siar-edge-testkit/
```

---

# 381. `siar-edge-core`

Owns:

```text
boundary classes
roles
errors
```

---

# 382. `siar-ingress`

L4/L7 admission.

---

# 383. `siar-egress`

Outbound destination/protocol enforcement.

---

# 384. `siar-api-gateway`

Auth/version/schema/rate-limits.

---

# 385. `siar-reverse-proxy`

Backend selection/connection management.

---

# 386. `siar-waf`

Explainable L7 security rules.

---

# 387. `siar-boundary-policy`

Signed policy compilation/evaluation.

---

# 388. `siar-federation-gateway`

Peer-domain boundary.

---

# 389. `siar-edge-observability`

Privacy-safe metrics.

---

# 390. `siar-edge-testkit`

Smuggling/DDoS/egress/failover/privacy tests.

---

# 391. Error Taxonomy

```rust
pub enum EdgeError {
    ProtocolDenied,
    RequestMalformed,
    RequestTooLarge,
    RateLimited,
    AuthenticationFailed,
    CapabilityDenied,
    BackendUnavailable,
    EgressDenied,
    PolicyInvalid,
    TenantScopeMismatch,
    Internal,
}
```

---

# 392. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Anonymous encrypted data-plane traffic is not forced through L7 content inspection.
2. Public, federation, tenant, and control-plane boundaries are distinct and cannot be silently merged.
3. Control-plane/admin ingress is never exposed through the same permissive public boundary as anonymous user traffic.
4. Egress is deny-by-default for critical services and restricted to explicit destinations/protocols.
5. API gateway checks never replace domain-layer authorization; both remain independently enforced.
6. Tenant context is explicit and cannot be selected solely by untrusted host/path/header input.
7. Edge/load-balancer routing never uses stable account-level affinity or global user correlation identifiers.
8. Edge logs exclude authorization secrets, payloads, conversation IDs, and long-lived user-IP histories.
9. Signed edge policy is anti-rollback protected and invalid updates never cause fallback to permissive defaults.
10. Failover only selects trust/privacy/legal-eligible backends discovered through Part 77.
11. WAF/security inspection applies only to plaintext protocol metadata it is entitled to inspect and never decrypts E2EE payloads.
12. Edge compromise is blast-radius-limited by scoped identities, scoped TLS keys, minimal durable state, and E2EE above the edge.
```

---

# 393. Initial Production Scope

Implement first:

```text
separate L4 anonymous ingress and L7 API ingress
rustls-based TLS/mTLS policies
Axum/Tower API gateway layer
QUIC ingress
request normalization
strict size/time bounds
rate limiting
tenant/federation context enforcement
deny-by-default egress gateway
backend discovery integration
signed anti-rollback edge policy
privacy-safe structured logs
stateless reverse proxy
edge testkit
```

Then add:

```text
anycast adapters
advanced anonymous rate tokens
multi-provider DDoS frontends
formal policy verification
confidential-compute edge options
advanced egress credential broker
```

---

# 394. Definition of Done

Part 78 is complete when:

- public, private, federation, control-plane, and egress boundaries are explicitly separated
- L4 and L7 paths are distinct
- anonymous/E2EE traffic can remain opaque through edge infrastructure
- TLS termination modes are service-specific
- API gateway and domain authorization remain separate
- request normalization and size/time bounds are enforced
- WAF only inspects plaintext protocol surfaces
- egress is deny-by-default
- plugin/integration outbound traffic cannot bypass the egress broker
- tenant/federation routing context is explicit
- backend selection uses Part 77 trusted discovery
- signed edge policies are anti-rollback protected
- privacy-safe edge logs/metrics are defined
- failover, request-smuggling, TLS, egress, tenant, fuzz, and formal tests are specified

---

# 395. Final Architecture

```text
                EXTERNAL NETWORK / FEDERATION
                           │
                           ▼
                    EDGE BOUNDARY LAYER
                           │
            ┌──────────────┼──────────────┐
            │              │              │
         L4 Ingress     L7/API Gate     Egress Gate
            │              │              │
            └──────────────┼──────────────┘
                           ▼
                 TRUSTED SERVICE BOUNDARY
                           │
                           ▼
                        SIAR CORE
```

Boundary-enforcement safety model:

```text
separate trust zones
+
minimal termination
+
strict normalization
+
deny-by-default egress
+
scoped auth/capabilities
+
privacy-safe observability
+
trusted backend discovery
```

not:

```text
route every packet through one giant proxy that can see and control everything
```

---

# 396. Final Principle

The edge should enforce **where traffic may go and what protocol rules it must obey**, without becoming the place where every identity, payload, or route is visible.

The correct model is:

```text
narrow ingress
+
opaque data-plane forwarding
+
strict L7 validation where justified
+
deny-by-default egress
+
scoped trust boundaries
```

This architecture gives SIAR a hardened ingress/egress and API boundary for public services, federation, control-plane access, tenants, integrations, relays, mailboxes, mixnet entry, and external dependencies while preserving the anonymity, least-authority, and local-first guarantees established across Parts 34–77.
