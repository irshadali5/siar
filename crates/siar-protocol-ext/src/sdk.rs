//! spec §43 "Public SDK Surface": "Applications should normally use:
//! `CommunicationClient`, `MessagingClient`, `FileTransferClient`,
//! `PresenceClient` — not raw extension handlers. Extension internals
//! remain replaceable."
//!
//! These are marker traits, not implementations — this crate has no
//! concrete client to back them with yet (no wire integration; see
//! `lib.rs`'s own "What's explicitly NOT here"). What's real here is
//! the *shape* of the boundary §43 describes: an application-facing
//! trait per feature area, separate from [`crate::registry::ProtocolExtension`]
//! (the raw handler interface extension authors implement), so a
//! future concrete client can implement these without applications
//! ever needing to hold a [`crate::registry::ExtensionHandler`]
//! directly — "extension internals remain replaceable" is exactly the
//! property a marker trait boundary buys, before there's even a
//! concrete type on either side of it.

/// spec §43's top-level umbrella client.
pub trait CommunicationClient {}

/// spec §43's per-feature clients — kept as separate traits, not
/// supertraits or associated types of [`CommunicationClient`], since
/// spec §43 lists them as siblings an application picks among (a
/// files-only application has no reason to depend on
/// `PresenceClient`'s existence at all — the same modularity §42
/// already argues for events applies here to clients).
pub trait MessagingClient {}
pub trait FileTransferClient {}
pub trait PresenceClient {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Not a mock of real behavior (there's nothing to mock yet) —
    /// this only proves the trait boundary spec §43 describes is
    /// actually implementable by something that isn't
    /// [`crate::registry::ProtocolExtension`], which is the concrete
    /// claim "applications use clients, not raw extension handlers"
    /// makes.
    struct StubMessagingClient;
    impl MessagingClient for StubMessagingClient {}

    #[test]
    fn spec_43_a_client_type_is_distinct_from_a_raw_extension_handler() {
        fn takes_messaging_client<T: MessagingClient>(_client: &T) {}
        takes_messaging_client(&StubMessagingClient);
    }
}
