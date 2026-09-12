//! §140 "Bandwidth Reservation", §141 "Traffic Shaping", §142
//! "Connection Admission", §143 "Thermal Awareness", §144 "Memory
//! Pressure" — grouped together as "resource pressure," the common
//! thread being: none of these decide *which path* to use (that's
//! [`crate::scoring`]/[`crate::plan`]'s job); all of them decide
//! *whether, and how much,* given a resource constraint that has
//! nothing to do with any one candidate's own quality.
//!
//! §140/§141 are explicitly forward-looking in the spec's own text
//! ("Future call + file coexistence") — [`BandwidthReservation`] and
//! [`TrafficShapingPolicy`] are the representation that future
//! integration needs, not a claim that call/file coexistence is fully
//! solved today. §142-144 are present-tense and get fuller treatment.
//!
//! None of these five sections' functions perform the actual resource
//! action (shaping traffic, closing a session, dropping a packet) —
//! this crate has no socket to shape, no session handle to close, no
//! packet buffer to drop from (same scope line this crate's own top
//! doc comment already draws for everything wire-related). Each
//! function computes the *decision*; a caller with the actual
//! resource in hand carries it out.

use serde::{Deserialize, Serialize};

use crate::metrics::Bitrate;
use crate::platform::{DeviceState, ThermalState};
use crate::setup::{static_setup_cost, SetupCost};
use crate::types::{DeliveryClass, Priority, TransportKind};

/// §140: "reserve bandwidth for realtime audio/video... Bulk transfer
/// yields." One field, not a duration/schedule — this crate has no
/// clock to track when a reservation started or should end (same "no
/// history" posture [`crate::explain::RouteMetricEvent`] and
/// [`crate::engine::health_after_outcome`] already take); a caller
/// creates one when a call starts and drops it when the call ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BandwidthReservation {
    pub reserved_for_realtime: Bitrate,
}

/// §140's own "Bulk transfer yields" — unconditional, not a partial
/// reduction: a `Bulk` operation yields entirely to any active
/// realtime reservation, the reservation's specific size doesn't
/// change that. Every other [`DeliveryClass`] is unaffected — §140
/// only names Bulk as the one that yields.
pub fn bulk_should_yield_to_reservation(
    class: DeliveryClass,
    reservation: Option<&BandwidthReservation>,
) -> bool {
    class == DeliveryClass::Bulk && reservation.is_some()
}

/// §141, transcribed with the spec's own two named caps: "Bulk max 5
/// Mbps while call active" and "Background max 1 Mbps." The first is
/// keyed on [`DeliveryClass::Bulk`] *and* `call_active` together —
/// the spec's own "while call active" qualifier, not an always-on
/// cap; the second is keyed on [`Priority::Background`] alone,
/// unconditionally, since the spec states it without a qualifier the
/// way it gives Bulk's cap one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrafficShapingPolicy {
    pub bulk_max_bitrate_during_call: Bitrate,
    pub background_max_bitrate: Bitrate,
}

/// Returns the cap that applies, if any. `None` means "no shaping
/// cap from this policy" — not "unlimited bandwidth," just that
/// nothing here restricts it; [`crate::scoring`]'s own
/// `min_bandwidth`/hard-constraint checks are a separate, unrelated
/// mechanism.
pub fn effective_traffic_cap(
    policy: &TrafficShapingPolicy,
    class: DeliveryClass,
    priority: Priority,
    call_active: bool,
) -> Option<Bitrate> {
    if class == DeliveryClass::Bulk && call_active {
        Some(policy.bulk_max_bitrate_during_call)
    } else if priority == Priority::Background {
        Some(policy.background_max_bitrate)
    } else {
        None
    }
}

/// §142, transcribed with the spec's own two named limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionAdmission {
    pub max_simultaneous_sessions: usize,
    pub max_expensive_radio_sessions: usize,
}

/// §142's own "routing may reject or defer low-priority new
/// connections" — read as "connections below `Critical` are subject
/// to the caps; `Critical` always gets through," the same emergency-
/// bypass shape §138's own opt-in override already established for a
/// different resource (relay). `new_connection_is_expensive_radio` is
/// a caller-supplied fact rather than this function inferring it from
/// a `TransportKind`, since [`static_setup_cost`] is exactly the
/// existing function that answers "is this expensive" — no need to
/// re-derive it here; see [`is_expensive_radio_transport`] if a
/// caller wants that derivation done for them.
pub fn admission_permitted(
    admission: &ConnectionAdmission,
    current_sessions: usize,
    current_expensive_radio_sessions: usize,
    new_connection_is_expensive_radio: bool,
    priority: Priority,
) -> bool {
    if priority == Priority::Critical {
        return true;
    }
    if current_sessions >= admission.max_simultaneous_sessions {
        return false;
    }
    if new_connection_is_expensive_radio
        && current_expensive_radio_sessions >= admission.max_expensive_radio_sessions
    {
        return false;
    }
    true
}

/// §142's own "max expensive radio sessions" needs a way to tell
/// which transports count — [`static_setup_cost`] (§44, unchanged)
/// already answers exactly this, so this is a one-line projection,
/// not a new judgment call.
pub fn is_expensive_radio_transport(transport: TransportKind) -> bool {
    static_setup_cost(transport) == SetupCost::Expensive
}

/// §143's own three named reductions, each its own function rather
/// than one that returns a bundle a caller has to destructure — the
/// three are checked at different call sites (strategy selection,
/// transport setup, background-bulk elimination), not all at once.
/// All three use the same `ThermalState::Critical` threshold
/// [`crate::discovery::discovery_permitted`] already established for
/// thermal pressure (§90) rather than inventing a second, different
/// severity line — `Elevated` alone doesn't yet restrict anything,
/// consistent with that precedent.
pub fn thermal_allows_multipath(device: Option<&DeviceState>) -> bool {
    !matches!(
        device.and_then(|d| d.thermal_state),
        Some(ThermalState::Critical)
    )
}

/// "Wi-Fi Direct setup" specifically — §143's own wording names that
/// one transport family, not transport setup in general.
pub fn thermal_allows_transport_setup(
    device: Option<&DeviceState>,
    transport: TransportKind,
) -> bool {
    if !matches!(
        transport,
        TransportKind::WifiDirect | TransportKind::WifiAware
    ) {
        return true;
    }
    !matches!(
        device.and_then(|d| d.thermal_state),
        Some(ThermalState::Critical)
    )
}

/// "Background bulk" — [`DeliveryClass::Bulk`] while the device isn't
/// foregrounded, reusing [`DeviceState::foreground`] rather than
/// inventing a second foreground signal.
pub fn thermal_allows_background_bulk(device: Option<&DeviceState>, class: DeliveryClass) -> bool {
    if class != DeliveryClass::Bulk {
        return true;
    }
    let Some(device) = device else { return true };
    if device.foreground == Some(true) {
        return true;
    }
    device.thermal_state != Some(ThermalState::Critical)
}

/// §144's own three-item list. "Reduce route queues" is
/// [`recommended_queue_capacity`]; "pause bulk acquisition" is
/// [`memory_pressure_allows_acquisition`]; "drop stale realtime
/// packets" has no function here at all — that's an action on a live
/// packet buffer this crate doesn't hold (same boundary as everything
/// else in this module's own doc comment). "Durable operations remain
/// persisted" is [`memory_pressure_allows_acquisition`]'s own
/// `durable` parameter overriding the pause, not a separate check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MemoryPressure {
    Normal,
    Elevated,
    Critical,
}

/// §144's "pause bulk acquisition... durable operations remain
/// persisted": a non-durable `Bulk` operation pauses under `Critical`
/// pressure; a durable one proceeds regardless, since pausing it
/// would contradict "remain persisted" — a durable operation's whole
/// point is that giving up on it isn't the answer to a resource
/// squeeze.
pub fn memory_pressure_allows_acquisition(
    device: Option<&DeviceState>,
    class: DeliveryClass,
    durable: bool,
) -> bool {
    if durable || class != DeliveryClass::Bulk {
        return true;
    }
    device.and_then(|d| d.memory_pressure) != Some(MemoryPressure::Critical)
}

/// §144's "reduce route queues" — a recommended capacity, not an
/// enforced one; the caller owns the actual
/// [`crate::fairness::RoundRobinFairQueue`] and decides whether/when
/// to resize it. Halves under `Elevated`, quarters under `Critical`
/// (never below 1 — a zero-capacity queue isn't "reduced," it's
/// disabled, which isn't what "reduce" asked for).
pub fn recommended_queue_capacity(base_capacity: usize, device: Option<&DeviceState>) -> usize {
    let divisor = match device.and_then(|d| d.memory_pressure) {
        Some(MemoryPressure::Critical) => 4,
        Some(MemoryPressure::Elevated) => 2,
        Some(MemoryPressure::Normal) | None => 1,
    };
    (base_capacity / divisor).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_140_only_bulk_yields_to_a_realtime_reservation() {
        let reservation = BandwidthReservation {
            reserved_for_realtime: Bitrate(1_000_000),
        };
        assert!(bulk_should_yield_to_reservation(
            DeliveryClass::Bulk,
            Some(&reservation)
        ));
        assert!(!bulk_should_yield_to_reservation(
            DeliveryClass::Interactive,
            Some(&reservation)
        ));
        assert!(!bulk_should_yield_to_reservation(DeliveryClass::Bulk, None));
    }

    #[test]
    fn spec_141_bulk_cap_only_applies_while_a_call_is_active() {
        let policy = TrafficShapingPolicy {
            bulk_max_bitrate_during_call: Bitrate(5_000_000),
            background_max_bitrate: Bitrate(1_000_000),
        };
        assert_eq!(
            effective_traffic_cap(&policy, DeliveryClass::Bulk, Priority::Normal, true),
            Some(Bitrate(5_000_000))
        );
        assert_eq!(
            effective_traffic_cap(&policy, DeliveryClass::Bulk, Priority::Normal, false),
            None
        );
    }

    #[test]
    fn spec_141_background_priority_is_capped_regardless_of_call_state() {
        let policy = TrafficShapingPolicy {
            bulk_max_bitrate_during_call: Bitrate(5_000_000),
            background_max_bitrate: Bitrate(1_000_000),
        };
        assert_eq!(
            effective_traffic_cap(
                &policy,
                DeliveryClass::Reliable,
                Priority::Background,
                false
            ),
            Some(Bitrate(1_000_000))
        );
    }

    #[test]
    fn spec_142_critical_priority_always_bypasses_admission_control() {
        let admission = ConnectionAdmission {
            max_simultaneous_sessions: 0,
            max_expensive_radio_sessions: 0,
        };
        assert!(admission_permitted(
            &admission,
            100,
            100,
            true,
            Priority::Critical
        ));
    }

    #[test]
    fn spec_142_low_priority_is_rejected_once_the_session_cap_is_reached() {
        let admission = ConnectionAdmission {
            max_simultaneous_sessions: 2,
            max_expensive_radio_sessions: 5,
        };
        assert!(admission_permitted(&admission, 1, 0, false, Priority::Low));
        assert!(!admission_permitted(&admission, 2, 0, false, Priority::Low));
    }

    #[test]
    fn spec_142_the_expensive_radio_cap_is_independent_of_the_session_cap() {
        let admission = ConnectionAdmission {
            max_simultaneous_sessions: 10,
            max_expensive_radio_sessions: 1,
        };
        assert!(!admission_permitted(
            &admission,
            0,
            1,
            true,
            Priority::Normal
        ));
        assert!(admission_permitted(
            &admission,
            0,
            1,
            false,
            Priority::Normal
        ));
    }

    #[test]
    fn is_expensive_radio_transport_matches_static_setup_cost() {
        assert!(is_expensive_radio_transport(TransportKind::WifiDirect));
        assert!(!is_expensive_radio_transport(TransportKind::IrohDirect));
    }

    #[test]
    fn spec_143_multipath_and_wifi_direct_setup_are_reduced_only_under_critical_thermal_pressure() {
        let elevated = DeviceState {
            thermal_state: Some(ThermalState::Elevated),
            ..Default::default()
        };
        let critical = DeviceState {
            thermal_state: Some(ThermalState::Critical),
            ..Default::default()
        };
        assert!(thermal_allows_multipath(Some(&elevated)));
        assert!(!thermal_allows_multipath(Some(&critical)));
        assert!(thermal_allows_transport_setup(
            Some(&critical),
            TransportKind::LocalLan
        ));
        assert!(!thermal_allows_transport_setup(
            Some(&critical),
            TransportKind::WifiDirect
        ));
    }

    #[test]
    fn spec_143_background_bulk_is_reduced_under_critical_thermal_pressure() {
        let backgrounded_and_critical = DeviceState {
            thermal_state: Some(ThermalState::Critical),
            foreground: Some(false),
            ..Default::default()
        };
        assert!(!thermal_allows_background_bulk(
            Some(&backgrounded_and_critical),
            DeliveryClass::Bulk
        ));
        assert!(thermal_allows_background_bulk(
            Some(&backgrounded_and_critical),
            DeliveryClass::Interactive
        ));

        let foregrounded_and_critical = DeviceState {
            thermal_state: Some(ThermalState::Critical),
            foreground: Some(true),
            ..Default::default()
        };
        assert!(thermal_allows_background_bulk(
            Some(&foregrounded_and_critical),
            DeliveryClass::Bulk
        ));
    }

    #[test]
    fn spec_144_durable_bulk_proceeds_even_under_critical_memory_pressure() {
        let device = DeviceState {
            memory_pressure: Some(MemoryPressure::Critical),
            ..Default::default()
        };
        assert!(!memory_pressure_allows_acquisition(
            Some(&device),
            DeliveryClass::Bulk,
            false
        ));
        assert!(memory_pressure_allows_acquisition(
            Some(&device),
            DeliveryClass::Bulk,
            true
        ));
    }

    #[test]
    fn spec_144_queue_capacity_shrinks_with_pressure_but_never_reaches_zero() {
        let normal = DeviceState::default();
        let elevated = DeviceState {
            memory_pressure: Some(MemoryPressure::Elevated),
            ..Default::default()
        };
        let critical = DeviceState {
            memory_pressure: Some(MemoryPressure::Critical),
            ..Default::default()
        };
        assert_eq!(recommended_queue_capacity(100, Some(&normal)), 100);
        assert_eq!(recommended_queue_capacity(100, Some(&elevated)), 50);
        assert_eq!(recommended_queue_capacity(100, Some(&critical)), 25);
        assert_eq!(recommended_queue_capacity(2, Some(&critical)), 1);
    }
}
