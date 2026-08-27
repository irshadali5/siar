//! Real audio device capture via `cpal 0.18.1` — verified against its
//! actual source (`examples/record_wav.rs`, `src/traits.rs`):
//! `cpal::default_host()`, `host.default_input_device()`,
//! `device.default_input_config()`, `device.build_input_stream(config,
//! data_callback, error_callback, timeout)`, `stream.play()`.
//!
//! Deliberate scope limits, stated rather than silently assumed:
//!
//! - Only `SampleFormat::I16` and `SampleFormat::F32` input are
//!   handled — those cover the overwhelming majority of real devices.
//!   Anything else (`I8`/`I32`/`U16`/...) is a clean
//!   `CaptureError::UnsupportedSampleFormat` at stream-build time, not
//!   a silent garbage-audio path.
//! - **No sample-rate conversion.** Opus only accepts 8/12/16/24/48kHz.
//!   If the device's default input config reports a rate Opus doesn't
//!   support, `AudioCapture::start` returns an error rather than
//!   feeding the encoder a rate it'll reject anyway. Real device rate
//!   mismatches (a device that only offers 44.1kHz, for example) need
//!   a resampler (e.g. the `rubato` crate) in front of this — not
//!   included here, since getting resampling *quality* right (not just
//!   compiling) is its own significant piece of DSP work I'm not going
//!   to hand-wave through.
//! - The producer (cpal's realtime audio callback) and the consumer
//!   (whatever thread calls `SharedPcmBuffer::pull_frame`, typically an
//!   encoding loop) share a `Mutex<VecDeque<i16>>`. Taking a lock on
//!   the realtime audio thread is not what a latency-obsessive
//!   implementation would do — a lock-free SPSC ring buffer (e.g. the
//!   `ringbuf` crate) is the real answer, and swapping it in later
//!   doesn't change this module's public shape (`SharedPcmBuffer`'s
//!   methods stay the same either way). Correctness first, buffer
//!   swapped in once someone's actually measuring xruns on real
//!   hardware.
//! - `cpal::Stream` is not `Send` on every backend (WASAPI's COM
//!   objects on Windows, notably) — `AudioCapture` therefore is not
//!   `Send` either, and must be created and dropped on the same
//!   thread. Only `SharedPcmBuffer` (an `Arc<Mutex<..>>`) is meant to
//!   cross threads.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const OPUS_SUPPORTED_RATES: [u32; 5] = [8_000, 12_000, 16_000, 24_000, 48_000];

#[derive(Debug)]
pub enum CaptureError {
    NoInputDevice,
    Cpal(String),
    UnsupportedSampleFormat(SampleFormat),
    UnsupportedSampleRate(u32),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::NoInputDevice => write!(f, "no default input device available"),
            CaptureError::Cpal(msg) => write!(f, "cpal error: {msg}"),
            CaptureError::UnsupportedSampleFormat(fmt) => {
                write!(f, "unsupported input sample format: {fmt}")
            }
            CaptureError::UnsupportedSampleRate(rate) => {
                write!(f, "device default rate {rate}Hz is not one of Opus's supported rates (no resampler wired up)")
            }
        }
    }
}

impl std::error::Error for CaptureError {}

/// Shared handle the encoding loop uses to pull fixed-size frames —
/// `Clone`, `Send`, `Sync`, safe to hand to a different thread than the
/// one that created the `AudioCapture`.
#[derive(Clone)]
pub struct SharedPcmBuffer {
    inner: Arc<Mutex<VecDeque<i16>>>,
}

impl SharedPcmBuffer {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn push(&self, samples: &[i16]) {
        let mut buf = self.inner.lock().expect("pcm buffer lock poisoned");
        buf.extend(samples.iter().copied());
        // Bound the buffer so a stalled consumer (encoding loop wedged,
        // app suspended) doesn't grow this without limit — 2 seconds of
        // 48kHz stereo audio is a generous cap for "the encoder is
        // just a bit behind," not for "this is now a permanent leak."
        const MAX_BUFFERED_SAMPLES: usize = 48_000 * 2 * 2;
        while buf.len() > MAX_BUFFERED_SAMPLES {
            buf.pop_front();
        }
    }

    /// Returns exactly `frame_len` samples if that many are available,
    /// or `None` if not — never a short frame, since Opus's `encode`
    /// requires an exact frame-size multiple of the channel count.
    pub fn pull_frame(&self, frame_len: usize) -> Option<Vec<i16>> {
        let mut buf = self.inner.lock().expect("pcm buffer lock poisoned");
        if buf.len() < frame_len {
            return None;
        }
        Some(buf.drain(..frame_len).collect())
    }
}

pub struct AudioCapture {
    stream: Stream,
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioCapture {
    /// Opens the default input device at its default config and starts
    /// capturing immediately (`stream.play()`), returning both the
    /// (non-`Send`) stream handle — keep it alive for as long as
    /// capture should continue, dropping it stops the stream — and a
    /// `SharedPcmBuffer` the encoding loop pulls from.
    pub fn start() -> Result<(Self, SharedPcmBuffer), CaptureError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(CaptureError::NoInputDevice)?;
        let config = device
            .default_input_config()
            .map_err(|e| CaptureError::Cpal(e.to_string()))?;

        let sample_rate = config.sample_rate();
        if !OPUS_SUPPORTED_RATES.contains(&sample_rate) {
            return Err(CaptureError::UnsupportedSampleRate(sample_rate));
        }
        let channels = config.channels();

        let buffer = SharedPcmBuffer::new();
        let buffer_for_callback = buffer.clone();
        let err_fn = |err: cpal::Error| tracing::warn!(error = %err, "audio capture stream error");

        let sample_format = config.sample_format();
        let stream_config: cpal::StreamConfig = config.into();

        let stream = match sample_format {
            SampleFormat::I16 => device
                .build_input_stream(
                    stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| buffer_for_callback.push(data),
                    err_fn,
                    None,
                )
                .map_err(|e| CaptureError::Cpal(e.to_string()))?,
            SampleFormat::F32 => device
                .build_input_stream(
                    stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // Standard full-scale float-to-i16 conversion;
                        // clamp guards against a device that (against
                        // spec) hands back values outside [-1.0, 1.0].
                        let converted: Vec<i16> = data
                            .iter()
                            .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                            .collect();
                        buffer_for_callback.push(&converted);
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| CaptureError::Cpal(e.to_string()))?,
            other => return Err(CaptureError::UnsupportedSampleFormat(other)),
        };

        stream
            .play()
            .map_err(|e| CaptureError::Cpal(e.to_string()))?;

        Ok((
            Self {
                stream,
                sample_rate,
                channels,
            },
            buffer,
        ))
    }

    pub fn pause(&self) -> Result<(), CaptureError> {
        self.stream
            .pause()
            .map_err(|e| CaptureError::Cpal(e.to_string()))
    }

    pub fn resume(&self) -> Result<(), CaptureError> {
        self.stream
            .play()
            .map_err(|e| CaptureError::Cpal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_buffer_pulls_exact_frame_or_none() {
        let buffer = SharedPcmBuffer::new();
        buffer.push(&[1, 2, 3]);
        assert!(
            buffer.pull_frame(4).is_none(),
            "must not return a short frame"
        );
        buffer.push(&[4]);
        assert_eq!(buffer.pull_frame(4), Some(vec![1, 2, 3, 4]));
        assert!(buffer.pull_frame(1).is_none());
    }

    #[test]
    fn shared_buffer_caps_growth_when_unconsumed() {
        let buffer = SharedPcmBuffer::new();
        for _ in 0..300_000 {
            buffer.push(&[0]);
        }
        let remaining = buffer.inner.lock().unwrap().len();
        assert!(
            remaining <= 48_000 * 2 * 2,
            "buffer grew past its documented cap: {remaining}"
        );
    }
}
