//! App lifecycle (plan.md §36–37). Platform-agnostic on purpose: mobile
//! OS callbacks (Android's `onPause`/`onResume`, iOS's
//! `applicationDidEnterBackground`/`applicationWillEnterForeground`) and
//! desktop's simpler always-on model both just need to report
//! transitions into this state machine — the sync/connection-teardown
//! *decisions* live here where they're testable, not scattered across
//! platform callback code this workspace has no way to verify at all
//! right now.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    /// Actively visible, user interacting or could be at any moment.
    Foreground,
    /// Not visible but the OS hasn't reclaimed the process yet — mobile
    /// OSes grant a limited window here (plan.md §36) before suspending.
    Background,
    /// Process may be frozen or killed at any point; plan.md §116's rule
    /// applies with extra force here — nothing may depend on a graceful
    /// shutdown happening from this state.
    Suspended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleTransition {
    Allowed,
    /// The OS/platform layer is reporting a transition this state
    /// machine doesn't expect (e.g. `Suspended -> Background` without
    /// passing through `Foreground` first, which some platforms do
    /// report on wake) — not an error, just informs the caller it should
    /// treat this as "resume from cold", not "resume from warm".
    ResumeFromCold,
}

impl LifecycleState {
    /// What connection/sync behavior a transition implies (plan.md §35's
    /// "mobile should be substantially more aggressive about shutdown
    /// than desktop").
    pub fn transition_to(self, next: LifecycleState) -> (LifecycleState, LifecycleTransition) {
        use LifecycleState::*;
        let kind = match (self, next) {
            (Suspended, Foreground) => LifecycleTransition::ResumeFromCold,
            _ => LifecycleTransition::Allowed,
        };
        (next, kind)
    }

    /// plan.md §35: whether pooled connections should be torn down
    /// eagerly on this transition rather than left to idle out.
    pub fn should_close_idle_connections(self, next: LifecycleState) -> bool {
        matches!(next, LifecycleState::Background | LifecycleState::Suspended) && self == LifecycleState::Foreground
    }

    /// plan.md §37: on resume, the sequence is identity -> DB -> Iroh
    /// endpoint -> mailbox fetch -> outbox flush -> UI update, and
    /// crucially never blocks the UI on network. This just flags *that*
    /// a full resync is warranted, not the sequence itself (that's
    /// `MessageService`'s job, once there's a compiler to verify the
    /// wiring against).
    pub fn should_trigger_full_resync(self, next: LifecycleState) -> bool {
        matches!(self, LifecycleState::Suspended) && next == LifecycleState::Foreground
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use LifecycleState::*;

    #[test]
    fn warm_foreground_to_background_is_an_ordinary_transition() {
        let (state, kind) = Foreground.transition_to(Background);
        assert_eq!(state, Background);
        assert_eq!(kind, LifecycleTransition::Allowed);
    }

    #[test]
    fn resuming_from_suspended_is_flagged_as_cold() {
        let (_, kind) = Suspended.transition_to(Foreground);
        assert_eq!(kind, LifecycleTransition::ResumeFromCold);
    }

    #[test]
    fn leaving_foreground_closes_idle_connections() {
        assert!(Foreground.should_close_idle_connections(Background));
        assert!(Foreground.should_close_idle_connections(Suspended));
        assert!(!Foreground.should_close_idle_connections(Foreground));
    }

    #[test]
    fn only_cold_resume_triggers_a_full_resync() {
        assert!(Suspended.should_trigger_full_resync(Foreground));
        assert!(!Background.should_trigger_full_resync(Foreground));
        assert!(!Foreground.should_trigger_full_resync(Background));
    }
}
