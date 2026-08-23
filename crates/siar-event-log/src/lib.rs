#![forbid(unsafe_code)]

//! siar-event-log: a first slice of "Part 04 — Offline Event Log
//! Architecture" (the fourth of the architecture documents supplied so
//! far — Part 01 has `siar-protocol-ext`, Part 02 has
//! `siar-identity-multidevice`, Part 03 has `siar-routing-policy`, all
//! in this same workspace).
//!
//! ## Scope: §92 "Implementation Phases" 1, made real (not just typed)
//!
//! - [`ids`] — §4's ID types plus §6/§28: `EventId` (UUIDv4 — see that
//!   type's own doc comment on why not a time-sortable variant yet),
//!   `StreamId` (§4's `[u8; 32]`, with [`ids::StreamId::from_name`]
//!   bridging §5's structured stream-naming convention via a
//!   deterministic blake3 hash), `CorrelationId`, `EventTypeId`,
//!   `LocalLogOffset`, `Timestamp`.
//! - [`envelope`] — §4 `EventEnvelope`, §7 `EventOrigin` (reusing
//!   `siar_domain::DeviceId`, a real dependency, not a parallel id).
//! - [`store`] — §20 `EventStore` trait, verbatim signatures (via
//!   `async-trait`, already a real workspace dependency for exactly
//!   this — see that module's own doc comment), plus §21's batch
//!   `AppendRequest`/`AppendResult` and §12's `expected_version`
//!   optimistic-concurrency guard.
//! - [`memory_store`] — [`memory_store::InMemoryEventStore`], a real,
//!   fully-tested implementation of the trait above: §11 atomic batch
//!   append (all-or-nothing under one lock), §12 concurrency-conflict
//!   rejection, §24 idempotent no-op on a repeated `EventId`. This is
//!   NOT §19's recommended SQLite backend — that's §92's own **Phase
//!   2**, a separate, larger piece of work (a real `siar-storage`
//!   integration) not attempted here, same relationship
//!   `siar-identity-multidevice::TrustedAccountStore`'s in-memory
//!   `HashMap` has to real durable storage for that crate.
//! - [`gap`] — §26 gap detection as a pure function, for the remote
//!   ingestion path (§23) to call.
//!
//! Every module above is covered by tests exercising real behavior —
//! actual concurrent-writer rejection, actual duplicate-event
//! deduplication, actual gap detection on the spec's own worked
//! example — not just type shapes.
//!
//! ## What's explicitly NOT here
//!
//! Nearly everything past Phase 1: §9's versioned-schema
//! upcasting machinery, §13/§14's local-first command flow and
//! transactional outbox (application-level patterns this crate's
//! trait *enables* but doesn't itself implement), §16-18 projections/
//! checkpoints/read-your-writes, §19 the actual SQLite backend, §22
//! integrity/hash-chaining, §23 the full remote-ingestion pipeline
//! (protocol/identity/authorization validation — this crate has no
//! dependency on `siar-identity-multidevice` or `siar-protocol-ext` for
//! that reason; only [`gap::detect_gap`] serves that path), §25's
//! hold-for-dependency out-of-order handling (`detect_gap` reports a
//! gap; it doesn't hold or reorder anything), §27 hybrid logical clocks
//! beyond what `stream_version` already provides, §29-32 pure decision
//! functions/effect processing conventions (a pattern this crate's
//! trait supports but doesn't enforce or provide a type for), §33-37
//! domain-specific event catalogs (messaging/file/identity/DTN/
//! emergency — those belong in the crates that own those domains,
//! defining their own `EventTypeId` constants and payload schemas
//! against this trait, not in this crate), §38-95 snapshotting,
//! compaction, privacy/deletion, replication/sync, storage limits,
//! multi-tenant isolation, platform boundaries, and all listed test/
//! ops sections. A genuinely small slice of a 95-section document.

pub mod envelope;
pub mod gap;
pub mod ids;
pub mod memory_store;
pub mod store;

pub use envelope::{EventEnvelope, EventOrigin};
pub use gap::{detect_gap, StreamGap};
pub use ids::{CorrelationId, EventId, EventTypeId, LocalLogOffset, StreamId, Timestamp};
pub use memory_store::InMemoryEventStore;
pub use store::{AppendRequest, AppendResult, EventStore, EventStoreError, NewEvent, StoredEvent};
