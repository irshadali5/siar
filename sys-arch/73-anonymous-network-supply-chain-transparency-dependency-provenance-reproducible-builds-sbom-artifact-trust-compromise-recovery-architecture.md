# Core System Architecture Part 73 — Anonymous Network Supply-Chain Transparency, Dependency Provenance, Reproducible Builds, SBOM, Artifact Trust & Compromise Recovery Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 73  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 49, 52–53, 61–66, 71–72  

**Primary purpose:** define SIAR's complete software supply-chain trust architecture from source input to production artifact, including dependency provenance, registry/mirror trust, vendoring, build isolation, reproducibility, SBOM, attestations, artifact transparency, signing, release promotion, compromise detection, artifact revocation, dependency incident response, and compromise recovery.

---

# 1. Purpose

A secure application can still be compromised before it runs.

Attack paths include:

```text
malicious dependency
compromised package registry
poisoned Git dependency
compromised CI runner
tampered build toolchain
malicious build script
artifact substitution
stolen signing key
poisoned binary cache
```

The governing principle is:

> **A production artifact should be explainable, reproducible, attributable, verifiable, and revocable from its final binary all the way back to its declared source and dependency inputs.**

---

# 2. Architectural Position

```text
Source
  │
  ▼
Dependency Resolution
  │
  ▼
Verified Inputs
  │
  ▼
Hermetic Build
  │
  ▼
Reproducibility Check
  │
  ▼
SBOM + Provenance
  │
  ▼
Signing + Transparency
  │
  ▼
Promotion
  │
  ▼
Production Verification
```

---

# 3. Core Separation

Keep distinct:

```text
source provenance
dependency provenance
toolchain provenance
build provenance
artifact identity
signing authority
release approval
runtime trust
```

---

# 4. Non-Goals

Part 73 does not assume:

```text
package registries are inherently trustworthy
a signature proves source correctness
SBOM alone equals supply-chain security
one CI provider is a complete root of trust
```

---

# 5. Supply-Chain Trust Domains

```rust
pub enum SupplyChainDomain {
    Source,
    Dependency,
    Toolchain,
    BuildEnvironment,
    BuildOutput,
    ArtifactRegistry,
    Signing,
    Promotion,
}
```

---

# 6. Source Provenance

Every release source has:

```text
repository
commit
tree digest
tag/release ref
review status
```

---

# 7. Source Revision

```rust
pub struct SourceRevision {
    pub repository: RepositoryId,
    pub commit: GitCommitHash,
    pub tree_digest: ContentDigest,
}
```

---

# 8. No Branch Name As Release Identity

Hard rule.

---

# 9. Release Source

Exact commit/tree.

---

# 10. Git Tag

Convenience only unless signed/verified.

---

# 11. Protected Branches

Recommended.

---

# 12. Mandatory Review

Critical code paths.

---

# 13. Code Ownership

Useful.

---

# 14. Source Signature

Optional but valuable.

---

# 15. Signed Commit/Tag

Not sufficient alone.

---

# 16. Dependency Classes

```rust
pub enum DependencySource {
    Registry,
    Git,
    Vendored,
    LocalWorkspace,
    SystemPackage,
}
```

---

# 17. Preferred Dependency Order

```text
workspace
vendor/pinned registry
pinned git
system package only when necessary
```

---

# 18. Registry Dependency

Must include:

```text
name
version
checksum
source registry
```

---

# 19. Cargo.lock

Required.

---

# 20. Lockfile Integrity

CI verifies no unexpected mutation.

---

# 21. No Unlocked Production Build

Hard rule.

---

# 22. Git Dependency

Must pin exact commit.

---

# 23. No Floating Branch/Tag

Hard rule.

---

# 24. Git Dependency Record

```rust
pub struct GitDependencyRef {
    pub repository: RepositoryId,
    pub commit: GitCommitHash,
    pub tree_digest: ContentDigest,
}
```

---

# 25. Vendoring

Recommended for high-assurance releases.

---

# 26. Vendor Snapshot

Content-addressed.

---

# 27. Vendor Manifest

```rust
pub struct VendorManifest {
    pub dependencies: Vec<VendoredDependency>,
    pub digest: ContentDigest,
}
```

---

# 28. No Manual Vendor Mutation

Hard rule.

---

# 29. Dependency Mirrors

Useful for availability.

---

# 30. Mirror Trust

Mirror is cache, not authority.

---

# 31. Verify Checksums Against Approved Metadata

Required.

---

# 32. No Trust In Mirror TLS Alone

Hard rule.

---

# 33. Registry Compromise Model

Assume:

```text
malicious package version
metadata substitution
account takeover
```

---

# 34. Defense

```text
checksums
lockfile
review
vendor snapshot
provenance
```

---

# 35. Dependency Provenance

Track origin.

---

# 36. Provenance Record

```rust
pub struct DependencyProvenance {
    pub package: PackageId,
    pub source: DependencySource,
    pub source_digest: ContentDigest,
    pub publisher: Option<PublisherIdentity>,
}
```

---

# 37. Publisher Identity

Informational unless independently verified.

---

# 38. No "Trusted Maintainer" Shortcut

Hard rule.

---

# 39. Build Script Risk

Rust crates may include:

```text
build.rs
proc macros
native compilation
```

---

# 40. Build Script Inventory

Required.

---

# 41. Procedural Macro Inventory

Required.

---

# 42. Native Code Inventory

Required.

---

# 43. High-Risk Dependency Types

```rust
pub enum DependencyRiskClass {
    PureLibrary,
    ProcMacro,
    BuildScript,
    NativeFfi,
    CodeGenerator,
}
```

---

# 44. Risk Policy

Higher scrutiny for executable build-time code.

---

# 45. No Unreviewed New Build Script In Critical Release

Hard rule.

---

# 46. Dependency Review

For new/updated crate:

```text
license
maintenance
security history
build scripts
unsafe use
network behavior
```

---

# 47. Unsafe Inventory

Track.

---

# 48. `cargo deny`

Recommended.

---

# 49. `cargo audit`

Recommended.

---

# 50. Advisory Intake

Automated.

---

# 51. Vulnerability Severity

```rust
pub enum SupplyChainSeverity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}
```

---

# 52. Critical Dependency Advisory

Release block unless explicitly not reachable/affected with evidence.

---

# 53. Reachability

Useful but not sole criterion.

---

# 54. Toolchain Provenance

Pin:

```text
Rust version
rustup channel/date
LLVM where relevant
linker
native tools
```

---

# 55. `rust-toolchain.toml`

Required.

---

# 56. Toolchain Digest

Prefer verified installer/archive checksum.

---

# 57. No Auto-Latest Toolchain

Hard rule.

---

# 58. Toolchain Mirror

Can be internal.

---

# 59. Mirror Verification

Checksum/signature.

---

# 60. Build Environment

Hermetic.

---

# 61. Inputs Must Be Declared

Hard rule.

---

# 62. Build Environment Types

```rust
pub enum BuildEnvironmentKind {
    Nix,
    Container,
    Vm,
    DedicatedHost,
}
```

---

# 63. Nix/Flake

Strong candidate.

---

# 64. Container

Useful but does not alone guarantee hermeticity.

---

# 65. Dedicated Host

Useful for high-assurance signing/build.

---

# 66. Build Network Access

Disable where feasible.

---

# 67. No Network During Final Build

Preferred hard rule.

---

# 68. Why

Prevents undeclared dependency fetch.

---

# 69. Build Input Store

Pre-populated verified inputs.

---

# 70. Build User

Unprivileged.

---

# 71. Ephemeral Build Worker

Preferred.

---

# 72. No Production Credentials

Hard rule.

---

# 73. No Signing Key In Build Environment

Hard rule.

---

# 74. Build Isolation

Separate from release signing.

---

# 75. Build Provenance

```rust
pub struct BuildProvenance {
    pub source: SourceRevision,
    pub lockfile_digest: ContentDigest,
    pub vendor_digest: Option<ContentDigest>,
    pub toolchain_digest: ContentDigest,
    pub environment_digest: ContentDigest,
    pub output_digest: ContentDigest,
}
```

---

# 76. Provenance Signed

By build attestation identity.

---

# 77. Build Attestation Identity

Separate from release signer.

---

# 78. No Same Key For Build And Release Signing

Hard rule.

---

# 79. Build Attestation

Means:

```text
this builder claims these inputs produced this output
```

---

# 80. Not:

```text
this artifact is approved for production
```

---

# 81. Reproducible Builds

Goal:

```text
same declared inputs
→ same output digest
```

---

# 82. Reproducibility Level

```rust
pub enum ReproducibilityLevel {
    BestEffort,
    Deterministic,
    IndependentlyReproduced,
}
```

---

# 83. BestEffort

Some nondeterminism remains.

---

# 84. Deterministic

Same controlled environment gives same output.

---

# 85. IndependentlyReproduced

Different trusted builders match.

---

# 86. High-Assurance Goal

`IndependentlyReproduced`.

---

# 87. Determinism Hazards

```text
timestamps
absolute paths
filesystem order
hostnames
locale
random build IDs
```

---

# 88. Normalize Inputs

Where possible.

---

# 89. `SOURCE_DATE_EPOCH`

Useful concept.

---

# 90. No Build Timestamp In Critical Binary Unless Reproducible

Hard rule.

---

# 91. Reproducibility Manifest

```rust
pub struct ReproducibilityReport {
    pub builders: Vec<BuilderIdentity>,
    pub output_digests: Vec<ContentDigest>,
    pub level: ReproducibilityLevel,
}
```

---

# 92. Mismatch

Release blocker until explained.

---

# 93. Independent Builder

Different trust domain preferred.

---

# 94. Same CI Provider Twice

Weaker evidence.

---

# 95. Independent Environment

Better.

---

# 96. Binary Reproducibility

Per target.

---

# 97. Package Reproducibility

Separate from binary.

---

# 98. OCI Image Reproducibility

Separate.

---

# 99. VM Image Reproducibility

Separate.

---

# 100. SBOM

Mandatory.

---

# 101. SBOM Includes

```text
package name
version
source
checksum
license
dependency relationships
native components
```

---

# 102. SBOM Formats

Possible:

```text
CycloneDX
SPDX
```

---

# 103. Canonical Internal Model

Recommended.

---

# 104. SBOM Type

```rust
pub struct Sbom {
    pub artifact: ContentDigest,
    pub components: Vec<SbomComponent>,
}
```

---

# 105. SBOM Generated From Resolved Build Graph

Preferred.

---

# 106. No Handwritten SBOM

Hard rule.

---

# 107. SBOM Digest

Linked to release manifest.

---

# 108. License Metadata

Included.

---

# 109. Unknown License

Release policy decision.

---

# 110. Denied License

Release block.

---

# 111. Transitive Dependency Visibility

Required.

---

# 112. Native System Libraries

Included where linked/runtime required.

---

# 113. Container Base Image

Included.

---

# 114. Build Tool Dependencies

Separate build SBOM useful.

---

# 115. Runtime SBOM

Production-facing.

---

# 116. Build SBOM

Build-chain facing.

---

# 117. Artifact Identity

Content digest.

---

# 118. No Mutable Tag As Authority

Hard rule.

---

# 119. Artifact Descriptor

```rust
pub struct TrustedArtifact {
    pub digest: ContentDigest,
    pub provenance: BuildProvenanceRef,
    pub sbom: SbomRef,
    pub signatures: Vec<ArtifactSignature>,
}
```

---

# 120. Artifact Registry

Immutable objects.

---

# 121. Registry Metadata

Versioned.

---

# 122. Registry Compromise

Must not allow undetected substitution.

---

# 123. Node Verifies

```text
digest
signature
provenance policy
security epoch
```

---

# 124. No Registry Trust Without Verification

Hard rule.

---

# 125. Artifact Transparency

Append-only release log.

---

# 126. Transparency Entry

```rust
pub struct ArtifactTransparencyEntry {
    pub artifact: ContentDigest,
    pub release: SoftwareVersion,
    pub provenance_digest: ContentDigest,
    pub sbom_digest: ContentDigest,
    pub signatures: Vec<ArtifactSignature>,
}
```

---

# 127. Transparency Log

Can be:

```text
internal append-only
public
federated mirrors
```

---

# 128. Goal

Detect:

```text
equivocation
hidden release
artifact substitution
```

---

# 129. No User Data In Transparency

Hard rule.

---

# 130. Release Signing

Part 72.

---

# 131. Signing Flow

```text
build
→ reproduce
→ verify SBOM
→ verify provenance
→ qualify tests
→ approve
→ sign release manifest
```

---

# 132. HSM Signer

Signs approved digest/manifest.

---

# 133. No Direct Artifact Signing Without Qualification

Hard rule.

---

# 134. Release Manifest

```rust
pub struct SupplyChainReleaseManifest {
    pub version: SoftwareVersion,
    pub artifact: ContentDigest,
    pub provenance: ContentDigest,
    pub sbom: ContentDigest,
    pub verification: ContentDigest,
    pub signatures: Vec<ArtifactSignature>,
}
```

---

# 135. Exact Promotion

Same artifact hash from candidate to production.

---

# 136. No Rebuild For Production

Hard rule.

---

# 137. Promotion State

```rust
pub enum ArtifactPromotionState {
    Built,
    Verified,
    Reproduced,
    Qualified,
    Signed,
    Candidate,
    Production,
    Revoked,
}
```

---

# 138. State Transitions

Monotonic except revocation.

---

# 139. No Skip Verification Stage

Hard rule.

---

# 140. Artifact Revocation

Needed.

---

# 141. Revocation Record

```rust
pub struct ArtifactRevocation {
    pub artifact: ContentDigest,
    pub reason: ArtifactRevocationReason,
    pub effective_at: Timestamp,
}
```

---

# 142. Revocation Reasons

```rust
pub enum ArtifactRevocationReason {
    DependencyCompromise,
    BuildCompromise,
    SigningCompromise,
    CriticalVulnerability,
    ProvenanceFailure,
}
```

---

# 143. Revoked Artifact

Cannot be newly deployed.

---

# 144. Existing Deployment

Policy determines:

```text
quarantine
drain
repair-forward
```

---

# 145. No Silent Continued Trust

Hard rule.

---

# 146. Dependency Compromise

Example:

```text
maintainer account compromise
malicious crate version
```

---

# 147. Response

```text
identify affected versions
map artifacts via SBOM
revoke affected artifacts if necessary
rebuild from safe dependency
rotate credentials if exposure occurred
```

---

# 148. SBOM Enables Blast-Radius Analysis

Critical.

---

# 149. Dependency Reachability

Optional optimization.

---

# 150. But

Known malicious build-time code may compromise build even if runtime unreachable.

---

# 151. Build-Time Compromise

Treat more seriously.

---

# 152. Poisoned Proc Macro

Can alter generated code.

---

# 153. Poisoned build.rs

Can alter build output.

---

# 154. Therefore

Build-time dependency compromise may invalidate artifacts broadly.

---

# 155. Registry Poisoning Incident

Scenario.

---

# 156. Response

Use:

```text
lockfile
vendor snapshot
trusted mirror
```

---

# 157. No Emergency Update From Unverified Alternate Registry

Hard rule.

---

# 158. Build Runner Compromise

Severe.

---

# 159. Response

```text
revoke builder identity
invalidate provenance
independently reproduce artifact
rotate build credentials
```

---

# 160. If Reproduction Differs

Treat artifact as suspect.

---

# 161. Build Attestation Compromise

Does not automatically compromise release signer.

---

# 162. Separation benefit.

---

# 163. Release Signing Key Compromise

Part 72.

---

# 164. Severe response.

---

# 165. Artifact Registry Compromise

If signatures/digests intact:

```text
availability issue
```

more than trust issue.

---

# 166. If transparency log compromised

Use mirrors/witnesses.

---

# 167. Transparency Witness

Optional.

---

# 168. Multi-Witness

Better for critical releases.

---

# 169. Supply-Chain Root Of Trust

Not one thing.

---

# 170. It is distributed across:

```text
source review
dependency verification
build environment
reproduction
release approval
HSM signing
runtime verification
```

---

# 171. Defense In Depth

Mandatory.

---

# 172. Dependency Update Workflow

```text
proposal
→ review
→ sandbox build
→ tests
→ diff SBOM
→ merge
```

---

# 173. SBOM Diff

Critical.

---

# 174. Shows:

```text
added components
removed components
version changes
license changes
```

---

# 175. New Native Dependency

High-risk review.

---

# 176. New Build Script

High-risk review.

---

# 177. New Proc Macro

High-risk review.

---

# 178. Dependency Pinning

Exact resolved version.

---

# 179. Semver Range

Allowed in Cargo.toml for developer convenience.

---

# 180. Lockfile defines release.

---

# 181. No `cargo update` In Release Job

Hard rule.

---

# 182. Dependency Refresh

Separate reviewed operation.

---

# 183. Vendor Refresh

Separate reviewed operation.

---

# 184. Toolchain Update Workflow

```text
proposal
→ reproducibility test
→ security review
→ benchmark
→ rollout
```

---

# 185. Compiler Update

Can change binary significantly.

---

# 186. Treat as supply-chain change.

---

# 187. Build Cache

Useful.

---

# 188. Cache Key

Includes full declared inputs.

---

# 189. Cached Output

Still verified.

---

# 190. No Trust In Cache Hit Alone

Hard rule.

---

# 191. Remote Cache

Signed.

---

# 192. Cache Poisoning

Detected via digest mismatch.

---

# 193. Base Images

Pinned by digest.

---

# 194. No `latest`

Hard rule.

---

# 195. Base Image SBOM

Included.

---

# 196. VM Base Image

Pinned/checksummed.

---

# 197. Host Packages

Declarative.

---

# 198. Nix Store Paths

Strong provenance signal.

---

# 199. But

Still need upstream trust/review.

---

# 200. Offline Build

High assurance.

---

# 201. Air-Gapped Build

Possible for release candidate.

---

# 202. Need imported verified source/vendor/toolchain bundle.

---

# 203. Offline Build Bundle

```rust
pub struct OfflineBuildBundle {
    pub source: SourceRevision,
    pub vendor_manifest: VendorManifest,
    pub toolchain_digest: ContentDigest,
    pub environment_digest: ContentDigest,
}
```

---

# 204. Bundle Signed.

---

# 205. No Network Dependency.

---

# 206. Build Reproduction Service

```rust
pub trait ReproducibilityVerifier {
    fn reproduce(
        &self,
        provenance: &BuildProvenance,
    ) -> Result<ReproducibilityReport, SupplyChainError>;
}
```

---

# 207. Supply-Chain Policy

```rust
pub struct SupplyChainPolicy {
    pub required_reproducibility: ReproducibilityLevel,
    pub require_sbom: bool,
    pub require_signed_provenance: bool,
    pub allowed_dependency_sources: BTreeSet<DependencySource>,
}
```

---

# 208. Critical Production Policy

Could require:

```text
SBOM
signed provenance
locked dependencies
independent reproduction
```

---

# 209. Small Self-Hosted

May use weaker profile.

---

# 210. Assurance Disclosure

Explicit.

---

# 211. Supply-Chain Assurance Level

```rust
pub enum SupplyChainAssurance {
    Development,
    Verified,
    Reproducible,
    HighAssurance,
}
```

---

# 212. No False High-Assurance Claim

Hard rule.

---

# 213. CI Trust Model

Untrusted pull request code cannot access:

```text
release secrets
signing proposal authority
production credentials
```

---

# 214. Fork PR

Fully isolated.

---

# 215. Protected Release Workflow

Separate.

---

# 216. Runner Ephemerality

Recommended.

---

# 217. Reuse

Avoid for high-assurance builds.

---

# 218. Runner Image

Pinned.

---

# 219. Runner Attestation

Part 71.

---

# 220. Builder Attestation

Optional but useful.

---

# 221. Build Worker Identity

```rust
pub struct BuilderIdentity(pub [u8; 32]);
```

---

# 222. Builder Trust

Scoped.

---

# 223. Compromised Builder

Revocable.

---

# 224. Provenance Must Record Builder.

---

# 225. Signing Service Trust

Part 72.

---

# 226. Promotion Service

Separate identity.

---

# 227. Separation Of Duties

Recommended:

```text
developer
builder
approver
signer
promoter
```

---

# 228. No Single Credential For All Stages

Hard rule.

---

# 229. Supply-Chain Transparency Dashboard

Shows:

```text
artifact digest
source commit
SBOM status
reproducibility
signatures
revocation status
```

---

# 230. No User Data.

---

# 231. Runtime Verification

Part 71.

---

# 232. Node startup verifies release manifest.

---

# 233. Artifact Digest Must Match.

---

# 234. SBOM Verification

Optional at node startup.

---

# 235. Better at release qualification.

---

# 236. Runtime Trust

At minimum:

```text
signed manifest
artifact digest
security epoch
```

---

# 237. Revocation Feed

Nodes receive signed revocations.

---

# 238. Anti-Rollback

Old revocation state cannot disappear.

---

# 239. Hard rule.

---

# 240. Compromise Recovery

Supply-chain compromise requires structured recovery.

---

# 241. Recovery Phases

```rust
pub enum SupplyChainRecoveryPhase {
    Detect,
    Contain,
    Assess,
    RebuildTrust,
    RebuildArtifacts,
    Revoke,
    Redeploy,
    Validate,
}
```

---

# 242. Detect

Identify:

```text
dependency
builder
registry
signer
artifact
```

---

# 243. Contain

Freeze:

```text
releases
promotion
affected registry path
```

---

# 244. Assess

Map affected artifacts.

---

# 245. SBOM Query

Critical.

---

# 246. Rebuild Trust

Rotate compromised credentials/keys.

---

# 247. Rebuild Artifacts

From known-good inputs.

---

# 248. Revoke

Affected artifacts/signers/builders.

---

# 249. Redeploy

Repair forward.

---

# 250. Validate

Independent reproduction + runtime attestation.

---

# 251. No Blind Rollback To Older Vulnerable Artifact

Hard rule.

---

# 252. Repair Forward

Preferred.

---

# 253. Rollback

Only if older artifact is known-safe and security epoch allows.

---

# 254. Source Compromise

Example malicious commit merged.

---

# 255. Response

Revert + audit review chain.

---

# 256. Repository Credential Compromise

Rotate.

---

# 257. Re-signing Safe Artifact

May be necessary.

---

# 258. But

Only after trust chain restored.

---

# 259. Dependency Source Disappearance

Availability event.

---

# 260. Vendor snapshot prevents urgent fetch.

---

# 261. No Supply-Chain Availability Panic

Goal.

---

# 262. Dependency License Change

SBOM/license diff catches.

---

# 263. Policy Can Block.

---

# 264. Dependency Abandonment

Risk register.

---

# 265. Forking

Possible.

---

# 266. Fork Must receive new provenance identity.

---

# 267. No Pretend Original Upstream.

---

# 268. Crate Fork Policy

```rust
pub struct ForkedDependencyRecord {
    pub original: PackageId,
    pub fork_repository: RepositoryId,
    pub fork_commit: GitCommitHash,
}
```

---

# 269. Fork Maintenance

Explicit owner.

---

# 270. Supply-Chain Risk Register

```rust
pub struct SupplyChainRiskEntry {
    pub component: PackageId,
    pub severity: SupplyChainSeverity,
    pub owner: OwnershipRef,
    pub mitigation: RiskMitigation,
}
```

---

# 271. Orphan Critical Dependency

High risk.

---

# 272. Dependency Health

Useful signal.

---

# 273. But

Not sole security metric.

---

# 274. SBOM Retention

Keep per release.

---

# 275. Provenance Retention

Keep per release.

---

# 276. Transparency Retention

Long-term.

---

# 277. Why

Historical incident analysis.

---

# 278. Artifact Retention

Keep supported/forensic releases as policy dictates.

---

# 279. Revoked Artifact

May retain in quarantined archive for analysis.

---

# 280. Never serve accidentally.

---

# 281. Quarantine Namespace

Separate.

---

# 282. No Production Pull Access.

---

# 283. Provenance Verification API

```rust
pub trait ProvenanceVerifier {
    fn verify(
        &self,
        provenance: &BuildProvenance,
        policy: &SupplyChainPolicy,
    ) -> Result<VerifiedProvenance, SupplyChainError>;
}
```

---

# 284. Artifact Trust Evaluator

```rust
pub trait ArtifactTrustEvaluator {
    fn evaluate(
        &self,
        artifact: &TrustedArtifact,
    ) -> ArtifactTrustDecision;
}
```

---

# 285. Trust Decision

```rust
pub enum ArtifactTrustDecision {
    Trusted,
    Untrusted,
    Revoked,
    InsufficientEvidence,
}
```

---

# 286. Promotion Gate

```rust
pub trait ArtifactPromotionGate {
    fn approve(
        &self,
        artifact: &TrustedArtifact,
    ) -> Result<(), SupplyChainError>;
}
```

---

# 287. Promotion Requires All Mandatory Evidence.

---

# 288. No Manual Override For Critical Missing Evidence

Hard rule.

---

# 289. Emergency Release

May reduce nonessential test breadth.

---

# 290. But still requires:

```text
verified source
locked dependencies
artifact digest
signing
critical security gates
```

---

# 291. No Unsigned Emergency Binary

Hard rule.

---

# 292. Dependency Transparency

Can publish high-level SBOM.

---

# 293. Sensitive Infrastructure Details

May be excluded.

---

# 294. Public Release SBOM

Recommended.

---

# 295. Internal Build SBOM

More detailed.

---

# 296. Supply-Chain Observability

Safe metrics:

```text
artifacts reproduced
dependency advisories
revoked artifacts
builder health
```

---

# 297. Forbidden Metrics

No user data.

---

# 298. Supply-Chain SLOs

Examples:

```text
100% production artifacts signed
100% production releases have SBOM
0 blocked dependency versions
reproducibility coverage target
```

---

# 299. Reproduction Lag SLO

Independent build completes before production promotion for critical releases.

---

# 300. Vulnerability Response SLO

Critical advisory triaged quickly.

---

# 301. Supply-Chain Incident Playbooks

Part 65.

---

# 302. Mandatory Playbooks

```text
registry compromise
dependency compromise
CI compromise
signing-key compromise
artifact-registry compromise
```

---

# 303. Disaster Exercises

Part 70.

---

# 304. Exercise registry compromise.

---

# 305. Exercise build-runner compromise.

---

# 306. Exercise revoked release rollout.

---

# 307. Testing

Need supply-chain testkit.

---

# 308. Test Scenarios

```text
wrong checksum
lockfile drift
poisoned mirror
tampered artifact
mismatched provenance
stolen signer
```

---

# 309. Checksum Test

Wrong dependency rejected.

---

# 310. Lockfile Test

Release job refuses uncommitted lock change.

---

# 311. Git Pin Test

Floating ref rejected.

---

# 312. Offline Build Test

Build succeeds with declared inputs only.

---

# 313. Repro Test

Independent outputs match.

---

# 314. SBOM Test

All runtime components represented.

---

# 315. Signature Test

Tampered artifact rejected.

---

# 316. Registry Substitution Test

Digest mismatch rejected.

---

# 317. Revocation Test

Revoked artifact not deployed.

---

# 318. Builder Compromise Test

Bad provenance invalidates trust.

---

# 319. Signing Compromise Test

New trust chain/rotation works.

---

# 320. Base Image Test

Tag mutation has no effect because digest pinned.

---

# 321. Cache Poisoning Test

Bad cache object rejected.

---

# 322. Dependency Removal Test

Vendored build still works.

---

# 323. Fuzzing

Fuzz:

```text
SBOM parser
provenance envelope
release manifest
revocation record
```

---

# 324. Property Tests

Properties:

```text
production promotion never accepts unsigned artifact
revoked artifact never transitions back to trusted without new digest/new release
dependency input digest change always changes provenance
candidate→production preserves exact artifact digest
```

---

# 325. Formal Verification Targets

Strong candidates:

```text
artifact promotion state machine
revocation precedence
supply-chain policy evaluation
compromise recovery state
```

---

# 326. Kani Candidate

policy gate logic.

---

# 327. TLA+ Candidate

multi-builder reproducibility + signer compromise transition.

---

# 328. Loom Candidate

concurrent promotion + revocation.

---

# 329. Performance

Supply-chain verification is not hot path.

---

# 330. Node Startup

Artifact verification must remain fast enough.

---

# 331. Repro Builds

Can be expensive.

---

# 332. Acceptable.

---

# 333. Storage

SBOM/provenance small compared to artifacts.

---

# 334. Crate Layout

Recommended:

```text
crates/
├── siar-supply-chain-core/
├── siar-dependency-provenance/
├── siar-build-provenance/
├── siar-reproducibility/
├── siar-sbom/
├── siar-artifact-trust/
├── siar-artifact-transparency/
├── siar-artifact-revocation/
├── siar-supply-chain-recovery/
└── siar-supply-chain-testkit/
```

---

# 335. `siar-supply-chain-core`

Owns:

```text
trust levels
states
errors
```

---

# 336. `siar-dependency-provenance`

Registry/git/vendor input records.

---

# 337. `siar-build-provenance`

Build environment/input/output attestations.

---

# 338. `siar-reproducibility`

Independent rebuild reports.

---

# 339. `siar-sbom`

Canonical internal SBOM + exporters.

---

# 340. `siar-artifact-trust`

Trust evaluation/promotion gates.

---

# 341. `siar-artifact-transparency`

Append-only artifact log.

---

# 342. `siar-artifact-revocation`

Artifact revocation state/feed.

---

# 343. `siar-supply-chain-recovery`

Compromise recovery orchestration.

---

# 344. `siar-supply-chain-testkit`

Registry/cache/builder/signer attack simulation.

---

# 345. Error Taxonomy

```rust
pub enum SupplyChainError {
    SourceUnverified,
    LockfileMismatch,
    DependencyChecksumMismatch,
    DependencySourceDenied,
    ToolchainUnverified,
    BuildEnvironmentMismatch,
    ReproducibilityMismatch,
    SbomMissing,
    ProvenanceMissing,
    SignatureInvalid,
    ArtifactRevoked,
    PromotionDenied,
    Internal,
}
```

---

# 346. Security & Operational Invariants

Mandatory:

```text
1. Every production artifact is bound to an exact source revision, locked dependency graph, toolchain, and build environment.
2. Floating Git refs, mutable tags, and unlocked dependency resolution are forbidden for production releases.
3. Build workers have no production credentials or release-signing keys.
4. High-assurance releases include machine-generated SBOM and signed build provenance.
5. Candidate and production promotion use the exact same artifact digest; production is never rebuilt separately.
6. Artifact registries and mirrors are treated as distribution channels, not roots of trust; nodes verify signatures and digests independently.
7. Revoked artifacts, builders, and signing authorities cannot regain trust through stale metadata or rollback.
8. Build-time executable dependencies such as proc macros, build scripts, and native code receive higher review scrutiny.
9. Supply-chain compromise recovery maps affected artifacts through SBOM/provenance before redeployment.
10. Emergency releases still require verified source, locked dependencies, integrity, signing, and critical security gates.
11. Supply-chain transparency, SBOMs, and provenance contain no user data.
12. Independent reproducibility is the preferred high-assurance evidence that declared source/input state corresponds to the released binary.
```

---

# 347. Initial Production Scope

Implement first:

```text
locked Cargo dependencies
exact Git commit pinning
dependency checksums
vendor snapshot option
build/proc-macro/native dependency inventory
pinned Rust/toolchain
hermetic build environment
network-disabled final release build
machine-generated SBOM
signed build provenance
artifact transparency entry
exact artifact promotion
artifact revocation
independent reproduction for critical releases
supply-chain incident playbooks
```

Then add:

```text
multi-builder reproducibility witnesses
public transparency mirrors
automated SBOM blast-radius queries
signed dependency mirror metadata
formal promotion/revocation verification
advanced source provenance
```

---

# 348. Definition of Done

Part 73 is complete when:

- source, dependency, toolchain, build, artifact, signing, and promotion trust domains are explicit
- every production build uses exact source revision and locked dependencies
- build scripts/proc macros/native dependencies are inventoried
- release builds are isolated and preferably network-free
- build workers do not possess release-signing authority
- SBOM and provenance are generated automatically
- reproducibility levels are recorded
- independent reproduction exists for high-assurance releases
- candidate→production preserves exact artifact digest
- artifact transparency and revocation are available
- node/runtime verification does not trust registry location alone
- supply-chain compromise can be mapped to affected artifacts
- repair-forward/revocation procedures are defined
- registry, mirror, cache, builder, dependency, signer, and provenance attack tests are specified

---

# 349. Final Architecture

```text
                         SOURCE REVISION
                               │
                               ▼
                      VERIFIED DEPENDENCIES
                               │
                               ▼
                     HERMETIC BUILD INPUTS
                               │
                               ▼
                        BUILD ARTIFACT
                               │
                 ┌─────────────┼─────────────┐
                 │             │             │
             Provenance       SBOM     Reproduction
                 │             │             │
                 └─────────────┼─────────────┘
                               ▼
                     RELEASE QUALIFICATION
                               │
                               ▼
                         HSM SIGNING
                               │
                               ▼
                    TRANSPARENCY + PROMOTION
                               │
                               ▼
                    PRODUCTION VERIFICATION
```

Supply-chain trust model:

```text
exact source
+
verified dependencies
+
pinned toolchain
+
hermetic build
+
reproducibility
+
SBOM/provenance
+
HSM signing
+
artifact transparency
+
revocation
```

not:

```text
CI produced a binary, so we assume it is trustworthy
```

---

# 350. Final Principle

Software supply-chain trust is not established by one signature or one CI pipeline.

The correct model is:

```text
trace every input
+
control the build
+
reproduce independently
+
record provenance
+
sign narrowly
+
verify everywhere
+
recover explicitly from compromise
```

This architecture gives SIAR an end-to-end software supply-chain trust system capable of explaining and verifying how each production artifact was created, identifying the blast radius of dependency/build/signing compromise, and recovering without weakening the anonymous-network security guarantees established across Parts 34–72.
