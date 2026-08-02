# Siar (v0.5.0) — Architecture

A serverless, cross-platform (Linux/macOS/Windows — Dioxus desktop renders on all
three from one codebase, plus Android via `crates/android`) P2P messenger on iroh. This document is the design
record for the v0.5.0 architecture: seed-phrase identity, unique usernames, a
request/accept contact model, compressed messages, file transfer, real-time calls, offline mesh networking, and a
Signal/WhatsApp-style UI. Calls (audio/video) are fully implemented — see §8.
Mesh networking (LAN/BLE) is also fully implemented for offline proximity messaging.

---

## 1. Goals carried over from your brief

1. Stop hardcoding name/ticket — real multi-identity, multi-device.
2. 24-word seed phrase: create, back up, and recover the *same* identity on
   any device.
3. Globally-searchable, unique usernames, backed by `iroh-docs` as a
   key→value registry (no central server).
4. Signal/Keet-style flow: search a username → send a contact request →
   the other side must **Accept** before a chat exists. Tickets still work
   as a manual/offline fallback (you asked to keep them).
5. Signal/WhatsApp-like UI (bubbles, sidebar, avatars, requests inbox).
6. Messages and files: compressed when it helps, not when it doesn't;
   streamed, not buffered, for large files.
7. Audio calls: Opus only. Video calls: codec preference AV1 → H.265 (Implemented, including Android MediaCodec support).
8. Offline Mesh: store-and-forward mesh routing over LAN and BLE for offline proximity messaging.

## 2. Identity model

```
24-word BIP39 mnemonic (256-bit entropy + checksum)
        │  bip39::Mnemonic::to_seed(passphrase="")   -> 64-byte seed
        ▼
   HKDF-SHA512(salt = None, ikm = seed)
        │  .expand(info = "siar/identity/ed25519/v1", 32 bytes)
        ▼                                   │  .expand(info = "siar/docs-author/v1", 32 bytes)
  iroh Ed25519 SecretKey (= EndpointId)      ▼
                                     iroh-docs AuthorId secret
```

* One seed → deterministic derivation of **every** key the app needs, via
  HKDF with distinct domain-separation strings. Adding a third derived key
  later (e.g. a backup-encryption key) costs one new `info` string, not a
  new secret to protect.
* **The seed phrase itself is never written to disk.** It is shown once at
  creation (and re-derivable/re-enterable at recovery time) and the user is
  responsible for writing it down — the same trust model every crypto
  wallet uses, and the only one consistent with "no accounts, no server."
  What *is* persisted locally (0600-permissioned, like today) is the
  derived 32-byte Ed25519 secret, exactly as `identity.rs` already does —
  so day-to-day launches never touch the mnemonic again.
* Recovering on a new device = re-entering the 24 words. HKDF is
  deterministic, so the same `EndpointId` and the same doc `AuthorId` pop
  back out, and `iroh-docs` sync then pulls your username claim and
  contact list back down from any peer still holding your namespace data.

**Multi-device note:** because the identity key is derived and portable,
the *same person* can run the app on two machines with two different
`EndpointId`s only if you choose to (e.g. per-device sub-keys) — v0.5.0 keeps
it simple: one seed = one `EndpointId`, usable on one active device at a
time (like a wallet, not like multi-device Signal's separate linked-device
protocol). Multi-device-simultaneous is a real follow-on project (Signal's
linked-device design is its own protocol); flagged in §9.

## 3. Unique usernames — `iroh-docs` registry, and its honest limits

`iroh-docs` gives you a replicated, range-based-set-reconciled key→value
store (`Replica`/`Entry`, keyed by `(namespace, author, key)`, synced over
`iroh-gossip` + content fetched via `iroh-blobs`). There is **no built-in
global consensus or write-locking** — that's a deliberate part of its
design (it's CRDT-like, not a blockchain).

**Design:**
* One well-known `NamespaceId` is baked into the binary — the "phonebook"
  every instance joins on startup (namespace *secret* is embedded too,
  since anyone must be able to write their own claim; the namespace does
  not gate who can write, our application logic does).
* Every user writes their claim under **their own `AuthorId`** (derived
  above) at key `user/<username-lowercase>` → postcard value
  `UsernameRecord { endpoint_id, ticket, claimed_at_unix_ms }`.
* **Claiming** = query all entries at that key (across authors) that are
  currently visible → if none, or all belong to *you*, write your own
  entry. If one exists from another author, the name is taken.
* **Conflict resolution for the race window** (two people claim the same
  name before either has synced with the other): the record with the
  *earlier* `claimed_at_unix_ms` wins once both entries eventually meet.
  The loser is told "this name was just taken, pick another" the next time
  the app syncs and notices two authors on one key.

**This is optimistic, eventually-consistent uniqueness — not
cryptographically guaranteed uniqueness.** It's the same category of
trade-off Secure Scuttlebutt / early Hypercore social apps make, and it's
an honest, well-understood limitation of doing this without a server or a
chain. It's good enough for "search my friend's `@handle`", not good
enough for "sell premium handles" or adjudicate disputes. If you ever want
harder guarantees, the natural upgrade (and it fits your existing interest
in a feeless DAG chain) is a small ordered-log/consensus layer purely for
name claims — everything else in this doc stays as-is. Noted as a v3 idea,
not built now.

* **Discovery mechanics:** the registry doc is joined at startup like any
  other iroh-gossip topic; entries propagate to peers as they connect.
  Search-by-username in the UI queries the *locally synced* view first
  (instant if you've been online a while) and can optionally dial a couple
  of long-lived "rendezvous" peers to pull fresher state on demand.

## 4. Contact flow (Keet/WhatsApp-style request → accept)

New ALPN `siar/contact/1`, independent of the DM ALPN so an
un-accepted stranger can reach you *only* with a request, never with
arbitrary chat traffic.

```
A searches "bob" in registry  →  gets Bob's EndpointId + ticket
A dials Bob on contact ALPN, sends:
    ContactMsg::Request { from_id, from_username, from_name, note }
Bob's handler stores it as a *pending inbound* row, surfaces AppEvent
Bob's UI shows "bob123 wants to connect — [Accept] [Decline]"

If Accept:
    Bob dials A back on contact ALPN: ContactMsg::Accept { .. }
    Both sides upsert `contacts` row → state = Accepted
    Only *now* does the DM ALPN session get opened / messages allowed

If Decline:
    ContactMsg::Reject is sent once, A's UI marks the request Declined
    No contact row is created; A cannot retry-spam without the UI
    surfacing it as a repeat request
```

Local state machine per contact (`store.rs`):
`None → PendingOut ⇄ PendingIn → Accepted | Blocked`.
`Blocked` contacts are dropped at the protocol handler (connection
accepted, request read, silently discarded) — no notification leak.

Tickets are kept exactly as today (`mtkt1...` bare `EndpointId`, base32) as
the **manual/offline fallback path** — e.g. pasting a ticket over
Signal/email when you don't know the person's username yet, or the
registry hasn't synced. Pasting a ticket goes through the *same*
request/accept flow, just skipping the username search step.

## 5. Wire format & compression

`Envelope` (postcard-encoded, as today) gets one framing byte prepended at
the transport-encode boundary, not inside the struct itself, so it applies
uniformly to every body kind:

```
[ 1 byte codec tag ][ payload ]
  0x00 = raw postcard bytes
  0x01 = zstd-compressed postcard bytes
```

`encode()` postcard-serializes as before, then tries `zstd::encode_all`
at a fast level (3): if the compressed form is smaller, ship it tagged
`0x01`; otherwise ship the raw bytes tagged `0x00`. This makes compression
**adaptive and free when it doesn't help** — a 6-character "hi" never gets
zstd-header overhead, a long pasted paragraph or JSON blob does. `decode()`
branches on the tag. One code path, every message kind benefits, no
per-call-site decisions to get wrong.

**Files** go through `iroh-blobs`, not the DM envelope, because blobs give
you BLAKE3-verified streaming, resumable/range fetches, and content
dedup — reimplementing that over raw QUIC streams would be strictly worse.
The DM envelope only ever carries a small `Body::File{ .. }`
*announcement* (name, mime, size, blake3 hash, whether the blob is itself
zstd-compressed) — never the file bytes.

Before adding a file to the blob store, its mime/extension is checked
against a short "already-compressed" table (jpg/png/webp/gif, mp4/mkv/webm,
mp3/opus/aac, zip/gz/zst/7z/rar, pdf) and zstd is **skipped** for those —
spending CPU to shrink an already-entropy-dense file is pure waste and
often makes it slightly bigger. Everything else (text docs, source code,
uncompressed audio/images, logs, csv) is zstd-compressed before being
handed to the blob store, and decompressed by the receiver after the
BLAKE3-verified download completes. This is the same adaptive-by-content
approach as the message envelope, just applied to bytes instead of a
struct.

## 5.1 Offline Mesh (BLE & LAN)

Store-and-forward message relay over Bluetooth LE (`ble`) and local Wi-Fi (`lan`). Used when the public relay/discovery path is down.
* **Delivery model:** Flooded mesh with TTL (Time-To-Live). No routing table. Every node re-broadcasts unseen envelopes and decrements TTL.
* **Payload:** Carries the same `Envelope` as the QUIC DM path.
* **Platform Support:** Desktop supports both LAN (UDP broadcast) and BLE (btleplug). Android supports LAN, while BLE is pending JNI bindings.

## 6. Storage (sqlite, unchanged engine, extended schema)

```
identity:      (nothing — the derived secret key lives in identity.key,
                the mnemonic is never persisted, see §2)

contacts:      endpoint_id PK, username NULL, alias,
               state (pending_out|pending_in|accepted|blocked),
               requested_at, accepted_at

messages:      ...existing columns... + kind ('text'|'file'),
               file_name, file_hash, file_size, file_compressed

settings:      key/value — theme, notify, "seed_backup_confirmed" flag,
               claimed username cache
```

**Storage engine choice stays `rusqlite`, deliberately, not `redb`.** This
document's technology table above and the standing project spec both
default to `redb` for local persistence; v0.5.0 uses `rusqlite` (bundled
sqlite) instead, per explicit direction. `rusqlite`'s SQL query surface
(the `WHERE`/`ORDER BY`/`GROUP BY` composition `store.rs` already leans on
for `list_contacts_where`, `recent_messages`, `distinct_rooms`) is a better
fit for a chat app's actual access patterns — filter-by-conversation,
paginate-by-timestamp, group-by-room — than `redb`'s pure key/value model,
which would push that query logic into hand-rolled Rust indexing instead.
`rusqlite`'s `bundled` feature compiles its own sqlite for every target
(desktop + Android + iOS, see §10), so the "no C dependency to wrangle
cross-platform" argument for `redb` doesn't actually hold here. This
applies to **chat messages and file-transfer records only** — see §11 for
where `iroh-docs` fits (it isn't a `redb`/`rusqlite` alternative; it's a
different kind of store for a different kind of data, namely conversation
metadata that must survive syncing while offline). §16 below is the ADR
for what changed on top of this section: encryption at rest, a per-message
content hash, and a handful of new local-only tables.

## 7. File/module layout

```
src/
  main.rs                 launch: load config, hand straight to Dioxus;
                           no more CLI --name (identity owns the name now)
  config.rs                CLI args (data-dir, relay-timeout, notify) only
  identity/
    mod.rs                 load/persist derived secret key (as before)
    seed.rs                 BIP39 generate/parse/validate + HKDF derivation
  ticket.rs                 bare-EndpointId ticket, unchanged format
  store.rs                  sqlite schema v2 (contacts state machine, files)
  protocol/
    mod.rs
    message.rs               Envelope + adaptive compression framing
    dm.rs                    1:1 messaging (unchanged transport, same ALPN)
  gossip/
    mod.rs
    room.rs                  group rooms (unchanged)
  net/
    mod.rs
    registry.rs              iroh-docs username registry (claim/search/watch)
    conv_docs.rs             iroh-docs conversation metadata: room title/
                             membership, DM shared settings (§11) — never
                             message content, which stays in store.rs
    contacts.rs              contact request/accept ALPN + state transitions
    transfer.rs               iroh-blobs file send/receive + adaptive zstd
    calls/
      mod.rs                  ALPN + signaling *design* only (§8) — not wired in yet
  app.rs                    ties endpoint+gossip+docs+blobs+router together
  ui/
    mod.rs                   root component, routes onboarding vs main shell
    theme.rs                 CSS tokens (Signal/WhatsApp-style palette)
    onboarding.rs             create/recover seed, claim username
    sidebar.rs                 chat list + search-by-username + requests badge
    chat.rs                    message bubbles, composer, file bubble
    requests.rs                 incoming/outgoing contact request inbox
```

## 8. Calls — Fully Implemented

* **New ALPN** `iroh-messenger/call/3`, opened only between two `Accepted`
  contacts (reuses the contact trust boundary — no cold-calling strangers).
  Video operates on a separate connection using `iroh-messenger/video/2` to avoid `accept_uni()` races.
* **Audio: Opus only.** `audiopus`/`libopus` bindings + `cpal` for
  cross-platform capture/playback.
* **Video: Asymmetric Codec Negotiation.**
  Each side independently selects its outgoing codec based on its encode capabilities vs the peer's decode capabilities.
  * Desktop: AV1 (`rav1e`/`dav1d`).
  * Android: H.264/H.265/AV1 negotiated via native `AMediaCodec` capability probes.
* Media rides QUIC streams (currently uni streams, ordered) rather than datagrams, with datagrams flagged as a future optimization for lossy links.

## 10. Cross-platform: desktop and phone from one codebase

Dioxus's `desktop` and `mobile` renderer features both wrap the same
`wry`-based webview shell, so `ui/` needs no fork for phone support — the
same components run via `cargo run`/`dx serve` on Linux/macOS/Windows and
via `dx build --platform android`/`--platform ios` for phone. What
actually changes across that boundary:

* **Layout.** A fixed two-pane sidebar+chat view doesn't fit a phone
  screen. `ui/theme.rs` adds a `@media (max-width: 760px)` breakpoint that
  shows one pane at a time (sidebar list, or the open chat with a back
  button) — the same WhatsApp/Signal single-pane-with-back-navigation
  pattern their phone apps use, driven by a `chat-open` CSS class toggled
  off `active.is_some()` rather than any platform detection.
* **Native integrations on phone.** `rfd` (desktop file dialogs) and
  `notify-rust` (desktop OS notifications) have no
  Android/iOS backends. They're scoped to desktop targets only in
  `Cargo.toml` (`[target.'cfg(not(any(target_os = "android", target_os =
  "ios")))'.dependencies]`), and the one place that currently calls one of
  them (`rfd` for attaching a file) is `#[cfg]`-gated. Clipboard copy uses
  the Android/iOS WebView's system clipboard with a DOM selection fallback;
  QR camera/gallery import uses native file-input capture.
* **Everything else is untouched.** iroh's `Endpoint` is plain QUIC over
  UDP/rustls — no platform-specific socket API — so identity, DMs, rooms,
  the registry, contacts, and file transfer all run unmodified on phone.
  `rusqlite`'s bundled sqlite compiles for Android/iOS targets same as
  desktop.

**What this pass does *not* solve, honestly:**
* A phone-native file picker (Android's Storage Access Framework intent,
  iOS's `UIDocumentPickerViewController`) to replace `rfd` on mobile.
* A phone-native notification path (Android's `NotificationManager` via
  JNI, iOS's `UNUserNotificationCenter`) to replace `notify-rust`.
* **Background execution on Android.** This has been partially addressed for Android using Kotlin-based foreground services (`CallForegroundService.kt` and `RelayForegroundService.kt`) to keep audio/video streams alive and network listeners active when the app is backgrounded. iOS still lacks push-triggered wakes.
* Real device/emulator testing — none of the above was run on actual
  mobile hardware in this pass; `dx build --platform android/ios` also has
  open upstream rough edges as of this writing (bundling failures have
  been reported on some host setups), independent of anything in this app.

## 9. Explicit non-goals / honest limitations

* Username uniqueness is optimistic/eventually-consistent, not guaranteed
  (see §3) — by design, not oversight.
* One seed = one active `EndpointId` at a time; true simultaneous
  multi-device (à la Signal linked devices) is a separate protocol, not
  in this pass.
* Calls and Mesh networking are now fully implemented.
* The registry namespace secret is embedded in the binary (so anyone can
  write their own claim) — this is intentional (no server = no gatekeeper
  to hold that secret exclusively), not a leaked credential.

## 11. ADR: conversation metadata over `iroh-docs`, message content stays in sqlite

**Context.** Until this pass, `iroh-docs` was wired up for exactly one
thing — the username registry (§3) — and rooms had no durable metadata at
all: a room was just a name, a gossip topic derived from that name, and
whatever local sqlite rows happened to accumulate as messages arrived.
That means membership and title existed only implicitly (whoever's sent a
message you've seen), and — per the standing project spec's §6.2 — a
gossip-only room silently loses anything broadcast while a member was
offline, with no catch-up mechanism. 1:1 DMs had the analogous gap: no
synced place for a shared nickname, pin/archive state, or a disappearing-
message TTL that both sides agree on.

**Options considered.**

1. Put the *entire* durable message log for DMs and rooms into `iroh-docs`
   (the shape the spec's §6.2 sketches: "an iroh-docs namespace per room
   for the durable, syncable message log"), with sqlite demoted to a local
   read cache.
2. Do nothing — leave rooms/DMs as gossip/direct-connection-only, accept
   the offline-message-loss and no-durable-metadata gaps as known limits.
3. **Chosen:** keep sqlite (`rusqlite`, see §6) as the sole store for
   message and file content, and add `iroh-docs`-backed conversation
   *metadata* only — room title/membership, DM shared settings — via the
   new `net::conv_docs` module.

**Decision.** Option 3, per explicit direction: `rusqlite` for chat
messages and files, `iroh-docs` for DM and group metadata.

**Consequences.**
* Every message write stays a single local sqlite insert — no doc-merge
  overhead, no growing-forever CRDT history to hold on every device, no
  change to the message-ordering approach already in place
  (`(sent_at_ms, sender, seq)`-style sort, spec §7). This keeps v0.5.0's
  existing performance characteristics (§12's budgets) intact rather than
  introducing a new write path on the hot "send a message" flow.
* What's *actually* durable-and-offline-syncing now is the thing that
  benefits most from it: a member added or removed from a room converges
  for everyone once they're back online, instead of only being visible to
  whoever was connected at the moment it happened. A DM's shared nickname/
  pin/archive/TTL settings likewise converge without either side needing
  to be online at the same time.
* **Honest gap this does *not* close:** messages broadcast over
  `iroh-gossip` to a room while a member is offline are still not
  recovered by this change — closing that fully would mean option 1 (the
  message log itself in `iroh-docs`), which was explicitly not chosen.
  Rooms are more durable than before (membership/title survive being
  offline) but "catch up on messages I missed" for rooms remains a real
  limitation, same category as the registry's optimistic-uniqueness
  trade-off in §3 — worth stating plainly rather than overselling in UI
  copy, per spec §14.2's "state explicitly whether/when it's implemented."
  1:1 DMs already reconnect and exchange history directly once both peers
  are online (`protocol::dm`), so this gap is specifically a rooms-while-
  offline gap, not a DMs gap.
* Namespace secrets for both room-metadata and DM-metadata docs are
  *derived*, not ticket-exchanged (`ticket::namespace_secret_for_room`/
  `namespace_secret_for_dm`) — same reasoning as the registry's baked-in
  namespace: there's no server to hand out a ticket from, and requiring
  one would add a coordination step neither a brand-new room nor an
  already-contact-accepted DM actually needs.
* `RoomDoc::set_title`/`remove_member` have no namespace-level access
  control (anyone who knows the room name can write); "only the admin can
  rename/remove" is a UI-enforced convention, not a cryptographic
  guarantee. Flagged here so it's never implied otherwise in-app — same
  spirit as §3 and §9's other honesty notes.

**UI wiring.** The chat header (both DMs and rooms) has an "ⓘ" button
(`ui::chat::ChatPane`'s `on_open_info`) opening `ui::ConvInfoPanel`: for a
room, title (editable) + member list with a per-member "Remove" button
shown only when the viewer is the recorded `admin`; for a DM, an editable
shared nickname, Pin/Archive toggles, and a disappearing-messages TTL
picker (Off/1h/1d/1w). `ui::spawn_open_conv_info` and its
`spawn_set_room_title`/`spawn_remove_room_member`/`spawn_set_dm_*`
siblings are the only UI call sites for `App::room_meta`, `room_members`,
`set_room_title`, `remove_room_member`, `dm_settings`, and `set_dm_*` —
see that function's doc comment for why it's safe for them to hold
`ui.core`'s write-lock across the doc-query `.await` (short version: the
network-bound first-open always happens earlier, through the decoupled
`ensure_room_metadata_standalone`/`DmDoc::open` path, before any of these
run). Pin/Archive/TTL currently only affect the settings record itself —
sorting pinned DMs to the top of the sidebar and hiding archived ones from
the main list is a natural next step, not yet wired into
`ui::build_chat_list`.

**Update — sidebar wiring, boot diagnostics, and a real persistence bug
fixed.** A follow-up pass did the "next step" above and more:

* `build_chat_list` now sorts pinned DMs to the top and filters archived
  ones out of the main list entirely; `ui::sidebar::Sidebar` gained an
  "Archived (N)" footer toggle that's the only way back to an archived
  chat (needed, since its `ConvInfoPanel` "Unarchive" button is otherwise
  unreachable once hidden). `ui.dm_settings_cache` keeps every accepted
  contact's `DmSettings` warm (via `spawn_preload_dm_settings`, called
  after boot and after accepting a contact) so this reflects reality from
  the first render, not only after opening each DM's info panel once.
* **A real bug, not just a warning:** `App::record_incoming_dm` /
  `record_incoming_room` existed but were never called —
  `handle_app_event` only ever updated the in-memory `ui.conversations`
  cache. Received messages displayed fine in the moment but were silently
  gone on restart (outgoing messages were fine — `log_outgoing_dm`/`_room`
  were already wired). Both are now called before the in-memory update.
* Room presence (`RoomEvent::NeighborUp`/`NeighborDown`/`Lagged`) was
  previously discarded (`RoomEvent(_) => {}`); now updates `ui.online` and
  drops a `BubbleKind::System` "so-and-so joined/left" line into that
  room's history, and `Lagged` surfaces as a toast — the honest, visible
  flag for the gossip-catch-up gap noted above.
* `AppRoot`'s `Screen::Main` branch previously always deferred to
  `MainShell`, which shows "Connecting…" for as long as `ui.core` is
  `None` — including after a boot failure, since the only failure signal
  was a toast that auto-dismisses. A boot failure now sets a persistent
  `ui.boot_error` shown as its own retryable screen, `Core::start` gained
  `tracing::info!` breadcrumbs at each major step (endpoint bind, blob
  store, `iroh-docs` engine, registry, router, relay wait) for narrowing
  down a stall via `RUST_LOG=info`, and `spawn_boot` wraps the whole
  `Core::start` call in a 45s outer timeout so a hang anywhere inside it
  surfaces as a retryable error instead of an indefinite spinner.
* `App::block_contact`/`is_accepted` are now wired into a "Block contact"
  button in the DM `ConvInfoPanel` (shown only for currently-accepted
  contacts). The redundant `App::join_room` wrapper was removed — the UI
  already does the join+metadata sequence itself via the decoupled
  `ensure_room_metadata_standalone`/`commit_room_doc` pattern, so the
  wrapper just duplicated that with no caller.
* `App::shutdown` is now called on Ctrl+C (closing the router/endpoint
  gracefully) — the desktop window's own close button isn't covered yet,
  since the exact Dioxus-desktop close-hook API for the pinned version
  wasn't confirmed and guessing at it felt worse than leaving it a known
  gap.

**Update — a real boot-blocking bug, and a proper Settings panel.**

* `identity::create_from_seed`/`load` used to also derive and persist a
  second key, `docs_author.key`, alongside `identity.key`. Nothing ever
  read it — `App::start` generates and persists its own docs-author
  identity independently via `net::registry::Registry::new` + the
  `docs_author_id` row in `store`'s settings table — but `load` still
  *required* the file to exist. A data dir with `identity.key` but no
  `docs_author.key` (as turned up in testing) therefore had
  `identity::exists() == true`, routed straight past onboarding, and then
  failed to boot every time with no way to self-heal, since the mnemonic
  needed to recreate that file is deliberately never persisted. Removed
  entirely rather than patched — there was nothing to fix a read path
  for, since nothing needed that file in the first place.
  `Seed::derive_docs_author_key` stays as tested-but-unwired API in case a
  future design wants it back.
* The titlebar's plain "@username" text became a proper avatar + name +
  gear affordance opening a tabbed **Settings** panel (`ui::SettingsPanel`):
  **Profile** (existing QR/ticket, now with a one-line explanation of what
  it's for), **Keys** (copyable `EndpointId`, plus an explicit, honest note
  that the recovery phrase was shown once at setup and is not retrievable
  — no invented "view seed phrase" feature, since the whole design point
  of never persisting the mnemonic would defeat itself if the app could
  just display it later), and **Storage** (data directory path, plus an
  on-demand — not per-render — disk-usage breakdown across the sqlite
  message store, the iroh-blobs file store, and the iroh-docs metadata
  store).

## 12. Reconciling with `p2p-messenger-architecture.md`

A separate architecture document was uploaded describing largely the same
system under different names (`state.rs`/`services/`/`storage/` instead of
this codebase's `app.rs`/`net/`/`store.rs`; `NodeId`/`ConvId` instead of
`EndpointId`/`ConvKey`) plus several capabilities this codebase didn't
have yet. Two different decisions were made about it, on purpose:

**Not done: renaming/restructuring modules to match its layout.** That
document's module names are a different *description* of essentially the
same architecture already documented above (§7's module layout), not a
different design. Renaming `net::registry` to `services::registry` (etc.)
across a codebase this size, with no compiler available in this session to
catch what the rename touches, is pure risk for zero functional benefit —
exactly the kind of change that's cheap to describe and expensive to get
subtly wrong without a build-and-test loop. Skipped.

**Done: the capabilities it named that this codebase genuinely didn't
have.**
* **Disappearing messages, for real** (previously just a `DmSettings`
  field with no enforcement). `Envelope` gained `expires_at_unix_ms`,
  computed by the sender from the conversation's synced TTL
  (`ui::send_to_active`) so every copy — sender's own echo and every
  recipient's — agrees on the identical timestamp rather than each side
  computing its own "now + ttl" and drifting. Three layers, matching the
  "give a receiver more than one chance to notice a message is stale" idea
  in the reference doc's worked example: the sqlite read path
  (`Store::recent_messages`) filters by `expires_at`, the in-memory render
  path (`ui::bubbles_for`) filters again as defense in depth, and a
  30-second periodic sweep (`ui::spawn_disappearing_sweep`,
  `Store::sweep_expired_messages`) physically deletes expired rows from
  both sqlite and the in-memory cache so they don't just accumulate
  forever. DMs only — rooms don't have a TTL setting in this design yet
  (`net::conv_docs::RoomMeta` has no such field); `App::log_outgoing_room`
  and `record_incoming_room` are explicit about that gap rather than
  silently applying a DM concept to rooms.
* **Manual contact verification** (`store::Contact::verified`, a plain
  local boolean — nothing about it travels over the wire, and accepting a
  contact request never implies it). Toggle lives in the DM tab of
  `ui::ConvInfoPanel` ("Mark verified"/"Unmark"), with a ✓ shown next to
  the name in the sidebar and explanatory copy that this is a manual
  out-of-band check (compare `EndpointId`s in person/QR/trusted channel),
  not something the app can confirm on its own — there's no CA here, same
  honesty already established for the username registry's optimistic
  uniqueness (§3).

**Explicitly deferred, not forgotten:** Ed25519 signing of gossip room
broadcasts (so a malicious relaying peer in the swarm can't forge a
`from_name`/rewrite content mid-flood — DMs already get this for free from
the mutually-authenticated QUIC connection itself, but gossip is
hop-relayed and doesn't carry that same guarantee), and a custom
sync-on-reconnect protocol for rooms ("give me everything after message id
X" — the concrete fix for the gossip-catch-up gap §11 already flags
honestly). Both are real, scoped, doable follow-ups; both are also new
protocol surface (wire format + handshake + verification logic) that
deserves its own compile-and-test loop rather than being bundled into an
already-large, still-unverified change set.

## 13. A real crash, and a reliability fix

**The "Join a room" button could crash the whole app.** `sidebar.rs` had
`on_join_room.call(room_input.read().clone())` — the `Ref` guard produced
by `.read()` isn't dropped until the end of the *enclosing statement*, and
since the whole `on_join_room.call(...)` invocation (which synchronously
reaches `spawn_join_room`, which does `ui.room_input.clone().set(...)` to
clear the field) is that same statement, the write happened while the read
guard from the same expression was still alive. Signals are backed by a
`RefCell`-like structure, so that's a conflicting-borrow panic
(`AlreadyBorrowed`) — and because it fired from inside a GTK/webkit2gtk
callback (not a context Rust can unwind through), it took the whole
process down with it, `abort()`-style, which is also why the logs showed
"Endpoint dropped without calling `Endpoint::close`" — the process died
before `App::shutdown` ever got a chance to run, not because shutdown
itself is broken.

Fixed by switching to `room_input.cloned()` (used correctly elsewhere in
this codebase, e.g. `title_input.cloned()` in `ConvInfoPanel`) —
`.cloned()` does its read-and-clone entirely inside its own function body,
so nothing escapes into the caller's statement to be held open. The rest
of the codebase was audited for the same shape (`.read().clone()` passed
directly into a call rather than bound to a `let` first) — everything
else either already used the safe `let x = sig.read().clone();` form, or
only appeared in `if`/`match` scrutinees whose bodies don't synchronously
write back to that same signal (which is safe, if easy to get wrong the
next time someone adds a case that does).

**Clipboard copy was unreliable on Linux**, independent of the crash: the
previous `copy_to_clipboard` created an `arboard::Clipboard`, called
`set_text`, and let it drop at the end of the function. X11 (and
similarly Wayland) has no OS-level clipboard store the way Windows/macOS
do — the copying app has to stay alive and answer other apps' paste
requests itself, so dropping the handle within ~100ms of writing (as
arboard's own runtime warning was pointing out) meant a paste shortly
after hitting "Copy" would often come up empty. Fixed using arboard's
documented pattern for this: `SetExtLinux`'s `.wait()`, run on its own
`std::thread::spawn` so it can block without freezing the UI, exiting on
`std::thread::spawn` so it can block without freezing the UI, exiting on
its own once something else takes clipboard ownership. Windows/macOS keep
the original short-lived-handle behavior, which is correct there.

## 14. Bounding every remaining network call

A prior pass bounded DM connect/send, room join/broadcast, and contact
request/accept/reject with explicit timeouts (`NET_TIMEOUT`, 8s each,
`protocol::dm`/`gossip::room`/`net::contacts`). Two gaps remained, both
closed here:

* **`App::start`'s own setup** (endpoint bind, `iroh-docs` engine
  spin-up, username registry sync) had no *individual* bounds — only the
  coarse 45s outer wrapper `ui::spawn_boot::BOOT_TIMEOUT` added in an
  earlier pass, which just said "didn't finish" with no indication of
  which step. This is exactly what surfaced in testing (offline startup
  stuck past the already-bounded relay wait, with no useful detail on
  *why*). Now: endpoint bind and the docs engine each get their own 15s
  timeout with a named error; the registry sync gets 10s and, on
  timeout, one retry without a cached author before genuinely failing
  (chosen because a stuck registry sync shouldn't be able to keep the
  whole app from opening — it's only needed for claiming/searching
  *usernames*, not for messaging already-known contacts or opening
  room/DM metadata). `BOOT_TIMEOUT` itself grew to 150s specifically so
  it stays a true last resort rather than racing ahead of these more
  specific inner timeouts and masking them — the old 45s value would
  actually have fired *before* a full bind+docs+registry(×2)+relay-wait
  sequence could complete on its own terms.
* **File downloads** (`net::transfer::fetch_incoming`) were the one
  remaining unbounded network call in the whole codebase — if a sender
  went offline between announcing a file and the receiver clicking
  download, it would hang forever with no feedback. Now wrapped in a
  120s timeout (longer than the 8s used for plain messages, since a real
  transfer can legitimately take a while) that surfaces as the file
  bubble's existing `FileState::Failed` + Retry affordance — no new UI
  needed, since that path already existed for other download failures.

## 15. "Contact request connect timed out" with a good connection: discovery lag, not relay

Reported: a contact request timing out even with a valid ticket and a
working internet connection (`relay: ok` showing the whole time). Worth
recording precisely because the obvious-looking fix — self-hosting a
relay/discovery server — wouldn't actually have addressed it.

**Relay and discovery are two different services**, and `relay_ok`
(`App::relay_ok`, the pill in the titlebar) only reflects the first one:
whether *this* endpoint has a home relay to fall back on for relayed
connections. Discovery is the separate mechanism that answers "given this
bare `EndpointId` (which is all a `ticket::encode`d ticket or a username
registry search result carries — see `ticket.rs`'s doc comment on that
choice), what address is this peer even reachable at right now?" A peer
has to *publish* its current address to the discovery service before
anyone else can look it up, and that publish isn't instantaneous —
especially right after the peer's own app just started. `relay: ok` being
green the whole time is completely consistent with discovery still not
having caught up for the *other* side.

`net::contacts::send_one`'s single `endpoint.connect(...)` attempt was
already correctly bounded (`NET_TIMEOUT`, 8s) — that's not the bug. The
bug was giving discovery lag exactly one 8-second window to resolve in,
with nothing to fall back on. Fixed by wrapping contact request/accept in
the same retry-with-backoff shape `connect_with_retry`/
`join_room_with_retry` already used successfully for DM connect and room
join (`app::request_contact_with_retry`/`accept_contact_with_retry`: 4
attempts, 500ms→1s→2s→4s backoff between them, ~40s worst case) rather
than adding a new mechanism — this failure mode isn't unique to contact
requests, it's inherent to *any* first connection to a peer resolved
purely by `EndpointId`, and the existing pattern already handles it well
elsewhere.

**Not done, and why:** embedding a resolved `NodeAddr` (relay URL + known
direct addresses) directly in the ticket, so a *freshly generated* ticket
could skip discovery lag entirely for that specific connection. This
would be a genuinely stronger fix for the common "I just generated this
ticket and I'm handing it to someone right now" case — but it needs
confirming the exact current-iroh method for reading an endpoint's own
resolved `NodeAddr` (likely an async watcher, given `endpoint.online()`'s
own shape), which wasn't confirmable without a compiler in this session,
and this file has already had a few guessed-API round trips corrected
this way. Retry-with-backoff was chosen as the safe, proven-pattern fix
now; the `NodeAddr`-in-ticket idea is a solid follow-up once it can be
checked against `cargo doc -p iroh` directly.

## 16. ADR: encryption at rest, per-message content hash, and new local-only tables

Follow-up to a storage-layer design review (sqlite = local state,
`iroh-docs` = replicated metadata, `iroh-blobs` = content — already true
per §6/§11, not changed here). Three concrete changes on top of that:

**Encryption at rest.** `messenger.db` was plaintext sqlite — a stolen or
imaged disk fully exposed chat history despite every wire transfer being
end-to-end encrypted. Now opened via SQLCipher (`rusqlite`'s
`bundled-sqlcipher-vendored-openssl` feature, see `Cargo.toml`), keyed by
`identity::storage_key` — one more HKDF-SHA512 purpose fanned out from the
persisted `identity.key`, domain-separated the same way `seed::Seed`
already fans the mnemonic out into the identity key and (unused) docs
author key. Deriving from `identity.key` rather than the mnemonic itself
is a deliberate compromise: the mnemonic exists only transiently at
onboarding (never persisted, see `seed.rs`), so it isn't available to
re-derive a key from on every later boot the way `identity.key` is.
Raw-key form (`PRAGMA key = "x'<hex>'"`), not a passphrase string — skips
SQLCipher's PBKDF2 stretching, which buys nothing for an already-uniform
256-bit HKDF output. **Migration note:** an existing plaintext
`messenger.db` from before this change will fail to open with a clear
error rather than silently corrupt — it needs a one-time
`sqlite3_rekey`/`sqlcipher_export`-based migration (attach the old
plaintext db, `sqlcipher_export` into a freshly-keyed one), which isn't
implemented yet since there's no production data to migrate.

**Per-message content hash.** `messages.content_hash` (BLAKE3 hex of
`body`, computed in `Store::log_message`) — the same content-addressing
`iroh-blobs` already gives file attachments via `file_hash`, extended to
text. Deliberately **not** implemented by routing text through
`iroh-blobs`'s async blob store the way files are: every `log_message`
call site (`App::log_outgoing_dm`/`record_incoming_dm`/
`record_incoming_room` etc.) is a plain synchronous method, several called
from paths that already hold a Dioxus `Signal` write-lock — see §13's
`AlreadyBorrowedMut` history for exactly the panic class an `.await` point
introduced there would risk reintroducing, for a workload (short text)
where blob-store dedup/GC bookkeeping overhead isn't obviously worth it
per message anyway. A plain synchronous `blake3::hash` gets the integrity/
tamper-evidence property without that risk. If real blob-backed dedup for
text is wanted later, it should run as a background sweep over
already-committed rows, not inline in the send/receive path.

**New local-only tables**: `notification_state`, `ui_cache`,
`download_history`, `upload_queue`, plus an FTS5 `messages_fts` index
(contentless, kept in sync by `AFTER INSERT`/`AFTER DELETE` triggers on
`messages` — see `Store::open`/`Store::search_messages`). All additive
`CREATE TABLE IF NOT EXISTS`, schema-only for now — nothing in `app.rs`/
`ui::` writes to `notification_state`/`download_history`/`upload_queue`
yet; they exist so the notification-center, downloads panel, and
send-retry-queue features (whenever built) have a place to land without a
schema migration at that point. `search_messages` is real and callable
today, just not wired into any UI panel yet.

**Multi-device**: still out of scope (per §9) — nothing here changes
that. The local/replicated split this ADR builds on is exactly what a
future multi-device design would need (each device keeps its own
encrypted sqlite; `iroh-docs` is already the thing that'd carry
cross-device metadata), so this is compatible groundwork, not a
prerequisite that's now been done.
