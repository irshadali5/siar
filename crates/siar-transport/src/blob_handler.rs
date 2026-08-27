//! Blob-transfer protocol handler (see `lib.rs` and `crate::blob` for why
//! this exists instead of `iroh-blobs`).
//!
//! `BlobStore` is defined here, not borrowed from `siar-storage`,
//! specifically so this crate doesn't depend on `siar-storage`
//! (plan.md §86's dependency direction: `transport` sits below
//! `storage` in the stack, so it can't import it — `siar-messaging`
//! is what implements this trait over `siar_storage::BlobRepository`
//! and hands the `Arc<dyn BlobStore>` down to `SiarEndpoint::bind`).

use iroh::{
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
};
use siar_protocol::{
    decode_frame_generic, encode_frame_generic, BlobRequest, BlobResponse, MAX_BLOB_FRAME_BYTES,
};
use std::sync::Arc;

pub const BLOB_ALPN: &[u8] = b"messenger/blob/1";

pub trait BlobStore: Send + Sync {
    fn get(&self, blob_hash: &[u8; 32]) -> Option<Vec<u8>>;
}

#[derive(Clone)]
pub struct BlobProtocolHandler {
    store: Arc<dyn BlobStore>,
}

impl std::fmt::Debug for BlobProtocolHandler {
    // `iroh::protocol::ProtocolHandler` requires `Debug` (it's used in
    // the router's own logging), but `BlobStore` implementors aren't
    // required to be `Debug` themselves — adding that bound would leak
    // out to every `BlobStore` impl (`siar-messaging::StorageBlobStore`
    // included) for a trait that only exists to be handed to iroh.
    // Hand-writing this instead of deriving keeps that bound local to
    // here.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobProtocolHandler")
            .finish_non_exhaustive()
    }
}

impl BlobProtocolHandler {
    pub fn new(store: Arc<dyn BlobStore>) -> Self {
        Self { store }
    }
}

impl ProtocolHandler for BlobProtocolHandler {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let (mut send, mut recv) = connection
            .accept_bi()
            .await
            .map_err(AcceptError::from_err)?;

        let bytes = recv
            .read_to_end(MAX_BLOB_FRAME_BYTES)
            .await
            .map_err(AcceptError::from_err)?;
        let (request, _consumed): (BlobRequest, usize) =
            decode_frame_generic(&bytes, MAX_BLOB_FRAME_BYTES).map_err(AcceptError::from_err)?;

        let response = match self.store.get(&request.blob_hash) {
            Some(ciphertext) => BlobResponse::Found { ciphertext },
            None => BlobResponse::NotFound,
        };

        let mut framed = Vec::new();
        encode_frame_generic(&response, MAX_BLOB_FRAME_BYTES, &mut framed)
            .map_err(AcceptError::from_err)?;
        send.write_all(&framed)
            .await
            .map_err(AcceptError::from_err)?;
        send.finish().map_err(AcceptError::from_err)?;

        connection.closed().await;
        Ok(())
    }
}
