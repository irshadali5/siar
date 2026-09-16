# Core System Architecture Part 69 — Anonymous Network Multi-Tenant Isolation, Organizational Boundaries, Delegated Administration & Enterprise Policy Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 69  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 45, 47–58, 61–68  

**Primary purpose:** define multi-tenant isolation, organizational boundaries, delegated administration, managed-device/BYOD policy, enterprise discovery, quotas, billing/accounting boundaries, cross-tenant sharing, tenant federation, legal/privacy isolation, and policy inheritance so multiple organizations can use common SIAR infrastructure without gaining authority over one another or collapsing identity/privacy boundaries.

---

# 1. Purpose

Enterprise or hosted SIAR deployments may serve multiple organizations.

Examples:

```text
school districts
universities
businesses
NGOs
government departments
community networks
managed service customers
```

Naive multi-tenancy is dangerous.

Shared infrastructure can accidentally create:

```text
cross-tenant data leakage
shared admin authority
global identity linkage
quota interference
billing correlation
policy inheritance bugs
shared-key blast radius
```

The governing principle is:

> **Tenants may share infrastructure, but they must not share authority, identity, cryptographic trust, or data visibility unless an explicit cross-tenant relationship says so.**

---

# 2. Architectural Position

```text
Shared Infrastructure
        │
        ▼
Tenant Isolation Layer
        │
   ┌────┼────┬────┐
   │    │    │    │
Tenant A   Tenant B   Tenant C
   │         │         │
   ▼         ▼         ▼
Org Policy  Org Policy  Org Policy
```

---

# 3. Core Separation

Keep distinct:

```text
infrastructure operator
tenant
organization
team/unit
managed user
personal user
device
billing account
admin authority
federation domain
```

---

# 4. Non-Goals

Part 69 does not create:

```text
one global enterprise administrator
cross-tenant superuser by default
shared tenant encryption keys
global employee directory
tenant-wide access to private message content
```

---

# 5. Tenant Identifier

```rust
pub struct TenantId(pub [u8; 32]);
```

Opaque.

---

# 6. Organization Identifier

```rust
pub struct OrganizationId(pub [u8; 32]);
```

---

# 7. Tenant vs Organization

A tenant is an infrastructure isolation boundary.

An organization is a product/governance boundary.

Often one-to-one, but not required.

---

# 8. Shared Tenant

Could host multiple related organizations.

---

# 9. Dedicated Tenant

One organization per isolation domain.

---

# 10. Tenant Mode

```rust
pub enum TenantMode {
    SharedInfrastructure,
    Dedicated,
    AirGappedDedicated,
}
```

---

# 11. Isolation Domains

Need separation across:

```text
data
configuration
keys
namespace
quota
billing
audit
admin
```

---

# 12. Tenant Context

```rust
pub struct TenantContext {
    pub tenant: TenantId,
    pub organization: Option<OrganizationId>,
}
```

---

# 13. No Ambient Current Tenant

Preferred hard rule.

---

# 14. Why

Implicit global tenant state causes cross-tenant bugs.

---

# 15. Explicit Context

Every tenant-scoped operation carries explicit context.

---

# 16. Tenant-Scoped Type

```rust
pub struct TenantScoped<T> {
    pub tenant: TenantId,
    pub value: T,
}
```

---

# 17. Type Safety

Use wrappers to prevent accidental mixing.

---

# 18. Example

```text
TenantScoped<GroupId>
```

cannot silently become another tenant's group.

---

# 19. Tenant Keyspace

Each tenant gets independent logical keyspace.

---

# 20. Database Isolation Modes

```rust
pub enum TenantDataIsolation {
    RowLevel,
    SchemaPerTenant,
    DatabasePerTenant,
    ClusterPerTenant,
}
```

---

# 21. Row-Level

Lowest cost.

Highest implementation risk.

---

# 22. SchemaPerTenant

Stronger logical isolation.

---

# 23. DatabasePerTenant

Stronger operational boundary.

---

# 24. ClusterPerTenant

Strongest, highest cost.

---

# 25. Recommendation

Use risk-tiered model.

---

# 26. High-Sensitivity Tenant

Prefer:

```text
database per tenant
or
dedicated cluster
```

---

# 27. Shared SaaS Tenant

Can use schema/row-level with strong enforcement.

---

# 28. Row-Level Isolation

Must be structural.

---

# 29. No Manual `WHERE tenant_id = ?` Discipline

Hard rule.

---

# 30. Tenant-Aware Repository

```rust
pub trait TenantRepository<T> {
    fn get(
        &self,
        ctx: &TenantContext,
        id: T::Id,
    ) -> Result<Option<T>, TenantError>;
}
```

---

# 31. Query Builder

Automatically scopes tenant.

---

# 32. Database RLS

Useful additional defense.

---

# 33. PostgreSQL RLS

Can enforce tenant filter.

---

# 34. Defense In Depth

Application + DB.

---

# 35. No Cross-Tenant Join

Hard rule except explicit administrative aggregate view.

---

# 36. Aggregate View

Must be non-user-identifying.

---

# 37. Tenant Cryptographic Boundary

No shared tenant content key.

---

# 38. Tenant Key Root

```rust
pub struct TenantKeyRootId(pub [u8; 32]);
```

---

# 39. Tenant Key Hierarchy

```text
tenant root
→ service KEKs
→ object DEKs
```

---

# 40. Hard Rule

Compromise of Tenant A key must not decrypt Tenant B data.

---

# 41. Tenant Root Storage

Secret store/HSM.

---

# 42. Tenant Key Rotation

Independent.

---

# 43. Shared Infrastructure Operator

Must not automatically possess tenant content keys.

---

# 44. Managed Escrow

If enterprise chooses it:

```text
explicit
tenant-scoped
documented
```

---

# 45. No Hidden Operator Escrow

Hard rule.

---

# 46. Tenant Namespace

Names scoped.

---

# 47. Example

```text
alice@org-a
alice@org-b
```

are unrelated.

---

# 48. Tenant Namespace ID

```rust
pub struct TenantNamespaceId(pub [u8; 32]);
```

---

# 49. No Global Employee Handle

Hard rule.

---

# 50. Enterprise Discovery

Can be organization-scoped.

---

# 51. Directory Visibility

```rust
pub enum EnterpriseDirectoryVisibility {
    Hidden,
    ExactSearch,
    OrganizationVisible,
    FederationVisible,
}
```

---

# 52. OrganizationVisible

Only members can browse/search.

---

# 53. FederationVisible

Only approved peer organizations.

---

# 54. No Public Default

Hard rule.

---

# 55. Managed Identity

Organization may manage:

```text
account enrollment
device enrollment
role membership
directory profile
```

---

# 56. But

Managed identity is separate from private communication keys.

---

# 57. Hard Rule

Org admin cannot read private message content merely because they manage identity.

---

# 58. Managed User

```rust
pub struct ManagedUserRef {
    pub tenant: TenantId,
    pub organization: OrganizationId,
    pub subject: ManagedSubjectId,
}
```

---

# 59. Managed Subject ID

Organization-scoped.

---

# 60. No Global Workforce ID

Hard rule.

---

# 61. Personal + Managed Identity

Same device may host both.

---

# 62. Profile Separation

```text
Personal profile
Managed org profile
```

---

# 63. Hard Rule

Org policy applies only to managed profile unless explicit device-management policy says otherwise.

---

# 64. BYOD

Bring-your-own-device support.

---

# 65. BYOD Boundary

Organization can control:

```text
managed app/profile
org data
org capabilities
```

---

# 66. Cannot

```text
inspect personal messages
read personal contacts
manage unrelated personal plugins
```

---

# 67. Managed Device

Stronger control.

---

# 68. Managed Device Policy

```rust
pub enum DeviceManagementMode {
    Unmanaged,
    ManagedProfile,
    FullyManaged,
}
```

---

# 69. Fully Managed

Org can enforce broader device policy.

---

# 70. UI Transparency

User sees:

```text
Managed by organization
```

---

# 71. Policy Source Visibility

Mandatory.

---

# 72. Enterprise Policy

```rust
pub struct EnterprisePolicyBundle {
    pub tenant: TenantId,
    pub organization: OrganizationId,
    pub version: EnterprisePolicyVersion,
    pub rules: Vec<EnterprisePolicyRule>,
    pub signatures: PolicySignatureBundle,
}
```

---

# 73. Policy Version

```rust
pub struct EnterprisePolicyVersion(pub u64);
```

---

# 74. Policy Scope

```rust
pub enum EnterprisePolicyScope {
    Tenant,
    Organization,
    Team,
    ManagedProfile,
}
```

---

# 75. Policy Inheritance

```text
tenant
→ organization
→ team
→ user/profile
```

---

# 76. Restrictive Merge

For privacy/security.

---

# 77. Enterprise Policy Cannot Weaken Network Floor

Hard rule.

---

# 78. Policy Precedence

```text
compile-time invariant
> network governance floor
> legal restriction
> tenant policy
> organization policy
> team policy
> user stronger privacy choice
> performance default
```

---

# 79. User Stronger Privacy Choice

Allowed.

---

# 80. User Cannot Disable Mandatory Org Security

If managed context.

---

# 81. But

UI must show reason/source.

---

# 82. Policy Resolver

```rust
pub trait EnterprisePolicyResolver {
    fn effective(
        &self,
        ctx: &EnterprisePolicyContext,
    ) -> Result<EffectiveEnterprisePolicy, TenantError>;
}
```

---

# 83. Delegated Administration

Organizations need admin delegation.

---

# 84. Admin Roles

```rust
pub enum EnterpriseAdminRole {
    TenantOwner,
    OrganizationAdmin,
    TeamAdmin,
    DeviceAdmin,
    SecurityAdmin,
    BillingAdmin,
    Auditor,
}
```

---

# 85. TenantOwner

Manages tenant-level product settings.

---

# 86. OrganizationAdmin

Manages organization membership/policy.

---

# 87. TeamAdmin

Scoped to one team/unit.

---

# 88. DeviceAdmin

Managed devices/profiles.

---

# 89. SecurityAdmin

Security policy/revocation.

---

# 90. BillingAdmin

Billing/quota only.

---

# 91. Auditor

Read-only audit.

---

# 92. No Superuser By Default

Hard rule.

---

# 93. Admin Capability

```rust
pub struct EnterpriseAdminCapability {
    pub tenant: TenantId,
    pub scope: EnterpriseAdminScope,
    pub actions: BTreeSet<EnterpriseAdminAction>,
    pub expires_at: Timestamp,
}
```

---

# 94. Admin Scope

```rust
pub enum EnterpriseAdminScope {
    Tenant,
    Organization(OrganizationId),
    Team(TeamId),
}
```

---

# 95. Admin Actions

```rust
pub enum EnterpriseAdminAction {
    ManageMembers,
    ManageTeams,
    ManageDevices,
    ManagePolicy,
    ManageBilling,
    ViewAudit,
    RevokeManagedAccess,
}
```

---

# 96. Scope Enforcement

Hard rule.

---

# 97. Team Admin

Cannot manage sibling team.

---

# 98. Billing Admin

Cannot access messages.

---

# 99. Security Admin

Cannot access billing unless granted.

---

# 100. Delegation

Admin can delegate narrower capability.

---

# 101. No Privilege Amplification

Hard rule.

---

# 102. Delegation Chain

Bounded depth.

---

# 103. Admin Delegation Record

```rust
pub struct AdminDelegationRecord {
    pub issuer: EnterpriseAdminId,
    pub delegate: EnterpriseAdminId,
    pub scope: EnterpriseAdminScope,
    pub actions: BTreeSet<EnterpriseAdminAction>,
    pub expires_at: Timestamp,
}
```

---

# 104. Admin Identity

Separate from user messaging identity.

---

# 105. Hard Rule

Admin actions never use communication identity as authorization.

---

# 106. Admin MFA

Recommended.

---

# 107. High-Risk Admin Action

May require:

```text
two-person approval
```

---

# 108. High-Risk Examples

```text
tenant key rotation
tenant deletion
federation trust change
managed recovery policy
```

---

# 109. Break-Glass

Tenant-scoped.

---

# 110. No Cross-Tenant Break-Glass

Hard rule.

---

# 111. Shared Operator Break-Glass

Infrastructure only.

---

# 112. It Cannot Read Tenant Content

Hard rule unless tenant explicitly escrowed keys.

---

# 113. Tenant Data Plane

Each request includes tenant context.

---

# 114. Tenant Context Propagation

```rust
pub struct TenantRequestContext {
    pub tenant: TenantId,
    pub request_id: ScopedRequestId,
}
```

---

# 115. Request ID

Scoped.

---

# 116. No Global Cross-Tenant Request ID

Hard rule.

---

# 117. Cache Isolation

Separate namespace per tenant.

---

# 118. Cache Key

```rust
pub struct TenantCacheKey {
    pub tenant: TenantId,
    pub key: CacheKey,
}
```

---

# 119. No Shared Cache Key Without Tenant Prefix

Hard rule.

---

# 120. Search Index Isolation

Separate tenant index or namespace.

---

# 121. No Cross-Tenant FTS

Hard rule.

---

# 122. Blob Storage Isolation

Prefix/bucket/key boundary.

---

# 123. Tenant Blob Namespace

Independent.

---

# 124. Cross-Tenant Dedup

Disabled.

---

# 125. Hard rule.

---

# 126. Why

Leaks content equality.

---

# 127. Queue Isolation

Per tenant.

---

# 128. Queue Key

```rust
pub struct TenantQueueKey {
    pub tenant: TenantId,
    pub queue: QueueClass,
}
```

---

# 129. One Tenant Cannot Starve Others

Hard rule.

---

# 130. Resource Governance

Part 49/63.

---

# 131. Tenant Quota

```rust
pub struct TenantQuota {
    pub storage_bytes: u64,
    pub bandwidth_bps: u64,
    pub relay_sessions: u64,
    pub api_operations: u64,
}
```

---

# 132. Quota Scope

Tenant/service.

---

# 133. User-Level Quota

Possible inside tenant.

---

# 134. But

No global user quota identity.

---

# 135. Quota Enforcement

At tenant boundary.

---

# 136. Fairness

Shared resources allocate:

```text
minimum guaranteed
+
burstable
```

---

# 137. No Tenant Privacy Downgrade Under Quota

Hard rule.

---

# 138. If Quota Exhausted

```text
reject/defer
```

not direct privacy bypass.

---

# 139. Billing Boundary

Tenant billing separate from communication identity.

---

# 140. Billing Account

```rust
pub struct TenantBillingAccountId(pub [u8; 32]);
```

---

# 141. Billing Mapping

Can map to tenant.

---

# 142. Cannot map directly to individual communications.

---

# 143. Hard rule.

---

# 144. Cost Allocation

Aggregate.

---

# 145. Example

```text
storage
relay minutes
egress
mailbox usage
```

---

# 146. Billing Admin

Sees aggregate.

---

# 147. No Per-Conversation Bill

Hard rule.

---

# 148. Tenant Wallet

Part 47 model possible.

---

# 149. Enterprise Subscription

Can use invoices/subscriptions.

---

# 150. Accounting Identity

Separate from federation/provider/user identity.

---

# 151. Audit Boundary

Tenant audit logs only tenant admin events.

---

# 152. No Message Content.

---

# 153. Tenant Audit Record

```rust
pub struct TenantAuditRecord {
    pub tenant: TenantId,
    pub actor: EnterpriseAdminId,
    pub action: EnterpriseAdminAction,
    pub scope: EnterpriseAdminScope,
    pub result: AuditResult,
}
```

---

# 154. Audit Retention

Tenant policy + legal.

---

# 155. Tenant Admin Should Not See Infrastructure Admin Audit

Unless operator contract says so.

---

# 156. Infrastructure Audit

Separate.

---

# 157. Cross-Tenant Audit

Only safe aggregate.

---

# 158. No Central "Who Talked To Whom" Audit

Hard rule.

---

# 159. Membership

Organizations manage membership.

---

# 160. Membership State

```rust
pub enum MembershipState {
    Invited,
    Active,
    Suspended,
    Revoked,
    Left,
}
```

---

# 161. Membership Capability

Scoped.

---

# 162. Join

Does not reveal user's personal identity unless policy requires.

---

# 163. Managed Account

May intentionally be identifiable to org.

---

# 164. Still

Communication transport identities remain separate.

---

# 165. Leaving Organization

Need clean separation.

---

# 166. Offboarding Flow

```text
revoke org capabilities
remove managed profile access
rotate org group keys
remove org directory entry
expire org provider credentials
```

---

# 167. Personal Profile

Unaffected.

---

# 168. Hard rule.

---

# 169. Organization-Owned Data

May remain per policy.

---

# 170. User-Owned Personal Data

Not captured.

---

# 171. Data Ownership Classification

```rust
pub enum EnterpriseDataOwnership {
    Personal,
    OrganizationOwned,
    Shared,
}
```

---

# 172. Ownership Affects

```text
retention
export
offboarding
```

---

# 173. No Implicit Ownership

Hard rule.

---

# 174. Enterprise Message Policy

Could classify specific managed conversations.

---

# 175. Personal Conversation

Org cannot redefine after fact.

---

# 176. Managed Workspace

Clear boundary.

---

# 177. Enterprise Group

Group membership controlled by org.

---

# 178. Group Key Rotation

On offboarding.

---

# 179. Historical Messages

Policy-specific.

---

# 180. No Universal Retroactive Erasure

Hard truth.

---

# 181. Enterprise Discovery

Directory scoped to organization.

---

# 182. Exact Search

Default recommended.

---

# 183. Full Browse

Optional.

---

# 184. Cross-Org Discovery

Explicit federation.

---

# 185. Tenant Federation

Part 58.

---

# 186. Org Federation Agreement

```rust
pub struct OrganizationFederationAgreement {
    pub local_org: OrganizationId,
    pub remote_domain: FederationDomainId,
    pub services: BTreeSet<FederatedEnterpriseService>,
    pub expires_at: Timestamp,
}
```

---

# 187. Federated Enterprise Service

```rust
pub enum FederatedEnterpriseService {
    Messaging,
    DirectoryLookup,
    GroupCollaboration,
    FileExchange,
}
```

---

# 188. No Full Admin Federation

Hard rule.

---

# 189. Cross-Tenant Sharing

Explicit capability.

---

# 190. Example

Tenant A user shares file with Tenant B user.

---

# 191. Share Capability

```rust
pub struct CrossTenantShareCapability {
    pub source_tenant: TenantId,
    pub destination_tenant: TenantId,
    pub resource: SharedResourceRef,
    pub expires_at: Timestamp,
}
```

---

# 192. No Implicit Cross-Tenant Read

Hard rule.

---

# 193. Sharing Creates

```text
narrow bilateral capability
```

not merged namespaces.

---

# 194. Cross-Tenant Group

Possible.

---

# 195. Recommended

Treat as federation object.

---

# 196. Governance

Explicit.

---

# 197. No Shared Tenant DB Shortcut

Hard rule.

---

# 198. Tenant Migration

Move tenant between infrastructure.

---

# 199. Migration Scope

```text
data
keys
config
audit
billing
```

---

# 200. Migration Does Not Change Org Identity

If continuity preserved.

---

# 201. Tenant Export Bundle

Encrypted.

---

# 202. Tenant Portability

```rust
pub struct TenantPortabilityBundle {
    pub tenant: TenantId,
    pub schema_version: u32,
    pub encrypted_payload: Bytes,
}
```

---

# 203. Operator Change

Should not require user identity changes.

---

# 204. Provider Migration

Rebind tenant-level services.

---

# 205. Key Migration

Preserve/rotate according policy.

---

# 206. Dedicated-to-Shared

Higher risk.

---

# 207. Requires explicit review.

---

# 208. Shared-to-Dedicated

Easier.

---

# 209. Tenant Deletion

High-risk.

---

# 210. State

```rust
pub enum TenantDeletionState {
    Requested,
    Approved,
    AccessRevoked,
    DataDeleting,
    KeysErasing,
    BackupExpiryPending,
    Completed,
}
```

---

# 211. Two-Person Approval

Recommended.

---

# 212. Tenant Delete

Must not affect others.

---

# 213. Hard rule.

---

# 214. Tenant Backup

Independent.

---

# 215. Backup Key

Tenant-scoped.

---

# 216. Restore

Cannot overwrite another tenant.

---

# 217. Hard rule.

---

# 218. Tenant Disaster Recovery

Part 50.

---

# 219. Shared Region Failure

Multiple tenants affected.

---

# 220. But

Recovery keeps isolation.

---

# 221. Tenant-Level RPO/RTO

Can differ by service tier.

---

# 222. Service Tier

```rust
pub enum EnterpriseServiceTier {
    Standard,
    HighAvailability,
    Dedicated,
}
```

---

# 223. Tier Affects

```text
replication
capacity
support
```

---

# 224. Tier Does Not Affect Privacy Floor

Hard rule.

---

# 225. Higher Price

May buy more availability/capacity.

---

# 226. Not weaker/stronger fundamental confidentiality by wealth.

---

# 227. Enterprise Legal Boundary

Part 55.

---

# 228. Tenant Jurisdiction

Can restrict:

```text
data residency
admin access region
backup region
```

---

# 229. Jurisdiction Policy

Tenant-scoped.

---

# 230. Shared Host

Must satisfy all colocated tenant requirements.

---

# 231. If conflict

Cannot colocate.

---

# 232. Placement Policy

```rust
pub struct TenantPlacementPolicy {
    pub allowed_regions: BTreeSet<RegionId>,
    pub dedicated_host: bool,
    pub forbidden_co_tenants: BTreeSet<TenantClass>,
}
```

---

# 233. No Silent Residency Violation

Hard rule.

---

# 234. Tenant Class

Could be:

```text
standard
regulated
high-security
```

---

# 235. Tenant Classification

Operational only.

---

# 236. No Public Disclosure.

---

# 237. Shared Host Side Channels

Potential:

```text
CPU
cache
memory pressure
network
```

---

# 238. High-Sensitivity Tenant

Dedicated host option.

---

# 239. Container Isolation

Not equivalent to hardware isolation.

---

# 240. Hard truth.

---

# 241. Multi-Tenant Runtime Isolation

Use:

```text
namespaces
cgroups
network namespaces
separate service identities
```

---

# 242. Database Connection Pools

Per tenant or bounded shared pool.

---

# 243. No One Tenant Connection Storm

Hard rule.

---

# 244. Tenant Rate Limit

Per API/service.

---

# 245. Noisy Neighbor

Part 63.

---

# 246. Per-Tenant Bulkhead

Required.

---

# 247. Tenant Circuit Breaker

```rust
pub enum TenantCircuitState {
    Closed,
    Open,
    HalfOpen,
}
```

---

# 248. Failure Isolation

Tenant B failure should not cascade to A.

---

# 249. Tenant Plugin Policy

Part 68.

---

# 250. Org can allowlist plugins.

---

# 251. Plugin Capability

Still user/org-scoped.

---

# 252. No Tenant Plugin Superuser

Hard rule.

---

# 253. Managed Integrations

Organization-wide integrations.

---

# 254. Example

```text
calendar
directory
storage
```

---

# 255. Integration Data Scope

Tenant-scoped.

---

# 256. Credential

Tenant-specific.

---

# 257. No Shared SaaS Refresh Token Across Tenants

Hard rule.

---

# 258. Integration Revocation

Independent.

---

# 259. Enterprise Search

Org-scoped.

---

# 260. No Cross-Tenant Search Index

Hard rule.

---

# 261. Organization Knowledge/Files

Can be shared inside managed workspace.

---

# 262. Access Control

Capability/role-based.

---

# 263. ACL

May exist at org layer.

---

# 264. But

Private messaging still uses relationship/security model.

---

# 265. No ACL Override Of E2EE

Hard rule.

---

# 266. Compliance Export

Tenant admin/legal flow.

---

# 267. Must obey Part 55.

---

# 268. No Operator-Level Bulk Tenant Export Without Authority

Hard rule.

---

# 269. Tenant Data Rights

Part 56.

---

# 270. Personal data request

Scoped to one tenant relationship.

---

# 271. Tenant Admin

Can manage org-owned records.

---

# 272. User

Can request access to personal data where applicable.

---

# 273. Multi-Tenant Observability

Safe metrics:

```text
tenant resource class
quota utilization bucket
service health
```

---

# 274. Avoid

```text
per-user communication metrics
```

---

# 275. Tenant Metrics Access

Org admin sees tenant aggregate.

---

# 276. Operator Metrics

Can see infrastructure aggregates.

---

# 277. No Cross-Tenant Business Intelligence

Hard rule unless explicit anonymized product analytics policy.

---

# 278. Tenant SLOs

Per service tier.

---

# 279. Privacy SLOs

Same minimum across tiers.

---

# 280. Isolation SLO

Examples:

```text
zero cross-tenant access violations
zero key reuse
zero cache namespace collision
```

---

# 281. Tenant Incident

Examples:

```text
cross-tenant query bug
tenant key leak
quota bleed
admin scope bypass
```

---

# 282. Severity

Usually high/critical.

---

# 283. Incident Playbook

Part 65.

---

# 284. Containment

Suspend affected tenant path.

---

# 285. Do Not Shut All Tenants Unless Required

Hard rule.

---

# 286. Cross-Tenant Leak

Preserve evidence minimally.

---

# 287. Rotate affected tenant keys if needed.

---

# 288. Tenant Isolation Testkit

Need dedicated.

---

# 289. Test Scenarios

```text
wrong tenant ID
cache collision
admin delegation bug
quota exhaustion
tenant deletion
tenant migration
cross-tenant sharing
```

---

# 290. Repository Test

Tenant A cannot fetch Tenant B row.

---

# 291. RLS Test

DB rejects cross-tenant query.

---

# 292. Cache Test

Same logical key in A/B remains separate.

---

# 293. Search Test

No cross-tenant result.

---

# 294. Blob Test

No cross-tenant dedup/exposure.

---

# 295. Key Test

Tenant A key cannot decrypt B data.

---

# 296. Admin Scope Test

Team admin cannot manage sibling team.

---

# 297. Billing Role Test

No message access.

---

# 298. BYOD Test

Org policy cannot inspect personal profile.

---

# 299. Offboarding Test

Managed access revoked, personal unaffected.

---

# 300. Quota Test

Tenant A overload does not starve B.

---

# 301. Tenant Delete Test

B survives untouched.

---

# 302. Migration Test

Tenant moves provider without identity collapse.

---

# 303. Federation Test

Cross-org communication uses explicit trust/capability.

---

# 304. Backup Restore Test

A backup cannot restore into B.

---

# 305. Legal Residency Test

Placement rejects forbidden region.

---

# 306. Plugin Test

Tenant allowlist does not grant hidden capability.

---

# 307. Fuzzing

Fuzz:

```text
tenant context
policy merge
admin scope
portability bundle
cross-tenant capability
```

---

# 308. Property Tests

Properties:

```text
tenant-scoped resource cannot be accessed with different TenantId
delegated admin capability is subset of issuer authority
tenant deletion cannot mutate unrelated tenant state
stronger user privacy choice is never weakened by org policy
```

---

# 309. Formal Verification Targets

Strong candidates:

```text
tenant-scoped authorization
policy inheritance
delegated admin attenuation
cross-tenant share capability
```

---

# 310. Kani Candidate

Tenant/resource scope matching.

---

# 311. TLA+ Candidate

Tenant migration + concurrent access.

---

# 312. Loom Candidate

Tenant deletion racing with requests.

---

# 313. Performance Tests

Measure:

```text
tenant scope enforcement
RLS overhead
per-tenant queueing
policy resolution
```

---

# 314. Scale Tests

Synthetic:

```text
10 tenants
1k tenants
10k tenants
```

---

# 315. Large Tenant

One tenant with many users.

---

# 316. Many Tiny Tenants

Different scaling profile.

---

# 317. No Per-Tenant Process Requirement

Hard rule for shared SaaS.

---

# 318. Dedicated Tenant

Can have separate deployment.

---

# 319. Tenant Sharding

Possible.

---

# 320. Shard Key

TenantId.

---

# 321. No User ID As Global Shard Key

Hard rule.

---

# 322. Tenant Move Between Shards

Explicit migration.

---

# 323. Control Plane

Tenant metadata only.

---

# 324. No user content.

---

# 325. Tenant Provisioning

State machine.

---

# 326. Provisioning State

```rust
pub enum TenantProvisioningState {
    Requested,
    Validating,
    Allocating,
    Configuring,
    Active,
    Suspended,
    Deleting,
    Deleted,
}
```

---

# 327. Tenant Provisioner

```rust
pub trait TenantProvisioner {
    fn provision(
        &self,
        request: TenantProvisioningRequest,
    ) -> Result<TenantId, TenantError>;
}
```

---

# 328. Provisioning Steps

```text
allocate namespace
create keys
create policy root
allocate storage
set quotas
enable services
```

---

# 329. No Active Before All Boundaries Ready

Hard rule.

---

# 330. Tenant Suspension

Can stop service.

---

# 331. Does Not Delete.

---

# 332. Read-Only Grace

Possible billing model.

---

# 333. Privacy Still intact.

---

# 334. Tenant Deprovisioning

Revoke first.

---

# 335. Then lifecycle delete.

---

# 336. Crate Layout

Recommended:

```text
crates/
├── siar-tenant-core/
├── siar-tenant-context/
├── siar-tenant-storage/
├── siar-enterprise-policy/
├── siar-enterprise-admin/
├── siar-tenant-quota/
├── siar-tenant-billing/
├── siar-tenant-federation/
├── siar-tenant-provisioning/
├── siar-tenant-observability/
└── siar-tenant-testkit/
```

---

# 337. `siar-tenant-core`

Owns:

```text
tenant/org/team IDs
modes
errors
```

---

# 338. `siar-tenant-context`

Explicit request/resource scoping.

---

# 339. `siar-tenant-storage`

DB/cache/blob/search isolation.

---

# 340. `siar-enterprise-policy`

Policy inheritance/merge.

---

# 341. `siar-enterprise-admin`

Delegated admin roles/capabilities.

---

# 342. `siar-tenant-quota`

Resource governance.

---

# 343. `siar-tenant-billing`

Aggregate tenant accounting.

---

# 344. `siar-tenant-federation`

Cross-org federation/sharing.

---

# 345. `siar-tenant-provisioning`

Create/suspend/delete/migrate tenants.

---

# 346. `siar-tenant-observability`

Privacy-safe tenant health metrics.

---

# 347. `siar-tenant-testkit`

Isolation/delegation/migration simulator.

---

# 348. Error Taxonomy

```rust
pub enum TenantError {
    TenantNotFound,
    ScopeMismatch,
    CrossTenantAccessDenied,
    PolicyConflict,
    AdminUnauthorized,
    DelegationInvalid,
    QuotaExceeded,
    PlacementUnsatisfied,
    MigrationFailed,
    DeletionBlocked,
    Internal,
}
```

---

# 349. Security & Privacy Invariants

Mandatory:

```text
1. Every tenant-scoped operation carries an explicit TenantId context.
2. No tenant can read, write, search, cache, decrypt, or administer another tenant's data without an explicit cross-tenant capability.
3. Tenant encryption/key roots are independent; compromise of one tenant does not decrypt another.
4. Enterprise administrators have scoped roles/capabilities; there is no implicit global superuser.
5. Admin delegation can only attenuate authority and is bounded by scope/time.
6. Organization policy cannot weaken network-level privacy/security floors.
7. BYOD managed policy cannot inspect or control unrelated personal-profile data.
8. Billing, audit, admin, and communication identities remain separate.
9. Quota/resource exhaustion in one tenant cannot silently degrade privacy or starve unrelated tenants.
10. Cross-tenant sharing is explicit, narrow, revocable, and does not merge namespaces.
11. Tenant deletion/migration/restore operations cannot mutate unrelated tenant state.
12. Shared infrastructure observability never becomes a cross-tenant user-behavior warehouse.
```

---

# 350. Initial Production Scope

Implement first:

```text
explicit TenantContext types
TenantId-prefixed storage/cache/search/blob keys
PostgreSQL RLS or schema isolation
tenant-scoped key hierarchy
tenant/org/team hierarchy
signed enterprise policy bundles
delegated admin capability model
BYOD/managed-profile separation
per-tenant quotas
aggregate billing/accounting
tenant provisioning state machine
offboarding
cross-tenant capability sharing
tenant audit isolation
tenant deletion/migration tests
```

Then add:

```text
dedicated tenant clusters
cross-org federated directory
managed-device attestation adapters
advanced tenant portability
tenant-specific residency placement engine
formal policy/delegation verification
```

---

# 351. Definition of Done

Part 69 is complete when:

- tenant, organization, team, user, device, admin, billing, and federation identities are distinct
- tenant context is explicit in every scoped request
- storage/cache/index/blob/queue layers enforce tenant boundaries
- tenant cryptographic roots are independent
- enterprise policy inheritance is versioned and restrictive
- delegated admin roles are scoped and attenuated
- BYOD and fully managed devices have different policy boundaries
- billing/admin/audit identities are separate from communication identity
- per-tenant quota and noisy-neighbor isolation are enforced
- cross-tenant sharing requires explicit capability
- org federation is explicit and non-transitive
- offboarding revokes managed access without affecting personal state
- tenant migration/deletion/backup/restore preserve isolation
- isolation, admin, quota, BYOD, federation, fuzz, and formal tests are specified

---

# 352. Final Architecture

```text
                      SHARED INFRASTRUCTURE
                              │
                              ▼
                     TENANT ISOLATION LAYER
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
       Tenant A            Tenant B            Tenant C
          │                   │                   │
      Org Policy          Org Policy          Org Policy
          │                   │                   │
      Scoped Admin        Scoped Admin        Scoped Admin
          │                   │                   │
          └────────────── explicit federation/sharing ───────┘
```

Multi-tenant safety model:

```text
explicit tenant context
+
independent key roots
+
storage/cache/search isolation
+
delegated least-privilege administration
+
quota/resource bulkheads
+
explicit cross-tenant capabilities
```

not:

```text
shared database rows plus a tenant_id column and hope every query remembers it
```

---

# 353. Final Principle

Multi-tenancy is not merely a billing feature.

It is a **security, privacy, cryptographic, administrative, and operational isolation model**.

The correct model is:

```text
shared infrastructure where useful
+
independent tenant authority
+
explicit policy boundaries
+
least-privilege administration
+
no implicit cross-tenant trust
```

This architecture lets SIAR serve enterprises, schools, NGOs, hosted customers, and managed organizations on common infrastructure while preserving the anonymity, identity, storage, and governance boundaries established across Parts 34–68.
