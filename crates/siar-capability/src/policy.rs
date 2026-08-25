//! §20 "Policy Filter", §21 "Hard Policy", §22 "User Policy", §23
//! "Application Policy".

use crate::id::CapabilityId;
use std::collections::HashSet;

/// §20: "Negotiation is: local support ∩ remote support ∩ local
/// security policy ∩ application policy ∩ runtime platform
/// availability." This crate implements the two policy layers that
/// are pure per-id disable lists — §21 hard policy and §22 user
/// policy — plus §23 application policy, which the spec's own
/// examples (ERP: "DTN disabled"; Emergency app: "DTN required")
/// show is the *same shape* of per-id allow/disallow, just sourced
/// from the embedding application rather than the user. "runtime
/// platform availability" (the fourth ∩ term) is not a policy at all
/// in this crate's sense — it's a live fact about the device (Wi-Fi
/// currently on, camera currently free) that belongs with the
/// dynamic/ephemeral capability machinery (§17, §45), not with a
/// static policy object, so it's not modeled here.
///
/// §21 is explicit that hard policy "cannot be overridden by peer
/// advertisement" — [`crate::negotiate::negotiate`] enforces this by
/// applying the merged disabled set *after* intersection, never
/// letting a remote-only capability route around it, and by treating
/// the removal of a capability either side had marked `Required` as a
/// hard failure rather than a silent drop (§8's required/optional
/// split still applies even when the reason for absence is policy,
/// not lack of support).
#[derive(Debug, Clone, Default)]
pub struct CapabilityPolicy {
    hard_disabled: HashSet<CapabilityId>,
    user_disabled: HashSet<CapabilityId>,
    app_disabled: HashSet<CapabilityId>,
}

impl CapabilityPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn disable_hard(&mut self, id: CapabilityId) {
        self.hard_disabled.insert(id);
    }

    pub fn disable_user(&mut self, id: CapabilityId) {
        self.user_disabled.insert(id);
    }

    pub fn disable_app(&mut self, id: CapabilityId) {
        self.app_disabled.insert(id);
    }

    pub fn is_disabled(&self, id: &CapabilityId) -> bool {
        self.hard_disabled.contains(id)
            || self.user_disabled.contains(id)
            || self.app_disabled.contains(id)
    }
}
