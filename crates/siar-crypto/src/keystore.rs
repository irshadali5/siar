//! Secure key store trait and memory hygiene (Part 28 §9, §10).
//!
//! §9's own sketch is deliberately minimal (`generate_device_key` +
//! `sign`, both operating on an opaque `KeyHandle`) — this module
//! implements exactly that surface, plus a `verifying_key` accessor no
//! caller can avoid needing (there'd be no way to check a signature
//! otherwise), rather than inventing additional trait methods the spec
//! doesn't ask for.
//!
//! `InMemorySecureKeyStore` is the one concrete implementation here. A
//! real hardware-backed store (§56 hardware-bound keys — platform
//! keychain / TPM / StrongBox) is a separate, platform-specific
//! implementation of the same trait and out of scope for this crate,
//! which stays platform-free.
//!
//! §10's memory-hygiene list is a set of practices, not a type to build,
//! so it's enforced here rather than represented: `SigningKey` already
//! zeroizes on drop (the `zeroize` cargo feature is enabled workspace-
//! wide for `ed25519-dalek`, same as `identity.rs` already relies on),
//! `KeyHandle` never carries key material so it's safe to log/`Debug`,
//! and `InMemorySecureKeyStore` gets a hand-written `Debug` impl instead
//! of a derive so that adding a field later can never accidentally start
//! printing key material through a forgotten derive.

use crate::platform_keystore::KeyStorageBackend;
use crate::CryptoError;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Opaque reference to a key held inside a `SecureKeyStore`. Carries no
/// key material — safe to log, send over IPC, store in a UI view model,
/// etc. (§9: "prefer opaque `KeyHandle`s over exporting raw private-key
/// bytes").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyHandle(u64);

/// What kind of key to generate. Only `Signing` is implemented today —
/// this crate's only current key-generation caller (`DeviceIdentity`)
/// only ever needs an Ed25519 signing key. Left as an enum with room to
/// grow (an X25519 ECDH variant is the obvious next addition) rather
/// than a bare `generate_device_key()` with no policy parameter, so
/// existing call sites don't need to change shape when that's added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyPolicy {
    Signing,
}

pub trait SecureKeyStore {
    fn generate_device_key(&mut self, policy: KeyPolicy) -> Result<KeyHandle, CryptoError>;

    fn sign(&self, key: &KeyHandle, message: &[u8]) -> Result<Signature, CryptoError>;

    /// Not in §9's literal sketch, but unavoidable in practice: a
    /// caller that can `sign` needs a way to hand the corresponding
    /// public key to a peer for `DeviceIdentity::verify` — otherwise
    /// nothing outside this store could ever check a signature it
    /// produced.
    fn verifying_key(&self, key: &KeyHandle) -> Result<VerifyingKey, CryptoError>;

    /// §8: which storage backend this implementation actually uses.
    /// Lets a caller check `KeyStorageBackend::is_non_exportable`
    /// before trusting a store with, say, an account root key — rather
    /// than every implementation silently claiming to be equally safe.
    fn backend(&self) -> KeyStorageBackend;
}

/// Software-only reference implementation. Keys live in process memory
/// for the store's lifetime; each `SigningKey` zeroizes itself on drop
/// (via the `zeroize` feature), and dropping the whole map drops every
/// key in it the same way.
#[derive(Default)]
pub struct InMemorySecureKeyStore {
    keys: HashMap<KeyHandle, SigningKey>,
    next_handle: u64,
}

impl InMemorySecureKeyStore {
    pub fn new() -> Self {
        Self::default()
    }
}

// Hand-written, not derived (§10: "no debug formatting" for secret
// material) — this intentionally never iterates `keys`, so a future
// field added to this struct that *does* hold sensitive data doesn't
// silently start being printed just because someone re-derives Debug.
impl fmt::Debug for InMemorySecureKeyStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemorySecureKeyStore")
            .field("key_count", &self.keys.len())
            .finish()
    }
}

impl SecureKeyStore for InMemorySecureKeyStore {
    fn generate_device_key(&mut self, policy: KeyPolicy) -> Result<KeyHandle, CryptoError> {
        match policy {
            KeyPolicy::Signing => {
                let signing_key = SigningKey::generate(&mut OsRng);
                let handle = KeyHandle(self.next_handle);
                self.next_handle += 1;
                self.keys.insert(handle, signing_key);
                Ok(handle)
            }
        }
    }

    fn sign(&self, key: &KeyHandle, message: &[u8]) -> Result<Signature, CryptoError> {
        self.keys
            .get(key)
            .map(|signing_key| signing_key.sign(message))
            .ok_or(CryptoError::MalformedKey)
    }

    fn verifying_key(&self, key: &KeyHandle) -> Result<VerifyingKey, CryptoError> {
        self.keys
            .get(key)
            .map(SigningKey::verifying_key)
            .ok_or(CryptoError::MalformedKey)
    }

    fn backend(&self) -> KeyStorageBackend {
        KeyStorageBackend::InMemorySoftware
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_sign_verify_round_trip() {
        let mut store = InMemorySecureKeyStore::new();
        let handle = store.generate_device_key(KeyPolicy::Signing).unwrap();
        let msg = b"hello secure key store";
        let sig = store.sign(&handle, msg).unwrap();
        let vk = store.verifying_key(&handle).unwrap();
        assert!(vk.verify_strict(msg, &sig).is_ok());
    }

    #[test]
    fn unknown_handle_is_rejected() {
        let store = InMemorySecureKeyStore::new();
        let bogus = KeyHandle(999);
        assert!(store.sign(&bogus, b"x").is_err());
        assert!(store.verifying_key(&bogus).is_err());
    }

    #[test]
    fn two_handles_from_the_same_store_are_independent_keys() {
        let mut store = InMemorySecureKeyStore::new();
        let a = store.generate_device_key(KeyPolicy::Signing).unwrap();
        let b = store.generate_device_key(KeyPolicy::Signing).unwrap();
        assert_ne!(a, b);

        let msg = b"distinguish me";
        let sig_a = store.sign(&a, msg).unwrap();
        let vk_b = store.verifying_key(&b).unwrap();
        // b's key must not validate a signature produced by a's key.
        assert!(vk_b.verify_strict(msg, &sig_a).is_err());
    }

    #[test]
    fn debug_impl_never_prints_key_material() {
        let mut store = InMemorySecureKeyStore::new();
        let _ = store.generate_device_key(KeyPolicy::Signing).unwrap();
        let debug_str = format!("{store:?}");
        assert!(debug_str.contains("key_count"));
        // Best-effort structural check: the handwritten Debug impl only
        // ever formats a `usize` count field, so there is no code path
        // by which key bytes could appear here — this assertion mainly
        // guards against someone later replacing the impl with a derive.
        assert!(!debug_str.contains("SigningKey"));
    }

    #[test]
    fn in_memory_store_truthfully_reports_its_backend() {
        let store = InMemorySecureKeyStore::new();
        assert_eq!(store.backend(), KeyStorageBackend::InMemorySoftware);
        assert!(!store.backend().is_non_exportable());
    }
}
