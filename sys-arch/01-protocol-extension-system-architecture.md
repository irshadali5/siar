# Part 01 — Protocol Extension System Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 01 of 24  
**Primary language:** Rust  
**Core transport:** Iroh-first, transport-neutral architecture  
**Primary goals:** extensibility, interoperability, safety, forward compatibility, modularity, zero unnecessary coupling

---

# 1. Purpose

The reusable communication platform must support future capabilities without turning the core protocol into a monolith.

The protocol extension system provides a controlled mechanism for adding new application-level protocols and features such as:

- messaging
- files
- group messaging
- presence
- receipts
- emergency communication
- device synchronization
- collaboration events
- ERP-specific payloads
- custom application features
- future experimental protocols

without requiring every node to implement every feature.

The extension system must preserve:

```text
compatibility
security
resource limits
version negotiation
independent deployment
capability discovery
transport neutrality
```

The goal is not a dynamic-plugin system for arbitrary untrusted code.

The goal is a **versioned protocol capability architecture**.

---

# 2. Fundamental Principle

Do not design:

```text
One giant protocol enum
├── Message
├── File
├── Presence
├── Emergency
├── Call
├── FutureFeature1
├── FutureFeature2
└── ...
```

That creates:

- central coupling
- large breaking changes
- forced feature support
- difficult third-party integration
- difficult protocol evolution

Instead:

```text
Communication Session
        │
        ├── Core Control Protocol
        │
        ├── Messaging Extension
        │
        ├── File Extension
        │
        ├── Presence Extension
        │
        ├── Emergency Extension
        │
        └── Application Extensions
```

Each extension is:

- independently identified
- independently versioned
- capability-negotiated
- size-limited
- authenticated
- transport-independent
- optional

---

# 3. Protocol Layers

```text
Application
    ↓
Extension Protocol
    ↓
Session Multiplexer
    ↓
Core Protocol
    ↓
Transport
    ↓
Iroh / LAN / Bluetooth / Wi-Fi / DTN
```

Responsibilities:

## Transport

Moves bytes.

## Core protocol

Provides:

- connection/session bootstrap
- protocol negotiation
- capabilities
- authentication binding
- extension discovery
- error framing
- flow-control metadata

## Extension protocol

Implements a reusable feature.

Examples:

```text
messaging
files
presence
groups
emergency
```

## Application protocol

Implements product-specific semantics on top of extension support.

---

# 4. Core Protocol Must Stay Small

The core protocol should know only what every peer needs.

Recommended responsibilities:

```text
Hello
ProtocolVersion
PeerIdentityBinding
CapabilityAdvertisement
ExtensionAdvertisement
ExtensionOpen
ExtensionClose
Ping/Pong
Error
Shutdown
```

It should **not** know:

```text
chat message format
file metadata
group membership
SOS payload
ERP business object
```

Those belong elsewhere.

---

# 5. Protocol Namespace

Every extension must have a stable identifier.

Recommended structure:

```text
<organization>/<protocol>/<major-version>
```

Examples:

```text
org.example.comm/messaging/1
org.example.comm/files/1
org.example.comm/presence/1
org.example.comm/emergency/1
```

Human-readable canonical names are useful for:

- documentation
- diagnostics
- interoperability
- test vectors
- configuration

A compact session-local numeric ID can be negotiated after handshake.

---

# 6. Strong Protocol Identifiers

Do not use arbitrary strings throughout the runtime.

```rust
pub struct ProtocolId {
    pub namespace: NamespaceId,
    pub protocol: ProtocolName,
    pub major: ProtocolMajor,
}
```

If hashes are later used for compact wire representation, collision handling must be explicit.

A safer initial design is:

```text
bounded canonical string during negotiation
        ↓
session-local numeric identifier afterward
```

---

# 7. Major and Minor Versions

Use explicit compatibility semantics.

## Major version

Wire-incompatible change.

```text
messaging/1
messaging/2
```

Peers sharing no major version cannot use that extension together.

## Minor version

Backward-compatible feature expansion within the same major version.

```text
Messaging v1.4
```

may communicate with:

```text
Messaging v1.2
```

using the common feature subset.

---

# 8. Capability Negotiation

Version alone is insufficient.

Messaging capabilities might include:

```text
text
reply
edit
reaction
read_receipt
typing
group_message
custom_content
```

File capabilities might include:

```text
range_resume
parallel_chunks
content_addressing
encrypted_blob
directory_manifest
```

The effective feature set is:

```text
local capabilities ∩ remote capabilities
```

---

# 9. Capability Representation

Use strong compact identifiers.

```rust
pub struct CapabilityId(pub u32);

pub struct CapabilitySet {
    pub values: Vec<CapabilityId>,
}
```

For very small capability sets, `SmallVec` may reduce allocation.

Avoid hot-path capability checks based on arbitrary strings.

Maintain a registry mapping IDs to human-readable names for diagnostics.

---

# 10. Negotiation Flow

```text
Peer A
  |
  | HELLO
  | core-version=1
  | extensions:
  |   messaging/1 [text, reply, edit]
  |   files/1 [resume, chunking]
  |
  v
Peer B
  |
  | HELLO_ACK
  | core-version=1
  | negotiated:
  |   messaging/1 [text, reply]
  |   files/1 [resume]
  |
  v
Session Ready
```

Application traffic requiring an extension must not begin until that extension has been negotiated.

---

# 11. Mandatory and Optional Extensions

A consumer may declare:

```rust
pub enum ExtensionRequirement {
    Required,
    Optional,
}
```

Example file product:

```text
files = required
messaging = absent
```

Example messenger:

```text
messaging = required
files = optional
presence = optional
```

An unsupported optional extension must not tear down the whole session.

---

# 12. Extension Registry

```rust
pub trait ProtocolExtension: Send + Sync {
    fn descriptor(&self) -> ExtensionDescriptor;

    fn create_handler(
        &self,
        negotiated: NegotiatedExtension,
    ) -> Result<Box<dyn ExtensionHandler>, ExtensionError>;
}
```

Registry:

```rust
pub struct ExtensionRegistry {
    // protocol id -> implementation
}
```

Build the registry explicitly during runtime construction.

Avoid mandatory global static registries.

---

# 13. Runtime Construction

```rust
let runtime = CommunicationRuntime::builder()
    .register_extension(MessagingExtension::new(config))
    .register_extension(FileExtension::new(file_config))
    .build()
    .await?;
```

A file-only application:

```rust
let runtime = CommunicationRuntime::builder()
    .register_extension(FileExtension::new(file_config))
    .build()
    .await?;
```

This is the intended reuse model.

---

# 14. Extension Isolation

Extensions must not reach directly into each other's private state.

Bad:

```text
MessagingExtension
      ↓ direct internal access
FileExtension
```

Good:

```text
MessagingExtension
      ↓
ContentReference API
      ↓
File capability
```

or application-level composition.

This prevents circular coupling.

---

# 15. Shared Services

Some infrastructure is legitimately shared:

```text
identity
crypto
session
clock
scheduler
metrics
resource policy
```

Expose constrained handles, not the entire runtime.

```rust
pub struct ExtensionContext {
    pub identity: IdentityHandle,
    pub session: SessionHandle,
    pub scheduler: SchedulerHandle,
    pub resources: ResourcePolicy,
}
```

Avoid:

```rust
Arc<EntireRuntime>
```

inside every extension.

---

# 16. Logical Channel Model

When transport supports multiplexing:

```text
Session
├── Core control
├── Messaging
├── File control
├── File data stream(s)
├── Presence/datagram-like traffic
└── Calls/media
```

Large file traffic must not head-of-line-block control or messaging traffic.

---

# 17. Session-Local Extension IDs

After negotiation:

```text
messaging/1 → 7
files/1 → 9
presence/1 → 12
```

The numeric mapping is session-local.

This reduces repeated framing overhead while retaining stable global protocol identities.

---

# 18. Framing

Every extension frame must be bounded.

Conceptually:

```text
FrameLength
ExtensionSessionId
FrameType
Flags
Payload
```

Before allocation:

```text
read bounded header
validate length
check extension limit
allocate/read safely
```

Never trust remote length fields.

---

# 19. Per-Extension Resource Limits

```rust
pub struct ExtensionLimits {
    pub max_frame_size: usize,
    pub max_in_flight_frames: usize,
    pub max_concurrent_streams: usize,
    pub max_buffered_bytes: usize,
}
```

Messaging:

```text
small frames
moderate concurrency
```

Files:

```text
streaming
bounded chunk buffers
possibly several streams
```

Presence:

```text
very small frames
high discardability
```

---

# 20. Backpressure

```text
Extension producer
      ↓
bounded queue
      ↓
session scheduler
      ↓
transport
```

A slow Bluetooth or DTN route must never produce:

```text
unlimited queued file chunks
        ↓
out-of-memory
```

Backpressure is mandatory, not optional optimization.

---

# 21. Traffic Priority

```rust
pub enum TrafficPriority {
    Critical,
    Control,
    Interactive,
    Normal,
    Bulk,
    Background,
}
```

Examples:

```text
SOS              → Critical
Receipt          → Control
Text             → Interactive
Thumbnail        → Normal
File chunk       → Bulk
Background sync  → Background
```

---

# 22. Fair Scheduling

Do not permanently starve bulk traffic.

Use:

```text
weighted fair scheduling
+
strict bounded emergency override
```

rather than naive perpetual highest-priority-first scheduling.

---

# 23. Extension Lifecycle

```text
Registered
   ↓
Advertised
   ↓
Negotiated
   ↓
Opening
   ↓
Active
   ↓
Closing
   ↓
Closed
```

Failures:

```text
Rejected
Unsupported
VersionMismatch
ProtocolError
```

Lifecycle state must be explicit.

---

# 24. Lazy Extension Opening

Capability negotiation does not mean immediate heavy initialization.

Example:

```text
files capability negotiated
      ↓
file extension dormant
      ↓
user sends first file
      ↓
OPEN_EXTENSION
      ↓
initialize transfer machinery
```

This saves CPU, memory, storage handles and battery on mobile.

---

# 25. Lazy Initialization Targets

Particularly suitable for:

```text
calls
video
camera
file hashing
large media indexes
nearby radios
```

Do not initialize expensive subsystems merely because the binary contains them.

---

# 26. Extension Shutdown

Graceful close:

```text
stop accepting new work
finish/persist critical state
send close when useful
release buffers
```

Abrupt close:

```text
transport disappears
persist resumable state
mark recoverable work
```

Correctness must not depend on graceful shutdown.

---

# 27. Typed Extension Errors

```rust
pub enum ExtensionError {
    Unsupported,
    VersionMismatch,
    CapabilityMismatch,
    ResourceLimit,
    ProtocolViolation,
    Unauthorized,
    StorageFailure,
    Internal,
}
```

Wire errors should use stable codes.

Internal debug strings must not become protocol semantics.

---

# 28. Protocol Violation Classification

Classify violations as:

```text
Recoverable
Extension-fatal
Session-fatal
Peer-abusive
```

Examples:

```text
unknown optional frame      → Recoverable
oversized malicious frame   → Peer-abusive
authentication failure      → Session-fatal
invalid file state          → Extension-fatal
```

---

# 29. Unknown Extensions

If a peer advertises an unknown optional extension:

```text
ignore or mark unsupported
```

Do not fail the entire connection.

This is fundamental for forward compatibility.

---

# 30. Unknown Capabilities

Within a known extension:

```text
unknown optional capability
    → ignore
```

```text
unknown required capability
    → reject extension operation/negotiation
```

The protocol must distinguish optional from required capability semantics.

---

# 31. Operation-Level Required Capabilities

An operation may require a specific capability.

Example:

```text
EditMessage
requires messaging.edit
```

If the peer lacks it:

```text
do not send unsupported wire operation
```

The application may:

```text
disable the action
send a new message instead
show unsupported
```

---

# 32. Security Requirements Per Extension

```rust
pub struct SecurityRequirements {
    pub authenticated_peer: bool,
    pub e2ee_required: bool,
    pub authorization_required: bool,
    pub allow_anonymous: bool,
}
```

Messaging likely requires:

```text
authenticated = true
E2EE = true
```

Public emergency broadcast can use different semantics but still needs explicit signature and authorization rules.

---

# 33. Authorization Hooks

Extensions should not invent product-specific authorization internally.

```rust
pub trait ExtensionAuthorization {
    async fn authorize(
        &self,
        peer: PeerIdentity,
        operation: OperationDescriptor,
    ) -> AuthorizationDecision;
}
```

Examples:

```text
Messenger → block/contact policy
ERP       → organization/role policy
Emergency → authority/priority policy
```

---

# 34. Namespaced Application Extensions

Examples:

```text
com.schoolerp/document-alert/1
com.rescue/incident/1
org.example/messaging/1
```

Namespaces prevent accidental interpretation between unrelated products.

---

# 35. Extension Descriptor

```rust
pub struct ExtensionDescriptor {
    pub id: ProtocolId,
    pub version: ExtensionVersion,
    pub capabilities: CapabilitySet,
    pub requirements: ExtensionRequirements,
    pub limits: ExtensionLimits,
}
```

Used for:

- negotiation
- documentation
- diagnostics
- testing
- compatibility tooling

---

# 36. Generated Documentation

Typed descriptors allow tooling to generate:

```text
supported extensions
version table
capabilities
resource limits
security requirements
```

This reduces documentation drift.

---

# 37. Wire Schema Ownership

Each extension owns its wire schema.

Example:

```text
comm-messaging/protocol/v1/
comm-files/protocol/v1/
comm-emergency/protocol/v1/
```

Do not directly serialize application domain types.

---

# 38. Domain Types vs Wire Types

Bad:

```text
one Message struct used by:
UI + DB + wire + domain
```

Good:

```text
DomainMessage
      ↓ validated conversion
WireMessageV1
      ↓ serialization
bytes
```

This allows protocol evolution independently of UI/database evolution.

---

# 39. Serialization Discipline

Postcard is suitable for compact Rust-oriented binary payloads, but enforce:

- explicit size limits
- fixed-width protocol-relevant integers
- versioned enums
- bounded strings/vectors
- semantic validation after decode
- golden compatibility tests

Avoid `usize` as a wire-semantic field.

---

# 40. Low-Copy Strategy

Use low-copy types where large payloads justify them:

```text
Bytes
Arc<[u8]>
borrowed slices
streamed file chunks
```

Do not overcomplicate tiny control frames solely to chase zero-copy.

The priority order is:

```text
correctness
bounded memory
simple ownership
measured optimization
```

---

# 41. Extension Events

Prefer feature-specific event types.

Messaging:

```text
MessageReceived
ReceiptReceived
TypingChanged
```

Files:

```text
TransferStarted
TransferProgress
TransferCompleted
```

The high-level client may optionally aggregate them.

---

# 42. Avoid a Mandatory Giant Event Enum

If every feature must import every event variant, modularity is weakened.

Prefer:

```text
CoreEvent
MessagingEvent
FileEvent
PresenceEvent
```

with optional aggregation for applications that want one stream.

---

# 43. Public SDK Surface

Applications should normally use:

```text
CommunicationClient
MessagingClient
FileTransferClient
PresenceClient
```

not raw extension handlers.

Extension internals remain replaceable.

---

# 44. Peer Capability Query

```rust
let caps = client.peer_capabilities(peer).await?;

if caps.supports(MESSAGING_V1) {
    // enable messaging UX
}

if caps.supports(FILES_V1) {
    // enable file-send UX
}
```

Consumers can adapt without hard-coding transport details.

---

# 45. Capability Cache

Persist a hint cache:

```text
peer
protocol versions
capabilities
last observed
expiry/source
```

But authenticate and renegotiate on a fresh session.

Cached capabilities are hints, not security truth.

---

# 46. Capability Changes

Capabilities can change because of:

- application upgrade
- feature disablement
- permissions
- hardware availability
- platform changes
- policy changes

Therefore they are not immutable identity data.

---

# 47. Stability Classification

```rust
pub enum ExtensionStability {
    Stable,
    Experimental,
    Internal,
}
```

Experimental protocols should require explicit opt-in in production builds.

---

# 48. Private/Enterprise Extensions

Enterprise software may define:

```text
com.company/internal-workflow/1
```

without modifying the core protocol.

That is a first-class design goal.

---

# 49. Third-Party Extension Loading

Initial recommendation:

```text
compile-time Rust crate integration
```

not arbitrary runtime-loaded native plugins.

Benefits:

- stronger typing
- smaller attack surface
- mobile compatibility
- easier signing/auditing
- predictable lifecycle

Runtime plugin architecture belongs to a later dedicated part.

---

# 50. FFI Boundary

Do not design the internal extension API around C ABI limitations yet.

If external-language integration becomes required, expose a separate:

```text
comm-ffi
```

with opaque handles and stable events.

Internal Rust APIs can remain ergonomic and strongly typed.

---

# 51. Transport Neutrality

Extension payloads must not require:

```text
Iroh Endpoint IDs
Bluetooth MAC addresses
IP addresses
Wi-Fi handles
```

unless a specific protocol explicitly models transport metadata.

Use abstract peer/session identities.

---

# 52. Delivery Classes

```rust
pub enum DeliveryClass {
    Realtime,
    ReliableInteractive,
    Durable,
    DelayTolerant,
}
```

Examples:

```text
Typing      → Realtime
Text        → Durable
SOS         → DelayTolerant + Critical
Video frame → Realtime
File chunk  → Durable/DelayTolerant depending policy
```

This informs routing.

---

# 53. Routing Requirements Per Operation

An operation can declare:

```text
realtime requirement
maximum age
durability
forwarding permission
size class
priority
```

The routing engine, not the extension, chooses the concrete network path.

---

# 54. Resource Accounting

Track per extension:

```text
memory
network bytes
stored bytes
queue depth
active streams
CPU-heavy operations
```

This supports:

- diagnostics
- quotas
- mobile battery policy
- abuse protection

---

# 55. Per-Peer Quotas

Limit:

```text
frames/sec
bytes/sec
concurrent streams
queued operations
DTN storage
file transfers
```

Policies may vary by trust level.

---

# 56. Abuse Handling

The runtime needs escalating controls:

```text
rate limit
pause extension
close extension
quarantine peer
close session
```

A malicious file stream should not corrupt or disable unrelated messaging state unless necessary for overall security.

---

# 57. Observability

Trace fields:

```text
peer_id
extension_id
version
frame_type
operation_id
duration
bytes
result
```

Never log sensitive payload contents or keys.

---

# 58. Metrics

Useful extension metrics:

```text
negotiation success
open latency
frames sent/received
protocol violations
queue depth
backpressure activations
unsupported capability count
version mismatch count
```

Local metrics should work without external telemetry.

---

# 59. Diagnostics

Example diagnostic view:

```text
Active extensions:
  messaging/1
    text
    reply
    receipts

  files/1
    resume
    chunking

Unsupported remote extensions:
  com.example/custom/2
```

---

# 60. Compatibility Matrix

Each stable extension must test:

```text
v1.0 ↔ v1.0
v1.0 ↔ v1.1
v1.1 ↔ v1.2
old subset ↔ new superset
```

Major incompatibility should fail cleanly and predictably.

---

# 61. Golden Wire Tests

Maintain test vectors:

```text
Rust value
    ↓
expected exact stable bytes
    ↓
decode back to equivalent wire value
```

Unexpected stable-wire changes should fail CI.

---

# 62. Fuzzing

Fuzz:

```text
core hello
extension advertisement
capabilities
extension open
frame parser
version parser
oversized lengths
invalid states
unknown fields
```

Each extension owns additional fuzz targets.

---

# 63. Property Tests

Important invariants:

```text
encode/decode round trip
unknown optional extension does not kill session
required unknown capability is rejected
duplicate advertisement is deterministic
invalid length cannot trigger unbounded allocation
```

---

# 64. Simulated Peer Testing

Example:

```text
Peer A:
  messaging/1
  files/1

Peer B:
  messaging/1
```

Expected:

```text
messaging works
files unsupported
session remains valid
```

---

# 65. Upgrade Example

Peer A:

```text
files/1
files/2
```

Peer B:

```text
files/1
```

Negotiate:

```text
files/1
```

Peer C:

```text
files/2
```

A ↔ C uses:

```text
files/2
```

This enables gradual protocol migration.

---

# 66. Multiple Major Versions

An implementation may temporarily support several majors:

```text
files/protocol/v1
files/protocol/v2
```

Convert into common domain models where semantics permit.

---

# 67. Deprecation Lifecycle

```text
Supported
   ↓
Deprecated
   ↓
Disabled by default
   ↓
Removed
```

Publish timelines.

Do not suddenly remove compatible wire protocols unless security requires it.

---

# 68. Security Deprecation

If a protocol version becomes unsafe:

```text
mark insecure
disable it
provide diagnostic
require upgrade where necessary
```

Security outranks backward compatibility.

---

# 69. Extension Persistence

Extensions may own durable state via dedicated repositories.

Messaging:

```text
messages
outbox
receipts
```

Files:

```text
transfer journal
verified chunks
```

Protocol handlers should not write arbitrary product database tables directly.

---

# 70. Wire vs Database Migration

These are separate concerns.

Example:

```text
messaging wire protocol v1
```

can coexist with:

```text
local database schema v7
```

Do not tie them together.

---

# 71. ERP Custom Extension Example

```text
com.example.erp/approval/1
```

Can reuse:

```text
identity
crypto
session
routing
DTN
```

without changing messaging or files.

---

# 72. Emergency Extension Design

Potential protocol families:

```text
emergency-sos/1
emergency-alert/1
emergency-resource/1
```

Critical fields include:

```text
priority
expiry
signature authenticity
DTN permission
location privacy semantics
```

---

# 73. File Extension Example

```text
files/1
```

Capabilities:

```text
manifest
range request
resume
parallel chunks
content addressing
encrypted metadata
```

Streams:

```text
control
data-0
data-1
...
```

No messaging dependency.

---

# 74. Messaging Extension Example

```text
messaging/1
```

Capabilities:

```text
text
reply
edit
reaction
delivery receipt
read receipt
custom content reference
```

Attachments are represented as content/blob references.

Actual transfer belongs to `files/1`.

---

# 75. Presence Extension Example

```text
presence/1
```

Properties:

```text
small frames
short expiry
usually not DTN
loss tolerant
```

Possible capabilities:

```text
basic presence
typing
activity hint
```

---

# 76. Calls and Media Extensions

Separate call control from media transport where useful:

```text
calls/1
media/1
```

Capabilities may include:

```text
audio codecs
video codecs
resolution
frame rate
hardware/software capability
```

Codec implementation remains outside the core protocol.

---

# 77. Protocol Composition

One product can negotiate:

```text
messaging/1
files/1
presence/1
calls/1
emergency/1
custom-app/1
```

without creating a central monolith.

---

# 78. Core Handshake State Machine

```text
Transport Connected
       ↓
Core Hello
       ↓
Identity Binding
       ↓
Core Version Agreement
       ↓
Extension Advertisement
       ↓
Capability Negotiation
       ↓
Session Established
       ↓
Lazy Extension Opens
```

Authentication failure stops application-level use.

---

# 79. Reconnection

Cached negotiation can speed reconnection, but peers must revalidate:

```text
identity
protocol compatibility
capability validity
```

Never blindly trust stale session metadata.

---

# 80. Mobile Efficiency

On Android/iOS:

- lazy-open heavy extensions
- stop ephemeral services when backgrounded
- persist durable work
- reopen after resume
- use tight queues
- obey battery policy
- avoid starting media/files if unused

This architecture directly supports mobile efficiency.

---

# 81. Headless Nodes

A headless node can register:

```text
files
DTN
relay
discovery
```

without:

```text
Dioxus
messaging UI
calls
```

Useful for:

```text
Raspberry Pi
NAS
server
emergency gateway
enterprise node
```

---

# 82. Compile-Time Features

Example:

```toml
[features]
default = ["core"]
messaging = ["dep:comm-messaging"]
files = ["dep:comm-files"]
presence = ["dep:comm-presence"]
dtn = ["dep:comm-dtn"]
calls = ["dep:comm-calls"]
```

Compile-time features choose which code is present.

Runtime negotiation chooses what peers actually use.

---

# 83. Binary Size

This prevents a file-only CLI from pulling in:

```text
Dioxus
AV1
camera
messenger UI
```

unless requested.

This is a major benefit for reusable software.

---

# 84. Typed Extension Configuration

Messaging:

```rust
pub struct MessagingConfig {
    pub max_message_size: usize,
    pub receipts: bool,
    pub editing: bool,
}
```

Files:

```rust
pub struct FileConfig {
    pub chunk_size: usize,
    pub max_parallel_chunks: usize,
    pub max_file_size: u64,
}
```

Avoid untyped generic configuration maps for core behavior.

---

# 85. Configuration Validation

Validate before opening network listeners.

Example invalid configuration:

```text
chunk size > extension maximum frame size
```

Fail fast during runtime construction.

---

# 86. Extension Dependencies

Represent dependencies explicitly:

```text
required dependency
optional integration
```

Example:

```text
messaging
    optional integration → files
```

Messaging itself must continue working without files.

---

# 87. Integration Interfaces

Messaging can depend on an abstract resolver:

```rust
pub trait ContentResolver {
    async fn resolve(
        &self,
        reference: ContentReference,
    ) -> Result<ResolvedContent, ResolveError>;
}
```

The file subsystem can provide an implementation.

This preserves modularity.

---

# 88. Core vs First-Party Extensions

Classify protocols:

```text
Core-maintained
First-party optional
Third-party
Experimental
```

Messaging should not be forced into the core merely because the flagship application uses it.

---

# 89. Extension Provenance

For compile-time extensions:

```text
crate review
supply-chain controls
binary signing
release provenance
```

are sufficient initially.

Runtime-loaded plugin signing belongs to the later plugin/module architecture.

---

# 90. Interoperability Specification

Each stable extension must document:

```text
protocol ID
versions
capabilities
wire schema
state machine
limits
security rules
error codes
test vectors
```

This allows non-Rust implementations later.

---

# 91. Rust Is the Reference, Not the Wire Format

Never depend on:

```text
native enum layout
usize width
pointer size
native endian
Rust-only implementation details
```

All wire semantics must be explicit.

---

# 92. Postcard Rules

If Postcard is used:

- use fixed-width integers where semantics matter
- avoid `usize` on wire
- version enum formats carefully
- bound collections
- keep golden test vectors
- validate decoded values

---

# 93. Error Code Strategy

Prefer extension-scoped stable codes.

Example:

```text
core:      0x0000–0x00FF
messaging: 0x0100–0x01FF
files:     0x0200–0x02FF
```

or local error spaces scoped by negotiated extension ID.

Human-readable text is diagnostic only.

---

# 94. Extension Health

```rust
pub struct ExtensionHealth {
    pub state: ExtensionState,
    pub queue_depth: usize,
    pub active_operations: usize,
    pub last_error: Option<ExtensionErrorSummary>,
}
```

Useful for:

```text
diagnostics
automated recovery
support bundles
UI health indicators
```

---

# 95. Recovery Semantics

Classify operations:

```text
Retryable
Resumable
Discardable
Expired
```

Examples:

```text
text message    → Retryable
file transfer   → Resumable
typing          → Discardable
video frame     → Discardable
expired SOS     → Expired
```

---

# 96. Scheduler Contract

Extensions submit:

```text
priority
deadline
payload size
delivery class
peer
durability
```

The central scheduler chooses:

```text
transport
queue
retry policy
```

Extensions declare requirements; routing executes policy.

---

# 97. Storage Isolation

Use separate logical namespaces:

```text
messaging/
files/
presence/
emergency/
custom/
```

A migration bug in one extension should not corrupt unrelated state.

---

# 98. Metrics Isolation

Examples:

```text
comm_frames_sent{extension="messaging"}
comm_bytes_sent{extension="files"}
```

Avoid unbounded metric labels such as full peer IDs in aggregated telemetry.

---

# 99. Capability Isolation

Applications should receive only capability handles they are allowed to use.

Example:

```text
ERP document module
    can access files
    cannot access messaging
```

This supports least privilege inside larger products.

---

# 100. Public API Example

```rust
let runtime = CommunicationRuntime::builder()
    .identity(identity)
    .transport(iroh)
    .register_extension(
        MessagingExtension::builder()
            .text(true)
            .receipts(true)
            .build()?,
    )
    .register_extension(
        FileExtension::builder()
            .resume(true)
            .parallelism(4)
            .build()?,
    )
    .build()
    .await?;

let peer = runtime.connect(peer_addr).await?;

let messaging = runtime.messaging()?;
let files = runtime.files()?;
```

The application does not manipulate low-level handshake internals.

---

# 101. File-Only Acceptance Test

```rust
let runtime = CommunicationRuntime::builder()
    .identity(identity)
    .transport(iroh)
    .register_extension(FileExtension::default())
    .build()
    .await?;
```

No messaging crate is required.

If this cannot compile cleanly, the architecture is still too coupled.

---

# 102. Custom ERP Extension

```text
com.example.erp/approval/1
```

can reuse:

```text
identity
crypto
transport
sessions
routing
DTN
```

without modifying the core protocol.

---

# 103. Anti-Patterns

Do not:

```text
put every feature in one enum
make every extension mandatory
serialize domain structs directly
allow unbounded queues
use raw strings for hot capability checks
make Dioxus part of protocol handling
let Kotlin own protocol state
assume Iroh is the only possible transport forever
```

---

# 104. Recommended Crates for This Part

```text
comm-types
comm-protocol-core
comm-extension-api
comm-session
comm-capability
```

Possible structure:

```text
crates/comm-extension-api/
├── src/
│   ├── lib.rs
│   ├── descriptor.rs
│   ├── registry.rs
│   ├── handler.rs
│   ├── lifecycle.rs
│   ├── capability.rs
│   ├── limits.rs
│   ├── security.rs
│   └── error.rs
└── Cargo.toml
```

---

# 105. Implementation Sequence

## Phase 1

Implement:

```text
ProtocolId
ExtensionVersion
CapabilityId
ExtensionDescriptor
ExtensionRegistry
```

## Phase 2

Implement:

```text
Hello
VersionNegotiation
ExtensionAdvertisement
CapabilityNegotiation
```

## Phase 3

Implement:

```text
lazy extension open
lifecycle
bounded framing
```

## Phase 4

Implement:

```text
priority scheduling
backpressure
resource quotas
```

## Phase 5

Convert:

```text
messaging
files
```

into independent extensions.

## Phase 6

Add:

```text
compatibility tests
fuzzing
golden wire tests
diagnostics
```

---

# 106. Definition of Done

Part 01 is complete when:

- messaging and files register independently
- file-only peers work without messaging
- unknown optional extensions do not break sessions
- incompatible required extensions fail deterministically
- capability intersection is negotiated
- frame sizes are bounded
- queues are bounded
- traffic priority exists
- protocol IDs are stable
- wire and domain types are separate
- extensions cannot access unrestricted runtime state
- compatibility tests exist
- fuzz tests exist
- diagnostics show negotiated extensions
- reconnect behavior is deterministic
- heavy extensions can lazy-open on mobile
- custom application extensions can be added without changing core protocol

---

# 107. Relationship to the Remaining 23 Parts

This document is foundational.

The remaining architecture sequence is:

```text
02 — Multi-Device Identity
03 — Transport & Routing Policy Engine
04 — Offline Event Log
05 — Robust File / Blob Subsystem
06 — DTN / Store-Carry-Forward
07 — Capability Negotiation Expansion
08 — Resource Limits & Backpressure
09 — Crash Recovery
10 — Protocol Fuzzing & Test Suite
11 — Relay / Self-Hosted Infrastructure
12 — Multipath Networking
13 — Battery-Aware Scheduling
14 — Proximity Abstraction
15 — QR / NFC Bootstrap Pairing
16 — Daemon & Headless Runtime
17 — Emergency Priority Architecture
18 — Network Diagnostics & Path Visualization
19 — C ABI / FFI
20 — Embedded Linux Node
21 — WASM-Compatible Components
22 — Third-Party Protocol Extensions
23 — External Interoperability Suite
24 — Plugin / Module Ecosystem
```

Later parts should build on the extension contracts rather than bypassing them.

---

# 108. Final Principle

The protocol should evolve through **independently versioned, negotiated capabilities**, not through an ever-growing central enum.

The architecture should allow:

```text
old client
+
new client
+
file-only application
+
messenger
+
ERP
+
emergency node
```

all to participate in the same communication ecosystem while using only the capabilities they understand.

That is the foundation required to turn the project from one application's protocol into a reusable communication platform.
