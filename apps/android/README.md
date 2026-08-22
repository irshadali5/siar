# apps/android

The consumer every one of `siar-transport-wifi-direct`,
`siar-transport-wifi-aware`, `siar-transport-ble-android`, and
`siar-transport-bluetooth-classic`'s own doc comments named as
missing since they were built: "this workspace doesn't have an
`apps/android` crate yet."

## What's real here

- A working Gradle project skeleton (`settings.gradle.kts`,
  `build.gradle.kts`, `app/build.gradle.kts`, `AndroidManifest.xml`
  with every permission each transport genuinely needs, and why).
- Four Kotlin bridge classes, each driving the real Android SDK API
  for its transport (`WifiP2pManager`, `WifiAwareManager`,
  `BluetoothLeScanner`/`BluetoothGattServer`, `BluetoothSocket`/
  `BluetoothServerSocket`) and calling into the existing Rust JNI
  crates through the exact method signatures those crates' own
  `jni_bridge.rs` files declare — checked field-by-field against the
  real Rust source, not guessed.
- A new Rust crate, `siar-android-connectivity`
  (`apps/android/rust-jni-glue/`), that finally owns a shared
  `siar_domain::ConnectivityState` for the whole app — the specific
  gap every transport crate's doc comment flagged. Real unit tests,
  **compiler-verified** in an isolated sandbox build (2/2 tests pass;
  see this workspace's own memory of this session for how). Caught and
  fixed one real bug this way: `siar_domain::connectivity` is a
  private module, `ConnectivityState`/`TransportLink` are re-exported
  at the crate root instead — the first draft's `use
  siar_domain::connectivity::{...}` failed to compile until fixed to
  `use siar_domain::{...}`.
- `MainActivity.kt` wires all four transports plus the connectivity
  bridge together, requests the right runtime permissions, and shows
  next.md §60's single-line connectivity summary.
- A new Rust crate, `siar-android-messaging`
  (`apps/android/messaging-jni/`), the real `siar_messaging::
  MessageService` FFI surface this README used to name as entirely
  missing. Bootstraps an identity + `SiarEndpoint`, sends/receives real
  text messages (both the `DeviceId`-addressed path and — as of this
  pass — the unlinkable token-mailbox path, matching `apps/cli`'s
  `send`/`send-anon`/`check-mailbox`/`check-mailbox-anon` one-for-one),
  and reports `TransportLink::InternetDirect` up into
  `siar-android-connectivity`'s shared `ConnectivityState` once its
  `SiarEndpoint` successfully binds — see that crate's own `lib.rs` doc
  comment for the exact scope and what's still not covered (no groups,
  no attachments, no identity persistence, no shutdown-time
  `mark_link_down`). `MainActivity.kt` calls `bootstrap()` off the main
  thread (it blocks on real network I/O) and starts polling for
  incoming events.

## What's honestly NOT here

- **iOS.** Explicitly out of scope for this pass — see the
  conversation that produced this module for why. Nothing here assumes
  an iOS app would share this Gradle project; a real `apps/ios` would
  be its own Xcode project with its own bridge layer to the same Rust
  crates.
- **Still no messenger UI.** `siar-android-messaging`/`MessagingBridge`
  can genuinely send and receive messages via both paths now, but
  there's no conversation screen, no contact list, no way to type in a
  peer ticket and see a chat thread — `MainActivity` only shows the
  connectivity summary. `apps/cli`/`apps/desktop` remain this
  workspace's only chat clients with an actual UI. No groups/MLS and no
  attachments are wired into this FFI surface either.
- **`LocalLan`/`InternetRelay` still aren't reported into
  `ConnectivityBridge`.** `InternetDirect` now is (as of this pass, once
  `siar-android-messaging`'s `SiarEndpoint` successfully binds) — the
  other two still aren't, since nothing here can currently tell a real
  direct iroh connection apart from a relayed one or from LAN-local
  discovery (same honest approximation `apps/emergency-node`'s
  `send_and_record` helper makes on the Rust side).
- **No identity persistence.** `siar-android-messaging` generates a
  fresh `DeviceIdentity`/`AccountId` every process start — same
  Phase-1 stand-in `apps/cli`'s own `bootstrap()` doc comment already
  carries, not something this pass fixed on either platform.
- **No actual `.so` build pipeline.** `app/build.gradle.kts` expects

  pre-built native libraries under `app/src/main/jniLibs/<abi>/` — the
  standard Android layout for this — but nothing in this pass runs
  `cargo-ndk` (or any cross-compiler) to actually produce them. No NDK,
  no Android build tools, and no way to cross-compile for Android exist
  in the sandbox this was written in.
- **Nothing here has been compiled or run.** No Android SDK, Gradle
  wrapper, or emulator exists in this sandbox. Every Kotlin file is
  written against real, current Android SDK API shapes — checked
  against actual documentation wherever genuinely uncertain (see
  `WifiAwareManagerBridge.kt`'s note on `WifiAwareNetworkInfo.getPort()`'s
  undocumented "no port" sentinel) — but none of it has a compiler
  behind it the way the Rust side of this pass did. Please build this
  for real and paste back whatever Gradle/Kotlin compiler errors come
  up, the same loop this whole project has used for every Rust
  delivery.
