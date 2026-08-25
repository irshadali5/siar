//! §159 "Error Model".
//!
//! The spec sketches two overlapping error enums: §103
//! `CapabilityError` (mid-document, conceptual) and §159
//! `CapabilityNegotiationError` (in the "Suggested Crate Structure /
//! Error Model" section, alongside the crate's actual public API
//! list). §159's variant set is a strict superset relevant to this
//! crate (it adds `Conflict` for §70's mutually-exclusive capabilities
//! and `Cancelled`) and its name matches what §157's public API
//! surface implies negotiation failures should be called, so this
//! crate implements §159 as the one canonical error type rather than
//! both — a real reconciliation, not a silent drop of §103, which is
//! otherwise fully covered: `Malformed`→`Malformed`,
//! `UnsupportedCoreVersion`→`UnsupportedCore`,
//! `RequiredCapabilityMissing`→`MissingRequired`,
//! `VersionMismatch`→`VersionMismatch`,
//! `SecurityPolicyFailure`→`SecurityPolicy`,
//! `InconsistentDependency`→`DependencyViolation`,
//! `DowngradeSuspected`→`DowngradeSuspected`,
//! `LimitExceeded`→`LimitExceeded`,
//! `TranscriptMismatch`→`TranscriptMismatch`.

use crate::id::CapabilityId;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CapabilityNegotiationError {
    #[error("advertisement is malformed")]
    Malformed,
    #[error("unsupported core protocol version")]
    UnsupportedCore,
    /// §8, §68: a required capability the peer needs is missing —
    /// either genuinely unsupported, or (§68) required-but-unknown-
    /// to-this-registry, which must fail the same way, never silently.
    #[error("required capability missing: {0}")]
    MissingRequired(CapabilityId),
    #[error("capability version mismatch: {0}")]
    VersionMismatch(CapabilityId),
    #[error("capability rejected by local security policy: {0}")]
    SecurityPolicy(CapabilityId),
    /// §69: an advertised capability's declared dependency
    /// (`CapabilityDependency::requires`) is not itself present in the
    /// same advertisement.
    #[error("capability {capability} requires {missing}, which is not advertised")]
    DependencyViolation {
        capability: CapabilityId,
        missing: CapabilityId,
    },
    /// §70: two mutually-exclusive capabilities were both advertised
    /// as required with no resolvable selection.
    #[error("mutually exclusive capabilities both required: {0} and {1}")]
    Conflict(CapabilityId, CapabilityId),
    #[error("negotiated value exceeds a policy or wire limit")]
    LimitExceeded,
    #[error("downgrade suspected relative to remembered security baseline")]
    DowngradeSuspected,
    #[error("negotiation transcript hash mismatch between peers")]
    TranscriptMismatch,
    #[error("negotiation cancelled")]
    Cancelled,
}
