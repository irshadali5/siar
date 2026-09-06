# Part 02 — Multi-Device Identity Architecture

Source spec: `sys-arch/02-multi-device-identity-architecture.md` — 204 numbered sections (verified via `grep -c '^# [0-9]*\.'`).

## Implementation status

**~169/204 (83%) sections.**

**In progress.** 203/203 tests, clippy clean (`-D warnings`), fmt clean, zero regressions to `siar-routing-policy`/`siar-crypto`. Compiled and tested against the real workspace `Cargo.lock` with rustc 1.91.1.

Round 8 (2026-09-05) closed §160-169:
- §160 Trust Assumptions — `trust_boundary::TrustedInput`/`UntrustedInput`, all three trusted foundations and six untrusted inputs pointed at the real enforcing code.
- §161/§163 Security Invariants / Property Tests — `security_invariants.rs`, all ten numbered invariants plus five example properties tested directly against real crate functions; invariants 2/3 get genuinely new multi-generation coverage, invariants already covered elsewhere are cited not duplicated.
- §162/§165 Testing Strategy / Simulated Multi-Device Tests — `integration_tests.rs`, a real end-to-end run of spec's own exact Alice/Bob topology and eight-step scenario; "process death during linking" is honestly named as a real, uncovered gap, not assumed fine.
- §164 Fuzz Targets — honest gap noted in `wire_limits.rs`: input bounding is real (`InputLimits`), but no actual `cargo-fuzz` harness exists in this workspace.
- §166 Disaster Test — added to `state_transport.rs`: a signed update round-trips through three simulated untrusted store-and-forward hops and remains valid.
- §167/§168 Performance Goals / Cache Strategy — `directory_cache::DirectoryCache::from_directory`, the only constructor, aggregating all five named cache categories from one directory; no incremental mutator exists at all.
- §169 Device Directory Size — `directory_cache::HandshakeSummary`, spec's exact four fields, paired with the existing `ConvergenceStatus`/`ReconciliationPlan` machinery for "request missing state only if needed."

Rounds 1-7 closed §91-159 — see prior delivered tarballs / project memory for that detail.

## Implementing crate(s)

- `siar-identity-multidevice` (this round's new modules: `trust_boundary.rs`, `security_invariants.rs`, `integration_tests.rs`, `directory_cache.rs`; extended `state_transport.rs`, `wire_limits.rs`)

## Known gaps / open questions

- §33-51, §52-70, §71-79, §80-90, §91-99, §100-111, §112-121, §122-130, §131-139, §140-149, §150-159, §160-169 done (see crate's own `lib.rs` doc comment for the full per-section breakdown and rationale).
- §170-204 still unmapped in detail — realistic estimate 3 more rounds. Next natural chunk: §170-179 (Protocol Extension/Capability Negotiation/Routing/DTN/File/Messaging/Call Integration, Call Ring Arbitration, Notification Integration, Device Endpoint Privacy). After that: §180-189 (rate limits/UX/audit/API surface), §190-199 (error types/migration/algorithm agility/account lifecycle), §200-204 (recommended implementation, phases, definition of done, final principle).
- **Real, named, unfixed gaps carried forward**: §125 schema versioning still absent from `DeviceCertificate`/`DeviceDirectory`; §164 has no real `cargo-fuzz` harness; "process death during linking" (§162) has no test; `RevocationReason` not yet wired into the durable audit event; `storage::IdentityStore`/`transaction`'s type-state machine and the four §131 client traits still have zero real call sites anywhere in this workspace; `RootTrustCacheEntry`/`VerifiedContact` overlap not consolidated; three separate "trust state"-shaped types still exist workspace-wide by design; §107/§91 transport-binding gaps (no real BLE/Wi-Fi/NFC wiring).

## Note

Verified directly against the current source tree and real `cargo test`/`cargo clippy`/`cargo fmt --check`/`cargo doc` runs on 2026-09-05 for all eight rounds captured here.
