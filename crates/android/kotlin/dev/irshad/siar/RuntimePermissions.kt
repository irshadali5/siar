package dev.irshad.siar

import android.Manifest
import android.app.Activity
import android.content.pm.PackageManager
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat

/**
 * Runtime permission requests. Covers three groups:
 *
 *  - RECORD_AUDIO, CAMERA, POST_NOTIFICATIONS (13+): calling, as before.
 *  - BLUETOOTH_SCAN, BLUETOOTH_CONNECT, BLUETOOTH_ADVERTISE (31+):
 *    needed by `net::mesh::ble`'s scanning — see that module's doc
 *    comment for why advertising isn't actually wired up yet even
 *    though the permission is requested; requesting it now means the
 *    manifest/prompt won't need to change again once a peripheral
 *    backend lands.
 *  - NEARBY_WIFI_DEVICES (33+): Android's permission for apps that do
 *    their own local Wi-Fi networking (`net::mesh::lan`'s UDP
 *    broadcast) without going through a system Wi-Fi Direct API.
 *
 * All of the above are runtime-dangerous only from the API levels
 * noted — below them, the manifest declaration (already carried over
 * in `Dioxus.toml`'s `[bundle.android.permissions]`) is suficient on
 * its own, same as this file already documented for the original
 * three. None of this requests ACCESS_FINE_LOCATION: that's only
 * needed for BLE scanning on API < 31, and `Dioxus.toml`'s
 * `min_sdk_version = 34` floor means this app never runs there.
 *
 * Where this plugs in: call `RuntimePermissions.requestMissing(activity)`
 * from whatever Activity `dx` generates as this app's entry point, once
 * per cold start (e.g. its `onCreate`). Exactly which Activity that is,
 * and exactly when in its lifecycle is soonest-safe to call this, is part
 * of the bootstrap-glue uncertainty flagged in `siar-android/src/
 * main.rs` — this file doesn't guess at that, it just assumes *some*
 * Activity method will call `requestMissing`.
 */
object RuntimePermissions {

    private val REQUIRED = buildList {
        add(Manifest.permission.RECORD_AUDIO)
        add(Manifest.permission.CAMERA)

        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
            add(Manifest.permission.POST_NOTIFICATIONS)
            add(Manifest.permission.NEARBY_WIFI_DEVICES)
        }
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
            add(Manifest.permission.BLUETOOTH_SCAN)
            add(Manifest.permission.BLUETOOTH_CONNECT)
            add(Manifest.permission.BLUETOOTH_ADVERTISE)
        }
    }.toTypedArray()

    const val REQUEST_CODE = 4200

    /** True once every permission in [REQUIRED] has been granted. */
    fun allGranted(activity: Activity): Boolean =
        REQUIRED.all {
            ContextCompat.checkSelfPermission(activity, it) == PackageManager.PERMISSION_GRANTED
        }

    /**
     * Requests whichever permissions in [REQUIRED] aren't already granted.
     * The request is ignored if nothing is missing or the Activity is finishing.
     */
    fun requestMissing(activity: Activity) {
        if (activity.isFinishing || activity.isDestroyed) return

        val missing = REQUIRED.filter {
            ContextCompat.checkSelfPermission(activity, it) != PackageManager.PERMISSION_GRANTED
        }
        if (missing.isNotEmpty()) {
            ActivityCompat.requestPermissions(activity, missing.toTypedArray(), REQUEST_CODE)
        }
    }
}
