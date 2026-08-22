//! §12 "Path Metrics", §13 "Metrics Confidence".

use serde::{Deserialize, Serialize};

/// §13: "Routing must not treat a 20-minute-old bandwidth estimate as
/// current truth." A bare, unqualified confidence label — this crate
/// doesn't invent a numeric decay curve (the spec doesn't specify one
/// either); [`crate::scoring`] uses [`Confidence::Stale`] to discount a
/// metric's weight rather than trusting it at face value, and it's the
/// caller's job to decide when a `Measured` value has aged into
/// `Stale` (this crate has no wall-clock/timer dependency to do that
/// itself — see this crate's own top doc comment on scope).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    Measured,
    Estimated,
    Stale,
}

/// §13's example type, generic over the metric it wraps.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MeasuredValue<T> {
    pub value: T,
    pub observed_at_millis: u64,
    pub confidence: Confidence,
}

/// A ratio in `[0.0, 1.0]` — used for packet loss (§12) and retry
/// jitter (§38). A plain `f64` newtype rather than a raw float in every
/// call site, so a caller can't accidentally pass a percentage (`0-100`)
/// where a fraction (`0.0-1.0`) is expected — a real, easy-to-make bug
/// class for exactly this kind of field.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ratio(f64);

impl Ratio {
    /// Clamps to `[0.0, 1.0]` rather than rejecting out-of-range input
    /// — a caller-side measurement bug (e.g. loss computed as `> 1.0`
    /// due to a counter race) shouldn't be able to panic or silently
    /// corrupt a routing decision; clamping is the same "measurement
    /// noise shouldn't crash policy" posture §13 already takes toward
    /// stale/uncertain metrics generally.
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

/// Bits per second. `u64`, not `f64` — a bitrate is a real measured
/// quantity that should round-trip exactly through postcard (Part 01
/// §92 "Postcard Rules": fixed-width integers over floats where a
/// value must survive a wire round trip unchanged); nothing about
/// bandwidth needs fractional precision at the bits/sec scale this
/// crate deals in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Bitrate(pub u64);

/// §12's `stability`/`energy_cost`/`monetary_cost` — three independent,
/// small ordered scales rather than one overloaded number, so a scorer
/// (§24) can weight each separately per policy profile (§27) instead of
/// each transport having to pre-combine them into a single opaque
/// score of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StabilityScore {
    VeryUnstable,
    Unstable,
    Moderate,
    Stable,
    VeryStable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EnergyCost {
    Free,
    Low,
    Moderate,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NetworkCost {
    Free,
    Low,
    Moderate,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SignalQuality {
    VeryPoor,
    Poor,
    Fair,
    Good,
    Excellent,
}

/// §12.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PathMetrics {
    pub rtt_millis: Option<u32>,
    pub estimated_bandwidth: Option<Bitrate>,
    pub packet_loss: Option<Ratio>,
    pub jitter_millis: Option<u32>,
    pub stability: StabilityScore,
    pub energy_cost: EnergyCost,
    pub monetary_cost: NetworkCost,
    pub signal_quality: Option<SignalQuality>,
    pub last_success_millis: Option<u64>,
}

impl PathMetrics {
    /// Every optional field absent, every scale at its most
    /// conservative (least-known) value — §12's own "missing metrics
    /// must be represented explicitly" starting point for a
    /// just-discovered candidate this crate has no history for yet.
    pub fn unknown() -> Self {
        Self {
            rtt_millis: None,
            estimated_bandwidth: None,
            packet_loss: None,
            jitter_millis: None,
            stability: StabilityScore::Moderate,
            energy_cost: EnergyCost::Moderate,
            monetary_cost: NetworkCost::Moderate,
            signal_quality: None,
            last_success_millis: None,
        }
    }
}
