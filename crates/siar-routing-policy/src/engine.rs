//! §119 "Routing Engine API", and §120 "Transport Manager API"'s own
//! scope boundary.
//!
//! §119's trait is transcribed with native `async fn`-in-trait syntax
//! (stable since Rust 1.75; this workspace pins 1.91.0, see
//! `rust-toolchain.toml`) rather than the `#[async_trait]` macro —
//! nothing else in this crate depends on an async runtime (no
//! `tokio`/`async-trait` in `Cargo.toml`), and native syntax needs
//! neither.
//!
//! §120 "Transport Manager API" is the one section in this round with
//! no code here at all. Its own trait names three types —
//! `ResolvedDestination`, `TransportSession`, `TransportError` — none
//! of which exist in this crate, and its own text is explicit about
//! why: "Routing does not own low-level sockets." This crate has no
//! `siar-transport`/`iroh` dependency at all (see this crate's top
//! doc comment on scope) — inventing local stand-ins for those three
//! types just to satisfy the trait signature would mean this crate
//! defining an interface to a resource it deliberately never touches,
//! which is worse than not defining it: a caller reading
//! `TransportManager` here would reasonably expect this crate to
//! actually call `acquire()` somewhere, and it never does. The right
//! place for that trait is whichever crate *does* own sockets
//! (`siar-transport`) — named here as a real, deliberate gap, not
//! quietly skipped.
//!
//! [`RouteRequest`] deliberately does **not** carry
//! [`crate::decision::PolicyLayers`], `account_id`, a
//! `TrustedAccountStore`, a `RoutingPolicy`, a `PathScorer`, or a
//! `DiscoveryBudget` — every one of those is exactly what §118's own
//! "each `CommunicationRuntime` owns its routing engine" describes as
//! belonging to the engine *implementation* (constructed once, held
//! for the runtime's lifetime), not to a per-call request. `plan`
//! takes `&self`, not `&mut self` (per §119's own signature) — an
//! implementation holding a [`crate::discovery::DiscoveryBudget`]
//! (which [`crate::decision::decide_route`] needs `&mut` access to)
//! therefore needs interior mutability of its own; see this module's
//! own test for a worked example using `std::sync::Mutex`.
//!
//! [`RouteDecision`] is a type alias for
//! [`crate::decision::RouteDecisionResult`], not a new type — that
//! type already has exactly the shape §114/§119 both want. `plan`'s
//! `Result<RouteDecision, RoutingError>` outer `Result` is this
//! trait's error channel for a request the engine cannot even
//! *attempt* — [`crate::decision::decide_route`] itself never returns
//! an error in that sense (every input, however unroutable, produces
//! one of `RouteDecisionResult`'s four variants), so a plain
//! composition of it, like this module's own reference
//! implementation, always returns `Ok`. A real integration might
//! still need the `Err` side for something this crate doesn't model
//! at all — a malformed request at an actual RPC boundary, say — which
//! is exactly the kind of thing a trait's error channel should stay
//! available for even when today's only implementation never uses it.

use crate::candidate::PathCandidate;
use crate::descriptor::{OperationDescriptor, OperationId};
use crate::error::RoutingError;
use crate::failure::RouteFailureClass;
use crate::platform::DeviceState;
use crate::privacy::PrivacyPolicy;
use crate::types::PathId;

/// §119's own `RouteDecision` — see this module's own doc comment for
/// why this is an alias, not a new type.
pub type RouteDecision = crate::decision::RouteDecisionResult;

/// §119's `RouteRequest`, holding the per-call inputs [`crate::decision::PolicyLayers`]
/// and friends deliberately don't belong on (see this module's own
/// doc comment). `candidates`/`current` are owned, not borrowed — a
/// request crossing an `async fn` boundary shouldn't carry a
/// lifetime tied to whatever the caller's stack frame looked like
/// when it built the request.
#[derive(Debug, Clone)]
pub struct RouteRequest {
    pub candidates: Vec<PathCandidate>,
    pub descriptor: OperationDescriptor,
    pub user: PrivacyPolicy,
    pub current: Option<PathCandidate>,
    pub device: Option<DeviceState>,
    pub now_millis: u64,
}

/// §121 "Feedback Loop"'s own first step, "transport result" — the
/// two outcomes a real transport attempt actually has. Reuses
/// [`RouteFailureClass`] (§36, already real since round 2) rather
/// than inventing a second failure taxonomy; a caller's transport
/// layer classifying its own error into that enum is exactly the
/// "natural integration point" that type's own doc comment already
/// says it's waiting for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteOutcome {
    Success,
    Failed(RouteFailureClass),
}

/// §121 "Feedback Loop"'s own "health update" step, made real as a
/// pure, stateless single-sample transition rather than left as a
/// diagram arrow. Deliberately does **not** take or return any
/// history (no failure count, no window, no clock) — that's the
/// "metrics update" step immediately before it in the spec's own
/// diagram, and this crate has kept that kind of state out of scope
/// consistently since round 9's [`crate::explain::RouteMetricEvent`]
/// (see that type's own doc comment). What's left, after subtracting
/// history, is exactly this: "given the health this path currently
/// has, and one new outcome, what's the health right now" — a
/// caller that *does* keep history is free to feed a whole sequence
/// of outcomes through this function one at a time, or use its own
/// history-aware model instead; this one just needs to be honest
/// about only ever seeing one sample.
///
/// `Success` never jumps straight from `Unreachable`/`Suspect`/
/// `Unknown` to [`crate::types::RouteHealth::Healthy`] in one call —
/// one success after a failing streak is evidence of recovery, not
/// proof of it, so it lands on `Degraded` first the same way a real
/// health tracker would want a little more than a single data point
/// before fully trusting a path again. From `Degraded`, one more
/// success is enough to reach `Healthy`; a path that was already
/// `Healthy` simply stays `Healthy`.
pub fn health_after_outcome(
    current: crate::types::RouteHealth,
    outcome: RouteOutcome,
) -> crate::types::RouteHealth {
    use crate::types::RouteHealth;
    match outcome {
        RouteOutcome::Success => match current {
            RouteHealth::Healthy | RouteHealth::Degraded => RouteHealth::Healthy,
            RouteHealth::Suspect | RouteHealth::Unreachable | RouteHealth::Unknown => {
                RouteHealth::Degraded
            }
        },
        RouteOutcome::Failed(class) => match class {
            RouteFailureClass::Temporary => match current {
                RouteHealth::Healthy => RouteHealth::Degraded,
                _ => RouteHealth::Suspect,
            },
            RouteFailureClass::Unknown => RouteHealth::Suspect,
            RouteFailureClass::TransportUnavailable
            | RouteFailureClass::AuthenticationFailure
            | RouteFailureClass::PolicyDenied
            | RouteFailureClass::RemoteRejected
            | RouteFailureClass::Permanent => RouteHealth::Unreachable,
        },
    }
}

/// §119's `RouteResultReport`. `operation_id`/`path_id` identify
/// *which* plan this is feedback about — the same two ids
/// [`crate::descriptor::OperationDescriptor`]'s own doc comment
/// already names as what a receiver needs to deduplicate a
/// hedged/redundant send; feedback about a specific attempt needs the
/// same pair to say which attempt it's about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteResultReport {
    pub operation_id: OperationId,
    pub path_id: PathId,
    pub outcome: RouteOutcome,
}

/// §130 "Call Path Change Integration": "Routing reports: new path,
/// quality update. Call/media engine handles: rebind, renegotiate,
/// adapt bitrate." Only the reporting half is this crate's to build —
/// `rebind`/`renegotiate`/`adapt bitrate` are call/media-layer
/// actions this crate has no media session to perform them on, the
/// same boundary [`RouteResultReport`]'s own doc comment already
/// draws for feedback in the other direction. `QualityUpdate` carries
/// [`crate::metrics::PathMetrics`] rather than a derived score,
/// leaving "should this trigger a bitrate change" as a call-layer
/// judgment call this crate doesn't make on its behalf.
#[derive(Debug, Clone, PartialEq)]
pub enum RouteChangeEvent {
    NewPath {
        primary: PathCandidate,
    },
    QualityUpdate {
        metrics: crate::metrics::PathMetrics,
    },
}

/// §119, transcribed with native async-fn-in-trait syntax — see this
/// module's own doc comment for why that's a faithful transcription
/// here and not a departure from the spec's own `async fn` signature.
/// `async_fn_in_trait` is silenced deliberately: the lint's own
/// concern is that a trait method can't add a `Send` bound on its
/// returned future without a breaking signature change later — real
/// for a trait meant to be called across an executor's worker
/// threads, but this crate has no async runtime dependency at all
/// (see this module's own doc comment) and isn't the one deciding
/// that; a caller adopting this trait for a real multi-threaded
/// runtime is free to re-express it with an explicit `impl Future +
/// Send` return themselves, the same accommodation the lint's own
/// suggestion already names.
#[allow(async_fn_in_trait)]
pub trait RoutingEngine {
    async fn plan(&self, request: RouteRequest) -> Result<RouteDecision, RoutingError>;
    async fn report_result(&self, report: RouteResultReport);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::decision::{ApplicationPolicy, PolicyLayers, SystemPolicy};
    use crate::descriptor::ByteCount;
    use crate::discovery::DiscoveryBudget;
    use crate::metrics::PathMetrics;
    use crate::policy::{RoutingPolicy, RoutingPolicyProfile};
    use crate::requirements::DeliveryRequirements;
    use crate::scoring::DefaultScorer;
    use crate::types::{
        Destination, MeteredState, PathCapabilities, PathId as PathIdType, RoamingState,
        RouteHealth, TransportKind,
    };
    use siar_domain::{AccountId, DeviceId};
    use siar_identity_multidevice::{
        DeviceCapabilitySet, DeviceCertificate, DeviceDirectory, DeviceDirectoryEntry,
        DeviceStatus, RootIdentityKey, TrustedAccountStore,
    };
    use std::sync::Mutex;
    use std::task::{Context, Poll, Wake};

    /// §118's own "each `CommunicationRuntime` owns its routing
    /// engine" — a real, minimal implementation, not just a trait
    /// with no body anywhere. Holds every one of the long-lived
    /// pieces [`RouteRequest`] deliberately doesn't carry.
    struct TestEngine {
        account_id: AccountId,
        trust_store: TrustedAccountStore,
        routing_policy: RoutingPolicy,
        system: SystemPolicy,
        application: ApplicationPolicy,
        discovery_budget: Mutex<DiscoveryBudget>,
        /// Per-path current health, updated by [`health_after_outcome`]
        /// — the one piece of history this reference engine keeps,
        /// specifically because it's exactly the single-sample state
        /// [`health_after_outcome`] itself needs and nothing more (no
        /// counts, no window). A real engine's actual `PathMetrics`
        /// store is the natural place this would really live.
        health_by_path:
            Mutex<std::collections::HashMap<crate::types::PathId, crate::types::RouteHealth>>,
    }

    impl RoutingEngine for TestEngine {
        async fn plan(&self, request: RouteRequest) -> Result<RouteDecision, RoutingError> {
            let layers = PolicyLayers {
                system: &self.system,
                application: &self.application,
                user: &request.user,
            };
            let scorer = DefaultScorer {
                weights: self.routing_policy.weights,
            };
            let mut budget = self.discovery_budget.lock().unwrap();
            Ok(crate::decision::decide_route(
                &request.candidates,
                &request.descriptor,
                &layers,
                self.account_id,
                &self.trust_store,
                &self.routing_policy,
                &scorer,
                request.current.as_ref(),
                request.device.as_ref(),
                Some(&mut budget),
                request.now_millis,
            ))
        }

        /// §121's own "health update" step, wired to a real (if
        /// minimal) store this time — see [`health_after_outcome`]'s
        /// own doc comment for why this is the *only* history this
        /// reference engine keeps.
        async fn report_result(&self, report: RouteResultReport) {
            let mut health = self.health_by_path.lock().unwrap();
            let current = health
                .get(&report.path_id)
                .copied()
                .unwrap_or(crate::types::RouteHealth::Unknown);
            health.insert(
                report.path_id,
                health_after_outcome(current, report.outcome),
            );
        }
    }

    fn candidate(peer: DeviceId) -> PathCandidate {
        PathCandidate {
            path_id: PathIdType::new(),
            transport: TransportKind::IrohDirect,
            peer,
            endpoint: TransportEndpoint(Vec::new()),
            metrics: PathMetrics::unknown(),
            capabilities: PathCapabilities {
                reliable_stream: true,
                datagram: false,
                large_files: false,
                realtime_media: false,
                peer_discovery: false,
                store_and_forward: false,
                metered: MeteredState::Unmetered,
                roaming: RoamingState::Unknown,
                requires_foreground: false,
            },
            health: RouteHealth::Healthy,
            underlay: None,
            state: crate::acquisition::CandidateState::Active,
        }
    }

    fn test_engine_with_one_device() -> (TestEngine, AccountId, DeviceId) {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device = DeviceId::new();
        let cert = DeviceCertificate::issue(
            &root,
            account,
            device,
            [1u8; 32],
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            1,
        );
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![DeviceDirectoryEntry {
                device_id: device,
                certificate: cert,
                status: DeviceStatus::Active,
                transport_endpoints: vec![],
            }],
        );
        let mut trust_store = TrustedAccountStore::new();
        trust_store
            .accept(directory, &root.root_public_key())
            .unwrap();

        let engine = TestEngine {
            account_id: account,
            trust_store,
            routing_policy: RoutingPolicyProfile::Balanced.policy(),
            system: SystemPolicy {
                max_operation_bytes: ByteCount(u64::MAX),
            },
            application: ApplicationPolicy::default(),
            discovery_budget: Mutex::new(DiscoveryBudget::for_priority(
                crate::types::Priority::Normal,
            )),
            health_by_path: Mutex::new(std::collections::HashMap::new()),
        };
        (engine, account, device)
    }

    /// This crate has no async runtime dependency at all (see this
    /// module's own doc comment), so there's no `#[tokio::test]` to
    /// reach for. Every future produced by [`TestEngine`]'s own
    /// `async fn` impls above resolves on its very first poll — they
    /// call only ordinary synchronous code, never `.await` anything
    /// that would actually park — so a no-op [`Wake`] and a single
    /// `poll` call is a complete, honest executor for exactly this
    /// case, not a hack that happens to work by luck.
    struct NoopWaker;
    impl Wake for NoopWaker {
        fn wake(self: std::sync::Arc<Self>) {}
    }

    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        let waker = std::task::Waker::from(std::sync::Arc::new(NoopWaker));
        let mut cx = Context::from_waker(&waker);
        // SAFETY: `fut` is a local variable never moved again after
        // this point, satisfying `Pin`'s contract without needing
        // `Box::pin` — the same "stack-pin a local future" pattern
        // `std::pin::pin!` (stable since 1.68) exists specifically
        // to make convenient; used directly here to avoid adding it
        // as a new import surface for one test helper.
        let fut = std::pin::pin!(fut);
        match fut.poll(&mut cx) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("test future did not resolve on its first poll"),
        }
    }

    #[test]
    fn a_routing_engine_implementation_plans_a_real_route() {
        let (engine, account, device) = test_engine_with_one_device();
        let descriptor = OperationDescriptor {
            operation_id: OperationId::new(),
            destination: Destination::Account(account),
            requirements: DeliveryRequirements::interactive_message(),
            estimated_size: ByteCount(10),
            content_class: crate::descriptor::ContentClass::Text,
            required_extension: None,
            created_at_millis: 0,
        };
        let request = RouteRequest {
            candidates: vec![candidate(device)],
            descriptor,
            user: PrivacyPolicy::default(),
            current: None,
            device: None,
            now_millis: 0,
        };

        let result = block_on(engine.plan(request)).unwrap();
        assert!(matches!(result, RouteDecision::Routed(_)));
    }

    #[test]
    fn report_result_updates_the_engines_own_per_path_health() {
        let (engine, _account, _device) = test_engine_with_one_device();
        let path_id = PathIdType::new();
        let report = RouteResultReport {
            operation_id: OperationId::new(),
            path_id,
            outcome: RouteOutcome::Failed(RouteFailureClass::Temporary),
        };
        block_on(engine.report_result(report));
        // Unknown (no prior record) + Temporary failure → Suspect,
        // per `health_after_outcome`'s own match arms.
        assert_eq!(
            engine.health_by_path.lock().unwrap().get(&path_id).copied(),
            Some(crate::types::RouteHealth::Suspect)
        );
    }

    /// §121's own "health update" step, tested as a set of pure
    /// transitions rather than only through `TestEngine`'s
    /// integration above.
    #[test]
    fn health_after_outcome_recovers_gradually_not_instantly() {
        use crate::types::RouteHealth;
        assert_eq!(
            health_after_outcome(RouteHealth::Unreachable, RouteOutcome::Success),
            RouteHealth::Degraded
        );
        assert_eq!(
            health_after_outcome(RouteHealth::Degraded, RouteOutcome::Success),
            RouteHealth::Healthy
        );
        assert_eq!(
            health_after_outcome(RouteHealth::Healthy, RouteOutcome::Success),
            RouteHealth::Healthy
        );
    }

    #[test]
    fn health_after_outcome_maps_permanent_style_failures_straight_to_unreachable() {
        use crate::types::RouteHealth;
        for class in [
            RouteFailureClass::TransportUnavailable,
            RouteFailureClass::AuthenticationFailure,
            RouteFailureClass::PolicyDenied,
            RouteFailureClass::RemoteRejected,
            RouteFailureClass::Permanent,
        ] {
            assert_eq!(
                health_after_outcome(RouteHealth::Healthy, RouteOutcome::Failed(class)),
                RouteHealth::Unreachable,
                "failure class {class:?} should map straight to Unreachable"
            );
        }
    }

    /// §130 "Call Path Change Integration" — a construction/matching
    /// test is genuinely all there is to check here: the type is pure
    /// data with no logic of its own (see this module's own doc
    /// comment for why "rebind"/"renegotiate"/"adapt bitrate" aren't
    /// this crate's to implement).
    #[test]
    fn route_change_event_reports_a_new_primary_without_acting_on_it() {
        let event = RouteChangeEvent::NewPath {
            primary: candidate(DeviceId::new()),
        };
        match event {
            RouteChangeEvent::NewPath { primary } => {
                assert_eq!(primary.transport, TransportKind::IrohDirect)
            }
            RouteChangeEvent::QualityUpdate { .. } => panic!("expected NewPath"),
        }
    }

    #[test]
    fn route_change_event_can_report_a_quality_update() {
        let event = RouteChangeEvent::QualityUpdate {
            metrics: crate::metrics::PathMetrics::unknown(),
        };
        assert!(matches!(event, RouteChangeEvent::QualityUpdate { .. }));
    }
}
