//! JNI boundary for Android Wi-Fi Aware (NAN) — next.md §18–20.
//!
//! Same risk-inversion pattern as `siar-transport-wifi-direct` (see
//! that crate's `lib.rs` doc comment for the full rationale, which
//! applies here unchanged): Kotlin's `WifiAwareManager.kt` drives
//! `android.net.wifi.aware.WifiAwareManager` directly — ordinary,
//! statically-typed Kotlin against the Android SDK. This crate is only
//! ever *called into* from Kotlin, never the reverse.
//!
//! Deliberately mirrors `siar-transport-wifi-direct`'s shape (one
//! process-lifetime handle, not per-session) but the role vocabulary is
//! Aware's own: next.md §19 says "Use Aware primarily for: nearby
//! discovery, capability advertisement, establishing direct data
//! paths" via **publish/subscribe**, not Wi-Fi Direct's group-owner
//! negotiation. A device is either publishing a service (advertising
//! itself as reachable) or subscribing to one (looking for peers) —
//! see [`jni_bridge::WifiAwareRole`].
//!
//! What this crate deliberately does NOT do, same as
//! `siar-transport-wifi-direct`: know anything about *which* peer was
//! discovered, or carry any messenger traffic itself. Per next.md §19
//! ("Then move normal protocol traffic through the established IP data
//! path"), once [`jni_bridge::WifiAwareSessionInfo`] reports a data
//! path is up, traffic goes through the existing
//! `SiarEndpoint`/mDNS local discovery running over that interface —
//! same multicast-lock caveat as Wi-Fi Direct's `lib.rs` doc comment
//! describes, held by `WifiAwareManager.kt`, noted here only because
//! it's why this link matters to mDNS at all.
//!
//! Also not here yet, same as Wi-Fi Direct: wiring
//! [`jni_bridge::session_info`]'s output into a shared
//! `siar_domain::ConnectivityState`. This workspace has no
//! `apps/android` crate yet to own that shared state and poll this.
//! [`jni_bridge::session_info`] is `pub` for that future caller.

#[cfg(target_os = "android")]
mod jni_bridge;
#[cfg(target_os = "android")]
pub use jni_bridge::{session_info, WifiAwareBridge, WifiAwareRole, WifiAwareSessionInfo};
