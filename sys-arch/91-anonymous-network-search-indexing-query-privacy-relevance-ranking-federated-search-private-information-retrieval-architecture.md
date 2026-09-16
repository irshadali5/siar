# Core System Architecture Part 91 — Anonymous Network Search, Indexing, Query Privacy, Relevance Ranking, Federated Search & Private Information Retrieval Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 91  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 32, 42, 45, 48, 51, 56, 58–60, 67–69, 74–90  

**Primary purpose:** define SIAR's search and indexing architecture across local/private/public/federated domains, including query privacy, result authorization, relevance ranking, anti-enumeration, private information retrieval, encrypted local indexing, public search projections, federated search fanout, deletion/index consistency, metadata minimization, result provenance, and privacy-safe observability.

---

# 1. Purpose

Search is one of the easiest subsystems to turn into a surveillance layer.

A naive search system can reveal:

```text
what users are interested in
who users are looking for
which communities they search
what private message terms they query
which results they click
```

The governing principle is:

> **SIAR search should minimize what the server learns about user intent, keep private-content search local by default, and treat authorization/privacy filtering as a prerequisite to ranking rather than a post-processing step.**

---

# 2. Architectural Position

```text
Query
  │
  ▼
Scope / Privacy Classification
  │
  ├── Local Private Index
  ├── Public Search Service
  ├── Tenant Search
  └── Federated Search
  │
  ▼
Authorization / Visibility Filter
  │
  ▼
Relevance Ranking
  │
  ▼
Results + Provenance
```

---

# 3. Core Separation

Keep distinct:

```text
query
index
search corpus
authorization filter
ranking
result projection
click/open state
analytics
```

---

# 4. Non-Goals

Part 91 does not create:

```text
a universal global index of all private content
centralized logging of every search query
search ranking based on hidden behavioral profiles
server-side indexing of E2EE private messages by default
```

---

# 5. Search Domains

```rust
pub enum SearchDomain {
    LocalPrivate,
    Relationship,
    SocialSpace,
    ManagedTenant,
    PublicNetwork,
    Federation,
}
```

---

# 6. LocalPrivate

Private messages, drafts, local files, cached content.

---

# 7. Relationship

Relationship-scoped shared content.

---

# 8. SocialSpace

Group/community/channel content.

---

# 9. ManagedTenant

Organization-owned corpus.

---

# 10. PublicNetwork

Public/listed content only.

---

# 11. Federation

Cross-domain public/federated content.

---

# 12. Hard Rule

Every query has an explicit search domain.

---

# 13. Query Identity

```rust
pub struct SearchQueryId(pub [u8; 32]);
```

---

# 14. Query ID Scope

Short-lived/local.

---

# 15. No Stable User Identity In Query ID

Hard rule.

---

# 16. Search Query

```rust
pub struct SearchQuery {
    pub id: SearchQueryId,
    pub domain: SearchDomain,
    pub terms: SearchTerms,
    pub filters: SearchFilters,
}
```

---

# 17. Search Terms

Bounded.

---

# 18. No Unbounded Query Payload

Hard rule.

---

# 19. Query Classification

```rust
pub enum QueryPrivacyClass {
    LocalOnly,
    PrivateRemote,
    PublicRemote,
    FederationRemote,
}
```

---

# 20. LocalOnly

Never leaves device.

---

# 21. PrivateRemote

Sent only to authorized private service if unavoidable.

---

# 22. PublicRemote

Public discovery query.

---

# 23. FederationRemote

Cross-domain query.

---

# 24. Default For Private Content

LocalOnly.

---

# 25. Hard Rule

Private message/draft search is local unless user explicitly opts into an alternate reviewed architecture.

---

# 26. Local Search Index

Encrypted local index.

---

# 27. Index Types

```rust
pub enum LocalIndexType {
    InvertedText,
    Prefix,
    Trigram,
    Metadata,
    Vector,
}
```

---

# 28. Inverted Text

Primary full-text.

---

# 29. Prefix

Optional local autocomplete.

---

# 30. Trigram

Fuzzy matching.

---

# 31. Metadata

Date/source/type.

---

# 32. Vector

Optional local semantic search.

---

# 33. Vector Search

On-device only by default.

---

# 34. No Cloud Embedding Requirement

Hard rule.

---

# 35. Embeddings

Derived/rebuildable.

---

# 36. Not authoritative.

---

# 37. Embedding Deletion

Must follow source deletion.

---

# 38. Hard rule.

---

# 39. Local Index Storage

Encrypted embedded DB/index.

---

# 40. Derived index may rebuild from authoritative local data.

---

# 41. Hard Rule

Search index is never sole source of truth.

---

# 42. Search Index State

```rust
pub enum IndexState {
    Building,
    Ready,
    Stale,
    Rebuilding,
    Failed,
}
```

---

# 43. Stale

Search may operate with explicit stale marker where safe.

---

# 44. Deleted/revoked entries

Must be removed before stale index considered usable for protected data.

---

# 45. Hard rule.

---

# 46. Index Document

```rust
pub struct IndexDocument {
    pub doc_id: SearchDocumentId,
    pub scope: SearchDocumentScope,
    pub fields: SearchableFields,
}
```

---

# 47. Search Document ID

Opaque/local.

---

# 48. No raw AccountId.

---

# 49. Document Scope

```rust
pub enum SearchDocumentScope {
    Local,
    Relationship(RelationshipId),
    SocialSpace(SocialSpaceId),
    Tenant(TenantId),
    Public,
}
```

---

# 50. No Cross-Scope Merge By Default

Hard rule.

---

# 51. Public Search Projection

Separate from private content.

---

# 52. Public Search Record

```rust
pub struct PublicSearchRecord {
    pub content_ref: PublicContentRef,
    pub title: Option<ContentTitle>,
    pub summary: Option<ContentSummary>,
    pub topics: Vec<TopicId>,
    pub source: PublicSourceRef,
}
```

---

# 53. No Private Fields

Hard rule.

---

# 54. Projection Compiler

```rust
pub trait SearchProjectionCompiler {
    fn public_projection(
        &self,
        source: &PublishedContent,
    ) -> Result<Option<PublicSearchRecord>, SearchError>;
}
```

---

# 55. No Reflection-Based Export

Hard rule.

---

# 56. Index Eligibility

Private/listed/public policy checked before indexing.

---

# 57. Searchability State

```rust
pub enum Searchability {
    NotSearchable,
    ExactOnly,
    ScopedSearchable,
    PublicSearchable,
}
```

---

# 58. Private

NotSearchable remotely.

---

# 59. ExactOnly

Handle-like resolution.

---

# 60. ScopedSearchable

Authorized group/tenant.

---

# 61. PublicSearchable

Public index.

---

# 62. Hard Rule

Content visibility and searchability are separate states.

---

# 63. Public content can choose NotSearchable.

---

# 64. No Automatic Indexing Just Because Content Is Public

Hard rule.

---

# 65. Query Privacy

Remote search leaks intent.

---

# 66. Mitigations:

```text
query minimization
batching
proxying
PIR where practical
local cache
```

---

# 67. No Query Logging By Default

Hard rule.

---

# 68. Public Search Service

Should not require authenticated account identity.

---

# 69. Anonymous Search Token

Optional abuse-control credential.

---

# 70. Token not persistent user ID.

---

# 71. Hard rule.

---

# 72. Query Transport

Anonymous/private route according to privacy mode.

---

# 73. Strict mode

Can route public search over anonymous path.

---

# 74. No direct-IP fallback if user requires anonymity.

---

# 75. Hard rule.

---

# 76. Query Terms

Potentially sensitive.

---

# 77. Logs must not store raw terms by default.

---

# 78. Metrics

Use aggregate query count/latency.

---

# 79. No search-interest profile.

---

# 80. Hard rule.

---

# 81. Public Search Abuse Control

Need rate limits.

---

# 82. Options:

```text
anonymous quota token
proof cost
coarse source bucket
```

---

# 83. No Permanent IP Reputation

Hard rule.

---

# 84. Search Ranking

After visibility/authorization filter.

---

# 85. Hard Rule

Unauthorized result can never enter candidate ranking set.

---

# 86. Ranking Modes

```rust
pub enum SearchRankingMode {
    Relevance,
    Recency,
    LocalPreference,
    ExactMatch,
}
```

---

# 87. Relevance

Text/metadata score.

---

# 88. Recency

Ordered by publication time.

---

# 89. LocalPreference

On-device only.

---

# 90. ExactMatch

Identity/handle resolution.

---

# 91. No Hidden Server Behavioral Ranking Baseline

Hard rule.

---

# 92. Relevance Score

```rust
pub struct RelevanceScore {
    pub text: u16,
    pub recency: u16,
    pub explicit_priority: u16,
}
```

---

# 93. No psychographic score.

---

# 94. Local Personalization

Optional.

---

# 95. Inputs:

```text
explicit topics
local search history
local saved content
```

---

# 96. Local Search History

Device-local by default.

---

# 97. No Cloud Search-History Profile

Hard rule.

---

# 98. Search History

Optional.

---

# 99. User can disable.

---

# 100. User can clear.

---

# 101. Default retention bounded.

---

# 102. Hard rule.

---

# 103. Autocomplete

Potential privacy leak.

---

# 104. Local history autocomplete

local only.

---

# 105. Public suggestion service

query-prefix privacy risk.

---

# 106. Default

No remote prefix suggestions.

---

# 107. Hard rule.

---

# 108. If remote autocomplete later

requires reviewed privacy architecture.

---

# 109. Exact Handle Search

Part 59.

---

# 110. No fuzzy people search by default.

---

# 111. Hard rule.

---

# 112. Identity Search

Exact handle/capability only.

---

# 113. No email/phone enumeration.

---

# 114. Hard rule.

---

# 115. Social Space Search

Public/listed directories.

---

# 116. Private spaces absent.

---

# 117. Exact-only spaces exact resolution.

---

# 118. No hidden membership leak.

---

# 119. Hard rule.

---

# 120. Content Search Filters

```rust
pub struct SearchFilters {
    pub content_types: BTreeSet<ContentType>,
    pub sources: BTreeSet<SearchSourceFilter>,
    pub date_range: Option<SearchDateRange>,
    pub topics: BTreeSet<TopicId>,
}
```

---

# 121. Filters Explicit.

---

# 122. No hidden personalization filters.

---

# 123. Hard rule.

---

# 124. Private Search

Local index can search:

```text
messages
attachments metadata
drafts
saved items
contacts
```

---

# 125. Message Search

Local plaintext after decryption.

---

# 126. No server plaintext index.

---

# 127. Hard rule.

---

# 128. Attachment Search

Metadata local.

---

# 129. OCR/text extraction optional local.

---

# 130. No central extraction for private E2EE file unless user opts in.

---

# 131. Hard rule.

---

# 132. Semantic Search

On-device model.

---

# 133. Vector index encrypted/local.

---

# 134. No raw embeddings uploaded.

---

# 135. Hard rule.

---

# 136. Search Result

```rust
pub struct SearchResult {
    pub result_id: SearchResultId,
    pub source: SearchResultSource,
    pub projection: SearchResultProjection,
    pub provenance: SearchProvenance,
}
```

---

# 137. Result ID

Ephemeral.

---

# 138. Search Provenance

```rust
pub enum SearchProvenance {
    LocalIndex,
    PublicIndex,
    TenantIndex,
    FederationDomain(FederationDomainId),
}
```

---

# 139. User can inspect origin.

---

# 140. No spoofed provenance.

---

# 141. Hard rule.

---

# 142. Result Projection

Minimal.

---

# 143. Does not contain unauthorized hidden fields.

---

# 144. Result Opening

Reauthorizes resource.

---

# 145. Hard Rule

Search result is not authorization proof.

---

# 146. Stale Result

May point to deleted/revoked content.

---

# 147. Open fetch rechecks current state.

---

# 148. No stale-access bypass.

---

# 149. Hard rule.

---

# 150. Search Result Snippets

Private local

generated locally.

---

# 151. Public server

safe indexed field only.

---

# 152. No server snippet from E2EE private content.

---

# 153. Hard rule.

---

# 154. Snippet Highlight

Local.

---

# 155. Query terms not sent back into logs.

---

# 156. Federated Search

Queries multiple domains.

---

# 157. Federation Query

```rust
pub struct FederatedSearchQuery {
    pub request_id: FederatedSearchRequestId,
    pub terms: SearchTerms,
    pub scopes: Vec<FederationDomainId>,
}
```

---

# 158. No AccountId.

---

# 159. Domain Selection

Explicit/user policy.

---

# 160. No broadcast to every federation domain.

---

# 161. Hard rule.

---

# 162. Federation Fanout Bound

Limit domains per query.

---

# 163. Query Privacy

Remote domains learn query unless PIR/private protocol used.

---

# 164. UI should be truthful.

---

# 165. No false private-search claim.

---

# 166. Hard rule.

---

# 167. Federated Search Flow

```text
local query
→ choose domains
→ minimize query
→ send scoped requests
→ gather signed/scoped results
→ normalize locally
→ rank locally
```

---

# 168. Local Merge

Preferred.

---

# 169. Remote domains do not know final ranking.

---

# 170. Hard rule.

---

# 171. Cross-Domain Result

```rust
pub struct FederatedSearchResult {
    pub domain: FederationDomainId,
    pub remote_ref: RemoteContentId,
    pub projection: SearchResultProjection,
    pub signature: FederationSignature,
}
```

---

# 172. Trust Validation

Domain signature/descriptor.

---

# 173. No transitive trust.

---

# 174. Hard rule.

---

# 175. Federation Result Ranking

Local.

---

# 176. No global search authority.

---

# 177. Hard rule.

---

# 178. Federation Query Logging

Remote domain should minimize.

---

# 179. No persistent cross-domain user query profile.

---

# 180. Hard rule.

---

# 181. Private Information Retrieval

Optional advanced architecture.

---

# 182. PIR Goal

Retrieve information while hiding requested item/query from server.

---

# 183. Useful for:

```text
directory exact lookup
public index shard retrieval
small curated datasets
```

---

# 184. Not automatically practical for full web-scale semantic search.

---

# 185. Hard truth.

---

# 186. PIR Modes

```rust
pub enum PirMode {
    Disabled,
    ExactLookup,
    ShardFetch,
    Experimental,
}
```

---

# 187. ExactLookup

Most realistic initial option.

---

# 188. ShardFetch

Fetch larger bucket and search locally.

---

# 189. Experimental

Reviewed algorithms only.

---

# 190. No Custom PIR Crypto

Hard rule.

---

# 191. PIR Privacy Depends On

```text
server model
dataset size
query shape
timing
```

---

# 192. No absolute privacy claim.

---

# 193. Hard rule.

---

# 194. Shard Fetch

Useful simple privacy method.

---

# 195. Server sees shard, not exact item.

---

# 196. Client searches locally.

---

# 197. Tradeoff

More bandwidth.

---

# 198. Query Obfuscation By Dummy Queries

Not sufficient alone.

---

# 199. Hard rule.

---

# 200. Private Contact Discovery

Part 85.

---

# 201. Exact identity lookup may eventually use OPRF/PIR.

---

# 202. No custom protocol.

---

# 203. Search Encryption

Private local index encrypted.

---

# 204. Search Index Key

Device/account-scoped.

---

# 205. No global index key.

---

# 206. Hard rule.

---

# 207. Encrypted Server Search

Searchable encryption has leakage.

---

# 208. Not baseline.

---

# 209. If added

separate threat analysis.

---

# 210. Hard rule.

---

# 211. Public Index Architecture

Components:

```text
projection builder
index writer
query service
ranking service
tombstone processor
```

---

# 212. Public Index Data

Only public/searchable projection.

---

# 213. No private drafts/messages.

---

# 214. Hard rule.

---

# 215. Index Writer

Consumes committed publication events.

---

# 216. No search-before-commit.

---

# 217. Hard rule.

---

# 218. Index Update Idempotency

Required.

---

# 219. Same revision event

same index state.

---

# 220. No duplicate document.

---

# 221. Hard rule.

---

# 222. Index Revision

```rust
pub struct SearchIndexRevision(pub u64);
```

---

# 223. Content revision maps to index revision.

---

# 224. Stale older revision rejected.

---

# 225. No rollback.

---

# 226. Hard rule.

---

# 227. Retraction

Remove active result.

---

# 228. Deletion

Delete/tombstone.

---

# 229. Moderation

Mark unavailable/remove according to policy.

---

# 230. Visibility Change

Public→private removes index entry.

---

# 231. Hard rule.

---

# 232. Search Tombstone

```rust
pub struct SearchTombstone {
    pub content: ContentId,
    pub deletion_epoch: u64,
}
```

---

# 233. Stale index writer cannot re-add older revision.

---

# 234. Hard rule.

---

# 235. Reindexing

Must apply deletion/tombstone ledger.

---

# 236. No full rebuild resurrecting deleted content.

---

# 237. Hard rule.

---

# 238. Search Cache

Public result cache allowed.

---

# 239. Private search cache local only.

---

# 240. Cache TTL bounded.

---

# 241. Revocation/deletion invalidates.

---

# 242. Hard rule.

---

# 243. CDN

Public index shards/search static metadata can use CDN.

---

# 244. No private query-specific CDN key.

---

# 245. Strict privacy mode can avoid third-party CDN.

---

# 246. Hard rule.

---

# 247. Query Result Caching

Public query results can cache by normalized query.

---

# 248. But this can reveal query popularity.

---

# 249. Strict mode may disable/shared-cache avoid.

---

# 250. Hard rule.

---

# 251. Query Normalization

Versioned.

---

# 252. Unicode normalization.

---

# 253. Case folding.

---

# 254. Language-aware tokenization.

---

# 255. No semantic change without version bump.

---

# 256. Hard rule.

---

# 257. Search Language

Local UI language.

---

# 258. Public query may reveal language.

---

# 259. Strict mode can normalize/coarsen.

---

# 260. No precise locale fingerprint if not needed.

---

# 261. Hard rule.

---

# 262. Tokenization

Per index version.

---

# 263. SearchSchemaVersion.

---

# 264. Mixed version support.

---

# 265. No hidden semantic drift.

---

# 266. Relevance Explainability

Result can say:

```text
exact title match
matches topic
recent publication
```

---

# 267. No opaque behavioral reason.

---

# 268. Hard rule.

---

# 269. Public Popularity

Can influence ranking only in coarse aggregate.

---

# 270. No per-user popularity personalization.

---

# 271. Hard rule.

---

# 272. Anti-Manipulation

Threats:

```text
keyword stuffing
spam farms
index poisoning
ranking manipulation
```

---

# 273. Defenses

```text
content quality rules
rate limits
source diversity
moderation status
duplicate detection
```

---

# 274. No invasive device fingerprint requirement.

---

# 275. Hard rule.

---

# 276. Spam Indexing

Publisher quotas.

---

# 277. No unlimited index submissions.

---

# 278. Public content only after publication commit.

---

# 279. Signed provenance.

---

# 280. Source reputation can be scoped/public.

---

# 281. No global person reputation score.

---

# 282. Hard rule.

---

# 283. Search Moderation

Removed content not returned.

---

# 284. Moderation state explicit.

---

# 285. Ranking cannot shadow-ban silently.

---

# 286. Hard rule.

---

# 287. Result Visibility Decision

```rust
pub enum SearchVisibilityDecision {
    Eligible,
    HiddenByUser,
    HiddenByPolicy,
    RemovedByModeration,
    Deleted,
}
```

---

# 288. User-facing distinction where appropriate.

---

# 289. Search Permissions

Managed tenant.

---

# 290. Tenant Search

Can index org-owned data.

---

# 291. User personal content excluded.

---

# 292. Hard rule.

---

# 293. Tenant Search Identity

Org-scoped.

---

# 294. No cross-tenant query.

---

# 295. Hard rule.

---

# 296. Tenant Search Logs

Restricted/retention bounded.

---

# 297. Search administrators do not gain message access automatically.

---

# 298. Hard rule.

---

# 299. Managed E-Discovery

Separate compliance architecture.

---

# 300. Not equivalent to ordinary search.

---

# 301. Hard rule.

---

# 302. Search History

Device local.

---

# 303. Optional encrypted sync.

---

# 304. Personal history not visible to tenant.

---

# 305. Hard rule.

---

# 306. Search Suggestions

Local history/explicit topics.

---

# 307. No central personalized suggestion service baseline.

---

# 308. Hard rule.

---

# 309. Query Click/Open State

Do not report to server by default.

---

# 310. Public click analytics

aggregate thresholded only if enabled.

---

# 311. No per-user clickstream.

---

# 312. Hard rule.

---

# 313. Search Result Open

Local action.

---

# 314. Remote server only receives fetch request for content if needed.

---

# 315. Strict mode can fetch through privacy route/cache.

---

# 316. No direct link tracking by default.

---

# 317. Hard rule.

---

# 318. Query Privacy Budget

Optional.

---

# 319. Limit number of remote queries in strict mode.

---

# 320. Use local cache/shard fetch.

---

# 321. No fake security theater.

---

# 322. Hard rule.

---

# 323. Search Service API

```rust
pub trait SearchService {
    fn search(
        &self,
        query: SearchQuery,
    ) -> Result<SearchPage, SearchError>;
}
```

---

# 324. Local Search Service

```rust
pub trait LocalSearchService {
    fn search_local(
        &self,
        query: SearchQuery,
    ) -> Result<SearchPage, SearchError>;
}
```

---

# 325. Public Search Service

```rust
pub trait PublicSearchService {
    fn search_public(
        &self,
        query: PublicSearchQuery,
    ) -> Result<SearchPage, SearchError>;
}
```

---

# 326. Federated Search Coordinator

```rust
pub trait FederatedSearchCoordinator {
    fn search_federation(
        &self,
        query: FederatedSearchQuery,
    ) -> Result<Vec<FederatedSearchResult>, SearchError>;
}
```

---

# 327. Index Writer

```rust
pub trait SearchIndexWriter {
    fn upsert(
        &self,
        projection: SearchIndexProjection,
    ) -> Result<(), SearchError>;

    fn tombstone(
        &self,
        tombstone: SearchTombstone,
    ) -> Result<(), SearchError>;
}
```

---

# 328. Ranking

```rust
pub trait SearchRanker {
    fn rank(
        &self,
        query: &SearchQuery,
        candidates: &mut [SearchCandidate],
        mode: SearchRankingMode,
    ) -> Result<(), SearchError>;
}
```

---

# 329. Authorization Filter

```rust
pub trait SearchAuthorizationFilter {
    fn filter(
        &self,
        subject: &AuthorizationSubject,
        candidates: Vec<SearchCandidate>,
    ) -> Result<Vec<SearchCandidate>, SearchError>;
}
```

---

# 330. Filter Before Rank

Hard rule.

---

# 331. Search Error Taxonomy

```rust
pub enum SearchError {
    QueryInvalid,
    QueryTooLarge,
    ScopeDenied,
    IndexUnavailable,
    IndexStale,
    AuthorizationDenied,
    FederationUnavailable,
    PirUnavailable,
    CursorExpired,
    ContentDeleted,
    Internal,
}
```

---

# 332. Query Cursor

Opaque/short-lived.

---

# 333. No account identity in cursor.

---

# 334. Cursor scoped to search domain/query.

---

# 335. Hard rule.

---

# 336. Pagination

Bounded.

---

# 337. No unlimited result dump.

---

# 338. Public directory scraping controls.

---

# 339. Hard rule.

---

# 340. Search Result Count

Potential enumeration leak.

---

# 341. Public search may show approximate count.

---

# 342. Private/identity exact lookup avoid count.

---

# 343. Hard rule.

---

# 344. Count Class

```rust
pub enum SearchCountClass {
    Few,
    Some,
    Many,
    VeryMany,
}
```

---

# 345. Strict mode can omit.

---

# 346. No hidden-member count inference.

---

# 347. Hard rule.

---

# 348. Privacy Threat Model

Attacks:

```text
query logging
timing correlation
result enumeration
identity lookup abuse
private index leakage
federation query correlation
```

---

# 349. Query Logging

off/minimized.

---

# 350. Timing Correlation

batch/jitter/anonymity route in strict mode.

---

# 351. Enumeration

exact lookup/rate limit/approx counts.

---

# 352. Identity Lookup Abuse

no fuzzy people search.

---

# 353. Private Index Leakage

local encrypted.

---

# 354. Federation Correlation

bounded explicit domains, no global fanout.

---

# 355. Hard rule.

---

# 356. Search Telemetry

Safe:

```text
query latency
index freshness
error rate
result count bucket
```

---

# 357. Forbidden:

```text
raw query terms
per-user search history
clicked result history
interest profile
```

---

# 358. Hard rule.

---

# 359. Public Trend/Search Analytics

Aggregate query category only if needed.

---

# 360. No raw terms.

---

# 361. Minimum cohort.

---

# 362. Hard rule.

---

# 363. Search SLOs

Examples:

```text
local query latency
public search availability
index freshness
deletion propagation
federated timeout
```

---

# 364. Privacy SLO

```text
0 private content in public index
0 raw private queries in telemetry
```

---

# 365. Security SLO

```text
0 deleted content resurrected by reindex
0 unauthorized result opened via stale authorization
```

---

# 366. Failure Modes

```text
local index corruption
public index outage
federation timeout
PIR unavailable
stale cache
```

---

# 367. Local Index Corruption

Rebuild from authoritative local data.

---

# 368. Public Index Outage

Public search unavailable.

---

# 369. No fallback to private content scan.

---

# 370. Hard rule.

---

# 371. Federation Timeout

Partial result with domain status.

---

# 372. No hidden direct query to alternate domain.

---

# 373. Hard rule.

---

# 374. PIR Unavailable

Use ordinary query only if user policy allows.

---

# 375. No silent privacy downgrade.

---

# 376. Hard rule.

---

# 377. Stale Cache

Open reauthorizes/deletion checks.

---

# 378. No stale access.

---

# 379. Hard rule.

---

# 380. Search & Account Lifecycle

Deleted account private index deleted according to lifecycle.

---

# 381. Public authored content lifecycle separate.

---

# 382. No cloud search history after account deletion.

---

# 383. Hard rule.

---

# 384. Search & Profile

Public profile searchable only if opted in.

---

# 385. Exact-handle profile search separate.

---

# 386. No public profile auto-index.

---

# 387. Hard rule.

---

# 388. Search & Contacts

Local contacts search local only.

---

# 389. No server contact graph query.

---

# 390. Hard rule.

---

# 391. Search & Social Spaces

Private spaces local/scoped.

---

# 392. Listed/public spaces public index.

---

# 393. Roster never indexed.

---

# 394. Hard rule.

---

# 395. Search & Feed

Feed discovery can use public search.

---

# 396. Feed ranking separate.

---

# 397. Search result ranking cannot grant feed access.

---

# 398. Hard rule.

---

# 399. Search & Publishing

Only committed search-eligible revision indexed.

---

# 400. Draft never indexed.

---

# 401. Hard rule.

---

# 402. Search & Interactions

Public comments only if policy allows.

---

# 403. Private comments local/scoped.

---

# 404. Mention index never global.

---

# 405. Hard rule.

---

# 406. Search & Notifications

Search history does not become activity inbox.

---

# 407. Search result open not notified to creator.

---

# 408. Hard rule.

---

# 409. Authorization Integration

Part 81.

---

# 410. Protected results filtered before ranking.

---

# 411. Result open reauthorizes.

---

# 412. Hard rule.

---

# 413. Authentication Integration

Public search can be unauthenticated.

---

# 414. Tenant search uses managed auth.

---

# 415. Search token ≠ auth token.

---

# 416. Hard rule.

---

# 417. Edge Integration

Public search endpoint L7.

---

# 418. Query size/rate limits.

---

# 419. Private/E2EE search local.

---

# 420. WAF cannot inspect local query.

---

# 421. East-West Integration

Search/index services mTLS.

---

# 422. Secrets Integration

Index service credentials scoped.

---

# 423. No global search admin supertoken.

---

# 424. Hard rule.

---

# 425. Event Bus Integration

Index writer consumes committed publication/visibility/deletion events.

---

# 426. No raw private content event bus.

---

# 427. Hard rule.

---

# 428. Database Integration

Public index separate from authoritative content DB.

---

# 429. Private local index separate from local authoritative messages.

---

# 430. No one universal index store.

---

# 431. Hard rule.

---

# 432. Backup

Local search index not required in backup.

---

# 433. Rebuildable.

---

# 434. Search history optional.

---

# 435. Public index rebuilt from authoritative public projections + deletion ledger.

---

# 436. Hard rule.

---

# 437. Deletion Ledger

Required during rebuild.

---

# 438. No backup reindex resurrection.

---

# 439. Hard rule.

---

# 440. Serialization

Postcard internal.

---

# 441. RON config/export.

---

# 442. JSON external API/federation if required.

---

# 443. Strict bounds.

---

# 444. Query terms length bounded.

---

# 445. Result page bounded.

---

# 446. Federation domains bounded.

---

# 447. PIR response bounded.

---

# 448. Hard rule.

---

# 449. Performance

Local text search:

milliseconds.

---

# 450. Public search:

low latency.

---

# 451. Federation:

parallel bounded fanout.

---

# 452. Local ranking after merge.

---

# 453. No synchronous global federation.

---

# 454. Hard rule.

---

# 455. Index Updates

Asynchronous from outbox.

---

# 456. Small freshness lag acceptable.

---

# 457. Deletion/revocation higher priority.

---

# 458. Hard rule.

---

# 459. Public Index Sharding

By content/source/topic.

---

# 460. Not user ID.

---

# 461. Hard rule.

---

# 462. Tenant Index

Tenant-sharded.

---

# 463. No cross-tenant shard query.

---

# 464. Hard rule.

---

# 465. Local Index Limits

Bounded:

```text
disk size
documents
embedding cache
```

---

# 466. Eviction/rebuild policies.

---

# 467. No index larger than source corpus without bounds.

---

# 468. Hard rule.

---

# 469. Search Cache

Bounded.

---

# 470. Query cache privacy-aware.

---

# 471. Strict mode can disable persistent cache.

---

# 472. Crate Layout

Recommended:

```text
crates/
├── siar-search-core/
├── siar-local-index/
├── siar-public-index/
├── siar-search-projection/
├── siar-search-ranking/
├── siar-search-privacy/
├── siar-search-federation/
├── siar-pir/
├── siar-search-observability/
└── siar-search-testkit/
```

---

# 473. `siar-search-core`

Owns:

```text
queries
results
domains
errors
```

---

# 474. `siar-local-index`

Encrypted local FTS/fuzzy/vector index.

---

# 475. `siar-public-index`

Public searchable projections.

---

# 476. `siar-search-projection`

Private/public scoped result projections.

---

# 477. `siar-search-ranking`

Relevance/recency/local preference ranking.

---

# 478. `siar-search-privacy`

query policy/anti-enumeration/cache rules.

---

# 479. `siar-search-federation`

bounded domain fanout/merge.

---

# 480. `siar-pir`

reviewed PIR/shard-fetch adapters.

---

# 481. `siar-search-observability`

aggregate technical metrics only.

---

# 482. `siar-search-testkit`

index/privacy/deletion/federation/PIR tests.

---

# 483. Testing

Need dedicated search/index privacy testkit.

---

# 484. Test Scenarios

```text
local private search
public search
visibility change
deletion/reindex
federated search
```

---

# 485. Private Search Test

Query/content never leaves device.

---

# 486. Public Projection Test

No private field indexed.

---

# 487. Visibility Test

Public→private removes result.

---

# 488. Deletion Test

Tombstone prevents reindex resurrection.

---

# 489. Draft Test

Draft never public-indexed.

---

# 490. Authorization Test

Protected candidate filtered before ranking.

---

# 491. Stale Result Test

Open reauthorizes.

---

# 492. Query Logging Test

Raw query absent from logs.

---

# 493. Search History Test

Device-local by default.

---

# 494. Autocomplete Test

No remote prefix query baseline.

---

# 495. Identity Search Test

No fuzzy people enumeration.

---

# 496. Social Space Test

Private group absent from public search.

---

# 497. Federation Test

Only selected domains queried.

---

# 498. Federation Privacy Test

No account ID sent.

---

# 499. PIR Test

No silent fallback when policy requires PIR.

---

# 500. Ranking Test

Chronological/exact modes deterministic.

---

# 501. Telemetry Test

No query/interest profile emitted.

---

# 502. Backup/Rebuild Test

Deletion ledger applied before index ready.

---

# 503. Fuzzing

Fuzz:

```text
query parser
search cursor
public index projection
federation result
PIR response parser
```

---

# 504. Property Tests

Properties:

```text
private content never enters public index
unauthorized result can never reach ranker
deleted/tombstoned document never returns after reindex
search result opening never bypasses current authorization
```

---

# 505. Formal Verification Targets

Strong candidates:

```text
index eligibility
deletion/reindex anti-resurrection
federated scope selection
authorization-before-ranking
```

---

# 506. Kani Candidate

searchability/visibility subset rules.

---

# 507. TLA+ Candidate

publish→index→visibility change→delete→reindex sequence.

---

# 508. Loom Candidate

concurrent index update + deletion tombstone + query.

---

# 509. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Private message, draft, contact, and other E2EE-content search is local by default and does not require server plaintext indexing.
2. Searchability is a separate explicit property from visibility; public visibility never automatically implies public indexing.
3. Authorization and visibility filtering happen before ranking, and a search result never acts as authorization proof.
4. Raw remote search queries, click/open histories, and per-user search-interest profiles are not logged or centralized by default.
5. Identity search uses exact/capability-scoped resolution and cannot become a fuzzy email/phone/member enumeration endpoint.
6. Federated search queries only explicitly selected domains, preserves independent domain identity, and performs final merge/ranking locally.
7. PIR/private-search mechanisms never silently fall back to a weaker privacy mode when policy requires them.
8. Search indexes are derived/rebuildable state; deleted/retracted/private content cannot reappear through stale caches, backups, or reindexing.
9. Public search projections are typed/minimal and cannot expose private fields through shared serializers or reflection.
10. Local personalization/search history remains local by default and server relevance ranking does not depend on hidden behavioral profiles.
11. Search telemetry contains service health, latency, freshness, and coarse result metrics only—not raw queries, clicked results, or inferred interests.
12. Search systems never become a global social graph, membership graph, subscriber graph, or interaction graph through indexing side channels.
```

---

# 510. Initial Production Scope

Implement first:

```text
typed SearchDomain/Query/Result
encrypted local full-text search
local fuzzy/prefix search
public search projection model
explicit Searchability state
public index writer from committed events
authorization-before-ranking pipeline
relevance + recency ranking
device-local search history
exact identity/handle lookup
public social-space/content search
federated bounded-domain search
deletion tombstones + reindex safety
privacy-safe metrics
search testkit
```

Then add:

```text
local semantic/vector search
reviewed PIR exact lookup
shard-fetch private lookup
privacy-preserving federation query batching
advanced local relevance model
formal index/deletion verification
```

---

# 511. Definition of Done

Part 91 is complete when:

- private search is local-first
- searchability is explicit and separate from visibility
- public projections contain public fields only
- authorization happens before ranking
- result open reauthorizes
- raw queries are absent from telemetry/logs
- search history is local by default
- identity search cannot enumerate users
- federation fanout is bounded/explicit
- PIR policies never silently downgrade
- public index rebuild honors deletion ledger
- stale/deleted/private content cannot reappear
- search does not construct social/interest graphs
- local/public/federated/PIR/deletion/fuzz/formal tests are specified

---

# 512. Final Architecture

```text
                         QUERY
                           │
                           ▼
                  SCOPE / PRIVACY POLICY
                           │
             ┌─────────────┼─────────────┐
             │             │             │
        LOCAL INDEX   PUBLIC INDEX   FEDERATED INDEX
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                 AUTHORIZATION FILTER
                           │
                           ▼
                    RELEVANCE RANKING
                           │
                           ▼
                 RESULTS + PROVENANCE
```

Search safety model:

```text
local-first private indexing
+
typed public projections
+
authorization before ranking
+
query minimization
+
bounded federation
+
optional PIR
+
anti-enumeration
+
deletion-safe rebuilds
```

not:

```text
upload every private document, log every query, learn every user's interests, and rank results from a centralized behavioral profile
```

---

# 513. Final Principle

Search should help users find information without forcing them to reveal what they are looking for or exposing content they were never authorized to see.

The correct model is:

```text
index locally when private
+
project minimally when public
+
filter before rank
+
query remotely only when needed
+
federate explicitly
+
hide intent where practical
+
rebuild safely
+
never profile search behavior
```

This architecture gives SIAR a privacy-preserving search foundation for private messages, drafts, files, contacts, public posts, groups, channels, managed tenant content, federated content, and future PIR-backed lookup while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–90.
