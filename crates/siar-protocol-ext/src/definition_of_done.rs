//! spec §106 "Definition of Done" — Part 01's own sixteen-item
//! completion checklist, turned into a real, checkable self-audit
//! rather than left as sixteen bullet points nobody re-verifies.
//! [`is_satisfied`] is answered honestly, including the two items this
//! crate does NOT yet satisfy — see their own match arms below for
//! why. A completion checklist that can't say "not yet" to some of its
//! own items isn't actually being checked.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DefinitionOfDoneItem {
    MessagingAndFilesRegisterIndependently,
    FileOnlyPeersWorkWithoutMessaging,
    UnknownOptionalExtensionsDoNotBreakSessions,
    IncompatibleRequiredExtensionsFailDeterministically,
    CapabilityIntersectionIsNegotiated,
    FrameSizesAreBounded,
    QueuesAreBounded,
    TrafficPriorityExists,
    ProtocolIdsAreStable,
    WireAndDomainTypesAreSeparate,
    ExtensionsCannotAccessUnrestrictedRuntimeState,
    CompatibilityTestsExist,
    FuzzTestsExist,
    DiagnosticsShowNegotiatedExtensions,
    ReconnectBehaviorIsDeterministic,
    HeavyExtensionsCanLazyOpenOnMobile,
}

impl DefinitionOfDoneItem {
    /// Returns `(satisfied, why)`. `why` always points at the real
    /// module/test that backs the claim — or, for the two `false`
    /// cases, names exactly what's missing.
    pub fn status(self) -> (bool, &'static str) {
        use DefinitionOfDoneItem::*;
        match self {
            // Genuinely NOT true yet at the concrete level: no
            // MessagingExtension/FileExtension types exist in this
            // workspace (this crate is deliberately standalone — see
            // "No wire integration"). True only in the abstract sense
            // that ExtensionDescriptor supports independent
            // registration for any two protocols, demonstrated by
            // examples.rs's spec_77 composition test — not by an
            // actual messaging/files pair.
            MessagingAndFilesRegisterIndependently => (
                false,
                "no concrete MessagingExtension/FileExtension exist to register — only the generic mechanism (examples::spec_77 composition test) is proven",
            ),
            FileOnlyPeersWorkWithoutMessaging => (
                false,
                "same gap — config::spec_86 proves the *mechanism* (messaging negotiates with no files extension present), not a real file-only peer",
            ),
            UnknownOptionalExtensionsDoNotBreakSessions => (
                true,
                "negotiation::missing_optional_extension_does_not_fail_session",
            ),
            IncompatibleRequiredExtensionsFailDeterministically => (
                true,
                "negotiation::spec_60_major_version_incompatibility_fails_cleanly_for_a_required_extension",
            ),
            CapabilityIntersectionIsNegotiated => (
                true,
                "negotiation::matches_spec_10_example",
            ),
            FrameSizesAreBounded => (
                true,
                "framing::a_hostile_oversized_length_is_rejected_before_any_allocation",
            ),
            QueuesAreBounded => (
                true,
                "backpressure::BoundedQueue, scheduler::every_tier_queue_is_bounded_including_critical",
            ),
            TrafficPriorityExists => (true, "lifecycle::TrafficPriority"),
            ProtocolIdsAreStable => (true, "identifier::ProtocolId, canonical_round_trip test"),
            WireAndDomainTypesAreSeparate => (
                true,
                "framing::FrameHeader (wire) vs descriptor::ExtensionDescriptor (domain) — never the same type",
            ),
            // Genuinely NOT fully true: ExtensionContext's
            // identity/session/scheduler/resources fields are named
            // placeholder types (see registry.rs's own doc comment) —
            // there's no real enforcement of restricted runtime state
            // access yet, because there's no real runtime state to
            // restrict access to.
            ExtensionsCannotAccessUnrestrictedRuntimeState => (
                false,
                "registry::ExtensionContext's fields are still placeholder types — real scoping needs real shared-service handles, not yet wired",
            ),
            CompatibilityTestsExist => (
                true,
                "negotiation's spec_60_* tests — the actual matrix spec §60 asks for",
            ),
            // Genuinely NOT true: no cargo-fuzz targets exist (see
            // §62's own honest "not attempted" note).
            FuzzTestsExist => (
                false,
                "§62 was explicitly not attempted this pass — needs cargo-fuzz infrastructure",
            ),
            DiagnosticsShowNegotiatedExtensions => (
                true,
                "observability::render_diagnostics",
            ),
            ReconnectBehaviorIsDeterministic => (
                true,
                "handshake::ReconnectionRevalidation — safe_to_reuse_cached_negotiation requires all three checks, no partial/nondeterministic path",
            ),
            HeavyExtensionsCanLazyOpenOnMobile => (
                true,
                "lifecycle::LazyInitTarget, negotiate() never itself advances past Negotiated",
            ),
        }
    }
}

/// A custom application extension being addable without changing the
/// core protocol — spec §106's own sixteenth item — is covered by
/// `examples::spec_71_a_third_party_namespace_negotiates_exactly_like_a_core_extension`
/// and isn't repeated as a `DefinitionOfDoneItem` variant here only
/// because it's identical in substance to that existing, already-cited
/// test; nothing new to check that isn't already checked there.
pub const CUSTOM_APP_EXTENSIONS_WITHOUT_CORE_CHANGES_NOTE: &str =
    "see examples::spec_71_a_third_party_namespace_negotiates_exactly_like_a_core_extension";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_106_exactly_four_items_are_honestly_not_yet_satisfied() {
        let all = [
            DefinitionOfDoneItem::MessagingAndFilesRegisterIndependently,
            DefinitionOfDoneItem::FileOnlyPeersWorkWithoutMessaging,
            DefinitionOfDoneItem::UnknownOptionalExtensionsDoNotBreakSessions,
            DefinitionOfDoneItem::IncompatibleRequiredExtensionsFailDeterministically,
            DefinitionOfDoneItem::CapabilityIntersectionIsNegotiated,
            DefinitionOfDoneItem::FrameSizesAreBounded,
            DefinitionOfDoneItem::QueuesAreBounded,
            DefinitionOfDoneItem::TrafficPriorityExists,
            DefinitionOfDoneItem::ProtocolIdsAreStable,
            DefinitionOfDoneItem::WireAndDomainTypesAreSeparate,
            DefinitionOfDoneItem::ExtensionsCannotAccessUnrestrictedRuntimeState,
            DefinitionOfDoneItem::CompatibilityTestsExist,
            DefinitionOfDoneItem::FuzzTestsExist,
            DefinitionOfDoneItem::DiagnosticsShowNegotiatedExtensions,
            DefinitionOfDoneItem::ReconnectBehaviorIsDeterministic,
            DefinitionOfDoneItem::HeavyExtensionsCanLazyOpenOnMobile,
        ];
        let unsatisfied: Vec<_> = all.iter().filter(|item| !item.status().0).collect();
        assert_eq!(
            unsatisfied.len(),
            4,
            "this test itself must be updated the moment any of these four genuinely becomes true — \
             not adjusted to make the count pass"
        );
    }
}
