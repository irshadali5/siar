//! §86 "Background Restrictions", §88 "Path Acquisition", §89 "Passive
//! vs Active Candidates".

use serde::{Deserialize, Serialize};

use crate::candidate::PathCandidate;
use crate::platform::DeviceState;

/// §89's own code block, transcribed exactly — `Active` (already
/// connected), `PassiveKnown` (seen via passive discovery, e.g. a
/// cached mDNS/BLE advertisement, no connection yet), `RequiresDiscovery`
/// (would need an active scan first), `RequiresSetup` (known reachable
/// but needs a handshake/pairing/group-creation step first — see
/// [`crate::setup::SetupCost`] for the cost *of* that step; this is
/// whether one is needed *at all*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateState {
    Active,
    PassiveKnown,
    RequiresDiscovery,
    RequiresSetup,
}

/// §89: "Scoring can penalize setup." Higher is better, same
/// direction as every other `_unit` function in
/// [`crate::scoring`] — kept here rather than in that module since it
/// reads a type this module owns, the same split
/// [`crate::setup::static_setup_cost`] already has from
/// [`crate::scoring::DefaultScorer`]'s own use of it.
pub fn candidate_state_unit(state: CandidateState) -> f64 {
    match state {
        CandidateState::Active => 1.0,
        CandidateState::PassiveKnown => 0.75,
        CandidateState::RequiresDiscovery => 0.4,
        CandidateState::RequiresSetup => 0.25,
    }
}

/// §86: "Candidate path must include: currently usable, requires
/// foreground. Routing should not repeatedly attempt impossible
/// background operations." Unlike [`crate::privacy::passes_privacy_policy`]'s
/// `Unknown` metered/roaming states (round 6, deliberately
/// conservative because those signals exist specifically to protect a
/// resource), an *unknown* foreground state here defaults to
/// permissive: this crate has no equivalent named reason to assume the
/// worst, and the actual harm §86 warns about — "repeatedly attempt
/// impossible background operations" — only exists when the app is
/// *confirmed* backgrounded, not merely when foreground state hasn't
/// been reported.
pub fn is_currently_usable(candidate: &PathCandidate, device: Option<&DeviceState>) -> bool {
    if !candidate.capabilities.requires_foreground {
        return true;
    }
    match device.and_then(|d| d.foreground) {
        Some(false) => false,
        Some(true) | None => true,
    }
}

/// §86 applied across a candidate list — composable, same shape as
/// [`crate::security::eliminate_untrusted_candidates`]/
/// [`crate::privacy::eliminate_privacy_violations`]: a caller that has
/// device state calls this (and [`crate::plan::plan_route`] does,
/// when given one); a caller that doesn't just skips it, since `device:
/// None` would keep every candidate anyway.
pub fn eliminate_background_restricted<'a>(
    candidates: &'a [PathCandidate],
    device: Option<&DeviceState>,
) -> Vec<&'a PathCandidate> {
    candidates
        .iter()
        .filter(|c| is_currently_usable(c, device))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::metrics::PathMetrics;
    use crate::types::{
        MeteredState, PathCapabilities, PathId, RoamingState, RouteHealth, TransportKind,
    };
    use siar_domain::DeviceId;

    fn candidate_requiring_foreground(requires_foreground: bool) -> PathCandidate {
        PathCandidate {
            path_id: PathId::new(),
            transport: TransportKind::WifiDirect,
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
                requires_foreground,
            },
            health: RouteHealth::Healthy,
            underlay: None,
            state: CandidateState::Active,
        }
    }

    #[test]
    fn a_foreground_only_candidate_is_unusable_while_confirmed_backgrounded() {
        let candidate = candidate_requiring_foreground(true);
        let device = DeviceState {
            foreground: Some(false),
            ..Default::default()
        };
        assert!(!is_currently_usable(&candidate, Some(&device)));
    }

    #[test]
    fn a_foreground_only_candidate_is_usable_while_confirmed_foregrounded() {
        let candidate = candidate_requiring_foreground(true);
        let device = DeviceState {
            foreground: Some(true),
            ..Default::default()
        };
        assert!(is_currently_usable(&candidate, Some(&device)));
    }

    #[test]
    fn an_unknown_foreground_state_defaults_permissive() {
        let candidate = candidate_requiring_foreground(true);
        assert!(is_currently_usable(&candidate, None));
        let unknown_device = DeviceState::default();
        assert!(is_currently_usable(&candidate, Some(&unknown_device)));
    }

    #[test]
    fn a_candidate_not_requiring_foreground_is_always_usable() {
        let candidate = candidate_requiring_foreground(false);
        let device = DeviceState {
            foreground: Some(false),
            ..Default::default()
        };
        assert!(is_currently_usable(&candidate, Some(&device)));
    }

    #[test]
    fn candidate_state_ranks_active_above_requires_setup() {
        assert!(
            candidate_state_unit(CandidateState::Active)
                > candidate_state_unit(CandidateState::RequiresSetup)
        );
    }
}
