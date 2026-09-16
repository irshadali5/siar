# Core System Architecture Part 97 — Anonymous Network Security Operations Center, Threat Intelligence, Indicators of Compromise, Detection Engineering, Hunting & Privacy-Preserving Defensive Intelligence Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 97  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 28, 42, 46, 51–53, 55–56, 61–73, 78–83, 92–96

**Primary purpose:** define SIAR's Security Operations Center (SOC), threat-intelligence, indicator-of-compromise (IoC), detection-engineering, infrastructure hunting, correlation, alerting, federation intelligence exchange, indicator signing, defensive knowledge lifecycle, analyst access, and privacy-preserving defensive intelligence architecture without creating centralized user surveillance, behavioral profiling, or broad content inspection.

---

# 1. Purpose

A mature security program needs more than incident response.

It needs continuous defensive capability to answer:

```text
What threats are currently relevant?
Which artifacts or credentials are known-bad?
Are known exploit patterns visible in infrastructure?
Are we seeing repeat compromise indicators?
Which detections are noisy or ineffective?
Are federated peers reporting relevant compromise evidence?
```

The governing principle is:

> **SIAR defensive intelligence should correlate infrastructure, cryptographic, artifact, credential, and control signals—not ordinary private user behavior.**

---

# 2. Architectural Position

```text
Threat Intelligence / Local Signals / Federation Advisories
                         │
                         ▼
                  Intelligence Intake
                         │
                         ▼
                    Normalization
                         │
           ┌─────────────┼─────────────┐
           │             │             │
         IoCs       Detection Rules   Threat Models
           │             │             │
           └─────────────┼─────────────┘
                         ▼
                  Detection Pipeline
                         │
                         ▼
                    Alert / Hunt
                         │
                         ▼
                  Incident Handoff
```

---

# 3. Core Separation

Keep distinct:

```text
threat intelligence
indicator
detection rule
alert
hunt
incident
forensic evidence
analytics
```

---

# 4. Non-Goals

Part 97 does not create:

```text
a global user behavior warehouse
private message inspection
user-interest profiling
cross-tenant surveillance
persistent packet capture
employee productivity monitoring
```

---

# 5. SOC Mission

SOC focuses on:

```text
defensive detection
triage
threat intelligence
detection tuning
hunting
incident support
```

---

# 6. SOC Must Not Become

```text
product analytics
content moderation
user behavior analysis
general-purpose monitoring
```

---

# 7. Hard Rule

SOC access is justified by security purpose, not convenience.

---

# 8. SOC Roles

```rust
pub enum SocRole {
    Tier1Analyst,
    Tier2Analyst,
    DetectionEngineer,
    ThreatIntelAnalyst,
    ThreatHunter,
    IncidentResponder,
    SocLead,
    PrivacyReviewer,
}
```

---

# 9. Tier1 Analyst

Alert triage.

---

# 10. Tier2 Analyst

Deeper investigation.

---

# 11. Detection Engineer

Detection logic/rules.

---

# 12. Threat Intel Analyst

External/internal defensive intelligence.

---

# 13. Threat Hunter

Hypothesis-driven infrastructure search.

---

# 14. Incident Responder

Part 96 handoff.

---

# 15. Soc Lead

Operational coordination.

---

# 16. Privacy Reviewer

Sensitive hunt/evidence boundaries.

---

# 17. No Universal SOC Superuser

Hard rule.

---

# 18. SOC Capability

```rust
pub struct SocCapability {
    pub role: SocRole,
    pub scope: SocScope,
    pub actions: BTreeSet<SocAction>,
    pub expires_at: Timestamp,
}
```

---

# 19. SOC Scope

```rust
pub enum SocScope {
    Environment(EnvironmentClass),
    Service(ServiceId),
    Tenant(TenantId),
    FederationPeer(FederationDomainId),
    Incident(IncidentId),
}
```

---

# 20. No Global User Scope

Hard rule.

---

# 21. SOC Actions

```rust
pub enum SocAction {
    ViewAlert,
    QuerySecurityTelemetry,
    ManageDetectionRule,
    ImportThreatIntel,
    PublishIndicator,
    ExecuteHunt,
    EscalateIncident,
    ExportScopedEvidence,
}
```

---

# 22. Capability Expiry

Required for sensitive actions.

---

# 23. No Permanent Elevated Hunting Credential

Hard rule.

---

# 24. Threat Intelligence Sources

```rust
pub enum ThreatIntelSource {
    InternalIncident,
    SecurityVendor,
    CommunityAdvisory,
    VulnerabilityFeed,
    RustSecurityAdvisory,
    OsVendor,
    FederationPeer,
    ManualAnalyst,
}
```

---

# 25. Internal Incident

Lessons/IoCs from Part 96.

---

# 26. Security Vendor

Commercial/open defensive feed.

---

# 27. Community Advisory

Public security communities.

---

# 28. Vulnerability Feed

CVE/advisory sources.

---

# 29. Rust Security Advisory

RustSec/dependency advisories.

---

# 30. OS Vendor

Linux/Android/etc.

---

# 31. Federation Peer

Signed peer advisory.

---

# 32. Manual Analyst

Reviewed entry.

---

# 33. Hard Rule

Threat intelligence is untrusted until normalized and validated.

---

# 34. Threat Intel Record

```rust
pub struct ThreatIntelRecord {
    pub intel_id: ThreatIntelId,
    pub source: ThreatIntelSource,
    pub confidence: IntelConfidence,
    pub classification: ThreatClassification,
    pub indicators: Vec<IndicatorRef>,
    pub expires_at: Option<Timestamp>,
}
```

---

# 35. Intel Confidence

```rust
pub enum IntelConfidence {
    Low,
    Medium,
    High,
    Confirmed,
}
```

---

# 36. Confidence ≠ Severity

Hard rule.

---

# 37. Threat Classification

```rust
pub enum ThreatClassification {
    Malware,
    CredentialTheft,
    SupplyChain,
    Exploit,
    HostCompromise,
    InfrastructureAbuse,
    FederationCompromise,
    PrivacyAttack,
}
```

---

# 38. IoC Model

Indicators must be typed.

---

# 39. Indicator Types

```rust
pub enum IndicatorType {
    ArtifactDigest,
    BinaryDigest,
    CertificateFingerprint,
    PublicKeyFingerprint,
    DomainName,
    IpNetwork,
    ServiceEndpoint,
    PackageVersion,
    VulnerabilityId,
    ExploitSignature,
    ProtocolFingerprint,
}
```

---

# 40. User IDs Are Not IoCs

Hard rule.

---

# 41. Message Content Is Not IoC

Hard rule.

---

# 42. Contact IDs Are Not IoCs

Hard rule.

---

# 43. Indicator ID

```rust
pub struct IndicatorId(pub [u8; 16]);
```

---

# 44. Indicator Record

```rust
pub struct Indicator {
    pub id: IndicatorId,
    pub indicator_type: IndicatorType,
    pub value_digest: Digest,
    pub confidence: IntelConfidence,
    pub valid_from: CoarseTimestamp,
    pub expires_at: Timestamp,
    pub scope: IndicatorScope,
}
```

---

# 45. Indicator Scope

```rust
pub enum IndicatorScope {
    LocalEnvironment,
    ServiceClass,
    TenantManaged,
    FederationPeer,
    GlobalDefensive,
}
```

---

# 46. GlobalDefensive

Only for universally relevant artifacts/infrastructure.

---

# 47. No Global User Indicator

Hard rule.

---

# 48. Indicator TTL

Required.

---

# 49. No Permanent IoC By Default

Hard rule.

---

# 50. Why

Threat context changes.

---

# 51. Indicator Value

Where possible store normalized value.

---

# 52. Sensitive indicator

store digest/reference.

---

# 53. Hard rule.

---

# 54. IoC Examples

Allowed:

```text
malicious artifact SHA-256
revoked cert fingerprint
known malicious domain
compromised release key fingerprint
known exploit byte/signature
```

---

# 55. Disallowed Baseline

```text
user search term
user contact identity
message sender pseudonym
private channel membership
```

---

# 56. Hard rule.

---

# 57. Indicator Source Provenance

Every indicator records source.

---

# 58. No Source-Free Indicator

Hard rule.

---

# 59. Indicator Status

```rust
pub enum IndicatorStatus {
    Candidate,
    Active,
    Revoked,
    Expired,
    Superseded,
}
```

---

# 60. Candidate

Not enforced.

---

# 61. Active

Used in detections/blocks.

---

# 62. Revoked

Source invalidated.

---

# 63. Expired

TTL ended.

---

# 64. Superseded

Replaced by newer knowledge.

---

# 65. Hard Rule

Expired/revoked indicators cannot remain silently active.

---

# 66. Indicator Signing

Defensive indicator bundles signed.

---

# 67. Indicator Bundle

```rust
pub struct IndicatorBundle {
    pub bundle_id: IndicatorBundleId,
    pub issued_at: CoarseTimestamp,
    pub expires_at: Timestamp,
    pub indicators: Vec<Indicator>,
    pub signer: ThreatIntelSignerRef,
    pub signature: SignatureBytes,
}
```

---

# 68. Signed Canonical Encoding

Postcard canonical bytes.

---

# 69. No unsigned enforcement bundle.

---

# 70. Hard rule.

---

# 71. Indicator Distribution

```text
SOC
→ signed bundle
→ service/agent
→ validate
→ activate
```

---

# 72. No Dynamic Arbitrary Rule Execution From Feed

Hard rule.

---

# 73. IoC Enforcement

Possible:

```text
block artifact
reject cert
quarantine endpoint
alert on package
block compromised federation peer
```

---

# 74. No User Account Ban From Infrastructure IoC Alone

Hard rule.

---

# 75. Detection Engineering

Detection rules convert signals into alerts.

---

# 76. Detection Rule

```rust
pub struct DetectionRule {
    pub rule_id: DetectionRuleId,
    pub class: DetectionRuleClass,
    pub severity: IncidentSeverity,
    pub inputs: Vec<SecuritySignalType>,
    pub condition: DetectionCondition,
    pub response: DetectionResponsePolicy,
}
```

---

# 77. Detection Rule Classes

```rust
pub enum DetectionRuleClass {
    Authentication,
    Credential,
    Host,
    Workload,
    Artifact,
    SupplyChain,
    NetworkInfrastructure,
    TenantIsolation,
    Federation,
    Privacy,
    AuditIntegrity,
}
```

---

# 78. No User Engagement Rule Class

Hard rule.

---

# 79. Detection Inputs

Allowed:

```text
auth failure classes
credential replay
artifact digest mismatch
host attestation
policy mismatch
network service anomaly
audit integrity failure
tenant boundary violation
```

---

# 80. Forbidden Baseline Inputs

```text
message content
search history
feed dwell time
contact graph
private social interactions
```

---

# 81. Hard rule.

---

# 82. Detection Condition

Bounded DSL.

---

# 83. Example operators:

```text
Equals
CountOverThreshold
RateOverWindow
Sequence
Any
All
Not
MatchesIndicator
```

---

# 84. No Turing-Complete Script

Hard rule.

---

# 85. Why

Deterministic review and safety.

---

# 86. Detection Rule Version

```rust
pub struct DetectionRuleVersion(pub u64);
```

---

# 87. Rules Signed

Hard rule.

---

# 88. Rule Lifecycle

```rust
pub enum DetectionRuleState {
    Draft,
    Testing,
    Active,
    Disabled,
    Deprecated,
}
```

---

# 89. Draft

Not executed.

---

# 90. Testing

Shadow mode.

---

# 91. Active

Produces alerts.

---

# 92. Disabled

Stops evaluation.

---

# 93. Deprecated

Scheduled removal.

---

# 94. No Direct Draft→Global Enforcement

Hard rule.

---

# 95. Detection Test Corpus

Synthetic.

---

# 96. Replay known security signals.

---

# 97. No production user content needed.

---

# 98. Hard rule.

---

# 99. Detection Rule Validation

Must test:

```text
precision
recall on known corpus
false positives
performance
privacy input boundaries
```

---

# 100. Hard Rule

A detection rule cannot be promoted without privacy classification.

---

# 101. Alert

```rust
pub struct SecurityAlert {
    pub alert_id: AlertId,
    pub rule: DetectionRuleId,
    pub severity: IncidentSeverity,
    pub scope: AlertScope,
    pub evidence_refs: Vec<EvidenceRef>,
    pub state: AlertState,
}
```

---

# 102. Alert Scope

```rust
pub enum AlertScope {
    Workload(WorkloadInstanceId),
    Service(ServiceId),
    Host(HostId),
    Tenant(TenantId),
    FederationPeer(FederationDomainId),
    Environment(EnvironmentClass),
}
```

---

# 103. No User Social Scope

Hard rule.

---

# 104. Alert State

```rust
pub enum AlertState {
    New,
    Investigating,
    Benign,
    TruePositive,
    Escalated,
    Closed,
}
```

---

# 105. Alert ≠ Incident

Hard rule.

---

# 106. Escalation

True/high-risk alert can create Part 96 incident.

---

# 107. No automatic incident from every signal.

---

# 108. Hard rule.

---

# 109. Alert Deduplication

Same underlying condition should aggregate.

---

# 110. Avoid alert storms.

---

# 111. Alert Fingerprint

Infrastructure-scoped.

---

# 112. No user identity.

---

# 113. Hard rule.

---

# 114. Detection Windows

Use coarse bounded windows.

---

# 115. No infinite history.

---

# 116. Hard rule.

---

# 117. Authentication Detection

Examples:

```text
refresh-token replay
credential use after revocation
impossible issuance state
```

---

# 118. Not:

```text
where user travels
what user browses
```

---

# 119. Hard rule.

---

# 120. Privacy Detection

Examples:

```text
private content in public index
telemetry enabled in max anonymity
direct-route fallback
unexpected identifier in logs
```

---

# 121. Privacy Failures Are SOC-Relevant

Hard rule.

---

# 122. Tenant Isolation Detection

Examples:

```text
tenant context mismatch
cross-tenant query attempt
wrong encryption key domain
```

---

# 123. No tenant content inspection needed.

---

# 124. Hard rule.

---

# 125. Supply-Chain Detection

Examples:

```text
unexpected artifact digest
unsigned build
SBOM mismatch
dependency advisory
```

---

# 126. No user data.

---

# 127. Hard rule.

---

# 128. Host Detection

Examples:

```text
attestation drift
unexpected executable
privilege escalation signal
credential theft
```

---

# 129. Host telemetry limited.

---

# 130. No broad employee surveillance.

---

# 131. Hard rule.

---

# 132. Network Infrastructure Detection

Examples:

```text
relay connection flood
unexpected service endpoint
TLS/mTLS failure spike
routing policy violation
```

---

# 133. Avoid user communication graphing.

---

# 134. Aggregate by service/endpoint class.

---

# 135. Hard rule.

---

# 136. Traffic Analysis Risk

Network security detection can itself deanonymize.

---

# 137. Therefore:

```text
no global cross-service flow correlation
no per-user route history
no source-destination graph
```

---

# 138. Hard rule.

---

# 139. Detection Engineering Privacy Review

Every rule declares:

```text
input classes
retention
scope
purpose
output
```

---

# 140. No Hidden Data Source

Hard rule.

---

# 141. Threat Hunting

Hypothesis-driven, not fishing expedition.

---

# 142. Hunt Definition

```rust
pub struct ThreatHunt {
    pub hunt_id: HuntId,
    pub hypothesis: HuntHypothesis,
    pub scope: HuntScope,
    pub data_sources: BTreeSet<HuntDataSource>,
    pub expires_at: Timestamp,
    pub approvals: ApprovalBundle,
}
```

---

# 143. Hunt Hypothesis Required

Hard rule.

---

# 144. Example

Good:

```text
"Compromised release key may have signed artifact X."
```

Bad:

```text
"Search all activity for anything suspicious."
```

---

# 145. Hard rule.

---

# 146. Hunt Scope

```rust
pub enum HuntScope {
    Service(ServiceId),
    HostClass(HostClass),
    Environment(EnvironmentClass),
    TenantManaged(TenantId),
    FederationPeer(FederationDomainId),
    Incident(IncidentId),
}
```

---

# 147. No Global User Hunt

Hard rule.

---

# 148. Hunt Data Sources

```rust
pub enum HuntDataSource {
    AuditMetadata,
    SecurityTelemetry,
    HostAttestation,
    ArtifactInventory,
    CredentialMetadata,
    NetworkInfrastructureMetadata,
    ComplianceState,
}
```

---

# 149. User Content Not Baseline

Hard rule.

---

# 150. Sensitive Hunt Expansion

Requires privacy reviewer + incident context.

---

# 151. No general hunting through private messages.

---

# 152. Hard rule.

---

# 153. Hunt Lifetime

Bounded.

---

# 154. Auto-expire.

---

# 155. No permanent query/watch.

---

# 156. Hard rule.

---

# 157. Hunt Query Language

Structured/typed.

---

# 158. No arbitrary SQL across all stores.

---

# 159. Hard rule.

---

# 160. Hunt Result

```rust
pub struct HuntResult {
    pub hunt: HuntId,
    pub matches: Vec<HuntMatch>,
    pub evaluated_scope: HuntScope,
    pub completed_at: CoarseTimestamp,
}
```

---

# 161. Matches Minimal

Evidence refs, not raw data.

---

# 162. Hard rule.

---

# 163. Hunt Match

```rust
pub struct HuntMatch {
    pub signal: SecuritySignalRef,
    pub confidence: DetectionConfidence,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 164. No User Behavioral Timeline

Hard rule.

---

# 165. Threat Intelligence Normalization

Different feeds need one internal model.

---

# 166. Normalize:

```text
indicator type
value
confidence
source
scope
TTL
```

---

# 167. Do Not Normalize Into Generic JSON Blob

Hard rule.

---

# 168. Threat Intel Dedup

Same indicator from multiple sources.

---

# 169. Preserve provenance list.

---

# 170. Confidence can rise.

---

# 171. No source erased.

---

# 172. Hard rule.

---

# 173. Intel Conflicts

One source says malicious, another benign.

---

# 174. Record conflict.

---

# 175. Do not silently choose.

---

# 176. Hard rule.

---

# 177. Indicator Confidence Aggregation

Policy-defined.

---

# 178. High-confidence enforcement may require multiple trusted sources.

---

# 179. Critical internal evidence can override.

---

# 180. Explicit.

---

# 181. No black-box scoring.

---

# 182. Hard rule.

---

# 183. IOC False Positives

Need revocation path.

---

# 184. Revoked indicator distributed quickly.

---

# 185. Clients/services reject old bundle by version/expiry.

---

# 186. Hard rule.

---

# 187. Indicator Bundle Version

Monotonic.

---

# 188. Anti-rollback.

---

# 189. Old malicious block set cannot restore.

---

# 190. Hard rule.

---

# 191. Indicator Transparency

Internal high-value indicators can have approval history.

---

# 192. Public indicators can cite advisory.

---

# 193. No secret user basis.

---

# 194. Hard rule.

---

# 195. Threat Intel Sharing

Federation peer exchange.

---

# 196. Share only relevant defensive indicators.

---

# 197. No user data.

---

# 198. Hard rule.

---

# 199. Federation Threat Intel Statement

```rust
pub struct FederationThreatIntelStatement {
    pub source_domain: FederationDomainId,
    pub indicators: Vec<Indicator>,
    pub issued_at: CoarseTimestamp,
    pub expires_at: Timestamp,
    pub signature: SignatureBytes,
}
```

---

# 200. Peer Trust

Statement validated against federation trust.

---

# 201. No transitive blind trust.

---

# 202. Hard rule.

---

# 203. Federation Confidence

Local policy may downgrade remote confidence.

---

# 204. Good.

---

# 205. No remote peer controls local enforcement directly.

---

# 206. Hard rule.

---

# 207. Threat Intel Privacy

Indicator sharing can leak internal architecture.

---

# 208. Share minimum.

---

# 209. Example

Share artifact hash, not internal path/host inventory.

---

# 210. Hard rule.

---

# 211. Detection Response Policy

```rust
pub enum DetectionResponsePolicy {
    AlertOnly,
    QuarantineCandidate,
    BlockCandidate,
    IncidentCandidate,
}
```

---

# 212. Candidate Means

Requires response policy/approval.

---

# 213. No detection rule directly performs arbitrary destructive action.

---

# 214. Hard rule.

---

# 215. Automated Enforcement

Allowed for narrow high-confidence controls.

---

# 216. Examples:

```text
reject revoked cert
reject known revoked artifact
```

---

# 217. Not:

```text
delete tenant
ban user globally
```

---

# 218. Hard rule.

---

# 219. Detection-to-Incident Handoff

```text
alert
→ analyst validation
→ scope/severity
→ create incident
```

---

# 220. High-confidence critical control failure may auto-create incident.

---

# 221. Still no broad investigative authority.

---

# 222. Hard rule.

---

# 223. Alert Evidence

References:

```text
audit event
control failure
attestation result
artifact digest
credential metadata
```

---

# 224. No private content.

---

# 225. Hard rule.

---

# 226. Alert Triage Metadata

Store:

```text
rule
scope
severity
disposition
evidence refs
```

---

# 227. Avoid free-form notes with sensitive data.

---

# 228. Hard rule.

---

# 229. Analyst Notes

If needed:

```text
bounded
encrypted
retention-controlled
```

---

# 230. No copying user content into note.

---

# 231. Hard rule.

---

# 232. SOC Queue

Prioritized by:

```text
severity
confidence
scope
control criticality
```

---

# 233. Not by user importance.

---

# 234. Hard rule.

---

# 235. Alert Priorities

Critical security/privacy events reserve queue capacity.

---

# 236. Low-confidence intel cannot starve.

---

# 237. Hard rule.

---

# 238. SOC Escalation

Runbook-driven.

---

# 239. No manual ad hoc privilege expansion.

---

# 240. Hard rule.

---

# 241. Detection Rule Repository

Versioned Git/RON.

---

# 242. Code review.

---

# 243. Tests.

---

# 244. Signed release bundle.

---

# 245. Hard rule.

---

# 246. Rule Change Audit

High-level.

---

# 247. Record:

```text
rule enabled
rule disabled
severity changed
data source changed
```

---

# 248. Not every evaluation.

---

# 249. Hard rule.

---

# 250. Privacy Guardrail Rule

Detection build should fail if rule references forbidden data source.

---

# 251. Type system can enforce.

---

# 252. Good.

---

# 253. Security Signal Type

```rust
pub enum SecuritySignalType {
    CredentialReplay,
    RevokedCredentialUse,
    ArtifactMismatch,
    AttestationFailure,
    AuditChainFailure,
    TenantBoundaryViolation,
    PrivacyPolicyViolation,
    FederationTrustViolation,
    ServiceAnomaly,
}
```

---

# 254. No `UserBehavior` Generic Signal

Hard rule.

---

# 255. Detection DSL

Typed over `SecuritySignalType`.

---

# 256. Compile-time privacy boundary possible.

---

# 257. Hard rule.

---

# 258. Local Endpoint Detection

Desktop/mobile.

---

# 259. Scope:

```text
app integrity
secure storage
local DB corruption
unexpected local key change
```

---

# 260. No scanning other personal apps/files.

---

# 261. Hard rule.

---

# 262. Managed Endpoint Detection

Organization-managed profile only.

---

# 263. No personal-profile inspection.

---

# 264. Hard rule.

---

# 265. Server SOC

Infrastructure signals.

---

# 266. No blanket user content visibility.

---

# 267. Hard rule.

---

# 268. Data Retention

Alert/Intel/Rule/Hunt different.

---

# 269. Threat intel

retain until expiry + history metadata.

---

# 270. Alerts

bounded.

---

# 271. Hunt results

short.

---

# 272. Rules

version history.

---

# 273. No infinite raw signal history.

---

# 274. Hard rule.

---

# 275. Retention Classes

```rust
pub enum SocRetentionClass {
    ShortAlert,
    StandardAlert,
    LongThreatIntel,
    RuleHistory,
    IncidentScoped,
}
```

---

# 276. Raw Signal TTL

Short.

---

# 277. Aggregate alert metadata longer.

---

# 278. Hard rule.

---

# 279. SOC Storage

Separate:

```text
threat intel store
indicator store
rule store
alert store
hunt store
```

---

# 280. No universal security lake.

---

# 281. Hard rule.

---

# 282. Why

Security lakes tend to accumulate unrestricted data.

---

# 283. SOC Search

Structured.

---

# 284. Example filters:

```text
indicator
rule
service
host class
tenant managed scope
coarse time
```

---

# 285. No arbitrary user search.

---

# 286. Hard rule.

---

# 287. Cross-Store Correlation

Allowed only through approved infrastructure keys:

```text
artifact digest
certificate fingerprint
service ID
host attestation ID
incident ID
```

---

# 288. Not user identity.

---

# 289. Hard rule.

---

# 290. Correlation Engine

```rust
pub trait DefensiveCorrelationEngine {
    fn correlate(
        &self,
        signals: &[SecuritySignalRef],
        scope: SocScope,
    ) -> Result<Vec<CorrelationFinding>, SocError>;
}
```

---

# 291. Correlation Rules Typed.

---

# 292. No graph-building over social identities.

---

# 293. Hard rule.

---

# 294. Correlation Finding

```rust
pub struct CorrelationFinding {
    pub finding_id: FindingId,
    pub scope: SocScope,
    pub indicators: Vec<IndicatorRef>,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 295. Privacy-Safe Correlation

Prefer infrastructure provenance.

---

# 296. Hard rule.

---

# 297. Threat Hunting Approval

Low-risk infra hunt

analyst capability enough.

---

# 298. Sensitive expansion

privacy reviewer + incident.

---

# 299. Hard rule.

---

# 300. Hunting Time Limit

Hours/days.

---

# 301. Never ongoing forever.

---

# 302. Hard rule.

---

# 303. Hunt Stop Conditions

```text
hypothesis confirmed
hypothesis rejected
scope exhausted
expiry reached
```

---

# 304. No endless exploratory search.

---

# 305. Hard rule.

---

# 306. Hunting Output

Can generate:

```text
new detection rule
new IoC
incident
control improvement
```

---

# 307. No new user profile.

---

# 308. Hard rule.

---

# 309. Threat Modeling Feedback

Threat intelligence updates threat model.

---

# 310. Example:

new correlation attack on anonymity.

---

# 311. Feed Part 42 threat model.

---

# 312. No automatic policy downgrade.

---

# 313. Hard rule.

---

# 314. Privacy Threat Intelligence

Important class.

---

# 315. Examples:

```text
new traffic-analysis technique
browser fingerprint exploit
metadata side channel
new relay-correlation attack
```

---

# 316. Defensive response:

```text
route policy
cover traffic tuning
client patch
```

---

# 317. No invasive monitoring.

---

# 318. Hard rule.

---

# 319. Vulnerability Intelligence

Map advisories to SBOM/artifacts.

---

# 320. Artifact Inventory

Part 73.

---

# 321. Result:

```text
affected release
affected service
severity
fixed version
```

---

# 322. No user data.

---

# 323. Hard rule.

---

# 324. Exploitability Context

Can consider:

```text
reachable service
enabled feature
artifact version
```

---

# 325. Not user behavior.

---

# 326. Hard rule.

---

# 327. Threat Intel Expiry

IoC may expire.

---

# 328. Vulnerability may remain until patched.

---

# 329. Explicit semantics.

---

# 330. Hard rule.

---

# 331. Indicator Revocation

If false positive.

---

# 332. New signed bundle.

---

# 333. Immediate disable.

---

# 334. Hard rule.

---

# 335. Detection Rule Rollout

Phases:

```text
Test
Shadow
Alerting
Enforcing
```

---

# 336. Enforcing only for safe, high-confidence rules.

---

# 337. Hard rule.

---

# 338. Shadow Mode

Collect aggregate false-positive data.

---

# 339. No new raw surveillance stream.

---

# 340. Hard rule.

---

# 341. Rule Quality Metrics

Allowed:

```text
alerts generated
true-positive ratio
false-positive ratio
evaluation latency
```

---

# 342. No user behavior.

---

# 343. Hard rule.

---

# 344. Detection Drift

Rules can degrade.

---

# 345. Monitor:

```text
precision
signal availability
schema compatibility
```

---

# 346. No automatic broader data collection to fix.

---

# 347. Hard rule.

---

# 348. Detection Coverage

Map rule to threat/control.

---

# 349. Coverage Matrix

```rust
pub struct DetectionCoverage {
    pub threat: ThreatModelRef,
    pub controls: Vec<ControlId>,
    pub rules: Vec<DetectionRuleId>,
}
```

---

# 350. No single numeric "security coverage score" required.

---

# 351. Critical gaps explicit.

---

# 352. Hard rule.

---

# 353. Threat Intel Confidence Decay

Old intelligence becomes less reliable.

---

# 354. Optional decay policy.

---

# 355. But do not keep infinite old blocks.

---

# 356. Hard rule.

---

# 357. IOC Matching

Exact where possible.

---

# 358. Fuzzy signature only when reviewed.

---

# 359. Avoid probabilistic user attribution.

---

# 360. Hard rule.

---

# 361. Malware/File Scanning

Public/managed file systems may scan where policy permits.

---

# 362. Private E2EE content cannot be centrally scanned.

---

# 363. Hard rule.

---

# 364. Local Client Safety Scan

Optional local-only.

---

# 365. Signatures updated.

---

# 366. No upload of private file content.

---

# 367. Hard rule.

---

# 368. Threat Intel Update Distribution

Low priority but timely for critical.

---

# 369. Signed.

---

# 370. Batched.

---

# 371. No device wake solely for low-severity feed.

---

# 372. Hard rule.

---

# 373. Critical Revocation Feed

Can wake.

---

# 374. Example revoked release key.

---

# 375. No sensitive content.

---

# 376. Hard rule.

---

# 377. SOC Notifications

Operators receive alerts.

---

# 378. User notification only when user impact/action required.

---

# 379. No exposing internal incident details.

---

# 380. Hard rule.

---

# 381. Security Advisory Integration

Public advisories signed.

---

# 382. Related indicators can be public.

---

# 383. Internal sensitive IoCs remain scoped.

---

# 384. Hard rule.

---

# 385. IOC Sharing Policy

```rust
pub enum IndicatorSharingPolicy {
    LocalOnly,
    TenantScoped,
    FederationScoped,
    Public,
}
```

---

# 386. Default Internal

LocalOnly.

---

# 387. No automatic external sharing.

---

# 388. Hard rule.

---

# 389. Threat Intel Classification

```rust
pub enum IntelDisclosureClass {
    Internal,
    Restricted,
    FederationShareable,
    Public,
}
```

---

# 390. Sharing enforces class.

---

# 391. No analyst freeform bypass.

---

# 392. Hard rule.

---

# 393. Tenant SOC

Optional enterprise scope.

---

# 394. Tenant sees only managed environment.

---

# 395. No personal user context.

---

# 396. Hard rule.

---

# 397. Tenant IoCs

Organization artifacts/certs/endpoints.

---

# 398. No employee personal identifiers.

---

# 399. Hard rule.

---

# 400. Federation SOC Exchange

Signed intelligence.

---

# 401. No remote SOC access to local telemetry.

---

# 402. Hard rule.

---

# 403. Peer Request For More Evidence

Requires explicit incident/cooperation scope.

---

# 404. No automatic sharing.

---

# 405. Hard rule.

---

# 406. Threat Intel Source Trust

Per-source policy.

---

# 407. Example:

```text
InternalIncident = High
TrustedVendor = Medium/High
UnknownPeer = Low
```

---

# 408. No blind aggregation.

---

# 409. Hard rule.

---

# 410. Indicator Enforcement Threshold

```rust
pub struct IndicatorEnforcementPolicy {
    pub min_confidence: IntelConfidence,
    pub required_sources: u8,
    pub action: EnforcementAction,
}
```

---

# 411. Enforcement Action

```rust
pub enum EnforcementAction {
    Alert,
    Reject,
    Quarantine,
}
```

---

# 412. No Delete.

---

# 413. Hard rule.

---

# 414. Threat Intelligence API

```rust
pub trait ThreatIntelService {
    fn ingest(
        &self,
        record: ThreatIntelRecord,
    ) -> Result<ThreatIntelId, SocError>;

    fn active_indicators(
        &self,
        scope: IndicatorScope,
    ) -> Result<Vec<Indicator>, SocError>;
}
```

---

# 415. Detection Engine

```rust
pub trait DetectionEngine {
    fn evaluate(
        &self,
        signal: SecuritySignal,
    ) -> Result<Vec<SecurityAlert>, SocError>;
}
```

---

# 416. Hunt Service

```rust
pub trait ThreatHuntService {
    fn execute(
        &self,
        capability: SocCapability,
        hunt: ThreatHunt,
    ) -> Result<HuntResult, SocError>;
}
```

---

# 417. Alert Service

```rust
pub trait AlertService {
    fn triage(
        &self,
        alert: AlertId,
        disposition: AlertDisposition,
    ) -> Result<(), SocError>;
}
```

---

# 418. Indicator Distributor

```rust
pub trait IndicatorDistributor {
    fn publish(
        &self,
        bundle: IndicatorBundle,
    ) -> Result<(), SocError>;
}
```

---

# 419. No Hidden General Query API

Hard rule.

---

# 420. SOC Error Taxonomy

```rust
pub enum SocError {
    Unauthorized,
    ScopeViolation,
    InvalidIndicator,
    IndicatorExpired,
    InvalidSignature,
    DetectionRuleInvalid,
    HuntApprovalRequired,
    HuntExpired,
    ForbiddenDataSource,
    AlertNotFound,
    FederationTrustFailure,
    Internal,
}
```

---

# 421. Audit Integration

Part 94.

---

# 422. Audit:

```text
detection rule activation
indicator enforcement bundle
sensitive hunt approval
external intel export
```

---

# 423. Not every alert evaluation.

---

# 424. Hard rule.

---

# 425. Compliance Integration

Part 95.

---

# 426. Controls verify:

```text
rule review
indicator expiry
SOC least privilege
forbidden data source absence
```

---

# 427. Hard rule.

---

# 428. Incident Integration

Part 96.

---

# 429. Escalation creates scoped incident.

---

# 430. Incident authority separate.

---

# 431. Hard rule.

---

# 432. Analytics Integration

Part 92.

---

# 433. SOC quality metrics aggregate only.

---

# 434. No use of product analytics raw data.

---

# 435. Hard rule.

---

# 436. Experiment Integration

Detection rules not A/B tested on weaker security/privacy.

---

# 437. Shadow rule validation allowed.

---

# 438. Hard rule.

---

# 439. Authorization Integration

Part 81.

---

# 440. Analyst capabilities scoped.

---

# 441. No SOC bypass of policy engine.

---

# 442. Hard rule.

---

# 443. Authentication Integration

Strong auth for analyst/operator actions.

---

# 444. Step-up for hunt/export/enforcement.

---

# 445. Hard rule.

---

# 446. Secrets Integration

Feed/vendor/API credentials brokered.

---

# 447. No static feed secret in analyst workstation.

---

# 448. Hard rule.

---

# 449. Host Integrity Integration

Part 71.

---

# 450. Attestation signals.

---

# 451. Quarantine handoff.

---

# 452. Hard rule.

---

# 453. Supply-Chain Integration

Part 73.

---

# 454. Artifact indicators mapped to release inventory.

---

# 455. No manual inventory guess.

---

# 456. Hard rule.

---

# 457. Event Bus Integration

Only typed security signals/alerts.

---

# 458. No raw evidence bodies.

---

# 459. Hard rule.

---

# 460. Database Integration

Stores:

```text
intel records
IoCs
detection rules
alerts
hunts
quality metrics
```

---

# 461. Separate from product DB.

---

# 462. Hard rule.

---

# 463. Retention Anti-Resurrection

Expired IoCs/hunts/alerts not restored active from backup.

---

# 464. Hard rule.

---

# 465. Backup

Rule/intel history can be backed up.

---

# 466. Active status/TTL preserved.

---

# 467. No expired block resurrection.

---

# 468. Hard rule.

---

# 469. Observability

Safe metrics:

```text
alerts by rule/severity
false-positive rate
intel ingestion failures
hunt completion
bundle distribution lag
```

---

# 470. Forbidden:

```text
user-linked alert metrics
user behavior graphs
```

---

# 471. Hard rule.

---

# 472. SOC SLOs

Examples:

```text
critical alert triage latency
indicator revocation propagation
critical rule activation latency
incident handoff latency
```

---

# 473. Privacy SLO

```text
0 forbidden private data sources in detection rules
0 cross-tenant hunt access
0 permanent hunt capability after expiry
```

---

# 474. Security SLO

```text
0 expired revoked indicators enforced
0 unsigned rule/intel bundle accepted
```

---

# 475. Failure Modes

```text
intel feed poisoning
rule false positive storm
indicator false positive
SOC store outage
federation intel abuse
```

---

# 476. Feed Poisoning

Source trust/provenance.

---

# 477. No direct enforcement from untrusted source.

---

# 478. Hard rule.

---

# 479. Rule False Positive Storm

Auto-disable/rollback if threshold exceeded.

---

# 480. Review.

---

# 481. No broad data expansion.

---

# 482. Hard rule.

---

# 483. Indicator False Positive

Revoke bundle.

---

# 484. Restore affected service carefully.

---

# 485. Hard rule.

---

# 486. SOC Store Outage

Critical static revocations remain locally cached within validity.

---

# 487. No permissive fallback to revoked artifact/cert.

---

# 488. Hard rule.

---

# 489. Federation Intel Abuse

Rate limit/trust downgrade/block peer intel channel.

---

# 490. No remote enforcement control.

---

# 491. Hard rule.

---

# 492. Testing

Need SOC/threat-intelligence testkit.

---

# 493. Test Scenarios

```text
malicious artifact IoC
revoked certificate
rule false-positive storm
hunt expiry
federation intel
```

---

# 494. IoC Expiry Test

Expired indicator no longer enforced.

---

# 495. IoC Revocation Test

Revoked indicator propagates.

---

# 496. Signature Test

Unsigned bundle rejected.

---

# 497. Poisoned Feed Test

Low-trust source cannot trigger high-risk enforcement alone.

---

# 498. Detection Privacy Test

Rule referencing private content fails validation.

---

# 499. Rule Rollout Test

Draft/testing rule cannot enforce.

---

# 500. Alert Dedup Test

Same condition does not flood queue.

---

# 501. Hunt Scope Test

Hunt cannot query outside authorized scope.

---

# 502. Hunt Expiry Test

Expired hunt stops.

---

# 503. Sensitive Hunt Test

Requires privacy approval/incident context.

---

# 504. Tenant Test

Tenant A SOC cannot see B.

---

# 505. Federation Test

Remote intel cannot directly control local enforcement.

---

# 506. Backup Test

Expired IoC not resurrected.

---

# 507. Correlation Test

Infrastructure IDs allowed, social IDs rejected.

---

# 508. Incident Handoff Test

Escalated alert creates correctly scoped incident.

---

# 509. Fuzzing

Fuzz:

```text
intel record
indicator bundle
detection rule
hunt query
federation intel statement
```

---

# 510. Property Tests

Properties:

```text
expired indicator can never be Active
forbidden data source can never be referenced by active detection rule
hunt result can never contain data outside hunt scope
remote federation intelligence can never directly grant local enforcement authority
```

---

# 511. Formal Verification Targets

Strong candidates:

```text
indicator lifecycle
rule rollout
hunt scope/expiry
detection-to-incident handoff
```

---

# 512. Kani Candidate

indicator status/scope enforcement.

---

# 513. TLA+ Candidate

intel ingest → activate → detect → alert → escalate → revoke.

---

# 514. Loom Candidate

concurrent indicator revoke + detection match + enforcement cache update.

---

# 515. Performance

Detection hot path must be bounded.

---

# 516. IoC exact matching fast.

---

# 517. Detection DSL precompiled.

---

# 518. Hunt queries offline/background.

---

# 519. No hunt on request path.

---

# 520. Hard rule.

---

# 521. Alert Queue

Bounded priority.

---

# 522. Critical reserve.

---

# 523. Low-confidence alert collapse.

---

# 524. No unlimited queue.

---

# 525. Hard rule.

---

# 526. Indicator Cache

Content-addressed/versioned.

---

# 527. Local validation before activate.

---

# 528. Expiry timer.

---

# 529. No stale infinite cache.

---

# 530. Hard rule.

---

# 531. Rule Evaluation Limits

CPU/time budget.

---

# 532. No catastrophic regex.

---

# 533. Prefer safe matcher engines.

---

# 534. Hard rule.

---

# 535. Threat Intel Storage

Normalize to typed records.

---

# 536. Avoid arbitrary feed blobs after processing.

---

# 537. Original source ref/digest enough where possible.

---

# 538. Hard rule.

---

# 539. Crate Layout

Recommended:

```text
crates/
├── siar-soc-core/
├── siar-threat-intel/
├── siar-indicator/
├── siar-detection-rule/
├── siar-detection-engine/
├── siar-security-alert/
├── siar-threat-hunt/
├── siar-defensive-correlation/
├── siar-federation-threat-intel/
├── siar-soc-observability/
└── siar-soc-testkit/
```

---

# 540. `siar-soc-core`

Owns:

```text
SOC roles
scopes
capabilities
errors
```

---

# 541. `siar-threat-intel`

Normalization/source trust/intel lifecycle.

---

# 542. `siar-indicator`

Typed IoCs/TTL/bundles/enforcement.

---

# 543. `siar-detection-rule`

Versioned typed detection DSL.

---

# 544. `siar-detection-engine`

Signal evaluation/alert generation.

---

# 545. `siar-security-alert`

Alert queue/dedup/disposition/escalation.

---

# 546. `siar-threat-hunt`

Scoped hypothesis-driven hunts.

---

# 547. `siar-defensive-correlation`

Infrastructure-only correlation.

---

# 548. `siar-federation-threat-intel`

Signed peer intelligence exchange.

---

# 549. `siar-soc-observability`

SOC pipeline health/quality only.

---

# 550. `siar-soc-testkit`

intel/rule/hunt/privacy/federation tests.

---

# 551. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. SOC detections and hunts operate on infrastructure, credential, cryptographic, artifact, audit, compliance, and service-security signals—not ordinary private user behavior.
2. Threat-intelligence indicators are typed, provenance-bound, scoped, expiring, and cannot use user identities, message content, search history, contact graphs, or private social membership as baseline IoCs.
3. Detection rules are signed, versioned, privacy-reviewed, tested, and expressed in a bounded deterministic DSL; arbitrary remote scripts are forbidden.
4. Detection rules cannot reference forbidden data sources, and this restriction should be enforced structurally by types/validation rather than convention alone.
5. Alerts are scope-bound security findings, not incidents; incident authority is granted only through the separate Part 96 incident process.
6. Threat hunts require explicit hypotheses, scopes, approved data sources, and expiry; open-ended "search everything" hunting is prohibited.
7. Sensitive hunt expansion requires incident context and privacy review, and no active hunt may survive its expiry.
8. Federation threat intelligence is signed and locally re-evaluated; a remote peer can never directly control local enforcement or gain access to local security telemetry.
9. Indicator and rule lifecycle state is anti-rollback; expired, revoked, superseded, or unsigned defensive artifacts cannot silently return to active enforcement.
10. SOC storage, correlation, metrics, and dashboards never become a universal security data lake or cross-tenant/user surveillance graph.
11. Security automation may reject revoked artifacts/credentials or quarantine infrastructure under signed policy, but cannot perform arbitrary destructive user/tenant actions.
12. SOC operations preserve the anonymity, local-first, least-authority, tenant-isolation, and anti-surveillance guarantees of the wider SIAR architecture.
```

---

# 552. Initial Production Scope

Implement first:

```text
typed SOC roles/scopes/capabilities
typed threat-intelligence sources
typed IoC model
indicator TTL/status/provenance
signed indicator bundles
local indicator cache
typed detection rule DSL
rule Draft/Test/Active lifecycle
alert queue/dedup/triage
privacy/security detection rules
supply-chain/artifact/cert indicators
hypothesis-driven scoped hunt model
federation threat-intel statements
incident escalation integration
audit/compliance integration
privacy-safe SOC metrics
SOC/threat-intelligence testkit
```

Then add:

```text
advanced infrastructure correlation
local endpoint defensive scanning
privacy-preserving federation intel aggregation
multi-source confidence models
formal IoC/rule/hunt verification
independent intelligence-feed trust policies
```

---

# 553. Definition of Done

Part 97 is complete when:

- SOC roles and authority are explicit/scoped
- threat intelligence has provenance/confidence/TTL
- indicators are typed and never user-behavior identifiers
- indicator bundles are signed and anti-rollback
- detection rules use a bounded typed DSL
- forbidden private data sources are structurally rejected
- rules have test/shadow/active lifecycle
- alerts are deduplicated and distinct from incidents
- hunts are hypothesis-driven, scoped, approved, and expiring
- federation intelligence cannot directly control local enforcement
- IoCs and rules cannot resurrect after expiry/revocation
- SOC correlation uses infrastructure/security identities only
- audit/compliance/incident handoffs are clean
- intel/rule/hunt/federation/fuzz/formal tests are specified

---

# 554. Final Architecture

```text
             THREAT INTEL / SECURITY SIGNALS
                         │
                         ▼
                  NORMALIZATION
                         │
            ┌────────────┼────────────┐
            │            │            │
          IoCs       RULES        THREAT MODEL
            │            │            │
            └────────────┼────────────┘
                         ▼
                  DETECTION ENGINE
                         │
                         ▼
                     ALERTS
                         │
              ┌──────────┼──────────┐
              │          │          │
           DISMISS      HUNT      INCIDENT
                         │
                         ▼
                 DEFENSIVE LEARNING
```

SOC safety model:

```text
typed defensive intelligence
+
scoped indicators
+
signed rule bundles
+
infrastructure-only correlation
+
hypothesis-driven hunts
+
privacy review
+
strict TTL/expiry
+
incident handoff
```

not:

```text
collect every user action in a giant security lake and search it forever whenever something looks suspicious
```

---

# 555. Final Principle

A SOC should know enough about infrastructure, credentials, artifacts, controls, and attack indicators to defend the network—without needing to know the private lives of the people using it.

The correct model is:

```text
ingest defensively
+
normalize strictly
+
scope indicators
+
detect from security signals
+
hunt by hypothesis
+
expire aggressively
+
federate cautiously
+
escalate through incident controls
+
never build surveillance infrastructure
```

This architecture gives SIAR a privacy-preserving foundation for security operations, threat intelligence, IoCs, detection engineering, infrastructure hunting, federation defensive intelligence, alerting, and incident escalation while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–96.
