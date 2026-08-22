//! See this crate's `lib.rs` doc comment for scope and the
//! risk-inversion rationale. JNI naming: package `com.siar.wifiaware`,
//! class `NativeWifiAwareBridge` → symbol prefix
//! `Java_com_siar_wifiaware_NativeWifiAwareBridge_`, same convention as
//! every other `jni_bridge.rs` in this workspace.
//!
//! `handle` contract (`Box::into_raw`/`Box::from_raw`, one alloc/one
//! free) is identical to `siar-transport-wifi-direct`'s — see that
//! crate's `jni_bridge.rs` for the full explanation, not repeated here.
//!
//! Address type note: Wi-Fi Aware data paths are IPv6-only
//! (Aware-specific NDP, not a regular Wi-Fi association) — Android's
//! `android.net.wifi.aware.WifiAwareNetworkInfo` exposes
//! `getPeerIpv6Addr()`/`getPort()` rather than the IPv4
//! `groupOwnerAddress` that Wi-Fi Direct's `WifiP2pInfo` uses. Kept as
//! a string here for the same reason Wi-Fi Direct's bridge keeps its
//! address as a string: parsing is whatever reads [`session_info`]
//! next's job, not this boundary's.

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong};
use jni::JNIEnv;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiAwareRole {
    /// This device advertised a service (`PublishConfig`) and a
    /// subscriber's data-path request was accepted.
    Publisher,
    /// This device discovered a peer's advertised service
    /// (`SubscribeConfig`) and requested a data path to it.
    Subscriber,
}

#[derive(Debug, Clone)]
pub struct WifiAwareSessionInfo {
    pub role: WifiAwareRole,
    /// `WifiAwareNetworkInfo.getPeerIpv6Addr()` on the Kotlin side —
    /// see this file's top doc comment on why Aware addresses are
    /// IPv6, unlike Wi-Fi Direct's.
    pub peer_ipv6_address: String,
    /// `WifiAwareNetworkInfo.getPort()`, present when the peer side
    /// negotiated a passthrough port (out-of-band, via the publish/
    /// subscribe service-specific info) rather than leaving discovery
    /// purely for `SiarEndpoint`'s own mDNS to take over. `None` when
    /// no port was negotiated at the Aware layer, which is the
    /// expected common case per this crate's `lib.rs` doc comment
    /// ("move normal protocol traffic through the established IP data
    /// path" via the existing mDNS-based discovery, not via Aware's
    /// own port negotiation).
    pub port: Option<u16>,
}

pub struct WifiAwareBridge {
    session: Option<WifiAwareSessionInfo>,
}

#[no_mangle]
pub extern "system" fn Java_com_siar_wifiaware_NativeWifiAwareBridge_createBridge<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jlong {
    Box::into_raw(Box::new(Mutex::new(WifiAwareBridge { session: None }))) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_siar_wifiaware_NativeWifiAwareBridge_destroyBridge<'local>(
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
        drop(Box::from_raw(handle as *mut Mutex<WifiAwareBridge>));
    }
}

/// # Safety
/// `handle` must be zero, or a value `createBridge` returned that
/// `destroyBridge` hasn't been called on yet — see this file's top doc
/// comment.
unsafe fn bridge_from_handle<'a>(handle: jlong) -> Option<&'a Mutex<WifiAwareBridge>> {
    if handle == 0 {
        return None;
    }
    Some(unsafe { &*(handle as *const Mutex<WifiAwareBridge>) })
}

/// Kotlin calls this from `ConnectivityManager.NetworkCallback.onCapabilitiesChanged`
/// once `NetworkCapabilities.getTransportInfo()` yields a
/// `WifiAwareNetworkInfo` for the requested network — i.e. once the
/// Aware data path is actually up, not merely once a service was
/// discovered. `port` is `-1` from Kotlin when Aware negotiated no
/// passthrough port (see [`WifiAwareSessionInfo::port`]'s doc comment).
#[no_mangle]
pub extern "system" fn Java_com_siar_wifiaware_NativeWifiAwareBridge_onDataPathOpened<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    is_publisher: jboolean,
    peer_ipv6_address: JString<'local>,
    port: jint,
) {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else { return };
    let address: String = env
        .get_string(&peer_ipv6_address)
        .map(String::from)
        .unwrap_or_else(|_| "<unreadable peer ipv6 address>".to_string());
    let role = if is_publisher != 0 { WifiAwareRole::Publisher } else { WifiAwareRole::Subscriber };
    let port = if port >= 0 { Some(port as u16) } else { None };

    bridge.lock().expect("WifiAwareBridge lock poisoned").session =
        Some(WifiAwareSessionInfo { role, peer_ipv6_address: address, port });
}

/// Kotlin calls this from `NetworkCallback.onLost` for the Aware
/// network request — data path torn down, session ended, or the peer
/// went out of range of the Aware cluster.
#[no_mangle]
pub extern "system" fn Java_com_siar_wifiaware_NativeWifiAwareBridge_onDataPathLost<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else { return };
    bridge.lock().expect("WifiAwareBridge lock poisoned").session = None;
}

/// Not part of the JNI surface — see `lib.rs`'s doc comment on why
/// wiring this into a shared `ConnectivityState` isn't done yet. `pub`
/// for that future caller, same as `siar-transport-wifi-direct`'s
/// `group_info`.
pub fn session_info(bridge: &Mutex<WifiAwareBridge>) -> Option<WifiAwareSessionInfo> {
    bridge.lock().expect("WifiAwareBridge lock poisoned").session.clone()
}
