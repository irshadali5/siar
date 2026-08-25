# 07 — Battery-Aware Scheduling & Emergency Mesh

> **Corresponding Specifications:** [`sys-arch/13-battery-aware-scheduling-architecture.md`](../sys-arch/13-battery-aware-scheduling-architecture.md), [`sys-arch/17-emergency-priority-classes-architecture.md`](../sys-arch/17-emergency-priority-classes-architecture.md), [`sys-arch/ui-ux-17-emergency-sos-offline-mesh-architecture.md`](../sys-arch/ui-ux-17-emergency-sos-offline-mesh-architecture.md)  
> **Key Crates:** [`crates/siar-emergency`](../crates/siar-emergency), [`crates/siar-routing-policy`](../crates/siar-routing-policy)

---

## 1. Five-Tier QoS Traffic Priority Engine

SIAR enforces strict quality-of-service traffic classification across all queues, buffers, and radio transmitters:

```
+-------------------------------------------------------------------------------+
| Tier   | Traffic Class             | Preemption | Drop Policy | Transports   |
+--------+---------------------------+------------+-------------+--------------+
| Tier 0 | Emergency SOS Broadcast   | Immediate  | Never Drop  | All Radios   |
| Tier 1 | Realtime Signaling / Voice| High       | Drop on TTL | Wi-Fi / LAN  |
| Tier 2 | Direct 1-on-1 Chat        | Medium     | DTN Spool   | Any Radio    |
| Tier 3 | Group MLS Epoch Commits   | Low        | DTN Spool   | Any Radio    |
| Tier 4 | Blob Sync & Background    | None       | Evict First | Unmetered Net|
+-------------------------------------------------------------------------------+
```

---

## 2. Battery-Aware Duty Cycling Modes

Mobile devices dynamically throttle radio scan intervals, discovery windows, and CPU wakeups according to remaining battery state of charge (SoC):

```mermaid
stateDiagram-v2
    [*] --> Normal: SoC > 50%
    Normal --> Conservative: 20% < SoC <= 50%
    Conservative --> Critical: 5% < SoC <= 20%
    Critical --> Survival: SoC <= 5%
    
    Normal: BLE scan 50% duty cycle\nWi-Fi Aware active\nInstant blob sync
    Conservative: BLE scan 10% duty cycle\nWi-Fi Aware on-demand\nDefer large blobs
    Critical: BLE scan 2% duty cycle (100ms every 5s)\nWi-Fi disabled\nText-only delivery
    Survival: Passive BLE beaconing only (1 burst/min)\nSOS alerts only\nZero background compute
```

---

## 3. Emergency SOS Protocol & Mesh Flooding

In disaster scenarios, search-and-rescue operations, or life-safety emergencies, the SOS engine triggers an autonomous epidemic broadcast across every active physical interface:

```rust
pub struct EmergencySosPayload {
    pub emergency_id: EmergencyId,
    pub sender_id: AccountId,
    pub timestamp: Timestamp,
    pub location: Option<GeoCoordinates>, // Latitude, Longitude, Altitude, Accuracy
    pub emergency_type: EmergencyType,    // Medical, Trapped, Fire, NaturalDisaster
    pub battery_remaining_pct: u8,
    pub text_message: Option<String>,
    pub hop_count: u8,
    pub signature: Signature,
}
```

```mermaid
sequenceDiagram
    autonumber
    actor Victim as Victim Node (Offline)
    participant Relay1 as Nearby Phone (Mule 1)
    participant Repeater as Solar Mesh Node
    participant Responder as Rescue Team / HQ

    Victim->>Victim: Trigger SOS (UI Button / Crash Sensor)
    Victim->>Relay1: Multi-Radio Blast (BLE Extended Adv + Wi-Fi Direct Probe)
    Relay1->>Relay1: Validate Ed25519 Signature & Pin to Priority 0 Queue
    Relay1->>Repeater: Forward over LoRa / Wi-Fi Mesh Hop
    Repeater->>Responder: High-Gain Long Range Uplink
    Responder-->>Victim: Cryptographic Acknowledgment & ETA Broadcast
```

### Invariants of Emergency SOS
1. **Preempts All Queues**: Emergency frames immediately jump to the head of all transmission rings.
2. **Exempt from Duty Cycles**: Devices in `Survival` mode still wake up to receive and relay SOS packets.
3. **No Centralized Key Verification**: Signed with standard Ed25519 identity, decryptable by any authorized responder or public emergency channel.
