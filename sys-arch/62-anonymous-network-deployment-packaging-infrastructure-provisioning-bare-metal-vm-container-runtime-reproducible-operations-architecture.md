# Core System Architecture Part 62 — Anonymous Network Deployment, Packaging, Infrastructure Provisioning, Bare-Metal/VM/Container Runtime & Reproducible Operations Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 62  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–61  

**Primary purpose:** define how SIAR anonymous-network infrastructure is built, packaged, provisioned, deployed, verified, operated, reproduced, upgraded, restored, and scaled across bare-metal servers, virtual machines, containers, self-hosted deployments, regional clusters, and multi-operator environments without sacrificing supply-chain integrity, privacy guarantees, or operational determinism.

---

# 1. Purpose

Production anonymity infrastructure must be deployable in many environments:

```text
developer laptop
single bare-metal node
home lab
cloud VM
regional cluster
multi-region provider
enterprise deployment
air-gapped or restricted network
```

Deployment inconsistency can cause:

```text
different binaries
different libraries
untracked config
manual drift
unreproducible hotfixes
unknown supply-chain state
```

The governing principle is:

> **A production node should be reproducible from versioned source, signed artifacts, declarative configuration, and explicit secrets—never from undocumented manual state.**

---

# 2. Deployment Objectives

SIAR deployments should be:

```text
reproducible
auditable
immutable where practical
portable
fault-domain aware
supply-chain verified
privacy preserving
operator friendly
```

---

# 3. Deployment Model

```text
Source
  │
  ▼
Hermetic Build
  │
  ▼
Signed Artifact
  │
  ▼
Artifact Registry
  │
  ▼
Provisioning
  │
  ▼
Runtime Host
  │
  ▼
Signed Configuration
  │
  ▼
Service Startup
```

---

# 4. Reproducibility Boundary

Production state should be reconstructable from:

```text
source commit
Cargo.lock
build environment lock
release manifest
config version
secret references
infrastructure manifest
```

---

# 5. Deployment Classes

```rust
pub enum DeploymentClass {
    Developer,
    SingleNode,
    BareMetal,
    VirtualMachine,
    Container,
    RegionalCluster,
    MultiRegion,
    AirGapped,
}
```

---

# 6. Developer Deployment

Goals:

```text
fast iteration
local simulation
fault testing
```

Security posture lower.

---

# 7. SingleNode Deployment

Useful for:

```text
self-hosting
small community
testnet
```

---

# 8. BareMetal Deployment

Useful for:

```text
performance
operator independence
reduced cloud dependence
```

---

# 9. VirtualMachine Deployment

Useful for:

```text
cloud providers
isolation
fast provisioning
```

---

# 10. Container Deployment

Useful for:

```text
packaging consistency
orchestration
CI
regional services
```

---

# 11. RegionalCluster

Multiple nodes/services in one region.

---

# 12. MultiRegion

Part 50 continuity architecture.

---

# 13. AirGapped

Supports:

```text
offline artifact import
offline trust bundles
manual signed update transport
```

---

# 14. Artifact Philosophy

Build once.

Promote exact artifact.

---

# 15. No Rebuild Per Environment

Hard rule.

---

# 16. Release Artifact

```rust
pub struct ReleaseArtifact {
    pub software_version: SoftwareVersion,
    pub build_hash: BuildHash,
    pub sbom: SbomDigest,
    pub signature: ReleaseSignature,
}
```

---

# 17. Build Hash

Content-addressed.

---

# 18. Artifact Types

```text
static binary
deb/rpm
OCI image
VM image
recovery image
offline bundle
```

---

# 19. Rust Binary

Prefer:

```text
static or minimally dynamic
```

where practical.

---

# 20. musl

Possible for simple Linux targets.

---

# 21. glibc

May still be needed for:

```text
system integration
hardware drivers
media
```

---

# 22. Packaging Must Not Change Semantics

Hard rule.

---

# 23. Release Manifest

```rust
pub struct ReleaseManifest {
    pub version: SoftwareVersion,
    pub commit: GitCommitHash,
    pub artifacts: Vec<ArtifactDescriptor>,
    pub protocol_support: ProtocolVersionRegistry,
    pub config_schema: ConfigurationSchemaVersion,
    pub signatures: ReleaseSignatureBundle,
}
```

---

# 24. Artifact Descriptor

```rust
pub struct ArtifactDescriptor {
    pub target: DeploymentTarget,
    pub sha256: [u8; 32],
    pub size: u64,
    pub sbom_digest: [u8; 32],
}
```

---

# 25. Reproducible Builds

Goal:

```text
same source
same lockfiles
same toolchain
same dependencies
→ same artifact hash
```

---

# 26. Rust Toolchain Pinning

Use:

```text
rust-toolchain.toml
```

---

# 27. Cargo Lock

Committed.

---

# 28. Dependency Pinning

Version + checksum.

---

# 29. No Unpinned Git Dependency

Hard rule in production.

---

# 30. Git Dependency Exception

If necessary:

```text
pin exact commit
vendor or mirror
```

---

# 31. Hermetic Build Environment

Recommended.

---

# 32. Nix-Like Model

Use:

```text
declarative inputs
locked dependencies
pure derivation
```

---

# 33. Nix Flakes

Suitable option.

---

# 34. Example

```text
flake.nix
flake.lock
```

---

# 35. Functional Build Principle

Build output depends only on declared inputs.

---

# 36. No Mutable System Package Dependency

Hard rule.

---

# 37. Build Cache

Can be shared.

---

# 38. Cache Trust

Artifacts still verified.

---

# 39. Binary Cache Signature

Required.

---

# 40. Reproducible CI

CI uses same locked environment as local release build.

---

# 41. Cross Compilation

Supported where reliable.

---

# 42. Native Build

Preferred for platform-sensitive components.

---

# 43. Release Build Matrix

Potential:

```text
x86_64-linux
aarch64-linux
```

---

# 44. Architecture Independence

Wire protocol remains same.

---

# 45. CPU Feature Policy

Avoid compiling release artifact with accidental host-specific CPU flags.

---

# 46. `target-cpu=native`

Forbidden for generic release.

---

# 47. Optimized Variant

If needed:

```text
x86_64-v2
x86_64-v3
```

explicit.

---

# 48. Deterministic Build Metadata

Avoid:

```text
build timestamp
absolute source path
builder hostname
```

inside artifact where possible.

---

# 49. Source Date Epoch

Can help reproducibility.

---

# 50. SBOM

Required.

---

# 51. SBOM Includes

```text
crates
native libraries
licenses
versions
checksums
```

---

# 52. Dependency Policy

Integrate supply-chain rules.

---

# 53. Vulnerability Scan

CI gate.

---

# 54. License Scan

CI gate.

---

# 55. Cargo Audit

Recommended.

---

# 56. Cargo Deny

Recommended.

---

# 57. Sigstore / Artifact Signature

Possible external ecosystem integration.

---

# 58. Internal Release Signing

Required regardless.

---

# 59. Artifact Registry

Stores signed artifacts.

---

# 60. Registry Classes

```text
development
candidate
production
retired
```

---

# 61. Artifact Promotion

```text
candidate
→ verified
→ production
```

---

# 62. Exact Promotion

Hard rule.

---

# 63. No Production Build From Git Branch

Hard rule.

---

# 64. Registry Mirroring

Multi-region.

---

# 65. Offline Registry

For air-gapped deployments.

---

# 66. Immutable Image

VM/container image is versioned artifact.

---

# 67. No Mutable "latest" Tag

Hard rule.

---

# 68. OCI Image Reference

Use digest.

---

# 69. Example

```text
registry/siar-mix@sha256:...
```

---

# 70. Runtime Host Classes

```rust
pub enum RuntimeHostClass {
    BareMetal,
    VirtualMachine,
    ContainerHost,
}
```

---

# 71. Bare Metal

Host OS manually installed once or provisioned declaratively.

---

# 72. VM Host

Cloud/virtualization layer.

---

# 73. Container Host

Runs one or more SIAR services.

---

# 74. Host Operating System

Recommended:

```text
minimal Linux
```

---

# 75. Candidate

```text
NixOS
Debian stable
minimal Fedora/RHEL derivative
```

---

# 76. NixOS Advantage

Declarative host configuration.

---

# 77. Traditional Distribution

Still supported.

---

# 78. Host Hardening

Baseline:

```text
minimal packages
firewall
automatic security updates or managed patching
disabled unused services
restricted SSH
```

---

# 79. SSH

Emergency/admin channel only.

---

# 80. No Password SSH

Hard rule.

---

# 81. Prefer

```text
hardware-backed keys
certificate auth
```

---

# 82. Root Login

Disabled.

---

# 83. sudo

Scoped.

---

# 84. Admin Role Integration

Part 61.

---

# 85. Firewall

Default deny.

---

# 86. Service Ports

Explicit.

---

# 87. Admin Network

Separate where possible.

---

# 88. Public Service Network

Only necessary endpoints.

---

# 89. Management Plane Separation

Recommended.

---

# 90. Example

```text
public NIC
admin VPN NIC
storage/internal NIC
```

---

# 91. Cloud Security Groups

Equivalent controls.

---

# 92. Host Kernel

Keep patched.

---

# 93. Kernel Hardening

Potential:

```text
ASLR
seccomp
namespaces
cgroups
landlock
```

---

# 94. Rust Helps Memory Safety

But not OS isolation.

---

# 95. Service Account

Each daemon runs under dedicated OS user.

---

# 96. No Shared Service User

Hard rule.

---

# 97. Filesystem Permissions

Per service.

---

# 98. Read-Only Root

Where practical.

---

# 99. Writable Paths

Explicit.

---

# 100. systemd

Recommended bare-metal/VM supervisor.

---

# 101. systemd Service

Can enforce:

```text
ProtectSystem
ProtectHome
PrivateTmp
NoNewPrivileges
CapabilityBoundingSet
```

---

# 102. Example Unit Principle

```text
least privilege
restart on failure
resource limits
```

---

# 103. No Shell Wrapper Dependency

Prefer direct binary execution.

---

# 104. Health Check

Separate endpoint/socket.

---

# 105. Startup Sequence

```text
verify binary
verify config
load secret refs
initialize storage
start listener
report ready
```

---

# 106. Readiness

Different from process alive.

---

# 107. Liveness

Process operational.

---

# 108. Readiness

Safe to receive traffic.

---

# 109. Graceful Shutdown

Required.

---

# 110. Shutdown Sequence

```text
stop new work
drain
flush durable state
revoke/close sessions if needed
exit
```

---

# 111. Mix Node Shutdown

Drain delayed queue safely.

---

# 112. Mailbox Shutdown

Stop deposits before reads if migrating.

---

# 113. Relay Shutdown

No new calls; existing sessions drain.

---

# 114. Container Runtime

Potential:

```text
Podman
containerd
Docker-compatible runtime
```

---

# 115. Rootless Containers

Preferred where practical.

---

# 116. Container Principle

One service responsibility per container.

---

# 117. No Privileged Container

Hard rule except explicit hardware case.

---

# 118. Host Networking

Only where performance/transport requires.

---

# 119. Default

Network namespace isolation.

---

# 120. Container Secrets

Mounted via:

```text
tmpfs
secret file
agent
```

---

# 121. No Secret In Image Layer

Hard rule.

---

# 122. No Secret In Container Env

Preferred hard rule for high-value keys.

---

# 123. Container Image

Minimal.

---

# 124. Distroless/Scratch

Possible for pure Rust service.

---

# 125. Need CA Certificates

Include explicit trusted CA bundle if required.

---

# 126. Timezone Data

Not needed for protocol.

---

# 127. Debug Tools

Excluded from production image.

---

# 128. Debug Variant

Separate artifact.

---

# 129. No Debug Shell In Production Image

Preferred.

---

# 130. Container Orchestration

Possible:

```text
systemd + Podman
Kubernetes
Nomad
custom SIAR controller
```

---

# 131. Kubernetes

Useful at scale.

---

# 132. Kubernetes Risks

Complexity + metadata.

---

# 133. Recommendation

Do not require Kubernetes.

---

# 134. Baseline Deployment

Support:

```text
systemd bare-metal/VM
Podman container
```

---

# 135. Kubernetes Adapter

Optional.

---

# 136. K8s Namespace

Per environment/service class.

---

# 137. Kubernetes Secrets

Prefer external secret manager or encrypted provider.

---

# 138. No Plain Secret Manifest

Hard rule.

---

# 139. Pod Security

Restricted profile.

---

# 140. Service Account

Least privilege.

---

# 141. NetworkPolicy

Default deny.

---

# 142. Stateful Services

Need durable volume.

---

# 143. Mailbox Storage

Persistent encrypted volume.

---

# 144. Control Plane DB

PostgreSQL/managed equivalent.

---

# 145. Mix Nodes

Mostly stateless.

---

# 146. Relays

Stateless/ephemeral.

---

# 147. Provider Catalog

Mostly control-plane state.

---

# 148. Infrastructure Provisioning

Declarative.

---

# 149. Provisioning Tools

Possible:

```text
Nix
Terraform/OpenTofu
Ansible
cloud-init
```

---

# 150. Preferred Separation

```text
cloud resources → OpenTofu/Terraform
host state → NixOS/systemd
application config → SIAR signed config
```

---

# 151. No Tool Lock-In

Hard rule.

---

# 152. Infrastructure Manifest

```rust
pub struct InfrastructureManifest {
    pub deployment_id: DeploymentId,
    pub regions: Vec<RegionDeployment>,
    pub services: Vec<ServiceDeployment>,
    pub artifact_manifest: ReleaseManifestRef,
}
```

---

# 153. Deployment ID

Infrastructure-only.

---

# 154. No User Identity

Hard rule.

---

# 155. Cloud Provisioning

Creates:

```text
network
VMs
load balancers
storage
DB
secret backends
```

---

# 156. Provider Independence

Avoid provider-specific logic in core service.

---

# 157. Cloud Adapter

```rust
pub trait InfrastructureProviderAdapter {
    fn provision(
        &self,
        spec: InfrastructureSpec,
    ) -> Result<ProvisioningResult, InfrastructureError>;
}
```

---

# 158. Self-Hosted Adapter

Can generate:

```text
systemd units
Podman manifests
```

---

# 159. Bare-Metal Provisioning

Need:

```text
disk layout
OS image
network config
host identity
```

---

# 160. PXE

Optional.

---

# 161. Provisioning Image

Signed.

---

# 162. Bootstrap Trust

Embedded/pinned.

---

# 163. No Network Fetch Before Trust Bootstrap

Hard rule.

---

# 164. Disk Encryption

Recommended for sensitive nodes.

---

# 165. Full-Disk Encryption

Useful.

---

# 166. Boot Unlock

Can use:

```text
TPM
operator
network-bound disk unlock
```

---

# 167. Caution

Automatic unlock may reduce physical security.

---

# 168. TPM

Infrastructure use only.

---

# 169. Remote Attestation

Optional.

---

# 170. Avoid On User Devices

Hard rule for global anonymity.

---

# 171. VM Images

Prebuilt or cloud-init.

---

# 172. Golden Image

Must still fetch signed exact artifact/config.

---

# 173. Golden Image Drift

Avoid long-lived manually patched images.

---

# 174. Image Rebuild

Regular.

---

# 175. Image Version

Explicit.

---

# 176. Kernel/Userland Patch Strategy

Two approaches:

```text
in-place OS update
immutable image replacement
```

---

# 177. Preferred

Immutable replacement for large deployments.

---

# 178. Small Self-Hosted

Managed package update acceptable.

---

# 179. Runtime Data

Separate from system image.

---

# 180. Volume Layout

Suggested:

```text
/var/lib/siar/<service>
/etc/siar/<service>
/run/siar/<service>
```

---

# 181. Config Path

Read-only.

---

# 182. Runtime Path

tmpfs where practical.

---

# 183. State Path

Service-owned.

---

# 184. Secret Path

Runtime-only.

---

# 185. Filesystem Encryption

Provider-sensitive data.

---

# 186. No Shared State Directory

Hard rule.

---

# 187. Node Classes

```rust
pub enum NodeClass {
    Mix,
    Gateway,
    Mailbox,
    Relay,
    Bulk,
    Bridge,
    Directory,
    ControlPlane,
}
```

---

# 188. Node Profile

```rust
pub struct NodeProfile {
    pub class: NodeClass,
    pub cpu_class: CpuClass,
    pub memory_class: MemoryClass,
    pub storage_class: StorageClass,
    pub network_class: NetworkClass,
}
```

---

# 189. Mix Node Profile

Network + CPU.

---

# 190. Mailbox Profile

Storage + IO.

---

# 191. Relay Profile

Network + low latency.

---

# 192. Bulk Profile

Storage + egress.

---

# 193. Bridge Profile

Network diversity.

---

# 194. Directory Profile

Control-plane reliability.

---

# 195. Resource Limits

Enforced by:

```text
systemd
cgroups
container runtime
```

---

# 196. No Unlimited Memory

Hard rule.

---

# 197. CPU Quota

Optional.

---

# 198. Open File Limit

Explicit.

---

# 199. Network Socket Limits

Explicit.

---

# 200. Storage Quota

Per service.

---

# 201. Capacity Planning

Part 49.

---

# 202. Deployment Topology

Part 50.

---

# 203. Provider Diversity

Deployment planner should know:

```text
operator
ASN
cloud
region
```

---

# 204. Placement Policy

```rust
pub struct PlacementPolicy {
    pub avoid_same_operator: bool,
    pub avoid_same_asn: bool,
    pub avoid_same_region: bool,
}
```

---

# 205. Control Plane

Never place all authorities in one zone.

---

# 206. Mailbox Replica Placement

Across independent failure domains.

---

# 207. Mix Layer Placement

Diverse.

---

# 208. Relay Placement

Close enough for latency but diversified.

---

# 209. Deployment Planner

```rust
pub trait DeploymentPlanner {
    fn plan(
        &self,
        desired: DesiredTopology,
        inventory: InfrastructureInventory,
    ) -> Result<DeploymentPlan, DeploymentError>;
}
```

---

# 210. Deployment Plan

```rust
pub struct DeploymentPlan {
    pub placements: Vec<ServicePlacement>,
    pub artifact: ReleaseManifestRef,
    pub config: RuntimeConfigurationBundleRef,
}
```

---

# 211. Provisioning Workflow

```text
allocate host
→ apply baseline
→ enroll node
→ install exact artifact
→ inject secret refs
→ fetch signed config
→ health check
→ join provider/catalog
```

---

# 212. Node Enrollment

Part 61.

---

# 213. Service Registration

Only after readiness.

---

# 214. No Advertise-Before-Ready

Hard rule.

---

# 215. Deployment State

```rust
pub enum DeploymentState {
    Planned,
    Provisioning,
    Configuring,
    Verifying,
    Active,
    Draining,
    Retired,
    Failed,
}
```

---

# 216. Deployment Journal

Durable control-plane record.

---

# 217. Idempotent Provisioning

Required.

---

# 218. Re-run Safe

Hard rule.

---

# 219. Destructive Changes

Require plan.

---

# 220. Infrastructure Plan

Human-reviewable.

---

# 221. Terraform/OpenTofu Plan

Suitable.

---

# 222. No Blind Apply

Production changes require approval policy.

---

# 223. Drift Detection

Infrastructure state vs desired.

---

# 224. Drift Classes

```text
expected
unknown
security-critical
```

---

# 225. Security-Critical Drift

Example:

```text
public admin port opened
firewall changed
```

---

# 226. Response

Alert + potentially quarantine.

---

# 227. Immutable Infrastructure

Preferred for drift prevention.

---

# 228. Node Replacement

Preferred over manual repair for reproducible fleet.

---

# 229. Exception

Stateful mailbox/storage node.

---

# 230. Stateful Node Replacement

Requires data migration.

---

# 231. Storage Migration

Part 50/52.

---

# 232. Deployment Upgrade

Part 52 rolling upgrade model.

---

# 233. Upgrade Workflow

```text
select wave
→ drain
→ replace/install artifact
→ verify
→ rejoin
```

---

# 234. Exact Artifact Promotion

Hard rule.

---

# 235. Config Migration

Before/after binary depending compatibility.

---

# 236. Release Manifest Declares

```text
supported config schema
database schema
protocol range
```

---

# 237. Preflight Check

Node verifies compatibility.

---

# 238. Refuse Unsafe Upgrade

Hard rule.

---

# 239. Rollback

Only to compatible artifact.

---

# 240. Image Rollback

Cannot restore revoked keys/config.

---

# 241. Hard rule.

---

# 242. Deployment Failure

Automatically:

```text
stop wave
mark unhealthy
```

---

# 243. No Auto-Rollback If State Irreversible

Hard rule.

---

# 244. Repair Forward

Preferred in such case.

---

# 245. Secrets During Deployment

Secret references only.

---

# 246. Provisioner Should Not Read High-Value Secret Value

Preferred.

---

# 247. HSM Handle Provisioning

Better.

---

# 248. Secret Bootstrap

Short-lived enrollment credential.

---

# 249. Destroy Bootstrap Credential

After use.

---

# 250. No Long-Lived Bootstrap Token

Hard rule.

---

# 251. Supply-Chain Threat Model

Threats:

```text
compromised dependency
compromised CI runner
artifact substitution
registry tampering
malicious build environment
```

---

# 252. CI Runner Isolation

Release builds on hardened runner.

---

# 253. Ephemeral Runner

Preferred.

---

# 254. No Developer Laptop Release Signing

Hard rule.

---

# 255. Signing Key

HSM/offline/secure signer.

---

# 256. Build Attestation

Recommended.

---

# 257. Provenance

Record:

```text
commit
builder
toolchain
lockfiles
artifact digest
```

---

# 258. SLSA-Style Provenance

Useful model.

---

# 259. No Trust In CI Log Alone

Hard rule.

---

# 260. Artifact Verification On Node

Required before install.

---

# 261. Verify

```text
signature
digest
release epoch
compatibility
```

---

# 262. Package Manager Integration

Optional.

---

# 263. Native Packages

Can provide:

```text
.deb
.rpm
```

---

# 264. Package Repository

Signed metadata.

---

# 265. Repository Key

Separate release role.

---

# 266. No Unsigned Local Package Override

Production hard rule.

---

# 267. OCI Deployment

Image digest + signature.

---

# 268. VM Deployment

Image checksum + signature.

---

# 269. Bare Binary Deployment

Binary hash + signature.

---

# 270. Offline Bundle

Contains:

```text
artifact
SBOM
release manifest
trust chain
config schema
```

---

# 271. Offline Install

Verify locally before activation.

---

# 272. Air-Gapped Update

Use removable media.

---

# 273. Media Handling

Operator policy.

---

# 274. No Online Time Dependency

Offline verification should rely on:

```text
signed epochs
local stored anti-rollback state
```

where possible.

---

# 275. Runtime Isolation

Linux primitives.

---

# 276. seccomp

Useful.

---

# 277. Landlock

Potential.

---

# 278. Namespaces

Container/service isolation.

---

# 279. Capability Bounding

Remove unnecessary Linux capabilities.

---

# 280. No CAP_SYS_ADMIN

Hard rule unless explicit.

---

# 281. PID Namespace

Containers.

---

# 282. Network Namespace

Service dependent.

---

# 283. User Namespace

Rootless containers.

---

# 284. cgroup

Resource limits.

---

# 285. OOM Policy

Important.

---

# 286. Mailbox

Prefer graceful degradation over kill if possible.

---

# 287. Mix Node

Restart acceptable.

---

# 288. Relay

Session interruption possible.

---

# 289. Restart Policy

Per service.

---

# 290. Restart Backoff

Jittered.

---

# 291. Avoid Restart Storm

Hard rule.

---

# 292. Health Integration

Part 51.

---

# 293. Deployment Health Gate

Use:

```text
SLO
privacy canary
resource health
```

---

# 294. Readiness Probe Privacy

Infrastructure only.

---

# 295. No User Traffic Probe

Hard rule.

---

# 296. Synthetic Probe

Allowed.

---

# 297. Logs

Part 51 safe logging.

---

# 298. Host Logs

Should not contain user payload.

---

# 299. systemd Journal

Can be used.

---

# 300. Log Rotation

Required.

---

# 301. Disk Pressure

Logs should be bounded.

---

# 302. No Unlimited Container Logs

Hard rule.

---

# 303. Backups

Part 50.

---

# 304. Host Backup

Prefer:

```text
config source
infrastructure manifests
control-plane state
```

not entire machine image.

---

# 305. Why

Machine image may contain:

```text
stale secrets
temporary state
logs
```

---

# 306. Image-Based DR

Only sanitized immutable image.

---

# 307. Stateful Data Backup

Service-specific.

---

# 308. Restore

Provision fresh host + restore approved durable state.

---

# 309. No Whole-VM Snapshot As Sole DR

Hard rule.

---

# 310. Multi-Cloud

Possible.

---

# 311. Core Service Portable

Hard goal.

---

# 312. Cloud-Specific Features

Wrapped by adapter.

---

# 313. Avoid Proprietary Lock-In For Core Identity

Hard rule.

---

# 314. Managed PostgreSQL

Acceptable for control plane.

---

# 315. Managed Object Storage

Acceptable for encrypted bulk data.

---

# 316. Key Management Service

Acceptable behind secret abstraction.

---

# 317. Operator Self-Host Mode

Should remain supported.

---

# 318. Home-Lab Mode

Simplified:

```text
one VM
systemd/Podman
local DB
local secret store
```

---

# 319. Production Mode

Multi-node.

---

# 320. Edge Deployment

Potential:

```text
bridges
gateways
mailbox edge
```

---

# 321. Edge Constraints

```text
low power
intermittent network
small storage
```

---

# 322. Edge Packaging

Single binary.

---

# 323. No Kubernetes Requirement

Hard rule.

---

# 324. Embedded Linux

Part 20 integration.

---

# 325. Device Upgrade

Signed artifact.

---

# 326. Reproducible Operations

Operations docs should be code/data.

---

# 327. Repository Layout

Recommended:

```text
deploy/
├── nix/
├── systemd/
├── podman/
├── kubernetes/
├── terraform/
├── images/
├── policies/
└── runbooks/
```

---

# 328. Environment Layout

```text
environments/
├── dev/
├── staging/
└── prod/
```

---

# 329. Region Layout

```text
regions/
├── region-a/
├── region-b/
└── region-c/
```

---

# 330. No Secret In Git

Hard rule.

---

# 331. SOPS/age

Possible for encrypted low-risk deployment secret manifests.

---

# 332. High-Value Keys

HSM/secret store instead.

---

# 333. GitOps

Useful.

---

# 334. GitOps Principle

Git stores desired state.

---

# 335. But

Signed runtime bundle remains authoritative for security-sensitive config.

---

# 336. No Automatic Merge-to-Prod Apply

Hard rule for critical network.

---

# 337. Promotion Requires

```text
review
tests
signature
rollout gate
```

---

# 338. Deployment Policy

```rust
pub struct DeploymentPolicy {
    pub environment: DeploymentEnvironment,
    pub max_parallel_fraction: f32,
    pub require_canary: bool,
    pub require_privacy_gate: bool,
}
```

---

# 339. Parallelism

Bounded.

---

# 340. Authority Nodes

Very low parallelism.

---

# 341. Stateless Relays

Higher parallelism.

---

# 342. Deployment Controller

```rust
pub trait DeploymentController {
    fn plan(
        &self,
        desired: DeploymentPlan,
    ) -> Result<DeploymentExecutionPlan, DeploymentError>;

    fn apply(
        &self,
        plan: DeploymentExecutionPlan,
    ) -> Result<DeploymentReport, DeploymentError>;
}
```

---

# 343. Runtime Adapter

```rust
pub trait RuntimeAdapter {
    fn install(
        &self,
        artifact: VerifiedArtifact,
        target: RuntimeTarget,
    ) -> Result<(), DeploymentError>;

    fn start(
        &self,
        service: ServiceInstance,
    ) -> Result<(), DeploymentError>;

    fn drain(
        &self,
        service: ServiceInstance,
    ) -> Result<(), DeploymentError>;
}
```

---

# 344. Artifact Verifier

```rust
pub trait ArtifactVerifier {
    fn verify(
        &self,
        artifact: &ArtifactDescriptor,
        release: &ReleaseManifest,
    ) -> Result<VerifiedArtifact, DeploymentError>;
}
```

---

# 345. Infrastructure Inventory

```rust
pub struct InfrastructureInventory {
    pub hosts: Vec<HostRecord>,
    pub regions: Vec<RegionRecord>,
    pub networks: Vec<NetworkRecord>,
}
```

---

# 346. Inventory Privacy

Infrastructure only.

---

# 347. No User Mapping

Hard rule.

---

# 348. Deployment Observability

Safe metrics:

```text
artifact version
deployment state
host health
rollout progress
```

---

# 349. Forbidden Metrics

No:

```text
user sessions by host
contact distribution
conversation counts
```

unless aggregate operational need and privacy-reviewed.

---

# 350. Deployment SLOs

Examples:

```text
successful provisioning
artifact verification
config convergence
rollout success
```

---

# 351. Privacy Deployment SLOs

Examples:

```text
zero unsigned artifact install
zero unsigned config activation
zero plaintext secret image
```

---

# 352. Incident Types

```rust
pub enum DeploymentIncident {
    ArtifactVerificationFailure,
    ConfigMismatch,
    SecretInjectionFailure,
    HostDrift,
    RuntimeIsolationFailure,
    RegistryUnavailable,
    ProvisioningFailure,
}
```

---

# 353. Artifact Verification Failure

Critical.

---

# 354. Do Not Run Unknown Artifact

Hard rule.

---

# 355. Registry Outage

Use cached verified artifact.

---

# 356. No Fetch From Random Mirror

Hard rule.

---

# 357. Provisioning Failure

Host remains out of service.

---

# 358. No Partial Join

Hard rule.

---

# 359. Runtime Isolation Failure

Quarantine.

---

# 360. Host Drift

Rebuild where possible.

---

# 361. Security Testing

Need deployment testkit.

---

# 362. Scenarios

```text
corrupt artifact
wrong signature
wrong config
secret missing
host reboot
container crash
VM loss
registry outage
rollback attempt
```

---

# 363. Artifact Test

Tampered binary rejected.

---

# 364. Config Test

Wrong environment rejected.

---

# 365. Secret Test

Missing secret prevents readiness.

---

# 366. Rollback Test

Old release security epoch rejected if blocked.

---

# 367. Container Privilege Test

No forbidden capabilities.

---

# 368. systemd Sandbox Test

Policy enforced.

---

# 369. Firewall Test

Only required ports open.

---

# 370. Image Secret Scan

No embedded secret.

---

# 371. SBOM Test

Artifact SBOM matches release.

---

# 372. Reproducibility Test

Independent builders produce same hash where target supports.

---

# 373. Drift Test

Manual host modification detected.

---

# 374. Disaster Rebuild Test

Fresh infrastructure restored from manifests.

---

# 375. Air-Gapped Test

Offline bundle verifies and deploys.

---

# 376. Fuzzing

Fuzz:

```text
release manifest
deployment manifest
artifact metadata
host inventory
```

---

# 377. Property Tests

Properties:

```text
unverified artifact never reaches Active state
secret value never enters image/config artifact
deployment never advertises service before readiness
rollback cannot revive blocked release
```

---

# 378. Formal Verification Targets

Strong candidates:

```text
deployment state machine
artifact promotion
rollout gating
bootstrap enrollment
```

---

# 379. TLA+ Candidate

Multi-node rolling deployment with failure.

---

# 380. Kani Candidate

artifact/config compatibility rules.

---

# 381. Loom Candidate

concurrent deployment controller state.

---

# 382. Performance Tests

Measure:

```text
cold start
image pull
binary startup
config load
secret injection
```

---

# 383. Startup Target

Service-specific.

---

# 384. Mix Node

Fast restart desirable.

---

# 385. Mailbox

Data consistency more important than fast startup.

---

# 386. Relay

Fast availability important.

---

# 387. Image Size

Keep minimal.

---

# 388. Network Pull Cost

Important for multi-region rollout.

---

# 389. Delta Updates

Potential.

---

# 390. But

Final reconstructed artifact must verify exact expected digest.

---

# 391. Hard rule.

---

# 392. Deployment Crate Layout

Recommended:

```text
crates/
├── siar-deployment-core/
├── siar-artifact/
├── siar-release-manifest/
├── siar-build-provenance/
├── siar-host-profile/
├── siar-runtime-adapter/
├── siar-provisioning/
├── siar-placement/
├── siar-deployment-controller/
├── siar-deployment-observability/
└── siar-deployment-testkit/
```

---

# 393. `siar-deployment-core`

Owns:

```text
deployment states
targets
errors
```

---

# 394. `siar-artifact`

Artifact verification.

---

# 395. `siar-release-manifest`

Signed release metadata.

---

# 396. `siar-build-provenance`

Build/SBOM/provenance.

---

# 397. `siar-host-profile`

Bare-metal/VM/container host profiles.

---

# 398. `siar-runtime-adapter`

systemd/Podman/Kubernetes adapters.

---

# 399. `siar-provisioning`

Host/cloud provisioning abstractions.

---

# 400. `siar-placement`

Failure-domain-aware placement.

---

# 401. `siar-deployment-controller`

Plan/apply/upgrade/drain.

---

# 402. `siar-deployment-observability`

Privacy-safe deployment metrics.

---

# 403. `siar-deployment-testkit`

Fault/rebuild/reproducibility simulator.

---

# 404. Error Taxonomy

```rust
pub enum DeploymentError {
    ArtifactInvalid,
    SignatureInvalid,
    SbomMismatch,
    ProvenanceInvalid,
    ConfigIncompatible,
    SecretUnavailable,
    HostUnsupported,
    RuntimeIsolationFailed,
    PlacementUnsatisfied,
    ProvisioningFailed,
    DriftDetected,
    RolloutPaused,
    RollbackBlocked,
    Internal,
}
```

---

# 405. Security & Operational Invariants

Mandatory:

```text
1. Production artifacts are built once, signed, content-addressed, and promoted unchanged.
2. No production node runs an unsigned or digest-mismatched artifact.
3. Build inputs, toolchain, Cargo.lock, and environment are pinned for reproducibility.
4. Secrets never appear in source control, image layers, release manifests, SBOMs, logs, or ordinary configuration.
5. Services are deployed with least privilege and explicit resource/network boundaries.
6. No service is advertised before readiness checks succeed.
7. Runtime environment differences cannot silently alter protocol/security semantics.
8. Deployment rollback cannot resurrect revoked keys, blocked protocol versions, or stale configuration.
9. Infrastructure can be rebuilt from declarative manifests without relying on undocumented manual state.
10. Kubernetes/container orchestration is optional; bare-metal/VM/systemd remains first-class.
11. Failure-domain placement respects operator/region/ASN/cloud diversity requirements.
12. Deployment telemetry contains infrastructure state only and does not become a user-routing map.
```

---

# 406. Initial Production Scope

Implement first:

```text
reproducible Rust release build
pinned toolchain/Cargo.lock
release manifest
SBOM
signed static/minimal Linux binaries
systemd deployment
Podman deployment
Nix/NixOS reproducible host option
OpenTofu/Terraform provisioning adapter
signed config integration
secret-ref injection
fault-domain-aware placement
artifact verification on node
rolling deployment controller
deployment drift detection
air-gapped offline bundle
```

Then add:

```text
Kubernetes adapter
multi-cloud deployment planner
remote attestation for infrastructure nodes
independent reproducible builders
delta artifact transport
automated bare-metal PXE provisioning
formal deployment-state verification
```

---

# 407. Definition of Done

Part 62 is complete when:

- build/release/deployment boundaries are explicit
- binaries/images/packages are content-addressed and signed
- exact artifact promotion is enforced
- SBOM and provenance are generated
- Rust/toolchain/dependency inputs are pinned
- bare-metal, VM, container, and air-gapped deployments are supported
- systemd and Podman are first-class runtimes
- Kubernetes remains optional
- host hardening and service sandboxing are explicit
- config and secrets remain separate
- node enrollment/bootstrap is authenticated
- infrastructure provisioning is declarative
- placement respects anonymous-network diversity requirements
- deployment rollout integrates with Part 52
- DR rebuild integrates with Part 50
- artifact/config/secret/drift/reproducibility tests are specified
- deployments can be recreated without undocumented manual state

---

# 408. Final Architecture

```text
                       SOURCE + LOCKFILES
                              │
                              ▼
                     HERMETIC BUILD
                              │
                              ▼
                    SIGNED RELEASE ARTIFACT
                              │
                              ▼
                       ARTIFACT REGISTRY
                              │
               ┌──────────────┼──────────────┐
               │              │              │
           Bare Metal         VM          Container
               │              │              │
               └──────────────┼──────────────┘
                              ▼
                    SIGNED CONFIGURATION
                              │
                              ▼
                     SCOPED SECRET INJECTION
                              │
                              ▼
                         ACTIVE NODE
```

Reproducible-operations model:

```text
pinned source
+
hermetic build
+
signed artifacts
+
declarative infrastructure
+
immutable deployment
+
scoped secrets
+
fault-domain-aware placement
```

not:

```text
manually SSH into servers and change things until they work
```

---

# 409. Final Principle

Anonymous infrastructure should be **reproducible enough that a failed or compromised node can be destroyed and rebuilt from trusted inputs rather than repaired through undocumented state**.

The correct model is:

```text
reproducible build
+
signed artifact
+
declarative provisioning
+
isolated runtime
+
immutable configuration
+
verifiable deployment
```

This architecture gives SIAR a production deployment foundation that works across self-hosted, bare-metal, VM, container, regional, multi-operator, and air-gapped environments while preserving the operational and privacy guarantees established across Parts 34–61.
