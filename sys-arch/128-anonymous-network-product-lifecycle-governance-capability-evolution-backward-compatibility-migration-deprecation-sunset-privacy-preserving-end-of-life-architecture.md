# Core System Architecture Part 128 — Anonymous Network Product Lifecycle Governance, Capability Evolution, Backward Compatibility, Migration, Deprecation, Sunset & Privacy-Preserving End-of-Life Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 128  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 52, 57, 67, 83, 99, 107–108, 122–127

**Primary purpose:** define SIAR's product-lifecycle architecture for post-GA evolution, backward compatibility, compatibility promises, migration contracts, replacement capabilities, protocol/client retirement, deprecation, sunset, end-of-life (EOL), data portability, archival obligations, rollback boundaries, lifecycle evidence, and privacy-preserving product retirement.

---

# 1. Purpose

A capability is not finished when it reaches General Availability.

Every production feature eventually changes:

```text
protocols evolve
dependencies disappear
clients age
storage formats migrate
products merge
capabilities become obsolete
security requirements tighten
```

A mature platform must answer:

```text
How long is compatibility promised?
How is a breaking change introduced safely?
How does a replacement capability take over?
What happens to old data?
What happens to offline clients?
When may support end?
What evidence proves a capability is safe to retire?
How are users prevented from being trapped?
```

The governing principle is:

> **SIAR product lifecycle governance should make evolution explicit, compatible where promised, migratable where necessary, reversible where feasible, and honest about deprecation and end-of-life—without using user profiling, forced lock-in, or opaque telemetry to justify retirement.**

---

# 2. Architectural Position

```text
                    GENERAL AVAILABILITY
                              │
                              ▼
                           MATURE
                              │
                 ┌────────────┼────────────┐
                 │            │            │
             EVOLVE        REPLACE      MAINTAIN
                 │            │            │
                 └────────────┼────────────┘
                              ▼
                          DEPRECATED
                              │
                              ▼
                       MIGRATION WINDOW
                              │
                              ▼
                            SUNSET
                              │
                              ▼
                          END-OF-LIFE
                              │
                              ▼
                           RETIRED
```

---

# 3. Core Separation

Keep distinct:

```text
deprecation
sunset
end-of-support
end-of-life
retirement
removal
migration
replacement
```

---

# 4. Non-Goals

Part 128 does not create:

```text
forced lock-in
surprise removals
user engagement-based EOL
developer performance scoring
silent compatibility breakage
```

---

# 5. Lifecycle State

```rust
pub enum ProductLifecycleState {
    GeneralAvailability,
    Mature,
    EvolutionPlanned,
    ReplacementAvailable,
    Deprecated,
    SunsetScheduled,
    EndOfSupport,
    EndOfLife,
    Retired,
}
```

---

# 6. No Direct GA→Retired

Hard rule.

---

# 7. Lifecycle Record

```rust
pub struct CapabilityLifecycleRecord {
    pub capability: CapabilityId,
    pub state: ProductLifecycleState,
    pub effective_at: Timestamp,
    pub replacement: Option<CapabilityId>,
}
```

---

# 8. Historical Lifecycle Immutable

Hard rule.

---

# 9. Lifecycle Transition Request

```rust
pub struct LifecycleTransitionRequest {
    pub capability: CapabilityId,
    pub from: ProductLifecycleState,
    pub to: ProductLifecycleState,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 10. Transition Policy

```rust
pub trait ProductLifecyclePolicy {
    fn allowed(
        &self,
        from: ProductLifecycleState,
        to: ProductLifecycleState,
    ) -> bool;
}
```

---

# 11. Hard rule.

---

# 12. Capability Evolution

Evolution may change:

```text
behavior
protocol
storage format
API
security policy
privacy semantics
```

---

# 13. Evolution Class

```rust
pub enum CapabilityEvolutionClass {
    CompatibleEnhancement,
    BehavioralChange,
    SchemaChange,
    ProtocolChange,
    SecurityTightening,
    Replacement,
    BreakingChange,
}
```

---

# 14. Hard rule.

---

# 15. Evolution Intent

```rust
pub struct CapabilityEvolutionPlan {
    pub capability: CapabilityId,
    pub class: CapabilityEvolutionClass,
    pub target_release: ReleaseId,
    pub migration: Option<MigrationPlanId>,
}
```

---

# 16. Backward Compatibility

Explicit promise.

```rust
pub enum BackwardCompatibilityPromise {
    None,
    BestEffort,
    LimitedWindow(Duration),
    Stable,
    LongTermStable,
}
```

---

# 17. Hard rule.

---

# 18. Compatibility Surface

```rust
pub enum CompatibilitySurface {
    Api,
    WireProtocol,
    PersistentData,
    Configuration,
    Cli,
    PluginInterface,
    BackupFormat,
}
```

---

# 19. Compatibility Contract

```rust
pub struct CompatibilityContract {
    pub capability: CapabilityId,
    pub surface: CompatibilitySurface,
    pub promise: BackwardCompatibilityPromise,
}
```

---

# 20. Hard rule.

---

# 21. No Implicit Compatibility Promise

Hard rule.

---

# 22. Breaking Change

A change is breaking if it invalidates a promised surface.

---

# 23. Hard rule.

---

# 24. Breaking Change Requirements

Must include:

```text
impact analysis
migration path
communication plan
sunset timeline
rollback/forward plan
```

---

# 25. Hard rule.

---

# 26. Protocol Evolution

Part 52.

---

# 27. Protocol Version State

```rust
pub enum ProtocolVersionLifecycle {
    Current,
    Supported,
    Deprecated,
    SunsetScheduled,
    Unsupported,
}
```

---

# 28. No Silent Unsupported Version Drop

Hard rule.

---

# 29. Version Negotiation

Must authenticate capabilities/version.

---

# 30. Hard rule.

---

# 31. Old Client Compatibility

Policy-defined window.

---

# 32. Hard rule.

---

# 33. Client Version Support

```rust
pub struct ClientSupportWindow {
    pub platform: PlatformClass,
    pub minimum_supported: Version,
    pub recommended: Version,
    pub end_of_support: Option<Timestamp>,
}
```

---

# 34. Hard rule.

---

# 35. Offline Client Consideration

Local-first clients may remain offline for long periods.

---

# 36. Migration must tolerate delayed reconnect.

---

# 37. Hard rule.

---

# 38. Reconnect Migration

Old client reconnect flow:

```text
authenticate
negotiate supported protocol
determine migration requirement
sync safe state
upgrade/guide
```

---

# 39. No Data Destruction On Old-Client Reconnect

Hard rule.

---

# 40. Persistent Data Evolution

Schema changes require migration contracts.

---

# 41. Migration Contract

```rust
pub struct MigrationContract {
    pub migration: MigrationPlanId,
    pub from_schema: SchemaVersion,
    pub to_schema: SchemaVersion,
    pub reversibility: MigrationReversibility,
}
```

---

# 42. Migration Reversibility

```rust
pub enum MigrationReversibility {
    Reversible,
    ConditionallyReversible,
    ForwardOnly,
}
```

---

# 43. ForwardOnly Requires Stronger Readiness

Hard rule.

---

# 44. Migration Preconditions

```text
backup/recovery verified
capacity available
compatibility window known
rollback boundary understood
```

---

# 45. Hard rule.

---

# 46. Expand-Migrate-Contract

Preferred for live schema evolution.

---

# 47. Hard rule.

---

# 48. Dual Read / Dual Write

Temporary compatibility strategy.

---

# 49. Must be time-bounded.

---

# 50. Hard rule.

---

# 51. Migration State

```rust
pub enum MigrationState {
    Planned,
    Ready,
    Running,
    DualCompatibility,
    Cutover,
    Verifying,
    Completed,
    RolledBack,
    Failed,
}
```

---

# 52. No Completed Before Verification

Hard rule.

---

# 53. Data Version Marker

Explicit.

---

# 54. Hard rule.

---

# 55. Data Downgrade

Only if reversibility proven.

---

# 56. Hard rule.

---

# 57. Backup Compatibility

Older backup formats may need import support.

---

# 58. Hard rule.

---

# 59. Backup Format Support Window

```rust
pub struct BackupFormatSupport {
    pub format: BackupFormatVersion,
    pub import_supported_until: Timestamp,
}
```

---

# 60. Hard rule.

---

# 61. Configuration Evolution

Config schema versioned.

---

# 62. Auto-migration allowed only if deterministic.

---

# 63. Hard rule.

---

# 64. Config Deprecation

Warn before removal.

---

# 65. Hard rule.

---

# 66. API Evolution

Public API deprecation requires:

```text
replacement
timeline
migration guidance
```

---

# 67. Hard rule.

---

# 68. Plugin Interface Evolution

Part 24/68.

---

# 69. Plugin compatibility scoped by declared interface version.

---

# 70. Hard rule.

---

# 71. Replacement Capability

```rust
pub struct CapabilityReplacement {
    pub old: CapabilityId,
    pub new: CapabilityId,
    pub parity: ReplacementParity,
}
```

---

# 72. Replacement Parity

```rust
pub enum ReplacementParity {
    Partial,
    FunctionalEquivalent,
    Superset,
}
```

---

# 73. No "Replacement Available" Without Declared Gaps

Hard rule.

---

# 74. Replacement Readiness

Replacement must be at sufficient maturity.

---

# 75. Hard rule.

---

# 76. Replacement Migration

Users/tenants need clear path.

---

# 77. Hard rule.

---

# 78. No Forced Migration Before Replacement Ready

Hard rule.

---

# 79. Deprecation

Deprecation means:

```text
still usable
discouraged for new use
replacement/migration available or planned
support timeline known
```

---

# 80. Hard rule.

---

# 81. Deprecation Record

```rust
pub struct DeprecationRecord {
    pub capability: CapabilityId,
    pub deprecated_at: Timestamp,
    pub sunset_not_before: Timestamp,
    pub replacement: Option<CapabilityId>,
}
```

---

# 82. Hard rule.

---

# 83. Minimum Deprecation Window

Policy-driven.

---

# 84. Security Emergency Can Shorten

Hard rule.

---

# 85. Sunset

Sunset means:

```text
new adoption blocked
migration strongly required
service removal scheduled
```

---

# 86. Sunset Record

```rust
pub struct SunsetPlan {
    pub capability: CapabilityId,
    pub sunset_at: Timestamp,
    pub migration_deadline: Timestamp,
    pub retirement_at: Timestamp,
}
```

---

# 87. Hard rule.

---

# 88. End Of Support

Support ends before full retirement in some cases.

---

# 89. Hard rule.

---

# 90. End Of Life

At EOL:

```text
no new fixes except critical transition/security cases
migration/export remains where promised
```

---

# 91. Hard rule.

---

# 92. Retirement

Retirement means capability no longer active in supported product.

---

# 93. Hard rule.

---

# 94. Retirement Preconditions

```text
migration complete or explicit remaining exception
data obligations resolved
replacement/exit path provided
runtime references removed
support/docs updated
```

---

# 95. Hard rule.

---

# 96. Data Portability

Before retirement, user/tenant-owned data must remain exportable as promised.

---

# 97. Hard rule.

---

# 98. Export Format

Portable, documented, versioned.

---

# 99. Hard rule.

---

# 100. No Proprietary Lock-In As Exit Strategy

Hard rule.

---

# 101. Data Deletion

Retirement must define deletion timing.

---

# 102. Hard rule.

---

# 103. Deletion State

```rust
pub enum LifecycleDataDisposition {
    Migrated,
    Exported,
    Deleted,
    ArchivedByPolicy,
}
```

---

# 104. Hard rule.

---

# 105. Archival

Only where retention policy requires.

---

# 106. Hard rule.

---

# 107. Cryptographic Erasure

Part 67.

---

# 108. Hard rule.

---

# 109. Orphan Data

No forgotten tables/blobs/indexes after capability retirement.

---

# 110. Hard rule.

---

# 111. Data Inventory Check

Part 106/123.

---

# 112. Hard rule.

---

# 113. Capability Runtime Reference

```rust
pub struct CapabilityRuntimeReference {
    pub capability: CapabilityId,
    pub deployment: DeploymentId,
    pub active: bool,
}
```

---

# 114. Retirement Requires Zero Active Required References

Hard rule.

---

# 115. Service Dependency Removal

Reverse dependency scan.

---

# 116. Hard rule.

---

# 117. Protocol Retirement

Requires:

```text
no supported clients depend
no active federation peer depends unless scoped exception
migration complete
```

---

# 118. Hard rule.

---

# 119. Federation Compatibility

Part 58.

Peer support window explicit.

---

# 120. Hard rule.

---

# 121. Cross-Domain Sunset

No unilateral silent protocol break.

---

# 122. Hard rule.

---

# 123. Tenant-Specific Migration

Managed tenants may have negotiated windows.

---

# 124. Does not weaken platform hard security floor.

---

# 125. Hard rule.

---

# 126. Security-Driven Sunset

Critical vulnerability may force accelerated retirement.

---

# 127. Still provide:

```text
safe migration
export
communication
```

where possible.

---

# 128. Hard rule.

---

# 129. Privacy-Driven Sunset

If capability cannot meet privacy floor, restrict/retire.

---

# 130. Hard rule.

---

# 131. Reliability-Driven Sunset

Repeated systemic failure may trigger replacement.

---

# 132. Part 120/121 integration.

---

# 133. Hard rule.

---

# 134. Cost Is Not Sole Retirement Reason

Hard rule.

---

# 135. Sustainability Is Not Sole Retirement Reason

Hard rule.

---

# 136. End-Of-Life Decision

```rust
pub struct EndOfLifeDecision {
    pub capability: CapabilityId,
    pub rationale: EndOfLifeRationale,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 137. Rationale

```rust
pub enum EndOfLifeRationale {
    SecurityUnsustainable,
    ReliabilityUnsustainable,
    DependencyEndOfLife,
    ReplacementComplete,
    ProtocolObsolete,
    OperationalBurden,
    ProductConsolidation,
}
```

---

# 138. Hard rule.

---

# 139. Evidence Required

Hard rule.

---

# 140. No Adoption-Based Auto-Sunset

Hard rule.

---

# 141. Low Usage Can Inform Planning

But not replace technical/data/user-obligation review.

---

# 142. Hard rule.

---

# 143. Lifecycle Communication

Need:

```text
internal operators
developers/integrators
managed tenants
end users if applicable
```

---

# 144. Hard rule.

---

# 145. Communication Timeline

```text
deprecation notice
migration reminders
sunset reminder
final EOL notice
```

---

# 146. Hard rule.

---

# 147. No Dark Patterns

Do not coerce migration through misleading UI.

---

# 148. Hard rule.

---

# 149. User-Facing Messaging

Must state:

```text
what changes
when
what user must do
what happens to data
where migration/export is available
```

---

# 150. Hard rule.

---

# 151. Offline Users

Need delayed-reconnect handling.

---

# 152. Hard rule.

---

# 153. Missed Sunset Notification

Reconnect flow still provides migration path.

---

# 154. Hard rule.

---

# 155. Accessibility

Migration/EOL notices accessible.

---

# 156. Hard rule.

---

# 157. Support Transition

Support team receives:

```text
timeline
migration tooling
known issues
exception process
```

---

# 158. Hard rule.

---

# 159. Operational Transition

On-call/runbooks updated.

---

# 160. Hard rule.

---

# 161. Feature Flags

Flags may disable deprecated feature.

---

# 162. Flag ≠ lifecycle state.

---

# 163. Hard rule.

---

# 164. Lifecycle Exceptions

Temporary extension.

```rust
pub struct LifecycleException {
    pub capability: CapabilityId,
    pub scope: LifecycleExceptionScope,
    pub expires_at: Timestamp,
    pub rationale: ExceptionRationale,
}
```

---

# 165. Exception Scope

```rust
pub enum LifecycleExceptionScope {
    Tenant(TenantId),
    Region(RegionId),
    FederationPeer(FederationDomainId),
    Deployment(DeploymentId),
}
```

---

# 166. No Individual User Exception Baseline

Hard rule.

---

# 167. Hard Invariant

Exception cannot extend insecure/unacceptable state below platform floor.

---

# 168. Hard rule.

---

# 169. Exception Renewal

Fresh review required.

---

# 170. No perpetual extension.

---

# 171. Hard rule.

---

# 172. Migration Tooling

Provide:

```text
dry run
progress
resume
verification
rollback where possible
```

---

# 173. Hard rule.

---

# 174. Migration Dry Run

```rust
pub struct MigrationDryRun {
    pub migration: MigrationPlanId,
    pub impacted_records: u64,
    pub estimated_duration: Duration,
    pub blocking_issues: Vec<MigrationIssue>,
}
```

---

# 175. No private content in dry-run summary.

---

# 176. Hard rule.

---

# 177. Migration Checkpoint

Durable state.

---

# 178. Hard rule.

---

# 179. Migration Resume

Idempotent.

---

# 180. Hard rule.

---

# 181. Migration Verification

```rust
pub struct MigrationVerification {
    pub migration: MigrationPlanId,
    pub source_count: u64,
    pub destination_count: u64,
    pub integrity_state: IntegrityState,
}
```

---

# 182. Counts are technical aggregate.

---

# 183. Hard rule.

---

# 184. Rollback Boundary

Some migration steps may cross point of no return.

---

# 185. Explicit.

---

# 186. Hard rule.

---

# 187. Forward Recovery

If rollback impossible, forward-fix path must exist.

---

# 188. Hard rule.

---

# 189. Protocol Downgrade

Never allow downgrade below security/privacy floor.

---

# 190. Hard rule.

---

# 191. Compatibility Test Matrix

```rust
pub struct CompatibilityTestMatrix {
    pub old_versions: BTreeSet<Version>,
    pub new_versions: BTreeSet<Version>,
    pub scenarios: Vec<CompatibilityScenario>,
}
```

---

# 192. Hard rule.

---

# 193. Compatibility Scenarios

```text
old client/new server
new client/old server
mixed multi-device
offline old client reconnect
federation version skew
```

---

# 194. Hard rule.

---

# 195. Version Skew Policy

```rust
pub struct VersionSkewPolicy {
    pub max_client_skew: VersionDistance,
    pub max_server_skew: VersionDistance,
}
```

---

# 196. Hard rule.

---

# 197. Compatibility Evidence

Stored in Part 126 archive.

---

# 198. Hard rule.

---

# 199. Lifecycle Baseline

```rust
pub struct LifecycleBaseline {
    pub capability: CapabilityId,
    pub state: ProductLifecycleState,
    pub compatibility_contracts: Vec<CompatibilityContract>,
    pub migration_plan: Option<MigrationPlanId>,
}
```

---

# 200. Immutable per release.

---

# 201. Hard rule.

---

# 202. Release Binding

Every release pins lifecycle baseline.

---

# 203. Hard rule.

---

# 204. Historical Reconstruction

"What compatibility promise applied in release X?"

Core requirement.

---

# 205. Hard rule.

---

# 206. Knowledge Graph Integration

Part 123.

Trace:

```text
capability
→ lifecycle
→ replacement
→ migration
→ protocol versions
→ tests
→ release
```

---

# 207. Hard rule.

---

# 208. Requirements Integration

Part 124.

Deprecation/removal may change requirement set.

---

# 209. Hard rule.

---

# 210. Assurance Integration

Part 125.

Migration/compatibility qualification required.

---

# 211. Hard rule.

---

# 212. Archive Integration

Part 126.

Lifecycle decisions/evidence archived.

---

# 213. Hard rule.

---

# 214. Product Readiness Integration

Part 127.

Replacement capability must meet readiness threshold.

---

# 215. Hard rule.

---

# 216. Architecture Governance

Part 122.

Breaking changes require ADR/design review.

---

# 217. Hard rule.

---

# 218. Risk Integration

Part 121.

Lifecycle transition surfaces migration/EOL risks.

---

# 219. Hard rule.

---

# 220. PIR Integration

Part 120.

Incident learning can accelerate replacement/sunset.

---

# 221. Hard rule.

---

# 222. Change Integration

Part 107.

Lifecycle transitions are governed changes.

---

# 223. Hard rule.

---

# 224. Release Integration

Part 108.

Release must satisfy active compatibility/lifecycle promises.

---

# 225. Hard rule.

---

# 226. Update Integration

Part 99.

Client upgrade enforcement only within published support policy.

---

# 227. Hard rule.

---

# 228. Account/Data Lifecycle Integration

Parts 67/83.

Capability retirement cannot resurrect deleted data or break deletion commitments.

---

# 229. Hard rule.

---

# 230. Backup/Restore Integration

Part 100.

Retired capability data may remain in old backup.

---

# 231. Restore must migrate or quarantine old format.

---

# 232. Hard rule.

---

# 233. Restore Anti-Resurrection

No re-enabling retired/insecure capability automatically.

---

# 234. Hard rule.

---

# 235. Federation Integration

Part 58.

Lifecycle policy advertised to peers.

---

# 236. Hard rule.

---

# 237. Plugin Ecosystem Integration

Part 24/68.

Plugin compatibility deprecation communicated.

---

# 238. Hard rule.

---

# 239. SDK Lifecycle

SDK version support window.

---

# 240. Hard rule.

---

# 241. API Client Generation

Old generated clients supported according to compatibility contract.

---

# 242. Hard rule.

---

# 243. CLI Lifecycle

Deprecated commands warn.

---

# 244. Removal requires replacement/documentation.

---

# 245. Hard rule.

---

# 246. Config Key Lifecycle

```rust
pub enum ConfigKeyLifecycle {
    Active,
    Deprecated,
    IgnoredWithWarning,
    Removed,
}
```

---

# 247. Hard rule.

---

# 248. No Silent Ignore Before Published Window

Hard rule.

---

# 249. Data Format Evolution

RON/Postcard schema versioning explicit.

---

# 250. Hard rule.

---

# 251. Postcard Compatibility

Never assume backward compatibility without explicit schema strategy.

---

# 252. Hard rule.

---

# 253. RON Human Config

Unknown/deprecated keys handled explicitly.

---

# 254. Hard rule.

---

# 255. JSON External Interop

Versioned externally.

---

# 256. Hard rule.

---

# 257. Lifecycle Decision Authority

```rust
pub enum LifecycleDecisionAuthority {
    ProductOwner,
    PlatformArchitecture,
    SecurityAssurance,
    PrivacyAssurance,
    ReliabilityAssurance,
    ReleaseEngineering,
}
```

---

# 258. Scoped authority.

---

# 259. No Single Person Can Override Hard Block

Hard rule.

---

# 260. Deprecation Approval

Requires product + technical authority.

---

# 261. Hard rule.

---

# 262. Accelerated Security Sunset

Requires security authority + migration/communication plan.

---

# 263. Hard rule.

---

# 264. Retirement Approval

Requires:

```text
data disposition verified
runtime references zero
compatibility obligations completed
```

---

# 265. Hard rule.

---

# 266. Lifecycle Review

```rust
pub struct ProductLifecycleReview {
    pub capability: CapabilityId,
    pub current_state: ProductLifecycleState,
    pub target_state: ProductLifecycleState,
    pub migration_readiness: ReadinessState,
    pub compatibility_readiness: ReadinessState,
    pub data_disposition_readiness: ReadinessState,
}
```

---

# 267. Hard rule.

---

# 268. Lifecycle Decision

```rust
pub enum ProductLifecycleDecision {
    Advance,
    Hold,
    Regress,
    CancelSunset,
}
```

---

# 269. Hard rule.

---

# 270. Cancel Sunset

Allowed if evidence changes.

---

# 271. Hard rule.

---

# 272. Lifecycle Freshness

```rust
pub enum LifecycleEvidenceFreshness {
    Fresh,
    DueForReview,
    Stale,
}
```

---

# 273. Stale migration/compatibility evidence blocks retirement.

---

# 274. Hard rule.

---

# 275. Lifecycle SLOs

Examples:

```text
deprecated capability has replacement/migration docs
sunset notices issued before deadline
migration failures below threshold
retired capability has zero runtime references
```

---

# 276. Hard rule.

---

# 277. Security SLO

```text
0 unsupported insecure protocol re-enabled after sunset
0 retirement bypassing data sanitization
```

---

# 278. Privacy SLO

```text
0 lifecycle decision based on covert user profiling
0 retirement process exposing private content
```

---

# 279. Failure Modes

```text
surprise breaking change
permanent deprecation
replacement not ready
orphan data after retirement
old offline clients unable to recover
```

---

# 280. Surprise Breaking Change

Prevent with compatibility contract + release gate.

---

# 281. Hard rule.

---

# 282. Permanent Deprecation

Review deadlines.

---

# 283. Hard rule.

---

# 284. Replacement Not Ready

Delay forced migration.

---

# 285. Hard rule.

---

# 286. Orphan Data

Inventory/data-disposition gate blocks retirement.

---

# 287. Hard rule.

---

# 288. Offline Old Client

Reconnect migration path.

---

# 289. Hard rule.

---

# 290. EOL Reversal

Rare but possible before final retirement.

---

# 291. Requires new lifecycle decision.

---

# 292. Hard rule.

---

# 293. Testing

Need product-lifecycle testkit.

---

# 294. Test Scenarios

```text
protocol deprecation
database schema migration
offline client reconnect after sunset
capability replacement
retirement with data export
```

---

# 295. Transition Test

GA→Retired direct transition rejected.

---

# 296. Compatibility Test

Stable API cannot break without governed breaking-change process.

---

# 297. Old Client Test

Supported old client still interoperates inside window.

---

# 298. Offline Reconnect Test

Old offline client can migrate without data loss.

---

# 299. Migration Test

Completed state requires integrity verification.

---

# 300. Forward-Only Test

Requires stronger backup/recovery evidence.

---

# 301. Replacement Test

Forced migration blocked if replacement below required maturity.

---

# 302. Sunset Test

New adoption blocked after sunset.

---

# 303. Retirement Test

Active runtime reference prevents retirement.

---

# 304. Data Test

Orphan data prevents retirement.

---

# 305. Restore Test

Old backup cannot silently resurrect retired capability.

---

# 306. Privacy Test

No user behavior segmentation needed for lifecycle decisions.

---

# 307. Federation Test

Peer version sunset does not silently break negotiated window.

---

# 308. Exception Test

Expired lifecycle exception stops extension.

---

# 309. Fuzzing

Fuzz:

```text
lifecycle transitions
compatibility contracts
migration states
sunset plans
lifecycle exceptions
```

---

# 310. Property Tests

Properties:

```text
capability can never retire while active required runtime references remain
hard security floor can never be weakened by lifecycle exception
historical lifecycle baseline can never mutate
replacement-dependent sunset can never force migration before replacement reaches required maturity
```

---

# 311. Formal Verification Targets

Strong candidates:

```text
lifecycle state machine
migration state machine
compatibility window
sunset/exception expiry
```

---

# 312. Kani Candidate

transition/exception/runtime-reference invariants.

---

# 313. TLA+ Candidate

GA → deprecate → migrate → sunset → EOL → retire, including rollback/cancel sunset.

---

# 314. Loom Candidate

concurrent client reconnect + migration + sunset transition.

---

# 315. Performance

Lifecycle governance is control-plane work.

---

# 316. Migration hot paths separately benchmarked.

---

# 317. Hard rule.

---

# 318. Storage

Separate:

```text
lifecycle records
compatibility contracts
migration plans
deprecation records
sunset plans
EOL decisions
lifecycle exceptions
data disposition receipts
```

---

# 319. No user/developer analytics warehouse.

---

# 320. Hard rule.

---

# 321. Partitioning

By:

```text
capability
protocol
platform
release
lifecycle state
```

---

# 322. No person/user partition.

---

# 323. Hard rule.

---

# 324. Crate Layout

Recommended:

```text
crates/
├── siar-lifecycle-core/
├── siar-capability-evolution/
├── siar-compatibility-policy/
├── siar-migration-governance/
├── siar-deprecation/
├── siar-sunset/
├── siar-eol/
├── siar-lifecycle-exceptions/
├── siar-lifecycle-observability/
└── siar-lifecycle-testkit/
```

---

# 325. `siar-lifecycle-core`

Owns:

```text
ProductLifecycleState
CapabilityEvolutionClass
BackwardCompatibilityPromise
ProductLifecycleError
```

---

# 326. `siar-capability-evolution`

Evolution plans/replacements.

---

# 327. `siar-compatibility-policy`

API/protocol/data/config compatibility contracts.

---

# 328. `siar-migration-governance`

Migration state/checkpoints/verification.

---

# 329. `siar-deprecation`

Deprecation lifecycle/window.

---

# 330. `siar-sunset`

Sunset schedule/communications/gates.

---

# 331. `siar-eol`

End-of-support/EOL/retirement/data disposition.

---

# 332. `siar-lifecycle-exceptions`

Scoped time-bounded extensions.

---

# 333. `siar-lifecycle-observability`

Aggregate lifecycle health only.

---

# 334. `siar-lifecycle-testkit`

compatibility/migration/retirement/privacy tests.

---

# 335. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Capability lifecycle state, compatibility promises, migration contracts, deprecation windows, sunset, EOL, and retirement are explicit versioned records and can never be inferred only from feature flags or code removal.
2. Backward compatibility is a declared contract by surface—API, wire protocol, persistent data, configuration, plugin interface, backup format—and breaking a promised surface requires governed migration and release evidence.
3. A capability cannot move directly from GA to Retired; deprecation, migration, sunset, data disposition, support transition, and retirement verification are distinct lifecycle stages.
4. Replacement-dependent migration cannot be forced before the replacement reaches the required product/readiness maturity and documented parity/gaps are understood.
5. Offline/local-first clients, delayed reconnects, mixed-version multi-device states, and federation version skew are first-class compatibility scenarios and cannot be ignored during sunset.
6. Migration completion requires integrity verification, durable checkpoints, resumability, and rollback/forward-recovery semantics appropriate to reversibility; forward-only migration receives stronger assurance.
7. Retirement cannot complete while required runtime references, unresolved data stores, backup/import obligations, active federation dependencies, or compatibility promises remain outstanding.
8. Lifecycle exceptions are scoped, time-bounded, reviewed, and cannot extend a capability below hard security/privacy/reliability floors or become permanent hidden support commitments.
9. Capability retirement preserves data portability, export, deletion, archival, and cryptographic-erasure obligations and cannot use proprietary lock-in or inaccessible formats as an exit strategy.
10. Lifecycle communications are truthful, accessible, sufficiently early, and explicit about behavior/data/migration impact; dark patterns and surprise removals are prohibited.
11. Lifecycle decisions use technical, support, security, privacy, reliability, compatibility, and obligation evidence and cannot be driven by covert user profiling, sensitive-segment analysis, or developer/product-owner scoring.
12. Product lifecycle governance integrates with product readiness, requirements, engineering assurance, assurance archives, architecture governance, risk/PIR, release/update control, backup/restore, data lifecycle, federation, plugin/SDK/API governance, and audit/compliance without creating an alternate route around platform trust.
```

---

# 336. Initial Production Scope

Implement first:

```text
typed ProductLifecycleState
capability lifecycle registry
compatibility contracts by surface
client/protocol support windows
migration contracts/state machine
offline reconnect migration
backup-format compatibility
replacement parity/readiness checks
deprecation records/windows
sunset plans
EOL/retirement decisions
data disposition checks
runtime-reference checks
federation/plugin/API lifecycle hooks
lifecycle exceptions with expiry
release lifecycle baselines
assurance-archive integration
privacy-safe lifecycle dashboards
lifecycle testkit
```

Then add:

```text
automated reverse-dependency retirement analysis
cross-version compatibility matrix generation
migration digital twins
temporal lifecycle graph
federation sunset negotiation tooling
portable migration bundles
formal sunset/retirement verification
```

---

# 337. Definition of Done

Part 128 is complete when:

- lifecycle states are typed/versioned;
- compatibility promises are explicit by surface;
- breaking changes require migration governance;
- old/offline clients have defined handling;
- migrations are resumable and verified;
- replacements have maturity/parity requirements;
- deprecation/sunset/EOL are distinct;
- data export/deletion/archival obligations are enforced;
- retirement requires zero blocking runtime/data/dependency references;
- lifecycle exceptions expire;
- historical lifecycle baselines are reconstructable;
- no user/developer profiling drives lifecycle decisions;
- compatibility/migration/privacy/fuzz/formal tests are specified.

---

# 338. Final Architecture

```text
                   GENERAL AVAILABILITY
                            │
                            ▼
                         EVOLUTION
                            │
              ┌─────────────┼─────────────┐
              │             │             │
        COMPATIBILITY    MIGRATION    REPLACEMENT
              │             │             │
              └─────────────┼─────────────┘
                            ▼
                       DEPRECATION
                            │
                            ▼
                          SUNSET
                            │
                            ▼
                      END-OF-LIFE
                            │
                            ▼
                        RETIREMENT
```

Lifecycle-governance safety model:

```text
explicit compatibility contracts
+
versioned migration plans
+
offline-client handling
+
replacement readiness
+
deprecation windows
+
data portability/disposition
+
retirement verification
+
historical lifecycle evidence
```

not:

```text
remove old behavior when convenient, break old clients silently, force users onto an unready replacement, and leave orphaned data behind
```

---

# 339. Final Principle

A trustworthy product lifecycle does not merely explain how a feature starts—it explains how that feature can change, migrate, and eventually end without breaking promises.

The correct model is:

```text
declare compatibility
+
evolve deliberately
+
migrate safely
+
support delayed/offline clients
+
deprecate visibly
+
sunset predictably
+
preserve user data rights
+
verify retirement
+
never use lifecycle management as a mechanism for lock-in or surveillance
```

This architecture gives SIAR a privacy-preserving lifecycle foundation for capability evolution, compatibility, migration, deprecation, sunset, EOL, replacement, data portability, and retirement while preserving the anonymity, local-first, least-authority, product-readiness, engineering-assurance, and anti-surveillance guarantees established across Parts 34–127.
