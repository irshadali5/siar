package com.siar.messaging

import android.os.Handler
import android.os.Looper

/**
 * Thin Kotlin wrapper over `siar-android-messaging`'s JNI surface — see
 * that crate's own `lib.rs` doc comment for the full picture of what
 * this does and doesn't cover yet (no groups, no attachments, no
 * anonymous mailbox path, no identity persistence — a real slice, not
 * the whole of `siar-messaging`).
 *
 * [pollEvents] runs on a fixed [Handler] tick, same tradeoff
 * [com.siar.ble.BleGattManager]'s fragment pump and
 * [com.siar.messenger.ConnectivityScreen]'s polling already carry for
 * the same reason: JNI here is call-in only, so Rust has no way to
 * notify Kotlin the instant an event is ready.
 */
object MessagingBridge {
    private val pumpHandler = Handler(Looper.getMainLooper())
    private var pumping = false

    /** Bootstraps a fresh identity + `SiarEndpoint` and starts the
     * incoming-event pump — call once, from a background thread (every
     * `Native*` call in this object blocks the calling thread on a
     * dedicated Tokio runtime; see the Rust side's own `runtime()` doc
     * comment). Returns this device's own shareable ticket string.
     *
     * `filesDir` should be the caller's `Context.filesDir.absolutePath`
     * — app-private storage the OS already sandboxes per-app, no new
     * permission needed. Identity/database now persist there across
     * restarts (previously regenerated fresh every launch — the "no
     * identity persistence" gap `siar-android-messaging`'s own doc
     * comment used to name explicitly). */
    fun bootstrap(filesDir: String): Result<String> = callNative { NativeMessagingBridge.bootstrap(filesDir) }

    /** Registers a peer's ticket so incoming messages from them can be
     * decrypted — must be called before any message from that peer
     * will show up via [onTextReceived]. See the Rust side's own
     * `add_peer` doc comment for why this precondition exists (it's
     * inherited unchanged from `MessageService::handle_incoming`, not
     * invented for this crate). */
    fun addPeer(ticket: String): Result<Unit> = callNative { NativeMessagingBridge.addPeer(ticket) }.map { }

    fun sendText(peerTicket: String, text: String): Result<String> =
        callNative { NativeMessagingBridge.sendText(peerTicket, text) }

    fun checkMailbox(relayTicket: String): Result<Unit> =
        callNative { NativeMessagingBridge.checkMailbox(relayTicket) }.map { }

    /** The unlinkable counterpart to [sendText] — see
     * `siar-android-messaging`'s own `send_text_anon_inner` doc comment
     * for exactly what this does and doesn't guarantee. */
    fun sendTextAnon(peerTicket: String, relayTicket: String, text: String): Result<String> =
        callNative { NativeMessagingBridge.sendTextAnon(peerTicket, relayTicket, text) }

    /** The unlinkable counterpart to [checkMailbox] — `peerTicket` must
     * already be registered via [addPeer], since the Rust side has no
     * other way to identify who a `TokenMailboxDeposit` response is
     * from (see [onAnonTextReceived]'s own doc comment). */
    fun checkMailboxAnon(peerTicket: String, relayTicket: String): Result<Unit> =
        callNative { NativeMessagingBridge.checkMailboxAnon(peerTicket, relayTicket) }.map { }

    /** Set by the caller (e.g. an activity or view model) to receive
     * incoming text messages, keyed by the sender's endpoint id in the
     * Debug-formatted form the Rust side emits (see that side's own
     * note on why Debug rather than Display was used). */
    var onTextReceived: ((senderEndpointDebug: String, text: String) -> Unit)? = null

    /** Set by the caller to receive a mailbox check-in result — the
     * count of items that arrived (see the Rust side's own doc comment
     * on why individual item contents aren't surfaced yet). */
    var onMailboxChecked: ((count: Int) -> Unit)? = null

    /** Set by the caller to receive messages that arrived via the
     * unlinkable path — `matchedPeerEndpointDebug` is *inferred* on the
     * Rust side (whichever registered peer's session key happened to
     * decrypt it), not read from the wire; see
     * `siar-android-messaging`'s own doc comment on the `anon_text`
     * event kind for why an anonymous response has no sender field at
     * all. */
    var onAnonTextReceived: ((matchedPeerEndpointDebug: String, text: String) -> Unit)? = null

    /** Starts the fixed-interval poll loop — call once, after
     * [bootstrap] succeeds. */
    fun startPolling() {
        if (pumping) return
        pumping = true
        pollEvents()
    }

    fun stopPolling() {
        pumping = false
    }

    /** Reports this app's messaging layer as going away — see the Rust
     * side's own `shutdown_inner` doc comment for exactly what this
     * does and doesn't do (real connectivity-state update, not a full
     * endpoint teardown). Call from `MainActivity.onDestroy`. */
    fun shutdown() {
        NativeMessagingBridge.shutdown()
    }

    private fun pollEvents() {
        if (!pumping) return
        var line = NativeMessagingBridge.pollNextEvent()
        while (line != null) {
            dispatch(line)
            line = NativeMessagingBridge.pollNextEvent()
        }
        pumpHandler.postDelayed({ pollEvents() }, POLL_INTERVAL_MILLIS)
    }

    private fun dispatch(line: String) {
        val parts = line.split("\t", limit = 3)
        when (parts.getOrNull(0)) {
            "text" -> {
                val sender = parts.getOrNull(1) ?: return
                val text = parts.getOrNull(2) ?: return
                onTextReceived?.invoke(sender, text)
            }
            "mailbox" -> {
                val count = parts.getOrNull(1)?.toIntOrNull() ?: return
                onMailboxChecked?.invoke(count)
            }
            "anon_text" -> {
                val matchedPeer = parts.getOrNull(1) ?: return
                val text = parts.getOrNull(2) ?: return
                onAnonTextReceived?.invoke(matchedPeer, text)
            }
        }
    }

    /** Every `Native*` call returns a plain string, `"error:<message>"`
     * on failure — see the Rust side's own `to_jstring` doc comment for
     * why this simple a channel was chosen. This wrapper is what turns
     * that back into an idiomatic Kotlin `Result`. */
    private inline fun callNative(call: () -> String): Result<String> {
        val result = call()
        return if (result.startsWith("error:")) {
            Result.failure(RuntimeException(result.removePrefix("error:")))
        } else {
            Result.success(result)
        }
    }

    private const val POLL_INTERVAL_MILLIS = 300L
}

/**
 * JNI declarations matching `apps/android/messaging-jni/src/lib.rs`
 * exactly (symbol `Java_com_siar_messaging_NativeMessagingBridge_*`).
 * `System.loadLibrary` name matches that crate's Cargo.toml package
 * name (hyphens to underscores): `siar_android_messaging`.
 */
private object NativeMessagingBridge {
    init {
        System.loadLibrary("siar_android_messaging")
    }

    external fun bootstrap(filesDir: String): String
    external fun addPeer(ticket: String): String
    external fun sendText(peerTicket: String, text: String): String
    external fun checkMailbox(relayTicket: String): String
    external fun sendTextAnon(peerTicket: String, relayTicket: String, text: String): String
    external fun checkMailboxAnon(peerTicket: String, relayTicket: String): String
    external fun pollNextEvent(): String?
    external fun shutdown()
}
