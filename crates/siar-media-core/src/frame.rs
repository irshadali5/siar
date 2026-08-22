use crate::capability::Resolution;
use crate::codec::VideoCodec;

/// Planar YUV 4:2:0, the common denominator every encoder in this
/// system (rav1e, and eventually Android hardware surfaces) accepts.
/// Deliberately not a generic pixel-format enum yet — architecture doc
/// §92 (from plan.md, not this doc, but the same principle applies):
/// don't chase generality the system doesn't need yet. If RGBA capture
/// is added later, the conversion happens at the capture boundary, not
/// by teaching every encoder every format.
#[derive(Debug, Clone)]
pub struct RawVideoFrame {
    pub resolution: Resolution,
    pub y_plane: Vec<u8>,
    pub u_plane: Vec<u8>,
    pub v_plane: Vec<u8>,
    /// Monotonic capture timestamp in microseconds — not wall-clock,
    /// just ordering + pacing for the jitter buffer on the receive side.
    pub timestamp_micros: u64,
}

impl RawVideoFrame {
    /// `y_plane` is `width*height` bytes; `u_plane`/`v_plane` are each
    /// `(width/2)*(height/2)` for 4:2:0 subsampling. This just checks
    /// the arithmetic — it doesn't allocate or convert anything.
    pub fn expected_plane_sizes(resolution: Resolution) -> (usize, usize, usize) {
        let luma = resolution.width as usize * resolution.height as usize;
        let chroma_w = resolution.width.div_ceil(2) as usize;
        let chroma_h = resolution.height.div_ceil(2) as usize;
        (luma, chroma_w * chroma_h, chroma_w * chroma_h)
    }

    pub fn is_well_formed(&self) -> bool {
        let (y, u, v) = Self::expected_plane_sizes(self.resolution);
        self.y_plane.len() == y && self.u_plane.len() == u && self.v_plane.len() == v
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncodedVideoFrame {
    pub codec: VideoCodec,
    pub data: Vec<u8>,
    pub is_keyframe: bool,
    pub timestamp_micros: u64,
}

#[derive(Debug, Clone)]
pub struct DecodedVideoFrame {
    pub frame: RawVideoFrame,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_sizes_match_4_2_0_subsampling() {
        let (y, u, v) = RawVideoFrame::expected_plane_sizes(Resolution::new(640, 360));
        assert_eq!(y, 640 * 360);
        assert_eq!(u, 320 * 180);
        assert_eq!(v, 320 * 180);
    }

    #[test]
    fn odd_dimensions_round_chroma_up() {
        // 4:2:0 chroma planes for an odd-dimension luma plane round up,
        // not down — a truncated allocation would be a decoder crash
        // waiting to happen the first time someone captures an odd size.
        let (_, u, v) = RawVideoFrame::expected_plane_sizes(Resolution::new(641, 361));
        assert_eq!(u, 321 * 181);
        assert_eq!(v, 321 * 181);
    }

    #[test]
    fn well_formed_checks_actual_buffer_lengths() {
        let resolution = Resolution::new(4, 2);
        let (y, u, v) = RawVideoFrame::expected_plane_sizes(resolution);
        let good = RawVideoFrame {
            resolution,
            y_plane: vec![0; y],
            u_plane: vec![0; u],
            v_plane: vec![0; v],
            timestamp_micros: 0,
        };
        assert!(good.is_well_formed());

        let mut bad = good.clone();
        bad.u_plane.pop();
        assert!(!bad.is_well_formed());
    }
}
