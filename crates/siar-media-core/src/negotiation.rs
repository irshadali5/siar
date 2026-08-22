use crate::capability::VideoCapabilities;
use crate::codec::{CodecImplementation, VideoCodec};

/// Architecture doc §7: "don't negotiate merely codec = AV1 — negotiate
/// Alice → Bob video and Bob → Alice video independently." A device can
/// have hardware decode without hardware encode for the same codec, so
/// the two directions of one call can legitimately end up on different
/// codecs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NegotiatedDirection {
    pub codec: VideoCodec,
    pub implementation: CodecImplementation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NegotiatedCall {
    /// What the offerer encodes and the answerer decodes.
    pub offerer_to_answerer: Option<NegotiatedDirection>,
    /// What the answerer encodes and the offerer decodes.
    pub answerer_to_offerer: Option<NegotiatedDirection>,
}

impl NegotiatedCall {
    /// `None` in both directions means no video is possible at all
    /// (architecture doc §5's "no usable AV1 encoder → audio-only call
    /// with desktop" case) — the caller checks this and falls back to
    /// audio-only rather than treating it as a negotiation error.
    pub fn is_audio_only(&self) -> bool {
        self.offerer_to_answerer.is_none() && self.answerer_to_offerer.is_none()
    }
}

/// Architecture doc §4's selection order, as an explicit priority list
/// rather than scattered if/else — this is the "policy scoring" step
/// that runs *after* intersecting local and remote capabilities.
/// Desktop's `VideoCapabilities::desktop_av1_software()` only ever lists
/// AV1, so this order only actually matters on Android, where multiple
/// codecs can be simultaneously viable.
pub const DEFAULT_ENCODE_PRIORITY: &[(VideoCodec, CodecImplementation)] = &[
    (VideoCodec::Av1, CodecImplementation::Hardware),
    (VideoCodec::H265, CodecImplementation::Hardware),
    (VideoCodec::H264, CodecImplementation::Hardware),
    (VideoCodec::Av1, CodecImplementation::Software),
];

/// Negotiates one direction: does `sender`'s encode capability list
/// contain something `receiver` can decode? Returns the
/// highest-priority match per `DEFAULT_ENCODE_PRIORITY`, or `None` if
/// there's no overlap at all (architecture doc §5's degradation case).
pub fn negotiate_direction(
    sender: &VideoCapabilities,
    receiver: &VideoCapabilities,
) -> Option<NegotiatedDirection> {
    DEFAULT_ENCODE_PRIORITY.iter().find_map(|&(codec, implementation)| {
        let sender_can_encode = sender
            .codecs
            .iter()
            .any(|c| c.codec == codec && c.implementation == implementation && c.can_encode);
        let receiver_can_decode = receiver
            .codecs
            .iter()
            .any(|c| c.codec == codec && c.can_decode);
        if sender_can_encode && receiver_can_decode {
            Some(NegotiatedDirection { codec, implementation })
        } else {
            None
        }
    })
}

/// Full call negotiation: both directions, independently (architecture
/// doc §7). `local`/`remote` are each one endpoint's full
/// `VideoCapabilities` — covering both their encode and decode support,
/// since one endpoint can encode in one codec and decode in another.
pub fn negotiate_call(local: &VideoCapabilities, remote: &VideoCapabilities) -> NegotiatedCall {
    NegotiatedCall {
        offerer_to_answerer: negotiate_direction(local, remote),
        answerer_to_offerer: negotiate_direction(remote, local),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{BitrateRange, FrameRateRange, Resolution, VideoCodecCapability};

    fn cap(codec: VideoCodec, implementation: CodecImplementation, encode: bool, decode: bool) -> VideoCodecCapability {
        VideoCodecCapability {
            codec,
            implementation,
            can_encode: encode,
            can_decode: decode,
            max_resolution: Resolution::new(1280, 720),
            max_fps: 30,
            supported_profiles: vec![],
            bitrate_range: BitrateRange { min_bps: 100_000, max_bps: 2_000_000 },
            frame_rate_range: FrameRateRange { min_fps: 15, max_fps: 30 },
        }
    }

    #[test]
    fn desktop_to_desktop_negotiates_av1_software_both_ways() {
        let desktop = VideoCapabilities::desktop_av1_software();
        let negotiated = negotiate_call(&desktop, &desktop);
        assert_eq!(
            negotiated.offerer_to_answerer,
            Some(NegotiatedDirection { codec: VideoCodec::Av1, implementation: CodecImplementation::Software })
        );
        assert_eq!(negotiated.offerer_to_answerer, negotiated.answerer_to_offerer);
        assert!(!negotiated.is_audio_only());
    }

    #[test]
    fn old_phone_and_modern_phone_negotiate_h264_hardware() {
        // architecture doc §4's example: "Old phone: H264 HW. Pixel
        // phone: AV1/HEVC/H264 HW. Negotiated: H264."
        let old_phone = VideoCapabilities::new(vec![cap(
            VideoCodec::H264,
            CodecImplementation::Hardware,
            true,
            true,
        )]);
        let modern_phone = VideoCapabilities::new(vec![
            cap(VideoCodec::Av1, CodecImplementation::Hardware, true, true),
            cap(VideoCodec::H265, CodecImplementation::Hardware, true, true),
            cap(VideoCodec::H264, CodecImplementation::Hardware, true, true),
        ]);

        let negotiated = negotiate_call(&old_phone, &modern_phone);
        assert_eq!(
            negotiated.offerer_to_answerer,
            Some(NegotiatedDirection { codec: VideoCodec::H264, implementation: CodecImplementation::Hardware })
        );
    }

    #[test]
    fn desktop_to_old_android_with_no_av1_is_audio_only() {
        // architecture doc §5: desktop only ever offers AV1; an Android
        // device with H264-hardware-only and no AV1 software fallback
        // can't do video with it at all.
        let desktop = VideoCapabilities::desktop_av1_software();
        let old_android = VideoCapabilities::new(vec![cap(
            VideoCodec::H264,
            CodecImplementation::Hardware,
            true,
            true,
        )]);

        let negotiated = negotiate_call(&desktop, &old_android);
        assert!(negotiated.is_audio_only());
    }

    #[test]
    fn asymmetric_encode_decode_support_is_handled_per_direction() {
        // architecture doc §7: a device can decode AV1 hardware without
        // being able to encode it. Alice can only encode H264; Bob can
        // encode AV1 (hw) and decode both.
        let alice = VideoCapabilities::new(vec![
            cap(VideoCodec::Av1, CodecImplementation::Hardware, false, true), // decode-only
            cap(VideoCodec::H264, CodecImplementation::Hardware, true, true),
        ]);
        let bob = VideoCapabilities::new(vec![cap(
            VideoCodec::Av1,
            CodecImplementation::Hardware,
            true,
            true,
        )]);

        let negotiated = negotiate_call(&alice, &bob);
        // Alice -> Bob: Alice can only encode H264, Bob can't decode it -> None
        assert_eq!(negotiated.offerer_to_answerer, None);
        // Bob -> Alice: Bob encodes AV1 hw, Alice can decode AV1 -> Some
        assert_eq!(
            negotiated.answerer_to_offerer,
            Some(NegotiatedDirection { codec: VideoCodec::Av1, implementation: CodecImplementation::Hardware })
        );
        assert!(!negotiated.is_audio_only());
    }
}
