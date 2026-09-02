//! spec §67 "Deprecation Lifecycle", §68 "Security Deprecation".

/// spec §67, verbatim four-stage lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeprecationStatus {
    Supported,
    Deprecated,
    DisabledByDefault,
    Removed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("cannot advance deprecation status from {from:?} to {to:?} — the normal path is strictly linear; see force_security_deprecate for the one sanctioned exception")]
pub struct InvalidDeprecationTransition {
    pub from: DeprecationStatus,
    pub to: DeprecationStatus,
}

impl DeprecationStatus {
    /// spec §67's diagram, strictly linear, one stage at a time —
    /// mirrors [`crate::lifecycle::ExtensionLifecycle`]'s own
    /// no-skip-ahead pattern. §67's other explicit rule, "do not
    /// suddenly remove compatible wire protocols unless security
    /// requires it," is exactly why this method has no way to jump
    /// straight to `Removed` — [`Self::force_security_deprecate`] is
    /// the one sanctioned way around that, and it's a different method
    /// entirely, not a parameter on this one.
    pub fn advance(
        self,
        to: DeprecationStatus,
    ) -> Result<DeprecationStatus, InvalidDeprecationTransition> {
        use DeprecationStatus::*;
        let valid = matches!(
            (self, to),
            (Supported, Deprecated)
                | (Deprecated, DisabledByDefault)
                | (DisabledByDefault, Removed)
        );
        if valid {
            Ok(to)
        } else {
            Err(InvalidDeprecationTransition { from: self, to })
        }
    }

    /// spec §68: "Security outranks backward compatibility." The one
    /// sanctioned way to skip stages in the normal §67 lifecycle —
    /// from any non-`Removed` status straight to `DisabledByDefault`,
    /// bypassing `Deprecated` entirely, because a protocol found to be
    /// unsafe has no business staying `Supported`-but-deprecated for a
    /// normal timeline. Never advances all the way to `Removed`
    /// automatically — §68's own list ends at "disable it... require
    /// upgrade where necessary," not deletion; an operator decides
    /// `Removed` separately once upgraded peers are confirmed.
    pub fn force_security_deprecate(self) -> DeprecationStatus {
        DeprecationStatus::DisabledByDefault
    }
}

/// spec §68's own four-step list, verbatim and in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SecurityDeprecationAction {
    MarkInsecure,
    Disable,
    ProvideDiagnostic,
    RequireUpgrade,
}

pub const SECURITY_DEPRECATION_STEPS: [SecurityDeprecationAction; 4] = [
    SecurityDeprecationAction::MarkInsecure,
    SecurityDeprecationAction::Disable,
    SecurityDeprecationAction::ProvideDiagnostic,
    SecurityDeprecationAction::RequireUpgrade,
];

#[cfg(test)]
mod tests {
    use super::*;
    use DeprecationStatus::*;

    #[test]
    fn spec_67_normal_path_is_strictly_linear() {
        assert_eq!(Supported.advance(Deprecated), Ok(Deprecated));
        assert_eq!(Deprecated.advance(DisabledByDefault), Ok(DisabledByDefault));
        assert_eq!(DisabledByDefault.advance(Removed), Ok(Removed));
    }

    #[test]
    fn spec_67_cannot_skip_stages_normally() {
        assert!(Supported.advance(DisabledByDefault).is_err());
        assert!(Supported.advance(Removed).is_err());
        assert!(Deprecated.advance(Removed).is_err());
    }

    #[test]
    fn spec_67_removed_is_terminal() {
        assert!(Removed.advance(Supported).is_err());
    }

    #[test]
    fn spec_68_security_deprecation_skips_straight_to_disabled_from_supported() {
        // The exact case §67's linear rule would otherwise forbid:
        // Supported -> DisabledByDefault with no Deprecated stopover.
        assert_eq!(Supported.force_security_deprecate(), DisabledByDefault);
    }

    #[test]
    fn spec_68_security_deprecation_never_auto_removes() {
        assert_ne!(Supported.force_security_deprecate(), Removed);
    }

    #[test]
    fn spec_68_steps_are_in_spec_order() {
        assert_eq!(
            SECURITY_DEPRECATION_STEPS,
            [
                SecurityDeprecationAction::MarkInsecure,
                SecurityDeprecationAction::Disable,
                SecurityDeprecationAction::ProvideDiagnostic,
                SecurityDeprecationAction::RequireUpgrade,
            ]
        );
    }
}
