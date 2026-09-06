//! §160 "Trust Assumptions": spec's own two named lists, verbatim.
//!
//! Not new mechanism — a traceability map, same approach
//! [`crate::threat_model`] uses for §159. Each variant below names one
//! of spec's inputs and points at the actual code that treats it as
//! trusted or untrusted; nothing here changes behavior, since every
//! one of these boundaries was already drawn by an earlier round.

/// §160's three trusted foundations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustedInput {
    CryptographicVerification,
    ConfiguredAuthorizationPolicy,
    SecureStorageGuarantees,
}

/// §160's six explicitly untrusted inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UntrustedInput {
    Relay,
    Lan,
    BluetoothPeerName,
    IpAddress,
    ServerDatabase,
    DeviceDisplayName,
}

impl TrustedInput {
    pub fn enforced_by(&self) -> &'static str {
        match self {
            TrustedInput::CryptographicVerification => {
                "directory::DeviceDirectory::verify_signature, certificate::DeviceCertificate::verify_signature, root_rotation::verify_root_rotation"
            }
            TrustedInput::ConfiguredAuthorizationPolicy => {
                "linking_authority::LinkingAuthorityPolicy, enterprise_policy::EnterpriseDevicePolicy, device_authorization::DeviceAuthorizationDecision"
            }
            TrustedInput::SecureStorageGuarantees => {
                "secure_storage::SecureStore (a trait boundary this crate trusts an implementation to honor, never verifies itself)"
            }
        }
    }
}

impl UntrustedInput {
    pub fn why_not_trusted(&self) -> &'static str {
        match self {
            // A relay only ever carries bytes this crate independently
            // verifies — see state_transport::SignedDeviceStateUpdate
            // and contact_verification::DirectoryServiceResponse.
            UntrustedInput::Relay => {
                "state_transport::SignedDeviceStateUpdate::verify / contact_verification::DirectoryServiceResponse::verify_and_accept never trust the carrier"
            }
            UntrustedInput::Lan => {
                "no LAN-address field anywhere is treated as identity — discovery_privacy::RotatingDiscoveryToken exists precisely so LAN presence never implies device identity"
            }
            UntrustedInput::BluetoothPeerName => {
                "device_privacy_presentation::GenericDeviceDescriptor never derives from any advertised peer name; a display name is not a certificate"
            }
            UntrustedInput::IpAddress => {
                "directory::DeviceEndpoint carries opaque, signed-directory-scoped bytes, never a bare address treated as authoritative on its own"
            }
            UntrustedInput::ServerDatabase => {
                "contact_verification::DirectoryServiceResponse::verify_and_accept — a directory service's own response is verified exactly like any other untrusted source"
            }
            UntrustedInput::DeviceDisplayName => {
                "discovery_privacy::PrivateDeviceMetadata::friendly_name never participates in any trust or verification decision anywhere in this crate"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_trusted_input_has_a_named_enforcement_point() {
        for input in [
            TrustedInput::CryptographicVerification,
            TrustedInput::ConfiguredAuthorizationPolicy,
            TrustedInput::SecureStorageGuarantees,
        ] {
            assert!(!input.enforced_by().is_empty());
        }
    }

    #[test]
    fn every_untrusted_input_has_a_named_reason() {
        for input in [
            UntrustedInput::Relay,
            UntrustedInput::Lan,
            UntrustedInput::BluetoothPeerName,
            UntrustedInput::IpAddress,
            UntrustedInput::ServerDatabase,
            UntrustedInput::DeviceDisplayName,
        ] {
            assert!(!input.why_not_trusted().is_empty());
        }
    }
}
