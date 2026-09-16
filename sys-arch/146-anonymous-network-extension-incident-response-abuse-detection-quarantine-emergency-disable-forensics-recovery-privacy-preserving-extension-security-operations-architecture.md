# Core System Architecture Part 146 — Anonymous Network Extension Incident Response, Abuse Detection, Quarantine, Emergency Disable, Forensics, Recovery & Privacy-Preserving Extension Security Operations Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 146  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 46, 94–99, 119–126, 133–145

**Primary purpose:** define SIAR's extension security-operations architecture for incident detection, abuse signals, scoped quarantine, emergency disable, forensic evidence, compromise investigation, blast-radius analysis, containment, recovery, reinstatement, publisher/dependency impact, post-incident improvement, and privacy-preserving defensive operations.

---

# 1. Purpose

Third-party extensions create a recurring security-operations problem.

Even after marketplace review, certification, sandboxing, capability controls, update governance, and observability, incidents can still occur:

```text
malicious package
compromised publisher key
sandbox escape attempt
unexpected data exfiltration
supply-chain compromise
permission abuse
resource abuse
crash-loop
privacy violation
```

The governing principle is:

> **SIAR must be able to detect, contain, quarantine, investigate, recover, and safely reinstate extension functionality without granting incident responders ambient access to user content or creating a universal emergency backdoor.**

---

# 2. Architectural Position

```text
                  EXTENSION SIGNALS
                         │
                         ▼
                 INCIDENT DETECTION
                         │
             ┌───────────┼───────────┐
             │           │           │
          ABUSE       SECURITY     PRIVACY
             │           │           │
             └───────────┼───────────┘
                         ▼
                 INCIDENT TRIAGE
                         │
             ┌───────────┼───────────┐
             │           │           │
         CONTAIN      QUARANTINE   DISABLE
             │           │           │
             └───────────┼───────────┘
                         ▼
                   FORENSIC REVIEW
                         │
                         ▼
                 RECOVERY / REINSTATE
```

---

# 3. Core Separation

Keep distinct:

```text
incident
alert
abuse signal
quarantine
revocation
emergency disable
forensic evidence
recovery
reinstatement
```

---

# 4. Non-Goals

Part 146 does not create:

```text
global emergency superuser
automatic access to private content
per-user behavioral surveillance
irreversible extension deletion as first response
security operations as general telemetry collection
```

---

# 5. Incident Identity

```rust
pub struct ExtensionIncidentId(pub [u8; 16]);
```

---

# 6. Incident Class

```rust
pub enum ExtensionIncidentClass {
    Security,
    Privacy,
    Abuse,
    Availability,
    SupplyChain,
    DataIntegrity,
    Compatibility,
    Operational,
}
```

---

# 7. Incident Severity

```rust
pub enum ExtensionIncidentSeverity {
    Sev4,
    Sev3,
    Sev2,
    Sev1,
}
```

---

# 8. Hard Rule

Severity is evidence-backed and does not directly imply motive.

---

# 9. Incident State

```rust
pub enum ExtensionIncidentState {
    Detected,
    Triaging,
    Containing,
    Quarantined,
    Investigating,
    Recovering,
    Monitoring,
    Resolved,
    Closed,
}
```

---

# 10. No Detected→Closed Direct

Hard rule.

---

# 11. Incident Scope

```rust
pub struct ExtensionIncidentScope {
    pub extension: ExtensionId,
    pub package: Option<ExtensionPackageId>,
    pub version: Option<ExtensionVersion>,
    pub publisher: Option<PublisherId>,
    pub tenant: Option<TenantId>,
    pub platform: Option<PlatformClass>,
}
```

---

# 12. Hard Rule

Scope is explicit and minimized.

---

# 13. No User Scope By Default

Hard rule.

---

# 14. Incident Trigger

```rust
pub enum ExtensionIncidentTrigger {
    SandboxViolation,
    CapabilityViolation,
    UnauthorizedDataFlow,
    RevokedDependency,
    SignatureAnomaly,
    CrashSpike,
    ResourceAbuse,
    PrivacyLeak,
    MaliciousBehaviorReport,
    PublisherCompromise,
    ManualSecurityReport,
}
```

---

# 15. Hard Rule

Triggers are technical facts or structured reports.

---

# 16. Detection Sources

Allowed sources:

```text
runtime policy violations
sandbox violations
revocation feeds
signed advisories
crash/health anomalies
user/admin reports
certification failures
dependency compromise evidence
```

---

# 17. Hard Rule

Do not create incident based only on opaque reputation or popularity.

---

# 18. Abuse Signal

```rust
pub struct ExtensionAbuseSignal {
    pub extension: ExtensionId,
    pub class: ExtensionAbuseClass,
    pub confidence: EvidenceConfidence,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 19. Abuse Class

```rust
pub enum ExtensionAbuseClass {
    NotificationSpam,
    PromptAbuse,
    ResourceAbuse,
    UnauthorizedEnumeration,
    PermissionProbing,
    SuspiciousNetworkUse,
    RepeatedPolicyViolation,
}
```

---

# 20. Hard Rule

Abuse detection must rely on technical behavior, not private user content.

---

# 21. Evidence Confidence

```rust
pub enum EvidenceConfidence {
    Confirmed,
    Strong,
    Moderate,
    Weak,
    Unknown,
}
```

---

# 22. Hard Rule

Weak/Unknown evidence cannot justify maximum containment without corroboration except where hard fail-safe policy says otherwise.

---

# 23. Incident Decision

```rust
pub enum ExtensionIncidentDecision {
    Observe,
    Throttle,
    RestrictCapability,
    QuarantineRuntime,
    BlockNewInstalls,
    EmergencyDisable,
    RevokePackage,
}
```

---

# 24. Hard Rule

Response is scoped to evidence and blast radius.

---

# 25. Containment First

Prefer the narrowest action that reliably stops harm.

---

# 26. Example

If only notification abuse is confirmed:

```text
revoke notification capability
```

not:

```text
delete all extension data
```

---

# 27. Hard Rule

Containment does not destroy user data by default.

---

# 28. Quarantine State

```rust
pub enum ExtensionQuarantineState {
    None,
    RuntimeOnly,
    CapabilityRestricted,
    BackgroundDisabled,
    NetworkDisabled,
    FullExecutionBlocked,
}
```

---

# 29. Hard Rule

Quarantine state is explicit and revocable.

---

# 30. Runtime Quarantine

Stops code execution while preserving data.

---

# 31. Hard Rule

Quarantined extension runtime cannot make broker calls.

---

# 32. Capability Restriction

Can remove selected capabilities.

---

# 33. Hard Rule

Restricted capabilities propagate via grant epoch immediately.

---

# 34. Network Disable

Blocks external/network sinks.

---

# 35. Hard Rule

Network disable does not imply local data deletion.

---

# 36. Background Disable

Cancels queued jobs.

---

# 37. Hard Rule

No orphan background execution after quarantine.

---

# 38. Full Execution Block

Extension code does not run.

---

# 39. Platform-Controlled Recovery UI

Still available.

---

# 40. Hard Rule

Revoked/quarantined extension cannot render its own trusted recovery screen.

---

# 41. Emergency Disable

Reserved for confirmed or sufficiently severe risk.

---

# 42. Emergency Disable Authority

```rust
pub enum ExtensionEmergencyAuthority {
    PlatformSecurity,
    MarketplaceSecurity,
    TenantSecurity,
}
```

---

# 43. Hard Rule

Authority scope limited to owned domain.

---

# 44. No Universal Superuser

Hard rule.

---

# 45. Emergency Action

```rust
pub struct ExtensionEmergencyAction {
    pub incident: ExtensionIncidentId,
    pub authority: ExtensionEmergencyAuthority,
    pub action: ExtensionIncidentDecision,
    pub scope: ExtensionIncidentScope,
    pub expires_at: Option<Timestamp>,
}
```

---

# 46. Hard Rule

Emergency actions are signed, attributable, scoped, and auditable.

---

# 47. Dual Control

Required for platform-wide high-impact disable where feasible.

---

# 48. Hard Rule

No single online operator can silently disable arbitrary unrelated extensions at global scope.

---

# 49. Emergency Disable Propagation

Through signed revocation/incident policy.

---

# 50. Hard Rule

Anti-rollback epoch.

---

# 51. Emergency Policy Epoch

```rust
pub struct ExtensionEmergencyPolicyEpoch(pub u64);
```

---

# 52. Hard Rule

Older permissive policy cannot override newer disable.

---

# 53. Offline Enforcement

Cached signed emergency policy applies locally.

---

# 54. Hard Rule

Offline device respects known disable state.

---

# 55. Stale Device

May require rebootstrap before re-enabling extension.

---

# 56. Hard Rule

No stale device silently reinstates disabled package.

---

# 57. Detection Pipeline

```text
signal
→ validate
→ deduplicate
→ correlate technically
→ classify
→ triage
→ act
```

---

# 58. Hard Rule

Correlation is extension/package technical correlation, not user behavioral correlation.

---

# 59. Alert Deduplication

Multiple equivalent alerts collapse.

---

# 60. Hard Rule

No alert storm.

---

# 61. Detection Rule

```rust
pub struct ExtensionDetectionRule {
    pub rule_id: ExtensionDetectionRuleId,
    pub trigger: ExtensionIncidentTrigger,
    pub threshold: DetectionThreshold,
}
```

---

# 62. Hard Rule

Rules versioned and reviewed.

---

# 63. Detection Threshold

```rust
pub enum DetectionThreshold {
    CountWithinWindow { count: u32, window: Duration },
    Immediate,
    Composite(Vec<DetectionPredicate>),
}
```

---

# 64. Hard Rule

Rules cannot inspect forbidden private data classes.

---

# 65. Sandbox Violation

Immediate high-confidence signal.

---

# 66. Capability Violation

Examples:

```text
unauthorized file access
unauthorized network origin
stale capability token
cross-tenant request
```

---

# 67. Hard Rule

Repeated violations can escalate quarantine.

---

# 68. Privacy Violation

Examples:

```text
unexpected external sink
lock-screen secret leak
retention failure
forbidden telemetry field
```

---

# 69. Hard Rule

Privacy incidents receive explicit class, not hidden under generic security.

---

# 70. Supply-Chain Incident

Examples:

```text
dependency revocation
publisher key compromise
tampered package
registry compromise
```

---

# 71. Hard Rule

Identify affected package graph via Part 142 lineage.

---

# 72. Blast Radius Analysis

```rust
pub struct ExtensionBlastRadius {
    pub affected_packages: BTreeSet<ExtensionPackageId>,
    pub affected_versions: BTreeSet<ExtensionVersion>,
    pub affected_platforms: BTreeSet<PlatformClass>,
    pub affected_dependencies: BTreeSet<DependencyPackageId>,
}
```

---

# 73. Hard Rule

Blast radius uses technical deployment/install metadata minimized for privacy.

---

# 74. No Individual User List Unless Operationally Necessary

Hard rule.

---

# 75. Affected Installed Extension Discovery

Can be device-local.

---

# 76. Hard Rule

Central incident response need not enumerate all users to enforce revocation.

---

# 77. Containment Plan

```rust
pub struct ExtensionContainmentPlan {
    pub incident: ExtensionIncidentId,
    pub actions: Vec<ExtensionContainmentAction>,
}
```

---

# 78. Containment Action

```rust
pub enum ExtensionContainmentAction {
    Throttle,
    RevokeCapability(ExtensionPermissionClass),
    DisableNetwork,
    DisableBackground,
    QuarantineRuntime,
    BlockNewInstall,
    BlockUpdate,
    EmergencyDisable,
}
```

---

# 79. Hard Rule

Actions compose restrictively.

---

# 80. No Containment That Weakens Security Elsewhere

Hard rule.

---

# 81. User/Admin Notification

Required when impact is meaningful.

---

# 82. Hard Rule

Notification content truthful and privacy-minimized.

---

# 83. Example

```text
"Extension disabled due to a security issue."
```

instead of exposing sensitive forensic details.

---

# 84. Publisher Notification

Publisher may receive incident notice.

---

# 85. Hard Rule

Publisher does not receive user identity or private content unless explicitly authorized/required.

---

# 86. Publisher Response

Can provide:

```text
fix
explanation
new package
evidence
```

---

# 87. Hard Rule

Publisher cannot self-clear quarantine.

---

# 88. Incident Authority

```rust
pub enum ExtensionIncidentRole {
    IncidentCommander,
    SecurityLead,
    PrivacyLead,
    ReleaseLead,
    MarketplaceLead,
    ForensicsLead,
    Scribe,
}
```

---

# 89. Hard Rule

Roles are scoped and temporary.

---

# 90. Incident Commander

Coordinates response.

---

# 91. Hard Rule

Incident commander is not automatic root/admin of extension data.

---

# 92. Separation of Duties

High-risk actions may require second approval.

---

# 93. Hard Rule

Forensics access and disable authority can be separated.

---

# 94. Forensic Evidence

Evidence must be minimized.

---

# 95. Evidence Class

```rust
pub enum ExtensionForensicEvidenceClass {
    PackageArtifact,
    SignatureMetadata,
    DependencyGraph,
    RuntimeViolation,
    CrashSummary,
    RedactedLog,
    PolicyDecision,
    NetworkPolicyEvent,
    DataFlowViolation,
}
```

---

# 96. Hard Rule

Private message content is not default forensic evidence.

---

# 97. Forensic Evidence Record

```rust
pub struct ExtensionForensicEvidence {
    pub evidence_id: EvidenceRef,
    pub incident: ExtensionIncidentId,
    pub class: ExtensionForensicEvidenceClass,
    pub digest: Digest,
    pub collected_at: Timestamp,
}
```

---

# 98. Hard Rule

Evidence provenance explicit.

---

# 99. Chain of Custody

For high-assurance incidents.

---

# 100. Chain Record

```rust
pub struct ExtensionForensicCustodyRecord {
    pub evidence: EvidenceRef,
    pub actor_role: ExtensionIncidentRole,
    pub action: CustodyAction,
    pub timestamp: Timestamp,
}
```

---

# 101. Hard Rule

Custody records do not expose private user details.

---

# 102. Evidence Immutability

Content-addressed.

---

# 103. Hard Rule

Corrections via supersession, not mutation.

---

# 104. Forensic Snapshot

Can include extension package/runtime metadata.

---

# 105. Hard Rule

No whole-device memory dump by default.

---

# 106. Native Extension Memory Forensics

Only in exceptional high-assurance environment.

---

# 107. Hard Rule

Requires explicit policy/authorization and redaction.

---

# 108. WASM Forensics

Prefer:

```text
module digest
trap stack
host-call history class
policy violation class
```

---

# 109. Hard Rule

No full linear-memory capture by default.

---

# 110. Network Forensics

Allowed:

```text
approved origin class
connection attempt result
policy denial
```

---

# 111. Hard Rule

No raw anonymous route/peer IP tracing.

---

# 112. Data-Flow Forensics

Can identify:

```text
source class
sink class
policy decision
```

without content.

---

# 113. Hard Rule

Use metadata classification, not payload inspection.

---

# 114. User Reports

Structured.

---

# 115. Report Class

```rust
pub enum ExtensionUserReportClass {
    Spam,
    MisleadingUi,
    UnexpectedPermissionPrompt,
    PrivacyConcern,
    SecurityConcern,
    StabilityIssue,
}
```

---

# 116. Hard Rule

User report content is not automatically shared with publisher.

---

# 117. Abuse Report Evidence

Optional attachments require explicit user consent.

---

# 118. Hard Rule

No automatic upload of conversation context.

---

# 119. Incident Timeline

Record only meaningful events.

---

# 120. Hard Rule

Do not record every low-level runtime action.

---

# 121. Incident Timeline Event

```rust
pub struct ExtensionIncidentTimelineEvent {
    pub incident: ExtensionIncidentId,
    pub at: Timestamp,
    pub class: IncidentTimelineEventClass,
    pub evidence: Option<EvidenceRef>,
}
```

---

# 122. Timeline Event Class

```rust
pub enum IncidentTimelineEventClass {
    Detection,
    Escalation,
    Containment,
    Quarantine,
    EvidenceCollected,
    FixAvailable,
    RecoveryStarted,
    Reinstated,
    Closed,
}
```

---

# 123. Hard Rule

Timeline is operational record, not surveillance log.

---

# 124. Situation Snapshot

```rust
pub struct ExtensionIncidentSituation {
    pub known: Vec<String>,
    pub suspected: Vec<String>,
    pub unknown: Vec<String>,
}
```

---

# 125. Hard Rule

Known/suspected/unknown remain distinct.

---

# 126. No Motive Speculation

Hard rule.

---

# 127. Root Cause Analysis

Part 120 model applies.

---

# 128. Hard Rule

Root cause distinct from trigger.

---

# 129. Corrective Actions

Examples:

```text
patch extension
tighten sandbox rule
add capability check
change update policy
revoke dependency
improve test coverage
```

---

# 130. Hard Rule

Implemented != verified.

---

# 131. Recovery Eligibility

```rust
pub enum ExtensionRecoveryEligibility {
    NotReady,
    ReadyForLimitedRecovery,
    ReadyForFullRecovery,
}
```

---

# 132. Recovery Preconditions

Possible:

```text
fixed package verified
new package certified
affected key rotated
dependency issue resolved
migration/recovery tested
revocation state updated
```

---

# 133. Hard Rule

Recovery is evidence-based.

---

# 134. Recovery Plan

```rust
pub struct ExtensionRecoveryPlan {
    pub incident: ExtensionIncidentId,
    pub target_package: ExtensionPackageId,
    pub scope: ExtensionRecoveryScope,
    pub verification: Vec<RecoveryVerificationStep>,
}
```

---

# 135. Recovery Scope

```rust
pub enum ExtensionRecoveryScope {
    ThisDevice,
    LimitedCohort,
    TenantScoped,
    GlobalEligible,
}
```

---

# 136. Hard Rule

Recovery scope can be narrower than original deployment.

---

# 137. Reinstatement

Separate from recovery execution.

---

# 138. Hard Rule

Successful fixed package install does not automatically clear incident.

---

# 139. Reinstatement Decision

```rust
pub enum ExtensionReinstatementDecision {
    Denied,
    Limited,
    Full,
}
```

---

# 140. Hard Rule

Reinstatement can require multi-authority approval for high-risk incident.

---

# 141. Reinstatement Criteria

```text
root cause addressed
fix verified
certification valid
no active revocation
privacy/security evidence fresh
recovery test passed
```

---

# 142. Hard Rule

No reinstatement solely on publisher assurance.

---

# 143. Capability Restoration

Restored gradually.

---

# 144. Hard Rule

Do not automatically restore every historical high-risk permission.

---

# 145. Background Restoration

Re-enable jobs only after runtime/capability health verified.

---

# 146. Hard Rule

No automatic daemon/background restoration before health gate.

---

# 147. Network Restoration

Can be staged.

---

# 148. Hard Rule

External sink/network capability restored only after data-flow verification.

---

# 149. User Consent After Incident

May require renewed consent for sensitive capability.

---

# 150. Hard Rule

Incident recovery cannot silently re-consent on behalf of user.

---

# 151. Data Recovery

Quarantined extension data may need migration.

---

# 152. Hard Rule

Recovery preserves deletion tombstones/retention policy.

---

# 153. No Resurrection During Recovery

Hard rule.

---

# 154. Secret Rotation

If incident exposed credential risk:

```text
revoke secret
rotate key
rebind fresh SecretRef
```

---

# 155. Hard Rule

Old secret reference invalid after rotation.

---

# 156. Publisher Key Compromise

Actions:

```text
suspend publisher key
freeze new package publication
identify affected packages
require key rotation/reverification
```

---

# 157. Hard Rule

Publisher identity history preserved.

---

# 158. Package Signature Reissue

Does not automatically restore certification.

---

# 159. Hard Rule

Re-signed artifact still requires evidence reconciliation.

---

# 160. Dependency Compromise

Part 142 graph traversal identifies roots.

---

# 161. Hard Rule

Only affected dependency/package graph scoped.

---

# 162. Registry Compromise

May freeze resolution/install while keeping known-good offline packages subject to policy.

---

# 163. Hard Rule

No automatic trust in newly served metadata.

---

# 164. Marketplace Response

Possible:

```text
hide listing
block new installs
show security notice
require update
delist
```

---

# 165. Hard Rule

Marketplace action separate from package runtime revocation.

---

# 166. Certification Response

Part 144:

```text
suspend
revoke
supersede
```

---

# 167. Hard Rule

Certification state propagates to install/update gates.

---

# 168. Update Response

Part 143 may deliver emergency fixed package.

---

# 169. Hard Rule

Emergency update still exact-artifact verified.

---

# 170. State Sync Response

Part 141 propagates restrictive disable/revocation state.

---

# 171. Hard Rule

Remote state cannot clear local incident quarantine automatically.

---

# 172. Observability Response

Part 145 may increase local diagnostic retention temporarily within policy.

---

# 173. Hard Rule

Incident mode does not unlock forbidden telemetry classes.

---

# 174. Incident-Specific Telemetry Expansion

If allowed:

```text
must be explicitly pre-authorized
time-bounded
scope-bounded
privacy-reviewed
```

---

# 175. Hard Rule

No emergency “log everything”.

---

# 176. Forensic Collection Capability

Separate capability.

---

# 177. Hard Rule

Only platform-controlled security component may collect forensic metadata.

---

# 178. Extension Cannot Self-Collect Incident Evidence From Other Extensions

Hard rule.

---

# 179. Cross-Extension Incident

If one shared dependency affects several extensions, each remains separately scoped.

---

# 180. Hard Rule

No merged cross-extension user activity graph.

---

# 181. Incident Command

Part 119 applies.

---

# 182. Hard Rule

Extension incident command cannot override hard privacy/security floor.

---

# 183. Breakglass

If supported:

```text
scope
purpose
expiry
dual approval
audit
```

---

# 184. Hard Rule

Breakglass cannot decrypt user content without separate lawful/product architecture.

---

# 185. No Universal Decryption Key

Hard rule.

---

# 186. No Emergency Backdoor

Hard rule.

---

# 187. Incident Communications

Separate:

```text
internal technical status
publisher notice
user/admin notice
public advisory
```

---

# 188. Hard Rule

Each audience sees minimum necessary information.

---

# 189. Public Advisory

Can state:

```text
affected extension/version
impact class
required action
fixed version
```

---

# 190. Hard Rule

No exposure of individual users/tenants.

---

# 191. Tenant Incident Notice

Tenant-specific only when tenant scope relevant.

---

# 192. Hard Rule

Personal context remains separate.

---

# 193. SLA/SLO During Incident

Incident can suspend ordinary extension availability SLO.

---

# 194. Hard Rule

Security/privacy invariants remain non-negotiable.

---

# 195. Recovery Monitoring

After reinstatement, temporary enhanced aggregate monitoring.

---

# 196. Hard Rule

Still no private-content/user behavior capture.

---

# 197. Monitoring Window

Finite.

---

# 198. Hard Rule

Automatically returns to normal telemetry policy.

---

# 199. Residual Risk

Explicit.

---

# 200. Residual Risk Record

```rust
pub struct ExtensionResidualRisk {
    pub incident: ExtensionIncidentId,
    pub statement: String,
    pub compensating_controls: Vec<ControlRef>,
    pub expires_at: Option<Timestamp>,
}
```

---

# 201. Hard Rule

Hard invariant violations cannot be accepted as residual risk.

---

# 202. Incident Closure

Requires:

```text
containment verified
recovery state known
corrective actions tracked
residual risk documented
evidence archived
```

---

# 203. Hard Rule

Closed does not erase incident.

---

# 204. Evidence Archive

Part 126.

---

# 205. Hard Rule

Incident evidence content-addressed and retention-governed.

---

# 206. Forensic Retention

Long enough for investigation, not indefinite by default.

---

# 207. Hard Rule

Sensitive evidence retention minimized.

---

# 208. Cryptographic Erasure

Where evidence policy permits after retention period.

---

# 209. Hard Rule

Audit digest/history may remain after raw evidence deletion.

---

# 210. Incident Metrics

Safe aggregate:

```text
incident count by class
containment latency
quarantine count
recovery duration class
```

---

# 211. Forbidden:

```text
user identity
developer blame score
publisher employee performance
private incident payload
```

---

# 212. Hard Rule

Security operations metrics are process/system metrics.

---

# 213. No Publisher Ranking From Incident Count

Hard rule.

---

# 214. Abuse Detection Metrics

Aggregate only.

---

# 215. Hard Rule

No user-report-based global publisher score.

---

# 216. Incident SLOs

Examples:

```text
confirmed critical disable propagation within target
quarantine activation within target
revocation cache update within target
recovery verification within target
```

---

# 217. Security SLO

```text
0 quarantined runtime retains broker authority
0 emergency-disabled package executes after policy convergence
0 revoked key signs newly accepted package
```

---

# 218. Privacy SLO

```text
0 incident mode enables forbidden telemetry
0 forensic evidence contains private payload by default
0 support/incident system builds user behavior timelines
```

---

# 219. Failure Modes

```text
false positive
overbroad quarantine
stale disable state
incomplete evidence
unsafe reinstatement
```

---

# 220. False Positive

Contain narrowly, review, reverse safely if cleared.

---

# 221. Overbroad Quarantine

Scope reconciliation can reduce containment.

---

# 222. Stale Disable State

Anti-rollback epoch + rebootstrap.

---

# 223. Incomplete Evidence

Known/suspected/unknown maintained.

---

# 224. Unsafe Reinstatement

Blocked by explicit recovery criteria.

---

# 225. User Appeal / Publisher Appeal

Marketplace/governance appeal path.

---

# 226. Hard Rule

Appeal does not auto-lift containment.

---

# 227. Appeal Evidence

Structured and scoped.

---

# 228. Hard Rule

Historical incident record preserved even if decision changes.

---

# 229. Testing

Need extension-incident-response testkit.

Required scenarios:

```text
sandbox violation
publisher key compromise
network exfiltration attempt
false positive
emergency disable propagation
recovery/reinstatement
```

---

# 230. Quarantine Test

Runtime loses all broker authority immediately.

---

# 231. Capability Restriction Test

Selected capability removed without affecting unrelated allowed scope.

---

# 232. Background Test

Queued jobs cancelled on quarantine.

---

# 233. Network Test

Network-disable containment blocks external sink.

---

# 234. Revocation Test

Offline device with known revocation cannot run package.

---

# 235. Anti-Rollback Test

Older permissive policy rejected.

---

# 236. Publisher Key Test

Compromised key cannot authorize new package.

---

# 237. Evidence Test

Forensic bundle contains no private payload.

---

# 238. User Report Test

No conversation context uploaded by default.

---

# 239. False Positive Test

Containment can be safely narrowed/reversed with audit.

---

# 240. Recovery Test

Fixed package must pass certification/recovery gates before reinstatement.

---

# 241. Permission Restoration Test

Historical sensitive grants are not automatically restored.

---

# 242. Telemetry Test

Incident mode does not enable forbidden telemetry classes.

---

# 243. Multi-Device Test

Restrictive incident state propagates; local quarantine cannot be remotely cleared by normal sync.

---

# 244. Fuzzing

Fuzz:

```text
incident records
emergency actions
quarantine policies
forensic manifests
recovery plans
```

---

# 245. Property Tests

Properties:

```text
quarantined runtime can never invoke privileged broker call
emergency-disable epoch can never be downgraded by older state
publisher self-action can never clear platform quarantine
recovery can never restore revoked capability without current policy approval
```

---

# 246. Formal Verification Targets

Strong candidates:

```text
incident lifecycle
quarantine/revocation precedence
emergency disable propagation
reinstatement gates
```

---

# 247. Kani Candidate

containment/authority/restoration invariants.

---

# 248. TLA+ Candidate

```text
detect → triage → contain → investigate → recover → reinstate/close
```

---

# 249. Loom Candidate

Concurrent:

```text
runtime host call
quarantine activation
grant epoch change
emergency policy refresh
```

---

# 250. Performance

Incident controls must be fast but not on normal hot path unless enforcement requires.

---

# 251. Enforcement Cache

Signed local incident/revocation state.

---

# 252. Hard Rule

Cache invalidated promptly on security epoch change.

---

# 253. Local Enforcement

Prefer local deny/quarantine once signed policy known.

---

# 254. Hard Rule

Do not require online round-trip for every broker call.

---

# 255. Incident Store

Separate:

```text
incident metadata
containment decisions
forensic evidence refs
recovery plans
reinstatement decisions
audit timeline
```

---

# 256. Hard Rule

No private user-content warehouse.

---

# 257. Evidence Blob Store

Content-addressed, encrypted where required.

---

# 258. Hard Rule

Access purpose-scoped.

---

# 259. Partitioning

By:

```text
extension
package
incident
publisher
dependency
```

---

# 260. Hard Rule

No user/person analytics partition.

---

# 261. Crate Layout

Recommended:

```text
crates/
├── siar-extension-incident-core/
├── siar-extension-detection/
├── siar-extension-abuse/
├── siar-extension-quarantine/
├── siar-extension-emergency-disable/
├── siar-extension-forensics/
├── siar-extension-recovery/
├── siar-extension-reinstatement/
├── siar-extension-incident-observability/
└── siar-extension-incident-testkit/
```

---

# 262. `siar-extension-incident-core`

Owns:

```text
ExtensionIncidentId
ExtensionIncidentClass
ExtensionIncidentState
ExtensionIncidentSeverity
```

---

# 263. `siar-extension-detection`

Technical detection rules/thresholds/dedup.

---

# 264. `siar-extension-abuse`

Notification/prompt/resource/policy abuse classification.

---

# 265. `siar-extension-quarantine`

Capability/runtime/network/background restriction state.

---

# 266. `siar-extension-emergency-disable`

Signed anti-rollback emergency policy and authority.

---

# 267. `siar-extension-forensics`

Evidence collection, custody, redaction, lineage.

---

# 268. `siar-extension-recovery`

Fixed package validation/data/secret recovery.

---

# 269. `siar-extension-reinstatement`

Evidence-backed restoration of capabilities/runtime.

---

# 270. `siar-extension-incident-observability`

Aggregate incident process health only.

---

# 271. `siar-extension-incident-testkit`

quarantine/revocation/forensics/recovery/privacy/formal tests.

---

# 272. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Extension alerts, incidents, abuse signals, containment, quarantine, emergency disable, package revocation, forensic evidence, recovery, and reinstatement are distinct typed states and no single alert automatically implies maximum containment or permanent revocation.
2. Incident response applies the narrowest reliable containment that stops the observed risk while preserving user data, unrelated extension capabilities, and unaffected tenants/devices whenever evidence permits.
3. Quarantine, capability restriction, network disable, background disable, and emergency-disable state are enforced locally through signed anti-rollback policy/grant epochs so stale runtimes or offline devices cannot continue privileged execution.
4. No incident role—including incident commander, security lead, marketplace operator, or publisher—receives universal root access, plaintext secrets, user message content, or global decryption capability merely because an incident exists.
5. Forensic evidence defaults to package/runtime/policy/crash/dependency/data-flow metadata; private payloads, whole-device memory dumps, raw anonymous route data, social graphs, and conversation history are excluded unless a separately governed exceptional process explicitly requires them.
6. Publisher reports, user reports, abuse signals, and weak-confidence alerts cannot silently become user surveillance, publisher scoring, or motive judgments; known/suspected/unknown and evidence confidence remain explicit throughout the incident.
7. Emergency actions are signed, scoped, time/epoch governed, auditable, and subject to separation of duties for broad impact; no single online operator can silently disable arbitrary unrelated extensions globally.
8. Recovery and reinstatement require a fixed/verified package, fresh certification/security/privacy evidence, resolved revocation state, validated migration/recovery behavior, and current policy approval; publisher assurance alone cannot clear quarantine.
9. Capability/background/network restoration after an incident is deliberate and may require renewed consent; stale runtime tokens, compromised secrets, revoked publisher keys, or historical high-risk grants can never be resurrected automatically.
10. Incident mode cannot enable forbidden telemetry, unrestricted logging, hidden remote debugging, or full payload capture; temporary diagnostic expansion, if allowed, remains pre-authorized, scoped, time-bounded, redacted, and privacy-reviewed.
11. Incident metrics, timelines, evidence stores, and post-incident records are technical/process artifacts and cannot become per-user behavior histories, employee/developer blame systems, tenant surveillance, or publisher popularity/risk rankings.
12. Extension incident response integrates with runtime sandboxing, permissions, data governance, IPC, background jobs, notifications, UI, state sync, dependency/revocation graphs, update orchestration, certification, observability, marketplace governance, supply-chain security, PIR, and release evidence without creating a side channel around SIAR's security, privacy, anonymity, local-first, or tenant-isolation guarantees.
```

---

# 273. Initial Production Scope

Implement first:

```text
typed incident lifecycle
technical incident triggers
abuse signal classes
evidence confidence
scoped quarantine states
capability/network/background restriction
full runtime quarantine
signed emergency-disable policy
anti-rollback emergency epoch
publisher key/package revocation integration
technical blast-radius analysis
forensic evidence registry
redacted incident bundles
chain-of-custody metadata
user/publisher notification paths
recovery plan
reinstatement criteria
secret/key rotation hooks
multi-device restrictive-state propagation
incident-to-update/certification hooks
privacy-safe incident metrics
extension-incident testkit
```

Then add:

```text
advanced multi-authority emergency workflows
privacy-preserving compromise correlation
formal containment/reinstatement proofs
automated dependency-graph incident fanout
incident simulation exercises
publisher transparency reports
```

---

# 274. Definition of Done

Part 146 is complete when:

- extension incidents have typed state/severity/scope;
- abuse/security/privacy triggers are separate;
- quarantine is scoped and reversible;
- emergency disable is signed and anti-rollback;
- no incident role becomes universal admin;
- forensic evidence defaults to technical metadata;
- blast radius is technical, not user behavioral;
- recovery/reinstatement require fresh evidence;
- sensitive grants/secrets are not auto-restored;
- incident mode cannot unlock forbidden telemetry;
- restrictive state propagates across devices;
- evidence, metrics, and timelines do not become surveillance;
- quarantine/revocation/recovery/privacy/fuzz/formal tests are specified.

---

# 275. Final Architecture

```text
                 EXTENSION SIGNALS
                        │
                        ▼
                   DETECTION
                        │
             ┌──────────┼──────────┐
             │          │          │
          SECURITY    PRIVACY     ABUSE
             │          │          │
             └──────────┼──────────┘
                        ▼
                     TRIAGE
                        │
                        ▼
                  CONTAINMENT
             ┌──────────┼──────────┐
             │          │          │
         RESTRICT    QUARANTINE   DISABLE
             │          │          │
             └──────────┼──────────┘
                        ▼
                   FORENSICS
                        │
                        ▼
                 RECOVERY / REINSTATE
```

Extension-incident safety model:

```text
technical detection
+
scoped containment
+
signed quarantine
+
anti-rollback disable
+
privacy-minimized forensics
+
evidence-backed recovery
+
deliberate reinstatement
+
post-incident improvement
```

not:

```text
give responders unlimited access, collect everything, delete extension data immediately, and trust the publisher to declare the incident fixed
```

---

# 276. Final Principle

Extension security operations are trustworthy when SIAR can answer **what happened, what evidence supports that conclusion, what authority was removed, what data was preserved, what must be fixed, and what evidence is required before authority returns**.

The correct model is:

```text
detect technical violations
+
triage with confidence
+
contain narrowly
+
quarantine decisively when needed
+
preserve evidence without collecting private content
+
rotate compromised trust
+
verify the fix
+
restore authority gradually
+
never turn incident response into universal surveillance or emergency backdoor
```

This architecture gives SIAR a privacy-preserving extension security-operations foundation for incident detection, abuse response, quarantine, emergency disable, forensics, recovery, reinstatement, publisher/dependency compromise handling, and post-incident improvement while preserving the anonymity, local-first, least-authority, runtime, permission, data-governance, update, certification, observability, and anti-surveillance guarantees established across Parts 34–145.
