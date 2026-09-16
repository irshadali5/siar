# Core System Architecture Part 130 — Anonymous Network SDK, Client Library, Generated Bindings, Developer Compatibility, Integration Certification & Privacy-Preserving Developer Platform Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 130  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 19, 21, 24, 68, 79–81, 122–129

**Primary purpose:** define SIAR's developer-platform architecture for Rust SDKs, client libraries, generated bindings, stable DTOs, C ABI/FFI ownership, extension capabilities, integration certification, developer compatibility, packaging, code generation, testkits, and privacy-preserving third-party access.

---

# 1. Purpose

A platform SDK is a security boundary.

Poorly designed SDKs can accidentally expose:

```text
internal identifiers
unstable storage models
unsafe raw protocol access
private metadata
global credentials
unbounded resource access
```

A mature developer platform must answer:

```text
Which APIs are stable?
Which APIs are internal?
How are bindings generated?
How are SDK versions tied to protocol versions?
What can third-party integrations access?
How is an integration certified?
How are secrets/capabilities scoped?
How do we prevent SDKs from becoming a privacy bypass?
```

The governing principle is:

> **SIAR developer tooling should expose intent-oriented, capability-scoped, versioned interfaces that preserve security/privacy boundaries, remain stable across implementation changes, and are verifiable across languages and platforms without exposing internal trust state or user-private data by default.**

---

# 2. Architectural Position

```text
                    SIAR CORE
                       │
                       ▼
                STABLE SDK SERVICE
                       │
          ┌────────────┼────────────┐
          │            │            │
        RUST SDK      C ABI      GENERATED BINDINGS
          │            │            │
          └────────────┼────────────┘
                       ▼
               CLIENT / INTEGRATION
                       │
                       ▼
               CERTIFICATION TESTKIT
                       │
                       ▼
                APPROVED INTEGRATION
```

---

# 3. Core Separation

Keep distinct:

```text
domain model
SDK model
wire model
storage model
binding model
integration capability
runtime privilege
```

---

# 4. Non-Goals

Part 130 does not create:

```text
direct database SDKs
raw internal protocol access for ordinary apps
global developer API keys
third-party user-behavior analytics
SDK surfaces that mirror internal structs
```

---

# 5. SDK Philosophy

Expose:

```text
intent
capability
stable identifiers
bounded async operations
typed errors
```

Do not expose:

```text
database rows
internal actor messages
transport internals
secret key material
```

---

# 6. SDK Identity

```rust
pub struct SdkId(pub [u8; 16]);
```

---

# 7. SDK Version

```rust
pub struct SdkVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}
```

---

# 8. SDK Compatibility

Semver may describe source/API compatibility.

Runtime behavior still governed by Part 129 compatibility registry.

---

# 9. Hard rule.

---

# 10. SDK Surface Classes

```rust
pub enum SdkSurfaceClass {
    CoreRust,
    EmbeddedRust,
    CAbi,
    KotlinBinding,
    SwiftBinding,
    JavaBinding,
    CSharpBinding,
    JavaScriptBinding,
    PythonBinding,
}
```

---

# 11. Rust-First Canonical API

The Rust SDK is the canonical typed API.

Generated language bindings map from stable SDK DTOs/commands/events.

---

# 12. Hard rule.

---

# 13. Stable SDK Model

```rust
pub struct SdkContact {
    pub id: SdkContactId,
    pub display_name: Option<String>,
}
```

SDK models are not aliases of database/domain structs.

---

# 14. Hard rule.

---

# 15. Opaque Identifiers

Use opaque IDs:

```rust
pub struct SdkConversationId(pub [u8; 16]);
pub struct SdkMessageId(pub [u8; 16]);
pub struct SdkDeviceId(pub [u8; 16]);
```

---

# 16. No Raw Internal IDs Unless Deliberately Public

Hard rule.

---

# 17. SDK Command Pattern

```rust
pub enum SdkCommand {
    SendMessage(SendMessageRequest),
    CreateConversation(CreateConversationRequest),
    AddAttachment(AddAttachmentRequest),
    SetPresence(SetPresenceRequest),
}
```

---

# 18. SDK Event Pattern

```rust
pub enum SdkEvent {
    MessageChanged(MessageSnapshot),
    ConversationChanged(ConversationSnapshot),
    TransferProgress(TransferProgress),
    ConnectivityChanged(ConnectivitySnapshot),
}
```

---

# 19. Hard rule.

---

# 20. SDK Query Pattern

```rust
pub trait ConversationSdk {
    async fn list_conversations(
        &self,
        query: ConversationQuery,
    ) -> Result<Page<ConversationSummary>, SdkError>;
}
```

---

# 21. Bounded Pagination

Hard rule.

---

# 22. No "Dump All History" Default API

Hard rule.

---

# 23. Capability-Based Access

SDK client operates with capability grants.

```rust
pub struct SdkCapabilityGrant {
    pub capability: SdkCapability,
    pub scope: SdkCapabilityScope,
    pub expires_at: Option<Timestamp>,
}
```

---

# 24. SDK Capability

```rust
pub enum SdkCapability {
    ReadConversations,
    SendMessages,
    ManageAttachments,
    ReadPresence,
    ManageDevices,
    AccessDiagnostics,
    AdminTenant,
}
```

---

# 25. No Capability Implies All Access

Hard rule.

---

# 26. Capability Scope

```rust
pub enum SdkCapabilityScope {
    LocalUser,
    Conversation(SdkConversationId),
    Tenant(TenantId),
    Integration(IntegrationId),
}
```

---

# 27. No Global Scope By Default

Hard rule.

---

# 28. Permission Attenuation

Parent capability may issue narrower delegated capability.

---

# 29. Hard rule.

---

# 30. SDK Session

```rust
pub struct SdkSession {
    pub session_id: SdkSessionId,
    pub subject: SdkSubject,
    pub capabilities: Vec<SdkCapabilityGrant>,
}
```

---

# 31. Session Expiry

Short-lived for remote integrations where possible.

---

# 32. Hard rule.

---

# 33. SDK Authentication

Depends environment:

```text
local embedded app
local daemon IPC
managed service integration
federated integration
```

---

# 34. Local Embedded Rust

May use process-local capability object.

---

# 35. Local IPC

Use authenticated local IPC session.

---

# 36. Remote Integration

Use scoped service/workload identity.

---

# 37. Hard rule.

---

# 38. Developer Credentials

Never one global permanent key.

---

# 39. Developer Credential

```rust
pub struct DeveloperCredential {
    pub credential_id: DeveloperCredentialId,
    pub integration: IntegrationId,
    pub scopes: BTreeSet<SdkCapability>,
    pub expires_at: Timestamp,
}
```

---

# 40. Hard rule.

---

# 41. Secret Storage

Credentials referenced via secret broker/secure store.

---

# 42. Hard rule.

---

# 43. No Secrets In Source Code

Hard rule.

---

# 44. C ABI Layer

Part 19.

C ABI must expose:

```text
opaque handles
explicit ownership
explicit allocator rules
versioned functions
error codes
```

---

# 45. ABI Handle

```rust
#[repr(transparent)]
pub struct SiarHandle(u64);
```

---

# 46. Hard rule.

---

# 47. FFI Ownership

Every returned allocation documents:

```text
who allocates
who frees
thread safety
lifetime
```

---

# 48. Hard rule.

---

# 49. No Rust Layout Across FFI

Hard rule.

---

# 50. ABI DTOs

Use stable C-compatible DTO/envelope.

---

# 51. Hard rule.

---

# 52. FFI Errors

```rust
#[repr(C)]
pub enum SiarFfiStatus {
    Ok = 0,
    InvalidArgument = 1,
    Unauthorized = 2,
    NotFound = 3,
    Conflict = 4,
    Internal = 255,
}
```

---

# 53. Detailed Errors

Returned through versioned structured error object.

---

# 54. Hard rule.

---

# 55. Generated Bindings

Preferred from stable interface description.

---

# 56. Binding Source

Generated from:

```text
Rust SDK contract metadata
C ABI
or dedicated IDL
```

---

# 57. Hard rule.

---

# 58. No Hand-Diverged Language API

If manual wrapper exists, conformance tests required.

---

# 59. Hard rule.

---

# 60. IDL

Possible SIAR SDK schema:

```text
commands
events
DTOs
errors
capabilities
version metadata
```

---

# 61. IDL Version

```rust
pub struct SdkIdlVersion(pub u32);
```

---

# 62. Hard rule.

---

# 63. Generated Binding Provenance

```rust
pub struct BindingProvenance {
    pub sdk_version: SdkVersion,
    pub idl_version: SdkIdlVersion,
    pub generator_digest: Digest,
    pub source_revision: RevisionId,
}
```

---

# 64. Hard rule.

---

# 65. Deterministic Generation

Same inputs → same generated output where possible.

---

# 66. Hard rule.

---

# 67. Generated File Editing

Generated files are not manually edited.

---

# 68. Hard rule.

---

# 69. Kotlin Binding

Android preferred:

```text
Kotlin UI/application
→ generated Kotlin wrapper
→ JNI/C ABI
→ Rust SDK
```

---

# 70. Hard rule.

---

# 71. Swift Binding

Apple platforms:

```text
Swift
→ C ABI generated wrapper
→ Rust SDK
```

---

# 72. Hard rule.

---

# 73. Java/C# Bindings

Generated stable DTOs + async adapters.

---

# 74. Hard rule.

---

# 75. JavaScript/Python

Only where product/integration needs justify.

---

# 76. Avoid exposing private local core to web by default.

---

# 77. Hard rule.

---

# 78. Async Model

Canonical Rust SDK async.

---

# 79. Binding Adaptation

Map to:

```text
Future
Promise
callback
suspend function
Task
```

per language.

---

# 80. Hard rule.

---

# 81. Cancellation

SDK operations should support cancellation where meaningful.

---

# 82. Hard rule.

---

# 83. Timeout

Remote/long operations bounded.

---

# 84. Hard rule.

---

# 85. Backpressure

Event streams bounded.

---

# 86. Hard rule.

---

# 87. Event Subscription

```rust
pub trait EventSubscription {
    async fn next(&mut self) -> Result<SdkEvent, SdkError>;
}
```

---

# 88. Bounded Buffer

Hard rule.

---

# 89. Slow Consumer

Use:

```text
drop/coalesce noncritical updates
disconnect
resync snapshot
```

depending event class.

---

# 90. Hard rule.

---

# 91. Snapshot + Delta Model

Preferred for UI/integrations.

---

# 92. Hard rule.

---

# 93. Stable Error Taxonomy

```rust
pub enum SdkError {
    InvalidArgument,
    Unauthorized,
    CapabilityDenied,
    NotFound,
    Conflict,
    Offline,
    Timeout,
    VersionMismatch,
    RateLimited,
    Unsupported,
    Internal,
}
```

---

# 94. No Leak Of Sensitive Internal Failure Detail

Hard rule.

---

# 95. Error Detail

Optional diagnostic correlation ref.

---

# 96. Hard rule.

---

# 97. Local-First SDK Semantics

Methods should expose local durability truth.

Example:

```rust
pub enum SendAcceptance {
    PersistedLocally,
    Rejected,
}
```

---

# 98. Do Not Equate With Remote Delivery

Hard rule.

---

# 99. Delivery State

```rust
pub enum SdkDeliveryState {
    LocalPersisted,
    Queued,
    SentToTransport,
    Delivered,
    Read,
    FailedRetryable,
    FailedPermanent,
}
```

---

# 100. Hard rule.

---

# 101. Attachment SDK

Use handles/streams, not giant byte arrays.

---

# 102. Example

```rust
pub struct AttachmentSource {
    pub handle: AttachmentSourceHandle,
    pub declared_size: u64,
}
```

---

# 103. Hard rule.

---

# 104. Android URI/FD Bridge

Kotlin passes FD/handle.

Rust streams content.

---

# 105. Hard rule.

---

# 106. Desktop Path Safety

Avoid unrestricted path access from third-party SDK.

---

# 107. Hard rule.

---

# 108. Privacy-Safe SDK Defaults

Default API should avoid returning:

```text
raw IP addresses
relay topology
internal device graph
full contact graph
private audit history
```

---

# 109. Hard rule.

---

# 110. Presence API

Expose only authorized presence projection.

---

# 111. Hard rule.

---

# 112. Search API

Scoped to caller's authorized local/tenant data.

---

# 113. Hard rule.

---

# 114. Diagnostics API

Separate privileged capability.

---

# 115. Hard rule.

---

# 116. Admin SDK

Separate package/surface from user SDK.

---

# 117. Hard rule.

---

# 118. Admin Capability

Never silently available in standard client.

---

# 119. Hard rule.

---

# 120. Developer Platform Layers

Recommended:

```text
siar-sdk-core
siar-sdk-client
siar-sdk-admin
siar-sdk-ffi
siar-sdk-codegen
siar-sdk-testkit
```

---

# 121. Hard rule.

---

# 122. SDK Version Compatibility

SDK release declares:

```text
supported server versions
supported protocol versions
minimum client core version
```

---

# 123. Hard rule.

---

# 124. SDK Compatibility Manifest

```rust
pub struct SdkCompatibilityManifest {
    pub sdk: SdkVersion,
    pub protocols: BTreeMap<ProtocolId, VersionRange<ProtocolVersion>>,
    pub server_versions: VersionRange<Version>,
}
```

---

# 125. Hard rule.

---

# 126. SDK Runtime Negotiation

Client validates compatibility before feature use.

---

# 127. Hard rule.

---

# 128. Generated Binding Compatibility

Binding version tied to SDK/IDL.

---

# 129. Hard rule.

---

# 130. Major Version Change

Breaking developer API change.

---

# 131. Must include:

```text
migration guide
deprecation window
compatibility notes
```

---

# 132. Hard rule.

---

# 133. Deprecated SDK API

```rust
pub struct SdkApiDeprecation {
    pub symbol: SdkSymbolId,
    pub deprecated_since: SdkVersion,
    pub removal_not_before: SdkVersion,
    pub replacement: Option<SdkSymbolId>,
}
```

---

# 134. Hard rule.

---

# 135. No Silent Removal

Hard rule.

---

# 136. SDK Schema Evolution

DTO evolution must follow Part 129 compatibility governance.

---

# 137. Hard rule.

---

# 138. Binding Field Rules

Prefer:

```text
optional additive fields
stable enums
version gates
```

---

# 139. Unknown Enum Handling

Explicit.

---

# 140. Hard rule.

---

# 141. Integration Identity

```rust
pub struct IntegrationId(pub [u8; 16]);
```

---

# 142. Integration Class

```rust
pub enum IntegrationClass {
    LocalApp,
    EnterpriseConnector,
    Bot,
    Automation,
    ImporterExporter,
    MonitoringAdapter,
    Plugin,
}
```

---

# 143. Hard rule.

---

# 144. Integration Registration

Records:

```text
identity
capabilities
owner organization
callback endpoints if any
certification state
```

---

# 145. Hard rule.

---

# 146. Integration Capability Policy

```rust
pub struct IntegrationPolicy {
    pub integration: IntegrationId,
    pub capabilities: BTreeSet<SdkCapability>,
    pub data_classes: BTreeSet<IntegrationDataClass>,
}
```

---

# 147. Data Class

```rust
pub enum IntegrationDataClass {
    MessagingMetadata,
    MessageContent,
    AttachmentContent,
    Presence,
    TenantAdmin,
    Diagnostics,
}
```

---

# 148. MessageContent Requires Explicit Capability

Hard rule.

---

# 149. AttachmentContent Separate

Hard rule.

---

# 150. No "ReadEverything"

Hard rule.

---

# 151. OAuth/OIDC-Style User Authorization

For user-authorized remote integrations where appropriate.

---

# 152. Scopes are narrow.

---

# 153. Hard rule.

---

# 154. Device-Local Approval

Local-first integration can receive local approval without central account linkage.

---

# 155. Hard rule.

---

# 156. Consent Receipt

Where user consent is required:

```rust
pub struct IntegrationConsentReceipt {
    pub integration: IntegrationId,
    pub scopes: BTreeSet<SdkCapability>,
    pub granted_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}
```

---

# 157. Hard rule.

---

# 158. Consent != Security Authorization Alone

Both needed.

---

# 159. Hard rule.

---

# 160. Integration Revocation

Immediate capability revocation.

---

# 161. Hard rule.

---

# 162. Token Rotation

Supported.

---

# 163. Hard rule.

---

# 164. Callback/Webhook Security

If supported:

```text
signed events
replay protection
destination allowlist
retry bounds
```

---

# 165. Hard rule.

---

# 166. No Raw Internal Events To Webhooks

Use stable external event schema.

---

# 167. Hard rule.

---

# 168. Integration Rate Limits

Per integration/capability scope.

---

# 169. Hard rule.

---

# 170. Resource Quotas

Bound:

```text
events
message sends
attachment bandwidth
search requests
```

---

# 171. Hard rule.

---

# 172. Abuse Boundary

Developer platform must not enable:

```text
mass scraping
contact enumeration
presence harvesting
```

---

# 173. Hard rule.

---

# 174. Pagination / Search Limits

Hard bounds.

---

# 175. Hard rule.

---

# 176. Bulk Export

Separate explicit capability.

---

# 177. Hard rule.

---

# 178. Integration Certification

Third-party integrations may require certification.

---

# 179. Certification Record

```rust
pub struct IntegrationCertification {
    pub certification_id: IntegrationCertificationId,
    pub integration: IntegrationId,
    pub sdk_version: SdkVersion,
    pub state: IntegrationCertificationState,
}
```

---

# 180. Certification State

```rust
pub enum IntegrationCertificationState {
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

# 181. Hard rule.

---

# 182. Certification Scope

Includes:

```text
SDK version
binding version
capability scopes
platform
integration class
```

---

# 183. Hard rule.

---

# 184. Integration Testkit

Provides:

```text
mock server/core
conformance suite
capability denial tests
rate-limit tests
offline behavior tests
version-skew tests
privacy tests
```

---

# 185. Hard rule.

---

# 186. Conformance Test

Checks integration does not depend on undefined behavior.

---

# 187. Hard rule.

---

# 188. Certification Evidence

Stored in Part 126 archive.

---

# 189. Hard rule.

---

# 190. Certification Revocation

Possible when:

```text
integration compromised
scope abuse
SDK incompatibility
privacy violation
```

---

# 191. Hard rule.

---

# 192. Revocation Does Not Delete History

Hard rule.

---

# 193. Runtime Enforcement

Certification may be required for privileged integration classes.

---

# 194. Hard rule.

---

# 195. Uncertified Local Development

Allowed in development sandbox.

---

# 196. Hard rule.

---

# 197. Developer Sandbox

Isolated test environment.

---

# 198. Sandbox Data

Synthetic by default.

---

# 199. Hard rule.

---

# 200. Sandbox Credentials

Cannot access production.

---

# 201. Hard rule.

---

# 202. Reference Implementation

Provide official sample apps/connectors.

---

# 203. Purpose:

```text
demonstrate correct patterns
not define hidden behavior
```

---

# 204. Hard rule.

---

# 205. Example Apps

Recommended:

```text
minimal Rust client
Android Kotlin binding demo
C ABI smoke test
enterprise connector example
```

---

# 206. Hard rule.

---

# 207. SDK Documentation

Required:

```text
quickstart
capability model
errors
lifecycle
compatibility
security
privacy
rate limits
```

---

# 208. Hard rule.

---

# 209. Security Documentation

Explain:

```text
token storage
scope minimization
webhook verification
secret rotation
```

---

# 210. Hard rule.

---

# 211. Privacy Documentation

Explain:

```text
which data classes each API can expose
consent expectations
retention expectations
```

---

# 212. Hard rule.

---

# 213. No Hidden Privileged Endpoints

Hard rule.

---

# 214. API Discovery

Developer documentation can enumerate public SDK APIs.

Internal APIs separate.

---

# 215. Hard rule.

---

# 216. Internal SDK

May expose more to first-party UI/daemon.

---

# 217. Still capability-scoped.

---

# 218. Hard rule.

---

# 219. Plugin Boundary

Part 24/68.

Plugins should consume SDK/brokered capabilities.

---

# 220. No direct internal crate access for untrusted plugins.

---

# 221. Hard rule.

---

# 222. WASM Plugins

Preferred isolation for third-party extensions where practical.

---

# 223. Hard rule.

---

# 224. Native Plugin

Requires stronger certification/sandbox.

---

# 225. Hard rule.

---

# 226. Brokered Network Access

Plugins/integrations request network capability rather than unrestricted sockets where feasible.

---

# 227. Hard rule.

---

# 228. File Access

Use brokered file handles/selectors.

---

# 229. Hard rule.

---

# 230. Clipboard/Camera/Mic

Require explicit platform/user permission.

---

# 231. Hard rule.

---

# 232. Capability Manifest

Integration declares requested capabilities.

```rust
pub struct IntegrationManifest {
    pub integration: IntegrationId,
    pub requested_capabilities: BTreeSet<SdkCapability>,
    pub network_access: NetworkAccessPolicy,
    pub file_access: FileAccessPolicy,
}
```

---

# 233. Hard rule.

---

# 234. Manifest Review

Diff capabilities between releases.

---

# 235. Hard rule.

---

# 236. Scope Expansion

Requires re-consent/re-certification where applicable.

---

# 237. Hard rule.

---

# 238. SDK Package Distribution

Official packages should be:

```text
signed
checksum-published
SBOM-linked
provenance-linked
```

---

# 239. Hard rule.

---

# 240. Rust Crate Distribution

Crates.io or controlled registry depending package.

---

# 241. Hard rule.

---

# 242. Native Binding Packages

Examples:

```text
Maven
Swift Package
NuGet
npm
PyPI
```

only where supported.

---

# 243. Hard rule.

---

# 244. Package Provenance

```rust
pub struct SdkPackageProvenance {
    pub sdk_version: SdkVersion,
    pub source_revision: RevisionId,
    pub build_digest: Digest,
    pub sbom: Digest,
}
```

---

# 245. Hard rule.

---

# 246. Package Signing

Dedicated SDK distribution signing identity.

---

# 247. Hard rule.

---

# 248. Compromised SDK Release

Revoke package/certification.

---

# 249. Hard rule.

---

# 250. Supply-Chain Integration

Part 73.

---

# 251. Hard rule.

---

# 252. SDK Release Qualification

Part 125.

---

# 253. Hard rule.

---

# 254. SDK Certification Archive

Part 126.

---

# 255. Hard rule.

---

# 256. Compatibility Registry Integration

Part 129.

SDK release declares exact compatible versions.

---

# 257. Hard rule.

---

# 258. Product Lifecycle Integration

Part 128.

Deprecated SDK versions get support/sunset policy.

---

# 259. Hard rule.

---

# 260. Product Readiness Integration

Part 127.

Developer-facing APIs must reach appropriate readiness before stable guarantee.

---

# 261. Hard rule.

---

# 262. Architecture Governance Integration

Part 122.

New public SDK surface requires design review.

---

# 263. Hard rule.

---

# 264. Knowledge Graph Integration

Part 123.

Trace:

```text
SDK symbol
→ requirement
→ ADR
→ implementation
→ tests
→ package
```

---

# 265. Hard rule.

---

# 266. Requirements Integration

Part 124.

Public SDK compatibility promises become requirements.

---

# 267. Hard rule.

---

# 268. Risk/PIR Integration

Parts 120–121.

Integration incidents create regression tests/risk entries.

---

# 269. Hard rule.

---

# 270. Release Integration

Part 108.

SDK package/release qualification tied to core release where necessary.

---

# 271. Hard rule.

---

# 272. Developer Compatibility Matrix

```rust
pub struct DeveloperCompatibilityMatrix {
    pub sdk_version: SdkVersion,
    pub binding_versions: BTreeMap<SdkSurfaceClass, Version>,
    pub supported_core_versions: VersionRange<Version>,
}
```

---

# 273. Hard rule.

---

# 274. Binding Conformance

Each generated binding must pass canonical contract suite.

---

# 275. Hard rule.

---

# 276. Cross-Language Semantic Parity

Same operation should preserve:

```text
meaning
errors
capability checks
delivery semantics
```

---

# 277. Hard rule.

---

# 278. Language Idiomaticity

Bindings may adapt naming/async style.

---

# 279. Must not change semantics.

---

# 280. Hard rule.

---

# 281. Nullability

Generated bindings map Rust Option explicitly.

---

# 282. Hard rule.

---

# 283. Integer Widths

Explicit.

---

# 284. Hard rule.

---

# 285. Binary Data

Use byte buffer/stream abstraction.

---

# 286. Hard rule.

---

# 287. Time

Use explicit timestamp/timezone semantics.

---

# 288. Hard rule.

---

# 289. Error Mapping

Canonical code + language-native wrapper.

---

# 290. Hard rule.

---

# 291. Thread Safety

Document per handle/object.

---

# 292. Hard rule.

---

# 293. Reentrancy

Document callback behavior.

---

# 294. Hard rule.

---

# 295. Memory Pressure

Binding avoids unnecessary copies.

---

# 296. Hard rule.

---

# 297. Zero-Copy

Only when safety/lifetime is explicit.

---

# 298. Hard rule.

---

# 299. JNI

Avoid large object graph marshalling.

---

# 300. Use handles/FDs/flat DTOs.

---

# 301. Hard rule.

---

# 302. Kotlin Coroutines

Map Rust async to suspend functions/Flow where appropriate.

---

# 303. Hard rule.

---

# 304. Swift Async/Await

Use generated wrapper.

---

# 305. Hard rule.

---

# 306. Callback Lifetime

Explicit unsubscribe/drop semantics.

---

# 307. Hard rule.

---

# 308. SDK Observability

Safe metrics:

```text
SDK version
operation class
latency
error class
binding class
```

---

# 309. No user identity.

---

# 310. Hard rule.

---

# 311. Developer Analytics Boundary

Allowed:

```text
package download aggregate
documentation error reports
conformance failures
```

---

# 312. Not:

```text
developer productivity
source repository surveillance
user behavior through integrations
```

---

# 313. Hard rule.

---

# 314. Anonymous Mode SDK

Must not expose extra metadata unavailable in anonymous-mode product UI.

---

# 315. Hard rule.

---

# 316. Privacy Mode Capability Reduction

Maximum anonymity may restrict:

```text
presence APIs
detailed diagnostics
peer metadata
```

---

# 317. Hard rule.

---

# 318. Integration Isolation

Third-party integrations cannot request weaker privacy mode on behalf of user silently.

---

# 319. Hard rule.

---

# 320. Sensitive Capability Prompt

User-visible approval where needed.

---

# 321. Hard rule.

---

# 322. Integration Data Retention

Third-party receives only scoped data; platform cannot guarantee external retention after authorized disclosure.

---

# 323. Must communicate scope clearly.

---

# 324. Hard rule.

---

# 325. Export Accountability

Bulk export capability produces audit receipt where policy requires.

---

# 326. Hard rule.

---

# 327. SDK Rate Limiting

Rate-limit result typed.

```rust
pub struct RateLimitInfo {
    pub retry_after: Option<Duration>,
    pub quota_class: QuotaClass,
}
```

---

# 328. Hard rule.

---

# 329. Offline SDK Behavior

Operations return truthful local state.

---

# 330. Hard rule.

---

# 331. Retry Responsibility

SDK may handle safe retry.

Application should not duplicate idempotent semantics unknowingly.

---

# 332. Hard rule.

---

# 333. Idempotency Token

For remotely retried write operations.

```rust
pub struct IdempotencyKey(pub [u8; 16]);
```

---

# 334. Hard rule.

---

# 335. SDK Cache Semantics

Do not expose stale data as authoritative without freshness state.

---

# 336. Hard rule.

---

# 337. Freshness

```rust
pub enum SdkDataFreshness {
    LocalAuthoritative,
    Synced,
    Stale,
    Unknown,
}
```

---

# 338. Hard rule.

---

# 339. Conformance Profile

```rust
pub struct SdkConformanceProfile {
    pub sdk_version: SdkVersion,
    pub required_tests: BTreeSet<ConformanceTestId>,
}
```

---

# 340. Hard rule.

---

# 341. Integration Qualification

Must test:

```text
capability denial
version mismatch
offline behavior
rate limit
revocation
privacy mode
```

---

# 342. Hard rule.

---

# 343. Certification Expiry

Can expire when SDK major/security policy changes.

---

# 344. Hard rule.

---

# 345. Re-Certification

Required after material scope expansion or incompatible SDK change.

---

# 346. Hard rule.

---

# 347. Developer Platform API

```rust
pub trait DeveloperPlatformService {
    fn register_integration(
        &self,
        manifest: IntegrationManifest,
    ) -> Result<IntegrationId, DeveloperPlatformError>;

    fn certification(
        &self,
        integration: IntegrationId,
    ) -> Result<IntegrationCertification, DeveloperPlatformError>;
}
```

---

# 348. SDK Metadata Service

```rust
pub trait SdkMetadataService {
    fn compatibility(
        &self,
        version: SdkVersion,
    ) -> Result<SdkCompatibilityManifest, DeveloperPlatformError>;
}
```

---

# 349. Codegen Service

```rust
pub trait BindingGenerationService {
    fn generate(
        &self,
        sdk: SdkVersion,
        target: SdkSurfaceClass,
    ) -> Result<GeneratedBindingArtifact, DeveloperPlatformError>;
}
```

---

# 350. No User-Analytics API

Hard rule.

---

# 351. Error Taxonomy

```rust
pub enum DeveloperPlatformError {
    SdkVersionUnknown,
    BindingUnsupported,
    CapabilityDenied,
    IntegrationUnknown,
    CertificationRequired,
    CertificationRevoked,
    VersionMismatch,
    InvalidManifest,
    RateLimited,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 352. Developer Platform SLOs

Examples:

```text
stable SDK docs available with package release
binding conformance tests pass
certification revocation propagated within target
compatibility manifest freshness within target
```

---

# 353. Security SLO

```text
0 integration obtains undeclared capability
0 revoked integration retains active production credential
0 generated binding bypasses canonical authz checks
```

---

# 354. Privacy SLO

```text
0 SDK default API exposes private metadata beyond caller scope
0 anonymous-mode SDK adds stable peer fingerprinting
0 developer analytics tied to user activity
```

---

# 355. Failure Modes

```text
binding semantic drift
SDK/core mismatch
integration capability creep
credential leak
uncertified privileged integration
```

---

# 356. Binding Drift

Caught by conformance suite.

---

# 357. Hard rule.

---

# 358. SDK/Core Mismatch

Fail compatibility check.

---

# 359. Hard rule.

---

# 360. Capability Creep

Manifest diff + re-consent/re-certification.

---

# 361. Hard rule.

---

# 362. Credential Leak

Revoke/rotate.

---

# 363. Hard rule.

---

# 364. Uncertified Privileged Integration

Runtime admission denied.

---

# 365. Hard rule.

---

# 366. Testing

Need developer-platform testkit.

---

# 367. Test Scenarios

```text
Rust SDK direct
Kotlin generated binding
C ABI ownership
integration scope expansion
certification revocation
```

---

# 368. Stable DTO Test

Internal DB field change does not alter public SDK contract.

---

# 369. Capability Test

Unauthorized API returns CapabilityDenied.

---

# 370. FFI Test

Allocate/free/lifetime contract verified.

---

# 371. Binding Parity Test

Rust/Kotlin/Swift semantics equivalent.

---

# 372. Async Cancellation Test

Cancel operation safely.

---

# 373. Backpressure Test

Slow subscriber bounded.

---

# 374. Offline Test

Send returns local persistence truth, not false delivery.

---

# 375. Version Test

Incompatible SDK/core fails early.

---

# 376. Integration Revocation Test

Revoked credential immediately denied.

---

# 377. Scope Expansion Test

New sensitive scope requires new approval.

---

# 378. Certification Test

Privileged integration denied when uncertified.

---

# 379. Privacy Test

Anonymous mode exposes no detailed peer implementation metadata.

---

# 380. Package Provenance Test

SDK package digest matches signed provenance.

---

# 381. Fuzzing

Fuzz:

```text
IDL
FFI DTOs
binding decoders
event envelopes
integration manifests
```

---

# 382. Property Tests

Properties:

```text
integration can never exercise capability absent from active grant
generated binding can never change canonical authorization semantics
revoked certification can never authorize privileged runtime admission
SDK contract can never expose internal secret-key material
```

---

# 383. Formal Verification Targets

Strong candidates:

```text
capability attenuation
integration grant/revocation
binding version compatibility
FFI handle lifecycle
```

---

# 384. Kani Candidate

capability-scope and handle-lifecycle invariants.

---

# 385. TLA+ Candidate

register → grant → certify → use → revoke → re-certify.

---

# 386. Loom Candidate

concurrent event subscription + cancellation + session revocation.

---

# 387. Performance

SDK overhead should be bounded.

---

# 388. Target:

```text
minimal copies
bounded allocations
no unnecessary JSON
stream large payloads
```

---

# 389. Hard rule.

---

# 390. Serialization

Internal SDK boundary:

```text
typed Rust in-process
Postcard over local IPC where appropriate
JSON only external interoperability when necessary
```

---

# 391. Hard rule.

---

# 392. SDK IPC

Versioned Postcard envelope.

---

# 393. Hard rule.

---

# 394. Storage

Separate:

```text
SDK registry
binding registry
integration registry
capability grants
certifications
compatibility manifests
```

---

# 395. No user/developer behavior warehouse.

---

# 396. Hard rule.

---

# 397. Partitioning

By:

```text
SDK version
integration
binding class
capability
certification
```

---

# 398. No user/person analytics partition.

---

# 399. Hard rule.

---

# 400. Crate Layout

Recommended:

```text
crates/
├── siar-sdk-core/
├── siar-sdk-client/
├── siar-sdk-admin/
├── siar-sdk-ffi/
├── siar-sdk-idl/
├── siar-sdk-codegen/
├── siar-integration-registry/
├── siar-integration-certification/
├── siar-sdk-observability/
└── siar-sdk-testkit/
```

---

# 401. `siar-sdk-core`

Owns:

```text
stable DTOs
commands
events
errors
capabilities
```

---

# 402. `siar-sdk-client`

Canonical Rust client API.

---

# 403. `siar-sdk-admin`

Explicit privileged/admin SDK.

---

# 404. `siar-sdk-ffi`

Stable C ABI.

---

# 405. `siar-sdk-idl`

Language-neutral binding contract.

---

# 406. `siar-sdk-codegen`

Deterministic binding generation.

---

# 407. `siar-integration-registry`

Integration identity/manifests/capabilities.

---

# 408. `siar-integration-certification`

Conformance/certification/revocation.

---

# 409. `siar-sdk-observability`

Aggregate SDK/integration health only.

---

# 410. `siar-sdk-testkit`

binding/conformance/security/privacy tests.

---

# 411. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Public SDK models, DTOs, commands, events, and errors are stable contract types and never direct aliases of database rows, actor messages, transport internals, or secret-bearing domain structures.
2. SDK and integration access is capability-scoped, least-privilege, attenuable, expiring where appropriate, and cannot silently acquire administrator, message-content, diagnostics, or bulk-export authority.
3. Generated bindings derive from a canonical versioned SDK/IDL contract with reproducible provenance; hand-written wrappers must pass the same semantic conformance suite.
4. FFI boundaries use opaque handles, explicit ownership/lifetime/freeing rules, stable integer widths, explicit nullability, and never expose Rust memory layout across language boundaries.
5. SDK compatibility is declared explicitly against core/protocol versions and enforced at runtime; semver alone cannot imply wire or security compatibility.
6. Third-party integrations declare capabilities/data classes/network/file access, and any material scope expansion requires renewed authorization and, when applicable, re-certification.
7. Privileged integrations may require active certification, and suspended/revoked certification or credentials immediately stop authorizing production access without deleting historical evidence.
8. SDK defaults minimize privacy exposure: no raw IP/topology/internal device graph/private audit data, no broad contact enumeration, and no detailed peer/software fingerprinting in anonymous modes.
9. SDK local-first semantics distinguish local persistence, transport send, remote delivery, and read state so applications cannot accidentally present queued data as delivered.
10. Developer-platform telemetry is aggregate by SDK/binding/operation/error/version class and cannot become user-activity tracking, developer productivity analytics, or integration-based private-content surveillance.
11. SDK packages, generated bindings, code generators, and reference artifacts are signed/provenance-linked/SBOM-aware and participate in supply-chain, assurance, compatibility, and lifecycle governance.
12. The developer platform integrates with authorization, plugins, compatibility, product lifecycle, requirements, architecture governance, knowledge traceability, assurance/certification archives, risk/PIR, release/update control, and audit/compliance without creating a side channel around SIAR's security, privacy, anonymity, or tenant boundaries.
```

---

# 412. Initial Production Scope

Implement first:

```text
canonical Rust SDK
stable SDK DTOs/commands/events/errors
opaque SDK identifiers
capability-scoped sessions
local IPC client
versioned C ABI
FFI ownership rules
SDK IDL
deterministic Kotlin binding generation
Android JNI wrapper
binding provenance
SDK compatibility manifests
integration registry
integration capability manifests
scoped developer credentials
integration revocation
developer sandbox
reference Rust/Kotlin examples
conformance testkit
privileged integration certification
signed SDK packages
privacy-safe SDK observability
```

Then add:

```text
Swift binding
Java/C# bindings
optional JS/Python integrations
WASM plugin SDK
automated SDK migration tooling
cross-language conformance matrix
portable integration certification bundles
formal capability-delegation verification
```

---

# 413. Definition of Done

Part 130 is complete when:

- canonical SDK surface is separate from internal domain/storage models;
- capabilities scope every sensitive SDK action;
- SDK/core/protocol compatibility is explicit;
- C ABI ownership/lifetime is specified;
- generated bindings have deterministic provenance;
- cross-language semantics are conformance-tested;
- integration manifests declare requested capabilities/data classes;
- credentials are scoped/rotatable/revocable;
- privileged integration certification exists;
- revoked integrations cannot retain production access;
- local-first delivery semantics remain truthful;
- SDK default APIs minimize metadata/privacy exposure;
- anonymous mode cannot expose extra fingerprinting detail;
- packages have provenance/signing/SBOM integration;
- no user/developer surveillance metrics are created;
- binding/capability/privacy/fuzz/formal tests are specified.

---

# 414. Final Architecture

```text
                     SIAR CORE
                        │
                        ▼
                  STABLE SDK MODEL
                        │
            ┌───────────┼───────────┐
            │           │           │
        RUST SDK      C ABI       SDK IDL
            │           │           │
            │           └──────┬────┘
            │                  ▼
            │           GENERATED BINDINGS
            │                  │
            └───────────┬──────┘
                        ▼
                  INTEGRATIONS/APPS
                        │
                        ▼
                CONFORMANCE/CERTIFICATION
                        │
                        ▼
                  RUNTIME ADMISSION
```

Developer-platform safety model:

```text
stable intent-oriented SDK
+
capability-scoped access
+
opaque identifiers
+
versioned compatibility
+
safe C ABI
+
deterministic generated bindings
+
integration certification
+
privacy-minimized observability
```

not:

```text
expose internal database structs, hand out permanent global API keys, let plugins open arbitrary internal APIs, and track users through third-party integrations
```

---

# 415. Final Principle

A developer platform is trustworthy when third-party software can build useful integrations without gaining more authority or metadata than the product itself intends to expose.

The correct model is:

```text
define a stable SDK boundary
+
expose intent rather than internals
+
scope every capability
+
generate bindings reproducibly
+
verify semantic parity
+
certify privileged integrations
+
revoke safely
+
preserve local-first truth
+
minimize metadata exposure
+
never let SDK convenience bypass privacy or security
```

This architecture gives SIAR a privacy-preserving developer-platform foundation for Rust SDKs, generated bindings, C ABI/FFI, client libraries, integration capabilities, developer compatibility, package provenance, conformance testing, and integration certification while preserving the anonymity, local-first, least-authority, product-lifecycle, compatibility, and anti-surveillance guarantees established across Parts 34–129.
