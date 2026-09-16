# Core System Architecture Part 89 — Anonymous Network Comments, Replies, Threads, Reactions, Mentions, Resharing & Privacy-Preserving Interaction Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 89  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 30–32, 43, 45–46, 56, 58–59, 67–69, 74–88  

**Primary purpose:** define SIAR's interaction architecture for comments, replies, threaded discussion, reactions, mentions, reshares, quotes, interaction notifications, privacy boundaries, anti-enumeration, moderation, federation, offline-first interaction queues, deletion/retraction, and social-graph minimization without turning user engagement into a centralized interaction graph.

---

# 1. Purpose

Interactions create the social layer around published content.

They include:

```text
comments
replies
threads
reactions
mentions
quotes
reshares
```

But interaction systems can easily become privacy-invasive because they reveal:

```text
who responds to whom
who reacted to what
who mentioned whom
which users repeatedly interact
how communities overlap
```

The governing principle is:

> **SIAR interactions must support conversation and discovery while keeping identity scope narrow, interaction visibility explicit, and engagement data from becoming a platform-owned global relationship graph.**

---

# 2. Architectural Position

```text
Published Content
      │
      ▼
Interaction Capability
      │
      ▼
Comment / Reply / Reaction / Mention / Reshare
      │
      ▼
Scoped Interaction State
      │
      ├── Notifications
      ├── Moderation
      ├── Feed Projection
      └── Federation
```

---

# 3. Core Separation

Keep distinct:

```text
content identity
interaction identity
interaction actor identity
thread identity
reaction identity
mention reference
reshare identity
moderation state
```

---

# 4. Non-Goals

Part 89 does not create:

```text
a global interaction graph
a public list of every person who viewed/reacted
a universal user mention namespace
a permanent reshare genealogy
a centralized engagement score per person
```

---

# 5. Interaction Types

```rust
pub enum InteractionType {
    Comment,
    Reply,
    Reaction,
    Mention,
    Reshare,
    Quote,
}
```

---

# 6. Comment

Top-level response to content.

---

# 7. Reply

Response to a comment/reply.

---

# 8. Reaction

Lightweight semantic interaction.

---

# 9. Mention

Explicit reference to another participant/publisher.

---

# 10. Reshare

Redistribution of existing content.

---

# 11. Quote

Reshare with additional author text.

---

# 12. Interaction ID

```rust
pub struct InteractionId(pub [u8; 32]);
```

---

# 13. Thread ID

```rust
pub struct ThreadId(pub [u8; 32]);
```

---

# 14. Reaction ID

```rust
pub struct ReactionId(pub [u8; 32]);
```

---

# 15. Reshare ID

```rust
pub struct ReshareId(pub [u8; 32]);
```

---

# 16. Hard Rule

Interaction IDs are opaque and never encode account/login identity.

---

# 17. Interaction Actor Identity

```rust
pub enum InteractionActor {
    RelationshipScoped(RelationshipId),
    MembershipScoped(MembershipPseudonym),
    PublisherScoped(PublisherPseudonym),
    Managed(ManagedPublisherId),
}
```

---

# 18. No Global AccountId Requirement

Hard rule.

---

# 19. Interaction Scope

```rust
pub enum InteractionScope {
    DirectRelationship,
    SocialSpace(SocialSpaceId),
    Channel(ChannelId),
    PublicContent(ContentId),
    ManagedWorkspace(OrganizationId),
}
```

---

# 20. Scope Binding

Every interaction belongs to one scope.

---

# 21. Hard Rule

Interaction identity is not portable across unrelated scopes by default.

---

# 22. Comment Record

```rust
pub struct CommentRecord {
    pub interaction_id: InteractionId,
    pub content: ContentId,
    pub actor: InteractionActor,
    pub body: CommentBody,
    pub visibility: InteractionVisibility,
    pub state: InteractionState,
}
```

---

# 23. Comment Body

Bounded text.

---

# 24. Optional media

Separate attachment references.

---

# 25. No Active HTML

Hard rule.

---

# 26. Interaction Visibility

```rust
pub enum InteractionVisibility {
    RelationshipOnly,
    MembersOnly,
    Subscribers,
    Public,
    Managed,
}
```

---

# 27. Visibility Cannot Exceed Parent Content Scope

Hard rule.

---

# 28. Example

Private group post reply cannot become public.

---

# 29. Interaction State

```rust
pub enum InteractionState {
    Active,
    Edited,
    Retracted,
    Deleted,
    Moderated,
}
```

---

# 30. Threading

Thread is explicit.

---

# 31. Thread Root

Top-level content or comment.

---

# 32. Parent Reference

```rust
pub enum ReplyParent {
    Content(ContentId),
    Interaction(InteractionId),
}
```

---

# 33. Reply Record

```rust
pub struct ReplyRecord {
    pub interaction_id: InteractionId,
    pub thread: ThreadId,
    pub parent: ReplyParent,
    pub actor: InteractionActor,
    pub body: CommentBody,
}
```

---

# 34. Thread Depth

Bounded.

---

# 35. Why

Prevents pathological nesting and expensive traversal.

---

# 36. Maximum Depth

Policy-defined.

---

# 37. Example

```text
8–16 levels
```

---

# 38. Hard Rule

Unbounded recursive thread trees are forbidden.

---

# 39. Thread Ordering

Options:

```rust
pub enum ThreadOrdering {
    Chronological,
    LocalRanked,
    ModeratorPinned,
}
```

---

# 40. Default

Chronological.

---

# 41. LocalRanked

On-device only.

---

# 42. ModeratorPinned

Scoped.

---

# 43. No Hidden Server Behavioral Ranking By Default

Hard rule.

---

# 44. Comment Capability

```rust
pub struct InteractionCapability {
    pub scope: InteractionScope,
    pub actions: BTreeSet<InteractionAction>,
    pub expires_at: Option<Timestamp>,
}
```

---

# 45. Interaction Actions

```rust
pub enum InteractionAction {
    Comment,
    Reply,
    React,
    Mention,
    Reshare,
    Quote,
    DeleteOwn,
    EditOwn,
}
```

---

# 46. No Interaction Without Capability

Hard rule.

---

# 47. Parent Content Policy

Can disable specific interaction classes.

---

# 48. Example

```text
comments disabled
reactions allowed
reshares denied
```

---

# 49. Content Interaction Policy

```rust
pub struct ContentInteractionPolicy {
    pub allow_comments: bool,
    pub allow_replies: bool,
    pub allow_reactions: bool,
    pub allow_mentions: bool,
    pub allow_reshares: bool,
}
```

---

# 50. Creator Preference

Cannot override social-space security floor.

---

# 51. Moderators May Restrict

Yes.

---

# 52. Cannot silently expand beyond content visibility.

---

# 53. Hard rule.

---

# 54. Comment Editing

Creates revision.

---

# 55. Interaction Revision ID

```rust
pub struct InteractionRevisionId(pub [u8; 32]);
```

---

# 56. Interaction revisions immutable.

---

# 57. Current head pointer.

---

# 58. No destructive in-place overwrite.

---

# 59. Hard rule.

---

# 60. Edit History Visibility

```rust
pub enum InteractionRevisionPolicy {
    CurrentOnly,
    PublicHistory,
    ModeratorVisible,
}
```

---

# 61. Default

CurrentOnly.

---

# 62. Public History

Explicit.

---

# 63. Moderator Visible

Social-space scoped.

---

# 64. No Automatic Full Public Edit History

Hard rule.

---

# 65. Comment Retraction

Author can retract.

---

# 66. Retraction removes active body from display where policy allows.

---

# 67. Thread continuity marker may remain.

---

# 68. Hard Rule

Retraction ≠ guaranteed global erasure.

---

# 69. Comment Deletion

Stronger.

---

# 70. Depends on ownership/retention.

---

# 71. Tombstone may remain.

---

# 72. No stale cache resurrection.

---

# 73. Interaction Deletion Epoch

```rust
pub struct InteractionDeletionEpoch(pub u64);
```

---

# 74. Monotonic.

---

# 75. Old replicas rejected.

---

# 76. Hard rule.

---

# 77. Reactions

Lightweight interaction.

---

# 78. Reaction Types

```rust
pub enum ReactionKind {
    Like,
    Appreciate,
    Agree,
    Disagree,
    Laugh,
    Celebrate,
    Custom(ReactionCode),
}
```

---

# 79. Custom Reaction

Bounded/allowlisted per space.

---

# 80. No Arbitrary Executable Reaction Payload

Hard rule.

---

# 81. Reaction Visibility

```rust
pub enum ReactionVisibility {
    AggregateOnly,
    ParticipantsOnly,
    PublicOptIn,
}
```

---

# 82. Default Public Content

AggregateOnly.

---

# 83. Default Private Group

ParticipantsOnly or aggregate.

---

# 84. PublicOptIn

Reacting user explicitly chooses identity exposure.

---

# 85. Hard Rule

A reaction never exposes actor identity publicly by default.

---

# 86. Reaction Aggregate

```rust
pub struct ReactionAggregate {
    pub kind: ReactionKind,
    pub count_class: CountClass,
}
```

---

# 87. Count Class

Could be exact for small private group if policy allows.

---

# 88. Public Content

Prefer thresholded/coarse counts.

---

# 89. No Small-Cohort Deanonymization

Hard rule.

---

# 90. Reaction Toggle

Idempotent.

---

# 91. Same actor + content + kind

At most one active reaction.

---

# 92. No duplicate counting on retry.

---

# 93. Hard rule.

---

# 94. Reaction Storage

Actor-scoped/private relation.

---

# 95. Public aggregate separated from actor mapping.

---

# 96. Hard rule.

---

# 97. Reaction Actor Mapping

Retained only as needed for undo/abuse controls.

---

# 98. Not exposed publicly.

---

# 99. No global "liked content" profile.

---

# 100. Hard rule.

---

# 101. Mentions

Explicit references.

---

# 102. Mention Targets

```rust
pub enum MentionTarget {
    Relationship(RelationshipId),
    Membership(MembershipPseudonym),
    Publisher(PublisherPseudonym),
    ManagedMember(OrganizationMembershipId),
}
```

---

# 103. No Account Email/Phone Mention

Hard rule.

---

# 104. Mention Scope Validation

Mention target must be valid within interaction scope.

---

# 105. Example

Group comment cannot mention arbitrary hidden member unless roster/policy permits.

---

# 106. Hard Rule

Mention must not become membership enumeration oracle.

---

# 107. Mention Resolution

Local/scoped.

---

# 108. Failure Response

Generic.

---

# 109. No "this user exists but is not in group" leak.

---

# 110. Hard rule.

---

# 111. Mention Notification

Opaque wake hint.

---

# 112. Notification fetches authorized content after wake.

---

# 113. No full private content in push payload.

---

# 114. Hard rule.

---

# 115. Mention Abuse

Rate limit.

---

# 116. User can disable mentions.

---

# 117. Space policy can restrict `@everyone`-like feature.

---

# 118. Broadcast Mention

Separate high-risk capability.

---

# 119. No ordinary member automatic broadcast mention.

---

# 120. Hard rule.

---

# 121. Mention All Capability

```rust
pub struct BroadcastMentionCapability {
    pub scope: InteractionScope,
    pub max_frequency: RateLimitPolicy,
}
```

---

# 122. Resharing

Redistributes existing content.

---

# 123. Reshare Visibility

Cannot exceed original content rights.

---

# 124. Public content

Can be reshared if source policy allows.

---

# 125. Private content

Requires explicit reshare capability.

---

# 126. Hard Rule

Private/member-scoped content cannot be reshared publicly without explicit authority.

---

# 127. Reshare Record

```rust
pub struct ReshareRecord {
    pub reshare_id: ReshareId,
    pub source_content: ContentId,
    pub actor: InteractionActor,
    pub target: PublicationTarget,
    pub attribution: AttributionPolicy,
}
```

---

# 128. Attribution Policy

```rust
pub enum AttributionPolicy {
    Required,
    Optional,
    HiddenBySource,
}
```

---

# 129. Public Content

Attribution recommended/required.

---

# 130. Private Anonymous Publisher

May hide source identity per source policy.

---

# 131. No account-level deanonymization.

---

# 132. Hard rule.

---

# 133. Reshare Chain

Potential graph leak.

---

# 134. Default

Only immediate source attribution.

---

# 135. No full chain genealogy.

---

# 136. Hard rule.

---

# 137. Quote

Reshare + new text.

---

# 138. Quote Body

New creator revision.

---

# 139. Original content reference immutable.

---

# 140. If source retracted/deleted

Quote behavior policy.

---

# 141. Options:

```rust
pub enum SourceRemovalPolicy {
    HideOriginalPreview,
    KeepCitationOnly,
    RemoveReshare,
}
```

---

# 142. No silent use of deleted private content.

---

# 143. Hard rule.

---

# 144. Thread Identity Privacy

Thread ID local/scoped.

---

# 145. No cross-space global thread ID.

---

# 146. Hard rule.

---

# 147. Comment Actor Pseudonym

Space-scoped.

---

# 148. Public content

Publisher pseudonym or anonymous capability.

---

# 149. No forced global real-name identity.

---

# 150. Hard rule.

---

# 151. Anonymous Comments

Possible if source policy allows.

---

# 152. Abuse implications.

---

# 153. Need scoped anti-spam capability.

---

# 154. Anonymous Comment Token

```rust
pub struct AnonymousInteractionToken {
    pub scope: InteractionScope,
    pub actions: BTreeSet<InteractionAction>,
    pub expires_at: Timestamp,
}
```

---

# 155. No global identity.

---

# 156. Rate limited.

---

# 157. No custom anonymous-credential crypto initially.

---

# 158. Hard rule.

---

# 159. Interaction Request State

For moderated spaces:

```rust
pub enum InteractionSubmissionState {
    Pending,
    Approved,
    Rejected,
    Published,
}
```

---

# 160. Pre-moderation Optional

---

# 161. Approval Scope

Exact revision digest.

---

# 162. Edit after approval invalidates approval.

---

# 163. Hard rule.

---

# 164. Moderation

Part 86 integration.

---

# 165. Moderator Actions

```text
hide comment
remove comment
lock thread
disable reactions
restrict user within space
```

---

# 166. Moderator cannot edit author's signed body.

---

# 167. Hard rule.

---

# 168. Thread Lock

```rust
pub enum ThreadState {
    Open,
    Locked,
    Archived,
    Deleted,
}
```

---

# 169. Locked

Read-only.

---

# 170. Existing content remains.

---

# 171. Archived

No new interaction, retained.

---

# 172. Deleted

Lifecycle removal.

---

# 173. No Hidden Lock State

UI should show.

---

# 174. Interaction Moderation State

Separate from content state.

---

# 175. Hard rule.

---

# 176. Moderation Audit

Scoped.

---

# 177. No global dossier.

---

# 178. Moderation Event

Minimal.

---

# 179. No unrelated interaction history.

---

# 180. Interaction Notifications

Types:

```rust
pub enum InteractionNotificationType {
    Reply,
    Mention,
    Reaction,
    Reshare,
    ModeratorAction,
}
```

---

# 181. Notification Eligibility

Depends on user preference and capability.

---

# 182. Reactions

May be batched.

---

# 183. No one push per reaction storm.

---

# 184. Hard rule.

---

# 185. Notification Fanout

Asynchronous.

---

# 186. Outbox.

---

# 187. No notification send in interaction commit transaction.

---

# 188. Hard rule.

---

# 189. Notification Payload

Opaque hint where private.

---

# 190. Public interaction may include safe preview.

---

# 191. Lock-screen privacy setting honored.

---

# 192. No hidden contact identity leak.

---

# 193. Hard rule.

---

# 194. Offline Interaction

Local queue.

---

# 195. User can write reply offline.

---

# 196. Queue for publish.

---

# 197. On reconnect:

```text
reauthorize
revalidate parent
check thread state
check deletion epoch
publish
```

---

# 198. Hard Rule

Offline queued interaction never bypasses current moderation/policy state.

---

# 199. Parent Deleted While Offline

Interaction fails gracefully.

---

# 200. Thread Locked While Offline

Reply denied at reconnect.

---

# 201. UI shows actionable reason.

---

# 202. Interaction Outbox

```rust
pub struct PendingInteraction {
    pub operation_id: InteractionOperationId,
    pub parent: ReplyParent,
    pub action: InteractionAction,
    pub payload: EncryptedInteractionPayload,
}
```

---

# 203. Idempotent.

---

# 204. No duplicate comment on retry.

---

# 205. Hard rule.

---

# 206. Interaction Operation ID

Random.

---

# 207. Not account-derived.

---

# 208. Feed Integration

Comments/reactions may produce feed updates.

---

# 209. But

Not every interaction becomes feed item.

---

# 210. Hard rule.

---

# 211. Example

Comment count update aggregate.

---

# 212. Reshare creates new feed item.

---

# 213. Reaction does not create feed event by default.

---

# 214. No reaction-spam feed.

---

# 215. Hard rule.

---

# 216. Ranking Integration

Interaction counts may be used only if feed policy allows.

---

# 217. No per-user engagement graph.

---

# 218. Public aggregate reaction/comment count can influence trend.

---

# 219. Thresholded.

---

# 220. No hidden personal interaction vector.

---

# 221. Hard rule.

---

# 222. Comment Count

Public content may show coarse/exact count.

---

# 223. Private group

local/authorized count.

---

# 224. Count ≠ member identity list.

---

# 225. Hard rule.

---

# 226. Thread Pagination

Cursor based.

---

# 227. Bounded page size.

---

# 228. No recursive full-tree fetch.

---

# 229. Hard rule.

---

# 230. Cursor Scope

Thread-specific.

---

# 231. No user identity in cursor.

---

# 232. Cursor expiry.

---

# 233. Thread Fetch

Authorization before response.

---

# 234. Ranking after eligibility.

---

# 235. Hard rule.

---

# 236. Reply Visibility

Never exceeds parent.

---

# 237. Hard rule.

---

# 238. Reaction Visibility

Never exposes more identity than source interaction policy.

---

# 239. Hard rule.

---

# 240. Mention Visibility

Mention target may be private.

---

# 241. Render display projection only.

---

# 242. No raw internal ID.

---

# 243. Hard rule.

---

# 244. Reshare Privacy

Resharing private content can leak existence.

---

# 245. Therefore require explicit reshare permission.

---

# 246. Hard rule.

---

# 247. Interaction Discovery

Public comments may be searchable if source policy allows.

---

# 248. Private comments local/member only.

---

# 249. No central private comment index.

---

# 250. Hard rule.

---

# 251. Public Comment Search

Optional.

---

# 252. Actor pseudonym only.

---

# 253. No account mapping.

---

# 254. Search Index

Updated after commit.

---

# 255. Retraction/delete remove/tombstone.

---

# 256. No stale search resurrection.

---

# 257. Hard rule.

---

# 258. Interaction Federation

Cross-domain public/federated comments.

---

# 259. Remote actor identity scoped to remote domain.

---

# 260. No AccountId merge.

---

# 261. Hard rule.

---

# 262. Federated Reply

Carries:

```text
remote interaction ref
source domain
signed/pseudonymous actor
scope capability
```

---

# 263. Federation Gateway

Validates domain/capability.

---

# 264. No raw internal IDs.

---

# 265. Hard rule.

---

# 266. Remote Reaction

Aggregate may be forwarded.

---

# 267. No cross-domain actor list.

---

# 268. Hard rule.

---

# 269. Federated Mention

Only if target addressable in federation scope.

---

# 270. No remote membership enumeration.

---

# 271. Hard rule.

---

# 272. Federated Reshare

Public content only by default.

---

# 273. Private federation content requires explicit capability.

---

# 274. Hard rule.

---

# 275. Cross-Domain Thread

Thread ownership remains source-domain.

---

# 276. Remote replies append through gateway.

---

# 277. No shared global interaction DB.

---

# 278. Hard rule.

---

# 279. Federation Outage

Local reply queue can persist.

---

# 280. On reconnect, revalidate source/thread.

---

# 281. No stale policy bypass.

---

# 282. Hard rule.

---

# 283. Managed Workspace Interactions

Org policy.

---

# 284. Managed identity.

---

# 285. Moderation authority scoped.

---

# 286. Personal interaction history not visible to org by default.

---

# 287. Hard rule.

---

# 288. Offboarding

Managed interactions remain according to ownership policy.

---

# 289. User loses future edit capability if org-owned.

---

# 290. Personal content unaffected.

---

# 291. Content Ownership

Inherited from parent context.

---

# 292. No cross-boundary migration automatically.

---

# 293. Hard rule.

---

# 294. Contact-Scoped Comments

Direct shared content.

---

# 295. Relationship identity.

---

# 296. No public projection.

---

# 297. Hard rule.

---

# 298. Group Interactions

Membership pseudonym.

---

# 299. If member removed

future interactions denied.

---

# 300. Existing interactions retained per policy.

---

# 301. No automatic personal contact edge.

---

# 302. Hard rule.

---

# 303. Blocking Integration

Part 85.

---

# 304. If actor blocked:

```text
hide interaction locally
deny notifications
deny new direct interaction
```

---

# 305. Public content comments

May still exist globally.

---

# 306. User can hide.

---

# 307. No global delete due local block.

---

# 308. Hard rule.

---

# 309. Muting

Local presentation only.

---

# 310. Not moderation.

---

# 311. Not block.

---

# 312. Hard rule.

---

# 313. Abuse Resistance

Threats:

```text
reply spam
reaction spam
mention storms
reshare amplification
brigading
```

---

# 314. Reply Spam

Rate limits.

---

# 315. Reaction Spam

Idempotent + per-content limits.

---

# 316. Mention Storm

Mention quotas.

---

# 317. Reshare Amplification

Source/target policy + rate limits.

---

# 318. Brigading

Join/interaction admission controls.

---

# 319. No invasive real-world identity requirement.

---

# 320. Hard rule.

---

# 321. Slow Mode

Can apply to comments/replies.

---

# 322. Per-space/channel.

---

# 323. No user behavioral profiling required.

---

# 324. Interaction Quota

```rust
pub struct InteractionQuota {
    pub max_comments: u32,
    pub max_replies: u32,
    pub max_mentions: u32,
    pub window: Duration,
}
```

---

# 325. Per-scope.

---

# 326. No global social score.

---

# 327. Hard rule.

---

# 328. Mention Rate

Lower.

---

# 329. Broadcast mention much lower.

---

# 330. Reshare Quota

Separate.

---

# 331. Comment Media

Bounded.

---

# 332. Attachment size lower than main publication by default.

---

# 333. No executable payload.

---

# 334. Media metadata stripped.

---

# 335. Hard rule.

---

# 336. Link Safety

No auto-preview if privacy mode forbids.

---

# 337. No tracking pixel rendering.

---

# 338. Hard rule.

---

# 339. Reaction Semantics

Custom reactions should be semantic code, not arbitrary image URL by default.

---

# 340. Avoid external fetch.

---

# 341. Hard rule.

---

# 342. Thread Archival

Can collapse inactive threads.

---

# 343. Archive state local/server metadata.

---

# 344. No delete.

---

# 345. User Can Hide Thread Locally

Yes.

---

# 346. Local hidden state not shared.

---

# 347. Hard rule.

---

# 348. Thread Subscription

User can follow thread.

---

# 349. Thread subscription private.

---

# 350. Author/moderator does not see follower list by default.

---

# 351. Hard rule.

---

# 352. Thread Subscription State

```rust
pub enum ThreadSubscriptionState {
    Following,
    Muted,
    Unfollowed,
}
```

---

# 353. Notifications

Use local subscription.

---

# 354. No public follower count required.

---

# 355. Interaction Analytics

Creator/moderator aggregate only.

---

# 356. Allowed:

```text
comment count
reaction count
reshare count
```

---

# 357. Not allowed by default:

```text
reactor identities
reader identities
commenter contact graph
precise timestamps per person
```

---

# 358. Hard rule.

---

# 359. Small Cohort Suppression

Public analytics threshold.

---

# 360. No deanonymizing slices.

---

# 361. Hard rule.

---

# 362. Interaction Telemetry

Operational only.

---

# 363. Safe:

```text
publish latency
notification lag
federation sync failure
```

---

# 364. Forbidden:

```text
user interaction timeline
who replied to whom globally
mention graph
reaction graph
```

---

# 365. Hard rule.

---

# 366. Moderation Metrics

Aggregate per space.

---

# 367. No cross-space offender dossier by default.

---

# 368. Hard rule.

---

# 369. Interaction Storage

Local/private:

```text
comment cache
thread state
local reaction state
block/mute state
```

Server/social-space:

```text
authorized interaction records
aggregate counts
moderation state
```

---

# 370. No centralized universal edge table:

```text
actor_id -> content_id -> reaction/comment
```

---

# 371. Hard rule.

---

# 372. Partitioning

By content/thread/social-space.

---

# 373. Not by global user ID.

---

# 374. Hard rule.

---

# 375. Thread Store

Bounded pagination.

---

# 376. Comment body encrypted for private spaces.

---

# 377. Public content plaintext if intentionally public.

---

# 378. Private reaction actor mapping encrypted/scoped.

---

# 379. No cross-space index.

---

# 380. Hard rule.

---

# 381. Transaction Boundary

Comment publish:

```text
validate parent/scope
→ persist interaction
→ update aggregate counters
→ write outbox
→ commit
```

---

# 382. No notification/federation RPC in transaction.

---

# 383. Hard rule.

---

# 384. Reaction Toggle Transaction

```text
check unique actor/content/kind
→ insert/delete reaction
→ update aggregate
→ outbox
→ commit
```

---

# 385. Idempotent.

---

# 386. No count drift.

---

# 387. Hard rule.

---

# 388. Aggregate Counter Repair

Reconciliation.

---

# 389. Counter is derived.

---

# 390. Actor mapping authoritative if retained.

---

# 391. Public approximate counter may be eventually consistent.

---

# 392. Hard rule.

---

# 393. Reshare Transaction

New publication + source reference + outbox.

---

# 394. Attribution validated.

---

# 395. Source visibility checked at commit.

---

# 396. No stale visibility bypass.

---

# 397. Hard rule.

---

# 398. Interaction Event Types

```rust
pub enum InteractionEvent {
    CommentPublished,
    ReplyPublished,
    ReactionChanged,
    MentionCreated,
    ContentReshared,
    InteractionRetracted,
    InteractionDeleted,
    ThreadLocked,
}
```

---

# 399. Event Payload

Minimal.

---

# 400. No account login IDs.

---

# 401. No raw private body on global bus.

---

# 402. Hard rule.

---

# 403. Notification Projection

Separate.

---

# 404. Feed Projection

Separate.

---

# 405. Federation Projection

Separate.

---

# 406. No one giant event payload.

---

# 407. Hard rule.

---

# 408. Event Bus Authorization

Producer/consumer scoped.

---

# 409. No analytics consumer access to private interaction events.

---

# 410. Hard rule.

---

# 411. Search Integration

Public comments only if policy says.

---

# 412. Private search local/member service only.

---

# 413. Mention search not global.

---

# 414. Hard rule.

---

# 415. Account Lifecycle

Deleted account cannot create new interactions.

---

# 416. Existing interactions follow ownership/retention.

---

# 417. No stale scheduled/offline reply after deletion.

---

# 418. Hard rule.

---

# 419. Publisher/Account Suspension

Policy specific.

---

# 420. Security lock blocks high-risk actions.

---

# 421. Managed offboarding removes managed-space interaction capability.

---

# 422. Personal spaces unaffected.

---

# 423. Hard rule.

---

# 424. Profile Integration

Interaction display uses scoped profile projection.

---

# 425. No automatic full personal profile exposure.

---

# 426. Hard rule.

---

# 427. Feed Integration

Public reshares can enter feed.

---

# 428. Comments generally remain under parent.

---

# 429. Reaction aggregates can affect trend if policy allows.

---

# 430. No per-user engagement ranking.

---

# 431. Hard rule.

---

# 432. Publishing Integration

Part 88.

---

# 433. Comments/replies are interaction records, not full publication records unless quote/reshare.

---

# 434. Reshare/quote creates publication object.

---

# 435. Hard rule.

---

# 436. Social Space Integration

Part 86.

---

# 437. Moderation, membership, bans, roster privacy.

---

# 438. Contact Integration

Part 85.

---

# 439. Direct interaction relationship-scoped.

---

# 440. Federation Integration

Part 58.

---

# 441. Cross-domain scope explicit.

---

# 442. Authorization Integration

Part 81.

---

# 443. Every interaction action capability-checked.

---

# 444. Authentication Integration

Part 82.

---

# 445. Actor session establishes subject only.

---

# 446. No auth session token copied into interaction object.

---

# 447. Hard rule.

---

# 448. Edge Integration

Part 78.

---

# 449. L7 validation for public interaction APIs.

---

# 450. Private bodies remain encrypted.

---

# 451. WAF cannot inspect E2EE private comments.

---

# 452. Hard rule.

---

# 453. East-West Integration

Part 79.

---

# 454. Interaction, notification, federation services mTLS.

---

# 455. Secrets Integration

Part 80.

---

# 456. Signing keys scoped.

---

# 457. No shared global interaction signing secret.

---

# 458. Database Integration

Part 74.

---

# 459. Comment/reaction/thread stores domain-separated.

---

# 460. No global interaction warehouse.

---

# 461. Hard rule.

---

# 462. Backup

Own local interaction state backed up logically.

---

# 463. Live capabilities/session tokens excluded.

---

# 464. Retracted/deleted state preserved.

---

# 465. No anti-resurrection bypass.

---

# 466. Hard rule.

---

# 467. Export

User can export own comments/replies/reactions where permitted.

---

# 468. Reaction export does not include other users.

---

# 469. Thread export respects visibility/ownership.

---

# 470. Hard rule.

---

# 471. Data Lifecycle

Interaction retention by scope.

---

# 472. Private thread

space policy.

---

# 473. Public comments

public retention.

---

# 474. Moderation evidence

bounded.

---

# 475. No indefinite deleted-body shadow archive.

---

# 476. Hard rule.

---

# 477. Schema Versioning

Separate:

```text
interaction protocol
thread storage
reaction schema
mention schema
federation interaction envelope
```

---

# 478. No conflation.

---

# 479. Unknown reaction kind

safe fallback.

---

# 480. Unknown moderation state

restrictive.

---

# 481. Hard rule.

---

# 482. Serialization

Postcard internal/protocol.

---

# 483. RON config/export.

---

# 484. JSON external API if needed.

---

# 485. Strict bounds.

---

# 486. Thread depth bounded.

---

# 487. Mention count bounded.

---

# 488. Reaction set bounded.

---

# 489. Reshare metadata bounded.

---

# 490. Hard rule.

---

# 491. Interaction Service Interfaces

```rust
pub trait InteractionService {
    fn comment(
        &self,
        capability: InteractionCapability,
        parent: ReplyParent,
        body: CommentBody,
    ) -> Result<InteractionId, InteractionError>;

    fn reply(
        &self,
        capability: InteractionCapability,
        parent: InteractionId,
        body: CommentBody,
    ) -> Result<InteractionId, InteractionError>;
}
```

---

# 492. Reaction Service

```rust
pub trait ReactionService {
    fn set_reaction(
        &self,
        capability: InteractionCapability,
        target: InteractionTarget,
        reaction: ReactionKind,
    ) -> Result<(), InteractionError>;
}
```

---

# 493. Mention Service

```rust
pub trait MentionService {
    fn validate_mentions(
        &self,
        scope: InteractionScope,
        mentions: &[MentionTarget],
    ) -> Result<(), InteractionError>;
}
```

---

# 494. Reshare Service

```rust
pub trait ReshareService {
    fn reshare(
        &self,
        capability: InteractionCapability,
        source: ContentId,
        target: PublicationTarget,
    ) -> Result<ReshareId, InteractionError>;
}
```

---

# 495. Thread Repository

```rust
pub trait ThreadRepository {
    fn page(
        &self,
        thread: ThreadId,
        cursor: Option<ThreadCursor>,
        limit: usize,
    ) -> Result<ThreadPage, InteractionError>;
}
```

---

# 496. No Remote User Graph Query API

Hard rule.

---

# 497. Error Taxonomy

```rust
pub enum InteractionError {
    ParentNotFound,
    ParentDeleted,
    ThreadLocked,
    InteractionDenied,
    MentionDenied,
    ReshareDenied,
    VisibilityViolation,
    ReactionDenied,
    RateLimited,
    ActorBlocked,
    ActorBanned,
    RevisionConflict,
    InteractionDeleted,
    FederationUnavailable,
    Internal,
}
```

---

# 498. Testing

Need dedicated interaction testkit.

---

# 499. Test Scenarios

```text
comment
reply
reaction toggle
mention
reshare
offline reply
```

---

# 500. Visibility Test

Reply cannot exceed parent visibility.

---

# 501. Comment Capability Test

Unauthorized comment rejected.

---

# 502. Thread Lock Test

New replies denied.

---

# 503. Depth Test

Excess nesting rejected.

---

# 504. Edit Test

New revision created.

---

# 505. Public History Test

Only if policy allows.

---

# 506. Retraction Test

Active body removed appropriately.

---

# 507. Deletion Test

Stale cache cannot resurrect.

---

# 508. Reaction Privacy Test

Public viewer cannot enumerate reactors by default.

---

# 509. Reaction Retry Test

No duplicate count.

---

# 510. Mention Enumeration Test

Invalid target does not reveal membership existence.

---

# 511. Mention Storm Test

Rate limit enforced.

---

# 512. Broadcast Mention Test

Requires elevated capability.

---

# 513. Reshare Privacy Test

Private content cannot be reshared publicly without capability.

---

# 514. Reshare Chain Test

Only immediate source exposed.

---

# 515. Federation Test

No account ID merge.

---

# 516. Managed Scope Test

Org moderation does not affect personal spaces.

---

# 517. Block Test

Local blocked actor hidden/no notifications.

---

# 518. Offline Test

Queued reply revalidates parent/policy.

---

# 519. Account Deletion Test

Queued interaction cannot publish after deletion.

---

# 520. Feed Test

Reaction does not automatically create feed item.

---

# 521. Analytics Test

No global interaction graph emitted.

---

# 522. Search Test

Private interaction never enters public index.

---

# 523. Fuzzing

Fuzz:

```text
interaction envelope
thread cursor
reaction payload
mention list
federated interaction envelope
```

---

# 524. Property Tests

Properties:

```text
interaction visibility never exceeds parent scope
reaction retry never increments aggregate twice
deleted interaction never becomes active from stale replica
mention validation never leaks hidden membership existence
```

---

# 525. Formal Verification Targets

Strong candidates:

```text
thread state machine
reaction idempotency
reshare visibility constraints
offline queued interaction revalidation
```

---

# 526. Kani Candidate

visibility/action subset checks.

---

# 527. TLA+ Candidate

comment publish + delete + federation propagation race.

---

# 528. Loom Candidate

concurrent reaction toggle + aggregate update.

---

# 529. Performance

Thread hot path:

```text
bounded page fetch
indexed parent/thread
```

---

# 530. No recursive full tree load.

---

# 531. Reaction updates

small transaction.

---

# 532. Mention validation

bounded list.

---

# 533. Notification fanout

async.

---

# 534. Federation

async/retry.

---

# 535. No synchronous global interaction propagation.

---

# 536. Hard rule.

---

# 537. Thread Cache

Bounded.

---

# 538. Large threads

pagination + collapsed subthreads.

---

# 539. No memory proportional to full thread history.

---

# 540. Reaction aggregate

cached/derived.

---

# 541. No giant per-item reactor list in memory.

---

# 542. Reshare references

small.

---

# 543. Mention resolution

local/scoped index.

---

# 544. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Interaction actor identities are relationship-, membership-, publisher-, or managed-scope identities and never require global login AccountId exposure.
2. Comment/reply visibility can never exceed the visibility or authorization scope of the parent content/thread.
3. Reactions expose actor identities only when explicitly permitted; public defaults use aggregate or thresholded counts.
4. Mentions are scope-validated and cannot be used to enumerate hidden members, contacts, or accounts.
5. Private/member-scoped content cannot be reshared into a broader visibility scope without explicit reshare authority.
6. Reshare/quote attribution exposes at most the source identity allowed by source policy and does not reveal full reshare genealogy.
7. Interaction moderation, ranking, blocking, muting, and deletion states remain distinct and cannot silently substitute for one another.
8. Offline queued comments/replies/reactions are reauthorized and revalidated against current parent/thread/moderation state before commit.
9. Interaction notifications use scoped/opaque hints and do not expose private content or contact identities through push infrastructure.
10. Interaction telemetry, analytics, moderation logs, and operational events never form a remotely queryable global who-interacted-with-whom graph.
11. Deleted/retracted interactions and revoked capabilities cannot be resurrected by stale caches, backups, federation replicas, or delayed jobs.
12. Reaction counters, comment counts, and other aggregates are derived state; retries or concurrency cannot create semantic double-counting.
```

---

# 545. Initial Production Scope

Implement first:

```text
typed InteractionId/ThreadId/ReactionId/ReshareId
comment/reply state machine
bounded thread depth
parent-scope visibility enforcement
immutable interaction revisions
aggregate-only public reactions
idempotent reaction toggle
scoped mention model
broadcast-mention capability
reshare/quote with source policy
thread lock/archive state
offline interaction outbox
notification outbox
social-space moderation integration
federated public interactions
anti-resurrection deletion epochs
privacy-safe aggregate analytics
interaction testkit
```

Then add:

```text
advanced anonymous comments
privacy-preserving public interaction aggregates
cross-provider thread federation
local thread ranking
formal reaction/thread verification
advanced moderation appeal integration
```

---

# 546. Definition of Done

Part 89 is complete when:

- comments/replies are scope-bound and thread-safe
- reaction identities remain private by default
- reaction retries are idempotent
- mention resolution cannot enumerate hidden identities
- reshare visibility never widens without authority
- reshare genealogy is minimized
- thread lock/archive/delete states are distinct
- offline queued interactions revalidate current policy
- notifications remain privacy-safe
- federation preserves scoped identities
- moderation cannot rewrite author content
- search/public discovery boundaries are explicit
- deletion/retraction anti-resurrection works
- telemetry excludes global interaction graphs
- comment/reaction/mention/reshare/fuzz/formal tests are specified

---

# 547. Final Architecture

```text
                    PUBLISHED CONTENT
                           │
                           ▼
                  INTERACTION POLICY
                           │
            ┌──────────────┼──────────────┐
            │              │              │
         COMMENT         REACTION       RESHARE
            │              │              │
            ▼              ▼              ▼
          THREAD        AGGREGATE      NEW TARGET
            │              │              │
            └──────────────┼──────────────┘
                           ▼
                 NOTIFY / MODERATE / FEDERATE
```

Interaction safety model:

```text
scoped actor identity
+
parent-bound visibility
+
private reaction identity
+
mention anti-enumeration
+
reshare authority
+
offline revalidation
+
async notifications
+
anti-resurrection
```

not:

```text
record every user-content edge globally and turn comments, reactions, mentions, and reshares into a permanent behavioral graph
```

---

# 548. Final Principle

Interactions should enrich content and conversation without creating a durable platform-owned map of who engages with whom.

The correct model is:

```text
reply within scope
+
react privately
+
mention only where authorized
+
reshare without widening access
+
moderate locally
+
notify minimally
+
federate explicitly
+
aggregate without graphing people
```

This architecture gives SIAR a privacy-preserving interaction foundation for comments, replies, threads, reactions, mentions, quotes, reshares, notifications, social-space moderation, and federation while preserving the anonymity, local-first, least-authority, and social-graph privacy guarantees established across Parts 34–88.
