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

mod identity;
mod session;
mod attachment;
mod device_cert;
mod error;
mod mailbox_token;

pub use error::CryptoError;
pub use identity::DeviceIdentity;
pub use session::Session;
pub use attachment::{decrypt_attachment, encrypt_attachment, AttachmentKey, BlobHash, EncryptedBlob};
pub use device_cert::{issue_device_certificate, verify_device_certificate, DeviceCertificate};
pub use mailbox_token::{epoch_for, MailboxToken, MailboxTokenSecret, EPOCH_LENGTH_MILLIS};
