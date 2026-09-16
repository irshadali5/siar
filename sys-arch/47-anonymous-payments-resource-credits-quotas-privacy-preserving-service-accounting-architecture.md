# Core System Architecture Part 47 — Anonymous Payments, Resource Credits, Quotas & Privacy-Preserving Service Accounting Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 47  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–46  

**Primary purpose:** define privacy-preserving payment, resource-credit, quota, metering, receipt, refund, and service-accounting architecture for SIAR's anonymous infrastructure without turning payment identity into a universal communication identifier.

---

# 1. Purpose

Anonymous infrastructure consumes real resources:

```text
mixnet bandwidth
bridge bandwidth
mailbox storage
anonymous bulk storage
realtime relay capacity
CPU
egress
long-term retention
```

If SIAR provides paid or metered infrastructure, naive billing can destroy anonymity.

Example failure:

```text
payment account
→ relay session
→ mailbox
→ file transfer
→ contact identity
```

becomes one globally correlatable graph.

The governing principle is:

> **Payment and resource accounting must be separated from communication identity, scoped to the minimum service context, and designed so providers can enforce quotas without learning more about the user than necessary.**

---

# 2. Architectural Position

```text
User Payment / Funding
        │
        ▼
Privacy-Preserving Credit Issuer
        │
        ▼
Scoped Resource Credentials
        │
        ├── Mixnet
        ├── Mailbox
        ├── Relay
        ├── Bulk Storage
        └── Other Services
        │
        ▼
Usage Meter / Quota Enforcement
        │
        ▼
Receipt / Settlement
```

---

# 3. Core Separation

Keep distinct:

```text
Funding Identity
Credit Identity
Service Credential
Communication Identity
Accounting Record
Operator Settlement
```

---

# 4. Funding Identity

The payment processor may know:

```text
payer
bank/card/wallet account
```

depending funding method.

That identity must not automatically propagate into service usage.

---

# 5. Credit Identity

Prefer unlinkable or pseudonymous credit instruments.

---

# 6. Service Credential

Each service gets a scoped credential.

---

# 7. Communication Identity

Never use:

```text
AccountId
DeviceId
ContactId
GroupId
MailboxId
```

as payment-account keys.

---

# 8. Accounting Record

Contains:

```text
resource category
usage amount
price class
receipt state
```

not conversation relationships.

---

# 9. Operator Settlement

Infrastructure operators may receive aggregate settlement without receiving user identity.

---

# 10. Payment Modes

```rust
pub enum PaymentPrivacyMode {
    StandardAccountBilling,
    PseudonymousCredits,
    UnlinkablePrepaidCredits,
    OfflineVoucher,
}
```

---

# 11. Standard Account Billing

Simplest.

Lowest privacy.

---

# 12. Pseudonymous Credits

Funding mapped to a non-public billing pseudonym.

Better separation.

---

# 13. Unlinkable Prepaid Credits

Preferred high-privacy mode.

Concept:

```text
pay once
→ receive blinded/unlinkable spendable resource credits
→ spend credits at services
```

---

# 14. Offline Voucher

Useful for:

```text
gift codes
cash-like distribution
offline bootstrap
```

---

# 15. Do Not Invent Payment Cryptography

Use reviewed:

```text
blind signatures
anonymous credentials
privacy-preserving e-cash
```

if implemented.

---

# 16. Initial Production Recommendation

Start with:

```text
prepaid pseudonymous resource credits
```

then add stronger unlinkable credits after dedicated cryptographic review.

---

# 17. Resource Credit

```rust
pub struct ResourceCredit {
    pub credit_id: ResourceCreditId,
    pub class: ResourceClass,
    pub units: u64,
    pub expires_at: Option<Timestamp>,
    pub proof: CreditProof,
}
```

---

# 18. Credit ID

Provider-facing ID should be random or unlinkable.

---

# 19. Resource Classes

```rust
pub enum ResourceClass {
    MixnetBandwidth,
    MailboxStorage,
    RelayMinutes,
    BulkStorage,
    BulkEgress,
    RealtimeRelay,
    BridgeBandwidth,
}
```

---

# 20. Avoid Universal Currency Internally

Using one universal token everywhere can increase cross-service linkability.

---

# 21. Scoped Credit Classes

Preferred:

```text
mailbox credit
relay credit
bulk storage credit
```

---

# 22. Cross-Service Conversion

Can happen at issuer boundary.

---

# 23. Service-Specific Spend Token

```rust
pub struct ServiceSpendToken {
    pub service: ServiceClass,
    pub amount: u64,
    pub nonce: [u8; 16],
    pub proof: SpendProof,
}
```

---

# 24. Spend Token Scope

Bound to:

```text
service class
amount
expiry
```

---

# 25. One-Time Spend

Preferred.

---

# 26. Double-Spend Problem

If token is one-time, provider must reject reused token.

---

# 27. Double-Spend Store

```rust
pub trait SpentTokenStore {
    fn check_and_mark(
        &self,
        token: SpendTokenId,
    ) -> Result<SpendStatus, AccountingError>;
}
```

---

# 28. Privacy Constraint

Spent-token storage should not contain communication identifiers.

---

# 29. Spend Status

```rust
pub enum SpendStatus {
    Fresh,
    AlreadySpent,
    Invalid,
}
```

---

# 30. Partial Spend

Two options:

```text
fixed denomination tokens
or
change tokens
```

---

# 31. Fixed Denominations

Simpler.

---

# 32. Change Token

More efficient, more complex.

---

# 33. Initial Recommendation

Use fixed/coarse denomination resource vouchers.

---

# 34. Denomination Privacy

Coarse units reduce exact usage fingerprinting.

---

# 35. Example Denominations

Illustrative:

```text
10 MB
100 MB
1 GB
1 relay hour
1 mailbox-month
```

Exact pricing is product policy, not architecture.

---

# 36. Quota Model

Not all resource control requires payment.

Some can be:

```text
free tier
earned credits
operator grant
paid credit
```

---

# 37. Quota Credential

```rust
pub struct QuotaCredential {
    pub scope: QuotaScope,
    pub limit: ResourceLimit,
    pub period: QuotaPeriod,
    pub proof: QuotaProof,
}
```

---

# 38. Quota Scope

```rust
pub enum QuotaScope {
    Service(ServiceClass),
    Capability(CapabilityScopeId),
    Community(CommunityId),
}
```

---

# 39. No Global User Quota ID Required

Hard rule.

---

# 40. Resource Limit

```rust
pub enum ResourceLimit {
    Bytes(u64),
    Objects(u64),
    Duration(Duration),
    Operations(u64),
}
```

---

# 41. Quota Period

```rust
pub enum QuotaPeriod {
    PerSession,
    Daily,
    Monthly,
    Lifetime,
}
```

---

# 42. Privacy-Preserving Metering

Provider needs to know:

```text
how much resource consumed
```

but not:

```text
who user's contacts are
```

---

# 43. Metering Principle

Meter at the service boundary.

---

# 44. Example

Mailbox provider may know:

```text
opaque mailbox capability consumed 50 MB
```

not:

```text
Alice received messages from Bob
```

---

# 45. Meter Event

```rust
pub struct MeterEvent {
    pub service: ServiceClass,
    pub resource: ResourceMeasurement,
    pub accounting_scope: AccountingScopeId,
    pub timestamp_bucket: TimeBucket,
}
```

---

# 46. Accounting Scope ID

Random/scoped.

Not reusable across unrelated services.

---

# 47. Timestamp Bucketing

Avoid unnecessarily precise billing timestamps.

---

# 48. Exact Timestamp

May be required internally for fraud/consistency.

Retention should be minimized.

---

# 49. Aggregation

Prefer:

```text
usage totals
```

over per-packet billing records.

---

# 50. No Per-Message Charge Record

Hard rule where possible.

---

# 51. Batch Accounting

Aggregate:

```text
N MB
per accounting interval
```

---

# 52. Usage Bucket

```rust
pub struct UsageBucket {
    pub scope: AccountingScopeId,
    pub period: AccountingPeriod,
    pub usage: ResourceMeasurement,
}
```

---

# 53. Accounting Period

Could be:

```text
hour
day
billing cycle
```

---

# 54. Service Receipt

```rust
pub struct ServiceReceipt {
    pub receipt_id: ReceiptId,
    pub service: ServiceClass,
    pub consumed: ResourceMeasurement,
    pub credit_spent: u64,
    pub period: AccountingPeriod,
    pub proof: ReceiptProof,
}
```

---

# 55. Receipt Privacy

Receipt should not contain:

```text
contact
conversation
peer IP
group
```

---

# 56. Receipt Purpose

Useful for:

```text
local accounting
refund
dispute
operator settlement
```

---

# 57. Receipt Authenticity

Provider signs receipt.

---

# 58. User Wallet

Local-first wallet stores:

```text
credits
spent state
receipts
refund state
```

---

# 59. Wallet Identity

Separate from messaging identity.

---

# 60. Wallet Storage

Encrypted.

---

# 61. Wallet Backup

Needs separate recovery policy.

---

# 62. Credit Backup

Some anonymous credit systems may not safely support naïve duplication.

---

# 63. Double-Spend Risk on Restore

Restoring same spendable token to two devices can create double-spend.

---

# 64. Therefore

Credit recovery architecture must define:

```text
single active wallet
or
issuer-side recovery/reissue
```

---

# 65. Wallet Recovery Policy

```rust
pub enum WalletRecoveryPolicy {
    SingleDeviceAuthority,
    IssuerReissue,
    NonRecoverableBearerCredits,
}
```

---

# 66. Bearer Credits

Like cash:

```text
loss may be irreversible
```

---

# 67. UX Honesty

If bearer token cannot be recovered, say so clearly.

---

# 68. Multi-Device Wallet

Avoid simply copying bearer tokens to every device.

---

# 69. Better

Allocate separate credit subsets per device.

---

# 70. Device Credit Pool

```rust
pub struct DeviceCreditPool {
    pub device_wallet_id: DeviceWalletId,
    pub credits: Vec<ResourceCreditRef>,
}
```

---

# 71. Transfer Between Devices

Use wallet transfer protocol.

---

# 72. Offline Spend

Possible for certain vouchers.

---

# 73. Offline Double-Spend

Hard problem.

---

# 74. Initial Recommendation

Require online validation for reusable/high-value credit spend.

---

# 75. Low-Value Offline Voucher

Could be accepted with bounded risk.

---

# 76. Free Credits

Issuer may grant:

```text
signup
community subsidy
operator promotion
```

---

# 77. Subsidy Privacy

Should not turn into identity marker.

---

# 78. Anonymous Voucher Distribution

Useful for:

```text
aid
community access
emergency communication
```

---

# 79. Voucher Code

High entropy.

---

# 80. Voucher Redemption

Issuer returns resource credits.

---

# 81. Voucher Abuse

Rate-limit redemption.

---

# 82. Pricing Model

Architecture supports:

```text
fixed price
dynamic price
operator-defined price
subscription package
```

---

# 83. Dynamic Pricing Privacy

Frequent price queries can reveal activity.

---

# 84. Signed Price Catalog

Clients can fetch generic price table.

---

# 85. Price Catalog

```rust
pub struct PriceCatalog {
    pub version: u64,
    pub prices: Vec<ServicePrice>,
    pub valid_until: Timestamp,
    pub signature: PriceSignature,
}
```

---

# 86. No Per-User Pricing in Strict Mode

Hard rule.

---

# 87. Operator-Specific Pricing

Possible if topology/provider selection supports it.

---

# 88. Service Price

```rust
pub struct ServicePrice {
    pub service: ServiceClass,
    pub unit: ResourceUnit,
    pub cost: u64,
}
```

---

# 89. Price Unit

```rust
pub enum ResourceUnit {
    Megabyte,
    Gigabyte,
    Minute,
    Object,
    Month,
    Operation,
}
```

---

# 90. Subscription

Could grant periodic scoped credits.

---

# 91. Subscription Credential

Do not expose billing account directly to provider.

---

# 92. Subscription Issuer

Issuer converts subscription entitlement to:

```text
service credits
```

---

# 93. Wallet Balance

Local.

---

# 94. Provider Should Not Query User's Full Balance

Hard rule.

---

# 95. Spend Authorization

Present only enough credit for requested service.

---

# 96. Credit Reservation

Needed for long operations.

Example:

```text
2-hour call
```

---

# 97. Reservation

```rust
pub struct CreditReservation {
    pub reservation_id: ReservationId,
    pub service: ServiceClass,
    pub max_amount: u64,
    pub expires_at: Timestamp,
}
```

---

# 98. Reservation Flow

```text
reserve
→ consume
→ finalize
→ refund unused
```

---

# 99. Realtime Relay Accounting

Meter:

```text
relay minutes
bandwidth
```

---

# 100. Call Privacy

Relay accounting scope must not expose peer identities.

---

# 101. Mixnet Accounting

Meter:

```text
bandwidth/cells
```

---

# 102. Critical Warning

Per-packet payment tokens can make packets linkable.

---

# 103. Better

Use:

```text
short-lived quota credential
```

for many packets.

---

# 104. Mixnet Session Credit

```rust
pub struct MixnetQuotaCredential {
    pub quota_id: [u8; 32],
    pub cell_budget: u64,
    pub expires_at: Timestamp,
    pub proof: QuotaProof,
}
```

---

# 105. Quota ID Rotation

Frequent.

---

# 106. Provider Visibility

Entry gateway may see quota credential.

Downstream mix nodes should not need it.

---

# 107. No Payment Marker Inside Sphinx Route

Hard rule.

---

# 108. Mailbox Accounting

Meter:

```text
storage bytes
retention
fetch egress
```

---

# 109. Mailbox Identity Separation

Billing scope separate from mailbox ID if practical.

---

# 110. Bulk Storage Accounting

Meter:

```text
stored bytes × retention
egress
operations
```

---

# 111. Bulk Provider

Uses anonymous object billing scope.

---

# 112. Object Billing Linkability

Do not reuse one stable billing ID for all objects.

---

# 113. Object-Level Credit

Possible for high privacy.

---

# 114. Bridge Accounting

May meter:

```text
bandwidth
session duration
```

---

# 115. Bridge Free Tier

Likely useful to avoid access failure under censorship.

---

# 116. Emergency Access

Architecture may allow:

```text
emergency zero-cost quota
```

with abuse controls.

---

# 117. Emergency Abuse

Must be bounded.

---

# 118. Resource Credit Transfer

Users may transfer credits.

---

# 119. Privacy Risk

Transfers can create social graph.

---

# 120. Initial Recommendation

Do not support user-to-user credit transfer in first release.

---

# 121. Gift Voucher

Issuer-mediated gift code safer.

---

# 122. Refund

Possible when service fails before consumption.

---

# 123. Refund Token

```rust
pub struct RefundClaim {
    pub receipt_id: ReceiptId,
    pub unused: u64,
    pub provider_proof: ReceiptProof,
}
```

---

# 124. Refund Privacy

Issuer should not need communication details.

---

# 125. Refund Abuse

Prevent double refund.

---

# 126. Refund State

```rust
pub enum RefundState {
    Eligible,
    Claimed,
    Rejected,
    Expired,
}
```

---

# 127. Service Failure

Unused reservation returned.

---

# 128. Provider Crash

Receipt/reconciliation mechanism required.

---

# 129. Idempotent Settlement

Hard rule.

---

# 130. Accounting Ledger

Use append-only accounting ledger.

---

# 131. Ledger Scope

Financial/resource events only.

No communication metadata.

---

# 132. Ledger Entry

```rust
pub struct AccountingEntry {
    pub entry_id: AccountingEntryId,
    pub service: ServiceClass,
    pub accounting_scope: AccountingScopeId,
    pub delta: ResourceDelta,
    pub reason: AccountingReason,
    pub timestamp_bucket: TimeBucket,
}
```

---

# 133. Accounting Reason

```rust
pub enum AccountingReason {
    CreditIssued,
    CreditReserved,
    CreditConsumed,
    CreditRefunded,
    CreditExpired,
}
```

---

# 134. Double-Entry Accounting

Useful internally.

---

# 135. Privacy Boundary

Double-entry does not imply identity linkage.

---

# 136. Issuer Ledger

Tracks:

```text
credits issued
credits redeemed
```

---

# 137. Provider Ledger

Tracks:

```text
resource consumed
receipts
settlement
```

---

# 138. Settlement Aggregator

Pays operator aggregate amounts.

---

# 139. Operator Settlement Record

```rust
pub struct OperatorSettlement {
    pub operator: OperatorId,
    pub period: AccountingPeriod,
    pub service: ServiceClass,
    pub units: u64,
    pub amount: SettlementAmount,
}
```

---

# 140. No User Mapping

Hard rule.

---

# 141. Fraud Detection

Possible threats:

```text
double spend
forged credit
replayed token
meter under-reporting
provider over-reporting
refund fraud
```

---

# 142. Credit Authenticity

Issuer signs/proves credit.

---

# 143. Provider Receipt

Provider signs consumption.

---

# 144. Cross-Checking

Issuer can reconcile:

```text
spent credits
provider receipts
```

without communication identity.

---

# 145. Provider Over-Billing

User can verify signed usage receipt against local meter where possible.

---

# 146. Local Meter

Client keeps expected usage.

---

# 147. Tolerance

Network overhead may differ.

---

# 148. Dispute

User submits:

```text
receipt
local measurement
```

not conversation content.

---

# 149. Abuse Rate Limiting

Free anonymous services need quota.

---

# 150. Free Quota Token

```rust
pub struct FreeTierCredential {
    pub service: ServiceClass,
    pub allowance: u64,
    pub period: QuotaPeriod,
    pub proof: QuotaProof,
}
```

---

# 151. Sybil Problem

Anonymous users can repeatedly claim free quota.

---

# 152. Possible Defenses

```text
device-bound coarse grant
proof-of-work
invite credential
operator-issued voucher
payment-backed grant
```

---

# 153. Privacy vs Sybil Resistance

No perfect solution.

---

# 154. Device Binding

Can hurt privacy.

Use only if policy explicitly accepts it.

---

# 155. Proof-of-Work

Consumes battery.

Use carefully.

---

# 156. Human Verification

Could be optional.

---

# 157. Invite-Based Free Quota

Useful for communities.

---

# 158. Free Tier Policy

Separate from paid anonymous credits.

---

# 159. Resource Quota Service

```rust
pub trait ResourceQuotaService {
    fn authorize(
        &self,
        credential: &QuotaCredential,
        request: ResourceRequest,
    ) -> Result<QuotaAuthorization, QuotaError>;

    fn consume(
        &self,
        authorization: QuotaAuthorization,
        actual: ResourceMeasurement,
    ) -> Result<ServiceReceipt, QuotaError>;
}
```

---

# 160. Credit Wallet Trait

```rust
pub trait AnonymousCreditWallet {
    fn balance(
        &self,
        class: ResourceClass,
    ) -> Result<ResourceBalance, WalletError>;

    fn prepare_spend(
        &self,
        request: SpendRequest,
    ) -> Result<ServiceSpendToken, WalletError>;

    fn apply_receipt(
        &self,
        receipt: ServiceReceipt,
    ) -> Result<(), WalletError>;
}
```

---

# 161. Credit Issuer Trait

```rust
#[async_trait]
pub trait CreditIssuer: Send + Sync {
    async fn issue(
        &self,
        funding: FundingReceipt,
        request: CreditIssueRequest,
    ) -> Result<Vec<ResourceCredit>, CreditError>;

    async fn redeem_refund(
        &self,
        claim: RefundClaim,
    ) -> Result<ResourceCredit, CreditError>;
}
```

---

# 162. Funding Adapter

Payment processors remain outside anonymous service plane.

---

# 163. Funding Methods

Architecture can support:

```text
card
bank transfer
wallet
gift voucher
organization grant
```

---

# 164. Privacy Caveat

Traditional payment processor may know payer identity.

---

# 165. Separation Goal

Processor should not learn:

```text
which contacts
which mailbox
which group
which relay call
```

---

# 166. Funding Receipt

```rust
pub struct FundingReceipt {
    pub issuer_reference: OpaqueFundingReference,
    pub amount: FundingAmount,
    pub status: FundingStatus,
}
```

---

# 167. Opaque Funding Reference

Never propagated to service provider.

---

# 168. KYC Boundary

If legally required for a payment provider, keep KYC at funding boundary.

---

# 169. No KYC Propagation

Hard rule unless law/product specifically requires a service identity.

---

# 170. Regional Compliance

Payment laws differ.

Architecture must support:

```text
provider-specific funding adapters
```

without altering anonymous-service credentials.

---

# 171. Tax/Invoice Boundary

Invoice may identify payer.

Keep separate from anonymous usage receipts.

---

# 172. Business Receipt

Separate artifact.

---

# 173. Service Usage Receipt

Pseudonymous/scoped.

---

# 174. Account Billing Mode

If user chooses convenience:

```text
subscription account
```

can still convert entitlement into service-scoped credits.

---

# 175. Wallet Top-Up

Flow:

```text
fund
→ issuer confirms
→ wallet receives credits
```

---

# 176. Auto Top-Up

Privacy concern.

Repeated automated funding correlates usage.

---

# 177. Strict Privacy Recommendation

Prefer manual/prepaid larger top-ups.

---

# 178. Auto Top-Up Option

Explicit convenience mode.

---

# 179. Low Balance Notification

Local.

---

# 180. Provider Should Not Send Identity-Linked Billing Notification

Hard rule.

---

# 181. Expiry

Credits may expire for business reasons.

---

# 182. Expiry Privacy

Do not force spend pattern before exact short deadline unnecessarily.

---

# 183. Credit Consolidation

Combining credits can increase linkability.

---

# 184. Avoid Provider-Side Consolidation

Wallet can select tokens locally.

---

# 185. Coin Selection Analogy

Choose credits to minimize:

```text
linkability
waste
```

---

# 186. Wallet Selection Policy

```rust
pub enum CreditSelectionPolicy {
    MinimizeChange,
    MinimizeLinkability,
    OldestFirst,
}
```

---

# 187. Strict Mode

Prefer:

```text
MinimizeLinkability
```

---

# 188. Receipt Linkability

One stable wallet receipt ID across services is bad.

---

# 189. Per-Service Receipt Namespace

Required.

---

# 190. Accounting Correlation Attack

Adversary combines:

```text
funding timestamp
credit issuance
service spend
traffic timing
```

---

# 191. Mitigations

```text
prepay
batch issuance
coarse denominations
delayed use
service-specific tokens
```

---

# 192. Timing Separation

Do not require immediate:

```text
fund → spend
```

---

# 193. Issuance Batching

Useful for unlinkability.

---

# 194. Blind Issuance

Future strong solution.

---

# 195. Privacy Lab Integration

Part 42 should test:

```text
funding-to-usage correlation
cross-service credit correlation
quota ID lifetime
receipt linkage
```

---

# 196. Maximum Privacy

Recommended:

```text
prepaid
service-scoped
short-lived quota credentials
local wallet
no identity-linked provider account
```

---

# 197. Service Denial

If no credit:

```text
service unavailable
```

not privacy downgrade.

---

# 198. Example

No private relay credits:

```text
private call cannot start
```

not:

```text
use direct call silently
```

---

# 199. Emergency Free Path

If product provides, must be explicit and bounded.

---

# 200. Wallet UX

Show:

```text
Mixnet bandwidth
Mailbox storage
Relay time
Bulk storage
```

rather than one opaque score if credits are scoped.

---

# 201. Simplified UX

Can show converted estimated balance locally.

---

# 202. No Provider-Side Unified User Wallet Required

Hard rule.

---

# 203. Spend Confirmation

For large spend:

```text
confirm
```

---

# 204. Small Spend

Automatic under local budget.

---

# 205. Budget Policy

```rust
pub struct WalletBudgetPolicy {
    pub auto_spend_limit: u64,
    pub daily_limit: Option<u64>,
}
```

---

# 206. Child/Managed Profiles

Separate product policy.

---

# 207. Organization Grants

Organization can issue service credits.

---

# 208. Grant Privacy

Organization may know grant recipient.

Provider still should not learn user's communications.

---

# 209. Community Credits

Can subsidize group/community service.

---

# 210. Group Quota

Group capability may carry pooled credit.

---

# 211. Privacy Risk

Pooled credit links group usage.

---

# 212. Strict Group Mode

Prefer member-specific anonymous service credits.

---

# 213. Infrastructure Operator Payments

Operators need compensation.

---

# 214. Settlement Input

Aggregate:

```text
valid receipts
resource units
service class
```

---

# 215. Operator Privacy

Operator identity can be public/governance identity.

---

# 216. User Privacy

Operator does not need user identity.

---

# 217. Settlement Fraud

Operator may fabricate usage.

---

# 218. Countermeasures

```text
signed spend tokens
client receipts
issuer reconciliation
capacity audits
```

---

# 219. No Packet-Level Settlement Logging

Hard rule.

---

# 220. Accounting Retention

Different layers:

```text
wallet receipts
issuer spend proofs
provider aggregate ledger
settlement records
```

---

# 221. Retention Minimum

Only as long as:

```text
fraud
refund
accounting
legal
```

requires.

---

# 222. Deletion

Service-specific identifiers should expire.

---

# 223. Audit

Accounting audit must not expose communication graph.

---

# 224. Finance Audit Export

Contains:

```text
amounts
service categories
settlements
```

not message metadata.

---

# 225. Observability

Safe metrics:

```text
credits issued
credits redeemed
quota failures
refund count
settlement totals
```

---

# 226. Forbidden Metrics

No:

```text
AccountId + mailbox ID
AccountId + relay session
contact + spend
group + payer identity
```

---

# 227. Remote Telemetry

Do not log token serials into general telemetry.

---

# 228. Support Bundle

Redact:

```text
unspent tokens
spend proofs
funding references
```

---

# 229. Security Threats

```text
token theft
double spend
forgery
replay
rollback
wallet cloning
receipt forgery
provider overbilling
issuer compromise
```

---

# 230. Token Theft

Bearer credit stolen = spend risk.

---

# 231. Wallet Protection

Use:

```text
OS secure store
encrypted DB
device authentication
```

---

# 232. Wallet Cloning

Detect/restrict via issuer rules if recoverable model.

---

# 233. Backup Rollback

Restoring old wallet state can replay spent credits.

---

# 234. Anti-Rollback

Spent-state reconciliation required.

---

# 235. Non-Recoverable Cash Mode

Alternative avoids issuer account but increases loss risk.

---

# 236. Issuer Compromise

Can mint fraudulent credits.

---

# 237. Issuer Key Rotation

Required.

---

# 238. Offline Root

Recommended for credit-signing hierarchy.

---

# 239. Provider Verification Keys

Distributed via signed catalog.

---

# 240. Credit Versioning

```rust
pub struct CreditVersion(pub u16);
```

---

# 241. Unknown Version

Reject safely.

---

# 242. Price Catalog Anti-Rollback

Required.

---

# 243. Wallet Schema

Potential:

```text
resource_credits
spent_tokens
service_receipts
credit_reservations
refund_claims
wallet_policy
```

---

# 244. Provider Accounting Schema

Potential:

```text
quota_credentials
spent_token_ids
usage_buckets
service_receipts
settlement_batches
```

---

# 245. Issuer Schema

Potential:

```text
funding_receipts
credit_issuance
redeemed_spends
refunds
issuer_key_epochs
```

---

# 246. Secret Storage

Unspent credit secrets stored securely.

---

# 247. Transactionality

Spend flow:

```text
reserve token
→ send
→ provider confirms
→ mark spent
→ persist receipt
```

must be crash-safe.

---

# 248. Crash Ambiguity

If crash after provider accepts but before wallet records receipt:

```text
token considered potentially spent
```

---

# 249. Spend Reconciliation

Query using privacy-safe receipt/spend proof.

---

# 250. Never Retry Same One-Time Token Blindly

Hard rule.

---

# 251. State Machine

```rust
pub enum SpendState {
    Available,
    Reserved,
    Submitted,
    Confirmed,
    Uncertain,
    Refunded,
}
```

---

# 252. Uncertain State

Conservative.

---

# 253. Reconciliation

Resolve:

```text
Confirmed
or
Refundable/Available
```

only with provider/issuer proof.

---

# 254. Formal Verification Targets

Strong candidates:

```text
spend state machine
double-spend prevention
reservation/refund
wallet restore
settlement idempotency
```

---

# 255. TLA+ Candidate

Credit reservation / consume / refund workflow.

---

# 256. Kani Candidate

Token scope and denomination validation.

---

# 257. Loom Candidate

Concurrent spend attempts.

---

# 258. Fuzzing

Fuzz:

```text
credit token
quota credential
receipt
price catalog
refund claim
```

---

# 259. Property Tests

Properties:

```text
one-time token cannot be successfully spent twice
provider cannot consume more than authorized quota
refund plus spend cannot exceed original credit
strict service never uses identity-linked fallback
```

---

# 260. Double-Spend Race Test

Two providers simultaneously receive same token.

---

# 261. Crash Tests

Crash at every spend transition.

---

# 262. Wallet Restore Tests

Old backup cannot create valid duplicate spending.

---

# 263. Price Tests

Old catalog rejected after newer accepted.

---

# 264. Receipt Forgery Test

Invalid provider signature rejected.

---

# 265. Provider Overbilling Test

Receipt over reservation rejected.

---

# 266. Quota Exhaustion Test

Service stops safely.

---

# 267. Privacy Tests

Verify service provider cannot infer:

```text
funding account
contact identity
group identity
```

from credential schema.

---

# 268. Cross-Service Correlation Test

Tokens for:

```text
mailbox
relay
bulk storage
```

must not share stable IDs.

---

# 269. Performance Tests

Measure:

```text
token verification
quota authorization
wallet selection
settlement aggregation
```

---

# 270. Scale Tests

High-rate:

```text
millions of spend tokens
large spent-token store
settlement batches
```

---

# 271. Privacy-Safe Spent Set

If using probabilistic structure, false-positive impact must be analyzed.

---

# 272. Exact Store Initially

Recommended.

---

# 273. Pruning

Spent token records can expire after token lifetime + dispute window.

---

# 274. Crate Layout

Recommended:

```text
crates/
├── siar-accounting-core/
├── siar-credit-wallet/
├── siar-credit-issuer/
├── siar-credit-token/
├── siar-quota/
├── siar-metering/
├── siar-receipts/
├── siar-refunds/
├── siar-price-catalog/
├── siar-settlement/
├── siar-accounting-observability/
└── siar-accounting-testkit/
```

---

# 275. `siar-accounting-core`

Owns:

```text
service classes
resource units
accounting IDs
errors
```

---

# 276. `siar-credit-wallet`

Local encrypted wallet.

---

# 277. `siar-credit-issuer`

Funding-to-credit issuance.

---

# 278. `siar-credit-token`

Spend token formats/proofs.

---

# 279. `siar-quota`

Quota credentials/authorization.

---

# 280. `siar-metering`

Usage aggregation.

---

# 281. `siar-receipts`

Signed service receipts.

---

# 282. `siar-refunds`

Refund claims/state.

---

# 283. `siar-price-catalog`

Signed pricing.

---

# 284. `siar-settlement`

Operator aggregation/payout.

---

# 285. `siar-accounting-observability`

Privacy-safe metrics.

---

# 286. `siar-accounting-testkit`

Fake issuer/provider/fraud tests.

---

# 287. Error Taxonomy

```rust
pub enum AccountingError {
    CreditInvalid,
    CreditExpired,
    CreditInsufficient,
    DoubleSpend,
    QuotaExceeded,
    ReservationExpired,
    ReceiptInvalid,
    RefundAlreadyClaimed,
    PriceCatalogStale,
    SettlementConflict,
    WalletLocked,
    Internal,
}
```

---

# 288. Security Invariants

Mandatory:

```text
1. Funding identity never becomes a universal communication identifier.
2. Service credentials are scoped and do not reuse AccountId/DeviceId/ContactId.
3. Mixnet packet routing never embeds payment identity.
4. Cross-service credits do not expose one stable provider-visible wallet ID.
5. One-time spend tokens cannot be accepted twice.
6. Receipt and refund processing is idempotent.
7. Strict privacy services never downgrade when credits are unavailable.
8. Provider receipts contain no contact/conversation/group identity.
9. Wallet backups cannot safely duplicate spendable bearer credit state.
10. Operator settlement is aggregate and does not require user identity.
11. Unspent bearer secrets never appear in telemetry/support bundles.
12. Payment cryptography uses reviewed constructions rather than custom schemes.
```

---

# 289. Initial Production Scope

Implement first:

```text
local encrypted resource wallet
service-scoped prepaid credits
fixed denominations
short-lived quota credentials
signed price catalog
service receipts
reservation/finalize/refund
exact double-spend store
aggregate operator settlement
strict separation from communication IDs
privacy-safe diagnostics
```

Then add:

```text
blind/unlinkable credit issuance
anonymous e-cash style spending
offline vouchers
privacy-preserving free-tier credentials
threshold issuer signing
advanced fraud proofs
```

---

# 290. Definition of Done

Part 47 is complete when:

- funding identity and communication identity are cleanly separated
- resource/service credit types are scoped
- no universal provider-visible wallet identity is required
- quotas can be enforced without social graph access
- mixnet/mailbox/relay/bulk services have explicit accounting boundaries
- one-time token and double-spend handling are defined
- reservations, refunds, receipts, and crash ambiguity are handled
- wallet backup/recovery does not clone bearer credits unsafely
- operator settlement is aggregate and privacy-preserving
- signed pricing and anti-rollback are defined
- free-tier/Sybil tradeoffs are explicit
- telemetry/support bundles exclude payment secrets and cross-service linkable IDs
- property/formal/fraud/privacy tests are specified
- stronger blind/unlinkable credits remain a reviewed future layer rather than ad hoc cryptography

---

# 291. Final Architecture

```text
                      FUNDING SOURCE
                           │
                           ▼
                     CREDIT ISSUER
                           │
                     Scoped Credits
                           │
            ┌──────────────┼──────────────┐
            │              │              │
         Mixnet         Mailbox        Relay/Bulk
            │              │              │
            ▼              ▼              ▼
       Quota/Meter     Quota/Meter     Quota/Meter
            │              │              │
            └──────────────┼──────────────┘
                           ▼
                      Signed Receipts
                           │
                           ▼
                 Aggregate Settlement
```

Strict privacy model:

```text
funding identity
    ↓
issuer boundary
    ↓
scoped/unlinkable credit
    ↓
service quota credential
    ↓
anonymous service usage
```

not:

```text
billing account
→ permanent ID
→ every relay/mailbox/file/call
```

---

# 292. Final Principle

Private communication infrastructure can be paid for without turning billing into a social-graph database.

The correct model is:

```text
prepaid/scoped credits
+
service-local quota credentials
+
minimal metering
+
signed receipts
+
aggregate settlement
+
strict identity separation
```

not:

```text
one account ID attached to every resource event
```

This architecture gives SIAR a path to sustainable paid infrastructure while preserving the anonymity and metadata-separation guarantees established in Parts 34–46.
