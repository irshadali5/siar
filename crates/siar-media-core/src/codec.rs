use std::fmt;

/// Video codecs the system knows about — deliberately not "every codec
/// that exists": desktop only ever implements `Av1` (software), Android
/// adds `H264`/`H265` for hardware interop (architecture doc §2–§5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VideoCodec {
    Av1,
    H264,
    H265,
}

impl fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            VideoCodec::Av1 => "AV1",
            VideoCodec::H264 => "H.264",
            VideoCodec::H265 => "H.265",
        })
    }
}

/// Whether a codec implementation is CPU-only or backed by the
/// platform's hardware block. Purely descriptive — `media-core` doesn't
/// care which crate provided the implementation, only whether it's safe
/// to assume the CPU-cost/thermal characteristics of hardware encode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodecImplementation {
    Software,
    Hardware,
}

/// Audio codec identity. Only `Opus` exists in this system — "keep
/// audio simple: Opus everywhere" (architecture doc §10) — but this is
/// still an enum rather than a unit type so a second audio codec is a
/// non-breaking addition later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioCodec {
    Opus,
}

impl fmt::Display for AudioCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Opus")
    }
}
