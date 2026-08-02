# siar-android

Thin Android platform crate: no CLI (nothing reads argv on Android), no
single-instance lock (Android already enforces one process per app),
just `CONFIG` setup and a hand-off to the same `siar_ui::AppRoot`
the desktop crate launches — see `src/main.rs`.

## Renderer: `webview` by default, `native` opt-in

See `Cargo.toml`'s `[features]` block for the full reasoning. Short
version: `native` (Blitz/WGPU, GPU-accelerated) is currently blocked by
a stylo 0.8.0 build failure with no quick fix available, so `webview`
(Dioxus's `mobile` feature — wry hosted by a `dx`-generated
`MainActivity.kt`) is the default and the one to build against right
now: `dx build --platform android`. `native` stays available via
`--features native` for once the upstream build issue clears.

## Kotlin in `kotlin/`

Only the pieces that are genuinely required and are NOT boilerplate
`dx` already generates for you:

- `RuntimePermissions.kt` — requests `RECORD_AUDIO`, `CAMERA`, (13+)
  `POST_NOTIFICATIONS`/`NEARBY_WIFI_DEVICES`, and (31+)
  `BLUETOOTH_SCAN`/`CONNECT`/`ADVERTISE` at runtime. Call
  `RuntimePermissions.requestMissing(activity)` once per cold start
  from whatever Activity `dx` scaffolds.
- `CallForegroundService.kt` — the typed foreground service Android 14
  requires to keep a call's audio/video alive when backgrounded.
- `RelayForegroundService.kt` + `BootCompletedReceiver.kt` — the
  "Background wake" toggle in Settings' Network tab: a low-priority
  foreground service that keeps the app reachable in the background,
  restarted after reboot if the user had it on. Off by default.
- `AndroidManifest.snippet.xml` — the `<service>`/`<receiver>`
  declarations for all three of the above. Merge into `dx`'s generated
  manifest.

Under the default `webview` feature, `dx build --platform android`
generates a real `MainActivity.kt` hosting wry's WebView — diff that
against these files to see where they plug in; they're additions
to make, not replacements for what `dx` scaffolds on its own.

## What's intentionally NOT in this crate

If you switch to `--features native`: the actual JNI/native-activity
bootstrap that gets an Android process to call into this Rust binary at
all under Blitz/WGPU is a second open question on top of the stylo
build failure itself — the equivalent of `MainActivity.kt` for that
renderer is generated differently by `dx` (likely via
`android-activity`/NDK rather than a WebView host), and isn't something
to guess at without a real build to check the generated output against.

This sits on the same list as the title bar/window chrome, live theme
sync, and context-menu replacement work called out separately — real
native-integration questions that need a build to answer rather than a
guess in one session. Under the default `webview` feature, none of
those four are blocking; they only matter once/if `native` is enabled.

Two more items joined that list with the offline-mesh/background-wake
feature (`net::mesh`, `RelayForegroundService`):

- **Rust → Kotlin control bridge.** Nothing in `siar-core` can call
  `RelayForegroundService.start`/`stop` or read `RuntimePermissions`
  grant state directly yet — Settings' toggles persist the setting to
  sqlite (which `RelayForegroundService`/`BootCompletedReceiver` read
  on their own start) rather than pretending a live call across the
  boundary exists. Closing this gap is what would let toggling
  "Background wake" in-app start the service immediately instead of on
  next boot/launch.
- **BLE peripheral/advertiser role on Android.** `net::mesh::ble` is
  cfg'd out on Android entirely (see `siar-core/Cargo.toml`) —
  `btleplug`'s Android backend needs a JVM/JNIEnv handle this crate has
  no verified way to obtain. `net::mesh::lan` (Wi-Fi broadcast) has no
  such gap and works on Android as written.

## Hardware H.264/H.265 (`net::calls::mediacodec`, in `siar-core`)

Lives in `siar-core`, not here — cfg'd to `target_os = "android"`, same
pattern as everything else Android-specific that the shared call/status
video pipeline needs to reach. Uses the NDK's `AMediaCodec` C API via
`ndk-sys`'s bindings (generated from the real NDK headers, not
hand-declared) rather than any Kotlin/JNI bridge — no Kotlin file in
this directory needed to change for it.

This is the one piece of this codebase written as raw `unsafe` FFI with
no compiler in this environment able to check it — see that module's
own doc for exactly what was verified against real NDK header source
before being written vs. what's still a "should be right" rather than a
"checked" call. **Test on a real device before a release build relies
on it.** The wire protocol now negotiates codec capability between two
calling devices (`net::calls::negotiate_codec`), but as of this pass
that negotiation result isn't actually consumed by the live-call
capture/encode loop yet (still always encodes `Av1`) — the hardware
codec and the wire capability exchange are both real and in place, the
last wiring step connecting them isn't done. See `BUILD_NOTES.md`.
