# Core System Architecture Part 82 — Anonymous Network Identity Provider, Authentication, MFA, Passkeys, Session Management & Account Security Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 82  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 2, 15, 28, 31, 42, 45–47, 53, 56–58, 61, 68–81

**Primary purpose:** define SIAR's account authentication architecture, including identity-provider responsibilities, passkeys/WebAuthn, MFA, password fallback, device-bound sessions, session/token lifecycle, refresh rotation, revocation, suspicious-login handling, anti-phishing, managed-enterprise IdP integration, recovery interactions, account security posture, and strict separation between authentication identity and anonymous communication identity.

---

# 1. Purpose

Authentication is necessary for many account and administrative workflows.

But authentication systems often become privacy hazards by centralizing:

```text
email addresses
phone numbers
device fingerprints
session histories
IP histories
social identity
```

The governing principle is:

> **SIAR authentication should prove account or administrative authority without turning the authentication provider into the identity root of the anonymous communication graph.**

---

# 2. Architectural Position

```text
Human / Device
      │
      ▼
Authentication Method
      │
      ▼
Identity Provider / Account Authority
      │
      ▼
Authenticated Session
      │
      ▼
Authorization Layer (Part 81)
      │
      ▼
Permitted Account / Admin Actions
```

---

# 3. Core Separation

Keep distinct:

```text
account identity
authentication credential
session identity
device identity
workload identity
transport identity
mailbox identity
anonymous communication identity
```

---

# 4. Non-Goals

Part 82 does not:

```text
make login identity equal to messaging identity
require phone numbers
require social login
expose authenticated account IDs into anonymous transport
reuse web session tokens as message capabilities
```

---

# 5. Authentication Domains

```rust
pub enum AuthenticationDomain {
    PersonalAccount,
    ManagedEnterprise,
    OperatorAdministration,
    ServiceConsole,
}
```

---

# 6. Personal Account

User-owned account authority.

---

# 7. Managed Enterprise

Organization-scoped identity.

---

# 8. Operator Administration

Infrastructure operators.

---

# 9. Service Console

Scoped support/operations interface.

---

# 10. Hard Rule

These domains do not share session/token authority by default.

---

# 11. Identity Provider Role

The identity provider authenticates.

It does not automatically authorize all actions.

---

# 12. Authentication ≠ Authorization

Hard rule.

---

# 13. Account Identifier

```rust
pub struct AccountId(pub [u8; 32]);
```

Opaque/random.

---

# 14. No Email As Primary Internal Key

Hard rule.

---

# 15. Login Handle

Optional.

Examples:

```text
email
username
managed enterprise subject
```

---

# 16. Login Handle Is Lookup Metadata

Not protocol identity.

---

# 17. Account Identity Boundary

Account identity must never become:

```text
Iroh EndpointId
mailbox ID
mixnet route ID
federation user ID
```

---

# 18. Hard Rule

Authentication identifiers never appear in anonymous routing metadata.

---

# 19. Authentication Methods

```rust
pub enum AuthenticationMethod {
    Passkey,
    Password,
    Totp,
    HardwareSecurityKey,
    ManagedIdentityProvider,
    RecoveryCredential,
}
```

---

# 20. Preferred Method

Passkey.

---

# 21. Why

Passkeys provide:

```text
phishing resistance
origin binding
device/user verification
no password reuse
```

---

# 22. WebAuthn/FIDO2

Use standard implementation.

---

# 23. No Custom Passkey Protocol

Hard rule.

---

# 24. Passkey Credential

```rust
pub struct PasskeyCredential {
    pub credential_id: PasskeyCredentialId,
    pub public_key: PasskeyPublicKey,
    pub sign_count: Option<u32>,
    pub transports: BTreeSet<AuthenticatorTransport>,
}
```

---

# 25. Private Key

Remains in authenticator/device.

---

# 26. Server Stores

```text
credential ID
public key
metadata
```

---

# 27. No Private Passkey Material Server-Side

Hard rule.

---

# 28. Resident Credentials

Supported where authenticator allows.

---

# 29. User Verification

Preferred.

---

# 30. Platform Authenticator

Examples:

```text
Android Credential Manager
Windows Hello
Touch ID
```

---

# 31. Hardware Security Key

Also supported.

---

# 32. Multiple Passkeys Per Account

Required.

---

# 33. Why

Avoid one-device lockout.

---

# 34. Passkey Registration

Requires authenticated account/session or recovery flow.

---

# 35. New Passkey Addition

High-risk account action.

---

# 36. Reauthentication

Required.

---

# 37. Existing Strong Credential

Preferred.

---

# 38. No Silent Passkey Enrollment

Hard rule.

---

# 39. Passkey Label

User-visible convenience only.

---

# 40. Do Not Store Exact Device Fingerprint

Hard rule.

---

# 41. Passwords

Fallback only.

---

# 42. Password Policy

Prefer:

```text
long minimum
no arbitrary composition rules
breached-password rejection
```

---

# 43. No Forced Periodic Password Rotation Without Risk Signal

Preferred.

---

# 44. Password Storage

Memory-hard KDF.

---

# 45. Recommended Class

Argon2id.

---

# 46. Password Hash Record

```rust
pub struct PasswordHashRecord {
    pub algorithm: PasswordHashAlgorithm,
    pub parameters: PasswordHashParameters,
    pub salt: Salt,
    pub hash: SecretHash,
}
```

---

# 47. Unique Salt

Mandatory.

---

# 48. Server Pepper

Optional.

---

# 49. If Used

Stored separately in secret management.

---

# 50. No Plaintext Password

Hard rule.

---

# 51. No Reversible Password Encryption

Hard rule.

---

# 52. Password Upgrade

Rehash on successful login when parameters outdated.

---

# 53. Breached Password Check

Prefer local/offline prefix database or privacy-preserving service.

---

# 54. No Full Password Sent To Third Party

Hard rule.

---

# 55. MFA

Required for high-risk contexts.

---

# 56. MFA Strength

```rust
pub enum MfaStrength {
    None,
    SingleStrongFactor,
    MultiFactor,
    PhishingResistant,
}
```

---

# 57. Passkey With User Verification

Can satisfy phishing-resistant strong auth.

---

# 58. Operator Administration

Require phishing-resistant MFA.

---

# 59. Tenant Security Admin

Prefer phishing-resistant MFA.

---

# 60. TOTP

Supported as fallback.

---

# 61. SMS OTP

Not recommended.

---

# 62. If Supported For Legacy

Low-assurance only.

---

# 63. Hard Rule

SMS OTP must not be sole protection for root/operator/high-risk actions.

---

# 64. Email OTP

Similarly low assurance.

---

# 65. TOTP Secret

Encrypted at rest.

---

# 66. Recovery Codes

One-time.

---

# 67. Recovery Code Storage

Hash only.

---

# 68. No Plaintext Recovery Code Storage

Hard rule.

---

# 69. MFA Enrollment

Reauthentication required.

---

# 70. MFA Removal

High-risk.

---

# 71. Require:

```text
strong existing factor
or
recovery procedure
```

---

# 72. No Support-Agent MFA Disable By Default

Hard rule.

---

# 73. Step-Up Authentication

Used for sensitive actions.

---

# 74. Examples

```text
new device enrollment
passkey addition/removal
recovery change
tenant admin grant
key export
account deletion
```

---

# 75. Authentication Assurance

```rust
pub enum AuthenticationAssurance {
    Low,
    Standard,
    Strong,
    PhishingResistant,
}
```

---

# 76. Session Records Assurance

Current level.

---

# 77. Action Requires Minimum Assurance

```rust
pub struct AuthenticationRequirement {
    pub minimum: AuthenticationAssurance,
    pub max_age: Duration,
}
```

---

# 78. Max Age

Sensitive action may require recent authentication.

---

# 79. No Old Session For Root-Like Action

Hard rule.

---

# 80. Session Model

```rust
pub struct SessionId(pub [u8; 32]);
```

Random opaque.

---

# 81. Session Record

```rust
pub struct SessionRecord {
    pub session_id: SessionId,
    pub account: AccountId,
    pub device: Option<DeviceId>,
    pub assurance: AuthenticationAssurance,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    pub state: SessionState,
}
```

---

# 82. Session State

```rust
pub enum SessionState {
    Active,
    ReauthRequired,
    Revoked,
    Expired,
}
```

---

# 83. Session Lifetime

Bounded.

---

# 84. No Permanent Session

Hard rule.

---

# 85. Access Token

Short-lived.

---

# 86. Refresh Token

Longer-lived but rotated.

---

# 87. Token Classes

```rust
pub enum SessionTokenType {
    Access,
    Refresh,
    Reauth,
}
```

---

# 88. Access Token

Minutes.

---

# 89. Refresh Token

Days/weeks depending context.

---

# 90. Operator Session

Shorter.

---

# 91. Refresh Token Rotation

Mandatory.

---

# 92. Refresh Family

```rust
pub struct RefreshTokenFamilyId(pub [u8; 32]);
```

---

# 93. Rotation Flow

```text
refresh token N
→ exchange
→ invalidate N
→ issue N+1
```

---

# 94. Reuse Detection

If old refresh token reused:

```text
revoke family
```

---

# 95. Hard Rule

Refresh token replay is treated as compromise signal.

---

# 96. Token Storage

Hash server-side if opaque bearer token.

---

# 97. No Plaintext Long-Lived Refresh Token Database

Preferred hard rule.

---

# 98. Client Storage

Platform secure store.

---

# 99. Browser

HttpOnly + Secure + SameSite cookies if web.

---

# 100. Native App

OS secure storage.

---

# 101. No Local Plaintext Token File

Hard rule.

---

# 102. Token Binding

Where practical bind to:

```text
device key
session key
DPoP-like proof
```

---

# 103. Bearer Token Risk

Stolen token usable by attacker.

---

# 104. Proof-of-Possession Token

Stronger.

---

# 105. Device-Bound Session

```rust
pub struct DeviceBoundSession {
    pub session: SessionId,
    pub device_key: DeviceSessionPublicKey,
}
```

---

# 106. Device Session Key

Separate from messaging device identity key.

---

# 107. Hard Rule

Login device key is not reused as E2EE messaging identity key.

---

# 108. Request Proof

Sign nonce/request context.

---

# 109. Token Theft

Less useful without device key.

---

# 110. Session Cookie

If browser, cannot easily device-bind universally.

---

# 111. Apply best available method per platform.

---

# 112. Session Revocation

User can revoke:

```text
one session
all sessions
one device
```

---

# 113. Session Revocation Record

```rust
pub struct SessionRevocation {
    pub session: SessionId,
    pub effective_at: Timestamp,
    pub reason: SessionRevocationReason,
}
```

---

# 114. Reasons

```rust
pub enum SessionRevocationReason {
    UserRequested,
    CredentialChanged,
    SuspiciousActivity,
    DeviceLost,
    AccountRecovery,
    AdminSecurityAction,
}
```

---

# 115. Revocation Propagation

Fast.

---

# 116. Cached auth state

Revocation overrides.

---

# 117. Hard Rule

Revoked session cannot return through offline cache.

---

# 118. Logout

Revokes refresh session.

---

# 119. Access Token

May remain valid until short expiry unless revocation check needed.

---

# 120. High-risk API

Check session revocation actively/local feed.

---

# 121. Session List

User-visible.

---

# 122. Show:

```text
approximate device type
last active coarse time
security state
```

---

# 123. Avoid:

```text
precise IP history
fine geolocation
```

---

# 124. Hard Rule

Session UI must not create a surveillance-grade location history.

---

# 125. Device Enrollment

Separate from login.

---

# 126. New Device

Authenticated account session establishes device association.

---

# 127. Device Approval

Can require existing trusted device.

---

# 128. QR/bootstrap

Part 15/57.

---

# 129. Device Enrollment State

```rust
pub enum DeviceEnrollmentState {
    Pending,
    Approved,
    Active,
    Revoked,
}
```

---

# 130. New Device ≠ Restored Device Identity

Hard rule.

---

# 131. Fresh DeviceId

Part 57.

---

# 132. Fresh messaging sessions/ratchets.

---

# 133. Account Session

May restore account access.

---

# 134. Does Not Restore Old Transport Session Secrets

Hard rule.

---

# 135. Suspicious Authentication Detection

Signals:

```text
refresh replay
impossible credential change sequence
many failed attempts
new passkey then recovery change
known compromised credential
```

---

# 136. Avoid

Heavy behavioral fingerprinting.

---

# 137. No Always-On Device Fingerprinting

Hard rule.

---

# 138. Risk Engine

Rule-based initially.

---

# 139. Authentication Risk Signal

```rust
pub enum AuthenticationRiskSignal {
    CredentialReplay,
    ExcessiveFailures,
    RecoveryAttempt,
    NewHighPrivilegeFactor,
    RevokedDeviceUse,
}
```

---

# 140. Risk Response

```rust
pub enum AuthenticationRiskResponse {
    Allow,
    RequireStepUp,
    DelaySensitiveAction,
    RevokeSession,
    LockSensitiveChanges,
}
```

---

# 141. No Silent Permanent Account Lock On Weak Signal

Hard rule.

---

# 142. Rate Limiting

Authentication endpoints need:

```text
per-account
per-source coarse
global
```

---

# 143. Enumeration Resistance

Login error should not reveal account existence unnecessarily.

---

# 144. Generic Failure

Preferred.

---

# 145. Timing Normalization

Where practical.

---

# 146. Password Guessing

Backoff/rate limit.

---

# 147. No Unlimited Login Attempts

Hard rule.

---

# 148. CAPTCHA

Optional fallback.

---

# 149. Privacy Cost

Third-party CAPTCHA can track.

---

# 150. Prefer first-party proof/rate limiting.

---

# 151. Anti-Phishing

Critical.

---

# 152. Passkeys

Primary defense.

---

# 153. Domain Binding

WebAuthn origin/RP ID.

---

# 154. No Passkey Ceremony On Untrusted Domain

Hard rule.

---

# 155. Login UI

Show exact official domain/app identity.

---

# 156. Deep-Link Authentication

Validate origin.

---

# 157. OAuth/OpenID Redirect

Strict redirect allowlist.

---

# 158. No Wildcard Redirect URI

Hard rule.

---

# 159. State + nonce

Required.

---

# 160. PKCE

Required for public clients.

---

# 161. Managed Identity Provider

Enterprise SSO.

---

# 162. Supported protocols

```text
OIDC
SAML where required
```

---

# 163. External IdP Identity

Organization-scoped.

---

# 164. Hard Rule

Managed IdP subject does not become SIAR global identity.

---

# 165. Managed Identity Mapping

```rust
pub struct ManagedIdentityBinding {
    pub organization: OrganizationId,
    pub issuer: ExternalIssuerId,
    pub subject: ExternalSubjectId,
    pub membership: OrganizationMembershipId,
}
```

---

# 166. External Subject

Scoped to issuer/org.

---

# 167. No Cross-Org Merge By Email

Hard rule.

---

# 168. OIDC Claims

Minimize.

---

# 169. Needed claims only.

---

# 170. No Full Directory Profile If Not Needed

Hard rule.

---

# 171. SAML

More complex.

---

# 172. Isolate parser/interop boundary.

---

# 173. JSON/XML external only.

---

# 174. Internal identity DTO

Typed Rust.

---

# 175. SCIM

Optional for enterprise provisioning.

---

# 176. SCIM affects managed membership only.

---

# 177. Does Not Touch Personal Identity.

---

# 178. Hard rule.

---

# 179. IdP Outage

Managed users may use cached session until expiry/policy.

---

# 180. New login unavailable.

---

# 181. No bypass to personal account unless explicitly linked.

---

# 182. Hard rule.

---

# 183. Enterprise Logout

Can revoke org session.

---

# 184. Personal session unaffected.

---

# 185. Operator Authentication

Highest assurance.

---

# 186. Requirements

```text
hardware-backed passkey/security key
short session
step-up for critical action
private admin boundary
```

---

# 187. No Password-Only Operator Login

Hard rule.

---

# 188. No Shared Operator Account

Hard rule.

---

# 189. Operator Session Scope

Explicit.

---

# 190. JIT Access

Part 80.

---

# 191. Authentication gives session.

---

# 192. Authorization gives admin capability.

---

# 193. Both required.

---

# 194. Support Personnel

Limited support sessions.

---

# 195. No Impersonation By Default

Hard rule.

---

# 196. If support access needed

Use explicit consent/scoped delegated session.

---

# 197. Support Session

Time-limited.

---

# 198. User-visible where appropriate.

---

# 199. No hidden "login as user" superpower.

---

# 200. Account Recovery

Part 57.

---

# 201. Recovery Authentication

Separate high-risk path.

---

# 202. Recovery Methods

```text
trusted device approval
recovery key
threshold recovery
managed enterprise recovery
```

---

# 203. Email-Only Recovery

Weak.

---

# 204. Avoid as sole method.

---

# 205. Phone-Only Recovery

Also weak.

---

# 206. Hard Rule

Recovery must not be weaker than normal authentication for high-value accounts.

---

# 207. Recovery Completion

Triggers:

```text
session revocation
refresh family revocation
new device credentials
security notification
```

---

# 208. Maybe credential rotation.

---

# 209. No Silent Recovery

Hard rule.

---

# 210. Recovery Cooldown

Sensitive changes can be delayed briefly.

---

# 211. Examples

```text
new recovery method
mass export
```

---

# 212. Risk-based.

---

# 213. Account Lock

Avoid permanent hard lock where recovery possible.

---

# 214. Security Lock State

```rust
pub enum AccountSecurityState {
    Normal,
    StepUpRequired,
    RecoveryRestricted,
    TemporarilyLocked,
    Compromised,
}
```

---

# 215. Compromised

Revokes sessions/credentials as policy dictates.

---

# 216. Account Deletion

Step-up auth.

---

# 217. Confirmation.

---

# 218. Lifecycle Part 67.

---

# 219. Session vs Message Identity

Critical.

---

# 220. Session AccountId

Used for account settings/admin.

---

# 221. Messaging layer

Uses relationship/device/group cryptographic identities.

---

# 222. Hard Rule

Message recipient does not need to know login email/account handle.

---

# 223. Anonymous Mode

Can minimize central account requirement.

---

# 224. Possible profiles

```rust
pub enum AccountProfileMode {
    LocalOnly,
    OptionalAccount,
    ManagedAccount,
}
```

---

# 225. LocalOnly

No server account required for local/offline/P2P use.

---

# 226. OptionalAccount

Needed for:

```text
backup
multi-device
managed mailbox
subscription
```

---

# 227. ManagedAccount

Enterprise.

---

# 228. Hard Rule

Core local P2P operation should not require cloud login unless feature truly depends on account service.

---

# 229. Local-First Authentication

Local device unlock may authorize local data access.

---

# 230. Device Unlock

OS biometric/PIN.

---

# 231. Not equivalent to server account login.

---

# 232. Separate local session.

---

# 233. Local Unlock Token

Memory-only.

---

# 234. No network exposure.

---

# 235. Local App Lock

Optional.

---

# 236. Use OS-backed user verification.

---

# 237. Session Persistence

Native app can preserve refresh credential securely.

---

# 238. But

App data backup must not clone live refresh token to new device.

---

# 239. Hard rule.

---

# 240. Backup Restore

Part 33/57.

---

# 241. Restored device

Requires new authentication.

---

# 242. No session-token restoration.

---

# 243. Hard rule.

---

# 244. Token Format

Opaque preferred for refresh tokens.

---

# 245. Access tokens

Opaque or signed.

---

# 246. Signed Token

Useful for offline verification.

---

# 247. But

Revocation harder.

---

# 248. Short TTL mitigates.

---

# 249. Token Audience

Mandatory.

---

# 250. Token Scope

Authentication session only, not broad authorization.

---

# 251. Example Access Session Claims

```rust
pub struct SessionClaims {
    pub session: SessionId,
    pub account: AccountId,
    pub assurance: AuthenticationAssurance,
    pub audience: SessionAudience,
    pub expires_at: Timestamp,
}
```

---

# 252. Do Not Put

```text
roles
all permissions
social graph
```

into long-lived auth token.

---

# 253. Authorization remains Part 81.

---

# 254. Token Signing Key

Separate from governance/release roots.

---

# 255. Key Rotation

Frequent.

---

# 256. JWKS-like publishing

If OIDC interoperability.

---

# 257. Internal key distribution

Signed config.

---

# 258. No Shared Signer Across Environments

Hard rule.

---

# 259. Session Audience

```rust
pub enum SessionAudience {
    AccountApi,
    ManagedOrgApi,
    OperatorConsole,
}
```

---

# 260. Audience Mismatch

Reject.

---

# 261. No One Token Works Everywhere

Hard rule.

---

# 262. CSRF

Browser sessions require CSRF defense.

---

# 263. SameSite cookie.

---

# 264. CSRF token for unsafe actions if needed.

---

# 265. Native apps

Not cookie-based.

---

# 266. XSS

Web UI hardening.

---

# 267. CSP.

---

# 268. HttpOnly.

---

# 269. No token in localStorage.

---

# 270. Hard rule.

---

# 271. Mobile Deep Links

Use verified app links/universal links.

---

# 272. Prevent malicious app interception.

---

# 273. OAuth App Redirect

PKCE.

---

# 274. Loopback redirect

Desktop optional.

---

# 275. Device Authorization Flow

Useful for headless/TV/CLI.

---

# 276. CLI Login

Could use browser passkey + device code.

---

# 277. No Password Pasting Into CLI If Avoidable.

---

# 278. CLI Token

Scoped.

---

# 279. Short/renewable.

---

# 280. Device Code

Short-lived.

---

# 281. User confirms on trusted browser/device.

---

# 282. Session Fixation

Prevent.

---

# 283. On login/step-up

Rotate session identifier.

---

# 284. Hard rule.

---

# 285. Session Concurrency

User may have multiple devices.

---

# 286. Policy can limit operator sessions.

---

# 287. Personal users

Reasonable multiple sessions.

---

# 288. No arbitrary one-device-only restriction.

---

# 289. Session Family

Per device/login.

---

# 290. Device Loss

Revoke device sessions.

---

# 291. Messaging device revocation

Related but separate.

---

# 292. Account security center

Shows both categories clearly.

---

# 293. Security Events

Examples:

```text
new passkey
new login
recovery started
MFA removed
session revoked
```

---

# 294. Security Notification

Local/push/email depending configured channel.

---

# 295. Do not include sensitive detail unnecessarily.

---

# 296. Notification Is Not Authentication.

---

# 297. Login Notification

May include coarse:

```text
device class
time
```

---

# 298. Avoid precise IP/location.

---

# 299. Hard rule.

---

# 300. Account Security Event Log

Local/account-level.

---

# 301. Retention bounded.

---

# 302. No lifetime tracking archive.

---

# 303. Security Event

```rust
pub struct AccountSecurityEvent {
    pub event_type: AccountSecurityEventType,
    pub session: Option<SessionId>,
    pub occurred_at: Timestamp,
}
```

---

# 304. No message metadata.

---

# 305. Password Reset

If supported.

---

# 306. Reset Link

Single-use.

---

# 307. Short-lived.

---

# 308. Store token hash.

---

# 309. No token in logs.

---

# 310. Referrer leakage protection.

---

# 311. Reset Completion

Revokes existing refresh families by policy.

---

# 312. Passkey-Primary Account

Password reset may not exist.

---

# 313. Better.

---

# 314. Account Enumeration

Registration/login/recovery endpoints.

---

# 315. Responses generic.

---

# 316. Rate limited.

---

# 317. Invite-only/enterprise

Different behavior allowed internally.

---

# 318. No public directory of accounts.

---

# 319. Hard rule.

---

# 320. Identity Provider Storage

Authoritative state:

```text
account
credential metadata
session state
recovery state
```

---

# 321. Not:

```text
message content
social graph
routing history
```

---

# 322. Hard rule.

---

# 323. Authentication Database

Separate logical store.

---

# 324. Tenant-managed auth

Tenant-scoped.

---

# 325. Personal auth

Separate.

---

# 326. No Cross-Tenant Session Table Query Without scope.

---

# 327. Part 69 applies.

---

# 328. Credential Table

Encrypted sensitive metadata.

---

# 329. Passkey public keys are not secret.

---

# 330. TOTP secrets are secret.

---

# 331. Refresh token hashes sensitive.

---

# 332. Session Database

Strong consistency for revocation.

---

# 333. Replication

HA.

---

# 334. No eventual revocation window beyond bounded design.

---

# 335. Session Cache

Can be local.

---

# 336. Revocation invalidates.

---

# 337. Authentication Service

Should remain small.

---

# 338. No giant profile service.

---

# 339. Profile metadata separate.

---

# 340. Why

Reduce auth blast radius.

---

# 341. IdP Service Interfaces

```rust
pub trait IdentityProvider {
    fn authenticate(
        &self,
        request: AuthenticationRequest,
    ) -> Result<AuthenticationResult, AuthenticationError>;

    fn revoke_session(
        &self,
        session: SessionId,
    ) -> Result<(), AuthenticationError>;
}
```

---

# 342. Passkey Service

```rust
pub trait PasskeyService {
    fn begin_registration(
        &self,
        account: AccountId,
    ) -> Result<PasskeyRegistrationChallenge, AuthenticationError>;

    fn finish_registration(
        &self,
        response: PasskeyRegistrationResponse,
    ) -> Result<PasskeyCredentialId, AuthenticationError>;
}
```

---

# 343. Session Service

```rust
pub trait SessionService {
    fn issue(
        &self,
        auth: VerifiedAuthentication,
    ) -> Result<SessionTokens, AuthenticationError>;

    fn refresh(
        &self,
        refresh: RefreshTokenProof,
    ) -> Result<SessionTokens, AuthenticationError>;
}
```

---

# 344. Risk Evaluator

```rust
pub trait AuthenticationRiskEvaluator {
    fn evaluate(
        &self,
        context: &AuthenticationRiskContext,
    ) -> AuthenticationRiskResponse;
}
```

---

# 345. Recovery Adapter

Part 57.

---

# 346. Managed IdP Adapter

OIDC/SAML adapter.

---

# 347. No External SDK Types In Core Domain

Hard rule.

---

# 348. Edge Integration

Part 78.

---

# 349. Login API behind L7 boundary.

---

# 350. WAF may protect request framing.

---

# 351. No passkey assertion logging.

---

# 352. Internal Service Integration

Part 79.

---

# 353. IdP uses mTLS internally.

---

# 354. Secrets Integration

Part 80.

---

# 355. Session signing keys dynamic/scoped.

---

# 356. Authorization Integration

Part 81.

---

# 357. Authenticated session maps to subject.

---

# 358. Authorization still separate.

---

# 359. Privacy Integration

Part 56.

---

# 360. Identity provider cannot repurpose authentication data for unrelated analytics by default.

---

# 361. Hard rule.

---

# 362. Legal Integration

Part 55.

---

# 363. Retention bounded.

---

# 364. Managed enterprise may require retention of admin authentication events.

---

# 365. Personal login history minimized.

---

# 366. Observability

Safe metrics:

```text
login success/failure aggregate
passkey adoption
refresh replay count
session revocation latency
```

---

# 367. Forbidden Metrics

No:

```text
per-user login heatmap
precise location history
social graph
message correlations
```

---

# 368. Authentication SLOs

Examples:

```text
login availability
passkey ceremony success
session refresh success
revocation propagation
```

---

# 369. Security SLO

```text
0 refresh replay accepted after detection
0 expired session accepted
```

---

# 370. Privacy SLO

```text
0 authentication identifiers in anonymous transport metadata
```

---

# 371. Failure Modes

```text
IdP outage
session DB outage
passkey provider issue
refresh-token replay
signing-key compromise
```

---

# 372. IdP Outage

Existing local/offline messaging can continue where account auth not needed.

---

# 373. Hard Rule

Core local P2P must not become unavailable solely because account IdP is down.

---

# 374. Account-Dependent Features

May degrade:

```text
backup
multi-device provisioning
managed mailbox administration
```

---

# 375. Session DB Outage

Fail closed for new/high-risk sessions.

---

# 376. Existing short access token may continue until expiry where safe.

---

# 377. Passkey Provider Issue

Alternative passkey/security key.

---

# 378. Refresh Replay

Revoke family.

---

# 379. Session Signing Key Compromise

Rotate key, invalidate affected sessions as needed.

---

# 380. Disaster Recovery

Part 70.

---

# 381. IdP backup

Logical encrypted backup.

---

# 382. No live refresh token secrets restored blindly.

---

# 383. Recovery revalidation.

---

# 384. Supply Chain

Part 73.

---

# 385. WebAuthn/OIDC/SAML libs pinned/reviewed.

---

# 386. Parser hardening.

---

# 387. Fuzzing

Important for external assertion formats.

---

# 388. Testing

Need authentication testkit.

---

# 389. Test Scenarios

```text
passkey success
wrong origin
wrong RP ID
refresh replay
expired session
MFA removal
```

---

# 390. Passkey Origin Test

Wrong origin rejected.

---

# 391. Challenge Replay Test

Rejected.

---

# 392. User Verification Test

Policy enforced.

---

# 393. Refresh Replay Test

Family revoked.

---

# 394. Session Fixation Test

Session ID rotates on login/step-up.

---

# 395. Device Binding Test

Stolen token without key rejected where binding enabled.

---

# 396. MFA Downgrade Test

High-risk account cannot silently remove strong factor.

---

# 397. Enterprise IdP Test

Same email in two orgs does not merge identity.

---

# 398. OIDC Redirect Test

Wildcard/unregistered redirect rejected.

---

# 399. PKCE Test

Required for public client.

---

# 400. Password Hash Test

Weak/legacy hash upgraded after login.

---

# 401. Enumeration Test

Login/recovery response doesn't reveal existence.

---

# 402. Rate Limit Test

Brute-force bounded.

---

# 403. Recovery Test

Recovery revokes old sessions.

---

# 404. Backup Restore Test

Refresh token/session not cloned.

---

# 405. Local-Only Mode Test

Core P2P works without IdP.

---

# 406. Privacy Test

No AccountId/login handle appears in anonymous protocol events.

---

# 407. Fuzzing

Fuzz:

```text
WebAuthn payload
OIDC token
SAML response
session token
recovery token
```

---

# 408. Property Tests

Properties:

```text
expired/revoked session never authenticates
refresh token is single-use within rotation family
managed IdP identity never merges across organizations by email
account identity never becomes transport/mailbox identity
```

---

# 409. Formal Verification Targets

Strong candidates:

```text
refresh-token rotation
session revocation
recovery/session invalidation
step-up assurance state
```

---

# 410. Kani Candidate

session/assurance transition logic.

---

# 411. TLA+ Candidate

refresh rotation + concurrent replay/revocation.

---

# 412. Loom Candidate

concurrent refresh + logout.

---

# 413. Performance

Authentication is not per-message hot path.

---

# 414. Login

Interactive.

---

# 415. Access token verification

Local/fast.

---

# 416. No remote introspection on every API request where signed short-lived token suffices.

---

# 417. Sensitive API

Can consult local revocation cache.

---

# 418. Session cache

Bounded.

---

# 419. Passkey verification

CPU reasonable.

---

# 420. Rate limiting

Protects expensive verification.

---

# 421. Cryptographic Agility

Part 66.

---

# 422. Passkey algorithms

Use standards ecosystem.

---

# 423. No custom authenticator crypto.

---

# 424. Token signatures

Algorithm registry.

---

# 425. Rotate gracefully.

---

# 426. Crate Layout

Recommended:

```text
crates/
├── siar-authn-core/
├── siar-account-identity/
├── siar-passkey/
├── siar-password-auth/
├── siar-mfa/
├── siar-session/
├── siar-authn-risk/
├── siar-managed-idp/
├── siar-account-recovery-adapter/
├── siar-authn-observability/
└── siar-authn-testkit/
```

---

# 427. `siar-authn-core`

Owns:

```text
auth methods
assurance
errors
```

---

# 428. `siar-account-identity`

Account/login-handle mapping.

---

# 429. `siar-passkey`

WebAuthn/passkey ceremonies.

---

# 430. `siar-password-auth`

Argon2id/fallback password logic.

---

# 431. `siar-mfa`

TOTP/security-key/recovery-code management.

---

# 432. `siar-session`

Access/refresh/session family lifecycle.

---

# 433. `siar-authn-risk`

Rule-based suspicious-auth signals.

---

# 434. `siar-managed-idp`

OIDC/SAML/enterprise mapping.

---

# 435. `siar-account-recovery-adapter`

Part 57 integration.

---

# 436. `siar-authn-observability`

Privacy-safe auth metrics.

---

# 437. `siar-authn-testkit`

Passkey/session/replay/recovery tests.

---

# 438. Error Taxonomy

```rust
pub enum AuthenticationError {
    InvalidCredential,
    CredentialExpired,
    CredentialRevoked,
    ChallengeInvalid,
    ChallengeReplayed,
    OriginMismatch,
    AssuranceInsufficient,
    SessionExpired,
    SessionRevoked,
    RefreshReplayDetected,
    RateLimited,
    RecoveryRequired,
    Internal,
}
```

---

# 439. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Authentication identity, messaging identity, transport identity, mailbox identity, and workload identity remain distinct.
2. Passkeys/WebAuthn are the preferred interactive authentication mechanism; custom authentication cryptography is forbidden.
3. Passwords, where supported, use modern memory-hard hashing and are never stored reversibly.
4. Root/operator/high-risk administrative accounts cannot rely on password-only or SMS-only authentication.
5. Access tokens are short-lived; refresh tokens rotate and old-token reuse triggers compromise handling.
6. Session revocation, credential changes, recovery, and device loss invalidate affected session authority.
7. Account/session identifiers never enter anonymous routing, mailbox, contact, or message protocol metadata.
8. Managed enterprise IdP subjects are organization-scoped and cannot be merged globally by email or username.
9. Local/offline P2P operation remains possible without central IdP availability when the requested feature does not require account services.
10. Account recovery does not restore old device/transport/session secrets and triggers appropriate session revocation.
11. Authentication telemetry avoids precise location history, social graph reconstruction, and user-message correlation.
12. Authentication only establishes a subject/session; authorization decisions remain Part 81 and must be evaluated separately.
```

---

# 440. Initial Production Scope

Implement first:

```text
opaque AccountId
passkey/WebAuthn primary login
multiple passkeys per account
Argon2id password fallback
TOTP + hardware-key MFA
phishing-resistant operator MFA
short-lived access tokens
rotating opaque refresh tokens
refresh replay detection
session/device revocation
step-up authentication
session security center
managed OIDC adapter
local-only/optional-account mode
privacy-safe auth metrics
authentication testkit
```

Then add:

```text
device-bound proof-of-possession sessions
enterprise SAML/SCIM adapters
advanced risk-based step-up
hardware-backed session keys
formal session/recovery verification
passkey sync/recovery UX refinements
```

---

# 441. Definition of Done

Part 82 is complete when:

- account identity is separate from communication/transport identities
- passkeys are the preferred authentication path
- password fallback is securely hashed and limited
- MFA strength/step-up requirements are explicit
- access/refresh token lifecycles are bounded
- refresh rotation and replay detection are enforced
- sessions are device-aware and individually revocable
- managed IdP subjects remain organization-scoped
- account recovery invalidates affected sessions and never restores old transport/session secrets
- local P2P can work without central IdP where appropriate
- operator authentication uses phishing-resistant strong factors
- auth telemetry avoids precise location/social graph leakage
- passkey/refresh/recovery/enterprise/fuzz/formal tests are specified

---

# 442. Final Architecture

```text
                   USER / MANAGED SUBJECT
                            │
                            ▼
                 PASSKEY / MFA / PASSWORD
                            │
                            ▼
                      IDENTITY PROVIDER
                            │
                            ▼
                 AUTHENTICATED SESSION
                            │
                            ▼
                 AUTHORIZATION (PART 81)
                            │
                            ▼
                    PROTECTED ACCOUNT ACTION
```

Account-security model:

```text
passkey-first authentication
+
phishing-resistant MFA
+
short-lived sessions
+
rotating refresh credentials
+
device-aware revocation
+
step-up for sensitive actions
+
strict separation from anonymous identity
```

not:

```text
one login account identifier reused everywhere across messaging, routing, devices, and administration
```

---

# 443. Final Principle

Authentication should prove control of an account or administrative identity without becoming the identity layer of the anonymous communication network.

The correct model is:

```text
authenticate strongly
+
bind sessions narrowly
+
rotate continuously
+
revoke quickly
+
recover safely
+
authorize separately
+
never reuse account identity as transport identity
```

This architecture gives SIAR a modern passkey-first account and administrative authentication foundation while preserving the local-first, anonymous, multi-device, tenant-isolated, and least-authority guarantees established across Parts 34–81.
