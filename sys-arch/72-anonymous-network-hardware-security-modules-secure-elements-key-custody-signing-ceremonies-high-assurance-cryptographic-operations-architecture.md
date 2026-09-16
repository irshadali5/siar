# Core System Architecture Part 72 — Anonymous Network Hardware Security Modules, Secure Elements, Key Custody, Signing Ceremonies & High-Assurance Cryptographic Operations Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 72  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 28, 52–53, 61–71  

**Primary purpose:** define SIAR's HSM/secure-element architecture, high-value key custody, signing ceremonies, dual-control, threshold/quorum operations, root/governance/release/provider signing workflows, backup/recovery of cryptographic authority, key compromise response, evidence/audit, and operational controls so critical cryptographic operations remain isolated from ordinary runtime and administrator authority.

---

# 1. Purpose

Some keys can compromise the entire network if mishandled.

Examples:

```text
governance root
release-signing key
directory authority key
provider-signing key
federation trust key
tenant root key
```

These keys require stronger controls than normal application secrets.

The governing principle is:

> **High-value cryptographic authority must be isolated, role-separated, ceremony-controlled, auditable, and recoverable without creating a universal master secret or a single-person failure point.**

---

# 2. Architectural Position

```text
Governance / Release / Root Authority
               │
               ▼
        Ceremony Coordinator
               │
     ┌─────────┼─────────┐
     │         │         │
 Custodian A Custodian B Custodian C
     │         │         │
     └─────────┼─────────┘
               ▼
         HSM / Secure Element
               │
               ▼
        Signed Trust Artifact
```

---

# 3. Core Separation

Keep distinct:

```text
root key
governance signing key
release signing key
provider signing key
runtime credential
service secret
device key
user key
backup key
```

---

# 4. Non-Goals

Part 72 does not:

```text
use HSMs for every ephemeral key
create one universal root for all domains
allow operators to export high-value private keys casually
treat cloud KMS as equivalent to offline root custody
```

---

# 5. Key Assurance Classes

```rust
pub enum KeyAssuranceClass {
    Routine,
    Sensitive,
    HighValue,
    RootAuthority,
}
```

---

# 6. Routine

Examples:

```text
ephemeral session
temporary runtime credential
```

---

# 7. Sensitive

Examples:

```text
provider service key
tenant service key
```

---

# 8. HighValue

Examples:

```text
release signing
directory authority
federation trust
```

---

# 9. RootAuthority

Examples:

```text
governance root
trust-root rollover key
```

---

# 10. Custody Policy

```rust
pub struct KeyCustodyPolicy {
    pub assurance: KeyAssuranceClass,
    pub storage: KeyStorageRequirement,
    pub approvals: ApprovalRequirement,
    pub exportability: ExportPolicy,
}
```

---

# 11. Storage Requirements

```rust
pub enum KeyStorageRequirement {
    SoftwareProtected,
    SecureElement,
    Hsm,
    OfflineHsm,
    MultiHsm,
}
```

---

# 12. Routine Keys

Can use secure software storage.

---

# 13. HighValue Keys

Prefer HSM.

---

# 14. RootAuthority Keys

Prefer offline HSM or equivalent high-assurance custody.

---

# 15. No Root Key On General Server

Hard rule.

---

# 16. HSM Role

HSM should:

```text
generate key
store key
sign
enforce policy
audit use
```

---

# 17. HSM Should Not

Expose private key material by default.

---

# 18. Exportability

```rust
pub enum ExportPolicy {
    NonExportable,
    WrappedExportOnly,
    ExportableForMigration,
}
```

---

# 19. Root Keys

`NonExportable` preferred.

---

# 20. Wrapped Export

May be needed for:

```text
vendor migration
backup
```

---

# 21. Hard Rule

Plaintext private-key export is prohibited for HighValue/RootAuthority keys.

---

# 22. HSM Vendor Abstraction

Avoid hard coupling.

---

# 23. HSM Backend Trait

```rust
pub trait HsmBackend {
    fn generate_key(
        &self,
        spec: KeyGenerationSpec,
    ) -> Result<HsmKeyHandle, HsmError>;

    fn sign(
        &self,
        key: &HsmKeyHandle,
        request: SignRequest,
    ) -> Result<Signature, HsmError>;
}
```

---

# 24. HSM Key Handle

Opaque.

---

# 25. No Raw Key Bytes

Hard rule.

---

# 26. Secure Element

Useful for:

```text
embedded appliance
edge node
managed device profile
```

---

# 27. Secure Element vs HSM

Secure element:

```text
device-local
smaller scope
```

HSM:

```text
central/high-value custody
multi-operator use
```

---

# 28. TPM

Different again.

---

# 29. TPM

Host attestation/sealing.

---

# 30. HSM

Authority signing/key custody.

---

# 31. No Conceptual Conflation

Hard rule.

---

# 32. Key Ownership

Every high-value key has:

```text
purpose
owner
custodian set
rotation policy
recovery policy
```

---

# 33. Key Custody Record

```rust
pub struct KeyCustodyRecord {
    pub key_id: KeyId,
    pub purpose: KeyPurpose,
    pub assurance: KeyAssuranceClass,
    pub custodians: Vec<CustodianId>,
    pub rotation_policy: KeyRotationPolicy,
}
```

---

# 34. Custodian Identity

Separate from messaging identity.

---

# 35. Custodian ID

```rust
pub struct CustodianId(pub [u8; 32]);
```

---

# 36. No Personal Email As Authority

Hard rule.

---

# 37. Custodian Authentication

Prefer:

```text
hardware token
smartcard
HSM operator credential
MFA
```

---

# 38. Single-Person Authority

Avoid.

---

# 39. Dual Control

At least two distinct approvals for critical actions.

---

# 40. Dual Control Policy

```rust
pub struct DualControlPolicy {
    pub required_approvals: u8,
    pub distinct_roles: bool,
}
```

---

# 41. Root Action

May require:

```text
3-of-5
```

or similar.

---

# 42. Threshold Policy

```rust
pub struct ApprovalRequirement {
    pub threshold: u8,
    pub total_custodians: u8,
}
```

---

# 43. Important Distinction

Approval quorum does not necessarily mean threshold cryptography.

---

# 44. Simple Safe Model

Use multiple independent signatures.

---

# 45. Threshold Cryptography

Only if mature/reviewed implementation exists.

---

# 46. No Custom Threshold Crypto

Hard rule.

---

# 47. Ceremony

A ceremony is a controlled high-risk cryptographic operation.

---

# 48. Ceremony Types

```rust
pub enum CeremonyType {
    RootGeneration,
    GovernanceKeyGeneration,
    ReleaseKeyGeneration,
    KeyRotation,
    RootRollover,
    EmergencyRevocation,
    KeyRecovery,
    KeyMigration,
}
```

---

# 49. Ceremony Manifest

```rust
pub struct CeremonyManifest {
    pub ceremony_id: CeremonyId,
    pub ceremony_type: CeremonyType,
    pub expected_artifacts: Vec<CeremonyArtifactType>,
    pub required_roles: BTreeSet<CeremonyRole>,
}
```

---

# 50. Ceremony ID

Infrastructure-only.

---

# 51. Ceremony Roles

```rust
pub enum CeremonyRole {
    Coordinator,
    Custodian,
    Witness,
    Auditor,
    SecurityObserver,
}
```

---

# 52. Coordinator

Runs procedure.

---

# 53. Custodian

Provides authority.

---

# 54. Witness

Confirms process.

---

# 55. Auditor

Checks evidence.

---

# 56. Security Observer

Can stop unsafe action.

---

# 57. No One Person Holds All Roles

Hard rule for root ceremonies.

---

# 58. Ceremony Environment

Prefer:

```text
offline
controlled room
known hardware
pre-verified software
```

for root actions.

---

# 59. Network Isolation

Root-generation ceremony should be offline where practical.

---

# 60. No Internet Dependency

Hard rule for RootAuthority generation.

---

# 61. Ceremony Workstation

Dedicated.

---

# 62. Boot State

Verified.

---

# 63. Software

Pinned and signed.

---

# 64. USB Media

Controlled.

---

# 65. No Personal Laptop

Hard rule.

---

# 66. Ceremony Software

Small trusted surface.

---

# 67. Prefer

```text
purpose-built CLI
no GUI complexity
no browser
```

---

# 68. Ceremony CLI

Typed.

---

# 69. Example

```text
siar-ceremony root generate
```

---

# 70. No Shell Script As Sole Critical Procedure

Hard rule.

---

# 71. Ceremony Plan

Pre-generated.

---

# 72. Dry Run

Mandatory.

---

# 73. Dry Run Uses

```text
test HSM
test keys
synthetic trust root
```

---

# 74. No Production Key During Dry Run

Hard rule.

---

# 75. Root Generation

Steps conceptually:

```text
verify environment
verify custodians
initialize HSM
generate key
record public key
generate trust-root bundle
multi-party verify
seal/export approved backup
destroy temporary state
```

---

# 76. Root Private Key

Never leaves protected custody.

---

# 77. Public Root

Can be widely distributed.

---

# 78. Public Root Fingerprint

Human-verifiable.

---

# 79. Root Fingerprint

```rust
pub struct RootFingerprint(pub [u8; 32]);
```

---

# 80. Multiple Channels

Publish fingerprint via independent paths.

---

# 81. No Single Website As Root Authenticity Source

Hard rule.

---

# 82. Governance Keys

May be online/offline depending role.

---

# 83. Governance Root

Offline.

---

# 84. Governance Operational Signers

Could be online HSM-backed.

---

# 85. Hierarchy

```text
offline root
→ operational authority certificates
→ signed policy bundles
```

---

# 86. Limited Delegation

Operational signer scope narrow.

---

# 87. Scope Certificate

```rust
pub struct SigningScopeCertificate {
    pub signer: KeyId,
    pub allowed_artifact_types: BTreeSet<SignedArtifactType>,
    pub valid_until: Timestamp,
}
```

---

# 88. Release Signing

Separate from governance.

---

# 89. Hard Rule

Release signer cannot sign governance policy.

---

# 90. Governance signer cannot sign arbitrary release artifact.

---

# 91. Domain Separation

Mandatory.

---

# 92. Release Signing Flow

```text
build artifact
→ verify provenance
→ verify SBOM/tests
→ create release manifest
→ approval quorum
→ HSM sign
→ publish
```

---

# 93. HSM Never Signs Raw Unreviewed Binary

Hard rule.

---

# 94. Sign Digest/Manifest

Preferred.

---

# 95. Signing Request

```rust
pub struct SignRequest {
    pub artifact_type: SignedArtifactType,
    pub digest: ContentDigest,
    pub policy_context: SigningPolicyContext,
}
```

---

# 96. Policy Context

Includes:

```text
release version
security epoch
approvals
```

---

# 97. Signing Service

Minimal.

---

# 98. No General Arbitrary Sign API

Hard rule.

---

# 99. Signing Service Trait

```rust
pub trait HighAssuranceSigner {
    fn sign(
        &self,
        request: AuthorizedSignRequest,
    ) -> Result<AlgorithmBoundSignature, SigningError>;
}
```

---

# 100. Authorized Sign Request

Constructed only after policy approval.

---

# 101. Approval Engine

```rust
pub trait SigningApprovalEngine {
    fn authorize(
        &self,
        proposal: &SigningProposal,
    ) -> Result<AuthorizedSignRequest, SigningError>;
}
```

---

# 102. Separation Of Duties

Builder != approver != signer operator where practical.

---

# 103. No CI Direct HSM Admin Credential

Hard rule.

---

# 104. CI Can Submit Proposal

Yes.

---

# 105. Human/Policy Review

Required.

---

# 106. Automated Signing

Possible for low-risk signed objects.

---

# 107. But

Root/release/governance signing requires stronger approval.

---

# 108. Key Generation

Prefer inside HSM.

---

# 109. Import

Avoid if possible.

---

# 110. If Import Needed

Use wrapped key.

---

# 111. Import Ceremony

High-risk.

---

# 112. Export

Even higher risk.

---

# 113. Key Migration

HSM vendor migration.

---

# 114. Migration Flow

```text
create target HSM domain
establish wrapping trust
export wrapped
import wrapped
verify public key continuity
revoke old custody path
```

---

# 115. No Plaintext Migration

Hard rule.

---

# 116. Key Backup

Need recoverability.

---

# 117. But

Backup must not create weaker copy.

---

# 118. HSM Backup

Options:

```text
wrapped backup
vendor secure backup
M-of-N recovery shares
```

---

# 119. Recovery Shares

Need mature reviewed scheme.

---

# 120. No Homegrown Secret Sharing

Hard rule.

---

# 121. Shamir

Can be used only through reviewed implementation with operational controls.

---

# 122. Share Custody

Different people/locations.

---

# 123. No All Shares Same Safe

Hard rule.

---

# 124. Geographic Separation

Recommended.

---

# 125. But

Jurisdiction/privacy/legal policy may constrain.

---

# 126. Recovery Share Metadata

No secret value in general inventory.

---

# 127. Recovery Policy

```rust
pub struct KeyRecoveryPolicy {
    pub required_shares: Option<u8>,
    pub required_custodians: u8,
    pub offline_only: bool,
}
```

---

# 128. Recovery Ceremony

Test periodically with test keys.

---

# 129. No Untested Root Recovery

Hard rule.

---

# 130. Production Root Recovery Test

Avoid casually.

---

# 131. Use Parallel Test Root

Recommended.

---

# 132. Key Rotation

Routine planned event.

---

# 133. High-Value Rotation

Ceremony-controlled.

---

# 134. Rotation Sequence

```text
generate next
→ certify next
→ distribute trust
→ overlap
→ prefer next
→ retire old
→ revoke/destroy old
```

---

# 135. Root Rollover

Part 53/66.

---

# 136. Stronger Ceremony

---

# 137. Dual Trust Window

Limited duration.

---

# 138. Anti-Rollback

Old root cannot return.

---

# 139. Hard rule.

---

# 140. Key Destruction

Documented ceremony.

---

# 141. Destruction Event

```rust
pub struct KeyDestructionRecord {
    pub key_id: KeyId,
    pub destroyed_at: Timestamp,
    pub method: KeyDestructionMethod,
}
```

---

# 142. Destruction Method

```rust
pub enum KeyDestructionMethod {
    HsmDelete,
    TokenZeroize,
    CryptoErase,
    PhysicalDestruction,
}
```

---

# 143. Physical Destruction

For retired HSM/token where necessary.

---

# 144. Evidence

Photographic/serial evidence possible internally.

---

# 145. No Public Infrastructure Serial Disclosure

Hard rule.

---

# 146. HSM Partitioning

Separate keys by domain.

---

# 147. Example

```text
release partition
governance partition
provider partition
tenant root partition
```

---

# 148. No Shared HSM Admin Domain If Avoidable

Hard rule.

---

# 149. Cloud HSM

Acceptable for some high-value keys.

---

# 150. Cloud HSM Limitations

Provider dependency.

---

# 151. RootAuthority

Prefer independent/offline custody.

---

# 152. Cloud KMS

Useful for:

```text
routine/sensitive keys
```

---

# 153. Cloud KMS ≠ Offline Root

Hard rule.

---

# 154. Multi-HSM

High assurance.

---

# 155. Multi-HSM Modes

```text
active/passive
independent quorum
geographic split
```

---

# 156. No Hidden Automatic Failover For Root Signing

Hard rule.

---

# 157. HSM Availability

Signing service should tolerate one device outage.

---

# 158. But

Do not reduce approval threshold.

---

# 159. Hard rule.

---

# 160. HSM Firmware

Part of trust chain.

---

# 161. Firmware Upgrade

High-risk.

---

# 162. Upgrade Process

```text
vendor advisory review
test unit validation
backup/recovery readiness
maintenance ceremony
post-upgrade attestation
```

---

# 163. No Blind Auto-Upgrade HSM Firmware

Hard rule.

---

# 164. HSM Attestation

If vendor provides.

---

# 165. Use as additional evidence.

---

# 166. No Vendor Attestation = Trust Proof

Hard rule.

---

# 167. Secure Element Provisioning

Factory/field.

---

# 168. Provisioning Key

Dedicated.

---

# 169. No Shared Fleet Master Secret

Hard rule.

---

# 170. Device Secure Element

Per-device key.

---

# 171. Managed Device Enterprise

Org-scoped.

---

# 172. Anonymous User Device

Do not require global hardware identity.

---

# 173. HSM Session Security

Use authenticated channel.

---

# 174. Operator Login

Hardware-backed.

---

# 175. Session Timeout

Short.

---

# 176. No Shared Admin Account

Hard rule.

---

# 177. PIN/Passphrase

Individual.

---

# 178. Role-Based HSM Permissions

Use if supported.

---

# 179. HSM Roles

Conceptually:

```text
security officer
crypto officer
auditor
```

---

# 180. Map To SIAR Roles

Explicit.

---

# 181. No HSM Superuser For Routine Signing

Hard rule.

---

# 182. Ceremony Approval

Out-of-band from HSM admin where possible.

---

# 183. Signing Proposal

```rust
pub struct SigningProposal {
    pub proposal_id: SigningProposalId,
    pub artifact_type: SignedArtifactType,
    pub digest: ContentDigest,
    pub required_policy: SigningPolicyId,
}
```

---

# 184. Proposal ID

Random/infrastructure-only.

---

# 185. Proposal Review

Human-readable summary.

---

# 186. Avoid Signing Opaque Digest Alone

Hard rule for human ceremony.

---

# 187. Need Semantic Context

Example:

```text
"Release 2.4.1, artifact digest X, security epoch Y"
```

---

# 188. Display Integrity

Trusted ceremony CLI.

---

# 189. QR/printed fingerprint

Optional.

---

# 190. Human Verification

Useful.

---

# 191. Signing Intent

Explicit.

---

# 192. Ceremony Transcript

Append-only.

---

# 193. Transcript Contains

```text
participants
role
proposal digest
public artifact hash
decision
signature
```

---

# 194. Transcript Excludes

```text
private key
PIN
recovery share
```

---

# 195. Hard rule.

---

# 196. Ceremony Transcript Type

```rust
pub struct CeremonyTranscript {
    pub ceremony: CeremonyId,
    pub proposal_digest: ContentDigest,
    pub approvals: Vec<CeremonyApproval>,
    pub resulting_artifacts: Vec<SignedArtifactRef>,
}
```

---

# 197. Transcript Signature

Signed by coordinator/witness or evidence key.

---

# 198. Auditability

Strong.

---

# 199. Privacy

No user data.

---

# 200. Key Usage Audit

HSM log + SIAR log.

---

# 201. Audit Record

```rust
pub struct HighAssuranceKeyUseRecord {
    pub key: KeyId,
    pub artifact_type: SignedArtifactType,
    pub digest: ContentDigest,
    pub time: Timestamp,
    pub approvals: ApprovalSummary,
}
```

---

# 202. No Signed Content Copy Needed

Digest enough.

---

# 203. Unexpected Key Use

Critical alert.

---

# 204. Use-Rate Limits

Can detect abuse.

---

# 205. Example

Root key used unexpectedly.

---

# 206. Immediate incident.

---

# 207. Key Use Policy

```rust
pub struct KeyUsePolicy {
    pub allowed_types: BTreeSet<SignedArtifactType>,
    pub max_uses_per_window: Option<u32>,
}
```

---

# 208. HSM Policy Should Enforce Where Possible

---

# 209. If Not

Application policy + audit.

---

# 210. Release-Key Compromise

Severe.

---

# 211. Response

```text
freeze releases
revoke signer
publish emergency trust update
rotate key
audit signed manifests
```

---

# 212. Governance-Key Compromise

More severe.

---

# 213. Response

Part 53 emergency governance.

---

# 214. Root Compromise

Catastrophic.

---

# 215. Need preplanned root-recovery/transition procedure.

---

# 216. No Improvised Root Recovery

Hard rule.

---

# 217. Provider Key Compromise

Scoped.

---

# 218. Revoke provider key.

---

# 219. Other tenants/providers unaffected.

---

# 220. Key Compromise Detection

Signals:

```text
unexpected HSM use
missing custodian token
audit mismatch
unauthorized signature
firmware compromise
```

---

# 221. Missing Hardware Token

Revoke operator credential.

---

# 222. Does Not Automatically Mean Key Compromise

But investigate.

---

# 223. HSM Theft

Physical compromise.

---

# 224. Security policy.

---

# 225. Tamper Evidence

Useful.

---

# 226. Remote HSM Facility

Physical security contract.

---

# 227. Operator Separation

Build engineer cannot alone release.

---

# 228. Release approver cannot alone alter governance root.

---

# 229. Security officer cannot alone sign arbitrary content.

---

# 230. Hard rules.

---

# 231. Signing Automation

Levels.

---

# 232. Automation Class

```rust
pub enum SigningAutomationClass {
    ManualCeremony,
    ApprovedAutomated,
    FullyAutomatedLowRisk,
}
```

---

# 233. Root

ManualCeremony.

---

# 234. Release

ApprovedAutomated or Manual depending maturity.

---

# 235. Routine short-lived provider token

Can be automated.

---

# 236. Automation Does Not Remove Policy

Hard rule.

---

# 237. Policy Engine

Always validates.

---

# 238. HSM API Availability

Signing can be queued.

---

# 239. No Fallback To Software Key

Hard rule.

---

# 240. If HSM Unavailable

Delay signing.

---

# 241. Emergency

Use approved backup HSM path.

---

# 242. Approval Threshold unchanged.

---

# 243. HSM Monitoring

Safe metrics:

```text
availability
queue depth
error rate
key use count
firmware state
```

---

# 244. Forbidden Metrics

No:

```text
user message-related signing
user identity
contact graph
```

---

# 245. HSM SLO

High availability for online signers.

---

# 246. Root HSM

Availability less important than custody safety.

---

# 247. Ceremony SLO

Not latency-oriented.

---

# 248. Integrity oriented.

---

# 249. HSM Backup Site

Separate.

---

# 250. Disaster Recovery

Part 70.

---

# 251. Scenario

Primary HSM destroyed.

---

# 252. Validate recovery with test key.

---

# 253. Root Recovery Runbook

Offline.

---

# 254. Signed copy.

---

# 255. No Sole SaaS storage.

---

# 256. Ceremony Documentation

Part 65.

---

# 257. Versioned.

---

# 258. Ceremony Software

Part 62 reproducible build.

---

# 259. Artifact signed.

---

# 260. Ceremony Workstation Attestation

Part 71.

---

# 261. Strongly recommended.

---

# 262. No Ceremony On Unknown Binary

Hard rule.

---

# 263. Ceremony Environment Manifest

```rust
pub struct CeremonyEnvironmentManifest {
    pub os_digest: ContentDigest,
    pub tool_digest: ContentDigest,
    pub hsm_firmware: Option<String>,
}
```

---

# 264. Record public metadata only.

---

# 265. HSM Firmware Version

Internal restricted.

---

# 266. Ceremony Reproducibility

Procedure reproducible, not secret material.

---

# 267. Root Key Generation

Random by HSM.

---

# 268. Public key should differ each run.

---

# 269. Deterministic ceremony not desired for secret generation.

---

# 270. Entropy Validation

Use vendor/HSM assurance.

---

# 271. No Custom Entropy Test

Hard rule.

---

# 272. Multi-Signature Governance

Could use independent keys.

---

# 273. Example

```text
3-of-5 authority signatures
```

---

# 274. Benefit

No single HSM compromise grants policy authority.

---

# 275. Authority Diversity

Different operators/sites.

---

# 276. No All Authorities In One Cloud

Hard rule.

---

# 277. Federation Trust

Bilateral keys.

---

# 278. HSM custody per domain if high value.

---

# 279. No Global Federation Signing Master Key

Hard rule.

---

# 280. Tenant Root Keys

Part 69.

---

# 281. Hosted enterprise

Could use tenant-dedicated HSM key.

---

# 282. High-security tenant

Dedicated partition/device.

---

# 283. Shared SaaS

Per-tenant key handles.

---

# 284. No key reuse across tenants.

---

# 285. Key Namespace

HSM labels include opaque tenant/service IDs.

---

# 286. No human-sensitive naming required.

---

# 287. HSM Multi-Tenancy

Must use partition/isolation capability.

---

# 288. If insufficient isolation

Use dedicated HSM.

---

# 289. Key Migration Across Tenants

Forbidden.

---

# 290. Hard rule.

---

# 291. Cryptographic Agility

Part 66.

---

# 292. HSM algorithm support may lag.

---

# 293. PQ challenge.

---

# 294. PQ HSM Strategy

Phase:

```text
software test
→ approved software/secure enclave
→ HSM once mature
```

---

# 295. Root PQ Migration

Do not force premature vendor lock-in.

---

# 296. Hybrid Root

Could use:

```text
classical HSM signature
+
PQ signature from separate approved signer
```

---

# 297. Independent custody.

---

# 298. No Novel Hybrid HSM Scheme

Hard rule.

---

# 299. Secure Element Android

Can hold device-specific managed profile key.

---

# 300. But

No global user attestation/identity.

---

# 301. C ABI/HSM FFI

Vendor libraries may require native API.

---

# 302. Isolate in small crate/process.

---

# 303. No HSM FFI Across Entire Codebase

Hard rule.

---

# 304. HSM Agent

Preferred.

---

# 305. Architecture

```text
SIAR signer
→ local authenticated IPC
→ HSM agent
→ vendor PKCS#11/API
```

---

# 306. HSM Agent Process

Minimal privileges.

---

# 307. PKCS#11

Common adapter.

---

# 308. Vendor API

Alternative.

---

# 309. Backend Capability Model

```rust
pub struct HsmCapabilities {
    pub algorithms: BTreeSet<SignatureAlgorithmId>,
    pub wrapped_export: bool,
    pub audit_log: bool,
    pub multi_operator: bool,
}
```

---

# 310. Capability Validation

Before use.

---

# 311. No Assume Every HSM Equal

Hard rule.

---

# 312. Signing Queue

Durable proposal queue.

---

# 313. Proposal immutable.

---

# 314. Approval state separate.

---

# 315. Sign only exact approved digest.

---

# 316. Hard rule.

---

# 317. Approval Race

Need atomic finalize.

---

# 318. Signing State

```rust
pub enum SigningProposalState {
    Draft,
    PendingApproval,
    Approved,
    Signing,
    Signed,
    Rejected,
    Expired,
}
```

---

# 319. Expiry

Proposals expire.

---

# 320. No stale approval reuse.

---

# 321. Approval Record

```rust
pub struct CeremonyApproval {
    pub custodian: CustodianId,
    pub proposal: SigningProposalId,
    pub decision: ApprovalDecision,
    pub timestamp: Timestamp,
}
```

---

# 322. Approval Signatures

Cryptographically authenticate.

---

# 323. No Shared Approval Credential

Hard rule.

---

# 324. Signer Policy

Checks proposal state.

---

# 325. No Direct HSM Sign CLI For Production Key

Hard rule outside ceremony tool.

---

# 326. Emergency Revocation

May be faster.

---

# 327. Still multi-party.

---

# 328. Predefined emergency authority set.

---

# 329. Emergency Key

Separate from normal key if needed.

---

# 330. Emergency Key Scope

Revocation only.

---

# 331. Cannot authorize arbitrary new trust.

---

# 332. Hard rule.

---

# 333. Dead-Man Risk

Avoid emergency key held by one person.

---

# 334. Ceremony Evidence Store

Append-only.

---

# 335. Storage

Encrypted/restricted.

---

# 336. Evidence Digest

Can be published internally.

---

# 337. Transparency Log

Could record signed artifact event.

---

# 338. No Private Custodian Details Public

Hard rule.

---

# 339. Key Custody Inventory

Restricted.

---

# 340. Public Inventory

Only public keys/fingerprints/status.

---

# 341. Internal Inventory

Includes:

```text
HSM partition
custody role
rotation date
```

---

# 342. No PIN/recovery share.

---

# 343. Key Status

```rust
pub enum HighAssuranceKeyState {
    Pending,
    Active,
    Rotating,
    Retired,
    Revoked,
    Destroyed,
}
```

---

# 344. Status Registry

Signed.

---

# 345. Anti-Rollback.

---

# 346. Testing

Need HSM/ceremony testkit.

---

# 347. Test HSM

Software emulator allowed.

---

# 348. Production HSM

Never use emulator.

---

# 349. Test Scenarios

```text
one custodian absent
wrong approval
expired proposal
HSM unavailable
wrapped backup restore
rotation
compromise
```

---

# 350. Threshold Test

Below required approvals must fail.

---

# 351. Role Separation Test

Same person cannot satisfy two distinct-role requirements if policy forbids.

---

# 352. Proposal Integrity Test

Changed digest invalidates approvals.

---

# 353. HSM Outage Test

No software fallback.

---

# 354. Backup Recovery Test

Test key recoverable.

---

# 355. Rotation Test

New key trusted, old retired.

---

# 356. Root Rollover Test

Anti-rollback preserved.

---

# 357. Emergency Revocation Test

Scope limited.

---

# 358. Audit Test

Every key use recorded.

---

# 359. Unauthorized Sign Test

Rejected.

---

# 360. Firmware Change Test

Requires revalidation.

---

# 361. Multi-Tenant HSM Test

Tenant A key cannot be used in B context.

---

# 362. Fuzzing

Fuzz:

```text
signing proposal
approval record
ceremony manifest
HSM capability response
```

---

# 363. Property Tests

Properties:

```text
signed digest equals approved digest
approval threshold cannot be bypassed
retired/revoked key cannot sign valid new artifact
tenant key handle cannot cross tenant boundary
```

---

# 364. Formal Verification Targets

Strong candidates:

```text
approval state machine
threshold policy
proposal immutability
emergency revocation scope
```

---

# 365. Kani Candidate

approval/threshold arithmetic.

---

# 366. TLA+ Candidate

multi-custodian ceremony with failure/retry.

---

# 367. Loom Candidate

concurrent approvals + signing finalize.

---

# 368. Performance

Not main goal.

---

# 369. Signing Throughput

Low for root/governance.

---

# 370. Release Signing

Modest.

---

# 371. Provider Signing

May be higher.

---

# 372. Batch Carefully.

---

# 373. No Blind Bulk Sign.

---

# 374. Crate Layout

Recommended:

```text
crates/
├── siar-hsm-core/
├── siar-hsm-agent/
├── siar-key-custody/
├── siar-signing-policy/
├── siar-ceremony-core/
├── siar-ceremony-cli/
├── siar-key-recovery/
├── siar-key-transparency/
├── siar-high-assurance-audit/
└── siar-hsm-testkit/
```

---

# 375. `siar-hsm-core`

Owns:

```text
HSM handles
capabilities
errors
```

---

# 376. `siar-hsm-agent`

Vendor/PKCS#11 isolation.

---

# 377. `siar-key-custody`

Ownership/custodian/assurance policy.

---

# 378. `siar-signing-policy`

Approval/scope/use rules.

---

# 379. `siar-ceremony-core`

Ceremony state/manifests/transcripts.

---

# 380. `siar-ceremony-cli`

Offline high-assurance operator tool.

---

# 381. `siar-key-recovery`

Wrapped backup/share recovery.

---

# 382. `siar-key-transparency`

Public key/status/transparency records.

---

# 383. `siar-high-assurance-audit`

Key-use evidence.

---

# 384. `siar-hsm-testkit`

HSM emulator/ceremony simulation.

---

# 385. Error Taxonomy

```rust
pub enum HsmError {
    DeviceUnavailable,
    AuthenticationFailed,
    KeyNotFound,
    OperationDenied,
    CapabilityUnsupported,
    FirmwareUntrusted,
    Internal,
}

pub enum SigningError {
    ProposalInvalid,
    ApprovalInsufficient,
    ApprovalExpired,
    RoleConflict,
    KeyRevoked,
    ScopeViolation,
    DigestMismatch,
    HsmFailure,
    Internal,
}
```

---

# 386. Security & Operational Invariants

Mandatory:

```text
1. RootAuthority private keys never reside on general-purpose production servers.
2. HighValue/RootAuthority keys are non-exportable or wrapped-export-only; plaintext export is prohibited.
3. No single person can unilaterally perform critical root/governance/release signing operations.
4. Signing authority is domain-separated; release, governance, federation, provider, and tenant keys are not interchangeable.
5. HSM unavailability never triggers silent fallback to software signing with a weaker key.
6. Signed digests must exactly match previously approved immutable proposals.
7. Critical key recovery is preplanned, periodically tested with non-production roots, and does not depend on one custodian or one location.
8. Ceremony transcripts contain approvals and public evidence but never PINs, private keys, or recovery shares.
9. Tenant/provider/key domains remain cryptographically separated inside shared HSM infrastructure.
10. Emergency signing/revocation keys are narrowly scoped and cannot silently create broader trust.
11. HSM/secure-element vendor capability differences are explicit; no platform is assumed equivalent without policy validation.
12. Root/key ceremonies use verified tools/environments and remain reproducible at the procedural level.
```

---

# 387. Initial Production Scope

Implement first:

```text
HSM backend abstraction
opaque key handles
PKCS#11/vendor agent process
key-assurance classes
key-custody registry
dual-control approval policy
release-signing proposal workflow
governance-signing workflow
offline ceremony CLI
signed ceremony transcript
key-use audit
wrapped backup/recovery support
root/release/provider key separation
test HSM emulator
rotation/revocation ceremonies
```

Then add:

```text
multi-HSM quorum signing
independent geographic custody
hardware-backed tenant root isolation
PQ-capable high-assurance signers
advanced ceremony transparency
formal approval-state verification
```

---

# 388. Definition of Done

Part 72 is complete when:

- high-value/root keys are assigned explicit assurance/custody classes
- HSM/secure-element/TPM responsibilities are clearly separated
- private key material does not escape protected custody by default
- key signing is proposal- and policy-driven
- dual-control/multi-party approval exists for critical operations
- release/governance/provider/federation/tenant keys are domain-separated
- ceremony roles, manifests, environments, transcripts, and evidence are defined
- HSM outage cannot trigger software fallback
- backup/recovery/migration procedures do not create weaker plaintext copies
- emergency revocation authority is narrowly scoped
- HSM vendor abstraction and capability validation are explicit
- rotation/root rollover/key destruction are ceremony-controlled
- test HSM, fuzz/property/formal verification are specified

---

# 389. Final Architecture

```text
                    HIGH-VALUE SIGNING REQUEST
                               │
                               ▼
                       SIGNING PROPOSAL
                               │
                               ▼
                      MULTI-PARTY APPROVAL
                               │
                               ▼
                       CEREMONY POLICY
                               │
                               ▼
                     HSM / SECURE ELEMENT
                               │
                               ▼
                       SIGNED ARTIFACT
                               │
                               ▼
                   AUDIT / TRANSPARENCY LOG
```

High-assurance cryptographic operations model:

```text
domain-separated keys
+
hardware custody
+
dual control
+
immutable signing proposals
+
ceremony evidence
+
tested recovery
+
no software fallback
```

not:

```text
one admin account can access a private key file and sign whatever it wants
```

---

# 390. Final Principle

High-value cryptographic authority must be difficult to misuse even by legitimate operators.

The correct model is:

```text
separate authority
+
hardware-backed custody
+
multi-party approval
+
narrow signing interfaces
+
auditable ceremonies
+
tested recovery
```

This architecture gives SIAR a high-assurance custody and signing foundation for trust roots, governance, releases, federation, providers, tenants, and other critical cryptographic authorities while preserving the anonymity and least-authority principles established across Parts 34–71.
