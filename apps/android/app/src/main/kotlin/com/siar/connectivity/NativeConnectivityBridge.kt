package com.siar.connectivity

/**
 * Ordinal mapping must match `siar-android-connectivity`'s
 * `link_from_ordinal` exactly — see that function's own doc comment.
 * Kept as a Kotlin enum (not raw ints scattered at call sites) so a
 * caller writes `TransportLinkKind.WifiDirect.ordinal`, not a bare `3`
 * whose meaning isn't visible at the call site.
 */
enum class TransportLinkKind {
    InternetDirect,
    InternetRelay,
    LocalLan,
    WifiDirect,
    WifiAware,
    BluetoothClassic,
    Ble,
}

/** Ordinal mapping must match the Rust side's `effectiveModeOrdinal`
 * match arms exactly. */
enum class EffectiveConnectivityKind {
    InternetDirect,
    InternetRelay,
    LocalLan,
    WifiPeerToPeer,
    BluetoothDirect,
    Isolated,
}

/**
 * Thin Kotlin wrapper — every other bridge class in this app
 * (`WifiDirectManager`, `WifiAwareManagerBridge`, `BleGattManager`,
 * `BluetoothClassicManager`) calls [markUp]/[markDown] directly from
 * its own connection-lifecycle callbacks, right where each event fires
 * — see `siar-android-connectivity`'s own crate doc comment for why
 * this is push-based rather than this bridge polling the other four.
 */
object ConnectivityBridge {
    fun markUp(link: TransportLinkKind) = NativeConnectivityBridge.markLinkUp(link.ordinal)
    fun markDown(link: TransportLinkKind) = NativeConnectivityBridge.markLinkDown(link.ordinal)

    fun effectiveMode(): EffectiveConnectivityKind =
        EffectiveConnectivityKind.entries[NativeConnectivityBridge.effectiveModeOrdinal()]
}

/**
 * JNI declarations matching
 * `apps/android/rust-jni-glue/src/lib.rs` exactly (symbol
 * `Java_com_siar_connectivity_NativeConnectivityBridge_*`).
 * `System.loadLibrary` name matches that crate's Cargo.toml package
 * name (hyphens to underscores): `siar_android_connectivity`.
 */
private object NativeConnectivityBridge {
    init {
        System.loadLibrary("siar_android_connectivity")
    }

    external fun markLinkUp(linkOrdinal: Int)
    external fun markLinkDown(linkOrdinal: Int)
    external fun effectiveModeOrdinal(): Int
}
