package com.siar.messaging

import android.os.Handler
import android.os.Looper

/**
 * Thin Kotlin wrapper over `siar-android-messaging`'s JNI surface — see
 * that crate's own `lib.rs` doc comment for the full picture of what
 * this covers (1:1 text, groups/MLS, 1:1 attachments) and what's still
 * out of scope (anonymous mailbox item contents, group attachments,
 * identity persistence's own remaining caveats).
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

    /** Publishes the real Application `Context` to the native side for
     * iroh's Android DNS resolver — see the Rust side's own
     * `initAndroidContext` doc comment for why this exists as a real,
     * explicit call now instead of the previous (likely-inert)
     * `JNI_OnLoad`-based guess. Call once, early — before [bootstrap] —
     * from `MainActivity.onCreate`, passing `applicationContext` (not
     * an `Activity` context, which doesn't outlive a single screen the
     * way this native-side reference needs to). Safe to call more than
     * once (each call installs a fresh global reference; the native
     * side doesn't track whether it's already been called), but one
     * call per process is all this needs. */
    fun initAndroidContext(context: android.content.Context) {
        NativeMessagingBridge.initAndroidContext(context.applicationContext)
    }

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

    /** Decodes a `PeerTicket` string into the same sender key
     * [onTextReceived] and [onAnonTextReceived] deliver — a
     * `PeerTicket` and a `{:?}`-formatted `iroh::EndpointId` look
     * nothing alike as strings even when they identify the same peer,
     * and Kotlin has no way to decode a ticket on its own (that's real
     * Rust-side logic). A contact list needs this to match an incoming
     * message back to the contact that sent it — call it once when a
     * contact's ticket is added and store the result alongside the
     * contact, not the raw ticket, as the thread key. */
    fun ticketEndpointDebug(ticket: String): Result<String> =
        callNative { NativeMessagingBridge.ticketEndpointDebug(ticket) }

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

    /** This device's own `DeviceId`, as plain UUID text — share
     * alongside [groupKeyPackage] and [accountId] with whoever will add
     * this device to a group via [groupAddMember]. */
    fun deviceId(): Result<String> = callNative { NativeMessagingBridge.deviceId() }

    /** This device's own `AccountId`, as plain UUID text — see
     * [deviceId]'s own doc comment. */
    fun accountId(): Result<String> = callNative { NativeMessagingBridge.accountId() }

    /** This device's own base64-encoded MLS key package, published once
     * at [bootstrap] time — hand this, [deviceId], and [accountId] to
     * whoever will call [groupAddMember] for this device. Empty string
     * (not a failure) if publishing failed at bootstrap — see the Rust
     * side's own `key_package_b64` field doc comment. */
    fun groupKeyPackage(): Result<String> = callNative { NativeMessagingBridge.groupKeyPackage() }

    /** Creates a new MLS group founded by this device's account.
     * Returns the new conversation id — share it with whoever will be
     * added via [groupAddMember]. */
    fun groupCreate(): Result<String> = callNative { NativeMessagingBridge.groupCreate() }

    /** Admin-only (enforced natively). Adds a new member — their
     * [deviceId]/[accountId]/ticket/[groupKeyPackage], all obtained from
     * them out-of-band beforehand — to an existing group, sending the
     * MLS commit and welcome over the wire. Returns the
     * post-admission group state, base64-encoded: relay this back to
     * the new member out-of-band alongside the conversation id, for
     * them to pass to [groupJoin]. */
    fun groupAddMember(
        conversation: String,
        peerTicket: String,
        peerDeviceId: String,
        peerAccountId: String,
        keyPackageB64: String,
    ): Result<String> =
        callNative { NativeMessagingBridge.groupAddMember(conversation, peerTicket, peerDeviceId, peerAccountId, keyPackageB64) }

    fun groupSendText(conversation: String, text: String): Result<String> =
        callNative { NativeMessagingBridge.groupSendText(conversation, text) }

    /** Joins a group this device was added to — consumes the buffered
     * `GroupMlsWelcome` the pump already received over the wire (see
     * the `group_invite` event) together with the group state obtained
     * out-of-band from the admin's [groupAddMember] call. Fails with a
     * clear message if there's no pending invite for this conversation
     * (already joined, already declined, or the welcome hasn't arrived
     * over the wire yet). */
    fun groupJoin(conversation: String, groupStateB64: String): Result<Unit> =
        callNative { NativeMessagingBridge.groupJoin(conversation, groupStateB64) }.map { }

    /** Discards a buffered invite without joining. Returns whether
     * there actually was a pending invite to discard — `false` isn't a
     * failure, just a no-op (already decided). */
    fun groupDeclineInvite(conversation: String): Result<Boolean> =
        callNative { NativeMessagingBridge.groupDeclineInvite(conversation) }.map { it == "true" }

    /** 1:1 only — see `siar-android-messaging`'s own top doc comment for
     * why group attachments aren't wired. `fileBytes` is base64-encoded
     * here, not passed as a raw byte array — see the Rust side's own
     * `sendAttachment` doc comment for why. `mediaType` is a MIME-type
     * string (`"image/jpeg"`, etc.); an unrecognized one becomes
     * `MediaType::Other` on the Rust side rather than failing. */
    fun sendAttachment(peerTicket: String, fileBytes: ByteArray, mediaType: String): Result<String> {
        val fileBytesB64 = android.util.Base64.encodeToString(fileBytes, android.util.Base64.NO_WRAP)
        return callNative { NativeMessagingBridge.sendAttachment(peerTicket, fileBytesB64, mediaType) }
    }

    /** Retrieves and decrypts a previously-received attachment's blob —
     * pass the exact `blobHashB64`/`encryptedSizeBytes`/`mediaType`/
     * `attachmentKeyB64` fields an `attachment` poll event carried (see
     * [onAttachmentReceived]). Returns the plaintext file bytes. */
    fun fetchAttachment(
        peerTicket: String,
        blobHashB64: String,
        encryptedSizeBytes: Long,
        mediaType: String,
        attachmentKeyB64: String,
    ): Result<ByteArray> =
        callNative { NativeMessagingBridge.fetchAttachment(peerTicket, blobHashB64, encryptedSizeBytes, mediaType, attachmentKeyB64) }
            .map { android.util.Base64.decode(it, android.util.Base64.NO_WRAP) }

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

    /** Set by the caller to receive a 1:1 attachment reference — pass
     * these five fields straight to [fetchAttachment] to retrieve the
     * actual bytes; nothing here fetches automatically (a real,
     * deliberate choice — attachments can be large, see
     * `siar_domain::MAX_ATTACHMENT_BYTES`, so fetching is opt-in per
     * item, driven by the UI, not automatic on arrival). */
    var onAttachmentReceived:
        ((senderEndpointDebug: String, blobHashB64: String, encryptedSizeBytes: Long, mediaType: String, attachmentKeyB64: String) -> Unit)? =
        null

    /** Set by the caller to receive a group invite — a `GroupMlsWelcome`
     * arrived and is buffered on the Rust side, waiting on [groupJoin]/
     * [groupDeclineInvite]. */
    var onGroupInvite: ((conversation: String, fromDeviceDebug: String) -> Unit)? = null

    /** Set by the caller to receive a group text message. */
    var onGroupText: ((conversation: String, senderDeviceDebug: String, text: String) -> Unit)? = null

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

    /** Tears down this app's messaging layer for real — stops the
     * native incoming-event pump and drops the bound endpoint, then
     * updates connectivity state (see the Rust side's own
     * `shutdown_inner` doc comment for the one remaining timing
     * caveat: pump teardown is scheduled, not guaranteed synchronous).
     * Also stops this bridge's own polling loop, since there's nothing
     * left to poll after this call. Call from `MainActivity.onDestroy`. */
    fun shutdown() {
        stopPolling()
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
        val kind = line.substringBefore('\t')
        when (kind) {
            "text" -> {
                val parts = line.split("\t", limit = 3)
                val sender = parts.getOrNull(1) ?: return
                val text = parts.getOrNull(2) ?: return
                onTextReceived?.invoke(sender, text)
            }
            "mailbox" -> {
                val parts = line.split("\t", limit = 2)
                val count = parts.getOrNull(1)?.toIntOrNull() ?: return
                onMailboxChecked?.invoke(count)
            }
            "anon_text" -> {
                val parts = line.split("\t", limit = 3)
                val matchedPeer = parts.getOrNull(1) ?: return
                val text = parts.getOrNull(2) ?: return
                onAnonTextReceived?.invoke(matchedPeer, text)
            }
            "attachment" -> {
                // Fixed-shape trailing fields (base64/number/MIME type,
                // never free text), unlike "text"/"group_text" above —
                // no embedded-tab risk, so an unlimited split is safe.
                val parts = line.split("\t")
                val sender = parts.getOrNull(1) ?: return
                val blobHashB64 = parts.getOrNull(2) ?: return
                val sizeBytes = parts.getOrNull(3)?.toLongOrNull() ?: return
                val mediaType = parts.getOrNull(4) ?: return
                val keyB64 = parts.getOrNull(5) ?: return
                onAttachmentReceived?.invoke(sender, blobHashB64, sizeBytes, mediaType, keyB64)
            }
            "group_invite" -> {
                val parts = line.split("\t", limit = 3)
                val conversation = parts.getOrNull(1) ?: return
                val fromDevice = parts.getOrNull(2) ?: return
                onGroupInvite?.invoke(conversation, fromDevice)
            }
            "group_text" -> {
                val parts = line.split("\t", limit = 4)
                val conversation = parts.getOrNull(1) ?: return
                val sender = parts.getOrNull(2) ?: return
                val text = parts.getOrNull(3) ?: return
                onGroupText?.invoke(conversation, sender, text)
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

    external fun initAndroidContext(context: android.content.Context)
    external fun bootstrap(filesDir: String): String
    external fun addPeer(ticket: String): String
    external fun ticketEndpointDebug(ticket: String): String
    external fun sendText(peerTicket: String, text: String): String
    external fun checkMailbox(relayTicket: String): String
    external fun sendTextAnon(peerTicket: String, relayTicket: String, text: String): String
    external fun checkMailboxAnon(peerTicket: String, relayTicket: String): String
    external fun deviceId(): String
    external fun accountId(): String
    external fun groupKeyPackage(): String
    external fun groupCreate(): String
    external fun groupAddMember(
        conversation: String,
        peerTicket: String,
        peerDeviceId: String,
        peerAccountId: String,
        keyPackageB64: String,
    ): String
    external fun groupSendText(conversation: String, text: String): String
    external fun groupJoin(conversation: String, groupStateB64: String): String
    external fun groupDeclineInvite(conversation: String): String
    external fun sendAttachment(peerTicket: String, fileBytesB64: String, mediaType: String): String
    external fun fetchAttachment(
        peerTicket: String,
        blobHashB64: String,
        encryptedSizeBytes: Long,
        mediaType: String,
        attachmentKeyB64: String,
    ): String
    external fun pollNextEvent(): String?
    external fun shutdown()
}
