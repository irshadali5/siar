package com.siar.messenger

import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.snapshots.SnapshotStateList
import com.siar.messaging.MessagingBridge

data class GroupMessage(val text: String, val fromMe: Boolean, val timestampMillis: Long)

/** A pending invite this device has not yet accepted or declined — see
 * `siar-android-messaging`'s own `pending_welcomes` doc comment for
 * where this comes from on the native side. */
data class GroupInvite(val conversation: String, val fromDeviceDebug: String)

/**
 * The groups/MLS half of the chat UI, mirroring [ChatStore]'s own shape
 * exactly (a plain Kotlin `object`, in-memory only — see that class's
 * own doc comment for why). Adding a member is a real multi-step,
 * out-of-band exchange (matching `apps/cli`/`apps/desktop`'s own group
 * flow — this codebase has no wire-carried key-package discovery, see
 * `siar-android-messaging`'s top doc comment): the new member shares
 * their [MessagingBridge.deviceId]/[MessagingBridge.accountId]/own
 * ticket/[MessagingBridge.groupKeyPackage] with the admin somehow
 * (pasted through an existing 1:1 chat, read aloud, however two people
 * actually exchange a string in real life); the admin calls
 * [addMember], which returns a group-state blob to relay back the same
 * way; the new member pastes that into [joinGroup].
 */
object GroupStore {
    /** conversation id -> display name, chosen locally by whoever
     * created or joined the group; not synced with other members (this
     * codebase has no group-metadata/naming sync — a real, named
     * limitation, not attempted here). */
    val groups: SnapshotStateList<Pair<String, String>> = mutableStateListOf()

    val invites: SnapshotStateList<GroupInvite> = mutableStateListOf()

    private val messagesByConversation = HashMap<String, SnapshotStateList<GroupMessage>>()

    fun messagesFor(conversation: String): SnapshotStateList<GroupMessage> =
        messagesByConversation.getOrPut(conversation) { mutableStateListOf() }

    /** Creates a new group founded by this device, with the given local
     * display name. */
    fun createGroup(name: String): Result<String> =
        MessagingBridge.groupCreate().onSuccess { conversation ->
            groups.add(conversation to name.trim().ifEmpty { conversation.take(8) })
            messagesFor(conversation)
        }

    /** Adds a member using details obtained from them out-of-band (see
     * this object's own doc comment) and remembers the returned group
     * state so the caller can relay it back to the new member. */
    fun addMember(
        conversation: String,
        peerTicket: String,
        peerDeviceId: String,
        peerAccountId: String,
        keyPackageB64: String,
    ): Result<String> = MessagingBridge.groupAddMember(conversation, peerTicket, peerDeviceId, peerAccountId, keyPackageB64)

    fun sendText(conversation: String, text: String): Result<Unit> =
        MessagingBridge.groupSendText(conversation, text).map {
            messagesFor(conversation).add(GroupMessage(text, fromMe = true, timestampMillis = System.currentTimeMillis()))
        }

    /** Joins using a group-state blob obtained out-of-band from the
     * admin's [addMember] call — see this object's own doc comment. On
     * success, removes the matching [invites] entry (if any — a device
     * could in principle join without ever having seen the invite
     * banner, e.g. after a restart lost the buffered welcome and it was
     * re-sent) and gives the new group a locally-chosen display name. */
    fun joinGroup(conversation: String, groupStateB64: String, name: String): Result<Unit> =
        MessagingBridge.groupJoin(conversation, groupStateB64).onSuccess {
            invites.removeAll { it.conversation == conversation }
            if (groups.none { it.first == conversation }) {
                groups.add(conversation to name.trim().ifEmpty { conversation.take(8) })
            }
            messagesFor(conversation)
        }

    fun declineInvite(conversation: String) {
        MessagingBridge.groupDeclineInvite(conversation)
        invites.removeAll { it.conversation == conversation }
    }

    /** Called from [MessagingBridge.onGroupInvite]. */
    fun recordInvite(conversation: String, fromDeviceDebug: String) {
        if (invites.none { it.conversation == conversation } && groups.none { it.first == conversation }) {
            invites.add(GroupInvite(conversation, fromDeviceDebug))
        }
    }

    /** Called from [MessagingBridge.onGroupText]. */
    fun recordIncoming(conversation: String, text: String) {
        messagesFor(conversation).add(GroupMessage(text, fromMe = false, timestampMillis = System.currentTimeMillis()))
        if (groups.none { it.first == conversation }) {
            // A text frame for a group this device apparently already
            // joined (MLS wouldn't decrypt it otherwise) but has no
            // local display name for — e.g. after a fresh process
            // start, since group membership itself isn't persisted
            // across restarts any more than contacts are (see
            // `ChatStore`'s own doc comment on this app's in-memory-only
            // scope). Shown under a placeholder name rather than
            // dropped.
            groups.add(conversation to "Group (${conversation.take(8)})")
        }
    }

    /** This device's own identity bundle for sharing with a group
     * admin — see this object's own doc comment. `null` fields mean
     * that particular native call failed (surfaced as `null` rather
     * than throwing, since this is meant for direct display in a
     * "share these with the admin" screen). */
    fun myGroupIdentity(): GroupIdentity = GroupIdentity(
        deviceId = MessagingBridge.deviceId().getOrNull(),
        accountId = MessagingBridge.accountId().getOrNull(),
        ticket = ChatStore.myTicket.value,
        keyPackageB64 = MessagingBridge.groupKeyPackage().getOrNull(),
    )
}

data class GroupIdentity(val deviceId: String?, val accountId: String?, val ticket: String?, val keyPackageB64: String?)
