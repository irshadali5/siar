//! BLE envelope fragmentation and wire framing — next.md §24–25.
//!
//! BLE payloads are small (a default, un-negotiated ATT MTU gives only
//! ~20 usable bytes per write; even a negotiated MTU rarely clears a
//! few hundred), so an encrypted envelope that's already been through
//! `siar-crypto` needs splitting into several `BleFragment`s before it
//! can go out over a GATT characteristic write, and reassembling on the
//! other end — see `reassembly.rs` for that half.
//!
//! `checksum` below is corruption detection for a noisy radio link,
//! **not** authentication — next.md §76's layering ("Bluetooth/Wi-Fi/
//! Iroh security becomes additional protection; conversation encryption
//! remains the authoritative protection") means the bytes this crate
//! fragments are already E2EE ciphertext with its own AEAD tag by the
//! time they arrive here. A corrupted fragment failing this checksum is
//! "ask for a retransmit," not a security decision — an attacker
//! forging a checksum-valid fragment gains nothing, since the payload
//! still has to decrypt correctly one layer up.

use thiserror::Error;

/// Fixed-size wire header: `protocol`(1) plus `transfer_id`(4,
/// big-endian) plus `fragment_index`(2, BE) plus `fragment_count`(2,
/// BE) plus `checksum`(2, BE) = 11 bytes, followed by `payload`.
///
/// (Spelled out as "plus" rather than `+` — a leading `+` on a doc-
/// comment continuation line reads to rustdoc's Markdown parser as an
/// unindented list-item continuation, which is exactly what clippy's
/// `doc_lazy_continuation` lint flags.)
const HEADER_LEN: usize = 11;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BleFragment {
    pub protocol: u8,
    pub transfer_id: u32,
    pub fragment_index: u16,
    pub fragment_count: u16,
    pub payload: Vec<u8>,
}

impl BleFragment {
    /// Serializes this fragment for one GATT characteristic write.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_LEN + self.payload.len());
        out.push(self.protocol);
        out.extend_from_slice(&self.transfer_id.to_be_bytes());
        out.extend_from_slice(&self.fragment_index.to_be_bytes());
        out.extend_from_slice(&self.fragment_count.to_be_bytes());
        out.extend_from_slice(&checksum16(&self.payload).to_be_bytes());
        out.extend_from_slice(&self.payload);
        out
    }

    /// Parses one GATT characteristic write's bytes back into a
    /// fragment, rejecting anything too short to hold a header or whose
    /// payload doesn't match its own checksum.
    pub fn decode(bytes: &[u8]) -> Result<Self, BleFragmentError> {
        if bytes.len() < HEADER_LEN {
            return Err(BleFragmentError::TooShort { got: bytes.len(), need: HEADER_LEN });
        }
        let protocol = bytes[0];
        let transfer_id = u32::from_be_bytes(bytes[1..5].try_into().expect("slice is exactly 4 bytes"));
        let fragment_index = u16::from_be_bytes(bytes[5..7].try_into().expect("slice is exactly 2 bytes"));
        let fragment_count = u16::from_be_bytes(bytes[7..9].try_into().expect("slice is exactly 2 bytes"));
        let claimed_checksum = u16::from_be_bytes(bytes[9..11].try_into().expect("slice is exactly 2 bytes"));
        let payload = bytes[HEADER_LEN..].to_vec();

        let actual_checksum = checksum16(&payload);
        if actual_checksum != claimed_checksum {
            return Err(BleFragmentError::ChecksumMismatch { claimed: claimed_checksum, actual: actual_checksum });
        }
        if fragment_index >= fragment_count {
            return Err(BleFragmentError::IndexOutOfRange { fragment_index, fragment_count });
        }

        Ok(Self { protocol, transfer_id, fragment_index, fragment_count, payload })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BleFragmentError {
    #[error("fragment too short: got {got} bytes, need at least {need}")]
    TooShort { got: usize, need: usize },
    #[error("fragment checksum mismatch: header claimed {claimed:#06x}, payload actually checksums to {actual:#06x}")]
    ChecksumMismatch { claimed: u16, actual: u16 },
    #[error("fragment_index {fragment_index} is out of range for fragment_count {fragment_count}")]
    IndexOutOfRange { fragment_index: u16, fragment_count: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FragmentEnvelopeError {
    #[error("cannot fragment an empty envelope")]
    EmptyEnvelope,
    #[error("max_payload_bytes must be at least 1")]
    ZeroMaxPayload,
    #[error("envelope needs {needed} fragments, more than fragment_count (u16) can address")]
    TooManyFragments { needed: usize },
}

/// A sane pure-logic default — comfortably under even an un-negotiated
/// ATT MTU's ~20 usable bytes is too conservative to be useful, so this
/// assumes at least *some* MTU negotiation happened, without asserting
/// a specific value. Real fragment sizing should come from the actual
/// negotiated MTU on the Kotlin/GATT side once that boundary exists —
/// this constant is for tests and callers that don't have a better
/// number yet, not a claim about what any real BLE stack will grant.
pub const DEFAULT_MAX_FRAGMENT_PAYLOAD_BYTES: usize = 180;

/// Splits `envelope` into `BleFragment`s of at most `max_payload_bytes`
/// each, all sharing one `transfer_id` the caller picks (and is
/// responsible for making unique enough per connection — see
/// `reassembly.rs`'s doc comment on that scope).
pub fn fragment_envelope(
    protocol: u8,
    transfer_id: u32,
    envelope: &[u8],
    max_payload_bytes: usize,
) -> Result<Vec<BleFragment>, FragmentEnvelopeError> {
    if envelope.is_empty() {
        return Err(FragmentEnvelopeError::EmptyEnvelope);
    }
    if max_payload_bytes == 0 {
        return Err(FragmentEnvelopeError::ZeroMaxPayload);
    }

    let needed = envelope.len().div_ceil(max_payload_bytes);
    if needed > u16::MAX as usize {
        return Err(FragmentEnvelopeError::TooManyFragments { needed });
    }
    let fragment_count = needed as u16;

    Ok(envelope
        .chunks(max_payload_bytes)
        .enumerate()
        .map(|(index, chunk)| BleFragment {
            protocol,
            transfer_id,
            fragment_index: index as u16,
            fragment_count,
            payload: chunk.to_vec(),
        })
        .collect())
}

/// A basic rolling checksum for corruption detection — see this file's
/// top doc comment for why this is deliberately not a cryptographic
/// checksum. Not a standard algorithm (not CRC-16, not Fletcher-16);
/// simple enough to be confident it's correct without a compiler, and
/// its own unit tests (below) pin down that it actually does detect a
/// single-byte flip, which is all it needs to promise.
fn checksum16(data: &[u8]) -> u16 {
    let mut sum: u16 = 0;
    for &byte in data {
        sum = sum.wrapping_add(byte as u16);
        sum = sum.rotate_left(1);
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_then_decode_round_trips() {
        let fragment = BleFragment { protocol: 7, transfer_id: 42, fragment_index: 1, fragment_count: 3, payload: vec![1, 2, 3, 4, 5] };
        let encoded = fragment.encode();
        let decoded = BleFragment::decode(&encoded).expect("valid fragment should decode");
        assert_eq!(decoded, fragment);
    }

    #[test]
    fn decode_rejects_a_buffer_shorter_than_the_header() {
        let err = BleFragment::decode(&[1, 2, 3]).unwrap_err();
        assert_eq!(err, BleFragmentError::TooShort { got: 3, need: HEADER_LEN });
    }

    #[test]
    fn decode_rejects_a_corrupted_payload() {
        let fragment = BleFragment { protocol: 1, transfer_id: 1, fragment_index: 0, fragment_count: 1, payload: vec![10, 20, 30] };
        let mut encoded = fragment.encode();
        let last = encoded.len() - 1;
        encoded[last] ^= 0xFF; // flip the last payload byte
        let err = BleFragment::decode(&encoded).unwrap_err();
        assert!(matches!(err, BleFragmentError::ChecksumMismatch { .. }));
    }

    #[test]
    fn decode_rejects_fragment_index_at_or_past_fragment_count() {
        let fragment = BleFragment { protocol: 1, transfer_id: 1, fragment_index: 2, fragment_count: 2, payload: vec![] };
        let err = BleFragment::decode(&fragment.encode()).unwrap_err();
        assert_eq!(err, BleFragmentError::IndexOutOfRange { fragment_index: 2, fragment_count: 2 });
    }

    #[test]
    fn fragment_envelope_splits_into_the_expected_chunk_count() {
        let envelope = vec![0u8; 25];
        let fragments = fragment_envelope(1, 99, &envelope, 10).expect("should fragment");
        assert_eq!(fragments.len(), 3); // 10 + 10 + 5
        assert!(fragments.iter().all(|f| f.fragment_count == 3 && f.transfer_id == 99 && f.protocol == 1));
        assert_eq!(fragments[0].fragment_index, 0);
        assert_eq!(fragments[2].fragment_index, 2);
        assert_eq!(fragments[2].payload.len(), 5);
    }

    #[test]
    fn fragment_envelope_rejects_empty_envelope() {
        assert_eq!(fragment_envelope(1, 1, &[], 10).unwrap_err(), FragmentEnvelopeError::EmptyEnvelope);
    }

    #[test]
    fn fragment_envelope_rejects_zero_max_payload() {
        assert_eq!(fragment_envelope(1, 1, &[1], 0).unwrap_err(), FragmentEnvelopeError::ZeroMaxPayload);
    }

    #[test]
    fn checksum_changes_on_single_byte_flip() {
        let original = checksum16(&[1, 2, 3, 4, 5]);
        let flipped = checksum16(&[1, 2, 3, 4, 6]);
        assert_ne!(original, flipped);
    }
}
