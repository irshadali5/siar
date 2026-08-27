//! §18 "Framing": "Never trust remote length fields." §18's own
//! four-step sequence — read bounded header, validate length, check
//! extension limit, allocate/read safely — implemented as real,
//! separate steps below, not collapsed into one "just deserialize it"
//! call.

use crate::descriptor::{ExtensionLimits, SessionLocalExtensionId};

/// §18's own conceptual struct, field-for-field. Fixed 8-byte
/// encoding, hand-rolled rather than via `postcard`/serde — the entire
/// point of a frame header is to be parseable *before* trusting
/// anything about the payload enough to hand it to a deserializer, so
/// this crate parses it with explicit byte slicing and checked
/// arithmetic instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    pub frame_length: u32,
    pub extension_session_id: SessionLocalExtensionId,
    pub frame_type: u8,
    pub flags: u8,
}

/// The header's own fixed wire size: 4 (`frame_length`) + 2
/// (`extension_session_id`) + 1 (`frame_type`) + 1 (`flags`) bytes.
pub const FRAME_HEADER_BYTES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FramingError {
    #[error("frame header needs {FRAME_HEADER_BYTES} bytes, got {actual}")]
    HeaderTooShort { actual: usize },
    #[error("frame_length {declared} exceeds this extension's max_frame_size {max}")]
    FrameTooLarge { declared: u32, max: usize },
    #[error("frame_length {declared} is smaller than the header itself ({FRAME_HEADER_BYTES} bytes) — malformed")]
    FrameLengthImpossiblySmall { declared: u32 },
}

/// §18 step 1: "read bounded header" — always reads exactly
/// [`FRAME_HEADER_BYTES`], regardless of what the input claims its own
/// length is, so a hostile or corrupt peer can never make this step
/// itself allocate or read more than that fixed amount.
pub fn parse_frame_header(bytes: &[u8]) -> Result<FrameHeader, FramingError> {
    if bytes.len() < FRAME_HEADER_BYTES {
        return Err(FramingError::HeaderTooShort {
            actual: bytes.len(),
        });
    }
    let frame_length = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let extension_session_id = SessionLocalExtensionId(u16::from_be_bytes([bytes[4], bytes[5]]));
    let frame_type = bytes[6];
    let flags = bytes[7];

    if (frame_length as usize) < FRAME_HEADER_BYTES {
        return Err(FramingError::FrameLengthImpossiblySmall {
            declared: frame_length,
        });
    }

    Ok(FrameHeader {
        frame_length,
        extension_session_id,
        frame_type,
        flags,
    })
}

/// §18 steps 2-3: "validate length" + "check extension limit" — run
/// against the already-parsed header, before step 4 (allocation) ever
/// happens. Kept as its own function, separate from
/// [`parse_frame_header`], so a caller can parse a header and consult
/// a routing table (to find which extension's [`ExtensionLimits`]
/// apply, keyed by [`FrameHeader::extension_session_id`]) before
/// deciding whether the frame is even allowed to proceed — exactly the
/// ordering §18's own four-step list describes.
pub fn validate_frame_length(
    header: &FrameHeader,
    limits: &ExtensionLimits,
) -> Result<(), FramingError> {
    if header.frame_length as usize > limits.max_frame_size {
        return Err(FramingError::FrameTooLarge {
            declared: header.frame_length,
            max: limits.max_frame_size,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> ExtensionLimits {
        ExtensionLimits {
            max_frame_size: 1024,
            max_in_flight_frames: 16,
            max_concurrent_streams: 4,
            max_buffered_bytes: 65536,
        }
    }

    fn header_bytes(frame_length: u32, session_id: u16, frame_type: u8, flags: u8) -> Vec<u8> {
        let mut bytes = frame_length.to_be_bytes().to_vec();
        bytes.extend_from_slice(&session_id.to_be_bytes());
        bytes.push(frame_type);
        bytes.push(flags);
        bytes
    }

    #[test]
    fn a_well_formed_header_parses_correctly() {
        let bytes = header_bytes(100, 7, 1, 0);
        let header = parse_frame_header(&bytes).unwrap();
        assert_eq!(header.frame_length, 100);
        assert_eq!(header.extension_session_id, SessionLocalExtensionId(7));
        assert_eq!(header.frame_type, 1);
    }

    #[test]
    fn a_truncated_buffer_never_reads_past_what_exists() {
        let short = vec![0u8; 4];
        assert_eq!(
            parse_frame_header(&short),
            Err(FramingError::HeaderTooShort { actual: 4 })
        );
    }

    #[test]
    fn a_frame_length_smaller_than_the_header_itself_is_rejected() {
        let bytes = header_bytes(3, 1, 0, 0); // declares 3 bytes total — impossible, header alone is 8
        assert_eq!(
            parse_frame_header(&bytes),
            Err(FramingError::FrameLengthImpossiblySmall { declared: 3 })
        );
    }

    #[test]
    fn a_hostile_oversized_length_is_rejected_before_any_allocation() {
        let bytes = header_bytes(10_000_000, 1, 0, 0);
        let header = parse_frame_header(&bytes).unwrap(); // parsing the header itself always succeeds — it's fixed-size
        let result = validate_frame_length(&header, &limits());
        assert_eq!(
            result,
            Err(FramingError::FrameTooLarge {
                declared: 10_000_000,
                max: 1024
            })
        );
    }

    #[test]
    fn a_frame_within_limits_validates() {
        let bytes = header_bytes(500, 1, 0, 0);
        let header = parse_frame_header(&bytes).unwrap();
        assert!(validate_frame_length(&header, &limits()).is_ok());
    }
}
