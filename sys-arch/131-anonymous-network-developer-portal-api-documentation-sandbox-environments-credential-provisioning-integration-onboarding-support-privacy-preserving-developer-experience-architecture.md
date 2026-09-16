# Core System Architecture Part 131 — Anonymous Network Developer Portal, API Documentation, Sandbox Environments, Credential Provisioning, Integration Onboarding, Support & Privacy-Preserving Developer Experience Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 131  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 68, 79–81, 122–130

**Primary purpose:** define SIAR's developer-experience architecture for developer portals, documentation, sandbox environments, credential provisioning, integration onboarding, test tenants, sample projects, support workflows, self-service diagnostics, quota visibility, incident/status communication, and privacy-preserving developer operations.

---

# 1. Purpose

A strong SDK is not enough if developers cannot safely understand, test, onboard, debug, and certify integrations.

A mature developer experience must answer:

```text
Where is the canonical documentation?
How can developers experiment safely?
How are sandbox credentials issued?
How are scopes reviewed?
How does an integration move from development to production?
How can developers diagnose errors without seeing private internals?
How are support and incident communications handled?
How do we prevent the portal itself from becoming a metadata-surveillance system?
```

The governing principle is:

> **SIAR developer experience should minimize setup friction while preserving strict separation between documentation, sandbox, certification, credentials, support, and production authority.**

---

# 2. Architectural Position

```text
                    DEVELOPER
                        │
                        ▼
                DEVELOPER PORTAL
                        │
        ┌───────────────┼────────────────┐
        │               │                │
       DOCS          SANDBOX         ONBOARDING
        │               │                │
        └───────────────┼────────────────┘
                        ▼
                INTEGRATION PROJECT
                        │
                        ▼
               CONFORMANCE / CERT
                        │
                        ▼
                 PRODUCTION ACCESS
```

---

# 3. Core Separation

Keep distinct:

```text
developer account
organization
integration identity
sandbox credential
production credential
SDK package
certification
support access
```

---

# 4. Non-Goals

Part 131 does not create:

```text
one-click production superkeys
shared credentials
production-data sandboxes
developer behavior profiling
support backdoors
undocumented APIs
```

---

# 5. Developer Portal Identity

```rust
pub struct DeveloperPortalAccountId(pub [u8; 16]);
```

---

# 6. Organization Identity

```rust
pub struct DeveloperOrganizationId(pub [u8; 16]);
```

---

# 7. Integration Project Identity

```rust
pub struct DeveloperProjectId(pub [u8; 16]);
```

---

# 8. Hard Rule

Portal account identity does not automatically imply runtime integration authority.

---

# 9. Developer Project

```rust
pub struct DeveloperProject {
    pub id: DeveloperProjectId,
    pub organization: DeveloperOrganizationId,
    pub integration: IntegrationId,
    pub state: DeveloperProjectState,
}
```

---

# 10. Project State

```rust
pub enum DeveloperProjectState {
    Draft,
    SandboxReady,
    Integrating,
    ConformanceTesting,
    CertificationReview,
    ProductionEligible,
    Suspended,
    Retired,
}
```

---

# 11. No Draft→ProductionEligible Direct

Hard rule.

---

# 12. Portal Architecture

Recommended:

```text
portal UI
→ developer application service
→ developer domain
→ SDK/integration registries
→ credential broker
→ sandbox control
→ certification/status/support adapters
```

---

# 13. Core Independent Of Portal UI

Hard rule.

---

# 14. Developer Portal UI

Could use Dioxus/Leptos as project architecture permits.

UI never directly owns credential authority.

---

# 15. Hard rule.

---

# 16. Documentation Architecture

Documentation is a product surface.

---

# 17. Documentation Classes

```rust
pub enum DeveloperDocClass {
    Quickstart,
    Concept,
    ApiReference,
    SdkReference,
    Tutorial,
    SecurityGuide,
    PrivacyGuide,
    MigrationGuide,
    Troubleshooting,
    Changelog,
}
```

---

# 18. Canonical Docs

Docs-as-code.

---

# 19. Hard rule.

---

# 20. Documentation Source

Recommended:

```text
Markdown
+
RON metadata
+
generated API reference
```

---

# 21. JSON only where external tooling requires it.

---

# 22. Hard rule.

---

# 23. Documentation Versioning

Docs pin:

```text
SDK version
API version
protocol version
```

---

# 24. Hard rule.

---

# 25. Version Selector

Portal allows viewing docs for supported historical versions.

---

# 26. Hard rule.

---

# 27. No "Latest Docs" For Old SDK Without Warning

Hard rule.

---

# 28. Generated API Reference

Generated from canonical SDK/IDL contract.

---

# 29. Hard rule.

---

# 30. Documentation Drift Detection

Part 123.

Check docs against:

```text
SDK symbols
IDL
compatibility manifest
examples
```

---

# 31. Hard rule.

---

# 32. Executable Examples

Code snippets should compile/test in CI.

---

# 33. Hard rule.

---

# 34. Example Project

```rust
pub struct ExampleProject {
    pub id: ExampleProjectId,
    pub sdk_version: SdkVersion,
    pub language: ExampleLanguage,
    pub source_revision: RevisionId,
}
```

---

# 35. Example Languages

```rust
pub enum ExampleLanguage {
    Rust,
    Kotlin,
    Swift,
    Java,
    CSharp,
    JavaScript,
    Python,
}
```

---

# 36. Only ship languages actually supported.

---

# 37. Hard rule.

---

# 38. Quickstart Principle

First successful sandbox call should require minimal steps.

---

# 39. But No Security Shortcuts

Hard rule.

---

# 40. Interactive API Explorer

Allowed for sandbox only by default.

---

# 41. Hard rule.

---

# 42. API Explorer Request

```rust
pub struct ExplorerRequest {
    pub project: DeveloperProjectId,
    pub endpoint: ApiEndpointId,
    pub sandbox_credential: SandboxCredentialRef,
}
```

---

# 43. No Production Credential Auto-Fill

Hard rule.

---

# 44. No Secret Echo

Portal never displays full credential after initial creation.

---

# 45. Hard rule.

---

# 46. SDK Playground

Could run:

```text
browser-based static examples
local generated commands
sandbox requests
```

---

# 47. No production private data.

---

# 48. Hard rule.

---

# 49. Sandbox Environment

Sandbox is isolated from production.

---

# 50. Sandbox Identity

```rust
pub struct SandboxEnvironmentId(pub [u8; 16]);
```

---

# 51. Sandbox Class

```rust
pub enum SandboxClass {
    SharedSynthetic,
    DedicatedProject,
    Certification,
}
```

---

# 52. Hard rule.

---

# 53. Sandbox Data

Synthetic by default.

---

# 54. Hard rule.

---

# 55. No Production User Data Copy

Hard rule.

---

# 56. Test Tenants

Sandbox can create synthetic tenants/accounts/devices.

---

# 57. Hard rule.

---

# 58. Synthetic Identity

Marked explicitly.

---

# 59. Hard rule.

---

# 60. Sandbox Isolation

Separate:

```text
credentials
databases
queues
object store
network policy
```

---

# 61. Hard rule.

---

# 62. Sandbox Network Egress

Restricted.

---

# 63. Hard rule.

---

# 64. Sandbox Reset

Self-service reset.

---

# 65. Hard rule.

---

# 66. Reset Does Not Affect Production

Hard rule.

---

# 67. Sandbox Expiry

Unused sandbox resources may expire.

---

# 68. Hard rule.

---

# 69. Sandbox Retention

Shorter retention than production where possible.

---

# 70. Hard rule.

---

# 71. Sandbox Quotas

```rust
pub struct SandboxQuota {
    pub request_rate: u32,
    pub storage_bytes: u64,
    pub attachment_bytes: u64,
    pub event_subscriptions: u32,
}
```

---

# 72. Hard rule.

---

# 73. No Hidden Quota

Developers can inspect current limits.

---

# 74. Hard rule.

---

# 75. Quota API

```rust
pub trait DeveloperQuotaService {
    fn quota(
        &self,
        project: DeveloperProjectId,
    ) -> Result<DeveloperQuotaSnapshot, DeveloperPortalError>;
}
```

---

# 76. Hard rule.

---

# 77. Credential Provisioning

Portal requests credentials from credential broker.

---

# 78. Portal Itself Does Not Mint Secrets

Hard rule.

---

# 79. Credential Class

```rust
pub enum DeveloperCredentialClass {
    Sandbox,
    Certification,
    Production,
}
```

---

# 80. Separate Classes

Hard rule.

---

# 81. Credential Lifecycle

```rust
pub enum DeveloperCredentialState {
    Pending,
    Active,
    Rotating,
    Revoked,
    Expired,
}
```

---

# 82. Hard rule.

---

# 83. Credential Scope

Inherited from integration manifest/certification.

---

# 84. Hard rule.

---

# 85. Sandbox Credential

Cannot call production endpoints.

---

# 86. Hard rule.

---

# 87. Production Credential

Issued only after production eligibility.

---

# 88. Hard rule.

---

# 89. Credential Display

Show secret once where unavoidable.

---

# 90. Prefer download/secret-manager handoff.

---

# 91. Hard rule.

---

# 92. Credential Hashing

Do not store retrievable plaintext unless architecture absolutely requires.

---

# 93. Hard rule.

---

# 94. Credential Rotation

Self-service for authorized organization admins.

---

# 95. Hard rule.

---

# 96. Overlap Window

Old/new credential can overlap briefly during rotation.

---

# 97. Hard rule.

---

# 98. Credential Revocation

Immediate.

---

# 99. Hard rule.

---

# 100. Compromise Workflow

```text
revoke
issue replacement
review logs
re-certify if scope/integration changed
```

---

# 101. Hard rule.

---

# 102. Secret Delivery

Possible integrations:

```text
download-once
vault/secret manager
mTLS workload enrollment
```

---

# 103. Hard rule.

---

# 104. No Emailing Secrets

Hard rule.

---

# 105. Project Onboarding

Stateful workflow.

---

# 106. Onboarding State

```rust
pub enum IntegrationOnboardingState {
    Created,
    ManifestSubmitted,
    SandboxProvisioned,
    QuickstartComplete,
    ConformanceReady,
    CertificationSubmitted,
    Approved,
    ProductionProvisioned,
}
```

---

# 107. Hard rule.

---

# 108. Onboarding Checklist

Generated from integration class.

---

# 109. Hard rule.

---

# 110. Integration Manifest

Part 130.

Portal visualizes capability requests.

---

# 111. Hard rule.

---

# 112. Capability Diff

When manifest changes:

```text
new scopes
removed scopes
data-class expansion
network/file-access changes
```

---

# 113. Hard rule.

---

# 114. Scope Expansion

Triggers:

```text
review
re-consent
re-certification
```

as applicable.

---

# 115. Hard rule.

---

# 116. Scope Reduction

Can often apply immediately.

---

# 117. Hard rule.

---

# 118. Onboarding Guidance

Portal explains why each capability is requested.

---

# 119. Hard rule.

---

# 120. Security Checklist

Examples:

```text
secret storage
webhook verification
scope minimization
rotation
revocation
```

---

# 121. Hard rule.

---

# 122. Privacy Checklist

Examples:

```text
data classes
retention
user consent
deletion
export
```

---

# 123. Hard rule.

---

# 124. Integration Certification Workflow

```text
sandbox complete
→ conformance test
→ security/privacy review if required
→ certification decision
→ production eligibility
```

---

# 125. Hard rule.

---

# 126. Certification Evidence

Portal references Part 126 archive.

---

# 127. Hard rule.

---

# 128. Certification Review UI

Displays:

```text
requested scopes
test results
compatibility
open findings
evidence
```

---

# 129. No user-private content.

---

# 130. Hard rule.

---

# 131. Production Promotion

Promotion does not reuse sandbox credential.

---

# 132. Hard rule.

---

# 133. Production Provisioning

Creates:

```text
production credential
production quota
production endpoint config
certification binding
```

---

# 134. Hard rule.

---

# 135. Environment Labels

Every credential/request clearly indicates:

```text
sandbox
certification
production
```

---

# 136. Hard rule.

---

# 137. Endpoint Isolation

Different domains/ALPNs/network routes where practical.

---

# 138. Hard rule.

---

# 139. Misrouting Protection

Production credential rejected in sandbox and vice versa.

---

# 140. Hard rule.

---

# 141. Developer Organization Roles

```rust
pub enum DeveloperOrganizationRole {
    Owner,
    Admin,
    Developer,
    SecurityReviewer,
    BillingViewer,
    SupportViewer,
}
```

---

# 142. Scoped roles.

---

# 143. Hard rule.

---

# 144. Organization Role != Runtime Integration Capability

Hard rule.

---

# 145. MFA/Passkeys

Required for sensitive developer portal actions where policy applies.

---

# 146. Hard rule.

---

# 147. Production Credential Creation

Step-up authentication.

---

# 148. Hard rule.

---

# 149. Organization Membership

Invitation-based.

---

# 150. Hard rule.

---

# 151. Join By Email Alone Not Sufficient

Hard rule.

---

# 152. Organization Offboarding

Revokes portal access.

---

# 153. Does not silently revoke integration runtime unless policy/action says so.

---

# 154. Hard rule.

---

# 155. Organization Transfer

Requires explicit governance.

---

# 156. Hard rule.

---

# 157. Developer Support Architecture

Support channels:

```text
documentation
self-service diagnostics
ticket/support case
security contact
incident/status updates
```

---

# 158. Hard rule.

---

# 159. Support Case Identity

```rust
pub struct DeveloperSupportCaseId(pub [u8; 16]);
```

---

# 160. Support Case

```rust
pub struct DeveloperSupportCase {
    pub id: DeveloperSupportCaseId,
    pub project: DeveloperProjectId,
    pub category: DeveloperSupportCategory,
    pub state: DeveloperSupportState,
}
```

---

# 161. Support Category

```rust
pub enum DeveloperSupportCategory {
    Integration,
    Authentication,
    Compatibility,
    Billing,
    Certification,
    Outage,
    Security,
}
```

---

# 162. Hard rule.

---

# 163. Support State

```rust
pub enum DeveloperSupportState {
    Open,
    Triaged,
    WaitingForDeveloper,
    Investigating,
    Resolved,
    Closed,
}
```

---

# 164. Hard rule.

---

# 165. Support Access

Support role receives minimum diagnostic scope.

---

# 166. Hard rule.

---

# 167. No Support Superuser

Hard rule.

---

# 168. Support Diagnostic Bundle

Developer can generate/share scoped diagnostic bundle.

---

# 169. Hard rule.

---

# 170. Diagnostic Bundle

```rust
pub struct DeveloperDiagnosticBundle {
    pub project: DeveloperProjectId,
    pub generated_at: Timestamp,
    pub components: Vec<DiagnosticComponent>,
    pub redaction_policy: DiagnosticRedactionPolicy,
}
```

---

# 171. Default Redaction

Strip:

```text
secrets
message content
user identifiers
access tokens
```

---

# 172. Hard rule.

---

# 173. Correlation IDs

Use scoped diagnostic IDs.

---

# 174. No global user trace.

---

# 175. Hard rule.

---

# 176. Self-Service Diagnostics

Portal can show:

```text
credential state
certification state
version compatibility
quota status
recent error-class aggregates
webhook health
```

---

# 177. Hard rule.

---

# 178. No Raw Backend Logs By Default

Hard rule.

---

# 179. Error Explorer

Shows documented error classes/remediation.

---

# 180. Hard rule.

---

# 181. Request Inspector

Sandbox only or privacy-safe production metadata.

---

# 182. Hard rule.

---

# 183. Production Request Inspector

Never displays private user payload by default.

---

# 184. Hard rule.

---

# 185. Webhook Inspector

Can show:

```text
delivery status
response code
retry state
```

---

# 186. Payload masked/redacted where sensitive.

---

# 187. Hard rule.

---

# 188. Status Page

Developer-specific status surface.

---

# 189. Status Scope

```text
API
SDK distribution
sandbox
certification
webhooks
```

---

# 190. Hard rule.

---

# 191. Incident Communication

Developers receive:

```text
impact
affected interfaces
workaround
next update
recovery state
```

---

# 192. No internal sensitive topology.

---

# 193. Hard rule.

---

# 194. Deprecation Communication

Portal surfaces:

```text
deprecated APIs
sunset dates
migration guides
```

---

# 195. Hard rule.

---

# 196. Compatibility Alerts

Project can subscribe to:

```text
SDK EOL
API deprecation
protocol support change
```

---

# 197. Hard rule.

---

# 198. Notification Preferences

Scoped.

---

# 199. Hard rule.

---

# 200. No Marketing Notifications Mixed With Critical Lifecycle Alerts

Hard rule.

---

# 201. Changelog

Versioned.

---

# 202. Machine-readable + human-readable.

---

# 203. Hard rule.

---

# 204. Changelog Entry

```rust
pub struct DeveloperChangelogEntry {
    pub version: Version,
    pub class: ChangelogClass,
    pub breaking: bool,
    pub migration: Option<DocumentRef>,
}
```

---

# 205. Changelog Class

```rust
pub enum ChangelogClass {
    Added,
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}
```

---

# 206. Hard rule.

---

# 207. Breaking Change

Highlighted.

---

# 208. Hard rule.

---

# 209. Migration Guide

Required for breaking supported surface.

---

# 210. Hard rule.

---

# 211. Developer Analytics Boundary

Useful aggregate metrics:

```text
docs page errors
sandbox API error classes
SDK version adoption aggregate
certification completion funnel aggregate
```

---

# 212. Not:

```text
individual developer productivity
detailed browsing history
source-code profiling
user behavior from integrations
```

---

# 213. Hard rule.

---

# 214. Documentation Analytics

Privacy-preserving aggregate only.

---

# 215. Hard rule.

---

# 216. Search Analytics

Query aggregation only where safe.

---

# 217. Do not retain sensitive pasted values.

---

# 218. Hard rule.

---

# 219. Portal Search

Search:

```text
docs
SDK symbols
errors
changelog
migration guides
```

---

# 220. No private support content by default.

---

# 221. Hard rule.

---

# 222. AI Documentation Assistant

Optional.

May answer from:

```text
official docs
SDK reference
changelog
known errors
```

---

# 223. Must cite sources internally/visibly where UI supports.

---

# 224. Hard rule.

---

# 225. AI Cannot Reveal Private Portal Data Without Authorization

Hard rule.

---

# 226. AI Cannot Invent Unsupported API

Hard rule.

---

# 227. AI Suggestions

Advisory.

---

# 228. Hard rule.

---

# 229. Sample Code Generation

Must target selected SDK version.

---

# 230. Hard rule.

---

# 231. Version-Aware Docs Assistant

Prompt context includes:

```text
SDK version
language
environment
```

---

# 232. Hard rule.

---

# 233. No Production Secrets To AI

Hard rule.

---

# 234. Support Escalation

Escalation path:

```text
docs
→ self-service
→ support
→ engineering/security
```

---

# 235. Hard rule.

---

# 236. Security Report Channel

Separate from normal support.

---

# 237. Hard rule.

---

# 238. Vulnerability Disclosure

Part 98.

---

# 239. Hard rule.

---

# 240. Integration Health

Portal may show:

```text
credential active
certification valid
webhook reachable
compatibility current
quota headroom
```

---

# 241. Hard rule.

---

# 242. No Business/User KPI Dashboard By Default

Hard rule.

---

# 243. Production Logs

Developer sees only integration-scoped safe logs.

---

# 244. Hard rule.

---

# 245. Log Field Policy

Allowed:

```text
request class
status
latency
correlation ID
```

Disallowed by default:

```text
message content
contact IDs
raw tokens
internal topology
```

---

# 246. Hard rule.

---

# 247. Log Retention

Short and policy-bounded.

---

# 248. Hard rule.

---

# 249. Developer Portal Security

Threat model includes:

```text
credential theft
organization takeover
CSRF/XSS
support impersonation
scope escalation
sandbox escape
```

---

# 250. Hard rule.

---

# 251. Portal Session

Short-lived sensitive session.

---

# 252. Hard rule.

---

# 253. Step-Up Actions

Examples:

```text
production credential create
credential revoke
owner transfer
scope expansion
```

---

# 254. Hard rule.

---

# 255. Audit

Audit high-value portal actions.

---

# 256. Do not audit every docs page view.

---

# 257. Hard rule.

---

# 258. Audit Events

```text
credential created/revoked
integration scope changed
certification approved/revoked
organization owner changed
```

---

# 259. Hard rule.

---

# 260. Sandbox Security

No route to production control plane.

---

# 261. Hard rule.

---

# 262. Network Namespace / Tenant Isolation

Use strong isolation.

---

# 263. Hard rule.

---

# 264. Sandbox Abuse Protection

Rate limits/quotas.

---

# 265. Hard rule.

---

# 266. Synthetic Abuse Scenarios

Certification sandbox may test:

```text
rate limits
invalid scopes
replay
malformed webhooks
```

---

# 267. Hard rule.

---

# 268. Production Eligibility

```rust
pub struct ProductionEligibility {
    pub project: DeveloperProjectId,
    pub certification: Option<IntegrationCertificationId>,
    pub compatibility: CompatibilityState,
    pub security_review: ReadinessState,
    pub privacy_review: ReadinessState,
}
```

---

# 269. Hard rule.

---

# 270. Production Eligibility Decision

```rust
pub enum ProductionEligibilityDecision {
    Eligible,
    EligibleWithConditions,
    NotEligible,
}
```

---

# 271. Hard rule.

---

# 272. Production Promotion Gate

Checks:

```text
project state
manifest
certification
compatibility
credential policy
security/privacy review
```

---

# 273. Hard rule.

---

# 274. No Manual Bypass Button

Hard rule.

---

# 275. Exception

If governance permits, typed and time-bounded.

---

# 276. Hard rule.

---

# 277. Developer Environment Config

```rust
pub struct DeveloperEnvironmentConfig {
    pub environment: DeveloperEnvironment,
    pub endpoint_set: EndpointSetId,
    pub credential_class: DeveloperCredentialClass,
}
```

---

# 278. Environment

```rust
pub enum DeveloperEnvironment {
    Sandbox,
    Certification,
    Production,
}
```

---

# 279. Hard rule.

---

# 280. Environment-Aware SDK Config

SDK requires explicit environment selection.

---

# 281. Hard rule.

---

# 282. No Production By Default

Hard rule.

---

# 283. Default SDK Environment

Sandbox.

---

# 284. Hard rule.

---

# 285. CI For Integrations

Developer can use non-human service credential for sandbox/certification CI.

---

# 286. Hard rule.

---

# 287. CI Credential

Scoped and expiring.

---

# 288. Hard rule.

---

# 289. Production CI

Use workload identity where possible.

---

# 290. Hard rule.

---

# 291. Secret Rotation Automation

Supported.

---

# 292. Hard rule.

---

# 293. Portal API

Portal itself exposes developer-management API.

---

# 294. Separate from end-user APIs.

---

# 295. Hard rule.

---

# 296. Developer Management API

```rust
pub trait DeveloperPortalService {
    fn create_project(
        &self,
        organization: DeveloperOrganizationId,
        manifest: IntegrationManifest,
    ) -> Result<DeveloperProjectId, DeveloperPortalError>;

    fn project(
        &self,
        id: DeveloperProjectId,
    ) -> Result<DeveloperProject, DeveloperPortalError>;
}
```

---

# 297. Credential Service

```rust
pub trait DeveloperCredentialService {
    fn issue(
        &self,
        project: DeveloperProjectId,
        class: DeveloperCredentialClass,
    ) -> Result<DeveloperCredentialReceipt, DeveloperPortalError>;

    fn revoke(
        &self,
        credential: DeveloperCredentialId,
    ) -> Result<(), DeveloperPortalError>;
}
```

---

# 298. Sandbox Service

```rust
pub trait DeveloperSandboxService {
    fn provision(
        &self,
        project: DeveloperProjectId,
    ) -> Result<SandboxEnvironmentId, DeveloperPortalError>;

    fn reset(
        &self,
        sandbox: SandboxEnvironmentId,
    ) -> Result<(), DeveloperPortalError>;
}
```

---

# 299. Support Service

```rust
pub trait DeveloperSupportService {
    fn open_case(
        &self,
        project: DeveloperProjectId,
        category: DeveloperSupportCategory,
    ) -> Result<DeveloperSupportCaseId, DeveloperPortalError>;
}
```

---

# 300. No User-Analytics API

Hard rule.

---

# 301. Error Taxonomy

```rust
pub enum DeveloperPortalError {
    OrganizationUnknown,
    ProjectUnknown,
    SandboxUnavailable,
    CredentialNotAllowed,
    ProductionNotEligible,
    CertificationRequired,
    ScopeExpansionReviewRequired,
    CompatibilityMismatch,
    QuotaExceeded,
    Unauthorized,
    Internal,
}
```

---

# 302. Observability

Safe metrics:

```text
sandbox health
credential issuance failures
certification workflow state
docs build health
portal availability
support case backlog
```

---

# 303. Forbidden:

```text
developer productivity
individual docs browsing profile
user behavior via integration
```

---

# 304. Hard rule.

---

# 305. Developer Portal SLOs

Examples:

```text
docs available
sandbox provisioning within target
credential revocation within target
status updates within incident cadence
```

---

# 306. Security SLO

```text
0 sandbox credential accepted in production
0 revoked production credential accepted
0 unapproved scope expansion active
```

---

# 307. Privacy SLO

```text
0 production user data copied into sandbox
0 support diagnostic bundle containing unredacted secrets by default
0 developer behavior ranking
```

---

# 308. Failure Modes

```text
sandbox escape
credential confusion
docs/version mismatch
scope drift
support overprivilege
```

---

# 309. Sandbox Escape

Contain/rotate/investigate.

---

# 310. Hard rule.

---

# 311. Credential Confusion

Environment-class binding rejects.

---

# 312. Hard rule.

---

# 313. Docs Version Mismatch

Version pin/warning.

---

# 314. Hard rule.

---

# 315. Scope Drift

Manifest diff + re-review.

---

# 316. Hard rule.

---

# 317. Support Overprivilege

Capability policy denies.

---

# 318. Hard rule.

---

# 319. Testing

Need developer-experience testkit.

---

# 320. Test Scenarios

```text
new project onboarding
sandbox reset
production promotion
credential rotation
scope expansion
support diagnostic bundle
```

---

# 321. Onboarding Test

Draft cannot skip to production.

---

# 322. Sandbox Test

Sandbox cannot reach production storage/control.

---

# 323. Credential Test

Sandbox credential rejected by production endpoint.

---

# 324. Rotation Test

New credential works; old expires/revokes after overlap.

---

# 325. Scope Test

Sensitive capability addition requires re-review.

---

# 326. Docs Test

Examples compile against selected SDK version.

---

# 327. Compatibility Test

Portal warns/blocks unsupported SDK version.

---

# 328. Production Gate Test

Missing certification denies promotion.

---

# 329. Support Test

Support cannot see raw message content by default.

---

# 330. Diagnostic Bundle Test

Secrets/tokens/private content redacted.

---

# 331. AI Docs Test

Assistant cannot claim nonexistent API.

---

# 332. Privacy Test

No individual developer browsing/activity profile stored.

---

# 333. Fuzzing

Fuzz:

```text
integration manifests
portal state transitions
credential requests
sandbox config
diagnostic bundle manifests
```

---

# 334. Property Tests

Properties:

```text
sandbox credential can never authorize production request
production credential can never be issued before production eligibility
scope expansion can never activate without required review
diagnostic bundle can never include a secret-class field under default redaction
```

---

# 335. Formal Verification Targets

Strong candidates:

```text
onboarding state machine
credential environment binding
scope-expansion review
promotion gate
```

---

# 336. Kani Candidate

credential/environment and promotion invariants.

---

# 337. TLA+ Candidate

project create → sandbox → conformance → certification → production → revoke.

---

# 338. Loom Candidate

concurrent credential rotation + request authentication + revocation.

---

# 339. Performance

Portal not on core messaging hot path.

---

# 340. Hard rule.

---

# 341. Caching

Docs/static reference aggressively cacheable.

---

# 342. Credential/certification state must remain fresh.

---

# 343. Hard rule.

---

# 344. Search Index

Docs index can be rebuilt.

---

# 345. Hard rule.

---

# 346. Sandbox Provisioning

Asynchronous durable job.

---

# 347. Hard rule.

---

# 348. Storage

Separate:

```text
developer organizations
projects
portal roles
sandbox metadata
credential metadata
onboarding state
support cases
notification preferences
```

---

# 349. Secrets not stored in general application DB.

---

# 350. Hard rule.

---

# 351. Partitioning

By:

```text
developer organization
project
environment
integration
```

---

# 352. No end-user data partition in developer portal.

---

# 353. Hard rule.

---

# 354. Crate Layout

Recommended:

```text
crates/
├── siar-developer-portal-core/
├── siar-developer-docs/
├── siar-developer-projects/
├── siar-developer-sandbox/
├── siar-developer-credentials/
├── siar-integration-onboarding/
├── siar-developer-support/
├── siar-developer-status/
├── siar-developer-observability/
└── siar-developer-portal-testkit/
```

---

# 355. `siar-developer-portal-core`

Owns:

```text
DeveloperProjectId
DeveloperEnvironment
DeveloperProjectState
DeveloperPortalError
```

---

# 356. `siar-developer-docs`

Versioned docs/API reference/examples/changelog.

---

# 357. `siar-developer-projects`

Organization/project lifecycle.

---

# 358. `siar-developer-sandbox`

Synthetic isolated environments.

---

# 359. `siar-developer-credentials`

Credential request/rotation/revocation metadata.

---

# 360. `siar-integration-onboarding`

Onboarding/promotions/scope-review.

---

# 361. `siar-developer-support`

Cases/diagnostic bundles/escalation.

---

# 362. `siar-developer-status`

API/sandbox/certification incident status.

---

# 363. `siar-developer-observability`

Aggregate developer-platform health only.

---

# 364. `siar-developer-portal-testkit`

sandbox/credential/privacy/state tests.

---

# 365. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Developer portal identity, organization membership, integration identity, sandbox credentials, production credentials, SDK access, and runtime capabilities remain separate authorities; portal login never implies production API authority.
2. Sandbox, certification, and production environments use distinct credentials, storage, quotas, endpoints, and control-plane boundaries; sandbox credentials can never authorize production access.
3. Production credentials are issued only after the project reaches explicit production eligibility, including required manifest, compatibility, certification, and security/privacy checks.
4. Scope/data-class/network/file-access expansion is detected as a manifest diff and triggers required re-review, re-consent, or re-certification before activation.
5. Developer documentation, generated references, examples, migration guides, and changelogs are version-aware and tied to supported SDK/API/protocol versions; examples are continuously compiled/tested.
6. Sandbox/test tenants use synthetic data by default and cannot receive copied production user data, secrets, or message history as a convenience for integration testing.
7. Support and self-service diagnostics expose only integration-scoped, redacted technical data; no support role or portal feature creates a universal backend/private-content view.
8. Credential issuance, display, storage, rotation, revocation, and workload enrollment follow least-privilege secret-management rules; secrets are never emailed, logged, or repeatedly displayed.
9. Developer-platform telemetry is aggregate and operational—docs health, sandbox failures, compatibility errors, certification workflow—not a persistent browsing profile, productivity score, source-code surveillance system, or route to end-user behavior.
10. Production status, lifecycle/deprecation notices, compatibility alerts, and incident communications remain distinct from marketing messaging so critical developer obligations cannot be obscured or opt-in dependent.
11. AI-assisted documentation/support may suggest code and explain official interfaces but cannot invent unsupported APIs, expose unauthorized portal data, consume production secrets, or become an authority for certification/credential decisions.
12. The developer-experience architecture integrates with SDKs, compatibility, product lifecycle/readiness, certification archives, authorization, secrets, incident/status, risk/PIR, architecture governance, audit/compliance, and integration certification without becoming an alternate path around SIAR's security, privacy, anonymity, or tenant boundaries.
```

---

# 366. Initial Production Scope

Implement first:

```text
developer organizations/projects
project state machine
versioned docs portal
generated SDK/API reference
compile-tested Rust/Kotlin examples
shared synthetic sandbox
dedicated certification sandbox
sandbox reset
sandbox quotas
sandbox/certification/production credential classes
credential broker integration
one-time secret delivery
rotation/revocation
integration manifest viewer/diff
onboarding checklist
conformance/certification workflow
production promotion gate
developer status/changelog/deprecation alerts
self-service compatibility/quota/credential diagnostics
redacted support bundle
support case workflow
privacy-safe portal metrics
developer-experience testkit
```

Then add:

```text
dedicated per-project sandbox clusters
interactive SDK playground
version-aware AI docs assistant
automated migration assistant
workload-identity provisioning
portable certification handoff
developer CLI for project/sandbox/credential operations
formal promotion/credential-state verification
```

---

# 367. Definition of Done

Part 131 is complete when:

- developer projects have a governed lifecycle;
- docs are versioned and generated from canonical interfaces;
- examples are continuously tested;
- sandbox is isolated and synthetic;
- credential classes are environment-bound;
- production credentials require eligibility;
- scope expansion triggers review;
- integration onboarding/certification is explicit;
- sandbox/production promotion is gated;
- support diagnostics are redacted/scoped;
- status/deprecation/compatibility communication exists;
- AI assistance remains advisory and secret-safe;
- no developer/end-user surveillance metrics are created;
- sandbox/credential/onboarding/privacy/fuzz/formal tests are specified.

---

# 368. Final Architecture

```text
                      DEVELOPER
                          │
                          ▼
                  DEVELOPER PORTAL
                          │
              ┌───────────┼───────────┐
              │           │           │
            DOCS       SANDBOX     PROJECT
              │           │           │
              └───────────┼───────────┘
                          ▼
                 INTEGRATION MANIFEST
                          │
                          ▼
               CONFORMANCE/CERTIFICATION
                          │
                          ▼
                  PRODUCTION ELIGIBILITY
                          │
                          ▼
                 PRODUCTION CREDENTIAL
                          │
                          ▼
                 SUPPORT / LIFECYCLE
```

Developer-experience safety model:

```text
versioned documentation
+
isolated synthetic sandbox
+
environment-bound credentials
+
manifest-driven scope
+
certification-gated promotion
+
scoped support diagnostics
+
clear lifecycle/status communication
+
privacy-safe developer telemetry
```

not:

```text
give every developer a permanent production key, copy production data into a sandbox, expose raw logs to support, and track every click in the portal
```

---

# 369. Final Principle

A developer platform should make the safe path the easiest path.

The correct model is:

```text
teach from canonical docs
+
let developers experiment in synthetic sandboxes
+
scope credentials by environment
+
make requested authority visible
+
certify before privileged production use
+
offer safe diagnostics/support
+
communicate compatibility and lifecycle clearly
+
never trade developer convenience for weaker security or privacy
```

This architecture gives SIAR a privacy-preserving developer-experience foundation for documentation, sandboxes, credential provisioning, onboarding, certification, production promotion, support, status communication, diagnostics, and lifecycle guidance while preserving the anonymity, local-first, least-authority, SDK, compatibility, and anti-surveillance guarantees established across Parts 34–130.
