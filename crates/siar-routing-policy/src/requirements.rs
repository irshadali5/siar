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

    /// §57's own "typing" example: "non-durable, expires quickly, no
    /// DTN." `Priority::Low` (never urgent enough to compete with real
    /// traffic) and `expiry_millis: Some(5_000)` transcribed directly
    /// from §57's own "typing after 5s" — see [`Self::has_expired`],
    /// which is precisely what makes that number actually enforced
    /// rather than decorative. [`crate::retry::RetryPolicy::no_retry`]'s
    /// own doc comment already quotes this same "should not retry" line
    /// from §38 — the two constructors are meant to be paired.
    pub fn typing_indicator() -> Self {
        Self {
            class: DeliveryClass::Interactive,
            priority: Priority::Low,
            max_latency_millis: None,
            min_bandwidth: None,
            durable: false,
            allow_metered: true,
            allow_relay: true,
            allow_bluetooth: true,
            allow_dtn: false,
            allow_multipath: false,
            expiry_millis: Some(5_000),
            max_cost: None,
            nearby_session_explicit: false,
            dtn_replication_budget: None,
        }
    }

    /// §57's own "file chunk" example: "resumable, bulk, DTN
    /// optional." `durable: true` is what "resumable" actually cashes
    /// out to in this crate's own vocabulary — a chunk that fails must
    /// be retriable, the same property §38 already grants
    /// [`Self::interactive_message`]'s ordinary messages. `allow_dtn:
    /// true` (permitted, not forced) is "optional" read the same way
    /// every other `allow_*` field already means "permitted"
    /// elsewhere in this struct. `allow_multipath: true` fits §75
    /// "Multipath Chunk Scheduler" 's own premise — file chunks are
    /// exactly the traffic multipath chunk scheduling exists for —
    /// though that scheduler itself remains out of this crate's scope
    /// (see this crate's own top doc comment).
    pub fn file_chunk() -> Self {
        Self {
            class: DeliveryClass::Bulk,
            priority: Priority::Low,
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
            dtn_replication_budget: None,
        }
    }

    /// §62 "Expiry-Aware Routing": "Routing should stop retrying after
    /// expiry" — `expiry_millis` has existed on this struct since
    /// §6/§7's own field list, but nothing in this crate actually
    /// compared it against a clock until now. `false` whenever
    /// `expiry_millis` is `None` (an operation with no expiry never
    /// expires, matching every other `Option`-shaped policy field in
    /// this crate defaulting to "unconstrained" rather than "already
    /// failed"). This crate has no clock of its own (see its top doc
    /// comment on scope) — `created_at_millis`/`now_millis` are both
    /// supplied by the caller.
    pub fn has_expired(&self, created_at_millis: u64, now_millis: u64) -> bool {
        match self.expiry_millis {
            Some(expiry) => now_millis.saturating_sub(created_at_millis) >= expiry,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_indicator_matches_57s_own_worked_example() {
        let req = DeliveryRequirements::typing_indicator();
        assert!(!req.durable);
        assert!(!req.allow_dtn);
        assert_eq!(req.expiry_millis, Some(5_000));
    }

    #[test]
    fn file_chunk_matches_57s_own_worked_example() {
        let req = DeliveryRequirements::file_chunk();
        assert!(req.durable); // "resumable"
        assert_eq!(req.class, DeliveryClass::Bulk);
        assert!(req.allow_dtn); // "optional" = permitted, not forced
    }

    #[test]
    fn no_expiry_never_expires() {
        let req = DeliveryRequirements::interactive_message();
        assert!(!req.has_expired(0, u64::MAX));
    }

    #[test]
    fn an_operation_past_its_expiry_window_has_expired() {
        let req = DeliveryRequirements::typing_indicator(); // 5_000ms expiry
        assert!(!req.has_expired(1_000, 5_999)); // 4_999ms elapsed
        assert!(req.has_expired(1_000, 6_000)); // exactly 5_000ms elapsed
        assert!(req.has_expired(1_000, 60_000)); // long past
    }
}
