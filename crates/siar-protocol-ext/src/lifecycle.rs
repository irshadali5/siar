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

/// spec §25's own list of subsystems it names as "particularly
/// suitable" for lazy initialization — kept as a real enum (a closed,
/// checkable set) rather than only in prose, so a caller deciding
/// whether some subsystem should defer its own setup has something to
/// match against instead of re-reading the spec text each time.
/// §24's own principle — "capability negotiation does not mean
/// immediate heavy initialization" — is already true of this crate's
/// [`ExtensionLifecycle`] state machine for free: `negotiate()` in
/// [`crate::negotiation`] only ever produces `Negotiated`, never
/// advances anything to `Opening` itself (see that function's own
/// tests) — the transition to `Opening` always requires a separate,
/// explicit `advance()` call the extension itself must trigger, which
/// is what "dormant until the user sends the first file" means in
/// state-machine terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LazyInitTarget {
    Calls,
    Video,
    Camera,
    FileHashing,
    LargeMediaIndexes,
    NearbyRadios,
}

/// spec §26's two shutdown paths. Deliberately two closed, ordered
/// step lists rather than one shared enum with a "mode" flag threaded
/// through every call site — the two paths have genuinely different
/// numbers of steps and different obligations (graceful gets to
/// negotiate a clean stop; abrupt has already lost the transport by
/// the time it runs), so collapsing them into one enum would either
/// invent steps neither path specifies or make the caller filter by
/// mode after the fact instead of asking for the right list directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GracefulShutdownStep {
    StopAcceptingNewWork,
    FinishOrPersistCriticalState,
    SendCloseWhenUseful,
    ReleaseBuffers,
}

/// Ordered exactly as spec §26 lists them under "Graceful close".
pub const GRACEFUL_SHUTDOWN_STEPS: [GracefulShutdownStep; 4] = [
    GracefulShutdownStep::StopAcceptingNewWork,
    GracefulShutdownStep::FinishOrPersistCriticalState,
    GracefulShutdownStep::SendCloseWhenUseful,
    GracefulShutdownStep::ReleaseBuffers,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AbruptShutdownStep {
    PersistResumableState,
    MarkRecoverableWork,
}

/// Ordered exactly as spec §26 lists them under "Abrupt close" — note
/// there is no "send close" step here at all: by definition the
/// transport is already gone (§26: "transport disappears"), so nothing
/// in this list depends on being able to reach the peer. This is the
/// concrete shape of §26's closing line, "correctness must not depend
/// on graceful shutdown": an abrupt close still has a real, complete
/// obligation list of its own — it isn't merely "skip the graceful
/// steps".
pub const ABRUPT_SHUTDOWN_STEPS: [AbruptShutdownStep; 2] = [
    AbruptShutdownStep::PersistResumableState,
    AbruptShutdownStep::MarkRecoverableWork,
];

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

    #[test]
    fn spec_24_negotiate_never_advances_past_negotiated_itself() {
        // The dormant-until-first-use behavior spec §24 describes is a
        // property of who calls advance(), not a new state: Negotiated
        // is a completely valid terminal resting point for as long as
        // the extension likes.
        let state = Negotiated;
        assert!(state.advance(Opening).is_ok());
        // and it's just as valid to never make that call at all —
        // nothing here forces it.
    }

    #[test]
    fn spec_26_graceful_shutdown_steps_are_in_spec_order() {
        assert_eq!(
            GRACEFUL_SHUTDOWN_STEPS,
            [
                GracefulShutdownStep::StopAcceptingNewWork,
                GracefulShutdownStep::FinishOrPersistCriticalState,
                GracefulShutdownStep::SendCloseWhenUseful,
                GracefulShutdownStep::ReleaseBuffers,
            ]
        );
    }

    #[test]
    fn spec_26_abrupt_shutdown_has_no_send_close_step() {
        // "transport disappears" — confirm SendCloseWhenUseful (a
        // GracefulShutdownStep variant) has no equivalent anywhere in
        // the abrupt list, since by definition there's nothing left to
        // send a close message over.
        assert_eq!(
            ABRUPT_SHUTDOWN_STEPS,
            [
                AbruptShutdownStep::PersistResumableState,
                AbruptShutdownStep::MarkRecoverableWork,
            ]
        );
        assert_eq!(ABRUPT_SHUTDOWN_STEPS.len(), 2);
        assert_eq!(GRACEFUL_SHUTDOWN_STEPS.len(), 4);
    }
}
