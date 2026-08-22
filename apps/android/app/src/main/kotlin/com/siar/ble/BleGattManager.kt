package com.siar.ble

import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattServer
import android.bluetooth.BluetoothGattServerCallback
import android.bluetooth.BluetoothGattService
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothProfile
import android.bluetooth.le.AdvertiseCallback
import android.bluetooth.le.AdvertiseData
import android.bluetooth.le.AdvertiseSettings
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanResult
import android.content.Context
import android.os.Handler
import android.os.Looper
import android.util.Log
import java.util.UUID

/**
 * Drives `BluetoothLeScanner`/`BluetoothGattServer`/`BluetoothGatt`
 * directly — same risk-inversion pattern as every other bridge in this
 * app; see `siar-transport-ble-android`'s `jni_bridge.rs` doc comment
 * for the full rationale, shared unchanged.
 *
 * Per-connection [nativeHandle]s (a `Map<BluetoothDevice, Long>`), not
 * a singleton — matches that crate's own documented departure from
 * `WifiDirectManager`'s one-radio model: a phone can hold several
 * simultaneous BLE links.
 *
 * [pumpOutgoingFragments] is the "Kotlin pulls" half of the queue
 * pattern `jni_bridge.rs`'s own doc comment describes
 * (`nextFragmentToSend` returning `null` once nothing's queued) — this
 * class runs it on a fixed [Handler] tick rather than a callback the
 * Rust side has no way to raise (JNI here is call-in-only, per this
 * whole app's risk-inversion rule), same tradeoff `siar-media-android`'s
 * own `output_ready_notifier` doc comment already names for that
 * crate's identical shape. A production build would likely replace
 * this with a dedicated background thread rather than the main-looper
 * [Handler] used here for simplicity — noted, not fixed, since getting
 * threading right without a way to test it in this sandbox would be
 * guessing at a correctness property, not just a style choice.
 */
class BleGattManager(private val context: Context) {
    private val bluetoothManager =
        context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
    private val adapter: BluetoothAdapter? = bluetoothManager?.adapter
    private val nativeHandles = mutableMapOf<String, Long>() // device address -> handle
    private var gattServer: BluetoothGattServer? = null
    private val connectedGatts = mutableMapOf<String, BluetoothGatt>()
    private val pumpHandler = Handler(Looper.getMainLooper())
    private var pumping = false

    /** Reassembly capacity per connection — passed straight through to
     * `createBridge`'s `reassembly_capacity` param; see that function's
     * own doc comment for the `<= 0` fallback on the Rust side. */
    var reassemblyCapacityPerConnection: Int = 8

    fun startAdvertisingAndScanning(serviceUuid: UUID) {
        startGattServer(serviceUuid)
        advertise(serviceUuid)
        scan(serviceUuid)
        startPump()
    }

    fun stop() {
        pumping = false
        adapter?.bluetoothLeScanner?.stopScan(scanCallback)
        adapter?.bluetoothLeAdvertiser?.stopAdvertising(advertiseCallback)
        gattServer?.close()
        connectedGatts.values.forEach { it.close() }
        connectedGatts.clear()
        nativeHandles.values.forEach { NativeBleBridge.destroyBridge(it) }
        nativeHandles.clear()
    }

    private fun handleFor(device: BluetoothDevice): Long =
        nativeHandles.getOrPut(device.address) {
            NativeBleBridge.createBridge(reassemblyCapacityPerConnection)
        }

    private fun startGattServer(serviceUuid: UUID) {
        val service = BluetoothGattService(serviceUuid, BluetoothGattService.SERVICE_TYPE_PRIMARY)
        val characteristic = BluetoothGattCharacteristic(
            CHARACTERISTIC_UUID,
            BluetoothGattCharacteristic.PROPERTY_WRITE or BluetoothGattCharacteristic.PROPERTY_NOTIFY,
            BluetoothGattCharacteristic.PERMISSION_WRITE,
        )
        service.addCharacteristic(characteristic)

        gattServer = bluetoothManager?.openGattServer(context, object : BluetoothGattServerCallback() {
            override fun onConnectionStateChange(device: BluetoothDevice, status: Int, newState: Int) {
                if (newState == BluetoothProfile.STATE_DISCONNECTED) {
                    nativeHandles.remove(device.address)?.let { NativeBleBridge.destroyBridge(it) }
                    if (nativeHandles.isEmpty()) {
                        com.siar.connectivity.ConnectivityBridge.markDown(com.siar.connectivity.TransportLinkKind.Ble)
                    }
                } else if (newState == BluetoothProfile.STATE_CONNECTED && nativeHandles.isEmpty()) {
                    com.siar.connectivity.ConnectivityBridge.markUp(com.siar.connectivity.TransportLinkKind.Ble)
                }
            }

            override fun onCharacteristicWriteRequest(
                device: BluetoothDevice,
                requestId: Int,
                characteristic: BluetoothGattCharacteristic,
                preparedWrite: Boolean,
                responseNeeded: Boolean,
                offset: Int,
                value: ByteArray,
            ) {
                NativeBleBridge.onFragmentReceived(handleFor(device), value)
                if (responseNeeded) {
                    gattServer?.sendResponse(device, requestId, 0, offset, null)
                }
            }
        })
        gattServer?.addService(service)
    }

    private fun advertise(serviceUuid: UUID) {
        val settings = AdvertiseSettings.Builder()
            .setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY)
            .setConnectable(true)
            .build()
        val data = AdvertiseData.Builder()
            .addServiceUuid(android.os.ParcelUuid(serviceUuid))
            .setIncludeDeviceName(false) // next.md §23-ish framing budget — see discovery.rs's own note
            .build()
        adapter?.bluetoothLeAdvertiser?.startAdvertising(settings, data, advertiseCallback)
    }

    private val advertiseCallback = object : AdvertiseCallback() {
        override fun onStartFailure(errorCode: Int) {
            Log.w(TAG, "BLE advertise failed: $errorCode")
        }
    }

    private fun scan(serviceUuid: UUID) {
        val filter = android.bluetooth.le.ScanFilter.Builder()
            .setServiceUuid(android.os.ParcelUuid(serviceUuid))
            .build()
        val settings = android.bluetooth.le.ScanSettings.Builder()
            .setScanMode(android.bluetooth.le.ScanSettings.SCAN_MODE_LOW_LATENCY)
            .build()
        adapter?.bluetoothLeScanner?.startScan(listOf(filter), settings, scanCallback)
    }

    private val scanCallback = object : ScanCallback() {
        override fun onScanResult(callbackType: Int, result: ScanResult) {
            val device = result.device
            if (connectedGatts.containsKey(device.address)) return
            val gatt = device.connectGatt(context, false, gattCallback)
            connectedGatts[device.address] = gatt
        }
    }

    private val gattCallback = object : BluetoothGattCallback() {
        override fun onConnectionStateChange(gatt: BluetoothGatt, status: Int, newState: Int) {
            if (newState == BluetoothProfile.STATE_CONNECTED) {
                gatt.discoverServices()
                if (nativeHandles.isEmpty()) {
                    com.siar.connectivity.ConnectivityBridge.markUp(com.siar.connectivity.TransportLinkKind.Ble)
                }
            } else if (newState == BluetoothProfile.STATE_DISCONNECTED) {
                nativeHandles.remove(gatt.device.address)?.let { NativeBleBridge.destroyBridge(it) }
                connectedGatts.remove(gatt.device.address)
                // Only report the whole `Ble` link down once every
                // connection is gone — one dropped peer among several
                // shouldn't flip the aggregate connectivity summary,
                // same reasoning `ConnectivityState`'s own doc comment
                // gives for tracking a *set* of active links rather
                // than one boolean per transport.
                if (nativeHandles.isEmpty()) {
                    com.siar.connectivity.ConnectivityBridge.markDown(com.siar.connectivity.TransportLinkKind.Ble)
                }
            }
        }

        override fun onCharacteristicChanged(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
            value: ByteArray,
        ) {
            NativeBleBridge.onFragmentReceived(handleFor(gatt.device), value)
        }
    }

    /** Queues `envelopeBytes` (already-encrypted — this class, like the
     * Rust bridge it calls into, never looks inside it) for fragmented
     * delivery to `device`. `protocol` is passed straight through to
     * `BleFragment`'s own app-defined tag. */
    fun sendEnvelope(device: BluetoothDevice, envelopeBytes: ByteArray, protocol: Byte, maxFragmentBytes: Int) {
        NativeBleBridge.queueEnvelopeToSend(handleFor(device), envelopeBytes, protocol, maxFragmentBytes)
    }

    private fun startPump() {
        if (pumping) return
        pumping = true
        pumpOutgoingFragments()
    }

    private fun pumpOutgoingFragments() {
        if (!pumping) return
        for ((address, handle) in nativeHandles) {
            val gatt = connectedGatts[address]
            var fragment = NativeBleBridge.nextFragmentToSend(handle)
            while (fragment != null) {
                gatt?.let { writeFragment(it, fragment) }
                fragment = NativeBleBridge.nextFragmentToSend(handle)
            }
            var received = NativeBleBridge.nextReceivedEnvelope(handle)
            while (received != null) {
                onEnvelopeReceived?.invoke(address, received)
                received = NativeBleBridge.nextReceivedEnvelope(handle)
            }
        }
        pumpHandler.postDelayed({ pumpOutgoingFragments() }, PUMP_INTERVAL_MILLIS)
    }

    private fun writeFragment(gatt: BluetoothGatt, fragment: ByteArray) {
        val characteristic =
            gatt.getService(SERVICE_UUID)?.getCharacteristic(CHARACTERISTIC_UUID) ?: return
        characteristic.value = fragment
        gatt.writeCharacteristic(characteristic)
    }

    /** Set by the caller (e.g. a service wiring this into the rest of
     * the app) to receive fully-reassembled envelope bytes, keyed by
     * the sending device's Bluetooth address. */
    var onEnvelopeReceived: ((deviceAddress: String, envelope: ByteArray) -> Unit)? = null

    companion object {
        private const val TAG = "BleGattManager"
        private const val PUMP_INTERVAL_MILLIS = 200L
        // Placeholder UUIDs — a real deployment would mint its own
        // fixed service/characteristic UUIDs and keep them stable
        // across releases; not invented here since there's no spec in
        // this workspace's docs pinning specific values.
        val SERVICE_UUID: UUID = UUID.fromString("6e400001-b5a3-f393-e0a9-e50e24dcca9e")
        val CHARACTERISTIC_UUID: UUID = UUID.fromString("6e400002-b5a3-f393-e0a9-e50e24dcca9e")
    }
}

/**
 * JNI declarations matching `siar-transport-ble-android/src/jni_bridge.rs`
 * exactly (symbol `Java_com_siar_ble_NativeBleBridge_*`).
 */
object NativeBleBridge {
    init {
        System.loadLibrary("siar_transport_ble_android")
    }

    external fun createBridge(reassemblyCapacity: Int): Long
    external fun destroyBridge(handle: Long)
    external fun onFragmentReceived(handle: Long, data: ByteArray)
    external fun nextReceivedEnvelope(handle: Long): ByteArray?
    external fun queueEnvelopeToSend(handle: Long, data: ByteArray, protocol: Byte, maxFragmentBytes: Int)
    external fun nextFragmentToSend(handle: Long): ByteArray?
}
