# siar — Roadmap & Coverage Hierarchy

Maintained alongside the code, not a substitute for it. This file tracks
**what's written vs. what's left**, ordered by actual need — not spec
number order. Update this file every time a spec's coverage changes.

Spec source: `sys-arch/` (33 numbered core-architecture docs +
27 `ui-ux-NN` docs, uploaded once, Wiki-style). Workspace source:
33-crate Rust workspace (`siar-source`) + `apps/{cli,desktop,android,
emergency-node}` + `platform/android`.

Legend: ✅ Done/substantial · 🟡 Partial · ⚪ Not started · 🔒 Blocked on
a prerequisite · 📋 Reconciled (already covered elsewhere, no new code
needed)

---

## Tier 0 — Foundational core crates (Parts 01-09)

All nine have real, compile+test-verified coverage, tracked crate-by-
crate. Status: **substantially built, gaps are depth not breadth** —
each has a documented list of untouched later sections. See per-crate
notes in project memory (`/areas/resilient-mesh.md`) for exact section
counts; not re-duplicated here since that detail changes less than this
file's priority ordering does.

| # | Crate | State |
|---|---|---|
| 01 | siar-protocol-ext | ✅ ~22/108 sections |
| 02 | siar-identity-multidevice | ✅ ~140/204 (+ safety_fingerprint.rs this round) |
| 03 | siar-routing-policy | ✅ ~60/200 |
| 04 | siar-event-log | 🟡 ~10/95 (Phase 2 SQLite blocked on rustc 1.87 in sandbox) |
| 05 | siar-blob-manifest | ✅ ~23/210 (+ metadata_encryption.rs) |
| 06 | siar-dtn-bundle | ✅ ~50/192 |
| 07 | siar-capability | ✅ ~19/164 |
| 08 | siar-resource-limits | ✅ ~56/193 |
| 09 | siar-crash-recovery | 🟡 ~15/186 (real total corrected this session — earliest-stage of the nine) |

**Three real, unresolved reconciliation questions**, deliberately not
silently resolved — documented in the newer crate's own `lib.rs`:
- two device-cert models (`siar_crypto::device_cert` vs
  `siar-identity-multidevice`)
- two routing/scoring systems (`siar-routing` vs `siar-routing-policy`)
- two DTN bundle models (`siar-dtn` vs `siar-dtn-bundle`)

---

## Tier 1 — Security backbone (Part 28)

**127 sections total. ~46 done. Assessment: good stopping point for
incremental section-by-section work** — what's built covers the
sections that unblock everything downstream (message envelopes, replay
protection, key storage abstraction, device revocation, trust states,
domain separation, safety fingerprints, identity-change UX). What's
left is dominated by four items that are each their own subsystem, not
a next "batch":

| Item | Sections | Why it's not a batch |
|---|---|---|
| The ratchet (forward secrecy/PCR) | §11-13 | Needs a real `vodozemac` integration, not hand-rolled crypto |
| Group Security | §25-27 | Needs reconciling against `siar-crypto-mls` (831 lines, unexamined) |
| Test-vector suite + fuzzing + named attack scenarios | §93-103 | A whole security test harness |
| Workspace crate split | §112-122 | The spec's own ask: split into 10 `comm-security-*` crates — a restructuring decision, not a code batch |

Everything else already closed (§5-10, §14-24, §28-36, §40-44) or
partially reconciled (§37-39 SAS/pairing — real gap: no
`protocol_version` field on `DeviceLinkInvite`, not fixed, breaking
change to an already-shipped signed struct). Untouched and NOT
subsystem-scale (genuinely next-batch-sized whenever picked back up):
§46-92 (abuse resistance, plugin/FFI/embedded boundaries, crash-safe
ratchets, diagnostics, failure taxonomy, threat-model docs, ADRs,
disclosure process, compromise-response playbooks), §104-111
(performance, security profiles).

**Recommendation: pause Part 28 here.** Come back to §46-92/§104-111 in
ordinary batches later; treat the four subsystem items as their own
dedicated project when prioritized.

---

## Tier 2 — UI/UX (27 specs, `ui-ux-01` through `ui-ux-27`)

**Started this round. 1 of 27 specs touched (partially).** This tier
was entirely ⚪ before this session — apps/desktop and apps/android
have substantial pre-existing UI code (chat, groups, attachments) but
none of it had been reconciled against the formal 27-spec set until
now.

| # | Spec | State |
|---|---|---|
| 01 | Product Foundation / Cross-Platform Interaction | ⚪ |
| 02 | Desktop (Dioxus) App Shell / Navigation | ⚪ (apps/desktop's shell pre-dates this spec, unreconciled) |
| 03 | Android (Compose) App Shell / Navigation | ⚪ (apps/android's shell pre-dates this spec, unreconciled) |
| 04 | Conversation List / Inbox | ⚪ (siar-ui-state::conversation_list.rs pre-exists, unreconciled) |
| 05 | Message Timeline | ⚪ (siar-ui-state::timeline.rs pre-exists, unreconciled) |
| 06 | Composer / Attachments / Voice / Drafts | ⚪ (siar-ui-state::composer.rs pre-exists, unreconciled) |
| 07 | Calls / Realtime Media | ⚪ (siar-calls crate pre-exists, unreconciled) |
| 08 | Contacts / Requests / Verification / Identity | ⚪ (siar-ui-state::contact_list.rs pre-exists, unreconciled) |
| 09 | Groups / Membership / Roles | ⚪ (siar-ui-state::group_list.rs pre-exists, unreconciled) |
| 10 | Files / Media Gallery / Transfer | ⚪ |
| 11 | Search / Local Knowledge Retrieval | ⚪ (Part 32 has no crate either — joint gap) |
| 12 | Nearby / QR / NFC Pairing / Device Linking | ⚪ (Part 15 has no crate either — joint gap) |
| 13 | Notifications / Background / Incoming Call | ⚪ (Part 31 has no crate either — joint gap) |
| 14 | Presence / Typing / Receipts / Status | ⚪ (Part 30 has no crate either — joint gap) |
| **15** | **Security Center / Devices / Keys / Recovery** | 🟡 **§3-25 of 8 sections done this round** (Overview + Devices — `SecurityHealth`, `DeviceListState`, `DeviceSecurityView`, verified in `siar-ui-state`; desktop `DevicesScreen` component written, unverified — see Tier 3). Identity & Verification, Recovery, Backups, Privacy, Advanced not started. |
| 16 | Backup / Restore / Export / Migration | ⚪ (Part 33 has no crate either — joint gap) |
| 17 | Emergency SOS / Offline Mesh | ⚪ (siar-emergency crate pre-exists, unreconciled) |
| 18 | Settings / Privacy / Notifications / Data Controls | ⚪ |
| 19 | Plugin/Module Ecosystem | ⚪ (Part 24 has no crate either — joint gap) |
| 20 | Diagnostics / Network Paths / Advanced Dev | ⚪ (Part 18 has no crate either — joint gap) |
| 21 | Accessibility | ⚪ |
| 22 | Design System (tokens/typography/icons/motion) | ⚪ — **candidate for next priority: almost everything else in this tier visually depends on it existing first** |
| 23 | Responsive/Adaptive Layout | ⚪ |
| 24 | Error/Loading/Empty/Offline/Degraded States | ⚪ |
| 25 | Onboarding / First-Run / Permissions | ⚪ |
| 26 | Performance / Virtualization / Large-Data UI | ⚪ |
| 27 | UI Testing / Screenshot / Release Quality Gates | ⚪ |

---

## Tier 3 — The verification-boundary problem (applies to ALL UI work)

**This isn't a spec, it's a standing constraint on everything in Tier
2.** `apps/desktop` transitively depends on `siar-messaging`/
`siar-transport`, workspace-pinned to `rust-version = "1.91"` (the
`iroh`/`stoolap` floor). The verification sandbox used for every crate
in Tier 0/1 only has rustc 1.75.0, with no network path to a newer
toolchain. `apps/android` needs a full Android/Gradle/JNI toolchain not
present at all.

**Practical consequence**: any `siar-ui-state` view-model work
(Tier 0/1-grade verification: real `cargo build`/`cargo test`) stays at
full rigor. Any actual `apps/desktop`/`apps/android` component code is
delivered flagged-unverified, in its own separate folder, needing a
local build + error report before it's trustworthy — this has been true
since the first UI deliverable this session and remains true for
everything in Tier 2's desktop/Android columns going forward, not a
one-off caveat.

---

## Suggested next priorities, in order

1. **Finish `ui-ux-15` Identity & Verification section** — directly
   consumes `SafetyFingerprint` (Part 28 §43) and `PeerTrustState`
   (§40-41), both already built; highest-leverage next slice in Tier 2.
2. **`ui-ux-22` Design System** — tokens/typography/icons/motion that
   every other visual spec in this tier implicitly depends on; doing it
   later means retrofitting styling into everything built before it.
3. **Part 28 §46-92 in ordinary batches** — abuse resistance and
   embedded/plugin/FFI security boundaries are the two sub-areas most
   likely to matter soon given `siar-protocol-ext`'s existing extension
   mechanism (Part 01) and the still-unstarted Parts 21/24
   (third-party extensions / plugin ecosystem).
4. **Joint gaps** (spec pairs where both the core-arch and ui-ux spec
   are ⚪): 11+32 (search), 12+15 (QR/NFC pairing), 13+31
   (notifications), 14+30 (presence/receipts), 16+33 (backup), 19+24
   (plugins), 20+18 (diagnostics). Each pair is naturally one unit of
   work (backend + its UI together), not two separate efforts.
5. **The four Part 28 subsystems** (ratchet, groups/MLS, test harness,
   crate split) — deliberately last; each needs its own dedicated,
   scoped effort rather than sharing a round with anything else.
