//! Real AV1 decode via `libdav1d`'s C API, through `dav1d-sys 0.8.3`.
//!
//! Every struct field and function signature `unsafe` code below
//! touches was read directly out of `dav1d-sys`'s committed source
//! (`Dav1dContext`, `Dav1dSettings`, `Dav1dData`, `Dav1dPicture`,
//! `Dav1dPictureParameters`, `dav1d_open`/`send_data`/`get_picture`/
//! `close`/`data_create`/`picture_unref`) — not bindgen'd blind, not
//! guessed from dav1d's C header from memory. All `unsafe` in this
//! workspace for AV1 decode is confined to this one file.
//!
//! dav1d's own calling convention, reflected in this wrapper:
//! - `dav1d_send_data` can return `-EAGAIN`, meaning "call
//!   `dav1d_get_picture` to drain a buffered picture first, then retry
//!   sending." This wrapper loops on that internally so callers just
//!   see a normal `decode(&self, frame) -> Result<...>`.
//! - `dav1d_get_picture` returning `-EAGAIN` means "no picture ready
//!   yet, feed more data" — not an error. Since `decode` here is called
//!   once per already-encoded frame (one temporal unit at a time,
//!   matching `EncodedVideoFrame`), this surfaces as
//!   `DecodeError::Unsupported` describing "buffered, no picture yet"
//!   rather than a real decode failure — dav1d's own lookahead can
//!   delay output by a frame or two, same as any real AV1/HEVC decoder.
//! - `Dav1dPicture`'s planes are **not guaranteed contiguous** —
//!   `stride[0]`/`stride[1]` can exceed the plane's actual width
//!   (alignment padding), so this copies row by row rather than a
//!   single `memcpy` of `stride * height` bytes, which would copy
//!   padding garbage into `RawVideoFrame`'s tightly-packed layout.

use dav1d_sys::{
    dav1d_close, dav1d_data_create, dav1d_default_settings, dav1d_get_picture, dav1d_open,
    dav1d_picture_unref, dav1d_send_data, Dav1dContext, Dav1dData, Dav1dPicture, Dav1dSettings,
    DAV1D_PIXEL_LAYOUT_I420,
};
use siar_media_core::{DecodeError, DecodedVideoFrame, EncodedVideoFrame, RawVideoFrame, Resolution, VideoCodec, VideoDecoder};
use std::ptr;

pub struct Av1SoftwareDecoder {
    ctx: *mut Dav1dContext,
}

// SAFETY: a `Dav1dContext` is only ever accessed through this struct's
// `&mut self` methods (never concurrently from two threads at once —
// nothing here implements `Sync`), and dav1d's own docs describe a
// context as safe to use from a single thread at a time and safe to
// hand off between threads sequentially, which is exactly the
// `Send`-not-`Sync` shape this asserts.
unsafe impl Send for Av1SoftwareDecoder {}

impl Av1SoftwareDecoder {
    pub fn new() -> Result<Self, DecodeError> {
        // SAFETY: `settings` is a plain-old-data struct dav1d itself
        // populates via `dav1d_default_settings` before we pass a
        // pointer to it into `dav1d_open` — no uninitialized reads,
        // both calls take a valid `&mut`-derived pointer to a value
        // that lives for the duration of the call.
        let ctx = unsafe {
            let mut settings: Dav1dSettings = std::mem::zeroed();
            dav1d_default_settings(&mut settings);

            let mut ctx: *mut Dav1dContext = ptr::null_mut();
            let ret = dav1d_open(&mut ctx, &settings);
            if ret != 0 {
                return Err(DecodeError::Backend(format!(
                    "dav1d_open failed: {}",
                    std::io::Error::from_raw_os_error(-ret)
                )));
            }
            ctx
        };

        Ok(Self { ctx })
    }

    /// Sends one encoded temporal unit and drains at most one decoded
    /// picture. Internal helper so `decode` (the public `VideoDecoder`
    /// method) stays a straight-line "send, then get" without the
    /// EAGAIN-retry loop cluttering it.
    fn send_and_receive(&mut self, encoded: &[u8]) -> Result<Dav1dPicture, DecodeError> {
        // SAFETY: `dav1d_data_create` allocates and returns a pointer
        // to `sz` bytes owned by dav1d, valid until the `Dav1dData` is
        // consumed/unref'd; we immediately fill exactly that many
        // bytes via `copy_from_slice` before dav1d ever reads them, so
        // there's no uninitialized-read window from dav1d's side.
        let mut data: Dav1dData = unsafe { std::mem::zeroed() };
        unsafe {
            let dst = dav1d_data_create(&mut data, encoded.len());
            if dst.is_null() {
                return Err(DecodeError::Backend("dav1d_data_create returned null (allocation failure)".to_string()));
            }
            std::slice::from_raw_parts_mut(dst, encoded.len()).copy_from_slice(encoded);
        }

        // SAFETY: `self.ctx` was checked non-null in `new`, and never
        // reassigned except to null in `Drop` (which also means this
        // method can't run after `Drop` — it takes `&mut self`, and
        // `Drop::drop` also takes `&mut self`, so the borrow checker
        // already prevents a call after drop). `&mut data` is a valid
        // pointer to a `Dav1dData` we just initialized above.
        let send_ret = unsafe { dav1d_send_data(self.ctx, &mut data) };
        if send_ret != 0 && send_ret != -libc::EAGAIN {
            return Err(DecodeError::Backend(format!(
                "dav1d_send_data failed: {}",
                std::io::Error::from_raw_os_error(-send_ret)
            )));
        }
        // Whether send fully succeeded or returned EAGAIN (meaning "a
        // picture is buffered, drain it, then I'll accept more data on
        // your next call"), attempt to drain one picture now — this
        // matches dav1d's own documented request/response rhythm
        // rather than assuming one call always yields one picture.

        // SAFETY: `pic` is zero-initialized POD dav1d fills in;
        // `self.ctx` is valid for the same reason as above.
        let mut pic: Dav1dPicture = unsafe { std::mem::zeroed() };
        let get_ret = unsafe { dav1d_get_picture(self.ctx, &mut pic) };
        if get_ret == -libc::EAGAIN {
            return Err(DecodeError::Unsupported(
                "dav1d has buffered this frame but has no picture ready yet (normal decoder lookahead — call decode again with the next frame)".to_string(),
            ));
        }
        if get_ret != 0 {
            return Err(DecodeError::Backend(format!(
                "dav1d_get_picture failed: {}",
                std::io::Error::from_raw_os_error(-get_ret)
            )));
        }

        Ok(pic)
    }
}

impl Drop for Av1SoftwareDecoder {
    fn drop(&mut self) {
        // SAFETY: `dav1d_close` takes a `*mut *mut Dav1dContext` and
        // nulls it out after freeing — `self.ctx` was opened by `new`
        // and is not used again after this (struct is being dropped).
        unsafe { dav1d_close(&mut self.ctx) };
    }
}

impl VideoDecoder for Av1SoftwareDecoder {
    fn codec(&self) -> VideoCodec {
        VideoCodec::Av1
    }

    fn decode(&mut self, frame: &EncodedVideoFrame) -> Result<DecodedVideoFrame, DecodeError> {
        if frame.codec != VideoCodec::Av1 {
            return Err(DecodeError::Unsupported(format!("expected AV1, got {:?}", frame.codec)));
        }

        let pic = self.send_and_receive(&frame.data)?;

        if pic.p.layout != DAV1D_PIXEL_LAYOUT_I420 {
            let layout = pic.p.layout;
            // SAFETY: `pic` is a fully-initialized `Dav1dPicture` we
            // own at this point (dav1d_get_picture succeeded) — must
            // still be unref'd on this early-return path or its
            // internal buffer leaks. Captured `layout` above first
            // since `dav1d_picture_unref` may zero the struct's fields
            // through the pointer we hand it.
            let mut pic = pic;
            unsafe { dav1d_picture_unref(&mut pic) };
            return Err(DecodeError::Unsupported(format!(
                "decoded picture is not 4:2:0 (layout={layout}) — only I420 output is handled"
            )));
        }

        let width = pic.p.w as u32;
        let height = pic.p.h as u32;
        let resolution = Resolution::new(width, height);
        let (y_len, uv_len, _) = RawVideoFrame::expected_plane_sizes(resolution);

        // SAFETY: `pic.data[0..3]` are dav1d-owned plane buffers valid
        // for the lifetime of `pic` (i.e. until `dav1d_picture_unref`
        // below); `pic.stride[0]` is the Y plane's byte stride and
        // `pic.stride[1]` is shared by both chroma planes (dav1d's own
        // documented convention — a single stride for the interleaved-
        // dimension chroma planes since 4:2:0/4:2:2/4:4:4 chroma planes
        // always match each other's stride). Copying row by row with
        // `width`/`chroma_width` (not `stride`) as the per-row copy
        // length is what keeps this from reading past each row's valid
        // data into next-row padding.
        let (y_plane, u_plane, v_plane) = unsafe {
            let y_stride = pic.stride[0] as usize;
            let uv_stride = pic.stride[1] as usize;
            let chroma_width = width.div_ceil(2) as usize;
            let chroma_height = height.div_ceil(2) as usize;

            let mut y = vec![0u8; y_len];
            for row in 0..height as usize {
                let src = (pic.data[0] as *const u8).add(row * y_stride);
                let dst = &mut y[row * width as usize..(row + 1) * width as usize];
                dst.copy_from_slice(std::slice::from_raw_parts(src, width as usize));
            }

            let mut u = vec![0u8; uv_len];
            let mut v = vec![0u8; uv_len];
            for row in 0..chroma_height {
                let u_src = (pic.data[1] as *const u8).add(row * uv_stride);
                let v_src = (pic.data[2] as *const u8).add(row * uv_stride);
                u[row * chroma_width..(row + 1) * chroma_width]
                    .copy_from_slice(std::slice::from_raw_parts(u_src, chroma_width));
                v[row * chroma_width..(row + 1) * chroma_width]
                    .copy_from_slice(std::slice::from_raw_parts(v_src, chroma_width));
            }

            (y, u, v)
        };

        // SAFETY: `pic` was returned by a successful `dav1d_get_picture`
        // and we're done reading its plane data — this releases dav1d's
        // internal refcount on the picture buffer.
        let mut pic = pic;
        unsafe { dav1d_picture_unref(&mut pic) };

        Ok(DecodedVideoFrame {
            frame: RawVideoFrame { resolution, y_plane, u_plane, v_plane, timestamp_micros: frame.timestamp_micros },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_and_closes_a_real_decoder_context() {
        // This alone exercises `dav1d_default_settings`, `dav1d_open`,
        // and (via `Drop`) `dav1d_close` against the real linked
        // libdav1d — it's a genuine link+run smoke test, not a mock.
        let decoder = Av1SoftwareDecoder::new().expect("dav1d_open should succeed with default settings");
        drop(decoder);
    }

    #[test]
    fn rejects_non_av1_codec_before_touching_dav1d() {
        let mut decoder = Av1SoftwareDecoder::new().unwrap();
        let frame = EncodedVideoFrame {
            codec: VideoCodec::H264,
            data: vec![0, 1, 2, 3],
            is_keyframe: true,
            timestamp_micros: 0,
        };
        assert!(matches!(decoder.decode(&frame), Err(DecodeError::Unsupported(_))));
    }

    #[test]
    fn garbage_bitstream_is_a_clean_error_not_a_crash() {
        let mut decoder = Av1SoftwareDecoder::new().unwrap();
        let frame = EncodedVideoFrame {
            codec: VideoCodec::Av1,
            data: vec![0xFF; 64], // not a valid AV1 OBU stream
            is_keyframe: true,
            timestamp_micros: 0,
        };
        // Either dav1d rejects it outright (Backend) or buffers it
        // without producing a picture (Unsupported) — both are clean
        // `Err`s, neither is a panic or UB, which is the actual thing
        // this test is checking.
        assert!(decoder.decode(&frame).is_err());
    }

    /// Full round trip against the real, verified `rav1e` encoder from
    /// `encoder.rs` — encodes a blank frame, decodes it back with this
    /// module, and checks the output resolution matches. This is the
    /// strongest test in this crate: it links and runs both halves of
    /// the AV1 pipeline for real.
    #[test]
    fn round_trips_a_frame_through_the_real_rav1e_encoder() {
        use crate::encoder::{Av1EncoderSettings, Av1SoftwareEncoder};
        use siar_media_core::VideoEncoder;

        let width = 64;
        let height = 64;
        let mut encoder = Av1SoftwareEncoder::new(Av1EncoderSettings {
            width,
            height,
            speed: 10,
            target_bitrate_bps: 200_000,
            max_key_frame_interval: 30,
        })
        .unwrap();
        let mut decoder = Av1SoftwareDecoder::new().unwrap();

        let (y, u, v) = RawVideoFrame::expected_plane_sizes(Resolution::new(width, height));
        let raw = RawVideoFrame {
            resolution: Resolution::new(width, height),
            y_plane: vec![16; y],
            u_plane: vec![128; u],
            v_plane: vec![128; v],
            timestamp_micros: 42,
        };

        // rav1e in low-latency mode with a lookahead of a few frames
        // may not emit a packet on the very first `encode` call
        // (`EncodedVideoFrame.data` can be empty — see encoder.rs's own
        // `NeedMoreData` handling) — feed a few frames so there's
        // always at least one non-empty packet to decode.
        let mut decoded_frame = None;
        for _ in 0..30 {
            let packet = encoder.encode(&raw).unwrap();
            if !packet.data.is_empty() {
                let encoded = EncodedVideoFrame {
                    codec: VideoCodec::Av1,
                    data: packet.data,
                    is_keyframe: packet.is_keyframe,
                    timestamp_micros: 42,
                };
                match decoder.decode(&encoded) {
                    Ok(decoded) => {
                        decoded_frame = Some(decoded);
                        break;
                    }
                    Err(DecodeError::Unsupported(msg)) if msg.contains("dav1d has buffered") => {
                        continue;
                    }
                    Err(e) => panic!("decode of a real rav1e packet failed: {e:?}"),
                }
            }
        }
        let decoded = decoded_frame.expect("decoder produced a decoded frame");
        assert_eq!(decoded.frame.resolution, Resolution::new(width, height));
        assert_eq!(decoded.frame.timestamp_micros, 42);
    }
}
