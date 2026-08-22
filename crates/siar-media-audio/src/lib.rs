//! Opus codec (`opus_codec.rs`) + real cross-platform device I/O
//! (`capture.rs`/`playback.rs` via `cpal`) implementing
//! `siar-media-core`'s `AudioEncoder`/`AudioDecoder` traits and device
//! access. Both the Opus FFI boundary and cpal's per-platform backends
//! use `unsafe` internally — permitted here per explicit instruction
//! (unsafe Rust calling into compiled libraries/OS APIs, rather than
//! this workspace authoring its own C/C++). Nothing in this crate's own
//! source is `unsafe`.
//!
//! Not included: sample-rate conversion (resampling) and jitter-buffer
//! logic tying capture → encode → network → decode → playback into one
//! pipeline. Those are `calls`-crate-level concerns (plan.md's own
//! architecture already separates "media plane" from "jitter buffer"),
//! and resampling specifically needs real DSP work this session isn't
//! going to fake.

pub mod capture;
pub mod opus_codec;
pub mod playback;

pub use capture::{AudioCapture, CaptureError, SharedPcmBuffer};
pub use opus_codec::{AudioChannels, OpusDecoder, OpusEncoder};
pub use playback::{AudioPlayback, PlaybackError, PlaybackQueue};
