# Part 28 — Production Security, E2EE, Key Management, Abuse Resistance & Privacy Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 28  
**Primary language:** Rust  
**Primary goals:** end-to-end confidentiality, forward secrecy, multi-device security, device revocation, offline/DTN-safe encryption, group security, secure local key storage, metadata minimization, abuse resistance, incident recovery, and auditable trust decisions across the complete platform.

---

# 1. Purpose

The platform already has Iroh/QUIC transport security, multi-device identity, DTN, files, proximity, QR/NFC bootstrap, emergency traffic, plugins, WASM, FFI, embedded nodes, Android media, audio DSP, and reproducible release tooling.

A production messenger still needs an **application-level security layer**. Transport encryption protects one connection; it does not by itself guarantee that data remains confidential when stored by a relay, carried by DTN nodes, copied from a server database, restored from a backup, or delivered to several devices.

The governing rule is:

> **Transport security protects the path; application E2EE protects the conversation regardless of path.**

---

# 2. Security Layers

```text
Application E2EE
      ↓
Conversation / Group Security
      ↓
Authenticated Device Sessions
      ↓
Iroh / QUIC Transport Security
      ↓
Internet / LAN / BLE / Relay / DTN
```

Local protections:

```text
OS Secure Key Store
Encrypted Durable State
Capability Boundaries
WASM / Plugin Sandbox
OS Process Sandbox
Signed Updates
```

---

# 3. Threat Model

Assume that any of these can be hostile or compromised:

```text
relay server
DTN carrier
public network
nearby Bluetooth peer
server database
third-party plugin
external FFI caller
stolen device
old backup
```

Trust only:

```text
current unlocked local device
explicitly authorized devices
verified contacts
explicit organization/authority keys
```

---

# 4. Required Security Properties

```text
confidentiality
integrity
authentication
forward secrecy
post-compromise recovery where practical
replay resistance
device revocation
key separation
metadata minimization
least privilege
crash-safe key evolution
```

---

# 5. Identity Hierarchy

Keep distinct:

```text
Account Identity
Device Identity
Session Identity
Conversation Keys
Group Epoch Keys
File Content Keys
Transport Keys
Backup Keys
```

Never reuse one key for unrelated purposes.

---

# 6. Account Identity

A long-lived logical account root:

```rust
pub struct AccountId([u8; 32]);
```

The account/root key should be used rarely, mainly for:

```text
authorizing devices
revoking devices
account recovery
high-value trust changes
```

It should not sign every message.

---

# 7. Device Identity

Every device gets an independent keypair:

```rust
pub struct DeviceId([u8; 32]);
```

A device certificate binds the device to the account.

```rust
pub struct DeviceCertificate {
    pub account: AccountId,
    pub device: DeviceId,
    pub device_public_key: PublicKey,
    pub issued_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub signature: Signature,
}
```

---

# 8. Device Key Storage

Prefer non-exportable platform-backed keys where practical:

```text
Android → Android Keystore
Apple → Keychain / Secure Enclave where appropriate
Windows → DPAPI / platform key facilities
Linux → TPM / Secret Service / encrypted software keystore
Embedded Linux → TPM when available
```

Rust owns the abstraction and policy; platform adapters own the unavoidable OS calls.

---

# 9. Secure Key Store Trait

```rust
pub trait SecureKeyStore {
    fn generate_device_key(
        &self,
        policy: KeyPolicy,
    ) -> Result<KeyHandle, KeyStoreError>;

    fn sign(
        &self,
        key: &KeyHandle,
        message: &[u8],
    ) -> Result<Signature, KeyStoreError>;
}
```

Prefer opaque `KeyHandle`s over exporting raw private-key bytes.

---

# 10. Memory Hygiene

Secret material should use:

```text
zeroizing containers
minimal cloning
short lifetimes
no debug formatting
no telemetry
```

Never log:

```text
private keys
session keys
file content keys
backup keys
ratchet secrets
```

---

# 11. One-to-One E2EE

A direct conversation needs:

```text
mutual identity binding
ephemeral key agreement
forward secrecy
out-of-order tolerance
replay protection
key rotation
post-compromise recovery where practical
```

Use a ratcheting abstraction rather than a static shared symmetric key.

```rust
pub trait ConversationRatchet {
    fn encrypt(
        &mut self,
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<EncryptedMessage, CryptoError>;

    fn decrypt(
        &mut self,
        message: &EncryptedMessage,
        aad: &[u8],
    ) -> Result<Vec<u8>, CryptoError>;
}
```

---

# 12. Forward Secrecy

Compromise of a long-term device key later should not automatically reveal old conversation traffic.

Ephemeral key agreement and ratcheting should ensure old message keys are discarded as the session advances.

---

# 13. Post-Compromise Recovery

After fresh authenticated key material is exchanged, future traffic should recover security even if an older session secret was exposed.

This property should be designed explicitly, not assumed.

---

# 14. Secure Message Envelope

```rust
pub struct SecureMessageEnvelope {
    pub conversation: ConversationId,
    pub sender_device: DeviceId,
    pub message_id: MessageId,
    pub epoch: SecurityEpoch,
    pub counter: u64,
    pub ciphertext: Bytes,
    pub authentication: AuthenticationTag,
}
```

The exact wire DTO must be versioned and canonical.

---

# 15. Associated Data

Authenticate metadata that must not be altered:

```text
conversation ID
message ID
sender device
protocol version
message type
security epoch
```

Only expose routing fields that transports actually need.

---

# 16. Replay Protection

Use:

```text
message IDs
ratchet counters
bounded replay windows
security epochs
```

The same ciphertext arriving through direct, relay, DTN, and multipath must remain one logical message.

---

# 17. Nonce Safety

Nonce uniqueness is mandatory.

Do not create ad-hoc nonce schemes. Use a well-reviewed AEAD construction and protocol state that guarantees safe nonce use.

---

# 18. Multi-Device Delivery

A logical account may have:

```text
phone
desktop
tablet
embedded gateway
```

Every device is independently authorized.

For small device sets, a simple model is:

```text
one logical message
→ recipient device envelopes
```

For large groups or many devices, use efficient group security.

---

# 19. New Device Enrollment

When adding a device:

```text
existing trusted device / account authority
→ authenticates new device
→ signs device certificate
→ establishes encrypted device-sync channel
```

Historical access policy must be explicit:

```text
future only
selected history
full history
```

---

# 20. Device Revocation

Lost or compromised device:

```text
revoke DeviceId
rotate affected security state
stop accepting new sessions
propagate revocation
```

```rust
pub struct DeviceRevocation {
    pub account: AccountId,
    pub device: DeviceId,
    pub revoked_at: Timestamp,
    pub reason: RevocationReason,
    pub signature: Signature,
}
```

---

# 21. Revocation Propagation

Use all available paths:

```text
direct P2P
relay
DTN
organization policy
offline signed file/QR where necessary
```

A disconnected peer may not learn instantly, so shorter-lived device credentials or security epochs can reduce risk.

---

# 22. Security Epoch

```rust
pub struct SecurityEpoch(pub u64);
```

Advance after events such as:

```text
device revocation
group membership change
identity recovery
major security incident
```

---

# 23. Device Clone Detection

A restored backup must not silently create two indistinguishable active device instances.

Use:

```rust
pub struct DeviceInstanceId([u8; 16]);
```

and detect concurrent/restored clones.

---

# 24. Backup Restore Safety

Restoring stale ratchet state can cause:

```text
message-key reuse
replay confusion
nonce reuse
```

Therefore restore must force:

```text
new device incarnation
or
fresh authenticated rekey
```

rather than blindly continuing old ratchet state.

---

# 25. Group Security

Group messaging needs independent group state.

```rust
pub struct GroupSecurityState {
    pub group_id: GroupId,
    pub epoch: GroupEpoch,
    pub members: Vec<GroupMemberDevice>,
}
```

Membership changes must advance group epoch.

---

# 26. Group Membership Changes

On:

```text
add member
remove member
revoke member device
```

rotate future group secrets.

A removed member must not decrypt future messages.

A new member's historical access must be explicit.

---

# 27. Group Authorization

Cryptographic membership changes must be signed/authorized.

Group-admin UI roles and cryptographic device trust are related but not identical concepts.

---

# 28. File Encryption

Part 05 blobs should be independently encrypted.

```text
file
 ↓
random file content key
 ↓
chunk encryption
 ↓
encrypted blob/chunks
 ↓
transport
```

The message carries only an encrypted/wrapped content key to authorized recipients.

---

# 29. File Content Key

```rust
pub struct FileContentKey(SecretBytes);
```

Use separate derivation/context from message keys.

---

# 30. Chunk Encryption

Each chunk should be independently authenticated and bound to:

```text
file ID
chunk index
encryption version
```

This supports:

```text
resume
reordering
partial download
parallel paths
```

without loading/re-encrypting the entire file.

---

# 31. Metadata Encryption

Where practical, encrypt:

```text
file name
caption
MIME metadata
thumbnail
document metadata
```

Routing infrastructure should see only what it needs.

---

# 32. DTN Security

DTN relays should carry opaque ciphertext.

They may need limited routing metadata:

```text
bundle ID
expiry
priority
size
routing token
```

They should not need message/file plaintext.

---

# 33. Relay Security

Relay infrastructure can observe network metadata, but should not receive application plaintext.

Do not claim full metadata anonymity; relays can still infer timing and traffic volume.

---

# 34. Metadata Minimization

Avoid unnecessary exposure of:

```text
stable account IDs
contact graph
precise location
file names
message types
raw device identifiers
```

Use scoped/pseudonymous identifiers where possible.

---

# 35. Scoped Peer IDs

For plugins/extensions:

```text
same peer
→ different pseudonym per extension
```

This prevents easy cross-plugin correlation.

---

# 36. Proximity Privacy

Bluetooth/LAN discovery should advertise rotating ephemeral discovery identifiers, not stable account/device IDs.

Identity becomes known only after authenticated handshake.

---

# 37. QR / NFC Bootstrap Security

Part 15 pairing must bind:

```text
account/device identity
ephemeral handshake keys
protocol versions
SAS
```

into one authenticated transcript.

QR/NFC proximity alone is not cryptographic trust.

---

# 38. SAS Verification

Short Authentication String may be:

```text
numeric code
word sequence
emoji sequence
```

Both devices derive the same value from the authenticated transcript.

---

# 39. Pairing Token Security

Pairing invitations should be:

```text
short-lived
single-use
rate-limited
cryptographically bound
```

Replay must fail.

---

# 40. Peer Trust States

```rust
pub enum PeerTrustState {
    Unknown,
    Seen,
    Verified,
    Trusted,
    Revoked,
    Blocked,
}
```

---

# 41. Trust Sources

Trust can come from:

```text
QR/SAS verification
NFC bootstrap
organization certificate
existing verified device
manual fingerprint comparison
```

Trust is not automatically transitive.

---

# 42. Identity Change UX

Routine session-key rotation should be invisible.

A root/account identity change should trigger a strong warning.

A newly added device should trigger a security event on existing trusted devices.

---

# 43. Safety Fingerprint

Provide an advanced verification fingerprint/safety number for high-assurance contacts.

---

# 44. Key Separation

Use distinct cryptographic domains:

```text
comm/msg/v1
comm/file/v1
comm/dtn/v1
comm/device-sync/v1
comm/backup/v1
comm/authority/v1
```

Never interpret a signature from one domain as valid in another.

---

# 45. Typed Signing

Avoid generic:

```text
sign arbitrary bytes
```

for sensitive account/root keys.

Use typed signing contexts:

```rust
pub enum SigningContext {
    DeviceCertificate,
    DeviceRevocation,
    AuthorityAlert,
    GroupMembership,
    RecoveryOperation,
}
```

---

# 46. Crypto Primitive Policy

Use mature reviewed Rust cryptography libraries.

Do not implement custom:

```text
cipher
hash
KDF
signature primitive
random generator
```

unless there is an extraordinary audited reason.

---

# 47. Security Suite

Start with one strong suite:

```rust
pub enum SecuritySuite {
    Suite1,
}
```

Authenticated negotiation binds:

```text
supported suites
chosen suite
protocol versions
capabilities
```

into the handshake transcript.

---

# 48. Downgrade Resistance

An attacker must not be able to strip modern capabilities or force a weaker security version silently.

Protocol/capability negotiation is authenticated.

---

# 49. Secure Randomness

Use OS CSPRNG through trusted Rust/platform facilities.

If secure randomness fails:

```text
fail securely
```

Never create predictable identity/session keys.

---

# 50. Local Database Security

E2EE protects remote/durable ciphertext.

Local encryption protects against stolen storage.

Classify:

```text
public metadata
private metadata
message ciphertext
optional plaintext cache
search index
keys
```

---

# 51. Recommended Local Storage Model

Prefer durable storage of:

```text
message ciphertext
encrypted attachment metadata
```

Decrypt into process memory/UI as needed.

If plaintext cache/search index exists, make its local-security implications explicit.

---

# 52. Full-Text Search Risk

A plaintext search index can reveal conversation text locally even when the message store is encrypted.

Options:

```text
encrypted index
bounded derived index
documented local plaintext index
```

Do not ignore the trade-off.

---

# 53. Notification Privacy

Support:

```text
full preview
sender only
generic new-message notification
```

User chooses lock-screen exposure.

---

# 54. Backup Architecture

Backups are a major security boundary.

Never store raw root/device secrets in ordinary unencrypted backups.

---

# 55. Backup Modes

```rust
pub enum BackupMode {
    DeviceState,
    AccountRecovery,
    ConversationArchive,
}
```

---

# 56. Hardware-Bound Keys

A non-exportable hardware-backed device key may intentionally not be restorable.

After restore:

```text
create new device identity
re-authorize
```

---

# 57. Account Recovery

Possible mechanisms:

```text
trusted existing device
high-entropy recovery key
recovery mnemonic
recovery file/QR
organization-managed recovery
```

A 6-digit PIN is not a sufficient standalone E2EE recovery secret.

---

# 58. Private vs Managed Accounts

```rust
pub enum AccountSecurityMode {
    Private,
    OrganizationManaged,
}
```

Managed recovery/escrow must be explicit and auditable.

Never build a hidden universal master key.

---

# 59. Cloud Backup

If offered:

```text
server stores ciphertext
```

The server should not be able to reset a normal account password and thereby decrypt E2EE data.

---

# 60. Abuse Resistance

Security must protect not just cryptographic secrets but resources.

Protect:

```text
CPU
RAM
storage
battery
radio
notifications
pairing UI
```

---

# 61. Cheap Validation First

Incoming untrusted work:

```text
length check
version check
rate limit
duplicate check
authorization class
then expensive crypto
```

---

# 62. Unknown Peer Limits

Apply separate quotas for:

```text
handshakes
message requests
file offers
pairing attempts
DTN bundles
signature verification
```

---

# 63. Message Request Inbox

Unknown-sender messages should enter a constrained request model rather than immediately consuming full trusted-user resources.

---

# 64. Unknown File Offers

Do not auto-download large attachments from unknown peers.

---

# 65. Blocklist

Blocked peers should be rejected before expensive protocol work wherever possible.

---

# 66. Sybil Resistance

A decentralized system cannot fully prevent arbitrary fake identities.

Mitigate with:

```text
trust states
verification
organization credentials
quotas
rate limits
```

Do not pretend identity creation itself prevents Sybil attacks.

---

# 67. Emergency Abuse

Part 17 priority remains separately authorized.

A peer or plugin cannot gain `AuthorityCritical` merely by requesting it.

---

# 68. Security Audit Log

Record high-value security events:

```text
device added
device revoked
identity changed
recovery used
backup restored
plugin permissions changed
authority credential changed
```

No message plaintext.

---

# 69. Audit Integrity

Use append-only logical events.

Optional future improvement:

```text
hash-chained audit records
```

for organization deployments.

---

# 70. Plugin Security Boundary

Plugins do not receive core conversation plaintext or keys by default.

Protocol extensions see their own payloads only.

WASM remains capability-limited.

Untrusted native plugins remain out-of-process.

---

# 71. FFI Security Boundary

Part 19 callers are untrusted.

Validate:

```text
handles
pointer/length relationships
enum values
buffer ownership
threading
```

before reaching security state.

---

# 72. Embedded Node Security

Field nodes may be physically captured.

Use:

```text
signed updates
read-only root filesystem
TPM where useful
restricted admin plane
E2EE relay payloads
```

A stolen relay disk should primarily contain ciphertext.

---

# 73. Update Security

App, daemon, and embedded updates require trusted signatures.

A compromised CDN/registry alone should not be able to install arbitrary code.

---

# 74. Rollback Protection

Maintain a minimum accepted security build/protocol version after serious vulnerabilities.

```rust
pub struct MinimumSecurityVersion(pub u32);
```

---

# 75. Crash-Safe Ratchets

Security state must be persisted transactionally.

Send:

```text
derive next state
encrypt
persist new ratchet state + ciphertext atomically
then report durable acceptance
```

---

# 76. Nonce / Key Reuse After Crash

A crash must not cause the same one-time message key/nonce to be reused for a different plaintext.

This is a hard invariant.

---

# 77. Receive Transaction

```text
validate
decrypt
advance ratchet
persist ratchet + message atomically
```

Duplicate delivery after restart must remain safe.

---

# 78. Ratchet Desynchronization

Do not silently reset peer identity.

Recovery:

```text
fresh authenticated session
new ratchet
preserve verified identity relationship
```

---

# 79. Key Store Failure

If key store is unavailable or corrupted:

```text
do not silently create a new identity
```

Enter a recovery/security-safe mode.

---

# 80. Security Safe Mode

Allows:

```text
diagnostics
backup/recovery actions
device management
```

while preventing unsafe message sending.

---

# 81. File Dedup vs Encryption

Global deduplication can leak equality/content relationships.

Default should favor privacy:

```text
per-account/per-conversation encrypted blobs
```

rather than server-visible global plaintext hashes.

Local dedup remains possible.

---

# 82. Emergency Messages

Private SOS:

```text
E2EE
```

Public authority alert:

```text
signed/authenticated
possibly intentionally public
```

Targeted authority message:

```text
E2EE + authority signature
```

---

# 83. Abuse Reporting

Because server cannot automatically read E2EE messages, reporting should be explicit.

User may choose to attach:

```text
specific reported message
sender identity metadata
relevant cryptographic proof
```

Do not upload entire conversations automatically.

---

# 84. Security Diagnostics

Expose:

```text
peer trust status
device certificate state
revocation state
security suite
security epoch
handshake failures
```

Never expose keys.

---

# 85. Failure Taxonomy

```rust
pub enum SecurityFailure {
    IdentityMismatch,
    RevokedDevice,
    InvalidSignature,
    Replay,
    ExpiredCredential,
    UnsupportedSuite,
    DowngradeDetected,
    KeyStoreFailure,
    RatchetDesync,
    UnauthorizedRole,
    CloneDetected,
}
```

---

# 86. Threat-Model Documentation

Maintain a dedicated:

```text
THREAT-MODEL.md
```

covering:

```text
assets
adversaries
trust assumptions
attack surfaces
mitigations
residual risks
```

---

# 87. Security ADRs

Write architecture decision records for:

```text
identity hierarchy
ratchet choice
group encryption design
recovery model
key-store model
backup model
```

Security decisions should not live only in code.

---

# 88. Responsible Disclosure

Ship:

```text
SECURITY.md
supported versions
security contact
reporting process
```

---

# 89. Vulnerability Response

```text
report
→ triage
→ reproduce
→ patch
→ regression test
→ coordinated release
→ revoke/minimum-version policy if necessary
```

---

# 90. Account Root Compromise

This is a high-severity event.

Recovery may require:

```text
new account root
device reauthorization
contact identity-change warning
group rekey
```

---

# 91. Device Compromise

Prefer revoking only the affected device and rotating relevant session/group state.

---

# 92. Conversation Secret Compromise

A fresh authenticated ratchet/key exchange should restore future confidentiality where the protocol supports post-compromise recovery.

---

# 93. Cryptographic Test Vectors

Part 23 should add public test-only vectors for:

```text
device certificates
revocations
session handshakes
secure envelopes
file-key wrapping
group epoch changes
bootstrap transcript
```

---

# 94. Fuzzing

Part 10 should fuzz:

```text
secure envelope parser
certificate parser
revocation parser
group message parser
bootstrap transcript parser
encrypted file metadata parser
```

Malformed ciphertext should return a typed error, never panic.

---

# 95. State-Machine Fuzzing

Generate sequences:

```text
pair
send
duplicate
reorder
rotate
revoke
restore
reconnect
group-add
group-remove
```

---

# 96. Core Security Properties Tests

Required invariants:

```text
revoked device cannot create new trusted session
removed group member cannot decrypt new epoch
duplicate ciphertext creates one logical message
downgrade attempts fail
old backup cannot silently reuse unsafe ratchet state
malicious plugin cannot read identity key
relay/DTN node cannot decrypt user payload
```

---

# 97. Database Theft Test

Copy server-side storage.

Attacker must not obtain:

```text
message plaintext
file plaintext
private user keys
```

---

# 98. Relay Compromise Test

Relay sees only:

```text
ciphertext
necessary routing metadata
traffic timing/volume
```

---

# 99. DTN Carrier Compromise Test

Same confidentiality property.

---

# 100. Pairing MITM Test

MITM/relay attempt must produce a SAS/transcript mismatch or authentication failure.

---

# 101. Pairing Replay Test

Old invitation/token is rejected.

---

# 102. Group Removal Test

After group epoch advances:

```text
removed device
```

cannot decrypt newly generated messages.

---

# 103. Restore Clone Test

Two active installations derived from the same stale backup must trigger clone/rekey handling rather than silently sharing ratchet identity.

---

# 104. Security Performance

Benchmark:

```text
message encrypt/decrypt
ratchet advancement
device fan-out
group epoch change
file chunk encryption
file-key wrapping
```

Security should remain well below network/media bottlenecks.

---

# 105. Streaming File Security

Never buffer an entire large file solely for encryption.

Use:

```text
read chunk
encrypt chunk
hash/authenticate
write/send
```

---

# 106. Security Profiles

```rust
pub enum SecurityProfile {
    Standard,
    HighSecurity,
    ManagedEnterprise,
}
```

---

# 107. Standard

```text
E2EE
device trust
platform secure store
normal backup policy
metadata minimization
```

---

# 108. High Security

May enable:

```text
stricter identity warnings
shorter credential lifetimes
no lock-screen previews
restricted backup
mandatory device verification
```

---

# 109. Managed Enterprise

May add explicitly disclosed:

```text
organization device policy
organization certificates
audit integration
optional organization recovery/escrow
```

---

# 110. No Silent Security-Mode Change

Changing a private account into organization-managed/escrowed mode must be explicit.

---

# 111. Public Security APIs

High-level modules should use:

```text
SecureMessenger
DeviceTrustManager
GroupSecurityManager
SecureFileManager
RecoveryManager
```

rather than calling crypto primitives throughout the codebase.

---

# 112. Suggested Workspace

```text
crates/
├── comm-security-core/
├── comm-security-identity/
├── comm-security-session/
├── comm-security-ratchet/
├── comm-security-group/
├── comm-security-files/
├── comm-security-keystore/
├── comm-security-android/
├── comm-security-apple/
├── comm-security-linux/
├── comm-security-windows/
├── comm-security-backup/
├── comm-security-audit/
├── comm-security-abuse/
└── comm-security-testkit/
```

---

# 113. `comm-security-core`

Owns:

```text
security suites
domain separation
security IDs
canonical security DTOs
errors
```

---

# 114. `comm-security-identity`

Owns:

```text
account/device certificates
revocation
trust states
device lifecycle
```

---

# 115. `comm-security-session`

Owns:

```text
authenticated handshake
ephemeral key agreement
transcript binding
session establishment
```

---

# 116. `comm-security-ratchet`

Owns:

```text
one-to-one forward-secret message state
out-of-order keys
post-compromise recovery
```

---

# 117. `comm-security-group`

Owns:

```text
group membership
epochs
group-key updates
```

---

# 118. `comm-security-files`

Owns:

```text
content keys
chunk encryption
attachment metadata protection
key wrapping
```

---

# 119. `comm-security-keystore`

Platform-neutral key-store abstraction.

Platform crates implement the OS-specific backends.

---

# 120. `comm-security-backup`

Owns:

```text
encrypted backups
recovery keys
restore epochs
clone detection
```

---

# 121. `comm-security-abuse`

Owns:

```text
unknown-peer quotas
blocklist
pairing limits
message requests
storage/CPU abuse protection
```

---

# 122. `comm-security-testkit`

Provides:

```text
test identities
fake secure store
deterministic test-only keys
malicious peer simulator
restore/clone scenarios
```

---

# 123. Implementation Phases

## Phase 1 — Identity / Device Trust

```text
account root
device certificates
device verification
revocation
```

## Phase 2 — One-to-One E2EE

```text
authenticated session
forward secrecy
ratchet
replay protection
```

## Phase 3 — Files / DTN

```text
content keys
chunk AEAD
opaque relay/DTN storage
```

## Phase 4 — Multi-Device

```text
device fan-out
history transfer
revoke/rekey
clone protection
```

## Phase 5 — Groups

```text
membership
epochs
group encryption
```

## Phase 6 — Secure Storage / Recovery

```text
platform keystore
encrypted backups
recovery
restore safety
```

## Phase 7 — Abuse Resistance

```text
unknown-peer budgets
pairing limits
request inbox
storage/CPU protection
```

## Phase 8 — Security UX / Audit

```text
identity changes
device list
security event log
diagnostics
```

## Phase 9 — Hardening

```text
fuzz
state-machine/property tests
external review
incident drills
```

---

# 124. Definition of Done

Part 28 is complete when:

- application E2EE protects messages independently of Iroh transport
- one-to-one messaging provides forward secrecy
- relay and DTN stores contain ciphertext rather than user plaintext
- every device has its own authorized identity
- device revocation is signed and propagated
- revoked devices cannot establish new trusted sessions
- multi-device delivery remains secure
- stale backup restoration cannot silently reuse unsafe ratchet state
- group membership changes rotate future group secrets
- removed members cannot decrypt new group messages
- files use independent content keys and streaming chunk encryption
- QR/NFC pairing is transcript-bound and replay-resistant
- secure stores use opaque key handles where possible
- unknown peers receive bounded CPU/storage/network budgets
- emergency priority does not bypass authentication
- plugins/WASM do not access private keys or conversations by default
- security negotiation resists downgrade
- security-critical DTOs are canonical and versioned
- malformed security data cannot panic
- security regression tests cover revocation, replay, downgrade, pairing MITM, group removal, backup clone, relay/DTN compromise, and plugin escape
- `THREAT-MODEL.md` and `SECURITY.md` exist before production launch

---

# 125. Relationship to Parts 1–27

Part 28 is cross-cutting and hardens:

```text
02 — Multi-Device Identity
04 — Offline Event Log
05 — Files / Blobs
06 — DTN
07 — Capability Negotiation
08 — Resource Limits
09 — Crash Recovery
10 — Fuzzing
11 — Relay Infrastructure
12 — Multipath
14 — Proximity
15 — QR/NFC Bootstrap
16 — Daemon
17 — Emergency Priority
18 — Diagnostics
19 — C ABI / FFI
20 — Embedded Linux
21 — Protocol Extensions
22 — WASM Components
23 — Interoperability
24 — Plugin Ecosystem
25 — Android Zero-Copy Media
26 — Audio
27 — Android Build / Release
```

---

# 126. Final Architecture

```text
                       APPLICATION
                            │
                            ▼
                    Security API Layer
          ┌─────────────────┼─────────────────┐
          │                 │                 │
       Identity          Sessions           Groups
          │                 │                 │
          └─────────────────┼─────────────────┘
                            │
                    E2EE / Ratchets
                            │
             ┌──────────────┼───────────────┐
             │              │               │
          Messages         Files           DTN
             │              │               │
             └──────────────┼───────────────┘
                            │
                       Iroh / QUIC
                            │
          Internet / Relay / LAN / BLE / Mesh
```

Local trust:

```text
OS Secure Store
      │
Account / Device Keys
      │
Session / Group Key Derivation
      │
Encrypted Durable State
```

---

# 127. Final Principle

The platform must remain secure even if:

```text
the relay is malicious
the DTN carrier is curious
the server database is copied
the network is hostile
a plugin is malicious
a device is lost
an old backup is restored
```

The security model therefore cannot stop at:

```text
"Iroh encrypts the connection."
```

The production model is:

```text
Application E2EE
+
Device Trust
+
Forward Secrecy
+
Key Rotation
+
Secure Local Key Storage
+
Crash-Safe Ratchets
+
Abuse Resistance
+
Metadata Minimization
+
Auditable Recovery
```

This converts the P2P communication stack from a secure transport system into a production-grade secure messaging foundation.
