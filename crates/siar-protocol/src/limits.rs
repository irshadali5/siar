//! Decode limits (plan.md §61): never decode arbitrary remote sizes.

/// Matches siar-domain's MAX_TEXT_BYTES; kept as an independent constant
/// here (rather than importing it) because the wire limit and the domain
/// validation limit are allowed to diverge later without one silently
/// changing the other.
pub const MAX_TEXT_FRAME_BYTES: usize = 64 * 1024;

pub const MAX_CONTROL_FRAME_BYTES: usize = 256 * 1024;
