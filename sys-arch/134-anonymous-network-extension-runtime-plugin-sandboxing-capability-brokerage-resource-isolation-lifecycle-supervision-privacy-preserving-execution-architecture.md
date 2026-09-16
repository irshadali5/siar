# Core System Architecture Part 134 — Anonymous Network Extension Runtime, Plugin Sandboxing, Capability Brokerage, Resource Isolation, Lifecycle Supervision & Privacy-Preserving Execution Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 134  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 21, 24, 68, 73, 79–81, 122–133

**Primary purpose:** define SIAR's extension-runtime architecture for plugin sandboxing, WASM execution, native extension isolation, capability brokerage, mediated file/network/device access, resource quotas, lifecycle supervision, crash containment, background execution, update/revocation enforcement, and privacy-preserving third-party execution.

---

# 1. Purpose

An extension marketplace is safe only if runtime execution remains constrained after installation.

A package can be properly signed and still:

```text
consume too much CPU
leak metadata
open unexpected network connections
read unauthorized files
retain stale privileges after revocation
crash the host
```

The governing principle is:

> **SIAR extensions execute inside explicit trust and resource boundaries, obtain authority only through capability brokers, and can be paused, revoked, updated, or terminated without gaining ambient access to the host, user data, devices, or network.**

---

# 2. Architectural Position

```text
                   INSTALLED EXTENSION
                           │
                           ▼
                    RUNTIME MANAGER
                           │
              ┌────────────┼────────────┐
              │            │            │
           WASM HOST    NATIVE HOST   BROKERS
              │            │            │
              └────────────┼────────────┘
                           ▼
                    CAPABILITY GATE
                           │
                           ▼
                  RESOURCE GOVERNOR
                           │
                           ▼
                   SUPERVISED EXECUTION
```

---

# 3. Core Separation

Keep distinct:

```text
package permission request
approved runtime grant
host resource budget
runtime identity
process identity
user consent
device permission
network policy
```

---

# 4. Non-Goals

Part 134 does not create:

```text
ambient filesystem access
ambient sockets
shared secret memory
in-process native code by default
extension-controlled policy engine
extension-to-extension covert channels
```

---

# 5. Runtime Identity

```rust
pub struct ExtensionRuntimeId(pub [u8; 16]);
```

---

# 6. Runtime Instance

```rust
pub struct ExtensionRuntimeInstance {
    pub runtime_id: ExtensionRuntimeId,
    pub extension: ExtensionId,
    pub package: ExtensionPackageId,
    pub class: ExtensionClass,
    pub state: ExtensionRuntimeState,
}
```

---

# 7. Runtime State

```rust
pub enum ExtensionRuntimeState {
    Provisioning,
    Starting,
    Running,
    Paused,
    Throttled,
    Quarantined,
    Stopping,
    Stopped,
    Failed,
    Revoked,
}
```

---

# 8. No Provisioning→Running Direct

Hard rule.

---

# 9. Runtime Class

```rust
pub enum ExtensionRuntimeClass {
    Wasm,
    NativeProcess,
}
```

---

# 10. WASM Preferred

Hard rule.

---

# 11. Native Process

Used only where WASM cannot satisfy required capability/performance.

---

# 12. Hard rule.

---

# 13. No Native In-Process Plugins By Default

Hard rule.

---

# 14. Runtime Manager

```rust
pub trait ExtensionRuntimeManager {
    async fn start(
        &self,
        extension: ExtensionId,
    ) -> Result<ExtensionRuntimeId, ExtensionRuntimeError>;

    async fn stop(
        &self,
        runtime: ExtensionRuntimeId,
    ) -> Result<(), ExtensionRuntimeError>;
}
```

---

# 15. Lifecycle Supervision

All extension instances supervised by host.

---

# 16. Hard rule.

---

# 17. Capability Broker

Extensions never call internal services directly.

---

# 18. Broker Interface

```rust
pub trait CapabilityBroker {
    async fn invoke(
        &self,
        runtime: ExtensionRuntimeId,
        request: CapabilityRequest,
    ) -> Result<CapabilityResponse, CapabilityBrokerError>;
}
```

---

# 19. Capability Request

```rust
pub struct CapabilityRequest {
    pub capability: SdkCapability,
    pub operation: CapabilityOperation,
    pub scope: SdkCapabilityScope,
}
```

---

# 20. Runtime Grant Check

Every request verifies active runtime grant.

---

# 21. Hard rule.

---

# 22. No Grant Caching Beyond Revocation Epoch

Hard rule.

---

# 23. Grant Epoch

```rust
pub struct ExtensionGrantEpoch(pub u64);
```

---

# 24. Increment On:

```text
grant change
scope change
revocation
security policy update
```

---

# 25. Hard rule.

---

# 26. Capability Token

Opaque short-lived token.

```rust
pub struct ExtensionCapabilityToken {
    pub runtime: ExtensionRuntimeId,
    pub grant_epoch: ExtensionGrantEpoch,
    pub expires_at: Timestamp,
}
```

---

# 27. No Long-Lived Bearer Capability

Hard rule.

---

# 28. Broker Classes

Recommended:

```text
messaging broker
storage broker
file broker
network broker
device broker
notification broker
diagnostic broker
background-job broker
```

---

# 29. Hard rule.

---

# 30. Messaging Broker

Exposes SDK-level operations only.

---

# 31. Hard rule.

---

# 32. Message Content Access

Separate capability.

---

# 33. Hard rule.

---

# 34. Conversation Scope

Per conversation or authorized aggregate scope.

---

# 35. Hard rule.

---

# 36. Contact Enumeration

No unrestricted enumeration.

---

# 37. Hard rule.

---

# 38. Presence Broker

Returns authorized projection only.

---

# 39. Hard rule.

---

# 40. Storage Broker

Each extension receives namespaced storage.

---

# 41. Extension Storage Namespace

```rust
pub struct ExtensionStorageNamespace {
    pub extension: ExtensionId,
    pub tenant: Option<TenantId>,
}
```

---

# 42. Hard rule.

---

# 43. No Cross-Extension Storage Access

Hard rule.

---

# 44. Storage Quota

```rust
pub struct ExtensionStorageQuota {
    pub max_bytes: u64,
    pub max_objects: u64,
}
```

---

# 45. Hard rule.

---

# 46. Storage Operations

```rust
pub trait ExtensionStorageBroker {
    async fn get(&self, key: ExtensionStorageKey) -> Result<Option<Vec<u8>>, BrokerError>;
    async fn put(&self, key: ExtensionStorageKey, value: &[u8]) -> Result<(), BrokerError>;
    async fn delete(&self, key: ExtensionStorageKey) -> Result<(), BrokerError>;
}
```

---

# 47. Values Bounded

Hard rule.

---

# 48. File Broker

Extensions use handles, not arbitrary paths.

---

# 49. File Handle

```rust
pub struct BrokeredFileHandle(pub u64);
```

---

# 50. File Permission

```rust
pub enum BrokeredFilePermission {
    Read,
    Write,
    ReadWrite,
}
```

---

# 51. User/Admin Approval Where Required

Hard rule.

---

# 52. No Raw Home Directory Access

Hard rule.

---

# 53. Temporary File Access

Time-bounded.

---

# 54. Hard rule.

---

# 55. Network Broker

No raw unrestricted socket access by default.

---

# 56. Network Policy

```rust
pub struct ExtensionNetworkPolicy {
    pub allowed_origins: BTreeSet<NetworkOrigin>,
    pub allow_local_network: bool,
    pub allow_anonymous_network: bool,
}
```

---

# 57. Hard rule.

---

# 58. Destination Allowlist

Exact or policy-scoped.

---

# 59. Hard rule.

---

# 60. DNS Mediation

Broker performs DNS if external networking allowed.

---

# 61. Hard rule.

---

# 62. No Direct DNS Exfiltration

Hard rule.

---

# 63. Local Network

Separate capability.

---

# 64. Hard rule.

---

# 65. Anonymous Network Access

Separate broker path.

---

# 66. Hard rule.

---

# 67. Extension Cannot Request Weaker Privacy Route

Hard rule.

---

# 68. Maximum Anonymity

Network broker may deny direct external access entirely.

---

# 69. Hard rule.

---

# 70. Device Broker

Potential classes:

```text
camera
microphone
clipboard
notifications
bluetooth
NFC
```

---

# 71. Hard rule.

---

# 72. Device Permission Triple Gate

Require:

```text
extension capability
platform permission
user/admin approval
```

---

# 73. Hard rule.

---

# 74. Camera/Mic Streams

Brokered stream handles.

---

# 75. Hard rule.

---

# 76. No Raw Device Handle Exposure

Hard rule.

---

# 77. Clipboard

Read/write separate capabilities.

---

# 78. Hard rule.

---

# 79. Notification Broker

Extension may request user-visible notification.

---

# 80. Hard rule.

---

# 81. Notification Content

Must follow privacy classification.

---

# 82. Hard rule.

---

# 83. Diagnostic Broker

Limited extension-scoped diagnostics.

---

# 84. Hard rule.

---

# 85. No Host-Wide Logs

Hard rule.

---

# 86. Resource Isolation

Track:

```text
CPU
memory
storage
network
open handles
background jobs
event queue depth
```

---

# 87. Resource Budget

```rust
pub struct ExtensionResourceBudget {
    pub cpu_millis_per_minute: u64,
    pub max_memory_bytes: u64,
    pub max_storage_bytes: u64,
    pub max_network_bytes_per_hour: u64,
    pub max_open_handles: u32,
    pub max_background_jobs: u32,
}
```

---

# 88. Hard rule.

---

# 89. Resource Governor

```rust
pub trait ExtensionResourceGovernor {
    fn usage(
        &self,
        runtime: ExtensionRuntimeId,
    ) -> Result<ExtensionResourceUsage, ExtensionRuntimeError>;
}
```

---

# 90. Hard rule.

---

# 91. Budget Enforcement

Possible actions:

```text
warn
throttle
pause
terminate
```

---

# 92. Hard rule.

---

# 93. No Host Starvation

Hard rule.

---

# 94. CPU Isolation

WASM fuel/epoch interruption or process quota.

---

# 95. Hard rule.

---

# 96. Memory Isolation

WASM memory limits / OS process limits.

---

# 97. Hard rule.

---

# 98. Storage Isolation

Namespace + quota.

---

# 99. Hard rule.

---

# 100. Network Isolation

Broker quota + OS/network sandbox.

---

# 101. Hard rule.

---

# 102. Background Execution

Brokered scheduler.

---

# 103. No Infinite Background Loop

Hard rule.

---

# 104. Background Job Definition

```rust
pub struct ExtensionBackgroundJob {
    pub job_id: ExtensionBackgroundJobId,
    pub extension: ExtensionId,
    pub schedule: ExtensionJobSchedule,
    pub budget: ExtensionJobBudget,
}
```

---

# 105. Job Schedule

```rust
pub enum ExtensionJobSchedule {
    OneShot(Timestamp),
    Periodic(Duration),
    EventDriven(ExtensionEventTrigger),
}
```

---

# 106. Minimum Period

Policy-defined.

---

# 107. Hard rule.

---

# 108. Job Budget

```rust
pub struct ExtensionJobBudget {
    pub max_runtime: Duration,
    pub max_memory_bytes: u64,
    pub max_network_bytes: u64,
}
```

---

# 109. Hard rule.

---

# 110. Mobile Background Policy

Respect Android lifecycle/battery limits.

---

# 111. Hard rule.

---

# 112. Desktop Daemon Policy

Idle CPU near zero.

---

# 113. Hard rule.

---

# 114. Event Delivery

Extensions subscribe only to authorized event classes.

---

# 115. Event Subscription

```rust
pub struct ExtensionEventSubscription {
    pub runtime: ExtensionRuntimeId,
    pub event_class: ExtensionEventClass,
    pub scope: SdkCapabilityScope,
}
```

---

# 116. Hard rule.

---

# 117. Event Queue

Bounded.

---

# 118. Hard rule.

---

# 119. Slow Consumer

Policy:

```text
coalesce
drop noncritical
pause
resync
```

---

# 120. Hard rule.

---

# 121. No Unbounded Event Replay

Hard rule.

---

# 122. Event Snapshot

Use snapshot + delta where appropriate.

---

# 123. Hard rule.

---

# 124. Extension-to-Extension Communication

Not direct by default.

---

# 125. Hard rule.

---

# 126. Brokered Inter-Extension Channel

Only explicit shared contract.

---

# 127. Hard rule.

---

# 128. Shared Capability

Host mediates.

---

# 129. Hard rule.

---

# 130. Covert Channel Reduction

Limit shared timing/storage/network channels where practical.

---

# 131. Hard rule.

---

# 132. WASM Runtime

Recommended properties:

```text
fuel metering
memory limits
WASI restrictions
no ambient preopens
deterministic host calls
```

---

# 133. Hard rule.

---

# 134. WASI

Use constrained custom WASI profile.

---

# 135. Hard rule.

---

# 136. No Default Filesystem Preopen

Hard rule.

---

# 137. No Default Network Socket

Hard rule.

---

# 138. Host Function Registry

```rust
pub struct ExtensionHostFunctionRegistry {
    pub version: HostFunctionRegistryVersion,
    pub functions: BTreeMap<HostFunctionId, HostFunctionDescriptor>,
}
```

---

# 139. Versioned

Hard rule.

---

# 140. Host Function Descriptor

```rust
pub struct HostFunctionDescriptor {
    pub required_capability: Option<SdkCapability>,
    pub max_payload_bytes: u64,
    pub blocking: bool,
}
```

---

# 141. Hard rule.

---

# 142. ABI Stability

WASM host ABI versioned.

---

# 143. Hard rule.

---

# 144. Host ABI Compatibility

Part 129 registry.

---

# 145. Hard rule.

---

# 146. Native Runtime

Out-of-process.

---

# 147. Hard rule.

---

# 148. Native IPC

Use versioned Postcard over local authenticated channel.

---

# 149. Hard rule.

---

# 150. Native Process Sandbox

OS-specific:

```text
Linux namespaces/seccomp/cgroups
Windows job objects/AppContainer where available
macOS sandbox/container profile where available
Android app/process restrictions where feasible
```

---

# 151. Hard rule.

---

# 152. Native Process Identity

Dedicated workload identity.

---

# 153. Hard rule.

---

# 154. No Shared Host UID/Token Where Isolation Available

Hard rule.

---

# 155. Native Filesystem

Empty/private working directory + brokered mounts.

---

# 156. Hard rule.

---

# 157. Native Network

Denied except broker/allowlist.

---

# 158. Hard rule.

---

# 159. Native Syscalls

Restrict dangerous syscalls where platform supports.

---

# 160. Hard rule.

---

# 161. Native Debug Privileges

Denied.

---

# 162. Hard rule.

---

# 163. Native Child Processes

Denied by default.

---

# 164. Hard rule.

---

# 165. Native Shared Memory

Brokered or denied.

---

# 166. Hard rule.

---

# 167. Extension Startup

Sequence:

```text
verify package
verify revocation state
load manifest
resolve compatibility
load active grants
apply resource policy
initialize sandbox
start runtime
perform readiness handshake
mark Running
```

---

# 168. Hard rule.

---

# 169. Startup Handshake

```rust
pub struct ExtensionRuntimeHello {
    pub extension: ExtensionId,
    pub version: ExtensionVersion,
    pub host_abi: HostAbiVersion,
    pub requested_features: BTreeSet<HostFeatureCode>,
}
```

---

# 170. Hard rule.

---

# 171. Host Response

```rust
pub struct ExtensionRuntimeWelcome {
    pub runtime_id: ExtensionRuntimeId,
    pub granted_features: BTreeSet<HostFeatureCode>,
    pub grant_epoch: ExtensionGrantEpoch,
}
```

---

# 172. Hard rule.

---

# 173. No Running Before Handshake Completes

Hard rule.

---

# 174. Runtime Readiness

Extension may report:

```text
Ready
Degraded
Failed
```

---

# 175. Host validates.

---

# 176. Hard rule.

---

# 177. Health Check

Extension-specific heartbeat optional.

---

# 178. Avoid high-frequency polling.

---

# 179. Hard rule.

---

# 180. Crash Containment

Extension crash does not crash host.

---

# 181. Hard rule.

---

# 182. Crash State

```rust
pub struct ExtensionCrashRecord {
    pub runtime: ExtensionRuntimeId,
    pub at: Timestamp,
    pub class: ExtensionCrashClass,
}
```

---

# 183. Crash Class

```rust
pub enum ExtensionCrashClass {
    Trap,
    Panic,
    ResourceLimit,
    ProtocolViolation,
    HostCallFailure,
    NativeProcessExit,
}
```

---

# 184. Hard rule.

---

# 185. Restart Policy

```rust
pub enum ExtensionRestartPolicy {
    Never,
    OnCrash,
    BoundedRetry,
}
```

---

# 186. Hard rule.

---

# 187. Bounded Restart

```rust
pub struct RestartBudget {
    pub max_restarts: u32,
    pub window: Duration,
}
```

---

# 188. Hard rule.

---

# 189. Crash Loop

Quarantine.

---

# 190. Hard rule.

---

# 191. Quarantine

Extension remains installed but not active.

---

# 192. Hard rule.

---

# 193. User/Admin Visible State

Explain:

```text
why disabled
what data remains
how to update/remove
```

---

# 194. Hard rule.

---

# 195. Runtime Update

Sequence:

```text
download/verify
preflight compatibility
pause old
checkpoint if supported
start new
verify health
switch
retire old
```

---

# 196. Hard rule.

---

# 197. Update Rollback

If new version fails and data format allows.

---

# 198. Hard rule.

---

# 199. Extension State Migration

Versioned.

---

# 200. Hard rule.

---

# 201. Migration Contract

```rust
pub struct ExtensionStateMigration {
    pub from: ExtensionVersion,
    pub to: ExtensionVersion,
    pub reversible: bool,
}
```

---

# 202. Hard rule.

---

# 203. Forward-Only Migration

Requires backup/checkpoint.

---

# 204. Hard rule.

---

# 205. No State Migration Before New Package Verified

Hard rule.

---

# 206. Revocation Enforcement

Part 133 signed revocation list.

---

# 207. Hard rule.

---

# 208. Revocation Check Points

```text
startup
periodic policy refresh
before privileged host call
before update activation
```

---

# 209. Hard rule.

---

# 210. Emergency Disable

Host can revoke runtime immediately.

---

# 211. Hard rule.

---

# 212. Revoked State

Runtime grants invalid.

---

# 213. Hard rule.

---

# 214. Data Preservation On Revocation

Do not delete data automatically.

---

# 215. Hard rule.

---

# 216. Privilege Downgrade

Host may remove capabilities without restart if safe.

---

# 217. Hard rule.

---

# 218. Capability Revocation During Call

In-flight request checks epoch/authorization before commit.

---

# 219. Hard rule.

---

# 220. TOCTOU Defense

Authorization context bound to operation.

---

# 221. Hard rule.

---

# 222. Runtime Policy Refresh

Signed local policy.

---

# 223. Hard rule.

---

# 224. Stale Policy

No permissive fail-open.

---

# 225. Hard rule.

---

# 226. Extension Secrets

Extension may need third-party credentials.

---

# 227. Secret Broker

```rust
pub trait ExtensionSecretBroker {
    async fn use_secret(
        &self,
        runtime: ExtensionRuntimeId,
        secret: SecretRef,
        operation: SecretOperation,
    ) -> Result<SecretOperationResult, BrokerError>;
}
```

---

# 228. Prefer Use-Without-Reveal

Hard rule.

---

# 229. No Secret Plaintext When Avoidable

Hard rule.

---

# 230. User-Owned API Key

Stored in secure key store.

---

# 231. Extension receives reference/capability, not raw secret where possible.

---

# 232. Hard rule.

---

# 233. Secret Scope

Bound to extension/integration.

---

# 234. Hard rule.

---

# 235. Secret Revocation

Immediate.

---

# 236. Hard rule.

---

# 237. Environment Variables

Do not inject secrets into extension env by default.

---

# 238. Hard rule.

---

# 239. Runtime Environment

Minimal.

---

# 240. Hard rule.

---

# 241. Time Access

Brokered/coarsened where privacy-sensitive.

---

# 242. Hard rule.

---

# 243. Randomness

Secure RNG via host function.

---

# 244. Hard rule.

---

# 245. System Information

Limit:

```text
hostname
username
exact hardware
network interfaces
```

---

# 246. Hard rule.

---

# 247. Fingerprinting Reduction

Extensions see normalized environment profile where possible.

---

# 248. Hard rule.

---

# 249. Anonymous Mode Runtime Profile

Further restrict:

```text
exact time
network paths
peer versions
device details
```

---

# 250. Hard rule.

---

# 251. Logging

Extension has own structured log stream.

---

# 252. Hard rule.

---

# 253. Log Quota

Bounded.

---

# 254. Hard rule.

---

# 255. Host Log Redaction

Secrets/private content stripped where possible.

---

# 256. Hard rule.

---

# 257. No Reading Host Logs

Hard rule.

---

# 258. Extension Log Access

Publisher/user/admin depending policy.

---

# 259. Hard rule.

---

# 260. Metrics

Extension-specific:

```text
CPU
memory
host-call count
error class
queue depth
```

---

# 261. Hard rule.

---

# 262. No User Behavior Metrics By Default

Hard rule.

---

# 263. Per-Extension Observability

Host tracks technical health.

---

# 264. Hard rule.

---

# 265. Resource Abuse Detection

Based on:

```text
quota
rate
policy violation
```

not user content.

---

# 266. Hard rule.

---

# 267. Extension Scheduler

Fair-share scheduling.

---

# 268. Hard rule.

---

# 269. Priority Class

```rust
pub enum ExtensionWorkPriority {
    Interactive,
    Background,
    Maintenance,
}
```

---

# 270. No Realtime-Critical By Default

Hard rule.

---

# 271. Host Critical Work Always Wins

Hard rule.

---

# 272. Battery Awareness

Mobile extension work deferred when optional.

---

# 273. Hard rule.

---

# 274. Thermal Awareness

Throttle optional extension work.

---

# 275. Hard rule.

---

# 276. Connectivity Awareness

Extensions can receive coarse connectivity state if allowed.

---

# 277. Hard rule.

---

# 278. No Raw Interface Metadata In Anonymous Mode

Hard rule.

---

# 279. Shutdown

Deterministic.

---

# 280. Shutdown Sequence

```text
stop new events
revoke new host calls
allow bounded graceful flush
checkpoint extension state
terminate
```

---

# 281. Hard rule.

---

# 282. Shutdown Deadline

Bounded.

---

# 283. Hard rule.

---

# 284. Forced Termination

After deadline.

---

# 285. Hard rule.

---

# 286. No Extension Can Block Host Shutdown

Hard rule.

---

# 287. Host Restart Recovery

Extension runtime state rebuilt from durable install/grant state.

---

# 288. Hard rule.

---

# 289. No Runtime Capability Resurrection

Grant epoch revalidated.

---

# 290. Hard rule.

---

# 291. Extension Runtime Registry

```rust
pub trait ExtensionRuntimeRegistry {
    fn active(
        &self,
        extension: ExtensionId,
    ) -> Result<Vec<ExtensionRuntimeInstance>, ExtensionRuntimeError>;
}
```

---

# 292. Hard rule.

---

# 293. Resource Policy Service

```rust
pub trait ExtensionResourcePolicyService {
    fn budget(
        &self,
        extension: ExtensionId,
    ) -> Result<ExtensionResourceBudget, ExtensionRuntimeError>;
}
```

---

# 294. Revocation Service Integration

```rust
pub trait ExtensionRuntimeRevocationService {
    async fn revoke_runtime(
        &self,
        runtime: ExtensionRuntimeId,
    ) -> Result<(), ExtensionRuntimeError>;
}
```

---

# 295. No Raw Host-Admin API

Hard rule.

---

# 296. Runtime Error Taxonomy

```rust
pub enum ExtensionRuntimeError {
    ExtensionUnknown,
    PackageRevoked,
    CompatibilityMismatch,
    CapabilityDenied,
    ResourceLimitExceeded,
    SandboxInitializationFailed,
    RuntimeHandshakeFailed,
    RuntimeCrashed,
    RuntimeQuarantined,
    PolicyStale,
    Unauthorized,
    Internal,
}
```

---

# 297. Observability

Safe metrics:

```text
runtime count
crash class
resource-throttle count
host-call error class
quarantine count
```

---

# 298. Forbidden:

```text
private message content
user social graph
extension-derived behavioral profile
```

---

# 299. Hard rule.

---

# 300. Extension Runtime SLOs

Examples:

```text
revocation enforcement within target
crash containment success
runtime startup within target
resource overage enforcement within target
```

---

# 301. Security SLO

```text
0 revoked runtime retains privileged capability
0 extension escapes sandbox boundary
0 native extension gets undeclared socket/file/device access
```

---

# 302. Privacy SLO

```text
0 extension receives prohibited peer metadata in anonymous mode
0 raw host logs exposed
0 secret injected in extension environment by default
```

---

# 303. Failure Modes

```text
sandbox escape
resource starvation
revocation race
crash loop
secret leakage
```

---

# 304. Sandbox Escape

Quarantine + security incident.

---

# 305. Hard rule.

---

# 306. Resource Starvation

Throttle/terminate offending extension.

---

# 307. Hard rule.

---

# 308. Revocation Race

Grant epoch + commit-time check.

---

# 309. Hard rule.

---

# 310. Crash Loop

Quarantine.

---

# 311. Hard rule.

---

# 312. Secret Leakage

Revoke secret + extension + investigate.

---

# 313. Hard rule.

---

# 314. Testing

Need extension-runtime testkit.

---

# 315. Test Scenarios

```text
WASM normal extension
native isolated extension
scope revocation while running
CPU exhaustion
background job overrun
```

---

# 316. Sandbox Test

Extension cannot access undeclared filesystem path.

---

# 317. Network Test

Extension cannot open undeclared destination.

---

# 318. Device Test

Camera/mic denied without triple gate.

---

# 319. Resource Test

CPU/memory limit enforced.

---

# 320. Queue Test

Slow event consumer bounded.

---

# 321. Crash Test

Extension crash does not crash host.

---

# 322. Crash Loop Test

Runtime quarantined after retry budget.

---

# 323. Revocation Test

Running extension loses capability immediately after grant epoch update.

---

# 324. TOCTOU Test

Revoked operation cannot commit after authorization invalidation.

---

# 325. Update Test

New package activated only after verification.

---

# 326. Rollback Test

Failed update returns to prior version where migration allows.

---

# 327. Secret Test

Extension cannot read secret plaintext when broker supports use-without-reveal.

---

# 328. Anonymous Mode Test

Host hides raw peer/network/interface metadata.

---

# 329. Shutdown Test

Extension cannot block application shutdown.

---

# 330. Fuzzing

Fuzz:

```text
host-call requests
runtime handshake
capability envelopes
resource policy
state migration manifests
```

---

# 331. Property Tests

Properties:

```text
runtime can never invoke capability absent from active grant
revoked grant epoch can never authorize a committing operation
extension resource usage can never exceed hard quota without throttle/termination
extension can never obtain direct filesystem/network/device access unless broker policy grants it
```

---

# 332. Formal Verification Targets

Strong candidates:

```text
runtime lifecycle
capability grant/revocation
update activation
background-job supervision
```

---

# 333. Kani Candidate

grant/resource/state transition invariants.

---

# 334. TLA+ Candidate

install → start → grant → run → update/revoke → stop/quarantine.

---

# 335. Loom Candidate

concurrent host call + grant revocation + runtime shutdown.

---

# 336. Performance

Extension runtime must not degrade core messaging.

---

# 337. Hard rule.

---

# 338. Host-Call Budget

Per extension.

---

# 339. Hard rule.

---

# 340. Batching

Allowed where semantically safe.

---

# 341. Hard rule.

---

# 342. Zero-Copy

Only with explicit lifetime/isolation guarantees.

---

# 343. Hard rule.

---

# 344. IPC Serialization

Native plugin IPC:

```text
Postcard
```

Preferred.

---

# 345. Hard rule.

---

# 346. JSON

Only external interop if needed.

---

# 347. Hard rule.

---

# 348. Runtime Storage

Separate:

```text
install records
active grants
runtime state
resource policy
background-job metadata
quarantine records
```

---

# 349. Secrets separate.

---

# 350. Hard rule.

---

# 351. No User-Behavior Warehouse

Hard rule.

---

# 352. Partitioning

By:

```text
extension
runtime
tenant
platform
```

---

# 353. No person/user analytics partition.

---

# 354. Hard rule.

---

# 355. Crate Layout

Recommended:

```text
crates/
├── siar-extension-runtime-core/
├── siar-extension-host-wasm/
├── siar-extension-host-native/
├── siar-capability-broker/
├── siar-extension-storage-broker/
├── siar-extension-network-broker/
├── siar-extension-device-broker/
├── siar-extension-resource-governor/
├── siar-extension-supervisor/
├── siar-extension-runtime-observability/
└── siar-extension-runtime-testkit/
```

---

# 356. `siar-extension-runtime-core`

Owns:

```text
ExtensionRuntimeId
ExtensionRuntimeState
ExtensionRuntimeError
grant epochs
runtime contracts
```

---

# 357. `siar-extension-host-wasm`

WASM runtime/fuel/memory/WASI restrictions.

---

# 358. `siar-extension-host-native`

Out-of-process native isolation.

---

# 359. `siar-capability-broker`

Capability validation/dispatch/attenuation.

---

# 360. `siar-extension-storage-broker`

Namespaced storage/quota.

---

# 361. `siar-extension-network-broker`

Allowlisted mediated networking.

---

# 362. `siar-extension-device-broker`

Camera/mic/clipboard/etc. mediation.

---

# 363. `siar-extension-resource-governor`

CPU/memory/network/job quotas.

---

# 364. `siar-extension-supervisor`

Lifecycle/crash/restart/quarantine/update/shutdown.

---

# 365. `siar-extension-runtime-observability`

Aggregate runtime health only.

---

# 366. `siar-extension-runtime-testkit`

sandbox/revocation/resource/privacy tests.

---

# 367. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Every extension executes in a supervised runtime with an explicit runtime identity, package digest, compatibility state, resource budget, and active grant epoch; no extension gains ambient authority from merely being installed.
2. WASM or equivalent sandboxing is the preferred third-party runtime, while native extensions run out-of-process with OS isolation, restricted syscalls, private working directories, and denied raw networking/filesystem/device access by default.
3. All sensitive actions pass through typed capability brokers, and runtime authority is derived from active scoped grants—not package requests, publisher status, certification, popularity, or cached historical permissions.
4. Capability revocation and policy changes are epoch-bound and revalidated before privileged operation commit so running extensions cannot retain stale authority through TOCTOU races.
5. Files, network, devices, secrets, notifications, diagnostics, background jobs, and extension-to-extension communication are brokered individually; no general-purpose host escape or "all access" capability exists.
6. Resource usage is explicitly bounded across CPU, memory, storage, network, handles, event queues, and background work; extension overload can be throttled, paused, quarantined, or terminated without starving core SIAR services.
7. Extension crashes, traps, protocol violations, and crash loops are contained and supervised; no extension may crash the host, block shutdown, or restart indefinitely without quarantine.
8. Revoked or emergency-disabled packages immediately lose runtime authority while extension/user data is preserved for explicit export/delete/remediation rather than silently destroyed.
9. Secrets are brokered by reference/use-without-reveal where possible and are never injected into generic environment variables, logs, extension storage, or shared process memory by default.
10. Anonymous/private modes expose a reduced normalized runtime environment and prohibit extensions from acquiring raw peer identities, route/network-interface details, detailed client versions, or weaker network paths than the platform permits.
11. Runtime metrics/logs are technical, bounded, extension-scoped, and privacy-safe; they cannot become host-wide logging access, private-content monitoring, behavioral user profiling, or developer/publisher scoring.
12. The extension runtime integrates with marketplace review/certification, SDK capabilities, authorization, secrets, compatibility, product lifecycle, supply-chain security, revocation, audit, risk/PIR, updates, and privacy controls without creating a side channel around SIAR's security, privacy, anonymity, tenant isolation, or local-first guarantees.
```

---

# 368. Initial Production Scope

Implement first:

```text
typed runtime identity/state machine
WASM-first host
fuel/memory limits
restricted WASI profile
versioned host ABI
capability broker
message/storage/file/network brokers
device broker skeleton
extension storage namespaces/quotas
resource governor
bounded event subscriptions
background job broker
crash supervision/restart budget/quarantine
grant epoch/revocation enforcement
secret broker references
signed policy refresh
update activation/rollback skeleton
privacy-reduced anonymous-mode profile
structured extension logs
runtime testkit
```

Then add:

```text
native isolated runtime
OS-specific sandbox hardening
brokered shared-memory/stream optimizations
advanced per-extension QoS
cross-device runtime-state sync
formal capability-revocation verification
fine-grained background/mobile scheduling
```

---

# 369. Definition of Done

Part 134 is complete when:

- extension runtimes have stable identities/states;
- WASM is the default isolated runtime;
- native plugins are out-of-process;
- all sensitive operations use brokers;
- active grant epochs drive authority;
- resource budgets are enforced;
- queues/background jobs are bounded;
- crashes are contained/quarantined;
- updates/revocations cannot retain stale privileges;
- secrets are use-without-reveal where possible;
- anonymous mode reduces fingerprinting/metadata exposure;
- extension shutdown cannot block host shutdown;
- no extension or runtime telemetry creates user/developer surveillance;
- sandbox/revocation/resource/privacy/fuzz/formal tests are specified.

---

# 370. Final Architecture

```text
                 INSTALLED EXTENSION
                         │
                         ▼
                  RUNTIME SUPERVISOR
                         │
             ┌───────────┼───────────┐
             │           │           │
          WASM HOST   NATIVE HOST   POLICY
             │           │           │
             └───────────┼───────────┘
                         ▼
                 CAPABILITY BROKERS
                         │
             ┌───────────┼───────────┐
             │           │           │
          STORAGE      NETWORK      DEVICES
             │           │           │
             └───────────┼───────────┘
                         ▼
                RESOURCE GOVERNOR
                         │
                         ▼
                  SUPERVISED RUN
                         │
                         ▼
             PAUSE / UPDATE / REVOKE
```

Extension-runtime safety model:

```text
WASM-first isolation
+
out-of-process native fallback
+
capability brokerage
+
grant epochs
+
resource quotas
+
crash supervision
+
brokered secrets/devices/network/files
+
privacy-reduced runtime profiles
```

not:

```text
load third-party native code into the host, give it filesystem and sockets, let it retain old privileges forever, and hope marketplace review is enough
```

---

# 371. Final Principle

Marketplace trust is only the beginning; runtime trust must be continuously enforced.

The correct model is:

```text
verify the package
+
start it inside a sandbox
+
grant only explicit capabilities
+
mediate every sensitive resource
+
bound CPU/memory/storage/network
+
supervise crashes and background work
+
revalidate grants on every privileged path
+
revoke safely
+
never expose more metadata than the active privacy mode permits
```

This architecture gives SIAR a privacy-preserving extension execution foundation for WASM/native sandboxing, capability brokerage, mediated resources, lifecycle supervision, resource isolation, secret handling, update/revocation enforcement, and failure containment while preserving the anonymity, local-first, least-authority, marketplace, SDK, compatibility, and anti-surveillance guarantees established across Parts 34–133.
