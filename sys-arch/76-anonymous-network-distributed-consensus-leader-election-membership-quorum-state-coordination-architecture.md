# Core System Architecture Part 76 — Anonymous Network Distributed Consensus, Leader Election, Membership, Quorum State & Coordination Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 76  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 5, 11, 34, 38–39, 46–53, 58, 61–75  

**Primary purpose:** define where SIAR requires strong distributed consensus, where it must avoid it, how leader election, membership, quorum state, replicated state machines, fencing, leases, and coordination work, and how consensus integrates with privacy, federation, multi-region operation, governance, deployment, eventing, and failure recovery.

---

# 1. Purpose

Consensus is powerful but expensive.

Misusing it can create:

```text
global bottlenecks
fragile availability
centralized trust
cross-region latency
privacy-sensitive shared state
```

The governing principle is:

> **SIAR uses consensus only for small, well-defined coordination domains that truly require strong agreement. User messaging and anonymous data-plane traffic must not depend on one global consensus cluster.**

---

# 2. Architectural Position

```text
Strongly Consistent Domain State
            │
            ▼
   Replicated State Machine
            │
            ▼
       Consensus Group
            │
      ┌─────┼─────┐
      │     │     │
    Node A Node B Node C
      │     │     │
      └─────┼─────┘
            ▼
     Committed Quorum State
```

---

# 3. Core Separation

Keep distinct:

```text
consensus
replication
leader election
membership
leases
locks
eventual sync
federation
```

---

# 4. Non-Goals

Part 76 does not create:

```text
one global Raft cluster
consensus for every chat message
consensus between all federation domains
consensus across anonymous clients
```

---

# 5. Consensus Use Cases

Good candidates:

```text
governance state
directory authority state
control-plane configuration
tenant provisioning authority
release/deployment coordination
```

---

# 6. Poor Candidates

Avoid consensus for:

```text
presence
typing
message delivery
attachment transfer
client local state
```

---

# 7. Consensus Domain

```rust
pub enum ConsensusDomain {
    Governance,
    Directory,
    ControlPlane,
    TenantControl,
    Deployment,
}
```

---

# 8. One Domain ≠ One Global Cluster

Hard rule.

---

# 9. Domain-Scoped Consensus

Each domain can have its own cluster.

---

# 10. Consensus Group ID

```rust
pub struct ConsensusGroupId(pub [u8; 32]);
```

---

# 11. No User Identity

Hard rule.

---

# 12. Replicated State Machine

```rust
pub trait ReplicatedStateMachine {
    type Command;
    type Response;

    fn apply(
        &mut self,
        command: Self::Command,
    ) -> Result<Self::Response, ConsensusError>;
}
```

---

# 13. Determinism

State-machine apply must be deterministic.

---

# 14. No Wall-Clock Randomness In Apply

Hard rule.

---

# 15. No Network Calls In Apply

Hard rule.

---

# 16. External Effects

Driven after commit.

---

# 17. Consensus Algorithm

Raft-like model recommended.

---

# 18. Why

```text
understandable
mature
leader-based
membership support
```

---

# 19. No Custom Consensus Algorithm

Hard rule.

---

# 20. Raft Library

Use reviewed Rust implementation where mature.

---

# 21. Consensus Node

```rust
pub struct ConsensusNode {
    pub node_id: ConsensusNodeId,
    pub group: ConsensusGroupId,
}
```

---

# 22. Node ID

Infrastructure-only.

---

# 23. Member Role

```rust
pub enum ConsensusRole {
    Voter,
    Learner,
}
```

---

# 24. Voter

Counts toward quorum.

---

# 25. Learner

Replicates but does not vote.

---

# 26. Learner Use

```text
new node catch-up
read replica
migration
```

---

# 27. Quorum

Majority of voters.

---

# 28. Quorum Requirement

```rust
pub struct QuorumRequirement {
    pub voters: u16,
    pub required: u16,
}
```

---

# 29. Odd Voter Count

Usually preferred.

---

# 30. Three Nodes

Tolerates one failure.

---

# 31. Five Nodes

Tolerates two.

---

# 32. No Two-Node Consensus For Critical State

Hard rule.

---

# 33. Quorum Safety

Without majority:

```text
no new authoritative writes
```

---

# 34. Hard Rule

Never reduce quorum silently to restore availability.

---

# 35. Split Brain

Must be impossible by design.

---

# 36. Leader Election

Leader chosen by consensus.

---

# 37. Leader Role

Coordinates replication.

---

# 38. Leader Is Not Root Authority

Hard rule.

---

# 39. Leadership ≠ Governance Authority

Leadership only orders commands.

---

# 40. Leader Election Timeout

Randomized.

---

# 41. Why

Avoid election collisions.

---

# 42. Election Timing

Infrastructure-only.

---

# 43. No user metadata involvement.

---

# 44. Term

```rust
pub struct ConsensusTerm(pub u64);
```

---

# 45. Log Index

```rust
pub struct LogIndex(pub u64);
```

---

# 46. Commit Index

Monotonic.

---

# 47. No Index Rollback

Hard rule.

---

# 48. Command Envelope

```rust
pub struct ConsensusCommand<T> {
    pub command_id: CommandId,
    pub term_context: Option<ConsensusTerm>,
    pub payload: T,
}
```

---

# 49. Command Idempotency

Required.

---

# 50. Duplicate Command

Must not apply twice semantically.

---

# 51. Leader Lease

Optional optimization.

---

# 52. Read Lease

Can support low-latency linearizable reads.

---

# 53. Lease Caveat

Clock assumptions matter.

---

# 54. Prefer ReadIndex/Quorum Read

For strict correctness.

---

# 55. Linearizable Read

```rust
pub enum ReadConsistency {
    Linearizable,
    LeaseBased,
    StaleAllowed,
}
```

---

# 56. Governance

Linearizable.

---

# 57. Directory Snapshot Read

Can allow stale bounded reads.

---

# 58. Control Plane

Depends on operation.

---

# 59. Hard Rule

Read consistency must be explicit.

---

# 60. Membership Change

High-risk.

---

# 61. Membership State

```rust
pub struct ConsensusMembership {
    pub voters: BTreeSet<ConsensusNodeId>,
    pub learners: BTreeSet<ConsensusNodeId>,
}
```

---

# 62. Add Node

Recommended flow:

```text
provision
→ attest
→ add learner
→ catch up
→ promote voter
```

---

# 63. No Direct Voter Addition Without Catch-Up

Hard rule.

---

# 64. Remove Node

```text
mark draining
→ transfer leadership if needed
→ remove membership
→ revoke credentials
```

---

# 65. Joint Consensus

Use for membership changes.

---

# 66. Why

Prevent quorum ambiguity.

---

# 67. No Single-Step Unsafe Membership Rewrite

Hard rule.

---

# 68. Membership Change State

```rust
pub enum MembershipChangeState {
    Planned,
    Joint,
    Finalizing,
    Completed,
    Failed,
}
```

---

# 69. Membership Authority

Part 53/61 policy.

---

# 70. Leader Cannot Add Itself New Voters Arbitrarily

Hard rule.

---

# 71. Membership Command Authorization

Signed/control-plane authorized.

---

# 72. Membership Quorum

Current cluster must approve.

---

# 73. No Out-of-Band DB Edit

Hard rule.

---

# 74. Node Identity

Separate from deployment host ID.

---

# 75. Node credential

Consensus-specific.

---

# 76. Mutual Authentication

Required.

---

# 77. mTLS/QUIC Identity

Possible.

---

# 78. No Anonymous Consensus Member

Hard rule.

---

# 79. But

Consensus membership is infrastructure-private.

---

# 80. Network Transport

Reliable ordered stream.

---

# 81. QUIC

Good candidate.

---

# 82. TLS

Fine.

---

# 83. Consensus Wire Format

Versioned Postcard preferred.

---

# 84. Strict bounds.

---

# 85. Compatibility

Mixed-version rolling upgrade.

---

# 86. Consensus Protocol Version

Independent.

---

# 87. State-Machine Version

Independent.

---

# 88. Wire Version

Independent.

---

# 89. No Version Conflation

Hard rule.

---

# 90. Log Entry

```rust
pub struct ConsensusLogEntry {
    pub term: ConsensusTerm,
    pub index: LogIndex,
    pub command: Bytes,
    pub schema_version: ConsensusEntryVersion,
}
```

---

# 91. Log Storage

Durable.

---

# 92. fsync Semantics

Critical.

---

# 93. Commit Ack

Only after required durability.

---

# 94. No Commit Before Durable Majority

Hard rule.

---

# 95. Storage Corruption

Critical.

---

# 96. Snapshot

Used for log compaction.

---

# 97. Snapshot Metadata

```rust
pub struct ConsensusSnapshot {
    pub last_included_index: LogIndex,
    pub last_included_term: ConsensusTerm,
    pub state_digest: ContentDigest,
}
```

---

# 98. Snapshot Integrity

Signed/checksummed.

---

# 99. Snapshot Install

Validated.

---

# 100. No Blind Snapshot Trust

Hard rule.

---

# 101. Snapshot Encryption

If state sensitive.

---

# 102. Governance state

May be public-ish but still integrity-critical.

---

# 103. Tenant control state

Sensitive.

---

# 104. Snapshot Retention

Bounded.

---

# 105. Snapshot Restore

Must respect anti-rollback.

---

# 106. Hard rule.

---

# 107. Log Compaction

Safe only after snapshot.

---

# 108. No Compaction Of Uncommitted State

Hard rule.

---

# 109. Leader Transfer

Needed for maintenance.

---

# 110. Graceful transfer.

---

# 111. No Forced Leadership On Unsynced Node

Hard rule.

---

# 112. Preferred Leader Placement

Can consider:

```text
latency
health
region
```

---

# 113. But

No static permanent leader.

---

# 114. Leader Re-election

Normal.

---

# 115. Multi-Region Consensus

Tradeoff.

---

# 116. Cross-Region RTT

Raises commit latency.

---

# 117. Critical Control State

May justify.

---

# 118. User data plane

Must not depend.

---

# 119. Region Placement

Spread voters.

---

# 120. Example

Five voters across:

```text
3 regions
2+ operators
```

where practical.

---

# 121. Failure Domain Diversity

Part 50/62.

---

# 122. No All Voters Same Cloud/ASN If Avoidable

Hard rule.

---

# 123. Quorum Placement Planner

```rust
pub trait QuorumPlacementPlanner {
    fn plan(
        &self,
        voters: u16,
        inventory: &InfrastructureInventory,
    ) -> Result<ConsensusPlacement, ConsensusError>;
}
```

---

# 124. Placement Constraints

```text
region diversity
operator diversity
cloud diversity
legal constraints
```

---

# 125. No Privacy-Weak Region Failover

Hard rule.

---

# 126. Consensus Under Partition

Minority side:

```text
read stale if allowed
no authoritative writes
```

---

# 127. Majority side

Continues.

---

# 128. Partition Healing

Logs reconcile.

---

# 129. No Dual Commit History

Hard rule.

---

# 130. Availability Tradeoff

Explicit.

---

# 131. CAP

Strong consistency sacrifices availability in minority partition.

---

# 132. This Is Acceptable

For authoritative control state.

---

# 133. Consensus Scope Minimization

Important.

---

# 134. Do Not Put High-Volume User Events In Consensus Log

Hard rule.

---

# 135. Why

```text
latency
privacy
storage
scaling
```

---

# 136. Consensus Stores Commands/authoritative state transitions.

---

# 137. User Messaging

Uses local durable stores/event queues.

---

# 138. Directory Topology

Consensus may commit snapshot metadata, not every packet.

---

# 139. Governance

Consensus tracks approved policy state.

---

# 140. Tenant Provisioning

Consensus tracks tenant lifecycle state.

---

# 141. Deployment

Consensus can coordinate desired rollout state.

---

# 142. No Cross-Domain Global Consensus

Hard rule.

---

# 143. Federation

Each domain has independent consensus.

---

# 144. Cross-domain agreement

Via signed federation protocols.

---

# 145. No federation-wide Raft.

---

# 146. Governance Quorum vs Consensus Quorum

Different concepts.

---

# 147. Governance Quorum

Cryptographic authority approval.

---

# 148. Consensus Quorum

Replication agreement.

---

# 149. Hard Rule

Do not substitute one for the other.

---

# 150. Example

Policy object:

```text
first governance signatures valid
then control-plane consensus commits it
```

---

# 151. Leader Cannot Bypass Governance Signatures

Hard rule.

---

# 152. Consensus Cannot Manufacture Authority

Hard rule.

---

# 153. External Authorization

Validated before state-machine command accepted.

---

# 154. Validation Layer

```rust
pub trait ConsensusCommandValidator<T> {
    fn validate(
        &self,
        command: &T,
    ) -> Result<ValidatedConsensusCommand<T>, ConsensusError>;
}
```

---

# 155. Validation Should Be Deterministic

Yes.

---

# 156. Signature Verification

Deterministic.

---

# 157. External HTTP lookup

Not allowed in apply.

---

# 158. Pre-validation

Can occur before proposal.

---

# 159. Commit-time invariant check

Still deterministic.

---

# 160. Fencing

Critical for external side effects.

---

# 161. Fencing Token

```rust
pub struct ConsensusFencingToken {
    pub term: ConsensusTerm,
    pub index: LogIndex,
}
```

---

# 162. External Worker

Uses latest fencing token.

---

# 163. Stale Leader Worker

Rejected.

---

# 164. Hard rule.

---

# 165. Lease

Use only where fencing exists.

---

# 166. Distributed Lock

Prefer consensus-backed lease only for narrow cases.

---

# 167. No Global Lock Service

Hard rule.

---

# 168. Better Alternatives

```text
partition ownership
DB unique constraint
workflow lease
```

---

# 169. Consensus Lock Use

Rare.

---

# 170. Lock Record

```rust
pub struct ConsensusLease {
    pub resource: CoordinationResourceId,
    pub holder: ConsensusNodeId,
    pub fencing: ConsensusFencingToken,
}
```

---

# 171. Lease Expiry

Requires careful clock semantics.

---

# 172. Prefer leader term/index fencing over wall-clock alone.

---

# 173. Leader Health

Heartbeat.

---

# 174. No User Traffic Heartbeat

Infrastructure only.

---

# 175. Election Storm

Need mitigation.

---

# 176. Randomized timeout.

---

# 177. Stable network.

---

# 178. PreVote

Recommended if library supports.

---

# 179. Why

Reduce disruptive elections.

---

# 180. Leadership Stickiness

Not too much.

---

# 181. Snapshot Transfer

Bandwidth bounded.

---

# 182. Large State Machine

Warning.

---

# 183. Consensus state should remain small.

---

# 184. Hard rule.

---

# 185. Bulk Data

Stored elsewhere.

---

# 186. Consensus Stores references/digests.

---

# 187. Example

Release manifest digest, not full artifact.

---

# 188. Mailbox ciphertext

Never consensus state.

---

# 189. User messages

Never consensus state.

---

# 190. Event Bus Integration

Part 75.

---

# 191. After Consensus Commit

Emit integration event via outbox.

---

# 192. No Event Before Commit

Hard rule.

---

# 193. Consensus Outbox

```text
apply command
→ update state
→ append outbox event
→ commit state-machine transaction
```

---

# 194. Dispatcher publishes later.

---

# 195. Exactly-once effect

Via idempotency.

---

# 196. Workflow Integration

Consensus can coordinate one workflow phase.

---

# 197. Do not put entire workflow in consensus unless necessary.

---

# 198. Database Integration

Part 74.

---

# 199. Consensus Log Storage

Dedicated store.

---

# 200. State Machine Store

Transactional.

---

# 201. Log + state atomicity

Implementation-dependent.

---

# 202. If library separates them

Need crash-safe recovery protocol.

---

# 203. No Applied State Ahead Of Commit Index

Hard rule.

---

# 204. No Commit Index Ahead Of Durable Log

Hard rule.

---

# 205. Read Model

Can derive from state machine.

---

# 206. Cache

Not authority.

---

# 207. Membership Store

Consensus-owned.

---

# 208. No External mutable membership DB.

---

# 209. Deployment Integration

Part 62.

---

# 210. Node bootstrap

Needs cluster join token/cert.

---

# 211. Join Token

Short-lived.

---

# 212. Join Token ≠ Membership

Hard rule.

---

# 213. Membership requires committed config change.

---

# 214. Attestation Integration

Part 71.

---

# 215. New voter

Must meet attestation policy.

---

# 216. Quarantined voter

Remove/drain.

---

# 217. No Compromised Voter Kept For Quorum Count

Hard rule.

---

# 218. HSM Integration

Part 72.

---

# 219. Consensus node key

Routine/sensitive.

---

# 220. Governance signer

Separate high-value key.

---

# 221. No HSM Root Used For Consensus Heartbeats

Hard rule.

---

# 222. Supply Chain

Part 73.

---

# 223. Consensus binary exact artifact.

---

# 224. Mixed version upgrade

Compatibility tested.

---

# 225. Protocol Feature Gate

Only after quorum supports.

---

# 226. Feature Activation

```rust
pub enum ConsensusFeatureState {
    Disabled,
    Supported,
    Activated,
}
```

---

# 227. Supported

Nodes understand.

---

# 228. Activated

State machine may emit/use.

---

# 229. No Early Feature Use

Hard rule.

---

# 230. Upgrade Strategy

```text
upgrade learners
→ upgrade followers
→ transfer leader
→ upgrade old leader
→ activate feature
```

---

# 231. Rollback

Only while compatible.

---

# 232. Anti-Rollback

Security epoch still applies.

---

# 233. No downgrade to blocked protocol.

---

# 234. Failure Scenarios

```text
leader crash
follower crash
minority partition
majority partition
disk corruption
snapshot corruption
membership change crash
```

---

# 235. Leader Crash

Election.

---

# 236. Follower Crash

No service interruption if quorum remains.

---

# 237. Minority Partition

No writes.

---

# 238. Majority Loss

Cluster unavailable for writes.

---

# 239. Hard rule

No emergency single-node mode.

---

# 240. Disk Corruption

Node quarantined/rebuilt.

---

# 241. Snapshot Corruption

Reject.

---

# 242. Membership Change Crash

Joint consensus recovers deterministically.

---

# 243. Learner Catchup Failure

Do not promote.

---

# 244. Stale Voter

Catch up before leadership.

---

# 245. No Force Leader To Stale Node

Hard rule.

---

# 246. Disaster Recovery

Part 70.

---

# 247. Consensus DR

Rebuild minority nodes from healthy quorum.

---

# 248. Total Quorum Loss

More serious.

---

# 249. Need cluster recovery procedure.

---

# 250. Recovery From Total Quorum Loss

Highly controlled.

---

# 251. Requires:

```text
latest durable state
operator approval
epoch fencing
new membership
```

---

# 252. No Automatic Bootstrap New Cluster From Stale Snapshot

Hard rule.

---

# 253. Recovery Epoch

```rust
pub struct ConsensusRecoveryEpoch(pub u64);
```

---

# 254. New recovery epoch

Prevents old cluster rejoining.

---

# 255. Old cluster credentials revoked.

---

# 256. Hard rule.

---

# 257. Split-Brain After Disaster

Prevent via recovery epoch.

---

# 258. Consensus Backup

Snapshot + log tail.

---

# 259. Backup encrypted.

---

# 260. Restore tested.

---

# 261. Backup not enough without quorum recovery plan.

---

# 262. Read-Only Mode

Possible during quorum loss.

---

# 263. Only if stale reads safe.

---

# 264. Governance

Can serve last known signed policy.

---

# 265. Cannot accept new policy.

---

# 266. Directory

Can serve cached signed snapshot.

---

# 267. Tenant provisioning

No new tenants.

---

# 268. Deployment

Freeze changes.

---

# 269. This is safe degradation.

---

# 270. Observability

Metrics:

```text
leader status
term
commit lag
replication lag
election count
membership
```

---

# 271. No User Data.

---

# 272. Consensus SLO

Examples:

```text
leader election within target
commit p99
follower lag
snapshot health
```

---

# 273. Election Count

Too high = instability.

---

# 274. No Per-User Metrics

Hard rule.

---

# 275. Alerting

```text
quorum risk
lagging voter
repeated elections
disk latency
snapshot failure
```

---

# 276. Runbooks

Part 65.

---

# 277. Mandatory Runbooks

```text
leader churn
voter loss
membership change failure
total quorum loss
snapshot corruption
```

---

# 278. Security Threats

```text
malicious voter
stolen node credential
network partition
replay
log tamper
membership injection
```

---

# 279. Malicious Voter

Raft assumes non-Byzantine failures.

---

# 280. Important

Standard Raft does not tolerate Byzantine nodes.

---

# 281. Hard truth.

---

# 282. Threat Model

Consensus members are authenticated trusted infrastructure.

---

# 283. If Byzantine tolerance required

Need separate BFT design.

---

# 284. Do Not Claim Raft Provides BFT

Hard rule.

---

# 285. Governance signatures

Can reduce damage of malicious leader.

---

# 286. Malicious leader cannot forge signed policy.

---

# 287. But

Could disrupt availability.

---

# 288. Detect/quarantine.

---

# 289. Byzantine Consensus

Not default.

---

# 290. Potential future

For selected governance domains.

---

# 291. But

Much higher complexity.

---

# 292. No custom BFT.

---

# 293. Membership Attacks

Join requires:

```text
authenticated node
attestation
authorized membership proposal
committed config change
```

---

# 294. Replay Join Token

Rejected.

---

# 295. Credential Theft

Revoke.

---

# 296. Node Re-enroll

Fresh identity.

---

# 297. Consensus Transport Security

mTLS/QUIC.

---

# 298. Certificate rotation

Must not break quorum.

---

# 299. Staged credential rotation.

---

# 300. No All-Node Credential Expiry Same Instant

Hard rule.

---

# 301. Time

Part 60.

---

# 302. Consensus safety should not depend on precise wall clock.

---

# 303. Election timers use monotonic clock.

---

# 304. Lease read uses bounded assumptions.

---

# 305. No wall-clock timestamp ordering for log.

---

# 306. Performance

Commit latency roughly quorum RTT + fsync.

---

# 307. Cross-region cost expected.

---

# 308. Keep consensus state/write rate low.

---

# 309. Batch Commands

Possible.

---

# 310. Preserve deterministic order.

---

# 311. No batch bypass of authorization.

---

# 312. Snapshot frequency

Based on log growth.

---

# 313. Not too frequent.

---

# 314. Not too rare.

---

# 315. Benchmark.

---

# 316. Membership Scaling

Consensus groups should stay small.

---

# 317. 3–7 voters typical conceptual range.

---

# 318. No Hundreds Of Voters

Hard rule.

---

# 319. Scale by sharding independent consensus domains.

---

# 320. Example

Tenant control plane shards by tenant group.

---

# 321. Directory authorities may use bounded group.

---

# 322. No global scale through giant quorum.

---

# 323. Consensus Sharding

Each shard independent.

---

# 324. Cross-Shard Transaction

Avoid.

---

# 325. If needed

Saga/event coordination.

---

# 326. No distributed cross-shard atomic transaction.

---

# 327. Hard rule.

---

# 328. Coordination Service

Narrow.

---

# 329. Could expose:

```text
leadership
membership
monotonic epoch
fencing token
```

---

# 330. Coordination API

```rust
pub trait CoordinationService {
    fn current_leader(
        &self,
        group: ConsensusGroupId,
    ) -> Result<Option<ConsensusNodeId>, ConsensusError>;

    fn fencing_token(
        &self,
        group: ConsensusGroupId,
    ) -> Result<ConsensusFencingToken, ConsensusError>;
}
```

---

# 331. No Generic Distributed Lock API By Default

Hard rule.

---

# 332. Why

Encourages misuse.

---

# 333. Prefer domain-specific coordination.

---

# 334. Consensus State Registry

Machine-readable.

---

# 335. Descriptor

```rust
pub struct ConsensusDomainDescriptor {
    pub domain: ConsensusDomain,
    pub consistency: ReadConsistency,
    pub voter_target: u16,
    pub bft_required: bool,
}
```

---

# 336. `bft_required`

If true, Raft adapter invalid.

---

# 337. Capability Validation

```rust
pub trait ConsensusCapabilityValidator {
    fn validate(
        &self,
        descriptor: &ConsensusDomainDescriptor,
        implementation: &ConsensusCapabilities,
    ) -> Result<(), ConsensusError>;
}
```

---

# 338. No Wrong Algorithm For Threat Model

Hard rule.

---

# 339. Testing

Need consensus testkit.

---

# 340. Deterministic Simulation

Part 64.

---

# 341. Simulate:

```text
message loss
delay
reordering
partition
node crash
disk failure
```

---

# 342. Leader Election Test

One leader per term.

---

# 343. Commit Safety Test

No conflicting committed entries.

---

# 344. Membership Test

Joint consensus preserves quorum.

---

# 345. Learner Test

Cannot vote before promotion.

---

# 346. Stale Leader Test

Fencing rejects side effect.

---

# 347. Partition Test

Minority cannot write.

---

# 348. Quorum Loss Test

No emergency unsafe mode.

---

# 349. Snapshot Test

Install/recover.

---

# 350. Snapshot Tamper Test

Reject.

---

# 351. Recovery Epoch Test

Old cluster cannot rejoin.

---

# 352. Credential Rotation Test

Quorum maintained.

---

# 353. Mixed-Version Test

Safe rolling upgrade.

---

# 354. Feature Activation Test

Only after support threshold.

---

# 355. Governance Integration Test

Unsigned policy rejected even if consensus proposes.

---

# 356. Tenant Isolation Test

Wrong shard/tenant command rejected.

---

# 357. Fuzzing

Fuzz:

```text
consensus wire frame
membership command
snapshot metadata
state-machine command
```

---

# 358. Property Tests

Properties:

```text
at most one leader per term
committed log prefix never diverges
minority partition cannot commit
membership transition never has ambiguous quorum
```

---

# 359. Formal Verification

Strongly recommended.

---

# 360. TLA+

Excellent fit.

---

# 361. Model:

```text
election
replication
membership
partition
recovery epoch
```

---

# 362. Kani

Good for:

```text
quorum arithmetic
membership validation
fencing checks
```

---

# 363. Loom

Good for local consensus adapter concurrency.

---

# 364. Jepsen-Style Testing

Conceptually valuable.

---

# 365. Test:

```text
linearizability
partition behavior
failover
```

---

# 366. No Production User Data.

---

# 367. Chaos

Part 64/70.

---

# 368. Periodic voter loss drill.

---

# 369. Leadership transfer drill.

---

# 370. Quorum loss tabletop.

---

# 371. Total cluster recovery test in staging.

---

# 372. No untested quorum recovery.

---

# 373. Crate Layout

Recommended:

```text
crates/
├── siar-consensus-core/
├── siar-consensus-raft/
├── siar-consensus-storage/
├── siar-consensus-membership/
├── siar-consensus-coordination/
├── siar-consensus-fencing/
├── siar-consensus-snapshot/
├── siar-consensus-observability/
└── siar-consensus-testkit/
```

---

# 374. `siar-consensus-core`

Owns:

```text
terms
indexes
groups
roles
errors
```

---

# 375. `siar-consensus-raft`

Reviewed Raft adapter.

---

# 376. `siar-consensus-storage`

Log/state-machine durability.

---

# 377. `siar-consensus-membership`

Joint-consensus membership changes.

---

# 378. `siar-consensus-coordination`

Leader/read/coordination APIs.

---

# 379. `siar-consensus-fencing`

Stale leader protection.

---

# 380. `siar-consensus-snapshot`

Snapshot/compaction/install.

---

# 381. `siar-consensus-observability`

Privacy-safe metrics.

---

# 382. `siar-consensus-testkit`

Deterministic partition/crash simulator.

---

# 383. Error Taxonomy

```rust
pub enum ConsensusError {
    NotLeader,
    QuorumUnavailable,
    MembershipInvalid,
    NodeNotCaughtUp,
    StaleTerm,
    FencingRejected,
    SnapshotInvalid,
    UnsupportedVersion,
    RecoveryEpochMismatch,
    UnauthorizedCommand,
    Internal,
}
```

---

# 384. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. SIAR has no single global consensus cluster for user messaging or anonymous data-plane traffic.
2. Consensus is used only for narrow authoritative coordination domains that genuinely require strong agreement.
3. Minority partitions cannot commit authoritative writes.
4. Quorum thresholds are never silently reduced to restore availability.
5. Membership changes use a safe joint-consensus transition and require authenticated authorization.
6. A consensus leader orders commands but does not gain governance/root authority by leadership alone.
7. Governance cryptographic approval and consensus replication quorum remain distinct and both are required where applicable.
8. Stale leaders/workers cannot perform external side effects after losing authority; fencing tokens prevent them.
9. Consensus state remains small; bulk data, user messages, attachments, and mailbox ciphertext stay outside consensus logs.
10. Total quorum-loss recovery creates a new recovery epoch so stale clusters cannot rejoin and split-brain.
11. Standard Raft is treated as crash-fault tolerant, not Byzantine-fault tolerant.
12. Consensus observability, membership, and node IDs are infrastructure-only and never become user identity or routing metadata.
```

---

# 385. Initial Production Scope

Implement first:

```text
domain-scoped consensus registry
reviewed Raft implementation
3/5-node voter profiles
learners
joint-consensus membership
authenticated QUIC/TLS transport
durable log + snapshot
linearizable reads where required
fencing tokens
recovery epoch
deployment/attestation integration
event-outbox after commit
privacy-safe metrics
deterministic consensus testkit
```

Then add:

```text
multi-region placement optimizer
formal TLA+ membership/recovery models
Jepsen-style linearizability harness
optional BFT evaluation for selected governance domains
advanced quorum placement automation
```

---

# 386. Definition of Done

Part 76 is complete when:

- consensus use-cases are explicitly bounded
- user messaging/data-plane traffic is outside global consensus
- each consensus domain has its own group/threat model
- leader election, term, log, commit, and read semantics are defined
- membership changes use learners + joint consensus
- quorum loss safely blocks writes
- stale leaders are fenced
- governance authority and consensus quorum are separated
- snapshots/compaction/restore are anti-rollback safe
- total quorum-loss recovery uses a new recovery epoch
- mixed-version upgrade and feature activation are explicit
- federation remains independent-domain, not global consensus
- Raft's non-Byzantine threat model is stated
- partition/crash/membership/snapshot/fuzz/formal tests are specified

---

# 387. Final Architecture

```text
                AUTHORITATIVE DOMAIN COMMAND
                             │
                             ▼
                    COMMAND VALIDATION
                             │
                             ▼
                      RAFT-LIKE CONSENSUS
                             │
                ┌────────────┼────────────┐
                │            │            │
              Voter        Voter        Voter
                │            │            │
                └────────────┼────────────┘
                             ▼
                        COMMIT INDEX
                             │
                             ▼
                 REPLICATED STATE MACHINE
                             │
                             ▼
                    POST-COMMIT OUTBOX
```

Consensus safety model:

```text
small scoped quorum
+
deterministic state machine
+
authenticated membership
+
joint consensus
+
fencing
+
anti-rollback recovery
+
no global user-data consensus
```

not:

```text
put every distributed problem into one giant Raft cluster
```

---

# 388. Final Principle

Consensus should be used where disagreement would be dangerous, and avoided where coordination can remain local, asynchronous, or eventually consistent.

The correct model is:

```text
strong consistency for authority
+
eventual/local coordination for user data
+
independent federation domains
+
narrow quorum scopes
```

This architecture gives SIAR strong agreement for governance, directory, control-plane, tenant, and deployment state while preserving the decentralized, privacy-preserving, local-first character of the anonymous data plane established across Parts 34–75.
