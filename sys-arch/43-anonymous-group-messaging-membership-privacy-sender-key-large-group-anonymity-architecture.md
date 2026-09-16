# Core System Architecture Part 43 — Anonymous Group Messaging, Membership Privacy, Sender-Key Distribution & Large-Group Anonymity Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 43  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–42  
**Primary purpose:** define the complete anonymous-group architecture for SIAR, including hidden membership, anonymous fanout, sender-key distribution, group-key evolution, role privacy, join/leave/rekey semantics, large-group scaling, traffic-analysis resistance, abuse controls, recovery, and provider-neutral Rust interfaces.

---

# 1. Purpose

One-to-one anonymous messaging is significantly simpler than anonymous groups.

A group introduces:

```text
many senders
many receivers
membership state
roles
join/leave events
group metadata
fanout
moderation
large traffic volume
```

Each of these can leak the social graph.

The governing principle is:

> **An anonymous group must protect not only message contents, but also membership, role relationships, sender linkage, group activity, and group-wide traffic patterns as far as the selected privacy profile requires.**

---

# 2. Architectural Position

```text
Group Application Layer
        │
        ▼
Group Security / Sender Keys
        │
        ▼
Anonymous Group Distribution Layer
        │
        ├── fanout
        ├── membership-hiding delivery
        ├── rekey
        ├── role privacy
        └── large-group scaling
        │
        ▼
Anonymous Transport / Mixnet / Mailboxes
```

---

# 3. Core Design Goals

The anonymous-group subsystem MUST support:

```text
E2EE group messaging
sender-key distribution
membership privacy
role privacy
join/leave rekey
multi-device members
anonymous delivery
bounded metadata leakage
small and large groups
moderation
abuse resistance
recovery
traffic-analysis resistance
```

---

# 4. Non-Goals

Part 43 does not require:

```text
public broadcast channels
fully permissionless anonymous forums
cryptocurrency incentives
global anonymous social discovery
```

Those are separate systems.

---

# 5. Group Privacy Modes

```rust
pub enum GroupPrivacyMode {
    Standard,
    PrivateMembership,
    AnonymousMembers,
    MaximumAnonymity,
}
```

---

# 6. Standard

Normal SIAR E2EE group.

Membership may be visible to members.

---

# 7. Private Membership

Group members know only the membership information necessary for operation.

---

# 8. Anonymous Members

Members may be represented through pseudonymous/anonymous group-local identities.

---

# 9. Maximum Anonymity

Stricter:

```text
membership minimized
roles hidden where possible
sender linkage reduced
no direct fanout
mixnet-only delivery
traffic shaping
```

---

# 10. Group Identifier

Application-level:

```rust
pub struct GroupId(pub [u8; 32]);
```

Do not expose raw `GroupId` to provider-facing transport.

---

# 11. Anonymous Group Transport ID

```rust
pub struct AnonymousGroupTransportId(pub [u8; 32]);
```

Random and unlinkable to application GroupId.

---

# 12. Group-Local Member Identity

```rust
pub struct GroupMemberPseudonym(pub [u8; 32]);
```

---

# 13. Why Group-Local Identity

Avoid reusing:

```text
AccountId
DeviceId
global username
```

inside anonymous group protocols.

---

# 14. Member Pseudonym Scope

Can be:

```text
per group
per epoch
per session
```

---

# 15. Recommended Initial Scope

```text
per group
```

with epoch rotation in Maximum Anonymity.

---

# 16. Membership Privacy

Membership information itself is sensitive.

Potential adversary wants to learn:

```text
who is in the group
when they joined
when they left
who is admin
who talks most
```

---

# 17. Membership Visibility Model

```rust
pub enum MembershipVisibility {
    FullToMembers,
    RoleScoped,
    Pseudonymous,
    Hidden,
}
```

---

# 18. FullToMembers

All members see all member identities.

---

# 19. RoleScoped

Moderators/admins may see more than ordinary members.

---

# 20. Pseudonymous

Members see group-local pseudonyms.

---

# 21. Hidden

Members may not receive a full membership list.

---

# 22. Hidden Membership Tradeoff

Makes features harder:

```text
mentions
member list
role management
read receipts
moderation
```

---

# 23. Group Security Model

Use established group cryptographic patterns where possible.

Potential approaches:

```text
sender keys
MLS-like group key schedule
hybrid sender-key + control epoch
```

---

# 24. Initial Recommendation

Use a hybrid:

```text
group-control epoch
+
per-sender sender keys
```

for scalability.

---

# 25. Group Epoch

```rust
pub struct GroupEpoch(pub u64);
```

---

# 26. Epoch Changes

Triggered by:

```text
join
leave
ban
device revocation
role-sensitive key change
security event
```

---

# 27. Sender Key

Each sender owns:

```rust
pub struct SenderKeyId(pub [u8; 16]);
```

and associated sender-key material.

---

# 28. Sender-Key Purpose

Avoid encrypting separately for every member for every message.

---

# 29. Sender-Key Distribution

Sender key is distributed to authorized group members through:

```text
group control channel
+
anonymous transport
```

---

# 30. Sender-Key Envelope

```rust
pub struct SenderKeyEnvelope {
    pub group_epoch: GroupEpoch,
    pub sender_pseudonym: GroupMemberPseudonym,
    pub sender_key_id: SenderKeyId,
    pub encrypted_key_material: Bytes,
}
```

---

# 31. Sender Key Rotation

Rotate on:

```text
epoch change
suspected compromise
periodic policy
device removal
```

---

# 32. Forward Secrecy

Key evolution should limit compromise impact.

---

# 33. Post-Compromise Security

Group key architecture should support recovery from compromised member/device where practical.

---

# 34. Join Flow

Conceptual:

```text
invite accepted
→ membership authorization
→ new epoch
→ new control secret
→ distribute current sender keys
→ begin delivery
```

---

# 35. Leave Flow

```text
member removed/leaves
→ new epoch
→ old member excluded
→ rotate affected keys
```

---

# 36. Rekey Requirement

A removed member must not decrypt future messages.

---

# 37. Backward Secrecy

New member should not automatically decrypt old history unless explicitly shared.

---

# 38. History Sharing Policy

```rust
pub enum GroupHistoryPolicy {
    None,
    SinceJoin,
    ExplicitShare,
    FullHistory,
}
```

---

# 39. Anonymous Group Join

Invite should not reveal full membership.

---

# 40. Anonymous Invitation

```rust
pub struct AnonymousGroupInvite {
    pub invite_id: [u8; 16],
    pub group_transport_hint: Bytes,
    pub authorization_capability: SecretBytes,
    pub expires_at: Timestamp,
}
```

---

# 41. Invite Security

Capability:

```text
unguessable
short-lived
scoped
single-use or bounded
```

---

# 42. Join Request Privacy

Join request should not expose global identity unless group policy requires.

---

# 43. Admission Modes

```rust
pub enum AnonymousGroupAdmission {
    InviteOnly,
    CapabilityOnly,
    ModeratorApproved,
    Managed,
}
```

---

# 44. InviteOnly

Requires explicit invitation.

---

# 45. CapabilityOnly

Possession of valid invite capability grants admission.

---

# 46. ModeratorApproved

Moderators approve request.

---

# 47. Managed

Organization policy.

---

# 48. Role Model

```rust
pub enum GroupRole {
    Owner,
    Admin,
    Moderator,
    Member,
}
```

---

# 49. Role Privacy

Role may itself be sensitive.

---

# 50. Role Visibility

```rust
pub enum RoleVisibility {
    PublicToGroup,
    VisibleToPrivilegedMembers,
    Hidden,
}
```

---

# 51. Hidden Moderator Identity

Possible in advanced group mode.

---

# 52. Moderation Capability

Instead of revealing moderator identity, use cryptographic moderation capability.

---

# 53. Moderation Token

```rust
pub struct ModerationCapability(SecretBytes);
```

---

# 54. Moderation Action

Could be authenticated as:

```text
authorized moderator action
```

without exposing which moderator performed it to ordinary members.

---

# 55. Governance Audit

Privileged audit may retain accountability separately.

---

# 56. Anonymous Sender Identity

Sender may appear as:

```text
group-local pseudonym
```

---

# 57. Fully Anonymous Sender

Potential advanced mode:

```text
message authenticated as valid group member
without persistent sender pseudonym
```

Requires more advanced cryptography.

---

# 58. Initial Recommendation

Use stable group-local pseudonym.

---

# 59. Sender Linkability

Within group:

```text
messages from same pseudonym are linkable
```

unless rotating pseudonym mode used.

---

# 60. Pseudonym Rotation

Potential:

```text
per epoch
```

---

# 61. Tradeoff

Rotation improves privacy but makes conversation continuity harder.

---

# 62. Group Fanout

Naive approach:

```text
sender
→ N recipient mailboxes
```

---

# 63. Fanout Cost

For group of N users with D devices:

```text
O(N × D)
```

---

# 64. Small Group Strategy

Direct anonymous fanout is acceptable for:

```text
small groups
```

---

# 65. Medium Group Strategy

Use:

```text
group mailbox distribution
```

---

# 66. Large Group Strategy

Need scalable distribution substrate.

---

# 67. Group Mailbox

A shared anonymous mailbox can hold encrypted group messages.

---

# 68. Shared Mailbox Privacy Risk

All members access same mailbox.

Provider may learn:

```text
group activity
fetch timing
```

---

# 69. Per-Member Retrieval Capability

Better:

```text
shared encrypted object
+
member-specific access capability
```

---

# 70. Group Distribution Modes

```rust
pub enum GroupDistributionMode {
    PerDeviceFanout,
    SharedMailbox,
    TreeFanout,
    AnonymousPublishSubscribe,
}
```

---

# 71. PerDeviceFanout

Strong separation, expensive.

---

# 72. SharedMailbox

Efficient, more observable group activity.

---

# 73. TreeFanout

Intermediate nodes relay group data.

Privacy analysis required.

---

# 74. AnonymousPublishSubscribe

Provider-neutral future model.

---

# 75. Initial Recommendation by Size

```text
Small:
    PerDeviceFanout

Medium:
    SharedMailbox + per-member capability

Large:
    AnonymousPublishSubscribe / tree fanout
```

---

# 76. Group Size Classes

```rust
pub enum GroupSizeClass {
    Small,
    Medium,
    Large,
    VeryLarge,
}
```

---

# 77. Group Size Leakage

Provider may infer size from traffic volume.

---

# 78. Mitigation

```text
batching
cover traffic
shared mailbox
padding
coarse size classes
```

---

# 79. Group Message Envelope

```rust
pub struct AnonymousGroupMessage {
    pub group_epoch: GroupEpoch,
    pub message_id: GroupMessageId,
    pub sender_key_id: SenderKeyId,
    pub ciphertext: Bytes,
}
```

---

# 80. Message ID

Transport-safe random identifier.

---

# 81. Application Message ID

Separate.

---

# 82. Control Messages

Examples:

```text
join
leave
role change
rekey
sender-key update
ban
epoch advance
```

---

# 83. Control Plane Separation

Group control messages should use higher integrity/ordering guarantees.

---

# 84. Control Channel

```rust
pub enum GroupControlMessage {
    EpochAdvance,
    SenderKeyUpdate,
    MembershipChange,
    RoleChange,
    DeviceChange,
    PolicyChange,
}
```

---

# 85. Ordering

Control messages require:

```text
monotonic epoch
causal consistency
```

---

# 86. Message Ordering

User messages can tolerate looser ordering.

---

# 87. Group Epoch State

```rust
pub struct GroupEpochState {
    pub epoch: GroupEpoch,
    pub members_digest: MembershipDigest,
    pub control_key_ref: SecretRef,
}
```

---

# 88. Membership Digest

Compact authenticated representation.

---

# 89. Hidden Membership

Digest does not reveal full membership list to ordinary clients.

---

# 90. Membership Proofs

Potential:

```text
member proves inclusion
without revealing all members
```

---

# 91. Merkle Membership

One possible technique.

---

# 92. Zero-Knowledge Membership

Possible future advanced mode.

---

# 93. Initial Recommendation

Use:

```text
authenticated membership tree
```

without introducing custom ZK initially.

---

# 94. Member Inclusion Proof

```rust
pub struct MembershipProof {
    pub epoch: GroupEpoch,
    pub proof: Bytes,
}
```

---

# 95. Membership Proof Privacy

Should reveal only:

```text
membership validity
```

not full roster.

---

# 96. Group Control Authority

Owner/admin authority signs membership updates.

---

# 97. Multi-Admin Control

Potential threshold approval for high-risk changes.

---

# 98. Threshold Admin Policy

```rust
pub struct GroupAdminQuorum {
    pub required: u16,
    pub total: u16,
}
```

---

# 99. Use Cases

```text
owner transfer
large-group policy change
mass removal
```

---

# 100. Large Group Governance

Central owner becomes operational bottleneck.

---

# 101. Role Delegation

Use scoped capabilities.

---

# 102. Moderation Without Full Roster

Moderator can act on:

```text
message pseudonym
member capability
```

without seeing real identity.

---

# 103. Ban Semantics

Ban should invalidate:

```text
membership capability
sender key access
future epoch access
```

---

# 104. Ban Privacy

Ordinary members need not learn banned member's real identity.

---

# 105. Spam Protection

Anonymous groups are vulnerable to spam.

---

# 106. Posting Capability

```rust
pub struct GroupPostCapability(SecretBytes);
```

---

# 107. Capability Quota

Can limit:

```text
messages per time window
attachment bytes
```

---

# 108. Rate Limits

Should be group-local, not global identity-based.

---

# 109. Proof-of-Work

Optional for open anonymous groups.

---

# 110. Caution

Heavy PoW hurts mobile devices.

---

# 111. Token Bucket

Preferred for authorized members.

---

# 112. Membership Capability

Each member gets scoped posting/read capability.

---

# 113. Read Capability

Controls access to:

```text
group mailbox
history
```

---

# 114. Write Capability

Controls message posting.

---

# 115. Moderator Capability

Controls moderation actions.

---

# 116. Capability Separation

Hard rule.

---

# 117. Multi-Device Member

Each device should have:

```text
device-specific receiving capability
```

---

# 118. Sender Key Across Devices

Options:

```text
shared member sender identity
or
device-specific sender key
```

---

# 119. Recommended

Use device-specific sender keys.

---

# 120. Why

Improves:

```text
revocation
compromise isolation
multi-device attribution
```

---

# 121. Group Member Pseudonym vs Device Pseudonym

```text
member pseudonym
    ├── device A sender key
    ├── device B sender key
```

---

# 122. Device Revocation

Rotate/remove only compromised device sender key.

---

# 123. Member Revocation

Remove all device keys.

---

# 124. Group State Sync

Group state must sync across user's trusted devices.

---

# 125. Sync Privacy

Do not expose:

```text
group membership
```

to non-participating providers.

---

# 126. Device Join

Existing trusted device authorizes new device access.

---

# 127. New Device History

Controlled by history policy.

---

# 128. Recovery

Account recovery should not automatically restore group membership if security policy requires reauthorization.

---

# 129. Group Recovery Policy

```rust
pub enum GroupRecoveryPolicy {
    RestoreIfDeviceTrusted,
    Reauthorize,
    NeverAutoRestore,
}
```

---

# 130. Maximum Anonymity

Prefer:

```text
Reauthorize
```

for sensitive groups.

---

# 131. Backup

Do not blindly back up live sender keys.

---

# 132. Backup-Safe Group State

Can include:

```text
group metadata
non-secret policy
history if encrypted
```

---

# 133. Fresh Sender Keys After Restore

Recommended.

---

# 134. Group Key Compromise

Security response:

```text
advance epoch
rotate control key
rotate sender keys
```

---

# 135. Compromise Notification

Members may receive:

```text
group security state changed
```

without exposing compromised member identity unnecessarily.

---

# 136. Sender Authentication

Recipient verifies:

```text
valid sender key
valid epoch
membership authorization
```

---

# 137. Unknown Sender Key

Hold/reject until control update arrives.

---

# 138. Out-of-Order Control

Buffer bounded.

---

# 139. Epoch Gap

Request/resync group state anonymously.

---

# 140. Resync Privacy

Do not expose exact missing history to provider.

---

# 141. Large Group Delivery

Large group needs scalable dissemination.

---

# 142. Anonymous Publish/Subscribe

Concept:

```text
publisher posts encrypted group object
subscribers fetch anonymously
```

---

# 143. Topic ID

Provider-facing topic must not expose GroupId.

---

# 144. Topic Token

```rust
pub struct AnonymousTopicId(pub [u8; 32]);
```

---

# 145. Topic Rotation

Rotate per epoch.

---

# 146. Subscriber Capability

```rust
pub struct GroupSubscriptionCapability(SecretBytes);
```

---

# 147. Publisher Capability

```rust
pub struct GroupPublishCapability(SecretBytes);
```

---

# 148. Provider View

Should see:

```text
opaque topic
traffic
```

not:

```text
group membership
real identities
```

---

# 149. Shared Topic Correlation

Provider can see everyone fetching same topic if access not anonymized.

---

# 150. Therefore

All topic access must go through:

```text
mixnet/anonymous transport
```

in strict modes.

---

# 151. Pull Model

Subscribers periodically fetch topic.

---

# 152. Push Model

Push can leak membership/timing.

---

# 153. Strict Mode

Prefer pull/cover-aware fetch.

---

# 154. Fetch Scheduling

Integrate with Part 37.

---

# 155. Group Cover Traffic

Potential:

```text
periodic fetch
dummy fetch
cover post
```

---

# 156. Cost

Large groups can make cover traffic expensive.

---

# 157. Group Cover Policy

```rust
pub enum GroupCoverPolicy {
    None,
    Adaptive,
    Standard,
    Maximum,
}
```

---

# 158. Large Group Tradeoff

Maximum cover may be impractical for very large groups.

Document honestly.

---

# 159. Group Traffic Shaping

Batch:

```text
messages
receipts
control updates
```

---

# 160. No Per-Message Immediate Fetch

Strict mode.

---

# 161. Read Receipts

Large anonymous groups:

```text
off by default
```

---

# 162. Delivery Receipts

Aggregate only if needed.

---

# 163. Typing Indicators

Disabled in strict anonymous groups.

---

# 164. Presence

Disabled or coarse.

---

# 165. Reactions

Can leak activity.

May be delayed/batched.

---

# 166. Mentions

Pseudonym-based.

---

# 167. Member Directory

Could be absent in hidden-membership mode.

---

# 168. Search

Local only.

---

# 169. Group Metadata

Sensitive:

```text
name
description
avatar
member count
```

---

# 170. Metadata Encryption

Group metadata distributed inside E2EE group state.

---

# 171. Member Count

Could be bucketed:

```text
<10
10–50
50–200
200+
```

if UI does not need exact count.

---

# 172. Group Avatar

Encrypted.

---

# 173. External Avatar Fetch

Forbidden in strict mode.

---

# 174. Link Previews

Same.

---

# 175. Attachments

Use Part 40 anonymous attachment architecture.

---

# 176. Group Attachment Capability

Can be:

```text
per member
or
group capability
```

---

# 177. Strict Mode

Prefer per-member or epoch-scoped capability where feasible.

---

# 178. Attachment Revocation

On member removal, old capability may still exist.

Use short TTL + epoch rotation.

---

# 179. Moderation of Attachments

Moderators can revoke group-visible reference.

---

# 180. Content Deletion

Cannot guarantee remote erase from member devices.

---

# 181. Group Delete Semantics

```text
remove from future group state
```

not:

```text
erase all copies everywhere
```

---

# 182. Message Deletion

Same honesty.

---

# 183. Anonymous Admin Action

Action can be authorized without revealing admin identity.

---

# 184. Audit Trail

Privileged local audit may record:

```text
capability action ID
```

---

# 185. No Public Moderator Mapping

Unless policy requires.

---

# 186. Group Abuse Reports

Member may report message.

---

# 187. Report Privacy

Report should not automatically reveal reporter identity to group.

---

# 188. Moderator Report Inbox

Could use anonymous report capability.

---

# 189. Report Evidence

Includes:

```text
message ciphertext/plaintext as user chooses
sender pseudonym
epoch
```

---

# 190. Abuse Reporting Tradeoff

Evidence can reveal content to moderators.

Must be explicit.

---

# 191. Open Anonymous Groups

Higher abuse risk.

---

# 192. Admission Control

Use:

```text
invite
capability
proof-of-work
rate limits
```

---

# 193. Closed Anonymous Groups

Easier to secure.

---

# 194. Recommended Initial Scope

Start with:

```text
closed invite-only anonymous groups
```

---

# 195. Large Public Anonymous Groups

Defer until abuse/moderation architecture matures.

---

# 196. Group Distribution Service

```rust
#[async_trait]
pub trait AnonymousGroupDistribution: Send + Sync {
    async fn publish(
        &self,
        group: AnonymousGroupTransportId,
        message: AnonymousGroupMessage,
    ) -> Result<GroupDeliveryId, GroupDistributionError>;

    async fn fetch(
        &self,
        subscription: &GroupSubscriptionCapability,
        cursor: Option<GroupCursor>,
    ) -> Result<GroupBatch, GroupDistributionError>;
}
```

---

# 197. Group Key Service

```rust
pub trait GroupKeyService {
    fn current_epoch(
        &self,
        group: GroupId,
    ) -> Result<GroupEpochState, GroupKeyError>;

    fn rotate_epoch(
        &self,
        group: GroupId,
        reason: EpochRotationReason,
    ) -> Result<GroupEpochState, GroupKeyError>;
}
```

---

# 198. Sender Key Service

```rust
pub trait SenderKeyService {
    fn create_sender_key(
        &self,
        group: GroupId,
        device: DeviceId,
    ) -> Result<SenderKeyId, GroupKeyError>;

    fn rotate_sender_key(
        &self,
        sender: SenderKeyId,
    ) -> Result<SenderKeyId, GroupKeyError>;
}
```

---

# 199. Membership Service

```rust
pub trait AnonymousMembershipService {
    fn prove_membership(
        &self,
        group: GroupId,
        member: GroupMemberPseudonym,
    ) -> Result<MembershipProof, MembershipError>;

    fn apply_change(
        &self,
        change: MembershipChange,
    ) -> Result<GroupEpoch, MembershipError>;
}
```

---

# 200. Capability Service

```rust
pub trait GroupCapabilityService {
    fn issue_post_capability(
        &self,
        member: GroupMemberPseudonym,
    ) -> Result<GroupPostCapability, CapabilityError>;

    fn revoke_member_capabilities(
        &self,
        member: GroupMemberPseudonym,
    ) -> Result<(), CapabilityError>;
}
```

---

# 201. Group State Database

Potential tables:

```text
anonymous_groups
group_epochs
group_membership_tree
group_member_pseudonyms
group_sender_keys
group_capabilities
group_distribution_state
group_control_log
```

---

# 202. `anonymous_groups`

Stores:

```text
GroupId
transport ID
privacy mode
size class
distribution mode
```

---

# 203. `group_epochs`

Stores:

```text
epoch
control key ref
membership digest
created_at
```

---

# 204. `group_sender_keys`

Secret refs only.

---

# 205. `group_capabilities`

Secret refs, scope, expiry, state.

---

# 206. Secret Storage

Use secure store.

---

# 207. Crash Safety

Control updates durable before use.

---

# 208. Epoch Transition Transaction

```text
persist new membership
persist new epoch
persist key refs
activate epoch
```

atomically.

---

# 209. Partially Applied Epoch

Forbidden.

---

# 210. Control Log

Append-only local group control history.

---

# 211. Control Log Privacy

Encrypted at rest.

---

# 212. Conflict Resolution

Concurrent admin changes require deterministic resolution.

---

# 213. Control Sequence

Use:

```text
epoch
+
monotonic control sequence
```

---

# 214. Split-Brain Group State

Must detect.

---

# 215. Admin Concurrency

Threshold/quorum can help for high-risk changes.

---

# 216. Multi-Admin Conflict

Define canonical ordering.

---

# 217. Group State Hash

```rust
pub struct GroupStateDigest(pub [u8; 32]);
```

---

# 218. State Validation

Each control event references previous digest.

---

# 219. Tamper Detection

Broken chain -> reject/resync.

---

# 220. Anonymous Group Search

Local only.

---

# 221. Discovery

Strict anonymous groups should not be globally discoverable initially.

---

# 222. Invite Distribution

Through:

```text
existing E2EE
QR
NFC
anonymous bootstrap
```

---

# 223. Group QR

Must not expose full membership or stable transport secret unnecessarily.

---

# 224. Invite Revocation

Possible until consumed/expired.

---

# 225. One-Time Invite

Preferred for sensitive groups.

---

# 226. Batch Invite

Can issue bounded multiple-use capability.

---

# 227. Invite Leak

Rotate/revoke.

---

# 228. Group Traffic Analysis

Adversary may infer:

```text
group active now
group size
top talkers
join/leave timing
```

---

# 229. Mitigations

```text
shared mailbox
cover fetch
batching
topic rotation
pseudonyms
coarse count
```

---

# 230. Join/Leave Hiding

Can batch membership changes.

---

# 231. Delayed Epoch Publication

Potential:

```text
membership changes applied in batch
```

to reduce exact timing leakage.

---

# 232. Security Tradeoff

Removed member should lose access quickly.

---

# 233. Therefore

Security-critical removals:

```text
immediate epoch advance
```

Privacy batching only for non-critical joins where policy permits.

---

# 234. Large Group Scaling

Need bounded client state.

---

# 235. Membership Tree

Prefer:

```text
Merkle tree / tree-based membership structure
```

for large groups.

---

# 236. O(N) Full Roster

Avoid for hidden-membership client.

---

# 237. Proof Size

Logarithmic where possible.

---

# 238. Group Rekey Complexity

Naive:

```text
O(N)
```

per membership change.

---

# 239. Tree-Based Rekey

Future improvement.

---

# 240. MLS-Inspired Tree

Could reduce rekey cost.

---

# 241. Caution

Do not implement custom MLS variant without deep review.

---

# 242. Initial Production Scope

Small/medium groups can accept simpler O(N) control fanout.

---

# 243. Large Group Future

Tree-based key schedule.

---

# 244. Membership Privacy With MLS

MLS membership visibility assumptions may differ.

Need separate adaptation/privacy review.

---

# 245. Pseudonymous Credentials

Future:

```text
anonymous credentials
```

for hidden-membership proof.

---

# 246. Zero-Knowledge Membership

Future only.

---

# 247. Group Broadcast Authenticity

Message must prove:

```text
valid current sender
```

without revealing global identity.

---

# 248. Sender Signature

Use group-local signing key or sender-key authentication.

---

# 249. Signature Linkability

Persistent signature key creates pseudonym linkability.

---

# 250. Epoch-Rotated Signature Key

Optional.

---

# 251. Large Group Read Scaling

Avoid per-message ACK from every member.

---

# 252. Delivery Aggregates

Potential:

```text
delivered to some
delivery unknown
```

---

# 253. Exact Recipient Receipts

Disabled in anonymous large groups.

---

# 254. Read Receipts

Off.

---

# 255. Presence

Off.

---

# 256. Typing

Off.

---

# 257. Group Calls

Outside current part.

Would need separate anonymous group realtime architecture.

---

# 258. Group Voice Notes

Treated as attachments/messages.

---

# 259. Group Files

Use Part 40.

---

# 260. Message Expiry

Can be group policy.

---

# 261. Ephemeral Group

Possible:

```rust
pub enum GroupLifetime {
    Persistent,
    Expiring,
    SessionOnly,
}
```

---

# 262. Expiring Group

Useful for sensitive temporary collaboration.

---

# 263. Expiry

Rotate/destroy:

```text
transport topic
capabilities
keys
```

---

# 264. Local Copies

Cannot guarantee deletion if members retained data.

---

# 265. Group Archive

Strict groups may disable server-side archive entirely.

---

# 266. Local Archive

User-controlled.

---

# 267. Backup Interaction

Group policy may forbid backup of secrets/history.

---

# 268. Policy

```rust
pub struct AnonymousGroupDataPolicy {
    pub allow_backup: bool,
    pub allow_export: bool,
    pub history_policy: GroupHistoryPolicy,
}
```

---

# 269. Export

Can leak membership/content.

Warn explicitly.

---

# 270. Plugins

Maximum anonymity group should restrict:

```text
network plugins
message-reading plugins
external AI
```

---

# 271. Plugin Capability

Group-specific permission required.

---

# 272. Moderation Plugin

Cannot receive full roster unless explicitly authorized.

---

# 273. Diagnostics

Normal:

```text
Anonymous group: Active
Group privacy: Maximum
```

---

# 274. Advanced Diagnostics

Can show:

```text
epoch
distribution mode
sender-key health
control sync state
```

---

# 275. Forbidden Diagnostics

No:

```text
membership roster
capability tokens
sender keys
transport topic secret
```

---

# 276. Telemetry

Aggregate only:

```text
group size class
delivery success
rekey failures
```

---

# 277. No Group ID Labels

Hard rule.

---

# 278. Testkit

Need synthetic anonymous group simulator.

---

# 279. Synthetic Group Sizes

```text
5
20
100
1,000
10,000
```

for architecture tests.

---

# 280. Unit Tests

Test:

```text
join
leave
ban
rekey
sender key rotation
history policy
```

---

# 281. Hidden Membership Tests

Ordinary member cannot enumerate roster.

---

# 282. Role Privacy Tests

Ordinary member cannot identify hidden moderator.

---

# 283. Capability Tests

Posting without valid capability rejected.

---

# 284. Revocation Tests

Removed member cannot decrypt future epoch.

---

# 285. Backward Secrecy Tests

New member cannot decrypt old history unless policy allows.

---

# 286. Device Revocation Tests

One device removed, member's other devices continue.

---

# 287. Multi-Device Tests

Independent sender keys.

---

# 288. Epoch Gap Tests

Resync works.

---

# 289. Control Conflict Tests

Concurrent admin changes converge deterministically.

---

# 290. Split-Brain Tests

Invalid state chain detected.

---

# 291. Distribution Tests

Per-device vs shared mailbox vs pub/sub.

---

# 292. Traffic Analysis Tests

Measure:

```text
group size inference
top-talker inference
join timing
```

---

# 293. Cover Traffic Tests

Strict mode hides immediate fetch/send where configured.

---

# 294. Large Group Tests

Bound:

```text
memory
CPU
membership proof size
control update cost
```

---

# 295. Abuse Tests

```text
spam
capability theft
invite flood
moderation abuse
```

---

# 296. Fuzzing

Fuzz:

```text
group control message
membership proof
sender-key envelope
capability parser
```

---

# 297. Property Tests

Properties:

```text
removed member never decrypts new epoch
new member cannot decrypt old epoch by default
invalid capability cannot publish
epoch monotonicity always holds
```

---

# 298. Formal Verification Targets

Good candidates:

```text
epoch transitions
membership state
capability revocation
sender-key lifecycle
```

---

# 299. TLA+ Candidate

Group membership/epoch state machine.

---

# 300. Loom Candidate

Concurrent sender-key rotation.

---

# 301. Kani Candidate

Capability scope/epoch checks.

---

# 302. Performance Tests

Measure:

```text
rekey time
fanout cost
fetch cost
sender-key distribution
memory
```

---

# 303. Privacy Tests

Use Part 42 privacy lab.

---

# 304. Group Correlation Attack

Adversary observes:

```text
same set of clients fetching around same times
```

---

# 305. Mitigation Evaluation

Compare:

```text
no cover
cover fetch
shared mailbox
rotating topic
```

---

# 306. Release Gates

Block if:

```text
removed member can decrypt new message
membership leak appears in provider-facing metadata
strict group uses direct fanout
capability token logged
```

---

# 307. Security Invariants

Mandatory:

```text
1. Anonymous groups never expose raw AccountId/DeviceId to provider-facing transport.
2. Group-local pseudonyms are distinct from global identity.
3. Removed members cannot decrypt future epochs.
4. New members cannot decrypt old history unless explicitly allowed.
5. Sender keys are device-scoped by default.
6. Membership/role capabilities are distinct.
7. Maximum-anonymity groups do not silently use direct member fanout.
8. Hidden-membership mode does not expose the full roster to ordinary members.
9. Group control state advances monotonically by epoch.
10. Group capability secrets and sender keys never appear in logs/telemetry.
11. Large-group provider-visible topic IDs are random and epoch-rotatable.
12. Read receipts, typing, and presence are disabled by default in strict anonymous groups.
```

---

# 308. Recommended Crate Layout

```text
crates/
├── siar-anon-group-core/
├── siar-anon-group-membership/
├── siar-anon-group-keys/
├── siar-anon-group-capability/
├── siar-anon-group-control/
├── siar-anon-group-distribution/
├── siar-anon-group-mailbox/
├── siar-anon-group-pubsub/
├── siar-anon-group-moderation/
├── siar-anon-group-observability/
└── siar-anon-group-testkit/
```

---

# 309. `siar-anon-group-core`

Owns:

```text
IDs
privacy modes
size classes
errors
```

---

# 310. `siar-anon-group-membership`

Membership tree/proofs/epoch membership state.

---

# 311. `siar-anon-group-keys`

Control keys/sender keys/rotation.

---

# 312. `siar-anon-group-capability`

Read/write/moderator capabilities.

---

# 313. `siar-anon-group-control`

Join/leave/role/epoch control log.

---

# 314. `siar-anon-group-distribution`

Provider-neutral fanout.

---

# 315. `siar-anon-group-mailbox`

Shared mailbox distribution.

---

# 316. `siar-anon-group-pubsub`

Large-group anonymous topic transport.

---

# 317. `siar-anon-group-moderation`

Anonymous moderation capability.

---

# 318. `siar-anon-group-observability`

Privacy-safe metrics.

---

# 319. `siar-anon-group-testkit`

Synthetic groups/fault injection.

---

# 320. Error Taxonomy

```rust
pub enum AnonymousGroupError {
    NotMember,
    InvalidCapability,
    EpochMismatch,
    SenderKeyMissing,
    MembershipProofInvalid,
    RoleUnauthorized,
    DistributionUnavailable,
    RekeyRequired,
    HistoryUnavailable,
    GroupExpired,
    Internal,
}
```

---

# 321. Initial Production Scope

Implement first:

```text
invite-only groups
group-local pseudonyms
per-device sender keys
group epochs
join/leave rekey
per-device anonymous fanout for small groups
shared mailbox for medium groups
hidden roster option
role-scoped capabilities
no presence/typing/read receipts in strict mode
privacy-safe diagnostics
```

Then add:

```text
anonymous pub/sub
tree-based rekey
hidden moderator identity
anonymous credentials
zero-knowledge membership
very-large-group distribution
```

---

# 322. Definition of Done

Part 43 is complete when:

- anonymous group privacy modes are explicit
- global identities are separated from group-local pseudonyms
- sender-key distribution and rotation are defined
- group epochs handle join/leave/ban/device removal
- forward/backward secrecy policies are explicit
- membership visibility can be full, role-scoped, pseudonymous, or hidden
- role privacy and capability-based moderation are defined
- small/medium/large group distribution strategies are distinct
- provider-facing topic/group identifiers are random and rotatable
- read receipts/presence/typing are suppressed in strict groups
- multi-device sender keys and device revocation are defined
- hidden-membership and large-group scaling are accounted for
- abuse/spam/moderation controls do not require global identity exposure
- backups/recovery do not blindly restore live sender keys
- diagnostics/telemetry do not expose roster or secrets
- privacy, scale, abuse, fuzz, property, and formal-state tests are specified

---

# 323. Final Architecture

```text
                         GROUP APPLICATION
                                │
                                ▼
                     GROUP CONTROL / EPOCH
                                │
                   ┌────────────┴────────────┐
                   │                         │
             Membership State         Sender Keys
                   │                         │
                   └────────────┬────────────┘
                                ▼
                    Anonymous Distribution
              ┌─────────────────┼─────────────────┐
              │                 │                 │
       Per-Device Fanout   Shared Mailbox    Anonymous Pub/Sub
              │                 │                 │
              └─────────────────┼─────────────────┘
                                ▼
                       Mixnet / Mailboxes
```

Strict anonymous group:

```text
Group-local pseudonyms
+
device-scoped sender keys
+
hidden/limited membership
+
epoch-based rekey
+
anonymous distribution
+
cover-aware fetch
+
no presence/typing/read receipts
```

---

# 324. Final Principle

Anonymous group messaging is not merely:

```text
normal group chat
+
mixnet
```

The correct model is:

```text
group-local identity
+
membership privacy
+
sender-key separation
+
epoch rekey
+
capability-based roles
+
anonymous fanout
+
large-group distribution
+
traffic-analysis resistance
```

This architecture gives SIAR a path from small private anonymous groups to large anonymous group systems without exposing the social graph or coupling group security to one transport provider.
