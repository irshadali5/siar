use crate::codec::{CodecImplementation, VideoCodec};

/// A codec profile identifier — deliberately a plain string rather than
/// a per-codec enum. AV1/H.264/H.265 each have their own profile
/// vocabularies (Main/High/Main10/...) and encoding this as one shared
/// enum would either lose codec-specific profiles or force every caller
/// to match on the codec first anyway. Callers that care compare against
/// well-known constants (see `profiles` module) rather than parsing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodecProfile(pub String);

impl CodecProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

pub mod profiles {
    /// Constants for the profile names actually referenced elsewhere in
    /// this crate (negotiation defaults, quirks). Not an exhaustive list
    /// of every AV1/H.264/H.265 profile — add more as real negotiation
    /// needs them.
    pub const AV1_MAIN: &str = "Main";
    pub const H264_BASELINE: &str = "Baseline";
    pub const H264_MAIN: &str = "Main";
    pub const H264_HIGH: &str = "High";
    pub const H265_MAIN: &str = "Main";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn pixel_count(self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitrateRange {
    pub min_bps: u32,
    pub max_bps: u32,
}

impl BitrateRange {
    pub fn contains(&self, bps: u32) -> bool {
        bps >= self.min_bps && bps <= self.max_bps
    }

    pub fn clamp(&self, bps: u32) -> u32 {
        bps.clamp(self.min_bps, self.max_bps)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRateRange {
    pub min_fps: u16,
    pub max_fps: u16,
}

/// What one local implementation of one codec can do — the Rust
/// equivalent of architecture doc §3's `VideoCodecCapability` struct.
/// A device can have several of these for the same `codec` (e.g. AV1
/// hardware *and* AV1 software) — that's why `Vec<VideoCodecCapability>`
/// is the shape callers hold, not a per-codec map.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoCodecCapability {
    pub codec: VideoCodec,
    pub implementation: CodecImplementation,
    pub can_encode: bool,
    pub can_decode: bool,
    pub max_resolution: Resolution,
    pub max_fps: u16,
    pub supported_profiles: Vec<CodecProfile>,
    pub bitrate_range: BitrateRange,
    pub frame_rate_range: FrameRateRange,
}

/// One endpoint's full video capability set, exchanged during call
/// signaling (architecture doc §6's `CALL_OFFER`/`CALL_ANSWER` payload).
#[derive(Debug, Clone, Default)]
pub struct VideoCapabilities {
    pub codecs: Vec<VideoCodecCapability>,
}

impl VideoCapabilities {
    pub fn new(codecs: Vec<VideoCodecCapability>) -> Self {
        Self { codecs }
    }

    /// Desktop's fixed capability set (architecture doc §2: "for
    /// Linux/Windows/macOS your application implements AV1 software
    /// encode/decode only — no software H.264/H.265/VP8/VP9").
    /// `AV1_SOFTWARE_LADDER` (see `resolution.rs`) sets the resolution
    /// ceiling: don't advertise a resolution the encoder isn't actually
    /// tuned to attempt.
    pub fn desktop_av1_software() -> Self {
        Self::new(vec![VideoCodecCapability {
            codec: VideoCodec::Av1,
            implementation: CodecImplementation::Software,
            can_encode: true,
            can_decode: true,
            max_resolution: crate::resolution::AV1_SOFTWARE_LADDER
                .last()
                .expect("ladder is non-empty")
                .resolution,
            max_fps: 30,
            supported_profiles: vec![CodecProfile::new(profiles::AV1_MAIN)],
            bitrate_range: BitrateRange {
                min_bps: 150_000,
                max_bps: 4_000_000,
            },
            frame_rate_range: FrameRateRange {
                min_fps: 15,
                max_fps: 30,
            },
        }])
    }
}
