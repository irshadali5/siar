# Part 02 — Multi-Device Identity Architecture

Source spec: `sys-arch/02-multi-device-identity-architecture.md` — 204 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**~149/204 (73%) sections.**

**In progress.** 180/180 tests, clippy clean (`-D warnings`), fmt clean, zero regressions to `siar-routing-policy`/`siar-crypto`. Compiled and tested against the real workspace `Cargo.lock` with rustc 1.91.1.

Round 6 (2026-09-05) closed §140-149:
- §140-144 (Headless Linking, File-Only/ERP/Emergency/Service-to-Service Reuse) — proved with real composition tests using only this crate's public API (`reuse_patterns.rs`, `spec_140_`-style tests matching the `spec_70_example_target` convention), not new runtime types. §142's one genuine addition: `MapsToAccount`, a trait an application's own id type implements — the SDK still defines no `EmployeeId` anywhere.
- §145 Privacy-Preserving Device Names — `device_privacy_presentation::remote_device_label`, the only function that can reveal a real friendly name, and only when called with an explicit `true`; the default path computes a generic capability-based descriptor instead.
- §146 Device Removal UX Semantics — `client_api::RevocationReason::presentation()`, exactly two distinct presentations ("Remove device" / "Revoke compromised device") over the identical underlying `revoke_device` call.
- §147 Device History Retention — `local_records::DeviceHistoryRecord` (spec's four fields, verbatim, no others) + `DeviceHistoryLog::retain_since`, a real trimming operation, not a policy comment.
- §148 Key Compromise Warnings — `session_cache::RevocationCache::authentication_attempt`, returning a real `AuthenticationOutcome` rather than a bare bool.
- §149 Root Trust Cache — `local_records::RootTrustCacheEntry`, deliberately kept SEPARATE from the already-shipped `VerifiedContact` rather than bolting a field onto it; the overlap is named, not hidden.

Rounds 1-5 closed §91-139 — see prior delivered tarballs / project memory for that detail.

## Implementing crate(s)

- `siar-identity-multidevice` (this round's new modules: `reuse_patterns.rs`, `device_privacy_presentation.rs`, `local_records.rs`; extended `client_api.rs` and `session_cache.rs`)

## Known gaps / open questions

- §33-51, §52-70, §71-79, §80-90, §91-99, §100-111, §112-121, §122-130, §131-139, §140-149 done (see crate's own `lib.rs` doc comment for the full per-section breakdown and rationale).
- §150-204 still unmapped in detail — realistic estimate 5-8 more rounds. Next natural chunk: §150-159 (Verification Methods, Device Linking Over Existing Secure Session, Device Linking Without/With Internet, Recovery Without Internet, Backup Relationship, New Device History Bootstrap, History Authorization, Device Removal and Backups, Threat Model).
- **Real, named, unfixed gaps carried forward**: §125 schema versioning still absent from `DeviceCertificate`/`DeviceDirectory`; `RevocationReason` not yet wired into the durable audit event; `storage::IdentityStore`/`transaction`'s type-state machine still have zero real call sites; `RootTrustCacheEntry`/`VerifiedContact` overlap not consolidated; three separate "trust state"-shaped types still exist workspace-wide by design; §107/§91 transport-binding gaps (no real BLE/Wi-Fi/NFC wiring); the four §131 client traits have no implementation anywhere in this workspace yet.

## Note

Verified directly against the current source tree and real `cargo test`/`cargo clippy`/`cargo fmt --check`/`cargo doc` runs on 2026-09-05 for all six rounds captured here.
