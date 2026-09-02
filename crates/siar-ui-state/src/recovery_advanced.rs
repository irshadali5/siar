//! Recovery, part 2 (ui-ux-15 §74-90): drill, passphrase, backup
//! security states, the recovery-vs-backup distinction, the
//! lost-all-devices flow, and recovery QR.

use siar_domain::AccountId;

use crate::recovery::RecoveryStatus;

/// §74-76: "Test Recovery" without logging out. §76: "do not upload
/// recovery key, local validation where possible" — there is
/// deliberately no method anywhere on this type that sends anything
/// over a network; the absence of a network call *is* the property
/// §76 asks for, not something a flag toggles on or off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryDrillStep {
    EnterMaterial,
    ValidateLocally,
    Done,
}

#[derive(Debug)]
pub struct RecoveryDrillState {
    step: RecoveryDrillStep,
    passed: Option<bool>,
}

impl RecoveryDrillState {
    pub fn start() -> Self {
        Self {
            step: RecoveryDrillStep::EnterMaterial,
            passed: None,
        }
    }

    pub fn step(&self) -> RecoveryDrillStep {
        self.step
    }

    /// §75: "verify user still has recovery material, backup can be
    /// opened." `passed` is whatever the local validation (re-entering
    /// selected groups, or opening the local backup) actually
    /// determined — this type doesn't perform that validation itself,
    /// only tracks the outcome, the same translated-input shape every
    /// other `*State` type in this crate uses.
    pub fn record_local_validation(&mut self, passed: bool) {
        self.passed = Some(passed);
        self.step = RecoveryDrillStep::ValidateLocally;
    }

    pub fn finish(&mut self) {
        if self.passed.is_some() {
            self.step = RecoveryDrillStep::Done;
        }
    }

    pub fn passed(&self) -> Option<bool> {
        self.passed
    }
}

/// §77's own four-level informal scale name isn't given verbatim by
/// the spec (only "must have clear strength guidance" is stated) —
/// this is a reasonable minimal scale, not a claim of a specific
/// entropy-scoring algorithm. Actually scoring passphrase strength
/// (zxcvbn-style pattern/dictionary analysis) is a real algorithmic
/// choice this module deliberately doesn't make — `PassphraseStrength`
/// represents the *result* of whatever scoring a caller uses, not the
/// scoring itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PassphraseStrengthLevel {
    TooWeak,
    Weak,
    Reasonable,
    Strong,
}

/// §78: "Do not pretend passphrase is equivalent to recovery key. If
/// security differs, explain." `is_equivalent_to_recovery_key` is the
/// one thing this type exists to make explicit rather than implied —
/// a caller rendering passphrase setup must read this field and show
/// §78's required explanation whenever it's `false`, not assume
/// equivalence by default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassphraseAssessment {
    pub level: PassphraseStrengthLevel,
    pub is_equivalent_to_recovery_key: bool,
}

/// §80, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupSecurityState {
    Healthy,
    Stale,
    Missing,
    Unverified,
    Failed,
}

/// §83-84: "Account recovery ≠ backup decryption, unless system
/// explicitly makes them one." Two separate status fields — reusing
/// `RecoveryStatus` (§62, already built) for account recovery and
/// `BackupSecurityState` (§80, above) for backup, rather than
/// conflating them into one shared enum, is the type-level expression
/// of §83's own distinction. §81-82: `backup_key_is_distinct` is what
/// tells a caller whether to render the precise "Backup Recovery Key"
/// label (§82) or fold backup into the same recovery-key language as
/// account recovery, for products that intentionally unify them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryAndBackupOverview {
    pub account: AccountId,
    pub account_recovery_status: RecoveryStatus,
    pub backup_status: BackupSecurityState,
    pub backup_key_is_distinct: bool,
}

/// §85's three listed possible entry points into account restoration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LostAllDevicesMethod {
    RestoreAccount,
    ScanImportBackup,
    EnterRecoveryKey,
}

/// §85's own four steps, expanded with a `ChooseMethod` lead-in (the
/// spec's list starts with the method itself already chosen — a real
/// flow needs a step where that choice happens) and a terminal `Done`.
/// §86: "do not restore stale device session identity blindly" is
/// enforced the same way `RecoveryRotationFlow` (`recovery.rs`)
/// enforces §72-73 — `advance()` only moves through this exact order,
/// so `CreateFreshDeviceIdentity` cannot be skipped on the way to
/// `VerifyRecoveredState`/`Done`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LostAllDevicesStep {
    ChooseMethod,
    RestoreOrImport,
    CreateFreshDeviceIdentity,
    VerifyRecoveredState,
    Done,
}

impl LostAllDevicesStep {
    const ORDER: [Self; 5] = [
        Self::ChooseMethod,
        Self::RestoreOrImport,
        Self::CreateFreshDeviceIdentity,
        Self::VerifyRecoveredState,
        Self::Done,
    ];

    fn index(self) -> usize {
        Self::ORDER
            .iter()
            .position(|s| *s == self)
            .expect("all variants are in ORDER")
    }
}

/// §87: "user enters recovery material into local trusted UI only" —
/// like §76 above, this is an absence-of-network-call property, not a
/// flag; there is no method on this type that transmits `method` or
/// any recovery material anywhere.
#[derive(Debug)]
pub struct LostAllDevicesFlow {
    method: Option<LostAllDevicesMethod>,
    step: LostAllDevicesStep,
}

impl LostAllDevicesFlow {
    pub fn start() -> Self {
        Self {
            method: None,
            step: LostAllDevicesStep::ChooseMethod,
        }
    }

    pub fn step(&self) -> LostAllDevicesStep {
        self.step
    }

    pub fn choose_method(&mut self, method: LostAllDevicesMethod) {
        self.method = Some(method);
        self.step = LostAllDevicesStep::RestoreOrImport;
    }

    /// Advances through the remaining fixed order. Like
    /// `RecoveryRotationFlow::advance`, there is no way to reach
    /// `Done`/`VerifyRecoveredState` without having passed through
    /// `CreateFreshDeviceIdentity` first.
    pub fn advance(&mut self) {
        let next_index = self.step.index() + 1;
        if let Some(next) = LostAllDevicesStep::ORDER.get(next_index) {
            self.step = *next;
        }
    }

    pub fn is_done(&self) -> bool {
        self.step == LostAllDevicesStep::Done
    }

    /// §86's own property, made checkable: true once
    /// `CreateFreshDeviceIdentity` has actually been reached — a
    /// caller can assert this before ever considering the recovered
    /// account "live."
    pub fn fresh_identity_created(&self) -> bool {
        self.step.index() >= LostAllDevicesStep::CreateFreshDeviceIdentity.index()
    }
}

/// §89-90: "if supported for transfer, short-lived and high-risk. Do
/// not share this code." `is_expired` is the enforcement half of
/// "short-lived"; the "do not share" warning itself (§90) is a display
/// string, not state — left to whatever renders this, same as every
/// other UI-copy-only requirement in this spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryQrState {
    pub generated_at_millis: u64,
    pub expires_at_millis: u64,
}

impl RecoveryQrState {
    pub fn is_expired(&self, now_millis: u64) -> bool {
        now_millis >= self.expires_at_millis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_drill_records_a_passed_result() {
        let mut drill = RecoveryDrillState::start();
        assert_eq!(drill.passed(), None);
        drill.record_local_validation(true);
        assert_eq!(drill.passed(), Some(true));
        drill.finish();
        assert_eq!(drill.step(), RecoveryDrillStep::Done);
    }

    #[test]
    fn recovery_drill_can_record_a_failed_result() {
        let mut drill = RecoveryDrillState::start();
        drill.record_local_validation(false);
        assert_eq!(drill.passed(), Some(false));
    }

    #[test]
    fn passphrase_assessment_flags_non_equivalence_explicitly() {
        let assessment = PassphraseAssessment {
            level: PassphraseStrengthLevel::Reasonable,
            is_equivalent_to_recovery_key: false,
        };
        assert!(!assessment.is_equivalent_to_recovery_key);
    }

    #[test]
    fn account_recovery_and_backup_status_are_independent_fields() {
        let overview = RecoveryAndBackupOverview {
            account: AccountId::new(),
            account_recovery_status: RecoveryStatus::Configured,
            backup_status: BackupSecurityState::Stale,
            backup_key_is_distinct: true,
        };
        // Account recovery can be healthy while backup is stale, or
        // vice versa — the two fields must never be collapsed into one.
        assert_eq!(overview.account_recovery_status, RecoveryStatus::Configured);
        assert_eq!(overview.backup_status, BackupSecurityState::Stale);
    }

    #[test]
    fn lost_all_devices_flow_cannot_reach_verify_before_fresh_identity() {
        let mut flow = LostAllDevicesFlow::start();
        flow.choose_method(LostAllDevicesMethod::EnterRecoveryKey);
        assert!(!flow.fresh_identity_created());

        flow.advance(); // RestoreOrImport -> CreateFreshDeviceIdentity
        assert_eq!(flow.step(), LostAllDevicesStep::CreateFreshDeviceIdentity);
        assert!(flow.fresh_identity_created());

        flow.advance(); // -> VerifyRecoveredState
        assert!(flow.fresh_identity_created());
    }

    #[test]
    fn lost_all_devices_flow_reaches_done() {
        let mut flow = LostAllDevicesFlow::start();
        flow.choose_method(LostAllDevicesMethod::ScanImportBackup);
        flow.advance();
        flow.advance();
        flow.advance();
        assert!(flow.is_done());
    }

    #[test]
    fn recovery_qr_expires_at_the_stated_time() {
        let qr = RecoveryQrState {
            generated_at_millis: 1_000,
            expires_at_millis: 2_000,
        };
        assert!(!qr.is_expired(1_500));
        assert!(qr.is_expired(2_000));
        assert!(qr.is_expired(3_000));
    }
}
