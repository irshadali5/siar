# Part 02 — Multi-Device Identity Architecture

Source spec: `sys-arch/02-multi-device-identity-architecture.md` — 204 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**~109/204 (53%) sections.**

**In progress.** 125/125 tests, clippy clean (`-D warnings`), fmt clean, zero regressions to `siar-routing-policy`/`siar-crypto`. Compiled and tested against the real workspace `Cargo.lock` with rustc 1.91.1.

Round 2 (2026-09-05) closed §100-111:
- §100 Device State Through DTN — `state_transport::SignedDeviceStateUpdate` reuses `audit_log::IdentityAuditPayload` directly as its wire payload rather than a parallel enum; a relay that only stores/forwards opaque bytes can neither read nor forge it, verification needs only the account's root public key.
- §101 Emergency Identity — `principal_claims::PrincipalType`, a plain label (Person/Authority/Responder/AnonymousLimited) with no emergency business rules attached to any variant.
- §102 Authority Identity / §103-104 Identity Claims / Claim Type — `principal_claims::IdentityClaim` (spec's struct verbatim) + `IdentityClaim::is_valid`, which requires signature validity, non-expiry, AND caller-supplied issuer trust all at once — the "Verified Authority" gate made into one function, never decided by this crate alone.
- §105-106 Privacy / Public vs Private Device Metadata — `discovery_privacy::DeviceMetadata` (public tier) vs `PrivateDeviceMetadata` (authenticated tier) as two distinct types, not one struct with fields a caller is trusted not to read early.
- §107 Rotating Discovery Tokens — `discovery_privacy::RotatingDiscoveryToken`, opaque BLAKE3-keyed-hash bytes with no embedded `AccountId`/`DeviceId`.
- §108 Device Tracking Resistance — `discovery_privacy::TransportDiscoveryIdentity`, a marker-trait seam for a real transport crate; zero implementors defined here.
- §109 Address Book Mapping — no new type; the SDK owning no `Contact`/address-book/social-graph concept is satisfied by absence.
- §110-111 Device Audit Log / Audit Event Type — already satisfied by `audit_log.rs`'s existing five event constructors and its `IdentityAuditPayload` enum from an earlier round; no new code needed.

Round 1 (2026-09-05) closed §91-99 — offline identity verification bound to root identity, contact verification with root-rotation awareness, new-device/identity-change notifications, verification-policy enum, device-transparency-log boundary trait, directory-service response gating. See git history / prior delivered tarball for that round's detail.

## Implementing crate(s)

- `siar-identity-multidevice` (this round's new modules: `state_transport.rs`, `principal_claims.rs`, `discovery_privacy.rs`)

## Known gaps / open questions

- §33-51, §52-70, §71-79, §80-90, §91-99, §100-111 done (see crate's own `lib.rs` doc comment for the full per-section breakdown and rationale).
- §112-204 still unmapped in detail — realistic estimate 9-13 more rounds at this project's historical pace.
- Real follow-up not yet resolved: `DeviceStatus` (3-variant, in the signed directory) vs `DeviceLifecycle` (5-variant, richer model) mismatch touches ~9 files' match arms.
- Three separate "trust state"-shaped types now exist workspace-wide by design (this crate's `DeviceTrustState`, `siar-ui-state`'s, `siar-crypto`'s `PeerTrustState`) — documented, not collided.
- §107's rotating discovery tokens have no real transport wiring (same posture every crate in this series takes toward hardware/platform bindings) — nothing calls `RotatingDiscoveryToken::derive`/`resolve` from an actual BLE/Wi-Fi advertisement path yet.
- §91's NFC method still has no real proximity-transport code either (carried over from round 1).

## Note

Verified directly against the current source tree and real `cargo test`/`cargo clippy`/`cargo fmt --check`/`cargo doc` runs on 2026-09-05 for both rounds captured here. Earlier totals (through §90) are still transcribed from ROADMAP.md/project notes and haven't been independently re-walked section-by-section.
