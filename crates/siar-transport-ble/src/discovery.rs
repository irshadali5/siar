//! Discovery beacon wire format — next.md §100, with the rotating-token
//! privacy property from §72.
//!
//! `ephemeral_node_id` is deliberately just `[u8; 16]` here, not derived
//! by this crate — next.md §72's "rotating discovery token derived from
//! a short-lived epoch secret" is a `siar-crypto` job (HKDF or similar
//! over a rotating epoch secret), not a BLE-framing one. This module
//! only carries whatever opaque bytes the caller derived; it has no
//! opinion on how they were produced, matching the layering
//! `siar-transport-ble`'s `Cargo.toml` doc comment already establishes
//! (transport crates carry bytes, crypto crates derive them).
//!
//! Byte budget, flagged rather than assumed solved: a legacy BLE
//! advertisement's payload is capped around 31 bytes total *including*
//! whatever AD-structure overhead Android's `AdvertiseData` API adds
//! (flags, any service UUID, length/type prefixes per field) — this
//! struct's own 23-byte encoding is what's left once that overhead is
//! subtracted, which is a real device/Android-API question this pure
//! crate can't verify. If it doesn't fit once real `AdvertiseData`
//! framing is measured, `capability_bits` (4 bytes) is the first thing
//! worth shrinking to a `u8`/`u16`, not `ephemeral_node_id` — a shorter
//! rotating ID weakens the anti-tracking property §72 exists for.

use thiserror::Error;

/// `protocol_version`(1) + `ephemeral_node_id`(16) + `capability_bits`(4,
/// big-endian) + `epoch`(2, BE) = 23 bytes.
const ENCODED_LEN: usize = 23;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoveryBeacon {
    pub protocol_version: u8,
    pub ephemeral_node_id: [u8; 16],
    pub capability_bits: u32,
    pub epoch: u16,
}

/// next.md §100's capability list. Bit assignments are this crate's own
/// choice (the doc doesn't pin specific bit positions) — stable once
/// shipped, since a beacon decoded with the wrong bit meaning is a
/// silent misread, not a decode error.
pub mod capability_bits {
    pub const BLE_TRANSPORT: u32 = 1 << 0;
    pub const WIFI_DIRECT: u32 = 1 << 1;
    pub const WIFI_AWARE: u32 = 1 << 2;
    pub const INTERNET_GATEWAY: u32 = 1 << 3;
    pub const DTN_RELAY: u32 = 1 << 4;
    pub const EMERGENCY_MODE: u32 = 1 << 5;
}

impl DiscoveryBeacon {
    pub fn encode(&self) -> [u8; ENCODED_LEN] {
        let mut out = [0u8; ENCODED_LEN];
        out[0] = self.protocol_version;
        out[1..17].copy_from_slice(&self.ephemeral_node_id);
        out[17..21].copy_from_slice(&self.capability_bits.to_be_bytes());
        out[21..23].copy_from_slice(&self.epoch.to_be_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DiscoveryBeaconError> {
        if bytes.len() != ENCODED_LEN {
            return Err(DiscoveryBeaconError::WrongLength { got: bytes.len(), need: ENCODED_LEN });
        }
        let mut ephemeral_node_id = [0u8; 16];
        ephemeral_node_id.copy_from_slice(&bytes[1..17]);
        Ok(Self {
            protocol_version: bytes[0],
            ephemeral_node_id,
            capability_bits: u32::from_be_bytes(bytes[17..21].try_into().expect("slice is exactly 4 bytes")),
            epoch: u16::from_be_bytes(bytes[21..23].try_into().expect("slice is exactly 2 bytes")),
        })
    }

    pub fn has_capability(&self, bit: u32) -> bool {
        self.capability_bits & bit != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DiscoveryBeaconError {
    #[error("discovery beacon must be exactly {need} bytes, got {got}")]
    WrongLength { got: usize, need: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_then_decode_round_trips() {
        let beacon = DiscoveryBeacon {
            protocol_version: 1,
            ephemeral_node_id: [7u8; 16],
            capability_bits: capability_bits::BLE_TRANSPORT | capability_bits::DTN_RELAY,
            epoch: 42,
        };
        let decoded = DiscoveryBeacon::decode(&beacon.encode()).expect("valid beacon should decode");
        assert_eq!(decoded, beacon);
    }

    #[test]
    fn decode_rejects_wrong_length() {
        let err = DiscoveryBeacon::decode(&[0u8; 10]).unwrap_err();
        assert_eq!(err, DiscoveryBeaconError::WrongLength { got: 10, need: ENCODED_LEN });
    }

    #[test]
    fn has_capability_reads_the_right_bit() {
        let beacon = DiscoveryBeacon {
            protocol_version: 1,
            ephemeral_node_id: [0u8; 16],
            capability_bits: capability_bits::WIFI_AWARE,
            epoch: 0,
        };
        assert!(beacon.has_capability(capability_bits::WIFI_AWARE));
        assert!(!beacon.has_capability(capability_bits::BLE_TRANSPORT));
    }
}
