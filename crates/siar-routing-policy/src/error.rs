//! §194 "Error Types", §195 "No `anyhow` in Public Routing API".

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoutingError {
    #[error("no candidate path survived hard-constraint elimination")]
    NoEligibleCandidates,
    #[error("destination account has no active devices to route to")]
    NoActiveDevicesForAccount,
    /// §47/§48: no trusted directory exists for the account a
    /// candidate claims to belong to — nothing to authenticate
    /// against, so the candidate is rejected rather than assumed safe.
    #[error("no trusted directory for this destination account")]
    UnknownDestination,
    /// §48: the directory exists, but doesn't currently trust this
    /// device — covers both "never was a member" and "was revoked
    /// since" identically (see [`crate::security`]'s own doc comment).
    #[error("device is not a currently-trusted member of this account")]
    UnauthorizedDevice,
}
