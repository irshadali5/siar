//! Security event view-model state (Part 28 §42, "Identity Change UX").
//!
//! §42's own rule set: "Routine session-key rotation should be
//! invisible. A root/account identity change should trigger a strong
//! warning. A newly added device should trigger a security event on
//! existing trusted devices." This module is the UI-facing shape of
//! that rule — a plain, `siar-crypto`-independent state slice
//! (following this crate's own dependency rule: `siar-domain` only —
//! see `lib.rs`'s top doc comment) that a component can render as a
//! banner/list, populated by whatever background task translates real
//! `siar-crypto`/`siar-identity-multidevice` events (a `SecurityEpoch`
//! advance, a verified `DeviceRevocation`, a new `DeviceCertificate`)
//! into this crate's own event shape — the same `AppEvent`-mediated
//! boundary `command.rs`'s own doc comment already describes for every
//! other backend→UI event in this crate, applied here rather than
//! reinvented.
//!
//! §42's "routine rotation should be invisible" isn't a variant here at
//! all — there is deliberately no `SecurityEventKind::RoutineRotation`.
//! Routine, expected events are simply never turned into a
//! `SecurityEvent` by whatever populates this state; "invisible" means
//! "never constructed," not "constructed and then filtered by
//! severity."

use siar_domain::{AccountId, DeviceId};

/// §42's own three-tier severity, inferred from its own text rather
/// than invented independently: a routine key rotation is invisible
/// (§42, and not represented by this enum at all — see this module's
/// top doc), a new device is a `Notice` ("should trigger a security
/// event on existing trusted devices" — informational, not alarming, a
/// legitimate account owner adding their own second device is the
/// common case), and a root/account identity change is a
/// `StrongWarning` ("should trigger a strong warning" — the spec's own
/// words).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityEventSeverity {
    Notice,
    StrongWarning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityEventKind {
    /// A new device was linked to this account. §42: "should trigger a
    /// security event on existing trusted devices" — this is that
    /// event, meant to be shown on every *other* already-trusted device
    /// for the same account, not on the newly-linked device itself.
    NewDeviceLinked { device: DeviceId },
    /// A device was revoked — surfaced whether the local user initiated
    /// it (confirmation) or another device/the account owner did
    /// (something this device should know about).
    DeviceRevoked { device: DeviceId },
    /// The account's root identity itself changed — §42's own
    /// explicitly named "strong warning" case. This is a rare,
    /// high-stakes event (see `siar_identity_multidevice::root_key`'s
    /// own doc comment on what a root key rotation implies) — nothing
    /// about ordinary usage should ever produce this for an account
    /// that hasn't gone through a deliberate root recovery/rotation
    /// flow.
    RootIdentityChanged { account: AccountId },
}

impl SecurityEventKind {
    /// The severity this kind carries, per §42's own text — see this
    /// type's own doc comment for the reasoning behind each mapping.
    pub const fn severity(&self) -> SecurityEventSeverity {
        match self {
            Self::NewDeviceLinked { .. } | Self::DeviceRevoked { .. } => {
                SecurityEventSeverity::Notice
            }
            Self::RootIdentityChanged { .. } => SecurityEventSeverity::StrongWarning,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityEvent {
    pub kind: SecurityEventKind,
    pub occurred_at_millis: u64,
    /// Whether the local user has acknowledged/dismissed this event.
    /// `StrongWarning`-severity events are the ones a caller should
    /// treat as blocking until acknowledged — see
    /// `SecurityEventState::unacknowledged_strong_warnings`.
    pub acknowledged: bool,
}

/// Its own state slice for the same granular-re-render reason every
/// other `*State` type in this crate already is (see `lib.rs`'s doc
/// comment on plan.md §94) — a component rendering the security-event
/// banner shouldn't re-render on every incoming chat message, and vice
/// versa.
#[derive(Debug, Default)]
pub struct SecurityEventState {
    events: Vec<SecurityEvent>,
}

impl SecurityEventState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, kind: SecurityEventKind, occurred_at_millis: u64) {
        self.events.push(SecurityEvent {
            kind,
            occurred_at_millis,
            acknowledged: false,
        });
    }

    pub fn acknowledge(&mut self, index: usize) {
        if let Some(event) = self.events.get_mut(index) {
            event.acknowledged = true;
        }
    }

    pub fn events(&self) -> &[SecurityEvent] {
        &self.events
    }

    /// What a component should treat as blocking/must-show — every
    /// `StrongWarning` the user hasn't acknowledged yet. Deliberately
    /// not "every unacknowledged event" — an unacknowledged `Notice`
    /// (e.g. an unread "new device linked" banner) is fine to leave
    /// sitting in a dismissible list; §42 only asks for a *strong*
    /// warning on the identity-change case specifically.
    pub fn unacknowledged_strong_warnings(&self) -> impl Iterator<Item = &SecurityEvent> {
        self.events
            .iter()
            .filter(|e| !e.acknowledged && e.kind.severity() == SecurityEventSeverity::StrongWarning)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_device_and_revocation_are_notices_not_strong_warnings() {
        assert_eq!(
            SecurityEventKind::NewDeviceLinked { device: DeviceId::new() }.severity(),
            SecurityEventSeverity::Notice
        );
        assert_eq!(
            SecurityEventKind::DeviceRevoked { device: DeviceId::new() }.severity(),
            SecurityEventSeverity::Notice
        );
    }

    #[test]
    fn root_identity_change_is_a_strong_warning() {
        assert_eq!(
            SecurityEventKind::RootIdentityChanged { account: AccountId::new() }.severity(),
            SecurityEventSeverity::StrongWarning
        );
    }

    #[test]
    fn unacknowledged_strong_warnings_excludes_notices() {
        let mut state = SecurityEventState::new();
        state.push(SecurityEventKind::NewDeviceLinked { device: DeviceId::new() }, 1_000);
        state.push(SecurityEventKind::RootIdentityChanged { account: AccountId::new() }, 2_000);

        let warnings: Vec<_> = state.unacknowledged_strong_warnings().collect();
        assert_eq!(warnings.len(), 1);
        assert!(matches!(warnings[0].kind, SecurityEventKind::RootIdentityChanged { .. }));
    }

    #[test]
    fn acknowledging_a_strong_warning_removes_it_from_the_blocking_list() {
        let mut state = SecurityEventState::new();
        state.push(SecurityEventKind::RootIdentityChanged { account: AccountId::new() }, 1_000);
        assert_eq!(state.unacknowledged_strong_warnings().count(), 1);

        state.acknowledge(0);
        assert_eq!(state.unacknowledged_strong_warnings().count(), 0);
        assert!(state.events()[0].acknowledged);
    }

    #[test]
    fn events_preserve_insertion_order() {
        let mut state = SecurityEventState::new();
        let device_a = DeviceId::new();
        let device_b = DeviceId::new();
        state.push(SecurityEventKind::NewDeviceLinked { device: device_a }, 1_000);
        state.push(SecurityEventKind::NewDeviceLinked { device: device_b }, 2_000);

        assert_eq!(state.events().len(), 2);
        assert!(matches!(
            state.events()[0].kind,
            SecurityEventKind::NewDeviceLinked { device } if device == device_a
        ));
        assert!(matches!(
            state.events()[1].kind,
            SecurityEventKind::NewDeviceLinked { device } if device == device_b
        ));
    }
}
