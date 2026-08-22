//! §194 "Error Types", §195 "No `anyhow` in Public Routing API".

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoutingError {
    #[error("no candidate path survived hard-constraint elimination")]
    NoEligibleCandidates,
    #[error("destination account has no active devices to route to")]
    NoActiveDevicesForAccount,
}
