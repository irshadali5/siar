# Core System Architecture Part 106 — Anonymous Network Infrastructure Inventory, Asset Graph, Service Dependency Mapping, Configuration Item Lifecycle & Privacy-Preserving Operational Topology Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 106  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 61–65, 69, 71–80, 94–105

**Primary purpose:** define SIAR's infrastructure-inventory and operational-topology architecture for canonical asset identity, configuration items, service dependencies, topology graphs, lifecycle state, ownership, provenance, discovery, reconciliation, stale/orphan detection, blast-radius analysis, regional/tenant/federation boundaries, graph-query safety, recovery integration, and privacy-preserving infrastructure visibility.

---

# 1. Purpose

A production system needs to know:

```text
what exists
where it runs
what owns it
what it depends on
what depends on it
which release/configuration it uses
which region/tenant/security domain it belongs to
whether it is still valid
```

Without a trustworthy inventory, operational decisions become guesses.

Examples:

```text
Which services depend on this database?
Which workloads still use the vulnerable artifact?
What will fail if this region disappears?
Which configuration items are orphaned?
Which relay fleet is using an old trust root?
```

The governing principle is:

> **SIAR's operational graph should describe infrastructure and technical dependencies—not people, users, communications, or social relationships.**

---

# 2. Architectural Position

```text
Deployment / Discovery / Desired State / Providers
                      │
                      ▼
                 Asset Intake
                      │
                      ▼
              Canonical Inventory
                      │
                      ▼
               Typed Asset Graph
                      │
       ┌──────────────┼──────────────┐
       │              │              │
 Dependencies      Ownership       Placement
       │              │              │
       └──────────────┼──────────────┘
                      ▼
                 Reconciliation
                      │
                      ▼
           Operational Topology View
```

---

# 3. Core Separation

Keep distinct:

```text
asset
configuration item
service
workload instance
dependency
ownership
placement
runtime observation
user relationship
```

---

# 4. Non-Goals

Part 106 does not create:

```text
a user social graph
employee relationship graph
message flow graph
contact graph
per-user device topology
packet-level communication graph
```

---

# 5. Asset Identity

Every managed infrastructure object gets a typed identity.

```rust
pub struct AssetId(pub [u8; 16]);
```

---

# 6. Asset IDs Are Opaque

Do not encode:

```text
hostname
tenant name
region
human owner
user ID
IP address
```

---

# 7. Hard Rule

Asset identity remains stable across metadata changes but not across semantically new assets.

---

# 8. Asset Types

```rust
pub enum AssetType {
    Service,
    Workload,
    Host,
    VirtualMachine,
    Container,
    Database,
    ObjectStore,
    Queue,
    Stream,
    Cache,
    Gateway,
    Relay,
    MailboxProvider,
    MixNode,
    Hsm,
    SecretBroker,
    CertificateAuthority,
    LoadBalancer,
    DnsZone,
    StorageVolume,
    BackupRepository,
    Artifact,
    Region,
    FederationGateway,
}
```

---

# 9. No User Asset Type

Hard rule.

---

# 10. No Personal Device Asset By Default

Personal clients remain outside centralized infrastructure inventory unless explicitly enrolled as managed devices.

---

# 11. Managed Device Class

If needed:

```rust
pub enum ManagedAssetType {
    ManagedDeviceClass,
    ManagedEndpoint,
}
```

---

# 12. Managed Endpoint Scope

Organization-managed only.

---

# 13. Hard Rule

Inventory must not silently enumerate personal devices.

---

# 14. Configuration Item

A Configuration Item (CI) is an asset or logical operational object whose lifecycle/configuration matters.

```rust
pub struct ConfigurationItem {
    pub asset_id: AssetId,
    pub asset_type: AssetType,
    pub lifecycle: AssetLifecycleState,
    pub desired_state: Option<DesiredStateId>,
    pub owner: Option<ServiceOwnerRef>,
}
```

---

# 15. Asset Lifecycle

```rust
pub enum AssetLifecycleState {
    Planned,
    Provisioning,
    Active,
    Degraded,
    Quarantined,
    Deprecated,
    Decommissioning,
    Retired,
}
```

---

# 16. Planned

Exists in desired inventory but not runtime.

---

# 17. Provisioning

Being created.

---

# 18. Active

Serving approved purpose.

---

# 19. Degraded

Operating below target.

---

# 20. Quarantined

Isolated for security/health reason.

---

# 21. Deprecated

Still operating but scheduled for replacement.

---

# 22. Decommissioning

Being drained/removed.

---

# 23. Retired

No longer active.

---

# 24. Hard Rule

Retired assets cannot silently return to Active without a new authorized lifecycle transition.

---

# 25. Asset State Machine

Recommended:

```text
Planned
  ↓
Provisioning
  ↓
Active
  ↓ ↘
Degraded  Quarantined
  ↓          ↓
Active     Active
  ↓
Deprecated
  ↓
Decommissioning
  ↓
Retired
```

---

# 26. Lifecycle Transition

```rust
pub struct AssetLifecycleTransition {
    pub asset: AssetId,
    pub from: AssetLifecycleState,
    pub to: AssetLifecycleState,
    pub reason: AssetTransitionReason,
    pub at: CoarseTimestamp,
}
```

---

# 27. No Arbitrary Transition

Hard rule.

---

# 28. Asset Provenance

Every inventory asset must have a provenance source.

```rust
pub enum AssetProvenance {
    DesiredState,
    DeploymentController,
    CloudProvider,
    HostAttestation,
    ServiceRegistry,
    OperatorImported,
    FederationDescriptor,
}
```

---

# 29. Provenance Is Not Trust

Hard rule.

---

# 30. Asset Provenance Record

```rust
pub struct AssetProvenanceRecord {
    pub asset: AssetId,
    pub source: AssetProvenance,
    pub source_ref: ProvenanceRef,
    pub observed_at: CoarseTimestamp,
}
```

---

# 31. Inventory Confidence

```rust
pub enum InventoryConfidence {
    Declared,
    Observed,
    Verified,
    Attested,
}
```

---

# 32. Declared

Known from desired state.

---

# 33. Observed

Seen at runtime.

---

# 34. Verified

Corroborated.

---

# 35. Attested

Cryptographically measured/attested.

---

# 36. Hard Rule

Declared does not imply runtime existence.

---

# 37. Canonical Asset Record

```rust
pub struct AssetRecord {
    pub id: AssetId,
    pub asset_type: AssetType,
    pub lifecycle: AssetLifecycleState,
    pub owner: Option<ServiceOwnerRef>,
    pub service: Option<ServiceId>,
    pub environment: EnvironmentClass,
    pub region: Option<RegionId>,
    pub tenant: Option<TenantId>,
    pub provenance: InventoryConfidence,
}
```

---

# 38. No User Identity Field

Hard rule.

---

# 39. Technical Metadata

May include:

```text
artifact digest
desired state ID
software version
service identity
region
security domain
network zone
```

---

# 40. Do Not Include

```text
user contact
message content
employee activity
private account metadata
```

---

# 41. Hard rule.

---

# 42. Asset Names

Human-readable names allowed as labels.

---

# 43. Names Are Not Authority

Hard rule.

---

# 44. Immutable Identity vs Mutable Label

```rust
pub struct AssetLabel(pub String);
```

---

# 45. Renaming Does Not Change AssetId

Good.

---

# 46. Asset Namespace

Names scoped.

```rust
pub enum AssetNamespace {
    Service,
    Environment,
    Tenant,
    Region,
    Federation,
}
```

---

# 47. Avoid global name uniqueness requirement.

---

# 48. Hard rule.

---

# 49. Asset Discovery

Discovery sources should be authoritative where possible.

Preferred:

```text
desired-state manifests
deployment platform
cloud provider APIs
service registry
host attestation
database catalog
object-storage inventory
```

---

# 50. Active Network Scanning

Secondary tool.

---

# 51. Hard Rule

Broad network scanning is not the primary inventory mechanism.

---

# 52. No User Network Scanning

Hard rule.

---

# 53. Discovery Event

```rust
pub struct AssetObservation {
    pub source: AssetProvenance,
    pub external_ref: ExternalAssetRef,
    pub asset_type: AssetType,
    pub metadata_digest: Digest,
    pub observed_at: CoarseTimestamp,
}
```

---

# 54. Canonicalization

Observations map to canonical AssetId.

---

# 55. Hard Rule

Do not create duplicate assets merely because two discovery systems use different names.

---

# 56. Asset Resolver

```rust
pub trait AssetResolver {
    fn resolve(
        &self,
        observation: &AssetObservation,
    ) -> Result<AssetResolution, InventoryError>;
}
```

---

# 57. Resolution

```rust
pub enum AssetResolution {
    Existing(AssetId),
    NewCandidate(NewAssetCandidate),
    Conflict(AssetIdentityConflict),
}
```

---

# 58. Conflict

Requires reconciliation.

---

# 59. No silent merge.

---

# 60. Hard rule.

---

# 61. Identity Merge

High risk.

---

# 62. Two records may accidentally refer to same object.

---

# 63. Merge requires strong evidence.

---

# 64. No fuzzy-name merge baseline.

---

# 65. Hard rule.

---

# 66. Asset Retirement

When asset disappears:

```text
not immediately delete inventory
```

---

# 67. Mark missing/stale first.

---

# 68. Confirm lifecycle transition.

---

# 69. Hard rule.

---

# 70. Asset Presence State

```rust
pub enum AssetPresenceState {
    ExpectedAndObserved,
    ExpectedMissing,
    UnexpectedObserved,
    Retired,
}
```

---

# 71. ExpectedMissing

Potential outage/drift.

---

# 72. UnexpectedObserved

Potential unauthorized asset.

---

# 73. Hard rule.

---

# 74. Orphan Asset

Asset with no valid desired state/owner/parent.

---

# 75. Orphan Type

```rust
pub enum OrphanClass {
    NoOwner,
    NoDesiredState,
    NoParentService,
    StaleProviderResource,
    UnreferencedStorage,
    UnknownRuntime,
}
```

---

# 76. Orphan Does Not Mean Safe To Delete

Hard rule.

---

# 77. Orphan Workflow

```text
detect
→ classify
→ verify
→ owner review
→ retire/delete if authorized
```

---

# 78. Hard rule.

---

# 79. Asset Graph

Graph models typed technical relationships.

---

# 80. Graph Node

```rust
pub struct AssetNode {
    pub asset: AssetId,
    pub asset_type: AssetType,
}
```

---

# 81. Graph Edge

```rust
pub struct AssetEdge {
    pub from: AssetId,
    pub to: AssetId,
    pub kind: DependencyKind,
    pub confidence: EdgeConfidence,
}
```

---

# 82. Edge Direction

Must be defined consistently.

---

# 83. Recommended convention:

```text
A --DependsOn--> B
```

means A requires B.

---

# 84. Hard rule.

---

# 85. Dependency Types

```rust
pub enum DependencyKind {
    DependsOn,
    RunsOn,
    StoresIn,
    ReadsFrom,
    WritesTo,
    PublishesTo,
    ConsumesFrom,
    RoutesThrough,
    AuthenticatesWith,
    AuthorizedBy,
    UsesSecretBroker,
    UsesCertificateAuthority,
    ReplicatesTo,
    BackedUpTo,
    ManagedBy,
    DeployedFrom,
    LocatedIn,
    FederatesWith,
}
```

---

# 86. No User Relationship Edges

Hard rule.

---

# 87. No `CommunicatesWithUser`

Hard rule.

---

# 88. No Message Sender/Recipient Edges

Hard rule.

---

# 89. Dependency Semantics

Each edge type defines:

```text
direction
cardinality
failure semantics
security semantics
```

---

# 90. Hard rule.

---

# 91. Edge Confidence

```rust
pub enum EdgeConfidence {
    Declared,
    Observed,
    Verified,
}
```

---

# 92. Declared Dependency

From config/manifests.

---

# 93. Observed Dependency

Seen through infrastructure-level evidence.

---

# 94. Verified Dependency

Declared + observed or attested.

---

# 95. Hard Rule

Observed network traffic alone is insufficient for permanent dependency authority.

---

# 96. Desired Dependency Graph

Declared from architecture/config.

---

# 97. Runtime Dependency Graph

Observed actual state.

---

# 98. Effective Dependency Graph

Reconciled view.

---

# 99. Hard rule.

---

# 100. Dependency Drift

Examples:

```text
unexpected database dependency
missing required queue
service calling old endpoint
```

---

# 101. Part 105 integration.

---

# 102. Dependency Drift Type

```rust
pub enum DependencyDrift {
    MissingRequiredEdge,
    UnexpectedEdge,
    EndpointChanged,
    DependencyVersionMismatch,
}
```

---

# 103. Unexpected Edge

Potential security issue.

---

# 104. Hard rule.

---

# 105. Service Dependency Mapping

Primary graph level.

---

# 106. Service Node

```rust
pub struct ServiceTopologyNode {
    pub service: ServiceId,
    pub owner: ServiceOwnerRef,
    pub criticality: ServiceCriticality,
}
```

---

# 107. Service Criticality

```rust
pub enum ServiceCriticality {
    Low,
    Standard,
    High,
    Critical,
}
```

---

# 108. Dependency Criticality

```rust
pub enum DependencyCriticality {
    Optional,
    Degradable,
    Required,
    Critical,
}
```

---

# 109. Service Dependency

```rust
pub struct ServiceDependency {
    pub consumer: ServiceId,
    pub provider: ServiceId,
    pub criticality: DependencyCriticality,
}
```

---

# 110. Optional

Feature can disappear.

---

# 111. Degradable

Service can continue in degraded mode.

---

# 112. Required

Core capability fails.

---

# 113. Critical

Security/control integrity depends on it.

---

# 114. Hard Rule

Dependency criticality must be explicit for continuity planning.

---

# 115. Dependency Failure Policy

```rust
pub enum DependencyFailurePolicy {
    Continue,
    Degrade,
    Queue,
    ReadOnly,
    FailClosed,
}
```

---

# 116. Security-Critical Dependencies

Usually FailClosed.

---

# 117. Hard rule.

---

# 118. Dependency Cycles

Some cycles possible but dangerous.

---

# 119. Detect strongly connected components.

---

# 120. Control-plane boot cycles especially prohibited.

---

# 121. Hard rule.

---

# 122. Boot Dependency Graph

Separate projection.

---

# 123. Example:

```text
secret broker
→ workload identity
→ service discovery
→ application
```

---

# 124. No circular boot dependency.

---

# 125. Hard rule.

---

# 126. Data Dependency Graph

Models technical data stores.

---

# 127. Example:

```text
service
→ database
→ backup repository
```

---

# 128. Not data-content lineage in full.

---

# 129. Hard rule.

---

# 130. Security Dependency Graph

Projection:

```text
service
→ authz
→ CA
→ secret broker
```

---

# 131. Helps blast-radius analysis.

---

# 132. Good.

---

# 133. Privacy Dependency Graph

Projection:

```text
private service
→ privacy policy
→ anonymous route
→ mailbox provider
```

---

# 134. Helps prevent fallback mistakes.

---

# 135. No user graph.

---

# 136. Hard rule.

---

# 137. Regional Topology

Map assets to regions.

---

# 138. `LocatedIn` edge.

---

# 139. Region Failure Query

```text
Which services lose Required/Critical dependencies?
```

---

# 140. Part 100/103 integration.

---

# 141. Hard rule.

---

# 142. Tenant Topology

Tenant-managed shared resources may be scoped.

---

# 143. Never expose Tenant A graph to Tenant B.

---

# 144. Hard rule.

---

# 145. Personal Data Exclusion

No personal user resources represented unless managed policy explicitly enrolls them.

---

# 146. Hard rule.

---

# 147. Federation Topology

Model domain-level edges only.

---

# 148. Example:

```text
local federation gateway
→ remote federation domain
```

---

# 149. Do not ingest remote user/service internal graph unless explicitly shared for interoperability.

---

# 150. Hard rule.

---

# 151. Federation Edge

```rust
pub struct FederationTopologyEdge {
    pub local_gateway: AssetId,
    pub remote_domain: FederationDomainId,
    pub capability_set: FederationCapabilitySet,
}
```

---

# 152. No remote user identities.

---

# 153. Hard rule.

---

# 154. Topology Views

Different roles need different projections.

---

# 155. Views:

```text
service
security
network
storage
regional
recovery
tenant
federation
```

---

# 156. No one unrestricted graph by default.

---

# 157. Hard rule.

---

# 158. Graph Projection

```rust
pub enum TopologyProjection {
    Service,
    Security,
    Storage,
    Region,
    Recovery,
    Tenant,
    Federation,
}
```

---

# 159. Authorization filters nodes/edges.

---

# 160. Hard rule.

---

# 161. Graph Query API

Typed query language.

---

# 162. Avoid arbitrary Cypher/SQL to untrusted callers.

---

# 163. Hard rule.

---

# 164. Query Examples

```text
dependencies(service)
dependents(asset)
assets_in_region(region)
blast_radius(asset)
orphans(scope)
```

---

# 165. Topology Query

```rust
pub enum TopologyQuery {
    Dependencies(AssetId),
    Dependents(AssetId),
    AssetsInRegion(RegionId),
    AssetsForService(ServiceId),
    Orphans(StateScope),
    BlastRadius(AssetId),
}
```

---

# 166. Bounded Traversal

Required.

---

# 167. Hard rule.

---

# 168. Query Budget

```rust
pub struct GraphQueryBudget {
    pub max_depth: u8,
    pub max_nodes: usize,
    pub max_edges: usize,
}
```

---

# 169. Prevent graph DoS.

---

# 170. Hard rule.

---

# 171. Blast Radius

Computes what may be affected by failure/compromise.

---

# 172. Blast Radius Result

```rust
pub struct BlastRadius {
    pub origin: AssetId,
    pub affected_services: Vec<ServiceId>,
    pub critical_paths: Vec<DependencyPath>,
}
```

---

# 173. Use edge semantics.

---

# 174. Not all dependents equally affected.

---

# 175. Hard rule.

---

# 176. Failure Propagation

Depends on:

```text
edge criticality
redundancy
fallback
region
```

---

# 177. No simple "all connected nodes fail."

---

# 178. Hard rule.

---

# 179. Redundancy Modeling

```rust
pub enum DependencyRedundancy {
    None,
    ActivePassive,
    ActiveActive,
    NOfM { required: u8, total: u8 },
}
```

---

# 180. Good for DR/capacity.

---

# 181. Hard rule.

---

# 182. Dependency Path

```rust
pub struct DependencyPath {
    pub nodes: Vec<AssetId>,
    pub edges: Vec<DependencyKind>,
}
```

---

# 183. Bound path length.

---

# 184. Hard rule.

---

# 185. Critical Path

Path with required/critical dependencies.

---

# 186. Useful for SLO/DR.

---

# 187. Hard rule.

---

# 188. Single Point Of Failure Detection

Graph can identify:

```text
one dependency with no redundant path
```

---

# 189. SPOF Finding

```rust
pub struct SinglePointOfFailure {
    pub asset: AssetId,
    pub affected_services: Vec<ServiceId>,
}
```

---

# 190. No automatic conclusion if topology confidence low.

---

# 191. Hard rule.

---

# 192. Topology Confidence

Aggregate from nodes/edges.

---

# 193. Unknown/incomplete graph explicit.

---

# 194. No false completeness claim.

---

# 195. Hard rule.

---

# 196. Inventory Freshness

Each observation expires.

---

# 197. Freshness Class

```rust
pub enum InventoryFreshness {
    Fresh,
    Aging,
    Stale,
    Unknown,
}
```

---

# 198. Stale does not mean absent.

---

# 199. Hard rule.

---

# 200. Freshness Policy

Per asset class.

---

# 201. Workload observations shorter TTL.

---

# 202. Region definitions longer TTL.

---

# 203. Hard rule.

---

# 204. Inventory Reconciliation

Compare:

```text
desired inventory
provider/runtime observations
asset graph
```

---

# 205. Reconciliation Outcomes

```rust
pub enum InventoryReconciliation {
    Match,
    MissingExpected,
    UnexpectedObserved,
    MetadataConflict,
    DependencyConflict,
}
```

---

# 206. No silent delete/merge.

---

# 207. Hard rule.

---

# 208. Inventory Reconciler

```rust
pub trait InventoryReconciler {
    fn reconcile(
        &self,
        scope: StateScope,
    ) -> Result<Vec<InventoryFinding>, InventoryError>;
}
```

---

# 209. Configuration Drift Integration

Part 105.

---

# 210. Inventory says *what exists*.

---

# 211. Desired-state governance says *what configuration it should have*.

---

# 212. Hard separation.

---

# 213. Asset Discovery vs Service Discovery

Different.

---

# 214. Asset inventory

long-lived operational knowledge.

---

# 215. Service discovery

runtime endpoint resolution.

---

# 216. Hard rule.

---

# 217. DNS Is Not Inventory

Hard rule.

---

# 218. Service Registry Is Not Full Inventory

Hard rule.

---

# 219. Cloud Provider Is Not Sole Inventory

Hard rule.

---

# 220. Multi-Source Reconciliation

Required.

---

# 221. Asset Source Priority

Example:

```text
signed desired state
deployment controller
provider
runtime observation
```

---

# 222. But conflicts surfaced.

---

# 223. No silent priority override for identity conflicts.

---

# 224. Hard rule.

---

# 225. Asset Ownership

Part 104.

---

# 226. Production asset should inherit/declare service owner.

---

# 227. Orphan `NoOwner` is governance issue.

---

# 228. Hard rule.

---

# 229. Asset Responsibility

Infrastructure responsibility can differ from application owner.

---

# 230. Example:

```text
DB platform owned by storage team
logical schema owned by app team
```

---

# 231. Model separately.

---

# 232. Hard rule.

---

# 233. Ownership Edge

`ManagedBy`.

---

# 234. Do not turn team memberships into graph edges.

---

# 235. Hard rule.

---

# 236. Artifact Dependency

`DeployedFrom`.

---

# 237. Asset tracks exact release/artifact digest.

---

# 238. Good for vulnerability response.

---

# 239. Hard rule.

---

# 240. Vulnerability Integration

Part 98.

---

# 241. Query:

```text
Which active assets run artifact X?
```

---

# 242. Exposure scope from inventory.

---

# 243. No user data.

---

# 244. Hard rule.

---

# 245. Update Integration

Part 99.

---

# 246. After rollout:

```text
inventory should show new artifact digest
```

---

# 247. Old digest still active => drift/exposure.

---

# 248. Hard rule.

---

# 249. Host Attestation Integration

Part 71.

---

# 250. Attested host/workload state can elevate confidence.

---

# 251. No attestation of user identity.

---

# 252. Hard rule.

---

# 253. Supply-Chain Integration

Part 73.

---

# 254. Artifact nodes connect to:

```text
source revision
SBOM/provenance
release
```

---

# 255. Source code repository may be represented as artifact provenance reference, not social metadata.

---

# 256. Hard rule.

---

# 257. Database Integration

Part 74.

---

# 258. Database asset includes:

```text
engine
cluster ID
region
role
schema version
```

---

# 259. No table row/user data in inventory.

---

# 260. Hard rule.

---

# 261. Queue/Event Integration

Part 75.

---

# 262. Queue/stream asset includes:

```text
retention
schema
producer services
consumer services
```

---

# 263. No event payload.

---

# 264. Hard rule.

---

# 265. Consensus Integration

Part 76.

---

# 266. Consensus cluster topology modeled.

---

# 267. Voter/learner role.

---

# 268. No runtime election history needed for inventory.

---

# 269. Hard rule.

---

# 270. Service Discovery Integration

Part 77.

---

# 271. Service-discovery records link to asset/workload.

---

# 272. Stale registry endpoint can be detected.

---

# 273. Hard rule.

---

# 274. Edge Integration

Part 78.

---

# 275. Gateways/load balancers are assets.

---

# 276. Routing dependencies explicit.

---

# 277. Hard rule.

---

# 278. East-West Integration

Part 79.

---

# 279. Service identity relationships can be modeled as authorization dependency.

---

# 280. Do not record every request.

---

# 281. Hard rule.

---

# 282. Secrets Integration

Part 80.

---

# 283. Model dependency on SecretBroker/SecretClass.

---

# 284. Do not inventory secret values.

---

# 285. Hard rule.

---

# 286. Authorization Integration

Part 81.

---

# 287. Service can link to policy bundle.

---

# 288. No user authorization graph.

---

# 289. Hard rule.

---

# 290. Identity Provider Integration

Part 82.

---

# 291. IdP is service asset.

---

# 292. No account graph imported.

---

# 293. Hard rule.

---

# 294. Tenant Integration

Part 69.

---

# 295. Shared platform assets remain platform scope.

---

# 296. Tenant-dedicated assets can carry TenantId.

---

# 297. Tenant A cannot enumerate B.

---

# 298. Hard rule.

---

# 299. Federation Integration

Parts 58/103.

---

# 300. Local graph exposes peer-domain edge only.

---

# 301. No global federated asset supergraph baseline.

---

# 302. Hard rule.

---

# 303. Geographic Governance Integration

Part 103.

---

# 304. Region/jurisdiction placement attached to asset.

---

# 305. Query:

```text
assets in prohibited region
```

---

# 306. No precise user location.

---

# 307. Hard rule.

---

# 308. Capacity Integration

Part 101.

---

# 309. Capacity data links to asset/service class.

---

# 310. Inventory contains configured capacity, not detailed time-series metrics.

---

# 311. Hard rule.

---

# 312. FinOps Integration

Part 102.

---

# 313. Cost attribution can reference AssetId/ServiceId.

---

# 314. Inventory does not become billing event store.

---

# 315. Hard rule.

---

# 316. DR Integration

Part 100.

---

# 317. Recovery topology projection includes:

```text
primary DB
replicas
backup repository
failover region
```

---

# 318. Query:

```text
Does service have a compliant recovery path?
```

---

# 319. Hard rule.

---

# 320. Incident Integration

Part 96.

---

# 321. Incident scope can use AssetId.

---

# 322. Blast-radius queries support containment.

---

# 323. Incident system does not gain unrestricted graph visibility automatically.

---

# 324. Hard rule.

---

# 325. SOC Integration

Part 97.

---

# 326. IoCs can match assets:

```text
artifact digest
cert fingerprint
endpoint
```

---

# 327. No user identifiers.

---

# 328. Hard rule.

---

# 329. Compliance Integration

Part 95.

---

# 330. Controls can query:

```text
production asset has owner
approved region
approved artifact
backup dependency
```

---

# 331. Evidence derived from topology without user data.

---

# 332. Hard rule.

---

# 333. Audit Integration

Part 94.

---

# 334. Audit:

```text
asset lifecycle transition
identity merge
critical dependency change
manual asset import
```

---

# 335. Not every topology read.

---

# 336. Hard rule.

---

# 337. Organizational Governance Integration

Part 104.

---

# 338. Asset/service owner references operational roles.

---

# 339. No workforce org chart replication.

---

# 340. Hard rule.

---

# 341. Asset Tags

Useful metadata.

---

# 342. Typed labels preferred.

---

# 343. Example:

```rust
pub enum AssetTag {
    Production,
    Critical,
    InternetFacing,
    Stateful,
    Managed,
}
```

---

# 344. Avoid arbitrary high-cardinality free-form tags in core.

---

# 345. Hard rule.

---

# 346. Environment

```rust
pub enum EnvironmentClass {
    Development,
    Testing,
    Staging,
    Production,
}
```

---

# 347. Production rules stricter.

---

# 348. Hard rule.

---

# 349. Network Zone

```rust
pub enum NetworkZone {
    PublicEdge,
    ServiceMesh,
    ControlPlane,
    DataPlane,
    Management,
    Isolated,
}
```

---

# 350. Topology uses coarse logical zone.

---

# 351. No packet path history.

---

# 352. Hard rule.

---

# 353. Security Zone Crossing

Graph can flag unexpected dependencies across zones.

---

# 354. Good security signal.

---

# 355. Hard rule.

---

# 356. Asset Criticality

```rust
pub enum AssetCriticality {
    Low,
    Standard,
    High,
    Critical,
}
```

---

# 357. Derived from service/control role.

---

# 358. Not manually inflated without policy.

---

# 359. Hard rule.

---

# 360. Topology Change

Any edge/node mutation.

---

# 361. Topology Change Record

```rust
pub struct TopologyChange {
    pub change_id: TopologyChangeId,
    pub scope: StateScope,
    pub change: TopologyMutation,
    pub observed_at: CoarseTimestamp,
}
```

---

# 362. Mutation Types

```rust
pub enum TopologyMutation {
    AssetAdded(AssetId),
    AssetRetired(AssetId),
    EdgeAdded(AssetEdge),
    EdgeRemoved(AssetEdge),
    PlacementChanged(AssetId),
}
```

---

# 363. No Global Clickstream

Hard rule.

---

# 364. Topology Change Validation

Critical edge changes may require change-authority evidence.

---

# 365. Example:

```text
service changes authorization provider
```

---

# 366. Hard rule.

---

# 367. Unexpected Topology Change

SOC/drift signal.

---

# 368. No automatic compromise conclusion.

---

# 369. Hard rule.

---

# 370. CMDB Boundary

This architecture provides CMDB-like technical functions.

---

# 371. But avoids classic CMDB failure patterns:

```text
manual stale spreadsheets
unbounded schema
employee/org graph
duplicate assets
```

---

# 372. Hard rule.

---

# 373. Configuration Item Schema

Typed and extensible.

---

# 374. Core fields stable.

---

# 375. Asset-specific metadata stored in typed variants.

---

# 376. Example:

```rust
pub enum AssetMetadata {
    Service(ServiceAssetMetadata),
    Database(DatabaseAssetMetadata),
    Host(HostAssetMetadata),
    Queue(QueueAssetMetadata),
    Region(RegionAssetMetadata),
}
```

---

# 377. No `serde_json::Value` Core Blob

Preferred hard rule.

---

# 378. Versioned Metadata

```rust
pub struct AssetSchemaVersion(pub u16);
```

---

# 379. Unknown newer critical metadata fails safely.

---

# 380. Hard rule.

---

# 381. Asset History

Keep lifecycle/configuration history at meaningful boundaries.

---

# 382. Do not store every heartbeat forever.

---

# 383. Hard rule.

---

# 384. History Events

Examples:

```text
created
owner changed
region changed
artifact changed
retired
```

---

# 385. No employee activity history.

---

# 386. Hard rule.

---

# 387. Asset Tombstone

Retired/deleted asset leaves tombstone to prevent ID reuse/confusion.

---

# 388. Tombstone

```rust
pub struct AssetTombstone {
    pub asset: AssetId,
    pub retired_at: CoarseTimestamp,
    pub final_type: AssetType,
}
```

---

# 389. AssetId Never Reused

Hard rule.

---

# 390. Garbage Collection

Retired detailed metadata can expire.

---

# 391. Tombstone remains minimum needed period.

---

# 392. Hard rule.

---

# 393. Asset Import

Manual/imported assets validated.

---

# 394. No direct trust.

---

# 395. Hard rule.

---

# 396. Asset Export

Scoped technical inventory export.

---

# 397. Redact:

```text
sensitive internal endpoints
security metadata
tenant identifiers if not required
```

---

# 398. Hard rule.

---

# 399. Graph Export

No unrestricted full production graph by default.

---

# 400. Use scoped projections.

---

# 401. Hard rule.

---

# 402. External Audit

Can receive signed inventory evidence snapshot.

---

# 403. Not live unrestricted graph access.

---

# 404. Hard rule.

---

# 405. Inventory Snapshot

```rust
pub struct InventorySnapshot {
    pub snapshot_id: InventorySnapshotId,
    pub scope: StateScope,
    pub asset_root: Digest,
    pub edge_root: Digest,
    pub generated_at: CoarseTimestamp,
}
```

---

# 406. Signed.

---

# 407. Useful for assurance.

---

# 408. Hard rule.

---

# 409. Merkle Root

Optional for large snapshots.

---

# 410. Use proven crypto implementation.

---

# 411. No custom cryptography.

---

# 412. Hard rule.

---

# 413. Topology Privacy

Infrastructure topology itself is sensitive.

---

# 414. Access must be least privilege.

---

# 415. Hard rule.

---

# 416. Attacker Value

Graph can reveal:

```text
critical dependencies
regions
security services
single points of failure
```

---

# 417. Therefore encrypt at rest and in transit.

---

# 418. Hard rule.

---

# 419. Topology Authorization

Projection/action based.

---

# 420. Roles do not automatically see all edges.

---

# 421. Hard rule.

---

# 422. Tenant View

Tenant sees:

```text
tenant-dedicated assets
approved shared service abstractions
```

---

# 423. Not platform internals.

---

# 424. Hard rule.

---

# 425. Federation View

Peer sees:

```text
published endpoints/capabilities
```

---

# 426. Not internal topology.

---

# 427. Hard rule.

---

# 428. Public View

Minimal public service metadata only.

---

# 429. No internal graph.

---

# 430. Hard rule.

---

# 431. Graph Query Privacy

Queries themselves may reveal sensitive intent.

---

# 432. Do not log raw arbitrary query text.

---

# 433. Typed query enums avoid this.

---

# 434. Hard rule.

---

# 435. Query Audit

Audit only high-risk graph exports/administrative changes.

---

# 436. Not every dependency lookup.

---

# 437. Hard rule.

---

# 438. Graph Storage Model

Options:

```text
SQL adjacency tables
embedded graph DB
dedicated graph engine
```

---

# 439. Baseline Recommendation

Keep authoritative inventory in transactional SQL.

---

# 440. Derived graph index can optimize traversal.

---

# 441. Hard rule.

---

# 442. Why

Lifecycle/ownership/inventory mutation needs strong transactions.

---

# 443. Graph Projection Rebuildable

Derived index is rebuildable.

---

# 444. Good.

---

# 445. SQL Schema Concept

```text
assets
asset_metadata
asset_edges
asset_provenance
asset_lifecycle_events
asset_tombstones
```

---

# 446. Tenant partition where applicable.

---

# 447. Hard rule.

---

# 448. Edge Uniqueness

Unique key:

```text
(from, to, kind, scope)
```

---

# 449. Prevent duplicates.

---

# 450. Hard rule.

---

# 451. Transaction Boundary

Asset+critical dependency changes can be atomic.

---

# 452. Outbox event after commit.

---

# 453. Hard rule.

---

# 454. Event Projection

Topology change events feed:

```text
drift
SOC
vulnerability
capacity
DR
```

---

# 455. Minimal payload.

---

# 456. No full graph broadcast.

---

# 457. Hard rule.

---

# 458. Event Type

```rust
pub enum InventoryEvent {
    AssetCreated,
    AssetStateChanged,
    AssetRetired,
    DependencyChanged,
    OwnershipChanged,
    PlacementChanged,
}
```

---

# 459. Event Contains IDs/typed metadata only.

---

# 460. Hard rule.

---

# 461. Inventory API

```rust
pub trait InventoryService {
    fn asset(
        &self,
        id: AssetId,
    ) -> Result<AssetRecord, InventoryError>;

    fn assets_for_service(
        &self,
        service: ServiceId,
    ) -> Result<Vec<AssetRecord>, InventoryError>;
}
```

---

# 462. Graph API

```rust
pub trait TopologyService {
    fn query(
        &self,
        query: TopologyQuery,
        budget: GraphQueryBudget,
    ) -> Result<TopologyResult, InventoryError>;
}
```

---

# 463. Lifecycle API

```rust
pub trait AssetLifecycleService {
    fn transition(
        &self,
        asset: AssetId,
        target: AssetLifecycleState,
    ) -> Result<(), InventoryError>;
}
```

---

# 464. No Generic Graph Mutation API

Hard rule.

---

# 465. Dependency Mutation

Typed/change-authorized.

---

# 466. Hard rule.

---

# 467. Inventory Error Taxonomy

```rust
pub enum InventoryError {
    AssetNotFound,
    AssetIdentityConflict,
    InvalidLifecycleTransition,
    DependencyCycle,
    ScopeViolation,
    QueryBudgetExceeded,
    StaleInventory,
    ProvenanceInvalid,
    Unauthorized,
    Internal,
}
```

---

# 468. Inventory Observability

Safe metrics:

```text
assets by type/state
stale assets
orphan count
dependency conflicts
inventory reconciliation lag
```

---

# 469. Forbidden:

```text
user activity
employee behavior
private communication topology
```

---

# 470. Hard rule.

---

# 471. Inventory SLOs

Examples:

```text
production asset discovery freshness
dependency reconciliation latency
orphan detection latency
critical topology query availability
```

---

# 472. Security SLO

```text
0 unauthorized assets silently Active
0 retired AssetId reused
0 unbounded topology query
```

---

# 473. Privacy SLO

```text
0 user social graph edges
0 personal-device enumeration by default
0 message-flow topology retained
```

---

# 474. Failure Modes

```text
provider API outage
duplicate asset observations
stale inventory
graph index corruption
partial discovery
```

---

# 475. Provider API Outage

Use existing inventory with freshness downgrade.

---

# 476. Do not mark assets deleted.

---

# 477. Hard rule.

---

# 478. Duplicate Observation

Resolve/correlate.

---

# 479. Do not silently merge.

---

# 480. Hard rule.

---

# 481. Stale Inventory

Mark Aging/Stale.

---

# 482. Critical decisions may require refresh.

---

# 483. Hard rule.

---

# 484. Graph Index Corruption

Rebuild from authoritative asset/edge tables.

---

# 485. Hard rule.

---

# 486. Partial Discovery

Completeness explicitly Unknown.

---

# 487. No false "all assets healthy."

---

# 488. Hard rule.

---

# 489. Testing

Need inventory/topology testkit.

---

# 490. Test Scenarios

```text
new deployment
orphan VM
unexpected dependency
region failure
retired asset reappearance
```

---

# 491. Identity Test

Rename does not change AssetId.

---

# 492. Duplicate Test

Conflicting observations do not silently merge.

---

# 493. Lifecycle Test

Invalid transition rejected.

---

# 494. Retirement Test

Retired AssetId not reused.

---

# 495. Presence Test

Provider outage does not delete assets.

---

# 496. Orphan Test

Unowned/unreferenced asset detected.

---

# 497. Dependency Test

Required edge missing surfaced.

---

# 498. Unexpected Edge Test

New undeclared dependency surfaced.

---

# 499. Cycle Test

Forbidden boot/control cycle rejected.

---

# 500. Blast Radius Test

Critical paths calculated with dependency semantics.

---

# 501. Redundancy Test

Redundant providers reduce blast radius correctly.

---

# 502. Tenant Test

Tenant A cannot query B topology.

---

# 503. Federation Test

Remote peer sees no internal graph.

---

# 504. Privacy Test

No user/social/message edges accepted.

---

# 505. Vulnerability Test

Artifact digest maps to affected assets.

---

# 506. DR Test

Recovery topology identifies compliant failover dependencies.

---

# 507. Drift Test

Observed topology mismatch feeds Part 105.

---

# 508. Update Test

Old artifact remains visible until actual rollout converges.

---

# 509. Graph Budget Test

Traversal stops at limits.

---

# 510. Snapshot Test

Signed inventory snapshot verifies.

---

# 511. Fuzzing

Fuzz:

```text
asset record
asset metadata variants
dependency edges
topology queries
lifecycle transitions
```

---

# 512. Property Tests

Properties:

```text
retired AssetId can never become a new unrelated asset
tenant-scoped query can never return another tenant's dedicated asset
graph traversal can never exceed configured node/edge/depth budget
user/social/message identity can never become a valid infrastructure dependency edge
```

---

# 513. Formal Verification Targets

Strong candidates:

```text
asset lifecycle state machine
dependency graph invariants
tenant projection isolation
blast-radius traversal bounds
```

---

# 514. Kani Candidate

lifecycle/dependency-type/scope invariants.

---

# 515. TLA+ Candidate

desired asset → provision → observe → active → deprecate → retire.

---

# 516. Loom Candidate

concurrent discovery + retirement + edge update.

---

# 517. Performance

Inventory writes are moderate.

---

# 518. Graph reads may be heavy.

---

# 519. Use indexes/projections.

---

# 520. No full graph scan for ordinary queries.

---

# 521. Hard rule.

---

# 522. Dependency Indexes

Index:

```text
from asset
to asset
kind
service
region
tenant
```

---

# 523. No user index.

---

# 524. Hard rule.

---

# 525. Graph Cache

Derived/rebuildable.

---

# 526. Scope-aware.

---

# 527. Short TTL for operational state.

---

# 528. Hard rule.

---

# 529. Snapshot Strategy

Periodic signed inventory snapshot for recovery/assurance.

---

# 530. Runtime graph reconstructed from DB + events.

---

# 531. Hard rule.

---

# 532. Backup

Inventory authoritative data backed up.

---

# 533. Restore reconciles against current providers/desired state before Active.

---

# 534. No stale asset resurrection.

---

# 535. Hard rule.

---

# 536. Anti-Resurrection

Asset tombstones/revocation epochs survive restore.

---

# 537. Hard rule.

---

# 538. Migration

Schema migration follows expand-migrate-contract.

---

# 539. Graph indexes rebuildable.

---

# 540. Hard rule.

---

# 541. Crate Layout

Recommended:

```text
crates/
├── siar-inventory-core/
├── siar-asset-identity/
├── siar-asset-lifecycle/
├── siar-asset-discovery/
├── siar-inventory-reconciliation/
├── siar-topology-graph/
├── siar-dependency-model/
├── siar-blast-radius/
├── siar-topology-projection/
├── siar-inventory-snapshot/
├── siar-inventory-observability/
└── siar-inventory-testkit/
```

---

# 542. `siar-inventory-core`

Owns:

```text
AssetId
AssetType
AssetRecord
InventoryError
```

---

# 543. `siar-asset-identity`

Canonicalization/provenance/conflict handling.

---

# 544. `siar-asset-lifecycle`

Lifecycle transitions/tombstones.

---

# 545. `siar-asset-discovery`

Provider/deployment/attestation adapters.

---

# 546. `siar-inventory-reconciliation`

Expected vs observed inventory.

---

# 547. `siar-topology-graph`

Transactional edge model + derived traversal indexes.

---

# 548. `siar-dependency-model`

Typed dependency semantics/criticality/failure policy.

---

# 549. `siar-blast-radius`

Bounded dependency impact analysis.

---

# 550. `siar-topology-projection`

Service/security/storage/region/tenant/federation views.

---

# 551. `siar-inventory-snapshot`

Signed inventory/evidence snapshots.

---

# 552. `siar-inventory-observability`

Inventory health/freshness only.

---

# 553. `siar-inventory-testkit`

identity/lifecycle/graph/privacy/DR tests.

---

# 554. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. The infrastructure inventory models technical assets, services, workloads, providers, regions, storage, security infrastructure, and configuration items—not users, contacts, messages, social relationships, or ordinary personal devices.
2. AssetId is opaque, never reused, and distinct from mutable names, IP addresses, hostnames, provider IDs, tenant names, or human owners.
3. Every active production asset has explicit lifecycle state, provenance, scope, and operational ownership; missing ownership/provenance is surfaced rather than silently accepted.
4. Desired, observed, and effective inventory/topology are distinct; discovery observations cannot silently override signed desired state or lifecycle authority.
5. Dependency edges are typed, directional, scoped, confidence-labeled, and bounded; observed traffic alone cannot become permanent dependency authority.
6. Tenant and federation topology projections are isolated: tenants cannot enumerate one another, peers cannot inspect local internal topology, and platform operators do not receive personal social/device graphs.
7. Graph queries are typed, authorization-filtered, and bounded by traversal depth/node/edge budgets so topology cannot become an unrestricted graph-query or denial-of-service surface.
8. Orphan, unexpected, stale, missing, or conflicting assets are explicit findings and are never automatically deleted or merged without verification.
9. Asset lifecycle, retirement tombstones, and restore rules prevent old or retired configuration items from being resurrected or their identities reused after recovery.
10. Blast-radius and dependency analysis respects criticality, redundancy, failover, and uncertainty; graph connectivity alone is never treated as proof of failure propagation.
11. Inventory snapshots, exports, observability, and audit evidence expose only the technical scope required and never become a workforce-monitoring, user-tracking, message-flow, or surveillance dataset.
12. Inventory/topology integrates with desired state, vulnerability management, SOC, incident response, DR, capacity, FinOps, residency, ownership, deployment, and compliance while remaining a technical source of operational truth rather than a universal organizational or behavioral graph.
```

---

# 555. Initial Production Scope

Implement first:

```text
typed AssetId/AssetType
canonical asset records
asset lifecycle state machine
asset tombstones
provenance/confidence model
desired/deployment/provider discovery adapters
presence state
orphan detection
typed dependency edges
service dependency criticality
SQL authoritative inventory
derived graph indexes
typed bounded topology queries
tenant/region/federation projections
blast-radius analysis
ownership/desired-state links
artifact/version links
inventory reconciliation
signed inventory snapshots
audit/compliance hooks
privacy-safe inventory metrics
inventory/topology testkit
```

Then add:

```text
advanced service-map visualization
multi-source dependency confidence scoring
topology simulation
failure-mode graph modeling
formal dependency-cycle verification
privacy-preserving cross-federation topology attestations
```

---

# 556. Definition of Done

Part 106 is complete when:

- infrastructure assets have opaque canonical identities
- active assets have lifecycle/provenance/owner/scope
- retired asset IDs cannot be reused
- discovery sources reconcile rather than overwrite
- expected/missing/unexpected/orphan states are explicit
- dependencies are typed/directional/criticality-aware
- tenant/federation projections are isolated
- graph queries are bounded and authorization-filtered
- blast-radius analysis understands redundancy and uncertainty
- artifact/version/region/owner/desired-state links exist
- inventory integrates with vulnerability/SOC/DR/configuration governance
- no user/social/message graph is created
- lifecycle/graph/privacy/fuzz/formal tests are specified

---

# 557. Final Architecture

```text
            DESIRED STATE / DEPLOYMENT / PROVIDERS
                            │
                            ▼
                     ASSET DISCOVERY
                            │
                            ▼
                   CANONICAL INVENTORY
                            │
                            ▼
                    TYPED ASSET GRAPH
                            │
             ┌──────────────┼──────────────┐
             │              │              │
        DEPENDENCIES     OWNERSHIP      PLACEMENT
             │              │              │
             └──────────────┼──────────────┘
                            ▼
                    RECONCILIATION
                            │
                            ▼
               OPERATIONAL PROJECTIONS
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
     BLAST RADIUS       DR/SECURITY       COMPLIANCE
```

Inventory safety model:

```text
opaque asset identities
+
typed lifecycle
+
multi-source provenance
+
typed dependency edges
+
bounded graph queries
+
scope-specific projections
+
reconciliation
+
privacy-safe inventory evidence
```

not:

```text
scan everything, merge anything with a similar name, record who communicates with whom, and call the resulting giant graph operational inventory
```

---

# 558. Final Principle

An operational topology should answer how infrastructure is connected without answering how people are connected.

The correct model is:

```text
identify assets canonically
+
track lifecycle explicitly
+
reconcile multiple technical sources
+
model dependencies semantically
+
bound graph traversal
+
project only authorized topology
+
preserve provenance and uncertainty
+
never turn infrastructure inventory into a user or workforce graph
```

This architecture gives SIAR a privacy-preserving operational source of truth for infrastructure inventory, configuration-item lifecycle, service dependency mapping, blast-radius analysis, asset ownership, regional topology, vulnerability exposure, recovery planning, and operational assurance while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–105.
