# Part 28 — Production Security, E2EE, Key Management, Abuse Resistance & Privacy Architecture

Source spec: `sys-arch/28-production-security-e2ee-key-management-privacy-architecture.md` — 127 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**46/127 (36%) sections.**

**Paused by deliberate call, not abandoned.** What's built covers everything that unblocks downstream work: message envelopes, replay protection, key-storage abstraction, device revocation, trust states, domain separation, safety fingerprints, identity-change UX (§5-10, §14-24, §28-36, §40-44 closed; §37-39 SAS/pairing partially reconciled — real gap: no `protocol_version` field on `DeviceLinkInvite`, not fixed because it's a breaking change to an already-shipped signed struct).

## Implementing crate(s)

- `siar-crypto`

- `siar-crypto-mls (Group Security, §25-27 — 831 lines, unexamined against spec at last check)`


## Known gaps / open questions

- Four remaining items are each their own subsystem, not a next batch: the ratchet/forward-secrecy (§11-13, needs real `vodozemac` integration, not hand-rolled crypto), Group Security (§25-27, needs reconciling against siar-crypto-mls), a full test-vector/fuzzing/attack-scenario harness (§93-103), and the spec's own ask to split into 10 `comm-security-*` crates (§112-122, a restructuring decision).

- §46-92 and §104-111 (abuse resistance, plugin/FFI/embedded boundaries, crash-safe ratchets, diagnostics, failure taxonomy, threat-model docs, ADRs, disclosure process, compromise-response playbooks, performance/security profiles) are untouched but ordinary-batch-sized whenever resumed.


## Note
Detail above is transcribed from ROADMAP.md and project working notes as of 2026-09-01. This file has not yet been independently re-verified line-by-line against the current source tree — treat the section fraction as the last real count, not a live measurement.
