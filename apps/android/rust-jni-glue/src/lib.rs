//! Owns the one `siar_domain::ConnectivityState` shared
//! across `apps/android` — the piece `siar-transport-wifi-direct`,
//! `siar-transport-wifi-aware`, `siar-transport-ble-android`, and
//! `siar-transport-bluetooth-classic`'s own `lib.rs` doc comments each
//! independently named as missing ("this workspace doesn't have an
//! `apps/android` crate yet to own that shared state and poll this").
//! This crate is that owner, finally — see this workspace's own memory
//! of prior sessions for how long that gap sat open.
//!
//! Deliberately thin: this crate does NOT poll the four transport
//! bridges' `group_info()`/`session_info()` accessors itself. Kotlin
//! already knows exactly when a link event happens (it's the one
//! driving `WifiP2pManager`/`WifiAwareManager`/GATT callbacks/RFCOMM
//! sockets in the first place) — having Kotlin push [`mark_link_up`]/
//! [`mark_link_down`] here directly, right where each event fires, is
//! simpler and lower-latency than this crate polling four other crates'
//! state on a timer and computing a diff. The four transport crates'
//! `group_info`/`session_info` accessors stay `pub` and unused by this
//! crate for anyone who does want a pull-based reader instead.
//!
//! JNI naming: package `com.siar.connectivity`, class
//! `NativeConnectivityBridge` → symbol prefix
//! `Java_com_siar_connectivity_NativeConnectivityBridge_`, same
//! convention as every `jni_bridge.rs` in this workspace. Unlike the
//! four transport crates, there's exactly one `ConnectivityState` for
//! the whole process — no `handle`/`createBridge` here, just a
//! `static` behind a `Mutex`, since there's nothing per-connection or
//! per-radio about "what's up right now" (next.md §59's own framing:
//! one state, however many links happen to feed it).
//!
//! [`mark_link_up`]/[`mark_link_down`]/[`snapshot`] are also plain
//! `pub` Rust functions, not only reachable via JNI — `siar-android-
//! messaging` (a later pass) depends on this crate directly (a normal
//! Cargo dependency; this crate's `rlib` output, not its `cdylib`) so
//! it can report its own `SiarEndpoint`'s connectivity without an
//! unnecessary JNI round-trip through Kotlin and back. Kotlin remains
//! the *other* real caller, for the four transport bridges it drives
//! directly — this crate has two real callers now, one Kotlin, one
//! Rust, which is exactly why the functions were already plain `pub`
//! rather than `pub(crate)` from the start.

use siar_domain::{ConnectivityState, TransportLink};
use std::sync::{Mutex, OnceLock};

static STATE: OnceLock<Mutex<ConnectivityState>> = OnceLock::new();

fn state() -> &'static Mutex<ConnectivityState> {
    STATE.get_or_init(|| Mutex::new(ConnectivityState::new()))
}

/// Not part of the JNI surface — the accessor a future Rust-side reader
/// (a routing engine, `siar_routing::path::PathTable`'s eventual
/// consumer on this platform) would call. `pub` for that not-yet-
/// written caller, same "built, no real caller yet" shape as this
/// crate's own top doc comment names for the four transport crates'
/// accessors.
pub fn snapshot() -> ConnectivityState {
    state()
        .lock()
        .expect("ConnectivityState lock poisoned")
        .clone()
}

/// Marks `link` up in the shared `ConnectivityState` — the plain-Rust
/// entry point both Kotlin (via the JNI wrapper below) and
/// `siar-android-messaging` (as a direct crate dependency) call. See
/// this module's top doc comment for why both are real callers now.
pub fn mark_link_up(link: TransportLink) {
    state()
        .lock()
        .expect("ConnectivityState lock poisoned")
        .mark_up(link);
}

/// The `mark_link_up` counterpart — see that function's own doc
/// comment.
pub fn mark_link_down(link: TransportLink) {
    state()
        .lock()
        .expect("ConnectivityState lock poisoned")
        .mark_down(link);
}

/// Not part of the JNI surface itself — shared by the two `markLink*`
/// JNI functions below (both `#[cfg(target_os = "android")]`) and by
/// this module's own tests. On a plain (non-Android) `cargo build`,
/// neither of those callers is compiled in, so this would otherwise
/// warn as dead code despite being real, exercised logic on the
/// platform this crate actually ships on — the `cfg_attr` below
/// reflects that precisely rather than blanket-suppressing the lint.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn link_from_ordinal(ordinal: i32) -> Option<TransportLink> {
    // Ordinal mapping kept explicit and in one place (not `#[repr]`
    // tricks on the Rust enum) — see `NativeConnectivityBridge.kt`'s
    // own `TransportLinkKind` for the Kotlin-side half of this contract
    // both files must agree on.
    use TransportLink::*;
    match ordinal {
        0 => Some(InternetDirect),
        1 => Some(InternetRelay),
        2 => Some(LocalLan),
        3 => Some(WifiDirect),
        4 => Some(WifiAware),
        5 => Some(BluetoothClassic),
        6 => Some(Ble),
        _ => None,
    }
}

#[cfg(target_os = "android")]
mod jni_bridge {
    use super::{link_from_ordinal, mark_link_down, mark_link_up, state};
    use jni::objects::JClass;
    use jni::sys::jint;
    use jni::JNIEnv;

    #[no_mangle]
    pub extern "system" fn Java_com_siar_connectivity_NativeConnectivityBridge_markLinkUp<
        'local,
    >(
        _env: JNIEnv<'local>,
        _class: JClass<'local>,
        link_ordinal: jint,
    ) {
        let Some(link) = link_from_ordinal(link_ordinal) else {
            return;
        };
        mark_link_up(link);
    }

    #[no_mangle]
    pub extern "system" fn Java_com_siar_connectivity_NativeConnectivityBridge_markLinkDown<
        'local,
    >(
        _env: JNIEnv<'local>,
        _class: JClass<'local>,
        link_ordinal: jint,
    ) {
        let Some(link) = link_from_ordinal(link_ordinal) else {
            return;
        };
        mark_link_down(link);
    }

    /// Returns `siar_domain::EffectiveConnectivity`'s
    /// ordinal for whatever Kotlin wants to show as the single-line
    /// connectivity summary (next.md §60) — same ordinal-mapping
    /// contract as the two `markLink*` functions above, mirrored in
    /// `NativeConnectivityBridge.kt`'s `EffectiveConnectivityKind`.
    #[no_mangle]
    pub extern "system" fn Java_com_siar_connectivity_NativeConnectivityBridge_effectiveModeOrdinal<
        'local,
    >(
        _env: JNIEnv<'local>,
        _class: JClass<'local>,
    ) -> jint {
        use siar_domain::EffectiveConnectivity::*;
        match state()
            .lock()
            .expect("ConnectivityState lock poisoned")
            .effective_mode()
        {
            InternetDirect => 0,
            InternetRelay => 1,
            LocalLan => 2,
            WifiPeerToPeer => 3,
            BluetoothDirect => 4,
            Isolated => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Uses `snapshot()`/direct `state()` access rather than the JNI
    // functions themselves (those are `#[cfg(target_os = "android")]`
    // and untestable outside that target, same limitation every JNI
    // bridge in this workspace has) — this still exercises the same
    // underlying logic those functions call into.

    #[test]
    fn link_from_ordinal_round_trips_every_defined_transport_link() {
        for ordinal in 0..=6 {
            assert!(
                link_from_ordinal(ordinal).is_some(),
                "ordinal {ordinal} should map to a TransportLink"
            );
        }
        assert_eq!(link_from_ordinal(7), None);
        assert_eq!(link_from_ordinal(-1), None);
    }

    #[test]
    fn marking_a_link_up_is_reflected_in_a_snapshot() {
        state().lock().unwrap().mark_up(TransportLink::LocalLan);
        assert!(snapshot().is_up(TransportLink::LocalLan));
        // Cleanup — this module's `state()` is a process-wide static,
        // so this test must leave it as it found it for any test that
        // runs after it in the same process.
        state().lock().unwrap().mark_down(TransportLink::LocalLan);
    }
}
