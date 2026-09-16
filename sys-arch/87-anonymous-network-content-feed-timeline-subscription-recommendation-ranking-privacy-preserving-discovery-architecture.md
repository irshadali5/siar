# Core System Architecture Part 87 — Anonymous Network Content Feed, Timeline, Subscription, Recommendation, Ranking & Privacy-Preserving Discovery Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 87  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 32, 43, 45–48, 56, 58–59, 67–69, 74–86  

**Primary purpose:** define SIAR's content feed, timeline, subscription, recommendation, ranking, discovery, trending, creator/subscriber privacy, local/on-device personalization, federated content ingestion, anti-manipulation, and privacy-preserving content distribution architecture without creating a centralized behavioral surveillance engine.

---

# 1. Purpose

Content feeds are powerful because they help users discover:

```text
posts
announcements
public channels
communities
articles
media
updates
```

But feed systems are also one of the most privacy-invasive parts of modern platforms.

Typical centralized feeds collect:

```text
what users viewed
what they paused on
what they clicked
what they ignored
who they follow
what topics they prefer
how long they watched
```

The governing principle is:

> **SIAR feeds should help users discover and organize content without requiring centralized behavioral profiles, global social graphs, or opaque engagement-maximizing ranking systems.**

---

# 2. Architectural Position

```text
Subscriptions / Public Sources / Local Signals
                    │
                    ▼
             Feed Candidate Set
                    │
                    ▼
            Privacy Policy Filter
                    │
                    ▼
          Local / Explicit Ranking
                    │
                    ▼
                Timeline
```

---

# 3. Core Separation

Keep distinct:

```text
subscription
feed candidate
timeline item
ranking score
recommendation source
public trend signal
behavioral signal
creator identity
subscriber identity
```

---

# 4. Non-Goals

Part 87 does not create:

```text
a global engagement graph
a centralized per-user interest profile
a mandatory algorithmic feed
a hidden infinite-scroll recommender optimized for addiction
```

---

# 5. Feed Types

```rust
pub enum FeedType {
    Chronological,
    Subscription,
    Curated,
    LocalRecommended,
    PublicDiscovery,
    Trending,
    Managed,
}
```

---

# 6. Chronological Feed

Pure time/order-based.

---

# 7. Subscription Feed

Only explicitly followed sources.

---

# 8. Curated Feed

User/admin-selected sources.

---

# 9. LocalRecommended

On-device recommendation.

---

# 10. PublicDiscovery

Public/listed content.

---

# 11. Trending

Aggregate public trend signal.

---

# 12. Managed

Organization-scoped feed.

---

# 13. Hard Rule

Chronological/subscription feed must always remain available when the feature exists.

---

# 14. Feed Source

```rust
pub enum FeedSource {
    Contact(RelationshipId),
    SocialSpace(SocialSpaceId),
    Channel(ChannelId),
    Publisher(PublisherRef),
    Federation(FederationDomainId),
    LocalCollection(LocalCollectionId),
}
```

---

# 15. Feed Source Identity

Scoped.

---

# 16. No Global User ID Requirement

Hard rule.

---

# 17. Subscription

Explicit user intent.

```rust
pub struct Subscription {
    pub subscription_id: SubscriptionId,
    pub source: FeedSource,
    pub state: SubscriptionState,
    pub delivery: SubscriptionDelivery,
}
```

---

# 18. Subscription State

```rust
pub enum SubscriptionState {
    Active,
    Muted,
    Paused,
    Unsubscribed,
}
```

---

# 19. Muted

Still subscribed but hidden from notification/high-priority feed.

---

# 20. Paused

Temporary stop.

---

# 21. Unsubscribed

No new feed items.

---

# 22. Hard Rule

Subscription does not imply public disclosure of subscriber identity.

---

# 23. Subscriber Privacy

Creator should not receive subscriber list by default.

---

# 24. Public Channels

Can expose aggregate subscriber count only.

---

# 25. No Individual Subscriber Export

Hard rule.

---

# 26. Subscription Identity

Opaque.

---

# 27. No AccountId Embedded In Feed Distribution Key

Hard rule.

---

# 28. Subscription Capability

```rust
pub struct SubscriptionCapability {
    pub source: FeedSource,
    pub permissions: SubscriptionPermissions,
    pub expires_at: Option<Timestamp>,
}
```

---

# 29. Subscription Permissions

```rust
pub struct SubscriptionPermissions {
    pub read: bool,
    pub receive_notifications: bool,
    pub fetch_archive: bool,
}
```

---

# 30. Public Source

May not require durable identity.

---

# 31. Private Source

Requires capability/membership.

---

# 32. Feed Item

```rust
pub struct FeedItem {
    pub item_id: FeedItemId,
    pub source: FeedSource,
    pub content_ref: ContentRef,
    pub published_order: FeedOrderKey,
    pub visibility: FeedVisibility,
}
```

---

# 33. Feed Item ID

Random/opaque.

---

# 34. No User Identity Encoding

Hard rule.

---

# 35. Feed Visibility

```rust
pub enum FeedVisibility {
    Public,
    Subscribers,
    Members,
    RelationshipScoped,
    Managed,
}
```

---

# 36. Candidate Collection

Fetch eligible feed items.

---

# 37. Candidate Sources

```text
subscriptions
contacts
joined social spaces
public discovery
local saved sources
```

---

# 38. No Global Candidate Graph

Hard rule.

---

# 39. Timeline State

```rust
pub struct TimelineState {
    pub cursor: TimelineCursor,
    pub mode: FeedType,
    pub filters: TimelineFilters,
}
```

---

# 40. Cursor

Opaque.

---

# 41. No User Behavior Embedded In Cursor

Hard rule.

---

# 42. Timeline Ordering

Separate from source publication order.

---

# 43. Chronological Ordering

Preferred default.

---

# 44. Ranking Modes

```rust
pub enum RankingMode {
    Chronological,
    UserPinned,
    LocalHeuristic,
    Curated,
}
```

---

# 45. No Opaque Server ML Ranker As Baseline

Hard rule.

---

# 46. Chronological

Deterministic.

---

# 47. UserPinned

User-defined priority.

---

# 48. LocalHeuristic

On-device.

---

# 49. Curated

Transparent source-defined list.

---

# 50. Feed Ranking Principle

Ranking should be:

```text
explainable
local where possible
user-controllable
reversible
```

---

# 51. Hard Rule

Server cannot silently personalize feed using hidden behavioral history.

---

# 52. Local Ranking Inputs

Possible:

```text
explicit follows
manual topic preference
local recency
user pin
mute state
```

---

# 53. Avoid

```text
dwell time
scroll hesitation
rage-clicks
late-night behavior
```

as persistent cloud profile.

---

# 54. Hard Rule

Behavioral telemetry is not required for feed ranking.

---

# 55. Local Behavioral Signals

If used:

```text
stay on device
expire quickly
opt-in
```

---

# 56. No Raw Behavioral Upload

Hard rule.

---

# 57. Preference-Based Ranking

User explicitly chooses interests.

---

# 58. Topic Preference

```rust
pub struct TopicPreference {
    pub topic: TopicId,
    pub weight: ExplicitPreferenceWeight,
}
```

---

# 59. Explicit Preference Weight

Small bounded range.

---

# 60. No Infinite Learned Score

Hard rule.

---

# 61. Ranking Score

```rust
pub struct LocalRankingScore {
    pub explicit_priority: i16,
    pub recency_bucket: u8,
    pub source_priority: u8,
}
```

---

# 62. No hidden psychological score.

---

# 63. Ranking Explanation

UI can say:

```text
Because you follow this channel
Because you pinned this source
Recent from subscribed community
```

---

# 64. No Deceptive "Recommended For You" Without Explanation

Hard rule.

---

# 65. Feed Algorithm Transparency

User can inspect active ranking mode.

---

# 66. User Can Switch To Chronological

Hard rule.

---

# 67. Feed Candidate Deduplication

Deduplicate same content from multiple sources.

---

# 68. Dedup Key

Content identity/digest where safe.

---

# 69. Privacy Caveat

Cross-source dedup can reveal linkage.

---

# 70. Strict Mode

Dedup only local.

---

# 71. No Server-Wide Cross-User Dedup Graph

Hard rule.

---

# 72. Public Discovery

Lists public/listed content.

---

# 73. Discovery Inputs

```text
public channels
listed communities
public publishers
public posts
```

---

# 74. No Private Content Leakage

Hard rule.

---

# 75. Discovery Query

```rust
pub struct DiscoveryQuery {
    pub text: Option<DiscoveryText>,
    pub topic: Option<TopicId>,
    pub scope: DiscoveryScope,
}
```

---

# 76. Discovery Scope

```rust
pub enum DiscoveryScope {
    Local,
    Tenant,
    Federation,
    PublicNetwork,
}
```

---

# 77. Search Privacy

No account identity required for public search.

---

# 78. Search Query Logging

Minimized.

---

# 79. No Long-Term Search Profile

Hard rule.

---

# 80. Public Search Result

```rust
pub struct DiscoveryResult {
    pub content_ref: ContentRef,
    pub source: FeedSource,
    pub title: Option<ContentTitle>,
    pub summary: Option<ContentSummary>,
}
```

---

# 81. No Subscriber Identity.

---

# 82. Recommendation Sources

```rust
pub enum RecommendationSource {
    ExplicitTopics,
    FollowedSources,
    LocalHistory,
    PublicTrending,
    CuratedList,
}
```

---

# 83. ExplicitTopics

Preferred.

---

# 84. FollowedSources

Safe.

---

# 85. LocalHistory

On-device only.

---

# 86. PublicTrending

Aggregate public signal.

---

# 87. CuratedList

Human/editorial.

---

# 88. No Cross-User Behavioral Collaborative Filtering By Default

Hard rule.

---

# 89. Why

It requires graphing user behavior.

---

# 90. Future Optional Private Recommendation

Could use privacy-preserving aggregation.

---

# 91. But

Must be reviewed separately.

---

# 92. No Custom Federated Learning Crypto

Hard rule.

---

# 93. Trending

Aggregate public signal.

---

# 94. Trend Inputs

Possible:

```text
public views
public reshares
public reactions
public subscriptions
```

---

# 95. Aggregate Only

---

# 96. No Individual Viewer Tracking

Hard rule.

---

# 97. Trend Window

Bounded.

---

# 98. Example

```rust
pub struct TrendWindow {
    pub duration: Duration,
    pub minimum_count: u64,
}
```

---

# 99. Minimum Count

Protect privacy.

---

# 100. Small cohort

Do not surface.

---

# 101. Hard Rule

Trend output must satisfy minimum anonymity bucket.

---

# 102. Exact Counts

May be hidden.

---

# 103. Trend Score

Coarse.

---

# 104. Trend Class

```rust
pub enum TrendClass {
    Emerging,
    Popular,
    VeryPopular,
}
```

---

# 105. No Real-Time Per-User Trend Dashboard

Hard rule.

---

# 106. Trend Computation

Can be server aggregate.

---

# 107. Input events

Stripped of user identity where possible.

---

# 108. No Global User Identifier.

---

# 109. Abuse

Trend manipulation likely.

---

# 110. Need anti-spam/bot weighting.

---

# 111. But

Do not require real-world identity.

---

# 112. Anti-Manipulation Inputs

```text
rate limits
proof cost
account age bucket
source diversity
```

---

# 113. Privacy-safe.

---

# 114. No Public Reputation Score.

---

# 115. Hard rule.

---

# 116. Creator Privacy

Creators can publish through:

```text
public pseudonym
space-scoped identity
managed identity
```

---

# 117. No global real-name requirement.

---

# 118. Publisher Ref

```rust
pub enum PublisherRef {
    Pseudonym(PublisherPseudonym),
    SocialSpace(SocialSpaceId),
    Managed(ManagedPublisherId),
}
```

---

# 119. Public Publisher Pseudonym

Stable by choice.

---

# 120. Can rotate with migration policy.

---

# 121. Subscriber should not automatically learn account/login identity.

---

# 122. Hard rule.

---

# 123. Feed Subscription Delivery

Possible:

```text
pull
push hint
mailbox fanout
pub/sub
```

---

# 124. Pull

Privacy-friendly.

---

# 125. Push Hint

Opaque wake hint.

---

# 126. Mailbox Fanout

Encrypted.

---

# 127. Pub/Sub

Public content.

---

# 128. No Push Payload With Sensitive Feed Details By Default

Hard rule.

---

# 129. Timeline Synchronization

Across own devices.

---

# 130. Sync state:

```text
subscriptions
mute state
pinned sources
read position optionally
```

---

# 131. Read Position

Privacy-sensitive.

---

# 132. Default

DeviceLocal.

---

# 133. User may enable encrypted sync.

---

# 134. Hard Rule

Read history is not cloud-synced by default.

---

# 135. Feed Read State

```rust
pub enum FeedReadStateSync {
    LocalOnly,
    EncryptedAccountSync,
}
```

---

# 136. Creator Read Receipts

Disabled by default.

---

# 137. No Per-Subscriber View Receipts

Hard rule.

---

# 138. Public View Counts

Aggregate.

---

# 139. Minimum threshold.

---

# 140. No individual viewer list.

---

# 141. Reactions

Can be:

```text
private
aggregate
public
```

---

# 142. Public reaction identity

Explicit opt-in.

---

# 143. Default public feed

Aggregate reactions.

---

# 144. No list of reactors by default.

---

# 145. Hard rule.

---

# 146. Reshare

Explicit action.

---

# 147. Reshare Graph

Potential social graph.

---

# 148. Do not globally expose chain by default.

---

# 149. Hard rule.

---

# 150. Attribution

Can show original content source.

---

# 151. No full reshare genealogy.

---

# 152. Feed Privacy Modes

```rust
pub enum FeedPrivacyMode {
    Standard,
    Private,
    MaximumAnonymity,
}
```

---

# 153. Standard

Normal subscriptions, local ranking.

---

# 154. Private

Minimal remote analytics.

---

# 155. MaximumAnonymity

Chronological/local feeds only, delayed sync, no behavioral personalization.

---

# 156. Hard Rule

Privacy mode never silently downgrades for better recommendations.

---

# 157. Local Recommendation Engine

Optional.

---

# 158. Inputs

```text
explicit topics
local subscriptions
local saved items
local interaction history
```

---

# 159. Outputs

Candidate ranking.

---

# 160. No Cloud Model Requirement

Hard rule.

---

# 161. On-Device Model

Optional.

---

# 162. Model Data

Local.

---

# 163. No raw prompt/content telemetry.

---

# 164. Model Update

Signed artifact.

---

# 165. No per-user model upload.

---

# 166. Hard rule.

---

# 167. Recommendation Store

Derived/rebuildable.

---

# 168. Not authoritative user state.

---

# 169. User Can Reset

Required.

---

# 170. Reset Deletes Local recommendation state.

---

# 171. Feed Filters

```rust
pub struct TimelineFilters {
    pub topics: BTreeSet<TopicId>,
    pub muted_sources: BTreeSet<FeedSourceRef>,
    pub content_types: BTreeSet<ContentType>,
}
```

---

# 172. Filter State

Local or account-synced.

---

# 173. No filter telemetry.

---

# 174. Feed Item Content Types

```rust
pub enum ContentType {
    Text,
    Image,
    Video,
    Audio,
    Link,
    Announcement,
    Article,
}
```

---

# 175. Media Autoplay

DeviceLocal preference.

---

# 176. External Links

Privacy-sensitive.

---

# 177. No Automatic External Fetch In Max-Anonymity

Hard rule.

---

# 178. Link Preview

Use local/proxy according to Part 84 privacy setting.

---

# 179. Feed Preloading

Performance vs privacy.

---

# 180. Default

Bounded.

---

# 181. No Preload Hundreds Of Unseen Items

Hard rule.

---

# 182. Why

Leaks interest and wastes bandwidth.

---

# 183. Preload Policy

```rust
pub struct FeedPreloadPolicy {
    pub max_items: u16,
    pub media_prefetch: bool,
}
```

---

# 184. Strict Mode

No media prefetch by default.

---

# 185. Offline Feed

Important.

---

# 186. Cache recent subscribed content.

---

# 187. Cache policy

Bounded.

---

# 188. User controls.

---

# 189. Public discovery cache

Separate.

---

# 190. No infinite feed history.

---

# 191. Hard rule.

---

# 192. Feed Item Retention

Depends on source.

---

# 193. Public channel

Can archive.

---

# 194. Private group

Membership/history policy.

---

# 195. Local read cache

Device-controlled.

---

# 196. Deletion

Removed item tombstone if required.

---

# 197. No resurrect via stale cache.

---

# 198. Hard rule.

---

# 199. Feed Item Lifecycle

```rust
pub enum FeedItemState {
    Published,
    Updated,
    Retracted,
    Deleted,
}
```

---

# 200. Retraction

Source indicates no longer active.

---

# 201. Client cache

honors.

---

# 202. Public Archive

May preserve if policy/legal says.

---

# 203. Truthful distinction:

```text
removed from feed
vs
globally erased
```

---

# 204. Hard rule.

---

# 205. Content Update

Versioned.

---

# 206. Feed Content Version

```rust
pub struct FeedContentVersion(pub u64);
```

---

# 207. Old stale update

Rejected.

---

# 208. No rollback.

---

# 209. Hard rule.

---

# 210. Feed Cursor

Source-scoped.

---

# 211. No global timeline clock required.

---

# 212. Chronological Merge

Merge multiple source cursors locally.

---

# 213. Hard Rule

No centralized ordered log of every user's feed.

---

# 214. Feed Merge Algorithm

K-way merge.

---

# 215. Local.

---

# 216. Federation

Remote feeds.

---

# 217. Federation gateway fetches public/federated content.

---

# 218. No global feed authority.

---

# 219. Hard rule.

---

# 220. Remote Publisher Identity

Domain scoped.

---

# 221. Cross-Domain Content Ref

```rust
pub struct FederatedContentRef {
    pub domain: FederationDomainId,
    pub content: RemoteContentId,
}
```

---

# 222. No account identity merge.

---

# 223. Federated Subscription

Bilateral.

---

# 224. Remote domain need not know final subscriber identity where pull model used.

---

# 225. Hard rule.

---

# 226. Federation Caching

Public content can be cached.

---

# 227. Private/member content

capability-controlled.

---

# 228. No federation gateway plaintext for E2EE group content.

---

# 229. Hard rule.

---

# 230. Federation Recommendation

Local only.

---

# 231. Remote domain provides metadata.

---

# 232. User client ranks.

---

# 233. No centralized cross-domain behavioral profile.

---

# 234. Hard rule.

---

# 235. Tenant/Managed Feed

Managed organization can publish announcements.

---

# 236. Managed feed appears in managed profile.

---

# 237. Personal feed remains separate.

---

# 238. Hard rule.

---

# 239. Tenant cannot inspect unrelated personal subscriptions.

---

# 240. Managed ranking

Org policy can pin critical announcements.

---

# 241. But

Only within managed workspace/profile.

---

# 242. No global takeover of personal feed.

---

# 243. Hard rule.

---

# 244. Critical Announcement

Higher priority.

---

# 245. Emergency broadcasts Part 41/17 style.

---

# 246. Still scoped.

---

# 247. Content Moderation

Part 86.

---

# 248. Removed public content excluded from discovery.

---

# 249. Private content moderation remains space-scoped.

---

# 250. No hidden platform-wide censorship state in local private feeds without policy authority.

---

# 251. Ranking & Moderation Separation

Hard rule.

---

# 252. Ranking should not silently shadow-ban content.

---

# 253. Moderation state explicit.

---

# 254. Ranking state explicit.

---

# 255. Visibility State

```rust
pub enum FeedVisibilityDecision {
    Eligible,
    HiddenByUser,
    HiddenByPolicy,
    RemovedBySource,
    RemovedByModeration,
}
```

---

# 256. Explainable.

---

# 257. User should distinguish:

```text
muted
moderated
deleted
unavailable
```

---

# 258. Hard rule.

---

# 259. Anti-Manipulation

Threats:

```text
bot farms
fake engagement
coordinated boosting
spam publishers
```

---

# 260. Responses

```text
rate limits
minimum trend bucket
source diversity
proof cost
manual curation
```

---

# 261. No invasive device fingerprinting requirement.

---

# 262. Hard rule.

---

# 263. Publisher Abuse

Can be blocked/muted locally.

---

# 264. Source Block

```rust
pub enum FeedSourceState {
    Active,
    Muted,
    Blocked,
}
```

---

# 265. Blocked

No feed candidates.

---

# 266. Stale sync cannot unblock.

---

# 267. Hard rule.

---

# 268. Content Safety

External media/link fetch.

---

# 269. Sandboxed rendering.

---

# 270. No active HTML/JS execution in feed.

---

# 271. Hard rule.

---

# 272. Markdown/structured content

Sanitize.

---

# 273. Images/video

Decode safely.

---

# 274. Link Preview

Privacy policy.

---

# 275. Content Metadata

Bounded.

---

# 276. No arbitrary gigantic metadata.

---

# 277. Feed Storage

Local:

```text
subscriptions
timeline cache
read state
local recommendation state
```

---

# 278. Server:

```text
public content index
source metadata
aggregate trend counters
```

---

# 279. Server Must Not Store

```text
global per-user read timeline
behavioral interest vector
subscriber edge graph
```

---

# 280. Hard rule.

---

# 281. Search

Part 32 integration.

---

# 282. Public feed search

server/public index allowed.

---

# 283. Private feed search

local.

---

# 284. No private content indexing centrally.

---

# 285. Hard rule.

---

# 286. Saved Items

Local/account encrypted.

---

# 287. Save action not public.

---

# 288. Creator not notified.

---

# 289. Hard rule.

---

# 290. Read Later

Same.

---

# 291. Bookmark Sync

Encrypted account state.

---

# 292. No bookmark analytics.

---

# 293. Hard rule.

---

# 294. Feed Notification

Separate.

---

# 295. Push notification uses opaque hint.

---

# 296. Content fetched after wake.

---

# 297. No full sensitive feed item in push payload.

---

# 298. Hard rule.

---

# 299. Feed Refresh

Pull-based.

---

# 300. Refresh Interval

Battery/network aware.

---

# 301. No high-frequency polling for dormant feeds.

---

# 302. Strict Mode

More batched.

---

# 303. Timeline Pagination

Bounded page size.

---

# 304. No unbounded infinite page response.

---

# 305. Hard rule.

---

# 306. Timeline Cursor Privacy

Opaque, non-user-identifying.

---

# 307. Cursor expiry.

---

# 308. No permanent cursor correlation token.

---

# 309. Hard rule.

---

# 310. Feed Sync Across Devices

Sync:

```text
subscriptions
mutes
pins
saved items
optional read position
```

---

# 311. Not sync:

```text
scroll velocity
dwell time
local ephemeral ranking history
```

---

# 312. Hard rule.

---

# 313. Feed Conflict Policy

Subscriptions:

set union/removal tombstone.

---

# 314. Block/mute:

restrictive wins where applicable.

---

# 315. Pin order:

user merge/manual or deterministic.

---

# 316. Read state:

device-local unless opted in.

---

# 317. No server LWW for security/privacy block state.

---

# 318. Hard rule.

---

# 319. Timeline Repository

```rust
pub trait TimelineRepository {
    fn page(
        &self,
        state: &TimelineState,
        limit: usize,
    ) -> Result<TimelinePage, FeedError>;
}
```

---

# 320. Subscription Repository

```rust
pub trait SubscriptionRepository {
    fn subscribe(
        &self,
        source: FeedSource,
    ) -> Result<SubscriptionId, FeedError>;

    fn unsubscribe(
        &self,
        id: SubscriptionId,
    ) -> Result<(), FeedError>;
}
```

---

# 321. Candidate Provider

```rust
pub trait FeedCandidateProvider {
    fn candidates(
        &self,
        context: &FeedContext,
    ) -> Result<Vec<FeedCandidate>, FeedError>;
}
```

---

# 322. Ranker

```rust
pub trait FeedRanker {
    fn rank(
        &self,
        candidates: &mut [FeedCandidate],
        mode: RankingMode,
    ) -> Result<(), FeedError>;
}
```

---

# 323. Public Discovery Service

```rust
pub trait PublicDiscoveryService {
    fn search(
        &self,
        query: DiscoveryQuery,
    ) -> Result<Vec<DiscoveryResult>, FeedError>;
}
```

---

# 324. Trend Aggregator

```rust
pub trait TrendAggregator {
    fn aggregate(
        &self,
        window: TrendWindow,
    ) -> Result<Vec<TrendSignal>, FeedError>;
}
```

---

# 325. No User-Level Trend API

Hard rule.

---

# 326. Ranking Policy

```rust
pub struct FeedRankingPolicy {
    pub allowed_modes: BTreeSet<RankingMode>,
    pub default_mode: RankingMode,
    pub allow_behavioral_local: bool,
}
```

---

# 327. Server cannot override user's chronological choice.

---

# 328. Hard rule.

---

# 329. Recommendation Policy

```rust
pub struct RecommendationPolicy {
    pub enabled: bool,
    pub allowed_sources: BTreeSet<RecommendationSource>,
    pub local_only_behavioral: bool,
}
```

---

# 330. Default

ExplicitTopics + FollowedSources + PublicTrending.

---

# 331. LocalHistory

Opt-in.

---

# 332. No Cloud Behavioral Model.

---

# 333. Hard rule.

---

# 334. Feed Explanation

```rust
pub enum RecommendationReason {
    FollowedSource,
    ExplicitTopic,
    PublicTrending,
    Curated,
    LocalHistory,
}
```

---

# 335. Every recommended item can expose one reason.

---

# 336. No hidden reason category.

---

# 337. Feed Control UX

User should have:

```text
Chronological
Subscriptions only
Recommendations on/off
Reset local personalization
Mute/block source
```

---

# 338. Hard rule.

---

# 339. No Dark Pattern Against Chronological Mode

Hard rule.

---

# 340. Feed Quality

Optimize:

```text
relevance
freshness
source diversity
user intent
```

---

# 341. Not optimize solely:

```text
session length
screen time
compulsive engagement
```

---

# 342. Hard rule.

---

# 343. Source Diversity

Avoid one publisher dominating.

---

# 344. Local diversity rule.

---

# 345. Example

max N consecutive items from same source.

---

# 346. No hidden political/content viewpoint balancing requirement.

---

# 347. Just source diversity.

---

# 348. Discovery Safety

Public search can surface malicious content.

---

# 349. Moderation status.

---

# 350. User filters.

---

# 351. Sensitive content labels optional.

---

# 352. No invasive identity inference.

---

# 353. Hard rule.

---

# 354. Content Classification

Can be publisher-declared/moderator-applied.

---

# 355. No centralized psychographic classification of user.

---

# 356. Hard rule.

---

# 357. Creator Analytics

Privacy-sensitive.

---

# 358. Allowed aggregate metrics:

```text
public view bucket
public reaction count
subscriber count bucket
```

---

# 359. Not allowed by default:

```text
viewer identity
exact read timestamp
viewer location
contact overlap
```

---

# 360. Hard rule.

---

# 361. Creator Insights

Thresholded.

---

# 362. Minimum cohort.

---

# 363. No small audience deanonymization.

---

# 364. Hard rule.

---

# 365. Public Metrics

Coarsened.

---

# 366. Example subscriber count:

```text
100+
1K+
10K+
```

---

# 367. Strict mode can hide.

---

# 368. Feed Telemetry

Safe:

```text
refresh success
candidate fetch latency
ranking latency
trend pipeline health
```

---

# 369. Forbidden:

```text
per-user read sequence
dwell-time profile
per-user interest vector
```

---

# 370. Hard rule.

---

# 371. Operational Logs

Source/service level.

---

# 372. No user content interaction history.

---

# 373. Feed SLO

Examples:

```text
timeline load latency
subscription update propagation
public discovery availability
```

---

# 374. Privacy SLO

```text
0 cloud behavioral profiles
0 subscriber identity export
0 viewer identity export
```

---

# 375. Security SLO

```text
0 private content appearing in public discovery
```

---

# 376. Failure Modes

```text
feed source outage
directory outage
trend service outage
ranking model failure
federation outage
```

---

# 377. Source Outage

Other sources continue.

---

# 378. Directory Outage

Cached subscriptions continue.

---

# 379. Trend Service Outage

Trending section disappears.

---

# 380. No fallback to invasive ranking.

---

# 381. Hard rule.

---

# 382. Ranking Failure

Fallback:

```text
chronological
```

---

# 383. Hard rule.

---

# 384. Recommendation Engine Failure

Disable recommendations.

---

# 385. Do not block subscription feed.

---

# 386. Hard rule.

---

# 387. Federation Outage

Local/federated cached feed continues within validity.

---

# 388. No identity-leaking direct fallback.

---

# 389. Hard rule.

---

# 390. Account Lifecycle

Part 83.

---

# 391. Deleted/deactivated account subscriptions handled per policy.

---

# 392. Deleted account cannot continue cloud feed sync.

---

# 393. Local cached public content may remain until local cleanup.

---

# 394. Profile Integration

Part 84.

---

# 395. Explicit topic preferences may sync encrypted.

---

# 396. Local behavioral preference remains local.

---

# 397. Contact Integration

Part 85.

---

# 398. Contacts may be feed sources.

---

# 399. Contact relationship does not imply follow/subscription.

---

# 400. Hard rule.

---

# 401. Social Space Integration

Part 86.

---

# 402. Joined space can auto-subscribe only if space policy/user setting says.

---

# 403. User can mute.

---

# 404. Group membership and feed subscription remain separate.

---

# 405. Hard rule.

---

# 406. Authorization Integration

Part 81.

---

# 407. Candidate eligibility checked before ranking.

---

# 408. Ranking cannot grant access.

---

# 409. Hard rule.

---

# 410. Authentication Integration

Part 82.

---

# 411. Public discovery may be unauthenticated.

---

# 412. Managed feed requires org auth.

---

# 413. Feed token not auth token.

---

# 414. Hard rule.

---

# 415. Edge Integration

Part 78.

---

# 416. Public feed APIs L7.

---

# 417. Private content remains E2EE.

---

# 418. WAF does not inspect encrypted private content.

---

# 419. Internal Service Integration

Part 79.

---

# 420. Feed services use workload identity/mTLS.

---

# 421. Secrets Integration

Part 80.

---

# 422. Publisher signing keys scoped.

---

# 423. No global feed signing secret.

---

# 424. Event Bus Integration

Part 75.

---

# 425. Publication events may feed public index/trend aggregation.

---

# 426. No user interaction event stream.

---

# 427. Hard rule.

---

# 428. Database Integration

Part 74.

---

# 429. Public content index separate from private subscription state.

---

# 430. No central table:

```text
user_id, viewed_item_id, dwell_time
```

---

# 431. Hard rule.

---

# 432. Storage Model

Client:

```text
subscriptions
timeline cache
local ranking state
saved items
optional read state
```

Server:

```text
public content
public discovery metadata
aggregate trends
```

---

# 433. No Global Behavioral Store.

---

# 434. Search Integration

Part 32.

---

# 435. Private feed search local.

---

# 436. Public feed search public index.

---

# 437. No private index upload.

---

# 438. Hard rule.

---

# 439. Content Portability

Export subscriptions/saved items.

---

# 440. Exclude behavioral state by default.

---

# 441. Recommendation model state optional local export.

---

# 442. No hidden cloud profile to export.

---

# 443. Hard rule.

---

# 444. Backup

Subscriptions and explicit preferences can be backed up.

---

# 445. Local behavior model optional/rebuildable.

---

# 446. Read state optional.

---

# 447. No server restoring revoked/deleted source.

---

# 448. Hard rule.

---

# 449. Deletion

Source delete/retract propagated.

---

# 450. Cached copies may persist according to retention/public archive.

---

# 451. No claim universal erasure.

---

# 452. Hard truth.

---

# 453. Trend Privacy

Aggregate counters retention short.

---

# 454. No indefinite raw trend event history.

---

# 455. Hard rule.

---

# 456. Trend Input Event

```rust
pub struct PublicTrendEvent {
    pub content: PublicContentRef,
    pub event_class: PublicTrendEventClass,
    pub coarse_epoch: TrendEpoch,
}
```

---

# 457. No user ID.

---

# 458. Trend Event Class

```rust
pub enum PublicTrendEventClass {
    View,
    Reaction,
    Reshare,
    Subscribe,
}
```

---

# 459. Dedup Abuse

Need approximate anonymous controls.

---

# 460. Could use scoped rate token/ephemeral source bucket.

---

# 461. No permanent identity.

---

# 462. Hard rule.

---

# 463. Trend Aggregation Windows

```text
hourly
daily
weekly
```

---

# 464. No second-by-second public user behavior.

---

# 465. Hard rule.

---

# 466. Curated Lists

Human/admin curated.

---

# 467. Curator identity scoped.

---

# 468. Transparency.

---

# 469. No hidden sponsored insertion without explicit label if product adds monetization later.

---

# 470. Feed Policy separates organic/curated.

---

# 471. Monetization Not Core

Part 54 economics separate.

---

# 472. Crate Layout

Recommended:

```text
crates/
├── siar-feed-core/
├── siar-subscription/
├── siar-timeline/
├── siar-feed-candidate/
├── siar-feed-ranking/
├── siar-feed-recommendation/
├── siar-public-discovery/
├── siar-trending/
├── siar-feed-federation/
├── siar-feed-observability/
└── siar-feed-testkit/
```

---

# 473. `siar-feed-core`

Owns:

```text
feed IDs
modes
errors
```

---

# 474. `siar-subscription`

Subscription state/capabilities.

---

# 475. `siar-timeline`

Cursor/pagination/local merge.

---

# 476. `siar-feed-candidate`

Candidate eligibility.

---

# 477. `siar-feed-ranking`

Chronological/local/curated rankers.

---

# 478. `siar-feed-recommendation`

Local explicit-topic recommendation.

---

# 479. `siar-public-discovery`

Public search/listing.

---

# 480. `siar-trending`

Thresholded aggregate trend signals.

---

# 481. `siar-feed-federation`

Remote feed ingestion.

---

# 482. `siar-feed-observability`

Aggregate pipeline metrics only.

---

# 483. `siar-feed-testkit`

ranking/privacy/federation/trend tests.

---

# 484. Error Taxonomy

```rust
pub enum FeedError {
    SourceNotFound,
    SubscriptionDenied,
    ContentUnavailable,
    ContentNotAuthorized,
    CursorExpired,
    RankingUnavailable,
    RecommendationDisabled,
    DiscoveryDenied,
    FederationUnavailable,
    SourceBlocked,
    Internal,
}
```

---

# 485. Testing

Need feed/discovery testkit.

---

# 486. Test Scenarios

```text
chronological mode
subscription mode
local recommendation
trend aggregation
source block
```

---

# 487. Chronological Test

Order deterministic.

---

# 488. Ranking Failure Test

Fallback chronological.

---

# 489. Recommendation Disabled Test

No recommended candidates.

---

# 490. Local History Test

Signal never leaves device.

---

# 491. Subscription Privacy Test

Creator cannot enumerate subscribers.

---

# 492. Read-State Test

Not synced unless opt-in.

---

# 493. Viewer Privacy Test

No individual viewer list.

---

# 494. Trending Test

Minimum bucket required.

---

# 495. Manipulation Test

Rate limits/source diversity.

---

# 496. Private Content Test

Never enters public discovery.

---

# 497. Moderation Test

Removed content excluded but moderation != ranking.

---

# 498. Contact Test

Contact relationship does not auto-follow.

---

# 499. Group Test

Group membership and feed subscription separate.

---

# 500. Federation Test

Remote domain receives no subscriber identity.

---

# 501. Managed Feed Test

Org pin affects managed profile only.

---

# 502. Deletion Test

Retracted/deleted source not resurrected by stale cache.

---

# 503. Telemetry Test

No per-user interest vector/log.

---

# 504. Fuzzing

Fuzz:

```text
feed cursor
discovery result
subscription envelope
trend event
federated feed envelope
```

---

# 505. Property Tests

Properties:

```text
unauthorized candidate can never enter ranked timeline
chronological mode never invokes behavioral ranker
disabled recommendation produces no recommendation items
private content never appears in public discovery
```

---

# 506. Formal Verification Targets

Strong candidates:

```text
candidate eligibility before ranking
subscription state
trend threshold privacy
federated content scope
```

---

# 507. Kani Candidate

ranking eligibility/filter order.

---

# 508. TLA+ Candidate

publication/retraction/cache/federation propagation.

---

# 509. Loom Candidate

concurrent subscription update + timeline refresh + source block.

---

# 510. Performance

Timeline hot path.

---

# 511. Candidate set bounded.

---

# 512. Ranking local/in-memory.

---

# 513. No remote ML inference.

---

# 514. Pagination.

---

# 515. Media lazy load.

---

# 516. Public search indexed.

---

# 517. Trend aggregate async.

---

# 518. No per-scroll server event.

---

# 519. Hard rule.

---

# 520. Timeline Cache

Bounded by:

```text
item count
bytes
age
```

---

# 521. Eviction local.

---

# 522. No infinite scroll storage.

---

# 523. Content blobs separate.

---

# 524. Previews bounded.

---

# 525. Source Fairness

One source cannot monopolize queue/cache.

---

# 526. Local scheduling.

---

# 527. No server per-user fairness profile.

---

# 528. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Chronological/subscription-only feed modes remain available and cannot be silently replaced by opaque behavioral ranking.
2. Private/member-scoped content is filtered for authorization before ranking and can never appear in public discovery.
3. User behavioral signals, read sequences, dwell time, scroll behavior, and local interest models remain local by default and are not uploaded as centralized behavioral profiles.
4. Subscriptions do not expose subscriber identities to creators or public APIs by default.
5. Public trending and creator analytics use thresholded aggregate signals without individual viewer/subscriber identities.
6. Ranking decisions are explainable through explicit reasons such as follow, topic, curation, or public trend.
7. Recommendation systems are optional, resettable, local-first, and never required for basic feed usability.
8. Managed/enterprise feeds affect only managed profiles and cannot inspect or reorder unrelated personal feeds by default.
9. Federation transports public/federated content and capabilities without creating a cross-domain user-interest or subscriber graph.
10. Ranking state, moderation state, and authorization state remain separate; ranking cannot grant access or silently substitute for moderation.
11. Feed telemetry contains pipeline/service metrics only and never becomes a per-user interest vector, view history, or engagement timeline.
12. Deleted/retracted/blocked content and sources cannot be resurrected into active timelines by stale caches, backups, or federated replicas.
```

---

# 529. Initial Production Scope

Implement first:

```text
typed FeedSource/FeedItem/Subscription
chronological feed
subscription-only feed
local merge/pagination
mute/block/pin controls
explicit topic preferences
local heuristic ranking
recommendations opt-in
public discovery
aggregate thresholded trending
creator subscriber-count buckets
local-only read state by default
encrypted subscription sync
federated public-feed ingestion
managed feed isolation
privacy-safe metrics
feed testkit
```

Then add:

```text
on-device recommendation model
privacy-preserving aggregate recommendation research
cross-provider feed portability
advanced public trend robustness
local diversity-aware ranking
formal feed/privacy verification
```

---

# 530. Definition of Done

Part 87 is complete when:

- subscription/chronological feeds work without recommendation infrastructure
- feed candidates are authorized before ranking
- ranking modes are explicit and user-controlled
- behavioral personalization stays local by default
- subscriptions do not reveal subscriber identities
- public analytics/trending use minimum aggregate thresholds
- creator insights do not expose viewer identities
- private/public feed search boundaries are explicit
- managed and personal feeds remain separate
- federation does not create cross-domain interest graphs
- stale caches cannot resurrect removed/deleted content
- telemetry excludes per-user engagement profiles
- ranking/privacy/trend/federation/fuzz/formal tests are specified

---

# 531. Final Architecture

```text
            SUBSCRIPTIONS / PUBLIC SOURCES
                         │
                         ▼
                 CANDIDATE COLLECTION
                         │
                         ▼
               AUTHORIZATION / PRIVACY
                         │
                         ▼
            CHRONOLOGICAL / LOCAL RANKING
                         │
                         ▼
                      TIMELINE
                         │
                ┌────────┼────────┐
                │        │        │
              SAVE      MUTE    SUBSCRIBE
                │        │        │
                └────────┼────────┘
                         ▼
                  LOCAL USER STATE
```

Feed safety model:

```text
explicit subscriptions
+
authorization before ranking
+
chronological fallback
+
local personalization
+
thresholded public trends
+
subscriber/viewer privacy
+
no behavioral cloud profile
```

not:

```text
record every interaction, build a hidden psychographic profile, and rank content to maximize time-on-screen
```

---

# 532. Final Principle

A feed should help users organize and discover content according to explicit intent without requiring the platform to know what keeps each person scrolling.

The correct model is:

```text
follow explicitly
+
rank locally
+
recommend optionally
+
aggregate publicly only when privacy-safe
+
keep subscriber/viewer identities hidden
+
separate moderation from ranking
+
never centralize behavioral profiles
```

This architecture gives SIAR a privacy-preserving foundation for timelines, subscriptions, recommendations, trending, discovery, public channels, creators, federated feeds, and managed announcements while preserving the anonymity, local-first, least-authority, and social-graph privacy guarantees established across Parts 34–86.
