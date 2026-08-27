//! Codec-implementation-agnostic media layer (architecture doc §1):
//! codec identity, capability description, negotiation, quality
//! ladders, and the `VideoEncoder`/`VideoDecoder`/`AudioEncoder`/
//! `AudioDecoder` traits that a real codec crate (`siar-media-av1`,
//! `siar-media-audio`, a future `siar-media-android`) implements.
//!
//! Nothing in this crate performs an encode or a decode. That's
//! deliberate — see §1's own framing: "the calls crate should know
//! about VideoEncoder/VideoDecoder/VideoCapabilities but not whether
//! the implementation is AV1 software or Android MediaCodec." This
//! crate IS that boundary, which is also why it has zero non-stdlib
//! dependencies: every function in it is pure, and every type is
//! trivially testable without a codec, a device, or a compiler beyond
//! what already builds the rest of this workspace.

pub mod capability;
pub mod codec;
pub mod error;
pub mod frame;
pub mod negotiation;
pub mod quirk;
pub mod resolution;

pub use capability::{
    BitrateRange, CodecProfile, FrameRateRange, Resolution, VideoCapabilities, VideoCodecCapability,
};
pub use codec::{AudioCodec, CodecImplementation, VideoCodec};
pub use error::{
    AudioDecoder, AudioEncoder, DecodeError, EncodeError, EncodedAudioFrame, VideoDecoder,
    VideoEncoder,
};
pub use frame::{DecodedVideoFrame, EncodedVideoFrame, RawVideoFrame};
pub use negotiation::{negotiate_call, negotiate_direction, NegotiatedCall, NegotiatedDirection};
pub use quirk::{matching_quirks, CodecQuirk, CodecWorkaround};
pub use resolution::{
    rung_above, rung_below, QualityRung, ANDROID_HARDWARE_LADDER, AV1_SOFTWARE_LADDER,
};
