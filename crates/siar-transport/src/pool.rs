//! Connection reuse (plan.md §34–35): don't dial a fresh QUIC connection
//! for every message. `iroh::endpoint::Connection` is a cheap `Clone`
//! handle over the real connection (confirmed in iroh's source — see
//! this crate's top-level doc comment on how these signatures were
//! verified), so caching the handle and checking `close_reason()` before
//! reuse is enough; no manual keepalive/ping logic needed here.
//!
//! Keyed by `(EndpointId, alpn)`, not just `EndpointId`: a connection
//! negotiates one ALPN, so the messaging connection and the blob
//! connection to the *same* peer (plan.md §22's attachment flow, added
//! alongside the original text-messaging ALPN) are two separate
//! connections, not one shared handle.

use iroh::endpoint::Connection;
use iroh::EndpointId;
use std::collections::HashMap;
use std::sync::Mutex;

type PoolKey = (EndpointId, Vec<u8>);

#[derive(Default)]
pub(crate) struct ConnectionPool {
    connections: Mutex<HashMap<PoolKey, Connection>>,
}

impl ConnectionPool {
    pub(crate) fn get_live(&self, peer: &EndpointId, alpn: &[u8]) -> Option<Connection> {
        let key = (*peer, alpn.to_vec());
        let connections = self
            .connections
            .lock()
            .expect("connection pool mutex poisoned");
        let conn = connections.get(&key)?;
        if conn.close_reason().is_some() {
            // Dead — caller will dial fresh and `insert` will replace it.
            return None;
        }
        Some(conn.clone())
    }

    pub(crate) fn insert(&self, peer: EndpointId, alpn: &[u8], connection: Connection) {
        self.connections
            .lock()
            .expect("connection pool mutex poisoned")
            .insert((peer, alpn.to_vec()), connection);
    }

    /// plan.md §35: idle connections should be allowed to expire rather
    /// than held forever. Phase 2 scope: an explicit sweep the retry
    /// scheduler calls periodically, not a timer per-connection yet —
    /// that's a reasonable later refinement once there's real traffic
    /// data to tune idle timeouts against (plan.md §90's metrics).
    pub(crate) fn evict_closed(&self) {
        self.connections
            .lock()
            .expect("connection pool mutex poisoned")
            .retain(|_, conn| conn.close_reason().is_none());
    }
}
