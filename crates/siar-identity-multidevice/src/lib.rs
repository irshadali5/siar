#![forbid(unsafe_code)]

//! siar-identity-multidevice: a first slice of "Part 02 — Multi-Device
//! Identity Architecture" (one of a 24-part architecture series the
//! person supplied; Part 01, "Protocol Extension System Architecture",
//! already has its own slice in this workspace —
//! `siar-protocol-ext` — and Part 03, "Transport Routing Policy
//! Engine Architecture", does not yet have a dedicated crate built
//! against *that* specific document, though a related, independently
//! evolved system, `siar-routing`, already covers similar ground
//! against a different, earlier design doc — see that crate's own top
//! doc comment).
//!
//! ## What's real here (implemented against the spec text, not guessed)
//!
//! - [`root_key`] — §5 "Account Identity", §6 "Root Key Strategy":
//!   [`root_key::RootIdentityKey`] signs rarely (device certificates
//!   and directory snapshots only), never per-session — the spec's own
//!   explicit alternative to "root key used for every message/session."
//! - [`capability`] — §14 "Device Capability Set":
//!   [`capability::DeviceCapabilitySet`], a bitset rather than an open
//!   string set (Part 01 §9's reasoning against arbitrary strings for
//!   hot capability checks, applied here to per-device authorization).
//! - [`certificate`] — §8 "Device Certificate", §9 "Device Certificate
//!   Semantics", §30 "Device Expiry":
//!   [`certificate::DeviceCertificate::issue`]/`verify_signature`
//!   enforce §9's rule that a valid signature proves account
//!   membership only, not current trust — [`certificate::DeviceCertificate::is_expired`]
//!   is a deliberately separate check (§30: "expiration is not a
//!   replacement for revocation").
//! - [`directory`] — §52 "Device Directory", §53 "Device Directory
//!   Entry": [`directory::DeviceDirectory`] as a signed snapshot (§52's
//!   own permitted alternative to a full event log) plus
//!   [`directory::DeviceDirectory::active_devices`], the §26 fan-out
//!   rule made concrete.
//! - [`trust_store`] — §55 "Stale Device Directory", §56 "Rollback
//!   Protection", §29 "Revocation Conflict Rules":
//!   [`trust_store::TrustedAccountStore::accept`] tracks the highest
//!   trusted generation per account and rejects anything at or below
//!   it — including, per a dedicated test, the spec's own named attack
//!   scenario: a revoked device regaining authority by replaying a
//!   stale, pre-revocation directory.
//!
//! Every one of the above is covered by tests that exercise the actual
//! cryptographic round trip (real Ed25519 keys, real signatures, real
//! rejection of tampered/forged/stale input) — not just type shapes.
//!
//! ## A real, deliberate divergence: two device-certificate models
//!
//! This workspace already has a device-linking system —
//! `siar_domain::device::{DeviceEvent, DeviceRegistry}` plus
//! `siar_crypto::device_cert::{DeviceCertificate, issue_device_certificate,
//! verify_device_certificate}` — built against a different, earlier
//! design document ("plan.md §38–42"). That system is
//! device-vouches-for-device: an already-trusted device signs a new
//! device's keys directly, with no account root key anywhere in the
//! model (that crate's own doc comment calls this a deliberate choice,
//! "avoids inventing a key-hierarchy this plan never specified").
//!
//! Part 02's spec explicitly asks for the opposite: a root identity key
//! that signs every device certificate (§6), used rarely, with
//! independent device keys never signing for each other. This crate
//! implements *that* model, under different type names
//! ([`certificate::DeviceCertificate`] here vs.
//! `siar_crypto::device_cert::DeviceCertificate`) in a different crate,
//! so nothing existing is silently replaced, broken, or shadowed.
//!
//! Reconciling the two — migrating the existing device-linking call
//! sites in `siar-messaging`/`apps/*` onto this root-key model, keeping
//! both for different trust contexts, or deciding the existing
//! simpler model is sufficient and retiring this one — is a genuine
//! product/architecture decision, not a mechanical follow-up. It is
//! deliberately not made here.
//!
//! ## What's explicitly NOT here
//!
//! - **No wire integration.** Nothing here touches `siar-protocol`,
//!   `siar-messaging`, or any JNI/app call site. This is a standalone
//!   policy layer, same posture `siar-protocol-ext` took toward
//!   `siar-messaging`'s existing traffic.
//! - **No persistent storage.** [`trust_store::TrustedAccountStore`] is
//!   in-memory (a `HashMap`) — real durability (§56: "or stronger state
//!   continuity") would mean a `siar-storage` repository, not attempted
//!   here.
//! - **No linking flow.** §15–20 (QR linking, NFC linking, numeric
//!   verification, linking trust decisions) — the actual UX/protocol by
//!   which a new device *obtains* a certificate — isn't implemented;
//!   this crate only covers what a certificate/directory *is* and how
//!   it's verified once issued.
//! - **No revocation event log, no offline propagation.** §22 "Device
//!   Addition Event" (a `DeviceEvent` enum distinct from
//!   `siar_domain::device::DeviceEvent`), §27–29's live propagation
//!   mechanics, §40 multi-device fan-out beyond the directory-filtering
//!   already in [`directory::DeviceDirectory::active_devices`], §33–39
//!   root key rotation/recovery, §41–51 (sender attribution,
//!   organizations, application namespaces), §57 fork detection, §60
//!   onward (linking authority, device roles, presence, secure storage,
//!   recovery, migration, and everything past — the spec runs to 204
//!   sections; this crate stops at a deliberately small, real Phase
//!   1/2 slice per its own §201 "Implementation Phases").
//! - **Parts 01's remaining ~90 sections past what `siar-protocol-ext`
//!   covers, and all of Part 03**, are unstarted by this crate (Part 01
//!   has its own crate and doc comment; Part 03 has no dedicated crate
//!   against its specific spec text at all — see this comment's own
//!   opening paragraph).

pub mod capability;
pub mod certificate;
pub mod directory;
pub mod error;
pub mod root_key;
pub mod trust_store;

pub use capability::DeviceCapabilitySet;
pub use certificate::DeviceCertificate;
pub use directory::{DeviceDirectory, DeviceDirectoryEntry, DeviceStatus};
pub use error::IdentityError;
pub use root_key::{RootIdentityKey, RootPublicKey};
pub use trust_store::TrustedAccountStore;
