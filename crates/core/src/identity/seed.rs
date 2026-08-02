//! 24-word seed phrase identity.
//!
//! One BIP39 mnemonic is the *only* secret the user has to remember or
//! back up. Everything else the app needs — the iroh node identity, the
//! iroh-docs author key used to sign username-registry claims, and any
//! future derived secret — comes out of that one seed via HKDF with a
//! distinct domain-separation string per purpose. This means:
//!
//!   - Recovery on a new device is "type the 24 words", full stop.
//!   - Adding a new derived key later never requires touching old data or
//!     asking the user to back up a second secret.
//!   - The mnemonic itself is never written to disk (see `identity/mod.rs`
//!     for what *is* persisted) — losing the device is fine as long as you
//!     still have the words on paper; the app has nothing at rest that
//!     alone reconstructs the mnemonic.
//!
//! Domain separation strings are versioned (`.../v1`) so a future key
//! rotation or added-purpose key can bump to `.../v2` for just that one
//! purpose without disturbing the others.

use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use hkdf::Hkdf;
use sha2::Sha512;

/// BIP39 with 24 words = 256 bits of entropy + an 8-bit checksum word.
/// Chosen per spec: more entropy than the common 12-word default, matching
/// what you'd want for something that's simultaneously your chat identity
/// and (eventually) your call/media keys.
const WORD_COUNT: usize = 24;

const INFO_IDENTITY: &[u8] = b"iroh-messenger/identity/ed25519/v1";
const INFO_DOCS_AUTHOR: &[u8] = b"iroh-messenger/docs-author/v1";

/// A freshly generated or user-supplied mnemonic, plus the seed bytes
/// derived from it (BIP39's own PBKDF2 step — separate from our HKDF step
/// below, which fans that seed out into purpose-specific keys).
pub struct Seed {
    mnemonic: Mnemonic,
    seed_bytes: [u8; 64],
}

impl Seed {
    /// Generate a brand new random 24-word mnemonic. Shown to the user
    /// exactly once at onboarding time — the caller is responsible for
    /// making sure they've confirmed writing it down before it's dropped.
    pub fn generate() -> Result<Self> {
        let mnemonic =
            Mnemonic::generate_in(Language::English, WORD_COUNT).context("generating mnemonic")?;
        Ok(Self::from_mnemonic(mnemonic))
    }

    /// Parse and validate a 24-word phrase the user typed in to recover an
    /// existing identity on a new device. Validates the BIP39 checksum, so
    /// a typo'd word (wrong word or wrong order) is caught here rather than
    /// silently producing a different identity than the user expects.
    pub fn from_phrase(phrase: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse_in(Language::English, phrase.trim())
            .context("that isn't a valid 24-word recovery phrase — check the words and order")?;
        if mnemonic.word_count() != WORD_COUNT {
            anyhow::bail!(
                "expected a {WORD_COUNT}-word phrase, got {}",
                mnemonic.word_count()
            );
        }
        Ok(Self::from_mnemonic(mnemonic))
    }

    fn from_mnemonic(mnemonic: Mnemonic) -> Self {
        // Empty BIP39 passphrase ("no 25th word") — kept simple/predictable
        // rather than adding a second secret the user would also have to
        // remember and never mistype.
        let seed_bytes = mnemonic.to_seed("");
        Self {
            mnemonic,
            seed_bytes,
        }
    }

    /// The 24 space-separated words, for display at onboarding time
    /// (create) or backup-verification time. Callers must never persist
    /// this to disk — see module doc.
    pub fn phrase(&self) -> String {
        self.mnemonic.to_string()
    }

    /// Only exercised by this module's own test (`cargo test`) right now,
    /// not by any production call site — `WORD_COUNT` is a fixed constant
    /// (24) so nothing in `ui::onboarding` needed to ask a `Seed` for its
    /// own length. Kept as real API rather than deleted since it's the
    /// obviously-correct thing to call if that ever changes.
    #[allow(dead_code)]
    pub fn word_count(&self) -> usize {
        self.mnemonic.word_count()
    }

    /// Derive the 32-byte secret that becomes this identity's iroh
    /// `SecretKey` (and therefore its `EndpointId`).
    pub fn derive_identity_key(&self) -> [u8; 32] {
        self.derive(INFO_IDENTITY)
    }

    /// Derive the 32-byte secret that *could* seed a deterministic
    /// iroh-docs `AuthorId` from this seed. No longer called by
    /// `identity::create_from_seed`/`load` — see the removal note on
    /// those — since `App::start` generates and persists its own
    /// docs-author identity independently (via
    /// `net::registry::Registry::new` + `store`'s `docs_author_id`
    /// setting) and nothing downstream ever read this one. Kept as real,
    /// tested API (see this module's tests) rather than deleted outright,
    /// in case a future design wants a seed-reproducible docs author again.
    #[allow(dead_code)]
    pub fn derive_docs_author_key(&self) -> [u8; 32] {
        self.derive(INFO_DOCS_AUTHOR)
    }

    fn derive(&self, info: &[u8]) -> [u8; 32] {
        // No salt: the seed itself already has 256 bits of entropy from a
        // CSPRNG (or, on recovery, from the user's own words), so an HKDF
        // salt would add nothing here — the `info` string is what gives us
        // domain separation between purposes, which is the property we
        // actually need.
        let hk = Hkdf::<Sha512>::new(None, &self.seed_bytes);
        let mut out = [0u8; 32];
        hk.expand(info, &mut out)
            .expect("32 bytes is within HKDF-SHA512's output-length limit");
        out
    }
}

impl Drop for Seed {
    fn drop(&mut self) {
        // Best-effort scrub. `Mnemonic`/`[u8; 64]` aren't guaranteed
        // non-reorderable by the optimizer without a crate like `zeroize`,
        // but overwriting is strictly better than leaving it — consider
        // adding `zeroize` here if this ever handles more sensitive
        // multi-user data than a single local desktop session already implies.
        self.seed_bytes = [0u8; 64];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_then_recover_gives_same_keys() {
        let seed = Seed::generate().unwrap();
        let phrase = seed.phrase();
        assert_eq!(seed.word_count(), WORD_COUNT);

        let recovered = Seed::from_phrase(&phrase).unwrap();
        assert_eq!(seed.derive_identity_key(), recovered.derive_identity_key());
        assert_eq!(
            seed.derive_docs_author_key(),
            recovered.derive_docs_author_key()
        );
    }

    #[test]
    fn identity_and_docs_keys_differ() {
        let seed = Seed::generate().unwrap();
        assert_ne!(seed.derive_identity_key(), seed.derive_docs_author_key());
    }

    #[test]
    fn rejects_garbage_phrase() {
        assert!(Seed::from_phrase("not a real seed phrase at all").is_err());
    }
}
