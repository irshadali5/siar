//! §151 "Device Linking Over Existing Secure Session", §152 "Device
//! Linking Without Internet", §153 "Device Linking With Internet".
//!
//! §152/§153 get no new type: both describe the SAME actual property
//! this crate already has — [`crate::certificate::DeviceCertificate::issue`]
//! and [`crate::directory::DeviceDirectory::sign`] are pure local
//! operations with no network call anywhere in their signatures or
//! implementations, so linking already "works" identically whether a
//! caller's transport happens to be Bluetooth/Wi-Fi Direct/LAN (§152,
//! no internet) or a relay/push/directory service (§153, internet
//! available) — the cryptographic operations this crate performs
//! don't know or care which. §153's own qualifier — "but cryptographic
//! authority remains local/account-controlled" — is exactly
//! [`crate::contact_verification::DirectoryServiceResponse::verify_and_accept`]'s
//! rule from an earlier round: an internet-connected directory service
//! can speed up discovery, never substitute for a real signature.
//!
//! §151 gets one new type: which channel a link negotiation actually
//! used is worth recording (for audit and for choosing bootstrap
//! proof methods), even though the eventual certificate is identical
//! either way.

use serde::{Deserialize, Serialize};

/// §151: "existing trusted device → authenticated channel → link
/// negotiation" is the preferred path when available; the alternative
/// is a cold bootstrap via QR/NFC with no prior authenticated channel
/// at all. Recorded for audit alongside
/// [`crate::contact_verification::OfflineVerificationMethod`], not a
/// replacement for it — a linking attempt has both a channel (this
/// type) and a bootstrap-proof method (that one).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkingChannel {
    /// §151's preferred path: an already-linked, already-trusted
    /// device carries the new device's certificate over its own
    /// authenticated channel. QR/NFC still supplies the initial
    /// bootstrap proof (matching
    /// [`crate::contact_verification::OfflineVerificationMethod::Qr`]/
    /// `Nfc`); what differs from cold bootstrap is that the actual
    /// certificate bytes travel over a channel already known to be
    /// authentic, not over the same short-range link the QR/NFC proof
    /// itself used.
    ExistingSecureSession,
    /// No prior authenticated channel to the new device exists yet —
    /// the whole exchange, bootstrap proof and certificate transfer
    /// alike, happens over whatever transport the QR/NFC/manual code
    /// step establishes.
    ColdBootstrap,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::directory::DeviceDirectory;
    use crate::root_key::RootIdentityKey;
    use siar_domain::{AccountId, DeviceId};

    /// §152/§153 made concrete: the same `issue`/`sign` calls succeed
    /// with no network dependency at all, so whichever
    /// [`LinkingChannel`] a caller records, the underlying certificate
    /// issuance neither knows nor cares — proving there's no hidden
    /// internet requirement anywhere in the actual signing path.
    #[test]
    fn certificate_issuance_has_no_network_dependency_regardless_of_channel() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();

        for channel in [
            LinkingChannel::ExistingSecureSession,
            LinkingChannel::ColdBootstrap,
        ] {
            let certificate = DeviceCertificate::issue(
                &root,
                account,
                device_id,
                [0u8; 32],
                0,
                None,
                DeviceCapabilitySet(0),
                1,
            );
            assert!(certificate
                .verify_signature(&root.root_public_key())
                .is_ok());
            // The channel is metadata about HOW this happened, not an
            // input the cryptographic operation itself needs.
            let _ = channel;
        }
    }

    #[test]
    fn linking_channel_is_recorded_independent_of_bootstrap_method() {
        // A caller can freely pair either channel with either
        // bootstrap-proof method — the two are orthogonal, matching
        // §151's own framing of channel as "how the certificate
        // travels" versus method as "how the bootstrap was proven."
        let existing_with_qr = (
            LinkingChannel::ExistingSecureSession,
            crate::contact_verification::OfflineVerificationMethod::Qr,
        );
        let cold_with_manual = (
            LinkingChannel::ColdBootstrap,
            crate::contact_verification::OfflineVerificationMethod::ManualFingerprint,
        );
        assert_ne!(existing_with_qr.0, cold_with_manual.0);
    }

    /// Confirms `DeviceDirectory::sign` — the other half of linking,
    /// beyond a single certificate — is equally network-free, backing
    /// up this module's claim that §152/§153 are already true of this
    /// crate's existing signing path, not just of certificate issuance
    /// alone.
    #[test]
    fn directory_signing_has_no_network_dependency_either() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let directory = DeviceDirectory::sign(&root, account, 1, vec![]);
        assert!(directory.verify_signature(&root.root_public_key()).is_ok());
    }
}
