# Core System Architecture Part 117 — Anonymous Network Physical Security, Tamper Detection, Site Access Control, Chain of Custody, Asset Protection & Privacy-Preserving Facility Security Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 117  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 55, 71–73, 94–100, 104–106, 115–116

**Primary purpose:** define SIAR's physical-security architecture for facility trust zones, asset protection, site access control, tamper detection, chain of custody, secure receiving/shipping, hardware movement, vendor/visitor handling, break-glass access, evidence preservation, asset destruction/reuse custody, and privacy-preserving facility security.

---

# 1. Purpose

Physical security protects the trust foundation beneath software and cryptography.

A secure platform must answer:

```text
Who or what is allowed into a site?
Which rooms or racks are high-trust zones?
How is tamper detected?
How is a device tracked when it moves between trusted locations?
How do we prove who had custody during repair or RMA?
What happens if a seal is broken?
How is emergency access granted without creating a permanent superuser?
How is evidence preserved without turning the facility into a workforce-surveillance system?
```

The governing principle is:

> **SIAR physical security should protect facilities, hardware, trust roots, and custody boundaries through scoped authorization, tamper evidence, controlled movement, and verifiable custody—without maintaining continuous behavioral surveillance of workers or users.**

---

# 2. Architectural Position

```text
                   FACILITY PERIMETER
                          │
                          ▼
                     TRUST ZONES
                          │
             ┌────────────┼────────────┐
             │            │            │
          Access        Tamper       Custody
             │            │            │
             └────────────┼────────────┘
                          ▼
                    ASSET PROTECTION
                          │
              ┌───────────┼───────────┐
              │           │           │
           Receive      Operate      Move
              │           │           │
              └───────────┼───────────┘
                          ▼
                 VERIFY / ESCALATE
                          │
                          ▼
              REUSE / RETURN / DESTROY
```

---

# 3. Core Separation

Keep distinct:

```text
site access
asset access
asset custody
tamper evidence
identity/authentication
authorization
employee attendance
user presence
```

---

# 4. Non-Goals

Part 117 does not create:

```text
continuous employee location tracking
facial-recognition attendance
keystroke/workforce analytics
user movement tracking
private-content inspection
```

---

# 5. Physical Trust Zone

```rust
pub struct PhysicalTrustZoneId(pub [u8; 16]);
```

---

# 6. Trust Zone Classes

```rust
pub enum PhysicalTrustZoneClass {
    Public,
    Reception,
    Operations,
    Restricted,
    HighSecurity,
    CryptographicCustody,
}
```

---

# 7. Public

No sensitive infrastructure.

---

# 8. Operations

General facility work.

---

# 9. Restricted

Infrastructure racks/equipment.

---

# 10. HighSecurity

Critical platform hardware.

---

# 11. CryptographicCustody

Examples:

```text
HSM
offline root signing
governance authority
key ceremony room
```

---

# 12. Hard Rule

Trust zones are infrastructure-control boundaries, not employee-ranking categories.

---

# 13. Zone Record

```rust
pub struct PhysicalTrustZone {
    pub id: PhysicalTrustZoneId,
    pub site: PhysicalSiteId,
    pub class: PhysicalTrustZoneClass,
    pub required_controls: BTreeSet<PhysicalControlId>,
}
```

---

# 14. Zone Inheritance

A stricter inner zone inherits outer controls.

---

# 15. Hard rule.

---

# 16. Physical Access Subject

Access can be granted to:

```rust
pub enum PhysicalAccessSubject {
    WorkforceIdentity(WorkforceIdentityId),
    VendorIdentity(VendorIdentityId),
    EmergencyTeam(EmergencyTeamId),
}
```

---

# 17. No User Identity

Hard rule.

---

# 18. Access Authorization

```rust
pub struct PhysicalAccessAuthorization {
    pub subject: PhysicalAccessSubject,
    pub zone: PhysicalTrustZoneId,
    pub purpose: PhysicalAccessPurpose,
    pub valid_from: Timestamp,
    pub expires_at: Timestamp,
}
```

---

# 19. Purpose Binding

```rust
pub enum PhysicalAccessPurpose {
    Installation,
    Maintenance,
    Repair,
    Audit,
    Inventory,
    IncidentResponse,
    Emergency,
}
```

---

# 20. Hard Rule

Access is purpose-bound and time-bounded.

---

# 21. Least Physical Privilege

A person approved for a site is not automatically approved for every room/rack.

---

# 22. Hard rule.

---

# 23. Zone Entry Decision

```rust
pub enum ZoneEntryDecision {
    Allow,
    Deny,
    RequireEscort,
    RequireDualControl,
}
```

---

# 24. Dual Control

Required for selected high-value zones.

---

# 25. Hard rule.

---

# 26. Access Factors

Can include:

```text
badge/token
MFA
PIN
hardware credential
escort approval
```

---

# 27. Biometrics

Not required by core architecture.

---

# 28. Hard rule.

---

# 29. Avoid Shared Credentials

No shared badge/passcode for high-trust zones.

---

# 30. Hard rule.

---

# 31. Visitor Access

Visitors/vendors use temporary identities.

---

# 32. Visitor Record

```rust
pub struct VisitorAuthorization {
    pub visitor: VendorIdentityId,
    pub sponsor: WorkforceIdentityId,
    pub zones: BTreeSet<PhysicalTrustZoneId>,
    pub expires_at: Timestamp,
}
```

---

# 33. Escort

May be required.

---

# 34. Hard rule.

---

# 35. Visitor Data Minimization

Keep only information needed for access/security/legal retention.

---

# 36. No permanent visitor profiling.

---

# 37. Hard rule.

---

# 38. Break-Glass Physical Access

Emergency access exists.

---

# 39. Break-Glass Authorization

```rust
pub struct PhysicalBreakGlassGrant {
    pub subject: PhysicalAccessSubject,
    pub zone: PhysicalTrustZoneId,
    pub reason: EmergencyReasonCode,
    pub expires_at: Timestamp,
}
```

---

# 40. Break Glass Cannot Be Permanent

Hard rule.

---

# 41. Break-Glass Controls

Require:

```text
strong authentication
short expiry
post-event review
high-sensitivity audit
```

---

# 42. Hard rule.

---

# 43. No Privacy/Security Override

Break glass cannot disable:

```text
crypto policy
tamper response
chain-of-custody rules
```

---

# 44. Hard rule.

---

# 45. Physical Access Events

Store only meaningful events:

```text
grant issued
entry authorized
access denied
break-glass used
high-security zone accessed
```

---

# 46. No Continuous Movement Timeline

Hard rule.

---

# 47. Access Event

```rust
pub struct PhysicalAccessEvent {
    pub zone: PhysicalTrustZoneId,
    pub subject_class: PhysicalAccessSubjectClass,
    pub event_type: PhysicalAccessEventType,
    pub at: CoarseTimestamp,
}
```

---

# 48. Subject Class Instead Of Identity Where Possible

Good for aggregate reporting.

---

# 49. Identity Retained Only Where Security Evidence Requires

Hard rule.

---

# 50. Anti-Tailgating

Facility systems may enforce:

```text
turnstile
mantrap
door interlock
escort
```

---

# 51. Hard rule.

---

# 52. Door State

```rust
pub enum SecureDoorState {
    ClosedLocked,
    OpenAuthorized,
    OpenUnexpected,
    Forced,
    Unknown,
}
```

---

# 53. Unknown ≠ Secure

Hard rule.

---

# 54. Tamper Detection

Tamper must be modeled at:

```text
rack
chassis
HSM
network appliance
shipping container
seal
```

---

# 55. Tamper Sensor Type

```rust
pub enum TamperSensorType {
    ChassisOpen,
    RackDoor,
    SealBreak,
    Motion,
    Shock,
    LightExposure,
    HsmTamper,
    EnclosureOpen,
}
```

---

# 56. No Personnel Tracking Sensors

Hard rule.

---

# 57. Tamper State

```rust
pub enum TamperState {
    Intact,
    Suspected,
    Confirmed,
    Cleared,
    Unknown,
}
```

---

# 58. Unknown ≠ Intact

Hard rule.

---

# 59. Tamper Event

```rust
pub struct TamperEvent {
    pub asset: Option<HardwareAssetId>,
    pub zone: Option<PhysicalTrustZoneId>,
    pub sensor_type: TamperSensorType,
    pub state: TamperState,
    pub observed_at: CoarseTimestamp,
}
```

---

# 60. Tamper Confidence

Corroborate where possible.

---

# 61. Hard rule.

---

# 62. Tamper Response

Sequence:

```text
detect
preserve state
quarantine affected asset/zone
restrict credentials if needed
capture evidence
investigate
re-attest/reprovision before trust restoration
```

---

# 63. Hard rule.

---

# 64. No Automatic Evidence Destruction

Hard rule.

---

# 65. HSM Tamper

HSM may zeroize keys.

---

# 66. System records event/impact, not key material.

---

# 67. Hard rule.

---

# 68. Asset Protection Class

```rust
pub enum AssetProtectionClass {
    Standard,
    Restricted,
    Critical,
    Cryptographic,
}
```

---

# 69. Protection Requirements

```rust
pub struct AssetProtectionPolicy {
    pub class: AssetProtectionClass,
    pub zone_required: PhysicalTrustZoneClass,
    pub tamper_controls: BTreeSet<TamperSensorType>,
    pub dual_control: bool,
}
```

---

# 70. Hard rule.

---

# 71. High-Value Assets

Examples:

```text
HSM
offline root key appliance
release signing host
governance authority
backup key escrow
```

---

# 72. Hard rule.

---

# 73. Asset Movement

Moving hardware between zones/sites is a governed custody transition.

---

# 74. Asset Movement Record

```rust
pub struct AssetMovement {
    pub asset: HardwareAssetId,
    pub from: CustodyLocation,
    pub to: CustodyLocation,
    pub reason: AssetMovementReason,
    pub movement_id: MovementId,
}
```

---

# 75. Custody Location

```rust
pub enum CustodyLocation {
    Site(PhysicalSiteId),
    Zone(PhysicalTrustZoneId),
    Vendor(VendorId),
    Transit(ShipmentId),
    DestructionFacility(DestructionFacilityId),
}
```

---

# 76. Asset Movement Reason

```rust
pub enum AssetMovementReason {
    Installation,
    Maintenance,
    Repair,
    Relocation,
    Rma,
    Reuse,
    Destruction,
}
```

---

# 77. No Untracked Movement

Hard rule.

---

# 78. Chain of Custody

Every high-value asset movement records custody handoff.

---

# 79. Custody Record

```rust
pub struct CustodyRecord {
    pub asset: HardwareAssetId,
    pub movement: MovementId,
    pub custodian_scope: CustodianScope,
    pub accepted_at: Timestamp,
    pub released_at: Option<Timestamp>,
}
```

---

# 80. Custodian Scope

```rust
pub enum CustodianScope {
    InternalTeam(TeamId),
    ApprovedVendor(VendorId),
    Carrier(CarrierId),
    DestructionProvider(VendorId),
}
```

---

# 81. Prefer Organizational Custody Over Person-Level History

Hard rule.

---

# 82. Person Identity

Used only if assurance policy requires explicit handoff.

---

# 83. Hard rule.

---

# 84. Custody Handoff

Two-sided receipt.

---

# 85. Handoff Receipt

```rust
pub struct CustodyHandoffReceipt {
    pub movement: MovementId,
    pub from: CustodianScope,
    pub to: CustodianScope,
    pub asset_digest: Digest,
    pub timestamp: Timestamp,
}
```

---

# 86. Hard rule.

---

# 87. Seal / Package Integrity

Shipment may use tamper-evident seal.

---

# 88. Seal Record

```rust
pub struct TamperSeal {
    pub seal_id: SealId,
    pub asset: HardwareAssetId,
    pub state: SealState,
}
```

---

# 89. Seal State

```rust
pub enum SealState {
    Applied,
    Verified,
    BrokenExpected,
    BrokenUnexpected,
    Unknown,
}
```

---

# 90. BrokenUnexpected

Triggers investigation.

---

# 91. Hard rule.

---

# 92. Secure Receiving

On arrival:

```text
verify shipment
verify seal
verify serial/provenance
inspect damage
re-enroll/re-attest
verify firmware baseline
```

---

# 93. No direct production admission after shipping.

---

# 94. Hard rule.

---

# 95. Secure Shipping

Before shipment:

```text
sanitize if data-bearing
record custody
apply seal if policy requires
issue transport manifest
```

---

# 96. Hard rule.

---

# 97. Shipping Manifest

```rust
pub struct SecureShipmentManifest {
    pub shipment: ShipmentId,
    pub assets: Vec<HardwareAssetId>,
    pub destination: CustodyLocation,
    pub seal_ids: Vec<SealId>,
}
```

---

# 98. No Sensitive Configuration In Manifest

Hard rule.

---

# 99. Carrier Boundary

Carrier sees minimum required logistics metadata.

---

# 100. No internal topology details.

---

# 101. Hard rule.

---

# 102. RMA Chain Of Custody

Part 115.

Before vendor return:

```text
sanitize
custody receipt
shipping seal
vendor authorization
```

---

# 103. Hard rule.

---

# 104. Failed Storage Media

If cannot sanitize:

```text
do not return by default
```

Prefer:

```text
retain/destroy
vendor on-site destruction
contractual secure destruction
```

---

# 105. Hard rule.

---

# 106. Repair Vendor

Vendor access is scoped to asset/task.

---

# 107. No general site access.

---

# 108. Hard rule.

---

# 109. Repair Evidence

```rust
pub struct RepairReceipt {
    pub asset: HardwareAssetId,
    pub vendor: VendorId,
    pub replaced_components: Vec<ComponentRef>,
    pub completed_at: Timestamp,
}
```

---

# 110. Trust Re-establishment

Sensitive replacements trigger re-enrollment/re-attestation.

---

# 111. Hard rule.

---

# 112. Asset Destruction

Selected hardware/media may require verified destruction.

---

# 113. Destruction State

```rust
pub enum AssetDestructionState {
    Approved,
    InCustody,
    Destroyed,
    Verified,
}
```

---

# 114. No "Destroyed" Without Evidence

Hard rule.

---

# 115. Destruction Receipt

```rust
pub struct DestructionReceipt {
    pub asset: HardwareAssetId,
    pub provider: Option<VendorId>,
    pub method: DestructionMethod,
    pub evidence_digest: Digest,
    pub completed_at: Timestamp,
}
```

---

# 116. Evidence Minimization

Do not require video of workers unless policy/legal reasons demand.

---

# 117. Hard rule.

---

# 118. Reuse / Refurbishment Custody

Part 114/115.

Reuse requires:

```text
sanitization
hardware health
firmware baseline
new custody scope
```

---

# 119. Hard rule.

---

# 120. Asset Protection At Rest

Controls may include:

```text
locked rack
cage
tamper seal
HSM enclosure
secure storage
```

---

# 121. Hard rule.

---

# 122. Asset Protection In Transit

Controls:

```text
sealed shipment
tracked carrier reference
handoff receipts
route confidentiality where needed
```

---

# 123. No Worker Tracking

Hard rule.

---

# 124. Location Confidentiality

High-value facility/rack locations are sensitive.

---

# 125. Access strictly scoped.

---

# 126. Hard rule.

---

# 127. Site Address Disclosure

Only operational staff/vendors who require it.

---

# 128. Public systems should not expose detailed facility topology.

---

# 129. Hard rule.

---

# 130. Physical Intrusion

Examples:

```text
forced door
unexpected rack open
tamper seal broken
unrecognized asset removal
```

---

# 131. Intrusion State

```rust
pub enum PhysicalIntrusionState {
    Suspected,
    Confirmed,
    Contained,
    Cleared,
}
```

---

# 132. Response

```text
deny further access if safe
quarantine affected assets
notify security/SOC
preserve evidence
re-attest systems
```

---

# 133. Hard rule.

---

# 134. Physical Security Incident

Part 96 integration.

---

# 135. Not every access denial is an incident.

---

# 136. Hard rule.

---

# 137. SOC Integration

Part 97.

Signals:

```text
unexpected tamper
forced entry
firmware changes after physical access
custody mismatch
```

---

# 138. No user data.

---

# 139. Hard rule.

---

# 140. Correlation With Cyber Signals

Useful:

```text
rack opened
+
boot measurement changed
```

---

# 141. Stronger security signal.

---

# 142. Hard rule.

---

# 143. Tamper Evidence Provenance

Store:

```text
sensor ID
asset/zone
timestamp
quality
```

---

# 144. No hidden source.

---

# 145. Hard rule.

---

# 146. Tamper Sensor Health

```rust
pub enum TamperSensorHealth {
    Healthy,
    Degraded,
    Failed,
    Unknown,
}
```

---

# 147. Failed sensor lowers protection confidence.

---

# 148. Hard rule.

---

# 149. Tamper Coverage

High-value assets need required coverage.

---

# 150. Coverage State

```rust
pub enum TamperCoverageState {
    Sufficient,
    Partial,
    Insufficient,
    Unknown,
}
```

---

# 151. Unknown ≠ Sufficient

Hard rule.

---

# 152. Access Control System Failure

Fail mode depends zone.

---

# 153. Fail Secure vs Fail Safe

```rust
pub enum DoorFailurePolicy {
    FailSecure,
    FailSafe,
    LifeSafetyOverride,
}
```

---

# 154. Life Safety Overrides Security

Hard rule.

---

# 155. Fire/Emergency Egress

Must always permit safe human exit according to facility safety design.

---

# 156. Hard rule.

---

# 157. Security Door Recovery

After outage/emergency:

```text
revalidate controller
review access logs
verify zone state
```

---

# 158. Hard rule.

---

# 159. Access Controller Security

Controllers are critical infrastructure assets.

---

# 160. They need:

```text
firmware governance
network isolation
authn/authz
time sync
```

---

# 161. Hard rule.

---

# 162. No Cloud-Only Facility Access Dependency

High-security sites should retain local safe behavior during WAN failure.

---

# 163. Hard rule.

---

# 164. Offline Access Decisions

Use cached signed policy with expiry.

---

# 165. No unsigned permissive fallback.

---

# 166. Hard rule.

---

# 167. Badge/Credential Lifecycle

```rust
pub enum PhysicalCredentialState {
    Active,
    Suspended,
    Revoked,
    Expired,
}
```

---

# 168. Immediate Revocation

For terminated/lost credentials.

---

# 169. Hard rule.

---

# 170. Lost Badge

Revoke credential.

---

# 171. Do not use it as permanent employee-risk signal.

---

# 172. Hard rule.

---

# 173. Temporary Access Credential

Short-lived.

---

# 174. Hard rule.

---

# 175. Dual Authorization

High-risk access can require two independent approvals.

---

# 176. Example:

```text
HSM custody room
offline root ceremony
```

---

# 177. Hard rule.

---

# 178. Key Ceremony Physical Controls

Part 72.

Requirements may include:

```text
dual control
sealed materials
offline devices
custody log
asset verification
```

---

# 179. No secret key material in custody log.

---

# 180. Hard rule.

---

# 181. Offline Root Storage

Physical custody may use:

```text
safe
vault
tamper-evident storage
```

---

# 182. Hard rule.

---

# 183. Backup Media Custody

If removable backup media exists:

```text
encrypt before writing
custody track media
site-safe storage
sanitize/destroy at retirement
```

---

# 184. Hard rule.

---

# 185. Removable Media

Disabled by default on critical hosts unless explicitly authorized.

---

# 186. Hard rule.

---

# 187. Media Import

Signed/verified offline bundles only.

---

# 188. Malware scanning where appropriate.

---

# 189. Hard rule.

---

# 190. Media Export

Policy-controlled.

---

# 191. No raw sensitive data export as convenience.

---

# 192. Hard rule.

---

# 193. Asset Presence Verification

Inventory reconciliation detects:

```text
asset expected but missing
unexpected asset present
```

---

# 194. Part 106.

---

# 195. Hard rule.

---

# 196. Missing Critical Asset

Potential physical-security incident.

---

# 197. Hard rule.

---

# 198. Unexpected Asset

Potential rogue hardware.

---

# 199. Quarantine/isolate before trust.

---

# 200. Hard rule.

---

# 201. Rogue Device Detection

At physical/network inventory level.

---

# 202. No user-device surveillance.

---

# 203. Hard rule.

---

# 204. Secure Cabling

Critical links may require:

```text
locked paths
tamper-evident patching
documented cross-connects
```

---

# 205. Hard rule.

---

# 206. Cross-Connect Custody

Provider-managed.

---

# 207. Minimal evidence.

---

# 208. Hard rule.

---

# 209. Physical Port Security

Unused management ports disabled/secured.

---

# 210. Hard rule.

---

# 211. Console Port Security

Physical console access is privileged.

---

# 212. JIT authorization.

---

# 213. Hard rule.

---

# 214. USB/Peripheral Policy

Critical systems:

```text
restrict removable peripherals
allow approved maintenance devices only
```

---

# 215. Hard rule.

---

# 216. Portable Maintenance Device

Should be:

```text
managed
attested where possible
temporary
clean/reprovisioned
```

---

# 217. Hard rule.

---

# 218. Chain-of-Custody State Machine

```rust
pub enum CustodyState {
    InTrustedSite,
    PreparedForTransfer,
    InTransit,
    Received,
    UnderVendorControl,
    Sanitized,
    Destroyed,
}
```

---

# 219. No Skipped Custody States For High-Value Asset

Hard rule.

---

# 220. Custody Conflict

Examples:

```text
asset location disagrees with manifest
seal mismatch
handoff missing
```

---

# 221. State

```rust
pub enum CustodyConflictState {
    None,
    Suspected,
    Confirmed,
    Resolved,
}
```

---

# 222. Confirmed conflict blocks trust.

---

# 223. Hard rule.

---

# 224. Custody Verification

```rust
pub trait CustodyVerificationService {
    fn verify(
        &self,
        asset: HardwareAssetId,
    ) -> Result<CustodyVerificationResult, PhysicalSecurityError>;
}
```

---

# 225. Physical Access Service

```rust
pub trait SiteAccessControlService {
    fn authorize(
        &self,
        subject: PhysicalAccessSubject,
        zone: PhysicalTrustZoneId,
        purpose: PhysicalAccessPurpose,
    ) -> Result<ZoneEntryDecision, PhysicalSecurityError>;
}
```

---

# 226. Tamper Monitoring Service

```rust
pub trait TamperMonitoringService {
    fn state(
        &self,
        asset: HardwareAssetId,
    ) -> Result<TamperState, PhysicalSecurityError>;
}
```

---

# 227. Asset Movement Service

```rust
pub trait AssetMovementService {
    fn begin(
        &self,
        movement: AssetMovement,
    ) -> Result<MovementId, PhysicalSecurityError>;

    fn handoff(
        &self,
        receipt: CustodyHandoffReceipt,
    ) -> Result<(), PhysicalSecurityError>;
}
```

---

# 228. No Generic Physical Admin Override API

Hard rule.

---

# 229. Error Taxonomy

```rust
pub enum PhysicalSecurityError {
    ZoneUnknown,
    AccessDenied,
    AccessExpired,
    DualControlRequired,
    TamperDetected,
    TamperStateUnknown,
    CustodyMismatch,
    SealMismatch,
    AssetMissing,
    AssetUnexpected,
    SanitizationRequired,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 230. Observability

Safe metrics:

```text
access denials by zone class
tamper alerts
custody conflicts
assets in transit
sanitization/destruction status
```

---

# 231. Forbidden:

```text
employee movement timeline
continuous badge telemetry
user location
private content
```

---

# 232. Hard rule.

---

# 233. Physical Security SLOs

Examples:

```text
all high-security zone access grants expire
critical assets have valid custody state
all storage leaving trust boundary has sanitization evidence
all confirmed tamper events trigger quarantine
```

---

# 234. Security SLO

```text
0 high-value assets in Unknown custody state during active service
0 unverified tamper reset restoring trust automatically
0 permanent break-glass access
```

---

# 235. Privacy SLO

```text
0 continuous workforce movement profiling
0 user presence data in facility security systems
```

---

# 236. Failure Modes

```text
access controller outage
tamper sensor outage
custody mismatch
rogue asset
seal break
```

---

# 237. Access Controller Outage

Use cached signed policy/local safe mode.

---

# 238. No permissive default.

---

# 239. Hard rule.

---

# 240. Tamper Sensor Outage

Coverage becomes Partial/Unknown.

---

# 241. High-value asset may require inspection.

---

# 242. Hard rule.

---

# 243. Custody Mismatch

Freeze movement/trust.

---

# 244. Investigate.

---

# 245. Hard rule.

---

# 246. Rogue Asset

Isolate and inventory.

---

# 247. No network admission.

---

# 248. Hard rule.

---

# 249. Seal Break

If expected maintenance:

```text
authorized break
```

else:

```text
tamper incident
```

---

# 250. Hard rule.

---

# 251. Testing

Need physical-security testkit.

---

# 252. Test Scenarios

```text
routine maintenance access
HSM room access
vendor repair
asset shipment
unexpected seal break
```

---

# 253. Access Test

Expired grant denied.

---

# 254. Dual-Control Test

Single approver cannot enter cryptographic custody zone.

---

# 255. Break-Glass Test

Grant expires automatically.

---

# 256. Privacy Test

Movement events cannot be queried as employee timeline.

---

# 257. Tamper Test

Unknown sensor != Intact.

---

# 258. Quarantine Test

Confirmed tamper removes asset from trust.

---

# 259. Custody Test

Missing handoff blocks transfer completion.

---

# 260. Seal Test

Unexpected break triggers conflict.

---

# 261. Shipment Test

Received hardware must re-attest before trust.

---

# 262. RMA Test

Unsanitized storage cannot ship.

---

# 263. Destruction Test

Destroyed state requires receipt.

---

# 264. Rogue Asset Test

Unexpected asset cannot receive workload.

---

# 265. Offline Access Test

Signed cached policy works; expired policy fails closed where life safety permits.

---

# 266. Fuzzing

Fuzz:

```text
access grants
custody transitions
tamper events
shipment manifests
seal records
```

---

# 267. Property Tests

Properties:

```text
expired physical access can never produce Allow
confirmed tamper can never directly restore Trusted hardware state
high-value asset custody can never skip required handoff states
storage-bearing asset can never leave trust boundary without required sanitization evidence
```

---

# 268. Formal Verification Targets

Strong candidates:

```text
access authorization
custody state machine
tamper-to-quarantine state transition
break-glass expiry
```

---

# 269. Kani Candidate

access/custody/tamper invariants.

---

# 270. TLA+ Candidate

trusted site → transfer → transit → receive → re-attest → active.

---

# 271. Loom Candidate

concurrent access revocation + entry decision + emergency override.

---

# 272. Performance

Physical security is off request path.

---

# 273. Local access decisions should work without WAN where safe.

---

# 274. Hard rule.

---

# 275. Storage

Separate:

```text
trust zones
access grants
high-sensitivity access events
tamper events
custody records
shipment manifests
seal records
destruction/sanitization receipts
```

---

# 276. No workforce tracking warehouse.

---

# 277. Hard rule.

---

# 278. Partitioning

By:

```text
site
zone
asset
movement
shipment
protection class
```

---

# 279. No user partition.

---

# 280. Hard rule.

---

# 281. Crate Layout

Recommended:

```text
crates/
├── siar-physical-security-core/
├── siar-site-access-control/
├── siar-trust-zones/
├── siar-tamper-detection/
├── siar-chain-of-custody/
├── siar-asset-movement/
├── siar-secure-shipping/
├── siar-asset-protection/
├── siar-physical-security-observability/
└── siar-physical-security-testkit/
```

---

# 282. `siar-physical-security-core`

Owns:

```text
PhysicalTrustZoneId
AssetProtectionClass
PhysicalSecurityError
```

---

# 283. `siar-site-access-control`

Purpose-bound/time-bounded access, escort, dual control, break glass.

---

# 284. `siar-trust-zones`

Zone hierarchy/control requirements.

---

# 285. `siar-tamper-detection`

Tamper sensors/state/coverage/quarantine integration.

---

# 286. `siar-chain-of-custody`

Custody states/handoffs/conflicts.

---

# 287. `siar-asset-movement`

Site/vendor/transit movement workflow.

---

# 288. `siar-secure-shipping`

Shipment/seal/receiving/RMA controls.

---

# 289. `siar-asset-protection`

Protection class/high-value asset policy.

---

# 290. `siar-physical-security-observability`

Aggregate security health only.

---

# 291. `siar-physical-security-testkit`

access/tamper/custody/privacy tests.

---

# 292. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Physical security controls facilities, trust zones, hardware assets, tamper state, custody, and asset movement—not user presence, private content, or continuous workforce behavior.
2. Site/zone access is purpose-bound, scoped, time-bounded, strongly authenticated, and least-privileged; high-security areas may require escort or dual control.
3. Break-glass physical access is temporary, auditable, and reviewed and cannot become a permanent facility superuser or disable tamper/custody controls.
4. Unknown, failed, or missing tamper evidence can never be interpreted as Intact for assets whose protection policy requires active tamper coverage.
5. Confirmed tamper or unexplained custody conflict immediately removes affected high-value hardware from trusted operational status until investigation and re-attestation/reprovisioning are complete.
6. Asset movement across sites, vendors, carriers, repair facilities, and destruction providers uses explicit custody transitions and verifiable two-sided handoff evidence for protected assets.
7. Data-bearing hardware cannot leave its trust boundary for RMA, resale, donation, reuse, or disposal without required sanitization/cryptographic-erasure evidence or an explicit secure-destruction path.
8. High-value cryptographic assets, HSMs, offline roots, and signing systems use stronger physical zones, dual control, custody controls, and tamper response without exposing key material in facility records.
9. Physical-access, visitor, badge, and custody records are minimized to operational/security evidence and cannot become continuous employee movement tracking, attendance analytics, or workforce scoring.
10. Site access control remains safely operable during WAN failure using valid cached signed policy or life-safety mechanisms; it never fails open to privileged zones merely because a remote service is unavailable.
11. Physical security events integrate with hardware trust, SOC, incident response, desired state, inventory, firmware governance, sanitization, and compliance without creating a parallel uncontrolled physical administration path.
12. Life safety overrides access-control confinement for emergency egress, but emergency egress does not authorize continued technical access to protected assets or weaken subsequent security investigation requirements.
```

---

# 293. Initial Production Scope

Implement first:

```text
typed PhysicalTrustZoneId/PhysicalTrustZoneClass
zone hierarchy
purpose-bound access grants
temporary vendor/visitor identities
escort/dual-control rules
break-glass physical access
secure-door states
tamper sensor/state model
tamper coverage/health
asset protection classes
chain-of-custody state machine
asset movement records
two-sided custody handoff
shipping manifests/seals
secure receiving
RMA custody
destruction receipts
offline signed access-policy cache
audit/compliance integration
privacy-safe dashboards
physical-security testkit
```

Then add:

```text
tamper-evident rack integration
facility-controller adapters
hardware shipping sensor integration
high-assurance custody signatures
cross-provider destruction verification
formal custody/access verification
```

---

# 294. Definition of Done

Part 117 is complete when:

- physical trust zones are explicit;
- privileged access is scoped/purpose-bound/time-bounded;
- high-security zones support escort/dual control;
- break-glass expires automatically;
- tamper state/coverage is typed;
- confirmed tamper removes hardware trust;
- asset movement has custody handoffs;
- shipping/RMA uses manifests and seal state;
- data-bearing assets require sanitization before leaving trust boundary;
- secure receiving includes provenance and re-attestation;
- offline facility access does not fail open;
- no user/workforce surveillance graph is created;
- access/tamper/custody/privacy/fuzz/formal tests are specified.

---

# 295. Final Architecture

```text
                 PHYSICAL FACILITY
                        │
                        ▼
                    TRUST ZONES
                        │
          ┌─────────────┼─────────────┐
          │             │             │
       ACCESS         TAMPER        CUSTODY
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                  ASSET PROTECTION
                        │
              ┌─────────┼─────────┐
              │         │         │
           OPERATE     MOVE     REPAIR
              │         │         │
              └─────────┼─────────┘
                        ▼
              VERIFY / QUARANTINE
                        │
                        ▼
            REUSE / RETURN / DESTROY
```

Physical-security safety model:

```text
explicit trust zones
+
time-bounded access
+
dual control for high-value areas
+
tamper evidence
+
custody handoffs
+
sanitization before trust-boundary exit
+
secure receiving/re-attestation
+
privacy-minimized access records
```

not:

```text
keep permanent master badges, trust unsealed shipments, send failed drives to vendors unsanitized, and record every employee movement forever
```

---

# 296. Final Principle

Physical security should prove that protected assets remained inside controlled trust and custody boundaries—not prove where people spent every minute.

The correct model is:

```text
segment physical trust
+
authorize narrowly
+
detect tamper
+
track protected asset custody
+
verify every movement boundary
+
sanitize before release
+
re-establish hardware trust after exposure
+
retain only the physical-security evidence actually required
```

This architecture gives SIAR a privacy-preserving facility-security foundation for trust zones, physical access, tamper detection, chain of custody, asset movement, secure shipping, RMA, destruction, and high-value hardware protection while preserving the anonymity, local-first, least-authority, hardware-trust, facility-resilience, and anti-surveillance guarantees established across Parts 34–116.
