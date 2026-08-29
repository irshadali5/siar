//! siar-crypto: identity keys + application-level E2EE (plan.md §12–13).
//!
//! Phase-1 scope only. This gives every message a real AEAD-encrypted
//! payload today, but the session layer here is intentionally minimal:
//! a static X25519 ECDH shared secret, not a ratchet. That means Phase 1
//! has confidentiality/integrity but **not** forward secrecy or
//! break-in recovery yet.
//!
//! plan.md §13 is explicit that a proven ratchet should be used rather
//! than inventing one — so the Phase-2 reliability pass should replace
//! `Session` with a real Double Ratchet implementation rather than
//! extending this one. `vodozemac` (Matrix's pure-Rust, audited-lineage
//! Olm/Megolm + Double Ratchet implementation) is the strongest
//! zero-C-dependency fit for that and is the recommended swap-in.
//!
//! `mailbox_token` (this pass) reuses this same identity/ECDH layer
//! for a different purpose: deriving rotating, unlinkable mailbox
//! capability tokens per next.md §32 — see that module's own doc
//! comment for the derivation and, importantly, what it still doesn't
//! solve.
//!
//! This crate is now also the home for several sections of Part 28
//! (Production Security / E2EE / Key Management / Privacy Architecture)
//! that don't depend on the still-unbuilt ratchet: `keystore` (§9-10,
//! opaque key handles + memory hygiene), `epoch` (§22, security epoch),
//! and `envelope`/`replay` (§14-17, secure message envelope +
//! associated data + replay protection + deterministic nonce
//! derivation). See each module's own doc comment for how it relates
//! to — and deliberately doesn't modify — `session.rs`.

mod attachment;
mod device_cert;
mod envelope;
mod epoch;
mod error;
mod identity;
mod keystore;
mod mailbox_token;
mod replay;
mod session;

pub use attachment::{
    decrypt_attachment, encrypt_attachment, AttachmentKey, BlobHash, EncryptedBlob,
};
pub use device_cert::{issue_device_certificate, verify_device_certificate, DeviceCertificate};
pub use envelope::{
    decrypt_envelope, encrypt_envelope, AuthenticationTag, MessageType, SecureMessageEnvelope,
    ENVELOPE_PROTOCOL_VERSION,
};
pub use epoch::SecurityEpoch;
pub use error::CryptoError;
pub use identity::DeviceIdentity;
pub use keystore::{InMemorySecureKeyStore, KeyHandle, KeyPolicy, SecureKeyStore};
pub use mailbox_token::{epoch_for, MailboxToken, MailboxTokenSecret, EPOCH_LENGTH_MILLIS};
pub use replay::{ReplayError, ReplayGuard};
pub use session::Session;
