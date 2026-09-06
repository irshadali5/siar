# Part 02 — Multi-Device Identity Architecture

Source spec: `sys-arch/02-multi-device-identity-architecture.md` — 204 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**~159/204 (78%) sections.**

**In progress.** 188/188 tests, clippy clean (`-D warnings`), fmt clean, zero regressions to `siar-routing-policy`/`siar-crypto`. Compiled and tested against the real workspace `Cargo.lock` with rustc 1.91.1.

Round 7 (2026-09-05) closed §150-159:
- §150 Verification Methods — `OfflineVerificationMethod` gained `OrganizationCertificate`/`TrustedDirectory`, its remaining two named methods; "record method for audit" needed no code, already true of every type carrying that field.
- §151 Device Linking Over Existing Secure Session — `linking_channel::LinkingChannel`, recorded alongside (not replacing) the bootstrap-proof method.
- §152/§153 Device Linking Without/With Internet — no new code; proved with real tests that certificate/directory signing has zero network dependency either way.
- §154 Recovery Without Internet — reconciliation note on `recovery.rs`; every recovery function was already a pure local computation.
- §155 Backup Relationship — `identity_backup::IdentityBackup`, spec's exact three named contents and nothing else; uses `DerivedRecoveryKey` (the one thing allowed to leave the device), never the raw `RecoverySecret`.
- §156/§157 New Device History Bootstrap / History Authorization — reconciliation notes on `DeviceDirectory::is_device_trusted`; "this device is authorized to participate" IS that function's return value, and history/sync policy stops exactly there.
- §158 Device Removal and Backups — `client_api::RemovalPresentation` gained a required `backup_caveat` field; a UI built on this type cannot omit the "this doesn't erase your backups" disclaimer.
- §159 Threat Model — `threat_model::ThreatCategory`, all eleven named threats mapped to the mitigating module/function, two backed by end-to-end tests exercising the real mitigation.

Rounds 1-6 closed §91-149 — see prior delivered tarballs / project memory for that detail.

## Implementing crate(s)

- `siar-identity-multidevice` (this round's new modules: `linking_channel.rs`, `identity_backup.rs`, `threat_model.rs`; extended `contact_verification.rs`, `recovery.rs`, `directory.rs`, `client_api.rs`)

## Known gaps / open questions

- §33-51, §52-70, §71-79, §80-90, §91-99, §100-111, §112-121, §122-130, §131-139, §140-149, §150-159 done (see crate's own `lib.rs` doc comment for the full per-section breakdown and rationale).
- §160-204 still unmapped in detail — realistic estimate 4 more rounds. Next natural chunk: §160-169 (Trust Assumptions, Security Invariants, Testing Strategy, Property Tests, Fuzz Targets, Simulated Multi-Device Tests, Disaster Test, Performance Goals, Cache Strategy, Device Directory Size). After that: §170-179 (integration with other Parts), §180-189 (rate limits/UX/audit/API surface), §190-199 (error types/migration/algorithm agility/account lifecycle), §200-204 (recommended implementation, phases, definition of done, final principle).
- **Real, named, unfixed gaps carried forward**: §125 schema versioning still absent from `DeviceCertificate`/`DeviceDirectory`; `RevocationReason` not yet wired into the durable audit event; `storage::IdentityStore`/`transaction`'s type-state machine and the four §131 client traits still have zero real call sites anywhere in this workspace; `RootTrustCacheEntry`/`VerifiedContact` overlap not consolidated; three separate "trust state"-shaped types still exist workspace-wide by design; §107/§91 transport-binding gaps (no real BLE/Wi-Fi/NFC wiring).

## Note

Verified directly against the current source tree and real `cargo test`/`cargo clippy`/`cargo fmt --check`/`cargo doc` runs on 2026-09-05 for all seven rounds captured here.
