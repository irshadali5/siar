# Part 19 — C ABI / FFI Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 19 of 24  
**Primary implementation language:** Rust  
**Primary ABI:** C-compatible ABI  
**Primary goals:** stable cross-language integration, safe ownership, ABI/version stability, async interop, zero/low-copy data paths, mobile/desktop bindings, daemon/library dual mode, error isolation, panic containment, reusable SDK distribution

---

# 1. Purpose

The communication platform is written primarily in Rust, but reusable infrastructure becomes much more valuable if other products can consume it without rewriting their entire application in Rust.

Potential consumers include:

```text
C
C++
Kotlin / Java
Swift / Objective-C
Python
Dart / Flutter
Go
C#
Node.js
other native runtimes
```

The architecture therefore needs a stable foreign-function boundary.

The core rule is:

> **Rust internals remain Rust; the public cross-language contract is a small, explicit, versioned C ABI built around opaque handles and stable data-transfer rules.**

Do not expose:

```text
Rust struct layout
Rust enums directly
Arc<T>
Box<T> internals
Vec<T> internals
String internals
trait objects
async Future
panic unwinding
```

across the ABI.

---

# 2. Why C ABI First

C ABI is the most portable native interoperability layer.

It can be consumed by:

```text
C/C++
JNI/JNA
Swift/Objective-C
Python ctypes/cffi
Dart FFI
C#
Go cgo
Node native addons
```

This gives one stable low-level boundary instead of independently designing incompatible bindings for each language.

---

# 3. Architectural Position

```text
                 Rust Core
     ┌────────────────────────────┐
     │ comm-runtime               │
     │ comm-messaging             │
     │ comm-files                 │
     │ comm-dtn                   │
     │ comm-routing               │
     │ comm-diagnostics           │
     └──────────────┬─────────────┘
                    │
              comm-ffi-core
                    │
               C ABI Layer
       ┌────────────┼─────────────┐
       │            │             │
     C/C++       Kotlin/Java    Swift
       │            │             │
     Python       Android         iOS/macOS
       │
   Dart/Flutter
```

---

# 4. FFI Scope

The C ABI should expose high-level platform operations:

```text
runtime creation
runtime start/stop
identity/device operations
send message
send file
query status
subscribe to events
network diagnostics
DTN status
bootstrap/pairing
```

It should not expose every internal crate function.

---

# 5. Stable Boundary Principle

The ABI should remain stable even if internal Rust architecture changes.

For example:

```text
Rust internal:
HashMap<DeviceId, Arc<PeerRuntime>>

C ABI:
comm_peer_handle_t
```

Internal layout can change freely.

---

# 6. Opaque Handles

Represent internal objects as opaque handles.

Example C:

```c
typedef struct comm_runtime comm_runtime_t;
typedef struct comm_client comm_client_t;
typedef struct comm_transfer comm_transfer_t;
typedef struct comm_subscription comm_subscription_t;
```

Actual memory layout remains private.

---

# 7. Handle Ownership

Every handle must have explicit lifecycle.

Example:

```c
comm_runtime_t* comm_runtime_create(...);
void comm_runtime_destroy(comm_runtime_t*);
```

No ambiguous ownership.

---

# 8. RAII Wrappers in High-Level Bindings

C API is manual.

Generated/manual language wrappers should convert to native resource models:

```text
C++ → RAII class
Swift → deinit wrapper
Kotlin → Closeable
Python → context manager/finalizer
Dart → Finalizer/explicit close
```

---

# 9. Handle Registry vs Raw Pointer

Two possible designs:

## Raw opaque pointer

```text
fast
simple
```

but invalid pointer misuse is dangerous.

## Integer handle registry

```text
safer validation
generation checking
cross-thread use easier
```

Recommended for public FFI:

```rust
pub struct FfiHandle {
    index: u32,
    generation: u32,
}
```

Expose as 64-bit integer.

---

# 10. Handle Type

```c
typedef uint64_t comm_handle_t;
```

Special value:

```text
0 = invalid/null handle
```

---

# 11. Typed Handles

Avoid one untyped global namespace when possible.

C typedef aliases:

```c
typedef comm_handle_t comm_runtime_handle_t;
typedef comm_handle_t comm_transfer_handle_t;
typedef comm_handle_t comm_subscription_handle_t;
```

Runtime validates object type internally.

---

# 12. Generation Counter

Handle generation prevents stale-handle reuse.

Example:

```text
slot 42 generation 3
destroyed
slot 42 reused generation 4

old handle gen 3
→ invalid
```

---

# 13. Handle Registry

```rust
pub struct HandleRegistry {
    // bounded typed slabs / arenas
}
```

Requirements:

```text
thread-safe
generation checked
bounded
no use-after-free
```

---

# 14. FFI Object Graph

Avoid returning nested borrowed pointers.

Prefer handles and copied/owned DTOs.

Example:

```text
RuntimeHandle
TransferHandle
SubscriptionHandle
```

---

# 15. ABI Versioning

Expose:

```c
uint32_t comm_abi_version_major(void);
uint32_t comm_abi_version_minor(void);
```

---

# 16. Major Version Rule

Breaking C ABI change:

```text
major++
```

Examples:

```text
function signature change
struct field reorder
enum value meaning change
ownership contract change
```

---

# 17. Minor Version Rule

Backward-compatible additions:

```text
new function
new optional feature
new enum value where extensible
```

---

# 18. ABI Compatibility Query

```c
comm_result_t comm_check_abi_compat(
    uint32_t required_major,
    uint32_t required_minor
);
```

---

# 19. Symbol Naming

Prefix all symbols:

```text
comm_
```

Never export generic names like:

```text
init
send
free
```

---

# 20. Symbol Visibility

Hide all non-public symbols.

Build shared library with only intended C API exports.

---

# 21. Header Generation

Maintain one canonical public header:

```text
include/comm.h
```

Can be generated with:

```text
cbindgen
```

but generation must be reviewed and ABI-tested.

---

# 22. C Header Rule

Public header should use only C-compatible types:

```text
stdint.h
stddef.h
stdbool.h where appropriate
```

Avoid compiler-specific layout unless explicitly guarded.

---

# 23. Primitive Types

Use fixed-width integers:

```text
uint8_t
uint16_t
uint32_t
uint64_t
int32_t
```

Avoid:

```text
long
usize
isize
```

in stable ABI.

---

# 24. Boolean Type

Prefer a fixed-width integer representation where cross-binding ambiguity exists.

Do not rely on language-specific bool ABI assumptions everywhere.

---

# 25. Enums

Expose enums as fixed-width integer typedefs.

Example:

```c
typedef uint32_t comm_runtime_state_t;
```

Constants:

```c
#define COMM_RUNTIME_READY 1u
#define COMM_RUNTIME_DEGRADED 2u
```

This gives better forward compatibility.

---

# 26. Unknown Enum Values

Consumers must tolerate unknown values.

Bindings should map:

```text
known enum
or
Unknown(raw)
```

where language supports.

---

# 27. Struct Layout

If exporting structs directly, require:

```rust
#[repr(C)]
```

Use direct structs only for:

```text
small immutable POD-like DTOs
```

Avoid complex nested layouts.

---

# 28. Extensible Struct Pattern

Stable config structs can use:

```c
typedef struct {
    uint32_t struct_size;
    uint32_t version;
    ...
} comm_runtime_config_t;
```

This supports adding fields later.

---

# 29. `struct_size`

Caller sets:

```text
sizeof(comm_runtime_config_t)
```

Library reads only fields available in caller's version.

---

# 30. Strings

Use UTF-8 length-delimited bytes.

```c
typedef struct {
    const uint8_t* ptr;
    size_t len;
} comm_bytes_view_t;
```

No NUL-termination requirement for the primary ABI.

---

# 31. Borrowed Inputs

For synchronous input:

```text
caller owns memory
library reads only during the call
```

unless explicit transfer-of-ownership API says otherwise.

---

# 32. Owned Output Buffers

```c
typedef struct {
    uint8_t* ptr;
    size_t len;
    size_t cap;
} comm_owned_buffer_t;
```

Must be released by Rust:

```c
void comm_buffer_free(comm_owned_buffer_t buffer);
```

Never free with a foreign allocator.

---

# 33. Allocator Boundary

Memory allocated by Rust is freed by Rust.

Memory allocated by caller remains caller-owned unless explicit ownership transfer occurs.

---

# 34. Large Data Paths

Do not move multi-gigabyte files through generic copied buffers.

Prefer:

```text
file path
file descriptor/handle
stream source
stream sink
```

---

# 35. Borrowed Callback Memory

If an event callback receives borrowed data:

```text
valid only until callback returns
```

Provide owned-copy helpers where needed.

---

# 36. Callback API

```c
typedef void (*comm_event_callback_t)(
    const comm_event_t* event,
    void* user_data
);
```

Rules:

```text
callback may not block core networking
user_data is opaque
callback thread semantics are documented
```

---

# 37. Callback Dispatch

Recommended:

```text
internal event
 ↓
bounded queue
 ↓
dedicated FFI dispatcher
 ↓
foreign callback
```

Do not call foreign callbacks while holding important runtime locks.

---

# 38. Callback Backpressure

Slow consumers must not block networking.

Use:

```text
bounded queues
coalescing
drop stale ephemeral events
resync-required events
```

---

# 39. Subscription Handle

```c
comm_result_t comm_subscribe(
    comm_runtime_handle_t runtime,
    uint32_t topic,
    comm_event_callback_t callback,
    void* user_data,
    comm_subscription_handle_t* out_subscription
);
```

Cancellation:

```c
comm_result_t comm_subscription_cancel(
    comm_subscription_handle_t subscription
);
```

---

# 40. Polling Alternative

Provide event polling for runtimes where callbacks are awkward.

```c
comm_result_t comm_poll_event(
    comm_subscription_handle_t subscription,
    comm_event_t* out_event
);
```

---

# 41. Async Model

Rust `Future` never crosses the C ABI.

Use:

```text
operation handle
poll/status
callback completion
cancel
```

---

# 42. Operation Handle

```c
typedef comm_handle_t comm_operation_handle_t;
```

Start operation:

```c
comm_result_t comm_send_file_async(
    comm_runtime_handle_t runtime,
    ...,
    comm_operation_handle_t* out_operation
);
```

---

# 43. Operation Status

```c
comm_result_t comm_operation_status(
    comm_operation_handle_t operation,
    comm_operation_status_t* out_status
);
```

---

# 44. Cancellation

```c
comm_result_t comm_operation_cancel(
    comm_operation_handle_t operation
);
```

This maps to structured Rust cancellation.

---

# 45. High-Level Async Wrappers

Bindings convert operation handles into:

```text
Kotlin suspend
Swift async/await
Python asyncio
Dart Future
C# Task
Java CompletableFuture
```

---

# 46. Error Model

Use stable numeric error codes.

```c
typedef uint32_t comm_error_code_t;
```

Success:

```text
0
```

---

# 47. Error Domains

Examples:

```text
Runtime
Network
Identity
Storage
Files
DTN
Capability
Permission
Resource
Bootstrap
Diagnostics
```

---

# 48. Rich Error Information

An error object may expose:

```text
domain
code
retryable
user-action hint
diagnostic ID
```

Do not make error strings the programmatic API.

---

# 49. No Panic Across FFI

Every exported function must prevent Rust unwinding across the ABI.

Use boundary panic protection where unwind strategy permits.

Unexpected panic maps to a stable internal error or process abort according to configured panic strategy.

---

# 50. In-Process vs Daemon FFI

Support both.

## In-Process

```text
Foreign App
 ↓
C ABI
 ↓
Rust Runtime
```

## Daemon-Backed

```text
Foreign App
 ↓
C ABI client
 ↓
Local IPC
 ↓
comm-daemon
```

Daemon mode provides stronger crash and language-runtime isolation.

---

# 51. Unified SDK Semantics

The same high-level operations should exist across both backends:

```text
send message
send file
query state
subscribe
bootstrap
diagnostics
```

---

# 52. FFI Workspace

```text
crates/
├── comm-ffi-types/
├── comm-ffi-core/
├── comm-ffi-daemon-client/
├── comm-ffi-platform/
└── comm-sdk-model/

bindings/
├── c/
├── cpp/
├── kotlin/
├── swift/
├── python/
├── dart/
├── csharp/
└── go/

include/
└── comm.h
```

---

# 53. `comm-ffi-types`

Contains only:

```text
repr(C) DTOs
constants
versions
error codes
```

No runtime behavior.

---

# 54. `comm-ffi-core`

Owns:

```text
handle registry
pointer validation
buffer helpers
callback dispatch
panic boundaries
runtime mappings
```

---

# 55. `comm-ffi-daemon-client`

Maps the same public API onto Part 16 IPC.

---

# 56. `comm-sdk-model`

Contains stable high-level Rust DTOs before conversion into C structures.

This decouples domain internals from ABI internals.

---

# 57. C++ Binding

Wrap C handles in RAII.

Properties:

```text
move-only where ownership is unique
automatic release
typed results
no raw free()
```

---

# 58. Kotlin / Android Binding

Architecture:

```text
Kotlin
 ↓
thin JNI layer
 ↓
C ABI / FFI adapter
 ↓
Rust Runtime
```

Keep JNI narrow.

---

# 59. Kotlin Coroutines

Map async operation handles to:

```kotlin
suspend fun
```

and events to:

```kotlin
Flow<Event>
```

with bounded buffering.

---

# 60. Android Platform Objects

Some objects cannot be represented portably:

```text
Context
Uri
Network
ParcelFileDescriptor
```

Use a platform-specific JNI adapter.

Do not contaminate the generic C ABI with Android classes.

---

# 61. Android Large Files

Prefer:

```text
ParcelFileDescriptor
→ duplicated raw FD
→ Rust FileSource
```

rather than copying through JVM arrays.

---

# 62. Swift Binding

Architecture:

```text
Swift
 ↓
C module / wrapper
 ↓
C ABI
 ↓
Rust
```

Map operations to:

```text
async throws
```

and events to:

```text
AsyncStream
```

where practical.

---

# 63. Swift Memory

Convert Rust-owned buffers to Swift `Data`/`String`, then free through Rust API.

---

# 64. Python Binding

Options:

```text
cffi
ctypes
PyO3 wrapper over service APIs
```

For stable ABI distribution, a C ABI-backed wrapper is attractive.

---

# 65. Python Async

Use:

```text
asyncio Future
```

and ensure callbacks enter Python safely with GIL handling.

---

# 66. Dart / Flutter Binding

Use:

```text
dart:ffi
```

Map:

```text
operation handle → Future
subscription → Stream
```

Respect Dart isolate/threading rules.

---

# 67. Go Binding

Use cgo if needed, but daemon mode can be cleaner for many Go server applications.

---

# 68. C# Binding

Use P/Invoke.

Map operation handles to `Task`.

---

# 69. Node.js Binding

Use N-API wrapper or daemon-client wrapper.

Do not block the Node event loop with synchronous native network calls.

---

# 70. Thread-Safety Contract

Every handle type must document:

```text
thread-safe
thread-confined
serialized internally
```

Recommended defaults:

```text
runtime/client handles → thread-safe
callbacks → serialized per subscription
temporary builder objects → may be thread-confined
```

---

# 71. Reentrancy

Foreign callbacks may call query APIs.

Avoid invoking callbacks while holding global internal locks.

---

# 72. Async Runtime Ownership

Create one long-lived async runtime per in-process runtime instance, not per FFI call.

---

# 73. Runtime Lifecycle API

```c
comm_result_t comm_runtime_create(
    const comm_runtime_config_t* config,
    comm_runtime_handle_t* out_runtime
);

comm_result_t comm_runtime_start(
    comm_runtime_handle_t runtime
);

comm_result_t comm_runtime_stop(
    comm_runtime_handle_t runtime
);

comm_result_t comm_handle_release(
    comm_handle_t handle
);
```

---

# 74. Shutdown Idempotency

Repeated stop/release must never double-free.

Return a stable invalid/already-closed result where appropriate.

---

# 75. Configuration DTO

Use versioned extensible C structs.

Example:

```c
typedef struct {
    uint32_t struct_size;
    uint32_t api_version;
    uint32_t runtime_mode;
    uint32_t log_level;
    uint64_t memory_budget;
} comm_runtime_config_t;
```

---

# 76. Typed DTO vs Binary Envelope

Use typed C DTOs for:

```text
common stable API
```

Use binary/versioned envelopes for:

```text
extension-specific payloads
large evolving diagnostic data
```

---

# 77. Postcard at the FFI Edge

Postcard remains excellent internally, but foreign SDK consumers should not be forced to implement Postcard for basic operations.

Use it only for explicit advanced binary extension APIs.

---

# 78. JSON at the FFI Edge

JSON is acceptable for:

```text
diagnostic export
external admin tooling
human/tool interoperability
```

Not as the main high-throughput core ABI.

---

# 79. RON at the FFI Edge

RON is suitable for:

```text
configuration
developer tools
scenario files
```

not primary cross-language runtime communication.

---

# 80. Stable IDs

Expose fixed-size IDs.

Example:

```c
typedef struct {
    uint8_t bytes[16];
} comm_id128_t;
```

Use 32-byte equivalent when domain IDs require it.

---

# 81. Time Types

Use documented fixed-width integer timestamps.

Example:

```text
int64_t Unix milliseconds
```

Use monotonic time internally for durations, not across ABI.

---

# 82. Optional Values

Use explicit:

```text
has_value
value
```

for C DTOs where no safe sentinel exists.

---

# 83. Arrays

Inputs:

```c
const T* ptr;
size_t len;
```

Outputs:

```text
owned buffer
or
explicit freeable array
```

All lengths validated before allocation.

---

# 84. Null Semantics

Every pointer argument must specify whether:

```text
null allowed
null + len 0 allowed
null always invalid
```

---

# 85. FFI Security

Foreign callers are not automatically trusted.

Validate:

```text
handle
pointer
length
enum value
struct_size
version
```

before expensive work.

---

# 86. Double Destroy

A second destroy/release returns:

```text
InvalidHandle
```

not memory corruption.

---

# 87. Concurrent Destroy

Use registry state/reference counting so:

```text
thread A uses
thread B closes
```

results in typed failure, never UAF.

---

# 88. Handle State

```rust
pub enum HandleState {
    Live,
    Closing,
    Closed,
}
```

---

# 89. Releasing a View vs Cancelling Work

Important:

```text
release transfer handle
≠
cancel transfer
```

A GC finalizer must not accidentally cancel durable work.

---

# 90. Reacquiring Durable Operations

Foreign app should be able to query:

```text
TransferId
MessageId
EmergencyId
```

after UI/process restart.

---

# 91. Event Model

Common event DTOs may use:

```text
event type
sequence
version
payload
```

High-level bindings translate to native sealed enums/classes.

---

# 92. Event Ordering

Document:

```text
ordered per subscription
global total order not guaranteed unless explicitly provided
```

---

# 93. Event Gap Detection

Sequence numbers allow:

```text
gap detected
→ request snapshot
```

following Part 16 semantics.

---

# 94. Subscription Backpressure

Coalesce:

```text
transfer progress
network metrics
presence
```

Preserve:

```text
security state
terminal operation state
critical emergency events
```

---

# 95. Platform Vtable / Reverse FFI

Rust sometimes needs platform services.

Expose a narrow versioned vtable.

```c
typedef struct {
    uint32_t struct_size;
    uint32_t version;
    void* user_data;
    comm_result_t (*open_file)(...);
    comm_result_t (*notify)(...);
} comm_platform_vtable_t;
```

---

# 96. Reverse FFI Rule

Rust requests semantic platform actions.

It should not embed Android/iOS UI policy.

---

# 97. Optional Hooks

Null optional function pointer means:

```text
unsupported
```

Never crash.

---

# 98. Secure Store Bridge

For platform keys:

```text
Android Keystore
Apple Keychain/Secure Enclave
TPM
```

prefer opaque key handles and signing APIs over exporting private key bytes.

---

# 99. Key Separation

Never pass private identity keys into Kotlin/Swift just because it is convenient.

Keep key access behind secure-store abstraction.

---

# 100. Proximity FFI

Expose neutral:

```text
NearbyPeer
ProximityEvent
connect intent
```

not raw Android `BluetoothDevice`.

---

# 101. Bootstrap FFI

Expose:

```text
create invitation
get QR payload
accept payload
get SAS
approve/reject
```

Camera/NFC I/O stays platform-specific.

---

# 102. Diagnostics FFI

Expose:

```text
basic path/status DTOs
advanced diagnostics snapshot
network doctor
event stream
```

---

# 103. Emergency FFI

Expose high-level:

```text
send_sos
cancel_emergency
ack_emergency
```

Do not allow raw unauthenticated setting of `AuthorityCritical`.

---

# 104. Extension FFI

Third-party extensions should use a generic namespaced API rather than adding a new exported symbol for every extension.

Conceptually:

```c
comm_result_t comm_extension_call(
    comm_runtime_handle_t runtime,
    comm_bytes_view_t namespace_id,
    comm_bytes_view_t request,
    comm_operation_handle_t* out_operation
);
```

---

# 105. Plugin ABI Is Separate

Part 24 plugin/module ABI should be narrower than the full application ABI and designed for sandboxing.

Do not automatically expose all user data or runtime controls to plugins.

---

# 106. Native Artifact Matrix

Linux:

```text
libcomm.so
libcomm.a optional
comm.h
pkg-config
```

Windows:

```text
comm.dll
comm.lib
comm.h
PDB symbols
```

macOS:

```text
libcomm.dylib
or framework/XCFramework packaging
```

Android:

```text
.so per ABI
AAR wrapper
```

Apple mobile:

```text
XCFramework
Swift Package wrapper
```

---

# 107. Android Packaging

Official Android SDK can publish an AAR containing:

```text
Kotlin API
JNI bridge
native libraries
```

---

# 108. Apple Packaging

Prefer:

```text
XCFramework
+
Swift Package
```

Final production signing/build validation requires Apple toolchains.

---

# 109. Python Packaging

Provide wheels for supported targets when practical.

---

# 110. Flutter Packaging

Provide a Flutter plugin package with native binaries and platform shims.

---

# 111. C/C++ Packaging

Provide:

```text
header
shared/static library
CMake config
pkg-config
examples
```

---

# 112. SDK Versioning Dimensions

Keep separate:

```text
SDK semantic version
C ABI version
daemon IPC version
wire protocol version
storage schema version
```

Do not conflate them.

---

# 113. Version Info API

Expose:

```c
comm_version_info_t comm_version_info(void);
```

Include:

```text
SDK version
ABI version
build ID
```

---

# 114. Feature Capability Query

SDK exposes built-in capabilities.

A binding can ask whether this build includes:

```text
files
DTN
proximity
emergency
daemon backend
```

---

# 115. Official SDK ABI Policy

For official binaries, prefer one stable superset ABI.

Disabled features return:

```text
Unsupported
```

rather than disappearing unpredictably.

---

# 116. Binding Generation

Potential tooling:

```text
cbindgen
UniFFI
JNI code generation
Swift module maps
```

The tool is implementation detail.

The ABI contract must remain understandable independently.

---

# 117. Unsafe Code Concentration

Keep unsafe in a few modules:

```text
ffi_ptr.rs
ffi_buffer.rs
ffi_callback.rs
ffi_handle.rs
```

Core communication crates should remain safe Rust wherever possible.

---

# 118. Unsafe Documentation

Every unsafe block documents:

```text
preconditions
ownership
lifetime
threading
why operation is valid
```

---

# 119. Strict Lints

Enable:

```text
unsafe_op_in_unsafe_fn
missing_safety_doc
```

and related lints.

---

# 120. FFI Test Matrix

Required:

```text
C smoke
C++ RAII
Kotlin Android
Swift
Python
Dart/Flutter
```

as official bindings mature.

---

# 121. ABI Golden Tests

Persist expected:

```text
sizeof
alignof
field offsets
enum numeric values
symbol list
```

for stable C types.

---

# 122. ABI Diff CI

Compare shared-library ABI against previous release.

Unexpected break:

```text
CI failure
```

---

# 123. Header Diff CI

`comm.h` changes require explicit review.

---

# 124. Symbol Export CI

Ensure only approved symbols are exported.

---

# 125. C Smoke Test

Compile pure C example against released library.

This is mandatory for ABI confidence.

---

# 126. Fuzzing

Part 10 should fuzz:

```text
config structs
enum values
byte views
handle sequences
operation lifecycle
event payload decoding
```

---

# 127. Sequence Fuzzing

Generate:

```text
create
start
subscribe
send
cancel
release
destroy
use stale handle
```

Assert:

```text
no panic
no UAF
no double free
```

---

# 128. Sanitizers

Run FFI harnesses under:

```text
ASan
UBSan
TSan where supported
```

---

# 129. Miri

Use Miri for unsafe Rust helpers:

```text
pointer conversions
buffer ownership
handle registry
```

---

# 130. Reentrancy Tests

Callback calls back into:

```text
query runtime
cancel operation
```

No deadlock.

---

# 131. Slow Callback Test

Foreign callback intentionally blocks.

Expected:

```text
core network continues
event queue bounded
```

---

# 132. Concurrent Destroy Test

One thread uses a handle while another closes it.

Expected:

```text
typed error
no UAF
```

---

# 133. Panic Boundary Test

Inject internal panic in test build.

Expected:

```text
no unwind crosses C boundary
```

---

# 134. Memory Leak Test

Repeated:

```text
create/destroy
subscribe/cancel
operation handle acquire/release
```

must return handle counts to baseline.

---

# 135. Daemon Restart Test

Daemon-backed SDK:

```text
daemon restarts
client reconnects
durable operation remains
subscriptions resync
```

---

# 136. Binding Compatibility Test

Old binding binary should work with newer compatible ABI release.

---

# 137. Documentation Requirements

Every public function documents:

```text
ownership
nullability
threading
blocking behavior
callback behavior
error codes
version added
```

---

# 138. Example Applications

Provide:

```text
examples/c
examples/cpp
examples/android-kotlin
examples/swift
examples/python
examples/flutter
```

---

# 139. C Example Scope

Show:

```text
runtime start
subscribe
send message
shutdown
```

---

# 140. Kotlin Example Scope

Show:

```text
service-scoped runtime
Flow events
suspend send
large-file FD transfer
```

---

# 141. Swift Example Scope

Show:

```text
async send
AsyncStream events
```

---

# 142. Python Example Scope

Show:

```text
asyncio
context manager
event subscription
```

---

# 143. Flutter Example Scope

Show:

```text
Future send
Stream progress
dispose/reconnect
```

---

# 144. Distribution Manifest

Each SDK release should include:

```text
native binaries
headers
binding package
checksums
version info
license notices
compatibility notes
```

---

# 145. Deprecation Policy

Mark API deprecated before removal.

Breaking removal waits for next ABI major.

---

# 146. Error-Code Stability

Never reuse an existing error code for different meaning.

---

# 147. Enum Stability

Never reuse retired enum numeric values.

---

# 148. Struct Stability

Do not reorder fields inside same ABI major.

---

# 149. 32-bit Considerations

If supporting 32-bit Android/embedded:

```text
test size_t assumptions
pointer-width assumptions
alignment
```

Wire/protocol sizes remain fixed-width.

---

# 150. Endianness Boundary

In-process C ABI uses target-native representation.

Network/IPC serialization remains independently specified and canonical.

---

# 151. Performance

FFI should not become a per-byte or per-packet crossing.

Prefer coarse operations:

```text
send chunk stream
start transfer
query snapshot
receive batched events
```

---

# 152. Minimize Crossing Frequency

Bad:

```text
foreign call for every packet
```

Good:

```text
foreign starts transfer
Rust owns hot path
foreign receives progress snapshots
```

---

# 153. Zero-Copy Priority

Most valuable zero/low-copy cases:

```text
large file input
large file output
media buffers
diagnostic exports
```

Do not overcomplicate tiny message paths.

---

# 154. Media FFI

If external product supplies media frames:

```text
buffer ownership
timestamp
format
lifetime
```

must be explicit.

Prefer buffer pool or frame handle abstraction over raw long-lived pointers.

---

# 155. Media Codec Boundary

Android hardware codecs may remain in Kotlin/platform side while Rust owns call/session logic.

Desktop AV1 software codec may stay fully Rust/native.

FFI should support both without forcing one codec implementation architecture.

---

# 156. Buffer Pool Handle

Advanced API may expose:

```text
comm_buffer_handle_t
```

for reusable media/file buffers.

Add only after profiling shows value.

---

# 157. Main-Thread Safety

Generic C ABI must not assume UI thread.

Bindings handle dispatch.

---

# 158. Foreign GC Runtime

Never rely on foreign GC timing for durable operation completion.

Explicit close/release APIs are primary.

---

# 159. Resource Limits

Part 08 applies to FFI:

```text
max handles
max subscriptions
max pending callbacks
max async ops
```

A foreign app cannot allocate infinite handles.

---

# 160. Diagnostics

Expose developer-only FFI counters:

```text
live handles
callback queue depth
invalid handle calls
```

No raw pointer values.

---

# 161. Crash Recovery

In-process host crash ends runtime process.

Durable state recovers on next start.

Daemon-backed mode can survive host UI/process crash independently.

---

# 162. Security Boundary

FFI is a memory-safety boundary.

Treat all foreign inputs as hostile until validated.

---

# 163. Authorization Boundary

FFI caller having access to a function does not mean it has protocol authority.

Runtime still enforces:

```text
identity roles
priority authorization
peer permissions
policy
```

---

# 164. Public API Example

```c
comm_result_t comm_runtime_create(
    const comm_runtime_config_t* config,
    comm_runtime_handle_t* out_runtime
);

comm_result_t comm_send_message(
    comm_runtime_handle_t runtime,
    const comm_peer_ref_t* peer,
    comm_bytes_view_t text_utf8,
    comm_operation_handle_t* out_operation
);

comm_result_t comm_subscribe(
    comm_runtime_handle_t runtime,
    uint32_t topic,
    comm_event_callback_t callback,
    void* user_data,
    comm_subscription_handle_t* out_subscription
);

comm_result_t comm_subscription_cancel(
    comm_subscription_handle_t subscription
);

comm_result_t comm_handle_release(
    comm_handle_t handle
);
```

---

# 165. Initial Production Scope

Implement first:

```text
ABI versioning
generation-checked opaque handles
stable error codes
UTF-8 byte views
owned output buffers
runtime lifecycle
async operation handles
subscriptions
bounded callback dispatcher
message API
file API
diagnostics API
bootstrap API
panic containment
C/C++ wrapper
Kotlin Android wrapper
Swift wrapper
```

Then:

```text
Python
Dart/Flutter
C#
Go
daemon-backed SDK
generic extension API
```

Defer initially:

```text
every language at once
complex cross-process shared-memory ABI
direct Rust-layout exposure
```

---

# 166. Implementation Phases

## Phase 1 — ABI Foundation

```text
versions
types
error model
handle registry
buffers
```

## Phase 2 — Runtime Lifecycle

```text
create
start
stop
release
```

## Phase 3 — Async Operations

```text
operation handles
status
cancel
completion
```

## Phase 4 — Events

```text
subscription
callback dispatcher
polling
backpressure
```

## Phase 5 — Core APIs

```text
messaging
files
diagnostics
bootstrap
emergency
```

## Phase 6 — Mobile Bindings

```text
Kotlin/JNI
Swift
platform service vtables
```

## Phase 7 — Other Bindings

```text
C++
Python
Dart
C#
Go
Node
```

## Phase 8 — Daemon Backend

```text
same public semantics
IPC-backed implementation
```

## Phase 9 — Hardening

```text
ABI diff
fuzz
ASan
UBSan
TSan
Miri
reentrancy
leak tests
```

---

# 167. Definition of Done

Part 19 is complete when:

- no Rust layout is exposed as stable ABI
- all long-lived objects use validated opaque handles
- stale handles are rejected
- ownership and free rules are explicit
- Rust memory is always freed by Rust
- async work uses operation handles rather than raw Rust Futures
- callbacks are bounded and isolated from core network workers
- callback threading and reentrancy are documented
- no Rust panic unwinds across the C ABI
- stable numeric errors exist
- C, C++, Kotlin, Swift, Python, and Dart can all consume the same platform model
- large file data can avoid unnecessary copies
- daemon-backed and in-process SDK modes share semantics
- Android/iOS objects stay in platform-specific bridges
- private identity keys are not unnecessarily copied across FFI
- ABI golden tests verify sizes, offsets, constants, and symbols
- older compatible bindings work with newer ABI releases
- fuzz, sanitizer, Miri, stale-handle, concurrent-destroy, reentrancy, and leak tests exist

---

# 168. Relationship to Earlier Parts

Part 19 exposes stable integration surfaces for:

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
```

It directly prepares:

```text
20 — Embedded Linux Node
21 — WASM-Compatible Components
22 — Third-Party Protocol Extensions
23 — External Interoperability Suite
24 — Plugin / Module Ecosystem
```

---

# 169. Final Architecture

```text
                      RUST CORE
                         │
                  Stable SDK Model
                         │
                   comm-ffi-core
                         │
                     C ABI v1
          ┌──────────────┼───────────────┐
          │              │               │
         C++           Kotlin           Swift
          │              │               │
       Desktop        Android         iOS/macOS
          │
       Python
          │
       Dart/Flutter
```

Optional daemon-backed mode:

```text
Foreign App
    │
Language Binding
    │
Stable C ABI
    │
FFI Daemon Client
    │
Local Secure IPC
    │
comm-daemon
    │
Communication Runtime
```

---

# 170. Final Principle

The FFI layer should make this possible:

```text
A Flutter application sends files.

An Android Kotlin application uses nearby Bluetooth and NFC.

A Swift application uses the same identity and messaging engine.

A Python automation tool queries transfer state.

A C++ desktop product embeds messaging.

All of them reuse the same Rust communication platform.
```

None of those applications need to know:

```text
how Arc works
how Tokio schedules futures
how Rust enums are laid out
how the internal database is structured
```

They see only:

```text
stable handles
stable functions
stable errors
stable events
stable ownership rules
```

That is the purpose of Part 19: make the Rust communication engine reusable as a language-neutral SDK without sacrificing safety at the boundary.
