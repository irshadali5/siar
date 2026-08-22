# Part 16 — Daemon & Headless Runtime Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 16 of 24  
**Primary language:** Rust  
**Primary goals:** long-lived communication runtime, daemon ownership of networking/storage/identity, headless deployment, GUI/CLI separation, secure IPC, background execution, crash recovery, service supervision, embedded/server reuse, multi-user isolation, remote administration, and platform-efficient operation

---

# 1. Purpose

The communication platform should not require the user interface to remain alive for core networking to function.

Messaging, file transfers, DTN forwarding, relay participation, discovery, background synchronization, and recovery often need a long-lived runtime that can continue even when:

```text
the Dioxus window is closed
the CLI exits
the desktop session restarts
the Android UI activity is recreated
a headless server has no GUI
an embedded gateway has no display
```

Therefore the platform should support a dedicated daemon/headless mode.

The core rule is:

> **The daemon owns durable communication state and long-lived networking; UI and CLI are clients of that runtime.**

---

# 2. Architectural Position

```text
                ┌───────────────────────┐
                │   Dioxus Desktop UI   │
                └──────────┬────────────┘
                           │
                     Secure Local IPC
                           │
                ┌──────────▼────────────┐
                │  comm-daemon/runtime  │
                ├───────────────────────┤
                │ Identity              │
                │ Event Log             │
                │ Messaging             │
                │ Files                 │
                │ DTN                   │
                │ Routing               │
                │ Proximity             │
                │ Capability            │
                │ Multipath             │
                │ Power policy          │
                │ Recovery              │
                └──────────┬────────────┘
                           │
                  Iroh / LAN / BLE / Wi-Fi
```

CLI and headless control use the same daemon.

---

# 3. Runtime Modes

Support multiple deployment modes.

```rust
pub enum RuntimeMode {
    InProcess,
    LocalDaemon,
    HeadlessNode,
    MobileService,
}
```

---

# 4. In-Process Mode

Useful for:

```text
simple mobile builds
tests
embedded usage
single-binary tools
```

Architecture:

```text
Dioxus/Application
       ↓
CommunicationRuntime
```

No IPC boundary required.

---

# 5. Local Daemon Mode

Desktop architecture:

```text
comm-daemon
   ↑
Unix socket / named pipe / local IPC
   ↑
Dioxus UI
CLI
other trusted local clients
```

This should be the preferred production desktop architecture if background communication is a key feature.

---

# 6. Headless Node Mode

Used for:

```text
relay/gateway
Raspberry Pi
community emergency node
server
NAS
home-lab
enterprise appliance
```

No GUI dependency.

---

# 7. Mobile Service Mode

Android/iOS have platform-specific background constraints.

Logical ownership remains:

```text
Rust CommunicationRuntime
```

but execution may be hosted by:

```text
foreground service
background task
app process
system-approved execution window
```

The platform adapter controls lifecycle; Rust owns durable state.

---

# 8. Daemon Responsibilities

The daemon owns:

```text
identity keys/access
device state
event log
outbox
message state
blob store
file-transfer state
DTN bundle store
network listeners
routing
proximity discovery
capability negotiation
background scheduling
crash recovery
resource budgets
```

---

# 9. UI Responsibilities

Dioxus owns:

```text
rendering
navigation
view state
user input
dialogs
local presentation preferences
```

It should not directly open protocol transports or mutate authoritative DB state.

---

# 10. CLI Responsibilities

CLI should:

```text
query daemon
issue explicit commands
inspect status
perform admin actions
```

Examples:

```text
comm status
comm peers
comm send
comm transfer list
comm doctor
comm dtn status
```

---

# 11. Sole Writer Principle

In daemon mode:

> **The daemon is the sole writer of communication databases and durable state.**

This avoids:

```text
database lock races
projection inconsistency
duplicate outbox workers
identity-state divergence
```

---

# 12. Read Access

UI may read via:

```text
IPC query API
```

rather than opening the database itself.

For very large local read models, shared/read-only optimization can be considered later, but IPC is safer initially.

---

# 13. IPC Architectural Goals

IPC must be:

```text
local
authenticated
versioned
bounded
streaming-capable
backpressure-aware
crash-tolerant
```

---

# 14. IPC Transport

Desktop options:

```text
Unix domain socket
Windows named pipe
localhost QUIC/TCP only if necessary
```

Preferred:

```text
Unix socket on Linux/macOS
named pipe on Windows
```

Do not expose a TCP port to LAN by default.

---

# 15. IPC Protocol

Use stable versioned protocol.

```text
comm-ipc/1
```

Small control frames can use:

```text
Postcard
```

Large binary blobs should use:

```text
streamed data
or file handles/references
```

not giant serialized messages.

---

# 16. IPC Handshake

```text
Connect
 ↓
ProtocolVersion
 ↓
ClientAuthentication
 ↓
CapabilityNegotiation
 ↓
SessionReady
```

Even local IPC should not assume every process is trusted.

---

# 17. Local Client Authentication

Possible mechanisms:

```text
filesystem socket permissions
OS user identity
shared local capability token
peer credentials
```

Use multiple layers where practical.

---

# 18. IPC Session Identity

```rust
pub struct IpcClientId([u8; 16]);
```

Each connected local client gets a runtime identity.

---

# 19. Client Roles

```rust
pub enum IpcClientRole {
    Ui,
    Cli,
    Admin,
    Automation,
    PluginHost,
}
```

Role limits available commands.

---

# 20. IPC Authorization

Examples:

```text
UI:
query + user actions

CLI:
query + explicit commands

Admin:
maintenance + diagnostics

Plugin:
restricted extension API
```

---

# 21. IPC Command Model

```rust
pub enum DaemonCommand {
    SendMessage(...),
    SendFile(...),
    PauseTransfer(...),
    ResumeTransfer(...),
    PairDevice(...),
    SetPowerPolicy(...),
    Shutdown,
}
```

Actual wire types should be versioned structs.

---

# 22. IPC Query Model

```rust
pub enum DaemonQuery {
    RuntimeStatus,
    ConversationList,
    MessagePage,
    TransferList,
    NearbyPeers,
    DtnStatus,
    DeviceDirectory,
    Diagnostics,
}
```

---

# 23. IPC Event Model

Daemon publishes:

```text
message updates
transfer progress
peer presence
nearby observations
runtime health
power-policy changes
DTN status
```

Clients subscribe.

---

# 24. Bounded Event Streams

Part 08 applies.

Each IPC client has:

```text
bounded outgoing queue
```

If UI is too slow:

```text
coalesce
drop stale ephemeral events
request snapshot/resync
```

---

# 25. Snapshot + Delta Model

Recommended:

```text
initial snapshot
+
incremental deltas
```

If client misses too much:

```text
invalidate subscription
request fresh snapshot
```

---

# 26. UI Reconnect

Daemon may restart.

UI should:

```text
detect disconnect
reconnect
request fresh state
resubscribe
```

Do not assume event sequence continuity across daemon restart.

---

# 27. Client Crash

If UI crashes:

```text
daemon continues
```

Messages/files/DTN remain active.

When UI returns:

```text
state reconstructed from daemon
```

---

# 28. Daemon Crash

System supervisor restarts daemon.

Part 09 recovery runs.

UI reconnects after runtime ready.

---

# 29. Supervisor Architecture

```text
Process Supervisor
       ↓
comm-daemon
       ↓
Runtime Supervisor
├── Event Projector
├── Outbox Worker
├── Transfer Scheduler
├── DTN Engine
├── Routing Worker
├── Discovery Worker
└── Health Monitor
```

---

# 30. Worker Supervision

Each long-lived worker has:

```text
start
stop
health
restart policy
resource budget
```

---

# 31. Restart Policy

Use bounded exponential backoff.

Repeated crashes:

```text
subsystem degraded
```

rather than infinite restart loops.

---

# 32. Runtime Readiness

Daemon state:

```rust
pub enum RuntimeReadiness {
    Starting,
    Recovering,
    Ready,
    Degraded,
    ReadOnly,
    Fatal,
}
```

IPC clients can query this.

---

# 33. Startup Sequence

Recommended:

```text
load config
 ↓
open secure storage
 ↓
open DB
 ↓
run recovery
 ↓
initialize identity
 ↓
restore queues
 ↓
start local IPC
 ↓
start networking
 ↓
start background workers
 ↓
Ready
```

IPC can start before networking if it reports `Recovering`.

---

# 34. Network Start Barrier

Do not accept network traffic until:

```text
identity
dedup state
critical stores
```

are ready.

---

# 35. Graceful Shutdown

Sequence:

```text
stop accepting new commands
 ↓
stop new network work
 ↓
flush critical state
 ↓
persist clean-shutdown marker
 ↓
close IPC
```

---

# 36. Forced Shutdown

Must still recover safely.

Do not depend on graceful shutdown for correctness.

---

# 37. Systemd Deployment

Linux service:

```text
comm-daemon.service
```

Recommended:

```text
Restart=on-failure
private runtime directory
restricted permissions
resource limits
sandboxing
```

---

# 38. Linux User Service

Desktop personal app can use:

```text
systemd --user
```

This is often better than root system service.

Daemon runs as the logged-in user.

---

# 39. Root Avoidance

Personal communication daemon should not require root.

Platform-specific capabilities should use:

```text
user permissions
polkit/helper only where absolutely necessary
```

---

# 40. Windows Service

Possible deployment:

```text
Windows Service
```

for always-on/headless.

For ordinary desktop user app, a user-session background process may be simpler.

Keep the runtime model same.

---

# 41. macOS Service

Potential:

```text
LaunchAgent
```

for user-session daemon.

System-level daemon only if truly needed.

---

# 42. Android Service

Android may host the Rust runtime inside:

```text
foreground service
application process
```

depending on active feature.

---

# 43. Android Ownership

Kotlin service owns:

```text
Android lifecycle
notification channel
foreground-service APIs
permission callbacks
```

Rust owns:

```text
communication runtime
transfer state
DTN state
routing
identity
```

---

# 44. Android Process Kill

If process is killed:

```text
durable state remains
```

When service/app restarts:

```text
Part 09 recovery
```

restores work.

---

# 45. Android Foreground-Service Conditions

Use for:

```text
active call
visible long transfer
emergency relay
```

Do not run permanently merely to avoid Android policy.

---

# 46. iOS Headless Constraints

iOS does not support arbitrary permanent daemons for normal apps.

The same Rust core can run during:

```text
foreground
approved background windows
system-provided execution opportunities
```

Persisted intent survives suspension.

---

# 47. Headless Linux Node

Headless node can enable:

```text
Iroh endpoint
DTN relay
local proximity
LAN discovery
self-hosted gateway
file relay/cache
```

No UI crate linked.

---

# 48. Feature-Reduced Binary

Example:

```text
comm-node
```

features:

```text
headless
dtn
iroh
lan
ble
```

without:

```text
Dioxus
media UI
desktop rendering
```

---

# 49. Binary Split

Recommended executables:

```text
comm-daemon
comm-cli
comm-desktop
comm-node
comm-conformance
```

Mobile may package runtime inside app/service rather than separate executable.

---

# 50. Workspace Layout

```text
apps/
├── desktop/
├── daemon/
├── cli/
└── node/

crates/
├── comm-runtime/
├── comm-ipc/
├── comm-daemon-api/
├── comm-admin/
└── comm-platform-*
```

---

# 51. `comm-runtime`

Owns:

```text
subsystem composition
supervision
lifecycle
health
```

It should be UI-neutral.

---

# 52. Runtime Builder

```rust
let runtime = CommunicationRuntime::builder()
    .identity(identity)
    .event_store(events)
    .blob_store(blobs)
    .transport(iroh)
    .build()
    .await?;
```

---

# 53. Runtime Handle

```rust
pub struct RuntimeHandle {
    // command/query interface
}
```

In-process clients use it directly.

Daemon wraps it with IPC.

---

# 54. Same API, Different Boundary

Design goal:

```text
InProcess:
UI → RuntimeHandle

Daemon:
UI → IPC Client → RuntimeHandle
```

This avoids duplicating business APIs.

---

# 55. Daemon API Layer

Use stable request/response DTOs.

Do not expose internal Arc/Mutex/domain structs over IPC.

---

# 56. Command IDs

Every mutating IPC command should have:

```rust
pub struct CommandId([u8; 16]);
```

This enables idempotency if client retries after disconnect.

---

# 57. IPC Retry Safety

Scenario:

```text
UI sends SendMessage
daemon commits
IPC response lost
UI reconnects
retries same CommandId
```

Daemon returns original result instead of creating duplicate action.

---

# 58. Command Deduplication

Keep bounded cache or durable mapping for important commands.

For operations already using stable IDs, CommandId can map to them.

---

# 59. Query Pagination

Large queries must be paginated.

Examples:

```text
message history
transfer history
device audit
```

Do not send entire DB snapshot.

---

# 60. Cursor Design

Use opaque:

```rust
pub struct QueryCursor(Bytes);
```

Bound and version it.

---

# 61. Streaming Queries

For:

```text
large file export
diagnostic bundle
```

use stream handles rather than huge response frames.

---

# 62. Local File Transfer Through IPC

If UI selects file:

```text
UI obtains platform file handle/path
 ↓
passes safe reference/FD to daemon where platform supports
```

or daemon imports through approved path.

Avoid copying multi-GB file through IPC unnecessarily.

---

# 63. Unix FD Passing

On Unix, future optimization can use:

```text
SCM_RIGHTS
```

to pass file descriptors.

Keep behind platform adapter.

---

# 64. Windows Handle Passing

Windows can use native handle duplication mechanisms.

Again, adapter-only.

---

# 65. Mobile File Source

On Android, content URI access may require Kotlin/platform involvement.

Daemon/runtime core sees:

```text
FileSource
```

abstraction.

---

# 66. Remote Administration

Headless nodes may need remote admin.

Do not expose the local IPC protocol directly to the Internet.

Use a separate secure admin transport.

---

# 67. Admin Plane

```text
comm-admin
```

Responsibilities:

```text
health
config
logs
node status
maintenance
```

Not:

```text
message plaintext browsing by default
```

---

# 68. Remote Admin Authentication

Use strong admin identity:

```text
mutual TLS
SSH-like key auth
organization certificate
```

Separate from user messaging identity unless deliberately unified.

---

# 69. Admin Authorization

Roles:

```text
Viewer
Operator
SecurityAdmin
Owner
```

---

# 70. Admin API Security

Every remote admin command:

```text
authenticated
authorized
audited
rate-limited
```

---

# 71. No Remote Shell by Default

Admin API should expose explicit operations.

Do not embed arbitrary shell execution.

---

# 72. Headless Provisioning

Part 15 QR/NFC can enroll a headless node.

Flow:

```text
node starts unprovisioned
 ↓
terminal QR
 ↓
admin scans
 ↓
node receives signed role/config
 ↓
daemon activates
```

---

# 73. Unprovisioned Mode

Before enrollment:

```text
no public relay forwarding
no trusted messaging
limited bootstrap listener only
```

---

# 74. Node Roles

```rust
pub enum NodeRole {
    PersonalDevice,
    RelayGateway,
    DtnNode,
    OrganizationNode,
    StorageNode,
    TestNode,
}
```

Role controls enabled features.

---

# 75. Headless Capability Set

Example DTN node:

```text
Iroh
BLE
LAN
DTN
files/chunks
no calls
no Dioxus
```

---

# 76. Embedded Node Profile

Part 20 will specialize:

```text
low memory
low CPU
limited storage
```

but uses same daemon/runtime architecture.

---

# 77. Multiple Local Users

System-wide daemon is harder because user data must be isolated.

Preferred desktop personal model:

```text
one daemon per OS user
```

---

# 78. Per-User Runtime Directory

Example:

```text
$XDG_RUNTIME_DIR/comm/
```

and:

```text
$XDG_DATA_HOME/comm/
```

with restrictive permissions.

---

# 79. Windows User Isolation

Use per-user app data and named-pipe ACLs.

---

# 80. Multi-Account Support

One user may have multiple communication identities.

Two designs:

```text
single daemon with account namespaces
```

or:

```text
one daemon instance per profile
```

Single daemon is more efficient but requires strict isolation.

---

# 81. Account Namespace

```rust
pub struct RuntimeAccountId(...);
```

Every command/query includes active account context.

---

# 82. Cross-Account Isolation

Never allow:

```text
conversation cache
blob reference
device directory
```

to leak between accounts.

---

# 83. Multi-Tenant Headless Mode

Server node may serve many tenants.

Part 08 hierarchical quotas apply.

---

# 84. Tenant Namespace

```text
TenantId
AccountId
DeviceId
```

must be explicit in all durable server-side state.

---

# 85. Config Architecture

Daemon config should be versioned.

Example:

```ron
(
    version: 1,
    mode: LocalDaemon,
    data_dir: "...",
    power_profile: Balanced,
)
```

---

# 86. Secret Separation

RON config must not contain:

```text
private identity keys
admin passwords
relay credentials
```

Load secrets through secure storage/environment/secret manager.

---

# 87. Hot Reload

Some config can reload:

```text
resource limits
relay pools
logging level
```

Others require restart:

```text
data directory
identity backend
```

---

# 88. Config Validation

Reject invalid combinations before runtime start.

---

# 89. Runtime Health

```rust
pub struct RuntimeHealth {
    pub readiness: RuntimeReadiness,
    pub identity: HealthState,
    pub storage: HealthState,
    pub messaging: HealthState,
    pub files: HealthState,
    pub dtn: HealthState,
    pub networking: HealthState,
}
```

---

# 90. Health Endpoint

Local/admin query:

```text
health
```

should be cheap and not expose secrets.

---

# 91. Metrics

Daemon metrics:

```text
active peers
outbox depth
transfer count
DTN bundles
IPC clients
worker restarts
memory
storage
network
```

---

# 92. Local Metrics Export

Desktop:

```text
off by default
```

or local-only.

Headless:

```text
Prometheus/OpenTelemetry adapter
```

optional.

---

# 93. Logs

Structured logs:

```text
timestamp
subsystem
event
severity
operation id
```

No plaintext messages by default.

---

# 94. Trace Correlation

Use:

```text
CorrelationId
TransferId
BundleId
CommandId
```

for tracing.

---

# 95. Diagnostic Bundle

Admin/user can export:

```text
versions
health
config redacted
metrics snapshot
recent errors
```

No secrets or message bodies unless explicitly opted in.

---

# 96. Crash Report

Crash report may include:

```text
stack
build ID
subsystem state summary
```

with privacy redaction.

---

# 97. Watchdog

Headless deployments can use:

```text
systemd watchdog
```

or external supervisor.

Internal watchdog should monitor:

```text
worker liveness
event-loop stalls
```

---

# 98. Avoid Self-Restart Loop

Daemon should not repeatedly exec itself internally without supervisor unless carefully designed.

Prefer OS supervisor.

---

# 99. Service Readiness Notification

On Linux systemd, optional:

```text
sd_notify READY=1
```

when runtime actually ready.

---

# 100. Graceful Upgrade

Desktop daemon upgrade:

```text
new binary installed
 ↓
daemon drains
 ↓
restart
 ↓
recovery
 ↓
UI reconnects
```

---

# 101. Rolling Upgrade

Headless fleet:

```text
one node at a time
```

where possible.

---

# 102. Protocol Compatibility

UI and daemon may temporarily be different versions during update.

IPC must negotiate compatible version.

---

# 103. IPC Versioning

```text
comm-ipc/1
comm-ipc/2
```

Unknown major:

```text
explicit incompatibility
```

---

# 104. Backward-Compatible IPC

Prefer additive optional fields/features inside major.

Do not require simultaneous UI/daemon replacement for every minor update.

---

# 105. IPC Capability Negotiation

Part 07 can be reused conceptually for local IPC:

```text
UI supports transfer preview v2
daemon supports v1
→ use v1
```

---

# 106. Daemon Migration Ownership

Only daemon performs DB migrations.

UI never migrates communication database.

---

# 107. Upgrade Lock

Prevent two daemon instances from running migrations concurrently.

---

# 108. Single-Instance Lock

Use:

```text
OS lock
socket ownership
database lock
```

to ensure one daemon per profile.

---

# 109. Stale Lock Recovery

Do not trust PID file alone.

Prefer OS-native locks.

---

# 110. Data Directory Ownership

Daemon verifies:

```text
permissions
owner
expected filesystem
```

before using sensitive data.

---

# 111. Read-Only Mode

If storage corrupt/read-only:

```text
daemon can serve queries/export
```

while blocking new sends.

---

# 112. Maintenance Mode

Admin can place daemon into:

```text
Maintenance
```

to:

```text
run backup
repair
migration
```

No new network mutations.

---

# 113. Backup Coordination

Daemon should expose:

```text
create consistent backup
```

rather than external scripts copying live DB blindly.

---

# 114. Backup API

```rust
pub trait BackupService {
    async fn create_backup(&self, target: BackupTarget) -> Result<BackupId, BackupError>;
}
```

---

# 115. Restore Mode

Restore should require:

```text
daemon stopped or maintenance mode
```

then Part 09 restore reconciliation.

---

# 116. Security Boundaries

Critical boundaries:

```text
UI process
daemon process
platform helper
remote admin client
plugin host
```

Treat each as separate trust domain where possible.

---

# 117. Sandboxing

Daemon can use OS hardening:

Linux:

```text
systemd sandboxing
seccomp where practical
filesystem restrictions
```

Windows/macOS equivalent where practical.

---

# 118. Least Privilege

Daemon should run with only permissions needed.

Avoid:

```text
root
CAP_NET_ADMIN
raw socket
```

unless feature requires and isolated helper is used.

---

# 119. Privileged Helper

If a platform feature truly requires elevated permission:

```text
small dedicated helper
```

with narrow IPC.

Do not run entire communication daemon privileged.

---

# 120. Plugin Isolation

Part 24 plugins should not run in daemon process by default if untrusted/third-party.

Prefer:

```text
plugin host process
+
bounded IPC
```

---

# 121. Plugin Crash

Plugin host crash:

```text
core daemon continues
```

---

# 122. UI Plugin Separation

UI extensions can be isolated from communication core.

---

# 123. Automation Clients

Future local automation can connect through explicit API.

Example:

```text
send file when generated
query transfer state
```

Requires client token/role.

---

# 124. No Hidden Database API

External products reusing SDK should use:

```text
runtime API
IPC API
library API
```

not direct DB mutation.

---

# 125. Reusable Embedding Modes

Other software can integrate:

## Library Mode

```text
link comm-runtime
```

## Local Daemon Mode

```text
connect to comm-daemon
```

## Remote Headless Node

```text
use application protocol/admin API
```

This increases reusability.

---

# 126. SDK Boundary

Expose stable:

```text
commands
queries
events
```

not UI-specific types.

---

# 127. File-Only Headless Product

A product can compile:

```text
comm-runtime
comm-files
comm-routing
comm-transport-iroh
```

without messaging UI.

---

# 128. Messaging-Only Product

Compile:

```text
messaging
identity
runtime
routing
```

without file subsystem if desired.

---

# 129. Emergency Node

Compile:

```text
DTN
proximity
routing
runtime
admin
```

without Dioxus.

---

# 130. Runtime Feature Flags

Example:

```toml
[features]
messaging = []
files = []
dtn = []
calls = []
proximity = []
daemon = []
headless = []
```

Avoid feature explosion; group logically.

---

# 131. Build Matrix

Part 10 CI should build:

```text
desktop-full
daemon-headless
file-only
dtn-node
minimal-runtime
```

---

# 132. Startup Performance

Daemon should become locally queryable quickly.

Secondary work can initialize later.

Example:

```text
IPC available
recent messages available
network starts
search index rebuild continues
```

---

# 133. Lazy Initialization

Optional subsystems can start only when enabled.

Example:

```text
Bluetooth proximity disabled
→ no BLE worker
```

---

# 134. Memory Efficiency

Headless mode should avoid:

```text
UI assets
render state
image caches
```

---

# 135. Process Memory Budget

Part 08 sets profile-specific daemon budget.

---

# 136. Idle Efficiency

Long-lived daemon should have:

```text
low wakeups
no busy loops
no frequent polling
event-driven timers
```

especially on laptops/mobile.

---

# 137. Polling Policy

Prefer:

```text
notifications
watchers
async waits
```

over fixed 1-second loops.

---

# 138. Timer Coalescing

Batch timers where possible.

Part 13 power policy can help.

---

# 139. Network Listener Lifecycle

Listeners:

```text
Iroh
LAN
proximity
IPC
```

should be individually restartable.

---

# 140. Partial Degradation

If BLE worker fails:

```text
Internet messaging still works
```

If DTN fails:

```text
direct messaging still works
```

Subsystem isolation is critical.

---

# 141. Health State Per Subsystem

```rust
pub enum HealthState {
    Healthy,
    Degraded,
    Unavailable,
    Recovering,
}
```

---

# 142. Circuit Breaker

Repeated failing external subsystem:

```text
temporarily stop attempts
```

Example:

```text
Bluetooth API repeatedly errors
```

Use cooldown.

---

# 143. Remote Endpoint Flood

Daemon is network-facing.

Part 08 and Part 10 protections apply independent of UI.

---

# 144. IPC Flood

Local malicious process may flood daemon.

Rate-limit and bound per client.

---

# 145. Slow IPC Client

Do not let one slow UI block daemon event loop.

Use per-client bounded queue and async writer.

---

# 146. Client Backpressure

If client misses noncritical deltas:

```text
send ResyncRequired
```

---

# 147. IPC Event Priority

Examples:

```text
Critical:
security state

Interactive:
message arrived

Normal:
transfer progress

Ephemeral:
network metric updates
```

---

# 148. Progress Coalescing

Transfer progress:

```text
update at e.g. bounded Hz
```

not every chunk.

---

# 149. Message Pagination

UI should fetch conversation page windows.

Daemon owns search/pagination logic.

---

# 150. Search IPC

Query:

```text
search messages
```

returns:

```text
bounded result page
```

not search-index internals.

---

# 151. Notification Service

Daemon/runtime decides semantic notification events.

Platform UI/service adapter renders OS notification.

---

# 152. Desktop Notification

Daemon may use platform adapter or notify UI.

Avoid hard UI dependency.

---

# 153. Android Notification

Kotlin service renders notification based on Rust semantic event.

---

# 154. Incoming Call

Mobile service may need to wake/show incoming-call UI according to OS policy.

Rust runtime owns call state.

---

# 155. Incoming Message While UI Closed

Daemon:

```text
persists message
updates projection
emits notification event
```

UI can remain absent.

---

# 156. Transfer While UI Closed

Daemon continues:

```text
chunking
routing
resume
```

according to power/background policy.

---

# 157. DTN While UI Closed

On desktop/headless:

```text
DTN can continue continuously
```

On mobile:

```text
subject to platform scheduling
```

---

# 158. Proximity While UI Closed

Depends on OS and Part 13 battery policy.

---

# 159. Remote Admin over Iroh

Future option:

```text
admin endpoint over authenticated Iroh
```

can avoid opening public TCP admin port.

Still use separate admin identity/protocol.

---

# 160. Admin Capability

Headless node may advertise:

```text
admin protocol available
```

only to authorized operators.

---

# 161. Audit Log

Admin commands:

```text
config changed
node restarted
relay disabled
```

recorded durably.

---

# 162. No Message-Content Admin by Default

Operators should not automatically gain access to E2EE content.

---

# 163. Backup Keys

Admin backup of server/headless configuration must keep identity key semantics clear.

---

# 164. Secure Key Backend

Possible:

```text
OS keychain
Android Keystore
TPM
software-encrypted key store
```

Daemon accesses through common secure-store trait.

---

# 165. Secure Store Trait

```rust
pub trait SecureStore {
    async fn load_key(&self, id: KeyId) -> Result<SecretKeyHandle, SecureStoreError>;
}
```

Prefer handles where platform supports non-exportable keys.

---

# 166. Headless TPM

Server/embedded node may use:

```text
TPM-backed keys
```

optional.

---

# 167. Daemon PID/Instance Metadata

Expose:

```text
instance ID
build ID
uptime
recovery generation
```

for diagnostics.

---

# 168. Instance ID

```rust
pub struct RuntimeInstanceId([u8; 16]);
```

Changes every process start.

Useful for client reconnect logic.

---

# 169. Event Sequence

Within one daemon instance:

```text
monotonic IPC event sequence
```

helps clients detect gaps.

---

# 170. Event Gap

If client receives:

```text
seq 100
then 105
```

request resync.

---

# 171. Persistent vs Instance Sequence

Do not treat IPC sequence as durable event-log offset.

They serve different purposes.

---

# 172. Runtime Clock

Daemon should own coherent monotonic timers.

Clients should not schedule critical network retry logic independently.

---

# 173. Shutdown Authorization

Ordinary UI may request daemon shutdown.

Headless/service mode may restrict it to admin.

---

# 174. Auto-Start

Desktop app installer may configure daemon autostart.

This should be user-visible and disable-able.

---

# 175. Autostart Policy

Options:

```text
start at login
start on demand
always-on system service
```

depends on product mode.

---

# 176. On-Demand Daemon

UI can spawn daemon if absent.

Need race-safe single-instance startup.

---

# 177. Spawn Race

Two clients start simultaneously.

Only one daemon wins instance lock.

Other client connects to existing daemon.

---

# 178. Daemon Discovery

Clients locate daemon via:

```text
fixed per-user socket path
runtime directory
platform service registry
```

---

# 179. Socket Permissions

Unix socket:

```text
0600-like
```

or user-only directory.

---

# 180. Named Pipe ACL

Windows:

```text
current user only
```

unless multi-user service intended.

---

# 181. Remote Node Discovery

Headless admin tools discover nodes through:

```text
configured identity
QR enrollment
directory
Iroh endpoint
```

not local IPC.

---

# 182. Versioned Admin Protocol

Keep separate:

```text
comm-admin/1
```

from:

```text
comm-ipc/1
```

---

# 183. Testing — IPC

Part 10 should test:

```text
malformed frame
unknown command
slow client
disconnect mid-command
duplicate CommandId
version mismatch
event gap
```

---

# 184. Testing — Daemon Crash

```text
send message
kill daemon
restart
UI reconnect
message sends exactly once logically
```

---

# 185. Testing — UI Crash

```text
start transfer
kill UI
transfer continues
restart UI
progress restored
```

---

# 186. Testing — Slow Client

UI stops reading events.

Expected:

```text
daemon bounded memory
client eventually resyncs
```

---

# 187. Testing — Multi-Client

CLI and Dioxus connected simultaneously.

Both see consistent state.

---

# 188. Testing — Duplicate Commands

UI retries after timeout.

Same CommandId:

```text
one logical operation
```

---

# 189. Testing — Upgrade

Old UI ↔ new daemon within supported compatibility.

Expected graceful negotiation.

---

# 190. Testing — Headless Node

Run:

```text
no display
no Dioxus
```

for days.

Verify memory/task/FD stability.

---

# 191. Testing — Platform Service

Android:

```text
activity destroyed
service alive
transfer continues
```

then:

```text
service killed
restart
recovery
```

---

# 192. Testing — Resource Limits

Many IPC clients.

Expected:

```text
bounded clients
bounded queues
fairness
```

---

# 193. Fuzzing

Fuzz:

```text
IPC parser
command payloads
query payloads
admin protocol
config
```

---

# 194. Security Tests

Attempt:

```text
unauthorized local client
socket permission bypass
admin command as UI role
duplicate command replay
```

---

# 195. Process Isolation Test

Third-party plugin host crash must not crash daemon.

---

# 196. Performance Tests

Measure:

```text
IPC round-trip
message-list query
transfer progress event load
100 clients/headless peers
```

---

# 197. Idle Power Test

Desktop/mobile daemon runtime idle should have low wakeup frequency.

---

# 198. Soak Test

Run daemon for:

```text
7+ days
```

with reconnects/transfers/DTN.

Track:

```text
memory
FDs
tasks
queue growth
```

---

# 199. Suggested Crate Structure

```text
crates/comm-runtime/
├── src/
│   ├── lib.rs
│   ├── builder.rs
│   ├── handle.rs
│   ├── lifecycle.rs
│   ├── supervisor.rs
│   ├── worker.rs
│   ├── health.rs
│   └── error.rs

crates/comm-ipc/
├── src/
│   ├── lib.rs
│   ├── protocol.rs
│   ├── transport.rs
│   ├── client.rs
│   ├── server.rs
│   ├── auth.rs
│   ├── command.rs
│   ├── query.rs
│   ├── event.rs
│   ├── subscription.rs
│   └── error.rs

crates/comm-daemon-api/
├── src/
│   ├── commands/
│   ├── queries/
│   ├── events/
│   └── version.rs

crates/comm-admin/
├── src/
│   ├── protocol.rs
│   ├── auth.rs
│   ├── audit.rs
│   └── maintenance.rs
```

---

# 200. Executables

```text
apps/daemon/
apps/cli/
apps/desktop/
apps/node/
```

---

# 201. `comm-daemon`

Main responsibilities:

```text
load configuration
acquire instance lock
recover
start IPC
start runtime
supervise
shutdown
```

---

# 202. `comm-node`

Headless profile:

```text
no desktop IPC required if remote admin only
no UI
optimized features
```

Can still expose local admin socket.

---

# 203. CLI Example

```text
comm status
comm devices
comm send <peer> "hello"
comm files send <peer> <path>
comm dtn pending
comm doctor
comm daemon restart
```

---

# 204. Public Runtime API

```rust
pub trait CommunicationRuntimeApi {
    async fn command(
        &self,
        command: RuntimeCommand,
    ) -> Result<RuntimeCommandResult, RuntimeError>;

    async fn query(
        &self,
        query: RuntimeQuery,
    ) -> Result<RuntimeQueryResult, RuntimeError>;

    fn subscribe(
        &self,
        topic: RuntimeTopic,
    ) -> RuntimeEventStream;
}
```

Same conceptual API works:

```text
in-process
IPC
FFI
```

---

# 205. Error Model

```rust
pub enum RuntimeError {
    NotReady,
    ReadOnly,
    Unauthorized,
    Unsupported,
    Busy,
    Storage,
    Identity,
    Network,
    Ipc,
    InvalidCommand,
    Cancelled,
}
```

---

# 206. Initial Production Scope

Implement first:

```text
comm-runtime
local daemon mode
Unix socket IPC
Windows named-pipe IPC
sole-writer DB model
command/query/event API
bounded subscriptions
CommandId idempotency
systemd --user integration
desktop autostart
headless Linux node
runtime health
crash recovery integration
```

Then:

```text
Android service hosting
remote admin protocol
multi-account namespaces
plugin-host isolation
```

Defer initially:

```text
system-wide multi-user daemon
complex remote orchestration
clustered daemon state
```

---

# 207. Implementation Phases

## Phase 1 — Runtime Core

```text
CommunicationRuntime
RuntimeHandle
Supervisor
Health
```

## Phase 2 — Local IPC

```text
protocol
Unix socket
named pipe
auth
```

## Phase 3 — Desktop Daemon

```text
single instance
autostart
Dioxus client
CLI client
```

## Phase 4 — Recovery

```text
daemon restart
client reconnect
CommandId replay
```

## Phase 5 — Headless

```text
comm-node
systemd service
admin socket
```

## Phase 6 — Mobile Hosting

```text
Android service
iOS execution adapter
```

## Phase 7 — Security/Isolation

```text
roles
plugin host
remote admin
```

## Phase 8 — Hardening

```text
fuzz
soak
multi-client
upgrade
idle-power
```

---

# 208. Definition of Done

Part 16 is complete when:

- the communication runtime can operate without Dioxus
- desktop UI can close while daemon continues messaging/files/DTN
- daemon is the sole writer of durable communication state
- CLI and GUI use the same command/query semantics
- IPC is versioned, authenticated, bounded, and backpressure-aware
- mutating IPC commands can be retried idempotently
- UI can reconnect after daemon restart and request a fresh snapshot
- daemon recovery runs before network listeners become fully active
- headless Linux node can run unattended
- systemd user/service integration works
- Windows local-service/background mode has an equivalent IPC design
- Android service hosts the same Rust runtime logic
- iOS degrades to system-approved execution windows without losing durable intent
- remote admin uses a separate authenticated protocol
- third-party plugin failures can be isolated
- per-user/profile data separation is enforced
- health, diagnostics, logs, metrics, backup, and maintenance modes exist
- fuzz, crash, reconnect, slow-client, soak, and permission tests exist

---

# 209. Relationship to Earlier Parts

Part 16 builds on:

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
```

It directly supports:

```text
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

# 210. Final Architecture

```text
                    LOCAL USER SPACE
 ┌──────────────────────────────────────────────────────┐
 │                                                      │
 │   Dioxus UI              CLI             Automation  │
 │      │                    │                  │        │
 │      └────────────── Secure IPC ─────────────┘        │
 │                           │                          │
 │                    ┌──────▼──────┐                   │
 │                    │ comm-daemon │                   │
 │                    ├─────────────┤                   │
 │                    │ Identity    │                   │
 │                    │ Event Log   │                   │
 │                    │ Messaging   │                   │
 │                    │ Files       │                   │
 │                    │ DTN         │                   │
 │                    │ Routing     │                   │
 │                    │ Proximity   │                   │
 │                    │ Power       │                   │
 │                    │ Recovery    │                   │
 │                    └──────┬──────┘                   │
 └───────────────────────────┼───────────────────────────┘
                             │
              ┌──────────────┼───────────────┐
              │              │               │
            Iroh            LAN           BLE/Wi-Fi
              │              │               │
              └──────────────┼───────────────┘
                             │
                          Peers
```

Headless variant:

```text
systemd / Windows service / embedded supervisor
                    │
               comm-node
                    │
     ┌──────────────┼──────────────┐
     │              │              │
   Iroh            DTN         Proximity
     │              │              │
     └──────────────┼──────────────┘
                    │
                 Network
```

---

# 211. Final Principle

The daemon/headless architecture should make this behavior normal:

```text
A user starts a 10 GB transfer.

They close the Dioxus window.

The transfer continues.

The daemon switches from relay to LAN.

The laptop sleeps and later wakes.

The daemon recovers.

The transfer resumes.

The user opens the UI again.

The UI reconnects and immediately sees the correct progress.
```

And the same runtime should also support:

```text
a Raspberry Pi emergency relay
a headless enterprise gateway
a local CLI tool
an Android foreground service
a reusable embedded library
```

without duplicating the communication engine.

That separation is what turns the project from a GUI application into a reusable communication platform.
