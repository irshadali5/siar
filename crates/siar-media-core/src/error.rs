use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// The input frame's plane sizes didn't match its declared
    /// resolution (`RawVideoFrame::is_well_formed`) — callers should
    /// check this before handing frames to an encoder, but implementors
    /// re-check because architecture doc §68 (plan.md, same principle):
    /// treat all input as hostile, even your own capture pipeline.
    MalformedFrame,
    /// The underlying codec implementation rejected the frame or
    /// config for a reason specific to it (bitstream limits, invalid
    /// speed setting, etc). The message is implementation-specific and
    /// not meant to be pattern-matched on.
    Backend(String),
    /// This implementation doesn't support what was asked (e.g. asking
    /// the AV1 software encoder for a resolution above its configured
    /// ceiling) — distinct from `Backend` so callers can tell "try a
    /// smaller resolution" apart from "this encoder is broken."
    Unsupported(String),
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::MalformedFrame => {
                write!(f, "frame plane sizes did not match its resolution")
            }
            EncodeError::Backend(msg) => write!(f, "encoder error: {msg}"),
            EncodeError::Unsupported(msg) => write!(f, "unsupported: {msg}"),
        }
    }
}

impl std::error::Error for EncodeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    MalformedBitstream(String),
    Backend(String),
    Unsupported(String),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::MalformedBitstream(msg) => write!(f, "malformed bitstream: {msg}"),
            DecodeError::Backend(msg) => write!(f, "decoder error: {msg}"),
            DecodeError::Unsupported(msg) => write!(f, "unsupported: {msg}"),
        }
    }
}

impl std::error::Error for DecodeError {}

use crate::codec::VideoCodec;
use crate::frame::{DecodedVideoFrame, EncodedVideoFrame, RawVideoFrame};

/// Architecture doc §1's trait sketch, unchanged in shape: the `calls`
/// crate depends on this, never on `siar-media-av1` or
/// `siar-media-android` directly — same "domain doesn't import
/// infrastructure" rule plan.md §86 applies everywhere else in this
/// workspace.
pub trait VideoEncoder: Send {
    fn codec(&self) -> VideoCodec;
    fn encode(&mut self, frame: &RawVideoFrame) -> Result<EncodedVideoFrame, EncodeError>;
}

pub trait VideoDecoder: Send {
    fn codec(&self) -> VideoCodec;
    fn decode(&mut self, frame: &EncodedVideoFrame) -> Result<DecodedVideoFrame, DecodeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedAudioFrame {
    pub data: Vec<u8>,
    pub timestamp_micros: u64,
}

/// Audio encoders operate on interleaved 16-bit PCM — Opus's own native
/// input format (see `siar-media-audio`), so there's no float/PCM
/// conversion sitting in this trait boundary.
pub trait AudioEncoder: Send {
    fn encode(&mut self, pcm: &[i16]) -> Result<EncodedAudioFrame, EncodeError>;
}

pub trait AudioDecoder: Send {
    /// `fec` mirrors Opus's own forward-error-correction flag — decode
    /// from the next packet's FEC data when the previous packet was
    /// lost, rather than modeling packet loss as a separate trait
    /// method.
    fn decode(&mut self, packet: &[u8], fec: bool) -> Result<Vec<i16>, DecodeError>;
}
