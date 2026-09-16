# Core System Architecture Part 55 — Anonymous Network Legal/Compliance Boundary, Jurisdiction Isolation, Data-Minimization & Lawful Operations Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 55  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–54  

**Primary purpose:** define how SIAR's anonymous infrastructure separates privacy-preserving protocol operation from jurisdiction-specific legal/compliance obligations, including operator legal boundaries, data minimization, retention, lawful requests, jurisdiction isolation, cross-border controls, transparency, operator suspension, shutdown behavior, and compliance-safe Rust control-plane interfaces.

> This document is a technical architecture specification, not jurisdiction-specific legal advice. Operators still need qualified counsel for the laws that apply to them.

---

# 1. Purpose

Anonymous infrastructure exists in real legal environments.

Operators may face obligations concerning:

```text
business registration
payments
tax
data protection
consumer law
content-related processes
lawful requests
sanctions
records retention
cybersecurity
cross-border transfer
```

A poor architecture can make compliance incompatible with privacy.

Example failure:

```text
collect every user's identity
retain every IP
log every route
store every social relationship
```

"just in case".

The governing principle is:

> **SIAR should satisfy lawful operational obligations by minimizing the data that exists in each service boundary, rather than collecting a universal identity and surveillance record across the network.**

---

# 2. Architectural Position

```text
Protocol/Data Plane
        │
        ▼
Privacy-Preserving Service Boundary
        │
        ▼
Operator Legal Boundary
        │
        ├── business/payment/tax
        ├── infrastructure records
        ├── lawful requests
        └── jurisdiction policy
        │
        ▼
Governance / Audit / Transparency
```

---

# 3. Core Separation

Keep distinct:

```text
communication identity
payment identity
operator legal identity
provider runtime identity
infrastructure audit data
moderation evidence
compliance records
```

---

# 4. Non-Goals

Part 55 does not create:

```text
universal KYC
global real-name identity
master decryption keys
general packet logging
global content scanning
jurisdiction-wide backdoor access
```

---

# 5. Legal Boundary Types

```rust
pub enum LegalBoundary {
    UserClient,
    ServiceProvider,
    Operator,
    SettlementEntity,
    GovernanceEntity,
    ManagedOrganization,
}
```

---

# 6. Client Boundary

User-owned endpoint.

Contains:

```text
contact graph
message content
private keys
local search
local call history
```

---

# 7. Service Provider Boundary

Examples:

```text
mailbox
relay
bulk storage
bridge
mix gateway
```

Should hold only the minimum data needed for that service.

---

# 8. Operator Boundary

Operator may have:

```text
business records
infrastructure metadata
provider keys
financial settlement records
```

---

# 9. Settlement Boundary

May contain:

```text
operator payout identity
payment/tax records
```

but not communication identity.

---

# 10. Governance Boundary

Contains:

```text
authority records
policy
operator admission
revocation
audit
```

not user message content.

---

# 11. Managed Organization Boundary

Enterprise/school/org-specific administration.

Separate privacy contract.

---

# 12. Jurisdiction Type

```rust
pub struct JurisdictionId(pub String);
```

Examples may represent:

```text
country
state/province
economic area
organization-specific regulatory zone
```

---

# 13. Jurisdiction Metadata

```rust
pub struct JurisdictionProfile {
    pub jurisdiction_id: JurisdictionId,
    pub data_residency: DataResidencyPolicy,
    pub retention: RetentionPolicy,
    pub service_restrictions: Vec<ServiceRestriction>,
    pub legal_contact: LegalContactRef,
}
```

---

# 14. Jurisdiction Isolation

Services may be grouped by legal region.

---

# 15. Isolation Goal

Avoid unnecessary data replication across legal regions.

---

# 16. Jurisdiction Domain

```rust
pub struct JurisdictionDomainId(pub [u8; 16]);
```

---

# 17. Provider Descriptor Integration

Part 48 provider descriptor can include:

```text
jurisdiction class
residency capability
operator legal entity class
```

---

# 18. Local Selection

Client can enforce jurisdiction policy locally.

---

# 19. User Preference

Possible:

```text
avoid jurisdiction X
prefer region Y
require in-country storage
```

---

# 20. Managed Policy

Organization can require:

```text
approved jurisdiction set
```

---

# 21. Privacy Floor

Jurisdiction policy cannot force client to silently lower anonymity.

---

# 22. No Automatic Legal-Downgrade Route

Hard rule.

---

# 23. Data Classification

Legal/compliance must operate on explicit classes.

---

# 24. Data Classes

```rust
pub enum LegalDataClass {
    UserContent,
    UserIdentity,
    NetworkMetadata,
    OperationalMetadata,
    FinancialRecord,
    GovernanceRecord,
    ModerationEvidence,
    SecurityAudit,
}
```

---

# 25. UserContent

Examples:

```text
messages
files
call media
profile notes
```

---

# 26. UserIdentity

Examples:

```text
account identity
billing identity
organization identity
```

---

# 27. NetworkMetadata

Examples:

```text
IP
session timing
route-related state
```

---

# 28. OperationalMetadata

Examples:

```text
CPU
queue depth
service version
```

---

# 29. FinancialRecord

Examples:

```text
invoice
operator payout
tax record
```

---

# 30. GovernanceRecord

Examples:

```text
policy decision
authority rotation
operator revocation
```

---

# 31. ModerationEvidence

Part 46 separate store.

---

# 32. SecurityAudit

Admin/operator actions.

---

# 33. Data-Minimization Principle

For every field ask:

```text
why is this needed?
who needs it?
for how long?
can it be aggregated?
can it remain local?
```

---

# 34. Data Collection Rule

```rust
pub struct DataCollectionRule {
    pub class: LegalDataClass,
    pub purpose: ProcessingPurpose,
    pub retention: RetentionDuration,
    pub storage_scope: StorageScope,
}
```

---

# 35. Purpose Limitation

A field collected for:

```text
billing
```

must not automatically become:

```text
routing identity
```

---

# 36. Storage Scope

```rust
pub enum StorageScope {
    DeviceLocal,
    ProviderLocal,
    OperatorLocal,
    RegionBound,
    GovernanceControlPlane,
}
```

---

# 37. Default

Prefer narrowest scope.

---

# 38. Retention Classes

```rust
pub enum LegalRetentionClass {
    None,
    Ephemeral,
    ShortOperational,
    Contractual,
    Statutory,
    SecurityIncident,
}
```

---

# 39. None

Do not store.

---

# 40. Ephemeral

Memory/session only.

---

# 41. ShortOperational

Hours/days.

---

# 42. Contractual

Needed for service/accounting.

---

# 43. Statutory

Required by applicable law.

---

# 44. SecurityIncident

Temporary incident retention.

---

# 45. Retention Policy

```rust
pub struct RetentionPolicy {
    pub rules: Vec<RetentionRule>,
}
```

---

# 46. Retention Rule

```rust
pub struct RetentionRule {
    pub data_class: LegalDataClass,
    pub duration: Duration,
    pub legal_basis_ref: Option<String>,
}
```

---

# 47. No Indefinite Default

Hard rule.

---

# 48. Expiry Enforcement

Automatic.

---

# 49. Retention Jobs

Delete expired data.

---

# 50. Deletion Audit

Record:

```text
class
count
period
```

not content.

---

# 51. User Content Retention

Provider should avoid possessing plaintext user content.

---

# 52. Mailbox Content

Opaque ciphertext.

---

# 53. Bulk Objects

Opaque ciphertext.

---

# 54. Relay Media

Ephemeral ciphertext.

---

# 55. Mixnet

Transient anonymous packets.

---

# 56. No Provider Content Decryption Key

Hard rule.

---

# 57. Lawful Request Reality

An operator can only provide what it actually possesses.

---

# 58. Architecture Principle

Do not create new surveillance data solely to satisfy hypothetical future requests.

---

# 59. Lawful Request Interface

Separate control-plane process.

---

# 60. Lawful Request Type

```rust
pub struct LawfulRequest {
    pub request_id: LawfulRequestId,
    pub jurisdiction: JurisdictionId,
    pub authority_ref: LegalAuthorityRef,
    pub scope: LawfulRequestScope,
    pub received_at: Timestamp,
}
```

---

# 61. Lawful Request Scope

```rust
pub enum LawfulRequestScope {
    OperatorRecord,
    FinancialRecord,
    InfrastructureRecord,
    ProviderRecord,
    ModerationRecord,
}
```

---

# 62. No Universal User-Content Scope

Hard rule.

If operator cannot decrypt content, the architecture should not add a backdoor.

---

# 63. Legal Validation

Request must pass:

```text
authenticity
jurisdiction
scope
authority
internal approval
```

---

# 64. Validation State

```rust
pub enum LawfulRequestState {
    Received,
    Validating,
    Rejected,
    Approved,
    Fulfilled,
    Closed,
}
```

---

# 65. Legal Review

Human/legal review required for sensitive requests.

---

# 66. No Automated Bulk Disclosure

Hard rule.

---

# 67. Data Extraction

Only exact data class/scope approved.

---

# 68. Export Manifest

```rust
pub struct LegalDisclosureManifest {
    pub request_id: LawfulRequestId,
    pub exported_classes: Vec<LegalDataClass>,
    pub item_counts: Vec<DisclosureCount>,
    pub approved_by: Vec<LegalApprovalRef>,
}
```

---

# 69. Disclosure Minimization

Do not include:

```text
unrelated accounts
unrelated services
unrelated operators
```

---

# 70. No Cross-Service Join

Hard rule unless independently justified and technically available.

---

# 71. Communication Metadata

If not retained, cannot be disclosed.

---

# 72. Route Metadata

Should not exist as long-term correlated record.

---

# 73. Transparency Reporting

Where legally permitted, publish aggregate:

```text
number of requests
request categories
jurisdiction groups
disclosures/rejections
```

---

# 74. No Case Details In Public Report

Unless law/policy permits and privacy-safe.

---

# 75. Transparency Record

```rust
pub struct LegalTransparencyAggregate {
    pub period: ReportingPeriod,
    pub received: u64,
    pub fulfilled: u64,
    pub rejected: u64,
}
```

---

# 76. Minimum Cohort

Use grouping if counts are small.

---

# 77. Warrant Canary

Policy/legal decision, not required by architecture.

---

# 78. Architecture Neutrality

SIAR does not depend on warrant-canary semantics.

---

# 79. Lawful Interception

Do not build generalized interception interface into E2EE.

---

# 80. Hard Rule

No universal interception/decryption key.

---

# 81. Provider-Level Suspension

Legal authority may require service suspension in a jurisdiction.

---

# 82. Suspension Scope

```rust
pub enum LegalServiceAction {
    SuspendNewService,
    DisableProvider,
    BlockFunding,
    StopLocalOperation,
}
```

---

# 83. No Global Spillover

Jurisdiction-specific action should not automatically shut down unrelated regions.

---

# 84. Jurisdiction-Scoped Provider State

```rust
pub enum JurisdictionServiceState {
    Active,
    Restricted,
    Suspended,
    Retired,
}
```

---

# 85. Client Response

Select another legally available provider if privacy requirements remain satisfied.

---

# 86. No Silent Direct Fallback

Hard rule.

---

# 87. Region Isolation

Services can be deployed into:

```text
RegionGroup A
RegionGroup B
```

with independent legal entities/operators.

---

# 88. Cross-Border Transfer

Must be explicit.

---

# 89. Data Residency Policy

```rust
pub enum DataResidencyPolicy {
    AnyApprovedRegion,
    InJurisdictionOnly,
    InRegionGroup(Vec<JurisdictionId>),
    LocalDeviceOnly,
}
```

---

# 90. User Content

Prefer ciphertext even when crossing borders.

---

# 91. Operational Data

May remain region-bound.

---

# 92. Financial Data

Stay at settlement/legal entity boundary.

---

# 93. Governance Data

Can be globally mirrored if non-personal.

---

# 94. Cross-Border Metadata Minimization

Avoid copying:

```text
IP logs
request identifiers
operator case notes
```

unless needed.

---

# 95. Replication Policy

```rust
pub struct JurisdictionReplicationPolicy {
    pub allowed_targets: Vec<JurisdictionDomainId>,
    pub data_classes: Vec<LegalDataClass>,
}
```

---

# 96. Default Deny

Preferred.

---

# 97. Replication Gate

```rust
pub trait JurisdictionReplicationGate {
    fn permit(
        &self,
        class: LegalDataClass,
        from: JurisdictionDomainId,
        to: JurisdictionDomainId,
    ) -> bool;
}
```

---

# 98. Multi-Region DR

Part 50 must respect residency.

---

# 99. DR Region

Must be legally allowed.

---

# 100. Backup Residency

Explicit.

---

# 101. Backup Data Classes

Control-plane backups contain only approved classes.

---

# 102. No Social Graph Backup To Operator

Hard rule.

---

# 103. Privacy Roles

Map operational roles.

---

# 104. Role Types

```rust
pub enum ComplianceRole {
    OperatorAdmin,
    SecurityOperator,
    LegalReviewer,
    FinancialOfficer,
    GovernanceAuditor,
}
```

---

# 105. Least Privilege

Legal reviewer does not get:

```text
provider signing key
message content
network route trace
```

---

# 106. Financial Officer

Does not get:

```text
contact graph
relay session pair
```

---

# 107. Security Operator

Does not get financial identity by default.

---

# 108. Separation of Duties

Critical disclosures require multi-role approval where appropriate.

---

# 109. Disclosure Approval

```rust
pub struct DisclosureApprovalPolicy {
    pub required_roles: Vec<ComplianceRole>,
    pub minimum_approvals: u8,
}
```

---

# 110. Break-Glass

Possible for critical legal/security operations.

---

# 111. Break-Glass Audit

Mandatory.

---

# 112. Break-Glass Cannot Reveal Data That Does Not Exist

Hard rule.

---

# 113. Data Subject Rights

Depending jurisdiction, users may have rights around:

```text
access
deletion
correction
export
objection
```

---

# 114. Local-First Advantage

Much personal data is device-local.

---

# 115. Operator Response

Can provide:

```text
account/payment records
service metadata it possesses
```

not local content it does not possess.

---

# 116. Data Access Export

Separate from Part 33 user content export.

---

# 117. Privacy Export

```rust
pub struct OperatorDataExport {
    pub data_classes: Vec<LegalDataClass>,
    pub generated_at: Timestamp,
}
```

---

# 118. Deletion Request

Delete what can lawfully be deleted.

---

# 119. Retention Conflict

Some records may need statutory retention.

---

# 120. Deletion Result

Structured:

```text
deleted
retained due to legal requirement
not held
```

---

# 121. No False "Deleted Everywhere"

Hard rule.

---

# 122. Account Deletion

Does not guarantee:

```text
other users' local copies
expired encrypted backups
```

unless technically controlled.

---

# 123. Provider Data Inventory

Every provider should declare what it stores.

---

# 124. Data Inventory Record

```rust
pub struct ProviderDataInventory {
    pub provider: AnonymousProviderId,
    pub stored_classes: Vec<LegalDataClass>,
    pub retention: RetentionPolicy,
}
```

---

# 125. Inventory Accuracy

Required.

---

# 126. Privacy Policy Generation

Can be derived partly from inventory.

---

# 127. No Marketing Overclaim

Hard rule.

---

# 128. Example

Do not claim:

```text
we store no metadata
```

if relay stores short-lived operational metadata.

---

# 129. Better

State:

```text
what
why
how long
```

---

# 130. Policy Versioning

Legal/privacy policy versioned.

---

# 131. Policy Type

```rust
pub struct LegalOperationsPolicyVersion(pub u64);
```

---

# 132. User Notice

Material changes require notice.

---

# 133. No Retroactive Expansion

Do not reinterpret old data for unrelated new purpose without legal/policy review.

---

# 134. Consent

Where needed, explicit.

---

# 135. Consent Record

```rust
pub struct ConsentRecord {
    pub policy_version: LegalOperationsPolicyVersion,
    pub purpose: ProcessingPurpose,
    pub granted_at: Timestamp,
}
```

---

# 136. Strict Mode

Avoid consent-dependent server processing where local processing suffices.

---

# 137. Age/Managed User Policies

Product-specific.

---

# 138. Architecture Boundary

Managed profiles may have additional controls, but anonymity/security primitives remain same.

---

# 139. Sanctions/Restricted Parties

Financial/operator settlement may need screening.

---

# 140. Screening Boundary

At:

```text
operator payout
funding/legal entity
```

not user communication path.

---

# 141. No Network-Level User Screening Requirement

Hard rule unless a specific service legally requires it.

---

# 142. Export-Control Boundaries

Software/distribution decisions may vary by jurisdiction.

---

# 143. Release Governance

Part 52 + Part 53 handle distribution restrictions.

---

# 144. Do Not Encode Legal Rule Into Cryptographic Primitive

Hard rule.

---

# 145. Why

Laws change faster than protocol.

---

# 146. Legal Policy Layer

Configuration/policy above protocol.

---

# 147. Compliance Adapter

```rust
pub trait CompliancePolicyAdapter {
    fn evaluate_service(
        &self,
        jurisdiction: &JurisdictionProfile,
        service: AnonymousServiceType,
    ) -> ComplianceDecision;
}
```

---

# 148. Compliance Decision

```rust
pub enum ComplianceDecision {
    Allowed,
    AllowedWithRestrictions(Vec<ServiceRestriction>),
    Suspended,
}
```

---

# 149. Service Restriction

Examples:

```text
storage residency
retention maximum/minimum
funding unavailable
provider class restricted
```

---

# 150. Hard Privacy Constraint Merge

```rust
pub trait LegalPrivacyPolicyMerger {
    fn merge(
        &self,
        legal: &ComplianceDecision,
        privacy: &EffectivePolicy,
    ) -> Result<EffectiveServicePolicy, PolicyMergeError>;
}
```

---

# 151. Merge Rule

Legal policy may restrict availability.

It must not enable a weaker privacy path.

---

# 152. Example

If lawful operation prohibits anonymous relay in one region:

```text
disable that provider/service
```

not:

```text
connect directly
```

---

# 153. Operator Legal Identity

Provider operator may be legal entity.

---

# 154. Provider Runtime Identity

Cryptographic operator/provider IDs separate.

---

# 155. Mapping

Legal entity ↔ operator ID held in governance/settlement boundary.

---

# 156. Public Exposure

Can publish organization name if operator chooses/required.

---

# 157. Private Ownership Records

Separate.

---

# 158. Legal Contact

Abuse/legal contact can be published.

---

# 159. No User Contact Through Operator

Hard rule.

---

# 160. Content Moderation Law

Jurisdiction-specific.

---

# 161. Architecture Separation

Part 46 moderation is scoped capability moderation.

Part 55 does not convert operators into content decryptors.

---

# 162. Moderation Evidence

Only available if voluntarily/report-generated.

---

# 163. No Server-Side Plaintext Scanning Requirement

Architecture does not include it.

---

# 164. Operator Abuse Workflow

Can act on:

```text
provider abuse
resource abuse
valid reports
```

without decrypting E2EE content.

---

# 165. Service Terms Enforcement

Use:

```text
capability revocation
quota
provider suspension
```

---

# 166. User Safety vs Legal Compliance

Separate policy domains.

---

# 167. Legal Request Audit Store

Separate from ordinary observability.

---

# 168. Request Audit Record

```rust
pub struct LegalRequestAuditRecord {
    pub request_id: LawfulRequestId,
    pub state: LawfulRequestState,
    pub reviewer_roles: Vec<ComplianceRole>,
    pub timestamp: Timestamp,
}
```

---

# 169. Sensitive Retention

Long enough for legal audit.

---

# 170. No Request Details In General Logs

Hard rule.

---

# 171. Case Store

Encrypted.

---

# 172. Access

Highly restricted.

---

# 173. Case ID

Never becomes provider/user runtime identifier.

---

# 174. Lawful Disclosure Export

Encrypted package.

---

# 175. Chain of Custody

If required:

```text
hash
sign
audit
```

---

# 176. No Hidden Live Access Interface

Hard rule.

---

# 177. Live Monitoring Requests

Architecture does not include persistent interception API.

---

# 178. If Law Requires Service Shutdown

Operator can:

```text
disable service/provider
```

---

# 179. Shutdown Behavior

Graceful where legally possible:

```text
stop new allocations
drain
expire
```

---

# 180. Immediate Suspension

For urgent cases.

---

# 181. Client State

Queued locally.

---

# 182. Provider Migration

If alternative legal/private provider exists.

---

# 183. No Decryption During Shutdown

Hard rule.

---

# 184. Provider Exit

Part 50 continuity.

---

# 185. Legal Hold

Some records may need preservation.

---

# 186. Legal Hold Scope

Only existing relevant records.

---

# 187. No New Collection Because Of Hold

Hard rule.

---

# 188. Legal Hold Record

```rust
pub struct LegalHold {
    pub hold_id: LegalHoldId,
    pub data_classes: Vec<LegalDataClass>,
    pub scope: LegalHoldScope,
    pub expires_at: Option<Timestamp>,
}
```

---

# 189. Hold Scope

Must be narrow.

---

# 190. Hold Does Not Disable Encryption

Hard rule.

---

# 191. Hold Expiry/Review

Periodic.

---

# 192. Compliance Logging

Only actions/process.

---

# 193. No Full Decision Context In General Ops Log

Hard rule.

---

# 194. Compliance SLOs

Examples:

```text
retention deletion completed
policy propagation
legal request audit completeness
residency enforcement
```

---

# 195. Privacy Compliance SLO

Examples:

```text
zero cross-jurisdiction unauthorized replication
zero retention beyond policy without legal hold
zero unauthorized disclosure
```

---

# 196. Error Budget

Critical privacy/legal violations:

```text
zero
```

---

# 197. Compliance Incident Types

```rust
pub enum ComplianceIncident {
    UnauthorizedReplication,
    RetentionOverrun,
    UnauthorizedDisclosure,
    ResidencyViolation,
    PolicyMismatch,
    LegalRequestProcessFailure,
}
```

---

# 198. Incident Severity

High/critical depending scope.

---

# 199. Incident Response

Contain:

```text
replication
access
provider
policy
```

---

# 200. Notification

Jurisdiction-specific.

---

# 201. Architecture Support

Record:

```text
scope
classes
timeline
```

without exposing more content.

---

# 202. Data Breach Boundary

Encrypted user content may still be compromised as ciphertext.

---

# 203. Key Compromise

Different severity.

---

# 204. If Provider Never Had Key

Breach impact lower.

---

# 205. Cryptographic Separation

Major compliance advantage.

---

# 206. Cross-Service Data Join

Forbidden by default.

---

# 207. Analytics Warehouse

Do not build unified user-centric compliance warehouse.

---

# 208. Better

Purpose-specific stores.

---

# 209. Stores

```text
financial records
legal requests
operator registry
security audit
moderation evidence
```

separate.

---

# 210. Common Identifier

Avoid global user key across stores.

---

# 211. Scoped Case Reference

Allowed.

---

# 212. Data Residency Enforcement

Can be type/policy enforced.

---

# 213. Residency Tag

```rust
pub struct ResidencyTag {
    pub domain: JurisdictionDomainId,
}
```

---

# 214. Region-Bound Store

Requires matching tag.

---

# 215. Typed Storage Wrapper

```rust
pub struct RegionBound<T> {
    pub residency: ResidencyTag,
    pub value: T,
}
```

---

# 216. Transfer Function

Requires policy approval.

---

# 217. Compile-Time Assistance

Rust types can make accidental cross-region copy harder.

---

# 218. Compliance-Safe DTOs

Never reuse broad domain object blindly.

---

# 219. Example

Legal export DTO contains:

```text
approved fields only
```

---

# 220. No Generic Serialize On Secret-Rich Types

Hard rule.

---

# 221. Disclosure Builder

```rust
pub trait LegalDisclosureBuilder {
    fn build(
        &self,
        request: &ApprovedLawfulRequest,
        sources: &dyn LegalDataSource,
    ) -> Result<LegalDisclosurePackage, ComplianceError>;
}
```

---

# 222. Data Source

Exposes only whitelisted classes.

---

# 223. Compliance Gate

```rust
pub trait LegalDataAccessGate {
    fn authorize(
        &self,
        actor: ComplianceActor,
        request: LegalDataAccessRequest,
    ) -> Result<ApprovedLegalAccess, ComplianceError>;
}
```

---

# 224. No Direct DB Access

Preferred.

---

# 225. Audit Every Access

Mandatory.

---

# 226. Legal Data Export Package

```rust
pub struct LegalDisclosurePackage {
    pub manifest: LegalDisclosureManifest,
    pub encrypted_payload: Bytes,
    pub integrity: DisclosureIntegrityProof,
}
```

---

# 227. Integrity

Signed/hash.

---

# 228. Privacy Review

Before introducing new collected legal field.

---

# 229. Collection Proposal

Must document:

```text
purpose
law/policy basis
retention
scope
alternatives
privacy impact
```

---

# 230. Compliance Schema Registry

Similar to telemetry registry.

---

# 231. Registry Entry

```rust
pub struct ComplianceDataField {
    pub field_name: &'static str,
    pub data_class: LegalDataClass,
    pub retention: LegalRetentionClass,
    pub purpose: ProcessingPurpose,
}
```

---

# 232. Unknown Remote Field

Rejected.

---

# 233. CI Compliance Gate

Fail build if new regulated field lacks registry entry.

---

# 234. Data Flow Inventory

Machine-readable.

---

# 235. Data Flow Edge

```rust
pub struct DataFlowEdge {
    pub class: LegalDataClass,
    pub from: StorageScope,
    pub to: StorageScope,
    pub jurisdiction_rule: JurisdictionReplicationPolicy,
}
```

---

# 236. Architecture Diagram Generation

Can derive compliance maps.

---

# 237. Data Protection Impact Assessment Support

Architecture can export:

```text
data classes
flows
retention
purposes
```

---

# 238. No Legal Conclusion Automation

Hard rule.

---

# 239. Operator Handbook

Must include:

```text
data inventory
retention
lawful request process
incident process
residency matrix
```

---

# 240. Jurisdiction Profile Files

RON/config.

---

# 241. Signed Runtime Policy

Governance-approved.

---

# 242. Human Legal Source

Separate from compiled runtime policy.

---

# 243. Update Workflow

```text
legal review
→ policy proposal
→ technical/privacy review
→ signed governance policy
→ staged rollout
```

---

# 244. Emergency Legal Change

May suspend service quickly.

---

# 245. Cannot Introduce Backdoor Via Emergency Config

Hard rule.

---

# 246. Policy Precedence

Recommended:

```text
compile-time security/privacy invariant
> signed governance floor
> jurisdiction restriction
> managed policy
> user preference
> performance
```

---

# 247. Jurisdiction Restriction

May remove options.

Cannot add unsafe options.

---

# 248. User Travel

User location should not need exact tracking.

---

# 249. Jurisdiction Determination

Prefer:

```text
chosen provider/operator jurisdiction
configured account/org jurisdiction
```

over device precise geolocation.

---

# 250. No Precise Location Requirement

Hard rule.

---

# 251. Provider Geofencing

If legally necessary, provider may enforce at service edge.

---

# 252. Privacy Caveat

IP-based region checks expose coarse network location to that provider.

---

# 253. Strict Mode

Avoid centralized global geolocation service.

---

# 254. Policy Selection

Client can avoid provider before connection based on catalog jurisdiction metadata.

---

# 255. Cross-Border Calls

Realtime relays may span regions.

---

# 256. User/Managed Policy

Can restrict relay jurisdictions.

---

# 257. Mixnet Routes

Jurisdiction diversity may be a privacy feature.

---

# 258. Legal Restriction

May reduce eligible nodes.

---

# 259. Failure Behavior

If insufficient eligible nodes:

```text
stop/queue
```

not weaker route.

---

# 260. Operator Tax Records

Separate accounting store.

---

# 261. User Invoice

Funding boundary.

---

# 262. Anonymous Credit Spend

Does not carry invoice identity.

---

# 263. Payment Processor Data

Outside anonymous transport plane.

---

# 264. KYC Where Required

At funding/operator payout boundary.

---

# 265. No KYC Propagation

Hard rule.

---

# 266. Organization Contracts

Enterprise operator may need identifiable customer account.

---

# 267. Contract Boundary

Still separate from per-message/per-route identity.

---

# 268. Tenant Identity

Can be organization-scoped.

---

# 269. No End-User Graph Exposure

Hard rule unless organization product explicitly requires and discloses it.

---

# 270. Compliance Observability

Safe metrics:

```text
retention jobs
request counts
residency violations
policy freshness
```

---

# 271. Forbidden Metrics

No:

```text
message subject
contact pair
route path
full legal case details
```

---

# 272. Legal Request Metrics

Aggregate only.

---

# 273. Support Access

Support cannot access legal case store by default.

---

# 274. SRE Access

SRE cannot access full financial/legal record by default.

---

# 275. RBAC Tests

Mandatory.

---

# 276. Legal Hold Tests

Ensure only scoped records protected from deletion.

---

# 277. Retention Tests

Expired data deleted.

---

# 278. Residency Tests

Forbidden cross-region write rejected.

---

# 279. Disclosure Tests

Export contains only approved classes.

---

# 280. No-Data Tests

Request for non-held data returns:

```text
not held
```

not reconstruction.

---

# 281. Backdoor Test

No operator API can decrypt E2EE content.

---

# 282. Shutdown Test

Suspended jurisdiction provider drains/stops without direct fallback.

---

# 283. DR Test

Backup restore does not violate residency.

---

# 284. Policy Conflict Test

Jurisdiction restriction + privacy policy merge correctly.

---

# 285. Cross-Border Test

Allowed ciphertext flow works.

---

# 286. Forbidden Transfer Test

Blocked metadata flow rejected.

---

# 287. Fuzzing

Fuzz:

```text
jurisdiction policy
retention rule
legal request manifest
disclosure package
```

---

# 288. Property Tests

Properties:

```text
legal restriction never enables weaker privacy path
expired retention item is not returned absent valid hold
disclosure package never contains unapproved data class
cross-region transfer requires explicit allow policy
```

---

# 289. Formal Verification Targets

Strong candidates:

```text
policy precedence
retention + legal hold state machine
residency transfer gate
lawful request approval workflow
```

---

# 290. TLA+ Candidate

Retention/deletion/legal-hold concurrency.

---

# 291. Kani Candidate

Jurisdiction policy merge and transfer gate.

---

# 292. Loom Candidate

Concurrent retention job vs legal hold activation.

---

# 293. Performance Tests

Measure:

```text
policy evaluation
retention scans
residency checks
legal export generation
```

---

# 294. No Per-Packet Legal Policy Query

Hard rule.

---

# 295. Cache Effective Policy

Signed/versioned.

---

# 296. Policy Refresh

Low-frequency.

---

# 297. Emergency Revocation

Higher priority.

---

# 298. Crate Layout

Recommended:

```text
crates/
├── siar-compliance-core/
├── siar-jurisdiction/
├── siar-data-classification/
├── siar-retention/
├── siar-residency/
├── siar-lawful-request/
├── siar-disclosure/
├── siar-legal-audit/
├── siar-compliance-policy/
├── siar-compliance-observability/
└── siar-compliance-testkit/
```

---

# 299. `siar-compliance-core`

Owns:

```text
legal data classes
roles
request states
errors
```

---

# 300. `siar-jurisdiction`

Jurisdiction profiles/service restrictions.

---

# 301. `siar-data-classification`

Purpose/scope/data-class registry.

---

# 302. `siar-retention`

TTL/legal hold/deletion.

---

# 303. `siar-residency`

Cross-region transfer gate.

---

# 304. `siar-lawful-request`

Validation/approval state machine.

---

# 305. `siar-disclosure`

Minimal approved export packages.

---

# 306. `siar-legal-audit`

Case access/disclosure audit.

---

# 307. `siar-compliance-policy`

Runtime legal policy compile/merge.

---

# 308. `siar-compliance-observability`

Privacy-safe process metrics.

---

# 309. `siar-compliance-testkit`

Residency/retention/disclosure simulations.

---

# 310. Error Taxonomy

```rust
pub enum ComplianceError {
    JurisdictionNotAllowed,
    ResidencyViolation,
    RetentionPolicyViolation,
    LegalHoldConflict,
    RequestUnauthorized,
    DisclosureScopeViolation,
    ApprovalInsufficient,
    PolicyStale,
    DataClassForbidden,
    ExportFailed,
    Internal,
}
```

---

# 311. Security & Compliance Invariants

Mandatory:

```text
1. Legal/compliance processing does not create a universal user identity across SIAR services.
2. E2EE service providers do not receive a universal content-decryption key.
3. Lawful requests can return only data already lawfully held by the responding boundary.
4. Jurisdiction restrictions may reduce availability but never silently weaken required anonymity.
5. Retention is explicit, bounded, and automatically enforced.
6. Legal holds preserve only existing scoped records; they do not enable new surveillance collection.
7. Cross-jurisdiction replication is deny-by-default and policy-gated.
8. Financial/tax/KYC identity remains at the funding or operator-payout boundary unless a specific service lawfully requires otherwise.
9. Legal request, financial, moderation, governance, and operational stores remain purpose-separated.
10. Disclosure exports are allowlisted by data class and fully audited.
11. No hidden live-interception or generalized deanonymization interface exists.
12. Compliance policy cannot override compile-time privacy/security invariants.
```

---

# 312. Initial Production Scope

Implement first:

```text
explicit legal/data classifications
provider data inventory
bounded retention engine
jurisdiction/provider metadata
local jurisdiction selection policy
region-bound storage policy
lawful request validation workflow
minimal disclosure builder
legal request audit store
operator/payment identity separation
signed compliance policy
residency/retention CI tests
```

Then add:

```text
multi-jurisdiction policy packs
automated DPIA/data-flow reports
multi-entity legal operations federation
advanced legal-hold workflow
regional transparency reporting
formal compliance-policy verification
```

---

# 313. Definition of Done

Part 55 is complete when:

- legal boundaries for clients/providers/operators/settlement/governance are explicit
- data classes, purposes, storage scopes, and retention are machine-readable
- provider data inventories are declared
- jurisdiction metadata and residency policies influence provider selection locally
- cross-border replication is explicitly gated
- lawful requests are validated, approved, scoped, audited, and unable to access data the operator never possessed
- no universal interception/decryption/deanonymization mechanism exists
- retention and legal-hold semantics are clearly separated
- financial/KYC/tax identity remains isolated from communication identity
- legal service suspension cannot trigger direct/privacy-weaker fallback
- DR and backups honor jurisdiction/residency rules
- compliance stores are purpose-separated and least-privilege
- transparency reporting is aggregate/privacy-safe
- retention, residency, legal request, shutdown, disclosure, fuzz, and formal tests are specified

---

# 314. Final Architecture

```text
                  ANONYMOUS DATA PLANE
                          │
                          ▼
                 MINIMAL SERVICE STATE
                          │
             ┌────────────┼────────────┐
             │            │            │
          Provider     Operator     Settlement
             │            │            │
             └────────────┼────────────┘
                          ▼
                JURISDICTION POLICY GATE
                          │
                          ▼
               LEGAL / RETENTION / AUDIT
```

Compliance safety model:

```text
minimal data
+
purpose separation
+
jurisdiction isolation
+
bounded retention
+
scoped lawful process
+
audited disclosure
```

not:

```text
collect everything so compliance can decide later
```

---

# 315. Final Principle

The strongest legal/privacy architecture is often the one that deliberately **does not possess unnecessary sensitive data**.

The correct model is:

```text
data minimization
+
service-local boundaries
+
jurisdiction-aware policy
+
purpose limitation
+
bounded retention
+
lawful scoped disclosure
+
no universal backdoor
```

This architecture gives SIAR a path to operate lawfully across different jurisdictions while preserving the anonymity, identity-separation, and privacy-floor guarantees established across Parts 34–54.
