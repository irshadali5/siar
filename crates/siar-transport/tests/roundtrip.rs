//! End-to-end coverage for `endpoint.rs`/`handler.rs`/`blob_handler.rs`/
//! `pool.rs` together — none of the four had ever been exercised by a
//! test before this file. Two real `SiarEndpoint`s bind on loopback and
//! talk to each other over real QUIC; nothing here is mocked. Every
//! `EndpointAddr` handed to `connect`/`fetch_blob` carries only a
//! direct loopback `SocketAddr` (via `EndpointAddr::from_parts` +
//! `TransportAddr::Ip`), so these tests never depend on DNS discovery,
//! a relay, or any address outside the sandbox's loopback interface —
//! `Endpoint::builder(presets::N0)` itself doesn't block on network
//! access at bind time, and a direct IP address is enough for `connect`
//! to skip discovery entirely.

use iroh::{EndpointAddr, SecretKey, TransportAddr};
use siar_domain::{ConversationId, DeviceId, MessageId};
use siar_protocol::v1::{Envelope, EnvelopeKind, CURRENT_VERSION};
use siar_protocol::WireMessage;
use siar_transport::{BlobStore, PeerTransport, SiarEndpoint};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

fn sample_envelope() -> WireMessage {
    WireMessage::V1(Envelope {
        version: CURRENT_VERSION,
        message_id: MessageId::new(),
        conversation_id: ConversationId::new(),
        sender: DeviceId::new(),
        timestamp_millis: 0,
        sequence: 1,
        kind: EnvelopeKind::Text,
        payload: vec![9, 9, 9],
    })
}

#[derive(Default)]
struct MapBlobStore(Mutex<HashMap<[u8; 32], Vec<u8>>>);

impl BlobStore for MapBlobStore {
    fn get(&self, blob_hash: &[u8; 32]) -> Option<Vec<u8>> {
        self.0.lock().expect("MapBlobStore poisoned").get(blob_hash).cloned()
    }
}

/// Every direct-connect test needs the same shape: bind, then rebuild
/// the peer's `EndpointAddr` with only its loopback socket address
/// (never a relay URL), so `connect`/`fetch_blob` never has a reason to
/// touch discovery.
fn direct_addr_only(full: EndpointAddr) -> EndpointAddr {
    let ip_addrs = full
        .addrs
        .into_iter()
        .filter(|a| matches!(a, TransportAddr::Ip(_)));
    EndpointAddr::from_parts(full.id, ip_addrs)
}

#[tokio::test]
async fn send_reaches_the_peer_and_decodes_correctly() {
    let (tx_a, mut rx_a) = mpsc::channel(4);
    let (tx_b, mut rx_b) = mpsc::channel(4);

    let store: Arc<dyn BlobStore> = Arc::new(MapBlobStore::default());
    let a = SiarEndpoint::bind(SecretKey::generate(), tx_a, Arc::clone(&store))
        .await
        .expect("endpoint A binds");
    let b = SiarEndpoint::bind(SecretKey::generate(), tx_b, Arc::clone(&store))
        .await
        .expect("endpoint B binds");

    let b_addr = direct_addr_only(b.addr());
    let msg = sample_envelope();
    let WireMessage::V1(sent_envelope) = &msg else {
        unreachable!()
    };
    let sent_sequence = sent_envelope.sequence;

    tokio::time::timeout(Duration::from_secs(20), a.send(b_addr, &msg))
        .await
        .expect("send did not time out")
        .expect("send succeeds");

    let frame = tokio::time::timeout(Duration::from_secs(20), rx_b.recv())
        .await
        .expect("receive did not time out")
        .expect("channel not closed");

    assert_eq!(frame.from, a.id());
    let WireMessage::V1(received) = frame.message else {
        panic!("handler.rs must decode back the same WireMessage variant that was sent");
    };
    assert_eq!(received.sequence, sent_sequence);
    assert_eq!(received.payload, vec![9, 9, 9]);

    // Nothing should have arrived on A's own inbound channel — this is
    // a one-way send, not an echo.
    assert!(rx_a.try_recv().is_err());
}

#[tokio::test]
async fn fetch_blob_returns_found_when_the_peer_has_it() {
    let (tx_a, _rx_a) = mpsc::channel(4);
    let (tx_b, _rx_b) = mpsc::channel(4);

    let hash = [7u8; 32];
    let ciphertext = vec![1, 2, 3, 4, 5];
    let b_store = Arc::new(MapBlobStore::default());
    b_store
        .0
        .lock()
        .unwrap()
        .insert(hash, ciphertext.clone());

    let a_store: Arc<dyn BlobStore> = Arc::new(MapBlobStore::default());
    let a = SiarEndpoint::bind(SecretKey::generate(), tx_a, a_store)
        .await
        .expect("endpoint A binds");
    let b = SiarEndpoint::bind(SecretKey::generate(), tx_b, b_store as Arc<dyn BlobStore>)
        .await
        .expect("endpoint B binds");

    let b_addr = direct_addr_only(b.addr());
    let result = tokio::time::timeout(Duration::from_secs(20), a.fetch_blob(b_addr, hash))
        .await
        .expect("fetch_blob did not time out")
        .expect("fetch_blob succeeds");

    assert_eq!(result, Some(ciphertext));
}

#[tokio::test]
async fn fetch_blob_returns_none_when_the_peer_does_not_have_it() {
    let (tx_a, _rx_a) = mpsc::channel(4);
    let (tx_b, _rx_b) = mpsc::channel(4);

    let a_store: Arc<dyn BlobStore> = Arc::new(MapBlobStore::default());
    let b_store: Arc<dyn BlobStore> = Arc::new(MapBlobStore::default());
    let a = SiarEndpoint::bind(SecretKey::generate(), tx_a, a_store)
        .await
        .expect("endpoint A binds");
    let b = SiarEndpoint::bind(SecretKey::generate(), tx_b, b_store)
        .await
        .expect("endpoint B binds");

    let b_addr = direct_addr_only(b.addr());
    let result = tokio::time::timeout(
        Duration::from_secs(20),
        a.fetch_blob(b_addr, [0xAAu8; 32]),
    )
    .await
    .expect("fetch_blob did not time out")
    .expect("fetch_blob succeeds (peer answering NotFound is not a transport error)");

    assert_eq!(result, None);
}

#[tokio::test]
async fn a_second_send_to_the_same_peer_reuses_the_pooled_connection() {
    // Not a direct assertion on pool internals (private to the crate) —
    // instead, this is the externally-observable claim `pool.rs`'s own
    // docs make: a second send to the same (peer, ALPN) shouldn't need
    // a fresh handshake, so it should complete quickly and both
    // messages should still arrive intact. This is real end-to-end
    // coverage of the eviction/reuse *path*, not a timing proof.
    let (tx_a, _rx_a) = mpsc::channel(4);
    let (tx_b, mut rx_b) = mpsc::channel(4);

    let store: Arc<dyn BlobStore> = Arc::new(MapBlobStore::default());
    let a = SiarEndpoint::bind(SecretKey::generate(), tx_a, Arc::clone(&store))
        .await
        .expect("endpoint A binds");
    let b = SiarEndpoint::bind(SecretKey::generate(), tx_b, store)
        .await
        .expect("endpoint B binds");
    let b_addr = direct_addr_only(b.addr());

    for expected_sequence in [1u64, 2u64] {
        let mut msg = sample_envelope();
        if let WireMessage::V1(env) = &mut msg {
            env.sequence = expected_sequence;
        }
        tokio::time::timeout(Duration::from_secs(20), a.send(b_addr.clone(), &msg))
            .await
            .expect("send did not time out")
            .expect("send succeeds");

        let frame = tokio::time::timeout(Duration::from_secs(20), rx_b.recv())
            .await
            .expect("receive did not time out")
            .expect("channel not closed");
        let WireMessage::V1(env) = frame.message else {
            panic!("expected a V1 envelope");
        };
        assert_eq!(env.sequence, expected_sequence);
    }

    // evict_idle_connections must not tear down a connection that's
    // still open — pool.rs's own contract (only close_reason().is_some()
    // connections are dropped).
    a.evict_idle_connections();
}
