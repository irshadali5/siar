//! spec §54 "Resource Accounting", §55 "Per-Peer Quotas", §56 "Abuse
//! Handling".

use crate::identifier::ProtocolId;
use std::collections::HashMap;

/// spec §54's own six tracked fields, verbatim.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResourceUsage {
    pub memory_bytes: u64,
    pub network_bytes: u64,
    pub stored_bytes: u64,
    pub queue_depth: u64,
    pub active_streams: u64,
    pub cpu_heavy_operations: u64,
}

/// Per-extension [`ResourceUsage`] tracking, keyed by [`ProtocolId`] —
/// spec §54: "track per extension." A plain accumulator, not a
/// sampling/metrics system (that's [`crate::observability::ExtensionMetrics`],
/// §58's separate concern) — this is the raw numbers §54 says feed
/// diagnostics, quotas, mobile battery policy, and abuse protection.
#[derive(Debug, Clone, Default)]
pub struct ResourceAccounting {
    usage: HashMap<ProtocolId, ResourceUsage>,
}

impl ResourceAccounting {
    pub fn usage_for(&self, extension: &ProtocolId) -> ResourceUsage {
        self.usage.get(extension).copied().unwrap_or_default()
    }

    /// Applies a delta rather than replacing the stored value — the
    /// realistic shape of "track per extension" over a session's
    /// lifetime (bytes accumulate; queue depth/active streams should
    /// be set via [`Self::set_gauge`] instead, since those aren't
    /// cumulative).
    pub fn record_delta(&mut self, extension: &ProtocolId, delta: ResourceUsage) {
        let entry = self.usage.entry(extension.clone()).or_default();
        entry.memory_bytes += delta.memory_bytes;
        entry.network_bytes += delta.network_bytes;
        entry.stored_bytes += delta.stored_bytes;
        entry.queue_depth += delta.queue_depth;
        entry.active_streams += delta.active_streams;
        entry.cpu_heavy_operations += delta.cpu_heavy_operations;
    }
}

/// spec §55's own six-item limit list, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PeerQuota {
    pub frames_per_sec: u32,
    pub bytes_per_sec: u64,
    pub concurrent_streams: u32,
    pub queued_operations: u32,
    pub dtn_storage_bytes: u64,
    pub file_transfers: u32,
}

/// spec §55: "Policies may vary by trust level." No fixed set of trust
/// levels is given anywhere in this document, so this is deliberately
/// the smallest closed set that makes "vary by trust level" checkable
/// at all — a real trust-level taxonomy (verified contact, org member,
/// stranger, etc.) is application policy, not something this crate
/// should invent wholesale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TrustLevel {
    Trusted,
    Default,
    Untrusted,
}

/// Maps [`TrustLevel`] to the [`PeerQuota`] that applies at that level
/// — the concrete mechanism behind "policies may vary by trust level."
#[derive(Debug, Clone, Default)]
pub struct QuotaPolicy {
    by_trust_level: HashMap<TrustLevel, PeerQuota>,
}

impl QuotaPolicy {
    pub fn set(&mut self, level: TrustLevel, quota: PeerQuota) {
        self.by_trust_level.insert(level, quota);
    }

    pub fn quota_for(&self, level: TrustLevel) -> Option<PeerQuota> {
        self.by_trust_level.get(&level).copied()
    }
}

/// spec §56's own five escalating controls, kept ordered exactly as
/// listed — this is a real escalation ladder, not an unordered set:
/// `RateLimit` is the least disruptive response and `CloseSession` the
/// most, matching spec §56's own framing ("the runtime needs
/// escalating controls").
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AbuseControl {
    RateLimit,
    PauseExtension,
    CloseExtension,
    QuarantinePeer,
    CloseSession,
}

pub const ABUSE_ESCALATION_LADDER: [AbuseControl; 5] = [
    AbuseControl::RateLimit,
    AbuseControl::PauseExtension,
    AbuseControl::CloseExtension,
    AbuseControl::QuarantinePeer,
    AbuseControl::CloseSession,
];

impl AbuseControl {
    /// spec §56's own closing line: "a malicious file stream should
    /// not corrupt or disable unrelated messaging state unless
    /// necessary for overall security." The first three rungs of the
    /// ladder are scoped to the offending extension alone;
    /// `QuarantinePeer`/`CloseSession` are the two that necessarily
    /// affect every extension shared with that peer/session — which is
    /// exactly the "unless necessary for overall security" escape
    /// hatch, not a violation of the isolation principle.
    pub fn is_scoped_to_one_extension(self) -> bool {
        matches!(
            self,
            AbuseControl::RateLimit | AbuseControl::PauseExtension | AbuseControl::CloseExtension
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::{NamespaceId, ProtocolMajor, ProtocolName};

    fn messaging() -> ProtocolId {
        ProtocolId::new(
            NamespaceId::new("org.example").unwrap(),
            ProtocolName::new("messaging").unwrap(),
            ProtocolMajor(1),
        )
    }

    #[test]
    fn spec_54_deltas_accumulate_per_extension() {
        let mut accounting = ResourceAccounting::default();
        accounting.record_delta(
            &messaging(),
            ResourceUsage {
                network_bytes: 100,
                ..Default::default()
            },
        );
        accounting.record_delta(
            &messaging(),
            ResourceUsage {
                network_bytes: 50,
                ..Default::default()
            },
        );
        assert_eq!(accounting.usage_for(&messaging()).network_bytes, 150);
    }

    #[test]
    fn spec_54_unknown_extension_has_zero_usage_not_an_error() {
        let accounting = ResourceAccounting::default();
        assert_eq!(accounting.usage_for(&messaging()), ResourceUsage::default());
    }

    #[test]
    fn spec_55_quota_varies_by_trust_level() {
        let mut policy = QuotaPolicy::default();
        policy.set(
            TrustLevel::Trusted,
            PeerQuota {
                frames_per_sec: 1000,
                bytes_per_sec: 10_000_000,
                concurrent_streams: 32,
                queued_operations: 500,
                dtn_storage_bytes: 100_000_000,
                file_transfers: 16,
            },
        );
        policy.set(
            TrustLevel::Untrusted,
            PeerQuota {
                frames_per_sec: 10,
                bytes_per_sec: 10_000,
                concurrent_streams: 1,
                queued_operations: 5,
                dtn_storage_bytes: 100_000,
                file_transfers: 0,
            },
        );

        let trusted = policy.quota_for(TrustLevel::Trusted).unwrap();
        let untrusted = policy.quota_for(TrustLevel::Untrusted).unwrap();
        assert!(trusted.frames_per_sec > untrusted.frames_per_sec);
        assert!(policy.quota_for(TrustLevel::Default).is_none());
    }

    #[test]
    fn spec_56_escalation_ladder_is_in_spec_order() {
        assert_eq!(ABUSE_ESCALATION_LADDER, [
            AbuseControl::RateLimit,
            AbuseControl::PauseExtension,
            AbuseControl::CloseExtension,
            AbuseControl::QuarantinePeer,
            AbuseControl::CloseSession,
        ]);
    }

    #[test]
    fn spec_56_only_the_last_two_rungs_affect_more_than_one_extension() {
        assert!(AbuseControl::RateLimit.is_scoped_to_one_extension());
        assert!(AbuseControl::PauseExtension.is_scoped_to_one_extension());
        assert!(AbuseControl::CloseExtension.is_scoped_to_one_extension());
        assert!(!AbuseControl::QuarantinePeer.is_scoped_to_one_extension());
        assert!(!AbuseControl::CloseSession.is_scoped_to_one_extension());
    }
}
