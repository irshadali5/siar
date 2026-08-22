package com.siar.messenger

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.siar.ble.BleGattManager
import com.siar.bluetoothclassic.BluetoothClassicManager
import com.siar.connectivity.ConnectivityBridge
import com.siar.messaging.MessagingBridge
import com.siar.wifiaware.WifiAwareManagerBridge
import com.siar.wifidirect.WifiDirectManager
import kotlin.concurrent.thread

/**
 * Wires together the four transport bridges this pass built
 * ([WifiDirectManager], [WifiAwareManagerBridge], [BleGattManager],
 * [BluetoothClassicManager]), the shared connectivity state
 * ([ConnectivityBridge]), and a real [MessagingBridge] wrapping
 * `siar-messaging::MessageService`/`GroupService` — plus a chat UI
 * ([MessengerScreen]/[ChatStore]/[GroupStore]) on top of it: a 1:1
 * contact list and conversation screen, groups/MLS (create, add
 * member, join via a pending-invite banner, group messaging), and 1:1
 * attachments (pick a file, send it; fetch and save a received one).
 * Every one of those Rust crates' own doc comments named this activity
 * as the missing real consumer.
 *
 * ## What this activity still deliberately does NOT do
 *
 * Group attachments — see `siar-android-messaging`'s own top doc
 * comment for why (no `DeviceId`-to-`PeerTicket` lookup exists
 * anywhere in this codebase to fetch one with). Contacts, groups, and
 * message history all live only in this process's memory — nothing
 * here reads back the message history the native side already
 * persists to `siar.db`, since no FFI exists yet to read it (see
 * `ChatStore`'s own doc comment); group membership itself isn't
 * persisted across restarts either (see `GroupStore.recordIncoming`'s
 * own comment on the placeholder name a group gets if a text frame for
 * it arrives after a fresh process start). `LocalLan`/`InternetDirect`/
 * `InternetRelay` also aren't reported into [ConnectivityBridge] from
 * anywhere in this activity — `MessagingBridge`'s own `SiarEndpoint`
 * isn't threaded through to `ConnectivityBridge` yet, a real remaining
 * gap between these two bridges this pass didn't close.
 *
 * ## What could not be verified this pass
 *
 * No Android SDK, NDK, Gradle, or emulator exists in this sandbox —
 * every line here is written against the real, current Android SDK
 * and Compose Material3 API shapes (checked against actual API
 * surfaces where genuinely uncertain — e.g. `Divider` being deprecated
 * in favor of `HorizontalDivider` as of Material3 1.2, confirmed via
 * real docs, not guessed), but none of it has been compiled or run.
 * Same honesty this workspace has applied to every iroh-touching Rust
 * crate all along, now applying to the Kotlin/Android side too.
 */
class MainActivity : ComponentActivity() {
    private lateinit var wifiDirectManager: WifiDirectManager
    private lateinit var wifiAwareManager: WifiAwareManagerBridge
    private lateinit var bleGattManager: BleGattManager
    private lateinit var bluetoothClassicManager: BluetoothClassicManager

    private val permissionLauncher =
        registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) { results ->
            if (results.values.all { it }) {
                startTransports()
            }
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        wifiDirectManager = WifiDirectManager(this)
        wifiAwareManager = WifiAwareManagerBridge(this)
        bleGattManager = BleGattManager(this)
        bluetoothClassicManager = BluetoothClassicManager(this)

        // Before anything else native-side: see `initAndroidContext`'s
        // own doc comment for why this needs to happen before
        // `startMessaging()`'s `bootstrap()` call constructs a
        // `SiarEndpoint` (that construction is what actually triggers
        // Android DNS resolution through this context).
        MessagingBridge.initAndroidContext(applicationContext)

        // Wired before `startMessaging()` below so no incoming event
        // from the very first poll tick onward is ever missed —
        // `ChatStore.recordIncoming`/[GroupStore.recordIncoming]/etc.
        // are what make an incoming message actually show up
        // somewhere in this app; without this the native side would
        // still receive and decrypt messages (see
        // `siar-android-messaging`'s own doc comment) with nothing in
        // this app ever finding out.
        MessagingBridge.onTextReceived = { sender, text -> ChatStore.recordIncoming(sender, text, anon = false) }
        MessagingBridge.onAnonTextReceived = { sender, text -> ChatStore.recordIncoming(sender, text, anon = true) }
        MessagingBridge.onAttachmentReceived = { sender, blobHashB64, sizeBytes, mediaType, keyB64 ->
            ChatStore.recordIncomingAttachment(sender, AttachmentRef(blobHashB64, sizeBytes, mediaType, keyB64))
        }
        MessagingBridge.onGroupInvite = { conversation, fromDevice -> GroupStore.recordInvite(conversation, fromDevice) }
        MessagingBridge.onGroupText = { conversation, _sender, text -> GroupStore.recordIncoming(conversation, text) }

        setContent {
            MaterialTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    Column(modifier = Modifier.fillMaxSize()) {
                        ConnectivityScreen()
                        HorizontalDivider()
                        MessengerScreen()
                    }
                }
            }
        }

        if (PermissionsHelper.hasAllRequiredPermissions(this)) {
            startTransports()
        } else {
            permissionLauncher.launch(PermissionsHelper.requiredPermissions())
        }

        startMessaging()
    }

    private fun startTransports() {
        wifiDirectManager.start()
        wifiAwareManager.start()
        bleGattManager.startAdvertisingAndScanning(BleGattManager.SERVICE_UUID)
        bluetoothClassicManager.listen()
    }

    /** `MessagingBridge.bootstrap()` blocks on real network I/O (binding
     * a `SiarEndpoint`) — see that function's own Rust-side doc comment.
     * Run off the main thread, not because Compose needs it async, but
     * because blocking the main thread on network I/O would freeze the
     * UI and risk an ANR. `thread { }` rather than a coroutine: this
     * activity has no other coroutine/`viewModelScope` machinery set up
     * yet, and a raw thread is the simplest correct choice for one
     * fire-and-forget call — a real app would likely use
     * `lifecycleScope.launch(Dispatchers.IO)` instead once it has more
     * than one such call to make. */
    private fun startMessaging() {
        val filesDir = applicationContext.filesDir.absolutePath
        thread {
            val result = MessagingBridge.bootstrap(filesDir)
            result.onSuccess { ticket ->
                ChatStore.myTicket.value = ticket
                MessagingBridge.startPolling()
            }
            result.onFailure { e ->
                android.util.Log.e("MainActivity", "MessagingBridge bootstrap failed", e)
            }
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        wifiDirectManager.stop()
        wifiAwareManager.stop()
        bleGattManager.stop()
        bluetoothClassicManager.stop()
        MessagingBridge.shutdown() // stops polling itself too — see that function's own doc comment
    }
}

/** next.md §60's single-line connectivity summary — polled rather than
 * pushed into Compose state on every link change, since none of the
 * four bridges currently have a way to notify this composable directly
 * (same call-in-only JNI limitation [BleGattManager]'s own doc comment
 * names for its fragment-pump loop). A production build would likely
 * replace this with a proper `StateFlow` a bridge pushes into instead
 * of polling; noted, not built, same reasoning as that pump loop's own
 * doc comment. */
@Composable
fun ConnectivityScreen() {
    var mode by remember { mutableStateOf(ConnectivityBridge.effectiveMode()) }
    Column(modifier = Modifier.padding(16.dp)) {
        Text("Siar")
        Text("Connectivity: $mode")
    }
}
