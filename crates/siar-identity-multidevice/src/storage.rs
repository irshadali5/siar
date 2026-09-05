//! §116 "Identity Storage", §117 "Storage Interface".
//!
//! Same posture this crate already takes for
//! [`crate::secure_storage::SecureStore`] (§81): a trait boundary with
//! no implementation, because a real store is backend-specific
//! (sqlite/sled/platform keychain/whatever `siar-storage` or an
//! application chooses) and this crate stays dependency-minimal by
//! design. §117's own snippet already draws the boundary this crate
//! must respect — "keep secret store separate: SecureStore" — so this
//! module defines [`IdentityStore`] for the tables §116 names and
//! deliberately does NOT let it touch anything
//! [`crate::secure_storage::SecureStore`] already owns (session keys,
//! local database keys). `prekey_bundles` appears in §116's own table
//! list, but its actual type
//! ([`crate::secure_storage::DevicePrekeyBundle`]) is public, not
//! secret material — see that module's own §85
//! "`directory_never_carries_session_keys`" note for the exact line
//! this crate already draws between the two.

use std::future::Future;

use siar_domain::{AccountId, DeviceId};

use crate::certificate::DeviceCertificate;
use crate::directory::DeviceDirectory;
use crate::error::IdentityError;
use crate::secure_storage::DevicePrekeyBundle;
use crate::state_chain::AccountStateEvent;

/// §117's own five methods, verbatim signatures, made concrete with
/// this crate's real types instead of spec's `(...)` placeholders.
/// `-> impl Future<...> + Send` rather than native `async fn`,
/// matching [`crate::secure_storage::SecureStore`]'s own precedent —
/// same reasoning: a `Send` bound so an implementation is usable
/// across an executor's worker threads, which native async-fn-in-trait
/// doesn't give without exactly this desugaring. No implementation is
/// provided — same as `SecureStore`, this is the seam an application
/// or `siar-storage` fills in.
pub trait IdentityStore {
    /// Current directory snapshot for an account, if this store has
    /// one at all yet.
    fn account_state(
        &self,
        account_id: AccountId,
    ) -> impl Future<Output = Result<Option<DeviceDirectory>, IdentityError>> + Send;

    /// §116's `device_events` table — appends one entry to the
    /// account's state chain (see [`crate::state_chain`]'s own honest
    /// scope note on how far this chain concept is wired up today).
    fn append_device_event(
        &mut self,
        event: AccountStateEvent,
    ) -> impl Future<Output = Result<(), IdentityError>> + Send;

    /// §116's `device_certificates` table.
    fn device_certificate(
        &self,
        device_id: DeviceId,
    ) -> impl Future<Output = Result<Option<DeviceCertificate>, IdentityError>> + Send;

    /// §116's `trust_state` table — deliberately opaque bytes here
    /// rather than a named type: which of this workspace's three
    /// documented "trust state"-shaped types (see this crate's own
    /// `lib.rs` doc comment) a given implementation persists is a
    /// choice for that implementation and its caller, not something
    /// this storage-boundary trait should hard-code.
    fn trust_state(
        &self,
        account_id: AccountId,
    ) -> impl Future<Output = Result<Option<Vec<u8>>, IdentityError>> + Send;

    fn save_trust_state(
        &mut self,
        account_id: AccountId,
        state: Vec<u8>,
    ) -> impl Future<Output = Result<(), IdentityError>> + Send;

    /// Not in §117's own five-method list, but §116 names
    /// `prekey_bundles` as its own table and nothing above reaches it
    /// — added so `IdentityStore`'s method set actually covers every
    /// non-secret table §116 names, rather than silently dropping one.
    fn prekey_bundle(
        &self,
        device_id: DeviceId,
    ) -> impl Future<Output = Result<Option<DevicePrekeyBundle>, IdentityError>> + Send;
}
