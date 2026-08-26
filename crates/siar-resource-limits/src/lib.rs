//! Part 08 — Resource Limits & Backpressure Architecture.
//!
//! Built the same way as this workspace's other Part-01–07 crates
//! (see [[resilient-mesh]] project memory): one real, deliberately-
//! scoped, tested slice per pass, tracked honestly against the spec's
//! own section numbers.
//!
//! ## This pass
//!
//! - [`types`] — §3 "Resource Dimensions", §4 "Resource Classes", §8
//!   "Resource Budget": [`types::ResourceKind`] (all ten dimensions
//!   §3 lists, so pressure is never collapsed into one generic "busy"
//!   flag), [`types::ResourceBudget`] (the concrete-quantity subset of
//!   those dimensions §8 gives a number shape for)
//! - [`token_bucket`] — §52 "Traffic Shaping", §53 "Token Bucket":
//!   [`token_bucket::TokenBucket`], a real (not conceptual-sketch)
//!   rate limiter — exact integer refill arithmetic (no `f64` drift),
//!   all-or-nothing consumption, and a refill clock that keeps
//!   advancing even on a rejected request. Usable directly for any of
//!   §53's three named purposes (per-peer bytes/sec, per-extension
//!   bandwidth, unknown-peer intake)
//!
//! ## Deliberately not attempted this pass
//!
//! Everything past these two modules: resource policy layering
//! (§5-7), the global/sub-budget hierarchy and borrowing (§9-11),
//! device resource profiles (§12-16), bounded queues/priority
//! queues/backpressure semantics (§17-27), admission control and
//! hierarchical accounting (§24-32), CPU/memory/connection/storage
//! budgets (§33-50), bandwidth fairness and weighted fair queueing
//! (§54-56), emergency preemption and drop policies (§57-61),
//! per-subsystem backpressure (messaging/files/DTN/routing/discovery/
//! capability/event-log/outbox/IPC/FFI/plugins, §62-77), and
//! cost-estimate admission (§78 onward). None of these are guessed
//! at here.

pub mod token_bucket;
pub mod types;

pub use token_bucket::TokenBucket;
pub use types::{ResourceBudget, ResourceKind};
