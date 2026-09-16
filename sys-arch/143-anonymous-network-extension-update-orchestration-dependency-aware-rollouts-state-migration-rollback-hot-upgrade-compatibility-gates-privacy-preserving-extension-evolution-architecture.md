# Core System Architecture Part 143 — Anonymous Network Extension Update Orchestration, Dependency-Aware Rollouts, State Migration, Rollback, Hot Upgrade, Compatibility Gates & Privacy-Preserving Extension Evolution Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 143  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 99, 107–109, 122–142

**Primary purpose:** define SIAR's extension evolution architecture for update candidates, dependency-aware rollout planning, compatibility gates, state/schema migration, hot/cold upgrade, staged deployment, rollback, crash recovery, revocation-driven emergency replacement, multi-device coordination, certification binding, offline/air-gapped update flow, and privacy-preserving lifecycle control.

---

# 1. Purpose

Extension updates are not merely package replacement.

An update may change:

```text
code
dependencies
permissions
data schema
background jobs
UI contributions
network destinations
compatibility
runtime ABI
```

A safe update system must answer:

```text
Is the new package authentic?
Is it compatible with the current host?
Did requested capabilities change?
Will local extension data migrate safely?
Can the upgrade roll back?
Will older devices still interoperate?
What happens if the new extension crash-loops?
How does an urgent security replacement propagate?
```

The governing principle is:

> **SIAR extension evolution must be staged, dependency-aware, state-aware, compatibility-gated, rollback-capable where semantics permit, and unable to silently widen authority or destroy user data during upgrade.**

---

# 2. Architectural Position

```text
                 NEW EXTENSION PACKAGE
                          │
                          ▼
                   UPDATE CANDIDATE
                          │
          ┌───────────────┼────────────────┐
          │               │                │
     COMPATIBILITY     DEPENDENCIES     POLICY DIFF
          │               │                │
          └───────────────┼────────────────┘
                          ▼
                   UPDATE PLAN
                          │
             ┌────────────┼────────────┐
             │            │            │
          PREFLIGHT    MIGRATION     ROLLOUT
             │            │            │
             └────────────┼────────────┘
                          ▼
                   NEW RUNTIME
                          │
                          ▼
                VERIFY / COMMIT / ROLLBACK
```

---

# 3. Core Separation

Keep distinct:

```text
package availability
update eligibility
update approval
migration readiness
runtime activation
rollout authorization
post-activation verification
```

---

# 4. Non-Goals

Part 143 does not create:

```text
automatic production updates with no gates
silent permission expansion
irreversible migration without checkpoint
global all-device simultaneous cutover
"latest always wins" behavior
```

---

# 5. Update Candidate Identity

```rust
pub struct ExtensionUpdateCandidateId(pub [u8; 16]);
```

---

# 6. Update Candidate

```rust
pub struct ExtensionUpdateCandidate {
    pub id: ExtensionUpdateCandidateId,
    pub extension: ExtensionId,
    pub from_version: ExtensionVersion,
    pub to_version: ExtensionVersion,
    pub package: ExtensionPackageId,
    pub lockfile_digest: Digest,
}
```

---

# 7. Hard Rule

Update candidate is immutable after qualification begins.

---

# 8. Update Class

```rust
pub enum ExtensionUpdateClass {
    Patch,
    Minor,
    Major,
    Security,
    EmergencySecurity,
    Compatibility,
    DataMigration,
}
```

---

# 9. Hard Rule

Update class is descriptive, not sufficient authorization.

---

# 10. Update State

```rust
pub enum ExtensionUpdateState {
    Discovered,
    Evaluating,
    AwaitingApproval,
    Ready,
    RollingOut,
    Verifying,
    Active,
    RolledBack,
    Failed,
    Suspended,
    Superseded,
}
```

---

# 11. No Discovered→Active Direct

Hard rule.

---

# 12. Update Source

```rust
pub enum ExtensionUpdateSource {
    Marketplace,
    EnterpriseRegistry,
    OfflineBundle,
    SecurityReplacement,
}
```

---

# 13. Hard Rule

Source must match package/source policy.

---

# 14. Package Verification

Before update planning:

```text
signature
digest
publisher identity
revocation state
provenance
SBOM
dependency lockfile
```

---

# 15. Hard Rule

Package verification precedes migration planning.

---

# 16. Update Eligibility

```rust
pub enum ExtensionUpdateEligibility {
    Eligible,
    EligibleWithApproval,
    Blocked,
    Unsupported,
}
```

---

# 17. Eligibility Inputs

```text
host version
SDK compatibility
host ABI
dependency graph
platform
policy
certification
```

---

# 18. Hard Rule

Eligibility is explicit per device/platform.

---

# 19. Compatibility Gate

```rust
pub struct ExtensionCompatibilityGate {
    pub host: CompatibilityDecision,
    pub sdk: CompatibilityDecision,
    pub abi: CompatibilityDecision,
    pub peer_extensions: Vec<CompatibilityDecision>,
}
```

---

# 20. Hard Rule

Any mandatory incompatibility blocks activation.

---

# 21. Unknown Compatibility

Unknown is not compatible.

---

# 22. Hard Rule

Unknown cannot silently pass production gate.

---

# 23. Dependency Gate

Compare Part 142 lockfiles.

---

# 24. Dependency Diff

```rust
pub struct ExtensionDependencyUpdateDiff {
    pub added: Vec<LockedDependency>,
    pub removed: Vec<LockedDependency>,
    pub changed: Vec<DependencyVersionChange>,
}
```

---

# 25. Hard Rule

Dependency changes are reviewed as part of extension update.

---

# 26. High-Risk Dependency Change

Examples:

```text
new native library
new registry
new cryptographic dependency
new build script
new runtime peer dependency
```

---

# 27. Hard Rule

High-risk dependency change raises review level.

---

# 28. Permission Diff

Part 135.

---

# 29. Hard Rule

New or broader permission requires renewed approval before activation.

---

# 30. Data-Flow Diff

Part 136.

---

# 31. Hard Rule

New external sink/retention behavior requires review.

---

# 32. Background-Job Diff

Part 138.

---

# 33. Hard Rule

New periodic job/frequency increase may require consent/policy review.

---

# 34. UI Contribution Diff

Part 140.

---

# 35. Hard Rule

New privileged UI action cannot silently appear without corresponding permission/policy.

---

# 36. Update Diff Summary

```rust
pub struct ExtensionUpdateDiff {
    pub permissions: PermissionDiff,
    pub data_flows: ExtensionDataFlowDiff,
    pub dependencies: ExtensionDependencyUpdateDiff,
    pub background_jobs: ExtensionBackgroundJobDiff,
    pub ui: ExtensionUiDiff,
}
```

---

# 37. Hard Rule

Update approval sees material diffs, not only version number.

---

# 38. Update Approval

```rust
pub enum ExtensionUpdateApproval {
    NotRequired,
    UserRequired,
    AdminRequired,
    SecurityRequired,
    MultiAuthorityRequired,
}
```

---

# 39. Hard Rule

Approval authority depends on changed behavior/scope.

---

# 40. Automatic Update Eligibility

Only if all true:

```text
signature valid
compatible
no capability expansion
no data-flow expansion
no high-risk dependency change
migration safe
policy allows auto-update
```

---

# 41. Hard Rule

Auto-update cannot expand authority.

---

# 42. Update Channel

```rust
pub enum ExtensionReleaseChannel {
    Stable,
    Beta,
    Development,
    Security,
}
```

---

# 43. Stable Default

Hard rule.

---

# 44. Channel Switching

Explicit user/admin action.

---

# 45. Hard Rule

Security channel can deliver urgent fixed package without opting user into Beta.

---

# 46. Version Pinning

User/admin may pin allowed version/range.

---

# 47. Hard Rule

Pin cannot override package revocation/security block.

---

# 48. Emergency Security Update

May override ordinary pin if hard security policy requires.

---

# 49. Hard Rule

Emergency replacement still requires signature/provenance verification.

---

# 50. Update Plan

```rust
pub struct ExtensionUpdatePlan {
    pub candidate: ExtensionUpdateCandidateId,
    pub preflight: Vec<UpdatePreflightCheck>,
    pub migration: Option<ExtensionMigrationPlan>,
    pub rollout: ExtensionRolloutPlan,
    pub rollback: ExtensionRollbackPlan,
}
```

---

# 51. Preflight Check

```rust
pub enum UpdatePreflightCheck {
    PackageVerified,
    CompatibilityPassed,
    DependencyPolicyPassed,
    CapabilityDiffApproved,
    DataFlowDiffApproved,
    StorageCapacitySufficient,
    MigrationDryRunPassed,
    BackupCheckpointReady,
}
```

---

# 52. Hard Rule

Required preflight gates cannot be skipped silently.

---

# 53. Preflight Result

```rust
pub enum UpdateGateResult {
    Pass,
    Fail,
    Unknown,
    NotApplicable,
}
```

---

# 54. Hard Rule

Unknown does not count as Pass.

---

# 55. Storage Capacity Check

Update must estimate:

```text
new package size
migration workspace
rollback checkpoint
temporary duplicate runtime
```

---

# 56. Hard Rule

Insufficient capacity blocks unless safe strategy exists.

---

# 57. Update Checkpoint

```rust
pub struct ExtensionUpdateCheckpoint {
    pub extension: ExtensionId,
    pub version: ExtensionVersion,
    pub state_schema: SchemaVersion,
    pub data_snapshot_ref: Option<SnapshotRef>,
    pub created_at: Timestamp,
}
```

---

# 58. Hard Rule

Checkpoint excludes live capability tokens/secrets.

---

# 59. State Migration

```rust
pub struct ExtensionMigrationPlan {
    pub from_schema: SchemaVersion,
    pub to_schema: SchemaVersion,
    pub reversible: bool,
    pub strategy: ExtensionMigrationStrategy,
}
```

---

# 60. Migration Strategy

```rust
pub enum ExtensionMigrationStrategy {
    InPlaceTransactional,
    CopyTransformSwap,
    DualReadWrite,
    ForwardOnlyCheckpointed,
}
```

---

# 61. Hard Rule

Migration strategy must be declared.

---

# 62. In-Place Transactional

Use when storage engine supports atomic migration.

---

# 63. Hard Rule

No partially committed schema.

---

# 64. Copy-Transform-Swap

Preferred for large/complex data when storage permits.

---

# 65. Hard Rule

Old state remains untouched until verification.

---

# 66. Dual-Read-Write

Useful for staged transition.

---

# 67. Hard Rule

Dual format period bounded.

---

# 68. Forward-Only Migration

Highest risk.

---

# 69. Hard Rule

Requires checkpoint/export/recovery plan.

---

# 70. Migration Dry Run

Run against synthetic/representative state where possible.

---

# 71. Hard Rule

Migration dry run does not modify production data.

---

# 72. Migration Validator

```rust
pub trait ExtensionMigrationValidator {
    fn validate(
        &self,
        before: &ExtensionStateSummary,
        after: &ExtensionStateSummary,
    ) -> Result<MigrationValidationResult, ExtensionUpdateError>;
}
```

---

# 73. Migration Validation

May verify:

```text
object counts
required fields
checksums
invariants
schema consistency
```

---

# 74. Hard Rule

Successful execution is not sufficient; state must verify.

---

# 75. Derived Data

Indexes/cache/embeddings may be rebuilt instead of migrated.

---

# 76. Hard Rule

Rebuildability explicit.

---

# 77. User Data

Durable user-owned data receives highest preservation priority.

---

# 78. Hard Rule

Update failure cannot silently discard user data.

---

# 79. Capability State Migration

Runtime tokens never migrate.

---

# 80. Hard Rule

Permission intent may persist, effective grants revalidated after activation.

---

# 81. Background Job Migration

Job definitions migrate/version.

---

# 82. Hard Rule

New jobs do not run before update activation and policy approval.

---

# 83. Notification/Action State

Outstanding action tokens may be invalidated on major update if semantics changed.

---

# 84. Hard Rule

Stale notification action cannot target incompatible runtime behavior.

---

# 85. UI State Migration

Only durable safe preferences migrate.

---

# 86. Hard Rule

Transient UI state does not block update.

---

# 87. Hot Upgrade

Hot upgrade means replacing extension runtime without restarting SIAR host.

---

# 88. Hot Upgrade Eligibility

```rust
pub struct HotUpgradeEligibility {
    pub state_transfer_supported: bool,
    pub host_abi_compatible: bool,
    pub migration_safe_online: bool,
    pub inflight_work_quiesceable: bool,
}
```

---

# 89. Hard Rule

Hot upgrade is optimization, not requirement.

---

# 90. Cold Upgrade

Stop runtime, migrate, start new version.

---

# 91. Hard Rule

Cold upgrade preferred when correctness simpler.

---

# 92. Quiescence

Before hot/cold activation:

```text
stop new host calls
stop new background jobs
drain bounded in-flight work
checkpoint state
```

---

# 93. Hard Rule

Update does not begin while unbounded in-flight work persists.

---

# 94. Quiescence Deadline

Bounded.

---

# 95. Hard Rule

After deadline, cancel/abort according to operation semantics.

---

# 96. In-Flight Durable Call

Use commit receipts/idempotency from Part 137/138.

---

# 97. Hard Rule

Update cannot duplicate uncertain side effects.

---

# 98. Stream Handling

Long-lived streams are:

```text
drained
checkpointed
or terminated
```

explicitly.

---

# 99. Hard Rule

No silent stream handoff without compatibility contract.

---

# 100. Runtime Replacement

```text
verify candidate
quiesce old runtime
prepare state
start new runtime
perform handshake
run health checks
switch routing
retire old runtime
```

---

# 101. Hard Rule

Traffic switches only after new runtime readiness.

---

# 102. Shadow Start

Optional:

```text
start new runtime
no privileged writes
validate startup/health
```

---

# 103. Hard Rule

Shadow runtime cannot perform side effects.

---

# 104. Hot State Transfer

If supported:

```rust
pub struct ExtensionRuntimeStateTransfer {
    pub schema: SchemaVersion,
    pub payload_ref: OpaqueStateTransferRef,
}
```

---

# 105. Hard Rule

State transfer excludes secrets/runtime tokens unless secure broker rebinds them.

---

# 106. New Runtime Identity

Always fresh.

---

# 107. Hard Rule

Old runtime ID/capability tokens invalid after cutover.

---

# 108. Activation Epoch

```rust
pub struct ExtensionActivationEpoch(pub u64);
```

---

# 109. Hard Rule

Runtime requests from prior activation epoch rejected after switch.

---

# 110. Rollout Scope

```rust
pub enum ExtensionRolloutScope {
    ThisDevice,
    SelectedDevices(BTreeSet<DeviceId>),
    TenantSubset(TenantRolloutSelector),
    AllEligible,
}
```

---

# 111. Hard Rule

Rollout scope must be explicit.

---

# 112. Device-Local First

Personal/local extension updates can apply device by device.

---

# 113. Hard Rule

No global atomic all-device assumption.

---

# 114. Multi-Device Coordination

Part 141.

---

# 115. Hard Rule

Each device locally re-verifies package/compatibility/policy.

---

# 116. Rollout Stage

```rust
pub enum ExtensionRolloutStage {
    Internal,
    Canary,
    Limited,
    Broad,
    Complete,
}
```

---

# 117. Hard Rule

Stage describes deployment breadth, not readiness/certification quality.

---

# 118. Canary Selection

Must avoid sensitive behavioral targeting.

---

# 119. Good Criteria

```text
explicit testers
device platform
tenant opt-in
admin-selected cohort
```

---

# 120. Forbidden Criteria

```text
private message behavior
sensitive inferred traits
engagement profile
```

---

# 121. Hard Rule

Rollout cohort cannot become surveillance segmentation.

---

# 122. Rollout Plan

```rust
pub struct ExtensionRolloutPlan {
    pub stages: Vec<ExtensionRolloutStagePlan>,
    pub pause_conditions: Vec<ExtensionRolloutPauseCondition>,
}
```

---

# 123. Stage Plan

```rust
pub struct ExtensionRolloutStagePlan {
    pub stage: ExtensionRolloutStage,
    pub scope: ExtensionRolloutScope,
    pub observation_window: Duration,
}
```

---

# 124. Hard Rule

Observation window finite and explicit.

---

# 125. Rollout Evidence

Use aggregate technical signals.

---

# 126. Allowed Signals

```text
startup failure rate
crash rate
migration failure rate
compatibility errors
resource regression
```

---

# 127. Hard Rule

No private user behavior required.

---

# 128. Pause Condition

```rust
pub enum ExtensionRolloutPauseCondition {
    CrashRateAbove(Threshold),
    MigrationFailuresAbove(Threshold),
    CompatibilityFailureDetected,
    SecurityIncident,
    PrivacyIncident,
    ManualPause,
}
```

---

# 129. Hard Rule

Security/privacy incident pauses immediately.

---

# 130. No Outcome Prediction

Rollout controller reacts to observed technical evidence.

---

# 131. Rollout Authorization

Separate from candidate qualification.

---

# 132. Hard Rule

Qualified package is merely eligible for rollout.

---

# 133. Tenant Rollout

Enterprise admins may choose rollout windows.

---

# 134. Hard Rule

Tenant schedule cannot bypass platform security block.

---

# 135. Maintenance Window

Optional.

---

# 136. Hard Rule

Urgent security update may override ordinary convenience window according to policy.

---

# 137. Update Deferral

User may defer ordinary update for bounded period.

---

# 138. Hard Rule

Deferral limit explicit for security-sensitive versions.

---

# 139. Mandatory Update

Reserved for:

```text
revoked package
hard incompatibility
critical security floor
```

---

# 140. Hard Rule

Mandatory update must still preserve user data/recovery options.

---

# 141. Compatibility Gates

Part 129 includes:

```text
host ABI
SDK/API
protocol versions
peer extension contracts
```

---

# 142. Hard Rule

Gate evaluated immediately before activation too, not only discovery time.

---

# 143. TOCTOU Protection

Compatibility/revocation/policy may change while update queued.

---

# 144. Hard Rule

Revalidate package, policy, compatibility, revocation before commit.

---

# 145. Dependency-Aware Rollout

If multiple extensions share platform-managed module:

```text
update module compatibility first
stage dependents
verify graph
```

---

# 146. Hard Rule

Dependency rollout ordering explicit.

---

# 147. Dependency Order

Topological plan where applicable.

---

# 148. Hard Rule

Cycles rejected or broken through explicit compatibility bridge.

---

# 149. Peer Extension Upgrade

May require coordinated version overlap.

---

# 150. Compatibility Window

```rust
pub struct PeerCompatibilityWindow {
    pub old: VersionRange<ExtensionVersion>,
    pub new: VersionRange<ExtensionVersion>,
}
```

---

# 151. Hard Rule

Coordinated upgrade must provide overlap or explicit outage/dependency block.

---

# 152. No Surprise Peer Breakage

Hard rule.

---

# 153. Update Bridge

Optional adapter to support old/new protocol temporarily.

---

# 154. Hard Rule

Bridge lifetime bounded/deprecated.

---

# 155. State Migration Ordering

Dependency state migration can precede root extension only if independent and rollback-safe.

---

# 156. Hard Rule

No hidden cascade migration.

---

# 157. Rollback Plan

```rust
pub struct ExtensionRollbackPlan {
    pub supported: bool,
    pub target_version: Option<ExtensionVersion>,
    pub data_strategy: RollbackDataStrategy,
}
```

---

# 158. Data Strategy

```rust
pub enum RollbackDataStrategy {
    NoDataChange,
    RestoreCheckpoint,
    ReverseMigration,
    ForwardRecoveryOnly,
}
```

---

# 159. Hard Rule

Rollback support is explicit.

---

# 160. No Fake Rollback

If forward-only schema migration exists, state must say rollback unsupported.

---

# 161. Hard Rule

UI/ops never present rollback button if impossible.

---

# 162. Rollback Trigger

```rust
pub enum ExtensionRollbackTrigger {
    StartupFailure,
    CrashLoop,
    MigrationValidationFailure,
    CompatibilityRegression,
    SecurityIssue,
    ManualOperatorDecision,
}
```

---

# 163. Hard Rule

Rollback trigger does not erase evidence.

---

# 164. Crash Loop

If new runtime exceeds restart budget:

```text
quarantine new version
attempt rollback if safe
```

---

# 165. Hard Rule

Do not repeatedly bounce between versions indefinitely.

---

# 166. Rollback Attempt Budget

Bounded.

---

# 167. Hard Rule

Repeated rollback failure leads to quarantine, not infinite loop.

---

# 168. Quarantine State

```rust
pub enum ExtensionUpdateQuarantineReason {
    StartupCrashLoop,
    MigrationFailure,
    SignatureIssue,
    CompatibilityIssue,
    ResourceRegression,
    SecurityIssue,
}
```

---

# 169. Hard Rule

Quarantine preserves data.

---

# 170. Safe Mode

Host may disable extension code while leaving:

```text
data export
remove
update
diagnostics
```

available through platform-owned UI.

---

# 171. Hard Rule

Revoked/quarantined extension does not render its own recovery UI.

---

# 172. Verification Phase

After activation, verify:

```text
runtime startup
state integrity
required permissions
background job registration
UI contribution load
dependency health
```

---

# 173. Hard Rule

Activation is provisional until verification passes.

---

# 174. Verification Window

Finite and explicit.

---

# 175. Hard Rule

A successful process launch is not enough.

---

# 176. Update Success

```rust
pub struct ExtensionUpdateSuccess {
    pub extension: ExtensionId,
    pub version: ExtensionVersion,
    pub activation_epoch: ExtensionActivationEpoch,
    pub verified_at: Timestamp,
}
```

---

# 177. Hard Rule

Success recorded only after verification.

---

# 178. Post-Update Cleanup

After confidence window:

```text
remove old package
remove rollback checkpoint if policy permits
compact migration artifacts
```

---

# 179. Hard Rule

Do not delete rollback data before rollback window closes.

---

# 180. Rollback Window

Explicit.

---

# 181. Hard Rule

Window may differ by migration reversibility.

---

# 182. Old Package Retention

Content-addressed package can remain cached briefly for rollback.

---

# 183. Hard Rule

Revoked old package is not reactivated even if cached.

---

# 184. Update Audit

Record high-value events:

```text
candidate approved
migration started/completed
activation switched
rollback
quarantine
security override
```

---

# 185. Hard Rule

No audit of every package-download byte.

---

# 186. Update Evidence

Store in assurance archive:

```text
package digest
lockfile
SBOM
compatibility results
migration evidence
rollout evidence
rollback evidence
```

---

# 187. Hard Rule

Evidence refers to immutable artifacts.

---

# 188. Certification Binding

If certified extension changes:

```text
package digest
lockfile
capabilities
data flow
ABI
```

relevant certification evidence must be re-evaluated.

---

# 189. Hard Rule

Certification does not automatically carry to materially changed package.

---

# 190. Security Update Certification

Can use expedited path but not skip hard security/privacy evidence.

---

# 191. Hard Rule

Emergency does not mean unverified.

---

# 192. Marketplace Integration

Part 133 provides update package/revocation metadata.

---

# 193. Hard Rule

Marketplace publication alone is not update authorization.

---

# 194. Dependency Integration

Part 142 exact locked graph participates in candidate identity/evidence.

---

# 195. Hard Rule

Dependency graph change invalidates stale update evidence.

---

# 196. Permission Integration

Part 135 diff/re-consent before activation.

---

# 197. Data Governance Integration

Part 136 migration/retention/deletion behavior validated.

---

# 198. Runtime Integration

Part 134 handles runtime cutover/quarantine.

---

# 199. IPC Integration

Part 137 handles quiescence, stream/call shutdown, activation epoch.

---

# 200. Background Integration

Part 138 jobs paused/migrated/re-registered.

---

# 201. Notification Integration

Part 139 may surface update/restart/re-consent only when needed.

---

# 202. UI Integration

Part 140 contribution registry changes atomically at activation.

---

# 203. State Sync Integration

Part 141 update intent/version preference may sync, but each device executes local verification.

---

# 204. Hard Rule

No remote device marks another device updated by assertion alone.

---

# 205. Multi-Device Version Skew

Expected during staged rollout.

---

# 206. Hard Rule

Extension data/protocol state must tolerate supported version skew.

---

# 207. Minimum Interoperability Window

Document supported overlap.

---

# 208. Hard Rule

If overlap impossible, coordinated downtime/block is explicit.

---

# 209. Offline Device

May update later.

---

# 210. Hard Rule

Offline old version can be blocked from incompatible shared state mutation if necessary.

---

# 211. Stale Device Policy

```rust
pub enum ExtensionVersionSkewPolicy {
    Allowed,
    ReadOnly,
    UpdateRequired,
    Blocked,
}
```

---

# 212. Hard Rule

Policy derived from compatibility, not arbitrary recency.

---

# 213. Air-Gapped Update

Use signed offline bundle.

---

# 214. Offline Update Bundle

```rust
pub struct ExtensionOfflineUpdateBundle {
    pub package: ExtensionPackageId,
    pub lockfile_digest: Digest,
    pub sbom_digest: Digest,
    pub provenance_digest: Digest,
    pub compatibility_snapshot: Digest,
    pub migration_manifest: Option<Digest>,
}
```

---

# 215. Hard Rule

All required artifacts included.

---

# 216. Offline Freshness

Security/revocation snapshot freshness explicit.

---

# 217. Hard Rule

Stale policy cannot permissively approve risky update.

---

# 218. Offline Preflight

Can verify:

```text
signature
digest
compatibility snapshot
dependency graph
migration manifest
```

---

# 219. Hard Rule

No hidden online dependency.

---

# 220. Update Delivery

Package download can use:

```text
marketplace CDN
enterprise mirror
peer-assisted cache if policy permits
offline media
```

---

# 221. Hard Rule

Delivery channel is not trust authority.

---

# 222. Content Addressing

Digest verified after download.

---

# 223. Hard Rule

Any byte-level mismatch rejects artifact.

---

# 224. Delta Update

Optional optimization.

---

# 225. Hard Rule

Delta result must reconstruct exact signed target digest.

---

# 226. Base Package

Target delta explicitly binds base digest.

---

# 227. Hard Rule

Wrong base fails; no fuzzy patching.

---

# 228. Download Resume

Safe via chunk hash/offset.

---

# 229. Hard Rule

Partial package never treated as verified.

---

# 230. Update Bandwidth Policy

Respect:

```text
metered connection
battery
tenant network
background quota
```

---

# 231. Hard Rule

Security urgency does not justify silent huge cellular transfer if policy requires consent, unless hard platform policy explicitly mandates.

---

# 232. Scheduling

Part 138 durable job can handle update check/download.

---

# 233. Hard Rule

Activation itself remains governed state transition.

---

# 234. Update Notification

User-visible only when needed:

```text
approval
re-consent
restart
failure
security urgency
```

---

# 235. Hard Rule

No attention spam for every background update check.

---

# 236. User Preference

Possible:

```rust
pub enum ExtensionUpdatePreference {
    AutomaticSafe,
    NotifyBeforeUpdate,
    Manual,
}
```

---

# 237. Hard Rule

Preference cannot override hard security/revocation policy.

---

# 238. Tenant Update Policy

```rust
pub struct TenantExtensionUpdatePolicy {
    pub allowed_channels: BTreeSet<ExtensionReleaseChannel>,
    pub maintenance_window: Option<MaintenanceWindow>,
    pub deferral_limit: Option<Duration>,
}
```

---

# 239. Hard Rule

Tenant policy can restrict ordinary rollout.

---

# 240. Device Policy

May restrict updates on battery/network/storage conditions.

---

# 241. Hard Rule

Device policy cannot reactivate revoked version.

---

# 242. Privacy Mode

Update checks/downloads should avoid exposing anonymous communication identity.

---

# 243. Hard Rule

Marketplace/update identity remains separate from anonymous messaging identity.

---

# 244. Update Telemetry

Allowed:

```text
candidate version
device platform
update stage
success/failure class
migration result class
crash rate aggregate
```

---

# 245. Forbidden:

```text
private message content
user behavior
social graph
behavioral rollout cohorts
```

---

# 246. Hard Rule

No per-user extension-update engagement profile.

---

# 247. Rollout Metrics

Aggregate by:

```text
platform
version
stage
error class
```

---

# 248. Hard Rule

Do not require account identity for technical aggregate if avoidable.

---

# 249. Privacy-Preserving Canary

Canary identity may be ephemeral/cohort-based.

---

# 250. Hard Rule

No stable user tracking solely for rollout.

---

# 251. Update Orchestrator

```rust
pub trait ExtensionUpdateOrchestrator {
    async fn evaluate(
        &self,
        candidate: ExtensionUpdateCandidate,
    ) -> Result<ExtensionUpdatePlan, ExtensionUpdateError>;

    async fn apply(
        &self,
        plan: ExtensionUpdatePlan,
    ) -> Result<ExtensionUpdateResult, ExtensionUpdateError>;
}
```

---

# 252. Migration Service

```rust
pub trait ExtensionMigrationService {
    async fn prepare(
        &self,
        plan: &ExtensionMigrationPlan,
    ) -> Result<ExtensionUpdateCheckpoint, ExtensionUpdateError>;

    async fn migrate(
        &self,
        plan: &ExtensionMigrationPlan,
    ) -> Result<MigrationValidationResult, ExtensionUpdateError>;
}
```

---

# 253. Rollback Service

```rust
pub trait ExtensionRollbackService {
    async fn rollback(
        &self,
        extension: ExtensionId,
        plan: ExtensionRollbackPlan,
    ) -> Result<(), ExtensionUpdateError>;
}
```

---

# 254. Rollout Service

```rust
pub trait ExtensionRolloutService {
    async fn advance(
        &self,
        candidate: ExtensionUpdateCandidateId,
        stage: ExtensionRolloutStage,
    ) -> Result<RolloutReceipt, ExtensionUpdateError>;
}
```

---

# 255. Compatibility Revalidation Service

```rust
pub trait ExtensionUpdateCompatibilityService {
    fn revalidate_before_activation(
        &self,
        candidate: ExtensionUpdateCandidateId,
    ) -> Result<ExtensionCompatibilityGate, ExtensionUpdateError>;
}
```

---

# 256. Error Taxonomy

```rust
pub enum ExtensionUpdateError {
    CandidateUnknown,
    PackageInvalid,
    PackageRevoked,
    CompatibilityFailed,
    DependencyPolicyFailed,
    PermissionApprovalRequired,
    DataFlowApprovalRequired,
    MigrationNotReady,
    MigrationFailed,
    RollbackUnavailable,
    RollbackFailed,
    RuntimeQuarantined,
    InsufficientStorage,
    PolicyStale,
    RolloutPaused,
    Unauthorized,
    Internal,
}
```

---

# 257. Update SLOs

Examples:

```text
safe candidate evaluation within target
security revocation blocks activation within target
rollback executes within target when supported
migration verification finishes within declared window
```

---

# 258. Security SLO

```text
0 revoked package activated
0 update silently expands capability
0 incompatible ABI activated
0 unverified migration committed
```

---

# 259. Privacy SLO

```text
0 rollout cohort based on private user behavior
0 anonymous messaging identity reused for update tracking
0 update telemetry contains private extension data
```

---

# 260. Failure Modes

```text
migration corruption
dependency incompatibility
crash-loop after activation
partial multi-device rollout
stale compatibility decision
```

---

# 261. Migration Corruption

Stop, restore checkpoint/forward-recover, quarantine.

---

# 262. Dependency Incompatibility

Block activation before runtime switch.

---

# 263. Crash Loop

Rollback/quarantine according to plan.

---

# 264. Partial Multi-Device Rollout

Supported through version-skew policy.

---

# 265. Stale Compatibility

Revalidate at activation.

---

# 266. Testing

Need extension-update testkit.

Required scenarios:

```text
safe patch update
permission-expanding update
forward-only migration
hot upgrade
crash-loop rollback
offline bundle
```

---

# 267. Package Test

Tampered package rejected.

---

# 268. Dependency Test

Changed locked digest invalidates prior qualification evidence.

---

# 269. Permission Diff Test

New scope blocks activation until approved.

---

# 270. Data Flow Test

New external sink blocks update until review.

---

# 271. Migration Test

Corrupt transformed state fails validation before commit.

---

# 272. Rollback Test

Reversible migration returns exact prior logical state.

---

# 273. Forward-Only Test

UI/ops does not expose unsupported rollback.

---

# 274. Hot Upgrade Test

Old runtime quiesces; new runtime receives fresh activation identity.

---

# 275. Stream Test

Long-running stream terminates safely during upgrade.

---

# 276. Idempotency Test

Uncertain side effect not duplicated during upgrade retry.

---

# 277. Crash Loop Test

New version quarantined after restart budget.

---

# 278. Multi-Device Test

Different versions coexist under allowed compatibility window.

---

# 279. Stale Device Test

Old incompatible device becomes read-only/update-required.

---

# 280. Offline Test

Offline bundle verifies without network fallback.

---

# 281. Revocation Test

Emergency update cannot reactivate revoked old package during rollback.

---

# 282. Privacy Test

Canary cohort does not use private behavioral signal.

---

# 283. Fuzzing

Fuzz:

```text
update manifests
migration manifests
rollout plans
rollback plans
compatibility snapshots
```

---

# 284. Property Tests

Properties:

```text
candidate can never activate with failed mandatory compatibility gate
scope-expanding update can never activate without required approval
rollback can never reactivate revoked target package
old runtime token can never authorize after activation epoch changes
```

---

# 285. Formal Verification Targets

Strong candidates:

```text
update lifecycle
activation epoch
migration commit/rollback
staged rollout pause/advance
```

---

# 286. Kani Candidate

compatibility/approval/activation invariants.

---

# 287. TLA+ Candidate

```text
discover → evaluate → approve → migrate → activate → verify → commit/rollback
```

---

# 288. Loom Candidate

Concurrent:

```text
runtime quiescence
permission revoke
update activation
rollback request
```

---

# 289. Performance

Update orchestration is control-plane work.

---

# 290. Hard Rule

No update checks on message-send/UI hot path.

---

# 291. Package Download

Asynchronous background job.

---

# 292. Migration Throughput

Bounded and progress-reportable.

---

# 293. Hard Rule

Migration cannot monopolize IO/CPU over core messaging.

---

# 294. Resource Scheduling

Use Part 101/113 principles:

```text
core reliability
security/privacy
interactive work
extension migration
bulk update work
```

---

# 295. Hard Rule

Update/migration yields to core-critical workload.

---

# 296. Storage

Separate:

```text
update candidates
update plans
approval records
migration checkpoints
rollout state
rollback metadata
activation epochs
```

---

# 297. Secrets separate.

---

# 298. Hard Rule

No private user content copied into update control DB.

---

# 299. Partitioning

By:

```text
extension
candidate
device
tenant rollout
```

No behavioral user cohort partition.

---

# 300. Crate Layout

Recommended:

```text
crates/
├── siar-extension-update-core/
├── siar-extension-update-evaluator/
├── siar-extension-update-compat/
├── siar-extension-migration/
├── siar-extension-rollout/
├── siar-extension-hot-upgrade/
├── siar-extension-rollback/
├── siar-extension-update-policy/
├── siar-extension-update-observability/
└── siar-extension-update-testkit/
```

---

# 301. `siar-extension-update-core`

Owns:

```text
ExtensionUpdateCandidateId
ExtensionUpdateState
ExtensionUpdateClass
ExtensionUpdateError
```

---

# 302. `siar-extension-update-evaluator`

Package/diff/gate/update-plan construction.

---

# 303. `siar-extension-update-compat`

Host/SDK/ABI/peer compatibility gates.

---

# 304. `siar-extension-migration`

Dry-run/checkpoint/transform/verify.

---

# 305. `siar-extension-rollout`

Stage planning/pause/advance/device/tenant coordination.

---

# 306. `siar-extension-hot-upgrade`

Quiescence/shadow start/runtime cutover/activation epoch.

---

# 307. `siar-extension-rollback`

Rollback eligibility/checkpoint restore/reverse migration/quarantine.

---

# 308. `siar-extension-update-policy`

Auto-update, security override, tenant/user/device policy.

---

# 309. `siar-extension-update-observability`

Aggregate technical rollout health only.

---

# 310. `siar-extension-update-testkit`

migration/rollback/hot-update/privacy/formal tests.

---

# 311. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Package discovery, update eligibility, user/admin approval, migration readiness, rollout authorization, runtime activation, post-activation verification, and final update success are distinct states and cannot be collapsed into one “update available/install” transition.
2. Every candidate binds an immutable package digest, exact dependency lockfile, provenance, SBOM, compatibility baseline, and migration metadata; any material artifact/dependency change creates a new candidate and invalidates stale evidence.
3. Capability, data-flow, network-origin, background-job, UI, dependency, and retention expansions are computed as explicit diffs and can never become active through automatic update without required re-consent/review/certification.
4. State migration is versioned, strategy-declared, validated, and checkpointed where needed; durable user data can never be silently discarded, and rollback is advertised only when the chosen migration/data strategy can actually support it.
5. Hot upgrade is optional and permitted only when runtime quiescence, host ABI, state transfer, stream/call handling, and migration semantics are safe; correctness always outranks zero-downtime convenience.
6. Runtime cutover uses a fresh activation epoch/runtime identity, and all old sessions, capability tokens, streams, and host-call authority become invalid after activation; stale runtime requests cannot continue after switch.
7. Rollout breadth is staged and independently authorized; technical canary/limited rollout uses explicit tester/device/tenant criteria and cannot rely on private behavioral data, inferred sensitive traits, or persistent user tracking.
8. Compatibility, package revocation, permission state, dependency policy, tenant/device policy, and security state are revalidated immediately before activation to prevent queued-update TOCTOU races.
9. Package revocation, critical security failure, migration corruption, startup crash-loop, or incompatible dependency state can pause, quarantine, or roll back the extension without deleting user data or reactivating a revoked package.
10. Multi-device extension evolution tolerates explicit supported version skew; each device independently verifies packages and materializes local effective state, and remote update assertions can never substitute for local package/security verification.
11. Update telemetry and rollout evidence are aggregate and technical—version, platform, migration/crash/error class—and cannot become per-user behavioral cohorts, private-content analytics, anonymous-identity correlation, or extension engagement profiling.
12. Extension evolution integrates with marketplace distribution, dependency governance, compatibility registry, permissions, data governance, runtime sandboxing, IPC, background jobs, notifications, UI, state sync, certification archives, vulnerability management, release governance, backup/restore, and audit without creating a side channel around SIAR's security, privacy, anonymity, local-first, or tenant-isolation guarantees.
```

---

# 312. Initial Production Scope

Implement first:

```text
typed update candidate/state machine
package/digest/provenance verification
compatibility gate
dependency lockfile diff
permission/data-flow/background/UI diffs
auto-update eligibility rules
preflight gate matrix
storage-capacity check
checkpoint model
transactional/copy-transform migration
migration validation
cold upgrade
runtime quiescence
fresh activation epoch
staged rollout
technical pause conditions
rollback eligibility
crash-loop quarantine
multi-device version-skew policy
offline signed update bundles
security override path
privacy-safe rollout metrics
extension-update testkit
```

Then add:

```text
hot upgrade
dual-read/write migration
peer-extension coordinated upgrades
delta package delivery
adaptive staged rollout
formal migration/rollback proofs
multi-device coordinated migration epochs
```

---

# 313. Definition of Done

Part 143 is complete when:

- update candidate identity is immutable;
- compatibility/dependency/policy diffs are explicit;
- scope expansion cannot auto-activate;
- migration strategy/reversibility are declared;
- durable checkpoints exist where required;
- rollback is truthful and bounded;
- activation uses fresh runtime epoch;
- staged rollout can pause on technical/security/privacy failure;
- revoked package cannot be reactivated;
- version skew is explicitly handled;
- offline updates remain fully verifiable;
- rollout telemetry is privacy-safe;
- migration/rollback/hot-upgrade/privacy/fuzz/formal tests are specified.

---

# 314. Final Architecture

```text
                 SIGNED UPDATE PACKAGE
                          │
                          ▼
                   UPDATE CANDIDATE
                          │
             ┌────────────┼────────────┐
             │            │            │
          COMPAT       DEP DIFF     POLICY DIFF
             │            │            │
             └────────────┼────────────┘
                          ▼
                     PREFLIGHT
                          │
                          ▼
                 CHECKPOINT / MIGRATE
                          │
                          ▼
                   STAGED ACTIVATE
                          │
                          ▼
                    VERIFY HEALTH
                       ┌──┴──┐
                       │     │
                    COMMIT  ROLLBACK
```

Extension-evolution safety model:

```text
immutable candidate
+
exact dependency graph
+
compatibility gates
+
permission/data-flow diffs
+
verified migration
+
fresh activation epoch
+
staged rollout
+
truthful rollback
+
privacy-safe evidence
```

not:

```text
download the newest package, replace files in place, hope schema migration works, and grant whatever new permissions the extension now asks for
```

---

# 315. Final Principle

Extension updates are trustworthy when the platform can answer **what changed, whether the new version is compatible, how state will migrate, what authority changed, how rollout is controlled, and how recovery works if activation fails**.

The correct model is:

```text
verify the exact candidate
+
diff dependencies and authority
+
gate compatibility
+
checkpoint state
+
migrate deliberately
+
quiesce old runtime
+
activate with a fresh epoch
+
verify before commit
+
roll back only when actually safe
+
never let an update silently widen trust
```

This architecture gives SIAR a privacy-preserving extension evolution foundation for dependency-aware updates, compatibility gating, state migration, hot/cold upgrade, staged rollout, rollback, quarantine, multi-device version skew, offline updates, and security-driven replacement while preserving the anonymity, local-first, least-authority, marketplace, runtime, permission, data-governance, state-sync, dependency-governance, and anti-surveillance guarantees established across Parts 34–142.
