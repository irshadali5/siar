//! Software AV1 for desktop (architecture doc §2). Encoder via `rav1e`,
//! decoder via `libdav1d` (through `dav1d-sys`) — both verified against
//! their actual published source rather than written from memory or
//! bindgen'd blind. See `encoder.rs`/`decoder.rs` for the specifics of
//! what was checked.

pub mod decoder;
pub mod encoder;

pub use decoder::Av1SoftwareDecoder;
pub use encoder::{Av1EncoderSettings, Av1SoftwareEncoder};
