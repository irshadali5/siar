package dev.irshad.siar

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat

/**
 * Foreground service that keeps a call's audio/video streams alive once
 * the app is backgrounded — the same requirement `PLATFORM_REQUIREMENTS.md`
 * and `dioxus.toml`'s `FOREGROUND_SERVICE_MICROPHONE`/`FOREGROUND_SERVICE_CAMERA`
 * permissions already anticipated. Android 14 requires the *typed*
 * foreground-service permission matching the service type actually
 * started, not just the generic `FOREGROUND_SERVICE` one — this class
 * picks `MICROPHONE`, or `MICROPHONE | CAMERA` together, based on the
 * `EXTRA_WITH_VIDEO` intent extra the caller passes in.
 *
 * How the Rust/call side is expected to drive this: `net::calls::place_call`/
 * the incoming-call accept path (`siar-core`) starts this service via
 * `startForegroundService(Intent(...))` when a call connects, and stops it
 * (`stopSelf()` from here, or `context.stopService(...)` from the caller)
 * when the call ends — mirroring the existing `CallEvent::Connected` /
 * `CallEvent::Ended` events already in `siar-ui`'s event loop. Wiring
 * that Rust → Kotlin call (JNI, or whatever binding `dx`'s Android
 * bootstrap ends up providing to reach an Android `Context` at all) is
 * the same bootstrap-glue uncertainty flagged in
 * `siar-android/src/main.rs` — this class only covers the Kotlin
 * side of "what a foreground call service needs to look like", not that
 * bridge.
 */
class CallForegroundService : Service() {

    companion object {
        const val EXTRA_WITH_VIDEO = "with_video"
        const val EXTRA_PEER_NAME = "peer_name"
        private const val CHANNEL_ID = "active_call"
        private const val NOTIFICATION_ID = 1

        fun start(context: Context, peerName: String, withVideo: Boolean) {
            val intent = Intent(context, CallForegroundService::class.java)
                .putExtra(EXTRA_PEER_NAME, peerName)
                .putExtra(EXTRA_WITH_VIDEO, withVideo)
            context.startForegroundService(intent)
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, CallForegroundService::class.java))
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val peerName = intent?.getStringExtra(EXTRA_PEER_NAME) ?: "Unknown"
        val withVideo = intent?.getBooleanExtra(EXTRA_WITH_VIDEO, false) ?: false

        ensureChannel()

        val notification: Notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(if (withVideo) "Video call" else "Voice call")
            .setContentText(peerName)
            .setSmallIcon(android.R.drawable.ic_menu_call)
            .setOngoing(true)
            .setCategory(NotificationCompat.CATEGORY_CALL)
            .build()

        val serviceType = if (withVideo) {
            ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE or ServiceInfo.FOREGROUND_SERVICE_TYPE_CAMERA
        } else {
            ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(NOTIFICATION_ID, notification, serviceType)
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }

        return START_NOT_STICKY
    }

    private fun ensureChannel() {
        val manager = getSystemService(NotificationManager::class.java) ?: return
        if (manager.getNotificationChannel(CHANNEL_ID) != null) return
        manager.createNotificationChannel(
            NotificationChannel(CHANNEL_ID, "Active call", NotificationManager.IMPORTANCE_LOW)
        )
    }
}
