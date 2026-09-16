# Core System Architecture Part 52 — Anonymous Network Upgrade, Protocol Evolution, Compatibility, Migration & Zero-Downtime Rollout Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 52  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–51  

**Primary purpose:** define how SIAR evolves protocols, schemas, keys, providers, clients, and distributed anonymous infrastructure without breaking compatibility, weakening privacy, creating version-based fingerprinting, or requiring downtime.

---

# 1. Purpose

Anonymous infrastructure must evolve.

Over time SIAR will need to change:

```text
wire formats
capability schemas
Sphinx/provider packet profiles
directory/topology formats
mailbox records
group control messages
relay signaling
resource credits
provider catalogs
client databases
cryptographic algorithms
```

A naive upgrade can cause:

```text
network partitions
mixed-version incompatibility
privacy downgrade
downgrade attacks
service outages
client fingerprinting
state corruption
```

The governing principle is:

> **Protocol evolution must be explicit, monotonic, privacy-preserving, and operable across mixed-version networks without silently lowering security or anonymity guarantees.**

---

# 2. Architectural Position

```text
Protocol / Schema Change
        │
        ▼
Versioned Specification
        │
        ▼
Compatibility Matrix
        │
        ▼
Capability Negotiation
        │
        ▼
Canary / Rolling Upgrade
        │
        ▼
Mixed-Version Operation
        │
        ▼
Migration / Deprecation
```

---

# 3. Core Separation

Keep separate:

```text
protocol version
software version
schema version
capability version
crypto suite version
policy version
catalog version
topology epoch
```

---

# 4. Why Separation Matters

A single application version number is not enough.

Example:

```text
client v7
```

may support:

```text
framing v3
mailbox v2
relay v4
catalog v5
```

---

# 5. Version Types

```rust
pub struct ProtocolVersion(pub u16);
pub struct SchemaVersion(pub u16);
pub struct CapabilityVersion(pub u16);
pub struct CryptoSuiteVersion(pub u16);
pub struct PolicyVersion(pub u64);
pub struct SoftwareVersion(pub semver::Version);
```

---

# 6. Version Domains

Each subsystem evolves independently.

---

# 7. Protocol Domains

```rust
pub enum ProtocolDomain {
    AnonymousFrame,
    MixnetPacket,
    Mailbox,
    ReplyCapability,
    GroupControl,
    PrivateCall,
    ProviderCatalog,
    CapacityMarket,
    ResourceAccounting,
    Telemetry,
}
```

---

# 8. Version Registry

```rust
pub struct ProtocolVersionRegistry {
    pub supported: BTreeMap<ProtocolDomain, VersionRange>,
}
```

---

# 9. Version Range

```rust
pub struct VersionRange {
    pub minimum: u16,
    pub maximum: u16,
}
```

---

# 10. Supported Window

SIAR should generally support:

```text
current
previous
```

for mature protocols.

---

# 11. Longer Windows

Possible when migration cost is high.

---

# 12. Unknown Future Version

Reject safely.

---

# 13. Unknown Old Version

Reject once below minimum.

---

# 14. No Best-Effort Parsing

Hard rule.

---

# 15. Protocol Negotiation

Peers/providers negotiate mutually supported versions.

---

# 16. Negotiation Message

```rust
pub struct ProtocolOffer {
    pub domain: ProtocolDomain,
    pub supported: Vec<ProtocolVersion>,
    pub capabilities: CapabilitySet,
}
```

---

# 17. Negotiation Result

```rust
pub struct NegotiatedProtocol {
    pub domain: ProtocolDomain,
    pub version: ProtocolVersion,
    pub capabilities: NegotiatedCapabilities,
}
```

---

# 18. Negotiation Principle

Choose:

```text
highest mutually supported safe version
```

subject to policy.

---

# 19. Downgrade Attack

Attacker tries to force older protocol.

---

# 20. Downgrade Protection

Negotiation must be authenticated.

---

# 21. Transcript Binding

Selected version/capabilities included in authenticated transcript.

---

# 22. Minimum Security Floor

```rust
pub struct SecurityVersionFloor {
    pub domain: ProtocolDomain,
    pub minimum: ProtocolVersion,
}
```

---

# 23. Maximum Anonymity

May require stricter minimum than Standard mode.

---

# 24. Example

```text
Standard:
    framing >= v2

Maximum Anonymity:
    framing >= v3
```

---

# 25. No Silent Legacy Mode

Hard rule.

---

# 26. Compatibility Matrix

Every release must publish machine-readable compatibility.

---

# 27. Matrix Entry

```rust
pub struct CompatibilityEntry {
    pub domain: ProtocolDomain,
    pub local_version: ProtocolVersion,
    pub remote_version: ProtocolVersion,
    pub status: CompatibilityStatus,
}
```

---

# 28. Compatibility Status

```rust
pub enum CompatibilityStatus {
    Full,
    DegradedSafe,
    Unsupported,
}
```

---

# 29. DegradedSafe

Feature reduction allowed.

Privacy/security reduction not allowed.

---

# 30. Example

Old peer lacks:

```text
attachment resume
```

Safe result:

```text
disable resume
```

---

# 31. Unsafe Degradation

Old peer lacks:

```text
required padding
```

Maximum Anonymity:

```text
reject session
```

---

# 32. Software Rollout Phases

Recommended:

```text
Development
Internal
Canary
Staged
Broad
Enforced
Deprecated
Removed
```

---

# 33. Rollout State

```rust
pub enum RolloutPhase {
    Development,
    Internal,
    Canary,
    Staged,
    Broad,
    Enforced,
    Deprecated,
    Removed,
}
```

---

# 34. Canary Principle

Canary infrastructure, not identifiable user cohorts.

---

# 35. Why

Stable user targeting harms privacy.

---

# 36. Infrastructure Canary

Deploy to:

```text
random small provider/node subset
```

---

# 37. Client Canary

If client canary exists:

```text
explicit opt-in
random ephemeral cohort
```

not persistent user ID.

---

# 38. Cohort ID

Never stable across releases.

---

# 39. Zero-Downtime Requirement

Upgrade should maintain service.

---

# 40. Core Pattern

```text
expand
→ dual-read/dual-write if needed
→ migrate
→ contract
```

---

# 41. Expand Phase

New code understands:

```text
old + new
```

---

# 42. Migrate Phase

State gradually transformed.

---

# 43. Contract Phase

Old path removed after safety window.

---

# 44. No "flag day"

Avoid network-wide simultaneous version switch.

---

# 45. Mixed-Version Operation

Must be first-class.

---

# 46. Mix Node Upgrade

Nodes may run different software versions.

---

# 47. Packet Profile

Route must choose format supported across path.

---

# 48. Route Capability Intersection

```rust
pub struct RouteCapabilityIntersection {
    pub protocol_version: ProtocolVersion,
    pub features: CapabilitySet,
}
```

---

# 49. Path Selection

Strict mode rejects path lacking required feature.

---

# 50. Gateway Upgrade

Gateways support current + previous protocol during rollout.

---

# 51. Mailbox Upgrade

Mailbox API/schema must support old/new clients during migration.

---

# 52. Realtime Relay Upgrade

Existing calls remain pinned to old runtime until session end where possible.

---

# 53. No Mid-Call Breaking Upgrade

Hard rule.

---

# 54. Bulk Provider Upgrade

Active uploads continue using negotiated version.

---

# 55. Directory Upgrade

Directory publication format versioned.

---

# 56. Client Must Validate Format Version

Hard rule.

---

# 57. Topology Format Migration

Publish:

```text
old snapshot
new snapshot
```

during overlap if needed.

---

# 58. Dual Publication Window

Bounded.

---

# 59. No Ambiguous Authoritative Format

Hard rule.

---

# 60. Catalog Upgrade

Provider catalog schema versioned.

---

# 61. Capacity Market Upgrade

Likewise.

---

# 62. Resource Credit Upgrade

Credit versions particularly sensitive.

---

# 63. Credit Migration

Old valid credits must:

```text
remain spendable until expiry
or
be exchangeable
```

---

# 64. No Silent Credit Invalidation

Hard rule.

---

# 65. Crypto Migration

Highest-risk class.

---

# 66. Crypto Suite Registry

```rust
pub struct CryptoSuiteId(pub u16);
```

---

# 67. Crypto Migration Phases

```text
introduce new suite
dual support
prefer new
enforce new
retire old
```

---

# 68. Algorithm Agility

Protocol should negotiate suite explicitly.

---

# 69. No Unauthenticated Crypto Negotiation

Hard rule.

---

# 70. Deprecating Crypto

Requires:

```text
minimum version policy
signed policy update
release window
```

---

# 71. Emergency Crypto Disable

Possible if vulnerability discovered.

---

# 72. Emergency Policy

```rust
pub struct EmergencyProtocolPolicy {
    pub blocked_versions: Vec<ProtocolVersion>,
    pub blocked_suites: Vec<CryptoSuiteId>,
    pub expires_at: Timestamp,
    pub signature: GovernanceSignature,
}
```

---

# 73. Emergency Disable

Can reduce availability.

Must not reduce security.

---

# 74. Client Behavior

If only blocked protocol available:

```text
stop
```

---

# 75. Schema Migration

Client and server databases evolve independently.

---

# 76. Schema Version Table

Every persistent store records version.

---

# 77. Migration Rule

Migrations must be:

```text
explicit
ordered
idempotent where possible
crash-safe
```

---

# 78. Client Local DB Migration

Must preserve:

```text
messages
contact graph
outbox
keys
policies
```

---

# 79. Backup Before Destructive Migration

Recommended.

---

# 80. Backup Privacy

Use Part 33 rules.

---

# 81. Transactional Migration

Prefer transaction.

---

# 82. Large Migration

Use resumable phases.

---

# 83. Migration State

```rust
pub enum MigrationState {
    NotStarted,
    Running,
    Paused,
    Completed,
    Failed,
}
```

---

# 84. Migration Journal

```rust
pub struct MigrationJournal {
    pub migration_id: MigrationId,
    pub from: SchemaVersion,
    pub to: SchemaVersion,
    pub state: MigrationState,
    pub checkpoint: Option<MigrationCheckpoint>,
}
```

---

# 85. Crash Recovery

Resume from checkpoint.

---

# 86. No Partial Semantic State

Hard rule.

---

# 87. Server DB Migration

Use expand/contract.

---

# 88. Example

```text
add new column/table
write both
backfill
read new
stop old writes
remove old later
```

---

# 89. Dual Write Caution

Can create divergence.

---

# 90. Write Authority

One canonical state transition.

---

# 91. Shadow Write

For validation only where safe.

---

# 92. Migration Verification

Compare:

```text
counts
checksums
invariants
```

not user content in telemetry.

---

# 93. Key Schema Migration

Secrets need extra care.

---

# 94. Secret Ref Migration

Move references atomically.

---

# 95. Never Copy Secrets Into Logs

Hard rule.

---

# 96. Key Rotation During Upgrade

May be coupled to migration.

---

# 97. Avoid Unnecessary Coupling

Protocol upgrade should not always rotate identity keys.

---

# 98. Mailbox Secret Migration

Prefer:

```text
new mailbox epoch
dual receive
retire old
```

---

# 99. Reply Capability Migration

Old capabilities expire naturally.

---

# 100. Group Epoch Migration

Do not rewrite old group history.

---

# 101. Group Protocol Upgrade

Advance group control epoch.

---

# 102. Mixed Group Clients

Need capability intersection.

---

# 103. Strict Group Mode

If member cannot meet minimum privacy version:

```text
member upgrade required
```

---

# 104. Anonymous Call Upgrade

Existing call continues old safe negotiated version.

---

# 105. New Calls

Use new preferred version.

---

# 106. Provider Capability Upgrade

Descriptor advertises new feature only after runtime ready.

---

# 107. Two-Phase Capability Advertisement

```text
deploy capability
verify
advertise
```

---

# 108. Removing Capability

Reverse:

```text
stop selecting
wait
remove runtime
```

---

# 109. No Advertise-Before-Ready

Hard rule.

---

# 110. Feature Flags

Useful for rollout.

---

# 111. Privacy-Safe Feature Flags

Keyed by:

```text
infrastructure
release channel
random ephemeral cohort
```

not stable user identity.

---

# 112. Flag Types

```rust
pub enum FeatureFlagScope {
    Node,
    Provider,
    Region,
    EphemeralClientCohort,
}
```

---

# 113. No Contact-Based Flagging

Hard rule.

---

# 114. Flag Expiry

All temporary rollout flags expire.

---

# 115. Kill Switch

Security-sensitive feature can be disabled.

---

# 116. Kill Switch Policy

Signed.

---

# 117. Kill Switch Does Not Enable Weaker Path

Hard rule.

---

# 118. Rollback

Rollback must be planned.

---

# 119. Safe Rollback

Possible only if state remains backward-compatible.

---

# 120. Irreversible Migration

Must be explicitly marked.

---

# 121. Migration Metadata

```rust
pub struct MigrationDefinition {
    pub id: MigrationId,
    pub from: SchemaVersion,
    pub to: SchemaVersion,
    pub reversible: bool,
    pub privacy_impact: PrivacyImpactClass,
}
```

---

# 122. Privacy Impact Class

```rust
pub enum PrivacyImpactClass {
    None,
    Low,
    Moderate,
    Critical,
}
```

---

# 123. Critical Migration

Requires privacy review.

---

# 124. Rollback Window

Define before rollout.

---

# 125. Post-Window

Old software may no longer be safe.

---

# 126. Client Upgrade Policy

```rust
pub enum ClientUpgradePolicy {
    Optional,
    Recommended,
    RequiredSoon,
    Required,
    BlockedVersion,
}
```

---

# 127. Optional

Normal feature release.

---

# 128. Recommended

Better privacy/performance.

---

# 129. RequiredSoon

Deprecation window active.

---

# 130. Required

Protocol minimum no longer met.

---

# 131. BlockedVersion

Known critical vulnerability.

---

# 132. Update Metadata

Signed.

---

# 133. Update Manifest

```rust
pub struct SignedUpdateManifest {
    pub version: SoftwareVersion,
    pub protocol_support: ProtocolVersionRegistry,
    pub minimum_supported: Option<SoftwareVersion>,
    pub urgency: ClientUpgradePolicy,
    pub artifacts: Vec<SignedArtifactRef>,
    pub signature: ReleaseSignature,
}
```

---

# 134. Update Privacy

Fetching update should not reveal unique client identity.

---

# 135. Update Mirrors

Use multiple signed mirrors.

---

# 136. Same Artifact For Many Users

Avoid per-user binaries.

---

# 137. Update Timing Correlation

All clients updating simultaneously is fingerprintable.

---

# 138. Normal Updates

Use randomized rollout/wake window.

---

# 139. Critical Security Update

Faster, still jittered where safe.

---

# 140. Offline Update

Support signed package import.

---

# 141. Censorship Resistance

Use Part 41 alternate update distribution.

---

# 142. Update Anti-Rollback

Client remembers minimum accepted release/security epoch.

---

# 143. No Signed-But-Old Downgrade

Hard rule.

---

# 144. Release Epoch

```rust
pub struct ReleaseSecurityEpoch(pub u64);
```

---

# 145. Security Epoch Monotonic

Cannot decrease.

---

# 146. Protocol Deprecation

Lifecycle:

```text
announce
warn
stop selecting
reject
remove
```

---

# 147. Deprecation Record

```rust
pub struct ProtocolDeprecation {
    pub domain: ProtocolDomain,
    pub version: ProtocolVersion,
    pub warning_at: Timestamp,
    pub reject_at: Timestamp,
    pub reason: DeprecationReason,
}
```

---

# 148. Deprecation Reason

```rust
pub enum DeprecationReason {
    Security,
    Privacy,
    Incompatibility,
    Maintenance,
}
```

---

# 149. Security Deprecation

Can have short window.

---

# 150. Maintenance Deprecation

Longer window.

---

# 151. Version Fingerprinting

If clients expose exact rich capability lists, they can be fingerprinted.

---

# 152. Mitigation

Advertise standardized profiles.

---

# 153. Capability Profile

```rust
pub enum CapabilityProfile {
    Baseline,
    Modern,
    StrictPrivacy,
}
```

---

# 154. Prefer Profile Over Long Feature List

Hard rule where feasible.

---

# 155. Exact Version Exposure

Only where protocol requires.

---

# 156. Pad/normalize negotiation messages

Useful where practical.

---

# 157. Provider Fingerprinting

Same issue.

---

# 158. Standard Provider Profiles

Recommended.

---

# 159. Mixed-Version Privacy

Old clients can become rare and identifiable.

---

# 160. Long-Tail Risk

Version telemetry should detect coarse adoption without tracking users.

---

# 161. Adoption Metric

Aggregate by:

```text
major protocol profile
```

---

# 162. No Stable Client Version Identifier

Hard rule.

---

# 163. Minimum Cohort

Do not publish tiny version buckets.

---

# 164. Forced Upgrade Tradeoff

Can reduce support burden but harm disconnected users.

---

# 165. Offline Grace

If protocol still safe, allow cached operation.

---

# 166. Security Block

If unsafe, fail closed.

---

# 167. Provider Upgrade Orchestration

Provider sequence:

```text
drain
upgrade
health check
rejoin
```

---

# 168. Drain

Stop accepting new sessions.

---

# 169. Existing Sessions

Finish when safe.

---

# 170. Mix Node Drain

Stop new delayed packets after threshold.

---

# 171. Mailbox Drain

Continue reads; redirect/stop new deposits if migration planned.

---

# 172. Relay Drain

No new calls; active calls finish.

---

# 173. Bulk Provider Drain

No new uploads; existing uploads finish.

---

# 174. Bridge Drain

Rotate descriptor before shutdown.

---

# 175. Directory Node Drain

Ensure quorum unaffected.

---

# 176. Upgrade Waves

```rust
pub struct UpgradeWave {
    pub wave_id: u16,
    pub max_fraction: f32,
    pub minimum_health: HealthRequirement,
}
```

---

# 177. Wave 1

Tiny canary.

---

# 178. Wave 2

Small cross-region subset.

---

# 179. Wave 3

Broad.

---

# 180. Never Upgrade All Authorities Together

Hard rule.

---

# 181. Never Upgrade All Nodes Of One Layer Together

Hard rule.

---

# 182. Fault-Domain Aware Rollout

Spread across:

```text
regions
operators
ASNs
```

---

# 183. Health Gate

Next wave only if:

```text
SLO healthy
privacy canaries pass
no critical incidents
```

---

# 184. Automatic Pause

Triggered on regression.

---

# 185. Automatic Rollback

Only if migration is reversible.

---

# 186. Otherwise

Stop rollout and repair forward.

---

# 187. Repair Forward

Often safer for schema/crypto migrations.

---

# 188. Privacy Canary

Examples:

```text
strict route still anonymous
no new telemetry field leak
padding still active
capability negotiation authenticated
```

---

# 189. Compatibility Canary

Test old/new pairs.

---

# 190. Matrix Test Set

For each release:

```text
current↔current
current↔previous
previous↔current
```

---

# 191. N-2 Testing

If supported window includes N-2.

---

# 192. Cross-Service Upgrade Test

Example:

```text
new client
old mailbox
new relay
old catalog
```

---

# 193. Combinatorial Explosion

Use risk-based coverage.

---

# 194. Critical Paths Exhaustive

For:

```text
security
privacy
state migration
```

---

# 195. Testkit

Need mixed-version network simulator.

---

# 196. Simulator Nodes

Each can run:

```text
version X
capability profile Y
schema Z
```

---

# 197. Upgrade Scenarios

```text
rolling provider upgrade
client lag
region lag
authority lag
key rotation
emergency block
```

---

# 198. Wire Golden Tests

Keep versioned binary test vectors.

---

# 199. Golden Vector

```rust
pub struct ProtocolGoldenVector {
    pub domain: ProtocolDomain,
    pub version: ProtocolVersion,
    pub bytes: Vec<u8>,
    pub semantic_hash: [u8; 32],
}
```

---

# 200. Golden Test Rule

Old supported bytes must still decode identically.

---

# 201. Canonical Encoding

Required for signed messages.

---

# 202. Postcard

Suitable internal binary encoding if versioned/bounded.

---

# 203. RON

Human-readable configs/migrations.

---

# 204. JSON

External interop only where necessary.

---

# 205. Unknown Fields

Version-dependent behavior must be explicit.

---

# 206. Do Not Rely On Accidental Serde Compatibility

Hard rule.

---

# 207. Envelope Version

Every wire message should carry:

```text
domain
version
type
length
```

within privacy constraints.

---

# 208. Bounds

Version negotiation must not permit oversized allocation.

---

# 209. Parser Isolation

Old-version parser isolated.

---

# 210. Parser Retirement

Remove after deprecation window.

---

# 211. Legacy Attack Surface

Old parser code increases risk.

---

# 212. Therefore

Support window should stay bounded.

---

# 213. Capability Negotiation State Machine

```rust
pub enum NegotiationState {
    Init,
    Offered,
    Selected,
    Authenticated,
    Established,
    Failed,
}
```

---

# 214. Established Only After Authentication

Hard rule.

---

# 215. Migration Control Plane

Tracks rollout progress without user identity.

---

# 216. Rollout Record

```rust
pub struct RolloutRecord {
    pub release: SoftwareVersion,
    pub phase: RolloutPhase,
    pub infrastructure_fraction: f32,
    pub started_at: Timestamp,
}
```

---

# 217. No User Upgrade Ledger

Hard rule.

---

# 218. Client Adoption

Only aggregate.

---

# 219. Rollout Metrics

Safe:

```text
node version distribution
provider version distribution
protocol profile distribution
error rate
```

---

# 220. Forbidden Rollout Metrics

No:

```text
user ID + version
contact + version
mailbox + version
```

---

# 221. Version Compatibility API

```rust
pub trait CompatibilityPolicy {
    fn evaluate(
        &self,
        local: &ProtocolVersionRegistry,
        remote: &ProtocolVersionRegistry,
        required: PrivacyRoutingMode,
    ) -> Result<CompatibilityDecision, CompatibilityError>;
}
```

---

# 222. Compatibility Decision

```rust
pub enum CompatibilityDecision {
    Compatible(NegotiatedProfile),
    SafeFeatureReduction(NegotiatedProfile),
    UpgradeRequired,
    Blocked,
}
```

---

# 223. Protocol Migration Service

```rust
pub trait ProtocolMigrationService {
    fn plan(
        &self,
        from: ProtocolVersion,
        to: ProtocolVersion,
    ) -> Result<ProtocolMigrationPlan, MigrationError>;
}
```

---

# 224. Schema Migration Service

```rust
pub trait SchemaMigrationService {
    fn migrate(
        &self,
        store: &dyn VersionedStore,
        target: SchemaVersion,
    ) -> Result<MigrationReport, MigrationError>;
}
```

---

# 225. Rollout Controller

```rust
pub trait RolloutController {
    fn advance(
        &self,
        release: SoftwareVersion,
        health: &ReleaseHealth,
    ) -> Result<RolloutPhase, RolloutError>;

    fn pause(
        &self,
        release: SoftwareVersion,
        reason: RolloutPauseReason,
    ) -> Result<(), RolloutError>;
}
```

---

# 226. Release Health

```rust
pub struct ReleaseHealth {
    pub slo_status: SloStatus,
    pub privacy_canaries_ok: bool,
    pub compatibility_ok: bool,
    pub active_critical_incident: bool,
}
```

---

# 227. Upgrade Preconditions

Before deployment:

```text
compatibility tests pass
migration tests pass
privacy review done
rollback/repair-forward plan defined
```

---

# 228. Release Artifact Promotion

Build once.

---

# 229. Exact Artifact Promotion

Same signed artifact from canary to production.

---

# 230. No Rebuild Between Waves

Hard rule.

---

# 231. SBOM

Attached to release.

---

# 232. Reproducible Build

Preferred.

---

# 233. Release Signature

Required.

---

# 234. Artifact Hash

Pinned.

---

# 235. Supply Chain

Upgrade path must not bypass verification during emergency.

---

# 236. Provider Auto-Update

Possible.

---

# 237. Auto-Update Policy

Only signed release channel.

---

# 238. Staggered Update

Required.

---

# 239. Client Auto-Update

Platform-dependent.

---

# 240. Android

Store/manual APK may differ.

---

# 241. Desktop

Signed package/update.

---

# 242. Update Channel

```rust
pub enum UpdateChannel {
    Stable,
    Beta,
    Nightly,
}
```

---

# 243. Maximum Privacy

Stable recommended.

---

# 244. Nightly

Higher fingerprinting risk.

---

# 245. Beta Participation

Explicit opt-in.

---

# 246. Version Privacy UI

No need to show peer exact version.

---

# 247. Show Compatibility State

Examples:

```text
Secure
Upgrade recommended
Upgrade required
```

---

# 248. Do Not Expose Peer Software Build

Hard rule.

---

# 249. Upgrade Failure UX

Client:

```text
This contact/provider requires a newer secure protocol version.
```

---

# 250. No "try insecure compatibility mode"

Hard rule.

---

# 251. Network Upgrade Status

Normal UI should not expose network topology.

---

# 252. Advanced Diagnostics

Can show:

```text
protocol profile
minimum supported
catalog version
```

locally.

---

# 253. Migration Observability

Use Part 51.

---

# 254. Safe Metrics

```text
migration success
migration duration
rollback count
compatibility failures
```

---

# 255. Forbidden Metrics

No user IDs.

---

# 256. Migration Logging

No raw secrets/content.

---

# 257. Upgrade Incident

If rollout causes privacy regression:

```text
pause immediately
```

---

# 258. Privacy Regression

Critical severity.

---

# 259. Availability Regression

May pause based on error budget.

---

# 260. Protocol Freeze

During major incident.

---

# 261. Change Freeze

Prevents compounding failures.

---

# 262. Emergency Rollback

Only validated previous artifact.

---

# 263. If Previous Artifact Is Blocked

Repair forward.

---

# 264. Data Rollback

Dangerous.

---

# 265. Prefer Forward-Compatible State

Hard rule.

---

# 266. Migration Checkpoint

Keep enough info to resume.

---

# 267. Do Not Keep Full Old Secret State Longer Than Needed

Hard rule.

---

# 268. Privacy During Dual Write

Dual write can double metadata.

---

# 269. Review

Every dual-write migration needs privacy analysis.

---

# 270. Example

Writing same mailbox event to old/new service could create correlation.

---

# 271. Better

Internal logical dual representation where possible.

---

# 272. Multi-Provider Migration

If moving provider:

```text
new provider
→ dual receive
→ switch
→ retire old
```

---

# 273. Provider Migration Timing

Jittered.

---

# 274. Avoid Network-Wide Synchronized Migration

Hard rule.

---

# 275. Directory Authority Migration

Add new authority before removing old.

---

# 276. Threshold Change

Explicit governance event.

---

# 277. No Automatic Quorum Reduction

Hard rule.

---

# 278. Key Ceremony

For authority/root migration.

---

# 279. Documented Ceremony

Audited.

---

# 280. Client Trust Bundle Update

Signed by existing trust root.

---

# 281. Trust Root Rollover

Overlap period.

---

# 282. Root Rollover

```text
old signs new
new signs future
old retired after window
```

---

# 283. Root Compromise

Separate emergency recovery path.

---

# 284. No Trust-On-First-Use Replacement

Hard rule.

---

# 285. Capability Deprecation

Not all protocol features last forever.

---

# 286. Capability State

```rust
pub enum CapabilityLifecycle {
    Experimental,
    Stable,
    Deprecated,
    Removed,
}
```

---

# 287. Experimental Capability

Not required for core privacy guarantee.

---

# 288. Stable Capability

Production contract.

---

# 289. Deprecated Capability

Still accepted temporarily.

---

# 290. Removed Capability

Rejected.

---

# 291. Experimental Privacy Feature

Must not be marketed as guaranteed.

---

# 292. Rollout Gates

Before Broad:

```text
compatibility green
privacy canaries green
SLO healthy
migration success threshold met
no critical incidents
```

---

# 293. Release Gate Failure

Pause.

---

# 294. Rollout Audit

All wave changes recorded.

---

# 295. Audit Contains

```text
release
wave
operator action
timestamp
```

---

# 296. No User Identity

Hard rule.

---

# 297. Test Categories

Need:

```text
wire compatibility
schema migration
crypto migration
mixed-version
rollout
rollback
privacy
load
failure
```

---

# 298. Mixed-Version Matrix Test

Automated.

---

# 299. Rolling Upgrade Test

Bring nodes through versions gradually.

---

# 300. Partial Region Upgrade Test

One region ahead.

---

# 301. Operator Skew Test

One operator lags.

---

# 302. Client Lag Test

Old supported client still works.

---

# 303. Unsupported Client Test

Fails safely with upgrade-required.

---

# 304. Downgrade Attack Test

MITM modifies negotiation.

Must fail authentication.

---

# 305. Version Fingerprinting Test

Ensure standardized capability profile reduces uniqueness.

---

# 306. Parser Fuzzing

Every supported version.

---

# 307. Migration Fuzzing

Corrupt/interrupted state.

---

# 308. Crash-at-Every-Step Migration

Required for critical stores.

---

# 309. Rollback Test

Only when marked reversible.

---

# 310. Repair-Forward Test

For irreversible migration.

---

# 311. Key Rotation Test

Old/new key overlap.

---

# 312. Root Rollover Test

Trust continues.

---

# 313. Emergency Block Test

Blocked protocol cannot reconnect.

---

# 314. Catalog Dual-Version Test

Old/new clients consume signed catalog safely.

---

# 315. Provider Capability Mismatch Test

Advertised feature unavailable.

Reject.

---

# 316. Active Session Drain Test

Calls/uploads/mailboxes finish safely.

---

# 317. Failover During Upgrade Test

Node upgrade + region outage.

---

# 318. Chaos Upgrade Test

Inject failures mid-wave.

---

# 319. Privacy Lab Integration

Part 42 should test:

```text
version fingerprinting
migration timing correlation
rare-client identification
downgrade behavior
```

---

# 320. Formal Verification Targets

Strong candidates:

```text
negotiation state machine
anti-downgrade policy
migration state machine
rollout gate state
```

---

# 321. TLA+ Candidate

Distributed rolling upgrade with quorum constraints.

---

# 322. Kani Candidate

Version-range and minimum-security evaluation.

---

# 323. Loom Candidate

Concurrent schema migration/read-write.

---

# 324. Property Tests

Properties:

```text
negotiated version is mutually supported
negotiated version never below policy floor
blocked version never accepted
irreversible migration never auto-rolls back
```

---

# 325. Performance Tests

Measure:

```text
negotiation cost
dual-read overhead
migration throughput
rollout drain time
```

---

# 326. Large-State Migration Test

Millions of mailbox/control records.

---

# 327. Battery Impact

Client migration should avoid long foreground CPU where possible.

---

# 328. Android Migration

Use resumable background/foreground-safe process.

---

# 329. Desktop Migration

Can use daemon.

---

# 330. Client Startup

If migration required:

```text
perform before opening incompatible store
```

---

# 331. Progressive Migration

Possible for non-critical caches.

---

# 332. Security-Critical Store

Must fully migrate before use.

---

# 333. Crate Layout

Recommended:

```text
crates/
├── siar-versioning-core/
├── siar-protocol-registry/
├── siar-compatibility/
├── siar-negotiation/
├── siar-migration-core/
├── siar-schema-migration/
├── siar-crypto-migration/
├── siar-rollout/
├── siar-update-manifest/
├── siar-deprecation/
├── siar-upgrade-observability/
└── siar-upgrade-testkit/
```

---

# 334. `siar-versioning-core`

Owns:

```text
version types
ranges
lifecycle states
errors
```

---

# 335. `siar-protocol-registry`

Supported protocol domains/versions.

---

# 336. `siar-compatibility`

Compatibility matrix/policy.

---

# 337. `siar-negotiation`

Authenticated version/capability negotiation.

---

# 338. `siar-migration-core`

Migration state/journal/checkpoints.

---

# 339. `siar-schema-migration`

Persistent store migrations.

---

# 340. `siar-crypto-migration`

Suite/key migration.

---

# 341. `siar-rollout`

Wave/canary/gate controller.

---

# 342. `siar-update-manifest`

Signed releases/update policy.

---

# 343. `siar-deprecation`

Protocol/capability retirement.

---

# 344. `siar-upgrade-observability`

Privacy-safe rollout metrics.

---

# 345. `siar-upgrade-testkit`

Mixed-version simulator/golden vectors.

---

# 346. Error Taxonomy

```rust
pub enum UpgradeError {
    UnsupportedVersion,
    DowngradeDetected,
    CompatibilityUnsatisfied,
    MigrationFailed,
    MigrationIrreversible,
    RolloutPaused,
    HealthGateFailed,
    SignatureInvalid,
    UpdateBlocked,
    SchemaTooNew,
    SchemaTooOld,
    Internal,
}
```

---

# 347. Security Invariants

Mandatory:

```text
1. Version/capability negotiation is authenticated.
2. Negotiated protocol never falls below configured security/privacy floor.
3. Mixed-version operation may reduce features but never silently reduce required privacy.
4. Unknown protocol versions are rejected safely.
5. Rolling upgrades never upgrade all critical authorities/failure domains at once.
6. Canarying does not require stable user identity.
7. Release artifacts are built once and promoted unchanged.
8. Signed update/protocol policy is anti-rollback protected.
9. Crypto migration uses explicit overlap/prefer/enforce/retire phases.
10. Irreversible migrations cannot be auto-rolled back.
11. Capability advertisement happens only after runtime support is verified.
12. Version telemetry never becomes a per-user tracking mechanism.
```

---

# 348. Initial Production Scope

Implement first:

```text
per-domain protocol versions
current/previous compatibility window
authenticated negotiation
minimum-security floors
machine-readable compatibility matrix
expand/migrate/contract schema migration
signed update manifests
infrastructure canaries
fault-domain-aware rolling upgrades
health/privacy rollout gates
exact artifact promotion
protocol deprecation lifecycle
mixed-version testkit
golden wire vectors
```

Then add:

```text
advanced root-key rollover
multi-authority rollout orchestration
automated repair-forward planning
private client capability-profile negotiation
formal rollout verification
```

---

# 349. Definition of Done

Part 52 is complete when:

- protocol/software/schema/capability/crypto/policy versions are explicitly separated
- every wire domain has a bounded supported-version window
- capability negotiation is authenticated and downgrade-resistant
- compatibility matrices distinguish safe feature reduction from unsafe privacy degradation
- rolling upgrades work across mixed node/client/provider versions
- control/data-plane services have drain and migration behavior
- schema migrations are journaled, crash-safe, and resumable
- crypto migrations use overlap/prefer/enforce/retire phases
- update manifests and trust bundles are signed and anti-rollback protected
- canaries use infrastructure or ephemeral cohorts rather than stable user targeting
- release artifacts are promoted unchanged between waves
- rollout gates use SLO and privacy-canary health
- deprecation/removal windows are explicit
- version fingerprinting and rare-client risk are accounted for
- rollback vs repair-forward behavior is predeclared
- mixed-version, downgrade, migration, chaos, fuzz, and formal tests are specified

---

# 350. Final Architecture

```text
                 VERSIONED SPECIFICATION
                          │
                          ▼
                  COMPATIBILITY MATRIX
                          │
                          ▼
              AUTHENTICATED NEGOTIATION
                          │
                          ▼
                    CANARY RELEASE
                          │
                          ▼
                  ROLLING UPGRADE
                          │
              ┌───────────┴───────────┐
              │                       │
         Mixed Version           Migration
              │                       │
              └───────────┬───────────┘
                          ▼
                    BROAD ENFORCEMENT
                          │
                          ▼
                  DEPRECATE / REMOVE
```

Strict evolution model:

```text
versioned protocol
+
authenticated capability negotiation
+
security floors
+
mixed-version support
+
rolling upgrade
+
crash-safe migration
+
anti-rollback update policy
```

not:

```text
everyone upgrade at once
or silently fall back to whatever still works
```

---

# 351. Final Principle

A privacy network is only sustainable if it can evolve **without turning upgrades into either outages or security downgrades**.

The correct model is:

```text
explicit versions
+
bounded compatibility
+
authenticated negotiation
+
fault-domain-aware rollout
+
zero-downtime migration
+
privacy-aware deprecation
```

This architecture gives SIAR a disciplined path to evolve every anonymous-network subsystem while preserving interoperability, anonymity guarantees, and production availability across mixed-version deployments.
