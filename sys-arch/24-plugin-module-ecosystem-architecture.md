# Part 24 — Plugin / Module Ecosystem Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 24 of 24  
**Primary language:** Rust  
**Primary goals:** production-grade plugin/module ecosystem, safe packaging and distribution, publisher trust, install/update/remove lifecycle, dependency management, permission governance, sandbox selection, offline installation, enterprise policy, registry/catalog support, compatibility certification, rollback, and long-term ecosystem stability

---

# 1. Purpose

The platform now supports:

```text
third-party protocol extensions
WASM-compatible components
C ABI / FFI
headless runtimes
external interoperability testing
```

Part 24 turns those pieces into a usable ecosystem.

The goal is to allow external developers and organizations to distribute reusable functionality such as:

```text
protocol extensions
workflow modules
ERP integrations
IoT modules
document processors
DTN policies
diagnostic tools
notification modules
custom synchronization
organization-specific features
```

without requiring every consumer to fork the platform.

The core rule is:

> **A plugin may extend the product, but it may never gain more authority than the host explicitly grants.**

---

# 2. Ecosystem Layers

```text
Developer
   ↓
Plugin SDK
   ↓
Plugin Package
   ↓
Signing / Validation
   ↓
Registry / Catalog / Offline Bundle
   ↓
Install Policy
   ↓
Sandbox / Native Host
   ↓
Plugin Runtime
   ↓
Core Capability APIs
```

---

# 3. Plugin vs Module vs Protocol Extension

Use precise terminology.

## Module

A reusable package implementing local functionality.

Examples:

```text
document processor
routing policy
notification formatter
ERP adapter
```

## Protocol Extension

Adds wire-level peer-to-peer semantics.

Defined by Part 21.

## Plugin

A distributable ecosystem package that may contain:

```text
one or more modules
one or more protocol extensions
optional UI metadata
schemas
assets
conformance vectors
```

Part 24 governs packaging/distribution/trust.

---

# 4. Package Contents

Recommended package layout:

```text
plugin-package/
├── manifest.ron
├── component.wasm
├── native/
│   ├── linux-x86_64/
│   ├── linux-aarch64/
│   ├── windows-x86_64/
│   └── macos-aarch64/
├── wit/
├── schemas/
├── assets/
├── docs/
├── conformance/
├── sbom/
├── LICENSE
└── signature/
```

Not every package needs every directory.

---

# 5. Package Types

```rust
pub enum PluginPackageKind {
    WasmOnly,
    NativeTrusted,
    Hybrid,
    MetadataOnly,
    ProtocolSpecOnly,
}
```

---

# 6. WASM-Only Package

Preferred for third-party logic.

Benefits:

```text
sandboxing
portability
resource limits
cross-platform reuse
```

---

# 7. Native Trusted Package

Allowed only for:

```text
built-in
organization-trusted
security-reviewed
platform-specific integration
```

because native code can escape memory isolation.

---

# 8. Hybrid Package

Example:

```text
portable WASM core
+
small native platform adapter
```

Useful for:

```text
device APIs
hardware
special OS integration
```

Native portion must have separate permission/trust review.

---

# 9. Manifest

```rust
pub struct PluginManifest {
    pub id: PluginId,
    pub name: String,
    pub version: Version,
    pub publisher: PublisherId,
    pub kind: PluginPackageKind,
    pub sdk_requirement: VersionReq,
    pub permissions: Vec<PluginPermission>,
    pub components: Vec<ComponentDescriptor>,
    pub extensions: Vec<ExtensionDescriptor>,
    pub dependencies: Vec<PluginDependency>,
    pub resource_profile: PluginResourceProfile,
    pub conformance: Option<ConformanceDescriptor>,
}
```

---

# 10. Plugin ID

Use globally unique namespace:

```text
org.example.whiteboard
com.vendor.school-erp
dev.irshad.file-tools
```

Represent:

```rust
pub struct PluginId(String);
```

---

# 11. Reserved Namespaces

Host reserves:

```text
comm.*
core.*
system.*
official.*
```

unless signed by platform authority.

---

# 12. Publisher Identity

Each publisher has cryptographic identity.

```rust
pub struct PublisherId([u8; 32]);
```

Package signature binds:

```text
plugin ID
version
manifest
payload digest
publisher
```

---

# 13. Publisher Trust

Possible trust classes:

```rust
pub enum PublisherTrust {
    Platform,
    Organization,
    VerifiedThirdParty,
    Unverified,
    Revoked,
}
```

---

# 14. Signing Is Not Permission

A valid signature means:

```text
package came from the claimed publisher
package was not modified
```

It does not mean:

```text
safe
approved
unrestricted
```

Permissions still require policy.

---

# 15. Package Digest

Every package gets canonical digest.

Use for:

```text
cache
verification
pinning
rollback
reproducibility
```

---

# 16. Package Canonicalization

Digest should cover a canonical archive representation or manifest-bound file digest tree.

Avoid signing ambiguous zip metadata.

---

# 17. Signature Envelope

Conceptually:

```rust
pub struct PluginSignatureEnvelope {
    pub publisher: PublisherId,
    pub algorithm: SignatureAlgorithm,
    pub manifest_digest: Digest,
    pub package_digest: Digest,
    pub signature: Signature,
}
```

---

# 18. Multiple Signatures

Enterprise may require:

```text
publisher signature
+
organization approval signature
```

for deployment.

---

# 19. Transparency Log

Optional future ecosystem feature:

```text
append-only publication log
```

for:

```text
publisher changes
package releases
revocations
```

Useful for supply-chain transparency.

---

# 20. Registry / Catalog Architecture

Separate:

```text
package storage
metadata registry
search catalog
trust/revocation metadata
```

---

# 21. Registry Is Optional

The runtime must support:

```text
local file install
offline USB install
organization registry
public registry
```

No central marketplace dependency.

---

# 22. Package Source

```rust
pub enum PluginSource {
    BuiltIn,
    LocalFile,
    OfflineBundle,
    OrganizationRegistry,
    PublicRegistry,
}
```

---

# 23. Offline Installation

Important for:

```text
embedded nodes
schools
disaster deployments
air-gapped organizations
```

Flow:

```text
package copied by USB/LAN
 ↓
signature verification
 ↓
policy evaluation
 ↓
install
```

---

# 24. Registry Metadata

```rust
pub struct RegistryEntry {
    pub plugin_id: PluginId,
    pub versions: Vec<PublishedVersion>,
    pub publisher: PublisherId,
    pub trust: PublisherTrust,
    pub revoked: bool,
}
```

---

# 25. Search Metadata

Catalog can expose:

```text
name
description
categories
permissions
platform support
compatibility
license
publisher
```

No execution required.

---

# 26. Install Pipeline

```text
Fetch/Read Package
      ↓
Digest Verify
      ↓
Signature Verify
      ↓
Manifest Parse
      ↓
Compatibility Check
      ↓
Permission Review
      ↓
Dependency Resolve
      ↓
Conformance Check
      ↓
Sandbox Decision
      ↓
Stage
      ↓
Smoke Test
      ↓
Activate
```

---

# 27. Staging

Never activate directly from downloaded bytes.

Use:

```text
staging area
```

and validate fully first.

---

# 28. Atomic Activation

Installed version becomes active only after:

```text
all validation passes
```

Use atomic registry/state update.

---

# 29. Install Failure

If validation fails:

```text
old version remains active
```

No partial install.

---

# 30. Permission Model

Part 21 permissions form the basis.

Plugin-level permissions may include:

```rust
pub enum PluginPermission {
    ProtocolNetworking,
    ExtensionState,
    SecretState,
    Files,
    Dtn,
    Proximity,
    Notifications,
    DiagnosticsBasic,
    DiagnosticsAdvanced,
    UiIntegration,
    BackgroundExecution,
    ExternalNetwork,
    OrganizationMetadata,
}
```

---

# 31. Permission Groups

User-facing permission groups:

```text
Network with peers
Access files you share
Run in background
Use nearby devices
Store private plugin data
Access advanced diagnostics
```

Do not expose only low-level technical flags.

---

# 32. Permission Review

Installation UI should show:

```text
requested permissions
why needed
resource limits
publisher trust
execution mode
```

---

# 33. Permission Changes on Update

If update adds:

```text
new permission
higher resource ceiling
new native code
```

require re-approval.

---

# 34. Silent Patch Updates

Can auto-update only if:

```text
same permission set
same trust class
compatible execution mode
policy allows
```

---

# 35. Permission Revocation

User/admin can revoke a permission.

Runtime:

```text
stops affected sessions
removes capability
marks plugin degraded/disabled if necessary
```

---

# 36. Dependency Model

Plugins may depend on:

```text
host SDK
other plugins
shared components
```

---

# 37. Dependency Descriptor

```rust
pub struct PluginDependency {
    pub id: PluginId,
    pub version: VersionReq,
    pub optional: bool,
}
```

---

# 38. Dependency Resolution

Use deterministic resolver.

Inputs:

```text
installed versions
registry versions
host compatibility
organization policy
```

---

# 39. Version Selection

Prefer:

```text
highest compatible stable version
```

unless lockfile/policy pins exact version.

---

# 40. Lockfile

For reproducible deployments:

```text
plugins.lock
```

Contains:

```text
plugin ID
exact version
package digest
publisher
source
```

---

# 41. Enterprise Lockfile

Organizations should be able to pin approved plugin set.

---

# 42. Dependency Cycles

Reject cycles initially.

Simpler and safer.

---

# 43. Optional Dependency

Plugin can degrade gracefully if missing.

---

# 44. Shared Dependency Isolation

Avoid one plugin forcing another onto incompatible shared library version.

WASM components naturally help.

Native shared dependencies should be minimized.

---

# 45. Sandbox Selection

Host selects execution based on:

```text
package kind
publisher trust
permissions
platform
policy
```

---

# 46. Sandbox Policy

Example:

```text
unverified third party → WASM only
verified third party → WASM preferred
organization trusted → WASM or out-of-process native
built-in → native allowed
```

---

# 47. No In-Process Untrusted Native Plugins

This should be a hard production rule.

---

# 48. Out-of-Process Native Host

If native code needed:

```text
plugin-host process
+
bounded local IPC
```

---

# 49. Plugin Host Process

Owns:

```text
native plugin library
plugin-specific memory
crash boundary
```

Core daemon retains:

```text
identity
storage
network
policy
```

---

# 50. Plugin Host Crash

Expected:

```text
plugin disabled/restarted
daemon continues
```

---

# 51. WASM Host

Part 22 provides:

```text
memory limits
fuel/deadline
capability imports
state storage
```

Preferred default.

---

# 52. Execution Trust Matrix

```text
Built-in:
native / WASM

Verified:
WASM / out-of-process native

Unverified:
WASM only

Development:
unsigned WASM/native only in dev mode
```

---

# 53. Plugin Lifecycle

```rust
pub enum PluginLifecycleState {
    Installed,
    Disabled,
    Validating,
    Staged,
    Starting,
    Running,
    Degraded,
    Failed,
    Quarantined,
    Updating,
    Removing,
}
```

---

# 54. Startup Order

```text
core ready
 ↓
plugin registry
 ↓
required plugins
 ↓
optional plugins
```

Optional plugin failure must not block core messaging.

---

# 55. Required Product Plugin

A custom product may declare:

```text
required
```

If it fails:

```text
product feature unavailable
```

but daemon health still distinguishes core vs plugin failure.

---

# 56. Plugin Health

```rust
pub struct PluginHealth {
    pub state: PluginLifecycleState,
    pub last_error: Option<PluginErrorCode>,
    pub memory_use: u64,
    pub storage_use: u64,
    pub sessions: u32,
}
```

---

# 57. Quarantine

Repeated:

```text
crash
trap
security violation
resource abuse
```

causes:

```text
Quarantined
```

---

# 58. Auto-Restart

Use bounded restart with backoff.

Do not infinite-loop a broken plugin.

---

# 59. Plugin Update

Flow:

```text
download
verify
stage
compatibility check
permission diff
state migration
smoke test
activate
```

---

# 60. Update Rollback

If:

```text
migration fails
smoke test fails
runtime traps
```

restore previous version when safe.

---

# 61. State Migration

Plugin owns schema version.

Host owns transaction/snapshot boundary.

---

# 62. Rollback Constraint

Do not roll back if old version cannot read migrated schema unless state snapshot restore exists.

---

# 63. State Snapshot

Before risky migration:

```text
checkpoint plugin namespace
```

bounded by storage policy.

---

# 64. Plugin Removal

Flow:

```text
disable
close sessions
cancel tasks
optional export
remove package
retain/delete state per policy
```

---

# 65. State Retention

Choices:

```text
keep
delete
export
```

---

# 66. Secret Retention

Secrets should usually be removed on uninstall unless user/admin explicitly retains.

---

# 67. Orphan State

If plugin temporarily absent:

```text
retain namespaced state
```

until retention policy expires.

---

# 68. Data Portability

Plugins with meaningful user data should expose export/import.

---

# 69. Export Format

Prefer:

```text
versioned archive
RON/JSON metadata
binary assets by digest
```

depending data.

---

# 70. Marketplace Governance

A public marketplace is optional.

If built, it needs governance distinct from runtime.

---

# 71. Marketplace Roles

```text
Publisher
Reviewer
Moderator
Security Team
Registry Operator
User
Organization Admin
```

---

# 72. Marketplace Does Not Grant Runtime Authority

Listing approval means:

```text
package allowed in catalog
```

not:

```text
all permissions auto-granted
```

---

# 73. Review Levels

```rust
pub enum ReviewLevel {
    Unreviewed,
    AutomatedChecks,
    ManualReview,
    SecurityReviewed,
    PlatformVerified,
}
```

---

# 74. Automated Checks

Can include:

```text
manifest validity
signature
SBOM
license
WASM import scan
native binary scan
conformance tests
resource tests
```

---

# 75. Manual Review

Review:

```text
permission justification
privacy
UI claims
security-sensitive behavior
```

---

# 76. Security Review

Required for:

```text
external network
secret storage
organization metadata
emergency integration
native code
remote control
```

---

# 77. Platform Verified

Highest trust for officially maintained plugins.

---

# 78. Rating/Reviews

If marketplace adds user reviews:

```text
do not use ratings as security signal
```

Trust comes from signatures, review, permissions, and policy.

---

# 79. Abuse Reporting

Users/admins can report:

```text
malware
privacy abuse
spam
misleading permissions
crashes
```

---

# 80. Emergency Revocation

Registry can publish signed revocation.

Runtime may disable:

```text
known malicious version
```

according to policy.

---

# 81. Revocation Granularity

Support:

```text
plugin ID
specific version
publisher key
package digest
```

---

# 82. Offline Revocation

Organizations can distribute signed revocation list via:

```text
USB
LAN
DTN
admin policy
```

---

# 83. Publisher Key Rotation

Support:

```text
old key signs handoff to new key
```

or registry-admin recovery process.

---

# 84. Publisher Takeover Protection

Pin plugin ID to publisher identity once trusted.

---

# 85. Namespace Transfer

Requires explicit signed transfer or registry governance.

---

# 86. Enterprise Policy

Organization can define:

```text
allowed plugins
blocked plugins
allowed publishers
max version
min version
permissions
resource ceilings
execution modes
```

---

# 87. Managed Plugin State

```rust
pub enum ManagedPluginPolicy {
    Mandatory,
    Allowed,
    Blocked,
}
```

---

# 88. Mandatory Plugin

Cannot be disabled by normal user if organization policy says mandatory.

But failure remains isolated.

---

# 89. Enterprise Approval

Package may require:

```text
organization signature
```

even if public publisher signature valid.

---

# 90. School/ERP Deployment Example

A school may deploy:

```text
attendance module
payment module
document module
emergency alert module
```

with:

```text
locked versions
approved publisher
limited permissions
```

---

# 91. Embedded Node Policy

Part 20 defaults:

```text
plugins disabled
or
signed allowlisted WASM only
```

for field appliances.

---

# 92. Emergency Node Policy

In disaster nodes:

```text
no arbitrary marketplace install
```

Use:

```text
pre-approved signed package set
```

---

# 93. Mobile Policy

Mobile builds may support only:

```text
built-in
signed WASM
```

initially.

Avoid arbitrary native plugin loading.

---

# 94. Desktop Policy

Desktop may allow richer plugin ecosystem.

Still:

```text
untrusted → sandboxed
```

---

# 95. Server Policy

Server can support tenant-scoped plugins with strong quotas.

---

# 96. Multi-Tenant Plugin Isolation

Namespace:

```text
TenantId
+
PluginId
```

for:

```text
state
secrets
sessions
quota
```

---

# 97. Tenant Plugin Allowlist

Tenant A can use plugin X while tenant B cannot.

---

# 98. Resource Hierarchy

```text
global
  ↓
tenant
  ↓
plugin
  ↓
component
  ↓
session
```

Part 08 enforces.

---

# 99. Plugin Resource Profile

```rust
pub struct PluginResourceProfile {
    pub max_memory_bytes: u64,
    pub max_storage_bytes: u64,
    pub max_cache_bytes: u64,
    pub max_tasks: u32,
    pub max_sessions: u32,
    pub max_network_bps: u64,
}
```

Host clamps values.

---

# 100. Dynamic Resource Reduction

Part 13/17 may temporarily reduce:

```text
plugin CPU
background tasks
network quota
```

during:

```text
battery saver
thermal pressure
emergency mode
```

---

# 101. Plugin Cannot Reserve Critical Capacity

Emergency reserved resources are core-owned.

---

# 102. Background Execution

Requires explicit permission.

Even then subject to:

```text
mobile OS
battery policy
daemon lifecycle
```

---

# 103. External Network Permission

High-risk.

If granted, use:

```text
allowlisted domains
timeouts
request/response size bounds
rate limits
```

---

# 104. No Raw Socket Permission by Default

Prefer semantic host APIs.

---

# 105. File Access

Plugin receives:

```text
BlobId
FileSourceHandle
sandbox path
```

not arbitrary filesystem.

---

# 106. Sandbox Filesystem

Namespace:

```text
plugin-data/<plugin-id>/
```

with path traversal protection.

---

# 107. Secret Storage

Separate API from normal state.

Use opaque secret handles where possible.

---

# 108. Notifications

Plugin can request semantic notification.

Host controls:

```text
priority
rate
presentation
```

---

# 109. Emergency Notification Restriction

Plugin cannot impersonate verified emergency authority.

---

# 110. UI Integration

Optional plugin UI should be declarative or sandboxed where possible.

---

# 111. Dioxus Integration

Possible approaches:

```text
host-rendered schema
predefined component slots
sandboxed web-like declarative UI
trusted native Rust UI module
```

Avoid arbitrary untrusted native Dioxus code in daemon/UI process.

---

# 112. UI Slot Model

Examples:

```text
Settings panel
Conversation action
File action
Diagnostics panel
ERP sidebar
```

---

# 113. Declarative UI Schema

Plugin may provide:

```text
labels
forms
buttons
tables
commands
```

Host renders in Dioxus.

This preserves visual consistency and safety.

---

# 114. Plugin Command Bridge

UI action maps to:

```text
plugin command ID
+
typed payload
```

Runtime validates permission.

---

# 115. No Direct UI Database Access

Plugin UI always talks through plugin/runtime API.

---

# 116. Theme Consistency

Host applies:

```text
colors
typography
spacing
accessibility
```

Plugin should not fully override app chrome.

---

# 117. Accessibility

Plugin UI schema should support:

```text
labels
roles
focus
screen-reader text
```

---

# 118. Localization

Plugin package may include translations.

Use locale-key system.

---

# 119. Assets

Bound:

```text
icon size
image size
asset count
```

No unbounded package assets.

---

# 120. Plugin Protocol Registration

Part 21 registry receives extension descriptors only after plugin activation.

---

# 121. Plugin Disable

Unregisters:

```text
protocol extensions
UI slots
notifications
background tasks
```

atomically where possible.

---

# 122. Capability Negotiation

Peer capabilities include plugin protocol only while plugin is active.

---

# 123. Plugin Upgrade and Active Sessions

Policy options:

```text
drain old sessions
migrate
terminate and reconnect
```

depending protocol.

---

# 124. Session Drain

Preferred for incompatible update.

---

# 125. Hot Reload

Development:

```text
fast
```

Production:

```text
careful staged transition
```

---

# 126. Plugin SDK

Provide:

```text
comm-plugin-sdk
```

on top of Parts 21/22.

---

# 127. SDK Modules

```text
manifest
permissions
state
secrets
protocol
files
dtn
diagnostics
ui-schema
testkit
packaging
```

---

# 128. Plugin Template

Provide generator:

```text
cargo generate
```

or:

```text
comm plugin new
```

---

# 129. Example Plugin

Ship:

```text
hello-plugin
```

then a realistic:

```text
ERP notification plugin
```

---

# 130. Developer CLI

```text
comm plugin new
comm plugin build
comm plugin test
comm plugin pack
comm plugin sign
comm plugin install
comm plugin doctor
```

---

# 131. Build Tool

`comm plugin build` should:

```text
compile WASM/native
validate WIT
collect manifest
run tests
generate SBOM
```

---

# 132. Pack Tool

Creates canonical package.

---

# 133. Sign Tool

Uses publisher key.

Prefer hardware-backed publisher key for serious publishers.

---

# 134. Developer Publisher Key

Can be local file in dev mode.

Production publisher should use stronger key management.

---

# 135. Plugin Doctor

Checks:

```text
manifest
signature
imports
permissions
compatibility
conformance
resource profile
```

---

# 136. Plugin Testkit

Simulates:

```text
peer
state
files
DTN
battery
resource pressure
permission denial
```

---

# 137. Conformance Integration

Part 23 suite can validate plugin protocol.

Package may include:

```text
conformance descriptor
golden vectors
```

---

# 138. Stable Plugin Release Requirements

Recommended:

```text
manifest complete
signature
SBOM
unit tests
resource tests
conformance tests
permission justification
upgrade/migration tests
```

---

# 139. High-Risk Plugin Requirements

Additionally:

```text
security review
publisher verification
manual approval
sandbox mandatory
```

---

# 140. Marketplace Publish Pipeline

```text
upload
 ↓
signature/digest check
 ↓
automated scan
 ↓
conformance
 ↓
policy/review
 ↓
publish
```

---

# 141. Automated Scan

Check:

```text
undeclared imports
native binaries
malformed archive
oversized assets
suspicious permissions
dependency metadata
```

---

# 142. Malware Scanning

Can be additional defense for native packages.

Do not treat scanner result as proof of safety.

---

# 143. SBOM Validation

Ensure declared dependencies match package metadata where possible.

---

# 144. License Metadata

Manifest should include:

```text
license expression
```

for legal/enterprise filtering.

---

# 145. Dependency License Policy

Organizations can block unwanted licenses.

---

# 146. Package Size Limit

Registry/runtime should enforce max package size.

---

# 147. Native Binary Size Limits

Per target.

---

# 148. Asset Bomb Protection

Bound:

```text
compressed size
uncompressed size
file count
nesting
```

to prevent archive bombs.

---

# 149. Path Traversal Protection

Package extractor rejects:

```text
../
absolute paths
symlink escape
```

---

# 150. Canonical Extract Directory

Use:

```text
plugins/<id>/<version>/<digest>/
```

or content-addressed store.

---

# 151. Content-Addressed Plugin Store

Benefits:

```text
dedup
rollback
verification
immutable package storage
```

---

# 152. Active Pointer

Registry maps:

```text
PluginId → active digest/version
```

---

# 153. Rollback

Switch active pointer back after validation.

---

# 154. Garbage Collection

Remove unused old plugin versions after retention period.

Keep at least one rollback version if policy allows.

---

# 155. Update Channels

Plugin can publish:

```text
stable
beta
nightly
```

Host defaults to stable.

---

# 156. Enterprise Channels

Organization can mirror/pin approved versions.

---

# 157. Auto-Update Policy

```rust
pub enum PluginUpdatePolicy {
    Manual,
    SecurityOnly,
    CompatiblePatch,
    CompatibleMinor,
    Managed,
}
```

---

# 158. Security Update

Registry can flag:

```text
security-critical
```

but host policy still decides timing.

---

# 159. Emergency Update Timing

During active SOS/critical flow:

```text
defer noncritical plugin update
```

---

# 160. Update Integrity

Never activate before:

```text
digest + signature verification
```

---

# 161. Registry Compromise Model

Even if catalog server compromised:

```text
publisher signature
lockfile digest
organization approval
```

should limit damage.

---

# 162. Trust Root Separation

Separate:

```text
registry TLS
publisher signatures
organization policy signatures
```

---

# 163. Plugin Revocation Cache

Persist last valid signed revocation state.

---

# 164. Offline Grace

If registry unavailable:

```text
installed plugins continue
```

according to last valid policy.

---

# 165. No Mandatory Cloud Check

Runtime startup must not require public registry.

---

# 166. Plugin Discovery

Catalog search is optional UI.

Manual install remains supported.

---

# 167. Recommendation System

If marketplace later recommends plugins:

```text
do not make opaque rankings security-sensitive
```

---

# 168. Privacy of Marketplace Use

Do not require upload of:

```text
installed plugin list
contact graph
usage history
```

unless user consents.

---

# 169. Anonymous Metrics

Optional aggregate:

```text
install count
crash rate
```

with privacy controls.

---

# 170. Crash Reporting

Plugin crash report includes:

```text
plugin ID/version
trap/error
host version
```

No user payload by default.

---

# 171. Compatibility Matrix

Registry entry may publish:

```text
host SDK versions
platforms
architectures
conformance profile
```

---

# 172. Platform Support

Example:

```text
Linux x86_64
Linux aarch64
Windows
macOS
Android
iOS
WASM host
```

---

# 173. WASM Portability Advantage

A WASM-only plugin can avoid many target-specific binaries.

---

# 174. Native Platform Adapter

Hybrid plugin may include:

```text
Android adapter
Linux adapter
```

only where necessary.

---

# 175. Missing Platform Adapter

Plugin can still load partially if optional.

---

# 176. Required Platform

Manifest can declare:

```text
android only
linux only
```

if truly platform-specific.

---

# 177. Host Compatibility

Before install:

```text
SDK version
WIT version
required capabilities
platform
architecture
```

must match.

---

# 178. Incompatible Plugin

Return:

```text
IncompatibleHost
```

with explanation.

---

# 179. Developer Mode

Explicit setting enables:

```text
unsigned plugins
local native plugins
hot reload
verbose diagnostics
```

---

# 180. Production Mode

Unsigned third-party plugin rejected by default.

---

# 181. Dev Plugin Isolation

Even dev plugin should run sandboxed where possible.

---

# 182. Debug Permission Override

Development override must be visibly marked.

---

# 183. Plugin Registry State

```rust
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub digest: Digest,
    pub source: PluginSource,
    pub state: PluginLifecycleState,
    pub granted_permissions: Vec<PluginPermission>,
}
```

---

# 184. Registry Database

Core-owned.

Plugins cannot edit installation records.

---

# 185. Plugin State Directory

Separate from registry/package store.

---

# 186. Backup

Backup includes:

```text
installed plugin metadata
plugin state
plugin secret references
lockfile
```

Package binaries may be re-fetched if trusted source exists, but offline backup can include them.

---

# 187. Restore

Verify signatures again.

Do not blindly trust restored plugin binaries.

---

# 188. Plugin Secret Restore

Depends on secure-store semantics.

Hardware-bound secrets may require re-provisioning.

---

# 189. Multi-Device Plugin State

Not all plugin state should sync.

Manifest declares:

```rust
pub enum PluginStateScope {
    DeviceLocal,
    AccountSynced,
    OrganizationSynced,
}
```

---

# 190. DeviceLocal

Examples:

```text
cache
local settings
hardware handles
```

---

# 191. AccountSynced

Examples:

```text
workflow preferences
plugin-specific user metadata
```

Sync through explicit extension protocol/events.

---

# 192. OrganizationSynced

Enterprise policy/config only.

---

# 193. Never Sync Secrets Accidentally

Secret state defaults:

```text
device-local
```

unless explicit secure synchronization design exists.

---

# 194. Plugin Event Log

Plugin semantic state can use namespaced Part 04 events.

---

# 195. Plugin Event Schema

Versioned independently.

---

# 196. Plugin Event Migration

Use upcasters/migrations inside namespace.

---

# 197. Plugin Diagnostics

Expose:

```text
health
resource usage
last error
permissions
version
```

through Part 18.

---

# 198. User-Level Diagnostics

Simple:

```text
Plugin failed to start.
```

---

# 199. Advanced Diagnostics

Show:

```text
trap
permission denied
incompatible host
migration failed
```

---

# 200. Network Diagnostics

If plugin protocol fails, diagnostics should distinguish:

```text
plugin unavailable
peer lacks extension
network failure
permission denied
```

---

# 201. Plugin Logging

Rate-limited structured logging.

---

# 202. Plugin Audit

Record:

```text
install
enable
disable
permission grant
permission revoke
update
rollback
uninstall
publisher change
quarantine
```

---

# 203. Audit Security

Admin audit is durable.

Do not let plugin alter its own audit history.

---

# 204. Plugin Security Invariants

1. Untrusted native code is not loaded in-process.
2. Plugins cannot read core private identity keys.
3. Plugins cannot mutate core databases directly.
4. Plugins cannot self-grant emergency authority.
5. Plugin permissions are explicit and revocable.
6. Permission expansion requires approval.
7. Plugin state is namespaced.
8. Plugin resource use is bounded.
9. Plugin failures do not crash core where sandboxing exists.
10. Package signatures are verified before activation.
11. Installed package digest is immutable.
12. Registry compromise alone cannot replace a pinned signed package silently.
13. Plugin update is atomic.
14. Rollback is possible where schema compatibility allows.
15. Optional plugin failure does not block core runtime.
16. Offline deployments do not require marketplace availability.
17. Enterprise policy can override marketplace/user choice.
18. Plugin diagnostics obey privacy redaction.
19. Plugins cannot bypass transport/routing/resource policy.
20. Plugin protocol compatibility is testable through Part 23.

---

# 205. Threat Model

Threats include:

```text
malicious plugin
compromised publisher
compromised registry
dependency confusion
package tampering
permission abuse
resource exhaustion
plugin crash loop
data exfiltration
native-code escape
supply-chain poisoning
```

---

# 206. Dependency Confusion

Use exact:

```text
PluginId
publisher
source policy
digest
```

not name-only resolution.

---

# 207. Source Pinning

Lockfile can pin:

```text
organization registry
public registry
local source
```

---

# 208. Publisher Mismatch

Same PluginId from different publisher:

```text
reject
```

unless explicit transfer.

---

# 209. Package Tampering

Digest/signature mismatch:

```text
reject before install
```

---

# 210. Resource Exhaustion

Part 08 limits:

```text
memory
storage
tasks
streams
logs
callbacks
```

---

# 211. Data Exfiltration

Prevent by:

```text
no ambient network
no arbitrary files
scoped peer IDs
permissioned metadata
```

---

# 212. Native Escape

Out-of-process native host limits blast radius but cannot fully sandbox all OS access without stronger OS sandboxing.

Use:

```text
namespaces
seccomp
AppContainer
sandbox-exec equivalents
```

where practical.

---

# 213. Marketplace Supply Chain

Recommended:

```text
publisher signature
SBOM
conformance
automated scanning
transparency log later
```

---

# 214. Reproducible Plugins

High-trust plugins should publish reproducible build instructions.

---

# 215. Source Availability

Marketplace may distinguish:

```text
open-source
source-available
closed-source
```

This is metadata, not security guarantee.

---

# 216. License Compatibility

Runtime should not enforce license globally unless product/organization chooses.

Enterprise policy may.

---

# 217. Plugin Categories

Examples:

```text
Messaging
Files
ERP
Automation
Diagnostics
Emergency
IoT
Collaboration
Developer Tools
```

---

# 218. Search Facets

Catalog can filter:

```text
WASM-only
open-source
offline-capable
headless-compatible
mobile-compatible
conformance-certified
```

---

# 219. Compatibility Badge

Use exact:

```text
Part 23 suite version
protocol profile
```

not vague "compatible".

---

# 220. Headless Compatibility

Manifest:

```text
headless = true/false
```

A headless plugin must not depend on Dioxus UI.

---

# 221. Offline Capability

Manifest can declare:

```text
offline-capable
DTN-capable
Internet-required
```

---

# 222. Mobile Constraints

Plugin should declare:

```text
background expectations
large runtime requirements
```

Host rejects impossible combinations.

---

# 223. Battery Impact Metadata

Optional:

```text
Low
Moderate
High
```

for user/admin awareness.

---

# 224. Resource Benchmark Metadata

Publisher may provide measured:

```text
memory
CPU
storage
```

Host still enforces real limits.

---

# 225. Plugin Lifecycle API

```rust
pub trait PluginManager {
    async fn install(&self, package: PluginPackage) -> Result<PluginId, PluginError>;
    async fn enable(&self, id: &PluginId) -> Result<(), PluginError>;
    async fn disable(&self, id: &PluginId) -> Result<(), PluginError>;
    async fn update(&self, id: &PluginId, package: PluginPackage) -> Result<(), PluginError>;
    async fn uninstall(&self, id: &PluginId, policy: UninstallPolicy) -> Result<(), PluginError>;
}
```

---

# 226. Plugin Query API

```text
list
inspect
permissions
health
updates
```

---

# 227. Dioxus Plugin Manager UI

Screens:

```text
Installed
Available
Updates
Permissions
Developer
Enterprise Policy
```

---

# 228. Installed Plugin Card

Show:

```text
name
version
publisher
status
permissions summary
update state
```

---

# 229. Permission Detail UI

Explain:

```text
what access means
why plugin requests it
```

---

# 230. Update UI

Show permission diff.

Example:

```text
This update adds:
• Access to nearby devices
```

Require approval.

---

# 231. Quarantine UI

Show:

```text
Plugin disabled after repeated crashes.
```

with:

```text
details
rollback
remove
report
```

---

# 232. Enterprise UI

Managed plugin may show:

```text
Installed by your organization
Cannot be disabled
```

---

# 233. CLI

```text
comm plugin list
comm plugin inspect <id>
comm plugin install <file>
comm plugin enable <id>
comm plugin disable <id>
comm plugin update <id>
comm plugin rollback <id>
comm plugin uninstall <id>
comm plugin doctor <id>
```

---

# 234. Headless Management

Same CLI/admin API works on embedded nodes.

---

# 235. Plugin Registry CLI

Publisher tooling:

```text
comm plugin publish
comm plugin yank
comm plugin revoke
```

if public registry exists.

---

# 236. Yank vs Revoke

## Yank

```text
not recommended for new installs
existing installs may continue
```

## Revoke

```text
security/trust issue
host may disable
```

---

# 237. Version Pinning

User/admin can pin:

```text
1.4.2
```

and disable auto-update.

---

# 238. Security Override

Critical security revocation may override normal pin according to organization/product policy.

---

# 239. Registry Mirroring

Enterprise can mirror approved public plugins.

---

# 240. Air-Gapped Mirror

Provide export/import bundle:

```text
packages
metadata
revocation list
signatures
```

---

# 241. Offline Bundle Manifest

```rust
pub struct OfflinePluginBundle {
    pub created_at: Timestamp,
    pub packages: Vec<PluginPackageRef>,
    pub registry_snapshot: RegistrySnapshot,
    pub signature: Signature,
}
```

---

# 242. Bundle Verification

Verify:

```text
bundle signature
package signatures
digests
```

---

# 243. Developer Experience

Good ecosystem requires:

```text
templates
docs
examples
testkit
local registry
debugger
conformance
```

---

# 244. Local Development Registry

Run:

```text
comm registry dev
```

optional.

Supports:

```text
publish local build
install/update
```

without public network.

---

# 245. Hot Reload Workflow

```text
build
pack
install --dev
reload
inspect logs
```

---

# 246. Plugin Debugger

Can inspect:

```text
events
resource use
state keys metadata
protocol sessions
```

not secrets by default.

---

# 247. Test Scenarios

Plugin testkit supports:

```text
offline
peer missing extension
permission denied
storage full
battery saver
emergency mode
update
rollback
```

---

# 248. Conformance Requirement

Protocol plugin should not publish "stable" without Part 23 tests.

---

# 249. Security Review Checklist

For high-risk plugin:

```text
permissions
data flow
identity usage
external network
state secrets
replay
resource abuse
update path
publisher keys
```

---

# 250. Marketplace Security Incident Flow

```text
report
 ↓
triage
 ↓
reproduce
 ↓
mark version unsafe
 ↓
publish revocation
 ↓
notify affected users/admins
 ↓
fixed release
```

---

# 251. User Notification

Security revocation should clearly state:

```text
plugin disabled because this version is unsafe
```

without panic-inducing vague language.

---

# 252. Plugin Analytics

If enabled, collect only aggregate health metrics with consent/policy.

---

# 253. No Hidden Telemetry

Plugins cannot bypass host telemetry/privacy policy.

---

# 254. Plugin Business Model

The architecture should support:

```text
free
paid
organization-private
open-source
commercial
```

without coupling core runtime to a specific payment marketplace.

---

# 255. License / Payment Separation

Runtime checks:

```text
installation/permission/trust
```

Licensing/payment can be separate entitlement provider.

---

# 256. Offline Entitlements

If paid plugins exist in disconnected deployments, use signed entitlement tokens with expiry/grace policy.

---

# 257. Entitlement Is Not Trust

A paid license does not grant extra security permissions automatically.

---

# 258. Private Organization Plugins

May never appear in public catalog.

Install through:

```text
organization registry
offline bundle
managed policy
```

---

# 259. Plugin Federation

Future multiple registries can coexist.

Runtime policy selects trusted sources.

---

# 260. No Global Registry Monopoly

Important for open ecosystem resilience.

---

# 261. Portability

WASM-only plugins should be preferred for broad cross-platform support.

---

# 262. Native Target Matrix

Native plugin must publish per-target binaries.

Host verifies architecture.

---

# 263. Missing Native Target

Plugin unavailable on that platform unless WASM fallback exists.

---

# 264. Hybrid Fallback

Example:

```text
WASM core works everywhere
native Bluetooth helper only on Android/Linux
```

---

# 265. Build Reproducibility

Plugin package should record:

```text
source commit
toolchain
build flags
WIT versions
```

---

# 266. SBOM

Required for verified/high-risk plugins.

---

# 267. Provenance

Future supply-chain metadata can use signed provenance attestations.

---

# 268. Release Channels

Official host can distinguish:

```text
stable
beta
dev
```

plugin ecosystems too.

---

# 269. Compatibility Freeze

Stable plugin protocol must respect wire version policy from Part 21.

---

# 270. Ecosystem Governance

Define public policy for:

```text
namespace ownership
publisher verification
revocation
appeals
security response
```

if public registry exists.

---

# 271. Namespace Dispute

Registry governance resolves only catalog namespace use.

Protocol IDs remain cryptographically publisher-bound where possible.

---

# 272. Removal From Marketplace

Does not necessarily uninstall existing plugin.

Revocation is separate.

---

# 273. Transparency

Marketplace should expose:

```text
publisher
permissions
review level
source/license
conformance level
```

before install.

---

# 274. No Dark Patterns

Do not hide permissions behind "recommended".

---

# 275. Plugin Trust Decision

Host computes:

```text
publisher trust
package signature
review level
permissions
execution mode
organization policy
```

---

# 276. Trust Decision Result

```rust
pub enum PluginTrustDecision {
    Allow,
    AllowSandboxed,
    RequireApproval,
    Block,
}
```

---

# 277. Install Policy Engine

```rust
pub trait PluginPolicyEngine {
    fn evaluate(
        &self,
        manifest: &PluginManifest,
        source: &PluginSource,
        publisher: &PublisherTrust,
        org_policy: Option<&OrganizationPluginPolicy>,
    ) -> PluginTrustDecision;
}
```

---

# 278. Policy Is Deterministic

Keep install trust decision explainable.

---

# 279. Diagnostic Explanation

Example:

```text
Blocked because:
• unsigned native code
• publisher not trusted
```

---

# 280. Plugin API Versioning

SDK API and plugin package schema are versioned separately.

---

# 281. Manifest Schema Version

```text
manifest_version = 1
```

---

# 282. Unknown Manifest Fields

Ignore only if marked forward-compatible.

Unknown required semantics:

```text
reject
```

---

# 283. Plugin Package Version

Semantic version for plugin release.

---

# 284. Host SDK Requirement

Example:

```text
>= 1.8, < 2.0
```

---

# 285. Protocol Version Requirement

Declared independently.

---

# 286. Compatibility Preflight

Before update:

```text
host compatible?
dependencies compatible?
state migration available?
permissions acceptable?
```

---

# 287. Dependency Update Cascade

Do not automatically upgrade huge dependency graph without showing plan.

---

# 288. Transactional Multi-Plugin Update

For tightly coupled set:

```text
stage all
validate all
activate atomically
```

where needed.

---

# 289. Failure Rollback

Restore previous set.

---

# 290. Plugin Bundle

Organizations can package related plugins:

```text
School ERP bundle
Emergency bundle
Developer bundle
```

---

# 291. Bundle Is Metadata

Do not create hidden permission aggregation.

Each plugin still has individual permissions.

---

# 292. Plugin Composition

Plugins can call each other only through explicit shared service/protocol contracts.

---

# 293. No Ambient Plugin Discovery

A plugin should not enumerate all other plugins unless permission/API allows.

---

# 294. Shared Services

Host may expose approved plugin service registry.

---

# 295. Service Contract

Versioned typed interface.

---

# 296. Service Dependency

Manifest declares it explicitly.

---

# 297. Service Permission

Consumer plugin gets only service methods exposed.

---

# 298. Plugin-to-Plugin Privacy

Do not share state namespaces.

---

# 299. Deadlock Avoidance

Plugin service calls should be async/bounded.

Avoid circular synchronous calls.

---

# 300. Initial Production Scope

Implement first:

```text
PluginId
manifest
package format
package digest/signature
install/enable/disable/uninstall
WASM-only third-party plugins
built-in trusted native plugins
permission review
resource profile
plugin registry
lockfile
offline installation
update/rollback
plugin doctor
CLI
Dioxus plugin manager
Part 23 conformance integration
```

Then:

```text
out-of-process native plugins
organization registry
public registry
publisher verification
revocation
marketplace metadata
declarative UI slots
offline registry bundles
```

Defer initially:

```text
arbitrary in-process native plugins
global marketplace payment system
complex transitive dependency graphs
unrestricted external network
fully dynamic native UI injection
```

---

# 301. Implementation Phases

## Phase 1 — Package & Manifest

```text
PluginId
manifest
canonical archive
digest
```

## Phase 2 — Trust

```text
publisher keys
signatures
verification
```

## Phase 3 — Registry

```text
installed state
activation
lockfile
```

## Phase 4 — Permissions

```text
review
grant
revoke
policy
```

## Phase 5 — Execution

```text
WASM host
trusted native
lifecycle
health
```

## Phase 6 — Updates

```text
stage
migration
rollback
```

## Phase 7 — UX / Tooling

```text
Dioxus manager
CLI
developer SDK
templates
doctor
```

## Phase 8 — Distribution

```text
offline bundles
organization registry
public registry
```

## Phase 9 — Governance

```text
publisher verification
revocation
review levels
security response
```

## Phase 10 — Hardening

```text
fuzz
package bombs
dependency confusion
crash loops
permission escalation
rollback
registry compromise
```

---

# 302. Definition of Done

Part 24 is complete when:

- plugins have globally unique IDs
- packages are canonical and digest-addressed
- publisher signatures are verified before activation
- install/update activation is atomic
- permissions are explicit and revocable
- permission expansion requires approval
- untrusted native code is never loaded in-process
- WASM is the default third-party execution model
- plugin state/secrets are namespaced and quota-limited
- dependency resolution is deterministic
- lockfiles support reproducible deployments
- offline installation works without public registry
- enterprise allow/block/mandatory policies exist
- updates can roll back safely where schema permits
- revocation can target plugin/version/publisher/digest
- marketplace/catalog availability is not required for runtime startup
- plugin diagnostics integrate with Part 18
- plugin protocol compatibility integrates with Part 23
- plugin failures do not corrupt or block core runtime
- CLI and Dioxus management exist
- package, signature, permission, dependency, crash-loop, revocation, and registry-compromise tests exist

---

# 303. Relationship to Earlier Parts

Part 24 is the ecosystem layer above all prior architecture.

It directly depends on:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
03 — Routing Policy
04 — Offline Event Log
05 — File / Blob System
06 — DTN
07 — Capability Negotiation
08 — Resource Limits
09 — Crash Recovery
10 — Fuzz / Test Suite
11 — Relay Infrastructure
12 — Multipath
13 — Battery Scheduling
14 — Proximity
15 — QR/NFC Bootstrap
16 — Daemon / Headless
17 — Emergency Priority
18 — Diagnostics
19 — C ABI / FFI
20 — Embedded Linux Node
21 — Third-Party Protocol Extensions
22 — WASM-Compatible Components
23 — External Interoperability Suite
```

---

# 304. Final Architecture

```text
                       PLUGIN DEVELOPER
                              │
                       comm-plugin-sdk
                              │
                        Plugin Package
                              │
                 ┌────────────┼────────────┐
                 │            │            │
              Signature    Manifest     Conformance
                 │            │            │
                 └────────────┼────────────┘
                              │
                        Registry / File
                              │
                        Install Pipeline
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
   Permission Check      Trust Check        Compatibility
         │                    │                    │
         └────────────────────┼────────────────────┘
                              │
                           Stage
                              │
                         Smoke Test
                              │
                          Activate
                              │
              ┌───────────────┼────────────────┐
              │               │                │
         WASM Sandbox   Native Host Proc   Built-In
              │               │                │
              └───────────────┼────────────────┘
                              │
                        Core Capability APIs
                              │
                        Communication Runtime
```

Offline deployment:

```text
Signed Plugin Bundle
       │
       ▼
USB / LAN / DTN
       │
       ▼
Embedded / Enterprise Node
       │
       ▼
Verify → Install → Activate
```

Enterprise deployment:

```text
Organization Policy
        │
        ├── allowed publishers
        ├── pinned versions
        ├── permissions
        └── mandatory plugins
                 │
                 ▼
          Managed Runtime
```

---

# 305. Final Principle

A mature plugin ecosystem should make this possible:

```text
A school installs an attendance plugin.
A hospital installs a responder workflow plugin.
A developer installs a whiteboard protocol plugin.
A Raspberry Pi emergency node installs only a signed DTN policy module.
A Flutter product embeds the same plugin-backed features through the SDK.
```

But none of those plugins should be able to:

```text
read identity private keys
claim emergency authority
consume unlimited memory
open arbitrary sockets
mutate core databases
silently expand permissions
replace trusted packages through registry compromise
crash the daemon through untrusted native code
```

The ecosystem therefore needs:

```text
packaging
+
signatures
+
permissions
+
sandboxing
+
resource limits
+
compatibility testing
+
rollback
+
governance
```

That is the purpose of Part 24: turn the communication platform into an extensible ecosystem without sacrificing the reliability, privacy, portability, and security established by Parts 1–23.
