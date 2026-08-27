//! §31 "Per-Extension Quotas", §32 "Extension Admission".
//!
//! Mirrors `peer_quota.rs`'s shape deliberately — same problem
//! (bounded per-owner resource consumption that must not let one
//! owner monopolize a shared runtime), same solution shape (a limits
//! struct, a usage-counters struct with matching field names, a
//! `try_charge` that reuses this crate's own §21-22 durable/ephemeral
//! split). Kept as a separate module rather than a generic type
//! parameterized over "peer or extension," because the two limit
//! structs' fields don't actually line up (`PeerQuota`'s five
//! dimensions are peer-traffic-shaped — pending messages, staged
//! files, DTN relay bytes; `ExtensionResourceLimits`'s four are
//! resource-shaped — memory, queue depth, streams, storage) and
//! forcing a shared generic over mismatched fields would cost more
//! clarity than the deduplication would save.

use crate::admission::{AdmissionResult, DeferredReason, DropReason};
use serde::{Deserialize, Serialize};

/// §32, verbatim field list. §31 names five budget *categories*
/// (memory, queue, stream, storage, CPU) but §32's own concrete struct
/// — the thing extension registration actually declares — only ever
/// materializes four of them as fields; there's no `max_cpu_work`
/// field to transcribe. That's consistent with this crate's own
/// `types::ResourceBudget`, which likewise has no CPU field for
/// exactly the reason §8 gives elsewhere in this same spec: CPU needs
/// a coarse class (see `crate::types`'s own doc comment, and §34's
/// `CpuWorkClass` two sections later), not a byte-shaped number, so
/// there is nothing here to omit by mistake — the fourth field
/// (queue budget) is `max_queued_ops`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionResourceLimits {
    pub max_memory_bytes: u64,
    pub max_queued_ops: u32,
    pub max_streams: u16,
    pub max_storage_bytes: u64,
}

impl ExtensionResourceLimits {
    /// §32's own closing line: "Runtime can tighten these." An
    /// extension's registration-time declaration is a ceiling it's
    /// asking for, not a grant — the runtime may impose its own
    /// (possibly stricter) cap, and the *effective* limit is always
    /// whichever of the two is smaller, field by field, never a
    /// blend or an average that could exceed either side's own idea
    /// of what's safe.
    pub fn tightened_by(&self, runtime_cap: &Self) -> Self {
        Self {
            max_memory_bytes: self.max_memory_bytes.min(runtime_cap.max_memory_bytes),
            max_queued_ops: self.max_queued_ops.min(runtime_cap.max_queued_ops),
            max_streams: self.max_streams.min(runtime_cap.max_streams),
            max_storage_bytes: self.max_storage_bytes.min(runtime_cap.max_storage_bytes),
        }
    }
}

/// Live outstanding usage for one extension, mirroring
/// [`ExtensionResourceLimits`]'s field names 1:1 — same reasoning
/// `peer_quota::PeerUsageCounters` already documents for doing the
/// same against [`crate::peer_quota::PeerQuota`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionUsageCounters {
    pub memory_bytes: u64,
    pub queued_ops: u32,
    pub streams: u16,
    pub storage_bytes: u64,
}

/// What's being requested against an extension's quota.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExtensionUsageDelta {
    pub memory_bytes: u64,
    pub queued_ops: u32,
    pub streams: u16,
    pub storage_bytes: u64,
}

impl ExtensionUsageCounters {
    /// Admits `delta` against `limits` (typically the result of
    /// [`ExtensionResourceLimits::tightened_by`], not the extension's
    /// raw declared limits) if every one of the four dimensions would
    /// stay within its own max after charging — checked independently,
    /// so an extension maxed out on `queued_ops` alone can still be
    /// blocked even with plenty of memory/stream/storage headroom
    /// left, the same independence
    /// [`crate::peer_quota::PeerUsageCounters::try_charge`] already
    /// enforces for peer quotas.
    pub fn try_charge(
        &mut self,
        limits: &ExtensionResourceLimits,
        delta: ExtensionUsageDelta,
        durable: bool,
    ) -> AdmissionResult {
        let fits = self.memory_bytes.saturating_add(delta.memory_bytes) <= limits.max_memory_bytes
            && self.queued_ops.saturating_add(delta.queued_ops) <= limits.max_queued_ops
            && self.streams.saturating_add(delta.streams) <= limits.max_streams
            && self.storage_bytes.saturating_add(delta.storage_bytes) <= limits.max_storage_bytes;

        if !fits {
            return if durable {
                AdmissionResult::Deferred(DeferredReason::AwaitingBudget)
            } else {
                AdmissionResult::Dropped(DropReason::Stale)
            };
        }

        self.memory_bytes += delta.memory_bytes;
        self.queued_ops += delta.queued_ops;
        self.streams += delta.streams;
        self.storage_bytes += delta.storage_bytes;
        AdmissionResult::Accepted
    }

    /// Gives back previously-charged usage — saturating, so a
    /// double-release can't underflow into a wraparound count.
    pub fn release(&mut self, delta: ExtensionUsageDelta) {
        self.memory_bytes = self.memory_bytes.saturating_sub(delta.memory_bytes);
        self.queued_ops = self.queued_ops.saturating_sub(delta.queued_ops);
        self.streams = self.streams.saturating_sub(delta.streams);
        self.storage_bytes = self.storage_bytes.saturating_sub(delta.storage_bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared() -> ExtensionResourceLimits {
        ExtensionResourceLimits {
            max_memory_bytes: 64 * 1024 * 1024,
            max_queued_ops: 1000,
            max_streams: 32,
            max_storage_bytes: 256 * 1024 * 1024,
        }
    }

    #[test]
    fn tightened_by_takes_the_smaller_of_each_field_independently() {
        let declared = declared();
        let runtime_cap = ExtensionResourceLimits {
            max_memory_bytes: 16 * 1024 * 1024,    // stricter than declared
            max_queued_ops: 5000,                  // looser than declared
            max_streams: 32,                       // equal
            max_storage_bytes: 1024 * 1024 * 1024, // looser than declared
        };

        let effective = declared.tightened_by(&runtime_cap);
        assert_eq!(effective.max_memory_bytes, 16 * 1024 * 1024); // runtime wins
        assert_eq!(effective.max_queued_ops, 1000); // declared wins
        assert_eq!(effective.max_streams, 32);
        assert_eq!(effective.max_storage_bytes, 256 * 1024 * 1024); // declared wins
    }

    #[test]
    fn tightened_never_exceeds_either_side_even_when_runtime_is_more_generous_everywhere() {
        let declared = declared();
        let generous_runtime = ExtensionResourceLimits {
            max_memory_bytes: u64::MAX,
            max_queued_ops: u32::MAX,
            max_streams: u16::MAX,
            max_storage_bytes: u64::MAX,
        };
        // A generous runtime cap must never let an extension exceed
        // what it itself declared wanting.
        assert_eq!(declared.tightened_by(&generous_runtime), declared);
    }

    #[test]
    fn charge_within_limits_succeeds_and_updates_counters() {
        let limits = declared();
        let mut usage = ExtensionUsageCounters::default();
        let delta = ExtensionUsageDelta {
            streams: 2,
            ..Default::default()
        };

        assert_eq!(
            usage.try_charge(&limits, delta, true),
            AdmissionResult::Accepted
        );
        assert_eq!(usage.streams, 2);
    }

    #[test]
    fn durable_charge_past_limit_defers_without_mutating_usage() {
        let limits = declared(); // max_streams: 32
        let mut usage = ExtensionUsageCounters {
            streams: 32,
            ..Default::default()
        };
        let delta = ExtensionUsageDelta {
            streams: 1,
            ..Default::default()
        };

        let result = usage.try_charge(&limits, delta, true);
        assert_eq!(
            result,
            AdmissionResult::Deferred(DeferredReason::AwaitingBudget)
        );
        assert_eq!(usage.streams, 32); // unchanged
    }

    #[test]
    fn non_durable_charge_past_limit_drops_instead_of_deferring() {
        let limits = declared();
        let mut usage = ExtensionUsageCounters {
            streams: 32,
            ..Default::default()
        };
        let delta = ExtensionUsageDelta {
            streams: 1,
            ..Default::default()
        };

        assert_eq!(
            usage.try_charge(&limits, delta, false),
            AdmissionResult::Dropped(DropReason::Stale)
        );
    }

    #[test]
    fn each_limit_dimension_is_enforced_independently() {
        // An extension maxed out on queued_ops must be blocked even
        // with plenty of memory/stream/storage headroom left.
        let limits = declared(); // max_queued_ops: 1000
        let mut usage = ExtensionUsageCounters {
            queued_ops: 1000,
            ..Default::default()
        };
        let delta = ExtensionUsageDelta {
            queued_ops: 1,
            ..Default::default()
        };

        assert_eq!(
            usage.try_charge(&limits, delta, true),
            AdmissionResult::Deferred(DeferredReason::AwaitingBudget)
        );
    }

    #[test]
    fn release_gives_back_charged_capacity() {
        let limits = declared();
        let mut usage = ExtensionUsageCounters::default();
        let delta = ExtensionUsageDelta {
            memory_bytes: 1024,
            ..Default::default()
        };

        usage.try_charge(&limits, delta, true);
        assert_eq!(usage.memory_bytes, 1024);
        usage.release(delta);
        assert_eq!(usage.memory_bytes, 0);
    }

    #[test]
    fn one_extension_monopolizing_its_own_quota_does_not_affect_a_second_extensions_counters() {
        // §31's whole point: extensions must not monopolize *shared*
        // resources — demonstrated here at the unit level by two
        // fully independent `ExtensionUsageCounters` instances never
        // interacting, which is what makes per-extension isolation
        // possible for a caller keyed by `ProtocolId`.
        let limits = declared();
        let mut noisy = ExtensionUsageCounters {
            streams: 32,
            ..Default::default()
        };
        let mut quiet = ExtensionUsageCounters::default();

        let delta = ExtensionUsageDelta {
            streams: 1,
            ..Default::default()
        };
        assert_eq!(
            noisy.try_charge(&limits, delta, true),
            AdmissionResult::Deferred(DeferredReason::AwaitingBudget)
        );
        assert_eq!(
            quiet.try_charge(&limits, delta, true),
            AdmissionResult::Accepted
        );
    }
}
