//! §134 "Local-Only Mode", §135 "Internet-Only Mode", §136
//! "Nearby-Only Mode", §137 "Route Scope".
//!
//! [`RouteScope`] is §137's own enum, transcribed exactly with its
//! four variants. [`transport_allowed_in_scope`] encodes §134/§135/§136's
//! own three named transport lists directly — each list is quoted
//! into that function's own doc comment so the mapping can be checked
//! against the spec text at a glance rather than trusted on faith.
//!
//! Two things worth naming rather than silently reconciling: §134's
//! own "Local-Only" list includes `Dtn`, but §136's own "Nearby-Only"
//! list does not, even though every other proximity transport
//! (`LocalLan`/`WifiDirect`/`WifiAware`/Bluetooth/`MeshRelay`) appears
//! in both. That's the spec's own asymmetry, not a transcription
//! error here — a DTN carrier can eventually hand a message off
//! through a peer that *does* have Internet access, which is
//! presumably why "nearby" (a claim about where the data stays)
//! excludes it while "local" (a claim about not dialing the Internet
//! *from this device*) doesn't. Second: this module doesn't
//! distinguish `PrivacyPolicy::nearby_only`/`internet_only` (§49, user
//! preference, already real since round 2) from `RouteScope`
//! (§134-137, an application-level *mode*, not a per-user toggle) —
//! they happen to draw the same nearby/internet line today, but
//! [`eliminate_out_of_scope_candidates`] is kept as its own function
//! rather than folded into [`crate::privacy::eliminate_privacy_violations`]
//! precisely so the two can diverge later (e.g. if `RouteScope` grows
//! a fifth variant `PrivacyPolicy` has no equivalent for) without one
//! silently changing the other's behavior.

use crate::candidate::PathCandidate;
use crate::types::TransportKind;

/// §137, transcribed exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteScope {
    Any,
    InternetOnly,
    LocalOnly,
    NearbyOnly,
}

/// §134: "LAN, Wi-Fi Direct/Aware, Bluetooth, mesh, DTN." §135: "Iroh
/// direct/relay only." §136: "LAN, Wi-Fi Direct, Wi-Fi Aware,
/// Bluetooth, mesh" (no DTN — see this module's own doc comment for
/// why that's the spec's own asymmetry, not an oversight here).
pub fn transport_allowed_in_scope(transport: TransportKind, scope: RouteScope) -> bool {
    match scope {
        RouteScope::Any => true,
        RouteScope::InternetOnly => {
            matches!(
                transport,
                TransportKind::IrohDirect | TransportKind::IrohRelay
            )
        }
        RouteScope::LocalOnly => matches!(
            transport,
            TransportKind::LocalLan
                | TransportKind::WifiDirect
                | TransportKind::WifiAware
                | TransportKind::BluetoothClassic
                | TransportKind::BluetoothLe
                | TransportKind::MeshRelay
                | TransportKind::Dtn
        ),
        RouteScope::NearbyOnly => matches!(
            transport,
            TransportKind::LocalLan
                | TransportKind::WifiDirect
                | TransportKind::WifiAware
                | TransportKind::BluetoothClassic
                | TransportKind::BluetoothLe
                | TransportKind::MeshRelay
        ),
    }
}

/// §137's own "applications can request a scope" — the composable,
/// list-wide version, same shape as every other elimination function
/// in this crate.
pub fn eliminate_out_of_scope_candidates(
    candidates: &[PathCandidate],
    scope: RouteScope,
) -> Vec<&PathCandidate> {
    candidates
        .iter()
        .filter(|c| transport_allowed_in_scope(c.transport, scope))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::metrics::PathMetrics;
    use crate::types::{MeteredState, PathCapabilities, PathId, RoamingState, RouteHealth};
    use siar_domain::DeviceId;

    fn candidate(transport: TransportKind) -> PathCandidate {
        PathCandidate {
            path_id: PathId::new(),
            transport,
            peer: DeviceId::new(),
            endpoint: TransportEndpoint(Vec::new()),
            metrics: PathMetrics::unknown(),
            capabilities: PathCapabilities {
                reliable_stream: true,
                datagram: false,
                large_files: false,
                realtime_media: false,
                peer_discovery: false,
                store_and_forward: false,
                metered: MeteredState::Unknown,
                roaming: RoamingState::Unknown,
                requires_foreground: false,
            },
            health: RouteHealth::Healthy,
            underlay: None,
            state: crate::acquisition::CandidateState::Active,
        }
    }

    #[test]
    fn spec_134_local_only_permits_dtn_but_not_iroh() {
        assert!(transport_allowed_in_scope(
            TransportKind::Dtn,
            RouteScope::LocalOnly
        ));
        assert!(!transport_allowed_in_scope(
            TransportKind::IrohDirect,
            RouteScope::LocalOnly
        ));
    }

    #[test]
    fn spec_135_internet_only_permits_only_iroh_direct_and_relay() {
        assert!(transport_allowed_in_scope(
            TransportKind::IrohDirect,
            RouteScope::InternetOnly
        ));
        assert!(transport_allowed_in_scope(
            TransportKind::IrohRelay,
            RouteScope::InternetOnly
        ));
        assert!(!transport_allowed_in_scope(
            TransportKind::LocalLan,
            RouteScope::InternetOnly
        ));
        assert!(!transport_allowed_in_scope(
            TransportKind::Dtn,
            RouteScope::InternetOnly
        ));
    }

    #[test]
    fn spec_136_nearby_only_excludes_dtn_unlike_local_only() {
        assert!(!transport_allowed_in_scope(
            TransportKind::Dtn,
            RouteScope::NearbyOnly
        ));
        assert!(transport_allowed_in_scope(
            TransportKind::MeshRelay,
            RouteScope::NearbyOnly
        ));
        assert!(!transport_allowed_in_scope(
            TransportKind::IrohDirect,
            RouteScope::NearbyOnly
        ));
    }

    #[test]
    fn any_scope_restricts_nothing() {
        for t in [
            TransportKind::IrohDirect,
            TransportKind::Dtn,
            TransportKind::BluetoothLe,
        ] {
            assert!(transport_allowed_in_scope(t, RouteScope::Any));
        }
    }

    #[test]
    fn eliminate_out_of_scope_candidates_keeps_only_the_allowed_transports() {
        let candidates = vec![
            candidate(TransportKind::IrohDirect),
            candidate(TransportKind::Dtn),
        ];
        let kept = eliminate_out_of_scope_candidates(&candidates, RouteScope::InternetOnly);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].transport, TransportKind::IrohDirect);
    }
}
