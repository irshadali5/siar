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
//! - [`device_state`] — §71 "Device Presence"
//!   ([`device_state::AccountPresence`], per-device map preserved,
//!   never collapsed to derive "account reachable"), §72
//!   "Reachability vs Trust", §73 "Device State Type"
//!   ([`device_state::DeviceState`], verbatim three-field struct —
//!   trust and reachability stay independent fields, never merged
//!   into one boolean), §74 "Device Lifecycle"
//!   ([`device_state::DeviceLifecycle`], verbatim five states — **a
//!   real, documented gap**: this is richer than
//!   [`directory::DeviceStatus`] (3 variants), which is what's
//!   actually embedded in the signed directory every function in this
//!   crate operates on; see that type's own doc comment for why
//!   retrofitting `DeviceStatus` itself wasn't done this round), §75
//!   "Suspension vs Revocation"
//!   ([`device_state::suspend_device`]/[`device_state::reinstate_suspended_device`],
//!   genuinely separate functions from
//!   [`revocation::revoke_device`] — reversible, tested doing so).
//! - [`device_flows`] — §76 "Lost Device Flow"
//!   ([`device_flows::handle_lost_device`], revokes now, names the
//!   three steps this crate can't do itself so a caller can't assume
//!   revocation alone was enough), §77 "Compromised Device Flow"
//!   ([`device_flows::handle_compromised_device`], one step longer
//!   than §76's own list — never claims key erasure, matching §77's
//!   own explicit warning that revocation alone doesn't erase
//!   anything already obtained), §78 "Device Reinstallation", §79
//!   "Device Migration" ([`device_flows::migrate_device`], structurally
//!   cannot copy a private key — its only key parameter is a public
//!   key — and rejects migrating a device to its own id, tested).
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
//! - [`contact_verification`] — §91 "Offline Identity Verification"
//!   ([`contact_verification::OfflineVerification::bind_root_identity`],
//!   the one constructor, binding a [`root_key::RootPublicKey`] rather
//!   than any transport/session value — §91's own "prevents
//!   re-verifying every transport change" only holds if that's true
//!   structurally), §92 "Contact Verification"
//!   ([`contact_verification::VerifiedContact`], `verified_root` vs
//!   `current_root` kept as two separate fields so root-rotation
//!   awareness — [`contact_verification::VerifiedContact::is_continuous`] —
//!   is a real comparison, not a claim; trust inheritance
//!   ([`contact_verification::VerifiedContact::new_device_inherits_trust`])
//!   requires both continuity AND `TrustAccountRoot`, never one alone),
//!   §93/§94 "New Device"/"Identity Change Notification"
//!   ([`contact_verification::IdentityNotification`], two variants, not
//!   one generic event with a severity field — §94's own "the
//!   distinction must be explicit"; an identity change is
//!   security-significant under every [`contact_verification::VerificationPolicy`],
//!   a new device only under the two non-default ones), §95
//!   "Verification Modes" ([`contact_verification::VerificationPolicy`],
//!   verbatim three variants, `TrustAccountRoot` default per spec's own
//!   words), §96 "Device Transparency Log"
//!   ([`contact_verification::DeviceTransparencyLog`], a boundary trait
//!   only — spec calls this "a future enhancement" itself, so nothing
//!   here calls `append`, matching [`recovery::RecoveryKeyDerivation`]'s
//!   existing precedent for a real external capability this crate
//!   doesn't implement), §97 "Self-Hosted Transparency"
//!   ([`contact_verification::TransparencyDeploymentMode`], a pure
//!   label — hosting one changes nothing about verification, per §97's
//!   own words), §99 "Directory Service Role"
//!   ([`contact_verification::DirectoryServiceResponse::verify_and_accept`],
//!   the only way to accept one — routes through the exact same
//!   [`directory::DeviceDirectory::verify_signature`] every other path
//!   in this crate already requires, so a directory service gets no
//!   weaker acceptance path than any other untrusted source). §98 "No
//!   Mandatory Central Directory" gets no new code — see that module's
//!   own top-of-file note for why it's already true structurally.
//! - [`state_transport`] — §100 "Device State Through DTN"
//!   ([`state_transport::SignedDeviceStateUpdate`], reusing
//!   [`audit_log::IdentityAuditPayload`] directly as its wire payload
//!   rather than a parallel enum, so a relay that only stores and
//!   forwards opaque bytes can neither read nor forge a device-state
//!   change — verification needs only the account's root public key,
//!   never anything from the transport that carried it).
//! - [`principal_claims`] — §101 "Emergency Identity"
//!   ([`principal_claims::PrincipalType`], a plain label with no
//!   emergency business rules attached to any variant, per §101's own
//!   explicit instruction), §102 "Authority Identity" and §103/§104
//!   "Identity Claims"/"Claim Type"
//!   ([`principal_claims::IdentityClaim`], spec's own struct shape
//!   verbatim; [`principal_claims::IdentityClaim::is_valid`] is §102's
//!   "UI may display Verified Authority only when cryptographic policy
//!   validates it" made into one function — signature, expiry, AND a
//!   caller-supplied trusted-issuer list must all agree, so this crate
//!   never decides issuer trust on an application's behalf).
//! - [`discovery_privacy`] — §105/§106 "Privacy"/"Public vs Private
//!   Device Metadata" ([`discovery_privacy::DeviceMetadata`] vs
//!   [`discovery_privacy::PrivateDeviceMetadata`], two genuinely
//!   different types rather than one struct with fields a caller is
//!   trusted not to read pre-authentication), §107 "Rotating Discovery
//!   Tokens" ([`discovery_privacy::RotatingDiscoveryToken`], opaque
//!   BLAKE3-keyed-hash bytes with no embedded `AccountId`/`DeviceId`
//!   field to accidentally leak), §108 "Device Tracking Resistance"
//!   ([`discovery_privacy::TransportDiscoveryIdentity`], a marker
//!   trait seam for a real transport crate to implement, deliberately
//!   defining zero implementors here). §109 "Address Book Mapping"
//!   gets no type at all — see that module's own top-of-file note for
//!   why the absence, not a boundary type, is the correct fix. §110
//!   "Device Audit Log"/§111 "Audit Event Type" needed no new code
//!   either: [`audit_log`]'s five existing event constructors already
//!   cover §110's exact five named items, and its
//!   [`audit_log::IdentityAuditPayload`] already covers §111.
//! - [`reconciliation`] — §113 "Cross-Device Consistency"
//!   ([`reconciliation::ConvergenceStatus`], a real three-way
//!   comparison rather than a boolean, so "conservative during
//!   divergence" survives as distinct information), §114
//!   "Reconciliation" ([`reconciliation::ReconciliationPlan`], a
//!   decision only — never itself requests, transmits, or applies
//!   anything), §119 "Idempotency"
//!   ([`reconciliation::EventDeduplicator`], keyed on an event's own
//!   BLAKE3 hash), §120 "Replay Protection" (no new mechanism — a
//!   pointer to the four existing checks that already cover it, plus
//!   [`reconciliation::ReplayProtectionIndex`] for tracking
//!   highest-seen-generation before a live directory exists). §112
//!   "Notifications" and §115 "Merkle / Hash Chain Support" needed no
//!   new code — see that module's own top note for why both are
//!   already true.
//! - [`storage`] — §116 "Identity Storage" / §117 "Storage Interface"
//!   ([`storage::IdentityStore`], a trait boundary with no
//!   implementation, matching [`secure_storage::SecureStore`]'s own
//!   precedent exactly, including its `-> impl Future<...> + Send`
//!   shape — a real store is backend-specific and out of scope for
//!   this dependency-minimal crate).
//! - [`transaction`] — §118 "Transaction Boundaries"
//!   ([`transaction::CertificateVerified`] →
//!   [`transaction::EventAppended`] → [`transaction::SnapshotUpdated`]
//!   → [`transaction::Committed`], a type-state machine where
//!   `Committed::into_audit_event` is the only way to get a
//!   `DeviceLinked` audit payload out of this module — "never emit
//!   before durable persistence" is a compile-time property of this
//!   path, not a comment).
//! - [`device_authorization`] — §121 "Device-Specific Authorization"
//!   ([`device_authorization::DeviceAuthorizationDecision::combine`],
//!   a pure combinator where the device's own capability set is a
//!   hard ceiling no user/network policy can override, and the
//!   tightest of any caller-supplied size limits wins).
//! - [`enterprise_policy`] — §122 "Enterprise Device Policy"
//!   ([`enterprise_policy::EnterpriseDevicePolicy`], `Default` requires
//!   nothing at all — "do not make attestation mandatory for the core
//!   protocol" made structural), §123 "Platform Attestation"
//!   ([`enterprise_policy::PlatformAttestation`], no field or method
//!   resembling identity anywhere on it — "must not replace
//!   cryptographic identity" is enforced by absence, not a comment),
//!   §124 "Device Health Claims"
//!   ([`enterprise_policy::DeviceHealthClaims`], every field
//!   self-reported and unverified, matching spec's "claims... not
//!   identity itself").
//! - [`wire_limits`] — §127/§128 "Input Limits"/"Device Count Policy"
//!   ([`wire_limits::InputLimits`], one configurable value rather than
//!   scattered constants, checked against real
//!   [`directory::DeviceDirectory`]/[`principal_claims::IdentityClaim`]
//!   data before any of it is retained). §126 "Serialization" needed
//!   no new code — a real audit (not an assumption) found zero
//!   `usize`/`SystemTime` fields on this crate's wire types. §125
//!   "Version Compatibility" is HONESTLY NOT fully closed: this
//!   crate's two most load-bearing wire types
//!   ([`certificate::DeviceCertificate`], [`directory::DeviceDirectory`])
//!   still carry no explicit schema-version field, and adding one now
//!   would break their already-shipped, already-tested signed-payload
//!   bytes — the same category of gap this crate already names openly
//!   for Part 28's `DeviceLinkInvite`. [`wire_limits::SchemaVersion`]
//!   exists so new wire types follow §125's convention going forward;
//!   the two pre-existing types' gap is named, not fixed, here.
//! - [`session_cache`] — §129 "Session Cache"
//!   ([`session_cache::SessionCacheEntry::is_invalidated_by`], checked
//!   against a live [`directory::DeviceDirectory`] rather than trusting
//!   the session's own fields, since a session invalidated by exactly
//!   one of §129's three named triggers still looks internally
//!   consistent on its own), §130 "Revocation Cache"
//!   ([`session_cache::RevocationCache::from_directory`], the only
//!   constructor — "must be derived from durable authenticated state"
//!   means there is no path to insert a device id into this cache that
//!   didn't come from a real signed directory).
//! - [`client_api`] — §131 "Device Identity API"
//!   ([`client_api::IdentityClient`], [`client_api::DeviceClient`],
//!   [`client_api::TrustClient`], [`client_api::RecoveryClient`],
//!   grounded in this crate's real existing functions rather than
//!   inventing parallel ones — §131's own `LinkPolicy` becomes
//!   [`linking_authority::LinkingAuthorityPolicy`], the type that
//!   already fills that role). §132 "Example API" needed no new type
//!   — its rule ("application should not directly construct signed
//!   device certificates") is exactly why these traits exist. §133
//!   "Revoke API" adds [`client_api::RevocationReason`], with an
//!   honest scope note that it is NOT yet threaded into
//!   [`audit_log::IdentityAuditPayload::DeviceRevoked`] — that's real
//!   future work, not silently assumed done. §134 "Recovery API"
//!   ([`client_api::RecoveryClient::begin_recovery`] returns the
//!   INITIAL [`recovery_state_machine::RecoveryState`], not a finished
//!   result — "guided state machine, not one monolithic call" made
//!   structural).
//! - [`linking_state_machine`] — §135 "State Machine for Linking"
//!   ([`linking_state_machine::LinkingState::advance`], guarded enum
//!   transitions matching [`device_state::DeviceLifecycle::advance`]'s
//!   own precedent exactly; all 8 success states plus all 4 named
//!   failure states, each failure reachable only from the states
//!   where it plausibly occurs, and none reachable once a certificate
//!   is actually issued).
//! - [`recovery_state_machine`] — §136 "State Machine for Recovery"
//!   ([`recovery_state_machine::RecoveryState::advance`], a strictly
//!   linear six-state chain — spec names no failure states here,
//!   unlike §135, so none are invented; rejection already happens one
//!   layer up, before this state machine is ever entered).
//! - [`platform_boundary`] — §137 "UI Boundary"
//!   ([`platform_boundary::DeviceListVm`],
//!   [`platform_boundary::DeviceLinkVm`],
//!   [`platform_boundary::SecurityIdentityVm`],
//!   [`platform_boundary::RecoveryVm`] — none of these types can name
//!   [`root_key::RootIdentityKey`] or any other secret-holding type,
//!   because none of those types are imported into that file at all;
//!   "never receives a private key" is enforced by absence, not a
//!   runtime check). §138 "Android Kotlin Boundary"/§139 "iOS
//!   Boundary" needed no new code — see that module's own top note for
//!   why both are already true of this crate's existing scope.
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
pub mod client_api;
pub mod contact_verification;
pub mod destination;
pub mod device_authorization;
pub mod device_classes;
pub mod device_flows;
pub mod device_keys;
pub mod device_state;
pub mod directory;
pub mod discovery_privacy;
pub mod enterprise_policy;
pub mod error;
pub mod fanout;
pub mod invite;
pub mod link_key;
pub mod linking_authority;
pub mod linking_state_machine;
pub mod namespace;
pub mod platform_boundary;
pub mod principal_claims;
pub mod reconciliation;
pub mod recovery;
pub mod recovery_state_machine;
pub mod revocation;
pub mod root_key;
pub mod root_rotation;
pub mod rotation;
pub mod safety_fingerprint;
pub mod secure_storage;
pub mod session_cache;
pub mod state_chain;
pub mod state_transport;
pub mod storage;
pub mod transaction;
pub mod trust_store;
pub mod verification_code;
pub mod wire_limits;

pub use approval::{LinkMethod, LinkingApprovalPrompt, VerificationStatus};
pub use audit_log::{
    decode_audit_payload, device_linked_event, device_revoked_event, device_rotated_event,
    device_suspended_event, fork_detected_event, identity_stream_id, is_audited_status,
    recovery_used_event, revocation_verified_event, root_rotated_event, IdentityAuditPayload,
    EVENT_TYPE_DEVICE_LINKED, EVENT_TYPE_DEVICE_REVOKED, EVENT_TYPE_DEVICE_ROTATED,
    EVENT_TYPE_DEVICE_SUSPENDED, EVENT_TYPE_FORK_DETECTED, EVENT_TYPE_RECOVERY_USED,
    EVENT_TYPE_REVOCATION_VERIFIED, EVENT_TYPE_ROOT_ROTATED,
};
pub use capability::DeviceCapabilitySet;
pub use certificate::DeviceCertificate;
pub use client_api::{DeviceClient, IdentityClient, RecoveryClient, RevocationReason, TrustClient};
pub use contact_verification::{
    DeviceTransparencyChange, DeviceTransparencyEntry, DeviceTransparencyLog,
    DirectoryServiceResponse, IdentityNotification, OfflineVerification, OfflineVerificationMethod,
    TransparencyDeploymentMode, VerificationPolicy, VerifiedContact,
};
pub use destination::{
    large_file_default_fan_out_policy, messaging_default_fan_out_policy, resolve_destination,
    spec_70_example_target, Destination, FanOutPolicy, ResolvedDevice, SyncTarget,
};
pub use device_authorization::DeviceAuthorizationDecision;
pub use device_classes::{
    headless_device_trust_class, spec_45_example_classification, DeviceTrustClass,
    HeadlessDeviceOwner, OrganizationDeviceRole, ServiceIdentityKind,
};
pub use device_flows::{
    handle_compromised_device, handle_lost_device, migrate_device, CompromisedDeviceOutcome,
    CompromisedDeviceStep, LostDeviceOutcome, LostDeviceStep,
};
pub use device_keys::{generate_new_device_keys, NewDeviceKeys, NewDevicePublicKeys};
pub use device_state::DeviceLifecycle;
pub use device_state::{
    reinstate_suspended_device, suspend_device, AccountPresence, DevicePresence,
    DeviceReachability, DeviceState, DeviceTrustState, InvalidLifecycleTransition,
};
pub use directory::{DeviceDirectory, DeviceDirectoryEntry, DeviceEndpoint, DeviceStatus};
pub use discovery_privacy::{
    DeviceMetadata, DiscoveryTokenEpoch, PrivateDeviceMetadata, RotatingDiscoveryToken,
    TransportDiscoveryIdentity,
};
pub use enterprise_policy::{
    DeviceHealthClaims, EnterpriseDevicePolicy, EnterprisePolicyViolation, Platform,
    PlatformAttestation,
};
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
pub use linking_state_machine::{InvalidLinkingTransition, LinkingState};
pub use namespace::{
    device_membership_is_isolated, is_shared_across_applications_by_default,
    AccountIsolationDomain, ApplicationNamespace, ApplicationScopedResource,
    CrossApplicationIdentityMode, LocalAccountSession,
};
pub use platform_boundary::{
    DeviceLinkVm, DeviceListVm, DeviceSummaryVm, RecoveryVm, SecurityIdentityVm,
};
pub use principal_claims::{ClaimType, ClaimValue, IdentityClaim, IssuerId, PrincipalType};
pub use reconciliation::{
    state_hash_of, ConvergenceStatus, EventDeduplicator, ReconciliationPlan, ReplayProtectionIndex,
};
pub use recovery::{
    add_device_via_recovery, DerivedRecoveryKey, RecoveryError, RecoveryEvidence,
    RecoveryKeyDerivation, RecoveryPolicy, RecoverySecret,
};
pub use recovery_state_machine::{InvalidRecoveryTransition, RecoveryState};
pub use revocation::{revoke_device, verify_revocation, RevocationError};
pub use root_key::{RootIdentityKey, RootPublicKey};
pub use root_rotation::{
    rotate_root_key, verify_root_rotation, CompromisedRootRecoveryStrategy, RootRotation,
    RootRotationError,
};
pub use rotation::{rotate_device_key, RotationError, RotationReason};
pub use safety_fingerprint::SafetyFingerprint;
pub use secure_storage::{
    verify_device_session_presentation, DevicePrekeyBundle, DeviceSessionPresentation,
    KeyDerivationDomain, LocalDatabaseKey, OneTimePrekey, PrekeyPool, SecretBytes, SecretKeyId,
    SecureStore, SecureStoreError, SessionAuthenticationError, SignedPrekey, StalePeerPolicy,
};
pub use session_cache::{RevocationCache, SessionCacheEntry, SessionId};
pub use state_chain::{AccountStateEvent, DeviceEvent, StateHash};
pub use state_transport::SignedDeviceStateUpdate;
pub use storage::IdentityStore;
pub use transaction::{CertificateVerified, Committed, EventAppended, SnapshotUpdated};
pub use trust_store::TrustedAccountStore;
pub use verification_code::derive_verification_code;
pub use wire_limits::{InputLimitViolation, InputLimits, SchemaVersion};
