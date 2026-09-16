# Core System Architecture Part 49 — Anonymous Resource Scheduling, Capacity Markets, Load Balancing & Privacy-Preserving Infrastructure Allocation Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 49  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–48  

**Primary purpose:** define the complete privacy-preserving infrastructure allocation architecture for SIAR, including resource scheduling, anonymous demand, provider capacity advertisement, load balancing, reservation, congestion control, fairness, priority, capacity markets, multi-operator allocation, and Rust control/data-plane boundaries.

---

# 1. Purpose

SIAR's anonymous infrastructure consumes scarce resources:

```text
mixnet packet capacity
mailbox storage
bulk storage
relay bandwidth
realtime relay slots
bridge bandwidth
directory/mirror capacity
```

If all clients simply choose the "best" provider, the network can suffer:

```text
hotspots
herd behavior
operator concentration
privacy correlation
congestion collapse
unfair resource use
```

The governing principle is:

> **SIAR must allocate scarce infrastructure through privacy-preserving, capacity-aware scheduling that balances load without requiring a central scheduler to know who the users are or what relationships generated the demand.**

---

# 2. Architectural Position

```text
Provider Capacity
       │
       ▼
Signed Capacity Advertisement
       │
       ▼
Local Eligibility + Privacy Filters
       │
       ▼
Resource Scheduler
       │
       ├── load balancing
       ├── reservation
       ├── quota
       ├── fairness
       └── priority
       │
       ▼
Provider Allocation
```

---

# 3. Core Separation

Keep distinct:

```text
provider capacity
user demand
service quota
payment credit
communication identity
allocation decision
operator settlement
```

---

# 4. Non-Goals

Part 49 does not require:

```text
centralized per-user auction
public bidding by user identity
token speculation
packet-by-packet market pricing
global demand surveillance
```

---

# 5. Resource Types

```rust
pub enum InfrastructureResource {
    MixnetCells,
    MailboxBytes,
    MailboxObjects,
    BulkStorageBytes,
    BulkEgressBytes,
    RealtimeBandwidth,
    RealtimeSessionSlot,
    BridgeBandwidth,
    DirectoryRequests,
}
```

---

# 6. Capacity Advertisement

Provider publishes coarse capacity.

---

# 7. Capacity Descriptor

```rust
pub struct CapacityAdvertisement {
    pub provider: AnonymousProviderId,
    pub resource: InfrastructureResource,
    pub capacity_class: CapacityClass,
    pub utilization_class: UtilizationClass,
    pub reservation_supported: bool,
    pub valid_until: Timestamp,
    pub signature: CapacitySignature,
}
```

---

# 8. Capacity Class

```rust
pub enum CapacityClass {
    Small,
    Medium,
    Large,
    VeryLarge,
}
```

---

# 9. Utilization Class

```rust
pub enum UtilizationClass {
    Low,
    Moderate,
    High,
    Saturated,
    Unknown,
}
```

---

# 10. Why Coarse Classes

Exact realtime utilization can enable:

```text
traffic inference
operator targeting
load fingerprinting
```

---

# 11. Fine-Grained Capacity

May exist internally.

Clients should receive only what selection needs.

---

# 12. Signed Capacity Epoch

```rust
pub struct CapacityEpoch(pub u64);
```

---

# 13. Capacity Freshness

Short enough for routing decisions, but not so frequent that every client polls continuously.

---

# 14. Capacity Source

Possible:

```text
provider self-report
independent observer
directory aggregation
```

---

# 15. Self-Report Is Not Fully Trusted

Hard rule.

---

# 16. Independent Capacity Verification

Synthetic probes can validate:

```text
availability
rough throughput
queue delay
```

---

# 17. User Demand Privacy

Do not send:

```text
user profile
contact graph
conversation
file name
```

to scheduler.

---

# 18. Demand Descriptor

Local:

```rust
pub struct ResourceDemand {
    pub resource: InfrastructureResource,
    pub amount: ResourceAmount,
    pub service_class: ServiceClass,
    pub deadline: Option<Instant>,
    pub privacy_mode: PrivacyRoutingMode,
}
```

---

# 19. Demand Minimization

Provider only learns amount necessary to serve request.

---

# 20. Allocation Scope

```rust
pub struct AllocationScopeId(pub [u8; 32]);
```

Random/scoped.

---

# 21. No Global Scheduler User ID

Hard rule.

---

# 22. Scheduling Modes

```rust
pub enum AllocationMode {
    Immediate,
    Reserved,
    Opportunistic,
    Background,
}
```

---

# 23. Immediate

Examples:

```text
realtime call
security-critical control
```

---

# 24. Reserved

Examples:

```text
large file upload
planned long relay session
```

---

# 25. Opportunistic

Examples:

```text
background backup
bulk prefetch
```

---

# 26. Background

Lowest urgency.

---

# 27. Priority Classes

```rust
pub enum InfrastructurePriority {
    Emergency,
    SecurityControl,
    Realtime,
    InteractiveMessaging,
    Attachment,
    Background,
    Cover,
}
```

---

# 28. Priority Invariant

Priority controls scheduling, not privacy downgrade.

---

# 29. Emergency Priority

Can preempt lower-priority work.

---

# 30. Emergency Caveat

Emergency use must not silently expose identity unless user explicitly chooses a less-private path.

---

# 31. Resource Scheduler Trait

```rust
pub trait AnonymousResourceScheduler {
    fn allocate(
        &self,
        demand: ResourceDemand,
        candidates: &[ProviderCandidate],
    ) -> Result<ResourceAllocation, AllocationError>;
}
```

---

# 32. Resource Allocation

```rust
pub struct ResourceAllocation {
    pub provider: AnonymousProviderId,
    pub reservation: Option<ResourceReservation>,
    pub fallback: Vec<AnonymousProviderId>,
}
```

---

# 33. Scheduling Inputs

Hard constraints first:

```text
privacy eligibility
protocol capability
revocation
quota/payment compatibility
operator diversity
```

Then:

```text
capacity
utilization
latency
cost
```

---

# 34. Local Scheduling

Preferred.

---

# 35. Why

Central scheduler seeing every allocation could reconstruct:

```text
usage history
service patterns
timing
```

---

# 36. Capacity Catalog

Clients fetch signed capacity metadata.

---

# 37. Capacity Cache

Local.

---

# 38. Capacity Staleness

If stale:

```text
decrease confidence
```

not immediately trust exact value.

---

# 39. Weighted Selection

Eligible provider weight can depend on:

```text
capacity
health
local failure history
operator diversity
```

---

# 40. No Deterministic Best Provider

Hard rule.

---

# 41. Why

Creates herd behavior.

---

# 42. Weighted Random Selection

Recommended.

---

# 43. Selection Weight

```rust
pub struct AllocationWeight(pub f64);
```

---

# 44. Weight Caps

Prevent dominant provider.

---

# 45. Operator-Level Weight Cap

Needed.

---

# 46. Provider-Level Weight Cap

Also needed.

---

# 47. Anti-Herd Jitter

Randomize refresh/failover decisions.

---

# 48. Provider Rotation

Can spread load.

---

# 49. Rotation Constraints

Do not rotate active:

```text
call
bulk upload session
mailbox transaction
```

mid-operation unless migration is supported.

---

# 50. Reservation

Some services need pre-allocation.

---

# 51. Reservation Object

```rust
pub struct ResourceReservation {
    pub reservation_id: ReservationId,
    pub provider: AnonymousProviderId,
    pub resource: InfrastructureResource,
    pub amount: ResourceAmount,
    pub starts_at: Timestamp,
    pub expires_at: Timestamp,
    pub credential: ReservationCredential,
}
```

---

# 52. Reservation Credential

Scoped secret/capability.

---

# 53. Reservation Privacy

No:

```text
AccountId
DeviceId
ConversationId
```

---

# 54. Reservation TTL

Short.

---

# 55. Reservation Overbooking

Provider may overbook carefully.

---

# 56. Overbooking Risk

Can cause:

```text
latency spikes
call failure
queue collapse
```

---

# 57. Critical Resources

Realtime relay should use stricter reservation.

---

# 58. Bulk Transfer

Can tolerate opportunistic allocation.

---

# 59. Mailbox Storage

Long-lived quota rather than short reservation.

---

# 60. Mixnet Cells

Usually quota-based, not per-route reservation.

---

# 61. Bridge Bandwidth

May use session quota.

---

# 62. Reservation State

```rust
pub enum ReservationState {
    Proposed,
    Confirmed,
    Active,
    Released,
    Expired,
    Failed,
}
```

---

# 63. Idempotency

Reserve/release must be idempotent.

---

# 64. Reservation Failure

Try eligible fallback.

---

# 65. No Privacy Weakening

Hard rule.

---

# 66. Congestion Control

Provider exposes coarse congestion state.

---

# 67. Congestion State

```rust
pub enum InfrastructureCongestion {
    Normal,
    Elevated,
    Severe,
}
```

---

# 68. Local Response

Can:

```text
reduce bulk traffic
defer background
choose alternate provider
```

---

# 69. Strict Mode Cover Traffic

Must coordinate with Part 37.

---

# 70. Cover Shedding

Under severe congestion:

```text
cover may be reduced
```

if profile permits.

---

# 71. Maximum Anonymity

If cover cannot be maintained, system should signal degraded privacy.

---

# 72. No Invisible Cover Collapse

Hard rule.

---

# 73. Load Balancing Strategy

```rust
pub enum LoadBalancingStrategy {
    PrivacyWeightedRandom,
    CapacityWeightedRandom,
    LatencyAwareRandom,
    CostAwareRandom,
}
```

---

# 74. Maximum Anonymity

Use:

```text
PrivacyWeightedRandom
```

---

# 75. Balanced Mode

May use capacity/latency.

---

# 76. Local Failure History

Can temporarily de-prioritize bad provider.

---

# 77. Decay

Old failures decay.

---

# 78. No Permanent Blacklisting From One Client Failure

Hard rule.

---

# 79. Cross-Service Allocation

Avoid using same operator for all services.

---

# 80. Cross-Service Scheduler

```rust
pub trait CrossServiceAllocator {
    fn allocate_bundle(
        &self,
        demands: &[ResourceDemand],
    ) -> Result<Vec<ResourceAllocation>, AllocationError>;
}
```

---

# 81. Example Bundle

```text
mailbox
+
realtime relay
+
bulk storage
```

---

# 82. Operator Exposure Constraint

```rust
pub struct OperatorExposureConstraint {
    pub max_services_per_operator: u8,
}
```

---

# 83. Multi-Operator Balancing

Prefer independent operators.

---

# 84. ASN/Cloud Balancing

Optional further diversity.

---

# 85. Jurisdiction Balancing

Policy input.

---

# 86. Resource Fairness

Providers must prevent one anonymous client/capability from consuming everything.

---

# 87. Fairness Scope

Use:

```text
quota credential
reservation credential
service capability
```

---

# 88. Fair Queueing

Potential:

```text
weighted fair queueing
deficit round robin
```

---

# 89. Anonymous Fairness Problem

Without identity, provider needs scoped credential to distinguish resource consumers.

---

# 90. Scoped Fairness Token

```rust
pub struct FairnessCredential(SecretBytes);
```

---

# 91. Token Lifetime

Short.

---

# 92. Token Rotation

Avoid long-term linkability.

---

# 93. Fairness Token Scope

Bound to:

```text
provider
service
resource class
epoch
```

---

# 94. No Cross-Provider Fairness ID

Hard rule.

---

# 95. No Cross-Service Fairness ID

Hard rule.

---

# 96. Scheduling Within Provider

Provider runtime may use:

```text
priority queues
fair queues
rate limiting
```

---

# 97. Provider Scheduler

```rust
pub trait ProviderResourceScheduler {
    fn admit(
        &self,
        request: ProviderResourceRequest,
    ) -> Result<ProviderAdmissionDecision, ProviderCapacityError>;
}
```

---

# 98. Admission Decision

```rust
pub enum ProviderAdmissionDecision {
    Accepted,
    Deferred,
    RejectedCapacity,
    RejectedQuota,
}
```

---

# 99. Deferred

Useful for:

```text
background bulk
```

---

# 100. Interactive Messaging

Should not be deferred excessively.

---

# 101. Realtime

Requires bounded admission latency.

---

# 102. Capacity Market Concept

Providers publish:

```text
available capacity class
price
service capability
```

Clients choose locally.

---

# 103. No Central Bid Auction

Initial architecture.

---

# 104. Why

Auctions reveal demand timing and willingness-to-pay.

---

# 105. Signed Capacity/Price Catalog

Integrates Parts 47–48.

---

# 106. Market Snapshot

```rust
pub struct CapacityMarketSnapshot {
    pub epoch: CapacityEpoch,
    pub entries: Vec<CapacityMarketEntry>,
    pub signatures: CatalogSignatureBundle,
}
```

---

# 107. Market Entry

```rust
pub struct CapacityMarketEntry {
    pub provider: AnonymousProviderId,
    pub resource: InfrastructureResource,
    pub capacity: CapacityClass,
    pub utilization: UtilizationClass,
    pub price: Option<ResourcePrice>,
}
```

---

# 108. Market Privacy

Snapshot is broad/global.

Not personalized.

---

# 109. Congestion Pricing

Optional.

---

# 110. Privacy Risk

Rapid per-minute pricing can correlate demand.

---

# 111. Recommendation

Use coarse price epochs.

---

# 112. Price Epoch

Example:

```text
hourly
daily
```

depending service.

---

# 113. No Per-Request Personalized Price

Strict mode.

---

# 114. Surge Pricing

If used:

```text
signed
coarse
globally visible
```

---

# 115. Maximum Price Policy

Local user setting.

---

# 116. Reservation Price Lock

Reservation can lock price for interval.

---

# 117. Price Lock

```rust
pub struct ReservationPriceLock {
    pub price_id: ProviderPriceId,
    pub valid_until: Timestamp,
}
```

---

# 118. No Bait-and-Switch

Hard rule.

---

# 119. Provider Settlement

Part 47 receipts feed settlement.

---

# 120. Allocation Does Not Need Settlement Identity

Hard rule.

---

# 121. Operator Capacity Incentives

Could reward high availability.

---

# 122. Architecture Boundary

Economic incentive design remains separate.

---

# 123. Scheduling Credits

Payment credits can influence eligibility:

```text
has enough resource credit?
```

but not identity.

---

# 124. Free Tier Scheduling

Quota-limited.

---

# 125. Paid Priority

Potential business option.

---

# 126. Privacy Concern

Priority class could fingerprint paid users.

---

# 127. Recommendation

Use small number of common priority classes.

---

# 128. No Unique Per-User QoS Profile

Hard rule.

---

# 129. Emergency Priority

May be free/privileged.

---

# 130. Abuse Risk

Attackers could mark everything emergency.

---

# 131. Emergency Credential

Need bounded authorization.

---

# 132. Emergency Resource Token

```rust
pub struct EmergencyResourceCredential {
    pub scope: EmergencyScope,
    pub expires_at: Timestamp,
    pub proof: EmergencyProof,
}
```

---

# 133. Emergency Scope

Examples:

```text
message
short relay call
SOS broadcast
```

---

# 134. No Unlimited Emergency Credit

Hard rule.

---

# 135. Network-Wide Capacity Health

Expose coarse state:

```rust
pub enum NetworkCapacityHealth {
    Healthy,
    Busy,
    Constrained,
    Critical,
}
```

---

# 136. Client Behavior

Busy:

```text
defer background
```

Constrained:

```text
throttle bulk
```

Critical:

```text
protect control/emergency/realtime
```

---

# 137. Capacity Health Source

Aggregate signed control-plane metadata.

---

# 138. Privacy

No per-user usage.

---

# 139. Queue Isolation

Provider should separate logical resource pools:

```text
control
realtime
interactive
bulk
cover
```

---

# 140. Hard Memory Limits

Required.

---

# 141. No Unbounded Queue

Hard rule.

---

# 142. Backpressure

Propagate to client.

---

# 143. Backpressure Signal

```rust
pub enum BackpressureSignal {
    SlowDown,
    PauseBackground,
    RetryAfter(Duration),
}
```

---

# 144. Retry-After Privacy

Coarse.

---

# 145. Client Retry

Use jitter.

---

# 146. Retry Storm

Avoid synchronized retries.

---

# 147. Allocation Cache

Client may cache recent provider plans.

---

# 148. Cache Lifetime

Short.

---

# 149. Active Session Pinning

Do not recalc every packet.

---

# 150. Mixnet Path

Resource allocation selects provider/gateway capacity.

Actual route selection still Part 38.

---

# 151. Mailbox

Resource allocation chooses mailbox provider/quota.

---

# 152. Bulk

Chooses provider + reservation.

---

# 153. Realtime

Chooses relay slots.

---

# 154. Bridge

Chooses bridge capacity.

---

# 155. Directory/Mirror

Usually free shared service; load balancing by mirror selection.

---

# 156. Mirror Load Balancing

Random among healthy mirrors.

---

# 157. Capacity Privacy Leakage

Repeated selection of underloaded provider could expose client behavior.

---

# 158. Mitigation

Use weighted random, not exact minimum-load selection.

---

# 159. Local Demand Bucketing

Amount can be bucketed.

---

# 160. Example

Instead of:

```text
need exactly 13.74 MB
```

request:

```text
16 MB class
```

---

# 161. Demand Size Classes

```rust
pub enum DemandSizeClass {
    Tiny,
    Small,
    Medium,
    Large,
    VeryLarge,
}
```

---

# 162. Privacy Benefit

Reduces exact object-size leakage during reservation.

---

# 163. Over-Reservation

Unused capacity may be refunded/released.

---

# 164. Reservation Padding

Can intentionally reserve coarse bucket.

---

# 165. Cost Tradeoff

Privacy may cost more.

---

# 166. Budget-Aware Scheduler

```rust
pub struct AllocationBudget {
    pub max_cost: Option<u64>,
    pub max_latency_class: Option<LatencyClass>,
}
```

---

# 167. Hard Privacy Still First

Budget cannot override.

---

# 168. Capacity Reservation Privacy

Provider sees requested class.

Avoid file/message identity.

---

# 169. Resource Pooling

Provider can pool anonymous clients.

---

# 170. No Dedicated User Queue

Strict mode.

---

# 171. Dedicated Enterprise Capacity

Possible separate explicit mode.

---

# 172. Enterprise Privacy

Organization may know user/member allocation.

Different privacy contract.

---

# 173. Multi-Tenant Provider

Need tenant isolation.

---

# 174. Anonymous Tenant

Credential-based, not identity-based.

---

# 175. Scheduling Audit

Provider can audit capacity decisions without user identity.

---

# 176. Audit Record

```rust
pub struct ProviderSchedulingAudit {
    pub resource: InfrastructureResource,
    pub priority: InfrastructurePriority,
    pub decision: ProviderAdmissionDecision,
    pub timestamp_bucket: TimeBucket,
}
```

---

# 177. No Scope ID in General Telemetry

Hard rule.

---

# 178. Internal Debug

May use ephemeral correlation ID locally.

---

# 179. No Cross-System Trace

Hard rule.

---

# 180. Capacity Abuse Detection

Detect:

```text
reservation spam
never-used reservations
quota exhaustion attacks
burst floods
```

---

# 181. Reservation Deposit

Could require small credit hold.

---

# 182. Refund

Released if reservation used honestly.

---

# 183. Privacy

Deposit uses scoped resource credit.

---

# 184. No Account Penalty Needed

Hard rule.

---

# 185. No-Show Penalty

Can consume small reservation fee.

---

# 186. Scope

Per provider/service token.

---

# 187. Capacity Hoarding

Attacker reserves many providers.

---

# 188. Defense

```text
reservation limits
credit cost
short TTL
proof-of-work optionally
```

---

# 189. Provider Collusion

Two operators can compare reservation patterns.

---

# 190. Cross-Provider ID Separation

Mandatory.

---

# 191. Timing Correlation

Reservation just before usage can link events.

---

# 192. Mitigation

```text
pre-reserve
coarse time windows
batch reservation
```

---

# 193. Reservation Window

```rust
pub enum ReservationTimeClass {
    Immediate,
    NearTerm,
    Flexible,
}
```

---

# 194. Background Bulk

Use flexible reservations.

---

# 195. Realtime

Immediate.

---

# 196. Mixnet Traffic

No per-message reservation.

---

# 197. Predictive Scheduling

Potential future.

---

# 198. Privacy Risk

Predicting from user history creates local behavioral profile.

---

# 199. Recommendation

If used, keep model:

```text
local-only
```

---

# 200. No Cloud Demand Predictor

Strict mode.

---

# 201. Local Prefetch

Can pre-reserve common capacity based on local coarse patterns.

---

# 202. User Control

Allow disabling predictive reservation.

---

# 203. Capacity Market Governance

Provider listings governed by Part 48.

---

# 204. Market Manipulation Threats

```text
fake capacity
fake congestion
fake price
withholding capacity
```

---

# 205. Verification

Independent probes + signed history.

---

# 206. Provider Slashing/Penalty

Economic details deferred.

---

# 207. Governance Sanction

Provider can be:

```text
deweighted
suspended
revoked
```

---

# 208. No Automatic Punishment From One Client Complaint

Hard rule.

---

# 209. Capacity History

Aggregate operator-level.

---

# 210. Client Selection History

Local only.

---

# 211. Scheduling Telemetry

Safe aggregate:

```text
allocation success rate
capacity class
congestion class
reservation success
```

---

# 212. Forbidden Telemetry

No:

```text
ContactId
GroupId
CallId
MailboxId
object name
```

---

# 213. Support Bundle

Can include:

```text
allocation failure class
catalog epoch
capacity freshness
```

---

# 214. Redactions

No reservation credential.

---

# 215. Testkit

Need deterministic capacity simulator.

---

# 216. Simulator Inputs

```text
providers
capacity
utilization
prices
failure rates
client demand
priority mix
```

---

# 217. Simulator Outputs

```text
allocation success
load distribution
operator concentration
queue delay
cost
privacy exposure
```

---

# 218. Herding Test

Many clients receive same market snapshot.

Verify selection spreads.

---

# 219. Saturation Test

One provider becomes saturated.

---

# 220. Failover Test

Load redistributes gradually.

---

# 221. Stampede Test

Provider outage.

---

# 222. Privacy Constraint Test

Cheap low-privacy provider must never receive strict demand.

---

# 223. Cross-Service Diversity Test

Mailbox + relay + bulk should not collapse to same operator in strict profile if alternatives exist.

---

# 224. Reservation Race Test

Many clients compete for last capacity.

---

# 225. Idempotency Test

Repeated reserve/release.

---

# 226. Refund Test

Unused reservation releases credit.

---

# 227. Emergency Priority Test

Emergency/control preempts background.

---

# 228. Starvation Test

Background eventually progresses.

---

# 229. Fairness Test

One credential cannot monopolize resource pool.

---

# 230. Quota Test

Provider refuses over-limit request.

---

# 231. Congestion Feedback Test

Client slows appropriately.

---

# 232. Fuzzing

Fuzz:

```text
capacity advertisement
reservation
market snapshot
backpressure signal
```

---

# 233. Property Tests

Properties:

```text
revoked provider never allocated
privacy-ineligible provider never selected
operator exposure constraint always holds when feasible
reservation cannot consume more than authorized resource
```

---

# 234. Formal Verification Targets

Strong candidates:

```text
reservation lifecycle
priority scheduler
fairness credential scope
fallback allocator
```

---

# 235. TLA+ Candidate

Distributed reservation + release state machine.

---

# 236. Kani Candidate

Hard policy constraint evaluation.

---

# 237. Loom Candidate

Concurrent reservation accounting.

---

# 238. Performance Tests

Measure:

```text
selection latency
reservation throughput
queue scheduling
capacity catalog parse
```

---

# 239. Scale Tests

Synthetic:

```text
100 providers
10k providers
1M active reservations
```

---

# 240. Privacy Lab Integration

Part 42 should test:

```text
provider allocation correlation
reservation timing leakage
operator exposure
market-snapshot fetch patterns
```

---

# 241. Security Invariants

Mandatory:

```text
1. Resource allocation does not require global user identity.
2. Privacy eligibility is evaluated before capacity, price, or latency.
3. No deterministic best-provider rule causes universal herding.
4. Operator concentration is constrained in strict mode.
5. Reservation credentials are scoped, expiring, and never reused across providers.
6. Mixnet packet-level routing never carries reservation/payment identity.
7. Emergency priority cannot silently downgrade privacy.
8. Cover reduction under congestion is explicitly reflected in privacy state.
9. Provider scheduling telemetry never includes communication identifiers.
10. Capacity-market pricing is coarse/signed and not personalized per request in strict mode.
11. Provider failover uses jitter and privacy-eligible candidates only.
12. Reservation/settlement/accounting operations are idempotent and crash-safe.
```

---

# 242. Recommended Crate Layout

```text
crates/
├── siar-resource-core/
├── siar-capacity-catalog/
├── siar-allocation-policy/
├── siar-resource-scheduler/
├── siar-resource-reservation/
├── siar-load-balancer/
├── siar-fairness/
├── siar-congestion/
├── siar-capacity-market/
├── siar-resource-observability/
└── siar-resource-testkit/
```

---

# 243. `siar-resource-core`

Owns:

```text
resource types
priority
demand classes
errors
```

---

# 244. `siar-capacity-catalog`

Capacity advertisements/cache.

---

# 245. `siar-allocation-policy`

Hard privacy/cost/operator constraints.

---

# 246. `siar-resource-scheduler`

Client-side allocation.

---

# 247. `siar-resource-reservation`

Reservation lifecycle.

---

# 248. `siar-load-balancer`

Weighted/randomized balancing.

---

# 249. `siar-fairness`

Provider-side scoped fair scheduling.

---

# 250. `siar-congestion`

Backpressure/congestion signaling.

---

# 251. `siar-capacity-market`

Signed capacity/price snapshots.

---

# 252. `siar-resource-observability`

Privacy-safe metrics.

---

# 253. `siar-resource-testkit`

Deterministic load/capacity simulator.

---

# 254. Error Taxonomy

```rust
pub enum AllocationError {
    NoEligibleProvider,
    CapacityUnavailable,
    ReservationRejected,
    QuotaExceeded,
    PrivacyConstraintUnsatisfied,
    OperatorDiversityUnsatisfied,
    PriceConstraint,
    Congested,
    ReservationExpired,
    Internal,
}
```

---

# 255. Initial Production Scope

Implement first:

```text
signed capacity classes
local weighted-random provider allocation
operator exposure constraints
reservation for realtime/bulk
quota-aware scheduling
priority classes
bounded fair queues
coarse congestion state
jittered retry/failover
signed capacity/price snapshots
privacy-safe diagnostics
deterministic capacity testkit
```

Then add:

```text
advanced anonymous reservation tokens
predictive local pre-reservation
multi-market federation
privacy-preserving capacity auctions
adaptive operator balancing
differentially private public capacity statistics
```

---

# 256. Definition of Done

Part 49 is complete when:

- resource types and priorities are explicit
- capacity advertisements are signed and coarse
- client demand remains local/minimized
- hard privacy constraints precede load/cost optimization
- allocation is randomized enough to avoid herd behavior
- operator diversity and cross-service exposure are constrained
- reservations are scoped, expiring, idempotent, and crash-safe
- provider-side fairness works without global user identity
- congestion/backpressure and priority scheduling are defined
- capacity markets publish broad signed snapshots rather than personalized quotes
- emergency priority cannot silently reduce privacy
- allocation/accounting/settlement remain identity-separated
- telemetry and support bundles exclude communication identifiers
- load, fairness, herding, failover, reservation, privacy, fuzz, and formal tests are specified

---

# 257. Final Architecture

```text
                 SIGNED CAPACITY / PRICE MARKET
                              │
                              ▼
                     LOCAL ELIGIBILITY
                              │
                              ▼
                  PRIVACY / OPERATOR FILTERS
                              │
                              ▼
                  WEIGHTED LOAD BALANCING
                              │
                   ┌──────────┴──────────┐
                   │                     │
              Immediate               Reserved
                   │                     │
                   ▼                     ▼
             Provider Slot        Capacity Reservation
                   │                     │
                   └──────────┬──────────┘
                              ▼
                      QUOTA / FAIRNESS
                              │
                              ▼
                         SERVICE USE
```

Strict allocation model:

```text
privacy constraints
+
operator diversity
+
capacity class
+
randomized load balancing
+
scoped reservation
+
fairness
+
quota
```

not:

```text
central scheduler tracks every user and assigns them resources
```

---

# 258. Final Principle

Capacity is an anonymity concern, not merely an operations concern.

The correct model is:

```text
signed coarse capacity
+
local demand
+
privacy-first allocation
+
randomized load balancing
+
scoped reservations
+
fair scheduling
+
identity-separated settlement
```

This architecture gives SIAR a scalable way to allocate anonymous infrastructure under real-world load without creating a central database of who used which service, when, and for whom.
