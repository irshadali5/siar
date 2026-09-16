# Core System Architecture Part 133 — Anonymous Network Extension Marketplace, Integration Distribution, Package Discovery, Trust Signals, Review, Revocation & Privacy-Preserving Developer Ecosystem Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 133  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 21, 24, 47–49, 68, 73, 94–99, 122–132

**Primary purpose:** define SIAR's developer-ecosystem architecture for extension marketplaces, package distribution, publisher identity, discovery, trust signals, certification, install-time capability disclosure, supply-chain checks, updates, revocation, emergency disable, compatibility, moderation, and privacy-preserving ecosystem growth.

---

# 1. Purpose

A healthy extension ecosystem can greatly expand the platform, but it also creates a major trust boundary.

Extensions may introduce:

```text
malicious code
excessive permissions
supply-chain compromise
tracking
unstable dependencies
compatibility breakage
misleading listings
```

A mature ecosystem must answer:

```text
Who published this extension?
What capabilities does it request?
Has the package been reviewed or certified?
Which versions are supported?
Can a compromised package be revoked?
How are updates distributed safely?
How can users discover extensions without behavioral tracking?
```

The governing principle is:

> **SIAR's extension ecosystem should make package authority, capability access, provenance, compatibility, review, and revocation explicit while keeping discovery useful without turning the marketplace into a user-surveillance or pay-to-rank system.**

---

# 2. Architectural Position

```text
                  EXTENSION PUBLISHER
                          │
                          ▼
                 PACKAGE SUBMISSION
                          │
                          ▼
              REVIEW / CERTIFICATION
                          │
                          ▼
                   MARKETPLACE INDEX
                          │
              ┌───────────┼───────────┐
              │           │           │
          DISCOVERY     INSTALL      UPDATE
              │           │           │
              └───────────┼───────────┘
                          ▼
                     RUNTIME
                          │
                          ▼
                  REVOKE / RETIRE
```

---

# 3. Core Separation

Keep distinct:

```text
publisher identity
package identity
extension identity
listing
certification
signature
capability request
runtime grant
```

---

# 4. Non-Goals

Part 133 does not create:

```text
pay-to-rank marketplace results
behavioral ad targeting
silent permission grants
unsigned extension distribution
unreviewed native code with full process access
```

---

# 5. Extension Identity

```rust
pub struct ExtensionId(pub [u8; 16]);
```

Stable across versions.

---

# 6. Package Identity

```rust
pub struct ExtensionPackageId(pub Digest);
```

Content-addressed.

---

# 7. Publisher Identity

```rust
pub struct PublisherId(pub [u8; 16]);
```

---

# 8. Listing Identity

```rust
pub struct MarketplaceListingId(pub [u8; 16]);
```

---

# 9. Hard Rule

Extension, package, publisher, and listing identities are distinct.

---

# 10. Extension Class

```rust
pub enum ExtensionClass {
    WasmPlugin,
    NativePlugin,
    Bot,
    Automation,
    EnterpriseConnector,
    ImporterExporter,
    UiExtension,
    MonitoringAdapter,
}
```

---

# 11. Preferred Isolation

WASM where practical.

---

# 12. Hard rule.

---

# 13. Native Extension

Requires stronger review/certification.

---

# 14. Hard rule.

---

# 15. Extension Manifest

```rust
pub struct ExtensionManifest {
    pub extension: ExtensionId,
    pub publisher: PublisherId,
    pub version: ExtensionVersion,
    pub class: ExtensionClass,
    pub requested_capabilities: BTreeSet<SdkCapability>,
    pub network_access: NetworkAccessPolicy,
    pub file_access: FileAccessPolicy,
}
```

---

# 16. Hard rule.

---

# 17. Manifest Digest

```rust
pub struct ExtensionManifestDigest(pub Digest);
```

---

# 18. Signed With Package

Hard rule.

---

# 19. Extension Version

```rust
pub struct ExtensionVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}
```

---

# 20. Compatibility

Extension declares:

```text
SDK version
protocol compatibility
host version
platform
```

---

# 21. Hard rule.

---

# 22. Extension Compatibility Manifest

```rust
pub struct ExtensionCompatibility {
    pub sdk_versions: VersionRange<SdkVersion>,
    pub host_versions: VersionRange<Version>,
    pub platforms: BTreeSet<PlatformClass>,
}
```

---

# 23. Hard rule.

---

# 24. Publisher Registration

Publisher identity separated from personal developer account.

---

# 25. Hard rule.

---

# 26. Publisher Verification

Possible levels:

```rust
pub enum PublisherVerificationLevel {
    Unverified,
    EmailVerified,
    OrganizationVerified,
    HighAssuranceVerified,
}
```

---

# 27. Verification Is Not Endorsement

Hard rule.

---

# 28. Publisher Key

Used to sign package metadata.

---

# 29. Hard rule.

---

# 30. Publisher Key Rotation

Supported.

---

# 31. Hard rule.

---

# 32. Publisher Key Compromise

Triggers:

```text
suspension
package review
key rotation
possible revocation
```

---

# 33. Hard rule.

---

# 34. Package Build

Package should include:

```text
artifact
manifest
SBOM
provenance
signature
license metadata
```

---

# 35. Hard rule.

---

# 36. Package Provenance

```rust
pub struct ExtensionPackageProvenance {
    pub source_revision: RevisionId,
    pub build_digest: Digest,
    pub sbom_digest: Digest,
    pub publisher_signature: Signature,
}
```

---

# 37. Hard rule.

---

# 38. Reproducible Build

Preferred.

---

# 39. Hard rule.

---

# 40. Supply-Chain Checks

Part 73.

Check:

```text
dependency provenance
licenses
known vulnerabilities
signature validity
reproducibility
```

---

# 41. Hard rule.

---

# 42. Extension Submission

```rust
pub struct ExtensionSubmission {
    pub package: ExtensionPackageId,
    pub manifest: ExtensionManifestDigest,
    pub publisher: PublisherId,
}
```

---

# 43. Hard rule.

---

# 44. Submission State

```rust
pub enum ExtensionSubmissionState {
    Submitted,
    AutomatedChecks,
    ManualReview,
    Certification,
    Approved,
    Rejected,
    Suspended,
}
```

---

# 45. No Submitted→Approved Direct

Hard rule.

---

# 46. Automated Checks

Examples:

```text
signature
malware/static scan
capability diff
compatibility
SBOM/vulnerability
manifest consistency
```

---

# 47. Hard rule.

---

# 48. Review Depth

Based on:

```text
extension class
capabilities
data access
native code
network access
```

---

# 49. No Publisher Fame Exception

Hard rule.

---

# 50. Review Finding

```rust
pub struct ExtensionReviewFinding {
    pub class: ExtensionFindingClass,
    pub severity: FindingSeverity,
    pub statement: String,
}
```

---

# 51. Finding Class

```rust
pub enum ExtensionFindingClass {
    Security,
    Privacy,
    Compatibility,
    CapabilityExcess,
    SupplyChain,
    Documentation,
    MisleadingListing,
}
```

---

# 52. Hard rule.

---

# 53. Certification

Privileged or high-risk extensions may require certification.

---

# 54. Certification Record

```rust
pub struct ExtensionCertification {
    pub certification_id: ExtensionCertificationId,
    pub extension: ExtensionId,
    pub version: ExtensionVersion,
    pub state: ExtensionCertificationState,
}
```

---

# 55. Certification State

```rust
pub enum ExtensionCertificationState {
    Draft,
    Testing,
    Passed,
    Conditional,
    Failed,
    Suspended,
    Revoked,
}
```

---

# 56. Hard rule.

---

# 57. Certification Scope

Bind:

```text
package digest
manifest
capabilities
SDK version
platform
```

---

# 58. Hard rule.

---

# 59. Marketplace Listing

```rust
pub struct MarketplaceListing {
    pub listing_id: MarketplaceListingId,
    pub extension: ExtensionId,
    pub publisher: PublisherId,
    pub current_version: ExtensionVersion,
    pub state: ListingState,
}
```

---

# 60. Listing State

```rust
pub enum ListingState {
    Draft,
    Published,
    Hidden,
    Suspended,
    Delisted,
    Retired,
}
```

---

# 61. Hard rule.

---

# 62. Listing Metadata

Allowed:

```text
name
summary
description
publisher
capabilities
platforms
version
license
support URL
privacy info
```

---

# 63. Hard rule.

---

# 64. No Misleading Permission Description

Hard rule.

---

# 65. Install-Time Permission Disclosure

Before install show:

```text
capabilities
data classes
network access
file access
admin access
```

---

# 66. Hard rule.

---

# 67. Capability Diff On Update

Show new permissions.

---

# 68. Hard rule.

---

# 69. Scope Expansion

Requires renewed approval.

---

# 70. Hard rule.

---

# 71. Scope Reduction

Can apply without new consent where safe.

---

# 72. Hard rule.

---

# 73. Permission Grant

Runtime grant separate from manifest request.

---

# 74. Hard rule.

---

# 75. Extension Runtime Grant

```rust
pub struct ExtensionRuntimeGrant {
    pub extension: ExtensionId,
    pub granted_capabilities: BTreeSet<SdkCapability>,
    pub scope: SdkCapabilityScope,
}
```

---

# 76. Hard rule.

---

# 77. Least Privilege

Do not auto-grant all requested capabilities if narrower grant possible.

---

# 78. Hard rule.

---

# 79. No ReadEverything

Hard rule.

---

# 80. Extension Sandboxing

WASM extension:

```text
memory sandbox
capability broker
no raw filesystem
no raw network by default
```

---

# 81. Hard rule.

---

# 82. Native Extension Sandboxing

Use process isolation/OS sandbox.

---

# 83. Hard rule.

---

# 84. Brokered File Access

Use selected handles.

---

# 85. Hard rule.

---

# 86. Brokered Network Access

Policy-scoped destinations.

---

# 87. Hard rule.

---

# 88. Camera/Mic/Clipboard

Explicit user/platform permission.

---

# 89. Hard rule.

---

# 90. Marketplace Discovery

Search by:

```text
name
category
capabilities
platform
compatibility
publisher
```

---

# 91. Hard rule.

---

# 92. Discovery Ranking

Allowed signals:

```text
text relevance
compatibility
certification
maintenance freshness
aggregate quality signals
```

---

# 93. Forbidden baseline signals:

```text
behavioral user profile
private conversation context
cross-service tracking
```

---

# 94. Hard rule.

---

# 95. No Pay-To-Rank

Hard rule.

---

# 96. Sponsored Placement

Not baseline.

If ever added, must be separately labeled and never override safety/relevance filters.

---

# 97. Hard rule.

---

# 98. Search Query Privacy

Avoid persistent per-user search history.

---

# 99. Hard rule.

---

# 100. Search Analytics

Aggregate only.

---

# 101. Hard rule.

---

# 102. Marketplace Categories

```rust
pub enum MarketplaceCategory {
    Productivity,
    Communication,
    Automation,
    Enterprise,
    ImportExport,
    Monitoring,
    DeveloperTool,
}
```

---

# 103. Hard rule.

---

# 104. Trust Signals

Trust signal is factual, not one overall score.

---

# 105. Trust Signal Types

```rust
pub enum ExtensionTrustSignal {
    PublisherVerified(PublisherVerificationLevel),
    PackageSigned,
    SbomAvailable,
    ReproducibleBuild,
    CertificationActive,
    SourceAvailable,
    LastUpdated(Timestamp),
}
```

---

# 106. No One "Trust Score"

Hard rule.

---

# 107. Why

One score hides tradeoffs.

---

# 108. Hard rule.

---

# 109. User Reviews

Optional.

---

# 110. Do Not Treat Reviews As Security Certification

Hard rule.

---

# 111. Review Identity Privacy

Prefer pseudonymous/optional display.

---

# 112. Hard rule.

---

# 113. Review Abuse

Moderation separate from publisher certification.

---

# 114. Hard rule.

---

# 115. Ratings

If used, aggregate only.

---

# 116. Hard rule.

---

# 117. No Rating-Based Security Override

Hard rule.

---

# 118. Package Discovery API

```rust
pub trait MarketplaceDiscoveryService {
    fn search(
        &self,
        query: MarketplaceQuery,
    ) -> Result<Vec<MarketplaceListingSummary>, MarketplaceError>;
}
```

---

# 119. Hard rule.

---

# 120. Query Limits

Bounded.

---

# 121. Hard rule.

---

# 122. Search Filter

```rust
pub struct MarketplaceQuery {
    pub text: Option<String>,
    pub category: Option<MarketplaceCategory>,
    pub platform: Option<PlatformClass>,
    pub certified_only: bool,
}
```

---

# 123. Hard rule.

---

# 124. No User-ID Parameter

Hard rule.

---

# 125. Recommendation

If offered, prefer:

```text
local/on-device based on installed features
explicit categories
```

---

# 126. Hard rule.

---

# 127. No Behavioral Collaborative Filtering Baseline

Hard rule.

---

# 128. Installation

Install flow:

```text
resolve package
verify signature
verify compatibility
show permissions
obtain approval
install sandboxed
```

---

# 129. Hard rule.

---

# 130. Install Record

```rust
pub struct ExtensionInstallRecord {
    pub extension: ExtensionId,
    pub version: ExtensionVersion,
    pub package: ExtensionPackageId,
    pub granted_capabilities: BTreeSet<SdkCapability>,
}
```

---

# 131. Hard rule.

---

# 132. Local-First Install

Desktop/mobile can install from signed package bundle.

---

# 133. Hard rule.

---

# 134. Offline Installation

Allowed with signed package + registry snapshot.

---

# 135. Hard rule.

---

# 136. Offline Registry

Must be freshness-aware.

---

# 137. Hard rule.

---

# 138. No Unsigned Offline Package

Hard rule.

---

# 139. Enterprise Private Marketplace

Optional.

---

# 140. Separate catalog.

---

# 141. Hard rule.

---

# 142. Private Marketplace Scope

Tenant/organization only.

---

# 143. Hard rule.

---

# 144. Enterprise Policy

Can restrict:

```text
allowed publishers
allowed capabilities
approved extensions
```

---

# 145. Cannot weaken platform hard security/privacy floor.

---

# 146. Hard rule.

---

# 147. Extension Update

Update flow:

```text
discover new version
verify provenance/signature
check compatibility
compute capability diff
apply policy
install/update
```

---

# 148. Hard rule.

---

# 149. Update Channel

```rust
pub enum ExtensionUpdateChannel {
    Stable,
    Beta,
    Development,
}
```

---

# 150. Hard rule.

---

# 151. Stable By Default

Hard rule.

---

# 152. Auto-Update

Allowed for signed compatible no-scope-expansion update.

---

# 153. Hard rule.

---

# 154. Scope-Expanding Update

Requires approval.

---

# 155. Hard rule.

---

# 156. Breaking Update

Requires migration/compatibility disclosure.

---

# 157. Hard rule.

---

# 158. Update Rollback

Keep previous package if safe.

---

# 159. Hard rule.

---

# 160. Data Migration

Extension-owned local data needs migration plan.

---

# 161. Hard rule.

---

# 162. Extension Data Namespace

Each extension isolated.

---

# 163. Hard rule.

---

# 164. Extension Storage

Quota-limited.

---

# 165. Hard rule.

---

# 166. Data Export

User/admin can export extension data where appropriate.

---

# 167. Hard rule.

---

# 168. Uninstall

Must define:

```text
remove runtime
revoke capabilities
stop background tasks
data retention/delete choice
```

---

# 169. Hard rule.

---

# 170. Uninstall Data Policy

```rust
pub enum ExtensionDataDisposition {
    DeleteNow,
    RetainTemporarily,
    ExportThenDelete,
}
```

---

# 171. Hard rule.

---

# 172. No Orphan Background Process

Hard rule.

---

# 173. Revocation

Package can be revoked.

---

# 174. Revocation Reasons

```rust
pub enum ExtensionRevocationReason {
    Malware,
    CredentialTheft,
    PrivacyViolation,
    SupplyChainCompromise,
    CriticalVulnerability,
    PolicyViolation,
}
```

---

# 175. Hard rule.

---

# 176. Revocation State

```rust
pub enum ExtensionRevocationState {
    Advisory,
    BlockNewInstalls,
    DisableOnNextStart,
    EmergencyDisable,
}
```

---

# 177. Hard rule.

---

# 178. Emergency Disable

Reserved for severe verified risk.

---

# 179. Hard rule.

---

# 180. Revocation Propagation

Signed revocation list.

---

# 181. Hard rule.

---

# 182. Revocation List Epoch

```rust
pub struct ExtensionRevocationEpoch(pub u64);
```

---

# 183. Anti-Rollback

Hard rule.

---

# 184. Offline Revocation

Use cached signed list.

---

# 185. Stale list handled conservatively for high-risk packages.

---

# 186. Hard rule.

---

# 187. Runtime Revocation Enforcement

Revoked extension loses capabilities.

---

# 188. Hard rule.

---

# 189. Emergency Disable Cannot Delete User Data Automatically

Hard rule.

---

# 190. Quarantine

Revoked package may be quarantined for investigation.

---

# 191. Hard rule.

---

# 192. Revocation Evidence

Archived.

---

# 193. Hard rule.

---

# 194. Certification Suspension

Separate from package revocation.

---

# 195. Hard rule.

---

# 196. Compatibility Revocation

Part 129.

---

# 197. Hard rule.

---

# 198. Publisher Suspension

May block new submissions.

---

# 199. Existing packages reviewed independently.

---

# 200. Hard rule.

---

# 201. Publisher Removal

Does not automatically delete installed extension data.

---

# 202. Hard rule.

---

# 203. Marketplace Moderation

Moderation covers:

```text
malware
misleading claims
spam
impersonation
policy violations
```

---

# 204. Hard rule.

---

# 205. No Ideological/Popularity Moderation In Technical Marketplace Baseline

Hard rule.

---

# 206. Appeals

Publisher can appeal rejection/delisting.

---

# 207. Hard rule.

---

# 208. Appeal Record

```rust
pub struct MarketplaceAppeal {
    pub listing: MarketplaceListingId,
    pub reason: String,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 209. Hard rule.

---

# 210. Review Transparency

Provide reason codes.

---

# 211. Hard rule.

---

# 212. No Secret Unexplained Delisting Except Security-Sensitive Detail

Hard rule.

---

# 213. Marketplace Listing Privacy

Do not expose publisher private account details.

---

# 214. Hard rule.

---

# 215. Publisher Organization Display

Only intended public profile fields.

---

# 216. Hard rule.

---

# 217. Marketplace Analytics

Allowed aggregate:

```text
search volume by category
install count aggregate
update adoption aggregate
revocation count
```

---

# 218. No user-level install history for ranking.

---

# 219. Hard rule.

---

# 220. Local Install History

Can remain local unless sync intentionally enabled.

---

# 221. Hard rule.

---

# 222. Server-Side Install Records

Only where needed for licensing/admin/update/security.

---

# 223. Minimize retention.

---

# 224. Hard rule.

---

# 225. Anonymous Mode

Marketplace requests must not expose private communication identity.

---

# 226. Hard rule.

---

# 227. Marketplace Account Identity

Separate from anonymous messaging identity.

---

# 228. Hard rule.

---

# 229. Anonymous Extension Restrictions

Maximum anonymity mode may prohibit:

```text
presence harvesting
peer metadata access
unrestricted network
```

---

# 230. Hard rule.

---

# 231. Privacy Mode Compatibility

Extension declares support.

---

# 232. Hard rule.

---

# 233. Extension Privacy Profile

```rust
pub struct ExtensionPrivacyProfile {
    pub supports_private_mode: bool,
    pub supports_anonymous_mode: bool,
    pub data_classes: BTreeSet<IntegrationDataClass>,
}
```

---

# 234. Hard rule.

---

# 235. No False Privacy Claim

Claims must be reviewable.

---

# 236. Hard rule.

---

# 237. Extension Resource Limits

Bound:

```text
CPU
memory
storage
network
background jobs
```

---

# 238. Hard rule.

---

# 239. Resource Quota

```rust
pub struct ExtensionResourceQuota {
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub network_bytes_per_hour: u64,
    pub max_background_jobs: u32,
}
```

---

# 240. Hard rule.

---

# 241. Abuse Isolation

One extension cannot starve host.

---

# 242. Hard rule.

---

# 243. Crash Isolation

Extension crash does not crash host where practical.

---

# 244. Hard rule.

---

# 245. Native Extension Crash

Run out-of-process if possible.

---

# 246. Hard rule.

---

# 247. Extension Logging

Scoped.

---

# 248. No host-private logs.

---

# 249. Hard rule.

---

# 250. Extension Diagnostics

Expose own runtime state.

---

# 251. Hard rule.

---

# 252. No host-global diagnostics without capability.

---

# 253. Hard rule.

---

# 254. Extension Telemetry

Developer can receive extension-specific aggregate telemetry only where user/admin policy permits.

---

# 255. Hard rule.

---

# 256. No Hidden Analytics SDK Injection

Hard rule.

---

# 257. Third-Party Network Calls

Declared in manifest.

---

# 258. Hard rule.

---

# 259. Domain Allowlist

Possible.

---

# 260. Hard rule.

---

# 261. Dynamic Domain Expansion

Requires updated manifest/policy.

---

# 262. Hard rule.

---

# 263. Extension Dependencies

Package may depend on:

```text
SDK
host API
other extension
```

---

# 264. Hard rule.

---

# 265. Dependency Graph

No hidden runtime downloads.

---

# 266. Hard rule.

---

# 267. Extension-to-Extension Dependency

Explicit.

---

# 268. Hard rule.

---

# 269. Dependency Confusion Defense

Use canonical IDs/digests.

---

# 270. Hard rule.

---

# 271. Marketplace Package Registry

```rust
pub trait ExtensionRegistryService {
    fn package(
        &self,
        id: ExtensionPackageId,
    ) -> Result<ExtensionPackageMetadata, MarketplaceError>;
}
```

---

# 272. Marketplace Service

```rust
pub trait ExtensionMarketplaceService {
    fn listing(
        &self,
        id: MarketplaceListingId,
    ) -> Result<MarketplaceListing, MarketplaceError>;

    fn search(
        &self,
        query: MarketplaceQuery,
    ) -> Result<Vec<MarketplaceListingSummary>, MarketplaceError>;
}
```

---

# 273. Install Service

```rust
pub trait ExtensionInstallService {
    fn install(
        &self,
        extension: ExtensionId,
        version: ExtensionVersion,
        approval: InstallApproval,
    ) -> Result<ExtensionInstallRecord, MarketplaceError>;
}
```

---

# 274. Revocation Service

```rust
pub trait ExtensionRevocationService {
    fn state(
        &self,
        package: ExtensionPackageId,
    ) -> Result<ExtensionRevocationState, MarketplaceError>;
}
```

---

# 275. No User-Analytics API

Hard rule.

---

# 276. Error Taxonomy

```rust
pub enum MarketplaceError {
    ExtensionUnknown,
    PackageUnknown,
    PublisherUnknown,
    SignatureInvalid,
    CompatibilityMismatch,
    CapabilityApprovalRequired,
    CertificationRequired,
    PackageRevoked,
    ListingSuspended,
    RevocationListStale,
    QuotaExceeded,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 277. Marketplace SLOs

Examples:

```text
signed package metadata available
revocation propagation within target
search index freshness
update metadata freshness
```

---

# 278. Security SLO

```text
0 unsigned package installed through marketplace path
0 revoked package newly installed
0 scope-expanding update auto-approved
```

---

# 279. Privacy SLO

```text
0 behavioral user profile required for discovery
0 anonymous messaging identity used as marketplace identity
0 hidden telemetry capability granted
```

---

# 280. Failure Modes

```text
malicious publisher
compromised package
stale revocation list
misleading permission description
ranking manipulation
```

---

# 281. Malicious Publisher

Suspend publisher and affected submissions.

---

# 282. Hard rule.

---

# 283. Compromised Package

Revoke package.

---

# 284. Hard rule.

---

# 285. Stale Revocation List

Restrictive behavior for risky installation/update.

---

# 286. Hard rule.

---

# 287. Misleading Permissions

Block listing/update until corrected.

---

# 288. Hard rule.

---

# 289. Ranking Manipulation

Use transparent bounded ranking signals.

---

# 290. Hard rule.

---

# 291. Testing

Need marketplace testkit.

---

# 292. Test Scenarios

```text
signed WASM extension
native extension review
scope-expanding update
package revocation
offline install
```

---

# 293. Signature Test

Unsigned package rejected.

---

# 294. Compatibility Test

Unsupported host version rejected.

---

# 295. Capability Test

New message-content scope requires approval.

---

# 296. Certification Test

Privileged native extension requires certification.

---

# 297. Update Test

Scope-preserving signed compatible update may auto-update.

---

# 298. Scope Expansion Test

Auto-update blocked.

---

# 299. Revocation Test

Revoked package cannot newly install/run privileged actions.

---

# 300. Offline Revocation Test

Expired revocation snapshot does not permissively install high-risk package.

---

# 301. Data Test

Uninstall removes runtime grants/background jobs.

---

# 302. Privacy Test

Marketplace search does not require user behavior profile.

---

# 303. Anonymous Mode Test

Extension cannot access prohibited peer metadata.

---

# 304. Publisher Key Test

Compromised key rotation preserves package history.

---

# 305. Fuzzing

Fuzz:

```text
extension manifests
package metadata
marketplace queries
revocation lists
update manifests
```

---

# 306. Property Tests

Properties:

```text
unsigned package can never transition to installed
revoked package can never gain new runtime capability
scope-expanding update can never apply without renewed approval
anonymous-mode extension can never gain prohibited metadata capability
```

---

# 307. Formal Verification Targets

Strong candidates:

```text
install state machine
capability grant/update
revocation propagation
publisher/package lineage
```

---

# 308. Kani Candidate

signature/capability/revocation invariants.

---

# 309. TLA+ Candidate

submit → review → publish → install → update → revoke → retire.

---

# 310. Loom Candidate

concurrent update download + revocation + runtime activation.

---

# 311. Performance

Marketplace not on core messaging hot path.

---

# 312. Hard rule.

---

# 313. Search Index

Derived/rebuildable.

---

# 314. Hard rule.

---

# 315. Package CDN

May distribute immutable signed blobs.

---

# 316. CDN not authority.

---

# 317. Hard rule.

---

# 318. Cached Marketplace Metadata

Signed/freshness-aware.

---

# 319. Hard rule.

---

# 320. Storage

Separate:

```text
publishers
extension identities
package metadata
listings
certifications
reviews
revocations
install metadata
```

---

# 321. Package blobs content-addressed.

---

# 322. Hard rule.

---

# 323. No behavioral user profile warehouse.

---

# 324. Hard rule.

---

# 325. Partitioning

By:

```text
extension
publisher
version
category
platform
```

---

# 326. No person/user analytics partition.

---

# 327. Hard rule.

---

# 328. Crate Layout

Recommended:

```text
crates/
├── siar-marketplace-core/
├── siar-extension-registry/
├── siar-publisher-registry/
├── siar-extension-review/
├── siar-extension-certification/
├── siar-extension-discovery/
├── siar-extension-install/
├── siar-extension-revocation/
├── siar-marketplace-observability/
└── siar-marketplace-testkit/
```

---

# 329. `siar-marketplace-core`

Owns:

```text
ExtensionId
ExtensionPackageId
PublisherId
MarketplaceError
```

---

# 330. `siar-extension-registry`

Package manifests/provenance/compatibility.

---

# 331. `siar-publisher-registry`

Publisher identity/verification/signing keys.

---

# 332. `siar-extension-review`

Automated/manual findings.

---

# 333. `siar-extension-certification`

Privileged extension certification.

---

# 334. `siar-extension-discovery`

Search/filter/ranking without behavioral profiling.

---

# 335. `siar-extension-install`

Permission disclosure/grants/install/update/uninstall.

---

# 336. `siar-extension-revocation`

Signed revocation lists/emergency disable.

---

# 337. `siar-marketplace-observability`

Aggregate ecosystem health only.

---

# 338. `siar-marketplace-testkit`

package/update/revocation/privacy tests.

---

# 339. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Extension identity, package digest, publisher identity, listing identity, certification, signature, manifest request, and runtime capability grant are distinct entities and cannot be conflated.
2. Every marketplace-distributed package is content-addressed, signed, provenance/SBOM-linked, compatibility-declared, and verified before installation or update.
3. Requested capabilities, data classes, network access, file access, admin authority, and privacy-mode support are disclosed before install; scope expansion requires renewed approval and, where applicable, re-certification.
4. Runtime authority comes from explicit active capability grants, not from package manifest claims, publisher verification status, popularity, marketplace rank, or certification alone.
5. WASM or equivalent sandboxing is preferred for third-party extensions, while native extensions require stronger isolation/review/certification and cannot receive unrestricted host process access by default.
6. Marketplace discovery/ranking uses technical relevance, compatibility, certification, maintenance, and aggregate non-behavioral signals and cannot depend on private conversation content, cross-service tracking, or persistent per-user behavioral profiles.
7. Trust is shown as factual multidimensional signals—publisher verification, signatures, SBOM, reproducibility, certification, source availability, freshness—not one opaque trust score or popularity-derived security claim.
8. Signed anti-rollback revocation state can block new installs, updates, or runtime authority and can emergency-disable severe-risk packages without deleting user data or rewriting history.
9. Extension updates preserve compatibility and capability policy; scope-preserving updates may be automated, while scope-expanding/breaking updates require explicit approval and migration disclosure.
10. Marketplace analytics, reviews, ratings, installs, and searches are minimized/aggregated and cannot become user profiling, developer ranking, publisher surveillance, or a hidden advertising system.
11. Anonymous/private modes restrict extension metadata/network/presence access according to platform privacy floors, and extensions cannot silently weaken anonymity or disclose detailed peer/client fingerprints.
12. The extension ecosystem integrates with the SDK, developer portal/toolchain, compatibility registry, product lifecycle, supply-chain security, assurance/certification archives, authorization, plugins, risk/PIR, updates, audit, and privacy controls without creating a side channel around SIAR's security, privacy, anonymity, or tenant boundaries.
```

---

# 340. Initial Production Scope

Implement first:

```text
typed ExtensionId/PackageId/PublisherId
extension manifest
publisher registry
publisher package signing
SBOM/provenance metadata
automated submission checks
manual review findings
WASM extension class
native-extension high-risk path
marketplace listing
capability/data/network/file disclosure
local approval/grants
compatibility checks
text/category/platform discovery
factual trust signals
signed package installation
scope-aware updates
signed revocation list
emergency disable
uninstall/data disposition
enterprise private catalog policy
privacy-safe marketplace metrics
marketplace testkit
```

Then add:

```text
reproducible-build verification service
portable marketplace bundles for offline/air-gapped use
extension dependency solver
cross-device extension-state sync
publisher transparency reports
local/on-device recommendation
formal install/update/revocation verification
```

---

# 341. Definition of Done

Part 133 is complete when:

- extension/package/publisher/listing identities are distinct;
- package signature/provenance/SBOM checks exist;
- requested capabilities are explicit;
- runtime grants are separate from requests;
- scope expansion requires approval;
- discovery works without behavioral profiling;
- trust signals are factual/multidimensional;
- WASM/native risk levels differ;
- signed revocation and emergency disable exist;
- update compatibility/capability diffs are enforced;
- uninstall revokes runtime authority and handles data;
- anonymous mode prevents prohibited metadata access;
- no pay-to-rank or hidden advertising path exists;
- package/update/revocation/privacy/fuzz/formal tests are specified.

---

# 342. Final Architecture

```text
                 EXTENSION PUBLISHER
                         │
                         ▼
                  SIGNED PACKAGE
                         │
                         ▼
              REVIEW / CERTIFICATION
                         │
                         ▼
                  MARKETPLACE INDEX
                         │
              ┌──────────┼──────────┐
              │          │          │
          DISCOVERY    INSTALL     UPDATE
              │          │          │
              └──────────┼──────────┘
                         ▼
                  CAPABILITY GRANT
                         │
                         ▼
                    SANDBOXED RUN
                         │
                         ▼
                REVOKE / UNINSTALL
```

Developer-ecosystem safety model:

```text
signed packages
+
publisher provenance
+
explicit capabilities
+
sandboxed runtime
+
compatibility checks
+
factual trust signals
+
privacy-safe discovery
+
signed revocation
```

not:

```text
rank extensions by user tracking, auto-grant every requested permission, let native plugins run inside the host process, and rely on popularity as proof of security
```

---

# 343. Final Principle

An extension ecosystem is trustworthy when users and administrators can understand who published a package, what authority it requests, what evidence supports it, and how quickly that authority can be revoked.

The correct model is:

```text
identify publisher and package
+
sign and verify provenance
+
declare capabilities honestly
+
review risk by extension class
+
discover without behavioral surveillance
+
install with explicit grants
+
update with capability diffs
+
revoke compromised packages quickly
+
preserve user data and history
+
never let ecosystem growth bypass platform trust
```

This architecture gives SIAR a privacy-preserving extension ecosystem foundation for package distribution, discovery, publisher identity, trust signals, review, certification, updates, revocation, runtime isolation, and marketplace governance while preserving the anonymity, local-first, least-authority, SDK, developer-portal, developer-toolchain, compatibility, and anti-surveillance guarantees established across Parts 34–132.
