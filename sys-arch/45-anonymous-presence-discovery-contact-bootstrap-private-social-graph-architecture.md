# Core System Architecture Part 45 — Anonymous Presence, Discovery, Contact Bootstrap & Private Social Graph Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 45  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–44  

**Primary purpose:** define the complete privacy-preserving social-graph architecture for SIAR, including contact bootstrap, invitation capabilities, QR/NFC/local exchange, anonymous presence, private contact discovery, graph-hiding identifiers, rendezvous/contact mailboxes, multi-device behavior, blocking/revocation, abuse resistance, recovery, and provider-neutral Rust interfaces.

---

# 1. Purpose

A communication system can encrypt messages perfectly and still expose a user's social graph.

Presence and discovery systems commonly reveal:

```text
who has an account
who is online
who talks to whom
who shares contacts
which phone numbers/emails belong to users
which devices belong to one account
when relationships are created
```

Part 45 defines a model where:

> **SIAR should establish and maintain relationships through scoped, privacy-preserving capabilities rather than a globally queryable social graph.**

---

# 2. Architectural Position

```text
User Identity / Account
        │
        ▼
Contact Bootstrap
        │
        ├── QR
        ├── NFC
        ├── Invite Link/Code
        ├── Existing Secure Contact
        └── Optional Private Discovery
        │
        ▼
Contact Capability Exchange
        │
        ▼
Private Social Graph
        │
        ├── Presence
        ├── Messaging
        ├── Calls
        └── Group Invitations
```

---

# 3. Governing Rules

Mandatory:

```text
social graph is local-first
global identity is not a provider-facing routing identifier
presence is capability-scoped
contact discovery is optional
address-book upload is not required
blocking revokes relationship capabilities
```

---

# 4. Core Identity Separation

Distinguish:

```text
Account Identity
Contact Identity
Relationship Identity
Presence Identity
Transport Identity
Mailbox Identity
```

---

# 5. Account Identity

Long-lived identity used for:

```text
trust
verification
multi-device account ownership
```

---

# 6. Contact Identity

The local representation of another user.

```rust
pub struct ContactId(pub [u8; 32]);
```

Local only.

---

# 7. Relationship Identity

Each relationship gets a scoped random ID.

```rust
pub struct RelationshipId(pub [u8; 32]);
```

---

# 8. Why Relationship IDs

Avoid exposing stable:

```text
AccountId
username
phone
email
```

to providers.

---

# 9. Presence Identity

Ephemeral/rotating identifier.

```rust
pub struct PresenceId(pub [u8; 32]);
```

---

# 10. Transport Identity

Anonymous routing uses its own identifiers.

---

# 11. Mailbox Identity

Anonymous mailbox IDs remain separate.

---

# 12. Social Graph Storage

Recommended local model:

```text
contacts
relationships
verification state
capabilities
presence policy
blocked relationships
```

---

# 13. No Global Contact Table Requirement

SIAR should not require:

```text
server knows every user-to-user relationship
```

---

# 14. Contact Bootstrap Modes

```rust
pub enum ContactBootstrapMode {
    Qr,
    Nfc,
    InviteCode,
    ExistingSecureChannel,
    PrivateDiscovery,
    ManagedDirectory,
}
```

---

# 15. QR Bootstrap

Best for in-person/private establishment.

---

# 16. NFC Bootstrap

Same principle with short-range exchange.

---

# 17. Invite Code

Can be shared through another channel.

---

# 18. Existing Secure Channel

A verified contact can introduce another capability.

---

# 19. Private Discovery

Optional service for users who want discoverability.

---

# 20. Managed Directory

Optional organization-managed environment.

Separate from public social discovery.

---

# 21. Contact Invite

```rust
pub struct ContactInvite {
    pub invite_id: ContactInviteId,
    pub public_identity: IdentityPublicMaterial,
    pub relationship_seed: SecretBytes,
    pub capabilities: ContactCapabilityOffer,
    pub expires_at: Timestamp,
}
```

---

# 22. Invite ID

```rust
pub struct ContactInviteId(pub [u8; 16]);
```

Random.

---

# 23. Invite Secret

Bearer secret.

Never log.

---

# 24. Invite Expiry

Bounded.

---

# 25. One-Time Invite

Preferred for sensitive bootstrap.

---

# 26. Multi-Use Invite

Possible for:

```text
public creator
business support
community organizer
```

with reduced privacy.

---

# 27. Invite Policy

```rust
pub enum ContactInvitePolicy {
    OneTime,
    BoundedUses(u16),
    UntilExpiry,
}
```

---

# 28. QR Payload

Contains only minimum bootstrap data.

---

# 29. No Full Profile Required

QR should not need:

```text
phone number
email
home address
```

---

# 30. QR Integrity

Signed/authenticated.

---

# 31. QR Confidentiality

Sensitive invite material can be encrypted if scan context supports it.

---

# 32. Contact Acceptance

Flow:

```text
scan/receive invite
→ validate
→ display identity
→ user accepts
→ derive relationship state
→ exchange scoped capabilities
```

---

# 33. Relationship Capability Bundle

```rust
pub struct RelationshipCapabilities {
    pub messaging: Option<MessageCapability>,
    pub presence: Option<PresenceCapability>,
    pub calling: Option<CallInvitationCapability>,
    pub file_transfer: Option<FileTransferCapability>,
}
```

---

# 34. Capability Independence

Hard rule.

Granting messaging does not automatically imply:

```text
presence
calls
files
```

---

# 35. Presence Capability

```rust
pub struct PresenceCapability(SecretBytes);
```

---

# 36. Presence Capability Scope

Bound to:

```text
relationship
device/account scope
expiry/rotation
```

---

# 37. Presence Semantics

Use coarse states:

```rust
pub enum PrivatePresenceState {
    Available,
    Away,
    Unknown,
}
```

---

# 38. Avoid False Offline

`Unknown` means:

```text
no reliable fresh presence information
```

not:

```text
definitely offline
```

---

# 39. Strict Anonymous Mode

Presence disabled by default.

---

# 40. Presence Privacy Levels

```rust
pub enum PresencePrivacy {
    Disabled,
    ContactsOnly,
    SelectedContacts,
    Coarse,
}
```

---

# 41. ContactsOnly

Presence available only to valid relationship capability holders.

---

# 42. SelectedContacts

Explicit allowlist.

---

# 43. Coarse

Avoid precise:

```text
last seen 22:14:08
```

---

# 44. Last-Seen Buckets

If enabled:

```text
recently
today
this week
unknown
```

---

# 45. No Exact Last-Seen by Default

Exact timestamps create correlation risk.

---

# 46. Presence Publication

Potential architecture:

```text
client
→ encrypted presence object
→ relationship mailbox/provider
```

---

# 47. Presence Object

```rust
pub struct EncryptedPresenceObject {
    pub presence_id: PresenceId,
    pub epoch: PresenceEpoch,
    pub ciphertext: Bytes,
    pub expires_at: Timestamp,
}
```

---

# 48. Presence Epoch

```rust
pub struct PresenceEpoch(pub u64);
```

---

# 49. Presence Rotation

Rotate:

```text
presence ID
encryption material
```

periodically.

---

# 50. Presence TTL

Short.

---

# 51. Expired Presence

Maps to:

```text
Unknown
```

---

# 52. Presence Delivery Models

```rust
pub enum PresenceDeliveryMode {
    PerRelationship,
    RelationshipMailbox,
    AnonymousTopic,
}
```

---

# 53. Per-Relationship

Strongest isolation, more bandwidth.

---

# 54. Relationship Mailbox

Efficient for small contact sets.

---

# 55. Anonymous Topic

Potential larger-scale method.

---

# 56. Provider Visibility

Provider should not know:

```text
Account A follows Account B
```

---

# 57. Relationship-Specific Presence IDs

Preferred.

---

# 58. Why

If same presence ID is shared to all contacts, provider can correlate the user's social graph.

---

# 59. Presence Fanout

For each authorized relationship:

```text
derive unique encrypted update
```

---

# 60. Cost

O(number of presence-enabled relationships).

---

# 61. Optimization

Batch publication through mixnet while preserving relationship-specific payloads.

---

# 62. No Global Presence Channel

Hard rule for strict architecture.

---

# 63. Presence Polling

Recipient fetch should integrate with Part 37 shaping.

---

# 64. Presence Push

Push provider can leak timing.

---

# 65. Strict Mode

Prefer:

```text
anonymous mailbox/poll
```

rather than conventional push.

---

# 66. Typing State

Typing is more sensitive than presence.

---

# 67. Typing Policy

```rust
pub enum TypingPrivacy {
    Disabled,
    EnabledPerConversation,
}
```

---

# 68. Strict Anonymous Mode

Typing disabled.

---

# 69. Discovery Problem

Users may want:

```text
find friends
find someone by username
find contacts
```

without exposing address books.

---

# 70. Discovery Modes

```rust
pub enum ContactDiscoveryMode {
    Disabled,
    ExactHandle,
    PrivateSetIntersection,
    ManagedDirectory,
}
```

---

# 71. Disabled

Best privacy.

---

# 72. Exact Handle

User explicitly searches known handle.

---

# 73. Private Set Intersection

Potential future.

Compare contacts without server learning raw entire address book.

---

# 74. Managed Directory

Enterprise/school/team environment.

---

# 75. Phone/Email Discovery

Should be opt-in.

---

# 76. Never Require Full Address Book Upload

Hard rule.

---

# 77. Hashing Phone Numbers Is Not Enough

Phone/email spaces are low entropy.

Simple hashes can be brute-forced.

---

# 78. Safer Discovery

Potential:

```text
OPRF
Private Set Intersection
rate-limited blind lookup
```

---

# 79. Do Not Invent Cryptography

Use reviewed PSI/OPRF protocol/library.

---

# 80. Discovery Server

Should learn minimum possible.

---

# 81. OPRF Concept

Client obtains blinded token for identifier.

Server cannot directly see raw identifier.

---

# 82. Discovery Token

```rust
pub struct DiscoveryToken(pub [u8; 32]);
```

---

# 83. Token Rotation

Protocol-dependent.

---

# 84. Exact Handle Discovery

Handle maps to contact bootstrap descriptor.

---

# 85. Handle Privacy

Public handle means discoverability tradeoff.

---

# 86. Handle Visibility

```rust
pub enum HandleVisibility {
    Private,
    ExactSearch,
    Public,
}
```

---

# 87. Exact Search

Not enumerable if possible.

---

# 88. Public

Suitable for:

```text
business
public creator
support identity
```

not maximum privacy.

---

# 89. Anti-Enumeration

Discovery service must prevent:

```text
download all usernames
probe all phone numbers
```

---

# 90. Defenses

```text
rate limiting
proof-of-work/captcha where appropriate
OPRF/PSI
exact-match API
abuse detection
```

---

# 91. Discovery Result

Return bootstrap descriptor, not social graph.

---

# 92. Bootstrap Descriptor

```rust
pub struct ContactBootstrapDescriptor {
    pub identity_public: IdentityPublicMaterial,
    pub invite_endpoint: OpaqueInviteEndpoint,
    pub expires_at: Timestamp,
    pub signature: IdentitySignature,
}
```

---

# 93. No Contact List Exposure

A lookup does not reveal who else is connected.

---

# 94. Relationship Establishment

Discovery only finds identity.

Relationship requires:

```text
request
acceptance
capability exchange
```

---

# 95. Contact Requests

```rust
pub struct ContactRequest {
    pub request_id: ContactRequestId,
    pub sender_identity: EncryptedIdentityIntroduction,
    pub requested_capabilities: ContactCapabilityOffer,
    pub expires_at: Timestamp,
}
```

---

# 96. Unknown Sender

Request arrives in:

```text
Requests Inbox
```

---

# 97. No Presence Before Acceptance

Hard rule.

---

# 98. No Auto File Download

Hard rule.

---

# 99. Request Abuse

Unknown users may spam requests.

---

# 100. Anti-Spam Controls

```text
request capability
rate limit
proof-of-work
invite-only mode
```

---

# 101. Privacy-Preserving Request Token

Could be issued via discovery/bootstrap.

---

# 102. Contact Request Policy

```rust
pub enum ContactRequestPolicy {
    Nobody,
    InviteOnly,
    KnownHandle,
    AnyoneRateLimited,
}
```

---

# 103. Block

Blocking must revoke relationship capabilities.

---

# 104. Blocked Relationship State

```rust
pub enum RelationshipState {
    Pending,
    Active,
    Muted,
    Blocked,
    Revoked,
}
```

---

# 105. Block Effects

Revoke/ignore:

```text
messaging capability
presence capability
call capability
file capability
```

---

# 106. Block Privacy

Do not necessarily send:

```text
you were blocked
```

---

# 107. Presence After Block

Becomes:

```text
Unknown
```

---

# 108. Unblock

Creates fresh capabilities.

---

# 109. Do Not Reuse Old Relationship Secrets

Hard rule.

---

# 110. Relationship Rotation

Capabilities may rotate periodically.

---

# 111. Relationship Epoch

```rust
pub struct RelationshipEpoch(pub u64);
```

---

# 112. Rotation Triggers

```text
device loss
contact re-verification
security event
manual reset
```

---

# 113. Verification

Separate:

```text
contact exists
identity verified
device verified
```

---

# 114. Trust Model

```rust
pub enum ContactTrust {
    Unverified,
    VerifiedIdentity,
    VerifiedDevice,
}
```

---

# 115. Familiarity Is Not Verification

Hard rule.

---

# 116. Verification Methods

```text
QR comparison
SAS
fingerprint
in-person
trusted introduction
```

---

# 117. Trusted Introduction

Contact A introduces B to C.

---

# 118. Introduction Privacy

A should not learn whether B and C later communicate unless explicitly informed.

---

# 119. Introduction Capability

```rust
pub struct ContactIntroduction {
    pub subject_identity: IdentityPublicMaterial,
    pub bootstrap_capability: SecretBytes,
    pub expires_at: Timestamp,
}
```

---

# 120. No Transitive Trust Automatically

Hard rule.

---

# 121. Private Social Graph

Local representation:

```rust
pub struct SocialGraph {
    pub relationships: Vec<RelationshipRecord>,
}
```

---

# 122. Relationship Record

```rust
pub struct RelationshipRecord {
    pub relationship_id: RelationshipId,
    pub contact_id: ContactId,
    pub state: RelationshipState,
    pub trust: ContactTrust,
    pub epoch: RelationshipEpoch,
    pub capabilities: RelationshipCapabilityRefs,
}
```

---

# 123. Provider Must Not Receive SocialGraph

Hard rule.

---

# 124. Server Sync

If multi-device sync is enabled:

```text
encrypt graph before server storage
```

---

# 125. Authoritative Graph

User/device E2EE state.

---

# 126. Server Role

Opaque encrypted sync blob/object.

---

# 127. Graph Sync Key

Separate from:

```text
message keys
backup keys
```

---

# 128. Multi-Device Contacts

All trusted devices may share:

```text
contact metadata
verification state
relationship policy
```

---

# 129. Device-Specific Capabilities

Some receiving capabilities remain device-specific.

---

# 130. Presence Across Devices

Need aggregation.

---

# 131. Multi-Device Presence Semantics

Account is:

```text
Available
```

if at least one authorized device is available.

---

# 132. Device Presence Hidden

Contacts need not know:

```text
which device
```

---

# 133. Device Count

Do not expose by default.

---

# 134. Presence Aggregator

Prefer client/account-controlled logic.

---

# 135. Presence Conflict

Use freshest valid signed update.

---

# 136. Presence Replay

Old state rejected by epoch/expiry.

---

# 137. Lost Device

Revoke device presence/call/message capabilities.

---

# 138. Device Revocation

Does not require deleting relationship.

---

# 139. Account Recovery

Recovered account should avoid reusing old transport/presence identifiers.

---

# 140. Fresh Relationship Transport State

Recommended after high-risk recovery.

---

# 141. Contact Reauthorization

Maximum privacy can require contacts to reapprove fresh capabilities.

---

# 142. Recovery Policy

```rust
pub enum SocialGraphRecoveryPolicy {
    RestoreEncryptedGraph,
    RestoreAndRotateCapabilities,
    ReauthorizeSensitiveContacts,
}
```

---

# 143. Maximum Privacy

Prefer:

```text
RestoreAndRotateCapabilities
```

or stronger.

---

# 144. Backup

Can include encrypted:

```text
contact names
notes
verification metadata
```

---

# 145. Backup Exclusions

Do not blindly restore:

```text
live presence secrets
single-use invites
transport session secrets
```

---

# 146. Contact Export

Privacy-sensitive.

---

# 147. Export Warning

Export may reveal:

```text
social graph
contact names
verification state
```

---

# 148. Contact Notes

Local only by default.

---

# 149. Profile Data

Contact profile may include:

```text
display name
avatar
status
```

---

# 150. Profile Privacy

Each field can be scoped.

---

# 151. Profile Field Visibility

```rust
pub enum ProfileVisibility {
    Nobody,
    Contacts,
    SelectedRelationships,
    Public,
}
```

---

# 152. Avatar Distribution

Encrypted/capability-gated for private profiles.

---

# 153. External Avatar URL

Avoid in strict mode.

---

# 154. Status Message

Potential metadata leak.

---

# 155. Strict Mode

Disable or relationship-encrypt.

---

# 156. Profile Update Fanout

Same privacy principle as presence.

---

# 157. Public Profiles

Explicitly trade privacy for discoverability.

---

# 158. Private Profiles

No global fetch.

---

# 159. Discovery Cache

Local.

---

# 160. Negative Lookup Cache

Bounded.

---

# 161. Discovery Privacy

Do not retain detailed search history server-side.

---

# 162. Local Search History

Optional.

---

# 163. Clear Search History

Supported.

---

# 164. Directory Queries

Use:

```text
exact-match
privacy-preserving protocol
```

not broad prefix enumeration in strict modes.

---

# 165. Username Prefix Search

High enumeration risk.

---

# 166. Recommendation

Do not support public prefix search in initial private architecture.

---

# 167. Nearby Discovery

Bluetooth/Wi-Fi proximity can leak physical co-location.

---

# 168. Nearby Mode

Separate explicit opt-in.

---

# 169. Nearby Identifier

Ephemeral rotating beacon.

---

# 170. Nearby Beacon

```rust
pub struct NearbyBootstrapBeacon(pub [u8; 16]);
```

---

# 171. Beacon Rotation

Frequent.

---

# 172. No Account ID in Beacon

Hard rule.

---

# 173. Proximity ≠ Trust

Hard rule.

---

# 174. Nearby Flow

```text
discover ephemeral beacon
→ user confirms device/person
→ secure bootstrap
→ optional verification
```

---

# 175. Bluetooth Privacy

OS may expose device metadata.

Minimize application-advertised identity.

---

# 176. Wi-Fi LAN Discovery

Same.

---

# 177. Local Discovery Scope

Do not automatically add nearby users.

---

# 178. NFC

Strong deliberate physical action.

Useful for verification/bootstrap.

---

# 179. QR Verification

Can combine:

```text
identity fingerprint
relationship bootstrap
```

but keep semantics clear.

---

# 180. Privacy-Preserving Discovery Provider Trait

```rust
#[async_trait]
pub trait PrivateContactDiscovery: Send + Sync {
    async fn lookup_handle(
        &self,
        handle: &ExactHandle,
    ) -> Result<Option<ContactBootstrapDescriptor>, DiscoveryError>;

    async fn private_match(
        &self,
        inputs: PrivateDiscoveryInput,
    ) -> Result<PrivateDiscoveryResult, DiscoveryError>;
}
```

---

# 181. Presence Service Trait

```rust
#[async_trait]
pub trait PrivatePresenceService: Send + Sync {
    async fn publish(
        &self,
        relationship: RelationshipId,
        state: PrivatePresenceState,
    ) -> Result<(), PresenceError>;

    async fn fetch(
        &self,
        relationship: RelationshipId,
    ) -> Result<PrivatePresenceState, PresenceError>;
}
```

---

# 182. Relationship Service Trait

```rust
pub trait RelationshipService {
    fn accept(
        &self,
        invite: ContactInvite,
    ) -> Result<RelationshipId, RelationshipError>;

    fn block(
        &self,
        relationship: RelationshipId,
    ) -> Result<(), RelationshipError>;

    fn rotate(
        &self,
        relationship: RelationshipId,
    ) -> Result<RelationshipEpoch, RelationshipError>;
}
```

---

# 183. Contact Invite Service

```rust
pub trait ContactInviteService {
    fn create(
        &self,
        policy: ContactInvitePolicy,
    ) -> Result<ContactInvite, InviteError>;

    fn consume(
        &self,
        invite: ContactInvite,
    ) -> Result<ContactBootstrapResult, InviteError>;

    fn revoke(
        &self,
        invite_id: ContactInviteId,
    ) -> Result<(), InviteError>;
}
```

---

# 184. Social Graph Store

```rust
pub trait SocialGraphStore {
    fn upsert_relationship(
        &self,
        record: RelationshipRecord,
    ) -> Result<(), SocialGraphError>;

    fn get_relationship(
        &self,
        id: RelationshipId,
    ) -> Result<Option<RelationshipRecord>, SocialGraphError>;
}
```

---

# 185. Storage Tables

Potential:

```text
contacts
relationships
relationship_epochs
relationship_capabilities
presence_state
contact_invites
blocked_relationships
verification_records
profile_visibility
discovery_cache
```

---

# 186. Secret Storage

Capability secrets via secure-store references.

---

# 187. DB Encryption

Local DB encrypted where platform design supports.

---

# 188. Relationship Transaction

Creating relationship should atomically persist:

```text
contact
relationship ID
epoch
capability refs
trust state
```

---

# 189. Partial Relationship

Forbidden.

---

# 190. Blocking Transaction

Persist block before destroying/revoking capability references.

---

# 191. Crash Recovery

Blocked relationship stays blocked after restart.

---

# 192. Presence State

Ephemeral/rebuildable.

---

# 193. Contact Graph

Authoritative user data.

---

# 194. Invite State

Durable until expiry/revocation.

---

# 195. Presence Backpressure

If many relationships:

```text
bounded fanout queue
```

---

# 196. Presence Priority

Lower than:

```text
message
call signaling
security
```

---

# 197. Presence Dropping

Old presence update can be coalesced/replaced by latest.

---

# 198. Coalescing

Useful:

```text
Available → Away → Available
```

before network send can collapse to latest.

---

# 199. No Durable Presence Backlog

Generally unnecessary.

---

# 200. Discovery Abuse Threats

```text
identifier enumeration
spam
relationship probing
phone/email guessing
invite scraping
```

---

# 201. Enumeration Testing

Attempt large lookup set.

Provider must rate-limit/prevent.

---

# 202. Relationship Probing

Attacker should not learn:

```text
is Alice connected to Bob?
```

---

# 203. Discovery Response

Only concerns lookup target.

---

# 204. Presence Probing

Without valid capability:

```text
no presence response
```

---

# 205. Block Probing

Blocked user should not receive explicit block bit.

---

# 206. Invite Guessing

Use sufficient entropy.

---

# 207. Capability Guessing

Cryptographically random.

---

# 208. Replay

Consumed one-time invite rejected.

---

# 209. Invite Theft

Bearer invite can be stolen.

Short TTL and one-time semantics reduce risk.

---

# 210. Invitation Confirmation

UI must show intended identity before acceptance.

---

# 211. Contact Impersonation

Verification is separate from discovery.

---

# 212. Exact Handle Collision

Handle service must guarantee unique canonical handle.

---

# 213. Homograph Risk

UI should surface suspicious Unicode/confusables.

---

# 214. Canonicalization

Careful handle normalization.

---

# 215. Display Name

Not unique.

---

# 216. Handle vs Display Name

Distinct types.

---

# 217. Public Handle Change

Old handle mapping expires/redirect policy.

---

# 218. Relationship Stability

Handle changes do not alter established relationship identity.

---

# 219. Account Key Change

High-risk.

Contacts need security notification/reverification.

---

# 220. Trust Reset

Depending cause.

---

# 221. Social Graph Correlation Tests

Use Part 42 privacy lab.

---

# 222. Test Scenario

Provider observes presence objects from many users.

Goal:

```text
cannot cluster all contacts of one user via stable presence ID
```

---

# 223. Contact Discovery Privacy Test

Ensure server cannot reconstruct raw address book in private matching protocol.

---

# 224. Presence Timing Test

Measure relationship correlation from update timing.

---

# 225. Cover-Aware Presence

Part 37 may add delay/batching.

---

# 226. Presence Rate

Bounded.

---

# 227. Presence Noise

Potential future:

```text
dummy relationship presence traffic
```

expensive.

---

# 228. Maximum Anonymity

Simplest strong policy:

```text
presence off
```

---

# 229. Social Graph Metrics

Local-only:

```text
contact count
blocked count
```

---

# 230. Remote Telemetry

Must not include contact graph size in strict mode unless coarse/opted-in.

---

# 231. Provider Metrics

Aggregate service health only.

---

# 232. Forbidden Telemetry

```text
RelationshipId
ContactId
PresenceId
lookup target
invite secret
```

---

# 233. Support Bundle

Redact:

```text
contact names
handles
capabilities
relationship IDs
presence IDs
```

---

# 234. UI — Contact Add

Modes:

```text
Scan QR
Share Invite
Enter Exact Handle
Nearby
```

---

# 235. UI — Presence

Use:

```text
Available
Away
Unknown
```

---

# 236. UI — Verification

Separate status:

```text
Unverified
Verified
Security changed
```

---

# 237. UI — Discovery Privacy

Explain:

```text
Exact-handle search only
```

or private contact matching.

---

# 238. UI — Address Book

If future PSI matching exists:

```text
Contacts are matched privately; raw address-book entries are not uploaded.
```

Only if implementation genuinely provides that.

---

# 239. No Misleading Claims

Hard rule.

---

# 240. Nearby UX

Show:

```text
Nearby does not mean verified.
```

---

# 241. Multi-Account

Relationship graph separated per account/profile.

---

# 242. Work/Personal Profiles

No cross-profile graph sharing without explicit action.

---

# 243. Managed Organization Directory

May expose organization membership.

Separate privacy contract.

---

# 244. Organization Discovery

Could use:

```text
organization-scoped identity
```

not global public handle.

---

# 245. Enterprise Admin

Should not automatically see personal contact graph.

---

# 246. Data Portability

Export contacts separately from live capabilities.

---

# 247. Import

Imported contacts are not automatically verified.

---

# 248. Legacy Migration

Imported phone/email contacts may need explicit discovery.

---

# 249. Deletion

Deleting contact locally can:

```text
remove metadata
revoke capabilities
```

depending action.

---

# 250. Remove vs Block

Different semantics.

---

# 251. Remove

Relationship ends.

May optionally notify.

---

# 252. Block

Relationship invalidated silently where possible.

---

# 253. Mute

Local notification behavior only.

Does not revoke capabilities.

---

# 254. Privacy-Safe Notification

Presence changes should not generate noisy remote-visible events.

---

# 255. Fuzzing

Fuzz:

```text
contact invite
bootstrap descriptor
presence object
discovery response
relationship capability bundle
```

---

# 256. Property Tests

Properties:

```text
blocked relationship cannot publish presence
one-time invite cannot be consumed twice
strict mode never exposes global AccountId as PresenceId
presence without capability returns no state
```

---

# 257. Formal Verification Targets

Good candidates:

```text
relationship state machine
invite consumption
block/revoke transition
capability rotation
```

---

# 258. TLA+ Candidate

Relationship lifecycle:

```text
Pending → Active → Blocked/Revoked
```

---

# 259. Kani Candidate

Invite use-count and expiry invariants.

---

# 260. Loom Candidate

Concurrent block vs presence publish.

---

# 261. Performance Tests

Measure:

```text
presence fanout
contact graph load
invite generation
discovery lookup
multi-device sync
```

---

# 262. Scale Tests

Synthetic graphs:

```text
100 contacts
1,000 contacts
10,000 contacts
```

while maintaining bounded queues.

---

# 263. Discovery Load Tests

Enumeration attack under rate limits.

---

# 264. Offline Tests

Contact graph works locally without server.

---

# 265. Recovery Tests

Restore graph + rotate sensitive capabilities.

---

# 266. Multi-Device Tests

Presence aggregation and capability sync.

---

# 267. Security Invariants

Mandatory:

```text
1. Social graph is authoritative local/E2EE state, not a provider-owned graph.
2. Raw AccountId/DeviceId are never provider-facing presence identifiers.
3. Presence requires an explicit scoped capability.
4. Maximum Anonymity disables presence by default.
5. One-time contact invites cannot be consumed more than once.
6. Block revokes relationship capabilities and survives restart.
7. Unblock creates fresh capabilities rather than restoring old secrets.
8. Simple phone/email hashing is never treated as private discovery.
9. Address-book upload is never required for SIAR operation.
10. Nearby proximity never implies verification.
11. Discovery providers cannot query or expose arbitrary relationship edges.
12. Contact/presence/invite secrets never appear in logs or telemetry.
```

---

# 268. Recommended Crate Layout

```text
crates/
├── siar-social-core/
├── siar-contact-invite/
├── siar-contact-bootstrap/
├── siar-contact-discovery/
├── siar-private-presence/
├── siar-relationship/
├── siar-social-graph/
├── siar-contact-verification/
├── siar-nearby-bootstrap/
├── siar-social-sync/
├── siar-social-observability/
└── siar-social-testkit/
```

---

# 269. `siar-social-core`

Owns:

```text
ContactId
RelationshipId
trust/state types
errors
```

---

# 270. `siar-contact-invite`

Invite generation/consumption/revocation.

---

# 271. `siar-contact-bootstrap`

QR/NFC/existing-channel bootstrap.

---

# 272. `siar-contact-discovery`

Exact-handle/PSI provider abstraction.

---

# 273. `siar-private-presence`

Presence capability/publication/fetch.

---

# 274. `siar-relationship`

Lifecycle/rotation/block.

---

# 275. `siar-social-graph`

Local authoritative graph.

---

# 276. `siar-contact-verification`

QR/SAS/fingerprint verification.

---

# 277. `siar-nearby-bootstrap`

Ephemeral Bluetooth/Wi-Fi proximity bootstrap.

---

# 278. `siar-social-sync`

Encrypted multi-device graph sync.

---

# 279. `siar-social-observability`

Privacy-safe diagnostics.

---

# 280. `siar-social-testkit`

Synthetic graph/discovery attacks.

---

# 281. Error Taxonomy

```rust
pub enum SocialGraphError {
    InviteExpired,
    InviteAlreadyConsumed,
    InviteInvalid,
    RelationshipBlocked,
    RelationshipRevoked,
    PresenceUnauthorized,
    DiscoveryRateLimited,
    DiscoveryUnavailable,
    VerificationChanged,
    CapabilityInvalid,
    Internal,
}
```

---

# 282. Initial Production Scope

Implement first:

```text
QR invite bootstrap
NFC bootstrap
one-time/bounded invites
local authoritative contact graph
relationship IDs
capability-separated messaging/presence/calls/files
contacts-only presence
Available/Away/Unknown semantics
block/revoke with fresh-unblock secrets
exact-handle discovery
anti-enumeration rate limits
multi-device encrypted graph sync
verification states
privacy-safe diagnostics
```

Then add:

```text
OPRF/PSI address-book matching
anonymous presence topics
private introductions
managed organization directories
advanced nearby discovery
presence cover traffic
```

---

# 283. Definition of Done

Part 45 is complete when:

- account, contact, relationship, presence, transport, and mailbox identities are separated
- contact bootstrap supports QR/NFC/invite/existing-channel flows
- relationship capabilities are independently scoped for messaging/presence/calls/files
- presence is capability-gated and coarse
- `Unknown` is distinct from `Offline`
- Maximum Anonymity disables presence by default
- global social graph storage is unnecessary
- exact-handle and future PSI/OPRF discovery boundaries are defined
- raw address-book upload is not required
- contact requests and anti-spam policy are explicit
- blocking/revocation and fresh-unblock capability semantics are defined
- multi-device presence and graph sync are privacy-preserving
- recovery rotates live anonymity-sensitive capabilities
- nearby discovery uses ephemeral IDs and never implies trust
- telemetry/support bundles do not expose graph identifiers
- enumeration, presence correlation, replay, block, recovery, and scale tests are specified

---

# 284. Final Architecture

```text
                   CONTACT BOOTSTRAP
          ┌──────────┬──────────┬──────────┐
          │          │          │          │
         QR         NFC      Invite     Discovery
          │          │          │          │
          └──────────┴──────────┴──────────┘
                           │
                           ▼
                    RELATIONSHIP STATE
                           │
                 Scoped Capabilities
          ┌──────────┬─────────┬──────────┐
          │          │         │          │
      Messaging   Presence   Calls      Files
          │          │         │          │
          └──────────┴─────────┴──────────┘
                           │
                           ▼
                  LOCAL PRIVATE GRAPH
```

Strict presence model:

```text
relationship-specific capability
+
rotating presence identity
+
encrypted short-lived state
+
anonymous fetch
+
coarse semantics
```

not:

```text
global account ID
→ centralized online-status database
```

---

# 285. Final Principle

A private social graph must not be treated as a public directory with encryption added later.

The correct model is:

```text
deliberate bootstrap
+
relationship-local identifiers
+
scoped capabilities
+
private verification
+
optional privacy-preserving discovery
+
coarse/disabled presence
+
local authoritative graph
```

This architecture gives SIAR a way to support familiar contacts, discovery, nearby bootstrap, and presence while preventing those convenience features from becoming a centralized map of who knows, follows, or communicates with whom.
