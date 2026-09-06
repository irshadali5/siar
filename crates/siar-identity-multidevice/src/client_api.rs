//! §131 "Device Identity API", §132 "Example API", §133 "Revoke API",
//! §134 "Recovery API".
//!
//! §131: "recommended public clients... keep public API small." Each
//! trait below has exactly the methods needed for the one example
//! spec itself shows (§132/§133/§134's code blocks) — grounded in this
//! crate's REAL existing functions
//! ([`crate::invite::DeviceLinkInvite::create`],
//! [`crate::revocation::revoke_device`],
//! [`crate::recovery::add_device_via_recovery`]) rather than inventing
//! parallel ones. Spec's own example names a `LinkPolicy` type this
//! crate doesn't have; [`crate::linking_authority::LinkingAuthorityPolicy`]
//! is the real type that already fills that role, used here instead of
//! inventing a duplicate.
//!
//! §132's own rule — "application should not directly construct
//! signed device certificates" — is why these are traits an
//! application calls into, not a suggestion to read
//! [`crate::certificate::DeviceCertificate::issue`]'s signature and
//! call it directly: an implementor of [`DeviceClient`] is the one
//! thing that holds the account's [`crate::root_key::RootIdentityKey`]
//! and actually calls `issue`/`revoke_device`/`add_device_via_recovery`
//! on the application's behalf. This crate provides no implementation
//! of any of these four traits — same posture as
//! [`crate::storage::IdentityStore`]/[`crate::secure_storage::SecureStore`]:
//! a real implementation needs actual persistence, actual root-key
//! custody, and actual notification delivery, none of which this
//! dependency-minimal crate owns.

use std::future::Future;

use siar_domain::{AccountId, DeviceId};

use crate::directory::DeviceDirectory;
use crate::error::IdentityError;
use crate::invite::DeviceLinkInvite;
use crate::linking_authority::LinkingAuthorityPolicy;
use crate::recovery::{RecoveryError, RecoveryEvidence};
use crate::recovery_state_machine::RecoveryState;

/// §131's first client — read-only account-level state, the one thing
/// every other client needs to look up before acting.
pub trait IdentityClient {
    fn current_directory(
        &self,
        account_id: AccountId,
    ) -> impl Future<Output = Result<Option<DeviceDirectory>, IdentityError>> + Send;
}

/// §131/§132's linking client. `create_device_link_invite` takes a
/// [`LinkingAuthorityPolicy`] rather than spec's named `LinkPolicy` —
/// see this module's own top note for why. `approve_device_link`
/// returns the whole updated [`DeviceDirectory`] (matching
/// [`crate::certificate::DeviceCertificate::issue`]'s own real return
/// shape one layer up), never the raw certificate — an application has
/// no reason to touch the certificate independent of the directory it
/// now belongs to.
pub trait DeviceClient {
    fn create_device_link_invite(
        &self,
        account_id: AccountId,
        inviter_device: DeviceId,
        policy: LinkingAuthorityPolicy,
    ) -> impl Future<Output = Result<DeviceLinkInvite, IdentityError>> + Send;

    fn approve_device_link(
        &mut self,
        invite: DeviceLinkInvite,
        new_device_public_key: [u8; 32],
    ) -> impl Future<Output = Result<DeviceDirectory, IdentityError>> + Send;
}

/// §133's own one named example, generalized to the small set of
/// reasons a revocation UI plausibly distinguishes — `Other(String)`
/// as the escape hatch, same pattern as
/// [`crate::principal_claims::ClaimType::Other`].
///
/// **Honest scope note**: this is deliberately NOT threaded into
/// [`crate::audit_log::IdentityAuditPayload::DeviceRevoked`] or the
/// signed [`DeviceDirectory`] itself this round — doing so would mean
/// adding a field to an already-shipped, already-tested audit-event
/// variant, the same category of change this crate is careful about
/// elsewhere (see §125's wire-versioning gap note in
/// [`crate::wire_limits`]). A `RevocationReason` is real, useful
/// client-facing/local-logging context; it is NOT yet part of the
/// durable, cross-device-synced record of why a device was revoked.
/// That integration is real future work, named here rather than
/// silently assumed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevocationReason {
    Lost,
    Compromised,
    Replaced,
    Other(String),
}

/// §131/§133's trust client.
pub trait TrustClient {
    /// §133's own five-item list of what "the library handles" —
    /// signed event ([`crate::revocation::revoke_device`]),
    /// generation (that function's own `current.generation + 1`,
    /// never caller-supplied), persistence (via whatever
    /// [`crate::storage::IdentityStore`] the implementor holds), sync
    /// notification (via [`crate::state_transport::SignedDeviceStateUpdate`]
    /// or an equivalent the implementor sends), and session
    /// invalidation (via [`crate::session_cache::SessionCacheEntry::is_invalidated_by`]
    /// against the returned directory) — this trait method is the one
    /// seam where an implementor is expected to actually do all five,
    /// none of which this crate can do on its own.
    fn revoke_device(
        &mut self,
        device_id: DeviceId,
        reason: RevocationReason,
    ) -> impl Future<Output = Result<DeviceDirectory, IdentityError>> + Send;
}

/// §131/§134's recovery client. §134: "recovery should be a guided
/// state machine, not one monolithic call" — `begin_recovery` returns
/// the INITIAL [`RecoveryState`] rather than a finished
/// [`DeviceDirectory`], so an application is structurally guided
/// through [`crate::recovery_state_machine`]'s states one at a time
/// instead of getting a single all-or-nothing result. Evidence
/// rejected by policy fails here, before any state machine begins —
/// [`crate::recovery::add_device_via_recovery`]'s own internal policy
/// check is that gate; §136's own six named states carry no failure
/// state precisely because the rejection path already happens here,
/// one layer up.
pub trait RecoveryClient {
    fn begin_recovery(
        &mut self,
        evidence: RecoveryEvidence,
    ) -> impl Future<Output = Result<RecoveryState, RecoveryError>> + Send;
}
