# Part 22 — WASM-Compatible Components Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 22 of 24  
**Primary implementation language:** Rust  
**Portable execution target:** WebAssembly Components + WIT, with WASI where host capabilities are appropriate  
**Primary host runtime:** native Rust runtime; Wasmtime is the preferred initial component host for server/desktop/embedded deployments  
**Primary goals:** portable components, sandboxed third-party execution, capability-based host access, browser/WASI portability, resource metering, deterministic interfaces, safe extension reuse, and preservation of the high-performance native networking core

---

# 1. Purpose

The communication platform should be able to reuse selected logic across:

```text
desktop
server
embedded Linux
mobile-supporting host runtimes
browser-compatible environments
third-party extension sandboxes
testing environments
```

without attempting to force the entire communication engine into WebAssembly.

The correct architecture is not:

```text
compile everything to WASM
```

Instead:

```text
Native Rust Core
+
Portable WASM-Compatible Components
```

The native core continues to own:

```text
Iroh networking
QUIC
Bluetooth
Wi-Fi Direct/Aware
native filesystem integration
hardware codecs
OS notifications
secure key stores
high-performance file I/O
daemon lifecycle
```

WASM-compatible components handle logic that benefits from:

```text
portability
sandboxing
determinism
cross-language interoperability
extension isolation
```

The core rule is:

> **Use WebAssembly for portable policy and extension logic; keep hardware-, OS-, transport-, and latency-critical systems native.**

---

# 2. Architectural Position

```text
                    Product / App
                         │
               Communication Runtime
                         │
        ┌────────────────┴────────────────┐
        │                                 │
 Native Rust Core                  Component Host
        │                                 │
 Iroh / Files / DTN /              WASM Components
 Proximity / Storage /              ├── policies
 Secure Store / Media               ├── transforms
        │                           ├── extensions
        │                           └── workflows
        │                                 │
        └──────── Capability APIs ─────────┘
```

---

# 3. What SHOULD Be WASM-Compatible

Good candidates:

```text
message transforms
application-specific protocol logic
validation rules
policy evaluation
workflow engines
document metadata transforms
ERP event adapters
format conversion
custom synchronization rules
routing policy plugins
notification rules
DTN forwarding policy plugins
content classification
extension state machines
deterministic business logic
test/reference implementations
```

---

# 4. What SHOULD Stay Native

Keep native:

```text
Iroh endpoint
QUIC transport
Bluetooth
Wi-Fi APIs
LAN sockets
hardware codec integration
AV1 software codec hot loops unless separately optimized
filesystem zero-copy
memory-mapped large files
platform keystore
OS biometric APIs
Android JNI platform adapters
system daemon integration
kernel/network-interface control
```

These depend heavily on OS/hardware or need native performance.

---

# 5. Component Model First

The portable architecture should target:

```text
WebAssembly Component Model
+
WIT interfaces
```

rather than exposing arbitrary raw WebAssembly imports.

Benefits:

```text
typed interfaces
explicit imports/exports
language-neutral contracts
composable components
better capability control
versionable APIs
```

---

# 6. WIT as the Contract

Example:

```wit
package comm:extension@1.0.0;

interface diagnostics {
    log: func(level: u8, message: string);
}

interface state {
    get: func(key: list<u8>) -> option<list<u8>>;
    put: func(key: list<u8>, value: list<u8>) -> result<_, string>;
}

world extension {
    import diagnostics;
    import state;
    export run: func(input: list<u8>) -> result<list<u8>, string>;
}
```

The actual production interfaces should use richer typed records instead of opaque bytes wherever practical.

---

# 7. WIT Package Naming

Recommended:

```text
comm:core
comm:extension
comm:storage
comm:routing
comm:dtn
comm:files
comm:diagnostics
```

Third-party namespaces:

```text
vendor:package
organization:extension-name
```

---

# 8. Component API Versioning

Separate:

```text
WIT package version
component package version
wire protocol version
extension state schema
host runtime version
```

These must not be conflated.

---

# 9. WIT Major Compatibility

Breaking changes:

```text
new package major
```

Examples:

```text
remove field
change required parameter
change semantic meaning
```

---

# 10. Additive Evolution

Prefer:

```text
new interface
new optional variant
new method in newer package version
```

without breaking existing components.

---

# 11. Capability-Based Host Design

A component receives only explicitly imported capabilities.

Example:

```text
component needs storage
→ import comm:storage/state

component does not need networking
→ no network capability
```

No ambient authority.

---

# 12. No Raw Host OS Access by Default

Do not expose:

```text
arbitrary filesystem
arbitrary socket
environment variables
process spawning
system shell
```

to untrusted components.

---

# 13. Host Capability Categories

```rust
pub enum WasmHostCapability {
    Diagnostics,
    ExtensionState,
    ExtensionSecrets,
    PeerMessaging,
    FileReferences,
    DtnDelivery,
    RoutingRequest,
    Time,
    Random,
    Notification,
}
```

---

# 14. Capability Grants

At load time:

```text
manifest requests
+
user/admin policy
+
host policy
=
effective grants
```

---

# 15. Component Manifest

Example:

```ron
(
    id: "org.example.whiteboard",
    version: "1.2.0",
    world: "comm-extension",
    wit: "comm:extension@1",
    permissions: [
        "state",
        "peer-messaging",
        "files",
    ],
    resources: (
        memory_bytes: 33554432,
        execution_ms: 100,
        persistent_bytes: 104857600,
    ),
)
```

---

# 16. WASM Is Not Automatically Safe

WebAssembly provides strong memory isolation relative to native plugins, but the host can still create unsafe capabilities.

Bad host API:

```text
open arbitrary path
connect arbitrary socket
execute command
```

Good host API:

```text
read own state
send protocol frame to authorized peer
request BlobId
```

---

# 17. Host Owns Policy

Even when component requests:

```text
send
store
DTN
critical priority
```

the native core still applies:

```text
identity
authorization
resource limits
battery policy
routing policy
emergency authorization
```

---

# 18. WASI Usage

Use WASI selectively.

Do not automatically provide the full WASI environment to every extension.

Possible:

```text
WASI clocks
WASI random
WASI limited filesystem
WASI limited streams
```

only where appropriate.

---

# 19. Prefer Domain APIs Over POSIX-Like APIs

For application extensions:

Good:

```text
comm:files/open-blob
comm:peer/send-event
```

Less desirable:

```text
filesystem path
TCP socket
```

This keeps policy enforceable.

---

# 20. Browser vs WASI Host

Browser WebAssembly and native WASI execution are different environments.

Design components against:

```text
portable WIT domain interfaces
```

rather than assuming:

```text
filesystem
threads
sockets
```

exist everywhere.

---

# 21. Portable Core World

Define a minimal browser/native-compatible world:

```text
comm:portable
```

Imports:

```text
clock
random
state
diagnostics
```

No native networking assumptions.

---

# 22. Native Extended World

For trusted native components:

```text
comm:native-extension
```

may add:

```text
file streams
background tasks
advanced diagnostics
```

Still no raw OS access unless explicitly needed.

---

# 23. Browser-Compatible Component Candidates

Good candidates:

```text
message formatting
policy rules
workflow state machines
protocol parsing
content validation
extension UI-neutral logic
offline local transforms
```

---

# 24. Browser-Incompatible Components

Examples:

```text
BLE scanner
Wi-Fi Direct manager
Iroh native endpoint
system daemon
hardware media codec
```

These remain host-provided/native.

---

# 25. Native Host Runtime

Preferred initial runtime:

```text
Wasmtime
```

Reasons:

```text
Rust embedding
Component Model support
resource controls
mature server/desktop use
```

Host runtime remains abstracted.

---

# 26. Runtime Abstraction

```rust
pub trait ComponentRuntime {
    async fn instantiate(
        &self,
        package: &ComponentPackage,
        grants: &CapabilityGrantSet,
    ) -> Result<ComponentInstance, ComponentError>;
}
```

---

# 27. Avoid Wasmtime Types in Domain Core

Keep:

```text
Store
Linker
Component
ResourceLimiter
```

inside:

```text
comm-wasm-wasmtime
```

not across all crates.

---

# 28. Suggested Crates

```text
crates/
├── comm-component-core/
├── comm-component-host/
├── comm-component-wit/
├── comm-wasm-wasmtime/
├── comm-component-storage/
├── comm-component-security/
├── comm-component-testkit/
└── comm-component-browser/
```

---

# 29. `comm-component-core`

Owns:

```text
component IDs
manifest
permissions
resource budgets
lifecycle
errors
```

---

# 30. `comm-component-wit`

Contains:

```text
WIT packages
generated bindings
compatibility fixtures
```

---

# 31. `comm-wasm-wasmtime`

Owns:

```text
Wasmtime engine
Store
Linker
component instantiation
fuel/epoch policy
memory limiter
trap conversion
```

---

# 32. `comm-component-browser`

Provides compatible host adapters for browser-supported worlds.

It does not attempt to implement native-only capabilities.

---

# 33. Component Identity

```rust
pub struct ComponentId(String);
```

May reuse extension ID from Part 21 when component implements an extension.

---

# 34. Component Package

```rust
pub struct ComponentPackage {
    pub manifest: ComponentManifest,
    pub bytes: Vec<u8>,
    pub digest: Digest,
    pub signature: Option<PackageSignature>,
}
```

Large package data should be streamed from storage rather than always held in RAM.

---

# 35. Package Signing

Signed packages provide:

```text
publisher authenticity
integrity
```

not automatic permission trust.

---

# 36. Package Digest

Use content digest to:

```text
cache
deduplicate
verify
pin
```

---

# 37. Execution Trust Classes

```rust
pub enum ComponentTrust {
    BuiltIn,
    TrustedPublisher,
    SandboxedThirdParty,
    Development,
}
```

---

# 38. All WASM Components Still Sandboxed

Even trusted components should use bounded capabilities.

Trust may increase resource/permission ceilings, not remove all policy.

---

# 39. Instance Lifecycle

```text
Installed
 ↓
Validated
 ↓
Compiled/Prepared
 ↓
Instantiated
 ↓
Running
 ↓
Stopping
 ↓
Stopped
```

Failure:

```text
Trapped
Failed
Quarantined
```

---

# 40. Instance Scope

Possible:

```text
singleton component
per-account instance
per-peer session
per-protocol session
per-task invocation
```

Choose according to state isolation needs.

---

# 41. Stateless Invocation

Ideal for:

```text
validation
transforms
policy evaluation
```

Create/call/reuse safely.

---

# 42. Stateful Component

Useful for:

```text
collaboration protocol
workflow
```

Persistent state must live in host-managed storage, not only WASM linear memory.

---

# 43. Linear Memory Is Ephemeral

Do not treat component memory as durable state.

After:

```text
restart
trap
upgrade
```

reconstruct from host-managed state/events.

---

# 44. Memory Budget

Every component has maximum memory.

Example:

```rust
pub struct ComponentMemoryBudget {
    pub max_linear_memory_bytes: u64,
    pub max_table_elements: u32,
}
```

---

# 45. Memory Growth

Host must deny memory growth beyond quota.

Trap/error should disable only affected invocation/session.

---

# 46. CPU Budget

Untrusted component must not run forever.

Use:

```text
fuel
epoch interruption
host deadline
```

depending workload.

---

# 47. Fuel

Useful for:

```text
deterministic instruction budget
tests
small policy execution
```

---

# 48. Epoch Interruption

Useful for:

```text
wall-time style coarse interruption
high-throughput runtime
```

Host decides per trust/workload.

---

# 49. Dual Limit

Strong model:

```text
fuel/instruction budget
+
wall-clock deadline
```

---

# 50. Invocation Budget

```rust
pub struct InvocationBudget {
    pub max_fuel: Option<u64>,
    pub max_wall_time: Duration,
    pub max_output_bytes: u64,
}
```

---

# 51. No Infinite Host Calls

Host calls are also bounded.

Component cannot escape CPU limit by repeatedly requesting expensive host work without quotas.

---

# 52. Async Host Calls

Networking/storage operations are naturally async.

Component API should use Component Model-supported async patterns where stable in chosen runtime/toolchain, or an explicit operation-handle model until mature.

Do not block native executor threads.

---

# 53. Stable Async Abstraction

Conceptually:

```text
request operation
yield
resume on result
```

The exact WIT mechanics may evolve.

Keep business API independent of runtime-specific async implementation details.

---

# 54. Host Operation Handle

Fallback architecture:

```text
component submits request
→ receives operation ID
→ polls/receives event
```

This also works in constrained hosts.

---

# 55. Deterministic Policy Components

For policy logic, prefer pure functions:

```text
input DTO
→ output decision
```

No clock/network unless explicitly needed.

---

# 56. Deterministic Testing

Inject:

```text
FakeClock
seeded randomness
fake storage
fake peer state
```

through WIT test host.

---

# 57. Clock Capability

Expose:

```text
monotonic
wall time
```

separately.

Components that do not need wall time should not receive it.

---

# 58. Random Capability

Provide secure random through host.

For deterministic tests:

```text
test host
```

can substitute seeded deterministic implementation.

Never use deterministic RNG for production cryptographic identity.

---

# 59. Storage Capability

Extension state API:

```wit
interface state {
    get: func(key: list<u8>) -> option<list<u8>>;
    put: func(key: list<u8>, value: list<u8>) -> result<_, state-error>;
    delete: func(key: list<u8>) -> result<_, state-error>;
}
```

---

# 60. Storage Namespacing

Host automatically scopes:

```text
tenant
account
component
```

Component never receives raw DB namespace identifiers it can escape.

---

# 61. Storage Quota

Enforce:

```text
total bytes
value bytes
keys
writes/sec
```

---

# 62. Transaction Capability

For complex state:

```text
begin transaction
get/put
commit
```

through safe host abstraction.

Do not expose raw SQL.

---

# 63. Event Store Capability

Protocol components may append their own extension events.

Host checks:

```text
namespace
size
schema
quota
```

---

# 64. File Capability

Component works with:

```text
BlobId
BlobReader resource
BlobWriter resource
```

not arbitrary paths.

---

# 65. Resource Handles in WIT

Component Model resources are ideal for host-owned entities such as:

```text
blob-reader
peer-session
operation
subscription
```

The host maintains real native objects.

---

# 66. Blob Reader Resource

Conceptual WIT:

```wit
resource blob-reader {
    read: func(max: u32) -> result<list<u8>, blob-error>;
}
```

Actual large streaming should use efficient stream interfaces where supported.

---

# 67. Avoid Giant Copies

WASM boundary crossing can copy data.

Do not route:

```text
4 GB file
```

through repeated giant component buffers.

Component should operate on:

```text
metadata
small chunks
stream handles
```

while native core owns hot transfer path.

---

# 68. Hot Path Rule

Do not invoke WASM for every QUIC packet.

WASM should sit at:

```text
operation
message
protocol frame
policy
workflow
```

granularity.

---

# 69. Message Transform Example

```text
Message DTO
 ↓
WASM component
 ↓
Validated/Transformed DTO
 ↓
Native sender
```

Good boundary.

---

# 70. Bad Packet Filter Example

```text
every encrypted packet
→ WASM
```

Too much overhead and policy risk for general use.

---

# 71. Routing Policy Component

A WASM policy may receive:

```text
path classes
latency class
metered flag
battery state
operation requirements
```

and return:

```text
preference weights
```

Native Part 03 still enforces hard constraints.

---

# 72. Hard Constraints Stay Native

Component cannot override:

```text
blocked peer
forbidden metered path
revoked device
resource limit
emergency authorization
```

It may only influence policy within allowed space.

---

# 73. DTN Policy Component

Can decide:

```text
which eligible bundle first
```

within host-supplied safe candidate set.

Host still enforces:

```text
hop limit
expiry
replication budget
storage quota
```

---

# 74. Emergency Boundary

Third-party WASM component cannot self-grant:

```text
Critical
AuthorityCritical
```

Part 17 authorizes effective class.

---

# 75. Identity API

Expose:

```text
peer trust class
scoped peer ID
sign-request handle if authorized
```

Never:

```text
raw identity private key
```

---

# 76. Scoped Peer IDs

Part 21 privacy model applies.

Each component can receive:

```text
component-scoped peer pseudonym
```

instead of raw DeviceId.

---

# 77. Secret Storage

Components may have own secret namespace.

Use:

```text
put-secret
sign/use-secret
```

according to policy.

---

# 78. No Secret Enumeration

Prefer named/opaque secret handles.

Do not let component list all host secrets.

---

# 79. Networking Capability

General third-party component should not get:

```text
raw TCP
raw UDP
```

by default.

It gets:

```text
open extension session to authorized peer
send protocol frame
```

---

# 80. External HTTP

If future extensions need external APIs:

```text
separate permission
allowlisted domains
request size limits
response size limits
timeouts
```

Do not grant ambient Internet.

---

# 81. DNS Privacy

Domain allowlist can still leak.

Document.

External network capability should be high-risk.

---

# 82. Notifications

Component requests:

```text
semantic notification
```

Host decides:

```text
OS priority
presentation
rate limit
```

---

# 83. UI Separation

Part 22 components are UI-neutral by default.

A future UI plugin may use:

```text
declarative UI schema
```

or separate Part 24 plugin interface.

Do not allow arbitrary native UI code from portable protocol component.

---

# 84. Browser Host

Browser-compatible host can provide:

```text
IndexedDB-backed state
browser clock
crypto RNG
postMessage/Web Worker
web transport adapter if product supplies
```

through the same domain-oriented WIT interfaces.

---

# 85. Browser Networking

The browser host cannot assume native Iroh/UDP/BLE parity.

Networking must be provided by:

```text
browser-supported transport
server bridge
WebTransport/WebSocket
product adapter
```

when needed.

Portable components should not care which.

---

# 86. Browser Component Role

A browser build may reuse:

```text
protocol parser
workflow logic
message transforms
validation
state machine
```

even if native P2P transport is unavailable.

---

# 87. WASM-Compatible Does Not Mean Web App

Important distinction:

```text
WASM-compatible component
```

can run:

```text
inside native desktop daemon
inside server
inside embedded node
inside browser
```

if its required host interfaces are available.

---

# 88. Embedded Linux Host

Part 20 can run sandboxed components with:

```text
small memory limit
fuel
strict storage quota
no JIT if platform/security policy prefers AOT
```

---

# 89. AOT Compilation

For embedded/server environments:

```text
precompile components
```

where runtime supports.

Benefits:

```text
faster startup
predictable deployment
reduced runtime compilation work
```

---

# 90. JIT vs AOT

Desktop/server:

```text
JIT or cached compilation
```

Embedded hardened appliance:

```text
AOT preferred where practical
```

Browser:

```text
browser engine decides
```

---

# 91. Precompiled Artifact Compatibility

Precompiled runtime artifacts may be:

```text
runtime-version
architecture
configuration
```

specific.

Do not distribute one precompiled artifact blindly to every host.

Keep original component package too.

---

# 92. Component Cache

Cache key:

```text
component digest
runtime version
target
engine config
```

---

# 93. Cache Is Rebuildable

Compiled WASM cache is:

```text
cache
```

not authoritative state.

Safe to delete.

---

# 94. Component Cold Start

Reduce by:

```text
precompile
pool common instances
lazy load
```

depending workload.

---

# 95. Instance Pooling

Useful for stateless transforms.

Do not reuse instance across security tenants without carefully resetting all state.

---

# 96. Tenant Isolation

For multi-tenant host:

```text
separate Store/instance
separate state namespace
separate resource budget
```

---

# 97. No Cross-Tenant Linear Memory

Never reuse active instance memory across tenants.

---

# 98. Component Resource Tree

```text
global host budget
  ↓
tenant
  ↓
component
  ↓
instance
  ↓
invocation
```

Part 08 hierarchical quotas apply.

---

# 99. Memory Pressure

Native host may:

```text
refuse new instance
evict idle instance
reduce cache
```

before harming core messaging.

---

# 100. Emergency Resource Pressure

When Part 17 emergency mode activates:

```text
pause low-priority components
reduce background extension CPU
preserve critical core networking
```

---

# 101. Battery-Aware WASM

On mobile/embedded battery:

```text
defer background components
lower CPU budget
disable expensive transforms
```

Part 13 remains authoritative.

---

# 102. Thermal-Aware WASM

If thermal serious:

```text
reduce component concurrency
```

before core communication suffers.

---

# 103. Lifecycle Supervision

Each component instance belongs to supervisor.

```text
start
health
stop
trap
restart
quarantine
```

---

# 104. Trap Handling

Trap:

```text
terminate invocation/session
record diagnostics
release resources
```

Core daemon continues.

---

# 105. Repeated Traps

Use backoff.

After threshold:

```text
quarantine component
```

---

# 106. Quarantine

State:

```text
installed
but execution disabled
```

until:

```text
update
admin action
automatic trusted rollback
```

---

# 107. Host Panic

WASM trap must not become host panic.

Convert runtime errors into typed component errors.

---

# 108. Component Error Model

```rust
pub enum ComponentError {
    InvalidPackage,
    SignatureRejected,
    IncompatibleWorld,
    PermissionDenied,
    ResourceDenied,
    FuelExhausted,
    DeadlineExceeded,
    MemoryLimit,
    Trap,
    HostCallFailed,
    StateMigrationFailed,
    Quarantined,
}
```

---

# 109. Guest Error Model

WIT interfaces should use typed variants, not stringly typed errors where practical.

---

# 110. Error Redaction

Host errors returned to component should not expose:

```text
filesystem paths
secret IDs
internal SQL
raw OS errors
```

unless needed.

---

# 111. Diagnostics

Component diagnostics include:

```text
instance count
CPU/fuel use
memory
traps
host-call errors
storage use
```

---

# 112. No High-Cardinality Metrics

Do not label fleet metrics with:

```text
peer ID
operation ID
```

---

# 113. Component Logs

Host logging API applies:

```text
rate limit
length limit
redaction
```

---

# 114. Log Injection

Component strings are untrusted.

Structured logging must escape/control output.

---

# 115. Testing Host

Provide:

```text
comm-component-testkit
```

with:

```text
FakeClock
FakeState
FakePeer
FakeFiles
FakeDtn
FakeDiagnostics
```

---

# 116. Component Unit Tests

Component authors test logic outside full application.

---

# 117. Host Conformance Tests

Every component world gets tests ensuring host implementation behaves consistently.

---

# 118. Browser/Native Parity Tests

Run same portable component against:

```text
native host
browser host
```

and compare semantic results.

---

# 119. Deterministic Golden Tests

For pure components:

```text
input
→ exact output
```

golden vectors.

---

# 120. WIT Compatibility Tests

CI checks accidental interface breaking changes.

---

# 121. Component Binary Validation

Before installation:

```text
parse
validate
inspect imports
verify manifest match
verify digest/signature
```

---

# 122. Import Allowlist

Reject package importing capabilities not declared.

---

# 123. Export Validation

Required exported world/functions must exist.

---

# 124. Package Bomb Protection

Bound:

```text
component size
custom sections
nested components
metadata
```

during validation.

---

# 125. Compilation DoS

Compiling malicious/huge WASM can consume resources.

Use:

```text
size limits
compilation concurrency limits
timeouts
```

---

# 126. Cache Poisoning

Cache key includes digest and engine config.

Never trust unverified externally supplied precompiled artifact.

---

# 127. Package Signature

Verify before expensive compilation where possible.

---

# 128. Publisher Trust

Part 21 rules apply:

```text
signed does not mean unrestricted
```

---

# 129. Update Permission Changes

New component version requesting new capability:

```text
requires re-approval
```

---

# 130. Component State Migration

State schema version:

```text
v1 → v2
```

Migration can be another controlled component entrypoint.

---

# 131. Migration Budget

Migration has:

```text
higher bounded CPU
storage transaction
deadline
```

---

# 132. Crash-Safe Migration

Host owns transaction boundary.

If migration traps:

```text
rollback
keep old state/version
```

where storage backend supports.

---

# 133. Component Rollback

Only if old component can read current state schema.

Otherwise require migration rollback or state snapshot restore.

---

# 134. Extension Integration

Part 21 protocol extension can be implemented as:

```text
WASM component
```

with imports:

```text
peer-session
state
files
diagnostics
```

---

# 135. Extension Session Resource

Host can expose:

```wit
resource peer-session
```

Methods:

```text
send-frame
open-stream
peer-trust
close
```

---

# 136. Session Security

Native core completes:

```text
peer authentication
extension negotiation
authorization
```

before handing session resource to component.

---

# 137. Component Cannot Open Arbitrary Peer

It requests:

```text
authorized peer reference
```

from host API.

Host verifies policy.

---

# 138. Protocol Frame Limits

Host validates:

```text
size
rate
extension namespace
session state
```

before transmission.

---

# 139. Protocol Parser Placement

Untrusted extension-specific parser may live inside WASM.

Core framing parser stays native and bounded.

---

# 140. Two-Layer Parsing

```text
Native:
frame length
extension ID
protocol version

WASM:
extension payload semantics
```

This is a strong isolation model.

---

# 141. Malformed Payload

If WASM parser traps:

```text
extension session fails
```

not daemon.

---

# 142. Fuzzing WASM Parsers

Use Part 10 corpus against:

```text
native host harness
WASM component parser
```

---

# 143. Differential Testing

Compare:

```text
native reference implementation
WASM implementation
```

when both exist.

---

# 144. WASM Reference Implementations

Portable components can serve as canonical protocol references.

This is useful for external interoperability.

---

# 145. Part 23 Conformance

External interoperability suite can execute reference WASM components as:

```text
oracle/reference behavior
```

for selected protocol semantics.

---

# 146. FFI Integration

Part 19 foreign-language apps do not need to know whether feature logic is native or WASM.

They use same SDK API.

---

# 147. Component Management API

Host-side:

```text
install
enable
disable
update
list
inspect
```

Part 24 may own distribution UI/catalog.

---

# 148. Local Developer Mode

Allow:

```text
unsigned local component
debug imports
hot reload
```

only in explicit development mode.

---

# 149. Hot Reload

For stateless components:

```text
swap immediately
```

For stateful/session components:

```text
drain
snapshot/migrate
start new version
```

---

# 150. Production Update

Use:

```text
verify
stage
instantiate smoke test
migrate
activate
rollback on failure
```

---

# 151. Shadow Execution

Advanced:

```text
run old + new policy components
compare output
use old result
```

during canary.

Useful for safe routing/business-policy upgrades.

---

# 152. Canary Components

Enable new version for:

```text
small user/device percentage
```

where product supports.

---

# 153. Deterministic Shadowing

Works best for pure policy components.

---

# 154. Component Distribution

Component Model tooling can support packaging/distribution through registry-style systems.

Architecture should allow:

```text
local file
organization registry
OCI-backed package source
built-in bundle
```

without hard-coding one registry.

---

# 155. Offline Installation

Embedded/emergency deployments need:

```text
USB
LAN
bundled update package
```

with signature verification.

---

# 156. Package Manifest vs Component Metadata

Do not trust only embedded custom sections.

Use an external signed manifest binding:

```text
component digest
permissions
publisher
resource budget
```

---

# 157. SBOM

Third-party component package should include:

```text
dependencies
licenses
source/build metadata
```

where possible.

---

# 158. Reproducible Builds

Recommended for trusted/high-risk components.

---

# 159. Component Risk Classes

```rust
pub enum ComponentRiskClass {
    Pure,
    Stateful,
    Networked,
    Sensitive,
}
```

---

# 160. Pure

Imports:

```text
clock maybe
no storage/network
```

Lowest risk.

---

# 161. Stateful

Uses:

```text
component state
```

but no peer/network.

---

# 162. Networked

Uses:

```text
peer protocol
files
DTN
```

Requires stronger review.

---

# 163. Sensitive

Uses:

```text
contact metadata
secret storage
organization authority hooks
```

Strongest governance.

---

# 164. No Authority Component by Default

A WASM component should not directly hold:

```text
root account signing authority
emergency authority key
```

Prefer host-mediated signing after policy verification.

---

# 165. Host-Mediated Signature

Component asks:

```text
sign this typed operation
```

Host validates operation class and identity permission.

---

# 166. Signing Oracle Risk

Do not expose:

```text
sign arbitrary bytes
```

for sensitive identity key.

Use domain-separated typed signing APIs.

---

# 167. Domain Separation

Example:

```text
sign-extension-event
sign-authorized-document
```

not raw signing.

---

# 168. Secure Store Handle

WIT resource can represent:

```text
secret-handle
```

with limited operations.

---

# 169. Component-to-Component Composition

Component Model allows composition.

Use cautiously.

Example:

```text
ERP workflow component
imports
document validation component
```

---

# 170. Dependency Graph

Host resolves:

```text
component dependencies
WIT versions
publisher policy
```

---

# 171. No Hidden Transitive Permissions

If component A imports component B:

```text
B does not automatically inherit all A host capabilities
```

Capability graph must remain explicit.

---

# 172. Dependency Resource Budget

Transitive components consume parent/global quota.

---

# 173. Dependency Failure

If optional dependency fails:

```text
degrade
```

Required dependency:

```text
component unavailable
```

Core runtime unaffected.

---

# 174. Dependency Cycle

Reject unsupported cycles.

---

# 175. Component Registry

Runtime registry tracks:

```text
id
digest
version
world
publisher
permissions
state
compiled cache
```

---

# 176. Registry State

```rust
pub enum ComponentInstallState {
    Installed,
    Enabled,
    Disabled,
    Failed,
    Quarantined,
    UpdatePending,
}
```

---

# 177. Startup

Daemon startup order:

```text
core recovery
 ↓
identity/network minimum ready
 ↓
component registry
 ↓
validate enabled components
 ↓
instantiate required components
 ↓
optional components lazily
```

---

# 178. Optional Components Must Not Block Ready

A broken optional component must not prevent messaging core from becoming ready.

---

# 179. Required Product Component

Some product may require one component.

Then:

```text
product feature unavailable
```

but runtime health should distinguish component failure from core corruption.

---

# 180. Headless Operation

All component management and execution works without Dioxus.

CLI:

```text
comm component list
comm component inspect
comm component enable
comm component disable
comm component doctor
```

---

# 181. Dioxus UI

Dioxus may present:

```text
component list
permissions
resource use
health
```

but the component core is UI-neutral.

---

# 182. Permission UX

Example:

```text
This component can:
• store up to 50 MB
• exchange protocol data with trusted peers
• access file references you explicitly share

It cannot:
• read your private identity keys
• open arbitrary network sockets
```

---

# 183. Diagnostics UI

Show:

```text
Running
Memory 12/32 MB
State 4 MB
Last trap none
Version 1.3.2
```

---

# 184. Browser Permission UX

Browser host may additionally depend on:

```text
browser storage permission
notifications
user gesture
```

but maps them into same domain capability semantics.

---

# 185. Serialization

At WIT boundary, prefer typed Component Model values.

Do not serialize everything into Postcard merely to cross WASM boundary.

---

# 186. Postcard Inside Extension Protocol

Still useful for:

```text
wire payload
durable compact extension event
```

if protocol defines it.

---

# 187. RON

Useful for:

```text
manifest
developer configuration
test scenario
```

not component call hot path.

---

# 188. JSON

Use for:

```text
external diagnostic/export
browser tooling
```

only where interoperability value outweighs overhead.

---

# 189. Host ↔ Component Copies

Measure copy cost.

For small structured values:

```text
fine
```

For large blobs:

```text
resource/stream handles
```

---

# 190. Batching

Reduce boundary crossings by:

```text
batching events
batching policy candidates
batching telemetry
```

---

# 191. Routing Policy Batch

Instead of:

```text
one WASM call per path metric
```

send:

```text
all candidates in one decision input
```

---

# 192. Collaboration Event Batch

Group small operations where latency permits.

---

# 193. Performance Budgets

Benchmark:

```text
instantiation
call overhead
WIT conversion
memory use
host call latency
```

on:

```text
desktop
server
Raspberry Pi-class device
```

---

# 194. Do Not Use WASM Where Slower by Design

If benchmark shows hot-path regression with no isolation benefit:

```text
keep native
```

---

# 195. Safety > Micro-Performance for Third-Party Logic

For untrusted extension code:

```text
moderate WASM overhead
```

is usually worth isolation.

---

# 196. Mobile Runtime Decision

Do not require embedding a heavyweight WASM runtime in every mobile build unless actual product extensions need it.

Feature-gate:

```text
wasm-components
```

---

# 197. Mobile Built-In Portable Logic

If a component is first-party and static, it may also be compiled as native Rust on mobile while using the same logical interface.

---

# 198. Dual Implementation Strategy

For selected components:

```text
native Rust implementation
+
WASM implementation
```

both satisfy shared test vectors.

Use native where sandbox not needed.

---

# 199. Portable Trait Model

Internal Rust can define:

```rust
pub trait PolicyComponent {
    fn evaluate(&self, input: PolicyInput) -> Result<PolicyOutput, PolicyError>;
}
```

Adapters:

```text
NativePolicyComponent
WasmPolicyComponent
```

---

# 200. Backend Transparency

Callers do not care whether implementation is:

```text
native
WASM
remote
```

as long as semantic contract matches.

---

# 201. Native Fallback

If WASM runtime unavailable on constrained platform:

```text
built-in native components still work
```

Third-party WASM feature can be marked unsupported.

---

# 202. Browser Fallback

If a WIT host capability is unavailable:

```text
component install/load reports capability missing
```

not runtime crash.

---

# 203. Capability Compatibility

Component manifest:

```text
required capabilities
optional capabilities
```

Host negotiates local availability at install/activation time.

---

# 204. Required Capability Missing

Result:

```text
IncompatibleHost
```

with diagnostic explanation.

---

# 205. Optional Capability Missing

Component loads with feature disabled.

---

# 206. Host Version Negotiation

Component declares:

```text
comm:extension@1
comm:files@2
```

Host selects compatible interface versions.

---

# 207. Multiple WIT Versions

Host may provide:

```text
v1 adapter
v2 native
```

during migration window.

---

# 208. Compatibility Shim

Use explicit host adapters.

Do not fake old semantics if safety changed.

---

# 209. Component Conformance Package

Each stable component can ship:

```text
manifest
WIT
golden inputs
expected outputs
fuzz corpus
state migration fixtures
```

---

# 210. Cross-Language Components

Because the Component Model is language-neutral, extension authors may eventually build components from:

```text
Rust
C/C++
JavaScript
Python
C#
TinyGo
other supported toolchains
```

Host contract remains WIT.

---

# 211. Language Support Policy

Do not promise every language is equally production-ready.

Officially support only toolchains that pass conformance and packaging tests.

---

# 212. Rust Component SDK

Provide ergonomic crate:

```text
comm-component-sdk
```

that generates/uses WIT bindings.

---

# 213. SDK APIs

```text
state
files
peer session
diagnostics
time
random
notifications
```

---

# 214. No Native Rust Dependency Leakage

A third-party Rust component should not need:

```text
comm-runtime internals
```

Only public SDK/WIT contracts.

---

# 215. Browser SDK

Generate TypeScript/JS bindings where component tooling supports.

---

# 216. Component Test CLI

```text
comm-component test component.wasm
```

Runs:

```text
validation
manifest
permissions
golden vectors
resource tests
```

---

# 217. Component Doctor

```text
comm component doctor org.example.whiteboard
```

Reports:

```text
signature
WIT compatibility
permissions
memory budget
state schema
last traps
```

---

# 218. Security Tests

Required:

```text
memory exhaustion
infinite loop
host-call flood
storage quota bypass
undeclared import
forged package
permission escalation
secret access
priority escalation
```

---

# 219. Infinite Loop Test

Component:

```text
loop {}
```

Expected:

```text
fuel/deadline trap
daemon healthy
```

---

# 220. Memory Bomb Test

Component repeatedly grows memory.

Expected:

```text
memory limit trap
```

---

# 221. Host Call Flood

Component requests thousands of expensive operations.

Expected:

```text
rate limit
quota
```

---

# 222. Storage Escape Test

Try keys/paths designed to escape namespace.

Expected:

```text
impossible/rejected
```

---

# 223. Network Escape Test

Component without external network permission cannot create arbitrary sockets.

---

# 224. Priority Escalation Test

Third-party component requests AuthorityCritical.

Expected:

```text
Part 17 rejection/downgrade
```

---

# 225. Identity Key Test

No component API returns raw long-term private identity key.

---

# 226. Trap Recovery Test

Component traps during peer session.

Expected:

```text
session closed
resources released
core transport survives
```

---

# 227. Migration Trap Test

Expected:

```text
old state retained
new version not activated
```

---

# 228. Update Rollback Test

New component fails smoke test.

Expected:

```text
old version remains active
```

---

# 229. Browser Parity Test

Same pure component input on native/browser hosts.

Expected:

```text
same semantic result
```

---

# 230. Embedded Test

Run component with low memory/fuel profile on ARM hardware.

---

# 231. Performance Test

Ensure policy component cannot materially delay critical routing beyond configured deadline.

Timeout fallback:

```text
native default policy
```

---

# 232. Fail-Closed vs Fail-Open

Per component function define fallback.

Security policy component:

```text
fail closed
```

Optional routing preference:

```text
fall back to native default
```

---

# 233. Component Failure Policy

```rust
pub enum ComponentFailurePolicy {
    FailClosed,
    FallbackNative,
    DisableFeature,
    RetryLater,
}
```

---

# 234. Security-Sensitive Components

Require:

```text
trusted publisher
strict limits
fail-closed
audit
```

---

# 235. Optional UX Components

Can:

```text
disable feature
```

without affecting core.

---

# 236. Audit Events

Record:

```text
installed
updated
permission granted
permission revoked
quarantined
publisher changed
```

---

# 237. No User Payload in Audit

Do not log component message/file contents.

---

# 238. Backup

Host backup can include:

```text
component manifests
enabled state
component persistent state
```

Compiled cache excluded.

---

# 239. Restore

If component missing:

```text
retain namespaced state
```

until compatible component installed.

---

# 240. Uninstall

Options:

```text
keep state
export
delete
```

according to product/admin policy.

---

# 241. Secret State Cleanup

Use best-effort deletion and revoke associated host credentials/tokens.

---

# 242. Ecosystem Boundary

Part 22 defines:

```text
portable execution and host interfaces
```

Part 21 defines:

```text
third-party protocol semantics
```

Part 24 can define:

```text
plugin packaging/catalog/governance
```

These layers must remain separate.

---

# 243. Recommended Production Defaults

```text
native core always enabled
WASM component runtime feature-gated
no raw sockets
no arbitrary filesystem
32–64 MB default third-party memory ceiling
bounded CPU/fuel
namespaced state
publisher verification
explicit permissions
quarantine on repeated traps
```

Actual memory values should be tuned per device profile.

---

# 244. Embedded Defaults

Tiny node:

```text
WASM runtime disabled
or only a few trusted AOT components
```

Standard edge node:

```text
sandboxed components enabled
strict memory/fuel
```

---

# 245. Desktop Defaults

Allow:

```text
sandboxed signed third-party components
```

if product ecosystem enables plugins.

---

# 246. Server Defaults

Allow:

```text
tenant-scoped components
```

only with strong resource isolation.

---

# 247. Mobile Defaults

Initially:

```text
built-in native logic
```

WASM runtime enabled only if real third-party extension requirement exists.

This controls binary size and complexity.

---

# 248. Browser Defaults

Only components whose required WIT interfaces have browser implementations are loadable.

---

# 249. Initial Production Scope

Implement first:

```text
WIT package definitions
comm-component-core
Wasmtime host adapter
manifest + permission model
memory limits
fuel/deadline execution limits
state storage
diagnostics
pure/stateless component support
Part 21 extension component support
testkit
```

Then:

```text
stream/resource APIs
browser host
AOT cache
stateful component migrations
package signing
component composition
cross-language SDKs
```

Defer initially:

```text
raw WASI socket access
arbitrary filesystem access
full UI components
unrestricted external HTTP
mobile third-party component marketplace
```

---

# 250. Implementation Phases

## Phase 1 — Portable Contracts

```text
WIT packages
component manifest
errors
permissions
```

## Phase 2 — Native Host

```text
Wasmtime
instantiation
memory limiter
fuel/deadline
```

## Phase 3 — Host Capabilities

```text
state
clock
random
diagnostics
```

## Phase 4 — Extension Integration

```text
peer session
files
DTN
routing request
```

## Phase 5 — Persistence

```text
state schema
migration
rollback
```

## Phase 6 — Distribution/Security

```text
signature
publisher trust
digest cache
```

## Phase 7 — Browser Portability

```text
portable host
JS/browser adapters
```

## Phase 8 — Hardening

```text
fuzz
resource abuse
trap recovery
cross-host conformance
embedded tests
```

---

# 251. Definition of Done

Part 22 is complete when:

- selected protocol/business logic can run as Component Model components
- WIT is the stable host/guest contract
- native networking/hardware hot paths remain native
- components receive only explicitly granted capabilities
- arbitrary filesystem/network/process access is denied by default
- component memory is bounded
- component execution time/instructions are bounded
- a trapped component cannot crash the daemon
- persistent state is host-owned and namespaced
- extension private keys/core identity keys are not exposed
- large blobs use resources/streams rather than giant WASM copies
- Part 21 extensions can be implemented as sandboxed components
- Part 17 emergency priority remains host-authorized
- Part 13/08 resource and battery policy can throttle components
- optional component failure does not prevent core runtime readiness
- native and browser hosts can run the same portable component where required interfaces exist
- component updates/migrations can rollback safely
- package signatures and publisher trust can be enforced
- fuzz, memory bomb, infinite loop, host-call flood, permission escape, migration, and cross-host conformance tests exist

---

# 252. Relationship to Earlier Parts

Part 22 builds on:

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
21 — Third-Party Protocol Extensions
```

It prepares:

```text
23 — External Interoperability Suite
24 — Plugin / Module Ecosystem
```

---

# 253. Final Architecture

```text
                      NATIVE RUST CORE
 ┌─────────────────────────────────────────────────────────┐
 │ Identity  Iroh  Routing  Files  DTN  Proximity  Media  │
 └───────────────────────┬─────────────────────────────────┘
                         │
                 Capability Host APIs
                         │
                    WIT Interfaces
                         │
                ┌────────┴─────────┐
                │                  │
         WASM Component A    WASM Component B
          Routing Policy       ERP Workflow
                │                  │
                └────────┬─────────┘
                         │
                    Sandboxed
                  Resource Limits
```

Portable deployment:

```text
Same component logic
   ├── Desktop Wasmtime host
   ├── Server Wasmtime host
   ├── Embedded Linux host
   └── Browser-compatible host
```

only where its imported capabilities are implemented.

---

# 254. Final Principle

The system should be able to reuse a component such as:

```text
"school ERP notification protocol"
```

in:

```text
desktop app
headless server
Raspberry Pi gateway
browser-compatible management client
```

without giving that component unrestricted access to:

```text
the filesystem
raw sockets
identity private keys
Bluetooth
OS processes
```

At the same time, the native Rust communication engine should continue to do the work it is best at:

```text
Iroh
QUIC
BLE/Wi-Fi
file streaming
hardware integration
AV1/media
daemon lifecycle
```

The architectural split is therefore:

```text
Native Rust:
performance + hardware + trusted core

WASM Components:
portability + isolation + extensibility
```

That is the purpose of Part 22: gain WebAssembly portability and sandboxing where they provide real architectural value without sacrificing the efficiency, native integration, and reliability of the core P2P platform.
