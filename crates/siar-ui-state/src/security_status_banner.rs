//! Global Security Status Banner (ui-ux-15 §96-100).
//!
//! §96: "may show only unresolved high-priority issues. Do not
//! permanently show 'secure' banners." §98: "critical security issue
//! should not become resolved merely because banner dismissed." §100:
//! "Unresolved state: Rust owns." Those three together are the whole
//! reason this is its own type rather than a component just calling
//! `SecurityEventState::unresolved_critical_events` directly: a
//! dismissal has to be tracked *somewhere* so the banner stops showing
//! an event the user already saw, but it must never be allowed to
//! touch `SecurityEvent::resolved` itself — that stays exactly what
//! `security_event.rs`'s `resolve()` already guarantees it is
//! (backend-owned, action-gated). `SecurityStatusBanner` is that
//! somewhere: a separate, purely local "seen" set layered on top of,
//! never feeding back into, real resolution state.

use std::collections::HashSet;

use crate::security_event::{SecurityEvent, SecurityEventId, SecurityEventState};

#[derive(Debug, Default)]
pub struct SecurityStatusBanner {
    dismissed: HashSet<SecurityEventId>,
}

impl SecurityStatusBanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// §96-97: only unresolved, critical-severity, not-yet-dismissed
    /// events. An event that becomes resolved (via
    /// `SecurityEventState::resolve`, the one real way that happens)
    /// disappears from this list on its own, the same as it disappears
    /// from `unresolved_critical_events()` — no separate bookkeeping
    /// needed here for that half.
    pub fn visible_issues<'a>(&self, events: &'a SecurityEventState) -> Vec<&'a SecurityEvent> {
        events
            .unresolved_critical_events()
            .into_iter()
            .filter(|event| !self.dismissed.contains(&event.id))
            .collect()
    }

    /// §98: dismissing hides the banner locally — it does **not** call
    /// `SecurityEventState::resolve`, and has no way to (this type
    /// doesn't hold a mutable reference to `SecurityEventState` at
    /// all). An event dismissed here still shows up in
    /// `SecurityEventState::unresolved_critical_events()` — only
    /// `visible_issues` on *this* type stops returning it.
    pub fn dismiss(&mut self, id: SecurityEventId) {
        self.dismissed.insert(id);
    }

    /// §99: "optional for non-critical warnings" — not implemented
    /// here (this type only ever tracks critical-severity events to
    /// begin with, via `visible_issues`'s own filter), left as a real,
    /// documented gap rather than a snooze mechanism bolted onto the
    /// wrong severity tier.
    pub fn dismissed_count(&self) -> usize {
        self.dismissed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security_event::SecurityEventKind;

    #[test]
    fn dismissing_an_event_hides_it_from_the_banner_but_not_from_resolution_state() {
        let mut events = SecurityEventState::new();
        let id = events.push(
            SecurityEventKind::SuspiciousAuthorization,
            1_000,
            None,
            None,
            vec![],
        );

        let mut banner = SecurityStatusBanner::new();
        assert_eq!(banner.visible_issues(&events).len(), 1);

        banner.dismiss(id);
        assert_eq!(banner.visible_issues(&events).len(), 0);

        // §98's own rule: dismissal must not have resolved anything.
        assert_eq!(events.unresolved_critical_events().len(), 1);
    }

    #[test]
    fn resolving_the_underlying_event_removes_it_from_the_banner_without_a_dismiss_call() {
        let mut events = SecurityEventState::new();
        let id = events.push(
            SecurityEventKind::IdentityChanged,
            1_000,
            None,
            None,
            vec![],
        );
        let banner = SecurityStatusBanner::new();
        assert_eq!(banner.visible_issues(&events).len(), 1);

        events.resolve(id);
        assert_eq!(banner.visible_issues(&events).len(), 0);
    }

    #[test]
    fn warning_and_info_severity_events_never_appear_in_the_banner() {
        let mut events = SecurityEventState::new();
        events.push(SecurityEventKind::DeviceLinked, 1_000, None, None, vec![]); // Info
        events.push(SecurityEventKind::BackupFailed, 2_000, None, None, vec![]); // Warning
        let banner = SecurityStatusBanner::new();
        assert!(banner.visible_issues(&events).is_empty());
    }

    #[test]
    fn no_events_means_no_permanent_secure_banner_content() {
        // §96: "do not permanently show 'secure' banners" — this
        // module's own contribution to that rule is structural: there
        // is no "everything is fine" variant here at all, only a
        // (possibly empty) list of issues. A component rendering an
        // empty list has nothing to show, which is the point.
        let events = SecurityEventState::new();
        let banner = SecurityStatusBanner::new();
        assert!(banner.visible_issues(&events).is_empty());
    }
}
