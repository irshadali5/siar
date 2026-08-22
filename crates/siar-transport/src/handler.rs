//! Inbound connection handling (plan.md §112's receive flow, steps up to
//! "decode limits" and "identity verification" — replay/E2EE/persistence
//! happen in `siar-messaging`, not here; this crate's job stops at
//! "decoded, size-bounded `WireMessage`").

use iroh::{
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
    EndpointId,
};
use siar_protocol::{decode_frame, WireMessage};
use tokio::sync::mpsc;

const MAX_INBOUND_FRAME_READ: usize = 256 * 1024; // matches MAX_CONTROL_FRAME_BYTES

#[derive(Debug, Clone)]
pub struct IncomingFrame {
    pub from: EndpointId,
    pub message: WireMessage,
}

#[derive(Debug, Clone)]
pub struct MessagingProtocolHandler {
    incoming: mpsc::Sender<IncomingFrame>,
}

impl MessagingProtocolHandler {
    pub fn new(incoming: mpsc::Sender<IncomingFrame>) -> Self {
        Self { incoming }
    }
}

impl ProtocolHandler for MessagingProtocolHandler {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let remote = connection.remote_id();

        let (_send, mut recv) = connection
            .accept_bi()
            .await
            .map_err(AcceptError::from_err)?;

        // Phase 1: one frame per stream (plan.md §11's Envelope is small;
        // splitting many messages across one long-lived stream is a
        // Phase-2 optimization, not correctness-relevant yet).
        let bytes = recv
            .read_to_end(MAX_INBOUND_FRAME_READ)
            .await
            .map_err(AcceptError::from_err)?;

        let (message, _consumed) = decode_frame(&bytes).map_err(AcceptError::from_err)?;

        // A full channel means the messaging core is backed up (plan.md
        // §56 backpressure) — drop the connection rather than buffer
        // unboundedly; the sender's outbox will retry.
        if self.incoming.send(IncomingFrame { from: remote, message }).await.is_err() {
            tracing::warn!("inbound frame channel closed; dropping connection");
        }

        connection.closed().await;
        Ok(())
    }
}
