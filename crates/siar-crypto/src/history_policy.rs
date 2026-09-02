//! Historical access policy (Part 28 §19, the one piece of "New Device
//! Enrollment" not already covered elsewhere in this workspace).
//!
//! §19's enrollment flow itself — "existing trusted device authenticates
//! new device → signs device certificate → establishes encrypted
//! device-sync channel" — is already real, working code:
//! `siar_identity_multidevice`'s linking flow (`EphemeralLinkKeyPair`,
//! `DeviceLinkInvite`, `derive_verification_code`,
//! `LinkingApprovalPrompt`) covers authenticate+sign; `Session`
//! (`session.rs`, this crate) is the encrypted channel a device-sync
//! exchange would run over. What's missing is §19's second
//! requirement — "historical access policy must be explicit" — which
//! nothing in this workspace represents as a type today: enrollment
//! currently has no way to say *which* of a new device's message
//! history it should receive.

use serde::{Deserialize, Serialize};

/// §19's three explicit options, verbatim. Deliberately not a
/// caller-supplied free-form range or count — the spec calls these out
/// as the policy choices, not an example of them, so this type doesn't
/// add options the spec didn't ask for (like "last N days"), which
/// would just be a different, unreviewed policy wearing this one's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryAccessPolicy {
    /// The new device sees only messages sent after it was enrolled.
    /// The safest option against a compromised or lost new device
    /// exposing prior history, and a reasonable default for a caller
    /// that has no policy input from the account owner.
    FutureOnly,
    /// The new device receives a caller-chosen subset of prior history.
    /// This type doesn't define *how* that subset is chosen (a date
    /// range, a set of conversations, a size cap) — that's a UI/backfill
    /// concern for whatever builds on this, not a security-layer
    /// decision this type should encode.
    SelectedHistory,
    /// The new device receives the account's full available history.
    /// The highest-exposure option; a caller enrolling a device with
    /// this policy is making a deliberate trust decision, not falling
    /// into it by default.
    FullHistory,
}

impl HistoryAccessPolicy {
    /// §19 doesn't state a default, but implies one by listing
    /// `FutureOnly` first among "future only / selected history / full
    /// history" and by the surrounding security posture of this whole
    /// document (new, unverified devices should start from the least
    /// exposure). A caller is free to ignore this and require an
    /// explicit choice instead — this exists for callers that want a
    /// safe fallback rather than an error when no policy was specified.
    pub const fn conservative_default() -> Self {
        Self::FutureOnly
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conservative_default_is_future_only() {
        assert_eq!(
            HistoryAccessPolicy::conservative_default(),
            HistoryAccessPolicy::FutureOnly
        );
    }

    #[test]
    fn all_three_policies_round_trip_through_postcard() {
        for policy in [
            HistoryAccessPolicy::FutureOnly,
            HistoryAccessPolicy::SelectedHistory,
            HistoryAccessPolicy::FullHistory,
        ] {
            let bytes = postcard::to_allocvec(&policy).unwrap();
            let decoded: HistoryAccessPolicy = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(decoded, policy);
        }
    }
}
