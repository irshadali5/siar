# Core System Architecture Part 84 — Anonymous Network Profile, Preferences, Settings, Personalization, Synchronization & Privacy-Preserving User-State Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 84  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 30–33, 45, 56–57, 59–61, 67–69, 74–83  

**Primary purpose:** define SIAR's profile, preference, settings, personalization, synchronization, and privacy-preserving user-state architecture, including local/device/account/tenant scopes, discoverability, contact-specific presentation, managed-policy overlays, offline-first synchronization, conflict resolution, encrypted user-state replication, backup/export integration, deletion, and strict separation from transport/routing identity.

---

# 1. Purpose

User-state is deceptively sensitive.

A typical profile/settings system can reveal:

```text
identity
language
timezone
device habits
contacts
presence preferences
privacy choices
notification behavior
UI preferences
account relationships
```

The governing principle is:

> **Profile and preference state must be local-first, scope-explicit, minimally synchronized, privacy-aware, and incapable of silently becoming routing, telemetry, or social-graph metadata.**

---

# 2. Architectural Position

```text
User Intent
   │
   ▼
Scoped Preference Model
   │
   ├── Local Device State
   ├── Account-Synced State
   ├── Contact-Scoped State
   └── Managed Policy Overlay
   │
   ▼
Privacy-Preserving Sync
   │
   ▼
Effective User State
```

---

# 3. Core Separation

Keep distinct:

```text
profile
preferences
device settings
privacy settings
managed policy
contact-specific presentation
synchronization metadata
```

---

# 4. Non-Goals

Part 84 does not:

```text
make one global profile document
sync every setting automatically
treat user preferences as authorization
infer preferences from surveillance
use personalization state as routing metadata
```

---

# 5. User-State Scope

```rust
pub enum UserStateScope {
    DeviceLocal,
    AccountSynced,
    ContactScoped,
    ConversationScoped,
    TenantManaged,
    OrganizationManaged,
}
```

---

# 6. DeviceLocal

Examples:

```text
window layout
theme
font scale
download location
local cache size
```

---

# 7. AccountSynced

Examples:

```text
display name
avatar reference
language preference
notification defaults
privacy defaults
```

---

# 8. ContactScoped

Examples:

```text
nickname
custom notification
visibility override
```

---

# 9. ConversationScoped

Examples:

```text
mute
pinned
draft behavior
```

---

# 10. TenantManaged

Organization policy overlay.

---

# 11. OrganizationManaged

Managed profile configuration.

---

# 12. Hard Rule

Every preference has an explicit scope.

---

# 13. Preference Key

```rust
pub struct PreferenceKey(pub u32);
```

---

# 14. Typed Preference

```rust
pub struct Preference<T> {
    pub key: PreferenceKey,
    pub scope: UserStateScope,
    pub value: T,
    pub version: PreferenceVersion,
}
```

---

# 15. Preference Version

```rust
pub struct PreferenceVersion(pub u64);
```

---

# 16. No Stringly Typed Settings In Domain Layer

Preferred hard rule.

---

# 17. Profile Model

```rust
pub struct UserProfile {
    pub display_name: Option<DisplayName>,
    pub avatar: Option<AvatarRef>,
    pub bio: Option<ProfileBio>,
    pub visibility: ProfileVisibility,
}
```

---

# 18. Display Name

User-controlled.

---

# 19. Not Account Identity

Hard rule.

---

# 20. Avatar

Encrypted/reference-based where appropriate.

---

# 21. Profile Bio

Bounded length.

---

# 22. Profile Visibility

```rust
pub enum ProfileVisibility {
    Private,
    ContactsOnly,
    ExactHandle,
    Public,
}
```

---

# 23. Default

`Private`.

---

# 24. No Automatic Public Profile

Hard rule.

---

# 25. Profile Publication

Explicit.

---

# 26. Public Profile

Separate projection.

---

# 27. Private Profile

Not globally discoverable.

---

# 28. Contact-Specific Presentation

Important for privacy.

---

# 29. Example

User may expose:

```text
nickname
avatar
status
```

differently to different contacts.

---

# 30. Contact Profile Projection

```rust
pub struct ContactProfileProjection {
    pub contact: RelationshipId,
    pub display_name: Option<DisplayName>,
    pub avatar: Option<AvatarRef>,
    pub status_visibility: StatusVisibility,
}
```

---

# 31. No One Global Profile View Required

Hard rule.

---

# 32. Contact Scoped Visibility

Can reduce disclosure.

---

# 33. No Cross-Contact Inference Through Profile APIs

Hard rule.

---

# 34. Profile Identity

Must not expose:

```text
AccountId
transport ID
mailbox ID
device ID
```

---

# 35. Hard Rule

Profile IDs are presentation-layer constructs.

---

# 36. Profile Record ID

```rust
pub struct ProfileRecordId(pub [u8; 32]);
```

---

# 37. Opaque.

---

# 38. Preferences Model

Categories:

```rust
pub enum PreferenceCategory {
    Appearance,
    Notifications,
    Privacy,
    Accessibility,
    DataUsage,
    Media,
    Security,
    Sync,
}
```

---

# 39. Appearance

DeviceLocal or AccountSynced.

---

# 40. Notifications

Mixed scope.

---

# 41. Privacy

Usually AccountSynced with stricter local overrides.

---

# 42. Accessibility

Can be DeviceLocal by default.

---

# 43. DataUsage

DeviceLocal/network-context-specific.

---

# 44. Media

DeviceLocal + account defaults.

---

# 45. Security

AccountSynced only where safe.

---

# 46. Sync

AccountSynced.

---

# 47. Hard Rule

Security-critical device secrets are never stored as user preferences.

---

# 48. Effective Preference

Multiple layers.

---

# 49. Layers

```text
platform hard floor
managed policy
account preference
device override
conversation/contact override
```

---

# 50. Precedence

More restrictive privacy/security setting wins.

---

# 51. Effective Preference Trait

```rust
pub trait EffectivePreferenceResolver {
    fn resolve<T>(
        &self,
        key: PreferenceKey,
        context: PreferenceContext,
    ) -> Result<T, PreferenceError>;
}
```

---

# 52. Managed Overlay

Can restrict.

---

# 53. Cannot weaken network/privacy floor.

---

# 54. User Device Override

Can be stricter.

---

# 55. Hard Rule

Managed policy cannot silently weaken personal privacy.

---

# 56. Personal vs Managed Profile

Separate.

---

# 57. Managed Profile

Organization-controlled subset.

---

# 58. Personal Profile

User-controlled.

---

# 59. No Shared Preference Namespace

Hard rule.

---

# 60. Managed Preference Key

```rust
pub struct ManagedPreferenceKey(pub u32);
```

---

# 61. Separate type.

---

# 62. Profile Synchronization

Local-first.

---

# 63. Local Write

Immediate.

---

# 64. Remote Sync

Asynchronous.

---

# 65. No Network Requirement For Local Preference Change

Hard rule.

---

# 66. Syncable Preference

Explicit declaration.

---

# 67. Sync Classification

```rust
pub enum SyncClass {
    LocalOnly,
    AccountSynced,
    ContactSynced,
    Managed,
}
```

---

# 68. No Automatic Sync Of New Preference Type

Hard rule.

---

# 69. Why

Prevents accidental sensitive state replication.

---

# 70. Sync Registry

```rust
pub struct PreferenceSyncDescriptor {
    pub key: PreferenceKey,
    pub sync_class: SyncClass,
    pub conflict_policy: PreferenceConflictPolicy,
    pub sensitivity: PreferenceSensitivity,
}
```

---

# 71. Sensitivity

```rust
pub enum PreferenceSensitivity {
    Public,
    Private,
    Sensitive,
    SecretLike,
}
```

---

# 72. SecretLike

Should usually be LocalOnly.

---

# 73. Hard Rule

SecretLike preferences cannot be synced by default.

---

# 74. Sync Payload

Minimal.

---

# 75. Sync Envelope

```rust
pub struct PreferenceSyncEnvelope {
    pub state_id: UserStateId,
    pub version: PreferenceVersion,
    pub payload: EncryptedPreferencePayload,
}
```

---

# 76. User State ID

Opaque.

---

# 77. No AccountId In Wire Payload If Not Needed

Hard rule.

---

# 78. Encryption

Synced private preferences encrypted.

---

# 79. Server sees ciphertext where feasible.

---

# 80. Sync Key

Account/device-scoped.

---

# 81. No Global Preferences Encryption Key

Hard rule.

---

# 82. Per-Account/User Key

Preferred.

---

# 83. Contact-Scoped Sync

Use relationship-scoped encryption where needed.

---

# 84. Sync Protocol

Versioned.

---

# 85. Postcard preferred internally.

---

# 86. RON human-readable for export/config.

---

# 87. JSON only external interop.

---

# 88. Conflict Resolution

Domain-specific.

---

# 89. Conflict Policies

```rust
pub enum PreferenceConflictPolicy {
    LastWriterWinsAllowed,
    MonotonicRestriction,
    SetUnion,
    SetIntersection,
    LocalWins,
    ManualResolution,
}
```

---

# 90. LastWriterWinsAllowed

Only low-risk preferences.

---

# 91. MonotonicRestriction

For privacy/security.

---

# 92. Example

If one device turns off read receipts, do not silently re-enable from stale device.

---

# 93. Hard Rule

Privacy-sensitive conflicts must not resolve toward weaker privacy by default.

---

# 94. SetUnion

Useful for benign lists.

---

# 95. SetIntersection

Useful for restriction sets.

---

# 96. LocalWins

Device-specific state.

---

# 97. ManualResolution

Rare.

---

# 98. Clock Use

Avoid wall-clock ordering alone.

---

# 99. Prefer version/vector metadata.

---

# 100. State Version

```rust
pub struct UserStateVersion {
    pub device: DeviceId,
    pub counter: u64,
}
```

---

# 101. Vector Clock

Can be used for small sync domains.

---

# 102. Or causal metadata.

---

# 103. No Global Lamport Clock Across All User State

Hard rule.

---

# 104. Per-Domain Counters

Better.

---

# 105. Offline Edits

Supported.

---

# 106. Merge On Reconnect.

---

# 107. No Data Loss From Simple LWW For Sensitive State.

---

# 108. Preference State Machine

```rust
pub enum PreferenceSyncState {
    LocalOnly,
    PendingSync,
    Synced,
    Conflict,
    Rejected,
}
```

---

# 109. Rejected

Policy violation or stale version.

---

# 110. Conflict

User may need resolution.

---

# 111. Sync Outbox

Transactional.

---

# 112. Local Preference Write

```text
write state
→ write sync outbox
→ commit
→ sync later
```

---

# 113. Hard Rule

No remote-first save.

---

# 114. Sync Inbox

Idempotent.

---

# 115. Duplicate payload

No duplicate effect.

---

# 116. Device Identity

Used for sync provenance.

---

# 117. Not public.

---

# 118. New Device

Starts from account-synced preference snapshot.

---

# 119. DeviceLocal preferences

Not copied.

---

# 120. Hard Rule

Device-specific state is not restored blindly to different device classes.

---

# 121. Device Class

Can affect defaults.

---

# 122. Example

Desktop layout vs mobile layout.

---

# 123. Adaptive Defaults

Local.

---

# 124. No Device Fingerprint Stored Centrally

Hard rule.

---

# 125. Personalization

Explicit user-controlled personalization preferred.

---

# 126. Examples

```text
theme
font size
notification behavior
media autoplay
```

---

# 127. Behavioral Personalization

High privacy risk.

---

# 128. Avoid by default.

---

# 129. Hard Rule

Do not infer persistent personalization from message/content behavior without explicit opt-in and local processing.

---

# 130. Local Personalization

Allowed.

---

# 131. Example

Frequently used emoji ranking.

---

# 132. Keep local.

---

# 133. No centralized behavioral profile.

---

# 134. Hard rule.

---

# 135. Recommendation Engine

Not core requirement.

---

# 136. If added

Local/on-device.

---

# 137. Sync only user-approved outputs.

---

# 138. Not raw behavior history.

---

# 139. Profile Discoverability

Separate from login identity.

---

# 140. Public Handle

Part 59.

---

# 141. Profile Lookup

By explicit handle/capability.

---

# 142. No email/phone reverse lookup by default.

---

# 143. Hard Rule

Address-book data is not uploaded raw.

---

# 144. Profile Search

Exact handle or explicit capability.

---

# 145. No wildcard enumeration.

---

# 146. Profile Enumeration Resistance

Rate limit + exact lookup.

---

# 147. Profile Visibility Token

Capability.

---

# 148. Public profile

Opt-in.

---

# 149. Contact profile

Relationship scoped.

---

# 150. No one profile endpoint exposing all account metadata.

---

# 151. Avatar Storage

Encrypted blob store.

---

# 152. Public avatar

Published projection.

---

# 153. Private avatar

Encrypted/restricted.

---

# 154. Avatar Reference

```rust
pub struct AvatarRef {
    pub blob: BlobId,
    pub variant: AvatarVariant,
}
```

---

# 155. No cross-user dedup in strict privacy mode.

---

# 156. Avatar Metadata

No EXIF.

---

# 157. Hard Rule

Strip sensitive image metadata before publication.

---

# 158. Profile Image Processing

Local or privacy-safe service.

---

# 159. No upload of original if user only wants local avatar.

---

# 160. Status Message

Optional.

---

# 161. Status Visibility

```rust
pub enum StatusVisibility {
    Hidden,
    Contacts,
    SelectedContacts,
    Public,
}
```

---

# 162. Presence != Status Message.

---

# 163. Keep separate.

---

# 164. No automatic status based on activity.

---

# 165. Hard rule.

---

# 166. Privacy Preferences

Examples:

```text
read receipts
typing
presence
profile visibility
link previews
external fetch
```

---

# 167. Privacy Defaults

Conservative.

---

# 168. Maximum anonymity

Many ephemerals disabled.

---

# 169. Preference Type

```rust
pub struct PrivacyPreference {
    pub feature: PrivacyFeature,
    pub state: PrivacyPreferenceState,
}
```

---

# 170. Privacy State

```rust
pub enum PrivacyPreferenceState {
    Disabled,
    Enabled,
    Ask,
}
```

---

# 171. Managed Policy

Can disable features.

---

# 172. Cannot force privacy-weaker setting below network floor.

---

# 173. Hard rule.

---

# 174. Notification Preferences

Multi-scope.

---

# 175. Global default.

---

# 176. Contact/conversation override.

---

# 177. Device-specific delivery channel.

---

# 178. No server need to know local sound/vibration settings.

---

# 179. Hard Rule

Only synchronize semantic notification intent, not all OS-specific details.

---

# 180. Example

Sync:

```text
mute conversation
priority contacts
```

Local only:

```text
sound file
vibration pattern
```

---

# 181. Accessibility Preferences

Mostly local.

---

# 182. Some account sync:

```text
text size preference
reduce motion
```

---

# 183. But device accessibility APIs may override.

---

# 184. Local OS preference wins.

---

# 185. Data Usage Preferences

Device/network-specific.

---

# 186. Examples

```text
download on Wi-Fi
media autoplay
background sync
```

---

# 187. Keep local by default.

---

# 188. No central record of network type habits.

---

# 189. Hard rule.

---

# 190. Security Preferences

Examples:

```text
app lock
auto-lock duration
security notifications
```

---

# 191. App lock

DeviceLocal.

---

# 192. Security notification preference

AccountSynced.

---

# 193. Recovery method

Not a preference.

---

# 194. Passkey

Not a preference.

---

# 195. Hard rule.

---

# 196. UI/UX Settings

DeviceLocal.

---

# 197. Dioxus desktop.

---

# 198. Android Compose.

---

# 199. Shared semantic setting schema.

---

# 200. Platform-specific rendering separate.

---

# 201. No shared widget state.

---

# 202. Profile Storage

Local authoritative copy.

---

# 203. Server synchronized copy

Encrypted account projection.

---

# 204. Public projection

Separate.

---

# 205. Hard Rule

Public profile is not same document as private profile.

---

# 206. Why

Avoid accidental field exposure.

---

# 207. PublicProfileProjection

```rust
pub struct PublicProfileProjection {
    pub handle: Option<PublicHandle>,
    pub display_name: Option<DisplayName>,
    pub avatar: Option<PublicAvatarRef>,
    pub bio: Option<PublicProfileBio>,
}
```

---

# 208. No private fields.

---

# 209. Contact Projection

Separate.

---

# 210. Account Projection

Separate.

---

# 211. Projection Compiler

```rust
pub trait ProfileProjectionCompiler {
    fn public_projection(
        &self,
        profile: &UserProfile,
        policy: &ProfileVisibilityPolicy,
    ) -> PublicProfileProjection;
}
```

---

# 212. No dynamic reflection-based field exposure.

---

# 213. Hard rule.

---

# 214. Profile Update

Transactional local write.

---

# 215. Server sync async.

---

# 216. Public projection publish after local commit.

---

# 217. Revocation

If visibility reduced, public projection removed.

---

# 218. Hard Rule

Privacy reduction applies before performance optimization.

---

# 219. Deletion

Part 83.

---

# 220. Profile deletion

Removes synchronized/private/public projections.

---

# 221. Contact-local nicknames on other users remain theirs.

---

# 222. No remote deletion claim.

---

# 223. Hard truth.

---

# 224. Account Deactivation

Public discoverability removed or hidden.

---

# 225. Suspension

Policy-specific.

---

# 226. Security Lock

Profile still may be visible depending policy.

---

# 227. No automatic publication change without policy.

---

# 228. Backup

Part 33.

---

# 229. Preferences included according to scope.

---

# 230. DeviceLocal

Usually not in portable backup unless user chooses.

---

# 231. AccountSynced

Portable.

---

# 232. Managed policy

Not backed up as user-owned state.

---

# 233. Hard Rule

Managed policy is re-fetched from authority, not restored from user backup.

---

# 234. Export

Human-readable.

---

# 235. RON/JSON external.

---

# 236. SecretLike settings excluded or redacted.

---

# 237. Hard Rule

Export cannot leak secret-like local state.

---

# 238. Import

Validates scope/type.

---

# 239. No import of managed policy.

---

# 240. No import of session/recovery secrets.

---

# 241. Hard rule.

---

# 242. Schema Version

Preference schema versioned.

---

# 243. Preference Schema

```rust
pub struct PreferenceSchemaVersion(pub u16);
```

---

# 244. Unknown Preference

Preserve if safe or ignore.

---

# 245. Security-sensitive unknown

Fail closed.

---

# 246. Migration

Part 74.

---

# 247. Preference migration

Pure/deterministic.

---

# 248. No network call during migration.

---

# 249. Policy Migration

Separate.

---

# 250. User State Repository

```rust
pub trait UserStateRepository {
    fn get<T>(
        &self,
        key: PreferenceKey,
        scope: UserStateScope,
    ) -> Result<Option<T>, PreferenceError>;

    fn set<T>(
        &self,
        preference: Preference<T>,
    ) -> Result<(), PreferenceError>;
}
```

---

# 251. Sync Repository

```rust
pub trait UserStateSyncRepository {
    fn enqueue(
        &self,
        envelope: PreferenceSyncEnvelope,
    ) -> Result<(), PreferenceError>;
}
```

---

# 252. No direct network in repository.

---

# 253. Sync Engine

Async.

---

# 254. Local conflict evaluator.

---

# 255. Privacy Policy Integration

Part 56.

---

# 256. Privacy preference changes recorded locally.

---

# 257. No unnecessary central audit.

---

# 258. User can inspect effective privacy settings.

---

# 259. Effective source visible.

---

# 260. Example:

```text
Read receipts: Disabled
Source: User preference
```

---

# 261. Managed overlay transparent.

---

# 262. No hidden policy.

---

# 263. Hard rule.

---

# 264. Authorization Integration

Part 81.

---

# 265. Preferences do not grant authority.

---

# 266. Example

"share publicly" preference cannot bypass capability requirement.

---

# 267. Hard Rule

Preference is intent/config, not authorization.

---

# 268. Authentication Integration

Part 82.

---

# 269. Profile changes may require step-up for sensitive fields.

---

# 270. Example

public handle/email/login alias.

---

# 271. UI cosmetic changes

No step-up.

---

# 272. Account Lifecycle Integration

Part 83.

---

# 273. Deleted profile cannot sync.

---

# 274. Deactivated profile restricted.

---

# 275. Suspension profile-specific.

---

# 276. No stale device can republish deleted profile.

---

# 277. Hard rule.

---

# 278. Deletion Epoch

User-state sync respects.

---

# 279. Sync envelope includes lifecycle version.

---

# 280. Old sync rejected.

---

# 281. Tenant/Managed Integration

Part 69.

---

# 282. Managed overlay scoped.

---

# 283. Personal settings preserved.

---

# 284. No org overwrite of personal state.

---

# 285. Hard rule.

---

# 286. Federation

No profile federation by default.

---

# 287. Remote domain sees explicit profile projection only.

---

# 288. No raw internal preference sync across federation.

---

# 289. Hard rule.

---

# 290. Contact Profile Federation

Relationship capability.

---

# 291. No transitive profile sharing.

---

# 292. Discovery

Part 59/77.

---

# 293. Profile discoverability controlled separately.

---

# 294. Public handle does not imply public profile.

---

# 295. Hard rule.

---

# 296. Telemetry

Minimal.

---

# 297. Safe metrics:

```text
sync success
conflict count
schema migration failures
```

---

# 298. Forbidden:

```text
user theme choice
privacy preference values
contact-specific settings
```

---

# 299. Hard rule.

---

# 300. No personalization analytics by default.

---

# 301. Observability Class

```rust
pub enum UserStateTelemetryClass {
    SafeAggregate,
    LocalOnly,
    Forbidden,
}
```

---

# 302. Preference Values

Usually `Forbidden`.

---

# 303. Sync Lag

Aggregate.

---

# 304. No per-user preference timeline.

---

# 305. Audit

Only managed/admin changes.

---

# 306. Personal preference changes

No central audit by default.

---

# 307. Managed policy change

Audit policy version, not user preference content.

---

# 308. Hard rule.

---

# 309. Sync Failure Modes

```text
offline device
stale update
conflict
account deletion
key rotation
server unavailable
```

---

# 310. Offline Device

Queue locally.

---

# 311. Stale Update

Conflict or reject.

---

# 312. Account Deleted

Reject.

---

# 313. Key Rotation

Rewrap/rekey sync state.

---

# 314. Server Unavailable

Local state continues.

---

# 315. No preference change blocked by server unless managed policy requires validation.

---

# 316. Hard rule.

---

# 317. Conflict UX

Only where needed.

---

# 318. Most low-risk conflicts auto-resolve.

---

# 319. Privacy conflicts resolve restrictive.

---

# 320. Managed policy conflicts resolve managed restriction.

---

# 321. Local device conflicts local wins.

---

# 322. User visibility

Show source if non-obvious.

---

# 323. Profile Sync Privacy

Server should not learn every local tweak.

---

# 324. Batch/compact state changes.

---

# 325. No detailed change timeline.

---

# 326. Snapshot Sync

Useful.

---

# 327. Delta Sync

Useful.

---

# 328. For sensitive preferences

Encrypted deltas.

---

# 329. Change Frequency Leakage

Can still leak.

---

# 330. Batch/jitter where needed.

---

# 331. Not overcomplicate low-risk state.

---

# 332. Max-Anonymity Mode

Prefer delayed/batched profile sync.

---

# 333. Presence/typing unaffected.

---

# 334. Profile changes not immediate if privacy mode asks.

---

# 335. No sync timing fingerprint.

---

# 336. Hard rule for strict mode where feasible.

---

# 337. Preference Sync Scheduler

Battery-aware.

---

# 338. Part 13.

---

# 339. Android Doze

Respect.

---

# 340. Desktop immediate when online.

---

# 341. No wake-up solely for cosmetic preference.

---

# 342. Hard rule.

---

# 343. Avatar Upload

Background/bulk priority.

---

# 344. Privacy changes

Higher priority.

---

# 345. Security settings

High priority.

---

# 346. Sync Priority

```rust
pub enum UserStateSyncPriority {
    Security,
    Privacy,
    Normal,
    Cosmetic,
}
```

---

# 347. Cosmetic first to defer.

---

# 348. Privacy/security not starved.

---

# 349. Storage

Encrypted embedded DB.

---

# 350. Server state

Encrypted account-state store.

---

# 351. Public profile

separate published projection store.

---

# 352. No single table mixing private/public profile.

---

# 353. Hard rule.

---

# 354. Search Index

Public handle/profile only if opted in.

---

# 355. Contact profile local.

---

# 356. No global profile index for private users.

---

# 357. Hard rule.

---

# 358. Personalization Engine

Optional local service.

---

# 359. Input

Local UI interaction.

---

# 360. Output

Local preference suggestions.

---

# 361. No background upload.

---

# 362. User must explicitly accept persistent change.

---

# 363. Hard rule.

---

# 364. Machine Learning

Not required.

---

# 365. If used

On-device.

---

# 366. No raw behavior cloud training by default.

---

# 367. Hard rule.

---

# 368. Recommendation State

Rebuildable/local.

---

# 369. Not part of account source of truth.

---

# 370. Security Settings Examples

```text
auto-lock
show security alerts
allow link previews
external fetch
```

---

# 371. External Fetch

Privacy-sensitive.

---

# 372. Default disabled in max-anonymity.

---

# 373. Link Preview

Local/proxied depending mode.

---

# 374. Preference must not silently cause direct external request.

---

# 375. Hard rule.

---

# 376. Timezone/Locale

Locale may sync.

---

# 377. Timezone

Potential fingerprint.

---

# 378. Keep local unless user wants cross-device sync.

---

# 379. No precise locale combination sent to anonymous peers.

---

# 380. Hard rule.

---

# 381. Language Preference

UI only.

---

# 382. Does not leak to message protocol unless content language itself.

---

# 383. Formatting

Local.

---

# 384. Date/time display

Local.

---

# 385. No remote formatting metadata.

---

# 386. Crate Layout

Recommended:

```text
crates/
├── siar-profile-core/
├── siar-profile-projection/
├── siar-preference-core/
├── siar-preference-policy/
├── siar-user-state-sync/
├── siar-user-state-crypto/
├── siar-user-state-conflict/
├── siar-managed-preferences/
├── siar-user-state-observability/
└── siar-user-state-testkit/
```

---

# 387. `siar-profile-core`

Owns:

```text
display name
avatar
bio
visibility
```

---

# 388. `siar-profile-projection`

Public/contact/account projections.

---

# 389. `siar-preference-core`

Typed preference keys/values/scopes.

---

# 390. `siar-preference-policy`

Precedence/effective setting resolution.

---

# 391. `siar-user-state-sync`

Outbox/inbox/offline sync.

---

# 392. `siar-user-state-crypto`

Encrypted sync payloads/key rotation.

---

# 393. `siar-user-state-conflict`

Conflict resolution policies.

---

# 394. `siar-managed-preferences`

Tenant/org managed overlays.

---

# 395. `siar-user-state-observability`

Aggregate sync metrics.

---

# 396. `siar-user-state-testkit`

Offline/conflict/deletion/privacy tests.

---

# 397. Error Taxonomy

```rust
pub enum PreferenceError {
    UnknownPreference,
    ScopeMismatch,
    ManagedPolicyViolation,
    PrivacyFloorViolation,
    Conflict,
    StaleVersion,
    SyncRejected,
    AccountInactive,
    AccountDeleted,
    EncryptionFailure,
    Internal,
}
```

---

# 398. Testing

Need comprehensive user-state testkit.

---

# 399. Test Scenarios

```text
offline edit
conflicting privacy setting
managed restriction
account deletion
new device sync
```

---

# 400. Offline Edit Test

Local save succeeds.

---

# 401. Conflict Test

Privacy resolves restrictive.

---

# 402. Cosmetic LWW Test

Allowed.

---

# 403. Managed Policy Test

Cannot weaken org/security floor.

---

# 404. Personal Profile Test

Org policy cannot overwrite personal profile.

---

# 405. New Device Test

Account-synced state restored, DeviceLocal not.

---

# 406. Deletion Test

Deleted account rejects stale sync.

---

# 407. Deletion Barrier Test

Old backup cannot republish profile.

---

# 408. Public Projection Test

Private fields never appear.

---

# 409. Contact Projection Test

One contact's projection not visible to another.

---

# 410. Federation Test

Remote domain receives only explicit projection.

---

# 411. Avatar Metadata Test

EXIF stripped.

---

# 412. Sync Key Rotation Test

State remains decryptable after rewrap/rekey.

---

# 413. Max-Anonymity Test

Profile sync batching/delay policy honored.

---

# 414. Telemetry Test

Preference values absent from logs/metrics.

---

# 415. Fuzzing

Fuzz:

```text
preference envelope
profile projection
sync metadata
conflict state
```

---

# 416. Property Tests

Properties:

```text
public projection is subset of private profile
managed policy can restrict but not expand privacy
DeviceLocal preferences never enter account sync
deleted account state never reappears through stale sync
```

---

# 417. Formal Verification Targets

Strong candidates:

```text
preference precedence
privacy conflict resolution
profile projection safety
deletion anti-resurrection
```

---

# 418. Kani Candidate

scope/precedence/subset rules.

---

# 419. TLA+ Candidate

offline multi-device sync + deletion epoch.

---

# 420. Loom Candidate

concurrent local edit + remote sync + policy update.

---

# 421. Performance

User-state is low/medium throughput.

---

# 422. Local reads

Fast/in-memory cache.

---

# 423. Writes

Embedded DB transaction.

---

# 424. Sync

Batched.

---

# 425. Public projection

Small.

---

# 426. Avatar

Blob transfer.

---

# 427. No large profile payload.

---

# 428. Preference bundle

Compact.

---

# 429. No per-message preference lookup from remote service.

---

# 430. Cache effective settings locally.

---

# 431. Invalidate on policy/preference change.

---

# 432. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Every profile/preference value has an explicit scope: device, account, contact, conversation, tenant, or organization.
2. Device-local state never synchronizes unless explicitly declared syncable.
3. Public, contact-scoped, account-private, and managed profile projections are separate typed objects; private fields cannot leak through reflection or shared serialization.
4. Profile/account identifiers never become transport, mailbox, routing, or anonymous protocol identifiers.
5. Privacy/security conflicts resolve toward the more restrictive effective state unless an explicit safe policy says otherwise.
6. Managed policy may restrict managed state but cannot silently weaken network privacy/security floors or overwrite unrelated personal state.
7. Personalization based on behavioral observation remains local by default and raw behavior histories are not uploaded for centralized profiling.
8. Synced private user-state is encrypted and does not use a global shared user-state key.
9. Deleted/deactivated account lifecycle state is enforced by sync, and stale devices/backups cannot republish deleted profile/preferences.
10. Preferences are intent/configuration only; they never grant authorization or override Part 81 capability policy.
11. User-state telemetry is aggregate and never includes preference values, contact-specific settings, or behavioral personalization data.
12. Local profile/preference changes remain functional offline; network synchronization is asynchronous and never required for basic local UX.
```

---

# 433. Initial Production Scope

Implement first:

```text
typed preference registry
explicit state scopes
private/public/contact profile projections
device-local vs account-synced split
transactional local user-state store
encrypted sync envelopes
outbox/inbox sync
restrictive privacy conflict policy
managed preference overlays
new-device state restore
account lifecycle/deletion epoch enforcement
avatar metadata stripping
privacy-safe sync metrics
user-state testkit
```

Then add:

```text
causal/vector preference sync
advanced contact-specific profile projection
local personalization engine
formal projection/conflict verification
cross-provider user-state portability
privacy-preserving batched strict-mode synchronization
```

---

# 434. Definition of Done

Part 84 is complete when:

- every user-state field has explicit scope
- local/device-only state never leaks into account sync
- public/private/contact/managed projections are separate
- sync is offline-first and transactional
- private sync payloads are encrypted
- conflicts are domain-specific
- privacy conflicts resolve restrictively
- managed overlays cannot overwrite unrelated personal state
- behavioral personalization remains local by default
- stale devices/backups cannot republish deleted state
- preferences never substitute for authorization
- telemetry excludes user preference values
- offline/conflict/projection/deletion/fuzz/formal tests are specified

---

# 435. Final Architecture

```text
                    USER INTENT
                        │
                        ▼
               TYPED SCOPED STATE
                        │
          ┌─────────────┼─────────────┐
          │             │             │
      DeviceLocal   AccountSynced  ContactScoped
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                POLICY / PRIVACY MERGE
                        │
                        ▼
                 EFFECTIVE SETTINGS
                        │
                        ▼
              ENCRYPTED ASYNC SYNC
```

User-state safety model:

```text
explicit scope
+
local-first state
+
separate profile projections
+
encrypted synchronization
+
restrictive privacy merge
+
managed overlay isolation
+
anti-resurrection
```

not:

```text
one giant cloud profile JSON document containing every preference and device detail
```

---

# 436. Final Principle

Profile and personalization state should improve the user experience without becoming a hidden identity graph or behavioral surveillance system.

The correct model is:

```text
keep local what can stay local
+
sync only declared state
+
project only what is intentionally shared
+
merge toward privacy
+
separate managed and personal control
+
never reuse user-state as routing identity
```

This architecture gives SIAR a privacy-preserving, local-first profile and preference foundation across devices, contacts, conversations, managed tenants, public profile projections, backups, and synchronized account state while preserving the anonymity and least-authority guarantees established across Parts 34–83.
