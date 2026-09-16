# Core System Architecture Part 42 — Anonymity Threat Model, Traffic Analysis, Correlation Attacks & Formal Privacy Verification Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 42  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–41  
**Primary purpose:** define the complete anonymity threat model, attack taxonomy, privacy invariants, correlation-analysis methodology, simulation architecture, formal-verification targets, red-team program, privacy metrics, and release gates for SIAR's high-anonymity subsystem.

---

# 1. Purpose

SIAR now has architecture for:

```text
mixnet transport
anonymous mailboxes
Sphinx-style framing
cover traffic
native mix nodes
directory/governance
anonymous bulk transfer
censorship resistance
```

The next requirement is to define exactly:

```text
what anonymity means
which adversaries are considered
what attacks are in scope
which properties must hold
how those properties are tested
how implementation regressions are detected
```

The governing principle is:

> **SIAR must never treat "uses a mixnet" as proof of anonymity. Anonymity is a set of explicit properties that must survive defined adversaries and be continuously verified.**

---

# 2. Architectural Position

```text
Implementation
   │
   ▼
Threat Model
   │
   ▼
Privacy Invariants
   │
   ▼
Simulation / Model Checking / Red Team
   │
   ▼
Evidence
   │
   ▼
Release Privacy Gates
```

---

# 3. Security vs Privacy

Security properties include:

```text
confidentiality
integrity
authentication
availability
```

Privacy/anonymity properties include:

```text
sender hiding
receiver hiding
relationship hiding
traffic-shape hiding
unlinkability
unobservability
```

These are related but not identical.

---

# 4. Core Anonymity Properties

Define explicit target properties.

---

# 5. Sender Anonymity

Observer should not easily determine:

```text
which user originated a given anonymous message
```

---

# 6. Receiver Anonymity

Observer should not easily determine:

```text
which user receives a given message
```

---

# 7. Relationship Anonymity

Observer should not easily determine:

```text
Alice communicates with Bob
```

---

# 8. Session Unlinkability

Observer should not easily determine:

```text
session A
and
session B
```

belong to same user.

---

# 9. Message Unlinkability

Observer should not easily link:

```text
packet X
to
packet Y
```

as same logical message unless required.

---

# 10. Traffic Unobservability

Stronger property:

```text
observer cannot reliably distinguish whether real communication occurred
```

within the designed threat model.

---

# 11. Activity Hiding

User action should not map trivially to:

```text
external packet timing
```

---

# 12. Location Hiding

Peer should not learn:

```text
sender/recipient network address
```

where anonymity mode promises endpoint hiding.

---

# 13. Metadata Minimization

Provider should learn only the minimum required to perform its role.

---

# 14. Privacy Mode Contract

```rust
pub struct PrivacyGuaranteeProfile {
    pub sender_anonymity: GuaranteeLevel,
    pub receiver_anonymity: GuaranteeLevel,
    pub relationship_anonymity: GuaranteeLevel,
    pub unlinkability: GuaranteeLevel,
    pub traffic_unobservability: GuaranteeLevel,
}
```

---

# 15. Guarantee Level

```rust
pub enum GuaranteeLevel {
    None,
    BestEffort,
    Strong,
    Required,
}
```

---

# 16. Maximum Anonymity Contract

All mandatory properties must map to:

```text
Required
```

or explicit documented limitation.

---

# 17. Adversary Classes

SIAR should model multiple adversaries.

---

# 18. Local Network Observer

Can observe:

```text
client IP
packet timing
packet size
destinations
```

---

# 19. ISP-Level Observer

Can observe broader:

```text
client traffic
bridge/gateway endpoints
timing
volume
```

---

# 20. Global Passive Adversary

Can observe traffic entering/leaving large portions of the network.

Strongest passive adversary.

---

# 21. Malicious Gateway

Can observe:

```text
client IP
ingress timing
traffic volume
```

but should not know destination.

---

# 22. Malicious Mix Node

Can observe:

```text
incoming peer
outgoing next hop
packet timing
```

at one layer.

---

# 23. Colluding Mix Nodes

Can correlate observations across layers.

---

# 24. Malicious Mailbox Provider

Can observe:

```text
deposit timing
fetch timing
object lifetime
```

but not plaintext.

---

# 25. Malicious Directory

Can:

```text
bias topology
include attacker nodes
exclude honest nodes
serve stale state
```

---

# 26. Malicious Bridge

Can observe:

```text
client IP
connection timing
transport fingerprint
```

---

# 27. Malicious Rendezvous Provider

Can observe:

```text
bulk upload/fetch timing
object size
```

---

# 28. Malicious Recipient

Knows:

```text
application-level content received
```

and may try to identify sender network metadata.

---

# 29. Malicious Sender

Can craft traffic to probe:

```text
receiver behavior
timing
mailbox activity
```

---

# 30. Compromised Plugin

May leak:

```text
message timing
contact graph
network activity
```

---

# 31. Compromised Device

Outside full anonymity guarantee.

If endpoint is compromised, anonymity may fail.

---

# 32. Coercive/Legal Adversary

May obtain:

```text
provider logs
operator records
server disks
```

Hence data minimization matters.

---

# 33. Attack Taxonomy

---

# 34. Timing Correlation

Compare:

```text
packet enters network at T1
packet exits near T2
```

---

# 35. Volume Correlation

Compare:

```text
burst size
total bytes
packet count
```

---

# 36. Packet Size Correlation

Mitigated by normalized cells.

---

# 37. Burst Correlation

User sends several messages rapidly.

Observer matches burst elsewhere.

---

# 38. Long-Term Intersection Attack

Observe users online across repeated communication windows.

Intersect candidate sets.

---

# 39. Statistical Disclosure Attack

Use repeated observations to infer likely communication partners.

---

# 40. Website/File Fingerprinting Analogy

Large attachment transfer pattern can reveal:

```text
specific object size
chunk count
duration
```

---

# 41. Route Capture Attack

Adversary controls enough nodes to occupy:

```text
first
and
last
```

relevant observation points.

---

# 42. Sybil Attack

Adversary runs many nominally independent nodes.

Part 39 mitigates.

---

# 43. Tagging Attack

Malicious node modifies packet characteristics to recognize it downstream.

---

# 44. Delay Attack

Attacker intentionally delays packet uniquely.

---

# 45. Drop-and-Watch Attack

Attacker drops traffic and observes downstream absence.

---

# 46. Replay Attack

Replay packet and observe corresponding behavior.

---

# 47. Active Probe Attack

Probe bridge/gateway to identify censorship-resistance endpoint.

---

# 48. Enumeration Attack

Collect bridge pool.

---

# 49. Clock-Correlation Attack

Correlate local timestamps with network events.

---

# 50. Receipt-Correlation Attack

Read/delivery receipt timing leaks communication.

---

# 51. Presence Correlation

Online/typing status leaks relationship.

---

# 52. Link Preview Leak

External URL fetch reveals user IP/activity.

---

# 53. Plugin Leak

Plugin performs identifiable outbound request.

---

# 54. Push Notification Leak

Push provider learns message-arrival timing.

---

# 55. Mailbox Polling Correlation

Recipient fetch immediately after deposit.

---

# 56. Capability Correlation

Stable mailbox/fetch capability reused too long.

---

# 57. Multi-Device Correlation

Several device mailboxes activate together.

---

# 58. Recovery Correlation

Restored device reuses old anonymous identifiers.

---

# 59. Update Correlation

All clients fetch update immediately after release.

---

# 60. Censorship Fallback Correlation

Strict mode silently falls back to direct route.

Critical failure.

---

# 61. Traffic Analysis Model

Represent observable trace:

```rust
pub struct ObservableEvent {
    pub timestamp_bucket: TimeBucket,
    pub direction: ObservableDirection,
    pub size_class: ObservableSizeClass,
    pub endpoint_class: EndpointClass,
}
```

---

# 62. Privacy Trace

```rust
pub struct PrivacyTrace {
    pub events: Vec<ObservableEvent>,
}
```

No user secrets.

Used only in controlled test/simulation.

---

# 63. Attacker Observation Function

```rust
pub trait AdversaryModel {
    fn observe(
        &self,
        world: &SimulatedWorld,
    ) -> PrivacyTrace;
}
```

---

# 64. Candidate Set

For anonymity analysis:

```rust
pub struct CandidateSet<T> {
    pub candidates: Vec<T>,
}
```

---

# 65. Anonymity Set Size

Simple metric:

```text
number of plausible senders/receivers
```

Useful but insufficient alone.

---

# 66. Entropy Metric

Use Shannon entropy of candidate probabilities.

---

# 67. Effective Anonymity Set

Can derive from entropy.

---

# 68. Guessing Entropy

Estimate expected guesses required.

---

# 69. Min-Entropy

Useful against best-guess adversary.

---

# 70. Mutual Information

Can measure leakage between:

```text
true sender
and
observed trace
```

in simulation.

---

# 71. Correlation Coefficient

Can be used for timing-series analysis, but not as sole privacy metric.

---

# 72. Classification Accuracy

Train/evaluate adversary model:

```text
can observer correctly link sender→receiver?
```

---

# 73. Baseline Comparison

Always compare against:

```text
random guess
non-anonymous transport
mixnet without cover
mixnet with cover
```

---

# 74. Differential Privacy

May be useful for:

```text
aggregate telemetry
network statistics
```

Not a replacement for transport anonymity.

---

# 75. DP Telemetry

If remote aggregate telemetry exists:

```text
noise
minimum aggregation set
rate limits
```

---

# 76. Maximum Anonymity

Prefer:

```text
local-only telemetry
```

---

# 77. Formal Privacy Invariants

Examples:

```text
No raw AccountId appears in provider-facing frame.
No direct route selected in MaximumAnonymity.
No reply capability is logged.
No per-packet cross-node trace ID exists.
No mailbox secret restored to a different device.
```

---

# 78. Invariant Encoding

Where possible encode as:

```text
type system
state machine
property test
model checker assertion
```

---

# 79. Rust Type-State

Example:

```rust
struct StrictAnonymousRoute(AnonymousRoute);
```

Cannot be constructed from direct path.

---

# 80. Newtype Boundaries

Use separate types for:

```text
AccountId
MailboxId
FrameId
RouteId
CapabilitySecret
```

---

# 81. No Stringly-Typed Privacy IDs

Hard rule.

---

# 82. Compile-Time Privacy Boundaries

Sensitive types do not implement:

```text
Display
Debug
Serialize
```

unless redacted wrapper.

---

# 83. Secret Debug

Use:

```text
<redacted>
```

---

# 84. Taint-Like API Design

Provider-facing APIs accept only:

```text
privacy-approved DTOs
```

not domain objects.

---

# 85. Formal Verification Targets

Prioritize small critical state machines.

---

# 86. Route Policy State Machine

Verify:

```text
MaximumAnonymity never returns non-anonymous route
```

---

# 87. Mailbox ACK State Machine

Verify:

```text
ACK only after durable persist
```

---

# 88. Reply Capability State Machine

Verify:

```text
single-use token cannot be reused
```

---

# 89. Topology Validation

Verify:

```text
revoked node never enters route
same operator not repeated in strict route
```

---

# 90. Traffic Scheduler

Verify:

```text
real durable message never dropped as cover
```

---

# 91. Attachment Transfer

Verify:

```text
file not Complete before integrity verification
```

---

# 92. Bridge Resolver

Verify:

```text
strict mode never falls back to direct peer
```

---

# 93. Model Checking Candidates

Possible tools/approaches:

```text
TLA+
PlusCal
Alloy
Kani
Prusti / Creusot where practical
proptest state machines
loom for concurrency
```

---

# 94. TLA+ Scope

Best for:

```text
distributed state machines
topology publication
revocation
mailbox ACK
provider migration
```

---

# 95. Kani Scope

Best for:

```text
bounded Rust invariants
parser/state logic
```

---

# 96. Loom Scope

Concurrency:

```text
reply token reservation
dedup
queue state
rotation
```

---

# 97. Property-Based Testing

Extensive for:

```text
routing
framing
reassembly
policy
```

---

# 98. Formal Methods Principle

Do not attempt to formally verify the entire system initially.

Focus on privacy-critical invariants.

---

# 99. Threat Model Document

Machine-readable plus human-readable.

---

# 100. Threat ID

```rust
pub struct ThreatId(pub &'static str);
```

Examples:

```text
TA-TIMING-001
TA-SYBIL-002
TA-TAG-003
```

---

# 101. Threat Record

```rust
pub struct ThreatRecord {
    pub id: ThreatId,
    pub adversary: AdversaryClass,
    pub asset: PrivacyAsset,
    pub attack: AttackClass,
    pub mitigations: Vec<MitigationId>,
    pub residual_risk: ResidualRisk,
}
```

---

# 102. Privacy Assets

```rust
pub enum PrivacyAsset {
    SenderIdentity,
    ReceiverIdentity,
    Relationship,
    OnlineStatus,
    MessageTiming,
    MessageVolume,
    Location,
    DeviceLinkage,
}
```

---

# 103. Residual Risk

```rust
pub enum ResidualRisk {
    Low,
    Medium,
    High,
    Unknown,
}
```

---

# 104. No Fake Precision

Do not convert residual risk into arbitrary 0–100 privacy score.

---

# 105. Attack Graph

Model attack chains.

Example:

```text
bridge observes IP
+
mailbox observes timing
+
plugin leaks event
→ relationship correlation
```

---

# 106. Multi-Layer Attack Analysis

Must consider combinations, not just isolated layers.

---

# 107. Correlation Lab

Build deterministic privacy simulation environment.

---

# 108. Simulation Components

```text
clients
mix nodes
gateways
mailboxes
bridges
directory
cover traffic
message workloads
network latency
adversary taps
```

---

# 109. Workload Models

Examples:

```text
chatty user
quiet user
bursty user
daily routine
large attachment user
multi-device user
```

---

# 110. Background Traffic Models

Include:

```text
cover
loop
mailbox fetch
updates
plugins
```

---

# 111. Network Models

```text
LAN
mobile
high latency
packet loss
regional differences
```

---

# 112. Observer Placement

Simulate:

```text
local ISP
first hop
one mix layer
multiple mix layers
mailbox
global observer
```

---

# 113. Attack Algorithms

Implement research-inspired attacks in lab:

```text
timing cross-correlation
volume matching
intersection
statistical disclosure
classifier-based linkage
```

---

# 114. Research Boundary

Attack implementations are for defensive evaluation only.

---

# 115. Timing Correlation Test

Input:

```text
ingress timestamps
egress timestamps
```

Output:

```text
linkage confidence
```

---

# 116. Volume Matching Test

Compare cell counts over windows.

---

# 117. Intersection Test

Across repeated sessions:

```text
candidate set intersection
```

---

# 118. Classifier Test

Train on synthetic traces.

Evaluate on held-out traces.

---

# 119. Adversary Advantage

Metric:

```text
success probability - random baseline
```

---

# 120. Privacy Budget

Not differential-privacy budget.

Engineering concept:

```text
acceptable adversary advantage
```

for specific scenario.

---

# 121. Release Thresholds

Example policy:

```text
classifier accuracy must not exceed defined threshold in benchmark scenario
```

Exact values require research/benchmarking.

---

# 122. No Universal Threshold

Different threat models need different limits.

---

# 123. Regression Testing

Compare current build against:

```text
previous release
baseline profile
```

---

# 124. Privacy Regression

Example:

```text
timing classifier accuracy rises materially
```

Release should block pending review.

---

# 125. Privacy Performance Benchmark

Separate from normal performance benchmark.

---

# 126. Tradeoff Dashboard

Track:

```text
latency
bandwidth
battery
adversary advantage
```

---

# 127. Pareto Analysis

Identify privacy/performance tradeoffs.

---

# 128. Do Not Optimize Latency Alone

Hard rule.

---

# 129. Red Team Program

Dedicated privacy red-team exercises.

---

# 130. Red Team Goals

Try to:

```text
identify sender
identify receiver
link sessions
infer online state
fingerprint attachments
enumerate bridges
force downgrade
```

---

# 131. Red Team Test Accounts

Synthetic only.

---

# 132. No Production User Traffic

Hard rule.

---

# 133. Red Team Environment

Use:

```text
staging/testnet
```

with realistic topology.

---

# 134. Adversarial Node

Deploy controlled malicious mix node.

---

# 135. Adversarial Gateway

Controlled gateway records allowed test metadata.

---

# 136. Adversarial Mailbox

Controlled provider.

---

# 137. Active Attack Tests

```text
delay packets
drop selected packets
replay
tag malformed traffic
cause congestion
serve stale topology
```

---

# 138. Tagging Resistance Test

Modify provider-visible packet field.

Verify downstream detection/rejection.

---

# 139. Drop-and-Watch Test

Drop selected ingress packets.

Measure observable downstream change.

---

# 140. Delay Attack Test

Add distinctive delay pattern.

---

# 141. Replay Attack Test

Replay packet and inspect response.

---

# 142. Malicious Directory Test

Serve attacker-heavy topology.

Client must reject if signatures/policy invalid.

---

# 143. Valid-but-Biased Directory

Harder threat.

Requires multi-authority/auditable assignment.

---

# 144. Bridge Enumeration Test

Request many bridges.

Broker should resist pool enumeration.

---

# 145. Push Provider Test

Evaluate metadata visible to push service.

---

# 146. Plugin Leak Test

Malicious plugin attempts external correlation request.

Maximum mode must block/restrict.

---

# 147. External Fetch Test

Link preview/avatar must not leak in strict mode.

---

# 148. Backup/Recovery Test

Restore must generate fresh anonymity identifiers where required.

---

# 149. Multi-Device Correlation Test

Simulate simultaneous device activity.

---

# 150. Attachment Fingerprinting Test

Compare file sizes/padded chunks/transfer duration.

---

# 151. Censorship Fallback Test

All anonymous paths unavailable.

Result:

```text
queued
```

not direct.

---

# 152. Privacy Logging Audit

Search binaries/log schemas for sensitive types.

---

# 153. Static Logging Guard

Sensitive types should not implement default formatting.

---

# 154. Structured Logging Allowlist

Only approved fields.

---

# 155. Telemetry Schema Review

Every telemetry field classified.

---

# 156. Telemetry Privacy Classification

```rust
pub enum TelemetryPrivacyClass {
    SafeAggregate,
    SensitiveAggregate,
    LocalOnly,
    Forbidden,
}
```

---

# 157. Forbidden Telemetry

Examples:

```text
mailbox ID
frame ID
contact ID
route hops
exact peer timing
```

---

# 158. Production Tracing

No cross-node packet trace IDs.

---

# 159. Debug Builds

May enable local synthetic trace only.

---

# 160. Differential Privacy for Network Statistics

Potential for public:

```text
node count
load
latency distribution
```

if aggregation risks operator/user leakage.

---

# 161. Minimum Cohort

Do not publish small-group stats.

---

# 162. Privacy Review Workflow

Any feature touching anonymous mode requires:

```text
threat update
metadata review
attack-surface review
test update
```

---

# 163. Privacy Change Request

```rust
pub struct PrivacyImpactAssessment {
    pub feature: FeatureId,
    pub new_metadata: Vec<MetadataField>,
    pub affected_threats: Vec<ThreatId>,
    pub mitigations: Vec<MitigationId>,
}
```

---

# 164. CI Privacy Gate

Fail if:

```text
new forbidden log field
strict route invariant fails
sensitive type serialized improperly
privacy benchmark regresses beyond threshold
```

---

# 165. Release Gate Levels

```rust
pub enum PrivacyGateSeverity {
    Advisory,
    Warning,
    Blocking,
}
```

---

# 166. Blocking Examples

```text
direct fallback in Maximum mode
mailbox secret in log
route hop in telemetry
reply token reused
revoked node selected
```

---

# 167. Warning Examples

```text
minor bandwidth-pattern regression
small increase in classifier confidence
```

subject to review.

---

# 168. Privacy Evidence Bundle

Release artifact should include:

```text
threat-model version
invariant test results
simulation summary
red-team status
privacy benchmark comparison
known residual risks
```

---

# 169. Threat Model Version

```rust
pub struct ThreatModelVersion(pub u64);
```

---

# 170. Release Metadata

Record:

```text
software version
threat model version
governance policy version
protocol version
```

---

# 171. Known Limitations

Must be documented.

Examples:

```text
endpoint compromise
global long-term intersection risk
large-transfer volume leakage
bridge sees client IP
```

---

# 172. User Communication

Do not expose academic detail by default.

---

# 173. Maximum Anonymity UX

Explain:

```text
Designed to reduce sender/receiver linkage and traffic metadata.
It cannot protect a compromised device or guarantee perfect untraceability.
```

---

# 174. No Absolute Claims

Forbidden marketing:

```text
100% anonymous
untraceable
impossible to identify
```

---

# 175. Privacy Levels

Could map:

```text
Standard
Private
Anonymous
Maximum Anonymity
```

to documented guarantee profile.

---

# 176. Formal Spec Repository

Recommended:

```text
spec/privacy/
```

---

# 177. Contents

```text
threat-model.md
privacy-invariants.md
tla/
kani/
simulation/
red-team/
release-gates/
```

---

# 178. TLA+ Modules

Potential:

```text
MailboxAck.tla
RoutePolicy.tla
Revocation.tla
ProviderMigration.tla
```

---

# 179. Rust Verification Modules

```text
route_policy_kani.rs
reply_capability_kani.rs
reassembly_kani.rs
```

---

# 180. Concurrency Models

Use loom for:

```text
dedup
reply reservation
mailbox rotation
provider failover
```

---

# 181. Privacy Testkit

Create:

```text
siar-privacy-testkit
```

---

# 182. Testkit Features

```text
synthetic users
adversary taps
trace generation
attack evaluators
privacy metrics
fault injection
```

---

# 183. Simulator Determinism

Seeded.

---

# 184. Statistical Repetition

Run multiple seeds.

---

# 185. Confidence Intervals

Privacy benchmark reports should include uncertainty.

---

# 186. Avoid Cherry-Picked Seed

Hard rule.

---

# 187. Benchmark Dataset

Synthetic workloads only.

---

# 188. Long-Term Simulation

Days/weeks of simulated activity.

---

# 189. Intersection Attack Requires Long Horizon

Include.

---

# 190. Network Churn

Simulate:

```text
nodes joining/leaving
route rotation
bridge blocking
```

---

# 191. Adversary Adaptation

Future simulator can adapt attacks based on observations.

---

# 192. Machine Learning Adversary

Useful defensive benchmark.

---

# 193. ML Privacy Risk

Model itself may overfit synthetic traces.

Use held-out workloads/topologies.

---

# 194. Multiple Attack Families

Never rely on one classifier.

---

# 195. Attack Success Matrix

Report by:

```text
threat model
privacy mode
workload
network condition
```

---

# 196. Privacy Regression Baseline

Keep stable reference release.

---

# 197. Architecture Change Review

If Part 37 scheduler changes:

```text
rerun timing attack suite
```

---

# 198. If Part 40 chunk size changes

Rerun:

```text
attachment fingerprint suite
```

---

# 199. If Part 41 transport changes

Rerun:

```text
DPI/fingerprint/probe suite
```

---

# 200. If Part 39 topology changes

Rerun:

```text
route capture/Sybil suite
```

---

# 201. Privacy SLOs

Examples:

```text
zero forbidden identifier leakage
zero silent downgrade
bounded route compromise probability under modeled attacker
privacy benchmark non-regression
```

---

# 202. Privacy SLO Caution

Do not promise mathematically absolute anonymity without proof.

---

# 203. Residual Risk Registry

Maintain:

```text
open privacy issues
accepted risk
mitigation roadmap
```

---

# 204. Risk Acceptance

High residual risk requires explicit security/privacy review.

---

# 205. Privacy Bug Severity

```rust
pub enum PrivacyBugSeverity {
    Low,
    Moderate,
    High,
    Critical,
}
```

---

# 206. Critical Privacy Bugs

Examples:

```text
direct fallback
identity leak
route logging
mailbox secret exposure
```

---

# 207. Disclosure Process

Security/privacy vulnerability reporting channel.

---

# 208. Reproduction

Use synthetic environment.

---

# 209. Hotfix

Critical privacy bug may disable affected mode/feature.

---

# 210. Safe Failure

If anonymity guarantee uncertain:

```text
disable strict mode action
```

rather than silently weaken.

---

# 211. Formal Threat Matrix

Example dimensions:

```text
adversary
observation point
capability
target property
mitigation
test
residual risk
```

---

# 212. Example — Timing Correlation

```text
Adversary:
    global passive

Target:
    relationship anonymity

Mitigations:
    fixed cells
    cover
    randomized delays
    batching

Tests:
    timing correlation benchmark
```

---

# 213. Example — Sybil Route Capture

```text
Adversary:
    malicious operator

Target:
    route unlinkability

Mitigations:
    operator identity
    admission
    concentration caps
    route diversity

Tests:
    Sybil simulation
```

---

# 214. Example — Bridge Enumeration

```text
Adversary:
    censor

Target:
    reachability

Mitigations:
    limited bridge allocation
    rotating descriptors
    access tokens

Tests:
    enumeration simulation
```

---

# 215. Example — Plugin Leak

```text
Adversary:
    compromised plugin

Target:
    sender identity

Mitigations:
    Maximum-mode network restriction
    capability sandbox

Tests:
    plugin exfiltration test
```

---

# 216. Privacy Boundary Diagram

```text
User Identity
    │
    │ E2EE-only binding
    ▼
Anonymous Application Identity
    │
    ▼
Mailbox / Reply Capability
    │
    ▼
Frame / Cell IDs
    │
    ▼
Mix Route
```

No layer should expose upward identity unnecessarily.

---

# 217. Linkability Budget

Each layer should document stable identifiers and lifetimes.

---

# 218. Identifier Lifetime Classes

```rust
pub enum IdentifierLifetime {
    PerPacket,
    PerMessage,
    PerSession,
    ShortEpoch,
    DeviceLifetime,
}
```

---

# 219. Strict Mode Goal

Prefer shortest feasible lifetime.

---

# 220. Stable Identifier Review

Any identifier lasting:

```text
DeviceLifetime
```

requires privacy review.

---

# 221. Recovery Freshness

Recovery creates fresh anonymity identities.

---

# 222. Provider Migration

Should avoid direct old→new stable mapping visible externally.

---

# 223. Dual Receive Window

Bounded.

---

# 224. Cross-Provider Correlation

Privacy lab should test.

---

# 225. Multi-Provider Redundancy

Can increase observable surface.

Requires privacy benchmark.

---

# 226. Directory Privacy

Directory must not receive user route queries if client can build routes locally.

---

# 227. Local Route Construction

Required.

---

# 228. Node Health Fetch

Same for all clients or broad snapshot.

Avoid per-route query.

---

# 229. Bridge Broker Privacy

Potential leak remains.

Track as residual risk until anonymized broker access.

---

# 230. Push Provider Residual Risk

Push timing leak documented.

Maximum mode may disable push.

---

# 231. Android OS Telemetry

Outside SIAR full control.

Document limitation.

---

# 232. Desktop OS Telemetry

Likewise.

---

# 233. Crash Reporting

Maximum mode local-only unless explicit opt-in.

---

# 234. Support Bundle

Redaction test is release blocker.

---

# 235. Data Retention

Privacy test logs retained only in test environment.

---

# 236. No Production Packet Capture

Hard rule.

---

# 237. Operator Debugging

Use synthetic probes, not user traffic capture.

---

# 238. Incident Investigation

If packet-level capture is unavoidable in controlled environment:

```text
testnet only
```

---

# 239. External Audit

Recommended before declaring production Maximum Anonymity.

---

# 240. Audit Scope

```text
protocol
routing
cover traffic
directory
bridge
mailbox
bulk transfer
logging
```

---

# 241. Cryptographic Audit

Separate specialist review.

---

# 242. Privacy/Traffic Analysis Audit

Separate expertise.

---

# 243. Red Team Cadence

Before major release and periodically.

---

# 244. Threat Model Update Cadence

At least every major architecture change.

---

# 245. Research Tracking

Maintain watchlist for:

```text
mixnet attacks
traffic analysis research
Sphinx issues
censorship fingerprinting
anonymity metrics
```

---

# 246. Architecture Decision Records

Every privacy-relevant tradeoff gets ADR.

---

# 247. ADR Example

```text
Why fixed route length?
Why one standard cell size?
Why push disabled in Maximum mode?
```

---

# 248. Release Checklist

Privacy section:

```text
threat model updated?
invariants pass?
no forbidden logs?
privacy benchmark stable?
known risks reviewed?
```

---

# 249. CI Layout

```text
Fast:
    invariants
    property tests
    static logging checks

Nightly:
    privacy simulation
    attack classifiers
    long-horizon intersection

Release:
    full privacy suite
    red-team signoff
```

---

# 250. Privacy Benchmark Artifacts

Store:

```text
configuration
seed set
results
plots
confidence intervals
```

---

# 251. Reproducibility

Another engineer should reproduce benchmark.

---

# 252. No Secret Benchmark Tuning

Hard rule.

---

# 253. Security Invariants

Mandatory:

```text
1. Maximum Anonymity never silently uses a non-anonymous path.
2. Provider-facing identifiers never directly expose AccountId/DeviceId/ConversationId.
3. Sensitive anonymity secrets never appear in logs, telemetry, crash reports, or support bundles.
4. Route construction remains client-side from signed topology.
5. Revoked nodes cannot appear in strict routes.
6. Mix nodes do not receive application identity state.
7. Traffic shaping decouples strict-mode network emission from user action.
8. Mailbox ACK occurs only after durable application persistence.
9. Reply capabilities are bounded, expiring, and non-reusable when single-use.
10. Attachment completion requires cryptographic verification.
11. Bridge failure cannot cause direct-peer fallback.
12. Privacy regressions are release-blocking when they violate required guarantees.
```

---

# 254. Recommended Crate Layout

```text
crates/
├── siar-privacy-core/
├── siar-privacy-threat-model/
├── siar-privacy-invariants/
├── siar-privacy-sim/
├── siar-privacy-attacks/
├── siar-privacy-metrics/
├── siar-privacy-formal/
├── siar-privacy-redteam/
├── siar-privacy-telemetry/
└── siar-privacy-testkit/
```

---

# 255. `siar-privacy-core`

Owns:

```text
guarantee levels
privacy assets
residual risk
privacy gate types
```

---

# 256. `siar-privacy-threat-model`

Threat records/matrix.

---

# 257. `siar-privacy-invariants`

Executable invariant checks.

---

# 258. `siar-privacy-sim`

Network/workload simulation.

---

# 259. `siar-privacy-attacks`

Defensive attack evaluators.

---

# 260. `siar-privacy-metrics`

Entropy/linkage/correlation metrics.

---

# 261. `siar-privacy-formal`

TLA+/Kani/loom integration.

---

# 262. `siar-privacy-redteam`

Scenario orchestration.

---

# 263. `siar-privacy-telemetry`

Telemetry schema/privacy classification.

---

# 264. `siar-privacy-testkit`

Synthetic workloads/adversaries.

---

# 265. Initial Production Scope

Implement first:

```text
versioned threat model
privacy guarantee profile
critical privacy invariants
synthetic trace simulator
timing/volume/intersection attack benchmarks
privacy-safe telemetry schema classification
static logging guards
route/mailbox/reply/downgrade formal state checks
CI privacy gates
residual-risk registry
release privacy evidence bundle
```

Then add:

```text
ML-based traffic classifiers
multi-authority adversary simulation
cross-provider correlation models
differentially private public network metrics
deeper formal proofs
external audit automation
```

---

# 266. Definition of Done

Part 42 is complete when:

- sender, receiver, relationship, unlinkability, and traffic-unobservability properties are explicit
- adversary classes from local observer to global passive and colluding infrastructure are defined
- timing, volume, intersection, disclosure, tagging, replay, delay, bridge, plugin, push, mailbox, and multi-device attacks are covered
- privacy guarantee profiles map product modes to required properties
- privacy invariants are executable where possible
- small critical state machines are targeted for TLA+/Kani/loom/property verification
- deterministic privacy simulation and adversary traces are defined
- correlation/intersection/classifier benchmarks compare against baselines
- privacy regressions are tracked across releases
- red-team methodology covers gateways, mix nodes, mailboxes, bridges, directory, plugins, and recovery
- telemetry/logging schema has explicit privacy classification
- forbidden identifiers and secrets are blocked from logs/support bundles
- release evidence includes threat-model version, invariant results, privacy benchmarks, and residual risks
- critical privacy regressions block release
- user-facing anonymity claims remain accurate and non-absolute

---

# 267. Final Architecture

```text
                    SIAR ANONYMITY SYSTEM
                             │
                             ▼
                       THREAT MODEL
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
       Passive             Active           Colluding
       Observer            Attacker         Infrastructure
          │                  │                  │
          └──────────────────┼──────────────────┘
                             ▼
                    PRIVACY INVARIANTS
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
       Simulation        Formal Methods       Red Team
          │                  │                  │
          └──────────────────┼──────────────────┘
                             ▼
                     PRIVACY EVIDENCE
                             │
                             ▼
                    RELEASE PRIVACY GATE
```

---

# 268. Final Principle

Anonymity is not a feature checkbox.

The correct model is:

```text
explicit threat model
+
explicit privacy properties
+
architecture invariants
+
attack simulation
+
formal verification where feasible
+
red-team testing
+
privacy regression gates
```

not:

```text
we use a mixnet
therefore we are anonymous
```

This architecture gives SIAR a disciplined way to measure, test, review, and continuously improve its anonymity claims across the complete high-anonymity subsystem built in Parts 34–41.
