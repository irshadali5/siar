# Part 02 — Multi-Device Identity Architecture

Source spec: `sys-arch/02-multi-device-identity-architecture.md` — 204 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**~99/204 (49%) sections.**

**In progress.** 114/114 tests, clippy clean (`-D warnings`), zero regressions to `siar-routing-policy`/`siar-crypto` (both depend on this crate). Compiled and tested against the real workspace `Cargo.lock` with rustc 1.91.1 on 2026-09-05, not just eyeballed.

This round (2026-09-05) closed §91-99:
- §91 Offline Identity Verification — `contact_verification::OfflineVerification::bind_root_identity` binds a `RootPublicKey`, never a transport/session value, so re-verifying on every transport change is structurally unnecessary.
- §92 Contact Verification — `contact_verification::VerifiedContact` keeps `verified_root`/`current_root` as two separate fields so root-rotation awareness (`is_continuous`) is a real comparison; trust inheritance (`new_device_inherits_trust`) requires both continuity AND the `TrustAccountRoot` policy.
- §93/§94 New Device / Identity Change Notification — `contact_verification::IdentityNotification`, two variants (not one generic event with a severity field, since spec says the distinction "must be explicit"). An identity change is security-significant under every policy; a new device only under the two non-default ones.
- §95 Verification Modes — `contact_verification::VerificationPolicy`, verbatim three variants (`TrustAccountRoot`/`VerifyEveryDevice`/`HighSecurity`), `TrustAccountRoot` as `Default`.
- §96 Device Transparency Log — `contact_verification::DeviceTransparencyLog`, a boundary trait only (spec itself calls this "a future enhancement" — nothing calls `append` anywhere yet), matching `recovery::RecoveryKeyDerivation`'s existing precedent.
- §97 Self-Hosted Transparency — `contact_verification::TransparencyDeploymentMode`, a pure label with no behavior (hosting one "does not change basic identity semantics" per spec).
- §98 No Mandatory Central Directory — no new code; already true structurally (every function in this crate operates on a signed `DeviceDirectory` + `RootPublicKey` a caller already has, never a live server connection).
- §99 Directory Service Role — `contact_verification::DirectoryServiceResponse::verify_and_accept`, the only way to accept one, routes through the exact same `DeviceDirectory::verify_signature` every other path in this crate already requires — a directory service gets no weaker acceptance path than any other untrusted source.

## Implementing crate(s)

- `siar-identity-multidevice` (new module this round: `contact_verification.rs`)

## Known gaps / open questions

- §33-51, §52-70, §71-79, §80-90, §91-99 done (see crate's own `lib.rs` doc comment for the full per-section breakdown and rationale).
- §100-110, §112-204 still unmapped in detail — realistic estimate 10-14 more rounds at this project's historical pace.
- Real follow-up not yet resolved: `DeviceStatus` (3-variant, in the signed directory) vs `DeviceLifecycle` (5-variant, richer model) mismatch touches ~9 files' match arms — not retrofitted this round either.
- Three separate "trust state"-shaped types now exist workspace-wide by design (this crate's `DeviceTrustState`, `siar-ui-state`'s, `siar-crypto`'s `PeerTrustState`) — documented, not collided.
- §91's NFC method has no real proximity-transport code (same posture every crate in this series takes toward hardware/platform bindings) — `OfflineVerificationMethod::Nfc` is a value a real NFC flow would report, nothing here constructs one from an actual NFC read.

## Note

Verified directly against the current source tree and a real `cargo test`/`cargo clippy`/`cargo fmt --check` run on 2026-09-05 — not a transcription this time. Earlier rounds' totals (through §90) are still transcribed from ROADMAP.md/project notes and haven't been independently re-walked section-by-section in this pass.
