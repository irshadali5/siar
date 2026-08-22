//! Mirror of `capture.rs` for output — same verified `cpal 0.18.1` API
//! (`build_output_stream`, confirmed by direct signature read, takes
//! `StreamConfig` by value same as `build_input_stream`), same
//! documented limits: I16/F32 output formats only, no sample-rate
//! conversion (the decoder must already be producing audio at the
//! output device's native rate — `AudioPlayback::start` takes
//! `decoder_sample_rate` and errors if it doesn't match the device's
//! default output config rather than silently mis-playing audio at the
//! wrong speed), and the same not-`Send`-on-every-backend caveat on
//! `cpal::Stream` applies to `AudioPlayback` as it does to
//! `AudioCapture`.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum PlaybackError {
    NoOutputDevice,
    Cpal(String),
    UnsupportedSampleFormat(SampleFormat),
    /// The decoder's sample rate doesn't match the output device's
    /// default config rate — see this module's doc comment on why that
    /// isn't silently handled.
    SampleRateMismatch { decoder_rate: u32, device_rate: u32 },
}

impl std::fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaybackError::NoOutputDevice => write!(f, "no default output device available"),
            PlaybackError::Cpal(msg) => write!(f, "cpal error: {msg}"),
            PlaybackError::UnsupportedSampleFormat(fmt) => {
                write!(f, "unsupported output sample format: {fmt}")
            }
            PlaybackError::SampleRateMismatch { decoder_rate, device_rate } => write!(
                f,
                "decoder produces {decoder_rate}Hz but output device default is {device_rate}Hz (no resampler wired up)"
            ),
        }
    }
}

impl std::error::Error for PlaybackError {}

/// Push decoded PCM in here from wherever `OpusDecoder::decode` runs;
/// the output stream's realtime callback pulls from it. Same
/// `Mutex<VecDeque>`-not-lock-free caveat as `SharedPcmBuffer` in
/// `capture.rs` — see that module's doc comment, it applies here
/// identically.
#[derive(Clone)]
pub struct PlaybackQueue {
    inner: Arc<Mutex<VecDeque<i16>>>,
}

impl PlaybackQueue {
    fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(VecDeque::new())) }
    }

    pub fn push(&self, samples: &[i16]) {
        let mut buf = self.inner.lock().expect("playback queue lock poisoned");
        buf.extend(samples.iter().copied());
        // Same reasoning as `SharedPcmBuffer`'s cap: bound growth if
        // the output device is somehow starved of callbacks for a
        // while (app backgrounded, device hiccup) rather than let a
        // decode-ahead buffer grow forever.
        const MAX_BUFFERED_SAMPLES: usize = 48_000 * 2 * 2;
        while buf.len() > MAX_BUFFERED_SAMPLES {
            buf.pop_front();
        }
    }

    /// Fills `out` from the queue, zero-filling (silence) whatever's
    /// left if the queue underruns — an audio output callback must
    /// always fill its full buffer; leaving it partially written would
    /// play back whatever garbage was already in that memory.
    fn fill_or_silence(&self, out: &mut [i16]) {
        let mut buf = self.inner.lock().expect("playback queue lock poisoned");
        let available = buf.len().min(out.len());
        for (slot, sample) in out.iter_mut().zip(buf.drain(..available)) {
            *slot = sample;
        }
        for slot in out.iter_mut().skip(available) {
            *slot = 0;
        }
    }
}

pub struct AudioPlayback {
    stream: Stream,
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioPlayback {
    pub fn start(decoder_sample_rate: u32) -> Result<(Self, PlaybackQueue), PlaybackError> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(PlaybackError::NoOutputDevice)?;
        let config = device.default_output_config().map_err(|e| PlaybackError::Cpal(e.to_string()))?;

        let device_rate = config.sample_rate();
        if device_rate != decoder_sample_rate {
            return Err(PlaybackError::SampleRateMismatch { decoder_rate: decoder_sample_rate, device_rate });
        }
        let channels = config.channels();

        let queue = PlaybackQueue::new();
        let queue_for_callback = queue.clone();
        let err_fn = |err: cpal::Error| tracing::warn!(error = %err, "audio playback stream error");

        let sample_format = config.sample_format();
        let stream_config: cpal::StreamConfig = config.into();

        let stream = match sample_format {
            SampleFormat::I16 => device
                .build_output_stream(
                    stream_config,
                    move |out: &mut [i16], _: &cpal::OutputCallbackInfo| queue_for_callback.fill_or_silence(out),
                    err_fn,
                    None,
                )
                .map_err(|e| PlaybackError::Cpal(e.to_string()))?,
            SampleFormat::F32 => device
                .build_output_stream(
                    stream_config,
                    move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let mut scratch = vec![0i16; out.len()];
                        queue_for_callback.fill_or_silence(&mut scratch);
                        for (dst, src) in out.iter_mut().zip(scratch.iter()) {
                            *dst = *src as f32 / i16::MAX as f32;
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| PlaybackError::Cpal(e.to_string()))?,
            other => return Err(PlaybackError::UnsupportedSampleFormat(other)),
        };

        stream.play().map_err(|e| PlaybackError::Cpal(e.to_string()))?;

        Ok((Self { stream, sample_rate: device_rate, channels }, queue))
    }

    pub fn pause(&self) -> Result<(), PlaybackError> {
        self.stream.pause().map_err(|e| PlaybackError::Cpal(e.to_string()))
    }

    pub fn resume(&self) -> Result<(), PlaybackError> {
        self.stream.play().map_err(|e| PlaybackError::Cpal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn underrun_is_silence_not_garbage() {
        let queue = PlaybackQueue::new();
        queue.push(&[1, 2]);
        let mut out = [9i16; 5];
        queue.fill_or_silence(&mut out);
        assert_eq!(out, [1, 2, 0, 0, 0]);
    }

    #[test]
    fn full_buffer_is_filled_exactly() {
        let queue = PlaybackQueue::new();
        queue.push(&[1, 2, 3, 4, 5]);
        let mut out = [0i16; 3];
        queue.fill_or_silence(&mut out);
        assert_eq!(out, [1, 2, 3]);
        // Remaining 2 samples stay queued for the next callback.
        let mut out2 = [0i16; 3];
        queue.fill_or_silence(&mut out2);
        assert_eq!(out2, [4, 5, 0]);
    }
}
