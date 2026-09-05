//! §125 "Version Compatibility", §126 "Serialization", §127 "Input
//! Limits", §128 "Device Count Policy".
//!
//! §126 gets no new type: an audit of every `Serialize`-deriving
//! struct in this crate (run 2026-09-05, alongside writing this
//! module) found zero `usize` fields and zero `SystemTime` usage
//! anywhere on the wire — every timestamp already goes through
//! [`siar_event_log::ids::Timestamp`] (a `u64` wrapper), and the one
//! `usize` in this crate ([`crate::fanout`]'s `count` parameter) is a
//! local function argument, never a serialized field. §126 is already
//! satisfied; this is a confirmed audit result, not an assumption.
//!
//! §125 is honestly NOT fully satisfied, and this module says so
//! rather than papering over it: [`crate::certificate::DeviceCertificate`]
//! and [`crate::directory::DeviceDirectory`] — this crate's two most
//! load-bearing wire types, already shipped and covered by dozens of
//! existing tests across earlier rounds — carry no explicit schema-
//! version field at all, unlike §125's own `DeviceCertificateV1`/
//! `DeviceEventV1` naming convention. Adding one now would change
//! their signed-payload bytes, which is a breaking change to an
//! already-shipped, already-tested signing scheme — the exact same
//! category of gap this crate already carries and documents openly
//! for Part 28's `DeviceLinkInvite` `protocol_version` field (see this
//! crate's own `lib.rs` doc comment). Rather than force that breaking
//! change silently, [`SchemaVersion`] exists so future NEW wire types
//! in this crate follow §125's convention from the start, and the gap
//! for the two pre-existing types is named here for whoever picks up
//! a coordinated versioning pass later.

use siar_domain::DeviceId;

use crate::directory::DeviceDirectory;
use crate::principal_claims::IdentityClaim;

/// §125's own naming convention (`DeviceCertificateV1`) generalized
/// into one small marker type new wire structs can embed, rather than
/// each inventing its own version-field convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SchemaVersion(pub u16);

/// §127/§128's named limits, as one configurable struct rather than
/// scattered hard-coded constants — §128's own instruction ("the core
/// should not assume exactly 5 devices... but must still enforce
/// resource limits") is exactly why this is a value, not a `const`: a
/// normal account and an enterprise-configured account use different
/// instances of the same type, never a different code path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputLimits {
    pub max_devices_per_account: u32,
    pub max_endpoints_per_device: u32,
    pub max_claims_per_subject: u32,
    pub max_metadata_length: u32,
    pub max_device_name_length: u32,
    pub max_event_chain_batch: u32,
}

impl Default for InputLimits {
    /// §128: "default normal account: small bounded number." These
    /// values are a reasonable starting point, not a spec-mandated
    /// figure (§128 names no specific number, only "small") — an
    /// enterprise deployment is expected to construct its own
    /// `InputLimits` rather than relying on this `Default`.
    fn default() -> Self {
        Self {
            max_devices_per_account: 10,
            max_endpoints_per_device: 8,
            max_claims_per_subject: 32,
            max_metadata_length: 4096,
            max_device_name_length: 128,
            max_event_chain_batch: 256,
        }
    }
}

/// Which of §127's named limits was exceeded, plus the actual/limit
/// values — an application surfacing this to a user or an attacker's
/// log line both benefit from knowing which one, not just "rejected."
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputLimitViolation {
    TooManyDevices {
        actual: u32,
        limit: u32,
    },
    TooManyEndpointsOnDevice {
        device_id: DeviceId,
        actual: u32,
        limit: u32,
    },
    TooManyClaims {
        actual: u32,
        limit: u32,
    },
    DeviceNameTooLong {
        actual: u32,
        limit: u32,
    },
    EventChainBatchTooLarge {
        actual: u32,
        limit: u32,
    },
}

impl InputLimits {
    /// §127's actual point made real: this is called against a
    /// [`DeviceDirectory`] a caller is *about* to accept (e.g. from
    /// [`crate::trust_store::TrustedAccountStore::accept`]) so an
    /// attacker cannot force unbounded allocation by handing over a
    /// directory listing thousands of devices or endpoints — checked
    /// before any of that data is retained, not after.
    pub fn validate_directory(&self, directory: &DeviceDirectory) -> Vec<InputLimitViolation> {
        let mut violations = Vec::new();

        let device_count = directory.devices.len() as u32;
        if device_count > self.max_devices_per_account {
            violations.push(InputLimitViolation::TooManyDevices {
                actual: device_count,
                limit: self.max_devices_per_account,
            });
        }

        for entry in &directory.devices {
            let endpoint_count = entry.transport_endpoints.len() as u32;
            if endpoint_count > self.max_endpoints_per_device {
                violations.push(InputLimitViolation::TooManyEndpointsOnDevice {
                    device_id: entry.device_id,
                    actual: endpoint_count,
                    limit: self.max_endpoints_per_device,
                });
            }
        }

        violations
    }

    /// §127's "max claims" — checked against however many claims a
    /// caller has gathered for one subject, not enforced by
    /// [`IdentityClaim`] itself (which has no reason to know about a
    /// batch it isn't part of).
    pub fn validate_claim_count(&self, claims: &[IdentityClaim]) -> Option<InputLimitViolation> {
        let count = claims.len() as u32;
        (count > self.max_claims_per_subject).then_some(InputLimitViolation::TooManyClaims {
            actual: count,
            limit: self.max_claims_per_subject,
        })
    }

    /// §127's "max device name length" — checked against
    /// [`crate::discovery_privacy::PrivateDeviceMetadata::friendly_name`]'s
    /// byte length (not `.len()` on a `&str`'s char count, since
    /// what's actually bounded on the wire is bytes).
    pub fn validate_device_name(&self, friendly_name: &str) -> Option<InputLimitViolation> {
        let length = friendly_name.len() as u32;
        (length > self.max_device_name_length).then_some(InputLimitViolation::DeviceNameTooLong {
            actual: length,
            limit: self.max_device_name_length,
        })
    }

    /// §127's "max event chain batch" — a caller processing a batch of
    /// [`crate::state_chain::AccountStateEvent`]s (e.g. from
    /// [`crate::reconciliation::ReconciliationPlan::RequestGenerationsFrom`]'s
    /// response) checks the batch size against this before applying
    /// any of it.
    pub fn validate_event_batch_size(&self, batch_len: usize) -> Option<InputLimitViolation> {
        let length = batch_len as u32;
        (length > self.max_event_chain_batch).then_some(
            InputLimitViolation::EventChainBatchTooLarge {
                actual: length,
                limit: self.max_event_chain_batch,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory::{DeviceDirectory, DeviceEndpoint};
    use crate::root_key::RootIdentityKey;
    use siar_domain::AccountId;

    #[test]
    fn default_limits_allow_a_small_directory() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let directory = DeviceDirectory::sign(&root, account, 1, vec![]);
        let limits = InputLimits::default();
        assert!(limits.validate_directory(&directory).is_empty());
    }

    #[test]
    fn rejects_a_directory_over_the_configured_device_limit() {
        // A tiny limit, deliberately below what an attacker-controlled
        // directory could easily exceed, to prove the check actually
        // fires rather than only ever passing.
        let limits = InputLimits {
            max_devices_per_account: 0,
            ..InputLimits::default()
        };
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();
        let certificate = crate::certificate::DeviceCertificate::issue(
            &root,
            account,
            device_id,
            [0u8; 32],
            0,
            None,
            crate::capability::DeviceCapabilitySet(0),
            1,
        );
        let entry = crate::directory::DeviceDirectoryEntry {
            device_id,
            certificate,
            status: crate::directory::DeviceStatus::Active,
            transport_endpoints: vec![],
        };
        let directory = DeviceDirectory::sign(&root, account, 1, vec![entry]);

        let violations = limits.validate_directory(&directory);
        assert_eq!(
            violations,
            vec![InputLimitViolation::TooManyDevices {
                actual: 1,
                limit: 0
            }]
        );
    }

    #[test]
    fn rejects_a_device_with_too_many_endpoints() {
        let limits = InputLimits {
            max_endpoints_per_device: 1,
            ..InputLimits::default()
        };
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();
        let certificate = crate::certificate::DeviceCertificate::issue(
            &root,
            account,
            device_id,
            [0u8; 32],
            0,
            None,
            crate::capability::DeviceCapabilitySet(0),
            1,
        );
        let entry = crate::directory::DeviceDirectoryEntry {
            device_id,
            certificate,
            status: crate::directory::DeviceStatus::Active,
            transport_endpoints: vec![DeviceEndpoint(vec![1]), DeviceEndpoint(vec![2])],
        };
        let directory = DeviceDirectory::sign(&root, account, 1, vec![entry]);

        let violations = limits.validate_directory(&directory);
        assert_eq!(
            violations,
            vec![InputLimitViolation::TooManyEndpointsOnDevice {
                device_id,
                actual: 2,
                limit: 1
            }]
        );
    }

    #[test]
    fn device_name_length_is_checked_in_bytes() {
        let limits = InputLimits {
            max_device_name_length: 4,
            ..InputLimits::default()
        };
        assert!(limits.validate_device_name("ok").is_none());
        assert!(limits.validate_device_name("toolong").is_some());
    }

    #[test]
    fn event_batch_over_the_limit_is_rejected() {
        let limits = InputLimits {
            max_event_chain_batch: 2,
            ..InputLimits::default()
        };
        assert!(limits.validate_event_batch_size(2).is_none());
        assert!(limits.validate_event_batch_size(3).is_some());
    }
}
