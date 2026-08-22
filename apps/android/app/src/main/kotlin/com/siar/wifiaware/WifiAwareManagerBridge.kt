package com.siar.wifiaware

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.net.wifi.aware.AttachCallback
import android.net.wifi.aware.DiscoverySessionCallback
import android.net.wifi.aware.PublishConfig
import android.net.wifi.aware.PublishDiscoverySession
import android.net.wifi.aware.SubscribeConfig
import android.net.wifi.aware.SubscribeDiscoverySession
import android.net.wifi.aware.WifiAwareManager
import android.net.wifi.aware.WifiAwareNetworkInfo
import android.net.wifi.aware.WifiAwareSession
import android.util.Log

/**
 * Drives `android.net.wifi.aware.WifiAwareManager` directly — same
 * risk-inversion pattern as [com.siar.wifidirect.WifiDirectManager];
 * see `siar-transport-wifi-aware`'s `lib.rs` doc comment for the full
 * rationale (shared with Wi-Fi Direct's, referenced from there).
 *
 * next.md §19: "Use Aware primarily for: nearby discovery, capability
 * advertisement, establishing direct data paths." This class exposes
 * both publish (advertise this device as reachable) and subscribe
 * (look for peers) — a real deployment picks one role per device
 * relationship, not both simultaneously against the same peer, though
 * nothing here enforces that; it's a scheduling decision left to
 * whatever calls this.
 */
class WifiAwareManagerBridge(private val context: Context) {
    private val manager: WifiAwareManager? =
        context.getSystemService(Context.WIFI_AWARE_SERVICE) as? WifiAwareManager
    private val connectivityManager: ConnectivityManager? =
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
    private var session: WifiAwareSession? = null
    private var nativeHandle: Long = 0

    /** Call once, typically from `MainActivity.onCreate`, before
     * [publish] or [subscribe]. No-ops if the device has no Wi-Fi
     * Aware hardware (`manager == null`) — the manifest's own
     * `<uses-feature android:required="false">` means this app must
     * run on such devices, just without this transport. */
    fun start() {
        nativeHandle = NativeWifiAwareBridge.createBridge()
        manager?.attach(object : AttachCallback() {
            override fun onAttached(attachedSession: WifiAwareSession) {
                session = attachedSession
            }

            override fun onAttachFailed() {
                Log.w(TAG, "Wi-Fi Aware attach failed")
            }
        }, null)
    }

    fun stop() {
        session?.close()
        session = null
        if (nativeHandle != 0L) {
            NativeWifiAwareBridge.destroyBridge(nativeHandle)
            nativeHandle = 0
        }
    }

    /** Advertises this device under `serviceName` — next.md §19's
     * publish role. */
    fun publish(serviceName: String) {
        val config = PublishConfig.Builder().setServiceName(serviceName).build()
        session?.publish(config, object : DiscoverySessionCallback() {
            override fun onPublishStarted(publishSession: PublishDiscoverySession) {
                Log.i(TAG, "Wi-Fi Aware publish started for $serviceName")
            }

            override fun onServiceLost(peerHandle: android.net.wifi.aware.PeerHandle, reason: Int) {
                NativeWifiAwareBridge.onDataPathLost(nativeHandle)
            }
        }, null)
    }

    /** Looks for peers advertising `serviceName` — next.md §19's
     * subscribe role. On a match, requests a data path; once that path
     * is actually up (not merely discovered), [onNetworkAvailable]
     * reports it into the native bridge. */
    fun subscribe(serviceName: String) {
        val config = SubscribeConfig.Builder().setServiceName(serviceName).build()
        // `onSubscribeStarted` and `onServiceDiscovered` are two
        // separate callback methods on the same anonymous
        // `DiscoverySessionCallback` — the `SubscribeDiscoverySession`
        // handle only arrives via the former, but
        // `createNetworkSpecifierOpen` (needed once a peer is actually
        // discovered) is only callable on that same session object.
        // Captured here as a `var` the callback object closes over,
        // rather than referencing the first callback's own parameter
        // name from inside the second (which doesn't compile — a
        // parameter isn't visible outside the method it belongs to).
        var activeSession: SubscribeDiscoverySession? = null
        session?.subscribe(config, object : DiscoverySessionCallback() {
            override fun onSubscribeStarted(subscribeSession: SubscribeDiscoverySession) {
                activeSession = subscribeSession
                Log.i(TAG, "Wi-Fi Aware subscribe started for $serviceName")
            }

            override fun onServiceDiscovered(
                peerHandle: android.net.wifi.aware.PeerHandle,
                serviceSpecificInfo: ByteArray?,
                matchFilter: MutableList<ByteArray>?,
            ) {
                val discoveredSession = activeSession ?: run {
                    Log.w(TAG, "service discovered before subscribe session was ready, dropping")
                    return
                }
                val networkSpecifier = discoveredSession.createNetworkSpecifierOpen(peerHandle)
                val request = NetworkRequest.Builder()
                    .addTransportType(NetworkCapabilities.TRANSPORT_WIFI_AWARE)
                    .setNetworkSpecifier(networkSpecifier)
                    .build()
                connectivityManager?.requestNetwork(request, networkCallback(isPublisher = false))
            }
        }, null)
    }

    /** Registered by the caller once a publish-side data-path request
     * is accepted, mirroring [subscribe]'s own network request — kept
     * as a caller-driven step rather than automatic, since accepting a
     * publish-side data-path request needs the `PeerHandle` from
     * `DiscoverySessionCallback.onMessageReceived`/an explicit accept
     * decision this class doesn't make on the app's behalf. */
    fun networkCallback(isPublisher: Boolean): ConnectivityManager.NetworkCallback =
        object : ConnectivityManager.NetworkCallback() {
            override fun onCapabilitiesChanged(network: Network, capabilities: NetworkCapabilities) {
                val info = capabilities.transportInfo as? WifiAwareNetworkInfo ?: return
                val address = info.peerIpv6Addr?.hostAddress ?: return
                // `siar-transport-wifi-aware`'s Rust side treats any
                // `port < 0` as "no port negotiated" (its own
                // jni_bridge.rs doc comment: Kotlin passes `-1` for
                // that case). What `WifiAwareNetworkInfo.getPort()`
                // itself actually returns when no port was negotiated
                // isn't clearly documented (checked; found no
                // authoritative source pinning it to -1 vs. 0) — so
                // this normalizes any non-positive value to -1 before
                // crossing the JNI boundary, rather than trust a
                // specific sentinel this pass couldn't verify.
                val port = if (info.port > 0) info.port else -1
                NativeWifiAwareBridge.onDataPathOpened(nativeHandle, isPublisher, address, port)
                com.siar.connectivity.ConnectivityBridge.markUp(com.siar.connectivity.TransportLinkKind.WifiAware)
            }

            override fun onLost(network: Network) {
                NativeWifiAwareBridge.onDataPathLost(nativeHandle)
                com.siar.connectivity.ConnectivityBridge.markDown(com.siar.connectivity.TransportLinkKind.WifiAware)
            }
        }

    companion object {
        private const val TAG = "WifiAwareManagerBridge"
    }
}

/**
 * JNI declarations matching `siar-transport-wifi-aware/src/jni_bridge.rs`
 * exactly (symbol `Java_com_siar_wifiaware_NativeWifiAwareBridge_*`).
 */
object NativeWifiAwareBridge {
    init {
        System.loadLibrary("siar_transport_wifi_aware")
    }

    external fun createBridge(): Long
    external fun destroyBridge(handle: Long)
    external fun onDataPathOpened(handle: Long, isPublisher: Boolean, peerIpv6Address: String, port: Int)
    external fun onDataPathLost(handle: Long)
}
