# v0.3 → v0.4: workspace split, renamed to Siar

## What changed

**Workspace, not single crate.** Four members under `crates/`:

- `siar-core` — identity, net (transport, gossip, contacts, conv
  docs, calls), protocol, store, media, ringtone, ticket, app (the
  `Endpoint`/`Router`/`Docs` orchestration). Zero UI dependency. Same
  code as before, same module layout — only the crate root moved, so
  `use crate::X` internal paths were unaffected.
- `siar-ui` — the whole Dioxus component tree (chat, onboarding,
  requests, sidebar, root `AppRoot`), depending on `siar-core`.
  This is the one codebase both platform crates share, regardless of
  which renderer feature a given platform crate builds with.
- `siar-desktop` — CLI parsing, single-instance file lock, launch.
- `siar-android` — no CLI, no instance lock (Android doesn't need
  either), same `CONFIG` + launch pattern. See its own `README.md`.

**CSS is a real module system now**, not one Rust string. `siar-ui/
src/css/` has one file per concern (`tokens.dark.css`, `tokens.light.css`,
`base.css`, `titlebar.css`, `sidebar.css`, `chat.css`, `composer.css`,
`onboarding.css`, `toast.css`, `responsive.css`), pulled together by
`css/mod.rs::stylesheet(dark)` — same call-site contract as the old
`theme::css(dark)`. Content is unchanged; only the packaging is. It
injects via `style { }` + `include_str!` rather than Dioxus's own
recommended `asset!` + `document::Stylesheet` — that path has an open
upstream rendering bug under the `native` renderer specifically
(DioxusLabs/dioxus#4666), and `style { }` works identically under
`webview` too, so there was no reason to special-case it either way.

**Renderer is a Cargo feature, not a fixed choice — `webview` by
default, `native` opt-in.** See `siar-desktop`/`siar-android`'s
`Cargo.toml` `[features]` blocks for the full reasoning; short version
in the changelog below. `siar-ui` itself doesn't declare a renderer
feature at all — `rsx!`/hooks/signals don't need one, only the platform
crates' `launch` call does, which is what lets one `siar-ui` build
against either backend without editing it.

## What's staying on the "needs a build" list

Real native-integration behavior that a build can confirm and a guess
can't, so none of it was touched. All four assume `webview` (the
default) unless noted:

1. **Title bar / window chrome** — a real custom title bar, drag
   regions, minimize/maximize/close, needs a tao/wry window handle.
   The existing `.titlebar` div in `css/titlebar.css` is in-content
   CSS, not OS chrome — left as-is.
2. **Live theme sync** (following OS light/dark mode changes at
   runtime) — `dark` is still the hardcoded `true` in `AppRoot`
   (`// TODO: load from persisted settings`), same as before this pass.
3. **Context-menu replacement** (a custom right-click/long-press menu
   instead of the OS/webview default) — needs whatever native menu API
   the enabled renderer exposes; not guessed at.
4. **Android's `native`-renderer bootstrap glue** — only relevant if
   you build with `--features native`. Under the default `webview`
   feature this doesn't apply: `dx` generates a real `MainActivity.kt`
   hosting wry, which is the established, checkable path. See
   `crates/android/README.md`.
5. **`document::eval` under `native`** (`siar-ui/src/lib.rs`, the
   Ctrl+K handler) — only relevant if you build with `--features
   native`; a no-op under `webview` isn't a concern there since that's
   a real webview DOM.

## Building

- `cargo build -p siar-desktop --release` — builds with `webview`
  (the default feature) unless you pass `--no-default-features
  --features native`.
- `dx serve` / `dx build --platform android` from `crates/desktop` or
  `crates/android` respectively (each carries its own `Dioxus.toml`).
- `cargo check --workspace --exclude siar-android` from a non-Android
  host — `siar-android`'s Cargo.toml depends on Android-only crates
  that won't resolve off-target; workspace `default-members` already
  excludes it from a bare `cargo build`.

Same as always: I can't run `cargo build`/`dx build` in this sandbox,
so none of this has been compiled — this is static restructuring and
review, not a verified build. Your feedback after compiling is what
closes the loop.

## v0.4.1: renamed to Siar, launch-API fix

**Rename.** Package/crate/binary names, the Kotlin package, the app's
data directory identifier, and `Dioxus.toml`/doc titles all moved from
`iroh-messenger`/`messenger-*` to `siar`/`siar-*`.

**Deliberately NOT renamed** — a compatibility concern, not a cosmetic
one:

- The ALPN strings (`net::calls`, `net::contacts`, `protocol::dm`), the
  HKDF domain-separation labels (`identity::{mod,seed}`), and the
  blake3 room/DM topic-hash inputs (`ticket.rs`) still read literally
  `"iroh-messenger/..."`. These aren't display strings — they're wire
  identifiers and key-derivation inputs. Renaming them would derive
  different keys from the same seed phrase (different identity for
  existing users) and mismatch ALPNs against any peer still on the old
  build. If you *do* want these renamed too, say so explicitly — it's
  a one-line-per-constant change, but it's a breaking one.
- `store.rs`'s sqlite filename — same category: it's what an existing
  user's on-disk data is literally named.

**Launch API correction.** The first pass called `dioxus::launch(...)`
in both platform crates' `main.rs`, assuming the `native` feature on
`dioxus` itself picked the renderer the way `desktop`/`web` do. Better
evidence — Dioxus's own "Native Platform" docs — shows the actual entry
point is a separate `dioxus_native::launch(...)`. Fixed (and since v0.4.2
made conditional on the `native` feature — see below).

## v0.4.2: `native` renderer blocked upstream — `webview` is now the default

Two build attempts, two different rustc pins (1.89.0, then 1.91.0),
identical failure both times: `stylo` 0.8.0 (a transitive dependency of
`blitz`, which `dioxus-native` is built on) fails with `E0275`, a
trait-solver overflow deriving `Debug` on deeply recursive generic
types (`GenericCalcNode`, `GenericColorMix`, `GenericTransformOperation`,
etc.).

That the *same* error survived a rustc version change rules out
rustc-version-sensitivity as the cause — the first pass's theory was
wrong, and said so plainly rather than proposing a third pin. Checked
crates.io's sparse index directly (`index.crates.io/st/yl/stylo`):
`0.8.0` is the only `0.8.x` release ever published — next is `0.9.0`,
a breaking jump `blitz`'s `Cargo.toml` doesn't request — so `cargo
update -p stylo` has nowhere to go either. That's a real, currently
unresolved upstream build failure in this exact dependency combination,
not something a local workaround fixes.

**Fix: renderer is now a Cargo feature, defaulting to `webview`.**
`siar-desktop` and `siar-android`'s `Cargo.toml` each define:

```toml
[features]
default = ["webview"]
webview = ["dioxus/desktop"]   # or ["dioxus/mobile"] for android
native  = ["dep:dioxus-native"]
```

`webview` (Dioxus's proven tao/wry backend) builds today. `native`
(Blitz/WGPU) stays available via `--no-default-features --features
native` for once the stylo issue clears upstream — nothing about the
`siar-ui` component tree needs to change either way; only the launch
call in each platform crate's `main.rs` branches on the feature.

`rust-toolchain.toml` stays pinned to **1.91.0** — that's unrelated to
the stylo saga, it's simply iroh 1.0.3's own stated floor, and applies
regardless of which renderer feature you build with.

## v0.4.3: toolchain bumped to 1.95.0 (`cfg_select!`)

Next build attempt (with `webview` as the default renderer, so the
stylo/native saga above is no longer in play) hit a different, unrelated
error: `libsqlite3-sys` 0.38.1's build script uses the `cfg_select!`
macro unconditionally, and that only stabilized in Rust **1.95.0**
(April 2026) — confirmed against the actual stabilization PR
(rust-lang/rust#149783, tracking issue #115585), not inferred. The
previous 1.91.0 pin was correct for iroh's own floor but several
releases too old for this transitive dependency. Bumped
`rust-toolchain.toml` and every crate's `rust-version` field to
`1.95.0`/`"1.95"` — one pin, since 1.95 clears iroh's 1.91 floor too.

## v0.4.4: crate-boundary bugs from the original split, caught by a real build

Two categories of self-inflicted bug, both consequences of splitting one
crate into four that only a real `cargo build` surfaced (not visible on
review, since every path involved was internally consistent within the
old single crate):

1. **`pub(crate)` doesn't cross a crate boundary.** Five items in
   `siar-core` — `App::remember_and_hint_peer`, `connect_with_retry`,
   `ensure_room_metadata_standalone`, `join_room_with_retry`, and
   `net::calls::audio::SAMPLE_RATE` — were `pub(crate)` in the original
   single crate, which was correct *there* (visible everywhere in that
   one crate, including what's now `siar-ui`'s code) but stops meaning
   that the moment `siar-ui` becomes a separate crate calling into
   `siar-core`. Widened all five to `pub`. Checked every other
   `pub(crate)` item in `siar-core` against what `siar-ui` actually
   calls (grep, not guesswork) — the two left (`request_contact_with_retry`,
   `accept_contact_with_retry`) are genuinely core-internal-only, `siar-ui`
   never calls them, so they stay as they were.
2. **Missing direct dependencies in `siar-ui`'s `Cargo.toml`.** `image`,
   `anyhow`, `data-encoding`, and `iroh-blobs` are all used directly in
   `siar-ui/src/lib.rs` (status-image/GIF encoding, error wrapping,
   base64/hex encoding, blob store handles) — not just inside
   `siar-core` as the first pass assumed. Added all four as direct
   dependencies of `siar-ui`, matching `siar-core`'s `image` feature set
   (`jpeg`, `png`, `gif`) and `iroh-blobs` version (`0.103`) so both
   crates resolve the same types. Swept the rest of `siar-ui` for the
   same pattern against every other core-only crate (`blake3`, `zstd`,
   `postcard`, `rusqlite`, `bip39`, `hkdf`, `sha2`, `directories`,
   `toml`, `jxl-oxide`, `cpal`, `opus`, `nokhwa`, `rav1e`, `dav1d`) —
   none of those are referenced directly from `siar-ui`, so no further
   additions needed there.

## v0.5: reply-to-message, theme sync, expanded Settings, context menus, custom title bar

**Reply-to-message** — the one genuinely missing core messaging feature
(reactions, edit, delete, typing indicators, read receipts, disappearing
messages, voice calls, status, call log, and full-text search all
already existed). Threaded through the whole stack: `Body::Text`/
`Body::File` gained a `reply_to: Option<u64>` field (a breaking enum
shape change), `store.rs` gained a `reply_to_envelope_id` column, and
the UI resolves quote previews locally from already-loaded history
rather than re-sending the original content over the wire a second
time — with a graceful "Original message unavailable" fallback when the
target isn't in the currently-loaded window.

**Theme sync** — `ThemeMode::{System,Light,Dark}`, persisted in the
`settings` table that already existed (no new table). `System` is
genuine OS-level live sync via `@media (prefers-color-scheme: dark)`,
not polling — the webview re-evaluates that on its own. `Light`/`Dark`
override it via a `[data-theme]` attribute Rust sets on `.app-shell`.
`css/mod.rs`'s `stylesheet()` dropped its `dark: bool` param entirely —
there's nothing left for Rust to decide by generating different CSS.

**Settings**: 3 tabs → 7 (Profile, **Appearance**, **Notifications**,
**Privacy**, Keys, Storage, **About**). New tabs are real, not stubs —
theme picker, notification/sound toggles, read-receipt/typing-indicator
toggles (both true privacy signals: they only gate what *you* send, not
what you can see), a live blocked-contacts list with working Unblock,
and version/stack/license info. `SettingsPanel` now takes `ui: UiState`
directly rather than growing a dozen more individual props — an
existing pattern in this codebase (`MainShell` already does this), not
a new one.

**Context menus** — right-click on a message bubble (React/Reply/Copy/
Edit/Delete, own-vs-peer-aware) or a sidebar chat row (Pin/Archive,
DMs only — matches what already existed via the conversation-info
panel, just faster to reach). One global `ContextMenu` component,
dismissed by clicking its own backdrop.

**Custom title bar** — `siar-desktop` now launches with
`with_decorations(false)` and a custom in-content bar (drag region,
minimize/maximize/close via `window.drag()`/`.set_minimized()`/
`.toggle_maximized()`/`.close()` — all verified against a real working
example, DioxusLabs/dioxus#532, not assumed). Gated behind a new
`desktop-chrome` feature on `siar-ui`, forwarded from `siar-desktop`'s
`webview` feature — needed because `dioxus::desktop::*` only physically
exists in a given build when `dioxus/desktop` itself is enabled
somewhere in it, and `siar-ui` doesn't select a renderer feature itself
(see its `Cargo.toml`). Never enabled for `siar-android` or the
`native` renderer.

**Known, currently-open trade-off, not a bug**: disabling OS decorations
removes the OS's own window-edge/corner drag-to-resize, and Dioxus
doesn't yet expose an API to bring it back on an undecorated window
(DioxusLabs/dioxus#3128 — checked via search, not assumed).
`with_resizable(true)` is still set, so programmatic/taskbar-menu
resize still works; what's lost is the everyday "grab the edge and
drag" gesture. Documented at the `TitleBar` component itself, with a
one-line revert (drop `with_decorations(false)` in `main.rs`, or don't
set `desktop-chrome`) if that turns out to matter more than the custom
look is worth.

**Not touched this pass** (same "needs a build" reasoning as before):
live theme sync's `System` mode itself doesn't need a build to trust —
it's a standard CSS media query — but the title bar's actual on-screen
behavior (does dragging feel right, does minimize/maximize animate
correctly, does the known resize limitation bite in practice) does.
