# Core System Architecture Part 142 — Anonymous Network Extension Dependency Management, Package Resolution, Version Constraints, Lockfiles, Dependency Isolation, Supply-Chain Policy & Privacy-Preserving Dependency Governance Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 142  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 24, 66, 73, 98–99, 122–141

**Primary purpose:** define SIAR's extension dependency architecture for deterministic dependency resolution, version constraints, content-addressed package identities, lockfiles, transitive dependency policy, dependency isolation, shared-runtime limits, package provenance, SBOMs, vulnerability handling, revocation/yanking, offline resolution, reproducible builds, compatibility governance, and privacy-preserving supply-chain operations.

---

# 1. Purpose

An extension may depend on:

```text
SDK libraries
shared runtime modules
other extensions
protocol adapters
codecs
data transforms
UI component packs
```

Dependency systems create substantial risk:

```text
dependency confusion
version drift
supply-chain compromise
transitive privilege escalation
shared-runtime contamination
non-reproducible builds
silent yanks/replacements
dependency metadata surveillance
```

The governing principle is:

> **SIAR dependency management must be deterministic, content-addressed, policy-constrained, provenance-aware, lockfile-driven, and incapable of widening extension authority or cross-extension trust merely because two packages share a dependency.**

---

# 2. Architectural Position

```text
                   EXTENSION MANIFEST
                          │
                          ▼
                 DEPENDENCY REQUIREMENTS
                          │
                          ▼
                    RESOLVER
                          │
              ┌───────────┼───────────┐
              │           │           │
           REGISTRY    POLICY      COMPATIBILITY
              │           │           │
              └───────────┼───────────┘
                          ▼
                    LOCKFILE
                          │
                          ▼
                 VERIFIED PACKAGES
                          │
                          ▼
               ISOLATED DEPENDENCY GRAPH
                          │
                          ▼
                    RUNTIME / BUILD
```

---

# 3. Core Separation

Keep distinct:

```text
package identity
package version
package digest
dependency requirement
resolved dependency
lockfile entry
runtime dependency
build dependency
capability dependency
extension-to-extension dependency
```

---

# 4. Non-Goals

Part 142 does not create:

```text
floating production dependencies
unsigned package substitution
ambient privilege sharing through libraries
global mutable package state
dependency-based user tracking
```

---

# 5. Package Identity

```rust
pub struct DependencyPackageId(pub [u8; 16]);
```

Logical identity.

---

# 6. Package Version

```rust
pub struct DependencyVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}
```

---

# 7. Package Digest

```rust
pub struct DependencyPackageDigest(pub Digest);
```

Content identity.

---

# 8. Hard Rule

Logical ID, semantic version, and content digest are distinct.

---

# 9. Registry Identity

```rust
pub struct DependencyRegistryId(pub [u8; 16]);
```

---

# 10. Dependency Requirement

```rust
pub struct DependencyRequirement {
    pub package: DependencyPackageId,
    pub version: VersionConstraint,
    pub source: DependencySource,
    pub kind: DependencyKind,
}
```

---

# 11. Dependency Kind

```rust
pub enum DependencyKind {
    Build,
    Runtime,
    Optional,
    Development,
    ExtensionPeer,
}
```

---

# 12. Hard Rule

Development/build dependencies never imply runtime authority.

---

# 13. Dependency Source

```rust
pub enum DependencySource {
    OfficialRegistry(DependencyRegistryId),
    EnterpriseRegistry(DependencyRegistryId),
    ContentAddressedBundle(DependencyPackageDigest),
    LocalPathDev,
}
```

---

# 14. Hard Rule

Production marketplace package cannot depend on arbitrary mutable local path.

---

# 15. Version Constraints

```rust
pub enum VersionConstraint {
    Exact(DependencyVersion),
    CompatibleMajor(DependencyVersion),
    Range {
        min: DependencyVersion,
        max_exclusive: DependencyVersion,
    },
}
```

---

# 16. Hard Rule

Resolver semantics are deterministic and documented.

---

# 17. No Implicit "Latest"

Hard rule.

---

# 18. Semantic Versioning

Semver describes package API expectation.

Runtime compatibility still governed by explicit compatibility metadata.

---

# 19. Hard Rule

Semver alone does not prove security/interoperability.

---

# 20. Dependency Manifest

```rust
pub struct ExtensionDependencyManifest {
    pub extension: ExtensionId,
    pub extension_version: ExtensionVersion,
    pub dependencies: Vec<DependencyRequirement>,
}
```

---

# 21. Hard Rule

All direct dependencies declared.

---

# 22. Hidden Dynamic Download

Forbidden by default.

---

# 23. Hard Rule

No undeclared runtime package fetch.

---

# 24. Transitive Dependency

Resolver computes full graph.

---

# 25. Dependency Graph

```rust
pub struct ResolvedDependencyGraph {
    pub root: ExtensionId,
    pub nodes: BTreeMap<DependencyPackageDigest, ResolvedDependencyNode>,
    pub edges: Vec<ResolvedDependencyEdge>,
}
```

---

# 26. Node

```rust
pub struct ResolvedDependencyNode {
    pub package: DependencyPackageId,
    pub version: DependencyVersion,
    pub digest: DependencyPackageDigest,
    pub provenance: PackageProvenanceRef,
}
```

---

# 27. Edge

```rust
pub struct ResolvedDependencyEdge {
    pub from: DependencyNodeRef,
    pub to: DependencyNodeRef,
    pub kind: DependencyKind,
}
```

---

# 28. Hard Rule

Dependency graph acyclic unless explicitly modeled shared cycle-break mechanism exists.

---

# 29. Resolver

```rust
pub trait ExtensionDependencyResolver {
    fn resolve(
        &self,
        manifest: &ExtensionDependencyManifest,
        policy: &DependencyPolicySnapshot,
    ) -> Result<ResolvedDependencyGraph, DependencyResolutionError>;
}
```

---

# 30. Determinism

Same:

```text
manifest
registry snapshot
policy snapshot
compatibility snapshot
```

must produce same resolution.

---

# 31. Hard Rule

No resolver dependence on wall-clock "latest available" after lock creation.

---

# 32. Registry Snapshot

```rust
pub struct DependencyRegistrySnapshot {
    pub registry: DependencyRegistryId,
    pub snapshot_version: u64,
    pub digest: Digest,
}
```

---

# 33. Hard Rule

Resolution uses signed/versioned registry metadata.

---

# 34. Lockfile

Canonical resolved state.

```rust
pub struct ExtensionDependencyLockfile {
    pub format_version: u32,
    pub root_extension: ExtensionId,
    pub root_version: ExtensionVersion,
    pub registry_snapshots: Vec<DependencyRegistrySnapshot>,
    pub packages: Vec<LockedDependency>,
}
```

---

# 35. Locked Dependency

```rust
pub struct LockedDependency {
    pub package: DependencyPackageId,
    pub version: DependencyVersion,
    pub digest: DependencyPackageDigest,
    pub source: DependencySource,
    pub provenance_digest: Digest,
}
```

---

# 36. Hard Rule

Production build/install resolves from lockfile, not fresh floating ranges.

---

# 37. Lockfile Human Format

RON preferred.

---

# 38. Hard Rule

Lockfile is machine-validated and canonicalized.

---

# 39. Lockfile Signature

Marketplace/certification build may sign lockfile digest.

---

# 40. Hard Rule

Lockfile tampering invalidates qualification evidence.

---

# 41. Lockfile Update

Explicit operation.

---

# 42. Hard Rule

Dependency update cannot happen silently during ordinary build/install.

---

# 43. Dependency Diff

Update operation displays:

```text
added package
removed package
version change
digest change
new publisher/source
new capability/runtime requirement
new vulnerability status
```

---

# 44. Hard Rule

Digest change under same version is suspicious and requires review.

---

# 45. Immutable Package Rule

Published version content is immutable.

---

# 46. Hard Rule

Same package ID/version cannot legally map to different digest.

---

# 47. Registry Replacement

If registry metadata shows changed digest for immutable release:

```text
quarantine
security incident
resolver rejection
```

---

# 48. Hard Rule

No silent replacement.

---

# 49. Package Provenance

Each dependency package binds:

```text
source revision
builder identity
artifact digest
SBOM
signature
```

---

# 50. Hard Rule

Unsigned package rejected unless explicit local development mode.

---

# 51. Local Development Mode

May permit unsigned local dependency.

---

# 52. Hard Rule

Local dev dependency cannot be promoted to production without signed/provenance-resolved replacement.

---

# 53. Publisher Identity

Dependency publisher distinct from extension publisher.

---

# 54. Hard Rule

Trust in root extension publisher does not automatically trust transitive publishers.

---

# 55. Dependency Policy

```rust
pub struct DependencyPolicy {
    pub allowed_registries: BTreeSet<DependencyRegistryId>,
    pub forbidden_packages: BTreeSet<DependencyPackageId>,
    pub forbidden_licenses: BTreeSet<LicenseId>,
    pub max_depth: u16,
    pub max_packages: u32,
}
```

---

# 56. Hard Rule

Graph size/depth bounded.

---

# 57. License Policy

Review:

```text
license compatibility
redistribution rights
copyleft implications
enterprise restrictions
```

---

# 58. Hard Rule

License review separate from security review.

---

# 59. Supply-Chain Policy

Includes:

```text
signature required
SBOM required
provenance required
known-vulnerability threshold
reproducibility requirement
```

---

# 60. Hard Rule

Policy applied to transitive dependencies too.

---

# 61. Dependency Vulnerability Record

```rust
pub struct DependencyVulnerability {
    pub package: DependencyPackageId,
    pub affected: VersionConstraint,
    pub advisory: AdvisoryId,
    pub severity: VulnerabilitySeverity,
    pub status: VulnerabilityStatus,
}
```

---

# 62. Hard Rule

Severity != automatic exploitability judgment.

---

# 63. Exposure Analysis

Need:

```text
is vulnerable code reachable?
is affected feature enabled?
is package runtime/build only?
```

---

# 64. Hard Rule

Do not blindly revoke based on version presence alone when reachability/effect differs, but do not dismiss advisory without evidence.

---

# 65. Vulnerability Decision

```rust
pub enum DependencyVulnerabilityDecision {
    Block,
    AllowTemporarilyWithException,
    NotAffectedWithEvidence,
    UpgradeRequired,
}
```

---

# 66. Hard Rule

Critical hard-invariant issues cannot be waived if policy forbids it.

---

# 67. Package Yank

Yank means:

```text
do not select for new resolution
```

not necessarily:

```text
break existing locked builds
```

---

# 68. Hard Rule

Yank and revocation are distinct.

---

# 69. Package Revocation

Revocation means package is unsafe/untrusted.

---

# 70. Hard Rule

Revoked package cannot be newly installed/executed according to revocation severity.

---

# 71. Revocation State

```rust
pub enum DependencyRevocationState {
    None,
    Advisory,
    BlockNewResolution,
    BlockExecution,
}
```

---

# 72. Hard Rule

Revocation list signed and anti-rollback protected.

---

# 73. Existing Extension Impact

If dependency revoked:

```text
identify affected extensions
suspend execution if required
offer safe update
preserve extension data
```

---

# 74. Hard Rule

Do not delete user extension data automatically.

---

# 75. Transitive Impact Graph

Registry/knowledge graph can answer:

```text
which extensions depend on package X?
which locked versions are affected?
```

---

# 76. Hard Rule

Graph contains technical package lineage, not user install behavior by default.

---

# 77. Dependency Isolation

Default:

```text
each extension resolves/runs dependencies inside its own package/runtime boundary
```

---

# 78. Hard Rule

Shared package code does not imply shared memory/state.

---

# 79. Duplicate Versions

Allowed where isolation requires it.

---

# 80. Hard Rule

Do not force global singleton version if it breaks isolation.

---

# 81. Shared Runtime Module

Optional for trusted common components.

---

# 82. Hard Rule

Shared runtime modules are platform-managed and separately certified.

---

# 83. No Arbitrary Third-Party Shared Library Injection

Hard rule.

---

# 84. Shared Module Identity

```rust
pub struct SharedRuntimeModuleId(pub [u8; 16]);
```

---

# 85. Shared Module Contract

```rust
pub struct SharedRuntimeModuleContract {
    pub module: SharedRuntimeModuleId,
    pub version: DependencyVersion,
    pub host_abi: HostAbiVersion,
    pub capabilities: BTreeSet<ModuleCapability>,
}
```

---

# 86. Hard Rule

Shared module cannot access caller extension authority implicitly.

---

# 87. Caller Authority

Explicit attenuated delegation required for shared module operation.

---

# 88. Hard Rule

Dependency cannot inherit root extension capabilities automatically.

---

# 89. Capability Dependency

If dependency requires broker capability:

```rust
pub struct DependencyCapabilityRequirement {
    pub package: DependencyPackageId,
    pub capability: SdkCapability,
}
```

---

# 90. Hard Rule

Root extension manifest must declare externally relevant capability requirements induced by dependency.

---

# 91. No Hidden Transitive Permission

Hard rule.

---

# 92. Capability Closure

Resolver computes effective required capability closure.

---

# 93. Hard Rule

Permission review shows direct + transitive capability effects.

---

# 94. Dependency Data Access

A library dependency within extension process can access extension memory.

---

# 95. Architectural Consequence

Third-party code linked into extension artifact shares the extension's sandbox.

---

# 96. Hard Rule

Publisher is responsible for auditing bundled dependencies; marketplace/certification evaluates supply-chain graph.

---

# 97. WASM Component Model

Preferred future isolation for separately distributed components.

---

# 98. Hard Rule

Component import/export capabilities explicitly typed.

---

# 99. Separate Dependency Process

For high-risk dependency, run as separate sandboxed service.

---

# 100. Hard Rule

Do not use in-process linking merely for convenience when isolation is required.

---

# 101. Native Dynamic Libraries

Strongly discouraged for third-party extension dependencies.

---

# 102. Hard Rule

No arbitrary `.so`/DLL loading from mutable filesystem.

---

# 103. If Native Library Required

Must be:

```text
content-addressed
signed
package-contained
platform-specific
reviewed
```

---

# 104. Hard Rule

Load only exact locked digest.

---

# 105. Static Linking

Preferred for Rust extension packages where feasible.

---

# 106. Hard Rule

Static linking does not remove provenance/SBOM obligations.

---

# 107. Cargo Dependencies

Extension source build may use Cargo crates.

---

# 108. Hard Rule

Cargo.lock pinned for production build.

---

# 109. Registry Mirroring

Official build infrastructure may mirror crates/dependencies.

---

# 110. Hard Rule

Mirror content digest must match upstream locked artifact.

---

# 111. Vendoring

Useful for:

```text
air-gapped builds
reproducibility
dependency archival
```

---

# 112. Hard Rule

Vendored package retains original provenance metadata.

---

# 113. Offline Resolution

Supported via signed dependency bundle.

---

# 114. Offline Bundle

```rust
pub struct ExtensionDependencyBundle {
    pub registry_snapshot: DependencyRegistrySnapshot,
    pub lockfile_digest: Digest,
    pub package_digests: Vec<DependencyPackageDigest>,
}
```

---

# 115. Hard Rule

Offline resolution uses pre-fetched signed immutable packages.

---

# 116. No Network Fallback

In air-gapped mode, missing package fails explicitly.

---

# 117. Hard Rule

Do not silently fetch from public registry.

---

# 118. Reproducible Build

Inputs include:

```text
source revision
toolchain
lockfile
dependency artifacts
build configuration
```

---

# 119. Hard Rule

Build environment cannot resolve unpinned dependency at build time.

---

# 120. Build Script Risk

Rust/Cargo build scripts can execute code.

---

# 121. Hard Rule

Extension marketplace build pipeline runs build scripts in constrained sandbox.

---

# 122. Build Script Network

Denied by default.

---

# 123. Hard Rule

Build script filesystem access scoped to build workspace.

---

# 124. Procedural Macros

Compiler-time code execution risk.

---

# 125. Hard Rule

Proc macros are dependencies and included in supply-chain review/SBOM.

---

# 126. Toolchain Plugins

Forbidden unless approved.

---

# 127. Hard Rule

No arbitrary compiler plugin loading.

---

# 128. Dependency Features

Cargo-style features can alter behavior.

---

# 129. Locked Feature Set

```rust
pub struct LockedDependencyFeatureSet {
    pub package: DependencyPackageId,
    pub features: BTreeSet<String>,
}
```

---

# 130. Hard Rule

Feature set part of lock/build provenance.

---

# 131. Optional Dependency Activation

Explicit.

---

# 132. Hard Rule

Runtime cannot unexpectedly activate undeclared optional dependency.

---

# 133. Platform-Specific Dependencies

Declared by target.

---

# 134. Hard Rule

Compatibility matrix includes platform-specific graph.

---

# 135. Target Triple

```rust
pub struct ExtensionBuildTarget {
    pub platform: PlatformClass,
    pub architecture: CpuArchitecture,
}
```

---

# 136. Hard Rule

Lockfile may include target-qualified entries while preserving deterministic resolution.

---

# 137. Multi-Platform Package

May have different dependency closure per target.

---

# 138. Hard Rule

Marketplace displays platform-specific support truthfully.

---

# 139. Dependency Update Strategy

Types:

```rust
pub enum DependencyUpdateMode {
    SecurityOnly,
    Patch,
    CompatibleMinor,
    ExplicitMajor,
}
```

---

# 140. Hard Rule

Production update policy explicit.

---

# 141. Security Update

Can prioritize update, but still requires:

```text
resolved lockfile
compatibility tests
package verification
```

---

# 142. Hard Rule

No untested dependency auto-bump directly to production runtime.

---

# 143. Automated Update PR

Developer tooling can generate dependency update proposal.

---

# 144. Hard Rule

Automation can propose, test, and produce evidence; it cannot self-certify high-risk changes.

---

# 145. Dependency Diff Review

Must classify:

```text
API change
transitive graph change
capability change
license change
advisory resolution
```

---

# 146. Hard Rule

Dependency update is not only a version-number diff.

---

# 147. Compatibility Testing

Run:

```text
unit
integration
contract
extension-runtime
platform
```

as required.

---

# 148. Hard Rule

Dependency update that changes runtime ABI triggers interoperability/compatibility checks.

---

# 149. Extension-to-Extension Dependency

Highest complexity.

---

# 150. Preferred Alternative

Depend on stable host SDK/interface rather than another extension.

---

# 151. Hard Rule

Direct extension dependency used only when explicit product need exists.

---

# 152. Peer Dependency Declaration

```rust
pub struct ExtensionPeerDependency {
    pub extension: ExtensionId,
    pub version: VersionRange<ExtensionVersion>,
    pub required_capabilities: BTreeSet<PeerExtensionCapability>,
}
```

---

# 153. Hard Rule

Peer dependency does not imply shared permissions/data.

---

# 154. Peer Communication

Uses brokered inter-extension contract from Part 137.

---

# 155. Hard Rule

No direct database/process access to peer extension.

---

# 156. Missing Peer Dependency

Extension enters degraded/blocked state.

---

# 157. Hard Rule

Host explains missing dependency.

---

# 158. Dependency Installation Consent

Installing extension may require installing peer extension.

---

# 159. Hard Rule

User/admin sees transitive extension installation impact.

---

# 160. No Forced Marketplace Install Without Approval

Hard rule.

---

# 161. Dependency Cycle

Peer extension cycle forbidden unless explicitly supported by architecture.

---

# 162. Hard Rule

Resolver rejects cycles by default.

---

# 163. Lockfile Portability

Lockfile can be archived/exported.

---

# 164. Hard Rule

No secret credentials inside lockfile.

---

# 165. Registry Credentials

Stored separately via secret broker.

---

# 166. Hard Rule

Private-registry token never committed.

---

# 167. Enterprise Registry

Tenant may allow private dependency registry.

---

# 168. Hard Rule

Tenant registry cannot override immutable package identity from another registry.

---

# 169. Namespace Isolation

Registry namespace + package ID prevent confusion.

---

# 170. Hard Rule

Same display name across registries is not same package identity.

---

# 171. Dependency Confusion Defense

Resolution never searches arbitrary registry fallback by name.

---

# 172. Hard Rule

Source registry is bound in requirement/lockfile.

---

# 173. Registry Trust Root

Signed registry metadata chain.

---

# 174. Hard Rule

No unsigned registry index for production.

---

# 175. Registry Compromise

Response:

```text
freeze resolution
revoke trust root if needed
use known-good snapshot
investigate package lineage
```

---

# 176. Hard Rule

Registry compromise does not automatically trust newly presented package metadata.

---

# 177. Metadata Expiry

Registry snapshot freshness explicit.

---

# 178. Hard Rule

Expired metadata cannot permissively resolve new production dependencies.

---

# 179. Existing Locked Build

May continue only under local policy if artifact/provenance remain trusted.

---

# 180. Hard Rule

Freshness and artifact trust are separate.

---

# 181. Dependency Transparency Log

Optional high-assurance feature.

---

# 182. Hard Rule

Transparency log records package/provenance events, not user installs.

---

# 183. Privacy Boundary

Dependency infrastructure should not learn:

```text
which individual user installed which extension
which private tenant uses which package
user-specific activity
```

unless operationally unavoidable and minimized.

---

# 184. Hard Rule

Resolution telemetry is aggregate and technical.

---

# 185. Private Tenant Build Privacy

Enterprise/private extensions may resolve via local mirror to avoid exposing dependency graph externally.

---

# 186. Hard Rule

No mandatory public registry beacon from private extension build.

---

# 187. Registry Query Minimization

Use:

```text
batch metadata
cache
mirror
snapshot
```

rather than per-user repeated requests.

---

# 188. Hard Rule

Do not send account/user identifiers to public registries.

---

# 189. Dependency Metrics

Safe aggregate:

```text
resolution failures
signature failures
revoked package count
outdated package count
```

---

# 190. Forbidden:

```text
per-user package graph
developer productivity
private repository mapping
```

---

# 191. Hard Rule

No dependency telemetry as workforce analytics.

---

# 192. SBOM

Every extension package includes complete transitive SBOM.

---

# 193. Hard Rule

Build/dev-only dependencies may be distinguished but remain represented for build provenance.

---

# 194. SBOM Identity

```rust
pub struct ExtensionSbomId(pub Digest);
```

---

# 195. Hard Rule

SBOM digest bound into package provenance/certification.

---

# 196. License Inventory

Derived from SBOM.

---

# 197. Hard Rule

Unknown license state blocks according to policy.

---

# 198. Advisory Feed

Signed vulnerability advisories.

---

# 199. Hard Rule

No unauthenticated advisory source drives automatic revocation.

---

# 200. Advisory Freshness

Explicit.

---

# 201. Hard Rule

Stale advisory state becomes Unknown, not Safe.

---

# 202. Dependency Risk Review

Could record:

```text
maintainer concentration
release freshness
unsafe code usage
native build scripts
cryptography role
```

---

# 203. Hard Rule

Do not create arbitrary one-number "dependency trust score".

---

# 204. Factual Signals

Preferred.

---

# 205. Unsafe Rust Signal

Useful factual metadata.

---

# 206. Hard Rule

Presence of unsafe is not itself a vulnerability verdict.

---

# 207. Cryptographic Dependency

Must use approved audited libraries/policy.

---

# 208. Hard Rule

No custom crypto dependency accepted casually.

---

# 209. Post-Quantum / Crypto Agility

Part 66 applies.

---

# 210. Hard Rule

Dependency lock does not block required security migration indefinitely.

---

# 211. Emergency Dependency Override

Security response may require forced update/block.

---

# 212. Hard Rule

Emergency action cannot introduce unsigned/unverified replacement.

---

# 213. Dependency Exception

```rust
pub struct DependencyPolicyException {
    pub package: DependencyPackageId,
    pub scope: DependencyExceptionScope,
    pub rationale: String,
    pub expires_at: Timestamp,
}
```

---

# 214. Hard Rule

Exceptions time-bounded.

---

# 215. Non-Waivable Conditions

Examples:

```text
revoked artifact
invalid signature
known malicious package
hard crypto policy violation
```

---

# 216. Hard Rule

No exception overrides these.

---

# 217. Resolution Policy Precedence

```text
hard platform invariant
> revocation/security policy
> tenant policy
> extension manifest
> lockfile update preference
```

---

# 218. Hard Rule

Lower layer cannot broaden higher restrictions.

---

# 219. Dependency Cache

Content-addressed immutable cache.

---

# 220. Hard Rule

Cache key is digest, not only package name/version.

---

# 221. Cache Verification

Recheck digest/signature before use.

---

# 222. Hard Rule

Corrupt cache entry quarantined.

---

# 223. Cache Sharing

Immutable public package blobs may share physical storage.

---

# 224. Hard Rule

Shared blob cache does not create shared mutable runtime state.

---

# 225. Sensitive Private Package

Private package metadata/blob follows tenant isolation.

---

# 226. Hard Rule

No cross-tenant private package dedup if existence itself is sensitive.

---

# 227. Build Isolation

Each extension build executes in isolated sandbox.

---

# 228. Hard Rule

Build dependency cannot access another extension source/workspace.

---

# 229. Network During Build

Denied by default after dependencies fetched.

---

# 230. Hard Rule

Hermetic build target.

---

# 231. Reproducible Inputs

Pin:

```text
Rust toolchain
target
dependency lockfile
feature set
environment
build flags
```

---

# 232. Hard Rule

Environment variables influencing build must be declared.

---

# 233. Non-Determinism

Timestamp/randomness/network availability should not affect artifact except documented provenance fields.

---

# 234. Hard Rule

Build timestamp excluded from semantic artifact where possible.

---

# 235. Dependency Resolver Service

```rust
pub trait DependencyResolverService {
    fn resolve(
        &self,
        manifest: ExtensionDependencyManifest,
        target: ExtensionBuildTarget,
    ) -> Result<ExtensionDependencyLockfile, DependencyGovernanceError>;
}
```

---

# 236. Lockfile Verification Service

```rust
pub trait DependencyLockfileVerifier {
    fn verify(
        &self,
        lockfile: &ExtensionDependencyLockfile,
    ) -> Result<VerifiedDependencyGraph, DependencyGovernanceError>;
}
```

---

# 237. Vulnerability Service

```rust
pub trait DependencyVulnerabilityService {
    fn evaluate(
        &self,
        graph: &VerifiedDependencyGraph,
    ) -> Result<DependencyVulnerabilityReport, DependencyGovernanceError>;
}
```

---

# 238. Policy Service

```rust
pub trait DependencyPolicyService {
    fn effective_policy(
        &self,
        extension: ExtensionId,
        tenant: Option<TenantId>,
    ) -> Result<DependencyPolicySnapshot, DependencyGovernanceError>;
}
```

---

# 239. Error Taxonomy

```rust
pub enum DependencyGovernanceError {
    PackageUnknown,
    VersionUnsatisfied,
    RegistryUntrusted,
    RegistryMetadataExpired,
    SignatureInvalid,
    DigestMismatch,
    DependencyCycle,
    PolicyDenied,
    RevokedPackage,
    VulnerabilityBlocked,
    LicenseDenied,
    LockfileInvalid,
    CompatibilityMismatch,
    PrivateRegistryUnauthorized,
    QuotaExceeded,
    Internal,
}
```

---

# 240. Dependency Graph Limits

Bound:

```text
max packages
max graph depth
max metadata bytes
max lockfile size
```

---

# 241. Hard Rule

Resolver rejects pathological graph size.

---

# 242. Algorithmic Complexity

Avoid exponential backtracking where possible.

---

# 243. Hard Rule

Resolution resource budget explicit.

---

# 244. Resolver Timeout

Bounded.

---

# 245. Hard Rule

Timeout does not produce partial lockfile.

---

# 246. Resolution Failure

Returns explanation:

```text
conflicting constraints
blocked policy
revoked package
untrusted registry
```

---

# 247. Hard Rule

Do not leak inaccessible private registry/package existence through error detail.

---

# 248. Lockfile Explain

Developer tooling can show why package/version selected.

---

# 249. Hard Rule

Explain is technical, not user-tracking metadata.

---

# 250. Update Planner

```rust
pub struct DependencyUpdatePlan {
    pub current: ExtensionDependencyLockfile,
    pub proposed: ExtensionDependencyLockfile,
    pub diff: DependencyGraphDiff,
}
```

---

# 251. Hard Rule

Update planner does not mutate active lockfile automatically.

---

# 252. Dependency Graph Diff

```rust
pub struct DependencyGraphDiff {
    pub added: Vec<LockedDependency>,
    pub removed: Vec<LockedDependency>,
    pub changed: Vec<DependencyVersionChange>,
}
```

---

# 253. Review Gate

High-risk changes trigger review:

```text
new native dependency
new build script
new registry
new crypto package
new external capability requirement
major version
```

---

# 254. Hard Rule

Risk-triggered review cannot be bypassed by version-number semantics.

---

# 255. Certification Integration

Part 125/126/133.

Certified extension binds exact lockfile/SBOM/provenance.

---

# 256. Hard Rule

Changing dependency lockfile invalidates relevant certification evidence.

---

# 257. Compatibility Integration

Part 129.

Dependency ABI/API changes can affect host compatibility.

---

# 258. Hard Rule

Compatibility baseline references locked dependency graph where relevant.

---

# 259. Lifecycle Integration

Part 128.

Deprecated dependency versions need migration plan.

---

# 260. Hard Rule

Extension EOL cannot leave unsafe unmaintained dependency path hidden.

---

# 261. Marketplace Integration

Part 133.

Marketplace listing may show factual supply-chain signals.

---

# 262. Hard Rule

No opaque package trust score.

---

# 263. Runtime Integration

Part 134.

Runtime loads only package-contained/host-approved locked dependencies.

---

# 264. Hard Rule

No late dynamic dependency resolution inside extension runtime.

---

# 265. Permission Integration

Part 135.

Transitive capability requirements surfaced to consent/policy review.

---

# 266. Hard Rule

Dependency cannot widen permissions invisibly.

---

# 267. Data Governance Integration

Part 136.

Dependency code remains within extension data-flow restrictions.

---

# 268. Hard Rule

Library dependency cannot create undeclared sink outside brokers.

---

# 269. IPC Integration

Part 137.

Peer/shared-module dependency communication uses typed brokered IPC.

---

# 270. State Sync Integration

Part 141.

Lockfile/package state may sync as installation intent metadata, but actual package blobs/device install remain locally verified.

---

# 271. Hard Rule

Remote device never trusts another device's "verified dependency" assertion without local verification.

---

# 272. Backup / Restore

Archive:

```text
lockfile
SBOM
package digests
provenance refs
```

as needed.

---

# 273. Hard Rule

Restore re-verifies package/revocation state before execution.

---

# 274. Account Recovery

No dependency trust state bypass.

---

# 275. Hard Rule

Recovered device fetches/verifies packages fresh or from trusted content-addressed archive.

---

# 276. Audit

Audit high-value supply-chain events:

```text
registry trust change
revoked dependency admitted/blocked
lockfile policy exception
private registry added
```

---

# 277. Hard Rule

No audit of every local dependency read/download.

---

# 278. Observability

Safe metrics:

```text
resolution duration
resolution failure class
signature/digest failures
revocation matches
SBOM generation health
```

---

# 279. Forbidden:

```text
per-user dependency graph
private repo mapping
developer productivity ranking
```

---

# 280. Dependency Governance SLOs

Examples:

```text
revocation propagation within target
resolver determinism
lockfile verification within target
advisory freshness within target
```

---

# 281. Security SLO

```text
0 unsigned production dependency accepted
0 digest mismatch ignored
0 revoked dependency newly resolved
0 hidden transitive capability increase
```

---

# 282. Privacy SLO

```text
0 public registry receives user/account identity
0 private extension graph used for analytics
0 dependency telemetry becomes workforce surveillance
```

---

# 283. Failure Modes

```text
dependency confusion
registry compromise
digest substitution
conflicting constraints
revoked transitive package
```

---

# 284. Dependency Confusion

Source-bound package identities prevent fallback.

---

# 285. Registry Compromise

Freeze new resolution, use known-good signed snapshot.

---

# 286. Digest Substitution

Reject/quarantine.

---

# 287. Conflicting Constraints

Fail resolution with explainable conflict.

---

# 288. Revoked Transitive Package

Identify affected roots and apply revocation policy.

---

# 289. Testing

Need extension dependency-governance testkit.

Required scenarios:

```text
deterministic resolve
offline locked build
dependency confusion attempt
revoked transitive package
peer-extension dependency
```

---

# 290. Determinism Test

Same manifest/snapshot/policy yields identical lockfile digest.

---

# 291. Digest Test

Same version/different artifact digest rejected.

---

# 292. Source Binding Test

Package name from unapproved registry cannot satisfy official-registry requirement.

---

# 293. Lockfile Test

Build does not re-resolve floating versions.

---

# 294. Offline Test

Air-gapped build uses signed bundle only.

---

# 295. Capability Closure Test

Transitive dependency capability surfaces in root permission review.

---

# 296. Revocation Test

Revoked dependency blocked from new execution according to policy.

---

# 297. Shared Module Test

Shared module cannot inherit caller capabilities implicitly.

---

# 298. Private Registry Test

Unauthorized tenant cannot enumerate private package.

---

# 299. Restore Test

Restored package graph rechecks current revocation/signature state.

---

# 300. Fuzzing

Fuzz:

```text
version constraints
lockfiles
registry metadata
dependency graphs
SBOM manifests
```

---

# 301. Property Tests

Properties:

```text
a lockfile entry can never resolve to a different content digest
a dependency can never add undeclared runtime capability without root manifest/policy visibility
a revoked package can never enter a new verified production graph
a package requirement can never fall back to an unbound registry by display name
```

---

# 302. Formal Verification Targets

Strong candidates:

```text
resolution determinism
source binding
revocation dominance
capability closure
```

---

# 303. Kani Candidate

lockfile/digest/source/capability invariants.

---

# 304. TLA+ Candidate

```text
publish → resolve → lock → certify → revoke → update
```

---

# 305. Loom Candidate

Concurrent:

```text
registry refresh
dependency resolution
revocation arrival
lockfile verification
```

---

# 306. Performance

Resolver is not runtime hot path.

---

# 307. Targets

```text
bounded graph
indexed registry metadata
content-addressed cache
incremental update diff
```

---

# 308. Hard Rule

No dependency resolution during message send/UI interaction.

---

# 309. Cache

Immutable package blobs keyed by digest.

---

# 310. Hard Rule

Revalidation of signature/digest before privileged use.

---

# 311. Storage

Separate:

```text
registry metadata
package blobs
lockfiles
SBOMs
provenance
revocation/advisory state
policy exceptions
```

---

# 312. Secrets separate.

---

# 313. Hard Rule

No user-install tracking warehouse.

---

# 314. Partitioning

By:

```text
registry
package
digest
extension root
tenant for private registries
```

---

# 315. Hard Rule

Private package metadata respects tenant isolation.

---

# 316. Crate Layout

Recommended:

```text
crates/
├── siar-extension-deps-core/
├── siar-extension-deps-resolver/
├── siar-extension-deps-lockfile/
├── siar-extension-deps-registry/
├── siar-extension-deps-policy/
├── siar-extension-deps-provenance/
├── siar-extension-deps-vulnerability/
├── siar-extension-deps-isolation/
├── siar-extension-deps-observability/
└── siar-extension-deps-testkit/
```

---

# 317. `siar-extension-deps-core`

Owns:

```text
DependencyPackageId
DependencyVersion
DependencyRequirement
DependencyGovernanceError
```

---

# 318. `siar-extension-deps-resolver`

Deterministic graph resolution and constraint solving.

---

# 319. `siar-extension-deps-lockfile`

Canonical RON lockfile, digest/signature verification, diff.

---

# 320. `siar-extension-deps-registry`

Signed registry snapshots, source binding, private registry policy.

---

# 321. `siar-extension-deps-policy`

License/security/registry/graph-size/update policies.

---

# 322. `siar-extension-deps-provenance`

SBOM/build/source provenance and reproducibility evidence.

---

# 323. `siar-extension-deps-vulnerability`

Advisory matching/exposure/revocation integration.

---

# 324. `siar-extension-deps-isolation`

Shared modules, peer dependencies, runtime load boundaries.

---

# 325. `siar-extension-deps-observability`

Aggregate resolver/supply-chain health only.

---

# 326. `siar-extension-deps-testkit`

resolution/confusion/revocation/offline/privacy tests.

---

# 327. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Package logical identity, semantic version, content digest, source registry, dependency requirement, lockfile entry, provenance record, SBOM entry, and runtime-loaded artifact are distinct typed concepts and cannot be inferred from one another implicitly.
2. Production extension builds and installations use explicit lockfiles plus immutable content digests; floating ranges may guide lockfile creation but can never cause silent dependency changes during ordinary build, install, startup, or runtime.
3. Package sources are registry-bound and signed; dependency resolution can never fall back by display name to an arbitrary registry, preventing dependency-confusion substitution.
4. Published package versions are immutable: a package ID/version mapping to a different digest is treated as corruption or compromise, not a legitimate update.
5. Transitive dependencies are subject to the same signature, provenance, SBOM, vulnerability, license, registry, compatibility, and revocation policies as direct dependencies.
6. Dependency inclusion never grants additional SIAR capability, data access, file/network/device authority, or peer-extension privilege implicitly; transitive capability requirements are surfaced to root-extension policy/consent review.
7. Extension dependencies are isolated within the extension sandbox by default; shared runtime modules are platform-managed and separately certified, while arbitrary native dynamic libraries or shared mutable state are prohibited.
8. Build-time code execution—build scripts, procedural macros, generators—is sandboxed, network-restricted, provenance-tracked, and cannot access unrelated extension workspaces or production secrets.
9. Yanking, vulnerability advisories, and artifact revocation are distinct states; hard revocation dominates new resolution/execution while existing locked builds are handled according to explicit policy and evidence rather than silently rewritten.
10. Offline/air-gapped resolution uses signed registry snapshots, immutable package bundles, and pinned lockfiles; missing artifacts fail explicitly and never trigger hidden network fallback.
11. Dependency resolution, registry access, vulnerability scanning, package caching, and observability are technical and privacy-minimized; they cannot become per-user install tracking, private extension graph analytics, public-registry account beacons, or developer workforce surveillance.
12. Dependency governance integrates with marketplace distribution, runtime isolation, permissions, data flow, IPC, state sync, compatibility, lifecycle, certification, supply-chain security, vulnerability management, updates, backup/restore, and audit without creating a side channel around SIAR's security, privacy, anonymity, local-first, or tenant-isolation guarantees.
```

---

# 328. Initial Production Scope

Implement first:

```text
typed dependency package/source/version identities
deterministic resolver
exact/source-bound resolution
RON lockfile
content digest pinning
signed registry snapshots
content-addressed package cache
transitive graph limits
SBOM generation
provenance verification
license policy
vulnerability/advisory matching
yank vs revoke distinction
revocation enforcement
Cargo.lock integration for Rust extension source
build-script/proc-macro sandbox policy
offline dependency bundle
dependency graph diff/update planner
transitive capability closure
peer-extension dependency restrictions
shared-module governance skeleton
privacy-safe dependency metrics
dependency-governance testkit
```

Then add:

```text
WASM component dependency isolation
transparency log integration
reproducible-build cross-verification
advanced reachability analysis
enterprise dependency mirrors
formal deterministic resolver proofs
portable long-term dependency archives
```

---

# 329. Definition of Done

Part 142 is complete when:

- package IDs, versions, digests, and sources are distinct;
- resolution is deterministic;
- production builds use lockfiles;
- immutable version→digest mapping is enforced;
- registry source binding prevents dependency confusion;
- transitive dependencies receive full supply-chain policy;
- dependency capabilities cannot widen authority invisibly;
- build scripts/proc macros are treated as executable supply-chain inputs;
- yanks and revocations are distinct;
- offline builds use signed immutable bundles;
- compatibility/certification bind exact dependency graph;
- private dependency graphs are not exposed as analytics;
- resolution/confusion/revocation/offline/privacy/fuzz/formal tests are specified.

---

# 330. Final Architecture

```text
                 EXTENSION MANIFEST
                        │
                        ▼
                VERSION CONSTRAINTS
                        │
                        ▼
              SIGNED REGISTRY SNAPSHOT
                        │
                        ▼
              DETERMINISTIC RESOLVER
                        │
                        ▼
                    LOCKFILE
                        │
                        ▼
               VERIFIED PACKAGE GRAPH
                        │
             ┌──────────┼──────────┐
             │          │          │
          BUILD      RUNTIME    CERTIFICATION
             │          │          │
             └──────────┼──────────┘
                        ▼
              REVOCATION / UPDATE
```

Dependency-governance safety model:

```text
source-bound package identity
+
deterministic resolution
+
immutable digests
+
lockfiles
+
signed registry metadata
+
transitive provenance/SBOM
+
dependency isolation
+
revocation-aware execution
+
privacy-safe registry access
```

not:

```text
resolve whatever is newest, trust package names, share mutable libraries across plugins, let transitive dependencies inherit permissions, and phone home every user's dependency graph
```

---

# 331. Final Principle

Dependency management is trustworthy when the system can answer **exactly what code was selected, from which source, under which policy, with which provenance, and what authority that code may exercise**.

The correct model is:

```text
bind package identity to source
+
resolve deterministically
+
lock exact content digests
+
verify provenance and SBOM
+
isolate dependency code
+
surface transitive capability effects
+
treat build-time code as executable supply-chain input
+
revoke compromised artifacts safely
+
support offline reproducible builds
+
never use dependency infrastructure as an install-tracking or trust-shortcut system
```

This architecture gives SIAR a privacy-preserving extension dependency foundation for package resolution, version constraints, lockfiles, transitive dependency governance, runtime isolation, supply-chain policy, reproducible builds, vulnerability response, revocation, and offline operation while preserving the anonymity, local-first, least-authority, marketplace, runtime, permission, data-governance, state-sync, and anti-surveillance guarantees established across Parts 34–141.
