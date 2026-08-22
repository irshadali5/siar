package com.siar.wifidirect

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.wifi.WifiManager
import android.net.wifi.p2p.WifiP2pConfig
import android.net.wifi.p2p.WifiP2pDevice
import android.net.wifi.p2p.WifiP2pInfo
import android.net.wifi.p2p.WifiP2pManager
import android.os.Build
import android.util.Log

/**
 * Drives `android.net.wifi.p2p.WifiP2pManager` directly — this is the
 * Kotlin half of the risk-inversion pattern `siar-transport-wifi-
 * direct`'s own `lib.rs` doc comment describes: this class calls
 * *into* [NativeWifiDirectBridge]'s JNI methods, never the reverse, so
 * an `UnsatisfiedLinkError` on a missing/renamed native symbol fails
 * loudly at class-load time rather than silently at first use.
 *
 * Deliberately does NOT dial a discovered peer or attempt to identify
 * *which* peer it is — matches `siar-transport-wifi-direct`'s own
 * documented scope exactly (next.md §17): once [onGroupFormed] fires,
 * the actual messenger traffic is expected to reuse the existing
 * `SiarEndpoint`/mDNS local discovery over the new P2P interface, which
 * needs the multicast lock held in [acquireMulticastLock] for as long
 * as the group exists.
 */
class WifiDirectManager(private val context: Context) {
    private val manager: WifiP2pManager? =
        context.getSystemService(Context.WIFI_P2P_SERVICE) as? WifiP2pManager
    private var channel: WifiP2pManager.Channel? = null
    private var multicastLock: WifiManager.MulticastLock? = null
    private var nativeHandle: Long = 0

    private val connectionListener = WifiP2pManager.ConnectionInfoListener { info: WifiP2pInfo ->
        if (info.groupFormed) {
            val address = info.groupOwnerAddress?.hostAddress ?: "<unknown group owner address>"
            NativeWifiDirectBridge.onGroupFormed(nativeHandle, info.isGroupOwner, address)
            acquireMulticastLock()
            com.siar.connectivity.ConnectivityBridge.markUp(com.siar.connectivity.TransportLinkKind.WifiDirect)
            Log.i(TAG, "Wi-Fi Direct group formed (owner=${info.isGroupOwner}, address=$address)")
        }
    }

    private val broadcastReceiver = object : BroadcastReceiver() {
        // `@Suppress` on this whole function rather than on the two
        // individual deprecated call sites inside it — Kotlin only
        // allows `@Suppress` on declarations (functions, classes,
        // `val`/`var`), not on an arbitrary expression used as an
        // `if`/`else` branch's value, which is what the
        // `getParcelableExtra` call below would otherwise need to be.
        // Both deprecations are explained inline where they occur.
        @Suppress("DEPRECATION")
        override fun onReceive(ctx: Context, intent: Intent) {
            when (intent.action) {
                WifiP2pManager.WIFI_P2P_CONNECTION_CHANGED_ACTION -> {
                    // `Intent.getParcelableExtra(String)` (single-arg) is
                    // deprecated since API 33 in favor of the type-safe
                    // `getParcelableExtra(String, Class<T>)` overload —
                    // real, fixable, version-gated below since the
                    // 2-arg overload doesn't exist pre-33 and this app's
                    // `minSdk` is 26.
                    val networkInfo = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        intent.getParcelableExtra(WifiP2pManager.EXTRA_NETWORK_INFO, android.net.NetworkInfo::class.java)
                    } else {
                        intent.getParcelableExtra(WifiP2pManager.EXTRA_NETWORK_INFO)
                    }
                    // `NetworkInfo`/`NetworkInfo.isConnected` are
                    // themselves deprecated workspace-wide since API 29
                    // (in favor of `ConnectivityManager.NetworkCallback`)
                    // — unlike the extraction above, this one has no
                    // fix: `WIFI_P2P_CONNECTION_CHANGED_ACTION` only
                    // ever carries connection state via `EXTRA_NETWORK_INFO`,
                    // and the Wi-Fi P2P API surface itself hasn't been
                    // updated with a non-deprecated alternative for this
                    // specific broadcast.
                    val isConnected = networkInfo?.isConnected == true
                    if (isConnected) {
                        manager?.requestConnectionInfo(channel, connectionListener)
                    } else {
                        NativeWifiDirectBridge.onGroupLost(nativeHandle)
                        releaseMulticastLock()
                        com.siar.connectivity.ConnectivityBridge.markDown(com.siar.connectivity.TransportLinkKind.WifiDirect)
                        Log.i(TAG, "Wi-Fi Direct group lost")
                    }
                }
            }
        }
    }

    /** Call once, typically from `MainActivity.onCreate`. */
    fun start() {
        nativeHandle = NativeWifiDirectBridge.createBridge()
        channel = manager?.initialize(context, context.mainLooper, null)
        context.registerReceiver(
            broadcastReceiver,
            IntentFilter(WifiP2pManager.WIFI_P2P_CONNECTION_CHANGED_ACTION),
        )
    }

    /** Call from `MainActivity.onDestroy` — releases both the JNI
     * handle and the multicast lock if still held, mirroring
     * `NativeWifiDirectBridge.destroyBridge`'s "exactly once" contract
     * documented on the Rust side. */
    fun stop() {
        runCatching { context.unregisterReceiver(broadcastReceiver) }
        releaseMulticastLock()
        if (nativeHandle != 0L) {
            NativeWifiDirectBridge.destroyBridge(nativeHandle)
            nativeHandle = 0
        }
    }

    /**
     * Begins peer discovery — now actually guarded by
     * [com.siar.messenger.PermissionsHelper.hasNearbyWifiDevices]
     * rather than just documented as a precondition (`ACCESS_FINE_LOCATION`
     * pre-API-33, `NEARBY_WIFI_DEVICES` on API 33+ — see the manifest's
     * own comment on why both exist).
     */
    fun discoverPeers(onFailure: (reason: Int) -> Unit = {}) {
        if (!com.siar.messenger.PermissionsHelper.hasNearbyWifiDevices(context)) {
            Log.w(TAG, "nearby Wi-Fi devices permission not granted, skipping peer discovery")
            return
        }
        manager?.discoverPeers(channel, object : WifiP2pManager.ActionListener {
            override fun onSuccess() = Unit
            override fun onFailure(reason: Int) = onFailure(reason)
        })
    }

    fun connect(device: WifiP2pDevice, onFailure: (reason: Int) -> Unit = {}) {
        if (!com.siar.messenger.PermissionsHelper.hasNearbyWifiDevices(context)) {
            Log.w(TAG, "nearby Wi-Fi devices permission not granted, skipping connect")
            return
        }
        val config = WifiP2pConfig().apply { deviceAddress = device.deviceAddress }
        manager?.connect(channel, config, object : WifiP2pManager.ActionListener {
            override fun onSuccess() = Unit
            override fun onFailure(reason: Int) = onFailure(reason)
        })
    }

    private fun acquireMulticastLock() {
        if (multicastLock?.isHeld == true) return
        val wifiManager = context.getSystemService(Context.WIFI_SERVICE) as? WifiManager ?: return
        multicastLock = wifiManager.createMulticastLock("siar-wifi-direct-mdns").apply {
            setReferenceCounted(true)
            acquire()
        }
    }

    private fun releaseMulticastLock() {
        multicastLock?.let { if (it.isHeld) it.release() }
        multicastLock = null
    }

    companion object {
        private const val TAG = "WifiDirectManager"
    }
}

/**
 * JNI declarations — package/class name and every method signature
 * here must match `siar-transport-wifi-direct/src/jni_bridge.rs`
 * exactly (symbol `Java_com_siar_wifidirect_NativeWifiDirectBridge_*`).
 * `System.loadLibrary` name matches that crate's `[lib] name` in its
 * Cargo.toml (Cargo's default: the crate name with hyphens replaced by
 * underscores).
 */
object NativeWifiDirectBridge {
    init {
        System.loadLibrary("siar_transport_wifi_direct")
    }

    external fun createBridge(): Long
    external fun destroyBridge(handle: Long)
    external fun onGroupFormed(handle: Long, isGroupOwner: Boolean, groupOwnerAddress: String)
    external fun onGroupLost(handle: Long)
}
