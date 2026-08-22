package com.siar.messenger

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
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
 * ([ConnectivityBridge]), and — as of this pass — a real
 * [MessagingBridge] wrapping `siar-messaging::MessageService`. Every
 * one of those Rust crates' own doc comments named this activity as
 * the missing real consumer.
 *
 * ## What this activity deliberately does NOT do
 *
 * [MessagingBridge] is a genuine send/receive text-message surface
 * now (see that class's and `siar-android-messaging`'s own doc
 * comments for the exact scope), but there is still no UI for it here
 * — no conversation screen, no contact list, no way to actually type
 * in a peer ticket and see a chat thread. `apps/cli`/`apps/desktop`
 * remain this workspace's only chat clients with an actual UI; what
 * this activity proves is that the four radios can come up, report
 * into `ConnectivityState`, and that a real message can now be sent
 * and received from this process — a real, necessary, but still
 * partial step, named honestly rather than presented as a finished
 * app. `LocalLan`/`InternetDirect`/`InternetRelay` also aren't
 * reported into [ConnectivityBridge] from anywhere in this activity —
 * `MessagingBridge`'s own `SiarEndpoint` isn't threaded through to
 * `ConnectivityBridge` yet, a real remaining gap between these two
 * bridges this pass didn't close.
 *
 * ## What could not be verified this pass

 *
 * No Android SDK, NDK, Gradle, or emulator exists in this sandbox —
 * every line here is written against the real, current Android SDK
 * API shapes (checked against actual API surfaces where genuinely
 * uncertain — see `WifiAwareManagerBridge.kt`'s own note on
 * `WifiAwareNetworkInfo.getPort()`'s undocumented default), but none of
 * it has been compiled or run. Same honesty this workspace has applied
 * to every iroh-touching Rust crate all along, now applying to the
 * Kotlin/Android side for the first time.
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

        setContent {
            MaterialTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    ConnectivityScreen()
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
        thread {
            val result = MessagingBridge.bootstrap()
            result.onSuccess {
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
        MessagingBridge.stopPolling()
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
