//! Ties `siar_domain`'s call-signaling state machine and quality-ladder
//! decision logic to this crate's media pipelines. This is the
//! `CallSession` architecture doc §1 refers to as depending on
//! `VideoEncoder`/`VideoDecoder`/`VideoCapabilities` and never on
//! `siar-media-av1`/`siar-media-android` by name — the pipelines it
//! holds are already-erased `Box<dyn ...>`/channel handles by the time
//! they get here (see `pipeline.rs` and `crate::android`), so this file
//! itself has no `#[cfg(target_os = "android")]` anywhere in it.

use siar_domain::{adapt_quality, CallControlEvent, CallState, MediaQuality, NetworkConditions};

use crate::pipeline::{
    AudioDecodePipeline, AudioEncodePipeline, VideoDecodePipeline, VideoEncodePipeline,
};
use crate::shutdown::CallShutdown;

/// One call's media-plane state: the signaling `CallState` plus
/// whichever pipelines got negotiated for it. `video` is `None` for an
/// audio-only call (`siar_media_core::NegotiatedCall::is_audio_only`) —
/// deliberately not a `Box<dyn VideoEncoder>` that's just never called,
/// since "no video negotiated" and "video negotiated but broken" should
/// stay distinguishable states, not collapse into the same one.
pub struct CallSession {
    state: CallState,
    quality: MediaQuality,
    shutdown: CallShutdown,
    audio_encode: AudioEncodePipeline,
    audio_decode: AudioDecodePipeline,
    video_encode: Option<VideoEncodePipeline>,
    video_decode: Option<VideoDecodePipeline>,
}

impl CallSession {
    /// `shutdown` should be the same `CallShutdown` whose `subscribe()`d
    /// receivers were handed to each `spawn_*_pipeline`/
    /// `spawn_android_hardware_*_pipeline` call that produced the
    /// pipelines passed in here — this is what `apply_control_event`
    /// signals once the call reaches `CallState::Ended`, so constructing
    /// a session with a *different* `CallShutdown` than the one its
    /// pipelines were subscribed to would silently leave them running
    /// forever after hangup.
    pub fn new(
        starting_quality: MediaQuality,
        shutdown: CallShutdown,
        audio_encode: AudioEncodePipeline,
        audio_decode: AudioDecodePipeline,
        video_encode: Option<VideoEncodePipeline>,
        video_decode: Option<VideoDecodePipeline>,
    ) -> Self {
        Self {
            state: CallState::Idle,
            quality: starting_quality,
            shutdown,
            audio_encode,
            audio_decode,
            video_encode,
            video_decode,
        }
    }

    pub fn state(&self) -> CallState {
        self.state
    }

    pub fn quality(&self) -> MediaQuality {
        self.quality
    }

    /// Applies one signaling event via `CallState::apply`'s own
    /// transition table. An illegal transition (`None`) is silently
    /// ignored here, matching `siar_domain::call`'s own documented
    /// contract for that case ("not a legal transition — caller ignores
    /// the event") rather than this layer inventing its own error type
    /// for something the state machine already defined as a no-op.
    ///
    /// Reaching `CallState::Ended` signals `shutdown` — this is the one
    /// place in this crate that fires it, so every pipeline's
    /// background tasks stop here rather than leaking past hangup.
    pub fn apply_control_event(&mut self, event: CallControlEvent) {
        if let Some(next) = self.state.apply(event) {
            self.state = next;
            if self.state == CallState::Ended {
                self.shutdown.signal();
            }
        }
    }

    /// Runs one step of bitrate adaptation against a sampled
    /// `NetworkConditions` reading. Sampling real RTT/loss/jitter off a
    /// live connection is `siar-transport`'s job (see
    /// `crate::transport::MediaTransport`'s doc comment) — this method
    /// takes the sample as a parameter rather than pulling it itself,
    /// so the pure decision logic stays testable the same way
    /// `siar_domain::call::adapt_quality`'s own tests already are.
    pub fn adapt_quality(&mut self, conditions: NetworkConditions) {
        self.quality = adapt_quality(self.quality, conditions);
    }

    pub fn audio_encode(&mut self) -> &mut AudioEncodePipeline {
        &mut self.audio_encode
    }

    pub fn audio_decode(&mut self) -> &mut AudioDecodePipeline {
        &mut self.audio_decode
    }

    pub fn video_encode(&mut self) -> Option<&mut VideoEncodePipeline> {
        self.video_encode.as_mut()
    }

    pub fn video_decode(&mut self) -> Option<&mut VideoDecodePipeline> {
        self.video_decode.as_mut()
    }

    pub fn is_video_call(&self) -> bool {
        self.video_encode.is_some() || self.video_decode.is_some()
    }
}
