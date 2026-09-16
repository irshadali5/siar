# Core System Architecture Part 96 — Anonymous Network Incident Response, Detection, Triage, Containment, Forensics, Recovery & Privacy-Preserving Security Operations Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 96  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 28, 42, 46, 51–53, 55–56, 61–73, 78–83, 92–95

**Primary purpose:** define SIAR's incident-response architecture for detection, triage, containment, privacy-preserving forensics, credential/key compromise handling, service and tenant isolation, federation incidents, recovery, crisis coordination, evidence handling, post-incident validation, and operational lessons learned—without allowing incident response to become open-ended surveillance or a backdoor around privacy and anonymity guarantees.

---

# 1. Purpose

Security incidents are inevitable in a sufficiently large system.

Examples include:

```text
credential compromise
malicious workload
supply-chain compromise
data corruption
privacy-policy regression
federation peer compromise
malicious administrator
key leakage
host compromise
service abuse
```

A response system must move fast enough to contain damage while preserving:

```text
evidence quality
privacy boundaries
tenant isolation
cryptographic guarantees
operational continuity
```

The governing principle is:

> **SIAR incident response must contain threats using the narrowest effective scope, collect only the evidence needed to resolve the incident, preserve verifiable accountability, and automatically expire any temporary investigative authority.**

---

# 2. Architectural Position

```text
Signal / Alert / Control Failure
            │
            ▼
        Incident Intake
            │
            ▼
          Triage
            │
    ┌───────┼────────┐
    │       │        │
  False   Monitor   Confirmed
    │                │
    │                ▼
    │           Containment
    │                │
    │                ▼
    │           Investigation
    │                │
    │                ▼
    │             Recovery
    │                │
    └───────────────►│
                     ▼
                Post-Incident
```

---

# 3. Core Separation

Keep distinct:

```text
alert
incident
forensic evidence
audit evidence
operational logs
containment action
recovery action
post-incident review
```

---

# 4. Non-Goals

Part 96 does not create:

```text
permanent packet capture
continuous user monitoring
retroactive message decryption
global admin override
indefinite incident logging
```

---

# 5. Incident ID

```rust
pub struct IncidentId(pub [u8; 16]);
```

---

# 6. Incident Identity

Opaque and operational.

---

# 7. No User Identity Encoding

Hard rule.

---

# 8. Incident Severity

```rust
pub enum IncidentSeverity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}
```

---

# 9. Critical Examples

```text
root key compromise
release-signing compromise
cross-tenant breach
privacy floor disabled
widespread malicious artifact
```

---

# 10. Incident Classes

```rust
pub enum IncidentClass {
    CredentialCompromise,
    HostCompromise,
    WorkloadCompromise,
    KeyCompromise,
    SupplyChainCompromise,
    DataIntegrityFailure,
    AvailabilityFailure,
    PrivacyViolation,
    TenantIsolationFailure,
    FederationCompromise,
    AbuseCampaign,
    MaliciousAdministrator,
    Unknown,
}
```

---

# 11. Credential Compromise

Examples:

```text
stolen admin session
leaked refresh token
compromised workload credential
```

---

# 12. Host Compromise

Node/VM/bare metal.

---

# 13. Workload Compromise

One service/process/container.

---

# 14. Key Compromise

Private key exposed or suspected.

---

# 15. Supply-Chain Compromise

Artifact/dependency/build pipeline.

---

# 16. Data Integrity Failure

Corruption, rollback, inconsistent authoritative state.

---

# 17. Availability Failure

DoS/partition/systemic outage with security implications.

---

# 18. Privacy Violation

Unexpected deanonymization/telemetry leak/privacy floor failure.

---

# 19. Tenant Isolation Failure

Cross-tenant data/authority boundary failure.

---

# 20. Federation Compromise

Remote domain/trust root/federation gateway.

---

# 21. Abuse Campaign

Spam/flood/harassment/credential stuffing.

---

# 22. Malicious Administrator

Insider/admin credential misuse.

---

# 23. Hard Rule

Incident class does not imply broad investigative access.

---

# 24. Incident State

```rust
pub enum IncidentState {
    Detected,
    Triaging,
    Confirmed,
    Containing,
    Investigating,
    Recovering,
    Monitoring,
    Resolved,
    Closed,
}
```

---

# 25. Detected

Signal received.

---

# 26. Triaging

Validity/scope being assessed.

---

# 27. Confirmed

Enough evidence to treat as real.

---

# 28. Containing

Active damage reduction.

---

# 29. Investigating

Root-cause/evidence work.

---

# 30. Recovering

Restoring trustworthy service.

---

# 31. Monitoring

Recovery appears successful, observing for recurrence.

---

# 32. Resolved

Technical response complete.

---

# 33. Closed

Post-incident review/actions complete.

---

# 34. Hard Rule

Closed incidents cannot silently reopen investigative powers.

---

# 35. Reopening

Creates new incident/reviewed continuation.

---

# 36. Detection Inputs

Allowed signal sources:

```text
security controls
host attestation
audit verification
service health
auth anomalies
supply-chain verification
user reports
federation trust failures
```

---

# 37. Detection Source

```rust
pub enum DetectionSource {
    SecurityControl(ControlId),
    AuditSystem,
    HostAttestation,
    ServiceHealth,
    AuthenticationSystem,
    SupplyChain,
    UserReport,
    FederationPeer,
    ManualOperator,
}
```

---

# 38. No General Behavioral Surveillance Feed

Hard rule.

---

# 39. Detection Event

```rust
pub struct DetectionEvent {
    pub event_id: DetectionEventId,
    pub source: DetectionSource,
    pub signal: DetectionSignal,
    pub severity_hint: IncidentSeverity,
    pub observed_at: CoarseTimestamp,
}
```

---

# 40. Signal Payload

Minimal and typed.

---

# 41. No Message Content Unless User Explicitly Reports It

Hard rule.

---

# 42. Security Signal Examples

```text
invalid signature
credential replay
unexpected artifact digest
audit chain break
tenant context violation
```

---

# 43. Product Analytics Not Detection Source

Hard rule.

---

# 44. Why

Avoid repurposing product telemetry into security surveillance.

---

# 45. Detection Confidence

```rust
pub enum DetectionConfidence {
    Low,
    Medium,
    High,
    Confirmed,
}
```

---

# 46. Confidence ≠ Severity

Hard rule.

---

# 47. Triage

Triage determines:

```text
is it real?
what scope?
what assets?
what authority is needed?
what immediate containment?
```

---

# 48. Triage Record

```rust
pub struct IncidentTriage {
    pub incident: IncidentId,
    pub class: IncidentClass,
    pub severity: IncidentSeverity,
    pub affected_scope: IncidentScope,
    pub confidence: DetectionConfidence,
}
```

---

# 49. Incident Scope

```rust
pub enum IncidentScope {
    Device(DeviceId),
    Workload(WorkloadInstanceId),
    Service(ServiceId),
    Host(HostId),
    Tenant(TenantId),
    FederationPeer(FederationDomainId),
    Environment(EnvironmentClass),
    GovernanceDomain(GovernanceDomainId),
}
```

---

# 50. No Global Scope By Default

Hard rule.

---

# 51. Scope Expansion

Requires explicit evidence/review.

---

# 52. Hard Rule

Investigative scope expands monotonically only when justified, not by default.

---

# 53. Triage Questions

```text
What control failed?
What trusted evidence confirms it?
What is the blast radius?
What can be contained safely?
What privacy-sensitive evidence would be needed?
```

---

# 54. No "collect everything first"

Hard rule.

---

# 55. Incident Roles

```rust
pub enum IncidentRole {
    IncidentCommander,
    SecurityResponder,
    ServiceOwner,
    PrivacyReviewer,
    ForensicsReviewer,
    CommunicationsLead,
}
```

---

# 56. Incident Commander

Coordinates.

---

# 57. Security Responder

Technical response.

---

# 58. Service Owner

System knowledge.

---

# 59. Privacy Reviewer

Approves sensitive investigative expansion.

---

# 60. Forensics Reviewer

Evidence quality/chain of custody.

---

# 61. Communications Lead

Status/notifications.

---

# 62. Separation Of Duties

High-severity incidents.

---

# 63. No one responder gets universal data access.

---

# 64. Hard rule.

---

# 65. Incident Capability

```rust
pub struct IncidentCapability {
    pub incident: IncidentId,
    pub role: IncidentRole,
    pub scope: IncidentScope,
    pub actions: BTreeSet<IncidentAction>,
    pub expires_at: Timestamp,
}
```

---

# 66. Incident Actions

```rust
pub enum IncidentAction {
    ViewEvidence,
    CollectScopedEvidence,
    Quarantine,
    RevokeCredential,
    RotateKey,
    DisableFeature,
    BlockPeer,
    RestoreService,
    ExportEvidence,
}
```

---

# 67. Expiry Required

Hard rule.

---

# 68. No Permanent Incident Credential

Hard rule.

---

# 69. Incident Access

Short-lived.

---

# 70. Step-Up Authentication

Required for high-risk actions.

---

# 71. No shared incident account.

---

# 72. Hard rule.

---

# 73. Containment Principles

Containment should:

```text
stop damage
preserve evidence
minimize blast radius
preserve privacy
```

---

# 74. Containment Hierarchy

Prefer:

```text
single credential
single workload
single host
single service
single tenant
single federation peer
```

before:

```text
global shutdown
```

---

# 75. Hard Rule

Use the narrowest effective containment scope.

---

# 76. Quarantine State

```rust
pub enum QuarantineState {
    None,
    NetworkRestricted,
    CredentialRestricted,
    ReadOnly,
    FullyQuarantined,
}
```

---

# 77. NetworkRestricted

Only incident/control-plane connectivity.

---

# 78. CredentialRestricted

Sensitive credentials revoked.

---

# 79. ReadOnly

Prevent writes.

---

# 80. FullyQuarantined

No normal traffic.

---

# 81. No Privacy Downgrade During Containment

Hard rule.

---

# 82. Example

Do not switch anonymous users to direct-IP path because relay is under incident.

---

# 83. If Privacy-Safe Path Unavailable

Queue/fail closed.

---

# 84. Hard rule.

---

# 85. Credential Containment

Possible actions:

```text
revoke session
revoke cert
rotate secret
disable bootstrap token
```

---

# 86. CredentialRevocationPlan

```rust
pub struct CredentialRevocationPlan {
    pub credential_class: CredentialClass,
    pub scope: IncidentScope,
    pub replacement_required: bool,
}
```

---

# 87. No Broad Credential Revocation Without Need

Hard rule.

---

# 88. Key Compromise

Special lifecycle.

---

# 89. KeyCompromiseState

```rust
pub enum KeyCompromiseState {
    Suspected,
    Confirmed,
    Revoking,
    Rotating,
    Reissued,
    Monitoring,
    Closed,
}
```

---

# 90. Key Classes Matter

Examples:

```text
workload cert
tenant key
release-signing key
governance key
mailbox key
user E2EE key
```

---

# 91. No One Universal Key Response

Hard rule.

---

# 92. Release-Signing Compromise

Actions:

```text
revoke signing key
halt promotion
publish advisory
rotate trust
verify artifacts
```

---

# 93. Root/Governance Key Compromise

Requires ceremony/multi-party process.

---

# 94. No emergency software fallback.

---

# 95. Hard rule.

---

# 96. User E2EE Key Compromise

Account/device recovery architecture applies.

---

# 97. Security service must not centrally decrypt historical content.

---

# 98. Hard rule.

---

# 99. Host Compromise

Preferred response:

```text
quarantine
revoke workload credentials
capture minimal evidence
rebuild from trusted artifact
```

---

# 100. Rebuild Over Repair

Hard rule where practical.

---

# 101. No Long-Lived "Cleaned" Host Assumption

---

# 102. Workload Compromise

Terminate/redeploy from trusted artifact.

---

# 103. Reissue credentials.

---

# 104. Verify dependent services.

---

# 105. Tenant Isolation Incident

Critical.

---

# 106. Immediate:

```text
stop affected cross-tenant path
quarantine service/route
preserve scoped evidence
notify incident/privacy leads
```

---

# 107. Do Not Disable All Tenant Isolation Controls For Debugging

Hard rule.

---

# 108. Federation Peer Compromise

Actions:

```text
suspend trust edge
revoke peer capability
stop inbound/outbound federation path
preserve local continuity
```

---

# 109. No cross-domain administrative takeover.

---

# 110. Hard rule.

---

# 111. Privacy Incident

Examples:

```text
telemetry leak
direct-IP fallback
wrong public profile projection
search indexing of private content
```

---

# 112. Privacy Incident Response

Treat as security incident.

---

# 113. Hard rule.

---

# 114. Immediate privacy containment

```text
stop leak
disable affected path
invalidate cache/index
revoke exposed tokens if needed
```

---

# 115. No "wait for next release"

for critical privacy leak.

---

# 116. Hard rule.

---

# 117. Evidence Collection

Minimal and scoped.

---

# 118. Evidence Classes

```rust
pub enum IncidentEvidenceType {
    AuditRecord,
    SecurityLog,
    HostAttestation,
    ArtifactManifest,
    CredentialMetadata,
    NetworkMetadata,
    ConfigSnapshot,
    MemoryArtifact,
    DiskArtifact,
    UserSubmittedEvidence,
}
```

---

# 119. Memory/Disk Artifacts

High privacy risk.

---

# 120. Not baseline.

---

# 121. Require explicit elevated approval.

---

# 122. Hard rule.

---

# 123. Evidence Collection Plan

```rust
pub struct EvidenceCollectionPlan {
    pub incident: IncidentId,
    pub scope: IncidentScope,
    pub evidence_types: BTreeSet<IncidentEvidenceType>,
    pub expires_at: Timestamp,
    pub approvals: ApprovalBundle,
}
```

---

# 124. Evidence Plan Required For Sensitive Collection

Hard rule.

---

# 125. No Open-Ended Collector

---

# 126. Evidence Minimization

Prefer:

```text
digests
structured security events
config snapshots
attestation proofs
```

over:

```text
full disk
full memory
packet capture
```

---

# 127. Hard rule.

---

# 128. User Content Evidence

Only when:

```text
user explicitly reports
legal/security scope requires
privacy review approves
```

---

# 129. Otherwise prohibited.

---

# 130. Hard rule.

---

# 131. Packet Capture

Not baseline.

---

# 132. If absolutely necessary

must be:

```text
time-bounded
scope-bounded
protocol-field-minimized
approved
encrypted
auto-expiring
```

---

# 133. No permanent packet capture infrastructure.

---

# 134. Hard rule.

---

# 135. Memory Capture

May contain secrets/plaintext.

---

# 136. Requires highest approval.

---

# 137. Prefer core-dump disabled baseline.

---

# 138. Hard rule.

---

# 139. Disk Forensics

Prefer immutable image of compromised infrastructure scope only.

---

# 140. No broad user data scrape.

---

# 141. Hard rule.

---

# 142. Evidence Object

```rust
pub struct IncidentEvidence {
    pub evidence_id: EvidenceId,
    pub incident: IncidentId,
    pub evidence_type: IncidentEvidenceType,
    pub scope: IncidentScope,
    pub digest: Digest,
    pub collected_at: CoarseTimestamp,
    pub retention: IncidentEvidenceRetention,
}
```

---

# 143. Evidence Integrity

Digest + signature/chain-of-custody.

---

# 144. No mutable evidence.

---

# 145. Hard rule.

---

# 146. Chain Of Custody

```rust
pub struct CustodyEvent {
    pub evidence: EvidenceId,
    pub action: CustodyAction,
    pub actor: IncidentActorRef,
    pub at: CoarseTimestamp,
    pub signature: SignatureBytes,
}
```

---

# 147. Custody Actions

```text
Collected
Transferred
Opened
Exported
Destroyed
```

---

# 148. Not every byte read.

---

# 149. Significant custody only.

---

# 150. Hard rule.

---

# 151. Evidence Encryption

Per incident/scope.

---

# 152. No one global forensic key.

---

# 153. Hard rule.

---

# 154. Evidence Key Custody

High severity:

```text
multi-party or HSM-backed
```

---

# 155. Evidence Key Expiry

After retention where legally/operationally allowed.

---

# 156. Hard rule.

---

# 157. Forensic Workstation

Isolated environment.

---

# 158. No internet access by default.

---

# 159. No production credentials.

---

# 160. Hard rule.

---

# 161. Evidence Export

Signed, scoped bundle.

---

# 162. No ad hoc zip/email.

---

# 163. Hard rule.

---

# 164. Forensic Tooling

Deterministic/traceable.

---

# 165. Tool versions recorded.

---

# 166. No opaque third-party cloud upload.

---

# 167. Hard rule.

---

# 168. Triage Timeline

Use sequence/coarse time.

---

# 169. No need for precise user activity timestamps.

---

# 170. Hard rule.

---

# 171. Incident Timeline

```rust
pub struct IncidentTimelineEntry {
    pub sequence: u64,
    pub state: IncidentState,
    pub action: IncidentTimelineAction,
    pub at: CoarseTimestamp,
}
```

---

# 172. Append-only.

---

# 173. Corrections via superseding entry.

---

# 174. Hard rule.

---

# 175. Incident Command

Single accountable commander per active incident.

---

# 176. But high-risk actions still require separate approvals.

---

# 177. Hard rule.

---

# 178. Crisis Communication

Internal operational channels.

---

# 179. Out-of-band comms recommended for severe identity/control-plane incidents.

---

# 180. Do not rely solely on compromised infrastructure.

---

# 181. Hard rule.

---

# 182. User Communication

If user security/privacy affected:

```text
what happened
what is known
what changed
what user should do
```

---

# 183. No speculative blame.

---

# 184. No overclaiming certainty.

---

# 185. Hard rule.

---

# 186. Tenant Communication

Scoped tenant incident.

---

# 187. No disclosure of unrelated tenant/user data.

---

# 188. Hard rule.

---

# 189. Federation Communication

Peer incident notifications use signed federation channel.

---

# 190. No email-only trust for high-risk federation changes.

---

# 191. Hard rule.

---

# 192. Containment Action Receipt

Part 94.

---

# 193. High-risk actions receive signed receipt.

---

# 194. Examples:

```text
credential revoked
peer blocked
key rotated
artifact revoked
tenant path disabled
```

---

# 195. Incident Audit

Record:

```text
incident created
scope expanded
sensitive evidence approved
containment executed
recovery accepted
incident closed
```

---

# 196. Not every investigative click.

---

# 197. Hard rule.

---

# 198. Detection False Positive

Close with evidence.

---

# 199. Do not retain expanded investigative authority.

---

# 200. Hard rule.

---

# 201. Monitoring State

After recovery.

---

# 202. Monitoring data remains scoped to incident controls.

---

# 203. No permanent enhanced surveillance.

---

# 204. Hard rule.

---

# 205. Recovery Principles

Recovery means restoring trust, not only service availability.

---

# 206. Required questions:

```text
Are artifacts trusted?
Are keys rotated?
Are compromised credentials invalid?
Are policies restored?
Are controls passing?
Is evidence preserved?
```

---

# 207. Recovery Gate

```rust
pub struct RecoveryGate {
    pub required_controls: Vec<ControlId>,
    pub required_key_rotations: Vec<KeyClass>,
    pub required_artifact_verifications: Vec<ArtifactDigest>,
    pub incident: IncidentId,
}
```

---

# 208. Recovery Requires Fresh Evidence

Hard rule.

---

# 209. "Service is running" ≠ recovered.

---

# 210. Hard rule.

---

# 211. Recovery States

```rust
pub enum RecoveryState {
    NotReady,
    PartiallyReady,
    ReadyForCanary,
    Canary,
    ReadyForFullService,
    Completed,
}
```

---

# 212. Canary Recovery

Preferred.

---

# 213. Limited traffic.

---

# 214. Re-evaluate controls.

---

# 215. No all-at-once if avoidable.

---

# 216. Hard rule.

---

# 217. Key Rotation After Incident

Must invalidate old credentials.

---

# 218. No dual-validity indefinitely.

---

# 219. Hard rule.

---

# 220. Artifact Recovery

Known-good signed release.

---

# 221. No "rebuild from compromised filesystem."

---

# 222. Hard rule.

---

# 223. Data Recovery

Authoritative DB/backup integrity validated.

---

# 224. Deletion barriers preserved.

---

# 225. No restoring revoked/deleted state.

---

# 226. Hard rule.

---

# 227. Tenant Recovery

Per tenant if incident scoped.

---

# 228. Avoid global service reset if not needed.

---

# 229. Hard rule.

---

# 230. Federation Recovery

Requires peer trust re-establishment.

---

# 231. New agreement/capability if peer identity changed.

---

# 232. No silent resumption with old compromised identity.

---

# 233. Hard rule.

---

# 234. Privacy Recovery

If privacy leak:

```text
remove exposed indexes/caches
rotate exposed pseudonyms/tokens if possible
verify telemetry/publication paths
```

---

# 235. No claim that already leaked public data can be recalled.

---

# 236. Hard truth.

---

# 237. Recovery Validation

Part 95 compliance engine.

---

# 238. Critical controls must pass.

---

# 239. Exception use tightly reviewed.

---

# 240. No restoring service under blanket exception.

---

# 241. Hard rule.

---

# 242. Post-Incident Review

Required for High/Critical.

---

# 243. Blameless but evidence-based.

---

# 244. Focus:

```text
technical causes
process gaps
control failures
detection gaps
privacy impact
recovery effectiveness
```

---

# 245. No personal blame dossier.

---

# 246. Hard rule.

---

# 247. Post-Incident Record

```rust
pub struct PostIncidentReview {
    pub incident: IncidentId,
    pub root_causes: Vec<RootCause>,
    pub contributing_factors: Vec<ContributingFactor>,
    pub actions: Vec<CorrectiveAction>,
}
```

---

# 248. Corrective Actions

Track to completion.

---

# 249. Not only prose.

---

# 250. Hard rule.

---

# 251. Corrective Action State

```rust
pub enum CorrectiveActionState {
    Planned,
    InProgress,
    Verified,
    Closed,
}
```

---

# 252. Verified Requires Evidence

---

# 253. No "fixed" without validation.

---

# 254. Hard rule.

---

# 255. Lessons Learned

Feed:

```text
control catalog
runbooks
tests
architecture
training
```

---

# 256. Do Not Feed

```text
behavioral employee surveillance
user-level profiling
```

---

# 257. Hard rule.

---

# 258. Detection Improvement

Can add new signal if narrowly scoped.

---

# 259. Requires privacy review.

---

# 260. No incident as excuse for permanent broad telemetry.

---

# 261. Hard rule.

---

# 262. Incident Data Retention

Class-based.

---

# 263. Incident metadata

Can retain longer.

---

# 264. Sensitive forensic evidence

shorter/strict legal policy.

---

# 265. No indefinite evidence lake.

---

# 266. Hard rule.

---

# 267. Incident Evidence Retention

```rust
pub enum IncidentEvidenceRetention {
    Days7,
    Days30,
    Days90,
    LongCritical,
    LegalScoped,
}
```

---

# 268. Destruction

Cryptographic erasure where possible.

---

# 269. Destruction Receipt

Audit event.

---

# 270. Hard rule.

---

# 271. Backup Of Incident Evidence

Only if required.

---

# 272. Same retention.

---

# 273. No backup extension loophole.

---

# 274. Hard rule.

---

# 275. Incident Search

Scoped structured search.

---

# 276. No global free-form query across all forensic data.

---

# 277. Hard rule.

---

# 278. Evidence Metadata Index

By:

```text
incident
type
scope
coarse time
```

---

# 279. No message/user index.

---

# 280. Hard rule.

---

# 281. Incident Correlation

Across incidents only for infrastructure indicators.

---

# 282. Examples:

```text
same malicious artifact digest
same compromised signing key
same exploit signature
```

---

# 283. Not user behavior.

---

# 284. Hard rule.

---

# 285. Indicators Of Compromise

IoCs may include:

```text
artifact hash
malicious cert
bad service endpoint
exploit signature
```

---

# 286. Avoid:

```text
user contact ID
message content
personal profile data
```

---

# 287. Hard rule.

---

# 288. IOC Distribution

Signed.

---

# 289. TTL.

---

# 290. Scoped.

---

# 291. No permanent global blacklist without governance.

---

# 292. Hard rule.

---

# 293. Abuse Campaign Response

Separate from infrastructure compromise.

---

# 294. Use:

```text
rate limits
capability revocation
space moderation
mailbox protection
```

---

# 295. Do not deanonymize all users.

---

# 296. Hard rule.

---

# 297. Malicious Administrator Incident

Highest sensitivity.

---

# 298. Immediate:

```text
revoke admin capability
revoke sessions
preserve scoped admin audit
rotate affected secrets
review actions
```

---

# 299. No shared admin accounts baseline.

---

# 300. Hard rule.

---

# 301. Admin Compromise Scope

Actions executed by credential, not assumed human intent.

---

# 302. Hard truth.

---

# 303. Insider Investigation

Evidence must remain role/scoped.

---

# 304. No broad employee surveillance.

---

# 305. Hard rule.

---

# 306. Supply-Chain Incident

Actions:

```text
halt promotion
revoke artifact
identify affected releases
rebuild reproducibly
rotate signing keys if needed
```

---

# 307. Artifact Inventory

From Part 73.

---

# 308. No user content needed.

---

# 309. Hard rule.

---

# 310. Privacy-Safe Detection Architecture

Detection should rely on:

```text
control failure
attestation
cryptographic verification
aggregate service anomalies
```

rather than:

```text
user behavior graph
```

---

# 311. Hard rule.

---

# 312. Anomaly Detection

Can be used for infrastructure.

---

# 313. Example:

```text
queue depth anomaly
auth failure spike
unexpected certificate issuance
```

---

# 314. Avoid behavioral anomaly models on private user actions.

---

# 315. Hard rule.

---

# 316. Machine Learning For Detection

Not baseline.

---

# 317. If added

input limited to infrastructure/security telemetry.

---

# 318. No message/search/contact behavior.

---

# 319. Hard rule.

---

# 320. Incident Automation

Useful for obvious containment.

---

# 321. Example:

```text
revoke expired cert
quarantine attestation-failed host
disable compromised release
```

---

# 322. High-risk automation requires policy.

---

# 323. No autonomous destructive actions outside signed runbooks.

---

# 324. Hard rule.

---

# 325. Automated Containment Policy

```rust
pub struct AutomatedContainmentPolicy {
    pub incident_class: IncidentClass,
    pub trigger: ControlId,
    pub allowed_actions: BTreeSet<IncidentAction>,
}
```

---

# 326. Signed/versioned.

---

# 327. No arbitrary script.

---

# 328. Hard rule.

---

# 329. Runbook Integration

Part 65.

---

# 330. Typed incident runbooks.

---

# 331. Step preconditions.

---

# 332. Required approvals.

---

# 333. No shell-paste playbooks baseline.

---

# 334. Hard rule.

---

# 335. Incident Runbook Step

```rust
pub struct IncidentRunbookStep {
    pub action: IncidentAction,
    pub required_state: IncidentState,
    pub requires_approval: bool,
    pub rollback: Option<IncidentAction>,
}
```

---

# 336. Rehearsal

Part 70 disaster exercises.

---

# 337. Incident scenarios should be drilled.

---

# 338. Include privacy-specific scenarios.

---

# 339. Hard rule.

---

# 340. Example Drill Scenarios

```text
release key compromise
tenant boundary failure
relay compromise
federation peer breach
privacy telemetry leak
```

---

# 341. No production user data in drill.

---

# 342. Hard rule.

---

# 343. Incident Notification

Internal high-priority.

---

# 344. External user notification based on impact.

---

# 345. No generic mass alert if unaffected.

---

# 346. Hard rule.

---

# 347. Incident Status Page

Public service-impact info.

---

# 348. No sensitive investigative detail.

---

# 349. Hard rule.

---

# 350. Security Advisory

If vulnerability public.

---

# 351. Signed.

---

# 352. Includes:

```text
affected versions
fixed versions
mitigations
```

---

# 353. No exploit secret details before coordinated release if dangerous.

---

# 354. Hard rule.

---

# 355. Federation Advisory

Signed domain-to-domain.

---

# 356. Trust updates separate from human-readable advisory.

---

# 357. Hard rule.

---

# 358. Incident Evidence Access

Capability-controlled.

---

# 359. Roles see minimum necessary.

---

# 360. Example:

```text
PrivacyReviewer may see data classification/evidence scope
ServiceOwner may see service config/log evidence
```

---

# 361. No all-evidence universal viewer.

---

# 362. Hard rule.

---

# 363. Evidence Export Approval

Required for external sharing.

---

# 364. Redaction.

---

# 365. Signed manifest.

---

# 366. Hard rule.

---

# 367. Incident Query API

```rust
pub trait IncidentQueryService {
    fn incident(
        &self,
        id: IncidentId,
    ) -> Result<IncidentRecord, IncidentError>;

    fn timeline(
        &self,
        id: IncidentId,
        cursor: Option<IncidentCursor>,
    ) -> Result<IncidentTimelinePage, IncidentError>;
}
```

---

# 368. No global raw evidence API.

---

# 369. Hard rule.

---

# 370. Detection Service

```rust
pub trait DetectionService {
    fn ingest(
        &self,
        event: DetectionEvent,
    ) -> Result<IncidentDisposition, IncidentError>;
}
```

---

# 371. Triage Service

```rust
pub trait TriageService {
    fn triage(
        &self,
        incident: IncidentId,
        decision: TriageDecision,
    ) -> Result<IncidentState, IncidentError>;
}
```

---

# 372. Containment Service

```rust
pub trait ContainmentService {
    fn execute(
        &self,
        capability: IncidentCapability,
        action: ContainmentCommand,
    ) -> Result<ContainmentReceipt, IncidentError>;
}
```

---

# 373. Evidence Service

```rust
pub trait IncidentEvidenceService {
    fn collect(
        &self,
        capability: IncidentCapability,
        plan: EvidenceCollectionPlan,
    ) -> Result<Vec<EvidenceId>, IncidentError>;
}
```

---

# 374. Recovery Service

```rust
pub trait RecoveryService {
    fn evaluate_gate(
        &self,
        gate: &RecoveryGate,
    ) -> Result<RecoveryState, IncidentError>;
}
```

---

# 375. No hidden side effects.

---

# 376. Hard rule.

---

# 377. Incident Error Taxonomy

```rust
pub enum IncidentError {
    IncidentNotFound,
    InvalidStateTransition,
    ScopeViolation,
    CapabilityDenied,
    ApprovalRequired,
    EvidencePlanRequired,
    EvidenceExpired,
    ContainmentFailed,
    RecoveryGateFailed,
    AuditFailure,
    PrivacyViolation,
    Internal,
}
```

---

# 378. Audit Integration

Part 94.

---

# 379. High-risk actions audited.

---

# 380. Incident itself not used as excuse to disable audit.

---

# 381. Hard rule.

---

# 382. Compliance Integration

Part 95.

---

# 383. Control failures can open incident.

---

# 384. Recovery requires controls pass.

---

# 385. No blanket exception.

---

# 386. Hard rule.

---

# 387. Analytics Integration

Part 92.

---

# 388. Use aggregate operational metrics.

---

# 389. Never promote raw product analytics into forensic data without explicit review.

---

# 390. Hard rule.

---

# 391. Experimentation Integration

Part 93.

---

# 392. Guardrail failures can pause experiment/trigger incident.

---

# 393. Experiment assignments irrelevant to forensic identity.

---

# 394. Hard rule.

---

# 395. Authorization Integration

Part 81.

---

# 396. Incident capabilities attenuated/scoped.

---

# 397. No all-powerful emergency role.

---

# 398. Hard rule.

---

# 399. Authentication Integration

Part 82.

---

# 400. High-risk incident actions require strong operator auth.

---

# 401. Incident capability short-lived.

---

# 402. Hard rule.

---

# 403. Secrets Integration

Part 80.

---

# 404. Emergency secret access through broker and scoped purpose.

---

# 405. No plaintext secret dump.

---

# 406. Hard rule.

---

# 407. HSM Integration

Part 72.

---

# 408. High-value key response via ceremony.

---

# 409. No bypass through incident tooling.

---

# 410. Hard rule.

---

# 411. Host Integrity Integration

Part 71.

---

# 412. Quarantine compromised hosts.

---

# 413. Re-attestation before return.

---

# 414. Hard rule.

---

# 415. Supply-Chain Integration

Part 73.

---

# 416. Artifact revocation/rebuild.

---

# 417. Provenance used for scope.

---

# 418. Hard rule.

---

# 419. Deployment Integration

Part 62.

---

# 420. Recovery deploys trusted exact artifact.

---

# 421. No mutable hotfix on compromised host.

---

# 422. Hard rule.

---

# 423. Database Integration

Part 74.

---

# 424. For data corruption incident:

```text
quiesce writes
verify authoritative state
reconcile
restore from validated backup if needed
```

---

# 425. No blind rollback.

---

# 426. Hard rule.

---

# 427. Event Bus Integration

Part 75.

---

# 428. Incident events minimal.

---

# 429. No evidence payloads on global bus.

---

# 430. Hard rule.

---

# 431. Federation Integration

Part 58.

---

# 432. Peer containment scoped.

---

# 433. No global federation shutdown unless necessary.

---

# 434. Hard rule.

---

# 435. Tenant Integration

Part 69.

---

# 436. Tenant incident isolation.

---

# 437. No cross-tenant evidence access.

---

# 438. Hard rule.

---

# 439. Data Lifecycle

Part 67.

---

# 440. Evidence retention bounded.

---

# 441. Crypto erasure where appropriate.

---

# 442. Hard rule.

---

# 443. Backup Integration

Part 33.

---

# 444. Incident evidence backup rare/controlled.

---

# 445. Recovery data validated.

---

# 446. Deleted/revoked state not resurrected.

---

# 447. Hard rule.

---

# 448. Observability

Safe metrics:

```text
incident count by class/severity
time to triage
time to containment
time to recovery
control recurrence
```

---

# 449. No user identity.

---

# 450. No evidence content.

---

# 451. Hard rule.

---

# 452. Incident SLOs

Examples:

```text
critical triage latency
containment latency
credential revocation latency
recovery verification latency
```

---

# 453. Privacy SLO

```text
0 unapproved sensitive evidence collection
0 open-ended incident access after closure
0 privacy downgrade during containment
```

---

# 454. Security SLO

```text
0 compromised credentials accepted after confirmed revocation
0 compromised artifact promoted after incident
```

---

# 455. Failure Modes

```text
incident system unavailable
audit unavailable
containment partially fails
evidence corruption
recovery control failure
```

---

# 456. Incident System Unavailable

Emergency fallback runbook.

---

# 457. Still uses signed/scoped operator capabilities where possible.

---

# 458. No shared root password.

---

# 459. Hard rule.

---

# 460. Audit Unavailable

High-risk actions may fail closed unless life/safety/emergency policy explicitly permits break-glass.

---

# 461. Break-glass audited later with signed recovery record.

---

# 462. Cannot weaken privacy floor.

---

# 463. Hard rule.

---

# 464. Partial Containment Failure

Escalate scope carefully.

---

# 465. No automatic global shutdown unless policy.

---

# 466. Hard rule.

---

# 467. Evidence Corruption

Mark invalid.

---

# 468. Preserve original.

---

# 469. No silent repair.

---

# 470. Hard rule.

---

# 471. Recovery Control Failure

Remain quarantined/partial service.

---

# 472. No "service is needed" bypass of critical control.

---

# 473. Hard rule.

---

# 474. Testing

Need incident-response testkit.

---

# 475. Test Scenarios

```text
credential compromise
host compromise
tenant boundary failure
privacy leak
federation compromise
```

---

# 476. State Machine Test

Invalid incident transitions rejected.

---

# 477. Scope Test

Responder capability cannot exceed incident scope.

---

# 478. Expiry Test

Incident capability auto-expires.

---

# 479. Evidence Plan Test

Sensitive evidence requires approval.

---

# 480. Privacy Test

User content not collected in ordinary incident.

---

# 481. Quarantine Test

Host removed from normal traffic.

---

# 482. Credential Test

Revoked credential cannot authenticate.

---

# 483. Key Rotation Test

Old key rejected after cutover.

---

# 484. Tenant Isolation Test

Tenant A incident does not expose tenant B.

---

# 485. Federation Test

Compromised peer blocked without affecting independent peers.

---

# 486. Recovery Gate Test

Service cannot return before required controls pass.

---

# 487. Audit Test

Containment/recovery receipts recorded.

---

# 488. Monitoring Test

Enhanced monitoring ends after incident closure.

---

# 489. Retention Test

Sensitive forensic evidence expires.

---

# 490. Backup Test

Expired evidence not resurrected.

---

# 491. Malicious Admin Test

Admin sessions/capabilities revoked.

---

# 492. Supply-Chain Test

Revoked artifact cannot deploy.

---

# 493. Privacy Incident Test

Public/private index leak containment works.

---

# 494. Fuzzing

Fuzz:

```text
incident record
evidence manifest
containment command
recovery gate
federation incident notice
```

---

# 495. Property Tests

Properties:

```text
incident capability can never authorize outside its scope
closed incident cannot retain active investigative capability
critical recovery gate failure can never produce Completed recovery state
sensitive evidence cannot be collected without a valid unexpired evidence plan
```

---

# 496. Formal Verification Targets

Strong candidates:

```text
incident state machine
scope expansion
capability expiry
containment/recovery gates
evidence-retention lifecycle
```

---

# 497. Kani Candidate

state/action/scope invariants.

---

# 498. TLA+ Candidate

detect → contain → rotate/revoke → recover → monitor → close.

---

# 499. Loom Candidate

concurrent containment action + credential use + recovery state update.

---

# 500. Performance

Incident systems are low-throughput but latency-sensitive for critical actions.

---

# 501. Credential revocation

fast.

---

# 502. Quarantine

fast.

---

# 503. Evidence collection

asynchronous.

---

# 504. Forensics

offline/batch.

---

# 505. No evidence scan on normal request path.

---

# 506. Hard rule.

---

# 507. Storage

Separate stores:

```text
incident metadata
incident timeline
evidence metadata
encrypted evidence blobs
containment receipts
post-incident actions
```

---

# 508. No user-data warehouse.

---

# 509. Hard rule.

---

# 510. Partitioning

By incident/scope.

---

# 511. No global user identifier.

---

# 512. Hard rule.

---

# 513. Crate Layout

Recommended:

```text
crates/
├── siar-incident-core/
├── siar-detection/
├── siar-triage/
├── siar-containment/
├── siar-incident-capability/
├── siar-forensics/
├── siar-evidence-custody/
├── siar-recovery/
├── siar-post-incident/
├── siar-incident-observability/
└── siar-incident-testkit/
```

---

# 514. `siar-incident-core`

Owns:

```text
IncidentId
states
classes
severity
errors
```

---

# 515. `siar-detection`

Typed detection sources/signals.

---

# 516. `siar-triage`

Scope/severity/confidence decisions.

---

# 517. `siar-containment`

Quarantine/revocation/blocking.

---

# 518. `siar-incident-capability`

Short-lived scoped responder authority.

---

# 519. `siar-forensics`

Minimal evidence collection/analysis workflows.

---

# 520. `siar-evidence-custody`

Evidence encryption, signatures, custody chain.

---

# 521. `siar-recovery`

Recovery gates/canary/revalidation.

---

# 522. `siar-post-incident`

Corrective actions/lessons learned.

---

# 523. `siar-incident-observability`

Aggregate incident metrics only.

---

# 524. `siar-incident-testkit`

compromise/privacy/containment/recovery tests.

---

# 525. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Incident-response authority is short-lived, role-scoped, incident-scoped, and never becomes a permanent emergency superuser mechanism.
2. Detection uses security/control/attestation/audit signals rather than centralized monitoring of ordinary private user behavior.
3. Containment uses the narrowest effective scope and never weakens anonymity, encryption, tenant isolation, or authorization guarantees for convenience.
4. Sensitive evidence collection requires an explicit, expiring collection plan with scope, evidence classes, approvals, and retention.
5. Packet capture, memory dumps, full disk images, and other invasive forensic sources are exceptional—not baseline—evidence types.
6. Incident evidence is immutable, encrypted, digest-verified, custody-tracked, retention-bounded, and never stored in a general analytics or audit warehouse.
7. Credential/key compromise response revokes old authority and reissues/rotates trust; compromised credentials are never left valid merely for availability.
8. Recovery requires fresh control/evidence validation; service availability alone can never prove that a compromised system is trustworthy again.
9. Tenant, federation-peer, managed-profile, and infrastructure incidents remain scope-isolated; one incident does not grant cross-tenant or cross-domain investigative access.
10. Incident closure expires temporary investigative capabilities and enhanced monitoring; an incident cannot silently create permanent surveillance.
11. Post-incident reviews improve controls, tests, runbooks, and architecture without creating user- or employee-level behavior dossiers.
12. Incident response preserves auditability and accountability while retaining the privacy, local-first, least-authority, and anti-surveillance guarantees of normal operation.
```

---

# 526. Initial Production Scope

Implement first:

```text
typed incident classes/severity/state
scoped triage model
short-lived incident capabilities
credential/session revocation
host/workload quarantine
federation peer suspension
privacy incident handling
key-compromise state machine
minimal evidence-collection plans
encrypted evidence store
chain-of-custody records
containment receipts
recovery gates using compliance controls
canary recovery
post-incident corrective actions
audit integration
privacy-safe incident metrics
incident-response testkit
```

Then add:

```text
isolated forensic workstation tooling
signed IOC distribution
advanced automated containment
privacy-preserving incident correlation
multi-party forensic approvals
formal incident/recovery verification
```

---

# 527. Definition of Done

Part 96 is complete when:

- detection inputs are security/control based, not behavioral surveillance
- incident triage produces explicit class/severity/scope/confidence
- responder authority is scoped and expiring
- containment is narrow and privacy-preserving
- invasive evidence collection requires approval
- evidence is encrypted, immutable, custody-tracked, and retention-bounded
- credential/key compromise revokes old authority
- tenant/federation incidents remain isolated
- privacy incidents are first-class security incidents
- recovery requires fresh compliance/control validation
- enhanced incident monitoring expires
- post-incident actions are evidence-backed
- audit/compliance/incident boundaries remain strict
- incident/forensics/recovery/fuzz/formal tests are specified

---

# 528. Final Architecture

```text
                   DETECTION SIGNAL
                         │
                         ▼
                        TRIAGE
                         │
              ┌──────────┼──────────┐
              │          │          │
           DISMISS     MONITOR   CONFIRMED
                                    │
                                    ▼
                               CONTAINMENT
                                    │
                                    ▼
                               INVESTIGATION
                                    │
                                    ▼
                                 RECOVERY
                                    │
                                    ▼
                                MONITORING
                                    │
                                    ▼
                           POST-INCIDENT REVIEW
```

Incident-response safety model:

```text
security-signal detection
+
scoped responder authority
+
narrow containment
+
minimal evidence
+
strong custody
+
credential/key revocation
+
verified recovery gates
+
automatic investigative expiry
```

not:

```text
declare an incident, grant broad permanent access, capture everything, and keep the resulting surveillance archive forever
```

---

# 529. Final Principle

Incident response should temporarily increase the system's ability to contain and understand a threat—not permanently reduce the privacy of everyone using the system.

The correct model is:

```text
detect from security evidence
+
scope carefully
+
contain narrowly
+
collect minimally
+
preserve evidence properly
+
rotate compromised trust
+
recover only after verification
+
expire emergency authority
```

This architecture gives SIAR a privacy-preserving security-operations foundation for detection, triage, containment, forensics, key compromise, tenant/federation incidents, recovery, crisis coordination, and post-incident improvement while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–95.
