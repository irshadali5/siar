//! Device-to-endpoint hints — `path.rs`'s own top doc comment flagged
//! this gap when `PathTable` was corrected to key on `EndpointId`
//! instead of `DeviceId`: *"If a caller genuinely needs a `DeviceId`-
//! keyed view... that's a join against whatever maps `DeviceId ->
//! EndpointId` for paired contacts — this table doesn't attempt that
//! mapping itself."* This is that join, built for the one real signal
//! a relay like `apps/emergency-node` actually has for it.
//!
//! `MeshEnvelope` (`siar-protocol::mesh`) deliberately carries no
//! sender identity — next.md §73–74's mesh-privacy design, a relay
//! shouldn't learn who's talking to whom just by forwarding traffic.
//! `MailboxCheckIn` is the one message type in this whole design that
//! breaks that symmetry on purpose: a device *choosing* to reveal its
//! own identity to ask "do you have anything for me" (see
//! `siar-protocol::mailbox`'s own doc comment). `DeviceRoutes` exists
//! to remember that disclosure past the single request/response that
//! triggered it, so a bundle arriving *after* a check-in can be pushed
//! proactively instead of the destination having to poll again.
//!
//! Unauthenticated, same caveat `MailboxCheckIn` itself already
//! carries — a device claim isn't cryptographically verified anywhere
//! in this pass (next.md §32's real capability/token system, still not
//! attempted), so every hint here is "worth trying," not "guaranteed
//! correct." Bounded by staleness (`remove_stale`), same shape as
//! `PathTable`'s own method of the same name — next.md §92's "mobile
//! topology changes too quickly" applies exactly as much to "which
//! endpoint is this device reachable at" as it does to "which path
//! reaches this endpoint."

use iroh::EndpointId;
use siar_domain::DeviceId;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
struct Hint {
    endpoint: EndpointId,
    last_seen: u64,
}

pub struct DeviceRoutes {
    hints: HashMap<DeviceId, Hint>,
}

impl DeviceRoutes {
    pub fn new() -> Self {
        Self {
            hints: HashMap::new(),
        }
    }

    /// Records or refreshes a device's self-disclosed endpoint —
    /// overwrites any previous hint for the same device outright
    /// rather than keeping history, since a device only has one
    /// current location worth acting on (unlike `PathTable`, which
    /// deliberately keeps multiple candidate paths per destination).
    pub fn record(&mut self, device: DeviceId, endpoint: EndpointId, now: u64) {
        self.hints.insert(
            device,
            Hint {
                endpoint,
                last_seen: now,
            },
        );
    }

    pub fn get(&self, device: DeviceId) -> Option<EndpointId> {
        self.hints.get(&device).map(|hint| hint.endpoint)
    }

    pub fn remove_stale(&mut self, now: u64, max_age: u64) {
        self.hints
            .retain(|_, hint| now.saturating_sub(hint.last_seen) <= max_age);
    }
}

impl Default for DeviceRoutes {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Same test-fixture construction as `path.rs`'s own tests — see
    // that module's test doc comment for why this specific
    // `SecretKey::from_str` shape was used instead of a guessed
    // `from_bytes` method.
    fn test_endpoint_id(seed: u8) -> EndpointId {
        use std::str::FromStr;
        let hex = format!("{seed:02x}").repeat(32);
        let secret = iroh::SecretKey::from_str(&hex).expect("valid 64-char hex test secret key");
        secret.public()
    }

    #[test]
    fn record_then_get_round_trips() {
        let mut routes = DeviceRoutes::new();
        let device = DeviceId::new();
        let endpoint = test_endpoint_id(1);
        routes.record(device, endpoint, 100);
        assert_eq!(routes.get(device), Some(endpoint));
    }

    #[test]
    fn get_on_unknown_device_is_none() {
        let routes = DeviceRoutes::new();
        assert_eq!(routes.get(DeviceId::new()), None);
    }

    #[test]
    fn a_second_record_overwrites_rather_than_accumulating() {
        let mut routes = DeviceRoutes::new();
        let device = DeviceId::new();
        routes.record(device, test_endpoint_id(1), 100);
        routes.record(device, test_endpoint_id(2), 200);
        assert_eq!(routes.get(device), Some(test_endpoint_id(2)));
    }

    #[test]
    fn remove_stale_drops_old_hints_and_keeps_recent_ones() {
        let mut routes = DeviceRoutes::new();
        let old_device = DeviceId::new();
        let recent_device = DeviceId::new();
        routes.record(old_device, test_endpoint_id(1), 0);
        routes.record(recent_device, test_endpoint_id(2), 90);

        routes.remove_stale(100, 50);
        assert_eq!(routes.get(old_device), None);
        assert_eq!(routes.get(recent_device), Some(test_endpoint_id(2)));
    }
}
