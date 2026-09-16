# Core System Architecture Part 65 — Anonymous Network Documentation, Operator Runbooks, Incident Playbooks, Architecture Decision Records & Production Knowledge Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 65  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–64  

**Primary purpose:** define how SIAR captures, versions, validates, distributes, and operationalizes technical knowledge across architecture documents, operator runbooks, incident playbooks, release notes, troubleshooting guides, ADRs, ownership maps, escalation procedures, service catalogs, and machine-readable production metadata.

---

# 1. Purpose

Production systems fail when knowledge exists only in people's heads.

Anonymous networks are especially sensitive because failures can involve:

```text
privacy degradation
provider compromise
key rotation
directory outage
mailbox corruption
federation partition
governance conflict
release rollback
resource saturation
```

Documentation must therefore be operational infrastructure.

The governing principle is:

> **SIAR production knowledge must be versioned, testable, release-linked, privacy-safe, and executable enough that operators can act correctly under stress without relying on tribal knowledge.**

---

# 2. Architectural Position

```text
Architecture & ADRs
        │
        ▼
Service Catalog
        │
        ▼
Runbooks / Playbooks
        │
        ▼
Operational Automation
        │
        ▼
Incidents / Releases
        │
        ▼
Feedback / Updates
```

---

# 3. Core Knowledge Classes

```rust
pub enum KnowledgeClass {
    Architecture,
    Adr,
    Runbook,
    IncidentPlaybook,
    Troubleshooting,
    ServiceCatalog,
    ReleaseNote,
    MigrationGuide,
    PolicyReference,
    OperatorChecklist,
}
```

---

# 4. Non-Goals

Part 65 does not accept:

```text
unversioned wiki pages as sole source of truth
secret-bearing runbooks
operator procedures that require memorized steps
release notes disconnected from actual artifacts
incident playbooks that are never tested
```

---

# 5. Knowledge Repository

Recommended tree:

```text
docs/
├── architecture/
├── adr/
├── services/
├── runbooks/
├── incidents/
├── troubleshooting/
├── releases/
├── migrations/
├── policies/
└── operators/
```

---

# 6. Architecture Docs

Describe:

```text
system boundaries
data flows
trust boundaries
protocols
failure modes
security/privacy invariants
```

---

# 7. ADRs

Architecture Decision Records capture:

```text
decision
context
alternatives
tradeoffs
consequences
status
```

---

# 8. ADR Status

```rust
pub enum AdrStatus {
    Proposed,
    Accepted,
    Superseded,
    Rejected,
    Deprecated,
}
```

---

# 9. ADR Identifier

```rust
pub struct AdrId(pub u32);
```

---

# 10. ADR Naming

Example:

```text
ADR-0042-use-postcard-for-internal-binary-wire.md
```

---

# 11. ADR Template

Required fields:

```text
Title
Status
Date
Context
Decision
Alternatives
Consequences
Security impact
Privacy impact
Operational impact
Supersedes
Superseded by
```

---

# 12. Architecture Change

Must update relevant ADRs.

---

# 13. No Silent Architecture Drift

Hard rule.

---

# 14. Service Catalog

Every production service has entry.

---

# 15. Service Catalog Entry

```rust
pub struct ServiceCatalogEntry {
    pub service: ServiceId,
    pub class: ServiceClass,
    pub owner: OwnershipRef,
    pub criticality: Criticality,
    pub dependencies: Vec<ServiceId>,
    pub runbook: KnowledgeRef,
    pub slo: SloRef,
}
```

---

# 16. Service Classes

```rust
pub enum ServiceClass {
    MixNode,
    Gateway,
    Mailbox,
    Relay,
    Bulk,
    Bridge,
    Directory,
    Governance,
    Settlement,
    ControlPlane,
    FederationGateway,
}
```

---

# 17. Criticality

```rust
pub enum Criticality {
    Tier0,
    Tier1,
    Tier2,
    Tier3,
}
```

---

# 18. Tier0

Examples:

```text
trust roots
governance signing
directory authority
```

---

# 19. Tier1

Examples:

```text
mailbox
mixnet gateway
relay
```

---

# 20. Tier2/Tier3

Lower-impact support systems.

---

# 21. Ownership

Must be explicit.

---

# 22. Ownership Record

```rust
pub struct OwnershipRef {
    pub primary: TeamRef,
    pub secondary: Option<TeamRef>,
    pub escalation: EscalationRef,
}
```

---

# 23. No Ownerless Service

Hard rule.

---

# 24. Runbook

Answers:

```text
what is wrong?
how do I verify?
what is safe to do?
what must never be done?
how do I recover?
```

---

# 25. Runbook Sections

Required:

```text
Purpose
Scope
Preconditions
Symptoms
Diagnostics
Safe actions
Unsafe actions
Escalation
Rollback/recovery
Verification
```

---

# 26. Runbook Metadata

```rust
pub struct RunbookMetadata {
    pub id: KnowledgeId,
    pub service: ServiceId,
    pub applicable_versions: VersionRange,
    pub last_verified: Timestamp,
    pub owner: OwnershipRef,
}
```

---

# 27. Version Binding

Runbook must declare supported release range.

---

# 28. Release Mismatch

Operator UI should warn.

---

# 29. Runbook Freshness

Track.

---

# 30. Freshness State

```rust
pub enum KnowledgeFreshness {
    Current,
    ReviewDue,
    Stale,
    InvalidForRelease,
}
```

---

# 31. Review Period

Depends on criticality.

---

# 32. Tier0

Review frequently.

---

# 33. Tier3

Can review less often.

---

# 34. Incident Playbook

Different from runbook.

---

# 35. Runbook

Routine known operation.

---

# 36. Incident Playbook

High-pressure coordinated response.

---

# 37. Incident Playbook Types

```rust
pub enum IncidentPlaybookType {
    Security,
    Privacy,
    Availability,
    DataIntegrity,
    Governance,
    Capacity,
    SupplyChain,
    Legal,
}
```

---

# 38. Incident Severity

```rust
pub enum IncidentSeverity {
    Sev0,
    Sev1,
    Sev2,
    Sev3,
}
```

---

# 39. Sev0

Catastrophic trust/privacy compromise.

---

# 40. Incident Roles

```text
Incident Commander
Operations Lead
Security Lead
Privacy Lead
Communications Lead
Scribe
```

---

# 41. Role Separation

One person may hold multiple roles in small team.

---

# 42. But

Critical actions still need approval rules.

---

# 43. Incident Timeline

Append-only.

---

# 44. Timeline Event

```rust
pub struct IncidentTimelineEvent {
    pub timestamp: Timestamp,
    pub actor: OperatorAdminId,
    pub action: IncidentAction,
    pub result: IncidentActionResult,
}
```

---

# 45. No User Content In Timeline

Hard rule.

---

# 46. Incident State

```rust
pub enum IncidentState {
    Detected,
    Triaged,
    Contained,
    Mitigating,
    Recovering,
    Monitoring,
    Closed,
}
```

---

# 47. Privacy Incident

Needs separate handling.

---

# 48. Privacy Incident Examples

```text
identifier leaked to telemetry
unexpected direct fallback
route metadata persisted
```

---

# 49. Security Incident Examples

```text
signing key compromise
provider intrusion
artifact substitution
```

---

# 50. Governance Incident

Examples:

```text
quorum conflict
authority compromise
policy equivocation
```

---

# 51. Incident Playbook Template

Required:

```text
Trigger
Impact
Immediate containment
Privacy/security constraints
Actions
Escalation
Evidence preservation
Recovery
Communication
Post-incident follow-up
```

---

# 52. Unsafe Actions Section

Mandatory.

---

# 53. Why

Operators under stress often improvise.

---

# 54. Example Unsafe Action

```text
"Do not disable anonymity routing to restore availability."
```

---

# 55. Runbook Automation

Many steps should be executable.

---

# 56. Automation Reference

```rust
pub struct RunbookAutomationRef {
    pub command: SafeOperatorCommand,
    pub required_role: OperatorRole,
}
```

---

# 57. No Copy-Paste Shell As Primary Automation

Hard rule.

---

# 58. Operator CLI

Typed.

---

# 59. Example

```text
siar-admin mailbox drain --provider <id>
```

---

# 60. Command Safety

Command validates:

```text
target
role
environment
policy
```

---

# 61. Dry Run

Preferred.

---

# 62. Runbook Step

```rust
pub struct RunbookStep {
    pub title: String,
    pub command: Option<RunbookAutomationRef>,
    pub verification: Option<VerificationProcedure>,
}
```

---

# 63. Verification After Action

Mandatory for critical step.

---

# 64. Example

```text
rotate credential
→ verify old revoked
→ verify new active
```

---

# 65. Troubleshooting Guide

Symptom-oriented.

---

# 66. Example

```text
mailbox deposits are slow
relay calls fail after handshake
directory snapshot rejected
```

---

# 67. Troubleshooting Flow

```text
symptom
→ likely causes
→ safe diagnostics
→ escalation
```

---

# 68. No User-Level Debugging By Default

Hard rule.

---

# 69. Diagnostics

Use:

```text
aggregate metrics
synthetic probes
safe logs
```

---

# 70. Support Guide

Separate from operator runbook.

---

# 71. Support Access

Less privileged.

---

# 72. No Secret/Infrastructure Mutation

Hard rule.

---

# 73. Release Notes

Generated from:

```text
commits
ADRs
migrations
protocol changes
security changes
```

---

# 74. Release Note Sections

```text
Highlights
Breaking changes
Security/privacy changes
Operator actions
Migration steps
Known issues
Rollback constraints
```

---

# 75. Artifact Binding

Release note references exact release manifest.

---

# 76. No Generic Release Note

Hard rule.

---

# 77. Migration Guide

Needed for:

```text
schema changes
protocol changes
storage migrations
deployment changes
```

---

# 78. Migration Guide Metadata

```rust
pub struct MigrationGuideMetadata {
    pub from: VersionRange,
    pub to: SoftwareVersion,
    pub reversible: bool,
}
```

---

# 79. Reversibility

Explicit.

---

# 80. No Assumed Rollback

Hard rule.

---

# 81. Operator Checklist

Useful for:

```text
release
disaster recovery
key rotation
new region
new provider
```

---

# 82. Checklist Type

```rust
pub struct OperatorChecklist {
    pub id: KnowledgeId,
    pub items: Vec<ChecklistItem>,
}
```

---

# 83. Checklist Completion

Can be recorded for critical operations.

---

# 84. Architecture Knowledge Graph

Useful concept.

---

# 85. Nodes

```text
services
ADRs
runbooks
SLOs
policies
releases
incidents
```

---

# 86. Edges

```text
depends on
owned by
documented by
supersedes
validated by
```

---

# 87. Machine-Readable Knowledge Index

Recommended.

---

# 88. Knowledge Index Entry

```rust
pub struct KnowledgeIndexEntry {
    pub id: KnowledgeId,
    pub class: KnowledgeClass,
    pub path: String,
    pub owner: OwnershipRef,
    pub version_range: Option<VersionRange>,
}
```

---

# 89. Documentation Build

Docs are built artifact.

---

# 90. Static Site / Offline Bundle

Supported.

---

# 91. Offline Operator Docs

Critical.

---

# 92. Why

Incident may coincide with control-plane outage.

---

# 93. Offline Bundle

Contains:

```text
runbooks
playbooks
service catalog
release notes
trust fingerprints
```

---

# 94. No Secrets In Offline Docs

Hard rule.

---

# 95. Sensitive Runbooks

Can be access-controlled.

---

# 96. Redaction Classes

```rust
pub enum DocumentationSensitivity {
    Public,
    Internal,
    Restricted,
    SecretForbidden,
}
```

---

# 97. SecretForbidden

Means:

```text
secrets must never appear
```

---

# 98. Restricted Docs

May contain:

```text
internal topology
admin endpoints
```

---

# 99. Public Docs

No operationally sensitive detail beyond intended publication.

---

# 100. Documentation Linter

Mandatory.

---

# 101. Linter Checks

```text
broken links
stale version range
missing owner
missing runbook
secret patterns
unsafe command patterns
```

---

# 102. Secret Scanner

Runs on docs.

---

# 103. No Embedded Tokens

Hard rule.

---

# 104. Link Integrity

CI.

---

# 105. Runbook Command Validation

CLI examples tested.

---

# 106. Example

If docs say:

```text
siar-admin provider drain
```

CI ensures command still exists.

---

# 107. Documentation Tests

Executable snippets.

---

# 108. Rust Examples

Use doctests where suitable.

---

# 109. Shell Examples

Validate in test harness.

---

# 110. No Untested Critical Command Snippet

Hard rule.

---

# 111. ADR Consistency

A superseded ADR must link replacement.

---

# 112. Architecture Diagram Versioning

Text-based where practical.

---

# 113. Mermaid

Possible.

---

# 114. Graphviz

Possible.

---

# 115. Generated Diagrams

Better for inventories.

---

# 116. No Diagram As Only Source Of Truth

Hard rule.

---

# 117. Service Dependency Graph

Generate from service catalog.

---

# 118. Ownership Graph

Generate.

---

# 119. Runbook Coverage Graph

Generate.

---

# 120. Missing Runbook

Release warning/block depending criticality.

---

# 121. Knowledge Coverage Policy

```rust
pub struct KnowledgeCoveragePolicy {
    pub require_runbook_for_tier0: bool,
    pub require_playbook_for_critical_incident: bool,
}
```

---

# 122. Production Gate

Tier0/Tier1 service without runbook:

```text
block release
```

recommended.

---

# 123. Incident Playbook Drill

Periodic.

---

# 124. Drill Types

```text
tabletop
staging
controlled production
```

---

# 125. Tabletop

Human decision exercise.

---

# 126. Staging Drill

Executable recovery.

---

# 127. Controlled Production Drill

Limited safe failure drill.

---

# 128. Drill Record

```rust
pub struct RunbookDrillRecord {
    pub knowledge_id: KnowledgeId,
    pub executed_at: Timestamp,
    pub outcome: DrillOutcome,
}
```

---

# 129. Drill Outcome

```rust
pub enum DrillOutcome {
    Passed,
    PassedWithIssues,
    Failed,
}
```

---

# 130. Failed Drill

Triggers doc/runbook update.

---

# 131. Incident Postmortem

Required for significant incident.

---

# 132. Postmortem Sections

```text
Summary
Impact
Detection
Timeline
Root cause
Contributing factors
What worked
What failed
Actions
Privacy/security impact
```

---

# 133. Blamelessness

Focus on system/process.

---

# 134. But

Do not erase accountability for intentional misuse.

---

# 135. Corrective Actions

Must have:

```text
owner
deadline
priority
```

---

# 136. Action Item

```rust
pub struct CorrectiveAction {
    pub id: ActionItemId,
    pub owner: OwnershipRef,
    pub due: Timestamp,
    pub status: ActionItemStatus,
}
```

---

# 137. Close Criteria

Evidence required.

---

# 138. Incident → Test

Part 64 integration.

---

# 139. Incident → Runbook

Update if missing/incorrect.

---

# 140. Incident → ADR

If architecture changed.

---

# 141. Incident Knowledge Loop

```text
incident
→ postmortem
→ corrective action
→ test
→ docs
→ release
```

---

# 142. Documentation Versioning

Git-based.

---

# 143. Review

Code review equivalent.

---

# 144. Critical Docs

Require domain owner approval.

---

# 145. Privacy/Security Docs

Require privacy/security reviewer where material.

---

# 146. Knowledge Release

Docs version tied to software release.

---

# 147. Documentation Release Manifest

```rust
pub struct DocumentationManifest {
    pub software_version: SoftwareVersion,
    pub docs_commit: GitCommitHash,
    pub knowledge_digest: [u8; 32],
}
```

---

# 148. Runtime Link

Node/admin UI can expose matching runbook version.

---

# 149. Example

```text
Running: 1.8.2
Runbook set: 1.8.x
```

---

# 150. Mismatch Warning

Required.

---

# 151. Operator Console Integration

Contextual docs.

---

# 152. Example

On mailbox degraded state:

```text
open Mailbox Saturation Runbook
```

---

# 153. No Auto-Execute Without Confirmation

Hard rule.

---

# 154. Safe Automation

Can prefill target/action.

---

# 155. Knowledge Search

Local/internal.

---

# 156. Search Index

Docs only.

---

# 157. No User Data In Knowledge Search

Hard rule.

---

# 158. Offline Search

Useful.

---

# 159. Embeddings/AI Search

Optional.

---

# 160. Privacy Constraint

Do not send internal runbooks to external AI by default.

---

# 161. Local AI

Potential.

---

# 162. External AI

Requires explicit policy.

---

# 163. Generated Answers

Must link source docs.

---

# 164. No Autonomous Incident Action Based Solely On AI

Hard rule.

---

# 165. Operator Knowledge Assistant

Can:

```text
retrieve runbook
summarize procedure
explain error
```

---

# 166. Cannot:

```text
bypass approvals
invent secret
override governance
```

---

# 167. Knowledge Freshness Automation

CI can detect:

```text
service changed
runbook untouched
```

---

# 168. Heuristic

If code path changes for critical service:

```text
request doc review
```

---

# 169. ADR Trigger

Architecture-impact label.

---

# 170. Release PR Template

Ask:

```text
ADR needed?
runbook update?
migration guide?
```

---

# 171. No Hidden Operational Change

Hard rule.

---

# 172. Ownership Rotation

Team changes.

---

# 173. Ownership Update

Must be quick.

---

# 174. Escalation Tree

Explicit.

---

# 175. Escalation Level

```rust
pub enum EscalationLevel {
    Primary,
    Secondary,
    Security,
    Privacy,
    Governance,
    Legal,
}
```

---

# 176. Escalation Policy

Incident-specific.

---

# 177. Contact Privacy

Do not publish personal phone/email in public docs.

---

# 178. Internal Contact Alias

Preferred.

---

# 179. On-Call System

Can integrate.

---

# 180. On-Call Data

Separate operational identity.

---

# 181. No User Identity Linkage

Hard rule.

---

# 182. Decision Log

During incident.

---

# 183. Decision Entry

```rust
pub struct IncidentDecision {
    pub decision: String,
    pub rationale: String,
    pub approvers: Vec<OperatorAdminId>,
    pub timestamp: Timestamp,
}
```

---

# 184. Sensitive Details

Redacted appropriately.

---

# 185. Legal Hold

Part 55 may apply to incident records.

---

# 186. Evidence Preservation

Playbook must specify.

---

# 187. But

Do not start collecting new user traffic.

---

# 188. Hard rule.

---

# 189. Runbook Safety Classes

```rust
pub enum RunbookRiskClass {
    SafeRoutine,
    Elevated,
    HighRisk,
    Critical,
}
```

---

# 190. SafeRoutine

Examples:

```text
restart stateless mirror
```

---

# 191. Elevated

Examples:

```text
drain relay pool
```

---

# 192. HighRisk

Examples:

```text
rotate provider signing key
```

---

# 193. Critical

Examples:

```text
root rollover
governance authority replacement
```

---

# 194. Approval Policy

Mapped by risk.

---

# 195. Critical Runbook

Requires:

```text
multi-party
preconditions
rollback/recovery
```

---

# 196. Runbook Step Typing

```rust
pub enum RunbookStepType {
    Observe,
    Validate,
    Mutate,
    Escalate,
    Verify,
}
```

---

# 197. Mutation Step

Must declare risk.

---

# 198. Verification Step

Required after mutation.

---

# 199. Runbook Engine

Optional.

---

# 200. Machine-Readable Runbook

Could be YAML/RON-like schema.

---

# 201. Recommended

Human Markdown + machine-readable front matter.

---

# 202. Example Metadata

```text
id
service
owner
risk
versions
required roles
```

---

# 203. Markdown Body

Human explanation.

---

# 204. Automation References

Typed.

---

# 205. Runbook Parser

```rust
pub trait RunbookParser {
    fn parse(
        &self,
        source: &str,
    ) -> Result<RunbookDocument, KnowledgeError>;
}
```

---

# 206. Knowledge Registry

```rust
pub trait KnowledgeRegistry {
    fn get(
        &self,
        id: KnowledgeId,
    ) -> Result<KnowledgeDocument, KnowledgeError>;

    fn for_service(
        &self,
        service: ServiceId,
    ) -> Vec<KnowledgeRef>;
}
```

---

# 207. Freshness Evaluator

```rust
pub trait KnowledgeFreshnessEvaluator {
    fn evaluate(
        &self,
        doc: &KnowledgeDocument,
        release: SoftwareVersion,
    ) -> KnowledgeFreshness;
}
```

---

# 208. Incident Playbook Selector

```rust
pub trait IncidentPlaybookSelector {
    fn select(
        &self,
        incident: IncidentClass,
        severity: IncidentSeverity,
    ) -> Result<KnowledgeRef, KnowledgeError>;
}
```

---

# 209. Knowledge CI

Checks:

```text
metadata
links
ownership
versions
commands
secret leaks
```

---

# 210. Docs Build Failure

Blocks merge for critical docs.

---

# 211. Broken Critical Link

Release blocker.

---

# 212. Stale Critical Runbook

Release blocker.

---

# 213. Runbook Command Drift

Release blocker.

---

# 214. Documentation Security

Threats:

```text
secret leakage
malicious instruction
stale unsafe procedure
tampered offline bundle
```

---

# 215. Signed Offline Docs

Recommended.

---

# 216. Knowledge Bundle

```rust
pub struct KnowledgeBundle {
    pub manifest: DocumentationManifest,
    pub documents: Vec<KnowledgeDocument>,
    pub signature: DocumentationSignature,
}
```

---

# 217. Offline Verification

Operators can verify bundle.

---

# 218. No Unsigned Emergency Wiki Dump

Hard rule for critical runbooks.

---

# 219. Tamper Detection

Hash/signature.

---

# 220. Knowledge Access Control

By sensitivity.

---

# 221. Public

Anyone.

---

# 222. Internal

Authenticated operator.

---

# 223. Restricted

Role-scoped.

---

# 224. SecretForbidden

Any secret-like content rejected.

---

# 225. Training Material

Separate from runbooks.

---

# 226. Training Docs

Explain concepts.

---

# 227. Runbooks

Action-oriented.

---

# 228. Onboarding

New operator must learn:

```text
privacy invariants
control-plane safety
incident roles
release process
```

---

# 229. Operator Certification

Optional internal process.

---

# 230. Tabletop Qualification

Useful for Tier0 access.

---

# 231. Access Review

Privilege tied to training/role.

---

# 232. No Permanent Elevated Access

Part 61 integration.

---

# 233. Knowledge Metrics

Safe:

```text
runbook coverage
stale docs count
drill pass rate
broken links
```

---

# 234. Forbidden Metrics

No:

```text
user incident drilldown
user message content
```

---

# 235. Knowledge SLOs

Examples:

```text
100% Tier0 services have current runbook
100% Sev0 incident classes have playbook
<5% stale Tier1 docs
```

---

# 236. Documentation Debt

Track.

---

# 237. Debt Prioritization

Critical docs highest.

---

# 238. Release Quality Integration

Part 64.

---

# 239. Release Gate

Can include:

```text
Documentation
```

---

# 240. Documentation Gate

Fails if:

```text
missing migration guide
stale critical runbook
missing ADR
```

---

# 241. Deployment Integration

Part 62.

---

# 242. Deployment Bundle

Can include matching docs bundle.

---

# 243. Config Integration

Part 61.

---

# 244. Runbooks reference config fields by schema/version.

---

# 245. Performance Integration

Part 63.

---

# 246. Capacity runbooks use measured thresholds.

---

# 247. Observability Integration

Part 51.

---

# 248. Alert links to runbook.

---

# 249. Incident Alert

Should include:

```text
service
severity
runbook ID
```

---

# 250. No User IDs In Alert

Hard rule.

---

# 251. Governance Integration

Part 53.

---

# 252. Governance ceremonies need documented runbooks.

---

# 253. Legal Integration

Part 55.

---

# 254. Lawful request playbook is restricted.

---

# 255. Privacy Integration

Part 56.

---

# 256. Privacy incident playbooks required.

---

# 257. Recovery Integration

Part 57.

---

# 258. Account recovery runbooks separate from user instructions.

---

# 259. Federation Integration

Part 58.

---

# 260. Peering incident playbooks.

---

# 261. Naming Integration

Part 59.

---

# 262. Namespace incident guide.

---

# 263. Time Integration

Part 60.

---

# 264. Clock/epoch divergence runbook.

---

# 265. Testing Integration

Part 64.

---

# 266. Runbooks themselves must be tested.

---

# 267. Documentation Test Scenario

```text
operator receives alert
→ follows runbook
→ expected recovery achieved
```

---

# 268. Runbook Drill Harness

Possible.

---

# 269. Runbook Coverage Matrix

```rust
pub struct RunbookCoverageMatrix {
    pub services: BTreeMap<ServiceId, Vec<KnowledgeRef>>,
    pub incidents: BTreeMap<IncidentClass, Vec<KnowledgeRef>>,
}
```

---

# 270. Missing Coverage

CI warning/failure.

---

# 271. ADR Coverage

Major architectural module should link ADRs.

---

# 272. No Orphan ADR

Hard rule.

---

# 273. Supersession Graph

Acyclic.

---

# 274. Validate Graph

CI.

---

# 275. Knowledge Integrity

Document hash.

---

# 276. Knowledge Digest

```rust
pub struct KnowledgeDigest(pub [u8; 32]);
```

---

# 277. Build Docs Once

Promote exact bundle.

---

# 278. No Regenerate Docs After Release Without Version Bump

Hard rule.

---

# 279. Hotfix Docs

Can publish supplemental signed patch bundle.

---

# 280. Versioned.

---

# 281. Knowledge Localization

Possible.

---

# 282. Canonical Source

One language/source.

---

# 283. Translation

Must not alter technical meaning.

---

# 284. Security-Critical Warning

Can retain canonical English snippet internally if needed.

---

# 285. Operator Accessibility

Docs readable under stress.

---

# 286. Style

Use:

```text
short steps
clear preconditions
warnings
verification
```

---

# 287. Avoid Dense Narrative In Runbook

Hard rule.

---

# 288. Architecture Docs Can Be Deep

Different purpose.

---

# 289. Incident Playbook Should Front-Load

```text
containment
do-not-do
escalation
```

---

# 290. Knowledge Export

Can generate PDF/offline HTML.

---

# 291. But

Markdown remains source.

---

# 292. No Secrets In Generated Format

Hard rule.

---

# 293. Knowledge Search Tags

```text
service
incident
risk
version
owner
```

---

# 294. No User/Customer Tags

Hard rule.

---

# 295. Change Notifications

Operators notified when critical runbook changes.

---

# 296. Acknowledgement

Optional for Tier0.

---

# 297. No Notification Spam

Only material updates.

---

# 298. Knowledge Recovery

Back up docs repo + signed bundles.

---

# 299. Air-Gapped Site

Store offline verified copy.

---

# 300. Disaster Scenario

If Git/Internet unavailable:

```text
runbooks still accessible
```

---

# 301. No Single SaaS Dependency

Hard rule.

---

# 302. Knowledge Audit

Periodic.

---

# 303. Audit Checks

```text
freshness
coverage
ownership
accuracy
drill evidence
```

---

# 304. External Audit

Can inspect relevant docs.

---

# 305. Restricted Redaction

Generate sanitized package.

---

# 306. No Manual Ad-Hoc Redaction

Preferred.

---

# 307. Redaction Policy

Machine-readable.

---

# 308. Sensitive Field Types

```rust
pub enum KnowledgeRedactionClass {
    OperatorInternal,
    SecuritySensitive,
    LegalSensitive,
    PublicSafe,
}
```

---

# 309. Public Export

Only PublicSafe.

---

# 310. Automation Safety

Runbook automation must obey Part 61 permissions.

---

# 311. Documentation Never Grants Authority

Hard rule.

---

# 312. If Runbook Says "rotate key"

Actual control plane still validates role/quorum.

---

# 313. Knowledge Threat Model

Threats:

```text
stale docs
malicious edit
secret leakage
misleading automation
tampered offline copy
ownership loss
```

---

# 314. Malicious Edit

Review + signed bundle.

---

# 315. Stale Docs

Version/freshness gates.

---

# 316. Secret Leakage

Scanner + restricted schema.

---

# 317. Misleading Automation

Typed command + dry run + approvals.

---

# 318. Tampered Offline Copy

Signature verification.

---

# 319. Ownership Loss

Service catalog validation.

---

# 320. Testing

Need documentation/knowledge testkit.

---

# 321. Scenarios

```text
stale runbook
removed CLI command
missing owner
secret accidentally pasted
old release docs used on new binary
```

---

# 322. Stale Runbook Test

Warn/block.

---

# 323. CLI Drift Test

Docs command fails CI.

---

# 324. Secret Leak Test

CI fails.

---

# 325. Ownership Test

No Tier0 owner -> fail.

---

# 326. Version Mismatch Test

Admin UI warns.

---

# 327. Offline Bundle Test

Verify signature/readability.

---

# 328. Incident Drill Test

Runbook leads to expected recovery.

---

# 329. ADR Graph Test

No broken supersession links.

---

# 330. Link Test

No broken critical links.

---

# 331. Fuzzing

Fuzz:

```text
front matter parser
knowledge index
runbook metadata
```

---

# 332. Property Tests

Properties:

```text
critical service always maps to at least one current runbook
superseded ADR points to valid replacement
public export excludes restricted classes
runbook automation never bypasses control-plane authorization
```

---

# 333. Formal Verification

Limited need.

---

# 334. Strong Candidates

```text
knowledge coverage graph
approval policy mapping
```

---

# 335. Performance

Docs build/search should be fast.

---

# 336. Offline Search Index

Bounded.

---

# 337. Crate Layout

Recommended:

```text
crates/
├── siar-knowledge-core/
├── siar-adr/
├── siar-service-catalog/
├── siar-runbook/
├── siar-incident-playbook/
├── siar-knowledge-index/
├── siar-knowledge-lint/
├── siar-knowledge-bundle/
├── siar-knowledge-observability/
└── siar-knowledge-testkit/
```

---

# 338. `siar-knowledge-core`

Owns:

```text
knowledge IDs
classes
sensitivity
errors
```

---

# 339. `siar-adr`

ADR parsing/status/supersession.

---

# 340. `siar-service-catalog`

Service ownership/dependency mapping.

---

# 341. `siar-runbook`

Runbook metadata/steps/automation refs.

---

# 342. `siar-incident-playbook`

Incident response content/state.

---

# 343. `siar-knowledge-index`

Search/index/relations.

---

# 344. `siar-knowledge-lint`

Freshness/link/secret/command validation.

---

# 345. `siar-knowledge-bundle`

Signed offline docs bundle.

---

# 346. `siar-knowledge-observability`

Coverage/freshness metrics.

---

# 347. `siar-knowledge-testkit`

Runbook drill/doc validation harness.

---

# 348. Error Taxonomy

```rust
pub enum KnowledgeError {
    MissingOwner,
    StaleDocument,
    VersionMismatch,
    BrokenReference,
    InvalidMetadata,
    SecretDetected,
    CommandDrift,
    SignatureInvalid,
    CoverageMissing,
    Internal,
}
```

---

# 349. Security & Operational Invariants

Mandatory:

```text
1. Every Tier0/Tier1 production service has an owned, version-compatible runbook.
2. Critical incident classes have explicit playbooks with containment and prohibited-action sections.
3. Documentation contains no secrets, tokens, recovery material, or production credentials.
4. Runbook automation cannot bypass control-plane authorization or governance requirements.
5. Release notes, migration guides, and operator docs are bound to exact software versions/artifacts.
6. Critical runbooks are periodically drilled and failed drills require corrective action.
7. Incident/postmortem records contain no unnecessary user-content or social-graph data.
8. Offline operator knowledge is signed, verifiable, and available during control-plane/network outage.
9. ADR supersession and service ownership are explicit and machine-validated.
10. Critical architecture or operational changes cannot ship without associated documentation review.
11. Public documentation export cannot include restricted operational/security data.
12. Production knowledge has a feedback loop from incidents, tests, releases, and architecture changes.
```

---

# 350. Initial Production Scope

Implement first:

```text
Markdown-based docs repository
ADR template/status/supersession
service catalog
runbook metadata/front matter
incident playbook template
operator checklist
release notes + migration guide
ownership/escalation map
knowledge linter
secret scanning
runbook command validation
signed offline docs bundle
documentation release gate
incident→postmortem→test→runbook workflow
```

Then add:

```text
machine-readable runbook execution graph
contextual admin-console knowledge links
offline semantic search
local AI-assisted operator retrieval
automated knowledge coverage maps
formal runbook drill orchestration
```

---

# 351. Definition of Done

Part 65 is complete when:

- architecture docs, ADRs, runbooks, incident playbooks, troubleshooting, release notes, and migration guides are distinct knowledge types
- service ownership, criticality, SLO/runbook mapping, and escalation are machine-readable
- critical runbooks declare supported release ranges
- docs are signed/version-bound for offline use
- incident playbooks define containment, unsafe actions, recovery, communication, and evidence handling
- runbook automation uses typed control-plane actions
- secret scanning and public/internal/restricted sensitivity rules are enforced
- ADR supersession graph is validated
- incidents feed permanent tests and documentation updates
- runbook drills produce evidence
- release quality includes a documentation gate
- offline operations do not depend on a single SaaS/wiki
- documentation can be rebuilt, searched, audited, and verified reproducibly

---

# 352. Final Architecture

```text
                     ARCHITECTURE / ADRs
                             │
                             ▼
                      SERVICE CATALOG
                             │
              ┌──────────────┼──────────────┐
              │              │              │
          Runbooks       Playbooks      Releases
              │              │              │
              └──────────────┼──────────────┘
                             ▼
                    KNOWLEDGE REGISTRY
                             │
                             ▼
                OPERATOR CONSOLE / OFFLINE BUNDLE
                             │
                             ▼
                      INCIDENT / OPERATION
                             │
                             ▼
                   POSTMORTEM / FEEDBACK LOOP
```

Knowledge safety model:

```text
versioned docs
+
explicit ownership
+
tested runbooks
+
signed offline access
+
incident feedback
+
release-linked knowledge
```

not:

```text
tribal knowledge and stale wiki pages
```

---

# 353. Final Principle

Operational knowledge is part of the production system.

The correct model is:

```text
document decisions
+
own every service
+
test every critical runbook
+
bind knowledge to releases
+
learn from incidents
```

This architecture gives SIAR a durable production knowledge system so architecture, operations, recovery, incident response, and release procedures remain accurate, verifiable, and usable even under outage or compromise.
