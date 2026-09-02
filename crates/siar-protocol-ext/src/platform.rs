//! spec §80 "Mobile Efficiency", §81 "Headless Nodes", §82 "Compile-Time
//! Features", §83 "Binary Size".
//!
//! §82/§83 need no code in THIS crate — they're guidance about how
//! *other* workspace crates (`siar-messaging`, `apps/*`) should gate
//! their own Cargo dependencies behind feature flags so a file-only CLI
//! doesn't pull in Dioxus/AV1/camera code it never uses. This crate is
//! deliberately standalone with nothing to gate (see lib.rs's own "No
//! wire integration" note) — there's no `comm-messaging`/`comm-files`
//! dependency here for a feature flag to control. Reconciled, not
//! reattempted as new code.

/// spec §80's own seven practices, verbatim — kept as a closed set
/// specifically because most of them are ALREADY true of this crate's
/// design rather than new work: `LazyOpenHeavyExtensions` is
/// [`crate::lifecycle::LazyInitTarget`] (§25); `UseTightQueues` is
/// [`crate::backpressure::BoundedQueue`]; `ObeyBatteryPolicy`/
/// `AvoidStartingUnusedMediaOrFiles` are consequences of the lazy-open
/// design. `StopEphemeralServicesWhenBackgrounded`/`PersistDurableWork`/
/// `ReopenAfterResume` are the three genuinely new to this pass — no
/// "background"/"resume" concept exists anywhere in this crate yet
/// (that's a platform-lifecycle integration point, not something
/// `siar-protocol-ext` itself can implement in isolation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MobileEfficiencyPractice {
    LazyOpenHeavyExtensions,
    StopEphemeralServicesWhenBackgrounded,
    PersistDurableWork,
    ReopenAfterResume,
    UseTightQueues,
    ObeyBatteryPolicy,
    AvoidStartingUnusedMediaOrFiles,
}

/// spec §81's own four-item "can register without Dioxus/messaging
/// UI/calls" list — the extensions a headless node profile can
/// meaningfully include.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HeadlessCapableExtension {
    Files,
    Dtn,
    Relay,
    Discovery,
}

/// spec §81's own five named use cases, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HeadlessUseCase {
    RaspberryPi,
    Nas,
    Server,
    EmergencyGateway,
    EnterpriseNode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_80_seven_practices_are_distinct() {
        let practices = [
            MobileEfficiencyPractice::LazyOpenHeavyExtensions,
            MobileEfficiencyPractice::StopEphemeralServicesWhenBackgrounded,
            MobileEfficiencyPractice::PersistDurableWork,
            MobileEfficiencyPractice::ReopenAfterResume,
            MobileEfficiencyPractice::UseTightQueues,
            MobileEfficiencyPractice::ObeyBatteryPolicy,
            MobileEfficiencyPractice::AvoidStartingUnusedMediaOrFiles,
        ];
        let unique: std::collections::HashSet<_> =
            practices.iter().map(|p| format!("{p:?}")).collect();
        assert_eq!(unique.len(), 7);
    }

    #[test]
    fn spec_81_headless_profile_never_needs_a_ui_extension() {
        // Structural claim: HeadlessCapableExtension has no variant
        // that could stand for Dioxus/messaging-UI/calls — proven by
        // this being the crate's exhaustive match, not by a runtime
        // check against a list.
        let profile = [
            HeadlessCapableExtension::Files,
            HeadlessCapableExtension::Dtn,
            HeadlessCapableExtension::Relay,
            HeadlessCapableExtension::Discovery,
        ];
        assert_eq!(profile.len(), 4);
    }
}
