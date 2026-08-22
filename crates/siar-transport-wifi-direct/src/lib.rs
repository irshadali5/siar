//! JNI boundary for Android Wi-Fi Direct — next.md §15–17.
//!
//! Same risk-inversion pattern as `siar-media-android` (see that
//! crate's `lib.rs` doc comment for the full rationale): Kotlin's
//! `WifiDirectManager.kt` drives `android.net.wifi.p2p.WifiP2pManager`
//! directly — ordinary, statically-typed Kotlin against the Android
//! SDK, compiler-checked on that side. This crate is only ever *called
//! into* from Kotlin, never the reverse, for the same "loud
//! `UnsatisfiedLinkError` beats a silent runtime `NoSuchMethodError`"
//! reason.
//!
//! Different shape from `siar-media-android`'s per-session handle
//! model, deliberately: a phone has exactly one Wi-Fi radio, so
//! there's exactly one Wi-Fi Direct group state worth tracking per
//! process, not one per concurrent call the way encode/decode sessions
//! need. Still handle-based rather than a bare global, though — same
//! `jlong` pattern, just created once at app startup instead of once
//! per call.
//!
//! What this crate deliberately does NOT do: dial the discovered peer,
//! or know anything about *which* peer it is. Per next.md §17 ("This
//! is preferable to inventing another high-bandwidth protocol"), once
//! [`jni_bridge::WifiDirectGroupInfo`] reports an IP-level link exists,
//! the actual messenger traffic goes through the *existing*
//! `SiarEndpoint`/mDNS local discovery (`siar-transport`'s
//! `local_discovery.rs`) running over that new interface — this
//! crate's only job is reporting the link's lifecycle (up/down, role)
//! so something upstream can mark `TransportLink::WifiDirect` up in a
//! `siar_domain::ConnectivityState` (next.md §59–60's UI status), and
//! so Kotlin knows to hold a `WifiManager.MulticastLock` for as long as
//! the group exists — Android silently drops multicast traffic
//! (including mDNS) on a Wi-Fi-family interface without one held, a
//! well-known Android gotcha, not an iroh one. Acquiring/releasing that
//! lock is `WifiDirectManager.kt`'s job; it's noted here only because
//! it's *why* this link matters to mDNS at all.
//!
//! What's also NOT here yet, and worth being explicit about rather than
//! guessing at nonexistent glue code: wiring [`jni_bridge::group_info`]'s
//! output into a shared `siar_domain::ConnectivityState` that the rest
//! of the app reads. That's app-assembly work — this workspace doesn't
//! have an `apps/android` crate yet to own that shared state and poll
//! this. [`jni_bridge::group_info`] is the accessor whatever does end up
//! owning that state should call; it's `pub` for exactly that future
//! caller, same as `siar-media-android`'s `push_input` is `pub` for its
//! own not-yet-written caller.

#[cfg(target_os = "android")]
mod jni_bridge;
#[cfg(target_os = "android")]
pub use jni_bridge::{group_info, WifiDirectBridge, WifiDirectGroupInfo, WifiDirectRole};
