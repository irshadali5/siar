//! §121 "Device-Specific Authorization": "applications may authorize
//! per device... Bob's laptop can receive 2 GB file, Bob's phone
//! should receive only metadata... this can combine device
//! capability, user policy, network policy."
//!
//! Deliberately a pure combinator, not a policy engine: this crate has
//! no opinion on what a "user policy" or "network policy" actually
//! contains (spec names them only as categories, not shapes), so
//! [`DeviceAuthorizationDecision::combine`] takes each as an
//! already-decided `bool`/limit from whatever owns that concern
//! ([`crate::capability::DeviceCapabilitySet`] for the device-capability
//! input; an application's own settings for user policy; a transport
//! crate for network policy) and only does the one thing that's
//! actually this crate's job: making sure a capability the device
//! itself doesn't have can never be granted no matter how permissive
//! the other two inputs are.

use crate::capability::DeviceCapabilitySet;

/// §121's own worked example, generalized to a size limit rather than
/// hard-coding "file" — the same shape covers "receive only metadata"
/// (`Some(0)`, or more precisely a caller mapping "metadata only" to
/// whatever byte ceiling that means for their protocol) and "2 GB
/// file" (`Some(2 * 1024 * 1024 * 1024)`) as the same kind of value,
/// not two different mechanisms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceAuthorizationDecision {
    pub allowed: bool,
    /// `None` means "no additional size ceiling beyond whatever the
    /// three inputs otherwise imply" — distinct from `Some(0)`, which
    /// really does mean zero bytes of payload (e.g. "metadata only").
    pub max_payload_bytes: Option<u64>,
}

impl DeviceAuthorizationDecision {
    /// §121's actual combination rule: the device's own
    /// [`DeviceCapabilitySet`] is a hard ceiling, not one vote among
    /// three. If `required_capability` isn't present, `allowed` is
    /// `false` regardless of what `user_policy_allows`/
    /// `network_policy_allows` say — no combination of application and
    /// network policy can authorize a device to do something it
    /// structurally cannot (e.g. an old device build that predates a
    /// capability bit). Only once the capability gate passes do the
    /// other two inputs get a say, and the size limit is the tightest
    /// of whichever caller-supplied limits are actually `Some`.
    pub fn combine(
        device_capabilities: DeviceCapabilitySet,
        required_capability: DeviceCapabilitySet,
        user_policy_allows: bool,
        network_policy_allows: bool,
        user_policy_max_bytes: Option<u64>,
        network_policy_max_bytes: Option<u64>,
    ) -> Self {
        let has_capability =
            (device_capabilities.0 & required_capability.0) == required_capability.0;
        let allowed = has_capability && user_policy_allows && network_policy_allows;

        let max_payload_bytes = match (user_policy_max_bytes, network_policy_max_bytes) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        Self {
            allowed,
            max_payload_bytes: if allowed { max_payload_bytes } else { Some(0) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_device_capability_is_never_overridden_by_permissive_policy() {
        let device_capabilities = DeviceCapabilitySet(0); // has nothing
        let required = DeviceCapabilitySet::RECEIVE_MESSAGE;

        let decision = DeviceAuthorizationDecision::combine(
            device_capabilities,
            required,
            true,
            true,
            Some(2 * 1024 * 1024 * 1024),
            Some(2 * 1024 * 1024 * 1024),
        );

        assert!(!decision.allowed);
        assert_eq!(decision.max_payload_bytes, Some(0));
    }

    #[test]
    fn bobs_laptop_and_phone_get_different_limits_from_the_same_capability_set() {
        let full_capable = DeviceCapabilitySet::RECEIVE_MESSAGE;
        let required = DeviceCapabilitySet::RECEIVE_MESSAGE;

        // Laptop: user policy allows full files.
        let laptop = DeviceAuthorizationDecision::combine(
            full_capable,
            required,
            true,
            true,
            Some(2 * 1024 * 1024 * 1024),
            None,
        );
        assert!(laptop.allowed);
        assert_eq!(laptop.max_payload_bytes, Some(2 * 1024 * 1024 * 1024));

        // Phone: same device capability set, but user policy caps it
        // to metadata-only.
        let phone =
            DeviceAuthorizationDecision::combine(full_capable, required, true, true, Some(0), None);
        assert!(phone.allowed);
        assert_eq!(phone.max_payload_bytes, Some(0));
    }

    #[test]
    fn tightest_limit_wins_when_both_policies_specify_one() {
        let capabilities = DeviceCapabilitySet::RECEIVE_MESSAGE;
        let decision = DeviceAuthorizationDecision::combine(
            capabilities,
            capabilities,
            true,
            true,
            Some(1000),
            Some(200),
        );
        assert_eq!(decision.max_payload_bytes, Some(200));
    }

    #[test]
    fn network_policy_can_veto_even_when_device_and_user_policy_allow() {
        let capabilities = DeviceCapabilitySet::RECEIVE_MESSAGE;
        let decision = DeviceAuthorizationDecision::combine(
            capabilities,
            capabilities,
            true,
            false,
            None,
            None,
        );
        assert!(!decision.allowed);
    }
}
