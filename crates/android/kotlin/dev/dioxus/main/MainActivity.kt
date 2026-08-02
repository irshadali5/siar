package dev.dioxus.main

import android.app.Activity
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Intent
import android.os.Bundle
import android.util.Base64
import android.webkit.JavascriptInterface
import android.webkit.WebView
import android.Manifest
import android.content.pm.PackageManager
import androidx.core.app.NotificationCompat
import androidx.core.content.ContextCompat
import dev.irshad.siar.RelayForegroundService
import dev.irshad.siar.RuntimePermissions

typealias BuildConfig = dev.irshad.siar.BuildConfig

/** Android host additions that Dioxus cannot express from Rust alone. */
class MainActivity : WryActivity() {
    private var pendingExport: ByteArray? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (RelayForegroundService.isEnabled(this)) {
            RelayForegroundService.start(this)
        }
    }

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        webView.addJavascriptInterface(SiarAndroidBridge(), "SiarAndroid")
    }

    @Deprecated("Activity result API is used here to keep the generated host dependency-free")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode != REQUEST_EXPORT || resultCode != Activity.RESULT_OK) return
        val bytes = pendingExport ?: return
        val uri = data?.data ?: return
        contentResolver.openOutputStream(uri, "w")?.use { it.write(bytes) }
        pendingExport = null
    }

    inner class SiarAndroidBridge {
        @JavascriptInterface
        fun setBackgroundWake(enabled: Boolean) {
            runOnUiThread {
                if (enabled) RuntimePermissions.requestNotifications(this@MainActivity)
                RelayForegroundService.setEnabled(this@MainActivity, enabled)
            }
        }

        @JavascriptInterface
        fun requestAudioPermission() {
            runOnUiThread { RuntimePermissions.requestAudio(this@MainActivity) }
        }

        @JavascriptInterface
        fun hasAudioPermission(): Boolean =
            ContextCompat.checkSelfPermission(
                this@MainActivity,
                Manifest.permission.RECORD_AUDIO,
            ) == PackageManager.PERMISSION_GRANTED

        @JavascriptInterface
        fun requestNearbyPermissions() {
            runOnUiThread { RuntimePermissions.requestNearby(this@MainActivity) }
        }

        @JavascriptInterface
        fun requestNotificationPermission() {
            runOnUiThread { RuntimePermissions.requestNotifications(this@MainActivity) }
        }

        /** Post only while the Activity is backgrounded; foreground messages are already visible. */
        @JavascriptInterface
        fun showNotification(title: String, body: String, playSound: Boolean) {
            runOnUiThread {
                if (hasWindowFocus()) return@runOnUiThread
                if (
                    android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU &&
                    ContextCompat.checkSelfPermission(
                        this@MainActivity,
                        Manifest.permission.POST_NOTIFICATIONS,
                    ) != PackageManager.PERMISSION_GRANTED
                ) return@runOnUiThread

                val manager = getSystemService(NotificationManager::class.java) ?: return@runOnUiThread
                val channelId = if (playSound) CHANNEL_MESSAGES else CHANNEL_MESSAGES_SILENT
                if (
                    android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O &&
                    manager.getNotificationChannel(channelId) == null
                ) {
                    val importance = if (playSound) {
                        NotificationManager.IMPORTANCE_HIGH
                    } else {
                        NotificationManager.IMPORTANCE_LOW
                    }
                    manager.createNotificationChannel(
                        NotificationChannel(channelId, "Messages", importance).apply {
                            if (!playSound) setSound(null, null)
                        },
                    )
                }

                val openApp = Intent(this@MainActivity, MainActivity::class.java).apply {
                    flags = Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP
                }
                val pendingIntent = PendingIntent.getActivity(
                    this@MainActivity,
                    0,
                    openApp,
                    PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
                )
                val notification = NotificationCompat.Builder(this@MainActivity, channelId)
                    .setSmallIcon(android.R.drawable.stat_notify_chat)
                    .setContentTitle(title.take(80))
                    .setContentText(body.take(240))
                    .setStyle(NotificationCompat.BigTextStyle().bigText(body.take(500)))
                    .setContentIntent(pendingIntent)
                    .setAutoCancel(true)
                    .setCategory(NotificationCompat.CATEGORY_MESSAGE)
                    .setPriority(if (playSound) NotificationCompat.PRIORITY_HIGH else NotificationCompat.PRIORITY_LOW)
                    .build()
                manager.notify(nextNotificationId++, notification)
            }
        }

        @JavascriptInterface
        fun saveFile(fileName: String, mimeType: String, base64: String) {
            val decoded = runCatching { Base64.decode(base64, Base64.DEFAULT) }.getOrNull() ?: return
            runOnUiThread {
                pendingExport = decoded
                val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
                    addCategory(Intent.CATEGORY_OPENABLE)
                    type = mimeType.ifBlank { "application/octet-stream" }
                    putExtra(Intent.EXTRA_TITLE, fileName.ifBlank { "siar-export" })
                }
                startActivityForResult(intent, REQUEST_EXPORT)
            }
        }
    }

    companion object {
        private const val REQUEST_EXPORT = 4301
        private const val CHANNEL_MESSAGES = "messages"
        private const val CHANNEL_MESSAGES_SILENT = "messages_silent"
        private var nextNotificationId = 1000
    }
}
