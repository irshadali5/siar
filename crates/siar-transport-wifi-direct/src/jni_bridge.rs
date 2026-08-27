//! See this crate's `lib.rs` doc comment for scope and the
//! risk-inversion rationale. JNI naming: package `com.siar.wifidirect`,
//! class `NativeWifiDirectBridge` → symbol prefix
//! `Java_com_siar_wifidirect_NativeWifiDirectBridge_`, same convention
//! `siar-media-android`'s `jni_bridge.rs` uses (see that file's doc
//! comment for why: no underscores in any Kotlin method name below, so
//! none of JNI's `_1` escaping applies).
//!
//! `handle` (`jlong` on the Kotlin side) is
//! `Box::into_raw(Box::new(Mutex::new(bridge))) as jlong`, reclaimed
//! exactly once by `destroyBridge` — same one-alloc/one-free contract
//! as `siar-media-android`'s `MediaSession`, and the same caveat: this
//! defends against forgetting to free it, not against a forged or
//! reused handle, which isn't a concern for our own Kotlin calling into
//! our own Rust.

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jlong};
use jni::JNIEnv;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiDirectRole {
    GroupOwner,
    Client,
}

#[derive(Debug, Clone)]
pub struct WifiDirectGroupInfo {
    pub role: WifiDirectRole,
    /// The group owner's address on the new P2P interface
    /// (`WifiP2pInfo.groupOwnerAddress.hostAddress` on the Kotlin side),
    /// kept as a string rather than parsed into `std::net::IpAddr`
    /// here — this crate has no use for it beyond handing it to
    /// whatever reads [`group_info`] next; parsing it is that caller's
    /// job if it needs to, not this boundary's.
    pub group_owner_address: String,
}

pub struct WifiDirectBridge {
    group: Option<WifiDirectGroupInfo>,
}

#[no_mangle]
pub extern "system" fn Java_com_siar_wifidirect_NativeWifiDirectBridge_createBridge<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jlong {
    Box::into_raw(Box::new(Mutex::new(WifiDirectBridge { group: None }))) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_siar_wifidirect_NativeWifiDirectBridge_destroyBridge<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    if handle == 0 {
        return;
    }
    // SAFETY: reclaims exactly the box `createBridge` allocated for this
    // handle — see this file's top doc comment.
    unsafe {
        drop(Box::from_raw(handle as *mut Mutex<WifiDirectBridge>));
    }
}

/// # Safety
/// `handle` must be zero, or a value `createBridge` returned that
/// `destroyBridge` hasn't been called on yet — see this file's top doc
/// comment.
unsafe fn bridge_from_handle<'a>(handle: jlong) -> Option<&'a Mutex<WifiDirectBridge>> {
    if handle == 0 {
        return None;
    }
    Some(unsafe { &*(handle as *const Mutex<WifiDirectBridge>) })
}

/// Kotlin calls this from `WifiP2pManager.ConnectionInfoListener` once
/// `WifiP2pInfo.groupFormed` is true — the IP-level link next.md §17
/// says the existing messenger protocol should just run over, once
/// mDNS (with the multicast lock held — see `lib.rs`'s doc comment)
/// picks the peer up on this new interface.
#[no_mangle]
pub extern "system" fn Java_com_siar_wifidirect_NativeWifiDirectBridge_onGroupFormed<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    is_group_owner: jboolean,
    group_owner_address: JString<'local>,
) {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else {
        return;
    };
    let address: String = env
        .get_string(&group_owner_address)
        .map(String::from)
        .unwrap_or_else(|_| "<unreadable group owner address>".to_string());
    let role = if is_group_owner != 0 {
        WifiDirectRole::GroupOwner
    } else {
        WifiDirectRole::Client
    };

    bridge.lock().expect("WifiDirectBridge lock poisoned").group = Some(WifiDirectGroupInfo {
        role,
        group_owner_address: address,
    });
}

/// Kotlin calls this from the `WIFI_P2P_CONNECTION_CHANGED_ACTION`
/// broadcast receiver once `WifiP2pInfo.groupFormed` goes false — group
/// torn down, teardown, or the peer walked out of range.
#[no_mangle]
pub extern "system" fn Java_com_siar_wifidirect_NativeWifiDirectBridge_onGroupLost<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else {
        return;
    };
    bridge.lock().expect("WifiDirectBridge lock poisoned").group = None;
}

/// Not part of the JNI surface — see `lib.rs`'s doc comment on why
/// wiring this into a shared `ConnectivityState` isn't done yet.
/// `pub` for that future caller to poll (or wrap in its own
/// notify/subscribe mechanism, same shape as `siar-media-android`'s
/// `output_ready_notifier` if polling turns out to be the wrong
/// choice once there's a real caller to write against).
pub fn group_info(bridge: &Mutex<WifiDirectBridge>) -> Option<WifiDirectGroupInfo> {
    bridge
        .lock()
        .expect("WifiDirectBridge lock poisoned")
        .group
        .clone()
}
