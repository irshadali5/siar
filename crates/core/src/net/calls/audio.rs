//! Mic capture -> Opus encode -> QUIC, and the reverse for playback.
//!
//! Hardware-vs-software note, since that was the actual question asked:
//! Opus doesn't have a meaningful "dedicated hardware encoder" story on
//! desktop platforms the way video codecs (H.264/AV1) do — there's no
//! consumer desktop soundcard/DSP with an Opus ASIC any Rust crate could
//! target. The `opus` crate (this module's dependency) wraps libopus, the
//! reference C implementation — hand-optimized, SIMD paths where it
//! matters, and what every other Opus integration (browsers, WebRTC
//! stacks, voice chat apps) actually runs under the hood. So "hardware if
//! available, else efficient software" resolves to: there's no realistic
//! hardware path on this class of device, and libopus already *is* the
//! efficient software path — this isn't a corner cut for lack of time,
//! it's the actual state of the art here.
//!
//! Framing: every ~80ms of audio (4 Opus frames) is bundled into one QUIC
//! uni stream — a fresh stream per batch, `postcard`-encoded — rather
//! than one persistent stream carrying a length-prefixed sequence of
//! frames. That's a real latency/simplicity tradeoff: reusing this
//! codebase's *only* already-proven stream primitives (`open_uni`,
//! `write_all`, `finish`, `read_to_end`) instead of also needing an exact-
//! byte-count read (`read_exact`) whose availability on this iroh version
//! isn't exercised anywhere else in this codebase. If that turns out to
//! be available, switching to one persistent stream per direction removes
//! this batching delay — flagged here as the first thing to revisit.
//!
//! Resampling: if the microphone's native rate isn't 48kHz (Opus's own
//! preferred rate), a simple linear-interpolation resampler runs before
//! encoding. This is a known-crude-but-correct technique — adequate for
//! a first working version, not broadcast quality. A dedicated resampling
//! crate (e.g. `rubato`) would sound better; deferred to avoid adding yet
//! another new dependency's API surface in the same pass as everything
//! else here.
//!
//! Threading: `cpal::Stream` isn't `Send`, so it can't live inside a
//! tokio task. Capture and playback each get their own dedicated OS
//! thread that owns the cpal stream for the call's duration, bridged to
//! the async world via `tokio::sync::mpsc` channels (whose senders are
//! plain sync fns, safe to call from the cpal callback or a plain thread).

use anyhow::{Context, Result};
use iroh::endpoint::Connection;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use opus::{Application, Channels, Decoder as OpusDecoder, Encoder as OpusEncoder};

pub const SAMPLE_RATE: u32 = 48_000;
const FRAME_SAMPLES: usize = 960; // 20ms @ 48kHz mono — the standard Opus voice frame size
const MAX_ENCODED_FRAME: usize = 1275; // Opus's own documented max single-frame packet size
const FRAMES_PER_BATCH: usize = 4; // ~80ms per QUIC stream — see module doc's framing tradeoff
const MAX_BATCH_BYTES: usize = 16 * 1024; // generous upper bound for one batch's postcard bytes
const PLAYBACK_JITTER_TARGET: usize = FRAME_SAMPLES * 2; // ~40ms buffered before playback starts

/// Create an Opus encoder with this module's standard settings. Shared
/// by the live-call capture thread and `record_and_encode_voice_clip`
/// (status voice) — one encoder-construction path, not two.
fn new_opus_encoder() -> Result<OpusEncoder> {
    OpusEncoder::new(SAMPLE_RATE, Channels::Mono, Application::Voip)
        .context("couldn't create Opus encoder")
}

/// Create an Opus decoder with this module's standard settings. Shared
/// by the live-call decode thread and `decode_voice_clip` (status voice)
/// — one decoder-construction path, not two.
fn new_opus_decoder() -> Result<OpusDecoder> {
    OpusDecoder::new(SAMPLE_RATE, Channels::Mono).context("couldn't create Opus decoder")
}

/// Record a short voice clip from the mic for `duration_secs` and encode
/// it to a self-contained Opus clip blob — status voice's counterpart to
/// a live call's continuous streaming, built on the exact same
/// `new_opus_encoder`/`resample_mono`/`FRAME_SAMPLES` this module's live
/// capture path already uses. Blocking — call via `spawn_blocking`.
/// The blob is a `postcard`-encoded `Vec<Vec<u8>>` of raw Opus packets,
/// in order — no container format needed since it's only ever read back
/// by `decode_voice_clip` below.
pub fn record_and_encode_voice_clip(duration_secs: u32) -> Result<Vec<u8>> {
    let host = cpal::default_host();
    let device = host.default_input_device().context("no microphone found")?;
    let supported = device
        .default_input_config()
        .context("microphone has no usable config")?;
    let device_channels = supported.channels() as usize;
    let device_rate = supported.sample_rate();
    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();

    let raw_mono: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));
    let err_fn =
        move |e| tracing::warn!(error = %e, "microphone error while recording a status voice clip");

    let stream = match sample_format {
        cpal::SampleFormat::I16 => {
            let cb_buf = raw_mono.clone();
            device.build_input_stream(
                config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    cb_buf
                        .lock()
                        .unwrap()
                        .extend(downmix_i16(data, device_channels));
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::F32 => {
            let cb_buf = raw_mono.clone();
            device.build_input_stream(
                config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    cb_buf
                        .lock()
                        .unwrap()
                        .extend(downmix_f32(data, device_channels));
                },
                err_fn,
                None,
            )
        }
        other => anyhow::bail!("unsupported microphone sample format: {other:?}"),
    }
    .context("couldn't open microphone stream")?;
    stream.play().context("couldn't start microphone stream")?;
    std::thread::sleep(Duration::from_secs(duration_secs as u64));
    drop(stream);

    let raw = raw_mono.lock().unwrap().clone();
    if raw.is_empty() {
        anyhow::bail!("no audio captured — microphone may be in use elsewhere");
    }
    let (pcm, _) = resample_mono(&raw, device_rate, SAMPLE_RATE, 0.0);
    encode_pcm_clip(&pcm)
}

/// Packetize already-48kHz-mono PCM into the same self-contained Opus
/// clip blob format `record_and_encode_voice_clip` produces — factored
/// out so `decode_and_encode_audio_file` (a local audio file, not the
/// mic) can produce an identical blob without duplicating the framing
/// loop. `pcm` must already be resampled to `SAMPLE_RATE`/mono; this
/// function only packetizes, it doesn't resample or downmix.
fn encode_pcm_clip(pcm: &[i16]) -> Result<Vec<u8>> {
    let mut encoder = new_opus_encoder()?;
    let mut packets: Vec<Vec<u8>> = Vec::new();
    let mut offset = 0;
    while offset + FRAME_SAMPLES <= pcm.len() {
        let frame = &pcm[offset..offset + FRAME_SAMPLES];
        let mut out = [0u8; MAX_ENCODED_FRAME];
        match encoder.encode(frame, &mut out) {
            Ok(len) => packets.push(out[..len].to_vec()),
            Err(e) => tracing::debug!(error = %e, "dropping one unencodable audio frame"),
        }
        offset += FRAME_SAMPLES;
    }
    if packets.is_empty() {
        anyhow::bail!("clip too short to encode — need at least one 20ms frame");
    }
    postcard::to_stdvec(&packets).context("serializing Opus clip")
}

/// Decode an arbitrary local audio file (mp3/wav/flac/ogg/aac — see
/// `symphonia`'s feature list in `Cargo.toml`) and re-encode it into the
/// same self-contained Opus clip blob format `record_and_encode_voice_clip`
/// produces, so status voice can be either recorded live or attached
/// from an existing file through one identical downstream path
/// (`app::App::broadcast_status`'s `audio_clip` doesn't know or care
/// which this came from). Blocking — call via `spawn_blocking`, same as
/// every other CPU-bound step in this module.
///
/// VERIFY: written against `symphonia`'s own documented usage pattern
/// (`MediaSourceStream` → `probe` → per-track `Decoder` → `SampleBuffer`)
/// — a very standard, widely-copied shape, but without a compiler to
/// check the exact method names against, per this codebase's usual
/// standard for that.
pub fn decode_and_encode_audio_file(bytes: &[u8], file_name_hint: Option<&str>) -> Result<Vec<u8>> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let mut hint = Hint::new();
    if let Some(name) = file_name_hint {
        if let Some(ext) = std::path::Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
        {
            hint.with_extension(ext);
        }
    }
    let mss = MediaSourceStream::new(
        Box::new(std::io::Cursor::new(bytes.to_vec())),
        Default::default(),
    );
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .context("couldn't recognize this as an audio file")?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .context("no decodable audio track in this file")?
        .clone();
    let track_id = track.id;
    let source_rate = track
        .codec_params
        .sample_rate
        .context("audio file has no sample rate")?;
    let source_channels = track
        .codec_params
        .channels
        .context("audio file has no channel layout")?
        .count()
        .max(1);
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .context("no decoder available for this audio file's codec")?;

    let mut interleaved: Vec<f32> = Vec::new();
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => {
                let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                buf.copy_interleaved_ref(decoded);
                interleaved.extend_from_slice(buf.samples());
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue, // skip one bad packet, keep going
            Err(_) => break,
        }
    }
    if interleaved.is_empty() {
        anyhow::bail!("couldn't decode any audio from this file");
    }

    // Downmix to mono (simple average across channels — status voice
    // clips don't need stereo) and convert f32 [-1.0, 1.0] to i16, same
    // representation `resample_mono`/`encode_pcm_clip` already expect.
    let mono_f32: Vec<f32> = if source_channels == 1 {
        interleaved
    } else {
        interleaved
            .chunks(source_channels)
            .map(|frame| frame.iter().sum::<f32>() / source_channels as f32)
            .collect()
    };
    let mono_i16: Vec<i16> = mono_f32
        .iter()
        .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect();

    let (pcm, _) = resample_mono(&mono_i16, source_rate, SAMPLE_RATE, 0.0);
    encode_pcm_clip(&pcm)
}

/// Decode a clip blob produced by `record_and_encode_voice_clip` back
/// into mono 48kHz PCM samples — built on the exact same
/// `new_opus_decoder`/`FRAME_SAMPLES` the live-call decode thread uses.
pub fn decode_voice_clip(blob: &[u8]) -> Result<Vec<i16>> {
    let packets: Vec<Vec<u8>> = postcard::from_bytes(blob).context("parsing Opus clip")?;
    let mut decoder = new_opus_decoder()?;
    let mut pcm = Vec::new();
    for packet in packets {
        let mut out = vec![0i16; FRAME_SAMPLES];
        match decoder.decode(&packet, &mut out, false) {
            Ok(n) => pcm.extend_from_slice(&out[..n]),
            Err(e) => tracing::debug!(error = %e, "dropping one undecodable audio frame"),
        }
    }
    Ok(pcm)
}

/// Runs capture+send and receive+playback concurrently until `hangup_rx`
/// fires (local user hung up) or the peer's side ends (their hangup, or
/// the connection died). Returns once the call is fully over — both
/// background tasks are guaranteed stopped by the time this returns.
pub async fn run_session(
    connection: &Connection,
    mut hangup_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let (encoded_tx, encoded_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let (decoded_tx, decoded_rx) = mpsc::unbounded_channel::<Vec<i16>>();

    let capture = start_capture(encoded_tx).context("starting microphone capture")?;
    let playback = start_playback(decoded_rx).context("starting audio playback")?;

    let mut send_task = tokio::spawn(send_captured_audio(connection.clone(), encoded_rx));
    let mut recv_task = tokio::spawn(receive_and_decode(connection.clone(), decoded_tx));

    let device_failure_watch = async {
        loop {
            if capture.has_failed() || playback.has_failed() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    };
    tokio::pin!(device_failure_watch);

    tokio::select! {
        _ = &mut hangup_rx => {}
        _ = &mut send_task => {}
        _ = &mut recv_task => {}
        _ = &mut device_failure_watch => {}
    }
    send_task.abort();
    recv_task.abort();
    capture.stop();
    playback.stop();
    Ok(())
}

async fn send_batch(connection: &Connection, batch: &[Vec<u8>]) -> Result<()> {
    let mut stream = connection.open_uni().await?;
    let bytes = postcard::to_stdvec(batch)?;
    stream.write_all(&bytes).await?;
    stream.finish()?;
    // Short timeout, not the usual multi-second one used for signaling —
    // audio batches are frequent and latency-sensitive, so it's better to
    // drop one late-acknowledged batch's wait than let it back up the
    // ones behind it. See `net::contacts::send_one`'s doc for why this
    // wait exists at all (finish() alone doesn't guarantee delivery).
    let _ = tokio::time::timeout(Duration::from_millis(200), stream.stopped()).await;
    Ok(())
}

async fn send_captured_audio(
    connection: Connection,
    mut encoded_rx: mpsc::UnboundedReceiver<Vec<u8>>,
) -> Result<()> {
    let mut batch = Vec::with_capacity(FRAMES_PER_BATCH);
    while let Some(frame) = encoded_rx.recv().await {
        batch.push(frame);
        if batch.len() >= FRAMES_PER_BATCH {
            if let Err(e) = send_batch(&connection, &batch).await {
                tracing::debug!(error = %e, "dropping one audio batch (send failed)");
            }
            batch.clear();
        }
    }
    Ok(())
}

async fn receive_and_decode(
    connection: Connection,
    decoded_tx: mpsc::UnboundedSender<Vec<i16>>,
) -> Result<()> {
    let mut decoder = new_opus_decoder()?;
    loop {
        let mut stream = match connection.accept_uni().await {
            Ok(s) => s,
            Err(_) => return Ok(()), // peer's side closed — call is over
        };
        let bytes = match stream.read_to_end(MAX_BATCH_BYTES).await {
            Ok(b) => b,
            Err(_) => return Ok(()),
        };
        let Ok(batch) = postcard::from_bytes::<Vec<Vec<u8>>>(&bytes) else {
            continue; // malformed batch — skip it rather than ending the whole call
        };
        for frame in batch {
            let mut out = [0i16; FRAME_SAMPLES];
            match decoder.decode(&frame, &mut out, false) {
                Ok(n) => {
                    let _ = decoded_tx.send(out[..n].to_vec());
                }
                Err(e) => tracing::debug!(error = %e, "dropping one undecodable audio frame"),
            }
        }
    }
}

/// Handle to a running capture or playback thread — dropping this alone
/// does *not* stop the thread (see module doc: the cpal `Stream` has to
/// keep living on that thread for callbacks to keep firing), so callers
/// must call `stop()` explicitly.
struct AudioThreadHandle {
    stop_tx: std::sync::mpsc::Sender<()>,
    /// Set by the thread's own error callback if the device dies mid-call
    /// (see the `device_failed` flags in `run_capture_thread`/
    /// `run_playback_thread`). A capture failure already ends the call on
    /// its own (dropping `encoded_tx` ends `send_captured_audio`, which
    /// `run_session`'s `select!` already watches) — this is specifically
    /// for the case that wouldn't otherwise propagate: a *playback*-only
    /// failure doesn't touch either tokio task directly, so `run_session`
    /// polls this instead.
    failed: Arc<std::sync::atomic::AtomicBool>,
}

impl AudioThreadHandle {
    fn stop(&self) {
        let _ = self.stop_tx.send(());
    }

    fn has_failed(&self) -> bool {
        self.failed.load(std::sync::atomic::Ordering::Relaxed)
    }
}

fn start_capture(encoded_tx: mpsc::UnboundedSender<Vec<u8>>) -> Result<AudioThreadHandle> {
    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let failed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let thread_failed = failed.clone();

    std::thread::spawn(move || {
        let outcome = run_capture_thread(encoded_tx, &ready_tx, &stop_rx, thread_failed);
        if let Err(e) = outcome {
            tracing::warn!(error = %e, "audio capture thread ended with error");
        }
    });

    match ready_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => Ok(AudioThreadHandle { stop_tx, failed }),
        Ok(Err(e)) => anyhow::bail!("{e}"),
        Err(_) => anyhow::bail!("audio capture didn't start within 5s"),
    }
}

fn run_capture_thread(
    encoded_tx: mpsc::UnboundedSender<Vec<u8>>,
    ready_tx: &std::sync::mpsc::Sender<Result<(), String>>,
    stop_rx: &std::sync::mpsc::Receiver<()>,
    device_failed: Arc<std::sync::atomic::AtomicBool>,
) -> Result<()> {
    let setup = (|| -> Result<_> {
        let host = cpal::default_host();
        let device = host.default_input_device().context("no microphone found")?;
        let supported = device
            .default_input_config()
            .context("microphone has no usable config")?;
        Ok((device, supported))
    })();
    let (device, supported) = match setup {
        Ok(v) => v,
        Err(e) => {
            let _ = ready_tx.send(Err(e.to_string()));
            return Ok(());
        }
    };

    let device_channels = supported.channels() as usize;
    let device_rate = supported.sample_rate();
    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();

    // Raw mono samples at the device's native rate, appended by the audio
    // callback (whatever thread cpal runs it on) and drained by this
    // thread's own loop below — hence the `Mutex`, the only thing shared
    // between those two.
    let raw_mono: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));
    // Desktop edge case: the OS can yank the input device out from under
    // a running stream at any time — headphones/a USB mic unplugged
    // mid-call, sleep/wake on a laptop switching the default device,
    // PipeWire/PulseAudio restarting. cpal surfaces that as a call to the
    // error callback, not as the stream just quietly stopping — so
    // without this flag, the thread's own loop below would keep running
    // forever with a dead stream, silently sending nothing, and the
    // person on the call would just hear silence with no idea the call
    // is effectively over. Setting this lets the loop notice and exit,
    // which (via `encoded_tx` being dropped) ends the whole call cleanly
    // through the same path a normal hangup does. `device_failed` is
    // passed in from `start_capture` rather than created here so
    // `AudioThreadHandle` can also observe it (see that struct's doc) —
    // needed for the *playback* side, whose failure doesn't otherwise
    // propagate anywhere.
    let err_flag = device_failed.clone();
    let err_fn = move |e| {
        tracing::warn!(error = %e, "audio input stream error — treating as device gone, ending call");
        err_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    };

    let stream_result = match sample_format {
        cpal::SampleFormat::I16 => {
            let cb_buf = raw_mono.clone();
            device.build_input_stream(
                config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mono = downmix_i16(data, device_channels);
                    cb_buf.lock().unwrap().extend(mono);
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::F32 => {
            let cb_buf = raw_mono.clone();
            device.build_input_stream(
                config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono = downmix_f32(data, device_channels);
                    cb_buf.lock().unwrap().extend(mono);
                },
                err_fn,
                None,
            )
        }
        other => {
            let _ = ready_tx.send(Err(format!(
                "unsupported microphone sample format: {other:?}"
            )));
            return Ok(());
        }
    };
    let stream = match stream_result {
        Ok(s) => s,
        Err(e) => {
            let _ = ready_tx.send(Err(format!("couldn't open microphone stream: {e}")));
            return Ok(());
        }
    };
    if let Err(e) = stream.play() {
        let _ = ready_tx.send(Err(format!("couldn't start microphone stream: {e}")));
        return Ok(());
    }

    let mut encoder = match new_opus_encoder() {
        Ok(e) => e,
        Err(e) => {
            let _ = ready_tx.send(Err(e.to_string()));
            return Ok(());
        }
    };
    let _ = ready_tx.send(Ok(()));

    // See module doc: crude linear resampling, only actually exercised
    // when `device_rate != SAMPLE_RATE`. `carry` holds not-yet-consumed
    // raw samples across loop iterations; `frac_pos` is the resampler's
    // fractional read position within `carry`, also threaded through.
    let mut carry: Vec<i16> = Vec::new();
    let mut frac_pos: f64 = 0.0;
    let mut pcm_ready: Vec<i16> = Vec::new();

    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }
        if device_failed.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::debug!("microphone stream failed mid-call — stopping capture");
            break;
        }
        std::thread::sleep(Duration::from_millis(10));

        let mut fresh = raw_mono.lock().unwrap();
        if fresh.is_empty() {
            continue;
        }
        carry.append(&mut fresh);
        drop(fresh);

        let (resampled, new_pos) = resample_mono(&carry, device_rate, SAMPLE_RATE, frac_pos);
        let consumed = new_pos.floor() as usize;
        carry.drain(0..consumed.min(carry.len()));
        frac_pos = new_pos - consumed as f64;
        pcm_ready.extend(resampled);

        let mut offset = 0;
        while offset + FRAME_SAMPLES <= pcm_ready.len() {
            let frame = &pcm_ready[offset..offset + FRAME_SAMPLES];
            let mut out = [0u8; MAX_ENCODED_FRAME];
            match encoder.encode(frame, &mut out) {
                Ok(len) => {
                    let _ = encoded_tx.send(out[..len].to_vec());
                }
                Err(e) => tracing::debug!(error = %e, "dropping one unencodable audio frame"),
            }
            offset += FRAME_SAMPLES;
        }
        pcm_ready.drain(0..offset);
    }

    drop(stream); // stops callbacks; thread exits right after
    Ok(())
}

fn start_playback(mut decoded_rx: mpsc::UnboundedReceiver<Vec<i16>>) -> Result<AudioThreadHandle> {
    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    // The decode side (`receive_and_decode`, a tokio task) feeds this
    // ring buffer; the cpal output callback (this thread) drains it. A
    // plain thread — not tokio — owns the buffer draining + cpal stream,
    // same `Send`-ness reason as capture.
    let ring: Arc<Mutex<VecDeque<i16>>> = Arc::new(Mutex::new(VecDeque::new()));
    let feed_ring = ring.clone();

    std::thread::spawn(move || {
        // Feed loop: pull decoded frames off the tokio channel and push
        // them into the ring buffer. This needs a little runtime of its
        // own since `UnboundedReceiver::recv` is async — a single-
        // threaded current-thread runtime confined to this one OS
        // thread, not shared with the app's main runtime.
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!(error = %e, "couldn't start playback feed runtime");
                return;
            }
        };
        rt.block_on(async move {
            while let Some(pcm) = decoded_rx.recv().await {
                feed_ring.lock().unwrap().extend(pcm);
            }
        });
    });

    let cb_ring = ring.clone();
    let failed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let thread_failed = failed.clone();
    std::thread::spawn(move || {
        let outcome = run_playback_thread(cb_ring, &ready_tx, &stop_rx, thread_failed);
        if let Err(e) = outcome {
            tracing::warn!(error = %e, "audio playback thread ended with error");
        }
    });

    match ready_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => Ok(AudioThreadHandle { stop_tx, failed }),
        Ok(Err(e)) => anyhow::bail!("{e}"),
        Err(_) => anyhow::bail!("audio playback didn't start within 5s"),
    }
}

fn run_playback_thread(
    ring: Arc<Mutex<VecDeque<i16>>>,
    ready_tx: &std::sync::mpsc::Sender<Result<(), String>>,
    stop_rx: &std::sync::mpsc::Receiver<()>,
    device_failed: Arc<std::sync::atomic::AtomicBool>,
) -> Result<()> {
    let setup = (|| -> Result<_> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("no speaker/headphones found")?;
        let supported = device
            .default_output_config()
            .context("speaker has no usable config")?;
        Ok((device, supported))
    })();
    let (device, supported) = match setup {
        Ok(v) => v,
        Err(e) => {
            let _ = ready_tx.send(Err(e.to_string()));
            return Ok(());
        }
    };

    let device_channels = supported.channels() as usize;
    let device_rate = supported.sample_rate();
    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();
    // Same reasoning as the capture thread's identical flag: a speaker/
    // headphones being unplugged (or a sleep/wake device switch) mid-call
    // surfaces here as an error-callback call, not a quiet stop — without
    // this, playback would just silently stop producing sound with
    // nothing noticing the call is effectively over. Passed in (not
    // created here) so `AudioThreadHandle`/`run_session` can poll it too
    // — see `AudioThreadHandle::has_failed`'s doc for why that's needed
    // on this side specifically.
    let err_flag = device_failed.clone();
    let err_fn = move |e| {
        tracing::warn!(error = %e, "audio output stream error — treating as device gone, ending call");
        err_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    };

    // Decoded audio is always 48kHz mono; if the output device wants a
    // different rate/channel count, upmix/resample right here in the
    // callback — same crude-but-correct resampler as capture. Each match
    // arm below clones its own handles independently (rather than moving
    // a shared outer clone into whichever arm happens to run) so there's
    // no ambiguity about which arm owns what.
    let stream_result = match sample_format {
        cpal::SampleFormat::I16 => {
            let cb_ring = ring.clone();
            let resample_state = Arc::new(Mutex::new(0.0f64));
            device.build_output_stream(
                config,
                move |out: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    fill_output_i16(out, &cb_ring, device_channels, device_rate, &resample_state);
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::F32 => {
            let cb_ring = ring.clone();
            let resample_state = Arc::new(Mutex::new(0.0f64));
            device.build_output_stream(
                config,
                move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    fill_output_f32(out, &cb_ring, device_channels, device_rate, &resample_state);
                },
                err_fn,
                None,
            )
        }
        other => {
            let _ = ready_tx.send(Err(format!("unsupported speaker sample format: {other:?}")));
            return Ok(());
        }
    };
    let stream = match stream_result {
        Ok(s) => s,
        Err(e) => {
            let _ = ready_tx.send(Err(format!("couldn't open speaker stream: {e}")));
            return Ok(());
        }
    };
    if let Err(e) = stream.play() {
        let _ = ready_tx.send(Err(format!("couldn't start speaker stream: {e}")));
        return Ok(());
    }
    let _ = ready_tx.send(Ok(()));

    // Nothing to actively pump here — the callback above drains `ring`
    // whenever the OS wants more samples. Just wait for the stop signal
    // (or a device failure) while keeping `stream` alive.
    loop {
        if stop_rx.recv_timeout(Duration::from_millis(50)).is_ok() {
            break;
        }
        if device_failed.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::debug!("speaker stream failed mid-call — stopping playback");
            break;
        }
    }
    drop(stream);
    Ok(())
}

/// Pulls resampled/channel-matched samples out of `ring` and into `out`.
/// Underruns (not enough buffered yet — including the initial
/// `PLAYBACK_JITTER_TARGET` warmup) fill with silence rather than
/// stalling the callback, since an audio callback blocking is far worse
/// than a brief silent gap.
fn fill_output_i16(
    out: &mut [i16],
    ring: &Arc<Mutex<VecDeque<i16>>>,
    device_channels: usize,
    device_rate: u32,
    resample_pos: &Arc<Mutex<f64>>,
) {
    let needed_mono = out.len() / device_channels.max(1);
    let mono = drain_resampled(ring, device_rate, resample_pos, needed_mono);
    for (i, frame) in out.chunks_mut(device_channels.max(1)).enumerate() {
        let s = mono.get(i).copied().unwrap_or(0);
        for ch in frame {
            *ch = s;
        }
    }
}

fn fill_output_f32(
    out: &mut [f32],
    ring: &Arc<Mutex<VecDeque<i16>>>,
    device_channels: usize,
    device_rate: u32,
    resample_pos: &Arc<Mutex<f64>>,
) {
    let needed_mono = out.len() / device_channels.max(1);
    let mono = drain_resampled(ring, device_rate, resample_pos, needed_mono);
    for (i, frame) in out.chunks_mut(device_channels.max(1)).enumerate() {
        let s = mono.get(i).copied().unwrap_or(0) as f32 / i16::MAX as f32;
        for ch in frame {
            *ch = s;
        }
    }
}

fn drain_resampled(
    ring: &Arc<Mutex<VecDeque<i16>>>,
    device_rate: u32,
    resample_pos: &Arc<Mutex<f64>>,
    needed_mono: usize,
) -> Vec<i16> {
    let mut buf = ring.lock().unwrap();
    if buf.len() < PLAYBACK_JITTER_TARGET {
        return Vec::new(); // still warming up — play silence rather than choppy early audio
    }
    if device_rate == SAMPLE_RATE {
        let take = needed_mono.min(buf.len());
        return buf.drain(0..take).collect();
    }
    // Need more raw (48kHz) samples than `needed_mono` output samples
    // when downsampling, or fewer when upsampling — approximate with the
    // ratio and just take what's there if we're short.
    let ratio = SAMPLE_RATE as f64 / device_rate as f64;
    let raw_needed = ((needed_mono as f64) * ratio).ceil() as usize + 2;
    let take = raw_needed.min(buf.len());
    let raw: Vec<i16> = buf.drain(0..take).collect();
    drop(buf);
    let mut pos = resample_pos.lock().unwrap();
    let (resampled, new_pos) = resample_mono(&raw, SAMPLE_RATE, device_rate, *pos);
    // `raw` is fully consumed each call (unlike the capture-side carry
    // buffer) since it's freshly drained from `ring` here, not persisted
    // across calls — only the fractional position threads through.
    *pos = new_pos.fract();
    resampled
}

fn downmix_i16(data: &[i16], channels: usize) -> Vec<i16> {
    if channels <= 1 {
        return data.to_vec();
    }
    let mut out = Vec::with_capacity(data.len() / channels);
    for frame in data.chunks(channels) {
        let sum: i32 = frame.iter().map(|&s| s as i32).sum();
        out.push((sum / channels as i32) as i16);
    }
    out
}

fn downmix_f32(data: &[f32], channels: usize) -> Vec<i16> {
    if channels == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(data.len() / channels);
    let channels_f32 = channels as f32;
    for frame in data.chunks(channels) {
        let avg = frame.iter().sum::<f32>() / channels_f32;
        out.push((avg.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
    }
    out
}

/// Fast fixed-point linear-interpolation resampler for mono audio.
fn resample_mono(input: &[i16], from_rate: u32, to_rate: u32, start_pos: f64) -> (Vec<i16>, f64) {
    if from_rate == to_rate || input.len() < 2 {
        return (input.to_vec(), input.len() as f64);
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let expected_len = ((input.len() as f64 / ratio) as usize).saturating_add(4);
    let mut out = Vec::with_capacity(expected_len);
    let ratio_fp = (ratio * 65536.0) as u64;
    let mut pos_fp = (start_pos * 65536.0) as u64;

    while (pos_fp >> 16) as usize + 1 < input.len() {
        let idx = (pos_fp >> 16) as usize;
        let frac_fp = (pos_fp & 0xFFFF) as i32;
        let s0 = input[idx] as i32;
        let s1 = input[idx + 1] as i32;
        let sample = s0 + (((s1 - s0) * frac_fp) >> 16);
        out.push(sample.clamp(-32768, 32767) as i16);
        pos_fp += ratio_fp;
    }
    let final_pos = pos_fp as f64 / 65536.0;
    (out, final_pos)
}
