//! Contact request/accept flow (Keet/WhatsApp-style): before two people can
//! DM, one must send a request and the other must accept it. This runs on
//! its own ALPN, deliberately separate from `protocol::dm::ALPN` — a
//! stranger can reach this handler and nothing else.
//!
//! Local state machine lives in `store::ContactState`:
//! `None -> PendingOut ⇄ PendingIn -> Accepted | Blocked`.

use crate::store::{ContactState, Store};
use anyhow::Result;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh::{Endpoint, EndpointAddr, EndpointId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tracing::warn;

pub const ALPN: &[u8] = b"iroh-messenger/contact/1";
const NET_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_FRAME_BYTES: usize = 4096; // requests are tiny; generous cap against abuse

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactMsg {
    Request {
        from_id: [u8; 32],
        from_username: Option<String>,
        from_name: String,
        note: String,
    },
    Accept {
        from_id: [u8; 32],
        from_username: Option<String>,
        from_name: String,
    },
    Reject {
        from_id: [u8; 32],
    },
}

impl ContactMsg {
    fn encode(&self) -> Result<Vec<u8>> {
        Ok(postcard::to_stdvec(self)?)
    }
    fn decode(bytes: &[u8]) -> Result<Self> {
        Ok(postcard::from_bytes(bytes)?)
    }
}

#[derive(Debug, Clone)]
pub enum ContactEvent {
    /// Someone we don't yet know sent us a request — surfaces in the
    /// requests inbox for the user to Accept/Decline.
    IncomingRequest {
        from_id: EndpointId,
        from_username: Option<String>,
        from_name: String,
        note: String,
    },
    /// A request we sent was accepted — the contact is now `Accepted` and
    /// a DM session can be opened.
    RequestAccepted {
        from_id: EndpointId,
        from_username: Option<String>,
        from_name: String,
    },
    /// A request we sent was declined. We remove the `PendingOut` row so
    /// the UI doesn't show it as still-pending forever.
    RequestRejected { from_id: EndpointId },
}

/// Registered with the `Router` to accept incoming contact-protocol
/// connections. Holds the `Store` directly (unlike `DmProtocol`, which
/// only forwards events) because a `Blocked` sender needs to be dropped
/// *here*, before ever reaching the UI layer.
#[derive(Clone)]
pub struct ContactProtocol {
    store: Arc<Store>,
    events: UnboundedSender<ContactEvent>,
}

// Manual impl: `ProtocolHandler` requires `Debug`, but `Store` wraps a
// `rusqlite::Connection` that doesn't implement it — and printing
// connection/channel internals wouldn't be useful in a debug log anyway.
impl std::fmt::Debug for ContactProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContactProtocol").finish_non_exhaustive()
    }
}

impl ContactProtocol {
    pub fn new(store: Arc<Store>, events: UnboundedSender<ContactEvent>) -> Self {
        Self { store, events }
    }
}

impl ProtocolHandler for ContactProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let peer = connection.remote_id();
        let peer_hex = crate::app::hex(peer);

        if self.store.is_blocked(&peer_hex) {
            // Silently drop: connection accepted, nothing read further,
            // no signal back to the sender that they were specifically
            // blocked (vs. e.g. offline).
            return Ok(());
        }

        let mut recv = match connection.accept_uni().await {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };
        let bytes = match recv.read_to_end(MAX_FRAME_BYTES).await {
            Ok(b) => b,
            Err(e) => {
                warn!(%peer, error = %e, "contact: failed reading frame");
                return Ok(());
            }
        };
        let msg = match ContactMsg::decode(&bytes) {
            Ok(m) => m,
            Err(e) => {
                warn!(%peer, error = %e, "contact: bad frame");
                return Ok(());
            }
        };

        match msg {
            ContactMsg::Request {
                from_username,
                from_name,
                note,
                ..
            } => {
                let _ = self.store.upsert_contact(
                    &peer_hex,
                    from_username.as_deref(),
                    &from_name,
                    ContactState::PendingIn,
                );
                let _ = self.events.send(ContactEvent::IncomingRequest {
                    from_id: peer,
                    from_username,
                    from_name,
                    note,
                });
            }
            ContactMsg::Accept {
                from_username,
                from_name,
                ..
            } => {
                let _ = self.store.upsert_contact(
                    &peer_hex,
                    from_username.as_deref(),
                    &from_name,
                    ContactState::Accepted,
                );
                let _ = self.events.send(ContactEvent::RequestAccepted {
                    from_id: peer,
                    from_username,
                    from_name,
                });
            }
            ContactMsg::Reject { .. } => {
                let _ = self.store.remove_contact(&peer_hex);
                let _ = self
                    .events
                    .send(ContactEvent::RequestRejected { from_id: peer });
            }
        }

        Ok(())
    }
}

/// Send a contact request to someone found via username search (or a
/// pasted ticket). Fire-and-forget over a single uni stream — no session
/// kept open, since a request is a one-shot message, not an ongoing chat.
///
/// `addr_hint`, when present, carries the peer's actual relay/direct
/// addresses (typically decoded straight from a ticket — see `ticket.rs`)
/// so the connect step below can dial them directly instead of waiting on
/// iroh's discovery service, which may not have learned this peer's
/// address yet. `None` falls back to dialing by bare id, i.e. discovery
/// only, same as before this existed — used for username-search-initiated
/// requests, where we only ever had an id to begin with.
#[allow(clippy::too_many_arguments)]
pub async fn send_request(
    endpoint: &Endpoint,
    to: EndpointId,
    addr_hint: Option<EndpointAddr>,
    my_id: EndpointId,
    my_username: Option<String>,
    my_name: &str,
    note: &str,
) -> Result<()> {
    let target = addr_hint.unwrap_or_else(|| to.into());
    send_one(
        endpoint,
        target,
        &ContactMsg::Request {
            from_id: *my_id.as_bytes(),
            from_username: my_username,
            from_name: my_name.to_string(),
            note: note.to_string(),
        },
    )
    .await
}

/// Accept an incoming request: dial the requester back and tell them so.
/// The caller (app.rs) is responsible for flipping the local contact row
/// to `Accepted` — done separately so the UI can update immediately even
/// if this network round trip is slow, rather than blocking the click.
pub async fn send_accept(
    endpoint: &Endpoint,
    to: EndpointId,
    my_id: EndpointId,
    my_username: Option<String>,
    my_name: &str,
) -> Result<()> {
    send_one(
        endpoint,
        to.into(),
        &ContactMsg::Accept {
            from_id: *my_id.as_bytes(),
            from_username: my_username,
            from_name: my_name.to_string(),
        },
    )
    .await
}

pub async fn send_reject(endpoint: &Endpoint, to: EndpointId, my_id: EndpointId) -> Result<()> {
    send_one(
        endpoint,
        to.into(),
        &ContactMsg::Reject {
            from_id: *my_id.as_bytes(),
        },
    )
    .await
}

async fn send_one(endpoint: &Endpoint, target: EndpointAddr, msg: &ContactMsg) -> Result<()> {
    let connection = tokio::time::timeout(NET_TIMEOUT, endpoint.connect(target, ALPN))
        .await
        .map_err(|_| anyhow::anyhow!("contact request connect timed out"))??;
    let mut send = connection.open_uni().await?;
    send.write_all(&msg.encode()?).await?;
    send.finish()?;
    // This is the actual bug behind "request sent" toasts the other side
    // never receives: `finish()` only *schedules* the FIN locally — it's
    // a buffer operation, not a round trip — so it returns almost
    // instantly even though the bytes are still in flight to the peer,
    // especially over a relay hop before hole-punching has kicked in.
    // `connection` is a local variable that goes out of scope the moment
    // this function returns, and dropping the last handle to a QUIC
    // connection closes it immediately (see iroh's own writeup on this,
    // "Closing a QUIC Connection": dropping too early is one of the two
    // main ways people accidentally lose data they thought they'd sent).
    // The receiver gets a CONNECTION_CLOSE before — or instead of — the
    // stream data. `send_request`/`send_accept`/`send_reject` all funnel
    // through here, so this alone explains requests silently vanishing
    // in both directions.
    //
    // `stopped()` is the documented way to actually wait for the peer to
    // receive the buffered data before we let the connection close: it
    // resolves once they've read it all (or reset the stream), i.e. once
    // there's nothing left to race. Bounded by the same timeout as the
    // connect above; if it times out we still return success rather than
    // failing the whole request, since the write itself did succeed —
    // this is strictly an improvement over not waiting at all, not a
    // delivery guarantee (nothing over a fire-and-forget uni stream can
    // be one, per that same iroh writeup).
    let _ = tokio::time::timeout(NET_TIMEOUT, send.stopped()).await;
    Ok(())
}
