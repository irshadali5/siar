# Core System Architecture Part 80 — Anonymous Network Secrets Distribution, Dynamic Credentials, Certificate Authority, Workload Enrollment & Secretless Runtime Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 80  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 28, 52–53, 61–79  

**Primary purpose:** define SIAR's complete secrets-distribution, dynamic-credential, internal certificate-authority, workload-enrollment, bootstrap-trust, secret-broker, just-in-time access, rotation/revocation, and secretless-runtime architecture so long-lived credentials are minimized and workloads authenticate through short-lived, scoped, renewable authority.

---

# 1. Purpose

Secrets are necessary but dangerous.

Common failure modes include:

```text
API keys in config files
database passwords in environment variables
long-lived service certificates
shared credentials
secrets copied into containers
stale credentials after redeployment
manual rotation
```

The governing principle is:

> **SIAR should minimize long-lived secrets inside workloads, replace them with short-lived dynamic credentials, and make secret access explicit, scoped, renewable, revocable, and attributable.**

---

# 2. Architectural Position

```text
Bootstrap Trust
      │
      ▼
Workload Enrollment
      │
      ▼
Attestation / Authorization
      │
      ▼
Credential Issuer / Secret Broker
      │
      ▼
Short-Lived Credential
      │
      ▼
Workload Runtime
```

---

# 3. Core Separation

Keep distinct:

```text
root CA
intermediate CA
workload certificate
service credential
database credential
external API credential
user secret
recovery secret
tenant key
```

---

# 4. Non-Goals

Part 80 does not:

```text
store every secret in one global vault
allow services to fetch arbitrary secrets
put root CA keys online
treat environment variables as preferred secret transport
require static passwords where dynamic credentials are possible
```

---

# 5. Secret Classes

```rust
pub enum SecretClass {
    EphemeralSession,
    DynamicCredential,
    ServiceSecret,
    HighValueKey,
    RecoverySecret,
    ExternalProviderCredential,
}
```

---

# 6. Ephemeral Session

Lifetime:

```text
seconds/minutes
```

---

# 7. Dynamic Credential

Lifetime:

```text
minutes/hours
```

---

# 8. Service Secret

Longer-lived but scoped.

---

# 9. HighValueKey

Part 72 HSM custody.

---

# 10. Recovery Secret

Part 57/66.

---

# 11. External Provider Credential

Examples:

```text
SMTP relay token
object-store credential
external API key
```

---

# 12. Secret Lifetime Principle

Shortest practical lifetime.

---

# 13. Hard Rule

Long-lived credential must be explicitly justified.

---

# 14. Secret Descriptor

```rust
pub struct SecretDescriptor {
    pub secret_id: SecretId,
    pub class: SecretClass,
    pub owner: SecretOwner,
    pub scope: SecretScope,
    pub rotation: SecretRotationPolicy,
}
```

---

# 15. Secret Owner

```rust
pub enum SecretOwner {
    Service(ServiceIdentity),
    Tenant(TenantId),
    Federation(FederationDomainId),
    OperatorRole(OperatorRoleId),
}
```

---

# 16. No User Messaging Identity As Secret Owner

Hard rule for infrastructure credentials.

---

# 17. Secret Scope

```rust
pub enum SecretScope {
    SingleWorkload,
    Service,
    Tenant,
    Environment,
    Provider,
}
```

---

# 18. Narrowest Scope Preferred

---

# 19. Shared Secret

Avoid.

---

# 20. No One Secret Across Multiple Services

Hard rule.

---

# 21. No One Secret Across Environments

Hard rule.

---

# 22. Development vs Production

Separate trust roots/issuers.

---

# 23. No Staging Credential In Production

Hard rule.

---

# 24. Dynamic Credential Principle

Prefer:

```text
issue on demand
short TTL
audience-bound
auto-renew
auto-revoke
```

---

# 25. Dynamic Credential Types

```rust
pub enum DynamicCredentialType {
    MtlsCertificate,
    DatabaseCredential,
    ObjectStoreCredential,
    ApiToken,
    QueueCredential,
}
```

---

# 26. Workload Certificate

Part 79.

---

# 27. Database Credential

Short-lived user/password/token.

---

# 28. Object Store Credential

Scoped prefix/bucket/object permissions.

---

# 29. API Token

Audience-bound external/internal token.

---

# 30. Queue Credential

Scoped producer/consumer access.

---

# 31. Credential Request

```rust
pub struct CredentialRequest {
    pub workload: WorkloadIdentity,
    pub credential_type: DynamicCredentialType,
    pub audience: CredentialAudience,
    pub scope: CredentialScope,
    pub requested_ttl: Duration,
}
```

---

# 32. Credential Issuer

```rust
pub trait CredentialIssuer {
    fn issue(
        &self,
        request: CredentialRequest,
    ) -> Result<IssuedCredential, CredentialError>;
}
```

---

# 33. Issue Conditions

Require:

```text
workload identity
attestation
policy
allowed audience
allowed scope
```

---

# 34. No Self-Selected Arbitrary Scope

Hard rule.

---

# 35. TTL Bound

Issuer enforces max TTL.

---

# 36. No Client Override

Hard rule.

---

# 37. Renewal

Automatic.

---

# 38. Renewal Requires

```text
still authorized
still healthy
still attested if required
```

---

# 39. Renewal Failure

Credential expires naturally.

---

# 40. No Infinite Grace

Hard rule.

---

# 41. Workload Enrollment

Enrollment is how a new workload proves eligibility.

---

# 42. Enrollment Inputs

```text
deployment identity
artifact digest
config digest
host attestation
service assignment
```

---

# 43. Enrollment Request

```rust
pub struct WorkloadEnrollmentRequest {
    pub service: ServiceIdentity,
    pub instance: WorkloadInstanceId,
    pub artifact: BuildHash,
    pub config: ConfigDigest,
    pub attestation: Option<AttestationEvidence>,
}
```

---

# 44. Enrollment Decision

```rust
pub enum EnrollmentDecision {
    Approved,
    Denied,
    PendingReview,
}
```

---

# 45. No Auto-Enrollment From Network Location

Hard rule.

---

# 46. Bootstrap Identity

Needed before workload certificate exists.

---

# 47. Bootstrap Credential

Short-lived single-purpose token.

---

# 48. Bootstrap Token

```rust
pub struct BootstrapToken {
    pub token_id: BootstrapTokenId,
    pub expected_service: ServiceIdentity,
    pub expires_at: Timestamp,
}
```

---

# 49. Single Use

Preferred.

---

# 50. No Reusable Bootstrap Secret

Hard rule.

---

# 51. Bootstrap Token Delivery

Secure provisioning channel.

---

# 52. Examples

```text
cloud metadata secure channel
deployment agent
TPM-sealed token
offline provisioning
```

---

# 53. Bootstrap Token Scope

Only enrollment.

---

# 54. Cannot call business APIs.

---

# 55. Hard rule.

---

# 56. Enrollment Flow

```text
start workload
→ obtain bootstrap token
→ present attestation
→ validate artifact/config
→ issue workload certificate
→ destroy bootstrap token
```

---

# 57. Workload Certificate

Then used for normal identity.

---

# 58. Re-Enrollment

After rebuild/compromise.

---

# 59. Fresh credential.

---

# 60. No Reuse Of Old Workload Identity Credential

Hard rule.

---

# 61. Internal Certificate Authority

Need hierarchy.

---

# 62. CA Hierarchy

```text
offline root CA
→ environment intermediate
→ service/workload issuing CA
→ short-lived workload certificate
```

---

# 63. Root CA

Offline.

---

# 64. Root Key

HSM.

---

# 65. Intermediate CA

Online HSM-backed where practical.

---

# 66. Issuing CA

Highly available but scoped.

---

# 67. No Root CA Online

Hard rule.

---

# 68. CA Domains

Separate:

```text
production
staging
development
federation
```

---

# 69. Environment Separation

Mandatory.

---

# 70. CA Identifier

```rust
pub struct CaId(pub [u8; 32]);
```

---

# 71. Certificate Profile

```rust
pub struct CertificateProfile {
    pub purpose: CertificatePurpose,
    pub max_ttl: Duration,
    pub allowed_sans: SanPolicy,
}
```

---

# 72. Certificate Purpose

```rust
pub enum CertificatePurpose {
    WorkloadIdentity,
    FederationPeer,
    OperatorAccess,
    InternalServer,
}
```

---

# 73. Separate Profiles

Hard rule.

---

# 74. No Workload Certificate Used As Operator Certificate

Hard rule.

---

# 75. SAN Validation

Typed service identity.

---

# 76. No arbitrary free-form SAN.

---

# 77. Certificate TTL

Short.

---

# 78. Revocation

Short TTL + revocation list/status.

---

# 79. CRL/OCSP

Possible.

---

# 80. But

Avoid per-request online OCSP dependency.

---

# 81. Prefer local cached revocation state.

---

# 82. Signed Revocation Feed

Good.

---

# 83. Revocation Entry

```rust
pub struct CredentialRevocation {
    pub credential_id: CredentialId,
    pub effective_at: Timestamp,
    pub reason: RevocationReason,
}
```

---

# 84. Revocation Reasons

```rust
pub enum RevocationReason {
    Compromise,
    WorkloadRemoved,
    PolicyChange,
    EnvironmentChange,
    CredentialMisuse,
}
```

---

# 85. Revocation Overrides TTL

Hard rule.

---

# 86. CA Rotation

Planned.

---

# 87. Intermediate Rotation

```text
create new intermediate
→ trust old+new
→ issue new certs
→ migrate workloads
→ retire old
```

---

# 88. Root Rotation

Part 53/66/72.

---

# 89. Root Rollover

Ceremony-controlled.

---

# 90. No Automatic Root Rotation

Hard rule.

---

# 91. Certificate Pinning

Avoid leaf pinning.

---

# 92. Prefer trust-root + service identity verification.

---

# 93. Why

Rotation easier.

---

# 94. Secret Broker

Central or distributed service that issues/accesses secrets.

---

# 95. Secret Broker Responsibilities

```text
authenticate workload
authorize secret access
issue dynamic credential
audit secret use
```

---

# 96. Secret Broker Must Not

Return arbitrary secret by user-controlled string name.

---

# 97. Hard rule.

---

# 98. Typed Secret Request

```rust
pub struct SecretRequest {
    pub workload: WorkloadIdentity,
    pub secret_ref: SecretRef,
    pub intended_use: SecretUse,
}
```

---

# 99. Secret Ref

Opaque.

---

# 100. Secret Use

```rust
pub enum SecretUse {
    DatabaseConnect,
    ApiCall,
    ObjectStoreAccess,
    SigningRequest,
}
```

---

# 101. No Generic "ReadSecret"

Hard rule for app workloads.

---

# 102. Secret Broker Authorization

Policy-based.

---

# 103. Caller gets only permitted secret.

---

# 104. Prefer dynamic credential over static secret fetch.

---

# 105. Secretless Runtime

Goal:

```text
application does not hold long-lived secret
```

---

# 106. Patterns

```text
local credential agent
socket-based signing
database auth agent
object-store token broker
```

---

# 107. Local Agent

Runs beside workload.

---

# 108. Workload talks to agent via Unix socket.

---

# 109. Agent holds/renews credential.

---

# 110. Application sees short-lived token or authenticated channel.

---

# 111. Better Pattern

Agent opens connection on behalf of workload.

---

# 112. Even better for secret minimization.

---

# 113. Secret Agent Trait

```rust
pub trait SecretAgent {
    fn connect_database(
        &self,
        request: DatabaseAccessRequest,
    ) -> Result<SecureDatabaseChannel, SecretError>;
}
```

---

# 114. Secretless DB Runtime

Application never sees DB password.

---

# 115. Database Credential Broker

Flow:

```text
workload mTLS
→ request DB role
→ policy check
→ create short-lived DB credential
→ connect
→ credential expires
```

---

# 116. PostgreSQL

Can support dynamic roles/passwords/certs.

---

# 117. Application DB Role

Least privilege.

---

# 118. No App DB Superuser

Hard rule.

---

# 119. Object Store

Issue temporary scoped credential.

---

# 120. Prefix/object scope.

---

# 121. No global bucket credential.

---

# 122. Hard rule.

---

# 123. Queue Broker

Issue producer/consumer credential.

---

# 124. Topic scoped.

---

# 125. No wildcard queue credential.

---

# 126. External API Credentials

Harder.

---

# 127. Third-party provider may only offer static API key.

---

# 128. Broker Pattern

Keep static key in broker.

---

# 129. Workload sends approved operation to broker.

---

# 130. Broker calls external API.

---

# 131. Application never receives provider key.

---

# 132. Hard rule where feasible.

---

# 133. Integration Broker

Part 68/78.

---

# 134. Credential Isolation

Provider key scoped to integration.

---

# 135. No one key shared across tenants.

---

# 136. Multi-Tenant Secret Isolation

Part 69.

---

# 137. Tenant Secret Namespace

Separate.

---

# 138. Tenant Key

Never accessible to another tenant workload.

---

# 139. Hard rule.

---

# 140. Secret Ref

Includes tenant scope internally.

---

# 141. No cross-tenant secret lookup.

---

# 142. Cross-Tenant Integration

Explicit broker mapping.

---

# 143. Federation Secrets

Separate trust domain.

---

# 144. Federation peer certs are not internal workload certs.

---

# 145. Hard rule.

---

# 146. Workload Enrollment Across Federation

Not allowed by default.

---

# 147. Each domain enrolls its own workloads.

---

# 148. User Devices

Different model.

---

# 149. End-user device keys

Not issued by infrastructure CA by default.

---

# 150. Hard rule.

---

# 151. Anonymous identity must not depend on enterprise workload CA.

---

# 152. Managed Enterprise Device

May have org-scoped managed profile certificate.

---

# 153. Still separate from personal identity.

---

# 154. Secret Storage

Need protection.

---

# 155. Secret Store Options

```text
HSM-backed service
encrypted database
OS keyring
TPM-sealed local cache
```

---

# 156. High-Value Key

HSM.

---

# 157. External Provider Key

Encrypted secret store.

---

# 158. Dynamic Credential

Prefer not persist.

---

# 159. In-Memory Secret

Zeroize when done where practical.

---

# 160. Rust

Use secret wrappers.

---

# 161. Example

```rust
pub struct SecretBytes(Zeroizing<Vec<u8>>);
```

---

# 162. Avoid `Debug`

Hard rule.

---

# 163. Avoid `Clone`

Where possible.

---

# 164. No Serialize For Secret Type By Default

Hard rule.

---

# 165. Secret Type Traits

Explicit.

---

# 166. Redacted Debug

```text
SecretBytes([REDACTED])
```

---

# 167. Secret Logging

Forbidden.

---

# 168. Panic/Crash Dumps

Must not expose secrets.

---

# 169. Core Dumps

Restricted.

---

# 170. Swap

Sensitive process hardening.

---

# 171. Memory Locking

Optional.

---

# 172. Not absolute security.

---

# 173. Environment Variables

Discouraged.

---

# 174. Why

```text
process inspection
crash dumps
debug output
child inheritance
```

---

# 175. Hard Rule

High-value long-lived secret not delivered via environment variable.

---

# 176. Secret File

Can be acceptable with:

```text
tmpfs
strict permissions
short lifetime
```

---

# 177. File Descriptor Delivery

Better.

---

# 178. Unix Socket Agent

Preferred.

---

# 179. Kubernetes Secret

Not inherently secret enough.

---

# 180. If used

Treat as delivery mechanism, not root security.

---

# 181. Avoid base64-as-security misconception.

---

# 182. Container Image

Never contains secret.

---

# 183. Hard rule.

---

# 184. Build System

Part 73.

---

# 185. Build logs

No secrets.

---

# 186. CI

No production secret unless release operation strictly needs.

---

# 187. Release Signer

Part 72 isolated.

---

# 188. No HSM PIN In CI.

---

# 189. Bootstrap Secrets

Need lifecycle.

---

# 190. State

```rust
pub enum BootstrapSecretState {
    Issued,
    Used,
    Expired,
    Revoked,
}
```

---

# 191. Single Use

Recommended.

---

# 192. No Persistent Bootstrap Token Cache.

---

# 193. Workload Credential Cache

Short-lived local.

---

# 194. Agent caches within TTL.

---

# 195. Revocation push/poll.

---

# 196. Offline Operation

Important.

---

# 197. If credential issuer unavailable

existing short-lived credential may continue until expiry.

---

# 198. Renewal outage

service may eventually fail closed.

---

# 199. No permanent static fallback.

---

# 200. Hard rule.

---

# 201. Offline Grace

Only pre-defined.

---

# 202. Could allow slightly longer cert TTL in isolated environment.

---

# 203. Explicit assurance downgrade label.

---

# 204. No silent TTL extension.

---

# 205. Secret Broker HA

Critical.

---

# 206. But

HA does not imply one global broker.

---

# 207. Regional brokers.

---

# 208. Tenant brokers optional.

---

# 209. No global user-data secret broker.

---

# 210. Issuance Authority

Distributed by environment/region.

---

# 211. Root of trust

Common or partitioned.

---

# 212. Regional intermediate compromise

Scope limited.

---

# 213. Better than global online intermediate.

---

# 214. Policy

Region/environment specific.

---

# 215. Workload Enrollment Registry

Stores:

```text
service
artifact
attestation state
credential status
```

---

# 216. No user data.

---

# 217. Enrollment Record

```rust
pub struct EnrollmentRecord {
    pub workload: WorkloadIdentity,
    pub service: ServiceIdentity,
    pub artifact: BuildHash,
    pub credential_state: CredentialState,
}
```

---

# 218. Credential State

```rust
pub enum CredentialState {
    Pending,
    Active,
    Rotating,
    Revoked,
    Expired,
}
```

---

# 219. Workload Removal

Revoke/expire credentials.

---

# 220. No orphan credentials.

---

# 221. Credential Reconciliation

Periodic.

---

# 222. Detect:

```text
credential for nonexistent workload
expired issuer
unauthorized scope
```

---

# 223. Auto-revoke stale orphan.

---

# 224. High-risk orphan

Alert.

---

# 225. Secret Access Policy

```rust
pub struct SecretAccessPolicy {
    pub workload: ServiceIdentity,
    pub secret: SecretRef,
    pub allowed_uses: BTreeSet<SecretUse>,
    pub max_ttl: Duration,
}
```

---

# 226. Deny By Default

Hard rule.

---

# 227. No Wildcard Secret Access

Hard rule.

---

# 228. Secret Versioning

Needed for rotation.

---

# 229. Secret Version

```rust
pub struct SecretVersion(pub u64);
```

---

# 230. Secret Rotation

```text
create v2
→ distribute/use v2
→ overlap if required
→ revoke v1
```

---

# 231. Overlap Window

Bounded.

---

# 232. No indefinite dual-secret period.

---

# 233. Dynamic credential rotation

Continuous.

---

# 234. Static provider key rotation

Scheduled.

---

# 235. Emergency rotation

Immediate.

---

# 236. Rotation Trigger

```rust
pub enum RotationTrigger {
    Scheduled,
    Compromise,
    PolicyChange,
    ProviderRequirement,
    PersonnelChange,
}
```

---

# 237. Personnel Change

Relevant for operator credentials.

---

# 238. No employee departure leaving active credential.

---

# 239. Operator Access

Separate.

---

# 240. Operator credentials should be short-lived.

---

# 241. Human authentication

MFA/hardware key.

---

# 242. Just-In-Time Admin Access

Preferred.

---

# 243. Admin Session Credential

```rust
pub struct AdminSessionCredential {
    pub operator: OperatorIdentity,
    pub scope: AdminScope,
    pub expires_at: Timestamp,
}
```

---

# 244. No permanent admin token.

---

# 245. Hard rule.

---

# 246. Break-Glass

Part 61/70.

---

# 247. Emergency credential

Scoped.

---

# 248. Short-lived.

---

# 249. Multi-party approval for high-risk scope.

---

# 250. Break-glass credential use

Audited.

---

# 251. No global all-secrets break-glass token.

---

# 252. Hard rule.

---

# 253. Secretless Signing

Critical.

---

# 254. Application calls signer/HSM agent.

---

# 255. Private key never leaves HSM.

---

# 256. Part 72.

---

# 257. Secretless Decryption

Possible for infrastructure.

---

# 258. Agent can decrypt/wrap without exporting root key.

---

# 259. But

Do not centralize user E2EE decryption.

---

# 260. Hard rule.

---

# 261. Secretless TLS

Private key operations via HSM/agent possible.

---

# 262. Useful for high-value endpoints.

---

# 263. Performance tradeoff.

---

# 264. Session key leaves? Depends implementation.

---

# 265. Private signing key remains protected.

---

# 266. Certificate Issuance Policy

Machine-readable.

---

# 267. Example

```rust
pub struct IssuancePolicy {
    pub subject_service: ServiceIdentity,
    pub allowed_environment: EnvironmentId,
    pub required_attestation: AttestationAssurance,
    pub max_ttl: Duration,
}
```

---

# 268. Issuer cannot override policy ad hoc.

---

# 269. Signed policy.

---

# 270. Anti-rollback.

---

# 271. Enrollment Approval

Automated for expected deployments.

---

# 272. Manual for exceptional workloads.

---

# 273. No manual approval becomes unlimited wildcard.

---

# 274. Workload Attestation Binding

Credential should bind to:

```text
service
artifact
environment
```

---

# 275. Maybe host assurance.

---

# 276. Do not bind too tightly to ephemeral host details if it harms recovery.

---

# 277. Balance.

---

# 278. Certificate Metadata

Minimize.

---

# 279. No human operator names if not needed.

---

# 280. No user data.

---

# 281. CA Transparency

Internal.

---

# 282. Record issuance/revocation metadata.

---

# 283. No secret material.

---

# 284. Certificate Transparency Public Log

Not necessary for internal workload certs.

---

# 285. Could leak topology.

---

# 286. Hard rule

Do not publicly log internal service certificates by default.

---

# 287. Internal Issuance Audit

Restricted.

---

# 288. Secret Audit

Record:

```text
workload
secret ref
use
result
```

---

# 289. Do not record secret value.

---

# 290. Hard rule.

---

# 291. Secret Access Observability

Safe metrics:

```text
issuance rate
renewal failures
revocation count
secret-broker availability
```

---

# 292. No secret values.

---

# 293. No user IDs.

---

# 294. Secret Access Trace

Service-level only.

---

# 295. No user-to-secret relationship.

---

# 296. CA SLO

Examples:

```text
issuance availability
renewal latency
revocation propagation
```

---

# 297. Secret Broker SLO

```text
credential issuance success
rotation success
```

---

# 298. Security SLO

```text
0 expired credentials accepted
0 wildcard secret grants
0 root CA key online
```

---

# 299. Privacy SLO

```text
0 user identities embedded in infrastructure certs
```

---

# 300. Failure Modes

```text
issuer outage
broker outage
revocation feed outage
CA compromise
bootstrap token theft
policy rollback
```

---

# 301. Issuer Outage

Existing credentials live until expiry.

---

# 302. New workload enrollment unavailable.

---

# 303. No static fallback.

---

# 304. Broker Outage

Dynamic external access may degrade.

---

# 305. Core service can continue if existing credentials valid.

---

# 306. Revocation Feed Outage

Use cached signed state within grace.

---

# 307. Critical compromise

Fail closed for affected identity.

---

# 308. CA Compromise

Rotate intermediate/root depending scope.

---

# 309. Bootstrap Token Theft

Single-use + attestation limits damage.

---

# 310. Policy Rollback

Rejected.

---

# 311. Disaster Recovery

Part 70.

---

# 312. CA/secret broker DR must be tested.

---

# 313. Root recovery

Part 72.

---

# 314. Intermediate CA rebuild

From root ceremony.

---

# 315. Secret Store Backup

Encrypted.

---

# 316. No backup plaintext.

---

# 317. Dynamic credentials

No need to back up.

---

# 318. Reissue.

---

# 319. Static provider credentials

Backup only if necessary.

---

# 320. Prefer provider-side rotation.

---

# 321. Tenant Secrets Backup

Tenant-specific encryption.

---

# 322. No cross-tenant backup key.

---

# 323. Federation Secrets Backup

Domain-scoped.

---

# 324. Supply Chain Integration

Part 73.

---

# 325. Issuer/broker binaries pinned/attested.

---

# 326. Workload artifact

Must be trusted before enrollment.

---

# 327. Edge Integration

Part 78.

---

# 328. Edge TLS certs dynamic where possible.

---

# 329. Internal Service Integration

Part 79.

---

# 330. mTLS identities issued here.

---

# 331. Database Integration

Part 74.

---

# 332. Dynamic DB credentials.

---

# 333. Event Bus Integration

Part 75.

---

# 334. Dynamic queue credentials.

---

# 335. Consensus Integration

Part 76.

---

# 336. Consensus members get scoped node credentials.

---

# 337. No consensus credential = governance signing credential.

---

# 338. Hard rule.

---

# 339. Service Discovery Integration

Part 77.

---

# 340. Workload cert proves service identity after endpoint discovery.

---

# 341. No secret in discovery record.

---

# 342. Runtime Secret Handling

Rust wrappers.

---

# 343. Example

```rust
pub struct Secret<T>(Zeroizing<T>);
```

---

# 344. Avoid accidental formatting.

---

# 345. Implement:

```text
Debug => [REDACTED]
Display => forbidden
Serialize => not implemented
Clone => selective
```

---

# 346. Secret String

Avoid normal `String`.

---

# 347. Secret Bytes

Zeroizing buffer.

---

# 348. Memory lifetime

Small.

---

# 349. Secret File

Delete/unlink promptly.

---

# 350. Process Fork

Avoid inheritance.

---

# 351. Child Process

No inherited secret FDs unless explicit.

---

# 352. Hard rule.

---

# 353. Crash Report

Redaction.

---

# 354. Panic Hook

Must not dump secret wrapper internals.

---

# 355. Metrics Labels

Never secret-derived.

---

# 356. Hard rule.

---

# 357. Secret Scanner

CI/runtime config scan.

---

# 358. Detect:

```text
private key blocks
API token patterns
password assignments
```

---

# 359. False positives acceptable with review.

---

# 360. Git Secret Scan

Required.

---

# 361. No committed secret.

---

# 362. If secret committed

Treat as compromised and rotate.

---

# 363. Deleting Git history alone is insufficient.

---

# 364. Hard truth.

---

# 365. Secret Inventory

Machine-readable.

---

# 366. Inventory Includes

```text
secret class
owner
scope
rotation
issuer
```

---

# 367. Inventory Excludes Secret Value

Hard rule.

---

# 368. Secret Drift

Detect static secrets that should be dynamic.

---

# 369. Policy Gate

New long-lived secret requires review.

---

# 370. Secret Budget

Interesting governance control.

---

# 371. Goal

Reduce number of static production secrets over time.

---

# 372. Static Secret Count

Metric.

---

# 373. But

Not security proof.

---

# 374. No "zero secrets" marketing claim if provider API keys remain.

---

# 375. Secretless Means

Application workload avoids holding durable credentials.

---

# 376. Not

No cryptographic secret exists anywhere.

---

# 377. Hard truth.

---

# 378. Testing

Need secrets/PKI testkit.

---

# 379. Test Scenarios

```text
expired cert
revoked cert
issuer outage
bootstrap replay
wrong tenant
wrong environment
```

---

# 380. Bootstrap Replay Test

Second use rejected.

---

# 381. Attestation Test

Wrong artifact denied enrollment.

---

# 382. Environment Test

Staging token cannot get prod cert.

---

# 383. Tenant Test

Tenant A workload cannot fetch B secret.

---

# 384. Secret Ref Test

Opaque ref cannot be guessed across scope.

---

# 385. TTL Test

Credential beyond max TTL rejected.

---

# 386. Renewal Test

Unauthorized workload cannot renew.

---

# 387. Revocation Test

Revoked credential rejected before natural expiry.

---

# 388. CA Rotation Test

Old/new overlap works.

---

# 389. Root Rollover Test

Anti-rollback preserved.

---

# 390. Broker Outage Test

No static fallback.

---

# 391. Secretless DB Test

App never receives password.

---

# 392. External API Broker Test

Provider key never reaches app.

---

# 393. Secret Logging Test

No secret wrapper value emitted.

---

# 394. Core Dump Test

Sensitive processes hardened.

---

# 395. Fuzzing

Fuzz:

```text
certificate parser
credential request
secret policy
bootstrap token
```

---

# 396. Property Tests

Properties:

```text
credential scope never exceeds policy scope
revoked credential never becomes valid from stale cache
bootstrap token usable at most once
tenant secret cannot cross tenant boundary
```

---

# 397. Formal Verification Targets

Strong candidates:

```text
credential issuance policy
renewal/revocation state machine
bootstrap token lifecycle
secret scope lattice
```

---

# 398. Kani Candidate

scope/TTL subset checks.

---

# 399. TLA+ Candidate

issuer rotation + revocation + renewal race.

---

# 400. Loom Candidate

concurrent renewal/revocation.

---

# 401. Performance

Credential issuance not per request.

---

# 402. Cache short-lived credential locally.

---

# 403. mTLS connection reuse.

---

# 404. Dynamic DB credential

Pool may reuse connection within TTL.

---

# 405. Connection expires/drains before credential expiry if required.

---

# 406. Secret Agent

Keeps renewal off app hot path.

---

# 407. CA Throughput

Moderate.

---

# 408. Use regional issuers.

---

# 409. No root in hot path.

---

# 410. Broker Throughput

Depends on external integration rate.

---

# 411. Cache provider connection if safe.

---

# 412. But

Never expose provider secret.

---

# 413. Crate Layout

Recommended:

```text
crates/
├── siar-secret-core/
├── siar-secret-types/
├── siar-workload-enrollment/
├── siar-internal-ca/
├── siar-credential-issuer/
├── siar-secret-broker/
├── siar-secret-agent/
├── siar-secret-policy/
├── siar-secret-observability/
└── siar-secret-testkit/
```

---

# 414. `siar-secret-core`

Owns:

```text
secret classes
secret refs
errors
```

---

# 415. `siar-secret-types`

Non-serializable redacted wrappers.

---

# 416. `siar-workload-enrollment`

Bootstrap/attestation/enrollment.

---

# 417. `siar-internal-ca`

CA hierarchy/cert issuance/revocation.

---

# 418. `siar-credential-issuer`

Dynamic service/database/storage credentials.

---

# 419. `siar-secret-broker`

External provider/static secret isolation.

---

# 420. `siar-secret-agent`

Local workload secretless access.

---

# 421. `siar-secret-policy`

Access/TTL/scope rules.

---

# 422. `siar-secret-observability`

Privacy-safe issuance/rotation metrics.

---

# 423. `siar-secret-testkit`

Replay/revocation/rotation/failure simulation.

---

# 424. Error Taxonomy

```rust
pub enum SecretError {
    Unauthorized,
    SecretNotFound,
    SecretExpired,
    SecretRevoked,
    ScopeMismatch,
    TenantMismatch,
    EnvironmentMismatch,
    BrokerUnavailable,
    Internal,
}

pub enum CredentialError {
    EnrollmentDenied,
    AttestationFailed,
    IssuerUnavailable,
    TtlExceeded,
    AudienceDenied,
    Revoked,
    Internal,
}
```

---

# 425. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Root CA and RootAuthority keys are never online in ordinary workload hot paths.
2. Workloads receive short-lived, scoped credentials instead of long-lived shared credentials wherever feasible.
3. Bootstrap credentials are single-purpose, short-lived, preferably single-use, and cannot access business APIs.
4. Workload enrollment requires explicit service assignment and trusted artifact/config state; network location alone never grants identity.
5. Development, staging, production, tenant, and federation credential domains remain cryptographically isolated.
6. Static provider/API credentials are brokered where practical so application workloads never receive the durable secret.
7. High-value or durable secrets are never embedded in container images, source repositories, logs, metrics, or ordinary environment variables.
8. Secret access is deny-by-default, typed, purpose-specific, and never exposed through a generic wildcard "read any secret" API.
9. Revocation and policy changes override local caches and renewable credentials.
10. Secret/credential records and audits contain metadata only; secret values are never logged or placed in observability systems.
11. Tenant/federation secret namespaces remain isolated and cannot be crossed through guessed names, headers, or service identity alone.
12. Issuer/broker outage never causes silent fallback to weaker static credentials.
```

---

# 426. Initial Production Scope

Implement first:

```text
secret classification/registry
opaque SecretRef
non-serializable redacted Rust secret wrappers
single-use bootstrap enrollment tokens
artifact/config-aware workload enrollment
offline root + online environment intermediate CA
short-lived workload mTLS certificates
automatic renewal/revocation
dynamic PostgreSQL credentials
scoped object-store/queue credentials
external credential broker
local Unix-socket secret agent
deny-by-default secret policy
privacy-safe issuance metrics
secret/PKI testkit
```

Then add:

```text
TPM-sealed bootstrap credentials
hardware-backed issuing CA
secretless TLS/HSM integration
advanced dynamic cloud credentials
formal credential-lifecycle verification
multi-region issuer failover with constrained trust
```

---

# 427. Definition of Done

Part 80 is complete when:

- secret classes, owners, scopes, and rotation policies are explicit
- production workloads can enroll without permanent shared secrets
- bootstrap credentials are bounded/single-purpose
- internal CA hierarchy is separated by environment and purpose
- workload certificates are short-lived, renewable, and revocable
- dynamic DB/object-store/queue credentials exist
- external provider secrets can remain inside brokers
- applications can use secretless agent-mediated access
- secret wrappers prevent accidental logging/serialization
- tenant/federation/environment isolation is enforced
- no permissive static fallback exists during issuer/broker outage
- CA/root/intermediate rotation and disaster recovery are defined
- replay/revocation/rotation/tenant/fuzz/formal tests are specified

---

# 428. Final Architecture

```text
                       BOOTSTRAP TRUST
                              │
                              ▼
                    WORKLOAD ENROLLMENT
                              │
                              ▼
                    ATTESTATION / POLICY
                              │
                              ▼
                 CREDENTIAL ISSUER / BROKER
                              │
                     ┌────────┼────────┐
                     │        │        │
                   mTLS      DB     External API
                     │        │        │
                     └────────┼────────┘
                              ▼
                     SECRETLESS WORKLOAD
```

Secrets-distribution safety model:

```text
short-lived identity
+
dynamic credentials
+
offline roots
+
typed secret policy
+
local secret agent
+
brokered static secrets
+
automatic rotation/revocation
```

not:

```text
put passwords and API keys in environment variables and rotate them manually every few months
```

---

# 429. Final Principle

A mature secrets architecture does not merely protect secret values; it reduces how many long-lived secrets applications ever need to possess.

The correct model is:

```text
bootstrap minimally
+
enroll explicitly
+
issue dynamically
+
scope narrowly
+
renew automatically
+
revoke quickly
+
broker static secrets
+
keep roots offline
```

This architecture gives SIAR a secret-minimized runtime foundation for internal PKI, service identities, databases, queues, storage, external integrations, operators, tenants, federation, and control-plane workloads while preserving the anonymity, least-authority, and isolation guarantees established across Parts 34–79.
