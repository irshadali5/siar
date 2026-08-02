//! Voice + video call signaling and lifecycle.
//!
//! Audio: Opus only, unchanged from the original audio-only pass — see
//! `audio`'s own module doc for the full design and its "hardware vs.
//! software" answer. Nothing in this file's audio path was touched.
//!
//! Video: AV1 on desktop via `video`/`rav1e`/`dav1d`, plus negotiated
//! H.264/H.265/AV1 MediaCodec support on Android.
//!
//! The module's first pass used VP9/libvpx and broke in a way
//! that turned out to be structural, not incidental — see `video`'s own
//! module doc for the full story and why AV1 was picked as the
//! deliberately more binding-robust replacement, along with the honest
//! answer on hardware acceleration and the performance/efficiency
//! tuning that went into the encode side specifically.
//!
//! Design — one QUIC `Connection` per call for signaling + audio, exactly
//! as before:
//!   1. A single bidi stream for the initial invite/answer handshake.
//!      Closes itself once that one round trip is done — it isn't reused.
//!   2. Two persistent uni streams for audio, one per direction, opened
//!      only after the callee accepts. Frames are length-prefixed and
//!      written continuously for the call's duration — *not* one QUIC
//!      stream per 20ms frame, which would be absurd overhead.
//!
//! Video, when negotiated, rides its *own separate* `Connection` on
//! `video::ALPN`, rather than sharing uni streams on the call connection
//! above. That's not incidental complexity — it's required by a real
//! constraint: `audio::receive_and_decode` already owns an unbounded loop
//! of `connection.accept_uni()` calls on the call connection, and iroh/
//! QUIC hands each arriving uni stream to exactly one waiting acceptor.
//! Adding a second `accept_uni()` loop for video frames on that *same*
//! connection would race the audio one for every incoming stream, with no
//! way to tell which loop should get which stream short of a tag-byte
//! convention baked into both — which would mean modifying `audio.rs`'s
//! already-working receive loop. A second connection sidesteps the whole
//! problem for a small, one-time extra handshake cost per call, which is
//! nothing next to a call's actual duration.
//!
//! The video connection needs a small hand-off on the callee side: the
//! caller dials `video::ALPN` directly (it already knows `peer`'s address
//! — it's mid-call with them), but the callee's *audio* protocol handler
//! (`CallProtocol::accept`, below) is the one that knows this call
//! negotiated video and is waiting for it — a *different*, automatically-
//! invoked handler (`video::VideoCallProtocol::accept`) is what the
//! Router actually calls when that inbound connection lands. `video_handoff`
//! (a single shared slot, not a registry — this app only ever has one call
//! active at a time, an invariant already relied on elsewhere, e.g.
//! `ui::mod`'s "already on a call" guards) is how the second hands the
//! `Connection` back to the first.
//!
//! No separate `Hangup` message: ending the call means finishing your own
//! outgoing audio stream, which the peer sees as a clean EOF on their
//! incoming stream — that alone means "call over," so there's nothing
//! else to signal or get out of sync. Video's `run_video_session` follows
//! the same convention.
//!
//! What's still missing, deliberately out of scope for this pass: no
//! jitter buffer (frames play back in arrival order, no reordering/
//! concealment for a late one), no acoustic echo cancellation, and audio
//! rides an *ordered* QUIC stream rather than unreliable datagrams — the
//! wrong tradeoff for real-time audio on a lossy link (a dropped frame
//! should be skipped, not retransmitted, stalling everything behind it).
//! `Connection::send_datagram`/`read_datagram` is the right fix; deferred
//! here because it's iroh API surface this codebase hasn't exercised
//! anywhere else, so it deserves its own compile-checked pass rather than
//! going in bundled with everything else in this one. Video inherits the
//! same ordered-stream tradeoff for the same reason, one frame per uni
//! stream (see `video`'s doc) rather than per-frame datagrams.

pub mod audio;
pub mod video;

#[cfg(target_os = "android")]
mod mediacodec;

/// What hardware video codecs this device can **decode**, beyond the
/// This is what goes out over the wire in `CallMsg`'s `decode_codecs`
/// field: it's what tells a *peer* which codecs are safe to send this
/// device. Real on Android (`mediacodec::available_hw_decode_codecs`, an
/// actual `AMediaCodec` capability probe); unconditionally empty
/// everywhere else for now — desktop's `ffmpeg`-subprocess `hw` module
/// (see `video.rs`) only wires up hardware *encode* candidates today, not
/// a verified hardware-decode probe for H.264/H.265 specifically. Desktop
/// always includes AV1 because dav1d is compiled into this build.
#[cfg(target_os = "android")]
pub use mediacodec::available_hw_decode_codecs;

#[cfg(not(target_os = "android"))]
pub fn available_hw_decode_codecs() -> Vec<VideoCodec> {
    vec![VideoCodec::Av1]
}

/// What this device can **encode** into hardware — used only locally,
/// on whichever side is about to pick a codec to send with (see
/// `negotiate_send_codec`); never put on the wire. A peer doesn't need
/// to know what this device can encode, only what it can decode — see
/// `VideoCodec`'s doc for why conflating the two was a real bug in an
/// earlier version of this negotiation.
#[cfg(target_os = "android")]
pub use mediacodec::available_hw_encode_codecs;

#[cfg(not(target_os = "android"))]
pub fn available_hw_encode_codecs() -> Vec<VideoCodec> {
    vec![VideoCodec::Av1]
}

use anyhow::{bail, Context, Result};
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh::{Endpoint, EndpointId};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tracing::warn;

/// Bumped to `/3` when `CallMsg` began carrying separate encode and
/// decode capability lists. `video::ALPN` is `/2` because frames now
/// carry their negotiated codec. Postcard is a compact positional format with no
/// wire-level field names/versioning of its own, so a struct shape
/// change is a breaking wire change here, same as every previous one in
/// this codebase's history handled it: a new ALPN string means an old
/// and new build simply fail to negotiate a connection at all (a clean,
/// loud failure) rather than one silently misparsing the other's bytes.
pub const ALPN: &[u8] = b"iroh-messenger/call/3";
const SIGNAL_TIMEOUT: Duration = Duration::from_secs(30); // covers a human noticing the ring and clicking Accept
const MAX_FRAME_BYTES: usize = 1024; // signaling messages are tiny fixed shapes
/// How long the callee side waits for the caller's separate video
/// connection to actually arrive, once both ends have agreed video is
/// happening, before giving up and continuing audio-only. Generous next
/// to `SIGNAL_TIMEOUT` since by this point the human has already picked
/// up — this is purely "how long can the second QUIC handshake
/// reasonably take," not "how long can a human take to answer."
const VIDEO_HANDOFF_TIMEOUT: Duration = Duration::from_secs(10);

/// Single shared slot for handing the callee's inbound `video::ALPN`
/// connection from `video::VideoCallProtocol::accept` (invoked
/// automatically by the Router) over to the in-progress `CallProtocol::accept`
/// call that's waiting for it. See this module's own doc for why one slot
/// (not a per-peer registry) is the right amount of complexity here.
pub type VideoHandoff = Arc<Mutex<Option<oneshot::Sender<Connection>>>>;

/// A video codec this build can encode into and/or decode out of, for
/// the wire. Capabilities are explicit, including AV1: desktop has a
/// software AV1 baseline, while Android may only expose hardware codecs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoCodec {
    Av1,
    H264,
    H265,
}

/// Picks the codec **this device** should encode outgoing video with,
/// for one direction of a call. Directional and per-side by design —
/// this is the fix for a real negotiation bug an earlier version of this
/// had: it compared both sides' *encode* capability against each other,
/// when what actually matters for "is it safe to send X" is whether the
/// **receiver** can *decode* X, a completely different question from
/// what either side can *encode*. That distinction is concretely
/// important for AV1 on Android: hardware AV1 *decode* is meaningfully
/// more common than hardware AV1 *encode* on today's devices, so a
/// device that can only decode AV1 still deserves to receive it, even
/// though it could never have been "offered" AV1 under encode-vs-encode
/// comparison.
///
/// Because each side computes its own send-codec independently — this
/// device's `my_encode_codecs` against the peer's advertised
/// `decode_codecs` — the two directions of one call can genuinely differ
/// (Android→desktop might use `Av1`, since desktop can always software-
/// decode it, while desktop→Android might use `H265` if the Android
/// device's hardware decodes it and this build can encode it). That's
/// correct, not an inconsistency to paper over: a symmetric single-codec
/// requirement would be strictly worse for battery/bandwidth on whichever
/// side has the better hardware, for no real benefit.
///
/// Prefers `H265` over `H264` over `Av1` when there's a real choice:
/// H.265 needs meaningfully less bitrate for the same quality, which on
/// a battery-sensitive mobile device matters for radio (not just codec)
/// power draw — smaller packets, less to encrypt/send/receive. `None`
/// is deliberate: silently falling back to AV1 was unsafe for Android
/// devices without an AV1 decoder.
pub fn negotiate_send_codec(
    my_encode_codecs: &[VideoCodec],
    their_decode_codecs: &[VideoCodec],
) -> Option<VideoCodec> {
    [VideoCodec::H265, VideoCodec::H264, VideoCodec::Av1]
        .into_iter()
        .find(|&preferred| {
            my_encode_codecs.contains(&preferred) && their_decode_codecs.contains(&preferred)
        })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CallMsg {
    Invite {
        from_id: [u8; 32],
        from_name: String,
        want_video: bool,
        encode_codecs: Vec<VideoCodec>,
        decode_codecs: Vec<VideoCodec>,
    },
    Answer {
        accepted: bool,
        video_accepted: bool,
        encode_codecs: Vec<VideoCodec>,
        decode_codecs: Vec<VideoCodec>,
    },
}

impl CallMsg {
    fn encode(&self) -> Result<Vec<u8>> {
        Ok(postcard::to_stdvec(self)?)
    }
    fn decode(bytes: &[u8]) -> Result<Self> {
        Ok(postcard::from_bytes(bytes)?)
    }
}

/// What the UI layer needs to know about, and act on, an in-progress call.
#[derive(Debug)]
pub enum CallEvent {
    /// Someone is calling us. The UI should ring and offer Accept/Decline;
    /// whichever the person picks must go into `decision` exactly once
    /// (`true` = accept). The `accept()` handler below is blocked on the
    /// other end of that channel — nothing about the call proceeds until
    /// it hears back, bounded by `SIGNAL_TIMEOUT` regardless.
    Incoming {
        from_id: EndpointId,
        from_name: String,
        wants_video: bool,
        decision: oneshot::Sender<bool>,
    },
    /// Local audio (and, if `video` is `true`, video) session for `peer`
    /// is up. UI should show the in-call bar.
    ///
    /// `video`: whether video actually got negotiated — both sides wanted
    /// it *and* both had a camera `video::camera_available()` could find.
    /// Not the same as what was merely requested in the invite.
    ///
    /// `hangup_tx`: `Some` only on the callee (accept) side — see
    /// `CallProtocol::accept`'s doc for why. The caller side already
    /// registers its hangup sender with `App::set_active_call_hangup`
    /// *before* `place_call` even starts (ui::mod's outgoing-call flow),
    /// so by the time its own `Connected` event fires there's nothing
    /// left to hand over — `None` there.
    ///
    /// `video_codec`: the direction-specific outgoing codec actually used
    /// by the live capture loop. It is only meaningful when `video` is true.
    Connected {
        peer: EndpointId,
        hangup_tx: Option<oneshot::Sender<()>>,
        video: bool,
        video_codec: VideoCodec,
    },
    /// A decoded frame is ready to display — either our own camera
    /// (`from_local_camera: true`, so the UI can show a self-view tile)
    /// or the peer's (`false`). Encoded as a `data:image/jpeg;base64,...`
    /// URI, same convention `ui::mod`'s avatar cache already uses, so the
    /// UI side needs no new image-handling code path.
    VideoFrame {
        peer: EndpointId,
        from_local_camera: bool,
        data_uri: String,
    },
    /// Video was negotiated and connected, but this device's own camera
    /// capture failed to actually start — most commonly because
    /// something else already has the (only) camera open. Deliberately
    /// not `Ended`: audio and receiving the peer's video keep working,
    /// only our own outgoing video is missing. `reason` is a short,
    /// already-user-facing string (see `video::describe_camera_error`).
    VideoUnavailable { peer: EndpointId, reason: String },
    /// The call with `peer` ended, for any reason (either side hung up,
    /// the connection died, the callee declined, they never answered in
    /// time, or the caller cancelled mid-ring). UI should drop the
    /// in-call bar back to normal chat view.
    Ended { peer: EndpointId, reason: String },
}

#[derive(Clone)]
pub struct CallProtocol {
    events: UnboundedSender<CallEvent>,
    video_handoff: VideoHandoff,
}

// Manual impl for the same reason as `ContactProtocol`/`DmProtocol` in
// their own modules: `ProtocolHandler` requires `Debug`, and the channel
// sender's internals wouldn't be useful in a debug log anyway.
impl std::fmt::Debug for CallProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallProtocol").finish_non_exhaustive()
    }
}

impl CallProtocol {
    pub fn new(events: UnboundedSender<CallEvent>, video_handoff: VideoHandoff) -> Self {
        Self {
            events,
            video_handoff,
        }
    }
}

/// Fan a single hang-up click out to N independent `oneshot::Receiver`s —
/// needed once video joined audio as a second concurrent session sharing
/// one hang-up action, since a `oneshot::Receiver` can only ever be
/// consumed once and `audio::run_session`'s signature (deliberately left
/// untouched — see this module's doc) already owns one outright. Returns
/// a closure that mints a fresh receiver each call.
///
/// Backed by a `watch` channel rather than `Notify`: consumers are minted
/// at different points in a call's lifecycle (audio's is created up
/// front, video's only once the video handoff completes, which can be
/// several seconds later), so a hang-up that happens in between has to
/// still be observed by a receiver created *after* it fired. `Notify::
/// notify_waiters()` only wakes tasks already waiting at the moment it's
/// called — a later `.notified()` call would just hang forever, having
/// missed it. `watch` retains its last value, so checking `.borrow()`
/// before falling back to `.changed()` catches an already-happened
/// hang-up instead of only ever being able to see a future one.
fn fan_out_hangup(hangup_rx: oneshot::Receiver<()>) -> impl Fn() -> oneshot::Receiver<()> {
    let (watch_tx, watch_rx) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        let _ = hangup_rx.await;
        let _ = watch_tx.send(true);
    });
    move || {
        let mut rx = watch_rx.clone();
        let (tx, out_rx) = oneshot::channel();
        tokio::spawn(async move {
            if !*rx.borrow() {
                let _ = rx.changed().await;
            }
            let _ = tx.send(());
        });
        out_rx
    }
}

impl ProtocolHandler for CallProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let peer = connection.remote_id();

        let (mut send, mut recv) = match connection.accept_bi().await {
            Ok(s) => s,
            Err(_) => return Ok(()), // peer vanished before signaling even opened; nothing to surface
        };

        let bytes =
            match tokio::time::timeout(SIGNAL_TIMEOUT, recv.read_to_end(MAX_FRAME_BYTES)).await {
                Ok(Ok(b)) => b,
                _ => return Ok(()),
            };
        let (from_name, want_video, peer_encode_codecs, peer_decode_codecs) =
            match CallMsg::decode(&bytes) {
                Ok(CallMsg::Invite {
                    from_name,
                    want_video,
                    encode_codecs,
                    decode_codecs,
                    ..
                }) => (from_name, want_video, encode_codecs, decode_codecs),
                _ => return Ok(()), // not an invite as the first message — protocol violation, drop it
            };

        let (decision_tx, decision_rx) = oneshot::channel();
        if self
            .events
            .send(CallEvent::Incoming {
                from_id: peer,
                from_name,
                wants_video: want_video,
                decision: decision_tx,
            })
            .is_err()
        {
            return Ok(()); // app's gone (shutting down) — nothing left to ring
        }

        // Block here for the human to answer, but also watch whether the
        // caller's connection itself goes away in the meantime (they hit
        // Cancel mid-ring, or the network just dropped) — without this,
        // a cancelled call left this side's ring banner showing "X is
        // calling..." for the full `SIGNAL_TIMEOUT`, since nothing told
        // it the call was already over. `SIGNAL_TIMEOUT` still bounds the
        // whole wait even if the UI-side channel is somehow never
        // fulfilled, so a missed ring still can't leak this task forever.
        let accepted = tokio::select! {
            r = tokio::time::timeout(SIGNAL_TIMEOUT, decision_rx) => r.unwrap_or(Ok(false)).unwrap_or(false),
            _ = connection.closed() => {
                let _ = self.events.send(CallEvent::Ended { peer, reason: "cancelled".to_string() });
                return Ok(());
            }
        };

        // Video only actually happens if both ends want it *and* this
        // device has a usable camera — `spawn_blocking` since enumerating
        // capture devices is a short blocking OS call (see
        // `video::camera_available`'s doc), not something to run inline
        // on the async runtime.
        let my_decode_codecs = available_hw_decode_codecs();
        let my_encode_codecs = available_hw_encode_codecs();
        let send_video_codec = negotiate_send_codec(&my_encode_codecs, &peer_decode_codecs);
        let receive_video_codec = negotiate_send_codec(&peer_encode_codecs, &my_decode_codecs);
        let video_accepted = accepted
            && want_video
            && send_video_codec.is_some()
            && receive_video_codec.is_some()
            && tokio::task::spawn_blocking(video::camera_available)
                .await
                .unwrap_or(false);

        let Ok(answer) = (CallMsg::Answer {
            accepted,
            video_accepted,
            encode_codecs: my_encode_codecs,
            decode_codecs: my_decode_codecs,
        })
        .encode() else {
            return Ok(()); // shouldn't happen (fixed small shape), but nothing to do if it does
        };
        if send.write_all(&answer).await.is_err() || send.finish().is_err() {
            return Ok(());
        }
        // See `net::contacts::send_one`'s doc for why this matters: without
        // it, dropping the stream (implicitly, once this fn moves past it)
        // can race the still-in-flight answer bytes over a relay hop.
        let _ = tokio::time::timeout(SIGNAL_TIMEOUT, send.stopped()).await;

        if !accepted {
            return Ok(());
        }

        let (hangup_tx, hangup_rx) = oneshot::channel();
        let _ = self.events.send(CallEvent::Connected {
            peer,
            hangup_tx: Some(hangup_tx),
            video: video_accepted,
            video_codec: send_video_codec.unwrap_or(VideoCodec::Av1),
        });
        let next_hangup = fan_out_hangup(hangup_rx);

        let audio_fut = audio::run_session(&connection, next_hangup());
        if video_accepted {
            let send_video_codec =
                send_video_codec.expect("video acceptance requires an outgoing codec");
            let receive_video_codec =
                receive_video_codec.expect("video acceptance requires an incoming codec");
            // Wait for the caller's separate video connection to land in
            // our shared hand-off slot (see this module's doc), then run
            // it concurrently with audio until either ends.
            let video_events = self.events.clone();
            let video_handoff = self.video_handoff.clone();
            let (video_conn_tx, video_conn_rx) = oneshot::channel();
            *video_handoff.lock().unwrap() = Some(video_conn_tx);
            let video_hangup = next_hangup();
            let video_fut = async move {
                match tokio::time::timeout(VIDEO_HANDOFF_TIMEOUT, video_conn_rx).await {
                    Ok(Ok(video_connection)) => {
                        if let Err(e) = video::run_video_session(
                            &video_connection,
                            peer,
                            send_video_codec,
                            receive_video_codec,
                            video_events,
                            video_hangup,
                        )
                        .await
                        {
                            tracing::debug!(%peer, error = %e, "video session ended with error");
                        }
                    }
                    _ => {
                        tracing::debug!(%peer, "caller's video connection never arrived — continuing audio-only");
                    }
                }
            };
            let (audio_result, ()) = tokio::join!(audio_fut, video_fut);
            let reason = match audio_result {
                Ok(()) => "call ended".to_string(),
                Err(e) => format!("call ended: {e}"),
            };
            let _ = self.events.send(CallEvent::Ended { peer, reason });
        } else {
            let reason = match audio_fut.await {
                Ok(()) => "call ended".to_string(),
                Err(e) => format!("call ended: {e}"),
            };
            let _ = self.events.send(CallEvent::Ended { peer, reason });
        }
        Ok(())
    }
}

/// Place a call to `peer` and, if they accept, run the audio (and,
/// if negotiated, video) session until either side hangs up or the
/// connection drops. Blocks for the whole duration of the call — the
/// caller (`ui`) should `spawn` this rather than awaiting it inline.
/// `hangup_rx` fires when the local user clicks "hang up"/"cancel" —
/// including *during* ringing, before the callee has even answered, which
/// this function now actually honors (previously it was accepted as a
/// parameter but not raced against the ringing phase at all, so
/// cancelling mid-ring did nothing until the call happened to connect).
pub async fn place_call(
    endpoint: &Endpoint,
    peer: EndpointId,
    my_id: EndpointId,
    my_name: &str,
    want_video: bool,
    events: UnboundedSender<CallEvent>,
    hangup_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let next_hangup = fan_out_hangup(hangup_rx);
    let mut cancelled = next_hangup();

    let ringing = async {
        let connection = tokio::time::timeout(SIGNAL_TIMEOUT, endpoint.connect(peer, ALPN))
            .await
            .context("call connect timed out")??;
        let (mut send, mut recv) = connection.open_bi().await?;

        let my_decode_codecs = available_hw_decode_codecs();
        let my_encode_codecs = available_hw_encode_codecs();
        let invite = CallMsg::Invite {
            from_id: *my_id.as_bytes(),
            from_name: my_name.to_string(),
            want_video,
            encode_codecs: my_encode_codecs.clone(),
            decode_codecs: my_decode_codecs,
        }
        .encode()?;
        send.write_all(&invite).await?;
        send.finish()?;
        let _ = tokio::time::timeout(SIGNAL_TIMEOUT, send.stopped()).await;

        let bytes = tokio::time::timeout(SIGNAL_TIMEOUT, recv.read_to_end(MAX_FRAME_BYTES))
            .await
            .context("no answer — they may be unreachable or just didn't pick up in time")??;
        let (accepted, peer_video_accepted, send_video_codec, receive_video_codec) =
            match CallMsg::decode(&bytes) {
                Ok(CallMsg::Answer {
                    accepted,
                    video_accepted,
                    encode_codecs,
                    decode_codecs,
                }) => (
                    accepted,
                    video_accepted,
                    negotiate_send_codec(&my_encode_codecs, &decode_codecs),
                    negotiate_send_codec(&encode_codecs, &available_hw_decode_codecs()),
                ),
                _ => bail!("unexpected reply from peer"),
            };
        let video_accepted =
            peer_video_accepted && send_video_codec.is_some() && receive_video_codec.is_some();
        Ok::<_, anyhow::Error>((
            connection,
            accepted,
            video_accepted,
            send_video_codec,
            receive_video_codec,
        ))
    };

    let (connection, accepted, video_accepted, send_video_codec, receive_video_codec) = tokio::select! {
        biased;
        _ = &mut cancelled => {
            let _ = events.send(CallEvent::Ended { peer, reason: "cancelled".to_string() });
            return Ok(());
        }
        r = ringing => r?,
    };

    if !accepted {
        let _ = events.send(CallEvent::Ended {
            peer,
            reason: "declined".to_string(),
        });
        return Ok(());
    }

    let _ = events.send(CallEvent::Connected {
        peer,
        hangup_tx: None,
        video: video_accepted,
        video_codec: send_video_codec.unwrap_or(VideoCodec::Av1),
    });

    let audio_fut = audio::run_session(&connection, next_hangup());
    if video_accepted {
        let send_video_codec =
            send_video_codec.expect("video acceptance requires an outgoing codec");
        let receive_video_codec =
            receive_video_codec.expect("video acceptance requires an incoming codec");
        // We're the caller, so we're also the one who dials the video
        // connection directly — no hand-off slot needed on this side,
        // unlike the callee (see this module's doc).
        let video_events = events.clone();
        let video_hangup = next_hangup();
        let video_fut = async move {
            match tokio::time::timeout(VIDEO_HANDOFF_TIMEOUT, endpoint.connect(peer, video::ALPN))
                .await
            {
                Ok(Ok(video_connection)) => {
                    if let Err(e) = video::run_video_session(
                        &video_connection,
                        peer,
                        send_video_codec,
                        receive_video_codec,
                        video_events,
                        video_hangup,
                    )
                    .await
                    {
                        tracing::debug!(%peer, error = %e, "video session ended with error");
                    }
                }
                _ => {
                    tracing::debug!(%peer, "couldn't open video connection — continuing audio-only")
                }
            }
        };
        let (audio_result, ()) = tokio::join!(audio_fut, video_fut);
        let reason = match audio_result {
            Ok(()) => "call ended".to_string(),
            Err(e) => {
                warn!(%peer, error = %e, "call session ended with error");
                format!("call ended: {e}")
            }
        };
        let _ = events.send(CallEvent::Ended { peer, reason });
    } else {
        let reason = match audio_fut.await {
            Ok(()) => "call ended".to_string(),
            Err(e) => {
                warn!(%peer, error = %e, "call session ended with error");
                format!("call ended: {e}")
            }
        };
        let _ = events.send(CallEvent::Ended { peer, reason });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{negotiate_send_codec, VideoCodec};

    #[test]
    fn codec_negotiation_prefers_efficiency_order() {
        let all = [VideoCodec::Av1, VideoCodec::H264, VideoCodec::H265];
        assert_eq!(negotiate_send_codec(&all, &all), Some(VideoCodec::H265));
        assert_eq!(
            negotiate_send_codec(&[VideoCodec::Av1, VideoCodec::H264], &all),
            Some(VideoCodec::H264)
        );
        assert_eq!(
            negotiate_send_codec(&[VideoCodec::Av1], &all),
            Some(VideoCodec::Av1)
        );
    }

    #[test]
    fn codec_negotiation_never_invents_a_fallback() {
        assert_eq!(
            negotiate_send_codec(&[VideoCodec::Av1], &[VideoCodec::H264]),
            None
        );
        assert_eq!(negotiate_send_codec(&[], &[VideoCodec::H265]), None);
    }
}
