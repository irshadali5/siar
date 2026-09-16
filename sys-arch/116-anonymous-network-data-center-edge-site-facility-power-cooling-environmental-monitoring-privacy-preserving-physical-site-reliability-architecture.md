# Core System Architecture Part 116 — Anonymous Network Data Center, Edge Site, Facility Power, Cooling, Environmental Monitoring & Privacy-Preserving Physical Site Reliability Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 116  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 50, 62–63, 70–71, 100–115

**Primary purpose:** define SIAR's physical-site reliability architecture for data centers, edge facilities, racks, power distribution, UPS/generator systems, cooling, environmental sensors, maintenance, site failover, fault-domain modeling, physical-site SLOs, disaster integration, and privacy-preserving facility observability.

---

# 1. Purpose

Physical infrastructure can fail even when software is healthy.

A resilient platform must understand:

```text
Which sites exist?
Which racks and power feeds serve each asset?
Which cooling domains can fail together?
Which edge sites lack redundant utility power?
Which sensors are trustworthy?
When does thermal or power degradation require workload drain?
What happens during generator failure?
Which facilities can continue independently?
```

The governing principle is:

> **SIAR site reliability should model power, cooling, environmental conditions, rack placement, maintenance, and physical failure domains explicitly—without turning facility telemetry into worker tracking or precise user-location infrastructure.**

---

# 2. Architectural Position

```text
          PHYSICAL SITE
               │
      ┌────────┼────────┐
      │        │        │
    Power    Cooling  Environment
      │        │        │
      └────────┼────────┘
               ▼
         Facility State
               │
        ┌──────┼──────┐
        │      │      │
      Safe   Degrade Emergency
        │      │      │
        └──────┼──────┘
               ▼
      Drain / Failover / Recover
```

---

# 3. Core Separation

Keep distinct:

```text
site
room
rack
power domain
cooling domain
environmental zone
hardware asset
human operator
```

---

# 4. Non-Goals

Part 116 does not create:

```text
employee location tracking
continuous camera analytics
biometric workforce monitoring
user geolocation
unbounded sensor telemetry retention
```

---

# 5. Site Identity

```rust
pub struct PhysicalSiteId(pub [u8; 16]);
```

Opaque canonical identity.

---

# 6. Site Type

```rust
pub enum PhysicalSiteType {
    DataCenter,
    Colocation,
    EdgeSite,
    OfficeLab,
    RecoverySite,
    AirGappedSite,
    EmbeddedRemoteSite,
}
```

---

# 7. Site Record

```rust
pub struct PhysicalSite {
    pub id: PhysicalSiteId,
    pub site_type: PhysicalSiteType,
    pub region: RegionId,
    pub lifecycle: SiteLifecycleState,
    pub resilience: SiteResilienceClass,
}
```

---

# 8. Site Lifecycle

```rust
pub enum SiteLifecycleState {
    Planned,
    Commissioning,
    Active,
    Degraded,
    Maintenance,
    Evacuating,
    Retired,
}
```

---

# 9. Hard Rule

A retired site cannot silently return to Active without requalification.

---

# 10. Site Resilience Class

```rust
pub enum SiteResilienceClass {
    BestEffort,
    Standard,
    HighAvailability,
    MissionCritical,
}
```

---

# 11. No Marketing Mapping

Do not equate arbitrary facility labels with guaranteed resilience.

---

# 12. Hard rule.

---

# 13. Room / Hall / Zone

```rust
pub struct FacilityZoneId(pub [u8; 16]);
```

Zones may represent:

```text
room
hall
cage
pod
edge enclosure
```

---

# 14. Rack Identity

```rust
pub struct RackId(pub [u8; 16]);
```

---

# 15. Rack Record

```rust
pub struct RackRecord {
    pub id: RackId,
    pub site: PhysicalSiteId,
    pub zone: FacilityZoneId,
    pub power_domain: PowerDomainId,
    pub cooling_domain: CoolingDomainId,
}
```

---

# 16. Hard Rule

Rack placement is infrastructure topology, not personnel data.

---

# 17. Power Architecture

Power should be modeled as an explicit graph.

---

# 18. Power Components

```rust
pub enum PowerComponentType {
    UtilityFeed,
    Transformer,
    Switchgear,
    Ups,
    BatteryBank,
    Generator,
    Ats,
    Pdu,
    RackPdu,
}
```

---

# 19. Power Asset

```rust
pub struct PowerAsset {
    pub id: PowerAssetId,
    pub component_type: PowerComponentType,
    pub site: PhysicalSiteId,
}
```

---

# 20. Power Domain

```rust
pub struct PowerDomainId(pub [u8; 16]);
```

Represents components likely to fail together.

---

# 21. Power Redundancy

```rust
pub enum PowerRedundancy {
    None,
    N,
    NPlusOne,
    TwoN,
    TwoNPlusOne,
}
```

---

# 22. Hard Rule

Redundant feeds must be traced to independent upstream paths, not merely two rack PDUs.

---

# 23. Utility Feed

Track provider/source independently where known.

---

# 24. Hard rule.

---

# 25. UPS

UPS provides:

```text
ride-through
power conditioning
short-term continuity
```

---

# 26. UPS State

```rust
pub enum UpsState {
    Online,
    OnBattery,
    Bypass,
    Degraded,
    Failed,
    Unknown,
}
```

---

# 27. Unknown ≠ Online

Hard rule.

---

# 28. Battery Runtime

```rust
pub struct BatteryRuntimeEstimate {
    pub remaining: Duration,
    pub confidence: EstimateConfidence,
}
```

---

# 29. No False Precision

Hard rule.

---

# 30. Generator

Generator covers longer utility outages.

---

# 31. Generator State

```rust
pub enum GeneratorState {
    Ready,
    Starting,
    Running,
    Degraded,
    Failed,
    Unknown,
}
```

---

# 32. Fuel State

```rust
pub struct FuelState {
    pub estimated_runtime: Duration,
    pub confidence: EstimateConfidence,
}
```

---

# 33. Hard Rule

Fuel estimates are planning evidence, not guaranteed runtime.

---

# 34. ATS / Transfer

Automatic Transfer Switch is explicit dependency.

---

# 35. Transfer Failure

Can defeat otherwise healthy generator/UPS design.

---

# 36. Hard rule.

---

# 37. PDU Capacity

Track:

```text
rated power
current draw
headroom
phase balance
```

---

# 38. Power Load

```rust
pub struct PowerLoad {
    pub watts: FixedPoint,
    pub headroom_percent: FixedPoint,
}
```

---

# 39. Hard Rule

Do not plan rack capacity from nameplate power alone.

---

# 40. Power Failure Domains

Examples:

```text
single utility feed
UPS bank
generator set
ATS
PDU chain
rack PDU
```

---

# 41. Part 110 integration.

---

# 42. Cooling Architecture

Cooling is also a fault-domain graph.

---

# 43. Cooling Components

```rust
pub enum CoolingComponentType {
    Chiller,
    CoolingTower,
    Crac,
    Crah,
    Pump,
    FanWall,
    InRowCooler,
    HeatExchanger,
}
```

---

# 44. Cooling Domain

```rust
pub struct CoolingDomainId(pub [u8; 16]);
```

---

# 45. Cooling Redundancy

```rust
pub enum CoolingRedundancy {
    None,
    N,
    NPlusOne,
    TwoN,
}
```

---

# 46. Hard Rule

Multiple coolers sharing one chilled-water loop are not independent.

---

# 47. Thermal Zone

```rust
pub struct ThermalZoneId(pub [u8; 16]);
```

---

# 48. Environmental Sensors

Sensor types:

```rust
pub enum EnvironmentalSensorType {
    Temperature,
    Humidity,
    Smoke,
    WaterLeak,
    Airflow,
    DifferentialPressure,
    PowerQuality,
}
```

---

# 49. Sensor Identity

```rust
pub struct SensorId(pub [u8; 16]);
```

---

# 50. Sensor Placement

At coarse infrastructure positions:

```text
site
room
rack
thermal zone
```

---

# 51. No Personnel Association

Hard rule.

---

# 52. Sensor Reading

```rust
pub struct EnvironmentalReading {
    pub sensor: SensorId,
    pub kind: EnvironmentalSensorType,
    pub value: FixedPoint,
    pub observed_at: CoarseTimestamp,
    pub quality: SensorQuality,
}
```

---

# 53. Sensor Quality

```rust
pub enum SensorQuality {
    Good,
    Suspect,
    Stale,
    Failed,
    Unknown,
}
```

---

# 54. Unknown/Stale ≠ Good

Hard rule.

---

# 55. Calibration

Sensors require calibration lifecycle.

---

# 56. Calibration State

```rust
pub enum CalibrationState {
    Valid,
    DueSoon,
    Expired,
    Unknown,
}
```

---

# 57. Hard Rule

Expired calibration lowers trust in environmental decisions.

---

# 58. Temperature Policy

```rust
pub struct TemperaturePolicy {
    pub warning_high: FixedPoint,
    pub critical_high: FixedPoint,
    pub warning_low: FixedPoint,
    pub critical_low: FixedPoint,
}
```

---

# 59. Humidity Policy

```rust
pub struct HumidityPolicy {
    pub warning_low: FixedPoint,
    pub warning_high: FixedPoint,
    pub critical_low: FixedPoint,
    pub critical_high: FixedPoint,
}
```

---

# 60. Thresholds Are Site/Hardware-Class Specific

Hard rule.

---

# 61. Thermal State

```rust
pub enum ThermalState {
    Normal,
    Elevated,
    Critical,
    Recovering,
    Unknown,
}
```

---

# 62. No Critical→Normal Direct

Prefer recovery hysteresis.

---

# 63. Hard rule.

---

# 64. Cooling Failure Response

Sequence:

```text
detect
confirm
reduce noncritical load
drain affected racks
fail over
shut down safely if required
```

---

# 65. Hard rule.

---

# 66. Power Failure Response

Sequence:

```text
utility lost
UPS ride-through
generator start
ATS transfer
monitor runtime
shed noncritical load
drain/failover before battery/fuel exhaustion
```

---

# 67. Hard rule.

---

# 68. Load Shedding at Facility Layer

Order:

```text
test/dev
batch
analytics
background
optional services
```

Preserve:

```text
security/control
core messaging
emergency services
recovery systems
```

---

# 69. Hard rule.

---

# 70. Graceful Shutdown

If power exhaustion predicted:

```text
stop new noncritical admission
drain workloads
checkpoint state
shutdown orderly
```

---

# 71. No Hard Power-Off As Planned Baseline

Hard rule.

---

# 72. Environmental Alert Classes

```rust
pub enum FacilityAlertClass {
    PowerWarning,
    PowerCritical,
    CoolingWarning,
    CoolingCritical,
    TemperatureCritical,
    HumidityCritical,
    WaterLeak,
    SmokeDetected,
    SensorFailure,
}
```

---

# 73. Alert Routing

To site/fleet/on-call roles.

---

# 74. No User Alerts

Hard rule.

---

# 75. Facility State

```rust
pub enum FacilityOperationalState {
    Normal,
    DegradedPower,
    DegradedCooling,
    EnvironmentalRisk,
    Emergency,
    Evacuating,
    Offline,
}
```

---

# 76. Hard Rule

Facility state can affect workload admission/placement.

---

# 77. Admission Integration

Part 101/105.

Example:

```text
DegradedCooling → no new high-power workloads
```

---

# 78. Hard rule.

---

# 79. Edge Site Architecture

Edge sites may have weaker facilities.

---

# 80. Edge Characteristics

Examples:

```text
single utility feed
small UPS
limited cooling
intermittent backhaul
no onsite staff
```

---

# 81. Hard Rule

Edge-site SLO/resilience claims reflect actual infrastructure.

---

# 82. Edge Site Policy

```rust
pub struct EdgeSitePolicy {
    pub max_offline_duration: Duration,
    pub local_autonomy: LocalAutonomyClass,
    pub power_reserve_required: Duration,
}
```

---

# 83. Local Autonomy

```rust
pub enum LocalAutonomyClass {
    Minimal,
    Limited,
    Extended,
}
```

---

# 84. Hard rule.

---

# 85. Offline Edge Operation

Can continue local approved services.

---

# 86. No policy relaxation merely because WAN is unavailable.

---

# 87. Hard rule.

---

# 88. Edge Buffering

Store-carry-forward where applicable.

---

# 89. Capacity bounded.

---

# 90. Hard rule.

---

# 91. Remote Site Maintenance

May require:

```text
signed offline bundle
local technician
remote supervised session
```

---

# 92. No permanent unmanaged remote access.

---

# 93. Hard rule.

---

# 94. Environmental Autonomy

Local site controller can execute pre-approved safety actions:

```text
fan boost
load shed
graceful shutdown
```

---

# 95. Cannot alter security/privacy policy.

---

# 96. Hard rule.

---

# 97. Site Controller

```rust
pub trait FacilityController {
    fn state(
        &self,
        site: PhysicalSiteId,
    ) -> Result<FacilityOperationalState, FacilityError>;

    fn execute_safety_action(
        &self,
        action: FacilitySafetyAction,
    ) -> Result<FacilityActionReceipt, FacilityError>;
}
```

---

# 98. Safety Action

```rust
pub enum FacilitySafetyAction {
    ReduceOptionalLoad,
    DrainRack(RackId),
    DrainSite(PhysicalSiteId),
    GracefulShutdown,
}
```

---

# 99. No Generic Remote Shell

Hard rule.

---

# 100. Physical Site Fault Domains

Part 110 integration.

Domains:

```text
utility feed
generator
UPS bank
cooling loop
room/hall
rack
carrier entrance
site
```

---

# 101. Hard rule.

---

# 102. Network Entrance Diversity

Multiple carriers may share same conduit.

---

# 103. Model common physical path where known.

---

# 104. Hard rule.

---

# 105. Water / Fire Risk

Environmental hazards can affect whole site/zone.

---

# 106. Include in scenario modeling.

---

# 107. Hard rule.

---

# 108. Flood/Leak Detection

Water sensors near:

```text
cooling pipes
raised floor
UPS/battery area
```

---

# 109. Infrastructure-only.

---

# 110. Hard rule.

---

# 111. Smoke/Fire Detection

Facility system authoritative.

---

# 112. SIAR consumes coarse alarm state.

---

# 113. No camera/person analytics.

---

# 114. Hard rule.

---

# 115. Physical Disaster Scenarios

Examples:

```text
utility outage
generator failure
cooling-loop loss
rack overheating
flood
fire
site evacuation
```

---

# 116. Hard rule.

---

# 117. Disaster Simulation

Part 100/110.

---

# 118. Facility Scenario

```rust
pub enum FacilityFailureScenario {
    UtilityLoss,
    UpsFailure,
    GeneratorFailure,
    CoolingDomainFailure,
    RackPowerLoss,
    SiteIsolation,
    EnvironmentalEmergency,
}
```

---

# 119. Test resilience.

---

# 120. Hard rule.

---

# 121. Facility Capacity

Track:

```text
power capacity
cooling capacity
rack slots
network ports
```

---

# 122. Hard rule.

---

# 123. Facility Headroom

```rust
pub struct FacilityHeadroom {
    pub power_percent: FixedPoint,
    pub cooling_percent: FixedPoint,
    pub rack_space_percent: FixedPoint,
}
```

---

# 124. Headroom Is Not Infinite

Hard rule.

---

# 125. Capacity Forecast Integration

Part 111.

Forecast:

```text
power demand
cooling demand
rack growth
```

---

# 126. Hard rule.

---

# 127. Sustainability Integration

Part 114.

Site can include:

```text
PUE
renewable fraction
cooling efficiency
```

---

# 128. But sustainability does not override resilience.

---

# 129. Hard rule.

---

# 130. Power Usage Effectiveness

If used, clearly scope measurement.

---

# 131. No fake precision.

---

# 132. Hard rule.

---

# 133. Environmental Efficiency

Metrics:

```text
power usage
cooling overhead
idle rack power
temperature stability
```

---

# 134. No worker metrics.

---

# 135. Hard rule.

---

# 136. Facility Maintenance

Classes:

```rust
pub enum FacilityMaintenanceClass {
    Electrical,
    Generator,
    UpsBattery,
    Cooling,
    FireSuppression,
    SensorCalibration,
    NetworkPlant,
}
```

---

# 137. Maintenance Window

```rust
pub struct FacilityMaintenanceWindow {
    pub site: PhysicalSiteId,
    pub class: FacilityMaintenanceClass,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
}
```

---

# 138. Redundancy-Aware

Do not take both A/B power paths down together.

---

# 139. Hard rule.

---

# 140. UPS Battery Maintenance

Track:

```text
age
capacity test
replacement due
```

---

# 141. Hard rule.

---

# 142. Generator Exercise

Periodic start/load test.

---

# 143. Actual failover drills where safe.

---

# 144. Hard rule.

---

# 145. Cooling Maintenance

Use redundancy plan.

---

# 146. No maintenance that leaves critical room without cooling reserve.

---

# 147. Hard rule.

---

# 148. Sensor Maintenance

Calibration and replacement.

---

# 149. Sensor outage may reduce site confidence.

---

# 150. Hard rule.

---

# 151. Facility Maintenance Planner

```rust
pub trait FacilityMaintenancePlanner {
    fn plan(
        &self,
        request: FacilityMaintenanceRequest,
    ) -> Result<FacilityMaintenancePlan, FacilityError>;
}
```

---

# 152. Inputs

```text
redundancy
current facility state
hardware load
capacity headroom
weather/provider risk if available
site access
```

---

# 153. No user activity.

---

# 154. Hard rule.

---

# 155. Physical Access

Part 115.

Facility work uses scoped/time-bounded access.

---

# 156. No continuous movement tracking.

---

# 157. Hard rule.

---

# 158. Site Evacuation

Operational evacuation may occur due:

```text
fire
flood
structural issue
extended power failure
```

---

# 159. System action:

```text
freeze new placement
drain eligible workloads
fail over
mark site unavailable
```

---

# 160. Hard rule.

---

# 161. Evacuation State

```rust
pub enum SiteEvacuationState {
    NotRequired,
    Preparing,
    Draining,
    Offline,
    Recovering,
    ReturnedToService,
}
```

---

# 162. Return To Service

Requires verification.

---

# 163. Hard rule.

---

# 164. Site Recovery

Checks:

```text
power stable
cooling stable
network stable
sensors valid
hardware re-attested if needed
```

---

# 165. No immediate full re-admission.

---

# 166. Hard rule.

---

# 167. Workload Re-Admission

Canary/ramp back.

---

# 168. Part 107/108.

---

# 169. Hard rule.

---

# 170. Facility SLOs

Examples:

```text
power availability
cooling availability
environmental sensor coverage
generator readiness
UPS battery readiness
```

---

# 171. No One Composite Score

Hard rule.

---

# 172. Facility SLI

```rust
pub enum FacilitySliKind {
    PowerAvailable,
    CoolingAvailable,
    TemperatureWithinRange,
    HumidityWithinRange,
    GeneratorReady,
    UpsReady,
    SensorCoverage,
}
```

---

# 173. Facility SLO

```rust
pub struct FacilitySlo {
    pub site: PhysicalSiteId,
    pub kind: FacilitySliKind,
    pub target: SloTarget,
}
```

---

# 174. Part 109 integration.

---

# 175. Error Budget

May apply to facility service.

---

# 176. Hard security hazards are not budgetable.

---

# 177. Hard rule.

---

# 178. Environmental Monitoring Privacy

Telemetry dimensions:

```text
site
zone
rack
sensor
```

---

# 179. Not:

```text
person
employee badge
user device
```

---

# 180. Hard rule.

---

# 181. Sensor Sampling

Use rate appropriate to hazard.

---

# 182. No need to retain high-frequency history forever.

---

# 183. Hard rule.

---

# 184. Sensor Retention

Keep:

```text
recent high-resolution
older rolled-up
incident evidence as needed
```

---

# 185. Hard rule.

---

# 186. Sensor Integrity

Sensors can fail/spoof.

---

# 187. Critical decisions may require corroboration.

---

# 188. Hard rule.

---

# 189. Multi-Sensor Confirmation

Example:

```text
temperature high + airflow low
```

raises confidence.

---

# 190. Good.

---

# 191. No Single-Sensor Trust For Critical Shutdown Where Avoidable

Hard rule.

---

# 192. Facility Control Security

BMS/DCIM integration is high-risk.

---

# 193. Use:

```text
read-only telemetry adapters where possible
typed safety commands
segmented management network
```

---

# 194. Hard rule.

---

# 195. No Generic BMS Remote Control

Hard rule.

---

# 196. Control Plane Isolation

Facility controller separate from user data plane.

---

# 197. Hard rule.

---

# 198. Facility Secrets

Credentials for BMS/UPS/generator APIs stored via Part 80 secret broker.

---

# 199. No hardcoded passwords.

---

# 200. Hard rule.

---

# 201. Vendor Appliance Integration

Treat vendor APIs as untrusted external systems.

---

# 202. Validate data.

---

# 203. Hard rule.

---

# 204. Monitoring Loss

If DCIM/BMS feed lost:

```text
state confidence decreases
```

---

# 205. Unknown ≠ Normal.

---

# 206. Hard rule.

---

# 207. Site Inventory Integration

Part 106.

Nodes:

```text
site
zone
rack
PDU
UPS
generator
cooling unit
sensor
```

---

# 208. Typed edges.

---

# 209. Hard rule.

---

# 210. Desired-State Integration

Part 105.

Facility policies:

```text
thresholds
maintenance schedules
safety actions
```

signed/versioned.

---

# 211. Hard rule.

---

# 212. Hardware Fleet Integration

Part 115.

Hardware placement links to rack/site/power/cooling domains.

---

# 213. Hard rule.

---

# 214. Resilience Integration

Part 110.

Facility failures are correlated fault domains.

---

# 215. Hard rule.

---

# 216. Capacity Integration

Part 101.

Site power/cooling can limit compute admission.

---

# 217. Hard rule.

---

# 218. Forecast Integration

Part 111.

Future power/cooling capacity forecast.

---

# 219. Hard rule.

---

# 220. Performance Integration

Part 112.

Thermal throttling may degrade performance.

---

# 221. Must be visible.

---

# 222. Hard rule.

---

# 223. Efficiency Integration

Part 113.

Idle/power consolidation may reduce facility load.

---

# 224. Resilience still first.

---

# 225. Hard rule.

---

# 226. Sustainability Integration

Part 114.

Cooling efficiency/renewable power informs environmental planning.

---

# 227. Hard rule.

---

# 228. Geographic Governance

Part 103.

Site belongs to approved region/jurisdiction.

---

# 229. Hard rule.

---

# 230. Organizational Governance

Part 104.

Site/facility owner and on-call responsibilities explicit.

---

# 231. Hard rule.

---

# 232. Change Integration

Part 107.

Facility changes are governed changes.

---

# 233. Hard rule.

---

# 234. Release Integration

Part 108.

Large release may require site capacity/headroom readiness.

---

# 235. Hard rule.

---

# 236. DR Integration

Part 100.

Site loss scenario maps to failover.

---

# 237. Hard rule.

---

# 238. Incident Integration

Part 96.

Power/cooling/environmental emergencies can trigger incident.

---

# 239. Hard rule.

---

# 240. SOC Integration

Physical tamper/environment anomalies are security signals where relevant.

---

# 241. No person tracking.

---

# 242. Hard rule.

---

# 243. Compliance Integration

Part 95.

Controls can require:

```text
redundant power
generator test
sensor calibration
environmental thresholds
physical access scope
```

---

# 244. Good.

---

# 245. Facility API

```rust
pub trait FacilityService {
    fn site(
        &self,
        id: PhysicalSiteId,
    ) -> Result<PhysicalSite, FacilityError>;

    fn state(
        &self,
        id: PhysicalSiteId,
    ) -> Result<FacilityOperationalState, FacilityError>;
}
```

---

# 246. Power Service

```rust
pub trait FacilityPowerService {
    fn status(
        &self,
        site: PhysicalSiteId,
    ) -> Result<PowerStatusSnapshot, FacilityError>;
}
```

---

# 247. Environmental Service

```rust
pub trait EnvironmentalMonitoringService {
    fn snapshot(
        &self,
        site: PhysicalSiteId,
    ) -> Result<EnvironmentalSnapshot, FacilityError>;
}
```

---

# 248. Maintenance Service

```rust
pub trait FacilityMaintenanceService {
    fn plan(
        &self,
        request: FacilityMaintenanceRequest,
    ) -> Result<FacilityMaintenancePlan, FacilityError>;
}
```

---

# 249. No General BMS Shell/API Passthrough

Hard rule.

---

# 250. Error Taxonomy

```rust
pub enum FacilityError {
    SiteUnknown,
    PowerStateUnknown,
    CoolingStateUnknown,
    SensorDataStale,
    CapacityInsufficient,
    RedundancyInsufficient,
    MaintenanceUnsafe,
    EvacuationRequired,
    ControlActionDenied,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 251. Observability

Safe metrics:

```text
site state
UPS state
generator state
power headroom
cooling headroom
temperature/humidity range
sensor freshness
```

---

# 252. Forbidden:

```text
employee movement
badge history
user devices
private content
```

---

# 253. Hard rule.

---

# 254. Facility SLO Examples

```text
Tier0 sites have current UPS/generator readiness
critical racks have dual independent power where policy requires
environmental sensors fresh within target interval
no maintenance below facility redundancy floor
```

---

# 255. Security SLO

```text
0 generic unauthenticated BMS control paths
0 facility management secrets embedded in code
```

---

# 256. Privacy SLO

```text
0 personnel movement profiling
0 facility telemetry linked to users
```

---

# 257. Failure Modes

```text
utility loss
generator no-start
UPS battery degradation
cooling failure
sensor failure
network plant loss
```

---

# 258. Utility Loss

Follow UPS→generator→load management.

---

# 259. Hard rule.

---

# 260. Generator No-Start

Estimate remaining battery runtime.

Drain before exhaustion.

---

# 261. Hard rule.

---

# 262. UPS Battery Degradation

Capacity test triggers maintenance.

---

# 263. Hard rule.

---

# 264. Cooling Failure

Throttle/load-shed/drain.

---

# 265. Hard rule.

---

# 266. Sensor Failure

State becomes Unknown for affected domain.

---

# 267. No green default.

---

# 268. Hard rule.

---

# 269. Network Plant Loss

Edge/local continuity where allowed.

---

# 270. No privacy downgrade.

---

# 271. Hard rule.

---

# 272. Testing

Need facility-reliability testkit.

---

# 273. Test Scenarios

```text
utility outage
generator failure
cooling domain loss
rack overheating
edge site isolation
```

---

# 274. Power Test

UPS runtime/failover sequence correct.

---

# 275. Generator Test

No-start triggers drain.

---

# 276. Redundancy Test

A/B power paths sharing common upstream fail validation.

---

# 277. Cooling Test

Shared chilled loop modeled as common fault.

---

# 278. Sensor Test

Stale sensor != Normal.

---

# 279. Calibration Test

Expired sensor lowers confidence.

---

# 280. Drain Test

Thermal critical state prevents new high-power workload admission.

---

# 281. Edge Test

WAN isolation does not relax security/privacy policy.

---

# 282. Privacy Test

No worker location history generated.

---

# 283. DR Test

Site loss maps to compliant failover.

---

# 284. Capacity Test

Power/cooling headroom considered before scale-out.

---

# 285. Maintenance Test

Electrical maintenance cannot remove redundant feed simultaneously.

---

# 286. Facility Controller Test

Only typed safety actions allowed.

---

# 287. Fuzzing

Fuzz:

```text
power topology
cooling topology
sensor readings
facility policies
maintenance plans
```

---

# 288. Property Tests

Properties:

```text
critical site state can never be Normal when required power/cooling evidence is Unknown
facility action can never bypass security/privacy/residency constraints
maintenance plan can never intentionally violate required facility redundancy
worker identity can never become a valid environmental telemetry dimension
```

---

# 289. Formal Verification Targets

Strong candidates:

```text
power failover state machine
cooling failure response
site evacuation state machine
maintenance redundancy
```

---

# 290. Kani Candidate

threshold/failover/redundancy invariants.

---

# 291. TLA+ Candidate

normal → utility loss → UPS → generator → drain/recovery.

---

# 292. Loom Candidate

concurrent sensor alarm + facility state transition + workload admission.

---

# 293. Performance

Environmental monitoring is off request path.

---

# 294. Critical facility state cached locally.

---

# 295. No request-path dependency on external BMS cloud.

---

# 296. Hard rule.

---

# 297. Storage

Separate:

```text
site inventory
power topology
cooling topology
sensor definitions
aggregate readings
maintenance plans
facility incidents
```

---

# 298. No worker tracking store.

---

# 299. Hard rule.

---

# 300. Partitioning

By:

```text
site
zone
rack
power domain
cooling domain
sensor type
```

---

# 301. No user/person partition.

---

# 302. Hard rule.

---

# 303. Crate Layout

Recommended:

```text
crates/
├── siar-facility-core/
├── siar-power-topology/
├── siar-cooling-topology/
├── siar-environmental-monitoring/
├── siar-site-controller/
├── siar-facility-maintenance/
├── siar-site-evacuation/
├── siar-facility-slo/
├── siar-facility-observability/
└── siar-facility-testkit/
```

---

# 304. `siar-facility-core`

Owns:

```text
PhysicalSiteId
PhysicalSiteType
FacilityOperationalState
errors
```

---

# 305. `siar-power-topology`

Utility/UPS/generator/ATS/PDU topology and state.

---

# 306. `siar-cooling-topology`

Cooling equipment/domains/redundancy.

---

# 307. `siar-environmental-monitoring`

Temperature/humidity/smoke/leak/airflow sensors.

---

# 308. `siar-site-controller`

Typed safety actions only.

---

# 309. `siar-facility-maintenance`

Electrical/cooling/sensor maintenance planning.

---

# 310. `siar-site-evacuation`

Drain/offline/recovery state machine.

---

# 311. `siar-facility-slo`

Power/cooling/environmental objectives.

---

# 312. `siar-facility-observability`

Aggregate site health only.

---

# 313. `siar-facility-testkit`

power/cooling/sensor/privacy tests.

---

# 314. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Facility reliability is modeled through explicit site, rack, power, cooling, network-plant, and environmental fault domains; redundant labels alone never prove independence.
2. Facility telemetry is scoped to infrastructure sites, zones, racks, and sensors and never tracks users, employee movement, badge histories, or private content.
3. Unknown, stale, or uncalibrated critical sensor data can never be interpreted as normal facility state.
4. Utility, UPS, generator, ATS, PDU, cooling, and carrier dependencies are modeled explicitly so shared upstream infrastructure cannot masquerade as redundant paths.
5. Facility safety actions are typed, bounded, and policy-controlled; no generic BMS, UPS, or remote-control shell is exposed through the platform.
6. Environmental or power degradation can restrict workload admission, trigger load shedding, or drain assets, but cannot disable encryption, authorization, anonymity, residency, or other hard security/privacy controls.
7. Facility maintenance must preserve declared power/cooling/fault-domain redundancy and cannot take correlated redundant paths offline together unless an explicit emergency degraded-mode policy authorizes it.
8. Edge/offline sites retain the same security/privacy policy during WAN isolation; loss of connectivity never justifies weaker identity, crypto, or routing rules.
9. Site evacuation and recovery are explicit state machines; return to service requires stable power, cooling, environmental sensing, network, and hardware trust verification rather than a simple "power restored" signal.
10. Facility observability and historical data are retained only at the technical resolution necessary for reliability, maintenance, incidents, and assurance and cannot become a workforce-surveillance system.
11. Capacity, sustainability, cost, or efficiency optimization cannot consume protected facility headroom or reduce required site redundancy below policy without a governed change.
12. Facility reliability integrates with hardware fleet management, inventory, desired state, capacity, forecasting, SLOs, resilience, DR, change/release governance, sustainability, security operations, and compliance without creating an alternate physical control plane around platform governance.
```

---

# 315. Initial Production Scope

Implement first:

```text
typed PhysicalSiteId/PhysicalSiteType
site/rack/zone inventory
power topology
power domains
UPS/generator/ATS/PDU state
cooling topology/domains
temperature/humidity/leak/smoke sensors
sensor freshness/calibration
facility operational state
power/cooling headroom
typed facility safety actions
edge-site policy
site evacuation state machine
facility maintenance planner
facility SLOs
hardware-fleet links
resilience/capacity integration
privacy-safe dashboards
facility testkit
```

Then add:

```text
advanced BMS/DCIM adapters
power-quality analytics
generator fuel logistics
thermal digital twin
rack-level CFD-informed planning
weather/grid-risk adapters where appropriate
formal power/cooling redundancy verification
```

---

# 316. Definition of Done

Part 116 is complete when:

- sites/zones/racks are canonical assets
- power/cooling topologies are explicit
- shared upstream fault domains are modeled
- UPS/generator/ATS states are typed
- environmental sensors have freshness/calibration state
- Unknown sensor data cannot imply Normal
- facility headroom affects workload admission
- maintenance preserves site redundancy
- edge sites maintain security/privacy during isolation
- site evacuation/recovery are explicit state machines
- facility control exposes only typed safety actions
- no worker/user tracking exists
- facility/privacy/fuzz/formal tests are specified

---

# 317. Final Architecture

```text
                   PHYSICAL SITE
                        │
          ┌─────────────┼─────────────┐
          │             │             │
        POWER        COOLING      ENVIRONMENT
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                FACILITY STATE ENGINE
                        │
              ┌─────────┼─────────┐
              │         │         │
            NORMAL   DEGRADED   EMERGENCY
              │         │         │
              └─────────┼─────────┘
                        ▼
            ADMISSION / DRAIN / FAILOVER
                        │
                        ▼
                 RECOVERY / VERIFY
```

Physical-site reliability model:

```text
explicit power topology
+
explicit cooling domains
+
calibrated environmental sensing
+
headroom-aware admission
+
resilience-aware maintenance
+
typed emergency actions
+
site evacuation/recovery
+
privacy-safe facility telemetry
```

not:

```text
assume two PDUs mean redundant power, trust stale sensors, expose a generic BMS shell, and track workers to understand facility state
```

---

# 318. Final Principle

A reliable physical site is a modeled system of power, cooling, environment, and fault domains—not merely a building containing servers.

The correct model is:

```text
identify sites and racks
+
map power/cooling dependencies
+
monitor calibrated environmental signals
+
preserve headroom and redundancy
+
drain before unsafe thresholds
+
recover through verification
+
isolate facility control
+
never turn physical telemetry into people tracking
```

This architecture gives SIAR a privacy-preserving physical-site reliability foundation for data centers, colocation, edge facilities, power, UPS/generators, cooling, environmental monitoring, maintenance, evacuation, and recovery while preserving the anonymity, local-first, least-authority, resilience, sustainability, and anti-surveillance guarantees established across Parts 34–115.
