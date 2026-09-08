//! §57 "Delivery Semantics", §58 "Operation Descriptor", §59 "Content
//! Class".
//!
//! §57 is a labeling requirement, not a type on its own — its own
//! four worked examples (typing/message/file chunk/call frame) are
//! each already, or now, a [`crate::requirements::DeliveryRequirements`]
//! constructor: `interactive_message()` and `realtime_media()` already
//! covered "message" and "call frame" before this round;
//! [`crate::requirements::DeliveryRequirements::typing_indicator`] and
//! [`crate::requirements::DeliveryRequirements::file_chunk`] close the
//! two this round adds, so all four of §57's own named examples now
//! have a real constructor, not just prose.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::requirements::DeliveryRequirements;
use crate::types::Destination;

/// §58's own field — a newtype for the same reason [`crate::types::PathId`]
/// already is one (see that type's own doc comment): an `OperationId`
/// should never be silently comparable to an unrelated `Uuid`-backed
/// id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct OperationId(Uuid);

impl OperationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self::new()
    }
}

/// §58's `estimated_size` field, and §60 "Size-Aware Routing"'s own
/// input — bytes, not bits (see [`crate::metrics::Bitrate`]'s own doc
/// comment for why bandwidth is bits/second; the two units meeting is
/// exactly [`crate::estimate::completion_time_millis`]'s job).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ByteCount(pub u64);

/// §59, named exactly as listed, in the order listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentClass {
    Control,
    Text,
    Metadata,
    Thumbnail,
    Voice,
    Image,
    File,
    RealtimeAudio,
    RealtimeVideo,
    Emergency,
}

/// §58, named exactly as listed. "Routing receives metadata, not
/// application payload plaintext" is a property of what a caller puts
/// in this struct (no ciphertext/plaintext field exists to violate
/// that in the first place), not something this crate enforces at
/// runtime — same posture as [`crate::candidate::TransportEndpoint`]'s
/// own opaque-bytes design one layer down.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationDescriptor {
    pub operation_id: OperationId,
    pub destination: Destination,
    pub requirements: DeliveryRequirements,
    pub estimated_size: ByteCount,
    pub content_class: ContentClass,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_freshly_generated_operation_ids_are_distinct() {
        assert_ne!(OperationId::new(), OperationId::new());
    }
}
