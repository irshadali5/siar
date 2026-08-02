package dev.irshad.siar

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat

/**
 * Foreground service that keeps this device reachable in the background
 * so an incoming DM, room message, or call can wake it — the Kotlin
 * half of the "Background wake" toggle in Settings' Network tab
 * (`store::Store::background_wake_enabled`, off by default).
 *
 * ## What this actually keeps alive, and what it doesn't
 *
 * Siar is serverless P2P (iroh/QUIC), not Firebase Cloud Messaging —
 * there's no push server that can wake the app from fully stopped.
 * "Wake on message" here means: this foreground service, while running,
 * is Android's signal to *not* freeze/kill the process under Doze or
 * app-standby the way a normal backgrounded app eventually would, so
 * the existing iroh `Endpoint` the Rust side already holds open can
 * keep receiving on its live QUIC connections and post a notification
 * itself (via the app's own notification path, not this service)
 * when something arrives. This service's own notification is only the
 * mandatory "app is active in background" one Android requires for any
 * foreground service — it is not the per-message notification.
 *
 * ## The bootstrap-glue gap this doesn't resolve
 *
 * Starting/stopping this service from the Settings toggle needs a
 * Rust → Kotlin call this crate has no verified bridge for yet — the
 * same open item as `CallForegroundService`'s own doc comment already
 * flags for the call-connect/call-end path, and the same class of gap
 * as the Android JNI/native-activity bootstrap noted in
 * `siar-android/src/main.rs`. Until that bridge exists, this class is
 * wired to start itself from two places that don't need one:
 * [BootCompletedReceiver] (so the setting survives a reboot) and
 * `start`/`stop` being called directly from whatever Activity `dx`
 * scaffolds, once that Activity can read `background_wake_enabled` out
 * of the same sqlite settings table the Rust side already persists it
 * to (or, more simply, out of the mirrored `SharedPreferences` flag
 * this class reads via [PREFS_NAME] — see [isEnabled]/[setEnabled],
 * meant as a lightweight Kotlin-readable mirror of that one boolean
 * setting, not a replacement for it).
 */
class RelayForegroundService : Service() {

    companion object {
        private const val CHANNEL_ID = "background_relay"
        private const val NOTIFICATION_ID = 2
        private const val PREFS_NAME = "siar_relay_prefs"
        private const val PREF_ENABLED = "background_wake_enabled"

        fun start(context: Context) {
            if (context is android.app.Activity && (context.isFinishing || context.isDestroyed)) return
            context.startForegroundService(Intent(context, RelayForegroundService::class.java))
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, RelayForegroundService::class.java))
        }

        /** Mirrored flag `BootCompletedReceiver` checks — see class doc. */
        fun isEnabled(context: Context): Boolean =
            prefs(context).getBoolean(PREF_ENABLED, false)

        fun setEnabled(context: Context, enabled: Boolean) {
            prefs(context).edit().putBoolean(PREF_ENABLED, enabled).apply()
            if (enabled) start(context) else stop(context)
        }

        private fun prefs(context: Context): SharedPreferences =
            context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        ensureChannel()

        val notification: Notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Siar")
            .setContentText("Running in background — ready to receive messages and calls")
            .setSmallIcon(android.R.drawable.stat_notify_sync)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_MIN)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .build()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            // `dataSync` is the closest typed category to "keep a
            // network session alive"; Android 14 has no foreground
            // service type specifically for P2P messaging apps.
            startForeground(NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }

        // START_STICKY, unlike CallForegroundService's START_NOT_STICKY:
        // a call ending is a clear, event-driven stop signal, but
        // "stay reachable in the background" is meant to persist across
        // Android killing and restarting the process under memory
        // pressure — restarting is the correct behavior here as long as
        // [isEnabled] is still true.
        return START_STICKY
    }

    private fun ensureChannel() {
        val manager = getSystemService(NotificationManager::class.java) ?: return
        if (manager.getNotificationChannel(CHANNEL_ID) != null) return
        manager.createNotificationChannel(
            NotificationChannel(CHANNEL_ID, "Background connection", NotificationManager.IMPORTANCE_MIN)
        )
    }
}
