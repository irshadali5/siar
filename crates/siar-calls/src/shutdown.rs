//! A shutdown signal shared by every background task a `CallSession`
//! owns (`pipeline.rs`'s `spawn_blocking` loops, `android.rs`'s two
//! `tokio::spawn` tasks per direction). One `CallShutdown` per call;
//! every pipeline gets its own `subscribe()`d receiver at construction
//! time, and `CallSession::apply_control_event` fires it the moment the
//! signaling state machine reaches `CallState::Ended` — that's the hook
//! this was flagged as missing for.
//!
//! Wraps `tokio::sync::watch` rather than `tokio_util::CancellationToken`
//! specifically to avoid adding a new dependency for this — `watch`'s
//! `changed()` is already exactly the "wait for one flip from false to
//! true" primitive this needs, and it's already reachable through
//! `tokio`'s existing `sync` feature.

use std::sync::Arc;

use tokio::sync::watch;

#[derive(Clone)]
pub struct CallShutdown {
    tx: Arc<watch::Sender<bool>>,
}

impl CallShutdown {
    /// The returned `watch::Receiver` is the initial one `watch::channel`
    /// hands back — most callers can drop it immediately after taking
    /// at least one `subscribe()`'d clone for a pipeline, since `send`
    /// only fails once *every* receiver (including subscribed clones)
    /// has been dropped, not specifically this original one.
    pub fn new() -> (Self, watch::Receiver<bool>) {
        let (tx, rx) = watch::channel(false);
        (Self { tx: Arc::new(tx) }, rx)
    }

    /// A fresh receiver for one background task to select against. Call
    /// once per `spawn_*_pipeline` call before wiring it into a
    /// `CallSession` — a receiver made *after* `signal()` already fired
    /// still observes the current (already-true) value via `borrow()`,
    /// but `changed()` alone won't resolve for it until a *further*
    /// change, so pipelines should subscribe before the call can
    /// possibly end, not lazily.
    pub fn subscribe(&self) -> watch::Receiver<bool> {
        self.tx.subscribe()
    }

    /// Idempotent — signaling an already-signaled shutdown is a no-op
    /// (`watch::Sender::send` just republishes the same `true` value).
    /// Ignores the "no receivers left" error: if every pipeline task
    /// already exited on its own, there's nothing left to signal, which
    /// isn't a failure this caller needs to react to.
    pub fn signal(&self) {
        let _ = self.tx.send(true);
    }
}
