//! Part 09 — Crash Recovery Architecture.
//!
//! Built the same way as this workspace's other Part-01–08 crates
//! (see [[resilient-mesh]] project memory): one real, deliberately-
//! scoped, tested slice per pass. This first pass covers §6-9 — the
//! four sections in this range concrete enough to implement for real
//! without a live database (this sandbox's rustc floor blocks a real
//! SQLite backend, the same constraint every other crate in this
//! series already documents).
//!
//! ## This pass — file transfer recovery (§18-21)
//!
//! - [`chunk_recovery`] — §18 "File Transfer Recovery", §19 "Chunk
//!   Commit Ordering", §20 "Chunk State Ambiguity":
//!   [`chunk_recovery::ChunkRecoveryState`] (the durable subset of
//!   §19's commit pipeline) with real transition validation, and
//!   [`chunk_recovery::reconcile_chunk_on_restart`]/
//!   [`chunk_recovery::ChunkRecoveryMap`] implementing §20's exact
//!   rule ("treat chunk as unverified, reverify on restart. Do not
//!   assume") as executable code — a genuine complement to
//!   `siar_blob_manifest::resume::ResumeBitmap` (that type's binary
//!   received/missing bitset has no way to represent §20's actual
//!   hazard; see this module's own doc comment for the full
//!   reasoning), not a competing replacement for it.
//! - [`finalization_recovery`] — §21 "Finalization Recovery":
//!   [`finalization_recovery::FinalizationState`] (§21's own
//!   pipeline) with real transition validation, and
//!   [`finalization_recovery::reconcile_finalization`] — §21 names
//!   three crash scenarios in prose; this function is the exhaustive
//!   5-case decision table those three scenarios are examples of,
//!   tested against each named scenario individually rather than
//!   only the cases the spec happened to call out.
//!
//! ## Earlier this pass — messaging recovery (§14-17)
//!
//! - [`messaging_recovery`] — §14 "Messaging Recovery", §15
//!   "Ambiguous Send Result", §16 "Delivery Receipt Recovery", §17
//!   "Inbox Recovery": [`messaging_recovery::Deduplicator`], the one
//!   real mechanism behind three sections that each separately
//!   restate "recipient deduplicates"/"sender applies idempotently"
//!   under different names, built once and reused for all three
//!   rather than three drift-prone hand-rolled `HashSet` checks;
//!   [`messaging_recovery::reload_outbox`] implementing §14's restart
//!   sequence with `OutboundRecoveryAction::RetrySend` structurally
//!   incapable of carrying a freshly-generated id (§14: "do not
//!   create new MessageId"); [`messaging_recovery::apply_receipt`]
//!   for §16's idempotent receipt application; and
//!   [`messaging_recovery::InboundReceiveState`]/
//!   [`messaging_recovery::receive_inbound`] for §17's
//!   validate→persist→commit→ACK sequence and its
//!   crash-before-ACK dedup. Uses this workspace's real
//!   `siar_domain::MessageId` and `siar_event_log::ids::EventId`
//!   rather than local stand-in types, since both concepts §14-17
//!   name already exist as concrete types elsewhere in this
//!   workspace.
//!
//! ## Earlier this pass — §11-13
//!
//! - [`staged_intent`] — §11 "WAL Is Not Enough", §12 "Cross-Store
//!   Atomicity": [`staged_intent::StagedOperationState`] (§12's
//!   three-stage persisted pipeline) with real transition validation,
//!   and [`staged_intent::reconcile`] — §12's own "recovery checks
//!   intermediate state" turned into an actual decision: the full
//!   3×2 truth table of (persisted state) × (what's actually true on
//!   disk/network), including §11's exact named scenario ("database
//!   row says file exists but file rename failed") as one of the
//!   tested cases rather than only the two cases that already agree.
//! - [`operation_state`] — §13 "Durable Operation State":
//!   [`operation_state::FileOperationState`] (§13's own worked
//!   example, verbatim) with real transition validation, and
//!   [`operation_state::is_definitely_complete`] — §13's own explicit
//!   warning ("never rely on absence of error to infer completion")
//!   made into one callable function instead of prose a caller could
//!   still get wrong at each call site: a missing record is always
//!   treated as incomplete, never as implied success.
//!
//! ## Earlier this pass — §6-9
//!
//! - [`shutdown_marker`] — §6 "Clean Shutdown Marker":
//!   [`shutdown_marker::ShutdownMarker`]/[`shutdown_marker::ShutdownMarkerStore`]
//!   (trait + an in-memory stand-in, same seam/stand-in split every
//!   other `*Store` trait in this workspace already uses),
//!   [`shutdown_marker::classify_startup`] (§6's "last start had no
//!   clean close → unclean startup path"), and
//!   [`shutdown_marker::recommended_scope`], built so §6's own
//!   caution — "do not depend on this marker for correctness, only
//!   for deciding how much recovery work to run" — is structural:
//!   both possible scopes still run every mandatory step, the
//!   classification only adds optional extra verification, never
//!   removes required work.
//! - [`idempotent_steps`] — §7 "Crash Recovery Must Be Idempotent":
//!   [`idempotent_steps::run_recovery`], directly implementing §7's
//!   own test case ("run recovery, crash halfway, run again — must
//!   be safe") via a completed-steps ledger checked before each step,
//!   tested by literally simulating a crash mid-pipeline and
//!   confirming a second run resumes rather than re-executing.
//! - [`state_machine`] — §8 "Recovery State Machine":
//!   [`state_machine::RecoveryState`] (verbatim happy-path chain and
//!   failure-branch list) and [`state_machine::RecoveryStateMachine`]
//!   with real transition validation — the failure-branch wiring
//!   (which stage can reach which failure) is this module's own
//!   reasoned mapping, since §8 lists the two sets flatly without
//!   specifying the edges; documented per-edge in that module.
//! - [`transaction_group`] — §9 "Storage Transaction Boundaries":
//!   [`transaction_group::TransactionGroup`], a real in-memory
//!   all-or-nothing operation batch, standing in for an actual
//!   database transaction (unavailable in this sandbox) while still
//!   genuinely testing the all-or-nothing property §9 requires,
//!   modeled directly on §9's own three-part worked example (event
//!   append + projection update + outbox enqueue).
//!
//! ## Deliberately not attempted
//!
//! §10 "WAL" has no code here on purpose — its own text is an
//! explicit instruction *not* to build something ("Do not manually
//! implement log semantics already provided by database engine unless
//! needed"), not a design this crate could implement. There is nothing
//! to build faithfully to that section beyond noting the decision,
//! which this doc comment now does. Everything past §21 — atomic
//! rename/temp-file/orphan cleanup (§22-24), blob reference recovery
//! (§25+), DTN/gateway recovery, and the rest of this 100+ section
//! spec — is not attempted this pass.

pub mod chunk_recovery;
pub mod finalization_recovery;
pub mod idempotent_steps;
pub mod messaging_recovery;
pub mod operation_state;
pub mod shutdown_marker;
pub mod staged_intent;
pub mod state_machine;
pub mod transaction_group;

pub use chunk_recovery::{
    reconcile_chunk_on_restart, ChunkRecoveryAction, ChunkRecoveryMap, ChunkRecoveryState,
    TransferId,
};
pub use finalization_recovery::{
    reconcile_finalization, FinalizationRecoveryAction, FinalizationState,
};
pub use idempotent_steps::{
    run_recovery, RecoveryLedger, RecoveryStep, RecoveryStepError, RecoveryStepId,
};
pub use messaging_recovery::{
    apply_receipt, reload_outbox, Deduplicator, DiscardReason, InboundReceiveOutcome,
    InboundReceiveState, OutboundMessageState, OutboundRecoveryAction, PendingOutboundMessage,
    ReceiptApplyOutcome,
};
pub use operation_state::{is_definitely_complete, FileOperationState};
pub use shutdown_marker::{
    classify_startup, recommended_scope, InMemoryShutdownMarkerStore, RecoveryScope,
    RuntimeGeneration, ShutdownMarker, ShutdownMarkerStore, StartupKind,
};
pub use staged_intent::{reconcile, ReconciliationAction, StagedOperationState};
pub use state_machine::{InvalidTransition, RecoveryState, RecoveryStateMachine};
pub use transaction_group::{TransactionGroup, TransactionStep};
