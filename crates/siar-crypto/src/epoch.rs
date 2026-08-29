//! Security epoch (Part 28 §22).
//!
//! A monotonically-increasing counter over the lifetime of an
//! account/conversation's security state. The spec's own list of
//! advance-triggers — device revocation, group membership change,
//! identity recovery, major security incident — are all events this
//! crate doesn't itself decide to fire (that's `siar-identity-multidevice`
//! for revocation, a future group-membership crate for membership
//! changes, etc). This type only gives every layer a single, ordered
//! way to *represent* "which security generation is this", plus the one
//! operation (`advance`) that's unambiguous regardless of which caller
//! triggers it.
//!
//! `SecureMessageEnvelope` (`envelope.rs`) folds the epoch into both its
//! associated data (§15) and its nonce derivation (§17) specifically so
//! that a ciphertext from a stale epoch can never be replayed as if it
//! were current — see `envelope.rs` and `replay.rs` for how.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct SecurityEpoch(pub u64);

impl SecurityEpoch {
    /// The starting epoch for a freshly-provisioned identity/conversation.
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Advance to the next epoch. Deliberately infallible and
    /// non-wrapping-on-overflow-by-panic-in-debug (`u64::MAX` epochs is
    /// not a realistic operational ceiling); a caller that somehow hit
    /// it would have far bigger problems than this method's behavior.
    #[must_use]
    pub fn advance(self) -> Self {
        Self(self.0 + 1)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_zero() {
        assert_eq!(SecurityEpoch::zero().as_u64(), 0);
        assert_eq!(SecurityEpoch::default(), SecurityEpoch::zero());
    }

    #[test]
    fn advance_is_monotonic() {
        let e0 = SecurityEpoch::zero();
        let e1 = e0.advance();
        let e2 = e1.advance();
        assert!(e1 > e0);
        assert!(e2 > e1);
        assert_eq!(e2.as_u64(), 2);
    }

    #[test]
    fn ordering_is_by_value() {
        let mut epochs = vec![SecurityEpoch(3), SecurityEpoch(1), SecurityEpoch(2)];
        epochs.sort();
        assert_eq!(epochs, vec![SecurityEpoch(1), SecurityEpoch(2), SecurityEpoch(3)]);
    }
}
