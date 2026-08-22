//! stoolap 0.4's `Value` enum has no blob variant (only `Integer`,
//! `Float`, `Text`, `Boolean`, `Timestamp`, and an internal `Extension`
//! used for JSON/vectors) — confirmed by reading stoolap's own
//! `core/value.rs` and `api/params.rs`: there is no `ToParam for Vec<u8>`
//! or `FromValue for Vec<u8>`, and a `BLOB`/`BINARY`/`VARBINARY` column
//! declaration is parsed but stored as `DataType::Text` under the hood
//! (`executor/ddl.rs`). So every ciphertext byte string this crate
//! stores — message payloads, blob ciphertext — goes in as base64 text
//! through `ToParam for String`, and comes back out through
//! `FromValue for String`, decoded here.

use crate::StorageError;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

pub fn encode_blob(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

pub fn decode_blob(text: &str) -> Result<Vec<u8>, StorageError> {
    STANDARD
        .decode(text)
        .map_err(|e| StorageError::MalformedId(format!("payload was not valid base64: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_arbitrary_bytes() {
        let bytes = vec![0u8, 1, 2, 255, 254, 253, 128, 127];
        let encoded = encode_blob(&bytes);
        assert_eq!(decode_blob(&encoded).unwrap(), bytes);
    }

    #[test]
    fn round_trips_empty() {
        assert_eq!(decode_blob(&encode_blob(&[])).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn rejects_malformed_base64() {
        assert!(decode_blob("not valid base64!!").is_err());
    }
}
