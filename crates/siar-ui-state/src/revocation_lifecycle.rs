//! Revocation lifecycle, local/remote wipe, and lost-device messaging
//! (ui-ux-15 §120-128).

use siar_domain::DeviceId;

/// §122, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationState {
    Active,
    Pending,
    Complete,
    Failed,
}

/// §121/§123/§124: a revocation in progress. §124's own rule —
/// "do not pretend success" — is why `Failed` is a real, distinct,
/// terminal state rather than something that quietly resolves to
/// `Complete` after a timeout: a caller that never calls
/// `mark_failed()` on a genuine failure is the only way this type
/// could misrepresent what happened, not anything this type does on
/// its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevocationProgress {
    pub device: DeviceId,
    pub state: RevocationState,
    /// §123: "high-risk operation may require connectivity to
    /// authoritative peers/server... if architecture supports offline
    /// signed revocation, show pending propagation." Distinct from
    /// `state == Pending` alone — a revocation can be `Pending` because
    /// it's simply in flight over an active connection, or `Pending`
    /// *and* `offline_pending` because it was signed offline and is
    /// waiting on propagation once connectivity returns. The two cases
    /// warrant different display copy (`display_label` below).
    pub offline_pending: bool,
}

impl RevocationProgress {
    pub fn start(device: DeviceId) -> Self {
        Self {
            device,
            state: RevocationState::Pending,
            offline_pending: false,
        }
    }

    pub fn mark_offline_pending(&mut self) {
        self.offline_pending = true;
    }

    pub fn mark_complete(&mut self) {
        self.state = RevocationState::Complete;
    }

    pub fn mark_failed(&mut self) {
        self.state = RevocationState::Failed;
    }

    /// §121's exact two-phase copy ("Revoking…" → "Revoked"), §123's
    /// offline-pending variant, §124's exact failure copy. No arm here
    /// ever says "Revoked" for anything but `Complete` — the one
    /// property §124 actually cares about.
    pub fn display_label(&self) -> &'static str {
        match (self.state, self.offline_pending) {
            (RevocationState::Active, _) => "Active",
            (RevocationState::Pending, true) => "Pending (will complete when connected)",
            (RevocationState::Pending, false) => "Revoking…",
            (RevocationState::Complete, _) => "Revoked",
            (RevocationState::Failed, _) => "Could not revoke device",
        }
    }
}

/// §127: "do not promise remote wipe unless platform/backend actually
/// supports it." A caller-supplied fact about what's actually
/// available — never assumed `LocalAndRemoteWipe` by default, since
/// that default would be exactly the false promise §127 warns against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WipeCapability {
    LocalWipeOnly,
    LocalAndRemoteWipe,
}

/// §128: "say 'Revoke access,' not 'Wipe device,' unless true." The
/// wording is a direct function of `WipeCapability` — there is no path
/// to the stronger label without the capability actually being
/// `LocalAndRemoteWipe`.
pub const fn lost_device_action_label(capability: WipeCapability) -> &'static str {
    match capability {
        WipeCapability::LocalWipeOnly => "Revoke access",
        WipeCapability::LocalAndRemoteWipe => "Revoke access and wipe device",
    }
}

/// §125-126: local wipe of the *current* device's own secrets. §125:
/// wipe may happen only after a successful account-state update
/// (i.e., after `RevocationProgress` reaches `Complete` for this
/// device) — `mark_revocation_confirmed` is the caller's attestation
/// of exactly that, not something this type checks against a
/// `RevocationProgress` itself (keeping the two types independently
/// usable rather than coupling local-wipe logic to revocation's
/// specific representation). §126: "requires separate explicit
/// action" — a second, independent flag, not implied by the
/// revocation confirmation alone.
#[derive(Debug, Default)]
pub struct LocalWipeConfirmation {
    revocation_confirmed: bool,
    explicit_wipe_action_taken: bool,
}

impl LocalWipeConfirmation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_revocation_confirmed(&mut self) {
        self.revocation_confirmed = true;
    }

    pub fn mark_explicit_wipe_requested(&mut self) {
        self.explicit_wipe_action_taken = true;
    }

    pub fn can_wipe_locally(&self) -> bool {
        self.revocation_confirmed && self.explicit_wipe_action_taken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revocation_progresses_through_the_two_phase_labels() {
        let mut progress = RevocationProgress::start(DeviceId::new());
        assert_eq!(progress.display_label(), "Revoking…");
        progress.mark_complete();
        assert_eq!(progress.display_label(), "Revoked");
    }

    #[test]
    fn offline_pending_revocation_gets_its_own_label() {
        let mut progress = RevocationProgress::start(DeviceId::new());
        progress.mark_offline_pending();
        assert_eq!(
            progress.display_label(),
            "Pending (will complete when connected)"
        );
    }

    #[test]
    fn a_failed_revocation_never_reports_revoked() {
        let mut progress = RevocationProgress::start(DeviceId::new());
        progress.mark_failed();
        assert_eq!(progress.display_label(), "Could not revoke device");
        assert_ne!(progress.display_label(), "Revoked");
    }

    #[test]
    fn lost_device_label_never_promises_wipe_without_the_capability() {
        assert_eq!(
            lost_device_action_label(WipeCapability::LocalWipeOnly),
            "Revoke access"
        );
        assert_eq!(
            lost_device_action_label(WipeCapability::LocalAndRemoteWipe),
            "Revoke access and wipe device"
        );
    }

    #[test]
    fn local_wipe_requires_both_confirmation_and_explicit_action() {
        let mut confirmation = LocalWipeConfirmation::new();
        assert!(!confirmation.can_wipe_locally());

        confirmation.mark_revocation_confirmed();
        assert!(!confirmation.can_wipe_locally());

        confirmation.mark_explicit_wipe_requested();
        assert!(confirmation.can_wipe_locally());
    }

    #[test]
    fn explicit_wipe_action_alone_without_revocation_confirmed_is_not_enough() {
        let mut confirmation = LocalWipeConfirmation::new();
        confirmation.mark_explicit_wipe_requested();
        assert!(!confirmation.can_wipe_locally());
    }
}
