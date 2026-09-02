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

        // iroh calls `accept()` once per *connection*, not once per
        // stream ("[o]nce accept() returns, the connection is
        // dropped" — iroh::protocol::ProtocolHandler::accept's own
        // docs). `pool.rs` exists specifically to reuse one connection
        // across many sends, so this handler has to loop and keep
        // accepting streams for as long as the connection stays open —
        // handling exactly one stream and then falling straight to
        // `connection.closed().await` (the original shape here) meant
        // every message after the first one sent on a pooled/reused
        // connection was silently never read. Confirmed as a real bug
        // via `tests/roundtrip.rs`'s two-sends-on-one-connection case,
        // not just inferred from the docs.
        loop {
            let (_send, mut recv) = match connection.accept_bi().await {
                Ok(streams) => streams,
                // The peer closed the connection (or it failed) —
                // that's the normal way a caller signals "no more
                // messages on this connection," not a protocol error
                // worth propagating and tearing the task down over.
                Err(_) => break,
            };

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
            if self
                .incoming
                .send(IncomingFrame {
                    from: remote,
                    message,
                })
                .await
                .is_err()
            {
                tracing::warn!("inbound frame channel closed; dropping connection");
                break;
            }
        }

        Ok(())
    }
}
