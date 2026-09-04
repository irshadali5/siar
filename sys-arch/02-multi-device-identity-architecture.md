# Part 02 — Multi-Device Identity Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 02 of 24  
**Primary language:** Rust  
**Primary goals:** multi-device accounts, cryptographic separation, secure linking, revocation, recovery, portability, reuse across messaging/files/emergency/ERP products

---

# 1. Purpose

The communication platform must not model a person, organization, or application identity as a single device key.

A modern user may have:

```text
phone
laptop
desktop
tablet
secondary phone
headless node
```

and all of them may belong to the same logical identity.

The architecture must therefore distinguish:

```text
Account Identity
Device Identity
Transport Identity
Session Identity
Application Profile
```

These are related, but they are not the same thing.

The multi-device identity system must support:

- secure device linking
- independent device keys
- multiple active devices
- device-specific transport addresses
- device revocation
- account recovery
- key rotation
- cross-device synchronization
- sender-device attribution
- per-device trust
- per-device capabilities
- offline pairing
- QR/NFC bootstrap
- enterprise/organization use
- application namespaces
- future headless nodes

The identity layer must remain reusable outside the messenger.

---

# 2. Fundamental Identity Model

The central model is:

```text
Logical Identity
      │
      ├── Device A
      │     ├── identity key
      │     ├── transport identity
      │     └── session state
      │
      ├── Device B
      │     ├── identity key
      │     ├── transport identity
      │     └── session state
      │
      └── Device C
            ├── identity key
            ├── transport identity
            └── session state
```

Never model:

```text
one user = one long-lived private key
```

That creates weak revocation and recovery semantics.

---

# 3. Identity Layers

The architecture should expose at least five layers.

## 3.1 Account Identity

Represents the long-lived logical principal.

Examples:

```text
person
organization
service account
emergency authority
business unit
```

## 3.2 Device Identity

Represents one independently revocable device.

## 3.3 Transport Identity

Represents transport-specific addressing/cryptographic identity.

Example:

```text
Iroh EndpointId
```

## 3.4 Session Identity

Represents ephemeral authenticated communication state.

## 3.5 Application Profile

Represents non-cryptographic user-facing data:

```text
display name
avatar
status
organization role
```

Do not collapse these layers.

---

# 4. Strong Type Model

Recommended types:

```rust
pub struct AccountId([u8; 32]);
pub struct DeviceId([u8; 16]);
pub struct IdentityKeyId([u8; 32]);
pub struct TransportIdentityId([u8; 32]);
pub struct SessionId([u8; 16]);
pub struct DeviceCertificateId([u8; 32]);
```

Do not use:

```rust
String
```

for every identifier.

Strong types prevent accidental cross-use.

---

# 5. Account Identity

The account identity is the durable logical principal.

It should be stable across device replacement.

Conceptually:

```rust
pub struct AccountIdentity {
    pub account_id: AccountId,
    pub root_public_key: PublicKey,
    pub generation: u64,
}
```

The root private key should not necessarily be online continuously.

For stronger security, consider:

```text
root identity key
    ↓
signs device certificates
```

rather than:

```text
root key used for every message/session
```

---

# 6. Root Key Strategy

Recommended architecture:

```text
Root Identity Key
        │
        ├── signs Device Certificate A
        ├── signs Device Certificate B
        └── signs Device Certificate C
```

The root key is used rarely.

Benefits:

- easier device revocation
- easier auditing
- less exposure
- better recovery
- device independence

The root key may be:

```text
hardware-protected
recovery-derived
offline-backed-up
```

depending on product mode.

---

# 7. Device Identity

Each device gets its own keypair.

```rust
pub struct DeviceIdentity {
    pub account_id: AccountId,
    pub device_id: DeviceId,
    pub signing_public_key: PublicKey,
    pub device_certificate: DeviceCertificate,
}
```

The device private key:

- never leaves the device under normal operation
- is stored in secure storage
- can be revoked independently
- signs device-level operations

---

# 8. Device Certificate

A device certificate binds:

```text
Account
   ↓
Device
   ↓
Device public key
```

Conceptually:

```rust
pub struct DeviceCertificate {
    pub account_id: AccountId,
    pub device_id: DeviceId,
    pub device_public_key: PublicKey,
    pub issued_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub capabilities: DeviceCapabilitySet,
    pub generation: u64,
    pub signature: Signature,
}
```

The signature is made by the account root or authorized linking authority.

---

# 9. Device Certificate Semantics

The certificate proves:

```text
This device key belongs to this logical account.
```

It does not automatically prove:

```text
this device is currently trusted
this device is not revoked
this device may perform every operation
```

Revocation and authorization are separate checks.

---

# 10. Transport Identity

Transport identities must not become account identity.

For Iroh:

```text
EndpointId
```

should be treated as:

```text
transport address/identity
```

not:

```text
user identity
```

A device descriptor can bind them:

```rust
pub struct DeviceEndpoint {
    pub device_id: DeviceId,
    pub transport: TransportKind,
    pub transport_identity: TransportIdentity,
    pub expires_at: Option<Timestamp>,
    pub signature: Signature,
}
```

---

# 11. Why Transport Identity Must Be Separate

Transport identities may change because of:

- reinstall
- key rotation
- migration
- different transport implementation
- temporary relay identity
- privacy rotation

The account identity should remain stable.

---

# 12. Application Profile Separation

User profile:

```rust
pub struct AccountProfile {
    pub display_name: DisplayName,
    pub avatar: Option<BlobId>,
    pub status: Option<StatusText>,
}
```

must not define cryptographic identity.

Changing:

```text
display name
avatar
status
```

must not rotate identity keys.

---

# 13. Device Metadata

Useful metadata:

```rust
pub struct DeviceMetadata {
    pub device_name: DeviceName,
    pub platform: PlatformKind,
    pub device_class: DeviceClass,
    pub capabilities: DeviceCapabilitySet,
    pub created_at: Timestamp,
    pub last_seen: Option<Timestamp>,
}
```

Device names are presentation data.

Never use:

```text
"My Pixel"
```

as security identity.

---

# 14. Device Capability Set

Examples:

```text
messaging
files
calls
AV1 hardware decode
AV1 hardware encode
H264 hardware
H265 hardware
Bluetooth
Wi-Fi Aware
DTN relay
headless node
```

Capabilities can change.

Therefore:

```text
capabilities are signed state
not permanent identity
```

---

# 15. Linking a New Device

Core flow:

```text
Existing trusted device
        ↓
Generate linking invitation
        ↓
New device scans/receives invitation
        ↓
Mutual authentication
        ↓
User verifies
        ↓
New device key generated
        ↓
Device certificate issued
        ↓
Device added to account state
        ↓
Other devices learn update
```

The linking process must not transmit existing device private keys.

---

# 16. Device Linking Invitation

A linking invitation should be:

```rust
pub struct DeviceLinkInvite {
    pub account_id: AccountId,
    pub inviter_device: DeviceId,
    pub ephemeral_link_key: PublicKey,
    pub expires_at: Timestamp,
    pub nonce: [u8; 16],
    pub signature: Signature,
}
```

It must be:

- short-lived
- one-time
- authenticated
- replay-resistant

---

# 17. QR Linking

QR is suitable for offline linking.

QR payload may include:

```text
account id
inviting device id
ephemeral key
expiration
protocol version
short authentication code
```

The QR should not contain:

```text
private keys
session keys
message history
```

---

# 18. NFC Linking

NFC can be an optional bootstrap mechanism.

Use it to exchange:

```text
ephemeral linking material
```

not to bypass verification.

NFC proves proximity, not identity.

---

# 19. Numeric Verification

For high assurance:

```text
New device shows 6–12 digit code
Existing device shows same code
User compares
```

The code is derived from the authenticated handshake transcript.

Do not generate an unrelated random code.

---

# 20. Linking Trust Decision

The existing device must explicitly approve:

```text
Add "Tablet" to your account?
```

Show:

```text
device type
platform
approximate time
link method
verification status
```

Avoid silent device addition.

---

# 21. New Device Key Generation

New device generates locally:

```text
device signing key
transport key
session/bootstrap material
local database key
```

Private keys remain local.

The existing device signs only the new public identity information.

---

# 22. Device Addition Event

Represent device lifecycle as signed events.

Example:

```rust
pub enum DeviceEvent {
    Added(DeviceCertificate),
    Revoked(DeviceRevocation),
    Renamed(DeviceRename),
    CapabilityUpdated(DeviceCapabilityUpdate),
    TransportUpdated(DeviceTransportUpdate),
}
```

This creates a clear audit trail.

---

# 23. Account Device State

Current state is derived from:

```text
ordered signed device events
```

or a signed snapshot plus events.

Do not maintain an unverified mutable map without provenance.

---

# 24. Device Generation Number

Each account state can include a monotonic generation.

Example:

```text
generation 42
```

A new device update:

```text
generation 43
```

This helps detect stale state.

Do not rely solely on wall-clock timestamps.

---

# 25. Device Revocation

Revocation is a first-class operation.

Flow:

```text
Trusted device
   ↓
Revoke Device B
   ↓
signed revocation event
   ↓
generation increments
   ↓
sync to all devices
   ↓
future sessions reject Device B
```

---

# 26. Revocation Semantics

Revocation should prevent:

```text
future session establishment
future group key delivery
future device synchronization
future account-authorized operations
```

It cannot guarantee deletion of data already copied to the revoked device.

Documentation and UI must state this accurately.

---

# 27. Immediate Local Revocation

As soon as Device A revokes Device B:

```text
Device A
```

must immediately:

- stop sending new data to B
- stop trusting B
- remove B from future fan-out
- update local authorization

Do not wait for a server round trip.

---

# 28. Offline Revocation Propagation

If other devices are offline:

```text
revocation event
```

must propagate later through:

- direct sync
- relay/mailbox
- DTN
- linked-device synchronization

Revocation state must therefore be durable.

---

# 29. Revocation Conflict Rules

Suppose:

```text
Device A revokes B
Device C offline adds some state involving B
```

Security-sensitive state must prefer:

```text
revocation
```

once authenticated.

Revoked devices cannot regain authority through stale sync.

---

# 30. Device Expiry

Optional device certificates may expire.

Useful for:

```text
temporary device
shared terminal
emergency node
enterprise guest device
```

Expiration is not a replacement for revocation.

---

# 31. Device Rotation

Keys may rotate without changing logical `DeviceId`.

Flow:

```text
Device key generation N
   ↓
rotation event
   ↓
generation N+1
```

Old keys become invalid for future authentication.

---

# 32. Rotation Reasons

Examples:

```text
scheduled rotation
suspected compromise
secure-storage migration
algorithm upgrade
device OS migration
```

The system must support rotation before an emergency requires it.

---

# 33. Root Key Rotation

Root identity rotation is much more serious.

Architecture should support:

```text
old root
  ↓ signs transition
new root
```

The transition event must be verifiable.

Never simply replace root key metadata locally.

---

# 34. Root Rotation Event

Conceptually:

```rust
pub struct RootRotation {
    pub account_id: AccountId,
    pub old_root: PublicKey,
    pub new_root: PublicKey,
    pub generation: u64,
    pub old_signature: Signature,
    pub new_signature: Signature,
}
```

This provides continuity.

---

# 35. Compromised Root Scenario

If the root private key itself is compromised, recovery requires a stronger mechanism.

Possible future strategies:

```text
recovery quorum
trusted-device quorum
offline recovery secret
social recovery
organization authority
```

Do not claim that ordinary rotation solves a compromised root.

---

# 36. Recovery Architecture

Recovery should be designed from the beginning.

Possible modes:

```text
Recovery key
Trusted device
Multi-device quorum
Organization-managed recovery
No recovery
```

Different products may choose different policies.

---

# 37. Recovery Policy Type

```rust
pub enum RecoveryPolicy {
    None,
    RecoverySecret,
    TrustedDeviceQuorum { threshold: u8 },
    OrganizationManaged,
}
```

The core identity system should support policy variation.

---

# 38. Recovery Secret

A recovery secret should not simply be:

```text
password → private key
```

without strong KDF and policy.

Use a hardened derivation and authenticated recovery record.

The recovery secret should normally:

- recreate account authority
- not decrypt all past device-local storage automatically
- require explicit new device issuance

---

# 39. Recovery Device Addition

Recovery path:

```text
User loses all devices
   ↓
uses recovery mechanism
   ↓
reestablishes account authority
   ↓
creates new device identity
   ↓
revokes lost devices
```

Past message recovery is a separate backup problem.

Identity recovery and data backup must not be conflated.

---

# 40. Multi-Device Messaging Fan-Out

For 1:1 communication:

```text
Alice Account
├── Phone
├── Laptop
└── Tablet

Bob Account
├── Phone
└── Laptop
```

When Alice sends to Bob:

```text
Bob Phone
Bob Laptop
Alice's other devices
```

may each need the event.

The exact cryptographic fan-out mechanism can evolve, but the identity model must expose device membership.

---

# 41. Sender Attribution

A message should identify:

```text
logical account
+
originating device
```

Example:

```rust
pub struct SenderIdentity {
    pub account_id: AccountId,
    pub device_id: DeviceId,
}
```

This enables:

- auditing
- compromised-device analysis
- multi-device sync
- device-specific receipts

---

# 42. Account-Level Presentation

UI can normally display:

```text
Alice
```

not:

```text
Alice / Pixel 9
```

Device attribution remains available in:

- security details
- diagnostics
- enterprise audit
- identity-change warnings

---

# 43. Device-Level Receipts

Delivery can occur per device.

Example:

```text
Bob Phone → delivered
Bob Laptop → offline
```

Account-level UI may aggregate:

```text
Delivered to Bob
```

according to product policy.

The core should retain device-level truth.

---

# 44. Sync Between User's Own Devices

Own-device sync must be explicit.

Examples:

```text
sent messages
read state
contacts
group state
settings
drafts optionally
```

Do not treat own devices as automatically fully trusted for every application secret.

Per-data-class policy is useful.

---

# 45. Device Trust Classes

Possible:

```rust
pub enum DeviceTrustClass {
    Full,
    Limited,
    Temporary,
    Headless,
}
```

Examples:

```text
Primary phone → Full
Laptop → Full
Public kiosk → Limited
Emergency relay → Headless
```

Applications can constrain synchronization accordingly.

---

# 46. Headless Devices

The identity architecture must support:

```text
NAS
Raspberry Pi
home relay
enterprise gateway
emergency node
```

These may belong to:

```text
user account
organization account
service identity
```

Do not assume every device has a screen.

---

# 47. Service Identities

Some applications need non-human principals.

Example:

```text
school server
emergency authority
automation service
document relay
```

Use the same identity primitives.

Do not create a completely separate authentication model unless necessary.

---

# 48. Organization Identity

An organization may own:

```text
Organization Account
├── server device
├── admin device
├── relay
└── automation node
```

Authorization is layered above identity.

Identity proves:

```text
this device belongs to this organization principal
```

Authorization decides:

```text
what it may do
```

---

# 49. Multiple Accounts on One Device

The core should allow:

```text
personal
work
organization
```

identities on one physical device.

Each requires isolated:

```text
keys
storage namespace
session state
trust graph
device membership
```

The application may choose whether to expose this feature.

---

# 50. Application Namespace

One physical device may run multiple applications using the communication SDK.

Therefore device state should include:

```text
application namespace
```

where appropriate.

Do not automatically share:

```text
contacts
message history
file access
session state
```

across applications.

---

# 51. Cross-Application Identity Reuse

This should be explicit, not automatic.

Possible modes:

```text
Isolated identity per app
Shared identity with user consent
Organization-managed identity
```

The default should favor isolation.

---

# 52. Device Directory

Each account maintains a signed device directory.

Conceptually:

```rust
pub struct DeviceDirectory {
    pub account_id: AccountId,
    pub generation: u64,
    pub devices: Vec<DeviceDirectoryEntry>,
    pub signature: Signature,
}
```

This can be a compact snapshot derived from events.

---

# 53. Device Directory Entry

```rust
pub struct DeviceDirectoryEntry {
    pub device_id: DeviceId,
    pub certificate: DeviceCertificate,
    pub status: DeviceStatus,
    pub transport_endpoints: Vec<DeviceEndpoint>,
}
```

Status:

```text
Active
Revoked
Expired
```

---

# 54. Device Directory Synchronization

A peer should not require a central server to learn device changes.

Possible paths:

```text
direct peer
own-device sync
mailbox
relay
DTN
directory service
```

The directory is signed, so transport is not trusted for authenticity.

---

# 55. Stale Device Directory

A peer may have stale account state.

If a device presents:

```text
newer signed generation
```

the peer can update.

If a device presents:

```text
older generation
```

the peer should not roll back.

---

# 56. Rollback Protection

Maintain:

```text
highest trusted generation per account
```

or stronger state continuity.

Never accept:

```text
generation 17
```

after already trusting:

```text
generation 22
```

unless a specific recovery protocol authorizes it.

---

# 57. Fork Detection

Two conflicting signed account states at the same generation may indicate:

```text
bug
compromise
concurrent invalid update
```

The system should detect this.

Do not silently choose one.

Raise:

```text
IdentityForkDetected
```

and require reconciliation/security handling.

---

# 58. Concurrent Device Changes

To avoid unnecessary forks, device-state updates should use a defined concurrency model.

Options:

```text
single account authority
quorum
serialized event chain
```

For initial implementation:

```text
serialized signed account-state chain
```

is easier to reason about.

---

# 59. Account State Chain

Each state event references:

```text
previous_state_hash
```

Example:

```rust
pub struct AccountStateEvent {
    pub account_id: AccountId,
    pub generation: u64,
    pub prev_hash: StateHash,
    pub event: DeviceEvent,
    pub signature: Signature,
}
```

This creates tamper-evident continuity.

---

# 60. Device Linking Authority

Who may add a new device?

Possible policies:

```text
root authority only
any full-trust device
threshold of trusted devices
organization admin
```

Make this configurable.

---

# 61. Default Consumer Policy

For a normal messenger:

```text
existing verified full-trust device
```

can authorize a new device.

For enterprise:

```text
organization policy
```

may require admin approval.

---

# 62. Link Approval Certificate

If device-to-device approval is allowed, the account architecture must define how a device gains delegation authority.

Example:

```text
root certificate
   ↓
delegates device-link authority
```

Do not let any device arbitrarily add peers.

---

# 63. Device Roles

Possible device roles:

```text
Primary
Standard
Limited
Relay
Recovery
Admin
```

These are account-level capabilities, not UI labels only.

---

# 64. Security Capabilities

A device certificate may include:

```text
can_link_device
can_revoke_device
can_rotate_account_state
can_receive_messages
can_sync_history
can_relay
```

Use explicit capability semantics.

---

# 65. Principle of Least Authority

A headless relay should not need:

```text
account root authority
message decryption keys
device linking authority
```

It should receive only the minimum credentials required.

---

# 66. Multi-Device File Transfer

A file may target:

```text
account
specific device
device group
```

Examples:

```text
Send to Bob's account
Send only to Bob's laptop
Sync file to all my devices
```

The identity layer must support device resolution.

---

# 67. Account Address vs Device Address

Define:

```rust
pub enum Destination {
    Account(AccountId),
    Device(DeviceId),
    Devices(Vec<DeviceId>),
}
```

The application can choose semantics.

Routing resolves account destinations into active authorized devices.

---

# 68. Device Resolution

Flow:

```text
Destination Account
     ↓
Device Directory
     ↓
Active authorized devices
     ↓
Transport endpoints
     ↓
Routing
```

Do not make the application manually maintain endpoint lists.

---

# 69. Fan-Out Policy

Possible policies:

```text
All active devices
Primary device only
Best reachable device
At least one device
Application-defined subset
```

Messaging may choose:

```text
all relevant devices
```

Large files may choose:

```text
specific device
```

---

# 70. Own-Device Synchronization Policy

Own-device sync may differ by data type.

Example:

```text
messages → all full devices
settings → selected devices
large files → on demand
drafts → optional
emergency state → all full devices
```

This policy belongs above identity but uses device metadata.

---

# 71. Device Presence

Presence should be device-aware.

Example:

```text
Account Alice:
Phone online
Laptop offline
Tablet nearby
```

Account-level presence can derive:

```text
Alice reachable
```

without losing device-level detail internally.

---

# 72. Reachability vs Trust

A device may be:

```text
trusted but unreachable
reachable but revoked
reachable but unknown
```

These are distinct states.

Never combine them into one boolean.

---

# 73. Device State Type

```rust
pub struct DeviceState {
    pub trust: DeviceTrustState,
    pub reachability: DeviceReachability,
    pub lifecycle: DeviceLifecycle,
}
```

This makes invalid assumptions harder.

---

# 74. Device Lifecycle

```text
PendingLink
Active
Suspended
Revoked
Expired
```

`Suspended` can be useful for:

```text
temporarily disabled device
enterprise policy
suspected compromise under investigation
```

---

# 75. Suspension vs Revocation

Suspension:

```text
temporary
reversible
```

Revocation:

```text
security-significant
normally permanent for that key generation
```

Do not use the same operation for both.

---

# 76. Lost Device Flow

User reports:

```text
Phone lost
```

System should allow:

```text
revoke lost device
rotate affected conversation/group state
invalidate device sessions
remove device from fan-out
```

The exact data-key rotation can occur asynchronously.

---

# 77. Compromised Device Flow

Stronger than simple loss.

Recommended response:

```text
revoke device
rotate account-sensitive session material
rotate group membership keys
invalidate device tokens
trigger security warning
```

Do not assume revocation alone erases any keys already obtained.

---

# 78. Device Reinstallation

Reinstall should normally create:

```text
new DeviceId
new device key
```

rather than silently resurrecting the previous device identity.

This makes security history clearer.

---

# 79. Device Migration

A deliberate migration can preserve user experience.

Flow:

```text
Old device
   ↓
authenticated migration
   ↓
new device identity
   ↓
state transfer
   ↓
old device optionally revoked
```

Private device keys should still not be copied unless the architecture explicitly defines secure key migration.

---

# 80. Secure Storage

Private device material should use platform secure storage.

Android:

```text
Android Keystore
```

Apple:

```text
Keychain / Secure Enclave where appropriate
```

Desktop:

```text
platform credential/key store
encrypted fallback
```

The identity crate should depend on a `SecureStore` abstraction.

---

# 81. Secure Store Trait

```rust
pub trait SecureStore {
    async fn store_secret(
        &self,
        key: SecretKeyId,
        secret: SecretBytes,
    ) -> Result<(), SecureStoreError>;

    async fn load_secret(
        &self,
        key: SecretKeyId,
    ) -> Result<SecretBytes, SecureStoreError>;
}
```

Avoid returning copyable secret types casually.

---

# 82. Secret Types

Use wrappers such as:

```text
SecretBytes
Zeroizing
secrecy-style wrappers
```

to reduce accidental logging/copying.

Never derive:

```rust
Debug
```

for raw secret material.

---

# 83. Local Database Key

Each device may have a local database encryption key.

This key is distinct from:

```text
account root key
device signing key
session key
```

Do not reuse one key for multiple purposes.

---

# 84. Key Separation

Use domain-separated derivation.

Conceptually:

```text
root material
├── identity signing
├── device certification
├── recovery
└── other explicit domains
```

Never:

```text
same key
for signing + encryption + database
```

---

# 85. Session Keys

Session keys are ephemeral or periodically renewed.

They should not become part of long-lived device directory state.

Identity authenticates session establishment.

Session cryptography protects current communication.

---

# 86. Prekeys / Offline Session Bootstrap

For asynchronous E2EE messaging, the architecture may require:

```text
signed prekeys
one-time prekeys
device-specific prekey bundles
```

Each bundle belongs to a specific device.

The account directory helps discover which device bundles are valid.

---

# 87. Prekey Bundle Type

Conceptually:

```rust
pub struct DevicePrekeyBundle {
    pub account_id: AccountId,
    pub device_id: DeviceId,
    pub identity_key: PublicKey,
    pub signed_prekey: SignedPrekey,
    pub one_time_prekeys: Vec<OneTimePrekey>,
}
```

The exact cryptographic protocol should use a proven design.

---

# 88. Prekey Expiry

Prekeys must:

```text
expire
rotate
replenish
```

Do not maintain indefinite prekey material.

---

# 89. Device Authentication During Session

Peer presents:

```text
device certificate
device key proof
current account-state generation
```

The remote verifies:

```text
certificate valid
device active
not revoked
account continuity valid
```

before application protocol becomes trusted.

---

# 90. Stale Peer Handling

If remote cannot verify current device status due to stale state:

Possible policy:

```text
allow limited communication
request newer account state
delay sensitive operation
```

Do not silently trust stale high-risk identity information.

---

# 91. Offline Identity Verification

Two users without Internet can verify via:

```text
QR
numeric code
NFC
manual fingerprint
```

The verification should bind:

```text
account root identity
```

not just current transport endpoint.

This prevents re-verifying every transport change.

---

# 92. Contact Verification

A verified contact state should be tied to:

```text
AccountId / root identity
```

with awareness of root rotation.

New devices signed under the verified account can inherit account-level trust according to policy.

---

# 93. New Device Notification

When a contact adds a device:

```text
Alice added a new device.
```

The UI may show this as informational or security-significant depending on verification policy.

Do not silently hide device changes in high-security mode.

---

# 94. Identity Change Notification

If account root changes unexpectedly:

```text
Security identity changed.
```

This is more serious than adding a device.

The distinction must be explicit.

---

# 95. Verification Modes

Possible:

```rust
pub enum VerificationPolicy {
    TrustAccountRoot,
    VerifyEveryDevice,
    HighSecurity,
}
```

Default messenger:

```text
verify account root
```

Enterprise/high-security:

```text
may require every device approval
```

---

# 96. Device Transparency Log

A future enhancement can provide:

```text
append-only transparency log
```

for device additions/revocations.

This can help detect:

```text
hidden unauthorized device additions
```

The core architecture should leave room for it.

---

# 97. Self-Hosted Transparency

Enterprise deployments may host:

```text
organization device transparency service
```

without changing basic identity semantics.

---

# 98. No Mandatory Central Directory

The identity system must work:

```text
P2P
offline
LAN-only
DTN
```

A directory server may improve discovery, but it is not the source of cryptographic truth.

---

# 99. Directory Service Role

Optional directory service can provide:

```text
latest signed device directory
prekey bundles
transport hints
```

It must not be trusted to forge identities.

All state is verified cryptographically.

---

# 100. Device State Through DTN

Signed device-state updates can be transported through:

```text
store-carry-forward
```

because authenticity does not depend on the relay.

This is important for disaster scenarios.

---

# 101. Emergency Identity

Emergency apps may need:

```text
person identity
authority identity
responder identity
anonymous/limited identity
```

The identity layer should support different principal types without embedding emergency business rules.

---

# 102. Authority Identity

An authority account may certify:

```text
Police Device
Hospital Device
Rescue Coordinator
```

The UI can display:

```text
Verified Authority
```

only when cryptographic policy validates it.

---

# 103. Identity Claims

Keep optional claims separate from core identity.

Examples:

```text
organization
role
department
emergency authority
```

Claims should be signed by an appropriate issuer.

Do not hard-code them into `AccountId`.

---

# 104. Claim Type

```rust
pub struct IdentityClaim {
    pub subject: AccountId,
    pub claim_type: ClaimType,
    pub value: ClaimValue,
    pub issuer: IssuerId,
    pub expires_at: Option<Timestamp>,
    pub signature: Signature,
}
```

Applications decide which issuers they trust.

---

# 105. Privacy

Device directories reveal metadata.

Minimize:

```text
device names
exact platform
last seen
capabilities
```

to only what is necessary.

Some information can be shared only after authenticated contact/session establishment.

---

# 106. Public vs Private Device Metadata

Public:

```text
device id
device key
required certificate fields
```

Private/authenticated:

```text
friendly device name
platform detail
last-seen
capabilities
```

This reduces unnecessary metadata exposure.

---

# 107. Rotating Discovery Tokens

Nearby discovery must not advertise permanent:

```text
AccountId
DeviceId
```

in plaintext.

Use rotating ephemeral discovery tokens.

Authenticated handshake resolves them to actual device identity.

---

# 108. Device Tracking Resistance

Avoid stable:

```text
BLE identifiers
Wi-Fi service identifiers
transport advertisements
```

that allow passive tracking.

Transport discovery identity and cryptographic account identity should be separated.

---

# 109. Address Book Mapping

The messenger may map:

```text
AccountId → Contact
```

but the identity SDK should not own:

```text
phone-number address book
social graph
contact labels
```

Those are application concerns.

---

# 110. Device Audit Log

Each account should expose a local audit view:

```text
Device added
Device revoked
Key rotated
Recovery used
Root changed
```

This is especially useful in enterprise and security-sensitive contexts.

---

# 111. Audit Event Type

```rust
pub enum IdentityAuditEvent {
    DeviceLinked,
    DeviceRevoked,
    DeviceSuspended,
    DeviceRotated,
    RootRotated,
    RecoveryUsed,
    ForkDetected,
}
```

Do not include secrets.

---

# 112. Notifications

Security-significant events may trigger:

```text
local notification
cross-device notification
email/push optional
```

The identity layer emits events; the product decides presentation.

---

# 113. Cross-Device Consistency

Eventually all active trusted devices should converge on:

```text
same account generation
same active device set
same revocation set
```

Temporary divergence is expected offline.

Security rules must remain conservative during divergence.

---

# 114. Reconciliation

When devices reconnect:

```text
exchange generation
compare state hash
request missing events
verify chain
apply
```

Do not exchange entire history unnecessarily.

---

# 115. Merkle / Hash Chain Support

For many events, use:

```text
state hash
event chain
Merkle summaries
```

to efficiently reconcile.

The first implementation can start with ordered signed events and later optimize.

---

# 116. Identity Storage

Suggested storage:

```text
accounts
device_certificates
device_events
device_directory_snapshots
trust_state
verification_state
prekey_bundles
recovery_metadata
identity_audit
```

Secret material belongs in secure storage, not ordinary tables.

---

# 117. Storage Interface

```rust
pub trait IdentityStore {
    async fn account_state(...);
    async fn append_device_event(...);
    async fn device_certificate(...);
    async fn trust_state(...);
    async fn save_trust_state(...);
}
```

Keep secret store separate:

```rust
SecureStore
```

---

# 118. Transaction Boundaries

Device addition:

```text
verify certificate
 ↓
append event
 ↓
update snapshot
 ↓
commit
 ↓
emit event
```

Never emit:

```text
DeviceAdded
```

before durable persistence.

---

# 119. Idempotency

Receiving the same device event repeatedly should produce:

```text
one logical state change
```

Use unique:

```text
event id / hash
```

and generation constraints.

---

# 120. Replay Protection

Reject:

```text
old device-add event
old transport endpoint
old revoked certificate
```

when newer state is known.

Use:

```text
generation
state hash
certificate generation
expiry
```

---

# 121. Device-Specific Authorization

Applications may authorize per device.

Example:

```text
Bob's laptop can receive 2 GB file
Bob's phone should receive only metadata
```

This can combine:

```text
device capability
user policy
network policy
```

---

# 122. Enterprise Device Policy

An enterprise can require:

```text
approved device
managed device
minimum app version
hardware-backed keys
device attestation
```

These should be optional policy layers.

Do not make attestation mandatory for the core protocol.

---

# 123. Platform Attestation

Android/iOS attestation can supplement identity.

It must not replace cryptographic identity.

Attestation answers:

```text
What environment is this device running in?
```

Identity answers:

```text
Which logical device/account is this?
```

---

# 124. Device Health Claims

Optional:

```text
OS version
app version
secure storage available
hardware key support
```

These are claims/capabilities, not identity itself.

---

# 125. Version Compatibility

Device directory schema must be versioned.

Wire state:

```text
DeviceCertificateV1
DeviceEventV1
```

Internal Rust structs may evolve independently.

---

# 126. Serialization

Use explicit protocol-safe types.

Avoid:

```rust
usize
SystemTime internals
platform enums
```

on wire.

Prefer:

```text
u16
u32
u64
fixed arrays
bounded bytes
explicit timestamps
```

---

# 127. Input Limits

Limit:

```text
max devices per account
max endpoints per device
max claims
max metadata length
max device name length
max event chain batch
```

Do not let an attacker force unbounded device-directory allocation.

---

# 128. Device Count Policy

Default normal account:

```text
small bounded number
```

Enterprise may configure larger limits.

The core should not assume:

```text
exactly 5 devices
```

but must still enforce resource limits.

---

# 129. Session Cache

Maintain device-authenticated session cache:

```text
AccountId
DeviceId
AccountGeneration
SessionId
LastAuthenticated
```

Invalidate when:

```text
device revoked
root changed
certificate rotated
```

---

# 130. Revocation Cache

Fast path:

```text
revoked device set
```

for session validation.

This must be derived from durable authenticated state.

---

# 131. Device Identity API

Recommended public clients:

```rust
IdentityClient
DeviceClient
TrustClient
RecoveryClient
```

Keep public API small.

---

# 132. Example API

```rust
let identity = client.identity();

let invite = identity
    .create_device_link_invite(LinkPolicy::default())
    .await?;

let result = identity
    .approve_device_link(link_request)
    .await?;
```

Application should not directly construct signed device certificates.

---

# 133. Revoke API

```rust
identity
    .revoke_device(
        device_id,
        RevocationReason::Lost,
    )
    .await?;
```

The library handles:

```text
signed event
generation
persistence
sync notification
session invalidation
```

---

# 134. Recovery API

```rust
let recovery = identity
    .begin_recovery(recovery_material)
    .await?;
```

Recovery should be a guided state machine, not one monolithic call.

---

# 135. State Machine for Linking

```text
Created
 ↓
InviteShared
 ↓
PeerConnected
 ↓
Verified
 ↓
Approved
 ↓
CertificateIssued
 ↓
Committed
 ↓
Completed
```

Failure states:

```text
Expired
Rejected
VerificationFailed
Cancelled
```

---

# 136. State Machine for Recovery

```text
Started
 ↓
RecoveryMaterialVerified
 ↓
AuthorityRestored
 ↓
NewDeviceCreated
 ↓
OldDevicesReviewed
 ↓
RevocationsCommitted
 ↓
Completed
```

Persist progress safely where appropriate.

---

# 137. UI Boundary

Dioxus receives view models:

```text
DeviceListVm
DeviceLinkVm
SecurityIdentityVm
RecoveryVm
```

It never receives:

```text
root private key
device private key
session key
```

---

# 138. Android Kotlin Boundary

Kotlin may handle:

```text
Android Keystore
biometric confirmation
QR camera
NFC
system notifications
```

Rust owns:

```text
identity state
certificate logic
linking protocol
revocation
recovery policy
```

---

# 139. iOS Boundary

Equivalent pattern:

```text
Keychain/Secure Enclave
camera
NFC where available
system authentication
```

through platform adapter.

No identity business logic should be duplicated in Swift.

---

# 140. Headless Linking

Headless nodes can link through:

```text
CLI approval
QR rendered elsewhere
one-time token
admin-issued certificate
```

The same identity semantics apply.

---

# 141. File-Only Product Reuse

A file-transfer app may use:

```text
AccountId
DeviceId
DeviceDirectory
device pairing
device revocation
```

without:

```text
contacts
message history
conversation model
```

This proves identity is reusable.

---

# 142. ERP Reuse

ERP can map:

```text
EmployeeId
   ↓
AccountId
```

or maintain its own mapping.

The communication SDK should not know what an employee is.

---

# 143. Emergency Reuse

Emergency application can link:

```text
Responder Account
├── phone
├── radio gateway
└── vehicle node
```

without changing identity semantics.

---

# 144. Service-to-Service Reuse

Headless services can use:

```text
organization identity
device certificate
transport identity
```

for secure automated communication.

---

# 145. Privacy-Preserving Device Names

Friendly names may stay local.

Remote peers do not need to know:

```text
"Irshad's Bedroom PC"
```

unless user explicitly shares it.

Use generic capability descriptors by default.

---

# 146. Device Removal UX Semantics

UI should distinguish:

```text
Remove device
```

from:

```text
Revoke compromised device
```

If both map to revocation technically, presentation should explain consequences.

---

# 147. Device History Retention

Keep historical records:

```text
revoked device id
revocation time
certificate fingerprint
reason
```

without retaining unnecessary sensitive metadata forever.

This helps security audits.

---

# 148. Key Compromise Warnings

If a formerly revoked device appears:

```text
reject authentication
log security event
optional user warning
```

Do not silently reconnect.

---

# 149. Root Trust Cache

For verified contacts, store:

```text
trusted root fingerprint
verification time
verification method
```

Root changes require explicit policy.

---

# 150. Verification Methods

Possible:

```text
QR
numeric code
manual fingerprint
organization certificate
trusted directory
```

Record method for audit.

---

# 151. Device Linking Over Existing Secure Session

Preferred when possible:

```text
existing trusted device
    ↓
authenticated channel
    ↓
link negotiation
```

QR/NFC supplies bootstrap proof.

The actual certificate transfer uses the secure channel.

---

# 152. Device Linking Without Internet

Must work:

```text
Bluetooth
Wi-Fi Direct
LAN
QR bootstrap
```

No central service is required.

This is essential for emergency/disaster use.

---

# 153. Device Linking With Internet

Internet can simplify:

```text
rendezvous
push
directory update
```

but cryptographic authority remains local/account-controlled.

---

# 154. Recovery Without Internet

If recovery policy supports offline secret/quorum:

```text
recovery
```

must not require a cloud server.

This is valuable for resilient communication.

---

# 155. Backup Relationship

Identity backup should contain:

```text
recovery material
public account state
encrypted necessary metadata
```

not blindly copy:

```text
all active session state
all device private keys
```

unless explicitly designed.

---

# 156. New Device History Bootstrap

After linking, a new device may request:

```text
conversation history
files
contacts
settings
```

This is a synchronization problem layered above identity.

Identity only establishes:

```text
this device is authorized to participate
```

---

# 157. History Authorization

A linked device may not automatically receive all old data.

Policy examples:

```text
new device gets full history
new device gets future messages only
new device gets selected history
```

This belongs to product/data policy.

---

# 158. Device Removal and Backups

Revoking a device does not erase:

```text
backups
exports
screenshots
copied files
```

The system must not claim otherwise.

---

# 159. Threat Model

Protect against:

```text
stolen device
malicious linking attempt
replayed linking invite
stale directory rollback
rogue transport endpoint
revoked device reconnect
identity fork
device key theft
root key rotation attack
directory server compromise
relay compromise
```

The design must assume network infrastructure is untrusted.

---

# 160. Trust Assumptions

The core trusts:

```text
cryptographic verification
configured authorization policy
secure storage to its documented guarantees
```

It does not trust:

```text
relay
LAN
Bluetooth peer name
IP address
server database
device display name
```

---

# 161. Security Invariants

1. A device cannot become account-authorized without valid authority.
2. Revoked devices cannot establish future trusted sessions.
3. Old account state cannot roll back newer trusted state.
4. Transport identity cannot substitute for account identity.
5. Application profile changes cannot change identity.
6. Private device keys do not leave device under normal operation.
7. New devices receive only data authorized by product policy.
8. Device linking is replay-resistant.
9. Account root changes are explicitly verifiable.
10. Identity state is usable without a central server.

---

# 162. Testing Strategy

Unit tests:

```text
certificate verification
generation ordering
revocation
rotation
linking invite expiry
capability enforcement
```

Integration tests:

```text
link device
sync directory
revoke device
attempt reconnect
```

Fault tests:

```text
duplicate events
out-of-order events
stale directory
conflicting generation
process death during linking
```

---

# 163. Property Tests

Examples:

```text
revoked device never becomes active after reconciliation
higher generation never rolls back
duplicate event is idempotent
valid event chain verifies
tampered chain fails
```

---

# 164. Fuzz Targets

Fuzz:

```text
device certificate parser
device event parser
directory snapshot
link invite
recovery record
identity claim
```

All input sizes must be bounded.

---

# 165. Simulated Multi-Device Tests

Topology:

```text
Alice:
A1 Phone
A2 Laptop
A3 Tablet

Bob:
B1 Phone
B2 Laptop
```

Test:

```text
A1 adds A3
A2 offline
A3 sends
A2 reconnects
A1 revokes A3
Bob receives updated directory
A3 reconnect attempt rejected
```

---

# 166. Disaster Test

```text
A1 no Internet
A2 nearby via BLE
A3 reachable through DTN
```

Device-state update:

```text
A1 revokes A3
```

must eventually propagate through:

```text
A2
DTN
gateway
```

and remain cryptographically valid.

---

# 167. Performance Goals

Identity operations are not usually hot-path data-plane operations.

Prioritize:

```text
correctness
security
small metadata
incremental sync
bounded directories
```

Avoid repeatedly verifying full account history for every message.

Use validated cached state.

---

# 168. Cache Strategy

Cache:

```text
verified device certificate
active device set
revocation set
highest generation
transport endpoints
```

Invalidate on signed state update.

---

# 169. Device Directory Size

Do not transmit full history unnecessarily.

Normal handshake can send:

```text
account id
device certificate
generation
state hash
```

Then request:

```text
missing state
```

only if needed.

---

# 170. Protocol Extension Integration

Part 01 defined protocol extensions.

Identity participates before application extensions become trusted.

Handshake:

```text
Core transport
 ↓
Identity binding
 ↓
Account/device verification
 ↓
Extension negotiation
 ↓
Messaging/files/etc.
```

Identity is therefore foundational.

---

# 171. Capability Negotiation Integration

Device capabilities advertise:

```text
which extensions
which media
which transports
```

but capabilities must be bound to authenticated device identity.

Do not trust anonymous capability advertisements for authorization.

---

# 172. Routing Integration

Routing uses:

```text
DeviceDirectory
```

to resolve:

```text
AccountId
   ↓
active devices
   ↓
transport endpoints
```

Routing cannot send account-level traffic correctly without identity resolution.

---

# 173. DTN Integration

DTN destinations can use opaque identifiers derived from:

```text
AccountId
DeviceId
routing capability
```

Relay peers do not need full account metadata.

---

# 174. File Integration

File transfer target can be:

```text
account
device
selected devices
```

This allows:

```text
send to laptop only
```

without creating a fake conversation.

---

# 175. Messaging Integration

Messaging can fan out to:

```text
recipient devices
sender's other devices
```

using authenticated device membership.

This allows consistent multi-device history.

---

# 176. Call Integration

Call invitation may target:

```text
account
```

and ring:

```text
all eligible devices
```

First device to accept can establish call ownership.

Other devices receive:

```text
call answered elsewhere
```

---

# 177. Call Ring Arbitration

Need deterministic state:

```text
Ringing
AcceptedBy(DeviceId)
Cancelled
Expired
```

This avoids simultaneous multi-device answers.

Identity provides the device list; call protocol handles arbitration.

---

# 178. Notification Integration

Push targets are device-specific.

Maintain:

```text
DeviceId → push endpoint
```

as application/platform metadata.

Push token is not identity.

---

# 179. Device Endpoint Privacy

Transport/push endpoints should be:

```text
rotatable
expiring
scoped
```

where possible.

Do not expose them permanently in public profile data.

---

# 180. Device Link Rate Limits

Protect against link spam.

Limit:

```text
active invites
attempts per time window
failed verification attempts
```

and require user confirmation.

---

# 181. Recovery Rate Limits

Recovery attempts should be rate-limited where infrastructure participates.

Offline recovery should use strong local cryptographic proof rather than relying solely on rate limits.

---

# 182. UX States

Device management UI:

```text
This device
Other devices
Recently revoked
Security alerts
```

Each row:

```text
name
platform
last active
trust status
role
```

Sensitive actions require confirmation.

---

# 183. New Device UX

Flow:

```text
Settings
 ↓
Linked Devices
 ↓
Link New Device
 ↓
Show/Scan QR
 ↓
Verify
 ↓
Approve
 ↓
Sync
```

Do not mix this with ordinary contact pairing.

---

# 184. Contact Pairing vs Device Linking

These are distinct.

Contact pairing:

```text
Alice ↔ Bob
```

Device linking:

```text
Alice Phone ↔ Alice Laptop
```

They must use different protocol states and UI language.

---

# 185. Device Name Validation

Device display names should be:

```text
bounded
sanitized
localizable
non-authoritative
```

Do not permit device name to influence security policy.

---

# 186. Audit Export

Support exporting:

```text
device list
security fingerprints
revocation history
identity generation
```

without exporting private keys.

Useful for enterprise/security support.

---

# 187. API Surface

Suggested modules:

```text
identity/
├── account.rs
├── device.rs
├── certificate.rs
├── directory.rs
├── event.rs
├── trust.rs
├── linking.rs
├── recovery.rs
├── claims.rs
├── audit.rs
└── error.rs
```

---

# 188. Crate Split

Recommended:

```text
comm-identity-types
comm-identity
comm-identity-protocol
comm-identity-storage
```

Keep split only if complexity justifies it.

Initially:

```text
comm-identity
```

with internal modules may be sufficient.

---

# 189. Error Types

```rust
pub enum IdentityError {
    InvalidCertificate,
    RevokedDevice,
    ExpiredDevice,
    StaleGeneration,
    ForkDetected,
    LinkExpired,
    LinkRejected,
    VerificationFailed,
    Unauthorized,
    RecoveryFailed,
    Storage,
    Crypto,
}
```

Do not collapse into:

```text
Identity failed
```

---

# 190. No `anyhow` in Public Domain API

Use typed errors for reusable library boundaries.

`anyhow` can be used in:

```text
CLI
application bootstrap
top-level integration
```

but not as the identity protocol contract.

---

# 191. Migration Strategy

Persistent identity schema must support migrations.

Never rewrite identity state destructively without backup/recovery strategy.

Migration tests should include:

```text
old device directory
old certificates
old trust state
old recovery metadata
```

---

# 192. Algorithm Agility

Do not hard-code one cryptographic algorithm into every type name.

Use:

```text
AlgorithmId
KeyType
SignatureScheme
```

where future migration matters.

But avoid needless generic abstraction in the first implementation.

---

# 193. Algorithm Downgrade Protection

A peer must not force:

```text
old weak algorithm
```

if policy requires stronger.

Negotiation chooses:

```text
mutually supported
∩
locally allowed
```

---

# 194. Root Key Backup

If user chooses recovery backup:

```text
encrypted export
```

must be:

- authenticated
- strongly derived
- versioned
- integrity-protected

Never export raw private key in plaintext.

---

# 195. Backup Import

Import must verify:

```text
format version
integrity
account identity
recovery policy
```

before replacing local state.

---

# 196. Identity Reset

Resetting identity means:

```text
new account identity
```

not:

```text
clear app cache
```

UI must explain:

```text
contacts may see you as a new identity
old encrypted relationships may not carry over
```

---

# 197. Account Deletion

Account deletion in a P2P system cannot erase all copies everywhere.

It can:

```text
revoke devices
publish tombstone/state
delete local secrets
request server cleanup
```

but must not promise impossible remote deletion.

---

# 198. Organization Offboarding

Enterprise example:

```text
employee leaves
 ↓
revoke organization device/capability
 ↓
remove from future keys
 ↓
block future sessions
```

Personal identity may remain unaffected if separated properly.

---

# 199. Multi-Tenant Safety

If platform is embedded in a multi-tenant app:

```text
Tenant A
Tenant B
```

must have isolated:

```text
identity namespaces
device directories
trust graphs
storage
```

Never key solely by `AccountId` if tenant context is required.

---

# 200. Recommended Initial Implementation

For the first production version, implement:

```text
AccountId
DeviceId
root identity key
device signing key
device certificate
signed device directory
device add
device revoke
device transport endpoint binding
QR link
numeric verification
account generation
rollback protection
device-level session authentication
secure storage abstraction
```

Defer initially:

```text
social recovery
transparency log
complex quorum recovery
attestation enforcement
multi-root organizations
```

but leave extension points.

---

# 201. Implementation Phases

## Phase 1 — Types

Implement:

```text
AccountId
DeviceId
DeviceCertificate
DeviceStatus
DeviceDirectory
```

## Phase 2 — Local Identity

```text
root key generation
device key generation
secure storage
certificate issuance
```

## Phase 3 — Linking

```text
invite
QR
ephemeral handshake
numeric verification
approval
certificate commit
```

## Phase 4 — Revocation

```text
signed revocation
generation
session invalidation
fan-out update
```

## Phase 5 — Directory Sync

```text
hash
generation exchange
incremental events
```

## Phase 6 — Recovery

```text
basic recovery secret or trusted-device policy
```

## Phase 7 — Hardening

```text
fuzzing
fault injection
rollback tests
fork tests
migration tests
```

---

# 202. Definition of Done

Part 02 is complete when:

- one account can contain multiple devices
- every device has independent keys
- transport identity is separate from account identity
- devices can be linked without Internet
- linking is authenticated and replay-resistant
- user confirmation is required
- device revocation is durable
- revoked devices cannot create future trusted sessions
- device state cannot roll back
- stale directories are detected
- account generation is monotonic
- own-device sync can discover active devices
- file transfer can target specific devices
- messaging can fan out across devices
- Dioxus does not receive secret keys
- Kotlin/Swift do not own identity logic
- secure storage is abstracted
- identity works without a central server
- device-state updates can travel through DTN
- the system supports headless/service devices
- file-only products can reuse the identity layer
- fuzz/property/integration tests exist

---

# 203. Relationship to Other Architecture Parts

Part 02 feeds directly into:

```text
03 — Transport & Routing Policy Engine
04 — Offline Event Log
05 — Robust File / Blob Subsystem
06 — DTN / Store-Carry-Forward
07 — Capability Negotiation Expansion
11 — Relay / Self-Hosted Infrastructure
14 — Proximity Abstraction
15 — QR / NFC Bootstrap Pairing
16 — Daemon & Headless Runtime
17 — Emergency Priority Architecture
19 — C ABI / FFI
20 — Embedded Linux Node
23 — External Interoperability Suite
```

Transport needs device resolution.

Messaging needs device fan-out.

Files need specific-device targeting.

DTN needs authenticated destination identities.

Emergency systems need device-independent principal identity.

---

# 204. Final Principle

The platform should treat a logical person or organization as a **cryptographic account containing independently revocable devices**, not as a single device key.

The correct model is:

```text
Account
  │
  ├── Device A
  ├── Device B
  ├── Device C
  └── Revoked Device D
```

with:

```text
Account identity
≠
Device identity
≠
Transport identity
≠
Session identity
≠
User profile
```

That separation gives the platform:

- secure multi-device operation
- device replacement
- device revocation
- offline linking
- headless-node support
- reusable identity for messaging/files/ERP/emergency products
- future recovery mechanisms
- transport independence
- better long-term protocol stability

This identity architecture should be implemented before higher-level multi-device messaging or file synchronization is considered production-ready.
