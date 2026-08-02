//! Hardware H.264/H.265 video codec access via the Android NDK's
//! `AMediaCodec` C API (`<media/NdkMediaCodec.h>`), for calls/status
//! video on Android specifically.
//!
//! # Why this exists alongside the AV1 path
//!
//! AV1 hardware decode/encode is still rare on Android devices (most
//! don't have it at all as of this writing), while H.264 hardware is
//! effectively universal and H.265 hardware is common on anything from
//! roughly the last several years — so on Android, "prefer hardware
//! acceleration" (this app's own stated priority, and the load-bearing
//! reason this module exists at all — battery drain from software
//! video codec work is real and this is a mobile, battery-powered
//! platform) means H.264/H.265 via `AMediaCodec`, not AV1 via `rav1e`/
//! `dav1d` software. See `net::calls::negotiate_send_codec` for how a call's
//! two ends agree on which codec to actually use — this module only
//! provides the codec itself, not the decision to prefer it.
//!
//! # Read this before trusting this file
//!
//! Everything else added to `net::calls` this pass (`net::mesh`, the
//! `if-addrs`/`bluer`/`symphonia` additions, the ffmpeg `hw` subprocess
//! module) calls into safe Rust APIs from crates whose exact method
//! signatures were checked against real documentation or published
//! examples before being used. This file is different in kind, not
//! just degree: it's raw `unsafe` FFI into a C API, via `ndk-sys`'s
//! bindings rather than hand-declared `extern "C"` signatures (using
//! `ndk-sys` specifically *because* its struct layouts/signatures are
//! generated from the real NDK headers, which is meaningfully safer
//! than retyping them by hand) — but there is still no compiler
//! available in this environment to check any of it against.
//! `AMediaCodec`'s function names, `AMediaCodecBufferInfo`'s field
//! layout, and the buffer-flag/status constants below were checked
//! against the NDK's own public header source and multiple independent
//! third-party usage examples (not training-data recall alone) before
//! being written — high confidence, not a blind guess — but this is
//! the one file in this codebase where "wrong" means a memory-safety
//! bug, not a compile error or a graceful fallback. **Test this on a
//! real device before trusting it in a release build**, more than
//! anything else this pass touched.
//!
//! # Scope
//!
//! Buffer-mode (not `Surface`-mode) encode and decode — raw I420 YUV
//! frames in, encoded Annex-B NAL units out, and the reverse for
//! decode. No `ANativeWindow` involved on either side: this app's
//! existing display pipeline already goes through a JPEG-per-frame data
//! URI (see `rgb_to_jpeg` and `CallEvent::VideoFrame`), so a decoded
//! frame needs to become an RGB buffer this process can touch directly,
//! not a frame handed straight to a compositor surface. That's a
//! deliberate simplicity trade-off, not an oversight: skipping the
//! surface path means no `ANativeWindow`/`SurfaceTexture` JNI
//! plumbing at all, at the cost of one extra YUV→RGB software
//! conversion per frame — cheap relative to what hardware encode/decode
//! itself already saves.

use super::VideoCodec;
use anyhow::{bail, Context, Result};
use std::ffi::CString;
use std::ptr;

#[cfg(target_os = "android")]
#[link(name = "mediandk")]
extern "C" {}

/// `android.media.MediaFormat.COLOR_FormatYUV420Flexible` — the
/// input/output color format this module configures every codec for.
/// "Flexible" is Android's way of saying the actual buffer layout is up
/// to the specific device/codec — commonly tightly-packed planar I420
/// (separate Y, U, V planes), but semi-planar NV12 (Y plane, then
/// interleaved U/V) is also common, arguably more common in practice
/// since it's the native layout a lot of hardware encoders use
/// internally.
///
/// **Known real limitation, not a hypothetical one**: `rgb_to_i420`/
/// `i420_to_rgb` below assume tightly-packed planar I420 unconditionally
/// — they don't check the codec's actual negotiated layout via
/// `AMediaCodec_getOutputFormat`/`AMediaFormat_getInt32("color-format"
/// / "stride" / "slice-height")` the way a fully-robust implementation
/// would. On a device whose codec resolves "flexible" to semi-planar
/// NV12 (or to padded planes with a stride wider than the frame width),
/// this will produce visibly wrong colors or a skewed image, not a
/// crash. Doing this properly means branching on the real negotiated
/// format and handling row-by-row copies for any stride padding —
/// real, scoped work, not done in this pass. Test on more than one
/// device model before trusting this in a release build; "works on my
/// test device" doesn't rule this out.
const COLOR_FORMAT_YUV420_FLEXIBLE: i32 = 0x7f42_0888;

/// `AMEDIACODEC_CONFIGURE_FLAG_ENCODE` — passed to `AMediaCodec_configure`
/// to select encoder rather than decoder behavior for a given format.
const CONFIGURE_FLAG_ENCODE: u32 = 1;
/// `AMEDIACODEC_BUFFER_FLAG_END_OF_STREAM`.
const BUFFER_FLAG_END_OF_STREAM: u32 = 4;
/// `AMEDIACODEC_BUFFER_FLAG_CODEC_CONFIG` — marks an output buffer as
/// codec-specific data (H.264/H.265 SPS/PPS) rather than an actual
/// frame; still needs to go out over the wire (a peer's decoder needs
/// it before it can decode anything), just not treated as "a frame" by
/// anything counting frames.
const BUFFER_FLAG_CODEC_CONFIG: u32 = 2;
/// `AMEDIACODEC_INFO_TRY_AGAIN_LATER` — not an error, just "nothing
/// ready yet, ask again."
const INFO_TRY_AGAIN_LATER: isize = -1;
/// `AMEDIACODEC_INFO_OUTPUT_FORMAT_CHANGED` — expected once near the
/// start of a decode session (the codec announcing the real output
/// format/color format), harmless to just skip past.
const INFO_OUTPUT_FORMAT_CHANGED: isize = -2;
/// `AMEDIACODEC_INFO_OUTPUT_BUFFERS_CHANGED` — legacy pre-API-21
/// buffer-array-invalidation signal; this module only ever calls
/// `AMediaCodec_getOutputBuffer` by fresh index after a successful
/// dequeue, never caches a buffer array, so there's nothing to
/// invalidate here — skipped past the same as a format change.
const INFO_OUTPUT_BUFFERS_CHANGED: isize = -3;
/// How long a dequeue call blocks waiting for a buffer before giving up
/// for this iteration and letting the caller retry — matches the sort
/// of poll interval the NDK's own sample code uses for buffer-mode (as
/// opposed to callback-mode) `AMediaCodec` usage.
const DEQUEUE_TIMEOUT_US: i64 = 10_000; // 10ms

fn mime_for(codec: VideoCodec) -> &'static str {
    match codec {
        VideoCodec::H264 => "video/avc",
        VideoCodec::H265 => "video/hevc",
        VideoCodec::Av1 => "video/av01",
    }
}

/// Owns one `AMediaCodec*` for the lifetime of a call or a status clip
/// encode/decode pass. `Drop` stops and deletes the underlying codec —
/// this is the one thing that has to happen reliably given everything
/// below it is `unsafe`: an `AMediaCodec` instance holds real hardware
/// codec resources (there are only so many concurrent hardware sessions
/// a device supports), and leaking one would eventually make every
/// *other* video path on the device — including whatever app the user
/// switches to next — start failing to get hardware acceleration too.
pub struct HwCodec {
    raw: *mut ndk_sys::AMediaCodec,
    encoding: bool,
    /// Set by `drain_output`/`drain_output_rgb` the moment a dequeued
    /// output buffer's flags report `BUFFER_FLAG_END_OF_STREAM` — the
    /// real "fully drained" signal `drain_until_eos` waits for, rather
    /// than inferring it from "dequeue returned try-again-later," which
    /// can happen transiently for reasons that aren't actually EOS.
    last_output_was_eos: bool,
}

// SAFETY: `AMediaCodec*` is documented (NdkMediaCodec.h) as safe to use
// from a single thread at a time with no concurrent access from
// others — this type is used that way throughout (one `HwCodec` owned
// by one dedicated encode/decode thread, matching this codebase's
// existing pattern for the ffmpeg subprocess `hw` module and the
// live-call capture/decode threads), so `Send` (moving it once, to the
// thread that will own it) is sound while `Sync` deliberately isn't
// implemented — nothing in this codebase needs to share one `HwCodec`
// across threads concurrently.
unsafe impl Send for HwCodec {}

impl Drop for HwCodec {
    fn drop(&mut self) {
        unsafe {
            ndk_sys::AMediaCodec_stop(self.raw);
            ndk_sys::AMediaCodec_delete(self.raw);
        }
    }
}

impl HwCodec {
    /// Real capability probe (not a hardcoded assumption) — actually
    /// configures and starts a decoder for the codec in question. Merely
    /// creating a codec handle is not enough: some Android vendors return
    /// a handle and then reject the requested resolution/color format.
    ///
    /// Decode, not encode: what a peer needs to know about *this*
    /// device before deciding it's safe to send it a given codec is
    /// whether this device can **decode** that codec — encode capability
    /// is a completely separate question. (See `negotiate_send_codec`'s
    /// doc for the bug this fixes: an earlier version of this
    /// negotiation used encode capability as a stand-in for "can accept
    /// this codec," which is backwards, and would have been wrong most
    /// concretely for AV1, where hardware *decode* is meaningfully more
    /// common than hardware *encode* on today's Android devices — a
    /// device that can only decode AV1, not encode it, still benefits
    /// from being told it's safe to receive AV1.)
    pub fn available_hw_decode_codecs() -> Vec<VideoCodec> {
        [VideoCodec::H265, VideoCodec::H264, VideoCodec::Av1]
            .into_iter()
            .filter(|&codec| Self::new_decoder(codec, 640, 360).is_ok())
            .collect()
    }

    /// Same probe, encoder side — used only locally (never put on the
    /// wire, see `negotiate_send_codec`'s doc) to decide what this
    /// device is actually capable of producing when it's this device's
    /// turn to pick an encode codec for what it sends.
    pub fn available_hw_encode_codecs() -> Vec<VideoCodec> {
        [VideoCodec::H265, VideoCodec::H264, VideoCodec::Av1]
            .into_iter()
            .filter(|&codec| Self::new_encoder(codec, 640, 360, 500_000, 15).is_ok())
            .collect()
    }

    pub fn new_encoder(
        codec: VideoCodec,
        width: i32,
        height: i32,
        bitrate: i32,
        fps: i32,
    ) -> Result<Self> {
        let mime = CString::new(mime_for(codec)).context("codec mime type")?;
        let raw = unsafe { ndk_sys::AMediaCodec_createEncoderByType(mime.as_ptr()) };
        if raw.is_null() {
            bail!("mediacodec: no hardware encoder available for {:?}", codec);
        }
        let format = MediaFormat::video(&mime, width, height, Some(bitrate), Some(fps))?;
        let status = unsafe {
            ndk_sys::AMediaCodec_configure(
                raw,
                format.raw,
                ptr::null_mut(),
                ptr::null_mut(),
                CONFIGURE_FLAG_ENCODE,
            )
        };
        if status != ndk_sys::media_status_t::AMEDIA_OK {
            unsafe { ndk_sys::AMediaCodec_delete(raw) };
            bail!(
                "mediacodec: configure (encoder) failed, status {:?}",
                status
            );
        }
        let status = unsafe { ndk_sys::AMediaCodec_start(raw) };
        if status != ndk_sys::media_status_t::AMEDIA_OK {
            unsafe { ndk_sys::AMediaCodec_delete(raw) };
            bail!("mediacodec: start (encoder) failed, status {:?}", status);
        }
        Ok(Self {
            raw,
            encoding: true,
            last_output_was_eos: false,
        })
    }

    pub fn new_decoder(codec: VideoCodec, width: i32, height: i32) -> Result<Self> {
        let mime = CString::new(mime_for(codec)).context("codec mime type")?;
        let raw = unsafe { ndk_sys::AMediaCodec_createDecoderByType(mime.as_ptr()) };
        if raw.is_null() {
            bail!("mediacodec: no hardware decoder available for {:?}", codec);
        }
        let format = MediaFormat::video(&mime, width, height, None, None)?;
        let status = unsafe {
            ndk_sys::AMediaCodec_configure(raw, format.raw, ptr::null_mut(), ptr::null_mut(), 0)
        };
        if status != ndk_sys::media_status_t::AMEDIA_OK {
            unsafe { ndk_sys::AMediaCodec_delete(raw) };
            bail!(
                "mediacodec: configure (decoder) failed, status {:?}",
                status
            );
        }
        let status = unsafe { ndk_sys::AMediaCodec_start(raw) };
        if status != ndk_sys::media_status_t::AMEDIA_OK {
            unsafe { ndk_sys::AMediaCodec_delete(raw) };
            bail!("mediacodec: start (decoder) failed, status {:?}", status);
        }
        Ok(Self {
            raw,
            encoding: false,
            last_output_was_eos: false,
        })
    }

    /// Feeds one RGB frame in (already resized to the encoder's
    /// configured width/height by the caller — see `resize_to_target`
    /// in `video.rs`, same requirement the AV1 path already has),
    /// converting to I420 first since `AMediaCodec`'s buffer-mode input
    /// doesn't accept RGB directly. Returns zero or more encoded NAL
    /// unit blobs — usually one, but codec-config data (SPS/PPS) can
    /// arrive as a separate buffer before the first real frame does.
    pub fn encode_frame(&mut self, rgb: &[u8], width: i32, height: i32) -> Vec<Vec<u8>> {
        debug_assert!(self.encoding, "encode_frame called on a decoder HwCodec");
        let i420 = rgb_to_i420(rgb, width as usize, height as usize);
        self.feed_input(&i420, 0);
        self.drain_output()
    }

    /// Feeds one encoded NAL unit blob in, returns zero or more decoded
    /// RGB frames out (converted back from the I420 `AMediaCodec`
    /// itself produces — see this module's doc for why this app's
    /// existing JPEG-per-frame display pipeline needs RGB rather than a
    /// surface handoff).
    pub fn decode_packet(&mut self, data: &[u8], width: i32, height: i32) -> Vec<Vec<u8>> {
        debug_assert!(!self.encoding, "decode_packet called on an encoder HwCodec");
        self.feed_input(data, 0);
        self.drain_output_rgb(width as usize, height as usize)
    }

    /// Signals end-of-stream and blocks (briefly — see `EOS_DRAIN_LIMIT`)
    /// until the codec actually confirms it's done, returning whatever
    /// final encoded blobs were still buffered inside it.
    ///
    /// This exists to fix a real correctness gap, not a hypothetical
    /// one: hardware encoders commonly hold back several frames in an
    /// internal reorder/lookahead buffer (this is *why* B-frames work at
    /// all) and won't emit them just because input stopped arriving —
    /// only an explicit end-of-stream signal tells the codec "no more
    /// input is coming, flush what you're holding." A clip encoded via
    /// `encode_frame` alone, with no call to this afterward, would
    /// silently lose however many frames the codec was still holding
    /// onto — not a crash, just quietly missing content, the kind of bug
    /// that's easy to not notice on a short test clip and only shows up
    /// as "the end of every status video looks cut off."
    pub fn finish_encoding(&mut self) -> Vec<Vec<u8>> {
        debug_assert!(self.encoding, "finish_encoding called on a decoder HwCodec");
        self.feed_input(&[], BUFFER_FLAG_END_OF_STREAM);
        self.drain_until_eos(Self::drain_output)
    }

    /// Decoder counterpart to `finish_encoding` — same reasoning, a
    /// decoder can also hold frames back for reordering.
    pub fn finish_decoding(&mut self, width: i32, height: i32) -> Vec<Vec<u8>> {
        debug_assert!(
            !self.encoding,
            "finish_decoding called on an encoder HwCodec"
        );
        self.feed_input(&[], BUFFER_FLAG_END_OF_STREAM);
        self.drain_until_eos(|s| s.drain_output_rgb(width as usize, height as usize))
    }

    /// Repeatedly drains (via whichever of `drain_output`/
    /// `drain_output_rgb` the caller passes) until the codec reports
    /// `BUFFER_FLAG_END_OF_STREAM` on an output buffer — the actual
    /// "I'm fully drained" signal — or `EOS_DRAIN_LIMIT` iterations pass
    /// without one, whichever comes first. The limit exists so a codec
    /// that never reports EOS for some reason (shouldn't happen, but
    /// this is still hardware/vendor driver code, not something to
    /// trust unconditionally) can't hang the calling thread forever —
    /// bounded at roughly `EOS_DRAIN_LIMIT * DEQUEUE_TIMEOUT_US`, a few
    /// hundred milliseconds, which is generous next to how long a real
    /// flush actually takes.
    fn drain_until_eos(
        &mut self,
        mut drain: impl FnMut(&mut Self) -> Vec<Vec<u8>>,
    ) -> Vec<Vec<u8>> {
        const EOS_DRAIN_LIMIT: u32 = 50;
        let mut collected = Vec::new();
        for _ in 0..EOS_DRAIN_LIMIT {
            if self.last_output_was_eos {
                break;
            }
            collected.extend(drain(self));
        }
        if !self.last_output_was_eos {
            tracing::debug!("mediacodec: gave up waiting for end-of-stream confirmation, using what drained so far");
        }
        collected
    }

    fn feed_input(&mut self, data: &[u8], flags: u32) {
        let idx = unsafe { ndk_sys::AMediaCodec_dequeueInputBuffer(self.raw, DEQUEUE_TIMEOUT_US) };
        if idx < 0 {
            tracing::debug!("mediacodec: no input buffer available this cycle, dropping one frame");
            return;
        }
        let mut out_size: usize = 0;
        let buf =
            unsafe { ndk_sys::AMediaCodec_getInputBuffer(self.raw, idx as usize, &mut out_size) };
        if buf.is_null() {
            return;
        }
        let n = data.len().min(out_size);
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), buf, n);
            ndk_sys::AMediaCodec_queueInputBuffer(self.raw, idx as usize, 0, n, 0, flags);
        }
    }

    /// Drains every output buffer currently ready (encoder path) —
    /// keeps calling `dequeueOutputBuffer` until it reports "nothing
    /// ready" rather than stopping after one, since one `feed_input`
    /// call can free up more than one pending output buffer.
    fn drain_output(&mut self) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        loop {
            let mut info = ndk_sys::AMediaCodecBufferInfo {
                offset: 0,
                size: 0,
                presentationTimeUs: 0,
                flags: 0,
            };
            let idx = unsafe {
                ndk_sys::AMediaCodec_dequeueOutputBuffer(self.raw, &mut info, DEQUEUE_TIMEOUT_US)
            };
            match idx {
                i if i == INFO_TRY_AGAIN_LATER => break,
                i if i == INFO_OUTPUT_FORMAT_CHANGED || i == INFO_OUTPUT_BUFFERS_CHANGED => {
                    continue
                }
                i if i < 0 => break, // unexpected negative status — nothing more to drain safely this cycle
                i => {
                    let mut out_size: usize = 0;
                    let buf = unsafe {
                        ndk_sys::AMediaCodec_getOutputBuffer(self.raw, i as usize, &mut out_size)
                    };
                    let is_eos = (info.flags & BUFFER_FLAG_END_OF_STREAM) != 0;
                    if is_eos {
                        self.last_output_was_eos = true;
                    }
                    if !buf.is_null() && !is_eos {
                        let len =
                            (info.size as usize).min(out_size.saturating_sub(info.offset as usize));
                        let slice = unsafe {
                            std::slice::from_raw_parts(buf.add(info.offset as usize), len)
                        };
                        if (info.flags & BUFFER_FLAG_CODEC_CONFIG) != 0 || len > 0 {
                            out.push(slice.to_vec());
                        }
                    }
                    unsafe {
                        ndk_sys::AMediaCodec_releaseOutputBuffer(self.raw, i as usize, false);
                    }
                    if is_eos {
                        break;
                    }
                }
            }
        }
        out
    }

    /// Same drain loop as `drain_output`, but converts each output
    /// buffer from I420 to an RGB byte buffer instead of returning the
    /// raw bytes — the decoder path's output is picture data, not
    /// bitstream, so "raw bytes" wouldn't mean anything to a caller the
    /// way it does for `drain_output`'s encoded NAL units.
    fn drain_output_rgb(&mut self, width: usize, height: usize) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        loop {
            let mut info = ndk_sys::AMediaCodecBufferInfo {
                offset: 0,
                size: 0,
                presentationTimeUs: 0,
                flags: 0,
            };
            let idx = unsafe {
                ndk_sys::AMediaCodec_dequeueOutputBuffer(self.raw, &mut info, DEQUEUE_TIMEOUT_US)
            };
            match idx {
                i if i == INFO_TRY_AGAIN_LATER => break,
                i if i == INFO_OUTPUT_FORMAT_CHANGED || i == INFO_OUTPUT_BUFFERS_CHANGED => {
                    continue
                }
                i if i < 0 => break,
                i => {
                    let mut out_size: usize = 0;
                    let buf = unsafe {
                        ndk_sys::AMediaCodec_getOutputBuffer(self.raw, i as usize, &mut out_size)
                    };
                    let is_eos = (info.flags & BUFFER_FLAG_END_OF_STREAM) != 0;
                    if is_eos {
                        self.last_output_was_eos = true;
                    }
                    if !buf.is_null() && !is_eos {
                        let len =
                            (info.size as usize).min(out_size.saturating_sub(info.offset as usize));
                        let slice = unsafe {
                            std::slice::from_raw_parts(buf.add(info.offset as usize), len)
                        };
                        if len >= width * height * 3 / 2 {
                            out.push(i420_to_rgb(slice, width, height));
                        }
                    }
                    unsafe {
                        ndk_sys::AMediaCodec_releaseOutputBuffer(self.raw, i as usize, false);
                    }
                    if is_eos {
                        break;
                    }
                }
            }
        }
        out
    }
}

/// Thin RAII wrapper over `AMediaFormat*`, just so `HwCodec::new_encoder`/
/// `new_decoder` don't need their own manual delete-on-every-error-path
/// bookkeeping for it.
struct MediaFormat {
    raw: *mut ndk_sys::AMediaFormat,
}

impl Drop for MediaFormat {
    fn drop(&mut self) {
        unsafe {
            ndk_sys::AMediaFormat_delete(self.raw);
        }
    }
}

impl MediaFormat {
    fn video(
        mime: &CString,
        width: i32,
        height: i32,
        bitrate: Option<i32>,
        fps: Option<i32>,
    ) -> Result<Self> {
        let raw = unsafe { ndk_sys::AMediaFormat_new() };
        if raw.is_null() {
            bail!("mediacodec: AMediaFormat_new failed");
        }
        unsafe {
            let key_mime = CString::new("mime").unwrap();
            let key_width = CString::new("width").unwrap();
            let key_height = CString::new("height").unwrap();
            let key_color_format = CString::new("color-format").unwrap();
            ndk_sys::AMediaFormat_setString(raw, key_mime.as_ptr(), mime.as_ptr());
            ndk_sys::AMediaFormat_setInt32(raw, key_width.as_ptr(), width);
            ndk_sys::AMediaFormat_setInt32(raw, key_height.as_ptr(), height);
            ndk_sys::AMediaFormat_setInt32(
                raw,
                key_color_format.as_ptr(),
                COLOR_FORMAT_YUV420_FLEXIBLE,
            );
            if let Some(bps) = bitrate {
                let key = CString::new("bitrate").unwrap();
                ndk_sys::AMediaFormat_setInt32(raw, key.as_ptr(), bps);
            }
            if let Some(fps) = fps {
                let key_fps = CString::new("frame-rate").unwrap();
                let key_i_frame = CString::new("i-frame-interval").unwrap();
                ndk_sys::AMediaFormat_setInt32(raw, key_fps.as_ptr(), fps);
                ndk_sys::AMediaFormat_setInt32(raw, key_i_frame.as_ptr(), 1); // one keyframe/sec — same cadence the AV1 encoder path already uses ("-g 30" at ~30fps in the ffmpeg hw module)
            }
        }
        Ok(Self { raw })
    }
}

/// RGB24 → I420 (planar Y, then U, then V, no padding between planes —
/// matches what `AMediaFormat`'s `COLOR_FormatYUV420Flexible` output is
/// documented to accept for buffer-mode input). Standard BT.601
/// coefficients, same family of math `i420_picture_to_jpeg` (this
/// file's existing AV1/dav1d decode path) already uses in the other
/// direction — not new math, the inverse of already-working math.
pub(crate) fn rgb_to_i420(rgb: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = vec![0u8; width * height * 3 / 2];
    let (y_plane, uv_rest) = out.split_at_mut(width * height);
    let (u_plane, v_plane) = uv_rest.split_at_mut(width * height / 4);

    for row in 0..height {
        for col in 0..width {
            let o = (row * width + col) * 3;
            if o + 2 >= rgb.len() {
                break;
            }
            let r = rgb[o] as i32;
            let g = rgb[o + 1] as i32;
            let b = rgb[o + 2] as i32;

            let yv = (306 * r + 601 * g + 117 * b) >> 10;
            y_plane[row * width + col] = yv.clamp(0, 255) as u8;

            if row % 2 == 0 && col % 2 == 0 {
                let u = ((-173 * r - 339 * g + 512 * b + 131072) >> 10).clamp(0, 255);
                let v = ((512 * r - 429 * g - 83 * b + 131072) >> 10).clamp(0, 255);
                let ci = (row / 2) * (width / 2) + col / 2;
                if ci < u_plane.len() && ci < v_plane.len() {
                    u_plane[ci] = u as u8;
                    v_plane[ci] = v as u8;
                }
            }
        }
    }
    out
}

pub(crate) fn i420_to_rgb(i420: &[u8], width: usize, height: usize) -> Vec<u8> {
    let expected_len = width * height * 3 / 2;
    if i420.len() < expected_len {
        return vec![0u8; width * height * 3];
    }
    let (y_plane, uv_rest) = i420.split_at(width * height);
    let (u_plane, v_plane) = uv_rest.split_at(width * height / 4);
    let mut rgb = vec![0u8; width * height * 3];

    for row in 0..height {
        for col in 0..width {
            let yv = y_plane[row * width + col] as i32;
            let ci = (row / 2) * (width / 2) + col / 2;
            let u = u_plane[ci] as i32 - 128;
            let v = v_plane[ci] as i32 - 128;

            let r = (yv + ((1436 * v) >> 10)).clamp(0, 255);
            let g = (yv - ((352 * u + 731 * v) >> 10)).clamp(0, 255);
            let b = (yv + ((1815 * u) >> 10)).clamp(0, 255);

            let o = (row * width + col) * 3;
            rgb[o] = r as u8;
            rgb[o + 1] = g as u8;
            rgb[o + 2] = b as u8;
        }
    }
    rgb
}

/// Real capability probe, exposed for `net::calls::mod.rs`'s
/// `place_call`/`accept` to call directly — specifically the *decode*
/// side (see `available_hw_decode_codecs`'s doc for why that's the
/// right one to put on the wire, not encode). This whole file only
/// exists in the build at all on Android (see the `#[cfg(target_os =
/// "android")] mod mediacodec;` at its call site) — the non-Android
/// fallback that returns an empty list lives there, not here, so this
/// file doesn't need its own redundant `#[cfg]` split.
pub fn available_hw_decode_codecs() -> Vec<VideoCodec> {
    static CODECS: std::sync::OnceLock<Vec<VideoCodec>> = std::sync::OnceLock::new();
    CODECS
        .get_or_init(HwCodec::available_hw_decode_codecs)
        .clone()
}

/// Same, encode side — used locally only (see `negotiate_send_codec`'s
/// doc), never put on the wire.
pub fn available_hw_encode_codecs() -> Vec<VideoCodec> {
    static CODECS: std::sync::OnceLock<Vec<VideoCodec>> = std::sync::OnceLock::new();
    CODECS
        .get_or_init(HwCodec::available_hw_encode_codecs)
        .clone()
}
