# Core System Architecture Part 86 — Anonymous Network Group Directory, Communities, Channels, Membership Discovery, Moderation Boundaries & Privacy-Preserving Social Spaces Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 86  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 43, 45–46, 48, 56, 58–59, 68–69, 74–85  

**Primary purpose:** define SIAR's group directory, community, channel, membership-discovery, join/leave, moderation-boundary, announcement, large-group, public/private social-space, federation, abuse-resistance, and privacy-preserving social architecture without constructing a centralized map of group membership or user social participation.

---

# 1. Purpose

Social spaces create powerful community features, but also major privacy risks.

A centralized group system can reveal:

```text
which communities users join
which topics users follow
which organizations overlap
which channels people read
which members moderate whom
```

The governing principle is:

> **SIAR social spaces must support discovery, communities, channels, and moderation without turning membership or participation into a globally queryable identity graph.**

---

# 2. Architectural Position

```text
Directory / Invite / Capability
            │
            ▼
        Social Space
            │
    ┌───────┼────────┐
    │       │        │
 Community Group   Channel
    │       │        │
    └───────┼────────┘
            ▼
      Membership State
            │
            ▼
   Roles / Moderation Scope
```

---

# 3. Core Separation

Keep distinct:

```text
group
community
channel
directory entry
membership
role
moderation authority
announcement subscription
federation membership
```

---

# 4. Non-Goals

Part 86 does not create:

```text
a global membership directory
a global list of who joined what
a universal moderator identity
automatic contact creation from group membership
public participant history by default
```

---

# 5. Social Space Types

```rust
pub enum SocialSpaceType {
    Group,
    Community,
    Channel,
    AnnouncementChannel,
    ManagedWorkspaceSpace,
}
```

---

# 6. Group

Interactive many-to-many space.

---

# 7. Community

Container for multiple related spaces.

---

# 8. Channel

Topic/room within community or standalone.

---

# 9. Announcement Channel

One-to-many or limited-publisher broadcast.

---

# 10. Managed Workspace Space

Org-controlled social space.

---

# 11. Social Space ID

```rust
pub struct SocialSpaceId(pub [u8; 32]);
```

---

# 12. ID Scope

Opaque logical identifier.

---

# 13. Hard Rule

SocialSpaceId is not user identity and does not encode owner/member data.

---

# 14. Community ID

```rust
pub struct CommunityId(pub [u8; 32]);
```

---

# 15. Channel ID

```rust
pub struct ChannelId(pub [u8; 32]);
```

---

# 16. Directory Entry ID

Separate.

```rust
pub struct DirectoryEntryId(pub [u8; 32]);
```

---

# 17. Hard Rule

Directory entry identity and social-space identity remain separate.

---

# 18. Social Space Visibility

```rust
pub enum SocialSpaceVisibility {
    Private,
    InviteOnly,
    ExactHandle,
    Listed,
    Public,
}
```

---

# 19. Private

No directory entry.

---

# 20. InviteOnly

Capability required.

---

# 21. ExactHandle

Exact lookup only.

---

# 22. Listed

Appears in scoped directory/category.

---

# 23. Public

Broadly discoverable.

---

# 24. Default

Private or InviteOnly.

---

# 25. Hard Rule

Creating a group does not automatically publish it.

---

# 26. Directory Model

Directory is metadata projection, not authority.

---

# 27. Directory Record

```rust
pub struct SocialDirectoryRecord {
    pub entry_id: DirectoryEntryId,
    pub social_space: SocialSpaceRef,
    pub title: DirectoryTitle,
    pub category: Option<DirectoryCategory>,
    pub visibility: SocialSpaceVisibility,
    pub capabilities: DirectoryCapabilities,
    pub signatures: DirectorySignatureBundle,
}
```

---

# 28. Directory Record Must Not Include

```text
full member list
member account IDs
device IDs
moderation history
```

---

# 29. Hard Rule

Directory discovery does not disclose membership.

---

# 30. Directory Scope

```rust
pub enum DirectoryScope {
    Local,
    Community,
    Tenant,
    FederationDomain,
    PublicNetwork,
}
```

---

# 31. No One Global Directory Required

Hard rule.

---

# 32. Public Network Directory

Optional.

---

# 33. Federation Directory

Domain-scoped.

---

# 34. Tenant Directory

Organization-scoped.

---

# 35. Local Directory

Device/local curated list.

---

# 36. Directory Authority

Signs directory projection.

---

# 37. Does Not Own Membership Authority

Hard rule.

---

# 38. Membership Authority

Social-space-specific.

---

# 39. Membership State

```rust
pub enum MembershipState {
    NotMember,
    Invited,
    Requested,
    Active,
    Muted,
    Suspended,
    Left,
    Removed,
    Banned,
}
```

---

# 40. Invited

Capability offered.

---

# 41. Requested

Join request pending.

---

# 42. Active

Member.

---

# 43. Muted

Local notification state, not membership authority.

---

# 44. Suspended

Temporary participation restriction.

---

# 45. Left

Voluntary.

---

# 46. Removed

Moderator/admin removal.

---

# 47. Banned

Membership capability denied.

---

# 48. Hard Rule

Muted is never treated as suspension/removal.

---

# 49. Membership ID

```rust
pub struct MembershipId(pub [u8; 32]);
```

---

# 50. Relationship To User Identity

Membership may be pseudonymous.

---

# 51. Membership Identity Mode

```rust
pub enum MembershipIdentityMode {
    AccountLinked,
    ScopedPseudonym,
    AnonymousCapability,
    ManagedIdentity,
}
```

---

# 52. AccountLinked

Explicitly identifiable.

---

# 53. ScopedPseudonym

Stable only within social space.

---

# 54. AnonymousCapability

No durable public identifier where protocol allows.

---

# 55. ManagedIdentity

Org-scoped identity.

---

# 56. Hard Rule

Public social spaces do not require global AccountId exposure.

---

# 57. Scoped Membership Pseudonym

```rust
pub struct MembershipPseudonym(pub [u8; 32]);
```

---

# 58. Domain Separation

Derived/scoped separately per social space.

---

# 59. No Cross-Group Correlation

Hard rule.

---

# 60. Membership Discovery

Different from space discovery.

---

# 61. Default

Member list private/restricted.

---

# 62. Membership Visibility

```rust
pub enum MembershipVisibility {
    Hidden,
    MembersOnly,
    ModeratorsOnly,
    PublicOptIn,
}
```

---

# 63. Hidden

No member list exposed.

---

# 64. MembersOnly

Only authenticated members.

---

# 65. ModeratorsOnly

Administrative use.

---

# 66. PublicOptIn

Only members who opted in.

---

# 67. Hard Rule

Public group does not imply public membership list.

---

# 68. Join Policy

```rust
pub enum JoinPolicy {
    InvitationOnly,
    ApprovalRequired,
    OpenCapability,
    ManagedMembership,
}
```

---

# 69. InvitationOnly

Scoped invite token.

---

# 70. ApprovalRequired

Join request + moderation approval.

---

# 71. OpenCapability

Anyone with directory capability can join.

---

# 72. ManagedMembership

Org directory controls.

---

# 73. Join Capability

```rust
pub struct JoinCapability {
    pub space: SocialSpaceId,
    pub allowed_role: InitialRole,
    pub expires_at: Timestamp,
    pub use_policy: JoinUsePolicy,
}
```

---

# 74. Join Capability Must Not Grant

```text
moderator authority
admin authority
cross-space access
```

unless explicitly encoded.

---

# 75. Hard Rule

Join capability is not generic social-space authority.

---

# 76. Join Use Policy

```rust
pub enum JoinUsePolicy {
    SingleUse,
    MultiUseBounded(u32),
    OpenUntilExpiry,
}
```

---

# 77. No Unlimited Permanent Join Link By Default

Hard rule.

---

# 78. Join Request

```rust
pub struct JoinRequest {
    pub request_id: JoinRequestId,
    pub space: SocialSpaceId,
    pub proposed_identity: MembershipIdentityProposal,
    pub proof: JoinProof,
}
```

---

# 79. Join Request Data Minimization

Only required metadata.

---

# 80. No Account Email/Phone By Default

Hard rule.

---

# 81. Join Flow

```text
discover/invite
→ validate capability
→ request/approve if required
→ create membership identity
→ distribute group keys/capabilities
→ activate membership
```

---

# 82. Membership Activation

Atomic local authority update + durable event.

---

# 83. No Message Decryption Before Membership Activation

Hard rule.

---

# 84. Leave Flow

```text
request leave
→ stop new send authority
→ update membership epoch
→ rotate/rekey if required
→ stop future state delivery
```

---

# 85. Leave Is Not Historical Erasure

Hard truth.

---

# 86. Removed/Banned Flow

Moderator authority required.

---

# 87. Rekey

Depending group E2EE model.

---

# 88. Group Epoch

```rust
pub struct GroupEpoch(pub u64);
```

---

# 89. Membership changes may advance epoch.

---

# 90. Old member

Cannot derive future keys.

---

# 91. Hard Rule

Removed/banned member cannot continue future group decryption through stale group state.

---

# 92. Role Model

```rust
pub enum SocialRole {
    Member,
    Publisher,
    Moderator,
    Administrator,
    Owner,
}
```

---

# 93. Role ≠ Authority

Compile to scoped capabilities.

---

# 94. Hard Rule

Domain code checks capabilities/actions, not role name alone.

---

# 95. Role Scope

```rust
pub enum RoleScope {
    Space(SocialSpaceId),
    Channel(ChannelId),
    Community(CommunityId),
}
```

---

# 96. No Global Moderator Role

Hard rule.

---

# 97. Moderator Authority

Scoped.

---

# 98. Moderator Actions

```rust
pub enum ModerationAction {
    RemoveMessage,
    RestrictMember,
    RemoveMember,
    BanMember,
    PinMessage,
    LockChannel,
    ApproveJoin,
}
```

---

# 99. Admin Actions

```text
change policy
assign roles
rotate ownership
delete space
```

---

# 100. Owner

High-risk role.

---

# 101. No One Moderator Automatically Gets Owner/Admin Actions

Hard rule.

---

# 102. Moderation Boundary

Moderation only applies inside the social space.

---

# 103. Hard Rule

Group moderator cannot affect personal account, contacts, unrelated groups, or transport identity.

---

# 104. Managed Workspace

Org admin may control managed spaces.

---

# 105. Personal spaces remain separate.

---

# 106. Hard rule.

---

# 107. Moderation Capability

```rust
pub struct ModerationCapability {
    pub moderator: MembershipPseudonym,
    pub scope: RoleScope,
    pub actions: BTreeSet<ModerationAction>,
    pub expires_at: Option<Timestamp>,
}
```

---

# 108. Delegation

Attenuation only.

---

# 109. No role escalation through delegation.

---

# 110. Hard rule.

---

# 111. Moderation Evidence

Scoped.

---

# 112. Can include:

```text
message reference
rule ID
reason class
```

---

# 113. Should not include unrelated conversation history.

---

# 114. Hard rule.

---

# 115. Moderation Audit

Local/social-space scoped.

---

# 116. No global user moderation dossier by default.

---

# 117. Hard rule.

---

# 118. Public Moderation Transparency

Optional.

---

# 119. Can publish:

```text
policy
aggregate actions
appeal process
```

---

# 120. Not full user identity history.

---

# 121. Channel Types

```rust
pub enum ChannelType {
    Discussion,
    Announcement,
    Media,
    Support,
    ReadOnly,
}
```

---

# 122. Discussion

Many senders.

---

# 123. Announcement

Limited publishers.

---

# 124. Media

Attachment-heavy.

---

# 125. Support

Threaded support/help.

---

# 126. ReadOnly

Archive/reference.

---

# 127. Channel Membership

Can inherit from community or be independent.

---

# 128. Inheritance

Explicit.

---

# 129. No Accidental Cross-Channel Access

Hard rule.

---

# 130. Channel Capability

```rust
pub struct ChannelCapabilities {
    pub read: CapabilityState,
    pub publish: CapabilityState,
    pub upload: CapabilityState,
    pub moderate: CapabilityState,
}
```

---

# 131. Announcement Channel

Publisher capability restricted.

---

# 132. Subscribers need not reveal identity publicly.

---

# 133. Hard rule.

---

# 134. Subscription State

```rust
pub enum SubscriptionState {
    Subscribed,
    Muted,
    Unsubscribed,
}
```

---

# 135. Subscription ≠ Membership

Hard rule.

---

# 136. Public Channel Subscription

Can be pseudonymous/private.

---

# 137. Directory Discovery

Public listing metadata.

---

# 138. Search Categories

Optional.

---

# 139. No behavioral ranking by default.

---

# 140. Hard rule.

---

# 141. Search Model

```rust
pub struct DirectorySearchQuery {
    pub text: Option<DirectoryQueryText>,
    pub category: Option<DirectoryCategory>,
    pub scope: DirectoryScope,
}
```

---

# 142. Search Privacy

No account identity required for public directory search.

---

# 143. Search Logs

Minimized.

---

# 144. No long-lived query profile.

---

# 145. Hard rule.

---

# 146. Exact Handle Search

Part 59.

---

# 147. Listed Directory Search

Can support text/category.

---

# 148. Public Directory

Must not expose member list.

---

# 149. Search Result

```rust
pub struct DirectorySearchResult {
    pub entry: DirectoryEntryId,
    pub title: DirectoryTitle,
    pub summary: Option<DirectorySummary>,
    pub join_mode: JoinPolicy,
}
```

---

# 150. No member identities.

---

# 151. Search Ranking

Use:

```text
text relevance
category
coarse popularity bucket
```

if enabled.

---

# 152. Popularity

Privacy-sensitive.

---

# 153. Prefer coarse bucket.

---

# 154. No exact member count in strict privacy mode.

---

# 155. Hard rule.

---

# 156. Popularity Bucket

```rust
pub enum PopularityClass {
    Small,
    Medium,
    Large,
    VeryLarge,
}
```

---

# 157. No per-user personalization required.

---

# 158. No "people you know joined this group."

---

# 159. Hard rule.

---

# 160. Community Structure

Community can contain channels/groups.

---

# 161. Community Graph

```rust
pub struct CommunityStructure {
    pub community: CommunityId,
    pub children: Vec<SocialSpaceRef>,
}
```

---

# 162. Bounded.

---

# 163. No arbitrary recursive cycles.

---

# 164. Hard rule.

---

# 165. Hierarchy Depth

Limited.

---

# 166. Why

Policy/UX/performance.

---

# 167. Community Role Inheritance

Explicit.

---

# 168. Example

CommunityAdmin may manage channels.

---

# 169. But

No implicit authority outside community.

---

# 170. Hard rule.

---

# 171. Large Group Architecture

Needs scalable fanout.

---

# 172. Avoid O(n) sender upload where possible.

---

# 173. Possible models:

```text
tree fanout
provider pub/sub
encrypted distribution service
```

---

# 174. Still E2EE where required.

---

# 175. Server sees minimal metadata.

---

# 176. Large Group Sender Keys

Part 43.

---

# 177. Membership epochs.

---

# 178. Rekey strategy.

---

# 179. No Central Plaintext Group Message Store

Hard rule.

---

# 180. Channel Message Storage

Ciphertext.

---

# 181. Public announcement content

May be public plaintext if intentionally public.

---

# 182. Distinguish public content from private membership metadata.

---

# 183. Hard rule.

---

# 184. Group Membership Privacy

Provider should not need full canonical roster where hidden-roster mode is used.

---

# 185. Membership modes:

```rust
pub enum RosterMode {
    FullRoster,
    MembersOnlyRoster,
    HiddenRoster,
    ModeratorOnlyRoster,
}
```

---

# 186. HiddenRoster

Recommended for sensitive groups.

---

# 187. Public group

May still use MembersOnlyRoster.

---

# 188. No assumption public=public roster.

---

# 189. Hard rule.

---

# 190. Hidden Roster Tradeoff

Moderation/distribution more complex.

---

# 191. Membership Proof

Capability/proof.

---

# 192. Server can validate membership without exposing full roster where design supports.

---

# 193. No custom anonymous-credential crypto initially.

---

# 194. Hard rule.

---

# 195. Use reviewed primitives.

---

# 196. Join Approval

Moderators can approve request.

---

# 197. Request metadata minimized.

---

# 198. No applicant contact graph.

---

# 199. Ban Model

Scoped.

---

# 200. Ban ID

```rust
pub struct BanRecord {
    pub space: SocialSpaceId,
    pub subject: BanSubjectRef,
    pub reason: BanReasonClass,
    pub expires_at: Option<Timestamp>,
}
```

---

# 201. Ban Subject

Space-scoped identity/capability.

---

# 202. Not global AccountId if avoidable.

---

# 203. Hard rule.

---

# 204. Temporary Ban

Supported.

---

# 205. Permanent Ban

Space-scoped.

---

# 206. No Automatic Cross-Group Ban Propagation

Hard rule.

---

# 207. Abuse Reporting

Part 46.

---

# 208. Reports scoped to social space.

---

# 209. Moderator sees only evidence necessary.

---

# 210. Platform abuse response

Separate.

---

# 211. No universal moderator backdoor.

---

# 212. Hard rule.

---

# 213. Appeals

Optional but recommended for managed/public spaces.

---

# 214. Appeal Scope

Only moderation action.

---

# 215. No account-wide authority.

---

# 216. Social Space Policy

```rust
pub struct SocialSpacePolicy {
    pub visibility: SocialSpaceVisibility,
    pub join_policy: JoinPolicy,
    pub roster_mode: RosterMode,
    pub moderation_policy: ModerationPolicy,
    pub retention: RetentionPolicyId,
}
```

---

# 217. Versioned.

---

# 218. Signed where authoritative.

---

# 219. Anti-Rollback.

---

# 220. No Silent Policy Weakening

Hard rule.

---

# 221. Public/Private Transition

High-risk.

---

# 222. Private → Public

Requires explicit admin/owner action.

---

# 223. Existing members should be informed if membership exposure changes.

---

# 224. Hard rule.

---

# 225. Public → Private

Allowed.

---

# 226. Directory entry removed.

---

# 227. Cached public metadata expires.

---

# 228. No immediate guarantee all external caches disappear.

---

# 229. Hard truth.

---

# 230. Space Ownership

Owner capability.

---

# 231. Ownership Transfer

Explicit two-party or multi-step.

---

# 232. No implicit owner change.

---

# 233. Hard rule.

---

# 234. Ownership Transfer Flow

```text
propose new owner
→ authenticate/authorize
→ accept
→ rotate admin capability
→ revoke old owner capability if transferring fully
```

---

# 235. No Account Password Transfer.

---

# 236. Ownership Capability

Scoped to social space.

---

# 237. Community Deletion

Long-running lifecycle.

---

# 238. Similar to Part 83 deletion saga.

---

# 239. Stop new joins.

---

# 240. Revoke roles.

---

# 241. Delete/expire directory entry.

---

# 242. Process content/retention.

---

# 243. Hard Rule

Deleted social space ID not reused.

---

# 244. Space Deactivation

Temporary.

---

# 245. No new messages/joins.

---

# 246. Can later reactivate.

---

# 247. Deactivation ≠ deletion.

---

# 248. Group Archive

Read-only local/server state.

---

# 249. Does not revoke membership automatically.

---

# 250. Hard rule.

---

# 251. Federation

Cross-domain communities.

---

# 252. Federation Gateway

Explicit.

---

# 253. No shared global DB.

---

# 254. Remote membership represented through federation-scoped pseudonym/capability.

---

# 255. No global user identity merge.

---

# 256. Hard rule.

---

# 257. Federation Join

Bilateral policy.

---

# 258. Remote moderation

Limited to federated space scope.

---

# 259. No remote domain admin authority.

---

# 260. Hard rule.

---

# 261. Federation Directory

Signed projection.

---

# 262. Remote domain may choose listed/public metadata.

---

# 263. No remote member list by default.

---

# 264. Cross-Domain Ban

Only federated space.

---

# 265. Not global.

---

# 266. Tenant/Enterprise Spaces

Part 69.

---

# 267. Org membership can auto-provision managed-space membership.

---

# 268. But

Only managed space.

---

# 269. No personal contact creation.

---

# 270. Hard rule.

---

# 271. Offboarding

Revokes managed-space membership.

---

# 272. Personal communities unaffected.

---

# 273. Managed directory

Can list org spaces.

---

# 274. User may join external/public communities separately.

---

# 275. Policy can restrict managed profile only.

---

# 276. No org visibility into personal community memberships by default.

---

# 277. Hard rule.

---

# 278. Group Discovery From Contacts

Optional local suggestion.

---

# 279. No server mutual-membership mining.

---

# 280. Hard rule.

---

# 281. Invitation Sharing

Member may invite if capability.

---

# 282. Invite privilege separate.

---

# 283. Member Role Does Not Automatically Include Invite

Hard rule.

---

# 284. Invite Capability

```rust
pub struct SpaceInviteCapability {
    pub space: SocialSpaceId,
    pub inviter_scope: MembershipPseudonym,
    pub max_uses: u32,
    pub expires_at: Timestamp,
}
```

---

# 285. Revocable.

---

# 286. Invitation Attribution

Optional.

---

# 287. Avoid public invite graph.

---

# 288. No "who invited whom" global analytics.

---

# 289. Hard rule.

---

# 290. Join Spam

Bound request queue.

---

# 291. Join Request Rate Limits

Per invitation/source capability.

---

# 292. CAPTCHA optional but privacy-costly.

---

# 293. Prefer proof/credits/approval.

---

# 294. No real-world identity requirement.

---

# 295. Social Space Search Abuse

Prevent crawling.

---

# 296. Rate limit.

---

# 297. Pagination bounds.

---

# 298. No complete downloadable public directory snapshot if threat model rejects enumeration.

---

# 299. Hard rule.

---

# 300. Public Directory Snapshot

If intentionally public, can be signed/replicated.

---

# 301. But

No member data.

---

# 302. Directory Caching

Allowed.

---

# 303. Metadata TTL.

---

# 304. Remove hidden/deleted entries on refresh.

---

# 305. No eternal cache guarantee.

---

# 306. Subscription Privacy

Announcement channels.

---

# 307. Server should avoid exposing subscriber list.

---

# 308. Publisher need not know subscribers individually.

---

# 309. Hard rule.

---

# 310. Subscriber Delivery

Can use mailbox/pub-sub distribution.

---

# 311. No per-subscriber read tracking by default.

---

# 312. Analytics

Aggregate only if enabled.

---

# 313. No subscriber identity export.

---

# 314. Hard rule.

---

# 315. Read Receipts

Off by default for large/public channels.

---

# 316. Why

Massive privacy/metadata leakage.

---

# 317. Per-message view counts

Optional aggregate.

---

# 318. Must not reveal individual viewers.

---

# 319. Hard rule.

---

# 320. Presence In Large Groups

Avoid full presence roster.

---

# 321. Use coarse aggregate or none.

---

# 322. Typing indicators

Small groups only.

---

# 323. No typing broadcast to thousands.

---

# 324. Hard rule.

---

# 325. Member Count

Coarse.

---

# 326. Exact count may be hidden.

---

# 327. Public spaces can show bucket.

---

# 328. Large Group Privacy

No precise online/member count if not needed.

---

# 329. Content Moderation Architecture

Moderator action on content references.

---

# 330. E2EE Challenge

Server cannot inspect private group content.

---

# 331. Private group moderation

Client/member reports + moderator decryption context.

---

# 332. No universal server-side plaintext moderation key.

---

# 333. Hard rule.

---

# 334. Public channels

Content may intentionally be public.

---

# 335. Server-side moderation can inspect public content if policy says so.

---

# 336. Keep identity metadata minimized.

---

# 337. Moderator Key

Separate capability, not decryption escrow.

---

# 338. No "moderator can decrypt all historical private groups" key.

---

# 339. Hard rule.

---

# 340. Content Retention

Per social-space policy.

---

# 341. Ephemeral groups

Possible.

---

# 342. Public archives

Possible.

---

# 343. Private groups

Local/device retention.

---

# 344. No global retention default.

---

# 345. Retention Class

```rust
pub enum SocialRetentionClass {
    Ephemeral,
    Standard,
    Archive,
    Managed,
}
```

---

# 346. Deletion

Part 67.

---

# 347. Removed content

Tombstone/reference if needed.

---

# 348. Do not retain plaintext moderation copies indefinitely.

---

# 349. Hard rule.

---

# 350. Search

Local search for private groups.

---

# 351. Server index for public content only if explicitly public.

---

# 352. No server private-content index.

---

# 353. Hard rule.

---

# 354. Channel Threads

Optional.

---

# 355. Thread ID

Channel-scoped.

---

# 356. No cross-space thread ID.

---

# 357. Pinning

Moderation capability.

---

# 358. Slow Mode

Channel policy.

---

# 359. Rate Limit

Can be local/server enforced.

---

# 360. No user behavior profiling needed.

---

# 361. Reaction Aggregation

Optional.

---

# 362. Private group

E2EE event.

---

# 363. Public channel

Aggregate.

---

# 364. No public list of reactors unless explicitly enabled.

---

# 365. Hard rule.

---

# 366. Community Recommendations

Default disabled.

---

# 367. If added

Use public metadata + local interest selection.

---

# 368. No centralized personal-interest model.

---

# 369. Hard rule.

---

# 370. Trending

Can be aggregate public-space metric.

---

# 371. Use coarse counts.

---

# 372. Do not combine with user identity.

---

# 373. No targeted recommendation required.

---

# 374. Social Space Repository

```rust
pub trait SocialSpaceRepository {
    fn get(
        &self,
        id: SocialSpaceId,
    ) -> Result<Option<SocialSpaceRecord>, SocialSpaceError>;

    fn update_policy(
        &self,
        id: SocialSpaceId,
        policy: SocialSpacePolicy,
    ) -> Result<(), SocialSpaceError>;
}
```

---

# 375. Membership Repository

```rust
pub trait MembershipRepository {
    fn membership(
        &self,
        space: SocialSpaceId,
        subject: MembershipSubjectRef,
    ) -> Result<MembershipState, SocialSpaceError>;
}
```

---

# 376. Directory Service

```rust
pub trait SocialDirectoryService {
    fn search(
        &self,
        query: DirectorySearchQuery,
    ) -> Result<Vec<DirectorySearchResult>, SocialSpaceError>;
}
```

---

# 377. Join Service

```rust
pub trait SocialJoinService {
    fn join(
        &self,
        request: JoinRequest,
    ) -> Result<MembershipId, SocialSpaceError>;
}
```

---

# 378. Moderation Service

```rust
pub trait ModerationService {
    fn apply(
        &self,
        actor: ModerationCapability,
        action: ModerationCommand,
    ) -> Result<ModerationResult, SocialSpaceError>;
}
```

---

# 379. No direct DB mutation.

---

# 380. Hard rule.

---

# 381. Authorization Integration

Part 81.

---

# 382. Roles compile to capabilities.

---

# 383. Membership state is ABAC input.

---

# 384. Moderator action checked at PEP.

---

# 385. No Role Name Shortcut.

---

# 386. Authentication Integration

Part 82.

---

# 387. Public/pseudonymous group participation may not require global account exposure.

---

# 388. Managed spaces require org auth.

---

# 389. Account Lifecycle

Part 83.

---

# 390. Deleted account loses memberships.

---

# 391. Suspended account policy-specific.

---

# 392. Managed offboarding removes managed memberships.

---

# 393. No stale membership after account deletion.

---

# 394. Hard rule.

---

# 395. Profile Integration

Part 84.

---

# 396. Group profile projection separate.

---

# 397. Member nickname/avatar can be space-scoped.

---

# 398. No automatic personal profile disclosure.

---

# 399. Hard rule.

---

# 400. Contact Integration

Part 85.

---

# 401. Group membership does not create contact edge.

---

# 402. Direct contact invite separate.

---

# 403. No mutual membership leakage.

---

# 404. Messaging Integration

Part 43.

---

# 405. Group encryption/membership epoch.

---

# 406. Channel message envelope.

---

# 407. No directory service in message path.

---

# 408. Hard rule.

---

# 409. Event Bus Integration

Part 75.

---

# 410. Membership/moderation events scoped.

---

# 411. No global social event stream.

---

# 412. Hard rule.

---

# 413. Directory Discovery Integration

Part 77.

---

# 414. Public/tenant directory service can use signed service records.

---

# 415. Group identity trust separate from endpoint identity.

---

# 416. Edge Integration

Part 78.

---

# 417. Directory/join APIs L7.

---

# 418. Group ciphertext paths opaque.

---

# 419. No WAF decryption of private group content.

---

# 420. Hard rule.

---

# 421. East-West Integration

Part 79.

---

# 422. Membership service, directory, moderation service use service identity/mTLS.

---

# 423. Secrets Integration

Part 80.

---

# 424. Group signing/admin keys scoped.

---

# 425. No shared moderator secret.

---

# 426. Federation Integration

Part 58.

---

# 427. Cross-domain social space state uses explicit federation protocol.

---

# 428. No database-level cross-domain join.

---

# 429. Hard rule.

---

# 430. Storage

Directory metadata:

```text
public/signed projection
```

Membership:

```text
encrypted/scoped authority state
```

Messages:

```text
ciphertext/public content by mode
```

---

# 431. No single giant table.

---

# 432. Hard rule.

---

# 433. Membership Privacy At Rest

Encrypt sensitive roster.

---

# 434. Tenant-specific keys.

---

# 435. No global member index.

---

# 436. Hard rule.

---

# 437. Search Index

Public/listed directory only.

---

# 438. Private groups not indexed.

---

# 439. Hidden groups absent from search.

---

# 440. Exact-handle groups only exact resolution.

---

# 441. No accidental indexing.

---

# 442. Hard rule.

---

# 443. Observability

Safe aggregate metrics:

```text
directory query success
join success
moderation action count
fanout lag
```

---

# 444. Forbidden:

```text
user→group membership graph
member identity list
viewer list
moderator dossier
```

---

# 445. Hard rule.

---

# 446. Per-Space Metrics

For owner/moderator, aggregate.

---

# 447. Examples:

```text
message volume
active member bucket
join request count
```

---

# 448. No individual participation analytics by default.

---

# 449. Hard rule.

---

# 450. Audit

High-risk moderation/admin actions.

---

# 451. Minimal.

---

# 452. No message content unless explicitly part of moderation evidence and permitted.

---

# 453. Retention bounded.

---

# 454. Moderation audit record:

```rust
pub struct ModerationAuditRecord {
    pub space: SocialSpaceId,
    pub action: ModerationAction,
    pub actor_scope: MembershipPseudonym,
    pub outcome: AuditOutcome,
}
```

---

# 455. No global account identifier where avoidable.

---

# 456. Privacy Threat Model

Attacks:

```text
directory crawling
membership enumeration
cross-group identity correlation
join-token leakage
moderator overreach
subscriber tracking
```

---

# 457. Directory Crawling

Rate limit/pagination.

---

# 458. Membership Enumeration

No public roster by default.

---

# 459. Cross-Group Correlation

Scoped pseudonyms.

---

# 460. Join Token Leakage

Expiry/revocation/scope.

---

# 461. Moderator Overreach

Capability-scoped enforcement.

---

# 462. Subscriber Tracking

No subscriber identity export.

---

# 463. No Stable Analytics Identifier Across Spaces

Hard rule.

---

# 464. Abuse Threat Model

```text
spam groups
mass invites
join-request flood
malicious moderators
raids
```

---

# 465. Spam Groups

Directory quality/rate limiting.

---

# 466. Mass Invites

Invite capability quotas.

---

# 467. Join Flood

Queue/admission controls.

---

# 468. Malicious Moderator

Scoped audit/appeal.

---

# 469. Raids

Join approval/slow mode/capabilities.

---

# 470. No Central Real-World Identity Verification Requirement.

---

# 471. Hard rule.

---

# 472. Security Events

Examples:

```text
ownership transfer
moderator grant
membership key rotation
policy publicization
```

---

# 473. Notify affected admins/members as appropriate.

---

# 474. No unnecessary broadcast of member identity.

---

# 475. Group Key Rotation

Part 43/66.

---

# 476. Membership change triggers as policy requires.

---

# 477. Large group may use sender-key generations.

---

# 478. Key compromise

rotate.

---

# 479. Removed member

future secrecy.

---

# 480. New member

past history exposure policy explicit.

---

# 481. History Sharing Policy

```rust
pub enum HistoryAccessPolicy {
    FromJoinOnly,
    RecentWindow,
    FullHistory,
    None,
}
```

---

# 482. Default Private Group

FromJoinOnly.

---

# 483. Public channel

FullHistory possible.

---

# 484. Hard Rule

New membership never silently gains historical private content unless policy explicitly says so.

---

# 485. Message History Export

Member scope.

---

# 486. Public content export unrestricted per policy.

---

# 487. Private group export may require capability.

---

# 488. No moderator automatic export-all authority.

---

# 489. Hard rule.

---

# 490. Backup

Part 33.

---

# 491. Own membership state backed up logically.

---

# 492. Live group ratchets/session keys not blindly restored.

---

# 493. New device re-provisioned.

---

# 494. Removed/banned state must survive restore.

---

# 495. Hard rule.

---

# 496. Deletion Anti-Resurrection

Space deletion epoch.

---

# 497. Membership revocation epoch.

---

# 498. Stale backup cannot re-list/rejoin deleted space automatically.

---

# 499. Hard rule.

---

# 500. Social Space Deletion Epoch

```rust
pub struct SocialSpaceDeletionEpoch(pub u64);
```

---

# 501. Directory Publication Epoch

Monotonic.

---

# 502. Old public record rejected.

---

# 503. No stale directory resurrection.

---

# 504. Hard rule.

---

# 505. Schema Versioning

Separate versions for:

```text
directory record
membership protocol
moderation protocol
message protocol
storage
```

---

# 506. No conflation.

---

# 507. Mixed Version

Rolling compatible.

---

# 508. Unknown moderation action

deny.

---

# 509. Unknown membership state

restrictive.

---

# 510. Hard rule.

---

# 511. Serialization

Postcard internal/protocol.

---

# 512. RON policy/config.

---

# 513. JSON external directory/API interop.

---

# 514. Strict bounds.

---

# 515. Directory text

Bounded.

---

# 516. Community children

Bounded.

---

# 517. Member list response

Bounded/paged.

---

# 518. No unbounded roster payload.

---

# 519. Hard rule.

---

# 520. Performance

Directory search

indexed.

---

# 521. Membership check

local/cache.

---

# 522. No central directory call per message.

---

# 523. Hard rule.

---

# 524. Large Group Fanout

Asynchronous.

---

# 525. Bounded queues.

---

# 526. Backpressure.

---

# 527. Bulk/attachment priority lower.

---

# 528. Moderation/security events prioritized.

---

# 529. No one space exhausts global fanout capacity.

---

# 530. Tenant/social-space quotas.

---

# 531. Fair scheduling.

---

# 532. Channel Partitioning

By social-space/channel ID.

---

# 533. No user-based partition key.

---

# 534. Hard rule.

---

# 535. Large Community Scaling

Shard independent channels.

---

# 536. Community metadata small.

---

# 537. Avoid giant cross-channel transaction.

---

# 538. Use saga/events.

---

# 539. No distributed 2PC.

---

# 540. Hard rule.

---

# 541. Crate Layout

Recommended:

```text
crates/
├── siar-social-space-core/
├── siar-community/
├── siar-channel/
├── siar-social-directory/
├── siar-membership/
├── siar-membership-privacy/
├── siar-social-invitation/
├── siar-social-moderation/
├── siar-social-federation/
├── siar-social-observability/
└── siar-social-testkit/
```

---

# 542. `siar-social-space-core`

Owns:

```text
space IDs
visibility
errors
```

---

# 543. `siar-community`

Community hierarchy/policy.

---

# 544. `siar-channel`

Channel types/capabilities/subscriptions.

---

# 545. `siar-social-directory`

Directory projections/search.

---

# 546. `siar-membership`

Join/leave/remove/ban state.

---

# 547. `siar-membership-privacy`

Scoped pseudonyms/roster visibility.

---

# 548. `siar-social-invitation`

Join capabilities/invite lifecycle.

---

# 549. `siar-social-moderation`

Scoped moderator capabilities/actions/audits.

---

# 550. `siar-social-federation`

Cross-domain spaces.

---

# 551. `siar-social-observability`

Aggregate metrics only.

---

# 552. `siar-social-testkit`

Directory/join/moderation/privacy/federation tests.

---

# 553. Error Taxonomy

```rust
pub enum SocialSpaceError {
    SpaceNotFound,
    DirectoryEntryNotFound,
    NotMember,
    MembershipSuspended,
    MembershipRemoved,
    MembershipBanned,
    JoinDenied,
    InviteInvalid,
    InviteExpired,
    CapabilityDenied,
    ModerationDenied,
    RosterHidden,
    FederationDenied,
    PolicyViolation,
    SpaceDeleted,
    Internal,
}
```

---

# 554. Testing

Need dedicated social-space testkit.

---

# 555. Test Scenarios

```text
private group
public channel
hidden roster
join approval
ban
ownership transfer
```

---

# 556. Directory Test

Private space never appears.

---

# 557. Exact Handle Test

Only exact lookup resolves.

---

# 558. Public Directory Test

No member list leakage.

---

# 559. Membership Test

Active required before private content.

---

# 560. Hidden Roster Test

Non-authorized caller cannot enumerate members.

---

# 561. Scoped Pseudonym Test

Same user gets unrelated pseudonyms across spaces.

---

# 562. Join Token Replay Test

Single-use invite cannot create duplicate memberships.

---

# 563. Remove/Ban Test

Future key access denied.

---

# 564. Group History Test

New member receives only allowed history.

---

# 565. Moderator Scope Test

Moderator cannot act outside space/channel.

---

# 566. Role Escalation Test

Moderator cannot self-promote to owner/admin.

---

# 567. Managed Space Test

Org admin controls managed space only.

---

# 568. Personal Space Test

Org admin cannot moderate personal group.

---

# 569. Federation Test

Remote domain cannot gain internal admin authority.

---

# 570. Subscription Test

Publisher cannot enumerate subscribers.

---

# 571. Analytics Test

No user→space membership graph emitted.

---

# 572. Deleted Space Test

Stale directory cache cannot resurrect listing.

---

# 573. Backup Test

Banned membership does not reactivate after restore.

---

# 574. Publicization Test

Private→public requires explicit authorized action.

---

# 575. Fuzzing

Fuzz:

```text
directory record
join capability
membership envelope
moderation command
federation social envelope
```

---

# 576. Property Tests

Properties:

```text
private space never becomes listed without explicit authorized transition
banned/removed membership cannot decrypt future group epochs
moderator authority is subset of social-space scope
public directory output never contains private roster state
```

---

# 577. Formal Verification Targets

Strong candidates:

```text
join/leave/remove/ban state machine
role/capability lattice
roster visibility
private→public transition
```

---

# 578. Kani Candidate

role/capability/scope subset checks.

---

# 579. TLA+ Candidate

membership epoch + remove/ban + concurrent delivery.

---

# 580. Loom Candidate

concurrent join/leave/moderation/ownership update.

---

# 581. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. A public or listed social space never implies a public member roster.
2. Membership identities are scoped to the social space where possible and are not globally correlatable account identifiers.
3. Group/community/channel membership never automatically creates pairwise personal contacts.
4. Directory records contain discovery metadata only and never become membership or moderation authority.
5. Join capabilities are scoped, expiring, replay-resistant, and cannot grant unrelated administrative authority.
6. Moderation roles compile to scoped capabilities; moderators cannot affect accounts, contacts, groups, or tenants outside their explicit scope.
7. Removed or banned members cannot derive future private-group message keys after the relevant membership/key epoch advances.
8. Private social-space content remains E2EE and moderation never relies on a universal server-side plaintext decryption key.
9. Enterprise/tenant administrators control only managed social spaces and cannot inspect or govern unrelated personal communities by default.
10. Social-space telemetry, search, moderation logs, and analytics never create a remotely queryable user-to-group membership graph.
11. Federation social spaces preserve independent domain identity/authority and do not create a global member namespace or transitive moderator power.
12. Deleted/private/revoked space state cannot be resurrected through stale directory caches, backups, offline clients, or old capabilities.
```

---

# 582. Initial Production Scope

Implement first:

```text
typed SocialSpaceId/CommunityId/ChannelId
private/invite/exact/listed/public visibility
signed directory projections
group/community/channel models
join capability + request/approval flow
scoped MembershipId/pseudonym
membership state machine
full/members/hidden/moderator-only roster modes
role-to-capability moderation
ban/remove/suspend flows
membership epochs and group-key integration
announcement channel subscriptions
public/private history policy
tenant/managed space isolation
basic federation gateway integration
privacy-safe metrics/audit
social-space testkit
```

Then add:

```text
advanced hidden-roster protocols
reviewed anonymous-membership credentials
large-scale encrypted pub/sub fanout
federated community directory
privacy-preserving popularity/trending
formal membership/moderation verification
```

---

# 583. Definition of Done

Part 86 is complete when:

- group/community/channel identities are explicit and opaque
- public/private/listed/invite-only discovery states are separate
- directory metadata is not membership authority
- member rosters can remain hidden
- scoped pseudonyms prevent cross-space correlation
- join/leave/remove/ban state machines are durable
- roles compile to scoped moderator capabilities
- private content stays E2EE without universal moderation escrow
- large/public channels can support private subscriptions
- group membership never creates personal contact edges
- enterprise/federation scopes stay isolated
- directory/search/analytics never expose global membership graphs
- stale cache/backup/capability cannot resurrect deleted/revoked state
- join/moderation/roster/federation/fuzz/formal tests are specified

---

# 584. Final Architecture

```text
              DIRECTORY / INVITE / HANDLE
                        │
                        ▼
                 SOCIAL SPACE
        ┌───────────────┼───────────────┐
        │               │               │
      GROUP         COMMUNITY        CHANNEL
        │               │               │
        └───────────────┼───────────────┘
                        ▼
                 MEMBERSHIP STATE
                        │
             ┌──────────┼──────────┐
             │          │          │
           ROLES      ROSTER    KEY EPOCH
             │          │          │
             └──────────┼──────────┘
                        ▼
               MODERATION BOUNDARY
```

Social-space safety model:

```text
scoped membership identity
+
separate directory projection
+
capability-based joins
+
hidden roster options
+
scoped moderation authority
+
membership epochs
+
E2EE private content
+
no global social graph
```

not:

```text
put every group, member, moderator, subscriber, and participation event into one centralized people graph
```

---

# 585. Final Principle

A social space should expose only the information necessary for people to discover and participate in that space, not the identities and relationships of everyone inside it.

The correct model is:

```text
discover metadata
+
join through scoped capability
+
identify members locally/scoped
+
moderate within boundaries
+
hide membership by default
+
federate explicitly
+
never centralize participation graphs
```

This architecture gives SIAR a privacy-preserving foundation for groups, communities, channels, announcement systems, membership discovery, moderation, public directories, tenant spaces, and federated social spaces while preserving the anonymity, local-first, least-authority, and social-graph privacy guarantees established across Parts 34–85.
