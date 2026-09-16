# Core System Architecture Part 88 — Anonymous Network Content Publishing, Posts, Articles, Media, Drafts, Revisions, Distribution & Creator-State Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 88  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 32, 40, 43, 56, 58–59, 67–69, 74–87  

**Primary purpose:** define SIAR's complete content-publishing architecture for posts, articles, media, drafts, revisions, scheduled publication, retraction, creator pseudonyms, local/offline authoring, multi-device synchronization, content signing, provenance, feed/channel/federation distribution, media processing, moderation, retention, analytics boundaries, and creator-state lifecycle.

---

# 1. Purpose

Publishing content appears simple:

```text
write
→ publish
```

In a real privacy-preserving distributed system, publishing includes:

```text
drafts
autosave
multi-device sync
media
revision history
scheduled publication
visibility changes
retraction
federation
moderation
signing
deletion
```

The governing principle is:

> **SIAR publishing must be local-first, revision-safe, cryptographically attributable where desired, privacy-scoped, resilient to offline operation, and incapable of silently exposing creator identity or draft history.**

---

# 2. Architectural Position

```text
Authoring
   │
   ▼
Draft State
   │
   ▼
Validation / Signing / Media Finalization
   │
   ▼
Publication Record
   │
   ├── Feed
   ├── Channel
   ├── Community
   ├── Public Discovery
   └── Federation
```

---

# 3. Core Separation

Keep distinct:

```text
draft
published revision
creator identity
publisher pseudonym
media blob
distribution state
moderation state
analytics
```

---

# 4. Non-Goals

Part 88 does not create:

```text
one global author identity
server-owned plaintext drafts by default
immutable public content forever
automatic exposure of edit history
centralized behavioral profiling of creators
```

---

# 5. Content Types

```rust
pub enum PublishedContentType {
    Post,
    Article,
    Announcement,
    ImagePost,
    VideoPost,
    AudioPost,
    LinkPost,
    DocumentPost,
}
```

---

# 6. Post

Short-form.

---

# 7. Article

Long-form.

---

# 8. Announcement

Publisher/channel notice.

---

# 9. Media Post

Primary media attachment.

---

# 10. Document Post

Document/file publication.

---

# 11. Content Identity

```rust
pub struct ContentId(pub [u8; 32]);
```

---

# 12. Revision Identity

```rust
pub struct ContentRevisionId(pub [u8; 32]);
```

---

# 13. Draft Identity

```rust
pub struct DraftId(pub [u8; 32]);
```

---

# 14. Hard Rule

DraftId, ContentId, and ContentRevisionId remain distinct.

---

# 15. Why

Draft metadata should not leak into published identifiers.

---

# 16. Creator Identity Modes

```rust
pub enum CreatorIdentityMode {
    RelationshipScoped,
    SpaceScopedPseudonym,
    PublicPseudonym,
    ManagedIdentity,
    AnonymousPublisherCapability,
}
```

---

# 17. RelationshipScoped

For private contact-sharing.

---

# 18. SpaceScopedPseudonym

Group/community identity.

---

# 19. PublicPseudonym

Public creator identity by choice.

---

# 20. ManagedIdentity

Organization identity.

---

# 21. AnonymousPublisherCapability

Limited publishing without stable public identity where supported.

---

# 22. Hard Rule

Publishing never requires exposing login AccountId.

---

# 23. Creator Reference

```rust
pub struct CreatorRef {
    pub mode: CreatorIdentityMode,
    pub pseudonym: Option<PublisherPseudonym>,
}
```

---

# 24. No Email/Phone In Creator Record

Hard rule.

---

# 25. Creator Profile

Part 84 projection.

---

# 26. Public publisher profile

Separate from private account profile.

---

# 27. Draft State

```rust
pub enum DraftState {
    Editing,
    Ready,
    Scheduled,
    Publishing,
    Published,
    Abandoned,
}
```

---

# 28. Editing

Normal local authoring.

---

# 29. Ready

Passed validation.

---

# 30. Scheduled

Waiting for publish time.

---

# 31. Publishing

Durable workflow in progress.

---

# 32. Published

Content exists.

---

# 33. Abandoned

Local draft no longer active.

---

# 34. Hard Rule

Draft is never considered published until publication commit succeeds.

---

# 35. Draft Record

```rust
pub struct DraftRecord {
    pub draft_id: DraftId,
    pub content_type: PublishedContentType,
    pub creator: CreatorRef,
    pub target: PublicationTarget,
    pub body: DraftBody,
    pub attachments: Vec<DraftAttachmentRef>,
    pub state: DraftState,
    pub version: DraftVersion,
}
```

---

# 36. Draft Version

```rust
pub struct DraftVersion(pub u64);
```

---

# 37. Local-First Draft

Primary authoring state stored locally.

---

# 38. Hard Rule

Network availability is not required to create/edit a draft.

---

# 39. Autosave

Transactional local save.

---

# 40. No Remote Autosave Requirement

Hard rule.

---

# 41. Autosave Frequency

Bounded/debounced.

---

# 42. Crash Recovery

Last committed local version.

---

# 43. No Unbounded In-Memory Draft

Hard rule.

---

# 44. Draft Storage

Encrypted local DB.

---

# 45. Draft Media

Encrypted local blob store.

---

# 46. Draft Encryption

At rest.

---

# 47. Server Draft Sync

Optional.

---

# 48. Default

Encrypted account sync only if enabled.

---

# 49. No Plaintext Cloud Draft Requirement

Hard rule.

---

# 50. Draft Sync Classification

```rust
pub enum DraftSyncMode {
    LocalOnly,
    EncryptedAccountSync,
    ManagedWorkspaceSync,
}
```

---

# 51. LocalOnly

Strong privacy.

---

# 52. EncryptedAccountSync

Own devices only.

---

# 53. ManagedWorkspaceSync

Managed workspace policy.

---

# 54. Hard Rule

Draft sync mode is explicit per draft/workspace.

---

# 55. Draft Sync Envelope

```rust
pub struct DraftSyncEnvelope {
    pub draft: DraftId,
    pub version: DraftVersion,
    pub encrypted_payload: EncryptedDraftPayload,
}
```

---

# 56. No plaintext body server-side where E2EE draft sync enabled.

---

# 57. Multi-Device Draft Editing

Potential conflicts.

---

# 58. Conflict Modes

```rust
pub enum DraftConflictPolicy {
    ManualMerge,
    StructuredMerge,
    LastWriterWinsAllowed,
}
```

---

# 59. Default Long-Form

Manual/structured merge.

---

# 60. Short low-risk metadata

LWW allowed.

---

# 61. Hard Rule

Never silently lose article body through naive LWW.

---

# 62. Structured Text Model

Possible:

```text
block-based document
operation log
CRDT
```

---

# 63. Initial Recommendation

Versioned block/document model with explicit merge.

---

# 64. CRDT

Optional later.

---

# 65. No custom complex CRDT until needed.

---

# 66. Draft Collaboration

Not baseline.

---

# 67. If added

Separate architecture.

---

# 68. Publication Target

```rust
pub enum PublicationTarget {
    DirectRelationship(RelationshipId),
    SocialSpace(SocialSpaceId),
    Channel(ChannelId),
    PublicPublisher(PublisherPseudonym),
    ManagedWorkspace(OrganizationId),
    FederationTarget(FederatedPublicationTarget),
}
```

---

# 69. Target determines visibility/authority.

---

# 70. No Publish Without Target Capability

Hard rule.

---

# 71. Visibility

```rust
pub enum ContentVisibility {
    PrivateRelationship,
    MembersOnly,
    Subscribers,
    Public,
    Managed,
}
```

---

# 72. Visibility Derived From Target/Policy

---

# 73. User may request stricter visibility.

---

# 74. Cannot widen beyond target policy without authorization.

---

# 75. Hard Rule

Publishing preference cannot override authorization/privacy floor.

---

# 76. Publication Validation

Before publish:

```text
target authorization
creator capability
body bounds
media availability
visibility policy
moderation preconditions
```

---

# 77. Validator Trait

```rust
pub trait PublicationValidator {
    fn validate(
        &self,
        draft: &DraftRecord,
    ) -> Result<ValidatedPublication, PublishError>;
}
```

---

# 78. No Network Calls Inside Final State Commit

Hard rule.

---

# 79. Media Finalization

Attachments finalized before publication commit or referenced via durable upload workflow.

---

# 80. Media Attachment

```rust
pub struct PublicationAttachment {
    pub blob: BlobId,
    pub media_type: MediaType,
    pub digest: ContentDigest,
    pub dimensions: Option<MediaDimensions>,
}
```

---

# 81. Media Blob

Content-addressed/encrypted where private.

---

# 82. Public Media

Can be plaintext/public CDN object by policy.

---

# 83. Private Media

Encrypt-before-upload.

---

# 84. Hard Rule

Private publication media never relies solely on transport TLS for confidentiality.

---

# 85. Media Metadata

Strip:

```text
EXIF
GPS
device model
camera serial
```

unless user explicitly preserves.

---

# 86. Hard Rule

Sensitive metadata stripped by default.

---

# 87. Media Processing

Resize/transcode/thumbnail.

---

# 88. Local-first where feasible.

---

# 89. Server processing

Only on content server is allowed to inspect.

---

# 90. Private E2EE media

Processing before encryption or on-device.

---

# 91. Hard Rule

Server cannot decrypt private media merely to generate thumbnails.

---

# 92. Thumbnail

Generate before encryption.

---

# 93. Public Media Pipeline

Can use server processing.

---

# 94. Separate trust path.

---

# 95. Article Body Model

```rust
pub struct ArticleDocument {
    pub title: ArticleTitle,
    pub blocks: Vec<ArticleBlock>,
}
```

---

# 96. Blocks

```rust
pub enum ArticleBlock {
    Paragraph(TextBlock),
    Heading(HeadingBlock),
    Quote(TextBlock),
    Code(CodeBlock),
    Image(MediaRef),
    Video(MediaRef),
    Link(LinkBlock),
}
```

---

# 97. Block Count Bounded.

---

# 98. No Arbitrary Active HTML

Hard rule.

---

# 99. Sanitized Markdown

Optional source.

---

# 100. Render to safe internal block model.

---

# 101. Scripts

Forbidden.

---

# 102. Iframes

Forbidden by default.

---

# 103. External Embed

Explicit safe embed/proxy policy.

---

# 104. No Remote Tracking Pixel

Hard rule.

---

# 105. Link Metadata

Fetched according to privacy setting.

---

# 106. Max-anonymity

No direct external preview fetch.

---

# 107. Post Body

```rust
pub struct PostBody {
    pub text: BoundedText,
    pub links: Vec<SafeLinkRef>,
}
```

---

# 108. Text Length Bounded.

---

# 109. Mentions

Scoped.

---

# 110. Mention Reference

```rust
pub enum MentionRef {
    Relationship(RelationshipId),
    Membership(MembershipPseudonym),
    PublicPublisher(PublisherPseudonym),
}
```

---

# 111. No Raw AccountId Mention

Hard rule.

---

# 112. Mention Notification

Capability/visibility checked.

---

# 113. No Mention Enumeration Oracle

Hard rule.

---

# 114. Hashtags/Topics

Optional public metadata.

---

# 115. Topic Tags

```rust
pub struct TopicTag(pub String);
```

---

# 116. Normalize.

---

# 117. Bounded count.

---

# 118. Private content

Do not send topic tags to public index.

---

# 119. Hard rule.

---

# 120. Content Signing

Useful for provenance.

---

# 121. Publication Signature

```rust
pub struct PublicationSignature {
    pub signer: PublisherSigningKeyId,
    pub content_digest: ContentDigest,
    pub signature: SignatureBytes,
}
```

---

# 122. Signer Identity

Publisher pseudonym/space identity.

---

# 123. Not login identity.

---

# 124. Hard rule.

---

# 125. Public Posts

Can be signed.

---

# 126. Private Group Messages

Use group/message crypto instead.

---

# 127. Signature Scope

Content revision.

---

# 128. Revision changes require new signature.

---

# 129. No Signature Reuse Across Revisions

Hard rule.

---

# 130. Provenance

```rust
pub struct ContentProvenance {
    pub content: ContentId,
    pub revision: ContentRevisionId,
    pub parent_revision: Option<ContentRevisionId>,
    pub signature: Option<PublicationSignature>,
}
```

---

# 131. No creator login identity embedded.

---

# 132. Revision Lifecycle

```rust
pub enum RevisionState {
    Draft,
    Published,
    Superseded,
    Retracted,
    Deleted,
}
```

---

# 133. First Publication

Revision 1.

---

# 134. Edit

Creates new revision.

---

# 135. Old revision

Superseded.

---

# 136. Public Revision History

Policy-controlled.

---

# 137. Private Revision History

Local/creator only by default.

---

# 138. Hard Rule

Editing content does not automatically expose all prior versions publicly.

---

# 139. Revision Visibility Policy

```rust
pub enum RevisionHistoryPolicy {
    Hidden,
    CurrentOnly,
    PublicHistory,
    ModeratorVisible,
}
```

---

# 140. PublicHistory

Explicit.

---

# 141. CurrentOnly

Most common.

---

# 142. ModeratorVisible

Space moderation.

---

# 143. Hidden

Private relationship content.

---

# 144. Revision Update

Atomic.

---

# 145. New revision published before current pointer changes.

---

# 146. No Partial Published Revision

Hard rule.

---

# 147. Content Head

```rust
pub struct ContentHead {
    pub content: ContentId,
    pub current_revision: ContentRevisionId,
    pub version: u64,
}
```

---

# 148. CAS Update

Prevent edit races.

---

# 149. Concurrent Edit

Conflict.

---

# 150. No Last-Writer Silently Deletes Published Revision

Hard rule.

---

# 151. Scheduled Publishing

Durable.

---

# 152. Schedule Record

```rust
pub struct ScheduledPublication {
    pub draft: DraftId,
    pub schedule: PublishSchedule,
    pub state: ScheduleState,
}
```

---

# 153. Schedule State

```rust
pub enum ScheduleState {
    Pending,
    Due,
    Publishing,
    Completed,
    Cancelled,
    Failed,
}
```

---

# 154. Time Semantics

Part 60.

---

# 155. User-specified wall-clock schedule.

---

# 156. Store timezone intent if needed locally.

---

# 157. Server schedule normalized.

---

# 158. Hard Rule

Critical publish timing does not depend on device being online if server scheduling enabled.

---

# 159. Local-Only Schedule

May publish when device reconnects.

---

# 160. UI must distinguish.

---

# 161. No false exact-time guarantee offline.

---

# 162. Hard truth.

---

# 163. Scheduled Draft Encryption

Server may store encrypted draft until due.

---

# 164. To publish public plaintext, server needs publishable payload/key.

---

# 165. Options:

```text
pre-encrypted public payload
trusted managed publisher
device-online execution
```

---

# 166. Private E2EE scheduled content

requires careful key availability.

---

# 167. No server escrow of universal private keys.

---

# 168. Hard rule.

---

# 169. Publication Workflow

```text
validate
→ finalize media
→ construct revision
→ sign
→ persist revision + head
→ create distribution outbox
→ commit
→ distribute asynchronously
```

---

# 170. Persist Before Distribution

Hard rule.

---

# 171. Distribution Outbox

Part 75.

---

# 172. Distribution Targets

```text
feed index
channel fanout
subscriber delivery
public search
federation
notification hints
```

---

# 173. No direct synchronous fanout in publish transaction.

---

# 174. Hard rule.

---

# 175. Publication State

```rust
pub enum PublicationState {
    Pending,
    Committed,
    Distributing,
    Distributed,
    PartiallyDistributed,
    FailedDistribution,
}
```

---

# 176. Publication Commit Success

Does not require all downstream delivery complete.

---

# 177. UI

Shows "published, distributing" if needed.

---

# 178. No false atomic-global-publication claim.

---

# 179. Hard truth.

---

# 180. Distribution Idempotency

Required.

---

# 181. Same revision distributed twice

No duplicate feed item.

---

# 182. Stable distribution event ID.

---

# 183. Feed Integration

Part 87.

---

# 184. Feed receives publication projection.

---

# 185. Ranking separate.

---

# 186. Publisher cannot force ranked placement except explicit pin/admin policy.

---

# 187. Hard rule.

---

# 188. Social Space Integration

Part 86.

---

# 189. Channel/group publication requires membership/capability.

---

# 190. Moderator/publication roles.

---

# 191. Announcement channel

publisher capability required.

---

# 192. No Role Name Shortcut.

---

# 193. Contact Integration

Direct publication to relationship.

---

# 194. Uses relationship capability.

---

# 195. Not global feed.

---

# 196. Federation Distribution

Cross-domain publication.

---

# 197. Federation gateway sends signed publication projection.

---

# 198. No raw internal draft/revision state.

---

# 199. Hard rule.

---

# 200. Public Federation

Can cache public content.

---

# 201. Private federation

Capability-controlled encrypted content.

---

# 202. No gateway plaintext if E2EE.

---

# 203. Creator Identity Across Federation

Domain scoped.

---

# 204. No global AccountId merge.

---

# 205. Hard rule.

---

# 206. Retraction

Author withdraws published content.

---

# 207. Retraction State

```rust
pub enum RetractionReason {
    AuthorChoice,
    Error,
    SecurityIssue,
    PolicyViolation,
    LegalRequirement,
}
```

---

# 208. Retraction Flow

```text
authorize
→ create retraction marker
→ update content head/state
→ distribute retraction
→ remove from discovery/feed
```

---

# 209. No Hard Delete Required Immediately.

---

# 210. Retraction vs Deletion

Separate.

---

# 211. Hard Rule

UI/API distinguish retracted from deleted.

---

# 212. Retraction Propagation

Asynchronous.

---

# 213. Cached copies may remain temporarily.

---

# 214. Public archives may preserve according to policy.

---

# 215. No universal erasure claim.

---

# 216. Hard truth.

---

# 217. Deletion

Stronger lifecycle action.

---

# 218. Deletion Policy Depends On

```text
public/private
retention
legal
ownership
```

---

# 219. Private content

Cryptographic erasure possible.

---

# 220. Public content

Copies may exist externally.

---

# 221. No claim external copies erased.

---

# 222. Hard rule.

---

# 223. Deletion Tombstone

Prevent stale federation/cache resurrection.

---

# 224. Content Deletion Epoch

```rust
pub struct ContentDeletionEpoch(pub u64);
```

---

# 225. Monotonic.

---

# 226. Stale revision older than epoch rejected.

---

# 227. Hard rule.

---

# 228. Restore/Backup

Part 33/67.

---

# 229. Draft backup

Logical encrypted.

---

# 230. Published creator state

Logical.

---

# 231. Live distribution jobs

Reconstructed/idempotent.

---

# 232. Deleted/retracted state

Must survive restore.

---

# 233. Hard rule.

---

# 234. Media Deletion

Blob refs.

---

# 235. GC after references removed/retention satisfied.

---

# 236. No cross-user dedup strict mode.

---

# 237. Shared Public Blob

If public dedup enabled, deletion semantics reference-counted.

---

# 238. Private content

Per-publication key.

---

# 239. Cryptographic erasure via key deletion.

---

# 240. Creator State

Need explicit lifecycle.

---

# 241. Creator Workspace

```rust
pub struct CreatorState {
    pub drafts: Vec<DraftId>,
    pub scheduled: Vec<DraftId>,
    pub publications: Vec<ContentId>,
    pub publisher_profiles: Vec<PublisherPseudonym>,
}
```

---

# 242. Stored locally/account encrypted.

---

# 243. No global creator dashboard requirement.

---

# 244. Creator State Sync

Encrypted.

---

# 245. No server behavioral profile.

---

# 246. Hard rule.

---

# 247. Publisher Pseudonym Lifecycle

```rust
pub enum PublisherState {
    Active,
    Paused,
    Rotating,
    Retired,
}
```

---

# 248. Rotation

Possible.

---

# 249. Public continuity proof optional.

---

# 250. No forced link to old pseudonym.

---

# 251. Hard rule.

---

# 252. Creator Profile

Separate from account profile.

---

# 253. Multiple public pseudonyms allowed.

---

# 254. No requirement to link them.

---

# 255. Hard rule.

---

# 256. Publisher Signing Key

Separate per pseudonym/domain.

---

# 257. No one signing key across all public identities.

---

# 258. Hard rule.

---

# 259. Publisher Key Rotation

Signed continuity statement if desired.

---

# 260. No silent substitution.

---

# 261. Identity change warning if verification used.

---

# 262. Content Ownership

```rust
pub enum ContentOwnership {
    Personal,
    SocialSpace,
    ManagedOrganization,
    SharedCollaborative,
}
```

---

# 263. Personal

Creator controls.

---

# 264. SocialSpace

Space policy.

---

# 265. ManagedOrganization

Org policy.

---

# 266. SharedCollaborative

Future collaboration.

---

# 267. Ownership Determines Delete/Edit Authority

Hard rule.

---

# 268. No Account Row Ownership Shortcut

Hard rule.

---

# 269. Moderation

Separate from creator authority.

---

# 270. Moderator may hide/remove within space.

---

# 271. Moderator does not become creator.

---

# 272. Hard rule.

---

# 273. Moderated Content State

```rust
pub enum ModerationVisibility {
    Visible,
    Limited,
    Hidden,
    Removed,
}
```

---

# 274. Content Revision Remains Creator State

Depending retention.

---

# 275. No silent content rewrite by moderator.

---

# 276. Hard rule.

---

# 277. Moderator Action

Can:

```text
hide
remove from space
label
lock replies
```

---

# 278. Cannot alter author's signed body.

---

# 279. Hard rule.

---

# 280. If correction needed

new author revision.

---

# 281. Managed Announcement

Org publisher may update according to policy.

---

# 282. Distinguish creator vs org authority.

---

# 283. Publication Capability

```rust
pub struct PublicationCapability {
    pub target: PublicationTarget,
    pub actions: BTreeSet<PublicationAction>,
    pub expires_at: Option<Timestamp>,
}
```

---

# 284. Publication Actions

```rust
pub enum PublicationAction {
    Create,
    Revise,
    Retract,
    Delete,
    Schedule,
    Pin,
}
```

---

# 285. Pin

Separate elevated capability.

---

# 286. No ordinary creator automatic pin.

---

# 287. Hard rule.

---

# 288. Content Validation

Security.

---

# 289. Text bounds.

---

# 290. Media size bounds.

---

# 291. MIME sniffing.

---

# 292. No trust in file extension.

---

# 293. Hard rule.

---

# 294. Malware Risk

Attachments may contain malicious files.

---

# 295. For public downloads:

```text
safe metadata
content-disposition
sandbox viewer
```

---

# 296. No auto-execution.

---

# 297. Hard rule.

---

# 298. Archive Files

Restricted preview.

---

# 299. Document Rendering

Sandboxed.

---

# 300. PDF/media pipeline Part 40/other components.

---

# 301. Image Decoder

Memory limits.

---

# 302. Video

Codec limits.

---

# 303. Zip bomb

Detect bounds.

---

# 304. Hard rule.

---

# 305. Content Hash

Integrity.

---

# 306. Digest algorithm registry Part 66.

---

# 307. No MD5/SHA1 for security integrity.

---

# 308. Content Addressing

Use modern digest.

---

# 309. Public Provenance

Can expose digest/signature.

---

# 310. Private content

Digest not globally published.

---

# 311. Hard rule.

---

# 312. Scheduled Revision

Can schedule update.

---

# 313. Current content remains until commit.

---

# 314. No half-updated state.

---

# 315. Publication Lock

Optimistic version.

---

# 316. Concurrent Creator Devices

CAS.

---

# 317. Conflict notification.

---

# 318. No silent overwrite.

---

# 319. Hard rule.

---

# 320. Draft Collaboration Future

Would require:

```text
collaborator capability
operation sync
authorship attribution
```

---

# 321. Not baseline.

---

# 322. Comments/Replies

Could be social interactions.

---

# 323. Reply Target

Content revision/content ID.

---

# 324. Visibility inherited/scoped.

---

# 325. No reply to content user cannot access.

---

# 326. Hard rule.

---

# 327. Threading

Local/server metadata.

---

# 328. No global reply graph across private spaces.

---

# 329. Hard rule.

---

# 330. Mention Notifications

Opaque push hint.

---

# 331. No full content in push.

---

# 332. Hard rule.

---

# 333. Public Search

Published public content only.

---

# 334. Drafts never indexed.

---

# 335. Private content never indexed centrally.

---

# 336. Hard rule.

---

# 337. Search Index Update

After publication commit via outbox.

---

# 338. Retraction removes index entry.

---

# 339. Deletion removes/tombstones.

---

# 340. No search-before-commit.

---

# 341. Hard rule.

---

# 342. Feed Distribution

Part 87.

---

# 343. Feed candidate after commit.

---

# 344. No direct creator access to ranking score.

---

# 345. Creator may see aggregate distribution status.

---

# 346. No algorithm gaming API.

---

# 347. Hard rule.

---

# 348. Creator Analytics

Privacy-safe only.

---

# 349. Allowed:

```text
aggregate public views
reaction buckets
subscriber-count bucket
distribution success
```

---

# 350. Forbidden by default:

```text
viewer identities
exact read timestamps
viewer contact graph
precise locations
per-viewer dwell time
```

---

# 351. Hard rule.

---

# 352. Small Audience

Suppress analytics below threshold.

---

# 353. Prevent deanonymization.

---

# 354. Creator Analytics Threshold

```rust
pub struct AnalyticsThreshold {
    pub minimum_cohort: u32,
}
```

---

# 355. No small cohort breakdown.

---

# 356. Hard rule.

---

# 357. Distribution Status

```rust
pub struct DistributionStatus {
    pub target_count_class: DistributionCountClass,
    pub state: PublicationState,
}
```

---

# 358. Coarse.

---

# 359. No subscriber identity list.

---

# 360. Hard rule.

---

# 361. Content Recommendation Metadata

Publisher can provide topics/categories.

---

# 362. Cannot provide user-targeting segment IDs.

---

# 363. Hard rule.

---

# 364. No "show this to users like X" behavioral target baseline.

---

# 365. Public Promotion

Future economics/ads separate.

---

# 366. Must be labeled if ever implemented.

---

# 367. Not core.

---

# 368. Anti-Spam Publishing

Rate limits.

---

# 369. Source quotas.

---

# 370. Storage quotas.

---

# 371. No invasive identity verification requirement.

---

# 372. Hard rule.

---

# 373. Public Publisher Abuse

Can suspend publisher pseudonym/space capability.

---

# 374. Does not automatically disable personal account unless separate authority.

---

# 375. Hard rule.

---

# 376. Publication Rate Policy

```rust
pub struct PublicationQuota {
    pub max_items: u32,
    pub window: Duration,
    pub max_bytes: u64,
}
```

---

# 377. Tenant/social-space scoped.

---

# 378. No global user behavior score.

---

# 379. Draft Limits

Bounded count/bytes.

---

# 380. No unlimited cloud drafts.

---

# 381. Local disk limits configurable.

---

# 382. Background Media Upload

Resumable.

---

# 383. Publish can wait for upload completion.

---

# 384. Or public article published without optional media? policy-specific.

---

# 385. No broken references at commit.

---

# 386. Hard rule.

---

# 387. Media Upload Session

```rust
pub struct MediaUploadSession {
    pub upload_id: UploadId,
    pub blob: BlobId,
    pub expected_digest: ContentDigest,
    pub state: UploadState,
}
```

---

# 388. Upload State

```rust
pub enum UploadState {
    Pending,
    Uploading,
    Verifying,
    Complete,
    Failed,
    Cancelled,
}
```

---

# 389. Verify digest before publication.

---

# 390. Hard rule.

---

# 391. Offline Public Publishing

Draft local.

---

# 392. Publish command queued.

---

# 393. On reconnect:

```text
revalidate auth/policy
finalize media
commit
```

---

# 394. Hard Rule

Offline queued publish does not bypass current policy at reconnect.

---

# 395. Scheduled Offline Publishing

If no server schedule exists, best-effort when online.

---

# 396. UI truthful.

---

# 397. Content Expiry

Optional.

---

# 398. Ephemeral publication.

---

# 399. Expiry Policy

```rust
pub enum ContentExpiryPolicy {
    Never,
    After(Duration),
    At(Timestamp),
}
```

---

# 400. Expiry Is Not Secure Erasure

Hard truth.

---

# 401. Expired content

Removed from active distribution.

---

# 402. Cached copies may exist.

---

# 403. Private encrypted content

key expiry/destruction can strengthen.

---

# 404. No false ephemeral claim.

---

# 405. Content Versioning

Storage/API/protocol versions separate.

---

# 406. Revision schema migration.

---

# 407. Unknown block

safe placeholder or reject.

---

# 408. Security-sensitive unknown

reject.

---

# 409. No silent active-content interpretation.

---

# 410. Hard rule.

---

# 411. Serialization

Postcard internal/protocol.

---

# 412. RON human export/config.

---

# 413. JSON external interop.

---

# 414. Article export

Markdown possible.

---

# 415. Sanitized.

---

# 416. No active script.

---

# 417. Creator Export

Exports:

```text
drafts
published content
revision metadata
publisher profile
```

---

# 418. Excludes:

```text
live session tokens
private signing key unless explicit secure key export
subscriber identities
```

---

# 419. Hard rule.

---

# 420. Publisher Key Backup

Part 33/57/66.

---

# 421. May backup encrypted logical key if user chooses.

---

# 422. HSM/secure element key may require continuity method.

---

# 423. No blind key cloning.

---

# 424. Hard rule.

---

# 425. Publisher Migration

Cross-provider/federation.

---

# 426. Export signed publisher continuity proof.

---

# 427. New domain can reference.

---

# 428. No global identity authority.

---

# 429. Hard rule.

---

# 430. Managed Publishing

Organization workspace.

---

# 431. Draft may be org-owned.

---

# 432. Ownership policy explicit.

---

# 433. Offboarding

Personal drafts remain personal if personal-owned.

---

# 434. Org-owned drafts remain org.

---

# 435. Hard rule.

---

# 436. Approval Workflow

Optional for managed publishing.

---

# 437. States:

```rust
pub enum EditorialState {
    Draft,
    InReview,
    Approved,
    Rejected,
    Published,
}
```

---

# 438. Reviewer capability scoped.

---

# 439. No reviewer automatic publish unless permitted.

---

# 440. Audit managed workflow.

---

# 441. No hidden personal draft audit.

---

# 442. Hard rule.

---

# 443. Publication Approval

Durable workflow.

---

# 444. Current revision approved by digest.

---

# 445. If draft changes after approval

approval invalidated.

---

# 446. Hard rule.

---

# 447. Approval Record

```rust
pub struct EditorialApproval {
    pub draft: DraftId,
    pub draft_digest: ContentDigest,
    pub approver: ManagedPublisherId,
}
```

---

# 448. No approving mutable pointer.

---

# 449. Good invariant.

---

# 450. Creator Workspace Search

Local.

---

# 451. Draft search index local encrypted/derived.

---

# 452. No central indexing of private drafts.

---

# 453. Hard rule.

---

# 454. Public Published Search

Central/public index allowed.

---

# 455. Private publication search local.

---

# 456. Observability

Safe metrics:

```text
publish success
distribution lag
media upload failures
revision conflict rate
```

---

# 457. Forbidden metrics:

```text
draft content
typing cadence
edit keystroke history
private topic interests
```

---

# 458. Hard rule.

---

# 459. No Keystroke Telemetry

Hard rule.

---

# 460. Draft Autosave Metrics

Only count/latency aggregate.

---

# 461. Publisher Analytics Separate

Thresholded and privacy-safe.

---

# 462. Publication SLOs

Examples:

```text
draft durability
publish commit latency
distribution completion
media integrity
```

---

# 463. Security SLO

```text
0 private draft indexed publicly
0 publication without target capability
```

---

# 464. Privacy SLO

```text
0 login AccountId in public creator metadata
0 viewer identity analytics
```

---

# 465. Failure Modes

```text
media upload fails
signing fails
distribution partially fails
federation unavailable
scheduler unavailable
```

---

# 466. Media Upload Failure

Draft remains.

---

# 467. Signing Failure

No publish.

---

# 468. Distribution Partial Failure

Publication remains committed; retry downstream.

---

# 469. Federation Down

Local publication succeeds; remote distribution retries.

---

# 470. Scheduler Down

Scheduled jobs recover from durable state.

---

# 471. No Duplicate Publication

Idempotency.

---

# 472. Hard rule.

---

# 473. Publication Idempotency Key

```rust
pub struct PublishOperationId(pub [u8; 32]);
```

---

# 474. Same operation returns same committed result.

---

# 475. No duplicate public post from retry.

---

# 476. Hard rule.

---

# 477. Event Bus Integration

Part 75.

---

# 478. Events:

```text
ContentPublished
ContentRevised
ContentRetracted
ContentDeleted
MediaFinalized
```

---

# 479. Minimal metadata.

---

# 480. No draft body on bus.

---

# 481. Hard rule.

---

# 482. Database Integration

Part 74.

---

# 483. Draft store, publication store, media metadata separate logically.

---

# 484. Transaction boundary:

```text
revision + content head + outbox
```

---

# 485. No network call inside transaction.

---

# 486. Hard rule.

---

# 487. Authorization Integration

Part 81.

---

# 488. Creator capability checked.

---

# 489. Revise/retract/delete separate actions.

---

# 490. No create capability implies delete.

---

# 491. Hard rule.

---

# 492. Authentication Integration

Part 82.

---

# 493. Sensitive public identity/key changes may require step-up.

---

# 494. Routine posting does not require repeated MFA.

---

# 495. Account Lifecycle

Part 83.

---

# 496. Deleted account cannot publish.

---

# 497. Suspended publisher policy-specific.

---

# 498. Managed offboarding revokes managed publish capabilities.

---

# 499. No stale scheduled publish after deletion/offboarding.

---

# 500. Hard rule.

---

# 501. Profile Integration

Part 84.

---

# 502. Publisher profile projection.

---

# 503. Personal profile remains separate.

---

# 504. Contacts Integration

Part 85.

---

# 505. Direct contact posts use relationship capability.

---

# 506. Social Space Integration

Part 86.

---

# 507. Channel/community publication authority.

---

# 508. Feed Integration

Part 87.

---

# 509. Feed index after commit.

---

# 510. Ranking separate.

---

# 511. Edge Integration

Part 78.

---

# 512. Public publishing API L7.

---

# 513. Media upload bounded.

---

# 514. Private E2EE payload opaque to WAF.

---

# 515. East-West Integration

Part 79.

---

# 516. Publisher/media/index services mTLS.

---

# 517. Secrets Integration

Part 80.

---

# 518. Signing keys short-lived/secure handles where possible.

---

# 519. No signing private key in logs/env.

---

# 520. Federation Integration

Part 58.

---

# 521. Signed public publication envelope.

---

# 522. No raw internal service identifiers.

---

# 523. Crate Layout

Recommended:

```text
crates/
├── siar-publish-core/
├── siar-draft/
├── siar-content-document/
├── siar-content-revision/
├── siar-publication/
├── siar-media-publication/
├── siar-publication-signing/
├── siar-publication-scheduler/
├── siar-publication-distribution/
├── siar-creator-state/
├── siar-publication-observability/
└── siar-publication-testkit/
```

---

# 524. `siar-publish-core`

Owns:

```text
content IDs
publication targets
visibility
errors
```

---

# 525. `siar-draft`

Local draft lifecycle/sync.

---

# 526. `siar-content-document`

Safe post/article block model.

---

# 527. `siar-content-revision`

Immutable revisions/head/CAS.

---

# 528. `siar-publication`

Commit/retract/delete workflow.

---

# 529. `siar-media-publication`

Attachment finalization/transcoding/metadata stripping.

---

# 530. `siar-publication-signing`

Publisher pseudonym provenance.

---

# 531. `siar-publication-scheduler`

Durable scheduled publishing.

---

# 532. `siar-publication-distribution`

Feed/channel/search/federation outbox consumers.

---

# 533. `siar-creator-state`

Publisher/draft/scheduled state.

---

# 534. `siar-publication-observability`

Aggregate publish/distribution metrics.

---

# 535. `siar-publication-testkit`

offline/revision/media/distribution/privacy tests.

---

# 536. Error Taxonomy

```rust
pub enum PublishError {
    DraftNotFound,
    DraftConflict,
    InvalidContent,
    AttachmentIncomplete,
    AttachmentDigestMismatch,
    PublicationDenied,
    VisibilityDenied,
    SigningFailed,
    ScheduleInvalid,
    PublicationAlreadyCommitted,
    RevisionConflict,
    ContentRetracted,
    ContentDeleted,
    DistributionFailed,
    Internal,
}
```

---

# 537. Testing

Need dedicated publishing testkit.

---

# 538. Test Scenarios

```text
offline draft
publish retry
revision race
scheduled publish
media failure
```

---

# 539. Offline Draft Test

Create/edit without network.

---

# 540. Draft Crash Test

Last committed autosave restored.

---

# 541. Multi-Device Conflict Test

No silent article-body loss.

---

# 542. Unauthorized Publish Test

Target capability required.

---

# 543. Media Metadata Test

EXIF/GPS stripped.

---

# 544. Media Digest Test

Corrupt upload rejected.

---

# 545. Private Media Test

Server cannot generate plaintext thumbnail post-encryption.

---

# 546. Publication Commit Test

Revision/head/outbox atomic.

---

# 547. Publish Retry Test

No duplicate post.

---

# 548. Distribution Failure Test

Committed publication retries asynchronously.

---

# 549. Revision Race Test

CAS conflict, no silent overwrite.

---

# 550. Retraction Test

Removed from active feed/discovery.

---

# 551. Deletion Test

Stale federation/cache cannot resurrect.

---

# 552. Scheduled Publish Test

Durable timer survives restart.

---

# 553. Offline Scheduled Test

UI indicates best-effort when device execution required.

---

# 554. Identity Test

AccountId never appears in public creator metadata.

---

# 555. Pseudonym Rotation Test

Continuity proof optional/not forced.

---

# 556. Moderator Test

Moderator cannot rewrite signed author body.

---

# 557. Managed Approval Test

Approval bound to exact draft digest.

---

# 558. Account Deletion Test

Pending schedule cannot publish after account deletion.

---

# 559. Federation Test

Remote receives publication projection, not draft/internal state.

---

# 560. Analytics Test

No viewer identities exposed.

---

# 561. Search Test

Draft/private content never enters public index.

---

# 562. Backup Test

Deleted/retracted state survives restore.

---

# 563. Fuzzing

Fuzz:

```text
article blocks
publication envelope
revision chain
media metadata
federated publication envelope
```

---

# 564. Property Tests

Properties:

```text
only committed publication can enter distribution
content head always references committed revision
deleted/retracted content cannot become active from stale replica
creator account identity is never present in public publisher projection
```

---

# 565. Formal Verification Targets

Strong candidates:

```text
draft→publish state machine
revision/head CAS
publish/distribution outbox
scheduled publish cancellation
deletion anti-resurrection
```

---

# 566. Kani Candidate

revision/state transition invariants.

---

# 567. TLA+ Candidate

publish commit + distribution retry + retraction race.

---

# 568. Loom Candidate

concurrent revision/publish/retract.

---

# 569. Performance

Draft editing

local.

---

# 570. Publish commit

small metadata transaction.

---

# 571. Media upload

streaming/resumable.

---

# 572. Distribution

async.

---

# 573. Public search indexing

async.

---

# 574. No fanout in request thread.

---

# 575. Hard rule.

---

# 576. Large Article

Block streaming/limits.

---

# 577. No giant single DB blob required.

---

# 578. Media references separate.

---

# 579. Feed projection small.

---

# 580. Creator Workspace Pagination

Draft/publication list paged.

---

# 581. No load-all-history.

---

# 582. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Draft, published content, and immutable revision identities remain separate; draft identifiers never become public publication identifiers.
2. Draft authoring is local-first and does not require server/network availability.
3. Private drafts and private publication media are never required to exist as server-side plaintext.
4. Publication requires explicit target-scoped capability and cannot widen visibility beyond policy/authorization.
5. Public creator/publisher pseudonyms remain separate from account/login, transport, mailbox, payment, and telemetry identities.
6. Media metadata such as GPS/EXIF/device identifiers is stripped by default before publication.
7. A publication becomes visible to distribution systems only after its revision, content head, and outbox commit durably.
8. Publication retries are idempotent and cannot create duplicate posts.
9. Revisions are immutable; edits create a new revision and concurrent edits cannot silently overwrite each other.
10. Moderators may control visibility inside their scope but cannot silently rewrite an author's signed content.
11. Retraction/deletion epochs prevent stale caches, backups, federation replicas, or delayed jobs from resurrecting removed content.
12. Creator analytics are aggregate/thresholded and never expose viewer identities, detailed dwell behavior, contact graphs, or precise location histories.
```

---

# 583. Initial Production Scope

Implement first:

```text
typed DraftId/ContentId/ContentRevisionId
local encrypted draft store
autosave/crash recovery
post/article safe block model
publication targets/visibility
target-scoped publication capabilities
media upload/finalization/digest verification
EXIF/GPS stripping
immutable content revisions + CAS head
publisher pseudonym/signing support
transactional publication + distribution outbox
feed/channel/public-search distribution
retraction/deletion tombstones
durable scheduled publishing
encrypted optional multi-device draft sync
thresholded creator analytics
publishing testkit
```

Then add:

```text
collaborative drafts
CRDT-based coauthoring
advanced public provenance chains
cross-provider publisher migration
privacy-preserving creator analytics
formal publishing/distribution verification
```

---

# 584. Definition of Done

Part 88 is complete when:

- drafts are local-first and crash-safe
- draft/publication/revision identities are separate
- public creator identity is pseudonymous/scoped
- publication requires target capability
- private draft/media state stays encrypted
- media metadata is stripped by default
- revisions are immutable and race-safe
- publication commit precedes distribution
- distribution is idempotent and asynchronous
- scheduled publishing is durable and policy-aware
- retraction/deletion anti-resurrection works
- moderation cannot rewrite creator-signed body
- private/public search boundaries are explicit
- creator analytics remain thresholded/privacy-safe
- offline/revision/media/distribution/fuzz/formal tests are specified

---

# 585. Final Architecture

```text
                    AUTHORING
                       │
                       ▼
                 LOCAL DRAFT
                       │
                       ▼
          VALIDATION / MEDIA FINALIZATION
                       │
                       ▼
               IMMUTABLE REVISION
                       │
                       ▼
               PUBLICATION COMMIT
                       │
                       ▼
              DISTRIBUTION OUTBOX
             ┌────────┼────────┐
             │        │        │
           FEED    CHANNEL  FEDERATION
             │        │        │
             └────────┼────────┘
                       ▼
                 DISCOVERY/DELIVERY
```

Publishing safety model:

```text
local-first drafts
+
scoped publisher identity
+
immutable revisions
+
transactional publish
+
async idempotent distribution
+
private media encryption
+
anti-resurrection
+
privacy-safe analytics
```

not:

```text
upload every draft and edit to a central server, expose the account behind every pseudonym, then synchronously fan out publication everywhere
```

---

# 586. Final Principle

Publishing should preserve the creator's control over identity, visibility, revision history, and distribution while keeping draft and behavioral data private by default.

The correct model is:

```text
author locally
+
publish through scoped authority
+
sign as chosen identity
+
commit before distribute
+
revise immutably
+
retract honestly
+
delete with anti-resurrection
+
measure only in aggregate
```

This architecture gives SIAR a local-first, privacy-preserving content-publishing foundation for posts, articles, announcements, media, social spaces, public publishers, managed organizations, and federated distribution while preserving the anonymity, least-authority, and social-graph privacy guarantees established across Parts 34–87.
