# Part 02 — Multi-Device Identity Architecture

Source spec: `sys-arch/02-multi-device-identity-architecture.md` — 204 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**~139/204 (68%) sections.**

**In progress.** 163/163 tests, clippy clean (`-D warnings`), fmt clean, zero regressions to `siar-routing-policy`/`siar-crypto`. Compiled and tested against the real workspace `Cargo.lock` with rustc 1.91.1.

Round 5 (2026-09-05) closed §131-139:
- §131 Device Identity API — `client_api::IdentityClient`/`DeviceClient`/`TrustClient`/`RecoveryClient`, grounded in this crate's real existing functions rather than a parallel API surface (spec's own `LinkPolicy` becomes the real `LinkingAuthorityPolicy` type).
- §132 Example API — no new type; its rule ("application should not directly construct signed device certificates") is exactly why the four client traits exist.
- §133 Revoke API — `client_api::RevocationReason`, with an honest note that it is NOT yet threaded into `audit_log::IdentityAuditPayload::DeviceRevoked` (real future work, not assumed done).
- §134 Recovery API — `RecoveryClient::begin_recovery` returns the *initial* `RecoveryState`, not a finished result — "guided state machine, not one monolithic call" made structural.
- §135 State Machine for Linking — `linking_state_machine::LinkingState::advance`, guarded transitions matching `DeviceLifecycle::advance`'s own precedent; all 8 success + 4 failure states, cancellation unreachable once a certificate is issued.
- §136 State Machine for Recovery — `recovery_state_machine::RecoveryState::advance`, strictly linear 6-state chain; no invented failure states since spec names none (rejection already happens one layer up, before this machine is entered).
- §137 UI Boundary — `platform_boundary::DeviceListVm`/`DeviceLinkVm`/`SecurityIdentityVm`/`RecoveryVm`; none can carry a private key because the key-holding types are never imported into that file at all.
- §138/§139 Android Kotlin / iOS Boundary — no new code; both describe this crate's own existing scope ("Rust owns identity state/certificate logic/linking/revocation/recovery policy"), already true.

Rounds 1-4 closed §91-130 — see prior delivered tarballs / project memory for that detail.

## Implementing crate(s)

- `siar-identity-multidevice` (this round's new modules: `client_api.rs`, `linking_state_machine.rs`, `recovery_state_machine.rs`, `platform_boundary.rs`)

## Known gaps / open questions

- §33-51, §52-70, §71-79, §80-90, §91-99, §100-111, §112-121, §122-130, §131-139 done (see crate's own `lib.rs` doc comment for the full per-section breakdown and rationale).
- §140-204 still unmapped in detail — realistic estimate 6-9 more rounds. Next natural chunk: §140-149 (Headless Linking, File-Only/ERP/Emergency/Service-to-Service Reuse, Privacy-Preserving Device Names, Device Removal UX Semantics, Device History Retention, Key Compromise Warnings, Root Trust Cache).
- **Real, named, unfixed gaps carried forward**: §125 schema versioning still absent from `DeviceCertificate`/`DeviceDirectory`; `RevocationReason` (§133) not yet wired into the durable audit event; `storage::IdentityStore`/`transaction`'s type-state machine still have zero real call sites; three separate "trust state"-shaped types still exist workspace-wide by design; §107/§91 transport-binding gaps (no real BLE/Wi-Fi/NFC wiring).
- None of the four new client traits (§131) has an implementation anywhere in this workspace yet — same posture as `IdentityStore`/`SecureStore`, named as a boundary, not built out.

## Note

Verified directly against the current source tree and real `cargo test`/`cargo clippy`/`cargo fmt --check`/`cargo doc` runs on 2026-09-05 for all five rounds captured here.
