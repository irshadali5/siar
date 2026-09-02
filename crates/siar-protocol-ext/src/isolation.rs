//! spec §96 "Scheduler Contract", §97 "Storage Isolation", §98
//! "Metrics Isolation", §99 "Capability Isolation".

use crate::identifier::ProtocolId;
use crate::routing::DeliveryClass;
use crate::security::PeerIdentity;

/// spec §96: "Extensions submit: priority, deadline, payload size,
/// delivery class, peer, durability. The central scheduler chooses:
/// transport, queue, retry policy." This struct is deliberately only
/// the first half — same discipline as
/// [`crate::routing::RoutingRequirements`] (§53): no field here could
/// ever name a transport, a queue, or a retry policy, because "the
/// central scheduler chooses" those, not the extension submitting the
/// work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SchedulingSubmission {
    pub priority: crate::lifecycle::TrafficPriority,
    pub deadline_millis: Option<u64>,
    pub payload_size_bytes: usize,
    pub delivery_class: DeliveryClass,
    pub peer: PeerIdentity,
    pub durable: bool,
}

/// spec §97's own five namespaces, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StorageNamespace {
    Messaging,
    Files,
    Presence,
    Emergency,
    Custom,
}

impl StorageNamespace {
    /// The literal namespace prefix spec §97 shows (`messaging/`,
    /// `files/`, ...) — real enough to actually prefix a storage key
    /// with, not just a label.
    pub fn prefix(self) -> &'static str {
        match self {
            StorageNamespace::Messaging => "messaging/",
            StorageNamespace::Files => "files/",
            StorageNamespace::Presence => "presence/",
            StorageNamespace::Emergency => "emergency/",
            StorageNamespace::Custom => "custom/",
        }
    }
}

/// spec §98's own two worked examples' shape: an extension-labeled
/// counter. "Avoid unbounded metric labels such as full peer IDs" is
/// enforced structurally here the same way [`crate::observability::TraceFields`]
/// enforces "never log payloads" — there is no peer-identifying field
/// anywhere on this type for a caller to accidentally add as a label.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LabeledMetric {
    pub metric_name: &'static str,
    pub extension: ProtocolId,
    pub value: u64,
}

/// spec §99's own worked example: "ERP document module can access
/// files, cannot access messaging." A named grant of exactly which
/// extensions one application module may reach — "applications should
/// receive only capability handles they are allowed to use," made
/// checkable rather than trusted to application code discipline alone.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModuleAccessGrant {
    pub module_name: String,
    pub permitted_extensions: Vec<ProtocolId>,
}

impl ModuleAccessGrant {
    pub fn permits(&self, extension: &ProtocolId) -> bool {
        self.permitted_extensions.contains(extension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::{NamespaceId, ProtocolMajor, ProtocolName};

    #[test]
    fn spec_96_submission_never_names_a_transport_queue_or_retry_policy() {
        // Structural claim, proven by this compiling with only the six
        // spec-named fields.
        let submission = SchedulingSubmission {
            priority: crate::lifecycle::TrafficPriority::Interactive,
            deadline_millis: Some(5_000),
            payload_size_bytes: 1024,
            delivery_class: DeliveryClass::Realtime,
            peer: PeerIdentity([0u8; 32]),
            durable: false,
        };
        assert!(!submission.durable);
    }

    #[test]
    fn spec_97_the_five_namespace_prefixes_are_all_distinct() {
        let prefixes = [
            StorageNamespace::Messaging.prefix(),
            StorageNamespace::Files.prefix(),
            StorageNamespace::Presence.prefix(),
            StorageNamespace::Emergency.prefix(),
            StorageNamespace::Custom.prefix(),
        ];
        let unique: std::collections::HashSet<_> = prefixes.iter().collect();
        assert_eq!(unique.len(), 5);
    }

    #[test]
    fn spec_99_erp_document_module_can_access_files_but_not_messaging() {
        let files = ProtocolId::new(
            NamespaceId::new("org.example").unwrap(),
            ProtocolName::new("files").unwrap(),
            ProtocolMajor(1),
        );
        let messaging = ProtocolId::new(
            NamespaceId::new("org.example").unwrap(),
            ProtocolName::new("messaging").unwrap(),
            ProtocolMajor(1),
        );
        let grant = ModuleAccessGrant {
            module_name: "erp-document-module".to_string(),
            permitted_extensions: vec![files.clone()],
        };

        assert!(grant.permits(&files));
        assert!(!grant.permits(&messaging));
    }
}
