use crate::codec::VideoCodec;

/// Architecture doc §44: "Production Android media stacks often
/// require workarounds... Centralize quirks" rather than scattering
/// `if samsung...` through the call engine. This is the data shape;
/// nothing in `siar-media-core` populates it yet — the actual quirk
/// database only makes sense after real device testing (§43), which
/// needs physical hardware this can't be built against blind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecQuirk {
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub codec: Option<VideoCodec>,
    pub workaround: CodecWorkaround,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecWorkaround {
    DisableEncoder,
    CapMaxResolution { width: u32, height: u32 },
    ForceByteBufferMode,
    AvoidDynamicBitrateUpdate,
}

/// Looks up every quirk matching `manufacturer`/`model`/`codec` — a
/// device can match more than one rule (e.g. a manufacturer-wide rule
/// plus a model-specific one), so this returns all matches rather than
/// the first one; the caller (a future `media-android` capability
/// prober) decides how to combine them.
pub fn matching_quirks<'a>(
    quirks: &'a [CodecQuirk],
    manufacturer: &str,
    model: &str,
    codec: VideoCodec,
) -> Vec<&'a CodecQuirk> {
    quirks
        .iter()
        .filter(|q| {
            q.manufacturer
                .as_deref()
                .is_none_or(|m| m.eq_ignore_ascii_case(manufacturer))
                && q.model
                    .as_deref()
                    .is_none_or(|m| m.eq_ignore_ascii_case(model))
                && q.codec.is_none_or(|c| c == codec)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manufacturer_wide_rule_matches_any_model() {
        let quirks = vec![CodecQuirk {
            manufacturer: Some("Samsung".to_string()),
            model: None,
            codec: Some(VideoCodec::Av1),
            workaround: CodecWorkaround::DisableEncoder,
        }];
        let matches = matching_quirks(&quirks, "samsung", "Galaxy S23", VideoCodec::Av1);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn codec_mismatch_excludes_the_rule() {
        let quirks = vec![CodecQuirk {
            manufacturer: Some("Samsung".to_string()),
            model: None,
            codec: Some(VideoCodec::Av1),
            workaround: CodecWorkaround::DisableEncoder,
        }];
        assert!(matching_quirks(&quirks, "Samsung", "Galaxy S23", VideoCodec::H264).is_empty());
    }

    #[test]
    fn model_specific_rule_does_not_match_other_models() {
        let quirks = vec![CodecQuirk {
            manufacturer: Some("Xiaomi".to_string()),
            model: Some("Redmi Note 9".to_string()),
            codec: None,
            workaround: CodecWorkaround::CapMaxResolution {
                width: 1280,
                height: 720,
            },
        }];
        assert!(matching_quirks(&quirks, "Xiaomi", "Redmi Note 12", VideoCodec::H264).is_empty());
        assert_eq!(
            matching_quirks(&quirks, "Xiaomi", "Redmi Note 9", VideoCodec::H264).len(),
            1
        );
    }
}
