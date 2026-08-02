//! `siar-core` — everything about this app that has nothing to do
//! with how it's drawn on screen: identity, networking (iroh transport,
//! gossip rooms, contact requests, conversation docs), wire protocol,
//! local storage, media codecs, ringtones, and shareable tickets.
//!
//! No `dioxus` dependency lives here on purpose. `siar-ui` (the
//! Dioxus component tree) depends on this crate; this crate never
//! depends back on it. That's what makes the same UI crate buildable
//! against both the desktop and Android platform crates without any
//! `#[cfg(feature = "ui")]` split inside the business logic itself.
//!
//! Platform-specific *pieces* of the business logic (voice/video call
//! codecs in `net::calls`, the Android ringtone path in `ringtone`) stay
//! `#[cfg(target_os = ...)]`-gated inside their existing modules exactly
//! as they were before this split — moving crates doesn't change which
//! target a given code path compiles for, only which Cargo.toml declares
//! the dependency that path needs. See each crate's Cargo.toml for the
//! target-gated dependency blocks that make that continue to work.

pub mod app;
pub mod backup;
pub mod config;
pub mod gossip;
pub mod identity;
pub mod media;
pub mod net;
pub mod protocol;
pub mod ringtone;
pub mod store;
pub mod ticket;

pub use config::{Config, CONFIG};
