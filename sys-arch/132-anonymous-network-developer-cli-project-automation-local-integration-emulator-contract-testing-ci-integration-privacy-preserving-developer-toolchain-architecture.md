# Core System Architecture Part 132 — Anonymous Network Developer CLI, Project Automation, Local Integration Emulator, Contract Testing, CI Integration & Privacy-Preserving Developer Toolchain Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 132  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 19, 27, 68, 99, 122–131

**Primary purpose:** define SIAR's developer-toolchain architecture for command-line automation, project bootstrapping, local integration emulation, contract testing, CI integration, ephemeral credentials, deterministic fixtures, offline development, failure injection, compatibility validation, and privacy-preserving automation.

---

# 1. Purpose

A developer platform needs more than a portal and SDK.

Developers also need a reliable toolchain for:

```text
project setup
automation
local testing
CI validation
version checks
sandbox control
contract verification
credential rotation
```

The governing principle is:

> **SIAR developer tooling should make the secure and privacy-preserving path automatable from local development through CI without requiring production access or hidden privileged shortcuts.**

---

# 2. Architectural Position

```text
                    DEVELOPER
                        │
                        ▼
                   SIAR CLI
                        │
          ┌─────────────┼─────────────┐
          │             │             │
       PROJECT       EMULATOR       CI
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                CONTRACT / COMPAT
                        │
                        ▼
                  SANDBOX / CERT
                        │
                        ▼
                 PRODUCTION GATE
```

---

# 3. Core Separation

Keep distinct:

```text
local emulator
shared sandbox
certification environment
production
CLI identity
runtime integration identity
CI workload identity
```

---

# 4. Non-Goals

Part 132 does not create:

```text
production-by-default tooling
shared local secrets
developer laptop superuser keys
production data emulation
CLI commands that bypass certification
```

---

# 5. CLI Identity

```rust
pub struct DeveloperCliId(pub [u8; 16]);
```

---

# 6. CLI Session

```rust
pub struct DeveloperCliSession {
    pub account: DeveloperPortalAccountId,
    pub organization: DeveloperOrganizationId,
    pub project: Option<DeveloperProjectId>,
    pub environment: DeveloperEnvironment,
}
```

---

# 7. Explicit Environment

Every command runs under explicit environment context.

---

# 8. Hard rule.

---

# 9. No Production Default

Default:

```text
local
or
sandbox
```

---

# 10. Hard rule.

---

# 11. CLI Command Model

```rust
pub enum DeveloperCliCommand {
    Init,
    Login,
    Project,
    Sandbox,
    Emulator,
    Credential,
    Contract,
    Test,
    Compat,
    Certify,
    Status,
    Doctor,
}
```

---

# 12. Typed Subcommands

Avoid free-form privileged shell behavior.

---

# 13. Hard rule.

---

# 14. Project Initialization

`siar init`

Creates:

```text
siar.ron
integration manifest
example source
test config
CI template
```

---

# 15. Hard rule.

---

# 16. Project Manifest

```rust
pub struct DeveloperProjectManifest {
    pub project_name: String,
    pub sdk_version: SdkVersion,
    pub target_environment: DeveloperEnvironment,
    pub requested_capabilities: BTreeSet<SdkCapability>,
}
```

---

# 17. Human-Readable Config

RON preferred.

---

# 18. Hard rule.

---

# 19. Project File

Example:

```ron
(
    project_name: "example-integration",
    sdk_version: "1.4.0",
    target_environment: "Sandbox",
    requested_capabilities: [
        "SendMessages",
        "ReadConversations",
    ],
)
```

---

# 20. Hard rule.

---

# 21. Local State Directory

Example:

```text
.siar/
├── state/
├── cache/
├── emulator/
├── fixtures/
└── logs/
```

---

# 22. Secrets Excluded

Hard rule.

---

# 23. Secret Location

Use OS keychain/secret store.

---

# 24. Hard rule.

---

# 25. CLI Authentication

Use portal login/passkey/device flow where supported.

---

# 26. No password storage.

---

# 27. Hard rule.

---

# 28. CLI Token

Short-lived.

---

# 29. Hard rule.

---

# 30. Refresh Credential

Stored in secure OS store.

---

# 31. Hard rule.

---

# 32. CI Authentication

No human token reuse.

---

# 33. Hard rule.

---

# 34. CI Workload Identity

```rust
pub struct CiWorkloadIdentity {
    pub project: DeveloperProjectId,
    pub environment: DeveloperEnvironment,
    pub scopes: BTreeSet<SdkCapability>,
    pub expires_at: Timestamp,
}
```

---

# 35. Expiring

Hard rule.

---

# 36. OIDC Federation

Preferred where CI provider supports it.

---

# 37. Hard rule.

---

# 38. No Long-Lived CI Secret By Default

Hard rule.

---

# 39. CLI Project Commands

Examples:

```text
siar project show
siar project validate
siar project diff
siar project sync
```

---

# 40. Hard rule.

---

# 41. Manifest Validation

Validate:

```text
capabilities
SDK version
compatibility
environment
schema
```

---

# 42. Hard rule.

---

# 43. Capability Diff

CLI shows:

```text
added scope
removed scope
data-class change
network/file-access change
```

---

# 44. Hard rule.

---

# 45. No Silent Scope Expansion

Hard rule.

---

# 46. Project Automation

Automate repeatable operations.

---

# 47. Automation Plan

```rust
pub struct DeveloperAutomationPlan {
    pub project: DeveloperProjectId,
    pub steps: Vec<DeveloperAutomationStep>,
}
```

---

# 48. Step

```rust
pub enum DeveloperAutomationStep {
    ValidateManifest,
    Build,
    RunContractTests,
    RunCompatibilityChecks,
    ProvisionSandbox,
    PublishEvidence,
}
```

---

# 49. No Production Credential Creation In Generic Automation Step

Hard rule.

---

# 50. Local Integration Emulator

Purpose:

```text
develop without internet
test deterministic behavior
simulate APIs/events
exercise failure paths
```

---

# 51. Emulator Identity

```rust
pub struct LocalEmulatorId(pub [u8; 16]);
```

---

# 52. Emulator Mode

```rust
pub enum EmulatorMode {
    Minimal,
    Standard,
    FullProtocolSimulation,
}
```

---

# 53. Hard rule.

---

# 54. Local-Only Binding

Default emulator binds:

```text
localhost
or
Unix domain socket
```

---

# 55. Hard rule.

---

# 56. No Public Network Bind By Default

Hard rule.

---

# 57. Emulator Data

Synthetic only.

---

# 58. Hard rule.

---

# 59. Emulator Storage

Embedded local store.

Recommended:

```text
SQLite/Stoolap-compatible abstraction
```

depending implementation.

---

# 60. Hard rule.

---

# 61. Emulator Reset

```text
siar emulator reset
```

Deterministic reset.

---

# 62. Hard rule.

---

# 63. Emulator Seed

```rust
pub struct EmulatorSeed(pub u64);
```

---

# 64. Reproducibility

Same seed + config → same fixture state where feasible.

---

# 65. Hard rule.

---

# 66. Synthetic Accounts

```rust
pub struct SyntheticAccountId(pub [u8; 16]);
```

---

# 67. Explicitly synthetic.

---

# 68. Hard rule.

---

# 69. Synthetic Conversations

Synthetic fixture graph.

---

# 70. Hard rule.

---

# 71. No Production IDs

Hard rule.

---

# 72. Fixture Package

```rust
pub struct FixturePackage {
    pub fixture_id: FixturePackageId,
    pub version: FixtureVersion,
    pub digest: Digest,
}
```

---

# 73. Versioned Fixtures

Hard rule.

---

# 74. Fixture Classes

```rust
pub enum FixtureClass {
    Minimal,
    Messaging,
    Attachments,
    MultiDevice,
    Offline,
    FailureScenario,
}
```

---

# 75. Hard rule.

---

# 76. Fixture Provenance

Signed where distributed officially.

---

# 77. Hard rule.

---

# 78. Emulator API

Matches stable SDK semantics.

---

# 79. Not internal implementation shortcuts.

---

# 80. Hard rule.

---

# 81. Emulator Fidelity

```rust
pub enum EmulatorFidelity {
    ContractOnly,
    Behavioral,
    ProtocolApproximation,
}
```

---

# 82. Never label approximation as production equivalent.

---

# 83. Hard rule.

---

# 84. Network Simulation

Emulator can simulate:

```text
latency
packet loss
offline
partition
reconnect
```

---

# 85. Hard rule.

---

# 86. Network Profile

```rust
pub struct EmulatorNetworkProfile {
    pub latency: Duration,
    pub packet_loss_percent: u8,
    pub offline: bool,
}
```

---

# 87. Bounded values.

---

# 88. Hard rule.

---

# 89. Failure Injection

```rust
pub enum EmulatorFailure {
    TransportUnavailable,
    Timeout,
    RateLimited,
    CredentialExpired,
    VersionMismatch,
    StorageFull,
}
```

---

# 90. Hard rule.

---

# 91. No Arbitrary Code Injection

Hard rule.

---

# 92. Clock Control

Deterministic simulated clock.

---

# 93. Hard rule.

---

# 94. Randomness Control

Explicit RNG seed.

---

# 95. Hard rule.

---

# 96. Local Webhook Emulator

Supports:

```text
event delivery
retry
signature verification
replay tests
```

---

# 97. Hard rule.

---

# 98. Webhook Receiver

Local endpoint.

---

# 99. Hard rule.

---

# 100. Webhook Signing

Uses sandbox/emulator signing key.

---

# 101. Never production key.

---

# 102. Hard rule.

---

# 103. Replay Testing

```rust
pub enum ReplayScenario {
    ValidReplayRejected,
    ExpiredTimestampRejected,
    WrongSignatureRejected,
}
```

---

# 104. Hard rule.

---

# 105. Contract Testing

Verifies integration expectations against stable SDK/API contracts.

---

# 106. Contract Test Identity

```rust
pub struct ContractTestId(pub [u8; 16]);
```

---

# 107. Contract Class

```rust
pub enum ContractClass {
    RequestResponse,
    EventSchema,
    ErrorSemantics,
    CapabilityAuthorization,
    RateLimit,
    VersionNegotiation,
}
```

---

# 108. Hard rule.

---

# 109. Contract Test Manifest

```rust
pub struct ContractTestManifest {
    pub sdk_version: SdkVersion,
    pub api_versions: BTreeMap<ApiId, ApiVersion>,
    pub tests: BTreeSet<ContractTestId>,
}
```

---

# 110. Hard rule.

---

# 111. Consumer Contract

Integration declares expected behavior.

---

# 112. Hard rule.

---

# 113. Provider Verification

Sandbox/cert env validates contract.

---

# 114. Hard rule.

---

# 115. Contract Drift

Detected when:

```text
field removed
error semantics changed
capability requirement changed
version support changed
```

---

# 116. Hard rule.

---

# 117. Contract Snapshot

Immutable per certification run.

---

# 118. Hard rule.

---

# 119. No Snapshot Blind Acceptance

Hard rule.

---

# 120. Compatibility Testing

Uses Part 129 compatibility registry.

---

# 121. CLI Commands

```text
siar compat check
siar compat matrix
siar compat explain
```

---

# 122. Hard rule.

---

# 123. Compatibility Explain

Returns:

```text
supported
unsupported
migration required
security floor mismatch
```

---

# 124. Hard rule.

---

# 125. No User-Specific Compatibility Check

Hard rule.

---

# 126. Local SDK Version Check

Compare:

```text
project SDK
binding version
API/protocol support
```

---

# 127. Hard rule.

---

# 128. Generated Binding Validation

CLI verifies provenance digest.

---

# 129. Hard rule.

---

# 130. Contract Test Environment

Possible:

```text
local emulator
shared sandbox
certification environment
```

---

# 131. Hard rule.

---

# 132. Contract Test Evidence

```rust
pub struct ContractTestEvidence {
    pub test: ContractTestId,
    pub environment: DeveloperEnvironment,
    pub artifact_digest: Digest,
    pub result: VerificationResult,
}
```

---

# 133. Hard rule.

---

# 134. CI Integration

CI should support:

```text
manifest validation
SDK compatibility
contract tests
linting
package provenance
certification evidence publishing
```

---

# 135. Hard rule.

---

# 136. CI Job Definition

```rust
pub struct DeveloperCiPlan {
    pub project: DeveloperProjectId,
    pub jobs: Vec<DeveloperCiJob>,
}
```

---

# 137. CI Job

```rust
pub enum DeveloperCiJob {
    Validate,
    Build,
    Test,
    Contract,
    Compat,
    Package,
    Evidence,
}
```

---

# 138. Hard rule.

---

# 139. CI Evidence

Signed by workload identity.

---

# 140. Hard rule.

---

# 141. No Developer Laptop Identity In CI

Hard rule.

---

# 142. Reproducible CI

Pin:

```text
SDK version
toolchain
dependencies
fixtures
emulator version
```

---

# 143. Hard rule.

---

# 144. CI Environment Manifest

```rust
pub struct DeveloperCiEnvironment {
    pub rust_toolchain: String,
    pub sdk_version: SdkVersion,
    pub emulator_version: EmulatorVersion,
    pub fixture_digest: Digest,
}
```

---

# 145. Hard rule.

---

# 146. CI Cache

Safe for:

```text
build artifacts
dependency downloads
generated bindings
```

---

# 147. Never cache plaintext secrets.

---

# 148. Hard rule.

---

# 149. Ephemeral CI Credentials

Short-lived per job.

---

# 150. Hard rule.

---

# 151. Credential Audience

Bound to:

```text
project
environment
job
```

---

# 152. Hard rule.

---

# 153. CI Promotion

CI cannot self-promote uncertified integration to production.

---

# 154. Hard rule.

---

# 155. Certification Submission

CLI can submit evidence.

---

# 156. But certification decision remains separate authority.

---

# 157. Hard rule.

---

# 158. CLI Certification Commands

```text
siar certify prepare
siar certify submit
siar certify status
```

---

# 159. Hard rule.

---

# 160. Certification Package

```rust
pub struct IntegrationCertificationPackage {
    pub project: DeveloperProjectId,
    pub manifest_digest: Digest,
    pub contract_evidence: Vec<ContractTestEvidenceId>,
    pub compatibility_baseline: CompatibilityBaselineId,
}
```

---

# 161. Hard rule.

---

# 162. Local Validation

`siar certify prepare`

checks:

```text
manifest
scopes
contract tests
compatibility
package provenance
```

---

# 163. Hard rule.

---

# 164. Doctor Command

```text
siar doctor
```

Checks:

```text
CLI version
SDK version
credentials
environment
network
sandbox access
compatibility
```

---

# 165. Hard rule.

---

# 166. Doctor Output

Privacy-safe.

---

# 167. No raw token display.

---

# 168. Hard rule.

---

# 169. Diagnostic Bundle

CLI can generate redacted bundle.

---

# 170. Hard rule.

---

# 171. Bundle Fields

```text
version
environment
error classes
compatibility state
credential metadata
```

---

# 172. No secrets/private content.

---

# 173. Hard rule.

---

# 174. CLI Output Formats

Human:

```text
plain text
table
```

Machine:

```text
JSON
```

where external automation needs it.

---

# 175. Internal config remains RON.

---

# 176. Hard rule.

---

# 177. Stable Machine Output

Versioned schema.

---

# 178. Hard rule.

---

# 179. Exit Codes

Stable categories.

```rust
pub enum DeveloperCliExitCode {
    Success = 0,
    InvalidInput = 2,
    AuthFailure = 3,
    CompatibilityFailure = 4,
    ContractFailure = 5,
    CertificationFailure = 6,
    Internal = 70,
}
```

---

# 180. Hard rule.

---

# 181. Shell Completion

Generated.

---

# 182. Hard rule.

---

# 183. CLI Plugin System

Not required initially.

---

# 184. If added, capability-scoped.

---

# 185. Hard rule.

---

# 186. No Arbitrary Native Plugin Loading By Default

Hard rule.

---

# 187. Developer CLI Updates

Signed packages.

---

# 188. Hard rule.

---

# 189. CLI Self-Update

Optional.

---

# 190. Must verify signature/provenance.

---

# 191. Hard rule.

---

# 192. Offline Development

Core requirement.

---

# 193. Offline Capabilities

```text
init
validate
emulator
contract tests
compat checks from cached registry
docs snapshot
```

---

# 194. Hard rule.

---

# 195. Cached Compatibility Registry

Signed.

---

# 196. Hard rule.

---

# 197. Expired Registry

Show warning/fail restrictive operations.

---

# 198. Never permissive fallback.

---

# 199. Hard rule.

---

# 200. Offline Docs Bundle

Versioned docs snapshot.

---

# 201. Hard rule.

---

# 202. Offline Fixture Bundle

Signed versioned fixtures.

---

# 203. Hard rule.

---

# 204. Local Project Lockfile

```rust
pub struct DeveloperProjectLock {
    pub sdk_version: SdkVersion,
    pub cli_version: DeveloperCliVersion,
    pub emulator_version: EmulatorVersion,
    pub fixture_digest: Digest,
}
```

---

# 205. Hard rule.

---

# 206. Lockfile Prevents Environment Drift

Hard rule.

---

# 207. Toolchain Manager

Optional.

---

# 208. Pins compatible CLI/SDK/emulator versions.

---

# 209. Hard rule.

---

# 210. Environment Reproduction

`siar env export`

Produces:

```text
tool versions
fixture digest
SDK version
compat baseline
```

---

# 211. No secrets.

---

# 212. Hard rule.

---

# 213. Environment Import

`siar env apply`

---

# 214. Hard rule.

---

# 215. Local Emulator Networking

Can simulate:

```text
same-device
LAN-like
high latency
offline
```

---

# 216. Hard rule.

---

# 217. Anonymous Network Simulation

May simulate metadata-level behavior.

---

# 218. Must not claim real anonymity.

---

# 219. Hard rule.

---

# 220. Mixnet Emulator

Optional later.

---

# 221. Hard rule.

---

# 222. Protocol Emulator

Uses production protocol library where safe.

---

# 223. Avoid duplicate fake protocol implementation.

---

# 224. Hard rule.

---

# 225. Emulated Crypto

Prefer real crypto with synthetic keys.

---

# 226. Hard rule.

---

# 227. No Disabled Crypto Shortcut

Hard rule.

---

# 228. Synthetic Key Store

Ephemeral/local.

---

# 229. Hard rule.

---

# 230. Multi-Device Emulation

Emulator can spawn synthetic device instances.

---

# 231. Hard rule.

---

# 232. Device Profile

```rust
pub struct SyntheticDeviceProfile {
    pub platform: PlatformClass,
    pub sdk_version: SdkVersion,
    pub protocol_versions: BTreeMap<ProtocolId, ProtocolVersion>,
}
```

---

# 233. Hard rule.

---

# 234. Version-Skew Emulation

Important for Part 129.

---

# 235. Hard rule.

---

# 236. Offline Queue Emulation

Test:

```text
persist
retry
delivery
dedup
```

---

# 237. Hard rule.

---

# 238. Attachment Emulation

Uses synthetic files.

---

# 239. Hard rule.

---

# 240. File Size Limits

Bounded.

---

# 241. Hard rule.

---

# 242. Media Emulation

Optional.

---

# 243. Never use real camera/mic automatically.

---

# 244. Hard rule.

---

# 245. Contract Assertions

Typed assertions.

```rust
pub enum ContractAssertion {
    Status(ApiStatusCode),
    ErrorClass(SdkErrorClass),
    CapabilityRequired(SdkCapability),
    EventSchema(EventSchemaId),
}
```

---

# 246. Hard rule.

---

# 247. Contract Test DSL

RON-based optional DSL.

---

# 248. Hard rule.

---

# 249. Example

```ron
(
    request: "send_message",
    given: ["valid_session"],
    expect: [
        CapabilityRequired("SendMessages"),
        Status("Accepted"),
    ],
)
```

---

# 250. Hard rule.

---

# 251. Property-Based Contract Tests

Generate bounded combinations.

---

# 252. Hard rule.

---

# 253. Fuzz Contract Boundary

Fuzz request/event decoders.

---

# 254. Hard rule.

---

# 255. Golden Contract Fixtures

Versioned.

---

# 256. Hard rule.

---

# 257. Breaking Contract Change

CI flags.

---

# 258. Hard rule.

---

# 259. Contract Baseline

```rust
pub struct ContractBaseline {
    pub sdk_version: SdkVersion,
    pub api_versions: BTreeMap<ApiId, ApiVersion>,
    pub schema_digests: BTreeMap<SchemaId, Digest>,
}
```

---

# 260. Immutable per certification.

---

# 261. Hard rule.

---

# 262. CI Compatibility Gate

Blocks if:

```text
unsupported SDK
breaking contract
revoked certification dependency
```

---

# 263. Hard rule.

---

# 264. Project Automation Hooks

Allow:

```text
pre-test
post-test
custom build
```

---

# 265. Hard rule.

---

# 266. Hook Sandbox

No implicit secret inheritance.

---

# 267. Hard rule.

---

# 268. Hook Environment

Explicit whitelist.

---

# 269. Hard rule.

---

# 270. No Arbitrary Production Access From Hook

Hard rule.

---

# 271. CLI Logging

Structured local logs.

---

# 272. Hard rule.

---

# 273. Log Redaction

Remove:

```text
tokens
secrets
message content
private identifiers
```

---

# 274. Hard rule.

---

# 275. Debug Mode

Still redacts secrets.

---

# 276. Hard rule.

---

# 277. Verbose Mode

May include protocol metadata only within policy.

---

# 278. Hard rule.

---

# 279. Telemetry

CLI telemetry opt-in or privacy-preserving aggregate per policy.

---

# 280. Hard rule.

---

# 281. No Command History Upload By Default

Hard rule.

---

# 282. Local Command History

Shell-owned.

---

# 283. Hard rule.

---

# 284. Crash Reports

User-controlled/sanitized.

---

# 285. Hard rule.

---

# 286. Developer Analytics Boundary

Allowed aggregate:

```text
CLI version
error class
emulator crash
contract failure class
```

---

# 287. Forbidden:

```text
full commands
file paths
source code
developer productivity
user data
```

---

# 288. Hard rule.

---

# 289. Team Automation

Project config can be committed.

---

# 290. Secrets excluded.

---

# 291. Hard rule.

---

# 292. Monorepo Support

Multiple integrations in workspace.

---

# 293. Hard rule.

---

# 294. Workspace Manifest

```rust
pub struct DeveloperWorkspaceManifest {
    pub projects: Vec<DeveloperProjectRef>,
    pub shared_sdk_version: Option<SdkVersion>,
}
```

---

# 295. Hard rule.

---

# 296. Per-Project Isolation

Credentials/scopes separate.

---

# 297. Hard rule.

---

# 298. Project Matrix CI

Run contract tests for each integration.

---

# 299. Hard rule.

---

# 300. Caching Across Projects

Only immutable public artifacts.

---

# 301. Hard rule.

---

# 302. Local Certification Preview

Shows blockers before submission.

---

# 303. Hard rule.

---

# 304. Certification Preview

```rust
pub struct CertificationPreview {
    pub blocking_contracts: Vec<ContractTestId>,
    pub compatibility_issues: Vec<CompatibilityIssue>,
    pub scope_changes: Vec<SdkCapability>,
}
```

---

# 305. Hard rule.

---

# 306. No Local Self-Certification Authority

Hard rule.

---

# 307. Developer Toolchain Status

`siar status`

Shows:

```text
project
environment
SDK
CLI
sandbox
certification
compatibility
```

---

# 308. Hard rule.

---

# 309. Security Status

Never expose internal secrets/topology.

---

# 310. Hard rule.

---

# 311. CLI Doctor Security Checks

Examples:

```text
world-readable config
secret in env file
stale credential
unsigned SDK package
```

---

# 312. Hard rule.

---

# 313. Local Secret Scanner

Optional.

---

# 314. Scope only project paths.

---

# 315. Hard rule.

---

# 316. No Home-Directory-Wide Scanning

Hard rule.

---

# 317. Project Template Security

`.gitignore` includes secret paths.

---

# 318. Hard rule.

---

# 319. CI Template Security

Uses workload identity.

---

# 320. Hard rule.

---

# 321. Code Generation

CLI can generate:

```text
client skeleton
webhook verifier
config
contract test
```

---

# 322. Hard rule.

---

# 323. Generated Code Provenance

Comment/header identifies generator version.

---

# 324. Hard rule.

---

# 325. Generated Code Review

User owns final integration behavior.

---

# 326. Hard rule.

---

# 327. No Hidden Network Calls In Generated Templates

Hard rule.

---

# 328. Dependency Pinning

Generated projects pin supported SDK major/minor policy.

---

# 329. Hard rule.

---

# 330. Toolchain Supply Chain

CLI, emulator, fixtures, codegen signed.

---

# 331. Hard rule.

---

# 332. SBOM

Published for toolchain releases.

---

# 333. Hard rule.

---

# 334. Reproducible Builds

Preferred.

---

# 335. Hard rule.

---

# 336. Toolchain Update Policy

Version-aware.

---

# 337. Hard rule.

---

# 338. Auto-Update

Optional.

---

# 339. Never mandatory without verification.

---

# 340. Hard rule.

---

# 341. Deprecated CLI Version

Warn with support timeline.

---

# 342. Hard rule.

---

# 343. Unsupported CLI

May block production-sensitive actions.

---

# 344. Local emulator remains usable where safe.

---

# 345. Hard rule.

---

# 346. Compatibility Between CLI And Portal

Versioned management API.

---

# 347. Hard rule.

---

# 348. CLI/Portal Contract

Certified like other API surfaces.

---

# 349. Hard rule.

---

# 350. CI Provider Neutrality

No hard dependency on one CI vendor.

---

# 351. Hard rule.

---

# 352. CI Integration Adapter

Supports:

```text
GitHub Actions
GitLab CI
generic OIDC CI
self-hosted runners
```

as adapters.

---

# 353. Hard rule.

---

# 354. Generic Core

Provider-neutral pipeline model.

---

# 355. Hard rule.

---

# 356. Self-Hosted Runner

Must still use scoped workload identity.

---

# 357. Hard rule.

---

# 358. Local Runner

Useful for air-gapped/offline projects.

---

# 359. Hard rule.

---

# 360. Air-Gapped Toolchain

Bundle:

```text
CLI
SDK
docs
fixtures
emulator
compat registry snapshot
```

---

# 361. Signed offline bundle.

---

# 362. Hard rule.

---

# 363. Bundle Freshness

Explicit.

---

# 364. Hard rule.

---

# 365. Offline Certification

Can prepare evidence offline.

---

# 366. Final certification sync requires authorized channel unless fully offline governance exists.

---

# 367. Hard rule.

---

# 368. Developer Toolchain API

```rust
pub trait DeveloperToolchainService {
    fn validate_project(
        &self,
        manifest: DeveloperProjectManifest,
    ) -> Result<ProjectValidationReport, DeveloperToolchainError>;

    fn prepare_certification(
        &self,
        project: DeveloperProjectId,
    ) -> Result<CertificationPreview, DeveloperToolchainError>;
}
```

---

# 369. Emulator Service

```rust
pub trait LocalEmulatorService {
    fn start(
        &self,
        config: EmulatorConfig,
    ) -> Result<LocalEmulatorId, DeveloperToolchainError>;

    fn reset(
        &self,
        emulator: LocalEmulatorId,
    ) -> Result<(), DeveloperToolchainError>;
}
```

---

# 370. Contract Test Service

```rust
pub trait ContractTestService {
    fn run(
        &self,
        baseline: ContractBaseline,
        environment: DeveloperEnvironment,
    ) -> Result<ContractTestRun, DeveloperToolchainError>;
}
```

---

# 371. CI Evidence Service

```rust
pub trait DeveloperCiEvidenceService {
    fn publish(
        &self,
        evidence: DeveloperCiEvidence,
    ) -> Result<EvidenceRef, DeveloperToolchainError>;
}
```

---

# 372. No Production Bypass API

Hard rule.

---

# 373. Error Taxonomy

```rust
pub enum DeveloperToolchainError {
    ProjectInvalid,
    CliVersionUnsupported,
    SdkVersionUnsupported,
    EmulatorUnavailable,
    ContractFailed,
    CompatibilityFailed,
    CredentialExpired,
    CredentialScopeDenied,
    CertificationBlocked,
    OfflineRegistryExpired,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 374. Observability

Safe metrics:

```text
CLI version
emulator startup failures
contract-test result class
compatibility failure class
CI evidence publication state
```

---

# 375. Forbidden:

```text
full commands
developer files
developer productivity
user behavior
```

---

# 376. Hard rule.

---

# 377. Toolchain SLOs

Examples:

```text
local emulator startup within target
contract test reproducibility within target
credential revocation reflected within target
signed offline bundle available for supported releases
```

---

# 378. Security SLO

```text
0 production credential stored in local project config
0 sandbox/CI credential authorizing production by mistake
0 unsigned toolchain component accepted when verification required
```

---

# 379. Privacy SLO

```text
0 production user data in emulator fixtures
0 source-code upload required for normal CLI operation
0 command history telemetry by default
```

---

# 380. Failure Modes

```text
local/production environment confusion
emulator divergence
contract drift
CI secret leakage
stale offline registry
```

---

# 381. Environment Confusion

Explicit environment binding + endpoint/credential class checks.

---

# 382. Hard rule.

---

# 383. Emulator Divergence

Conformance tests against real sandbox.

---

# 384. Hard rule.

---

# 385. Contract Drift

CI fails.

---

# 386. Hard rule.

---

# 387. CI Secret Leakage

Prefer workload identity + redaction.

---

# 388. Hard rule.

---

# 389. Stale Offline Registry

Restrictive failure.

---

# 390. Hard rule.

---

# 391. Testing

Need developer-toolchain testkit.

---

# 392. Test Scenarios

```text
offline project init
emulator version skew
contract drift
CI workload identity
certification preparation
```

---

# 393. Environment Test

Production command using sandbox credential rejected.

---

# 394. Hard rule.

---

# 395. Emulator Determinism Test

Same seed/config yields same synthetic state.

---

# 396. Contract Test

Breaking API field removal detected.

---

# 397. Compatibility Test

Unsupported SDK version fails before runtime.

---

# 398. CI Credential Test

Credential audience bound to CI job/project.

---

# 399. Offline Test

Expired compatibility registry does not permissively approve production-sensitive action.

---

# 400. Privacy Test

Fixture bundle contains no production user identifiers.

---

# 401. Fuzzing

Fuzz:

```text
CLI args
project manifests
contract test DSL
emulator config
CI evidence envelopes
```

---

# 402. Property Tests

Properties:

```text
sandbox credential can never authorize production endpoint
generated contract baseline can never silently omit mandatory capability checks
local emulator can never load production credential through project config
expired compatibility registry can never lower production safety checks
```

---

# 403. Formal Verification Targets

Strong candidates:

```text
environment/credential binding
project lifecycle automation
certification gate
contract baseline evolution
```

---

# 404. Kani Candidate

credential/environment/CLI-state invariants.

---

# 405. TLA+ Candidate

init → sandbox → contract → certification → production.

---

# 406. Loom Candidate

concurrent credential rotation + CI request + revocation.

---

# 407. Performance

CLI startup should be fast.

---

# 408. Emulator idle CPU near zero.

---

# 409. Hard rule.

---

# 410. CI parallelism

Bounded.

---

# 411. Hard rule.

---

# 412. Cache

Use immutable caches for:

```text
SDK
CLI artifacts
fixtures
compat registry
```

---

# 413. Hard rule.

---

# 414. Storage

Separate:

```text
project manifests
local emulator state
fixture cache
contract baselines
CI evidence metadata
toolchain lockfiles
```

---

# 415. Secrets outside general storage.

---

# 416. Hard rule.

---

# 417. Partitioning

By:

```text
project
environment
contract baseline
SDK version
```

---

# 418. No end-user/person analytics partition.

---

# 419. Hard rule.

---

# 420. Crate Layout

Recommended:

```text
crates/
├── siar-dev-cli-core/
├── siar-dev-project/
├── siar-dev-emulator/
├── siar-dev-fixtures/
├── siar-contract-test/
├── siar-dev-compat/
├── siar-dev-ci/
├── siar-dev-certification/
├── siar-dev-toolchain-observability/
└── siar-dev-toolchain-testkit/
```

---

# 421. `siar-dev-cli-core`

Owns:

```text
CLI command model
environment context
exit codes
DeveloperToolchainError
```

---

# 422. `siar-dev-project`

Project manifest/lockfile/workspace automation.

---

# 423. `siar-dev-emulator`

Local synthetic service emulator.

---

# 424. `siar-dev-fixtures`

Signed synthetic fixture packages.

---

# 425. `siar-contract-test`

Contract baseline/assertions/conformance.

---

# 426. `siar-dev-compat`

Compatibility registry validation/explain.

---

# 427. `siar-dev-ci`

Provider-neutral CI/workload identity/evidence.

---

# 428. `siar-dev-certification`

Certification package preparation/preview.

---

# 429. `siar-dev-toolchain-observability`

Aggregate toolchain health only.

---

# 430. `siar-dev-toolchain-testkit`

CLI/emulator/CI/privacy/formal tests.

---

# 431. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Local, sandbox, certification, and production environments are explicit typed contexts and cannot be selected implicitly from credential contents, filesystem state, or previous commands.
2. Sandbox, emulator, and CI credentials can never authorize production endpoints; production credentials are never stored in project manifests, lockfiles, caches, or generated code.
3. The local emulator uses synthetic identities/data/keys and cannot load production user data, production secrets, production signing keys, or private message history as a convenience.
4. Contract tests verify stable SDK/API/event/error/capability semantics and certification baselines, while snapshots and generated expectations can never be bulk-accepted blindly as proof of compatibility.
5. CI uses scoped workload identity and short-lived credentials rather than developer tokens or long-lived shared secrets wherever possible.
6. CLI automation can validate, test, prepare certification, provision sandbox resources, and publish evidence but cannot grant itself certification, production eligibility, or privileged scopes.
7. Offline development remains useful through signed cached docs, fixtures, SDKs, and compatibility registries, but expired/stale offline state can never cause permissive production-sensitive decisions.
8. Emulator failure/network/time/randomness controls are bounded, explicit, deterministic where possible, and cannot become arbitrary code-injection or uncontrolled production fault-injection mechanisms.
9. Developer toolchain logs, diagnostics, crash reports, telemetry, and support bundles redact secrets/private content and never upload shell history, source code, local paths, user behavior, or developer productivity data by default.
10. Generated project/code/CI templates have explicit provenance, avoid hidden network calls and embedded secrets, and are reviewed/tested like other SDK surfaces.
11. Toolchain artifacts—CLI, emulator, fixture bundles, generators, offline bundles—are versioned, signed/provenance-linked, and compatible with the SDK/API/protocol baselines they claim to support.
12. Developer tooling integrates with the portal, SDK, compatibility registry, integration certification, assurance archive, product lifecycle, release governance, secret management, CI, support, and audit without creating a side channel around SIAR's security, privacy, anonymity, or tenant isolation.
```

---

# 432. Initial Production Scope

Implement first:

```text
Rust developer CLI
explicit environment context
project init/validate/diff
RON project manifest
toolchain lockfile
secure CLI login/token storage
local synthetic emulator
deterministic fixtures/seeds
offline/reconnect/network failure profiles
local webhook verifier
contract test framework
compatibility check/explain
SDK/binding provenance checks
provider-neutral CI plan
OIDC workload identity support
ephemeral CI credentials
CI evidence publication
certification prepare/submit/status
doctor/status commands
redacted diagnostics
signed CLI/emulator/fixture packages
offline docs/compat bundles
privacy-safe toolchain metrics
developer-toolchain testkit
```

Then add:

```text
multi-device emulator
mixnet/protocol simulation
interactive contract debugger
CI adapters for major providers
project workspace orchestration
air-gapped certification bundle
automated migration rehearsal
formal environment/credential-state verification
```

---

# 433. Definition of Done

Part 132 is complete when:

- CLI environment selection is explicit;
- local/sandbox/prod credentials are strongly separated;
- emulator uses only synthetic data/keys;
- project manifests and lockfiles are versioned;
- contract testing is stable and certification-aware;
- compatibility checks use signed registry state;
- CI uses short-lived workload identity;
- CLI can prepare but never self-approve certification;
- offline development remains safe/restrictive;
- diagnostics redact secrets/private content;
- toolchain packages have signing/provenance;
- no source-code/user/developer surveillance telemetry is created;
- CLI/emulator/contract/CI/privacy/fuzz/formal tests are specified.

---

# 434. Final Architecture

```text
                    DEVELOPER
                        │
                        ▼
                    SIAR CLI
                        │
             ┌──────────┼──────────┐
             │          │          │
          PROJECT    EMULATOR     CI
             │          │          │
             └──────────┼──────────┘
                        ▼
                CONTRACT / COMPAT
                        │
                        ▼
                CERTIFICATION PREP
                        │
                        ▼
                GOVERNED PROMOTION
```

Developer-toolchain safety model:

```text
explicit environments
+
synthetic local emulation
+
contract-driven validation
+
signed compatibility state
+
ephemeral CI identity
+
certification separation
+
offline-safe tooling
+
privacy-safe diagnostics
```

not:

```text
store production keys in config files, clone production data locally, let CI reuse developer tokens, and allow a CLI command to bypass certification
```

---

# 435. Final Principle

Developer automation should reduce repetitive effort without reducing trust boundaries.

The correct model is:

```text
bootstrap projects reproducibly
+
emulate locally with synthetic state
+
test contracts deterministically
+
validate compatibility explicitly
+
use ephemeral CI identity
+
prepare certification automatically
+
keep approval separate
+
support offline development safely
+
never let automation become a production privilege bypass
```

This architecture gives SIAR a privacy-preserving developer-toolchain foundation for CLI workflows, project automation, local integration emulation, contract testing, CI integration, compatibility validation, certification preparation, deterministic fixtures, and offline development while preserving the anonymity, local-first, least-authority, SDK, developer-portal, compatibility, and anti-surveillance guarantees established across Parts 34–131.
