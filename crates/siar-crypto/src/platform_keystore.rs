//! Device key storage backends (Part 28 §8).
//!
//! §8's own division of labor: "Rust owns the abstraction and policy;
//! platform adapters own the unavoidable OS calls." `SecureKeyStore`
//! (§9, `keystore.rs`) is already that abstraction — this module is the
//! *policy* half: naming which backend a key is meant to live in, so a
//! caller/config can express "this key should be Android-Keystore-
//! backed" even before a real Android-Keystore-backed `SecureKeyStore`
//! implementation exists to honor that request.
//!
//! What this module deliberately does **not** contain: any actual OS
//! call. `KeyStorageBackend` is a policy value, and `InMemorySecureKeyStore`
//! (§9) truthfully reports `KeyStorageBackend::InMemorySoftware` because
//! that's the only backend this crate can honestly implement without
//! JNI (Android Keystore), platform Keychain/Secure Enclave bindings
//! (Apple), DPAPI (Windows), or a TPM/Secret Service integration
//! (Linux) — every one of those is a real, separate, platform-specific
//! engineering effort (comparable in scope to this workspace's existing
//! Android JNI glue, `apps/android/messaging-jni`), not something to
//! stub out as if it were already working.

use serde::{Deserialize, Serialize};

/// §8's own listed backends, one variant each, plus the one backend
/// this crate can actually provide today (`InMemorySoftware`, backing
/// `keystore.rs`'s `InMemorySecureKeyStore`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStorageBackend {
    AndroidKeystore,
    AppleKeychainOrSecureEnclave,
    WindowsDpapi,
    LinuxTpmOrSecretService,
    EmbeddedLinuxTpm,
    /// Software-only, in-process key storage. Not listed in §8's own
    /// per-platform table — every platform there names a hardware- or
    /// OS-backed option — but a real, honestly-named fallback for
    /// exactly what `InMemorySecureKeyStore` provides today, and for
    /// any environment (e.g. a headless daemon/test harness) where a
    /// platform-backed store genuinely isn't available.
    InMemorySoftware,
}

impl KeyStorageBackend {
    /// Whether keys held under this backend are non-exportable by
    /// construction (the OS/hardware never releases raw key bytes to
    /// this process) — §8's own stated preference ("prefer
    /// non-exportable platform-backed keys where practical"). Every
    /// real platform backend qualifies; `InMemorySoftware` does not,
    /// since this crate holds the raw `SigningKey` directly (see
    /// `keystore.rs`).
    pub const fn is_non_exportable(self) -> bool {
        !matches!(self, Self::InMemorySoftware)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_real_platform_backend_is_non_exportable() {
        for backend in [
            KeyStorageBackend::AndroidKeystore,
            KeyStorageBackend::AppleKeychainOrSecureEnclave,
            KeyStorageBackend::WindowsDpapi,
            KeyStorageBackend::LinuxTpmOrSecretService,
            KeyStorageBackend::EmbeddedLinuxTpm,
        ] {
            assert!(backend.is_non_exportable());
        }
    }

    #[test]
    fn in_memory_software_is_exportable() {
        assert!(!KeyStorageBackend::InMemorySoftware.is_non_exportable());
    }
}
