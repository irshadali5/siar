//! §4 "Core Event Envelope"'s ID types, §6 "Local Global Offset", §28
//! "Offline IDs".

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// §28: "collision resistant, offline, portable, stable across
/// retries" — a random UUIDv4 satisfies all four without this crate
/// needing to reach further. §28 also names a time-sortable 128-bit id
/// (e.g. UUIDv7) as something that "can improve database locality" —
/// a real, deferred improvement, not implemented here: it would need
/// this crate to pin a `uuid` feature/version this workspace's root
/// `Cargo.toml` doesn't currently request, a decision left open rather
/// than made unilaterally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct EventId(Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// §4 declares `StreamId([u8; 32])`, and §5 names streams by structured
/// text (`conversation/<id>`, `transfer/<id>`, ...). [`StreamId::from_name`]
/// is what bridges the two: a deterministic blake3 hash of the name
/// string, so two callers naming "the same stream" (e.g.
/// `format!("conversation/{conversation_id}")`) always agree on its
/// `StreamId` without any coordination or a central name registry.
/// `blake3`, not a general-purpose hasher — already a real workspace
/// dependency (`siar-crypto`) for exactly this kind of deterministic
/// content hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StreamId([u8; 32]);

impl StreamId {
    pub fn from_name(name: &str) -> Self {
        Self(*blake3::hash(name.as_bytes()).as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

/// §9: "Never permanently serialize current domain structs. Use
/// explicit schemas ... `MessageQueuedV1`." This is the numeric tag
/// that names one such schema on the wire/in storage — a plain `u32`
/// newtype a caller assigns constants for (e.g. `EventTypeId(1)` for
/// `MessageQueuedV1`), not derived from hashing the type name: a stable
/// identifier deserves an explicit, reviewable assignment, not an
/// opaque hash a caller can't eyeball for collisions across domains
/// (§33-37: messaging/file/identity/DTN/emergency events all share this
/// one numeric namespace).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventTypeId(pub u32);

/// §6: "device-local append offset ... never global truth across
/// devices."
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LocalLogOffset(pub u64);

/// Milliseconds since the Unix epoch — §27's own "wall clocks are
/// useful for display, not universal ordering": this exists for
/// [`crate::envelope::EventEnvelope::created_at`]'s display/diagnostic
/// value, not as an ordering key (that's `stream_version`/
/// [`LocalLogOffset`]'s job).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn now() -> Self {
        let millis = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0);
        Self(millis)
    }
}
