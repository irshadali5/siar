//! Call control plane (plan.md §48, §51).
//!
//! Scope note: this is the *signaling* state machine and the bitrate
//! *decision* logic only — both are pure data transformations, fully
//! testable without a microphone, camera, or codec library. Actual
//! audio/video capture, Opus/AV1 encoding, and realtime media transport
//! (plan.md §49–50) need real device access and codec bindings this
//! sandbox cannot exercise at any level (not even "does this function
//! signature exist" — there's no camera to query). Writing that blind
//! would be pure guessing, so it's left for a session with real hardware
//! and a compiler.

use crate::{AccountId, DeviceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallState {
    Idle,
    Offering,
    Ringing,
    Connecting,
    Active,
    Ended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallControlEvent {
    Offer,
    Ring,
    Accept,
    Reject,
    Hangup,
    ConnectionEstablished,
    ConnectionLost,
}

impl CallState {
    /// plan.md §122's illegal-states-unrepresentable rule applied to call
    /// signaling: an explicit transition table rather than scattered
    /// `if state == X` checks means "can I hang up from Idle" has one
    /// place to look, not N.
    pub fn apply(self, event: CallControlEvent) -> Option<CallState> {
        use CallControlEvent::*;
        use CallState::*;
        match (self, event) {
            (Idle, Offer) => Some(Offering),
            (Offering, Ring) => Some(Ringing),
            (Ringing, Accept) => Some(Connecting),
            (Ringing, Reject) => Some(Ended),
            (Connecting, ConnectionEstablished) => Some(Active),
            (Connecting, ConnectionLost) => Some(Ended),
            (Active, ConnectionLost) => Some(Ended),
            (Offering | Ringing | Connecting | Active, Hangup) => Some(Ended),
            _ => None, // not a legal transition — caller ignores the event
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallParticipant {
    pub account: AccountId,
    pub device: DeviceId,
}

/// plan.md §51: network conditions the adaptation logic reacts to.
/// Deliberately plain numbers, not a live measurement — whatever
/// actually samples RTT/loss/jitter off a real connection is
/// `siar-transport`'s job (and needs a real network to test against);
/// this type is just the input shape the pure decision function below
/// takes.
#[derive(Debug, Clone, Copy)]
pub struct NetworkConditions {
    pub round_trip_millis: u32,
    pub packet_loss_fraction: f32, // 0.0..=1.0
    pub jitter_millis: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaQuality {
    pub video_bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
    pub resolution_height: u32,
}

const QUALITY_LADDER: &[MediaQuality] = &[
    MediaQuality { video_bitrate_kbps: 150, audio_bitrate_kbps: 16, resolution_height: 180 },
    MediaQuality { video_bitrate_kbps: 400, audio_bitrate_kbps: 24, resolution_height: 360 },
    MediaQuality { video_bitrate_kbps: 1000, audio_bitrate_kbps: 32, resolution_height: 480 },
    MediaQuality { video_bitrate_kbps: 2500, audio_bitrate_kbps: 32, resolution_height: 720 },
];

/// plan.md §51: "do not blindly stream at fixed bitrate" — steps down
/// the quality ladder under loss/high RTT, steps up only when
/// conditions are comfortably good (asymmetric on purpose: ramping
/// quality back up too eagerly just causes it to immediately step back
/// down on the next bad sample, a visible flicker rather than a stable
/// picture).
pub fn adapt_quality(current: MediaQuality, conditions: NetworkConditions) -> MediaQuality {
    let current_index = QUALITY_LADDER
        .iter()
        .position(|q| *q == current)
        .unwrap_or(QUALITY_LADDER.len() - 1);

    let should_step_down = conditions.packet_loss_fraction > 0.05
        || conditions.round_trip_millis > 300
        || conditions.jitter_millis > 100;
    let should_step_up = conditions.packet_loss_fraction < 0.01
        && conditions.round_trip_millis < 150
        && conditions.jitter_millis < 30;

    let next_index = if should_step_down {
        current_index.saturating_sub(1)
    } else if should_step_up {
        (current_index + 1).min(QUALITY_LADDER.len() - 1)
    } else {
        current_index
    };

    QUALITY_LADDER[next_index]
}

#[cfg(test)]
mod tests {
    use super::*;
    use CallControlEvent::*;
    use CallState::*;

    #[test]
    fn happy_path_offer_to_active() {
        let mut state = Idle;
        for event in [Offer, Ring, Accept, ConnectionEstablished] {
            state = state.apply(event).expect("legal transition");
        }
        assert_eq!(state, Active);
    }

    #[test]
    fn rejecting_a_ring_ends_the_call() {
        let state = Idle.apply(Offer).unwrap().apply(Ring).unwrap();
        assert_eq!(state.apply(Reject), Some(Ended));
    }

    #[test]
    fn hangup_works_from_any_in_progress_state() {
        for state in [Offering, Ringing, Connecting, Active] {
            assert_eq!(state.apply(Hangup), Some(Ended));
        }
    }

    #[test]
    fn illegal_transitions_are_rejected() {
        assert_eq!(Idle.apply(Accept), None);
        assert_eq!(Ended.apply(Offer), None);
        assert_eq!(Idle.apply(Hangup), None);
    }

    #[test]
    fn quality_steps_down_under_high_packet_loss() {
        let good = QUALITY_LADDER[3];
        let bad_conditions = NetworkConditions { round_trip_millis: 50, packet_loss_fraction: 0.1, jitter_millis: 10 };
        let next = adapt_quality(good, bad_conditions);
        assert!(next.video_bitrate_kbps < good.video_bitrate_kbps);
    }

    #[test]
    fn quality_steps_up_under_good_conditions() {
        let low = QUALITY_LADDER[0];
        let good_conditions = NetworkConditions { round_trip_millis: 40, packet_loss_fraction: 0.0, jitter_millis: 5 };
        let next = adapt_quality(low, good_conditions);
        assert!(next.video_bitrate_kbps > low.video_bitrate_kbps);
    }

    #[test]
    fn quality_does_not_exceed_the_top_of_the_ladder() {
        let top = QUALITY_LADDER[QUALITY_LADDER.len() - 1];
        let great_conditions = NetworkConditions { round_trip_millis: 10, packet_loss_fraction: 0.0, jitter_millis: 1 };
        assert_eq!(adapt_quality(top, great_conditions), top);
    }

    #[test]
    fn quality_does_not_go_below_the_bottom_of_the_ladder() {
        let bottom = QUALITY_LADDER[0];
        let terrible_conditions = NetworkConditions { round_trip_millis: 900, packet_loss_fraction: 0.5, jitter_millis: 500 };
        assert_eq!(adapt_quality(bottom, terrible_conditions), bottom);
    }

    #[test]
    fn mediocre_conditions_hold_steady_rather_than_flicker() {
        let mid = QUALITY_LADDER[1];
        let mediocre = NetworkConditions { round_trip_millis: 200, packet_loss_fraction: 0.02, jitter_millis: 50 };
        assert_eq!(adapt_quality(mid, mediocre), mid);
    }
}
