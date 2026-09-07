# Part 03 — Transport & Routing Policy Engine Architecture

Source spec: `sys-arch/03-transport-routing-policy-engine-architecture.md` — 200 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**74/200 (37%) sections.**

Partial — Round 1 (§1–§42) and Round 2 (§43–§56) complete (49/49 unit tests).

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
Detail above reflects implementation through Round 2 (§43–§56) completed on 2026-09-08. Next target: §57 onward.
