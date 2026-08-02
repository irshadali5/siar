package dev.irshad.siar

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/**
 * Restarts [RelayForegroundService] after a reboot if the user had
 * "Background wake" turned on — without this, the setting would
 * silently stop doing anything the next time the device restarts,
 * since a foreground service doesn't survive that on its own. Requires
 * `RECEIVE_BOOT_COMPLETED` (see `AndroidManifest.snippet.xml`) and,
 * like every permission in this package, is inert unless the user
 * actually opted into the feature it's supporting — this receiver does
 * nothing at all when [RelayForegroundService.isEnabled] is false.
 */
class BootCompletedReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != Intent.ACTION_BOOT_COMPLETED) return
        if (RelayForegroundService.isEnabled(context)) {
            RelayForegroundService.start(context)
        }
    }
}
