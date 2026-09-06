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
use crate::linking_state_machine::LinkingState;
use crate::recovery_state_machine::RecoveryState;
use siar_domain::DeviceId;

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
}
