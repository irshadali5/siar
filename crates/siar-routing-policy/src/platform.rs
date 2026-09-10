//! §85 "Battery-Aware Inputs", §87 "Platform Policy Integration".
//!
//! §87 lists six things a platform adapter exposes: "network type,
//! metered, roaming, power saver, background restrictions, radio
//! availability." Four of those six already have a home elsewhere in
//! this crate rather than here: "network type" is
//! [`crate::types::TransportKind`] itself; "metered"/"roaming" are
//! [`crate::types::MeteredState`]/[`crate::types::RoamingState`]
//! (round 6); "radio availability" is
//! [`crate::acquisition::CandidateState`] (this round, see that
//! module). "Power saver" and "background restrictions" are the two
//! that had nowhere to live until now — [`DeviceState::battery_saver`]
//! here, and [`crate::candidate::PathCandidate::capabilities`]'s new
//! `requires_foreground` flag over in [`crate::acquisition`]. §87's own
//! closing line — "Rust routing policy remains authoritative" — is
//! exactly why none of these six are modeled as a single opaque
//! platform blob: each is its own typed field this crate's own logic
//! can reason about, not a black box the platform adapter interprets
//! on this crate's behalf.
//!
//! §85 itself is deliberately thin: "Part 13 will deepen battery
//! scheduling" is the spec's own admission that this section is a
//! placeholder for representation, not a request for this round to
//! invent full battery-scheduling policy. [`DeviceState`] is that
//! representation; [`crate::discovery::discovery_permitted`] is the
//! one real behavior wired to it this round — enough to make it true
//! that supplying this context actually changes a routing outcome,
//! not inert data collected for its own sake.

use serde::{Deserialize, Serialize};

/// A small ordered scale, deliberately coarse — same "no need for fake
/// precision" reasoning as [`crate::metrics::EnergyCost`]'s own doc
/// comment (§84, which names that exact phrase). A platform adapter
/// reports a percentage; this crate has no principled use for the
/// exact number, only for which coarse band it falls in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BatteryLevelClass {
    Critical,
    Low,
    Medium,
    High,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalState {
    Normal,
    Elevated,
    Critical,
}

/// §85's own list, one field each, every field optional — a platform
/// adapter that only reports some of these (or none yet, at startup)
/// shouldn't be forced to fabricate the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DeviceState {
    pub battery_level_class: Option<BatteryLevelClass>,
    pub charging: Option<bool>,
    pub battery_saver: Option<bool>,
    pub thermal_state: Option<ThermalState>,
    pub foreground: Option<bool>,
}
