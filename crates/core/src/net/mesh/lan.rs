//! LAN mesh transport: flood `Envelope`s as UDP broadcast datagrams on
//! the local subnet. Works over any Wi-Fi network the device is already
//! joined to — including a phone-hosted hotspot with no uplink at all,
//! or two devices on the same router with no internet at all — since
//! broadcast only ever needs L2/L3 reachability on that one subnet,
//! never a route to the internet.
//!
//! Deliberately not multicast/mDNS: see `Cargo.toml`'s comment on why
//! broadcast is the one that needs no Android `MulticastLock`.
//!
//! Deliberately not a peer list either — there's no discovery/handshake
//! step. Every enabled node just listens on `PORT` and re-broadcasts
//! what it hears (via `MeshManager`'s dedup+TTL policy), so "join the
//! mesh" is exactly "start listening", nothing more.
//!
//! ## Why both the global and subnet-directed broadcast address
//!
//! `255.255.255.255` (the global broadcast address) is the simplest
//! thing to send to, but some Wi-Fi drivers and access points — and
//! this shows up on Android more than desktop Linux/Windows — filter
//! or drop it while still passing the *subnet-directed* broadcast
//! address for whatever network the device is actually on (e.g.
//! `192.168.1.255` for a `192.168.1.0/24` network). `if-addrs` (see
//! `Cargo.toml`) is used here to enumerate this device's local IPv4
//! interfaces and compute each one's real subnet broadcast address, so
//! `broadcast()` sends to the global address *and* every subnet-
//! directed one it can find — belt-and-suspenders where the belt
//! alone isn't reliable enough on every network this needs to work on.

use super::envelope::Envelope;
use super::{MeshInboundHandle, MeshStatus, MeshTransport};
use iroh::EndpointId;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::net::UdpSocket;

/// High, unassigned range — low collision odds with anything else
/// running on a home/office/hotspot LAN.
const PORT: u16 = 47_631;
/// Real payloads are chat-message scale; this covers the largest single
/// UDP datagram this transport will ever try to send without needing
/// its own fragmentation logic.
const MAX_DATAGRAM: usize = 60_000;

pub struct LanTransport {
    socket: Arc<UdpSocket>,
    /// The receive loop below is `tokio::spawn`ed, which means it's a
    /// fully independent task the moment it starts — dropping
    /// `LanTransport` on its own does nothing to it, it would just keep
    /// running (still holding its own `Arc<UdpSocket>` clone alive)
    /// forever, silently burning battery/CPU on a socket nothing is
    /// reading from anymore. `Drop` below aborts it explicitly, which is
    /// what actually makes `MeshManager::stop()` — and so toggling
    /// "Offline mesh" off in Settings — really stop this transport
    /// rather than just stop *routing new messages to* an already-
    /// running one.
    recv_task: tokio::task::JoinHandle<()>,
}

impl Drop for LanTransport {
    fn drop(&mut self) {
        self.recv_task.abort();
    }
}

impl LanTransport {
    pub async fn start(
        _my_id: EndpointId,
        status: Arc<MeshStatus>,
        inbound: MeshInboundHandle,
    ) -> anyhow::Result<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", PORT)).await?;
        socket.set_broadcast(true)?;
        let socket = Arc::new(socket);

        let recv_socket = socket.clone();
        let recv_task = tokio::spawn(async move {
            let mut buf = vec![0u8; MAX_DATAGRAM];
            loop {
                let (len, _from) = match recv_socket.recv_from(&mut buf).await {
                    Ok(v) => v,
                    Err(err) => {
                        tracing::debug!(?err, "mesh(lan): recv error");
                        continue;
                    }
                };
                match Envelope::decode(&buf[..len]) {
                    Ok(env) => {
                        status.note_peer_seen(env.sender.to_vec());
                        inbound.received(env).await;
                    }
                    Err(err) => tracing::debug!(?err, "mesh(lan): dropped undecodable datagram"),
                }
            }
        });

        Ok(Self { socket, recv_task })
    }
}

/// Every broadcast address worth sending this datagram to: the global
/// one, plus each local IPv4 interface's own subnet-directed one (see
/// module doc for why both). Recomputed on every call rather than
/// cached once at `start()` — this is chat-message cadence, not a hot
/// path, and recomputing means a Wi-Fi network change (new subnet,
/// new/removed interface) is picked up on the very next message with
/// no extra plumbing to invalidate a cache.
fn broadcast_targets() -> Vec<Ipv4Addr> {
    let mut targets = vec![Ipv4Addr::BROADCAST];
    let interfaces = match if_addrs::get_if_addrs() {
        Ok(v) => v,
        Err(err) => {
            tracing::debug!(?err, "mesh(lan): couldn't enumerate local interfaces, falling back to global broadcast only");
            return targets;
        }
    };
    for iface in interfaces {
        if iface.is_loopback() {
            continue;
        }
        let if_addrs::IfAddr::V4(v4) = iface.addr else {
            continue;
        };
        // `if-addrs` already asks the OS for this interface's broadcast
        // address where the OS provides one — trust that over computing
        // it by hand, and only fall back to the manual `ip | !netmask`
        // calculation (correct for any standard subnet) if the OS
        // didn't report one for this interface.
        let bcast = v4
            .broadcast
            .unwrap_or_else(|| Ipv4Addr::from(u32::from(v4.ip) | !u32::from(v4.netmask)));
        if !targets.contains(&bcast) {
            targets.push(bcast);
        }
    }
    targets
}

#[async_trait::async_trait]
impl MeshTransport for LanTransport {
    async fn broadcast(&self, envelope: &Envelope) -> anyhow::Result<()> {
        let bytes = envelope.encode()?;
        if bytes.len() > MAX_DATAGRAM {
            anyhow::bail!(
                "mesh(lan): envelope too large for one datagram ({} bytes)",
                bytes.len()
            );
        }
        // Send to every target rather than stopping at the first
        // success — different interfaces (Wi-Fi vs. a wired/USB-
        // tether one, if both are up) each need their own broadcast,
        // and this is a flood protocol anyway: an extra duplicate
        // datagram on a network that has none of these peers is
        // silently absorbed by `MeshManager`'s dedup cache on receipt,
        // not a correctness problem.
        let mut sent_any = false;
        let mut last_err = None;
        for target in broadcast_targets() {
            match self.socket.send_to(&bytes, (target, PORT)).await {
                Ok(_) => sent_any = true,
                Err(err) => last_err = Some(err),
            }
        }
        if sent_any {
            Ok(())
        } else {
            Err(last_err
                .map(anyhow::Error::from)
                .unwrap_or_else(|| anyhow::anyhow!("mesh(lan): no broadcast target reachable")))
        }
    }

    fn name(&self) -> &'static str {
        "lan"
    }
}
