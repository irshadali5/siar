//! §170 "Protocol Extension Integration", §171 "Capability Negotiation
//! Integration".

use crate::capability::DeviceCapabilitySet;
use crate::directory::DeviceDirectoryEntry;

/// §170's own five-step handshake, verbatim order. "Identity is
/// therefore foundational" is made structural the same way
/// [`crate::transaction`]'s device-addition sequence and
/// [`crate::linking_state_machine::LinkingState`] already are: a
/// guarded `advance` that only permits moving forward through this
/// exact order, so an application wiring `siar-protocol-ext`'s
/// extension negotiation together with this crate's identity binding
/// cannot accidentally negotiate extensions before identity is
/// verified — the type itself refuses that call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakePhase {
    CoreTransport,
    IdentityBinding,
    AccountDeviceVerification,
    ExtensionNegotiation,
    ApplicationLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("cannot advance handshake from {from:?} to {to:?}")]
pub struct InvalidHandshakeTransition {
    pub from: HandshakePhase,
    pub to: HandshakePhase,
}

impl HandshakePhase {
    pub fn new() -> Self {
        HandshakePhase::CoreTransport
    }

    pub fn advance(self, to: HandshakePhase) -> Result<HandshakePhase, InvalidHandshakeTransition> {
        use HandshakePhase::*;
        let valid = matches!(
            (self, to),
            (CoreTransport, IdentityBinding)
                | (IdentityBinding, AccountDeviceVerification)
                | (AccountDeviceVerification, ExtensionNegotiation)
                | (ExtensionNegotiation, ApplicationLayer)
        );
        if valid {
            Ok(to)
        } else {
            Err(InvalidHandshakeTransition { from: self, to })
        }
    }

    /// §171's own rule made checkable at the type level: extension
    /// negotiation — and by extension (no pun intended), capability
    /// negotiation, since §171 says capabilities and extensions travel
    /// together — is only reachable once identity has actually been
    /// verified.
    pub fn identity_is_verified(&self) -> bool {
        matches!(
            self,
            HandshakePhase::ExtensionNegotiation | HandshakePhase::ApplicationLayer
        )
    }
}

impl Default for HandshakePhase {
    fn default() -> Self {
        Self::new()
    }
}

/// §171: "capabilities must be bound to authenticated device identity.
/// Do not trust anonymous capability advertisements for
/// authorization." The only constructor is
/// [`AuthenticatedCapabilityAdvertisement::from_directory_entry`] —
/// there is no way to build one from a bare, wire-supplied
/// `DeviceCapabilitySet` a peer merely claims; the capabilities always
/// come from the certificate a [`DeviceDirectoryEntry`] already
/// carries; a peer advertising capabilities it wasn't actually issued
/// simply has no path to producing one of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthenticatedCapabilityAdvertisement {
    capabilities: DeviceCapabilitySet,
}

impl AuthenticatedCapabilityAdvertisement {
    pub fn from_directory_entry(entry: &DeviceDirectoryEntry) -> Self {
        Self {
            capabilities: entry.certificate.capabilities,
        }
    }

    pub fn capabilities(&self) -> DeviceCapabilitySet {
        self.capabilities
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::DeviceCertificate;
    use crate::directory::DeviceStatus;
    use crate::root_key::RootIdentityKey;
    use siar_domain::{AccountId, DeviceId};

    #[test]
    fn handshake_cannot_skip_identity_verification_to_reach_extension_negotiation() {
        let result = HandshakePhase::new().advance(HandshakePhase::ExtensionNegotiation);
        assert!(result.is_err());
    }

    #[test]
    fn handshake_walked_in_order_reaches_extension_negotiation_with_identity_verified() {
        let phase = HandshakePhase::new()
            .advance(HandshakePhase::IdentityBinding)
            .unwrap()
            .advance(HandshakePhase::AccountDeviceVerification)
            .unwrap()
            .advance(HandshakePhase::ExtensionNegotiation)
            .unwrap();
        assert!(phase.identity_is_verified());
    }

    #[test]
    fn early_phases_report_identity_not_yet_verified() {
        assert!(!HandshakePhase::CoreTransport.identity_is_verified());
        assert!(!HandshakePhase::IdentityBinding.identity_is_verified());
        assert!(!HandshakePhase::AccountDeviceVerification.identity_is_verified());
    }

    #[test]
    fn capability_advertisement_only_ever_reflects_the_signed_certificate() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();
        let real_capabilities = DeviceCapabilitySet::SEND_MESSAGE;
        let certificate = DeviceCertificate::issue(
            &root,
            account,
            device_id,
            [0u8; 32],
            0,
            None,
            real_capabilities,
            1,
        );
        let entry = DeviceDirectoryEntry {
            device_id,
            certificate,
            status: DeviceStatus::Active,
            transport_endpoints: vec![],
        };

        let advertisement = AuthenticatedCapabilityAdvertisement::from_directory_entry(&entry);
        assert_eq!(advertisement.capabilities(), real_capabilities);
    }
}
