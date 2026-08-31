//! Security Center Presentation API (ui-ux-15 §135-140).
//!
//! §138-140's traits are the actual contract between whatever runs the
//! real backend logic (a background task talking to
//! `siar-crypto`/`siar-identity-multidevice`/etc, outside this crate
//! entirely per this crate's own dependency rule) and the Dioxus/
//! Compose UI layers. Everything above this module (`security_center`,
//! `device_lifecycle`, `recovery`, ...) is the *shape* of the data;
//! this module is the *contract* for how a UI actually asks for it and
//! acts on it.
//!
//! Reuses existing types wherever the spec's own named type is the
//! same concept under a different name, rather than duplicating:
//! `RecoveryStatusView` is `RecoveryOverview` (§62-63, already built);
//! §137's `SecurityEventView` is this crate's existing `SecurityEvent`
//! (§40). Where the spec's referenced type genuinely doesn't exist yet
//! (`UiError`, `SecurityEventCursor`/`Page`, `ReauthProof`,
//! `RevocationResultView`, `RecoveryMaterialView`,
//! `RecoveryVerificationInput`/`Result`), it's built here, minimally,
//! since these API traits can't type-check without them.

use siar_domain::DeviceId;

use crate::device_lifecycle::ReauthPurpose;
use crate::recovery::{RecoveryMethod, RecoveryOverview, RecoveryStatus};
use crate::recovery_advanced::BackupSecurityState;
use crate::security_center::{DeviceSecurityView, SecurityHealth};
use crate::security_event::{SecurityEvent, SecurityEventId};
use crate::revocation_lifecycle::RevocationState;

/// §135, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceSecurityCapabilities {
    pub can_revoke: bool,
    pub can_rename: bool,
    pub can_view_activity: bool,
    pub can_reauthenticate: bool,
}

/// §136, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryCapabilities {
    pub can_create: bool,
    pub can_rotate: bool,
    pub can_test: bool,
    pub can_export: bool,
}

/// §137, verbatim — `SecurityEventView` in the spec's own sketch is
/// this crate's `SecurityEvent` (see this module's top doc comment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityCenterSnapshot {
    pub health: SecurityHealth,
    pub trusted_device_count: u32,
    pub unresolved_event_count: u32,
    pub recovery_status: RecoveryStatus,
    pub backup_security: BackupSecurityState,
    pub top_events: Vec<SecurityEvent>,
}

/// Minimal, honestly-scoped error type for this API surface — the spec
/// references `UiError` throughout §138-140 but never defines it. Same
/// under-specification pattern as `DeviceSecurityFlag`/
/// `SecurityEventAction` elsewhere in this crate: a small, real set
/// covering the failure modes an async presentation call obviously
/// needs (not found, not authorized, connectivity, and a catch-all),
/// not a guess at an exhaustive taxonomy — §204 ("Security Error
/// Taxonomy") is a later, more complete section of this same spec this
/// round doesn't reach yet, and may supersede this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiError {
    NotFound,
    NotAuthorized,
    Network,
    Internal(String),
}

/// Opaque pagination cursor for `SecurityPresentation::events` (§138).
/// Wraps the last-seen `SecurityEventId` rather than a raw offset —
/// stable under insertion the same way `SecurityEventId` itself already
/// is, so a page boundary doesn't shift if new events arrive between
/// calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityEventCursor(pub SecurityEventId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityEventPage {
    pub events: Vec<SecurityEvent>,
    pub next_cursor: Option<SecurityEventCursor>,
}

/// Proof that a reauthentication for `purpose` actually succeeded —
/// the thing `device_lifecycle::ReauthResult::Success` produces that a
/// caller then hands to a high-risk API call like `revoke`/`create`/
/// `rotate` below. Fields are private with no public constructor other
/// than `new`: this type's whole point is that its existence should
/// mean "a real reauth happened," so it's deliberately not
/// `Default`-constructible or built from raw parts a caller could
/// fabricate casually. This crate can't *cryptographically* enforce
/// that only a genuine successful reauth ever calls `new` — that
/// enforcement is the platform reauth adapter's job (§142-143, not
/// built this round) — but the type's shape doesn't invite misuse the
/// way a bare `bool` or unit struct would.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReauthProof {
    purpose: ReauthPurpose,
    verified_at_millis: u64,
}

impl ReauthProof {
    pub fn new(purpose: ReauthPurpose, verified_at_millis: u64) -> Self {
        Self { purpose, verified_at_millis }
    }

    pub fn purpose(&self) -> ReauthPurpose {
        self.purpose
    }

    pub fn verified_at_millis(&self) -> u64 {
        self.verified_at_millis
    }
}

/// The outcome of one `DeviceSecurityPresentation::revoke` call.
/// Deliberately not the same type as `revocation_lifecycle::RevocationProgress`
/// — that type tracks an in-progress revocation over time (it carries
/// `device`, since a caller may be tracking several at once);
/// `RevocationResultView` is just the settled answer to one already-
/// identified call, so it doesn't need to repeat the device ID the
/// caller already passed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevocationResultView {
    pub state: RevocationState,
    pub offline_pending: bool,
}

/// The result of `RecoveryPresentation::create`/`rotate` (§140) —
/// **the one place actual recovery secret material appears anywhere in
/// this crate's type surface.** Every other recovery-related type in
/// this crate (`RecoveryKeyDisplayState`, `RecoveryOverview`, ...)
/// deliberately holds no secret bytes/text, only *state about*
/// secrets — because until now, nothing in this crate's API needed to
/// carry a freshly-generated secret back to a caller. `create`/
/// `rotate` are exactly that need: the material has to be shown to the
/// user at least once at creation time, and this is where it's
/// carried. `RecoveryKeyDisplayState` still governs *re-display* of an
/// already-existing key later — this type and that one aren't in
/// tension, they cover two different moments (first creation vs.
/// later reveal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryMaterialView {
    pub method: RecoveryMethod,
    pub material_display: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryVerificationInput {
    pub method: RecoveryMethod,
    pub submitted_material: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryVerificationResult {
    Valid,
    Invalid,
    Expired,
}

/// §138, verbatim.
#[async_trait::async_trait]
pub trait SecurityPresentation {
    async fn snapshot(&self) -> Result<SecurityCenterSnapshot, UiError>;
    async fn events(&self, cursor: Option<SecurityEventCursor>) -> Result<SecurityEventPage, UiError>;
    async fn resolve_event(&self, event: SecurityEventId) -> Result<(), UiError>;
}

/// §139, verbatim.
#[async_trait::async_trait]
pub trait DeviceSecurityPresentation {
    async fn devices(&self) -> Result<Vec<DeviceSecurityView>, UiError>;
    async fn device(&self, id: DeviceId) -> Result<DeviceSecurityView, UiError>;
    async fn rename(&self, id: DeviceId, name: String) -> Result<(), UiError>;
    async fn revoke(&self, id: DeviceId, auth: ReauthProof) -> Result<RevocationResultView, UiError>;
}

/// §140, verbatim. `RecoveryStatusView` is `RecoveryOverview` (see this
/// module's top doc comment).
#[async_trait::async_trait]
pub trait RecoveryPresentation {
    async fn status(&self) -> Result<RecoveryOverview, UiError>;
    async fn create(&self, auth: ReauthProof) -> Result<RecoveryMaterialView, UiError>;
    async fn verify(&self, proof: RecoveryVerificationInput) -> Result<RecoveryVerificationResult, UiError>;
    async fn rotate(&self, auth: ReauthProof) -> Result<RecoveryMaterialView, UiError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use siar_domain::AccountId;

    /// A minimal mock implementation — not a real backend, just enough
    /// to prove the trait's method signatures actually type-check and
    /// are callable through `dyn` / generic call sites, the same way a
    /// real background-task implementation eventually will be.
    struct MockSecurityPresentation;

    #[async_trait::async_trait]
    impl SecurityPresentation for MockSecurityPresentation {
        async fn snapshot(&self) -> Result<SecurityCenterSnapshot, UiError> {
            Ok(SecurityCenterSnapshot {
                health: SecurityHealth::Healthy,
                trusted_device_count: 2,
                unresolved_event_count: 0,
                recovery_status: RecoveryStatus::Configured,
                backup_security: BackupSecurityState::Healthy,
                top_events: Vec::new(),
            })
        }

        async fn events(&self, _cursor: Option<SecurityEventCursor>) -> Result<SecurityEventPage, UiError> {
            Ok(SecurityEventPage { events: Vec::new(), next_cursor: None })
        }

        async fn resolve_event(&self, _event: SecurityEventId) -> Result<(), UiError> {
            Ok(())
        }
    }

    struct MockRecoveryPresentation;

    #[async_trait::async_trait]
    impl RecoveryPresentation for MockRecoveryPresentation {
        async fn status(&self) -> Result<RecoveryOverview, UiError> {
            Ok(RecoveryOverview {
                account: AccountId::new(),
                status: RecoveryStatus::NotConfigured,
                last_verified_millis: None,
                configured_methods: Vec::new(),
            })
        }

        async fn create(&self, auth: ReauthProof) -> Result<RecoveryMaterialView, UiError> {
            if auth.purpose() != ReauthPurpose::ShowRecoveryKey {
                return Err(UiError::NotAuthorized);
            }
            Ok(RecoveryMaterialView {
                method: RecoveryMethod::RecoveryKey,
                material_display: "AAAA-BBBB-CCCC-DDDD".to_string(),
            })
        }

        async fn verify(&self, proof: RecoveryVerificationInput) -> Result<RecoveryVerificationResult, UiError> {
            if proof.submitted_material == "AAAA-BBBB-CCCC-DDDD" {
                Ok(RecoveryVerificationResult::Valid)
            } else {
                Ok(RecoveryVerificationResult::Invalid)
            }
        }

        async fn rotate(&self, _auth: ReauthProof) -> Result<RecoveryMaterialView, UiError> {
            Ok(RecoveryMaterialView {
                method: RecoveryMethod::RecoveryKey,
                material_display: "EEEE-FFFF-GGGG-HHHH".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn security_presentation_snapshot_round_trips_through_a_mock() {
        let presentation = MockSecurityPresentation;
        let snapshot = presentation.snapshot().await.unwrap();
        assert_eq!(snapshot.health, SecurityHealth::Healthy);
        assert_eq!(snapshot.trusted_device_count, 2);
    }

    #[tokio::test]
    async fn recovery_create_requires_the_right_reauth_purpose() {
        let presentation = MockRecoveryPresentation;
        let wrong_purpose = ReauthProof::new(ReauthPurpose::ResetIdentity, 1_000);
        assert_eq!(presentation.create(wrong_purpose).await, Err(UiError::NotAuthorized));

        let right_purpose = ReauthProof::new(ReauthPurpose::ShowRecoveryKey, 1_000);
        assert!(presentation.create(right_purpose).await.is_ok());
    }

    #[tokio::test]
    async fn recovery_verify_distinguishes_valid_from_invalid_material() {
        let presentation = MockRecoveryPresentation;
        let valid = RecoveryVerificationInput {
            method: RecoveryMethod::RecoveryKey,
            submitted_material: "AAAA-BBBB-CCCC-DDDD".to_string(),
        };
        assert_eq!(presentation.verify(valid).await.unwrap(), RecoveryVerificationResult::Valid);

        let invalid = RecoveryVerificationInput {
            method: RecoveryMethod::RecoveryKey,
            submitted_material: "wrong".to_string(),
        };
        assert_eq!(presentation.verify(invalid).await.unwrap(), RecoveryVerificationResult::Invalid);
    }

    #[test]
    fn security_event_cursor_wraps_a_stable_event_id() {
        let mut events = crate::security_event::SecurityEventState::new();
        let id = events.push(crate::security_event::SecurityEventKind::DeviceLinked, 1_000, None, None, vec![]);
        let cursor = SecurityEventCursor(id);
        assert_eq!(cursor.0, id);
    }
}
