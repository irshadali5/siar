# Part 20 — Embedded Linux Node Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 20 of 24  
**Primary language:** Rust  
**Primary targets:** Raspberry Pi-class SBCs, ARM/x86 embedded Linux, gateways, kiosks, routers, field nodes, disaster-relay nodes, industrial edge devices  
**Primary goals:** low-resource operation, unattended service, resilient networking, DTN gateway behavior, power-loss tolerance, secure provisioning, maintainable updates, cross-compilation, observability, and reuse without any GUI dependency

---

# 1. Purpose

The communication platform should support not only phones and desktops, but also small always-on Linux nodes.

Examples:

```text
Raspberry Pi
Orange Pi
Rockchip SBC
x86 mini PC
industrial ARM gateway
router-class Linux device
field emergency relay
school/campus gateway
vehicle node
community mesh node
NAS appliance
```

These devices are valuable because they can remain:

```text
always on
always listening
physically deployed
connected to multiple networks
plugged into mains or battery backup
```

and can serve as:

```text
DTN relay
Internet gateway
nearby discovery node
file cache
local coordination node
self-hosted relay companion
organization edge node
emergency broadcast node
```

The core rule is:

> **An embedded node must run the same communication core as desktop/mobile, but with stricter resource, storage, lifecycle, and unattended-operation policies.**

---

# 2. Architectural Position

```text
Embedded Linux Device
      ↓
comm-node
      ↓
CommunicationRuntime
├── Identity
├── Routing
├── Iroh
├── LAN
├── BLE
├── DTN
├── Files
├── Diagnostics
├── Emergency
└── Power/Resource Policy
      ↓
Local / Internet / Offline Peers
```

No Dioxus dependency is required.

---

# 3. Primary Embedded Roles

```rust
pub enum EmbeddedNodeRole {
    PersonalGateway,
    DtnRelay,
    InternetGateway,
    EmergencyNode,
    OrganizationEdge,
    FileCache,
    RelayCompanion,
    ProvisioningNode,
}
```

A deployment may combine roles.

---

# 4. Personal Gateway

Example:

```text
home Raspberry Pi
```

Responsibilities:

```text
always-on peer presence
DTN relay for personal devices
local file transfer helper
remote device bridge
```

---

# 5. DTN Relay

Responsibilities:

```text
receive encrypted bundles
store
carry
forward
expire
deduplicate
ack
```

No message plaintext required.

---

# 6. Internet Gateway

Bridges:

```text
local/offline proximity network
→ Iroh/Internet
```

Example:

```text
BLE/Wi-Fi field devices
→ gateway
→ remote Internet peer
```

---

# 7. Emergency Node

Optimized for:

```text
SOS relay
authority alerts
local discovery
DTN
low-power operation
gateway detection
```

---

# 8. Organization Edge Node

Could be deployed at:

```text
school
hospital
office
warehouse
campus
```

for:

```text
local communication
document transfer
emergency alerts
edge caching
organization-specific relay policy
```

---

# 9. File Cache Node

Stores encrypted blobs/chunks opportunistically.

Use cases:

```text
large local files
software/media distribution
temporary offline caching
```

Must not become authoritative user storage unless explicitly configured.

---

# 10. Relay Companion

Can run alongside:

```text
iroh-relay
```

for:

```text
DTN gateway
organization control
local discovery
health monitoring
```

Relay and DTN roles remain logically separate.

---

# 11. Headless-First Design

Embedded node must compile without:

```text
Dioxus
desktop rendering
camera UI
mobile frameworks
large image/UI dependencies
```

Use:

```text
CLI
systemd
remote admin
config files
QR terminal enrollment
```

---

# 12. Binary

Recommended executable:

```text
comm-node
```

Optional specialized binaries:

```text
comm-node-dtn
comm-node-gateway
comm-node-emergency
```

but one configurable binary may be easier initially.

---

# 13. Resource Profiles

Embedded nodes need dedicated profiles.

```rust
pub enum EmbeddedResourceProfile {
    Tiny,
    Small,
    Standard,
    EdgeServer,
}
```

---

# 14. Tiny Profile

Target:

```text
256–512 MiB RAM
low-end ARM
small flash
```

Enable only:

```text
identity
Iroh
LAN
basic DTN
diagnostics
```

Avoid:

```text
software AV1
heavy indexing
large concurrent transfer sets
```

---

# 15. Small Profile

Target:

```text
1 GB RAM
```

Supports:

```text
moderate DTN
BLE
LAN
files
gateway
```

---

# 16. Standard Profile

Target:

```text
2–4 GB RAM
```

Supports:

```text
larger relay/cache
multiple transfers
rich diagnostics
```

---

# 17. EdgeServer Profile

Target:

```text
4 GB+
fast storage
multi-NIC
```

Supports:

```text
high concurrency
larger cache
multiple tenants
```

---

# 18. Hard Resource Budgets

Part 08 applies strictly.

Embedded profile should bound:

```text
RAM
open files
connections
streams
transfer workers
DTN bytes
blob cache
event queue
diagnostic history
```

---

# 19. Example Tiny Limits

Illustrative only:

```text
active peers: 16
active file transfers: 2
DTN storage: 128 MiB
memory pool: 32 MiB
diagnostic history: small ring buffer
```

Tune from benchmarks.

---

# 20. No Unbounded Allocation

Especially avoid:

```text
unbounded Vec
unbounded async channels
full-file buffering
giant manifest materialization
```

---

# 21. Streaming Everything

Prefer:

```text
stream file
stream hash
stream decrypt
stream forward
```

over buffering full objects.

---

# 22. Compute Constraints

Low-end SBCs may struggle with:

```text
software AV1
large cryptographic batches
compression
hashing many GB concurrently
```

Use bounded CPU work queues.

---

# 23. Media Role

Embedded node should generally not encode/decode video unless explicitly configured.

It can relay encrypted media transport without understanding codec payload.

---

# 24. Storage Architecture

Recommended split:

```text
config
identity/secure state
event DB
DTN store
blob cache
temp/staging
diagnostics
```

---

# 25. Storage Classes

Use Part 08:

```text
Durable
Cache
Staging
Relay
Temporary
```

with separate quotas.

---

# 26. Flash Wear

Embedded flash/eMMC/SD cards have finite write endurance.

Reduce unnecessary writes.

Avoid:

```text
per-packet fsync
high-frequency diagnostic journaling
constantly rewritten counters
```

---

# 27. Write Coalescing

Batch safe metadata updates.

Do not weaken durability of critical identity/emergency events.

---

# 28. SQLite/WAL Considerations

If SQLite-like DB is used:

```text
WAL
checkpoint policy
fsync policy
```

must consider flash endurance and crash safety.

Tune based on hardware.

---

# 29. Read-Only Root Filesystem

For resilient appliances, support:

```text
read-only rootfs
```

with writable data partition:

```text
/var/lib/comm-node
```

or equivalent.

---

# 30. Data Partition

Recommended:

```text
config:
 /etc/comm-node

persistent:
 /var/lib/comm-node

runtime:
 /run/comm-node

logs:
 journald or bounded log path
```

---

# 31. Ephemeral Root

Possible deployment:

```text
immutable OS image
+
persistent data volume
```

This simplifies recovery and updates.

---

# 32. Filesystem Choice

Do not hard-code one filesystem.

Consider deployment properties:

```text
ext4
f2fs
btrfs
read-only squashfs + data partition
```

Validate atomic-rename/durability assumptions.

---

# 33. Power-Loss Resilience

Embedded field nodes may lose power without shutdown.

Part 09 requirements are mandatory.

Assume:

```text
no destructor
no flush callback
no graceful stop
```

---

# 34. Power-Loss Test

Physically or via fault harness:

```text
cut power
restart
verify
```

especially for:

```text
event log
DTN bundle store
blob staging
identity state
```

---

# 35. UPS Support

Optional:

```text
UPS HAT
USB UPS
battery-backed gateway
```

Power adapter can expose:

```text
mains lost
battery remaining
```

to Part 13.

---

# 36. Graceful Low-Power Mode

When UPS battery low:

```text
stop bulk
preserve critical DTN
flush essential state
```

---

# 37. Thermal Constraints

Fanless SBCs may throttle heavily.

Monitor:

```text
thermal zone
CPU frequency
```

where available.

Reduce:

```text
hash concurrency
file workers
background scanning
```

---

# 38. CPU Affinity

Optional advanced tuning.

Not required initially.

---

# 39. Network Interfaces

Embedded node may have:

```text
Ethernet
Wi-Fi
cellular modem
USB Ethernet
BLE
multiple NICs
```

Part 03/12 should use them as independent path candidates.

---

# 40. Ethernet Preference

For stationary gateways:

```text
Ethernet
```

is often ideal primary Internet path.

---

# 41. Wi-Fi Fallback

Use Wi-Fi as:

```text
backup
local AP
mesh/local path
```

depending role.

---

# 42. Cellular Modem

Optional:

```text
USB LTE/5G modem
```

for emergency gateways.

Mark as:

```text
metered
possibly roaming
```

---

# 43. Multi-Uplink

Embedded gateway can use:

```text
Ethernet + cellular
```

with Part 12 warm failover/multipath.

---

# 44. BLE Hardware

Prefer known-stable adapters.

Support:

```text
onboard BLE
USB Bluetooth dongle
```

through Linux proximity adapter.

---

# 45. Wi-Fi AP Mode

Some emergency/community nodes may provide a local AP.

Architecture:

```text
hostapd/system network layer
 ↓
LAN peers
 ↓
comm-node
```

Do not embed full AP management into core communication crate initially.

---

# 46. Local Captive Portal

Optional provisioning UI may exist, but should be a separate module.

Do not depend on it for core operation.

---

# 47. LAN Discovery

Part 14 provides:

```text
local peer discovery
```

without Internet.

---

# 48. DTN Encounter Support

BLE/LAN events trigger Part 06 encounter flow.

Embedded node can be a highly capable store-carry-forward peer.

---

# 49. Gateway Bridging

```text
Local DTN
   ↓
comm-node
   ↓
Iroh direct/relay
   ↓
Internet destination
```

This is a primary embedded use case.

---

# 50. Gateway State

```rust
pub enum GatewayState {
    Offline,
    LocalOnly,
    InternetAvailable,
    Degraded,
}
```

---

# 51. Gateway Announcement

Nearby peers may receive a coarse hint:

```text
Internet gateway available
```

Actual forwarding remains authenticated/policy-controlled.

---

# 52. Gateway Privacy

Do not broadcast:

```text
public IP
ISP
organization secrets
```

in nearby advertisements.

---

# 53. DTN Capacity Advertisement

Node can advertise:

```text
Low
Normal
High
```

relay capacity class.

Not exact free disk.

---

# 54. Organization Policy

Embedded nodes may be provisioned with:

```text
allowed tenant/org
relay policy
emergency authority trust
storage quota
public relay fallback policy
```

---

# 55. Secure Enrollment

Part 15 supports:

```text
terminal QR
NFC where hardware exists
signed enrollment token
```

---

# 56. First-Boot Flow

```text
boot
 ↓
unprovisioned mode
 ↓
display terminal QR
 ↓
admin scans
 ↓
secure enrollment
 ↓
write signed config
 ↓
activate node role
```

---

# 57. Unprovisioned Restrictions

Before enrollment:

```text
no user data relay
no organization authority
no remote admin except bootstrap
```

---

# 58. Device Identity

Embedded node generates its own device key locally.

Do not ship one shared private key across devices.

---

# 59. Hardware-Bound Keys

Where available:

```text
TPM 2.0
secure element
```

can store identity/admin keys.

---

# 60. TPM Integration

Optional secure-store backend:

```text
comm-secure-store-tpm
```

Should support:

```text
sign
seal/unseal
key handles
```

without exporting private key bytes.

---

# 61. Software Key Store

Fallback:

```text
encrypted key file
```

with strict filesystem permissions.

---

# 62. Remote Administration

Use Part 16 admin plane.

Possible transports:

```text
local Unix socket
SSH-wrapped CLI
authenticated Iroh admin protocol
mutual TLS
```

---

# 63. No Open Admin Port by Default

Prefer:

```text
local admin
or
authenticated overlay
```

rather than public HTTP admin endpoint.

---

# 64. Admin Roles

```text
Viewer
Operator
SecurityAdmin
Owner
```

---

# 65. Admin Commands

Examples:

```text
status
health
network doctor
DTN queue
storage
upgrade
restart
backup
config
```

---

# 66. No Remote Shell

Explicit admin API only.

Do not expose arbitrary shell execution through comm-node.

---

# 67. SSH

System administrators may still use SSH as an OS management layer.

This is separate from application admin API.

---

# 68. systemd

Recommended deployment:

```text
comm-node.service
```

Use:

```text
Restart=on-failure
WatchdogSec
ProtectSystem
ProtectHome
PrivateTmp
NoNewPrivileges
```

where compatible.

---

# 69. systemd Sandbox

Tighten:

```text
filesystem write paths
network families
capabilities
```

based on enabled features.

---

# 70. Watchdog

Use:

```text
systemd watchdog
```

or external supervisor.

Internal runtime health feeds readiness.

---

# 71. Boot Ordering

Node may need:

```text
network-online.target
bluetooth.service
```

but should still start in LocalOnly/Offline mode if Internet absent.

Do not block forever waiting for WAN.

---

# 72. Offline Boot

A disaster node must boot fully without:

```text
DNS
Internet
cloud database
```

---

# 73. Time Without Internet

If RTC is available:

```text
use it
```

Otherwise handle clock uncertainty for:

```text
expiry
certificates
DTN lifetime
```

carefully.

---

# 74. RTC

Embedded deployments should consider adding RTC hardware if expiry/security decisions rely on time across power loss.

---

# 75. Clock Trust

Distinguish:

```text
trusted time
approximate time
unknown time
```

where needed.

---

# 76. NTP

Use when Internet/local time source available.

Do not make core startup depend on NTP.

---

# 77. Storage Full Behavior

When nearly full:

```text
evict cache
expire DTN
stop bulk relay
preserve critical reserve
```

---

# 78. SD Card Failure

Detect I/O errors.

Enter:

```text
degraded
read-only
fatal
```

according to scope.

---

# 79. Removable Storage

Optional large blob/DTN store may use USB SSD.

Keep authoritative identity DB separate from removable cache where practical.

---

# 80. Storage Mount Loss

If external blob cache disappears:

```text
files/cache degraded
```

but identity/control can continue.

---

# 81. Event Store Location

Critical event DB should live on reliable persistent storage.

---

# 82. Blob Cache Location

Can live on larger external disk.

---

# 83. Storage Quotas

Example:

```text
DTN relay: 2 GiB
file cache: 8 GiB
critical reserve: 64 MiB
diagnostics: 32 MiB
```

Tune by device.

---

# 84. CPU Scheduling

Use bounded pools.

Avoid spawning one worker per peer/file.

---

# 85. Hashing

Large file hashing:

```text
1–2 workers
```

on low-end SBC.

---

# 86. Crypto Acceleration

Use hardware crypto where libraries/kernel provide safely.

Do not write custom crypto assembly without strong reason.

---

# 87. SIMD

Portable Rust/SIMD optimizations can be enabled by target.

Must have non-SIMD fallback where required.

---

# 88. Cross-Compilation Targets

Likely:

```text
aarch64-unknown-linux-gnu
armv7-unknown-linux-gnueabihf
x86_64-unknown-linux-gnu
```

Optional musl:

```text
aarch64-unknown-linux-musl
x86_64-unknown-linux-musl
```

depending dependencies.

---

# 89. GNU vs musl

Choose based on:

```text
Bluetooth/system integration
glibc dependency
static linking goals
system libraries
```

Do not assume musl works with every platform integration automatically.

---

# 90. Build Profiles

```toml
[profile.release]
lto = "thin"
codegen-units = 1
strip = "symbols"
panic = "abort"
```

Example only.

Benchmark compile/runtime tradeoffs.

---

# 91. Binary Size

Embedded builds should exclude:

```text
UI
unused codecs
developer tools
unused transports
```

through feature flags.

---

# 92. Feature Profiles

Example:

```text
embedded-basic
embedded-dtn
embedded-gateway
embedded-emergency
```

---

# 93. Example Cargo Features

```toml
[features]
iroh = []
lan = []
ble = []
dtn = []
files = []
emergency = []
admin = []
tpm = []
```

Keep feature interactions tested.

---

# 94. Cross Compilation Pipeline

Recommended:

```text
x86_64 Linux builder
→ cargo/cross/cargo-zigbuild or containerized toolchain
→ ARM artifacts
→ QEMU smoke
→ hardware test
```

---

# 95. Hardware Test Is Required

Cross-compilation success is not enough.

Need real SBC testing for:

```text
Bluetooth
Wi-Fi
power loss
thermal
SD/eMMC behavior
systemd
```

---

# 96. QEMU

Useful for:

```text
basic boot
binary execution
storage migration
config parsing
```

Not sufficient for radio/network hardware validation.

---

# 97. Supported Hardware Matrix

Maintain:

```text
board
CPU arch
RAM
storage
BLE
Wi-Fi
kernel
status
```

---

# 98. Kernel Requirements

Document minimum:

```text
kernel version
Bluetooth stack
network features
filesystem assumptions
```

based on actual dependencies.

---

# 99. Distribution Support

Potential:

```text
Debian
Ubuntu Server
Raspberry Pi OS
Fedora IoT
Yocto/OpenEmbedded image
Buildroot
custom immutable Linux
```

---

# 100. Generic Linux First

Keep comm-node distribution-neutral.

Package adapters later.

---

# 101. Debian Package

Provide:

```text
.deb
systemd unit
config directory
data directory setup
```

---

# 102. RPM Package

Optional later.

---

# 103. OCI Container

Possible for nodes with suitable host networking.

But Bluetooth/LAN may require host integration.

Do not make container mandatory.

---

# 104. Bare-Metal Service

Recommended for simple gateway deployments.

---

# 105. Immutable Image

For managed appliances:

```text
A/B OS image
```

can improve update safety.

---

# 106. Update Architecture

Need:

```text
signed artifacts
version checks
rollback
health confirmation
```

---

# 107. Application-Only Update

Simpler:

```text
download signed comm-node binary/package
verify
replace
restart
```

---

# 108. Full-System Update

For appliance:

```text
A/B partitions
```

or immutable image framework.

---

# 109. Update Trust

Artifact must be signed by trusted release key.

TLS alone is not enough for offline/cached update verification.

---

# 110. Update Manifest

```rust
pub struct UpdateManifest {
    pub version: Version,
    pub target: TargetTriple,
    pub digest: Digest,
    pub min_schema: u32,
    pub signature: Signature,
}
```

---

# 111. Rollback

If new version fails health check:

```text
restore previous known-good
```

---

# 112. Schema Compatibility

Rollback must respect:

```text
min reader/writer DB versions
```

from Part 09.

---

# 113. Offline Update

Support:

```text
USB package
LAN package
preloaded image
```

for disconnected sites.

Still verify signature.

---

# 114. Staged Fleet Updates

Organization:

```text
canary nodes
small batch
full rollout
```

---

# 115. Update During Emergency

Critical emergency mode may defer non-security update.

Security-critical update policy may override.

---

# 116. Backup

Important node backup includes:

```text
configuration
identity state
event DB
organization policy
```

DTN/cache may be optional.

---

# 117. Identity Backup

If keys are hardware-bound:

```text
backup may not restore same identity
```

Document replacement/re-enrollment process.

---

# 118. Restore

Restore should run in maintenance mode.

Then Part 09 reconciliation.

---

# 119. Factory Reset

Securely removes:

```text
identity
tokens
config
user/tenant data
```

Cache/DTN wiped.

---

# 120. Decommission

Admin should revoke node identity before disposal when possible.

---

# 121. Remote Decommission

If node online:

```text
signed revoke
wipe policy
```

But physical wipe still preferred for sensitive deployments.

---

# 122. Security Hardening

Recommendations:

```text
no password SSH by default
firewall
least privilege
signed updates
secure boot where available
TPM where useful
read-only root
restricted admin
```

---

# 123. Secure Boot

Optional platform-specific feature.

Useful in managed appliances.

---

# 124. Measured Boot

TPM-backed attestation may be added later for enterprise.

Not necessary for v1.

---

# 125. Firewall

Expose only required services.

Prefer outbound/overlay connectivity where possible.

---

# 126. mDNS/LAN Exposure

Nearby discovery should advertise minimal ephemeral info.

---

# 127. Bluetooth Exposure

Same Part 14 privacy rules.

---

# 128. Unknown Peer Limits

Embedded node is always-on and attractive to attackers.

Use stricter:

```text
connection rate
bundle rate
handshake rate
file offers
```

for unknown peers.

---

# 129. DDoS

If public Internet-facing:

```text
provider/network protections
```

may be needed.

An SBC cannot absorb serious volumetric DDoS.

---

# 130. Public Relay Role

Do not expose tiny home SBC as unrestricted public relay by default.

Use authentication/allowlist/quotas.

---

# 131. Local Community Role

Safer default:

```text
known organization/community
```

rather than Internet-open.

---

# 132. Emergency Public Intake

If accepting unknown SOS:

```text
strict quota
small payload only
no large file
```

---

# 133. Logging

Use journald or bounded rotating logs.

Do not fill flash.

---

# 134. Log Level

Default:

```text
info/warn
```

with low volume.

Debug temporarily enabled via admin.

---

# 135. Diagnostics Retention

Keep small bounded history.

Part 18 diagnostics are cache, not authoritative.

---

# 136. Metrics

Useful:

```text
uptime
CPU
memory
temperature
storage
connections
DTN bundles
gateway state
relay usage
```

---

# 137. Prometheus/OpenTelemetry

Optional for organization deployments.

Disable on tiny personal nodes if unnecessary.

---

# 138. Local Health

```text
comm-node status
comm-node doctor
```

should work offline.

---

# 139. Remote Health

Admin API can query:

```text
Ready
Degraded
ReadOnly
Fatal
```

---

# 140. LED Status

Optional hardware adapter can drive:

```text
power
network
emergency
fault
```

LEDs.

Keep hardware GPIO logic separate.

---

# 141. Physical Button

Optional:

```text
pairing mode
factory reset
SOS gateway mode
```

Requires debouncing and deliberate security semantics.

---

# 142. GPIO Adapter

Separate crate:

```text
comm-platform-gpio
```

not core.

---

# 143. Displayless Pairing

Terminal QR over:

```text
SSH
serial console
small OLED
```

possible.

---

# 144. Serial Console

Useful recovery channel for field nodes.

---

# 145. Boot Recovery

If normal runtime fails:

```text
maintenance/recovery service
```

may expose local admin diagnostics.

---

# 146. Safe Mode

Example triggers:

```text
3 failed boots
migration failure
corrupt config
```

Safe mode:

```text
network restricted
read-only diagnostics
admin repair
```

---

# 147. Config

RON config example:

```ron
(
    version: 1,
    role: DtnRelay,
    resource_profile: Small,
    storage: (
        dtn_bytes: 1073741824,
        cache_bytes: 2147483648,
    ),
)
```

---

# 148. Secrets

Never embed in RON:

```text
private keys
admin passwords
API secrets
```

---

# 149. Environment/Secret Files

For simple deployments:

```text
root-readable secret file
```

or system secret manager.

---

# 150. Config Reload

Reload safe fields:

```text
limits
relay pools
logging
DTN quota
```

Restart for:

```text
data directory
identity backend
```

---

# 151. Multi-Tenant Edge Node

Optional.

Requires:

```text
tenant namespaces
per-tenant quota
separate audit
```

---

# 152. Tenant Isolation

One tenant cannot consume all:

```text
DTN storage
file cache
connections
```

---

# 153. Single-Tenant First

Recommended initial production scope:

```text
one organization/profile per node
```

Simpler and safer.

---

# 154. File Cache Semantics

Cache is content-addressed encrypted blob data.

Eviction safe if:

```text
not authoritative
```

---

# 155. Cache Admission

Only cache blobs according to:

```text
policy
size
popularity
tenant
storage pressure
```

---

# 156. Cache Privacy

Node should not need plaintext file names or content.

---

# 157. DTN Store Semantics

Bundle store is encrypted application payload.

Node may inspect only routing metadata required by protocol.

---

# 158. Gateway Queue

Pending Internet forwarding should be durable.

On WAN recovery:

```text
resume by priority
```

---

# 159. Internet Flapping

Use hysteresis.

Do not repeatedly drain/refill between offline/online state for every short WAN glitch.

---

# 160. Connectivity State Machine

```text
Offline
 ↓
LocalOnly
 ↓
InternetProbing
 ↓
InternetAvailable
 ↓
Degraded
```

---

# 161. WAN Probe

Prefer actual Iroh/relay reachability over generic ping.

---

# 162. DNS Failure

Node may still:

```text
serve LAN
BLE
DTN
```

---

# 163. Local-First Invariant

No central control service is required for:

```text
boot
local communication
DTN
emergency relay
```

---

# 164. Cloud-Optional Management

Organization control plane can improve administration, but node continues with last-valid signed policy offline.

---

# 165. Signed Policy Cache

Persist:

```text
last valid config
version
expiry/grace
```

---

# 166. Policy Rollback Protection

Do not accept stale organization policy without explicit recovery rule.

---

# 167. Emergency Mode

Part 17 can alter:

```text
storage reserve
scan rate
routing
DTN replication
```

---

# 168. Emergency Node Boot

Emergency-profile node should reach usable local state quickly.

Prioritize:

```text
identity
DTN
BLE/LAN
critical queues
```

before secondary diagnostics/indexing.

---

# 169. Battery-Powered Field Node

Part 13 profile:

```text
Saver
Emergency
```

with:

```text
scan duty cycling
wake windows
reduced CPU
```

---

# 170. Solar-Powered Node

Optional power adapter can expose:

```text
charging
battery class
```

Same policy engine applies.

---

# 171. Sleep/Wake

Some embedded deployments may suspend between wake windows.

Durable state must survive.

---

# 172. Wake Schedule

Possible:

```text
wake every 5 min
scan/forward
sleep
```

for extreme low-power nodes.

This is deployment-specific.

---

# 173. Real-Time Clock Alarm

Could be used for wake scheduling on supported hardware.

Not core requirement.

---

# 174. Hardware Watchdog

Enable on field appliances where available.

If daemon/system hangs:

```text
hardware resets device
```

---

# 175. Watchdog Safety

Repeated reboot loop should trigger:

```text
safe mode
```

rather than endless destructive restart.

---

# 176. Boot Counter

Persist/observe:

```text
recent failed boots
```

carefully to avoid flash wear.

---

# 177. Network Namespace

Advanced deployments may isolate comm-node network interfaces.

Not required initially.

---

# 178. Container Networking

If containerized, ensure:

```text
UDP/QUIC
mDNS
BLE host access
LAN broadcast/multicast
```

work correctly.

---

# 179. Host Networking

May be necessary for:

```text
LAN discovery
Bluetooth
```

depending container runtime.

---

# 180. Security Trade-Off

Host networking reduces isolation.

Bare-metal service may be simpler/safer for tiny nodes.

---

# 181. Package Integrity

Verify:

```text
hash
signature
target architecture
```

before install.

---

# 182. SBOM

Production releases should include:

```text
SBOM
license inventory
dependency versions
```

especially enterprise/field deployments.

---

# 183. Reproducible Builds

Aim for reproducible/traceable builds.

Pin:

```text
Rust toolchain
Cargo.lock
build environment
```

---

# 184. Cross-Platform ABI

Embedded nodes can expose Part 19 C ABI if another local application embeds runtime.

But default headless node can simply run the daemon binary.

---

# 185. Plugin Policy

Part 24 plugins on embedded nodes should be heavily restricted.

Prefer:

```text
no plugins
or signed allowlisted plugins
```

for field appliances.

---

# 186. Third-Party Protocol Extensions

Part 22 may be enabled if resource profile supports.

Each extension receives quota.

---

# 187. WASM Components

Part 21 may later provide sandboxed extension logic.

Potentially attractive on embedded nodes if runtime overhead is acceptable.

---

# 188. OTA Update Scheduler

Update only when:

```text
not in critical emergency transmission
sufficient storage
sufficient power
```

unless update is emergency security patch.

---

# 189. Pre-Update Checks

```text
signature valid
disk space
schema compatibility
backup/checkpoint
power stable
```

---

# 190. Post-Update Health

After restart:

```text
recovery
health checks
network
DTN
admin
```

Then mark version good.

---

# 191. Canary

In a fleet:

```text
1–5% nodes
```

first.

---

# 192. Fleet Inventory

Track:

```text
NodeId
hardware model
software version
role
health
```

No user message contents.

---

# 193. Fleet Management

Optional organization tooling.

Node core remains usable without it.

---

# 194. Security Incident

If node compromised:

```text
revoke identity
remove config trust
rebuild image
re-enroll
```

Do not try to trust a compromised installation after unknown modification.

---

# 195. Physical Theft

Assume attacker can access storage.

Mitigations:

```text
disk encryption
TPM
limited plaintext
E2EE bundles
key revocation
```

---

# 196. Full-Disk Encryption

Possible:

```text
LUKS
```

on managed devices.

Boot-key provisioning may be operationally challenging.

---

# 197. Encrypted Application Data

Even without full-disk encryption, user payload should remain encrypted end-to-end.

---

# 198. Device Revocation

Part 02 revocation immediately stops node being trusted once state propagates.

---

# 199. Emergency Offline Revocation

In fully disconnected field deployment, revocation propagation may be delayed.

Use:

```text
short-lived authority credentials
local revocation lists
physical re-provisioning
```

where necessary.

---

# 200. Testing Strategy

Required categories:

```text
unit
integration
QEMU
real hardware
power-cut
thermal
storage-full
network partition
BLE
LAN
systemd
upgrade/rollback
```

---

# 201. Hardware Matrix Testing

At minimum test:

```text
one Raspberry Pi-class ARM64
one low-memory ARM board
one x86_64 mini-PC
```

if those are official targets.

---

# 202. Long Soak

Run:

```text
7–30 days
```

with:

```text
DTN
network flaps
file transfers
restarts
```

Track:

```text
memory
FDs
disk growth
temperature
```

---

# 203. Power-Cut Test

Perform repeated ungraceful cuts.

Expected:

```text
no logical corruption
recovery succeeds
```

---

# 204. SD Wear Test

Simulate long diagnostic/DTN workload.

Measure write amplification.

---

# 205. Storage-Full Test

Fill storage.

Expected:

```text
cache eviction
bulk intake stops
critical reserve preserved
```

---

# 206. Network Partition Test

WAN unavailable for hours/days.

Expected:

```text
local/DTN continues
```

Then WAN returns:

```text
gateway drains queue by priority
```

---

# 207. Thermal Test

Artificial CPU/network load.

Expected:

```text
throttle
no crash
```

---

# 208. Bluetooth Adapter Removal

Unplug USB dongle.

Expected:

```text
BLE degraded
other paths continue
```

---

# 209. Wi-Fi Loss

Expected:

```text
Ethernet/Iroh/DTN fallback
```

---

# 210. Upgrade Test

Install new version.

Expected:

```text
schema migration
health
```

Rollback test too.

---

# 211. Corrupt Config Test

Expected:

```text
safe mode or clear failure
```

No destructive reset.

---

# 212. Security Tests

Attempt:

```text
unauthorized admin
fake enrollment
stale signed config
unknown-peer flood
```

---

# 213. Fuzzing

Part 10 fuzz:

```text
embedded config
admin protocol
enrollment
DTN metadata
diagnostic export
```

---

# 214. Performance Benchmarks

Measure:

```text
idle RAM
idle wakeups
DTN forwarding
file throughput
hash throughput
CPU usage
temperature
```

---

# 215. Acceptance Targets

Define per hardware profile:

```text
max idle RAM
max idle CPU
max boot-to-ready
max storage growth
```

after real benchmarking.

---

# 216. Suggested Workspace

```text
apps/
└── node/
    ├── src/
    │   ├── main.rs
    │   ├── config.rs
    │   ├── service.rs
    │   └── maintenance.rs
    └── Cargo.toml

crates/
├── comm-runtime/
├── comm-dtn/
├── comm-routing/
├── comm-proximity-linux/
├── comm-admin/
├── comm-power-linux/
├── comm-secure-store-tpm/
├── comm-update/
└── comm-platform-gpio/

deploy/
├── systemd/
├── deb/
├── container/
├── immutable-image/
└── examples/

hardware/
├── raspberry-pi/
├── generic-arm64/
└── x86-edge/
```

---

# 217. `comm-update`

Responsibilities:

```text
signed manifest
artifact verification
compatibility
staging
rollback marker
health confirmation
```

---

# 218. `comm-power-linux`

Reads:

```text
thermal
battery/UPS where available
power source
```

and maps to Part 13.

---

# 219. `comm-secure-store-tpm`

Optional.

Responsibilities:

```text
TPM key handles
sign
seal
attestation later
```

---

# 220. `comm-platform-gpio`

Optional.

Responsibilities:

```text
LED
button
watchdog hooks
```

No communication policy.

---

# 221. CLI

Examples:

```text
comm-node status
comm-node doctor
comm-node peers
comm-node dtn
comm-node storage
comm-node update
comm-node backup
comm-node enroll
```

---

# 222. First-Boot CLI

```text
comm-node enroll --show-qr
```

---

# 223. Local Maintenance

```text
comm-node maintenance enter
comm-node backup create
comm-node verify storage
```

---

# 224. API Boundary

Embedded apps can either:

```text
run comm-node daemon
```

or:

```text
embed comm-runtime directly
```

Use the daemon by default for appliance-style deployment.

---

# 225. Initial Production Scope

Implement first:

```text
ARM64 + x86_64 Linux
headless comm-node
systemd service
small/standard resource profiles
Iroh
LAN
BLE
DTN
gateway mode
terminal QR enrollment
signed config
bounded storage
crash recovery
local admin CLI
diagnostics
signed binary updates
```

Then:

```text
TPM
UPS integration
A/B updates
Wi-Fi AP helper
multi-tenant mode
```

Defer initially:

```text
full custom Linux distribution
complex cluster orchestration
UWB
arbitrary plugin ecosystem
```

---

# 226. Implementation Phases

## Phase 1 — Headless Binary

```text
comm-node
runtime
config
systemd
```

## Phase 2 — Embedded Profiles

```text
memory
storage
connections
CPU pools
```

## Phase 3 — Local Networking

```text
LAN
BLE
proximity
```

## Phase 4 — DTN Gateway

```text
store-carry-forward
Internet bridge
```

## Phase 5 — Provisioning

```text
terminal QR
signed config
admin roles
```

## Phase 6 — Reliability

```text
power-loss recovery
read-only root support
watchdog
safe mode
```

## Phase 7 — Updates

```text
signed artifacts
rollback
fleet canary
```

## Phase 8 — Hardware Hardening

```text
real boards
thermal
storage wear
long soak
power cut
```

---

# 227. Definition of Done

Part 20 is complete when:

- the runtime compiles without Dioxus
- ARM64 and x86_64 Linux builds exist
- headless node boots without Internet
- resource usage is bounded for embedded profiles
- DTN relay can operate for long periods
- local BLE/LAN communication works without WAN
- Internet gateway bridging resumes after partition
- file/DTN state survives hard power loss
- storage-full mode preserves critical reserve
- node can be provisioned securely with terminal QR
- node private key is generated locally
- systemd supervision/watchdog works
- remote admin is authenticated and explicit
- no public admin shell exists by default
- signed update verification and rollback exist
- diagnostics work offline
- low-memory/thermal/power profiles degrade gracefully
- real hardware, power-cut, storage-full, network-partition, and soak tests exist

---

# 228. Relationship to Earlier Parts

Part 20 builds on:

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
```

It prepares:

```text
21 — WASM-Compatible Components
22 — Third-Party Protocol Extensions
23 — External Interoperability Suite
24 — Plugin / Module Ecosystem
```

---

# 229. Final Architecture

```text
                   EMBEDDED LINUX NODE
 ┌─────────────────────────────────────────────────────────┐
 │                                                         │
 │                     comm-node                           │
 │                                                         │
 │  Identity   Routing   DTN   Files   Emergency           │
 │      │         │       │      │        │                │
 │      └─────────┴───────┴──────┴────────┘                │
 │                         │                               │
 │                  Runtime Supervisor                     │
 │                         │                               │
 │   ┌───────────────┬─────┴─────┬───────────────────┐    │
 │   │               │           │                   │    │
 │ Ethernet         Wi-Fi       BLE                Iroh   │
 │   │               │           │                   │    │
 │   └───────────────┴───────────┴───────────────────┘    │
 │                         │                               │
 │               Local / Internet / DTN                    │
 │                                                         │
 │ systemd + watchdog + signed updates + diagnostics       │
 └─────────────────────────────────────────────────────────┘
```

Disaster gateway example:

```text
Phones with no Internet
      │
     BLE
      │
      ▼
Embedded Linux Gateway
      │
      ├── local DTN store
      │
      ├── Wi-Fi/LAN
      │
      └── intermittent cellular/Iroh
                    │
                    ▼
             Remote destination
```

---

# 230. Final Principle

The embedded Linux node should behave like a reliable appliance, not like a desktop app with the screen removed.

It should:

```text
boot without Internet
run for weeks
survive hard power cuts
stay within fixed memory/storage limits
relay encrypted DTN traffic
bridge local devices to Internet when available
degrade safely when storage, battery, or temperature is constrained
update securely
recover automatically
```

A Raspberry Pi-class node should therefore be able to sit in a:

```text
home
school
shelter
vehicle
community center
field site
```

and quietly provide:

```text
local connectivity
offline relay
Internet bridging
emergency forwarding
file assistance
```

without requiring cloud availability or a graphical interface.

That is the role of Part 20: turn the reusable communication platform into a dependable edge appliance for ordinary local-first use and severe disconnected environments alike.
