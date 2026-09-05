//! §122 "Enterprise Device Policy", §123 "Platform Attestation", §124
//! "Device Health Claims".
//!
//! One thread runs through all three: none of this may become a
//! second way to establish identity. §122 says these must be
//! "optional policy layers" and explicitly "do not make attestation
//! mandatory for the core protocol"; §123 says attestation "must not
//! replace cryptographic identity" — it answers a different question
//! ("what environment is this device running in?" vs. identity's
//! "which logical device/account is this?"); §124 calls health claims
//! "claims/capabilities, not identity itself." Structurally: nothing
//! in this module can produce a [`crate::root_key::RootPublicKey`], a
//! [`crate::directory::DeviceDirectory`], or a
//! [`crate::certificate::DeviceCertificate`] — every type here is
//! pure data an application-level policy can layer on top of an
//! already-established identity, never a substitute for one.

use serde::{Deserialize, Serialize};

/// §124's four named optional claims, verbatim. Every field is
/// self-reported by the device — this type carries no verification of
/// its own, matching §124's "claims," not "proofs." A caller wanting
/// stronger assurance combines this with [`PlatformAttestation`]
/// (itself also unverified by this crate — see that type's own doc
/// comment) via whatever platform-specific attestation service they
/// trust; neither type does that verification here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceHealthClaims {
    pub os_version: Option<String>,
    pub app_version: Option<String>,
    pub secure_storage_available: Option<bool>,
    pub hardware_key_support: Option<bool>,
}

/// §123: attestation "answers: what environment is this device
/// running in?" — deliberately just a platform tag plus an opaque
/// statement, never an `AccountId`/`DeviceId`/certificate field. There
/// is no method on this type that returns anything resembling
/// identity, and no function in this crate that accepts a
/// `PlatformAttestation` in place of a `DeviceCertificate` anywhere —
/// verifying the statement bytes against Play Integrity/DeviceCheck is
/// a platform-specific concern this dependency-minimal crate doesn't
/// implement (same posture as §96's transparency log or §116's
/// storage boundary: a real shape, no implementation).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Android,
    Ios,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformAttestation {
    pub platform: Platform,
    pub statement: Vec<u8>,
}

/// §122's five named optional requirements, as booleans/an optional
/// version floor rather than a single "compliant: bool" — an
/// application needs to know WHICH requirement failed to give a
/// useful error, and collapsing them early would lose that.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnterpriseDevicePolicy {
    pub require_approved_device: bool,
    pub require_managed_device: bool,
    pub minimum_app_version: Option<String>,
    pub require_hardware_backed_keys: bool,
    pub require_attestation: bool,
}

/// One requirement §122 names that failed, plus enough context to
/// explain why — returned as a `Vec` from
/// [`EnterpriseDevicePolicy::check`] rather than the first failure
/// only, since an application surfacing this to an admin benefits from
/// the complete list in one pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnterprisePolicyViolation {
    DeviceNotApproved,
    DeviceNotManaged,
    AppVersionBelowMinimum {
        minimum: String,
        actual: Option<String>,
    },
    HardwareBackedKeysUnavailable,
    AttestationMissing,
}

impl EnterpriseDevicePolicy {
    /// `Default::default()` is every field at its most permissive
    /// (`false`/`None`) — §122's own "do not make attestation
    /// mandatory for the core protocol" made structural: a caller who
    /// never opts into an `EnterpriseDevicePolicy` gets a policy that
    /// requires nothing at all, not a policy this crate silently
    /// defaults to being strict.
    ///
    /// `is_approved`/`is_managed`/`attestation` are the application's
    /// own determinations (MDM enrollment status, an actual attestation
    /// result) — this function only combines them against policy, it
    /// does not determine any of them itself.
    pub fn check(
        &self,
        is_approved: bool,
        is_managed: bool,
        health: &DeviceHealthClaims,
        attestation: Option<&PlatformAttestation>,
    ) -> Vec<EnterprisePolicyViolation> {
        let mut violations = Vec::new();

        if self.require_approved_device && !is_approved {
            violations.push(EnterprisePolicyViolation::DeviceNotApproved);
        }
        if self.require_managed_device && !is_managed {
            violations.push(EnterprisePolicyViolation::DeviceNotManaged);
        }
        if let Some(minimum) = &self.minimum_app_version {
            // Lexicographic comparison of version strings is only
            // correct for equal-width, zero-padded numeric segments —
            // a known limitation, not a claim of real semver ordering.
            // A real implementation should parse and compare
            // structured version numbers; this crate stays
            // dependency-minimal and leaves that parsing to the
            // application, same posture as §125's wire-versioning gap
            // note below.
            let below_minimum = match &health.app_version {
                Some(actual) => actual.as_str() < minimum.as_str(),
                None => true,
            };
            if below_minimum {
                violations.push(EnterprisePolicyViolation::AppVersionBelowMinimum {
                    minimum: minimum.clone(),
                    actual: health.app_version.clone(),
                });
            }
        }
        if self.require_hardware_backed_keys && health.hardware_key_support != Some(true) {
            violations.push(EnterprisePolicyViolation::HardwareBackedKeysUnavailable);
        }
        if self.require_attestation && attestation.is_none() {
            violations.push(EnterprisePolicyViolation::AttestationMissing);
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_requires_nothing() {
        let policy = EnterpriseDevicePolicy::default();
        let health = DeviceHealthClaims {
            os_version: None,
            app_version: None,
            secure_storage_available: None,
            hardware_key_support: None,
        };
        assert!(policy.check(false, false, &health, None).is_empty());
    }

    #[test]
    fn reports_every_failed_requirement_not_just_the_first() {
        let policy = EnterpriseDevicePolicy {
            require_approved_device: true,
            require_managed_device: true,
            minimum_app_version: None,
            require_hardware_backed_keys: true,
            require_attestation: true,
        };
        let health = DeviceHealthClaims {
            os_version: None,
            app_version: None,
            secure_storage_available: None,
            hardware_key_support: Some(false),
        };

        let violations = policy.check(false, false, &health, None);
        assert_eq!(violations.len(), 4);
        assert!(violations.contains(&EnterprisePolicyViolation::DeviceNotApproved));
        assert!(violations.contains(&EnterprisePolicyViolation::DeviceNotManaged));
        assert!(violations.contains(&EnterprisePolicyViolation::HardwareBackedKeysUnavailable));
        assert!(violations.contains(&EnterprisePolicyViolation::AttestationMissing));
    }

    #[test]
    fn app_version_below_minimum_is_flagged() {
        let policy = EnterpriseDevicePolicy {
            minimum_app_version: Some("2.0.0".to_string()),
            ..Default::default()
        };
        let health = DeviceHealthClaims {
            os_version: None,
            app_version: Some("1.5.0".to_string()),
            secure_storage_available: None,
            hardware_key_support: None,
        };
        let violations = policy.check(true, true, &health, None);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn satisfying_every_requirement_yields_no_violations() {
        let policy = EnterpriseDevicePolicy {
            require_approved_device: true,
            require_managed_device: true,
            minimum_app_version: Some("2.0.0".to_string()),
            require_hardware_backed_keys: true,
            require_attestation: true,
        };
        let health = DeviceHealthClaims {
            os_version: Some("14".to_string()),
            app_version: Some("2.0.0".to_string()),
            secure_storage_available: Some(true),
            hardware_key_support: Some(true),
        };
        let attestation = PlatformAttestation {
            platform: Platform::Android,
            statement: vec![1, 2, 3],
        };

        assert!(policy
            .check(true, true, &health, Some(&attestation))
            .is_empty());
    }
}
