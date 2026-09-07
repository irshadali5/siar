//! §44 "Transport Setup Cost", §46 "Connection Pool Integration".

use serde::{Deserialize, Serialize};

use crate::types::TransportKind;

/// §44's own worked example ("existing Iroh session → cheap, Wi-Fi
/// Direct group creation → expensive, Bluetooth pairing → expensive")
/// as an ordered scale rather than a raw cost number — this crate has
/// no measured latency/battery figures for any of these (no wire
/// integration; see this crate's own top doc comment), so a coarse,
/// three-step ordering that [`crate::scoring`] can fold into its
/// weighted sum is honest about the precision actually available,
/// matching [`crate::metrics::StabilityScore`]/[`crate::metrics::EnergyCost`]'s
/// own choice of ordered enum over invented numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SetupCost {
    Cheap,
    Moderate,
    Expensive,
}

/// §46: "Transport Manager should expose: active connection,
/// connecting, idle, unreachable. Routing can reuse pooled
/// connections. Do not create a connection per message." This crate
/// has no Transport Manager of its own to query (same scope boundary
/// as everything else live/wire-related) — a caller that does own one
/// reports the state per candidate via
/// [`crate::metrics::PathMetrics::pool_state`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionPoolState {
    Active,
    Connecting,
    Idle,
    Unreachable,
}

/// §44's static cost per transport, before any pool-state adjustment.
/// `IrohDirect`/`IrohRelay` and `LocalLan` need no pairing/group
/// creation step (§44's own "existing Iroh session → cheap" example,
/// extended to LAN for the same reason — a socket connect, not a
/// negotiated session); `Dtn` needs no live connection at all (§56:
/// it hands off to store-and-forward, not a dial); `WifiDirect`/
/// `WifiAware`/`BluetoothClassic` match §44's own "expensive" example
/// exactly (group creation, pairing); `BluetoothLe`/`MeshRelay` sit in
/// between — BLE's GATT connection setup is lighter than classic
/// pairing but still a real radio/discovery cost (§54's own
/// "discovery, control" framing), and a mesh hop reuses an already-
/// authenticated peer session (§55) rather than negotiating a new one,
/// but still carries per-hop forwarding setup §44 doesn't name a
/// tier for on its own.
pub fn static_setup_cost(transport: TransportKind) -> SetupCost {
    match transport {
        TransportKind::IrohDirect | TransportKind::IrohRelay | TransportKind::LocalLan => {
            SetupCost::Cheap
        }
        TransportKind::Dtn => SetupCost::Cheap,
        TransportKind::BluetoothLe | TransportKind::MeshRelay => SetupCost::Moderate,
        TransportKind::WifiDirect | TransportKind::WifiAware | TransportKind::BluetoothClassic => {
            SetupCost::Expensive
        }
    }
}

/// §46's actual point: "routing can reuse pooled connections" — an
/// [`ConnectionPoolState::Active`] connection collapses setup cost to
/// [`SetupCost::Cheap`] regardless of the transport's static cost
/// (§44's own example is precisely this: an *existing* Iroh session is
/// cheap; a *cold* one wouldn't automatically be). [`ConnectionPoolState::Unreachable`]
/// raises cost to [`SetupCost::Expensive`] even for a normally-cheap
/// transport — a pool that has already given up on a connection is a
/// real signal that re-establishing it costs more than the static
/// baseline assumes (this is a pool-liveness cost, not a path-health
/// judgment — [`crate::types::RouteHealth`] is the hard-constraint
/// check for whether the path is usable at all; this only adjusts
/// *cost*). `Connecting`/`Idle` don't move the static cost — a
/// connection mid-handshake or idle-but-present hasn't yet earned
/// `Active`'s full discount, but hasn't shown `Unreachable`'s
/// penalty signal either.
pub fn effective_setup_cost(
    transport: TransportKind,
    pool_state: Option<ConnectionPoolState>,
) -> SetupCost {
    match pool_state {
        Some(ConnectionPoolState::Active) => SetupCost::Cheap,
        Some(ConnectionPoolState::Unreachable) => SetupCost::Expensive,
        Some(ConnectionPoolState::Connecting) | Some(ConnectionPoolState::Idle) | None => {
            static_setup_cost(transport)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_active_pooled_connection_is_always_cheap_even_for_an_expensive_transport() {
        assert_eq!(
            effective_setup_cost(TransportKind::WifiDirect, Some(ConnectionPoolState::Active)),
            SetupCost::Cheap
        );
    }

    #[test]
    fn an_unreachable_pool_entry_makes_even_a_cheap_transport_expensive() {
        assert_eq!(
            effective_setup_cost(
                TransportKind::IrohDirect,
                Some(ConnectionPoolState::Unreachable)
            ),
            SetupCost::Expensive
        );
    }

    #[test]
    fn no_pool_state_falls_back_to_the_static_cost() {
        assert_eq!(
            effective_setup_cost(TransportKind::BluetoothClassic, None),
            SetupCost::Expensive
        );
        assert_eq!(
            effective_setup_cost(TransportKind::LocalLan, None),
            SetupCost::Cheap
        );
    }

    #[test]
    fn wifi_direct_and_bluetooth_pairing_are_both_expensive_per_44s_own_example() {
        assert_eq!(
            static_setup_cost(TransportKind::WifiDirect),
            SetupCost::Expensive
        );
        assert_eq!(
            static_setup_cost(TransportKind::BluetoothClassic),
            SetupCost::Expensive
        );
    }
}
