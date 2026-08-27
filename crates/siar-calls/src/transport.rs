//! The network plug point. `siar-domain::call.rs`'s own doc comment
//! already drew this boundary — "realtime audio/video capture, codec
//! encoding, and realtime media transport... need real device access
//! and codec bindings this sandbox cannot exercise at any level" — and
//! it still applies: wiring real QUIC send/receive against `siar-
//! transport`'s live `SiarEndpoint` needs a real network to validate
//! framing, pacing under real loss/jitter, and stream-vs-datagram
//! choice against. This trait is the shape [`crate::session::CallSession`]
//! expects that wiring to have, not the wiring itself.

use std::future::Future;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaChannel {
    Video,
    Audio,
}

/// One call's bidirectional media transport, from `siar-calls`'s point
/// of view: encoded bytes in, encoded bytes out, per channel. Framing
/// (how a `siar-transport` implementation turns one `send` call into
/// one or more QUIC stream writes or datagrams, and reassembles them on
/// receive) is entirely the implementor's concern — this trait doesn't
/// assume stream vs. datagram, since that choice itself is one of the
/// things that wants real-network measurement, not a guess here.
pub trait MediaTransport: Send + Sync + 'static {
    /// Sends one already-encoded frame's bytes on `channel`. Does not
    /// itself retry or fragment — an implementation over unreliable
    /// datagrams might drop this outright under loss, and one over a
    /// reliable stream might block under backpressure; either is a
    /// legitimate implementation, and `session.rs`'s pipelines already
    /// treat "send didn't happen this cycle" as an expected outcome
    /// (see `pipeline.rs`'s backpressure notes), not an error to
    /// propagate loudly.
    fn send(
        &self,
        channel: MediaChannel,
        data: Vec<u8>,
    ) -> impl Future<Output = Result<(), TransportError>> + Send;

    /// Receives the next frame's bytes for `channel`, in whatever order
    /// the underlying transport delivered them — reordering is
    /// `crate::jitter::JitterBuffer`'s job on the receiving end, not
    /// this trait's.
    fn recv(
        &self,
        channel: MediaChannel,
    ) -> impl Future<Output = Result<Vec<u8>, TransportError>> + Send;
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TransportError {
    #[error("media transport connection closed")]
    Closed,
    #[error("media transport error: {0}")]
    Other(String),
}
