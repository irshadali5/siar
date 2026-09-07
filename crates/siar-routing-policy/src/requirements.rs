//! §6 "Delivery Requirements".

use serde::{Deserialize, Serialize};

use crate::metrics::{Bitrate, NetworkCost};
use crate::types::{DeliveryClass, Priority};

/// §6. Every field the spec lists, in the order it lists them, plus
/// two fields added this round for spec text that names a signal this
/// struct didn't yet carry: `nearby_session_explicit` is §52 "Wi-Fi
/// Direct/Aware"'s third threshold case ("explicit nearby session") —
/// the other two (large transfer, active call) are already derivable
/// from `min_bandwidth`/`class` without a new field, but "the caller
/// explicitly asked for a nearby session" has no existing signal to
/// reuse. `dtn_replication_budget` is §56 "DTN Routing Boundary"'s own
/// enumeration of what the routing engine (not the DTN subsystem)
/// decides — "DTN allowed? priority? expiry? **replication budget**?"
/// — the first three already exist (`allow_dtn`, `priority`,
/// `expiry_millis`); this is the one that didn't.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeliveryRequirements {
    pub class: DeliveryClass,
    pub priority: Priority,
    pub max_latency_millis: Option<u32>,
    pub min_bandwidth: Option<Bitrate>,
    pub durable: bool,
    pub allow_metered: bool,
    pub allow_relay: bool,
    pub allow_bluetooth: bool,
    pub allow_dtn: bool,
    pub allow_multipath: bool,
    pub expiry_millis: Option<u64>,
    pub max_cost: Option<NetworkCost>,
    pub nearby_session_explicit: bool,
    pub dtn_replication_budget: Option<u8>,
}

impl DeliveryRequirements {
    /// §7's own worked examples: `video call frame → Realtime`.
    /// Non-durable (a dropped frame isn't retried), Critical priority,
    /// tight latency, no DTN/relay/Bluetooth (§29 "Low-Latency Policy":
    /// "avoid DTN", "avoid BLE" — a realtime frame that arrives after a
    /// DTN hop is arriving too late to be useful at all).
    pub fn realtime_media() -> Self {
        Self {
            class: DeliveryClass::Realtime,
            priority: Priority::Critical,
            max_latency_millis: Some(150),
            min_bandwidth: None,
            durable: false,
            allow_metered: true,
            allow_relay: true,
            allow_bluetooth: false,
            allow_dtn: false,
            allow_multipath: false,
            expiry_millis: None,
            max_cost: None,
            // A call is exactly §52's "active call" threshold case —
            // justifies expensive setup (Wi-Fi Direct/Aware) on its
            // own without needing the explicit-nearby-session flag.
            nearby_session_explicit: false,
            dtn_replication_budget: None,
        }
    }

    /// §7: `text message → Interactive/Reliable`. Durable, normal
    /// priority, everything allowed — an ordinary message should be
    /// deliverable by whatever path can carry it.
    pub fn interactive_message() -> Self {
        Self {
            class: DeliveryClass::Reliable,
            priority: Priority::Normal,
            max_latency_millis: None,
            min_bandwidth: None,
            durable: true,
            allow_metered: true,
            allow_relay: true,
            allow_bluetooth: true,
            allow_dtn: true,
            allow_multipath: false,
            expiry_millis: None,
            max_cost: None,
            nearby_session_explicit: false,
            dtn_replication_budget: None,
        }
    }

    /// §7: `SOS → Reliable/DelayTolerant`. §33 "Emergency Policy":
    /// "allow mesh, allow DTN... ignore some cost preferences" —
    /// Critical priority, every transport allowed, no cost ceiling.
    pub fn emergency() -> Self {
        Self {
            class: DeliveryClass::DelayTolerant,
            priority: Priority::Critical,
            max_latency_millis: None,
            min_bandwidth: None,
            durable: true,
            allow_metered: true,
            allow_relay: true,
            allow_bluetooth: true,
            allow_dtn: true,
            allow_multipath: true,
            expiry_millis: None,
            max_cost: None,
            nearby_session_explicit: false,
            // §33/§56: emergency traffic should replicate widely
            // rather than trust a single DTN carrier — an explicit,
            // generous budget rather than "unlimited" (still bounded,
            // per this crate's general no-unbounded-anything posture;
            // see [`crate::retry`]'s own bounded-backoff reasoning).
            dtn_replication_budget: Some(8),
        }
    }
}
