# Part 02 — Multi-Device Identity Architecture

Source spec: `sys-arch/02-multi-device-identity-architecture.md` — 204 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**~130/204 (64%) sections.**

**In progress.** 150/150 tests, clippy clean (`-D warnings`), fmt clean, zero regressions to `siar-routing-policy`/`siar-crypto`. Compiled and tested against the real workspace `Cargo.lock` with rustc 1.91.1.

Round 4 (2026-09-05) closed §122-130:
- §122 Enterprise Device Policy — `enterprise_policy::EnterpriseDevicePolicy`, `Default` requires nothing at all ("do not make attestation mandatory for the core protocol" made structural).
- §123 Platform Attestation — `enterprise_policy::PlatformAttestation`, no field or method resembling identity anywhere on it.
- §124 Device Health Claims — `enterprise_policy::DeviceHealthClaims`, every field self-reported/unverified.
- §125 Version Compatibility — **honestly NOT fully closed**: `DeviceCertificate`/`DeviceDirectory` still carry no schema-version field; fixing that now would break already-shipped signed bytes (same category of gap already named for Part 28's `DeviceLinkInvite`). `wire_limits::SchemaVersion` exists so *new* wire types follow the convention going forward.
- §126 Serialization — no new code; a real audit (run this round) confirmed zero `usize`/`SystemTime` fields anywhere on this crate's wire types.
- §127/§128 Input Limits / Device Count Policy — `wire_limits::InputLimits`, one configurable value (not scattered constants), checked against real `DeviceDirectory`/`IdentityClaim` data before any of it is retained.
- §129 Session Cache — `session_cache::SessionCacheEntry::is_invalidated_by`, checked against a live directory rather than trusting the session's own fields.
- §130 Revocation Cache — `session_cache::RevocationCache::from_directory` is the only constructor; there's no path to insert a device id that didn't come from a real signed directory.

Rounds 1-3 closed §91-121 — see prior delivered tarballs / project memory for that detail.

## Implementing crate(s)

- `siar-identity-multidevice` (this round's new modules: `enterprise_policy.rs`, `wire_limits.rs`, `session_cache.rs`)

## Known gaps / open questions

- §33-51, §52-70, §71-79, §80-90, §91-99, §100-111, §112-121, §122-130 done (see crate's own `lib.rs` doc comment for the full per-section breakdown and rationale).
- §131-204 still unmapped in detail — realistic estimate 7-10 more rounds at this project's historical pace. Next natural chunk: §131-139 (Device Identity API, Example/Revoke/Recovery APIs, State Machines for Linking and Recovery, UI/Kotlin/iOS boundaries).
- **Real, named, unfixed gap**: §125 schema versioning is absent from `DeviceCertificate`/`DeviceDirectory` — retrofitting is a breaking change to already-shipped signed structs, not done this round or any prior one.
- `storage::IdentityStore`/`transaction`'s type-state machine still have zero real call sites (carried over from round 3).
- Three separate "trust state"-shaped types still exist workspace-wide by design (documented, not collided).
- §107/§91 transport-binding gaps remain (no real BLE/Wi-Fi/NFC wiring).

## Note

Verified directly against the current source tree and real `cargo test`/`cargo clippy`/`cargo fmt --check`/`cargo doc` runs on 2026-09-05 for all four rounds captured here.
