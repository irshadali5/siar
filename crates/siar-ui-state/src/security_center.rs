//! Security Center — Overview and Devices sections (ui-ux-15 §3-25).
//!
//! §1's governing principle, quoted directly because it's the reason
//! every type here is a plain, translated view rather than a re-export
//! of a `siar-crypto`/`siar-identity-multidevice` type: "The UI
//! explains security consequences; Rust owns cryptographic truth, key
//! state, authorization, revocation, and recovery validation." This
//! crate's own dependency rule (`siar-domain` only — see `lib.rs`'s top
//! doc, and `security_event.rs`'s doc comment for the same reasoning
//! applied to Part 28 §42) means `DeviceTrustState` here is a distinct
//! type from `siar_crypto::PeerTrustState`, not an alias or a re-export
//! — deliberately: this crate's `Compromised` variant, for instance, is
//! a *display* classification a caller derives from combining a
//! `PeerTrustState::Revoked` with a `RevocationReason::SuspectedCompromise`
//! (see `siar_crypto::revocation`), not a state `siar-crypto` itself
//! tracks. Keeping that derivation outside this crate is what §1's
//! principle actually asks for — the UI layer explains, it doesn't
//! decide.
//!
//! Scoped to §3's first two of eight Security Center sections
//! (Overview, Devices) this round — Identity & Verification, Recovery,
//! Backups, Security Events (already covered separately by
//! `security_event.rs`), Privacy, and Advanced are further sections of
//! the same spec, not attempted here.

use siar_domain::DeviceId;

/// §5, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityHealth {
    Healthy,
    Attention,
    Critical,
    Unknown,
}

/// §12, verbatim — "informational only" per the spec's own note, so
/// this has no behavior beyond being a display label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    AndroidPhone,
    AndroidTablet,
    Desktop,
    Laptop,
    ServerNode,
    Unknown,
}

/// §13, verbatim. See this module's own top doc comment for why
/// `Compromised` is a UI-layer classification derived from
/// `siar-crypto` state elsewhere, not tracked here directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceTrustState {
    Trusted,
    Pending,
    Revoked,
    Compromised,
    Unknown,
}

/// §11 references this type but the spec gives no enumeration of
/// values anywhere in this document — this is a minimal, honestly-
/// scoped starting set covering the concrete flag-worthy conditions
/// the surrounding sections (§7-8, §25+) already name, not a guess at
/// an exhaustive list the spec doesn't actually specify. Extending
/// this later (a real, spec-named flag this round missed) is a small,
/// additive change — adding variants to a `#[non_exhaustive]`-style
/// enum a UI already renders generically shouldn't require touching
/// every match site, so this enum is intentionally kept to variants a
/// caller would want to branch on individually.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceSecurityFlag {
    /// §8 Critical: "identity compromise suspected."
    SuspectedCompromise,
    /// §21: authorization not yet complete.
    PendingAuthorization,
    /// §7 Attention: "new device not reviewed."
    UnreviewedSinceLinking,
}

/// §11, field-for-field, with `Timestamp` following this workspace's
/// existing convention of caller-supplied `u64` millis (same as
/// `siar-crypto::DeviceRevocation::revoked_at_millis`) rather than a
/// dedicated `Timestamp` type — none exists elsewhere in this
/// workspace, and inventing one here for a single struct would be the
/// kind of unrequested addition this session's earlier rounds have
/// consistently avoided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceSecurityView {
    pub id: DeviceId,
    pub display_name: String,
    pub kind: DeviceKind,
    pub status: DeviceTrustState,
    pub added_at_millis: u64,
    pub last_active_millis: Option<u64>,
    pub current_device: bool,
    pub security_flags: Vec<DeviceSecurityFlag>,
}

/// §14's four list sections, computed from a flat device list rather
/// than stored as four separate lists — a device only ever belongs to
/// exactly one of these at a time (its `DeviceTrustState`), so keeping
/// one source of truth and grouping on read avoids the four-lists-that-
/// could-disagree problem a caller would otherwise have to keep in
/// sync by hand.
#[derive(Debug, Default)]
pub struct DeviceListState {
    devices: Vec<DeviceSecurityView>,
}

impl DeviceListState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_devices(&mut self, devices: Vec<DeviceSecurityView>) {
        self.devices = devices;
    }

    pub fn all(&self) -> &[DeviceSecurityView] {
        &self.devices
    }

    /// §16: "clearly label: this device." At most one device should
    /// ever have `current_device: true` — this returns the first if a
    /// caller's data violates that, rather than panicking on a UI-layer
    /// read path.
    pub fn this_device(&self) -> Option<&DeviceSecurityView> {
        self.devices.iter().find(|d| d.current_device)
    }

    pub fn trusted(&self) -> impl Iterator<Item = &DeviceSecurityView> {
        self.devices
            .iter()
            .filter(|d| !d.current_device && d.status == DeviceTrustState::Trusted)
    }

    pub fn pending(&self) -> impl Iterator<Item = &DeviceSecurityView> {
        self.devices
            .iter()
            .filter(|d| d.status == DeviceTrustState::Pending)
    }

    /// §14's "Revoked / History" — includes both `Revoked` and
    /// `Compromised`, since both are past-tense/no-longer-active states
    /// a user would look for under the same history section rather than
    /// two separate lists for what's functionally one "no longer has
    /// access" bucket.
    pub fn revoked_or_history(&self) -> impl Iterator<Item = &DeviceSecurityView> {
        self.devices.iter().filter(|d| {
            matches!(
                d.status,
                DeviceTrustState::Revoked | DeviceTrustState::Compromised
            )
        })
    }

    pub fn trusted_count(&self) -> usize {
        self.trusted().count() + usize::from(self.this_device().is_some())
    }
}

/// §4/§10's overview — "is there anything I need to act on?" A caller
/// (the Security Center's top-level screen) assembles this from
/// `DeviceListState` plus whatever recovery/backup state lives in the
/// sections this round doesn't cover, rather than this type reaching
/// into those directly — same one-way, translated-input shape as every
/// other `*State` type in this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityOverview {
    pub health: SecurityHealth,
    pub trusted_device_count: usize,
    pub has_unreviewed_new_device: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_device(status: DeviceTrustState, current: bool) -> DeviceSecurityView {
        DeviceSecurityView {
            id: DeviceId::new(),
            display_name: "Test Device".to_string(),
            kind: DeviceKind::Desktop,
            status,
            added_at_millis: 1_700_000_000_000,
            last_active_millis: Some(1_700_000_100_000),
            current_device: current,
            security_flags: Vec::new(),
        }
    }

    #[test]
    fn this_device_finds_the_current_device() {
        let mut state = DeviceListState::new();
        let current = sample_device(DeviceTrustState::Trusted, true);
        let current_id = current.id;
        state.set_devices(vec![
            sample_device(DeviceTrustState::Trusted, false),
            current,
        ]);
        assert_eq!(state.this_device().unwrap().id, current_id);
    }

    #[test]
    fn trusted_excludes_the_current_device() {
        let mut state = DeviceListState::new();
        state.set_devices(vec![
            sample_device(DeviceTrustState::Trusted, true),
            sample_device(DeviceTrustState::Trusted, false),
            sample_device(DeviceTrustState::Trusted, false),
        ]);
        // 2 other trusted devices, not counting "this device".
        assert_eq!(state.trusted().count(), 2);
    }

    #[test]
    fn trusted_count_includes_this_device() {
        let mut state = DeviceListState::new();
        state.set_devices(vec![
            sample_device(DeviceTrustState::Trusted, true),
            sample_device(DeviceTrustState::Trusted, false),
        ]);
        // "This device" + 1 other trusted device = 2, matching §10's
        // own example ("4 trusted devices" presumably counts the
        // device showing the screen too).
        assert_eq!(state.trusted_count(), 2);
    }

    #[test]
    fn revoked_and_compromised_both_land_in_history() {
        let mut state = DeviceListState::new();
        state.set_devices(vec![
            sample_device(DeviceTrustState::Revoked, false),
            sample_device(DeviceTrustState::Compromised, false),
            sample_device(DeviceTrustState::Trusted, false),
        ]);
        assert_eq!(state.revoked_or_history().count(), 2);
    }

    #[test]
    fn pending_devices_are_tracked_separately_from_trusted() {
        let mut state = DeviceListState::new();
        state.set_devices(vec![
            sample_device(DeviceTrustState::Pending, false),
            sample_device(DeviceTrustState::Trusted, false),
        ]);
        assert_eq!(state.pending().count(), 1);
        assert_eq!(state.trusted().count(), 1);
    }

    /// ui-ux-15 §120: "Do not show stale 'trusted' device after
    /// revocation." Not new logic — proving a property this type
    /// already had, since `set_devices` is a full replace and
    /// `trusted()` filters strictly on `status == Trusted`, so a
    /// device that transitioned to `Revoked` in the caller's data
    /// simply doesn't match that filter on the next read. This test
    /// exists to make that guarantee explicit and regression-checked,
    /// not to add behavior.
    #[test]
    fn a_device_that_transitions_to_revoked_no_longer_appears_trusted() {
        let mut state = DeviceListState::new();
        let device = sample_device(DeviceTrustState::Trusted, false);
        let device_id = device.id;
        state.set_devices(vec![device]);
        assert_eq!(state.trusted().count(), 1);

        let mut revoked = sample_device(DeviceTrustState::Revoked, false);
        revoked.id = device_id;
        state.set_devices(vec![revoked]);

        assert_eq!(state.trusted().count(), 0);
        assert_eq!(state.revoked_or_history().count(), 1);
    }
}
