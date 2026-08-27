//! Connectivity state — next.md §59–60.
//!
//! Pure data: what transports are currently up, and the single-label
//! summary the UI shows for it. Actually detecting any of this —
//! Wi-Fi Direct group formation, BLE proximity, LAN mDNS discovery
//! results — is platform- or transport-specific and lives in
//! `siar-transport`/`siar-transport-wifi-direct`/(future BLE and
//! Bluetooth Classic crates); this module is just the shape those
//! report into, so the UI and a later routing engine (next.md §90's
//! `TransportManager`) have one place to read "what's up right now"
//! from, not scattered booleans per transport.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportLink {
    InternetDirect,
    InternetRelay,
    LocalLan,
    WifiDirect,
    WifiAware,
    BluetoothClassic,
    Ble,
}

impl TransportLink {
    /// Relative preference when more than one link reaches the same
    /// destination — lower is better. Extracted out of
    /// [`ConnectivityState::effective_mode`]'s own if/else chain (added
    /// this pass, for `siar-routing::path::PathTable::best_route_for`/
    /// `recommend_upgrade`, next.md §90's "BLE→Wi-Fi upgrade" decision)
    /// so both call sites share one ordering instead of two independent
    /// orderings that could silently drift apart. `effective_mode`
    /// below is rewritten to use this rather than duplicating the order
    /// inline — same behavior, one source of truth.
    ///
    /// Note this can't be derived from `siar_routing::path::
    /// capabilities_for`'s `TransportCapabilities` — `LocalLan` and
    /// `WifiDirect`/`WifiAware` report identical capabilities there
    /// (both High bandwidth, Low latency) but are NOT equally
    /// preferred: a LAN connection typically means "already associated
    /// to the same Wi-Fi network," while Wi-Fi Direct/Aware need an
    /// explicit peer-to-peer group negotiation first. That distinction
    /// only exists in this ranking, not in raw capability class.
    pub fn preference_rank(self) -> u8 {
        use TransportLink::*;
        match self {
            InternetDirect => 0,
            InternetRelay => 1,
            LocalLan => 2,
            WifiDirect | WifiAware => 3,
            BluetoothClassic => 4,
            Ble => 5,
        }
    }
}

/// next.md §59: "internally multiple modes may coexist" — this is a
/// set, not a single enum variant, on purpose. [`effective_mode`] below
/// picks the one label the UI shows when it wants a single summary
/// (next.md §60), but the underlying set is what a routing engine
/// (a later phase) actually needs to make per-message transport
/// choices from.
///
/// [`effective_mode`]: ConnectivityState::effective_mode
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectivityState {
    active: Vec<TransportLink>,
}

impl ConnectivityState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_up(&self, link: TransportLink) -> bool {
        self.active.contains(&link)
    }

    /// Idempotent — marking an already-up link up again is a no-op,
    /// not a duplicate entry.
    pub fn mark_up(&mut self, link: TransportLink) {
        if !self.active.contains(&link) {
            self.active.push(link);
        }
    }

    /// Also a no-op if `link` wasn't up — same idempotency in reverse.
    pub fn mark_down(&mut self, link: TransportLink) {
        self.active.retain(|active_link| *active_link != link);
    }

    pub fn active_links(&self) -> &[TransportLink] {
        &self.active
    }

    pub fn has_internet(&self) -> bool {
        self.is_up(TransportLink::InternetDirect) || self.is_up(TransportLink::InternetRelay)
    }

    /// next.md §60's single-line UI summary ("Internet unavailable, 3
    /// nearby relay devices" vs. plain "Internet"). Picked by
    /// [`TransportLink::preference_rank`] (next.md §119's routing
    /// preference, shared with `siar-routing::path::PathTable`'s own
    /// use of the same ranking — see that method's doc comment), not by
    /// insertion order into `active` — two links coming up in either
    /// order should report the same effective mode.
    pub fn effective_mode(&self) -> EffectiveConnectivity {
        let Some(best) = self
            .active
            .iter()
            .copied()
            .min_by_key(|link| link.preference_rank())
        else {
            return EffectiveConnectivity::Isolated;
        };
        use TransportLink::*;
        match best {
            InternetDirect => EffectiveConnectivity::InternetDirect,
            InternetRelay => EffectiveConnectivity::InternetRelay,
            LocalLan => EffectiveConnectivity::LocalLan,
            WifiDirect | WifiAware => EffectiveConnectivity::WifiPeerToPeer,
            BluetoothClassic | Ble => EffectiveConnectivity::BluetoothDirect,
        }
    }
}

/// next.md §59's `ConnectivityMode` enum, unchanged in shape from the
/// doc — the single-label summary [`ConnectivityState::effective_mode`]
/// derives from the underlying set of active links. `MeshOnly` from the
/// doc's version isn't here yet: it depends on DTN/mesh state (next.md
/// Phase 4), which doesn't exist in this workspace yet — adding it now
/// would be a variant this type can't actually distinguish from
/// `Isolated`, which is worse than not having it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectiveConnectivity {
    InternetDirect,
    InternetRelay,
    LocalLan,
    WifiPeerToPeer,
    BluetoothDirect,
    Isolated,
}

#[cfg(test)]
mod tests {
    use super::*;
    use TransportLink::*;

    #[test]
    fn no_links_is_isolated() {
        assert_eq!(
            ConnectivityState::new().effective_mode(),
            EffectiveConnectivity::Isolated
        );
    }

    #[test]
    fn internet_direct_wins_over_everything_else() {
        let mut state = ConnectivityState::new();
        state.mark_up(Ble);
        state.mark_up(LocalLan);
        state.mark_up(InternetDirect);
        assert_eq!(
            state.effective_mode(),
            EffectiveConnectivity::InternetDirect
        );
    }

    #[test]
    fn local_lan_beats_wifi_direct_and_bluetooth() {
        let mut state = ConnectivityState::new();
        state.mark_up(Ble);
        state.mark_up(WifiDirect);
        state.mark_up(LocalLan);
        assert_eq!(state.effective_mode(), EffectiveConnectivity::LocalLan);
    }

    #[test]
    fn wifi_direct_and_wifi_aware_both_map_to_wifi_peer_to_peer() {
        let mut direct = ConnectivityState::new();
        direct.mark_up(WifiDirect);
        assert_eq!(
            direct.effective_mode(),
            EffectiveConnectivity::WifiPeerToPeer
        );

        let mut aware = ConnectivityState::new();
        aware.mark_up(WifiAware);
        assert_eq!(
            aware.effective_mode(),
            EffectiveConnectivity::WifiPeerToPeer
        );
    }

    #[test]
    fn marking_up_is_idempotent() {
        let mut state = ConnectivityState::new();
        state.mark_up(LocalLan);
        state.mark_up(LocalLan);
        assert_eq!(state.active_links(), &[LocalLan]);
    }

    #[test]
    fn marking_down_removes_only_that_link() {
        let mut state = ConnectivityState::new();
        state.mark_up(LocalLan);
        state.mark_up(Ble);
        state.mark_down(LocalLan);
        assert_eq!(state.active_links(), &[Ble]);
    }

    #[test]
    fn marking_down_an_absent_link_is_a_no_op() {
        let mut state = ConnectivityState::new();
        state.mark_up(LocalLan);
        state.mark_down(Ble);
        assert_eq!(state.active_links(), &[LocalLan]);
    }

    #[test]
    fn has_internet_true_for_either_direct_or_relay() {
        let mut relay = ConnectivityState::new();
        relay.mark_up(InternetRelay);
        assert!(relay.has_internet());
        assert!(!ConnectivityState::new().has_internet());
    }

    #[test]
    fn preference_rank_orders_internet_above_lan_above_wifi_p2p_above_bluetooth() {
        assert!(InternetDirect.preference_rank() < InternetRelay.preference_rank());
        assert!(InternetRelay.preference_rank() < LocalLan.preference_rank());
        assert!(LocalLan.preference_rank() < WifiDirect.preference_rank());
        assert!(WifiDirect.preference_rank() < BluetoothClassic.preference_rank());
        assert!(BluetoothClassic.preference_rank() < Ble.preference_rank());
    }

    #[test]
    fn wifi_direct_and_wifi_aware_rank_equally() {
        assert_eq!(WifiDirect.preference_rank(), WifiAware.preference_rank());
    }
}
