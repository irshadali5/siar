//! Extension lifecycle, traffic priority, and typed errors — spec §21
//! "Traffic Priority", §23 "Extension Lifecycle", §27 "Typed
//! Extension Errors".

/// spec §21, verbatim enum and verbatim ordering (`Critical` highest
/// down to `Background` lowest) — spec §22 "Fair Scheduling" is what
/// actually consumes an ordering like this ("weighted fair scheduling
/// plus strict bounded emergency override, rather than naive perpetual
/// highest-priority-first scheduling"); the scheduler itself is out of
/// scope for this crate (see this crate's top-level doc comment) — this
/// is only the priority vocabulary a future scheduler would consume.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum TrafficPriority {
    Critical,
    Control,
    Interactive,
    Normal,
    Bulk,
    Background,
}

/// spec §23's lifecycle diagram, as an explicit state machine —
/// "Lifecycle state must be explicit," spec §23's own stated
/// requirement for why this isn't left as an implicit consequence of
/// which fields happen to be populated. [`ExtensionLifecycle::advance`]
/// is this crate's enforcement of the diagram's actual edges (spec §23
/// shows exactly one path through `Registered -> Advertised ->
/// Negotiated -> Opening -> Active -> Closing -> Closed`, plus the
/// `Rejected`/`Unsupported`/`VersionMismatch`/`ProtocolError` failure
/// exits) — invalid transitions are a programming error caught at the
/// call site, not a silent state overwrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExtensionLifecycle {
    Registered,
    Advertised,
    Negotiated,
    Opening,
    Active,
    Closing,
    Closed,
    Rejected,
    Unsupported,
    VersionMismatch,
    ProtocolError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("cannot transition extension lifecycle from {from:?} to {to:?}")]
pub struct InvalidLifecycleTransition {
    pub from: ExtensionLifecycle,
    pub to: ExtensionLifecycle,
}

impl ExtensionLifecycle {
    /// Validates one step along spec §23's diagram. The four failure
    /// states (`Rejected`/`Unsupported`/`VersionMismatch`/
    /// `ProtocolError`) are reachable from any non-terminal state —
    /// the spec draws them as exits from the happy path generally, not
    /// from one specific stage, and a protocol violation in particular
    /// (spec §28) can genuinely occur at any point after negotiation.
    pub fn advance(
        self,
        to: ExtensionLifecycle,
    ) -> Result<ExtensionLifecycle, InvalidLifecycleTransition> {
        use ExtensionLifecycle::*;
        let is_failure_exit =
            matches!(to, Rejected | Unsupported | VersionMismatch | ProtocolError);
        let is_terminal = matches!(
            self,
            Closed | Rejected | Unsupported | VersionMismatch | ProtocolError
        );
        let valid = match (self, to) {
            _ if is_terminal => false,
            _ if is_failure_exit => true,
            (Registered, Advertised) => true,
            (Advertised, Negotiated) => true,
            (Negotiated, Opening) => true,
            (Opening, Active) => true,
            (Active, Closing) => true,
            (Closing, Closed) => true,
            _ => false,
        };
        if valid {
            Ok(to)
        } else {
            Err(InvalidLifecycleTransition { from: self, to })
        }
    }
}

/// spec §27, verbatim enum.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, thiserror::Error, serde::Serialize, serde::Deserialize,
)]
pub enum ExtensionError {
    #[error("extension unsupported")]
    Unsupported,
    #[error("extension version mismatch")]
    VersionMismatch,
    #[error("extension capability mismatch")]
    CapabilityMismatch,
    #[error("extension resource limit exceeded")]
    ResourceLimit,
    #[error("extension protocol violation")]
    ProtocolViolation,
    #[error("extension unauthorized")]
    Unauthorized,
    #[error("extension storage failure")]
    StorageFailure,
    #[error("internal extension error")]
    Internal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ExtensionLifecycle::*;

    #[test]
    fn happy_path_advances_in_order() {
        let mut state = Registered;
        for next in [Advertised, Negotiated, Opening, Active, Closing, Closed] {
            state = state.advance(next).unwrap();
        }
        assert_eq!(state, Closed);
    }

    #[test]
    fn cannot_skip_stages() {
        assert!(Registered.advance(Active).is_err());
    }

    #[test]
    fn cannot_leave_a_terminal_state() {
        assert!(Closed.advance(Active).is_err());
        assert!(Rejected.advance(Registered).is_err());
    }

    #[test]
    fn protocol_error_reachable_from_active() {
        assert!(Active.advance(ProtocolError).is_ok());
    }
}
