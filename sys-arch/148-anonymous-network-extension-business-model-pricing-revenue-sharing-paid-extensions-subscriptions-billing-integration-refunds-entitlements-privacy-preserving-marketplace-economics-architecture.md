# Core System Architecture Part 148 — Anonymous Network Extension Business Model, Pricing, Revenue Sharing, Paid Extensions, Subscriptions, Billing Integration, Refunds, Entitlements & Privacy-Preserving Marketplace Economics Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 148  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 47, 54, 102, 133, 147 and related Parts 68, 83, 94, 126, 135–146

**Primary purpose:** define SIAR's privacy-preserving marketplace economics architecture for extension pricing, one-time purchases, subscriptions, trials, bundles, revenue sharing, platform fees, entitlement issuance, billing integration, refunds, chargebacks, grace periods, enterprise licensing, payout settlement, offline entitlement verification, and economic isolation from user identity, messaging identity, authorization, telemetry, and marketplace governance.

---

# 1. Purpose

A mature extension ecosystem may need sustainable commercial models.

Extensions may be:

```text
free
one-time paid
subscription-based
enterprise licensed
bundled
metered in narrow cases
```

But marketplace monetization creates architectural risks:

```text
billing identity linked to messaging identity
entitlement checks tied to tracking
publisher access to user purchase history
pay-to-rank marketplace pressure
billing failure becoming data-loss
refund abuse causing inconsistent authority
platform fees hidden from publishers/users
```

The governing principle is:

> **SIAR marketplace economics must support sustainable paid extensions without making payment identity, account behavior, publisher economics, or purchase history part of the anonymous communication trust model.**

---

# 2. Architectural Position

```text
                 EXTENSION OFFER
                        │
                        ▼
                 COMMERCE CATALOG
                        │
             ┌──────────┼──────────┐
             │          │          │
         PRICING      BILLING   ENTITLEMENT
             │          │          │
             └──────────┼──────────┘
                        ▼
                PURCHASE / RENEWAL
                        │
                        ▼
                 ENTITLEMENT LEDGER
                        │
             ┌──────────┼──────────┐
             │          │          │
          DEVICE      TENANT     USER
             │          │          │
             └──────────┼──────────┘
                        ▼
                 EXTENSION ACCESS
```

---

# 3. Core Separation

Keep distinct:

```text
commercial offer
price
payment transaction
billing customer identity
entitlement
extension installation
runtime authorization
publisher payout
marketplace governance
```

---

# 4. Non-Goals

Part 148 does not create:

```text
billing identity as messaging identity
publisher access to card/payment credentials
purchase-history-based ranking
pay-to-bypass security review
global behavioral monetization
```

---

# 5. Commercial Offer Identity

```rust
pub struct ExtensionOfferId(pub [u8; 16]);
```

---

# 6. Product Identity

```rust
pub struct ExtensionCommercialProductId(pub [u8; 16]);
```

---

# 7. Offer Type

```rust
pub enum ExtensionOfferType {
    Free,
    OneTimePurchase,
    Subscription,
    Trial,
    EnterpriseLicense,
    Bundle,
}
```

---

# 8. Hard Rule

Offer type does not change extension security/privacy requirements.

---

# 9. Price Identity

```rust
pub struct ExtensionPriceId(pub [u8; 16]);
```

---

# 10. Money Type

Use fixed-point decimal/minor units.

```rust
pub struct Money {
    pub currency: CurrencyCode,
    pub minor_units: i64,
}
```

---

# 11. Hard Rule

No binary floating point for monetary values.

---

# 12. Currency

```rust
pub struct CurrencyCode(pub [u8; 3]);
```

---

# 13. Hard Rule

Currency explicit for every price/settlement.

---

# 14. Price Definition

```rust
pub struct ExtensionPrice {
    pub price_id: ExtensionPriceId,
    pub amount: Money,
    pub billing_period: Option<BillingPeriod>,
    pub tax_mode: TaxMode,
}
```

---

# 15. Billing Period

```rust
pub enum BillingPeriod {
    Monthly,
    Quarterly,
    SemiAnnual,
    Annual,
}
```

---

# 16. Hard Rule

Billing period is part of the offer, not inferred from amount.

---

# 17. Price Versioning

Prices are versioned.

---

# 18. Hard Rule

Existing subscription terms do not silently change because publisher updates catalog price.

---

# 19. Grandfathering

Explicit policy.

---

# 20. Hard Rule

Grandfathered prices remain tied to original commercial terms until changed under allowed process.

---

# 21. Trial

```rust
pub struct ExtensionTrialPolicy {
    pub duration: Duration,
    pub requires_payment_method: bool,
    pub auto_converts: bool,
}
```

---

# 22. Hard Rule

Auto-conversion disclosed explicitly.

---

# 23. No Dark Billing

Hard rule.

---

# 24. Free Trial Reminder

Optional platform-controlled reminder before conversion.

---

# 25. Hard Rule

Publisher cannot hide cancellation path.

---

# 26. One-Time Purchase

Creates durable entitlement subject to product lifecycle policy.

---

# 27. Hard Rule

One-time purchase is not equivalent to perpetual platform compatibility guarantee.

---

# 28. Subscription

Represents recurring commercial access.

---

# 29. Subscription Identity

```rust
pub struct ExtensionSubscriptionId(pub [u8; 16]);
```

---

# 30. Subscription State

```rust
pub enum ExtensionSubscriptionState {
    Trialing,
    Active,
    PastDue,
    GracePeriod,
    Paused,
    CancelAtPeriodEnd,
    Cancelled,
    Expired,
    Refunded,
}
```

---

# 31. Hard Rule

Billing state and entitlement state are distinct.

---

# 32. Entitlement Identity

```rust
pub struct ExtensionEntitlementId(pub [u8; 16]);
```

---

# 33. Entitlement Scope

```rust
pub enum ExtensionEntitlementScope {
    UserAccount,
    Device,
    Tenant,
    Organization,
}
```

---

# 34. Hard Rule

Scope explicit.

---

# 35. Entitlement State

```rust
pub enum ExtensionEntitlementState {
    Active,
    Grace,
    Suspended,
    Expired,
    Revoked,
}
```

---

# 36. Hard Rule

Revoked entitlement does not imply extension package revocation.

---

# 37. Entitlement Grant

```rust
pub struct ExtensionEntitlement {
    pub entitlement_id: ExtensionEntitlementId,
    pub product: ExtensionCommercialProductId,
    pub scope: ExtensionEntitlementScope,
    pub state: ExtensionEntitlementState,
    pub valid_until: Option<Timestamp>,
}
```

---

# 38. Hard Rule

Entitlement is commercial authorization only.

---

# 39. Runtime Security Separation

An entitlement never grants:

```text
message content access
file access
network permission
camera/microphone
admin capability
```

---

# 40. Hard Rule

Commercial payment can never bypass Part 135 permissions.

---

# 41. Installation Separation

Paid entitlement may allow install/activation.

---

# 42. Hard Rule

Install eligibility still requires package verification/certification/policy.

---

# 43. Feature Entitlements

Extension may define paid tiers.

```rust
pub struct ExtensionFeatureEntitlement {
    pub feature: ExtensionFeatureId,
    pub required_product: ExtensionCommercialProductId,
}
```

---

# 44. Hard Rule

Paid feature flags remain separate from security capability flags.

---

# 45. No Pay-to-Privilege

Hard rule.

---

# 46. Commerce Catalog

```rust
pub struct ExtensionCommerceCatalog {
    pub extension: ExtensionId,
    pub products: Vec<ExtensionCommercialProduct>,
}
```

---

# 47. Commercial Product

```rust
pub struct ExtensionCommercialProduct {
    pub product_id: ExtensionCommercialProductId,
    pub offer_type: ExtensionOfferType,
    pub prices: Vec<ExtensionPrice>,
}
```

---

# 48. Hard Rule

Catalog data signed/versioned.

---

# 49. Catalog Epoch

```rust
pub struct ExtensionCommerceCatalogEpoch(pub u64);
```

---

# 50. Hard Rule

Anti-rollback.

---

# 51. Publisher Pricing Authority

Publisher proposes pricing.

Marketplace validates format/policy.

---

# 52. Hard Rule

Marketplace cannot silently alter publisher net terms without versioned contract.

---

# 53. Platform Fee

```rust
pub struct PlatformFeePolicy {
    pub percentage_bps: u32,
    pub fixed_fee: Option<Money>,
}
```

---

# 54. Hard Rule

Fee policy explicit.

---

# 55. Revenue Share

```rust
pub struct ExtensionRevenueShare {
    pub gross: Money,
    pub platform_fee: Money,
    pub taxes_withheld: Money,
    pub refunds: Money,
    pub publisher_net: Money,
}
```

---

# 56. Hard Rule

Settlement arithmetic reproducible.

---

# 57. No Hidden Fee Bucket

Hard rule.

---

# 58. Revenue Share Versioning

Publisher contract version pinned per transaction.

---

# 59. Hard Rule

Historical transaction economics immutable.

---

# 60. Settlement Period

```rust
pub enum SettlementPeriod {
    Weekly,
    Monthly,
}
```

---

# 61. Hard Rule

Settlement period independent from subscription period.

---

# 62. Publisher Balance

```rust
pub struct PublisherSettlementBalance {
    pub pending: Money,
    pub available: Money,
    pub reserved: Money,
}
```

---

# 63. Hard Rule

Balance state separated from entitlement/user state.

---

# 64. Reserve

May exist for refunds/chargebacks.

---

# 65. Hard Rule

Reserve policy explicit in publisher agreement.

---

# 66. Marketplace Economic Role

Prefer:

```text
merchant of record
or
marketplace facilitator
```

depending deployment/legal model.

---

# 67. Hard Rule

Role explicitly modeled; not ambiguous in accounting.

---

# 68. Billing Provider

External PSP integration isolated.

---

# 69. Billing Provider Identity

```rust
pub struct BillingProviderId(pub [u8; 16]);
```

---

# 70. Hard Rule

Publisher never receives raw payment credentials.

---

# 71. Tokenized Payment Method

Stored by billing provider.

---

# 72. Hard Rule

SIAR stores provider references only where necessary.

---

# 73. Billing Customer Reference

```rust
pub struct BillingCustomerRef {
    pub provider: BillingProviderId,
    pub opaque_customer_id: String,
}
```

---

# 74. Hard Rule

Billing customer reference never used as messaging/contact identity.

---

# 75. Purchase Identity

```rust
pub struct ExtensionPurchaseId(pub [u8; 16]);
```

---

# 76. Purchase State

```rust
pub enum ExtensionPurchaseState {
    Created,
    PendingPayment,
    Paid,
    Failed,
    Cancelled,
    Refunded,
    ChargedBack,
}
```

---

# 77. Hard Rule

Paid only after provider-confirmed settlement/authorization semantics.

---

# 78. Idempotency

Every billing mutation uses idempotency key.

---

# 79. Hard Rule

Retry cannot double-charge.

---

# 80. Billing Idempotency Key

```rust
pub struct BillingIdempotencyKey(pub [u8; 16]);
```

---

# 81. Webhook Processing

Billing provider events arrive through authenticated webhook.

---

# 82. Hard Rule

Webhook signature verified.

---

# 83. Event Replay

Deduplicate by provider event ID.

---

# 84. Hard Rule

Webhook replay cannot create duplicate entitlement.

---

# 85. Billing Event

```rust
pub struct BillingEvent {
    pub provider_event_id: String,
    pub event_type: BillingEventType,
    pub purchase: Option<ExtensionPurchaseId>,
    pub subscription: Option<ExtensionSubscriptionId>,
}
```

---

# 86. Hard Rule

Provider event payload is normalized into typed internal event.

---

# 87. Billing Event Types

```rust
pub enum BillingEventType {
    PaymentSucceeded,
    PaymentFailed,
    SubscriptionRenewed,
    SubscriptionCancelled,
    RefundIssued,
    ChargebackOpened,
    ChargebackWon,
    ChargebackLost,
}
```

---

# 88. Hard Rule

Unknown provider event type safely ignored/quarantined until supported.

---

# 89. Purchase Workflow

```text
create purchase intent
→ provider checkout
→ provider confirmation
→ issue entitlement
→ receipt
```

---

# 90. Hard Rule

Entitlement not issued from client-side “success” alone.

---

# 91. Checkout Session

Short-lived.

---

# 92. Hard Rule

Checkout session cannot authorize extension runtime directly.

---

# 93. Receipt Identity

```rust
pub struct ExtensionReceiptId(pub [u8; 16]);
```

---

# 94. Receipt

```rust
pub struct ExtensionCommerceReceipt {
    pub receipt_id: ExtensionReceiptId,
    pub purchase: ExtensionPurchaseId,
    pub product: ExtensionCommercialProductId,
    pub amount: Money,
    pub issued_at: Timestamp,
}
```

---

# 95. Hard Rule

Receipt excludes unnecessary payment credentials.

---

# 96. Receipt Privacy

Publisher sees:

```text
product
amount
country/tax class if legally needed
transaction status
```

not:

```text
full billing identity
card details
private account data
```

---

# 97. Hard Rule

Minimum necessary publisher commerce data.

---

# 98. Entitlement Issuance

```rust
pub trait ExtensionEntitlementService {
    fn issue(
        &self,
        purchase: ExtensionPurchaseId,
    ) -> Result<ExtensionEntitlementId, ExtensionCommerceError>;
}
```

---

# 99. Hard Rule

Issuance checks payment state idempotently.

---

# 100. Entitlement Token

For offline/local verification.

---

# 101. Signed Entitlement Token

```rust
pub struct SignedExtensionEntitlement {
    pub entitlement: ExtensionEntitlement,
    pub issuer: EntitlementIssuerId,
    pub signature: Signature,
}
```

---

# 102. Hard Rule

Token contains no raw payment identity.

---

# 103. Entitlement Verification

Local device verifies signature + scope + expiry.

---

# 104. Hard Rule

No network round-trip required on every extension launch.

---

# 105. Offline Grace

```rust
pub struct ExtensionOfflineEntitlementPolicy {
    pub max_offline_duration: Duration,
}
```

---

# 106. Hard Rule

Offline grace explicit.

---

# 107. Expired Offline Token

Extension may enter limited/read-only/disabled paid-feature state.

---

# 108. Hard Rule

Do not delete user data because entitlement cannot currently be refreshed.

---

# 109. Read-Only Grace

Recommended for durable paid extension data where feasible.

---

# 110. Hard Rule

Commercial access loss must not destroy user-owned data.

---

# 111. Subscription Renewal

Provider event extends entitlement.

---

# 112. Hard Rule

Renewal changes commercial validity only.

---

# 113. Past Due

May enter grace period.

---

# 114. Grace Period

```rust
pub struct ExtensionGracePeriodPolicy {
    pub duration: Duration,
    pub capabilities: GraceCapabilitySet,
}
```

---

# 115. Hard Rule

Grace capabilities concern commercial features, not security permissions.

---

# 116. Grace Behavior

Examples:

```text
full access
read-only
limited premium features
```

---

# 117. Hard Rule

Behavior disclosed before purchase.

---

# 118. Cancellation

Subscription cancellation may be immediate or period-end.

---

# 119. Hard Rule

Cancellation semantics explicit.

---

# 120. Cancel At Period End

Entitlement remains until paid period ends.

---

# 121. Hard Rule

No early entitlement termination unless refund/abuse/legal policy says so.

---

# 122. Pause

Optional.

---

# 123. Hard Rule

Publisher/platform cannot silently pause for arbitrary commercial reasons if contract disallows it.

---

# 124. Refund

```rust
pub struct ExtensionRefund {
    pub refund_id: ExtensionRefundId,
    pub purchase: ExtensionPurchaseId,
    pub amount: Money,
    pub reason: ExtensionRefundReason,
}
```

---

# 125. Refund Reason

```rust
pub enum ExtensionRefundReason {
    UserRequested,
    DuplicateCharge,
    TechnicalFailure,
    PolicyViolation,
    PublisherInitiated,
}
```

---

# 126. Hard Rule

Refund state separate from package revocation.

---

# 127. Full Refund

May revoke future entitlement.

---

# 128. Hard Rule

User data remains exportable/retained according to data policy.

---

# 129. Partial Refund

Does not necessarily revoke entitlement.

---

# 130. Hard Rule

Semantics explicit.

---

# 131. Chargeback

Chargeback may temporarily suspend entitlement.

---

# 132. Hard Rule

Chargeback cannot cause data deletion.

---

# 133. Chargeback State

```rust
pub enum ChargebackState {
    Open,
    Won,
    Lost,
    Closed,
}
```

---

# 134. Hard Rule

Dispute outcome drives entitlement according to policy.

---

# 135. Refund Window

Policy-defined.

---

# 136. Hard Rule

Refund rules disclosed in offer.

---

# 137. Consumer Rights

Marketplace implementation must support jurisdiction-specific requirements through policy layer.

---

# 138. Hard Rule

Do not encode one jurisdiction's rules as universal truth.

---

# 139. Tax Handling

Separate service.

---

# 140. Tax Mode

```rust
pub enum TaxMode {
    TaxInclusive,
    TaxExclusive,
    ProviderCalculated,
}
```

---

# 141. Hard Rule

Tax calculation not mixed with extension permission logic.

---

# 142. Invoice

Optional/required depending context.

---

# 143. Hard Rule

Invoice data kept in commerce domain.

---

# 144. Enterprise Licensing

Organization may buy seats or tenant-wide license.

---

# 145. Enterprise License Model

```rust
pub enum EnterpriseLicenseModel {
    TenantWide,
    SeatBased,
    DeviceBased,
    ConcurrentUse,
}
```

---

# 146. Hard Rule

License model explicit.

---

# 147. Seat Identity

Use opaque seat allocation, not employee surveillance.

---

# 148. Hard Rule

No per-employee productivity telemetry for licensing.

---

# 149. Seat Assignment

Tenant admin can assign/revoke.

---

# 150. Hard Rule

Personal context not exposed to tenant license admin.

---

# 151. Concurrent License

Requires active lease.

---

# 152. Hard Rule

Lease metadata minimized.

---

# 153. Tenant Billing

Tenant billing identity separate from extension user identities.

---

# 154. Hard Rule

Publisher receives aggregate seat/license data only as needed.

---

# 155. Device-Based License

Useful for offline/embedded installations.

---

# 156. Hard Rule

Device identity used only within licensing scope.

---

# 157. Bundles

Multiple paid extensions/products.

---

# 158. Bundle Identity

```rust
pub struct ExtensionBundleId(pub [u8; 16]);
```

---

# 159. Hard Rule

Bundle entitlement decomposes into explicit child entitlements.

---

# 160. Bundle Refund

Defines allocation rules.

---

# 161. Hard Rule

Revenue allocation deterministic.

---

# 162. Marketplace Promotions

Optional.

---

# 163. Promotion Identity

```rust
pub struct ExtensionPromotionId(pub [u8; 16]);
```

---

# 164. Hard Rule

Promotions do not affect security/trust ranking.

---

# 165. Coupon

Optional, scoped.

---

# 166. Hard Rule

Coupon does not reveal sensitive user profile.

---

# 167. No Behavioral Price Discrimination

Hard rule.

---

# 168. Regional Pricing

May be supported using explicit market/currency policy.

---

# 169. Hard Rule

Do not derive sensitive personal attributes for price.

---

# 170. Price Transparency

Show:

```text
gross price
billing frequency
trial terms
renewal date
tax handling
cancellation/refund terms
```

---

# 171. Hard Rule

No hidden recurring charge.

---

# 172. Publisher Revenue Reporting

Publisher dashboard may show:

```text
gross revenue
refunds
fees
tax withholding
net payout
subscription counts
```

---

# 173. Hard Rule

No user purchase identity unless legally/operationally required and minimized.

---

# 174. No Cross-Product User Tracking

Hard rule.

---

# 175. Revenue Analytics

Aggregate by:

```text
product
period
currency
region class if legally necessary
```

---

# 176. Hard Rule

No detailed behavioral funnels.

---

# 177. Marketplace Fee Governance

Fee policy versioned and published to publishers.

---

# 178. Hard Rule

No secret publisher-specific fee manipulation.

---

# 179. Enterprise Negotiated Terms

Allowed where contract explicitly defines them.

---

# 180. Hard Rule

Negotiated commercial terms cannot alter security/privacy governance.

---

# 181. Payout Identity

```rust
pub struct PublisherPayoutAccountId(pub [u8; 16]);
```

---

# 182. Hard Rule

Payout identity separate from publisher runtime/signing identity.

---

# 183. Payout Provider

External provider may handle bank transfer.

---

# 184. Hard Rule

Payment/payout provider data minimized.

---

# 185. Payout State

```rust
pub enum PublisherPayoutState {
    Pending,
    Processing,
    Paid,
    Failed,
    Held,
}
```

---

# 186. Hard Rule

Payout failure does not suspend extension users automatically.

---

# 187. Fraud/Risk Hold

May hold publisher settlement.

---

# 188. Hard Rule

Economic hold separate from package security revocation.

---

# 189. Publisher Debt

Refunds/chargebacks may create negative balance.

---

# 190. Hard Rule

Negative payout balance does not automatically revoke extensions unless contract/policy separately says so.

---

# 191. Marketplace Insolvency Isolation

Entitlement ledger should survive billing-provider outage where possible.

---

# 192. Hard Rule

Billing provider outage does not instantly revoke active entitlements.

---

# 193. Billing Provider Outage

Use cached signed entitlement + grace policy.

---

# 194. Hard Rule

No online hard dependency for every app launch.

---

# 195. Billing Provider Migration

Possible.

---

# 196. Hard Rule

Migration preserves entitlement identity/history without exposing raw credentials.

---

# 197. Entitlement Re-Issuance

New issuer signs equivalent entitlements.

---

# 198. Hard Rule

No double entitlement inflation.

---

# 199. Commerce Ledger

Authoritative transactional ledger.

---

# 200. Ledger Event

```rust
pub enum CommerceLedgerEvent {
    PurchaseCreated,
    PaymentConfirmed,
    EntitlementIssued,
    SubscriptionRenewed,
    RefundIssued,
    ChargebackOpened,
    EntitlementSuspended,
    PayoutSettled,
}
```

---

# 201. Hard Rule

Append-only event history or equivalent immutable accounting journal.

---

# 202. Double-Entry Accounting

Recommended for marketplace settlement.

---

# 203. Hard Rule

Publisher payout ledger reconciles to provider settlements.

---

# 204. Reconciliation

Daily/periodic.

---

# 205. Hard Rule

Mismatch becomes financial incident, not silent adjustment.

---

# 206. Reconciliation Record

```rust
pub struct CommerceReconciliationRecord {
    pub period: SettlementPeriodRef,
    pub expected: Money,
    pub provider_reported: Money,
    pub difference: Money,
}
```

---

# 207. Hard Rule

Difference explicit.

---

# 208. Entitlement Ledger

Separate from financial ledger.

---

# 209. Hard Rule

Financial correction does not silently mutate entitlement history.

---

# 210. Refund Entitlement Policy

Explicit mapping.

---

# 211. Hard Rule

Commerce event→entitlement effect deterministic.

---

# 212. Local-First Entitlement Cache

Device keeps signed entitlement snapshot.

---

# 213. Hard Rule

Cache keyed by product/scope, not purchase-history tracking.

---

# 214. Cache Refresh

Triggered by:

```text
purchase
renewal
refund
periodic refresh
manual restore
```

---

# 215. Hard Rule

No high-frequency entitlement beaconing.

---

# 216. Restore Purchases

User can restore paid extension entitlement.

---

# 217. Hard Rule

Restore uses authenticated commerce account context, not messaging identity.

---

# 218. New Device

Fetch entitlement intent/signed token.

---

# 219. Hard Rule

Device still independently verifies extension package/capabilities.

---

# 220. Account Recovery

Restores commercial entitlements separately from runtime authority.

---

# 221. Hard Rule

Recovery does not restore stale capability tokens.

---

# 222. Family/Group Sharing

Optional future capability.

---

# 223. Hard Rule

Sharing policy explicit and scope-bounded.

---

# 224. No Unbounded Credential Sharing

Hard rule.

---

# 225. Entitlement Transfer

Usually not allowed unless product policy supports it.

---

# 226. Hard Rule

Transfer does not imply transfer of user extension data.

---

# 227. Gift Purchase

Optional.

---

# 228. Hard Rule

Gift code/token contains no payer identity.

---

# 229. Promo Code

Rate-limited/anti-abuse.

---

# 230. Hard Rule

No coupon brute-force enumeration.

---

# 231. Commerce Abuse

Examples:

```text
refund abuse
chargeback abuse
coupon abuse
entitlement replay
receipt tampering
```

---

# 232. Hard Rule

Commerce abuse detection uses transaction mechanics, not private messaging behavior.

---

# 233. Entitlement Replay Protection

Signed token has:

```text
entitlement ID
scope
issuer
validity
epoch
```

---

# 234. Hard Rule

Revoked entitlement epoch invalidates stale token.

---

# 235. Entitlement Epoch

```rust
pub struct ExtensionEntitlementEpoch(pub u64);
```

---

# 236. Hard Rule

Anti-rollback enforced.

---

# 237. Entitlement Revocation List

Signed and cached.

---

# 238. Hard Rule

Old token cannot override newer revocation.

---

# 239. Commerce Privacy Boundary

Billing identity is one of the most sensitive cross-domain identifiers.

---

# 240. Hard Rule

Do not reuse:

```text
billing customer ID
purchase ID
receipt ID
subscription ID
```

as:

```text
message sender ID
contact ID
telemetry ID
marketplace ranking ID
```

---

# 241. Publisher Privacy Boundary

Publisher receives only commerce data necessary to:

```text
understand product revenue
support entitlements
resolve refunds/support
```

---

# 242. Hard Rule

No private conversation/activity data shared for commerce.

---

# 243. Telemetry Boundary

Part 145 aggregate telemetry cannot be joined with purchase identity for behavioral monetization.

---

# 244. Hard Rule

No purchase-to-engagement user profiles.

---

# 245. Marketplace Ranking

Part 147.

---

# 246. Hard Rule

Paying publisher cannot buy ranking/trust/security advantage.

---

# 247. Ads / Sponsored Discovery

If ever supported, must be visually distinct and separate from governance/ranking.

---

# 248. Hard Rule

Sponsored placement cannot change trust/certification status.

---

# 249. No Commercial Override

Hard rule.

---

# 250. Free Extension Economics

Publisher may:

```text
accept donations
offer paid companion product
operate free/open-source extension
```

---

# 251. Hard Rule

Donation does not create hidden user tracking.

---

# 252. Open-Source Paid Distribution

Supported.

---

# 253. Hard Rule

Source availability and paid distribution are distinct.

---

# 254. Licensing Model

Commercial licensing independent from software source license.

---

# 255. Hard Rule

Marketplace displays commercial terms separately from software license.

---

# 256. Enterprise Contract

Can include:

```text
support SLA
volume discount
private distribution
seat licensing
```

---

# 257. Hard Rule

Enterprise contract cannot weaken extension sandbox/privacy floor.

---

# 258. Tax / Legal Data Boundary

Tax records may require legal identity.

---

# 259. Hard Rule

Tax/legal identity stored in commerce/compliance domain only.

---

# 260. No Messaging Use

Hard rule.

---

# 261. Data Retention

Commerce records may require longer retention than extension telemetry.

---

# 262. Hard Rule

Retention policy class-specific and legally governed.

---

# 263. Payment Credential Retention

Avoid directly storing raw card/bank credentials.

---

# 264. Hard Rule

Use provider tokenization.

---

# 265. Commerce Export

User may export receipts/subscription history.

---

# 266. Hard Rule

Export does not include secrets/provider tokens.

---

# 267. Publisher Export

Publisher can export settlement/accounting records.

---

# 268. Hard Rule

No user private account data beyond necessary transaction fields.

---

# 269. Deletion

User account deletion may not immediately erase legally required financial records.

---

# 270. Hard Rule

Commerce retention exception explicit and minimized.

---

# 271. Entitlement Continuity After Account Deletion

Depends on product/account lifecycle policy.

---

# 272. Hard Rule

No ambiguous orphan entitlement.

---

# 273. Commerce Audit

High-value events:

```text
price change
fee change
refund
payout
entitlement revoke
provider migration
```

---

# 274. Hard Rule

Audit contains commerce metadata, not private extension content.

---

# 275. Fraud Detection

Separate from extension abuse/security.

---

# 276. Hard Rule

Fraud model cannot inspect message content/social graph.

---

# 277. Risk Score

If used by payment provider, treated as external billing signal.

---

# 278. Hard Rule

Not reused for marketplace governance/trust ranking.

---

# 279. Commerce Incident

Examples:

```text
double charge
wrong entitlement
settlement mismatch
provider webhook compromise
```

---

# 280. Hard Rule

Commerce incident does not automatically become extension security incident.

---

# 281. Commerce Incident Response

Separate operational track.

---

# 282. Hard Rule

Can coordinate with Part 146 only when technical security overlap exists.

---

# 283. Entitlement Service API

```rust
pub trait ExtensionEntitlementVerifier {
    fn verify(
        &self,
        token: &SignedExtensionEntitlement,
        product: ExtensionCommercialProductId,
        scope: ExtensionEntitlementScope,
    ) -> Result<EntitlementDecision, ExtensionCommerceError>;
}
```

---

# 284. Entitlement Decision

```rust
pub enum EntitlementDecision {
    Active,
    Grace,
    Expired,
    Revoked,
    Invalid,
}
```

---

# 285. Hard Rule

Decision concerns paid feature access only.

---

# 286. Catalog Service

```rust
pub trait ExtensionCommerceCatalogService {
    fn catalog(
        &self,
        extension: ExtensionId,
    ) -> Result<ExtensionCommerceCatalog, ExtensionCommerceError>;
}
```

---

# 287. Purchase Service

```rust
pub trait ExtensionPurchaseService {
    async fn create_checkout(
        &self,
        product: ExtensionCommercialProductId,
        price: ExtensionPriceId,
    ) -> Result<CheckoutSessionRef, ExtensionCommerceError>;
}
```

---

# 288. Subscription Service

```rust
pub trait ExtensionSubscriptionService {
    async fn cancel(
        &self,
        subscription: ExtensionSubscriptionId,
        timing: CancellationTiming,
    ) -> Result<(), ExtensionCommerceError>;
}
```

---

# 289. Refund Service

```rust
pub trait ExtensionRefundService {
    async fn request(
        &self,
        purchase: ExtensionPurchaseId,
        reason: ExtensionRefundReason,
    ) -> Result<ExtensionRefundId, ExtensionCommerceError>;
}
```

---

# 290. Settlement Service

```rust
pub trait PublisherSettlementService {
    fn statement(
        &self,
        publisher: PublisherId,
        period: SettlementPeriodRef,
    ) -> Result<PublisherSettlementStatement, ExtensionCommerceError>;
}
```

---

# 291. Error Taxonomy

```rust
pub enum ExtensionCommerceError {
    ProductUnknown,
    PriceUnknown,
    CurrencyUnsupported,
    PaymentFailed,
    BillingProviderUnavailable,
    WebhookInvalid,
    SubscriptionInactive,
    RefundNotAllowed,
    ChargebackOpen,
    EntitlementExpired,
    EntitlementRevoked,
    SignatureInvalid,
    PolicyDenied,
    ReconciliationMismatch,
    Unauthorized,
    Internal,
}
```

---

# 292. Commerce Observability

Safe metrics:

```text
payment success rate
refund rate
renewal failure rate
payout reconciliation mismatch
entitlement verification failure
```

---

# 293. Hard Rule

Aggregate technical/financial metrics only.

---

# 294. Forbidden:

```text
message activity
contact graph
user engagement
private extension usage
```

---

# 295. Commerce SLOs

Examples:

```text
entitlement issuance within target after confirmed payment
refund processing within target
webhook reconciliation within target
offline verification availability
```

---

# 296. Hard Rule

Commercial SLO failure cannot weaken security/privacy controls.

---

# 297. Financial Integrity Invariants

```text
gross = platform fee + taxes + refunds/adjustments + publisher net
```

with accounting policy-defined exact treatment.

---

# 298. Hard Rule

Settlement equations deterministic.

---

# 299. No Silent Rounding Drift

Hard rule.

---

# 300. Rounding Policy

Explicit per currency.

---

# 301. Hard Rule

Rounding reproducible.

---

# 302. Testing

Need extension-commerce testkit.

Required scenarios:

```text
one-time purchase
subscription renewal
failed renewal + grace
full refund
chargeback
offline entitlement
```

---

# 303. Idempotency Test

Duplicate provider webhook issues one entitlement.

---

# 304. Replay Test

Old entitlement token rejected after revocation epoch.

---

# 305. Refund Test

Full refund changes commercial entitlement without deleting extension data.

---

# 306. Chargeback Test

Chargeback suspension does not corrupt local user data.

---

# 307. Offline Test

Signed active entitlement verifies offline within grace.

---

# 308. Billing Outage Test

Existing valid entitlement continues according to cached policy.

---

# 309. Security Test

Paid entitlement does not grant SIAR permissions.

---

# 310. Publisher Privacy Test

Publisher dashboard cannot retrieve user messaging identity.

---

# 311. Telemetry Isolation Test

Purchase identity cannot join extension usage telemetry.

---

# 312. Fee Calculation Test

Settlement sums exactly.

---

# 313. Provider Migration Test

Entitlements survive provider change without duplicate grants.

---

# 314. Enterprise Seat Test

Tenant seat assignment exposes no personal-context data.

---

# 315. Fuzzing

Fuzz:

```text
commerce catalog
signed entitlement token
billing event
refund record
settlement statement
```

---

# 316. Property Tests

Properties:

```text
payment retry can never create more than one purchase entitlement
commercial entitlement can never imply runtime security permission
revoked entitlement epoch can never be downgraded by older token
publisher settlement can never include hidden user-activity fields
```

---

# 317. Formal Verification Targets

Strong candidates:

```text
payment→entitlement state
subscription lifecycle
refund/chargeback transitions
settlement accounting
entitlement anti-rollback
```

---

# 318. Kani Candidate

money/idempotency/entitlement invariants.

---

# 319. TLA+ Candidate

```text
checkout → payment → entitlement → renewal/failure → refund/chargeback → expire/revoke
```

---

# 320. Loom Candidate

Concurrent:

```text
payment webhook
refund request
subscription renewal
entitlement refresh
```

---

# 321. Performance

Commerce is control-plane work.

---

# 322. Hard Rule

No billing provider call on every extension host call.

---

# 323. Entitlement Cache

Local signed cache.

---

# 324. Hard Rule

Revocation epoch checked when refreshed/known.

---

# 325. Checkout Latency

External PSP bounded by user-facing workflow, not runtime hot path.

---

# 326. Webhook Queue

Durable/bounded.

---

# 327. Hard Rule

Webhook backlog cannot lose financial events silently.

---

# 328. Settlement Batch

Periodic.

---

# 329. Hard Rule

Accounting workloads separate from messaging runtime.

---

# 330. Storage Domains

Separate:

```text
commerce catalog
purchase ledger
subscription ledger
entitlement ledger
refund/chargeback records
publisher settlements
tax/invoice records
```

---

# 331. Hard Rule

Commerce DB separated logically from messaging/contact data.

---

# 332. Secrets

Store:

```text
PSP API keys
webhook secrets
payout credentials
```

in secure secret system.

---

# 333. Hard Rule

No payment secret in extension package/runtime.

---

# 334. Partitioning

By:

```text
publisher
product
purchase/subscription
tenant for enterprise billing
```

---

# 335. Hard Rule

No message/conversation partition.

---

# 336. Crate Layout

Recommended:

```text
crates/
├── siar-extension-commerce-core/
├── siar-extension-catalog/
├── siar-extension-pricing/
├── siar-extension-billing/
├── siar-extension-entitlements/
├── siar-extension-subscriptions/
├── siar-extension-refunds/
├── siar-extension-settlement/
├── siar-extension-commerce-policy/
├── siar-extension-commerce-observability/
└── siar-extension-commerce-testkit/
```

---

# 337. `siar-extension-commerce-core`

Owns:

```text
ExtensionOfferId
ExtensionPurchaseId
ExtensionSubscriptionId
ExtensionCommerceError
Money
CurrencyCode
```

---

# 338. `siar-extension-catalog`

Commercial products/offers/catalog epochs.

---

# 339. `siar-extension-pricing`

Price versions, currencies, billing periods, promotions.

---

# 340. `siar-extension-billing`

PSP adapters, checkout, webhook normalization/idempotency.

---

# 341. `siar-extension-entitlements`

Signed entitlements, local verification, grace, anti-rollback.

---

# 342. `siar-extension-subscriptions`

Recurring lifecycle, renewal/cancel/pause.

---

# 343. `siar-extension-refunds`

Refund/chargeback workflows and entitlement effects.

---

# 344. `siar-extension-settlement`

Revenue sharing, fees, payout/reconciliation.

---

# 345. `siar-extension-commerce-policy`

Refund/trial/grace/fee/enterprise policy.

---

# 346. `siar-extension-commerce-observability`

Aggregate commerce health only.

---

# 347. `siar-extension-commerce-testkit`

billing/entitlement/refund/privacy/accounting tests.

---

# 348. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Commercial offers, prices, payment transactions, billing identities, subscriptions, entitlements, extension installation, runtime permissions, marketplace governance, publisher payouts, and anonymous messaging identities are distinct domains and cannot be used interchangeably.
2. Paying for an extension or feature grants only the explicit commercial entitlement; it can never grant message/file/network/device/admin capability, bypass user consent, weaken sandboxing, override privacy mode, or elevate marketplace trust.
3. Billing/customer/payment-provider identifiers, receipt IDs, subscription IDs, and purchase histories are never reused as messaging/contact IDs, telemetry correlation IDs, marketplace ranking IDs, or cross-product behavioral tracking identifiers.
4. Payment-provider credentials and raw payment methods remain outside extension/publisher runtimes; SIAR stores provider references/tokens only where necessary, validates signed webhooks, and uses idempotency to prevent duplicate charges or duplicate entitlement issuance.
5. Entitlements are signed, scoped, expiry/epoch aware, locally verifiable, and anti-rollback protected; offline verification avoids per-launch billing beacons while stale tokens cannot override a newer entitlement revocation.
6. Billing-provider outage, temporary renewal failure, cancellation, refund, chargeback, or entitlement expiration can reduce paid-feature access according to explicit policy but cannot silently delete user-owned extension data, revoke unrelated SIAR permissions, or corrupt local state.
7. Publisher revenue sharing, platform fees, taxes, refunds, reserves, settlement, and rounding are explicit, reproducible, versioned accounting rules with immutable historical transaction economics and no hidden fee bucket.
8. Publisher commerce dashboards receive only minimum necessary aggregate transaction/revenue information and cannot expose user messaging identities, contact graphs, private extension activity, or join purchase history with observability for behavioral monetization.
9. Marketplace ranking, certification, governance, appeals, security review, incident handling, and trust signals are independent from publisher revenue, advertising spend, platform fee tier, or payment volume; commercial relationships can never buy security/privacy exceptions.
10. Enterprise licensing is tenant-scoped and may allocate seats/devices/concurrency without exposing personal-context data or creating employee productivity surveillance; tenant billing identity remains separate from user extension activity.
11. Commerce records may retain legally required financial data under separate retention policy, but tax/billing/legal identity stays confined to the commerce/compliance domain and is never repurposed for anonymous communication, extension telemetry, or user profiling.
12. Extension economics integrates with marketplace distribution, publisher governance, package verification, entitlement-aware UI, state sync, account/device recovery, incident response, refunds, subscriptions, settlement, and offline operation without creating a side channel around SIAR's security, privacy, anonymity, local-first, or tenant-isolation guarantees.
```

---

# 349. Initial Production Scope

Implement first:

```text
typed commercial products/offers/prices
fixed-point Money
one-time purchase
monthly/annual subscription
trial policy
billing provider abstraction
authenticated/idempotent webhooks
purchase ledger
subscription state machine
signed entitlement tokens
local/offline entitlement verification
grace periods
cancel-at-period-end
full/partial refunds
chargeback state
publisher revenue share
platform fee versioning
settlement/reconciliation
enterprise tenant-wide + seat licensing
receipt export
publisher aggregate revenue dashboard
commerce privacy boundaries
extension-commerce testkit
```

Then add:

```text
bundles
device/concurrent licensing
multi-currency localized pricing
promotions/coupons
cross-provider entitlement migration
family/group sharing
formal settlement proofs
privacy-preserving marketplace economic analytics
```

---

# 350. Definition of Done

Part 148 is complete when:

- pricing/products/offers are typed and versioned;
- billing and messaging identity are separate;
- purchases/subscriptions issue explicit entitlements;
- entitlements are locally verifiable and anti-rollback;
- commercial access never grants security permissions;
- renewals/refunds/chargebacks have deterministic entitlement effects;
- billing failures never silently destroy extension data;
- publisher/platform fee settlement is reproducible;
- enterprise licensing is tenant-scoped without workforce surveillance;
- publisher dashboards expose aggregate commerce only;
- commerce cannot affect marketplace trust/governance;
- billing/entitlement/refund/privacy/accounting/fuzz/formal tests are specified.

---

# 351. Final Architecture

```text
                 EXTENSION OFFER
                        │
                        ▼
                  PRICE / TERMS
                        │
                        ▼
                   CHECKOUT
                        │
                        ▼
                PAYMENT CONFIRMATION
                        │
                        ▼
               SIGNED ENTITLEMENT
                        │
             ┌──────────┼──────────┐
             │          │          │
          DEVICE      ACCOUNT    TENANT
             │          │          │
             └──────────┼──────────┘
                        ▼
                  PAID FEATURE ACCESS
                        │
                        ▼
                SETTLEMENT / REFUND
```

Marketplace-economics safety model:

```text
explicit offers
+
fixed-point accounting
+
provider isolation
+
signed entitlements
+
offline verification
+
deterministic refunds
+
transparent revenue sharing
+
identity separation
+
aggregate commerce analytics
```

not:

```text
tie purchases to messaging identity, phone home on every launch, delete user data when payment fails, or let high-paying publishers buy trust and marketplace privilege
```

---

# 352. Final Principle

Extension monetization is trustworthy when SIAR can answer **what was purchased, what commercial access it grants, how long that access lasts, how money is split, what happens after cancellation/refund/failure, and which identities never cross the commerce boundary**.

The correct model is:

```text
price transparently
+
bill through isolated providers
+
issue signed commercial entitlements
+
verify locally where possible
+
keep payment separate from security authority
+
preserve user data through billing failures
+
reconcile publisher revenue exactly
+
support refunds and disputes deterministically
+
never monetize anonymous communication metadata or governance trust
```

This architecture gives SIAR a privacy-preserving marketplace economics foundation for paid extensions, subscriptions, trials, billing integration, entitlements, refunds, chargebacks, enterprise licensing, revenue sharing, publisher settlements, offline access, and commercial sustainability while preserving the anonymity, local-first, least-authority, marketplace-governance, runtime, permission, data-governance, observability, and anti-surveillance guarantees established across Parts 34–147.
