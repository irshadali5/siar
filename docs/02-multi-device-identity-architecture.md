# Part 02 — Multi-Device Identity Architecture

Source spec: `sys-arch/02-multi-device-identity-architecture.md` — 204 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**~179/204 (88%) sections.**

**In progress.** 216/216 tests, clippy clean (`-D warnings`), fmt clean, zero regressions to `siar-routing-policy`/`siar-crypto`. Compiled and tested against the real workspace `Cargo.lock` with rustc 1.91.1.

Round 9 (2026-09-05) closed §170-179:
- §170 Protocol Extension Integration — `handshake_integration::HandshakePhase`, a guarded five-phase state machine matching spec's own diagram; extension negotiation is structurally unreachable before identity verification.
- §171 Capability Negotiation Integration — `handshake_integration::AuthenticatedCapabilityAdvertisement`, the only constructor pulls capabilities from an actual signed certificate.
- §172 Routing Integration — `routing_integration::resolve_account_endpoints`, filtered to `Active` devices only.
- §173 DTN Integration — `routing_integration::dtn_opaque_identifier`, a one-way BLAKE3 hash a relay can't reverse.
- §174/§175 File/Messaging Integration — no new code; reconciliation notes on `destination.rs` showing these are already exactly its existing `Destination`/`FanOutPolicy` variants.
- §176/§177 Call Integration / Call Ring Arbitration — `call_integration::CallRingState`, spec's four named states; a second `AcceptedBy` is refused by the type itself.
- §178 Notification Integration — `notification_integration::PushToken`/`PushEndpointRegistry`, no function anywhere lets a push token substitute for identity.
- §179 Device Endpoint Privacy — `notification_integration::ScopedEndpoint`, `Ephemeral` as the no-action-needed default, visible in a public profile only when explicitly scoped that way and unexpired.

Rounds 1-8 closed §91-169 — see prior delivered tarballs / project memory for that detail.

## Implementing crate(s)

- `siar-identity-multidevice` (this round's new modules: `handshake_integration.rs`, `routing_integration.rs`, `call_integration.rs`, `notification_integration.rs`; extended `destination.rs`)

## Known gaps / open questions

- §33-51, §52-70, §71-79, §80-90, §91-99, §100-111, §112-121, §122-130, §131-139, §140-149, §150-159, §160-169, §170-179 done (see crate's own `lib.rs` doc comment for the full per-section breakdown and rationale).
- §180-204 still unmapped in detail — realistic estimate 2 more rounds to finish this spec entirely. Next natural chunk: §180-189 (Device Link/Recovery Rate Limits, UX States, New Device UX, Contact Pairing vs Device Linking, Device Name Validation, Audit Export, API Surface, Crate Split, Error Types). After that: §190-204 (No-anyhow-in-public-API, Migration Strategy, Algorithm Agility/Downgrade Protection, Root Key Backup, Backup Import, Identity Reset, Account Deletion, Organization Offboarding, Multi-Tenant Safety, Recommended Initial Implementation, Implementation Phases, Definition of Done, Relationship to Other Parts, Final Principle).
- **Real, named, unfixed gaps carried forward**: §125 schema versioning; §164 no real fuzz harness; "process death during linking" untested; `RevocationReason` not wired into the durable audit event; `storage::IdentityStore`/`transaction`/the four §131 client traits have zero real call sites anywhere in this workspace; `RootTrustCacheEntry`/`VerifiedContact` overlap not consolidated; three separate "trust state"-shaped types exist by design; §107/§91 transport-binding gaps (no real BLE/Wi-Fi/NFC wiring).

## Note

Verified directly against the current source tree and real `cargo test`/`cargo clippy`/`cargo fmt --check`/`cargo doc` runs on 2026-09-05 for all nine rounds captured here.
