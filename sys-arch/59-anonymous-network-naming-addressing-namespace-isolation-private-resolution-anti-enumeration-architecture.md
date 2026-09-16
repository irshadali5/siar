# Core System Architecture Part 59 — Anonymous Network Naming, Addressing, Namespace Isolation, Private Resolution & Anti-Enumeration Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 59  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–58  

**Primary purpose:** define the naming and addressing architecture for SIAR, including namespace isolation, private name resolution, anti-enumeration, per-scope pseudonyms, capability-based addressing, handle/privacy tradeoffs, address rotation, federation naming, provider/service naming, collision handling, migration, and strict separation from transport, billing, and global identity.

---

# 1. Purpose

Naming is a privacy boundary.

A globally searchable namespace can reveal:

```text
who exists
who belongs to which domain
which services they use
which group names exist
which providers are active
```

A globally stable identifier can become:

```text
tracking handle
billing join key
contact graph key
cross-service correlation key
```

The governing principle is:

> **SIAR should use the least-global identifier necessary for each purpose, prefer scoped capabilities over public names, and make enumeration difficult by design.**

---

# 2. Architectural Position

```text
Human Name / Handle
        │
        ▼
Namespace Scope
        │
        ▼
Private Resolver / Capability Exchange
        │
        ▼
Scoped Address
        │
        ▼
Transport / Mailbox / Service Endpoint
```

---

# 3. Core Separation

Keep distinct:

```text
display name
public handle
account authority
relationship ID
group pseudonym
mailbox address
transport endpoint
provider ID
federation domain
billing identity
```

---

# 4. Non-Goals

Part 59 does not require:

```text
one global username
one global user directory
global people search
public list of all groups
global mapping from handle to account key
```

---

# 5. Identifier Classes

```rust
pub enum IdentifierClass {
    DisplayName,
    PublicHandle,
    PrivateHandle,
    ScopedPseudonym,
    CapabilityAddress,
    ServiceAddress,
    ProviderAddress,
    FederationDomain,
}
```

---

# 6. Display Name

Human-readable.

Not unique.

---

# 7. Public Handle

Potentially searchable.

Higher privacy cost.

---

# 8. Private Handle

Resolvable only with exact knowledge or authorization.

---

# 9. Scoped Pseudonym

Stable only inside a scope.

---

# 10. Capability Address

Opaque random bearer-style address.

Preferred for strict privacy.

---

# 11. Service Address

Used for:

```text
mailbox
relay
bulk provider
bridge
federation gateway
```

---

# 12. Provider Address

Provider operational identity/address.

Separate from users.

---

# 13. Namespace Scope

```rust
pub enum NamespaceScope {
    LocalDevice,
    Relationship,
    Group,
    Community,
    Domain,
    Federation,
    Public,
}
```

---

# 14. Scope Principle

Use smallest scope that satisfies requirement.

---

# 15. LocalDevice

Examples:

```text
local labels
local aliases
```

---

# 16. Relationship

Per-contact pseudonym.

---

# 17. Group

Group-local pseudonym.

---

# 18. Community

Community-local handle.

---

# 19. Domain

Federation-domain scoped name.

---

# 20. Federation

Rare.

Should not imply globally unique identity.

---

# 21. Public

Highest privacy cost.

---

# 22. Namespace ID

```rust
pub struct NamespaceId(pub [u8; 32]);
```

---

# 23. Scoped Name

```rust
pub struct ScopedName {
    pub namespace: NamespaceId,
    pub value: String,
}
```

---

# 24. No Cross-Namespace Equality Assumption

Hard rule.

Same text in two namespaces does not imply same entity.

---

# 25. Handle Policy

```rust
pub enum HandleVisibility {
    Private,
    ExactSearch,
    Public,
}
```

---

# 26. Private

Not resolvable through network search.

---

# 27. ExactSearch

Resolvable only with exact handle.

---

# 28. Public

Discoverable/listable if policy explicitly allows.

---

# 29. Default Recommendation

```text
ExactSearch
```

for optional user handles.

---

# 30. Maximum Anonymity

Prefer:

```text
capability bootstrap
```

over handle lookup.

---

# 31. Display Name ≠ Handle

Hard rule.

---

# 32. Handle ≠ Account Authority

Hard rule.

---

# 33. Handle Resolution

Returns bootstrap descriptor, not raw account identity.

---

# 34. Resolution Result

```rust
pub struct PrivateResolutionResult {
    pub bootstrap: ContactBootstrapDescriptor,
    pub assurance: ResolutionAssurance,
}
```

---

# 35. Contact Bootstrap Descriptor

Contains only what is needed to establish relationship.

---

# 36. No Full Profile Dump

Hard rule.

---

# 37. Resolution Assurance

```rust
pub enum ResolutionAssurance {
    ExactMatch,
    SignedDirectoryMatch,
    ManagedDirectoryMatch,
}
```

---

# 38. Private Resolution

A resolver should not support:

```text
prefix scan
substring scan
wildcard scan
```

in privacy-preserving mode.

---

# 39. Exact-Match Only

Hard rule for private lookup.

---

# 40. Resolver Interface

```rust
#[async_trait]
pub trait PrivateNameResolver: Send + Sync {
    async fn resolve_exact(
        &self,
        query: ExactNameQuery,
    ) -> Result<Option<PrivateResolutionResult>, ResolutionError>;
}
```

---

# 41. Exact Name Query

```rust
pub struct ExactNameQuery {
    pub namespace: NamespaceId,
    pub normalized_name: NormalizedHandle,
}
```

---

# 42. Normalization

Must be deterministic.

---

# 43. Unicode

Need:

```text
normalization
confusable detection
case handling
```

---

# 44. Handle Normalization

Use explicit profile.

---

# 45. Example

```text
NFKC-like normalization
case folding where appropriate
```

Exact rules must be versioned.

---

# 46. Confusable Handling

Warn for visually similar names.

---

# 47. Do Not Auto-Merge Confusables

Hard rule.

---

# 48. Homograph Attack

Example:

```text
latin a
cyrillic а
```

---

# 49. UI

Show security warning where needed.

---

# 50. Handle Version

```rust
pub struct HandleNormalizationVersion(pub u16);
```

---

# 51. Anti-Enumeration

Core requirement.

---

# 52. Threats

```text
brute-force handles
dictionary attacks
prefix scans
timing oracle
existence oracle
rate abuse
```

---

# 53. Response Normalization

Existing and non-existing handles should have similar:

```text
latency
response size
```

where practical.

---

# 54. Rate Limiting

Required.

---

# 55. Anti-Enumeration Controls

Possible:

```text
token bucket
proof-of-work
anonymous quota token
blind lookup
OPRF
PSI
```

---

# 56. Initial Production

Use:

```text
exact match
rate limit
high-entropy namespaces
response padding
```

---

# 57. Future

Use reviewed:

```text
OPRF/PSI
```

for stronger privacy.

---

# 58. No Raw Address-Book Upload

Hard rule.

---

# 59. Low-Entropy Identifiers

Phone/email are easy to guess.

---

# 60. Simple Hash Is Not Private

Hard rule.

---

# 61. If Phone/Email Discovery Added

Need:

```text
OPRF/PSI
or
trusted managed directory
```

not simple unsalted hash lookup.

---

# 62. Resolver Privacy

Resolver should learn minimum query information.

---

# 63. Query Blinding

Future option.

---

# 64. Local Cache

Can reduce resolver use.

---

# 65. Cache Privacy

Local-only.

---

# 66. Negative Cache

Short TTL.

---

# 67. Why

Avoid creating permanent record of searched names.

---

# 68. Search History

Local-only.

---

# 69. No Remote Search History

Hard rule.

---

# 70. Public Handles

Trade privacy for convenience.

---

# 71. Public Handle Policy

Explicit user opt-in.

---

# 72. Public Handle Risks

```text
enumeration
spam
cross-platform correlation
```

---

# 73. Public Handle UI

Warn:

```text
This makes you easier to find.
```

---

# 74. Handle Rotation

Supported.

---

# 75. Handle Rotation Does Not Change Relationship

Hard rule.

---

# 76. Relationship Identity

Stable local relationship capability.

---

# 77. Old Handle

Should stop resolving after grace/redirect policy.

---

# 78. Redirect

Privacy risk.

---

# 79. Handle Redirect Policy

```rust
pub enum HandleRedirectPolicy {
    None,
    Temporary,
    Permanent,
}
```

---

# 80. Maximum Privacy

Prefer:

```text
None
```

---

# 81. Temporary Redirect

Can ease migration.

---

# 82. Redirect TTL

Short.

---

# 83. No Permanent Global Handle History

Hard rule.

---

# 84. Namespace Collision

Two entities request same handle.

---

# 85. Collision Policy

Namespace-specific.

---

# 86. Private Random Handle

Collision negligible.

---

# 87. Human Handle

Need registration.

---

# 88. Handle Registration

```rust
pub struct HandleRegistration {
    pub namespace: NamespaceId,
    pub handle: NormalizedHandle,
    pub owner_proof: HandleOwnershipProof,
    pub expires_at: Option<Timestamp>,
}
```

---

# 89. Owner Proof

Scoped.

---

# 90. No Global Account Proof Exposure

Hard rule.

---

# 91. Handle Squatting

Possible.

---

# 92. Policy

Could use:

```text
expiry
activity renewal
managed namespace
```

---

# 93. No Auction Required

Hard rule.

---

# 94. Namespace Authority

Different namespaces can have different issuers.

---

# 95. Namespace Authority Descriptor

```rust
pub struct NamespaceAuthorityDescriptor {
    pub namespace: NamespaceId,
    pub authority_key: NamespaceAuthorityKey,
    pub policy: NamespacePolicy,
}
```

---

# 96. Namespace Policy

```rust
pub struct NamespacePolicy {
    pub visibility: HandleVisibility,
    pub normalization: HandleNormalizationVersion,
    pub registration: RegistrationPolicy,
    pub anti_enumeration: AntiEnumerationPolicy,
}
```

---

# 97. Registration Policy

```rust
pub enum RegistrationPolicy {
    Open,
    InviteOnly,
    Managed,
    CapabilityOnly,
}
```

---

# 98. Public User Namespace

If ever offered:

```text
Open + ExactSearch
```

more privacy-preserving than directory listing.

---

# 99. Private Community Namespace

Could be:

```text
Managed
```

---

# 100. Group Naming

Group display name need not be globally unique.

---

# 101. Group Address

Use opaque group capability.

---

# 102. Group Discovery

Invite/capability-based.

---

# 103. No Global Group Search By Default

Hard rule.

---

# 104. Public Community

Can publish group discovery separately.

---

# 105. Group Address

```rust
pub struct GroupAddress {
    pub group_transport_id: AnonymousGroupTransportId,
    pub join_capability: Option<GroupJoinCapability>,
}
```

---

# 106. No Raw GroupId To Provider

Hard rule.

---

# 107. Mailbox Addressing

Mailbox IDs are random/rotatable.

---

# 108. Mailbox Address

```rust
pub struct MailboxAddress {
    pub provider: AnonymousProviderId,
    pub opaque_mailbox_id: OpaqueMailboxId,
    pub capability: MailboxCapabilityRef,
}
```

---

# 109. Mailbox ID

Not human-readable.

---

# 110. Mailbox Rotation

Supported.

---

# 111. Old Mailbox Address

Expires/revokes.

---

# 112. No Global Mailbox Directory

Hard rule.

---

# 113. Transport Addressing

Transport endpoint IDs are ephemeral/technical.

---

# 114. No Human Naming Of Transport Endpoint

Hard rule.

---

# 115. Transport ID Rotation

Normal.

---

# 116. Provider Naming

Providers need stable public identity.

---

# 117. Provider ID

Cryptographic.

---

# 118. Provider Display Name

Human-readable.

---

# 119. Provider Name ≠ Operator ID

Hard rule.

---

# 120. Operator Name

Organization/business label.

---

# 121. Provider Namespace

Catalog-controlled.

---

# 122. Service Endpoint Address

Can rotate independently.

---

# 123. Federation Domain Naming

Human domain label optional.

---

# 124. FederationDomainId

Cryptographic canonical identifier.

---

# 125. Human Domain Name

Example:

```text
example.community
```

or custom label.

---

# 126. Human Name Not Trust Anchor

Hard rule.

---

# 127. Trust Uses

```text
pinned domain ID
trust bundle
```

---

# 128. Federation Address

```rust
pub struct FederatedAddress {
    pub domain: FederationDomainId,
    pub destination: OpaqueFederatedCapability,
}
```

---

# 129. Optional Handle@Domain

Convenience.

---

# 130. Privacy Risk

Stable globally recognizable.

---

# 131. Strict Mode

Prefer invite/capability.

---

# 132. Inter-Domain Resolver

Resolves:

```text
exact handle + domain
```

---

# 133. No Global Cross-Domain People Search

Hard rule.

---

# 134. Domain Resolver Interface

```rust
#[async_trait]
pub trait FederatedNameResolver: Send + Sync {
    async fn resolve(
        &self,
        domain: FederationDomainId,
        handle: NormalizedHandle,
    ) -> Result<Option<FederatedResolutionResult>, ResolutionError>;
}
```

---

# 135. Resolver Trust

Validate domain trust first.

---

# 136. No Lookup To Untrusted Domain

Strict mode.

---

# 137. Resolver Routing

Can use anonymous transport.

---

# 138. Why

Lookup itself reveals interest.

---

# 139. Private Resolver Access

Use mixnet/mailbox-capable channel where practical.

---

# 140. Resolver Query Metadata

Minimize.

---

# 141. No Source Account ID

Hard rule.

---

# 142. Resolver Quota Credential

Scoped anonymous token.

---

# 143. Resolver Abuse

Rate limit.

---

# 144. Resolver Availability

Can have mirrors.

---

# 145. Resolver Mirror

Trust signed response.

---

# 146. Signed Resolution Record

```rust
pub struct SignedNameRecord {
    pub namespace: NamespaceId,
    pub handle: NormalizedHandle,
    pub bootstrap: ContactBootstrapDescriptor,
    pub valid_until: Timestamp,
    pub signature: NamespaceSignature,
}
```

---

# 147. TTL

Short enough for rotation.

---

# 148. Cache

Client caches until TTL.

---

# 149. Revocation

Can revoke name record.

---

# 150. Revocation Feed

Scoped per namespace.

---

# 151. Handle Transfer

Dangerous.

---

# 152. Recommendation

Do not allow silent transfer.

---

# 153. Transfer Flow

```text
old owner signs release
new owner registers
contacts see security change
```

---

# 154. Recycled Handles

Privacy risk.

---

# 155. Cooldown

Recommended.

---

# 156. Handle Reuse Policy

```rust
pub struct HandleReusePolicy {
    pub cooldown: Duration,
    pub require_security_warning: bool,
}
```

---

# 157. Long Cooldown

Reduces impersonation.

---

# 158. Security Notice

Old contacts should not automatically trust recycled handle.

---

# 159. Handle Verification

Handle resolution ≠ identity verification.

---

# 160. Hard rule.

---

# 161. Verified Contact

Part 45 separate.

---

# 162. Handle Lookup

Only bootstraps.

---

# 163. Namespace Privacy Modes

```rust
pub enum NamespacePrivacyMode {
    CapabilityOnly,
    ExactLookup,
    PublicDirectory,
}
```

---

# 164. CapabilityOnly

Strongest.

---

# 165. ExactLookup

Balanced.

---

# 166. PublicDirectory

Highest discoverability.

---

# 167. Maximum Anonymity

Default:

```text
CapabilityOnly
```

---

# 168. Standard

Could allow ExactLookup.

---

# 169. User Choice

Explicit.

---

# 170. Anti-Enumeration Policy

```rust
pub struct AntiEnumerationPolicy {
    pub exact_match_only: bool,
    pub rate_limit: RateLimitPolicy,
    pub response_padding: bool,
    pub proof_required: Option<LookupProofPolicy>,
}
```

---

# 171. Lookup Proof Policy

```rust
pub enum LookupProofPolicy {
    AnonymousQuota,
    ProofOfWork,
    ManagedCredential,
}
```

---

# 172. Proof-of-Work

Use cautiously due battery/accessibility.

---

# 173. Anonymous Quota

Preferred where available.

---

# 174. Managed Credential

Enterprise/private directory.

---

# 175. Timing Side Channel

Resolver response latency should be normalized.

---

# 176. Size Side Channel

Pad responses.

---

# 177. Existence Side Channel

Cannot eliminate entirely in exact lookup.

---

# 178. But

Reduce bulk enumeration.

---

# 179. Decoy/Private Information Retrieval

Future.

---

# 180. OPRF-Based Resolution

Potential.

---

# 181. Important

Use reviewed constructions only.

---

# 182. No Custom Crypto

Hard rule.

---

# 183. Private Set Intersection

Potential for contact discovery.

---

# 184. Resolver Logging

Do not log raw queries.

---

# 185. Hard rule.

---

# 186. Safe Resolver Metrics

```text
query count
error rate
rate-limit hits
```

---

# 187. Forbidden Resolver Metrics

No:

```text
queried handle
source identity
query-to-result mapping
```

---

# 188. Short-Lived Security Logs

If attack mitigation requires:

```text
coarse source network
```

use shortest possible retention.

---

# 189. No Search Analytics

Hard rule.

---

# 190. Naming Privacy Dashboard

Can show:

```text
Public handle: Off
Exact lookup: On
Capability-only contacts: 14
```

locally.

---

# 191. Do Not Upload Dashboard State

Hard rule.

---

# 192. Handle Change UX

Explain:

```text
existing contacts remain connected
old handle stops resolving
```

---

# 193. Public-to-Private Change

Should be immediate for new discovery.

---

# 194. Private-to-Public Change

Requires explicit confirmation.

---

# 195. Public Directory Enrollment

Separate consent.

---

# 196. Name Exposure Receipt

Optional local record.

---

# 197. Naming Policy Change

Versioned.

---

# 198. Namespace Policy Version

```rust
pub struct NamespacePolicyVersion(pub u64);
```

---

# 199. Anti-Rollback

Required.

---

# 200. Client Stores

Highest accepted policy version per namespace.

---

# 201. Resolver Policy

Signed.

---

# 202. Namespace Governance

Part 53 integration.

---

# 203. Managed Namespace

Organization authority.

---

# 204. Federation Namespace

Domain authority.

---

# 205. Public Namespace

Potential SIAR governance authority.

---

# 206. No One Authority For All Namespaces

Hard rule.

---

# 207. Namespace Delegation

Possible.

---

# 208. Delegation Record

```rust
pub struct NamespaceDelegation {
    pub parent: NamespaceId,
    pub child: NamespaceId,
    pub authority: NamespaceAuthorityKey,
    pub valid_until: Timestamp,
    pub signature: NamespaceSignature,
}
```

---

# 209. Delegation Is Not User Trust

Hard rule.

---

# 210. Namespace Tree

Optional.

---

# 211. Avoid DNS-Style Global Hierarchy Requirement

Hard rule.

---

# 212. Flat Cryptographic Namespace

Can coexist.

---

# 213. Self-Certifying Names

Useful for services/providers.

---

# 214. Example

Hash/public-key-derived provider IDs.

---

# 215. Human Alias

Can map to self-certifying ID.

---

# 216. Alias Trust

Alias resolution signed.

---

# 217. Service Discovery

Part 48 uses provider IDs/catalog.

---

# 218. Naming Layer

Adds human alias only.

---

# 219. No Provider Lookup By User Identity

Hard rule.

---

# 220. Naming and Payments

Handle must never be wallet ID.

---

# 221. Hard rule.

---

# 222. Naming and Moderation

Reputation subject ID must be separate.

---

# 223. Hard rule.

---

# 224. Naming and Telemetry

Handle never appears in operational telemetry.

---

# 225. Hard rule.

---

# 226. Naming and Recovery

Recovery locator separate from handle.

---

# 227. Hard rule.

---

# 228. Naming and Federation

Federated address can include domain ID + capability.

---

# 229. Public Handle@Domain

Optional convenience only.

---

# 230. Naming Data Model

Potential tables:

```text
local_aliases
registered_handles
namespace_policies
resolver_cache
handle_history_local
namespace_revocations
```

---

# 231. Local Alias

Never synced unless user chooses.

---

# 232. Registered Handle

Server-side minimal record.

---

# 233. Resolver Cache

Local.

---

# 234. Handle History

Local only.

---

# 235. Server Must Not Keep Historical Handle Graph

Hard rule unless explicit policy requires short cooldown metadata.

---

# 236. Cooldown Store

Can store salted/opaque registration key.

---

# 237. Not public history.

---

# 238. Address Rotation

Important.

---

# 239. Rotatable Types

```text
mailbox address
presence address
transport address
nearby beacon
federation session address
```

---

# 240. Less-Rotatable Types

```text
public handle
provider ID
federation domain ID
```

---

# 241. Rotation Class

```rust
pub enum AddressRotationClass {
    PerSession,
    ShortEpoch,
    LongEpoch,
    StablePublic,
}
```

---

# 242. Address Registry

Local.

---

# 243. Address Lifetime

Explicit.

---

# 244. Expired Address

Must fail safely.

---

# 245. No Automatic Fallback To Stable Address

Hard rule.

---

# 246. Capability Expiry

Address capability expires.

---

# 247. Replay Protection

Required.

---

# 248. Address Binding

Signed/authenticated.

---

# 249. Address Binding Record

```rust
pub struct AddressBinding {
    pub scoped_subject: ScopedSubjectRef,
    pub address: CapabilityAddress,
    pub epoch: u64,
    pub expires_at: Timestamp,
    pub proof: AddressBindingProof,
}
```

---

# 250. Provider Must Not Learn ScopedSubjectRef

Use encrypted/opaque form where provider-facing.

---

# 251. Name Resolution State Machine

```rust
pub enum ResolutionState {
    QueryPrepared,
    Submitted,
    Resolved,
    NotFound,
    RateLimited,
    Failed,
}
```

---

# 252. Name Registration State

```rust
pub enum RegistrationState {
    Available,
    Pending,
    Active,
    Rotating,
    Revoked,
    Expired,
}
```

---

# 253. Naming Service Trait

```rust
pub trait NamingService {
    fn normalize(
        &self,
        raw: &str,
        version: HandleNormalizationVersion,
    ) -> Result<NormalizedHandle, NamingError>;

    fn validate(
        &self,
        handle: &NormalizedHandle,
        policy: &NamespacePolicy,
    ) -> Result<(), NamingError>;
}
```

---

# 254. Registration Service

```rust
#[async_trait]
pub trait HandleRegistrationService: Send + Sync {
    async fn register(
        &self,
        request: HandleRegistrationRequest,
    ) -> Result<HandleRegistration, NamingError>;

    async fn revoke(
        &self,
        namespace: NamespaceId,
        handle: NormalizedHandle,
    ) -> Result<(), NamingError>;
}
```

---

# 255. Resolver Policy Service

```rust
pub trait ResolverPolicyService {
    fn policy(
        &self,
        namespace: NamespaceId,
    ) -> Result<NamespacePolicy, NamingError>;
}
```

---

# 256. Address Rotation Service

```rust
pub trait AddressRotationService {
    fn rotate(
        &self,
        class: AddressRotationClass,
    ) -> Result<AddressRotationReport, NamingError>;
}
```

---

# 257. Naming Threat Model

Threats:

```text
enumeration
homograph
handle squatting
recycled handle impersonation
resolver logging
timing oracle
cross-namespace correlation
address replay
malicious namespace authority
```

---

# 258. Cross-Namespace Correlation

Same user chooses same handle everywhere.

---

# 259. Mitigation

Warn user.

---

# 260. Strict Mode

Recommend independent handles/pseudonyms.

---

# 261. Automatic Random Alias

Can generate per namespace.

---

# 262. No Forced Same Handle

Hard rule.

---

# 263. Resolver Compromise

Could observe exact queries.

---

# 264. Mitigation

Anonymous transport + future blind lookup.

---

# 265. Namespace Authority Compromise

Can forge records.

---

# 266. Response

Revoke authority/policy.

---

# 267. Contact Verification

Still protects identity.

---

# 268. Handle Record Compromise

Does not reveal message keys.

---

# 269. Naming Privacy SLOs

Examples:

```text
zero raw query logging
zero handle in telemetry
zero global user namespace requirement
```

---

# 270. Naming Availability SLOs

Examples:

```text
exact resolution availability
registration availability
policy freshness
```

---

# 271. Observability

Safe:

```text
query counts
not-found rate
rate-limit count
registration count
```

---

# 272. Minimum Cohorts

For public metrics.

---

# 273. No Per-Handle Metrics

Hard rule.

---

# 274. Support Bundle

Can include:

```text
namespace policy version
resolution error class
```

---

# 275. Exclude

```text
queried handles
registered private handles
capability addresses
```

---

# 276. Testkit

Need naming/enumeration simulator.

---

# 277. Scenarios

```text
exact lookup
dictionary attack
prefix scan
timing measurement
homograph
handle rotation
recycled handle
federated resolution
resolver compromise
```

---

# 278. Exact Lookup Test

Correct record resolves.

---

# 279. Prefix Enumeration Test

Unsupported.

---

# 280. Wildcard Enumeration Test

Unsupported.

---

# 281. Rate-Limit Test

Repeated guessing throttled.

---

# 282. Timing Test

Existing/non-existing response distributions sufficiently similar.

---

# 283. Response Size Test

Padded classes.

---

# 284. Homograph Test

Confusable warning.

---

# 285. Handle Rotation Test

Existing relationship survives.

---

# 286. Old Handle Test

Stops resolving after policy.

---

# 287. Recycled Handle Test

Old verification does not carry over.

---

# 288. Cross-Namespace Test

Same text maps to unrelated scoped names.

---

# 289. Federation Test

handle@domain resolves only after domain trust check.

---

# 290. Private Resolver Test

No raw source account ID.

---

# 291. Fuzzing

Fuzz:

```text
handle normalization
namespace policy
signed name record
address binding
federated address
```

---

# 292. Property Tests

Properties:

```text
DisplayName never implies identity uniqueness
handle lookup never returns raw account secret
same handle text in distinct namespaces is not same identifier
expired capability address cannot authorize new operation
```

---

# 293. Formal Verification Targets

Strong candidates:

```text
namespace scope separation
handle rotation state
address expiry/replay
resolver policy precedence
```

---

# 294. Kani Candidate

Normalization/version/scoping invariants.

---

# 295. TLA+ Candidate

Handle rotation/revocation/cache convergence.

---

# 296. Loom Candidate

Concurrent handle rotation + resolution cache.

---

# 297. Performance Tests

Measure:

```text
normalization
exact lookup
cache hit
signature verification
```

---

# 298. Scale Tests

Synthetic namespaces:

```text
100k
10M
100M
```

records.

---

# 299. Large Directory Strategy

Need:

```text
sharding
private lookup
```

without public enumeration.

---

# 300. No Prefix Index Exposure

Hard rule.

---

# 301. Crate Layout

Recommended:

```text
crates/
├── siar-naming-core/
├── siar-namespace/
├── siar-handle-normalization/
├── siar-handle-registration/
├── siar-private-resolution/
├── siar-anti-enumeration/
├── siar-capability-address/
├── siar-federated-naming/
├── siar-address-rotation/
├── siar-naming-observability/
└── siar-naming-testkit/
```

---

# 302. `siar-naming-core`

Owns:

```text
identifier classes
namespace scopes
errors
```

---

# 303. `siar-namespace`

Namespace authority/policy/versioning.

---

# 304. `siar-handle-normalization`

Unicode normalization/confusable checks.

---

# 305. `siar-handle-registration`

Registration/rotation/revocation.

---

# 306. `siar-private-resolution`

Exact-match resolver.

---

# 307. `siar-anti-enumeration`

Rate limit/padding/proof policies.

---

# 308. `siar-capability-address`

Opaque address types/lifetimes/replay protection.

---

# 309. `siar-federated-naming`

handle@domain / federated address resolution.

---

# 310. `siar-address-rotation`

Ephemeral address lifecycle.

---

# 311. `siar-naming-observability`

Privacy-safe metrics.

---

# 312. `siar-naming-testkit`

Enumeration/homograph/rotation simulator.

---

# 313. Error Taxonomy

```rust
pub enum NamingError {
    InvalidHandle,
    ConfusableHandle,
    NamespaceUnknown,
    NamespacePolicyStale,
    HandleUnavailable,
    ResolutionRateLimited,
    ResolutionDenied,
    RecordExpired,
    SignatureInvalid,
    AddressExpired,
    AddressReplay,
    FederationDomainUntrusted,
    Internal,
}
```

---

# 314. Security & Privacy Invariants

Mandatory:

```text
1. SIAR does not require a single global user namespace.
2. Display names are never treated as unique identity.
3. Handles are scoped to namespaces and are separate from account, transport, payment, and moderation identifiers.
4. Private resolution supports exact-match lookup rather than prefix/wildcard enumeration.
5. Resolver logs and telemetry never contain raw private queries or result mappings.
6. Simple hashes of low-entropy identifiers are not treated as private discovery.
7. Capability-based addressing is preferred in strict anonymity modes.
8. Mailbox, presence, transport, and federation session addresses are rotatable and replay-protected.
9. Foreign/federated name resolution occurs only after domain trust validation.
10. Handle transfer/reuse never inherits old identity verification.
11. Namespace policy and signed name records are versioned and anti-rollback protected.
12. No naming identifier serves as a universal cross-service correlation key.
```

---

# 315. Initial Production Scope

Implement first:

```text
scoped namespaces
display-name/handle separation
ExactSearch handles
capability-only bootstrap
signed name records
rate-limited exact resolver
response padding classes
Unicode/confusable validation
handle rotation/revocation
short-lived mailbox/presence addresses
federated handle@domain resolution
no raw query logging
local search-history only
```

Then add:

```text
OPRF-based private lookup
PSI contact discovery
PIR-backed large directories
private federated resolver federation
advanced decoy/batch resolution
```

---

# 316. Definition of Done

Part 59 is complete when:

- user, group, provider, mailbox, transport, and federation naming are explicitly separated
- namespace scope is machine-readable
- no global user namespace is required
- public/exact/private handle visibility modes are defined
- capability addresses are first-class
- handle normalization/confusable policy is versioned
- exact-match resolution and anti-enumeration controls are explicit
- simple-hash low-entropy discovery is rejected
- handle rotation/reuse/transfer semantics are defined
- mailbox/transport/presence/federation addresses rotate independently
- federated resolution validates domain trust first
- resolver telemetry and support data exclude private queries
- naming policy is signed and anti-rollback protected
- enumeration, homograph, replay, rotation, federation, fuzz, and formal tests are specified

---

# 317. Final Architecture

```text
                    HUMAN-FACING NAME
                           │
                           ▼
                    NAMESPACE SCOPE
                           │
             ┌─────────────┴─────────────┐
             │                           │
        Exact Handle               Capability Address
             │                           │
             ▼                           ▼
       Private Resolver             Direct Bootstrap
             │                           │
             └─────────────┬─────────────┘
                           ▼
                   SCOPED ADDRESS BINDING
                           │
                           ▼
              MAILBOX / GROUP / SERVICE / PEER
```

Naming safety model:

```text
scoped namespaces
+
exact/private resolution
+
anti-enumeration
+
rotatable capability addresses
+
federation-aware trust
```

not:

```text
one global username directory for every identity and service
```

---

# 318. Final Principle

Names should help humans find the right entity without becoming permanent global tracking handles.

The correct model is:

```text
human-readable aliases where needed
+
scoped namespaces
+
private exact resolution
+
capability bootstrap
+
rotatable technical addresses
```

This architecture gives SIAR a naming and addressing system that supports usability, federation, and discovery while preserving identity separation and resisting enumeration across the anonymous network.
