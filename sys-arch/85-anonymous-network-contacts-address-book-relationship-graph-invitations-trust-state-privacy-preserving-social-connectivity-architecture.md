# Core System Architecture Part 85 — Anonymous Network Contacts, Address Book, Relationship Graph, Invitations, Trust State & Privacy-Preserving Social Connectivity Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 85  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 2, 15, 28, 30, 43, 45–46, 56–59, 67–69, 74–84  

**Primary purpose:** define SIAR's contacts, local address-book, relationship graph, invitation, trust-state, blocking, discovery, relationship synchronization, multi-device contact state, social-graph privacy, and contact-specific profile/capability architecture without creating a centralized graph of who knows whom.

---

# 1. Purpose

Contacts are not merely names in a list.

A contact relationship can reveal:

```text
who knows whom
how often people interact
which identity belongs to which person
which groups overlap
which devices belong to the same account
```

The governing principle is:

> **SIAR treats social connectivity as scoped cryptographic relationships, not as rows in a global people graph.**

---

# 2. Architectural Position

```text
Invitation / Bootstrap Capability
              │
              ▼
       Relationship Establishment
              │
              ▼
       Local Relationship Record
              │
      ┌───────┼────────┐
      │       │        │
   Trust   Profile   Capabilities
      │       │        │
      └───────┼────────┘
              ▼
       Contact-Specific State
```

---

# 3. Core Separation

Keep distinct:

```text
local address-book entry
SIAR relationship
public handle
account identity
device identity
transport identity
group membership
federation identity
```

---

# 4. Non-Goals

Part 85 does not create:

```text
a global friend graph
a centralized uploaded address book
automatic contact merging by phone/email
a global trust score
a permanent universal contact identifier
```

---

# 5. Relationship Identity

Each relationship receives a local random identifier.

```rust
pub struct RelationshipId(pub [u8; 32]);
```

---

# 6. Relationship ID Scope

Local account/device relationship scope.

---

# 7. Hard Rule

RelationshipId is not globally stable or meaningful outside the relationship context.

---

# 8. Remote Relationship Reference

Each side can use a different identifier.

---

# 9. No Shared Global Contact ID

Hard rule.

---

# 10. Contact Record

```rust
pub struct ContactRecord {
    pub relationship: RelationshipId,
    pub state: RelationshipState,
    pub trust: ContactTrustState,
    pub profile: ContactProfileCache,
    pub capabilities: RelationshipCapabilities,
}
```

---

# 11. Relationship State

```rust
pub enum RelationshipState {
    PendingOutgoing,
    PendingIncoming,
    Active,
    Blocked,
    Revoked,
    Archived,
}
```

---

# 12. PendingOutgoing

Invitation sent but not accepted.

---

# 13. PendingIncoming

Invitation received.

---

# 14. Active

Relationship established.

---

# 15. Blocked

Local communication denied.

---

# 16. Revoked

Relationship authority terminated.

---

# 17. Archived

Local UI state only.

---

# 18. Hard Rule

Archive is not a security revocation state.

---

# 19. Contact Trust State

Trust is multidimensional.

```rust
pub struct ContactTrustState {
    pub familiarity: FamiliarityState,
    pub identity_verification: IdentityVerificationState,
    pub device_trust: DeviceTrustSummary,
}
```

---

# 20. Familiarity

```rust
pub enum FamiliarityState {
    Unknown,
    Invited,
    Accepted,
    Known,
}
```

---

# 21. Identity Verification

```rust
pub enum IdentityVerificationState {
    Unverified,
    VerifiedInPerson,
    VerifiedQr,
    VerifiedOutOfBand,
    ChangedSinceVerification,
}
```

---

# 22. Device Trust

Separate.

---

# 23. Hard Rule

Accepted contact ≠ cryptographically verified identity.

---

# 24. Hard Rule

Verified identity ≠ all devices trusted forever.

---

# 25. Invitation

A capability-based bootstrap.

---

# 26. Invitation Type

```rust
pub enum InvitationType {
    DirectCapability,
    Qr,
    Nfc,
    PrivateLink,
    ExactHandleBootstrap,
    ManagedDirectoryInvite,
}
```

---

# 27. Direct Capability

Strong privacy baseline.

---

# 28. QR

Good in-person bootstrap.

---

# 29. NFC

Convenient local bootstrap.

---

# 30. Private Link

Share through another channel.

---

# 31. Exact Handle Bootstrap

Optional.

---

# 32. Managed Directory Invite

Enterprise-scoped.

---

# 33. Invitation Capability

```rust
pub struct ContactInvitation {
    pub invitation_id: InvitationId,
    pub inviter_bootstrap_key: BootstrapPublicKey,
    pub capability: ContactBootstrapCapability,
    pub expires_at: Timestamp,
}
```

---

# 34. Invitation ID

Random.

---

# 35. No AccountId Embedded

Hard rule.

---

# 36. Bootstrap Capability

May authorize only:

```text
contact request
initial profile projection
initial key exchange
```

---

# 37. It Must Not Authorize

```text
admin actions
account recovery
tenant access
arbitrary file access
```

---

# 38. Hard Rule

Contact invitation authority is narrowly scoped.

---

# 39. Invitation Expiry

Required.

---

# 40. One-Time Use

Preferred.

---

# 41. Invitation Replay

Rejected or idempotently mapped to same pending relationship.

---

# 42. No Multiple Relationships From One Single-Use Invite

Hard rule.

---

# 43. Invitation State

```rust
pub enum InvitationState {
    Issued,
    Presented,
    Redeemed,
    Revoked,
    Expired,
}
```

---

# 44. Invitation Revocation

Possible before acceptance.

---

# 45. Invitation Storage

Local.

---

# 46. Server may hold opaque routing capability if feature requires.

---

# 47. No central list of inviter/invitee identities.

---

# 48. Invitation Flow

```text
create scoped invitation
→ share through chosen channel
→ receiver verifies format/capability
→ receiver explicitly accepts
→ perform relationship key exchange
→ create local relationship IDs
→ exchange profile projection
→ activate relationship
```

---

# 49. Explicit Acceptance

Required by default.

---

# 50. No Automatic Contact Addition From Mere Message Receipt

Hard rule.

---

# 51. Message Request

Separate from trusted contact.

---

# 52. Message Request State

```rust
pub enum MessageRequestState {
    Pending,
    Accepted,
    Rejected,
    Blocked,
}
```

---

# 53. Accepting Message Request

May establish limited relationship.

---

# 54. User can later verify identity.

---

# 55. Relationship Capability

```rust
pub struct RelationshipCapabilities {
    pub send_message: CapabilityState,
    pub call: CapabilityState,
    pub profile_view: CapabilityState,
    pub presence_view: CapabilityState,
    pub file_send: CapabilityState,
}
```

---

# 56. Capability State

```rust
pub enum CapabilityState {
    Allowed,
    Denied,
    Ask,
}
```

---

# 57. Capabilities Are Relationship Scoped

Hard rule.

---

# 58. No Contact Relationship Automatically Grants All Features

Hard rule.

---

# 59. Relationship Key Material

Separate from account/login identity.

---

# 60. Relationship Root

Cryptographic state for peer relationship.

---

# 61. Device-Level Sessions

Derived separately.

---

# 62. Multi-Device Fanout

Each peer device authorized individually.

---

# 63. Hard Rule

One account relationship does not mean unknown newly-added devices are silently trusted.

---

# 64. Device Change

May change trust status.

---

# 65. Device Notification

Contact can be informed of newly-added device where protocol supports.

---

# 66. Verification State

May become:

```text
ChangedSinceVerification
```

if identity root changes unexpectedly.

---

# 67. Expected New Device

Does not necessarily invalidate person verification.

---

# 68. Identity Root Change

More severe.

---

# 69. Trust Model

Separate:

```text
person/contact relationship
account/identity continuity
device trust
session trust
```

---

# 70. No Single Boolean `trusted`

Preferred hard rule.

---

# 71. Address Book

OS/local address book integration is optional.

---

# 72. Local Address Book Entry

```rust
pub struct LocalAddressBookEntry {
    pub local_id: LocalAddressBookId,
    pub display_name: String,
    pub phone_numbers: Vec<PhoneNumber>,
    pub emails: Vec<EmailAddress>,
}
```

---

# 73. Sensitive Data

Local only by default.

---

# 74. Hard Rule

Raw device address book is never uploaded wholesale.

---

# 75. Contact Matching

Privacy-sensitive.

---

# 76. Default

No automatic server-side address-book matching.

---

# 77. Safer Alternatives

```text
explicit invite
QR/NFC
exact handle
reviewed PSI/OPRF protocol
```

---

# 78. Simple Hashing Phone/Email

Not sufficient privacy.

---

# 79. Why

Small searchable input space.

---

# 80. Hard Rule

Do not treat SHA256(phone/email) upload as private contact discovery.

---

# 81. Private Contact Discovery

Future optional feature.

---

# 82. Use Reviewed Protocol

Potential:

```text
OPRF
PSI
private set intersection
```

---

# 83. No Custom PSI Crypto

Hard rule.

---

# 84. Discovery Service

Should learn minimum possible.

---

# 85. No persistent uploaded contact list.

---

# 86. Rate Limits

Prevent enumeration.

---

# 87. User Opt-In

Required.

---

# 88. Exact Handle Discovery

Part 59.

---

# 89. Exact match only by default.

---

# 90. No prefix/wildcard people search.

---

# 91. Hard rule.

---

# 92. Address Book Linking

Local mapping:

```rust
pub struct AddressBookLink {
    pub local_entry: LocalAddressBookId,
    pub relationship: RelationshipId,
}
```

---

# 93. Link Never Synced By Default

Hard rule.

---

# 94. Why

Phone/email metadata may leak.

---

# 95. Contact Nickname

Local user-owned metadata.

---

# 96. Not shared.

---

# 97. Contact Notes

Local.

---

# 98. Sensitive.

---

# 99. No Central Sync By Default

Hard rule unless user explicitly enables encrypted sync.

---

# 100. Contact Groups/Labels

Local UI metadata.

---

# 101. Not equivalent to messaging group membership.

---

# 102. Hard rule.

---

# 103. Relationship Graph

Local graph only.

---

# 104. Graph Node

Relationship-local identity.

---

# 105. Graph Edge

User's own relationship.

---

# 106. No Central Adjacency Graph

Hard rule.

---

# 107. Relationship Graph Storage

```rust
pub struct RelationshipGraph {
    pub owner: LocalProfileId,
    pub relationships: BTreeMap<RelationshipId, ContactRecord>,
}
```

---

# 108. Server Synchronization

Encrypted user-owned state only where multi-device sync enabled.

---

# 109. Server should not interpret graph.

---

# 110. Hard Rule

Synced contact graph is opaque ciphertext where feasible.

---

# 111. Relationship Metadata Minimization

Do not sync:

```text
frequency
interaction score
message count
```

unless strictly local feature.

---

# 112. No "Top Friends" Server Metric

Hard rule.

---

# 113. Interaction Ranking

If useful, local only.

---

# 114. Relationship Sync

Local-first.

---

# 115. Durable state:

```text
relationship state
trust verification state
relationship capabilities
profile projection cache
```

---

# 116. Sync Classification

```rust
pub enum RelationshipSyncClass {
    LocalOnly,
    AccountEncrypted,
    RelationshipProtocol,
}
```

---

# 117. LocalOnly

Nickname, notes, OS address-book link.

---

# 118. AccountEncrypted

Own multi-device relationship record.

---

# 119. RelationshipProtocol

State shared with peer.

---

# 120. Hard Rule

Local-only contact metadata never enters relationship protocol.

---

# 121. Multi-Device Contact Sync

New own device can receive encrypted relationship state.

---

# 122. But

New device must establish fresh device sessions.

---

# 123. Hard Rule

Syncing contact record does not clone ratchet/session keys blindly.

---

# 124. Contact Sync Snapshot

May include:

```text
RelationshipId local mapping
peer identity continuity record
profile cache
user-owned nickname if encrypted sync enabled
```

---

# 125. Session/Ratchet Keys

Excluded.

---

# 126. New Device Re-provisioning

Part 57.

---

# 127. Relationship peer may receive device-add event.

---

# 128. Device fanout updates.

---

# 129. Contact State Conflict

Examples:

```text
block on one device
rename nickname on another
capability preference change
```

---

# 130. Conflict Policies

```rust
pub enum ContactConflictPolicy {
    RestrictiveWins,
    LastWriterWinsAllowed,
    Manual,
}
```

---

# 131. Block

RestrictiveWins.

---

# 132. Revocation

RestrictiveWins.

---

# 133. Nickname

LWW allowed.

---

# 134. Verification

Never silently upgraded by conflict.

---

# 135. Hard Rule

Security/trust conflicts resolve toward less trust/more restriction.

---

# 136. Blocking

Local security/privacy action.

---

# 137. Block Effects

```text
reject new direct messages
reject calls
hide presence
stop profile updates
```

---

# 138. Block Does Not Necessarily Notify Peer

Preferred.

---

# 139. No Block Oracle

Hard rule.

---

# 140. Peer may only observe generic delivery unavailability.

---

# 141. Blocked Contact

No automatic unblock from stale sync.

---

# 142. Hard rule.

---

# 143. Revocation

Stronger than block.

---

# 144. Revocation removes relationship capability.

---

# 145. May rotate related keys.

---

# 146. Peer cannot use old invitation to recreate relationship.

---

# 147. Hard rule.

---

# 148. Relationship Revocation Record

```rust
pub struct RelationshipRevocation {
    pub relationship: RelationshipId,
    pub revocation_epoch: u64,
}
```

---

# 149. Epoch Monotonic

---

# 150. Old sync/invite state cannot resurrect.

---

# 151. Hard Rule

Revoked relationship cannot be restored from stale backup without explicit new invitation.

---

# 152. Reconnecting After Revocation

Creates new relationship.

---

# 153. New RelationshipId.

---

# 154. No hidden continuity.

---

# 155. Trust Verification

QR/in-person/out-of-band.

---

# 156. Verification Artifact

Fingerprint/SAS.

---

# 157. Example

```rust
pub struct VerificationRecord {
    pub method: IdentityVerificationMethod,
    pub verified_identity: PeerIdentityDigest,
    pub verified_at: Option<CoarseTimestamp>,
}
```

---

# 158. No Precise Location

Hard rule.

---

# 159. Verification Timestamp

Local/coarse.

---

# 160. Verification State Sync

Encrypted to own devices.

---

# 161. Not sent publicly.

---

# 162. Safety Number / Fingerprint

Derived from identity keys.

---

# 163. Human-comparable.

---

# 164. QR Verification

Encode reviewed fingerprint/capability.

---

# 165. No QR containing unnecessary account metadata.

---

# 166. Hard rule.

---

# 167. Identity Change

Contact sees warning.

---

# 168. Auto-Accept Only If cryptographically proven continuity.

---

# 169. Otherwise require review.

---

# 170. No Silent Trust Carryover

Hard rule.

---

# 171. Contact Profile Exchange

Uses Part 84 projections.

---

# 172. Relationship-specific projection.

---

# 173. Peer sees only fields user exposes.

---

# 174. Public profile not automatically used.

---

# 175. Hard Rule

A contact relationship does not automatically expose all private profile fields.

---

# 176. Contact Visibility Policy

```rust
pub struct ContactVisibilityPolicy {
    pub show_avatar: bool,
    pub show_status: bool,
    pub show_presence: bool,
    pub show_read_receipts: bool,
}
```

---

# 177. Per-Contact Override

Allowed.

---

# 178. Effective policy

Most restrictive between global and per-contact.

---

# 179. No Per-Contact Override Can Bypass Max-Anonymity Floor

Hard rule.

---

# 180. Presence

Part 45.

---

# 181. Capability-based.

---

# 182. Not global broadcast.

---

# 183. Read Receipts/Typing

Relationship scoped.

---

# 184. No Graph-Wide presence fanout.

---

# 185. Hard rule.

---

# 186. Invitations & Abuse

Unsolicited requests can be abusive.

---

# 187. Rate Limits

Per invite capability/source class.

---

# 188. No Central Permanent IP Reputation

Hard rule.

---

# 189. Request Inbox

Bounded.

---

# 190. Spam Filtering

Local heuristics.

---

# 191. Sender Reputation

Scoped/local.

---

# 192. No global social reputation.

---

# 193. Abuse Report

Part 46.

---

# 194. User can report invitation/message-request evidence.

---

# 195. Evidence scoped.

---

# 196. No automatic contact graph disclosure.

---

# 197. Invitation Privacy

Invite channel may reveal metadata.

---

# 198. In-person QR

Strong privacy.

---

# 199. Private link

Leaks to chosen transport/channel.

---

# 200. Exact handle

Directory sees query class.

---

# 201. UX should explain tradeoff.

---

# 202. No False "Anonymous" Claim For Contact Discovery

Hard rule.

---

# 203. Contact Import

From OS address book.

---

# 204. Import means local copy/link.

---

# 205. It does not imply network upload.

---

# 206. Hard rule.

---

# 207. Contact Export

Portable encrypted/human-readable export.

---

# 208. Exclude:

```text
ratchet keys
live sessions
bearer invitations
```

---

# 209. Include:

```text
nicknames
relationship metadata
verification record
```

subject to user choice.

---

# 210. No Export Of Peer Secrets

Hard rule.

---

# 211. Backup

Part 33.

---

# 212. Backup Contact State

Logical.

---

# 213. On Restore

Relationship authority needs safe rebind.

---

# 214. Do not blindly restore active live session state.

---

# 215. Peer identity continuity preserved if valid.

---

# 216. New own device gets fresh sessions.

---

# 217. Deleted/Revoked Relationships

Deletion barrier.

---

# 218. No backup resurrection.

---

# 219. Hard rule.

---

# 220. Contact Deletion

User may remove local record.

---

# 221. Options:

```text
RemoveLocal
Block
RevokeRelationship
```

---

# 222. Different semantics.

---

# 223. Hard Rule

UI/API must not conflate delete-local with cryptographic relationship revocation.

---

# 224. RemoveLocal

May leave peer able to message if relationship remains.

---

# 225. Block

Prevents interaction locally.

---

# 226. RevokeRelationship

Terminates capability.

---

# 227. Account Deletion

Part 83.

---

# 228. Account deletion revokes outbound relationship authority where possible.

---

# 229. Peers retain their own copies of historical received messages/contact notes.

---

# 230. No remote deletion promise.

---

# 231. Hard truth.

---

# 232. Public Handle Retirement

Part 59/83.

---

# 233. Relationship does not depend permanently on public handle.

---

# 234. Handle can rotate without relationship loss.

---

# 235. Hard rule.

---

# 236. Managed Enterprise Contacts

Separate domain.

---

# 237. Org Directory

Can expose managed members within org policy.

---

# 238. Managed directory identity

Org-scoped.

---

# 239. No automatic personal contact creation.

---

# 240. Hard rule.

---

# 241. Employee Directory Entry

Not personal relationship.

---

# 242. Directory can facilitate invite.

---

# 243. User accepts relationship separately where appropriate.

---

# 244. Organization Offboarding

Removes managed directory entry.

---

# 245. Personal relationship may remain if independently established.

---

# 246. Hard Rule

Offboarding does not silently erase personal social relationships.

---

# 247. Tenant Boundary

Part 69.

---

# 248. Tenant contacts separate.

---

# 249. No cross-tenant graph query.

---

# 250. Cross-Tenant Relationship

Explicit capability/federation.

---

# 251. Federation Contacts

Part 58.

---

# 252. Cross-domain contact bootstrap uses signed domain identity and relationship capability.

---

# 253. No global federation people directory.

---

# 254. Hard rule.

---

# 255. Remote Federation Domain

Sees only needed bootstrap metadata.

---

# 256. No transitive contact graph.

---

# 257. Groups

Part 43.

---

# 258. Group membership does not automatically create pairwise contacts.

---

# 259. Hard rule.

---

# 260. Group Member Profile

Group-scoped projection.

---

# 261. Direct Contact Invite

Separate action.

---

# 262. No automatic "friend everyone in group."

---

# 263. Contact Suggestions

High privacy risk.

---

# 264. Default

No server-generated social suggestions.

---

# 265. Hard rule.

---

# 266. If ever supported

Use local signals only.

---

# 267. Example

```text
recently shared QR
same local event code
```

---

# 268. No centralized mutual-contact graph.

---

# 269. Mutual Contacts

Not exposed by default.

---

# 270. Why

Graph leakage.

---

# 271. Hard rule.

---

# 272. Contact Search

Local list full-text search.

---

# 273. Server not needed.

---

# 274. Search index

DeviceLocal.

---

# 275. No remote search query telemetry.

---

# 276. Hard rule.

---

# 277. Contact Sorting

Local.

---

# 278. Recent interaction order

Local.

---

# 279. No central ranking.

---

# 280. Relationship Graph Query API

Local-only.

---

# 281. Example

```rust
pub trait RelationshipRepository {
    fn get(
        &self,
        id: RelationshipId,
    ) -> Result<Option<ContactRecord>, ContactError>;

    fn list(
        &self,
        filter: ContactFilter,
    ) -> Result<Vec<ContactSummary>, ContactError>;
}
```

---

# 282. No API

```text
find all contacts of remote user
```

Hard rule.

---

# 283. Invitation Service

```rust
pub trait ContactInvitationService {
    fn create(
        &self,
        policy: InvitationPolicy,
    ) -> Result<ContactInvitation, ContactError>;

    fn redeem(
        &self,
        invitation: ContactInvitation,
    ) -> Result<PendingRelationship, ContactError>;
}
```

---

# 284. Relationship Establishment Service

```rust
pub trait RelationshipEstablishmentService {
    fn accept(
        &self,
        pending: PendingRelationship,
    ) -> Result<RelationshipId, ContactError>;
}
```

---

# 285. Verification Service

```rust
pub trait ContactVerificationService {
    fn verify(
        &self,
        relationship: RelationshipId,
        evidence: VerificationEvidence,
    ) -> Result<VerificationRecord, ContactError>;
}
```

---

# 286. Block Service

```rust
pub trait RelationshipBlockService {
    fn block(
        &self,
        relationship: RelationshipId,
    ) -> Result<(), ContactError>;
}
```

---

# 287. Revocation Service

```rust
pub trait RelationshipRevocationService {
    fn revoke(
        &self,
        relationship: RelationshipId,
    ) -> Result<RelationshipRevocation, ContactError>;
}
```

---

# 288. Relationship Keying

Local DB key by RelationshipId.

---

# 289. No AccountId as primary graph edge key.

---

# 290. Hard rule.

---

# 291. Storage

Encrypted local DB.

---

# 292. Synced ciphertext optional.

---

# 293. Relationship secrets

secure store/crypto state.

---

# 294. Contact notes/nicknames

encrypted.

---

# 295. Search index

local encrypted/derived where possible.

---

# 296. Server Mailbox

Does not store contact list.

---

# 297. Hard rule.

---

# 298. Social Graph Telemetry

Forbidden.

---

# 299. Safe Aggregate Metrics

Possible:

```text
invitation success rate
request rejection rate
verification feature usage aggregate
```

---

# 300. Forbidden Metrics

```text
degree per user
mutual contacts
relationship edge list
who contacts whom
```

---

# 301. Hard rule.

---

# 302. Contact Analytics

Local only.

---

# 303. No central "engagement graph."

---

# 304. Observability

Sync health only.

---

# 305. Relationship events

Do not send to global analytics bus.

---

# 306. Hard rule.

---

# 307. Audit

Managed enterprise admin actions only where relevant.

---

# 308. Personal contact creation/blocking

No central audit.

---

# 309. Security Event

Identity-key change can be locally logged/notified.

---

# 310. Retention bounded.

---

# 311. Contact State Events

```rust
pub enum RelationshipEvent {
    InvitationCreated,
    InvitationRedeemed,
    RelationshipActivated,
    TrustChanged,
    Blocked,
    Revoked,
}
```

---

# 312. Event Bus

Local/account encrypted only.

---

# 313. No server-wide social event stream.

---

# 314. Hard rule.

---

# 315. Privacy Threat Model

Attacks include:

```text
address-book harvesting
handle enumeration
invite replay
graph reconstruction
device-correlation
profile-correlation
```

---

# 316. Address-Book Harvesting

Raw upload forbidden.

---

# 317. Handle Enumeration

Exact lookup/rate limit.

---

# 318. Invite Replay

Single-use/idempotency.

---

# 319. Graph Reconstruction

No central graph/logging.

---

# 320. Device Correlation

Separate device IDs, minimized profile data.

---

# 321. Profile Correlation

Relationship-specific projection.

---

# 322. Anti-Correlation

Do not expose same stable identifier across:

```text
contact bootstrap
mailbox
transport
payments
telemetry
```

---

# 323. Hard rule.

---

# 324. Abuse Threat Model

Attackers may send mass invites.

---

# 325. Defense

```text
bounded inbox
rate limits
proof/credits if needed
local filtering
```

---

# 326. No proof-of-real-world-identity requirement.

---

# 327. Safety Blocking

Immediate local effect.

---

# 328. Blocked user cannot bypass via same relationship.

---

# 329. New identity/spam remains general abuse problem.

---

# 330. Reputation

Part 46, scoped.

---

# 331. No global blacklist that deanonymizes.

---

# 332. Message Requests

Can be disabled entirely.

---

# 333. Invite Policy

```rust
pub enum IncomingContactPolicy {
    InvitationsOnly,
    ExactHandleRequests,
    ContactsOfManagedDirectory,
    Disabled,
}
```

---

# 334. Default Privacy-Friendly

`InvitationsOnly`.

---

# 335. Public Account

May choose ExactHandleRequests.

---

# 336. No Implicit Open DMs

Hard rule.

---

# 337. Presence Integration

Part 45.

---

# 338. Relationship capability controls.

---

# 339. If Blocked/Revoked

Presence stops.

---

# 340. Typing/read receipt

stop.

---

# 341. Profile updates

stop.

---

# 342. Notification Integration

Part 31/84.

---

# 343. Incoming request notification

privacy-safe.

---

# 344. Do not include remote sensitive data on lock screen by default.

---

# 345. Call Integration

Part 44.

---

# 346. Calls only if relationship capability allows.

---

# 347. Unknown callers

request policy.

---

# 348. No call reveals direct IP in strict privacy mode.

---

# 349. File Sharing

Part 40.

---

# 350. Relationship capability.

---

# 351. Unknown incoming files

restricted.

---

# 352. Security Verification UX

Show:

```text
Unverified
Verified
Identity changed
New device
```

---

# 353. Avoid misleading "Trusted" single label.

---

# 354. Hard rule.

---

# 355. Trust State Changes

Typed.

---

# 356. Example

```rust
pub enum TrustTransition {
    Verify,
    IdentityChanged,
    DeviceAdded,
    DeviceRevoked,
    RelationshipRevoked,
}
```

---

# 357. No Automatic Verified State

except cryptographically proven continuity.

---

# 358. Local Nickname

Does not affect identity verification.

---

# 359. Hard rule.

---

# 360. Verification On Multiple Own Devices

Sync encrypted proof state.

---

# 361. New device can inherit "user previously verified this identity" marker.

---

# 362. But device-specific session still fresh.

---

# 363. Profile Key Rotation

If contact profile encryption key rotates

continuity proof required.

---

# 364. No silent new identity.

---

# 365. Relationship Secret Rotation

Periodic/after compromise.

---

# 366. Old keys retired.

---

# 367. Key lifecycle Part 66.

---

# 368. Contact Data Lifecycle

Part 67.

---

# 369. Local contact record retention

User-controlled.

---

# 370. Revoked state tombstone

bounded but sufficient anti-resurrection.

---

# 371. Contact Notes

Delete when contact removed if user requests.

---

# 372. Verification record

delete with relationship record unless retention needed.

---

# 373. No indefinite metadata.

---

# 374. Managed Directory Data

Org retention separately.

---

# 375. Account Deletion

Deletes local account-owned relationship records in own cloud sync.

---

# 376. Other parties' relationship copies remain theirs.

---

# 377. No remote erasure claim.

---

# 378. Hard truth.

---

# 379. Schema Versioning

Relationship protocol version separate from storage version.

---

# 380. Invitation version.

---

# 381. Trust-state version.

---

# 382. No conflation.

---

# 383. Migration

Part 74.

---

# 384. Unknown future trust state

display conservatively.

---

# 385. No default "verified."

---

# 386. Hard rule.

---

# 387. Relationship Protocol Envelope

```rust
pub struct RelationshipEnvelope<T> {
    pub protocol_version: RelationshipProtocolVersion,
    pub relationship_ref: OpaqueRelationshipRef,
    pub message: T,
}
```

---

# 388. Opaque Relationship Ref

Per-peer.

---

# 389. No local RelationshipId exposed directly.

---

# 390. Hard rule.

---

# 391. Serialization

Postcard internal/network binary.

---

# 392. RON export/config.

---

# 393. JSON only interop.

---

# 394. Strict bounds.

---

# 395. No unbounded contact profile payload.

---

# 396. Contact Photo

Part 84 avatar pipeline.

---

# 397. No original EXIF.

---

# 398. Contact-supplied media

untrusted input.

---

# 399. Decode sandbox/bounds.

---

# 400. Testing

Need dedicated contact/social testkit.

---

# 401. Test Scenarios

```text
invite replay
expired invite
block
revoke
identity change
multi-device sync
```

---

# 402. Invite Replay Test

Second redemption cannot create duplicate relationship.

---

# 403. Expiry Test

Expired invite rejected.

---

# 404. Wrong Audience Test

Invitation not usable by unintended context if bound.

---

# 405. Block Test

Messages/calls/presence denied.

---

# 406. Block Sync Test

Stale device cannot unblock.

---

# 407. Revoke Test

Old relationship capability invalid.

---

# 408. Backup Test

Revoked relationship not resurrected.

---

# 409. Verification Test

Identity change downgrades state.

---

# 410. Device Add Test

Person verification and device trust remain distinct.

---

# 411. Contact Projection Test

One contact never sees another's projection.

---

# 412. Address Book Test

Raw address book never enters network request.

---

# 413. Hash Discovery Test

Simple phone/email hash upload prohibited.

---

# 414. Managed Directory Test

Directory membership does not auto-create personal contact.

---

# 415. Group Test

Group membership does not auto-create pairwise contacts.

---

# 416. Federation Test

No global cross-domain contact graph.

---

# 417. Telemetry Test

No relationship IDs/edges in remote analytics.

---

# 418. Deletion Test

Local removal vs block vs revoke behave distinctly.

---

# 419. Fuzzing

Fuzz:

```text
invitation envelope
relationship protocol
profile projection
trust-state transition
```

---

# 420. Property Tests

Properties:

```text
revoked relationship never returns active without new invitation
block/revoke conflict resolves restrictive
verified state never appears without valid verification evidence
local address-book data never enters network payload
```

---

# 421. Formal Verification Targets

Strong candidates:

```text
invitation state machine
relationship activation/revocation
trust-state transitions
multi-device restrictive merge
```

---

# 422. Kani Candidate

capability/state transition subset rules.

---

# 423. TLA+ Candidate

multi-device block/revoke + stale sync + backup restore.

---

# 424. Loom Candidate

concurrent invitation redemption/acceptance/revocation.

---

# 425. Performance

Contacts are low/medium throughput.

---

# 426. Local search

fast.

---

# 427. Relationship lookup

O(1)/indexed.

---

# 428. No remote search for local contact list.

---

# 429. Multi-device sync

incremental/batched.

---

# 430. Profile projections

small.

---

# 431. Contact graph

not loaded globally server-side.

---

# 432. Memory

Client may cache summaries.

---

# 433. Large contact lists

paged/virtualized.

---

# 434. No O(n) network fanout for every profile update.

---

# 435. Fanout

Use relationship-scoped update mechanisms.

---

# 436. Max-Anonymity Mode

Profile/presence fanout reduced.

---

# 437. No timing-rich contact updates.

---

# 438. Batch/jitter noncritical updates.

---

# 439. Crate Layout

Recommended:

```text
crates/
├── siar-contact-core/
├── siar-contact-invitation/
├── siar-relationship-core/
├── siar-contact-trust/
├── siar-contact-verification/
├── siar-contact-profile/
├── siar-address-book/
├── siar-private-contact-discovery/
├── siar-contact-sync/
├── siar-contact-observability/
└── siar-contact-testkit/
```

---

# 440. `siar-contact-core`

Owns:

```text
RelationshipId
contact state
errors
```

---

# 441. `siar-contact-invitation`

Invitation capability/state.

---

# 442. `siar-relationship-core`

Relationship activation/capabilities/revocation.

---

# 443. `siar-contact-trust`

Familiarity/identity/device trust.

---

# 444. `siar-contact-verification`

QR/out-of-band verification.

---

# 445. `siar-contact-profile`

Relationship-specific profile projections.

---

# 446. `siar-address-book`

Local OS address-book links/import.

---

# 447. `siar-private-contact-discovery`

Optional reviewed PSI/OPRF adapter.

---

# 448. `siar-contact-sync`

Encrypted multi-device relationship sync.

---

# 449. `siar-contact-observability`

Aggregate privacy-safe metrics only.

---

# 450. `siar-contact-testkit`

Replay/block/revoke/identity-change/graph-privacy tests.

---

# 451. Error Taxonomy

```rust
pub enum ContactError {
    InvitationInvalid,
    InvitationExpired,
    InvitationRevoked,
    InvitationAlreadyRedeemed,
    RelationshipNotFound,
    RelationshipBlocked,
    RelationshipRevoked,
    VerificationFailed,
    IdentityChanged,
    CapabilityDenied,
    DiscoveryDenied,
    SyncConflict,
    Internal,
}
```

---

# 452. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. SIAR has no global social graph or universal contact identifier.
2. Relationship IDs are scoped and opaque; account, transport, mailbox, payment, and telemetry identities remain separate.
3. Raw address books are local by default and are never uploaded wholesale.
4. Simple hashes of phone numbers or email addresses are not treated as private contact discovery.
5. Contact invitations are scoped, expiring, replay-resistant capabilities and cannot authorize unrelated actions.
6. Accepted relationship, verified identity, and trusted device are distinct states and are never collapsed into one boolean.
7. Blocking and revocation resolve restrictively across multi-device conflicts and cannot be undone by stale sync or backup restore.
8. Revoked relationships require a new invitation/new relationship identity to reconnect.
9. Contact-specific profile, presence, typing, receipts, calls, and file capabilities are relationship-scoped and cannot leak to other contacts.
10. Group membership, managed-directory membership, and federation membership never automatically create pairwise personal contact relationships.
11. Contact analytics, telemetry, and operational logs never contain a remotely queryable user-to-user edge list.
12. Multi-device synchronization may restore relationship metadata but never blindly clones old ratchet/session secrets.
```

---

# 453. Initial Production Scope

Implement first:

```text
typed RelationshipId
pending/active/blocked/revoked relationship state
capability-based invitations
QR/private-link bootstrap
explicit acceptance/message requests
separate familiarity/identity/device trust
QR/out-of-band verification
relationship-scoped profile/presence/call/file capabilities
local address-book linkage only
encrypted multi-device contact sync
restrictive block/revoke conflict handling
relationship revocation epoch
backup anti-resurrection
local contact search
privacy-safe aggregate metrics
contact/social testkit
```

Then add:

```text
reviewed PSI/OPRF private contact discovery
NFC/bootstrap refinements
advanced relationship portability
federated private contact bootstrap
formal relationship-state verification
privacy-preserving batched profile update fanout
```

---

# 454. Definition of Done

Part 85 is complete when:

- contacts are modeled as scoped cryptographic relationships
- invitation capabilities are expiring/replay-resistant
- relationship acceptance is explicit
- accepted/verified/device-trusted states remain separate
- address books remain local by default
- no simple-hash address-book discovery is used
- public handles do not become global contact IDs
- relationship-specific profile/presence/call/file permissions are enforced
- block/revoke semantics are separate and restrictive
- revoked relationships cannot resurrect from backup/stale sync
- multi-device sync never clones live session/ratchet keys
- managed/group/federation membership never auto-creates personal contact edges
- telemetry contains no centralized social graph
- invite/block/revoke/identity-change/fuzz/formal tests are specified

---

# 455. Final Architecture

```text
                  INVITATION / BOOTSTRAP
                           │
                           ▼
                    EXPLICIT ACCEPTANCE
                           │
                           ▼
                 CRYPTOGRAPHIC RELATIONSHIP
                           │
              ┌────────────┼────────────┐
              │            │            │
           TRUST        PROFILE      CAPABILITIES
              │            │            │
              └────────────┼────────────┘
                           ▼
                   LOCAL CONTACT STATE
                           │
                           ▼
              ENCRYPTED MULTI-DEVICE SYNC
```

Social-connectivity safety model:

```text
scoped relationship identity
+
capability invitation
+
explicit acceptance
+
separate trust dimensions
+
local address book
+
relationship-scoped disclosure
+
restrictive block/revoke
+
no centralized graph
```

not:

```text
upload every phone number, build a global friend graph, and assign everyone a permanent contact ID
```

---

# 456. Final Principle

A contact should be a private relationship between participants, not a globally enumerable edge in a platform-owned graph.

The correct model is:

```text
bootstrap narrowly
+
accept explicitly
+
verify independently
+
share per relationship
+
store graph locally
+
sync encrypted
+
block/revoke durably
+
never centralize the social graph
```

This architecture gives SIAR a privacy-preserving social-connectivity foundation for direct contacts, message requests, invitations, verification, address books, multi-device contact state, groups, enterprise directories, and federation while preserving the anonymity, least-authority, and local-first guarantees established across Parts 34–84.
