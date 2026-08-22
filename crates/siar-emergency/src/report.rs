//! The compact emergency report — next.md §46: "A useful SOS can be
//! hundreds of bytes rather than megabytes. Perfect for BLE mesh."
//!
//! [`LocationSharing`]'s variants encode next.md §51's explicit opt-in
//! levels ("No location / Approximate location / Precise one-time
//! location / Live location for X minutes"). This type can't enforce
//! §51's actual policy rule — "Do not broadcast precise GPS
//! automatically simply because Emergency Mode is active" is a UI/call-
//! site discipline, not something a data shape can prevent someone from
//! violating by passing `PreciseOneTime` in — but making [`None`] the
//! `Default` at least means the *easy* thing to construct is the
//! private one, not the reverse.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::kind::EmergencyMessageKind;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum LocationSharing {
    #[default]
    None,
    /// The caller is responsible for having already coarsened the
    /// coordinate (e.g. rounded to a lower precision, or snapped to a
    /// grid cell) before constructing this — this type just carries
    /// whatever `GeoPoint` it's given, it doesn't degrade precision
    /// itself.
    Approximate(GeoPoint),
    PreciseOneTime(GeoPoint),
    /// next.md §52: periodic deltas every 30–120s while live, not a
    /// continuous stream — `expires_at` is an opaque tick in the
    /// caller's own monotonic units (same pattern as
    /// `siar_dtn::bundle::MeshBundle`, next.md §96), not a wall-clock
    /// deadline this type checks itself.
    Live { point: GeoPoint, expires_at: u64 },
}

/// next.md §46's `ShortText` — bounded so a report stays "hundreds of
/// bytes," not accidentally a full essay riding on top of an otherwise
/// tiny BLE-mesh-friendly payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortNote(String);

/// A conservative cap, not a measured one — worth revisiting once
/// there's a real BLE fragment budget to weigh it against (see
/// `siar_transport_ble::fragment::DEFAULT_MAX_FRAGMENT_PAYLOAD_BYTES`),
/// same "starting point, not a permanent constant" framing
/// `siar_domain::attachment::MAX_ATTACHMENT_BYTES` already uses for
/// itself.
pub const MAX_SHORT_NOTE_BYTES: usize = 280;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ShortNoteError {
    #[error("note is {0} bytes, exceeds the {MAX_SHORT_NOTE_BYTES}-byte limit")]
    TooLong(usize),
}

impl ShortNote {
    pub fn parse(text: String) -> Result<Self, ShortNoteError> {
        if text.len() > MAX_SHORT_NOTE_BYTES {
            return Err(ShortNoteError::TooLong(text.len()));
        }
        Ok(Self(text))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmergencyReport {
    pub kind: EmergencyMessageKind,
    pub sender: siar_domain::AccountId,
    #[serde(default)]
    pub location: LocationSharing,
    pub created_at: u64,
    pub people: Option<u16>,
    pub note: Option<ShortNote>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_sharing_defaults_to_none() {
        assert_eq!(LocationSharing::default(), LocationSharing::None);
    }

    #[test]
    fn short_note_accepts_reasonable_length() {
        assert!(ShortNote::parse("trapped on 3rd floor, two people".to_string()).is_ok());
    }

    #[test]
    fn short_note_rejects_too_long() {
        let text = "x".repeat(MAX_SHORT_NOTE_BYTES + 1);
        let err = ShortNote::parse(text).unwrap_err();
        assert_eq!(err, ShortNoteError::TooLong(MAX_SHORT_NOTE_BYTES + 1));
    }
}
