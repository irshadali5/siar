//! Same property as `decode_frame.rs`, for the blob-transfer protocol
//! (plan.md §100's "sync frame" / attachment-metadata targets, applied
//! to this workspace's hand-rolled blob protocol — see
//! `siar-transport`'s module docs for why it isn't `iroh-blobs` yet).
//!
//! Run with: cargo fuzz run decode_blob_frame

#![no_main]
use libfuzzer_sys::fuzz_target;
use siar_protocol::{decode_frame_generic, BlobRequest, MAX_BLOB_FRAME_BYTES};

fuzz_target!(|data: &[u8]| {
    let _: Result<(BlobRequest, usize), _> = decode_frame_generic(data, MAX_BLOB_FRAME_BYTES);
});
