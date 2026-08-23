//! §4 "Core Event Envelope", §7 "Event Origin".

use serde::{Deserialize, Serialize};
use siar_domain::DeviceId;

use crate::ids::{CorrelationId, EventId, EventTypeId, StreamId, Timestamp};

/// §7. Reuses `siar_domain::DeviceId` rather than inventing a
/// parallel device identifier — this crate already depends on
/// `siar-domain` for exactly this reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventOrigin {
    LocalDevice(DeviceId),
    RemoteDevice(DeviceId),
    Imported,
    Recovery,
    System,
}

/// §4, field-for-field. `payload: Vec<u8>`, not the spec's own
/// `Bytes` — this crate has no existing dependency on the `bytes`
/// crate, and `Vec<u8>` round-trips through postcard identically; the
/// spec's choice is almost certainly about avoiding a copy on an
/// existing `bytes::Bytes` buffer inside a real storage/networking
/// layer, which is exactly the kind of backend-specific optimization
/// this crate's own top doc comment says it stays out of.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: EventId,
    pub stream_id: StreamId,
    pub stream_version: u64,
    pub event_type: EventTypeId,
    pub schema_version: u16,
    pub created_at: Timestamp,
    pub origin: EventOrigin,
    pub correlation_id: Option<CorrelationId>,
    pub causation_id: Option<EventId>,
    pub payload: Vec<u8>,
}
