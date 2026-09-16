# Core System Architecture Part 79 — Anonymous Network Internal Service-to-Service Communication, Zero-Trust Networking, mTLS, Service Identity & East-West Security Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 79  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 28, 51–53, 58, 61–78  

**Primary purpose:** define SIAR's east-west networking and service-to-service trust architecture, including workload identity, mTLS, zero-trust authorization, internal service discovery, credential issuance/rotation, segmentation, workload attestation, internal policy, tenant/federation isolation, observability, compromise containment, and service-mesh boundaries.

---

# 1. Purpose

Internal networks are not inherently trustworthy.

Threats include:

```text
compromised service
stolen service credential
lateral movement
misrouted internal request
overprivileged service account
shared secrets
flat network access
```

The governing principle is:

> **Every service-to-service interaction must authenticate the workload, authorize the specific action, and remain scoped by explicit policy even inside the same cluster, host, or private network.**

---

# 2. Architectural Position

```text
Service A
   │
   ▼
Workload Identity
   │
   ▼
mTLS Channel
   │
   ▼
East-West Authorization
   │
   ▼
Service B
```

---

# 3. Core Separation

Keep distinct:

```text
service identity
workload instance identity
host identity
tenant identity
user identity
federation identity
transport credential
application capability
```

---

# 4. Non-Goals

Part 79 does not create:

```text
a trusted flat LAN
shared API keys across services
implicit authorization because traffic is "internal"
mandatory sidecars for every deployment
```

---

# 5. Zero-Trust Principle

Never trust network location alone.

---

# 6. Internal Request Requirements

Each request must establish:

```text
who is calling
what service/action is requested
what scope applies
whether current policy allows it
```

---

# 7. Service Identity

```rust
pub struct ServiceIdentity(pub [u8; 32]);
```

---

# 8. Workload Identity

```rust
pub struct WorkloadIdentity {
    pub service: ServiceIdentity,
    pub instance: WorkloadInstanceId,
}
```

---

# 9. Workload Instance ID

Ephemeral/rotatable.

---

# 10. Host Identity

Part 71.

---

# 11. Hard Rule

WorkloadIdentity ≠ HostAttestationId.

---

# 12. Why

One host can run multiple workloads.

---

# 13. Service Naming

Logical service name.

---

# 14. Service Name Is Not Authorization

Hard rule.

---

# 15. Service Identity Credential

Short-lived X.509/SPIFFE-like certificate or equivalent.

---

# 16. Credential Lifetime

Hours or less where operationally practical.

---

# 17. No Long-Lived Shared Service Certificate

Hard rule.

---

# 18. mTLS

Default for authenticated east-west transport.

---

# 19. Mutual Authentication

Both sides authenticate.

---

# 20. TLS Library

`rustls` preferred where practical.

---

# 21. QUIC

Can carry equivalent workload identity.

---

# 22. HTTP/2 / HTTP/3

Allowed for RPC.

---

# 23. Raw Postcard/QUIC

Preferred for internal binary hot paths where appropriate.

---

# 24. JSON

External interoperability only unless unavoidable.

---

# 25. Workload Credential Issuer

Dedicated internal authority.

---

# 26. Not Same As Governance Root

Hard rule.

---

# 27. Not Same As Release Signer

Hard rule.

---

# 28. Not Same As User Identity Authority

Hard rule.

---

# 29. Service CA

Intermediate scoped authority.

---

# 30. Root

Offline/high-assurance per Part 72.

---

# 31. Issuance Chain

```text
offline trust root
→ workload intermediate
→ short-lived workload certificate
```

---

# 32. Service Credential Request

```rust
pub struct WorkloadCredentialRequest {
    pub service: ServiceIdentity,
    pub workload: WorkloadInstanceId,
    pub attestation: Option<WorkloadAttestation>,
}
```

---

# 33. Workload Attestation

Part 71 integration.

---

# 34. Credential Issuance Requires

```text
authorized deployment
expected artifact
allowed config
service membership
```

---

# 35. No Certificate From DNS Name Alone

Hard rule.

---

# 36. Workload Attestation Classes

```rust
pub enum WorkloadAttestation {
    SoftwareVerified,
    HostAttested,
    HardwareBacked,
}
```

---

# 37. Minimum Assurance

Service-class specific.

---

# 38. Tier0 Services

Prefer host-attested/hardware-backed.

---

# 39. Small Self-Hosted

Software-verified acceptable with explicit assurance label.

---

# 40. Certificate SAN

Contains workload/service identity.

---

# 41. Avoid

```text
user IDs
tenant user IDs
message IDs
```

---

# 42. Certificate Rotation

Automatic.

---

# 43. Rotation Overlap

Short.

---

# 44. No Credential Expiry Cliff Across Fleet

Hard rule.

---

# 45. Credential Renewal

Before expiry with jitter.

---

# 46. Jitter

Prevents thundering herd.

---

# 47. Credential Revocation

Needed.

---

# 48. Revocation Sources

```text
workload compromised
service removed
host quarantined
deployment revoked
```

---

# 49. Revocation Feed

Signed.

---

# 50. Revocation Overrides Cached Trust

Hard rule.

---

# 51. Short-Lived Credentials

Reduce CRL dependence.

---

# 52. Revocation Still Needed For High-Risk Incidents

Yes.

---

# 53. Internal Authorization

mTLS authenticates identity.

---

# 54. It Does Not Authorize Action

Hard rule.

---

# 55. Service Authorization Policy

```rust
pub struct ServiceAuthorizationPolicy {
    pub caller: ServiceIdentity,
    pub callee: ServiceIdentity,
    pub actions: BTreeSet<ServiceAction>,
    pub scope: ServiceScope,
}
```

---

# 56. Service Action

```rust
pub enum ServiceAction {
    Read,
    Write,
    Publish,
    Subscribe,
    Admin,
    Health,
}
```

---

# 57. Domain-Specific Actions

Preferred over generic verbs where practical.

---

# 58. Example

```text
MailboxDeposit
MailboxFetch
DirectoryRead
ConfigRead
```

---

# 59. Scope

```rust
pub enum ServiceScope {
    GlobalInfrastructure,
    Tenant(TenantId),
    Federation(FederationDomainId),
    Resource(ResourceScopeId),
}
```

---

# 60. No Global Scope By Default

Hard rule.

---

# 61. Least Privilege

Each service gets only required actions.

---

# 62. Default Deny

Hard rule.

---

# 63. No "Any Internal Service"

Hard rule.

---

# 64. Authorization Evaluation

At callee boundary.

---

# 65. Optionally At Caller Too

Defense in depth.

---

# 66. Hard Rule

Callee always enforces.

---

# 67. Policy Source

Part 61 signed runtime policy.

---

# 68. Policy Compilation

Produces service ACL/capability table.

---

# 69. No Manual Per-Host ACL Drift

Hard rule.

---

# 70. Service Policy Bundle

```rust
pub struct ServicePolicyBundle {
    pub version: RuntimePolicyVersion,
    pub grants: Vec<ServiceAuthorizationPolicy>,
    pub signatures: PolicySignatureBundle,
}
```

---

# 71. Anti-Rollback

Required.

---

# 72. No Local Weakening

Hard rule.

---

# 73. Identity Propagation

Dangerous.

---

# 74. Service A Should Not Forward Full User Identity To B Unless Required.

---

# 75. Principle

Pass only minimal authorization context.

---

# 76. Request Context

```rust
pub struct ServiceRequestContext {
    pub caller: ServiceIdentity,
    pub request_id: ScopedRequestId,
    pub tenant: Option<TenantId>,
    pub capability: Option<CapabilityRef>,
}
```

---

# 77. No Global User Correlation ID

Hard rule.

---

# 78. Request ID

Random, short-lived.

---

# 79. User Authorization Context

Opaque capability/projection.

---

# 80. No raw account object by default.

---

# 81. Confused Deputy Defense

Critical.

---

# 82. Service B Must know:

```text
caller identity
delegated capability
target scope
```

---

# 83. It Must Not Assume

```text
Service A was allowed, therefore B may do anything A asks
```

---

# 84. Hard rule.

---

# 85. Delegated Capability

Attenuated.

---

# 86. Example

```rust
pub struct DelegatedServiceCapability {
    pub issuer: ServiceIdentity,
    pub audience: ServiceIdentity,
    pub scope: ServiceScope,
    pub actions: BTreeSet<ServiceAction>,
    pub expires_at: Timestamp,
}
```

---

# 87. Audience-Bound

Mandatory.

---

# 88. No Bearer Token Usable By Any Service

Hard rule.

---

# 89. Token Lifetime

Short.

---

# 90. Replay Protection

Required where state-changing.

---

# 91. Nonce / Operation ID

Use stable idempotency key.

---

# 92. Internal RPC

Must be idempotent where retried.

---

# 93. Transport Retry

Cannot duplicate side effects.

---

# 94. Service-to-Service Protocol Classes

```rust
pub enum InternalProtocol {
    QuicPostcard,
    Http2,
    Http3,
    UnixSocket,
    NamedPipe,
}
```

---

# 95. Same-Host

Unix socket/IPC preferred where practical.

---

# 96. Cross-Host

mTLS/QUIC.

---

# 97. No TCP Cleartext On Internal Network

Hard rule.

---

# 98. Loopback Exception

Still authenticate process if security-sensitive.

---

# 99. Connection Reuse

Recommended.

---

# 100. Reduces handshake overhead.

---

# 101. But

Credential rotation must be respected.

---

# 102. Long-Lived Connection

Re-auth or reconnect before credential expiry.

---

# 103. No Indefinite Authenticated Connection

Hard rule.

---

# 104. Channel Binding

Useful.

---

# 105. Application capability can bind to TLS exporter/channel.

---

# 106. Optional.

---

# 107. East-West Network Segmentation

Defense in depth.

---

# 108. Network Zones

```text
edge
control
data
storage
admin
```

---

# 109. No Flat Subnet Trust

Hard rule.

---

# 110. Segment Rules

Only necessary service pairs communicate.

---

# 111. Example

```text
API gateway → application
application → DB
mix node ↛ admin service
```

---

# 112. Firewall

Host/network level.

---

# 113. Kubernetes NetworkPolicy

Optional.

---

# 114. Podman/systemd+nftables

Equivalent.

---

# 115. No K8s Dependency

Hard rule.

---

# 116. Segmentation Is Not Authorization

Hard rule.

---

# 117. Both required.

---

# 118. Service Discovery

Part 77.

---

# 119. Internal Resolver Returns

```text
service identity
eligible endpoints
trust metadata
```

---

# 120. Caller verifies service identity via mTLS.

---

# 121. DNS Name Alone Not Enough

Hard rule.

---

# 122. Internal DNS

Can locate service.

---

# 123. mTLS establishes cryptographic identity.

---

# 124. Discovery Cache

Signed/anti-rollback where needed.

---

# 125. Service Mesh

Optional.

---

# 126. Possible Models

```text
sidecar
node proxy
library
```

---

# 127. Preferred Baseline

Library-native identity + policy.

---

# 128. Why

Less metadata/complexity.

---

# 129. Sidecar Mesh

Optional for enterprise/large cluster.

---

# 130. Hard Rule

Core correctness/security must not depend on a proprietary mesh control plane.

---

# 131. Sidecar Risks

```text
extra metadata
central policy chokepoint
resource overhead
TLS double termination
```

---

# 132. No Mandatory Envoy

Hard rule.

---

# 133. Library-Level mTLS

Can be built with:

```text
rustls
quinn
tower
hyper
```

---

# 134. Service Identity Adapter

Pluggable.

---

# 135. SPIFFE-Like Semantics

Can inspire.

---

# 136. But

No external dependency required.

---

# 137. Service Identity URI

Optional internal format.

---

# 138. Example

```text
siar://service/mailbox
```

---

# 139. No user-scoped path.

---

# 140. Tenant-Aware Service Calls

Explicit TenantId.

---

# 141. Policy checks tenant.

---

# 142. No Caller-Supplied Tenant Header Trusted Alone

Hard rule.

---

# 143. Tenant Context

Bound to capability/token.

---

# 144. Cross-Tenant Service Call

Explicitly authorized.

---

# 145. No implicit cross-tenant internal RPC.

---

# 146. Federation East-West

Separate.

---

# 147. Federation peer is external trust boundary, not internal service.

---

# 148. Hard rule.

---

# 149. Federation Gateway

Part 58/78.

---

# 150. No Internal Service Certificate Accepted From Federation Peer

Hard rule.

---

# 151. Internal PKI

Independent per administrative domain.

---

# 152. Federation trust uses federation identity.

---

# 153. Multi-Region Internal Trust

Same service identity can have regional instances.

---

# 154. Instance credentials unique.

---

# 155. No Shared Private Key Across Regions

Hard rule.

---

# 156. Regional Identity

Can be attribute.

---

# 157. But

Service action policy remains service-level.

---

# 158. Region Restrictions

Part 55.

---

# 159. Service request can be blocked if data residency forbids route.

---

# 160. No Cross-Region Failover That Violates Policy

Hard rule.

---

# 161. Workload Lifecycle

```rust
pub enum WorkloadState {
    Starting,
    Attesting,
    Active,
    Draining,
    Quarantined,
    Terminated,
}
```

---

# 162. Active

Credential valid + policy loaded.

---

# 163. Draining

No new long-lived work.

---

# 164. Quarantined

No service calls.

---

# 165. Terminated

Credential revoked/expired.

---

# 166. Deployment Integration

Part 62.

---

# 167. Workload Start Flow

```text
start process
→ verify artifact/config
→ attest host/workload
→ issue credential
→ load policy
→ register service
→ accept traffic
```

---

# 168. No Service Registration Before Credential/Policy Ready

Hard rule.

---

# 169. Workload Stop Flow

```text
drain
→ deregister
→ revoke/expire credential
→ terminate
```

---

# 170. Compromise Flow

```text
quarantine
→ revoke workload credential
→ remove discovery endpoint
→ rotate dependent credentials if needed
→ rebuild
```

---

# 171. No Credential Reuse After Confirmed Compromise

Hard rule.

---

# 172. Host Compromise

Part 71.

---

# 173. All workloads on host may need quarantine.

---

# 174. Service Credential Theft

Scope to one workload/service.

---

# 175. Short lifetime limits damage.

---

# 176. CA Compromise

Severe.

---

# 177. Response

```text
freeze issuance
rotate intermediate
reissue workloads
revoke old chain
```

---

# 178. Root compromise

Part 72 catastrophic procedure.

---

# 179. Intermediate Separation

Per environment/region possible.

---

# 180. Example

```text
prod-east workload CA
prod-west workload CA
staging workload CA
```

---

# 181. No Staging CA Trusted In Production

Hard rule.

---

# 182. Environment Isolation

Critical.

---

# 183. Development credential

Cannot call production.

---

# 184. Hard rule.

---

# 185. Service Authorization Cache

Local.

---

# 186. Short-lived.

---

# 187. Revocation/policy update invalidates.

---

# 188. No indefinite cached allow.

---

# 189. Policy Failures

If policy service unavailable:

```text
use last valid signed policy within validity
```

---

# 190. No permissive fallback.

---

# 191. Hard rule.

---

# 192. Control Plane Calls

Most restricted.

---

# 193. Require:

```text
mTLS
service identity
scoped capability
explicit action policy
```

---

# 194. Database Access

Prefer service-specific DB credentials.

---

# 195. DB Credential ≠ service mTLS credential.

---

# 196. Separate secrets.

---

# 197. No Shared Database User Across Services

Hard rule.

---

# 198. Event Bus Access

Part 75.

---

# 199. Producer/consumer identity via service identity.

---

# 200. Topic scope policy.

---

# 201. No wildcard bus access.

---

# 202. Object Store Access

Scoped credentials.

---

# 203. No one service-wide storage super credential.

---

# 204. Egress

Part 78.

---

# 205. Internal service cannot make arbitrary external calls.

---

# 206. Egress broker/policy required.

---

# 207. Plugin service

Highly restricted.

---

# 208. No direct network if not needed.

---

# 209. Internal DNS

Scoped per environment.

---

# 210. No public DNS dependency for core east-west resolution.

---

# 211. Service Record

Can be signed.

---

# 212. Internal TTL

Short.

---

# 213. DNS cache

Bounded.

---

# 214. No security trust from cached address.

---

# 215. mTLS identity validates.

---

# 216. Internal Load Balancing

Part 77.

---

# 217. Random/weighted among eligible instances.

---

# 218. No user-level sticky hash.

---

# 219. Session affinity only if protocol needs.

---

# 220. East-West Traffic Steering

Obey:

```text
trust
region
tenant
capacity
health
```

---

# 221. Performance Never Overrides Authorization

Hard rule.

---

# 222. Service Health

Separate from identity.

---

# 223. Healthy but unauthorized

Cannot call.

---

# 224. Authorized but unhealthy

Not selected.

---

# 225. Circuit Breaker

Per downstream service.

---

# 226. Retry

Bounded/jittered.

---

# 227. Idempotency

Required.

---

# 228. No Retry Storm.

---

# 229. Timeout Budget

Per RPC.

---

# 230. Deadline Propagation

Useful.

---

# 231. But

No user-identifying trace context.

---

# 232. Request Budget

```rust
pub struct RpcBudget {
    pub timeout: Duration,
    pub max_retries: u8,
}
```

---

# 233. Deadline

Monotonic.

---

# 234. No Infinite RPC.

---

# 235. Backpressure

Bounded channels/connections.

---

# 236. Connection Pool

Per downstream service.

---

# 237. Pool Size bounded.

---

# 238. No one service can exhaust all sockets.

---

# 239. Bulkheads

Per dependency.

---

# 240. Service dependency failure contained.

---

# 241. No Cascading Failure

Goal.

---

# 242. Priority

Security-control calls get reserved capacity.

---

# 243. Bulk/background calls throttle first.

---

# 244. Internal Protocol Versioning

Explicit.

---

# 245. Capability negotiation

Part 7/52.

---

# 246. No silent downgrade.

---

# 247. Service Compatibility

Current/previous versions.

---

# 248. Rolling Upgrade

Mixed-version calls.

---

# 249. New feature

Activate only after support.

---

# 250. No breaking RPC overnight.

---

# 251. mTLS Certificate SAN Versioning

Avoid embedding protocol version.

---

# 252. Keep identity stable, capability separate.

---

# 253. Identity Is Not Capability

Hard rule.

---

# 254. Internal API Gateway?

Avoid central mandatory east-west gateway.

---

# 255. Why

Creates bottleneck/observer.

---

# 256. Prefer direct authenticated service calls.

---

# 257. Policy distributed.

---

# 258. Central control plane compiles policy, data plane enforces locally.

---

# 259. Hard Rule

No central authorization lookup on every east-west request.

---

# 260. Why

Availability/privacy/latency.

---

# 261. Authorization Cache

Signed policy.

---

# 262. Revocation push/pull.

---

# 263. Emergency Revocation

Fast.

---

# 264. Policy Version

Monotonic.

---

# 265. No rollback.

---

# 266. Service-to-Service Secrets

Minimize.

---

# 267. Prefer identity + capability over static API key.

---

# 268. Static shared secret

Forbidden for critical services.

---

# 269. Per-service scoped secret

Only where protocol requires.

---

# 270. Secret rotation automated.

---

# 271. Secret Store

Part 61.

---

# 272. No Secret In Env Vars For High-Value Long-Lived Credential

Preferred hard rule.

---

# 273. Secret Delivery

File descriptor/memory/agent.

---

# 274. No Logging Secrets.

---

# 275. East-West Observability

Safe metrics:

```text
request count
latency
error class
connection count
policy deny count
```

---

# 276. Forbidden

```text
message content
contact ID
conversation ID
raw auth token
user identity
```

---

# 277. Service Graph

Potentially sensitive.

---

# 278. Infrastructure service graph is acceptable internally.

---

# 279. But

Do not derive user social graph.

---

# 280. No per-user edge graph.

---

# 281. Distributed Tracing

Part 51.

---

# 282. Traditional global trace ID

Risky.

---

# 283. Use scoped trace/request IDs.

---

# 284. Cross-service trace

Short-lived/infrastructure-only.

---

# 285. No trace across anonymous route segments.

---

# 286. Hard rule.

---

# 287. Logging

Structured.

---

# 288. Include:

```text
caller service
callee service
route class
result
policy version
```

---

# 289. Avoid:

```text
user payload
stable user ID
full token
```

---

# 290. Privacy-Safe Correlation

Service-level only.

---

# 291. Audit

Policy changes/certificate issuance/revocation.

---

# 292. Not every internal request.

---

# 293. High-volume request audit would leak metadata.

---

# 294. Hard rule.

---

# 295. Security Monitoring

Detect:

```text
unexpected service pair
auth failures
policy-deny spikes
credential misuse
```

---

# 296. No Deep Packet Inspection.

---

# 297. Compromise Detection

Part 71.

---

# 298. Unexpected service call pattern

Signal.

---

# 299. But

Use service-level, not user-level patterns.

---

# 300. Quarantine

Service workload.

---

# 301. Remove from discovery.

---

# 302. Revoke credential.

---

# 303. Block east-west policy.

---

# 304. No whole-network shutdown unless systemic.

---

# 305. Service Dependency Graph

Machine-readable.

---

# 306. Descriptor

```rust
pub struct ServiceDependency {
    pub caller: ServiceIdentity,
    pub callee: ServiceIdentity,
    pub actions: BTreeSet<ServiceAction>,
    pub criticality: Criticality,
}
```

---

# 307. CI Policy

Every production dependency must be declared.

---

# 308. Undeclared service call

Denied.

---

# 309. Hard rule.

---

# 310. Dependency Drift

Detect.

---

# 311. New service edge requires review.

---

# 312. Security Review

Required for:

```text
admin access
cross-tenant
cross-region
high-value secret service
```

---

# 313. East-West Policy Compiler

```rust
pub trait EastWestPolicyCompiler {
    fn compile(
        &self,
        graph: &ServiceDependencyGraph,
        rules: &ServicePolicyRules,
    ) -> Result<CompiledEastWestPolicy, PolicyError>;
}
```

---

# 314. Local Policy Enforcer

```rust
pub trait ServiceAuthorizationEnforcer {
    fn authorize(
        &self,
        caller: &WorkloadIdentity,
        request: &ServiceRequestContext,
        action: ServiceAction,
    ) -> Result<(), AuthorizationError>;
}
```

---

# 315. Credential Provider

```rust
pub trait WorkloadCredentialProvider {
    fn issue(
        &self,
        request: WorkloadCredentialRequest,
    ) -> Result<WorkloadCredential, IdentityError>;
}
```

---

# 316. Service Dialer

```rust
pub trait SecureServiceDialer {
    fn connect(
        &self,
        target: ServiceIdentity,
        constraints: ConnectionConstraints,
    ) -> Result<SecureChannel, TransportError>;
}
```

---

# 317. No Generic Raw Socket Access In Domain Code

Preferred.

---

# 318. Why

Centralizes identity/policy checks.

---

# 319. Raw Socket

Allowed in transport/edge crates.

---

# 320. Internal RPC Client

Generated/typed where practical.

---

# 321. No stringly typed endpoints.

---

# 322. Service IDs typed.

---

# 323. Rust Type Safety

Strongly use newtypes.

---

# 324. Example

```rust
pub struct MailboxServiceId(ServiceIdentity);
pub struct DirectoryServiceId(ServiceIdentity);
```

---

# 325. Prevent accidental endpoint confusion.

---

# 326. AI-Generated Code Safety

Type system catches:

```text
wrong service
wrong tenant
wrong capability
```

where possible.

---

# 327. Testkit

Dedicated.

---

# 328. Test Scenarios

```text
wrong certificate
expired certificate
revoked workload
wrong tenant
undeclared service pair
```

---

# 329. mTLS Test

Unknown issuer rejected.

---

# 330. SAN Test

Wrong service identity rejected.

---

# 331. Expiry Test

Expired credential rejected.

---

# 332. Revocation Test

Cached connection loses/re-establishes trust correctly.

---

# 333. Policy Test

Unauthorized action rejected.

---

# 334. Confused Deputy Test

Delegated capability cannot expand scope.

---

# 335. Tenant Test

Tenant A capability cannot access Tenant B resource.

---

# 336. Federation Test

Federation identity not accepted as internal workload identity.

---

# 337. Environment Test

Staging credential cannot call production.

---

# 338. Discovery Test

Service IP change does not affect identity.

---

# 339. Compromise Test

Quarantined workload removed/revoked.

---

# 340. Rotation Test

Certificate rolls without fleet outage.

---

# 341. Policy Rollback Test

Old permissive policy rejected.

---

# 342. Network Segmentation Test

Forbidden pair blocked even if credential valid.

---

# 343. Mesh Bypass Test

Direct socket cannot bypass app-layer authorization.

---

# 344. Fuzzing

Fuzz:

```text
workload certificate parser
capability token
service policy
request context
```

---

# 345. Property Tests

Properties:

```text
valid mTLS identity alone never implies action authorization
revoked workload never becomes authorized from stale cache
tenant scope is invariant across delegated capability chain
undeclared service dependency is denied
```

---

# 346. Formal Verification Targets

Strong candidates:

```text
service authorization lattice
delegation attenuation
policy precedence
credential revocation
```

---

# 347. Kani Candidate

scope/action subset checks.

---

# 348. TLA+ Candidate

certificate rotation + revocation + connection reuse.

---

# 349. Loom Candidate

concurrent policy refresh + request authorization.

---

# 350. Performance

mTLS handshake cost amortized with connection reuse.

---

# 351. No Per-Request Certificate Issuance.

---

# 352. Policy check local/in-memory.

---

# 353. No central auth lookup hot path.

---

# 354. Connection Pool

Bounded.

---

# 355. QUIC 0-RTT

Use carefully.

---

# 356. State-changing requests

No unsafe replay.

---

# 357. 0-RTT disabled or limited for replay-sensitive RPC.

---

# 358. Hard rule.

---

# 359. Session Resumption

Scoped.

---

# 360. Resume only same service identity/trust domain.

---

# 361. No cross-service ticket reuse.

---

# 362. Rust Crate Layout

Recommended:

```text
crates/
├── siar-service-identity/
├── siar-workload-identity/
├── siar-internal-pki/
├── siar-mtls/
├── siar-east-west-policy/
├── siar-service-capability/
├── siar-secure-rpc/
├── siar-service-discovery-client/
├── siar-east-west-observability/
└── siar-east-west-testkit/
```

---

# 363. `siar-service-identity`

Owns service/workload IDs.

---

# 364. `siar-workload-identity`

Enrollment/attestation/lifecycle.

---

# 365. `siar-internal-pki`

Short-lived workload credentials.

---

# 366. `siar-mtls`

rustls/QUIC identity binding.

---

# 367. `siar-east-west-policy`

Authorization policy compilation/enforcement.

---

# 368. `siar-service-capability`

Delegated audience-bound capabilities.

---

# 369. `siar-secure-rpc`

Typed internal RPC/channel abstraction.

---

# 370. `siar-service-discovery-client`

Part 77 integration.

---

# 371. `siar-east-west-observability`

Privacy-safe service metrics.

---

# 372. `siar-east-west-testkit`

mTLS/policy/revocation/tenant simulation.

---

# 373. Error Taxonomy

```rust
pub enum IdentityError {
    AttestationFailed,
    CredentialDenied,
    CredentialExpired,
    CredentialRevoked,
    ServiceMismatch,
    EnvironmentMismatch,
    Internal,
}

pub enum AuthorizationError {
    CallerUnknown,
    ActionDenied,
    ScopeMismatch,
    TenantMismatch,
    CapabilityInvalid,
    PolicyExpired,
    Internal,
}

pub enum TransportError {
    DiscoveryFailed,
    TlsFailed,
    PeerIdentityMismatch,
    Timeout,
    ConnectionUnavailable,
    Internal,
}
```

---

# 374. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Internal network location never grants implicit trust; every service-to-service connection authenticates workload identity.
2. mTLS proves service/workload identity but never replaces application/domain authorization.
3. Workload credentials are short-lived, service-scoped, environment-scoped, and independently revocable.
4. Service, workload, host, tenant, federation, and user identities remain distinct.
5. Service policies are deny-by-default and least-privilege; wildcard east-west access is forbidden for critical services.
6. Tenant/federation scopes are explicit and cannot be inferred solely from untrusted headers or network topology.
7. Internal requests propagate only minimal authorization context and never require global user-correlation identifiers.
8. Delegated service capabilities are audience-bound, scope-limited, time-bounded, and attenuating.
9. Staging/development workload identities can never authenticate to production.
10. Revocation/quarantine overrides cached credentials, discovery, and policy state.
11. East-west observability is service-level and never becomes a user/contact/conversation graph.
12. Core security works without a mandatory proprietary service mesh; sidecars are optional implementation mechanisms, not trust roots.
```

---

# 375. Initial Production Scope

Implement first:

```text
typed ServiceIdentity/WorkloadIdentity
short-lived internal workload certificates
rustls-based mTLS
host/workload attestation integration
deny-by-default service authorization
service dependency graph
audience-bound delegated capabilities
tenant/federation/environment scoping
bounded connection pools
service discovery integration
credential rotation/revocation
quarantine integration
privacy-safe service metrics
east-west contract testkit
```

Then add:

```text
multi-region workload CA hierarchy
hardware-backed workload credential issuance
optional SPIFFE-compatible adapter
optional service-mesh adapter
formal authorization/delegation verification
advanced workload identity federation for tightly controlled enterprise domains
```

---

# 376. Definition of Done

Part 79 is complete when:

- every internal service/workload has cryptographic identity
- mTLS authenticates both sides
- application authorization remains independent
- workload credentials are short-lived and revocable
- host/workload/service/user/tenant/federation identities remain separate
- service dependency graph is explicit
- authorization is deny-by-default
- delegated capability is audience/scope/time bound
- tenant/federation/environment boundaries are enforced
- internal DNS/service discovery does not establish trust alone
- connection reuse respects credential expiry/revocation
- service mesh remains optional
- quarantine removes compromised workloads from discovery and authorization
- service-level observability excludes user metadata
- mTLS/revocation/confused-deputy/tenant/fuzz/formal tests are specified

---

# 377. Final Architecture

```text
                      SERVICE A
                         │
                         ▼
                 WORKLOAD IDENTITY
                         │
                         ▼
                      mTLS
                         │
                         ▼
              LOCAL AUTHORIZATION GATE
                         │
                         ▼
                      SERVICE B
                         │
                         ▼
                DOMAIN AUTHORIZATION
```

East-west security model:

```text
short-lived workload identity
+
mTLS
+
deny-by-default policy
+
audience-bound capability
+
network segmentation
+
local authorization
+
revocation/quarantine
```

not:

```text
it is inside the cluster, therefore it is trusted
```

---

# 378. Final Principle

Internal networks should be treated as hostile enough that compromise of one service does not grant lateral authority over the rest of the system.

The correct model is:

```text
authenticate every workload
+
authorize every action
+
minimize propagated identity
+
segment every dependency
+
rotate credentials continuously
+
contain compromise locally
```

This architecture gives SIAR a zero-trust east-west foundation for control-plane services, mailboxes, relays, directories, federation gateways, tenant services, databases, queues, storage systems, and deployment agents while preserving the anonymity and least-authority guarantees established across Parts 34–78.
