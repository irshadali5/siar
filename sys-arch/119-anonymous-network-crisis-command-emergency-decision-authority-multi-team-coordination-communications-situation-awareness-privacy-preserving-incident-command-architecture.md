# Core System Architecture Part 119 — Anonymous Network Crisis Command, Emergency Decision Authority, Multi-Team Coordination, Communications, Situation Awareness & Privacy-Preserving Incident Command Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 119  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 53, 65, 70, 94–100, 104, 107–118

**Primary purpose:** define SIAR's crisis-command and incident-command architecture for emergency decision authority, crisis roles, command handoff, multi-team coordination, situation awareness, communications, action tracking, external liaison, degraded-mode command, evidence, recovery transition, and privacy-preserving high-pressure operations.

---

# 1. Purpose

Major incidents can span:

```text
security
infrastructure
network
facility
hardware
legal/compliance
support
communications
recovery
```

A crisis-command system must answer:

```text
Who is in command?
What authority do they have?
What decisions were made?
Which teams are executing which actions?
What is known vs suspected?
How do we communicate under degraded conditions?
When does command hand off?
When does emergency authority end?
```

The governing principle is:

> **SIAR crisis command should create temporary, scoped, auditable coordination authority that improves decision quality under pressure without creating permanent superuser power, workforce surveillance, or policy bypass.**

---

# 2. Architectural Position

```text
                    INCIDENT / CRISIS
                           │
                           ▼
                    COMMAND ACTIVATION
                           │
              ┌────────────┼────────────┐
              │            │            │
          COMMAND       OPERATIONS   SITUATION
              │            │            │
              └────────────┼────────────┘
                           ▼
                  COORDINATED ACTION
                           │
              ┌────────────┼────────────┐
              │            │            │
           SECURITY      FACILITY     RECOVERY
              │            │            │
              └────────────┼────────────┘
                           ▼
                  STABILIZE / TRANSITION
```

---

# 3. Core Separation

Keep distinct:

```text
incident command
technical authority
policy authority
security authority
communications authority
legal advice
organizational hierarchy
```

---

# 4. Non-Goals

Part 119 does not create:

```text
a permanent command hierarchy
an all-powerful emergency administrator
employee productivity monitoring
private chat surveillance
automatic policy suspension
```

---

# 5. Crisis Classification

```rust
pub enum CrisisClass {
    Security,
    Availability,
    DataIntegrity,
    Privacy,
    PhysicalFacility,
    RegionalDisaster,
    SupplyChain,
    MultiDomain,
}
```

---

# 6. Crisis Severity

```rust
pub enum CrisisSeverity {
    Sev4,
    Sev3,
    Sev2,
    Sev1,
    Catastrophic,
}
```

---

# 7. Severity Must Be Operationally Defined

Hard rule.

---

# 8. Crisis Scope

```rust
pub enum CrisisScope {
    Service(ServiceId),
    Region(RegionId),
    Site(PhysicalSiteId),
    Tenant(TenantId),
    FederationDomain(FederationDomainId),
    PlatformWide,
}
```

---

# 9. No User Scope

Hard rule.

---

# 10. Command Activation

```rust
pub struct CrisisActivation {
    pub incident: IncidentId,
    pub crisis_class: CrisisClass,
    pub severity: CrisisSeverity,
    pub scope: CrisisScope,
    pub activated_at: Timestamp,
}
```

---

# 11. Activation Trigger

Examples:

```text
major security incident
region outage
facility disaster
widespread integrity failure
```

---

# 12. Hard rule.

---

# 13. Crisis Command State

```rust
pub enum CrisisCommandState {
    Monitoring,
    Activated,
    Stabilizing,
    Recovering,
    Transitioning,
    Closed,
}
```

---

# 14. No Activated→Closed Direct

Hard rule.

---

# 15. Command Roles

```rust
pub enum CrisisRole {
    IncidentCommander,
    OperationsLead,
    SecurityLead,
    FacilityLead,
    RecoveryLead,
    CommunicationsLead,
    LegalLiaison,
    CustomerSupportLead,
    Scribe,
}
```

---

# 16. Roles Are Temporary

Hard rule.

---

# 17. Role Assignment

```rust
pub struct CrisisRoleAssignment {
    pub incident: IncidentId,
    pub role: CrisisRole,
    pub principal: WorkforcePrincipalId,
    pub expires_at: Timestamp,
}
```

---

# 18. Expiry Mandatory

Hard rule.

---

# 19. Incident Commander

Coordinates decisions.

Does not automatically gain:

```text
root shell
HSM access
policy rewrite authority
database superuser
```

---

# 20. Hard rule.

---

# 21. Operations Lead

Coordinates service/infrastructure actions.

---

# 22. Security Lead

Coordinates security containment/evidence.

---

# 23. Facility Lead

Coordinates physical-site response.

---

# 24. Recovery Lead

Coordinates restoration/failback.

---

# 25. Communications Lead

Coordinates internal/external status.

---

# 26. Legal Liaison

Provides legal/compliance interface.

---

# 27. Hard Rule

Legal liaison does not gain technical control-plane authority by role alone.

---

# 28. Scribe

Maintains timeline/decision record.

---

# 29. Not Workforce Performance Logger

Hard rule.

---

# 30. Authority Model

```rust
pub struct EmergencyAuthority {
    pub incident: IncidentId,
    pub role: CrisisRole,
    pub allowed_actions: BTreeSet<EmergencyAction>,
    pub scope: CrisisScope,
    pub expires_at: Timestamp,
}
```

---

# 31. Emergency Action

```rust
pub enum EmergencyAction {
    FreezeChanges,
    HaltRollout,
    IsolateService,
    DrainRegion,
    TriggerFailover,
    RevokeCredential,
    ActivateRecoveryPlan,
    IssueStatusUpdate,
    RequestVendorEscalation,
}
```

---

# 32. No Generic "DoAnything"

Hard rule.

---

# 33. Hard Security/Privacy Invariants

Emergency authority cannot disable:

```text
encryption
authorization
tenant isolation
anonymity floor
residency constraints
signature verification
```

---

# 34. Hard rule.

---

# 35. Emergency Authority Precedence

```text
hard invariants
> safety
> incident containment
> recovery
> emergency command
> normal operations
```

---

# 36. Hard rule.

---

# 37. Separation of Duties

High-risk actions may require:

```text
incident commander + security lead
incident commander + recovery lead
dual operational approval
```

---

# 38. Hard rule.

---

# 39. Multi-Team Coordination

Teams may include:

```text
SRE
security
network
database
facility
hardware
support
legal/compliance
```

---

# 40. Coordination Unit

```rust
pub struct CrisisWorkstream {
    pub id: WorkstreamId,
    pub incident: IncidentId,
    pub owner_role: CrisisRole,
    pub objective: WorkstreamObjective,
    pub state: WorkstreamState,
}
```

---

# 41. Workstream State

```rust
pub enum WorkstreamState {
    Planned,
    Active,
    Blocked,
    Completed,
    Cancelled,
}
```

---

# 42. Hard rule.

---

# 43. Action Item

```rust
pub struct CrisisActionItem {
    pub id: ActionId,
    pub workstream: WorkstreamId,
    pub action: EmergencyAction,
    pub state: ActionState,
    pub deadline: Option<Timestamp>,
}
```

---

# 44. Action State

```rust
pub enum ActionState {
    Pending,
    Approved,
    Executing,
    Verified,
    Failed,
    Cancelled,
}
```

---

# 45. No Executing→Verified Without Evidence

Hard rule.

---

# 46. Situation Awareness

Need one consistent operational picture.

---

# 47. Situation Snapshot

```rust
pub struct SituationSnapshot {
    pub incident: IncidentId,
    pub generated_at: Timestamp,
    pub known: Vec<SituationFact>,
    pub suspected: Vec<SituationHypothesis>,
    pub unknowns: Vec<SituationUnknown>,
}
```

---

# 48. Known vs Suspected

Distinct.

---

# 49. Hard rule.

---

# 50. Situation Fact

```rust
pub struct SituationFact {
    pub statement: SituationStatement,
    pub evidence: Vec<EvidenceRef>,
    pub confidence: SituationConfidence,
}
```

---

# 51. Situation Confidence

```rust
pub enum SituationConfidence {
    Low,
    Medium,
    High,
    Confirmed,
}
```

---

# 52. No Rumor As Fact

Hard rule.

---

# 53. Unknowns

```rust
pub struct SituationUnknown {
    pub topic: SituationTopic,
    pub owner: Option<CrisisRole>,
}
```

---

# 54. Good.

---

# 55. Situation Report

Periodic SITREP.

---

# 56. SITREP Fields

```text
current state
impact
known causes
unknowns
actions in progress
risks
next decision point
```

---

# 57. Hard rule.

---

# 58. No User-Level Detail

Hard rule.

---

# 59. Crisis Timeline

Track major events.

```rust
pub struct CrisisTimelineEvent {
    pub incident: IncidentId,
    pub event_type: TimelineEventType,
    pub at: Timestamp,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 60. Timeline Events

Examples:

```text
incident declared
failover started
credential revoked
site evacuated
service restored
```

---

# 61. Not Every Chat Message

Hard rule.

---

# 62. Decision Log

```rust
pub struct CrisisDecision {
    pub incident: IncidentId,
    pub decision_id: DecisionId,
    pub decision: CrisisDecisionKind,
    pub rationale: DecisionRationale,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 63. Decision Types

```rust
pub enum CrisisDecisionKind {
    EscalateSeverity,
    FreezeChanges,
    Isolate,
    Failover,
    Rollback,
    ForwardFix,
    ActivateDR,
    CommunicateExternally,
    TransitionToRecovery,
}
```

---

# 64. No Unexplained Override

Hard rule.

---

# 65. Decision Rationale

Typed where possible.

---

# 66. Example:

```rust
pub enum DecisionRationale {
    SecurityContainment,
    AvailabilityProtection,
    DataIntegrityProtection,
    PrivacyProtection,
    FacilitySafety,
    RecoveryRequirement,
}
```

---

# 67. Communications Architecture

Need:

```text
command channel
workstream channels
status channel
external liaison
```

---

# 68. Crisis Communication Channel

```rust
pub enum CrisisChannelClass {
    Command,
    TechnicalWorkstream,
    ExecutiveStatus,
    ExternalStatus,
    Vendor,
}
```

---

# 69. Hard rule.

---

# 70. Channel Segmentation

Security-sensitive detail may remain in restricted channel.

---

# 71. Hard rule.

---

# 72. No Single Universal Crisis Chat

Hard rule.

---

# 73. Communications Reliability

Need fallback:

```text
primary collaboration
secondary messaging
phone/voice
offline contact tree
```

---

# 74. Hard rule.

---

# 75. Communications Under Network Failure

Use local/offline mechanisms where necessary.

---

# 76. No reduction of identity verification.

---

# 77. Hard rule.

---

# 78. External Status

External messages should be:

```text
accurate
timely
non-speculative
privacy-preserving
```

---

# 79. Hard rule.

---

# 80. No Premature Attribution

Hard rule.

---

# 81. Public Status Data

May include:

```text
affected services
region-level impact
workaround
recovery progress
```

---

# 82. Not:

```text
internal topology
individual users
security secrets
```

---

# 83. Hard rule.

---

# 84. Vendor Escalation

Vendors receive minimum required technical context.

---

# 85. No user content.

---

# 86. Hard rule.

---

# 87. Regulator/Authority Liaison

Handled through approved legal/compliance process.

---

# 88. Crisis system records request/response evidence.

---

# 89. No backdoor.

---

# 90. Hard rule.

---

# 91. Emergency Change Freeze

Part 107.

Routine changes freeze.

---

# 92. Allowed:

```text
security containment
recovery
incident-critical fixes
```

---

# 93. Hard rule.

---

# 94. Emergency Change Approval

```rust
pub struct EmergencyChangeApproval {
    pub incident: IncidentId,
    pub change: ChangeId,
    pub reason: EmergencyReasonCode,
    pub expires_at: Timestamp,
}
```

---

# 95. Hard rule.

---

# 96. No Automatic Broad Emergency Approval

Hard rule.

---

# 97. Escalation

Severity can increase.

---

# 98. Escalation Rule

```rust
pub struct CrisisEscalationRule {
    pub from: CrisisSeverity,
    pub to: CrisisSeverity,
    pub trigger: EscalationTrigger,
}
```

---

# 99. Escalation Trigger

```rust
pub enum EscalationTrigger {
    MultiRegionImpact,
    DataIntegrityRisk,
    SecurityCompromise,
    PhysicalSafetyIssue,
    RecoveryFailure,
    UnknownBlastRadius,
}
```

---

# 100. Hard rule.

---

# 101. De-Escalation

Requires evidence.

---

# 102. No severity downgrade for convenience.

---

# 103. Hard rule.

---

# 104. Command Handoff

Incidents may span shifts.

---

# 105. Handoff Record

```rust
pub struct CommandHandoff {
    pub incident: IncidentId,
    pub from: WorkforcePrincipalId,
    pub to: WorkforcePrincipalId,
    pub at: Timestamp,
    pub situation_snapshot: SituationSnapshotId,
}
```

---

# 106. Hard rule.

---

# 107. Handoff Checklist

```text
current state
open decisions
active actions
risks
unknowns
next checkpoints
```

---

# 108. Hard rule.

---

# 109. No Informal Silent Handoff

Hard rule.

---

# 110. Shift Fatigue

Crisis architecture supports rotation.

---

# 111. No assumption one person remains commander indefinitely.

---

# 112. Hard rule.

---

# 113. Succession

Predefined alternate roles.

---

# 114. Hard rule.

---

# 115. Command Quorum

For selected catastrophic actions.

Examples:

```text
global certificate revocation
root trust emergency
platform-wide shutdown
```

---

# 116. Hard rule.

---

# 117. No Single-Person Catastrophic Action Where Avoidable

Hard rule.

---

# 118. Crisis Command Under Partition

Control plane may be split.

---

# 119. Regional autonomy can execute preapproved local actions.

---

# 120. Hard rule.

---

# 121. No Split-Brain Command

Command epochs.

```rust
pub struct CrisisCommandEpoch(pub u64);
```

---

# 122. Higher epoch supersedes lower.

---

# 123. Hard rule.

---

# 124. Command Lease

```rust
pub struct CrisisCommandLease {
    pub incident: IncidentId,
    pub epoch: CrisisCommandEpoch,
    pub commander: WorkforcePrincipalId,
    pub expires_at: Timestamp,
}
```

---

# 125. Prevent stale commander actions.

---

# 126. Hard rule.

---

# 127. Offline Command

Use signed local emergency policy.

---

# 128. Scope limited.

---

# 129. Hard rule.

---

# 130. Crisis Identity

Use strong workforce authentication.

---

# 131. No shared emergency accounts.

---

# 132. Hard rule.

---

# 133. Device Trust

Command actions from managed/approved devices where possible.

---

# 134. Hard rule.

---

# 135. Command Session

Short-lived.

---

# 136. Hard rule.

---

# 137. Break Glass

Part 104/117.

Break glass may provide temporary authority.

---

# 138. Cannot override hard security/privacy invariant.

---

# 139. Hard rule.

---

# 140. Situation Data Classification

```rust
pub enum CrisisInformationClass {
    Public,
    Internal,
    Restricted,
    SecuritySensitive,
}
```

---

# 141. Hard rule.

---

# 142. Security-Sensitive Data

Examples:

```text
exploit detail
key compromise
internal topology
facility vulnerability
```

---

# 143. Restricted distribution.

---

# 144. Hard rule.

---

# 145. Privacy-Sensitive Data

Avoid collecting unless necessary.

---

# 146. Hard rule.

---

# 147. Crisis Notes

Do not paste private user content.

---

# 148. Use references/evidence handles.

---

# 149. Hard rule.

---

# 150. Evidence Preservation

Part 94/96.

Preserve:

```text
decision records
state transitions
change receipts
incident evidence
```

---

# 151. Hard rule.

---

# 152. No Overcollection

Incident command does not justify unlimited logging.

---

# 153. Hard rule.

---

# 154. Crisis Dashboards

Show:

```text
services
regions
sites
workstreams
actions
risks
unknowns
```

---

# 155. Not employee activity heatmaps.

---

# 156. Hard rule.

---

# 157. Situation Awareness Data Sources

Allowed:

```text
SLO state
capacity
inventory
site/facility state
security alerts
incident evidence
```

---

# 158. Forbidden:

```text
private messages
user social graph
employee productivity analytics
```

---

# 159. Hard rule.

---

# 160. Crisis Data Freshness

Every snapshot has timestamp/freshness.

---

# 161. Stale data marked.

---

# 162. Hard rule.

---

# 163. Conflicting Evidence

Represent explicitly.

---

# 164. No forced consensus.

---

# 165. Hard rule.

---

# 166. Crisis Assumption

```rust
pub struct CrisisAssumption {
    pub statement: SituationStatement,
    pub confidence: SituationConfidence,
    pub expires_at: Option<Timestamp>,
}
```

---

# 167. Expire stale assumptions.

---

# 168. Hard rule.

---

# 169. Action Verification

Each action requires result evidence.

---

# 170. Hard rule.

---

# 171. Failed Action

Does not vanish from board.

---

# 172. Hard rule.

---

# 173. Dependency Between Actions

```rust
pub struct CrisisActionDependency {
    pub action: ActionId,
    pub depends_on: ActionId,
}
```

---

# 174. No cycles.

---

# 175. Hard rule.

---

# 176. Critical Path In Crisis

Identify actions that block recovery.

---

# 177. Good.

---

# 178. Time-To-Decision

Track for process improvement.

---

# 179. Not operator performance ranking.

---

# 180. Hard rule.

---

# 181. Coordination Load

Too many participants can hurt command.

---

# 182. Roles/channels constrain fanout.

---

# 183. Hard rule.

---

# 184. Single Source Of Operational Truth

Situation board is canonical summary.

---

# 185. Not every chat thread.

---

# 186. Hard rule.

---

# 187. War-Room Pattern

Possible.

---

# 188. But structured state remains outside transient chat.

---

# 189. Hard rule.

---

# 190. Crisis Communications Retention

Retain only required evidence.

---

# 191. Hard rule.

---

# 192. External Communications Approval

High-severity external messages may require:

```text
communications lead
incident commander
legal liaison
```

---

# 193. Hard rule.

---

# 194. Security Disclosure

Part 98/97.

Coordinate vulnerability disclosure carefully.

---

# 195. No premature technical detail.

---

# 196. Hard rule.

---

# 197. Customer Support Coordination

Support gets:

```text
known impact
workaround
status
```

---

# 198. Not security-sensitive internals.

---

# 199. Hard rule.

---

# 200. Federation Coordination

Part 58.

Peer domains receive scoped status.

---

# 201. No internal topology dump.

---

# 202. Hard rule.

---

# 203. Multi-Region Command

Regional subcommands may exist.

---

# 204. Each has scoped authority.

---

# 205. Hard rule.

---

# 206. Parent / Regional Command

```rust
pub struct CrisisCommandHierarchy {
    pub parent_incident: IncidentId,
    pub regional_commands: BTreeMap<RegionId, IncidentId>,
}
```

---

# 207. No unlimited delegation.

---

# 208. Hard rule.

---

# 209. Delegation

Capabilities are attenuated.

---

# 210. Hard rule.

---

# 211. Crisis Resource Prioritization

Use Part 101 work classes.

---

# 212. Emergency command cannot mark arbitrary business workload as security critical.

---

# 213. Hard rule.

---

# 214. Logistics Coordination

Part 118.

Track:

```text
spares
temporary sites
vendor dispatch
```

---

# 215. No courier/person tracking beyond shipment status.

---

# 216. Hard rule.

---

# 217. Facility Coordination

Part 116–118.

Facility lead handles site safety/state.

---

# 218. Incident commander does not override fire/life-safety systems.

---

# 219. Hard rule.

---

# 220. Security Coordination

Part 96–97.

Security containment remains scoped.

---

# 221. Hard rule.

---

# 222. Recovery Coordination

Part 100/118.

Recovery lead manages:

```text
restore
failover
requalification
failback
```

---

# 223. Hard rule.

---

# 224. Change Coordination

Part 107.

All emergency technical changes still get change IDs/receipts.

---

# 225. Hard rule.

---

# 226. Release Coordination

Part 108.

Routine launches frozen during Sev1/Catastrophic crisis unless explicitly incident-critical.

---

# 227. Hard rule.

---

# 228. SLO Integration

Part 109.

Situation board shows reliability impact.

---

# 229. Hard rule.

---

# 230. Resilience Integration

Part 110.

Command uses fault-domain/blast-radius analysis.

---

# 231. Hard rule.

---

# 232. Forecast/Capacity Integration

Parts 101/111.

Command uses current headroom/failover reserve.

---

# 233. Hard rule.

---

# 234. Performance Integration

Part 112.

Recovery may have degraded performance.

---

# 235. Hard rule.

---

# 236. Sustainability/Efficiency

Parts 113–114.

Emergency operations can temporarily consume more energy/cost.

---

# 237. Security/recovery first.

---

# 238. Hard rule.

---

# 239. Hardware/Facility Integration

Parts 115–118.

Physical incidents feed command state.

---

# 240. Hard rule.

---

# 241. Organizational Governance

Part 104.

Crisis roles mapped from prepared on-call/escalation structures.

---

# 242. No permanent role inheritance.

---

# 243. Hard rule.

---

# 244. Audit Integration

Part 94.

Audit:

```text
command activated
role assigned
break glass
major decision
authority expiration
command closed
```

---

# 245. Not every chat message.

---

# 246. Hard rule.

---

# 247. Compliance Integration

Part 95.

Controls may require:

```text
decision evidence
SoD
timely escalation
role expiry
post-crisis review
```

---

# 248. Hard rule.

---

# 249. Crisis Closure

Only after:

```text
stabilized
recovery ownership assigned
emergency authority revoked
open actions transferred
```

---

# 250. Hard rule.

---

# 251. Transition To Recovery

```rust
pub struct CrisisRecoveryTransition {
    pub incident: IncidentId,
    pub open_actions: Vec<ActionId>,
    pub recovery_owner: CrisisRole,
    pub transitioned_at: Timestamp,
}
```

---

# 252. No Silent Disappearance

Hard rule.

---

# 253. Emergency Authority Revocation

Automatic on:

```text
expiry
role removal
incident closure
```

---

# 254. Hard rule.

---

# 255. Post-Crisis Review

Review:

```text
decision quality
timing
coordination
tooling
runbooks
unknowns
```

---

# 256. No Blame Scoring

Hard rule.

---

# 257. Learning Outputs

Feed:

```text
runbooks
tests
automation
role training
simulations
```

---

# 258. Hard rule.

---

# 259. Crisis Exercise

Tabletop and simulated command.

---

# 260. Exercise Types

```rust
pub enum CrisisExerciseType {
    Tabletop,
    Functional,
    MultiTeamSimulation,
    RegionalSimulation,
}
```

---

# 261. No real user data needed.

---

# 262. Hard rule.

---

# 263. Exercise Evaluation

Assess:

```text
activation
handoff
decision logs
communication
authority expiry
```

---

# 264. Not employee ranking.

---

# 265. Hard rule.

---

# 266. Incident Command API

```rust
pub trait CrisisCommandService {
    fn activate(
        &self,
        activation: CrisisActivation,
    ) -> Result<CrisisCommandReceipt, CrisisCommandError>;

    fn assign_role(
        &self,
        assignment: CrisisRoleAssignment,
    ) -> Result<(), CrisisCommandError>;
}
```

---

# 267. Situation Service

```rust
pub trait SituationAwarenessService {
    fn snapshot(
        &self,
        incident: IncidentId,
    ) -> Result<SituationSnapshot, CrisisCommandError>;
}
```

---

# 268. Decision Service

```rust
pub trait CrisisDecisionService {
    fn record(
        &self,
        decision: CrisisDecision,
    ) -> Result<DecisionReceipt, CrisisCommandError>;
}
```

---

# 269. Command Handoff Service

```rust
pub trait CrisisHandoffService {
    fn handoff(
        &self,
        handoff: CommandHandoff,
    ) -> Result<(), CrisisCommandError>;
}
```

---

# 270. No Generic Emergency Root API

Hard rule.

---

# 271. Error Taxonomy

```rust
pub enum CrisisCommandError {
    IncidentUnknown,
    CommandNotActive,
    AuthorityExpired,
    ScopeViolation,
    DualControlRequired,
    SituationDataStale,
    ConflictingEvidence,
    InvalidStateTransition,
    HandoffIncomplete,
    Unauthorized,
    Internal,
}
```

---

# 272. Observability

Safe metrics:

```text
active crises
severity
workstream status
open actions
handoff freshness
decision backlog
```

---

# 273. Forbidden:

```text
employee activity ranking
continuous attendance
private message content
user behavior
```

---

# 274. Hard rule.

---

# 275. Crisis Command SLOs

Examples:

```text
Sev1 commander assigned within target
situation snapshot refreshed within target
handoff completed before lease expiry
emergency authority revoked after closure
```

---

# 276. Security SLO

```text
0 expired command lease accepted
0 emergency role creates universal technical authority
0 hard privacy/security invariant bypass
```

---

# 277. Privacy SLO

```text
0 crisis command workforce-surveillance dashboards
0 private user content in situation board by default
```

---

# 278. Failure Modes

```text
commander unreachable
split-brain command
communication outage
stale situation data
authority not revoked
```

---

# 279. Commander Unreachable

Use succession/handoff policy.

---

# 280. Hard rule.

---

# 281. Split-Brain Command

Command epoch/lease prevents stale authority.

---

# 282. Hard rule.

---

# 283. Communication Outage

Fallback channels.

---

# 284. Local signed policy.

---

# 285. Hard rule.

---

# 286. Stale Situation Data

Mark stale/unknown.

---

# 287. No green default.

---

# 288. Hard rule.

---

# 289. Authority Not Revoked

Automatic expiry.

---

# 290. Hard rule.

---

# 291. Testing

Need crisis-command testkit.

---

# 292. Test Scenarios

```text
regional outage
security breach
facility disaster
multi-team incident
network-partitioned command
```

---

# 293. Activation Test

Critical incident creates command state.

---

# 294. Role Test

Assignments expire.

---

# 295. Authority Test

Action outside scope denied.

---

# 296. SoD Test

Catastrophic action requires dual approval.

---

# 297. Hard-Invariant Test

Emergency command cannot disable anonymity/encryption.

---

# 298. Situation Test

Suspected facts stay distinct.

---

# 299. Timeline Test

Only meaningful events persisted.

---

# 300. Handoff Test

Stale commander lease rejected.

---

# 301. Split-Brain Test

Higher command epoch wins.

---

# 302. Offline Test

Regional preapproved actions remain bounded.

---

# 303. Communication Test

Fallback channel usable.

---

# 304. Privacy Test

No employee activity timeline derived.

---

# 305. Closure Test

Authority revoked on incident closure.

---

# 306. Recovery Test

Open actions transfer explicitly.

---

# 307. Fuzzing

Fuzz:

```text
role assignments
command leases
decision records
situation snapshots
handoff records
```

---

# 308. Property Tests

Properties:

```text
expired command authority can never authorize action
crisis command can never grant a capability outside incident scope
suspected situation evidence can never silently become confirmed fact
incident closure can never leave active emergency authority
```

---

# 309. Formal Verification Targets

Strong candidates:

```text
command lease/epoch state machine
emergency authority scope
handoff
closure/revocation
```

---

# 310. Kani Candidate

authority-expiry and scope invariants.

---

# 311. TLA+ Candidate

activate → assign → coordinate → handoff → recover → close.

---

# 312. Loom Candidate

concurrent command handoff + authority expiry + action execution.

---

# 313. Performance

Crisis command is control-plane work.

---

# 314. Situation board must remain available under degraded conditions.

---

# 315. Hard rule.

---

# 316. Local Caching

Use bounded signed cached command state for outage resilience.

---

# 317. Hard rule.

---

# 318. Storage

Separate:

```text
crisis activations
role assignments
command leases
situation snapshots
decision records
action items
handoffs
closure records
```

---

# 319. No workforce-surveillance warehouse.

---

# 320. Hard rule.

---

# 321. Partitioning

By:

```text
incident
region
site
service
workstream
```

---

# 322. No user/person analytics partition.

---

# 323. Hard rule.

---

# 324. Crate Layout

Recommended:

```text
crates/
├── siar-crisis-core/
├── siar-crisis-authority/
├── siar-crisis-command/
├── siar-crisis-situation/
├── siar-crisis-workstreams/
├── siar-crisis-communications/
├── siar-crisis-handoff/
├── siar-crisis-evidence/
├── siar-crisis-observability/
└── siar-crisis-testkit/
```

---

# 325. `siar-crisis-core`

Owns:

```text
CrisisClass
CrisisSeverity
CrisisCommandState
CrisisCommandError
```

---

# 326. `siar-crisis-authority`

Temporary scoped emergency capabilities/leases.

---

# 327. `siar-crisis-command`

Activation, commander assignment, epoch, lifecycle.

---

# 328. `siar-crisis-situation`

Facts/hypotheses/unknowns/SITREP.

---

# 329. `siar-crisis-workstreams`

Actions/dependencies/verification.

---

# 330. `siar-crisis-communications`

Command/status/vendor/external channel policy.

---

# 331. `siar-crisis-handoff`

Shift succession and state transfer.

---

# 332. `siar-crisis-evidence`

Decision/timeline/closure evidence.

---

# 333. `siar-crisis-observability`

Aggregate command health only.

---

# 334. `siar-crisis-testkit`

authority/handoff/partition/privacy tests.

---

# 335. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Crisis authority is temporary, scoped, purpose-bound, expiring, and attached to an incident; no emergency role creates a permanent platform superuser.
2. Hard security/privacy/safety invariants remain above emergency command authority and cannot be disabled merely because the incident is severe.
3. Crisis roles coordinate decisions but do not automatically inherit root shell, HSM custody, database superuser, physical access, or policy-rewrite capabilities.
4. Major destructive or platform-wide actions require explicit separation of duties or dual approval where policy requires it.
5. Situation awareness preserves the distinction between confirmed facts, hypotheses, unknowns, and stale evidence; uncertainty can never be silently converted into certainty.
6. Crisis communication is segmented by sensitivity and purpose and cannot become a universal unrestricted chat or evidence dump containing private user content.
7. Command epochs, leases, expiry, succession, and explicit handoffs prevent stale or split-brain command authority during long incidents or network partitions.
8. Emergency changes remain identifiable governed changes with exact scope and evidence; crisis mode cannot turn ad hoc shell commands into an accepted control plane.
9. Crisis dashboards, timelines, handoffs, and metrics track incident state, actions, decisions, services, regions, and sites—not employee productivity, attendance, movement, or user behavior.
10. Emergency authority, break-glass access, and crisis roles are revoked automatically when their lease expires or the crisis closes.
11. Recovery transition explicitly transfers remaining work, evidence, and ownership before command closure; emergency command cannot disappear while unresolved actions remain ownerless.
12. Crisis command integrates with incident response, SOC, DR, facility disaster management, physical security, release/change governance, SLOs, resilience, capacity, compliance, and organizational governance without creating an alternate authority hierarchy that bypasses platform trust.
```

---

# 336. Initial Production Scope

Implement first:

```text
typed CrisisClass/CrisisSeverity/CrisisCommandState
command activation
temporary role assignments
scoped EmergencyAuthority
command leases/epochs
dual-control catastrophic actions
workstreams/actions/dependencies
known/suspected/unknown situation model
SITREP snapshots
decision log
meaningful-event timeline
command handoff
fallback communications policy
emergency change freeze
external/vendor communication policy
closure/recovery transition
automatic authority expiry
audit/compliance integration
privacy-safe crisis dashboards
crisis-command testkit
```

Then add:

```text
multi-region subcommands
offline regional command attestations
cross-federation crisis coordination
command simulation exercises
automated SITREP synthesis from typed evidence
formal split-brain-command verification
```

---

# 337. Definition of Done

Part 119 is complete when:

- crisis command activation is explicit;
- roles are temporary and scoped;
- incident commander is not a universal superuser;
- hard invariants remain above emergency authority;
- dual control exists for catastrophic actions;
- situation facts/hypotheses/unknowns are distinct;
- SITREP, timeline, and decisions are structured;
- command handoff/lease/epoch prevents stale authority;
- communications are segmented;
- offline/degraded command remains bounded;
- closure revokes emergency power automatically;
- unresolved work transfers to recovery;
- no workforce/user surveillance is created;
- authority/handoff/privacy/fuzz/formal tests are specified.

---

# 338. Final Architecture

```text
                    MAJOR INCIDENT
                          │
                          ▼
                  CRISIS ACTIVATION
                          │
             ┌────────────┼────────────┐
             │            │            │
          COMMAND       SITUATION    WORKSTREAMS
             │            │            │
             └────────────┼────────────┘
                          ▼
                 COORDINATED DECISIONS
                          │
             ┌────────────┼────────────┐
             │            │            │
         SECURITY      FACILITY      RECOVERY
             │            │            │
             └────────────┼────────────┘
                          ▼
                    STABILIZATION
                          │
                          ▼
                 RECOVERY TRANSITION
                          │
                          ▼
                       CLOSURE
```

Crisis-command safety model:

```text
temporary scoped authority
+
role separation
+
decision evidence
+
known/suspected/unknown distinction
+
command leases and handoff
+
segmented communications
+
automatic authority expiry
+
privacy-minimized situation awareness
```

not:

```text
give one commander permanent root access, dump every employee/user activity into a war-room dashboard, and suspend policy until the crisis is over
```

---

# 339. Final Principle

Crisis command should make authority clearer and narrower under pressure, not broader and less accountable.

The correct model is:

```text
activate command explicitly
+
assign temporary roles
+
separate fact from hypothesis
+
coordinate workstreams
+
record decisions
+
scope emergency authority
+
handoff cleanly
+
transition to recovery
+
revoke power automatically
+
never turn crisis operations into surveillance
```

This architecture gives SIAR a privacy-preserving incident-command foundation for crisis activation, emergency authority, multi-team coordination, situation awareness, communications, handoff, degraded-mode command, recovery transition, and command closure while preserving the anonymity, local-first, least-authority, operational-governance, facility-continuity, and anti-surveillance guarantees established across Parts 34–118.
