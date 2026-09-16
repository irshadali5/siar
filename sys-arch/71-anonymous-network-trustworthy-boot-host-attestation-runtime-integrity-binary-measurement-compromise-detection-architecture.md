# Core System Architecture Part 71 — Anonymous Network Trustworthy Boot, Host Attestation, Runtime Integrity, Binary Measurement & Compromise Detection Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 71  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 28, 51–53, 61–70  

**Primary purpose:** define SIAR's trustworthy-boot, measured-boot, host-attestation, runtime-integrity, binary/configuration measurement, compromise detection, quarantine, anti-rollback, host identity, and node re-enrollment architecture for infrastructure systems while preserving anonymity and preventing attestation from becoming a client fingerprinting mechanism.

---

# 1. Purpose

Infrastructure nodes can be compromised even when application code is memory-safe.

Threats include:

```text
modified bootloader
tampered kernel
patched binary
stale vulnerable image
rootkit
unexpected runtime module
stolen node credential
```

The governing principle is:

> **SIAR infrastructure should be able to prove what it booted and what it is running, detect meaningful divergence from trusted state, and quarantine compromised nodes without requiring global hardware attestation for end-user devices.**

---

# 2. Architectural Position

```text
Firmware / Bootloader
        │
        ▼
Measured / Verified Boot
        │
        ▼
Host Measurement Register
        │
        ▼
Artifact + Config Measurement
        │
        ▼
Attestation Evidence
        │
        ▼
Verifier
        │
        ▼
Allow / Degrade / Quarantine
```

---

# 3. Core Separation

Keep distinct:

```text
boot trust
host identity
artifact identity
config identity
runtime integrity
service identity
user identity
```

---

# 4. Non-Goals

Part 71 does not:

```text
require TPM on every client device
treat attestation as proof a service is bug-free
make one hardware vendor mandatory
allow remote attestation to identify users
```

---

# 5. Trustworthy Boot Classes

```rust
pub enum BootTrustClass {
    Unmeasured,
    Measured,
    Verified,
    Attested,
}
```

---

# 6. Unmeasured

No cryptographic boot evidence.

---

# 7. Measured

Components are hashed/measured.

---

# 8. Verified

Boot chain enforces signatures.

---

# 9. Attested

Remote verifier receives signed measurement evidence.

---

# 10. Recommended Infrastructure Baseline

```text
Verified + Measured
```

for critical nodes.

---

# 11. Tier0 Recommendation

```text
Verified + Measured + Attested
```

where hardware/platform supports it.

---

# 12. Boot Chain

Possible chain:

```text
firmware
→ bootloader
→ kernel
→ initramfs
→ OS image
→ SIAR service artifact
```

---

# 13. Measurement Record

```rust
pub struct BootMeasurement {
    pub component: BootComponent,
    pub digest: ContentDigest,
}
```

---

# 14. Boot Component

```rust
pub enum BootComponent {
    Firmware,
    Bootloader,
    Kernel,
    Initramfs,
    OsImage,
    ServiceArtifact,
}
```

---

# 15. Verified Boot

Each trusted stage verifies next.

---

# 16. Secure Boot

Platform implementation option.

---

# 17. Hard Rule

Unsigned boot component cannot enter production trust chain.

---

# 18. Measured Boot

Measurements recorded in TPM-like PCRs or equivalent.

---

# 19. TPM

Preferred infrastructure primitive where available.

---

# 20. TPM Roles

```text
measurement
attestation key
sealed secret support
```

---

# 21. HSM

Different role.

---

# 22. TPM ≠ HSM

Hard rule conceptually.

---

# 23. TPM

Host-bound trust.

---

# 24. HSM

High-value key custody.

---

# 25. Attestation Key

Dedicated.

---

# 26. No Reuse Of Provider Signing Key

Hard rule.

---

# 27. Host Attestation Identity

```rust
pub struct HostAttestationId(pub [u8; 32]);
```

---

# 28. Infrastructure Only

Must not be exposed to end users.

---

# 29. Attestation Evidence

```rust
pub struct AttestationEvidence {
    pub host: HostAttestationId,
    pub boot_measurements: Vec<BootMeasurement>,
    pub artifact_digest: BuildHash,
    pub config_digest: ConfigDigest,
    pub nonce: AttestationNonce,
    pub signature: AttestationSignature,
}
```

---

# 30. Nonce

Prevents replay.

---

# 31. Attestation Freshness

Short-lived.

---

# 32. No Static Evidence Reuse

Hard rule.

---

# 33. Attestation Verifier

```rust
pub trait AttestationVerifier {
    fn verify(
        &self,
        evidence: &AttestationEvidence,
        policy: &AttestationPolicy,
    ) -> Result<AttestationDecision, AttestationError>;
}
```

---

# 34. Decision

```rust
pub enum AttestationDecision {
    Trusted,
    Degraded,
    Quarantine,
}
```

---

# 35. Trusted

Measurements match allowed state.

---

# 36. Degraded

Known non-critical variance.

---

# 37. Quarantine

Unsafe/unknown critical divergence.

---

# 38. Attestation Policy

```rust
pub struct AttestationPolicy {
    pub allowed_boot_profiles: BTreeSet<BootProfileId>,
    pub allowed_artifacts: BTreeSet<BuildHash>,
    pub allowed_config_versions: BTreeSet<RuntimePolicyVersion>,
}
```

---

# 39. Boot Profile

Versioned allowed measurement set.

---

# 40. No One Hardcoded PCR Set Forever

Hard rule.

---

# 41. Profile Lifecycle

```text
proposed
→ approved
→ active
→ retiring
→ blocked
```

---

# 42. Boot Profile Version

```rust
pub struct BootProfileVersion(pub u64);
```

---

# 43. Anti-Rollback

Older vulnerable boot profile must not regain trust.

---

# 44. Hard rule.

---

# 45. Artifact Measurement

Part 62 exact artifact digest.

---

# 46. Runtime Service Measurement

Compare loaded executable to expected digest.

---

# 47. Config Measurement

Part 61 signed config digest.

---

# 48. No Config Blind Spot

Hard rule.

---

# 49. Runtime Measurement Bundle

```rust
pub struct RuntimeMeasurementBundle {
    pub artifact: BuildHash,
    pub config: ConfigDigest,
    pub policy_version: RuntimePolicyVersion,
    pub environment: DeploymentEnvironment,
}
```

---

# 50. Service Identity Binding

Service credential should bind to trusted host state where appropriate.

---

# 51. Sealed Credential

Optional.

---

# 52. TPM Sealing

Can release secret only when measurement policy matches.

---

# 53. Use Case

```text
node runtime credential
```

---

# 54. Caution

Overly strict sealing can make recovery difficult.

---

# 55. Recommended

Seal selected infrastructure credentials only.

---

# 56. Never Seal User Data Keys To Host

Hard rule.

---

# 57. Why

User data should remain portable/recoverable.

---

# 58. Runtime Integrity

Boot state alone is insufficient.

---

# 59. Runtime Threats

```text
binary patch
LD_PRELOAD
unexpected library
kernel module
debugger injection
memory tampering
```

---

# 60. Runtime Integrity Classes

```rust
pub enum RuntimeIntegrityClass {
    StaticMeasured,
    ContinuousMeasured,
    Behavioral,
}
```

---

# 61. StaticMeasured

Measure executable/config at startup.

---

# 62. ContinuousMeasured

Periodically validate expected state.

---

# 63. Behavioral

Detect anomalies from safe system signals.

---

# 64. Recommended Baseline

Static + selected continuous checks.

---

# 65. File Integrity

Monitor:

```text
binary
config
service unit
critical libraries
```

---

# 66. Measurement Manifest

```rust
pub struct IntegrityManifest {
    pub files: BTreeMap<PathBuf, ContentDigest>,
}
```

---

# 67. No User Data Paths

Hard rule.

---

# 68. Runtime Verifier

```rust
pub trait RuntimeIntegrityVerifier {
    fn verify(
        &self,
        manifest: &IntegrityManifest,
    ) -> Result<RuntimeIntegrityReport, IntegrityError>;
}
```

---

# 69. Runtime Integrity Report

```rust
pub struct RuntimeIntegrityReport {
    pub status: IntegrityStatus,
    pub deviations: Vec<IntegrityDeviation>,
}
```

---

# 70. Integrity Status

```rust
pub enum IntegrityStatus {
    Clean,
    Warning,
    Compromised,
    Unknown,
}
```

---

# 71. Unknown

Critical services may quarantine.

---

# 72. Root Filesystem

Read-only preferred.

---

# 73. Immutable OS

Part 62.

---

# 74. Advantage

Reduces runtime drift.

---

# 75. Kernel Module Policy

Restrict.

---

# 76. Module Allowlist

Possible.

---

# 77. No Arbitrary Module Load

Hard rule for critical hosts.

---

# 78. eBPF

Useful but powerful.

---

# 79. eBPF Program Policy

Signed/allowlisted for production.

---

# 80. No Arbitrary Debug eBPF On Tier0

Hard rule.

---

# 81. Dynamic Linker

Control.

---

# 82. `LD_PRELOAD`

Blocked in production service environment.

---

# 83. Debugger Attach

Restricted.

---

# 84. `ptrace`

Disable/restrict for production daemons.

---

# 85. Core Dumps

Disabled/restricted.

---

# 86. Process Privilege

Least privilege.

---

# 87. No Root Service

Hard rule.

---

# 88. Linux Security Controls

Possible:

```text
seccomp
Landlock
SELinux/AppArmor
cgroups
namespaces
```

---

# 89. Mandatory Access Control

Recommended for critical hosts.

---

# 90. Runtime Policy

Service-specific.

---

# 91. Runtime Security Profile

```rust
pub struct RuntimeSecurityProfile {
    pub seccomp_profile: Option<SeccompProfileId>,
    pub filesystem_policy: FilesystemPolicy,
    pub network_policy: NetworkPolicy,
}
```

---

# 92. Integrity Includes Security Profile

Yes.

---

# 93. Security Profile Digest

Measured.

---

# 94. Host Identity

Separate from service identity.

---

# 95. Host Identity Lifecycle

```rust
pub enum HostIdentityState {
    Enrolling,
    Active,
    Suspected,
    Quarantined,
    Retired,
    Revoked,
}
```

---

# 96. Host Enrollment

Part 61/62.

---

# 97. Enrollment Requirements

```text
approved image
attestation
node identity
config
```

---

# 98. No Auto-Trust New Host

Hard rule.

---

# 99. Host Re-Enrollment

Required after major compromise.

---

# 100. Fresh Host Identity

Recommended after confirmed compromise.

---

# 101. No Credential Reuse

Hard rule.

---

# 102. Compromise Detection

Sources:

```text
attestation failure
integrity deviation
unexpected config
network anomaly
operator report
security alert
```

---

# 103. Compromise Signal

```rust
pub enum CompromiseSignal {
    BootMismatch,
    ArtifactMismatch,
    ConfigMismatch,
    RuntimeMutation,
    CredentialMisuse,
    UnexpectedKernelState,
    AttestationReplay,
}
```

---

# 104. Signal Confidence

```rust
pub enum SignalConfidence {
    Low,
    Medium,
    High,
    Critical,
}
```

---

# 105. Compromise Evaluator

```rust
pub trait CompromiseEvaluator {
    fn evaluate(
        &self,
        signals: &[CompromiseSignalRecord],
    ) -> CompromiseDecision;
}
```

---

# 106. Decision

```rust
pub enum CompromiseDecision {
    Continue,
    Degrade,
    Investigate,
    Quarantine,
}
```

---

# 107. Quarantine

Remove node from serving.

---

# 108. Quarantine Flow

```text
stop new traffic
drain if safe
remove from directory/catalog
revoke runtime credential
preserve evidence
```

---

# 109. No Immediate Evidence Destruction

Hard rule.

---

# 110. But

Secrets may need rotation.

---

# 111. Quarantine Scope

Node-specific.

---

# 112. No Whole-Fleet Shutdown For One Node

Hard rule unless systemic compromise.

---

# 113. Systemic Compromise

Examples:

```text
release key compromise
common image compromise
```

---

# 114. Then

Fleet-wide action may be required.

---

# 115. Directory Integration

Quarantined node removed from topology.

---

# 116. Mix Node

Stop new route selection.

---

# 117. Mailbox

Stop new deposits.

---

# 118. Relay

Stop new sessions.

---

# 119. Provider Catalog

Mark unavailable/revoked.

---

# 120. Federation

Peer can distrust compromised provider/node.

---

# 121. Quarantine Propagation

Signed control-plane state.

---

# 122. No User-Visible Host Identity

Hard rule.

---

# 123. Attestation Privacy

Important.

---

# 124. Infrastructure Attestation

Acceptable.

---

# 125. End-User Device Attestation

High privacy risk.

---

# 126. Default

Do not require.

---

# 127. Why

Hardware attestation can reveal:

```text
device model
hardware identity
vendor
security state
```

---

# 128. Strict Anonymity

No global device attestation.

---

# 129. Managed Enterprise Device

Part 69 may allow scoped attestation.

---

# 130. But

Only managed profile/organization scope.

---

# 131. No Cross-Service Attestation Identifier

Hard rule.

---

# 132. Attestation Pseudonym

If needed for managed devices.

---

# 133. Scope-specific.

---

# 134. Hardware Attestation Provider

Potential vendor dependency.

---

# 135. No Single Vendor Requirement

Hard rule.

---

# 136. Software Attestation

Weaker, but can provide integrity hints.

---

# 137. Trust Level

Explicit.

---

# 138. Attestation Assurance

```rust
pub enum AttestationAssurance {
    None,
    Software,
    TpmBacked,
    HardwareVerified,
}
```

---

# 139. Policy Can Require Level

Per node class.

---

# 140. Example

Tier0:

```text
TpmBacked
```

---

# 141. Self-Hosted Small Deployment

May use:

```text
Software
```

with explicit reduced assurance.

---

# 142. No Misleading Label

Hard rule.

---

# 143. Binary Measurement

Exact hash.

---

# 144. Release Manifest Binding

Part 62.

---

# 145. Measurement Verifier

Checks:

```text
artifact hash
release signature
security epoch
```

---

# 146. Old Artifact

If blocked:

```text
quarantine
```

---

# 147. No Operator Override Of Blocked Artifact

Hard rule.

---

# 148. Config Measurement

Exact effective config digest.

---

# 149. Effective Config

After layering.

---

# 150. Runtime Policy Digest

Measured separately if dynamic.

---

# 151. No Hidden Local Override

Hard rule.

---

# 152. Drift Detection

Part 61.

---

# 153. Attestation Can Confirm Drift state.

---

# 154. Host Package Inventory

Optional.

---

# 155. SBOM Comparison

Can verify expected package set.

---

# 156. No Package Inventory From User Device

Hard rule.

---

# 157. OS Version

Infrastructure only.

---

# 158. Firmware Version

Infrastructure only.

---

# 159. Firmware Update

High-risk.

---

# 160. Update Must Be Signed.

---

# 161. Anti-Rollback Firmware

Use platform support.

---

# 162. Secure Boot Key Management

Separate role.

---

# 163. No Developer Personal Key

Hard rule.

---

# 164. Boot Signing Key

HSM/offline controlled.

---

# 165. Root-of-Trust Key

Rarely used.

---

# 166. Boot Key Rotation

Need rollover.

---

# 167. Dual Trust Window

Possible.

---

# 168. Transition

```text
trust old+new
→ sign new image with new
→ migrate fleet
→ remove old
```

---

# 169. Compromise

Immediate revoke if platform supports.

---

# 170. Recovery Media

Signed.

---

# 171. Recovery Image

Separate artifact.

---

# 172. No Unsigned Rescue Shell

Hard rule.

---

# 173. Maintenance Mode

Explicit.

---

# 174. Maintenance Attestation

Can be distinct profile.

---

# 175. Maintenance Node

Must not serve production traffic.

---

# 176. Hard rule.

---

# 177. Runtime Compromise Detection

Behavioral signals.

---

# 178. Examples

```text
unexpected outbound connection
unexpected file write
privilege escalation
unknown process
```

---

# 179. Detection Agent

Separate low-privilege service.

---

# 180. Agent Trust

Itself measured.

---

# 181. No Full Packet Capture

Hard rule.

---

# 182. Why

Packet capture risks user metadata/content.

---

# 183. Network Integrity Monitoring

Use aggregate/system-level signals.

---

# 184. Example

```text
unexpected destination class
port
connection count
```

---

# 185. File Integrity Monitoring

Critical paths only.

---

# 186. No Monitoring User Data Directory Content

Hard rule.

---

# 187. Runtime Anomaly Detection

Simple rules first.

---

# 188. No Black-Box ML Required

Hard rule.

---

# 189. Why

Explainability.

---

# 190. Behavioral Baseline

Infrastructure only.

---

# 191. Alert Privacy

No user identifiers.

---

# 192. Compromise Incident

Part 65/70.

---

# 193. Incident Response

```text
quarantine
revoke
rebuild
rotate
re-enroll
```

---

# 194. Rebuild Over Repair

Preferred.

---

# 195. Fresh Image

From trusted artifact.

---

# 196. Fresh Runtime Credential

Required.

---

# 197. Fresh Host Identity

Recommended after serious compromise.

---

# 198. Preserve Necessary Evidence

Before destruction.

---

# 199. Evidence Store

Restricted.

---

# 200. No User Data Collection Expansion

Hard rule.

---

# 201. Forensic Image

May include sensitive infrastructure state.

---

# 202. Access tightly controlled.

---

# 203. Cryptographic Key Compromise

Part 66.

---

# 204. If Node Held Service Key

Rotate.

---

# 205. HSM-backed Key

May remain safe even if host compromised.

---

# 206. But

Assume misuse possible.

---

# 207. Audit signing operations.

---

# 208. Attestation Freshness Service

No central user-tracking implications because infrastructure only.

---

# 209. Attestation Challenge

```rust
pub struct AttestationChallenge {
    pub nonce: AttestationNonce,
    pub verifier: VerifierId,
    pub expires_at: Timestamp,
}
```

---

# 210. No Reusable Challenge

Hard rule.

---

# 211. Verifier Identity

Infrastructure authority.

---

# 212. Multi-Verifier

Possible.

---

# 213. Avoid Single Point Of Trust

For critical fleets.

---

# 214. Attestation Policy Authority

Governance/operations.

---

# 215. Policy Change

Signed.

---

# 216. No Silent Allowlist Expansion

Hard rule.

---

# 217. Attestation Transparency

Infrastructure-level.

---

# 218. Could publish approved boot profile digests.

---

# 219. No sensitive topology detail required.

---

# 220. Remote Peer Attestation

Optional.

---

# 221. Example

Federation domain may require gateway attestation.

---

# 222. But

Attestation does not create transitive trust.

---

# 223. Hard rule.

---

# 224. Federation Attestation Policy

```rust
pub struct FederationAttestationPolicy {
    pub minimum_assurance: AttestationAssurance,
    pub accepted_profiles: BTreeSet<BootProfileId>,
}
```

---

# 225. Gateway Attestation

Per peer/domain agreement.

---

# 226. No User Device Attestation Crossing Federation

Hard rule.

---

# 227. Provider Selection

Part 48 can include assurance class.

---

# 228. But

Do not rank only by hardware trust.

---

# 229. Diversity Still matters.

---

# 230. Self-Hosted Node

May not have TPM.

---

# 231. Capability Advertisement

Can state:

```text
Software assurance
```

---

# 232. No exclusion by default unless policy requires stronger.

---

# 233. Host Attestation Cache

Short-lived.

---

# 234. No long-lived stale trust.

---

# 235. Attestation Cache

```rust
pub struct AttestationCacheEntry {
    pub host: HostAttestationId,
    pub decision: AttestationDecision,
    pub valid_until: Timestamp,
}
```

---

# 236. Revocation Overrides Cache

Hard rule.

---

# 237. Compromised Host Revocation

Signed.

---

# 238. Host Revocation Record

```rust
pub struct HostRevocation {
    pub host: HostAttestationId,
    pub effective_at: Timestamp,
    pub reason: HostRevocationReason,
}
```

---

# 239. Revocation Reason

```rust
pub enum HostRevocationReason {
    Compromised,
    Retired,
    MeasurementFailure,
    CredentialLeak,
}
```

---

# 240. Host Rejoin

Only after re-enrollment.

---

# 241. Runtime Health vs Integrity

Different.

---

# 242. Healthy

Can still be compromised.

---

# 243. Integrity Clean

Can still be overloaded.

---

# 244. Separate status.

---

# 245. Node Trust State

```rust
pub struct NodeTrustState {
    pub health: HealthState,
    pub integrity: IntegrityStatus,
    pub attestation: AttestationDecision,
}
```

---

# 246. Routing Eligibility

Depends on all relevant states.

---

# 247. Example

Healthy but Quarantine attestation:

```text
not eligible
```

---

# 248. Directory Eligibility

```rust
pub trait NodeEligibilityEvaluator {
    fn eligible(
        &self,
        state: &NodeTrustState,
        policy: &NodeEligibilityPolicy,
    ) -> bool;
}
```

---

# 249. No Performance Override

Hard rule.

---

# 250. Compromise Scoring

Avoid opaque global score.

---

# 251. Prefer explicit signals/policies.

---

# 252. No User Reputation Coupling

Hard rule.

---

# 253. Attestation Observability

Safe metrics:

```text
trusted hosts
degraded hosts
quarantined hosts
profile mismatch count
```

---

# 254. Forbidden Metrics

No:

```text
user-to-host mapping
contact activity
message count per attested host
```

---

# 255. Attestation SLOs

Examples:

```text
100% Tier0 hosts on approved profile
0 blocked artifact active
quarantine propagation within target
```

---

# 256. Integrity SLO

No unexplained critical drift.

---

# 257. Runtime Drift SLO

All drift investigated.

---

# 258. Release Gate

New artifact/image must be added to attestation policy before rollout.

---

# 259. But

Only signed approved release.

---

# 260. Deployment Integration

Part 62.

---

# 261. Deployment Flow

```text
provision
→ boot
→ attest
→ verify artifact/config
→ issue runtime credential
→ advertise service
```

---

# 262. No Advertise Before Attestation For Required Node Class

Hard rule.

---

# 263. Emergency Recovery

If attestation service unavailable.

---

# 264. Strict Tier0

May fail closed.

---

# 265. Other tiers

Can use cached fresh attestation.

---

# 266. No "skip attestation" toggle.

---

# 267. Hard rule.

---

# 268. Attestation Service Outage

Part 70 exercise.

---

# 269. Recovery Mode

Predefined.

---

# 270. Trusted Recovery Profile

Signed.

---

# 271. Runtime Update

Hot patching discouraged.

---

# 272. Why

Breaks measurement/reproducibility.

---

# 273. Preferred

Replace artifact/node.

---

# 274. Live Kernel Patch

If used:

```text
must be signed/measured
```

---

# 275. No Invisible Patch

Hard rule.

---

# 276. Container Runtime

Container image digest measured.

---

# 277. Host kernel still matters.

---

# 278. Container Attestation

Can include:

```text
host profile
image digest
config digest
```

---

# 279. Kubernetes

Admission controller can enforce image signatures.

---

# 280. But

Kubernetes not required.

---

# 281. systemd/Podman

Equivalent policy possible.

---

# 282. VM Attestation

Cloud vendor may provide.

---

# 283. Use carefully.

---

# 284. Cloud Attestation

Vendor trust dependency.

---

# 285. Cross-cloud abstraction.

---

# 286. Bare Metal

TPM-based.

---

# 287. Virtualized TPM

Weaker depending hypervisor trust.

---

# 288. Assurance level should reflect.

---

# 289. Confidential Computing

Optional future.

---

# 290. Examples conceptually:

```text
hardware-backed encrypted memory
attested execution environment
```

---

# 291. But

No dependency for core SIAR.

---

# 292. Why

Vendor lock-in/performance/complexity.

---

# 293. Could protect high-value infrastructure.

---

# 294. User Privacy

Remote confidential-compute attestation still infrastructure-scoped.

---

# 295. Testing

Need host-integrity testkit.

---

# 296. Test Scenarios

```text
tampered bootloader
wrong kernel
wrong artifact
wrong config
runtime file modification
stale profile
attestation replay
```

---

# 297. Boot Mismatch Test

Reject/quarantine.

---

# 298. Artifact Mismatch Test

Reject.

---

# 299. Config Mismatch Test

Reject/degrade by policy.

---

# 300. Replay Test

Old attestation evidence rejected.

---

# 301. Revocation Test

Revoked host remains ineligible.

---

# 302. Re-enrollment Test

Fresh host identity accepted after rebuild.

---

# 303. Secret Sealing Test

Credential unavailable under wrong measurement.

---

# 304. Maintenance Profile Test

Cannot serve production traffic.

---

# 305. Container Image Test

Unexpected digest rejected.

---

# 306. Runtime Mutation Test

Quarantine.

---

# 307. Attestation Outage Test

Cached/fail-closed behavior correct.

---

# 308. Federation Test

Peer attestation does not create transitive trust.

---

# 309. Client Privacy Test

No user-device attestation identifier appears in anonymous protocol.

---

# 310. Fuzzing

Fuzz:

```text
attestation evidence
measurement manifests
boot profiles
host revocation
```

---

# 311. Property Tests

Properties:

```text
blocked boot profile never becomes Trusted
revoked host never becomes eligible from stale cached attestation
maintenance profile never serves production
user identity never participates in infrastructure attestation key
```

---

# 312. Formal Verification Targets

Strong candidates:

```text
attestation decision state
host revocation
re-enrollment
boot-profile anti-rollback
```

---

# 313. Kani Candidate

measurement/profile matching.

---

# 314. TLA+ Candidate

fleet rollout with attestation-policy transition.

---

# 315. Loom Candidate

concurrent attestation refresh + revocation.

---

# 316. Performance Tests

Measure:

```text
boot verification
attestation generation
verification latency
runtime integrity scan
```

---

# 317. Hot Path Rule

Attestation is not per packet.

---

# 318. Hard rule.

---

# 319. Cache Decisions

Short-lived.

---

# 320. Revocation-aware.

---

# 321. Resource Cost

TPM operations can be slow.

---

# 322. Batch/async verification.

---

# 323. No user-visible latency dependency.

---

# 324. Crate Layout

Recommended:

```text
crates/
├── siar-attestation-core/
├── siar-boot-profile/
├── siar-measured-boot/
├── siar-host-identity/
├── siar-runtime-integrity/
├── siar-attestation-verifier/
├── siar-host-revocation/
├── siar-integrity-observability/
└── siar-attestation-testkit/
```

---

# 325. `siar-attestation-core`

Owns:

```text
assurance levels
evidence
decisions
errors
```

---

# 326. `siar-boot-profile`

Allowed measurements/profile lifecycle.

---

# 327. `siar-measured-boot`

TPM/software measurement adapters.

---

# 328. `siar-host-identity`

Enrollment/re-enrollment.

---

# 329. `siar-runtime-integrity`

File/config/runtime verification.

---

# 330. `siar-attestation-verifier`

Policy evaluation.

---

# 331. `siar-host-revocation`

Quarantine/revocation state.

---

# 332. `siar-integrity-observability`

Privacy-safe fleet metrics.

---

# 333. `siar-attestation-testkit`

Tamper/replay/rollback simulator.

---

# 334. Error Taxonomy

```rust
pub enum AttestationError {
    EvidenceInvalid,
    SignatureInvalid,
    NonceExpired,
    ReplayDetected,
    BootProfileUnknown,
    BootProfileBlocked,
    ArtifactMismatch,
    ConfigMismatch,
    HostRevoked,
    AssuranceInsufficient,
    Internal,
}

pub enum IntegrityError {
    FileMismatch,
    RuntimeMutation,
    PolicyMismatch,
    VerificationUnavailable,
    Internal,
}
```

---

# 335. Security & Privacy Invariants

Mandatory:

```text
1. Critical infrastructure can prove approved boot, artifact, and configuration state before becoming eligible to serve.
2. Attestation uses dedicated infrastructure identities and never user communication identities.
3. User devices are not globally required to expose hardware attestation identifiers.
4. Blocked boot profiles, artifacts, and revoked hosts cannot regain trust through rollback or stale evidence.
5. Attestation evidence is freshness-bound and replay-protected.
6. Runtime integrity monitoring is limited to infrastructure state and never expands into user-content inspection.
7. Quarantine removes compromised nodes from service before repair/rebuild.
8. Confirmed compromise triggers fresh credentials and, where appropriate, fresh host identity.
9. Maintenance/recovery profiles cannot silently serve production traffic.
10. Remote peer/federation attestation does not create transitive trust.
11. Hardware/TPM absence may reduce assurance, but never silently claim equivalent trust.
12. Attestation and integrity checks remain off the per-packet data path.
```

---

# 336. Initial Production Scope

Implement first:

```text
exact artifact/config measurement
signed boot profile registry
software + TPM-backed attestation abstraction
host enrollment/re-enrollment
attestation nonce/replay protection
node eligibility gate
runtime integrity manifest
host revocation/quarantine
attestation cache with revocation override
deployment integration
privacy-safe fleet metrics
tamper/replay/rollback testkit
```

Then add:

```text
sealed runtime credentials
verified-boot integration
multi-verifier attestation
federation gateway attestation
confidential-computing adapters
formal attestation-policy transition verification
```

---

# 337. Definition of Done

Part 71 is complete when:

- boot trust classes and assurance levels are explicit
- critical hosts measure/verify boot, artifact, and config
- attestation evidence is nonce-bound and replay-safe
- host identity is separate from service/user identity
- blocked/stale profiles cannot regain trust
- runtime integrity covers binaries/config/security policy
- quarantine/removal/re-enrollment flows are explicit
- no global user-device attestation is required
- provider/federation attestation remains scoped and non-transitive
- maintenance/recovery profiles cannot serve production traffic
- runtime credentials can optionally bind to trusted host state
- observability is fleet-level and privacy-safe
- tamper/replay/rollback/revocation/fuzz/formal tests are specified

---

# 338. Final Architecture

```text
                       FIRMWARE / BOOT
                              │
                              ▼
                    VERIFIED / MEASURED BOOT
                              │
                              ▼
                 ARTIFACT + CONFIG MEASUREMENT
                              │
                              ▼
                     ATTESTATION EVIDENCE
                              │
                              ▼
                   POLICY-BASED VERIFIER
                              │
             ┌────────────────┼────────────────┐
             │                │                │
          Trusted          Degraded        Quarantine
             │                │                │
             └────────────────┼────────────────┘
                              ▼
                     NODE SERVICE ELIGIBILITY
```

Trustworthy-host model:

```text
verified boot
+
measured artifact/config
+
fresh attestation
+
runtime integrity
+
revocation/quarantine
+
rebuild/re-enroll after compromise
```

not:

```text
a node started successfully, therefore it must be trustworthy
```

---

# 339. Final Principle

Infrastructure trust should be evidence-based, scoped, and revocable.

The correct model is:

```text
measure
+
verify
+
attest
+
monitor
+
quarantine
+
rebuild
```

while keeping hardware attestation away from anonymous user identity and end-user protocol semantics.

This architecture gives SIAR a trustworthy-infrastructure foundation for mix nodes, mailboxes, relays, gateways, directories, control-plane systems, and federation gateways without turning platform attestation into a new privacy or fingerprinting channel.
