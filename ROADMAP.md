# siar — Roadmap & Coverage Hierarchy

Maintained alongside the code, not a substitute for it. This file tracks
**what's written vs. what's left**, ordered by actual need — not spec
number order. Update this file every time a spec's coverage changes.

Spec source: `sys-arch/` (33 numbered core-architecture docs +
27 `ui-ux-NN` docs, uploaded once, Wiki-style). Workspace source:
33-crate Rust workspace (`siar-source`) + `apps/{cli,desktop,android,
emergency-node}` + `platform/android`.

Legend: ✅ Done/substantial · 🟡 Partial · ⚪ Not started · 🔒 Blocked on
a prerequisite · 📋 Reconciled (already covered elsewhere, no new code
needed)

---

## Tier 0 — Foundational core crates (Parts 01-09)

All nine have real, compile+test-verified coverage, tracked crate-by-
crate. Status: **substantially built, gaps are depth not breadth** —
each has a documented list of untouched later sections. See per-crate
notes in project memory (`/areas/resilient-mesh.md`) for exact section
counts; not re-duplicated here since that detail changes less than this
file's priority ordering does.

**2026-09-01: working through all nine 1-by-1, round by round.** Total
remaining across all nine is ~1,110 sections (1,552 total, ~445 done
after this round) — at this project's actual historical pace (5-15
sections per round, each needing real spec-reading + code + compile +
test, not just prose), that's realistically 70-140+ more rounds, not
something any single session finishes. Tracking progress here as it
happens rather than promising a completion date. This round: spec 01
(`siar-protocol-ext`) §17 (confirmed already done, doc was stale),
§24-26, §28-31, §32-33 — see its own crate row below and its `lib.rs`
doc comment for specifics. Also fixed two real bugs in `siar-transport`
(pooled-connection reuse silently dropping every message after the
first) and added full test coverage to `siar-transport`/`siar-messaging`'s
`MessageService` half this session — see `/areas/resilient-mesh.md`,
not spec-numbered work but real correctness fixes found via testing.

**2026-09-05/06 update:** spec 02 (`siar-identity-multidevice`) is now
ALSO complete (204/204), across 11 rounds this session (§91-204) on top
of the ~90/204 already done before this session started. Two of the
nine Tier 0 crates are now fully spec-complete (01 and 02); the
remaining ~1,110-section estimate above is now overstated by roughly
114 sections (204 − 90) — a precise updated total across all nine
hasn't been recomputed, but 03-09 remain the actual "~1,110 minus 114"
scope.

**2026-09-08 update:** spec 03 (`siar-routing-policy`) round 2 done —
§43-56 (~14 sections), bringing it to ~74/200. Real new code: §43
targeted per-transport cache invalidation; §44/§46 a `setup.rs` module
(`SetupCost`/`ConnectionPoolState`) now folded into
`DefaultScorer` as two new `PolicyWeights` terms
(`setup_cost`/`existing_connection`, tuned per profile) — closing a gap
this crate's own doc comments had named but not filled; §47/§48 a new
`security.rs` with an `AuthenticatedSession` smart-constructor that
re-verifies a candidate's peer against Part 02's `TrustedAccountStore`
(defense-in-depth on top of `resolve.rs`'s existing device-list-stage
filtering); §49/§50/§52 a new `privacy.rs` (`PrivacyPolicy` hard
constraints, a direct-preference soft bonus, and §52's Wi-Fi
Direct/Aware setup-threshold elimination). §51/§53/§54/§56 accounted
for via existing/new fields without new functions (documented in
`privacy.rs`'s own doc comment); §55 "Mesh Forwarding" named as a real,
unimplemented gap (no distinct next-hop/hop-budget candidate shape
exists). Two new `DeliveryRequirements` fields
(`nearby_session_explicit`, `dtn_replication_budget`) — required a
downstream fix in `siar-dtn-bundle`'s own test helper (missing-fields
compile error caught by a full-workspace check, not just this crate's
own `cargo check`). 49/49 tests (up from 35), clippy clean, fmt clean,
doc-warning-free, zero regressions verified in `siar-dtn-bundle`
(37/37), `siar-identity-multidevice` (251/251), and `siar-protocol-ext`
(115/115). Next for spec 03: §57 onward.

**2026-09-08 update (round 3):** §57-62 done, ~74→~80/200. New
`descriptor.rs` — `OperationId`/`ByteCount`/`ContentClass`/
`OperationDescriptor`, named exactly per §58/§59; two new
`DeliveryRequirements` constructors (`typing_indicator`, `file_chunk`)
close out §57's own four worked examples (message/call-frame already
had constructors from Phase 1). New `estimate.rs` — §60's own
`completion_time ≈ setup + bytes / bandwidth` formula as a real
function (`completion_time_millis`, `None` for "with uncertainty"
rather than a fabricated confidence interval), plus §61's hard
"exceeds deadline → drop" elimination
(`eliminate_deadline_exceeding_candidates`). §62 closed without a new
module: `DeliveryRequirements::has_expired` and
`RetryPolicy::allows_attempt_at` finally give the `expiry_millis` field
(present since Phase 1) an actual enforcement path — it had sat unused
until this round. 63/63 tests (up from 49), clippy clean, fmt clean,
doc-warning-free (one broken intra-doc link from `[Routing]` in a doc
comment caught and escaped), zero regressions in `siar-dtn-bundle`
(37/37), `siar-identity-multidevice` (251/251), `siar-protocol-ext`
(115/115) — this round added no new `DeliveryRequirements` fields, so
no downstream literal-construction fix was needed this time.

**2026-09-08 update (round 4):** §63-68 done, ~80→~86/200.
Confirmed §63/§64/§65 ("Queue Architecture"/"Weighted Fair
Scheduling"/"Backpressure") were already real one layer down in
`siar-protocol-ext`'s `FairScheduler`/`BoundedQueue`, wired through
this crate's existing `dispatch.rs` bridge — documented in that
module's own doc comment rather than reimplemented. New this round:
`PerTransportDispatchQueue` (§66) — one independent
`RouteDispatchQueue` per `TransportKind`, so a stalled Bluetooth
backlog genuinely cannot block a healthy Iroh one, which a single
shared queue could never guarantee. New `fairness.rs` module: one
generic `RoundRobinFairQueue<K, T>` (bounded per-key, round-robin
across whichever keys have pending items) instantiated two ways in its
own tests — keyed by `DeviceId` for §67 "Per-Peer Fairness" and by the
new `ContentClass` (from round 3's `descriptor.rs`) for §68
"Per-Extension Fairness" — one primitive closing two spec sections
rather than writing the same round-robin logic twice. Deliberately not
wired into `RouteDispatchQueue` itself: `FairScheduler`'s own per-tier
queue is a fixed plain FIFO owned by `siar-protocol-ext`, not
swappable from this crate — composable instead, same posture as
`security.rs`/`privacy.rs`'s own elimination functions. 71/71 tests (up
from 63), clippy clean, fmt clean, doc-warning-free, zero regressions
in `siar-dtn-bundle` (37/37), `siar-identity-multidevice` (251/251),
`siar-protocol-ext` (115/115). This round's diff was delivered as a
git patch (`git format-patch`/`git diff` against round-3's tree) per
explicit request, not a tarball — the project now has an actual git
repo (initialized this round, baseline commit = round 3's state) to
make that possible going forward.

**2026-09-08 update (round 5):** §69-74 done, ~86→~92/200. New
`diversity.rs`: a real `UnderlayId` newtype (§74's own code block is
literally just `pub struct UnderlayId;` — read as an illustrative
stub, not a literal instruction, same as this crate already does for
similar bare-stub spec snippets), `are_diverse`/`group_by_underlay`,
and `most_diverse_fallback` — and unlike round 4's `fairness.rs`
primitive (deliberately left composable/unwired), this one **is**
wired directly into `plan_route`'s existing redundant-strategy replica
selection, replacing a blind "take the next-best-scored fallback" with
"take the best-scored fallback that's actually on a different
underlay." New `quality.rs` for §71: `PathQualitySignal`/
`quality_signal_for`, a purpose-narrowed projection of `PathMetrics`
for a call's media-adaptation consumer (the video→audio→voice-message
fallback ladder itself stays out of scope — that's a media decision,
not a routing one).

§69/§70/§72 ("Route Planning for Messaging/Files/Emergency") were
verified rather than separately implemented — real integration tests
against `plan_route`'s actual output. This caught a real mistake
before it shipped: the first draft of the §69 messaging test assumed
DTN would naturally rank last "for free" from existing scoring terms;
running it showed that assumption was false (DTN's *setup* cost is
cheap — no live connection to establish — which is a different
property from DTN being a *good* choice for immediate delivery, and
nothing in current scoring captures that difference without real
caller-supplied latency data). The test was corrected to only assert
what's robustly true (cheap-setup transports beat expensive-setup ones
all else equal) rather than a claim that happened to be untested and
wrong. §70's files test uses realistic differentiated metrics (a real
`min_bandwidth` floor plus measured bandwidth per candidate) rather
than relying on coincidental tie-breaking. §72's emergency-composition
test was robust as originally written and needed no correction.

82/82 tests (up from 71), clippy clean, fmt clean, doc-warning-free
(one broken intra-doc link to a `#[cfg(test)]`-only module, caught and
reworded), zero regressions in `siar-dtn-bundle` (37/37,
no `PathCandidate` literal construction there so no downstream fix
needed this time), `siar-identity-multidevice` (251/251),
`siar-protocol-ext` (115/115). Delivered as a git commit on top of
round 4's tree (same repo from round 4, not re-initialized). Next for
spec 03: §75 onward — §75-78 (multipath chunk scheduler/path collapse/
duplicate chunks/realtime multipath) are explicitly named by the spec
itself as future/optional/advanced work, not v1, so likely to be
documented as out-of-scope rather than implemented; §79 (congestion
signals) is partially covered already by existing `PathMetrics` fields
(rtt/loss/bandwidth) — worth checking exactly what's still missing
(send-queue depth, connection-congestion signal) before assuming it
needs a full new module.

**2026-09-08 update (round 6):** §75-84 done, ~92→~101/200. §75-78
(multipath chunk scheduler, path collapse, duplicate chunks, realtime
multipath) documented as out-of-scope — the spec itself names all four
as future/optional/advanced, not v1. §81 (Cold vs Warm Path) and §84
(Battery Cost) turned out to already be covered by existing work
(`ConnectionPoolState`/`effective_setup_cost` from round 2, and
`EnergyCost` from Phase 1 respectively) — documented rather than
reimplemented, per the note left last round to check before assuming
new code is needed.

Real new work: §82/§83 (Metered Networks, Roaming) got a genuine
`bool`→tri-state refactor — new `MeteredState`/`RoamingState` enums
(`Metered`/`Unmetered`/`Unknown` and `Roaming`/`NotRoaming`/`Unknown`)
replacing a plain `metered: bool` that couldn't express "the platform
hasn't told us yet," wired into `passes_hard_constraints` via two new
functions that both treat `Unknown` conservatively. This refactor
caught a real, pre-existing bug: `DeliveryRequirements::file_chunk()`
had `allow_metered: true`, directly contradicting §82's own "block
bulk" worked example — now corrected, with a new `allow_roaming_bulk`
field added for §83's own "allow cellular but forbid roaming bulk
transfer" example. §79 (Congestion Signals) added `CongestionState`
plus two new `PathMetrics` fields, finally closing a gap
`DefaultScorer`'s own doc comment had named as unmodeled since this
crate's first phase — congestion is now a real scored term across all
7 policy profiles; failure penalty remains open (documented) since it
needs failure history this crate keeps no record of. §80 (Route
Stability Score) got a new `derive_stability_score` heuristic turning
lifetime/failure-rate/path-changes/timeouts into the existing
`StabilityScore` enum.

Process note: this round's §82/§83 work was actually started in an
unfinished prior session and found already sitting uncommitted in the
working tree at the start of this round — it compiled and was
functionally sound, but had no dedicated tests for the new
metered/roaming hard-constraint logic. Six tests were added before
treating it as done, rather than trusting that "it compiles" meant
"it's finished." As anticipated last round, adding `allow_roaming_bulk`
to `DeliveryRequirements` required the same downstream
`siar-dtn-bundle` test-helper fix as round 2's field additions did —
worth continuing to expect this every time a `DeliveryRequirements`
field is added, and to keep checking the whole workspace, not just
this crate, before calling a round done.

95/95 tests (up from 82), clippy clean, fmt clean, doc-warning-free
(two broken intra-doc links caught: one from this round's own edit,
one pre-existing in the found-uncommitted work). Zero regressions in
`siar-dtn-bundle` (37/37, after the fix above), `siar-identity-multidevice`
(251/251), `siar-protocol-ext` (115/115).

**2026-09-08 update (round 7):** §85-90 done, ~101→~107/200. New
`platform.rs`: `BatteryLevelClass`/`ThermalState`/`DeviceState` (§85),
plus a doc comment reconciling §87's six named platform signals
against where each actually lives in this crate — three already
existed (network type = `TransportKind`; metered/roaming =
`MeteredState`/`RoamingState` from round 6), the other three land this
round (power saver = `DeviceState::battery_saver`; background
restrictions = the new `requires_foreground` capability flag; radio
availability = `CandidateState`). New `acquisition.rs`: `CandidateState`
transcribed exactly from §89's own code block
(`Active`/`PassiveKnown`/`RequiresDiscovery`/`RequiresSetup`), a new
`candidate_state` scoring weight across all 7 `PolicyWeights` profiles,
and `is_currently_usable`/`eliminate_background_restricted` for §86 —
deliberately the *opposite* default from round 6's metered/roaming
`Unknown` handling: an unreported foreground state defaults
*permissive* here, since (unlike metered/roaming) there's no named
"protect a resource" bias to justify assuming the worst. New
`discovery.rs`: a real stateful `DiscoveryBudget` (sliding window +
cooldown, the first genuinely stateful-across-calls type in this
crate — everything before it was either pure functions or took
`now_millis` as a bare parameter) and `discovery_permitted`, which
layers `DeviceState` on top: thermal-critical blocks unconditionally,
with no priority override (a hardware safety margin — even an SOS
shouldn't force active scanning while the device is about to
thermally shut down), while battery-saver blocks unless
`Priority::Critical` (a user preference, overridable the same way
§52's `justifies_expensive_setup` already treats Critical priority).

All three new pieces are wired through the real pipeline this round,
not left composable-only the way round 4's `fairness.rs` was: `RoutingContext`
gained a `device: Option<DeviceState>` field, `PathCandidate` gained
`state: CandidateState`, `PathCapabilities` gained
`requires_foreground: bool`, and `plan_route` itself gained a `device`
parameter and now actually calls the §86 background-restriction filter
as a real elimination pass. This was the most invasive round yet at
the call-site level: `plan_route`'s own signature changed (a 6th
parameter), which meant fixing all 10 existing call sites in
`plan.rs`'s own test suite, on top of the now-familiar mechanical
`PathCandidate`/`PathCapabilities` literal updates across 9 files. A
genuine bug was caught mid-implementation, not just mid-review this
time: the first draft of `DiscoveryBudget`'s own cooldown test used a
short window that let the second attempt's timestamp age out of the
window before the cap was ever actually hit, so the intended
exhaustion path never triggered — caught by actually running the test
suite (it failed), not by re-reading the code, and fixed by widening
the test's window so the cap is hit while both timestamps are still
live.

106/106 tests (up from 95), clippy clean, fmt clean, doc-warning-free.
No `DeliveryRequirements` fields were touched this round, so — for the
first time since round 3 — no downstream `siar-dtn-bundle` fix was
needed; still confirmed via the full workspace check rather than
assumed. Zero regressions in `siar-dtn-bundle` (37/37),
`siar-identity-multidevice` (251/251), `siar-protocol-ext` (115/115).
Next for spec 03: §91 onward (Route Escalation Ladder, Timeout by
Stage, Hedged Requests, Deduplication Requirement, Route Diagnostics —
§91-95, a natural "resilience mechanics" cluster; §95-99/§124-127
diagnostics-and-testing work is flagged in lib.rs as a whole phase not
yet attempted, worth checking how much of it this round's cluster
might already close incidentally).

**2026-09-08 update (round 8):** §91-96 done, ~107→~113/200. New
`resilience.rs`: `EscalationStage`/`escalation_stage_of` (§91) — no
new state needed at all, built entirely by composing round 7's
`CandidateState` (stages 1-2), round 2's `SetupCost` (the 3-vs-4
lightweight/expensive split), and `TransportKind::Dtn` (stage 5)
directly, confirming the "check how much this cluster might close
incidentally" instinct from last round's note was worth having.
`timeout_millis_for_stage` (§92, same coarse-named-heuristic style as
`estimate.rs`/`discovery.rs`). `HedgePolicy`/`hedge_policy_for` (§93)
— unlike most of this round's other pieces, this one required a real
API change: `RouteStrategy` gained a `Hedged` variant and `RoutePlan`
gained `hedge_delay_millis`, with `plan_route` itself now producing
`Hedged` for high-priority, non-bulk, non-delay-tolerant traffic with
a fallback available. `RouteDiagnostics`/`diagnose` (§95), deliberately
scoped to the two rejection checks (hard constraints, background
restriction) this crate can run without an external policy object —
security/privacy checks need a `TrustedAccountStore`/`PrivacyPolicy` a
caller may not have, and a caller composing those itself already knows
which one rejected a candidate. §94 needed no new function, just a doc
comment extending `OperationId`'s existing dedup note to cover
`Hedged` alongside `Redundant`. §96 "Path Visualization" is explicitly
deferred by the spec itself to "Part 18" — documented, not attempted.

A real bug was caught mid-round by running the test suite, not by
re-reading code: the first hedge-strategy integration test's own
candidates used the default `MeteredState::Unknown`/`RoamingState::Unknown`
from the crate's usual test-helper pattern, which round 6's own hard
constraints correctly block for a Bulk-class, `allow_metered: false`
request — the test failed with `NoEligibleCandidates` until the test's
own candidates were given explicit `Unmetered`/`NotRoaming`
capabilities, the same fix `files_prefer_high_bandwidth_direct_over_small_only_bluetooth`
(round 5) already needed for the same reason.

119/119 tests (up from 106), clippy clean, fmt clean, doc-warning-free
(two broken intra-doc links to private items, caught and fixed). Also
fixed, while in the file: a real content gap in this file's own §85-90
coverage bullet from round 7 — a sentence fragment had gone missing
mid-edit, leaving a dangling `(§171-175)` reference with nothing
before it. Zero regressions in `siar-dtn-bundle` (37/37),
`siar-identity-multidevice` (251/251), `siar-protocol-ext` (115/115).
Next for spec 03: §97 onward.

| # | Crate | State |
|---|---|---|
| 01 | siar-protocol-ext | ✅ **108/108 — spec complete** (final round: §91-92 reconciled, §93-95 error codes/health/recovery, §96-99 scheduler contract/storage/metrics/capability isolation, §100-105 reconciled with notes, §106 honest 16-item Definition of Done self-audit — 4 genuine gaps named, §107-108 reconciled) |
| 02 | siar-identity-multidevice | ✅ **204/204 — spec complete** (final round, 2026-09-05: §190-204 — algorithm agility/downgrade protection utilities kept deliberately minimal per spec's own "avoid needless abstraction" caution; a root-key backup envelope that structurally cannot carry plaintext key material; backup-import validation run before any local state is touched; identity-reset/account-deletion presentations with required disclaimer fields; a guarded organization-offboarding state machine that operates only on organization-scoped device ids, never a personal AccountId; multi-tenant-safe composite keys; migration-fixture round-trip tests (honestly incomplete pending §125); and an itemized 21-item Definition-of-Done self-audit — **19/21 fully done, 2 honestly `PartiallyDone`** (no-UI-shipped confirmation prompt; property/integration tests exist but no real fuzz harness). Also fixed a genuinely broken intra-doc link left over from an earlier round, dropping this crate's doc-warning count from 4 to 3. 6 new modules (`algorithm_agility.rs`, `root_key_backup.rs`, `identity_lifecycle.rs`, `migration_fixtures.rs`, `definition_of_done.rs`) plus a `namespace.rs` extension, 20 new tests, 251/251 total, clippy clean, zero regressions. Across all 11 rounds this session: 137 new tests written, zero regressions in siar-routing-policy/siar-crypto at any point, every round compiled+tested+clippy+fmt+doc-checked for real against the actual uploaded Cargo.lock with rustc 1.91.1. Real, named, still-open gaps carried forward into future work: §125 schema versioning absent from DeviceCertificate/DeviceDirectory; §164 no cargo-fuzz harness; §191 full cross-version migration tests blocked on §125; `storage::IdentityStore`/`transaction`/all four `client_api` traits have zero real call sites anywhere in this workspace yet; `RootTrustCacheEntry`/`VerifiedContact` overlap not consolidated; §107/§91 have no real BLE/Wi-Fi/NFC transport wiring.) |
| 03 | siar-routing-policy | ✅ ~113/200 (round 8, 2026-09-08: §91-96 — new `EscalationStage`/`escalation_stage_of` (§91) built entirely from existing round 2/6/7 types, `timeout_millis_for_stage` (§92), `HedgePolicy`/`hedge_policy_for` (§93) wired into a genuinely new `RouteStrategy::Hedged` that `plan_route` can now actually produce, `RouteDiagnostics`/`diagnose` (§95); §94 closed via a doc-comment extension, §96 explicitly deferred by the spec itself to "Part 18"; a real integration-test failure caught and fixed mid-round (round 6/7's metered/roaming hard constraints blocking a hedge test's own candidates until their capabilities were set correctly); rounds 2-7 covered §43-90 — see this crate's own lib.rs/round notes for what's genuinely covered vs merely accounted-for) |
| 04 | siar-event-log | 🟡 ~10/95 (Phase 2 SQLite blocker below is now STALE — see Tier 3 update) |
| 05 | siar-blob-manifest | ✅ ~23/210 (+ metadata_encryption.rs) |
| 06 | siar-dtn-bundle | ✅ ~50/192 |
| 07 | siar-capability | ✅ ~19/164 |
| 08 | siar-resource-limits | ✅ ~56/193 |
| 09 | siar-crash-recovery | 🟡 ~15/186 (real total corrected this session — earliest-stage of the nine) |

**Three real, unresolved reconciliation questions**, deliberately not
silently resolved — documented in the newer crate's own `lib.rs`:
- two device-cert models (`siar_crypto::device_cert` vs
  `siar-identity-multidevice`)
- two routing/scoring systems (`siar-routing` vs `siar-routing-policy`)
- two DTN bundle models (`siar-dtn` vs `siar-dtn-bundle`)

---

## Tier 1 — Security backbone (Part 28)

**127 sections total. ~46 done. Assessment: good stopping point for
incremental section-by-section work** — what's built covers the
sections that unblock everything downstream (message envelopes, replay
protection, key storage abstraction, device revocation, trust states,
domain separation, safety fingerprints, identity-change UX). What's
left is dominated by four items that are each their own subsystem, not
a next "batch":

| Item | Sections | Why it's not a batch |
|---|---|---|
| The ratchet (forward secrecy/PCR) | §11-13 | Needs a real `vodozemac` integration, not hand-rolled crypto |
| Group Security | §25-27 | Needs reconciling against `siar-crypto-mls` (831 lines, unexamined) |
| Test-vector suite + fuzzing + named attack scenarios | §93-103 | A whole security test harness |
| Workspace crate split | §112-122 | The spec's own ask: split into 10 `comm-security-*` crates — a restructuring decision, not a code batch |

Everything else already closed (§5-10, §14-24, §28-36, §40-44) or
partially reconciled (§37-39 SAS/pairing — real gap: no
`protocol_version` field on `DeviceLinkInvite`, not fixed, breaking
change to an already-shipped signed struct). Untouched and NOT
subsystem-scale (genuinely next-batch-sized whenever picked back up):
§46-92 (abuse resistance, plugin/FFI/embedded boundaries, crash-safe
ratchets, diagnostics, failure taxonomy, threat-model docs, ADRs,
disclosure process, compromise-response playbooks), §104-111
(performance, security profiles).

**Recommendation: pause Part 28 here.** Come back to §46-92/§104-111 in
ordinary batches later; treat the four subsystem items as their own
dedicated project when prioritized.

---

## Tier 2 — UI/UX (27 specs, `ui-ux-01` through `ui-ux-27`)

**Started this round. 1 of 27 specs touched (partially).** This tier
was entirely ⚪ before this session — apps/desktop and apps/android
have substantial pre-existing UI code (chat, groups, attachments) but
none of it had been reconciled against the formal 27-spec set until
now.

| # | Spec | State |
|---|---|---|
| 01 | Product Foundation / Cross-Platform Interaction | ⚪ |
| 02 | Desktop (Dioxus) App Shell / Navigation | ⚪ (apps/desktop's shell pre-dates this spec, unreconciled) |
| 03 | Android (Compose) App Shell / Navigation | ⚪ (apps/android's shell pre-dates this spec, unreconciled) |
| 04 | Conversation List / Inbox | ⚪ (siar-ui-state::conversation_list.rs pre-exists, unreconciled) |
| 05 | Message Timeline | ⚪ (siar-ui-state::timeline.rs pre-exists, unreconciled) |
| 06 | Composer / Attachments / Voice / Drafts | ⚪ (siar-ui-state::composer.rs pre-exists, unreconciled) |
| 07 | Calls / Realtime Media | ⚪ (siar-calls crate pre-exists, unreconciled) |
| 08 | Contacts / Requests / Verification / Identity | ⚪ (siar-ui-state::contact_list.rs pre-exists, unreconciled) |
| 09 | Groups / Membership / Roles | ⚪ (siar-ui-state::group_list.rs pre-exists, unreconciled) |
| 10 | Files / Media Gallery / Transfer | ⚪ |
| 11 | Search / Local Knowledge Retrieval | ⚪ (Part 32 has no crate either — joint gap) |
| 12 | Nearby / QR / NFC Pairing / Device Linking | ⚪ (Part 15 has no crate either — joint gap) |
| 13 | Notifications / Background / Incoming Call | ⚪ (Part 31 has no crate either — joint gap) |
| 14 | Presence / Typing / Receipts / Status | ⚪ (Part 30 has no crate either — joint gap) |
| **15** | **Security Center / Devices / Keys / Recovery** | 🟡 **§3-90, §96-100, §110-141, §144, §148-183, §186-194 (reconciled) done, ~83% of 221 total sections** — everything from prior rounds, plus now `RevocationCapabilities`/`sign_out_copy` (§179 — "this signs the device out" only ever renders when actually true), the fixed revocation-can't-erase-remote-copies disclaimer (§180-181), `RecoveryScope` and its history caveat (§182-183 — silence about unrestorable history would itself be the overclaim these sections warn against). **Real spec-internal inconsistency found and documented, not silently resolved**: §184's compromise-response checklist has a different order and two extra steps versus §38's (built in round 7) — extended the existing `CompromiseResponseStep` enum with §184's two genuinely new steps (`ReVerifyAffectedContacts`, `CreateFreshBackup`) rather than reordering the original five, since earlier rounds' tests already depend on that order. §186-194 closed via reconciliation, no new code — event correlation is explicitly deferred by the spec itself as "advanced future feature"; retention/search/Alerts-vs-Events are policy notes; the four cross-crate integration points (§190-193) were already satisfied by design (`RecoveryStatus`/`BackupSecurityState` reused not duplicated since round 11; `DeviceLinked`/`VerificationFailed`/`IdentityChanged` event kinds already exist) or point to still-unbuilt Parts (07 calls, 12 linking is partially built elsewhere, 13 notifications). |
| 16 | Backup / Restore / Export / Migration | ⚪ (Part 33 has no crate either — joint gap) |
| 17 | Emergency SOS / Offline Mesh | ⚪ (siar-emergency crate pre-exists, unreconciled) |
| 18 | Settings / Privacy / Notifications / Data Controls | ⚪ |
| 19 | Plugin/Module Ecosystem | ⚪ (Part 24 has no crate either — joint gap) |
| 20 | Diagnostics / Network Paths / Advanced Dev | ⚪ (Part 18 has no crate either — joint gap) |
| 21 | Accessibility | ⚪ |
| 22 | Design System (tokens/typography/icons/motion) | ⚪ — **candidate for next priority: almost everything else in this tier visually depends on it existing first** |
| 23 | Responsive/Adaptive Layout | ⚪ |
| 24 | Error/Loading/Empty/Offline/Degraded States | ⚪ |
| 25 | Onboarding / First-Run / Permissions | ⚪ |
| 26 | Performance / Virtualization / Large-Data UI | ⚪ |
| 27 | UI Testing / Screenshot / Release Quality Gates | ⚪ |

---

## Tier 3 — The verification-boundary problem — **STALE as of 2026-09-01, see below**

**Original text, kept for history**: This isn't a spec, it's a standing
constraint on everything in Tier 2. `apps/desktop` transitively depends
on `siar-messaging`/`siar-transport`, workspace-pinned to
`rust-version = "1.91"` (the `iroh`/`stoolap` floor). The verification
sandbox used for every crate in Tier 0/1 only has rustc 1.75.0, with no
network path to a newer toolchain. `apps/android` needs a full
Android/Gradle/JNI toolchain not present at all.

**Correction**: `apt-cache search "^rustc-1"` in this sandbox exposes
`rustc-1.91`/`cargo-1.91`/`rust-1.91-clippy` packages directly
(`archive.ubuntu.com` is allowlisted) — installable directly, binaries
land at `/usr/lib/rust-1.91/bin`. Combined with the right system libs
(`libgtk-3-dev libssl-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev
libwebkit2gtk-4.1-dev libasound2-dev cmake libopus-dev libdav1d-dev`),
`cargo check --workspace --all-targets` against the ORIGINAL
repo-root `Cargo.lock` (v4 format — parses fine under cargo 1.91;
don't delete/regenerate it) passes clean across all 33 crates,
including `apps/desktop` (webview/GTK/audio/AV1). So: `apps/desktop`
and `apps/android`'s Rust glue ARE now compile-verifiable here — only
real device/emulator testing (actual Kotlin/JNI runtime behavior,
hardware codecs) remains genuinely out of reach. Disk space is tight
(~6G free after the above) — expect to `rm -rf target && apt-get
clean` between big check runs.

**Practical consequence, updated**: full `cargo build`/`cargo
test`/`cargo clippy -D warnings` rigor is now available for
EVERYTHING in this workspace, not just Tier 0/1-grade crates — spec 04
`siar-event-log`'s "Phase 2 SQLite blocked on rustc 1.87" note above is
one direct casualty of this correction and should be re-evaluated
next time that crate is picked up. `apps/desktop`/`apps/android`
Rust-level code no longer needs to ship flagged-unverified by default —
only genuine device/emulator/hardware-codec behavior does.

---

## Suggested next priorities, in order

**Superseded by explicit user instruction (2026-09-01): work through the
9 Tier 0 core specs first, one by one, before returning to this list.**
Spec 01 (`siar-protocol-ext`) is complete (108/108). Spec 02
(`siar-identity-multidevice`) is now ALSO complete (204/204) as of
2026-09-05. Spec 03 (`siar-routing-policy`) is in progress, ~80/200 as
of 2026-09-08 (round 8, §91-96) — the next crate in this project's
explicit priority order ("work through the 9 Tier 0 core specs first,
one by one"), continuing with §97 onward. Note there is a real, documented unresolved
reconciliation question between `siar-routing` (pre-existing,
next.md-era) and `siar-routing-policy` (this spec's own crate) — see
that crate's own `lib.rs` for the current state of that question
before starting new work there.

Original list, resumes once Tier 0 is done:

1. `ui-ux-15` **continue in ordinary batches** (§195-221 remain): next
   natural slice continues the integration section (§195-206 — Privacy
   Settings Integration onward), then testing matrix + final scope
   (§207-221) — likely finishes the spec in 2-3 more rounds.
2. **`ui-ux-22` Design System** — tokens/typography/icons/motion that
   every other visual spec in this tier implicitly depends on; doing it
   later means retrofitting styling into everything built before it.
4. **Part 28 §46-92 in ordinary batches** — abuse resistance and
   embedded/plugin/FFI security boundaries are the two sub-areas most
   likely to matter soon given `siar-protocol-ext`'s existing extension
   mechanism (Part 01) and the still-unstarted Parts 21/24
   (third-party extensions / plugin ecosystem).
5. **Joint gaps** (spec pairs where both the core-arch and ui-ux spec
   are ⚪): 11+32 (search), 12+15 (QR/NFC pairing), 13+31
   (notifications), 14+30 (presence/receipts), 16+33 (backup), 19+24
   (plugins), 20+18 (diagnostics). Each pair is naturally one unit of
   work (backend + its UI together), not two separate efforts.
6. **Deliberately last, by explicit priority call**: plugin/module
   ecosystem (Part 24, ui-ux-19), WASM components (Part 22), third-party
   protocol extensions (Part 21), external interoperability (Part 23).
   Usability and reliability come first; extensibility/ecosystem work
   is lower priority until the core product is solid.
7. **The four Part 28 subsystems** (ratchet, groups/MLS, test harness,
   crate split) — each needs its own dedicated, scoped effort rather
   than sharing a round with anything else.
