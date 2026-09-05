# Part 04 — Offline Event Log Architecture

Source spec: `sys-arch/04-offline-event-log-architecture.md` — 95 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**10/95 (11%) sections.**

Partial, earliest-stage of the Tier-0 nine.

## Implementing crate(s)

- `siar-event-log`


## Known gaps / open questions

- Older note 'Phase 2 SQLite blocked on rustc 1.87' is STALE — the sandbox now has rustc 1.91 available (see Tier 3 / nix hermeticity notes), so this blocker should be re-evaluated next time this crate is picked up.


## Note
Detail above is transcribed from ROADMAP.md and project working notes as of 2026-09-01. This file has not yet been independently re-verified line-by-line against the current source tree — treat the section fraction as the last real count, not a live measurement.
