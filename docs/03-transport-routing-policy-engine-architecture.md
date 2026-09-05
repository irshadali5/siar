# Part 03 — Transport & Routing Policy Engine Architecture

Source spec: `sys-arch/03-transport-routing-policy-engine-architecture.md` — 200 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**60/200 (30%) sections.**

Partial.

## Implementing crate(s)

- `siar-routing-policy`

- `siar-routing (pre-existing, next.md-era — unreconciled second routing/scoring system, see below)`


## Known gaps / open questions

- Unresolved-by-design reconciliation: two routing/scoring systems — `siar-routing` (next.md-era) vs `siar-routing-policy` (Part 03-era) — documented in the newer crate's own lib.rs, not silently merged.


## Note
Detail above is transcribed from ROADMAP.md and project working notes as of 2026-09-01. This file has not yet been independently re-verified line-by-line against the current source tree — treat the section fraction as the last real count, not a live measurement.
