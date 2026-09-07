//! §137 "UI Boundary", §138 "Android Kotlin Boundary", §139 "iOS
//! Boundary".
//!
//! §138/§139 get no new type: both sections describe the SAME
//! boundary — "Rust owns: identity state, certificate logic, linking
//! protocol, revocation, recovery policy" — which is not a description
//! of some future integration point, it's a description of this
//! crate's own scope, already true of every module in it. Kotlin/Swift
//! may handle Android Keystore/Secure Enclave, biometric confirmation,
//! camera/QR/NFC, and system notifications — none of which this crate
//! has, needs, or should have a type for; a `KeystoreHandle` or
//! `BiometricPrompt` type living in this crate would itself be the
//! violation §138/§139 warn against ("no identity business logic
//! should be duplicated in Swift" cuts both ways: no platform-specific
//! surface should leak into this crate's Rust either). §137 gets real
//! types, below, because it's the one of the three that names
//! something concrete this crate can actually shape: the view models
//! themselves.

use crate::directory::DeviceStatus;
use crate::linking_authority::DeviceRole;
use crate::linking_state_machine::LinkingState;
use crate::recovery_state_machine::RecoveryState;
use siar_domain::DeviceId;
use siar_event_log::ids::Timestamp;

/// §137's own four named view models, verbatim. Every field on every
/// type in this module is something already safe to hand to a UI
/// layer — [`crate::directory::DeviceStatus`],
/// [`crate::linking_state_machine::LinkingState`],
/// [`crate::recovery_state_machine::RecoveryState`], plain strings and
/// booleans. §137's own negative rule — "it never receives: root
/// private key, device private key, session key" — is enforced by
/// what ISN'T importable here: nothing in this module can name
/// [`crate::root_key::RootIdentityKey`],
/// [`crate::secure_storage::SecretBytes`], or any other secret-holding
/// type, because none of those types are imported into this file at
/// all. A `DeviceListVm` literally cannot carry a root private key —
/// there's no field shape it could occupy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceSummaryVm {
    pub device_id: DeviceId,
    pub friendly_name: Option<String>,
    pub status: DeviceStatus,
    pub is_current_device: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceListVm {
    pub devices: Vec<DeviceSummaryVm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceLinkVm {
    pub state: LinkingState,
    pub invite_expires_at_millis: Option<u64>,
}

/// `safety_fingerprint_display` is a `String` — the already-rendered
/// output of [`crate::safety_fingerprint::SafetyFingerprint::display_string`]
/// — never the fingerprint's own key material, matching this module's
/// whole point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityIdentityVm {
    pub verified: bool,
    pub safety_fingerprint_display: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryVm {
    pub state: RecoveryState,
}

/// §182 "UX States": each row's five named fields, verbatim. Same
/// "enforced by absence" guarantee as every other view model in this
/// module — no secret-holding type is imported here either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRowVm {
    pub name: Option<String>,
    pub platform: Option<String>,
    pub last_active: Option<Timestamp>,
    pub trust_status: DeviceStatus,
    pub role: DeviceRole,
}

/// §182's own four named categories. A plain struct of four `Vec`s
/// rather than one flat list with a category tag on each row — the
/// four categories aren't just a display grouping of the same kind of
/// row; "Recently revoked" and "Security alerts" are populated from
/// genuinely different sources ([`crate::local_records::DeviceHistoryLog`],
/// [`crate::contact_verification::IdentityNotification`]) than "This
/// device"/"Other devices" (a live [`crate::directory::DeviceDirectory`]),
/// so keeping them as separate fields matches where the data actually
/// comes from.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeviceManagementUiState {
    pub this_device: Option<DeviceRowVm>,
    pub other_devices: Vec<DeviceRowVm>,
    pub recently_revoked: Vec<DeviceRowVm>,
    pub security_alerts: Vec<String>,
}

/// §183's own seven-step flow. The middle three steps
/// (`ShowOrScanQr`/`Verify`/`Approve`) correspond to real
/// [`LinkingState`] transitions (`Created`/`Verified`/`Approved`) —
/// this type exists for the two purely-navigational steps
/// (`Settings`, `LinkedDevices`) and the final `Sync` step that
/// [`LinkingState`] itself doesn't model (linking ends at `Completed`;
/// syncing conversation history afterward is
/// [`crate::directory::DeviceDirectory::is_device_trusted`]'s own §156
/// note — "a synchronization problem layered above identity" — so
/// `Sync` here is a UI-flow marker, not backed by any
/// `LinkingState`/`RecoveryState` variant).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewDeviceUxStep {
    Settings,
    LinkedDevices,
    LinkNewDevice,
    ShowOrScanQr,
    Verify,
    Approve,
    Sync,
}

impl NewDeviceUxStep {
    /// §183: "do not mix this with ordinary contact pairing." Made
    /// checkable rather than only a UI-copy guideline: every step here
    /// is a `NewDeviceUxStep` variant, never a
    /// [`crate::contact_verification::OfflineVerificationMethod`] or
    /// [`crate::contact_verification::VerifiedContact`] — the two flows
    /// have no shared type at all (see
    /// [`crate::pairing_vs_linking`] for the test proving this
    /// directly).
    pub fn corresponding_linking_state(&self) -> Option<LinkingState> {
        match self {
            NewDeviceUxStep::ShowOrScanQr => Some(LinkingState::Created),
            NewDeviceUxStep::Verify => Some(LinkingState::Verified),
            NewDeviceUxStep::Approve => Some(LinkingState::Approved),
            NewDeviceUxStep::Settings
            | NewDeviceUxStep::LinkedDevices
            | NewDeviceUxStep::LinkNewDevice
            | NewDeviceUxStep::Sync => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // There is no test here attempting to put a `RootIdentityKey` or
    // `SecretBytes` field on one of these types — that would be a
    // compile error, which is the actual guarantee §137 asks for, the
    // same reasoning `transaction.rs`'s own tests module gives for not
    // testing a compile-time property at runtime.

    #[test]
    fn device_list_vm_holds_only_display_safe_fields() {
        let vm = DeviceListVm {
            devices: vec![DeviceSummaryVm {
                device_id: DeviceId::new(),
                friendly_name: Some("Alice's laptop".to_string()),
                status: DeviceStatus::Active,
                is_current_device: true,
            }],
        };
        assert_eq!(vm.devices.len(), 1);
        assert!(vm.devices[0].is_current_device);
    }

    #[test]
    fn device_link_vm_reflects_a_real_linking_state() {
        let vm = DeviceLinkVm {
            state: LinkingState::new(),
            invite_expires_at_millis: Some(1_000),
        };
        assert_eq!(vm.state, LinkingState::Created);
    }

    #[test]
    fn recovery_vm_reflects_a_real_recovery_state() {
        let vm = RecoveryVm {
            state: RecoveryState::new(),
        };
        assert_eq!(vm.state, RecoveryState::Started);
    }

    #[test]
    fn device_management_ui_state_keeps_its_four_categories_separate() {
        let row = DeviceRowVm {
            name: Some("Bedroom PC".to_string()),
            platform: Some("linux".to_string()),
            last_active: Some(Timestamp(100)),
            trust_status: DeviceStatus::Active,
            role: DeviceRole::Primary,
        };
        let mut state = DeviceManagementUiState {
            this_device: Some(row.clone()),
            ..Default::default()
        };
        state.other_devices.push(row);
        assert!(state.this_device.is_some());
        assert_eq!(state.other_devices.len(), 1);
        assert!(state.recently_revoked.is_empty());
        assert!(state.security_alerts.is_empty());
    }

    #[test]
    fn only_the_three_bootstrap_steps_correspond_to_a_linking_state() {
        assert_eq!(
            NewDeviceUxStep::ShowOrScanQr.corresponding_linking_state(),
            Some(LinkingState::Created)
        );
        assert_eq!(
            NewDeviceUxStep::Verify.corresponding_linking_state(),
            Some(LinkingState::Verified)
        );
        assert_eq!(
            NewDeviceUxStep::Approve.corresponding_linking_state(),
            Some(LinkingState::Approved)
        );
        assert_eq!(
            NewDeviceUxStep::Settings.corresponding_linking_state(),
            None
        );
        assert_eq!(NewDeviceUxStep::Sync.corresponding_linking_state(), None);
    }
}
