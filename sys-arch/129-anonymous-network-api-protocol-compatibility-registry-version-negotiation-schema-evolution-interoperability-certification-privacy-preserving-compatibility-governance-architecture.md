# Core System Architecture Part 129 — Anonymous Network API & Protocol Compatibility Registry, Version Negotiation, Schema Evolution, Interoperability Certification & Privacy-Preserving Compatibility Governance Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 129  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 52, 58, 59, 66, 68, 99, 108, 122–128

**Primary purpose:** define SIAR's compatibility-governance architecture for APIs, wire protocols, schema evolution, version negotiation, mixed-version interoperability, compatibility registries, downgrade resistance, client/server/device/federation compatibility, certification, deprecation, and privacy-preserving compatibility observability.

---

# 1. Purpose

Compatibility failures are dangerous because they often appear only during:

```text
mixed-version deployments
offline reconnect
partial rollouts
federation
multi-device state convergence
migration
rollback
```

A mature system must answer:

```text
Which versions can communicate safely?
Which schemas can coexist?
What happens when old clients reconnect?
How are protocol capabilities negotiated?
How are downgrade attacks prevented?
Which compatibility combinations are certified?
When may a version be retired?
```

The governing principle is:

> **SIAR compatibility should be explicit, authenticated, versioned, testable, certification-backed, and fail-safe under version mismatch without silently weakening security, privacy, durability, or anonymity guarantees.**

---

# 2. Architectural Position

```text
                 API / PROTOCOL DEFINITION
                           │
                           ▼
                  COMPATIBILITY REGISTRY
                           │
                ┌──────────┼──────────┐
                │          │          │
             VERSION     SCHEMA    CAPABILITY
                │          │          │
                └──────────┼──────────┘
                           ▼
                  VERSION NEGOTIATION
                           │
                           ▼
                 INTEROPERABILITY TEST
                           │
                           ▼
                     CERTIFICATION
                           │
                           ▼
                  RUNTIME ENFORCEMENT
```

---

# 3. Core Separation

Keep distinct:

```text
API version
protocol version
schema version
security epoch
feature capability
implementation release
compatibility promise
```

---

# 4. Non-Goals

Part 129 does not create:

```text
implicit compatibility
version negotiation based on user identity
silent downgrade paths
compatibility claims from semver alone
global compatibility telemetry tied to users
```

---

# 5. API Identity

```rust
pub struct ApiId(pub [u8; 16]);
```

---

# 6. Protocol Identity

```rust
pub struct ProtocolId(pub [u8; 16]);
```

---

# 7. Schema Identity

```rust
pub struct SchemaId(pub [u8; 16]);
```

---

# 8. Stable Identities

Names may change.

Canonical IDs do not.

---

# 9. Hard rule.

---

# 10. API Surface Class

```rust
pub enum ApiSurfaceClass {
    InternalRust,
    LocalIpc,
    ExternalHttp,
    ExternalSdk,
    PluginApi,
    FederationApi,
}
```

---

# 11. Protocol Class

```rust
pub enum ProtocolClass {
    Messaging,
    Sync,
    Media,
    Presence,
    AnonymousTransport,
    Mailbox,
    Federation,
    ControlPlane,
    Update,
}
```

---

# 12. Hard rule.

---

# 13. Version Types

```rust
pub struct ApiVersion(pub u32);
pub struct ProtocolVersion(pub u32);
pub struct SchemaVersion(pub u32);
pub struct SecurityEpoch(pub u64);
```

---

# 14. Do Not Conflate

Hard rule.

---

# 15. Semantic Versioning

Useful for human-facing package/API compatibility.

Not authoritative for runtime protocol safety.

---

# 16. Hard rule.

---

# 17. Protocol Versioning

Protocol versions are explicit integers/typed versions.

---

# 18. Hard rule.

---

# 19. Security Epoch

Monotonic safety floor.

```rust
pub struct ProtocolSecurityEpoch(pub u64);
```

---

# 20. Older epoch may become forbidden even if protocol version is otherwise compatible.

---

# 21. Hard rule.

---

# 22. Compatibility Registry

Canonical source of declared compatibility.

```rust
pub struct CompatibilityRegistry {
    pub registry_version: CompatibilityRegistryVersion,
    pub api_entries: BTreeMap<ApiId, ApiCompatibilityEntry>,
    pub protocol_entries: BTreeMap<ProtocolId, ProtocolCompatibilityEntry>,
    pub schema_entries: BTreeMap<SchemaId, SchemaCompatibilityEntry>,
}
```

---

# 23. Signed / Versioned

Hard rule.

---

# 24. Registry Provenance

Include:

```text
source revision
architecture baseline
effective date
signature
```

---

# 25. Hard rule.

---

# 26. API Compatibility Entry

```rust
pub struct ApiCompatibilityEntry {
    pub api: ApiId,
    pub supported_versions: VersionRange<ApiVersion>,
    pub deprecated_versions: BTreeSet<ApiVersion>,
    pub security_epoch: SecurityEpoch,
}
```

---

# 27. Protocol Compatibility Entry

```rust
pub struct ProtocolCompatibilityEntry {
    pub protocol: ProtocolId,
    pub supported_versions: VersionRange<ProtocolVersion>,
    pub minimum_security_epoch: ProtocolSecurityEpoch,
    pub negotiation_policy: NegotiationPolicyId,
}
```

---

# 28. Schema Compatibility Entry

```rust
pub struct SchemaCompatibilityEntry {
    pub schema: SchemaId,
    pub readable_versions: BTreeSet<SchemaVersion>,
    pub writable_versions: BTreeSet<SchemaVersion>,
}
```

---

# 29. Read Compatibility != Write Compatibility

Hard rule.

---

# 30. Compatibility Relation

```rust
pub enum CompatibilityRelation {
    Full,
    ReadOnly,
    WriteOnly,
    TransformRequired,
    Unsupported,
}
```

---

# 31. Hard rule.

---

# 32. Compatibility Matrix

```rust
pub struct CompatibilityMatrix<V> {
    pub pairs: BTreeMap<(V, V), CompatibilityRelation>,
}
```

---

# 33. No Assumption From Numeric Adjacency

Hard rule.

---

# 34. Version Negotiation

Negotiation must be authenticated where security-sensitive.

---

# 35. Negotiation Hello

```rust
pub struct ProtocolHello {
    pub protocol: ProtocolId,
    pub supported_versions: Vec<ProtocolVersion>,
    pub security_epoch: ProtocolSecurityEpoch,
    pub capabilities: BTreeSet<CapabilityCode>,
}
```

---

# 36. Bounds

Version/capability lists bounded.

---

# 37. Hard rule.

---

# 38. Negotiation Result

```rust
pub struct NegotiatedProtocol {
    pub protocol: ProtocolId,
    pub version: ProtocolVersion,
    pub security_epoch: ProtocolSecurityEpoch,
    pub capabilities: BTreeSet<CapabilityCode>,
}
```

---

# 39. Selection Rule

Highest mutually compatible version that satisfies:

```text
security floor
privacy floor
capability requirements
```

---

# 40. Hard rule.

---

# 41. No "Highest Number Wins" Blindly

Hard rule.

---

# 42. Downgrade Resistance

Peer must not force version below allowed floor.

---

# 43. Hard rule.

---

# 44. Downgrade Attempt

```rust
pub enum NegotiationFailure {
    NoCommonVersion,
    SecurityEpochTooLow,
    RequiredCapabilityMissing,
    RegistryMismatch,
    AuthenticationFailed,
}
```

---

# 45. Hard rule.

---

# 46. No Silent Fallback To Direct/Legacy Protocol

Hard rule.

---

# 47. Anonymous Mode

Maximum-anonymity path may have stricter minimum protocol/security versions.

---

# 48. Hard rule.

---

# 49. Negotiation Authentication

Version negotiation should be integrity-protected as early as practical.

---

# 50. Hard rule.

---

# 51. Capability Negotiation

Part 7.

Capabilities are independent from protocol version where useful.

---

# 52. Capability Code

```rust
pub struct CapabilityCode(pub u32);
```

---

# 53. Required / Optional Capabilities

```rust
pub struct CapabilityRequirement {
    pub required: BTreeSet<CapabilityCode>,
    pub optional: BTreeSet<CapabilityCode>,
}
```

---

# 54. Hard rule.

---

# 55. Capability Intersection

Use bounded deterministic set.

---

# 56. Hard rule.

---

# 57. Extension Capability

Unknown optional capability ignored if protocol permits.

---

# 58. Unknown required capability fails.

---

# 59. Hard rule.

---

# 60. Schema Evolution

Schema evolution must be designed, not assumed.

---

# 61. Schema Evolution Class

```rust
pub enum SchemaEvolutionClass {
    Additive,
    CompatibleRename,
    TransformRequired,
    Breaking,
}
```

---

# 62. Hard rule.

---

# 63. Postcard Schema Evolution

Postcard is compact but not inherently self-describing.

---

# 64. Therefore:

```text
explicit envelope version
stable field ordering strategy
new enum variants handled intentionally
migration adapters
```

---

# 65. Hard rule.

---

# 66. Versioned Postcard Envelope

```rust
pub struct WireEnvelope {
    pub protocol: ProtocolId,
    pub version: ProtocolVersion,
    pub message_type: MessageTypeCode,
    pub payload: Vec<u8>,
}
```

---

# 67. Payload Size Bounded

Hard rule.

---

# 68. Enum Evolution

Unknown enum variant cannot cause unsafe behavior.

---

# 69. Hard rule.

---

# 70. Strategy Options

```text
reserved variants
version-gated decode
extension envelope
```

---

# 71. Hard rule.

---

# 72. RON Configuration Evolution

RON human-owned config should include schema version.

---

# 73. Example

```ron
(
    schema_version: 4,
    ...
)
```

---

# 74. Hard rule.

---

# 75. Unknown RON Fields

Policy-defined:

```text
reject
warn
preserve
```

---

# 76. Never silently ignore security-critical configuration.

---

# 77. Hard rule.

---

# 78. Deprecated Config Keys

```rust
pub enum ConfigFieldLifecycle {
    Active,
    Deprecated,
    Alias,
    Removed,
}
```

---

# 79. Hard rule.

---

# 80. JSON External API Evolution

JSON only at external boundaries where necessary.

---

# 81. Explicit versioning.

---

# 82. Hard rule.

---

# 83. JSON Unknown Fields

Use contract-specific policy.

---

# 84. Hard rule.

---

# 85. API Evolution

Internal Rust APIs can evolve faster than external interfaces.

---

# 86. Hard rule.

---

# 87. External API Contract

```rust
pub struct ExternalApiContract {
    pub api: ApiId,
    pub version: ApiVersion,
    pub compatibility: BackwardCompatibilityPromise,
}
```

---

# 88. Hard rule.

---

# 89. API Version Negotiation

Possible via:

```text
path
header
content type
explicit handshake
```

---

# 90. Hard rule.

---

# 91. No User-Agent Guessing As Authority

Hard rule.

---

# 92. Local IPC Compatibility

Desktop app/daemon versions may differ during upgrade.

---

# 93. Require negotiated IPC version.

---

# 94. Hard rule.

---

# 95. Rolling Upgrade Compatibility

Need compatibility across N/N+1 versions during staged rollout.

---

# 96. Hard rule.

---

# 97. Server Compatibility Window

```rust
pub struct RollingUpgradeWindow {
    pub old: ProtocolVersion,
    pub new: ProtocolVersion,
    pub expires_at: Timestamp,
}
```

---

# 98. Time-Bounded

Hard rule.

---

# 99. Database Schema Compatibility

Part 74/128.

---

# 100. During rolling upgrade:

```text
old code reads new-compatible schema
new code reads old-compatible schema
```

where required.

---

# 101. Hard rule.

---

# 102. Expand-Migrate-Contract

Preferred.

---

# 103. Hard rule.

---

# 104. Multi-Device Compatibility

Devices on one account may have version skew.

---

# 105. Hard rule.

---

# 106. Multi-Device Negotiation

Capability per device.

---

# 107. No assumption all devices upgrade simultaneously.

---

# 108. Hard rule.

---

# 109. Message Fanout

Sender may need per-device compatible encoding.

---

# 110. Hard rule.

---

# 111. Feature Downgrade

If one target device lacks feature:

```text
explicit fallback representation
or
partial support
or
fail
```

---

# 112. No silent privacy/security weakening.

---

# 113. Hard rule.

---

# 114. Offline Device Reconnect

Old versions may reconnect after months.

---

# 115. Compatibility policy includes delayed reconnect.

---

# 116. Hard rule.

---

# 117. Reconnect Decision

```rust
pub enum ReconnectCompatibilityDecision {
    Compatible,
    CompatibleWithMigration,
    UpgradeRequired,
    Unsupported,
}
```

---

# 118. Hard rule.

---

# 119. Unsupported Reconnect

Preserve local data/export/recovery path.

---

# 120. Hard rule.

---

# 121. Federation Compatibility

Part 58.

---

# 122. Federation Peer Contract

```rust
pub struct FederationCompatibilityContract {
    pub local_domain: FederationDomainId,
    pub remote_domain: FederationDomainId,
    pub protocols: BTreeMap<ProtocolId, VersionRange<ProtocolVersion>>,
    pub expires_at: Option<Timestamp>,
}
```

---

# 123. Hard rule.

---

# 124. No Transitive Compatibility Assumption

Hard rule.

---

# 125. Federation Negotiation

Peer advertises signed supported versions/capabilities.

---

# 126. Hard rule.

---

# 127. Federation Downgrade

Reject below bilateral minimum.

---

# 128. Hard rule.

---

# 129. Interoperability Certification

Compatibility claim should be backed by test evidence.

---

# 130. Certification Scope

```rust
pub struct InteroperabilityCertificationScope {
    pub implementation_a: ImplementationVersionRef,
    pub implementation_b: ImplementationVersionRef,
    pub protocol: ProtocolId,
}
```

---

# 131. Hard rule.

---

# 132. Interoperability Certification

```rust
pub struct InteroperabilityCertification {
    pub certification_id: InteropCertificationId,
    pub scope: InteroperabilityCertificationScope,
    pub state: InteropCertificationState,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 133. Certification State

```rust
pub enum InteropCertificationState {
    Draft,
    Passed,
    Conditional,
    Failed,
    Superseded,
    Revoked,
}
```

---

# 134. Hard rule.

---

# 135. Interop Test Classes

```text
handshake
message exchange
error handling
upgrade
rollback
version skew
malformed input
capability mismatch
```

---

# 136. Hard rule.

---

# 137. Compatibility Test Matrix

```rust
pub struct InteropTestMatrix {
    pub implementations: Vec<ImplementationVersionRef>,
    pub protocols: Vec<ProtocolId>,
    pub cases: Vec<InteropTestCase>,
}
```

---

# 138. Hard rule.

---

# 139. Pairwise Testing

Minimum.

---

# 140. Critical protocols may require broader matrix.

---

# 141. Hard rule.

---

# 142. Mixed-Version Matrix

Examples:

```text
client N ↔ server N
client N-1 ↔ server N
client N ↔ server N-1
device N ↔ device N-2
federation domain N ↔ domain N-1
```

---

# 143. Hard rule.

---

# 144. Compatibility Certification Evidence

Bind:

```text
source revisions
artifact digests
schema versions
protocol versions
test environment
```

---

# 145. Hard rule.

---

# 146. Certification Archive

Part 126.

---

# 147. Hard rule.

---

# 148. Compatibility Revocation

A newly found flaw may invalidate certification.

---

# 149. Hard rule.

---

# 150. Revocation Reasons

```rust
pub enum InteropRevocationReason {
    SecurityIssue,
    ProtocolBug,
    SchemaBug,
    EvidenceInvalid,
    ImplementationRegression,
}
```

---

# 151. Hard rule.

---

# 152. Runtime Compatibility Decision

Uses signed/current registry.

---

# 153. Hard rule.

---

# 154. Stale Registry

Do not assume permissive compatibility.

---

# 155. Hard rule.

---

# 156. Registry Freshness

```rust
pub enum RegistryFreshness {
    Fresh,
    GracePeriod,
    Expired,
}
```

---

# 157. Expired Registry

Use safe cached floor or fail depending protocol.

---

# 158. Never permissive fallback.

---

# 159. Hard rule.

---

# 160. Registry Distribution

Part 61/99.

Signed, versioned.

---

# 161. Hard rule.

---

# 162. Anti-Rollback

Compatibility registry has monotonic policy epoch.

---

# 163. Hard rule.

---

# 164. Registry Policy Epoch

```rust
pub struct CompatibilityPolicyEpoch(pub u64);
```

---

# 165. Older epoch rejected if below local floor.

---

# 166. Hard rule.

---

# 167. Compatibility Exceptions

Temporary narrow exception possible for waivable non-hard surfaces.

---

# 168. Hard rule.

---

# 169. Compatibility Exception

```rust
pub struct CompatibilityException {
    pub scope: CompatibilityExceptionScope,
    pub reason: ExceptionRationale,
    pub expires_at: Timestamp,
}
```

---

# 170. Exception Scope

```rust
pub enum CompatibilityExceptionScope {
    Tenant(TenantId),
    FederationPeer(FederationDomainId),
    Deployment(DeploymentId),
}
```

---

# 171. No User Scope

Hard rule.

---

# 172. Security Floor Non-Waivable

Hard rule.

---

# 173. Exception Cannot Enable Known-Insecure Protocol

Hard rule.

---

# 174. Deprecation

Compatibility registry tracks version lifecycle.

---

# 175. Version Lifecycle

```rust
pub enum CompatibilityVersionState {
    Current,
    Supported,
    Deprecated,
    SunsetScheduled,
    Unsupported,
    Forbidden,
}
```

---

# 176. Forbidden

Security/privacy floor violation.

---

# 177. Hard rule.

---

# 178. Sunset Window

Part 128.

---

# 179. Hard rule.

---

# 180. Legacy Compatibility

Legacy adapters may exist.

---

# 181. Must be:

```text
isolated
scoped
time-bounded
audited
```

---

# 182. Hard rule.

---

# 183. Legacy Adapter

```rust
pub struct LegacyCompatibilityAdapter {
    pub old_version: ProtocolVersion,
    pub new_version: ProtocolVersion,
    pub expires_at: Timestamp,
}
```

---

# 184. Hard rule.

---

# 185. Adapter Security

Never translates away cryptographic guarantees.

---

# 186. Hard rule.

---

# 187. Adapter Data Loss

Explicit if unavoidable.

---

# 188. Hard rule.

---

# 189. Schema Transformer

```rust
pub trait SchemaTransformer<Old, New> {
    fn transform(&self, old: Old) -> Result<New, SchemaEvolutionError>;
}
```

---

# 190. Hard rule.

---

# 191. Transform Validation

Property tests + round-trip where possible.

---

# 192. Hard rule.

---

# 193. Lossy Transform

Must be explicit.

---

# 194. Hard rule.

---

# 195. Versioned Domain DTO

Internal domain model not tied directly to wire DTO.

---

# 196. Hard rule.

---

# 197. Boundary Adapter

Protocol-specific mapping layer.

---

# 198. Hard rule.

---

# 199. Avoid Version Checks Everywhere

Centralize negotiation/compatibility adapters.

---

# 200. Hard rule.

---

# 201. Feature Flags

Do not substitute for protocol version.

---

# 202. Hard rule.

---

# 203. Capability Negotiation + Feature Flags

Separate:

```text
peer can support feature
vs
local product chooses to expose feature
```

---

# 204. Hard rule.

---

# 205. Compatibility Observability

Safe dimensions:

```text
protocol
version pair
platform
region
failure class
```

---

# 206. No user identity.

---

# 207. Hard rule.

---

# 208. Compatibility Failure Metric

```rust
pub struct CompatibilityFailureMetric {
    pub protocol: ProtocolId,
    pub local_version: ProtocolVersion,
    pub peer_version: Option<ProtocolVersion>,
    pub class: NegotiationFailureClass,
}
```

---

# 209. Hard rule.

---

# 210. Coarse Aggregation

Hard rule.

---

# 211. No Peer Fingerprinting

Avoid storing persistent remote software fingerprint for anonymous peers.

---

# 212. Hard rule.

---

# 213. Anonymous Network Compatibility

Mixnet/messaging metadata sensitive.

---

# 214. Avoid stable compatibility identity across anonymous sessions.

---

# 215. Hard rule.

---

# 216. Version Fingerprinting Risk

Too much version detail can fingerprint clients.

---

# 217. Mitigation:

```text
coarse supported range
normalized capability sets
version cohorts
```

where protocol allows.

---

# 218. Hard rule.

---

# 219. Fingerprinting vs Compatibility

Balance explicitly.

---

# 220. Hard rule.

---

# 221. Privacy-Preserving Negotiation

Only disclose versions/capabilities necessary.

---

# 222. Hard rule.

---

# 223. Maximum Anonymity

May expose generic compatibility profile rather than detailed implementation version.

---

# 224. Hard rule.

---

# 225. Implementation Version Privacy

Separate:

```text
protocol version
implementation build version
```

Do not disclose build unnecessarily.

---

# 226. Hard rule.

---

# 227. Client Platform Disclosure

Avoid if not necessary.

---

# 228. Hard rule.

---

# 229. Compatibility Registry Access

Public parts:

```text
documented API/protocol versions
```

Restricted parts:

```text
security-forbidden versions
internal service matrix
```

as policy requires.

---

# 230. Hard rule.

---

# 231. Registry API

```rust
pub trait CompatibilityRegistryService {
    fn protocol(
        &self,
        id: ProtocolId,
    ) -> Result<ProtocolCompatibilityEntry, CompatibilityError>;

    fn api(
        &self,
        id: ApiId,
    ) -> Result<ApiCompatibilityEntry, CompatibilityError>;
}
```

---

# 232. Negotiation Service

```rust
pub trait ProtocolNegotiationService {
    fn negotiate(
        &self,
        local: &ProtocolHello,
        remote: &ProtocolHello,
    ) -> Result<NegotiatedProtocol, CompatibilityError>;
}
```

---

# 233. Schema Compatibility Service

```rust
pub trait SchemaCompatibilityService {
    fn relation(
        &self,
        schema: SchemaId,
        from: SchemaVersion,
        to: SchemaVersion,
    ) -> Result<CompatibilityRelation, CompatibilityError>;
}
```

---

# 234. Interop Certification Service

```rust
pub trait InteropCertificationService {
    fn certify(
        &self,
        scope: InteroperabilityCertificationScope,
        evidence: Vec<EvidenceRef>,
    ) -> Result<InteropCertificationId, CompatibilityError>;
}
```

---

# 235. No User-Analytics API

Hard rule.

---

# 236. Error Taxonomy

```rust
pub enum CompatibilityError {
    ApiUnknown,
    ProtocolUnknown,
    SchemaUnknown,
    NoCommonVersion,
    SecurityEpochTooLow,
    RequiredCapabilityMissing,
    RegistryExpired,
    PolicyEpochRollback,
    SchemaTransformUnavailable,
    CertificationMissing,
    CertificationRevoked,
    ExceptionExpired,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 237. Compatibility Governance

Review triggers:

```text
new protocol version
schema change
API break
security epoch increase
client support change
federation compatibility change
```

---

# 238. Hard rule.

---

# 239. Design Review Integration

Part 122.

Breaking/novel compatibility changes require review.

---

# 240. Hard rule.

---

# 241. Requirements Integration

Part 124.

Compatibility promises are requirements.

---

# 242. Hard rule.

---

# 243. Assurance Integration

Part 125.

Compatibility certification evidence satisfies verification obligations.

---

# 244. Hard rule.

---

# 245. Archive Integration

Part 126.

Store interop certifications/matrices.

---

# 246. Hard rule.

---

# 247. Product Readiness Integration

Part 127.

GA depends on compatibility readiness.

---

# 248. Hard rule.

---

# 249. Lifecycle Integration

Part 128.

Version sunset follows product lifecycle.

---

# 250. Hard rule.

---

# 251. Release Integration

Part 108.

Release gate checks required compatibility matrix.

---

# 252. Hard rule.

---

# 253. Update Integration

Part 99.

Minimum supported client can increase only per published policy.

---

# 254. Hard rule.

---

# 255. Risk Integration

Part 121.

Known compatibility gaps are tracked risks.

---

# 256. Hard rule.

---

# 257. PIR Integration

Part 120.

Compatibility incidents create regression tests/certification updates.

---

# 258. Hard rule.

---

# 259. Knowledge Graph Integration

Part 123.

Trace:

```text
API/protocol/schema version
→ ADR
→ implementation
→ tests
→ certification
→ release
```

---

# 260. Hard rule.

---

# 261. Compatibility Baseline

```rust
pub struct CompatibilityBaseline {
    pub baseline_id: CompatibilityBaselineId,
    pub registry_version: CompatibilityRegistryVersion,
    pub policy_epoch: CompatibilityPolicyEpoch,
    pub certifications: BTreeSet<InteropCertificationId>,
}
```

---

# 262. Immutable Per Release

Hard rule.

---

# 263. Historical Reconstruction

"What versions were certified for release X?"

Core requirement.

---

# 264. Hard rule.

---

# 265. Compatibility SLOs

Examples:

```text
supported client/server pairs pass interop suite
registry freshness within target
no forbidden security epoch negotiated
deprecated-version usage below migration threshold
```

---

# 266. Hard rule.

---

# 267. Security SLO

```text
0 negotiation below minimum security epoch
0 revoked interop certification accepted
0 silent downgrade to legacy protocol
```

---

# 268. Privacy SLO

```text
0 user identity in compatibility telemetry
0 unnecessary build-version disclosure in anonymous mode
```

---

# 269. Failure Modes

```text
registry stale
schema transformer missing
version skew untested
negotiation downgrade
old offline client reconnect
```

---

# 270. Stale Registry

Use restrictive cached state/fail-safe.

---

# 271. Hard rule.

---

# 272. Missing Transformer

Reject migration/communication if unsafe.

---

# 273. Hard rule.

---

# 274. Untested Version Skew

No certification.

---

# 275. Hard rule.

---

# 276. Downgrade Attempt

Reject/log aggregate security event.

---

# 277. Hard rule.

---

# 278. Offline Reconnect

Apply explicit compatibility/migration decision.

---

# 279. Hard rule.

---

# 280. Testing

Need compatibility testkit.

---

# 281. Test Scenarios

```text
client N-1 ↔ server N
client N ↔ server N-1
multi-device skew
federation skew
schema migration
security epoch increase
```

---

# 282. Negotiation Test

Select highest safe mutual version.

---

# 283. Hard rule.

---

# 284. Downgrade Test

Peer proposing low security epoch fails.

---

# 285. Hard rule.

---

# 286. Capability Test

Missing required capability fails negotiation.

---

# 287. Schema Test

Declared additive change decodes correctly.

---

# 288. Hard rule.

---

# 289. Unknown Enum Variant Test

No unsafe default.

---

# 290. Hard rule.

---

# 291. RON Config Test

Deprecated security-critical field cannot be silently ignored.

---

# 292. Hard rule.

---

# 293. JSON API Test

Unknown external field behavior matches contract.

---

# 294. Hard rule.

---

# 295. Rolling Upgrade Test

N/N+1 servers coexist during window.

---

# 296. Hard rule.

---

# 297. Multi-Device Test

Older device receives supported fallback or explicit unsupported state.

---

# 298. Hard rule.

---

# 299. Federation Test

Bilateral minimum enforced.

---

# 300. Hard rule.

---

# 301. Registry Anti-Rollback Test

Older policy epoch rejected.

---

# 302. Hard rule.

---

# 303. Certification Test

Uncertified pair cannot claim supported status.

---

# 304. Hard rule.

---

# 305. Privacy Test

Anonymous negotiation does not reveal unnecessary implementation build.

---

# 306. Hard rule.

---

# 307. Fuzzing

Fuzz:

```text
handshake
version sets
capability sets
schema envelopes
registry records
```

---

# 308. Property Tests

Properties:

```text
negotiation can never choose version below local security floor
unsupported required capability can never silently disappear
registry rollback can never lower policy epoch
uncertified version pair can never be represented as certified
```

---

# 309. Formal Verification Targets

Strong candidates:

```text
version negotiation
security epoch monotonicity
registry anti-rollback
schema compatibility relation
```

---

# 310. Kani Candidate

negotiation/compatibility-matrix invariants.

---

# 311. TLA+ Candidate

N→N+1 rolling upgrade with client/federation skew.

---

# 312. Loom Candidate

concurrent registry refresh + connection negotiation + policy epoch update.

---

# 313. Performance

Negotiation is connection-path work.

---

# 314. Keep:

```text
bounded sets
precomputed compatibility lookup
cached signed registry
```

---

# 315. Hard rule.

---

# 316. No Remote Registry Lookup In Hot Handshake

Hard rule.

---

# 317. Storage

Separate:

```text
API registry
protocol registry
schema registry
compatibility matrices
certifications
exceptions
policy epochs
```

---

# 318. No user-version history warehouse.

---

# 319. Hard rule.

---

# 320. Partitioning

By:

```text
API
protocol
schema
platform
release
federation domain
```

---

# 321. No person/user partition.

---

# 322. Hard rule.

---

# 323. Crate Layout

Recommended:

```text
crates/
├── siar-compat-core/
├── siar-api-registry/
├── siar-protocol-registry/
├── siar-schema-registry/
├── siar-version-negotiation/
├── siar-schema-evolution/
├── siar-interop-certification/
├── siar-compat-policy/
├── siar-compat-observability/
└── siar-compat-testkit/
```

---

# 324. `siar-compat-core`

Owns:

```text
ApiId
ProtocolId
SchemaId
CompatibilityRelation
CompatibilityError
```

---

# 325. `siar-api-registry`

External/internal API versions/contracts.

---

# 326. `siar-protocol-registry`

Wire protocol versions/security epochs/capabilities.

---

# 327. `siar-schema-registry`

Postcard/RON/JSON schema evolution metadata.

---

# 328. `siar-version-negotiation`

Authenticated bounded negotiation.

---

# 329. `siar-schema-evolution`

Typed schema transforms/migrations.

---

# 330. `siar-interop-certification`

Interop matrix/evidence/certification lifecycle.

---

# 331. `siar-compat-policy`

Version lifecycle/exceptions/anti-rollback.

---

# 332. `siar-compat-observability`

Aggregate compatibility health only.

---

# 333. `siar-compat-testkit`

mixed-version/fuzz/privacy/formal tests.

---

# 334. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. API versions, protocol versions, schema versions, implementation releases, capabilities, and security epochs are distinct typed concepts and can never be inferred from one another implicitly.
2. Runtime version negotiation selects only explicitly compatible versions that satisfy current security/privacy floors and can never silently fall back to legacy/direct/weaker behavior.
3. Compatibility claims are backed by signed registry entries and interoperability certification evidence; semver or numeric version proximity alone never proves runtime compatibility.
4. Postcard, RON, JSON, persistent-data, and API schema evolution use explicit versioning and conversion policy; unknown fields/variants never default into security-sensitive behavior.
5. Rolling upgrades, multi-device version skew, offline reconnect, old-client/new-server, new-client/old-server, and federation skew are first-class certified scenarios rather than exceptional edge cases.
6. Compatibility registries, policy epochs, and security epochs are signed, cached, bounded, freshness-aware, and anti-rollback protected; stale registry state never causes permissive compatibility.
7. Compatibility exceptions are narrow, scoped, time-bounded, and cannot lower non-waivable cryptographic, authorization, privacy, anonymity, residency, or protocol-security floors.
8. Interoperability certification is bound to exact implementation versions/artifact digests, protocol/schema versions, environments, and evidence and can be suspended/revoked without rewriting history.
9. Anonymous/private negotiation discloses only the minimum protocol/capability information necessary and avoids persistent implementation/version fingerprinting or stable peer compatibility profiles.
10. Compatibility telemetry is aggregated by protocol/version/platform/region/failure class and cannot become per-user version history, user profiling, or developer performance analytics.
11. Version deprecation, sunset, client minimums, legacy adapters, protocol retirement, and federation compatibility changes follow published lifecycle policy and preserve migration/data-recovery obligations.
12. Compatibility governance integrates with requirements, architecture governance, engineering knowledge, assurance, certification archives, product readiness/lifecycle, release/update control, risk/PIR, federation, multi-device, and data migration without creating an alternate route around platform trust.
```

---

# 335. Initial Production Scope

Implement first:

```text
typed ApiId/ProtocolId/SchemaId
typed API/protocol/schema/security versions
signed compatibility registry
protocol security epochs
authenticated bounded negotiation
capability negotiation
Postcard versioned envelope
RON schema/version policy
external JSON contract versioning
rolling-upgrade compatibility matrix
multi-device version-skew handling
offline reconnect compatibility
federation compatibility contracts
interop test matrix
interop certifications
registry anti-rollback/freshness
version lifecycle/deprecation
compatibility exceptions
release compatibility baselines
privacy-safe compatibility metrics
compatibility testkit
```

Then add:

```text
automatic protocol/schema diffing
semantic compatibility analysis
generated interop matrix planning
cross-federation certification exchange
formal negotiation verification
compatibility digital twins
portable signed compatibility bundles
```

---

# 336. Definition of Done

Part 129 is complete when:

- API/protocol/schema/security versions are distinct and typed;
- compatibility registry is signed/versioned;
- negotiation is authenticated and downgrade-resistant;
- capability negotiation is bounded;
- Postcard/RON/JSON evolution rules are explicit;
- rolling/mixed-version scenarios are certified;
- offline old-client reconnect is defined;
- federation compatibility is bilateral and explicit;
- interop evidence binds exact versions/artifacts;
- registry anti-rollback works;
- lifecycle/deprecation integrates with Part 128;
- anonymous negotiation minimizes fingerprinting;
- no user/developer compatibility profiling exists;
- mixed-version/privacy/fuzz/formal tests are specified.

---

# 337. Final Architecture

```text
              API / PROTOCOL / SCHEMA
                         │
                         ▼
                COMPATIBILITY REGISTRY
                         │
              ┌──────────┼──────────┐
              │          │          │
           VERSION     SCHEMA    CAPABILITY
              │          │          │
              └──────────┼──────────┘
                         ▼
                VERSION NEGOTIATION
                         │
                         ▼
               INTEROP CERTIFICATION
                         │
                         ▼
                 RELEASE BASELINE
                         │
                         ▼
                RUNTIME ENFORCEMENT
```

Compatibility-governance safety model:

```text
typed versions
+
signed registry
+
security epochs
+
authenticated negotiation
+
explicit schema evolution
+
mixed-version certification
+
anti-rollback
+
privacy-minimized telemetry
```

not:

```text
assume semver means interoperability, silently fall back to old protocols, ignore unknown security-critical fields, and track every user's client version forever
```

---

# 338. Final Principle

Compatibility is not an accident of two versions happening to work together; it is a governed, tested, and certified contract.

The correct model is:

```text
identify APIs/protocols/schemas explicitly
+
version every boundary
+
negotiate safely
+
enforce security floors
+
evolve schemas deliberately
+
certify mixed-version behavior
+
sunset versions predictably
+
minimize version fingerprinting
+
never turn compatibility telemetry into user surveillance
```

This architecture gives SIAR a privacy-preserving compatibility foundation for API evolution, protocol negotiation, schema migration, mixed-version operation, federation interoperability, certification, downgrade resistance, and version lifecycle governance while preserving the anonymity, local-first, least-authority, product-lifecycle, engineering-assurance, and anti-surveillance guarantees established across Parts 34–128.
