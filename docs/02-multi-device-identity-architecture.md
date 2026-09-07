# Part 02 — Multi-Device Identity Architecture

Source spec: `sys-arch/02-multi-device-identity-architecture.md` — 204 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**~189/204 (93%) sections.**

**In progress.** 231/231 tests, clippy clean (`-D warnings`), fmt clean, zero regressions to `siar-routing-policy`/`siar-crypto`. Compiled and tested against the real workspace `Cargo.lock` with rustc 1.91.1.

Round 10 (2026-09-05) closed §180-189:
- §180 Device Link Rate Limits — `link_rate_limits::LinkRateLimiter`, three independently tracked counters (active invites / attempts-per-window / failed verifications).
- §181 Recovery Rate Limits — `link_rate_limits::RecoveryRateLimiter` for the infrastructure-side half; offline recovery already uses cryptographic proof, not rate limits (no new code needed there).
- §182 UX States — `platform_boundary::DeviceManagementUiState`, four separately-sourced categories rather than one tagged list.
- §183 New Device UX — `platform_boundary::NewDeviceUxStep`, its three bootstrap steps mapped onto real `LinkingState` variants.
- §184 Contact Pairing vs Device Linking — `pairing_vs_linking.rs`, a proof module (no new type) confirming the two flows share no type at all.
- §185 Device Name Validation — `wire_limits::sanitize_device_name`, strips control characters and truncates on a byte-safe char boundary.
- §186 Audit Export — `audit_export::AuditExport`, spec's exact four named contents and nothing else.
- §187/§188 API Surface / Crate Split — no new code; this crate's real module set already exceeds the suggested sketch, and the single-crate-many-modules structure already matches the recommendation.
- §189 Error Types — `error_taxonomy::SuggestedErrorCategory`, mapping spec's twelve suggested variants onto this crate's real, more granular error types, with an honest explanation of why a wholesale rename isn't being made now.

Rounds 1-9 closed §91-179 — see prior delivered tarballs / project memory for that detail.

## Implementing crate(s)

- `siar-identity-multidevice` (this round's new modules: `link_rate_limits.rs`, `pairing_vs_linking.rs`, `audit_export.rs`, `error_taxonomy.rs`; extended `platform_boundary.rs`, `wire_limits.rs`)

## Known gaps / open questions

- §33-51, §52-70, §71-79, §80-90, §91-99, §100-111, §112-121, §122-130, §131-139, §140-149, §150-159, §160-169, §170-179, §180-189 done (see crate's own `lib.rs` doc comment for the full per-section breakdown and rationale).
- §190-204 is the LAST chunk for this spec — realistic estimate 1 more round. Covers: No `anyhow` in Public Domain API, Migration Strategy, Algorithm Agility, Algorithm Downgrade Protection, Root Key Backup, Backup Import, Identity Reset, Account Deletion, Organization Offboarding, Multi-Tenant Safety, Recommended Initial Implementation, Implementation Phases, Definition of Done, Relationship to Other Architecture Parts, Final Principle.
- **Real, named, unfixed gaps carried forward**: §125 schema versioning; §164 no real fuzz harness; "process death during linking" untested; `RevocationReason` not wired into the durable audit event; §189's `IdentityError` deliberately not renamed to match spec's suggested enum (mapped instead); `storage::IdentityStore`/`transaction`/the four §131 client traits have zero real call sites anywhere in this workspace; `RootTrustCacheEntry`/`VerifiedContact` overlap not consolidated; three separate "trust state"-shaped types exist by design; §107/§91 transport-binding gaps (no real BLE/Wi-Fi/NFC wiring).

## Note

Verified directly against the current source tree and real `cargo test`/`cargo clippy`/`cargo fmt --check`/`cargo doc` runs on 2026-09-05 for all ten rounds captured here.
