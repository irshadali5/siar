package com.siar.messenger

import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.snapshots.SnapshotStateList
import com.siar.messaging.MessagingBridge

/**
 * A registered peer, as far as this app's UI is concerned. `ticket` is
 * the opaque `PeerTicket` string [MessagingBridge.addPeer] was given
 * (needed again for every [MessagingBridge.sendText] call); `endpointKey`
 * is what [MessagingBridge.ticketEndpointDebug] decoded it to — the
 * same key [MessagingBridge.onTextReceived] delivers for messages
 * *from* this contact. Two different strings for the same peer,
 * deliberately kept as two separate fields rather than assumed
 * interchangeable — see [ChatStore.addContact]'s own comment.
 */
data class Contact(val nickname: String, val ticket: String, val endpointKey: String)

data class ChatMessage(val text: String, val fromMe: Boolean, val timestampMillis: Long, val anon: Boolean = false)

/**
 * The Android chat UI this whole app was still missing — [MessagingBridge]
 * itself has been a real send/receive surface for a while (see that
 * class's own doc comment), but until this file there was no
 * contact list, no conversation screen, and no way to actually type a
 * message and see a thread. This is a plain Kotlin `object`
 * (Compose-observable via `mutableStateListOf`/`mutableStateOf`
 * directly, no `ViewModel` — this module has no
 * `lifecycle-viewmodel-compose` dependency, and `MessagingBridge`
 * itself already uses this same plain-`object` shape), not a database:
 * contacts and messages live only in process memory and are gone on
 * process death — real, honest scope, matching this crate's own doc
 * comment about `siar-android-messaging` not surfacing individual
 * mailbox item contents yet either. The native side already persists
 * messages to `siar.db` (see that crate's own `bootstrap_inner` doc
 * comment); there's just no FFI yet to read that history back out on
 * a fresh launch — a real, separate follow-up, not attempted here.
 */
object ChatStore {
    /** This device's own shareable ticket, set once [MainActivity]'s
     * `startMessaging` bootstrap call succeeds — `null` until then, so
     * the contact-list screen can show "still connecting" instead of a
     * blank/wrong value. */
    val myTicket = mutableStateOf<String?>(null)

    val contacts: SnapshotStateList<Contact> = mutableStateListOf()

    private val messagesByEndpointKey = HashMap<String, SnapshotStateList<ChatMessage>>()

    /** Returns the live, Compose-observable message list for a contact
     * — same list instance every call for a given key, created empty
     * on first access, so a [ConversationScreen] recomposes correctly
     * as new messages are appended to it. */
    fun messagesFor(endpointKey: String): SnapshotStateList<ChatMessage> =
        messagesByEndpointKey.getOrPut(endpointKey) { mutableStateListOf() }

    /** Registers a peer with the native side, decodes its ticket into
     * the sender key incoming messages will arrive keyed by (see
     * [Contact]'s own doc comment for why both are needed — a
     * `PeerTicket` and its `{:?}`-formatted `EndpointId` aren't the
     * same string even though they identify the same peer), and adds
     * it to the visible contact list. Two native calls, not one — if
     * `addPeer` succeeds but the decode somehow doesn't (it shouldn't,
     * since it's the same ticket string, but this doesn't assume that
     * can never fail), the peer is left registered for decryption
     * without a visible contact row rather than silently swallowing
     * the second error. */
    fun addContact(nickname: String, ticket: String): Result<Contact> {
        val trimmedTicket = ticket.trim()
        val trimmedNickname = nickname.trim().ifEmpty { trimmedTicket.take(12) }
        return MessagingBridge.addPeer(trimmedTicket).mapCatching {
            val endpointKey = MessagingBridge.ticketEndpointDebug(trimmedTicket).getOrThrow()
            val contact = Contact(trimmedNickname, trimmedTicket, endpointKey)
            contacts.removeAll { it.endpointKey == endpointKey } // re-adding an existing contact replaces its row rather than duplicating it
            contacts.add(contact)
            messagesFor(endpointKey) // ensure a (possibly already-populated, if messages arrived before this contact was named) list exists
            contact
        }
    }

    /** Sends a text message to an already-added [Contact] and appends
     * it to the local thread immediately on success — the native side
     * has no "message sent" event of its own to echo back (see
     * `siar-android-messaging`'s own doc comment on what incoming
     * event kinds exist), so this is the only place an outgoing
     * message ever gets recorded. */
    fun sendText(contact: Contact, text: String): Result<Unit> =
        MessagingBridge.sendText(contact.ticket, text).map {
            messagesFor(contact.endpointKey).add(ChatMessage(text, fromMe = true, timestampMillis = System.currentTimeMillis()))
        }

    /** Called from [MessagingBridge.onTextReceived]/[onAnonTextReceived].
     * `senderKey` matches a [Contact.endpointKey] when the sender is
     * already a known contact; when it isn't (a message arrived from a
     * peer never added via [addContact] — e.g. this device only ever
     * called `addPeer` indirectly, or the two devices raced each
     * other's first message), the message is still kept, filed under
     * that raw key, rather than dropped — [ContactListScreen] shows it
     * as an unnamed contact the person can still open and reply to. */
    fun recordIncoming(senderKey: String, text: String, anon: Boolean) {
        messagesFor(senderKey).add(ChatMessage(text, fromMe = false, timestampMillis = System.currentTimeMillis(), anon = anon))
        if (contacts.none { it.endpointKey == senderKey }) {
            // No ticket to re-register with `addPeer`/send future
            // messages to yet — just a placeholder so the thread shows
            // up in the list. `ticket` is left equal to the key itself
            // as an honest "not a real ticket" placeholder; sending to
            // this contact will fail until the real ticket is added
            // via [addContact], which replaces this row (see that
            // function's own dedup-by-endpointKey comment).
            contacts.add(Contact(nickname = "Unknown ($senderKey)", ticket = senderKey, endpointKey = senderKey))
        }
    }
}
