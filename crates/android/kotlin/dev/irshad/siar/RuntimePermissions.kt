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
 * Permissions are requested at the moment a person enables the related
 * feature, rather than showing an intimidating camera/microphone/nearby
 * device wall on first launch. The app's API 26 floor means BLE discovery
 * on Android 8-11 still needs location permission, so nearby mesh remains
 * Wi-Fi-only there; Android 12+ uses the dedicated Bluetooth permissions.
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

    const val REQUEST_CODE = 4200

    fun requestAudio(activity: Activity) = request(activity, listOf(Manifest.permission.RECORD_AUDIO))

    fun requestNotifications(activity: Activity) {
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
            request(activity, listOf(Manifest.permission.POST_NOTIFICATIONS))
        }
    }

    fun requestNearby(activity: Activity) {
        val permissions = buildList {
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
                add(Manifest.permission.NEARBY_WIFI_DEVICES)
            }
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
                add(Manifest.permission.BLUETOOTH_SCAN)
                add(Manifest.permission.BLUETOOTH_CONNECT)
                add(Manifest.permission.BLUETOOTH_ADVERTISE)
            }
        }
        request(activity, permissions)
    }

    private fun request(activity: Activity, permissions: List<String>) {
        if (activity.isFinishing || activity.isDestroyed) return

        val missing = permissions.filter {
            ContextCompat.checkSelfPermission(activity, it) != PackageManager.PERMISSION_GRANTED
        }
        if (missing.isNotEmpty()) {
            ActivityCompat.requestPermissions(activity, missing.toTypedArray(), REQUEST_CODE)
        }
    }
}
