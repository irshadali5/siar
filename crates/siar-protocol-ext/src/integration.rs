//! spec §87 "Integration Interfaces", §88 "Core vs First-Party
//! Extensions", §89 "Extension Provenance", §90 "Interoperability
//! Specification".

use crate::identifier::ProtocolId;

/// spec §87's own `ContentReference`/`ResolvedContent`/`ResolveError` —
/// referenced but never defined elsewhere in this document, so kept
/// minimal here rather than guessed at in detail: `ContentReference`
/// wraps whatever opaque reference bytes messaging would have gotten
/// from files (spec §74: "attachments are represented as content/blob
/// references"), `ResolvedContent` is the resolved bytes, and
/// `ResolveError` carries only a reason string rather than an invented
/// taxonomy of failure modes this document never lists.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ContentReference(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResolvedContent(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, serde::Serialize, serde::Deserialize)]
#[error("content resolution failed: {reason}")]
pub struct ResolveError {
    pub reason: String,
}

/// spec §87, verbatim trait shape ("Messaging can depend on an
/// abstract resolver... the file subsystem can provide an
/// implementation. This preserves modularity") — the concrete
/// mechanism behind spec §86's "messaging itself must continue working
/// without files": messaging depends on this trait, never on
/// `siar-messaging`/a files crate directly, so a build with no file
/// extension present at all just has no `ContentResolver`
/// implementation registered, not a compile-time dependency on files
/// existing.
pub trait ContentResolver {
    fn resolve(
        &self,
        reference: ContentReference,
    ) -> impl std::future::Future<Output = Result<ResolvedContent, ResolveError>> + Send;
}

/// spec §88, verbatim four-tier classification. Deliberately a
/// separate axis from [`crate::descriptor::ExtensionStability`] (§47)
/// — stability is about API maturity (a `CoreMaintained` extension can
/// still be `Experimental`), this is about who maintains it and how
/// central it is to the product. Spec §88's own point —
/// "messaging should not be forced into the core merely because the
/// flagship application uses it" — is exactly why these two axes must
/// stay independent: conflating them would make "the app I'm building
/// uses this a lot" (a `FirstPartyOptional`/`CoreMaintained` question)
/// silently imply "this is battle-tested" (a stability question), and
/// vice versa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExtensionClassification {
    CoreMaintained,
    FirstPartyOptional,
    ThirdParty,
    Experimental,
}

/// spec §89's own four-item list, "sufficient initially" for
/// compile-time extensions (spec §49's already-reconciled recommended
/// loading model) — verbatim, with the explicit "not runtime plugin
/// signing" scope spec §89 itself draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProvenanceControl {
    CrateReview,
    SupplyChainControls,
    BinarySigning,
    ReleaseProvenance,
}

/// spec §90's own nine-item documentation checklist for a stable
/// extension, as real boolean gates rather than only a prose list —
/// [`Self::is_complete`] is the concrete form of spec §90's own point:
/// "this allows non-Rust implementations later" only holds once every
/// one of these nine is actually true, not most of them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InteropDocumentationChecklist {
    pub protocol_id_documented: bool,
    pub versions_documented: bool,
    pub capabilities_documented: bool,
    pub wire_schema_documented: bool,
    pub state_machine_documented: bool,
    pub limits_documented: bool,
    pub security_rules_documented: bool,
    pub error_codes_documented: bool,
    pub test_vectors_documented: bool,
}

impl InteropDocumentationChecklist {
    pub fn is_complete(&self) -> bool {
        self.protocol_id_documented
            && self.versions_documented
            && self.capabilities_documented
            && self.wire_schema_documented
            && self.state_machine_documented
            && self.limits_documented
            && self.security_rules_documented
            && self.error_codes_documented
            && self.test_vectors_documented
    }
}

/// Pairs a [`ProtocolId`] with its [`InteropDocumentationChecklist`] —
/// spec §90 applies per stable extension, not once for the whole
/// crate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExtensionInteropStatus {
    pub extension: ProtocolId,
    pub checklist: InteropDocumentationChecklist,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::{NamespaceId, ProtocolMajor, ProtocolName};

    struct StubFileResolver;
    impl ContentResolver for StubFileResolver {
        async fn resolve(&self, reference: ContentReference) -> Result<ResolvedContent, ResolveError> {
            if reference.0.is_empty() {
                return Err(ResolveError {
                    reason: "empty reference".to_string(),
                });
            }
            Ok(ResolvedContent(reference.0))
        }
    }

    #[tokio::test]
    async fn spec_87_a_files_provided_resolver_satisfies_the_abstract_trait() {
        let resolver = StubFileResolver;
        let result = resolver.resolve(ContentReference(vec![1, 2, 3])).await.unwrap();
        assert_eq!(result.0, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn spec_87_resolve_failure_carries_a_reason() {
        let resolver = StubFileResolver;
        let err = resolver.resolve(ContentReference(vec![])).await.unwrap_err();
        assert!(!err.reason.is_empty());
    }

    #[test]
    fn spec_88_classification_is_independent_of_stability() {
        // Structural claim: constructing an ExtensionClassification
        // requires nothing from ExtensionStability, and vice versa —
        // proven by this compiling with no shared type between them.
        let classification = ExtensionClassification::FirstPartyOptional;
        let stability = crate::descriptor::ExtensionStability::Experimental;
        assert_eq!(classification, ExtensionClassification::FirstPartyOptional);
        assert_eq!(stability, crate::descriptor::ExtensionStability::Experimental);
    }

    #[test]
    fn spec_90_checklist_is_incomplete_until_all_nine_are_true() {
        let mut checklist = InteropDocumentationChecklist::default();
        assert!(!checklist.is_complete());

        checklist.protocol_id_documented = true;
        checklist.versions_documented = true;
        checklist.capabilities_documented = true;
        checklist.wire_schema_documented = true;
        checklist.state_machine_documented = true;
        checklist.limits_documented = true;
        checklist.security_rules_documented = true;
        checklist.error_codes_documented = true;
        assert!(
            !checklist.is_complete(),
            "eight of nine must not count as complete"
        );

        checklist.test_vectors_documented = true;
        assert!(checklist.is_complete());
    }

    #[test]
    fn spec_90_status_pairs_a_real_protocol_id_with_its_checklist() {
        let status = ExtensionInteropStatus {
            extension: ProtocolId::new(
                NamespaceId::new("org.example").unwrap(),
                ProtocolName::new("messaging").unwrap(),
                ProtocolMajor(1),
            ),
            checklist: InteropDocumentationChecklist::default(),
        };
        assert!(!status.checklist.is_complete());
    }
}
