# Part 06 — DTN / Store-Carry-Forward Architecture

Source spec: `sys-arch/06-dtn-store-carry-forward-architecture.md` — 192 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**50/192 (26%) sections.**

Partial.

## Implementing crate(s)

- `siar-dtn-bundle`

- `siar-dtn (pre-existing, next.md-era, sync — unreconciled second DTN bundle model, see below)`


## Known gaps / open questions

- Unresolved-by-design reconciliation: two DTN bundle models — `siar-dtn` (sync, next.md-era) vs `siar-dtn-bundle` (async, Part 06-era).


## Note
Detail above is transcribed from ROADMAP.md and project working notes as of 2026-09-01. This file has not yet been independently re-verified line-by-line against the current source tree — treat the section fraction as the last real count, not a live measurement.
