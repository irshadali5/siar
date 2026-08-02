//! The wire format. Both the direct-message protocol and gossip rooms send
//! the same `Envelope`. v2 adds:
//!
//!   - A one-byte adaptive-compression frame around the postcard bytes
//!     (see `encode`/`decode`) — compression is applied only when it
//!     actually shrinks the payload, so short messages never pay zstd's
//!     fixed overhead.
//!   - `Body::File`, an *announcement* of a file available over
//!     `iroh-blobs` (see `net::transfer`) — the envelope never carries the
//!     file bytes themselves, only enough metadata to fetch and, if
//!     needed, decompress them.
//!
//! Every envelope still carries a random `id`, used the same way as before
//! to correlate `Body::Ack` with the `Text`/`File` envelope it confirms —
//! a successful local `send()` on a QUIC stream only proves the bytes were
//! handed to the local connection, not that they reached the peer.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Cap on a single encoded (post-decompression) message, to stop a peer
/// from making us buffer unbounded memory when reading a uni stream to
/// completion. File bodies are announcements only, so this limit is about
/// text/metadata size, not file size.
pub const MAX_MESSAGE_BYTES: usize = 256 * 1024;

/// Below this size, don't even try zstd — the frame-tag byte plus zstd's
/// own minimal header overhead reliably costs more than it saves on tiny
/// payloads (a "hi" or a Typing/Ack envelope).
const COMPRESSION_MIN_INPUT: usize = 128;

const ZSTD_LEVEL: i32 = 3; // fast; chat messages are latency-sensitive, not archival

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub id: u64,
    /// Sender's display name at time of sending (not authenticated — the
    /// cryptographic identity is the connection's peer EndpointId, this is
    /// just for display).
    pub from_name: String,
    pub sent_unix_ms: u64,
    /// Disappearing messages: absolute expiry, computed by the *sender*
    /// from the conversation's synced TTL policy
    /// (`net::conv_docs::DmSettings::disappearing_ttl_secs`) and carried
    /// here so every recipient (and the sender's own local echo) stores
    /// and enforces the identical timestamp — rather than each side
    /// separately computing "now + ttl" off its own clock and drifting.
    /// `None` means "doesn't expire."
    pub expires_at_unix_ms: Option<u64>,
    pub body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Body {
    Text {
        text: String,
        /// `Envelope::id` of the message this one quotes, if any — the
        /// reply target's own content isn't re-sent here, just its id;
        /// whoever renders this looks the target up in their own
        /// already-stored history (see `store::StoredMessage::reply_to_envelope_id`
        /// and `ui::chat`'s reply-preview rendering). If the target
        /// isn't in the recipient's history at all (e.g. it predates
        /// them joining a room), the UI falls back to a generic
        /// "original message unavailable" line rather than erroring.
        reply_to: Option<u64>,
    },
    /// Announces a file available over iroh-blobs. `blake3_hash` is the
    /// hex-encoded root hash of the (possibly zstd-compressed) blob;
    /// `compressed` tells the receiver whether to zstd-decompress after
    /// the BLAKE3-verified download completes. `size_bytes` is the size of
    /// the blob as stored (i.e. post-compression, if compressed), so a
    /// progress bar can be accurate about what's actually being fetched.
    File {
        name: String,
        mime: String,
        size_bytes: u64,
        compressed: bool,
        blake3_hash: String,
        /// Same meaning as `Text::reply_to` above.
        reply_to: Option<u64>,
    },
    /// Sent once when a DM session opens, so the receiving side can show a
    /// friendly name instead of a raw ticket before the first text message.
    Hello,
    /// Confirms receipt of the `Text`/`File` envelope whose id is carried here.
    Ack(u64),
    /// Best-effort "I'm typing" signal — not acked, not retried, not
    /// logged to history. Fine to drop; it's purely a transient UI hint.
    Typing,
    /// Connection keepalive — see `ui::spawn_dm_keepalive`. Not acked, not
    /// logged, not shown; the only thing that matters is whether *sending*
    /// one succeeds, which is how a half-dead session gets noticed before
    /// the user's next real message hits it.
    Ping,
    /// A status update (WhatsApp/Signal "story"-style) — broadcast to
    /// every accepted contact over their existing DM session (see
    /// `app::broadcast_status`), not tied to any one conversation, so it's
    /// not logged to per-conversation history the way `Text`/`File` are.
    /// `expires_at_unix_ms` on the envelope is always `Some` for this
    /// variant — statuses always expire, never persist indefinitely.
    /// `image` is optional and, when present, is announce-not-payload —
    /// same shape as `AvatarUpdate` below: the canonical PNG bytes (see
    /// `media::decode_status_image`) live in the sender's local
    /// `iroh-blobs` store and are fetched on demand by whoever's actually
    /// viewing the status, not attached to every copy of this envelope.
    Status {
        text: String,
        image: Option<StatusImage>,
        video: Option<StatusVideo>,
        audio: Option<StatusAudio>,
    },
    /// Announces a new display picture, same announcement-not-payload
    /// shape as `Body::File`: the actual PNG bytes live in the local
    /// `iroh-blobs` store (see `net::transfer::fetch_avatar_bytes`), this
    /// just tells contacts a new one exists and how to fetch it. Broadcast
    /// to every accepted contact the same way `Status` is (see
    /// `app::broadcast_avatar_update`) whenever `App::set_my_avatar`
    /// changes it — not sent unprompted otherwise, so a peer's first
    /// avatar fetch always happens lazily (download-on-demand) rather
    /// than every contact eagerly pulling everyone's picture up front.
    AvatarUpdate {
        blake3_hash: String,
        size_bytes: u64,
    },
    /// Adds or removes an emoji reaction on a previously sent message.
    /// `target_id` is that message's `Envelope::id`. `remove: true`
    /// means "take my reaction back off" (tapping the same emoji
    /// again) — not "the message was deleted", that's `Delete` below.
    Reaction {
        target_id: u64,
        emoji: String,
        remove: bool,
    },
    /// Replaces a previously sent message's text in place. `target_id`
    /// is the original envelope's id. Only meaningful (and only
    /// applied) when it arrives from the same sender who owns
    /// `target_id` — enforced on the receiving end by checking against
    /// the stored row's `sender_id`, the same trust boundary as
    /// everything else here (there's no separate per-message signature
    /// on the wire beyond "which authenticated QUIC connection this
    /// arrived on").
    Edit { target_id: u64, new_text: String },
    /// Tombstones a previously sent message. Everyone who already has a
    /// local copy keeps whatever they already stored/rendered up to
    /// this point — this is "delete for everyone going forward", the
    /// same semantics every other messenger's delete has, not a promise
    /// to erase it from history that already synced.
    Delete { target_id: u64 },
    /// "I've now seen everything you sent up to and including this
    /// timestamp." Deliberately a coarser timestamp watermark rather
    /// than acking each message's id individually: this is a read
    /// receipt (a privacy-sensitive, user-visible signal someone might
    /// want to turn off later), not `Ack` (a delivery-confirmation
    /// implementation detail nobody sees). DM-only for now — see
    /// `Store::set_read_watermark`'s doc for why room read receipts
    /// (which member has seen which message, in an interleaved
    /// multi-sender stream) are a separate, harder problem left for
    /// later rather than bolted on here.
    Read { up_to_sent_unix_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusImage {
    pub blake3_hash: String,
    pub size_bytes: u64,
}

/// A short recorded video status clip — announce-not-payload, same shape
/// as `StatusImage`. The blob it points to is an AV1 clip produced by
/// `net::calls::video::encode_clip` (postcard-encoded `Vec<Vec<u8>>` of
/// raw temporal units, no container format), decoded back with
/// `net::calls::video::decode_clip` — the same encoder/decoder live
/// calls use, not a second AV1 implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusVideo {
    pub blake3_hash: String,
    pub size_bytes: u64,
}

/// A short recorded voice status clip — announce-not-payload, same shape
/// as `StatusImage`/`StatusVideo`. The blob it points to is an Opus clip
/// produced by `net::calls::audio::record_and_encode_voice_clip`
/// (postcard-encoded `Vec<Vec<u8>>` of raw Opus packets), decoded back
/// with `net::calls::audio::decode_voice_clip` — the same encoder/
/// decoder live calls use, not a second Opus implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusAudio {
    pub blake3_hash: String,
    pub size_bytes: u64,
}

impl Envelope {
    pub fn text(from_name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::Text {
                text: text.into(),
                reply_to: None,
            },
        }
    }

    /// Same as `text()`, quoting an earlier message. A separate
    /// constructor rather than a third `text()` parameter every existing
    /// call site would need to update to `None` — most sends aren't
    /// replies.
    pub fn text_reply(
        from_name: impl Into<String>,
        text: impl Into<String>,
        reply_to: u64,
    ) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::Text {
                text: text.into(),
                reply_to: Some(reply_to),
            },
        }
    }

    /// Attach a disappearing-message expiry, `ttl_secs` after this
    /// envelope's own `sent_unix_ms` — call right after `text()`/`file()`
    /// when the conversation has a TTL configured. A no-op builder-style
    /// method (takes/returns `Self`) so call sites can stay a single
    /// expression: `Envelope::text(name, body).with_expiry(ttl)`.
    pub fn with_expiry(mut self, ttl_secs: Option<u64>) -> Self {
        self.expires_at_unix_ms = ttl_secs.map(|secs| self.sent_unix_ms + secs * 1000);
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn file(
        from_name: impl Into<String>,
        name: impl Into<String>,
        mime: impl Into<String>,
        size_bytes: u64,
        compressed: bool,
        blake3_hash: impl Into<String>,
        reply_to: Option<u64>,
    ) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::File {
                name: name.into(),
                mime: mime.into(),
                size_bytes,
                compressed,
                blake3_hash: blake3_hash.into(),
                reply_to,
            },
        }
    }

    pub fn hello(from_name: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::Hello,
        }
    }

    pub fn ack(from_name: impl Into<String>, acked_id: u64) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::Ack(acked_id),
        }
    }

    pub fn reaction(
        from_name: impl Into<String>,
        target_id: u64,
        emoji: impl Into<String>,
        remove: bool,
    ) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::Reaction {
                target_id,
                emoji: emoji.into(),
                remove,
            },
        }
    }

    pub fn edit(from_name: impl Into<String>, target_id: u64, new_text: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::Edit {
                target_id,
                new_text: new_text.into(),
            },
        }
    }

    pub fn delete(from_name: impl Into<String>, target_id: u64) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::Delete { target_id },
        }
    }

    pub fn read_receipt(from_name: impl Into<String>, up_to_sent_unix_ms: u64) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::Read { up_to_sent_unix_ms },
        }
    }

    pub fn typing(from_name: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::Typing,
        }
    }

    pub fn ping(from_name: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::Ping,
        }
    }

    pub fn status(
        from_name: impl Into<String>,
        text: impl Into<String>,
        image: Option<StatusImage>,
        video: Option<StatusVideo>,
        audio: Option<StatusAudio>,
        expires_at_unix_ms: u64,
    ) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: Some(expires_at_unix_ms),
            body: Body::Status {
                text: text.into(),
                image,
                video,
                audio,
            },
        }
    }

    pub fn avatar_update(
        from_name: impl Into<String>,
        blake3_hash: impl Into<String>,
        size_bytes: u64,
    ) -> Self {
        Self {
            id: next_id(),
            from_name: from_name.into(),
            sent_unix_ms: now_unix_ms(),
            expires_at_unix_ms: None,
            body: Body::AvatarUpdate {
                blake3_hash: blake3_hash.into(),
                size_bytes,
            },
        }
    }

    /// Postcard-serialize, then adaptively zstd-compress: a one-byte tag
    /// (`0x00` raw, `0x01` zstd) is prepended, and zstd is only kept if it
    /// actually made the payload smaller. This is the single choke point
    /// compression flows through, so every body kind benefits uniformly —
    /// callers never decide per-message whether to compress.
    pub fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let raw = postcard::to_stdvec(self)?;

        if raw.len() < COMPRESSION_MIN_INPUT {
            return Ok(tagged(0x00, raw));
        }

        match zstd::stream::encode_all(raw.as_slice(), ZSTD_LEVEL) {
            Ok(compressed) if compressed.len() < raw.len() => Ok(tagged(0x01, compressed)),
            _ => Ok(tagged(0x00, raw)),
        }
    }

    pub fn decode(bytes: &[u8]) -> anyhow::Result<Self> {
        let (tag, payload) = bytes
            .split_first()
            .ok_or_else(|| anyhow::anyhow!("empty envelope"))?;

        let raw = match tag {
            0x00 => payload.to_vec(),
            0x01 => zstd::stream::decode_all(payload)
                .map_err(|e| anyhow::anyhow!("zstd decompress failed: {e}"))?,
            other => anyhow::bail!("unknown envelope compression tag {other}"),
        };

        if raw.len() > MAX_MESSAGE_BYTES {
            anyhow::bail!("decoded envelope exceeds {MAX_MESSAGE_BYTES} bytes");
        }

        Ok(postcard::from_bytes(&raw)?)
    }
}

fn tagged(tag: u8, mut payload: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 1);
    out.push(tag);
    out.append(&mut payload);
    out
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Locally-unique id for correlating an `Ack` with the envelope it
/// confirms. Just needs uniqueness within this process's lifetime, not
/// cryptographic randomness — a counter mixed with the timestamp covers
/// that without a dedicated RNG dependency.
static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_id() -> u64 {
    let counter = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    now_unix_ms().wrapping_mul(1_000_003).wrapping_add(counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_message_stays_uncompressed() {
        let env = Envelope::text("bob", "hi");
        let bytes = env.encode().unwrap();
        assert_eq!(bytes[0], 0x00, "tiny payloads shouldn't pay zstd overhead");
        let back = Envelope::decode(&bytes).unwrap();
        assert!(matches!(back.body, Body::Text { text, .. } if text == "hi"));
    }

    #[test]
    fn large_repetitive_message_compresses() {
        let long = "the quick brown fox jumps over the lazy dog ".repeat(50);
        let env = Envelope::text("bob", long.clone());
        let bytes = env.encode().unwrap();
        assert_eq!(
            bytes[0], 0x01,
            "highly compressible text should be tagged zstd"
        );
        let back = Envelope::decode(&bytes).unwrap();
        assert!(matches!(back.body, Body::Text { text, .. } if text == long));
    }

    #[test]
    fn file_announcement_roundtrips() {
        let env = Envelope::file(
            "bob",
            "report.pdf",
            "application/pdf",
            4096,
            false,
            "deadbeef",
            None,
        );
        let bytes = env.encode().unwrap();
        let back = Envelope::decode(&bytes).unwrap();
        match back.body {
            Body::File {
                name, size_bytes, ..
            } => {
                assert_eq!(name, "report.pdf");
                assert_eq!(size_bytes, 4096);
            }
            _ => panic!("expected File body"),
        }
    }
}
