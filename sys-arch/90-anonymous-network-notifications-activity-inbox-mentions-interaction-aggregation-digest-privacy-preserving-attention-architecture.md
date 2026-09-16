# Core System Architecture Part 90 — Anonymous Network Notifications, Activity Inbox, Mentions, Interaction Aggregation, Digest & Privacy-Preserving Attention Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 90  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 17, 30–31, 43–46, 56, 61, 67–69, 74–75, 81–89  

**Primary purpose:** define SIAR's notification, activity-inbox, interaction-aggregation, mention/reply/reaction delivery, push-wake, local attention state, digest, quiet-hours, priority, batching, deduplication, privacy-preserving attention, managed-policy, and lifecycle architecture without turning notifications into a behavioral surveillance or engagement-optimization system.

---

# 1. Purpose

Notifications sit between system events and human attention.

Poor notification architecture causes:

```text
spam
privacy leaks
duplicated alerts
missed critical events
lock-screen exposure
engagement manipulation
server-side behavior profiling
```

The governing principle is:

> **SIAR notifications should deliver necessary attention signals while revealing the minimum metadata possible, respecting user control, batching low-value activity, and never becoming a mechanism for maximizing compulsive engagement.**

---

# 2. Architectural Position

```text
Domain / Interaction Event
          │
          ▼
 Notification Eligibility
          │
          ▼
 Aggregation / Priority
          │
          ├── Local Activity Inbox
          ├── Push Wake Hint
          ├── Digest
          └── Managed Alert
```

---

# 3. Core Separation

Keep distinct:

```text
domain event
notification event
activity inbox item
push wake hint
user attention state
digest item
read/seen state
delivery receipt
```

---

# 4. Non-Goals

Part 90 does not create:

```text
a global engagement timeline
a cloud database of every notification opened
behavioral push optimization
mandatory notification delivery for every event
lock-screen disclosure of private content
```

---

# 5. Notification Classes

```rust
pub enum NotificationClass {
    Security,
    Emergency,
    DirectCommunication,
    Mention,
    Reply,
    Reaction,
    SocialActivity,
    ContentUpdate,
    System,
    Managed,
}
```

---

# 6. Security

Examples:

```text
new login
identity-key change
recovery started
```

---

# 7. Emergency

High-priority emergency/SOS event.

---

# 8. Direct Communication

Direct message/call.

---

# 9. Mention

Explicit mention.

---

# 10. Reply

Reply to user content.

---

# 11. Reaction

Reaction to user content.

---

# 12. Social Activity

Join request, invite, group event.

---

# 13. Content Update

Subscribed feed/channel update.

---

# 14. System

Sync/error/storage/system event.

---

# 15. Managed

Organization-scoped alert.

---

# 16. Notification Priority

```rust
pub enum NotificationPriority {
    Critical,
    High,
    Normal,
    Low,
    Background,
}
```

---

# 17. Critical

Security/emergency.

---

# 18. High

Direct call/message depending user preference.

---

# 19. Normal

Mentions/replies.

---

# 20. Low

Reactions/general updates.

---

# 21. Background

Non-attention sync status.

---

# 22. Hard Rule

Priority is domain/policy-derived, not engagement-score-derived.

---

# 23. Notification Event

```rust
pub struct NotificationEvent {
    pub event_id: NotificationEventId,
    pub class: NotificationClass,
    pub priority: NotificationPriority,
    pub subject: NotificationSubjectRef,
    pub source: NotificationSourceRef,
    pub privacy: NotificationPrivacyClass,
}
```

---

# 24. Event ID

Random/opaque.

---

# 25. No AccountId Encoding

Hard rule.

---

# 26. Notification Privacy Class

```rust
pub enum NotificationPrivacyClass {
    PublicSafe,
    PrivateMetadata,
    Sensitive,
    WakeOnly,
}
```

---

# 27. PublicSafe

Could appear with preview.

---

# 28. PrivateMetadata

Minimal generic text.

---

# 29. Sensitive

No content preview.

---

# 30. WakeOnly

Push contains only opaque wake signal.

---

# 31. Default For E2EE Private Events

WakeOnly or Sensitive.

---

# 32. Hard Rule

Private content is fetched after wake through normal authorization.

---

# 33. Notification Source Ref

```rust
pub enum NotificationSourceRef {
    Relationship(RelationshipId),
    SocialSpace(SocialSpaceId),
    Thread(ThreadId),
    Content(ContentId),
    System(SystemNotificationSource),
}
```

---

# 34. No Login Identity Required

Hard rule.

---

# 35. Notification Subject Ref

What is affected.

---

# 36. Example:

```text
message
comment
security event
join request
```

---

# 37. Notification Projection

Domain events are transformed into minimal notification projections.

---

# 38. Hard Rule

Notification service does not receive full private domain objects unless required.

---

# 39. Projection Trait

```rust
pub trait NotificationProjection {
    fn project(
        &self,
        event: &DomainEvent,
    ) -> Option<NotificationEvent>;
}
```

---

# 40. Eligibility

Not all domain events generate notifications.

---

# 41. Eligibility Inputs

```text
user preference
relationship policy
privacy mode
quiet hours
managed policy
event priority
```

---

# 42. Eligibility Decision

```rust
pub enum NotificationEligibility {
    DeliverNow,
    QueueForDigest,
    InboxOnly,
    Suppress,
}
```

---

# 43. Suppression

Valid normal behavior.

---

# 44. Hard Rule

Suppressed notification does not imply lost domain event.

---

# 45. Activity Inbox

Durable user-facing notification center.

---

# 46. Activity Inbox Item

```rust
pub struct ActivityItem {
    pub activity_id: ActivityId,
    pub class: NotificationClass,
    pub source: NotificationSourceRef,
    pub created_order: ActivityOrderKey,
    pub state: ActivityState,
}
```

---

# 47. Activity State

```rust
pub enum ActivityState {
    Unseen,
    Seen,
    Read,
    Archived,
    Dismissed,
}
```

---

# 48. Unseen

Never presented.

---

# 49. Seen

Displayed.

---

# 50. Read

User opened relevant context.

---

# 51. Archived

Retained but hidden from active inbox.

---

# 52. Dismissed

Removed from active attention.

---

# 53. Hard Rule

Activity state is attention state, not domain-authority state.

---

# 54. Dismissing Notification

Does not delete message/comment/security event.

---

# 55. Hard rule.

---

# 56. Local-First Activity Inbox

Client keeps local inbox state.

---

# 57. Server May Store

Encrypted account-sync projection if user enables multi-device sync.

---

# 58. No Global Plaintext Activity History

Hard rule.

---

# 59. Activity Inbox Scope

```rust
pub enum ActivityInboxScope {
    DeviceLocal,
    AccountEncrypted,
    ManagedWorkspace,
}
```

---

# 60. DeviceLocal

Strong privacy default.

---

# 61. AccountEncrypted

Own-device sync.

---

# 62. ManagedWorkspace

Managed events only.

---

# 63. Personal And Managed Inbox

Separate views/storage.

---

# 64. Hard Rule

Organization policy cannot inspect unrelated personal activity inbox.

---

# 65. Aggregation

Important to reduce spam.

---

# 66. Aggregate Examples

```text
12 people reacted
3 replies in thread
5 updates from channel
```

---

# 67. Aggregation Key

```rust
pub enum AggregationKey {
    Thread(ThreadId),
    Content(ContentId),
    SocialSpace(SocialSpaceId),
    NotificationClass(NotificationClass),
}
```

---

# 68. No User-Based Global Aggregation Key

Hard rule.

---

# 69. Aggregation Window

```rust
pub struct AggregationWindow {
    pub duration: Duration,
    pub max_items: u32,
}
```

---

# 70. Reaction Aggregation

Strongly recommended.

---

# 71. Mention Aggregation

Limited.

---

# 72. Direct Messages

Usually not aggregated beyond conversation summary.

---

# 73. Emergency/Security

Never hidden in low-priority aggregate.

---

# 74. Hard Rule

Critical notifications cannot be downgraded into silent digest by generic aggregation.

---

# 75. Aggregated Activity

```rust
pub struct AggregatedActivity {
    pub aggregate_id: AggregateId,
    pub key: AggregationKey,
    pub count_class: CountClass,
    pub latest_order: ActivityOrderKey,
}
```

---

# 76. Public/Private Count Privacy

Use coarse count where small-cohort identity could leak.

---

# 77. No Public Actor List By Default

Hard rule.

---

# 78. Reaction Aggregation

Actor identities excluded from notification payload by default.

---

# 79. Replies

May include latest sender projection if relationship/group policy allows.

---

# 80. Mentions

Can identify mention actor in authorized scope.

---

# 81. Push Architecture

Push is wake infrastructure, not message transport.

---

# 82. Hard Rule

Push provider is not authoritative delivery channel.

---

# 83. Push Hint

```rust
pub struct PushWakeHint {
    pub hint_id: PushHintId,
    pub wake_class: PushWakeClass,
    pub opaque_locator: OpaqueWakeLocator,
}
```

---

# 84. Push Wake Class

```rust
pub enum PushWakeClass {
    Message,
    Call,
    Security,
    Activity,
    Sync,
}
```

---

# 85. Push Payload Must Not Contain

```text
message plaintext
private comment text
contact graph
auth token
mailbox secret
```

---

# 86. Hard rule.

---

# 87. Push Provider Tokens

Sensitive infrastructure metadata.

---

# 88. Device-scoped.

---

# 89. No user social identity encoded.

---

# 90. Push Token Lifecycle

```rust
pub enum PushTokenState {
    Active,
    Rotating,
    Revoked,
    Expired,
}
```

---

# 91. Token Rotation

Supported.

---

# 92. Device removal revokes token.

---

# 93. Account deletion revokes token.

---

# 94. Hard rule.

---

# 95. Push Registration

Bound to device/account session where account mode used.

---

# 96. Local-only mode

No push required.

---

# 97. Android

Platform push optional.

---

# 98. Desktop

Local daemon notification path.

---

# 99. No dependency on push for correctness.

---

# 100. Hard rule.

---

# 101. Notification Fetch Flow

```text
push wake
→ app wakes
→ authenticate local/device context
→ fetch mailbox/activity
→ authorize
→ decrypt locally
→ render
```

---

# 102. Push Provider Never Sees Decrypted Message

Hard rule.

---

# 103. Lock-Screen Privacy

Per-device.

---

# 104. Preview Policy

```rust
pub enum LockScreenPreviewPolicy {
    Hidden,
    SenderOnly,
    Generic,
    FullWhenUnlocked,
}
```

---

# 105. Default Sensitive

Generic/Hidden.

---

# 106. DeviceLocal Setting

Not centrally inferred.

---

# 107. Hard rule.

---

# 108. Mention Notification

Created by Part 89.

---

# 109. Must validate current access before rendering.

---

# 110. If parent deleted/revoked

Notification can become unavailable.

---

# 111. No stale content leak.

---

# 112. Hard rule.

---

# 113. Reply Notification

Thread scoped.

---

# 114. If user muted thread

InboxOnly or suppress.

---

# 115. Reaction Notification

Batch.

---

# 116. Default

Queue/aggregate.

---

# 117. No one alert per reaction storm.

---

# 118. Hard rule.

---

# 119. Reshare Notification

Optional.

---

# 120. Depends on creator preference.

---

# 121. Subscriber Updates

Usually feed/inbox/digest, not immediate push.

---

# 122. Hard Rule

New public content should not default to high-priority push merely to increase engagement.

---

# 123. Quiet Hours

User-controlled.

---

# 124. Quiet Hours Policy

```rust
pub struct QuietHoursPolicy {
    pub enabled: bool,
    pub start_local: LocalTime,
    pub end_local: LocalTime,
    pub bypass: BTreeSet<NotificationClass>,
}
```

---

# 125. DeviceLocal

By default.

---

# 126. Can sync semantic preference if user wants.

---

# 127. Timezone Local

Avoid central timezone tracking if unnecessary.

---

# 128. Hard rule.

---

# 129. Critical Bypass

User may allow security/emergency.

---

# 130. Managed Emergency

Can bypass managed quiet hours only if policy allows.

---

# 131. Hard Rule

Managed organization cannot bypass personal-profile quiet hours outside managed context.

---

# 132. Do Not Disturb

OS integration.

---

# 133. SIAR should respect OS DND.

---

# 134. No bypass unless platform APIs/user explicitly permit.

---

# 135. Hard rule.

---

# 136. Attention Budget

Optional user-controlled anti-spam mechanism.

---

# 137. Attention Budget

```rust
pub struct AttentionBudget {
    pub class: NotificationClass,
    pub max_interruptions: u16,
    pub window: Duration,
}
```

---

# 138. Interruptions

Push/sound/vibration.

---

# 139. Activity inbox still receives item.

---

# 140. Hard Rule

Exceeding budget downgrades interruption, not domain delivery.

---

# 141. Notification Channels

```rust
pub enum NotificationDeliveryChannel {
    InApp,
    OsNotification,
    PushWake,
    Email,
    ManagedWebhook,
}
```

---

# 142. InApp

Primary.

---

# 143. OS Notification

Local.

---

# 144. PushWake

Opaque wake.

---

# 145. Email

Optional external integration.

---

# 146. ManagedWebhook

Enterprise/admin events only.

---

# 147. Hard Rule

Private user interaction notifications never route to managed webhook unless explicitly managed-owned event.

---

# 148. Email Notification

Privacy-sensitive.

---

# 149. Default

Generic subject/body.

---

# 150. No private message plaintext.

---

# 151. User can choose richer preview if desired.

---

# 152. Hard rule.

---

# 153. Notification Preferences

Part 84 integration.

---

# 154. Global semantic preferences:

```text
mentions
replies
reactions
security
group updates
feed updates
```

---

# 155. Per-Contact Override

Allowed.

---

# 156. Per-Thread Override

Allowed.

---

# 157. Per-Space Override

Allowed.

---

# 158. Effective Preference

Most specific + privacy/security floor.

---

# 159. Managed Overlay

Managed context only.

---

# 160. Hard rule.

---

# 161. Notification Preference Model

```rust
pub struct NotificationPreference {
    pub class: NotificationClass,
    pub behavior: NotificationBehavior,
}
```

---

# 162. Behavior

```rust
pub enum NotificationBehavior {
    Interrupt,
    Silent,
    Digest,
    InboxOnly,
    Disabled,
}
```

---

# 163. Security Class

May not permit Disabled for critical compromise warning in managed profile.

---

# 164. But user personal account policy separate.

---

# 165. Hard rule.

---

# 166. Digest Architecture

Digests summarize low/normal-priority activity.

---

# 167. Digest Types

```rust
pub enum DigestType {
    Activity,
    SocialSpace,
    Feed,
    SecuritySummary,
    Managed,
}
```

---

# 168. Activity Digest

Replies/reactions/mentions.

---

# 169. Social Space Digest

Group updates.

---

# 170. Feed Digest

Subscribed public content.

---

# 171. Security Summary

Noncritical security events only.

---

# 172. Managed Digest

Org events.

---

# 173. Critical Security

Never delayed into digest only.

---

# 174. Hard rule.

---

# 175. Digest Schedule

```rust
pub enum DigestSchedule {
    Manual,
    Daily,
    Weekly,
    Custom,
}
```

---

# 176. User Local Time

Local/device scheduling preferred.

---

# 177. Server digest

Can operate on encrypted/minimal activity if enabled.

---

# 178. Strong privacy baseline

Generate digest locally.

---

# 179. Hard rule.

---

# 180. Digest Inputs

Only eligible items.

---

# 181. No Hidden Behavioral Ranking

---

# 182. Digest Ordering

```text
criticality
user pins
recency
```

---

# 183. Not dwell-time optimization.

---

# 184. Hard rule.

---

# 185. Digest Item

```rust
pub struct DigestItem {
    pub source: NotificationSourceRef,
    pub class: NotificationClass,
    pub summary: DigestSummaryRef,
}
```

---

# 186. Summary

Generated from authorized local content.

---

# 187. No central AI summary of private events by default.

---

# 188. Hard rule.

---

# 189. AI Digest

Optional on-device.

---

# 190. Input remains local.

---

# 191. No raw private notification upload.

---

# 192. Hard rule.

---

# 193. Digest Read State

Local.

---

# 194. No creator visibility.

---

# 195. No notification-open tracking by sender.

---

# 196. Hard rule.

---

# 197. Seen/Read State Synchronization

Privacy-sensitive.

---

# 198. Default

DeviceLocal.

---

# 199. Optional encrypted account sync.

---

# 200. Never exposed to notification source unless domain feature explicitly needs receipt.

---

# 201. Hard rule.

---

# 202. Delivery Receipt

Infrastructure-specific.

---

# 203. Means notification reached device/inbox.

---

# 204. Not same as read.

---

# 205. Not same as domain delivery.

---

# 206. Hard rule.

---

# 207. Notification Delivery State

```rust
pub enum NotificationDeliveryState {
    Pending,
    Queued,
    WakeSent,
    InboxDelivered,
    Displayed,
    Suppressed,
    FailedRetryable,
    FailedPermanent,
}
```

---

# 208. Displayed

OS/app rendered.

---

# 209. Does not mean user read.

---

# 210. Hard rule.

---

# 211. Notification Outbox

Transactional.

---

# 212. Domain event commit first.

---

# 213. Notification projection/outbox after committed state.

---

# 214. No notification before domain commit.

---

# 215. Hard rule.

---

# 216. Notification Worker

Idempotent.

---

# 217. Duplicate event

No duplicate visible alert.

---

# 218. Deduplication Key

```rust
pub struct NotificationDedupKey {
    pub event: NotificationEventId,
    pub device: DeviceId,
}
```

---

# 219. Device-scoped.

---

# 220. No global user identifier required.

---

# 221. Aggregation Dedup

Same reaction/update merged.

---

# 222. No counter inflation.

---

# 223. Hard rule.

---

# 224. Retry

Exponential backoff + jitter.

---

# 225. Push transient failure

Retry bounded.

---

# 226. Permanent invalid push token

revoke token.

---

# 227. No infinite queue.

---

# 228. Hard rule.

---

# 229. Queue Bounds

By:

```text
items
bytes
age
priority
```

---

# 230. Low-priority expired items can collapse into digest.

---

# 231. Critical items retained according to security policy.

---

# 232. No unbounded notification backlog.

---

# 233. Hard rule.

---

# 234. Offline Devices

Inbox sync after reconnect.

---

# 235. Push unavailable

No correctness loss.

---

# 236. Messages/domain events remain durable elsewhere.

---

# 237. Hard rule.

---

# 238. Expired Activity

Low-value event can expire.

---

# 239. Example

typing/presence do not become inbox items.

---

# 240. Hard Rule

Ephemeral realtime state never becomes durable notification history.

---

# 241. Call Notification

Incoming call special path.

---

# 242. Realtime.

---

# 243. Time-sensitive.

---

# 244. If missed

create missed-call activity.

---

# 245. No detailed media metadata in push.

---

# 246. Hard rule.

---

# 247. Emergency Notification

Part 17/41/etc.

---

# 248. Priority Critical.

---

# 249. Distinct UI.

---

# 250. No marketing/social aggregation with emergency.

---

# 251. Hard rule.

---

# 252. Security Notification

Examples:

```text
new device
key change
recovery
session replay
```

---

# 253. Security Event Source

Account/security service.

---

# 254. Security notifications stored locally/account encrypted.

---

# 255. No suppression by normal social mute.

---

# 256. Hard rule.

---

# 257. Identity-Key Change

High.

---

# 258. User should see before sending sensitive message where relevant.

---

# 259. No hidden dismiss-only path.

---

# 260. Account Lifecycle Integration

Deleted account:

```text
no new personal notifications
revoke push token
delete/expire inbox sync according to lifecycle
```

---

# 261. Pending deletion

only deletion/security management alerts.

---

# 262. Hard rule.

---

# 263. Suspended Account

Security/appeal/account alerts may still deliver.

---

# 264. Social/activity notifications may suppress.

---

# 265. Managed Offboarding

Managed notification channel revoked.

---

# 266. Personal notifications unaffected.

---

# 267. Hard rule.

---

# 268. Contact Integration

Part 85.

---

# 269. Blocked contact

no direct notifications.

---

# 270. Existing public content interactions

local block policy hides/suppresses.

---

# 271. Stale sync cannot re-enable.

---

# 272. Hard rule.

---

# 273. Social Space Integration

Part 86.

---

# 274. Space mute.

---

# 275. Mention can optionally override space mute.

---

# 276. User controls.

---

# 277. Ban/removal

future space notifications stop.

---

# 278. Hard rule.

---

# 279. Feed Integration

Part 87.

---

# 280. Public feed updates usually InboxOnly/Digest.

---

# 281. User can promote source.

---

# 282. No feed source self-promotes to critical.

---

# 283. Hard rule.

---

# 284. Publishing Integration

Part 88.

---

# 285. Creator publication can trigger follower/subscriber update.

---

# 286. Subscribers' preference determines delivery.

---

# 287. Publisher cannot know who got push.

---

# 288. Hard rule.

---

# 289. Interaction Integration

Part 89.

---

# 290. Mention/reply/reaction/reshare event projections.

---

# 291. No direct sender-controlled push priority beyond allowed domain bounds.

---

# 292. Hard rule.

---

# 293. Authorization Integration

Part 81.

---

# 294. Notification rendering always revalidates content access.

---

# 295. Notification itself is not authorization proof.

---

# 296. Hard rule.

---

# 297. Authentication Integration

Part 82.

---

# 298. Session state needed for account inbox sync.

---

# 299. Local notification display can work offline.

---

# 300. No push token as auth credential.

---

# 301. Hard rule.

---

# 302. Profile/Preference Integration

Part 84.

---

# 303. Semantic preferences synchronized optionally.

---

# 304. OS-specific sound/vibration local.

---

# 305. No server storage of device-specific notification behavior unless required.

---

# 306. Hard rule.

---

# 307. Federation Integration

Remote events arrive through federation gateway.

---

# 308. Convert to local notification projection.

---

# 309. No remote domain push provider access to local device token.

---

# 310. Hard rule.

---

# 311. Remote domain never receives local push token.

---

# 312. Federation Notification Flow

```text
remote event
→ federation gateway
→ local domain event
→ local notification projection
→ local delivery
```

---

# 313. No cross-domain direct push.

---

# 314. Hard rule.

---

# 315. Tenant/Managed Notifications

Organization can create managed alerts.

---

# 316. Scoped to managed profile.

---

# 317. Managed alert priority bounded by policy.

---

# 318. No "critical" flag self-selected by arbitrary sender.

---

# 319. Hard rule.

---

# 320. Managed Broadcast

Explicit elevated capability.

---

# 321. Emergency broadcast capability separate.

---

# 322. Rate limited.

---

# 323. Audit.

---

# 324. No unlimited org notification spam.

---

# 325. Hard rule.

---

# 326. Managed Notification Capability

```rust
pub struct ManagedNotificationCapability {
    pub organization: OrganizationId,
    pub classes: BTreeSet<NotificationClass>,
    pub max_priority: NotificationPriority,
    pub expires_at: Timestamp,
}
```

---

# 327. Notification Channels On Device

Examples:

```text
Messages
Calls
Mentions
Security
Groups
System
```

---

# 328. Android Notification Channels

Map semantic classes.

---

# 329. Desktop categories

Equivalent.

---

# 330. UI semantic model shared.

---

# 331. Platform rendering separate.

---

# 332. Hard rule.

---

# 333. Notification Grouping

OS-level.

---

# 334. Conversation grouping.

---

# 335. Thread grouping.

---

# 336. Reaction aggregation.

---

# 337. No user graph inference on server needed.

---

# 338. Local Notification Renderer

```rust
pub trait NotificationRenderer {
    fn render(
        &self,
        item: &ResolvedNotification,
        policy: &DeviceNotificationPolicy,
    ) -> Result<RenderedNotification, NotificationError>;
}
```

---

# 339. Rendering After Local Decryption

Preferred.

---

# 340. Hard rule.

---

# 341. Notification Resolution

```rust
pub trait NotificationResolver {
    fn resolve(
        &self,
        event: &NotificationEvent,
    ) -> Result<ResolvedNotification, NotificationError>;
}
```

---

# 342. Resolution checks:

```text
authorization
current deletion/revocation
block/mute state
profile projection
```

---

# 343. No stale preview.

---

# 344. Hard rule.

---

# 345. Activity Inbox Query

```rust
pub trait ActivityInboxRepository {
    fn page(
        &self,
        cursor: Option<ActivityCursor>,
        limit: usize,
    ) -> Result<ActivityPage, NotificationError>;
}
```

---

# 346. Bounded pagination.

---

# 347. No load-all activity history.

---

# 348. Hard rule.

---

# 349. Notification Preference Resolver

```rust
pub trait NotificationPreferenceResolver {
    fn behavior(
        &self,
        event: &NotificationEvent,
        context: &NotificationContext,
    ) -> Result<NotificationBehavior, NotificationError>;
}
```

---

# 350. Deterministic.

---

# 351. No behavioral engagement model.

---

# 352. Hard rule.

---

# 353. Aggregation Service

```rust
pub trait NotificationAggregator {
    fn ingest(
        &self,
        event: NotificationEvent,
    ) -> Result<AggregationDecision, NotificationError>;
}
```

---

# 354. Digest Builder

```rust
pub trait DigestBuilder {
    fn build(
        &self,
        eligible: &[ActivityItem],
        policy: &DigestPolicy,
    ) -> Result<NotificationDigest, NotificationError>;
}
```

---

# 355. No hidden content ranking model.

---

# 356. Digest Policy

```rust
pub struct DigestPolicy {
    pub schedule: DigestSchedule,
    pub included_classes: BTreeSet<NotificationClass>,
    pub max_items: u16,
}
```

---

# 357. Digest Size Bounded.

---

# 358. No Infinite Summary.

---

# 359. Notification Expiry

```rust
pub struct NotificationExpiryPolicy {
    pub class: NotificationClass,
    pub max_age: Duration,
}
```

---

# 360. Low-value short.

---

# 361. Security longer.

---

# 362. Expiry does not delete underlying domain event.

---

# 363. Hard rule.

---

# 364. Activity Retention

User-controlled/bounded.

---

# 365. Example:

```text
7 days
30 days
90 days
```

---

# 366. Security event retention may differ.

---

# 367. No indefinite attention history by default.

---

# 368. Hard rule.

---

# 369. Deletion / Retraction

If source content deleted:

```text
activity item becomes unavailable
preview removed
```

---

# 370. If interaction retracted:

```text
notification can collapse/remove body
```

---

# 371. No stale deleted-content preview.

---

# 372. Hard rule.

---

# 373. Notification Tombstone

Minimal.

---

# 374. Prevents stale sync from restoring.

---

# 375. Notification Deletion Epoch

```rust
pub struct NotificationDeletionEpoch(pub u64);
```

---

# 376. Monotonic.

---

# 377. Old inbox replicas rejected.

---

# 378. Hard rule.

---

# 379. Multi-Device Notification Sync

Possible sync:

```text
dismissed
read/seen
archive
mute preference
```

---

# 380. Default privacy:

read state local.

---

# 381. User may opt into encrypted sync.

---

# 382. Dismissed security warning

can sync if desired.

---

# 383. No sender visibility.

---

# 384. Hard rule.

---

# 385. Multi-Device Duplicate Avoidance

Own devices each may receive wake.

---

# 386. Activity inbox event shared logically.

---

# 387. Local renderer decides alert.

---

# 388. Optional "first active device wins" optimization risky for privacy.

---

# 389. Initial baseline

Allow per-device rendering based on local preference.

---

# 390. No centralized live-device attention tracking.

---

# 391. Hard rule.

---

# 392. Device Activity

Do not upload "screen currently on"/"user actively viewing app" by default.

---

# 393. Hard rule.

---

# 394. Foreground Suppression

Local app can suppress OS notification when open.

---

# 395. No server needs foreground state.

---

# 396. Good privacy.

---

# 397. Notification Action Buttons

Examples:

```text
Reply
Mark read
Mute
Accept call
```

---

# 398. Actions must reauthorize.

---

# 399. Notification action token

short-lived/local or audience-bound.

---

# 400. No long-lived auth token in notification payload.

---

# 401. Hard rule.

---

# 402. Quick Reply

Encrypt/sign via normal messaging path.

---

# 403. Notification shell is not message transport.

---

# 404. Hard rule.

---

# 405. Notification Deep Link

Opaque internal route.

---

# 406. Revalidates target.

---

# 407. No sensitive ID in external URL where avoidable.

---

# 408. Hard rule.

---

# 409. Email/External Digest

If enabled:

```text
generic summary
safe links
no secret tokens beyond single-purpose short-lived link
```

---

# 410. Authentication required to view private content.

---

# 411. Hard rule.

---

# 412. Unsubscribe Link

For optional email.

---

# 413. Scoped single-purpose token.

---

# 414. Does not grant account access.

---

# 415. Hard rule.

---

# 416. Notification Abuse

Threats:

```text
mention spam
reaction storms
group broadcast abuse
publisher push spam
managed alert abuse
```

---

# 417. Mention Spam

Part 89 rate limits.

---

# 418. Reaction Storms

Aggregate.

---

# 419. Group Broadcast

Elevated capability/rate limits.

---

# 420. Publisher Spam

Subscriber preferences + attention budget.

---

# 421. Managed Alert Abuse

Capability + audit + quota.

---

# 422. No sender override of user Disabled setting except security/emergency policy.

---

# 423. Hard rule.

---

# 424. Attention Safety

System should not optimize for:

```text
open rate
return frequency
time-on-app
notification-induced sessions
```

as primary goals.

---

# 425. Hard rule.

---

# 426. Product Metrics

Allowed aggregate technical metrics:

```text
delivery failure
push invalid token rate
digest generation latency
```

---

# 427. Avoid:

```text
which notification made user return
conversion from notification to session
individual open propensity
```

---

# 428. Hard rule.

---

# 429. Notification Recommendation

Not needed.

---

# 430. No ML to choose emotional/persuasive push text.

---

# 431. Hard rule.

---

# 432. Notification Copy

Domain-defined neutral templates.

---

# 433. Example:

```text
You have a new reply.
Security settings changed.
```

---

# 434. Avoid urgency inflation.

---

# 435. Only critical events use critical wording.

---

# 436. Hard rule.

---

# 437. Privacy Threat Model

Attacks:

```text
push-provider metadata correlation
lock-screen leakage
activity history reconstruction
sender graph inference
cross-device correlation
```

---

# 438. Push Provider Correlation

Opaque payloads, minimized token mapping.

---

# 439. Lock-Screen Leakage

Local preview policy.

---

# 440. Activity History

Bounded/encrypted/local.

---

# 441. Sender Graph

No centralized per-contact notification ledger.

---

# 442. Cross-Device Correlation

No global attention telemetry.

---

# 443. Hard rule.

---

# 444. Operational Architecture

Components:

```text
Notification Projector
Eligibility Engine
Aggregator
Activity Inbox
Push Wake Dispatcher
Digest Builder
Local Renderer
Preference Resolver
```

---

# 445. Notification Projector

Converts domain events.

---

# 446. Eligibility Engine

Applies semantics/preferences.

---

# 447. Aggregator

Combines low-value events.

---

# 448. Activity Inbox

Durable local/account state.

---

# 449. Push Wake Dispatcher

Opaque push hints.

---

# 450. Digest Builder

Local/optional account summary.

---

# 451. Local Renderer

Final privacy-aware presentation.

---

# 452. Preference Resolver

Effective notification intent.

---

# 453. No Single Central Notification Brain

Hard rule.

---

# 454. Service Boundaries

Server notification service knows:

```text
event class
device token
opaque locator
priority
```

---

# 455. Should not know:

```text
message plaintext
private profile details
social graph
precise user activity state
```

---

# 456. Hard rule.

---

# 457. Event Bus Integration

Part 75.

---

# 458. Domain events feed projector through scoped topics.

---

# 459. Notification service subscribes only needed projections.

---

# 460. No broad access to all private domain events.

---

# 461. Hard rule.

---

# 462. Database Integration

Part 74.

---

# 463. Server stores:

```text
push token registry
minimal delivery queue
optional encrypted activity sync
```

---

# 464. Client stores:

```text
activity inbox
seen/read state
local preferences
aggregates
```

---

# 465. No global notification analytics warehouse.

---

# 466. Hard rule.

---

# 467. Edge Integration

Part 78.

---

# 468. Push registration API L7.

---

# 469. Token validation.

---

# 470. No notification content inspection beyond external API metadata.

---

# 471. Internal Service Integration

Part 79.

---

# 472. Notification services use mTLS.

---

# 473. Secrets Integration

Part 80.

---

# 474. Push provider API key brokered/scoped.

---

# 475. No provider key in client.

---

# 476. Hard rule.

---

# 477. Authorization Integration

Part 81.

---

# 478. Notification action/deep link reauthorizes.

---

# 479. Authentication Integration

Part 82.

---

# 480. Session needed for account activity sync.

---

# 481. Push token not identity proof.

---

# 482. Hard rule.

---

# 483. Account Lifecycle Integration

Part 83.

---

# 484. Revocation/deletion/token cleanup.

---

# 485. Profile Integration

Part 84.

---

# 486. Notification semantic preferences.

---

# 487. Contact Integration

Part 85.

---

# 488. block/mute/contact-specific behavior.

---

# 489. Social Space Integration

Part 86.

---

# 490. space notification policies/managed broadcasts.

---

# 491. Feed Integration

Part 87.

---

# 492. feed digest/subscriber updates.

---

# 493. Publishing Integration

Part 88.

---

# 494. publication events.

---

# 495. Interaction Integration

Part 89.

---

# 496. replies/mentions/reactions/reshares.

---

# 497. Data Lifecycle

Part 67.

---

# 498. Activity/inbox retention.

---

# 499. Deleted source removes preview.

---

# 500. No stale restore.

---

# 501. Backup

Part 33.

---

# 502. Local inbox backup optional.

---

# 503. Push tokens excluded.

---

# 504. Read/unread optional.

---

# 505. Notification deletion epochs preserved if backed up.

---

# 506. Hard rule.

---

# 507. Federation

No remote domain receives local push token.

---

# 508. Remote event normalized through local domain.

---

# 509. Hard rule.

---

# 510. Serialization

Postcard internal.

---

# 511. RON preferences/config/export.

---

# 512. JSON external push provider API only.

---

# 513. Strict bounds.

---

# 514. No giant payload.

---

# 515. Push hint tiny.

---

# 516. Digest item count bounded.

---

# 517. Activity pagination bounded.

---

# 518. Hard rule.

---

# 519. Performance

Notification projection low-cost.

---

# 520. Aggregation in-memory + durable checkpoint.

---

# 521. Push dispatch async.

---

# 522. Digest batch.

---

# 523. Local renderer fast.

---

# 524. No domain transaction waits for push provider.

---

# 525. Hard rule.

---

# 526. Queue Priority

```text
Critical
High
Normal
Low
Background
```

---

# 527. Reserved capacity for Critical/Security.

---

# 528. Low-priority collapsible.

---

# 529. No reaction storm starving security notification.

---

# 530. Hard rule.

---

# 531. Push Token Fanout

Per user device.

---

# 532. Bounded.

---

# 533. Invalid token cleanup.

---

# 534. No permanent dead token buildup.

---

# 535. Rate Limits

Per source/class/device.

---

# 536. No sender can create unlimited interrupting notifications.

---

# 537. Hard rule.

---

# 538. Digest Performance

Generate from local/activity summary state.

---

# 539. No need to scan entire content history.

---

# 540. Incremental aggregation.

---

# 541. Activity Inbox Pagination

Virtualized UI.

---

# 542. No load-all.

---

# 543. Crate Layout

Recommended:

```text
crates/
├── siar-notification-core/
├── siar-notification-projection/
├── siar-notification-policy/
├── siar-notification-aggregation/
├── siar-activity-inbox/
├── siar-push-wake/
├── siar-notification-digest/
├── siar-notification-render/
├── siar-notification-observability/
└── siar-notification-testkit/
```

---

# 544. `siar-notification-core`

Owns:

```text
event IDs
classes
priorities
privacy classes
errors
```

---

# 545. `siar-notification-projection`

Domain event → minimal notification projection.

---

# 546. `siar-notification-policy`

Eligibility/preferences/quiet hours/attention budgets.

---

# 547. `siar-notification-aggregation`

Reaction/reply/feed grouping.

---

# 548. `siar-activity-inbox`

Durable local/account-encrypted activity state.

---

# 549. `siar-push-wake`

Provider-independent opaque wake dispatch.

---

# 550. `siar-notification-digest`

Local/batched digest generation.

---

# 551. `siar-notification-render`

Platform-neutral resolved notification + UI adapters.

---

# 552. `siar-notification-observability`

Aggregate technical metrics only.

---

# 553. `siar-notification-testkit`

dedup/aggregation/privacy/lifecycle/failure tests.

---

# 554. Error Taxonomy

```rust
pub enum NotificationError {
    EventInvalid,
    Suppressed,
    SourceUnavailable,
    AuthorizationDenied,
    PushTokenInvalid,
    PushProviderUnavailable,
    QuietHours,
    RateLimited,
    DigestUnavailable,
    ActivityDeleted,
    AccountInactive,
    Internal,
}
```

---

# 555. Testing

Need dedicated notification/attention testkit.

---

# 556. Test Scenarios

```text
private message wake
reaction storm
quiet hours
security alert
digest
```

---

# 557. Private Push Test

Push contains no plaintext.

---

# 558. Push Token Test

Token not usable as auth credential.

---

# 559. Reaction Aggregation Test

100 reactions produce bounded aggregate alerts.

---

# 560. Critical Priority Test

Security event not collapsed into low-priority digest.

---

# 561. Quiet Hours Test

Normal alert suppressed/digested.

---

# 562. Emergency Bypass Test

Only policy-approved critical class bypasses.

---

# 563. Block Test

Blocked contact creates no direct notification.

---

# 564. Mute Test

Muted thread becomes inbox/digest according to policy.

---

# 565. Mention Test

Current authorization rechecked before preview.

---

# 566. Deleted Content Test

Old notification preview removed/unavailable.

---

# 567. Deletion Epoch Test

Stale inbox sync cannot resurrect deleted activity.

---

# 568. Duplicate Delivery Test

Retry produces one visible activity.

---

# 569. Multi-Device Test

No server attention-state surveillance required.

---

# 570. Federation Test

Remote domain never receives local push token.

---

# 571. Managed Test

Org broadcast scoped to managed profile.

---

# 572. Account Deletion Test

Push tokens revoked/no new activity.

---

# 573. Digest Privacy Test

Local digest does not upload private contents.

---

# 574. Read-State Test

No sender sees notification-open state.

---

# 575. External Email Test

Private content requires authenticated fetch.

---

# 576. Attention Budget Test

Low-priority interruption downgraded, event retained.

---

# 577. Telemetry Test

No per-user open propensity or activity graph logged.

---

# 578. Fuzzing

Fuzz:

```text
notification projection
push hint
activity envelope
digest state
aggregation key
```

---

# 579. Property Tests

Properties:

```text
private event projection never produces plaintext push payload
critical notification cannot be downgraded by generic low-priority aggregation
deleted activity cannot return from stale sync
duplicate domain event does not create duplicate visible notification
```

---

# 580. Formal Verification Targets

Strong candidates:

```text
delivery/dedup state machine
aggregation priority rules
deletion anti-resurrection
quiet-hours/bypass precedence
```

---

# 581. Kani Candidate

priority/eligibility/precedence invariants.

---

# 582. TLA+ Candidate

domain commit → notification outbox → retry → deletion race.

---

# 583. Loom Candidate

concurrent duplicate delivery + aggregation + read-state update.

---

# 584. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Push providers receive opaque wake metadata only for private/E2EE events; private plaintext is fetched and decrypted after wake.
2. A notification is never authorization proof; content access is revalidated when the notification is resolved or opened.
3. Notification priority is derived from domain/security semantics and explicit user/managed policy, never from engagement propensity or behavioral scoring.
4. Reaction, feed, and low-value social events are aggressively aggregatable, while critical security/emergency notifications cannot be silently downgraded into low-priority digests.
5. Activity inbox state, seen/read status, quiet hours, and device presentation preferences remain local by default and are synchronized only through encrypted opt-in mechanisms.
6. Managed notification policy affects managed context only and cannot inspect or override unrelated personal notification state.
7. Blocked/revoked/deleted sources cannot continue producing actionable notifications, and stale sync/backups cannot resurrect deleted activity state.
8. Notification infrastructure never receives or exports a global who-notified-whom, who-opened-what, or per-user engagement graph.
9. Notification delivery/display state remains distinct from message delivery, content read state, and user engagement.
10. Push tokens are device-scoped routing metadata, never authentication credentials or social identities, and are revoked on device/account lifecycle changes.
11. Notification workers, digests, and delivery queues are bounded, idempotent, priority-aware, and cannot allow low-value storms to starve security/emergency events.
12. Notification systems optimize for timely, user-controlled attention—not return frequency, session length, open rate, or compulsive engagement.
```

---

# 585. Initial Production Scope

Implement first:

```text
typed NotificationEvent/ActivityItem
semantic notification classes/priorities
privacy classification
domain-event projection layer
eligibility/preference resolver
local activity inbox
reaction/reply aggregation
quiet hours
attention budget
opaque push wake hints
push token lifecycle
device-local lock-screen preview policy
notification outbox/dedup/retry
local digest generation
managed notification isolation
multi-device encrypted inbox sync optional
deletion/revocation anti-resurrection
privacy-safe operational metrics
notification testkit
```

Then add:

```text
on-device AI digest
advanced aggregate interaction summaries
cross-device encrypted attention-state sync
privacy-preserving multi-provider push routing
formal priority/dedup verification
enterprise digest/report adapters
```

---

# 586. Definition of Done

Part 90 is complete when:

- domain events project into minimal notification events
- private push payloads are opaque wake hints
- activity inbox is local-first
- seen/read state is private by default
- reactions/social events aggregate safely
- critical security/emergency alerts retain priority
- quiet hours and attention budgets work
- sender cannot bypass user notification policy
- managed notifications stay managed-scoped
- federation never receives local push tokens
- notification open/read behavior is not exposed to senders
- deletion/revocation removes actionable stale notifications
- queues/retries/dedup are bounded and idempotent
- digests can be generated locally without cloud private-content analysis
- telemetry excludes engagement-surveillance graphs
- push/aggregation/quiet-hours/digest/fuzz/formal tests are specified

---

# 587. Final Architecture

```text
                   DOMAIN EVENT
                        │
                        ▼
              NOTIFICATION PROJECTION
                        │
                        ▼
                ELIGIBILITY / POLICY
                        │
            ┌───────────┼───────────┐
            │           │           │
        INTERRUPT    INBOX ONLY    DIGEST
            │           │           │
            ▼           ▼           ▼
       PUSH WAKE    ACTIVITY LOG   SUMMARY
            │           │           │
            └───────────┼───────────┘
                        ▼
                LOCAL RESOLUTION
                        │
                        ▼
                  USER ATTENTION
```

Attention safety model:

```text
minimal projection
+
local-first inbox
+
opaque push wake
+
priority semantics
+
aggregation
+
quiet hours
+
attention budgets
+
privacy-safe digests
+
no engagement profiling
```

not:

```text
track every notification open, learn which alerts pull the user back fastest, and optimize pushes for maximum time-on-app
```

---

# 588. Final Principle

A notification should interrupt the user only when the event deserves attention according to the user's intent and security needs—not because a ranking system predicts it will increase engagement.

The correct model is:

```text
project minimally
+
aggregate aggressively
+
interrupt selectively
+
wake opaquely
+
resolve locally
+
digest privately
+
respect quiet time
+
never profile attention
```

This architecture gives SIAR a privacy-preserving foundation for push wake, activity inboxes, mentions, replies, reactions, security alerts, digests, managed notifications, and attention control while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–89.
