# Core System Architecture Part 50 — Anonymous Network Reliability, Disaster Recovery, Partition Tolerance & Multi-Region Continuity Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 50  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–49  

**Primary purpose:** define the complete reliability and disaster-recovery architecture for SIAR's anonymous infrastructure, including failure-domain isolation, partition tolerance, durable client state, directory/topology continuity, mailbox recovery, provider failover, multi-region operation, backup boundaries, quorum behavior, recovery objectives, and privacy-preserving continuity procedures.

---

# 1. Purpose

Anonymous communication infrastructure has to survive:

```text
node crashes
operator failures
cloud-region outages
network partitions
directory unavailability
mailbox loss
provider compromise
regional censorship
database corruption
software rollout failures
natural disasters
```

Reliability work can itself create privacy failures.

Examples:

```text
fallback to direct transport
restore old mailbox secret
copy all user metadata into a central DR system
use globally stable recovery IDs
```

The governing principle is:

> **SIAR reliability mechanisms must preserve privacy invariants during failure. Availability is important, but continuity must never silently weaken anonymity.**

---

# 2. Architectural Position

```text
Clients / Outboxes
       │
       ▼
Anonymous Providers
       │
       ▼
Regional Failure Domains
       │
       ▼
Cross-Region Continuity
       │
       ▼
Recovery / Rebuild / Failover
```

---

# 3. Reliability Goals

The system should tolerate:

```text
single node loss
single operator loss
single region loss
directory mirror loss
partial provider loss
temporary network partition
```

without losing:

```text
durable user messages
security state
privacy policy
```

---

# 4. Privacy Goal During Failure

Hard rule:

```text
failure
≠
privacy downgrade
```

---

# 5. Failure Domain Model

```rust
pub enum FailureDomain {
    Process,
    Host,
    Rack,
    AvailabilityZone,
    Region,
    Operator,
    ASN,
    CloudProvider,
    DirectoryAuthority,
    ServiceProvider,
}
```

---

# 6. Why Failure Domains Matter

Two nodes are not independent if they share:

```text
host
operator
region
cloud
ASN
```

---

# 7. Reliability Diversity

Use multiple dimensions:

```text
operator
region
cloud
ASN
authority
```

---

# 8. Continuity Classes

```rust
pub enum ContinuityClass {
    BestEffort,
    Standard,
    HighAvailability,
    DisasterResilient,
}
```

---

# 9. Best Effort

Single-provider/simple deployment.

---

# 10. Standard

Redundant nodes in one region.

---

# 11. High Availability

Multi-zone, multi-operator.

---

# 12. Disaster Resilient

Multi-region + multi-operator + independent control-plane continuity.

---

# 13. Service Continuity Matrix

Services have different durability requirements.

---

# 14. Mix Nodes

Transient packet state.

Can be lost.

---

# 15. Mailboxes

Durable queued anonymous messages.

Must survive provider/node failure according to policy.

---

# 16. Directory

Signed topology/control state.

Must remain publishable and verifiable.

---

# 17. Bulk Storage

Encrypted object durability.

---

# 18. Realtime Relay

No durable session recovery required.

Call reconnects.

---

# 19. Bridge

Stateless or near-stateless.

---

# 20. Client Outbox

Authoritative unsent message state.

Critical.

---

# 21. Durability Classes

```rust
pub enum AnonymousDataDurability {
    Ephemeral,
    Reconstructable,
    Durable,
    SecurityCritical,
}
```

---

# 22. Ephemeral

Examples:

```text
mix delay queues
cover packets
realtime jitter state
```

---

# 23. Reconstructable

Examples:

```text
provider health cache
catalog cache
search indexes
```

---

# 24. Durable

Examples:

```text
client outbox
mailbox contents
bulk encrypted objects
```

---

# 25. Security Critical

Examples:

```text
identity keys
mailbox secrets
reply capabilities
revocation state
governance roots
```

---

# 26. Recovery Rule

Security-critical state must not be blindly replicated or restored.

---

# 27. Client Durable Outbox

The client's local outbox is the primary reliability anchor.

---

# 28. Persist Before Send

Hard rule.

---

# 29. Outbox State

```rust
pub enum DurableOutboxState {
    Queued,
    Submitted,
    Deposited,
    Delivered,
    Expired,
    Failed,
}
```

---

# 30. Crash Recovery

On restart:

```text
reload durable outbox
revalidate policy
re-resolve anonymous provider
resume
```

---

# 31. No Duplicate User Message

MessageId dedup required.

---

# 32. Transport Delivery ID

Separate from application MessageId.

---

# 33. Retry

New transport path may be used.

---

# 34. Privacy Constraint

Retry path must satisfy same anonymity class.

---

# 35. Network Partition

Client may be isolated from:

```text
Internet
mixnet
directory
mailbox provider
```

---

# 36. Partition Behavior

Queue locally.

---

# 37. No Direct Escape

Hard rule.

---

# 38. Local-Only Mode

During partition, local transports may continue only if policy allows.

---

# 39. Example

Same-LAN messaging may continue in:

```text
Standard
Private
```

but Maximum Anonymity may prohibit nearby transport.

---

# 40. Partition Policy

```rust
pub struct PartitionPolicy {
    pub allow_local_transport: bool,
    pub allow_dtn: bool,
    pub require_anonymous_route: bool,
}
```

---

# 41. DTN Integration

Store-carry-forward can maintain delivery under disconnected conditions.

---

# 42. Privacy Label

DTN bundle carries required privacy class.

---

# 43. Strict DTN Rule

Maximum-anonymity bundle cannot traverse privacy-ineligible opportunistic carrier.

---

# 44. Regional Failure

One region becomes unavailable.

---

# 45. Client Response

Use:

```text
fresh signed topology
alternate provider
alternate region
```

---

# 46. No Geographic Pinning Requirement

Clients should not rely on one home region.

---

# 47. Multi-Region Provider Set

Each service should have:

```text
primary candidates
secondary-region candidates
```

---

# 48. Region Selection

Local, privacy-aware.

---

# 49. Region Failover

Use jitter to avoid stampede.

---

# 50. Region Health

```rust
pub enum RegionHealth {
    Healthy,
    Degraded,
    Unavailable,
    Unknown,
}
```

---

# 51. Region Health Source

Aggregate signed control-plane signal.

---

# 52. Avoid Precise Client Location

Hard rule.

---

# 53. Multi-Region Topology

Mixnet layers should span multiple regions.

---

# 54. Do Not Map Layer to Region

Bad:

```text
L1 = Asia
L2 = Europe
L3 = US
```

---

# 55. Better

Each layer contains nodes across independent regions/operators.

---

# 56. Region Loss Effect

Topology builder removes unavailable nodes next epoch.

---

# 57. Mid-Epoch Failure

Clients use health/revocation overlay.

---

# 58. Directory Continuity

Directory control plane must survive outages.

---

# 59. Directory Components

```text
registry
policy
topology builder
signers
revocation feed
mirrors
```

---

# 60. Authority Quorum

Multi-authority future model.

---

# 61. Quorum Availability

Need enough authorities online to sign new epoch.

---

# 62. Quorum Failure

Existing valid topology can remain usable until freshness limit.

---

# 63. No Unsafe Emergency Signing

Hard rule.

Do not reduce quorum silently.

---

# 64. Quorum Degradation Policy

```rust
pub enum QuorumFailurePolicy {
    ContinueCurrentEpoch,
    FreezeNewTopology,
    StopStrictNewRoutes,
}
```

---

# 65. Maximum Anonymity

If topology freshness expires:

```text
stop creating new routes
```

---

# 66. Directory Mirrors

Read-only distribution layer.

---

# 67. Mirror Failure

Use another mirror.

---

# 68. Mirror Compromise

Signature verification prevents trust substitution.

---

# 69. Revocation Continuity

Revocation feed must be independently distributed.

---

# 70. Why

A stale topology may still contain compromised node.

---

# 71. Revocation Freshness

Stricter than topology freshness.

---

# 72. Revocation Outage

Strict mode may stop new routes after threshold.

---

# 73. Mailbox Reliability

Mailbox is durable store-and-forward.

---

# 74. Mailbox Failure Types

```text
process crash
storage corruption
region loss
operator loss
credential loss
```

---

# 75. Mailbox Replication

Possible:

```text
single provider replicas
multi-region same operator
multi-provider redundancy
```

---

# 76. Privacy Tradeoff

Replication increases metadata exposure.

---

# 77. Initial Recommendation

For standard HA:

```text
multi-region same provider
```

For maximum resilience/privacy:

```text
optional multi-provider encrypted redundancy
```

---

# 78. Multi-Provider Mailbox

Deposit identical encrypted payload to two providers.

---

# 79. Leakage

Two providers can compare object timing/size.

---

# 80. Mitigation

Use:

```text
independent IDs
independent capabilities
timing jitter
```

---

# 81. Redundancy Policy

```rust
pub enum MailboxRedundancyPolicy {
    Single,
    SameProviderMultiRegion,
    MultiProvider,
}
```

---

# 82. Mailbox Replica ID

Provider-specific.

---

# 83. No Shared Stable Object ID

Hard rule across providers.

---

# 84. Mailbox Write Quorum

Possible:

```text
1-of-2
2-of-3
```

---

# 85. Delivery Semantics

Application delivery after:

```text
at least one durable mailbox deposit
```

depending policy.

---

# 86. Strong Durability Mode

Require multiple deposits.

---

# 87. Latency Tradeoff

More replicas increase delay.

---

# 88. Mailbox Read

Recipient can fetch from:

```text
primary
fallback
all
```

---

# 89. Duplicate Dedup

Application MessageId.

---

# 90. Mailbox Disaster Recovery

If provider loses mailbox:

```text
client outbox retry
sender resend
secondary provider
```

---

# 91. Recipient Recovery

Fresh device gets fresh mailbox identity.

---

# 92. Never Restore Old Live Mailbox Secret Blindly

Hard rule.

---

# 93. Bulk Object Reliability

Encrypted chunks can be replicated.

---

# 94. Bulk Redundancy

Options:

```text
single provider
replicated provider
erasure-coded multi-provider
```

---

# 95. Initial Recommendation

Single provider + client retry for ordinary attachments.

---

# 96. High-Durability File

Optional multi-provider copy.

---

# 97. Backup vs Transfer

Bulk rendezvous is not archival backup.

---

# 98. Object Loss UX

```text
Attachment no longer available
```

---

# 99. Realtime Relay Reliability

Session is ephemeral.

---

# 100. Relay Failure

Migrate/reconnect.

---

# 101. Fresh Relay Credentials

Required.

---

# 102. Media Keys

May rotate during migration.

---

# 103. No Session State Restore From Disk

Hard rule.

---

# 104. Bridge Reliability

Bridge loss triggers alternate bridge/transport.

---

# 105. Censorship + Disaster

Bridge distribution needs independent channels.

---

# 106. Multi-Region Bridge Pools

Useful.

---

# 107. Provider Catalog Continuity

Signed catalog cached locally.

---

# 108. Catalog Outage

Use cached valid state within policy.

---

# 109. Capacity Market Outage

Use cached coarse capacity + local experience.

---

# 110. Price Catalog Outage

Existing price-lock/reservation may continue.

---

# 111. New Paid Allocation

May fail if pricing freshness required.

---

# 112. Accounting Continuity

Spend state must survive crash.

---

# 113. Uncertain Spend

Never blindly retry one-time token.

---

# 114. Settlement Outage

Service can continue if provider accepts deferred settlement.

---

# 115. Settlement Recovery

Idempotent batch replay.

---

# 116. Control Plane Database

Critical services may use PostgreSQL HA.

---

# 117. Data Plane Independence

Mix nodes should not depend on central DB per packet.

---

# 118. Why

Control-plane outage should not stop already-established forwarding.

---

# 119. Control Plane RPO

Define acceptable loss.

---

# 120. Recovery Point Objective

```rust
pub struct RecoveryPointObjective(pub Duration);
```

---

# 121. Recovery Time Objective

```rust
pub struct RecoveryTimeObjective(pub Duration);
```

---

# 122. Suggested Classes

```rust
pub enum RecoveryClass {
    R0,
    R1,
    R2,
    R3,
}
```

---

# 123. R0

Security-critical:

```text
near-zero logical data loss
```

---

# 124. R1

Durable user state:

```text
seconds/minutes
```

---

# 125. R2

Operational state:

```text
minutes
```

---

# 126. R3

Reconstructable cache:

```text
hours acceptable
```

---

# 127. Example Mapping

```text
Identity keys      → R0
Directory registry → R0/R1
Mailbox messages   → R1
Capacity cache     → R3
```

---

# 128. Backup Boundary

Backups must not become central privacy dump.

---

# 129. Backup Classes

```text
control-plane database
operator config
keys
user-independent logs
```

---

# 130. Excluded From Routine Central Backup

Prefer not to centrally back up:

```text
per-user route history
packet traces
presence IDs
contact graph
```

---

# 131. Key Backup

Special treatment.

---

# 132. Directory Root Key

Offline backup.

---

# 133. Authority Key

HSM/secure backup.

---

# 134. Provider Keys

Secure rotation/recovery.

---

# 135. Mailbox User Secrets

Client-owned; not operator recovery key.

---

# 136. No Master Decryption Key

Hard rule.

---

# 137. Encrypted Database Backups

Control-plane backups encrypted.

---

# 138. Backup Key Separation

Separate from application encryption keys.

---

# 139. Backup Region

Independent failure domain.

---

# 140. Backup Access

Least privilege.

---

# 141. Backup Retention

Bounded.

---

# 142. Disaster Restore

Must restore:

```text
control state
policy
registry
revocation
catalog metadata
```

without restoring stale security sessions.

---

# 143. Restore Validation

Before serving:

```text
schema
signatures
policy version
revocation freshness
```

---

# 144. Security Epoch

After major disaster, may advance:

```text
directory signing epoch
provider key epoch
```

---

# 145. Compromise vs Outage

Different recovery.

---

# 146. Outage Recovery

May reuse keys if uncompromised.

---

# 147. Compromise Recovery

Rotate keys and revoke old.

---

# 148. Do Not Confuse

Hard rule.

---

# 149. Regional Database Replication

For control plane:

```text
primary + replicas
```

---

# 150. Cross-Region Write Strategy

Potential:

```text
single writer
multi-writer
consensus
```

---

# 151. Initial Recommendation

Single logical writer with cross-region replicas for simpler correctness.

---

# 152. Multi-Writer Risk

Complex conflict handling.

---

# 153. Governance Data

Needs strong consistency.

---

# 154. Provider Health Data

Can tolerate eventual consistency.

---

# 155. Consistency Classes

```rust
pub enum ContinuityConsistency {
    Strong,
    Eventual,
    Rebuildable,
}
```

---

# 156. Strong

Examples:

```text
revocation
topology publication
credit settlement state
```

---

# 157. Eventual

Examples:

```text
health
capacity
```

---

# 158. Rebuildable

Examples:

```text
cache
index
```

---

# 159. Network Partition Control Plane

Split-brain must be prevented.

---

# 160. Directory Split Brain

Two regions publishing conflicting topology is critical.

---

# 161. Quorum Prevents

Need single logical authority state.

---

# 162. Minority Partition

Read-only/degraded.

---

# 163. No Independent Topology Publication

Hard rule.

---

# 164. Provider Split Brain

Mailbox replicas may diverge.

---

# 165. Conflict Rule

Mailbox items are append-only/idempotent.

---

# 166. Why Append-Only Helps

Simplifies merge.

---

# 167. Tombstones

Deletion/ACK requires careful replication.

---

# 168. Ack Replication

ACK after local durable write.

Cross-region propagation can follow.

---

# 169. Delete After Ack

Use retention grace.

---

# 170. Grace Window

Prevents losing item during replica lag.

---

# 171. Message Tombstone

```rust
pub struct MailboxTombstone {
    pub item_id: MailboxItemId,
    pub acknowledged_at: Timestamp,
    pub expires_at: Timestamp,
}
```

---

# 172. Tombstone Retention

Long enough for replica convergence.

---

# 173. Replay Safety

Duplicate redelivery harmless.

---

# 174. Partition Healing

Merge append-only items + tombstones.

---

# 175. Bulk Metadata Replication

Same pattern where possible.

---

# 176. Object Chunks

Immutable.

---

# 177. Manifest

Immutable after finalization.

---

# 178. Immutability

Strong reliability property.

---

# 179. Mutable State

Keep minimal.

---

# 180. Disaster Failover Coordinator

Local/control-plane function.

---

# 181. Failover Plan

```rust
pub struct FailoverPlan {
    pub failed_domain: FailureDomain,
    pub affected_services: Vec<AnonymousServiceType>,
    pub replacement_regions: Vec<RegionId>,
    pub privacy_constraints: PrivacyConstraintSet,
}
```

---

# 182. Failover Policy Engine

```rust
pub trait ContinuityPolicyEngine {
    fn plan(
        &self,
        event: FailureEvent,
        topology: &ContinuityTopology,
    ) -> Result<FailoverPlan, ContinuityError>;
}
```

---

# 183. Failure Event

```rust
pub struct FailureEvent {
    pub domain: FailureDomain,
    pub severity: FailureSeverity,
    pub detected_at: Timestamp,
}
```

---

# 184. Failure Severity

```rust
pub enum FailureSeverity {
    Degraded,
    PartialOutage,
    MajorOutage,
    Disaster,
    Compromise,
}
```

---

# 185. Automatic vs Manual Failover

Automatic for:

```text
ordinary node/region outage
```

Manual/security review for:

```text
suspected compromise
authority key issue
```

---

# 186. Why

Compromise recovery often requires revocation/rotation, not simple reroute.

---

# 187. Provider Failover

Uses Part 48 fallback list.

---

# 188. Resource Reallocation

Uses Part 49 scheduler.

---

# 189. Payment/Reservation Recovery

Uses Part 47 reconciliation.

---

# 190. Cross-Part Integration

Reliability must orchestrate existing subsystems, not bypass them.

---

# 191. Health Detection

Sources:

```text
local client failures
independent probes
provider self-report
control-plane monitors
```

---

# 192. Failure Detection Delay

Balance:

```text
fast response
vs
false positive
```

---

# 193. Hysteresis

Required.

---

# 194. Flapping Provider

Temporary deweight before hard removal.

---

# 195. Hard Failure

Immediate failover.

---

# 196. Privacy During Failover

Avoid all clients moving to same fallback.

---

# 197. Randomized Failover

Weighted among eligible alternatives.

---

# 198. Stampede Protection

Use:

```text
jitter
capacity-aware selection
backoff
```

---

# 199. Emergency Capacity

Reserve small headroom.

---

# 200. Headroom Policy

```rust
pub struct ContinuityHeadroom {
    pub reserve_fraction: f32,
}
```

---

# 201. Use

For sudden region/provider failure.

---

# 202. Cost

Underutilization during normal operation.

---

# 203. Realtime Calls

Reserve more headroom.

---

# 204. Mailbox/Bulk

Can tolerate slower failover.

---

# 205. Graceful Degradation

Order:

```text
background
bulk
attachments
interactive
security/emergency
```

---

# 206. Never Degrade Privacy First

Hard rule.

---

# 207. Feature Degradation

Possible:

```text
pause large files
lower video quality
delay background sync
```

---

# 208. Security Control

Always high priority.

---

# 209. Disaster Mode

```rust
pub enum AnonymousNetworkMode {
    Normal,
    Degraded,
    RegionalFailover,
    DisasterRecovery,
    SecurityRecovery,
}
```

---

# 210. User UX

Normal users see coarse:

```text
Anonymous network degraded
Messages may be delayed
```

---

# 211. Do Not Expose Topology Details

Hard rule.

---

# 212. Advanced Diagnostics

Can show:

```text
region health
provider redundancy class
catalog freshness
```

---

# 213. No Exact Route Disclosure

Hard rule.

---

# 214. Operator Console

Shows:

```text
service health
replica lag
capacity
failover state
backup freshness
```

---

# 215. DR Runbook

Every service needs:

```text
detect
contain
fail over
recover
validate
return
```

---

# 216. Runbook Versioning

Signed/internal controlled.

---

# 217. Disaster Drill

Regularly test.

---

# 218. Drill Types

```text
region loss
directory quorum loss
mailbox DB corruption
provider key compromise
catalog outage
mass relay failure
```

---

# 219. Game Day

Controlled production/staging exercise.

---

# 220. Privacy Check During Drill

Must verify:

```text
no direct fallback
no stale secret restoration
no central packet logging
```

---

# 221. Backup Restore Drill

Required.

---

# 222. Restore Validation

Measure:

```text
RPO
RTO
privacy invariants
```

---

# 223. Recovery Evidence

Record:

```text
drill date
scope
result
RPO/RTO achieved
privacy issues
```

---

# 224. No User Metadata in Drill Report

Hard rule.

---

# 225. Multi-Region Continuity Topology

```rust
pub struct ContinuityTopology {
    pub regions: Vec<RegionDescriptor>,
    pub operators: Vec<OperatorContinuityDescriptor>,
    pub services: Vec<ServiceContinuityDescriptor>,
}
```

---

# 226. Region Descriptor

```rust
pub struct RegionDescriptor {
    pub region_id: RegionId,
    pub provider: CloudOrInfraProviderId,
    pub health: RegionHealth,
}
```

---

# 227. Operator Continuity Descriptor

Contains:

```text
regions
capacity classes
independence metadata
```

---

# 228. Independence Audit

Two "regions" on same physical provider may still share failure risk.

---

# 229. Cloud Concentration

Avoid.

---

# 230. ASN Concentration

Avoid.

---

# 231. Control-Plane Concentration

Avoid all authorities in same region/provider.

---

# 232. Backup Concentration

Avoid backup in same failure domain.

---

# 233. Recovery Secrets

Use separate:

```text
root recovery key
operator recovery key
```

---

# 234. Secret Escrow

If used, threshold split.

---

# 235. No Single Recovery Master Secret

Hard rule.

---

# 236. Threshold Recovery

Possible:

```text
M-of-N custodians
```

for authority root.

---

# 237. Custodian Separation

Independent people/organizations.

---

# 238. Client Key Recovery

Separate from network DR.

---

# 239. User Identity Recovery

Follows Part 33 rules.

---

# 240. Fresh Anonymous Identity After Recovery

Hard rule where required.

---

# 241. Configuration Continuity

All production config versioned.

---

# 242. Config Backup

Contains:

```text
non-secret config
policy
deployment manifests
```

---

# 243. Secret Injection

Restored separately.

---

# 244. Infrastructure as Code

Recommended.

---

# 245. Immutable Deployment

Helps rebuild.

---

# 246. Reproducible Builds

Required for trusted recovery.

---

# 247. Artifact Registry

Multi-region/mirrored.

---

# 248. Signed Binaries

Required.

---

# 249. Supply Chain During Disaster

Do not bypass signature verification.

---

# 250. Emergency Manual Build

Still signed through approved path.

---

# 251. Schema Migration Recovery

Backups must include schema version.

---

# 252. Forward Restore

Use tested migration.

---

# 253. Downgrade

Avoid unsupported rollback.

---

# 254. Data Corruption Detection

Use:

```text
checksums
Merkle roots
database consistency checks
```

---

# 255. Mailbox Corruption

Corrupt item quarantined.

---

# 256. Do Not Crash Whole Mailbox

Hard rule.

---

# 257. Provider Compromise

Treat as security recovery.

---

# 258. Response

```text
revoke
remove from catalog/topology
rotate keys
migrate clients
```

---

# 259. Do Not Restore Compromised Key

Hard rule.

---

# 260. Compromised Region

May require broader key rotation.

---

# 261. Forensic Logging

Privacy tension.

---

# 262. Principle

Use infrastructure-level logs.

Avoid packet/user-level logs.

---

# 263. Incident Timeline

Can record:

```text
node health
operator actions
key rotations
topology changes
```

---

# 264. No User Traffic Reconstruction

Hard rule.

---

# 265. SLOs

Reliability SLO examples:

```text
directory availability
mailbox durability
provider catalog freshness
relay allocation success
```

---

# 266. Privacy SLO

Examples:

```text
zero silent direct fallback
zero stale-secret restore
zero cross-region raw social graph replication
```

---

# 267. Error Budgets

Can be tracked.

---

# 268. Privacy Incidents

Not absorbed into ordinary availability error budget.

---

# 269. They Are Security Events

Hard rule.

---

# 270. Continuity Metrics

Privacy-safe:

```text
failover count
replica lag bucket
RPO/RTO
catalog freshness
region health
```

---

# 271. Forbidden Metrics

No:

```text
per-user failover history
route identity
contact graph
mailbox secret
```

---

# 272. Testkit

Need multi-region fault simulator.

---

# 273. Fault Types

```text
process crash
host loss
zone loss
region loss
operator loss
network partition
packet loss
high latency
database corruption
authority loss
key compromise
```

---

# 274. Unit Tests

Test:

```text
failover policy
staleness
quorum rules
recovery classes
```

---

# 275. Integration Tests

Bring down:

```text
mailbox region
directory mirror
relay cluster
bulk provider
```

---

# 276. Region Loss Test

Messages remain queued/deliver through alternate eligible providers.

---

# 277. Quorum Loss Test

No unauthorized lower-quorum topology signing.

---

# 278. Revocation Outage Test

Strict route creation halts after freshness threshold.

---

# 279. Mailbox Replica Test

Partition + heal + dedup.

---

# 280. Tombstone Test

ACK/deletion converges.

---

# 281. Bulk Object Loss Test

Client receives correct unavailable/retry behavior.

---

# 282. Relay Failure Test

Call migrates or ends without direct fallback.

---

# 283. Bridge Failure Test

Alternate bridge.

---

# 284. Provider Catalog Outage Test

Valid cache works.

---

# 285. Catalog Expiry Test

Strict new selection stops.

---

# 286. Spend Reconciliation Test

Accounting recovers from crash.

---

# 287. Backup Restore Test

Control plane rebuilds in alternate region.

---

# 288. Key Compromise Test

Old key revoked, not restored.

---

# 289. Privacy Drill Test

No direct path selected during outage.

---

# 290. Chaos Engineering

Run controlled fault injection.

---

# 291. Privacy-Aware Chaos

Failures must include assertions:

```text
privacy mode remains satisfied
```

---

# 292. Fuzzing

Fuzz:

```text
failover event
recovery plan
replica merge
tombstone
backup metadata
```

---

# 293. Property Tests

Properties:

```text
failed strict provider cannot cause non-anonymous allocation
revoked key never reactivated by restore
mailbox duplicate delivery is idempotent
quorum cannot decrease automatically
```

---

# 294. Formal Verification Targets

Strong candidates:

```text
directory quorum failover
mailbox replication/ACK
recovery state machine
provider failover policy
```

---

# 295. TLA+ Candidate

Multi-region directory quorum + failover.

---

# 296. TLA+ Candidate

Mailbox append/ACK/tombstone convergence.

---

# 297. Kani Candidate

Recovery policy hard constraints.

---

# 298. Loom Candidate

Concurrent replica merge/ACK.

---

# 299. Performance Tests

Measure:

```text
failover time
recovery time
replica lag
resync bandwidth
```

---

# 300. Scale Tests

Synthetic:

```text
regional outage with millions of queued messages
```

---

# 301. Stampede Test

Mass client failover.

---

# 302. Capacity Headroom Test

Alternate regions absorb load.

---

# 303. Multi-Operator Outage Test

One operator disappears entirely.

---

# 304. Disaster Recovery Drill Cadence

Recommended:

```text
regular staging
periodic production game-day
```

---

# 305. Crate Layout

Recommended:

```text
crates/
├── siar-continuity-core/
├── siar-failure-domain/
├── siar-failover-policy/
├── siar-region-health/
├── siar-mailbox-replication/
├── siar-directory-continuity/
├── siar-backup-recovery/
├── siar-disaster-runbook/
├── siar-chaos/
├── siar-continuity-observability/
└── siar-continuity-testkit/
```

---

# 306. `siar-continuity-core`

Owns:

```text
failure events
RPO/RTO types
continuity classes
errors
```

---

# 307. `siar-failure-domain`

Failure-domain metadata and independence.

---

# 308. `siar-failover-policy`

Privacy-aware recovery planning.

---

# 309. `siar-region-health`

Regional status/aggregation.

---

# 310. `siar-mailbox-replication`

Replica/ACK/tombstone behavior.

---

# 311. `siar-directory-continuity`

Quorum/mirror/revocation continuity.

---

# 312. `siar-backup-recovery`

Control-plane backup/restore.

---

# 313. `siar-disaster-runbook`

Machine-readable recovery workflows.

---

# 314. `siar-chaos`

Fault injection.

---

# 315. `siar-continuity-observability`

Privacy-safe SLO/DR metrics.

---

# 316. `siar-continuity-testkit`

Multi-region simulator.

---

# 317. Error Taxonomy

```rust
pub enum ContinuityError {
    RegionUnavailable,
    ProviderUnavailable,
    QuorumUnavailable,
    TopologyStale,
    RevocationStateStale,
    ReplicaDiverged,
    BackupInvalid,
    RecoveryKeyUnavailable,
    PrivacyConstraintUnsatisfied,
    CapacityInsufficient,
    Internal,
}
```

---

# 318. Security Invariants

Mandatory:

```text
1. Failure never silently causes a privacy downgrade.
2. Strict messages remain queued if no eligible anonymous route exists.
3. Directory quorum requirements cannot auto-decrease during outage.
4. Revoked or compromised keys are never restored from backup.
5. Fresh anonymous mailbox/session identities are created after recovery where required.
6. Mailbox replicas use provider-specific identifiers and application MessageId dedup.
7. Cross-region backup does not become a centralized user-metadata archive.
8. Realtime relay failure cannot fall back to direct peer media silently.
9. Provider failover uses privacy-eligible candidates only.
10. Disaster recovery preserves signed policy, revocation, and anti-rollback state.
11. Split-brain topology publication is forbidden.
12. Reliability telemetry never includes social graph or route identifiers.
```

---

# 319. Initial Production Scope

Implement first:

```text
local durable outbox
multi-region directory mirrors
single logical directory writer + replicas
provider failover
regional health
mailbox same-provider multi-region replication
append-only mailbox items + tombstones
RPO/RTO classes
encrypted cross-region control-plane backups
disaster runbooks
chaos/fault testkit
privacy-aware failover assertions
```

Then add:

```text
multi-provider mailbox redundancy
threshold authority continuity
cross-provider encrypted erasure coding
advanced regional headroom scheduling
automated privacy-aware disaster orchestration
```

---

# 320. Definition of Done

Part 50 is complete when:

- process/host/zone/region/operator/ASN/cloud/authority failure domains are explicit
- continuity classes and RPO/RTO categories are defined
- client durable outbox remains authoritative for unsent messages
- partitions result in queueing rather than privacy downgrade
- directory quorum, mirror, topology, and revocation continuity are defined
- mailbox replication and duplicate/tombstone behavior are defined
- multi-region provider failover preserves privacy constraints
- control-plane backup boundaries avoid centralizing user metadata
- compromised keys are never restored
- disaster restore, key rotation, and security recovery are distinguished
- realtime, bridge, bulk, accounting, provider, and resource failover integrate with previous parts
- chaos, region-loss, split-brain, restore, capacity-stampede, and privacy-invariant tests are specified
- disaster drills measure both availability and privacy correctness

---

# 321. Final Architecture

```text
                    CLIENT DURABLE STATE
                            │
                            ▼
                  PRIVACY-AWARE FAILOVER
                            │
            ┌───────────────┼───────────────┐
            │               │               │
        Region A        Region B        Region C
            │               │               │
            └───────────────┼───────────────┘
                            ▼
                  MULTI-OPERATOR SERVICES
                            │
                            ▼
                CONTROL-PLANE CONTINUITY
                            │
                  Backup / Quorum / DR
```

Failure semantics:

```text
provider/region unavailable
    ↓
select privacy-eligible replacement
    ↓
if none available
    ↓
queue / degrade feature

NOT

switch to weaker direct path
```

---

# 322. Final Principle

Anonymous infrastructure should fail **closed with respect to privacy**, while failing **gracefully with respect to availability**.

The correct model is:

```text
durable client state
+
multi-region/operator diversity
+
signed control-plane continuity
+
privacy-aware failover
+
bounded replication
+
secure backup
+
regular disaster drills
```

not:

```text
when anonymous infrastructure breaks, bypass it
```

This architecture gives SIAR a path to survive major outages and disasters while preserving the anonymity, identity-separation, and no-silent-downgrade guarantees established across Parts 34–49.
