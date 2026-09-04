# Part 11 — Relay & Self-Hosted Infrastructure Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 11 of 24  
**Primary language:** Rust  
**Primary networking foundation:** Iroh-first, infrastructure-neutral above the adapter boundary  
**Primary goals:** reliable NAT fallback, self-hosting, regional resilience, private deployments, relay isolation, gateway bridging, scalable operations, graceful degradation, observability, abuse resistance, and production-ready Internet infrastructure

---

# 1. Purpose

A peer-to-peer communication platform should attempt direct communication wherever practical, but production software cannot assume every pair of devices can establish a direct path.

Real-world environments include:

```text
carrier-grade NAT
enterprise firewalls
symmetric NAT
restricted UDP
mobile carrier networks
hotel/campus Wi-Fi
double NAT
corporate proxies
unstable Internet
regional outages
```

Therefore the platform requires a reliable relay layer.

The relay layer must support:

```text
public/community relay use
dedicated relay use
self-hosted relays
organization-private relays
regional relay fleets
multi-cloud deployment
DTN gateway bridging
controlled relay-only policies
```

while preserving:

```text
end-to-end application security
transport independence
self-hosting
no product-level relay lock-in
```

---

# 2. Current Iroh Grounding

The platform should use Iroh's relay functionality as its primary Internet relay implementation.

Current Iroh architecture provides:

```text
direct peer connection where possible
NAT traversal
relay fallback
configurable relay sets
self-hosted relay deployment
dedicated relay infrastructure
```

The communication platform should still expose its own relay abstraction.

Correct layering:

```text
Application
   ↓
Communication Runtime
   ↓
Routing Policy
   ↓
Relay Abstraction
   ↓
Iroh Relay Adapter
   ↓
iroh-relay
```

Not:

```text
Application business logic
   ↓
iroh-relay internals
```

---

# 3. Fundamental Principle

A relay is a:

```text
connectivity/data-path helper
```

not:

```text
message database
account authority
identity source of truth
plaintext server
product business backend
```

The relay should remain as ignorant of application content as practical.

For private messaging:

```text
Peer A
  ↓ encrypted transport/application traffic
Relay
  ↓ encrypted transport/application traffic
Peer B
```

The relay should not need message plaintext.

---

# 4. Infrastructure Layers

Separate four infrastructure responsibilities:

```text
Relay Plane
Discovery / Rendezvous Plane
Bootstrap / Configuration Plane
Operational Control Plane
```

These may be deployed together initially but must remain architecturally separate.

---

# 5. Relay Plane

Responsibilities:

```text
maintain endpoint reachability
forward traffic when direct path unavailable
handle connection multiplexing
enforce relay quotas
expose health metrics
support TLS/network ingress
```

The relay plane must not own:

```text
contacts
message history
files in plaintext
group membership semantics
ERP records
```

---

# 6. Discovery / Rendezvous Plane

Responsibilities:

```text
help peers discover current contact information
publish signed endpoint information
provide relay hints
support address lookup
```

Discovery is not the same as relay.

A peer may discover another device through:

```text
directory
DNS-like discovery
QR
LAN
Bluetooth
own-device sync
```

and still use a relay for transport.

---

# 7. Bootstrap Plane

New client needs:

```text
initial relay set
protocol configuration
trusted service identifiers
minimum version policy
```

Bootstrap configuration can ship inside the application and later update through signed configuration.

---

# 8. Operational Control Plane

Operators need:

```text
relay registry
deployment state
health
metrics
configuration rollout
certificate status
quota policy
version status
incident controls
```

This control plane is administrative infrastructure.

It should not become the P2P application's source of user truth.

---

# 9. Deployment Models

Support multiple modes.

## Mode A — Community/Public Relays

Useful for:

```text
development
small deployments
fallback
bootstrap
```

## Mode B — Dedicated Managed Relays

Useful for:

```text
production
known capacity
regional control
SLA
```

## Mode C — Self-Hosted Relay

Useful for:

```text
privacy-sensitive deployments
enterprise
schools
government
community networks
cost control
```

## Mode D — Hybrid

```text
self-hosted preferred
+
public fallback
```

This is a strong general-purpose architecture.

---

# 10. Relay Provider Abstraction

```rust
pub trait RelayProvider {
    async fn relay_candidates(
        &self,
        ctx: &RelaySelectionContext,
    ) -> Result<Vec<RelayCandidate>, RelayError>;
}
```

Possible implementations:

```text
StaticRelayProvider
IrohDefaultRelayProvider
SelfHostedRelayProvider
OrganizationRelayProvider
HybridRelayProvider
```

---

# 11. Relay Candidate

```rust
pub struct RelayCandidate {
    pub relay_id: RelayId,
    pub endpoint: RelayEndpoint,
    pub region: Option<RegionId>,
    pub provider: RelayProviderId,
    pub health: RelayHealth,
    pub policy: RelayPolicyMetadata,
}
```

Do not leak raw provider-specific structs upward.

---

# 12. Relay Identity

Each managed relay should have a stable administrative identity:

```rust
pub struct RelayId([u8; 16]);
```

Separate:

```text
RelayId
```

from:

```text
DNS hostname
IP address
deployment instance
```

A relay can change IP/instance while preserving logical identity.

---

# 13. Relay Endpoint

```rust
pub struct RelayEndpoint {
    pub url: RelayUrl,
    pub transport_security: RelayTlsPolicy,
}
```

Iroh adapter converts this into its current relay configuration type.

---

# 14. Relay Selection

Relay selection should consider:

```text
reachability
latency
health
region
load
operator policy
privacy
cost
tenant isolation
```

Iroh may make its own home-relay decisions internally; the platform's infrastructure layer still determines which relay set is eligible.

---

# 15. Home Relay Concept

A device should normally maintain a stable preferred relay relationship for reachability.

Architecture:

```text
Configured relay set
       ↓
probe
       ↓
preferred/home relay
       ↓
fallback relays
```

Do not switch continuously for tiny latency improvements.

Use hysteresis.

---

# 16. Relay Hysteresis

```rust
pub struct RelaySwitchPolicy {
    pub minimum_hold_time: Duration,
    pub latency_improvement_threshold: Duration,
    pub failure_override: bool,
}
```

This prevents relay thrashing.

---

# 17. Relay Health

```rust
pub enum RelayHealth {
    Healthy,
    Degraded,
    Unreachable,
    Maintenance,
    Unknown,
}
```

Health inputs:

```text
connectivity
handshake success
latency
packet/stream failures
capacity
operator status
```

---

# 18. Multi-Relay Resilience

Never operate production with exactly one unavoidable relay.

Recommended:

```text
Region A Relay 1
Region A Relay 2
Region B Relay 1
Region B Relay 2
```

Clients receive a set.

Direct paths still remain preferred where policy permits.

---

# 19. Failure Domains

Do not deploy all relays behind:

```text
same VM
same availability zone
same provider
same DNS failure domain
```

Production architecture should support:

```text
multi-AZ
multi-region
optionally multi-cloud
```

---

# 20. Regional Architecture

Example:

```text
India
├── Delhi relay
├── Mumbai relay
└── Bengaluru relay

Europe
├── Frankfurt relay
└── London relay

Asia-Pacific
└── Singapore relay
```

Client chooses eligible low-latency healthy relay.

---

# 21. Region Selection

Inputs:

```text
latency probe
current network
configured region preference
organization policy
data residency policy
```

Do not rely solely on IP geolocation.

---

# 22. Data Residency

Enterprise deployments may require:

```text
relay traffic stays within selected region/provider
```

Expose:

```rust
pub enum RelayRegionPolicy {
    Any,
    Preferred(Vec<RegionId>),
    Restricted(Vec<RegionId>),
}
```

---

# 23. Tenant Isolation

Production SaaS may need:

```text
shared relay fleet
dedicated tenant relay
```

Support both.

Dedicated relays provide stronger:

```text
capacity isolation
operational isolation
version control
```

---

# 24. Relay Pools

Logical grouping:

```text
public-fallback
consumer-production
enterprise-eu
enterprise-in
emergency
development
```

A client policy selects one or more pools.

---

# 25. Relay Configuration

```rust
pub struct RelayConfig {
    pub pools: Vec<RelayPoolConfig>,
    pub selection: RelaySelectionPolicy,
    pub fallback: RelayFallbackPolicy,
    pub health: RelayHealthPolicy,
}
```

---

# 26. Signed Infrastructure Configuration

Clients may receive updated relay sets.

The configuration should be:

```text
versioned
signed
expiry-bound
rollback-protected
```

Do not trust arbitrary relay URLs delivered by an untrusted server.

---

# 27. Infrastructure Config Envelope

```rust
pub struct InfrastructureConfig {
    pub version: u64,
    pub valid_from: Timestamp,
    pub valid_until: Timestamp,
    pub relay_pools: Vec<RelayPoolConfig>,
    pub bootstrap: BootstrapPolicy,
    pub signature: Signature,
}
```

---

# 28. Rollback Protection

If client has trusted:

```text
infra config v42
```

it should not silently accept:

```text
v37
```

unless recovery policy explicitly allows it.

---

# 29. Offline Bootstrap

If config service unavailable:

```text
use last-known-valid signed relay set
```

or bundled defaults.

The app must not become unusable merely because configuration control plane is offline.

---

# 30. Bootstrap Expiry

Expired config may enter:

```text
grace mode
```

rather than immediately disabling all networking.

Security-critical revocations may require stricter handling.

---

# 31. Self-Hosted Relay Deployment

A self-hosted installation should support:

```text
single-node
HA pair
regional fleet
Kubernetes
systemd VM
container
bare metal
```

Do not require Kubernetes.

---

# 32. Recommended Small Deployment

```text
DNS
 ↓
Relay VM
├── iroh-relay
├── metrics exporter
└── system supervisor
```

This is enough for many organizations.

---

# 33. Recommended HA Deployment

```text
relay1.example
relay2.example
relay3.example
```

Clients know all three.

Prefer client-side selection/failover rather than hiding every relay behind generic TCP load balancing that may interfere with protocol behavior.

---

# 34. TLS

Self-hosted relay ingress must use secure TLS as required by current Iroh deployment architecture.

Certificate management should support:

```text
ACME
operator-provided certificates
rotation
expiry monitoring
```

---

# 35. Certificate Monitoring

Alert before:

```text
certificate expiry
ACME renewal failure
DNS misconfiguration
```

This is operationally critical.

---

# 36. DNS

Use stable hostnames.

Avoid hard-coding relay IPs into clients.

DNS gives:

```text
instance replacement
provider migration
address rotation
```

but signed infrastructure config determines which names are trusted.

---

# 37. Public IP Requirement

Internet relays normally require publicly reachable infrastructure.

Self-hosted home-lab deployments must ensure:

```text
public reachability
port/firewall configuration
DNS
TLS
```

A relay behind restrictive NAT defeats the purpose.

---

# 38. Relay Firewall Policy

Expose only required ports/services.

Management interfaces should be:

```text
private network
VPN
admin authentication
```

not public unless explicitly secured.

---

# 39. Relay Statelessness Goal

Keep relay application state minimal.

Ideal relay stores:

```text
active connection state
temporary routing state
metrics
rate-limit counters
```

not user application history.

This improves:

```text
horizontal scaling
replacement
privacy
disaster recovery
```

---

# 40. Persistent Relay State

If operational state requires persistence:

```text
configuration
tenant policy
rate-limit policy
audit events
```

keep it separate from user message content.

---

# 41. E2EE Boundary

Even when transport traffic passes through relay:

```text
application payload remains end-to-end protected
```

The relay must never be considered a trusted plaintext middlebox.

---

# 42. Metadata Reality

Relay operators may still observe some metadata:

```text
connection timing
traffic volume
relay client addresses
```

Do not claim relay use provides metadata anonymity.

Privacy documentation must be explicit.

---

# 43. Relay-Only Privacy Mode

The platform may later support a policy that prefers/forces relay-only paths where underlying Iroh support permits.

Possible reasons:

```text
hide direct peer IP from remote peer
enterprise network policy
diagnostics
```

Trade-offs:

```text
higher latency
higher bandwidth cost
relay metadata exposure
```

---

# 44. Direct-Preferred Default

General default:

```text
direct if healthy
relay fallback
```

This minimizes:

```text
latency
relay cost
central dependency
```

---

# 45. Relay-Preferred Policy

Some deployments may choose:

```text
relay preferred
```

for operational predictability.

Expose policy, do not hard-code.

---

# 46. Relay Fallback Chain

Example:

```text
Direct
 ↓ failure
Private relay
 ↓ failure
Managed dedicated relay
 ↓ failure
Public fallback
```

Policy can restrict any layer.

---

# 47. Private-Only Deployment

Enterprise mode:

```text
Direct
+
organization relays
```

and:

```text
no public relays
```

This should be fully supported.

---

# 48. Air-Gapped / Local Mode

If no Internet:

```text
relay infrastructure irrelevant
```

Routing falls back to:

```text
LAN
Wi-Fi
Bluetooth
DTN
```

The core product remains functional.

---

# 49. Relay Capacity Planning

Capacity dimensions:

```text
concurrent connections
bandwidth
network packets
CPU
memory
file descriptors
egress cost
```

Do not size only by registered user count.

---

# 50. Capacity Formula

Conceptual estimates:

```text
concurrent endpoints
×
average relay-use fraction
×
average relayed throughput
```

plus peak factors.

Measure production traffic rather than relying solely on estimates.

---

# 51. Direct Connection Ratio

Important operational metric:

```text
direct connection percentage
relay fallback percentage
```

High unexpected relay usage may indicate:

```text
NAT traversal issue
network policy
regional degradation
client regression
```

---

# 52. Relay Bandwidth Cost

Relay traffic creates infrastructure egress.

Large files/video can dominate cost.

Routing policy should prefer direct paths when safe and possible.

---

# 53. File Transfer Policy

For large files:

```text
direct/LAN preferred
relay allowed
```

Optional application policy:

```text
ask before > X GB over metered/relay path
```

---

# 54. Call Traffic

Realtime audio/video over relay needs:

```text
low latency
sufficient bandwidth
capacity isolation
```

Relay fleet should monitor realtime quality separately from bulk traffic.

---

# 55. Traffic Classes

Relay infrastructure may classify at connection/session level:

```text
control
interactive
realtime
bulk
```

without needing application plaintext.

Enforcement must not compromise E2EE.

---

# 56. QoS

Possible:

```text
per-tenant rate limits
connection class limits
bulk shaping
```

Do not create protocol-level starvation of critical control.

---

# 57. Resource Limits

Part 08 applies server-side too.

Relay hard limits:

```text
connections
streams
memory
FDs
bandwidth
handshakes/sec
unknown-client rate
```

---

# 58. Admission Control

New relay connection passes:

```text
global capacity
tenant policy
authentication/API policy if used
rate limit
```

before expensive resources.

---

# 59. Authentication to Relay

Different deployments may use:

```text
open relay access
project API keys
tenant credentials
network ACL
```

Relay-access authorization is separate from P2P identity.

---

# 60. Relay Access Credential

Do not reuse:

```text
user E2EE private key
```

as relay billing/access credential.

Separate concerns.

---

# 61. Credential Rotation

Relay access credentials should support:

```text
rotation
revocation
overlap window
```

without changing account identity.

---

# 62. Rate Limiting

Protect against:

```text
connection floods
bandwidth abuse
handshake CPU abuse
tenant runaway traffic
```

Use:

```text
token buckets
connection caps
per-source limits
```

---

# 63. DDoS Reality

A public relay is exposed infrastructure.

Architecture must expect:

```text
volumetric attacks
connection attacks
protocol abuse
```

Application-level Rust safety alone cannot solve network DDoS.

Use provider/network protections where needed.

---

# 64. Front-Door Infrastructure

Depending on protocol constraints and provider support:

```text
DDoS-protected network edge
firewall
provider anti-DDoS
```

may sit before relay.

Do not assume generic HTTP CDN proxying is compatible.

---

# 65. Relay Abuse Monitoring

Track:

```text
connections/sec
failed handshake
bandwidth per tenant
abnormal duration
protocol parse errors
```

---

# 66. Privacy-Preserving Metrics

Aggregate by:

```text
relay
region
tenant
traffic class
```

Avoid exporting raw peer identities unless needed for security/admin and properly protected.

---

# 67. Structured Logging

Fields:

```text
relay_id
region
connection_id
protocol result
latency
bytes
error code
```

Do not log payload bytes.

---

# 68. Log Retention

Relay logs can reveal metadata.

Use:

```text
short retention by default
access control
redaction
```

Longer security retention only when justified.

---

# 69. Health Endpoints

Expose:

```text
liveness
readiness
metrics
version
```

Administrative endpoints should not reveal sensitive configuration.

---

# 70. Liveness

Answers:

```text
process alive?
```

---

# 71. Readiness

Answers:

```text
can accept production relay traffic?
```

A relay may be alive but not ready because:

```text
certificate missing
config invalid
capacity exhausted
```

---

# 72. Synthetic Probes

Run external probes:

```text
connect
relay test traffic
measure latency
```

from multiple regions.

This detects failures beyond process-level health.

---

# 73. Relay Doctor Tool

Provide operator command:

```text
comm-relay doctor
```

Checks:

```text
DNS
TLS
ports
Iroh relay handshake
latency
configuration
system limits
```

---

# 74. Version Reporting

Every relay exposes administrative:

```text
software version
protocol version
build ID
```

for fleet control.

Do not reveal excessive internals publicly.

---

# 75. Upgrade Policy

Use:

```text
canary
blue/green
rolling
```

upgrades.

Never upgrade all global relays simultaneously.

---

# 76. Canary Relay

Example:

```text
5% eligible clients
```

or internal test fleet first.

Watch:

```text
connection failures
latency
direct/relay ratio
CPU
memory
```

---

# 77. Version Locking

Enterprise/self-hosted deployments may intentionally stay on tested relay versions.

Infrastructure config should record compatibility constraints.

---

# 78. Client/Relay Compatibility

Clients should support documented compatible relay protocol versions.

Do not tie relay upgrade to application UI release unnecessarily.

---

# 79. Rollback

Keep previous known-good relay deployment artifact available.

Rollback should not require changing peer identity/application data.

---

# 80. Graceful Draining

Before relay shutdown:

```text
mark not-ready
stop new sessions
allow existing sessions to drain
```

Then terminate after timeout.

---

# 81. Forced Shutdown

If emergency:

```text
clients fail over
```

The platform must tolerate abrupt relay loss.

---

# 82. Client Failover

Relay disappears:

```text
existing path fails
 ↓
routing reevaluates
 ↓
direct or alternate relay selected
```

No message/file semantic state is lost.

---

# 83. Relay Failure During File Transfer

Part 05 resumes missing chunks on new path.

Do not restart entire file.

---

# 84. Relay Failure During Call

Media path may:

```text
switch relay/path
adapt temporarily
```

If transition impossible:

```text
reconnect call
```

No protocol should assume relay immutability.

---

# 85. DNS Failure

Clients should retain:

```text
resolved addresses for reasonable TTL
known relay set
```

but obey certificate/security validation.

Signed config + DNS resiliency complement each other.

---

# 86. Control Plane Failure

If admin/control service is down:

```text
relay data plane should continue
```

where possible.

Do not require live central database lookup for every packet.

---

# 87. Database Independence

Relay data plane should avoid mandatory Postgres dependency for basic forwarding.

This improves:

```text
availability
latency
operational simplicity
```

---

# 88. Optional Control Database

For hosted SaaS operations:

```text
tenants
API credentials
relay registry
quota config
audit metadata
```

may live in Postgres/Neon.

But relay forwarding should cache required policy locally.

---

# 89. Control Data Cache

Relay can receive signed/versioned policy snapshot.

If database temporarily unavailable:

```text
continue with last-valid policy
```

within expiry/grace.

---

# 90. Neon/Postgres Role

If your wider platform already uses Neon/Postgres, use it for:

```text
operator/admin metadata
tenant configuration
billing counters
relay fleet registry
```

not the per-packet relay hot path.

---

# 91. Metrics Storage

Use metrics backend suitable for time series.

Keep this separate from application Postgres if scale demands.

Architecture does not mandate a specific vendor.

---

# 92. Object Storage

Relay should not use object storage for ordinary live forwarding.

DTN/store-forward gateways may use dedicated encrypted blob storage under Part 06/05 architecture.

---

# 93. Relay vs DTN Gateway

Important distinction:

## Relay

```text
continuous-ish network path helper
```

## DTN Gateway

```text
durably stores bundles across time
```

Do not silently make all relays DTN stores.

---

# 94. Combined Relay + DTN Gateway

A server may run both:

```text
iroh relay
+
DTN gateway service
```

but keep separate resource quotas and security boundaries.

---

# 95. Gateway Architecture

```text
Local DTN
 ↓
Gateway Node
 ├── comm-dtn
 ├── comm-routing
 ├── Iroh endpoint
 └── optional iroh-relay nearby/colocated
 ↓
Internet
```

Gateway can deliver bundles to remote peers.

---

# 96. Disaster Deployment

Example:

```text
Shelter Raspberry Pi
├── Wi-Fi AP
├── BLE
├── DTN store
└── intermittent satellite/4G Internet

When Internet available:
    DTN → Iroh → remote destination
```

No central cloud is needed during partition.

---

# 97. Community Relay

A community organization can host:

```text
one or more relays
+
optional DTN gateways
```

for local users.

This supports local sovereignty.

---

# 98. Enterprise Relay

Enterprise can require:

```text
organization relays only
specific regions
private DNS
VPN-admin access
```

Application code remains unchanged.

---

# 99. School/ERP Deployment

A school could:

```text
use public/dedicated Internet relays normally
```

while local-campus devices may communicate directly/LAN.

The relay layer does not need ERP domain knowledge.

---

# 100. Relay Discovery

Relay URLs may come from:

```text
bundled configuration
signed config update
organization provisioning
manual advanced config
```

Not from arbitrary remote message payload.

---

# 101. Manual Self-Hosted Relay

Advanced UI/CLI may allow:

```text
Add relay URL
```

but should show:

```text
unverified custom infrastructure
```

until trust policy validates it.

---

# 102. Provisioning Profiles

Enterprise installer can provide signed:

```text
organization infrastructure profile
```

containing relay pools and policies.

---

# 103. Relay Policy Object

```rust
pub struct RelayPolicy {
    pub allowed_pools: Vec<RelayPoolId>,
    pub allow_public_fallback: bool,
    pub preferred_regions: Vec<RegionId>,
    pub relay_only: bool,
}
```

---

# 104. Routing Integration

Part 03 sees relay paths as candidates:

```text
IrohDirect
IrohRelay(relay_id)
```

It compares:

```text
latency
health
policy
cost
```

---

# 105. Capability Integration

Part 07 can expose:

```text
relay mode support
infrastructure protocol version
```

but detailed relay fleet info need not be advertised peer-to-peer.

---

# 106. Resource Integration

Part 08 server profile controls:

```text
connections
streams
bandwidth
memory
FDs
tenant quota
```

---

# 107. Crash Recovery Integration

Part 09 relay process can restart.

Live connections disappear; clients reconnect/fail over.

Relay persistent control metadata recovers separately.

P2P application semantic state remains at endpoints.

---

# 108. Test Integration

Part 10 should include:

```text
relay failure
relay switch
TLS failure
DNS failure
multi-relay selection
version mismatch
rate-limit behavior
```

---

# 109. Local Test Fleet

Development:

```text
relay-a
relay-b
client-a
client-b
```

run via local processes/containers.

Inject:

```text
packet loss
relay kill
DNS switch
```

---

# 110. Relay Protocol Fuzzing

Fuzz relay protocol parser/adapters.

Current upstream Iroh test coverage should be complemented by your platform's adapter and configuration tests.

---

# 111. Self-Hosted Configuration Validation

Before start validate:

```text
hostname
TLS
listen ports
capacity
metrics bind
admin bind
```

Fail fast on unsafe config.

---

# 112. RON Configuration

Your platform wrapper may use RON:

```ron
(
    relay_id: "in-delhi-1",
    region: "in-delhi",
    mode: SelfHosted,
    public_hostname: "relay.example",
)
```

Then translate into current Iroh relay configuration.

Do not fork Iroh's config format unnecessarily unless wrapper adds real value.

---

# 113. Secrets

Relay secrets/API credentials belong in:

```text
environment secret
secret manager
protected file
```

not committed RON config.

---

# 114. Configuration Split

```text
relay.ron
```

contains non-secret policy.

Secrets loaded separately.

---

# 115. Container Deployment

Container image should:

```text
run as non-root where possible
read-only root filesystem where practical
explicit writable paths
health checks
version labels
```

---

# 116. Systemd Deployment

Provide:

```text
systemd service
restart policy
resource limits
sandboxing
```

for simple Linux deployments.

---

# 117. Kubernetes Deployment

Optional architecture:

```text
Deployment/Stateful-style service as required by network model
PodDisruptionBudget
anti-affinity
metrics
rolling update
```

Do not make Kubernetes mandatory.

---

# 118. Autoscaling

Relay autoscaling based on:

```text
concurrent connections
bandwidth
CPU
memory
```

not just HTTP request count.

---

# 119. Scaling Caveat

Relay connections may be long-lived.

Traditional request-per-second autoscaling may react poorly.

Prefer connection/bandwidth-aware scaling.

---

# 120. Stable Relay Identity vs Instance

Logical relay:

```text
india-west-1
```

may be served by changing infrastructure instances.

Client trust should be tied to configured service identity/URL policy rather than ephemeral VM identity.

---

# 121. Drain-Aware Autoscaling

Before scaling down:

```text
drain
```

to reduce connection disruption.

---

# 122. Load Balancing

Use only load-balancing patterns compatible with Iroh relay behavior.

Do not assume any generic Layer-7 HTTP load balancer works transparently.

Test chosen deployment.

---

# 123. Anycast

Potential future optimization for global relay ingress.

Complex operationally.

Not necessary initially.

---

# 124. Multi-Cloud

Useful only after scale/reliability justify.

Recommended abstraction supports:

```text
AWS
GCP
Azure
Hetzner
bare metal
```

without client changes.

---

# 125. Cost-Aware Relay Selection

Operator may assign:

```text
cost class
```

to relay pools.

Clients can prefer lower-cost while respecting latency/reliability.

Do not expose financial pricing details to peers.

---

# 126. Bandwidth Quotas

Per project/tenant:

```text
soft quota
hard quota
burst
```

But critical safety communication may have reserved policy depending deployment.

---

# 127. Billing Separation

Hosted relay billing is operational SaaS logic.

It must not affect:

```text
message encryption
identity
protocol semantics
```

---

# 128. Abuse Isolation

One tenant's abuse should not make other tenants unusable.

Use:

```text
per-tenant quotas
dedicated pools for large customers
global emergency protection
```

---

# 129. Noisy Neighbor Detection

Metrics:

```text
tenant connection share
bandwidth share
error share
```

trigger throttling/migration.

---

# 130. Private Relay Authentication

Enterprise relay can require project credential before providing relay service.

This prevents public abuse.

P2P peer authentication happens separately end-to-end.

---

# 131. Relay Audit Events

Operator audit:

```text
config changed
credential rotated
relay added
relay drained
version upgraded
```

not user message events.

---

# 132. Administrative RBAC

Control plane roles:

```text
Viewer
Operator
SecurityAdmin
InfrastructureAdmin
```

Separate from communication user roles.

---

# 133. Secure Admin Interface

Use:

```text
strong authentication
TLS
least privilege
audit trail
```

Avoid exposing unauthenticated debug endpoints.

---

# 134. SSH/Admin Access

Prefer:

```text
short-lived keys
VPN/private network
restricted operators
```

Production operations should be reproducible, not manual snowflake servers.

---

# 135. Infrastructure as Code

Maintain:

```text
Terraform/OpenTofu or equivalent
Ansible/system config
container manifests
```

The application architecture does not depend on one IaC tool.

---

# 136. Immutable Deployment

Prefer:

```text
replace instance
```

over manually mutating production server for upgrades.

This makes rollback/recovery easier.

---

# 137. Secrets Rotation Runbook

Document:

```text
issue new credential
deploy overlap
update clients/control plane
revoke old
verify
```

---

# 138. Certificate Failure Runbook

Document:

```text
renew
switch relay
disable affected endpoint
verify failover
```

---

# 139. Region Failure Runbook

```text
mark region unhealthy
clients select alternate
drain DNS/config
investigate
```

---

# 140. Relay Compromise

If relay host compromised:

```text
remove from signed config
rotate relay/admin credentials
rebuild from clean image
review metadata exposure
```

E2EE should limit content compromise.

---

# 141. Relay Compromise Limits

A relay compromise may expose:

```text
metadata
traffic timing
IP addresses
operational credentials
```

but should not expose:

```text
application plaintext
account private keys
blob decryption keys
```

if boundaries are respected.

---

# 142. Key Separation

Relay TLS/admin keys:

```text
≠ account identity keys
≠ device keys
≠ message encryption keys
```

---

# 143. Incident Kill Switch

Signed infrastructure config should be able to:

```text
remove relay
disable pool
force fallback
```

without application update.

---

# 144. Emergency Bootstrap

Keep at least one independent fallback path:

```text
bundled known relays
```

for control-plane outage.

---

# 145. Split-Brain Config

If clients hold different relay-set versions:

```text
they can still communicate directly
```

and may use different compatible relays.

Infrastructure config should not become application-level consensus.

---

# 146. Relay Discovery Privacy

Do not expose user's full relay configuration to arbitrary peers unless protocol needs it.

Peer only needs connectivity information needed for current endpoint discovery.

---

# 147. NAT Traversal Observability

Track:

```text
direct success
hole-punch attempts
relay fallback
time to usable path
```

This is one of the most useful network health metrics.

---

# 148. Path Upgrade Observability

Connections may begin relayed and become direct.

Record:

```text
relay → direct
```

as diagnostics.

This helps cost/performance analysis.

---

# 149. Relay Stickiness Observability

Monitor:

```text
relay switches/session
```

High churn indicates instability.

---

# 150. SLOs

Example relay SLO dimensions:

```text
availability
connection success
median/P95 relay latency
failover success
TLS availability
```

Set actual numeric targets after benchmarking.

---

# 151. SLI Definitions

Define exact measurement:

```text
connection_success =
successful relay connection /
eligible relay connection attempts
```

Avoid vague "uptime".

---

# 152. Alerting

Alert on:

```text
connection failure spike
latency spike
bandwidth saturation
FD pressure
memory pressure
certificate expiry
region unreachable
```

---

# 153. Alert Noise

Use:

```text
multi-window thresholds
```

to avoid paging on tiny transient fluctuations.

---

# 154. Dashboard

Operator dashboard:

```text
relay fleet
regions
health
connections
bandwidth
direct/relay ratio
errors
versions
certificate status
```

No application plaintext.

---

# 155. Logs vs Metrics vs Traces

Use:

```text
metrics → fleet health
logs → discrete errors/audit
traces → selected connection diagnostics
```

Do not trace every byte.

---

# 156. Trace Sampling

High-volume relay fleet should sample traces.

Security/critical failures may use higher sampling.

---

# 157. Client Diagnostics

Client can expose:

```text
home relay
relay RTT
direct/relayed status
last failover
```

to advanced diagnostics.

---

# 158. User-Facing Network State

Normal UI:

```text
Connected
Connecting
Limited network
Offline
```

Do not make ordinary users choose relay servers.

---

# 159. Advanced User UI

Optional:

```text
Use community relays
Use custom relay
Private relay only
```

with clear consequences.

---

# 160. Self-Hosted Admin UX

A future admin console may manage:

```text
relay URL
health
region
version
credential status
```

but CLI/config should remain sufficient.

---

# 161. Headless Control

All relay operations must be scriptable.

No GUI dependency.

---

# 162. Infrastructure API

Potential internal interface:

```rust
pub trait InfrastructureDirectory {
    async fn relay_pools(&self) -> Result<Vec<RelayPool>, InfraError>;
    async fn config_version(&self) -> Result<u64, InfraError>;
}
```

---

# 163. Relay Adapter Interface

```rust
pub trait RelayAdapter {
    async fn configure(
        &self,
        relays: &[RelayCandidate],
    ) -> Result<(), RelayError>;

    async fn health(
        &self,
    ) -> Result<Vec<RelayRuntimeStatus>, RelayError>;
}
```

Iroh implementation translates to current endpoint relay configuration.

---

# 164. Iroh-Specific Module

Recommended:

```text
comm-transport-iroh/
├── endpoint.rs
├── relay.rs
├── discovery.rs
└── metrics.rs
```

Keep current Iroh API churn contained here.

---

# 165. Iroh Version Isolation

Because Iroh is still evolving toward a stable long-term API, avoid spreading its concrete types across all crates.

This substantially lowers upgrade cost.

---

# 166. Adapter Compatibility Tests

For every Iroh upgrade:

```text
relay config
direct fallback
home relay selection
connection establishment
```

must pass Part 10 regression tests.

---

# 167. Relay Config Compatibility

Your persistent application config should use your own stable schema.

Translate into the current Iroh API at runtime.

This avoids forcing user config migration on every upstream API rename.

---

# 168. Production Upgrade Discipline

When upgrading Iroh:

```text
read changelog
run protocol tests
run relay failover tests
canary
then production
```

Never auto-bump critical networking dependencies blindly.

---

# 169. Upstream vs Platform Responsibility

Use Iroh for:

```text
QUIC connectivity
NAT traversal
relay transport
endpoint behavior
```

Your platform owns:

```text
product relay policy
tenant policy
self-hosting profiles
signed infrastructure config
observability integration
DTN gateway integration
```

---

# 170. Relay Security Review

Review:

```text
TLS config
admin exposure
rate limits
dependency versions
container privileges
logging
credentials
```

before release.

---

# 171. Fuzzing

Part 10 should fuzz:

```text
your relay config parser
relay policy messages
control-plane config envelope
adapter boundary
```

Upstream protocol fuzzing remains upstream responsibility plus integration regression.

---

# 172. Chaos Testing

Kill:

```text
home relay
whole region
DNS
control database
```

during:

```text
message
file
call
```

Expected:

```text
safe failover/degradation
```

---

# 173. Regional Partition Test

Clients in:

```text
Region A
Region B
```

with Region A relays down.

Direct connections or alternate regions should continue according to policy.

---

# 174. Private-Only Failure Test

If organization relay fleet is entirely unavailable and public fallback forbidden:

```text
Internet P2P may still work direct
```

where reachable.

If not:

```text
client reports limited/offline
```

and may use local/DTN paths.

---

# 175. Certificate Expiry Test

Simulate expired relay cert.

Client must:

```text
reject insecure connection
select alternate
```

never disable validation silently.

---

# 176. Config Rollback Test

Feed older signed infra config.

Expected:

```text
reject/ignore according to rollback policy
```

---

# 177. Config Signature Test

Tamper relay URL.

Expected:

```text
config rejected
```

---

# 178. Rate-Limit Test

Flood self-hosted relay.

Expected:

```text
bounded resources
other tenants remain usable
```

---

# 179. Soak Test

Run:

```text
thousands of connect/disconnect cycles
```

checking:

```text
memory
FDs
connection leaks
relay switch behavior
```

---

# 180. Long-Lived Session Test

Keep connections alive for:

```text
hours/days
```

through network changes.

Ensure relay infrastructure remains stable.

---

# 181. Capacity Test

Load until:

```text
soft threshold
```

then:

```text
hard threshold
```

Validate:

```text
admission
metrics
alerts
graceful degradation
```

---

# 182. Security Test

Attempt:

```text
unauthorized admin access
invalid credentials
malformed client
oversized handshake
```

No crash/unbounded allocation.

---

# 183. Suggested Workspace

```text
infrastructure/
├── relay/
│   ├── README.md
│   ├── configs/
│   ├── systemd/
│   ├── container/
│   ├── kubernetes/
│   └── dashboards/
│
├── bootstrap/
├── control-plane/
├── observability/
└── runbooks/

crates/
├── comm-relay-policy/
├── comm-infra-config/
├── comm-transport-iroh/
└── comm-relay-admin/
```

---

# 184. `comm-relay-policy`

Responsibilities:

```text
relay pools
selection policy
region policy
fallback rules
cost classes
```

No Iroh concrete API types.

---

# 185. `comm-infra-config`

Responsibilities:

```text
signed config
version
rollback protection
bootstrap
serialization
```

---

# 186. `comm-relay-admin`

Optional operator tooling:

```text
health
fleet status
config validation
doctor
```

---

# 187. Deployment Artifacts

Repository should provide:

```text
example self-host config
systemd unit
container build
health-check script
metrics example
firewall guidance
runbooks
```

---

# 188. Documentation

Required:

```text
self-hosting.md
relay-security.md
relay-scaling.md
relay-observability.md
relay-upgrades.md
relay-incident-response.md
private-enterprise-deployment.md
dtn-gateway.md
```

---

# 189. Initial Production Scope

Implement first:

```text
Iroh relay adapter
static + signed relay pools
self-hosted single-node deployment
3+ relay production pool support
health probes
client failover
region tags
metrics
rate limits
systemd/container deployment
TLS/certificate monitoring
relay doctor
```

Then:

```text
dedicated tenant pools
control-plane config service
multi-region automation
blue/green rollout
DTN gateway
```

Defer initially:

```text
anycast
complex global autoscaling
multi-cloud orchestration
custom relay protocol
```

---

# 190. Implementation Phases

## Phase 1 — Relay Abstraction

```text
RelayId
RelayCandidate
RelayPool
RelayPolicy
```

## Phase 2 — Iroh Adapter

```text
current relay config mapping
health
home relay diagnostics
```

## Phase 3 — Self-Hosting

```text
single relay
DNS
TLS
systemd/container
```

## Phase 4 — Resilience

```text
multi-relay
failover
hysteresis
regional pools
```

## Phase 5 — Operations

```text
metrics
alerts
doctor
runbooks
```

## Phase 6 — Security/Isolation

```text
access credentials
tenant quota
admin RBAC
signed infra config
```

## Phase 7 — DTN Gateway

```text
local store-carry-forward
Internet bridge
Iroh delivery
```

## Phase 8 — Scale

```text
multi-region
canary/blue-green
capacity testing
```

---

# 191. Definition of Done

Part 11 is complete when:

- direct P2P remains preferred where policy allows
- NAT traversal failure can use configured relay infrastructure
- the application can use public, dedicated, or self-hosted relay pools
- public fallback can be disabled
- relay URLs are not hard-coded throughout application logic
- Iroh-specific relay types stay behind an adapter
- relay configuration can be updated through signed/versioned policy
- relay config rollback is prevented
- clients have multiple production relay candidates
- relay failure does not corrupt message/file state
- large file transfer resumes after relay switch
- relay control-plane failure does not automatically stop data-plane service
- application plaintext is not required at relay
- relay/admin credentials are separate from user/device keys
- per-tenant/per-peer resource limits exist
- TLS/certificate health is monitored
- health/readiness/synthetic probes exist
- observability reveals direct-vs-relay behavior
- regional failover is tested
- self-hosted deployment works without Kubernetes
- headless DTN gateway can bridge local offline networks to Iroh/Internet
- chaos, certificate, config-signature, rate-limit, and soak tests exist

---

# 192. Relationship to Earlier Parts

Part 11 builds on:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
03 — Transport & Routing Policy Engine
04 — Offline Event Log
05 — Robust File / Blob Subsystem
06 — DTN / Store-Carry-Forward
07 — Capability Negotiation
08 — Resource Limits & Backpressure
09 — Crash Recovery
10 — Fuzzing & Protocol Test Suite
```

It prepares:

```text
12 — Multipath Networking
13 — Battery-Aware Scheduling
14 — Proximity Abstraction
15 — QR / NFC Bootstrap Pairing
16 — Daemon & Headless Runtime
17 — Emergency Priority Architecture
18 — Network Diagnostics & Path Visualization
19 — C ABI / FFI
20 — Embedded Linux Node
21 — WASM-Compatible Components
22 — Third-Party Protocol Extensions
23 — External Interoperability Suite
24 — Plugin / Module Ecosystem
```

---

# 193. Final Architecture

```text
                           CLIENTS
                ┌────────────┼────────────┐
                │            │            │
             Android       Desktop      Headless
                │            │            │
                └────── Communication ────┘
                             Runtime
                                │
                         Routing Policy
                                │
             ┌──────────────────┼───────────────────┐
             │                  │                   │
          Direct P2P         Relay Path          Local/DTN
             │                  │                   │
             │          Relay Abstraction           │
             │                  │                   │
             │        ┌─────────┼─────────┐         │
             │        │         │         │         │
             │      Public   Dedicated  Self-host   │
             │        │         │         │         │
             │        └─────────┼─────────┘         │
             │                Iroh                  │
             └──────────────────┼───────────────────┘
                                │
                         Remote Endpoint
```

Operational infrastructure:

```text
                    Signed Infrastructure Config
                              │
                 ┌────────────┼────────────┐
                 │            │            │
              Relay Fleet   Health      Operator
                 │          /Metrics    Control
          ┌──────┼──────┐
          │      │      │
        Region  Region  Region
          A      B      C
```

Disaster bridge:

```text
Offline Mesh
    ↓
DTN Gateway
    ↓
Internet available later
    ↓
Iroh Direct/Relay
    ↓
Destination
```

---

# 194. Final Principle

The relay architecture should ensure:

> **Relays improve reachability without becoming the center of trust or the center of product state.**

The application should be able to move between:

```text
community relays
managed dedicated relays
self-hosted relays
organization-private relays
```

without redesigning messaging, files, identity, or DTN.

When direct networking works:

```text
use direct
```

When it does not:

```text
relay reliably
```

When Internet disappears entirely:

```text
local/mesh/DTN continues
```

When Internet returns:

```text
DTN gateways bridge back into Iroh
```

That layered behavior is what makes the communication platform resilient enough for ordinary consumer use, enterprise deployment, self-hosting, and disaster/emergency operation.
