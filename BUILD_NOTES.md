# Build notes — read this before `cargo build`

Updated after a real `cargo build` run surfaced 29 errors — thank you for
that, it let me fix real API mistakes instead of guessed ones. Everything
below reflects what's now fixed vs. what's still genuinely uncertain.

## Fixed this round (grounded in your actual compiler output + confirmed docs)

- **`bip39`**: `Mnemonic::generate`/`::parse` don't exist without features.
  Now uses `Mnemonic::generate_in(Language::English, 24)` and
  `Mnemonic::parse_in(Language::English, phrase)`, with `rand` and
  `unicode-normalization` features enabled in `Cargo.toml` (those methods
  are feature-gated).
- **`iroh-docs` registry** (`src/net/registry.rs`, `src/app.rs`): rewritten
  against the confirmed current API from docs.iroh.computer:
  `Docs::persistent(path).spawn(endpoint, blobs_store, gossip)`,
  `docs.author_create()`, `docs.import(ticket)`,
  `doc.set_bytes(author, key, value)`,
  `doc.get_one(Query::single_latest_per_key().key_exact(key))`. The `Docs`
  handle is now spawned in `app.rs` (not buried inside `Registry`) so its
  ALPN can be registered on the same shared `Router` — it needs
  `.accept(iroh_docs::ALPN, docs)` just like blobs and gossip do, which
  the first draft missed entirely.
- **Deterministic docs-author dropped**: the first draft tried to derive a
  stable `AuthorId` from your seed phrase, which needs an
  import-a-raw-author-secret call I couldn't confirm exists. Simplified to:
  create a random author once, persist just the `AuthorId` in sqlite
  settings, reuse it across relaunches on *this device*. See the module
  doc at the top of `registry.rs` for why this doesn't weaken the
  uniqueness guarantee — the thing that has to be stable is the
  *endpoint_id you claim*, not who signed the claim, and endpoint_id is
  still fully seed-derived.
- **`iroh-blobs`**: `FsStore` → `iroh_blobs::api::Store` needed `.into()`
  (your compiler told me this one directly). `add_bytes(..)` returns a
  `TagInfo` with a `.hash` *field*, not a method. Reading downloaded bytes
  now goes through `store.blobs().get_bytes(hash)` and converts to
  `Vec<u8>` immediately (the previous `.as_slice()` call was hitting a
  weird unstable-feature diagnostic — sidestepped rather than chased).
- **`Store`/sqlite thread-safety**: `ContactProtocol` (an iroh
  `ProtocolHandler`) must be `Send + Sync + Debug`. `rusqlite::Connection`
  isn't `Sync`, so `Arc<Store>` wasn't either. Fixed by wrapping the
  connection in a `Mutex` inside `Store` — sqlite serializes writes anyway,
  so this doesn't cost real concurrency, it just makes the compiler's
  guarantee match what was already true. Added a manual `Debug` impl for
  `ContactProtocol` since `Store`'s internals don't derive it.
- **Dioxus prop rules**: any `struct` passed as a `#[component]` prop needs
  `Clone + PartialEq` (`ChatListEntry`, `RequestEntry`, `UiState` — added).
  A `match` arm returning a bare `Component {}` instead of `rsx! { Component
  {} }` doesn't type-check against sibling arms that return `Element` —
  fixed in the `Screen::Main` arm. An `if/else` embedded directly inside an
  interpolated `"{...}"` string in `chat.rs` doesn't parse — moved into a
  local variable computed just above.
- Three spots called `.write()` directly on a `Signal` field access
  (`ui.core.write()`) instead of `ui.core.clone().write()` — `Signal`'s
  mutation methods need the clone first when the binding itself isn't
  `mut`; fixed all three (all in the DM/room send paths).

## Still genuinely uncertain — marked `// VERIFY:` in source, check `cargo doc -p iroh-docs -p iroh-blobs --open` for your exact pinned versions

1. `REGISTRY_NAMESPACE_SECRET` in `registry.rs` is still a placeholder
   all-zero array. Generate a real one once and hardcode it, or every
   install joins its own disconnected phonebook.
2. `NamespaceSecret::from_bytes(..)` and `Capability::Write(..)` — I'm
   confident a `Capability` enum with something like `Write`/`Read`
   variants exists (mirrors `ShareMode::Write` used elsewhere in the same
   docs), but haven't seen the variant names in a runnable example.
3. `Query::single_latest_per_key().key_prefix(..)` — `key_exact` is
   confirmed from a real example; `key_prefix` is inferred by the same
   naming convention, not directly confirmed.
4. `store.downloader(&endpoint).download(hash, from)` in
   `net/transfer.rs` — `.downloader(&endpoint)` is confirmed straight from
   your compiler's own suggestion, but I couldn't confirm
   `Downloader`'s actual download-by-hash-from-peer method name/signature.
   See github.com/n0-computer/iroh-docs/issues/86 — someone else hit
   exactly this same gap and it was open, unresolved, as of this writing.
   Budget real time for this one specifically.
5. `iroh_docs::api::Doc` as the type name for an open document handle —
   inferred from the `iroh_docs::api::protocol::ShareMode` module path
   used in confirmed examples, not directly confirmed itself.
6. `AuthorId: Serialize + Deserialize` (used to persist it via postcard in
   `app.rs`) — likely, given it round-trips through doc tickets/entries
   elsewhere, but not directly confirmed.

None of these are "the design is wrong" issues — they're all "confirm the
exact spelling" issues, isolated to `registry.rs`/`transfer.rs`/the docs
wiring in `app.rs`. Everything else (identity, compression framing,
contacts protocol, sqlite schema, UI) is either unchanged from your
previously-compiling code or plain postcard/zstd/rusqlite with no iroh
version-drift risk.

## Still deliberately deferred (not bugs)

- `net/calls/` has no media pipeline — see `ARCHITECTURE.md` §8.
- `ui::mod::check_username` doesn't query the registry yet (stubbed
  `true`) — onboarding runs before a `Core`/`Registry` exists; see the
  inline `// TODO`.
- Theme toggle hardcoded to dark in `ui/mod.rs`.

## Round 3 — fixed after the second real `cargo build`

- **`registry.rs` stream `.next()` not `Unpin`**: `doc.get_many(..)`
  returns a stream that isn't `Unpin`, so `StreamExt::next()` can't be
  called on it directly. Fixed with `Box::pin(entries)` before the loop —
  this exact pattern is what someone else used for the same call in
  iroh-docs' own issue tracker (n0-computer/iroh-docs#86).
- **`transfer.rs` download**: `Downloader::download` takes `providers:
  impl ContentDiscovery`, and a bare `EndpointId` doesn't implement it —
  only things implementing `IntoIterator<Item = EndpointId>` do. Fixed by
  passing `vec![from]` instead of `from`.
- **`store.rs`**: two methods (`alias_for`, `get_setting`) had `self.conn`
  spread across multiple lines, which my earlier line-based `sed` fix
  didn't catch (it only matched `self.conn` sitting on one line) — these
  were still touching the bare `Mutex<Connection>` instead of locking it.
  Fixed both. Separately, three `.prepare(&sql)` calls held the
  `MutexGuard` only as a temporary (`self.conn().prepare(...)`), which
  compiles as "creates a temporary value which is freed while still in
  use" — a `Statement` borrows from the guard, so the guard needs a name
  that outlives it. Fixed by binding `let conn = self.conn();` first in
  all three (`list_contacts_where`, `recent_messages`, `distinct_rooms`).

## Cross-platform (desktop + phone)

Added in this round — see `ARCHITECTURE.md` §10 for the full picture.
Short version: `dioxus`'s `mobile` feature is now enabled alongside
`desktop`; `rfd`/`arboard`/`notify-rust` are scoped to desktop-only targets
since none of them have Android/iOS backends; the UI gets a
`@media (max-width: 760px)` breakpoint for phone-width single-pane
navigation. None of this was tested on real mobile hardware or an
emulator — I don't have that in this environment. To actually build for
phone you'll additionally need:

- **Android**: Android NDK + SDK installed, `rustup target add
  aarch64-linux-android` (and other ABIs as needed), then
  `dx build --platform android`. Expect to hit real upstream rough edges —
  e.g. github.com/DioxusLabs/dioxus#3762 and #3487 both report `dx build
  --platform android` bundling silently failing on some host setups as of
  this writing; if you hit that, it's a known open issue, not something
  wrong in this codebase.
- **iOS**: a Mac with Xcode, `rustup target add aarch64-apple-ios`, then
  `dx build --platform ios`. Needs an Apple developer account for a real
  device (simulator doesn't need signing).
- Background execution, push-triggered wake, and platform-native
  file-picker/notifications are **not implemented** — see ARCHITECTURE.md
  §10 for exactly what's missing and why it's nontrivial (OS-level
  background restrictions, not a library gap).

## Round 4 — two small compile fixes, plus new UI/iroh features

- `let mut entries = ...` had an unused-`mut` warning (the binding gets
  immediately shadowed by its `Box::pin`'d version) — dropped the `mut`
  from the first declaration.
- `active.is_some()` then later `if let Some(key) = active { .. }` in the
  same render function partially moved `active` (it's `Option<ConvKey>`,
  and `ConvKey` isn't `Copy`), which the borrow checker rejected even
  though the moves look sequential in source order — `rsx!`'s macro
  expansion doesn't preserve strict lexical evaluation order across a
  whole block. Fixed by cloning at the match site
  (`if let Some(key) = active.clone()`), leaving the original intact for
  the earlier `.is_some()` check.

### New features added this round

- **Delivery receipts, for real this round.** Previously `acked` was
  always `false` for anything just sent — nothing ever flipped it. Now:
  the bubble pushed on send carries the same `id` as the envelope actually
  transmitted; the receiver auto-replies with `Body::Ack(id)` on receipt
  of any `Text`/`File` message; the sender's `handle_app_event` matches
  that id back to the bubble and flips `acked`, which is what actually
  drives the ✓ / ✓✓ glyph in `chat.rs`.
- **Typing indicators.** Composer keystrokes call a rate-limited
  `maybe_send_typing` (at most one `Body::Typing` per ~3s per
  conversation); the receiving side stamps a timestamp and the chat
  header's subtitle shows "typing…" while it's fresh, auto-expiring after
  4s via a delayed task if no follow-up arrives.
- **Presence.** `DmEvent::PeerConnected`/`PeerDisconnected` now feed an
  `online: HashSet<EndpointId>` — shown as a small green dot on the
  sidebar avatar and "online"/"offline" in the chat header. Documented
  honestly in the field's doc comment: this is "we currently have an open
  session with them", not full network presence.
- **Ticket/QR profile panel.** Clicking your `@username` in the titlebar
  opens a panel with your ticket as both text and a QR code (via the
  `qrcode` crate, previously an unused dependency) and a copy button (via
  the platform clipboard; Android/iOS use the WebView's system clipboard
  API with a selection-copy fallback).
- **Room create/join UI.** There was previously no way to create or join a
  brand-new room from the UI at all — the sidebar only ever showed rooms
  already in sqlite history. Added a "join or create a room" input at the
  bottom of the Chats tab, calling the existing (already-implemented)
  `App::join_room`.

None of the above changes touch the `// VERIFY:` spots from earlier
rounds (registry/transfer's iroh-docs/iroh-blobs specifics) — they're all
in `ui/` plus the already-solid `protocol::message` ack/typing envelope
kinds that existed from the start.

## Round 5 — two more E0716s, plus real (not just cosmetic) file transfer

- Same category of bug as round 3's `store.rs` fixes: `let mut last =
  ui.last_typing_sent.clone().write();` and `let mut core_ref =
  ui.core.clone().write();` both bind a write-guard from a *temporary*
  cloned `Signal` in a plain `let` statement — the temporary Signal drops
  at the end of that statement, invalidating the guard. `if let`/`match`
  scrutinees get temporary lifetime extension in Rust (which is why the
  same-looking pattern elsewhere in this file was fine), but a bare `let`
  doesn't. Fixed both by binding the cloned `Signal` to its own name
  first, then calling `.write()` on that.
- While fixing `spawn_join_room`'s version of this, realized the original
  fix attempt would have held a `Signal` write-guard across an `.await` on
  `core.join_room(...)` — a `Send`-across-await risk that's exactly why
  every *other* action function in this file only ever clones cheap
  handles (`Endpoint`, `Gossip`, `DmSession`) out of a short-lived `read()`
  and does the actual `.await` outside any signal borrow. Rewrote
  `spawn_join_room` to follow that same established pattern instead of
  patching around the symptom.

### File transfer was cosmetic — now it's real

Caught while addressing "make file transfer robust": incoming files were
recorded as a bubble and nothing else. There was no code path that ever
called `net::transfer::fetch_incoming` — files could be *sent* but never
actually *received* onto disk. Fixed:

- File bubbles now carry real state (`Idle → Downloading → Done(path) |
  Failed(err)`), driven by an actual `Download`/`Retry` button.
- Incoming DM/room file events now record who sent it
  (`StoredContent::File::from`), so a click can actually fetch it.
- History reloaded from sqlite after a restart still knows how to
  re-fetch a DM file (the peer *is* the conversation), but is honest that
  it can't for a room file from history, since sqlite doesn't currently
  persist the original sender as a parsed `EndpointId` for room messages
  (only a display name) — that bubble shows as not-fetchable rather than
  silently failing a download attempt.
- Downloads land in `<data_dir>/downloads/<original filename>`; the path
  sanitization already in `transfer.rs` (stripping `../`, path separators)
  applies here same as before.

This still rests on the two `// VERIFY:` spots in `transfer.rs` from
round 2/3 (`store.downloader(&endpoint).download(...)`, and now also
`Hash: FromStr` for parsing the hex hash string back out of the bubble) —
the UI plumbing around it is solid, but that one network call is still
the least-confirmed piece in the whole codebase. Budget real testing time
there specifically, ideally between two actual running instances.

## Round 5 — platform scope narrowed: Linux + Windows + Android only

No source changes needed for the desktop cfg blocks in `Cargo.toml` —
`cfg(not(any(target_os = "android", target_os = "ios")))` already covers
Linux+Windows without change (macOS just goes unused, harmlessly). What
actually changed:

- **`Dioxus.toml`**: added `[bundle.android]` with `min_sdk_version = 34`
  (Android 14 — deliberate floor, drops pre-14 permission-model
  back-compat entirely) and `[bundle.android.permissions]` for the call
  feature's actual runtime needs: `RECORD_AUDIO`, `CAMERA`,
  `POST_NOTIFICATIONS` (mandatory as its own permission since Android 13,
  a manifest entry alone isn't enough), and both the generic
  `FOREGROUND_SERVICE` and the *typed* `FOREGROUND_SERVICE_MICROPHONE` /
  `FOREGROUND_SERVICE_CAMERA` — Android 14 rejects starting a
  foreground service without the typed permission matching its declared
  type, this is new as of API 34 and easy to miss.
- **iOS dropped**: `dioxus`'s `mobile` feature build still targets
  Android only going forward — nobody runs `dx build --platform ios`
  against this config anymore, so the iOS toolchain/signing requirements
  mentioned in Round 3 no longer apply. Left the `cfg(not(any(...,
  target_os = "ios")))` guards in `Cargo.toml` as-is rather than
  stripping `"ios"` out of them — it's inert (never compiled for) and
  removing it buys nothing but a smaller diff to review.

### "Kotlin shell" — what that actually means with Dioxus 0.7

There is no separate Kotlin app to write. `dx build --platform android`
generates the Gradle project *and* a `MainActivity.kt`, and Dioxus 0.7
added the ability to customize that generated `MainActivity.kt` directly
(DioxusLabs/dioxus#4294) rather than only being able to touch it through
Rust/JNI calls. That's the actual home for:

- Runtime permission request flow for `RECORD_AUDIO`/`CAMERA`/
  `POST_NOTIFICATIONS` (declaring them in `Dioxus.toml` gets them into
  the manifest; *requesting* them at runtime on first use is still
  Kotlin-side `ActivityCompat.requestPermissions` code)
- The foreground `Service` + notification channel that keeps a call
  alive when the app backgrounds — genuinely new, doesn't exist anywhere
  in this codebase yet, and is Kotlin, not Rust
- Anything else that's "the Android shell" in the sense you meant it —
  this is the file for it

Haven't located the exact generated `MainActivity.kt` template contents
myself (would need a real `dx build --platform android` run in an
environment with the Android SDK/NDK installed, which this sandbox
doesn't have) — treat the above as "this is where it goes," not a
drop-in file yet.

### One known upstream rough edge to expect, not a bug in this codebase

`dx build --android --release` has an open issue (DioxusLabs/dioxus#5251,
filed against CLI 0.7.3) where the generated Gradle config uses obsolete
Java 8 source/target compatibility, which current AGP 8.8+/Gradle 9.1+
reject outright. If a release build fails on Java-version warnings
escalating to errors, that's this — check the issue for whether it's
fixed in whatever CLI version you're actually running before assuming
something here is misconfigured.

### Still not done — the actual "make it feel native" work

Everything above is platform/build config. The UI itself
(`ui/mod.rs`, `theme.rs`) still renders like a webview app: browser-style
scrollbars, no OS theme sync, no native title bar/context menus. That's
real per-file work, not a config change, and hasn't been started —
flagging so it doesn't get lost as "already handled."

## Round 6 — keyboard shortcuts + multi-line composer

- **Multi-line composer** (`ui/chat.rs`): the message input was a single-line
  `input` where any `Enter` sent immediately — no way to compose a message
  with a line break. Now a `textarea` (`.composer-input`, capped at 140px
  before it scrolls internally): `Enter` sends, `Shift+Enter` inserts a
  newline. `.composer`'s `align-items` changed from `center` to `flex-end`
  so the attach/send buttons stay bottom-anchored as it grows.
- **Global shortcuts** (`ui/mod.rs`, on the `app-shell` wrapper div, since
  it's the one element present across every screen): `Escape` closes
  whichever overlay is topmost (story viewer → conv info → settings →
  clears an active search), `Ctrl/Cmd+K` focuses the sidebar search box,
  `Ctrl/Cmd+,` opens Settings, `Ctrl/Cmd+Shift+A` toggles the archived
  view. Deliberately nothing on a bare single key (no bare "n", "/", etc.)
  — those would fire mid-typing in the composer or a room-name field.

### Unverified — needs a real build to confirm, not assumed safe

Two APIs used here that I could not compile-check in this sandbox (no
rustc/cargo available — see earlier rounds):

1. `KeyboardEvent::modifiers()` returning a `Modifiers` with
   `.ctrl()`/`.shift()`/`.meta()` — this is the standard `dioxus_html`/
   `keyboard_types` shape as of past versions, but hasn't been checked
   against the pinned `dioxus = "0.7"` here specifically.
2. `dioxus::document::eval("...")` to focus `#sidebar-search-input` by id
   for Ctrl+K — same caveat as the `MainActivity.kt` note above: correct
   as far as documented Dioxus 0.7 usage goes, not yet run. If it no-ops,
   Ctrl+K just does nothing (no crash risk) rather than misbehaving.

**Please run `cargo check` locally before relying on this** — flagging
explicitly rather than presenting untested code as verified.

### Deliberately deferred, to keep this round reviewable

- Arrow-key navigation of the conversation list (`Alt+↑`/`Alt+↓`) — needs
  the `entries` list that's local to `MainShell`, whose `rsx!` currently
  has no single root element to attach a second keydown handler to
  without a larger restructure. Doable, just a separate, riskier change.
- Local full-text filtering of the *existing* chat list as you type in
  the search box — right now that box only ever queries the registry for
  *new* contacts (`spawn_search`); it doesn't filter what's already in
  the sidebar. Worth doing, scoped out of this round.

## Round 7 — local chat-list search

Picked this one to start on without more back-and-part-forth on the
feature-priority question, since it was already flagged as deferred in
Round 6 and is small/self-contained. `build_chat_list` now also filters
by `ui.search_query` (case-insensitive substring on the display name),
on top of the registry lookup (`spawn_search`) that already ran for
finding *new* contacts — same two-part result Telegram/WhatsApp show.
Deliberately name-only, not message-content: full-text search over
message history is a real but separate, heavier feature (would need an
index, not a linear scan on every keystroke).

Still waiting on a steer for: message reactions/edit/delete, richer
read receipts, group video calls — see the open question in chat.

## Round 8 — room/group join fix (recap) + ringtone

### Room ticket fix, now fully wired

Round 7's write-up covered the diagnosis; this round is the actual fix
landing: `ticket::encode_room`/`decode_room` (new `mrtk1...` ticket
type), `join_room_with_retry` and `RoomDoc::open` now take a real
bootstrap peer instead of always `vec![]`, `spawn_join_room` detects
ticket-vs-plain-name input, and a "Copy invite ticket" button was added
to the room-info panel. One unverified call: `Endpoint::add_endpoint_addr`
— see the inline `VERIFY` comment at its call site in `spawn_join_room`;
if `cargo check` says that method doesn't exist, check iroh's current
`Endpoint` docs for whatever seeds the address book now, and swap the
name — the surrounding logic (seed the address, then bootstrap gossip
*and* the docs sync with the same peer) is the actual fix regardless of
that one method's exact name.

### Ringtone (`src/ringtone.rs`, new module)

Dedicated cpal output stream on its own thread — separate from
`net::calls::audio`'s call-session I/O, since ringing happens *before*
any call session exists. Ringback (outgoing) and ring (incoming) use
different cadences/frequencies on purpose, so they're distinguishable by
ear alone. Hooked into every call-state transition in `ui/mod.rs`:
starts on `CallEvent::Incoming` and on placing a call
(`spawn_call_peer_with`), stops on `CallEvent::Connected`,
`CallEvent::Ended`, decline (`spawn_answer_call`), hang-up
(`spawn_hang_up`), and the early-return path if call setup fails before
dialing even starts.

**Important compile-time catch, not just an addition**: `cpal` is only a
dependency under `cfg(not(any(target_os = "android", target_os =
"ios")))` in `Cargo.toml` (see Round 5) — a naive `use cpal::...` at the
top of `ringtone.rs` would build fine on desktop and then fail outright
on `dx build --platform android`, since the crate wouldn't even be in
scope. Split into `desktop`/`android_stub` submodules behind matching
`#[cfg(target_os = "android")]` gates instead, both exposing the same
`Ringtone` type/API, so `ui/mod.rs`'s call sites don't need their own
`#[cfg]` branches — Android just gets a real-but-silent `Ringtone` for
now. An actual Android ringtone is native-platform territory (system
`Ringtone`/notification-channel sound via the foreground service in
`MainActivity.kt`, not `cpal`) — noted as follow-up work for whoever
picks up the Android shell, not something to fake through this module.

### Still open from this batch of requests

Message reactions, edit/delete, and read receipts beyond the current
single/double-check are **not started**. Each needs new
`protocol::message` wire variants, `store.rs` schema changes, and
gossip/DM-broadcast wiring before any UI work — bigger and riskier to
rush through blind in the same pass as the room-join fix and ringtone
above. Flagging clearly rather than shipping a half-wired version of any
of them.

## Round 9 — fixing the compile errors Round 8 shipped

`cargo check` caught five real errors — all now fixed, all worth
recording *why*, since two of them were genuine API-knowledge gaps, not
typos:

1. **`cpal::sample_rate()` doesn't return a tuple struct in this pinned
   version.** `ringtone.rs` had `.sample_rate().0`, copied from an older
   cpal API shape (`SampleRate(u32)`) instead of matching this
   codebase's own `net::calls::audio.rs`, which already correctly does
   `let device_rate = supported.sample_rate();` with no `.0`. Fixed to
   match. Should have grepped audio.rs for this before writing it fresh,
   not after.
2. **`Endpoint::add_node_addr` (and any `add_endpoint_addr` guess at its
   renamed form) doesn't exist.** Turns out this was removed from iroh
   upstream — n0-computer/iroh#3485 — specifically because it was being
   replaced by a proper `Discovery` service. The current way to seed a
   known address before dialing a bare `EndpointId` is
   `iroh::discovery::static_provider::StaticProvider`: create one,
   register it once via `endpoint.discovery().add(provider.clone())`,
   and call `provider.add_node_info(addr)` at runtime whenever you learn
   an address out-of-band (a room ticket, here). Added a
   `static_provider: StaticProvider` field + `App::static_provider()`
   accessor, registered right after `Endpoint::builder(..).bind()` in
   `App::start`. `spawn_join_room` now calls
   `static_provider.add_node_info(addr)` instead of the nonexistent
   method — same fix, just via the API iroh actually still has.
3. **Three more `ensure_room_metadata_standalone`/`RoomDoc::open` call
   sites** (room-info load, room title rename, remove-member) that
   Round 8's edit missed — the bootstrap-parameter signature change
   needed updating at every call site, not just the two involved in the
   join flow. All pass `vec![]` (no ticket context at any of those three
   — they're all operating on a room already joined/synced, not
   inviting into a new one).

No more flagged-unverified API calls left in this batch — `StaticProvider`
is confirmed against current upstream docs/changelog, not a guess.

## Still owed from this round's request: reactions, edit/delete, read receipts

Not started. Real scope for next round:
- `protocol::message`: new wire variants — `Reaction { target_id, emoji, remove: bool }`,
  `Edit { target_id, new_body }`, `Delete { target_id }`, and a `Read { up_to_id }`
  receipt distinct from the existing single/double-check delivery marks.
- `store.rs`: schema changes to hold reactions (message_id → Vec<(sender, emoji)>),
  an edited-body + edited-at column, a tombstone/deleted flag, and a
  per-conversation last-read watermark per contact.
- Broadcast wiring through both the gossip (rooms) and direct (DMs) paths
  — these are two separate delivery mechanisms in this codebase and both
  need every new message type handled.
- UI: reaction picker + display in `chat.rs`, edit-in-place composer
  state, delete confirmation, read-receipt indicator beyond the current
  sent/delivered checks.

## Round 11 — reactions, edit/delete, read receipts: wired end-to-end

Completes what Round 10 left half-done. Full path now exists for all
four, both DM and room (read receipts DM-only, by design — see
`Body::Read`'s doc):

- **Wire protocol** (`protocol/message.rs`): `Body::Reaction`,
  `Body::Edit`, `Body::Delete`, `Body::Read` + `Envelope::reaction/edit/
  delete/read_receipt` builders.
- **Storage** (`store.rs`): `envelope_id` now actually persisted (it
  never was before this batch of work — a real gap, since nothing could
  address "this specific message" over the network without it),
  `reactions` + `read_watermarks` tables, edit/delete tombstone columns.
  `apply_edit`/`apply_delete` check sender ownership *in the SQL `WHERE`
  clause itself* — a forged edit/delete from someone who didn't send the
  original message matches zero rows and silently no-ops, no separate
  check-then-write race.
- **Incoming** (`app.rs`): `record_incoming_dm`/`record_incoming_room`
  persist all four; `ui/mod.rs`'s `AppEvent` handlers additionally
  update the live in-memory bubble list (`apply_reaction_locally`/
  `apply_edit_locally`/`apply_delete_locally`, mirroring the existing
  `mark_acked` pattern) so the change shows up without a reload.
- **Outgoing** (`ui/mod.rs`): `spawn_send_edit`/`spawn_send_delete`/
  `spawn_send_reaction` share a new `deliver_envelope` helper (connect-
  or-reuse-session for DMs, join-or-reuse for rooms — same logic
  `spawn_dm_send`/`spawn_room_send` already had, factored out since
  these three don't share those two functions' "log a new message on
  success" side effect). Editing reuses the existing composer: "Edit"
  pre-fills it and sets `ui.editing`; `send_to_active` checks that first
  and sends an `Edit` instead of a new message. `Escape` cancels an
  in-progress edit (checked before the overlay-closing priorities from
  Round 6, since it's the most "modal" state the composer can be in).
- **UI** (`chat.rs`): reaction badges (grouped, tap to add/remove your
  own), a hover-revealed quick-react row (👍❤️😂😮) plus Edit/Delete on
  your own text messages, an "(edited)" marker, tombstoned messages shown
  as an italic placeholder, and a third read-receipt tick state (bold
  accent color) alongside the existing sent/delivered ones.
- **History migration note**: `envelope_id` is `NULL` for every message
  row written before this round — those can't be reacted to/edited/
  deleted (nothing to address them by), which is correct, not a bug;
  don't expect the feature to reach backward into old conversations.

Brace-balance-checked across every file touched this round; no
unverified/guessed external APIs introduced (unlike Round 8/9's iroh
detour) — everything here builds on this codebase's own existing
patterns (`mark_acked`, `spawn_dm_send`/`spawn_room_send`,
`add_column_if_missing`). Still: **please run `cargo check`** — that's
the one thing this sandbox categorically cannot do, and a change this
size across 5 files is exactly the kind that benefits most from it.

### Deliberately out of scope this round

- No confirmation dialog before Delete — fires immediately. Worth adding
  (a small inline "Undo?" toast would fit this codebase's existing toast
  pattern better than a modal), just not done here.
- No reaction picker beyond the 4 quick-react emojis — a full emoji
  picker is a separate, larger UI component.
- Room read receipts (who in a room has seen which message) — see
  `Body::Read`'s doc for why this needs a genuinely different design
  (per-member tracking against an interleaved multi-sender timeline)
  than the single DM watermark implemented here.

## Round 12 — native-feel CSS pass (Linux/Windows), pure styling only

Picked back up the platform-scope work from earlier in the session —
this is the low-risk half of it (pure CSS, zero compile risk), not the
whole native-feel table:

- **Font**: `font-family` now leads with the `system-ui` keyword instead
  of a hardcoded web-safe stack — resolves to the OS's actual configured
  UI font (Cantarell/Ubuntu on GNOME, Segoe UI Variable on Windows 11) in
  both webview backends this app ships on, without needing to detect the
  platform and hand-pick a name.
- **Scrollbars**: thin, overlay-style via `::-webkit-scrollbar*` — both
  backends (webkit2gtk on Linux, WebView2 on Windows) are Chromium/
  WebKit-based despite the `-webkit-` prefix, so this isn't Safari-only
  despite the name.
- **Text selection**: `user-select: none` app-wide (title bar, tabs,
  buttons, bubble-meta — the "app chrome" that shouldn't highlight like a
  web page), with `user-select: text` explicitly restored on `.bubble`
  (message content — the thing people actually want to select/copy) and
  on all `input`/`textarea` elements.

### Still deferred — needs real API calls, not just CSS

Native title bar/window chrome, GTK/Windows theme (dark/light) live
sync, right-click context menu suppression/replacement, and matching
native animation easing curves all need `tao`/`wry` window-handle calls
or dbus (`xdg-desktop-portal`) — real external APIs I can't verify
against the pinned versions without a build, the same caution that
applied to the iroh discovery-API detour in Round 8/9. Rather than guess
a third time on unverified APIs in the same session, these stay on the
list for whenever there's a build available to check against.

## v0.6 — offline mesh (BLE/Wi-Fi) + Android background wake, untested (no build available)

Same caveat as every entry above: written and reviewed by hand against
the actual crate APIs I could verify from existing call sites in this
codebase (e.g. `EndpointId::as_bytes`/`from_bytes`, the `DmEvent`
pipeline, the `SettingsToggle` component signature) — not compiled,
since this sandbox still has no `cargo`/`rustc`. Test a real build
before trusting it.

- Added `net::mesh`: a flood-with-TTL relay (`Envelope`/`SeenCache` in
  `envelope.rs`) driven by two `MeshTransport` impls — `lan.rs` (UDP
  broadcast, real on every target including Android — deliberately not
  mDNS/multicast, which needs a `WifiManager.MulticastLock` on Android
  this crate can't request) and `ble.rs` (btleplug scanning, desktop
  only; see that file's doc comment for why BLE *sending* is a
  documented no-op — btleplug has no cross-platform peripheral/
  advertiser role, and Android BLE is cfg'd out entirely pending the
  same JNI-bridge gap already tracked for the native renderer).
- Mesh-delivered messages decode through the exact same `DmEvent::
  Received` → `record_incoming_dm` path a QUIC-delivered one does — see
  `App::start`'s new inbound-forwarding task. Outgoing mesh fallback is
  wired into `spawn_dm_send` (`siar-ui`) for the two real failure
  points: connect failure and post-connect send failure.
- Two new off-by-default settings (`background_wake_enabled`,
  `offline_mesh_enabled`) in `store.rs`, plus a new Network tab in
  Settings with live status/counts.
- Caught one real bug in review before it would've hit a build:
  `self: &Arc<Self>` isn't a stable Rust receiver type (only bare
  `Arc<Self>` is, without the unstable `arbitrary_self_types` feature)
  — `MeshManager::start` takes `self: Arc<Self>` instead, and every
  call site was checked for whether it still needs the `Arc` afterward
  (`.clone().start()`) or can just move it (`.start()`).
- Also caught a second one: an early draft of the mesh-fallback UI code
  held a `Signal` read guard across an `.await` inside a `match` arm —
  the same `AlreadyBorrowed`-class hazard this codebase already hit
  once with a held *write* guard (see the Round-notes above on that
  panic). Fixed by extracting the owned `Arc<MeshManager>` before the
  await (`try_mesh_send` in `siar-ui`), not by holding the guard
  through it.
- Android: new `RelayForegroundService.kt` (`START_STICKY`, typed
  `dataSync` foreground service, `PRIORITY_MIN` notification — Android
  requires *a* notification for any foreground service, there's no way
  around that) + `BootCompletedReceiver.kt` to restart it after reboot.
  Both read a `SharedPreferences` mirror of the setting rather than a
  live Rust→Kotlin call — that bridge doesn't exist yet, see
  `crates/android/README.md`'s updated "intentionally NOT in this
  crate" section for exactly what's still open there.

## v0.6.1 — real fix for a `!Send` future, dependency bumps, settings/profile UI polish

- **The actual bug from the first real build attempt**: `for t in
  self.transports.lock().unwrap().iter().cloned().collect::<Vec<_>>()
  { ... t.broadcast(&next).await ... }` in `net::mesh::mod.rs` looks
  like the `MutexGuard` temporary is dropped as soon as `.collect()`
  runs, but it isn't — a temporary created in a `for` loop's *head*
  expression lives until the end of the whole `for` statement, not
  just until the iterator is produced. So the guard was actually held
  across every `.await` in the loop body, making the enclosing
  `tokio::spawn`ed future `!Send` and failing to compile. Fixed in
  both `send()` and `on_received()` by binding the collected `Vec` to
  its own `let` *before* the loop, so the guard drops at the end of
  that `let` statement instead of the end of the loop.
- Bumped the six dependencies `cargo` flagged as having newer versions
  available: `android_logger` 0.14→0.15, `btleplug` 0.11→0.12, `cpal`
  0.17→0.18, `hkdf` 0.12→0.13, `sha2` 0.10→0.11, `wl-clipboard-rs`
  0.8→0.9. Each is a 0.x minor bump, which semver treats as
  potentially breaking — I checked the one call site that seemed
  highest-risk (`Hkdf::<Sha512>::new(None, &seed)` in `identity/
  seed.rs`, which is the oldest-stable part of this API and unlikely
  to have moved), but none of the others were re-verified against a
  real build. Watch for API-shape errors on `cpal`/`btleplug`
  specifically if this doesn't compile clean.
- Settings/Profile UI polish: new `css/settings.css` (real sliding
  switch for `SettingsToggle` instead of an "On"/"Off" text button, a
  segmented pill tab bar instead of flat square tabs, a proper profile
  card with a gradient avatar ring and a bordered ticket/QR card, a
  connection-status pill with a colored dot, and a 2×2 status-card grid
  for the Network tab's mesh readout instead of stacked text lines).

## v0.6.2 — cpal 0.18 fixes, LAN/BLE mesh reliability pass, "same router, no internet" confirmed

- **cpal 0.18 broke 8 call sites**: `Device::build_input_stream`/
  `build_output_stream` take `StreamConfig` by value in 0.18, not
  `&StreamConfig` (this changed in the 0.17→0.18 bump from the last
  round). Fixed all 8 (`net/calls/audio.rs` ×6, `ringtone.rs` ×2) by
  passing `config.clone()` — `config` is a small, cheap-to-copy struct,
  and each call site sits in its own `match` arm anyway, so a bare move
  would also have worked, but `.clone()` keeps every site identical
  regardless of arm structure.
- **LAN mesh reliability**: added `if-addrs` (pure Rust, wraps
  `getifaddrs`/the Windows IP Helper API) so `net::mesh::lan` sends to
  every local interface's actual subnet-directed broadcast address
  (e.g. `192.168.1.255`) *in addition to* the global `255.255.255.255`
  — some Wi-Fi drivers and access points filter the global address,
  and this matters more on Android than desktop. This is the direct
  mechanism for "two devices on the same router, no internet at all":
  broadcast only ever needs L2/L3 reachability on that one subnet, so
  it doesn't care whether the router itself has a working WAN uplink.
- **Peer count was a bug, not just cosmetic**: `peers_seen_recently`
  used to `fetch_add(1, ...)` on every packet/BLE event, so it only
  ever climbed for the life of the process — "12 nearby" could mean
  one very chatty peer an hour ago. Replaced with `MeshStatus::
  note_peer_seen`, a real decaying set (dedup by sender id, 2-minute
  window) shared by both transports, so the Network tab's number now
  means "distinct peers seen in roughly the last 2 minutes."
- **BLE scan reliability**: added a 20s rescan supervisor — BlueZ (and
  other OS Bluetooth stacks) can silently end a discovery session on
  its own internal timeout, which previously would've left
  `ble_active` reporting `true` while quietly discovering nothing new.
  `start_scan` on an already-scanning adapter is a normal no-op, so
  this is cheap insurance, not meaningful extra cost. Also caught and
  removed a speculative addition (a "keep the peripheral cache warm"
  properties lookup) from an earlier draft of this pass before it went
  out — no verified benefit, cut per this project's own standing rule
  against guessing at unverified API behavior.

## v0.7 — status from local media, real BLE peripheral advertising (Linux), AV1 hw accel extended to status video

Three separate asks this round; scoped honestly rather than attempting
all of them at full breadth — see "Not done this round" below for what
was deliberately left out and why.

### Status from local media
- New `net::calls::audio::decode_and_encode_audio_file` — decodes an
  arbitrary local audio file (mp3/aac/flac/wav, via `symphonia`, pure
  Rust) and re-encodes it into the exact same Opus clip blob format
  `record_and_encode_voice_clip` (live mic) already produces, sharing
  the packetization loop (factored out as `encode_pcm_clip`) so there's
  one Opus-encode implementation, not two. Wired into a new "🎵 Attach
  audio file" button next to status's existing record-voice button.
  Desktop only — see below.
- Status image attach from a local file already existed (`spawn_attach_
  status_image`, `rfd`-based) — nothing changed there, it just wasn't
  new this round.
- **Not done**: local video file attach. General video container/codec
  decode (mp4/mov/webm — anything that isn't already a bare AV1
  bitstream) needs a real demuxer+decoder, which is a materially bigger
  dependency than `symphonia` covers (audio-only) — didn't want to pull
  in an ffmpeg-class dependency for this without discussing the
  tradeoff first. Recording a fresh video clip via the camera
  (`spawn_record_status_video`) remains the supported path.
- **Not done**: any of this on Android/mobile. `rfd` (the file-dialog
  crate every local-file attach path in this codebase uses — image,
  avatar, now audio) doesn't support Android; closing that needs a
  Kotlin Storage-Access-Framework picker bridged back into Rust via JNI,
  which is a real, separate, and fairly involved piece of native-bridge
  work — flagged rather than guessed at blind, same standard as every
  other Android bootstrap gap already tracked in this project.

### BLE peripheral advertising (Linux)
- `net::mesh::ble` now actually advertises this node's presence on
  Linux, via `bluer` (BlueZ's official Rust bindings) — verified against
  `bluer`'s own published `le_advertise`/`gatt_server` examples field-
  for-field (`Advertisement`'s `advertisement_type`/`service_uuids`/
  `manufacturer_data`/`discoverable`/`local_name`, and
  `AdvertisementHandle`'s drop-to-unregister behavior) before writing it,
  not guessed. This is the piece that makes two Siar nodes' BLE
  scanners actually see each other, rather than each one scanning into
  silence.
- Deliberately scoped to *advertising*, not a full GATT server:
  `bluer`'s `Application`/`Service`/`Characteristic` API for publishing
  read/write characteristics exists, but its exact struct shape wasn't
  something to confidently verify from documentation snippets the way
  the simpler `Advertisement` struct was — a subtly wrong GATT server
  tends to fail confusingly rather than loudly, and this file already
  has a standing rule against shipping that kind of guess.
- Net effect: `BleTransport::broadcast()` — actually sending an
  `Envelope`'s bytes over BLE — is still a documented no-op, on every
  platform including Linux now. Advertising presence and running a
  GATT server peers can write real message bytes into are two different
  pieces of work; only the first is done.
- Windows' peripheral role (`GattServiceProvider` via WinRT, reachable
  from Rust through the official `windows` crate — no hand-written C++
  needed there either) and Android's (`BluetoothGattServer`, a Kotlin
  API needing a JNI bridge, not a C one) remain open — real gaps, not
  fabricated code. Neither actually needs raw C/C++, for what it's
  worth — the "if C/C++ is needed" case didn't come up on Linux (a Rust
  crate covers it) and doesn't look like it'll come up on Windows
  either; it's JNI/Kotlin work on Android, not C.

### AV1 hardware acceleration
- Added `av1_vaapi` to the hardware-encoder candidate list (Linux
  systems without an NVENC/QSV-capable GPU — most AMD GPUs, Intel
  systems without QSV set up) — with the device/upload-filter args VAAPI
  specifically needs that the other three candidates don't
  (`-vaapi_device`, `-vf format=nv12,hwupload`), correctly split into
  "before the first `-i`" vs. "after it" (an earlier draft of this had
  them in one flat list in the wrong position relative to `-i` — caught
  and fixed before it went out).
- Status video (`encode_clip`/`decode_clip`) now tries the same hardware
  path live calls already used and falls back to software on any
  failure — this existed for live calls already (from a previous
  session, not this one) but status video was still 100% software
  before this round even on a machine with a working hardware encoder.
- **Not done**: Android hardware AV1. The existing hw-accel approach
  shells out to `ffmpeg` as a subprocess, which isn't viable on Android
  (no bundled ffmpeg binary, no general shell access) — real hardware
  accel there needs Android's `MediaCodec` (NDK C API or the Java/Kotlin
  one), a genuinely separate implementation from the ffmpeg-subprocess
  approach used elsewhere, not an extension of it. Flagged rather than
  half-built.

## v0.8 — theme styles, real background-task leak fixes, Android H.264/H.265 via MediaCodec (highest-risk code in this repo)

### Theme system
Added `store::ThemeStyle` (`Regular`/`HackerGreen`/`HackerRed`) as a
second axis alongside the existing `ThemeMode` (System/Light/Dark) —
picking a hacker style overrides light/dark entirely rather than
combining with it (it's a fixed palette, not a light/dark variant of
its own); switching back to Regular restores whatever `ThemeMode` was
already set to. New CSS: `tokens.hacker-green.css`, `tokens.hacker-red.css`
(color tokens), `hacker.css` (monospace font + glow on the handful of
already-high-emphasis elements — title bar text, own-message bubbles,
settings header — not every line of chat, which would be unreadable).
Settings' Appearance tab got a new "Style" row beneath the existing
Theme row. Caught two wrong CSS class-name guesses in my own first draft
before shipping it (`.sidebar-header` doesn't exist at all;
`--bubble-own` is a *variable* name, the class is `.bubble.own`) — fixed
by grepping the actual CSS/rsx for real class names rather than trusting
the first guess.

### Real edge case, not a hypothetical one
`net::mesh`'s `LanTransport` and `BleTransport` each `tokio::spawn`ed
their receive-loop/rescan-supervisor tasks as fully detached — dropping
the transport (which is what `MeshManager::stop()` did, e.g. when the
user turned "Offline mesh" off in Settings) did nothing to actually stop
them. Three background tasks total would keep running indefinitely,
still holding the Bluetooth adapter and UDP socket, still burning
radio/CPU, regardless of what the Settings toggle said. Fixed by
capturing each `JoinHandle` and aborting it in `Drop` — the fix a
battery-sensitive-platform instruction is specifically about, not a
cosmetic one.

### Android H.264/H.265 (`net::calls::mediacodec`)
Real `AMediaCodec` (NDK C API) wrapper via `ndk-sys` — encode and
buffer-mode decode for both codecs, a real capability probe (actually
tries to create a hardware encoder rather than assuming), RGB↔I420
conversion reusing this file's existing (already-working) YUV math from
the AV1/dav1d decode path. This is the one file in the whole project
built as raw `unsafe` FFI with no compiler available to check it —
function names/struct layout/constants were checked against the NDK's
own public header source and independent third-party usage examples
before writing, not recalled from training data alone, but it's still
the highest real-risk code this pass touched. **Needs a real-device test
before a release build depends on it**, more than anything else here.

Wire protocol: `net::calls::CallMsg` now carries `hw_video_codecs: Vec<
VideoCodec>` in both `Invite` and `Answer`, `negotiate_codec` picks
H.265 > H.264 > AV1 by mutual support, `CallEvent::Connected` exposes
the result. `calls::ALPN` bumped `/1` → `/2` for this — postcard has no
wire-level struct versioning of its own, so a shape change here is
breaking regardless; a version bump means old and new builds fail to
negotiate cleanly rather than one silently misparsing the other's bytes,
consistent with how every previous breaking wire change in this project
was handled.

**Completed in the codec/QR follow-up below**: this earlier pass stopped
before the live capture loop consumed the selected codec and before status
clips carried a codec tag. The follow-up added both wire-format changes and
compile-checked them for desktop and Android; this note is retained as the
historical boundary of the earlier pass.

## v0.10 — QR intake, mobile clipboard, and end-to-end codec dispatch

- The Chats search header now offers QR intake from the rear camera or
  gallery. Images are bounded to 20 MB, decoded off the render loop, and the
  QR payload must pass the existing Siar ticket decoder before a contact
  request is sent.
- Mobile clipboard copy now calls the WebView/system clipboard and falls
  back to a temporary selected textarea for older Android WebViews.
- Call signaling advertises both encode and decode lists, negotiates each
  direction independently, and refuses video when no honest intersection
  exists. Live frames carry the selected codec and mismatches are dropped.
- Status-video blobs now store their codec. Android MediaCodec capability
  probes configure and start the requested format once, then cache the result.

## v0.9 — AV1 in MediaCodec, a real negotiation bug fixed, MediaCodec edge-case pass, encrypted backup

### AV1 via AMediaCodec
Removed the AV1 rejection from `mediacodec.rs`'s encoder/decoder
constructors — AV1 hardware is real on some newer Android SoCs, more
commonly for *decode* than *encode*, and the capability probe now
checks for it like the other two codecs.

### A real negotiation bug, not a hypothetical one
The capability probe (`available_hw_codecs`, added last pass) only ever
checked *encode* capability and put that on the wire — but what a peer
actually needs to know before deciding it's safe to send this device a
given codec is whether this device can **decode** it, a different
question. This under-serves AV1 specifically: hardware AV1 *decode* is
meaningfully more common than hardware AV1 *encode* on today's Android
devices, so a decode-only device was getting nothing out of this
negotiation. Fixed by splitting into `available_hw_decode_codecs`
(what goes on the wire) and `available_hw_encode_codecs` (used only
locally), and renaming `negotiate_codec` → `negotiate_send_codec` with
explicit directional semantics: each side of a call now computes its
own send-codec independently (my encode capability × their decode
capability), so the two directions of one call can genuinely use
different codecs when that's the better outcome for each side.

### MediaCodec edge-case pass
- **Real correctness bug fixed**: hardware encoders commonly hold
  several frames in an internal reorder/lookahead buffer and won't
  emit them without an explicit end-of-stream signal — `encode_frame`
  alone, with nothing calling it afterward, would silently lose
  however many frames the codec was still holding onto. Added
  `finish_encoding`/`finish_decoding`, which signal EOS and drain until
  the codec actually confirms it's done (bounded — a codec that never
  confirms can't hang the caller forever).
- **Also fixed**: `feed_input`'s `flags` parameter was being accepted
  but silently ignored (hardcoded `0` in the actual `queueInputBuffer`
  call) — meant EOS could never actually reach the codec even after
  the fix above was written; caught before it shipped.
- **Found and left unfixed, honestly**: `rgb_to_i420`/`i420_to_rgb`
  assume tightly-packed planar I420 unconditionally. Android's
  "flexible" YUV420 format can also resolve to semi-planar NV12 on a
  given device, which this doesn't detect or handle — would produce
  wrong colors, not a crash, on such a device. Fixing this properly
  means querying the codec's actual negotiated format/stride and
  branching — real, scoped work, flagged rather than attempted blind
  on top of everything else in this pass.
- One doc comment fixed that referenced two functions
  (`read_i420`/`write_i420_padded`) that don't exist anywhere in the
  file — leftover from an earlier draft, caught on re-review.

### Encrypted backup (`backup.rs`)
New: `backup::create_backup`/`restore_backup` — the recovery seed
phrase, `messenger.db` (+ WAL/SHM sidecars if present, so nothing
written since the last checkpoint is silently dropped), and every media
blob, packed into one file and encrypted with a user-chosen backup
passphrase (Argon2id → XChaCha20-Poly1305) kept deliberately separate
from the identity/seed itself. `create_backup` cross-checks the entered
seed phrase against the device's actual current identity
(`identity::verify_seed_matches_current`, new) before doing anything
else, so a wrong/mistyped phrase is caught at backup time instead of
surfacing as "this restored to a different identity than expected"
months later.

Wired into two places, deliberately not the same place:
- **Settings → Storage** ("Back up now"): safe to run against a live
  app, since it only reads current files.
- **Onboarding** ("Restore from encrypted backup"), not Settings —
  restore rewrites `messenger.db` and the blob store on disk, which is
  not safe to do underneath a *running* app with an open SQLCipher
  connection and iroh endpoint. Onboarding runs before any of that
  exists, which is the only place this is safe to expose right now.

"Online drive" scoped honestly: this saves one encrypted file via the
same local file dialog every other local-file feature in this codebase
uses — "backing up to an online drive" means pointing that dialog at a
Dropbox/Google Drive/iCloud sync folder already on disk, not a real
OAuth/upload-API integration with any specific provider (that's a
credentials-and-token-management undertaking in its own right, and
Signal Desktop among others ships exactly this simpler pattern for
exactly this reason).

Caught and fixed one real security-relevant edge case while writing
`restore_backup`: a backup file is untrusted input by the time it
reaches that function (could be corrupted, tampered with, or
hand-crafted) — added `safe_join` to refuse any blob file path containing
`..` or an absolute path, which without the check could otherwise write
files anywhere on disk this process has permission to, not just inside
the blob store directory.

**Known real limitation, stated plainly**: `create_backup`/
`restore_backup` hold the entire database and every media blob in
memory at once. Fine for typical chat history; a large media library
could mean a multi-gigabyte in-memory buffer. Streaming encryption
would avoid this at real added complexity — not done this pass.

Desktop only (both flows use `rfd` for file picking, same mobile gap as
every other local-file feature already in this codebase).

## v0.9.1 — real compile fix: #[cfg] isn't valid directly on an rsx! element

`rsx!` has its own macro grammar, not plain Rust item syntax — attaching
`#[cfg(...)]` directly to a `button { ... }` node (what the previous
pass's onboarding restore-button gate did) is a syntax error there
("expected identifier"), even though the identical attribute works fine
on a plain Rust `fn`/`{ }` block. Fixed by using `cfg!(...)` (a
compile-time bool literal) as an ordinary `if` condition instead, which
`rsx!` does support natively, for the button itself — and real
`#[cfg(...)]` (valid, since it's plain Rust there) inside the `onclick`
closure bodies that actually call `rfd`.

Also caught while fixing this: `spawn_create_backup`'s doc comment
claimed the Storage tab was "already desktop-only," used as the reason
it didn't need its own mobile gate. That claim was wrong — the tab has
no such gate — so without fixing it, "Back up now" would have been
reachable, and broken, on mobile. Fixed with the same
cfg-gate-and-toast pattern `spawn_attach_status_image` already
established.
