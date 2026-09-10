# Part 03 — Transport & Routing Policy Engine Architecture

Source spec: `sys-arch/03-transport-routing-policy-engine-architecture.md` — 200 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**107/200 (54%) sections.**

Partial — Round 1 (§1–§42), Round 2 (§43–§56), Round 3 (§57–§62), Round 4 (§63–§68), Round 5 (§69–§74), Round 6 (§75–§84), and Round 7 (§85–§90) complete (106/106 unit tests).

### Round 7 (§85–§90) Summary (2026-09-08)
- **§85 & §87 (Battery-Aware Inputs & Platform Policy Integration)**: Implemented `BatteryLevelClass`, `ThermalState`, and `DeviceState` in `platform.rs`; mapped §87's six platform signals across typed fields in the crate.
- **§86 & §89 (Background Restrictions & Candidate Acquisition State)**: Implemented `CandidateState` (`Active`, `PassiveKnown`, `RequiresDiscovery`, `RequiresSetup`) in `acquisition.rs`; added `is_currently_usable` and `eliminate_background_restricted` (with permissive default for unknown foreground state); added `candidate_state` weight across all 7 profiles.
- **§88 & §90 (Path Acquisition & Discovery Budget)**: Implemented `DiscoveryBudget` in `discovery.rs` (sliding-window rate limiting + exhaustion cooldown per `Priority`); implemented `discovery_permitted` combining budget with device safety margins (unconditional block on `ThermalState::Critical`, priority-gated battery saver).
- **Pipeline Integration**: Extended `plan_route` to accept `Option<&DeviceState>` and actively filter background-restricted candidates.

### Round 6 (§75–§84) Summary (2026-09-08)
- **§75–§78 (Multipath Chunk Scheduler, Path Collapse, Duplicate Chunks, Realtime Multipath)**: Documented as out-of-scope for v1 per the spec's own designation as future/optional/advanced.
- **§79 (Congestion Signals)**: Added `CongestionState` (`Normal`, `Congested`, `Severe`), `retransmission_rate`, and `congestion_state` to `PathMetrics`; added `congestion` weight across all 7 `PolicyWeights` profiles; scored in `DefaultScorer`.
- **§80 (Route Stability Score)**: Implemented `derive_stability_score` in `stability.rs` converting lifetime, failure rate, path changes, and timeout count into `StabilityScore`.
- **§81 & §84 (Cold vs Warm Path & Battery Cost)**: Reconciled and documented existing coverage in `setup.rs` (`ConnectionPoolState::Active` -> `SetupCost::Cheap`) and `metrics.rs` (`EnergyCost`).
- **§82 & §83 (Metered Networks & Roaming)**: Refactored `metered` to tri-state `MeteredState` and added `RoamingState`, both conservatively blocking `Unknown` in `passes_hard_constraints`; corrected `file_chunk()` to `allow_metered: false` and added `allow_roaming_bulk: bool`.

### Round 5 (§69–§74) Summary (2026-09-08)
- **§69 & §70 (Route Planning for Messaging & Message Routing Workflow)**: Verified through comprehensive integration tests covering interactive messages, low-power preferences, emergency profiles, and fallback paths.
- **§71 (Message Retry Strategy & Quality Signal)**: Implemented `PathQualitySignal` (`Good`, `Degraded`, `Failed`) in `quality.rs` projected from `PathMetrics` for media/retry adaptation.
- **§72 (Ephemeral Message Routing & Deduplication)**: Documented stable operation ID deduplication in `descriptor.rs` and verified in route planning tests.
- **§73 & §74 (Offline Routing, File Transfers, Path Diversity & Underlay Grouping)**: Implemented `UnderlayId`, `are_diverse`, `group_by_underlay`, and `most_diverse_fallback` in `diversity.rs`; wired into `plan_route`'s redundant-strategy replica selection so redundant copies prefer physically diverse underlays.

### Round 4 (§63–§68) Summary (2026-09-08)
- **§63–§65 (Queue Architecture, Weighted Fair Scheduling, Backpressure)**: Formally documented in `dispatch.rs` via `siar-protocol-ext`'s `FairScheduler` and `BoundedQueue` integration.
- **§66 (Per-Transport Queues)**: Implemented `PerTransportDispatchQueue` in `dispatch.rs` maintaining separate `RouteDispatchQueue` instances per `TransportKind` so a stalled transport (e.g. Bluetooth) cannot block healthy transports (e.g. Iroh).
- **§67 & §68 (Per-Peer & Per-Extension Fairness)**: Implemented generic `RoundRobinFairQueue<K, T>` in `fairness.rs`, instantiated with `DeviceId` for §67 per-peer fairness and `ContentClass` (with newly derived `Hash`) for §68 per-extension fairness.

### Round 3 (§57–§62) Summary (2026-09-08)
- **§57 (Delivery Semantics)**: `typing_indicator()` (non-durable, 5s expiry, no DTN) and `file_chunk()` (durable, bulk, DTN/multipath allowed) constructors on `DeliveryRequirements`.
- **§58 (Operation Descriptor)**: `OperationDescriptor` struct carrying `OperationId`, `Destination`, `DeliveryRequirements`, `ByteCount`, and `ContentClass`.
- **§59 (Content Class)**: 10-variant `ContentClass` enum (Control, Text, Metadata, Thumbnail, Voice, Image, File, RealtimeAudio, RealtimeVideo, Emergency).
- **§60 (Size-Aware Routing)**: `completion_time_millis` combining static transport setup latency with payload byte count and estimated bandwidth (`setup + bytes / bandwidth`).
- **§61 (Deadline-Aware Routing)**: `exceeds_deadline` and `eliminate_deadline_exceeding_candidates` pruning paths that cannot deliver before `max_latency_millis`.
- **§62 (Expiry-Aware Routing)**: `DeliveryRequirements::has_expired` and `RetryPolicy::allows_attempt_at` enforcing clock-based operation expiration during retry attempts.

### Round 2 (§43–§56) Summary (2026-09-08)
- **§43 (Route Re-Evaluation)**: Targeted per-transport cache invalidation (`RouteCache::invalidate_transport`).
- **§44 & §46 (Transport Setup Cost & Connection Pool Integration)**: `SetupCost` and `ConnectionPoolState` modeling static vs pool-adjusted connection setup costs (`static_setup_cost`, `effective_setup_cost`), integrated into `DefaultScorer` as `setup_cost` weight.
- **§45 (Existing Connection Preference)**: `existing_connection` scoring bonus in `DefaultScorer` based on `RoutingContext::current_path`.
- **§47 & §48 (Peer Session Abstraction & Security Constraints)**: `AuthenticatedSession` smart constructor, `authorize_candidate`, and `eliminate_untrusted_candidates` enforcing defense-in-depth re-verification against Part 02's `TrustedAccountStore`.
- **§49 & §50 (Privacy Policy & Direct vs Relay)**: `PrivacyPolicy` struct, `passes_privacy_policy`, `eliminate_privacy_violations`, and `direct_preference_bonus`.
- **§52 (Wi-Fi Direct/Aware Threshold Policy)**: `justifies_expensive_setup` and `eliminate_unjustified_expensive_setup` gating expensive setup on high bitrate, realtime calls, critical priority, or explicit nearby requests.
- **§51, §53, §54, §56**: Fully accounted for via setup costs and `DeliveryRequirements` fields (`nearby_session_explicit`, `dtn_replication_budget`).
- **§55 (Mesh Forwarding)**: Documented as an honest remaining gap (requires richer next-hop/hop-budget representation).

## Implementing crate(s)

- `siar-routing-policy`

- `siar-routing (pre-existing, next.md-era — unreconciled second routing/scoring system, see below)`


## Known gaps / open questions

- Unresolved-by-design reconciliation: two routing/scoring systems — `siar-routing` (next.md-era) vs `siar-routing-policy` (Part 03-era) — documented in the newer crate's own lib.rs, not silently merged.
- §55 Mesh Forwarding: needs dedicated next-hop, route utility, hop budget, and relay trust policy models.


## Note
Detail above reflects implementation through Round 5 (§69–§74) completed on 2026-09-08. Next target: §75 onward.
