//! Encrypted mailbox client model (plan.md §30–32).
//!
//! plan.md §32 is explicit: avoid stable public account IDs indexing
//! mailbox messages — use rotating capability tokens instead. This
//! module only holds the client-side shape of that; the actual mailbox
//! relay service (a small always-on server plan.md §108 lists as
//! optional infrastructure) is out of scope for this session, same
//! reasoning as the Android/iOS platform code — a real relay is a
//! network service with its own deployment/security review needs, not
//! something to sketch blind.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// plan.md §32: what the sender learns about the recipient's mailbox
/// through encrypted identity/session establishment — an opaque address
/// token plus an authorization secret, not the recipient's stable
/// account ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailboxCapability {
    pub address_token: [u8; 16],
    pub authorization_secret: [u8; 32],
    pub expires_at_millis: u64,
}

impl MailboxCapability {
    pub fn is_expired(&self, now_millis: u64) -> bool {
        now_millis >= self.expires_at_millis
    }
}

/// What actually gets handed to the relay (plan.md §31: "opaque
/// encrypted envelope" — the relay sees this struct's fields and nothing
/// else, never the plaintext message or even which conversation it's
/// part of).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxEnvelope {
    pub address_token: [u8; 16],
    pub ciphertext: Vec<u8>,
    pub expires_at_millis: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MailboxError {
    #[error("mailbox capability expired")]
    CapabilityExpired,
    #[error("envelope is {0} bytes, exceeds the {MAX_MAILBOX_ENVELOPE_BYTES}-byte limit")]
    EnvelopeTooLarge(usize),
}

/// plan.md §61's decode-limits discipline again: a mailbox relay is
/// explicitly untrusted infrastructure (plan.md §31), so envelopes
/// destined for it get the same size discipline as anything else
/// crossing a trust boundary.
pub const MAX_MAILBOX_ENVELOPE_BYTES: usize = 512 * 1024;

/// Wraps an already-encrypted message payload (the output of
/// `siar_crypto::Session::encrypt`, or an attachment reference's
/// ciphertext) for mailbox delivery. Returns an error rather than
/// silently truncating or forwarding an expired/oversized envelope —
/// the caller decides what to do next (retry later, fall back to direct
/// delivery once the peer's back online).
pub fn prepare_envelope(
    capability: &MailboxCapability,
    ciphertext: Vec<u8>,
    now_millis: u64,
) -> Result<MailboxEnvelope, MailboxError> {
    if capability.is_expired(now_millis) {
        return Err(MailboxError::CapabilityExpired);
    }
    if ciphertext.len() > MAX_MAILBOX_ENVELOPE_BYTES {
        return Err(MailboxError::EnvelopeTooLarge(ciphertext.len()));
    }
    Ok(MailboxEnvelope {
        address_token: capability.address_token,
        ciphertext,
        expires_at_millis: capability.expires_at_millis,
    })
}

/// Convenience for call sites that don't already have a millisecond
/// clock reading handy — kept separate from `prepare_envelope` so tests
/// can pass a fixed `now_millis` instead of depending on the real clock.
pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before 1970")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capability(expires_at_millis: u64) -> MailboxCapability {
        MailboxCapability {
            address_token: [1u8; 16],
            authorization_secret: [2u8; 32],
            expires_at_millis,
        }
    }

    #[test]
    fn prepares_a_valid_envelope() {
        let cap = capability(10_000);
        let envelope = prepare_envelope(&cap, vec![1, 2, 3], 5_000).unwrap();
        assert_eq!(envelope.address_token, cap.address_token);
        assert_eq!(envelope.ciphertext, vec![1, 2, 3]);
    }

    #[test]
    fn rejects_an_expired_capability() {
        let cap = capability(1_000);
        let err = prepare_envelope(&cap, vec![1], 2_000).unwrap_err();
        assert_eq!(err, MailboxError::CapabilityExpired);
    }

    #[test]
    fn rejects_an_oversized_envelope() {
        let cap = capability(u64::MAX);
        let too_big = vec![0u8; MAX_MAILBOX_ENVELOPE_BYTES + 1];
        let err = prepare_envelope(&cap, too_big, 0).unwrap_err();
        assert!(matches!(err, MailboxError::EnvelopeTooLarge(_)));
    }

    #[test]
    fn expiry_boundary_is_inclusive() {
        let cap = capability(1_000);
        assert!(cap.is_expired(1_000));
        assert!(!cap.is_expired(999));
    }
}
