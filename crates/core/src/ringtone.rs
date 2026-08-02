//! A ringing/ringback tone for the window between "call requested" and
//! "call connected" (or declined/timed out). Deliberately its own tiny
//! module rather than folded into `net::calls::audio`: it runs on a
//! separate device stream, before any call session exists, and needs
//! none of that module's decode/resample/jitter machinery — just a
//! repeating sine-wave cadence.
//!
//! Mirrors `net::calls::audio::run_playback_thread`'s shape (own thread
//! owns the `cpal::Stream` for its whole life, sample-format dispatch on
//! I16/F32) on purpose, so this doesn't need re-deriving from scratch.

//! A ringing/ringback tone for the window between "call requested" and
//! "call connected" (or declined/timed out). Deliberately its own tiny
//! module rather than folded into `net::calls::audio`: it runs on a
//! separate device stream, before any call session exists, and needs
//! none of that module's decode/resample/jitter machinery — just a
//! repeating sine-wave cadence.
//!
//! Desktop (Linux/Windows) only — mirrors `net::calls::audio`'s shape on
//! purpose (own thread owns the `cpal::Stream` for its whole life,
//! sample-format dispatch on I16/F32). On Android, `cpal` isn't even a
//! dependency (see the `target_os = "android"` exclusion in
//! `Cargo.toml`), and ringing sound there is expected to come from the
//! platform's own call/notification channel (`MainActivity.kt`-side —
//! see `BUILD_NOTES.md`'s foreground-service section), not from this
//! module — so `Ringtone::start` below is a real player on desktop and a
//! silent no-op on Android, same public API either way so call sites in
//! `ui/mod.rs` don't need their own `#[cfg]` branches.

#[cfg(not(target_os = "android"))]
mod desktop {
    use anyhow::{Context, Result};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// A handle to an in-progress ringtone. Playback stops when this is
    /// dropped — there's no separate `stop()` to remember to call, since
    /// every call site here already has a natural point (call answered,
    /// declined, hung up, timed out) where the handle just goes out of
    /// scope or gets set to `None`.
    pub struct Ringtone {
        stop_flag: Arc<AtomicBool>,
        // Not joined in `Drop` — see the impl below for why.
        _thread: Option<std::thread::JoinHandle<()>>,
    }

    impl Ringtone {
        /// `outgoing = true` for the caller's side (US ringback cadence:
        /// 2s tone / 4s silence, 440+480Hz) — `false` for the callee's
        /// side (a faster, higher repeating chirp), so incoming vs
        /// outgoing don't sound identical with your eyes off the screen.
        pub fn start(outgoing: bool) -> Self {
            let stop_flag = Arc::new(AtomicBool::new(false));
            let flag = stop_flag.clone();
            let thread = std::thread::Builder::new()
                .name("ringtone".into())
                .spawn(move || {
                    if let Err(e) = run(&flag, outgoing) {
                        // No speaker, or it went away mid-ring — not
                        // worth surfacing as a call-ending error the way
                        // a lost audio device mid-*call* is; the call
                        // itself hasn't started yet, and the visible
                        // ringing UI already tells the story without
                        // sound.
                        tracing::warn!(error = %e, "ringtone playback unavailable");
                    }
                })
                .expect("spawning the ringtone thread");
            Self {
                stop_flag,
                _thread: Some(thread),
            }
        }
    }

    impl Drop for Ringtone {
        fn drop(&mut self) {
            self.stop_flag.store(true, Ordering::Relaxed);
            // Deliberately not joining: `Drop` here usually runs on the
            // Dioxus UI task the moment a call is answered/declined/
            // ended, and that's exactly the moment you don't want
            // blocked on however long cpal takes to tear a stream down.
            // The playback thread notices the flag (checked every ~50ms,
            // see `run` below) and exits on its own; it holds nothing
            // else that needs synchronous cleanup.
        }
    }

    fn run(stop_flag: &Arc<AtomicBool>, outgoing: bool) -> Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("no speaker/headphones found")?;
        let supported = device
            .default_output_config()
            .context("speaker has no usable config")?;
        let channels = supported.channels() as usize;
        let sample_rate = supported.sample_rate() as f32;
        let sample_format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();

        // (on-seconds, off-seconds, tone A Hz, tone B Hz). The two
        // frequencies summed together is what makes it read as "a
        // phone", not just a single beep — same idea as real
        // DTMF/ringback tones.
        let (on_secs, off_secs, freq_a, freq_b): (f32, f32, f32, f32) = if outgoing {
            (2.0, 4.0, 440.0, 480.0)
        } else {
            (1.0, 0.5, 480.0, 620.0)
        };
        let period = on_secs + off_secs;
        let t = Arc::new(std::sync::Mutex::new(0.0f32));

        let err_fn = |e| tracing::warn!(error = %e, "ringtone stream error");

        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                let t = t.clone();
                device.build_output_stream(
                    config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        write_tone(
                            data,
                            channels,
                            sample_rate,
                            &t,
                            period,
                            on_secs,
                            freq_a,
                            freq_b,
                            |s| s,
                        )
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let t = t.clone();
                device.build_output_stream(
                    config,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        write_tone(
                            data,
                            channels,
                            sample_rate,
                            &t,
                            period,
                            on_secs,
                            freq_a,
                            freq_b,
                            |s| (s * i16::MAX as f32) as i16,
                        )
                    },
                    err_fn,
                    None,
                )
            }
            other => anyhow::bail!("unsupported speaker sample format: {other:?}"),
        }
        .context("building ringtone output stream")?;

        stream.play().context("starting ringtone playback")?;
        while !stop_flag.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        Ok(())
        // `stream` drops here, on this same thread, once the loop above
        // exits — cpal streams aren't required to be `Send`, which is
        // the whole reason this lives on its own thread instead of being
        // stored in `Ringtone` directly.
    }

    /// Fills one callback buffer with the ring cadence, quiet outside the
    /// "on" window. `to_sample` converts a `[-1.0, 1.0]` float to
    /// whatever the device's native sample type is.
    #[allow(clippy::too_many_arguments)]
    fn write_tone<S: Copy + Default>(
        data: &mut [S],
        channels: usize,
        sample_rate: f32,
        t: &Arc<std::sync::Mutex<f32>>,
        period: f32,
        on_secs: f32,
        freq_a: f32,
        freq_b: f32,
        to_sample: impl Fn(f32) -> S,
    ) {
        let mut t = t.lock().unwrap_or_else(|p| p.into_inner());
        for frame in data.chunks_mut(channels) {
            let phase = *t % period;
            // 0.12 amplitude per tone, well below clipping even summed —
            // this plays over a speaker next to someone's ear, not
            // through headphones tuned for the call itself, so
            // quieter-than-instinct on purpose.
            let sample = if phase < on_secs {
                0.12 * ((2.0 * std::f32::consts::PI * freq_a * *t).sin()
                    + (2.0 * std::f32::consts::PI * freq_b * *t).sin())
            } else {
                0.0
            };
            let s = to_sample(sample);
            for out in frame.iter_mut() {
                *out = s;
            }
            *t += 1.0 / sample_rate;
        }
    }
}

#[cfg(target_os = "android")]
mod android_stub {
    /// No-op on Android — see this module's top doc for why. Kept as a
    /// real (if empty) struct rather than a type alias so `Drop`-timing
    /// call sites in `ui/mod.rs` (`.write().take()` to stop playback)
    /// compile identically on both platforms.
    pub struct Ringtone;

    impl Ringtone {
        pub fn start(_outgoing: bool) -> Self {
            Self
        }
    }
}

#[cfg(target_os = "android")]
pub use android_stub::Ringtone;
#[cfg(not(target_os = "android"))]
pub use desktop::Ringtone;
