# Core System Architecture Part 123 — Anonymous Network Engineering Knowledge Graph, Architecture Traceability, Requirement-to-Code Mapping, Decision Provenance & Privacy-Preserving Technical Knowledge Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 123  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 65, 68, 73–75, 94–108, 120–122

**Primary purpose:** define SIAR's engineering knowledge architecture for typed technical knowledge graphs, architecture traceability, requirement-to-code mapping, decision provenance, standards/ADR linkage, evidence lineage, repository/crate/symbol indexing, test/deployment traceability, stale-link detection, impact analysis, technical knowledge access control, and privacy-preserving organizational knowledge without developer or user surveillance.

---

# 1. Purpose

A large architecture becomes difficult to trust when important relationships exist only in human memory.

A production system should be able to answer:

```text
Which requirement caused this design?
Which ADR authorized this implementation?
Which crates implement this protocol?
Which tests verify this invariant?
Which deployments run this component?
What breaks if this standard changes?
Which incident caused this guardrail?
Which decisions are obsolete because their assumptions changed?
```

The governing principle is:

> **SIAR technical knowledge should form an explicit, typed, evidence-backed graph connecting requirements, decisions, standards, code, tests, runtime assets, risks, and operational evidence—without becoming a graph of developers, users, private communications, or behavioral activity.**

---

# 2. Architectural Position

```text
                    REQUIREMENTS
                         │
                         ▼
              PRINCIPLES / STANDARDS
                         │
                         ▼
                   ADR / DESIGN
                         │
                         ▼
              CODE / CRATE / PROTOCOL
                         │
                         ▼
                  TEST / EVIDENCE
                         │
                         ▼
              RELEASE / DEPLOYMENT
                         │
                         ▼
                  RUNTIME / INCIDENT
                         │
                         └──────────────► LEARNING
```

All layers remain queryable bidirectionally.

---

# 3. Core Separation

Keep distinct:

```text
authoritative fact
derived fact
inference
human note
generated summary
runtime observation
historical decision
current policy
```

---

# 4. Non-Goals

Part 123 does not create:

```text
developer social graphs
employee contribution scoring
user-behavior graphs
private-message indexing
one giant unrestricted metadata lake
```

---

# 5. Knowledge Node Identity

```rust
pub struct KnowledgeNodeId(pub [u8; 16]);
```

Opaque, stable, never reused.

---

# 6. Knowledge Node Class

```rust
pub enum KnowledgeNodeClass {
    Requirement,
    Principle,
    TechnicalStandard,
    Adr,
    DesignReview,
    ArchitectureException,
    Risk,
    SystemicWeakness,
    Service,
    Protocol,
    Crate,
    Module,
    CodeSymbol,
    DatabaseSchema,
    ApiContract,
    Test,
    Benchmark,
    Release,
    Deployment,
    RuntimeAsset,
    Incident,
    PostIncidentReview,
    Runbook,
    Control,
    Evidence,
}
```

---

# 7. No User / Employee Node Class

Hard rule.

---

# 8. Knowledge Node

```rust
pub struct KnowledgeNode {
    pub id: KnowledgeNodeId,
    pub class: KnowledgeNodeClass,
    pub canonical_ref: CanonicalKnowledgeRef,
    pub authority: KnowledgeAuthority,
    pub lifecycle: KnowledgeLifecycleState,
}
```

---

# 9. Knowledge Authority

```rust
pub enum KnowledgeAuthority {
    Authoritative,
    Derived,
    Observed,
    Advisory,
}
```

---

# 10. Authoritative

Examples:

```text
accepted ADR
active technical standard
signed protocol schema
released artifact digest
```

---

# 11. Derived

Examples:

```text
generated dependency edge
impact-analysis projection
```

---

# 12. Observed

Examples:

```text
runtime deployment mapping
inventory snapshot
```

---

# 13. Advisory

Examples:

```text
AI-generated explanation
draft architecture note
```

---

# 14. Hard Rule

Advisory knowledge never silently becomes authoritative.

---

# 15. Knowledge Lifecycle

```rust
pub enum KnowledgeLifecycleState {
    Draft,
    Active,
    Superseded,
    Deprecated,
    Retired,
    Stale,
}
```

---

# 16. Historical Nodes Remain Addressable

Hard rule.

---

# 17. Knowledge Edge Identity

```rust
pub struct KnowledgeEdgeId(pub [u8; 16]);
```

---

# 18. Edge Classes

```rust
pub enum KnowledgeRelation {
    Requires,
    Implements,
    Constrains,
    DecidedBy,
    Supersedes,
    DependsOn,
    Calls,
    PersistsTo,
    Exposes,
    Tests,
    Verifies,
    Benchmarks,
    Deploys,
    RunsOn,
    ObservedBy,
    Violates,
    Mitigates,
    TriggeredBy,
    LearnedFrom,
    EvidencedBy,
    GovernedBy,
    ExceptionTo,
    MigratesFrom,
    MigratesTo,
}
```

---

# 19. Typed Direction

Every relation defines valid source/target classes.

---

# 20. Hard rule.

---

# 21. Example Traceability Chain

```text
Requirement
→ DecidedBy ADR
→ GovernedBy Standard
→ Implements Crate
→ Tests TestCase
→ Deploys Release
→ RunsOn RuntimeAsset
→ ObservedBy SLO
```

---

# 22. Hard rule.

---

# 23. Requirement Node

```rust
pub struct RequirementKnowledge {
    pub id: RequirementId,
    pub class: RequirementClass,
    pub statement: RequirementStatement,
    pub strength: RequirementStrength,
}
```

---

# 24. Requirement Class

```rust
pub enum RequirementClass {
    Functional,
    Security,
    Privacy,
    Reliability,
    Performance,
    Operational,
    Compatibility,
    ComplianceTechnical,
}
```

---

# 25. Requirement Strength

```rust
pub enum RequirementStrength {
    Must,
    Should,
    May,
}
```

---

# 26. Hard Invariant

Some `Must` requirements are non-waivable.

---

# 27. Hard rule.

---

# 28. Requirement Origin

```rust
pub enum RequirementOrigin {
    ProductArchitecture,
    SecurityArchitecture,
    PrivacyArchitecture,
    ProtocolRequirement,
    IncidentLearning,
    RiskRemediation,
    TechnicalComplianceControl,
}
```

---

# 29. Requirement-to-Decision Trace

Every architecture-changing requirement should trace to:

```text
ADR
standard
or explicit implementation rule
```

---

# 30. Hard rule.

---

# 31. Requirement-to-Code Mapping

```rust
pub struct RequirementImplementationLink {
    pub requirement: RequirementId,
    pub implementation: ImplementationRef,
    pub verification: Vec<TestRef>,
}
```

---

# 32. Implementation Ref

```rust
pub enum ImplementationRef {
    Crate(CrateId),
    Module(ModuleId),
    Symbol(CodeSymbolId),
    Protocol(ProtocolId),
    Schema(DatabaseSchemaId),
    DeploymentPolicy(DeploymentPolicyId),
}
```

---

# 33. No File-Path-Only Identity

Hard rule.

---

# 34. Why

Paths can change.

Use stable repository/symbol identifiers where possible.

---

# 35. Repository Identity

```rust
pub struct RepositoryId(pub [u8; 16]);
```

---

# 36. Revision Identity

```rust
pub struct RevisionId(pub Digest);
```

---

# 37. Source Location

```rust
pub struct SourceLocation {
    pub repository: RepositoryId,
    pub revision: RevisionId,
    pub path: RepoPath,
    pub symbol: Option<CodeSymbolId>,
}
```

---

# 38. Hard Rule

Traceability always records revision context.

---

# 39. Crate Identity

```rust
pub struct CrateId {
    pub workspace: WorkspaceId,
    pub name: String,
}
```

---

# 40. Crate Layer

```rust
pub enum CrateArchitectureLayer {
    Domain,
    Protocol,
    Application,
    Infrastructure,
    Adapter,
    Ui,
    Testkit,
}
```

---

# 41. Part 122 Layering Integration

Hard rule.

---

# 42. Code Symbol

```rust
pub struct CodeSymbol {
    pub id: CodeSymbolId,
    pub crate_id: CrateId,
    pub kind: CodeSymbolKind,
    pub qualified_name: String,
}
```

---

# 43. Code Symbol Kind

```rust
pub enum CodeSymbolKind {
    Trait,
    Struct,
    Enum,
    Function,
    Impl,
    Module,
    Constant,
    Macro,
}
```

---

# 44. Symbol Hash

Optional structural fingerprint.

---

# 45. Hard rule.

---

# 46. Symbol Rename

Preserve lineage if confidently detected.

---

# 47. Otherwise create new node + `MigratesFrom`.

---

# 48. Hard rule.

---

# 49. Protocol Traceability

Protocol node links:

```text
wire schema
version
negotiation rules
implementation crate
fuzz harness
compatibility tests
release support
```

---

# 50. Hard rule.

---

# 51. Database Schema Traceability

Schema node links:

```text
migration
repository model
service
retention rule
backup policy
```

---

# 52. Hard rule.

---

# 53. API Contract Traceability

API node links:

```text
caller
service
authorization policy
schema/version
contract tests
```

---

# 54. Hard rule.

---

# 55. Standard Traceability

Part 122.

```text
TechnicalStandard
→ Constrains Crate
→ Constrains Protocol
→ VerifiedBy ArchitectureLint/Test
```

---

# 56. Hard rule.

---

# 57. ADR Traceability

```text
ADR
→ Decides Service/Protocol/Crate
→ Supersedes ADR
→ EvidencedBy DesignReview
→ ConstrainedBy Standard
```

---

# 58. Hard rule.

---

# 59. Architecture Exception Traceability

```text
Exception
→ ExceptionTo Standard
→ AppliesTo Scope
→ LinkedTo Risk
→ ExpiresAt
```

---

# 60. Hard rule.

---

# 61. Risk Traceability

Part 121.

```text
Risk
→ Affects Service
→ LinkedTo SystemicWeakness
→ MitigatedBy Change
→ VerifiedBy Test/Simulation
```

---

# 62. Hard rule.

---

# 63. Incident Learning Traceability

Part 120.

```text
Incident
→ LearnedFrom PIR
→ Creates Risk
→ Creates Requirement
→ Creates Test
→ Changes Standard/ADR
```

---

# 64. Good.

---

# 65. Hard rule.

---

# 66. Test Traceability

Every high-value invariant should link to a verification artifact.

---

# 67. Test Node

```rust
pub struct TestKnowledge {
    pub test: TestId,
    pub class: TestClass,
    pub implementation: SourceLocation,
}
```

---

# 68. Test Class

```rust
pub enum TestClass {
    Unit,
    Property,
    Fuzz,
    Integration,
    EndToEnd,
    Benchmark,
    Simulation,
    FormalModel,
}
```

---

# 69. Verification Edge

```text
Test → Verifies Requirement
Test → Verifies Standard
Test → Verifies RiskRemediation
```

---

# 70. Hard rule.

---

# 71. Formal Model Traceability

Link:

```text
TLA+ spec
Kani proof harness
Loom concurrency model
```

to relevant invariant/state machine.

---

# 72. Hard rule.

---

# 73. Benchmark Traceability

Performance/efficiency requirement:

```text
Requirement
→ Benchmark
→ ReleaseGate
```

---

# 74. Hard rule.

---

# 75. Release Traceability

```rust
pub struct ReleaseKnowledge {
    pub release: ReleaseId,
    pub artifact_manifest: Digest,
    pub architecture_baseline: ArchitectureBaselineVersion,
    pub source_revision: RevisionId,
}
```

---

# 76. Every Release Pins

```text
source revision
architecture baseline
standards version
SBOM/provenance
test evidence
```

---

# 77. Hard rule.

---

# 78. Deployment Traceability

```rust
pub struct DeploymentKnowledge {
    pub deployment: DeploymentId,
    pub release: ReleaseId,
    pub environment: EnvironmentClass,
    pub regions: BTreeSet<RegionId>,
}
```

---

# 79. Runtime Mapping

```text
Release
→ Deploys Deployment
→ RunsOn RuntimeAsset
```

---

# 80. Hard rule.

---

# 81. Runtime Asset Links

Part 106.

Connect:

```text
service
deployment
host
region
site
fault domain
```

---

# 82. No user flow graph.

---

# 83. Hard rule.

---

# 84. Evidence Node

```rust
pub struct EvidenceKnowledge {
    pub id: EvidenceId,
    pub kind: EvidenceKind,
    pub authority: KnowledgeAuthority,
    pub digest: Digest,
}
```

---

# 85. Evidence Kinds

```rust
pub enum EvidenceKind {
    TestResult,
    AuditRecord,
    SimulationResult,
    BenchmarkResult,
    Attestation,
    ChangeReceipt,
    ReleaseReceipt,
    IncidentEvidence,
}
```

---

# 86. Evidence Immutable

Hard rule.

---

# 87. Evidence Lineage

```text
source observation
→ normalized evidence
→ decision
```

---

# 88. Hard rule.

---

# 89. Decision Provenance

Every decision should answer:

```text
who/what authority
what inputs
which standard
which risks
which alternatives
which evidence
which version
```

---

# 90. Decision Provenance Record

```rust
pub struct DecisionProvenance {
    pub decision: AdrId,
    pub baseline: ArchitectureBaselineVersion,
    pub standards: BTreeSet<TechnicalStandardId>,
    pub evidence: BTreeSet<EvidenceId>,
    pub risks: BTreeSet<EngineeringRiskId>,
}
```

---

# 91. Person Identity

Approver identity may be recorded for accountability.

---

# 92. Not usable as performance dimension.

---

# 93. Hard rule.

---

# 94. Knowledge Provenance

Every derived node/edge stores derivation source.

---

# 95. Provenance Type

```rust
pub enum KnowledgeProvenance {
    HumanAuthored,
    RepositoryDerived,
    RuntimeObserved,
    PolicyCompiled,
    IncidentDerived,
    AiGeneratedAdvisory,
}
```

---

# 96. AI-Generated Advisory

Must remain labeled.

---

# 97. Hard rule.

---

# 98. AI Cannot Auto-Approve Architecture

Hard rule.

---

# 99. Generated Knowledge

Useful for:

```text
summaries
candidate links
missing-trace suggestions
impact hypotheses
```

---

# 100. Hard rule.

---

# 101. Generated Edge State

```rust
pub enum DerivedEdgeState {
    Candidate,
    Confirmed,
    Rejected,
    Stale,
}
```

---

# 102. Candidate ≠ Confirmed

Hard rule.

---

# 103. Knowledge Graph Storage

Graph is logical model.

---

# 104. Physical storage may be:

```text
Postgres relational tables
graph projection
search index
```

---

# 105. Do Not Require Graph DB

Hard rule.

---

# 106. Canonical Authority

Postgres/typed records recommended.

---

# 107. Graph projection can be derived.

---

# 108. Hard rule.

---

# 109. Graph Edge Record

```rust
pub struct KnowledgeEdge {
    pub id: KnowledgeEdgeId,
    pub from: KnowledgeNodeId,
    pub relation: KnowledgeRelation,
    pub to: KnowledgeNodeId,
    pub provenance: KnowledgeProvenance,
    pub state: DerivedEdgeState,
}
```

---

# 110. For Human-Authoritative Edge

`state = Confirmed`.

---

# 111. Hard rule.

---

# 112. Bidirectional Query

Core requirement.

Examples:

```text
Which code implements requirement X?
Which requirements depend on crate Y?
Which tests verify ADR Z?
Which incidents created this standard?
```

---

# 113. Hard rule.

---

# 114. Traceability Query API

```rust
pub trait TraceabilityQueryService {
    fn upstream(
        &self,
        node: KnowledgeNodeId,
        depth: QueryDepth,
    ) -> Result<TraceabilitySubgraph, KnowledgeGraphError>;

    fn downstream(
        &self,
        node: KnowledgeNodeId,
        depth: QueryDepth,
    ) -> Result<TraceabilitySubgraph, KnowledgeGraphError>;
}
```

---

# 115. Query Depth Bounded

Hard rule.

---

# 116. No Unbounded Graph Traversal

Hard rule.

---

# 117. Path Query

```rust
pub trait TracePathService {
    fn paths(
        &self,
        from: KnowledgeNodeId,
        to: KnowledgeNodeId,
        limits: TraceQueryLimits,
    ) -> Result<Vec<TracePath>, KnowledgeGraphError>;
}
```

---

# 118. Limits

```rust
pub struct TraceQueryLimits {
    pub max_depth: u8,
    pub max_nodes: usize,
    pub max_paths: usize,
}
```

---

# 119. Hard rule.

---

# 120. Impact Analysis

"What if X changes?"

---

# 121. Impact Query

```rust
pub trait ArchitectureImpactService {
    fn impact(
        &self,
        changed: KnowledgeNodeId,
    ) -> Result<ImpactReport, KnowledgeGraphError>;
}
```

---

# 122. Impact Categories

```rust
pub enum ImpactCategory {
    Code,
    Protocol,
    Data,
    Test,
    Deployment,
    Risk,
    Standard,
    Documentation,
}
```

---

# 123. Impact Is Candidate Until Validated

Hard rule.

---

# 124. Change Integration

Part 107.

Change proposal can request impacted nodes.

---

# 125. Hard rule.

---

# 126. Design Review Integration

Part 122.

Design review automatically receives:

```text
upstream requirements
active standards
known risks
downstream implementation
```

---

# 127. Hard rule.

---

# 128. Stale Trace Detection

Links can become outdated.

---

# 129. Staleness Reasons

```rust
pub enum TraceStalenessReason {
    SourceDeleted,
    SymbolMoved,
    AdrSuperseded,
    StandardDeprecated,
    TestRemoved,
    ReleaseRetired,
    RuntimeAssetGone,
}
```

---

# 130. Hard rule.

---

# 131. Trace Health

```rust
pub enum TraceHealth {
    Healthy,
    Partial,
    Broken,
    Unknown,
}
```

---

# 132. Unknown ≠ Healthy

Hard rule.

---

# 133. Requirement Coverage

```rust
pub struct RequirementCoverage {
    pub requirement: RequirementId,
    pub implementation_links: usize,
    pub verification_links: usize,
    pub health: TraceHealth,
}
```

---

# 134. No "100% coverage" claim without defined scope.

---

# 135. Hard rule.

---

# 136. Orphan Detection

Find:

```text
requirement without implementation
implementation without governing requirement/ADR
standard without enforcement
test without linked invariant
```

---

# 137. Hard rule.

---

# 138. Orphan Class

```rust
pub enum KnowledgeOrphanClass {
    UnimplementedRequirement,
    UngovernedImplementation,
    UnverifiedInvariant,
    UnenforcedStandard,
    UnownedRisk,
}
```

---

# 139. Good.

---

# 140. Hard rule.

---

# 141. Trace Completeness Policy

Different classes require different minimum links.

---

# 142. Example

Security requirement:

```text
Requirement
→ ADR/Standard
→ Implementation
→ Test
```

---

# 143. Hard rule.

---

# 144. Trace Policy

```rust
pub struct TraceabilityPolicy {
    pub requirement_class: RequirementClass,
    pub required_relations: BTreeSet<KnowledgeRelation>,
}
```

---

# 145. Hard rule.

---

# 146. Code Indexing

Repository parser extracts:

```text
crates
modules
traits
structs
enums
functions
dependency edges
```

---

# 147. Rust Analyzer Integration

Preferred for Rust semantic symbols.

---

# 148. Hard rule.

---

# 149. Build Metadata Integration

Cargo metadata supplies:

```text
workspace
crate dependency graph
features
targets
```

---

# 150. Hard rule.

---

# 151. No Source Code Upload To External AI By Default

Hard rule.

---

# 152. Local/approved model only if code analysis requires AI.

---

# 153. Hard rule.

---

# 154. Repository Scanner

```rust
pub trait RepositoryKnowledgeIndexer {
    fn index_revision(
        &self,
        repository: RepositoryId,
        revision: RevisionId,
    ) -> Result<RepositoryKnowledgeSnapshot, KnowledgeGraphError>;
}
```

---

# 155. Deterministic Extraction Preferred

Hard rule.

---

# 156. Symbol Link Hints

Code annotations can declare links.

Example:

```rust
#[siar_requirement("REQ-SEC-042")]
#[siar_adr("ADR-017")]
pub struct VerifiedEnvelope { ... }
```

---

# 157. Macro/Attribute Approach Optional

Hard rule.

---

# 158. Alternative

Sidecar RON mapping.

---

# 159. Hard rule.

---

# 160. Avoid Polluting Domain Logic Excessively

Hard rule.

---

# 161. Link Manifest

```ron
(
    requirement: "REQ-SEC-042",
    implementation: "siar-protocol::VerifiedEnvelope",
    tests: ["verified_envelope_rejects_invalid_signature"],
)
```

---

# 162. Human Reviewable.

---

# 163. Hard rule.

---

# 164. Test Indexing

Use:

```text
cargo test metadata
test naming convention
link attributes
sidecar manifest
```

---

# 165. Hard rule.

---

# 166. CI Traceability Gate

CI can reject:

```text
deleted implementation for mandatory requirement
removed test for hard invariant
expired exception
unresolved broken trace
```

---

# 167. Hard rule.

---

# 168. Release Traceability Gate

Part 108.

Release requires minimum trace health for critical requirements.

---

# 169. Hard rule.

---

# 170. Architecture Baseline Integration

Part 122.

Knowledge snapshot pins:

```text
baseline
revision
release
```

---

# 171. Hard rule.

---

# 172. Knowledge Snapshot

```rust
pub struct EngineeringKnowledgeSnapshot {
    pub snapshot: KnowledgeSnapshotId,
    pub architecture_baseline: ArchitectureBaselineVersion,
    pub repository_revisions: BTreeMap<RepositoryId, RevisionId>,
    pub generated_at: Timestamp,
}
```

---

# 173. Immutable

Hard rule.

---

# 174. Current Graph

Mutable projection.

Historical snapshots immutable.

---

# 175. Hard rule.

---

# 176. Temporal Query

"What was true at release X?"

---

# 177. Core requirement.

---

# 178. Temporal Trace Service

```rust
pub trait TemporalTraceService {
    fn snapshot_for_release(
        &self,
        release: ReleaseId,
    ) -> Result<EngineeringKnowledgeSnapshot, KnowledgeGraphError>;
}
```

---

# 179. Hard rule.

---

# 180. Decision Provenance Query

"Why is this architecture this way?"

---

# 181. Query returns:

```text
requirement
ADR
standard
risks
alternatives
incident learning
```

---

# 182. Hard rule.

---

# 183. Explainability View

Human-facing narrative derived from graph.

---

# 184. Derived/advisory.

---

# 185. Hard rule.

---

# 186. No Invented Rationale

Generated explanation cites authoritative nodes.

---

# 187. Hard rule.

---

# 188. Knowledge Search

Full-text search over allowed technical metadata/documents.

---

# 189. Search Scope

```rust
pub enum KnowledgeSearchScope {
    Architecture,
    Code,
    Tests,
    Operations,
    Risk,
    IncidentLearning,
}
```

---

# 190. Hard rule.

---

# 191. Search Result

```rust
pub struct KnowledgeSearchResult {
    pub node: KnowledgeNodeId,
    pub authority: KnowledgeAuthority,
    pub snippet: String,
}
```

---

# 192. Authority Visible

Hard rule.

---

# 193. No Private User Content Index

Hard rule.

---

# 194. Access Control

Technical knowledge may contain sensitive details.

---

# 195. Knowledge Sensitivity

```rust
pub enum KnowledgeSensitivity {
    Public,
    Internal,
    Restricted,
    SecuritySensitive,
}
```

---

# 196. Hard rule.

---

# 197. Restricted Examples

```text
internal topology
vulnerability detail
facility location
key-management design
```

---

# 198. Hard rule.

---

# 199. Authorization

Use Part 81 capability/ABAC model.

---

# 200. Knowledge Access Scope

```rust
pub struct KnowledgeAccessContext {
    pub subject: AuthorizationSubject,
    pub sensitivity: KnowledgeSensitivity,
    pub purpose: KnowledgeAccessPurpose,
}
```

---

# 201. Purpose

```rust
pub enum KnowledgeAccessPurpose {
    Development,
    DesignReview,
    IncidentResponse,
    Audit,
    Operations,
}
```

---

# 202. Hard rule.

---

# 203. No Broad "Engineering Can See Everything"

Hard rule.

---

# 204. Metadata Leakage

Even graph structure can reveal architecture.

---

# 205. Restricted nodes hide adjacency where needed.

---

# 206. Hard rule.

---

# 207. Redacted Graph Views

```rust
pub struct KnowledgeGraphView {
    pub nodes: Vec<KnowledgeNodeView>,
    pub edges: Vec<KnowledgeEdgeView>,
    pub redactions: usize,
}
```

---

# 208. Hard rule.

---

# 209. Federation Knowledge

Share only agreed public/interoperability nodes.

---

# 210. No internal architecture graph federation by default.

---

# 211. Hard rule.

---

# 212. Tenant Knowledge

Managed tenant may see:

```text
their integration contracts
their deployment state
their scoped controls
```

---

# 213. Not platform secrets.

---

# 214. Hard rule.

---

# 215. Privacy-Preserving Knowledge Model

Graph dimensions never include:

```text
user identity
user contacts
message behavior
employee productivity
developer contribution count
```

---

# 216. Hard rule.

---

# 217. Developer Identity

Commit authorship may exist in source-control systems.

---

# 218. Engineering graph does not need it for architecture traceability.

---

# 219. Hard rule.

---

# 220. Decision Accountability

Approver identities allowed where required.

---

# 221. Never aggregate into performance metrics.

---

# 222. Hard rule.

---

# 223. AI Integration

AI can assist with:

```text
candidate edge discovery
document summarization
stale-link suggestions
impact hypotheses
```

---

# 224. AI Cannot:

```text
invent authoritative relationship
approve ADR
close risk
declare requirement verified
```

---

# 225. Hard rule.

---

# 226. AI Provenance

```rust
pub struct AiKnowledgeSuggestion {
    pub suggestion_id: KnowledgeSuggestionId,
    pub proposed_edges: Vec<KnowledgeEdgeCandidate>,
    pub model_context_digest: Digest,
}
```

---

# 227. Candidate Only

Hard rule.

---

# 228. Human / Deterministic Confirmation

Required before authoritative use.

---

# 229. Hard rule.

---

# 230. Knowledge Drift

Three major types:

```text
documentation drift
code drift
runtime drift
```

---

# 231. Documentation Drift

ADR/standard says X, docs say Y.

---

# 232. Code Drift

Implementation no longer conforms.

---

# 233. Runtime Drift

Deployment differs from expected release/config.

---

# 234. Hard rule.

---

# 235. Drift Finding

```rust
pub struct KnowledgeDriftFinding {
    pub kind: KnowledgeDriftKind,
    pub expected: KnowledgeNodeId,
    pub observed: KnowledgeNodeId,
    pub severity: FindingSeverity,
}
```

---

# 236. Hard rule.

---

# 237. Drift Resolution

```text
fix implementation
update design through governance
supersede ADR
retire stale link
```

---

# 238. No Silent Documentation Edit To Match Bug

Hard rule.

---

# 239. Knowledge Freshness

```rust
pub enum KnowledgeFreshness {
    Fresh,
    DueForReview,
    Stale,
    Broken,
}
```

---

# 240. Hard rule.

---

# 241. Freshness Signals

```text
source revision changed
linked ADR superseded
linked standard deprecated
runtime asset disappeared
test removed
```

---

# 242. Hard rule.

---

# 243. Review Cadence

High-risk nodes reviewed more frequently.

---

# 244. Hard rule.

---

# 245. Orphan / Broken-Link Queue

Dedicated remediation queue.

---

# 246. Hard rule.

---

# 247. Knowledge Repair Action

```rust
pub enum KnowledgeRepairAction {
    RebindSymbol,
    AddVerification,
    SupersedeDecision,
    UpdateStandardLink,
    RetireNode,
    ConfirmDerivedEdge,
}
```

---

# 248. Hard rule.

---

# 249. Traceability SLOs

Examples:

```text
100% hard security requirements have implementation + verification links
100% releases pin architecture baseline and source revision
critical broken traces repaired within target
```

---

# 250. Hard rule.

---

# 251. Knowledge Quality Metrics

Safe:

```text
broken critical traces
orphaned requirements
stale ADR links
unverified hard invariants
```

---

# 252. Forbidden:

```text
traceability errors by developer
knowledge contributions per employee
```

---

# 253. Hard rule.

---

# 254. Knowledge Graph Observability

Aggregate only.

---

# 255. Hard rule.

---

# 256. Query Audit

Security-sensitive graph access can be audited.

---

# 257. Do not audit every benign lookup if unnecessary.

---

# 258. Hard rule.

---

# 259. Knowledge Export

Export may include:

```text
public architecture map
SBOM-linked component map
compliance trace package
```

---

# 260. Apply redaction policy.

---

# 261. Hard rule.

---

# 262. Compliance Trace Package

Can show:

```text
control
→ requirement
→ standard
→ implementation
→ test evidence
```

---

# 263. Without user data.

---

# 264. Hard rule.

---

# 265. Security Assurance Trace

```text
security requirement
→ threat model
→ control
→ implementation
→ verification
```

---

# 266. Hard rule.

---

# 267. Privacy Assurance Trace

```text
privacy requirement
→ data-flow constraint
→ implementation
→ deletion/retention test
```

---

# 268. Hard rule.

---

# 269. Reliability Assurance Trace

```text
SLO
→ architecture
→ redundancy model
→ test/simulation
→ deployment
```

---

# 270. Hard rule.

---

# 271. Incident Trace

```text
incident
→ root cause
→ risk
→ remediation
→ test
→ release
```

---

# 272. Hard rule.

---

# 273. Requirement Change

When requirement changes:

```text
identify dependent ADRs
standards
code
tests
deployments
```

---

# 274. Hard rule.

---

# 275. Standard Change

Impact graph identifies affected implementations/releases.

---

# 276. Hard rule.

---

# 277. ADR Supersession

Flags downstream links for review.

---

# 278. Hard rule.

---

# 279. Crate Deletion

Cannot silently remove linked hard requirement implementation.

---

# 280. CI gate.

---

# 281. Hard rule.

---

# 282. Test Deletion

Critical invariant loses verification link.

---

# 283. CI gate.

---

# 284. Hard rule.

---

# 285. Deployment Retirement

Historical trace remains.

---

# 286. Current projection updates.

---

# 287. Hard rule.

---

# 288. Data Retention

Durable architecture history retained long-term.

---

# 289. High-volume derived runtime observations rolled up.

---

# 290. Hard rule.

---

# 291. No Raw Telemetry Replication Into Knowledge Graph

Hard rule.

---

# 292. Use evidence references.

---

# 293. Hard rule.

---

# 294. Knowledge Graph API

```rust
pub trait EngineeringKnowledgeGraph {
    fn node(
        &self,
        id: KnowledgeNodeId,
    ) -> Result<KnowledgeNode, KnowledgeGraphError>;

    fn edges(
        &self,
        id: KnowledgeNodeId,
        relation: Option<KnowledgeRelation>,
    ) -> Result<Vec<KnowledgeEdge>, KnowledgeGraphError>;
}
```

---

# 295. Requirement Trace Service

```rust
pub trait RequirementTraceService {
    fn coverage(
        &self,
        requirement: RequirementId,
    ) -> Result<RequirementCoverage, KnowledgeGraphError>;
}
```

---

# 296. Provenance Service

```rust
pub trait DecisionProvenanceService {
    fn provenance(
        &self,
        adr: AdrId,
    ) -> Result<DecisionProvenance, KnowledgeGraphError>;
}
```

---

# 297. Drift Service

```rust
pub trait KnowledgeDriftService {
    fn evaluate(
        &self,
        snapshot: KnowledgeSnapshotId,
    ) -> Result<Vec<KnowledgeDriftFinding>, KnowledgeGraphError>;
}
```

---

# 298. No People-Analytics API

Hard rule.

---

# 299. Error Taxonomy

```rust
pub enum KnowledgeGraphError {
    NodeUnknown,
    EdgeInvalid,
    TraceBroken,
    TraceIncomplete,
    SnapshotUnknown,
    ProvenanceMissing,
    CandidateNotConfirmed,
    AccessDenied,
    QueryLimitExceeded,
    ScopeViolation,
    Internal,
}
```

---

# 300. Performance

Knowledge queries must be bounded.

---

# 301. Use indexed adjacency.

---

# 302. Avoid recursive unbounded SQL.

---

# 303. Hard rule.

---

# 304. Caching

Cache immutable snapshot queries.

---

# 305. Access-control-aware cache keys.

---

# 306. Hard rule.

---

# 307. Incremental Indexing

Only changed revision portions re-indexed.

---

# 308. Hard rule.

---

# 309. Event Pipeline

Repository/release/governance events update derived graph asynchronously.

---

# 310. Authoritative writes remain transactional.

---

# 311. Hard rule.

---

# 312. Eventual Derived Consistency

Allowed.

---

# 313. Authoritative decision state not eventual.

---

# 314. Hard rule.

---

# 315. Storage Model

Recommended canonical tables:

```text
knowledge_nodes
knowledge_edges
knowledge_snapshots
knowledge_provenance
requirement_links
symbol_index
trace_health
```

---

# 316. Derived search/graph indexes can be rebuilt.

---

# 317. Hard rule.

---

# 318. PostgreSQL

Recommended authoritative store.

---

# 319. Local developer projection may use SQLite/Stoolap-compatible adapters where appropriate.

---

# 320. Hard rule.

---

# 321. Content-Addressed Documents

Architecture docs/evidence can use digest-addressed references.

---

# 322. Hard rule.

---

# 323. Crate Layout

Recommended:

```text
crates/
├── siar-knowledge-core/
├── siar-knowledge-registry/
├── siar-requirement-trace/
├── siar-code-index/
├── siar-decision-provenance/
├── siar-impact-analysis/
├── siar-knowledge-drift/
├── siar-knowledge-search/
├── siar-knowledge-access/
├── siar-knowledge-observability/
└── siar-knowledge-testkit/
```

---

# 324. `siar-knowledge-core`

Owns:

```text
KnowledgeNodeId
KnowledgeNodeClass
KnowledgeRelation
KnowledgeAuthority
KnowledgeGraphError
```

---

# 325. `siar-knowledge-registry`

Authoritative node/edge persistence.

---

# 326. `siar-requirement-trace`

Requirement→decision→implementation→verification.

---

# 327. `siar-code-index`

Cargo/rust-analyzer/repository semantic indexing.

---

# 328. `siar-decision-provenance`

ADR/standard/risk/evidence lineage.

---

# 329. `siar-impact-analysis`

Bounded upstream/downstream impact queries.

---

# 330. `siar-knowledge-drift`

Staleness/orphans/broken links.

---

# 331. `siar-knowledge-search`

Authorized technical full-text/search projection.

---

# 332. `siar-knowledge-access`

Sensitivity/redaction/capability policy.

---

# 333. `siar-knowledge-observability`

Aggregate trace health only.

---

# 334. `siar-knowledge-testkit`

graph/trace/privacy/invariant tests.

---

# 335. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. The engineering knowledge graph contains technical entities—requirements, standards, ADRs, services, protocols, crates, symbols, tests, releases, deployments, risks, incidents, controls, and evidence—not user social graphs or employee-performance graphs.
2. Authoritative, derived, observed, advisory, and AI-generated knowledge remain explicitly distinguished; generated or inferred links can never silently become authoritative.
3. Every critical traceability relation records canonical identity, provenance, and source revision/version so code movement, ADR supersession, standard deprecation, and release history remain explainable over time.
4. Hard security/privacy/reliability requirements require traceability to governing decision/standard, implementation, and verification evidence; missing links become explicit trace-health defects.
5. Historical architecture decisions, releases, evidence, and snapshots are immutable; current projections may evolve but cannot rewrite what was true for an older release.
6. Impact analysis and graph traversal are bounded, access-controlled, and treated as candidate analysis until validated; they cannot become an unbounded metadata-discovery surface.
7. Sensitive architecture nodes, adjacency, facility/topology information, security details, and evidence obey authorization/redaction policy; graph metadata itself is treated as potentially sensitive.
8. Source-code indexing uses deterministic local/approved tooling by default; source is not uploaded to external AI services merely to build traceability.
9. AI may propose summaries, links, stale-edge candidates, and impact hypotheses but cannot approve ADRs, verify requirements, close risks, or authoritatively assert provenance without confirmation.
10. Traceability quality, orphan detection, knowledge freshness, and graph observability are measured by technical node/edge state and cannot be aggregated into developer productivity, contribution, rejection, or blame scores.
11. Release/change/design-review gates can require traceability health for critical requirements, but historical knowledge remains available even after components, tests, standards, or deployments are retired.
12. The engineering knowledge architecture integrates with standards/ADR governance, risk/PIR, inventory, change/release, SLOs, resilience, supply chain, compliance, audit, and incident response without becoming an alternate authority or surveillance system.
```

---

# 336. Initial Production Scope

Implement first:

```text
typed KnowledgeNodeId/KnowledgeNodeClass
typed KnowledgeRelation validation
authoritative/derived/advisory distinction
requirement registry
requirement→ADR/standard links
requirement→crate/module/symbol links
requirement→test links
Cargo metadata crate graph
rust-analyzer symbol index
ADR/standard/risk/PIR provenance links
release/source/baseline snapshots
deployment/runtime trace links
bounded upstream/downstream queries
impact analysis
trace health/orphan detection
exception/risk linkage
CI traceability gates for hard requirements
knowledge sensitivity/access control
redacted graph views
privacy-safe graph metrics
knowledge testkit
```

Then add:

```text
semantic symbol rename tracking
multi-repository traceability
graph visualizations
AI-assisted candidate-link discovery
automatic ADR impact prompts
compliance/assurance trace-package generation
temporal graph diff
formal edge-type validation
```

---

# 337. Definition of Done

Part 123 is complete when:

- technical knowledge nodes/edges are typed;
- no user/employee graph classes exist;
- authoritative and generated knowledge are distinguishable;
- requirements trace to decisions, code, and tests;
- code links are revision-aware;
- releases pin source revision + architecture baseline;
- deployment/runtime links are queryable;
- historical snapshots remain immutable;
- ADR/standard changes propagate trace review;
- orphan/broken traces are detectable;
- impact traversal is bounded;
- restricted graph metadata is access-controlled/redacted;
- AI suggestions remain advisory;
- CI/release can enforce critical trace health;
- no developer/user scoring is created;
- graph/trace/privacy/fuzz/formal tests are specified.

---

# 338. Testing

Required scenarios:

```text
requirement mapped to crate + property test
ADR superseded after release
crate symbol renamed
critical test deleted
standard deprecated
runtime deployment retired
incident creates new requirement
```

---

# 339. Edge-Type Validation Test

Invalid relation:

```text
User → Implements → Crate
```

is impossible because User is not a node type.

---

# 340. Requirement Coverage Test

Hard security requirement without verification link becomes `Partial/Broken`.

---

# 341. Revision Test

Trace points to exact source revision.

---

# 342. Supersession Test

Old ADR remains historical; downstream current links are flagged for review.

---

# 343. Test Deletion Test

Removing the only verifier for a hard invariant fails CI.

---

# 344. Release Snapshot Test

Old release query reconstructs old source/baseline relationships.

---

# 345. Access Test

Restricted topology adjacency is not revealed to unauthorized caller.

---

# 346. AI Suggestion Test

AI candidate edge cannot satisfy required trace until confirmed.

---

# 347. Privacy Test

No query or aggregation by employee/user identity exists.

---

# 348. Fuzzing

Fuzz:

```text
node records
edge relations
trace paths
snapshot manifests
redaction policies
```

---

# 349. Property Tests

Properties:

```text
invalid node-class/relation combinations can never be persisted
advisory/candidate edge can never satisfy mandatory verification coverage
historical snapshot can never mutate after publication
unauthorized restricted node can never leak adjacency through a graph query
```

---

# 350. Formal Verification Targets

Strong candidates:

```text
typed edge relation matrix
snapshot immutability
trace-health state machine
authorization-aware graph traversal
```

---

# 351. Kani Candidate

Edge type, bounded traversal, authority-state invariants.

---

# 352. TLA+ Candidate

requirement → decision → implementation → verification → release → supersession.

---

# 353. Loom Candidate

concurrent repository re-index + ADR supersession + release snapshot creation.

---

# 354. Final Architecture

```text
                     REQUIREMENTS
                          │
                          ▼
                 PRINCIPLES / STANDARDS
                          │
                          ▼
                    ADR / REVIEW
                          │
                          ▼
                PROTOCOL / CRATE / CODE
                          │
                          ▼
                  TEST / FORMAL EVIDENCE
                          │
                          ▼
                   RELEASE / DEPLOYMENT
                          │
                          ▼
                   RUNTIME / INCIDENT
                          │
                          ▼
                    PIR / RISK / LEARNING
                          │
                          └──────────────► REQUIREMENTS
```

Knowledge safety model:

```text
typed technical graph
+
immutable provenance
+
revision-aware code links
+
bidirectional traceability
+
bounded impact analysis
+
historical snapshots
+
access-controlled metadata
+
AI advisory-only derivation
```

not:

```text
scrape every repository and chat into one graph, infer authority from AI output, track which developer touched each problem, and expose the entire architecture graph to everyone
```

---

# 355. Final Principle

Engineering knowledge becomes durable when every important technical statement can answer both:

```text
Why does this exist?
```

and:

```text
What depends on it?
```

The correct model is:

```text
capture requirements
+
link decisions
+
trace implementation
+
verify with tests/evidence
+
pin releases and deployments
+
preserve historical provenance
+
detect stale or missing links
+
bound and authorize graph access
+
never turn technical traceability into people or user surveillance
```

This architecture gives SIAR a privacy-preserving technical knowledge foundation for requirement-to-code mapping, architecture traceability, decision provenance, impact analysis, historical reconstruction, assurance evidence, and organizational learning while preserving the anonymity, local-first, least-authority, architecture-governance, engineering-risk, and anti-surveillance guarantees established across Parts 34–122.
