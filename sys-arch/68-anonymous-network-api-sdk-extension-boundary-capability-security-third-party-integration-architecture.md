# Core System Architecture Part 68 — Anonymous Network API, SDK, Extension Boundary, Capability Security & Third-Party Integration Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 68  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 19, 21–24, 34–67  

**Primary purpose:** define SIAR's public/internal APIs, SDKs, FFI/WASM/plugin boundaries, capability-security model, third-party integration broker, versioning and compatibility policy, sandboxing, data-minimization contracts, extension lifecycle, and release/security controls so integrations can extend the platform without gaining ambient authority over identity, storage, routing, cryptographic, or privacy-sensitive internals.

---

# 1. Purpose

Extensibility is valuable.

It is also one of the easiest ways to break privacy.

A plugin, SDK, automation, or external service may accidentally gain access to:

```text
raw contact graphs
message history
transport identities
mailbox capabilities
device keys
provider credentials
routing metadata
```

The governing principle is:

> **Third-party code receives explicit capabilities for specific actions and data projections, never ambient access to SIAR internals.**

---

# 2. Architectural Position

```text
Third-Party App / Plugin / SDK Consumer
                │
                ▼
        Capability Boundary
                │
                ▼
         Public API Broker
                │
                ▼
      Application Use-Case Layer
                │
                ▼
            SIAR Core
```

Native integration path:

```text
External Language
      │
      ▼
Stable C ABI / IPC
      │
      ▼
Capability-Gated Rust Adapter
      │
      ▼
SIAR Core
```

---

# 3. Core Separation

Keep distinct:

```text
public API
internal API
operator API
plugin API
FFI API
wire protocol
storage schema
```

---

# 4. Non-Goals

Part 68 does not expose:

```text
raw database handles
private keys
arbitrary SQL
raw transport sockets
global user identity maps
internal service objects
operator/admin authority
```

---

# 5. API Classes

```rust
pub enum ApiClass {
    PublicSdk,
    LocalApp,
    Plugin,
    Ffi,
    Operator,
    Federation,
    Internal,
}
```

---

# 6. Public SDK

Stable product-facing integration layer.

---

# 7. Local App API

Desktop/mobile application interface.

---

# 8. Plugin API

Sandboxed extension surface.

---

# 9. FFI

C ABI/native language interoperability.

---

# 10. Operator API

Part 61.

Separate and privileged.

---

# 11. Federation API

Part 58.

Domain-to-domain only.

---

# 12. Internal API

Not compatibility-guaranteed externally.

---

# 13. Hard Rule

Public integrations never depend directly on internal service/storage types.

---

# 14. API Surface Principle

Expose:

```text
intent
```

not:

```text
implementation
```

---

# 15. Good API Example

```rust
send_message(...)
```

---

# 16. Bad API Example

```rust
insert_message_row(...)
```

---

# 17. Capability Security

Core authorization model.

---

# 18. Capability

```rust
pub struct CapabilityToken<TScope> {
    pub id: CapabilityId,
    pub scope: TScope,
    pub expires_at: Option<Timestamp>,
}
```

---

# 19. Capability Properties

Capabilities should be:

```text
explicit
scoped
revocable
auditable
short-lived where possible
```

---

# 20. No Ambient Authority

Hard rule.

---

# 21. Capability Scope

```rust
pub enum ExtensionCapability {
    SendMessages,
    ReadConversationMetadata,
    ReadSelectedMessages,
    CreateGroup,
    ReadContacts,
    WriteLocalFiles,
    UseExternalNetwork,
    RegisterCommand,
    SubscribeEvents,
}
```

---

# 22. Capability Granularity

Prefer:

```text
specific operation
specific data class
specific scope
```

---

# 23. Example

Bad:

```text
ReadEverything
```

Good:

```text
ReadSelectedMessages(conversation)
```

---

# 24. Capability Scope Type

```rust
pub struct ScopedCapability {
    pub capability: ExtensionCapability,
    pub resource: CapabilityResource,
}
```

---

# 25. Resource Scope

```rust
pub enum CapabilityResource {
    GlobalNone,
    Conversation(ConversationId),
    Group(GroupId),
    Contact(ContactId),
    LocalDirectory(LocalDirectoryId),
}
```

---

# 26. GlobalNone

Means action not tied to user object.

---

# 27. No Global User Scope By Default

Hard rule.

---

# 28. Capability Issuance

User/application policy grants capability.

---

# 29. Consent Integration

Part 56.

---

# 30. Managed Policy Integration

Enterprise can restrict extensions.

---

# 31. Hard Rule

Managed policy may disable plugin capabilities, not weaken security floors.

---

# 32. Capability Lifetime

```rust
pub enum CapabilityLifetime {
    OneShot,
    Session,
    Persistent,
}
```

---

# 33. OneShot

Best for sensitive actions.

---

# 34. Session

Useful for UI/plugin session.

---

# 35. Persistent

Requires explicit user approval.

---

# 36. Capability Revocation

Immediate.

---

# 37. Revocation Registry

```rust
pub trait CapabilityRevocationRegistry {
    fn revoke(
        &self,
        id: CapabilityId,
    ) -> Result<(), CapabilityError>;
}
```

---

# 38. Revocation Must Survive Restart

For persistent capabilities.

---

# 39. Capability Delegation

Default:

```text
forbidden
```

---

# 40. If Supported

Must be narrower than parent.

---

# 41. Attenuation

```rust
pub trait CapabilityAttenuator {
    fn attenuate(
        &self,
        parent: &ScopedCapability,
        child_scope: CapabilityResource,
    ) -> Result<ScopedCapability, CapabilityError>;
}
```

---

# 42. No Privilege Amplification

Hard rule.

---

# 43. SDK Design

Prefer language-neutral concepts.

---

# 44. Rust SDK

First-class.

---

# 45. Other Languages

Through:

```text
C ABI
IPC
generated bindings
```

---

# 46. SDK Types

Stable DTOs.

---

# 47. Do Not Expose Internal Structs

Hard rule.

---

# 48. DTO Principle

```text
small
versioned
purpose-specific
```

---

# 49. Example

```rust
pub struct MessageSummaryDto {
    pub message_id: PublicMessageRef,
    pub sender_label: String,
    pub preview: Option<String>,
}
```

---

# 50. Public Reference

Opaque.

---

# 51. No Raw Database Primary Key

Hard rule.

---

# 52. Versioned API

```rust
pub struct ApiVersion(pub u16);
```

---

# 53. Compatibility Window

Current + previous major where feasible.

---

# 54. API Version Negotiation

Explicit.

---

# 55. No Silent Field Repurposing

Hard rule.

---

# 56. Evolution Rules

Allowed:

```text
add optional field
add capability
add enum variant with forward-compatible handling
```

---

# 57. Breaking Changes

Require new API version.

---

# 58. Unknown Enum Variant

Handle safely.

---

# 59. SDK Feature Discovery

```rust
pub struct SdkCapabilities {
    pub api_version: ApiVersion,
    pub features: BTreeSet<SdkFeature>,
}
```

---

# 60. Feature Discovery

Avoid probing sensitive internals.

---

# 61. Local IPC

Preferred for desktop daemon architecture.

---

# 62. IPC Transport

Possible:

```text
Unix domain socket
named pipe
local QUIC
```

---

# 63. Local-Only Binding

Default.

---

# 64. No Public Listener

Hard rule.

---

# 65. IPC Authentication

Use:

```text
OS peer credentials
session capability
```

---

# 66. Same-User Check

Useful.

---

# 67. IPC Frame

Versioned.

---

# 68. Postcard

Preferred.

---

# 69. JSON

Only if external ecosystem needs it.

---

# 70. IPC Backpressure

Bounded.

---

# 71. No Unlimited Subscriber Queue

Hard rule.

---

# 72. Event Subscription

Extensions may subscribe to events.

---

# 73. Event Classes

```rust
pub enum PublicEvent {
    MessageReceived,
    MessageStateChanged,
    ContactChanged,
    GroupChanged,
    ConnectivityChanged,
}
```

---

# 74. Event Projection

Minimal.

---

# 75. No Internal Event Bus Exposure

Hard rule.

---

# 76. Event Capability

Required.

---

# 77. Event Filtering

Per resource scope.

---

# 78. Event Delivery

At-least-once or best-effort, declared.

---

# 79. Extension Boundary

Plugins live outside trusted core.

---

# 80. Plugin Types

```rust
pub enum PluginKind {
    UiExtension,
    Automation,
    Importer,
    Exporter,
    LocalProcessor,
    IntegrationConnector,
}
```

---

# 81. UI Extension

Presentation only unless capability granted.

---

# 82. Automation

Can perform allowed actions.

---

# 83. Importer

Parses external data.

---

# 84. Exporter

Writes approved user data.

---

# 85. Local Processor

Runs on-device transformations.

---

# 86. Integration Connector

Talks to external service.

---

# 87. Plugin Manifest

```rust
pub struct PluginManifest {
    pub plugin_id: PluginId,
    pub version: PluginVersion,
    pub kind: PluginKind,
    pub requested_capabilities: Vec<ScopedCapabilityRequest>,
    pub network_policy: PluginNetworkPolicy,
}
```

---

# 88. Plugin ID

Stable for update lineage.

---

# 89. Plugin Signature

Recommended.

---

# 90. Unsigned Plugin

Development only or explicit unsafe mode.

---

# 91. Production Store

Signed.

---

# 92. Plugin Capability Review

User sees:

```text
what data
what actions
what external network access
```

---

# 93. No Hidden Capability

Hard rule.

---

# 94. Capability Diff On Update

Required.

---

# 95. Update Adds Capability

Fresh approval required.

---

# 96. Plugin Runtime

Prefer sandbox.

---

# 97. WASM

Strong candidate.

---

# 98. WASM Benefits

```text
memory isolation
portable execution
restricted imports
```

---

# 99. WASM Runtime

Host exposes capability-gated functions.

---

# 100. No WASI Full Filesystem By Default

Hard rule.

---

# 101. WASI Network

Disabled by default.

---

# 102. Explicit Network Capability

Required.

---

# 103. Native Plugin

Higher risk.

---

# 104. Native Plugin Policy

Development or tightly controlled deployments.

---

# 105. Native Plugin Isolation

Separate process preferred.

---

# 106. No In-Process Arbitrary Native Plugin

Production hard rule where possible.

---

# 107. Plugin Process

Communicates via capability-gated IPC.

---

# 108. Crash Isolation

Plugin crash must not crash core.

---

# 109. Resource Limits

Plugin gets:

```text
CPU
memory
storage
network
```

budgets.

---

# 110. Plugin Resource Policy

```rust
pub struct PluginResourcePolicy {
    pub max_memory_bytes: u64,
    pub max_cpu_millis_per_window: u64,
    pub max_storage_bytes: u64,
}
```

---

# 111. No Resource DoS

Hard rule.

---

# 112. Plugin Storage

Dedicated namespace.

---

# 113. No Direct Core DB Access

Hard rule.

---

# 114. Plugin Local KV

Can be provided.

---

# 115. Plugin File Access

Virtualized/scoped.

---

# 116. File Picker Capability

Prefer user-selected file handles.

---

# 117. No Arbitrary `$HOME` Access

Hard rule.

---

# 118. Plugin Network Access

Default off.

---

# 119. Network Policy

```rust
pub enum PluginNetworkPolicy {
    None,
    ApprovedDomains(BTreeSet<DomainName>),
    UserMediated,
}
```

---

# 120. No Raw Socket Capability

Hard rule.

---

# 121. External HTTP

Through broker.

---

# 122. Network Broker

Applies:

```text
domain allowlist
TLS policy
privacy mode
rate limits
```

---

# 123. Strict Anonymous Mode

External network may be disabled entirely.

---

# 124. No Plugin Bypass Of Anonymous Routing

Hard rule.

---

# 125. Link Preview Plugin

Must respect Part 56 privacy policy.

---

# 126. External AI Plugin

Explicit per-request consent.

---

# 127. Contact Sync Plugin

No raw contact upload without explicit contract.

---

# 128. Third-Party Integration Broker

Central boundary.

---

# 129. Broker Responsibilities

```text
capability check
data projection
network policy
rate limiting
audit
error normalization
```

---

# 130. Integration Broker Trait

```rust
pub trait IntegrationBroker {
    fn authorize(
        &self,
        plugin: PluginId,
        operation: IntegrationOperation,
    ) -> Result<IntegrationAuthorization, IntegrationError>;
}
```

---

# 131. Integration Operation

```rust
pub enum IntegrationOperation {
    ReadMessages,
    SendMessage,
    ReadContacts,
    ExportData,
    ExternalRequest,
    SubscribeEvents,
}
```

---

# 132. Data Projection

Return only fields needed.

---

# 133. Projection Type

```rust
pub trait DataProjection<TPurpose> {
    type Output;

    fn project(
        &self,
        input: &Self::Input,
    ) -> Self::Output;
}
```

---

# 134. No Full Object Serialization

Hard rule.

---

# 135. External Service Contracts

Each integration declares:

```text
data classes
processing purpose
retention
network destination
auth method
```

---

# 136. Integration Privacy Contract

```rust
pub struct IntegrationPrivacyContract {
    pub data_classes: BTreeSet<DataTypeId>,
    pub purpose: DataPurpose,
    pub retention: RetentionPolicy,
    pub recipients: BTreeSet<ExternalRecipientClass>,
}
```

---

# 137. Part 56 Integration

Privacy dashboard can show active integrations.

---

# 138. Part 67 Integration

Retention/deletion policy applies to integration data.

---

# 139. Revoking Integration

Must:

```text
revoke capability
delete local token
stop background tasks
request remote deletion where supported
```

---

# 140. Remote Deletion Outcome

Honest.

---

# 141. External Auth

OAuth-like flows possible.

---

# 142. Token Storage

Secure store.

---

# 143. No Token In Plugin Plaintext If Broker Can Hold It

Preferred.

---

# 144. Brokered Credential

Plugin receives opaque handle.

---

# 145. Credential Handle

```rust
pub struct ExternalCredentialHandle(pub [u8; 32]);
```

---

# 146. No Credential Export

Hard rule by default.

---

# 147. Secret Use

Broker performs signed/authenticated request.

---

# 148. Token Rotation

Managed centrally.

---

# 149. Plugin Cannot Read Refresh Token

Preferred.

---

# 150. Webhook Ingress

Possible external integration.

---

# 151. Webhook Security

Use:

```text
signature
nonce/replay protection
rate limit
```

---

# 152. Webhook Endpoint

Should not reveal user identity.

---

# 153. Per-Integration Opaque Endpoint

Preferred.

---

# 154. No Global User Webhook ID

Hard rule.

---

# 155. Webhook Payload

Minimal.

---

# 156. Inbound Validation

Strict schema.

---

# 157. No Arbitrary JSON-to-Domain Mapping

Hard rule.

---

# 158. SDK Error Model

Stable.

---

# 159. Error Classes

```rust
pub enum SdkErrorKind {
    Unauthorized,
    CapabilityMissing,
    InvalidInput,
    NotFound,
    RateLimited,
    Conflict,
    Unavailable,
    Unsupported,
    Internal,
}
```

---

# 160. Error Privacy

Do not leak hidden state.

---

# 161. Example

Avoid:

```text
"contact exists but blocked you"
```

if not appropriate.

---

# 162. Error Normalization

Required.

---

# 163. Rate Limits

Per extension/integration.

---

# 164. Rate Limit Scope

```rust
pub struct ExtensionRateLimit {
    pub operations: u64,
    pub window: Duration,
}
```

---

# 165. No Unlimited Event Subscription

Hard rule.

---

# 166. Quotas

Storage/network/CPU.

---

# 167. Abuse Protection

Plugin flood cannot exhaust core.

---

# 168. Backpressure

Bounded IPC channels.

---

# 169. Circuit Breaker

For external integrations.

---

# 170. External Provider Failure

Does not block core messaging.

---

# 171. Hard rule.

---

# 172. Retry

Bounded with jitter.

---

# 173. Idempotency

Required for side-effecting external actions.

---

# 174. Idempotency Key

Scoped, random.

---

# 175. No Global User ID

Hard rule.

---

# 176. Import Architecture

Importer reads external data.

---

# 177. Import Stages

```text
parse
validate
normalize
preview
apply
```

---

# 178. Dry Run

Mandatory for bulk import.

---

# 179. Import Validation

Strict.

---

# 180. No Direct Store Write

Hard rule.

---

# 181. Import Goes Through Use-Cases

Preserves invariants.

---

# 182. Export Architecture

Exporter receives explicit projection.

---

# 183. Export Scope

User-selected.

---

# 184. No Hidden Additional Fields

Hard rule.

---

# 185. Export Watermark

Optional metadata.

---

# 186. Importer/Exporter Sandboxing

Recommended.

---

# 187. SDK Stability

Semantic versioning.

---

# 188. API Lifecycle

```rust
pub enum ApiLifecycleState {
    Experimental,
    Stable,
    Deprecated,
    Removed,
}
```

---

# 189. Experimental

No long-term stability promise.

---

# 190. Stable

Compatibility supported.

---

# 191. Deprecated

Migration path required.

---

# 192. Removed

Only after policy window.

---

# 193. Deprecation Notice

Machine-readable.

---

# 194. API Deprecation

```rust
pub struct ApiDeprecation {
    pub api: ApiId,
    pub deprecated_since: ApiVersion,
    pub removal_after: Option<ApiVersion>,
    pub replacement: Option<ApiId>,
}
```

---

# 195. No Immediate Removal Of Stable API

Hard rule except critical security issue.

---

# 196. Security Emergency

Can disable unsafe endpoint.

---

# 197. Migration Guide

Part 65.

---

# 198. SDK Compatibility Tests

Current/previous.

---

# 199. Generated Bindings

If used, generated from canonical schema.

---

# 200. Canonical Schema

Versioned.

---

# 201. Schema Formats

Potential:

```text
Rust source schema
C header
IDL
```

---

# 202. Avoid Overcomplicated IDL

Keep boundary narrow.

---

# 203. C ABI

Part 19.

---

# 204. C ABI Rule

Only:

```text
opaque handles
fixed-layout primitive structs
explicit ownership
```

---

# 205. No Rust ABI Exposure

Hard rule.

---

# 206. Opaque Handle

```c
typedef struct siar_client_handle siar_client_handle;
```

---

# 207. Ownership Functions

Explicit:

```text
create
retain
release
```

---

# 208. Error Buffer

Explicit size/ownership.

---

# 209. Panic Boundary

No Rust panic crosses FFI.

---

# 210. FFI Input Validation

Strict.

---

# 211. FFI Threading

Document.

---

# 212. Callback Safety

Avoid arbitrary callback reentrancy.

---

# 213. Event Pull API

Often safer than callbacks.

---

# 214. Android JNI

Narrow bridge.

---

# 215. Kotlin

Lifecycle/permission/UI only.

---

# 216. Rust Core

Product truth.

---

# 217. No Huge Byte Arrays Through JNI

Hard rule.

---

# 218. Use

```text
file descriptor
buffer handle
stream
```

---

# 219. Desktop Dioxus

Direct Rust presenter/IPC.

---

# 220. WASM SDK

For safe local extensions.

---

# 221. Browser/Web

Not required for core messaging app.

---

# 222. WASM Runtime Capability

Still useful for plugins.

---

# 223. Host Function Registry

```rust
pub enum HostFunction {
    ReadProjectedData,
    SendMessage,
    StorePluginState,
    ExternalRequest,
}
```

---

# 224. Each Host Function

Capability checked.

---

# 225. No Raw Syscall Exposure

Hard rule.

---

# 226. Extension Signing

Plugin package signed.

---

# 227. Trust Levels

```rust
pub enum PluginTrustLevel {
    Development,
    UserInstalled,
    VerifiedPublisher,
    ManagedApproved,
}
```

---

# 228. Trust Level ≠ Capability

Hard rule.

---

# 229. Verified Publisher

Does not grant extra data access automatically.

---

# 230. Plugin Package

Contains:

```text
manifest
binary/WASM
signature
SBOM
license metadata
```

---

# 231. Plugin SBOM

Recommended.

---

# 232. Plugin Update

Signed.

---

# 233. Update Rollback

Anti-rollback for compromised versions if needed.

---

# 234. Plugin Revocation

Store/governance can revoke malicious plugin signature.

---

# 235. Local User Can Disable

Immediate.

---

# 236. Kill Switch

Scoped to plugin version/id.

---

# 237. No Network-Wide User Data Sweep

Hard rule.

---

# 238. Extension Observability

Safe metrics:

```text
plugin crashes
rate-limit hits
capability denials
```

---

# 239. Forbidden Metrics

No:

```text
messages read by plugin
contact names
conversation IDs
```

---

# 240. Plugin Logs

Separate.

---

# 241. Log Privacy

No core secrets.

---

# 242. Support Bundle

Plugin inclusion explicit.

---

# 243. No Plugin Raw Memory Dump

Hard rule.

---

# 244. Capability Audit

Local/admin as appropriate.

---

# 245. Audit Record

```rust
pub struct CapabilityAuditRecord {
    pub plugin: PluginId,
    pub capability: ExtensionCapability,
    pub action: CapabilityAuditAction,
    pub timestamp: Timestamp,
}
```

---

# 246. No Resource Identity Unless Needed

Prefer scoped opaque ref.

---

# 247. User Privacy Dashboard

Can show:

```text
Plugin X can read selected messages
Plugin Y can access network
```

---

# 248. Revoke Button

Immediate.

---

# 249. Capability History

Local only.

---

# 250. Managed Environment

Admin can allowlist plugins.

---

# 251. But

Admin policy cannot secretly grant broader user-data access than product contract.

---

# 252. Hard rule.

---

# 253. Third-Party API Gateway

If remote/public SDK service exists.

---

# 254. Default

Prefer local SDK over cloud API.

---

# 255. Why

Reduces centralization/privacy exposure.

---

# 256. Remote API

Only for services that genuinely need it.

---

# 257. Remote API Auth

Scoped token/capability.

---

# 258. No Account-Wide Super Token

Hard rule.

---

# 259. Token Audience

Bound to API/service.

---

# 260. Token Scope

Bound to operation.

---

# 261. Token Expiry

Short.

---

# 262. Refresh Token

Secure.

---

# 263. Token Introspection

Avoid central hot-path dependency if possible.

---

# 264. Signed Capability Token

Possible.

---

# 265. Revocation

Short TTL + revocation list.

---

# 266. Public API Data

Minimum.

---

# 267. Remote API Should Not Expose

```text
transport IDs
mailbox IDs
device keys
provider routing
```

---

# 268. Webhook Callback

Opaque.

---

# 269. Remote Integration Data Residency

Part 55.

---

# 270. Privacy Contract

Part 56.

---

# 271. Lifecycle Contract

Part 67.

---

# 272. SDK Documentation

Part 65.

---

# 273. Every Stable API Needs

```text
contract
capability requirement
privacy notes
error model
versioning rules
examples
```

---

# 274. No Undocumented Stable Endpoint

Hard rule.

---

# 275. API Schema Tests

Golden snapshots.

---

# 276. Compatibility Tests

Current ↔ previous SDK.

---

# 277. FFI ABI Tests

Layout/symbol stability.

---

# 278. Plugin Sandbox Tests

Filesystem/network denial.

---

# 279. Capability Escalation Test

Plugin cannot request/use undeclared capability.

---

# 280. Revocation Test

Revoked capability immediately stops access.

---

# 281. Attenuation Test

Child capability never exceeds parent.

---

# 282. Network Broker Test

Unapproved domain blocked.

---

# 283. Strict Mode Test

Plugin cannot bypass anonymous-routing/privacy policy.

---

# 284. Token Leak Test

Credentials not in logs.

---

# 285. Plugin Crash Test

Core unaffected.

---

# 286. Resource DoS Test

Plugin memory/CPU limited.

---

# 287. Update Capability Diff Test

New privilege requires approval.

---

# 288. Import Test

Malformed data cannot bypass domain validation.

---

# 289. Export Test

Only approved projection leaves.

---

# 290. Webhook Replay Test

Duplicate rejected.

---

# 291. Event Queue Test

Bounded/backpressured.

---

# 292. Unknown API Version Test

Safe rejection/compat response.

---

# 293. Fuzzing

Fuzz:

```text
SDK frames
plugin manifest
FFI inputs
webhook payloads
capability tokens
```

---

# 294. Property Tests

Properties:

```text
capability attenuation never increases authority
revoked capability never authorizes future operation
plugin cannot access data outside declared resource scope
public DTO serialization never includes internal secret fields
```

---

# 295. Formal Verification Targets

Strong candidates:

```text
capability lattice
attenuation
revocation precedence
API compatibility mapping
```

---

# 296. Kani Candidate

Capability subset/authorization checks.

---

# 297. TLA+ Candidate

Capability revocation racing with in-flight operation.

---

# 298. Loom Candidate

Concurrent event subscription/revocation.

---

# 299. Performance Tests

Measure:

```text
IPC latency
capability checks
event fanout
WASM host call overhead
```

---

# 300. Performance Rule

Capability checks remain mandatory.

---

# 301. No Security Bypass Cache

Hard rule.

---

# 302. Authorization Cache

Allowed only if revocation-aware.

---

# 303. API SLOs

Examples:

```text
local SDK call latency
event delivery
plugin isolation recovery
```

---

# 304. Privacy SLOs

Examples:

```text
zero unauthorized data projection
zero plugin network bypass
zero raw secret exposure
```

---

# 305. Extension Incident

Examples:

```text
malicious plugin
credential leak
capability escalation bug
```

---

# 306. Incident Response

```text
revoke plugin
revoke capabilities
rotate external token
notify affected user/admin
```

---

# 307. No Global Panic Revocation Of Unrelated Plugins

Hard rule.

---

# 308. Plugin Store

Optional future.

---

# 309. Store Responsibilities

```text
signature verification
manifest review
SBOM
revocation
```

---

# 310. Store Approval

Not proof of safety.

---

# 311. User Still Sees Capabilities

Hard rule.

---

# 312. Developer SDK

Needs testing sandbox.

---

# 313. Mock Core

Useful.

---

# 314. Test Capability Issuer

Development only.

---

# 315. No Development Capability Issuer In Production

Hard rule.

---

# 316. SDK Testkit

Provides:

```text
synthetic identities
messages
events
external broker
```

---

# 317. Extension Compatibility Matrix

Tracks:

```text
SDK version
core version
plugin API version
```

---

# 318. Semantic Versioning

Use carefully.

---

# 319. Protocol Version ≠ SDK Version

Hard rule.

---

# 320. Storage Schema ≠ API Version

Hard rule.

---

# 321. Adapter Layer

Maps between them.

---

# 322. API Deprecation Governance

Part 52/53.

---

# 323. Emergency Removal

Security issue can override normal deprecation.

---

# 324. Migration Assistant

Can provide automated code hints/tools.

---

# 325. No Auto-Migration That Expands Privilege

Hard rule.

---

# 326. Documentation Generation

Generate:

```text
Rust docs
C headers
capability tables
schema reference
```

---

# 327. Capability Matrix

Machine-readable.

---

# 328. Example

```text
API endpoint
→ required capability
→ data classes
→ privacy contract
```

---

# 329. CI Gate

Every public endpoint must map to capability and privacy contract.

---

# 330. No Unclassified Endpoint

Hard rule.

---

# 331. API Threat Model

Threats:

```text
capability theft
plugin escalation
malicious integration
confused deputy
token replay
schema smuggling
FFI memory corruption
```

---

# 332. Confused Deputy

Core must know:

```text
who requested
what capability
what resource
```

---

# 333. Explicit Request Context

```rust
pub struct ExtensionRequestContext {
    pub plugin: PluginId,
    pub capability: CapabilityId,
    pub resource: CapabilityResource,
}
```

---

# 334. No Implicit "current plugin"

Preferred.

---

# 335. Token Replay

Nonce/expiry/audience.

---

# 336. Capability Theft

Secure storage + revocation.

---

# 337. FFI Memory Safety

Opaque handles + validated buffers.

---

# 338. Native Plugin Memory Risk

Process isolation.

---

# 339. Remote Integration Abuse

Rate limits/circuit breaker.

---

# 340. Security & Privacy Invariants

Mandatory:

```text
1. Public/third-party integrations never receive direct database, key-store, raw transport, or operator-control access.
2. Every extension operation requires an explicit capability with bounded scope and lifetime.
3. Capability delegation, if allowed, can only attenuate authority.
4. Persistent capabilities are revocable and revocation survives restart.
5. Plugins receive purpose-specific data projections, not full internal objects.
6. Plugin network access is denied by default and cannot bypass SIAR privacy-routing policy.
7. Native extensions are process-isolated where possible; WASM is preferred for sandboxed in-process extensibility.
8. Stable SDK/API contracts are versioned independently from storage schema and wire protocol.
9. Plugin updates that request new capabilities require fresh approval.
10. Third-party credentials are brokered and stored securely; plugins do not receive refresh secrets by default.
11. Capability/audit telemetry never becomes a record of user messages, contacts, or conversation behavior.
12. No development/test bypass capability issuer exists in production builds.
```

---

# 341. Recommended Crate Layout

```text
crates/
├── siar-api-core/
├── siar-sdk/
├── siar-sdk-dto/
├── siar-capability/
├── siar-integration-broker/
├── siar-plugin-core/
├── siar-plugin-wasm/
├── siar-plugin-ipc/
├── siar-ffi/
├── siar-api-compat/
├── siar-extension-observability/
└── siar-extension-testkit/
```

---

# 342. `siar-api-core`

Owns:

```text
API versions
endpoint IDs
stable errors
```

---

# 343. `siar-sdk`

Rust SDK.

---

# 344. `siar-sdk-dto`

Stable purpose-specific DTOs.

---

# 345. `siar-capability`

Issuance, attenuation, revocation, authorization.

---

# 346. `siar-integration-broker`

External network/credential/data boundary.

---

# 347. `siar-plugin-core`

Manifest/lifecycle/resource policy.

---

# 348. `siar-plugin-wasm`

Sandboxed WASM host.

---

# 349. `siar-plugin-ipc`

Out-of-process plugin protocol.

---

# 350. `siar-ffi`

Stable C ABI.

---

# 351. `siar-api-compat`

Version/deprecation/compatibility rules.

---

# 352. `siar-extension-observability`

Privacy-safe plugin/API metrics.

---

# 353. `siar-extension-testkit`

Capability/sandbox/FFI/integration simulation.

---

# 354. Error Taxonomy

```rust
pub enum CapabilityError {
    Missing,
    Expired,
    Revoked,
    ScopeViolation,
    DelegationInvalid,
    Internal,
}

pub enum IntegrationError {
    Unauthorized,
    CapabilityDenied,
    ExternalNetworkDenied,
    RateLimited,
    ExternalUnavailable,
    CredentialUnavailable,
    InvalidPayload,
    Internal,
}
```

---

# 355. Initial Production Scope

Implement first:

```text
stable Rust SDK
opaque public references
versioned DTOs
capability registry
persistent capability revocation
local IPC API
event subscriptions with bounded queues
C ABI with opaque handles
WASM plugin sandbox
plugin manifest/capability approval
network broker
credential broker
import/export adapters
compatibility tests
capability/privacy CI gate
```

Then add:

```text
managed plugin store
generated bindings for additional languages
remote API for approved service use-cases
fine-grained capability delegation
formal capability-lattice verification
advanced plugin provenance/SBOM enforcement
```

---

# 356. Definition of Done

Part 68 is complete when:

- public, plugin, FFI, operator, federation, and internal APIs are explicitly separated
- all public references are opaque
- stable DTOs are independent of internal database/domain structs
- capability-scoped authorization is mandatory for third-party operations
- persistent grants are revocable
- data projections minimize fields
- WASM/out-of-process isolation protects the trusted core
- plugin files/network/storage/resources are sandboxed
- external credentials remain in broker/secure-store when possible
- plugin updates cannot silently gain privileges
- import/export paths go through normal domain/use-case validation
- C ABI ownership and panic boundaries are explicit
- SDK/API compatibility and deprecation rules are versioned
- external integration failures cannot block core messaging
- capability, sandbox, FFI, replay, resource, fuzz, and formal tests are specified

---

# 357. Final Architecture

```text
             THIRD-PARTY APP / PLUGIN / SDK
                           │
                           ▼
                     CAPABILITY TOKEN
                           │
                           ▼
                API / INTEGRATION BROKER
                           │
             ┌─────────────┼─────────────┐
             │             │             │
          Data View      Action       Network
         Projection      Gateway      Broker
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                    APPLICATION LAYER
                           │
                           ▼
                        SIAR CORE
```

Extension-safety model:

```text
opaque references
+
explicit capabilities
+
purpose-specific projections
+
sandboxed execution
+
brokered external access
+
versioned contracts
+
revocation
```

not:

```text
plugins get a Rust object, database handle, and network socket and are trusted to behave
```

---

# 358. Final Principle

Extensibility should increase product capability without expanding the trusted computing base unnecessarily.

The correct model is:

```text
narrow APIs
+
least-privilege capabilities
+
sandboxed extensions
+
brokered external services
+
stable versioned SDKs
```

This architecture lets SIAR support third-party integrations, local automation, plugins, native bindings, and future SDK ecosystems while preserving the identity, storage, cryptographic, and anonymity boundaries established across Parts 34–67.
