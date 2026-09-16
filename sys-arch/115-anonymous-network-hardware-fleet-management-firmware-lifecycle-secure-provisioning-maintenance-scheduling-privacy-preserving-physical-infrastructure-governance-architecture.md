# Core System Architecture Part 115 — Anonymous Network Hardware Fleet Management, Firmware Lifecycle, Secure Provisioning, Maintenance Scheduling & Privacy-Preserving Physical Infrastructure Governance Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 115  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 62, 71–73, 98–100, 104–106, 110, 113–114

**Primary purpose:** define SIAR's hardware-fleet and physical-infrastructure governance architecture for hardware identity, secure enrollment, provisioning, measured boot, firmware baselines, firmware lifecycle, maintenance scheduling, spare inventory, physical access, RMA, component replacement, decommissioning, secure sanitization, fleet evidence, and privacy-preserving physical operations.

---

# 1. Purpose

Software security depends on physical infrastructure that is:

```text
known
provisioned correctly
firmware-current
measured
maintained
repairable
replaceable
retired safely
```

Fleet management must answer:

```text
Which physical assets exist?
Which firmware is approved?
Which host is due for maintenance?
Which assets have drifted from their secure baseline?
Which components may be replaced safely?
Which spare hardware can be promoted?
Which devices must be quarantined or retired?
```

The governing principle is:

> **SIAR physical infrastructure should be explicitly identified, securely provisioned, firmware-governed, maintenance-aware, and cryptographically verifiable without collecting unnecessary workforce or user-location data.**

---

# 2. Architectural Position

```text
Hardware Acquisition
       │
       ▼
 Secure Enrollment
       │
       ▼
 Provisioning Baseline
       │
       ▼
 Firmware / Boot Trust
       │
       ▼
  Fleet Operation
       │
  ┌────┼─────┐
  │    │     │
Patch Repair Maintain
  │    │     │
  └────┼─────┘
       ▼
 Re-Attest / Verify
       │
       ▼
Retire / Reuse / Recycle
```

---

# 3. Core Separation

Keep distinct:

```text
hardware identity
host identity
workload identity
firmware state
boot measurements
maintenance state
physical location
human operator identity
```

---

# 4. Non-Goals

Part 115 does not create:

```text
employee physical-movement tracking
user-device fleet enumeration by default
continuous CCTV/biometric monitoring
a generic remote-management backdoor
firmware rollout without rollback protection
```

---

# 5. Hardware Asset Identity

```rust
pub struct HardwareAssetId(pub [u8; 16]);
```

Opaque and never reused.

---

# 6. Hardware Identity Sources

May include:

```text
manufacturer serial
TPM identity
BMC identity
inventory tag
motherboard UUID
```

---

# 7. Hard Rule

External serials are attributes, not primary authorization identity.

---

# 8. Hardware Asset Record

```rust
pub struct HardwareAsset {
    pub id: HardwareAssetId,
    pub class: HardwareClass,
    pub lifecycle: HardwareLifecycleState,
    pub owner_scope: HardwareOwnerScope,
    pub trust_state: HardwareTrustState,
}
```

---

# 9. Hardware Class

```rust
pub enum HardwareClass {
    Server,
    StorageNode,
    RelayNode,
    MixNode,
    MailboxNode,
    HsmAppliance,
    NetworkAppliance,
    EdgeNode,
    EmbeddedNode,
    BuildHost,
    RecoveryHost,
    DevelopmentHost,
}
```

---

# 10. Hardware Owner Scope

```rust
pub enum HardwareOwnerScope {
    Platform,
    Region(RegionId),
    Tenant(TenantId),
    FederationDomain(FederationDomainId),
}
```

---

# 11. No User Scope

Hard rule.

---

# 12. Hardware Trust State

```rust
pub enum HardwareTrustState {
    Unenrolled,
    Enrolled,
    Provisioning,
    Trusted,
    Degraded,
    Quarantined,
    Compromised,
    Retired,
}
```

---

# 13. State Transitions

```text
Unenrolled
  ↓
Enrolled
  ↓
Provisioning
  ↓
Trusted
  ↓ ↘
Degraded  Quarantined
  ↓          ↓
Trusted   Compromised
  ↓          ↓
Retired ←────┘
```

---

# 14. Hard Rule

No direct `Unenrolled -> Trusted`.

---

# 15. Enrollment

Secure enrollment binds physical asset identity to platform inventory.

---

# 16. Enrollment Request

```rust
pub struct HardwareEnrollmentRequest {
    pub asset: HardwareAssetId,
    pub hardware_class: HardwareClass,
    pub evidence: EnrollmentEvidence,
}
```

---

# 17. Enrollment Evidence

```rust
pub enum EnrollmentEvidence {
    TpmEndorsement,
    ManufacturerCertificate,
    OfflineCeremony,
    ApprovedManualImport,
}
```

---

# 18. No Blind Serial-Number Enrollment

Hard rule.

---

# 19. Enrollment Authority

Scoped operational role.

---

# 20. No Single Universal Hardware Admin

Hard rule.

---

# 21. Provisioning Baseline

Every hardware class has approved baseline.

```rust
pub struct HardwareProvisioningBaseline {
    pub hardware_class: HardwareClass,
    pub firmware_policy: FirmwarePolicyRef,
    pub boot_policy: BootPolicyRef,
    pub os_image: Option<ArtifactDigest>,
    pub network_policy: NetworkPolicyRef,
}
```

---

# 22. Provisioning Is Deterministic

Prefer:

```text
PXE/secure boot
immutable image
signed configuration
declarative networking
```

---

# 23. Hard Rule

Manual snowflake configuration is drift.

---

# 24. Firmware Domain

Firmware includes:

```text
UEFI/BIOS
BMC
NIC firmware
storage controller
drive firmware
TPM/HSM firmware
network appliance firmware
```

---

# 25. Firmware Component

```rust
pub struct FirmwareComponent {
    pub component_id: FirmwareComponentId,
    pub kind: FirmwareKind,
    pub version: FirmwareVersion,
}
```

---

# 26. Firmware Kind

```rust
pub enum FirmwareKind {
    Uefi,
    Bmc,
    Nic,
    StorageController,
    Drive,
    Tpm,
    Hsm,
    NetworkDevice,
}
```

---

# 27. Firmware Baseline

```rust
pub struct FirmwareBaseline {
    pub hardware_class: HardwareClass,
    pub components: BTreeMap<FirmwareKind, FirmwareVersionConstraint>,
    pub security_epoch: FirmwareSecurityEpoch,
}
```

---

# 28. Anti-Rollback

FirmwareSecurityEpoch monotonic for critical components.

---

# 29. Hard Rule

Known-vulnerable firmware cannot be restored simply because it was previously installed.

---

# 30. Firmware State

```rust
pub enum FirmwareComplianceState {
    Compliant,
    UpdateRequired,
    Unsupported,
    Unknown,
}
```

---

# 31. Unknown ≠ Compliant

Hard rule.

---

# 32. Firmware Policy

```rust
pub struct FirmwarePolicy {
    pub allowed_versions: Vec<FirmwareVersionConstraint>,
    pub minimum_security_epoch: FirmwareSecurityEpoch,
    pub update_strategy: FirmwareUpdateStrategy,
}
```

---

# 33. Firmware Update Strategy

```rust
pub enum FirmwareUpdateStrategy {
    ImmediateSecurity,
    Staged,
    MaintenanceWindow,
    ManualCeremony,
}
```

---

# 34. Security Firmware

May override normal maintenance timing but not artifact verification.

---

# 35. Hard rule.

---

# 36. Firmware Artifact Trust

Firmware package must be:

```text
vendor-signed where applicable
hash-pinned
provenance-recorded
verified before execution
```

---

# 37. Hard Rule

No arbitrary URL firmware flashing.

---

# 38. Firmware Update State Machine

```rust
pub enum FirmwareUpdateState {
    Planned,
    Verified,
    Scheduled,
    Applying,
    Rebooting,
    ReAttesting,
    Completed,
    RolledBack,
    Failed,
}
```

---

# 39. No Completed Before Re-Attestation

Hard rule.

---

# 40. Firmware Rollback

Allowed only if:

```text
rollback supported
target version still allowed
security epoch not violated
```

---

# 41. Hard rule.

---

# 42. Secure Boot

Secure boot chain verifies approved boot components.

---

# 43. Boot Policy

```rust
pub struct BootPolicy {
    pub secure_boot_required: bool,
    pub approved_bootloaders: BTreeSet<ArtifactDigest>,
    pub approved_kernels: BTreeSet<ArtifactDigest>,
}
```

---

# 44. Measured Boot

Part 71 integration.

---

# 45. Boot Measurements

```rust
pub struct BootMeasurementSet {
    pub firmware: Digest,
    pub bootloader: Digest,
    pub kernel: Digest,
    pub initrd: Option<Digest>,
    pub policy: Digest,
}
```

---

# 46. Attestation

```rust
pub struct HardwareAttestation {
    pub asset: HardwareAssetId,
    pub measurements: BootMeasurementSet,
    pub trust_state: HardwareTrustState,
    pub observed_at: CoarseTimestamp,
}
```

---

# 47. Attestation Is Infrastructure Trust

Not user attestation.

---

# 48. Hard rule.

---

# 49. Admission

Workload admission may require Trusted hardware.

---

# 50. Sensitive Workloads

Examples:

```text
key custody
identity authority
release signing
governance authority
```

require strongest hardware trust class.

---

# 51. Hard rule.

---

# 52. Quarantine

Triggers:

```text
unexpected firmware
failed attestation
unknown boot measurement
tamper event
unsupported security baseline
```

---

# 53. Quarantine Action

```rust
pub enum HardwareQuarantineAction {
    RemoveFromService,
    IsolateNetwork,
    DisableSensitiveWorkloads,
    RequireManualReview,
}
```

---

# 54. Hard Rule

Quarantine does not erase forensic evidence automatically.

---

# 55. Physical Location

Only infrastructure-level coarse site/rack placement where necessary.

---

# 56. Physical Site

```rust
pub struct PhysicalSiteId(pub String);
```

---

# 57. Rack Location

```rust
pub struct RackLocation {
    pub site: PhysicalSiteId,
    pub rack: RackId,
    pub unit: Option<u16>,
}
```

---

# 58. No Employee Tracking

Hard rule.

---

# 59. Physical Location Privacy

Site metadata access restricted.

---

# 60. Hard rule.

---

# 61. Fleet Inventory

Part 106 integration.

Hardware inventory fields:

```text
asset ID
class
trust state
firmware state
site/rack
lifecycle
owner
warranty
maintenance due
```

---

# 62. No User Assignment By Default

Hard rule.

---

# 63. Fleet Group

```rust
pub struct FleetGroup {
    pub id: FleetGroupId,
    pub class: HardwareClass,
    pub members: BTreeSet<HardwareAssetId>,
}
```

---

# 64. Fleet Groups

Examples:

```text
relay-fleet-eu
build-hosts
hsm-cluster
edge-nodes
```

---

# 65. No Workforce Group

Hard rule.

---

# 66. Fleet Desired State

Part 105.

```rust
pub struct FleetDesiredState {
    pub fleet: FleetGroupId,
    pub firmware_baseline: FirmwareBaselineId,
    pub provisioning_baseline: HardwareProvisioningBaselineId,
    pub maintenance_policy: MaintenancePolicyId,
}
```

---

# 67. Drift

Examples:

```text
wrong firmware
wrong OS image
wrong BMC config
unexpected network config
```

---

# 68. Hard rule.

---

# 69. Maintenance Scheduling

Maintenance should be explicit.

---

# 70. Maintenance Class

```rust
pub enum HardwareMaintenanceClass {
    Routine,
    Preventive,
    Security,
    Repair,
    Replacement,
    Emergency,
}
```

---

# 71. Maintenance Window

```rust
pub struct HardwareMaintenanceWindow {
    pub asset: HardwareAssetId,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
    pub class: HardwareMaintenanceClass,
}
```

---

# 72. Availability-Aware Scheduling

Part 110.

---

# 73. Hard Rule

Do not maintain multiple members of same redundancy domain simultaneously if it breaks resilience policy.

---

# 74. Maintenance Planner

```rust
pub trait HardwareMaintenancePlanner {
    fn plan(
        &self,
        fleet: FleetGroupId,
        request: MaintenanceRequest,
    ) -> Result<MaintenancePlan, HardwareFleetError>;
}
```

---

# 75. Planner Inputs

```text
redundancy
capacity
SLO
firmware urgency
site access
spares
```

---

# 76. No User Traffic Pattern Required

Hard rule.

---

# 77. Maintenance Batch

```rust
pub struct MaintenanceBatch {
    pub assets: Vec<HardwareAssetId>,
    pub max_concurrent: u16,
}
```

---

# 78. Bounded Concurrent Maintenance

Hard rule.

---

# 79. Drain Before Maintenance

Workloads drained where required.

---

# 80. Stateful Nodes

Need:

```text
replication healthy
quorum safe
checkpoint/backup current
```

---

# 81. Hard rule.

---

# 82. Firmware Maintenance

Sequence:

```text
verify artifact
drain asset
snapshot state if needed
apply firmware
reboot
attest
run health checks
return to service
```

---

# 83. Hard rule.

---

# 84. Emergency Firmware Response

Urgent vulnerability may shorten window.

---

# 85. Still requires:

```text
artifact verification
redundancy safety
post-update attestation
```

---

# 86. Hard rule.

---

# 87. Preventive Maintenance

Track:

```text
drive wear
fan status
thermal events
power supply health
memory errors
```

---

# 88. Technical Signals Only

Hard rule.

---

# 89. Predictive Maintenance

May use hardware telemetry.

---

# 90. No employee/user data.

---

# 91. Hard rule.

---

# 92. Hardware Health

```rust
pub enum HardwareHealthState {
    Healthy,
    Warning,
    Degraded,
    Critical,
    Unknown,
}
```

---

# 93. Unknown ≠ Healthy

Hard rule.

---

# 94. Hardware Health Evidence

```rust
pub struct HardwareHealthEvidence {
    pub asset: HardwareAssetId,
    pub state: HardwareHealthState,
    pub signals: Vec<HardwareSignal>,
}
```

---

# 95. Signal Examples

```text
SMART
ECC error count
temperature
fan speed
power supply state
BMC health
```

---

# 96. Privacy-Safe

Infrastructure only.

---

# 97. Spare Inventory

Need ready replacements.

---

# 98. Spare Class

```rust
pub enum SpareClass {
    ColdSpare,
    WarmSpare,
    HotSpare,
}
```

---

# 99. Spare Record

```rust
pub struct SpareHardware {
    pub asset: HardwareAssetId,
    pub class: SpareClass,
    pub compatibility: HardwareCompatibilityProfile,
    pub readiness: SpareReadinessState,
}
```

---

# 100. Spare Readiness

```rust
pub enum SpareReadinessState {
    Ready,
    NeedsProvisioning,
    NeedsFirmwareUpdate,
    NeedsValidation,
    Unusable,
}
```

---

# 101. Hot Spare Must Be Tested

Hard rule.

---

# 102. Spare Rotation

Prevent shelf rot.

---

# 103. Periodic power-on/attestation where safe.

---

# 104. Hard rule.

---

# 105. Component Replacement

Examples:

```text
drive
NIC
RAM
power supply
fan
HSM module
```

---

# 106. Replacement Record

```rust
pub struct ComponentReplacement {
    pub asset: HardwareAssetId,
    pub component: ComponentClass,
    pub old_component_ref: Option<ComponentRef>,
    pub new_component_ref: ComponentRef,
}
```

---

# 107. Trust Reset

Some replacements require re-enrollment/re-attestation.

---

# 108. Hard rule.

---

# 109. TPM/Motherboard Replacement

Creates new trust identity.

---

# 110. Do not silently retain old identity.

---

# 111. Hard rule.

---

# 112. Drive Replacement

Requires:

```text
data replication verified
old drive sanitization
new drive provisioning
```

---

# 113. Hard rule.

---

# 114. RMA

Returned hardware must be sanitized according to data class.

---

# 115. RMA State

```rust
pub enum RmaState {
    Approved,
    Sanitizing,
    ReadyForShipment,
    Shipped,
    VendorReceived,
    Closed,
}
```

---

# 116. No Shipment Before Sanitization Evidence

Hard rule.

---

# 117. RMA Exception

If drive physically failed and cannot be sanitized:

```text
retain
destroy
contractual secure destruction
```

based on policy.

---

# 118. Hard rule.

---

# 119. Hardware Sanitization

Part 67/114.

Methods:

```text
crypto erasure
secure erase
key destruction
physical destruction
```

---

# 120. Sanitization Evidence

```rust
pub struct SanitizationReceipt {
    pub asset: HardwareAssetId,
    pub method: SanitizationMethod,
    pub completed_at: Timestamp,
    pub evidence_digest: Digest,
}
```

---

# 121. No Reuse/Disposal Without Required Receipt

Hard rule.

---

# 122. Physical Access Governance

Physical access is privileged.

---

# 123. Access Scope

```rust
pub enum PhysicalAccessScope {
    Site(PhysicalSiteId),
    Rack(RackId),
    Asset(HardwareAssetId),
}
```

---

# 124. Access Grant

```rust
pub struct PhysicalAccessGrant {
    pub scope: PhysicalAccessScope,
    pub purpose: PhysicalAccessPurpose,
    pub expires_at: Timestamp,
}
```

---

# 125. Purpose

```rust
pub enum PhysicalAccessPurpose {
    Maintenance,
    Installation,
    Repair,
    Audit,
    Emergency,
}
```

---

# 126. Time-Bounded

Hard rule.

---

# 127. No Permanent "Datacenter Superuser"

Preferred hard rule.

---

# 128. Two-Person Rule

For high-value assets:

```text
HSM
root signing appliance
governance authority
```

---

# 129. Hard rule.

---

# 130. Physical Access Audit

Record access authorization and maintenance result.

---

# 131. Do Not Build Continuous Worker Movement History

Hard rule.

---

# 132. Badge System Boundary

External facility system may validate access.

---

# 133. SIAR stores minimum evidence.

---

# 134. Hard rule.

---

# 135. Camera/Biometric Boundary

Not part of core architecture.

---

# 136. Hard rule.

---

# 137. BMC Management

BMC is high-risk.

---

# 138. BMC Network

Separate management network.

---

# 139. Hard rule.

---

# 140. BMC Authentication

Strong hardware/workforce auth.

---

# 141. No shared passwords.

---

# 142. Hard rule.

---

# 143. BMC Firmware

Governed like other firmware.

---

# 144. Hard rule.

---

# 145. Out-of-Band Management

Useful for recovery.

---

# 146. Must not become alternate ungoverned admin path.

---

# 147. Hard rule.

---

# 148. Remote Console

JIT access only.

---

# 149. Session expires.

---

# 150. Hard rule.

---

# 151. Network Appliance Fleet

Routers/switches/load balancers have:

```text
firmware
config baseline
boot state
physical placement
```

---

# 152. Hard rule.

---

# 153. HSM Appliance Fleet

Special rules:

```text
dual control
tamper evidence
firmware allowlist
backup key custody
secure ceremony
```

---

# 154. Part 72.

---

# 155. Hard rule.

---

# 156. Embedded Node Fleet

Part 20.

---

# 157. Edge/embedded nodes may have intermittent connectivity.

---

# 158. Use signed offline bundles.

---

# 159. Hard rule.

---

# 160. Embedded Firmware Update

A/B slots where possible.

---

# 161. Rollback protection.

---

# 162. Hard rule.

---

# 163. Air-Gapped Hardware

Provision via signed offline media.

---

# 164. Track media chain of custody.

---

# 165. Hard rule.

---

# 166. Supply-Chain Integration

Part 73.

Hardware acquisition should track:

```text
vendor
model
firmware origin
tamper state
shipping provenance where needed
```

---

# 167. No unsupported counterfeit/unknown hardware in high-trust fleet.

---

# 168. Hard rule.

---

# 169. Supply-Chain Verification

High-value assets may require:

```text
sealed packaging validation
serial verification
firmware reflash
TPM reset/enrollment
```

---

# 170. Hard rule.

---

# 171. Hardware Provenance

```rust
pub struct HardwareProvenance {
    pub asset: HardwareAssetId,
    pub vendor: VendorId,
    pub model: HardwareModelId,
    pub procurement_ref: ProcurementRef,
    pub provenance_confidence: ProvenanceConfidence,
}
```

---

# 172. Provenance Confidence

```rust
pub enum ProvenanceConfidence {
    Declared,
    Verified,
    HighAssurance,
}
```

---

# 173. No False Assurance

Hard rule.

---

# 174. Maintenance Dependency Graph

Part 106.

Maintenance planner uses:

```text
asset dependencies
quorum
storage replication
network topology
fault domains
```

---

# 175. Hard rule.

---

# 176. Maintenance and SLOs

Part 109.

Planned maintenance accounting policy explicit.

---

# 177. No retroactive exclusion.

---

# 178. Hard rule.

---

# 179. Maintenance and Resilience

Part 110.

Never reduce below required redundancy unless emergency governance explicitly accepts degraded mode.

---

# 180. Hard rule.

---

# 181. Capacity Integration

Part 101/111.

Maintenance may temporarily reduce capacity.

---

# 182. Check headroom before drain.

---

# 183. Hard rule.

---

# 184. Performance Integration

Part 112.

Maintenance state may affect latency/throughput.

---

# 185. Validate after return to service.

---

# 186. Hard rule.

---

# 187. Efficiency/Sustainability Integration

Parts 113–114.

Older hardware may remain if:

```text
secure
reliable
efficient enough
supported
```

---

# 188. Otherwise retire/repurpose.

---

# 189. Hard rule.

---

# 190. Geographic Governance

Part 103.

Physical asset placement must respect region/jurisdiction policy.

---

# 191. Hard rule.

---

# 192. Organizational Governance

Part 104.

Hardware/maintenance owners explicit.

---

# 193. Physical access and firmware approval roles separate where high risk.

---

# 194. Hard rule.

---

# 195. Desired-State Integration

Part 105.

Fleet baseline signed/versioned.

---

# 196. Hard rule.

---

# 197. Inventory Integration

Part 106.

Hardware assets are first-class inventory nodes.

---

# 198. Hard rule.

---

# 199. Change Integration

Part 107.

Firmware updates, component replacement, fleet migration, and maintenance are governed changes.

---

# 200. Hard rule.

---

# 201. Release Integration

Part 108.

Hardware/firmware prerequisites may block software release if incompatible.

---

# 202. Hard rule.

---

# 203. Vulnerability Integration

Part 98.

Firmware vulnerabilities map to affected assets.

---

# 204. Query:

```text
Which active assets run vulnerable firmware X?
```

---

# 205. Hard rule.

---

# 206. Update Integration

Part 99.

Firmware artifacts use signed release channels.

---

# 207. Hard rule.

---

# 208. Incident Integration

Part 96.

Hardware tamper/attestation failure can trigger incident.

---

# 209. Hard rule.

---

# 210. SOC Integration

Part 97.

Hardware indicators:

```text
unexpected firmware
BMC compromise
boot measurement change
tamper alert
```

---

# 211. No user data.

---

# 212. Hard rule.

---

# 213. Compliance Integration

Part 95.

Controls can require:

```text
firmware current
secure boot enabled
physical access scoped
retired media sanitized
```

---

# 214. Good.

---

# 215. Hardware Fleet API

```rust
pub trait HardwareFleetService {
    fn asset(
        &self,
        id: HardwareAssetId,
    ) -> Result<HardwareAsset, HardwareFleetError>;

    fn fleet(
        &self,
        id: FleetGroupId,
    ) -> Result<FleetGroup, HardwareFleetError>;
}
```

---

# 216. Enrollment API

```rust
pub trait HardwareEnrollmentService {
    fn enroll(
        &self,
        request: HardwareEnrollmentRequest,
    ) -> Result<HardwareEnrollmentReceipt, HardwareFleetError>;
}
```

---

# 217. Firmware Service

```rust
pub trait FirmwareLifecycleService {
    fn compliance(
        &self,
        asset: HardwareAssetId,
    ) -> Result<FirmwareComplianceState, HardwareFleetError>;

    fn plan_update(
        &self,
        asset: HardwareAssetId,
        target: FirmwareBaselineId,
    ) -> Result<FirmwareUpdatePlan, HardwareFleetError>;
}
```

---

# 218. Maintenance Service

```rust
pub trait FleetMaintenanceService {
    fn plan(
        &self,
        request: MaintenanceRequest,
    ) -> Result<MaintenancePlan, HardwareFleetError>;

    fn complete(
        &self,
        plan: MaintenancePlanId,
        evidence: MaintenanceEvidence,
    ) -> Result<MaintenanceReceipt, HardwareFleetError>;
}
```

---

# 219. Sanitization Service

```rust
pub trait HardwareSanitizationService {
    fn sanitize(
        &self,
        asset: HardwareAssetId,
        method: SanitizationMethod,
    ) -> Result<SanitizationReceipt, HardwareFleetError>;
}
```

---

# 220. No Generic Remote Shell API

Hard rule.

---

# 221. Error Taxonomy

```rust
pub enum HardwareFleetError {
    AssetUnknown,
    EnrollmentInvalid,
    FirmwareUntrusted,
    FirmwareRollbackBlocked,
    AttestationFailed,
    MaintenanceUnsafe,
    RedundancyInsufficient,
    SpareUnavailable,
    SanitizationRequired,
    PhysicalAccessDenied,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 222. Observability

Safe metrics:

```text
fleet size
firmware compliance
hardware health
maintenance due
spare readiness
attestation failures
```

---

# 223. Forbidden:

```text
employee movement tracking
user-device enumeration
private content
```

---

# 224. Hard rule.

---

# 225. Fleet SLOs

Examples:

```text
100% Tier0 assets attested
critical firmware compliance within deadline
no maintenance below redundancy floor
sanitization evidence before disposal
```

---

# 226. Hard rule.

---

# 227. Security SLO

```text
0 unsupported critical firmware on trusted Tier0 assets
0 unenrolled asset admitted to high-trust workload
0 firmware rollback below security epoch
```

---

# 228. Privacy SLO

```text
0 workforce movement profiling
0 personal-device inventory expansion
```

---

# 229. Failure Modes

```text
firmware update failure
BMC compromise
spare unavailable
maintenance causes quorum loss
attestation mismatch
```

---

# 230. Firmware Update Failure

Asset remains drained/quarantined until safe recovery.

---

# 231. No blind retry loop.

---

# 232. Hard rule.

---

# 233. BMC Compromise

Treat as high-severity host trust failure.

---

# 234. Rebuild/re-enroll preferred.

---

# 235. Hard rule.

---

# 236. Spare Unavailable

Do not violate redundancy policy to continue maintenance.

---

# 237. Defer maintenance unless security urgency demands governed degraded mode.

---

# 238. Hard rule.

---

# 239. Quorum Risk

Maintenance planner blocks action.

---

# 240. Hard rule.

---

# 241. Attestation Mismatch

Quarantine and investigate.

---

# 242. Hard rule.

---

# 243. Testing

Need hardware-fleet testkit.

---

# 244. Test Scenarios

```text
new server enrollment
firmware update
drive replacement
HSM maintenance
RMA
```

---

# 245. Enrollment Test

Unverified serial alone cannot enroll Trusted asset.

---

# 246. Firmware Test

Unapproved firmware rejected.

---

# 247. Anti-Rollback Test

Old firmware below epoch blocked.

---

# 248. Attestation Test

Unexpected measurement quarantines asset.

---

# 249. Maintenance Test

Planner prevents simultaneous redundant-node maintenance.

---

# 250. Quorum Test

Maintenance cannot reduce voters below safe quorum.

---

# 251. Capacity Test

Drain refused when headroom insufficient.

---

# 252. Spare Test

Warm/hot spare readiness validated.

---

# 253. Component Test

Motherboard replacement forces trust re-enrollment.

---

# 254. Drive Test

Old drive sanitization receipt required.

---

# 255. RMA Test

Shipment blocked before sanitization evidence.

---

# 256. Physical Access Test

Expired grant rejected.

---

# 257. BMC Test

Management plane cannot bypass scoped authorization.

---

# 258. Privacy Test

No continuous employee-location history generated.

---

# 259. Embedded Test

A/B firmware rollback respects security epoch.

---

# 260. Air-Gap Test

Signed offline provisioning accepted; unsigned media rejected.

---

# 261. Fuzzing

Fuzz:

```text
firmware manifests
hardware enrollment evidence
maintenance plans
lifecycle transitions
sanitization receipts
```

---

# 262. Property Tests

Properties:

```text
unenrolled hardware can never become Trusted without valid enrollment
firmware below accepted security epoch can never become compliant
maintenance plan can never intentionally violate declared redundancy floor
hardware reuse can never complete without required sanitization evidence
```

---

# 263. Formal Verification Targets

Strong candidates:

```text
hardware trust state machine
firmware update/rollback state machine
maintenance concurrency constraints
sanitization-before-reuse lifecycle
```

---

# 264. Kani Candidate

lifecycle/firmware-epoch/maintenance invariants.

---

# 265. TLA+ Candidate

enroll → provision → attest → operate → maintain → re-attest → retire.

---

# 266. Loom Candidate

concurrent maintenance scheduling + asset health change + drain completion.

---

# 267. Performance

Fleet governance is off request path.

---

# 268. Attestation refresh may be periodic/event-driven.

---

# 269. No request-path dependency on central fleet DB.

---

# 270. Hard rule.

---

# 271. Storage

Separate:

```text
hardware inventory
firmware baselines
attestation summaries
maintenance plans
spare inventory
sanitization receipts
physical access grants
```

---

# 272. No workforce tracking warehouse.

---

# 273. Hard rule.

---

# 274. Partitioning

By:

```text
fleet
hardware class
region
site
asset
tenant managed scope
```

---

# 275. No user partition.

---

# 276. Hard rule.

---

# 277. Crate Layout

Recommended:

```text
crates/
├── siar-hardware-core/
├── siar-hardware-enrollment/
├── siar-firmware-lifecycle/
├── siar-secure-boot/
├── siar-hardware-attestation/
├── siar-fleet-maintenance/
├── siar-spare-inventory/
├── siar-physical-access/
├── siar-hardware-sanitization/
├── siar-hardware-observability/
└── siar-hardware-testkit/
```

---

# 278. `siar-hardware-core`

Owns:

```text
HardwareAssetId
HardwareClass
HardwareTrustState
errors
```

---

# 279. `siar-hardware-enrollment`

Enrollment/provenance/trust bootstrap.

---

# 280. `siar-firmware-lifecycle`

Firmware baselines, epochs, update plans.

---

# 281. `siar-secure-boot`

Boot policy/measurement validation.

---

# 282. `siar-hardware-attestation`

TPM/measured-boot evidence.

---

# 283. `siar-fleet-maintenance`

Maintenance windows/batches/drain safety.

---

# 284. `siar-spare-inventory`

Cold/warm/hot spare readiness.

---

# 285. `siar-physical-access`

Scoped/time-bounded physical maintenance authorization.

---

# 286. `siar-hardware-sanitization`

Erase/reuse/RMA/disposal receipts.

---

# 287. `siar-hardware-observability`

Fleet technical health only.

---

# 288. `siar-hardware-testkit`

enrollment/firmware/maintenance/privacy tests.

---

# 289. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Every production hardware asset has an opaque canonical identity, lifecycle state, trust state, provenance, firmware baseline, and ownership scope; external serials alone are never authorization identity.
2. Unenrolled or unverified hardware cannot host high-trust workloads, and no hardware can transition directly from unknown/unenrolled state to Trusted without provisioning and attestation.
3. Firmware artifacts are signed/hash-pinned/provenance-checked, governed by monotonic security epochs, and cannot roll back to known-vulnerable versions merely because rollback is operationally convenient.
4. Firmware or hardware maintenance completion requires post-change health validation and re-attestation where relevant; command completion/reboot alone is insufficient.
5. Maintenance scheduling respects quorum, fault-domain diversity, capacity headroom, SLOs, and recovery requirements and cannot take redundant members offline concurrently beyond declared safety limits.
6. BMC, remote console, out-of-band management, and physical access remain scoped privileged paths with strong authentication, time bounds, audit, and no permanent universal superuser.
7. Component replacement that changes hardware trust identity—especially motherboard/TPM/HSM changes—requires explicit trust re-establishment rather than inheriting the old identity silently.
8. Reuse, RMA, resale, donation, or disposal of storage-bearing hardware requires sanitization/cryptographic-erasure evidence appropriate to the data class before the asset leaves its prior trust boundary.
9. Hardware health, firmware, site/rack, and maintenance data are infrastructure telemetry and cannot become employee movement tracking, personal-device enumeration, user profiling, or private-content observation.
10. Sustainability or cost pressure can never keep unsupported/insecure hardware in critical production, and maintenance urgency cannot bypass security, firmware-integrity, or resilience gates.
11. Air-gapped, embedded, and edge hardware use signed offline provisioning/update bundles and retain the same anti-rollback and trust-state invariants as continuously connected fleets.
12. Hardware-fleet governance integrates with supply chain, inventory, desired state, vulnerability management, updates, resilience, capacity, release/change governance, sustainability, audit, compliance, SOC, and incident response without creating an alternate unmanaged physical-control plane.
```

---

# 290. Initial Production Scope

Implement first:

```text
typed HardwareAssetId/HardwareClass/HardwareTrustState
hardware enrollment/provenance
fleet groups
provisioning baselines
firmware inventory/baselines
firmware security epochs
secure boot policy
measured-boot attestation
firmware update state machine
quarantine flow
maintenance classes/windows
redundancy-aware maintenance planner
spare inventory/readiness
component replacement records
RMA workflow
sanitization receipts
BMC management boundary
physical-access grants
audit/compliance integration
privacy-safe fleet metrics
hardware-fleet testkit
```

Then add:

```text
predictive hardware maintenance
advanced BMC/vendor adapters
hardware supply-chain attestation
automated spare-demand forecasting
fleet-wide A/B firmware rollout orchestration
formal maintenance/quorum verification
secure offline provisioning ceremonies
```

---

# 291. Definition of Done

Part 115 is complete when:

- hardware identities are canonical and opaque
- enrollment is evidence-based
- trusted state requires provisioning/attestation
- firmware baselines/security epochs exist
- secure/measured boot is integrated
- firmware rollback protection works
- maintenance respects redundancy/quorum/capacity
- component replacement can force re-enrollment
- spare readiness is tracked
- RMA/disposal requires sanitization evidence
- BMC/physical access is scoped and time-bounded
- embedded/air-gapped fleets use signed offline bundles
- no workforce/user surveillance graph is created
- enrollment/firmware/maintenance/privacy/fuzz/formal tests are specified

---

# 292. Final Architecture

```text
                  HARDWARE ACQUISITION
                          │
                          ▼
                   SECURE ENROLLMENT
                          │
                          ▼
               PROVISIONING / BOOT POLICY
                          │
                          ▼
                 FIRMWARE BASELINE
                          │
                          ▼
                    ATTESTED FLEET
                          │
              ┌───────────┼───────────┐
              │           │           │
           PATCH       MAINTAIN     REPAIR
              │           │           │
              └───────────┼───────────┘
                          ▼
                    RE-ATTESTATION
                          │
                          ▼
              REUSE / RETIRE / SANITIZE
```

Hardware-governance safety model:

```text
canonical hardware identity
+
secure enrollment
+
signed firmware baselines
+
anti-rollback epochs
+
measured boot
+
resilience-aware maintenance
+
scoped physical/BMC access
+
sanitization before reuse/disposal
```

not:

```text
trust a serial number, patch firmware ad hoc, keep permanent BMC admin credentials, and return failed drives without verified sanitization
```

---

# 293. Final Principle

Physical infrastructure should be treated as a cryptographically governed lifecycle, not a collection of boxes that happen to run software.

The correct model is:

```text
identify hardware canonically
+
enroll with evidence
+
provision deterministically
+
measure and attest boot state
+
govern firmware versions
+
maintain without breaking redundancy
+
re-establish trust after sensitive replacement
+
sanitize before reuse or disposal
+
never turn fleet operations into people tracking
```

This architecture gives SIAR a privacy-preserving hardware-fleet foundation for secure provisioning, firmware lifecycle, measured boot, maintenance scheduling, spare management, component replacement, BMC governance, physical access, RMA, and safe retirement while preserving the anonymity, local-first, least-authority, resilience, sustainability, and anti-surveillance guarantees established across Parts 34–114.
