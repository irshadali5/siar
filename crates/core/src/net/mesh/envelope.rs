//! Wire format for one flooded mesh message, and the dedup cache every
//! transport consults before re-broadcasting one.

use iroh::EndpointId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Hop budget for a freshly-sent envelope. Small on purpose: this is a
/// same-room/same-building mesh, not a long-haul store-and-forward
/// network, and every extra hop multiplies flood traffic across every
/// node that heard the previous one.
pub const DEFAULT_TTL: u8 = 6;

/// How long a message id stays in the dedup cache. Bounds memory (no
/// unbounded growth over a long-running session) while comfortably
/// outlasting how long a flood wave can realistically take to finish
/// propagating at BLE/LAN mesh scale.
const SEEN_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Random per-message id, used only for flood dedup — not the same
    /// as any id inside `payload`. 16 bytes (uuid v4) is plenty at this
    /// scale and keeps every mesh packet's header small.
    pub id: [u8; 16],
    pub ttl: u8,
    /// Original sender's `EndpointId`, as raw bytes — kept even through
    /// re-floods so `App` knows who a mesh-delivered message is
    /// actually from, not just which node last relayed it. Stored as
    /// `[u8; 32]` rather than `EndpointId` itself: nothing else in this
    /// codebase serde-derives `EndpointId` directly (see `app::hex`/
    /// `parse_hex`, which every other wire path goes through instead),
    /// so this avoids depending on an unverified `Serialize` impl from
    /// `iroh` and costs nothing extra — it's the same 32 bytes either
    /// way, just without a newtype wrapper.
    pub sender: [u8; 32],
    /// An already-`protocol::message::Envelope::encode`d message. The
    /// mesh transports never inspect or decode this themselves.
    pub payload: Vec<u8>,
}

impl Envelope {
    pub fn new(sender: EndpointId, payload: Vec<u8>) -> Self {
        Self {
            id: *uuid::Uuid::new_v4().as_bytes(),
            ttl: DEFAULT_TTL,
            sender: *sender.as_bytes(),
            payload,
        }
    }

    /// The original sender, parsed back from `sender`'s raw bytes.
    /// `None` on the (should-never-happen-in-practice) case of a
    /// corrupt/foreign 32 bytes that don't decode to a valid key.
    pub fn sender_id(&self) -> Option<EndpointId> {
        EndpointId::from_bytes(&self.sender).ok()
    }

    /// `Some(next)` with `ttl - 1` if there's budget left to re-flood,
    /// `None` once a message has exhausted its hop count.
    pub fn decremented(&self) -> Option<Self> {
        if self.ttl == 0 {
            return None;
        }
        Some(Self {
            ttl: self.ttl - 1,
            ..self.clone()
        })
    }

    pub fn encode(&self) -> anyhow::Result<Vec<u8>> {
        Ok(postcard::to_allocvec(self)?)
    }

    pub fn decode(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(postcard::from_bytes(bytes)?)
    }
}

/// Tracks which envelope ids have already been seen (received *or*
/// originated locally) so a flood doesn't loop or re-deliver the same
/// message to `App` twice. One `Mutex<HashMap>` is plenty here — mesh
/// traffic volume is chat-message scale, not a hot path that needs
/// lock-free structures.
pub struct SeenCache {
    seen: Mutex<HashMap<[u8; 16], Instant>>,
}

impl SeenCache {
    pub fn new() -> Self {
        Self {
            seen: Mutex::new(HashMap::new()),
        }
    }

    /// Records an id as seen without asking whether it was new — used
    /// when *this* node originates a message, so it never re-processes
    /// its own flood echo as a fresh inbound message.
    pub fn mark(&self, id: [u8; 16]) {
        self.prune_and_insert(id);
    }

    /// Records an id as seen and reports whether it was actually new.
    /// `false` means "already relayed, drop it" — the core flood-dedup
    /// check every transport's receive path runs before handing an
    /// envelope up to `MeshManager::on_received`.
    pub fn mark_and_check_new(&self, id: [u8; 16]) -> bool {
        self.prune_and_insert(id)
    }

    fn prune_and_insert(&self, id: [u8; 16]) -> bool {
        let mut seen = self.seen.lock().unwrap();
        let now = Instant::now();
        seen.retain(|_, seen_at| now.duration_since(*seen_at) < SEEN_TTL);
        seen.insert(id, now).is_none()
    }
}
