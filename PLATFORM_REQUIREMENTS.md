# Platform requirements & permissions

Covers what each target platform needs to build and run this app, with
particular attention to voice calling (`net::calls`), since microphone
access is the one thing here that genuinely needs OS-level permission
handling rather than just a library dependency.

## Desktop (the only platform this app currently builds for)

This project is a `dioxus-desktop` app (a native window + embedded
webview), built with plain `cargo build`/`cargo run`. There is currently
**no Android or iOS target wired up** — no `AndroidManifest.xml`, no
Gradle project, no NDK toolchain configuration, nothing under a `mobile/`
directory. See "Android" below for what actually adding that would take;
it's a real gap, not just a missing permissions file.

### Build dependencies

| Need | Arch Linux | Debian/Ubuntu | macOS | Windows |
|---|---|---|---|---|
| Opus codec | `pacman -S opus` | `apt install libopus-dev` | `brew install opus` | statically linked by the `opus` crate's build script — nothing extra needed |
| Audio I/O (cpal) | `pacman -S alsa-lib` (PipeWire's ALSA shim is usually already present) | `apt install libasound2-dev` | none (uses CoreAudio directly) | none (uses WASAPI directly) |
| WebView (Dioxus itself) | `pacman -S webkit2gtk-4.1` | `apt install libwebkit2gtk-4.1-dev` | none (uses WKWebView) | none (uses WebView2, usually preinstalled on Windows 10/11) |
| pkg-config, a C compiler | `pacman -S base-devel pkgconf` | `apt install build-essential pkg-config` | Xcode Command Line Tools | a working MSVC or MinGW toolchain (whatever `rustc` already needs) |

If `cargo build` fails specifically on the `opus` or `cpal` crates'
build scripts, it's almost always one of the above missing, not a bug in
this codebase — the error message usually names the missing `.pc` file or
library directly.

### Runtime microphone permission

Voice calls need mic access; this is handled entirely by the OS, not by
this app's code (`cpal` just asks the OS for a stream and gets whatever
the OS decides to grant):

- **Linux (PipeWire/PulseAudio):** no OS-level prompt in most desktop
  environments — access is effectively ambient once ALSA/PipeWire is
  set up. Some hardened/sandboxed setups (Flatpak, some immutable
  distros) *do* gate this behind `xdg-desktop-portal`; if mic capture
  silently returns silence there, that's the portal denying access, not
  this app.
- **macOS:** first launch that touches the microphone triggers the
  standard system permission dialog. If it's dismissed/denied, later
  calls will fail to open an input stream — check *System Settings →
  Privacy & Security → Microphone* and make sure this app (or your
  terminal, if running via `cargo run` rather than a bundled `.app`) is
  allowed. A bundled release build would need
  `NSMicrophoneUsageDescription` in its `Info.plist`; `cargo run` during
  development inherits the permission grant of whatever terminal app you
  used to launch it.
- **Windows:** *Settings → Privacy & security → Microphone* has a
  system-wide toggle plus a per-app list; "Let desktop apps access your
  microphone" needs to be on. WASAPI will otherwise just fail to open
  the input device.

None of this is something the app can bypass or need to "request" in
code the way a mobile app does — see Android below for the contrast.

## Android (not yet a real build target — here's what's actually missing)

Dioxus does have a mobile story, but it's a materially different build
pipeline from the desktop one this project currently uses — not a config
flag. Getting this app running on Android would need, roughly:

1. **Project scaffolding**: `dx create` or equivalent to generate the
   Android wrapper project (Gradle build, `AndroidManifest.xml`, NDK
   toolchain wiring). None of that exists in this repo yet.
2. **`AndroidManifest.xml` permissions** — the actual runtime-request
   equivalent of the desktop OS prompts above:
   - `RECORD_AUDIO` — for calls. This is a *dangerous* permission on
     Android; the manifest entry alone isn't enough, the app also has to
     call `ActivityCompat.requestPermissions` (or the Rust-side
     equivalent Dioxus mobile exposes) at the moment a call is first
     placed/answered, and handle the user saying no gracefully — right
     now `net::calls::audio`'s capture-thread error path already
     surfaces "no microphone found"-style errors up through
     `CallEvent::Ended`, which is the right shape to also carry
     "permission denied" once there's an actual Android build to test
     that against.
   - `INTERNET`, `ACCESS_NETWORK_STATE` — for the P2P networking iroh
     already relies on everywhere in this app, not calls specifically.
   - `POST_NOTIFICATIONS` (Android 13+) — for the desktop-notification
     equivalent (`notify-rust` is desktop-only per `Cargo.toml`; Android
     needs its own notification channel setup instead).
   - A **foreground service** declaration would be needed for calls to
     survive the app being backgrounded mid-call (Android aggressively
     suspends background audio/network work otherwise) — this is a
     real architectural piece, not a manifest one-liner.
3. **`cpal`/`opus` on Android**: `cpal` does support Android (AAudio/
   OpenSL ES backends) and `opus`'s build script can target Android via
   the NDK, but neither has been exercised on this codebase's dependency
   set yet — worth a dedicated test pass once the scaffolding above
   exists, rather than assuming it Just Works.
4. **UI adaptation**: the four-section bottom nav added in this pass
   (`ui::BottomNavBar`) was actually designed with a phone-sized layout
   in mind, so that part travels reasonably well; the desktop-only
   native dialogs (`rfd`, `arboard`, `notify-rust`) would each need an
   Android equivalent swapped in, matching the existing pattern noted in
   `Cargo.toml`'s target-specific dependency comment.

None of this is wired up today. If Android support is wanted next,
scaffolding the actual mobile build target is the prerequisite step
before any permissions work is meaningful.
