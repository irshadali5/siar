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
//! - [`link_key`] — §16/§17's `ephemeral_link_key`:
//!   [`link_key::EphemeralLinkKeyPair`], a real X25519 keypair
//!   generated fresh per linking attempt, with real Diffie-Hellman
//!   agreement (mirrors `siar_crypto::DeviceIdentity`'s own
//!   `x25519_dalek` usage, duplicated rather than imported — this
//!   crate stays independent of `siar-crypto`, see below).
//! - [`invite`] — §16 "Device Linking Invitation":
//!   [`invite::DeviceLinkInvite`], root-key-signed, with a real,
//!   internally-generated nonce (a caller can't accidentally reuse
//!   one — §16's "one-time" requirement) and
//!   [`invite::DeviceLinkInvite::contains_no_secret_material`] making
//!   §17's "the QR should not contain private/session keys" rule a
//!   checkable property, not just a followed convention.
//! - [`verification_code`] — §19 "Numeric Verification":
//!   [`verification_code::derive_verification_code`] is a real
//!   transcript-derived (not random) 6-digit code — the invite's own
//!   signed content plus both ephemeral public keys plus the derived
//!   Diffie-Hellman shared secret, so an attacker observing only the
//!   public handshake traffic can't precompute it.
//! - [`approval`] — §20 "Linking Trust Decision":
//!   [`approval::LinkingApprovalPrompt`] carries every field §20 says
//!   must be shown before approval, with a real, if narrow, guardrail
//!   against "silent device addition" — the ordinary constructor can
//!   only produce a `NumericCodeConfirmed` prompt; an unverified one
//!   requires calling a differently-named, harder-to-reach-by-accident
//!   function instead.
//! - [`revocation`] — §25 "Device Revocation", §26 "Revocation
//!   Semantics" (the check half —
//!   [`directory::DeviceDirectory::is_device_trusted`]), §27
//!   "Immediate Local Revocation": [`revocation::revoke_device`] is the
//!   piece that was missing before this round — `DeviceStatus::Revoked`
//!   was a value a directory could hold, and
//!   [`trust_store::TrustedAccountStore`] already rejected a stale
//!   directory trying to un-revoke a device (§29, tested since the
//!   original session), but nothing actually *produced* a revocation
//!   until now. [`revocation::verify_revocation`] independently
//!   double-checks the result (generation advanced, target really
//!   revoked, every other device's status untouched) rather than
//!   trusting `revoke_device`'s own return value blindly.
//! - [`rotation`] — §31 "Device Rotation"
//!   ([`rotation::rotate_device_key`], mirroring `revoke_device`'s
//!   exact shape: same generation-advances-by-one discipline, same
//!   real-error-not-silent-noop handling for an unknown or already-
//!   revoked device), §32 "Rotation Reasons"
//!   ([`rotation::RotationReason`], verbatim five reasons). See that
//!   module's own doc comment for why "device key generation N -> N+1"
//!   is modeled via the directory's existing generation counter rather
//!   than a second, invented per-device counter.
//! - [`root_rotation`] — §33 "Root Key Rotation", §34 "Root Rotation
//!   Event" ([`root_rotation::rotate_root_key`]/[`root_rotation::verify_root_rotation`],
//!   a dual-signed continuity attestation — old root authorizes,
//!   new root accepts, both signatures cover the same payload), §35
//!   "Compromised Root Scenario"
//!   ([`root_rotation::CompromisedRootRecoveryStrategy`], spec's own
//!   five named future candidates — only two get real implementations,
//!   see [`recovery`] — plus a test making "rotation structurally
//!   requires the old private key" visible, not just true by
//!   inspection).
//! - [`recovery`] — §36 "Recovery Architecture", §37 "Recovery Policy
//!   Type" ([`recovery::RecoveryPolicy`], verbatim four variants), §38
//!   "Recovery Secret" ([`recovery::RecoverySecret`], deliberately not
//!   `Serialize`/`Deserialize` so nothing can put it on the wire by
//!   accident; [`recovery::RecoveryKeyDerivation`] is the boundary
//!   trait to a real Argon2id implementation this dependency-minimal
//!   crate doesn't provide itself), §39 "Recovery Device Addition"
//!   ([`recovery::add_device_via_recovery`], provably the same
//!   certificate-issuance path [`rotation::rotate_device_key`]/
//!   [`revocation::revoke_device`] already use, gated by recovery
//!   evidence instead of an existing device's approval — tested for
//!   both the `RecoverySecret` and `TrustedDeviceQuorum` policies,
//!   including a revoked device's signature correctly not counting
//!   toward quorum).
//! - [`fanout`] — §40 "Multi-Device Messaging Fan-Out"
//!   ([`fanout::fan_out_targets`], recipient's active devices plus the
//!   sender's OTHER active devices, never the originating one), §41
//!   "Sender Attribution" ([`fanout::SenderIdentity`], verbatim), §42
//!   "Account-Level Presentation" ([`fanout::account_level_display`],
//!   spec's own four device-revealing contexts vs. the one that
//!   doesn't), §43 "Device-Level Receipts"
//!   ([`fanout::aggregate_delivered_to_account`], only ever derived
//!   from real [`fanout::DeviceReceipt`]s, never stored on its own —
//!   "the core should retain device-level truth" enforced
//!   structurally), §44 "Sync Between User's Own Devices"
//!   ([`fanout::OwnDeviceSyncPolicy`], per-[`fanout::SyncDataClass`],
//!   defaults to NOT synced for anything unconfigured).
//! - [`device_classes`] — §45 "Device Trust Classes"
//!   ([`device_classes::DeviceTrustClass`], verbatim four, plus spec's
//!   own four worked examples reproduced as a test), §46 "Headless
//!   Devices" ([`device_classes::HeadlessDeviceOwner`], verbatim
//!   three), §47 "Service Identities"
//!   ([`device_classes::ServiceIdentityKind`], a label only — tested
//!   proving a "service identity" uses the exact same
//!   `RootIdentityKey`/`DeviceDirectory` a normal account does, no
//!   separate authentication model), §48 "Organization Identity"
//!   ([`device_classes::OrganizationDeviceRole`], verbatim four —
//!   "identity proves membership, authorization decides what it may
//!   do" is already true across this workspace: this crate has zero
//!   dependency on `siar-protocol-ext::ExtensionAuthorization`, spec
//!   01 §33's real authorization-decision trait).
//! - [`namespace`] — §49 "Multiple Accounts on One Device"
//!   ([`namespace::device_membership_is_isolated`], checking no two
//!   [`namespace::LocalAccountSession`]s share a `DeviceId` — a real,
//!   checkable form of "isolated device membership"), §50
//!   "Application Namespace" ([`namespace::ApplicationNamespace`],
//!   open string newtype; [`namespace::is_shared_across_applications_by_default`]
//!   always `false` for all four of spec's named resources), §51
//!   "Cross-Application Identity Reuse"
//!   ([`namespace::CrossApplicationIdentityMode`], whose literal
//!   `Default` impl is `IsolatedPerApp` — "the default should favor
//!   isolation" made structural, not just documented).
//! - §52 "Device Directory", §53 "Device Directory Entry": already
//!   built (see [`directory`]'s own top-of-file doc) — this round adds
//!   the one field that pass was missing,
//!   [`directory::DeviceDirectoryEntry::transport_endpoints`] (spec's
//!   own conceptual struct always had it). Kept fully opaque
//!   ([`directory::DeviceEndpoint`], a `Vec<u8>` newtype) rather than
//!   typed against any real transport's address format — this crate
//!   has no transport dependency and shouldn't gain one just to type
//!   this field.
//! - §54 "Device Directory Synchronization": no new code — "the
//!   directory is signed, so transport is not trusted for
//!   authenticity" is already true of every code path in [`directory`]
//!   ([`directory::DeviceDirectory::verify_signature`] is the only way
//!   a directory is ever accepted, regardless of which of spec's six
//!   listed sync paths carried it here).
//! - §55 "Stale Device Directory", §56 "Rollback Protection": already
//!   built (see [`trust_store`]'s own top-of-file doc, written in an
//!   earlier round).
//! - §57 "Fork Detection": **a real bug found and fixed this round**,
//!   not just new coverage — [`trust_store::TrustedAccountStore::accept`]
//!   used to treat ANY same-generation resend as a harmless no-op,
//!   regardless of whether the content actually matched. That's
//!   exactly the "silently choose one" behavior spec §57 explicitly
//!   forbids for two genuinely different signed directories at the
//!   same generation. Fixed: same generation + same signature bytes
//!   (Ed25519 signing is deterministic, so identical content always
//!   produces identical signatures) is still a harmless resend;
//!   same generation + different signature now returns
//!   [`error::IdentityError::IdentityForkDetected`], per spec §57's
//!   own words, "do not silently choose one... require
//!   reconciliation/security handling."
//! - §58 "Concurrent Device Changes": no new code — this crate's
//!   existing design (one root key signs the entire directory snapshot
//!   per generation) already IS spec §58's own first listed option,
//!   "single account authority," which is exactly why forks are a
//!   `bug`/`compromise` signal here rather than a routine occurrence
//!   needing its own resolution protocol.
//! - [`state_chain`] — §59 "Account State Chain"
//!   ([`state_chain::AccountStateEvent`]/[`state_chain::StateHash`]/[`state_chain::DeviceEvent`],
//!   spec's own struct shape, bridgeable to a real
//!   [`directory::DeviceDirectory::state_hash`] — honestly scoped: this
//!   crate's live data path is still the signed-snapshot model, and
//!   `siar-event-log` (Part 04) doesn't yet implement hash-chaining
//!   itself either, so this type is a real, usable primitive toward
//!   §59, not a claim the chain exists end-to-end anywhere in this
//!   workspace yet).
//! - [`linking_authority`] — §60 "Device Linking Authority"
//!   ([`linking_authority::LinkingAuthorityPolicy`], verbatim four),
//!   §61 "Default Consumer Policy"
//!   ([`linking_authority::default_consumer_policy`]/[`linking_authority::default_enterprise_policy`]),
//!   §62 "Link Approval Certificate"
//!   ([`linking_authority::device_can_approve_links`], checked against
//!   the real `LINK_NEW_DEVICE` capability bit, not just "is Active" —
//!   tested including a revoked device with the bit still set), §63
//!   "Device Roles" ([`linking_authority::DeviceRole`], verbatim six,
//!   each mapped to a real [`capability::DeviceCapabilitySet`] via
//!   [`linking_authority::DeviceRole::default_capabilities`] — "not UI
//!   labels only" made real), §64 "Security Capabilities" (extended
//!   [`capability::DeviceCapabilitySet`] with its three still-missing
//!   named bits — `ROTATE_ACCOUNT_STATE`/`SYNC_HISTORY`/`RELAY`,
//!   alongside the five already there from an earlier round), §65
//!   "Principle of Least Authority"
//!   ([`linking_authority::headless_relay_minimum_capabilities`],
//!   spec's own worked example — a relay gets `RELAY` and nothing that
//!   could link, revoke, rotate account state, or send messages).
//! - [`destination`] — §66 "Multi-Device File Transfer", §67 "Account
//!   Address vs Device Address" ([`destination::Destination`],
//!   verbatim three-variant enum), §68 "Device Resolution"
//!   ([`destination::resolve_destination`], the real flow — directory
//!   lookup, active+capability-authorized filtering, transport
//!   endpoints attached — so "the application must manually maintain
//!   endpoint lists" never has to be true for a caller of this
//!   function), §69 "Fan-Out Policy"
//!   ([`destination::FanOutPolicy`], verbatim five, plus spec's own
//!   two named defaults for messaging vs. large files), §70
//!   "Own-Device Synchronization Policy"
//!   ([`destination::SyncTarget`]/[`destination::spec_70_example_target`]
//!   — a distinct axis from [`fanout::OwnDeviceSyncPolicy`] §44: that
//!   type gates whether a data class is trusted/synced at all, this
//!   one picks which devices once trust says yes; see this module's
//!   own doc comment for why they're kept separate types).
//! - [`device_keys`] — §21 "New Device Key Generation": the piece
//!   sitting between [`link_key`]'s ephemeral handshake key and
//!   [`certificate::DeviceCertificate::issue`]'s signature — before
//!   this round, nothing generated the actual permanent keys a
//!   newly-linked device would use going forward.
//!   [`device_keys::NewDeviceKeys`] bundles every key §21 lists (device
//!   signing key, transport key, local database key); only
//!   [`device_keys::NewDeviceKeys::public_keys`]'s output is meant to
//!   leave the device — there's no function anywhere in that module
//!   that returns or serializes a private key, matching §21's own
//!   "private keys remain local" rule structurally, not just by
//!   convention. A dedicated test runs the real §21 → §8 pipeline
//!   end to end: generate keys locally, certify only the public
//!   signing key, verify the resulting certificate.
//! - [`audit_log`] — a real cross-crate integration with
//!   `siar-event-log` (this same session's Part 04 crate), not itself
//!   named by a single spec section: constructs real
//!   `siar_event_log::NewEvent`s for the three identity operations
//!   this crate can already really perform (device linked, device
//!   revoked, revocation verified), closing the "no revocation event
//!   log" half of the gap this crate's own notes used to carry. See
//!   that module's own doc comment for why it only constructs events
//!   rather than appending them itself.
//!
//! Every one of the above is covered by tests that exercise the actual
//! cryptographic round trip (real Ed25519/X25519 keys, real signatures,
//! real Diffie-Hellman agreement, real rejection of tampered/forged/
//! stale/mismatched input) — not just type shapes. One test runs the
//! full realistic flow end to end: revoke a real device, accept the
//! result into a real `TrustedAccountStore`, then confirm a stale
//! pre-revocation directory is rejected — §29's own scenario, now
//! exercised against this round's real output instead of a hand-built
//! fixture standing in for one.
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
//!   here (and, separately, `siar-storage` itself needs rustc 1.87 —
//!   past what this pass's own sandbox environment could verify a
//!   build against, an additional real reason this wasn't attempted
//!   blind this round).
//! - **Linking flow covers §16-17/§19-20 only.** §18 NFC linking has no
//!   real proximity-transport code (NFC needs platform bindings this
//!   crate doesn't have — the same "no wire integration" posture every
//!   crate in this series takes); [`approval::LinkMethod::Nfc`] exists
//!   as a value an NFC-based flow would report, nothing constructs one.
//!   §21's key *generation* is real now (see [`device_keys`] above) —
//!   what's still not attempted is binding the generated transport
//!   public key into anything a root key signs (see
//!   [`device_keys::NewDeviceKeys::transport_public_key_bytes`]'s own
//!   doc comment for the real reason: `DeviceCertificate` only
//!   certifies one key today). §16's own "replay-resistant"
//!   requirement is only half-real: the nonce contributes real entropy
//!   to the signed payload, but nothing tracks *used* nonces to reject
//!   an actual replay — that needs a persistent store, the same gap
//!   [`trust_store::TrustedAccountStore`] already has.
//! - **Revocation event log now real; offline propagation still not.**
//!   §25-27's revocation *operation* is real (see [`revocation`]
//!   above), and now so is turning it into an audit trail: [`audit_log`]
//!   builds real `siar_event_log::NewEvent`s for device-linked/
//!   device-revoked/revocation-verified via that same workspace's
//!   `siar_event_log::EventStore` rather than a redundant event system
//!   inside this crate (§52's own "signed snapshot over event log"
//!   choice this crate already made — see `directory.rs`'s own doc
//!   comment). [`audit_log`] only *constructs* events, though — no
//!   caller here actually appends them to a live `EventStore`, since
//!   that's the caller's own I/O to own (see that module's own doc
//!   comment for why). §28's actual propagation transport (direct
//!   sync/relay/DTN/linked-device sync) is still fully unattempted —
//!   this crate has no wire integration for any of it, same posture
//!   every crate in this series takes. §40 multi-device fan-out beyond the
//!   directory-filtering already in
//!   [`directory::DeviceDirectory::active_devices`], §33–39
//!   root key rotation/recovery, §41–51 (sender attribution,
//!   organizations, application namespaces), §57 fork detection, §60
//!   onward (linking authority, device roles, presence, secure storage,
//!   recovery, migration, and everything past — the spec runs to 204
//!   sections; this crate stops at a deliberately small, real Phase
//!   1/2 slice per its own §201 "Implementation Phases", now extended
//!   with real slices of both the linking flow and revocation, neither
//!   of which that Phase list separately numbers).
//! - **Parts 01's remaining ~90 sections past what `siar-protocol-ext`
//!   covers, and all of Part 03**, are unstarted by this crate (Part 01
//!   has its own crate and doc comment; Part 03 has no dedicated crate
//!   against its specific spec text at all — see this comment's own
//!   opening paragraph).

pub mod approval;
pub mod audit_log;
pub mod capability;
pub mod certificate;
pub mod destination;
pub mod device_classes;
pub mod device_keys;
pub mod directory;
pub mod error;
pub mod fanout;
pub mod invite;
pub mod link_key;
pub mod linking_authority;
pub mod namespace;
pub mod recovery;
pub mod revocation;
pub mod root_key;
pub mod root_rotation;
pub mod rotation;
pub mod safety_fingerprint;
pub mod state_chain;
pub mod trust_store;
pub mod verification_code;

pub use approval::{LinkMethod, LinkingApprovalPrompt, VerificationStatus};
pub use audit_log::{
    decode_audit_payload, device_linked_event, device_revoked_event, identity_stream_id,
    is_audited_status, revocation_verified_event, IdentityAuditPayload, EVENT_TYPE_DEVICE_LINKED,
    EVENT_TYPE_DEVICE_REVOKED, EVENT_TYPE_REVOCATION_VERIFIED,
};
pub use capability::DeviceCapabilitySet;
pub use certificate::DeviceCertificate;
pub use destination::{
    large_file_default_fan_out_policy, messaging_default_fan_out_policy, resolve_destination,
    spec_70_example_target, Destination, FanOutPolicy, ResolvedDevice, SyncTarget,
};
pub use device_classes::{
    headless_device_trust_class, spec_45_example_classification, DeviceTrustClass,
    HeadlessDeviceOwner, OrganizationDeviceRole, ServiceIdentityKind,
};
pub use device_keys::{generate_new_device_keys, NewDeviceKeys, NewDevicePublicKeys};
pub use directory::{DeviceDirectory, DeviceDirectoryEntry, DeviceEndpoint, DeviceStatus};
pub use error::IdentityError;
pub use fanout::{
    account_level_display, aggregate_delivered_to_account, fan_out_targets, DeviceReceipt,
    DeviceReceiptStatus, OwnDeviceSyncPolicy, PresentationContext, SenderIdentity, SyncDataClass,
};
pub use invite::DeviceLinkInvite;
pub use link_key::{EphemeralLinkKeyPair, EphemeralLinkPublicKey};
pub use linking_authority::{
    default_consumer_policy, default_enterprise_policy, device_can_approve_links,
    headless_relay_minimum_capabilities, DeviceRole, LinkingAuthorityPolicy,
};
pub use namespace::{
    device_membership_is_isolated, is_shared_across_applications_by_default,
    AccountIsolationDomain, ApplicationNamespace, ApplicationScopedResource,
    CrossApplicationIdentityMode, LocalAccountSession,
};
pub use recovery::{
    add_device_via_recovery, DerivedRecoveryKey, RecoveryError, RecoveryEvidence,
    RecoveryKeyDerivation, RecoveryPolicy, RecoverySecret,
};
pub use revocation::{revoke_device, verify_revocation, RevocationError};
pub use root_key::{RootIdentityKey, RootPublicKey};
pub use root_rotation::{
    rotate_root_key, verify_root_rotation, CompromisedRootRecoveryStrategy, RootRotation,
    RootRotationError,
};
pub use rotation::{rotate_device_key, RotationError, RotationReason};
pub use safety_fingerprint::SafetyFingerprint;
pub use state_chain::{AccountStateEvent, DeviceEvent, StateHash};
pub use trust_store::TrustedAccountStore;
pub use verification_code::derive_verification_code;
