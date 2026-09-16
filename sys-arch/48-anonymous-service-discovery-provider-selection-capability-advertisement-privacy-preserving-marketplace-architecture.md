# Core System Architecture Part 48 — Anonymous Service Discovery, Provider Selection, Capability Advertisement & Privacy-Preserving Marketplace Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 48  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–47  

**Primary purpose:** define how SIAR discovers, validates, filters, ranks, rotates, and selects anonymous infrastructure providers while preserving privacy, preventing centralized usage correlation, integrating capability/price/health/reputation metadata, and keeping provider discovery independent from routing, payments, and user identity.

---

# 1. Purpose

As SIAR grows, anonymous services may be offered by multiple providers:

```text
mixnet gateways
mailbox providers
bulk-storage providers
realtime relays
bridges
directory mirrors
censorship-resistance transports
anonymous file rendezvous services
```

A naive provider marketplace can create a powerful tracking point.

Example failure:

```text
user account
→ searches providers
→ chooses relay
→ pays provider
→ uses mailbox
→ fetches files
```

all through one centralized service.

The governing principle is:

> **Provider discovery and selection must be based on signed, privacy-preserving capability metadata and local policy evaluation, not on identity-linked per-user marketplace queries.**

---

# 2. Architectural Position

```text
Signed Provider Catalogs
         │
         ▼
Provider Discovery Layer
         │
         ▼
Eligibility / Privacy Filters
         │
         ▼
Health / Diversity / Price Evaluation
         │
         ▼
Local Provider Selection
         │
         ▼
Service-Specific Provider Adapter
```

---

# 3. Core Separation

Keep separate:

```text
Provider Identity
Operator Identity
Capability Advertisement
Price Advertisement
Provider Health
User Identity
Payment Identity
Routing Identity
Usage Identity
```

---

# 4. Non-Goals

Part 48 does not require:

```text
centralized app-store marketplace
public user profiles
operator social network
token speculation
provider bidding per packet
auctioning individual user requests
```

---

# 5. Provider Types

```rust
pub enum AnonymousServiceType {
    MixnetGateway,
    Mailbox,
    BulkStorage,
    RealtimeRelay,
    Bridge,
    DirectoryMirror,
    Rendezvous,
    CensorshipTransport,
}
```

---

# 6. Provider Identity

Each provider has a cryptographic provider ID:

```rust
pub struct AnonymousProviderId(pub [u8; 32]);
```

---

# 7. Operator Identity

Several providers may belong to one operator.

```rust
pub struct ProviderOperatorId(pub [u8; 32]);
```

---

# 8. Why Separate Provider and Operator

A single operator can run:

```text
gateway A
relay B
mailbox C
```

These must not automatically count as independent privacy domains.

---

# 9. Provider Descriptor

```rust
pub struct AnonymousProviderDescriptor {
    pub provider_id: AnonymousProviderId,
    pub operator_id: ProviderOperatorId,
    pub service_type: AnonymousServiceType,
    pub capabilities: ProviderCapabilities,
    pub pricing: Option<ProviderPriceRef>,
    pub health: ProviderHealthRef,
    pub policy: ProviderPolicyMetadata,
    pub valid_until: Timestamp,
    pub signature: ProviderDescriptorSignature,
}
```

---

# 10. Provider Descriptor Principle

Descriptor should describe:

```text
what service exists
what capabilities it supports
what limits apply
```

not:

```text
which users use it
```

---

# 11. Capability Advertisement

Capabilities must be explicit.

---

# 12. Provider Capabilities

```rust
pub enum ProviderCapabilities {
    Mixnet(MixnetProviderCapabilities),
    Mailbox(MailboxProviderCapabilities),
    Bulk(BulkProviderCapabilities),
    Realtime(RealtimeProviderCapabilities),
    Bridge(BridgeProviderCapabilities),
    Mirror(MirrorProviderCapabilities),
}
```

---

# 13. Mixnet Capabilities

Example:

```rust
pub struct MixnetProviderCapabilities {
    pub protocol_versions: Vec<MixnetProtocolVersion>,
    pub provider_cover: bool,
    pub provider_loop: bool,
    pub max_packet_size: usize,
    pub supports_reply_capabilities: bool,
}
```

---

# 14. Mailbox Capabilities

```rust
pub struct MailboxProviderCapabilities {
    pub max_object_size: u64,
    pub retention_classes: Vec<RetentionClass>,
    pub supports_multi_device: bool,
    pub supports_single_use_reply: bool,
}
```

---

# 15. Bulk Capabilities

```rust
pub struct BulkProviderCapabilities {
    pub max_object_size: u64,
    pub supports_resume: bool,
    pub supports_range_fetch: bool,
    pub supports_padding: bool,
    pub hides_sender_ip: bool,
    pub hides_receiver_ip: bool,
}
```

---

# 16. Realtime Capabilities

```rust
pub struct RealtimeProviderCapabilities {
    pub udp: bool,
    pub quic_datagram: bool,
    pub tcp_fallback: bool,
    pub supports_padding: bool,
    pub max_bitrate: u64,
}
```

---

# 17. Bridge Capabilities

```rust
pub struct BridgeProviderCapabilities {
    pub transports: Vec<PluggableTransportId>,
    pub probe_resistant: bool,
    pub tcp_only: bool,
    pub websocket: bool,
}
```

---

# 18. Capability Versioning

Every capability schema is versioned.

---

# 19. Unknown Capability

Ignore safely or reject if mandatory.

---

# 20. Provider Policy Metadata

May include:

```text
jurisdiction category
retention policy
logging policy
abuse policy
minimum software version
```

---

# 21. Policy Honesty

Claims are only metadata until verified/audited.

---

# 22. Trust Classes

```rust
pub enum ProviderTrustClass {
    Unknown,
    Registered,
    Verified,
    Audited,
}
```

---

# 23. No "Trusted Forever"

Trust status must expire/revalidate.

---

# 24. Provider Catalog

```rust
pub struct ProviderCatalog {
    pub version: ProviderCatalogVersion,
    pub generated_at: Timestamp,
    pub valid_until: Timestamp,
    pub providers: Vec<AnonymousProviderDescriptor>,
    pub signatures: CatalogSignatureBundle,
}
```

---

# 25. Catalog Distribution

Possible:

```text
directory authorities
signed mirrors
offline bundles
peer-assisted signed transfer
```

---

# 26. Mirror Trust

Clients trust:

```text
catalog signature
```

not mirror.

---

# 27. Catalog Privacy

All clients should ideally fetch:

```text
same broad catalog
```

rather than query exact desired provider category per user.

---

# 28. Why

Per-user query patterns can reveal:

```text
needs censorship bridge
needs mailbox
needs high-bandwidth relay
```

---

# 29. Broad Snapshot Model

Preferred initial design:

```text
download provider catalog
evaluate locally
```

---

# 30. Local Selection

Hard rule for strict mode.

---

# 31. Discovery API

```rust
pub trait ProviderCatalogSource {
    fn current_catalog(
        &self,
    ) -> Result<Arc<ProviderCatalog>, ProviderDiscoveryError>;
}
```

---

# 32. Marketplace Query Minimization

Avoid:

```text
GET /best-relay-for-user-X
```

---

# 33. Private Discovery Query

If catalog becomes too large, future options:

```text
PIR
OPRF-assisted lookup
bucketed anonymous queries
```

---

# 34. Initial Recommendation

Use signed local catalog until scale demands more advanced private lookup.

---

# 35. Provider Eligibility

Selection begins with hard constraints.

---

# 36. Eligibility Inputs

```text
service type
protocol version
privacy requirement
health floor
operator revocation
policy compatibility
capacity
price class
```

---

# 37. Provider Requirements

```rust
pub struct ProviderRequirements {
    pub service_type: AnonymousServiceType,
    pub privacy_mode: PrivacyRoutingMode,
    pub required_capabilities: RequiredCapabilities,
    pub maximum_price: Option<ResourcePrice>,
    pub health_floor: ProviderHealthClass,
}
```

---

# 38. Hard Constraints First

Never let:

```text
lower price
lower latency
higher capacity
```

override:

```text
privacy requirement
protocol compatibility
revocation
operator diversity
```

---

# 39. Provider Filter

```rust
pub trait ProviderEligibilityFilter {
    fn eligible(
        &self,
        descriptor: &AnonymousProviderDescriptor,
        requirements: &ProviderRequirements,
    ) -> bool;
}
```

---

# 40. Privacy Requirement

Example:

```text
Anonymous bulk transfer
```

requires:

```text
hides sender IP
hides receiver IP
```

---

# 41. Strict Selection Failure

If no provider satisfies requirements:

```text
service unavailable
```

not weaker provider.

---

# 42. Selection Inputs After Eligibility

Then optimize among eligible providers by:

```text
health
operator diversity
capacity
price
latency class
historical reliability
```

---

# 43. Provider Score

Internal only.

---

# 44. Score Structure

```rust
pub struct ProviderScore {
    pub privacy_fit: u16,
    pub health: u16,
    pub capacity: u16,
    pub cost: u16,
    pub latency: u16,
}
```

---

# 45. No Public Universal Provider Score

Hard rule.

---

# 46. Selection Strategy

```rust
pub enum ProviderSelectionStrategy {
    PrivacyFirst,
    Balanced,
    CostAware,
    PerformanceAware,
}
```

---

# 47. Maximum Anonymity

Always:

```text
PrivacyFirst
```

---

# 48. Provider Selector

```rust
pub trait AnonymousProviderSelector {
    fn select(
        &self,
        catalog: &ProviderCatalog,
        requirements: &ProviderRequirements,
        strategy: ProviderSelectionStrategy,
    ) -> Result<ProviderSelection, ProviderSelectionError>;
}
```

---

# 49. Provider Selection

```rust
pub struct ProviderSelection {
    pub primary: AnonymousProviderId,
    pub fallbacks: Vec<AnonymousProviderId>,
    pub policy_snapshot: ProviderSelectionPolicySnapshot,
}
```

---

# 50. Fallback Providers

All fallbacks must already satisfy hard constraints.

---

# 51. No Post-Failure Weakening

Hard rule.

---

# 52. Provider Rotation

Provider reuse can create linkability.

---

# 53. Rotation Policy

```rust
pub enum ProviderRotationPolicy {
    Stable,
    ShortEpoch,
    PerSession,
    Adaptive,
}
```

---

# 54. Stable

Lower churn.

Higher linkability.

---

# 55. ShortEpoch

Good default for privacy-sensitive services.

---

# 56. PerSession

Useful for:

```text
call relays
rendezvous
```

---

# 57. Adaptive

Rotate based on:

```text
health
privacy policy
operator concentration
```

---

# 58. Rotation Must Avoid Synchronization

All clients rotating simultaneously creates fingerprintable events.

---

# 59. Randomized Rotation Window

Use jitter.

---

# 60. Provider Stickiness

Some services need temporary stickiness.

Examples:

```text
mailbox retention
bulk upload session
realtime call
```

---

# 61. Session-Bound Provider

Rotate only after session ends.

---

# 62. Mailbox Provider Migration

Use Part 35 migration flow.

---

# 63. Bulk Provider Migration

Use Part 40 rules.

---

# 64. Realtime Relay Migration

Use Part 44 rules.

---

# 65. Operator Diversity

Provider selection should track operator.

---

# 66. Cross-Service Diversity

Avoid:

```text
same operator
=
gateway
mailbox
relay
bulk storage
```

where possible.

---

# 67. Why

One operator can correlate multiple service views.

---

# 68. Privacy Domain Graph

```rust
pub struct OperatorExposureGraph {
    pub operator: ProviderOperatorId,
    pub services: Vec<AnonymousServiceType>,
}
```

---

# 69. Exposure Policy

```rust
pub struct CrossServiceDiversityPolicy {
    pub max_services_per_operator: u8,
    pub require_independent_relay_and_mailbox: bool,
}
```

---

# 70. Maximum Anonymity

Prefer operator separation across:

```text
entry
mailbox
bulk
realtime
```

when available.

---

# 71. Geographic Diversity

Optional.

---

# 72. Jurisdiction Diversity

Can be policy input.

---

# 73. Avoid Exact User-Location-Based Ranking

Hard rule.

---

# 74. Latency Classes

Use coarse:

```text
Low
Medium
High
Unknown
```

---

# 75. Health Model

```rust
pub enum ProviderHealthClass {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
```

---

# 76. Health Inputs

```text
availability
queue depth
error rate
latency class
software freshness
capacity
```

---

# 77. Health Sources

```text
provider self-report
independent observers
client-local experience
```

---

# 78. Self-Report Is Not Sufficient

Hard rule.

---

# 79. Independent Health Evidence

Signed.

---

# 80. Client-Local Experience

Useful but private.

---

# 81. Local Health Cache

```rust
pub struct LocalProviderExperience {
    pub provider: AnonymousProviderId,
    pub success_rate_bucket: ExperienceBucket,
    pub latency_bucket: ExperienceBucket,
}
```

---

# 82. No Remote Upload Required

Hard rule.

---

# 83. Provider Reputation

Infrastructure reputation from Part 39 can feed selection.

---

# 84. Keep Separate from User Reputation

Hard rule.

---

# 85. Provider Reputation Inputs

```text
uptime
correct behavior
policy compliance
audit state
revocations
```

---

# 86. Price Integration

Part 47 signed price catalogs integrate here.

---

# 87. Provider Price Reference

```rust
pub struct ProviderPriceRef {
    pub catalog_version: u64,
    pub price_id: ProviderPriceId,
}
```

---

# 88. Price Selection Privacy

Evaluate locally.

---

# 89. No Per-User Dynamic Quote

Strict mode forbids unless privacy-preserving.

---

# 90. Price Classes

Could publish:

```text
Free
Low
Medium
High
```

or exact signed price.

---

# 91. Resource Credit Compatibility

Provider advertises accepted credit classes.

---

# 92. Payment Compatibility

```rust
pub struct PaymentCapabilities {
    pub accepted_credit_classes: Vec<ResourceClass>,
    pub supports_prepaid: bool,
    pub supports_refunds: bool,
}
```

---

# 93. Selection Constraint

If wallet lacks accepted credit, provider may be ineligible.

---

# 94. No Identity-Linked Account Requirement in Strict Mode

Hard rule.

---

# 95. Marketplace Architecture

Marketplace means:

```text
catalog
policy
selection
pricing
operator participation
```

not:

```text
social storefront tracking users
```

---

# 96. Marketplace Governance

Needs:

```text
provider admission
descriptor signing
policy requirements
revocation
dispute handling
```

---

# 97. Provider Admission

Can reuse Part 39 infrastructure governance.

---

# 98. Provider Admission Record

```rust
pub struct ProviderAdmissionRecord {
    pub provider_id: AnonymousProviderId,
    pub operator_id: ProviderOperatorId,
    pub service_type: AnonymousServiceType,
    pub state: ProviderAdmissionState,
}
```

---

# 99. Provider Admission State

```rust
pub enum ProviderAdmissionState {
    Pending,
    Eligible,
    Active,
    Suspended,
    Revoked,
    Retired,
}
```

---

# 100. Provider Descriptor Signing

Provider signs own descriptor.

Directory/governance signs inclusion.

---

# 101. Dual Trust

Client verifies:

```text
provider descriptor signature
catalog inclusion signature
```

---

# 102. Provider Self-Claim

Cannot self-claim:

```text
Audited
Verified
```

without governance evidence.

---

# 103. Audit Attestation

```rust
pub struct ProviderAuditAttestation {
    pub provider_id: AnonymousProviderId,
    pub auditor: AuditorId,
    pub audit_class: AuditClass,
    pub expires_at: Timestamp,
    pub signature: AuditSignature,
}
```

---

# 104. Audit Class

Examples:

```text
Security
Privacy
Operational
```

---

# 105. Audit Expiry

Required.

---

# 106. Policy Claims

Examples:

```text
no long-term logs
24h metadata retention
zero-access storage
```

---

# 107. Claim Verification

Architecture should distinguish:

```text
SelfDeclared
GovernanceVerified
ThirdPartyAudited
```

---

# 108. Claim Assurance

```rust
pub enum ClaimAssurance {
    SelfDeclared,
    Verified,
    Audited,
}
```

---

# 109. UI Honesty

Do not display self-declared policy as independently verified.

---

# 110. Marketplace Anti-Manipulation

Threats:

```text
fake reviews
fake uptime
fake capacity
self-dealing
operator clones
price manipulation
```

---

# 111. No Public User Review System Initially

Hard rule.

---

# 112. Why

Public reviews create:

```text
Sybil spam
usage leakage
identity correlation
```

---

# 113. Better Reputation Sources

Use:

```text
signed health evidence
audits
governance history
local client experience
```

---

# 114. Capacity Advertisement

Provider may advertise capacity class.

---

# 115. Capacity Class

```rust
pub enum ProviderCapacityClass {
    Small,
    Medium,
    Large,
    VeryLarge,
}
```

---

# 116. Capacity Verification

Independent probes where practical.

---

# 117. Selection Weight

Capacity can affect probability.

---

# 118. Weight Cap

Prevents one provider dominating.

---

# 119. Marketplace Concentration

Monitor:

```text
operator share
provider share
ASN share
cloud share
```

---

# 120. Concentration Policy

```rust
pub struct MarketplaceConcentrationPolicy {
    pub max_operator_share: f32,
    pub max_provider_share: f32,
    pub max_asn_share: f32,
}
```

---

# 121. Strict Mode

Hard caps where possible.

---

# 122. Bootstrap Market

Small provider set may violate diversity.

---

# 123. Bootstrap State

Expose:

```text
Reduced provider diversity
```

---

# 124. No Hidden Relaxation

Hard rule.

---

# 125. Provider Enumeration

Catalog itself may reveal all providers.

---

# 126. Is That a Problem?

For public infrastructure:

```text
usually acceptable
```

For bridges:

```text
not acceptable
```

---

# 127. Service-Specific Discovery Model

Public provider catalogs for:

```text
mailbox
relay
bulk
gateway
```

Restricted distribution for:

```text
bridges
censorship-sensitive endpoints
```

---

# 128. Bridge Discovery

Must remain Part 41 broker model.

---

# 129. Do Not Put Private Bridges in Public Catalog

Hard rule.

---

# 130. Directory Mirrors

Can be public.

---

# 131. Anonymous Search

Future marketplace may support query like:

```text
need bulk provider with 10 GB + padding
```

without revealing user.

---

# 132. Private Query Techniques

Potential:

```text
PIR
bucketed query
downloaded index shards
```

---

# 133. Initial Scale Strategy

Shard catalog by:

```text
service type
protocol version
```

not user.

---

# 134. Catalog Shard

```rust
pub struct ProviderCatalogShard {
    pub service_type: AnonymousServiceType,
    pub version: ProviderCatalogVersion,
    pub providers: Vec<AnonymousProviderDescriptor>,
    pub signature: CatalogSignatureBundle,
}
```

---

# 135. Shard Privacy

Service-type fetch leaks coarse intent.

---

# 136. Strict Mode

Can prefetch several/common shards.

---

# 137. Catalog Cache

Local.

---

# 138. Cache Freshness

Bounded.

---

# 139. Stale Catalog

May use within grace period if:

```text
not revoked
not expired
```

---

# 140. Emergency Revocation

Separate fast revocation feed.

---

# 141. Provider Revocation

```rust
pub struct ProviderRevocation {
    pub provider_id: AnonymousProviderId,
    pub reason: ProviderRevocationReason,
    pub effective_at: Timestamp,
    pub signature: GovernanceSignature,
}
```

---

# 142. Revocation Reasons

```rust
pub enum ProviderRevocationReason {
    KeyCompromise,
    MaliciousBehavior,
    PolicyViolation,
    VulnerableSoftware,
    OperatorRequest,
    Administrative,
}
```

---

# 143. Client Behavior

Never select revoked provider.

---

# 144. Active Sessions

Depending severity:

```text
migrate
terminate
finish safely
```

---

# 145. Provider Migration

Service-specific.

---

# 146. Failover

Fallback list generated locally.

---

# 147. Failover Order

Can be randomized among similarly eligible providers.

---

# 148. Why Randomized

Avoid herd behavior.

---

# 149. Global Outage

If many clients fail over simultaneously:

```text
stampede
```

---

# 150. Failover Jitter

Required.

---

# 151. Load Spreading

Provider selection can use:

```text
capacity-weighted randomization
```

---

# 152. Deterministic Best Provider

Bad.

Creates concentration.

---

# 153. Better

Weighted random among privacy-eligible set.

---

# 154. Weighted Selection

```rust
pub trait WeightedProviderSelector {
    fn choose(
        &self,
        candidates: &[ProviderCandidate],
        rng: &mut dyn CryptoRngCore,
    ) -> Result<AnonymousProviderId, ProviderSelectionError>;
}
```

---

# 155. CSPRNG

Use secure randomness for selection where unpredictability matters.

---

# 156. Selection Seed

Do not persist globally across devices.

---

# 157. Multi-Device Selection

Devices can choose independently.

---

# 158. Correlation Risk

All devices using same provider can link account activity.

---

# 159. Strict Mode

Prefer independent providers where feasible.

---

# 160. Shared Mailbox Constraint

Some account-wide service may require shared provider.

---

# 161. Device-Scoped Provider State

Default where possible.

---

# 162. Provider Capability Negotiation

After selection, runtime still authenticates/negotiates.

---

# 163. Descriptor ≠ Runtime Truth

Provider may become stale/misconfigured.

---

# 164. Runtime Negotiation

```rust
pub trait ProviderCapabilityNegotiator {
    fn negotiate(
        &self,
        descriptor: &AnonymousProviderDescriptor,
        runtime: RuntimeProviderCapabilities,
    ) -> Result<NegotiatedProviderCapabilities, ProviderError>;
}
```

---

# 165. Downgrade Protection

Runtime cannot negotiate below required privacy capability.

---

# 166. Example

Descriptor says:

```text
supports padding
```

runtime says no.

Strict mode:

```text
reject
```

---

# 167. Capability Mismatch

Report to local diagnostics.

Optional governance report.

---

# 168. Provider Fingerprinting

Provider-specific protocol quirks can identify users/services.

---

# 169. Standardized Client Profiles

Use common protocol behavior where possible.

---

# 170. Marketplace Fingerprinting

Do not send:

```text
full device model
OS build
contact count
```

during provider selection.

---

# 171. Request Minimization

Selection is local.

Connection sends only:

```text
required protocol negotiation
```

---

# 172. Provider Access Credentials

Use:

```text
scoped anonymous capability
quota credential
```

from Parts 35/47.

---

# 173. No Account Login to Provider

Strict mode.

---

# 174. Convenience Mode

Provider-specific account login may exist only as lower-privacy explicit mode.

---

# 175. Marketplace Payment Flow

```text
catalog
→ local select
→ obtain compatible credit
→ connect with scoped credential
```

---

# 176. No Checkout Tracking Pixel Equivalent

Hard rule.

---

# 177. User Preferences

Possible:

```text
prefer lowest cost
prefer strongest privacy
avoid specific operator
avoid jurisdiction
prefer audited providers
```

---

# 178. Preference Storage

Local.

---

# 179. Managed Policy

Organization may enforce:

```text
approved providers only
```

---

# 180. Managed Policy Visibility

Explicit.

---

# 181. User Override

Depends on organization policy.

---

# 182. Provider Blocklist

Local user can block provider/operator.

---

# 183. Blocklist Privacy

Never upload unless explicitly synced E2EE.

---

# 184. Provider Favorites

Could increase linkability.

Use cautiously.

---

# 185. Manual Pinning

Advanced users may pin provider.

---

# 186. Pinning Warning

Stable provider selection may reduce unlinkability.

---

# 187. Provider Switching UX

Normal users need not see raw IDs.

---

# 188. UX Labels

Examples:

```text
Private mailbox provider
Anonymous relay provider
Anonymous file provider
```

---

# 189. Advanced UX

Can show:

```text
operator
audit status
cost class
health
capabilities
```

---

# 190. Do Not Expose Raw Selection Score

Hard rule.

---

# 191. Provider Marketplace UI

Optional.

---

# 192. Marketplace View

Could show provider cards without exposing user's usage history.

---

# 193. No "Users also chose"

Avoid social tracking.

---

# 194. No Personalized Recommendations Server-Side

Strict mode.

---

# 195. Local Recommendation Engine

Possible:

```text
evaluate catalog locally
```

---

# 196. User Decision Inputs

Local:

```text
privacy preference
budget
network constraints
```

---

# 197. Provider Discovery Service

```rust
pub trait AnonymousServiceDiscovery {
    fn providers(
        &self,
        service: AnonymousServiceType,
    ) -> Result<Vec<AnonymousProviderDescriptor>, ProviderDiscoveryError>;
}
```

---

# 198. Provider Policy Engine

```rust
pub trait ProviderPolicyEngine {
    fn evaluate(
        &self,
        provider: &AnonymousProviderDescriptor,
        requirements: &ProviderRequirements,
    ) -> ProviderPolicyDecision;
}
```

---

# 199. Policy Decision

```rust
pub enum ProviderPolicyDecision {
    Eligible,
    Ineligible(ProviderIneligibilityReason),
}
```

---

# 200. Ineligibility Reasons

```rust
pub enum ProviderIneligibilityReason {
    Revoked,
    UnsupportedProtocol,
    PrivacyRequirementMissing,
    HealthTooLow,
    OperatorConflict,
    PriceTooHigh,
    PolicyMismatch,
}
```

---

# 201. Selection Service

```rust
pub trait ProviderSelectionService {
    fn plan(
        &self,
        requirements: ProviderRequirements,
    ) -> Result<ProviderSelection, ProviderSelectionError>;
}
```

---

# 202. Selection Audit

Local-only decision trace.

---

# 203. Decision Trace

```rust
pub struct ProviderSelectionTrace {
    pub candidate_count: usize,
    pub rejected_counts: BTreeMap<ProviderIneligibilityReason, usize>,
    pub final_strategy: ProviderSelectionStrategy,
}
```

---

# 204. Privacy

No user identity.

---

# 205. Useful for Diagnostics

Shows:

```text
no provider met strict privacy requirement
```

---

# 206. Marketplace Operator Registration

Provider operator joins through governance.

---

# 207. Listing Requirements

Could include:

```text
key ownership
endpoint validation
capacity proof
policy declaration
pricing
software version
```

---

# 208. Listing Fee

If any, governance/business concern.

Not required architecturally.

---

# 209. No Pay-to-Rank in Strict Mode

Hard rule.

---

# 210. Sponsored Ranking

Should not override privacy/performance policy.

Prefer no sponsored ranking.

---

# 211. Marketplace Neutrality

Selection must be policy-driven, not commercial placement.

---

# 212. Transparency

Provider catalog changes can enter transparency log.

---

# 213. Catalog Events

```text
provider added
provider suspended
provider revoked
price changed
policy changed
```

---

# 214. User Privacy

Transparency log contains no user selection events.

---

# 215. Marketplace Analytics

Safe aggregate:

```text
number of active providers
capacity by service type
price ranges
health distribution
```

---

# 216. No Provider Popularity Tracking by Default

Hard rule.

---

# 217. Popularity Can Be Sensitive

It may reveal where users concentrate.

---

# 218. Capacity Planning

Use aggregate anonymous load from providers.

---

# 219. Differential Privacy

Potential for public market stats.

---

# 220. Small Provider Protection

Avoid publishing exact traffic for small operators.

---

# 221. Marketplace Abuse

Threats:

```text
malicious provider
fake capability
fake price
bait-and-switch
selective denial
traffic discrimination
```

---

# 222. Capability Bait-and-Switch

Descriptor says:

```text
privacy feature available
```

but runtime disables it.

---

# 223. Defense

Runtime capability validation.

---

# 224. Selective Denial

Provider may block some clients.

---

# 225. Detection

Local health evidence + independent probes.

---

# 226. Traffic Discrimination

Provider may prioritize certain users.

---

# 227. Strict Privacy Goal

Provider should not have stable user identity to discriminate by.

---

# 228. Price Manipulation

Signed catalog prevents silent local price rewrite.

---

# 229. Catalog Rollback

Reject older catalog after newer accepted.

---

# 230. Provider Key Rotation

Descriptor signed by current key.

---

# 231. Key Transition

Signed old→new transition or governance revalidation.

---

# 232. Operator Key Compromise

Suspend all affected providers pending revalidation.

---

# 233. Service Endpoint Rotation

Normal.

---

# 234. Endpoint Lifetime

Can be shorter than provider identity.

---

# 235. Provider Endpoint Privacy

Some services can use multiple endpoints.

---

# 236. Anycast/CDN

Possible, but operator/ASN metadata must reflect reality.

---

# 237. Public Provider Catalog Attack Surface

Parser must be strict/bounded.

---

# 238. Catalog Size Limits

Bound:

```text
provider count
descriptor size
capability count
signature count
```

---

# 239. No Unbounded JSON

Use:

```text
Postcard internally
RON for human-readable config
JSON only external interop if necessary
```

---

# 240. Signed Catalog Encoding

Canonical binary form required for signatures.

---

# 241. Canonicalization

Never sign ambiguous serialization.

---

# 242. Catalog Compression

Optional.

Decompress with hard limits.

---

# 243. Catalog Cache Database

Potential:

```text
provider_catalogs
provider_descriptors
provider_health_cache
provider_audit_attestations
provider_revocations
provider_preferences
provider_selection_history_local
```

---

# 244. Selection History

Local only.

---

# 245. Why Store History

Useful for:

```text
rotation
avoid unstable providers
```

---

# 246. History Retention

Bounded.

---

# 247. No Long-Term Behavioral Profile

Hard rule.

---

# 248. Provider Experience Decay

Old failures lose influence.

---

# 249. Backoff

Repeatedly failing provider gets temporary local penalty.

---

# 250. Penalty Scope

Local device only.

---

# 251. Marketplace Federation

Future:

```text
multiple catalogs
multiple governance domains
```

---

# 252. Federation Risk

Duplicate providers / conflicting policy.

---

# 253. Canonical Provider Identity

Required.

---

# 254. Catalog Merge

Future logic:

```text
verify each domain
deduplicate provider IDs
apply local trust policy
```

---

# 255. Cross-Domain Trust

Explicit local policy.

---

# 256. Initial Production Recommendation

Single SIAR governance catalog with signed mirrors.

---

# 257. Migration Roadmap

```text
single catalog
→ multiple mirrors
→ independent auditors
→ multiple governance catalogs
```

---

# 258. Offline Provider Discovery

Signed catalog bundle can be imported.

---

# 259. Use Cases

```text
censorship
air-gapped staging
emergency deployment
```

---

# 260. Bundle Freshness

Bounded.

---

# 261. Offline Selection

Allowed if provider endpoint reachable and revocation freshness acceptable.

---

# 262. Strict Mode

Stale revocation feed may block new selection.

---

# 263. Android

Catalog fetch/cache should be low-frequency.

---

# 264. Battery

Provider discovery must not poll constantly.

---

# 265. Desktop Daemon

Can refresh catalog in background.

---

# 266. Refresh Policy

```rust
pub struct CatalogRefreshPolicy {
    pub refresh_interval: Duration,
    pub max_staleness: Duration,
    pub jitter: Duration,
}
```

---

# 267. Jitter

Avoid synchronized catalog fetch.

---

# 268. Network Cost

Mobile can defer non-urgent refresh on metered network.

---

# 269. Emergency Revocation Feed

Still higher priority.

---

# 270. Observability

Safe local metrics:

```text
catalog age
candidate counts
selection failure reason
provider health class
```

---

# 271. Forbidden Telemetry

No:

```text
exact user's provider sequence
wallet linkage
contact linkage
mailbox mapping
```

---

# 272. Remote Metrics

If any, aggregate and privacy-reviewed.

---

# 273. Support Bundle

Can include:

```text
catalog version
provider capability mismatch class
selection failure category
```

---

# 274. Redactions

No:

```text
service credential
payment token
relationship ID
```

---

# 275. Testkit

Need fake marketplace/catalog.

---

# 276. Test Scenarios

```text
healthy providers
revoked provider
capability mismatch
price change
operator concentration
catalog rollback
failover storm
```

---

# 277. Unit Tests

Test:

```text
descriptor validation
catalog signature
eligibility
selection
rotation
```

---

# 278. Privacy Tests

Ensure:

```text
selection remains local
catalog source sees no user requirements
cross-service provider IDs do not expose user identity
```

---

# 279. Operator Diversity Test

Strict mode avoids same operator across restricted service combination.

---

# 280. Capability Downgrade Test

Runtime capability weaker than descriptor.

Reject.

---

# 281. Revocation Test

Revoked provider never selected.

---

# 282. Price Rollback Test

Old price catalog rejected.

---

# 283. Catalog Rollback Test

Old provider catalog rejected after new accepted.

---

# 284. Failover Test

Only eligible fallback providers used.

---

# 285. Stampede Test

Mass provider outage produces jittered failover.

---

# 286. Load Distribution Test

Weighted random selection avoids one dominant provider.

---

# 287. Marketplace Concentration Test

Policy detects operator/ASN concentration.

---

# 288. Offline Bundle Test

Signed bundle works within freshness window.

---

# 289. Fuzzing

Fuzz:

```text
provider descriptor
catalog
audit attestation
revocation
capability set
```

---

# 290. Property Tests

Properties:

```text
revoked provider never eligible
strict privacy requirement cannot be overridden by price
fallback set contains only eligible providers
provider/operator IDs never equal user IDs
```

---

# 291. Formal Verification Targets

Good candidates:

```text
eligibility policy
fallback resolver
operator diversity constraints
catalog anti-rollback
```

---

# 292. TLA+ Candidate

Provider rotation/failover state machine.

---

# 293. Kani Candidate

Eligibility hard-constraint evaluation.

---

# 294. Loom Candidate

Concurrent catalog refresh + active selection.

---

# 295. Performance Tests

Measure:

```text
catalog parse
signature verification
local filtering
selection latency
cache memory
```

---

# 296. Scale Tests

Synthetic catalogs:

```text
100
1,000
10,000
100,000
```

providers.

---

# 297. Large Catalog Strategy

Shard/index locally.

---

# 298. No Per-User Server Ranking

Hard rule even at scale.

---

# 299. Privacy Lab Integration

Part 42 should test:

```text
provider-selection correlation
cross-service operator exposure
catalog-fetch intent leakage
rotation fingerprinting
```

---

# 300. Security Invariants

Mandatory:

```text
1. Provider discovery does not require user identity.
2. Strict-mode provider selection is local from signed metadata.
3. Privacy requirements are evaluated before price/latency/performance.
4. Revoked or expired providers are never selected.
5. Fallback providers satisfy the same hard privacy constraints.
6. Provider IDs and operator IDs are separate from all user/contact/group IDs.
7. Same operator is not treated as independent across services in strict mode.
8. Bridge descriptors that require enumeration resistance are never placed in the public provider catalog.
9. Provider runtime negotiation cannot silently downgrade advertised privacy capability.
10. Price catalogs and provider catalogs are anti-rollback protected.
11. Marketplace governance never records user selection events in transparency logs.
12. Commercial ranking cannot override privacy eligibility.
```

---

# 301. Recommended Crate Layout

```text
crates/
├── siar-provider-core/
├── siar-provider-catalog/
├── siar-provider-discovery/
├── siar-provider-policy/
├── siar-provider-selection/
├── siar-provider-health/
├── siar-provider-reputation/
├── siar-provider-audit/
├── siar-provider-marketplace/
├── siar-provider-observability/
└── siar-provider-testkit/
```

---

# 302. `siar-provider-core`

Owns:

```text
provider IDs
service types
capability enums
errors
```

---

# 303. `siar-provider-catalog`

Signed catalog parsing/cache/anti-rollback.

---

# 304. `siar-provider-discovery`

Catalog/mirror/offline bundle retrieval.

---

# 305. `siar-provider-policy`

Hard eligibility and local policy.

---

# 306. `siar-provider-selection`

Weighted privacy-aware selection/rotation/failover.

---

# 307. `siar-provider-health`

Health evidence and local experience.

---

# 308. `siar-provider-reputation`

Infrastructure reputation only.

---

# 309. `siar-provider-audit`

Audit attestations and assurance level.

---

# 310. `siar-provider-marketplace`

Listing/governance/pricing integration.

---

# 311. `siar-provider-observability`

Privacy-safe diagnostics.

---

# 312. `siar-provider-testkit`

Fake providers/catalogs/outages.

---

# 313. Error Taxonomy

```rust
pub enum ProviderSelectionError {
    CatalogUnavailable,
    CatalogStale,
    SignatureInvalid,
    NoEligibleProvider,
    ProviderRevoked,
    CapabilityMissing,
    OperatorConflict,
    PriceConstraint,
    HealthConstraint,
    RuntimeCapabilityMismatch,
    Internal,
}
```

---

# 314. Initial Production Scope

Implement first:

```text
signed provider catalog
local filtering
service-specific capability descriptors
operator IDs
health classes
signed price references
privacy-first weighted selection
fallback list
provider rotation
cross-service operator diversity
revocation feed
catalog anti-rollback
privacy-safe diagnostics
```

Then add:

```text
private catalog queries
PIR/sharded discovery
multiple marketplace governance domains
third-party audits
advanced cross-service diversity optimization
differentially private market statistics
```

---

# 315. Definition of Done

Part 48 is complete when:

- provider and operator identities are explicit and separate
- provider capabilities are signed and versioned
- provider catalogs can be distributed via signed mirrors/offline bundles
- strict-mode selection happens locally
- hard privacy constraints always precede cost/performance ranking
- health, capacity, price, audit status, and operator diversity are integrated
- cross-service operator exposure is controlled
- provider rotation and jittered failover are defined
- runtime capability negotiation cannot weaken required privacy
- provider revocation and catalog anti-rollback are enforced
- bridges requiring enumeration resistance remain outside public catalogs
- commercial ranking cannot override privacy policy
- diagnostics/telemetry avoid revealing user provider history
- privacy, failover, concentration, rollback, capability mismatch, fuzz, and formal tests are specified

---

# 316. Final Architecture

```text
                   SIGNED PROVIDER CATALOGS
                            │
                            ▼
                     LOCAL DISCOVERY
                            │
                            ▼
                    HARD ELIGIBILITY
             ┌──────────────┼──────────────┐
             │              │              │
          Privacy        Revocation     Capability
             │              │              │
             └──────────────┼──────────────┘
                            ▼
                    DIVERSITY / HEALTH
                            │
                            ▼
                      PRICE / LATENCY
                            │
                            ▼
                  WEIGHTED LOCAL SELECT
                            │
                            ▼
                 PROVIDER-SPECIFIC ADAPTER
```

Strict privacy selection model:

```text
signed metadata
+
local policy
+
hard privacy constraints
+
operator diversity
+
health
+
cost/performance optimization
```

not:

```text
send user profile to marketplace
→ marketplace tells user which provider to use
```

---

# 317. Final Principle

Provider choice is part of the anonymity architecture.

The correct model is:

```text
signed capability advertisement
+
local privacy-aware selection
+
operator diversity
+
auditable governance
+
revocation
+
anti-rollback
+
privacy-preserving pricing integration
```

rather than:

```text
centralized marketplace tracks every user selection
```

This architecture gives SIAR a scalable provider ecosystem without turning service discovery into a centralized map of anonymous infrastructure usage.
