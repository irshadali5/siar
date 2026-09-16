# Core System Architecture Part 104 — Anonymous Network Service Ownership, Operational Responsibility, Escalation, On-Call, Change Authority & Privacy-Preserving Organizational Governance Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 104  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 51, 53, 61–65, 69–70, 79–83, 94–103

**Primary purpose:** define SIAR's organizational-governance architecture for service ownership, operational responsibility, on-call duty, escalation, change authority, separation of duties, emergency authority, maintenance ownership, handoff, accountability, tenant/federation organizational boundaries, and privacy-preserving staffing metadata without creating employee-surveillance, hidden social graphs, or permanent privilege concentration.

---

# 1. Purpose

Large distributed systems fail operationally when nobody can answer:

```text
Who owns this service?
Who may approve this change?
Who responds at 03:00?
Who can declare an incident?
Who can rotate this key?
Who decides when to fail over?
Who verifies the fix?
```

The governing principle is:

> **SIAR operational governance must make responsibility explicit, authority scoped, escalation deterministic, and accountability verifiable—without turning organization metadata into an employee-monitoring or social-graph system.**

---

# 2. Architectural Position

```text
Service / Domain / Control
           │
           ▼
      Ownership Registry
           │
           ▼
  Responsibility Assignment
           │
      ┌────┼─────┐
      │    │     │
   OnCall Change Escalation
      │    │     │
      └────┼─────┘
           ▼
     Scoped Authority
           │
           ▼
   Audit / Review / Handoff
```

---

# 3. Core Separation

Keep distinct:

```text
ownership
operational responsibility
approval authority
execution authority
on-call duty
incident command
audit accountability
```

---

# 4. Non-Goals

Part 104 does not create:

```text
employee productivity monitoring
presence surveillance
keystroke/logon tracking
internal social graphs
universal administrator roles
```

---

# 5. Service Ownership

Every production service has explicit ownership.

---

# 6. Service Owner

```rust
pub struct ServiceOwnerRef(pub String);
```

---

# 7. Prefer Role/Team Reference

Examples:

```text
identity-platform
relay-operations
storage-reliability
security-infrastructure
```

---

# 8. Avoid Personal Identity In Core Runtime

Hard rule.

---

# 9. Service Ownership Record

```rust
pub struct ServiceOwnership {
    pub service: ServiceId,
    pub owner: ServiceOwnerRef,
    pub backup_owner: Option<ServiceOwnerRef>,
    pub escalation_policy: EscalationPolicyId,
}
```

---

# 10. Ownership Is Not Authority By Itself

Hard rule.

---

# 11. Ownership Means Accountability

Authority remains capability/policy based.

---

# 12. Responsibility Domains

```rust
pub enum ResponsibilityDomain {
    ServiceOperation,
    Security,
    Privacy,
    Data,
    Release,
    Recovery,
    Compliance,
    Cost,
    Capacity,
    Federation,
}
```

---

# 13. Responsibility Assignment

```rust
pub struct ResponsibilityAssignment {
    pub scope: ResponsibilityScope,
    pub domain: ResponsibilityDomain,
    pub owner: ServiceOwnerRef,
}
```

---

# 14. Responsibility Scope

```rust
pub enum ResponsibilityScope {
    Service(ServiceId),
    Tenant(TenantId),
    Region(RegionId),
    FederationPeer(FederationDomainId),
    GovernanceDomain(GovernanceDomainId),
}
```

---

# 15. No Global User Scope

Hard rule.

---

# 16. RACI-Like Model

Useful internally.

---

# 17. Responsibility Role

```rust
pub enum ResponsibilityRole {
    Accountable,
    Responsible,
    Consulted,
    Informed,
}
```

---

# 18. Accountable

One clear accountable role preferred.

---

# 19. Responsible

Execution role.

---

# 20. Consulted

Input.

---

# 21. Informed

Notification only.

---

# 22. Hard Rule

Accountability and execution may be separated.

---

# 23. No Ambiguous Shared Accountability

Preferred hard rule.

---

# 24. Operational Role

```rust
pub enum OperationalRole {
    ServiceOperator,
    OnCallPrimary,
    OnCallSecondary,
    ChangeApprover,
    ReleaseApprover,
    IncidentCommander,
    SecurityReviewer,
    PrivacyReviewer,
    RecoveryApprover,
}
```

---

# 25. Roles Are Scoped

Hard rule.

---

# 26. Role Assignment

```rust
pub struct OperationalRoleAssignment {
    pub role: OperationalRole,
    pub scope: ResponsibilityScope,
    pub principal: WorkforcePrincipalRef,
    pub valid_from: Timestamp,
    pub valid_until: Option<Timestamp>,
}
```

---

# 27. Workforce Principal

Opaque internal identity.

---

# 28. Not user/account identity.

---

# 29. Hard rule.

---

# 30. Principal Separation

Keep separate:

```text
workforce identity
customer identity
messaging identity
admin identity
federation identity
```

---

# 31. Hard rule.

---

# 32. On-Call Model

On-call is operational duty.

---

# 33. OnCallRotation

```rust
pub struct OnCallRotation {
    pub rotation_id: RotationId,
    pub scope: ResponsibilityScope,
    pub primary: RotationSchedule,
    pub secondary: Option<RotationSchedule>,
}
```

---

# 34. Schedule Metadata

Minimal.

---

# 35. No GPS/Presence Tracking

Hard rule.

---

# 36. On-Call State

```rust
pub enum OnCallState {
    Scheduled,
    Active,
    HandoffPending,
    Completed,
}
```

---

# 37. Active

Current duty window.

---

# 38. HandoffPending

Transfer not yet acknowledged.

---

# 39. Completed

Duty ended.

---

# 40. Hard Rule

On-call schedules do not imply real-time employee presence tracking.

---

# 41. Duty Window

```rust
pub struct DutyWindow {
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
}
```

---

# 42. Explicit.

---

# 43. No indefinite on-call assignment.

---

# 44. Hard rule.

---

# 45. On-Call Handoff

Must be explicit.

---

# 46. Handoff Record

```rust
pub struct OnCallHandoff {
    pub from: WorkforcePrincipalRef,
    pub to: WorkforcePrincipalRef,
    pub scope: ResponsibilityScope,
    pub acknowledged_at: Timestamp,
}
```

---

# 47. No silent handoff.

---

# 48. Hard rule.

---

# 49. Escalation Architecture

Escalation should be deterministic.

---

# 50. Escalation Policy

```rust
pub struct EscalationPolicy {
    pub id: EscalationPolicyId,
    pub steps: Vec<EscalationStep>,
}
```

---

# 51. Escalation Step

```rust
pub struct EscalationStep {
    pub delay: Duration,
    pub target: EscalationTarget,
}
```

---

# 52. Escalation Target

```rust
pub enum EscalationTarget {
    OnCallPrimary,
    OnCallSecondary,
    ServiceOwner,
    SecurityDuty,
    PrivacyDuty,
    IncidentCommander,
}
```

---

# 53. No arbitrary human contact graph.

---

# 54. Hard rule.

---

# 55. Escalation Triggers

Examples:

```text
critical alert unacknowledged
incident severity escalation
failed change
regional outage
security control failure
```

---

# 56. No user behavior triggers.

---

# 57. Hard rule.

---

# 58. Escalation State

```rust
pub enum EscalationState {
    Pending,
    Acknowledged,
    Escalated,
    Resolved,
}
```

---

# 59. Acknowledgement

Confirms receipt, not resolution.

---

# 60. Hard rule.

---

# 61. Change Authority

Every risky change has explicit authority requirements.

---

# 62. Change Class

```rust
pub enum ChangeClass {
    Routine,
    Standard,
    HighRisk,
    Emergency,
}
```

---

# 63. Routine

Low-risk reversible.

---

# 64. Standard

Normal production change.

---

# 65. HighRisk

Security/privacy/data-impacting.

---

# 66. Emergency

Urgent incident/continuity change.

---

# 67. Change Request

```rust
pub struct ChangeRequest {
    pub change_id: ChangeId,
    pub scope: ResponsibilityScope,
    pub class: ChangeClass,
    pub artifact_digest: Option<ArtifactDigest>,
    pub policy_digest: Option<Digest>,
}
```

---

# 68. No Free-Form Shell Change In Core

Hard rule.

---

# 69. Change Authority Policy

```rust
pub struct ChangeAuthorityPolicy {
    pub class: ChangeClass,
    pub required_roles: Vec<OperationalRole>,
    pub approval_threshold: ApprovalThreshold,
}
```

---

# 70. High-Risk Change

May require:

```text
service approver
security reviewer
privacy reviewer
```

---

# 71. Hard rule.

---

# 72. Separation Of Duties

Critical.

---

# 73. Example

Same principal should not:

```text
author
approve
execute
```

high-risk root-key change.

---

# 74. Separation Rule

```rust
pub struct SeparationOfDutiesRule {
    pub action: GovernanceActionClass,
    pub forbidden_role_combinations: Vec<RoleCombination>,
}
```

---

# 75. Hard rule.

---

# 76. Change Approval

```rust
pub struct ChangeApproval {
    pub change: ChangeId,
    pub approver_role: OperationalRole,
    pub principal: WorkforcePrincipalRef,
    pub decision: ApprovalDecision,
    pub approved_at: Timestamp,
}
```

---

# 77. Approval Decision

```rust
pub enum ApprovalDecision {
    Approve,
    Reject,
}
```

---

# 78. No "implicit approve by silence."

Hard rule.

---

# 79. Change Execution Capability

Separate from approval.

---

# 80. Short-lived.

---

# 81. Change Capability

```rust
pub struct ChangeCapability {
    pub change: ChangeId,
    pub scope: ResponsibilityScope,
    pub allowed_actions: BTreeSet<ChangeAction>,
    pub expires_at: Timestamp,
}
```

---

# 82. No Permanent Production Write Privilege

Preferred hard rule.

---

# 83. Change Action

```rust
pub enum ChangeAction {
    Deploy,
    Rollback,
    RotateKey,
    UpdatePolicy,
    Failover,
    Restore,
    Quarantine,
}
```

---

# 84. Typed.

---

# 85. No generic `RunCommand`.

Hard rule.

---

# 86. Release Authority

Part 99 integration.

---

# 87. Release promotion requires exact artifact and approvals.

---

# 88. No rebuild during approval.

---

# 89. Hard rule.

---

# 90. Policy Change Authority

Part 61/95.

---

# 91. Signed policy version.

---

# 92. Sensitive privacy/security policy requires elevated review.

---

# 93. Hard rule.

---

# 94. Key Management Authority

Part 72/80.

---

# 95. Root/high-value key changes use ceremonies.

---

# 96. No regular on-call unilateral root-key authority.

---

# 97. Hard rule.

---

# 98. Recovery Authority

Part 100.

---

# 99. Restore/failover permissions distinct.

---

# 100. Hard rule.

---

# 101. Incident Authority

Part 96.

---

# 102. Incident commander coordinates.

---

# 103. Does not automatically gain all technical privileges.

---

# 104. Hard rule.

---

# 105. Emergency Authority

Need break-glass design.

---

# 106. Break-Glass Principle

```text
short-lived
scoped
strongly authenticated
audited
reviewed
```

---

# 107. BreakGlassGrant

```rust
pub struct BreakGlassGrant {
    pub scope: ResponsibilityScope,
    pub actions: BTreeSet<ChangeAction>,
    pub reason: EmergencyReasonCode,
    pub expires_at: Timestamp,
}
```

---

# 108. No permanent emergency role.

---

# 109. Hard rule.

---

# 110. Break-Glass Limits

Cannot:

```text
disable encryption
disable authorization
disable privacy floor
create universal admin
```

---

# 111. Hard rule.

---

# 112. Emergency Reason

Typed.

---

# 113. Example:

```rust
pub enum EmergencyReasonCode {
    ActiveSecurityIncident,
    RegionalFailure,
    CriticalServiceOutage,
    KeyCompromise,
}
```

---

# 114. No generic "urgent."

Preferred hard rule.

---

# 115. Emergency Approval

Critical actions may still require second approver.

---

# 116. Even under incident.

---

# 117. Hard rule.

---

# 118. Organizational Hierarchy

Core architecture should not model full corporate org chart.

---

# 119. Model only operational responsibility graph.

---

# 120. Hard rule.

---

# 121. Why

Avoid unnecessary internal social graph.

---

# 122. Organizational Unit

```rust
pub struct OperationalUnitRef(pub String);
```

---

# 123. Examples:

```text
platform
security
storage
network
```

---

# 124. Minimal.

---

# 125. No manager-chain analytics.

---

# 126. Hard rule.

---

# 127. Workforce Privacy

Operational governance metadata can be sensitive.

---

# 128. Store only:

```text
role assignments
duty windows
approval records
```

---

# 129. Avoid:

```text
location
activity history
productivity
presence
```

---

# 130. Hard rule.

---

# 131. Workforce Metadata Retention

Bounded.

---

# 132. On-call historical schedules shorter.

---

# 133. High-risk approval history longer.

---

# 134. No lifetime employee behavior archive.

---

# 135. Hard rule.

---

# 136. Workforce Data Classification

```rust
pub enum WorkforceDataClass {
    OperationalAssignment,
    ApprovalEvidence,
    EmergencyAccess,
    ContactEndpoint,
}
```

---

# 137. Contact Endpoint

Used for alert delivery only.

---

# 138. No employee social identity enrichment.

---

# 139. Hard rule.

---

# 140. Contact Channel

Examples:

```text
work email
pager
work phone
```

---

# 141. Separate from personal contact data.

---

# 142. Hard rule.

---

# 143. On-Call Notification

Use minimum information.

---

# 144. Example:

```text
incident ID
severity
service
required action
```

---

# 145. No private user content.

---

# 146. Hard rule.

---

# 147. Escalation Privacy

Escalation logs should not become performance scoring.

---

# 148. Measure service/process quality, not employee ranking.

---

# 149. Hard rule.

---

# 150. Operational Performance Metrics

Allowed:

```text
time to acknowledge
handoff failures
coverage gaps
change rollback rate
```

---

# 151. Aggregate by service/rotation.

---

# 152. Not per-person leaderboard.

---

# 153. Hard rule.

---

# 154. Service Catalog Integration

Part 65.

---

# 155. Every service catalog entry includes:

```text
owner
backup owner
on-call rotation
runbook
SLO
dependencies
```

---

# 156. No ownerless production service.

---

# 157. Hard rule.

---

# 158. Service Lifecycle

Ownership required before production admission.

---

# 159. Service Lifecycle State

```rust
pub enum ServiceLifecycleState {
    Draft,
    PreProduction,
    Production,
    Deprecated,
    Retired,
}
```

---

# 160. Production requires valid ownership.

---

# 161. Hard rule.

---

# 162. Ownership Transfer

Explicit workflow.

---

# 163. Ownership Transfer Record

```rust
pub struct OwnershipTransfer {
    pub service: ServiceId,
    pub from: ServiceOwnerRef,
    pub to: ServiceOwnerRef,
    pub effective_at: Timestamp,
}
```

---

# 164. No silent reassignment.

---

# 165. Hard rule.

---

# 166. Transfer Checklist

```text
runbooks
alerts
dashboards
secrets ownership
release authority
capacity knowledge
DR knowledge
```

---

# 167. Handoff Complete Only After Acceptance

Hard rule.

---

# 168. Owner Departure

Role reassignment required.

---

# 169. No orphaned privileges.

---

# 170. Hard rule.

---

# 171. Role Revocation

Immediate on offboarding.

---

# 172. Old operational capabilities expire/revoke.

---

# 173. Part 83 identity lifecycle integration.

---

# 174. Hard rule.

---

# 175. Temporary Delegation

Supported.

---

# 176. Delegation Record

```rust
pub struct OperationalDelegation {
    pub from_role: OperationalRole,
    pub to_principal: WorkforcePrincipalRef,
    pub scope: ResponsibilityScope,
    pub expires_at: Timestamp,
}
```

---

# 177. Cannot grant more authority than delegator holds.

---

# 178. Hard rule.

---

# 179. No Transitive Infinite Delegation

Hard rule.

---

# 180. Delegation Depth

Bounded.

---

# 181. Example:

```text
max depth = 1 or 2
```

---

# 182. Good.

---

# 183. Approval Quorum

For high-risk organizational actions.

---

# 184. Quorum Policy

```rust
pub struct ApprovalThreshold {
    pub required: u8,
    pub eligible_roles: BTreeSet<OperationalRole>,
}
```

---

# 185. No count of duplicated same principal.

---

# 186. Hard rule.

---

# 187. Distinct Principal Requirement

Critical.

---

# 188. Approval system enforces.

---

# 189. Hard rule.

---

# 190. Change Freeze

Possible.

---

# 191. Freeze Policy

```rust
pub struct ChangeFreezePolicy {
    pub scope: ResponsibilityScope,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
    pub exceptions: BTreeSet<ChangeClass>,
}
```

---

# 192. Emergency security changes may bypass with proper authority.

---

# 193. No permanent freeze.

---

# 194. Hard rule.

---

# 195. Maintenance Ownership

Recurring maintenance needs owner.

---

# 196. Examples:

```text
key rotation
certificate renewal
backup drill
capacity test
dependency upgrade
```

---

# 197. Maintenance Task

```rust
pub struct MaintenanceResponsibility {
    pub task: MaintenanceTaskId,
    pub owner: ServiceOwnerRef,
    pub cadence: MaintenanceCadence,
}
```

---

# 198. No ownerless recurring security task.

---

# 199. Hard rule.

---

# 200. Change Window

Optional.

---

# 201. Maintenance Window

Does not override emergency/security actions.

---

# 202. Hard rule.

---

# 203. Incident Escalation

Part 96.

---

# 204. Critical incident can page:

```text
service on-call
security on-call
privacy on-call
incident commander
```

---

# 205. Based on class.

---

# 206. Hard rule.

---

# 207. Privacy Incident Escalation

Privacy reviewer first-class.

---

# 208. Not optional after confirmed privacy incident.

---

# 209. Hard rule.

---

# 210. Security Incident Escalation

Security responder first-class.

---

# 211. No purely operations-only handling for critical security breach.

---

# 212. Hard rule.

---

# 213. DR Escalation

Part 100.

---

# 214. Failover authority separated from recovery verification.

---

# 215. Good separation.

---

# 216. Hard rule.

---

# 217. Capacity Escalation

Part 101.

---

# 218. Capacity owner/service owner.

---

# 219. No autoscaler-only silent policy shift.

---

# 220. Hard rule.

---

# 221. Cost Escalation

Part 102.

---

# 222. Budget owner + service owner.

---

# 223. Finance cannot unilaterally disable required security/privacy controls.

---

# 224. Hard rule.

---

# 225. Residency Escalation

Part 103.

---

# 226. Jurisdiction conflict requires policy/legal/governance escalation.

---

# 227. On-call operator cannot improvise cross-border override.

---

# 228. Hard rule.

---

# 229. Tenant Organizational Governance

Managed tenant may define:

```text
tenant admin
tenant security admin
tenant compliance reviewer
```

---

# 230. Tenant roles scoped to tenant.

---

# 231. No access to platform-wide or personal user data.

---

# 232. Hard rule.

---

# 233. Tenant Change Authority

Managed settings only.

---

# 234. Cannot alter platform hard security/privacy floor.

---

# 235. Hard rule.

---

# 236. Federation Organizational Governance

Each federation domain governs own operators.

---

# 237. Peers exchange only domain-level authority proofs.

---

# 238. No remote employee directory.

---

# 239. Hard rule.

---

# 240. Federation Change Approval

Examples:

```text
peer trust creation
peer revocation
protocol policy
```

---

# 241. Scoped domain authority.

---

# 242. Hard rule.

---

# 243. Governance Authority

Part 53.

---

# 244. Organizational governance does not replace cryptographic governance authority.

---

# 245. Hard rule.

---

# 246. Identity Provider Integration

Part 82.

---

# 247. Workforce authentication separate from customer auth.

---

# 248. Strong MFA/passkeys.

---

# 249. Hard rule.

---

# 250. Authorization Integration

Part 81.

---

# 251. Operational roles compile to scoped capabilities.

---

# 252. Role name itself grants nothing without policy decision.

---

# 253. Hard rule.

---

# 254. Secrets Integration

Part 80.

---

# 255. On-call may request JIT secret capability.

---

# 256. Not possess standing plaintext secrets.

---

# 257. Hard rule.

---

# 258. Audit Integration

Part 94.

---

# 259. Audit:

```text
role assignment
ownership transfer
high-risk approval
break-glass grant
change execution
```

---

# 260. Not every on-call page.

---

# 261. Hard rule.

---

# 262. Compliance Integration

Part 95.

---

# 263. Controls:

```text
production services have owners
on-call coverage exists
high-risk SoD enforced
break-glass expires
```

---

# 264. Good.

---

# 265. SOC Integration

Part 97.

---

# 266. SOC roles distinct from service operator.

---

# 267. No SOC superadmin.

---

# 268. Hard rule.

---

# 269. Vulnerability Integration

Part 98.

---

# 270. Every critical exposure has remediation owner.

---

# 271. Service owner accountable for SLA.

---

# 272. Hard rule.

---

# 273. Update Integration

Part 99.

---

# 274. Release promotion authority explicit.

---

# 275. Emergency security release still signed/reviewed according policy.

---

# 276. Hard rule.

---

# 277. DR Integration

Part 100.

---

# 278. Restore/failover approvers explicit.

---

# 279. No generic operator grant.

---

# 280. Hard rule.

---

# 281. FinOps Integration

Part 102.

---

# 282. Budget owner separate from runtime operator.

---

# 283. Hard rule.

---

# 284. Geographic Governance Integration

Part 103.

---

# 285. Residency-policy approver explicit.

---

# 286. Region migration needs ownership and approval.

---

# 287. Hard rule.

---

# 288. Change Management State Machine

```rust
pub enum ChangeState {
    Draft,
    Review,
    Approved,
    Scheduled,
    Executing,
    Verifying,
    Completed,
    RolledBack,
    Rejected,
}
```

---

# 289. No Draft→Executing

Hard rule.

---

# 290. Emergency Change State

Can skip scheduling but not authorization/audit.

---

# 291. Hard rule.

---

# 292. Verification

Change completion requires verification.

---

# 293. Example:

```text
health checks
security controls
privacy controls
migration checks
```

---

# 294. No "deploy command succeeded" = completed.

---

# 295. Hard rule.

---

# 296. Change Rollback

Separate authority may be needed.

---

# 297. Emergency rollback allowed if pre-approved safe.

---

# 298. Hard rule.

---

# 299. Organizational Policy

Signed/versioned.

---

# 300. OrganizationalPolicy

```rust
pub struct OrganizationalPolicy {
    pub version: OrganizationalPolicyVersion,
    pub ownership_rules: Vec<OwnershipRule>,
    pub escalation_rules: Vec<EscalationPolicy>,
    pub change_authority: Vec<ChangeAuthorityPolicy>,
}
```

---

# 301. Anti-Rollback

Old permissive policy rejected.

---

# 302. Hard rule.

---

# 303. Policy Distribution

Part 61.

---

# 304. No manual spreadsheet as runtime authority.

---

# 305. Human docs may mirror but not replace signed policy.

---

# 306. Hard rule.

---

# 307. Organizational Registry

Stores:

```text
service ownership
role assignments
on-call rotations
escalation policies
change authority policies
```

---

# 308. Does not store:

```text
employee productivity
chat history
social connections
personal location
```

---

# 309. Hard rule.

---

# 310. Registry Access

Scoped.

---

# 311. Service teams can view relevant ownership.

---

# 312. Security/admin can view needed authority records.

---

# 313. No unrestricted org graph export.

---

# 314. Hard rule.

---

# 315. Workforce Privacy By Design

Prefer:

```text
role refs
team refs
opaque principals
```

over:

```text
rich HR profile
```

---

# 316. Hard rule.

---

# 317. On-Call Reporting

Aggregate.

---

# 318. Safe metrics:

```text
coverage gaps
ack latency
escalation count
handoff failure
```

---

# 319. No individual ranking.

---

# 320. Hard rule.

---

# 321. Change Reporting

Safe:

```text
change success rate
rollback rate
verification failure
```

---

# 322. Aggregate by service/change class.

---

# 323. No employee leaderboard.

---

# 324. Hard rule.

---

# 325. Escalation Reporting

Process improvement.

---

# 326. No punitive surveillance architecture.

---

# 327. Hard rule.

---

# 328. Shift Coverage

Need coverage validation.

---

# 329. Coverage State

```rust
pub enum CoverageState {
    Covered,
    SinglePoint,
    Gap,
}
```

---

# 330. Gap

No valid responder.

---

# 331. Alert before gap begins.

---

# 332. Hard rule.

---

# 333. Time Zones

Use explicit schedules.

---

# 334. No location inference.

---

# 335. Hard rule.

---

# 336. Follow-The-Sun

Possible.

---

# 337. Handoff by rotation schedule.

---

# 338. No workforce location tracking.

---

# 339. Hard rule.

---

# 340. Outsourced/Contractor Access

More restrictive.

---

# 341. Time-bound.

---

# 342. Scope-bound.

---

# 343. No standing production access.

---

# 344. Hard rule.

---

# 345. Third-Party Operator

Capability limited to specific service/task.

---

# 346. Audit.

---

# 347. Hard rule.

---

# 348. Human Error Controls

Reduce dependence on memory.

---

# 349. Use:

```text
typed runbooks
approval gates
safe defaults
automation
```

---

# 350. No blame-based architecture.

---

# 351. Good.

---

# 352. Operational Runbooks

Part 65.

---

# 353. Every critical service has:

```text
startup
shutdown
failover
rollback
incident
recovery
```

---

# 354. Ownership links to runbook.

---

# 355. Hard rule.

---

# 356. Runbook Authority

Runbook guides action.

---

# 357. Does not grant privilege.

---

# 358. Hard rule.

---

# 359. Change Simulation

High-risk changes can test in staging/simulation.

---

# 360. Approval sees evidence.

---

# 361. Good.

---

# 362. No production experiment as first validation.

---

# 363. Hard rule.

---

# 364. Maintenance Calendar

Operational metadata.

---

# 365. Does not need employee calendar details beyond duty window.

---

# 366. Hard rule.

---

# 367. Organizational Resilience

Avoid single human dependencies.

---

# 368. Critical domain has:

```text
primary owner
backup owner
secondary approver
documented runbook
```

---

# 369. Hard rule.

---

# 370. Bus Factor Control

Can be represented as coverage requirement.

---

# 371. Not employee scoring.

---

# 372. Hard rule.

---

# 373. Knowledge Handoff

Part 65 docs/ADRs.

---

# 374. Service ownership transfer requires docs current.

---

# 375. Hard rule.

---

# 376. Privilege Review

Periodic.

---

# 377. Review role assignments.

---

# 378. Remove stale grants.

---

# 379. No permanent inherited role.

---

# 380. Hard rule.

---

# 381. Access Recertification

High-risk roles more frequent.

---

# 382. Uses role/scope data only.

---

# 383. No productivity analytics.

---

# 384. Hard rule.

---

# 385. Organizational Incident

Example:

```text
ownerless critical service
expired on-call coverage
stale break-glass grants
```

---

# 386. Can raise governance alert.

---

# 387. Good.

---

# 388. No HR disciplinary model.

---

# 389. Hard rule.

---

# 390. Operational Governance API

```rust
pub trait OperationalGovernanceService {
    fn ownership(
        &self,
        service: ServiceId,
    ) -> Result<ServiceOwnership, OrganizationalGovernanceError>;

    fn change_authority(
        &self,
        request: &ChangeRequest,
    ) -> Result<ChangeAuthorityDecision, OrganizationalGovernanceError>;
}
```

---

# 391. On-Call API

```rust
pub trait OnCallService {
    fn active_rotation(
        &self,
        scope: ResponsibilityScope,
    ) -> Result<OnCallRotation, OrganizationalGovernanceError>;
}
```

---

# 392. Escalation API

```rust
pub trait EscalationService {
    fn escalate(
        &self,
        scope: ResponsibilityScope,
        reason: EscalationReason,
    ) -> Result<EscalationId, OrganizationalGovernanceError>;
}
```

---

# 393. Change Approval API

```rust
pub trait ChangeApprovalService {
    fn approve(
        &self,
        request: ChangeId,
        approval: ChangeApproval,
    ) -> Result<ChangeAuthorityDecision, OrganizationalGovernanceError>;
}
```

---

# 394. No General "SetAdmin"

Hard rule.

---

# 395. Change Authority Decision

```rust
pub enum ChangeAuthorityDecision {
    Approved(ChangeCapability),
    AdditionalApprovalRequired(Vec<OperationalRole>),
    Denied(ChangeDenialReason),
}
```

---

# 396. Explicit.

---

# 397. Hard rule.

---

# 398. Error Taxonomy

```rust
pub enum OrganizationalGovernanceError {
    ServiceUnowned,
    RotationMissing,
    CoverageGap,
    ApprovalMissing,
    SeparationOfDutiesViolation,
    ChangeDenied,
    BreakGlassExpired,
    DelegationInvalid,
    ScopeMismatch,
    Unauthorized,
    Internal,
}
```

---

# 399. Observability

Safe metrics:

```text
services without owner
on-call coverage gaps
approval latency
change rollback rate
break-glass count
```

---

# 400. Forbidden:

```text
employee activity history
individual performance ranking
location tracking
```

---

# 401. Hard rule.

---

# 402. Organizational Governance SLOs

Examples:

```text
0 production service without owner
0 critical on-call coverage gap
0 expired break-glass capability active
```

---

# 403. Security SLO

```text
0 high-risk change without required approvals
0 SoD bypass
```

---

# 404. Privacy SLO

```text
0 personal location tracking
0 employee behavioral profiling
0 workforce social graph export
```

---

# 405. Failure Modes

```text
owner unavailable
rotation gap
approval service outage
identity provider outage
emergency during change freeze
```

---

# 406. Owner Unavailable

Escalate to backup owner/on-call.

---

# 407. No authority broadening automatically.

---

# 408. Hard rule.

---

# 409. Rotation Gap

Alert before active gap.

---

# 410. Use backup coverage.

---

# 411. Hard rule.

---

# 412. Approval Service Outage

Routine change paused.

---

# 413. Emergency path uses signed break-glass policy.

---

# 414. Hard rule.

---

# 415. Identity Provider Outage

Existing short-lived operator credentials may continue within policy.

---

# 416. No universal password fallback.

---

# 417. Hard rule.

---

# 418. Emergency During Freeze

Break-glass/change exception.

---

# 419. Still audited/reviewed.

---

# 420. Hard rule.

---

# 421. Testing

Need organizational-governance testkit.

---

# 422. Test Scenarios

```text
owner transfer
on-call handoff
high-risk change approval
break-glass
tenant admin scope
```

---

# 423. Ownership Test

Production service without owner rejected.

---

# 424. Handoff Test

New on-call not active until acknowledged.

---

# 425. Escalation Test

Unacknowledged critical page escalates.

---

# 426. SoD Test

Same principal cannot satisfy forbidden role combination.

---

# 427. Approval Test

High-risk change blocked without threshold.

---

# 428. Break-Glass Test

Capability auto-expires.

---

# 429. Tenant Test

Tenant admin cannot change platform hard policy.

---

# 430. Federation Test

Remote domain cannot assign local workforce role.

---

# 431. Privacy Test

On-call operation needs no GPS/presence history.

---

# 432. Offboarding Test

Old role/capability revoked.

---

# 433. Delegation Test

Delegated authority cannot exceed source.

---

# 434. Freeze Test

Emergency exception works only under policy.

---

# 435. Backup Owner Test

Owner absence does not create universal admin.

---

# 436. Audit Test

High-risk ownership/authority changes recorded.

---

# 437. Policy Rollback Test

Old permissive org policy rejected.

---

# 438. Fuzzing

Fuzz:

```text
ownership record
on-call schedule
escalation policy
change approval
delegation
```

---

# 439. Property Tests

Properties:

```text
change capability can never exceed approved scope/actions
expired break-glass grant can never authorize execution
forbidden SoD combination can never satisfy approval threshold
tenant operational role can never authorize unrelated platform scope
```

---

# 440. Formal Verification Targets

Strong candidates:

```text
change approval state machine
on-call handoff
break-glass expiry
delegation attenuation
```

---

# 441. Kani Candidate

role/scope/approval invariants.

---

# 442. TLA+ Candidate

draft change → approvals → execution → verify → complete/rollback.

---

# 443. Loom Candidate

concurrent approval + revocation + execution start.

---

# 444. Performance

Governance checks are low-frequency but latency-sensitive for incidents.

---

# 445. Ownership lookup cacheable.

---

# 446. Change approval strongly consistent.

---

# 447. No global org graph traversal.

---

# 448. Hard rule.

---

# 449. Storage

Separate stores:

```text
service ownership
role assignments
on-call rotations
escalation policies
change approvals
break-glass grants
delegations
```

---

# 450. No workforce behavior warehouse.

---

# 451. Hard rule.

---

# 452. Partitioning

By:

```text
service
tenant
region
governance domain
federation domain
```

---

# 453. Not by employee behavioral profile.

---

# 454. Hard rule.

---

# 455. Crate Layout

Recommended:

```text
crates/
├── siar-org-governance-core/
├── siar-service-ownership/
├── siar-responsibility/
├── siar-oncall/
├── siar-escalation/
├── siar-change-authority/
├── siar-separation-of-duties/
├── siar-break-glass/
├── siar-operational-delegation/
├── siar-org-governance-observability/
└── siar-org-governance-testkit/
```

---

# 456. `siar-org-governance-core`

Owns:

```text
roles
scopes
errors
policy versions
```

---

# 457. `siar-service-ownership`

Production ownership registry.

---

# 458. `siar-responsibility`

Accountable/responsible/consulted/informed mappings.

---

# 459. `siar-oncall`

Duty rotations and handoff.

---

# 460. `siar-escalation`

Escalation chains/paging.

---

# 461. `siar-change-authority`

Change classes/approval thresholds/capabilities.

---

# 462. `siar-separation-of-duties`

Forbidden role-combination rules.

---

# 463. `siar-break-glass`

Emergency scoped authority.

---

# 464. `siar-operational-delegation`

Temporary attenuated delegation.

---

# 465. `siar-org-governance-observability`

Coverage/change/process health only.

---

# 466. `siar-org-governance-testkit`

ownership/on-call/SoD/privacy tests.

---

# 467. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Every production service and critical operational domain has an explicit accountable owner and backup/escalation path; ownerless production services are not allowed.
2. Ownership, on-call duty, approval authority, execution authority, incident command, and audit accountability are separate concepts and cannot be collapsed into a universal admin role.
3. Operational roles and workforce principals are scoped to services, tenants, regions, federation domains, or governance domains and never imply access to unrelated customer or personal data.
4. High-risk changes enforce explicit approval thresholds and separation of duties; the same principal cannot satisfy forbidden author/approver/executor combinations.
5. Break-glass and emergency authority is short-lived, reason-bound, scope-bound, strongly authenticated, auditable, and cannot disable hard security/privacy floors.
6. On-call and escalation systems store duty/coverage information only and do not collect GPS, continuous presence, productivity, communications, or employee social-graph data.
7. Ownership transfer, delegation, offboarding, and role changes revoke or attenuate old authority so stale human assignments cannot retain hidden access.
8. Delegation can never grant more authority than the delegator possesses, cannot silently become transitive, and must expire.
9. Tenant and federation organizational roles remain domain-scoped; tenant administrators cannot alter platform hard invariants, and remote federation operators cannot assign local workforce authority.
10. Change completion requires post-change verification, not merely successful command execution, and rollback follows the same scoped authority model.
11. Organizational metrics measure service/process health such as coverage gaps, acknowledgement latency, and rollback rates—not individual employee ranking or behavioral surveillance.
12. Organizational governance integrates with identity, authorization, audit, compliance, incident response, recovery, updates, FinOps, capacity, and residency without creating permanent privilege concentration or workforce-surveillance infrastructure.
```

---

# 468. Initial Production Scope

Implement first:

```text
typed ResponsibilityScope/OperationalRole
service ownership registry
backup ownership
on-call primary/secondary rotation
explicit handoff
escalation policies
change classes
approval thresholds
separation-of-duties checks
short-lived change capabilities
break-glass grants
temporary delegation
service-catalog integration
offboarding revocation
audit/compliance integration
tenant/federation role isolation
privacy-safe process metrics
organizational governance testkit
```

Then add:

```text
follow-the-sun rotations
multi-party high-assurance approvals
formal delegation/approval verification
managed contractor access
advanced service ownership transfer automation
cross-organization federation operator attestations
```

---

# 469. Definition of Done

Part 104 is complete when:

- every production service has explicit owner and backup/escalation path
- ownership and authority are separate
- on-call rotations are scoped and explicit
- handoffs require acknowledgement
- high-risk changes require approvals
- separation of duties is enforced
- break-glass is expiring and cannot weaken hard invariants
- tenant/federation roles remain isolated
- offboarding removes stale authority
- delegation is attenuated and bounded
- workforce metadata is privacy-minimized
- process metrics contain no employee-surveillance data
- audit/compliance/incident/recovery integrations are defined
- ownership/change/on-call/privacy/fuzz/formal tests are specified

---

# 470. Final Architecture

```text
                    SERVICE / DOMAIN
                           │
                           ▼
                   OWNERSHIP REGISTRY
                           │
                           ▼
                 RESPONSIBILITY MODEL
                           │
              ┌────────────┼────────────┐
              │            │            │
           ON-CALL     ESCALATION    CHANGE AUTH
              │            │            │
              └────────────┼────────────┘
                           ▼
                   SCOPED CAPABILITIES
                           │
                           ▼
                 EXECUTE / VERIFY / AUDIT
```

Organizational-governance safety model:

```text
explicit ownership
+
scoped roles
+
deterministic escalation
+
separation of duties
+
short-lived change authority
+
privacy-minimized workforce metadata
+
verification and audit
```

not:

```text
give a few people permanent superuser rights, track every employee continuously, and rely on informal knowledge when production breaks
```

---

# 471. Final Principle

Operational governance should make responsibility obvious and authority narrow.

The correct model is:

```text
assign ownership explicitly
+
separate responsibility from privilege
+
schedule on-call without surveillance
+
escalate deterministically
+
approve risky changes with independent roles
+
expire emergency access
+
verify every critical change
+
never centralize permanent human power
```

This architecture gives SIAR a privacy-preserving organizational foundation for service ownership, operational responsibility, on-call coverage, escalation, change authority, emergency response, tenant/federation governance, and workforce accountability while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–103.
