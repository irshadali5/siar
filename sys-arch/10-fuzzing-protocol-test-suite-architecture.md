# Part 10 — Fuzzing & Protocol Test Suite Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 10 of 24  
**Primary language:** Rust  
**Primary goals:** protocol safety, parser hardening, deterministic compatibility, state-machine validation, fuzzing, property testing, fault injection, interoperability, regression prevention, release gating

---

# 1. Purpose

The reusable communication platform is network-facing, stateful, cross-platform, multi-version, and security-sensitive.

That means correctness cannot rely only on:

```text
unit tests
happy-path integration tests
manual QA
```

The test architecture must deliberately attack:

- parsers
- serializers
- state machines
- version negotiation
- capability negotiation
- identity transitions
- transfer resumption
- DTN forwarding
- routing decisions
- crash recovery
- resource limits
- malformed input handling
- downgrade attempts
- cross-version compatibility
- non-Rust implementations

The goal is:

> **Every externally influenced state transition must have a test strategy, and every parser must assume hostile input.**

---

# 2. Test Pyramid

Recommended layers:

```text
Static checks
   ↓
Unit tests
   ↓
Property tests
   ↓
Golden vector tests
   ↓
Parser fuzzing
   ↓
State-machine fuzzing
   ↓
Integration tests
   ↓
Multi-peer simulation
   ↓
Crash/fault injection
   ↓
Cross-version compatibility
   ↓
External interoperability
   ↓
Release qualification
```

No single layer is sufficient.

---

# 3. Test Categories

The platform should classify tests explicitly.

```rust
pub enum TestCategory {
    Unit,
    Property,
    Golden,
    Fuzz,
    Integration,
    Simulation,
    FaultInjection,
    Compatibility,
    Interop,
    Performance,
    Security,
}
```

This helps CI and release gating.

---

# 4. Primary Targets

All protocol-visible components must be covered.

At minimum:

```text
core handshake
protocol extension registry
capability negotiation
identity/device certificates
session establishment
messaging frames
file offer/manifest/chunk protocol
DTN bundle/inventory/ACK protocol
routing metadata
daemon IPC
FFI serialization
custom extension boundaries
```

---

# 5. Wire Parser Rule

Every parser must satisfy:

```text
arbitrary bytes
→ valid object
or
→ bounded deterministic error
```

Never:

```text
panic
OOM
infinite loop
unbounded allocation
UB
hang
```

---

# 6. Fuzzing Strategy

Use multiple fuzzing styles:

```text
byte-level fuzzing
structure-aware fuzzing
state-machine fuzzing
sequence fuzzing
mutation fuzzing
differential fuzzing
```

---

# 7. Rust Fuzzing Tools

Recommended Rust tooling:

```text
cargo-fuzz
libFuzzer
proptest
quickcheck where useful
arbitrary
honggfuzz optionally
AFL++ optionally for selected targets
```

Use one primary standard first:

```text
cargo-fuzz + libFuzzer
```

for maintainability.

---

# 8. Fuzz Workspace

Recommended:

```text
fuzz/
├── Cargo.toml
├── fuzz_targets/
│   ├── core_handshake.rs
│   ├── capability_advertisement.rs
│   ├── device_certificate.rs
│   ├── message_frame.rs
│   ├── file_manifest.rs
│   ├── file_offer.rs
│   ├── chunk_header.rs
│   ├── dtn_bundle.rs
│   ├── dtn_inventory.rs
│   ├── ack_summary.rs
│   ├── daemon_ipc.rs
│   └── custom_extension_frame.rs
└── corpus/
```

---

# 9. Parser Fuzz Target Contract

Each parser fuzz target should:

1. take arbitrary bytes
2. apply strict size bound
3. decode
4. validate
5. if valid:
   - re-encode
   - decode again
   - compare semantic equality where appropriate
6. assert no panic

---

# 10. Bounded Allocation

Before fuzzing, parsers must enforce:

```text
max frame bytes
max list count
max string length
max manifest entries
max capability count
max nesting
```

Fuzzing should verify these limits hold.

---

# 11. Postcard Fuzzing

For Postcard-based wire structures:

- fuzz raw decode
- fuzz structurally valid but semantically invalid values
- fuzz length-prefix handling
- fuzz enum discriminants
- fuzz nested vectors
- fuzz truncation
- fuzz trailing bytes
- fuzz version fields

Serialization success does not imply semantic validity.

---

# 12. Golden Wire Vectors

Stable protocols need golden vectors.

Each vector stores:

```text
input semantic object
canonical encoded bytes
expected decode result
expected hash/transcript where relevant
```

Golden vectors prevent accidental wire drift.

---

# 13. Golden Vector Repository

Recommended:

```text
test-vectors/
├── core/
├── identity/
├── capabilities/
├── messaging/
├── files/
├── dtn/
└── security/
```

Each vector should include metadata:

```text
protocol version
schema version
expected result
```

---

# 14. Binary Golden Files

Store binary fixtures:

```text
*.bin
```

plus readable descriptor:

```text
*.ron
```

Example:

```text
message-created-v1.bin
message-created-v1.ron
```

RON describes expected semantic content.

---

# 15. Golden Stability Policy

Once stable protocol is released:

```text
golden bytes cannot change accidentally
```

Intentional change requires:

- protocol review
- version decision
- migration/compatibility update
- regenerated test vectors
- changelog

---

# 16. Round-Trip Properties

For stable codecs:

```text
decode(encode(x)) == x
```

where representation is canonical.

For normalized types:

```text
decode(encode(x)) == normalize(x)
```

---

# 17. Canonical Encoding Property

If protocol hashes serialized data:

```text
same semantic object
→ same canonical bytes
```

This is critical for:

```text
capability transcript hash
device-state hash
bundle identity
```

---

# 18. Unknown Field / Capability Tests

Test:

```text
unknown optional capability
→ ignore safely

unknown required capability
→ deterministic negotiation failure
```

This must be covered by golden and property tests.

---

# 19. State-Machine Testing

Parsers alone are insufficient.

Stateful protocols need explicit state-machine tests.

Examples:

```text
handshake
device linking
file transfer
DTN exchange
capability negotiation
call setup
```

---

# 20. Model-Based Testing

Define a simplified reference model.

Example file transfer model:

```text
Created
Offered
Accepted
Transferring
Paused
Completed
Cancelled
Failed
```

Generate random valid/invalid transitions and compare implementation behavior.

---

# 21. State Transition Fuzzing

Fuzz sequences:

```text
Accept
Pause
Resume
Pause
Cancel
Resume
Complete
```

Implementation must:

```text
accept valid transitions
reject invalid transitions
never panic
```

---

# 22. Protocol Sequence Fuzzing

Instead of arbitrary bytes only, fuzz sequences of valid message types.

Example:

```text
HELLO
CAPS
CAPS
OPEN
CLOSE
DATA
```

This exposes bugs in ordering assumptions.

---

# 23. Handshake State Machine

Test all transitions:

```text
Connected
HelloSent
HelloReceived
IdentityBound
CapabilitiesNegotiated
Active
Closing
Closed
```

Invalid examples:

```text
DATA before identity
OPEN unknown extension
duplicate HELLO with conflicting version
```

---

# 24. Capability Negotiation Model

Part 07 should have a pure reference function:

```rust
fn negotiate_ref(
    local: CapabilitySet,
    remote: CapabilitySet,
    policy: Policy,
) -> ExpectedResult
```

Property test implementation against reference.

---

# 25. Downgrade Tests

Generate scenarios where an attacker strips:

```text
security algorithms
protocol versions
capabilities
```

Expected:

```text
transcript mismatch
minimum policy failure
downgrade warning
```

---

# 26. Identity Fuzzing

Fuzz:

```text
device certificate
device directory
link invite
revocation event
root rotation
identity claim
```

Verify:

```text
invalid signature never accepted
stale generation never becomes current
duplicate event idempotent
```

---

# 27. Identity Chain Property Tests

Invariants:

```text
generation strictly increases
revoked device never returns active through stale event
fork at same generation is detected
root rotation continuity must verify
```

---

# 28. Linking Protocol Tests

Test:

```text
valid QR invite
expired invite
replayed invite
wrong numeric verification
double approval
crash before commit
```

---

# 29. Messaging Protocol Tests

Test:

```text
message create
duplicate delivery
edit before original
reaction duplicate
delete
receipt ordering
```

---

# 30. Message Deduplication Property

Same `MessageId` received N times:

```text
one logical message
```

regardless of transport or DTN path.

---

# 31. File Manifest Fuzzing

Fuzz:

```text
zero chunks
duplicate chunk indexes
overlapping ranges
overflowing offsets
huge declared size
bad hashes
unsorted chunks
truncated manifest
```

All must reject safely.

---

# 32. Chunk Math Property Tests

Verify:

```text
sum(chunk sizes) == total size
ranges non-overlapping
last chunk bounded
offset + size never overflows
```

---

# 33. Resume Bitmap Fuzzing

Fuzz:

```text
bitmaps
range sets
duplicates
large indexes
corrupt compressed bitmap
```

Ensure bounded decode and canonical normalization.

---

# 34. File State-Machine Fuzzing

Generate operations:

```text
offer
accept
write chunk
pause
resume
cancel
finalize
```

Expected invariants:

```text
Completed cannot return to Transferring
Cancelled cannot finalize
verified bytes never decrease except reset/recovery repair
```

---

# 35. Blob Integrity Tests

Mutate one byte in:

```text
chunk
manifest
root
```

Receiver must detect.

---

# 36. DTN Bundle Fuzzing

Fuzz:

```text
bundle headers
expiry
hop limit
replication budget
payload refs
destination tokens
priority
```

---

# 37. DTN Property Tests

Invariants:

```text
hop count never increases
copies never exceed origin budget
expired bundle never forwarded
same BundleId stored once
delivered tombstone blocks immediate resurrection
```

---

# 38. DTN Encounter Sequence Tests

Random sequence:

```text
A meets B
B meets C
A meets C
gateway appears
ACK returns
```

Check:

```text
bounded copies
no loops
eventual delivery when path exists
```

---

# 39. Routing Property Tests

Part 03 invariants:

```text
revoked peer never selected
forbidden transport never selected
realtime never uses DTN
metered-disabled bulk never uses metered path
```

---

# 40. Routing Differential Tests

Compare optimized router against simple reference scorer.

For same inputs:

```text
selected plan should match reference policy
```

---

# 41. Resource Limit Tests

Part 08 invariants:

```text
usage never exceeds hard budget
critical reserve cannot be consumed by bulk
cancelled permits release
per-peer quota <= global quota
```

---

# 42. Queue Saturation Tests

Produce:

```text
1,000,000 operations
```

into bounded system.

Assert:

```text
bounded memory
backpressure
correct defer/drop semantics
```

---

# 43. Crash-Recovery Tests

Part 09 must be tested with real process termination.

Examples:

```text
commit then SIGKILL
network send then SIGKILL
chunk write then SIGKILL
rename then SIGKILL
revocation then SIGKILL
```

---

# 44. Failpoint Architecture

Internal failpoints:

```text
after_event_commit
before_outbox_insert
after_outbox_insert
before_chunk_ack
after_blob_rename
before_delivery_marker
```

Tests can force crash at each location.

---

# 45. Process Kill Harness

Build helper:

```text
spawn child
perform scripted action
kill child
restart
query state
```

Do not rely only on exceptions/panics.

---

# 46. Network Fault Simulation

Simulate:

```text
packet loss
duplication
reordering
latency
bandwidth limits
disconnect
half-open connection
transport switch
```

---

# 47. Simulated Transport

Use deterministic transport:

```rust
pub struct SimulatedTransport {
    // scripted network conditions
}
```

It should implement same transport abstraction as Iroh/LAN adapters.

---

# 48. Deterministic Seed

Every randomized test logs:

```text
seed
```

so failures reproduce exactly.

---

# 49. Multi-Peer Simulator

Topology:

```text
A
├── B
├── C
└── D
```

with configurable links.

Support:

```text
online/offline
encounter windows
gateway state
battery
storage
latency
bandwidth
```

---

# 50. Simulation Scenarios

Required:

```text
normal Internet
LAN-only
Bluetooth-only
DTN partition
gateway bridge
multi-device account
storage pressure
battery saver
mass peer appearance
```

---

# 51. Disaster Scenario Suite

Example:

```text
20 peers
no Internet
BLE encounters
2 Wi-Fi gateways appear intermittently
critical SOS + normal chat + file traffic
```

Validate:

```text
critical traffic survives
bulk yields
bounded replication
```

---

# 52. Interop Test Harness

External implementations should be able to run:

```text
server mode
client mode
scripted protocol vectors
```

against Rust reference.

---

# 53. Interop Modes

```text
Rust ↔ Rust old version
Rust ↔ Rust new version
Rust ↔ C/FFI sample
Rust ↔ external implementation
```

---

# 54. Compatibility Matrix

Maintain:

```text
N
N-1
N-2
```

where product policy requires.

At minimum test current against previous supported major/minor versions.

---

# 55. Wire Compatibility Tests

For each stable protocol:

```text
new decoder reads old vector
old decoder handles new optional capability safely
```

where promised.

---

# 56. Protocol Major Tests

Different majors should:

```text
negotiate common older version
or fail cleanly
```

Never accidentally cross-decode incompatible frames.

---

# 57. Storage Compatibility Tests

Separate from wire compatibility.

Test:

```text
old database
→ new binary
→ migrate
→ replay
```

---

# 58. Downgrade Storage Tests

If old binary opens new DB:

```text
safe read
or explicit refusal
```

Never silent corruption.

---

# 59. Cross-Platform Tests

Required targets:

```text
Linux
Windows
Android
macOS where available
iOS when supported
```

Not every fuzz job must run everywhere.

But protocol golden vectors must be platform-independent.

---

# 60. Endianness / Architecture Tests

Wire format must not depend on:

```text
native endian
pointer width
usize
```

Run tests on:

```text
x86_64
aarch64
```

at minimum.

---

# 61. Miri

Use Miri selectively for unsafe or tricky memory code.

Targets:

```text
FFI
buffer management
zero-copy helpers
custom parsing
```

Do not run full app under Miri if impractical.

---

# 62. Sanitizers

Use:

```text
AddressSanitizer
UndefinedBehaviorSanitizer
ThreadSanitizer
```

where supported.

Rust reduces many classes but unsafe/FFI/native codecs still need them.

---

# 63. Loom

For tricky concurrency primitives, consider:

```text
loom
```

for:

```text
state machines
permit release
lock ordering
concurrent caches
```

---

# 64. Deadlock Tests

Stress:

```text
resource permits
multi-lock ordering
shutdown/recovery
```

with timeouts.

---

# 65. Property-Based Test Data

Use generators for:

```text
valid IDs
invalid IDs
version ranges
capability sets
file manifests
bundle graphs
device event chains
```

Avoid only hand-written cases.

---

# 66. Shrinking

Property framework should minimize failing cases.

This is extremely useful for complex capability/routing bugs.

---

# 67. Metamorphic Testing

Useful properties:

```text
adding unsupported optional capability does not change result
reordering canonical input does not change semantic result
duplicating idempotent frame does not alter final state
```

---

# 68. Differential Serialization Testing

Where multiple codecs/implementations exist:

```text
Rust encoder
external decoder
external encoder
Rust decoder
```

Compare semantic result.

---

# 69. Security Regression Corpus

When a bug is found:

```text
add exact triggering input to regression corpus
```

Never rely on fuzz engine rediscovering it.

---

# 70. Corpus Management

Keep:

```text
minimal
deduplicated
version-labeled
```

corpora.

Avoid giant random corpus committed to repository.

---

# 71. Fuzz Seed Corpus

Seed with:

```text
valid smallest frame
valid largest-normal frame
boundary values
old protocol vectors
```

This improves exploration.

---

# 72. Dictionary Files

Provide protocol token dictionaries where useful for fuzzers.

Especially for:

```text
frame type bytes
magic values
version markers
```

---

# 73. Stateful Fuzz Harness

Example:

```rust
enum Action {
    Connect,
    SendHello,
    SendCaps,
    OpenExtension,
    SendData,
    Close,
}
```

Fuzzer generates action sequence.

Implementation must preserve invariants.

---

# 74. Invariant Assertions

Add internal assertions in debug/test builds:

```text
stream_version monotonic
resource usage nonnegative
transfer completed implies full verification
revoked device not active
```

These help fuzzing detect bugs earlier.

---

# 75. No Panics on Remote Input

Remote-input path should return typed errors.

Fuzz test enforces:

```text
panic = bug
```

unless clearly impossible internal invariant after prior validation.

---

# 76. Timeout Testing

Every parser/state-machine test should have bounded execution.

Detect:

```text
infinite loop
algorithmic complexity attack
```

---

# 77. Complexity Attack Tests

Construct adversarial inputs:

```text
many duplicate capabilities
huge near-valid manifest
deep nested structures
pathological set reconciliation
```

Measure time.

---

# 78. Allocation Attack Tests

Measure peak memory when parsing near-limit payloads.

Assert:

```text
bounded overhead
```

not 100× input size.

---

# 79. CPU DoS Tests

Examples:

```text
signature verification flood
hash mismatch flood
invalid capability dependency graph
```

Ensure rate limiting/admission works.

---

# 80. Protocol Limits Golden Tests

For each limit:

```text
max accepted
max+1 rejected
```

Examples:

```text
frame bytes
manifest chunks
capability count
bundle size
message attachments
```

---

# 81. FFI Tests

Part 19 should test:

```text
invalid handle
double free attempt
null pointer
oversized buffer
callback after shutdown
panic boundary
```

---

# 82. Daemon IPC Tests

Part 16 should fuzz:

```text
IPC frame
subscription messages
snapshot requests
malformed client
slow client
```

---

# 83. Plugin/Extension Boundary Tests

Part 24 should test:

```text
plugin registers duplicate capability
plugin exceeds queue
plugin emits malformed frame
plugin panics
plugin fails recovery
```

Core must remain stable.

---

# 84. Test Double Architecture

Provide:

```text
FakeClock
FakeRandom
FakeTransport
FakeStorage
FakeSecureStore
FakePlatform
```

This makes deterministic tests possible.

---

# 85. Time Control

Do not use real sleeps in most tests.

Use:

```text
virtual clock
manual advance
```

for:

```text
expiry
retry
backoff
DTN lifetime
```

---

# 86. Randomness Injection

Cryptographic code uses secure RNG in production.

Tests can inject deterministic RNG only through test-safe abstractions.

Never accidentally allow deterministic RNG in production identity generation.

---

# 87. Clock Injection

Use trait:

```rust
pub trait Clock {
    fn now(&self) -> Timestamp;
}
```

with:

```text
SystemClock
TestClock
```

---

# 88. Network Simulation Time

Simulator should use virtual time.

This makes multi-day DTN simulations fast.

---

# 89. Scenario DSL

Consider RON scenario files:

```ron
(
    peers: 4,
    events: [
        Connect("A","B"),
        SendMessage("A","D"),
        Disconnect("A","B"),
        AdvanceSeconds(60),
        Connect("B","C"),
    ],
)
```

This aligns with Rust/RON preference and produces readable fixtures.

---

# 90. Scenario Runner

Runner loads:

```text
*.ron
```

and executes deterministic multi-peer tests.

---

# 91. Example Scenario Categories

```text
messaging/
files/
dtn/
identity/
routing/
resource/
crash/
```

---

# 92. Security Test Suite

Dedicated security suite should include:

```text
replay
downgrade
oversized payload
unauthorized state change
signature tampering
revoked peer
storage exhaustion
priority abuse
```

---

# 93. Threat-Model Traceability

Each threat from security docs should map to test IDs.

Example:

```text
THREAT-DTN-004 → TEST-DTN-SPAM-002
```

This improves auditability.

---

# 94. Test IDs

Use stable IDs for major protocol/security tests.

Example:

```text
CAP-NEG-001
FILE-MANIFEST-007
DTN-REPLAY-003
CRASH-OUTBOX-004
```

---

# 95. Coverage

Track:

```text
line
branch
protocol state transition
error code
wire frame type
```

Line coverage alone is insufficient.

---

# 96. Protocol Coverage Matrix

Maintain table:

```text
Frame Type | Decode | Invalid | Fuzz | Golden | Interop
```

Every stable frame type must be covered.

---

# 97. State Transition Coverage

For each state machine:

```text
valid transition tested?
invalid transition tested?
crash around transition tested?
```

---

# 98. Error-Code Coverage

Every stable wire error code should have at least one test.

---

# 99. Mutation Testing

Use mutation testing selectively.

Good targets:

```text
capability negotiation
authorization
routing constraints
state machines
```

If changing `>` to `<` does not fail tests, coverage is weak.

---

# 100. Performance Regression Tests

Benchmarks:

```text
handshake
capability negotiation
manifest decode
chunk verify
bundle inventory reconcile
route planning
```

Track regressions.

---

# 101. Latency Budgets

Set practical thresholds for local operations.

Example:

```text
capability negotiation compute < X ms
manifest validation linear
route selection bounded
```

Avoid overfitting exact microbenchmarks across CI hardware.

---

# 102. Memory Regression Tests

Track peak allocations for:

```text
large manifest
large capability set
transfer buffer pool
DTN inventory
```

---

# 103. Binary Size Tests

Feature modularity should be tested.

Examples:

```text
file-only binary does not pull Dioxus/media
headless node excludes UI
```

CI can inspect dependency tree/binary size trends.

---

# 104. Feature Matrix CI

Build combinations:

```text
default
messaging
files
dtn
messaging+files
headless
android
desktop
full
```

Detect accidental dependency coupling.

---

# 105. Minimal Feature Build

Critical acceptance:

```text
cargo check --no-default-features --features files
```

works without messaging/UI.

---

# 106. Cross-Compilation Checks

CI should compile:

```text
Linux x86_64
Linux aarch64
Windows
Android targets
```

macOS/iOS require Apple environment for final native builds/tests.

Do not claim Linux cross-compilation can fully produce signed/tested Apple releases.

---

# 107. Android Instrumentation

Android-specific tests:

```text
MediaCodec capability
Keystore
Bluetooth permission
background process kill
file URI access
```

Run on emulator + at least some real devices before production release.

---

# 108. Device Matrix

Important Android variation:

```text
different vendors
Android versions
hardware codec availability
Bluetooth stacks
```

Protocol core stays deterministic, platform adapters need real-device coverage.

---

# 109. Windows Tests

Test:

```text
filesystem rename semantics
path handling
socket behavior
service/daemon behavior
```

---

# 110. Linux Tests

Test:

```text
Wayland/Dioxus unaffected by headless core
filesystem permissions
daemon/systemd
Bluetooth stack
```

---

# 111. macOS/iOS Tests

Final native behavior needs Apple CI/hardware.

Protocol golden vectors can still be generated/validated elsewhere.

---

# 112. CI Layers

Recommended:

## Fast PR CI

```text
fmt
clippy
unit
property subset
golden
feature build matrix
```

## Standard PR CI

```text
integration
simulation
sanitizer selected
fuzz smoke
```

## Nightly

```text
long fuzz
large simulations
crash injection
performance
cross-version
```

## Release

```text
full compatibility
interop
platform tests
security regression
long fuzz corpus
```

---

# 113. Fuzz Smoke Test

Every PR runs short fuzz session:

```text
10–60 seconds per critical target
```

This catches obvious regressions.

Long campaigns run nightly/continuous.

---

# 114. Continuous Fuzzing

Ideal:

```text
OSS-Fuzz or self-hosted fuzz workers
```

for public project.

If not possible initially:

```text
dedicated scheduled CI fuzz jobs
```

---

# 115. Fuzz Artifact Handling

When crash found:

```text
save minimized input
attach stack trace
record commit
create regression test
```

---

# 116. Automatic Minimization

Use:

```text
cargo fuzz tmin
```

or equivalent to reduce reproducer.

---

# 117. Security Bug Workflow

```text
fuzz finding
 ↓
triage
 ↓
security severity
 ↓
fix
 ↓
regression corpus
 ↓
advisory if released
```

---

# 118. Secret Handling in Test Logs

Never log:

```text
real private keys
production tokens
user files
```

Use generated test identities.

---

# 119. Deterministic Test Keys

Test fixtures may use known keys clearly marked:

```text
TEST ONLY
```

Never compile into production trust roots.

---

# 120. Cryptographic Known-Answer Tests

For crypto primitives/protocol wrappers:

```text
known input
known output
```

Use official/reference vectors where applicable.

Do not invent cryptographic correctness solely from round-trip tests.

---

# 121. Crypto Differential Tests

Where possible compare against:

```text
upstream crate/reference implementation
```

for key derivation/signature/AEAD primitives.

---

# 122. Nonce-Reuse Tests

Test that chunk/session nonce derivation cannot repeat under:

```text
same blob
different chunk
retry
resume
```

---

# 123. Identity Security Tests

Test:

```text
revoked device handshake
stale directory
forged certificate
root change
replayed link invite
```

---

# 124. Authorization Tests

Identity valid but operation unauthorized:

```text
must reject
```

Authentication and authorization tests must be separate.

---

# 125. Resource Abuse Tests

Malicious peer:

```text
many small valid frames
many signature checks
many file offers
many DTN bundles
```

Assert:

```text
bounded resource usage
```

---

# 126. Slowloris-Like Tests

Peer sends headers/payload extremely slowly.

Connection/session layer must enforce:

```text
timeouts
minimum progress
resource slot limits
```

---

# 127. Partial Frame Tests

Every frame parser:

```text
0 bytes
1 byte
header-1
exact header
payload-1
exact payload
payload+trailing
```

---

# 128. Truncation Corpus

Automatically generate all prefix truncations of golden frames.

Each must:

```text
return incomplete/error
never panic
```

---

# 129. Bit-Flip Corpus

For each golden vector, mutate bits at strategic positions.

Useful for parser robustness.

---

# 130. Length-Field Attacks

Fuzz lengths:

```text
0
1
max
max+1
u32::MAX
u64::MAX
```

Ensure no overflow.

---

# 131. Integer Overflow Tests

Targets:

```text
offset + size
chunk_count * chunk_size
expiry arithmetic
replication counters
queue counters
```

---

# 132. Unicode Tests

For human-readable metadata:

```text
invalid UTF-8
combining marks
very long grapheme clusters
RTL
control chars
```

Protocol security identity should never depend on display strings.

---

# 133. Filename Tests

Part 05:

```text
../
absolute path
reserved Windows names
NUL-like invalid chars
very long names
Unicode normalization
```

Storage path must remain safe.

---

# 134. RON Config Fuzzing

If RON config/scenario files are parsed:

```text
malformed
deep nesting
huge lists
unknown fields
```

Use bounded config parsing where possible.

---

# 135. Database Migration Tests

For each schema release:

```text
fixture old DB
→ migrate
→ verify
```

Also:

```text
kill during migration
→ restart
```

---

# 136. Projection Rebuild Tests

Part 04:

```text
live projection
```

must equal:

```text
fresh rebuild from events
```

for deterministic histories.

---

# 137. Recovery Matrix

Crash points across domains:

```text
message
file
DTN
identity
projection
storage GC
```

Maintain matrix rather than ad hoc tests.

---

# 138. Compatibility Snapshot

For each release, preserve:

```text
wire vectors
DB fixture
capability set
identity fixture
```

This supports future regression testing.

---

# 139. Test Artifact Versioning

Store fixtures under:

```text
v1/
v2/
```

Do not overwrite old compatibility fixtures.

---

# 140. Protocol Conformance Runner

Build CLI:

```text
comm-conformance
```

Commands:

```text
validate-frame
run-vectors
serve-peer
connect-peer
run-suite
```

This becomes foundation for Part 23 external interoperability.

---

# 141. Example Conformance Command

```bash
comm-conformance run-suite --protocol files/1
```

Output:

```text
PASS FILE-001
PASS FILE-002
FAIL FILE-007
```

---

# 142. Machine-Readable Results

Support:

```text
JSON/JUnit
```

for CI interoperability even if core project prefers RON/Postcard internally.

External tooling compatibility justifies JSON here.

---

# 143. Test Protocol Peer

A scripted peer can intentionally:

```text
send malformed order
delay ACK
duplicate frame
downgrade capability
```

Useful against real app binaries.

---

# 144. Black-Box Tests

Not all tests should link internal Rust crates.

Run actual binaries and test over network/IPC.

This catches integration mistakes.

---

# 145. Release Candidate Soak Tests

Run long-duration:

```text
24h+
```

simulations/real nodes for:

```text
memory leaks
queue drift
DTN store growth
reconnect storms
```

---

# 146. Memory Leak Tests

Track runtime memory over repeated:

```text
connect/disconnect
send/cancel
transfer/resume
```

No monotonic leak beyond caches.

---

# 147. File Descriptor Leak Tests

Repeated sessions/transfers must not exhaust FDs.

---

# 148. Task Leak Tests

Supervisor should return active task count to baseline after operations.

---

# 149. Long DTN Simulation

Simulate virtual days/weeks.

Check:

```text
expiry
tombstone cleanup
bounded storage
no route loops
```

---

# 150. Multi-Version Soak

Run:

```text
old peer
new peer
mixed network
```

for long sessions.

---

# 151. Performance vs Correctness

A performance optimization is not accepted unless:

```text
all golden/property/fuzz tests remain valid
```

---

# 152. Benchmark Baselines

Store trend, not absolute universal pass/fail on noisy CI.

Hard fail only for severe regressions.

---

# 153. CI Reproducibility

Pin:

```text
Rust toolchain
test seeds where deterministic
dependency lockfile
```

Use clean environments.

---

# 154. Hermetic Test Data

Tests must not depend on:

```text
developer home directory
random Internet service
local Bluetooth device
```

unless explicitly marked integration/hardware test.

---

# 155. Networkless Core Test Suite

Most protocol tests should run with:

```text
no Internet
```

This is important for local-first architecture.

---

# 156. Hardware Test Layer

Separate tags:

```text
requires-bluetooth
requires-wifi-aware
requires-android-hw-codec
```

CI schedules them only on capable runners.

---

# 157. Test Flakiness Policy

Flaky test is a bug.

Do not repeatedly rerun until green without investigation.

Use deterministic simulation to eliminate timing races.

---

# 158. Async Test Timeouts

Every async integration test has explicit timeout.

Hung test should fail clearly.

---

# 159. Retry in Tests

Only retry when testing retry behavior.

Do not hide nondeterminism with generic retries.

---

# 160. Logging on Failure

Capture:

```text
seed
peer states
protocol transcript
resource snapshot
event-log offsets
```

without secrets.

---

# 161. Transcript Recording

Test builds can record redacted protocol transcript:

```text
frame type
length
sequence
state
```

for debugging.

---

# 162. Differential State Snapshot

When simulator fails, dump:

```text
expected model
actual runtime
```

side-by-side.

---

# 163. Formal-Like Invariants

Maintain explicit invariants in docs/tests.

Examples:

```text
No revoked device is authenticated.
No completed blob has unverified chunks.
No DTN bundle forwards after expiry.
No resource usage exceeds hard budget.
```

---

# 164. Test Traceability

Architecture requirement:

```text
REQ-FILE-RESUME-01
```

maps to:

```text
TEST-FILE-RESUME-12
```

Useful as project grows.

---

# 165. Security Release Gate

Release blocked if:

```text
known parser panic
known auth bypass
known downgrade bypass
known unbounded allocation
known crash-consistency violation
```

---

# 166. Compatibility Release Gate

Release blocked if promised supported versions fail conformance.

---

# 167. Fuzz Release Gate

Before stable release:

```text
critical fuzz targets run extended campaign
no unresolved crashers
```

Exact CPU-hours depend on resources, but must be explicit.

---

# 168. Test Suite Health Dashboard

Track:

```text
unit pass rate
fuzz crash count
coverage
compat matrix
platform matrix
flaky tests
performance trend
```

Part 18 diagnostics may expose local runtime, while CI dashboard is separate.

---

# 169. Suggested Test Workspace

```text
tests/
├── unit/
├── property/
├── golden/
├── integration/
├── simulation/
├── crash/
├── compatibility/
├── interop/
└── security/

fuzz/
├── fuzz_targets/
└── corpus/

test-vectors/
├── core/
├── identity/
├── capabilities/
├── messaging/
├── files/
└── dtn/

tools/
└── comm-conformance/
```

---

# 170. Test Utility Crate

```text
comm-testkit/
```

can provide:

```text
FakeClock
FakeTransport
FakeStorage
TestIdentity
ScenarioRunner
Failpoints
ProtocolPeer
```

Production crates must not depend on it.

---

# 171. `comm-testkit` Structure

```text
crates/comm-testkit/
├── src/
│   ├── clock.rs
│   ├── transport.rs
│   ├── storage.rs
│   ├── identity.rs
│   ├── peer.rs
│   ├── scenario.rs
│   ├── failpoint.rs
│   └── assertions.rs
```

---

# 172. Test-Only Feature Flags

Use carefully:

```text
test-hooks
failpoints
deterministic-rng
```

Ensure they cannot accidentally enable in release builds.

---

# 173. Build Guard

CI can fail if release binary includes:

```text
test-hooks
```

---

# 174. Protocol Vector Generation

Provide explicit tool:

```text
comm-vectors generate
```

but generated stable vectors require review before commit.

---

# 175. Vector Verification

CI runs:

```text
comm-vectors verify
```

to ensure repository fixtures match current stable protocol expectations.

---

# 176. Compatibility Policy

Document support window.

Example:

```text
Current stable major
Previous stable major if explicitly supported
```

Do not promise indefinite backward compatibility accidentally.

---

# 177. Unknown Future Peer Tests

Current implementation should tolerate:

```text
new optional capabilities
new optional extensions
```

according to protocol rules.

Simulate future-peer advertisements.

---

# 178. Old Peer Tests

New implementation must not send unsupported frame types after negotiated downgrade.

---

# 179. Feature Disable Tests

Compile/run with:

```text
files disabled
DTN disabled
calls disabled
```

Peer negotiation must adapt.

---

# 180. Partial Capability Tests

Example:

```text
resume yes
parallel no
```

File transfer must select valid fallback.

---

# 181. Media Negotiation Tests

Matrix:

```text
AV1 SW ↔ AV1 HW
AV1 ↔ H264
H265 ↔ H264
decode-only ↔ encode-only
```

Ensure directional capability logic.

---

# 182. Platform Capability Tests

Mock:

```text
Bluetooth unavailable
Wi-Fi Aware unavailable
background denied
```

Routing/DTN must degrade correctly.

---

# 183. Resource Profile Tests

Run same scenario under:

```text
Embedded
MobileLow
Desktop
Server
```

Check different concurrency without correctness changes.

---

# 184. Headless Tests

Run communication node:

```text
no Dioxus
no UI
```

and execute messaging/files/DTN protocol suites.

This proves headless reusability.

---

# 185. File-Only Tests

Build/run file app without messaging.

Must pass file/identity/routing tests.

---

# 186. Messaging-Only Tests

Build/run messaging without file subsystem.

Attachments capability should negotiate unavailable cleanly.

---

# 187. Emergency Profile Tests

Enable:

```text
critical reserve
DTN
nearby
```

Validate priority and replication controls.

---

# 188. Security Fuzz Corpus Retention

Security-relevant regressions remain indefinitely unless protocol removed.

---

# 189. Crash Corpus

Each discovered crash-consistency bug becomes scripted failpoint regression.

---

# 190. Protocol Corpus

Each parser vulnerability becomes minimized binary fixture.

---

# 191. Review Requirements

New wire message type cannot merge without:

```text
decoder test
invalid-input test
golden vector
fuzz target coverage
state-machine test if stateful
compatibility documentation
```

---

# 192. New Capability Requirements

New capability must include:

```text
ID
version semantics
required/optional behavior
parameter bounds
negotiation tests
unknown-peer tests
```

---

# 193. New Event Schema Requirements

New durable event must include:

```text
versioned codec
golden vector
replay test
migration/upcast policy
```

---

# 194. New Resource Type Requirements

New queue/buffer must include:

```text
capacity
overflow behavior
pressure behavior
saturation test
```

---

# 195. New Background Worker Requirements

Must include:

```text
supervision
cancellation
crash behavior
resource budget
test
```

---

# 196. Test Failure Triage

Classify:

```text
Product bug
Protocol bug
Security bug
Test bug
Flaky environment
Performance regression
```

Security/protocol bugs receive highest priority.

---

# 197. Reproducer Format

Every serious failure should have:

```text
seed
binary input/scenario
commit
platform
expected
actual
```

---

# 198. External Security Testing

Before broad production release:

```text
third-party security review
penetration testing
protocol audit
```

especially for:

```text
identity
crypto usage
file parsing
DTN exposure
```

Fuzzing does not replace cryptographic/security review.

---

# 199. Bug Bounty Readiness

If project becomes public and mature:

```text
SECURITY.md
private disclosure channel
supported versions
response process
```

consider bug bounty later.

---

# 200. Protocol Conformance Documentation

Publish:

```text
wire schemas
state machines
error codes
test vectors
capabilities
limits
```

This improves external implementation quality.

---

# 201. Release Test Checklist

Before release candidate:

- all required CI green
- no unresolved critical fuzz crash
- golden vectors stable
- supported version matrix green
- crash tests green
- resource abuse tests green
- major platform smoke tests green
- file resume tests green
- DTN partition tests green
- identity revocation tests green

---

# 202. Stable Release Gate

Stable release must not proceed if:

```text
known data-loss crash bug
known protocol panic
known auth bypass
known unbounded remote allocation
known incompatible wire regression
```

---

# 203. Initial Production Scope

Implement first:

```text
cargo-fuzz harness
golden vectors
proptest generators
state-machine tests
simulated transport
virtual clock
failpoint system
process-kill recovery tests
compatibility fixtures
conformance CLI skeleton
CI fuzz smoke
nightly fuzz
```

Defer initially:

```text
full OSS-Fuzz integration
large external implementation farm
formal verification
advanced mutation-testing infrastructure
```

---

# 204. Implementation Phases

## Phase 1 — Test Foundations

```text
comm-testkit
FakeClock
FakeTransport
deterministic scenario runner
```

## Phase 2 — Golden Vectors

```text
core
identity
capability
messaging
files
DTN
```

## Phase 3 — Property Tests

```text
state invariants
round-trip
resource bounds
routing constraints
```

## Phase 4 — Fuzzing

```text
parsers
manifests
bundles
capabilities
identity
IPC
```

## Phase 5 — Stateful Fuzzing

```text
handshake
linking
file transfer
DTN encounters
```

## Phase 6 — Crash/Fault Tests

```text
failpoints
SIGKILL
disk full
network duplication
```

## Phase 7 — Compatibility/Interop

```text
old/new versions
conformance tool
external vectors
```

## Phase 8 — Release Gating

```text
CI matrices
nightly
soak
security regression
```

---

# 205. Definition of Done

Part 10 is complete when:

- every network parser has a fuzz target or documented equivalent
- malformed input cannot panic or allocate unbounded memory
- stable wire formats have golden vectors
- protocol hashes use canonical encoding verified by tests
- every state machine has valid/invalid transition tests
- capability negotiation has downgrade/unknown-feature tests
- identity revocation/root continuity has property tests
- file manifests/resume state are fuzzed
- DTN replication/hop/expiry invariants are property tested
- routing hard constraints are property tested
- resource saturation stays bounded
- process-kill crash tests exist
- simulated network supports loss/reordering/partition
- multi-peer disaster scenarios run deterministically
- previous supported versions are tested
- headless/file-only/messaging-only feature builds are tested
- conformance tooling exists
- security regression corpus is retained
- release gates block known protocol/security/data-loss failures

---

# 206. Relationship to Earlier Parts

Part 10 verifies:

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
```

It becomes a foundation for validating:

```text
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

---

# 207. Final Principle

A protocol is not production-ready merely because:

```text
two current clients can talk to each other
```

It is production-ready when:

```text
old and new clients negotiate correctly
malformed peers cannot crash it
duplicate traffic is harmless
state machines reject invalid order
crashes preserve durable truth
resource floods remain bounded
protocol changes are caught by golden vectors
external implementations can verify conformance
```

The fuzzing and protocol test architecture therefore becomes a permanent engineering system, not a one-time QA phase.

Every new protocol feature must enter this test ecosystem before it is considered stable.
