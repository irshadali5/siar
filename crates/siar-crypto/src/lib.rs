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
//! `envelope`/`replay` (§14-17, secure message envelope + associated
//! data + replay protection + deterministic nonce derivation),
//! `fanout` (§18, multi-device delivery), `history_policy` (§19's
//! missing piece — the enrollment flow itself lives in
//! `siar-identity-multidevice`), `revocation` (§20-21, a standalone
//! signed revocation record + epoch-based propagation mitigation, built
//! on top of — not replacing — `siar-identity-multidevice`'s directory-
//! generation revocation model), `clone_detection`/`restore_safety`
//! (§23-24, device-instance clone detection and forcing restores onto a
//! safe path), and `platform_keystore` (§8, the policy-only
//! `KeyStorageBackend` abstraction `keystore.rs::SecureKeyStore::backend`
//! reports — no actual OS/hardware call is implemented anywhere in this
//! crate; real Android Keystore/Keychain/DPAPI/TPM adapters are a
//! separate, platform-specific effort). See each module's own doc
//! comment for how it relates to — and, where relevant, deliberately
//! doesn't modify — `session.rs` or `siar-identity-multidevice`.
//!
//! §5-7 (Identity Hierarchy / Account Identity / Device Identity) are
//! reconciliation notes rather than new modules — the types already
//! exist, just under different names than Part 28's own Rust sketches
//! use: §6's `AccountId([u8; 32])` ("a long-lived logical account
//! root... used rarely, mainly for authorizing/revoking devices,
//! account recovery, high-value trust changes... not for every
//! message") is conceptually `siar_identity_multidevice::RootIdentityKey`/
//! `RootPublicKey`, not `siar_domain::AccountId` (a `Uuid` used
//! throughout this workspace as the account's routing/addressing
//! identifier — a distinct, already-established concept that should
//! keep its existing name; renaming or duplicating it to match §6's
//! literal type name would recreate exactly the "one key for unrelated
//! purposes" confusion §5 warns against). §7's `DeviceCertificate` DTO
//! (account/device/device_public_key/issued_at/expires_at/signature)
//! matches `siar_identity_multidevice::DeviceCertificate` field-for-
//! field (which additionally carries `capabilities` and `generation`).
//! §5's own list — keep Account/Device/Session/Conversation/Group-
//! Epoch/File-Content/Transport/Backup keys distinct — already holds
//! across this workspace's existing types (`RootIdentityKey`,
//! `DeviceIdentity`, `Session`, `AttachmentKey` are all separate,
//! non-interchangeable types); Group Epoch Keys, Transport Keys, and
//! Backup Keys don't exist as concrete types yet (§25-27 and §112+
//! respectively — not built).

mod attachment;
mod clone_detection;
mod device_cert;
mod envelope;
mod epoch;
mod error;
mod fanout;
mod history_policy;
mod identity;
mod keystore;
mod mailbox_token;
mod platform_keystore;
mod replay;
mod restore_safety;
mod revocation;
mod session;

pub use attachment::{
    decrypt_attachment, encrypt_attachment, AttachmentKey, BlobHash, EncryptedBlob,
};
pub use clone_detection::{CloneDetector, CloneVerdict, DeviceInstanceId};
pub use device_cert::{issue_device_certificate, verify_device_certificate, DeviceCertificate};
pub use envelope::{
    decrypt_envelope, encrypt_envelope, AuthenticationTag, MessageType, SecureMessageEnvelope,
    ENVELOPE_PROTOCOL_VERSION,
};
pub use epoch::SecurityEpoch;
pub use error::CryptoError;
pub use fanout::{fan_out_envelope, RecipientDevice};
pub use history_policy::HistoryAccessPolicy;
pub use identity::DeviceIdentity;
pub use keystore::{InMemorySecureKeyStore, KeyHandle, KeyPolicy, SecureKeyStore};
pub use mailbox_token::{epoch_for, MailboxToken, MailboxTokenSecret, EPOCH_LENGTH_MILLIS};
pub use platform_keystore::KeyStorageBackend;
pub use replay::{ReplayError, ReplayGuard};
pub use restore_safety::{decide_restore, RestoreDecision};
pub use revocation::{is_epoch_stale_after_revocation, DeviceRevocation, RevocationReason};
pub use session::Session;
