//! §6 "Clean Shutdown Marker".

use serde::{Deserialize, Serialize};

/// §6's own "or equivalent generation marker" — a monotonic counter
/// identifying one runtime lifetime, so a marker left behind by an
/// old process can never be confused with the current one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RuntimeGeneration(pub u64);

/// §6's `runtime_started` / `runtime_clean_shutdown` pair for one
/// generation. `clean_shutdown_at_millis` is `None` until
/// [`ShutdownMarkerStore::record_clean_shutdown`] is called — a marker
/// still `None` when the *next* generation starts is exactly the "last
/// start had no clean close" signal §6 describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShutdownMarker {
    pub generation: RuntimeGeneration,
    pub started_at_millis: u64,
    pub clean_shutdown_at_millis: Option<u64>,
}

/// §6 doesn't specify a storage mechanism (that's whatever durable
/// store the caller already has) — this trait is the seam, so a real
/// caller backs it with actual persistence while this crate's own
/// [`InMemoryShutdownMarkerStore`] stands in for tests, the same
/// pattern `siar_event_log`/`siar_dtn_bundle`'s own `*Store` traits
/// already use in this workspace.
pub trait ShutdownMarkerStore {
    /// Records the start of a new runtime generation — always a new,
    /// not-yet-cleanly-shut-down marker.
    fn record_start(&mut self, now_millis: u64) -> RuntimeGeneration;

    /// Marks `generation`'s shutdown as clean. Never called for a
    /// generation that crashed — that absence is the whole mechanism.
    fn record_clean_shutdown(&mut self, generation: RuntimeGeneration, now_millis: u64);

    /// The most recent marker written, if any — `None` only on a
    /// genuinely first-ever run.
    fn last_marker(&self) -> Option<ShutdownMarker>;
}

/// §6: "On startup: last start had no clean close → unclean startup
/// path." A first-ever run (no prior marker at all) is [`StartupKind::Clean`]
/// — there is no prior crash to react to, so nothing about the
/// absence of history should trigger extra recovery work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StartupKind {
    Clean,
    Unclean,
}

/// §6's own explicit caution: "Do not depend on this marker for
/// correctness, only for deciding how much recovery work to run." This
/// function — and every caller of it — must only ever feed into a
/// *scope* decision (e.g. [`RecoveryScope`]), never into skipping a
/// step that [`crate::idempotent_steps`]'s pipeline would otherwise
/// consider mandatory. The marker is a hint for doing less work
/// faster on the common clean-restart path, not a correctness
/// mechanism in its own right — recovery steps must stay safe to run
/// even if this classification were wrong or the marker were lost
/// entirely.
pub fn classify_startup(marker: Option<ShutdownMarker>) -> StartupKind {
    match marker {
        None => StartupKind::Clean,
        Some(m) if m.clean_shutdown_at_millis.is_some() => StartupKind::Clean,
        Some(_) => StartupKind::Unclean,
    }
}

/// The one thing [`StartupKind`] is actually allowed to influence
/// (§6's "how much recovery work to run"): whether to run only the
/// mandatory idempotent steps, or also perform extra proactive
/// integrity verification a clean restart doesn't need. Both paths
/// still run every mandatory step — this only adds work, never
/// removes it, keeping faith with §6's "not for correctness" rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecoveryScope {
    MandatoryStepsOnly,
    MandatoryStepsPlusIntegrityCheck,
}

pub fn recommended_scope(startup: StartupKind) -> RecoveryScope {
    match startup {
        StartupKind::Clean => RecoveryScope::MandatoryStepsOnly,
        StartupKind::Unclean => RecoveryScope::MandatoryStepsPlusIntegrityCheck,
    }
}

/// A test/example-friendly `ShutdownMarkerStore` — an in-memory
/// history, not a real durable store (that's the caller's job; see
/// this module's own doc comment on [`ShutdownMarkerStore`]).
#[derive(Debug, Clone, Default)]
pub struct InMemoryShutdownMarkerStore {
    history: Vec<ShutdownMarker>,
}

impl ShutdownMarkerStore for InMemoryShutdownMarkerStore {
    fn record_start(&mut self, now_millis: u64) -> RuntimeGeneration {
        let generation = RuntimeGeneration(self.history.len() as u64 + 1);
        self.history.push(ShutdownMarker {
            generation,
            started_at_millis: now_millis,
            clean_shutdown_at_millis: None,
        });
        generation
    }

    fn record_clean_shutdown(&mut self, generation: RuntimeGeneration, now_millis: u64) {
        if let Some(marker) = self.history.iter_mut().find(|m| m.generation == generation) {
            marker.clean_shutdown_at_millis = Some(now_millis);
        }
    }

    fn last_marker(&self) -> Option<ShutdownMarker> {
        self.history.last().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_ever_run_is_classified_clean() {
        assert_eq!(classify_startup(None), StartupKind::Clean);
    }

    #[test]
    fn a_marker_with_recorded_clean_shutdown_is_clean() {
        let mut store = InMemoryShutdownMarkerStore::default();
        let gen = store.record_start(0);
        store.record_clean_shutdown(gen, 100);
        assert_eq!(classify_startup(store.last_marker()), StartupKind::Clean);
    }

    #[test]
    fn a_marker_missing_clean_shutdown_is_unclean() {
        // §6's exact scenario: the process started but never got to
        // record a clean close (crash, kill, power loss).
        let mut store = InMemoryShutdownMarkerStore::default();
        store.record_start(0);
        assert_eq!(classify_startup(store.last_marker()), StartupKind::Unclean);
    }

    #[test]
    fn each_start_gets_a_distinct_generation() {
        let mut store = InMemoryShutdownMarkerStore::default();
        let a = store.record_start(0);
        let b = store.record_start(100);
        assert_ne!(a, b);
    }

    #[test]
    fn recommended_scope_only_adds_work_never_removes_mandatory_steps() {
        // §6's own caution, checked at the type level: both variants
        // of RecoveryScope include the mandatory steps.
        assert_eq!(
            recommended_scope(StartupKind::Clean),
            RecoveryScope::MandatoryStepsOnly
        );
        assert_eq!(
            recommended_scope(StartupKind::Unclean),
            RecoveryScope::MandatoryStepsPlusIntegrityCheck
        );
    }

    #[test]
    fn clean_shutdown_on_a_stale_generation_does_not_affect_the_current_one() {
        let mut store = InMemoryShutdownMarkerStore::default();
        let gen1 = store.record_start(0);
        let _gen2 = store.record_start(50);
        // Recording a clean shutdown for the *old* generation must not
        // retroactively mark the current (still-running) one clean.
        store.record_clean_shutdown(gen1, 100);
        assert_eq!(classify_startup(store.last_marker()), StartupKind::Unclean);
    }
}
