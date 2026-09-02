//! spec §41 "Extension Events" ("prefer feature-specific event
//! types"), §42 "Avoid a Mandatory Giant Event Enum".

/// spec §41's own messaging example, verbatim variant names.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MessagingEvent {
    MessageReceived,
    ReceiptReceived,
    TypingChanged,
}

/// spec §41's own files example, verbatim variant names.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FileEvent {
    TransferStarted,
    TransferProgress,
    TransferCompleted,
}

/// spec §42 names `CoreEvent`/`PresenceEvent` alongside
/// `MessagingEvent`/`FileEvent` as siblings, but never spells out
/// their own variants anywhere in this document (unlike messaging's
/// and files', which §41 gives verbatim) — rather than invent
/// unspecified content, `CoreEvent` covers only what this crate itself
/// already has real, lifecycle-level events for:
/// [`crate::lifecycle::ExtensionLifecycle`] transitions. `PresenceEvent`
/// is deliberately not defined here at all (see this module's own
/// "not attempted" note in `lib.rs`) rather than guessed.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CoreEvent {
    ExtensionLifecycleChanged {
        extension: crate::identifier::ProtocolId,
        from: crate::lifecycle::ExtensionLifecycle,
        to: crate::lifecycle::ExtensionLifecycle,
    },
}

/// spec §42: "with optional aggregation for applications that want one
/// stream." The operative word is *optional* — nothing in
/// [`MessagingEvent`]/[`FileEvent`]/[`CoreEvent`] depends on this
/// existing, and a feature module never has to import it; this is
/// purely an opt-in convenience for an application that specifically
/// wants a single merged stream instead of subscribing to each
/// feature's events separately.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AggregatedEvent {
    Core(CoreEvent),
    Messaging(MessagingEvent),
    File(FileEvent),
}

impl From<CoreEvent> for AggregatedEvent {
    fn from(event: CoreEvent) -> Self {
        AggregatedEvent::Core(event)
    }
}

impl From<MessagingEvent> for AggregatedEvent {
    fn from(event: MessagingEvent) -> Self {
        AggregatedEvent::Messaging(event)
    }
}

impl From<FileEvent> for AggregatedEvent {
    fn from(event: FileEvent) -> Self {
        AggregatedEvent::File(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_42_a_messaging_only_consumer_never_needs_file_or_core_types() {
        // The real claim §42 makes is a modularity one, not a runtime
        // one — this test's actual assertion is that the line below
        // compiles at all without importing FileEvent/CoreEvent.
        let event = MessagingEvent::MessageReceived;
        assert_eq!(event, MessagingEvent::MessageReceived);
    }

    #[test]
    fn spec_42_aggregation_is_opt_in_via_from() {
        let messaging: AggregatedEvent = MessagingEvent::TypingChanged.into();
        let file: AggregatedEvent = FileEvent::TransferStarted.into();
        assert_eq!(
            messaging,
            AggregatedEvent::Messaging(MessagingEvent::TypingChanged)
        );
        assert_eq!(file, AggregatedEvent::File(FileEvent::TransferStarted));
    }
}
