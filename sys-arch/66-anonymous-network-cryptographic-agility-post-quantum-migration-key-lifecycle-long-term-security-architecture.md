# Core System Architecture Part 66 — Anonymous Network Cryptographic Agility, Post-Quantum Migration, Key Lifecycle & Long-Term Security Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 66  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 28, 34–65  

**Primary purpose:** define SIAR's cryptographic agility, algorithm-suite abstraction, key lifecycle, hybrid post-quantum transition, long-term confidentiality, deprecation, rekeying, root/signing migration, crypto-policy governance, downgrade resistance, and implementation boundaries so cryptography can evolve without destabilizing the full system.

---

# 1. Purpose

Cryptography ages.

Algorithms can become:

```text
deprecated
weakened
misused
poorly implemented
too slow
not quantum resistant
```

A production anonymous network cannot assume today's cryptographic choices are permanent.

The governing principle is:

> **SIAR must treat cryptography as a versioned, policy-controlled capability with explicit lifecycle and migration semantics, not as hard-coded constants scattered across the codebase.**

---

# 2. Architectural Position

```text
Cryptographic Policy
        │
        ▼
Algorithm / Suite Registry
        │
        ▼
Negotiation / Selection
        │
        ▼
Protocol-Specific Crypto Adapter
        │
        ▼
Key Lifecycle
        │
        ▼
Rotation / Migration / Retirement
```

---

# 3. Core Separation

Keep distinct:

```text
algorithm
cipher suite
protocol version
key type
key epoch
key usage
key storage
key policy
```

---

# 4. Non-Goals

Part 66 does not:

```text
invent custom cryptography
guarantee quantum safety before reviewed standards/libraries exist
use one universal key across protocols
allow user-selected arbitrary algorithms in production
```

---

# 5. Cryptographic Domains

```rust
pub enum CryptoDomain {
    Transport,
    MessageE2ee,
    GroupMessaging,
    AnonymousPacket,
    Mailbox,
    ProviderSigning,
    GovernanceSigning,
    ReleaseSigning,
    Recovery,
    Backup,
    Settlement,
}
```

---

# 6. Crypto Inventory

Maintain machine-readable inventory.

```rust
pub struct CryptoInventoryEntry {
    pub domain: CryptoDomain,
    pub suite: CryptoSuiteId,
    pub key_types: Vec<CryptoKeyType>,
    pub security_level: SecurityLevel,
    pub status: CryptoSuiteStatus,
}
```

---

# 7. Suite Status

```rust
pub enum CryptoSuiteStatus {
    Experimental,
    Allowed,
    Preferred,
    Deprecated,
    Blocked,
}
```

---

# 8. Hard Rule

Blocked suites cannot be re-enabled by local operator config.

---

# 9. Algorithm Registry

```rust
pub struct CryptoSuiteDescriptor {
    pub id: CryptoSuiteId,
    pub kex: Option<KexAlgorithmId>,
    pub signature: Option<SignatureAlgorithmId>,
    pub aead: Option<AeadAlgorithmId>,
    pub hash: HashAlgorithmId,
    pub kdf: KdfAlgorithmId,
}
```

---

# 10. No Stringly-Typed Crypto

Use typed IDs.

---

# 11. Example IDs

```text
X25519
Ed25519
ChaCha20-Poly1305
SHA-256
HKDF-SHA256
```

---

# 12. Post-Quantum IDs

Future-reviewed:

```text
ML-KEM
ML-DSA
SLH-DSA
```

only where implementation maturity is acceptable.

---

# 13. No Invented PQ Primitive

Hard rule.

---

# 14. Hybrid Mode

Transition can combine:

```text
classical
+
post-quantum
```

---

# 15. Hybrid Security Goal

Security survives if at least one component remains secure.

---

# 16. Hybrid KEX

Conceptually:

```text
shared_classical
shared_pq
→ KDF combiner
→ session key
```

---

# 17. Combiner

Must use reviewed construction.

---

# 18. No Concatenate-And-Hope

Hard rule.

---

# 19. Hybrid Signature

More complicated.

May use:

```text
dual signatures
```

for critical trust objects.

---

# 20. PQ Transition Phases

```rust
pub enum PqMigrationPhase {
    ClassicalOnly,
    HybridOptional,
    HybridPreferred,
    HybridRequired,
    PostQuantumPrimary,
}
```

---

# 21. ClassicalOnly

Current baseline.

---

# 22. HybridOptional

Capability testing.

---

# 23. HybridPreferred

Use hybrid when both support.

---

# 24. HybridRequired

Security floor.

---

# 25. PostQuantumPrimary

Future.

---

# 26. Crypto Policy

```rust
pub struct CryptoPolicy {
    pub version: CryptoPolicyVersion,
    pub per_domain: BTreeMap<CryptoDomain, CryptoDomainPolicy>,
}
```

---

# 27. Crypto Policy Version

```rust
pub struct CryptoPolicyVersion(pub u64);
```

Monotonic.

---

# 28. Domain Policy

```rust
pub struct CryptoDomainPolicy {
    pub minimum_security_level: SecurityLevel,
    pub allowed_suites: BTreeSet<CryptoSuiteId>,
    pub preferred_suites: Vec<CryptoSuiteId>,
}
```

---

# 29. Policy Source

Part 53 governance.

---

# 30. Operator Cannot Lower Crypto Floor

Hard rule.

---

# 31. User Can Require Stronger Policy

Possible.

---

# 32. Negotiation

Authenticated.

---

# 33. Crypto Negotiation Input

```rust
pub struct CryptoCapabilities {
    pub suites: Vec<CryptoSuiteId>,
    pub policy_version: CryptoPolicyVersion,
}
```

---

# 34. Negotiation Rule

Select:

```text
highest mutually supported
+
locally allowed
+
policy-safe
```

suite.

---

# 35. No Plain "First Match"

Hard rule.

---

# 36. Downgrade Resistance

Negotiated suite must be included in authenticated transcript.

---

# 37. Transcript Binding

Required.

---

# 38. Strip Attack

Attacker removes strong capability.

Detection required.

---

# 39. Legacy Peer

If no safe suite:

```text
fail
```

not downgrade.

---

# 40. Hard Rule

No insecure compatibility mode.

---

# 41. Standardized Profiles

Reduce fingerprinting.

```rust
pub enum CryptoProfile {
    Baseline,
    Modern,
    Hybrid,
    Strict,
}
```

---

# 42. Why Profiles

Avoid exposing unique algorithm combinations.

---

# 43. No Arbitrary User Suite Checklist

Hard rule.

---

# 44. Crypto Key Classes

```rust
pub enum CryptoKeyType {
    AccountAuthority,
    DeviceIdentity,
    Session,
    Ratchet,
    GroupSender,
    Mailbox,
    ProviderSigning,
    GovernanceSigning,
    ReleaseSigning,
    Backup,
    Recovery,
    Settlement,
}
```

---

# 45. Different Keys For Different Roles

Hard rule.

---

# 46. No Universal Master Key

Hard rule.

---

# 47. Key Metadata

```rust
pub struct KeyMetadata {
    pub key_id: KeyId,
    pub key_type: CryptoKeyType,
    pub suite: CryptoSuiteId,
    pub epoch: KeyEpoch,
    pub state: KeyLifecycleState,
}
```

---

# 48. Key Epoch

```rust
pub struct KeyEpoch(pub u64);
```

---

# 49. Key Lifecycle State

```rust
pub enum KeyLifecycleState {
    Pending,
    Active,
    Rotating,
    Retiring,
    Revoked,
    Destroyed,
}
```

---

# 50. Key Generation

Use:

```text
OS CSPRNG
HSM RNG
reviewed library RNG
```

---

# 51. No Custom RNG

Hard rule.

---

# 52. Entropy Failure

Fatal for key generation.

---

# 53. Key Storage Classes

```rust
pub enum KeyStorageClass {
    DeviceSecureStore,
    EncryptedFile,
    Hsm,
    MemoryOnly,
    UserRecoveryProtected,
}
```

---

# 54. Memory-Only Keys

Examples:

```text
session
ephemeral KEX
```

---

# 55. Device Secure Store

Examples:

```text
device identity
local backup unlock key
```

---

# 56. HSM

Examples:

```text
governance signer
release signer
provider high-value signer
```

---

# 57. Key Serialization

Private key types should avoid generic Serialize.

---

# 58. Hard Rule

No private key in Debug/Display.

---

# 59. Secret Zeroization

Use where practical.

---

# 60. Caveat

Zeroization is not a full memory-forensics guarantee.

---

# 61. Key Lifetimes

Explicit per type.

---

# 62. Session Keys

Short-lived.

---

# 63. Ratchet Keys

Continuously evolve.

---

# 64. Group Sender Keys

Rotate on:

```text
membership changes
security event
epoch
```

---

# 65. Mailbox Keys

Medium-lived, rotatable.

---

# 66. Provider Signing Keys

Longer-lived but rotated periodically.

---

# 67. Governance Root

Very long-lived, offline, rarely used.

---

# 68. Long-Lived Key Risk

Higher compromise impact.

---

# 69. Key Rotation Policy

```rust
pub struct KeyRotationPolicy {
    pub key_type: CryptoKeyType,
    pub max_age: Duration,
    pub overlap: Duration,
    pub compromise_action: CompromiseAction,
}
```

---

# 70. Rotation Pattern

```text
generate next
→ publish next
→ overlap
→ prefer next
→ retire old
→ destroy
```

---

# 71. Compromise Pattern

```text
revoke old
→ generate new
→ propagate urgently
```

---

# 72. No Normal Overlap After Confirmed Compromise

Hard rule.

---

# 73. Key Revocation

Signed where applicable.

---

# 74. Revocation Record

```rust
pub struct KeyRevocation {
    pub key_id: KeyId,
    pub effective_epoch: KeyEpoch,
    pub reason: KeyRevocationReason,
}
```

---

# 75. Revocation Reason

```rust
pub enum KeyRevocationReason {
    Compromised,
    Superseded,
    Retired,
    AlgorithmDeprecated,
}
```

---

# 76. Anti-Rollback

Revoked key cannot return after restore.

---

# 77. Hard Rule.

---

# 78. Forward Secrecy

Required where protocol supports.

---

# 79. Message Sessions

Use ratcheting/proven protocol.

---

# 80. Past Session Confidentiality

Compromise of current state should not reveal all old messages.

---

# 81. Post-Compromise Security

Desired.

---

# 82. Group Messaging

More complex.

---

# 83. Sender-Key Model

Needs epoch/rekey.

---

# 84. PQ Forward Secrecy

Long-term concern.

---

# 85. Harvest-Now-Decrypt-Later

Threat:

```text
attacker records ciphertext today
breaks classical key exchange later
```

---

# 86. Priority Domains For PQ

Highest priority:

```text
long-lived private messages
backups
anonymous mailbox payloads
sensitive group history
```

---

# 87. Lower Priority

Short-lived low-sensitivity control data may migrate later.

---

# 88. Crypto Migration Priority

```rust
pub enum MigrationPriority {
    Immediate,
    High,
    Normal,
    Low,
}
```

---

# 89. Long-Term Confidentiality Class

```rust
pub enum ConfidentialityHorizon {
    Ephemeral,
    Short,
    Medium,
    LongTerm,
}
```

---

# 90. Crypto Policy Can Depend On Horizon

---

# 91. Backup Encryption

Long-term.

---

# 92. Therefore

PQ migration should prioritize backup key wrapping/envelope when mature.

---

# 93. Stored Ciphertext

Question:

```text
must old stored ciphertext be re-encrypted?
```

---

# 94. Answer

Depends on threat and key exposure.

---

# 95. Re-encryption Strategy

```rust
pub enum ReencryptionStrategy {
    None,
    RewrapKey,
    Lazy,
    Background,
    MandatoryImmediate,
}
```

---

# 96. Rewrap Key

If data encrypted under random DEK:

```text
keep ciphertext
rewrap DEK under new KEK
```

---

# 97. Efficient

Preferred where possible.

---

# 98. Full Re-encryption

Needed if content cipher itself deprecated.

---

# 99. Lazy Migration

On read/write.

---

# 100. Background Migration

Batch.

---

# 101. Immediate

Only for severe compromise.

---

# 102. Re-encryption Must Preserve Integrity

Hard rule.

---

# 103. Storage Crypto Envelope

```rust
pub struct EncryptedObjectEnvelope {
    pub data_suite: CryptoSuiteId,
    pub wrapped_key_suite: CryptoSuiteId,
    pub wrapped_key: Bytes,
    pub ciphertext: Bytes,
}
```

---

# 104. Envelope Benefit

Independent data/key algorithm migration.

---

# 105. Key Encryption Key

Separate from data encryption key.

---

# 106. No Same Key For All Stored Objects

Hard rule.

---

# 107. Per-Object/Per-Batch DEK

Recommended.

---

# 108. Attachment Encryption

Random per-attachment key.

---

# 109. Backup Key Hierarchy

```text
root recovery key
→ backup KEK
→ object DEKs
```

---

# 110. Avoid Huge Blast Radius

Hard rule.

---

# 111. Crypto Agility At Wire Layer

Versioned envelope.

---

# 112. Example

```rust
pub struct CryptoEnvelope {
    pub suite: CryptoSuiteId,
    pub key_epoch: KeyEpoch,
    pub payload: Bytes,
}
```

---

# 113. Bounded Suite Registry

Reject unknown.

---

# 114. No Dynamic Plugin Crypto

Hard rule for production protocol.

---

# 115. Why

Unreviewed algorithms are unsafe.

---

# 116. External Crypto Provider

Possible internal abstraction.

---

# 117. Trait

```rust
pub trait CryptoProvider {
    fn supports(&self, suite: CryptoSuiteId) -> bool;
}
```

---

# 118. Protocol Adapter

Each protocol asks provider for approved primitive.

---

# 119. No Primitive Leakage Into Domain Layer

Preferred.

---

# 120. Example

Messaging domain should request:

```text
MessageSessionCrypto
```

not hard-code a crate everywhere.

---

# 121. Typed Crypto Capability

```rust
pub struct ApprovedCryptoSuite<TDomain> {
    pub id: CryptoSuiteId,
    _marker: PhantomData<TDomain>,
}
```

---

# 122. Compile-Time Assistance

Prevents using:

```text
backup suite
```

for:

```text
governance signing
```

by mistake.

---

# 123. Crypto Backend

Could be:

```text
RustCrypto
ring-like backend
HSM-backed signer
platform secure store
```

---

# 124. Pure Rust Preference

Use pure Rust where secure/mature.

---

# 125. HSM Exceptions

Allowed for high-value signing.

---

# 126. FFI

If required by HSM/vendor, isolate behind narrow boundary.

---

# 127. Unsafe Review

Mandatory.

---

# 128. Cryptographic Library Policy

Requirements:

```text
actively maintained
reviewed
constant-time where applicable
memory-safe where possible
```

---

# 129. Dependency Pinning

Part 62.

---

# 130. Crypto Crate Upgrade

High-risk change.

---

# 131. Requires

```text
known-answer tests
interop tests
benchmarks
security review
```

---

# 132. Known-Answer Tests

KATs.

---

# 133. Protocol Golden Vectors

Part 64.

---

# 134. Crypto Test Vectors

Use official/upstream vectors.

---

# 135. No Handwritten Expected Values Without Source

Hard rule.

---

# 136. Constant-Time Requirements

Secret-dependent operations.

---

# 137. Timing Side Channel

Test/review.

---

# 138. Avoid Secret-Dependent Branches

Where applicable.

---

# 139. Side-Channel Scope

Also includes:

```text
cache
memory access
error differences
```

---

# 140. Error Normalization

Do not reveal:

```text
valid username
valid key
valid recipient
```

through crypto errors.

---

# 141. Signature Verification

Reject non-canonical forms where algorithm requires.

---

# 142. Public-Key Validation

Mandatory.

---

# 143. Key Substitution

Bind keys to:

```text
role
domain
identity
transcript
```

---

# 144. Cross-Protocol Attack

Prevent by domain separation.

---

# 145. Domain Separation

Use explicit context strings/labels.

---

# 146. Example

```text
SIAR-MSG-KDF-v1
SIAR-BACKUP-KDF-v1
SIAR-GOV-SIGN-v1
```

---

# 147. No Key Reuse Across Domains

Hard rule.

---

# 148. KDF Labels

Versioned.

---

# 149. Hash Agility

Needed but cautious.

---

# 150. Hash Migration

Can be:

```text
dual digest
new digest
```

---

# 151. Content Addressing

Hash algorithm embedded in identifier.

---

# 152. Multihash-Like Structure

Useful.

---

# 153. Example

```rust
pub struct ContentDigest {
    pub algorithm: HashAlgorithmId,
    pub digest: Bytes,
}
```

---

# 154. No Bare 32-Byte Hash Ambiguity

Hard rule.

---

# 155. Signature Algorithm Migration

Trust roots complex.

---

# 156. Root Rollover

Part 53.

---

# 157. Hybrid Root Transition

Possible:

```text
old classical root signs new hybrid root bundle
```

---

# 158. New Root Bundle

May include:

```text
classical key
PQ key
```

---

# 159. Dual Verification

During transition.

---

# 160. Governance Signature Policy

Could require:

```text
classical + PQ
```

for critical decisions later.

---

# 161. Threshold + PQ

Complex.

---

# 162. Recommendation

Do not combine novel threshold-PQ construction prematurely.

---

# 163. Initial Future Strategy

Use multiple independent ordinary signatures.

---

# 164. Release Signing

Long-lived trust concern.

---

# 165. Release Manifest

Can carry multiple signatures.

---

# 166. Example

```rust
pub struct ReleaseSignatureSet {
    pub signatures: Vec<AlgorithmBoundSignature>,
}
```

---

# 167. Signature Bound To Algorithm ID

Required.

---

# 168. Provider Signing

Rotation easier.

---

# 169. Device Identity Signing

Frequent enough to migrate.

---

# 170. Account Authority

More sensitive due continuity.

---

# 171. Account Key Migration

Use transition certificate.

---

# 172. Device Key Migration

Fresh device authorization.

---

# 173. Relationship Identity

Should not depend directly on one algorithm public key.

---

# 174. Stable Logical Identity

Can reference key history.

---

# 175. Key History

Authenticated chain.

---

# 176. Identity Key Record

```rust
pub struct IdentityKeyRecord {
    pub epoch: KeyEpoch,
    pub public_keys: Vec<AlgorithmBoundPublicKey>,
    pub transition_proof: Option<KeyTransitionProof>,
}
```

---

# 177. No Raw Key = Permanent Account ID

Hard rule where possible.

---

# 178. Mailbox Crypto Migration

New mailbox epoch.

---

# 179. Old Mailbox

Dual receive only if not compromised.

---

# 180. Ratchet Migration

Cannot casually switch primitive mid-session.

---

# 181. Recommended

Start new session epoch.

---

# 182. Session Migration

```text
authenticate old session
→ negotiate new suite
→ establish fresh session
→ retire old
```

---

# 183. No State Splicing

Hard rule.

---

# 184. Group Crypto Migration

Create new group crypto epoch.

---

# 185. Members Must Support New Minimum

If strict.

---

# 186. Otherwise

Group may remain on old safe suite temporarily.

---

# 187. Explicit Policy

No silent mixed-security state.

---

# 188. Anonymous Packet Crypto

Sphinx/Nym provider-specific.

---

# 189. Migration

Follow reviewed protocol/provider evolution.

---

# 190. No Homegrown PQ Sphinx Variant

Hard rule.

---

# 191. Transport QUIC/TLS

Use platform/library-supported secure versions.

---

# 192. PQ TLS

Adopt when mature in chosen stack.

---

# 193. Application E2EE

Still independent.

---

# 194. Defense In Depth

Transport PQ does not replace app E2EE PQ planning.

---

# 195. Key Compromise Categories

```rust
pub enum KeyCompromiseClass {
    Session,
    Device,
    Account,
    Provider,
    Governance,
    Release,
    Recovery,
}
```

---

# 196. Different Response

Per class.

---

# 197. Session Compromise

Rekey/restart session.

---

# 198. Device Compromise

Revoke device.

---

# 199. Account Compromise

Account authority rotation/recovery.

---

# 200. Provider Compromise

Revoke provider key.

---

# 201. Governance Compromise

Part 53 emergency recovery.

---

# 202. Release Key Compromise

Block manifests signed after/with compromised key as policy dictates.

---

# 203. Recovery Key Compromise

Rotate recovery package/epoch.

---

# 204. Crypto Incident Playbooks

Part 65.

---

# 205. Mandatory Playbooks

```text
signing key compromise
algorithm deprecation
root rollover
PQ migration
randomness failure
```

---

# 206. Randomness Incident

Critical.

---

# 207. Entropy Health

OS responsibility largely.

---

# 208. But

Startup can validate CSPRNG availability.

---

# 209. No Homegrown Entropy Estimator

Hard rule.

---

# 210. Secure Deletion

Key destruction.

---

# 211. Storage Caveat

Flash/SSD secure erase not guaranteed.

---

# 212. Therefore

Cryptographic erasure preferred.

---

# 213. Cryptographic Erasure

Destroy KEK/DEK.

---

# 214. Still

Backups/replicas must respect lifecycle.

---

# 215. Backup Key Retirement

May intentionally make old backup unreadable.

---

# 216. Policy

Explicit.

---

# 217. Long-Term Archive

May require periodic re-encryption.

---

# 218. Archive Crypto Review

Scheduled.

---

# 219. Crypto Review Interval

```rust
pub struct CryptoReviewPolicy {
    pub interval: Duration,
    pub horizon: ConfidentialityHorizon,
}
```

---

# 220. Algorithm Deprecation

Lifecycle:

```text
Allowed
→ PreferredAlternativeAvailable
→ Deprecated
→ DisabledForNew
→ Blocked
```

---

# 221. New Connections

Stop using deprecated suite first.

---

# 222. Existing Sessions

May drain.

---

# 223. Stored Data

Migration plan separate.

---

# 224. Emergency Block

Immediate if broken.

---

# 225. Crypto Deprecation Record

```rust
pub struct CryptoDeprecation {
    pub suite: CryptoSuiteId,
    pub effective_at: Timestamp,
    pub action: DeprecationAction,
    pub policy_version: CryptoPolicyVersion,
}
```

---

# 226. Deprecation Action

```rust
pub enum DeprecationAction {
    Warn,
    DisableNew,
    RequireMigration,
    Block,
}
```

---

# 227. Crypto Agility Compatibility

Part 52.

---

# 228. Mixed-Version Fleet

Must understand:

```text
suite support
policy floor
migration stage
```

---

# 229. Provider Capability Advertisement

Can advertise standardized crypto profile.

---

# 230. Avoid Full Algorithm Fingerprint

Use profile where possible.

---

# 231. Client Fingerprinting

Unique suite sets are bad.

---

# 232. Hard Rule

Do not expose experimental per-user algorithm combinations to remote peers in production.

---

# 233. PQ Performance

Potentially larger:

```text
public keys
ciphertexts
signatures
```

---

# 234. Benchmark

Part 63.

---

# 235. Capacity Impact

Model:

```text
bandwidth
memory
CPU
fragmentation
```

---

# 236. Anonymous Packet Impact

Large PQ material may increase fingerprinting.

---

# 237. Padding Classes

May need update.

---

# 238. No Unreviewed Variable-Size Fingerprint

Hard rule.

---

# 239. Storage Impact

PQ signatures may increase:

```text
governance log
release manifest
```

acceptable.

---

# 240. Mobile Impact

CPU/battery benchmark.

---

# 241. Hardware Acceleration

May differ.

---

# 242. Crypto Policy Must Not Depend On Device Brand

Hard rule.

---

# 243. Weak Device

If cannot meet mandatory suite:

```text
feature unavailable
```

not downgrade below floor.

---

# 244. Crypto Feature Availability

Explicit UI.

---

# 245. No User Blame

Just explain compatibility.

---

# 246. PQ Rollout Testing

Need:

```text
interop
performance
fragmentation
battery
failure
```

---

# 247. Harvest-Now-Decryption Test Model

Cannot test future quantum break directly.

---

# 248. Instead

Verify long-horizon data uses approved hybrid/PQ profile once required.

---

# 249. Crypto Compliance Scanner

Static inventory.

---

# 250. Scanner Finds

```text
hard-coded algorithm
deprecated suite
key reuse
unsafe hash
```

---

# 251. Code Search Is Not Enough

Use typed APIs.

---

# 252. Approved Crypto Provider

Central factory.

---

# 253. Example

```rust
pub trait ApprovedCryptoProvider {
    fn message_suite(
        &self,
        policy: &CryptoPolicy,
    ) -> Result<MessageCryptoSuite, CryptoError>;
}
```

---

# 254. No Direct Primitive Construction In Business Crates

Preferred hard rule.

---

# 255. Exceptions

Low-level crypto adapter crates only.

---

# 256. Crate Boundary

```text
domain/application
    ↓
crypto interface
    ↓
crypto adapter
    ↓
reviewed library
```

---

# 257. Crypto Adapter Crates

Few and heavily reviewed.

---

# 258. Unsafe/FFI

Contained there.

---

# 259. Crypto Audit

Focused.

---

# 260. Test Vectors

Mandatory.

---

# 261. Key Type Phantom Marker

```rust
pub struct SecretKey<TUsage> {
    bytes: SecretBytes,
    _usage: PhantomData<TUsage>,
}
```

---

# 262. Prevent Accidental Cross-Use

Compile-time help.

---

# 263. Public Key Wrapper

```rust
pub struct PublicKey<TUsage> {
    bytes: Bytes,
    _usage: PhantomData<TUsage>,
}
```

---

# 264. Algorithm Bound Key

```rust
pub struct AlgorithmBoundPublicKey<TUsage> {
    pub algorithm: PublicKeyAlgorithmId,
    pub key: PublicKey<TUsage>,
}
```

---

# 265. Key Derivation

Explicit context.

---

# 266. Derived Key Type

```rust
pub struct DerivedKey<TPurpose> {
    key: SecretBytes,
    _purpose: PhantomData<TPurpose>,
}
```

---

# 267. Key Usage Policy

```rust
pub trait KeyUsagePolicy<TPurpose> {
    fn validate(
        &self,
        metadata: &KeyMetadata,
    ) -> Result<(), CryptoError>;
}
```

---

# 268. Crypto State Store

Stores:

```text
public key history
epochs
revocations
policy versions
```

---

# 269. Private Key Store

Separate.

---

# 270. No Private Key In General DB

Hard rule for high-value keys.

---

# 271. Device Keys

Encrypted local store.

---

# 272. Session Keys

Memory.

---

# 273. Provider Keys

Secret store/HSM.

---

# 274. Root Keys

Offline.

---

# 275. Crypto Observability

Safe:

```text
suite profile
rotation success
deprecated suite count
```

---

# 276. Forbidden Metrics

No:

```text
raw public key
key ID linked to user
session key age per user
```

---

# 277. Aggregate Suite Adoption

Infrastructure/profile only.

---

# 278. Maximum Anonymity

No per-client crypto profile telemetry.

---

# 279. Crypto Incident Metrics

Infrastructure only.

---

# 280. Crypto SLOs

Examples:

```text
100% new sessions on approved suite
0 blocked suite use
rotation propagation within target
```

---

# 281. Long-Term Security SLO

No long-horizon data on deprecated profile beyond migration deadline.

---

# 282. Crypto Migration Planner

```rust
pub trait CryptoMigrationPlanner {
    fn plan(
        &self,
        from: CryptoPolicyVersion,
        to: CryptoPolicyVersion,
        inventory: &CryptoInventory,
    ) -> Result<CryptoMigrationPlan, CryptoError>;
}
```

---

# 283. Migration Plan

```rust
pub struct CryptoMigrationPlan {
    pub phases: Vec<CryptoMigrationPhase>,
    pub affected_domains: BTreeSet<CryptoDomain>,
    pub reencryption: Vec<ReencryptionTask>,
}
```

---

# 284. Migration Phase

```rust
pub enum CryptoMigrationPhase {
    Introduce,
    DualSupport,
    PreferNew,
    RequireNew,
    RetireOld,
}
```

---

# 285. Rollback

Before irreversible state.

---

# 286. After key destruction

Cannot rollback.

---

# 287. Hard Rule

Migration plan must mark irreversible steps.

---

# 288. PQ Migration Timeline

Not hardcoded.

---

# 289. Triggered by

```text
standards maturity
library maturity
interop
performance
governance
threat horizon
```

---

# 290. No Hype-Driven Migration

Hard rule.

---

# 291. No "Wait Until Broken"

Also hard rule.

---

# 292. Crypto Advisory Intake

Track upstream security advisories.

---

# 293. Dependency Watch

Part 51/65.

---

# 294. Crypto Risk Register

```rust
pub struct CryptoRiskEntry {
    pub suite: CryptoSuiteId,
    pub risk: CryptoRiskClass,
    pub action: CryptoRiskAction,
}
```

---

# 295. Risk Classes

```rust
pub enum CryptoRiskClass {
    Informational,
    Watch,
    Elevated,
    Critical,
}
```

---

# 296. Crypto Testkit

Need dedicated.

---

# 297. Test Classes

```text
KAT
interop
property
fuzz
migration
rollback
compromise
performance
```

---

# 298. Known Answer Tests

Every primitive adapter.

---

# 299. Cross-Version Interop

Current↔previous suite profile.

---

# 300. Negotiation Downgrade Test

Attacker strips strongest capability.

Must fail/detect.

---

# 301. Unknown Suite Test

Reject.

---

# 302. Deprecated Suite Test

No new session.

---

# 303. Blocked Suite Test

Reject even operator override.

---

# 304. Key Rotation Test

Old/new overlap.

---

# 305. Key Compromise Test

Old immediately invalid.

---

# 306. Key Restore Test

Revoked key not resurrected.

---

# 307. Backup Rewrap Test

Ciphertext remains decryptable under new KEK.

---

# 308. Full Re-encryption Test

Integrity preserved.

---

# 309. PQ Hybrid Test

Both components contribute according reviewed combiner.

---

# 310. Hybrid Degradation Test

One component failure modeled safely.

---

# 311. Signature Dual-Verification Test

Policy semantics correct.

---

# 312. Account Transition Test

Identity continuity preserved.

---

# 313. Group Migration Test

New crypto epoch.

---

# 314. Mailbox Migration Test

New mailbox key epoch.

---

# 315. Federation Crypto Profile Test

Foreign domain below floor rejected.

---

# 316. Fragmentation Test

Larger PQ artifacts remain bounded.

---

# 317. Fuzzing

Fuzz:

```text
crypto envelope
algorithm IDs
public keys
signatures
key transition records
```

---

# 318. Property Tests

Properties:

```text
blocked suite never selected
revoked key never becomes active again
cross-domain key type cannot satisfy wrong usage
new policy floor is monotonic unless explicit higher-version governance decision changes it safely
```

---

# 319. Formal Verification Targets

Strong candidates:

```text
suite negotiation
key lifecycle
anti-rollback
migration state machine
```

---

# 320. TLA+ Candidate

Mixed-fleet crypto migration.

---

# 321. Kani Candidate

suite-selection/downgrade logic.

---

# 322. Loom Candidate

concurrent key rotation/session creation.

---

# 323. Performance Tests

Measure:

```text
KEX
signature
AEAD
ratchet step
PQ hybrid handshake
```

---

# 324. Benchmark Profiles

```text
desktop
mobile
server
```

---

# 325. Memory Tests

PQ key/signature sizes.

---

# 326. Wire Size Tests

Padding/fragmentation impact.

---

# 327. Battery Tests

Android.

---

# 328. Release Gate

Crypto changes are high-risk.

---

# 329. Required Gates

```text
KAT
interop
fuzz
security review
performance
migration test
```

---

# 330. External Review

Recommended for major algorithm/suite introduction.

---

# 331. Crate Layout

Recommended:

```text
crates/
├── siar-crypto-core/
├── siar-crypto-registry/
├── siar-crypto-policy/
├── siar-crypto-provider/
├── siar-key-lifecycle/
├── siar-key-store/
├── siar-crypto-negotiation/
├── siar-crypto-migration/
├── siar-pq-hybrid/
├── siar-crypto-observability/
└── siar-crypto-testkit/
```

---

# 332. `siar-crypto-core`

Owns:

```text
suite IDs
algorithm IDs
key types
errors
```

---

# 333. `siar-crypto-registry`

Approved suite inventory.

---

# 334. `siar-crypto-policy`

Allowed/preferred/blocked rules.

---

# 335. `siar-crypto-provider`

Protocol-facing crypto abstraction.

---

# 336. `siar-key-lifecycle`

Generation/rotation/revocation/destruction.

---

# 337. `siar-key-store`

Device/file/HSM backends.

---

# 338. `siar-crypto-negotiation`

Authenticated suite selection.

---

# 339. `siar-crypto-migration`

Phased migrations/re-encryption.

---

# 340. `siar-pq-hybrid`

Reviewed hybrid adapters only.

---

# 341. `siar-crypto-observability`

Privacy-safe crypto metrics.

---

# 342. `siar-crypto-testkit`

KAT/interoperability/migration/failure tests.

---

# 343. Error Taxonomy

```rust
pub enum CryptoError {
    SuiteUnsupported,
    SuiteBlocked,
    SecurityFloorUnsatisfied,
    NegotiationDowngrade,
    KeyRevoked,
    KeyExpired,
    KeyUsageInvalid,
    MigrationRequired,
    KeyStoreUnavailable,
    RandomnessUnavailable,
    SignatureInvalid,
    Internal,
}
```

---

# 344. Security Invariants

Mandatory:

```text
1. No production protocol invents custom cryptography.
2. Crypto algorithms/suites are centrally registered, typed, versioned, and policy-controlled.
3. Blocked suites cannot be re-enabled by operator config.
4. Negotiation is authenticated and downgrade-resistant.
5. Keys are domain/usage-separated; no universal master key exists.
6. Revoked keys and deprecated key epochs cannot be resurrected by backup/restore.
7. Long-term data can migrate encryption independently from transport/session crypto.
8. Recovery never restores stale ratchet/session/reply secrets.
9. PQ migration uses reviewed standardized constructions and phased hybrid deployment.
10. Unique per-user experimental crypto combinations are not exposed as remote fingerprints.
11. High-value root/governance/release keys use stronger storage and operational controls than routine session keys.
12. Crypto migrations mark irreversible steps and preserve auditability.
```

---

# 345. Initial Production Scope

Implement first:

```text
central crypto registry
typed suite/algorithm IDs
per-domain crypto policy
authenticated suite negotiation
key-type separation
key epochs
rotation/revocation lifecycle
device secure-store + HSM abstractions
storage crypto envelope with DEK/KEK separation
crypto inventory scanner
KAT + golden vectors
migration state machine
deprecation/block policy
```

Prepare now for PQ by:

```text
keeping algorithm IDs explicit
supporting multi-key identity records
supporting multiple signatures
making envelope formats suite-aware
avoiding fixed key/signature sizes
designing hybrid migration phases
```

Then add, when standards/libraries are production-ready:

```text
hybrid classical+PQ KEX
dual/hybrid signatures for critical trust objects
PQ-aware backup wrapping
PQ-capable device/account identity transition
PQ transport where supported
formal migration verification
```

---

# 346. Definition of Done

Part 66 is complete when:

- every cryptographic use is assigned to an explicit domain
- algorithm/suite IDs are typed and versioned
- crypto policy defines allowed/preferred/deprecated/blocked suites
- negotiation is authenticated and downgrade-resistant
- key types/lifetimes/storage/rotation/revocation are explicit
- storage encryption supports independent DEK/KEK migration
- root/provider/device/account/session/group/mailbox keys have distinct lifecycles
- long-term confidentiality requirements drive migration priority
- PQ migration phases are defined without inventing new cryptography
- release/governance signing can support multiple algorithms/signatures
- identity continuity is separated from one permanent algorithm key
- migration/rollback/irreversible boundaries are explicit
- KAT, interop, fuzz, downgrade, compromise, restore, PQ, performance, and formal tests are specified

---

# 347. Final Architecture

```text
                      GOVERNANCE CRYPTO POLICY
                               │
                               ▼
                        CRYPTO REGISTRY
                               │
                   ┌───────────┴───────────┐
                   │                       │
             Classical Suites        PQ/Hybrid Suites
                   │                       │
                   └───────────┬───────────┘
                               ▼
                    AUTHENTICATED NEGOTIATION
                               │
                               ▼
                        PROTOCOL ADAPTERS
                               │
                               ▼
                         KEY LIFECYCLE
                               │
              ┌────────────────┼────────────────┐
              │                │                │
           Rotate           Revoke          Migrate
              │                │                │
              └────────────────┼────────────────┘
                               ▼
                    LONG-TERM SECURITY STATE
```

Cryptographic agility model:

```text
typed suite registry
+
policy-controlled negotiation
+
domain-separated keys
+
explicit key lifecycle
+
phased migration
+
reviewed PQ transition
```

not:

```text
hard-coded algorithms scattered across every crate
```

---

# 348. Final Principle

Cryptographic agility is not the ability to switch algorithms casually.

It is the ability to **change cryptography safely, deliberately, and audibly without breaking identity, privacy, interoperability, or long-lived data**.

The correct model is:

```text
central inventory
+
typed usage boundaries
+
authenticated negotiation
+
key lifecycle
+
migration planning
+
conservative PQ adoption
```

This architecture gives SIAR a path from today's classical cryptography toward future hybrid and post-quantum security while preserving the anonymous-network guarantees established across Parts 34–65.
