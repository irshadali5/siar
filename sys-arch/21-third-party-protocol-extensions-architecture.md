# Part 21 — Third-Party Protocol Extensions Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 21 of 24  
**Primary language:** Rust  
**Primary goals:** safe third-party protocol extensibility, versioned namespaced extensions, capability negotiation, lifecycle isolation, bounded resources, permissioned access, crash containment, stable wire contracts, interoperability, conformance tooling, and prevention of policy bypass

---

# 1. Purpose

A reusable communication platform becomes much more valuable when external developers can add protocol features without forking the core.

Possible third-party extensions include:

```text
collaborative editing
whiteboard
game state sync
ERP notifications
document workflows
IoT telemetry
task coordination
custom file metadata
organization commands
specialized emergency workflows
remote-control protocols
application-specific sync
```

But arbitrary extension code creates serious risks:

```text
wire incompatibility
resource exhaustion
identity bypass
privacy leaks
unsafe storage access
protocol collisions
crash propagation
priority abuse
unbounded queues
plugin lock-in
```

Therefore third-party protocol support must be explicitly constrained.

The core rule is:

> **Extensions may add protocol semantics, but they must not bypass the platform’s identity, routing, capability, resource, security, crash-recovery, or emergency-policy boundaries.**

---

# 2. Architectural Position

```text
Third-Party Extension
        ↓
Extension SDK
        ↓
Extension Host Boundary
        ↓
Protocol Extension Runtime
        ↓
Core Services
├── Identity
├── Capability Negotiation
├── Routing
├── Resource Limits
├── Storage
├── Event Log
├── Diagnostics
└── Transport
```

Extensions do not talk directly to:

```text
raw sockets
database files
identity private keys
unbounded task spawning
```

unless explicitly granted through constrained APIs.

---

# 3. Extension Categories

Recommended categories:

```rust
pub enum ExtensionKind {
    ApplicationProtocol,
    DataSync,
    FileMetadata,
    ControlPlane,
    Notification,
    Collaboration,
    DeviceIntegration,
    Experimental,
}
```

This classification helps permission policy and review.

---

# 4. Namespaced Extension IDs

Every extension must have a globally collision-resistant identifier.

Example:

```text
org.example.whiteboard
com.company.erp.events
dev.irshad.customsync
```

Represent internally:

```rust
pub struct ExtensionId(String);
```

Validate:

```text
ASCII/UTF-8 policy
max length
reverse-DNS style
no reserved core prefixes
```

---

# 5. Reserved Namespaces

Core reserves:

```text
core.*
iroh.*
comm.*
system.*
```

Third parties cannot register these.

---

# 6. Extension Protocol Name

Wire protocol example:

```text
ext/org.example.whiteboard/1
```

where:

```text
extension namespace = org.example.whiteboard
major version = 1
```

---

# 7. Semantic Versioning

Separate:

```text
extension package version
wire protocol version
storage schema version
SDK compatibility version
```

Do not conflate them.

---

# 8. Wire Major Version

Breaking wire change:

```text
major++
```

Examples:

```text
message layout incompatible
state machine incompatible
semantics changed
```

---

# 9. Wire Minor Version

Additive compatible capabilities may use:

```text
minor capability flags
```

without changing protocol major.

---

# 10. Extension Manifest

Every extension provides a manifest.

```rust
pub struct ExtensionManifest {
    pub id: ExtensionId,
    pub display_name: String,
    pub package_version: Version,
    pub protocol_versions: Vec<ProtocolVersion>,
    pub sdk_requirement: SdkRequirement,
    pub permissions: Vec<ExtensionPermission>,
    pub resource_profile: ExtensionResourceProfile,
    pub capabilities: Vec<ExtensionCapability>,
}
```

---

# 11. Manifest Must Be Declarative

The runtime should know before loading:

```text
what extension wants
what protocols it exposes
what resources it requests
what permissions it needs
```

Do not discover everything only after executing extension code.

---

# 12. Permission Model

Extensions request explicit capabilities.

```rust
pub enum ExtensionPermission {
    SendProtocolFrames,
    ReceiveProtocolFrames,
    ReadOwnState,
    WriteOwnState,
    ReadPeerIdentityBasic,
    ReadTrustedContactMetadata,
    UseFiles,
    UseDtn,
    UseProximity,
    UseDiagnostics,
    RegisterNotifications,
    BackgroundExecution,
}
```

Sensitive permissions should require stronger user/admin approval.

---

# 13. No Raw Identity Keys

There is no permission:

```text
ReadPrivateIdentityKey
```

Extensions instead request:

```text
sign operation
authenticate session
```

through constrained services.

---

# 14. No Raw Database Access

Extensions do not receive:

```text
SQLite connection
DB path
internal event tables
```

They use extension-owned state APIs.

---

# 15. Extension-Owned Storage

Provide namespaced storage:

```text
extension_id
+
version
+
key/value or event namespace
```

Example:

```rust
pub trait ExtensionStateStore {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, ExtensionStateError>;
    async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), ExtensionStateError>;
}
```

---

# 16. Storage Quotas

Every extension gets:

```text
max persistent bytes
max cache bytes
max value size
max write rate
```

Part 08 enforces.

---

# 17. Storage Schema Version

Extension manifest declares:

```text
storage_schema_version
```

Migrations run only inside its namespace.

---

# 18. Migration Failure Isolation

If one extension migration fails:

```text
disable that extension
```

Do not corrupt core startup.

---

# 19. Extension State Is Not Core State

Core should not require third-party extension state to recover messaging/files/identity.

This preserves modularity.

---

# 20. Extension Lifecycle

```rust
pub enum ExtensionLifecycleState {
    Installed,
    Disabled,
    Starting,
    Running,
    Degraded,
    Stopping,
    Failed,
    Quarantined,
}
```

---

# 21. Lifecycle Hooks

Conceptually:

```rust
pub trait ProtocolExtension {
    async fn start(&mut self, ctx: ExtensionContext) -> Result<(), ExtensionError>;
    async fn stop(&mut self) -> Result<(), ExtensionError>;
    async fn on_session(&mut self, session: ExtensionSession) -> Result<(), ExtensionError>;
}
```

Actual plugin/module runtime may use FFI/WASM/IPC rather than Rust trait directly.

---

# 22. Extension Context

Only expose approved services.

```rust
pub struct ExtensionContext {
    pub identity: ExtensionIdentityApi,
    pub storage: ExtensionStateApi,
    pub transport: ExtensionTransportApi,
    pub files: Option<ExtensionFilesApi>,
    pub diagnostics: ExtensionDiagnosticsApi,
}
```

No access to internal runtime structs.

---

# 23. Capability Negotiation

Part 07 is mandatory.

Before using extension:

```text
local extension installed
remote extension supported
wire version compatible
permissions/policy allow
```

Only then open extension protocol.

---

# 24. Capability Advertisement

Peer advertises:

```text
extension ID
supported major/minor
optional features
```

through bounded capability metadata.

---

# 25. Unknown Extension

If peer advertises unsupported extension:

```text
ignore safely
```

unless it is marked required for a specific application action.

---

# 26. Required Extension

Application operation may say:

```text
requires org.example.whiteboard/1
```

If peer lacks it:

```text
typed UnsupportedExtension
```

not generic network error.

---

# 27. Wire Isolation

Every extension gets a protocol namespace.

Do not multiplex third-party payloads into core message types without explicit framing.

---

# 28. Extension Frame Header

Conceptually:

```rust
pub struct ExtensionFrameHeader {
    pub extension_id: ExtensionIdHash,
    pub protocol_major: u16,
    pub message_type: u16,
    pub payload_len: u32,
}
```

Payload remains bounded.

---

# 29. Extension ID Hash

To reduce wire size:

```text
manifest namespace
→ stable hash/assigned compact ID
```

Need collision-safe negotiation.

Do not trust hash alone if collision ambiguity exists.

---

# 30. Payload Limits

Each extension has:

```text
max frame size
max message rate
max stream count
```

The extension cannot raise these itself beyond platform policy.

---

# 31. Streaming Extensions

Large data should use:

```text
stream API
file/blob subsystem
```

rather than huge frame payloads.

---

# 32. File Reuse

Third-party extension should reuse Part 05 for:

```text
attachments
large snapshots
binary assets
```

instead of inventing another file transport.

---

# 33. DTN Reuse

If extension wants offline store-carry-forward:

```text
request UseDtn
```

and provide bounded DTN envelope semantics.

Do not implement independent hidden DTN network.

---

# 34. Routing Reuse

Extensions submit:

```text
delivery requirements
```

to Part 03.

They do not choose raw network interfaces directly.

---

# 35. Multipath Reuse

Extensions may request:

```text
allow multipath
redundancy preference
```

but Part 12 decides actual paths.

---

# 36. Battery Policy Reuse

Extension background work is subject to Part 13.

It cannot force continuous scanning or multiple radios.

---

# 37. Proximity Reuse

Extension can request nearby discovery through Part 14.

It does not directly control BLE/Wi-Fi APIs.

---

# 38. Emergency Priority Boundary

Third-party extensions cannot assign:

```text
AuthorityCritical
```

directly.

They may request a priority class, but Part 17 authorizes effective priority.

---

# 39. Priority Abuse Prevention

Extension manifest can declare:

```text
max requested priority
```

Example:

```text
Routine
Important
Urgent
```

Only trusted system/authority extensions may even request Critical.

---

# 40. Identity Boundary

Extensions receive only minimum peer identity information.

Default:

```text
opaque peer reference
verified/unverified state
optional display name after policy
```

Not full contact graph.

---

# 41. Peer Reference

```rust
pub struct ExtensionPeerRef {
    pub opaque_id: ExtensionScopedPeerId,
    pub trust: PeerTrustClass,
}
```

---

# 42. Extension-Scoped Peer IDs

To reduce correlation:

```text
same peer
```

may map to different opaque ID per extension.

This prevents cross-extension tracking.

---

# 43. Stable Scope

Mapping can be:

```text
HMAC(platform_secret, extension_id || DeviceId)
```

or equivalent stable privacy-preserving derivation.

Do not expose raw DeviceId unless permission requires it.

---

# 44. Contact Metadata Permission

Sensitive extension may request:

```text
ReadTrustedContactMetadata
```

User/admin should explicitly approve.

---

# 45. Message Content Access

Third-party extension should not automatically read core messaging history.

If a future API allows it:

```text
separate high-risk permission
```

not default.

---

# 46. Extension Messaging

Prefer:

```text
extension owns its own protocol payloads
```

rather than injecting arbitrary hidden metadata into core chat messages.

---

# 47. UI Integration

Part 24 plugin/module ecosystem may expose UI.

Part 21 protocol extension itself should remain UI-neutral.

---

# 48. Headless Compatibility

Every third-party protocol should be able to run without Dioxus if it claims headless support.

---

# 49. Execution Models

Possible extension implementations:

```text
native Rust crate linked at build time
WASM component
out-of-process plugin
C ABI plugin
remote service adapter
```

Part 21 defines semantics independent of implementation.

---

# 50. Built-In Extension

A first-party feature can use the same extension interface.

This is useful dogfooding.

---

# 51. Statically Linked Extension

Best performance.

Trade-off:

```text
requires application rebuild
less runtime isolation
```

Useful for trusted enterprise/custom builds.

---

# 52. WASM Extension

Potentially best for:

```text
sandboxing
portable third-party logic
resource control
```

Part 24 may define packaging/runtime details.

---

# 53. Out-of-Process Extension

Strong isolation:

```text
extension host
+
IPC
```

Good for untrusted native plugins.

---

# 54. Native In-Process Extension Risk

A native plugin loaded into daemon process can:

```text
crash process
read memory
bypass sandbox
```

Therefore only trusted/signed extensions should run in-process.

---

# 55. Trust Levels

```rust
pub enum ExtensionTrustLevel {
    BuiltIn,
    TrustedSigned,
    SandboxedThirdParty,
    DevelopmentOnly,
}
```

---

# 56. Trust Determines Execution

Example:

```text
BuiltIn → native in-process
TrustedSigned → native or WASM
SandboxedThirdParty → WASM/out-of-process
DevelopmentOnly → explicit dev mode
```

---

# 57. Extension Signing

Package may be signed.

Manifest includes:

```text
publisher identity
digest
signature
```

Runtime verifies before install/load if policy requires.

---

# 58. Publisher Identity

Separate from messaging identity.

Use:

```text
extension publisher certificate/key
```

---

# 59. Signature Does Not Mean Safe

Signed means:

```text
publisher authenticated
package untampered
```

not:

```text
bug-free
trusted with all permissions
```

Still sandbox and permission-limit.

---

# 60. Installation Policy

Consumer product can support:

```text
built-in only
allowlisted publishers
user-installable sandboxed
enterprise-managed
```

---

# 61. Enterprise Allowlist

Organization can define:

```text
allowed extension IDs
allowed publishers
max versions
permissions
```

---

# 62. Revocation

Extension can be revoked due to:

```text
security issue
publisher compromise
protocol abuse
```

Revocation policy can disable it on next config sync/start.

---

# 63. Offline Revocation

For disconnected environments:

```text
signed revocation list
```

can propagate via DTN/admin provisioning.

---

# 64. Extension Update

Update must consider:

```text
wire version
storage schema
SDK compatibility
permission changes
```

---

# 65. Permission Expansion

If update requests new permission:

```text
require re-approval
```

Do not silently grant.

---

# 66. Storage Migration

Update runs extension-local migration.

Must be:

```text
bounded
idempotent
crash-safe
```

---

# 67. Rollback

Extension rollback requires compatibility with:

```text
storage schema
wire state
```

If unsafe:

```text
refuse rollback
```

---

# 68. Extension Crash

If extension panics/fails:

```text
mark extension failed
stop its sessions
preserve core runtime
```

For in-process native plugin, full process isolation may not exist; this is why sandboxed execution is preferred.

---

# 69. Quarantine

Repeated failures:

```text
Failed
→ Quarantined
```

Requires user/admin action to re-enable.

---

# 70. Crash Recovery

Part 09 should restore:

```text
core state first
```

then extension state.

Extension recovery runs after core minimum invariants.

---

# 71. Extension Recovery Hook

```rust
pub trait RecoverableExtension {
    async fn recover(&mut self, ctx: ExtensionRecoveryContext)
        -> Result<ExtensionRecoveryReport, ExtensionError>;
}
```

---

# 72. Recovery Failure

If extension recovery fails:

```text
disable extension
core stays usable
```

---

# 73. Event Log Integration

Third-party extension may register its own durable events.

Use:

```text
extension event namespace
```

not arbitrary core event variants.

---

# 74. Extension Event Envelope

```rust
pub struct ExtensionEvent {
    pub extension: ExtensionId,
    pub schema_version: u16,
    pub event_type: u16,
    pub payload: Bytes,
}
```

---

# 75. Event Limits

Bound:

```text
event size
event rate
storage quota
```

---

# 76. Event Immutability

Once committed, extension event schema meaning should remain stable.

Use migration/upcast policy.

---

# 77. Projection Support

Extensions can maintain namespaced projections.

They cannot mutate core projections directly.

---

# 78. Snapshot Support

Extension may create:

```text
own snapshot/checkpoint
```

under quota.

---

# 79. Diagnostics Integration

Extension can publish:

```text
health
reason codes
basic metrics
```

through namespaced diagnostic API.

---

# 80. Diagnostic Namespace

Example:

```text
EXT-org.example.whiteboard-001
```

Avoid collisions with core reason codes.

---

# 81. Diagnostic Privacy

Extension diagnostics also pass through Part 18 redaction.

---

# 82. Logging

Extensions should use structured host logging API.

Do not write arbitrary unbounded files.

---

# 83. Log Quota

Rate-limit extension logs.

Repeated spam can be dropped/coalesced.

---

# 84. Metrics

Allow:

```text
counters
gauges
histograms
```

with bounded cardinality.

No raw peer IDs in metric labels.

---

# 85. Resource Profile

Manifest example:

```rust
pub struct ExtensionResourceProfile {
    pub max_memory_bytes: u64,
    pub max_persistent_bytes: u64,
    pub max_concurrent_sessions: u16,
    pub max_streams: u16,
    pub max_tasks: u16,
}
```

Host clamps to global maximum.

---

# 86. Dynamic Resource Budget

Part 08 may reduce extension budget under:

```text
battery saver
memory pressure
emergency mode
```

---

# 87. CPU Budget

For sandboxed runtime:

```text
fuel
instruction quota
time slice
```

where execution engine supports.

---

# 88. Network Budget

Per extension:

```text
bytes/sec
streams
connection attempts
DTN bytes
```

---

# 89. Storage Budget

Separate:

```text
persistent
cache
temporary
```

---

# 90. Background Budget

Extensions cannot assume continuous execution.

Part 13/16 lifecycle rules apply.

---

# 91. Mobile Restrictions

On Android/iOS:

```text
extension background work
```

is subject to system-approved execution windows.

---

# 92. Extension Session

```rust
pub struct ExtensionSession {
    pub session_id: ExtensionSessionId,
    pub peer: ExtensionPeerRef,
    pub protocol_version: ProtocolVersion,
    pub transport: ExtensionTransportHandle,
}
```

---

# 93. Session Authentication

Core authenticates peer before extension session if extension requires identity.

Extension can declare:

```text
anonymous allowed
verified peer required
trusted contact required
```

---

# 94. Authentication Requirement

Manifest:

```rust
pub enum ExtensionAuthRequirement {
    AnonymousAllowed,
    VerifiedDevice,
    TrustedContact,
    OrganizationMember,
}
```

---

# 95. Anonymous Protocols

If allowed:

```text
strict quotas
small payloads
no privileged priority
```

---

# 96. Session Authorization

Core can reject session based on:

```text
peer trust
extension policy
organization policy
user blocklist
```

---

# 97. Session Limits

Per extension/per peer:

```text
max active sessions
max streams
```

---

# 98. Protocol State Machine

Every extension must define explicit state machine.

Example:

```text
Opening
Ready
Closing
Closed
```

Application-specific states inside.

---

# 99. State Machine Test Requirement

Extension cannot be considered stable without:

```text
valid transition tests
invalid transition tests
fuzzing
```

---

# 100. Extension SDK

Provide Rust SDK:

```text
comm-extension-sdk
```

with:

```text
manifest types
protocol APIs
storage API
capability API
diagnostics API
test kit
```

---

# 101. SDK Stability

Extension SDK has its own semver.

Do not expose unstable internal runtime APIs.

---

# 102. Minimal SDK Surface

Start small.

Good:

```text
send frame
open stream
store state
publish event
query peer trust
register diagnostics
```

Avoid giant omnipotent SDK.

---

# 103. Extension Context Capability Tokens

Internally, APIs can be capability objects.

If extension lacks permission:

```text
field/API absent
```

or returns `PermissionDenied`.

Capability-based design is preferable to a global god-object.

---

# 104. No Arbitrary Thread Spawn

Extensions should use host task scheduler.

```rust
ctx.spawn_task(...)
```

subject to quotas.

---

# 105. Task Supervision

Every extension task belongs to:

```text
extension session
or
extension lifecycle
```

Host can cancel all on disable/unload.

---

# 106. Timer API

Provide host timer.

Avoid extension creating uncontrolled busy loops.

---

# 107. Clock API

Use host clock abstraction.

This improves deterministic testing.

---

# 108. Randomness API

For non-crypto random needs:

```text
host random
```

For crypto:

```text
use vetted crypto APIs
```

Do not expose raw platform entropy handles unnecessarily.

---

# 109. Cryptography API

Extension should not invent security casually.

Provide helper services for:

```text
hash
signature verification
session-bound authentication
```

where practical.

---

# 110. Extension E2EE

If extension payload rides an already secure peer session:

```text
transport/session encryption
```

may be sufficient.

If extension has special end-to-end semantics:

```text
define explicitly
```

and undergo security review.

---

# 111. Secret Storage

Extensions may request namespaced secret storage.

Example:

```text
extension API token
private extension state
```

But never core identity key material.

---

# 112. Secret Store Permission

Separate:

```text
ExtensionSecretStore
```

from ordinary state storage.

---

# 113. Secret Export

Host should minimize raw secret reads where handle-based use is possible.

---

# 114. File Access

Extensions should not receive arbitrary filesystem access.

Use:

```text
file picker result
blob handle
sandboxed storage directory
```

---

# 115. Sandbox Directory

If native/sandbox runtime needs files:

```text
extensions/<id>/
```

with quota and permissions.

---

# 116. Path Traversal Protection

Extension file paths are relative and normalized.

Reject:

```text
../
absolute path
symlink escape
```

---

# 117. UI Permission

Part 24 may define:

```text
RegisterUiPanel
```

separately from protocol permissions.

---

# 118. Notification Permission

Extension can request:

```text
RegisterNotifications
```

but host controls OS notification priority.

---

# 119. Emergency Notification Restriction

Third-party notification cannot masquerade as:

```text
verified emergency authority alert
```

unless extension/publisher is explicitly trusted and policy grants it.

---

# 120. Capability Negotiation Example

Local:

```text
org.example.whiteboard/1
features:
  snapshots
  cursors
```

Remote:

```text
org.example.whiteboard/1
features:
  snapshots
```

Negotiated:

```text
snapshots
```

---

# 121. Fallback

If extension unavailable:

```text
application may degrade
```

Example:

```text
send exported image instead of collaborative whiteboard
```

Fallback belongs to product/extension design.

---

# 122. Required vs Optional Capability

Manifest/protocol marks features:

```text
required
optional
```

Unknown required capability:

```text
fail negotiation
```

---

# 123. Interoperability

Third-party protocol should be implementable by another language/runtime if publisher wants.

Publish:

```text
wire spec
state machine
test vectors
error codes
```

---

# 124. Conformance Suite

Part 23 infrastructure should allow extension-specific conformance packages.

Example:

```text
conformance/extensions/org.example.whiteboard/
```

---

# 125. Golden Vectors

Stable extension should ship:

```text
valid frames
invalid frames
canonical encodings
state-machine examples
```

---

# 126. Fuzz Targets

Required for parsers.

Examples:

```text
extension frame
manifest payload
state snapshot
```

---

# 127. Resource Abuse Tests

Simulate extension sending:

```text
huge frame
too many streams
too many events
too much storage
```

Host must contain it.

---

# 128. Crash Tests

Kill extension host/process during:

```text
state write
stream
migration
```

Core remains correct.

---

# 129. Version Compatibility Tests

Test:

```text
v1 ↔ v1
v1 ↔ v1.1
v1 ↔ unsupported v2
```

---

# 130. Extension Disable Mid-Session

Expected:

```text
cancel extension tasks
close extension streams
persist safe state
release resources
```

Core peer connection may remain.

---

# 131. Peer Lacks Extension

Opening extension protocol returns:

```text
UnsupportedByPeer
```

not crash.

---

# 132. Extension Not Installed

Remote request:

```text
reject/ignore cleanly
```

---

# 133. Extension Permission Revoked

If user/admin revokes:

```text
stop affected sessions
```

and require re-approval to resume.

---

# 134. Permission Changes During Update

New permission request blocks update activation until accepted.

---

# 135. Extension Dependencies

Avoid arbitrary extension-to-extension dependency chains initially.

If needed later:

```text
declare explicit dependency IDs/version ranges
```

---

# 136. Dependency Cycle

Reject:

```text
A requires B
B requires A
```

unless host supports cycle semantics.

Simpler:

```text
acyclic only
```

---

# 137. Core Dependency

Manifest states minimum platform SDK/runtime version.

---

# 138. Optional Dependency

Extension may integrate with:

```text
files
DTN
proximity
```

if host build supports.

---

# 139. Feature Degradation

If host lacks optional subsystem:

```text
extension still loads
feature disabled
```

---

# 140. Extension Marketplace Is Separate

Part 21 defines protocol/runtime contract.

Distribution marketplace/repository belongs to Part 24 ecosystem.

---

# 141. Development Mode

Developers need:

```text
unsigned local extension
hot reload
verbose diagnostics
```

only in explicit dev mode.

---

# 142. Dev Mode Warning

Never silently allow unsigned extension loading in production mode.

---

# 143. Hot Reload

Useful for sandboxed extension development.

Production reload must still preserve lifecycle correctness.

---

# 144. Extension Debug API

Developer can inspect:

```text
state
sessions
resource use
diagnostics
```

not other extension secrets.

---

# 145. Extension Doctor

CLI:

```text
comm extension doctor org.example.whiteboard
```

Checks:

```text
manifest
permissions
protocol registration
storage
compatibility
```

---

# 146. Extension List

CLI:

```text
comm extension list
```

Shows:

```text
installed
version
status
permissions
```

---

# 147. Extension Enable/Disable

Admin/user:

```text
comm extension disable <id>
comm extension enable <id>
```

subject to policy.

---

# 148. Extension Uninstall

Uninstall flow:

```text
stop
revoke sessions
optional export
remove package
handle extension state according to policy
```

---

# 149. State Retention on Uninstall

Options:

```text
keep state
delete state
export state
```

Product/admin policy decides.

---

# 150. Secure Deletion

For secret extension state:

```text
best-effort secure deletion
```

subject to filesystem realities.

---

# 151. Extension Package

Conceptual package:

```text
manifest.ron
module.wasm / native library
signature
assets
schemas
test-vectors
```

Exact packaging belongs to Part 24.

---

# 152. Manifest Format

RON is appropriate for local package metadata:

```text
human-readable
Rust-friendly
versioned
```

Wire protocol remains binary.

---

# 153. Package Digest

Manifest/signature binds:

```text
all executable assets
protocol schemas
```

---

# 154. Reproducible Extension Builds

Recommended for trusted ecosystem.

Publish:

```text
source commit
build metadata
digest
```

---

# 155. SBOM

Extension package should include dependency/license metadata where possible.

---

# 156. License Policy

Host may enforce:

```text
allowed licenses
```

for enterprise distribution.

Protocol compatibility is independent of package license.

---

# 157. Sandbox Execution

For third-party code, preferred:

```text
WASM component
or
out-of-process host
```

with:

```text
memory cap
CPU/time cap
no ambient filesystem
no ambient network
```

---

# 158. No Ambient Authority

Sandbox principle:

```text
extension gets only capability handles explicitly granted
```

This is a major security goal.

---

# 159. Native Plugin Exception

Native in-process extension only for:

```text
built-in
trusted signed
explicit deployment policy
```

because it can bypass memory/process isolation.

---

# 160. Out-of-Process IPC

Extension host communicates through:

```text
versioned bounded local IPC
```

similar Part 16 principles.

---

# 161. Host Crash

Extension host process crash:

```text
daemon continues
extension marked failed
```

---

# 162. Host Restart

May restart sandboxed host with backoff.

Persistent extension state remains in core-owned store.

---

# 163. Secret Isolation

Out-of-process extension only receives scoped secret handles/data explicitly granted.

---

# 164. Extension Network Access

Do not grant raw Internet socket access by default.

Extension uses:

```text
peer protocol API
```

If external HTTP/API access is needed later:

```text
separate permission
```

---

# 165. External Network Permission

Possible:

```text
ExternalNetworkAccess(domains/policy)
```

but high-risk.

Not necessary for v1 protocol extensions.

---

# 166. Extension-to-Server Architecture

If extension needs its own backend:

```text
application may support external service
```

but this should not be hidden inside unrestricted raw sockets.

---

# 167. Transport-Independent Design

Extension protocol should not assume:

```text
Iroh only
BLE only
relay only
```

Core delivery may change path.

---

# 168. Session Reconnect

Extension should tolerate:

```text
path switch
connection reconnect
```

without corrupting logical state.

---

# 169. Stable Operation IDs

Extension long-lived operations should use stable IDs.

```rust
pub struct ExtensionOperationId([u8; 16]);
```

---

# 170. Duplicate Delivery

Extension protocol must define idempotency where retries are possible.

---

# 171. Crash Recovery Contract

Extension must declare which operations are:

```text
durable
ephemeral
reconstructible
```

---

# 172. Durable Intent

If extension says operation accepted:

```text
persist before success
```

same Part 09 principle.

---

# 173. Ephemeral State

Examples:

```text
cursor position
typing
live pointer
```

can be dropped on restart.

---

# 174. Extension Priority Classes

By default:

```text
Routine
Important
```

Higher classes require policy.

---

# 175. Extension Scheduling

Work descriptor includes:

```text
priority
energy class
deadline
durability
```

host maps to Part 08/13 scheduler.

---

# 176. Extension Backpressure

Send API can return:

```text
WouldBlock
Deferred
QuotaExceeded
```

Extension must handle it.

---

# 177. No Hidden Infinite Retry

Extension should not internally busy-loop on quota denial.

Host retry scheduler can be used.

---

# 178. Queue Ownership

Extension-owned queues are bounded and visible to host resource accounting.

---

# 179. Extension Memory

Sandboxed runtime memory cap is hard.

Native trusted extension still receives logical allocation budgets.

---

# 180. Diagnostics Reason

If extension disabled by battery/resource policy:

```text
structured reason
```

available to UI/CLI.

---

# 181. User Consent

First install may show:

```text
This extension can:
- communicate with peers
- store up to 100 MB
- access nearby devices
```

No vague "full access" where avoidable.

---

# 182. Permission UX

Group permissions by meaningful capability, not low-level implementation detail.

---

# 183. Enterprise Policy UX

Managed organization can lock:

```text
mandatory
optional
forbidden
```

extensions.

---

# 184. Mandatory Extension Risk

Even mandatory extension failure should not corrupt core.

Application feature may become unavailable, but daemon remains healthy.

---

# 185. Wire Registration

Extension protocol registry:

```rust
pub struct ExtensionProtocolRegistry {
    // id → descriptor
}
```

Registration checks:

```text
namespace
version overlap
duplicate IDs
permissions
resource profile
```

---

# 186. Duplicate Registration

Two extensions claim same ID:

```text
reject second
```

---

# 187. Publisher Takeover

Namespace ownership alone is not globally enforceable without registry/signing infrastructure.

For official ecosystem, bind:

```text
ExtensionId
↔ publisher key
```

after first trusted publication.

---

# 188. Trust-on-First-Publisher Policy

Official catalog may pin publisher key for an extension ID.

Future package signed by different publisher:

```text
reject unless transfer process
```

---

# 189. Namespace Transfer

Should require explicit signed handoff from old publisher or catalog/admin override.

---

# 190. Protocol Security Review

Extensions handling:

```text
identity
money
remote control
emergency
```

deserve stricter review.

---

# 191. Risk Classes

```rust
pub enum ExtensionRiskClass {
    Low,
    Medium,
    High,
    Critical,
}
```

Risk derived from requested permissions and semantics.

---

# 192. High-Risk Extension

May require:

```text
manual install
trusted publisher
security audit
native execution prohibited
```

---

# 193. Critical Extension

Examples:

```text
authority alerts
remote device control
financial approval
```

Should not be generally third-party-installable without strong governance.

---

# 194. Protocol Fuzzing

Part 10 infrastructure can dynamically register extension fuzz targets/test corpora.

---

# 195. Conformance Metadata

Manifest may point to:

```text
test vector version
conformance suite version
```

---

# 196. Release Gate for Stable Extension

Must have:

```text
wire spec
state machine spec
golden vectors
fuzz coverage
resource tests
compatibility tests
```

---

# 197. External Implementation

A publisher can implement same extension protocol in:

```text
Rust
C++
Kotlin
Go
```

as long as wire conformance passes.

---

# 198. Extension SDK for Other Languages

Part 19 C ABI can expose generic extension API.

Do not force extension authors to use Rust.

---

# 199. C ABI Extension API

Conceptually:

```c
comm_result_t comm_extension_open(
    comm_runtime_handle_t runtime,
    comm_bytes_view_t extension_id,
    const comm_peer_ref_t* peer,
    comm_extension_session_handle_t* out_session
);
```

---

# 200. Generic Frame Send

```c
comm_result_t comm_extension_send(
    comm_extension_session_handle_t session,
    uint16_t message_type,
    comm_bytes_view_t payload
);
```

Still bounded by host.

---

# 201. WASM Component API

Future component interface can map same semantics:

```text
open session
send
receive
state get/put
diagnostics
```

---

# 202. Protocol Extension vs Plugin

Important distinction:

## Protocol extension

```text
defines peer-to-peer wire semantics
```

## Plugin

```text
extends local product behavior/UI/workflow
```

Part 24 may combine them, but they are conceptually different.

---

# 203. Protocol Extension Without Plugin

A built-in application may support external wire protocol extension statically.

---

# 204. Plugin Without Protocol Extension

A UI plugin may only add:

```text
local automation
```

with no new wire protocol.

---

# 205. Extension Revocation Mid-Session

Host:

```text
close extension streams
cancel extension tasks
persist safe state
```

Core transport may continue.

---

# 206. Peer Revocation

If peer DeviceId revoked:

```text
all extension sessions close
```

automatically.

---

# 207. Identity Change

Extension cannot keep stale authenticated session after account/device identity invalidates.

---

# 208. Capability Change

If remote extension capability disappears after reconnect:

```text
operation degrades/fails cleanly
```

---

# 209. Path Change

Extension session may migrate over new transport without protocol restart if runtime supports.

---

# 210. Offline Mode

Extensions can operate:

```text
LAN
BLE
DTN
```

if their delivery requirements allow.

---

# 211. DTN-Safe Extension Requirement

To use DTN, extension must declare:

```text
payload idempotency
expiry
size bound
replication semantics
```

---

# 212. Realtime Extension

Realtime protocols should declare:

```text
DTN not allowed
low latency
ephemeral
```

---

# 213. File-Oriented Extension

Should reference `BlobId` rather than embedding large bytes.

---

# 214. Extension Protocol Example

Whiteboard:

```text
extension: org.example.whiteboard
wire: v1

frames:
OpenBoard
StrokeBatch
Cursor
SnapshotRef
CloseBoard
```

Use:

```text
Cursor → ephemeral
StrokeBatch → durable/idempotent
SnapshotRef → Part 05 blob
```

---

# 215. ERP Example

```text
extension: com.school.erp.notifications
```

Frames:

```text
InvoiceReady
AttendanceAlert
DocumentRequest
```

Could work as reusable application protocol without modifying core messaging.

---

# 216. IoT Example

```text
extension: org.example.sensor
```

Frames:

```text
TelemetryBatch
Command
Ack
```

Resource quotas prevent runaway sensor streams.

---

# 217. Remote Control Warning

Extensions that actuate devices require:

```text
strong authorization
replay protection
audit
rate limits
```

High-risk.

---

# 218. Extension Audit Events

For privileged extension:

```text
installed
enabled
permission changed
publisher changed
security failure
```

recorded in admin audit.

---

# 219. User Data Export

Extension should provide export hook for its own durable user state if meaningful.

---

# 220. Data Portability

Avoid extension lock-in.

Stable extension state export can use:

```text
RON/JSON/archive
```

depending audience.

---

# 221. Uninstall Warning

If deleting extension data:

```text
show size
export option
```

where user-facing product supports.

---

# 222. Extension Backup

Core backup service can include extension state namespace.

Secrets handled according to secure-store policy.

---

# 223. Restore

Restore extension state only if:

```text
extension/version compatible
```

Otherwise keep quarantined data until extension installed.

---

# 224. Unknown Extension State

Do not delete automatically just because extension temporarily missing.

Policy can retain.

---

# 225. Orphan Cleanup

Long-uninstalled extension state may be garbage-collected after:

```text
retention period
explicit consent/admin policy
```

---

# 226. Multi-Tenant Extension State

Headless enterprise mode namespaces:

```text
TenantId
+
ExtensionId
```

---

# 227. Per-Tenant Permission

Extension allowed for tenant A may be forbidden for tenant B.

---

# 228. Resource Fairness

One extension/tenant cannot starve others.

Hierarchical Part 08 quotas:

```text
global
→ tenant
→ extension
→ peer/session
```

---

# 229. Security Invariants

1. Extensions cannot read core private keys.
2. Extensions cannot mutate core DB directly.
3. Extensions cannot open raw unbounded network paths by default.
4. Unknown extension IDs are ignored/rejected safely.
5. Resource usage is bounded per extension.
6. Priority requests are re-authorized by core.
7. Peer identity/session policy remains core-owned.
8. Extension crash/recovery failure cannot corrupt core state.
9. Permission expansion requires approval.
10. Extension storage is namespaced.
11. Stable wire versions are explicit.
12. Native untrusted code is not loaded in-process by default.
13. Extension diagnostics and telemetry obey privacy redaction.
14. Extension disable/uninstall releases tasks, streams, and handles.
15. Core remains usable when an optional extension fails.

---

# 230. Suggested Workspace

```text
crates/
├── comm-extension-core/
├── comm-extension-sdk/
├── comm-extension-registry/
├── comm-extension-storage/
├── comm-extension-host/
├── comm-extension-security/
├── comm-extension-testkit/
└── comm-extension-ffi/

extensions/
├── built-in/
└── examples/

conformance/
└── extensions/
```

---

# 231. `comm-extension-core`

Responsibilities:

```text
IDs
manifest
permissions
lifecycle
protocol registration
capabilities
```

---

# 232. `comm-extension-sdk`

Public developer API.

---

# 233. `comm-extension-registry`

Tracks:

```text
installed
enabled
version
publisher
trust
```

---

# 234. `comm-extension-storage`

Namespaced state/secret store.

---

# 235. `comm-extension-host`

Runs:

```text
WASM
out-of-process
trusted native adapters
```

---

# 236. `comm-extension-security`

Handles:

```text
signature verification
publisher trust
permission checks
revocation
```

---

# 237. `comm-extension-testkit`

Provides:

```text
fake peer
fake clock
resource limiter
protocol harness
golden-vector runner
```

---

# 238. `comm-extension-ffi`

Generic cross-language extension API from Part 19.

---

# 239. Developer Workflow

```text
create manifest
define wire protocol
define state machine
implement extension
run local testkit
run fuzzing
run conformance
package/sign
install in dev mode
```

---

# 240. Extension Template

Provide:

```text
cargo generate
```

or template repo with:

```text
manifest.ron
src/lib.rs
tests/
vectors/
README.md
```

---

# 241. Documentation Requirements

Every extension should document:

```text
purpose
protocol ID
wire versions
permissions
resource usage
security model
offline behavior
DTN behavior
storage schema
compatibility policy
```

---

# 242. Stable Protocol Spec

Publish separately from implementation where interoperability matters.

---

# 243. Initial Production Scope

Implement first:

```text
ExtensionId namespace
manifest
permission model
protocol registry
capability negotiation integration
extension-owned storage
resource quotas
lifecycle
built-in/static Rust extension support
sandbox-ready host abstraction
generic diagnostics
testkit
```

Then:

```text
WASM host
out-of-process native host
signed packages
publisher trust
generic C ABI extension API
extension conformance packages
```

Defer initially:

```text
public marketplace
arbitrary native in-process third-party plugins
external raw network permissions
complex extension dependency graphs
```

---

# 244. Implementation Phases

## Phase 1 — Core Types

```text
ExtensionId
Manifest
Permission
Lifecycle
```

## Phase 2 — Protocol Registry

```text
namespace
versions
capabilities
session open/close
```

## Phase 3 — Storage / Resources

```text
state namespace
quota
task/stream limits
```

## Phase 4 — Built-In SDK

```text
Rust extension trait
testkit
examples
```

## Phase 5 — Security

```text
publisher identity
signatures
trust levels
revocation
```

## Phase 6 — Sandboxed Host

```text
WASM / out-of-process abstraction
```

## Phase 7 — Cross-Language

```text
Part 19 generic extension API
```

## Phase 8 — Hardening

```text
fuzz
crash
quota abuse
permission revocation
compatibility
conformance
```

---

# 245. Definition of Done

Part 21 is complete when:

- every extension has a unique namespaced ID
- wire protocol versions are explicit
- extension capabilities negotiate through Part 07
- unknown extensions fail safely
- permissions are declared before activation
- extensions cannot access core private keys or raw databases
- extension state is namespaced and quota-limited
- extension tasks/streams are supervised and bounded
- third-party priority requests are re-authorized by Part 17
- routing/multipath/DTN/proximity use core services rather than raw transport access
- extension crash/recovery failure cannot corrupt core startup
- extension disable/revocation closes its sessions safely
- permission expansion requires re-approval
- trusted vs sandboxed execution is explicit
- signed package/publisher trust can be enforced
- stable extensions ship specs, golden vectors, fuzz tests, and conformance tests
- generic SDK/FFI can support non-Rust extension authors
- core messaging/files/identity remain fully usable when optional extensions fail

---

# 246. Relationship to Earlier Parts

Part 21 builds on:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
03 — Transport & Routing Policy Engine
04 — Offline Event Log
05 — Robust File / Blob Subsystem
06 — DTN / Store-Carry-Forward
07 — Capability Negotiation
08 — Resource Limits & Backpressure
09 — Crash Recovery
10 — Fuzzing & Protocol Test Suite
11 — Relay / Self-Hosted Infrastructure
12 — Multipath Networking
13 — Battery-Aware Scheduling
14 — Proximity Abstraction
15 — QR / NFC Bootstrap
16 — Daemon & Headless Runtime
17 — Emergency Priority Classes
18 — Network Diagnostics & Path Visualization
19 — C ABI / FFI
20 — Embedded Linux Node
```

It prepares:

```text
22 — External Interoperability Suite
23 — Plugin / Module Packaging & Distribution
24 — Ecosystem Governance / Marketplace / Trust
```

---

# 247. Final Architecture

```text
                THIRD-PARTY EXTENSION
                         │
                  Extension Manifest
                         │
              Permission / Trust Check
                         │
                 Extension Host
                         │
        ┌────────────────┼────────────────┐
        │                │                │
     Storage          Protocol         Diagnostics
        │                │                │
        └──────────── Core Services ──────┘
                         │
          Identity / Routing / DTN / Files
                         │
                    Peer Network
```

Safe sandbox path:

```text
Extension Package
      ↓
Signature Verify
      ↓
WASM / Out-of-Process Host
      ↓
Capability-Scoped APIs
      ↓
Core Runtime
```

---

# 248. Final Principle

A third-party extension should be able to add something substantial, such as:

```text
collaborative whiteboarding
ERP event delivery
IoT telemetry
custom workflow synchronization
```

without being allowed to:

```text
read identity private keys
bypass routing policy
consume unlimited memory
claim emergency authority
write directly into core databases
open arbitrary raw network sockets
crash the whole daemon
```

The extension architecture should therefore provide:

```text
freedom at the protocol-semantic layer
+
strict boundaries at the security/resource/runtime layer
```

That combination is what makes the platform extensible without turning it into an unsafe plugin host.
