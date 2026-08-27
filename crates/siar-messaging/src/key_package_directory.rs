//! Key-package publish/fetch — closes part of the gap
//! `group_service.rs`'s module doc has been flagging since the MLS path
//! landed: "`add_member_mls` takes `new_member_key_package_bytes` as a
//! direct parameter — this module has no way to fetch a device's
//! current key package on its own."
//!
//! This is deliberately a small, local directory abstraction — not
//! next.md §41's full contact-discovery system (QR codes, invite links,
//! username service, nearby discovery). It answers one narrower
//! question: "given a `DeviceId` I already know about (from
//! `DeviceDirectory`, a contact card, wherever), what's their most
//! recently published MLS key package?" Whatever next.md §41 mechanism
//! eventually resolves "who is this person's device" is a different,
//! earlier step this trait doesn't attempt.
//!
//! RFC 9420 key packages are single-use — `take` (not `fetch`/`peek`)
//! is named to make that consumption explicit at the call site, not a
//! detail the caller has to remember separately.

use siar_domain::DeviceId;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

pub trait KeyPackageDirectory: Send + Sync {
    /// Publishes a freshly generated, serialized key package
    /// (`siar_crypto_mls::encode_key_package`'s output) for `device`,
    /// available for exactly one future `take`.
    fn publish(&self, device: DeviceId, key_package_bytes: Vec<u8>);

    /// Consumes and returns one of `device`'s published key packages,
    /// or `None` if it has none outstanding. A device that calls
    /// `publish` more than once before any are consumed has more than
    /// one available — `take` doesn't specify *which* one comes back
    /// beyond "some order this implementation defines" (the in-memory
    /// implementation below uses FIFO; that's not part of this trait's
    /// contract).
    fn take(&self, device: DeviceId) -> Option<Vec<u8>>;
}

/// Reference implementation — in-memory only, matching every other
/// piece of this workspace's MLS integration's current persistence
/// story (see `siar_crypto_mls`'s own doc comment). A real deployment
/// needs this backed by something a sender can actually reach a
/// recipient's key packages through when they're offline (next.md
/// §31's mailbox concept, or a dedicated key-package publish endpoint)
/// — this type exists so `GroupService`'s directory-backed methods have
/// something real to test against and a caller has a working default,
/// not as a claim that in-process memory is a deployable answer to
/// "how does Bob's key package reach Alice."
#[derive(Default)]
pub struct InMemoryKeyPackageDirectory {
    packages: Mutex<HashMap<DeviceId, VecDeque<Vec<u8>>>>,
}

impl InMemoryKeyPackageDirectory {
    pub fn new() -> Self {
        Self::default()
    }
}

impl KeyPackageDirectory for InMemoryKeyPackageDirectory {
    fn publish(&self, device: DeviceId, key_package_bytes: Vec<u8>) {
        self.packages
            .lock()
            .expect("InMemoryKeyPackageDirectory lock poisoned")
            .entry(device)
            .or_default()
            .push_back(key_package_bytes);
    }

    fn take(&self, device: DeviceId) -> Option<Vec<u8>> {
        let mut packages = self
            .packages
            .lock()
            .expect("InMemoryKeyPackageDirectory lock poisoned");
        let queue = packages.get_mut(&device)?;
        let package = queue.pop_front();
        if queue.is_empty() {
            packages.remove(&device);
        }
        package
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_returns_none_when_nothing_was_published() {
        let dir = InMemoryKeyPackageDirectory::new();
        assert_eq!(dir.take(DeviceId::new()), None);
    }

    #[test]
    fn take_returns_and_consumes_a_published_package() {
        let dir = InMemoryKeyPackageDirectory::new();
        let device = DeviceId::new();
        dir.publish(device, b"key-package-bytes".to_vec());

        assert_eq!(dir.take(device), Some(b"key-package-bytes".to_vec()));
        // Single-use: the second take finds nothing.
        assert_eq!(dir.take(device), None);
    }

    #[test]
    fn multiple_publishes_come_back_in_fifo_order() {
        let dir = InMemoryKeyPackageDirectory::new();
        let device = DeviceId::new();
        dir.publish(device, b"first".to_vec());
        dir.publish(device, b"second".to_vec());

        assert_eq!(dir.take(device), Some(b"first".to_vec()));
        assert_eq!(dir.take(device), Some(b"second".to_vec()));
        assert_eq!(dir.take(device), None);
    }

    #[test]
    fn packages_for_different_devices_dont_cross_over() {
        let dir = InMemoryKeyPackageDirectory::new();
        let alice = DeviceId::new();
        let bob = DeviceId::new();
        dir.publish(alice, b"alice-package".to_vec());

        assert_eq!(dir.take(bob), None);
        assert_eq!(dir.take(alice), Some(b"alice-package".to_vec()));
    }
}
