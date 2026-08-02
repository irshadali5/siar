//! Group chat "rooms" built on `iroh-gossip`.
//!
//! Everyone who joins the same room name subscribes to the same
//! blake3-derived `TopicId` and gossips message envelopes to each other —
//! no central server, membership converges as peers exchange bootstrap
//! addresses. Good fit for a chat room: eventually-consistent broadcast,
//! self-healing as peers come and go.

use crate::protocol::message::Envelope;
use crate::ticket::topic_for_room;
use anyhow::Result;
use iroh::EndpointId;
use iroh_gossip::api::{Event, GossipSender};
use iroh_gossip::{Gossip, TopicId};
use n0_future::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tracing::warn;

/// Same rationale as `protocol::dm::NET_TIMEOUT` — bound any single network
/// call so a dead network fails fast and visibly instead of hanging on
/// QUIC's own (much longer) idle timeout.
const NET_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Clone)]
pub enum RoomEvent {
    Received {
        room: String,
        from: EndpointId,
        envelope: Envelope,
    },
    NeighborUp {
        room: String,
        peer: EndpointId,
    },
    NeighborDown {
        room: String,
        peer: EndpointId,
    },
    /// The gossip receiver fell behind and dropped some in-flight messages.
    /// There's no general way to recover exactly what was missed — this is
    /// surfaced so the UI can tell the user their view of the room may have
    /// a gap, rather than silently showing an incomplete conversation.
    Lagged {
        room: String,
    },
}

/// A joined room: keep the sender half around to broadcast, and a spawned
/// task drains the receiver half and forwards decoded events to the app.
/// `Clone` because `GossipSender` itself is a cheap cloneable handle — same
/// rationale as `DmSession`: lets a background task broadcast on its own
/// copy without the main loop giving up ownership.
#[derive(Clone)]
pub struct Room {
    name: String,
    sender: GossipSender,
}

impl Room {
    /// Join (or create, if nobody's there yet) the room `name`, optionally
    /// seeding with known bootstrap peers already in the swarm.
    pub async fn join(
        gossip: &Gossip,
        name: &str,
        bootstrap: Vec<EndpointId>,
        events: UnboundedSender<RoomEvent>,
    ) -> Result<Self> {
        let topic_id = TopicId::from_bytes(topic_for_room(name));
        let (sender, mut receiver) =
            tokio::time::timeout(NET_TIMEOUT, gossip.subscribe(topic_id, bootstrap))
                .await
                .map_err(|_| anyhow::anyhow!("joining room timed out after {NET_TIMEOUT:?}"))??
                .split();

        let room_name = name.to_string();
        tokio::spawn(async move {
            while let Some(event) = receiver.next().await {
                match event {
                    Ok(Event::Received(msg)) => match Envelope::decode(&msg.content) {
                        Ok(envelope) => {
                            let _ = events.send(RoomEvent::Received {
                                room: room_name.clone(),
                                from: msg.delivered_from,
                                envelope,
                            });
                        }
                        Err(e) => warn!(room = %room_name, error = %e, "gossip: bad envelope"),
                    },
                    Ok(Event::NeighborUp(peer)) => {
                        let _ = events.send(RoomEvent::NeighborUp {
                            room: room_name.clone(),
                            peer,
                        });
                    }
                    Ok(Event::NeighborDown(peer)) => {
                        let _ = events.send(RoomEvent::NeighborDown {
                            room: room_name.clone(),
                            peer,
                        });
                    }
                    Ok(Event::Lagged) => {
                        warn!(room = %room_name, "gossip: receiver lagged, missed messages");
                        let _ = events.send(RoomEvent::Lagged {
                            room: room_name.clone(),
                        });
                    }
                    Err(e) => {
                        warn!(room = %room_name, error = %e, "gossip: stream error");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            name: name.to_string(),
            sender,
        })
    }

    /// Kept as public API surface (not currently called — the TUI tracks
    /// room names itself in its sidebar list rather than asking `Room`).
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn broadcast(&self, envelope: &Envelope) -> Result<()> {
        tokio::time::timeout(
            NET_TIMEOUT,
            self.sender.broadcast(envelope.encode()?.into()),
        )
        .await
        .map_err(|_| anyhow::anyhow!("broadcast timed out after {NET_TIMEOUT:?}"))??;
        Ok(())
    }
}
