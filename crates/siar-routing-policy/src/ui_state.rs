//! §116 "UI-Friendly State".
//!
//! "Product UI maps it into wording. Do not expose raw transport
//! errors to normal users" — the whole point of this five-variant
//! enum is that it's smaller and less specific than
//! [`crate::decision::RouteDecisionResult`]/[`crate::decision::DeferredReason`],
//! not a renaming of them. [`ui_state_for`] is a genuine many-to-one
//! collapse: several different engine-level reasons legitimately map
//! to the same neutral state, because a normal user doesn't need to
//! know *which* policy layer is currently blocking them, only that
//! they should keep waiting.
//!
//! Two things worth naming honestly about that collapse, since
//! neither is dictated by §116's own text:
//! - [`RouteUiState::CarriedByNearbyPeer`] is inferred, not
//!   transcribed as a direct field mapping: it fires specifically for
//!   a [`crate::plan::RoutePlan`] whose strategy is
//!   [`crate::plan::RouteStrategy::DelayTolerant`] or whose primary
//!   candidate's transport is [`crate::types::TransportKind::Dtn`] —
//!   read as "this is the one case where 'sending' actually means
//!   'handed to a store-and-forward carrier,' which is a materially
//!   different thing to tell a user than 'still connecting.'"
//! - [`RouteDecisionResult::Rejected`]/[`RouteDecisionResult::Unreachable`]
//!   have no dedicated state in §116's own five-item list (the list
//!   reads as describing the *in-flight* states of something still
//!   trying, not a terminal failure state) — both map to
//!   [`RouteUiState::WaitingForConnection`] here, the same neutral
//!   "can't proceed right now" wording a
//!   [`crate::decision::DeferredReason::NoSuitablePathYet`] would also
//!   get, rather than this module inventing a sixth variant the spec
//!   never named. A caller that wants to distinguish "will resolve on
//!   its own" from "needs a decision only a human/app layer can make"
//!   still has the full [`crate::decision::RouteDecisionResult`] to
//!   inspect directly — collapsing it is what this module does, not
//!   what it forces on every caller.
//!
//! [`RouteUiState::Delivered`] is transcribed because §116 lists it,
//! but has no case in [`ui_state_for`] that produces it — this crate
//! has no delivery-acknowledgement concept at all (it plans routes,
//! it doesn't confirm receipt), so nothing here could honestly claim
//! to derive that state. A caller reaching `Delivered` gets there from
//! an ack signal entirely outside this crate, not from anything
//! [`ui_state_for`] returns.

use crate::decision::{DeferredReason, RouteDecisionResult};
use crate::plan::RouteStrategy;
use crate::types::TransportKind;

/// §116, transcribed exactly, in the order listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteUiState {
    Sending,
    WaitingForConnection,
    WaitingForWiFi,
    CarriedByNearbyPeer,
    Delivered,
}

/// See this module's own doc comment for the two inferred (not
/// spec-dictated) collapsing choices this function makes.
pub fn ui_state_for(result: &RouteDecisionResult) -> RouteUiState {
    match result {
        RouteDecisionResult::Routed(plan) => {
            if plan.strategy == RouteStrategy::DelayTolerant
                || plan.primary.transport == TransportKind::Dtn
            {
                RouteUiState::CarriedByNearbyPeer
            } else {
                RouteUiState::Sending
            }
        }
        RouteDecisionResult::Deferred(reason) => match reason {
            DeferredReason::WaitingForWifi | DeferredReason::WaitingForUnmetered => {
                RouteUiState::WaitingForWiFi
            }
            DeferredReason::WaitingForPeer => RouteUiState::WaitingForConnection,
            DeferredReason::BatteryPolicy
            | DeferredReason::BackgroundRestriction
            | DeferredReason::NoSuitablePathYet => RouteUiState::WaitingForConnection,
        },
        RouteDecisionResult::Rejected(_) | RouteDecisionResult::Unreachable => {
            RouteUiState::WaitingForConnection
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::decision::RejectReason;
    use crate::explain::RouteReason;
    use crate::metrics::PathMetrics;
    use crate::plan::RoutePlan;
    use crate::types::{PathCapabilities, PathId, RouteHealth};
    use siar_domain::DeviceId;

    fn plan_with(strategy: RouteStrategy, transport: TransportKind) -> RoutePlan {
        RoutePlan {
            primary: crate::candidate::PathCandidate {
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
                    metered: crate::types::MeteredState::Unknown,
                    roaming: crate::types::RoamingState::Unknown,
                    requires_foreground: false,
                },
                health: RouteHealth::Healthy,
                underlay: None,
                state: crate::acquisition::CandidateState::Active,
            },
            fallbacks: Vec::new(),
            replicas: Vec::new(),
            strategy,
            hedge_delay_millis: None,
            reason: RouteReason::LowestLatency,
            created_at_millis: 0,
            valid_until_millis: 0,
        }
    }

    #[test]
    fn an_ordinary_routed_plan_is_sending() {
        let plan = plan_with(RouteStrategy::Single, TransportKind::IrohDirect);
        assert_eq!(
            ui_state_for(&RouteDecisionResult::Routed(plan)),
            RouteUiState::Sending
        );
    }

    #[test]
    fn a_delay_tolerant_plan_is_carried_by_nearby_peer() {
        let plan = plan_with(RouteStrategy::DelayTolerant, TransportKind::Dtn);
        assert_eq!(
            ui_state_for(&RouteDecisionResult::Routed(plan)),
            RouteUiState::CarriedByNearbyPeer
        );
    }

    #[test]
    fn a_dtn_transport_primary_is_carried_by_nearby_peer_even_off_strategy() {
        // §116's own point is about the transport actually carrying
        // the message, not merely the strategy label.
        let plan = plan_with(RouteStrategy::Single, TransportKind::Dtn);
        assert_eq!(
            ui_state_for(&RouteDecisionResult::Routed(plan)),
            RouteUiState::CarriedByNearbyPeer
        );
    }

    #[test]
    fn waiting_for_wifi_and_waiting_for_unmetered_both_read_as_waiting_for_wifi() {
        assert_eq!(
            ui_state_for(&RouteDecisionResult::Deferred(
                DeferredReason::WaitingForWifi
            )),
            RouteUiState::WaitingForWiFi
        );
        assert_eq!(
            ui_state_for(&RouteDecisionResult::Deferred(
                DeferredReason::WaitingForUnmetered
            )),
            RouteUiState::WaitingForWiFi
        );
    }

    #[test]
    fn every_other_deferred_reason_reads_as_waiting_for_connection() {
        for reason in [
            DeferredReason::WaitingForPeer,
            DeferredReason::BatteryPolicy,
            DeferredReason::BackgroundRestriction,
            DeferredReason::NoSuitablePathYet,
        ] {
            assert_eq!(
                ui_state_for(&RouteDecisionResult::Deferred(reason)),
                RouteUiState::WaitingForConnection
            );
        }
    }

    #[test]
    fn rejected_and_unreachable_never_leak_as_a_distinct_raw_state() {
        assert_eq!(
            ui_state_for(&RouteDecisionResult::Rejected(
                RejectReason::UnauthorizedDevice
            )),
            RouteUiState::WaitingForConnection
        );
        assert_eq!(
            ui_state_for(&RouteDecisionResult::Unreachable),
            RouteUiState::WaitingForConnection
        );
    }
}
