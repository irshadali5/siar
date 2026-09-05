# Part 02 — Multi-Device Identity Architecture

Source spec: `sys-arch/02-multi-device-identity-architecture.md` — 204 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**~121/204 (59%) sections.**

**In progress.** 136/136 tests, clippy clean (`-D warnings`), fmt clean, zero regressions to `siar-routing-policy`/`siar-crypto`. Compiled and tested against the real workspace `Cargo.lock` with rustc 1.91.1.

Round 3 (2026-09-05) closed §112-121:
- §112 Notifications / §115 Merkle-Hash-Chain Support — no new code; already satisfied (§112 by `audit_log` + `contact_verification::IdentityNotification`'s existing split; §115 by `state_chain`'s existing `AccountStateEvent`/`StateHash`).
- §113 Cross-Device Consistency — `reconciliation::ConvergenceStatus`, a real three-way comparison (converged / local-ahead / local-behind / same-generation-different-state) rather than a boolean, so "conservative during divergence" survives as distinct information.
- §114 Reconciliation — `reconciliation::ReconciliationPlan`, a decision derived from a `ConvergenceStatus` — never itself requests, transmits, or applies anything ("decide, don't dial").
- §119 Idempotency — `reconciliation::EventDeduplicator`, keyed on an event's own BLAKE3 hash.
- §120 Replay Protection — mostly a pointer to four already-existing checks (`RollbackRejected`, the one-signed-snapshot-per-generation design, revocation/certificate-generation checks, prekey/claim expiry); `reconciliation::ReplayProtectionIndex` adds tracking of highest-seen-generation before a live directory exists.
- §116-117 Identity Storage / Storage Interface — `storage::IdentityStore`, a trait boundary with zero implementation, matching `secure_storage::SecureStore`'s own precedent exactly (including its `-> impl Future<...> + Send` method shape).
- §118 Transaction Boundaries — `transaction::CertificateVerified → EventAppended → SnapshotUpdated → Committed`, a type-state machine; `Committed::into_audit_event` is the only way to get a `DeviceLinked` audit payload out, so "never emit before durable persistence" is a compile-time property, not a comment.
- §121 Device-Specific Authorization — `device_authorization::DeviceAuthorizationDecision::combine`, a pure combinator: device capability is a hard ceiling no user/network policy can override; the tightest of any caller-supplied size limits wins.

Round 2 closed §100-111, round 1 closed §91-99 — see prior delivered tarballs / git history for that detail (also summarized in project memory).

## Implementing crate(s)

- `siar-identity-multidevice` (this round's new modules: `reconciliation.rs`, `storage.rs`, `transaction.rs`, `device_authorization.rs`)

## Known gaps / open questions

- §33-51, §52-70, §71-79, §80-90, §91-99, §100-111, §112-121 done (see crate's own `lib.rs` doc comment for the full per-section breakdown and rationale).
- §122-204 still unmapped in detail — realistic estimate 8-11 more rounds at this project's historical pace.
- Real follow-up not yet resolved: `DeviceStatus` (3-variant) vs `DeviceLifecycle` (5-variant) mismatch.
- Three separate "trust state"-shaped types still exist workspace-wide by design (documented, not collided) — `storage::IdentityStore::trust_state` deliberately returns opaque bytes rather than picking one of the three, for the same reason.
- `storage::IdentityStore` and `transaction`'s type-state machine have zero real call sites yet — nothing in this crate's device-linking flow (`device_flows.rs`) has been rewired to go through `transaction::CertificateVerified` instead of calling `audit_log::device_linked_event` directly. That rewiring is real future work, not done implicitly by this round adding the type.
- §107/§91's transport-binding gaps (carried over from earlier rounds) remain: no real BLE/Wi-Fi/NFC wiring anywhere in this crate.

## Note

Verified directly against the current source tree and real `cargo test`/`cargo clippy`/`cargo fmt --check`/`cargo doc` runs on 2026-09-05 for all three rounds captured here.
