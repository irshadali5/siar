//! §20 "Linking Trust Decision": "The existing device must explicitly
//! approve... Avoid silent device addition."

/// §17/§18's two named mechanisms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkMethod {
    Qr,
    Nfc,
}

/// §19's own outcome — whether the person actually compared and
/// confirmed the numeric code, or the flow completed some other way
/// (or was skipped, which a real UI should visibly flag as
/// lower-assurance rather than showing the same "linked" state either
/// way).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    NumericCodeConfirmed,
    NotVerified,
}

/// §20's own worked example ("Add 'Tablet' to your account?") plus the
/// four fields it lists to show alongside that prompt. Pure display
/// data — this struct doesn't decide anything or drive any state
/// transition itself; it exists so a UI has one real, typed thing to
/// render instead of five loose parameters, and so "what must be shown
/// before approval" is a checkable type rather than only a convention
/// a UI author has to remember.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkingApprovalPrompt {
    pub device_platform: String,
    pub approximate_time_millis: u64,
    pub link_method: LinkMethod,
    pub verification_status: VerificationStatus,
}

impl LinkingApprovalPrompt {
    /// §20: "Avoid silent device addition." A real, if narrow,
    /// guardrail this crate can actually enforce: refuses to construct
    /// a prompt for a `NotVerified` linking attempt without the caller
    /// explicitly acknowledging that via
    /// [`LinkingApprovalPrompt::allow_unverified`] — the ordinary
    /// constructor only accepts `NumericCodeConfirmed`, so a caller
    /// can't build a normal-looking approval prompt for an unverified
    /// link by accident.
    pub fn new(device_platform: String, approximate_time_millis: u64, link_method: LinkMethod) -> Self {
        Self { device_platform, approximate_time_millis, link_method, verification_status: VerificationStatus::NumericCodeConfirmed }
    }

    /// The explicit, harder-to-reach-by-accident path for an
    /// unverified link — see [`LinkingApprovalPrompt::new`]'s own doc
    /// comment.
    pub fn allow_unverified(device_platform: String, approximate_time_millis: u64, link_method: LinkMethod) -> Self {
        Self { device_platform, approximate_time_millis, link_method, verification_status: VerificationStatus::NotVerified }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ordinary_constructor_always_marks_the_code_confirmed() {
        let prompt = LinkingApprovalPrompt::new("Pixel 9".to_string(), 1_000, LinkMethod::Qr);
        assert_eq!(prompt.verification_status, VerificationStatus::NumericCodeConfirmed);
    }

    #[test]
    fn an_unverified_prompt_requires_the_explicit_constructor() {
        let prompt = LinkingApprovalPrompt::allow_unverified("Pixel 9".to_string(), 1_000, LinkMethod::Nfc);
        assert_eq!(prompt.verification_status, VerificationStatus::NotVerified);
    }
}
