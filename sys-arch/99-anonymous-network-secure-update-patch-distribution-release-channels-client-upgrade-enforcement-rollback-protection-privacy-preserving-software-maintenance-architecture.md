# Core System Architecture Part 99 — Anonymous Network Secure Update, Patch Distribution, Release Channels, Client Upgrade Enforcement, Rollback Protection & Privacy-Preserving Software Maintenance Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 99  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 27, 52, 62, 66, 71–73, 80, 94–98  

**Primary purpose:** define SIAR's secure software-update architecture for release metadata, signing, release channels, staged rollout, patch distribution, update mirrors, delta/full packages, client/server compatibility, upgrade enforcement, downgrade and rollback protection, offline maintenance, broken-release recovery, privacy-preserving update checks, and post-update verification across desktop, Android, servers, embedded nodes, and managed deployments.

---

# 1. Purpose

Software maintenance is part of the security model.

A secure system can still fail if its update mechanism allows:

```text
unsigned updates
rollback to vulnerable versions
malicious mirrors
stale metadata
forced telemetry-linked update checks
broken mandatory upgrades
unsafe hotfixes
```

The governing principle is:

> **SIAR updates must be signed, reproducible, anti-rollback, recoverable, privacy-minimized, and independently verifiable—while keeping upgrade policy separate from user tracking and product analytics.**

---

# 2. Architectural Position

```text
Source / CI / Reproducible Build
             │
             ▼
        Release Artifact
             │
             ▼
      Verification Manifest
             │
             ▼
      Signed Update Metadata
             │
      ┌──────┼──────┐
      │      │      │
   Stable  Beta   Security
      │      │      │
      └──────┼──────┘
             ▼
      Update Distribution
             │
      ┌──────┼─────────┐
      │      │         │
   Mirror   CDN      Offline
      │      │         │
      └──────┼─────────┘
             ▼
       Client Verification
             │
             ▼
       Install / Reboot
             │
             ▼
       Post-Update Verify
```

---

# 3. Core Separation

Keep distinct:

```text
release artifact
release metadata
update channel
rollout policy
compatibility policy
minimum supported version
security enforcement
telemetry
```

---

# 4. Non-Goals

Part 99 does not create:

```text
silent unsigned auto-update
per-user update profiling
update checks tied to global user identity
emergency bypass of signature verification
unbounded remote code execution
```

---

# 5. Release Identity

```rust
pub struct ReleaseId(pub [u8; 32]);
```

---

# 6. Release Version

```rust
pub struct ReleaseVersion(pub String);
```

---

# 7. Artifact Identity

Content-addressed digest.

---

# 8. Hard Rule

Version string is not artifact identity.

---

# 9. Release Artifact

```rust
pub struct ReleaseArtifact {
    pub release: ReleaseId,
    pub version: ReleaseVersion,
    pub platform: TargetPlatform,
    pub architecture: CpuArchitecture,
    pub digest: ArtifactDigest,
    pub size: u64,
}
```

---

# 10. Target Platforms

```rust
pub enum TargetPlatform {
    Linux,
    Windows,
    Macos,
    Android,
    ServerLinux,
    EmbeddedLinux,
}
```

---

# 11. Architectures

```rust
pub enum CpuArchitecture {
    X86_64,
    Aarch64,
    Armv7,
    Riscv64,
}
```

---

# 12. Release Artifact Provenance

Must include:

```text
source commit
lockfile
toolchain
build environment
SBOM
provenance
signatures
```

---

# 13. Hard Rule

No release without provenance manifest.

---

# 14. Verification Manifest

```rust
pub struct ReleaseVerificationManifest {
    pub release: ReleaseId,
    pub source_revision: SourceRevision,
    pub artifact_digests: Vec<ArtifactDigest>,
    pub sbom_digest: Digest,
    pub provenance_digest: Digest,
    pub test_manifest_digest: Digest,
}
```

---

# 15. Release Metadata

Clients consume signed metadata.

---

# 16. Update Metadata

```rust
pub struct UpdateMetadata {
    pub release: ReleaseId,
    pub version: ReleaseVersion,
    pub channel: ReleaseChannel,
    pub artifacts: Vec<ReleaseArtifact>,
    pub compatibility: CompatibilityPolicy,
    pub minimum_previous_version: Option<ReleaseVersion>,
    pub security_level: UpdateSecurityClass,
}
```

---

# 17. Release Channel

```rust
pub enum ReleaseChannel {
    Stable,
    Beta,
    Nightly,
    Security,
    Managed,
}
```

---

# 18. Stable

Production.

---

# 19. Beta

Opt-in preview.

---

# 20. Nightly

Developer/test only.

---

# 21. Security

Emergency security release.

---

# 22. Managed

Organization-specific approved channel.

---

# 23. Hard Rule

Channel membership is explicit, not inferred from user behavior.

---

# 24. Security Update Class

```rust
pub enum UpdateSecurityClass {
    Routine,
    RecommendedSecurity,
    RequiredSecurity,
    EmergencySecurity,
}
```

---

# 25. Routine

Normal release.

---

# 26. RecommendedSecurity

Security improvement but no immediate cutoff.

---

# 27. RequiredSecurity

Old client/service must upgrade by deadline.

---

# 28. EmergencySecurity

Old version blocked rapidly.

---

# 29. Hard Rule

Mandatory upgrade requires documented security/privacy/compatibility justification.

---

# 30. Update Metadata Signing

Use dedicated release metadata signing key.

---

# 31. No CI Worker Online Root Key

Hard rule.

---

# 32. Signing Hierarchy

Recommended:

```text
Offline Release Root
        │
        ▼
Release Metadata Key
        │
        ▼
Channel Metadata Signatures
```

---

# 33. Root Key

Offline/HSM-backed.

---

# 34. Metadata Key

Shorter-lived.

---

# 35. Channel Key

Optional separate keys.

---

# 36. Hard Rule

Compromise of one online channel key must not equal root compromise.

---

# 37. Key Rotation

Supported.

---

# 38. Metadata Includes Key IDs

---

# 39. Old metadata remains verifiable.

---

# 40. Revoked key cannot sign future releases.

---

# 41. Hard rule.

---

# 42. Metadata Canonical Encoding

Internal signing over canonical Postcard bytes.

---

# 43. RON for human config.

---

# 44. JSON only external interoperability.

---

# 45. No signature over ambiguous JSON.

---

# 46. Hard rule.

---

# 47. Update Metadata Versioning

Separate schema version.

---

# 48. Client must reject unsupported security-critical metadata.

---

# 49. No permissive unknown-field interpretation for security semantics.

---

# 50. Hard rule.

---

# 51. Metadata Anti-Rollback

Clients store highest accepted security epoch.

---

# 52. Security Epoch

```rust
pub struct ReleaseSecurityEpoch(pub u64);
```

---

# 53. Monotonic.

---

# 54. New security release can increment epoch.

---

# 55. Client rejects metadata with older epoch unless explicit recovery mode.

---

# 56. Hard rule.

---

# 57. Version Rollback Protection

Separate from security epoch.

---

# 58. Example

User may intentionally downgrade non-security beta release.

---

# 59. But never below security floor.

---

# 60. Hard rule.

---

# 61. Installed State

```rust
pub struct InstalledReleaseState {
    pub version: ReleaseVersion,
    pub release: ReleaseId,
    pub security_epoch: ReleaseSecurityEpoch,
    pub channel: ReleaseChannel,
}
```

---

# 62. Persisted Locally.

---

# 63. Backed by secure storage where possible.

---

# 64. No server-held device identity required.

---

# 65. Hard rule.

---

# 66. Client Update Check

Privacy-sensitive.

---

# 67. Baseline request should include only:

```text
platform
architecture
current version
channel
metadata schema version
```

---

# 68. Exclude:

```text
account ID
device ID
contacts
IP-derived profile
feature history
```

---

# 69. Hard rule.

---

# 70. Update Check Request

```rust
pub struct UpdateCheckRequest {
    pub platform: TargetPlatform,
    pub architecture: CpuArchitecture,
    pub current_version: ReleaseVersion,
    pub channel: ReleaseChannel,
    pub metadata_version: UpdateMetadataSchemaVersion,
}
```

---

# 71. No Stable User Identifier

Hard rule.

---

# 72. Strict Privacy Mode

Can fetch full channel metadata anonymously/cacheably.

---

# 73. Client evaluates locally.

---

# 74. No query per exact device version if avoidable.

---

# 75. Hard rule.

---

# 76. Static Channel Manifest

Preferred privacy model.

---

# 77. Example:

```text
/stable/latest.postcard
/security/latest.postcard
```

---

# 78. Cache-friendly.

---

# 79. No personalized response.

---

# 80. Hard rule.

---

# 81. Mirror Architecture

Multiple mirrors.

---

# 82. Mirror is untrusted for authenticity.

---

# 83. Clients verify signatures/digests.

---

# 84. Hard rule.

---

# 85. Mirror Discovery

Signed mirror list.

---

# 86. Mirror Entry

```rust
pub struct UpdateMirror {
    pub mirror_id: MirrorId,
    pub endpoint: UpdateEndpoint,
    pub regions: Vec<CoarseRegion>,
    pub capabilities: MirrorCapabilities,
}
```

---

# 87. No user assignment per mirror.

---

# 88. Client can choose locally.

---

# 89. Hard rule.

---

# 90. Mirror Selection

Factors:

```text
latency
availability
region
privacy mode
```

---

# 91. No hidden user identity affinity.

---

# 92. Hard rule.

---

# 93. CDN

Allowed for public artifacts.

---

# 94. CDN sees IP unless privacy route used.

---

# 95. Strict mode can fetch through proxy/mix/private relay.

---

# 96. No claim CDN fetch is anonymous otherwise.

---

# 97. Hard rule.

---

# 98. Artifact Download

Streaming.

---

# 99. Resume supported.

---

# 100. Partial chunks verified.

---

# 101. Final digest mandatory.

---

# 102. No install before full verification.

---

# 103. Hard rule.

---

# 104. Chunk Manifest

```rust
pub struct ArtifactChunkManifest {
    pub artifact: ArtifactDigest,
    pub chunk_size: u32,
    pub chunk_digests: Vec<Digest>,
}
```

---

# 105. Optional.

---

# 106. Helps resume/P2P distribution.

---

# 107. Still verify full artifact digest.

---

# 108. Hard rule.

---

# 109. Delta Updates

Optional.

---

# 110. Useful for large binaries.

---

# 111. Delta Package

```rust
pub struct DeltaUpdate {
    pub from: ArtifactDigest,
    pub to: ArtifactDigest,
    pub patch_digest: Digest,
}
```

---

# 112. Client verifies:

```text
source digest
patch digest
result digest
```

---

# 113. No trust in delta alone.

---

# 114. Hard rule.

---

# 115. Delta Fallback

If patch fails, fetch full artifact.

---

# 116. No security downgrade.

---

# 117. Hard rule.

---

# 118. P2P Update Distribution

Possible.

---

# 119. Peers can distribute public signed artifact chunks.

---

# 120. Peers are untrusted.

---

# 121. Content-addressed verification.

---

# 122. No peer learns account identity.

---

# 123. Hard rule.

---

# 124. LAN Update Sharing

Useful offline/local network.

---

# 125. Signed artifact only.

---

# 126. No automatic executable trust from nearby peer.

---

# 127. Hard rule.

---

# 128. Offline Update Bundle

For air-gapped/low-connectivity environments.

---

# 129. Offline Bundle

```rust
pub struct OfflineUpdateBundle {
    pub metadata: UpdateMetadata,
    pub artifacts: Vec<ReleaseArtifactBundle>,
    pub signatures: SignatureBundle,
}
```

---

# 130. Import From USB/File

Verify all metadata/signatures.

---

# 131. No trust based on physical media.

---

# 132. Hard rule.

---

# 133. Offline Anti-Rollback

Bundle security epoch checked.

---

# 134. Old signed bundle can still be unsafe.

---

# 135. Hard rule.

---

# 136. Time Freshness

Metadata expiration.

---

# 137. Update Metadata Includes

```text
issued_at
expires_at
```

---

# 138. Expired metadata rejected.

---

# 139. Hard rule.

---

# 140. Clock Issues

Use Part 60 time architecture.

---

# 141. No timestamp-only trust.

---

# 142. Security epoch + expiry + signatures.

---

# 143. Hard rule.

---

# 144. Update Rollout

Phased.

---

# 145. Rollout Phases

```rust
pub enum UpdateRolloutPhase {
    Internal,
    Canary,
    Small,
    Medium,
    Broad,
    GeneralAvailability,
}
```

---

# 146. Server Infrastructure

Select by instance/shard, not user.

---

# 147. Client App

Can use local random assignment.

---

# 148. No server persistent user rollout ID.

---

# 149. Hard rule.

---

# 150. Rollout Policy

```rust
pub struct UpdateRolloutPolicy {
    pub phase: UpdateRolloutPhase,
    pub percentage_basis_points: u16,
    pub eligibility: UpdateEligibility,
}
```

---

# 151. Eligibility

```text
platform
architecture
current version range
managed policy
```

---

# 152. No behavioral segmentation.

---

# 153. Hard rule.

---

# 154. Update Eligibility

```rust
pub struct UpdateEligibility {
    pub platforms: BTreeSet<TargetPlatform>,
    pub architectures: BTreeSet<CpuArchitecture>,
    pub from_versions: VersionRange,
}
```

---

# 155. Rollout Assignment

Local deterministic bucket.

---

# 156. Experiment ID not required.

---

# 157. No cross-release identity.

---

# 158. Hard rule.

---

# 159. Update Telemetry

Minimal operational metrics.

---

# 160. Allowed aggregate:

```text
download failure rate
install failure rate
crash regression
rollback rate
```

---

# 161. Forbidden:

```text
who updated exactly when
account-linked version timeline
per-user rollout history
```

---

# 162. Hard rule.

---

# 163. Post-Update Health

Local checks.

---

# 164. Examples:

```text
binary starts
database migration passes
config loads
network stack initializes
```

---

# 165. Aggregate health can report if consent/policy allows.

---

# 166. No user content.

---

# 167. Hard rule.

---

# 168. Server Update Health

Infrastructure SLOs.

---

# 169. Client Update Health

Privacy-safe aggregate.

---

# 170. No device fingerprint.

---

# 171. Hard rule.

---

# 172. Client Upgrade Enforcement

Sometimes necessary.

---

# 173. Enforcement Classes

```rust
pub enum UpgradeEnforcement {
    None,
    Warn,
    Deadline(Timestamp),
    NetworkMinimum(ReleaseVersion),
    HardBlock,
}
```

---

# 174. Warn

User notified.

---

# 175. Deadline

Upgrade required by date.

---

# 176. NetworkMinimum

Server refuses unsafe old protocol/client.

---

# 177. HardBlock

Only for severe security/privacy risk.

---

# 178. Hard Rule

HardBlock is exceptional.

---

# 179. Enforced Upgrade Must Preserve

```text
local data access
export/recovery where safe
security center
update path
```

---

# 180. No destructive lockout.

---

# 181. Hard rule.

---

# 182. Upgrade Enforcement Policy

```rust
pub struct UpgradeEnforcementPolicy {
    pub minimum_version: ReleaseVersion,
    pub minimum_security_epoch: ReleaseSecurityEpoch,
    pub mode: UpgradeEnforcement,
    pub rationale: UpgradeRationale,
}
```

---

# 183. Upgrade Rationale

Typed.

---

# 184. Example:

```rust
pub enum UpgradeRationale {
    CriticalVulnerability,
    ProtocolSecurityFloor,
    PrivacyRegression,
    TrustRootChange,
    DataMigrationRequired,
}
```

---

# 185. No "business preference" HardBlock.

---

# 186. Hard rule.

---

# 187. Grace Period

Where safe.

---

# 188. Security-critical may be short.

---

# 189. Routine compatibility longer.

---

# 190. No hidden deadline.

---

# 191. Hard rule.

---

# 192. Server Compatibility Gate

Before normal session:

```text
protocol version
security epoch
required capabilities
```

---

# 193. No account-specific discrimination.

---

# 194. Hard rule.

---

# 195. Compatibility Policy

```rust
pub struct CompatibilityPolicy {
    pub minimum_protocol: ProtocolVersion,
    pub maximum_protocol: ProtocolVersion,
    pub required_capabilities: BTreeSet<CapabilityId>,
}
```

---

# 196. Graceful Unsupported Response

Client learns:

```text
upgrade required
reason
minimum version
update channel
```

---

# 197. No opaque failure.

---

# 198. Hard rule.

---

# 199. Protocol Migration

Mixed-version window.

---

# 200. New server supports old+new within policy.

---

# 201. Security floor eventually drops old.

---

# 202. No indefinite legacy support.

---

# 203. Hard rule.

---

# 204. Data Schema Migration

Update may modify local DB.

---

# 205. Migration must be transactional/idempotent.

---

# 206. Backup/checkpoint before risky migration.

---

# 207. Hard rule.

---

# 208. Migration State

```rust
pub enum MigrationState {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}
```

---

# 209. No mark updated until migrations complete.

---

# 210. Hard rule.

---

# 211. Forward-Only Migration

Common.

---

# 212. If downgrade impossible, declare.

---

# 213. No fake rollback support.

---

# 214. Hard rule.

---

# 215. Application Rollback

Safe only if:

```text
schema compatible
security epoch allows
artifact trusted
```

---

# 216. Rollback Policy

```rust
pub enum RollbackPolicy {
    Allowed,
    AllowedWithinVersionRange(VersionRange),
    SecurityBlocked,
    MigrationBlocked,
}
```

---

# 217. SecurityBlocked

Old version vulnerable.

---

# 218. MigrationBlocked

Data incompatible.

---

# 219. Hard rule.

---

# 220. Broken Release Recovery

Need robust path.

---

# 221. Client Recovery Slot

Keep previous known-good artifact where practical.

---

# 222. But only if security policy allows rollback.

---

# 223. Hard rule.

---

# 224. A/B System Slot

Server/embedded environments can use dual slots.

---

# 225. Example:

```text
slot A = current
slot B = new
```

---

# 226. Health check before activation.

---

# 227. Automatic revert only if allowed.

---

# 228. Hard rule.

---

# 229. Desktop Update Strategy

Prefer:

```text
download
verify
stage
atomic replace
restart
post-check
```

---

# 230. Windows

Signed installer/MSI/EXE.

---

# 231. Linux

AppImage/deb/rpm or package repository according distribution.

---

# 232. macOS

Signed/notarized bundle.

---

# 233. No cross-platform unsigned generic updater.

---

# 234. Hard rule.

---

# 235. Android Update Strategy

Primary:

```text
Play Store/App store channel where applicable
```

Secondary/direct:

```text
signed APK from official channel
```

---

# 236. APK Signature Verification

Platform + metadata digest.

---

# 237. No unsigned emergency APK.

---

# 238. Hard rule.

---

# 239. Android Split/APK Artifacts

Metadata tracks exact variant.

---

# 240. Architecture/SDK compatibility.

---

# 241. No install wrong ABI.

---

# 242. Hard rule.

---

# 243. Server Update Strategy

Immutable deployment.

---

# 244. Build once, promote exact artifact.

---

# 245. No rebuild per environment.

---

# 246. Hard rule.

---

# 247. Server Rollout

```text
canary
health check
small batch
broad
```

---

# 248. No SSH-edit deployment.

---

# 249. Hard rule.

---

# 250. Embedded Node Update

Resource-limited.

---

# 251. Signed bundle.

---

# 252. Dual-slot where possible.

---

# 253. Power-failure-safe.

---

# 254. No partial firmware activation.

---

# 255. Hard rule.

---

# 256. Bootloader Trust

Must verify signed image.

---

# 257. Boot rollback counter/security epoch.

---

# 258. No unsigned recovery image.

---

# 259. Hard rule.

---

# 260. Update Mirrors And Compromise

Mirror serves malicious bytes.

---

# 261. Digest/signature verification catches.

---

# 262. Mirror cannot forge metadata.

---

# 263. Hard rule.

---

# 264. Metadata Server Compromise

Online metadata key compromise.

---

# 265. Response:

```text
revoke key
publish new root-signed metadata
raise security epoch
```

---

# 266. Clients reject revoked key.

---

# 267. Hard rule.

---

# 268. Root Key Compromise

Critical.

---

# 269. Requires Part 72 ceremony + Part 96 incident process.

---

# 270. No ordinary online rotation.

---

# 271. Hard rule.

---

# 272. Freeze Attack

Attacker serves old valid metadata.

---

# 273. Prevent with:

```text
expiry
security epoch
latest-version state
```

---

# 274. Hard rule.

---

# 275. Rollback Attack

Attacker serves old vulnerable artifact.

---

# 276. Prevent with:

```text
metadata version
security epoch
minimum version
artifact denylist
```

---

# 277. Hard rule.

---

# 278. Mix-And-Match Attack

Attacker combines metadata/artifact from different releases.

---

# 279. Manifest binds artifact digests to release.

---

# 280. Hard rule.

---

# 281. Endless Data Attack

Huge malicious artifact.

---

# 282. Metadata contains expected size.

---

# 283. Client enforces bounds.

---

# 284. Hard rule.

---

# 285. Dependency Confusion In Update Channel

Artifacts content-addressed and signed.

---

# 286. No package-name-only trust.

---

# 287. Hard rule.

---

# 288. Release Transparency

Part 94.

---

# 289. Release metadata root can be published to transparency log.

---

# 290. Client may verify inclusion.

---

# 291. No user identity required.

---

# 292. Good.

---

# 293. Reproducible Build Verification

Optional independent verifier.

---

# 294. Multiple builders compare digest.

---

# 295. Stronger assurance for critical releases.

---

# 296. No requirement every client rebuild.

---

# 297. Hard rule.

---

# 298. Release Promotion

Stages:

```text
Built
Verified
Signed
Canary
Stable
Revoked
```

---

# 299. Release State

```rust
pub enum ReleaseState {
    Built,
    Verified,
    Signed,
    Canary,
    Stable,
    Revoked,
}
```

---

# 300. No Built→Stable Shortcut

Hard rule.

---

# 301. Revoked Release

Never served as valid latest.

---

# 302. Existing installations may be blocked/forced update based on severity.

---

# 303. Hard rule.

---

# 304. Release Revocation Record

```rust
pub struct ReleaseRevocation {
    pub release: ReleaseId,
    pub reason: RevocationReason,
    pub security_epoch: ReleaseSecurityEpoch,
    pub signature: SignatureBytes,
}
```

---

# 305. Revocation Reason

```rust
pub enum RevocationReason {
    CriticalVulnerability,
    SigningCompromise,
    CorruptArtifact,
    PrivacyRegression,
    SupplyChainCompromise,
}
```

---

# 306. Signed.

---

# 307. Anti-rollback.

---

# 308. Hard rule.

---

# 309. Update Policy Source

Signed governance/release policy.

---

# 310. No server response can arbitrarily force update.

---

# 311. Client validates policy signature.

---

# 312. Hard rule.

---

# 313. Managed Organization Updates

Tenant can select approved managed channel.

---

# 314. Managed policy can delay routine updates.

---

# 315. Cannot delay critical security minimum beyond platform policy.

---

# 316. Hard rule.

---

# 317. Managed Update Policy

```rust
pub struct ManagedUpdatePolicy {
    pub channel: ReleaseChannel,
    pub defer_until: Option<Timestamp>,
    pub allow_beta: bool,
}
```

---

# 318. Applies only managed profile/device scope.

---

# 319. Personal installation policy remains user-controlled where applicable.

---

# 320. Hard rule.

---

# 321. Air-Gapped Enterprise

Offline signed bundle.

---

# 322. Internal mirror allowed.

---

# 323. Mirror does not resign upstream artifact unless organization intentionally wraps with managed approval metadata.

---

# 324. Upstream signature still preserved.

---

# 325. Hard rule.

---

# 326. Managed Approval Layer

```rust
pub struct ManagedReleaseApproval {
    pub organization: OrganizationId,
    pub upstream_release: ReleaseId,
    pub approved_at: CoarseTimestamp,
    pub signature: SignatureBytes,
}
```

---

# 327. Org approval cannot make revoked upstream release safe.

---

# 328. Hard rule.

---

# 329. Update Download Privacy

Public release artifact request should be cacheable.

---

# 330. Avoid signed URL containing account ID.

---

# 331. Hard rule.

---

# 332. Paid Distribution

If commercial licensing later

license auth should be separate from artifact request where possible.

---

# 333. No artifact URL as user tracking beacon.

---

# 334. Hard rule.

---

# 335. Update Metrics

Operational aggregate only.

---

# 336. Examples:

```text
metadata fetch failure
artifact hash failure
install failure
post-update crash rate
```

---

# 337. No user identity.

---

# 338. Hard rule.

---

# 339. Update Failure Reporting

Optional local diagnostic.

---

# 340. User can export logs.

---

# 341. No automatic private data upload.

---

# 342. Hard rule.

---

# 343. Crash Loop After Update

Recovery mode.

---

# 344. Detect local startup failures.

---

# 345. If rollback allowed, revert.

---

# 346. If rollback blocked by security/migration

enter safe recovery shell/UI.

---

# 347. Hard rule.

---

# 348. Recovery Mode

Allows:

```text
verify update
redownload
export local data
restore backup
view diagnostics
```

---

# 349. Does not allow insecure network operation.

---

# 350. Hard rule.

---

# 351. Safe Mode

Minimal UI/service.

---

# 352. No third-party plugins.

---

# 353. No optional modules.

---

# 354. Good.

---

# 355. Plugin Update Compatibility

Part 24.

---

# 356. Plugin API version tracked.

---

# 357. Incompatible plugins disabled safely.

---

# 358. No plugin blocks security update.

---

# 359. Hard rule.

---

# 360. WASM Extension Updates

Signed separately.

---

# 361. Capability manifest versioned.

---

# 362. No extension auto-escalation after update.

---

# 363. Hard rule.

---

# 364. Database Migration Rollback

Need explicit support matrix.

---

# 365. For irreversible migration:

```text
backup/clone before
declare downgrade blocked
```

---

# 366. No fake safety.

---

# 367. Hard rule.

---

# 368. Content/Data Format Compatibility

Postcard protocol schema versioned.

---

# 369. Old/new clients coexist within defined window.

---

# 370. Unknown critical schema rejected.

---

# 371. Hard rule.

---

# 372. Update Checking Offline

No network.

---

# 373. Local app keeps running if version safe.

---

# 374. Expired metadata alone should not brick purely local features.

---

# 375. Network access may require current security floor.

---

# 376. Hard rule.

---

# 377. Emergency Revocation Offline

Cannot be learned while offline.

---

# 378. On reconnect, enforce.

---

# 379. No false claim offline client is current.

---

# 380. Hard truth.

---

# 381. Time-Bounded Security Grace

Possible.

---

# 382. Example:

```text
network access allowed for 48h while update downloads
```

---

# 383. But no grace if actively exploitable critical protocol flaw.

---

# 384. Hard rule.

---

# 385. Update Scheduler

Battery/network aware on client.

---

# 386. Download low priority unless security update.

---

# 387. Security update can preempt background bulk.

---

# 388. No wake storm.

---

# 389. Hard rule.

---

# 390. Metered Network

User policy.

---

# 391. Security-critical update may ask before large download unless deadline.

---

# 392. No silent huge metered download by default.

---

# 393. Hard rule.

---

# 394. Update State Machine

```rust
pub enum ClientUpdateState {
    Idle,
    Checking,
    Available,
    Downloading,
    Verifying,
    Staged,
    Installing,
    RestartRequired,
    PostVerifying,
    Completed,
    Failed,
    RecoveryRequired,
}
```

---

# 395. Invalid transitions rejected.

---

# 396. No `Completed` Before PostVerifying.

---

# 397. Hard rule.

---

# 398. Update Operation ID

```rust
pub struct UpdateOperationId(pub [u8; 16]);
```

---

# 399. Idempotent.

---

# 400. Retry does not duplicate install.

---

# 401. Hard rule.

---

# 402. Download Resume State

Persist:

```text
artifact digest
verified chunks
bytes received
```

---

# 403. No resume against changed artifact.

---

# 404. Hard rule.

---

# 405. Update Metadata Repository

```rust
pub trait UpdateMetadataRepository {
    fn latest(
        &self,
        channel: ReleaseChannel,
        platform: TargetPlatform,
        architecture: CpuArchitecture,
    ) -> Result<UpdateMetadata, UpdateError>;
}
```

---

# 406. Release Verifier

```rust
pub trait ReleaseVerifier {
    fn verify_metadata(
        &self,
        metadata: &UpdateMetadata,
    ) -> Result<VerifiedUpdateMetadata, UpdateError>;

    fn verify_artifact(
        &self,
        artifact: &Path,
        expected: &ArtifactDigest,
    ) -> Result<(), UpdateError>;
}
```

---

# 407. Upgrade Policy Evaluator

```rust
pub trait UpgradePolicyEvaluator {
    fn evaluate(
        &self,
        installed: &InstalledReleaseState,
        metadata: &VerifiedUpdateMetadata,
    ) -> Result<UpgradeDecision, UpdateError>;
}
```

---

# 408. Upgrade Decision

```rust
pub enum UpgradeDecision {
    Current,
    Optional,
    Recommended,
    RequiredByDeadline(Timestamp),
    RequiredBeforeNetworkAccess,
    Revoked,
}
```

---

# 409. No hidden forcing.

---

# 410. Hard rule.

---

# 411. Update Installer

Platform-specific adapter.

---

# 412. Rust orchestration.

---

# 413. Platform-native signing/install where required.

---

# 414. No security-sensitive shell interpolation.

---

# 415. Hard rule.

---

# 416. Server Rollout Controller

```rust
pub trait ServerRolloutController {
    fn advance(
        &self,
        release: ReleaseId,
        phase: UpdateRolloutPhase,
    ) -> Result<(), UpdateError>;
}
```

---

# 417. Advance Requires Gates

```text
health
security
privacy
compatibility
```

---

# 418. Hard rule.

---

# 419. Release Channel Policy

```rust
pub struct ReleaseChannelPolicy {
    pub channel: ReleaseChannel,
    pub allowed_security_classes: BTreeSet<UpdateSecurityClass>,
    pub automatic_install: bool,
}
```

---

# 420. Nightly

Never auto-promoted to stable.

---

# 421. Hard rule.

---

# 422. Stable Promotion

Exact artifact.

---

# 423. No rebuild.

---

# 424. Hard rule.

---

# 425. Security Channel Promotion

Can promote same artifact to stable after validation.

---

# 426. Or dedicated patch release.

---

# 427. No hidden branch divergence.

---

# 428. Hard rule.

---

# 429. Release Notes

Human-readable.

---

# 430. Security advisory separate if needed.

---

# 431. No telemetry beacon in notes.

---

# 432. Hard rule.

---

# 433. Update Transparency UI

Show:

```text
version
channel
signature status
security classification
download size
restart required
upgrade deadline
```

---

# 434. No manipulative urgency for routine update.

---

# 435. Hard rule.

---

# 436. Mandatory Security Upgrade UI

Explain:

```text
why required
what version fixes
deadline
what remains available locally
```

---

# 437. No opaque "update to continue."

---

# 438. Hard rule.

---

# 439. Update Provenance UI

Advanced users can inspect:

```text
artifact digest
signing key ID
source revision
SBOM/provenance refs
```

---

# 440. Good.

---

# 441. Audit Integration

Part 94.

---

# 442. Audit:

```text
release promoted
release revoked
metadata key rotated
mandatory upgrade policy changed
managed approval granted
```

---

# 443. Not every client download.

---

# 444. Hard rule.

---

# 445. Compliance Integration

Part 95.

---

# 446. Controls:

```text
approved artifact only
minimum security epoch
release signature valid
rollback protection active
```

---

# 447. Hard rule.

---

# 448. Incident Integration

Part 96.

---

# 449. Compromised release can trigger incident.

---

# 450. Emergency revocation.

---

# 451. Recovery through trusted update channel.

---

# 452. Hard rule.

---

# 453. SOC Integration

Part 97.

---

# 454. Revoked artifact/key distributed as IoC.

---

# 455. No direct SOC update authority.

---

# 456. Hard rule.

---

# 457. Vulnerability Integration

Part 98.

---

# 458. Security patch maps to resolved exposure.

---

# 459. `Fixed` only after deployment/update verification.

---

# 460. Hard rule.

---

# 461. Supply-Chain Integration

Part 73.

---

# 462. Reproducible builds, SBOM, provenance, exact promotion.

---

# 463. Hard rule.

---

# 464. HSM Integration

Part 72.

---

# 465. Root/release-signing ceremonies.

---

# 466. No online software fallback.

---

# 467. Hard rule.

---

# 468. Secrets Integration

Part 80.

---

# 469. Metadata signing key via HSM/broker.

---

# 470. No private signing key in CI env.

---

# 471. Hard rule.

---

# 472. Host Integrity Integration

Part 71.

---

# 473. Server/embedded node verifies booted artifact after update.

---

# 474. Client integrity optional local.

---

# 475. Hard rule.

---

# 476. Event Bus Integration

Release events minimal.

---

# 477. No user download events.

---

# 478. Hard rule.

---

# 479. Database Integration

Stores:

```text
release metadata
channel state
revocations
rollout state
managed approvals
```

---

# 480. No per-user update timeline.

---

# 481. Hard rule.

---

# 482. Backup

Release metadata/revocation history backed up.

---

# 483. Client installed state local.

---

# 484. Security epoch anti-rollback preserved.

---

# 485. No old backup resets security floor.

---

# 486. Hard rule.

---

# 487. Multi-Region Distribution

Mirrors/CDN.

---

# 488. Metadata consistent.

---

# 489. Revocation high priority.

---

# 490. No region serving revoked latest.

---

# 491. Hard rule.

---

# 492. Federation Update Compatibility

Federated peers negotiate protocol version.

---

# 493. No shared update authority.

---

# 494. Peer can require minimum protocol capability.

---

# 495. Cannot force remote domain software installation.

---

# 496. Hard rule.

---

# 497. Update Observability

Safe metrics:

```text
channel metadata availability
artifact download failure
signature verification failure
rollout health
rollback count
```

---

# 498. Forbidden:

```text
user-level exact update history
per-account version timeline
```

---

# 499. Hard rule.

---

# 500. Update SLOs

Examples:

```text
critical metadata availability
revocation propagation
security patch mirror freshness
canary evaluation latency
```

---

# 501. Security SLO

```text
0 unsigned release accepted
0 revoked artifact served as valid
0 security-epoch rollback accepted
```

---

# 502. Privacy SLO

```text
0 stable user IDs in update checks
0 account-linked artifact download logs
0 behavioral rollout targeting
```

---

# 503. Failure Modes

```text
mirror outage
metadata key compromise
broken release
migration failure
forced-upgrade path failure
```

---

# 504. Mirror Outage

Try other signed mirror/CDN.

---

# 505. No trust downgrade.

---

# 506. Hard rule.

---

# 507. Metadata Key Compromise

Revoke via root.

---

# 508. Security epoch bump.

---

# 509. Incident.

---

# 510. Hard rule.

---

# 511. Broken Release

Pause rollout.

---

# 512. Revoke if severe.

---

# 513. Roll back only if allowed.

---

# 514. Publish fixed release.

---

# 515. Hard rule.

---

# 516. Migration Failure

Enter recovery mode.

---

# 517. Restore checkpoint if safe.

---

# 518. No continue with partial schema.

---

# 519. Hard rule.

---

# 520. Forced Upgrade Path Failure

Must preserve update/recovery capability.

---

# 521. Server should not lock out update metadata endpoint.

---

# 522. Hard rule.

---

# 523. Update Endpoint Exception

Even blocked old client can access:

```text
update metadata
artifact download
security advisory
recovery docs
```

---

# 524. No normal app API.

---

# 525. Hard rule.

---

# 526. Testing

Need secure-update testkit.

---

# 527. Test Scenarios

```text
signed stable update
stale metadata
revoked release
broken migration
mandatory security update
```

---

# 528. Signature Test

Unsigned/invalid metadata rejected.

---

# 529. Artifact Test

Digest mismatch rejected.

---

# 530. Freeze Test

Expired/old metadata rejected.

---

# 531. Rollback Test

Lower security epoch rejected.

---

# 532. Mix-And-Match Test

Wrong artifact for manifest rejected.

---

# 533. Mirror Test

Malicious mirror cannot forge release.

---

# 534. Delta Test

Bad delta falls back full artifact.

---

# 535. Offline Bundle Test

Old signed bundle rejected by security epoch.

---

# 536. Client Privacy Test

Update request contains no account/device ID.

---

# 537. Rollout Test

Local bucket assignment stable within release phase.

---

# 538. Managed Policy Test

Tenant cannot delay emergency security floor indefinitely.

---

# 539. Mandatory Upgrade Test

Blocked client can still access updater/recovery.

---

# 540. DB Migration Test

Partial failure cannot mark update complete.

---

# 541. Crash Loop Test

Recovery mode reached safely.

---

# 542. Revocation Test

Revoked release not offered.

---

# 543. Backup Test

Old installed-state backup cannot lower security epoch.

---

# 544. Federation Test

Peer cannot force software installation.

---

# 545. Fuzzing

Fuzz:

```text
update metadata
channel manifest
delta manifest
offline update bundle
revocation record
```

---

# 546. Property Tests

Properties:

```text
release with invalid signature can never reach Install state
metadata security epoch lower than accepted floor can never become Available
revoked release can never be selected as valid latest
mandatory upgrade cannot disable update/recovery path
```

---

# 547. Formal Verification Targets

Strong candidates:

```text
client update state machine
security epoch anti-rollback
release promotion/revocation
mandatory upgrade compatibility gate
```

---

# 548. Kani Candidate

upgrade decision and rollback-policy invariants.

---

# 549. TLA+ Candidate

publish → rollout → revoke → client check → install/recover.

---

# 550. Loom Candidate

concurrent metadata refresh + download completion + revocation arrival.

---

# 551. Performance

Update checks infrequent/cacheable.

---

# 552. Full channel manifests static.

---

# 553. CDN-friendly.

---

# 554. Artifact downloads resumable.

---

# 555. Delta optional.

---

# 556. No high-frequency polling.

---

# 557. Hard rule.

---

# 558. Storage

Client:

```text
installed release state
highest security epoch
download resume state
previous safe artifact if allowed
```

Server:

```text
signed channel metadata
release manifests
revocations
rollout state
```

---

# 559. No per-user update history.

---

# 560. Hard rule.

---

# 561. Crate Layout

Recommended:

```text
crates/
├── siar-update-core/
├── siar-release-metadata/
├── siar-release-signing/
├── siar-update-channel/
├── siar-update-distribution/
├── siar-delta-update/
├── siar-update-policy/
├── siar-update-installer/
├── siar-update-recovery/
├── siar-update-observability/
└── siar-update-testkit/
```

---

# 562. `siar-update-core`

Owns:

```text
ReleaseId
versions
channels
states
errors
```

---

# 563. `siar-release-metadata`

Canonical signed update manifests.

---

# 564. `siar-release-signing`

Root/channel signing and rotation.

---

# 565. `siar-update-channel`

Stable/Beta/Nightly/Security/Managed channels.

---

# 566. `siar-update-distribution`

Mirrors/CDN/P2P/offline bundle adapters.

---

# 567. `siar-delta-update`

Delta manifest/apply/verification.

---

# 568. `siar-update-policy`

security epoch, minimum version, enforcement, compatibility.

---

# 569. `siar-update-installer`

Platform-specific install orchestration.

---

# 570. `siar-update-recovery`

rollback, safe mode, recovery shell.

---

# 571. `siar-update-observability`

aggregate release/update health only.

---

# 572. `siar-update-testkit`

signature/rollback/recovery/privacy tests.

---

# 573. Error Taxonomy

```rust
pub enum UpdateError {
    MetadataInvalid,
    SignatureInvalid,
    MetadataExpired,
    SecurityEpochRollback,
    ReleaseRevoked,
    ArtifactDigestMismatch,
    ArtifactTooLarge,
    DeltaInvalid,
    PlatformUnsupported,
    CompatibilityDenied,
    UpgradeRequired,
    MigrationFailed,
    RollbackDenied,
    InstallFailed,
    RecoveryRequired,
    Internal,
}
```

---

# 574. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Every installable release artifact is bound to signed canonical metadata, an exact artifact digest, provenance, SBOM, and release identity.
2. Mirrors, CDNs, peers, and offline media are untrusted distribution channels; authenticity comes only from signatures, digests, and anti-rollback state.
3. Clients persist a monotonic security epoch/minimum security floor and cannot accept stale signed metadata or vulnerable rollback releases below that floor.
4. Update checks do not require stable account/device identifiers and should be cacheable/static whenever possible.
5. Mandatory upgrade policy is reserved for genuine security, privacy, trust-root, protocol-floor, or incompatible-data reasons—not product engagement or business preference.
6. Clients blocked for security reasons still retain access to update metadata, signed artifacts, recovery, security information, and safe local export where feasible.
7. Release rollout uses staged canary/partial/broad phases and cannot promote an artifact that fails health, security, privacy, compatibility, or supply-chain gates.
8. Emergency security updates never bypass signature verification, provenance, artifact integrity, or trusted key hierarchy.
9. Database/config/data migrations are versioned, crash-safe, and cannot mark an update complete before migration and post-update verification succeed.
10. Managed organizations may select/defer approved channels within policy but cannot reauthorize revoked releases or indefinitely bypass critical platform security floors.
11. Release/update telemetry is aggregate and operational only; there is no durable per-user upgrade timeline, behavioral rollout targeting, or update-based device tracking.
12. Rollback, recovery, revocation, and backup paths preserve anti-rollback state so old manifests, stale backups, or compromised mirrors cannot resurrect an unsafe release.
```

---

# 575. Initial Production Scope

Implement first:

```text
typed ReleaseId/ReleaseVersion/ReleaseChannel
canonical signed update metadata
offline root + release metadata signing hierarchy
artifact digest verification
security epoch anti-rollback
stable/beta/security/managed channels
static privacy-friendly update checks
mirror/CDN distribution
resumable full artifact downloads
optional delta format
client update state machine
minimum supported version policy
mandatory security upgrade path
server phased rollout
database migration safety
release revocation
recovery/safe mode
managed update policy
privacy-safe update metrics
secure-update testkit
```

Then add:

```text
P2P/LAN artifact distribution
transparency-log inclusion verification
multi-builder reproducibility voting
advanced delta/chunk distribution
formal update/revocation verification
air-gapped enterprise update orchestration
```

---

# 576. Definition of Done

Part 99 is complete when:

- update metadata is canonical, signed, versioned, and expiring
- artifacts are digest-bound to release manifests
- security epochs prevent rollback/freeze attacks
- mirrors/CDNs/offline media are treated as untrusted
- update checks contain no stable user/device identity
- release channels are explicit
- staged rollouts and health gates work
- mandatory upgrades are security-justified and transparent
- blocked clients retain update/recovery access
- migrations are crash-safe
- broken releases can be revoked/recovered safely
- managed channels cannot override revoked/security-floor policy
- aggregate telemetry has no per-user update history
- signature/rollback/recovery/privacy/fuzz/formal tests are specified

---

# 577. Final Architecture

```text
                 REPRODUCIBLE BUILD
                        │
                        ▼
               VERIFIED ARTIFACT
                        │
                        ▼
                 SIGNED METADATA
                        │
            ┌───────────┼───────────┐
            │           │           │
         STABLE        BETA      SECURITY
            │           │           │
            └───────────┼───────────┘
                        ▼
              MIRROR / CDN / P2P
                        │
                        ▼
                 CLIENT VERIFY
                        │
                        ▼
              DOWNLOAD / STAGE
                        │
                        ▼
                 INSTALL / MIGRATE
                        │
                        ▼
                POST-UPDATE VERIFY
```

Update safety model:

```text
signed metadata
+
exact artifact digests
+
reproducible provenance
+
security epoch
+
staged rollout
+
privacy-friendly checks
+
safe migration
+
revocation/recovery
```

not:

```text
phone home with a device identity, trust whichever server answers, install arbitrary bytes, and hope rollback saves you
```

---

# 578. Final Principle

Software updates should strengthen trust without becoming a privileged tracking or remote-control channel.

The correct model is:

```text
build reproducibly
+
sign narrowly
+
publish transparently
+
fetch privately
+
verify locally
+
roll out gradually
+
enforce only for real security floors
+
recover without weakening trust
```

This architecture gives SIAR a secure, privacy-preserving software-maintenance foundation across desktop, Android, servers, embedded nodes, managed deployments, federation compatibility, vulnerability patching, release revocation, and offline environments while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–98.
