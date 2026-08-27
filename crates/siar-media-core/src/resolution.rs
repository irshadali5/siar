use crate::capability::Resolution;

/// One rung of a quality ladder: a resolution/frame-rate pair the call
/// controller can step down (or up) to, roughly ordered by encoding
/// cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualityRung {
    pub resolution: Resolution,
    pub fps: u16,
    pub target_bitrate_bps: u32,
}

/// Architecture doc §2's desktop AV1-software starting targets: "don't
/// start by trying to software-encode 4K 60fps." Ordered lowest-cost
/// first. `adaptive.rs`'s controller walks this ladder rather than
/// picking an arbitrary resolution, so "one step down" always means the
/// same thing.
pub const AV1_SOFTWARE_LADDER: &[QualityRung] = &[
    QualityRung {
        resolution: Resolution::new(640, 360),
        fps: 30,
        target_bitrate_bps: 300_000,
    },
    QualityRung {
        resolution: Resolution::new(854, 480),
        fps: 30,
        target_bitrate_bps: 700_000,
    },
    QualityRung {
        resolution: Resolution::new(1280, 720),
        fps: 30,
        target_bitrate_bps: 1_500_000,
    },
    QualityRung {
        resolution: Resolution::new(1920, 1080),
        fps: 30,
        target_bitrate_bps: 3_000_000,
    },
];

/// Architecture doc §46's thermal/CPU degradation ladder for Android
/// hardware encode — starts higher than the desktop software ladder
/// because hardware AV1/HEVC/AVC encode is far cheaper than rav1e
/// software encode, until thermal throttling kicks in.
pub const ANDROID_HARDWARE_LADDER: &[QualityRung] = &[
    QualityRung {
        resolution: Resolution::new(640, 360),
        fps: 20,
        target_bitrate_bps: 250_000,
    },
    QualityRung {
        resolution: Resolution::new(960, 540),
        fps: 24,
        target_bitrate_bps: 600_000,
    },
    QualityRung {
        resolution: Resolution::new(1280, 720),
        fps: 30,
        target_bitrate_bps: 1_200_000,
    },
    QualityRung {
        resolution: Resolution::new(1920, 1080),
        fps: 30,
        target_bitrate_bps: 2_500_000,
    },
];

/// `rung_below`/`rung_above` are what `adaptive.rs`'s controller calls
/// after deciding "step down" or "step up" — they never invent a
/// resolution outside a fixed ladder.
pub fn rung_below(ladder: &[QualityRung], current: QualityRung) -> Option<QualityRung> {
    let idx = ladder.iter().position(|r| *r == current)?;
    if idx == 0 {
        None
    } else {
        Some(ladder[idx - 1])
    }
}

pub fn rung_above(ladder: &[QualityRung], current: QualityRung) -> Option<QualityRung> {
    let idx = ladder.iter().position(|r| *r == current)?;
    ladder.get(idx + 1).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_is_sorted_by_pixel_count() {
        for ladder in [AV1_SOFTWARE_LADDER, ANDROID_HARDWARE_LADDER] {
            for pair in ladder.windows(2) {
                assert!(pair[0].resolution.pixel_count() < pair[1].resolution.pixel_count());
            }
        }
    }

    #[test]
    fn rung_below_bottom_is_none() {
        let bottom = AV1_SOFTWARE_LADDER[0];
        assert_eq!(rung_below(AV1_SOFTWARE_LADDER, bottom), None);
    }

    #[test]
    fn rung_above_top_is_none() {
        let top = *AV1_SOFTWARE_LADDER.last().unwrap();
        assert_eq!(rung_above(AV1_SOFTWARE_LADDER, top), None);
    }

    #[test]
    fn steps_move_exactly_one_rung() {
        let mid = AV1_SOFTWARE_LADDER[1];
        assert_eq!(
            rung_below(AV1_SOFTWARE_LADDER, mid),
            Some(AV1_SOFTWARE_LADDER[0])
        );
        assert_eq!(
            rung_above(AV1_SOFTWARE_LADDER, mid),
            Some(AV1_SOFTWARE_LADDER[2])
        );
    }
}
