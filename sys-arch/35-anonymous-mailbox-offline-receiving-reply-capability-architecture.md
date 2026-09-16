# Core System Architecture Part 35 — Anonymous Mailbox, Offline Receiving & Reply Capability Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 35  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Part 34 — Mixnet, Loopix, Sphinx, Nym & High-Anonymity Transport Architecture  
**Primary purpose:** define the complete anonymous mailbox, offline receiving, reply capability, mailbox rotation, multi-device fanout, retention, quota, replay protection, recovery, abuse resistance, and provider-neutral Rust architecture for SIAR's high-anonymity transport plane.

---

# 1. Purpose

A high-anonymity transport is not sufficient if recipients must remain continuously online.

SIAR needs a way for:

```text
sender online now
recipient offline now
recipient reconnects later
message still arrives
```

without exposing:

```text
recipient AccountId
recipient DeviceId
recipient IP address
stable direct endpoint
normal relay address
```

to the sending side or provider unnecessarily.

The governing principle is:

> **Anonymous receiving must use unlinkable, rotatable, capability-based mailbox identities that are separate from SIAR's normal account/device identities.**

---

# 2. Architectural Position

```text
SIAR Messaging
     │
     ▼
Application E2EE
     │
     ▼
Anonymous Routing Layer
     │
     ▼
Anonymous Mailbox Layer
     │
 ┌───┴────────────────────┐
 │                        │
Deposit                 Retrieve
 │                        │
Mixnet                 Mixnet
 │                        │
Provider / Mailbox Service
 │                        │
 └────────────┬───────────┘
              ▼
        Recipient Device
```

---

# 3. Why a Dedicated Mailbox Layer

Anonymous transport addresses three different problems:

```text
1. anonymous path selection
2. anonymous packet forwarding
3. asynchronous recipient availability
```

The mailbox solves the third.

---

# 4. Mailbox Requirements

The mailbox system MUST support:

```text
offline recipients
encrypted-at-rest payloads
anonymous deposit
anonymous retrieval
rotatable mailbox identities
multi-device fanout
bounded retention
deduplication
expiry
quotas
anti-abuse
reply capabilities
provider failover
crash recovery
provider-neutral APIs
```

---

# 5. Non-Goals

The mailbox is not:

```text
the user's primary message database
the application source of truth
a replacement for SIAR local history
a place to store plaintext
a place to expose stable AccountId
```

---

# 6. Authoritative State

Authoritative message state remains:

```text
recipient local message store
+
SIAR delivery state machine
```

The mailbox is only a transport/storage relay.

---

# 7. Mailbox Identity

Each receiving endpoint uses:

```rust
pub struct AnonymousMailboxId(pub [u8; 32]);
```

This is separate from:

```text
AccountId
DeviceId
EndpointId
relay address
```

---

# 8. Mailbox Secret

Mailbox access requires secret material:

```rust
pub struct AnonymousMailboxSecret(SecretBytes);
```

Never expose it to:

```text
UI
plugins
logs
support bundles
```

---

# 9. Mailbox Descriptor

```rust
pub struct AnonymousMailboxDescriptor {
    pub mailbox_id: AnonymousMailboxId,
    pub provider: AnonymousProviderId,
    pub epoch: MailboxEpoch,
    pub expires_at: Timestamp,
    pub capabilities: MailboxCapabilities,
}
```

---

# 10. Mailbox Capability Set

```rust
pub struct MailboxCapabilities {
    pub supports_batch_fetch: bool,
    pub supports_ack: bool,
    pub supports_reply_tokens: bool,
    pub supports_rotation_overlap: bool,
}
```

---

# 11. Per-Device Mailboxes

Recommended:

```text
Device A → Mailbox A
Device B → Mailbox B
Device C → Mailbox C
```

Benefits:

```text
independent revocation
reduced correlation
device-local lifecycle
clean rotation
```

---

# 12. Account-Level Delivery

The sender may fan out:

```text
one application message
→ encrypted envelope for Device A
→ encrypted envelope for Device B
→ encrypted envelope for Device C
```

---

# 13. Device Discovery

The sender must not directly fetch stable device endpoints in a privacy-leaking way.

Instead, SIAR should distribute:

```text
anonymous mailbox descriptors
```

inside trusted E2EE/account metadata channels.

---

# 14. Mailbox Descriptor Distribution

Possible paths:

```text
existing E2EE conversation state
trusted multi-device sync
QR/device verification exchange
anonymous bootstrap
```

---

# 15. Descriptor Authenticity

Mailbox descriptors must be authenticated by trusted application identity/device state.

---

# 16. Mailbox Rotation

Mailboxes should rotate periodically or after security events.

---

# 17. Rotation Reasons

```text
time-based privacy rotation
device compromise
provider migration
account recovery
manual reset
high-risk event
```

---

# 18. Mailbox Epoch

```rust
pub struct MailboxEpoch(pub u64);
```

---

# 19. Rotation State

```rust
pub enum MailboxRotationState {
    Stable,
    PublishingNew,
    DualReceive,
    RetiringOld,
    Complete,
    Failed,
}
```

---

# 20. Dual-Receive Window

During rotation:

```text
old mailbox
+
new mailbox
```

may both remain active for a bounded overlap.

---

# 21. Bounded Overlap

Overlap minimizes:

```text
message loss
```

but increases:

```text
correlation surface
```

Therefore it must be short and policy-controlled.

---

# 22. Rotation Policy

```rust
pub struct MailboxRotationPolicy {
    pub max_epoch_age: Duration,
    pub overlap: Duration,
    pub rotate_on_provider_change: bool,
    pub rotate_on_security_event: bool,
}
```

---

# 23. Anonymous Deposit

Sender flow:

```text
Resolve recipient mailbox
→ E2EE encrypt
→ Wrap anonymous envelope
→ Route through mixnet
→ Deposit to mailbox
```

---

# 24. Deposit API

```rust
#[async_trait]
pub trait AnonymousMailboxProvider: Send + Sync {
    async fn deposit(
        &self,
        mailbox: &AnonymousMailboxDescriptor,
        envelope: AnonymousMailboxEnvelope,
    ) -> Result<MailboxDepositReceipt, MailboxError>;

    async fn fetch(
        &self,
        mailbox: &AnonymousMailboxDescriptor,
        cursor: Option<MailboxCursor>,
    ) -> Result<MailboxBatch, MailboxError>;

    async fn acknowledge(
        &self,
        mailbox: &AnonymousMailboxDescriptor,
        items: &[MailboxItemId],
    ) -> Result<(), MailboxError>;
}
```

---

# 25. Mailbox Envelope

```rust
pub struct AnonymousMailboxEnvelope {
    pub version: MailboxEnvelopeVersion,
    pub item_id: MailboxItemId,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    pub body: Bytes,
}
```

---

# 26. Payload Rule

`body` contains:

```text
already E2EE-protected SIAR payload
```

Mailbox provider must never receive plaintext application content.

---

# 27. Deposit Receipt

```rust
pub struct MailboxDepositReceipt {
    pub item_id: MailboxItemId,
    pub accepted_at: Timestamp,
    pub retention_until: Timestamp,
}
```

---

# 28. Delivery Semantics

Mailbox deposit means:

```text
provider accepted for offline delivery
```

It does **not** mean:

```text
recipient device received
recipient user read
```

---

# 29. Delivery State Model

```rust
pub enum AnonymousMailboxDeliveryState {
    Queued,
    Submitted,
    Deposited,
    Retrieved,
    ApplicationAccepted,
    Expired,
    Failed,
}
```

---

# 30. User-Facing Simplification

Possible:

```text
Queued
Sending anonymously
Stored for recipient
Delivered
Expired
```

---

# 31. Retrieval

Recipient flow:

```text
start anonymity client
→ fetch mailbox batch
→ validate envelope
→ deduplicate
→ decrypt E2EE
→ persist message
→ acknowledge mailbox item
```

---

# 32. Persist Before ACK

Hard rule:

```text
mailbox item
→ validate
→ persist application state
→ ACK mailbox
```

Never ACK before durable local persistence.

---

# 33. Crash Safety

If crash occurs after persist but before ACK:

```text
item may reappear
```

Application dedup handles it.

---

# 34. Deduplication

Two layers:

```text
MailboxItemId
Application MessageId
```

---

# 35. Mailbox Item Dedup

Prevents repeated processing of same transport item.

---

# 36. Application Message Dedup

Prevents duplicate user-visible messages across:

```text
mailbox retry
provider retry
multi-device sync
route duplication
```

---

# 37. Batch Retrieval

Use bounded batches.

Example:

```text
32–128 mailbox items
```

benchmark-dependent.

---

# 38. Batch Limits

Bound:

```text
item count
total bytes
processing time
```

---

# 39. Retrieval Cursor

```rust
pub struct MailboxCursor {
    pub opaque: Bytes,
}
```

Provider-specific opaque cursor hidden behind adapter.

---

# 40. Cursor Privacy

Do not encode:

```text
AccountId
DeviceId
contact identity
```

into provider-visible cursor if avoidable.

---

# 41. Retention

Mailbox items must expire.

---

# 42. Retention Policy

```rust
pub struct MailboxRetentionPolicy {
    pub default_ttl: Duration,
    pub max_ttl: Duration,
    pub max_items: u32,
    pub max_bytes: u64,
}
```

---

# 43. Expiry

Expired transport item:

```text
deleted from mailbox
```

Application may still retain sender-side local message state.

---

# 44. Expired Delivery UX

Correct:

```text
Message expired before the recipient retrieved it.
```

---

# 45. Mailbox Capacity

Each mailbox has bounded:

```text
item count
bytes
retention
rate
```

---

# 46. Queue Full

Sender may receive:

```text
MailboxFull
```

---

# 47. Queue Full Behavior

Do not:

```text
silently switch to direct path
```

under strict anonymity.

---

# 48. Mailbox Quota Error

```rust
pub enum MailboxError {
    Unavailable,
    Full,
    Expired,
    InvalidDescriptor,
    AuthenticationFailed,
    RateLimited,
    Unsupported,
    Internal,
}
```

---

# 49. Sender-Side Retry

Use durable outbox.

---

# 50. Retry Backoff

Exponential/jittered retry.

---

# 51. Expiry-Aware Retry

Stop retry once application expiry reached.

---

# 52. Anonymous Reply Capability

A recipient should be able to reply without learning sender's network location.

---

# 53. Reply Capability Model

```rust
pub struct AnonymousReplyCapability {
    pub id: ReplyCapabilityId,
    pub provider: AnonymousProviderId,
    pub secret: SecretBytes,
    pub expires_at: Timestamp,
    pub max_uses: u32,
}
```

---

# 54. Reply Capability Properties

Desired:

```text
unforgeable
bounded
expiring
non-guessable
minimally linkable
```

---

# 55. SURB-Style Replies

Where provider supports:

```text
Single-Use Reply Blocks
```

should be preferred for one-shot anonymous replies.

---

# 56. Reply Capability Use

```text
incoming anonymous message
→ includes reply capability
→ recipient sends response
→ sender receives without revealing stable address
```

---

# 57. Reply Capability Storage

Sensitive local storage.

---

# 58. Never Sync Blindly

Reply capabilities should not be automatically replicated across devices.

---

# 59. Reply Capability State

```rust
pub enum ReplyCapabilityState {
    Available,
    Reserved,
    Consumed,
    Expired,
    Invalid,
}
```

---

# 60. Reserve Before Use

To avoid concurrent double-use:

```text
Available
→ Reserved
→ Consumed
```

---

# 61. Crash During Use

If outcome uncertain:

```text
mark capability spent
```

for single-use capability unless provider supports safe replay semantics.

---

# 62. Reply Pool

Sender may pre-generate a bounded pool.

---

# 63. Reply Pool Policy

```rust
pub struct ReplyPoolPolicy {
    pub target_size: u32,
    pub min_size: u32,
    pub max_size: u32,
    pub refill_threshold: u32,
}
```

---

# 64. Refill Timing

Avoid:

```text
refill immediately after each incoming message
```

because that may create timing correlation.

Use:

```text
background batched replenishment
```

---

# 65. Reply Capability Rotation

Expired/unused capabilities are discarded.

---

# 66. Reply Capability Revocation

Usually not guaranteed after distribution.

Therefore:

```text
short lifetimes
single-use
limited use-count
```

are preferred.

---

# 67. Bidirectional Conversation

For ongoing anonymous conversation:

```text
each side periodically supplies fresh reply capabilities
```

---

# 68. Avoid Stable Anonymous Address When Possible

Stable anonymous addresses increase correlation.

Use:

```text
rotating mailbox descriptors
reply capabilities
```

where practical.

---

# 69. Anonymous Invitation

A contact invitation may include:

```text
temporary mailbox descriptor
reply capabilities
ephemeral identity proof
```

---

# 70. Invitation Expiry

Short-lived.

---

# 71. Anonymous Bootstrap State

```rust
pub enum AnonymousBootstrapState {
    Created,
    Waiting,
    Contacted,
    Authenticated,
    Upgraded,
    Expired,
}
```

---

# 72. Upgrade to Persistent Relationship

Once peers establish trust:

```text
rotate/bootstrap mailbox
→ issue conversation-specific mailbox descriptors
```

---

# 73. Mailbox Scope

Possible:

```rust
pub enum MailboxScope {
    Device,
    Conversation,
    Invitation,
    EphemeralSession,
}
```

---

# 74. Device-Scoped Mailbox

Simplest.

One anonymous receiving mailbox per device.

---

# 75. Conversation-Scoped Mailbox

Stronger separation.

Each conversation gets separate anonymous address.

Tradeoff:

```text
more privacy
more key/state management
```

---

# 76. Invitation-Scoped Mailbox

Temporary.

---

# 77. Ephemeral Session Mailbox

Short-lived for sensitive exchanges.

---

# 78. Recommended Initial Scope

Start with:

```text
per-device mailbox
```

Then add:

```text
conversation-scoped
```

for Maximum Anonymity.

---

# 79. Multi-Device Fanout

Application determines target device set.

---

# 80. Fanout Policy

```rust
pub enum AnonymousFanoutPolicy {
    AllTrustedDevices,
    ActiveDevicesOnly,
    PrimaryDeviceOnly,
    ExplicitDevices,
}
```

---

# 81. Privacy Tradeoff

Sending to many mailboxes increases:

```text
traffic volume
correlation possibilities
```

---

# 82. Maximum Anonymity Fanout

May prefer:

```text
single privacy-designated receiving device
```

or carefully batched fanout.

---

# 83. Device Capability

Each device advertises whether it supports:

```text
anonymous receive
anonymous replies
cover traffic
background mailbox fetch
```

---

# 84. Capability Negotiation

```rust
pub struct AnonymousDeviceCapabilities {
    pub mailbox_receive: bool,
    pub reply_capability: bool,
    pub background_fetch: bool,
    pub provider_ids: Vec<AnonymousProviderId>,
}
```

---

# 85. Provider Compatibility

Two peers need not use same normal transport stack.

Anonymous delivery requires at least one compatible provider path.

---

# 86. Provider Migration

Mailbox descriptors are provider-specific.

---

# 87. Migration Flow

```text
create new provider mailbox
→ publish new descriptor
→ dual receive
→ retire old mailbox
```

---

# 88. Migration Invariant

No message loss during valid overlap where possible.

---

# 89. Provider Failure

If provider mailbox unavailable:

```text
sender outbox waits
recipient fetch retries
```

---

# 90. Multi-Provider Redundancy

Possible advanced mode:

```text
publish descriptors for Provider A and Provider B
```

---

# 91. Redundancy Tradeoff

Improves availability but increases:

```text
traffic
metadata surface
complexity
```

---

# 92. Multi-Provider Policy

```rust
pub enum MailboxProviderPolicy {
    SinglePreferred,
    Failover,
    Redundant,
}
```

---

# 93. Failover

Try another approved anonymous provider.

---

# 94. No Cross-Class Failover

Never fail over to:

```text
ordinary relay
direct endpoint
```

under strict anonymity.

---

# 95. Polling

Mailbox retrieval can create timing fingerprints.

---

# 96. Retrieval Scheduling

Options:

```text
continuous provider-native receive
Poisson-like polling
batched periodic polling
push-assisted anonymous wake
```

provider dependent.

---

# 97. Push Wake

Ordinary push notifications can leak metadata.

---

# 98. Push Policy

If used, push should contain:

```text
opaque wake hint only
```

and never:

```text
sender
message ID
conversation ID
mailbox content
```

---

# 99. Maximum Anonymity Push

May disable ordinary push entirely.

---

# 100. Background Fetch Android

Must respect:

```text
Doze
battery saver
foreground-service limits
network restrictions
```

---

# 101. Background Fetch Desktop

Daemon can maintain receive loop.

---

# 102. Fetch Scheduler

```rust
pub trait AnonymousMailboxScheduler {
    async fn schedule_fetch(
        &self,
        mailbox: AnonymousMailboxId,
        policy: MailboxFetchPolicy,
    ) -> Result<(), MailboxError>;
}
```

---

# 103. Fetch Policy

```rust
pub enum MailboxFetchPolicy {
    Continuous,
    Adaptive,
    BatterySaving,
    MaximumAnonymity,
}
```

---

# 104. Adaptive Fetch

Uses:

```text
battery
foreground state
network cost
provider health
```

without switching to lower privacy transport.

---

# 105. Cover Traffic Interaction

Mailbox polling should be integrated with cover traffic.

---

# 106. Loop Traffic

A Loopix-inspired implementation may use loop messages to:

```text
measure health
maintain cover
mask activity
```

---

# 107. Keepalive Privacy

Avoid direct heartbeat pattern tied to user activity.

---

# 108. Mailbox ACK Privacy

ACK behavior itself can leak timing.

---

# 109. Batched ACK

Potentially:

```text
batch acknowledgements
```

if provider semantics allow.

---

# 110. ACK Delay

Can be randomized within bounded window.

---

# 111. Read Receipt Separation

Mailbox ACK is:

```text
transport acknowledgement
```

not:

```text
read receipt
```

---

# 112. Delivery Receipt Separation

Application may send separate E2EE delivery receipt later.

---

# 113. Replay Protection

Required at multiple layers.

---

# 114. Replay Sources

```text
malicious provider
malicious sender
network duplication
retry duplication
```

---

# 115. Replay Cache

Bounded cache:

```rust
pub struct ReplayCachePolicy {
    pub max_entries: u32,
    pub retention: Duration,
}
```

---

# 116. Replay Key

Use:

```text
MailboxItemId
anonymous envelope nonce/id
MessageId
```

as appropriate.

---

# 117. Expired Replay Entries

Evict after safe retention.

---

# 118. Abuse Resistance

Mailbox service must survive hostile senders.

---

# 119. Threats

```text
mailbox flooding
large-message flooding
fragment exhaustion
reply-token abuse
CPU exhaustion
storage exhaustion
```

---

# 120. Anonymous Quotas

Quota should avoid requiring stable real-world identity.

---

# 121. Capability-Based Deposit

Preferred:

```text
deposit token
mailbox capability
invitation capability
```

---

# 122. Deposit Capability

```rust
pub struct MailboxDepositCapability {
    pub token: SecretBytes,
    pub expires_at: Timestamp,
    pub quota_class: MailboxQuotaClass,
}
```

---

# 123. Unknown Senders

For anonymous requests:

```text
small quota
no auto-download
request inbox
```

---

# 124. Trusted Contacts

Can receive higher capability quota.

---

# 125. Rate Limit State

```rust
pub enum MailboxQuotaClass {
    Unknown,
    Contact,
    Trusted,
    System,
}
```

---

# 126. Attachment Quotas

Anonymous unknown sender attachment defaults:

```text
metadata only
manual download
strict size bound
```

---

# 127. Fragment Flood Defense

Bound incomplete assemblies per:

```text
mailbox
source capability class
time window
```

---

# 128. CPU Budget

Decrypt/validate in bounded worker pool.

---

# 129. Mailbox Storage Budget

Never allow mailbox processing to exhaust device storage.

---

# 130. Local Staging

Fetched encrypted items may be staged temporarily before validation.

---

# 131. Staging Limits

Bound bytes/count/time.

---

# 132. Invalid Envelope

Drop/quarantine according to security policy.

---

# 133. Corrupt Envelope

Do not retry forever.

---

# 134. Authentication Failure

Treat as security event if repeated.

---

# 135. Unknown Version

```text
unsupported
```

not parse heuristically.

---

# 136. Versioning

```rust
pub struct MailboxEnvelopeVersion(pub u16);
```

---

# 137. Forward Compatibility

Unknown optional fields ignored only if authenticated schema permits.

---

# 138. Backward Compatibility

Maintain explicit migration matrix.

---

# 139. Backup

Do not blindly back up:

```text
live mailbox secrets
live reply tokens
provider session keys
```

---

# 140. Backup-Safe State

Can include:

```text
mailbox policy
provider preference
rotation metadata
non-secret descriptor history
```

---

# 141. Restore to New Device

Generate fresh:

```text
MailboxId
MailboxSecret
reply pool
provider identity
```

---

# 142. Same-Device Restore

May preserve more state if secure key-store continuity is verified.

---

# 143. Recovery

Account recovery does not imply old anonymous mailbox recovery.

---

# 144. Recovery Policy

Recommended:

```text
fresh anonymous identity after account recovery
```

---

# 145. Lost Device

Revoke device and retire its mailbox where provider supports.

---

# 146. Revocation Limit

Provider may not be able to erase already-delivered/distributed descriptors immediately.

Therefore rotation/expiry remain important.

---

# 147. Device Revocation Event

```text
revoke device
→ stop fetch
→ rotate account mailbox descriptors
→ invalidate future fanout
```

---

# 148. Multi-Device Sync

Sync only:

```text
public mailbox descriptors
capability metadata
rotation epochs
```

through E2EE.

---

# 149. Do Not Sync Mailbox Secrets

Hard rule unless architecture explicitly requires shared mailbox.

---

# 150. Conversation-Specific Mailbox Secrets

Remain device-local.

---

# 151. Database Schema

Potential local tables:

```text
anonymous_mailboxes
anonymous_mailbox_epochs
anonymous_outbox
anonymous_inbox_staging
anonymous_replay_cache
anonymous_reply_capabilities
anonymous_reply_pool
anonymous_provider_state
```

---

# 152. `anonymous_mailboxes`

Fields:

```text
mailbox_id
provider_id
scope
epoch
state
created_at
expires_at
secret_ref
```

---

# 153. Secret Reference

Database stores:

```text
secure-store reference
```

not raw secret.

---

# 154. `anonymous_outbox`

Stores:

```text
delivery_id
recipient mailbox ref
encrypted payload ref
state
retry metadata
expiry
```

---

# 155. `anonymous_inbox_staging`

Stores only bounded temporary transport state.

---

# 156. `anonymous_reply_capabilities`

Stores:

```text
capability id
provider
state
expiry
max uses
secret ref
```

---

# 157. Transactionality

Critical transitions:

```text
fetch
persist message
mark dedup
ACK
```

must be crash-safe.

---

# 158. Transaction Flow

Recommended:

```text
begin local transaction
→ validate mailbox item
→ dedup check
→ persist E2EE envelope/message
→ mark MailboxItemId processed
→ commit
→ send mailbox ACK
```

---

# 159. ACK Failure

If local commit succeeded:

```text
safe to retry ACK
```

---

# 160. Duplicate Fetch After ACK Failure

Dedup suppresses user duplicate.

---

# 161. Metrics

Privacy-safe:

```text
mailbox fetch success
mailbox deposit success
queue depth
batch size bucket
expired items
replay drops
provider availability
```

---

# 162. Forbidden Metrics

Do not record:

```text
peer mailbox ID
peer AccountId mapping
reply token
full timing trace per contact
```

---

# 163. Logs

Allowed:

```text
provider unavailable
mailbox fetch batch count
retry scheduled
```

---

# 164. Logs Must Redact

```text
mailbox IDs
secrets
reply capabilities
provider routes
```

---

# 165. Diagnostics UX

Normal:

```text
Anonymous receiving: Ready
Messages waiting: 2
Provider: Available
```

---

# 166. Advanced Diagnostics

Could show:

```text
mailbox epoch
queue depth
last fetch success
provider state
reply pool health
```

with identifiers truncated or omitted.

---

# 167. No Route Hop Display

Still forbidden.

---

# 168. Health Model

```rust
pub enum MailboxHealth {
    Healthy,
    Degraded,
    Unavailable,
    RotationRequired,
    QuotaLimited,
}
```

---

# 169. Reply Pool Health

```rust
pub enum ReplyPoolHealth {
    Healthy,
    Low,
    Empty,
    Replenishing,
}
```

---

# 170. UI Warning

If reply pool exhausted:

```text
Anonymous replies may be delayed while reply capabilities are replenished.
```

---

# 171. Offline UX

Recipient offline is normal.

Do not show sender:

```text
recipient offline
```

if anonymity mode avoids presence disclosure.

---

# 172. Sender UX

Use:

```text
Stored for delivery
```

not:

```text
Recipient is offline
```

---

# 173. Presence Independence

Mailbox status must not become a hidden presence oracle.

---

# 174. Fetch Timing Privacy

Sender should not learn exact mailbox retrieval timestamp unless application receipt policy allows.

---

# 175. Delivery Receipt Delay

High-anonymity mode may batch/delay application-level delivery receipts.

---

# 176. Read Receipt

Off by default in Maximum Anonymity.

---

# 177. Group Messaging

Anonymous group delivery may require:

```text
fanout to device mailboxes
```

or future group-anonymity protocol.

---

# 178. Group Fanout Cost

Potentially expensive:

```text
N members × devices
```

---

# 179. Initial Group Scope

Recommend:

```text
small groups only
```

for anonymous mailbox fanout.

---

# 180. Large Groups

Require separate architecture later.

---

# 181. Broadcast

Do not treat mailbox system as unbounded broadcast system.

---

# 182. Emergency Messaging

Anonymous mailbox may be too latent for emergency-critical delivery.

---

# 183. Emergency Policy

Use only if user explicitly selects anonymity over reachability.

---

# 184. File Attachments

Small files may be fragmented through mailbox.

---

# 185. Large Files

Use Part 34 anonymous attachment policy.

---

# 186. Attachment Mailbox Quota

Strict.

---

# 187. Preview Fetch

No automatic external preview fetch.

---

# 188. Plugin Access

Plugins cannot read:

```text
MailboxId
MailboxSecret
ReplyCapability
```

---

# 189. Plugin Event

At most:

```text
anonymous transport available/unavailable
```

if capability permits.

---

# 190. API Layer

Core mailbox service:

```rust
#[async_trait]
pub trait AnonymousMailboxService: Send + Sync {
    async fn create_mailbox(
        &self,
        scope: MailboxScope,
        provider: AnonymousProviderId,
    ) -> Result<AnonymousMailboxDescriptor, MailboxError>;

    async fn rotate_mailbox(
        &self,
        mailbox: AnonymousMailboxId,
    ) -> Result<AnonymousMailboxDescriptor, MailboxError>;

    async fn send_to_mailbox(
        &self,
        request: AnonymousMailboxSendRequest,
    ) -> Result<AnonymousDeliveryId, MailboxError>;

    async fn fetch_pending(
        &self,
        mailbox: AnonymousMailboxId,
    ) -> Result<MailboxFetchResult, MailboxError>;

    async fn retire_mailbox(
        &self,
        mailbox: AnonymousMailboxId,
    ) -> Result<(), MailboxError>;
}
```

---

# 191. Reply Service

```rust
#[async_trait]
pub trait AnonymousReplyService: Send + Sync {
    async fn generate(
        &self,
        policy: ReplyCapabilityPolicy,
    ) -> Result<Vec<AnonymousReplyCapability>, ReplyError>;

    async fn send_reply(
        &self,
        capability: ReplyCapabilityId,
        envelope: Bytes,
    ) -> Result<AnonymousDeliveryId, ReplyError>;

    async fn replenish(
        &self,
    ) -> Result<(), ReplyError>;
}
```

---

# 192. Reply Policy

```rust
pub struct ReplyCapabilityPolicy {
    pub single_use: bool,
    pub lifetime: Duration,
    pub max_uses: u32,
}
```

---

# 193. Mailbox Resolver

```rust
pub trait AnonymousMailboxResolver {
    async fn resolve_for_device(
        &self,
        device: DeviceId,
    ) -> Result<Vec<AnonymousMailboxDescriptor>, ResolveError>;
}
```

---

# 194. Resolver Privacy

Resolution occurs from:

```text
trusted locally available E2EE metadata
```

where possible.

---

# 195. No Public Account Lookup

Maximum anonymity should avoid:

```text
AccountId → public mailbox query
```

if it creates stable correlation.

---

# 196. Capability Distribution

Preferred:

```text
directly inside established E2EE conversation
```

---

# 197. Testkit

Create:

```text
siar-anonymity-mailbox-testkit
```

or include in:

```text
siar-anonymity-testkit
```

---

# 198. Fake Mailbox Provider

Supports:

```text
deposit
fetch
ack
drop
duplicate
delay
quota
expiry
rotation
```

---

# 199. Unit Tests

Test:

```text
mailbox creation
rotation
retention
quota
dedup
ACK ordering
reply reservation
```

---

# 200. Crash Tests

Critical:

```text
crash after fetch
crash after local persist
crash before ACK
crash during rotation
crash during reply use
```

---

# 201. Replay Tests

```text
same item twice
same reply capability twice
same E2EE message via two mailboxes
```

---

# 202. Expiry Tests

```text
expired mailbox
expired item
expired reply capability
```

---

# 203. Quota Tests

```text
mailbox full
sender rate-limited
fragment quota exceeded
```

---

# 204. Provider Failure Tests

```text
deposit unavailable
fetch unavailable
ACK unavailable
rotation unavailable
```

---

# 205. Rotation Tests

```text
new descriptor published
dual receive
old mailbox retired
late message on old mailbox
```

---

# 206. Multi-Device Tests

```text
two mailboxes
one device revoked
one device offline
dedup across devices
```

---

# 207. Recovery Tests

```text
new device gets fresh mailbox
old mailbox not restored blindly
```

---

# 208. Leak Tests

Canary secrets must never appear in:

```text
logs
support bundle
UI
plugin event
```

---

# 209. Correlation Tests

Verify no obvious field maps:

```text
AccountId
→ MailboxId
```

in provider-facing protocol metadata.

---

# 210. Fuzzing

Fuzz:

```text
mailbox envelope parser
batch parser
reply capability parser
cursor parser
rotation metadata
```

---

# 211. Property Tests

Properties:

```text
ACK never precedes durable persist
duplicate mailbox item never duplicates application message
expired reply capability cannot be used
retired mailbox cannot accept new sends after retirement policy completes
```

---

# 212. Performance Tests

Measure:

```text
batch fetch throughput
dedup lookup
reply pool replenishment
rotation overhead
storage growth
```

---

# 213. Battery Tests

Android:

```text
continuous receive
adaptive fetch
Doze
battery saver
```

---

# 214. Scale Tests

Synthetic:

```text
10k pending mailbox items
1k mailboxes
large replay cache
large outbox
```

within configured supported limits.

---

# 215. Security Invariants

Mandatory:

```text
1. Mailbox IDs never equal AccountId or DeviceId.
2. Mailbox secrets never leave secure storage.
3. E2EE payload exists before mailbox deposit.
4. Local durable persist occurs before ACK.
5. Application MessageId handles duplicate transport delivery.
6. Reply capabilities are secret, bounded, expiring, and preferably single-use.
7. Strict anonymity never falls back to ordinary direct/relay delivery.
8. Rotation overlap is bounded.
9. Account recovery creates fresh anonymous receiving identity.
10. Plugins never receive mailbox secrets or reply capabilities.
```

---

# 216. Initial Production Scope

Implement first:

```text
per-device mailbox
Nym-backed provider adapter
durable deposit/fetch
batch retrieval
persist-before-ACK
MailboxItemId + MessageId dedup
bounded retention
quota handling
rotation epochs
single-use reply capabilities
privacy-safe diagnostics
```

Then add:

```text
conversation-scoped mailboxes
multi-provider failover
adaptive fetch
reply pools
anonymous invitation mailboxes
advanced cover-aware polling
```

---

# 217. Definition of Done

Part 35 is complete when:

- anonymous mailbox identity is separate from account/device identity
- per-device anonymous mailbox lifecycle is defined
- deposit/fetch/ack APIs are provider-neutral
- offline recipient delivery works without direct endpoint exposure
- fetched messages are persisted before mailbox ACK
- duplicate transport delivery cannot create duplicate user-visible messages
- mailbox retention, expiry, quotas, and capacity limits are explicit
- mailbox rotation includes bounded overlap and retirement
- reply capabilities/SURB-style replies are bounded, secret, and replay-safe
- reply pool replenishment avoids obvious timing correlation
- multi-device fanout and revocation behavior are defined
- provider migration and optional redundancy are defined
- Android background/Doze constraints and desktop daemon receiving are defined
- backup/recovery never blindly restores stale live mailbox secrets
- plugin access to mailbox secrets is forbidden
- diagnostics and metrics are privacy-safe
- crash, replay, quota, rotation, multi-device, recovery, fuzz, leak, and performance tests are defined

---

# 218. Final Architecture

```text
                        SENDER
                          │
                          ▼
                    Application E2EE
                          │
                          ▼
                 Anonymous Transport
                          │
                          ▼
                 Anonymous Mailbox
                    ┌─────┴─────┐
                    │           │
                 Deposit     Retention
                    │           │
                    └─────┬─────┘
                          ▼
                  Recipient Fetch
                          │
                          ▼
                    Validate/Dedup
                          │
                          ▼
                    Persist Locally
                          │
                          ▼
                     Mailbox ACK
```

Reply path:

```text
Sender
  │
  ├─ includes bounded reply capability
  │
  ▼
Recipient
  │
  └─ replies using capability
          │
          ▼
       Mixnet
          │
          ▼
        Sender
```

Rotation path:

```text
Old Mailbox
    │
    ├──── bounded overlap ────┐
    │                         │
    ▼                         ▼
Retiring                 New Mailbox
                              │
                              ▼
                           Stable
```

---

# 219. Final Principle

Anonymous offline delivery should not require stable public addressing.

The correct model is:

```text
rotatable anonymous mailbox
+
E2EE payload
+
bounded retention
+
persist-before-ACK
+
single-use reply capabilities
+
strict separation from AccountId/DeviceId
```

not:

```text
public user ID
→ permanent mailbox address
→ provider tracks everything
```

This gives SIAR durable asynchronous messaging over the high-anonymity transport plane while preserving offline delivery, multi-device support, crash safety, and strong metadata separation.
