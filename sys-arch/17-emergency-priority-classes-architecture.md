# Part 17 — Emergency Priority Classes Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 17 of 24  
**Primary language:** Rust  
**Primary goals:** explicit emergency traffic classes, trustworthy priority authorization, reserved capacity, preemption, bounded replication, deadline-aware routing, DTN integration, emergency broadcast, authority alerts, abuse resistance, degraded-network operation, cross-platform reuse

---

# 1. Purpose

A disaster-capable communication platform must distinguish between:

```text
typing indicator
normal chat
file attachment
location update
medical request
SOS
authority alert
```

Treating all traffic equally is incorrect.

The emergency architecture must ensure that under:

```text
network congestion
low battery
storage pressure
relay overload
DTN partition
radio scarcity
multiple simultaneous transfers
```

the most important traffic still has a path through the system.

The core rule is:

> **Emergency priority must be explicit, authenticated where required, bounded, and enforced consistently across every subsystem.**

Priority is not merely UI decoration.

It affects:

- queue ordering
- resource reservation
- routing
- multipath
- DTN replication
- storage eviction
- battery scheduling
- relay admission
- notification behavior
- retry policy
- expiry
- delivery confirmation

---

# 2. Architectural Position

```text
Application Intent
      ↓
Emergency Classification
      ↓
Priority Authorization
      ↓
Delivery Requirements
      ↓
Resource Admission
      ↓
Routing / Multipath / DTN
      ↓
Transport
```

Cross-cutting integrations:

```text
Part 03 — Routing
Part 05 — Files
Part 06 — DTN
Part 08 — Resources
Part 11 — Relays
Part 12 — Multipath
Part 13 — Battery
Part 14 — Proximity
Part 16 — Daemon
```

---

# 3. Priority Is Not Trust

A sender may request:

```text
Critical
```

but that does not mean the system should honor it.

Priority must be mapped through:

```text
identity
role
context
policy
rate limits
user intent
```

---

# 4. Recommended Priority Classes

```rust
pub enum EmergencyPriority {
    Routine,
    Important,
    Urgent,
    Critical,
    AuthorityCritical,
}
```

---

# 5. Routine

Examples:

```text
ordinary chat
typing
presence
normal background sync
bulk file
```

Characteristics:

```text
no emergency reserve
normal retry
normal expiry
```

---

# 6. Important

Examples:

```text
important user message
requested document
normal delivery receipt
priority contact update
```

Characteristics:

```text
higher scheduling weight
still not allowed to preempt safety traffic
```

---

# 7. Urgent

Examples:

```text
time-sensitive help request
urgent location update
medical assistance request
family emergency message
```

Characteristics:

```text
shorter retry delay
eligible for DTN
may use reserved queue fraction
```

---

# 8. Critical

Examples:

```text
SOS
distress signal
life-safety location
critical incident report
```

Characteristics:

```text
reserved capacity
preemption
aggressive route search
redundant delivery
DTN replication
stronger persistence
```

---

# 9. AuthorityCritical

Reserved for:

```text
verified emergency authority
trusted organization command
evacuation alert
civil defense alert
```

It must require:

```text
cryptographic authority
policy approval
strict rate limits
```

Unknown peers cannot self-assign this class.

---

# 10. Priority Mapping

Remote requested priority:

```text
RequestedPriority
      ↓
Authorization Policy
      ↓
EffectivePriority
```

Example:

```text
unknown peer asks AuthorityCritical
→ downgrade or reject
```

---

# 11. Priority Authorization

```rust
pub trait PriorityAuthorizer {
    fn authorize(
        &self,
        requester: &IdentityContext,
        requested: EmergencyPriority,
        operation: &EmergencyOperation,
    ) -> PriorityAuthorizationDecision;
}
```

---

# 12. Authorization Decision

```rust
pub enum PriorityAuthorizationDecision {
    Allow(EmergencyPriority),
    Downgrade(EmergencyPriority),
    Reject,
}
```

---

# 13. Local User Intent

A local user pressing:

```text
SOS
```

is stronger evidence than a remote peer merely tagging a frame critical.

Still apply:

```text
rate limits
duplicate suppression
expiry
```

---

# 14. Emergency Operation Types

```rust
pub enum EmergencyOperation {
    Sos,
    DistressMessage,
    LocationBeacon,
    MedicalRequest,
    SafetyCheck,
    AuthorityAlert,
    EmergencyAttachment,
    EmergencyAck,
}
```

---

# 15. Emergency Message Envelope

```rust
pub struct EmergencyEnvelope {
    pub emergency_id: EmergencyId,
    pub priority: EmergencyPriority,
    pub operation: EmergencyOperation,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    pub sender: IdentityRef,
    pub payload_ref: EmergencyPayloadRef,
    pub authorization: EmergencyAuthorization,
}
```

---

# 16. Emergency ID

```rust
pub struct EmergencyId([u8; 16]);
```

Stable across:

```text
direct send
relay
DTN copy
multipath duplicate
retry
```

This enables end-to-end deduplication.

---

# 17. Emergency Payload Reference

```rust
pub enum EmergencyPayloadRef {
    Inline(Bytes),
    Event(EventId),
    Blob(BlobId),
    Bundle(BundleId),
}
```

Large media should not be embedded directly.

---

# 18. Emergency Authorization

Potential forms:

```text
local-user asserted
verified contact
organization-signed
authority-signed
device-role based
```

Do not force every emergency message to use same authority model.

---

# 19. Authority Identity

Part 02 may define special roles/certificates:

```text
EmergencyAuthority
OrganizationResponder
TrustedGateway
```

Role assignment must be explicit and revocable.

---

# 20. Authority Alert Security

An authority alert must bind:

```text
issuer
scope
expiry
message
priority
protocol version
```

into signature/authenticated envelope.

---

# 21. Authority Alert Scope

```rust
pub enum AuthorityScope {
    Individual,
    Group,
    Organization,
    NearbyArea,
    Region,
    EmergencyDomain,
}
```

---

# 22. No Unbounded Broadcast

Even AuthorityCritical must have:

```text
expiry
scope
rate limit
dedup
```

No infinite flood.

---

# 23. Reserved Resource Capacity

Part 08 should reserve capacity for:

```text
Critical
AuthorityCritical
```

Examples:

```text
queue slots
storage bytes
DTN bytes
connection attempts
CPU verification budget
```

---

# 24. Critical Queue Reserve

Normal/bulk traffic must not fill all queue slots.

Example:

```text
100 queue slots
10 reserved for Critical
```

Values are configuration/profile-specific.

---

# 25. Storage Reserve

Keep reserved storage for:

```text
small SOS
location
ACKs
identity/security events
```

Large video must not consume it.

---

# 26. Bandwidth Reserve

When possible, preserve capacity for:

```text
critical control
text
location
```

over large file transfer.

---

# 27. Preemption

Critical work may temporarily preempt:

```text
background sync
bulk file
relay cache work
thumbnail generation
```

---

# 28. Preemption Must Be Safe

Do not preempt mid-transaction in a way that corrupts durable state.

Use safe pause boundaries:

```text
chunk boundary
task boundary
scheduler quantum
```

---

# 29. Critical Work Ordering

Recommended:

```text
AuthorityCritical control
Critical SOS/control
Critical text/location
Urgent text
Emergency media preview
Normal interactive
Bulk
Background
```

Use fairness within classes.

---

# 30. Strict Priority vs Starvation

Do not allow permanent starvation.

Use:

```text
critical reserve
weighted fairness
bounded preemption
```

rather than strict-priority everything.

---

# 31. Delivery Requirements

Emergency operation should translate into explicit requirements.

```rust
pub struct EmergencyDeliveryRequirements {
    pub priority: EmergencyPriority,
    pub deadline: Option<Timestamp>,
    pub max_latency: Option<Duration>,
    pub require_ack: bool,
    pub dtn_allowed: bool,
    pub redundancy: RedundancyPolicy,
    pub max_replication: u8,
}
```

---

# 32. Deadline-Aware Scheduling

An emergency payload with:

```text
expires in 5 min
```

should not sit behind a 2-hour bulk transfer.

Deadline becomes routing/scheduler input.

---

# 33. Expiry

Emergency traffic must expire.

Examples:

```text
SOS → hours/day
location beacon → minutes
authority evacuation alert → hours
```

Do not forward stale emergencies indefinitely.

---

# 34. Location Update Expiry

A stale location may be harmful.

Use short lifetime.

Each update can supersede previous.

---

# 35. Coalescing Emergency Location

If new location arrives:

```text
old undelivered location
```

may be superseded if same emergency/session and policy permits.

---

# 36. SOS Lifecycle

```text
Created
 ↓
Persisted
 ↓
Sending
 ↓
Relayed
 ↓
ReachedGateway
 ↓
Delivered
 ↓
Acknowledged
```

Alternative:

```text
Cancelled
Expired
```

---

# 37. Persist Before Transmission

SOS must be durably committed before any network attempt.

Part 04/09 invariant.

---

# 38. Emergency Routing Policy

Part 03 should score:

```text
reachability
latency
diversity
energy
cost
```

differently under Critical priority.

Reliability may outweigh cost/energy.

---

# 39. Direct Path Preference

If reliable direct path exists:

```text
send immediately
```

but critical policy may also create a second independent copy.

---

# 40. Redundant Delivery

Critical traffic may use:

```text
Internet direct/relay
+
DTN
```

or:

```text
Wi-Fi
+
cellular
```

when allowed.

---

# 41. Path Diversity

Redundancy should prefer distinct failure domains.

Example:

```text
Wi-Fi + cellular
```

better than:

```text
two relay connections over same Wi-Fi
```

---

# 42. Multipath Integration

Part 12 can implement:

```text
redundant control
warm failover
```

for emergencies.

Do not stripe tiny SOS text.

---

# 43. DTN Integration

Critical traffic is a core DTN use case.

Recommended:

```text
SOS text
location
small metadata
thumbnail
```

receive high replication priority.

---

# 44. DTN Replication Budget

Example policy:

```text
Routine = 0–1
Urgent = 2–4
Critical = 4–8
AuthorityCritical = bounded policy
```

Exact values should be tuned.

---

# 45. Critical DTN Reserve

Relay store should preserve a reserved portion for critical bundles.

---

# 46. DTN Eviction

Under pressure:

```text
expired
delivered
bulk
normal
urgent
critical
authority-critical last
```

---

# 47. Emergency Gateway

A nearby charging/headless node may advertise:

```text
EmergencyGateway
```

as capability.

It can bridge:

```text
BLE/LAN/DTN
→ Iroh/Internet
```

---

# 48. Gateway Trust

Gateway can carry encrypted emergency payload without plaintext authority.

Authority verification occurs end-to-end.

---

# 49. Emergency Broadcast

There are two broad modes:

```text
private emergency unicast
public/organizational alert broadcast
```

Keep them separate.

---

# 50. Private Emergency

Examples:

```text
SOS to family
medical request to responder group
```

E2EE to intended recipients.

---

# 51. Public Emergency Alert

Examples:

```text
evacuation order
hazard notice
shelter update
```

Signed, readable by intended local/public audience.

---

# 52. Broadcast Security

Require:

```text
trusted issuer
scope
expiry
signature
version
```

Receivers verify before presenting as authoritative.

---

# 53. Unknown Emergency Reports

Anonymous/unknown distress reports may still be useful.

Policy may allow:

```text
low-bandwidth urgent relay
```

with:

```text
strict quota
unverified label
no AuthorityCritical
```

---

# 54. Anonymous SOS

If supported:

```text
anonymous emergency report
```

should be visibly marked unverified.

Do not masquerade as trusted authority.

---

# 55. Spam Resistance

Emergency channels are high-value attack targets.

Use:

```text
per-origin rate limit
per-peer quota
proof-of-contact/identity where available
short expiry
duplicate suppression
priority authorization
```

---

# 56. Fake SOS Flood

Unknown peer flood must not consume all critical reserve.

Reserve may be partitioned:

```text
local-user critical
trusted-authority critical
unknown-emergency limited
```

---

# 57. Priority Buckets

Example:

```rust
pub enum CriticalReserveBucket {
    LocalUser,
    TrustedAuthority,
    VerifiedPeer,
    UnknownEmergency,
}
```

Each has quota.

---

# 58. Local User Reserve

A device must preserve enough resource for its own user to send SOS even under remote flood.

---

# 59. Authority Reserve

Verified authorities can have separate bounded capacity.

---

# 60. Unknown Emergency Quota

Allow humane emergency functionality without enabling trivial DoS.

---

# 61. Verification CPU Budget

Authority signatures may be expensive.

Use bounded verification queues.

Unknown senders should face cheaper filters first.

---

# 62. Cheap Validation First

For emergency frame:

```text
size
version
expiry
rate limit
duplicate
then signature
```

---

# 63. Emergency Attachments

Priority should depend on attachment type.

Recommended:

```text
text → highest
location → highest
thumbnail → high
voice → high
full image → medium
video → low/bulk
```

---

# 64. Attachment Derivatives

Part 05 supports:

```text
thumbnail
preview
original
```

Emergency scheduler can send small derivative first.

---

# 65. Thumbnail-First

Example:

```text
photo 8 MB
thumbnail 20 KB
```

Send:

```text
SOS text
location
thumbnail
```

first.

Full image later.

---

# 66. Voice Note

A short low-bitrate voice note may carry important context.

Classify above full-resolution media.

---

# 67. Video

Video is expensive.

Do not allow a large emergency video to block:

```text
text
location
ACK
```

---

# 68. Emergency File Chunking

Large emergency file still uses Part 05 chunking.

Scheduler prioritizes first meaningful chunks/derivatives.

---

# 69. Media Calls

Emergency call should prioritize:

```text
audio continuity
```

over video quality.

Part 13 media degradation applies.

---

# 70. Emergency Call Ladder

```text
video + audio
 ↓
lower video
 ↓
audio only
 ↓
voice note
 ↓
text/SOS
```

---

# 71. Battery-Aware Emergency Policy

Low battery does not mean "turn everything up."

Emergency mode should:

```text
increase reachability
reserve power for critical
suppress noncritical
```

---

# 72. Critical Battery

At critically low battery:

```text
send SOS/text/location
minimize video
limit large media
reduce relay burden
```

unless user explicitly overrides.

---

# 73. Charging Emergency Node

A charging device may accept more relay load.

---

# 74. Thermal Emergency Policy

Do not overheat device with:

```text
AV1 software encode
massive Wi-Fi transfer
```

when simple text/location would suffice.

---

# 75. Nearby Emergency Discovery

Part 14 emergency scan may increase:

```text
BLE discovery
gateway discovery
DTN peer detection
```

within bounded policy.

---

# 76. Emergency Pairing

Part 15 can support:

```text
EmergencyTeamPairing
```

for responders/gateways.

---

# 77. Emergency Team Roles

Examples:

```text
Responder
Coordinator
Gateway
Authority
Observer
```

Roles affect permissions/priority.

---

# 78. Group Emergency Channel

A team/group can have:

```text
emergency channel
```

with strict member/authority rules.

---

# 79. Group Fan-Out

Avoid naive:

```text
send independently to every member immediately
```

for huge groups.

Use group protocol/relay/DTN policy.

---

# 80. Delivery Acknowledgement

Critical traffic should distinguish:

```text
stored locally
relayed
reached gateway
delivered to recipient
acknowledged by human
```

---

# 81. Human Acknowledgement

Some operations need:

```text
"I saw this"
```

separate from device delivery.

---

# 82. Emergency ACK Types

```rust
pub enum EmergencyAckKind {
    DeviceDelivered,
    UserSeen,
    ResponderAccepted,
    AuthorityConfirmed,
}
```

---

# 83. ACK Priority

ACKs should be high priority and tiny.

---

# 84. ACK via DTN

If no direct path, ACK can travel store-carry-forward.

---

# 85. Cancellation

User may cancel SOS.

Cancellation must itself be:

```text
high priority
signed/authenticated
DTN-capable
```

---

# 86. Cancellation Is Best Effort

Cannot erase already delivered message.

Remote nodes should mark:

```text
cancelled
```

if they later receive cancel event.

---

# 87. Superseding Alerts

Authority may issue:

```text
Update
Cancel
AllClear
```

with same alert lineage.

---

# 88. Alert Lineage

```rust
pub struct AlertSeriesId([u8; 16]);
```

Allows:

```text
initial alert
update
cancel
```

---

# 89. Alert Version

Within series:

```text
monotonic sequence
```

Receivers ignore stale updates.

---

# 90. Offline Authority Alerts

Signed alert can spread via:

```text
DTN
BLE
LAN
Wi-Fi
```

without Internet.

---

# 91. Authority Key Rotation

Part 02/security infrastructure must support:

```text
authority certificate rotation
revocation
```

Clients should reject stale revoked issuer credentials.

---

# 92. Time Without Internet

Expiry cannot rely perfectly on global clock.

Use:

```text
issued time
lifetime
trusted local age
bounded skew
```

and authority semantics.

---

# 93. Replay Protection

Same `EmergencyId`:

```text
display once
```

unless it is an update/new sequence.

---

# 94. Duplicate Paths

Same SOS may arrive via:

```text
Iroh
DTN
LAN
```

One logical emergency record.

---

# 95. Emergency Event Log

Part 04 semantic events:

```text
EmergencyCreated
EmergencyUpdated
EmergencyDelivered
EmergencyAcknowledged
EmergencyCancelled
EmergencyExpired
```

---

# 96. No Low-Level Retry Events

Do not permanently journal every retransmission.

---

# 97. Crash Recovery

Pending emergency state is among first recovered.

Part 09 startup order should prioritize:

```text
identity
emergency
critical outbox
```

---

# 98. Recovery After Send Ambiguity

Retry same `EmergencyId`.

Recipient deduplicates.

---

# 99. Daemon Ownership

Part 16 daemon should continue emergency sending even if UI closes.

---

# 100. UI Crash

Emergency state remains in daemon.

UI reconnects and sees status.

---

# 101. Headless Emergency Node

A headless node can act as:

```text
gateway
authority relay
DTN relay
alert distributor
```

without messaging UI.

---

# 102. Emergency Runtime Mode

```rust
pub enum EmergencyRuntimeMode {
    Off,
    Prepared,
    Active,
}
```

---

# 103. Prepared Mode

Can reserve:

```text
small storage
capability
configuration
```

without aggressive scanning.

---

# 104. Active Mode

Enables:

```text
higher discovery
critical routing
DTN reserve
priority scheduling
```

---

# 105. Activation

Can be triggered by:

```text
user
trusted organization
device policy
```

Be careful with remote activation.

---

# 106. Remote Activation Policy

Do not let arbitrary remote peer force:

```text
high battery drain
aggressive radio use
```

Only trusted authority/policy can activate emergency mode remotely.

---

# 107. Emergency Deactivation

Can happen via:

```text
user
all-clear
expiry
admin
```

---

# 108. Data Usage

Critical emergency may allow metered transport if user policy says emergency override allowed.

This should be explicit.

---

# 109. Cost Policy

```rust
pub struct EmergencyCostPolicy {
    pub allow_metered_for_critical: bool,
    pub allow_roaming_for_critical: bool,
    pub allow_public_relay: bool,
}
```

---

# 110. Privacy Policy

Emergency may reveal:

```text
location
identity
medical context
```

Only include what operation needs.

---

# 111. Location Privacy

Do not automatically attach precise location to every emergency.

User/product policy decides.

---

# 112. Coarse vs Precise Location

Support:

```text
precise
coarse
none
```

when possible.

---

# 113. Sensitive Payload Encryption

Private SOS remains E2EE.

Relay/DTN nodes carry ciphertext.

---

# 114. Public Alert Payload

Authority broadcast is intentionally readable by target public domain, but signature/authenticity remains mandatory.

---

# 115. Emergency Metadata

Minimize relay-visible metadata.

Priority may need to be visible to scheduler, but detailed content should not.

---

# 116. Priority Privacy Trade-Off

If relay sees:

```text
Critical
```

it learns something about traffic.

Could use coarse transport classes where necessary.

Document this.

---

# 117. Priority Forgery

Relay/peer must not be able to mutate priority silently.

Priority must be integrity-bound.

---

# 118. Priority Downgrade Attack

If attacker strips critical marking:

```text
end-to-end integrity/transcript
```

should detect.

---

# 119. Priority Upgrade Attack

Attacker cannot change Routine to Critical without authorization.

---

# 120. Relay Infrastructure

Part 11 relay can enforce:

```text
connection-level critical reserve
```

without reading payload.

---

# 121. Tenant Emergency Quotas

Enterprise:

```text
tenant normal quota
+
tenant emergency reserve
```

No tenant gets infinite emergency bandwidth.

---

# 122. Abuse Monitoring

Track:

```text
critical claims
rejections
downgrades
authority verification failures
```

---

# 123. Telemetry Privacy

Do not export:

```text
emergency message contents
exact location
medical details
```

---

# 124. Emergency Metrics

Safe:

```text
critical messages created
delivered
delivery latency class
DTN relay count
gateway success
```

Use aggregate.

---

# 125. Diagnostics

Advanced local view:

```text
Emergency ID
Priority
Created
Expires
Paths attempted
DTN copies
Gateway reached
Delivered
ACK state
```

---

# 126. Normal UI

Keep simple:

```text
Sending emergency alert…
Carried by nearby devices
Reached Internet gateway
Delivered
```

---

# 127. Alert Verification UI

Authority alert should show:

```text
Verified authority
issuer
expiry
scope
```

without technical crypto details.

---

# 128. Unverified Alert UI

Unknown-origin emergency report:

```text
Unverified report
```

must be clearly distinguished from authority alert.

---

# 129. Accessibility

Emergency action should be:

```text
easy
clear
low interaction
```

but prevent accidental triggering where possible.

---

# 130. Accidental Trigger Protection

Possible:

```text
press-and-hold
confirmation
hardware gesture
```

Product-specific.

Do not make safety flow excessively cumbersome.

---

# 131. Lock-Screen Integration

Platform-specific future feature:

```text
quick emergency action
```

must still invoke Rust emergency service and durable commit.

---

# 132. Notification Priority

Critical incoming alert may use high-priority OS notifications where platform policy permits.

---

# 133. Do Not Abuse OS Critical Alerts

Authority/platform entitlements may be restricted.

Only use such OS mechanisms when legally/technically permitted.

---

# 134. Rate Limits

Example classes:

```text
Local SOS:
very high trust, strict user-trigger rate

Verified peer urgent:
moderate

Unknown emergency:
small quota

Authority alerts:
issuer-specific quota
```

---

# 135. Cooldown

Repeated identical emergency actions may coalesce.

Do not accidentally suppress legitimate updates.

---

# 136. Deduplication Key

Use:

```text
EmergencyId
AlertSeriesId + sequence
```

not text comparison.

---

# 137. Authority Alert Size

Keep alert body small.

Large media should be separate blob references.

---

# 138. Authority Broadcast via DTN

Bundle should carry:

```text
signed alert
scope
expiry
```

Relays do not need authority trust to forward, but clients verify before showing as trusted.

---

# 139. Geographic Scope

Offline geography is hard.

Possible sources:

```text
configured region
local network domain
cell/site hints
GPS
manual incident area
```

Do not overclaim precision.

---

# 140. Nearby Area Scope

May mean:

```text
bounded hop count
short expiry
local encounter domain
```

instead of exact geofence.

---

# 141. Resource Pressure

If memory/storage critical:

```text
evict bulk/cache
preserve emergency reserve
```

---

# 142. Network Congestion

Throttle:

```text
bulk
background
```

before emergency.

---

# 143. CPU Pressure

Suspend:

```text
indexing
compression
AV1 background encode
```

to preserve critical communication processing.

---

# 144. Battery Critical

Keep:

```text
text/location/ACK
```

reduce:

```text
video
bulk relay
```

---

# 145. Thermal Critical

Same principle.

---

# 146. Emergency Multipath Policy

```rust
pub struct EmergencyMultipathPolicy {
    pub use_redundant_paths: bool,
    pub max_paths: u8,
    pub require_failure_domain_diversity: bool,
}
```

---

# 147. Emergency DTN Policy

```rust
pub struct EmergencyDtnPolicy {
    pub enabled: bool,
    pub replication_budget: u8,
    pub hop_limit: u8,
    pub bundle_lifetime: Duration,
}
```

---

# 148. Emergency File Policy

```rust
pub struct EmergencyAttachmentPolicy {
    pub send_thumbnail_first: bool,
    pub max_auto_media_bytes: u64,
    pub voice_priority: EmergencyPriority,
}
```

---

# 149. Emergency Runtime Policy

```rust
pub struct EmergencyPolicy {
    pub priority_authorization: PriorityPolicy,
    pub cost: EmergencyCostPolicy,
    pub multipath: EmergencyMultipathPolicy,
    pub dtn: EmergencyDtnPolicy,
    pub attachments: EmergencyAttachmentPolicy,
}
```

---

# 150. Policy Source

Can come from:

```text
product defaults
user settings
organization config
authority policy
```

Hard safety limits always win.

---

# 151. Enterprise Policy

Organization can define:

```text
who may send authority alerts
allowed scope
max frequency
required signing role
```

---

# 152. School/ERP Example

A school may define:

```text
Principal/Admin:
AuthorityCritical campus alert

Teacher:
Urgent class safety message

Student:
Critical personal SOS
```

without changing core protocol.

---

# 153. Community Disaster Example

```text
Residents:
SOS / urgent

Community coordinator:
signed local alert

Gateway node:
relay only
```

---

# 154. Medical Deployment Example

```text
Patient:
SOS

Responder:
ResponderAccepted ACK

Hospital authority:
AuthorityCritical alert
```

---

# 155. Protocol Extension

Suggested:

```text
emergency/1
```

Capabilities:

```text
sos
location
authority_alert
dtn
ack
cancel
```

---

# 156. Capability Negotiation

Part 07 determines:

```text
which emergency features peer supports
```

But unsupported peers may still receive compatible fallback text.

---

# 157. Fallback

If recipient lacks `emergency/1`:

```text
send normal signed/high-priority message
```

where safe and meaningful.

Do not silently lose SOS.

---

# 158. Protocol Wire Types

Use versioned explicit types:

```text
EmergencyCreateV1
EmergencyAckV1
EmergencyCancelV1
AuthorityAlertV1
AuthorityAlertUpdateV1
```

---

# 159. Postcard

Postcard is suitable for small control/envelope structures.

Large media remains blob/chunk stream.

---

# 160. Parser Limits

Bound:

```text
alert size
metadata count
recipient count
scope descriptor
signature count
```

---

# 161. Fuzzing

Part 10 should fuzz:

```text
emergency envelope
authority alert
ack
cancel
scope
priority authorization metadata
```

---

# 162. State-Machine Fuzzing

Generate:

```text
create
update
deliver
ack
cancel
expire
duplicate
```

---

# 163. Property Tests

Invariants:

```text
Critical local SOS cannot be silently downgraded by remote input
AuthorityCritical requires authorized issuer
expired emergency never forwards
same EmergencyId is deduplicated
bulk cannot consume critical reserve
cancel never resurrects original
```

---

# 164. Priority Abuse Test

Unknown peer sends 10k `AuthorityCritical`.

Expected:

```text
rejected/downgraded
bounded CPU/memory/storage
```

---

# 165. Local Reserve Test

Remote flood fills normal/unknown emergency quotas.

Local user presses SOS.

Expected:

```text
accepted through local reserve
```

---

# 166. Storage Pressure Test

Disk nearly full.

Expected:

```text
bulk/cache evicted
small critical message persists
```

---

# 167. Battery Test

Battery critical.

SOS:

```text
text/location sends
large video deferred
```

---

# 168. Multipath Test

Wi-Fi + cellular.

Critical SOS:

```text
redundant paths if policy allows
```

---

# 169. DTN Partition Test

No Internet.

```text
A → B → C → gateway
```

SOS reaches destination and ACK eventually returns.

---

# 170. Authority Replay Test

Old signed evacuation alert replayed after expiry.

Expected:

```text
reject
```

---

# 171. Authority Revocation Test

Issuer revoked.

New alerts rejected.

Existing historical alert remains auditable but not newly trusted.

---

# 172. Alert Update Ordering Test

Receive:

```text
sequence 3
then 2
```

Keep sequence 3.

---

# 173. Cancel Race Test

Original and cancel arrive different paths.

Final state:

```text
cancelled
```

if cancel valid and later in lineage.

---

# 174. Duplicate Path Test

Same SOS via direct + DTN.

One logical alert.

---

# 175. Crash Test

Crash:

```text
after SOS commit
before send
```

Restart:

```text
SOS resumes
```

---

# 176. Daemon UI Test

Close UI while SOS pending.

Daemon continues.

---

# 177. Gateway Test

Nearby node receives encrypted emergency bundle and later obtains Internet.

Delivery continues.

---

# 178. Real-Device Test

Android:

```text
low battery
background
BLE
Wi-Fi
foreground service
```

validate actual behavior.

---

# 179. Suggested Crate Structure

```text
crates/comm-emergency/
├── src/
│   ├── lib.rs
│   ├── id.rs
│   ├── priority.rs
│   ├── operation.rs
│   ├── envelope.rs
│   ├── authorization.rs
│   ├── policy.rs
│   ├── routing.rs
│   ├── dtn.rs
│   ├── multipath.rs
│   ├── attachment.rs
│   ├── ack.rs
│   ├── broadcast.rs
│   ├── authority.rs
│   ├── lifecycle.rs
│   ├── diagnostics.rs
│   └── error.rs
└── Cargo.toml
```

---

# 180. Public API

```rust
let handle = emergency
    .send_sos(
        destination,
        EmergencyPayload::TextAndLocation(...),
        EmergencyPolicy::default(),
    )
    .await?;
```

---

# 181. Authority API

```rust
authority
    .broadcast_alert(scope, alert)
    .await?;
```

only available when current identity has authority role.

---

# 182. Status API

```rust
let status = emergency.status(emergency_id).await?;
```

Returns:

```text
stored
relayed
gateway
delivered
acknowledged
cancelled
expired
```

---

# 183. Initial Production Scope

Implement first:

```text
Routine/Important/Urgent/Critical/AuthorityCritical
priority authorization
local critical reserve
SOS
location
delivery/user ACK
cancel
DTN replication
direct + DTN redundancy
thumbnail-first attachment
authority-signed alerts
expiry/dedup
resource/battery integration
```

Defer initially:

```text
complex geographic alert routing
public anonymous large-scale broadcast
ML urgency classification
automatic medical diagnosis
```

---

# 184. Implementation Phases

## Phase 1 — Priority Model

```text
classes
authorization
effective priority
```

## Phase 2 — SOS

```text
durable create
send
ACK
cancel
```

## Phase 3 — Resource Reservation

```text
queue
storage
bandwidth
critical reserve
```

## Phase 4 — DTN / Multipath

```text
replication
redundant delivery
gateway
```

## Phase 5 — Attachments

```text
thumbnail
voice
full media policy
```

## Phase 6 — Authority Alerts

```text
signed issuer
scope
update
cancel
all-clear
```

## Phase 7 — Runtime/UI

```text
daemon
notifications
status
```

## Phase 8 — Hardening

```text
fuzz
abuse
crash
battery
partition
authority replay
```

---

# 185. Definition of Done

Part 17 is complete when:

- emergency priority classes are explicit and typed
- remote peers cannot self-grant AuthorityCritical
- local user SOS has reserved queue/storage capacity
- critical work can preempt bulk safely
- deadline/expiry affects scheduling
- SOS is durably committed before transmission
- direct, relay, multipath, and DTN paths can all carry the same EmergencyId
- duplicate delivery remains one logical emergency
- DTN replication budgets are bounded
- critical relay storage has reserved capacity
- small text/location outranks large emergency media
- thumbnail-first emergency media works
- battery/thermal pressure preserves critical communication
- verified authority alerts are signed, scoped, expiring, and replay-resistant
- unknown emergency reports are visibly unverified and strictly quota-limited
- cancellation/update/all-clear semantics are supported
- ACK can return through a different path/DTN
- daemon continues emergency delivery with UI closed
- fuzz, abuse, crash, partition, authority, battery, and multipath tests exist

---

# 186. Relationship to Earlier Parts

Part 17 builds on:

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
11 — Relay / Self-Hosted Infrastructure
12 — Multipath Networking
13 — Battery-Aware Scheduling
14 — Proximity Abstraction
15 — QR / NFC Bootstrap
16 — Daemon & Headless Runtime
```

It directly supports:

```text
18 — Network Diagnostics & Path Visualization
19 — C ABI / FFI
20 — Embedded Linux Node
22 — Third-Party Protocol Extensions
23 — External Interoperability Suite
24 — Plugin / Module Ecosystem
```

---

# 187. Final Architecture

```text
                  USER / AUTHORITY ACTION
                           │
                    Emergency Intent
                           │
                  Priority Authorization
                           │
                   Durable Commit
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
      Direct            Relay             Local/DTN
        │                  │                  │
        └────────────── Multipath ────────────┘
                           │
                    Destination(s)
                           │
                         ACK
```

Resource behavior:

```text
Critical arrives
     ↓
reserve queue slot
reserve storage
pause/throttle bulk
prefer fast/diverse routes
create bounded DTN copies
send text/location first
send media later
```

Authority broadcast:

```text
Trusted Authority
      ↓
Signed Alert
      ↓
Scope + Expiry
      ↓
Internet / LAN / DTN
      ↓
Receiver verifies
      ↓
Display as Verified Authority Alert
```

---

# 188. Final Principle

The emergency system should make this possible:

```text
A phone has 7% battery.
Internet is down.
A 4 GB file transfer is active.
DTN relay storage is nearly full.

The user presses SOS.

The system:
  persists the SOS,
  pauses bulk transfer,
  reserves storage,
  sends text and location first,
  searches nearby peers,
  creates bounded DTN copies,
  uses an Internet path if one appears,
  optionally sends over independent paths,
  and keeps retrying until delivered, expired, or cancelled.
```

At the same time:

```text
an unknown attacker cannot simply mark arbitrary traffic
as "critical" and consume those protected resources.
```

That combination—

```text
priority
+
authorization
+
reserved capacity
+
bounded redundancy
+
graceful degradation
```

—is what makes emergency support trustworthy rather than merely high-priority messaging.
