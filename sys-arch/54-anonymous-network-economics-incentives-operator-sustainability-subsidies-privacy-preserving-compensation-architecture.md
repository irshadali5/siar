# Core System Architecture Part 54 — Anonymous Network Economics, Incentives, Operator Sustainability, Subsidies & Privacy-Preserving Compensation Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 54  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–53  

**Primary purpose:** define the complete economic and sustainability architecture for SIAR's anonymous infrastructure, including operator compensation, sustainable pricing, subsidies, reserve pools, public-good funding, anti-gaming controls, privacy-preserving settlement, emergency support, and strict separation between economics, routing, identity, and governance.

---

# 1. Purpose

Anonymous infrastructure costs money.

Operators pay for:

```text
servers
bandwidth
storage
power
cloud egress
hardware
security
monitoring
staff
compliance
```

If incentives are poorly designed, the network may suffer from:

```text
centralization
pay-to-win routing
fake capacity
Sybil operators
traffic manipulation
speculative token economics
unfair subsidies
operator churn
```

The governing principle is:

> **SIAR economics should make privacy-preserving infrastructure sustainable without turning operator compensation into a mechanism for user tracking, routing manipulation, speculation, or governance capture.**

---

# 2. Architectural Position

```text
User / Organization Funding
          │
          ▼
      Credit Issuance
          │
          ▼
 Service-Specific Usage
          │
          ▼
 Signed Service Receipts
          │
          ▼
 Aggregate Settlement
          │
          ▼
 Operator Compensation
          │
          ├── direct usage revenue
          ├── subsidies
          ├── grants
          └── reserve support
```

---

# 3. Core Separation

Keep distinct:

```text
user payment
service usage
operator compensation
governance influence
routing weight
reputation
subsidy eligibility
```

---

# 4. Non-Goals

Part 54 does not require:

```text
cryptocurrency
tradeable network token
proof-of-stake governance
speculative mining
user identity-linked billing
operator voting power based on wealth
```

---

# 5. Economic Actors

```rust
pub enum EconomicActor {
    User,
    Organization,
    ServiceProvider,
    Operator,
    SubsidyFund,
    GovernanceBody,
    SettlementService,
}
```

---

# 6. Service Provider

Offers:

```text
mailbox
relay
bulk storage
mix gateway
bridge
```

---

# 7. Operator

Controls one or more providers.

---

# 8. Subsidy Fund

Supports services that cannot rely solely on direct user fees.

---

# 9. Settlement Service

Aggregates receipts and pays operators.

---

# 10. Economic Principles

Mandatory:

```text
privacy before profit
no pay-to-route preference
no wealth-based governance
no user-level payout tracing
no packet-level payment linkage
```

---

# 11. Operator Cost Model

Operators have real costs.

---

# 12. Cost Classes

```rust
pub enum OperatorCostClass {
    Compute,
    Bandwidth,
    Storage,
    Egress,
    Power,
    Hardware,
    Operations,
    Security,
}
```

---

# 13. Cost Model

```rust
pub struct OperatorCostModel {
    pub fixed_monthly: u64,
    pub variable: Vec<VariableCost>,
}
```

---

# 14. Variable Cost

```rust
pub struct VariableCost {
    pub class: OperatorCostClass,
    pub unit: ResourceUnit,
    pub cost_per_unit: u64,
}
```

---

# 15. Why Cost Model Matters

Sustainable prices should be grounded in:

```text
real operational cost
+
reasonable reserve
+
operator margin
```

not arbitrary speculation.

---

# 16. Revenue Sources

Potential:

```text
usage fees
subscriptions
organization grants
public-good subsidies
donations
operator sponsorship
```

---

# 17. Revenue Mix

A resilient network should not depend on one source.

---

# 18. Direct Usage Revenue

Part 47 service credits.

---

# 19. Subscription Revenue

Subscription can periodically issue:

```text
resource credits
```

without provider learning user account identity.

---

# 20. Organization Grants

Useful for:

```text
schools
NGOs
enterprises
communities
```

---

# 21. Public-Good Subsidy

Supports services such as:

```text
bridges
entry gateways
emergency capacity
```

where direct metering is harmful or impractical.

---

# 22. Donation Funding

Can feed subsidy pool.

---

# 23. No Routing Influence From Donation

Hard rule.

---

# 24. Operator Revenue Classes

```rust
pub enum OperatorRevenueClass {
    Usage,
    Reservation,
    SubscriptionAllocation,
    Subsidy,
    Grant,
}
```

---

# 25. Settlement Flow

```text
service usage
→ signed receipt
→ aggregation
→ operator settlement
```

---

# 26. Aggregate Settlement

Operator receives:

```text
total valid units
total compensation
period
service class
```

---

# 27. No User Mapping

Hard rule.

---

# 28. Operator Settlement

```rust
pub struct OperatorEconomicSettlement {
    pub operator: ProviderOperatorId,
    pub period: AccountingPeriod,
    pub service: AnonymousServiceType,
    pub billable_units: u64,
    pub gross_amount: u64,
    pub subsidy_amount: u64,
    pub fees: u64,
    pub net_amount: u64,
}
```

---

# 29. Settlement Inputs

Only:

```text
valid receipts
pricing
subsidy policy
operator eligibility
```

---

# 30. Routing Must Not Depend On Operator Revenue

Hard rule.

---

# 31. Pay-To-Win Risk

If high-paying operator gets more traffic, centralization follows.

---

# 32. Therefore

Provider selection from Part 48 remains based on:

```text
privacy
eligibility
diversity
health
capacity
cost
```

not operator sponsorship.

---

# 33. Incentive Goals

Incentives should encourage:

```text
availability
capacity
diversity
security
long-term participation
```

---

# 34. Incentive Signals

Potential:

```text
uptime class
capacity delivered
successful reservations
audit status
region diversity
```

---

# 35. No Per-User Performance Reward

Hard rule.

---

# 36. Operator Quality Multiplier

Possible.

---

# 37. Quality Multiplier

```rust
pub struct OperatorQualityMultiplier(pub f32);
```

---

# 38. Inputs

Only aggregate:

```text
uptime
service correctness
policy compliance
capacity reliability
```

---

# 39. Cap Multiplier

Prevent runaway advantage.

---

# 40. New Operator Support

New operators may need bootstrap support.

---

# 41. Probation Subsidy

```rust
pub struct NewOperatorSubsidy {
    pub duration: Duration,
    pub max_amount: u64,
}
```

---

# 42. Goal

Reduce entry barrier without granting full traffic dominance.

---

# 43. Operator Diversity Subsidy

Potentially reward:

```text
underrepresented region
underrepresented ASN
independent operator
```

---

# 44. Privacy Benefit

More diversity improves anonymity resilience.

---

# 45. Subsidy Rule

Must not rely on user identity.

---

# 46. Subsidy Categories

```rust
pub enum SubsidyCategory {
    Diversity,
    Emergency,
    PublicGood,
    Bootstrap,
    CapacityReserve,
    CensorshipResistance,
}
```

---

# 47. Diversity Subsidy

Supports regions/operators lacking capacity.

---

# 48. Emergency Subsidy

Funds spare capacity during disaster.

---

# 49. Public-Good Subsidy

For infrastructure with weak direct monetization.

---

# 50. Bootstrap Subsidy

For early network growth.

---

# 51. Capacity Reserve Subsidy

Pays operators to maintain unused headroom.

---

# 52. Censorship-Resistance Subsidy

Supports:

```text
bridges
alternate transports
```

---

# 53. Subsidy Fund

```rust
pub struct SubsidyFund {
    pub fund_id: SubsidyFundId,
    pub category: SubsidyCategory,
    pub budget: u64,
    pub policy_version: GovernancePolicyVersion,
}
```

---

# 54. Subsidy Governance

Part 53 governance controls:

```text
eligibility
budget
duration
review
```

---

# 55. No Automatic Unlimited Subsidy

Hard rule.

---

# 56. Budget Caps

Required.

---

# 57. Time-Limited Programs

Preferred.

---

# 58. Subsidy Eligibility

Could require:

```text
active operator
healthy provider
minimum audit class
capacity commitment
```

---

# 59. Subsidy Claim

```rust
pub struct SubsidyClaim {
    pub operator: ProviderOperatorId,
    pub category: SubsidyCategory,
    pub period: AccountingPeriod,
    pub evidence: SubsidyEvidence,
}
```

---

# 60. Subsidy Evidence

Aggregate only.

---

# 61. Examples

```text
capacity maintained
uptime bucket
independent region
bridge availability
```

---

# 62. No User-Level Subsidy Evidence

Hard rule.

---

# 63. Public-Good Funding Model

Some services should be funded from pool.

---

# 64. Good Candidates

```text
directory mirrors
bridge infrastructure
emergency relays
public bootstrap gateways
```

---

# 65. Why

Charging every use may:

```text
harm access
increase linkage
reduce resilience
```

---

# 66. Cross-Subsidy

Profitable services can fund public goods.

---

# 67. Cross-Subsidy Pool

```rust
pub struct CrossSubsidyPolicy {
    pub source_services: Vec<AnonymousServiceType>,
    pub target_categories: Vec<SubsidyCategory>,
    pub contribution_rate: f32,
}
```

---

# 68. Contribution Rate

Governed.

---

# 69. No Hidden Tax

User-facing pricing should explain service fee structure at product level.

---

# 70. Reserve Fund

Network should maintain financial reserve.

---

# 71. Reserve Purposes

```text
region outage
capacity spike
operator exit
emergency deployment
```

---

# 72. Reserve Policy

```rust
pub struct NetworkReservePolicy {
    pub minimum_months: u8,
    pub maximum_draw_per_period: u64,
}
```

---

# 73. Reserve Isolation

Reserve cannot be casually used for unrelated spending.

---

# 74. Operator Exit Risk

Provider may shut down.

---

# 75. Exit Support

Could fund:

```text
migration
grace period
temporary replacement capacity
```

---

# 76. Exit Bond

Potential future.

---

# 77. Bond Tradeoff

Can discourage small operators.

---

# 78. Initial Recommendation

Do not require large financial bond initially.

---

# 79. Alternative

Use:

```text
probation
capacity caps
reputation
reserve fund
```

---

# 80. Sybil Operator Economics

Attacker may create many operators to capture subsidies.

---

# 81. Defense

Use Part 39:

```text
operator identity
admission
probation
diversity validation
```

---

# 82. Subsidy Sybil Defense

Do not pay per operator count.

---

# 83. Better

Pay for:

```text
verified useful capacity
independent failure domain
audited operation
```

---

# 84. Fake Capacity

Operator advertises capacity to earn subsidy.

---

# 85. Defense

Independent probes.

---

# 86. Paid Capacity Verification

```rust
pub struct VerifiedCapacityRecord {
    pub operator: ProviderOperatorId,
    pub service: AnonymousServiceType,
    pub capacity_class: CapacityClass,
    pub verification: CapacityVerificationClass,
}
```

---

# 87. Capacity Verification Class

```rust
pub enum CapacityVerificationClass {
    SelfReported,
    Observed,
    IndependentlyVerified,
}
```

---

# 88. Subsidy Weight

Higher assurance gets higher confidence.

---

# 89. Uptime Incentive

Potential compensation bonus.

---

# 90. Avoid Perfection Trap

Do not make small outage economically catastrophic.

---

# 91. Use Buckets

```text
Healthy
Good
Degraded
Poor
```

---

# 92. Uptime Class

```rust
pub enum EconomicUptimeClass {
    Excellent,
    Good,
    Acceptable,
    Poor,
}
```

---

# 93. Compensation Curve

Should be smooth.

---

# 94. No Cliff Incentive

Avoid:

```text
99.9% gets paid
99.89% gets nothing
```

---

# 95. Capacity Commitment

Provider may commit reserve capacity.

---

# 96. Commitment Contract

```rust
pub struct CapacityCommitment {
    pub operator: ProviderOperatorId,
    pub resource: InfrastructureResource,
    pub class: CapacityClass,
    pub period: AccountingPeriod,
}
```

---

# 97. Reserve Compensation

Pay for maintaining headroom even if unused.

---

# 98. Why

Needed for disaster resilience.

---

# 99. Emergency Draw

During Part 50 disaster mode, reserve capacity activates.

---

# 100. Emergency Premium

Could compensate higher costs.

---

# 101. Privacy

Still aggregate.

---

# 102. Bridge Economics

Bridges are often censorship-sensitive.

---

# 103. Direct Billing Risk

Bridge payments may reveal:

```text
who needs censorship bypass
```

---

# 104. Recommendation

Subsidize bridges from public-good pool.

---

# 105. Realtime Relay Economics

Can be usage-based.

---

# 106. Mailbox Economics

Can use storage/retention credits.

---

# 107. Bulk Economics

Can use storage + egress.

---

# 108. Mixnet Economics

Per-packet payment is dangerous.

---

# 109. Better

Use:

```text
gateway quota
provider settlement aggregate
subsidy/revenue pool
```

---

# 110. Mix Node Compensation

Potentially based on:

```text
capacity
uptime
validated forwarding availability
```

not individual packet receipts.

---

# 111. Why

Avoid packet-level economic fingerprint.

---

# 112. Node Compensation Pool

```rust
pub struct MixnetCompensationPool {
    pub period: AccountingPeriod,
    pub budget: u64,
}
```

---

# 113. Distribution Factors

```text
eligible capacity
availability
operator diversity
layer participation
```

---

# 114. Cap Dominance

One operator cannot capture unlimited share.

---

# 115. Maximum Operator Share

Governance policy.

---

# 116. Economic Concentration Metric

```rust
pub struct EconomicConcentration {
    pub operator_share: f32,
    pub top_n_share: f32,
}
```

---

# 117. High Concentration

Triggers:

```text
review
subsidy adjustment
new-operator incentives
```

---

# 118. No Forced Routing Redistribution

Economic concern should not override privacy.

---

# 119. Anti-Centralization Subsidy

Can support underrepresented providers.

---

# 120. Price Competition

Providers may set prices.

---

# 121. Price Floor

Potential to prevent predatory underpricing.

---

# 122. Price Ceiling

Potential for essential public services.

---

# 123. Governance Caution

Avoid over-central planning.

---

# 124. Initial Recommendation

Use:

```text
transparent signed prices
local selection
subsidy for public goods
```

rather than hard price controls.

---

# 125. Free Tier

Important for accessibility.

---

# 126. Free Tier Funding

Can come from:

```text
cross-subsidy
donations
reserve
organization grants
```

---

# 127. Free Tier Abuse

Use Part 47 quota controls.

---

# 128. No Identity Requirement Just For Free Tier

Hard rule where avoidable.

---

# 129. Community-Sponsored Access

Organization can issue credits.

---

# 130. Sponsorship Privacy

Organization may know recipient.

Provider should not learn communication details.

---

# 131. Gift Credits

Issuer-mediated.

---

# 132. User-to-User Transfer

Still discouraged in initial design.

---

# 133. Operator Payout Privacy

Operator payout account is not user privacy issue.

---

# 134. Operator Financial Identity

May be known to settlement system.

---

# 135. Separate From Provider Runtime Identity

Hard rule.

---

# 136. Payout Adapter

```rust
pub trait OperatorPayoutAdapter {
    fn payout(
        &self,
        settlement: &OperatorEconomicSettlement,
    ) -> Result<PayoutReceipt, EconomicError>;
}
```

---

# 137. Payout Methods

Architecture can support:

```text
bank
payment processor
organization invoice
```

---

# 138. No Requirement For Crypto Payment

Hard rule.

---

# 139. Settlement Currency

Can be fiat.

---

# 140. Internal Resource Unit

Separate from currency.

---

# 141. Why

Avoid coupling protocol to one monetary system.

---

# 142. Economic Unit

```rust
pub struct EconomicUnit(pub u64);
```

---

# 143. Conversion

Price catalog maps resource unit to currency/credit.

---

# 144. Multi-Currency

Funding layer can support.

---

# 145. Provider Price Publication

Signed.

---

# 146. Price Change Frequency

Coarse.

---

# 147. No Millisecond Dynamic Pricing

Hard rule.

---

# 148. Market Stability

Avoid extreme oscillation.

---

# 149. Price Smoothing

Potential:

```text
epoch average
bounded delta
```

---

# 150. Maximum Price Delta

Governance/business policy.

---

# 151. Subsidy Smoothing

Likewise.

---

# 152. Economic Attack

Provider artificially congests service to trigger higher subsidy/price.

---

# 153. Defense

Independent health/capacity evidence.

---

# 154. Demand Manipulation

Attacker generates fake load to affect market.

---

# 155. Defense

Use:

```text
validated reservations
paid quota
aggregate demand
```

---

# 156. Wash Usage

Operator generates fake usage to pay itself.

---

# 157. Defense

Credits/receipts must represent genuine external spend where applicable.

---

# 158. Public-Good Service

For subsidy-funded service, rely on:

```text
independent availability proof
capacity evidence
```

---

# 159. No Self-Reported Payout Only

Hard rule.

---

# 160. Settlement Fraud

Potential:

```text
forged receipt
duplicate receipt
inflated unit count
```

---

# 161. Defense

Part 47 signed receipt/idempotent settlement.

---

# 162. Subsidy Fraud

Potential:

```text
fake operator
fake region
fake capacity
```

---

# 163. Defense

Part 39/48 governance + independent verification.

---

# 164. Economic Reputation

Keep separate from privacy trust.

---

# 165. Operator Economic Record

Can track:

```text
settlement correctness
capacity commitment
subsidy compliance
```

---

# 166. No User Reputation Mixing

Hard rule.

---

# 167. Operator Economic State

```rust
pub enum OperatorEconomicState {
    Normal,
    UnderReview,
    SubsidySuspended,
    SettlementSuspended,
}
```

---

# 168. Suspension

Does not automatically revoke network operation unless governance separately decides.

---

# 169. Why

Economic dispute ≠ security compromise.

---

# 170. Governance Separation

Part 53 controls operator revocation.

---

# 171. Economic System Cannot Revoke Anonymity Provider Alone

Hard rule.

---

# 172. Subsidy Governance Workflow

```text
proposal
→ budget approval
→ eligibility rules
→ signed program
→ claims
→ audit
→ renewal/expiry
```

---

# 173. Subsidy Program

```rust
pub struct SubsidyProgram {
    pub program_id: SubsidyProgramId,
    pub category: SubsidyCategory,
    pub budget: u64,
    pub start: Timestamp,
    pub end: Timestamp,
    pub rules: SubsidyEligibilityRules,
    pub policy_version: GovernancePolicyVersion,
}
```

---

# 174. Mandatory End Date

Preferred.

---

# 175. Renewal

Explicit governance action.

---

# 176. Emergency Subsidy

Short expiry.

---

# 177. Subsidy Claim Review

Automated where evidence clear.

Manual for anomaly.

---

# 178. Claim Privacy

No user data.

---

# 179. Public Transparency

Can publish:

```text
subsidy budgets
operator grants
program totals
```

---

# 180. Privacy

Operator-level financial data may be public if policy says so.

---

# 181. User Usage Not Public

Hard rule.

---

# 182. Economic Transparency Report

Possible:

```text
total operator payouts
public-good spending
reserve level
concentration
```

---

# 183. Differential Privacy

Likely unnecessary for large aggregate financial totals.

---

# 184. Small Operator Consideration

Avoid publishing highly granular traffic-derived payout if it reveals sensitive load.

---

# 185. Coarse Disclosure

Can publish ranges.

---

# 186. Sustainability SLOs

Examples:

```text
operator churn
capacity reserve
public-good coverage
settlement delay
```

---

# 187. Economic Health Classes

```rust
pub enum NetworkEconomicHealth {
    Healthy,
    Tight,
    Underfunded,
    Critical,
}
```

---

# 188. Underfunded

Means:

```text
reserve or subsidy insufficient
```

---

# 189. User Impact

May cause:

```text
higher price
lower free tier
reduced public-good capacity
```

but not privacy downgrade.

---

# 190. Hard Rule

Financial shortage never enables weaker anonymity path automatically.

---

# 191. Economic Forecasting

Useful.

---

# 192. Inputs

Aggregate:

```text
capacity
cost trends
settlement
reserve
growth
```

---

# 193. No Per-User Forecasting Needed

Hard rule.

---

# 194. Forecast Model

Local control-plane service.

---

# 195. Cost Shock

Example:

```text
cloud egress rises
```

---

# 196. Response

```text
price update
subsidy adjustment
operator migration
```

---

# 197. Multi-Operator Sustainability

Avoid dependency on one subsidized operator.

---

# 198. Minimum Operator Count

Can be governance target.

---

# 199. Economic Diversity Target

```rust
pub struct EconomicDiversityTarget {
    pub minimum_operators: usize,
    pub maximum_top_operator_share: f32,
}
```

---

# 200. Incentive for Underrepresented Region

Can help resilience.

---

# 201. Carbon/Energy Consideration

Optional.

---

# 202. Energy Efficiency Incentive

Could reward:

```text
lower energy per resource unit
```

---

# 203. Privacy

No user linkage.

---

# 204. Caution

Do not make opaque environmental scoring core to anonymity.

---

# 205. Operator Reporting

Provider can submit:

```text
cost class
capacity
resource usage
```

---

# 206. Self-Reported Cost

Not fully trusted.

---

# 207. Reimbursement Programs

Need evidence.

---

# 208. General Subsidy

Better than exact cost reimbursement in many cases.

---

# 209. Why

Reduces accounting complexity and incentive to inflate costs.

---

# 210. Operator Marketplace Exit

If revenue falls:

```text
retire provider gracefully
```

---

# 211. Exit Notice

Signed.

---

# 212. Continuity

Part 50 migrates affected services.

---

# 213. New Operator Entry

Part 39/48 admission.

---

# 214. Economic Bootstrap

Part 54 can support initial subsidy.

---

# 215. Infrastructure Grant

```rust
pub struct InfrastructureGrant {
    pub operator: ProviderOperatorId,
    pub purpose: SubsidyCategory,
    pub amount: u64,
    pub period: AccountingPeriod,
}
```

---

# 216. Grant Milestones

Potential:

```text
deploy
pass audit
maintain capacity
```

---

# 217. Clawback

Possible for fraud.

---

# 218. Clawback Governance

Requires evidence and due process.

---

# 219. No Automatic Clawback For Outage

Hard rule.

---

# 220. Reserve Draw

Governed event.

---

# 221. Reserve Draw Record

```rust
pub struct ReserveDraw {
    pub amount: u64,
    pub reason: ReserveDrawReason,
    pub policy_version: GovernancePolicyVersion,
}
```

---

# 222. Reserve Draw Reason

```rust
pub enum ReserveDrawReason {
    Disaster,
    CapacityShortage,
    OperatorExit,
    EmergencyBridgeDeployment,
}
```

---

# 223. Reserve Replenishment

From:

```text
service fees
donations
budget allocation
```

---

# 224. Network Foundation Model

Possible organizational model.

---

# 225. Architecture Neutrality

SIAR protocol should not require one legal entity.

---

# 226. Multiple Funding Entities

Future.

---

# 227. Federation

Different governance domains may fund own operators.

---

# 228. Economic Interoperability

Resource credits could remain domain-specific.

---

# 229. Cross-Domain Settlement

Future complexity.

---

# 230. Initial Recommendation

Single settlement/governance domain.

---

# 231. Compensation Policy

```rust
pub struct CompensationPolicy {
    pub version: GovernancePolicyVersion,
    pub service_rates: Vec<ServiceCompensationRate>,
    pub quality_multipliers: Vec<QualityRule>,
    pub concentration_caps: EconomicDiversityTarget,
}
```

---

# 232. Compensation Rate

```rust
pub struct ServiceCompensationRate {
    pub service: AnonymousServiceType,
    pub resource: ResourceUnit,
    pub rate: u64,
}
```

---

# 233. Policy Signed

Part 53 governance.

---

# 234. No Hidden Compensation Algorithm

Hard rule.

---

# 235. Operator Can Reproduce Settlement

Recommended.

---

# 236. Settlement Determinism

Same inputs -> same result.

---

# 237. Settlement Engine

```rust
pub trait OperatorSettlementEngine {
    fn settle(
        &self,
        period: AccountingPeriod,
        receipts: &[ValidatedServiceReceipt],
        policy: &CompensationPolicy,
    ) -> Result<Vec<OperatorEconomicSettlement>, EconomicError>;
}
```

---

# 238. Subsidy Engine

```rust
pub trait SubsidyEngine {
    fn evaluate(
        &self,
        claim: &SubsidyClaim,
        program: &SubsidyProgram,
    ) -> Result<SubsidyDecision, EconomicError>;
}
```

---

# 239. Subsidy Decision

```rust
pub enum SubsidyDecision {
    Approved(u64),
    Rejected(SubsidyRejectionReason),
    ManualReview,
}
```

---

# 240. Economic Health Monitor

```rust
pub trait EconomicHealthMonitor {
    fn evaluate(
        &self,
        state: &AggregateEconomicState,
    ) -> NetworkEconomicHealth;
}
```

---

# 241. Aggregate Economic State

Contains:

```text
reserve
operator churn
settlement delay
capacity coverage
```

---

# 242. No User Billing Details

Hard rule.

---

# 243. Operator Cost Privacy

Operators may consider exact costs commercially sensitive.

---

# 244. Governance View

Can use coarse ranges.

---

# 245. Public View

Even coarser.

---

# 246. Financial Audit

May require detailed operator financial records.

---

# 247. Separate From Network Telemetry

Hard rule.

---

# 248. Financial Auditor

Does not gain user traffic data.

---

# 249. Tax/Compliance

Operator payouts may need tax documentation.

---

# 250. Keep At Settlement Boundary

Hard rule.

---

# 251. No Tax Identity In Provider Protocol

Hard rule.

---

# 252. Economic Observability

Safe metrics:

```text
settlement latency
reserve level
operator count
capacity funded
subsidy burn
```

---

# 253. Forbidden Metrics

No:

```text
user payment + provider usage linkage
contact-level spend
route-level revenue
```

---

# 254. Economic Incident Types

```rust
pub enum EconomicIncident {
    SettlementBacklog,
    SubsidyFraud,
    ReserveShortfall,
    OperatorExit,
    CapacityUnderfunding,
    PricingError,
}
```

---

# 255. Pricing Error

Could harm availability.

---

# 256. Response

Correct price policy.

No security bypass.

---

# 257. Settlement Outage

Providers may continue using accrued signed receipts.

---

# 258. Deferred Settlement

Supported.

---

# 259. Long Outage

Reserve liquidity may matter.

---

# 260. Operator Credit Line

Future business mechanism.

---

# 261. Not Protocol Requirement

Hard rule.

---

# 262. Economic Abuse Tests

Need simulator.

---

# 263. Scenarios

```text
one operator underprices
fake capacity
wash usage
subsidy Sybil
mass operator exit
cost shock
reserve depletion
```

---

# 264. Market Concentration Simulation

Track:

```text
operator share
regional diversity
service diversity
```

---

# 265. Sustainability Simulation

Project:

```text
revenue
cost
subsidy
reserve
```

---

# 266. Privacy Simulation

Check compensation data cannot reconstruct user behavior.

---

# 267. Wash Usage Test

Operator cannot trivially pay itself with forged usage.

---

# 268. Fake Capacity Test

Subsidy requires independent evidence.

---

# 269. Sybil Subsidy Test

Many related operators cannot multiply subsidy unfairly.

---

# 270. Concentration Test

Subsidy does not create dominant operator.

---

# 271. Price Shock Test

Client/provider market stabilizes.

---

# 272. Reserve Shock Test

Network continues under disaster.

---

# 273. Settlement Replay Test

Duplicate settlement rejected.

---

# 274. Claim Replay Test

Duplicate subsidy claim rejected.

---

# 275. Fuzzing

Fuzz:

```text
compensation policy
subsidy claim
settlement batch
reserve draw
```

---

# 276. Property Tests

Properties:

```text
same receipt cannot pay twice
operator compensation does not alter routing eligibility
subsidy claim cannot exceed program budget
reserve draw cannot exceed configured policy
```

---

# 277. Formal Verification Targets

Strong candidates:

```text
settlement idempotency
subsidy budget accounting
reserve policy
concentration cap enforcement
```

---

# 278. TLA+ Candidate

Subsidy allocation under concurrent claims.

---

# 279. Kani Candidate

Settlement arithmetic/overflow/budget bounds.

---

# 280. Loom Candidate

Concurrent settlement batch processing.

---

# 281. Performance Tests

Measure:

```text
receipt aggregation
settlement computation
subsidy evaluation
market simulation
```

---

# 282. Scale Tests

Millions of receipts.

Thousands of providers.

---

# 283. Precision

Use integer/fixed-point arithmetic.

---

# 284. No Floating Money Arithmetic

Hard rule.

---

# 285. Decimal Representation

Use exact integer smallest unit or reviewed fixed-decimal type.

---

# 286. Overflow Safety

Checked arithmetic.

---

# 287. Rounding

Explicit policy.

---

# 288. Crate Layout

Recommended:

```text
crates/
├── siar-economics-core/
├── siar-operator-cost/
├── siar-compensation/
├── siar-subsidy/
├── siar-reserve/
├── siar-economic-policy/
├── siar-economic-concentration/
├── siar-economic-health/
├── siar-payout/
├── siar-economic-observability/
└── siar-economic-testkit/
```

---

# 289. `siar-economics-core`

Owns:

```text
economic types
amounts
periods
errors
```

---

# 290. `siar-operator-cost`

Cost classes/models.

---

# 291. `siar-compensation`

Settlement/payout calculation.

---

# 292. `siar-subsidy`

Programs/claims/decisions.

---

# 293. `siar-reserve`

Reserve draw/replenishment.

---

# 294. `siar-economic-policy`

Signed compensation/subsidy policy.

---

# 295. `siar-economic-concentration`

Operator concentration analysis.

---

# 296. `siar-economic-health`

Network sustainability state.

---

# 297. `siar-payout`

External payout adapters.

---

# 298. `siar-economic-observability`

Privacy-safe financial health metrics.

---

# 299. `siar-economic-testkit`

Economic attack/sustainability simulator.

---

# 300. Error Taxonomy

```rust
pub enum EconomicError {
    SettlementDuplicate,
    ReceiptInvalid,
    SubsidyBudgetExceeded,
    ClaimInvalid,
    OperatorIneligible,
    ReserveInsufficient,
    PolicyMismatch,
    PriceInvalid,
    PayoutFailed,
    ArithmeticOverflow,
    Internal,
}
```

---

# 301. Security & Economic Invariants

Mandatory:

```text
1. Operator compensation never requires user identity or social-graph data.
2. Routing/provider eligibility is not increased because an operator pays more or receives sponsorship.
3. No packet-level payment identity exists inside anonymous routing.
4. Subsidies are based on verified aggregate service/capacity evidence, not raw user records.
5. Operator payout identity is isolated from provider runtime identity.
6. Economic state cannot weaken privacy/security policy.
7. Economic disputes do not automatically become network revocation decisions.
8. Subsidy and settlement processing are idempotent and budget-bounded.
9. No speculative token or stake is required for SIAR governance.
10. Financial audit data remains separate from user traffic telemetry.
11. Concentration caps prevent one operator from capturing excessive subsidy/compensation share where policy requires.
12. Network underfunding degrades capacity/features, never silently anonymity.
```

---

# 302. Initial Production Scope

Implement first:

```text
service-usage settlement
aggregate operator payouts
fixed compensation policy
public-good subsidy programs
capacity reserve subsidy
network reserve fund
operator diversity/concentration monitoring
signed economic policies
privacy-safe economic metrics
deterministic settlement engine
```

Then add:

```text
advanced independent capacity verification
multi-funder subsidy pools
cross-domain settlement
formal economic simulations
privacy-preserving operator grant mechanisms
```

---

# 303. Definition of Done

Part 54 is complete when:

- real operator cost classes are explicit
- direct usage revenue, grants, subsidy, reserve, and public-good funding are separated
- operator compensation uses aggregate signed receipts
- no user identity is required for operator payout
- routing/provider selection is economically neutral
- public-good and censorship-resistance subsidies are defined
- reserve capacity and disaster support are fundable
- subsidy Sybil/fake-capacity/wash-usage attacks are addressed
- operator concentration and sustainability are monitored
- governance controls economic policy, but economics cannot override privacy floors
- financial/tax identity remains at settlement boundary
- exact arithmetic and settlement idempotency are required
- economic shocks cause capacity/pricing changes, not privacy downgrade
- sustainability, concentration, fraud, fuzz, and formal tests are specified

---

# 304. Final Architecture

```text
                     USER / ORG FUNDING
                             │
                             ▼
                      CREDIT / REVENUE
                             │
                             ▼
                    SERVICE USAGE RECEIPTS
                             │
                             ▼
                    AGGREGATE SETTLEMENT
                             │
             ┌───────────────┼───────────────┐
             │               │               │
         Operator Pay     Subsidy Pool    Reserve Fund
             │               │               │
             └───────────────┼───────────────┘
                             ▼
                 SUSTAINABLE INFRASTRUCTURE
```

Economic safety model:

```text
real cost
+
transparent price
+
aggregate receipts
+
operator compensation
+
subsidy
+
reserve
+
anti-concentration controls
```

not:

```text
token speculation
+
pay-to-route
+
user-level payout tracking
```

---

# 305. Final Principle

Economic sustainability is necessary for anonymity infrastructure, but economics must remain **subordinate to privacy, diversity, and security**.

The correct model is:

```text
usage-based revenue
+
public-good subsidy
+
operator sustainability
+
aggregate privacy-preserving settlement
+
anti-centralization incentives
```

without making identity, routing, or governance dependent on wealth.

This architecture gives SIAR a viable path to fund long-lived anonymous infrastructure while preserving the privacy, operator-diversity, and governance guarantees established across Parts 34–53.
