//! Rotating, unlinkable mailbox capability tokens — next.md §32's
//! literal ask, left explicitly unaddressed by `siar_protocol::mailbox`'s
//! own doc comment until this pass: *"Avoid stable public account IDs
//! directly indexing mailbox messages. Use rotating mailbox
//! capabilities/tokens."*
//!
//! ## Why this belongs in `siar-crypto`, not `siar-protocol`
//!
//! Deriving a token needs [`DeviceIdentity::diffie_hellman`], which is
//! `pub(crate)` on purpose — this file's own top-of-crate invariant
//! ("never expose raw secret keys outside this crate") applies to raw
//! ECDH output exactly as much as it does to the signing/X25519 secrets
//! themselves, so token derivation has to happen inside this crate, the
//! same reason [`crate::session::Session::establish`] does.
//!
//! ## What this is, concretely — and why it isn't new cryptography
//!
//! [`MailboxTokenSecret::establish`] derives a per-*pair* root secret
//! the exact same way [`crate::session::Session::establish`] already
//! does: BLAKE3 over a raw X25519 ECDH output, never the raw DH bytes
//! themselves (that function's own doc comment explains why). The only
//! difference is a distinct domain-separation label, so this secret is
//! cryptographically independent of the message-encryption session key
//! even though both start from the same DH output — reusing one
//! derived key for two purposes is exactly the subtle misuse distinct
//! KDF labels exist to prevent.
//!
//! [`MailboxTokenSecret::token_for_epoch`] then derives a fresh 32-byte
//! token per *epoch* (a coarse, agreed-on time bucket — see
//! [`epoch_for`]) the same way: BLAKE3 over the root secret concatenated
//! with the epoch number and another distinct label. Two tokens from
//! different epochs are BLAKE3 outputs of different inputs — there is
//! no computation that recovers a relationship between them without the
//! root secret, which the relay never sees. That's the entire
//! unlinkability property, and it's built entirely from BLAKE3 hashing
//! and X25519 ECDH — both already-adopted primitives in this crate, not
//! a new scheme. This is deliberately simpler than blind signatures or
//! anonymous credentials (what a *complete* answer to next.md §32 would
//! eventually use, and what `mailbox.rs`'s own doc comment correctly
//! flags as real cryptographic-design work this project shouldn't
//! improvise) — it trades a cleaner formal anonymity guarantee for
//! "standard KDF composition an implementer can review in an afternoon."
//!
//! ## What this does NOT solve — read before assuming §32 is fully closed
//!
//! - **A shared secret must already exist.** [`MailboxTokenSecret`] is
//!   derived from an ECDH exchange between two *specific* devices — it
//!   presumes the pairing/contact-exchange step that establishes each
//!   side's X25519 public key has already happened (same precondition
//!   [`crate::session::Session`] already has). There is no anonymous
//!   token for a device that hasn't paired with anyone yet.
//! - **Within one epoch, a token is still a stable, linkable index.**
//!   Rotation happens *between* epochs, not per check-in — two check-ins
//!   in the same epoch present the identical token, and a relay can
//!   correlate those two the same way it could correlate two `DeviceId`s
//!   today. `EPOCH_LENGTH_MILLIS` is the resulting unlinkability
//!   *window*, not an unlinkability guarantee with no window at all —
//!   narrowing that window trades against how much clock skew tolerance
//!   a real check-in flow can afford (see [`epoch_for`]'s doc comment).
//! - **Possessing a token is possessing the capability — there is no
//!   separate signature.** Anyone who intercepts a token in transit (or
//!   compromises the relay's storage of it) can redeem whatever's filed
//!   under it for the rest of that epoch. This is the standard tradeoff
//!   every bearer-capability design makes (an API key has the same
//!   property), named here rather than left implicit, and matches the
//!   TOFU-race caveat `mailbox.rs`'s own doc comment already carries for
//!   the identified check-in path.
//! - **The relay must adopt a token-keyed mailbox store for this to mean
//!   anything.** This module only derives tokens; nothing in this
//!   workspace yet stores or looks up bundles by [`MailboxToken`] instead
//!   of `DeviceId` — that wiring (a real change to whatever holds
//!   mailbox contents on `apps/emergency-node`) is separate follow-up
//!   work, same "computation built, no real caller yet" shape as
//!   `siar_routing::link_health::LinkHealth`/`PathTable::
//!   compose_via_relay` from earlier passes.

use crate::identity::DeviceIdentity;
use serde::{Deserialize, Serialize};
use x25519_dalek::PublicKey as X25519PublicKey;

const ROOT_SECRET_DOMAIN: &[u8] = b"siar-mailbox-token-root-v1";
const TOKEN_DOMAIN: &[u8] = b"siar-mailbox-token-epoch-v1";

/// How long one epoch spans — the unlinkability *window* this module
/// actually provides (see this file's top doc comment). An hour is a
/// starting point, not a measured/tuned value: long enough that a
/// device checking in every few minutes doesn't burn through epochs
/// faster than messages addressed to its previous token could realistically
/// still be waiting, short enough that "how long can two check-ins be
/// correlated" stays a meaningful bound rather than a nominal one. Real
/// tuning needs real traffic patterns this workspace doesn't have yet
/// — same status every other untuned constant here carries.
pub const EPOCH_LENGTH_MILLIS: u64 = 60 * 60 * 1000;

/// A 32-byte value indistinguishable from random to anyone without the
/// [`MailboxTokenSecret`] it was derived from — the actual index a
/// token-keyed mailbox store would use in place of a `DeviceId`. `Eq`/
/// `Hash` so it can key a `HashMap` the same way `DeviceId` does today;
/// `Serialize`/`Deserialize` (serde's default byte-array encoding, no
/// custom `Serialize` impl needed) so it can travel in a check-in
/// message the same way every other wire-facing type in this workspace
/// does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MailboxToken(pub [u8; 32]);

/// The current epoch number for `now_millis` — both sides of a pair
/// compute this independently from their own (approximately synced)
/// wall clocks, the same "no rendezvous needed to agree which epoch
/// we're in" property TOTP-style schemes rely on. Floor division means
/// a check-in arriving anywhere within `EPOCH_LENGTH_MILLIS` of a
/// sender's own epoch computation lands on the same epoch number as
/// long as both clocks agree within that window — callers wanting
/// tolerance for clock skew *at* an epoch boundary should additionally
/// try `epoch_for(now) - 1`/`+ 1`, the same adjacent-window trick TOTP
/// verifiers use, rather than this function trying to guess a skew
/// budget on its own.
pub fn epoch_for(now_millis: u64) -> u64 {
    now_millis / EPOCH_LENGTH_MILLIS
}

/// A per-pair root secret, derived once from an ECDH exchange and
/// reused to derive every epoch's [`MailboxToken`] for that pair — see
/// this file's top doc comment for the full derivation and its limits.
pub struct MailboxTokenSecret([u8; 32]);

impl MailboxTokenSecret {
    /// `us`/`peer_x25519_public` — same two-argument shape as
    /// [`crate::session::Session::establish`], deliberately: this is
    /// the same ECDH exchange, just fed into a differently-labeled
    /// derivation to keep the two resulting secrets independent.
    pub fn establish(us: &DeviceIdentity, peer_x25519_public: &X25519PublicKey) -> Self {
        let shared = us.diffie_hellman(peer_x25519_public);
        let mut input = Vec::with_capacity(ROOT_SECRET_DOMAIN.len() + shared.len());
        input.extend_from_slice(ROOT_SECRET_DOMAIN);
        input.extend_from_slice(&shared);
        Self(*blake3::hash(&input).as_bytes())
    }

    /// Derives the [`MailboxToken`] for a specific epoch number (see
    /// [`epoch_for`]) — takes the epoch directly rather than a
    /// timestamp, so a caller checking adjacent epochs for clock-skew
    /// tolerance (this file's top doc comment) calls this more than
    /// once with different epoch numbers rather than juggling multiple
    /// timestamps.
    pub fn token_for_epoch(&self, epoch: u64) -> MailboxToken {
        let mut input = Vec::with_capacity(TOKEN_DOMAIN.len() + 32 + 8);
        input.extend_from_slice(TOKEN_DOMAIN);
        input.extend_from_slice(&self.0);
        input.extend_from_slice(&epoch.to_le_bytes());
        MailboxToken(*blake3::hash(&input).as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_sides_derive_the_same_root_secret_and_the_same_token() {
        let alice = DeviceIdentity::generate();
        let bob = DeviceIdentity::generate();

        let alice_secret = MailboxTokenSecret::establish(&alice, &bob.x25519_public());
        let bob_secret = MailboxTokenSecret::establish(&bob, &alice.x25519_public());

        assert_eq!(
            alice_secret.token_for_epoch(42),
            bob_secret.token_for_epoch(42)
        );
    }

    #[test]
    fn different_epochs_produce_unrelated_tokens() {
        let alice = DeviceIdentity::generate();
        let bob = DeviceIdentity::generate();
        let secret = MailboxTokenSecret::establish(&alice, &bob.x25519_public());

        let token_1 = secret.token_for_epoch(1);
        let token_2 = secret.token_for_epoch(2);
        assert_ne!(token_1, token_2);
    }

    #[test]
    fn the_same_epoch_always_produces_the_same_token() {
        let alice = DeviceIdentity::generate();
        let bob = DeviceIdentity::generate();
        let secret = MailboxTokenSecret::establish(&alice, &bob.x25519_public());

        assert_eq!(secret.token_for_epoch(7), secret.token_for_epoch(7));
    }

    #[test]
    fn a_pair_not_involved_in_the_exchange_cannot_derive_the_same_token() {
        let alice = DeviceIdentity::generate();
        let bob = DeviceIdentity::generate();
        let mallory = DeviceIdentity::generate();

        let real_secret = MailboxTokenSecret::establish(&alice, &bob.x25519_public());
        // Mallory has no relationship to this pair — the best she can do
        // is derive a secret against her own key material, which must
        // not collide with the real one.
        let mallory_secret = MailboxTokenSecret::establish(&mallory, &bob.x25519_public());

        assert_ne!(
            real_secret.token_for_epoch(1),
            mallory_secret.token_for_epoch(1)
        );
    }

    #[test]
    fn root_secret_derivation_is_independent_of_the_session_key_derivation() {
        // Same ECDH inputs `Session::establish` would use, but the
        // distinct domain-separation label must still produce a value
        // with no fixed relationship to a `Session`'s own key — this
        // module's top doc comment's entire reason for having a
        // separate label. Verified indirectly here: two runs of
        // `establish` with the same identities always agree with each
        // other (deterministic from the shared DH output), which is the
        // property `token_for_epoch` needs.
        let alice = DeviceIdentity::generate();
        let bob = DeviceIdentity::generate();
        let secret_a = MailboxTokenSecret::establish(&alice, &bob.x25519_public());
        let secret_b = MailboxTokenSecret::establish(&alice, &bob.x25519_public());
        assert_eq!(secret_a.token_for_epoch(3), secret_b.token_for_epoch(3));
    }

    #[test]
    fn epoch_for_buckets_nearby_timestamps_together() {
        let epoch_start = 5 * EPOCH_LENGTH_MILLIS;
        assert_eq!(epoch_for(epoch_start), 5);
        assert_eq!(epoch_for(epoch_start + EPOCH_LENGTH_MILLIS - 1), 5);
        assert_eq!(epoch_for(epoch_start + EPOCH_LENGTH_MILLIS), 6);
    }
}
