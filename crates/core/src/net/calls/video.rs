//! Camera capture -> AV1 encode -> QUIC, and the reverse for display.
//! Sibling of `audio.rs`, same overall shape, different hardware/codec.
//!
//! Codec history in this module, for whoever reads this next: the first
//! pass used VP9 via `vpx-rs` (wraps libvpx through `env-libvpx-sys`,
//! which generates its FFI bindings with `bindgen`). That broke — not in
//! a subtle way, in a total one: `env-libvpx-sys` hasn't been updated
//! since September 2023, and its bindgen pass can't parse the headers of
//! a current (1.16) libvpx install, so every single field of
//! `vpx_codec_enc_cfg`/`vpx_codec_dec_cfg` came back as an opaque
//! placeholder — 89 compile errors from that one cause. Confirmed on a
//! real machine with `clang` installed and `pkg-config` correctly
//! resolving `vpx`, so it wasn't a missing-dependency problem — the sys
//! crate itself is simply stale relative to current libvpx. A
//! ffmpeg-subprocess workaround was drafted next (same libvpx underneath,
//! through a stable CLI/pipe boundary instead of FFI) but dropped before
//! landing here in favor of switching codecs outright:
//!
//! Now: AV1, via `rav1e` (encode) + `dav1d` (decode). Deliberately picked
//! for being structurally immune to the exact failure class above, not
//! just "hopefully fine this time":
//!   - `rav1e` is pure Rust — no C library, no bindgen, no FFI boundary
//!     at all on the encode side. There's no header-version mismatch to
//!     have, ever.
//!   - `dav1d` (VideoLAN's decoder — the fastest AV1 decoder that exists,
//!     hand-optimized asm throughout) is wrapped by the `dav1d`/
//!     `dav1d-sys` crates using *hand-written* bindings against dav1d's
//!     small, stable, opaque-handle C API (`dav1d_open`/`send_data`/
//!     `get_picture`), not bindgen against the full header. There's no
//!     giant config struct being field-mapped for bindgen to get wrong.
//!
//! VERIFY: `rav1e`/`dav1d`'s exact method names below were written
//! against each crate's documented usage, not compile-checked — no
//! `cargo`/`rustc` in this sandbox, same limitation noted elsewhere in
//! this codebase. The *shapes* (encoder: push frame, pull packet in a
//! loop; decoder: push data, pull picture in a loop) are each crate's
//! own well-established, stable pattern; only exact names might need a
//! small adjustment against the pinned versions in `Cargo.toml`.
//!
//! Performance/efficiency, since that was explicitly asked for: `rav1e`
//! is configured with `low_latency: true` (streams frame-by-frame with no
//! lookahead reordering buffer — the right choice for a live call; the
//! *wrong* choice is rav1e's default two-pass-friendly lookahead mode,
//! which trades latency for compression a call can't afford to spend)
//! and `SpeedSettings::from_preset(10)` (rav1e's fastest preset — same
//! "lowest-latency software path, not highest-compression" tradeoff
//! Opus/the old VP9 pass both made). `dav1d` needs no equivalent tuning
//! on the decode side — it's already the fastest AV1 decoder available
//! and decode is inherently far cheaper than encode regardless of codec.
//! Same honest hardware answer as before: no mature, safe, cross-platform
//! Rust crate exposes hardware AV1 encode/decode uniformly today (macOS
//! VideoToolbox does support AV1 decode on very recent Apple Silicon, but
//! nothing on Windows/Linux is remotely uniform), so "most efficient
//! available" means the fastest well-optimized software path, which is
//! exactly what the settings above configure.
//!
//! Hardware acceleration: unlike the flat "no safe cross-platform crate
//! for this exists" answer this module gave before, there's now a real
//! hardware path — see the `hw` submodule below for the full design.
//! Short version: `ffmpeg` (subprocess, not a Rust dependency) is probed
//! at call-start for a working hardware AV1 encoder (NVENC/QSV/AMF/
//! VideoToolbox, tried in that order with an actual dry-run test, not
//! just a name lookup) and decoder (`-hwaccel auto`, which is ffmpeg's
//! own built-in "try hardware, else software" resolution). Either
//! direction falls back automatically — to the pure-Rust
//! `rav1e`/`dav1d` path above — the moment `ffmpeg` isn't installed at
//! all, or no working hardware encoder is found. Nothing about this
//! probing is visible to the peer or renegotiated over the wire: AV1 is
//! decoder-implementation-agnostic, so which backend either side
//! happens to be running is a pure local implementation detail.
//!
//! Resolution/frame rate/framing/local-preview design: unchanged from the
//! VP9 draft's reasoning — see the equivalent sections below, they're the
//! same tradeoffs for the same reasons, just a different codec underneath.

use anyhow::{Context, Result};
use image::ImageEncoder;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh::EndpointId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, mpsc::UnboundedSender, oneshot};

#[cfg(not(target_os = "android"))]
use nokhwa::pixel_format::RgbFormat;
#[cfg(not(target_os = "android"))]
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
#[cfg(not(target_os = "android"))]
use nokhwa::Camera;
#[cfg(not(target_os = "android"))]
use rav1e::prelude::*;

#[cfg(target_os = "android")]
use super::mediacodec;
use super::{CallEvent, VideoHandoff};

pub const ALPN: &[u8] = b"iroh-messenger/call-video/2";

const WIDTH: usize = 640;
const HEIGHT: usize = 360;
const TARGET_FPS: usize = 15;
const TARGET_BITRATE_BPS: i32 = 500_000; // 500kbps, matches the resolution/fps above
const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024; // generous ceiling for one encoded AV1 temporal unit
#[cfg(not(target_os = "android"))]
const LOCAL_PREVIEW_EVERY_NTH: u32 = 3; // see "Local preview" below
const JPEG_QUALITY: u8 = 60; // display-only preview quality, not the wire codec

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VideoFrameMsg {
    /// Repeated on every frame so a malformed or mismatched stream is
    /// rejected before bytes reach the wrong decoder. The codec itself
    /// is negotiated directionally in the signaling handshake.
    codec: super::VideoCodec,
    payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VideoClipMsg {
    codec: super::VideoCodec,
    packets: Vec<Vec<u8>>,
}

fn encode_clip_packets(codec: super::VideoCodec, packets: Vec<Vec<u8>>) -> Result<Vec<u8>> {
    postcard::to_stdvec(&VideoClipMsg { codec, packets }).context("serializing video clip")
}

fn decode_clip_packets(blob: &[u8]) -> Result<VideoClipMsg> {
    if let Ok(clip) = postcard::from_bytes::<VideoClipMsg>(blob) {
        return Ok(clip);
    }
    // Backward compatibility for pre-codec-envelope desktop status clips,
    // which were always AV1.
    let packets = postcard::from_bytes::<Vec<Vec<u8>>>(blob).context("parsing video clip")?;
    Ok(VideoClipMsg {
        codec: super::VideoCodec::Av1,
        packets,
    })
}

/// True if this device has at least one usable camera right now. Run via
/// `spawn_blocking` by callers (see `CallProtocol::accept`) — device
/// enumeration is a short blocking OS call, not something to run inline
/// on the async runtime.
pub fn camera_available() -> bool {
    #[cfg(not(target_os = "android"))]
    {
        nokhwa::query(nokhwa::utils::ApiBackend::Auto)
            .map(|list| !list.is_empty())
            .unwrap_or(false)
    }

    #[cfg(target_os = "android")]
    {
        // QR capture is handled by the WebView file/camera input, but live
        // video needs a native Camera2 frame bridge before it can truthfully
        // advertise a usable camera to the call protocol.
        false
    }
}

/// Registered with the `Router` on `video::ALPN`. Its only job is handing
/// the freshly-accepted `Connection` back to whichever call is currently
/// waiting for it via `video_handoff` — see `super`'s module doc for why
/// this indirection exists instead of the callee dialing directly.
#[derive(Clone)]
pub struct VideoCallProtocol {
    video_handoff: VideoHandoff,
}

impl std::fmt::Debug for VideoCallProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoCallProtocol").finish_non_exhaustive()
    }
}

impl VideoCallProtocol {
    pub fn new(video_handoff: VideoHandoff) -> Self {
        Self { video_handoff }
    }
}

impl ProtocolHandler for VideoCallProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        // Take (not clone) — one connection satisfies exactly one waiting
        // call. A stray/unexpected dial with nothing waiting is just
        // dropped, same as the analogous "peer vanished" cases elsewhere
        // in this codebase's protocol handlers.
        let waiting = self.video_handoff.lock().unwrap().take();
        if let Some(tx) = waiting {
            let _ = tx.send(connection);
        }
        Ok(())
    }
}

/// Runs capture+encode+send and receive+decode concurrently until
/// `hangup_rx` fires or the peer's side ends. Mirrors `audio::run_session`'s
/// contract exactly (see that function's doc) — same shutdown guarantee,
/// same "whichever ends first tears down everything else" behavior.
pub async fn run_video_session(
    connection: &Connection,
    peer: EndpointId,
    send_codec: super::VideoCodec,
    receive_codec: super::VideoCodec,
    events: UnboundedSender<CallEvent>,
    mut hangup_rx: oneshot::Receiver<()>,
) -> Result<()> {
    #[cfg(not(target_os = "android"))]
    {
        let (encoded_tx, encoded_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (preview_tx, mut preview_rx) = mpsc::unbounded_channel::<Vec<u8>>();

        let capture = match start_capture(send_codec, encoded_tx, preview_tx) {
            Ok(c) => Some(c),
            Err(e) => {
                let _ = events.send(CallEvent::VideoUnavailable {
                    peer,
                    reason: describe_camera_error(&e),
                });
                None
            }
        };
        let decoder =
            start_decoder(receive_codec, peer, events.clone()).context("starting video decoder")?;

        let local_events = events.clone();
        let local_preview_task = tokio::spawn(async move {
            while let Some(jpeg) = preview_rx.recv().await {
                let data_uri = format!(
                    "data:image/jpeg;base64,{}",
                    data_encoding::BASE64.encode(&jpeg)
                );
                let _ = local_events.send(CallEvent::VideoFrame {
                    peer,
                    from_local_camera: true,
                    data_uri,
                });
            }
        });

        let mut send_task = tokio::spawn(send_captured_video(
            connection.clone(),
            send_codec,
            encoded_rx,
        ));
        let mut recv_task = tokio::spawn(receive_and_forward(
            connection.clone(),
            receive_codec,
            decoder.frame_tx.clone(),
        ));

        let device_failure_watch = async {
            loop {
                match &capture {
                    Some(c) if c.has_failed() => return,
                    None => return std::future::pending().await,
                    Some(_) => {}
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        };
        tokio::pin!(device_failure_watch);

        tokio::select! {
            _ = &mut hangup_rx => {}
            _ = &mut send_task => {}
            _ = &mut recv_task => {}
            _ = &mut device_failure_watch, if capture.is_some() => {}
        }
        send_task.abort();
        recv_task.abort();
        local_preview_task.abort();
        if let Some(c) = &capture {
            c.stop();
        }
        decoder.stop();
        Ok(())
    }

    #[cfg(target_os = "android")]
    {
        let (encoded_tx, encoded_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (preview_tx, mut preview_rx) = mpsc::unbounded_channel::<Vec<u8>>();

        let capture = match start_capture(send_codec, encoded_tx, preview_tx) {
            Ok(c) => Some(c),
            Err(e) => {
                let _ = events.send(CallEvent::VideoUnavailable {
                    peer,
                    reason: describe_camera_error(&e),
                });
                None
            }
        };
        let decoder = start_decoder(receive_codec, peer, events.clone())
            .context("starting Android AMediaCodec video decoder")?;

        let local_events = events.clone();
        let local_preview_task = tokio::spawn(async move {
            while let Some(jpeg) = preview_rx.recv().await {
                let data_uri = format!(
                    "data:image/jpeg;base64,{}",
                    data_encoding::BASE64.encode(&jpeg)
                );
                let _ = local_events.send(CallEvent::VideoFrame {
                    peer,
                    from_local_camera: true,
                    data_uri,
                });
            }
        });

        let mut send_task = tokio::spawn(send_captured_video(
            connection.clone(),
            send_codec,
            encoded_rx,
        ));
        let mut recv_task = tokio::spawn(receive_and_forward(
            connection.clone(),
            receive_codec,
            decoder.frame_tx.clone(),
        ));

        let device_failure_watch = async {
            loop {
                match &capture {
                    Some(c) if c.has_failed() => return,
                    None => return std::future::pending().await,
                    Some(_) => {}
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        };
        tokio::pin!(device_failure_watch);

        tokio::select! {
            _ = &mut hangup_rx => {}
            _ = &mut send_task => {}
            _ = &mut recv_task => {}
            _ = &mut device_failure_watch, if capture.is_some() => {}
        }
        send_task.abort();
        recv_task.abort();
        local_preview_task.abort();
        if let Some(c) = &capture {
            c.stop();
        }
        decoder.stop();
        Ok(())
    }
}

/// Turn a raw camera-open error into something worth actually showing
/// someone, rather than a driver's own error string. The "busy" case is
/// by far the most common one anyone testing this app will hit — most
/// OSes only let one process hold a webcam open at a time, so two
/// windows/accounts on the same laptop both trying to use the same
/// physical camera is expected to produce exactly this, on whichever
/// side didn't grab it first. Matched on substrings from common
/// nokhwa/OS-backend error text rather than a typed error variant, since
/// nokhwa surfaces backend-specific OS errors as opaque strings here.
fn describe_camera_error(e: &anyhow::Error) -> String {
    let msg = e.to_string().to_lowercase();
    if msg.contains("busy") || msg.contains("already") || msg.contains("in use") {
        "camera is already in use by another app (or another window of this app — only one process can hold a webcam open at a time)".to_string()
    } else if msg.contains("permission") || msg.contains("denied") {
        "camera access was denied — check this OS's camera permission settings for this app"
            .to_string()
    } else if msg.contains("didn't start") || msg.contains("no camera") || msg.contains("not found")
    {
        "no camera found on this device".to_string()
    } else {
        format!("camera unavailable ({e})")
    }
}

async fn send_captured_video(
    connection: Connection,
    codec: super::VideoCodec,
    mut encoded_rx: mpsc::UnboundedReceiver<Vec<u8>>,
) -> Result<()> {
    while let Some(payload) = encoded_rx.recv().await {
        let bytes = match postcard::to_stdvec(&VideoFrameMsg { codec, payload }) {
            Ok(b) => b,
            Err(e) => {
                tracing::debug!(error = %e, "dropping one unencodable video frame envelope");
                continue;
            }
        };
        let mut stream = match connection.open_uni().await {
            Ok(s) => s,
            Err(_) => return Ok(()), // connection's gone — call is over
        };
        if stream.write_all(&bytes).await.is_err() || stream.finish().is_err() {
            continue; // drop this one frame, keep the session alive
        }
        // Shorter than audio's — a stale video frame is even less useful
        // to wait around for than a stale audio batch.
        let _ = tokio::time::timeout(Duration::from_millis(300), stream.stopped()).await;
    }
    Ok(())
}

/// Just pulls frames off the wire and hands the raw AV1 temporal unit to
/// the decoder thread — all the actual decode/convert/JPEG-encode work
/// happens over there (see `start_decoder`), off the async runtime.
async fn receive_and_forward(
    connection: Connection,
    expected_codec: super::VideoCodec,
    frame_tx: std::sync::mpsc::Sender<Vec<u8>>,
) -> Result<()> {
    loop {
        let mut stream = match connection.accept_uni().await {
            Ok(s) => s,
            Err(_) => return Ok(()), // peer's side closed — call is over
        };
        let bytes = match stream.read_to_end(MAX_FRAME_BYTES).await {
            Ok(b) => b,
            Err(_) => return Ok(()),
        };
        let Ok(frame) = postcard::from_bytes::<VideoFrameMsg>(&bytes) else {
            continue; // malformed frame — skip it rather than ending the call
        };
        if frame.codec != expected_codec {
            tracing::warn!(?expected_codec, received_codec = ?frame.codec, "dropping video frame with non-negotiated codec");
            continue;
        }
        if frame_tx.send(frame.payload).is_err() {
            return Ok(()); // decoder thread's gone
        }
    }
}

/// Handle to the running capture+encode thread — same contract as
/// `audio::AudioThreadHandle` (see that struct's doc): dropping this
/// alone doesn't stop the thread, `stop()` must be called explicitly.
struct CaptureThreadHandle {
    stop_tx: std::sync::mpsc::Sender<()>,
    failed: Arc<std::sync::atomic::AtomicBool>,
}

impl CaptureThreadHandle {
    fn stop(&self) {
        let _ = self.stop_tx.send(());
    }
    fn has_failed(&self) -> bool {
        self.failed.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(not(target_os = "android"))]
fn start_capture(
    codec: super::VideoCodec,
    encoded_tx: mpsc::UnboundedSender<Vec<u8>>,
    preview_tx: mpsc::UnboundedSender<Vec<u8>>,
) -> Result<CaptureThreadHandle> {
    if codec != super::VideoCodec::Av1 {
        anyhow::bail!("desktop capture cannot encode negotiated codec {codec:?}");
    }
    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let failed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let thread_failed = failed.clone();

    let hw_encoder = hw::probe_hw_encoder();
    match &hw_encoder {
        Some(name) => tracing::info!(encoder = %name, "video: hardware encode available, using it"),
        None => tracing::info!("video: no working hardware encoder found, using software (rav1e)"),
    }

    // Neither `nokhwa::Camera` nor `rav1e::Context` are meant to hop
    // between threads mid-use — same reasoning as `audio.rs`'s capture
    // thread for `cpal::Stream`. Keeping capture *and* encode on this one
    // dedicated OS thread sidesteps the question entirely rather than
    // needing to prove either type is `Send`.
    std::thread::spawn(move || {
        let outcome = match hw_encoder {
            Some(encoder_name) => hw::run_capture_and_encode_thread_hw(
                &encoder_name,
                encoded_tx,
                preview_tx,
                &ready_tx,
                &stop_rx,
                thread_failed,
            ),
            None => run_capture_and_encode_thread(
                encoded_tx,
                preview_tx,
                &ready_tx,
                &stop_rx,
                thread_failed,
            ),
        };
        if let Err(e) = outcome {
            tracing::warn!(error = %e, "video capture/encode thread ended with error");
        }
    });

    match ready_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => Ok(CaptureThreadHandle { stop_tx, failed }),
        Ok(Err(e)) => anyhow::bail!("{e}"),
        Err(_) => anyhow::bail!("camera/encoder didn't start within 5s"),
    }
}

#[cfg(target_os = "android")]
fn start_capture(
    _codec: super::VideoCodec,
    _encoded_tx: mpsc::UnboundedSender<Vec<u8>>,
    _preview_tx: mpsc::UnboundedSender<Vec<u8>>,
) -> Result<CaptureThreadHandle> {
    anyhow::bail!("live camera capture is not available on Android yet")
}

/// Create a fresh AV1 encoder with this module's standard settings (see
/// module doc's "Performance/efficiency" section for the reasoning
/// behind each one). Shared by the live-call capture thread and
/// `encode_clip` (status video) — one encoder-construction path, not two.
#[cfg(not(target_os = "android"))]
fn new_av1_encoder() -> Result<rav1e::Context<u8>> {
    let enc_cfg = EncoderConfig {
        width: WIDTH,
        height: HEIGHT,
        speed_settings: SpeedSettings::from_preset(10), // fastest — see module doc
        low_latency: true,                              // no lookahead buffering — see module doc
        bit_depth: 8,
        chroma_sampling: ChromaSampling::Cs420,
        bitrate: TARGET_BITRATE_BPS,
        ..Default::default()
    };
    let cfg = Config::new().with_encoder_config(enc_cfg).with_threads(
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
    );
    cfg.new_context::<u8>()
        .map_err(|e| anyhow::anyhow!("couldn't create AV1 encoder: {e}"))
}

/// Resize + convert to I420 + encode one already-captured RGB frame,
/// returning whatever complete packets it produced (usually zero or one
/// — rav1e may buffer internally before a packet is ready). Shared by
/// the live-call capture thread and `encode_clip` — one per-frame encode
/// path, not two.
#[cfg(not(target_os = "android"))]
fn encode_frame_into_packets(ctx: &mut rav1e::Context<u8>, rgb: &image::RgbImage) -> Vec<Vec<u8>> {
    let resized = resize_to_target(rgb);
    let mut av1_frame = ctx.new_frame();
    rgb_to_i420_into(
        resized.as_raw(),
        resized.width() as usize,
        resized.height() as usize,
        &mut av1_frame,
    );
    if let Err(e) = ctx.send_frame(av1_frame) {
        tracing::debug!(error = %e, "dropping one unencodable video frame");
        return Vec::new();
    }
    let mut packets = Vec::new();
    loop {
        match ctx.receive_packet() {
            Ok(packet) => packets.push(packet.data),
            Err(EncoderStatus::Encoded) | Err(EncoderStatus::NeedMoreData) => break,
            Err(e) => {
                tracing::debug!(error = %e, "AV1 encoder packet pull ended early");
                break;
            }
        }
    }
    packets
}

/// Encode a short sequence of already-captured RGB frames (see
/// `record_video_clip`) into one self-contained AV1 clip blob — status
/// video's counterpart to a live call's continuous per-frame streaming,
/// built on the exact same `new_av1_encoder`/`encode_frame_into_packets`
/// helpers so there's one AV1 encode implementation in this codebase,
/// not two. The blob is just a `postcard`-encoded `Vec<Vec<u8>>` of raw
/// AV1 temporal units, in order — no container format needed since it's
/// only ever read back by `decode_clip` below, never by anything else.
///
/// Tries hardware first (same `hw::probe_hw_encoder` live calls use),
/// falling back to software on any failure — status video didn't get
/// this before (only live calls did), even though it's the exact same
/// AV1 bitstream either way and the exact same machine's GPU either way.
pub fn encode_clip(frames: &[image::RgbImage]) -> Result<Vec<u8>> {
    if let Some(encoder_name) = hw::probe_hw_encoder() {
        match hw::encode_clip_hw(&encoder_name, frames) {
            Ok(blob) => {
                tracing::info!(encoder = %encoder_name, "status video: hardware encode used");
                let packets = postcard::from_bytes::<Vec<Vec<u8>>>(&blob)
                    .context("parsing hardware AV1 clip")?;
                return encode_clip_packets(super::VideoCodec::Av1, packets);
            }
            Err(e) => {
                tracing::debug!(error = %e, "status video: hardware encode failed, falling back to software")
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        let mut ctx = new_av1_encoder()?;
        let mut packets: Vec<Vec<u8>> = Vec::new();
        for rgb in frames {
            packets.extend(encode_frame_into_packets(&mut ctx, rgb));
        }
        ctx.flush();
        while let Ok(packet) = ctx.receive_packet() {
            packets.push(packet.data);
        }
        encode_clip_packets(super::VideoCodec::Av1, packets)
    }

    #[cfg(target_os = "android")]
    {
        let hw_codecs = mediacodec::available_hw_encode_codecs();
        let codec = hw_codecs
            .first()
            .copied()
            .unwrap_or(super::VideoCodec::H264);
        let mut encoder = mediacodec::HwCodec::new_encoder(
            codec,
            WIDTH as i32,
            HEIGHT as i32,
            TARGET_BITRATE_BPS,
            TARGET_FPS as i32,
        )?;
        let mut raw_packets = Vec::new();
        for img in frames {
            let resized = resize_to_target(img);
            raw_packets.extend(encoder.encode_frame(resized.as_raw(), WIDTH as i32, HEIGHT as i32));
        }
        raw_packets.extend(encoder.finish_encoding());
        encode_clip_packets(codec, raw_packets)
    }
}

/// Decode a clip blob produced by `encode_clip` back into a sequence of
/// JPEG frames, ready for the story viewer to step through — built on
/// the exact same `decode_packet_into_jpegs` helper the live-call decode
/// thread uses.
///
/// Same hardware-first, software-fallback shape as `encode_clip`. Note
/// this works regardless of which side originally *encoded* the clip —
/// AV1 is the same bitstream either way (see the `hw` module's doc on
/// wire compatibility), so a clip one device encoded on hardware decodes
/// fine here even in software, and vice versa.
pub fn decode_clip(blob: &[u8]) -> Result<Vec<Vec<u8>>> {
    let clip = decode_clip_packets(blob)?;

    if hw::ffmpeg_available() && clip.codec == super::VideoCodec::Av1 {
        let legacy_blob =
            postcard::to_stdvec(&clip.packets).context("preparing AV1 clip for ffmpeg")?;
        match hw::decode_clip_hw(&legacy_blob) {
            Ok(jpegs) => return Ok(jpegs),
            Err(e) => {
                tracing::debug!(error = %e, "status video: hardware decode failed, falling back to software")
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        if clip.codec != super::VideoCodec::Av1 {
            anyhow::bail!("desktop cannot decode {:?} status video", clip.codec);
        }
        let mut decoder = dav1d::Decoder::new().context("opening dav1d AV1 decoder")?;
        let mut jpegs = Vec::new();
        for packet in clip.packets {
            jpegs.extend(decode_packet_into_jpegs(&mut decoder, packet));
        }
        Ok(jpegs)
    }

    #[cfg(target_os = "android")]
    {
        if !mediacodec::available_hw_decode_codecs().contains(&clip.codec) {
            anyhow::bail!("this device cannot decode {:?} status video", clip.codec);
        }
        let mut decoder =
            mediacodec::HwCodec::new_decoder(clip.codec, WIDTH as i32, HEIGHT as i32)?;
        let mut jpegs = Vec::new();
        for packet in clip.packets {
            let rgbs = decoder.decode_packet(&packet, WIDTH as i32, HEIGHT as i32);
            for rgb in &rgbs {
                if let Some(jpeg) = rgb_to_jpeg(rgb, WIDTH as u32, HEIGHT as u32) {
                    jpegs.push(jpeg);
                }
            }
        }
        let flushes = decoder.finish_decoding(WIDTH as i32, HEIGHT as i32);
        for rgb in &flushes {
            if let Some(jpeg) = rgb_to_jpeg(rgb, WIDTH as u32, HEIGHT as u32) {
                jpegs.push(jpeg);
            }
        }
        Ok(jpegs)
    }
}

/// Open the camera at its highest available frame rate. Shared by the
/// software capture thread, the hardware capture thread (`hw` module),
/// and `record_video_clip` — one camera-open path, not three.
#[cfg(not(target_os = "android"))]
fn open_camera() -> std::result::Result<Camera, String> {
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = Camera::new(CameraIndex::Index(0), requested)
        .map_err(|e| format!("couldn't open camera: {e}"))?;
    camera
        .open_stream()
        .map_err(|e| format!("couldn't start camera stream: {e}"))?;
    Ok(camera)
}

/// Record a short clip from the camera for `duration_secs`, returning
/// the captured frames (already resized to `WIDTH`×`HEIGHT` — see
/// `resize_to_target`) at this module's standard `TARGET_FPS`, ready for
/// `encode_clip`. Blocking — call via `spawn_blocking`. Used for status
/// video (`app::App::broadcast_status`'s video path); live calls use
/// `run_video_session` instead, which streams rather than batch-records.
pub fn record_video_clip(_duration_secs: u32) -> Result<Vec<image::RgbImage>> {
    #[cfg(not(target_os = "android"))]
    {
        let mut camera = open_camera().map_err(|e| anyhow::anyhow!("{e}"))?;
        let frame_interval = Duration::from_millis(1000 / TARGET_FPS as u64);
        let deadline = std::time::Instant::now() + Duration::from_secs(_duration_secs as u64);
        let mut frames = Vec::new();
        let mut last_capture = std::time::Instant::now() - frame_interval;
        while std::time::Instant::now() < deadline {
            if last_capture.elapsed() < frame_interval {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }
            let frame = camera
                .frame()
                .context("reading a camera frame while recording")?;
            let Ok(rgb) = frame.decode_image::<RgbFormat>() else {
                continue;
            };
            frames.push(resize_to_target(&rgb));
            last_capture = std::time::Instant::now();
        }
        let _ = camera.stop_stream();
        if frames.is_empty() {
            anyhow::bail!("no frames captured — camera may be in use elsewhere");
        }
        Ok(frames)
    }

    #[cfg(target_os = "android")]
    {
        anyhow::bail!("video recording is unavailable on Android in this build")
    }
}

#[cfg(not(target_os = "android"))]
fn run_capture_and_encode_thread(
    encoded_tx: mpsc::UnboundedSender<Vec<u8>>,
    preview_tx: mpsc::UnboundedSender<Vec<u8>>,
    ready_tx: &std::sync::mpsc::Sender<Result<(), String>>,
    stop_rx: &std::sync::mpsc::Receiver<()>,
    device_failed: Arc<std::sync::atomic::AtomicBool>,
) -> Result<()> {
    let mut camera = match open_camera() {
        Ok(c) => c,
        Err(e) => {
            let _ = ready_tx.send(Err(e));
            return Ok(());
        }
    };

    let mut ctx = match new_av1_encoder() {
        Ok(c) => c,
        Err(e) => {
            let _ = ready_tx.send(Err(e.to_string()));
            return Ok(());
        }
    };
    let _ = ready_tx.send(Ok(()));

    let frame_interval = Duration::from_millis(1000 / TARGET_FPS as u64);
    let mut frame_index: u32 = 0;
    let mut last_encode = std::time::Instant::now();
    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }
        let frame = match camera.frame() {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!(error = %e, "camera frame read failed — treating as device gone, ending video");
                device_failed.store(true, std::sync::atomic::Ordering::Relaxed);
                break;
            }
        };
        let Ok(rgb) = frame.decode_image::<RgbFormat>() else {
            continue;
        };
        let rgb = resize_to_target(&rgb);

        if frame_index.is_multiple_of(LOCAL_PREVIEW_EVERY_NTH) {
            if let Some(jpeg) = rgb_to_jpeg(rgb.as_raw(), rgb.width(), rgb.height()) {
                let _ = preview_tx.send(jpeg);
            }
        }
        frame_index = frame_index.wrapping_add(1);

        if last_encode.elapsed() < frame_interval {
            continue;
        }
        last_encode = std::time::Instant::now();

        for packet in encode_frame_into_packets(&mut ctx, &rgb) {
            let _ = encoded_tx.send(packet);
        }
    }

    let _ = camera.stop_stream();
    Ok(())
}

/// Handle to the running decode thread. Same "explicit `stop()`" contract
/// as `CaptureThreadHandle`.
struct DecoderThreadHandle {
    frame_tx: std::sync::mpsc::Sender<Vec<u8>>,
    stop_tx: std::sync::mpsc::Sender<()>,
}

impl DecoderThreadHandle {
    fn stop(&self) {
        let _ = self.stop_tx.send(());
    }
}

#[cfg(not(target_os = "android"))]
fn start_decoder(
    codec: super::VideoCodec,
    peer: EndpointId,
    events: UnboundedSender<CallEvent>,
) -> Result<DecoderThreadHandle> {
    if codec != super::VideoCodec::Av1 {
        anyhow::bail!("desktop decoder cannot decode negotiated codec {codec:?}");
    }
    let (frame_tx, frame_rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
    let use_hw = hw::ffmpeg_available();
    tracing::info!(
        backend = if use_hw {
            "hardware (ffmpeg -hwaccel auto)"
        } else {
            "software (dav1d)"
        },
        "video: decoder backend selected"
    );
    std::thread::spawn(move || {
        let outcome = if use_hw {
            hw::run_decode_thread_hw(peer, events, &frame_rx, &stop_rx)
        } else {
            run_decode_thread(peer, events, &frame_rx, &stop_rx)
        };
        if let Err(e) = outcome {
            tracing::warn!(error = %e, "video decode thread ended with error");
        }
    });
    Ok(DecoderThreadHandle { frame_tx, stop_tx })
}

#[cfg(target_os = "android")]
fn start_decoder(
    codec: super::VideoCodec,
    peer: EndpointId,
    events: UnboundedSender<CallEvent>,
) -> Result<DecoderThreadHandle> {
    let (frame_tx, frame_rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();

    std::thread::spawn(move || {
        let outcome = run_decode_thread_android(codec, peer, events, &frame_rx, &stop_rx);
        if let Err(e) = outcome {
            tracing::warn!(error = %e, "Android video decode thread ended with error");
        }
    });

    Ok(DecoderThreadHandle { frame_tx, stop_tx })
}

#[cfg(target_os = "android")]
fn run_decode_thread_android(
    codec: super::VideoCodec,
    peer: EndpointId,
    events: UnboundedSender<CallEvent>,
    frame_rx: &std::sync::mpsc::Receiver<Vec<u8>>,
    stop_rx: &std::sync::mpsc::Receiver<()>,
) -> Result<()> {
    let mut decoder = match mediacodec::HwCodec::new_decoder(codec, WIDTH as i32, HEIGHT as i32) {
        Ok(dec) => dec,
        Err(e) => anyhow::bail!("couldn't open AMediaCodec decoder: {e}"),
    };

    loop {
        if stop_rx.try_recv().is_ok() {
            let flushes = decoder.finish_decoding(WIDTH as i32, HEIGHT as i32);
            for rgb in &flushes {
                if let Some(jpeg) = rgb_to_jpeg(rgb, WIDTH as u32, HEIGHT as u32) {
                    let data_uri = format!(
                        "data:image/jpeg;base64,{}",
                        data_encoding::BASE64.encode(&jpeg)
                    );
                    let _ = events.send(CallEvent::VideoFrame {
                        peer,
                        from_local_camera: false,
                        data_uri,
                    });
                }
            }
            return Ok(());
        }

        let payload = match frame_rx.recv_timeout(Duration::from_millis(500)) {
            Ok(p) => p,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        };

        let rgb_frames = decoder.decode_packet(&payload, WIDTH as i32, HEIGHT as i32);
        for rgb in &rgb_frames {
            if let Some(jpeg) = rgb_to_jpeg(rgb, WIDTH as u32, HEIGHT as u32) {
                let data_uri = format!(
                    "data:image/jpeg;base64,{}",
                    data_encoding::BASE64.encode(&jpeg)
                );
                let _ = events.send(CallEvent::VideoFrame {
                    peer,
                    from_local_camera: false,
                    data_uri,
                });
            }
        }
    }
    Ok(())
}

/// Feed one AV1 temporal unit to `decoder` and return however many
/// decoded pictures it produced, already converted to JPEG (usually zero
/// or one — dav1d may buffer internally before a picture is ready).
/// Shared by the live-call decode thread and `decode_clip` (status
/// video) — one per-packet decode path, not two.
#[cfg(not(target_os = "android"))]
fn decode_packet_into_jpegs(decoder: &mut dav1d::Decoder, payload: Vec<u8>) -> Vec<Vec<u8>> {
    if let Err(e) = decoder.send_data(payload, None, None, None) {
        tracing::debug!(error = %e, "dropping one undecodable video frame");
        return Vec::new();
    }
    let mut jpegs = Vec::new();
    while let Ok(picture) = decoder.get_picture() {
        if let Some(jpeg) = i420_picture_to_jpeg(&picture) {
            jpegs.push(jpeg);
        }
    }
    jpegs
}

#[cfg(not(target_os = "android"))]
fn run_decode_thread(
    peer: EndpointId,
    events: UnboundedSender<CallEvent>,
    frame_rx: &std::sync::mpsc::Receiver<Vec<u8>>,
    stop_rx: &std::sync::mpsc::Receiver<()>,
) -> Result<()> {
    let mut decoder = dav1d::Decoder::new().context("opening dav1d AV1 decoder")?;
    loop {
        if stop_rx.try_recv().is_ok() {
            return Ok(());
        }
        let payload = match frame_rx.recv_timeout(Duration::from_millis(500)) {
            Ok(p) => p,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
        };
        for jpeg in decode_packet_into_jpegs(&mut decoder, payload) {
            let data_uri = format!(
                "data:image/jpeg;base64,{}",
                data_encoding::BASE64.encode(&jpeg)
            );
            let _ = events.send(CallEvent::VideoFrame {
                peer,
                from_local_camera: false,
                data_uri,
            });
        }
    }
}

/// Resize a captured camera frame to exactly `WIDTH`×`HEIGHT` — the
/// camera's actual native resolution (whatever
/// `RequestedFormatType::AbsoluteHighestFrameRate` picked) isn't
/// guaranteed to match the fixed size every downstream buffer (AV1
/// frame planes especially) is allocated for. A no-op cost-wise when it
/// already matches; `image`'s `Triangle` filter is a reasonable quality/
/// speed default for a resize this small and this frequent (every
/// captured frame).
fn resize_to_target(rgb: &image::RgbImage) -> image::RgbImage {
    if rgb.width() == WIDTH as u32 && rgb.height() == HEIGHT as u32 {
        return rgb.clone();
    }
    image::imageops::resize(
        rgb,
        WIDTH as u32,
        HEIGHT as u32,
        image::imageops::FilterType::Triangle,
    )
}

fn rgb_to_jpeg(rgb: &[u8], w: u32, h: u32) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity((w * h / 4) as usize);
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY);
    encoder
        .write_image(rgb, w, h, image::ExtendedColorType::Rgb8)
        .ok()?;
    Some(out)
}

/// BT.601 full-range RGB -> planar I420 (4:2:0), written directly into a
/// freshly-created `rav1e` frame's Y/U/V planes. Hand-rolled rather than
/// pulling in a dedicated color-conversion crate — it's ~15 lines of
/// well-known, stable arithmetic, and after two different native-codec
/// dependencies broke this module in a row, not adding a third external
/// dependency for something this small and this stable is itself a
/// reliability choice, not just a style one. Not broadcast-grade color
/// science (no BT.709/BT.2020 matrix selection, no studio-range
/// handling) — a video call's picture quality was never going to turn on
/// that, and 4:2:0 chroma is simple 2x2-box-averaged here rather than a
/// proper filtered downsample, which is the standard "good enough, don't
/// overbuild it" tradeoff for this exact use.
#[cfg(not(target_os = "android"))]
fn rgb_to_i420_into(rgb: &[u8], w: usize, h: usize, frame: &mut Frame<u8>) {
    let y_stride = frame.planes[0].cfg.stride;
    let u_stride = frame.planes[1].cfg.stride;
    let v_stride = frame.planes[2].cfg.stride;
    let [y_plane, u_plane, v_plane] = &mut frame.planes;

    let y_data = y_plane.data_origin_mut();
    for row in 0..h {
        for col in 0..w {
            let i = (row * w + col) * 3;
            if i + 2 >= rgb.len() {
                break;
            }
            let r = rgb[i] as i32;
            let g = rgb[i + 1] as i32;
            let b = rgb[i + 2] as i32;
            let y = (306 * r + 601 * g + 117 * b) >> 10;
            y_data[row * y_stride + col] = y.clamp(0, 255) as u8;
        }
    }
    let (cw, ch) = (w / 2, h / 2);
    let u_data = u_plane.data_origin_mut();
    let v_data = v_plane.data_origin_mut();
    for crow in 0..ch {
        for ccol in 0..cw {
            let (row, col) = (crow * 2, ccol * 2);
            let mut sum_r = 0i32;
            let mut sum_g = 0i32;
            let mut sum_b = 0i32;
            for (dr, dc) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                let i = ((row + dr) * w + (col + dc)) * 3;
                if i + 2 < rgb.len() {
                    sum_r += rgb[i] as i32;
                    sum_g += rgb[i + 1] as i32;
                    sum_b += rgb[i + 2] as i32;
                }
            }
            let r = sum_r / 4;
            let g = sum_g / 4;
            let b = sum_b / 4;
            let u = ((-173 * r - 339 * g + 512 * b + 131072) >> 10).clamp(0, 255);
            let v = ((512 * r - 429 * g - 83 * b + 131072) >> 10).clamp(0, 255);
            u_data[crow * u_stride + ccol] = u as u8;
            v_data[crow * v_stride + ccol] = v as u8;
        }
    }
}

/// The reverse conversion, from a decoded `dav1d` picture (planar I420)
/// straight to a JPEG for display — same reasoning as the encode side's
/// hand-rolled conversion above, and the same
/// `data:image/jpeg;base64,...` convention `ui::mod`'s avatar cache
/// already uses, so the UI side needs no new image-handling path.
#[cfg(not(target_os = "android"))]
fn i420_picture_to_jpeg(picture: &dav1d::Picture) -> Option<Vec<u8>> {
    let (w, h) = (picture.width() as usize, picture.height() as usize);
    let y = picture.plane(dav1d::PlanarImageComponent::Y);
    let u = picture.plane(dav1d::PlanarImageComponent::U);
    let v = picture.plane(dav1d::PlanarImageComponent::V);
    let y_stride = picture.stride(dav1d::PlanarImageComponent::Y) as usize;
    let c_stride = picture.stride(dav1d::PlanarImageComponent::U) as usize;

    let mut rgb = vec![0u8; w * h * 3];
    for row in 0..h {
        for col in 0..w {
            let yv = y[row * y_stride + col] as i32;
            let uv = u[(row / 2) * c_stride + col / 2] as i32 - 128;
            let vv = v[(row / 2) * c_stride + col / 2] as i32 - 128;
            let r = (yv + ((1436 * vv) >> 10)).clamp(0, 255);
            let g = (yv - ((352 * uv + 731 * vv) >> 10)).clamp(0, 255);
            let b = (yv + ((1815 * uv) >> 10)).clamp(0, 255);
            let o = (row * w + col) * 3;
            rgb[o] = r as u8;
            rgb[o + 1] = g as u8;
            rgb[o + 2] = b as u8;
        }
    }
    rgb_to_jpeg(&rgb, w as u32, h as u32)
}

/// Hardware-accelerated encode/decode via `ffmpeg` as a subprocess, with
/// the pure-Rust `rav1e`/`dav1d` path above as the automatic fallback
/// whenever this isn't available or working. Kept as a separate module
/// deliberately: everything above this line has zero native-toolchain
/// risk (see this file's own module doc for exactly why that matters
/// here), and that property is worth preserving as a real fallback, not
/// just a version-1 stepping stone — someone without `ffmpeg` installed,
/// or without a GPU AV1 encoder, still gets a fully working call through
/// the code above with nothing in this module ever touched.
///
/// Wire compatibility: regardless of which backend either side is
/// running, what goes out over the network is always one bare AV1
/// temporal unit per `VideoFrameMsg` (see that struct's doc) — AV1 is a
/// decoder-implementation-agnostic bitstream, so a peer encoding with
/// hardware here and a peer decoding with software `dav1d` there (or any
/// other combination) interoperate with no special-casing needed on
/// either end. This module's whole job is translating to/from that bare
/// format at its own boundary — `ffmpeg`'s IVF muxer/demuxer is used
/// purely as the container to talk to `ffmpeg` itself over its own
/// stdin/stdout pipes, stripped back off before anything hits the wire
/// or the decode channel.
///
/// Encode: real hardware presence is checked with an actual tiny dry-run
/// encode (not just "is this encoder name compiled into this ffmpeg
/// build" — that alone doesn't mean the GPU/driver on this machine
/// actually supports it), tried in a fixed vendor-priority order.
/// Decode: `-hwaccel auto` does ffmpeg's own "try hardware, else
/// software" internally, so there's no equivalent per-vendor probing
/// needed here — this module only has to check whether `ffmpeg` itself
/// is present at all.
#[cfg(target_os = "android")]
mod hw {
    use anyhow::{bail, Result};

    pub fn probe_hw_encoder() -> Option<String> {
        None
    }

    pub fn ffmpeg_available() -> bool {
        false
    }

    pub fn encode_clip_hw(_encoder_name: &str, _frames: &[image::RgbImage]) -> Result<Vec<u8>> {
        bail!("hardware video encode is unavailable on Android in this build")
    }

    pub fn decode_clip_hw(_blob: &[u8]) -> Result<Vec<Vec<u8>>> {
        bail!("hardware video decode is unavailable on Android in this build")
    }
}

#[cfg(not(target_os = "android"))]
mod hw {
    use super::{
        rgb_to_jpeg, CallEvent, EndpointId, UnboundedSender, HEIGHT, LOCAL_PREVIEW_EVERY_NTH,
        MAX_FRAME_BYTES, TARGET_BITRATE_BPS, TARGET_FPS, WIDTH,
    };
    use anyhow::Context;
    use nokhwa::pixel_format::RgbFormat;
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::mpsc;

    /// Tried in this order — roughly "most likely to be both present and
    /// worth using" first. A machine with none of these compiled into its
    /// `ffmpeg` build (or no working GPU encoder at all) just falls
    /// through to the software path automatically.
    ///
    /// `av1_vaapi` covers Linux systems without an NVENC/QSV-capable GPU
    /// (most AMD GPUs, and Intel systems where QSV isn't set up) — the
    /// generic VAAPI path every mainline Linux GPU driver exposes. It's
    /// listed last, after the vendor-specific ones, since where NVENC/QSV
    /// work they're generally the more mature/tested ffmpeg encode path.
    const HW_ENCODER_CANDIDATES: &[&str] = &[
        "av1_nvenc",
        "av1_qsv",
        "av1_amf",
        "av1_videotoolbox",
        "av1_vaapi",
    ];

    /// Extra ffmpeg args a given hardware encoder needs beyond plain
    /// `-c:v <name>`. NVENC/QSV/AMF/VideoToolbox accept ordinary
    /// system-memory frames directly in current ffmpeg builds — nothing
    /// extra needed. VAAPI encoders are the odd one out: they require an
    /// explicit render-node device and an upload filter to get frames
    /// from system memory onto the GPU surface the encoder actually
    /// reads from. Returned as `(before the first -i, after it)` — a
    /// hardware device needs to exist before ffmpeg opens any input
    /// against it, while the upload filter is a normal per-output filter
    /// that goes where `-vf` always goes, after `-i`. Mixing both into
    /// one flat arg list placed after `-i` (an earlier draft of this did
    /// exactly that) put `-vaapi_device` in the wrong position relative
    /// to the input it's supposed to set up.
    ///
    /// VERIFY: `/dev/dri/renderD128` is the default DRI render node on
    /// most single-GPU Linux systems, but isn't guaranteed on multi-GPU
    /// setups (a discrete GPU might be `renderD129` instead, for
    /// example). Not a silent-failure risk either way, though —
    /// `probe_hw_encoder` runs a real dry-run encode against whatever
    /// this returns before ever trusting `av1_vaapi`, so a wrong device
    /// path here just means the probe correctly fails closed and this
    /// falls through to software, same as "ffmpeg not installed" or "no
    /// GPU at all" already do.
    fn hw_encoder_extra_args(name: &str) -> (Vec<String>, Vec<String>) {
        if name == "av1_vaapi" {
            (
                vec![
                    "-vaapi_device".to_string(),
                    "/dev/dri/renderD128".to_string(),
                ],
                vec!["-vf".to_string(), "format=nv12,hwupload".to_string()],
            )
        } else {
            (Vec::new(), Vec::new())
        }
    }

    pub fn ffmpeg_available() -> bool {
        Command::new("ffmpeg")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Returns the name of the first hardware AV1 encoder that actually
    /// works on this machine right now, or `None` if `ffmpeg` isn't
    /// installed or none of `HW_ENCODER_CANDIDATES` do. The probe is a
    /// real 1-frame encode against a synthetic test pattern
    /// (`-f lavfi -i color=...`), not just a name lookup in `ffmpeg
    /// -encoders` — that list only says an encoder was compiled in, not
    /// that this machine's GPU/driver combination can actually drive it,
    /// and the difference matters (a compiled-in-but-nonfunctional
    /// encoder would otherwise fail loudly the moment a real call tried
    /// to use it, mid-ring, rather than being caught here up front).
    pub fn probe_hw_encoder() -> Option<String> {
        if !ffmpeg_available() {
            return None;
        }
        for candidate in HW_ENCODER_CANDIDATES {
            let (pre_input, post_input) = hw_encoder_extra_args(candidate);
            let ok = Command::new("ffmpeg")
                .args(["-hide_banner", "-loglevel", "error"])
                .args(&pre_input)
                .args(["-f", "lavfi", "-i", "color=size=64x64:rate=1"])
                .args(&post_input)
                .args(["-frames:v", "1", "-c:v", candidate, "-f", "null", "-"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                return Some(candidate.to_string());
            }
        }
        None
    }

    fn ivf_file_header(width: u16, height: u16, fps: u32) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0..4].copy_from_slice(b"DKIF");
        h[6..8].copy_from_slice(&32u16.to_le_bytes());
        h[8..12].copy_from_slice(b"AV01");
        h[12..14].copy_from_slice(&width.to_le_bytes());
        h[14..16].copy_from_slice(&height.to_le_bytes());
        h[16..20].copy_from_slice(&fps.to_le_bytes());
        h[20..24].copy_from_slice(&1u32.to_le_bytes());
        h
    }

    fn ivf_frame_header(size: u32, timestamp: u64) -> [u8; 12] {
        let mut h = [0u8; 12];
        h[0..4].copy_from_slice(&size.to_le_bytes());
        h[4..12].copy_from_slice(&timestamp.to_le_bytes());
        h
    }

    /// Same shape/contract as `super::run_capture_and_encode_thread` —
    /// `start_capture` picks whichever of the two to actually run.
    pub fn run_capture_and_encode_thread_hw(
        encoder_name: &str,
        encoded_tx: mpsc::UnboundedSender<Vec<u8>>,
        preview_tx: mpsc::UnboundedSender<Vec<u8>>,
        ready_tx: &std::sync::mpsc::Sender<Result<(), String>>,
        stop_rx: &std::sync::mpsc::Receiver<()>,
        device_failed: Arc<AtomicBool>,
    ) -> anyhow::Result<()> {
        let mut camera = match super::open_camera() {
            Ok(c) => c,
            Err(e) => {
                let _ = ready_tx.send(Err(e));
                return Ok(());
            }
        };

        let (pre_input, post_input) = hw_encoder_extra_args(encoder_name);
        let mut child = match Command::new("ffmpeg")
            .args(["-hide_banner", "-loglevel", "error"])
            .args(&pre_input)
            .args([
                "-f",
                "rawvideo",
                "-pixel_format",
                "rgb24",
                "-video_size",
                &format!("{WIDTH}x{HEIGHT}"),
                "-framerate",
                &TARGET_FPS.to_string(),
                "-i",
                "-",
            ])
            .args(&post_input)
            .args([
                "-c:v",
                encoder_name,
                "-b:v",
                &format!("{}k", TARGET_BITRATE_BPS / 1000),
                "-g",
                "30",
                "-f",
                "ivf",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = ready_tx.send(Err(format!(
                    "couldn't launch ffmpeg for hardware encode: {e}"
                )));
                return Ok(());
            }
        };
        let mut stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");

        // Dedicated reader thread: parse ffmpeg's IVF-muxed stdout,
        // stripping the container back off so what reaches `encoded_tx`
        // is the same bare-temporal-unit shape the software path
        // produces — see this module's doc on wire compatibility.
        let reader_encoded_tx = encoded_tx.clone();
        let reader_handle = std::thread::spawn(move || {
            let _ = read_ivf_stream(stdout, reader_encoded_tx);
        });

        let _ = ready_tx.send(Ok(()));
        let frame_interval = Duration::from_millis(1000 / TARGET_FPS as u64);
        let mut frame_index: u32 = 0;
        let mut last_encode = std::time::Instant::now();
        loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }
            let frame = match camera.frame() {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!(error = %e, "camera frame read failed — treating as device gone, ending video");
                    device_failed.store(true, std::sync::atomic::Ordering::Relaxed);
                    break;
                }
            };
            let Ok(rgb) = frame.decode_image::<RgbFormat>() else {
                continue;
            };
            let rgb = super::resize_to_target(&rgb);

            if frame_index.is_multiple_of(LOCAL_PREVIEW_EVERY_NTH) {
                if let Some(jpeg) = rgb_to_jpeg(rgb.as_raw(), rgb.width(), rgb.height()) {
                    let _ = preview_tx.send(jpeg);
                }
            }
            frame_index = frame_index.wrapping_add(1);

            if last_encode.elapsed() < frame_interval {
                continue;
            }
            last_encode = std::time::Instant::now();

            if stdin.write_all(rgb.as_raw()).is_err() {
                tracing::warn!("ffmpeg hardware encoder pipe closed — falling back would need a session restart, ending video");
                break;
            }
        }

        drop(stdin); // signal EOF to ffmpeg so it flushes and exits cleanly
        let _ = child.wait();
        let _ = reader_handle.join();
        let _ = camera.stop_stream();
        Ok(())
    }

    fn read_ivf_stream(
        mut stdout: impl Read,
        encoded_tx: UnboundedSender<Vec<u8>>,
    ) -> anyhow::Result<()> {
        let mut file_header = [0u8; 32];
        stdout.read_exact(&mut file_header)?;
        loop {
            let mut frame_header = [0u8; 12];
            if stdout.read_exact(&mut frame_header).is_err() {
                return Ok(()); // ffmpeg exited — encode session over
            }
            let size = u32::from_le_bytes(frame_header[0..4].try_into().unwrap()) as usize;
            if size == 0 || size > MAX_FRAME_BYTES {
                continue; // malformed/unreasonable — skip rather than trust it blindly
            }
            let mut payload = vec![0u8; size];
            if stdout.read_exact(&mut payload).is_err() {
                return Ok(());
            }
            let _ = encoded_tx.send(payload);
        }
    }

    /// Same shape/contract as `super::run_decode_thread` —
    /// `start_decoder` picks whichever of the two to actually run.
    pub fn run_decode_thread_hw(
        peer: EndpointId,
        events: UnboundedSender<CallEvent>,
        frame_rx: &std::sync::mpsc::Receiver<Vec<u8>>,
        stop_rx: &std::sync::mpsc::Receiver<()>,
    ) -> anyhow::Result<()> {
        let mut child = Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-hwaccel",
                "auto",
                "-f",
                "ivf",
                "-i",
                "-",
                "-f",
                "rawvideo",
                "-pixel_format",
                "rgb24",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut stdin = child.stdin.take().expect("piped stdin");
        let mut stdout = child.stdout.take().expect("piped stdout");

        let reader_handle = std::thread::spawn(move || {
            let frame_size = WIDTH * HEIGHT * 3;
            let mut buf = vec![0u8; frame_size];
            loop {
                if stdout.read_exact(&mut buf).is_err() {
                    return; // ffmpeg exited — decode session over
                }
                if let Some(jpeg) = rgb_to_jpeg(&buf, WIDTH as u32, HEIGHT as u32) {
                    let data_uri = format!(
                        "data:image/jpeg;base64,{}",
                        data_encoding::BASE64.encode(&jpeg)
                    );
                    let _ = events.send(CallEvent::VideoFrame {
                        peer,
                        from_local_camera: false,
                        data_uri,
                    });
                }
            }
        });

        stdin.write_all(&ivf_file_header(
            WIDTH as u16,
            HEIGHT as u16,
            TARGET_FPS as u32,
        ))?;
        let mut frame_index: u64 = 0;
        loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }
            let payload = match frame_rx.recv_timeout(Duration::from_millis(500)) {
                Ok(p) => p,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            };
            if stdin
                .write_all(&ivf_frame_header(payload.len() as u32, frame_index))
                .is_err()
                || stdin.write_all(&payload).is_err()
            {
                tracing::warn!("ffmpeg hardware decoder pipe closed, ending video");
                break;
            }
            frame_index += 1;
        }

        drop(stdin);
        let _ = child.wait();
        let _ = reader_handle.join();
        Ok(())
    }

    /// Batch (whole-clip, not streaming) hardware encode — the `hw`
    /// counterpart to `super::encode_clip`, used for status video rather
    /// than a live call. Same IVF-mux-then-strip approach as
    /// `run_capture_and_encode_thread_hw`, and the same reason for a
    /// dedicated reader thread rather than writing all frames then
    /// reading all output sequentially: `ffmpeg`'s stdout pipe has a
    /// limited OS buffer, and a clip's worth of frames can exceed it —
    /// writing everything to stdin first would block once that buffer
    /// filled, with nothing yet reading stdout to drain it. A real
    /// deadlock, not a theoretical one, for anything but the shortest
    /// clips.
    pub fn encode_clip_hw(
        encoder_name: &str,
        frames: &[image::RgbImage],
    ) -> anyhow::Result<Vec<u8>> {
        let Some(first) = frames.first() else {
            anyhow::bail!("no frames to encode");
        };
        let (w, h) = (first.width(), first.height());
        let (pre_input, post_input) = hw_encoder_extra_args(encoder_name);

        let mut child = Command::new("ffmpeg")
            .args(["-hide_banner", "-loglevel", "error"])
            .args(&pre_input)
            .args([
                "-f",
                "rawvideo",
                "-pixel_format",
                "rgb24",
                "-video_size",
                &format!("{w}x{h}"),
                "-framerate",
                &TARGET_FPS.to_string(),
                "-i",
                "-",
            ])
            .args(&post_input)
            .args([
                "-c:v",
                encoder_name,
                "-b:v",
                &format!("{}k", TARGET_BITRATE_BPS / 1000),
                "-g",
                "30",
                "-f",
                "ivf",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("couldn't launch ffmpeg for hardware clip encode")?;
        let mut stdin = child.stdin.take().expect("piped stdin");
        let mut stdout = child.stdout.take().expect("piped stdout");

        // Collects raw AV1 temporal units (IVF frame payloads, container
        // stripped) into the exact same `Vec<Vec<u8>>` shape
        // `super::encode_clip`'s software path produces — one blob
        // format regardless of which encoder actually produced it, same
        // wire-compatibility reasoning as the live streaming path.
        let reader_handle = std::thread::spawn(move || -> anyhow::Result<Vec<Vec<u8>>> {
            let mut file_header = [0u8; 32];
            stdout
                .read_exact(&mut file_header)
                .context("reading IVF file header from ffmpeg")?;
            let mut packets = Vec::new();
            loop {
                let mut frame_header = [0u8; 12];
                if stdout.read_exact(&mut frame_header).is_err() {
                    break; // ffmpeg exited — clip's fully encoded
                }
                let size = u32::from_le_bytes(frame_header[0..4].try_into().unwrap()) as usize;
                if size == 0 || size > MAX_FRAME_BYTES {
                    continue;
                }
                let mut payload = vec![0u8; size];
                if stdout.read_exact(&mut payload).is_err() {
                    break;
                }
                packets.push(payload);
            }
            Ok(packets)
        });

        for frame in frames {
            if stdin.write_all(frame.as_raw()).is_err() {
                break; // ffmpeg's gone — reader thread will still drain what it already produced
            }
        }
        drop(stdin); // signal EOF so ffmpeg flushes and exits
        let _ = child.wait();
        let packets = reader_handle
            .join()
            .map_err(|_| anyhow::anyhow!("hardware encode reader thread panicked"))??;
        if packets.is_empty() {
            anyhow::bail!("hardware encoder produced no output");
        }
        postcard::to_stdvec(&packets).context("serializing hardware-encoded AV1 clip")
    }

    /// Batch hardware decode — the `hw` counterpart to
    /// `super::decode_clip`. Takes the same blob shape either encode
    /// path produces (see wire-compatibility note in this module's doc)
    /// and returns JPEG frames the same way the software path does, so
    /// callers never need to know or care which one actually ran.
    pub fn decode_clip_hw(blob: &[u8]) -> anyhow::Result<Vec<Vec<u8>>> {
        let packets: Vec<Vec<u8>> =
            postcard::from_bytes(blob).context("parsing AV1 clip for hardware decode")?;

        let mut child = Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-hwaccel",
                "auto",
                "-f",
                "ivf",
                "-i",
                "-",
                "-f",
                "rawvideo",
                "-pixel_format",
                "rgb24",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("couldn't launch ffmpeg for hardware clip decode")?;
        let mut stdin = child.stdin.take().expect("piped stdin");
        let mut stdout = child.stdout.take().expect("piped stdout");

        // Same deadlock reasoning as `encode_clip_hw`: a dedicated reader
        // thread, not write-everything-then-read.
        let reader_handle = std::thread::spawn(move || -> anyhow::Result<Vec<Vec<u8>>> {
            let frame_size = WIDTH * HEIGHT * 3;
            let mut buf = vec![0u8; frame_size];
            let mut jpegs = Vec::new();
            loop {
                if stdout.read_exact(&mut buf).is_err() {
                    break; // ffmpeg exited — clip's fully decoded
                }
                if let Some(jpeg) = rgb_to_jpeg(&buf, WIDTH as u32, HEIGHT as u32) {
                    jpegs.push(jpeg);
                }
            }
            Ok(jpegs)
        });

        stdin
            .write_all(&ivf_file_header(
                WIDTH as u16,
                HEIGHT as u16,
                TARGET_FPS as u32,
            ))
            .context("writing IVF header to ffmpeg")?;
        for (i, packet) in packets.iter().enumerate() {
            if stdin
                .write_all(&ivf_frame_header(packet.len() as u32, i as u64))
                .is_err()
                || stdin.write_all(packet).is_err()
            {
                break;
            }
        }
        drop(stdin);
        let _ = child.wait();
        let jpegs = reader_handle
            .join()
            .map_err(|_| anyhow::anyhow!("hardware decode reader thread panicked"))??;
        if jpegs.is_empty() {
            anyhow::bail!("hardware decoder produced no output");
        }
        Ok(jpegs)
    }
}
