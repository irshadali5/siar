//! Store-carry-forward DTN core — next.md §29–39, §68–69. Phase 4 of
//! `next.md`'s roadmap.
//!
//! - [`bundle`]: [`bundle::MeshBundle`] (next.md §29), hop-limit
//!   decrement (§30), and replication-budget consumption (§38).
//! - [`dedup`]: [`dedup::SeenBundles`], bounded so mesh forwarding can't
//!   storm (§31).
//! - [`store`]: [`store::BundleStore`], a bounded, priority-evicting
//!   local store (§68).
//!
//! next.md §33 also names `scheduler.rs`, `forwarding.rs`,
//! `inventory.rs`, `custody.rs`, `expiry.rs`, and `quota.rs` as
//! module-level concerns. Some of those map directly onto what's here
//! (`expiry`/`quota` are `MeshBundle::is_expired`/`BundleStore`;
//! `inventory` is `dedup::SeenBundles` today, though next.md §36's
//! Bloom-filter-based reconciliation between two *different* nodes'
//! inventories is a distinct, not-yet-built piece from "have I seen
//! this bundle before" locally). `scheduler`, `forwarding`, and
//! `custody` genuinely aren't here yet — next.md §35–37's peer-
//! encounter protocol and forwarding-class logic need something to
//! actually talk to a peer through (a transport session, a connection
//! event), which doesn't exist as a pluggable concept in this crate or
//! `siar-transport` yet. Scoped out the same way this workspace has
//! scoped out every other real-network-shaped piece so far — not an
//! oversight, and worth its own pass once there's a connection
//! abstraction to build peer-encounter logic against.

pub mod bundle;
pub mod dedup;
pub mod store;
