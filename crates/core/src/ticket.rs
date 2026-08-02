//! Shareable "tickets" — the thing you paste into Signal/WhatsApp/email to
//! give someone your address directly, without going through the username
//! registry at all. The username registry (`net::registry`) is the
//! *primary* way people find each other in v2, but a ticket is still the
//! right fallback for "the registry hasn't synced yet" or "I don't have a
//! username picked yet."
//!
//! v2.1 change: a ticket now carries a full `EndpointAddr` (the endpoint's
//! current relay URL and direct addresses), not just the bare 32-byte
//! `EndpointId`. The original design followed iroh's general advice that
//! discovery makes bare IDs enough to dial — true for peers discovery
//! already knows about, but a ticket's whole point is introducing someone
//! discovery has *never seen*, right now, in person. In that case a bare
//! ID means the receiving side's `connect()` has nothing to try except
//! wait on pkarr/DNS discovery to learn an address it may not have
//! published yet — which is exactly the "contact request connect timed
//! out" failures this change fixes. Embedding the address is strictly
//! additive: iroh still falls back to discovery automatically if the
//! embedded hints turn out stale (the person moved networks since
//! generating the ticket), so this can only help, never hurt, compared to
//! the ID-only version.
//!
//! This does mean a ticket is a point-in-time snapshot rather than a
//! long-lived identifier — a ticket generated weeks ago and pasted today
//! just degrades gracefully back to discovery-only behavior, same as
//! before this change.
//!
//! A pasted ticket goes through the same request/accept flow as a
//! username-search result (see `net::contacts`) — it's a different way to
//! *find* someone, not a way to skip their consent.

use anyhow::{bail, Result};
use data_encoding::BASE32_NOPAD;
use iroh::{EndpointAddr, EndpointId};

const PREFIX: &str = "mtkt1"; // "messenger ticket, version 1"

/// Encode an `EndpointAddr` (id + current relay/direct addresses) as a
/// shareable ticket string.
pub fn encode(addr: EndpointAddr) -> Result<String> {
    let bytes = postcard::to_stdvec(&addr)?;
    Ok(format!(
        "{PREFIX}{}",
        BASE32_NOPAD.encode(&bytes).to_lowercase()
    ))
}

/// Decode a ticket string back into an `EndpointAddr`.
pub fn decode(ticket: &str) -> Result<EndpointAddr> {
    let ticket = ticket.trim();
    let Some(rest) = ticket.strip_prefix(PREFIX) else {
        bail!("not a valid messenger ticket (expected it to start with `{PREFIX}`)");
    };
    let bytes = BASE32_NOPAD
        .decode(rest.to_uppercase().as_bytes())
        .map_err(|e| anyhow::anyhow!("malformed ticket: {e}"))?;
    postcard::from_bytes(&bytes).map_err(|e| anyhow::anyhow!("malformed ticket: {e}"))
}

/// Derive a stable 32-byte gossip `TopicId` from a human-readable room name,
/// so anyone who types the same room name converges on the same swarm.
pub fn topic_for_room(name: &str) -> [u8; 32] {
    *blake3::hash(name.trim().to_lowercase().as_bytes()).as_bytes()
}

/// Derive a stable `iroh-docs` `NamespaceSecret` for a room's *metadata*
/// document (title, membership) from its human-readable name — same idea as
/// `topic_for_room`, but domain-separated so the two never collide even
/// though both are hashes of the same input string. Every member who types
/// the same room name independently arrives at the identical namespace
/// secret, so there's no ticket to pass around and no "who creates it"
/// coordination problem: the first person to type a room name and the
/// hundredth both derive the same writable namespace on their own.
///
/// Anyone who knows the room name can therefore write to its metadata doc —
/// same trust model as the gossip topic itself (knowing the name *is* the
/// invitation) and consistent with `net::registry`'s "namespace doesn't
/// gate who can write, application logic does" approach. Membership
/// tombstones (`net::conv_docs::RoomDoc::remove_member`) are an
/// application-level convention enforced by the UI (only shown as available
/// to the room's recorded `admin`), not a namespace-level ACL.
pub fn namespace_secret_for_room(name: &str) -> [u8; 32] {
    *blake3::hash(format!("iroh-messenger/room-meta/v1/{}", name.trim().to_lowercase()).as_bytes())
        .as_bytes()
}

/// Derive a stable `iroh-docs` `NamespaceSecret` for a 1:1 conversation's
/// metadata document (shared nickname/title, pinned/archived, disappearing-
/// message TTL) from the pair of endpoint IDs involved. Sorting the pair
/// before hashing makes the derivation symmetric — both sides compute the
/// exact same 32 bytes independently, without either one generating a
/// namespace and shipping the other a ticket first (which would need its
/// own delivery channel before the DM channel even exists).
///
/// This document only ever has two legitimate writers in practice (the two
/// parties of the DM); nothing stops a third party who somehow learned both
/// endpoint IDs from also deriving this secret, but doing so only lets them
/// write graffiti into a metadata doc they'd have no way to get either
/// party to look at over the actual DM/contact-accepted trust boundary —
/// the message content itself never lives here (see module doc on
/// `net::conv_docs`), so there's nothing sensitive to leak or corrupt.
pub fn namespace_secret_for_dm(a: EndpointId, b: EndpointId) -> [u8; 32] {
    let (lo, hi) = if a.as_bytes() <= b.as_bytes() {
        (a, b)
    } else {
        (b, a)
    };
    let mut input = b"iroh-messenger/dm-meta/v1/".to_vec();
    input.extend_from_slice(lo.as_bytes());
    input.extend_from_slice(hi.as_bytes());
    *blake3::hash(&input).as_bytes()
}

/// A room ticket: the room's name plus one member's current address, so a
/// second device has an actual peer to dial instead of just a topic ID it
/// hopes someone else independently subscribed to.
///
/// `topic_for_room`/`namespace_secret_for_room` above are deliberately
/// name-derived so *knowing the name is the invitation* — no ticket
/// needed in principle. In practice that promise only holds if there's
/// some way to find another peer already subscribed to the same topic,
/// and neither `iroh-gossip`'s `subscribe(topic, bootstrap)` nor
/// `iroh-docs` sync has a "find anyone else who derived this same ID"
/// mechanism — both need an actual `EndpointId`/address to dial. Two
/// people independently typing the same room name with an empty
/// bootstrap list each just sit alone on their own copy of that topic,
/// which is exactly the "creates a new room instead of joining" bug this
/// fixes: the name alone was never enough to connect two *independent*
/// swarms, only to agree on which topic/namespace to use once connected.
/// A ticket supplies the missing piece — a real peer to bootstrap from —
/// the same way `net::registry::BOOTSTRAP_REGISTRY_PEERS` solves the
/// identical cold-start problem for the username registry.
const ROOM_PREFIX: &str = "mrtk1"; // "messenger room ticket, version 1"

#[derive(serde::Serialize, serde::Deserialize)]
struct RoomTicketPayload {
    name: String,
    host: EndpointAddr,
}

/// Encode a room ticket: whoever creates a room (the first person to type
/// its name into "Create room") shares this, not just the bare name, with
/// whoever they want to invite — the same way a contact ticket is shared,
/// not the bare `EndpointId`.
pub fn encode_room(name: &str, host: EndpointAddr) -> Result<String> {
    let payload = RoomTicketPayload {
        name: name.trim().to_string(),
        host,
    };
    let bytes = postcard::to_stdvec(&payload)?;
    Ok(format!(
        "{ROOM_PREFIX}{}",
        BASE32_NOPAD.encode(&bytes).to_lowercase()
    ))
}

/// Decode a room ticket back into `(room name, host's address)`. The
/// caller adds `host` to the endpoint's known addresses and passes
/// `host.id` as the gossip bootstrap peer — see `spawn_join_room`.
pub fn decode_room(ticket: &str) -> Result<(String, EndpointAddr)> {
    let ticket = ticket.trim();
    let Some(rest) = ticket.strip_prefix(ROOM_PREFIX) else {
        bail!("not a valid room ticket (expected it to start with `{ROOM_PREFIX}`)");
    };
    let bytes = BASE32_NOPAD
        .decode(rest.to_uppercase().as_bytes())
        .map_err(|e| anyhow::anyhow!("malformed room ticket: {e}"))?;
    let payload: RoomTicketPayload =
        postcard::from_bytes(&bytes).map_err(|e| anyhow::anyhow!("malformed room ticket: {e}"))?;
    Ok((payload.name, payload.host))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = iroh::SecretKey::generate();
        let id = key.public();
        // Hint-less address (as if generated before the endpoint ever
        // bound / learned any relay info) — the minimum case, same as
        // what the old ID-only format covered.
        let addr = EndpointAddr::from(id);
        let ticket = encode(addr).unwrap();
        assert!(ticket.starts_with(PREFIX));
        let decoded = decode(&ticket).unwrap();
        assert_eq!(id, decoded.id); // `EndpointAddr::id` — confirmed via rustc.
    }

    #[test]
    fn room_topic_is_case_and_whitespace_insensitive() {
        assert_eq!(topic_for_room("Rust Lang"), topic_for_room("  rust lang  "));
    }

    #[test]
    fn room_ticket_roundtrip() {
        let key = iroh::SecretKey::generate();
        let host = EndpointAddr::from(key.public());
        let ticket = encode_room("Rust Lang", host.clone()).unwrap();
        assert!(ticket.starts_with(ROOM_PREFIX));
        let (name, decoded_host) = decode_room(&ticket).unwrap();
        assert_eq!(name, "Rust Lang");
        assert_eq!(decoded_host.id, host.id);
    }
}
