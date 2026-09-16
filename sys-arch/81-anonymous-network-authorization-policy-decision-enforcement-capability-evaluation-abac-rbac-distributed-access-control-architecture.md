# Core System Architecture Part 81 — Anonymous Network Authorization, Policy Decision/Enforcement Points, Capability Evaluation, ABAC/RBAC & Distributed Access-Control Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 81  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 28, 46, 53, 56, 61, 68–80  

**Primary purpose:** define SIAR's authorization and distributed access-control architecture, including capabilities, RBAC, ABAC, policy decision points, policy enforcement points, policy compilation, delegation, attenuation, tenant/federation scope, offline authorization, revocation, policy precedence, auditing, local evaluation, and type-safe Rust interfaces.

---

# 1. Purpose

Authentication answers:

```text
who are you?
```

Authorization answers:

```text
what are you allowed to do?
```

In a distributed anonymous network, authorization becomes difficult because authority is spread across:

```text
users
devices
services
tenants
organizations
federation domains
operators
plugins
workloads
```

The governing principle is:

> **SIAR authorization must be explicit, local where possible, scope-bounded, revocable, attenuating, privacy-preserving, and impossible to widen silently through delegation or policy composition.**

---

# 2. Architectural Position

```text
Identity / Capability / Context
            │
            ▼
     Policy Decision Point
            │
            ▼
     Authorization Decision
            │
            ▼
     Policy Enforcement Point
            │
            ▼
       Protected Action
```

---

# 3. Core Separation

Keep distinct:

```text
authentication
authorization
capability
role
attribute
policy
consent
governance
```

---

# 4. Non-Goals

Part 81 does not create:

```text
one global IAM database
one superuser role
one policy engine that every packet must contact
authorization based only on network location
authorization based only on identity name
```

---

# 5. Authorization Subject Classes

```rust
pub enum AuthorizationSubject {
    User(UserRef),
    Device(DeviceRef),
    Service(ServiceIdentity),
    Workload(WorkloadIdentity),
    Admin(EnterpriseAdminId),
    Plugin(PluginInstanceId),
    FederationPeer(FederationDomainId),
}
```

---

# 6. Subject Is Not Authority

Identity alone does not grant action.

---

# 7. Hard Rule

Authenticated identity without policy/capability does not imply authorization.

---

# 8. Resource Model

```rust
pub enum ProtectedResource {
    Conversation(ConversationId),
    Group(GroupId),
    Tenant(TenantId),
    Workspace(WorkspaceId),
    Service(ServiceIdentity),
    File(FileId),
    Administrative(AdminResourceId),
}
```

---

# 9. Resource Scope

Must be explicit.

---

# 10. Action Model

```rust
pub enum AuthorizationAction {
    Read,
    Write,
    Send,
    Delete,
    Share,
    Administer,
    Configure,
    Export,
    Subscribe,
    Execute,
}
```

---

# 11. Domain-Specific Actions

Preferred.

---

# 12. Example

```text
SendMessage
AddGroupMember
RotateKey
ReadAudit
```

---

# 13. Authorization Request

```rust
pub struct AuthorizationRequest {
    pub subject: AuthorizationSubject,
    pub action: ActionId,
    pub resource: ResourceRef,
    pub context: AuthorizationContext,
}
```

---

# 14. Authorization Context

```rust
pub struct AuthorizationContext {
    pub tenant: Option<TenantId>,
    pub organization: Option<OrganizationId>,
    pub federation: Option<FederationDomainId>,
    pub device_state: Option<DeviceTrustState>,
    pub privacy_mode: PrivacyRoutingMode,
}
```

---

# 15. No Hidden Ambient Context

Hard rule.

---

# 16. Authorization Decision

```rust
pub enum AuthorizationDecision {
    Allow(AuthorizationGrant),
    Deny(DenialReason),
    Indeterminate,
}
```

---

# 17. Default

`Deny`.

---

# 18. Indeterminate

Must not become Allow.

---

# 19. Hard Rule

Fail closed for protected actions.

---

# 20. Capability Model

Capabilities are first-class.

---

# 21. Capability

```rust
pub struct Capability {
    pub subject: CapabilitySubject,
    pub audience: CapabilityAudience,
    pub scope: CapabilityScope,
    pub actions: BTreeSet<ActionId>,
    pub expires_at: Option<Timestamp>,
    pub delegation: DelegationPolicy,
}
```

---

# 22. Capability Subject

Who may use it.

---

# 23. Audience

Who may accept it.

---

# 24. Scope

Which resources.

---

# 25. Actions

What operations.

---

# 26. Expiry

Time bound.

---

# 27. Delegation

Whether/how it can be attenuated.

---

# 28. Capability Principle

Possession does not mean unlimited delegation.

---

# 29. Delegation Policy

```rust
pub enum DelegationPolicy {
    Forbidden,
    AttenuationOnly,
    BoundedDepth(u8),
}
```

---

# 30. Attenuation

Child capability can only reduce:

```text
actions
scope
lifetime
audience
```

---

# 31. Hard Rule

Delegation can never widen authority.

---

# 32. Delegation Chain

```rust
pub struct DelegationChain {
    pub grants: Vec<CapabilityGrant>,
}
```

---

# 33. Chain Validation

Every link verified.

---

# 34. Effective Capability

Intersection of all links.

---

# 35. No Union Across Chain

Hard rule.

---

# 36. Maximum Delegation Depth

Bounded.

---

# 37. Role-Based Access Control

Useful for organizational administration.

---

# 38. RBAC Roles

Examples:

```text
WorkspaceAdmin
SecurityAdmin
BillingAdmin
Auditor
```

---

# 39. Role ≠ Permission

Role compiles to capability grants.

---

# 40. Hard Rule

Domain code should enforce actions/scopes, not role names.

---

# 41. Why

Prevents role sprawl from leaking into business logic.

---

# 42. Role Definition

```rust
pub struct RoleDefinition {
    pub role: RoleId,
    pub grants: Vec<CapabilityTemplate>,
}
```

---

# 43. Role Assignment

```rust
pub struct RoleAssignment {
    pub subject: AuthorizationSubject,
    pub role: RoleId,
    pub scope: RoleScope,
}
```

---

# 44. RBAC Scope

Explicit.

---

# 45. No Global Admin Role By Default

Hard rule.

---

# 46. Attribute-Based Access Control

Useful for contextual rules.

---

# 47. ABAC Attributes

Examples:

```text
tenant
device posture
region
managed/unmanaged
data classification
time window
```

---

# 48. ABAC Attribute Type

```rust
pub enum AuthorizationAttribute {
    Tenant(TenantId),
    Region(RegionId),
    DeviceTrust(DeviceTrustState),
    DataClass(DataClassification),
    ManagedProfile(bool),
}
```

---

# 49. No Arbitrary String Attributes

Preferred hard rule.

---

# 50. Why

Typed attributes reduce policy ambiguity.

---

# 51. Attribute Source

Must be authoritative.

---

# 52. User-Supplied Attribute

Never trusted directly.

---

# 53. Hard Rule

Authorization attributes require trusted provenance.

---

# 54. Attribute Provenance

```rust
pub struct TrustedAttribute<T> {
    pub value: T,
    pub issuer: AttributeIssuer,
    pub valid_until: Option<Timestamp>,
}
```

---

# 55. Policy Types

```rust
pub enum PolicyType {
    Capability,
    Role,
    Attribute,
    Governance,
    Tenant,
    UserPreference,
}
```

---

# 56. Policy Precedence

Must integrate Parts 53/56/61/69.

---

# 57. Recommended Precedence

```text
compile-time hard invariant
> network governance/security floor
> legal restriction
> tenant/org managed policy
> explicit capability scope
> user stronger privacy restriction
> optimization/default
```

---

# 58. Important

Capability cannot override higher security/legal floor.

---

# 59. User Stronger Restriction

Can reduce authority.

---

# 60. Cannot increase beyond capability/policy.

---

# 61. Hard Rule

Policy composition is monotonic toward restriction.

---

# 62. Policy Decision Point

PDP evaluates authorization.

---

# 63. PDP Trait

```rust
pub trait PolicyDecisionPoint {
    fn evaluate(
        &self,
        request: &AuthorizationRequest,
    ) -> Result<AuthorizationDecision, AuthorizationError>;
}
```

---

# 64. Local PDP

Preferred.

---

# 65. Why

```text
low latency
partition tolerance
privacy
```

---

# 66. Central PDP

Useful for admin/control-plane policy compilation.

---

# 67. But

No per-request central authorization hot path.

---

# 68. Hard Rule

Protected data-plane requests should not require live central PDP availability.

---

# 69. Policy Enforcement Point

PEP enforces decision.

---

# 70. PEP Examples

```text
API boundary
service RPC
repository layer
plugin broker
tenant admin service
```

---

# 71. Multiple PEPs

Defense in depth.

---

# 72. But

Avoid contradictory policy implementations.

---

# 73. Canonical Policy Model

Single typed policy representation.

---

# 74. PEP Uses Compiled Policy

---

# 75. Policy Compiler

```rust
pub trait PolicyCompiler {
    fn compile(
        &self,
        source: &PolicySourceBundle,
    ) -> Result<CompiledPolicy, PolicyError>;
}
```

---

# 76. Compiled Policy

Immutable/versioned.

---

# 77. Policy Version

Monotonic.

---

# 78. Anti-Rollback

Required.

---

# 79. No Local Policy Mutation

Hard rule.

---

# 80. Policy Bundle

```rust
pub struct AuthorizationPolicyBundle {
    pub version: PolicyVersion,
    pub scope: PolicyScope,
    pub rules: Vec<PolicyRule>,
    pub signatures: PolicySignatureBundle,
}
```

---

# 81. Policy Scope

```rust
pub enum PolicyScope {
    GlobalSecurityFloor,
    Tenant(TenantId),
    Organization(OrganizationId),
    Workspace(WorkspaceId),
    Service(ServiceIdentity),
}
```

---

# 82. Scope Matching

Exact.

---

# 83. No Implicit Parent Scope Expansion

Hard rule.

---

# 84. Policy Rule

Typed.

---

# 85. Example

```rust
pub struct PolicyRule {
    pub subject_selector: SubjectSelector,
    pub action: ActionId,
    pub resource_selector: ResourceSelector,
    pub effect: PolicyEffect,
    pub conditions: Vec<PolicyCondition>,
}
```

---

# 86. Policy Effect

```rust
pub enum PolicyEffect {
    Allow,
    Deny,
}
```

---

# 87. Explicit Deny

Overrides allow where same/higher precedence applies.

---

# 88. Default Deny

Always.

---

# 89. No "First Match Wins" Ambiguity

Preferred hard rule.

---

# 90. Deterministic Evaluation

Required.

---

# 91. Policy Conflict

Return deny or explicit conflict state.

---

# 92. No Random Resolution.

---

# 93. Attribute Conditions

Typed.

---

# 94. Example

```text
DeviceTrust >= Managed
Region ∈ allowed_regions
DataClass != Restricted
```

---

# 95. No Policy Code Execution

Hard rule.

---

# 96. Why

Policies must be declarative.

---

# 97. Policy Language

Internal typed AST.

---

# 98. RON

Can represent human-reviewed policy config.

---

# 99. Postcard

Compiled binary form.

---

# 100. JSON

External interop only.

---

# 101. No Embedded Scripting Language In Initial Scope

Hard rule.

---

# 102. Policy Testing

Every policy bundle has tests.

---

# 103. Example Cases

```text
expected allow
expected deny
scope mismatch
revoked capability
```

---

# 104. Policy Static Analysis

Detect:

```text
unreachable rule
wildcard scope
role cycle
delegation amplification
conflicting grants
```

---

# 105. No Wildcard Resource In High-Risk Admin Policy

Hard rule.

---

# 106. Capability Evaluation

First validate:

```text
signature
issuer
audience
expiry
revocation
delegation chain
scope
action
```

---

# 107. Validation Order

Security-sensitive.

---

# 108. Invalid Signature

Immediate deny.

---

# 109. Wrong Audience

Deny.

---

# 110. Expired

Deny.

---

# 111. Revoked

Deny.

---

# 112. Scope mismatch

Deny.

---

# 113. No Partial Accept

Hard rule.

---

# 114. Capability Revocation

Needed.

---

# 115. Revocation Record

```rust
pub struct CapabilityRevocation {
    pub capability_id: CapabilityId,
    pub effective_at: Timestamp,
}
```

---

# 116. Revocation Feed

Signed.

---

# 117. Cached Capability

Revocation overrides.

---

# 118. Hard Rule

Stale cache cannot resurrect revoked authority.

---

# 119. Capability IDs

Opaque/random.

---

# 120. No User Identity Encoding.

---

# 121. Capability Storage

Local where possible.

---

# 122. Bearer Capability

Risky.

---

# 123. Prefer bound capability.

---

# 124. Binding

```text
subject
audience
device/workload
```

---

# 125. Transferable Capability

Only explicit use case.

---

# 126. No Unbounded Bearer Token

Hard rule.

---

# 127. Offline Authorization

Important.

---

# 128. Device may be offline.

---

# 129. Use cached signed policy/capabilities.

---

# 130. Within validity.

---

# 131. No infinite offline trust.

---

# 132. Offline Grace

Policy-defined.

---

# 133. Revocation Catch-Up

Required on reconnect.

---

# 134. Sensitive Action Offline

Can require fresh authority.

---

# 135. Example

```text
tenant policy change
admin role grant
```

---

# 136. Hard Rule

Offline mode cannot create new higher authority without proper signed delegation.

---

# 137. User-to-User Authorization

Capabilities/relationship model.

---

# 138. Examples

```text
send message
share file
invite to group
```

---

# 139. No Global ACL Service

Hard rule.

---

# 140. Relationship capability

Local/peer-scoped.

---

# 141. Group Authorization

Group epoch/membership state.

---

# 142. Membership Change

Rotates keys where required.

---

# 143. Admin Authorization

Part 69.

---

# 144. Role-to-capability compilation.

---

# 145. Platform Operator

Separate authority domain.

---

# 146. Platform operator ≠ tenant admin.

---

# 147. Hard rule.

---

# 148. Plugin Authorization

Part 68.

---

# 149. Plugin capability scope.

---

# 150. No plugin inherits host app authority.

---

# 151. Hard rule.

---

# 152. Service-to-Service Authorization

Part 79.

---

# 153. mTLS identity + capability/policy.

---

# 154. Workload identity alone insufficient.

---

# 155. Secret Authorization

Part 80.

---

# 156. Secret access is action/scope-specific.

---

# 157. No generic read-any-secret role.

---

# 158. Event Bus Authorization

Part 75.

---

# 159. Topic publish/subscribe capabilities.

---

# 160. Database Authorization

Part 74.

---

# 161. Repository/service identity + DB least privilege.

---

# 162. No application DB owner role.

---

# 163. Edge Authorization

Part 78.

---

# 164. Edge performs coarse auth only.

---

# 165. Domain PEP rechecks.

---

# 166. Federation Authorization

Part 58.

---

# 167. Federation peer only gets explicitly peered service scope.

---

# 168. No transitive authorization.

---

# 169. Hard rule.

---

# 170. Tenant/Federation Scope

Must be embedded in trusted context.

---

# 171. Header alone not enough.

---

# 172. Capability must bind scope.

---

# 173. Multi-Tenant Role

Tenant scoped.

---

# 174. Same human can hold roles in multiple tenants.

---

# 175. But

Assignments are separate.

---

# 176. No role union across tenants.

---

# 177. Hard rule.

---

# 178. Cross-Tenant Share

Explicit capability.

---

# 179. No merged ACL namespace.

---

# 180. Enterprise Policy

Part 69.

---

# 181. Can restrict managed workspace.

---

# 182. Cannot alter personal workspace authority.

---

# 183. User Consent

Part 56.

---

# 184. Consent is not authorization grant.

---

# 185. Important Distinction

Consent may be required in addition to authorization.

---

# 186. Hard Rule

Authorization does not imply user consent.

---

# 187. Conversely

Consent does not create authority.

---

# 188. Data Classification

ABAC input.

---

# 189. Example

Restricted data cannot export externally.

---

# 190. Export Capability

Must also satisfy classification policy.

---

# 191. No capability override of data-classification floor.

---

# 192. Device Trust

ABAC input.

---

# 193. Managed workspace may require:

```text
trusted device
screen lock
recent attestation
```

---

# 194. But

Do not apply to personal context.

---

# 195. Hard rule.

---

# 196. Region/Jurisdiction

ABAC input.

---

# 197. Data residency rules.

---

# 198. No role can override forbidden region unless legal policy explicitly allows.

---

# 199. Policy Decision Cache

Local.

---

# 200. Cache Key

```rust
pub struct DecisionCacheKey {
    pub subject: SubjectDigest,
    pub action: ActionId,
    pub resource: ResourceDigest,
    pub policy_version: PolicyVersion,
}
```

---

# 201. No Raw User Identity In Shared Cache Key

Hard rule.

---

# 202. Cache TTL

Short.

---

# 203. Revocation invalidates.

---

# 204. Policy version change invalidates.

---

# 205. No Long-Lived Allow Cache

Hard rule.

---

# 206. Deny Cache

Can be short-lived.

---

# 207. Decision Proof

Optional.

---

# 208. Authorization Proof

```rust
pub struct AuthorizationGrant {
    pub decision_id: DecisionId,
    pub policy_version: PolicyVersion,
    pub scope: CapabilityScope,
    pub expires_at: Option<Timestamp>,
}
```

---

# 209. Grant Is Not Transferable Unless Explicit.

---

# 210. No Reuse Across Audience.

---

# 211. PDP Availability

Local PDP should operate from signed policy.

---

# 212. Central policy compiler can be unavailable.

---

# 213. Last valid policy within validity continues.

---

# 214. No permissive fallback.

---

# 215. Hard rule.

---

# 216. Emergency Policy

Can only restrict or revoke quickly.

---

# 217. Emergency allow-expansion

High-risk, multi-party approval.

---

# 218. But

Cannot violate hard security floor.

---

# 219. Hard rule.

---

# 220. Policy Distribution

Push hint + pull signed bundle.

---

# 221. Anti-rollback.

---

# 222. Partial Policy Update

Avoid.

---

# 223. Bundle applied atomically.

---

# 224. Hard rule.

---

# 225. Policy Validation

Before activation.

---

# 226. Activation State

```rust
pub enum PolicyActivationState {
    Draft,
    Signed,
    Staged,
    Active,
    Superseded,
    Revoked,
}
```

---

# 227. No Unsigned Active Policy.

---

# 228. Audit

Authorization audit must be selective.

---

# 229. Audit High-Risk Actions

Examples:

```text
admin grants
tenant policy changes
secret access
cross-tenant export
```

---

# 230. Do Not Audit Every User Message Authorization

Hard rule.

---

# 231. Why

Would create metadata surveillance.

---

# 232. Audit Record

```rust
pub struct AuthorizationAuditRecord {
    pub actor_class: AuthorizationSubjectClass,
    pub action: ActionId,
    pub resource_class: ResourceClass,
    pub outcome: AuditOutcome,
    pub policy_version: PolicyVersion,
}
```

---

# 233. No Message Content.

---

# 234. No Contact Graph.

---

# 235. No Long-Lived User Identifier Unless legally/operationally required for managed admin action.

---

# 236. Privacy-Safe Metrics

```text
allow count by service/action class
deny count
policy cache hit
revocation lag
```

---

# 237. Forbidden Metrics

No:

```text
user-to-resource graph
conversation-level auth timeline
```

---

# 238. Authorization SLO

Examples:

```text
policy propagation
revocation propagation
decision latency
```

---

# 239. Security SLO

```text
0 stale revoked capabilities accepted
```

---

# 240. Privacy SLO

```text
0 global user correlation identifiers in PDP/PEP telemetry
```

---

# 241. Denial Semantics

Do not reveal unnecessary existence.

---

# 242. Example

Unauthorized lookup may return generic not-found.

---

# 243. Prevent enumeration.

---

# 244. Hard rule where resource privacy requires.

---

# 245. Error Taxonomy

```rust
pub enum AuthorizationError {
    Unauthenticated,
    Unauthorized,
    CapabilityInvalid,
    CapabilityExpired,
    CapabilityRevoked,
    AudienceMismatch,
    ScopeMismatch,
    PolicyConflict,
    PolicyExpired,
    AttributeUntrusted,
    Internal,
}
```

---

# 246. External Error Mapping

Minimal.

---

# 247. Internal diagnostics richer.

---

# 248. No Policy Internals Leaked To Attacker.

---

# 249. Policy-as-Code?

Use declarative typed config.

---

# 250. Avoid general-purpose scripting.

---

# 251. Why

Determinism, auditability, static analysis.

---

# 252. Policy AST

```rust
pub enum PolicyExpr {
    All(Vec<PolicyExpr>),
    Any(Vec<PolicyExpr>),
    Not(Box<PolicyExpr>),
    HasAction(ActionId),
    InScope(CapabilityScope),
    AttributeEquals(AttributeKey, AttributeValue),
}
```

---

# 253. Bounds

Expression depth/size bounded.

---

# 254. No Recursive Unbounded Policy.

---

# 255. Compile to bytecode/tree.

---

# 256. Deterministic evaluation.

---

# 257. No dynamic I/O during decision.

---

# 258. Hard rule.

---

# 259. External Attribute Fetch

Preload/trust before evaluation.

---

# 260. No network call in policy hot path.

---

# 261. Policy Complexity Limit

Required.

---

# 262. Prevent DoS.

---

# 263. Decision Budget

```rust
pub struct DecisionBudget {
    pub max_rules: usize,
    pub max_depth: usize,
}
```

---

# 264. No unbounded authorization complexity.

---

# 265. Authorization Layering

Recommended:

```text
identity validation
→ capability validation
→ hard policy floor
→ tenant/org policy
→ resource-specific rules
→ user stronger restriction
```

---

# 266. Performance Optimizations

After correctness.

---

# 267. Rust Type-Level Safety

Strongly recommended.

---

# 268. Typed Capability

```rust
pub struct CapabilityFor<A, R> {
    pub action: PhantomData<A>,
    pub resource: R,
    pub raw: Capability,
}
```

---

# 269. Example

```rust
CapabilityFor<SendMessage, ConversationRef>
```

---

# 270. Prevent passing file-share capability to admin function.

---

# 271. Newtypes

Use for every authority domain.

---

# 272. No generic `String` role/action/resource IDs in domain APIs.

---

# 273. Compile-Time Scope

Where possible.

---

# 274. Runtime validation still required.

---

# 275. Repository Guard

```rust
pub trait AuthorizedRepository<T> {
    fn get(
        &self,
        auth: &AuthorizationGrant,
        id: T::Id,
    ) -> Result<Option<T>, StorageError>;
}
```

---

# 276. No Unauthenticated Repository Path In High-Risk Domain

Preferred hard rule.

---

# 277. Command Handler Guard

```rust
pub trait AuthorizedCommandHandler<C> {
    fn handle(
        &self,
        auth: &AuthorizationGrant,
        command: C,
    ) -> Result<(), CommandError>;
}
```

---

# 278. No "remember to check auth" convention.

---

# 279. Type/interface forces it.

---

# 280. Background Job Authorization

Important.

---

# 281. Job must carry authorization context or derive service authority.

---

# 282. No replay of stale user capability indefinitely.

---

# 283. Long Workflow

Use capability snapshot/version or reauthorize step.

---

# 284. Sensitive long workflow

Reauthorize before high-risk step.

---

# 285. Hard rule.

---

# 286. Workflow Delegation

Service acts on behalf of user.

---

# 287. Use audience-bound attenuated capability.

---

# 288. No copying raw user session token into queue.

---

# 289. Hard rule.

---

# 290. Queue/Event Bus

Store capability reference/digest where possible.

---

# 291. Secret capability payload encrypted.

---

# 292. Expiry checked at execution.

---

# 293. Reconciliation

Detect orphaned grants.

---

# 294. Examples:

```text
role assignment to deleted user
capability to deleted resource
plugin grant to removed plugin
```

---

# 295. Reconciliation Job

Idempotent.

---

# 296. No stale orphan authority.

---

# 297. Access Review

Managed tenants may need periodic review.

---

# 298. Review only org-managed authority.

---

# 299. No personal social graph review.

---

# 300. Hard rule.

---

# 301. Role Explosion

Avoid.

---

# 302. Prefer:

```text
small stable roles
+
capability scopes
+
attributes
```

---

# 303. ABAC Explosion

Also avoid.

---

# 304. Keep attribute vocabulary controlled.

---

# 305. Authorization Model Summary

Use:

```text
capabilities for delegation/resource authority
RBAC for admin grouping
ABAC for contextual constraints
hard policy floor for security/privacy
```

---

# 306. No One Model Solves Everything

Hard truth.

---

# 307. Testing

Need authorization testkit.

---

# 308. Test Scenarios

```text
wrong audience
expired capability
revoked capability
delegation amplification
wrong tenant
```

---

# 309. Audience Test

Capability rejected by wrong service.

---

# 310. Scope Test

Cannot access sibling resource.

---

# 311. Delegation Test

Child never broader.

---

# 312. Revocation Test

Stale cache cannot allow.

---

# 313. Policy Conflict Test

Deterministic deny.

---

# 314. ABAC Test

Untrusted attribute rejected.

---

# 315. RBAC Test

Role grants only compiled capabilities.

---

# 316. Tenant Test

No cross-tenant role union.

---

# 317. Federation Test

No transitive authorization.

---

# 318. Plugin Test

Plugin cannot inherit app authority.

---

# 319. Service Test

mTLS identity alone insufficient.

---

# 320. Secret Test

Wrong use-purpose denied.

---

# 321. Offline Test

Expired cached authority denied.

---

# 322. Workflow Test

High-risk step reauthorization.

---

# 323. Privacy Test

No user graph emitted in audit.

---

# 324. Policy Rollback Test

Old permissive policy rejected.

---

# 325. Fuzzing

Fuzz:

```text
capability token
policy bundle
delegation chain
attribute set
```

---

# 326. Property Tests

Properties:

```text
delegated authority is subset of parent
revoked capability can never authorize
higher-precedence deny cannot be overridden by lower allow
tenant-scoped grant cannot authorize resource in another tenant
```

---

# 327. Formal Verification Targets

Strong candidates:

```text
capability attenuation
policy precedence
role-to-capability compilation
revocation
```

---

# 328. Kani Candidate

scope/action subset lattice.

---

# 329. TLA+ Candidate

revocation + offline cache + policy update.

---

# 330. Loom Candidate

concurrent policy refresh + decision cache.

---

# 331. Performance

Authorization is hot path.

---

# 332. Decision target

Microseconds to low milliseconds locally.

---

# 333. No remote PDP request.

---

# 334. Precompile policies.

---

# 335. Use compact enums/newtypes.

---

# 336. Decision cache.

---

# 337. Cache key includes policy version.

---

# 338. Revocation invalidation.

---

# 339. No unbounded policy AST.

---

# 340. Memory

Bounded compiled policy.

---

# 341. No per-user global graph in memory.

---

# 342. Crate Layout

Recommended:

```text
crates/
├── siar-authz-core/
├── siar-capability/
├── siar-rbac/
├── siar-abac/
├── siar-policy-model/
├── siar-policy-compiler/
├── siar-pdp/
├── siar-pep/
├── siar-authz-revocation/
├── siar-authz-observability/
└── siar-authz-testkit/
```

---

# 343. `siar-authz-core`

Owns:

```text
subjects
resources
actions
decisions
errors
```

---

# 344. `siar-capability`

Capability/delegation/attenuation.

---

# 345. `siar-rbac`

Role definitions/assignments.

---

# 346. `siar-abac`

Trusted attributes/conditions.

---

# 347. `siar-policy-model`

Typed policy AST.

---

# 348. `siar-policy-compiler`

Static analysis/compiled representation.

---

# 349. `siar-pdp`

Local policy evaluation.

---

# 350. `siar-pep`

API/service/repository enforcement helpers.

---

# 351. `siar-authz-revocation`

Capability/policy revocation state.

---

# 352. `siar-authz-observability`

Privacy-safe metrics/audit.

---

# 353. `siar-authz-testkit`

Scope/delegation/revocation/policy simulation.

---

# 354. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Authentication never implies authorization.
2. Default authorization result is deny; indeterminate never becomes allow.
3. Capability delegation can only attenuate authority and can never widen action, scope, audience, or lifetime.
4. RBAC roles compile into explicit scoped capabilities; domain code does not trust role names directly.
5. ABAC attributes are typed and accepted only from trusted issuers.
6. Policy composition is monotonic toward restriction and lower-precedence rules cannot weaken higher security/privacy/legal floors.
7. Revocation and policy updates override cached authorization state.
8. Tenant, organization, federation, service, plugin, and personal authority scopes remain independent and cannot merge implicitly.
9. Policy decision is local and deterministic; no external network I/O occurs in the hot authorization path.
10. Authorization audits avoid message content, contact graphs, and universal user correlation identifiers.
11. User consent and authorization remain separate requirements; neither substitutes for the other.
12. High-risk domain APIs require typed authorization/capability inputs so missing authorization checks are difficult to represent in code.
```

---

# 355. Initial Production Scope

Implement first:

```text
typed subject/resource/action model
capability format with audience/scope/action/expiry
delegation attenuation
role-to-capability compiler
trusted typed ABAC attributes
signed policy bundles
local PDP
API/service/repository PEP helpers
revocation feed
policy anti-rollback
decision cache with version/revocation invalidation
tenant/federation/plugin/service scope enforcement
privacy-safe authorization metrics
authorization testkit
```

Then add:

```text
formal policy verifier
capability proof compression
advanced offline authorization profiles
managed access-review workflows
policy explainability tooling
optional standards interop adapters
```

---

# 356. Definition of Done

Part 81 is complete when:

- authentication and authorization are separate everywhere
- capabilities are audience/scope/action/time bound
- delegated capabilities attenuate only
- RBAC compiles to capabilities
- ABAC uses trusted typed attributes
- PDP/PEP boundaries are explicit
- local deterministic authorization works offline within policy validity
- policy precedence and anti-rollback are enforced
- revocation invalidates cached authority
- tenant/federation/plugin/service/personal scopes remain isolated
- high-risk APIs structurally require authorization inputs
- audits/metrics remain privacy-safe
- scope/delegation/revocation/tenant/fuzz/formal tests are specified

---

# 357. Final Architecture

```text
            AUTHENTICATED SUBJECT
                     │
                     ▼
          CAPABILITY / ROLE / ATTRIBUTES
                     │
                     ▼
             LOCAL POLICY DECISION
                     │
                     ▼
          ALLOW / DENY / INDETERMINATE
                     │
                     ▼
             POLICY ENFORCEMENT
                     │
                     ▼
              PROTECTED ACTION
```

Distributed authorization safety model:

```text
typed capability
+
RBAC for grouping
+
ABAC for context
+
local PDP
+
explicit PEP
+
revocation
+
scope isolation
```

not:

```text
the caller is logged in and inside the network, therefore allow it
```

---

# 358. Final Principle

Authorization should express **who may do exactly what, to exactly which resource, under exactly which constraints**, and nothing more.

The correct model is:

```text
authenticate narrowly
+
grant explicitly
+
attenuate delegation
+
evaluate locally
+
enforce everywhere
+
revoke quickly
+
audit minimally
```

This architecture gives SIAR a distributed authorization foundation for users, services, workloads, plugins, tenants, federation peers, administrators, secret brokers, databases, queues, storage, and APIs while preserving the anonymity and least-authority guarantees established across Parts 34–80.
