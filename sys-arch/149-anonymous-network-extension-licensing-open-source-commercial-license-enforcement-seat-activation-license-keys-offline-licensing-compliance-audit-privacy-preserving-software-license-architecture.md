# Core System Architecture Part 149 — Anonymous Network Extension Licensing, Open-Source/Commercial License Enforcement, Seat Activation, License Keys, Offline Licensing, Compliance, Audit & Privacy-Preserving Software License Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 149  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 73, 94, 103, 133, 142, 147–148

**Primary purpose:** define SIAR's extension software-license architecture for open-source and commercial licensing, license identities, activation, seats, device binding, floating/concurrent access, offline proofs, license keys, compliance, audit, revocation, renewal, enterprise deployment, source-license obligations, and privacy-preserving enforcement.

---

# 1. Purpose

Software licensing and marketplace billing are related but different.

A user may:

```text
purchase a paid extension
receive an enterprise seat
install an open-source extension
activate a commercial binary
run an offline licensed copy
```

The governing principle is:

> **SIAR software licensing must describe and enforce legal/commercial usage rights without turning license checks into device fingerprinting, user tracking, hidden telemetry, or a substitute for security authorization.**

---

# 2. Architectural Position

```text
                 SOFTWARE PACKAGE
                        │
                        ▼
                  LICENSE POLICY
                        │
             ┌──────────┼──────────┐
             │          │          │
         OPEN-SOURCE  COMMERCIAL  ENTERPRISE
             │          │          │
             └──────────┼──────────┘
                        ▼
                   LICENSE GRANT
                        │
                        ▼
                 ACTIVATION STATE
                        │
             ┌──────────┼──────────┐
             │          │          │
           USER       DEVICE      SEAT
             │          │          │
             └──────────┼──────────┘
                        ▼
               LOCAL LICENSE PROOF
                        │
                        ▼
                  EXTENSION ACCESS
```

---

# 3. Core Separation

Keep distinct:

```text
software source license
commercial purchase
license grant
activation
seat
device binding
entitlement
runtime permission
marketplace governance
```

---

# 4. Non-Goals

Part 149 does not create:

```text
DRM that breaks offline ownership
covert device fingerprinting
license checks as runtime authorization
publisher access to machine identifiers
telemetry-based license enforcement
```

---

# 5. License Identity

```rust
pub struct SoftwareLicenseId(pub [u8; 16]);
```

---

# 6. License Version

```rust
pub struct SoftwareLicenseVersion(pub u32);
```

---

# 7. License Family

```rust
pub enum SoftwareLicenseFamily {
    OpenSource,
    SourceAvailable,
    Commercial,
    Enterprise,
    Evaluation,
}
```

---

# 8. Hard Rule

License family is explicit.

---

# 9. Source License

```rust
pub struct SourceLicenseDescriptor {
    pub license_id: SoftwareLicenseId,
    pub spdx_id: Option<String>,
    pub text_digest: Digest,
    pub family: SoftwareLicenseFamily,
}
```

---

# 10. Hard Rule

SPDX identifier does not replace exact license text where custom terms exist.

---

# 11. Commercial License

```rust
pub struct CommercialLicenseDescriptor {
    pub license_id: SoftwareLicenseId,
    pub product: ExtensionCommercialProductId,
    pub activation_model: ActivationModel,
    pub terms_digest: Digest,
}
```

---

# 12. Hard Rule

Commercial terms versioned and immutable per issued grant.

---

# 13. License Grant

```rust
pub struct ExtensionLicenseGrant {
    pub grant_id: ExtensionLicenseGrantId,
    pub extension: ExtensionId,
    pub license: SoftwareLicenseId,
    pub scope: ExtensionLicenseScope,
    pub state: ExtensionLicenseState,
}
```

---

# 14. License Scope

```rust
pub enum ExtensionLicenseScope {
    User,
    Device,
    Seat,
    Tenant,
    Organization,
    Site,
}
```

---

# 15. Hard Rule

Scope explicit.

---

# 16. License State

```rust
pub enum ExtensionLicenseState {
    Active,
    Grace,
    Suspended,
    Expired,
    Revoked,
}
```

---

# 17. Hard Rule

License state is commercial/legal state, not security trust state.

---

# 18. Entitlement Separation

Part 148 entitlement may prove paid access.

Software license determines usage rights.

---

# 19. Hard Rule

Paid entitlement and software license are not the same object.

---

# 20. Runtime Permission Separation

License possession never grants:

```text
files
network
camera
microphone
admin access
message access
```

---

# 21. Hard Rule

License enforcement cannot bypass Part 135 permissions.

---

# 22. Open-Source Licensing

Supported directly.

---

# 23. Open-Source Package Metadata

```rust
pub struct OpenSourceExtensionLicense {
    pub extension: ExtensionId,
    pub license: SourceLicenseDescriptor,
    pub source_url: Option<CanonicalSourceRef>,
}
```

---

# 24. Hard Rule

Marketplace may distribute open-source extension commercially or freely.

---

# 25. Paid Distribution ≠ Proprietary License

Hard rule.

---

# 26. Dual Licensing

Supported.

---

# 27. Dual License Model

```rust
pub struct DualLicensePolicy {
    pub open_source_license: SoftwareLicenseId,
    pub commercial_license: SoftwareLicenseId,
}
```

---

# 28. Hard Rule

User/publisher must know which license applies to a specific package/build/distribution.

---

# 29. Copyleft Obligations

Marketplace should track factual obligations such as:

```text
source availability
license notices
modification disclosure
corresponding source
```

---

# 30. Hard Rule

Compliance evidence is factual, not a legal verdict.

---

# 31. License Notice Bundle

```rust
pub struct LicenseNoticeBundle {
    pub notices: Vec<ThirdPartyLicenseNotice>,
    pub digest: Digest,
}
```

---

# 32. Hard Rule

Third-party notices bound to exact package/SBOM.

---

# 33. Source Offer

If required by license, package metadata can include source reference.

---

# 34. Hard Rule

Source fulfillment policy independent from runtime license activation.

---

# 35. Proprietary Commercial License

May restrict:

```text
number of seats
number of devices
organization scope
subscription duration
redistribution
```

---

# 36. Hard Rule

Enforcement must match published terms.

---

# 37. Activation Model

```rust
pub enum ActivationModel {
    NoActivation,
    UserBound,
    DeviceBound,
    SeatBased,
    Floating,
    TenantWide,
    OfflineSignedGrant,
}
```

---

# 38. Hard Rule

Activation model explicit.

---

# 39. License Activation Identity

```rust
pub struct ExtensionLicenseActivationId(pub [u8; 16]);
```

---

# 40. Activation State

```rust
pub enum ExtensionLicenseActivationState {
    Pending,
    Active,
    Grace,
    Deactivated,
    Suspended,
    Revoked,
}
```

---

# 41. Activation Record

```rust
pub struct ExtensionLicenseActivation {
    pub activation_id: ExtensionLicenseActivationId,
    pub grant: ExtensionLicenseGrantId,
    pub binding: LicenseBinding,
    pub state: ExtensionLicenseActivationState,
}
```

---

# 42. Binding

```rust
pub enum LicenseBinding {
    UserAccount,
    Device(LicenseDeviceId),
    Seat(ExtensionSeatId),
    Tenant(TenantId),
    Site(ExtensionSiteLicenseId),
}
```

---

# 43. Hard Rule

Binding uses licensing-domain identifiers only.

---

# 44. License Device Identity

```rust
pub struct LicenseDeviceId(pub [u8; 16]);
```

---

# 45. Hard Rule

License device ID is not hardware fingerprint.

---

# 46. Device Enrollment

Generate random license device identity and bind it to authenticated device membership.

---

# 47. Hard Rule

No serial-number/IMEI/MAC-address based licensing by default.

---

# 48. Hardware Binding

If enterprise needs stronger binding, use privacy-preserving attestation where supported.

---

# 49. Hard Rule

Raw hardware identifiers remain local/hashed/attested rather than exposed to publisher.

---

# 50. TPM/Secure Element

Optional high-assurance binding.

---

# 51. Hard Rule

Attestation scope only proves licensed device class/state, not user identity/activity.

---

# 52. Seat Licensing

```rust
pub struct ExtensionSeatId(pub [u8; 16]);
```

---

# 53. Seat Record

```rust
pub struct ExtensionSeat {
    pub seat_id: ExtensionSeatId,
    pub license: ExtensionLicenseGrantId,
    pub state: ExtensionSeatState,
}
```

---

# 54. Seat State

```rust
pub enum ExtensionSeatState {
    Available,
    Assigned,
    Reserved,
    Revoked,
}
```

---

# 55. Hard Rule

Seat assignment does not require employee productivity monitoring.

---

# 56. Seat Assignment

```rust
pub struct ExtensionSeatAssignment {
    pub seat: ExtensionSeatId,
    pub subject: SeatSubject,
}
```

---

# 57. Seat Subject

```rust
pub enum SeatSubject {
    UserAccount,
    Device(LicenseDeviceId),
}
```

---

# 58. Hard Rule

Personal/private messaging context never exposed to tenant seat admin.

---

# 59. Seat Reassignment

Allowed according to license terms.

---

# 60. Hard Rule

Reassignment history bounded and used for licensing/accounting only.

---

# 61. Floating Licensing

Concurrent usage model.

---

# 62. Lease Identity

```rust
pub struct FloatingLicenseLeaseId(pub [u8; 16]);
```

---

# 63. Lease

```rust
pub struct FloatingLicenseLease {
    pub lease_id: FloatingLicenseLeaseId,
    pub grant: ExtensionLicenseGrantId,
    pub device: LicenseDeviceId,
    pub expires_at: Timestamp,
}
```

---

# 64. Hard Rule

Lease proves occupancy, not user activity.

---

# 65. Lease Renewal

Periodic.

---

# 66. Hard Rule

Renewal interval must not become high-frequency tracking beacon.

---

# 67. Grace Lease

Short offline lease extension.

---

# 68. Hard Rule

Temporary network loss does not immediately terminate work.

---

# 69. Concurrent Use Count

Server tracks active leases only.

---

# 70. Hard Rule

No detailed usage timeline required.

---

# 71. Site License

For fixed organizational deployment.

---

# 72. Site Identity

```rust
pub struct ExtensionSiteLicenseId(pub [u8; 16]);
```

---

# 73. Hard Rule

Site license does not imply blanket access to tenant user data.

---

# 74. Tenant-Wide License

Commercial scope for one tenant.

---

# 75. Hard Rule

Tenant-wide commercial rights still respect per-user/runtime permissions.

---

# 76. Offline Licensing

First-class.

---

# 77. Offline License Proof

```rust
pub struct SignedOfflineLicense {
    pub grant: ExtensionLicenseGrant,
    pub offline_until: Timestamp,
    pub issuer: LicenseIssuerId,
    pub signature: Signature,
}
```

---

# 78. Hard Rule

Offline proof contains no payment credentials.

---

# 79. Offline Duration

Policy-defined.

---

# 80. Hard Rule

Air-gapped enterprise licenses can have long validity if contract permits.

---

# 81. Offline Verification

Local signature verification.

---

# 82. Hard Rule

No recurring online check required inside valid offline window.

---

# 83. Offline Expiry

Possible outcomes:

```text
grace
read-only
renewal required
disabled premium features
```

---

# 84. Hard Rule

No silent data deletion.

---

# 85. Offline Renewal

Through signed renewal bundle.

---

# 86. Hard Rule

Bundle anti-rollback protected.

---

# 87. License Epoch

```rust
pub struct SoftwareLicenseEpoch(pub u64);
```

---

# 88. Hard Rule

Newer revocation/renewal state dominates older offline proof.

---

# 89. License Keys

Human-entered license keys may be supported.

---

# 90. License Key

```rust
pub struct ExtensionLicenseKey(String);
```

---

# 91. Hard Rule

Plain license key is not direct authorization object.

---

# 92. Key Exchange

License key is redeemed once to obtain signed license grant/activation.

---

# 93. Hard Rule

Runtime uses signed grant, not continuously stored plaintext key.

---

# 94. License Key Storage

If retained, secure secret storage.

---

# 95. Hard Rule

Never in logs/config/plaintext sync.

---

# 96. Key Entropy

High.

---

# 97. Hard Rule

No guessable sequential keys.

---

# 98. Redemption Rate Limit

Yes.

---

# 99. Hard Rule

Rate limiting should not fingerprint users.

---

# 100. License Key Replay

Prevented through redemption state.

---

# 101. Hard Rule

One-time key cannot activate beyond allowed scope.

---

# 102. Multi-Activation Key

If offered, explicit activation count.

---

# 103. Hard Rule

Activation count is licensing metadata, not user activity.

---

# 104. Activation Limit

```rust
pub struct ActivationLimit {
    pub max_active_devices: u32,
}
```

---

# 105. Hard Rule

Limit enforcement uses active bindings only.

---

# 106. Device Deactivation

User/admin can release device activation.

---

# 107. Hard Rule

Old device may retain local data but loses licensed premium access after convergence.

---

# 108. Lost Device

License may be reissued after revocation.

---

# 109. Hard Rule

Lost-device recovery does not restore stale runtime tokens.

---

# 110. Device Replacement

Allow transfer according to policy.

---

# 111. Hard Rule

Transfer requires authenticated licensing context.

---

# 112. License Renewal

Commercial license may renew through Part 148 entitlement.

---

# 113. Hard Rule

Renewal modifies commercial validity only.

---

# 114. Perpetual License

Possible.

---

# 115. Hard Rule

Perpetual usage rights do not guarantee perpetual updates/support/platform compatibility.

---

# 116. Maintenance License

Optional separate right for updates/support.

---

# 117. Maintenance Entitlement

```rust
pub struct MaintenanceEntitlement {
    pub extension: ExtensionId,
    pub valid_until: Timestamp,
}
```

---

# 118. Hard Rule

Maintenance expiry does not revoke already licensed perpetual package unless terms say so.

---

# 119. Version-Capped License

Possible.

---

# 120. Hard Rule

License can state allowed major version range explicitly.

---

# 121. Update Eligibility

Part 143 checks commercial license compatibility where needed.

---

# 122. Hard Rule

Security update needed to preserve hard security floor may have special policy.

---

# 123. Upgrade Purchase

Optional.

---

# 124. Hard Rule

Old licensed data remains usable/exportable according to original rights.

---

# 125. License Revocation

Reasons:

```text
refund
chargeback
fraudulent key
contract termination
publisher error
security/compromise of license issuer
```

---

# 126. Hard Rule

License revocation is distinct from package security revocation.

---

# 127. Revocation Record

```rust
pub struct ExtensionLicenseRevocation {
    pub grant: ExtensionLicenseGrantId,
    pub epoch: SoftwareLicenseEpoch,
    pub reason: LicenseRevocationReason,
}
```

---

# 128. Hard Rule

Revocation reason scoped/minimized.

---

# 129. Revocation List

Signed.

---

# 130. Hard Rule

Old offline proof cannot override newer revocation epoch.

---

# 131. Grace After Revocation

Usually none for confirmed refund/fraud, but policy-defined where contract requires.

---

# 132. Hard Rule

Still preserve/export user data.

---

# 133. License Suspension

Temporary.

---

# 134. Hard Rule

Suspension can be reversed without creating new license identity.

---

# 135. License Restore

Only after authorized reinstatement.

---

# 136. Hard Rule

Publisher cannot arbitrarily restore platform-blocked license state if marketplace/accounting policy forbids.

---

# 137. Open-Source Enforcement

Open-source licenses often concern redistribution/modification/source obligations rather than runtime activation.

---

# 138. Hard Rule

Do not impose artificial license-key activation on open-source rights unless a separate commercial service is being licensed.

---

# 139. Source-Available License

Custom terms may restrict production use/redistribution.

---

# 140. Hard Rule

Marketplace labels such licenses separately from recognized open-source licenses.

---

# 141. Commercial License Text

Versioned and digest-bound.

---

# 142. Hard Rule

User can inspect applicable terms before purchase/activation.

---

# 143. Acceptance Record

```rust
pub struct ExtensionLicenseAcceptance {
    pub license: SoftwareLicenseId,
    pub version: SoftwareLicenseVersion,
    pub subject: LicenseAcceptanceSubject,
    pub accepted_at: Timestamp,
}
```

---

# 144. Hard Rule

Acceptance record contains no unrelated behavioral data.

---

# 145. Acceptance Scope

```rust
pub enum LicenseAcceptanceSubject {
    UserAccount,
    Tenant,
    Organization,
}
```

---

# 146. Hard Rule

Tenant acceptance does not automatically bind unrelated personal context.

---

# 147. Compliance Registry

Track factual obligations.

---

# 148. Compliance Obligation

```rust
pub struct LicenseComplianceObligation {
    pub obligation_id: LicenseComplianceObligationId,
    pub package: ExtensionPackageId,
    pub class: LicenseComplianceClass,
    pub state: ComplianceState,
}
```

---

# 149. Compliance Class

```rust
pub enum LicenseComplianceClass {
    NoticeRequired,
    SourceOfferRequired,
    AttributionRequired,
    RedistributionRestriction,
    ModificationDisclosure,
    CommercialUseRestriction,
}
```

---

# 150. Hard Rule

Do not infer legal conclusion beyond encoded factual policy.

---

# 151. Compliance State

```rust
pub enum ComplianceState {
    Satisfied,
    Unsatisfied,
    Unknown,
    NotApplicable,
}
```

---

# 152. Hard Rule

Unknown is not Satisfied.

---

# 153. License Compatibility Analysis

Part 142 dependency graph can detect potential license conflicts.

---

# 154. Hard Rule

Tool provides factual compatibility warnings, not definitive legal advice.

---

# 155. SBOM Integration

Every dependency has license metadata.

---

# 156. Hard Rule

License scan results bind exact SBOM/package digest.

---

# 157. Third-Party Notices

Generated deterministically from SBOM.

---

# 158. Hard Rule

No omission due to transitive depth.

---

# 159. Compliance Gate

Marketplace release may block if required license metadata missing.

---

# 160. Hard Rule

Compliance gate separate from security certification.

---

# 161. Audit

License audit should answer:

```text
which license applied
which version
which grant
which activation
which seat
which revocation epoch
```

---

# 162. Hard Rule

Audit does not need user activity logs.

---

# 163. License Audit Record

```rust
pub struct LicenseAuditRecord {
    pub event: LicenseAuditEvent,
    pub grant: ExtensionLicenseGrantId,
    pub timestamp: Timestamp,
}
```

---

# 164. Audit Event

```rust
pub enum LicenseAuditEvent {
    GrantIssued,
    Activated,
    Deactivated,
    SeatAssigned,
    SeatReleased,
    Renewed,
    Suspended,
    Revoked,
}
```

---

# 165. Hard Rule

Audit events reflect licensing state transitions only.

---

# 166. No Usage Surveillance

Hard rule.

---

# 167. Publisher Audit View

Publisher may see:

```text
active license count
seat count
renewals
revocations
```

---

# 168. Hard Rule

No device/user drill-down unless contract/support requires and policy allows.

---

# 169. Enterprise Audit

Tenant admin may see seat allocation.

---

# 170. Hard Rule

No private extension usage timeline.

---

# 171. Compliance Export

Can export:

```text
license grants
seat assignments
notice bundle
SBOM license list
```

---

# 172. Hard Rule

Export omits secrets/license-key plaintext.

---

# 173. License Server

Optional for enterprise floating licenses.

---

# 174. Hard Rule

Server can run on-premise/offline.

---

# 175. Local License Server

Architecture:

```text
signed enterprise grant
→ local license server
→ short-lived floating leases
→ local clients
```

---

# 176. Hard Rule

Local server never needs messaging content.

---

# 177. License Server Trust

Bound to tenant/site.

---

# 178. Hard Rule

Cannot issue licenses outside signed capacity.

---

# 179. Offline License Server

Supports air-gapped deployment.

---

# 180. Hard Rule

Root grant has explicit capacity/expiry.

---

# 181. Seat Capacity

```rust
pub struct LicensedCapacity(pub u32);
```

---

# 182. Hard Rule

Capacity arithmetic deterministic.

---

# 183. Lease Clock

Use monotonic time where possible.

---

# 184. Hard Rule

Wall-clock rollback cannot extend leases indefinitely.

---

# 185. Time Validation

Combine signed epoch/last-seen monotonic state.

---

# 186. Hard Rule

No fragile online NTP dependency for every check.

---

# 187. Clock Tamper

May trigger conservative grace/revalidation.

---

# 188. Hard Rule

Do not wipe data on time uncertainty.

---

# 189. License Restore From Backup

Restore license intent/receipt, then revalidate current grant.

---

# 190. Hard Rule

Do not restore stale activation/revoked grant blindly.

---

# 191. State Sync

Part 141 may sync license intent/seat metadata where appropriate.

---

# 192. Hard Rule

Active device-bound proof remains device-local.

---

# 193. Account Recovery

Restores license ownership context separately from runtime permission.

---

# 194. Hard Rule

Fresh device activation required where binding demands.

---

# 195. Revoked Device

Cannot continue using device-bound activation after convergence.

---

# 196. Hard Rule

License revocation and device membership revocation are independent.

---

# 197. License Privacy Boundary

Licensing data includes potentially identifying commercial/device information.

---

# 198. Hard Rule

Do not join licensing data with:

```text
messages
contacts
social graph
telemetry clickstream
search history
```

---

# 199. Publisher Boundary

Publisher sees aggregate licensing state where possible.

---

# 200. Hard Rule

Publisher cannot query raw license-device fingerprints.

---

# 201. Marketplace Boundary

License state may affect:

```text
download eligibility
activation
paid feature access
```

not:

```text
trust ranking
certification
security exceptions
```

---

# 202. Hard Rule

No pay-to-trust path.

---

# 203. License Telemetry

Allowed aggregate:

```text
activation success rate
lease failure rate
offline renewal failure
seat utilization aggregate
```

---

# 204. Hard Rule

No detailed user/device activity timeline.

---

# 205. Seat Utilization

For enterprise planning, aggregate only.

---

# 206. Hard Rule

No productivity inference.

---

# 207. License Metrics Retention

Bounded.

---

# 208. Hard Rule

Do not retain detailed lease history indefinitely.

---

# 209. License Incident

Examples:

```text
issuer key compromise
activation replay
license server compromise
counterfeit grant
```

---

# 210. Hard Rule

License incident response uses Part 146 but remains licensing-scoped.

---

# 211. Issuer Key Compromise

Rotate issuer, revoke affected proofs, reissue grants.

---

# 212. Hard Rule

Reissue does not change runtime permissions.

---

# 213. Counterfeit Key

Invalidate key/grant only.

---

# 214. Hard Rule

Do not punish unrelated legitimate licenses.

---

# 215. License Abuse Detection

Based on:

```text
concurrent activations beyond terms
replay
forged signature
invalid seat lease
```

---

# 216. Hard Rule

No behavioral profiling.

---

# 217. Soft vs Hard Enforcement

```rust
pub enum LicenseEnforcementMode {
    Advisory,
    Graceful,
    Strict,
}
```

---

# 218. Hard Rule

Mode explicit in license terms.

---

# 219. Advisory

Warn only.

---

# 220. Graceful

Limit premium features/read-only.

---

# 221. Strict

Disable paid functionality.

---

# 222. Hard Rule

Strict still preserves user data/export.

---

# 223. Enforcement UI

Host-owned.

---

# 224. Hard Rule

Extension cannot spoof payment/license status.

---

# 225. License Status View

Shows:

```text
license type
validity
seat/device scope
offline expiry
renewal path
```

---

# 226. Hard Rule

No hidden enforcement state.

---

# 227. Developer API

Extension may query abstract license state.

---

# 228. License Query

```rust
pub trait ExtensionLicenseService {
    fn status(
        &self,
        extension: ExtensionId,
    ) -> Result<ExtensionLicenseStatus, ExtensionLicenseError>;
}
```

---

# 229. License Status

```rust
pub enum ExtensionLicenseStatus {
    Licensed,
    Grace,
    Unlicensed,
    Suspended,
    Revoked,
}
```

---

# 230. Hard Rule

Extension receives only necessary status, not billing/license personal details.

---

# 231. Activation API

```rust
pub trait ExtensionLicenseActivationService {
    async fn activate(
        &self,
        grant: ExtensionLicenseGrantId,
        binding: LicenseBinding,
    ) -> Result<ExtensionLicenseActivationId, ExtensionLicenseError>;
}
```

---

# 232. Seat Service

```rust
pub trait ExtensionSeatService {
    fn assign(
        &self,
        seat: ExtensionSeatId,
        subject: SeatSubject,
    ) -> Result<(), ExtensionLicenseError>;
}
```

---

# 233. Offline Service

```rust
pub trait ExtensionOfflineLicenseService {
    fn verify(
        &self,
        license: &SignedOfflineLicense,
    ) -> Result<OfflineLicenseDecision, ExtensionLicenseError>;
}
```

---

# 234. Offline Decision

```rust
pub enum OfflineLicenseDecision {
    Valid,
    Grace,
    Expired,
    Revoked,
    Invalid,
}
```

---

# 235. Compliance Service

```rust
pub trait LicenseComplianceService {
    fn evaluate(
        &self,
        package: ExtensionPackageId,
    ) -> Result<LicenseComplianceReport, ExtensionLicenseError>;
}
```

---

# 236. Error Taxonomy

```rust
pub enum ExtensionLicenseError {
    LicenseUnknown,
    GrantUnknown,
    ActivationLimitReached,
    SeatUnavailable,
    LicenseExpired,
    LicenseSuspended,
    LicenseRevoked,
    OfflineProofExpired,
    SignatureInvalid,
    EpochRollback,
    ComplianceUnknown,
    PolicyDenied,
    Unauthorized,
    Internal,
}
```

---

# 237. License SLOs

Examples:

```text
activation availability
offline verification latency
seat assignment consistency
revocation propagation
```

---

# 238. Hard Rule

Licensing SLO failure cannot weaken security/privacy policy.

---

# 239. Security SLO

```text
0 forged license accepted
0 revoked grant reactivated through stale proof
0 license state grants SIAR security permission
```

---

# 240. Privacy SLO

```text
0 raw hardware fingerprint sent to publisher
0 licensing data joined to messaging identity
0 seat telemetry used for employee productivity scoring
```

---

# 241. Failure Modes

```text
license server unavailable
clock uncertainty
stale offline proof
duplicate seat assignment
issuer compromise
```

---

# 242. License Server Unavailable

Use cached/offline/grace policy.

---

# 243. Clock Uncertainty

Conservative bounded grace.

---

# 244. Stale Offline Proof

Require renewal; preserve data.

---

# 245. Duplicate Seat

Resolve transactionally.

---

# 246. Issuer Compromise

Rotate/reissue/revoke affected grants.

---

# 247. Testing

Need extension-license testkit.

Required scenarios:

```text
open-source package
device-bound commercial license
seat licensing
floating lease
offline license
revocation
```

---

# 248. Open-Source Test

Open-source runtime does not require commercial activation unless separate service entitlement applies.

---

# 249. Device Binding Test

Random license device ID works without hardware fingerprint export.

---

# 250. Seat Test

Seat reassignment revokes old active binding according to policy.

---

# 251. Floating Lease Test

Concurrent-use limit enforced without detailed activity tracking.

---

# 252. Offline Test

Signed offline license validates without network.

---

# 253. Epoch Test

Old offline license rejected after newer revocation.

---

# 254. License Key Test

Redeemed key cannot be replayed outside allowed activation count.

---

# 255. Data Preservation Test

License expiry does not delete user data.

---

# 256. Security Separation Test

License activation does not add SIAR runtime permission.

---

# 257. Publisher Privacy Test

Publisher cannot retrieve hardware identity.

---

# 258. Enterprise Privacy Test

Seat usage does not expose personal-context data.

---

# 259. Compliance Test

SBOM notices include transitive licenses.

---

# 260. Backup Test

Restore does not reactivate stale revoked grant.

---

# 261. Fuzzing

Fuzz:

```text
license key parser
signed offline grant
activation record
seat lease
license compliance metadata
```

---

# 262. Property Tests

Properties:

```text
software license can never grant SIAR security capability
revoked epoch can never be downgraded by older offline proof
seat count can never exceed licensed capacity without explicit policy exception
publisher output can never include raw hardware fingerprint
```

---

# 263. Formal Verification Targets

Strong candidates:

```text
activation lifecycle
seat allocation
floating leases
offline anti-rollback
revocation
```

---

# 264. Kani Candidate

capacity/epoch/activation invariants.

---

# 265. TLA+ Candidate

```text
grant → activate → lease/seat → renew → suspend/revoke → restore
```

---

# 266. Loom Candidate

Concurrent:

```text
seat assignment
deactivation
renewal
revocation
```

---

# 267. Performance

License verification should be local and cheap.

---

# 268. Hard Rule

No remote license call per host action.

---

# 269. Signed License Cache

Local.

---

# 270. Hard Rule

Cache invalidated by known revocation epoch.

---

# 271. Lease Refresh

Background/low-frequency.

---

# 272. Hard Rule

No high-frequency beacon.

---

# 273. Storage Domains

Separate:

```text
license catalog
license grants
activations
seats
leases
revocation epochs
compliance records
```

---

# 274. Hard Rule

No message/contact data in license store.

---

# 275. Secrets

License issuer keys, redemption secrets, private registry credentials in secure secret system.

---

# 276. Hard Rule

No license-signing secret in extension runtime.

---

# 277. Partitioning

By:

```text
publisher
extension
license
tenant
site
```

---

# 278. Hard Rule

No user-behavior partition.

---

# 279. Crate Layout

Recommended:

```text
crates/
├── siar-extension-license-core/
├── siar-extension-license-catalog/
├── siar-extension-license-activation/
├── siar-extension-license-seat/
├── siar-extension-license-floating/
├── siar-extension-license-offline/
├── siar-extension-license-compliance/
├── siar-extension-license-audit/
├── siar-extension-license-policy/
├── siar-extension-license-observability/
└── siar-extension-license-testkit/
```

---

# 280. `siar-extension-license-core`

Owns:

```text
SoftwareLicenseId
ExtensionLicenseGrantId
ExtensionLicenseActivationId
ExtensionLicenseError
```

---

# 281. `siar-extension-license-catalog`

Source/commercial/enterprise license descriptors.

---

# 282. `siar-extension-license-activation`

User/device activation and limits.

---

# 283. `siar-extension-license-seat`

Seat allocation/reassignment.

---

# 284. `siar-extension-license-floating`

Concurrent-use leases/local license server.

---

# 285. `siar-extension-license-offline`

Signed offline proofs, anti-rollback epochs, air-gapped renewal.

---

# 286. `siar-extension-license-compliance`

SBOM license metadata, notices, source-offer obligations.

---

# 287. `siar-extension-license-audit`

License-state transition audit only.

---

# 288. `siar-extension-license-policy`

Enforcement/grace/transfer/renewal policy.

---

# 289. `siar-extension-license-observability`

Aggregate activation/seat/license health only.

---

# 290. `siar-extension-license-testkit`

activation/offline/seat/compliance/privacy/formal tests.

---

# 291. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Software source license, commercial purchase, entitlement, license grant, activation, seat, lease, device binding, runtime permission, package trust, and marketplace governance are distinct typed domains and cannot be substituted for one another.
2. A software license grants only contractual/commercial usage rights; it can never grant SIAR file/network/device/message/admin capability, weaken sandboxing, bypass consent, or alter anonymous-routing/privacy policy.
3. Device-bound licensing uses randomized licensing-domain device identities or scoped attestation rather than raw serial/IMEI/MAC/hardware fingerprints, and publishers cannot query underlying device identity.
4. License keys are redemption credentials, not perpetual runtime authority; redeemed keys yield signed scoped grants, remain out of logs/config/sync, and are protected against guessing, replay, and over-activation.
5. Offline licenses are signed, scope/expiry/epoch aware, locally verifiable, and anti-rollback protected so air-gapped use does not require recurring network checks while stale proofs cannot override newer suspension/revocation.
6. Seat/floating licensing tracks only allocation/concurrency necessary to enforce contractual capacity and cannot become user activity timelines, employee productivity analytics, behavior scoring, or hidden device telemetry.
7. License expiry, suspension, revocation, seat loss, failed renewal, or offline uncertainty can reduce licensed feature access according to explicit policy but cannot silently delete user data, revoke unrelated SIAR permissions, or corrupt extension state.
8. Open-source/source-available/commercial license metadata, notices, SBOM references, source-offer obligations, and compliance states are bound to exact package/version artifacts; Unknown compliance never counts as Satisfied.
9. Publisher/license audit views expose minimum necessary licensing facts—grants, activations, seat counts, renewals, revocations—without raw hardware identity, messaging identity, private extension usage, or cross-product behavioral profiles.
10. Marketplace ranking, certification, incident response, governance, and security review remain independent from license tier, seat count, revenue, or publisher commercial terms; no license payment can buy trust or policy exceptions.
11. Backup, state sync, account recovery, and device replacement restore license ownership/intention only according to policy and never blindly resurrect stale activations, revoked grants, expired leases, or old runtime capability tokens.
12. Extension licensing integrates with commerce, dependencies, package distribution, entitlement state, update orchestration, publisher governance, compliance/audit, enterprise deployment, incident response, and offline operation without creating a side channel around SIAR's security, privacy, anonymity, local-first, or tenant-isolation guarantees.
```

---

# 292. Initial Production Scope

Implement first:

```text
typed software-license IDs/versions
open-source/commercial/enterprise license descriptors
dual-license metadata
license grants
user/device/seat/tenant scopes
random device-license identity
device activation limits
seat allocation/reassignment
signed offline licenses
license epochs + anti-rollback
license key redemption
license revocation/suspension
perpetual + subscription-linked licenses
license status API
license compliance registry
SBOM third-party notice generation
source-offer metadata
license audit transitions
enterprise on-prem license server
privacy-safe licensing metrics
extension-license testkit
```

Then add:

```text
floating/concurrent licensing
TPM-backed privacy-preserving device binding
site licenses
formal seat/capacity proofs
cross-provider license migration
portable offline enterprise license bundles
advanced license compatibility analysis
```

---

# 293. Definition of Done

Part 149 is complete when:

- software licensing is distinct from commerce entitlement/security authority;
- open-source and commercial terms are typed;
- device licensing avoids raw fingerprints;
- license keys redeem into signed grants;
- seat/concurrent licensing is privacy-bounded;
- offline license verification works without recurring network checks;
- revocation epochs prevent stale proof replay;
- license loss never silently deletes user data;
- compliance records bind exact SBOM/package artifacts;
- publisher audits reveal licensing facts, not user activity;
- backup/recovery cannot resurrect revoked activation;
- activation/offline/seat/compliance/privacy/fuzz/formal tests are specified.

---

# 294. Final Architecture

```text
                 EXTENSION PACKAGE
                        │
                        ▼
                SOFTWARE LICENSE
                        │
             ┌──────────┼──────────┐
             │          │          │
        OPEN-SOURCE  COMMERCIAL  ENTERPRISE
             │          │          │
             └──────────┼──────────┘
                        ▼
                  LICENSE GRANT
                        │
             ┌──────────┼──────────┐
             │          │          │
          DEVICE       SEAT      TENANT
             │          │          │
             └──────────┼──────────┘
                        ▼
                 SIGNED ACTIVATION
                        │
                        ▼
              LOCAL LICENSE VERIFICATION
```

Software-license safety model:

```text
explicit license identity
+
scoped grants
+
privacy-preserving activation
+
signed offline proofs
+
anti-rollback revocation
+
seat/concurrency limits
+
SBOM/license compliance
+
minimal audit data
```

not:

```text
fingerprint every device, phone home constantly, expose users to publishers, or treat a paid license as permission to access private data
```

---

# 295. Final Principle

Software licensing is trustworthy when SIAR can answer **which legal/commercial rights apply, who or what is licensed, how that right is activated, how it works offline, how revocation behaves, and which identifiers never leave the licensing boundary**.

The correct model is:

```text
declare exact license terms
+
separate open-source and commercial rights
+
issue scoped signed grants
+
activate with privacy-preserving identities
+
support seats and offline use
+
enforce revocation with epochs
+
preserve data through license failure
+
audit state transitions only
+
never turn licensing into device surveillance or security authority
```

This architecture gives SIAR a privacy-preserving software-license foundation for open-source/commercial licensing, seat activation, license keys, offline licensing, enterprise deployment, compliance, audit, revocation, renewal, and source-license obligations while preserving the anonymity, local-first, least-authority, marketplace-governance, commerce, runtime, permission, and anti-surveillance guarantees established across Parts 34–148.
