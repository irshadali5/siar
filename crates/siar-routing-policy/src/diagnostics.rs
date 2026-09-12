//! §156 "Route Decision Logging", §157 "Developer Diagnostics", §158
//! "Route History".
//!
//! §156's own "not payload content" is already true by construction —
//! this crate has never had a message-body/payload type anywhere in
//! it (same boundary [`crate::descriptor::OperationDescriptor`]'s own
//! doc comment already draws), so [`RouteDecisionLog`] couldn't
//! accidentally carry payload content even if a future round wanted
//! it to.
//!
//! §157's own worked example —
//!
//! ```text
//! Destination: Bob Phone
//! Selected: Iroh Direct
//! Score: 8240
//! Fallback: Iroh Relay
//! Reason: existing session + low RTT
//! ```
//!
//! — names one field this crate genuinely cannot produce:
//! "Destination: Bob Phone" is a human-assigned contact name, and
//! this crate has never had an address book or naming service
//! dependency (its own `Destination` type carries only ids). Every
//! other line has a real source: "Selected"/"Fallback" are
//! [`crate::types::TransportKind`], "Score" is
//! [`crate::scoring::RouteScore::as_fixed_point`] (§155, this round),
//! "Reason" is [`crate::explain::RouteReason::description`] (new this
//! round specifically for this worked example). [`DeveloperDiagnostics`]
//! carries exactly the fields this crate *can* produce, and its own
//! doc comment names the one it can't.

use crate::descriptor::OperationId;
use crate::plan::RoutePlan;
use crate::policy::RoutingPolicyProfile;
use crate::types::PathId;
use crate::types::TransportKind;

/// §156, transcribed with its own five named fields. "Candidate
/// count" is a plain `usize` — this crate doesn't otherwise track how
/// many candidates existed once a plan is built, so a caller supplies
/// it from what it had on hand when it called
/// [`crate::plan::plan_route`]/[`crate::decision::decide_route`].
/// "Policy profile" is `Option<RoutingPolicyProfile>` rather than
/// required — `decide_route` itself takes a raw
/// [`crate::policy::RoutingPolicy`], not the named
/// [`RoutingPolicyProfile`] enum it usually comes from, so a caller
/// using a hand-built policy has none to supply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteDecisionLog {
    pub operation_id: OperationId,
    pub selected_path: PathId,
    pub reason: crate::explain::RouteReason,
    pub candidate_count: usize,
    pub policy_profile: Option<RoutingPolicyProfile>,
}

/// Builds §156's log entry from a plan a caller already has, plus the
/// two facts only the caller knows (which operation this was for, how
/// many candidates it started with).
pub fn log_for_decision(
    operation_id: OperationId,
    plan: &RoutePlan,
    candidate_count: usize,
    policy_profile: Option<RoutingPolicyProfile>,
) -> RouteDecisionLog {
    RouteDecisionLog {
        operation_id,
        selected_path: plan.primary.path_id,
        reason: plan.reason,
        candidate_count,
        policy_profile,
    }
}

/// §157's own worked example, transcribed field-for-field minus
/// "Destination" — see this module's own doc comment for why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeveloperDiagnostics {
    pub selected: TransportKind,
    pub score: u16,
    pub fallback: Option<TransportKind>,
    pub reason: &'static str,
}

/// Builds §157's diagnostics view from a plan and the policy weights
/// that scored it (needed for [`crate::scoring::RouteScore::as_fixed_point`]'s
/// own normalization).
pub fn diagnostics_for(
    plan: &RoutePlan,
    weights: &crate::policy::PolicyWeights,
) -> DeveloperDiagnostics {
    DeveloperDiagnostics {
        selected: plan.primary.transport,
        score: plan.primary_score.as_fixed_point(weights),
        fallback: plan.fallbacks.first().map(|c| c.transport),
        reason: plan.reason.description(),
    }
}

/// §158: "keep bounded recent history: last N decisions... do not
/// retain indefinitely." A fixed-capacity ring buffer, not a caller
/// discipline — the bound is enforced structurally by
/// [`RouteHistory::push`] itself evicting the oldest entry, the same
/// "bounded by construction, not by someone remembering to trim it"
/// shape [`crate::fairness::RoundRobinFairQueue`]'s own per-key
/// capacity already takes for a different kind of history.
#[derive(Debug, Clone)]
pub struct RouteHistory {
    capacity: usize,
    entries: std::collections::VecDeque<RouteDecisionLog>,
}

impl RouteHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: std::collections::VecDeque::new(),
        }
    }

    /// Evicts the oldest entry first when already at capacity — "last
    /// N decisions" means the N *most recent*, not the first N ever
    /// seen.
    pub fn push(&mut self, entry: RouteDecisionLog) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn recent(&self) -> impl Iterator<Item = &RouteDecisionLog> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::{PathCandidate, TransportEndpoint};
    use crate::explain::RouteReason;
    use crate::metrics::PathMetrics;
    use crate::plan::RouteStrategy;
    use crate::scoring::RouteScore;
    use crate::types::{
        MeteredState, PathCapabilities, PathId as PathIdType, RoamingState, RouteHealth,
    };
    use siar_domain::DeviceId;

    fn candidate(transport: TransportKind) -> PathCandidate {
        PathCandidate {
            path_id: PathIdType::new(),
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

    fn plan_with(
        primary_transport: TransportKind,
        fallback_transport: Option<TransportKind>,
        reason: RouteReason,
        score: f64,
    ) -> RoutePlan {
        RoutePlan {
            primary: candidate(primary_transport),
            fallbacks: fallback_transport.into_iter().map(candidate).collect(),
            replicas: vec![],
            strategy: RouteStrategy::Single,
            hedge_delay_millis: None,
            reason,
            created_at_millis: 0,
            valid_until_millis: 0,
            primary_score: RouteScore(score),
        }
    }

    #[test]
    fn spec_156_log_carries_the_five_named_fields() {
        let plan = plan_with(
            TransportKind::IrohDirect,
            Some(TransportKind::IrohRelay),
            RouteReason::ExistingHealthyConnection,
            5.0,
        );
        let op_id = OperationId::new();
        let log = log_for_decision(op_id, &plan, 3, Some(RoutingPolicyProfile::Balanced));

        assert_eq!(log.operation_id, op_id);
        assert_eq!(log.selected_path, plan.primary.path_id);
        assert_eq!(log.reason, RouteReason::ExistingHealthyConnection);
        assert_eq!(log.candidate_count, 3);
        assert_eq!(log.policy_profile, Some(RoutingPolicyProfile::Balanced));
    }

    /// §157's own worked example, transcribed as far as this crate
    /// honestly can: selected transport, score, fallback, reason —
    /// not "Destination: Bob Phone," which has no source here (see
    /// this module's own doc comment).
    #[test]
    fn spec_157_diagnostics_matches_the_worked_examples_shape() {
        let weights = RoutingPolicyProfile::Balanced.policy().weights;
        let max_possible = weights.sum();
        let plan = plan_with(
            TransportKind::IrohDirect,
            Some(TransportKind::IrohRelay),
            RouteReason::ExistingHealthyConnection,
            max_possible, // maximal score → 10_000 at the top of the range
        );

        let diagnostics = diagnostics_for(&plan, &weights);
        assert_eq!(diagnostics.selected, TransportKind::IrohDirect);
        assert_eq!(diagnostics.score, 10_000);
        assert_eq!(diagnostics.fallback, Some(TransportKind::IrohRelay));
        assert_eq!(diagnostics.reason, "existing healthy connection");
    }

    #[test]
    fn diagnostics_reports_no_fallback_for_a_single_strategy_plan() {
        let weights = RoutingPolicyProfile::Balanced.policy().weights;
        let plan = plan_with(
            TransportKind::LocalLan,
            None,
            RouteReason::LowestLatency,
            0.0,
        );
        let diagnostics = diagnostics_for(&plan, &weights);
        assert_eq!(diagnostics.fallback, None);
    }

    #[test]
    fn spec_158_history_keeps_only_the_n_most_recent_entries() {
        let mut history = RouteHistory::new(2);
        let plan = plan_with(
            TransportKind::IrohDirect,
            None,
            RouteReason::LowestLatency,
            1.0,
        );

        let first_op = OperationId::new();
        let second_op = OperationId::new();
        let third_op = OperationId::new();
        history.push(log_for_decision(first_op, &plan, 1, None));
        history.push(log_for_decision(second_op, &plan, 1, None));
        history.push(log_for_decision(third_op, &plan, 1, None));

        assert_eq!(history.len(), 2);
        let ids: Vec<_> = history.recent().map(|e| e.operation_id).collect();
        // The oldest (`first_op`) was evicted; the two most recent
        // remain, in order.
        assert_eq!(ids, vec![second_op, third_op]);
    }

    #[test]
    fn a_zero_capacity_history_still_holds_at_least_one_entry() {
        let mut history = RouteHistory::new(0);
        let plan = plan_with(
            TransportKind::IrohDirect,
            None,
            RouteReason::LowestLatency,
            1.0,
        );
        history.push(log_for_decision(OperationId::new(), &plan, 1, None));
        assert_eq!(history.len(), 1);
    }
}
