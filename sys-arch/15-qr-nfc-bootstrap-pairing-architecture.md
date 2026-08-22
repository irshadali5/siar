# Part 15 — QR / NFC Bootstrap & Secure Pairing Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 15 of 24  
**Primary language:** Rust  
**Primary goals:** secure device/contact bootstrap, QR and NFC pairing, short-lived invitations, offline pairing, authenticated key exchange, misbinding resistance, replay protection, multi-device identity integration, transport handoff, recovery safety, reusable across messaging/files/DTN/emergency/custom products

---

# 1. Purpose

Before two devices can communicate securely, they often need a trustworthy way to bootstrap identity.

Internet directories and usernames are useful, but they are not always available or desirable.

The platform should therefore support intentional physical bootstrap using:

```text
QR code
NFC tap
manual short authentication string
numeric comparison
nearby BLE-assisted discovery
```

The bootstrap layer must support:

- linking a new device to an existing account
- adding a contact face-to-face
- secure file handoff
- offline pairing
- emergency/team provisioning
- headless node enrollment
- organization device provisioning
- cross-device trust establishment

The fundamental rule is:

> **QR/NFC bootstrap establishes cryptographic context; it must never silently bypass identity authorization.**

---

# 2. Architectural Position

```text
User Intent
   ↓
Bootstrap Session
   ↓
QR / NFC / Manual Verification
   ↓
Ephemeral Cryptographic Exchange
   ↓
Secure Nearby/Internet Session
   ↓
Identity Authentication
   ↓
Authorization
   ↓
Device / Contact / Capability Binding
```

Part 14 discovers nearby transport opportunities.

Part 15 provides intentional trust bootstrap.

Part 02 remains the authority for account/device identity.

---

# 3. Bootstrap Is Not Bulk Transport

QR and NFC should carry:

```text
small bootstrap payload
```

not:

```text
message history
large files
full certificates if avoidable
```

After bootstrap:

```text
BLE
LAN
Wi-Fi Direct
Iroh
```

carries the actual protocol exchange.

---

# 4. Bootstrap Use Cases

Recommended explicit modes:

```rust
pub enum BootstrapPurpose {
    LinkOwnDevice,
    AddContact,
    PairForFileTransfer,
    EnrollOrganizationDevice,
    EnrollRelayNode,
    EmergencyTeamPairing,
}
```

Each mode has different authorization semantics.

---

# 5. Do Not Reuse One Generic Pairing Flow

For example:

```text
Add Contact
```

does not grant:

```text
account device authority
```

and:

```text
Link Own Device
```

should not behave like casual nearby sharing.

Use purpose-specific state machines over common primitives.

---

# 6. Bootstrap Session ID

```rust
pub struct BootstrapSessionId([u8; 16]);
```

Generated randomly and locally.

It scopes:

```text
invitation
handshake
expiry
replay detection
diagnostics
```

---

# 7. Bootstrap Invitation

Conceptual:

```rust
pub struct BootstrapInvitation {
    pub version: u16,
    pub session_id: BootstrapSessionId,
    pub purpose: BootstrapPurpose,
    pub initiator_ephemeral_key: EphemeralPublicKey,
    pub rendezvous: Vec<RendezvousHint>,
    pub expires_at: Timestamp,
    pub nonce: BootstrapNonce,
    pub authenticator: Option<BootstrapAuthenticator>,
}
```

Keep payload bounded.

---

# 8. Invitation Must Be Short-Lived

Recommended semantics:

```text
valid for minutes
single-use where possible
explicitly cancellable
```

Do not create permanent QR codes that authorize device linking.

---

# 9. Single-Use Invitation

Once successfully consumed:

```text
mark used
```

Subsequent attempts:

```text
reject
```

Persist consumption if replay could cross process restart.

---

# 10. Expiry

Invitation expires even if never used.

Expired invitation:

```text
cannot establish new trust
```

---

# 11. Bootstrap Nonce

```rust
pub struct BootstrapNonce([u8; 16]);
```

Provides uniqueness/freshness.

Do not derive security solely from session ID.

---

# 12. Ephemeral Key Pair

Each bootstrap session creates an ephemeral key pair.

```text
EphemeralPrivateKey
EphemeralPublicKey
```

The private key:

```text
never enters QR/NFC payload
```

---

# 13. Ephemeral Key Lifetime

Destroy after:

```text
successful bootstrap
expiry
cancel
fatal error
```

Do not persist indefinitely.

---

# 14. QR Payload

QR should encode:

```text
version
session ID
purpose
ephemeral public key
short rendezvous hints
expiry
integrity/authentication metadata
```

Use compact binary payload then a QR-safe encoding if required.

---

# 15. QR Encoding Format

Internal canonical representation may be Postcard.

Presentation layer can encode bytes as:

```text
Base64url
Base32
custom URI wrapper
```

A human-readable URI form can help OS camera integration.

Example conceptually:

```text
comm://pair/<encoded-bootstrap>
```

Exact URI becomes public protocol surface and must be versioned.

---

# 16. QR Size Discipline

QR payload should remain small.

Do not embed:

```text
full history
large certificate chains
large profile metadata
```

If more data required, QR carries a capability/token to fetch it over the authenticated session.

---

# 17. NFC Payload

NFC can carry the same logical bootstrap invitation.

Use NDEF-compatible representation where needed by platform adapter.

Core Rust model remains identical.

---

# 18. QR and NFC Share One Semantic Format

Avoid:

```text
QR protocol
NFC protocol
```

with separate security designs.

Instead:

```text
BootstrapInvitation
      ↓
QR adapter
or
NFC adapter
```

---

# 19. Manual Code Fallback

When QR/NFC unavailable, support:

```text
short authentication string
numeric comparison
```

Manual code should verify an already-established cryptographic handshake, not replace strong key exchange.

---

# 20. SAS

Short Authentication String:

```text
derived from handshake transcript
```

Example:

```text
482 917
```

Both users compare.

If equal:

```text
approve
```

If different:

```text
abort
```

---

# 21. SAS Security Principle

SAS must be derived from:

```text
both ephemeral keys
session transcript
purpose
identities where known
```

not generated independently on each UI.

---

# 22. Emoji / Word Verification

Optional UX:

```text
4 words
or
emoji sequence
```

can represent the same transcript-derived SAS.

Numeric comparison should remain available for accessibility/interoperability.

---

# 23. No "Accept Anyway"

If SAS mismatch:

```text
abort
```

Do not provide a casual bypass button for high-trust operations like device linking.

---

# 24. QR Security Property

Scanning QR creates an out-of-band binding:

```text
scanner received the initiator's bootstrap material
through a user-directed physical channel
```

The subsequent network session must prove possession of the matching ephemeral private key.

---

# 25. NFC Security Property

NFC offers intentional very-short-range interaction.

Still:

```text
NFC proximity != account authorization
```

The cryptographic handshake and user approval remain required.

---

# 26. Bootstrap Channel Threats

Threats include:

```text
QR replacement
camera scanning wrong code
screen capture
NFC relay
replay
stale invitation
session hijack
mismatched purpose
malicious nearby peer
```

Architecture must explicitly defend against these.

---

# 27. Purpose Binding

The invitation cryptographically binds:

```text
BootstrapPurpose
```

An invitation for:

```text
PairForFileTransfer
```

must not be usable as:

```text
LinkOwnDevice
```

---

# 28. Session Binding

All subsequent handshake messages carry:

```text
BootstrapSessionId
```

and bind into transcript.

---

# 29. Rendezvous Hints

QR/NFC may contain hints such as:

```text
ephemeral proximity token
Iroh endpoint hint
LAN hint
BLE service hint
```

They help find the peer.

They are not trusted identity.

---

# 30. Minimal Rendezvous Disclosure

Prefer:

```text
opaque temporary token
```

rather than stable addresses when possible.

---

# 31. Transport Selection

After QR/NFC scan:

```text
candidate paths discovered
```

Part 03 selects:

```text
LAN
BLE
Wi-Fi Direct
Iroh direct/relay
```

The pairing protocol is transport-independent.

---

# 32. Transport Handoff

Example:

```text
QR scanned
 ↓
BLE session starts
 ↓
identity handshake
 ↓
Wi-Fi Direct becomes available
 ↓
remaining device-sync data moves to Wi-Fi
```

Bootstrap trust remains unchanged.

---

# 33. Offline Pairing

QR/NFC pairing must work with:

```text
no Internet
```

provided devices can establish some local transport.

Example:

```text
QR
+
BLE
```

or:

```text
QR
+
LAN
```

---

# 34. Fully Offline Device Linking

Own-device linking can:

```text
scan QR
authenticate locally
authorize device
issue device certificate/state event
transfer minimum account state
```

without central server.

Later both devices synchronize when Internet returns.

---

# 35. Own-Device Linking

This is the most security-sensitive bootstrap purpose.

Flow:

```text
Existing trusted device
    ↓
Create Link Invitation
    ↓
New device scans
    ↓
Ephemeral secure handshake
    ↓
User verifies
    ↓
Existing device authorizes
    ↓
Device certificate/event committed
    ↓
New device becomes account member
```

---

# 36. Existing Device Authority

Only a device with appropriate Part 02 authority may approve new device.

Do not grant authority merely because it can display QR.

---

# 37. Link Authorization

```rust
pub struct DeviceLinkAuthorization {
    pub account_id: AccountId,
    pub new_device_key: DevicePublicKey,
    pub permissions: DevicePermissions,
    pub generation: u64,
    pub authorizer: DeviceId,
}
```

This is signed according to Part 02 identity architecture.

---

# 38. Commit Point

A device is not linked until:

```text
signed device-add event/certificate
is durably committed
```

Scanning QR is not the commit point.

---

# 39. Interrupted Device Link

Crash before commit:

```text
new device remains unlinked
```

Crash after commit:

```text
recovery sees linked device
```

This integrates Part 09.

---

# 40. Half-Linked State Must Not Exist

Never allow:

```text
new device has account secrets
but no durable authorization
```

Transfer sensitive account state only after or atomically around durable authorization according to security design.

---

# 41. Minimal Initial State

After authorization, transfer only what new device needs first:

```text
account identity
device directory
current security state
configuration
```

Then optionally:

```text
message history
blob metadata
```

---

# 42. History Policy

Part 02 device policy may specify:

```text
future only
recent history
full history
selected conversations
```

Pairing UI should expose policy where appropriate.

---

# 43. New Device Key Generation

New device generates its own long-term device key locally.

Do not generate the new device's private key on the old device and transmit it.

---

# 44. Secure Storage Before Link

New device should verify:

```text
hardware/OS secure storage available
```

where required, before final authorization.

---

# 45. Device Naming

Display name:

```text
"Pixel 9"
"Work Laptop"
```

is metadata only.

Identity is cryptographic DeviceId/public key.

---

# 46. Permission Assignment

Own-device link may grant:

```text
normal device
admin/link-authority
recovery authority
headless relay
```

Use least privilege.

---

# 47. Default Permissions

New personal device should not automatically gain every sensitive authority unless product deliberately chooses that model.

---

# 48. Device-Link Confirmation UX

Existing device shows:

```text
Link new device?
Device type
Verification code
Requested permissions
```

New device shows matching verification.

---

# 49. Biometric / PIN Confirmation

For high-risk operations, existing device may require local:

```text
PIN
biometric
OS authentication
```

before signing device authorization.

Platform adapter owns biometric UI; Rust owns authorization semantics.

---

# 50. Contact Bootstrap

Face-to-face contact addition is less privileged.

Flow:

```text
Alice shows QR
Bob scans
secure handshake
both verify
exchange identity/contact cards
mark verified contact
```

No account-device authority is granted.

---

# 51. Mutual Contact Verification

Both sides should be able to verify.

Do not create asymmetric hidden trust accidentally.

---

# 52. Contact Trust Level

Possible:

```rust
pub enum ContactTrust {
    Unverified,
    VerifiedInPerson,
    VerifiedByExistingTrust,
}
```

This is application/security metadata.

---

# 53. Contact QR Privacy

A public contact QR may be intentionally shareable.

A private face-to-face QR should be short-lived.

Support separate modes.

---

# 54. Permanent Contact Code

If product supports public QR profile:

```text
it must not grant high-trust device authorization
```

It can provide:

```text
public identity key/contact handle
```

then communication still authenticates normally.

---

# 55. Temporary Pairing QR

Use for:

```text
high-trust verification
```

with short expiry and one-time semantics.

---

# 56. File-Transfer Bootstrap

For ad-hoc nearby file share:

```text
sender displays QR
receiver scans
session establishes
file offer starts
```

No permanent contact relationship required.

---

# 57. Ephemeral File Session

Can create:

```text
session-only trust
```

with no saved contact after transfer.

---

# 58. File Session Authorization

Session permits:

```text
receive offered file
```

not arbitrary messaging/account access.

---

# 59. Headless Node Enrollment

A headless relay/server may display:

```text
terminal QR
```

or accept NFC/token.

Admin scans and approves:

```text
node role
capabilities
organization
```

---

# 60. Node Enrollment Certificate

Issue a purpose-specific certificate/authorization.

Do not make headless relay a normal chat device unless intended.

---

# 61. Organization Enrollment

Enterprise may use:

```text
signed enrollment QR
```

containing:

```text
organization bootstrap
infrastructure profile
one-time enrollment capability
```

---

# 62. Organization Trust Anchor

Enrollment QR can carry:

```text
organization root fingerprint
```

so device knows which organization infrastructure to trust.

---

# 63. Enrollment Server Optional

Organization flow may contact server if online.

Offline enrollment should remain possible where policy allows.

---

# 64. Emergency Team Pairing

During emergency deployment:

```text
team leader
field devices
gateway nodes
```

can be provisioned rapidly through QR/NFC.

Use pre-defined roles and limited authority.

---

# 65. Bulk Provisioning

For many devices, one static QR should not grant unlimited permanent authorization.

Use:

```text
signed enrollment batch token
usage limit
expiry
role constraints
```

---

# 66. Enrollment Token

```rust
pub struct EnrollmentToken {
    pub issuer: IdentityRef,
    pub role: EnrollmentRole,
    pub expires_at: Timestamp,
    pub max_uses: u32,
    pub nonce: [u8; 16],
}
```

Signed by authorized issuer.

---

# 67. Usage Counter

If offline multi-use enrollment exists, global usage enforcement is difficult.

For high assurance:

```text
prefer single-use
```

or require online/authority reconciliation.

Document limits.

---

# 68. QR Replay Protection

Replay defenses:

```text
short expiry
single-use session ID
ephemeral key
transcript binding
persisted consumed-session tombstone
```

---

# 69. Consumed Invitation Tombstone

Store:

```text
BootstrapSessionId
consumed_at
```

for at least the invitation replay horizon.

---

# 70. Replay After Crash

Consumption marker must survive process crash before final UI completion if trust authorization was committed.

---

# 71. Invitation Cancellation

Initiator can cancel.

Persist cancellation if invitation remains externally visible.

---

# 72. Cancelled QR

Scanner may still possess screenshot.

Handshake checks invitation state and rejects.

---

# 73. Screenshot Risk

QR can be photographed.

Therefore security relies on:

```text
expiry
one-time token
ephemeral proof
user verification
```

not physical secrecy alone.

---

# 74. QR Replacement Attack

Attacker may replace displayed QR with theirs.

Mitigation:

```text
SAS comparison
trusted UI context
purpose display
```

For highest assurance, require both QR possession and transcript verification.

---

# 75. NFC Relay Attack

NFC can theoretically be relayed.

Do not treat NFC distance as absolute proof.

Use transcript verification for high-risk device linking.

---

# 76. Man-in-the-Middle Resistance

Handshake must authenticate both ephemeral contributions and bind them to out-of-band bootstrap material.

Use a vetted authenticated key exchange design.

Do not invent cryptographic primitives.

---

# 77. Noise-Like Handshake

A Noise-based or similarly vetted handshake framework is appropriate conceptually.

Exact pattern depends on known/unknown static keys.

Architecture requirement:

```text
ephemeral forward secrecy
transcript authentication
identity binding
```

---

# 78. Crypto Agility

Handshake suite is negotiated only within safe allowed profiles.

Do not allow attacker-controlled downgrade.

Part 07 security negotiation applies.

---

# 79. Transcript

Canonical transcript includes:

```text
bootstrap version
purpose
session ID
initiator ephemeral key
responder ephemeral key
identity keys
capability/session selections
```

as appropriate.

---

# 80. Transcript Hash

```rust
pub struct BootstrapTranscriptHash([u8; 32]);
```

Used for:

```text
SAS
confirmation
audit/debug identifiers
```

Do not expose raw secret inputs.

---

# 81. Confirmation Messages

Both peers exchange:

```text
transcript confirmation
```

before final authorization.

This catches mismatched negotiation state.

---

# 82. Pairing State Machine

```text
Created
 ↓
Advertised
 ↓
Scanned
 ↓
TransportConnecting
 ↓
Handshake
 ↓
VerificationPending
 ↓
AuthorizationPending
 ↓
Committing
 ↓
Completed
```

Terminal:

```text
Cancelled
Expired
Rejected
Failed
```

---

# 83. Strict Transition Validation

Examples:

```text
Completed → Handshake
```

invalid.

```text
Expired → Completed
```

invalid.

Use explicit typed state transitions.

---

# 84. Verification Pending

For high-trust flow, neither side is authorized yet.

User confirmation required.

---

# 85. Authorization Pending

Cryptographic peer verified, but policy authorization still pending.

Example:

```text
which device permissions?
```

---

# 86. Idempotent Completion

Repeated final confirmation message:

```text
does not create duplicate device/contact
```

Stable session/device IDs ensure idempotency.

---

# 87. Timeout

Each stage has timeout:

```text
scan
connect
handshake
verification
authorization
```

Avoid indefinitely-held pairing sessions.

---

# 88. Resource Limits

Part 08 limits:

```text
active invitations
pending scans
pairing handshakes
verification sessions
```

---

# 89. Pairing Flood

Unknown nearby attackers may initiate many attempts.

Use:

```text
rate limit
user-intent gate
temporary block
```

---

# 90. User-Intent Gate

High-trust linking should only be accepted when user explicitly put device into:

```text
Link Device
```

mode.

Do not process unsolicited device-link attempts in background.

---

# 91. Pairing Window

Example:

```text
pairing mode active 2 minutes
```

Then automatically closes.

Exact default configurable.

---

# 92. QR Scanner Boundary

Platform camera/scanner returns:

```text
raw decoded payload
```

Rust core validates protocol.

Do not put trust decisions in UI scanner code.

---

# 93. Android Responsibilities

Kotlin may own:

```text
camera integration
QR scanning surface
NFC reader/writer
biometric prompt
OS permission
```

Rust owns:

```text
bootstrap state machine
crypto
expiry
authorization
identity events
```

---

# 94. iOS Responsibilities

Swift/platform adapter similarly owns:

```text
camera
NFC
biometric
```

while Rust core owns semantics.

---

# 95. Desktop QR

Desktop can:

```text
render QR
scan via camera if supported
import QR image explicitly
```

For security, importing an old screenshot still remains subject to expiry/single-use rules.

---

# 96. Terminal QR

Headless node can render QR as terminal blocks.

Useful for:

```text
Raspberry Pi
server enrollment
```

---

# 97. Accessibility

Provide alternatives:

```text
numeric code
copyable bootstrap code
manual verification
```

Do not require camera/NFC for all users.

---

# 98. Manual Bootstrap Code

A manual code can encode:

```text
session locator
short token
```

but high-security trust still relies on cryptographic transcript confirmation.

---

# 99. Human Error

Design verification strings to minimize mistakes.

Avoid:

```text
long hexadecimal fingerprints
```

as default UX.

Use them only advanced/debug.

---

# 100. Advanced Fingerprint

Security screen may expose:

```text
full identity fingerprint
```

for manual expert verification.

---

# 101. Identity Fingerprint

Fingerprint should derive from stable identity public key, not display metadata.

---

# 102. Multi-Device Integration

New device linking updates:

```text
device directory
account generation
permissions
```

Part 02 distributes the resulting signed state.

---

# 103. Device Revocation During Pairing

If authorizing device becomes revoked mid-session:

```text
abort
```

before commit.

---

# 104. Account Generation Change

If account generation changes during pairing:

```text
revalidate authorization
```

Do not commit using stale state.

---

# 105. Concurrent Device Links

Two authorized devices may link new devices concurrently.

Part 02 conflict/generation rules apply.

Bootstrap layer does not invent its own global serialization.

---

# 106. Capability Negotiation

After secure transport/session:

```text
Part 07
```

determines available features.

Bootstrap payload should not carry full runtime capability state.

---

# 107. Proximity Integration

Part 14 provides:

```text
BLE observation
Wi-Fi candidate
LAN candidate
```

after QR/NFC bootstrap.

---

# 108. Routing Integration

Part 03 chooses best path to continue pairing/state transfer.

---

# 109. File Integration

Ad-hoc QR file share uses Part 05 after ephemeral session authorization.

---

# 110. DTN Integration

DTN can distribute:

```text
signed organization bootstrap announcement
```

but high-trust enrollment should still require explicit verification/authorization.

---

# 111. Offline Event Log

Part 04 records semantic transitions such as:

```text
DeviceAdded
ContactVerified
NodeEnrolled
```

Do not journal every QR scan frame.

---

# 112. Crash Recovery

Part 09 handles:

```text
pending invitation
consumed invitation
committed device event
orphan ephemeral state
```

Ephemeral handshake sessions can simply expire/restart if not committed.

---

# 113. Crash Before Scan

No effect.

---

# 114. Crash After Scan Before Authorization

Session expires/restarts.

No trust committed.

---

# 115. Crash During Authorization Commit

Database/event transaction defines outcome.

Recovery checks committed device/contact event.

---

# 116. Crash After Commit Before UI Success

Recovery shows paired state.

Do not ask user to pair again unnecessarily.

---

# 117. Invitation Persistence

Persist minimum data required for:

```text
expiry
cancel
single-use
crash recovery
```

Private ephemeral key may be stored only if session recovery is explicitly desired and secure.

Simpler v1:

```text
ephemeral session lost on crash
invitation cancelled/expired
```

for incomplete sessions.

---

# 118. Secure Secret Storage

Long-term keys use secure storage.

Short-lived ephemeral bootstrap private keys should remain memory-only where practical.

---

# 119. Pairing Audit

Security history may record:

```text
new device linked
authorizing device
time
permissions
```

not QR payload.

---

# 120. Contact Audit

Optional:

```text
verified in person
```

with timestamp.

Avoid storing exact physical location.

---

# 121. Organization Audit

Record:

```text
enrollment issuer
role
device
policy version
```

---

# 122. Privacy

QR invitation may expose:

```text
protocol presence
purpose
temporary routing hints
```

Minimize stable metadata.

---

# 123. No Personal Data in QR by Default

Avoid:

```text
name
email
phone
organization details
```

unless user explicitly chose public contact-card QR.

---

# 124. Bootstrap Logging

Logs may include:

```text
session-id hash
purpose
state
error
```

Never:

```text
ephemeral private key
full invitation secret
SAS secret derivation material
```

---

# 125. Error Model

```rust
pub enum BootstrapError {
    InvalidPayload,
    UnsupportedVersion,
    Expired,
    AlreadyConsumed,
    Cancelled,
    PurposeMismatch,
    TransportUnavailable,
    HandshakeFailed,
    VerificationFailed,
    Unauthorized,
    IdentityChanged,
    ResourceDenied,
    SecureStoreUnavailable,
    Timeout,
}
```

---

# 126. User-Facing Errors

Map to:

```text
This QR code expired.
This pairing request was already used.
Verification codes did not match.
This device is not authorized to add another device.
Nearby connection failed.
```

---

# 127. No Security Ambiguity

Do not show generic:

```text
Something went wrong
```

for verification mismatch.

Make security failures clear.

---

# 128. QR Parser Fuzzing

Part 10 fuzz targets:

```text
bootstrap envelope
URI parser
base encoding
version field
rendezvous hints
expiry
```

---

# 129. NFC Parser Fuzzing

Fuzz:

```text
NDEF wrapper
bootstrap payload
truncation
duplicate records
unexpected record order
```

---

# 130. State-Machine Fuzzing

Generate:

```text
scan
connect
verify
cancel
expire
approve
duplicate confirm
```

and assert valid transitions.

---

# 131. Replay Tests

Reuse same QR:

```text
after successful pairing
```

Expected:

```text
rejected
```

---

# 132. Expiry Test

Scan after expiration:

```text
reject before trust
```

---

# 133. Purpose Confusion Test

Use file-transfer QR for device linking.

Expected:

```text
PurposeMismatch
```

---

# 134. MITM Test

Swap responder ephemeral key.

Expected:

```text
SAS/transcript mismatch
```

---

# 135. Wrong-Device Test

QR from Alice, network session from Mallory.

Expected:

```text
proof-of-possession/transcript authentication fails
```

---

# 136. Screenshot Replay Test

Old screenshot after invitation consumed.

Expected:

```text
AlreadyConsumed/Expired
```

---

# 137. Crash Test

Kill after authorization commit but before final response.

Restart:

```text
paired exactly once
```

---

# 138. Concurrent Scan Test

Two scanners use same one-time invitation.

Only one succeeds.

---

# 139. Contact Pairing Test

Mutual verification creates contact trust but no device-link authority.

---

# 140. File Session Test

Ephemeral file pairing completes transfer without adding persistent contact.

---

# 141. Offline Test

No Internet.

QR + BLE/LAN still links device/contact according to policy.

---

# 142. Transport Switch Test

Pairing starts over BLE, continues over LAN/Wi-Fi.

Transcript/session identity remains valid.

---

# 143. Revocation Race Test

Authorizer revoked before commit.

Expected:

```text
authorization fails
```

---

# 144. Account Generation Race

Account state advances mid-link.

Expected:

```text
revalidate or restart authorization
```

---

# 145. Permission Test

Camera denied.

Manual/NFC alternatives remain available where supported.

---

# 146. NFC Disabled Test

QR/manual flow remains available.

---

# 147. Resource Flood Test

Thousands of fake pairing attempts.

Expected:

```text
bounded memory
bounded handshakes
no prompt spam
```

---

# 148. Golden Vectors

Maintain stable vectors for:

```text
BootstrapInvitationV1
transcript canonicalization
SAS derivation test inputs
enrollment tokens
```

Do not expose real secrets.

---

# 149. Cryptographic Test Vectors

Use upstream/reference vectors for cryptographic primitives.

Protocol-specific transcript vectors should be deterministic.

---

# 150. Suggested Crate Structure

```text
crates/comm-bootstrap/
├── src/
│   ├── lib.rs
│   ├── invitation.rs
│   ├── purpose.rs
│   ├── session.rs
│   ├── state.rs
│   ├── transcript.rs
│   ├── sas.rs
│   ├── rendezvous.rs
│   ├── authorization.rs
│   ├── replay.rs
│   ├── enrollment.rs
│   ├── policy.rs
│   ├── diagnostics.rs
│   └── error.rs
└── Cargo.toml
```

Adapters:

```text
comm-bootstrap-qr
comm-bootstrap-nfc
```

Platform UI adapters remain in:

```text
comm-platform-android
comm-platform-ios
comm-ui
```

---

# 151. QR Adapter

Responsibilities:

```text
canonical bytes
QR-safe encoding
URI wrapper
decode/validate framing
```

Does not authorize identity.

---

# 152. NFC Adapter

Responsibilities:

```text
NDEF mapping
record validation
bootstrap bytes
```

Does not authorize identity.

---

# 153. Bootstrap Service API

```rust
pub trait BootstrapService {
    async fn create_invitation(
        &self,
        purpose: BootstrapPurpose,
        policy: BootstrapPolicy,
    ) -> Result<BootstrapInvitationHandle, BootstrapError>;

    async fn accept_invitation(
        &self,
        invitation: BootstrapInvitation,
    ) -> Result<BootstrapSessionHandle, BootstrapError>;
}
```

---

# 154. Invitation Handle

```rust
pub struct BootstrapInvitationHandle {
    pub session_id: BootstrapSessionId,
}
```

Operations:

```text
render QR
write NFC
cancel
status
```

---

# 155. Session Handle

Operations:

```text
verification code
approve
reject
status
```

High-level UI should not manipulate raw keys.

---

# 156. Bootstrap Policy

```rust
pub struct BootstrapPolicy {
    pub lifetime: Duration,
    pub single_use: bool,
    pub require_sas: bool,
    pub allowed_transports: TransportSet,
}
```

Purpose can impose stricter minimums.

---

# 157. Link-Device Policy

Recommended:

```text
short lifetime
single use
SAS or strong out-of-band confirmation
explicit authorizer approval
```

---

# 158. Contact Policy

Could allow:

```text
QR proof + mutual confirmation
```

with session-only or persistent trust choice.

---

# 159. File-Session Policy

Recommended:

```text
short lifetime
single use
session-scoped authority
no account permissions
```

---

# 160. Enrollment Policy

Organization controls:

```text
role
expiry
issuer
allowed infrastructure
```

---

# 161. UI Flow — Link Device

```text
Existing device:
Settings
→ Devices
→ Link new device
→ QR displayed

New device:
Link existing account
→ Scan QR
→ Secure connection
→ Compare code

Both:
Confirm

Existing device:
Approve requested permissions

System:
Commit DeviceAdded
→ sync minimum state
```

---

# 162. UI Flow — Add Contact

```text
Show My Verification QR
       ↓
Other user scans
       ↓
Both see identity + verification
       ↓
Confirm
       ↓
Contact marked Verified
```

---

# 163. UI Flow — Nearby File

```text
Send nearby
→ Show/scan short-lived QR
→ connect
→ file offer
→ transfer
→ session expires
```

---

# 164. Dioxus Boundary

Dioxus owns:

```text
screens
QR rendering component
verification UI
permission explanation
```

Rust bootstrap service owns all state/security.

---

# 165. Android Kotlin Boundary

Kotlin owns only:

```text
camera plumbing
NFC APIs
biometric APIs
lifecycle integration
```

No trust logic.

---

# 166. Headless API

CLI:

```text
comm link-device --show-qr
comm enroll-node --show-qr
```

Useful for server/embedded nodes.

---

# 167. Recovery CLI

Advanced:

```text
comm bootstrap list-pending
comm bootstrap cancel <id>
```

No secret dump.

---

# 168. Diagnostics

Advanced view:

```text
Purpose: LinkOwnDevice
State: VerificationPending
Expires in: 01:23
Transport: BLE
Peer identity: authenticated
```

Never display raw private key material.

---

# 169. Metrics

Safe metrics:

```text
bootstrap started
completed
expired
verification failed
transport failed
purpose class
```

No QR token values.

---

# 170. Security Invariants

1. QR/NFC alone never grants account authority.
2. Invitation purpose cannot change.
3. Expired invitations cannot authorize.
4. Consumed single-use invitations cannot authorize twice.
5. New device private key is generated on the new device.
6. Device link requires an authorized existing identity.
7. Device is linked only after durable signed authorization.
8. SAS derives from the authenticated transcript.
9. SAS mismatch is never bypassed silently.
10. Transport switching does not change bootstrap identity.
11. Crash after commit produces exactly one authorization.
12. Ad-hoc file pairing cannot escalate into account access.
13. Stable personal identity is not unnecessarily embedded in private QR/NFC payloads.

---

# 171. Initial Production Scope

Implement first:

```text
BootstrapInvitationV1
QR encode/decode
short-lived single-use sessions
ephemeral key exchange
transcript hash
numeric SAS
own-device linking
face-to-face contact verification
ad-hoc file session
BLE/LAN/Iroh handoff
consumed-session tombstones
crash-safe authorization commit
```

Then:

```text
NFC
organization enrollment
headless node enrollment
word/emoji SAS
```

Defer initially:

```text
mass offline multi-use enrollment
complex proximity proofs
custom cryptographic handshake inventions
```

---

# 172. Implementation Phases

## Phase 1 — Bootstrap Core

```text
session ID
purpose
invitation
expiry
single-use
```

## Phase 2 — QR

```text
canonical encoding
QR wrapper
scanner integration
```

## Phase 3 — Secure Handshake

```text
ephemeral keys
transcript
SAS
confirmation
```

## Phase 4 — Device Linking

```text
authorization
DeviceAdded
minimum state sync
```

## Phase 5 — Contact / File Sessions

```text
verified contact
session-only file trust
```

## Phase 6 — NFC

```text
NDEF adapter
tap flow
```

## Phase 7 — Organization / Headless

```text
enrollment tokens
role policy
terminal QR
```

## Phase 8 — Hardening

```text
fuzzing
replay
MITM
crash
race
flood
real-device tests
```

---

# 173. Definition of Done

Part 15 is complete when:

- QR and NFC share one semantic bootstrap model
- bootstrap invitations are short-lived and bounded
- high-trust invitations can be single-use
- replay after successful use is rejected
- QR/NFC payload contains no private key
- a secure ephemeral handshake follows bootstrap
- transcript-derived SAS verification exists
- purpose confusion is cryptographically prevented
- device linking requires an already authorized device
- new device generates its own long-term private key
- DeviceAdded authorization is durable before device becomes trusted
- interrupted pairing cannot create a half-linked device
- contact verification cannot grant device authority
- ad-hoc file pairing can remain session-only
- pairing works without Internet when local transport exists
- pairing can move between BLE/LAN/Wi-Fi/Iroh paths
- Android/iOS camera/NFC code contains no trust policy
- consumed invitations survive crash/restart where required
- fuzz, replay, MITM, race, crash, flood, and offline tests exist

---

# 174. Relationship to Earlier Parts

Part 15 builds on:

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
```

It directly supports:

```text
16 — Daemon & Headless Runtime
17 — Emergency Priority Architecture
18 — Network Diagnostics & Path Visualization
19 — C ABI / FFI
20 — Embedded Linux Node
22 — Third-Party Protocol Extensions
23 — External Interoperability Suite
24 — Plugin / Module Ecosystem
```

---

# 175. Final Architecture

```text
                 USER INTENT
                     │
          ┌──────────┼───────────┐
          │                      │
       Show QR                 NFC Tap
          │                      │
          └──────────┬───────────┘
                     │
            BootstrapInvitation
                     │
              Ephemeral Key
                     │
               Rendezvous
                     │
       ┌─────────────┼─────────────┐
       │             │             │
      BLE           LAN          Iroh
       │             │             │
       └─────────────┼─────────────┘
                     │
           Secure Handshake
                     │
            Transcript Hash
                     │
             SAS Verification
                     │
            Identity Binding
                     │
              Authorization
                     │
        Durable Domain Commit
          ┌──────────┼──────────┐
          │          │          │
     DeviceAdded  Contact    File Session
                  Verified
```

---

# 176. Final Principle

The bootstrap system should make this flow safe:

```text
A new phone has no Internet.

The existing phone displays a QR code.

The new phone scans it.

The phones find each other over BLE.

They establish an ephemeral encrypted session.

Both screens show the same verification code.

The user confirms.

The existing authorized device signs and durably commits
the new device authorization.

Only then does the new phone receive account state.

Later Wi-Fi becomes available,
and history synchronization moves to the faster path.
```

The QR code is not the account credential.

NFC is not the account credential.

Physical proximity is not the account credential.

The actual trust chain is:

```text
intentional bootstrap
+
authenticated cryptographic transcript
+
human/policy verification
+
authorized identity signature
+
durable commit
```

That separation is what makes QR/NFC pairing safe enough for a reusable, offline-capable, multi-device P2P communication platform.
