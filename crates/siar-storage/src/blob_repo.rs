//! Blob storage (plan.md §18–19: blobs get their own store, not the
//! `messages` table). Content-addressed — `blob_hash` is the primary key
//! precisely because the hash *is* the identity of the content
//! (plan.md §22).

use crate::{blob_codec::{decode_blob, encode_blob}, StorageError};
use stoolap::Database;
use std::sync::Arc;

pub trait BlobRepository {
    /// Idempotent under the same hash — re-publishing an already-stored
    /// blob just bumps `ref_count` rather than erroring or duplicating
    /// storage (plan.md §24's content addressing implies this: the same
    /// ciphertext hash means the same bytes are already there).
    fn put(&self, blob_hash: &[u8; 32], ciphertext: &[u8]) -> Result<(), StorageError>;

    fn get(&self, blob_hash: &[u8; 32]) -> Result<Option<Vec<u8>>, StorageError>;
}

pub struct StoolapBlobRepository {
    db: Arc<Database>,
}

impl StoolapBlobRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl BlobRepository for StoolapBlobRepository {
    fn put(&self, blob_hash: &[u8; 32], ciphertext: &[u8]) -> Result<(), StorageError> {
        let hash_hex = hex_encode(blob_hash);

        self.db.execute("BEGIN", ()).map_err(StorageError::from_stoolap)?;

        let already_exists = {
            let mut rows = self
                .db
                .query("SELECT 1 FROM blobs WHERE blob_hash = $1", (hash_hex.clone(),))
                .map_err(StorageError::from_stoolap)?;
            rows.next().is_some()
        };

        let result = if already_exists {
            self.db.execute(
                "UPDATE blobs SET ref_count = ref_count + 1 WHERE blob_hash = $1",
                (hash_hex,),
            )
        } else {
            self.db.execute(
                "INSERT INTO blobs (blob_hash, ciphertext, ref_count) VALUES ($1, $2, 1)",
                (hash_hex, encode_blob(ciphertext)),
            )
        };

        if let Err(e) = result {
            let _ = self.db.execute("ROLLBACK", ());
            return Err(StorageError::from_stoolap(e));
        }
        self.db.execute("COMMIT", ()).map_err(StorageError::from_stoolap)?;
        Ok(())
    }

    fn get(&self, blob_hash: &[u8; 32]) -> Result<Option<Vec<u8>>, StorageError> {
        let rows = self
            .db
            .query(
                "SELECT ciphertext FROM blobs WHERE blob_hash = $1",
                (hex_encode(blob_hash),),
            )
            .map_err(StorageError::from_stoolap)?;

        for row in rows {
            let row = row.map_err(StorageError::from_stoolap)?;
            let ciphertext_b64: String = row.get(0).map_err(StorageError::from_stoolap)?;
            return Ok(Some(decode_blob(&ciphertext_b64)?));
        }
        Ok(None)
    }
}

fn hex_encode(bytes: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_then_get_round_trips() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapBlobRepository::new(db);
        let hash = [7u8; 32];
        repo.put(&hash, b"file bytes").unwrap();

        let found = repo.get(&hash).unwrap().unwrap();
        assert_eq!(found, b"file bytes");
    }

    #[test]
    fn get_on_unknown_hash_is_none() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapBlobRepository::new(db);
        assert!(repo.get(&[9u8; 32]).unwrap().is_none());
    }

    #[test]
    fn republishing_the_same_hash_does_not_duplicate_storage() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapBlobRepository::new(db);
        let hash = [1u8; 32];
        repo.put(&hash, b"same content").unwrap();
        repo.put(&hash, b"same content").unwrap();

        // Still just the one row's content, retrievable — the ref-count
        // bump is internal bookkeeping, not user-visible duplication.
        assert_eq!(repo.get(&hash).unwrap().unwrap(), b"same content");
    }
}
