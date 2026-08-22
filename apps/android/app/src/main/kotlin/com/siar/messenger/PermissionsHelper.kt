package com.siar.messenger

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.content.ContextCompat

/** Every dangerous permission this app's transports need, gated by API
 * level where the requirement itself is API-level-gated — see
 * `AndroidManifest.xml`'s own comments on why each one is declared.
 *
 * The granular `has*` checkers below (added alongside the manifest's
 * `ACCESS_COARSE_LOCATION`/`NEARBY_WIFI_DEVICES` additions) are what
 * each bridge class (`BleGattManager`, `BluetoothClassicManager`,
 * `WifiAwareManagerBridge`, `WifiDirectManager`) now guards its
 * dangerous-permission-gated platform calls with — real runtime checks,
 * not just lint appeasement: Bluetooth/location permissions can
 * genuinely be revoked by the user while this app is running, and a
 * call made after that happens throws a real `SecurityException`
 * these checks prevent. */
object PermissionsHelper {
    fun requiredPermissions(): Array<String> {
        val permissions = mutableListOf(Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.ACCESS_COARSE_LOCATION)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            permissions += Manifest.permission.BLUETOOTH_SCAN
            permissions += Manifest.permission.BLUETOOTH_ADVERTISE
            permissions += Manifest.permission.BLUETOOTH_CONNECT
        } else {
            permissions += Manifest.permission.BLUETOOTH
            permissions += Manifest.permission.BLUETOOTH_ADMIN
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            permissions += Manifest.permission.NEARBY_WIFI_DEVICES
        }
        return permissions.toTypedArray()
    }

    fun hasAllRequiredPermissions(context: Context): Boolean =
        requiredPermissions().all {
            ContextCompat.checkSelfPermission(context, it) == PackageManager.PERMISSION_GRANTED
        }

    private fun has(context: Context, permission: String): Boolean =
        ContextCompat.checkSelfPermission(context, permission) == PackageManager.PERMISSION_GRANTED

    /** Gates `BluetoothLeScanner`/GATT-server calls that scan/discover —
     * `BLUETOOTH_SCAN` on API 31+, the pre-split `BLUETOOTH`/
     * `BLUETOOTH_ADMIN` pair below it. */
    fun hasBluetoothScan(context: Context): Boolean =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            has(context, Manifest.permission.BLUETOOTH_SCAN)
        } else {
            has(context, Manifest.permission.BLUETOOTH_ADMIN)
        }

    /** Gates `BluetoothLeAdvertiser`/GATT-server advertise calls. */
    fun hasBluetoothAdvertise(context: Context): Boolean =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            has(context, Manifest.permission.BLUETOOTH_ADVERTISE)
        } else {
            has(context, Manifest.permission.BLUETOOTH_ADMIN)
        }

    /** Gates GATT connect/read/write/socket-connect calls (the largest
     * category — `MissingPermission`'s 12 `BLUETOOTH_CONNECT` sites
     * were all this one permission). */
    fun hasBluetoothConnect(context: Context): Boolean =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            has(context, Manifest.permission.BLUETOOTH_CONNECT)
        } else {
            has(context, Manifest.permission.BLUETOOTH)
        }

    /** Gates Wi-Fi Direct/Wi-Fi Aware peer discovery/connect calls —
     * `NEARBY_WIFI_DEVICES` on API 33+, `ACCESS_FINE_LOCATION` below
     * that (see `AndroidManifest.xml`'s own comment on why both are
     * declared). */
    fun hasNearbyWifiDevices(context: Context): Boolean =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            has(context, Manifest.permission.NEARBY_WIFI_DEVICES)
        } else {
            has(context, Manifest.permission.ACCESS_FINE_LOCATION)
        }
}
