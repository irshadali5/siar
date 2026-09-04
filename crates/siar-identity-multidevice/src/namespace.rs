//! §49 "Multiple Accounts on One Device", §50 "Application Namespace",
//! §51 "Cross-Application Identity Reuse".

use siar_domain::{AccountId, DeviceId};

/// §49's own five-item isolation list, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AccountIsolationDomain {
    Keys,
    StorageNamespace,
    SessionState,
    TrustGraph,
    DeviceMembership,
}

/// One (account, device) pairing for a single account logged into this
/// physical hardware — §49: "personal / work / organization identities
/// on one physical device." Each account gets its own [`DeviceId`]
/// (its own certificate in its own account's directory), never the
/// same `DeviceId` reused across accounts — that's what makes
/// [`AccountIsolationDomain::DeviceMembership`] real rather than
/// aspirational.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LocalAccountSession {
    pub account_id: AccountId,
    pub device_id: DeviceId,
}

/// §49: "each requires isolated... device membership." Checkable here
/// as: no two [`LocalAccountSession`]s on the same physical hardware
/// may share a `DeviceId` — a shared `DeviceId` across two accounts
/// would mean one account's revocation/rotation could affect the
/// other's standing, which is exactly the isolation failure this rule
/// exists to prevent.
pub fn device_membership_is_isolated(sessions: &[LocalAccountSession]) -> bool {
    let mut seen = std::collections::HashSet::new();
    sessions.iter().all(|s| seen.insert(s.device_id))
}

/// §50, verbatim — kept as an opaque string newtype rather than a
/// closed enum: application namespaces are assigned by whoever builds
/// on this SDK, an open set this crate has no business enumerating.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ApplicationNamespace(pub String);

/// §50's own four-item "do not automatically share" list, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ApplicationScopedResource {
    Contacts,
    MessageHistory,
    FileAccess,
    SessionState,
}

/// The actual rule behind §50's list: every one of these resources
/// defaults to NOT shared across application namespaces. A `true`
/// result requires [`crate::identifier`]... this crate has no sharing
/// mechanism at all yet — this function exists to make the default
/// explicit and checkable now, before one does.
pub fn is_shared_across_applications_by_default(_resource: ApplicationScopedResource) -> bool {
    false
}

/// §51's own three possible modes, verbatim, for whether an identity
/// is reused across two different applications built on this SDK on
/// the same device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CrossApplicationIdentityMode {
    IsolatedPerApp,
    SharedWithUserConsent,
    OrganizationManaged,
}

impl Default for CrossApplicationIdentityMode {
    /// §51: "the default should favor isolation." Not merely
    /// documented — this is the literal `Default` impl, so any caller
    /// that constructs one via `Default::default()` gets isolation
    /// without having to know that's the spec-mandated safe choice.
    fn default() -> Self {
        CrossApplicationIdentityMode::IsolatedPerApp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_49_device_ids_reused_across_accounts_are_not_isolated() {
        let shared_device = DeviceId::new();
        let sessions = vec![
            LocalAccountSession {
                account_id: AccountId::new(),
                device_id: shared_device,
            },
            LocalAccountSession {
                account_id: AccountId::new(),
                device_id: shared_device,
            },
        ];
        assert!(!device_membership_is_isolated(&sessions));
    }

    #[test]
    fn spec_49_distinct_device_ids_per_account_are_isolated() {
        let sessions = vec![
            LocalAccountSession {
                account_id: AccountId::new(),
                device_id: DeviceId::new(),
            },
            LocalAccountSession {
                account_id: AccountId::new(),
                device_id: DeviceId::new(),
            },
        ];
        assert!(device_membership_is_isolated(&sessions));
    }

    #[test]
    fn spec_50_nothing_is_shared_across_applications_by_default() {
        assert!(!is_shared_across_applications_by_default(
            ApplicationScopedResource::Contacts
        ));
        assert!(!is_shared_across_applications_by_default(
            ApplicationScopedResource::MessageHistory
        ));
        assert!(!is_shared_across_applications_by_default(
            ApplicationScopedResource::FileAccess
        ));
        assert!(!is_shared_across_applications_by_default(
            ApplicationScopedResource::SessionState
        ));
    }

    #[test]
    fn spec_51_default_mode_favors_isolation() {
        assert_eq!(
            CrossApplicationIdentityMode::default(),
            CrossApplicationIdentityMode::IsolatedPerApp
        );
    }
}
