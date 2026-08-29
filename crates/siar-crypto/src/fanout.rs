//! Multi-device delivery (Part 28 §18).
//!
//! §18's own model for small device sets: "one logical message →
//! recipient device envelopes." `fan_out_envelope` is exactly that:
//! the same logical plaintext, encrypted once per recipient device
//! under that device's own key, producing one `SecureMessageEnvelope`
//! per device rather than a single shared ciphertext. "For large groups
//! or many devices, use efficient group security" (§18's own second
//! sentence) is deliberately out of scope here — that's §25-27 Group
//! Security, which needs a real MLS-style group key schedule
//! (`siar-crypto-mls` exists in this workspace but isn't reconciled
//! against this spec yet) rather than N individual envelopes, and
//! isn't something this module should approximate by just calling
//! itself in a bigger loop.
//!
//! Note what's *not* in `SecureMessageEnvelope`: a recipient device ID.
//! That's deliberate, not an oversight — each envelope here is
//! encrypted under a key specific to one recipient device, so only
//! that device's cipher can ever open it; which device an envelope is
//! "for" is established by which per-device channel/mailbox it's
//! delivered over, not by a plaintext-visible field inside the
//! envelope itself (a recipient-device field would be one more piece
//! of metadata visible to anything routing the envelope, which §15's
//! own "only expose routing fields that transports actually need"
//! principle argues against adding without a concrete need).

use crate::envelope::{encrypt_envelope, MessageType, SecureMessageEnvelope};
use crate::epoch::SecurityEpoch;
use crate::CryptoError;
use chacha20poly1305::ChaCha20Poly1305;
use siar_domain::{ConversationId, DeviceId, MessageId};

/// One recipient device's own encryption context. This crate has no
/// ratchet yet (see `envelope.rs`'s own doc comment), so `cipher` is
/// whatever per-device key the caller's session layer has already
/// derived for that device. `next_counter` must be a counter value
/// never before used under `(cipher, epoch)` for this device — the
/// same send-side obligation `envelope.rs` already documents, made
/// explicit per-recipient here since a fan-out advances several
/// independent counter streams at once, one per device, not one
/// shared stream.
pub struct RecipientDevice<'a> {
    pub device: DeviceId,
    pub cipher: &'a ChaCha20Poly1305,
    pub next_counter: u64,
}

/// Encrypts `plaintext` once per entry in `recipients`, returning one
/// envelope per device in the same order. A failure encrypting for any
/// one device aborts the whole fan-out — this never returns a partial
/// envelope set, since a caller silently treating a partial fan-out as
/// "delivered" is exactly the kind of inconsistent multi-device state
/// §18 exists to avoid. A caller that genuinely wants best-effort
/// per-device delivery should call `encrypt_envelope` directly per
/// device instead and handle each result independently.
pub fn fan_out_envelope(
    plaintext: &[u8],
    conversation: ConversationId,
    sender_device: DeviceId,
    message_id: MessageId,
    epoch: SecurityEpoch,
    recipients: &[RecipientDevice<'_>],
) -> Result<Vec<SecureMessageEnvelope>, CryptoError> {
    recipients
        .iter()
        .map(|recipient| {
            encrypt_envelope(
                recipient.cipher,
                plaintext,
                conversation,
                sender_device,
                message_id,
                MessageType::Application,
                epoch,
                recipient.next_counter,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chacha20poly1305::KeyInit;

    #[test]
    fn fans_out_one_envelope_per_recipient() {
        let cipher_a = ChaCha20Poly1305::new(&[1u8; 32].into());
        let cipher_b = ChaCha20Poly1305::new(&[2u8; 32].into());
        let cipher_c = ChaCha20Poly1305::new(&[3u8; 32].into());

        let recipients = vec![
            RecipientDevice { device: DeviceId::new(), cipher: &cipher_a, next_counter: 0 },
            RecipientDevice { device: DeviceId::new(), cipher: &cipher_b, next_counter: 0 },
            RecipientDevice { device: DeviceId::new(), cipher: &cipher_c, next_counter: 0 },
        ];

        let envelopes = fan_out_envelope(
            b"same logical message",
            ConversationId::new(),
            DeviceId::new(),
            MessageId::new(),
            SecurityEpoch::zero(),
            &recipients,
        )
        .unwrap();

        assert_eq!(envelopes.len(), 3);
        // Each envelope is under a different key, so ciphertexts differ
        // even though the plaintext, conversation, and message_id are
        // all identical across the fan-out.
        assert_ne!(envelopes[0].ciphertext, envelopes[1].ciphertext);
        assert_ne!(envelopes[1].ciphertext, envelopes[2].ciphertext);
    }

    #[test]
    fn each_recipients_envelope_only_decrypts_under_its_own_cipher() {
        use crate::envelope::decrypt_envelope;

        let cipher_a = ChaCha20Poly1305::new(&[9u8; 32].into());
        let cipher_b = ChaCha20Poly1305::new(&[8u8; 32].into());
        let recipients = vec![
            RecipientDevice { device: DeviceId::new(), cipher: &cipher_a, next_counter: 0 },
            RecipientDevice { device: DeviceId::new(), cipher: &cipher_b, next_counter: 0 },
        ];

        let envelopes = fan_out_envelope(
            b"secret",
            ConversationId::new(),
            DeviceId::new(),
            MessageId::new(),
            SecurityEpoch::zero(),
            &recipients,
        )
        .unwrap();

        assert_eq!(
            decrypt_envelope(&cipher_a, &envelopes[0], MessageType::Application).unwrap(),
            b"secret"
        );
        // B's cipher must not be able to open A's envelope.
        assert!(decrypt_envelope(&cipher_b, &envelopes[0], MessageType::Application).is_err());
    }

    #[test]
    fn empty_recipient_list_yields_no_envelopes_without_erroring() {
        let envelopes = fan_out_envelope(
            b"nobody to send to",
            ConversationId::new(),
            DeviceId::new(),
            MessageId::new(),
            SecurityEpoch::zero(),
            &[],
        )
        .unwrap();
        assert!(envelopes.is_empty());
    }
}
