//! One-to-one direct messaging over a dedicated ALPN.
//!
//! Unchanged from v1's transport design — only ever opened between two
//! contacts already in the `Accepted` state (see `net::contacts`); a
//! stranger can only reach you on the separate contact-request ALPN.
//!
//! Design: a single QUIC `Connection` is kept open per active peer. Each
//! outgoing chat message is sent on its own unidirectional stream (streams
//! are cheap in QUIC — no head-of-line blocking between messages, and we
//! don't need request/response semantics for a chat line). The accepting
//! side loops `accept_uni`, reads each stream to completion, decodes an
//! `Envelope`, and forwards it to the app via a channel.
//!
//! Note on delivery: a successful `send()` here only proves the local QUIC
//! stack accepted the write — over a degraded connection, `write_all()` +
//! `finish()` can return `Ok(())` without the bytes ever reaching the peer.
//! Actual delivery confirmation is handled one layer up, in `ui`, via
//! `Body::Ack` envelopes sent back over this same send path in reverse.

use super::message::{Envelope, MAX_MESSAGE_BYTES};
use anyhow::Result;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh::{Endpoint, EndpointAddr, EndpointId};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, warn};

/// Per-attempt timeout for connect/send. Without this, a dead network (no
/// route, cable pulled, Wi-Fi off) relies entirely on QUIC's own internal
/// idle timeout to notice — which can take a long while — rather than
/// failing fast so the app can report it and retry.
const NET_TIMEOUT: Duration = Duration::from_secs(8);

/// ALPN identifying this application's DM protocol. Versioned so a future
/// breaking wire-format change can run alongside old clients during
/// rollout. Deliberately separate from `net::contacts::ALPN` — a peer must
/// have been `Accept`ed as a contact before it can ever reach this handler.
pub const ALPN: &[u8] = b"iroh-messenger/dm/1";

/// Events the DM handler surfaces up to the application.
#[derive(Debug, Clone)]
pub enum DmEvent {
    Received {
        from: EndpointId,
        envelope: Envelope,
    },
    PeerConnected {
        from: EndpointId,
    },
    PeerDisconnected {
        from: EndpointId,
    },
}

/// Registered with the `Router` to accept incoming DM connections.
#[derive(Debug, Clone)]
pub struct DmProtocol {
    events: UnboundedSender<DmEvent>,
}

impl DmProtocol {
    pub fn new(events: UnboundedSender<DmEvent>) -> Self {
        Self { events }
    }
}

impl ProtocolHandler for DmProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let peer = connection.remote_id();
        debug!(%peer, "dm: incoming connection");
        let _ = self.events.send(DmEvent::PeerConnected { from: peer });

        loop {
            let mut recv = match connection.accept_uni().await {
                Ok(r) => r,
                Err(_) => break, // peer closed the connection
            };
            let bytes = match recv.read_to_end(MAX_MESSAGE_BYTES).await {
                Ok(b) => b,
                Err(e) => {
                    warn!(%peer, error = %e, "dm: failed reading stream");
                    continue;
                }
            };
            match Envelope::decode(&bytes) {
                Ok(envelope) => {
                    let _ = self.events.send(DmEvent::Received {
                        from: peer,
                        envelope,
                    });
                }
                Err(e) => warn!(%peer, error = %e, "dm: failed decoding envelope"),
            }
        }

        connection.closed().await;
        let _ = self.events.send(DmEvent::PeerDisconnected { from: peer });
        Ok(())
    }
}

/// An open outgoing DM connection to one peer. Cheap to keep around for the
/// duration of a chat session; opens a fresh uni stream per message.
#[derive(Clone)]
pub struct DmSession {
    connection: Connection,
}

impl DmSession {
    pub async fn connect(endpoint: &Endpoint, addr: impl Into<EndpointAddr>) -> Result<Self> {
        let connection = tokio::time::timeout(NET_TIMEOUT, endpoint.connect(addr, ALPN))
            .await
            .map_err(|_| anyhow::anyhow!("connect timed out after {NET_TIMEOUT:?}"))??;
        Ok(Self { connection })
    }

    pub async fn send(&self, envelope: &Envelope) -> Result<()> {
        tokio::time::timeout(NET_TIMEOUT, self.send_inner(envelope))
            .await
            .map_err(|_| anyhow::anyhow!("send timed out after {NET_TIMEOUT:?}"))?
    }

    async fn send_inner(&self, envelope: &Envelope) -> Result<()> {
        let mut send = self.connection.open_uni().await?;
        send.write_all(&envelope.encode()?).await?;
        send.finish()?;
        Ok(())
    }

    pub async fn say_hello(&self, my_name: &str) -> Result<()> {
        self.send(&Envelope::hello(my_name)).await
    }
}
