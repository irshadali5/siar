//! Software AV1 encode via `rav1e` — verified against `rav1e 0.8.1`'s
//! actual source (not just its docs): `Config::new()
//! .with_encoder_config(..).new_context::<u8>()`, `ctx.new_frame()`,
//! `frame.planes[i].copy_from_raw_u8(bytes, stride, bytewidth)`,
//! `ctx.send_frame(frame)`, `ctx.receive_packet()` returning
//! `Packet { data, frame_type, .. }`. All pure-Rust, no `unsafe` written
//! by this crate.
//!
//! One field's unit is not 100% confirmed by the doc comment alone:
//! `EncoderConfig::bitrate` (`i32`) is documented only as "the target
//! bitrate for the bitrate mode" without stating units in the struct
//! doc. Every external reference for rav1e (its own CLI's `--bitrate`
//! flag, which multiplies by 1000 before assigning this field) points
//! to bits/second, and that's what this module assumes — but this is
//! the one place in this file that's inference rather than something
//! read directly off a type signature. Worth a quick sanity check
//! (encode a few seconds at a known bitrate, measure output size) on
//! real hardware before trusting the target-bitrate numbers exactly.

use rav1e::prelude::{ChromaSampling, Config, Context, EncoderConfig, FrameType};
use siar_media_core::{EncodeError, RawVideoFrame, VideoCodec, VideoEncoder};

pub struct Av1SoftwareEncoder {
    context: Context<u8>,
    width: usize,
    height: usize,
}

pub struct Av1EncoderSettings {
    pub width: u32,
    pub height: u32,
    /// 0 (slowest/best quality) – 10 (fastest). rav1e's own
    /// `with_speed_preset` clamps anything above 10 to 10.
    pub speed: u8,
    pub target_bitrate_bps: i32,
    pub max_key_frame_interval: u64,
}

impl Av1SoftwareEncoder {
    pub fn new(settings: Av1EncoderSettings) -> Result<Self, EncodeError> {
        if settings.width < 16 || settings.height < 16 {
            // rav1e's own `InvalidConfig::InvalidWidth`/`InvalidHeight`
            // reject below 16 — checking here first gives a clearer
            // error than whatever `new_context` would report.
            return Err(EncodeError::Unsupported(format!(
                "AV1 requires width and height >= 16, got {}x{}",
                settings.width, settings.height
            )));
        }

        let mut enc_cfg = EncoderConfig::with_speed_preset(settings.speed);
        enc_cfg.width = settings.width as usize;
        enc_cfg.height = settings.height as usize;
        enc_cfg.bit_depth = 8;
        enc_cfg.chroma_sampling = ChromaSampling::Cs420;
        enc_cfg.bitrate = settings.target_bitrate_bps;
        enc_cfg.max_key_frame_interval = settings.max_key_frame_interval.max(1);
        // Realtime calling, not offline transcoding: don't reorder
        // frames waiting for a better encode decision (architecture
        // doc §48's "call quality controller" needs predictable,
        // in-order output to react to).
        enc_cfg.low_latency = true;

        let cfg = Config::new().with_encoder_config(enc_cfg);
        let context: Context<u8> = cfg
            .new_context()
            .map_err(|e| EncodeError::Backend(format!("rav1e config rejected: {e}")))?;

        Ok(Self { context, width: settings.width as usize, height: settings.height as usize })
    }
}

impl VideoEncoder for Av1SoftwareEncoder {
    fn codec(&self) -> VideoCodec {
        VideoCodec::Av1
    }

    fn encode(&mut self, frame: &RawVideoFrame) -> Result<siar_media_core::EncodedVideoFrame, EncodeError> {
        if !frame.is_well_formed() {
            return Err(EncodeError::MalformedFrame);
        }
        if frame.resolution.width as usize != self.width || frame.resolution.height as usize != self.height {
            return Err(EncodeError::Unsupported(format!(
                "encoder configured for {}x{}, got {}x{}",
                self.width, self.height, frame.resolution.width, frame.resolution.height
            )));
        }

        let mut rav1e_frame = self.context.new_frame();
        // Plane 0 = Y (full resolution), planes 1/2 = U/V (halved in
        // both dimensions for 4:2:0) — `RawVideoFrame`'s own doc comment
        // on `expected_plane_sizes` describes the same layout, so these
        // strides match what `is_well_formed` already validated above.
        rav1e_frame.planes[0].copy_from_raw_u8(&frame.y_plane, self.width, 1);
        let chroma_stride = self.width.div_ceil(2);
        rav1e_frame.planes[1].copy_from_raw_u8(&frame.u_plane, chroma_stride, 1);
        rav1e_frame.planes[2].copy_from_raw_u8(&frame.v_plane, chroma_stride, 1);

        self.context
            .send_frame(rav1e_frame)
            .map_err(|e| EncodeError::Backend(format!("send_frame: {e}")))?;

        // rav1e can buffer several frames before it starts producing
        // packets (lookahead) even in low-latency mode's reduced form —
        // `EncoderStatus::NeedMoreData` is not a real error here, it
        // just means "no packet yet, feed more frames," which for a
        // realtime caller means: this call produced no output this
        // time, and that's fine, the caller keeps calling `encode` for
        // the next frame.
        loop {
            match self.context.receive_packet() {
                Ok(packet) => {
                    return Ok(siar_media_core::EncodedVideoFrame {
                        codec: VideoCodec::Av1,
                        data: packet.data,
                        is_keyframe: packet.frame_type == FrameType::KEY,
                        timestamp_micros: frame.timestamp_micros,
                    });
                }
                Err(rav1e::EncoderStatus::NeedMoreData) => {
                    // No packet ready yet for *this* call. Real-time
                    // callers feed frames one at a time and just accept
                    // an empty result here; this thin wrapper reports
                    // it as an empty (zero-byte) non-keyframe rather
                    // than an error, since "no output yet" isn't a
                    // failure — it's normal lookahead behavior.
                    return Ok(siar_media_core::EncodedVideoFrame {
                        codec: VideoCodec::Av1,
                        data: Vec::new(),
                        is_keyframe: false,
                        timestamp_micros: frame.timestamp_micros,
                    });
                }
                Err(rav1e::EncoderStatus::Encoded) => continue,
                Err(e) => return Err(EncodeError::Backend(format!("receive_packet: {e}"))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siar_media_core::capability::Resolution;

    fn blank_frame(width: u32, height: u32, timestamp_micros: u64) -> RawVideoFrame {
        let (y, u, v) = RawVideoFrame::expected_plane_sizes(Resolution::new(width, height));
        RawVideoFrame {
            resolution: Resolution::new(width, height),
            y_plane: vec![16; y],  // 16 = standard "black" in limited-range YUV
            u_plane: vec![128; u], // 128 = neutral chroma
            v_plane: vec![128; v],
            timestamp_micros,
        }
    }

    #[test]
    fn rejects_sub_16_dimensions_before_touching_rav1e() {
        let result = Av1SoftwareEncoder::new(Av1EncoderSettings {
            width: 8,
            height: 8,
            speed: 10,
            target_bitrate_bps: 500_000,
            max_key_frame_interval: 120,
        });
        assert!(matches!(result, Err(EncodeError::Unsupported(_))));
    }

    #[test]
    fn encoding_a_well_formed_frame_does_not_error() {
        // Speed 10 (fastest) keeps this test from being slow; still a
        // real rav1e encode of a real (blank) frame, not a mock.
        let mut encoder = Av1SoftwareEncoder::new(Av1EncoderSettings {
            width: 64,
            height: 64,
            speed: 10,
            target_bitrate_bps: 200_000,
            max_key_frame_interval: 30,
        })
        .unwrap();

        let frame = blank_frame(64, 64, 0);
        let result = encoder.encode(&frame);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn mismatched_resolution_is_rejected() {
        let mut encoder = Av1SoftwareEncoder::new(Av1EncoderSettings {
            width: 64,
            height: 64,
            speed: 10,
            target_bitrate_bps: 200_000,
            max_key_frame_interval: 30,
        })
        .unwrap();

        let frame = blank_frame(32, 32, 0);
        assert!(matches!(encoder.encode(&frame), Err(EncodeError::Unsupported(_))));
    }

    #[test]
    fn malformed_frame_is_rejected_before_reaching_rav1e() {
        let mut encoder = Av1SoftwareEncoder::new(Av1EncoderSettings {
            width: 64,
            height: 64,
            speed: 10,
            target_bitrate_bps: 200_000,
            max_key_frame_interval: 30,
        })
        .unwrap();

        let mut frame = blank_frame(64, 64, 0);
        frame.u_plane.pop();
        assert_eq!(encoder.encode(&frame), Err(EncodeError::MalformedFrame));
    }
}
