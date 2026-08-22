package com.siar.bluetoothclassic

import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothSocket
import android.content.Context
import android.util.Log
import java.io.IOException
import java.util.UUID
import java.util.concurrent.Executors

/**
 * Drives `BluetoothSocket`/`BluetoothServerSocket` directly — same
 * risk-inversion pattern as every other bridge in this app; see
 * `siar-transport-bluetooth-classic`'s `jni_bridge.rs` doc comment for
 * the full rationale, mirrored deliberately from
 * `siar-transport-ble-android`'s (method names included — anyone who's
 * read [com.siar.ble.BleGattManager] has already read the shape of
 * this class).
 *
 * One RFCOMM socket, one native handle, per peer — next.md §7/§21's
 * "one RFCOMM socket per peer relationship" — using a dedicated blocking
 * I/O thread per connection (`Executors.newCachedThreadPool`), the
 * conventional pattern for `BluetoothSocket` since its streams are
 * blocking, not `Handler`-postable the way BLE's GATT callbacks are.
 */
class BluetoothClassicManager(private val context: Context) {
    private val adapter: BluetoothAdapter? =
        (context.getSystemService(Context.BLUETOOTH_SERVICE) as? android.bluetooth.BluetoothManager)?.adapter
    private val ioExecutor = Executors.newCachedThreadPool()
    private val connections = mutableMapOf<String, Connection>()

    private class Connection(val socket: BluetoothSocket, val handle: Long)

    /** Listens for incoming RFCOMM connections under [SERVICE_UUID].
     * Runs its own accept loop on [ioExecutor] until [stop] closes the
     * server socket out from under it (the conventional way to
     * interrupt a blocking `accept()` on this API). */
    fun listen() {
        if (!com.siar.messenger.PermissionsHelper.hasBluetoothConnect(context)) {
            Log.w(TAG, "BLUETOOTH_CONNECT not granted, skipping RFCOMM listen")
            return
        }
        ioExecutor.execute {
            val serverSocket = try {
                adapter?.listenUsingRfcommWithServiceRecord(SERVICE_NAME, SERVICE_UUID)
            } catch (e: IOException) {
                Log.w(TAG, "RFCOMM listen failed", e)
                null
            } ?: return@execute

            while (true) {
                val socket = try {
                    serverSocket.accept()
                } catch (e: IOException) {
                    break // server socket closed — see stop()
                }
                onConnected(socket)
            }
        }
    }

    /** Initiates an outgoing RFCOMM connection to `device`. */
    fun connect(device: BluetoothDevice) {
        if (!com.siar.messenger.PermissionsHelper.hasBluetoothConnect(context)) {
            Log.w(TAG, "BLUETOOTH_CONNECT not granted, skipping RFCOMM connect")
            return
        }
        ioExecutor.execute {
            val socket = try {
                device.createRfcommSocketToServiceRecord(SERVICE_UUID).also { it.connect() }
            } catch (e: IOException) {
                Log.w(TAG, "RFCOMM connect failed for ${device.address}", e)
                return@execute
            }
            onConnected(socket)
        }
    }

    private fun onConnected(socket: BluetoothSocket) {
        val handle = NativeBluetoothClassicBridge.createBridge()
        val address = socket.remoteDevice.address
        connections[address] = Connection(socket, handle)
        if (connections.size == 1) {
            com.siar.connectivity.ConnectivityBridge.markUp(com.siar.connectivity.TransportLinkKind.BluetoothClassic)
        }
        ioExecutor.execute { readLoop(address, socket, handle) }
    }

    private fun readLoop(address: String, socket: BluetoothSocket, handle: Long) {
        val buffer = ByteArray(4096)
        try {
            while (true) {
                val count = socket.inputStream.read(buffer)
                if (count < 0) break
                NativeBluetoothClassicBridge.onBytesReceived(handle, buffer.copyOf(count))
                var envelope = NativeBluetoothClassicBridge.nextReceivedEnvelope(handle)
                while (envelope != null) {
                    onEnvelopeReceived?.invoke(address, envelope)
                    envelope = NativeBluetoothClassicBridge.nextReceivedEnvelope(handle)
                }
            }
        } catch (e: IOException) {
            Log.i(TAG, "RFCOMM connection to $address closed: ${e.message}")
        } finally {
            closeConnection(address)
        }
    }

    /** Queues `envelopeBytes` for framed delivery to `deviceAddress`,
     * then flushes every currently-ready chunk straight through the
     * socket's `OutputStream` — unlike BLE, RFCOMM has no MTU to pace
     * writes against, matching `nextChunkToSend`'s own doc comment. */
    fun sendEnvelope(deviceAddress: String, envelopeBytes: ByteArray) {
        val connection = connections[deviceAddress] ?: return
        NativeBluetoothClassicBridge.queueEnvelopeToSend(connection.handle, envelopeBytes)
        ioExecutor.execute {
            var chunk = NativeBluetoothClassicBridge.nextChunkToSend(connection.handle)
            while (chunk != null) {
                try {
                    connection.socket.outputStream.write(chunk)
                } catch (e: IOException) {
                    closeConnection(deviceAddress)
                    return@execute
                }
                chunk = NativeBluetoothClassicBridge.nextChunkToSend(connection.handle)
            }
        }
    }

    private fun closeConnection(address: String) {
        connections.remove(address)?.let {
            runCatching { it.socket.close() }
            NativeBluetoothClassicBridge.destroyBridge(it.handle)
            if (connections.isEmpty()) {
                com.siar.connectivity.ConnectivityBridge.markDown(com.siar.connectivity.TransportLinkKind.BluetoothClassic)
            }
        }
    }

    fun stop() {
        connections.keys.toList().forEach { closeConnection(it) }
    }

    /** Set by the caller to receive fully-framed envelope bytes, keyed
     * by the sending device's Bluetooth address. */
    var onEnvelopeReceived: ((deviceAddress: String, envelope: ByteArray) -> Unit)? = null

    companion object {
        private const val TAG = "BluetoothClassicManager"
        private const val SERVICE_NAME = "SiarMessenger"
        // Placeholder UUID — see BleGattManager's own note on why this
        // isn't pinned against a published spec in this workspace.
        val SERVICE_UUID: UUID = UUID.fromString("8ce255c0-200a-11e0-ac64-0800200c9a66")
    }
}

/**
 * JNI declarations matching
 * `siar-transport-bluetooth-classic/src/jni_bridge.rs` exactly (symbol
 * `Java_com_siar_bluetoothclassic_NativeBluetoothClassicBridge_*`).
 */
object NativeBluetoothClassicBridge {
    init {
        System.loadLibrary("siar_transport_bluetooth_classic")
    }

    external fun createBridge(): Long
    external fun destroyBridge(handle: Long)
    external fun onBytesReceived(handle: Long, data: ByteArray)
    external fun nextReceivedEnvelope(handle: Long): ByteArray?
    external fun queueEnvelopeToSend(handle: Long, data: ByteArray)
    external fun nextChunkToSend(handle: Long): ByteArray?
}
