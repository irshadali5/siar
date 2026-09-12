//! §159 "Telemetry Export".
//!
//! "If enabled: aggregate route success, direct/relay ratio, failover
//! rate. Redact peer identity." [`TelemetrySummary`] has no
//! [`siar_domain::DeviceId`]/[`crate::types::PathId`] field anywhere —
//! redaction here isn't a step applied to raw data before export,
//! it's a type that was never given anywhere to put an identity in
//! the first place, the same "can't leak what it has no field for"
//! shape [`crate::explain::RouteMetricEvent`]'s own §99 already
//! established for a different privacy requirement.
//!
//! "If enabled" is a caller decision (whether to call
//! [`summarize_telemetry`] at all, and whether to transmit its
//! output anywhere) — this module doesn't have an enabled/disabled
//! flag of its own, since gating on a setting a *product* exposes to
//! users isn't a fact this crate has an opinion on.

use crate::diagnostics::RouteDecisionLog;
use crate::explain::RouteReason;
use crate::metrics::Ratio;
use crate::types::TransportKind;

/// §159's own three named aggregates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetrySummary {
    pub route_success_rate: Ratio,
    pub direct_relay_ratio: Ratio,
    pub failover_rate: Ratio,
}

fn is_direct(transport: TransportKind) -> bool {
    !matches!(
        transport,
        TransportKind::IrohRelay | TransportKind::MeshRelay
    )
}

/// Computes §159's summary from a batch of [`RouteDecisionLog`]
/// entries (typically [`crate::diagnostics::RouteHistory::recent`]'s
/// own output) and the transport each one actually selected.
/// `route_success_rate` needs an outcome per entry, which
/// `RouteDecisionLog` doesn't itself carry (it's a *decision* record,
/// not a *result* one — see [`crate::engine::RouteResultReport`] for
/// where outcomes live) — `outcomes` is a parallel slice a caller
/// supplies from its own result-tracking, `true` for success.
/// Returns `None` for an empty batch — an aggregate over zero
/// decisions isn't "0%," it's "no data yet," and a caller should be
/// able to tell the two apart rather than displaying a misleading
/// zero.
pub fn summarize_telemetry(
    entries: &[(RouteDecisionLog, TransportKind)],
    outcomes: &[bool],
) -> Option<TelemetrySummary> {
    if entries.is_empty() {
        return None;
    }

    let total = entries.len() as f64;
    let direct_count = entries.iter().filter(|(_, t)| is_direct(*t)).count() as f64;
    let failover_count = entries
        .iter()
        .filter(|(log, _)| {
            matches!(
                log.reason,
                RouteReason::RelayFallback | RouteReason::EmergencyRedundancy
            )
        })
        .count() as f64;
    let success_count = outcomes.iter().filter(|ok| **ok).count() as f64;
    let outcome_total = outcomes.len().max(1) as f64;

    Some(TelemetrySummary {
        route_success_rate: Ratio::new(success_count / outcome_total),
        direct_relay_ratio: Ratio::new(direct_count / total),
        failover_rate: Ratio::new(failover_count / total),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::OperationId;
    use crate::types::PathId;

    fn log(reason: RouteReason) -> RouteDecisionLog {
        RouteDecisionLog {
            operation_id: OperationId::new(),
            selected_path: PathId::new(),
            reason,
            candidate_count: 1,
            policy_profile: None,
        }
    }

    #[test]
    fn an_empty_batch_has_no_summary_rather_than_a_misleading_zero() {
        assert_eq!(summarize_telemetry(&[], &[]), None);
    }

    #[test]
    fn telemetry_summary_carries_no_identity_field_by_construction() {
        // Compile-time property as much as a runtime one: this test
        // exists mainly to document that the struct below has nothing
        // to destructure into an identity — if a future edit ever
        // added a `DeviceId`/`PathId` field here, this test's own
        // exhaustive-field construction would be the first thing to
        // fail to compile.
        let _ = TelemetrySummary {
            route_success_rate: Ratio::new(1.0),
            direct_relay_ratio: Ratio::new(1.0),
            failover_rate: Ratio::new(0.0),
        };
    }

    #[test]
    fn spec_159_direct_relay_ratio_and_failover_rate_are_computed_correctly() {
        let entries = vec![
            (log(RouteReason::LowestLatency), TransportKind::IrohDirect),
            (log(RouteReason::RelayFallback), TransportKind::IrohRelay),
            (log(RouteReason::LowestLatency), TransportKind::LocalLan),
            (log(RouteReason::LowestLatency), TransportKind::IrohRelay),
        ];
        let outcomes = vec![true, true, true, false];

        let summary = summarize_telemetry(&entries, &outcomes).unwrap();
        assert_eq!(summary.direct_relay_ratio, Ratio::new(0.5)); // 2 of 4 direct
        assert_eq!(summary.failover_rate, Ratio::new(0.25)); // 1 of 4 a failover reason
        assert_eq!(summary.route_success_rate, Ratio::new(0.75)); // 3 of 4 succeeded
    }
}
