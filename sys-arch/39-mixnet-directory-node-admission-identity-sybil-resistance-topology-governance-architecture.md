# Core System Architecture Part 39 — Mixnet Directory, Node Admission, Identity, Sybil Resistance & Topology Governance Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 39  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:**  
- Part 34 — Mixnet, Loopix, Sphinx, Nym & High-Anonymity Transport Architecture  
- Part 35 — Anonymous Mailbox, Offline Receiving & Reply Capability Architecture  
- Part 36 — Sphinx Packet, Cell, Fragmentation & Anonymous Message Framing Architecture  
- Part 37 — Cover Traffic, Traffic Shaping, Timing Obfuscation & Loop Traffic Architecture  
- Part 38 — Native Loopix-Inspired Mixnet Topology, Mix Nodes, Layering & Packet Forwarding Architecture  

**Primary purpose:** define the complete control-plane trust architecture for SIAR's native mixnet, including directory authorities, node and operator identity, node admission, Sybil resistance, topology eligibility, health evidence, layer assignment, concentration limits, epoch signing, emergency revocation, governance, auditability, decentralization, and Rust service boundaries.

---

# 1. Purpose

A mixnet is only as strong as the topology it allows clients to use.

If one adversary can cheaply create many nodes, that adversary may control:

```text
entry gateways
multiple mix layers
mailbox gateways
directory views
routing concentration
```

and significantly weaken anonymity.

The governing principle is:

> **SIAR must make it difficult for one real-world operator to appear as many independent anonymity participants, while keeping admission auditable and avoiding unnecessary identity exposure.**

---

# 2. Architectural Position

```text
Node Operator
    │
    ▼
Node Registration
    │
    ▼
Identity / Admission Validation
    │
    ▼
Sybil Resistance / Eligibility
    │
    ▼
Directory Authorities
    │
    ▼
Topology Builder
    │
    ▼
Signed Epoch Snapshot
    │
    ▼
Clients / Mix Nodes
```

---

# 3. Data Plane vs Governance Plane

Part 38 data plane:

```text
packet forwarding
delay
mixing
mailbox delivery
```

Part 39 control/governance plane:

```text
who may participate
which role they may serve
which layer they occupy
whether they are healthy
whether they are revoked
who signs topology state
```

These must remain separate.

---

# 4. Core Threats

Part 39 addresses:

```text
Sybil attacks
operator concentration
fake node registration
stolen node identity
malicious directory authority
biased topology construction
health-report manipulation
fake geographic/operator metadata
revoked-node persistence
control-plane split brain
governance capture
```

---

# 5. Non-Goals

Part 39 does not fully define:

```text
economic tokenomics
cryptocurrency
anonymous payments
bridges/censorship resistance
formal anonymity proofs
full public DAO governance
```

Those can be later layers.

---

# 6. Identity Layers

Use distinct identity domains:

```text
Node Identity
Operator Identity
Directory Authority Identity
Topology Signer Identity
Administrative Identity
```

---

# 7. Node Identity

Each node has a cryptographic identity:

```rust
pub struct MixNodeId(pub [u8; 32]);
```

Derived from or bound to a node signing key.

---

# 8. Node Identity Key

Used to sign:

```text
registration
capability claims
health reports
key rotations
leave requests
```

---

# 9. Operator Identity

A single operator may run multiple nodes.

```rust
pub struct OperatorId(pub [u8; 32]);
```

---

# 10. Why Operator Identity Matters

If:

```text
Node A
Node B
Node C
```

are all controlled by one organization, they must not be counted as three independent anonymity domains.

---

# 11. Operator Identity Privacy

Operator identity is governance metadata.

Do not require public exposure of private personal details.

---

# 12. Operator Registration

Can bind:

```text
organization credential
verified infrastructure domain
legal entity where appropriate
community attestation
resource proof
```

depending deployment model.

---

# 13. Directory Authority Identity

```rust
pub struct DirectoryAuthorityId(pub [u8; 32]);
```

Authorities sign:

```text
node registry state
revocation state
topology epochs
policy versions
```

---

# 14. Authority Separation

Directory authority should not itself forward user packets by default.

This reduces trust concentration.

---

# 15. Initial Trust Model

Practical early deployment:

```text
1 authoritative SIAR directory
+
auditable signed snapshots
+
separate topology builder
```

This is centralized but operationally simple.

---

# 16. Target Trust Model

Long-term:

```text
multiple independent directory authorities
+
threshold signatures
+
auditable deterministic topology rules
```

---

# 17. Threshold Signatures

Conceptual:

```text
M of N authorities
```

must approve topology epoch.

---

# 18. Why Threshold Signing

Reduces risk from:

```text
single compromised directory
single malicious operator
single administrative mistake
```

---

# 19. Directory Quorum

```rust
pub struct DirectoryQuorum {
    pub threshold: u16,
    pub total: u16,
}
```

---

# 20. Node Registration Lifecycle

```text
Unregistered
→ Pending
→ Verified
→ Eligible
→ Active
→ Draining
→ Retired
```

---

# 21. Node Status

```rust
pub enum NodeAdmissionState {
    Pending,
    Verified,
    Eligible,
    Active,
    Suspended,
    Revoked,
    Retired,
}
```

---

# 22. Registration Request

```rust
pub struct NodeRegistrationRequest {
    pub node_id: MixNodeId,
    pub operator_id: OperatorId,
    pub requested_roles: Vec<MixnetNodeRole>,
    pub endpoints: Vec<NodeEndpoint>,
    pub public_keys: MixNodePublicKeys,
    pub capabilities: MixNodeCapabilities,
    pub metadata: NodeAdmissionMetadata,
    pub proof: AdmissionProof,
}
```

---

# 23. Admission Metadata

Possible:

```text
ASN
region
hosting provider
capacity class
software version
operator grouping
```

---

# 24. Metadata Minimization

Collect only metadata needed for:

```text
diversity
security
operations
```

---

# 25. Node Endpoint Validation

Verify node actually controls endpoint.

Methods:

```text
challenge-response
TLS/QUIC challenge
signed nonce
```

---

# 26. Key Possession

Registration requires proof of possession of node private key.

---

# 27. Operator Binding

Node must prove authorization from operator identity.

---

# 28. Admission Proof Types

```rust
pub enum AdmissionProof {
    OperatorSigned,
    ResourceBonded,
    AuthorityApproved,
    CommunityAttested,
    Hybrid,
}
```

---

# 29. Sybil Resistance Principle

No single mechanism is perfect.

Use layered defenses.

---

# 30. Sybil Resistance Layers

Potential:

```text
operator identity
resource cost
capacity proof
infrastructure diversity
admission review
health history
rate limits
topology concentration caps
```

---

# 31. Resource Cost

Make node creation non-free.

Could be:

```text
bond
deposit
proof of resource ownership
operational history
```

---

# 32. Avoid Assuming Stake = Independence

A wealthy attacker may still control many nodes.

Therefore stake/resource proof is only one factor.

---

# 33. Operator Concentration Cap

Topology policy should enforce:

```text
maximum nodes/hops per operator
```

---

# 34. Per-Route Rule

Strict mode:

```text
one operator max per route
```

---

# 35. Per-Layer Rule

Limit operator share per layer.

---

# 36. Network-Wide Rule

Limit operator share of total eligible capacity.

---

# 37. Hosting Provider Concentration

Operator diversity alone is insufficient if all nodes run on one cloud.

---

# 38. Cloud/ASN Concentration

Track:

```text
ASN
hosting provider
region
```

---

# 39. Independence Dimensions

```rust
pub struct DiversityDimensions {
    pub operator: bool,
    pub asn: bool,
    pub provider: bool,
    pub region: bool,
}
```

---

# 40. Sybil Risk Score

Optional internal control-plane score.

```rust
pub struct SybilRiskScore(pub u16);
```

---

# 41. No User-Facing "Anonymity Score"

Hard rule.

---

# 42. Risk Inputs

Possible:

```text
operator overlap
shared ASN
shared IP prefix
shared certificate patterns
shared infrastructure fingerprints
registration timing
ownership evidence
```

---

# 43. Risk Score Limit

Do not treat heuristic score as proof.

---

# 44. Capacity Proof

Node should demonstrate claimed throughput.

---

# 45. Benchmark Challenge

Directory may issue synthetic load challenge.

---

# 46. Capacity Class

```rust
pub enum CapacityClass {
    Small,
    Medium,
    Large,
    VeryLarge,
}
```

---

# 47. Capacity Weighting

Route/topology selection may use capacity weighting.

But weight is capped to prevent dominant nodes.

---

# 48. Maximum Weight Fraction

```rust
pub struct CapacityWeightPolicy {
    pub max_node_fraction: f32,
    pub max_operator_fraction: f32,
}
```

---

# 49. Health History

Eligibility should consider sustained behavior.

---

# 50. Health Evidence

Examples:

```text
uptime
packet-forward success
latency class
queue health
software freshness
```

---

# 51. Health Evidence Source

Prefer:

```text
independent probes
+
node self-report
```

not self-report alone.

---

# 52. Probe Network

Directory observers can run synthetic probes.

---

# 53. Probe Privacy

Probe traffic must not reveal user routes.

Use synthetic/test traffic.

---

# 54. Probe Identity

Separate from normal clients.

---

# 55. Eligibility Window

A new node may require probation before full selection weight.

---

# 56. Probation State

```rust
pub enum EligibilityTier {
    Probation,
    Normal,
    Preferred,
    Restricted,
}
```

---

# 57. Probation Purpose

Detect unstable/malicious nodes before major routing share.

---

# 58. Node Join Flow

```text
generate keys
→ register operator
→ register node
→ prove endpoint/key control
→ validate metadata
→ enter probation
→ observe health
→ become eligible
→ assign next epoch
```

---

# 59. Node Role Eligibility

Not every node can serve every role.

---

# 60. Gateway Requirements

May require:

```text
stable client connectivity
DDoS protection
high availability
public ingress
```

---

# 61. Mix Node Requirements

May require:

```text
high throughput
low packet loss
strict delay behavior
```

---

# 62. Mailbox Requirements

May require:

```text
durable storage
retention guarantees
backup
quota enforcement
```

---

# 63. Role Certification

```rust
pub struct RoleEligibility {
    pub role: MixnetNodeRole,
    pub eligible: bool,
    pub reason: Option<RoleIneligibilityReason>,
}
```

---

# 64. Node Software Version

Directory may require minimum version.

---

# 65. Version Policy

```rust
pub struct SoftwareVersionPolicy {
    pub minimum: Version,
    pub recommended: Version,
    pub blocked: Vec<VersionRange>,
}
```

---

# 66. Vulnerable Version

Can be suspended/revoked.

---

# 67. Attestation Options

Potential:

```text
signed software build
reproducible build hash
TPM attestation
remote attestation
```

---

# 68. Remote Attestation Caution

Attestation can create:

```text
hardware centralization
vendor dependence
privacy concerns
```

Therefore optional/future.

---

# 69. Preferred Initial Supply-Chain Proof

Use:

```text
signed release artifact
reproducible build hash
SBOM
```

---

# 70. Node Build Identity

```rust
pub struct NodeBuildIdentity {
    pub version: Version,
    pub build_hash: [u8; 32],
    pub sbom_hash: [u8; 32],
}
```

---

# 71. Admission Authority

```rust
pub trait NodeAdmissionAuthority {
    async fn evaluate(
        &self,
        request: NodeRegistrationRequest,
    ) -> Result<NodeAdmissionDecision, AdmissionError>;
}
```

---

# 72. Admission Decision

```rust
pub struct NodeAdmissionDecision {
    pub state: NodeAdmissionState,
    pub eligible_roles: Vec<RoleEligibility>,
    pub tier: EligibilityTier,
    pub expires_at: Timestamp,
}
```

---

# 73. Decision Expiry

Admission should be periodically revalidated.

---

# 74. Continuous Eligibility

Eligibility is not permanent.

---

# 75. Revalidation Inputs

```text
health
version
revocation
operator concentration
infrastructure metadata
policy changes
```

---

# 76. Suspension

Temporary state.

Examples:

```text
health degraded
version outdated
metadata unresolved
```

---

# 77. Revocation

Security-critical.

Examples:

```text
private key compromise
malicious behavior
false registration proof
critical software compromise
```

---

# 78. Revocation Object

```rust
pub struct NodeRevocation {
    pub node_id: MixNodeId,
    pub reason: RevocationReason,
    pub effective_at: Timestamp,
    pub signature: DirectorySignature,
}
```

---

# 79. Revocation Reason

```rust
pub enum RevocationReason {
    KeyCompromise,
    MaliciousBehavior,
    PolicyViolation,
    VulnerableSoftware,
    OperatorRequest,
    Administrative,
}
```

---

# 80. Emergency Revocation

Must propagate faster than epoch turnover.

---

# 81. Revocation Feed

Separate signed stream.

---

# 82. Client Behavior

Clients reject revoked nodes immediately.

---

# 83. Node Self-Revocation

Operator can revoke lost/compromised node.

---

# 84. Operator Revocation

If operator identity compromised:

```text
all bound nodes require suspension/revalidation
```

---

# 85. Directory Authority Revocation

More critical.

Requires trust-anchor transition.

---

# 86. Trust Root Rotation

Signed by old quorum where possible.

---

# 87. Catastrophic Authority Compromise

Need emergency recovery procedure.

---

# 88. Governance Policy Version

```rust
pub struct GovernancePolicyVersion(pub u64);
```

---

# 89. Governance Policy

Defines:

```text
admission requirements
diversity caps
role requirements
epoch rules
revocation rules
software requirements
```

---

# 90. Policy Snapshot

```rust
pub struct GovernancePolicy {
    pub version: GovernancePolicyVersion,
    pub admission: AdmissionPolicy,
    pub diversity: DiversityPolicy,
    pub topology: TopologyPolicy,
    pub software: SoftwareVersionPolicy,
}
```

---

# 91. Signed Policy

All control-plane decisions reference signed policy version.

---

# 92. Auditability

A client/operator should be able to verify:

```text
which policy version
which topology epoch
which authority signatures
```

---

# 93. Deterministic Topology Builder

Where practical, topology assignment should be deterministic from:

```text
eligible registry
epoch randomness
policy
```

---

# 94. Why Deterministic

Makes hidden bias harder.

---

# 95. Topology Random Seed

Use verifiable/random beacon if decentralized.

---

# 96. Initial Deployment Seed

Directory-generated signed randomness may be acceptable initially.

---

# 97. Future Randomness

Potential:

```text
multi-authority commit/reveal
VRF
public randomness beacon
```

---

# 98. Layer Assignment

```rust
pub trait LayerAssigner {
    fn assign(
        &self,
        eligible: &[EligibleNode],
        randomness: EpochRandomness,
        policy: &TopologyPolicy,
    ) -> Result<LayerAssignment, TopologyError>;
}
```

---

# 99. Layer Assignment Constraints

Must satisfy:

```text
role eligibility
minimum node count
operator caps
capacity balance
ASN/provider diversity
```

---

# 100. Assignment Fairness

No operator should receive structurally privileged position repeatedly.

---

# 101. Layer Rotation

Nodes reshuffled across epochs.

---

# 102. Gateway Pool

Separate from mix-layer assignment.

---

# 103. Mailbox Pool

Separate role.

---

# 104. Topology Builder

```rust
pub trait TopologyBuilder {
    fn build(
        &self,
        registry: &EligibleRegistry,
        randomness: EpochRandomness,
        policy: &GovernancePolicy,
    ) -> Result<UnsignedTopologySnapshot, TopologyError>;
}
```

---

# 105. Topology Validation

Before signing:

```text
minimum nodes
diversity
version compatibility
role eligibility
capacity
revocation exclusion
```

---

# 106. Topology Signing

```rust
pub trait TopologySigner {
    fn sign(
        &self,
        topology: &UnsignedTopologySnapshot,
    ) -> Result<TopologySignatureBundle, SigningError>;
}
```

---

# 107. Signature Bundle

```rust
pub struct TopologySignatureBundle {
    pub signatures: Vec<DirectoryAuthoritySignature>,
    pub quorum: DirectoryQuorum,
}
```

---

# 108. Client Verification

Client checks:

```text
quorum
authority trust
epoch
policy version
revocation state
```

---

# 109. Split-Brain Topology

Two valid-looking snapshots for same epoch is serious.

---

# 110. Equivocation Detection

Authorities should detect:

```text
same epoch
different topology hash
same signer
```

---

# 111. Equivocation Evidence

```rust
pub struct DirectoryEquivocationProof {
    pub epoch: TopologyEpoch,
    pub first_hash: [u8; 32],
    pub second_hash: [u8; 32],
    pub signer: DirectoryAuthorityId,
}
```

---

# 112. Equivocation Response

Suspend signer and require governance review.

---

# 113. Transparency Log

Recommended.

Append-only log of:

```text
registrations
admission decisions
revocations
policy updates
topology hashes
authority key changes
```

---

# 114. Transparency Log Purpose

Makes hidden control-plane changes auditable.

---

# 115. Privacy Constraint

No user routing data enters transparency log.

---

# 116. Transparency Entry

```rust
pub struct TransparencyEntry {
    pub sequence: u64,
    pub event: GovernanceEvent,
    pub timestamp: Timestamp,
    pub signature: DirectorySignature,
}
```

---

# 117. Merkle Log

Future/useful implementation.

Allows:

```text
inclusion proofs
consistency proofs
```

---

# 118. Governance Events

```rust
pub enum GovernanceEvent {
    NodeRegistered,
    NodeEligible,
    NodeSuspended,
    NodeRevoked,
    OperatorRegistered,
    PolicyUpdated,
    TopologyPublished,
    AuthorityRotated,
}
```

---

# 119. Operator Concentration Policy

```rust
pub struct ConcentrationPolicy {
    pub max_nodes_per_operator: u32,
    pub max_capacity_fraction_per_operator: f32,
    pub max_nodes_per_asn_fraction: f32,
    pub max_nodes_per_provider_fraction: f32,
}
```

---

# 120. Hard vs Soft Caps

Hard caps:

```text
must never violate
```

Soft caps:

```text
preferred, may degrade only under explicit policy
```

---

# 121. Strict Anonymity Mode

Uses hard caps.

---

# 122. Network Bootstrap Problem

Small network may not satisfy strong diversity.

---

# 123. Bootstrap Solution

Expose:

```text
Development
Bootstrap
Production
```

governance modes.

---

# 124. Governance Mode

```rust
pub enum GovernanceMode {
    Development,
    Bootstrap,
    Production,
}
```

---

# 125. Development

Minimal constraints, clearly non-production.

---

# 126. Bootstrap

Some relaxed caps but explicit reduced-anonymity state.

---

# 127. Production

Full constraints.

---

# 128. No Hidden Relaxation

Hard rule.

---

# 129. Anonymity Set Health

Directory can publish coarse state:

```rust
pub enum NetworkDiversityHealth {
    Healthy,
    Reduced,
    Insufficient,
}
```

---

# 130. Health Calculation

Based on:

```text
eligible nodes
operator diversity
ASN diversity
layer size
capacity concentration
```

---

# 131. Client Response

Maximum Anonymity:

```text
Insufficient
→ no new route
```

---

# 132. Standard Anonymous Mode

May allow Reduced with warning.

---

# 133. Node Admission Rate Limit

Prevent registration flood.

---

# 134. Operator Rate Limit

Bound:

```text
new nodes per epoch
```

---

# 135. Probation Capacity Cap

New nodes receive small routing weight.

---

# 136. Gradual Promotion

```text
Probation
→ Normal
→ Preferred
```

based on health history.

---

# 137. Preferred Tier

Should not become permanent privilege.

---

# 138. Tier Decay/Reevaluation

Regularly reassess.

---

# 139. Reputation

Use cautiously.

---

# 140. Reputation Inputs

Only operational:

```text
uptime
correct forwarding
availability
```

not social/user reputation.

---

# 141. Reputation Privacy

No user interaction history.

---

# 142. Reputation Manipulation

Avoid self-reported-only scoring.

---

# 143. Independent Observers

Multiple observers can contribute health evidence.

---

# 144. Observer Identity

Signed.

---

# 145. Observer Quorum

Health status may require multiple corroborating reports for severe action.

---

# 146. Immediate Local Revocation Exception

Cryptographic compromise can trigger immediate emergency action.

---

# 147. Health Evidence Model

```rust
pub struct HealthEvidence {
    pub node_id: MixNodeId,
    pub observer: DirectoryAuthorityId,
    pub interval: TimeRange,
    pub availability: f32,
    pub latency_class: HealthLatencyClass,
    pub packet_success: f32,
    pub signature: ObserverSignature,
}
```

---

# 148. Health Evidence Retention

Bounded and aggregated.

---

# 149. No Packet-Level Evidence

Hard rule.

---

# 150. Fraud Detection

Detect:

```text
fake endpoint
fake operator metadata
impossible geography/ASN claims
duplicate node fingerprints
```

---

# 151. Infrastructure Fingerprinting

Can be used carefully for Sybil detection.

---

# 152. Fingerprint Privacy

Only node infrastructure, never user clients.

---

# 153. Shared IP Prefix

Potential Sybil indicator.

Not definitive due to hosting/NAT.

---

# 154. Shared TLS/Certificate Artifacts

Potential indicator.

---

# 155. Shared Deployment Fingerprints

Can indicate common operator.

---

# 156. Manual Review

Some anomalies require manual governance review.

---

# 157. Automated Decision Boundaries

Automation may:

```text
flag
temporarily restrict
```

but high-impact revocation may require quorum.

---

# 158. Governance Roles

Potential:

```text
Directory Authority
Security Reviewer
Operator
Observer
Release Signer
```

---

# 159. Separation of Duties

One person/service should not control:

```text
admission
topology
revocation
release signing
```

alone where avoidable.

---

# 160. Policy Change Workflow

```text
proposal
→ review
→ quorum approval
→ signed policy
→ delayed activation
```

---

# 161. Emergency Policy Change

Possible for critical vulnerability.

Requires explicit audit event.

---

# 162. Delayed Activation

Helps operators/clients prepare.

---

# 163. Policy Rollback

Signed rollback to known-good version.

---

# 164. Client Policy Compatibility

Client knows minimum supported policy schema.

---

# 165. Unknown Policy Version

Fail safely.

---

# 166. Node Removal Appeal

Governance can support operator appeal.

Not required in client protocol.

---

# 167. Appeal Does Not Delay Security Revocation

Hard rule.

---

# 168. Public vs Private Operator Metadata

Public:

```text
operator pseudonymous ID
node count
capacity class
ASN/provider category
```

Private to authorities if needed:

```text
legal/contact data
```

---

# 169. Privacy-Preserving Operator Identity

Possible pseudonymous operator identity with authority verification.

---

# 170. Do Not Publish Personal Addresses

Hard rule.

---

# 171. Multi-Operator Hosting Company

One hosting company may host many independent operators.

Therefore:

```text
operator diversity
+
provider diversity
```

both matter.

---

# 172. Cloud Provider Metadata

Canonicalized.

---

# 173. ASN Metadata

Derived from endpoint IP.

---

# 174. Region Metadata

Coarse only.

---

# 175. No Exact Geolocation Requirement

Hard rule.

---

# 176. Node Admission API

```rust
#[async_trait]
pub trait NodeRegistry {
    async fn register(
        &self,
        request: NodeRegistrationRequest,
    ) -> Result<NodeRegistrationReceipt, RegistryError>;

    async fn status(
        &self,
        node: MixNodeId,
    ) -> Result<NodeAdmissionState, RegistryError>;

    async fn revoke(
        &self,
        revocation: NodeRevocationRequest,
    ) -> Result<(), RegistryError>;
}
```

---

# 177. Operator Registry API

```rust
#[async_trait]
pub trait OperatorRegistry {
    async fn register_operator(
        &self,
        request: OperatorRegistrationRequest,
    ) -> Result<OperatorId, RegistryError>;

    async fn bind_node(
        &self,
        operator: OperatorId,
        node: MixNodeId,
    ) -> Result<(), RegistryError>;
}
```

---

# 178. Directory Authority API

```rust
#[async_trait]
pub trait DirectoryAuthority {
    async fn eligible_registry(
        &self,
    ) -> Result<EligibleRegistry, DirectoryError>;

    async fn policy(
        &self,
    ) -> Result<GovernancePolicy, DirectoryError>;

    async fn revocations(
        &self,
    ) -> Result<RevocationSet, DirectoryError>;
}
```

---

# 179. Governance Store

Persistent control-plane database.

---

# 180. Authoritative Tables

Potential:

```text
operators
nodes
node_keys
admission_decisions
health_evidence
revocations
policies
topology_epochs
authority_keys
transparency_entries
```

---

# 181. Storage Backend

PostgreSQL reasonable for control plane.

---

# 182. Why Not Mixnet Node DB

Keep directory data separate from forwarding nodes.

---

# 183. Transactionality

Critical changes:

```text
revoke node
publish revocation
update registry
```

must be atomic or transactionally coordinated.

---

# 184. Topology Publication Transaction

```text
build
→ validate
→ sign
→ persist
→ publish
```

---

# 185. Publication Idempotency

Same epoch/hash can be republished safely.

---

# 186. Conflicting Publication

Rejected and audited.

---

# 187. Control-Plane HA

Directory authorities should be highly available.

---

# 188. HA Does Not Mean Shared Signing Key Everywhere

Use secure signing architecture.

---

# 189. Signing Key Storage

Prefer:

```text
HSM
hardware-backed key
offline/isolated signer
```

for high-value authority keys.

---

# 190. Online vs Offline Keys

Online keys for routine short-lived signing.

Offline root for trust-anchor recovery.

---

# 191. Root Key

Rarely used.

---

# 192. Epoch Signing Key

Rotatable.

---

# 193. Key Hierarchy

```text
Offline Root
   ↓
Directory Authority Key
   ↓
Epoch Signing Key
```

---

# 194. Key Rotation

Scheduled and auditable.

---

# 195. Key Compromise

Emergency revoke/replace.

---

# 196. Client Trust Bundle

Ships with:

```text
root trust anchors
minimum quorum
policy schema version
```

---

# 197. Trust Bundle Update

Secure software update or signed root transition.

---

# 198. Anti-Rollback

Client rejects older trust bundle after secure update.

---

# 199. Time Validation

Use bounded clock skew.

---

# 200. Offline Client

May use cached valid topology until expiry.

---

# 201. Topology Freshness Policy

```rust
pub struct TopologyFreshnessPolicy {
    pub max_age: Duration,
    pub emergency_revocation_max_age: Duration,
}
```

---

# 202. Revocation Freshness

Stricter than topology freshness.

---

# 203. Offline Maximum Anonymity

If revocation state too stale:

```text
do not create new anonymous route
```

---

# 204. Directory Censorship

If directory blocked:

```text
cached topology
multiple mirrors
signed bundles
```

can help.

Full censorship resistance later.

---

# 205. Directory Mirror

Mirror can distribute signed artifacts without authority.

---

# 206. Mirror Trust

Clients trust signatures, not mirror.

---

# 207. Governance Transparency UI

Advanced/operator UI can show:

```text
current policy version
epoch
authority quorum
network diversity health
```

---

# 208. Normal User UI

Only:

```text
Anonymous network healthy
Reduced anonymity set
Anonymous network unavailable
```

---

# 209. No Node List in Normal UI

Hard rule.

---

# 210. Operator Console

Shows:

```text
registration state
role eligibility
health
software compliance
upcoming epoch
revocation status
```

---

# 211. Admission Failure UX

Operator gets clear reason:

```text
unsupported version
endpoint unreachable
operator cap exceeded
probation incomplete
```

---

# 212. Security-Sensitive Reason

Do not expose anti-Sybil detection internals excessively.

---

# 213. Observability

Control plane can measure:

```text
node count
operator count
ASN diversity
provider diversity
eligible capacity
revocation latency
policy freshness
```

---

# 214. No User Traffic Metrics

Hard rule.

---

# 215. Governance Audit Log

All privileged actions logged.

---

# 216. Admin Action Record

```rust
pub struct GovernanceAuditRecord {
    pub actor: GovernanceActorId,
    pub action: GovernanceAction,
    pub policy_version: GovernancePolicyVersion,
    pub timestamp: Timestamp,
}
```

---

# 217. Admin Identity

Separate from directory authority signing identity.

---

# 218. Two-Person Rule

Recommended for:

```text
root rotation
authority addition/removal
emergency policy change
```

---

# 219. Emergency Actions

Need fast but auditable workflow.

---

# 220. Compromise Drill

Regularly test:

```text
node compromise
operator compromise
authority compromise
directory outage
split-brain
```

---

# 221. Simulator Integration

Part 38 simulator should consume governance/topology policy.

---

# 222. Sybil Simulation

Inject attacker controlling:

```text
10%
20%
30%
```

of nominal nodes.

Measure route compromise probability.

---

# 223. Concentration Simulation

Test:

```text
one operator
one ASN
one cloud
```

dominance scenarios.

---

# 224. Layer Assignment Simulation

Verify caps across many epochs.

---

# 225. Epoch Randomness Tests

Check no deterministic bias.

---

# 226. Equivocation Tests

Same signer signs conflicting epoch.

Must be detected.

---

# 227. Revocation Latency Tests

Measure:

```text
compromise event
→ client rejects node
```

---

# 228. Registration Flood Tests

Thousands of fake node requests.

---

# 229. Operator Flood Tests

Many pseudonymous operators from same infrastructure.

---

# 230. Metadata Fraud Tests

False ASN/provider/region claims.

---

# 231. Health Fraud Tests

Node self-reports healthy while independent probes fail.

---

# 232. Version Enforcement Tests

Blocked build cannot enter topology.

---

# 233. Key Rotation Tests

Node and authority key rotation.

---

# 234. Trust Bundle Rollback Test

Old bundle rejected.

---

# 235. Transparency Log Tests

Verify:

```text
append-only
inclusion
consistency
```

if Merkle log used.

---

# 236. Policy Upgrade Tests

Old/new client compatibility.

---

# 237. Governance Quorum Tests

Insufficient signatures rejected.

---

# 238. Byzantine Authority Tests

One malicious signer cannot publish alone under threshold model.

---

# 239. Availability Tradeoff

Higher quorum improves security but can reduce availability.

---

# 240. Quorum Policy

Choose based on deployment.

Example:

```text
3 of 5
```

future multi-authority network.

---

# 241. Initial Single Authority

Explicitly marked transitional.

---

# 242. Migration to Multi-Authority

```text
single signer
→ dual signature
→ threshold quorum
```

---

# 243. Governance Decentralization Roadmap

Phase 1:

```text
SIAR-operated directory
```

Phase 2:

```text
independent observers
```

Phase 3:

```text
multiple directory authorities
```

Phase 4:

```text
threshold topology signing
```

---

# 244. Do Not Decentralize Prematurely

Complex governance can create more security bugs than it solves.

---

# 245. Economic Sybil Defense

Optional later.

Could include:

```text
bonding
resource commitment
slashing
```

but Part 39 does not require cryptocurrency.

---

# 246. Non-Crypto Sybil Defense

Possible:

```text
verified operators
capacity audits
infrastructure diversity
admission limits
probation
manual review
```

---

# 247. Hybrid Model

Recommended likely path:

```text
operator verification
+
resource proof
+
probation
+
diversity caps
+
health evidence
```

---

# 248. Public Permissionless Goal

If SIAR later wants permissionless node joining, stronger economic/cryptographic Sybil resistance becomes mandatory.

---

# 249. Permissioned Early Network

Reasonable for early production.

---

# 250. Permissioned Does Not Mean Centralized Routing

Clients still select routes from eligible topology.

---

# 251. Governance Abuse Risk

Directory authorities could exclude honest operators.

---

# 252. Mitigation

```text
transparency log
published policy
appeal
multi-authority quorum
auditable topology
```

---

# 253. Censorship Risk

Authority could remove node/operators.

Transparency makes it visible but does not eliminate power.

---

# 254. Future Federation

Multiple governance domains may publish interoperable topologies.

Separate future architecture.

---

# 255. Security Invariants

Mandatory:

```text
1. Node identity, operator identity, and directory authority identity are separate key domains.
2. One operator must not count as multiple independent route hops in strict mode.
3. Node admission requires proof of node-key possession and operator authorization.
4. Topology eligibility is continuously revalidated.
5. Revoked nodes are excluded immediately from new routes.
6. Policy changes and topology publications are signed and auditable.
7. Directory mirrors are not trusted; signatures are.
8. Topology assignment obeys diversity constraints before performance optimization.
9. Same-epoch equivocation is detectable.
10. Governance/control-plane logs never contain user traffic or route data.
11. Production anonymity constraints are never silently relaxed during low-node conditions.
12. Single-authority operation is treated as transitional, not equivalent to threshold-governed decentralization.
```

---

# 256. Recommended Crate Layout

```text
crates/
├── siar-directory-core/
├── siar-directory-registry/
├── siar-directory-admission/
├── siar-directory-operator/
├── siar-directory-sybil/
├── siar-directory-health/
├── siar-directory-topology/
├── siar-directory-governance/
├── siar-directory-transparency/
├── siar-directory-signing/
├── siar-directory-client/
├── siar-directory-observer/
└── siar-directory-testkit/
```

---

# 257. `siar-directory-core`

Owns:

```text
IDs
policy versions
common types
errors
```

---

# 258. `siar-directory-registry`

Node registry persistence.

---

# 259. `siar-directory-admission`

Admission workflow.

---

# 260. `siar-directory-operator`

Operator registration/binding.

---

# 261. `siar-directory-sybil`

Risk signals/concentration checks.

---

# 262. `siar-directory-health`

Observer evidence and health aggregation.

---

# 263. `siar-directory-topology`

Eligibility/layer assignment/snapshot build.

---

# 264. `siar-directory-governance`

Policy lifecycle.

---

# 265. `siar-directory-transparency`

Append-only audit/transparency log.

---

# 266. `siar-directory-signing`

Authority/epoch signing.

---

# 267. `siar-directory-client`

Client snapshot/revocation validation.

---

# 268. `siar-directory-observer`

Synthetic health probes.

---

# 269. `siar-directory-testkit`

Fault/adversarial simulation.

---

# 270. Error Taxonomy

```rust
pub enum DirectoryError {
    InvalidRegistration,
    OperatorUnauthorized,
    EndpointUnverified,
    Ineligible,
    SybilRiskTooHigh,
    OperatorCapExceeded,
    UnsupportedVersion,
    InsufficientQuorum,
    InvalidSignature,
    StalePolicy,
    Revoked,
    EquivocationDetected,
    Internal,
}
```

---

# 271. Initial Production Scope

Implement first:

```text
operator identity
node identity
signed registration
endpoint verification
single SIAR directory authority
probation
health observers
operator/ASN/provider diversity caps
signed governance policy
signed topology epoch
emergency revocation feed
transparency log
client verification
simulation/testkit
```

Then add:

```text
multi-authority directory
threshold signatures
verifiable epoch randomness
Merkle transparency proofs
resource bonding
advanced Sybil heuristics
public operator onboarding
```

---

# 272. Definition of Done

Part 39 is complete when:

- node, operator, directory authority, and admin identities are separate
- node admission lifecycle and probation are explicit
- endpoint/key/operator-control proofs are defined
- layered Sybil resistance combines identity, cost, health, rate limits, and concentration caps
- operator, ASN, hosting-provider, and region diversity are accounted for
- topology eligibility is continuously revalidated
- signed governance policy drives admission and topology
- layer assignment is auditable and preferably deterministic from signed inputs
- signed topology epochs and emergency revocation feeds are defined
- authority quorum and threshold-signing migration path are specified
- equivocation detection is defined
- transparency logging excludes user traffic
- governance roles, separation of duties, and root-key recovery are defined
- low-node bootstrap mode cannot silently masquerade as production anonymity
- operator console, user-facing coarse health, and privacy-safe control-plane metrics are defined
- Sybil, concentration, equivocation, registration flood, health fraud, key rotation, and quorum tests are specified
- initial centralized deployment has a clear path toward multi-authority governance

---

# 273. Final Architecture

```text
                         NODE OPERATOR
                               │
                               ▼
                    Signed Node Registration
                               │
                               ▼
                  Admission / Proof Validation
                               │
                               ▼
                    Sybil / Diversity Checks
                               │
                               ▼
                         Probation / Health
                               │
                               ▼
                         Eligible Registry
                               │
                               ▼
                       Topology Builder
                               │
                               ▼
                    Directory Authority Quorum
                               │
                               ▼
                      Signed Epoch Snapshot
                               │
                               ▼
                         SIAR Clients
```

Governance trust path:

```text
Offline Root
    ↓
Directory Authorities
    ↓
Signed Governance Policy
    ↓
Eligible Node Registry
    ↓
Signed Topology Epoch
    ↓
Client Verification
```

---

# 274. Final Principle

A mixnet cannot treat "many node IDs" as equivalent to "many independent operators."

The correct model is:

```text
cryptographic node identity
+
operator binding
+
admission cost
+
probation
+
independent health evidence
+
operator/ASN/provider concentration limits
+
signed auditable topology
+
revocation
+
governance transparency
```

not:

```text
anyone creates unlimited nodes
→ directory lists them
→ clients randomly pick
```

This architecture gives SIAR a realistic path from a tightly controlled early network to a more decentralized, multi-authority mixnet without sacrificing the topology integrity required for meaningful anonymity.
