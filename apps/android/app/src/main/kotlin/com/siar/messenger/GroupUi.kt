package com.siar.messenger

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.weight
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

/** Top-level switcher for the groups tab — same plain-nullable-selection
 * shape [MessengerScreen] itself uses, no navigation library (see that
 * function's own comment on why). */
@Composable
fun GroupsScreen() {
    var openConversation by remember { mutableStateOf<String?>(null) }
    val conversation = openConversation
    if (conversation == null) {
        GroupListScreen(onOpenGroup = { openConversation = it })
    } else {
        BackHandler { openConversation = null }
        val name = GroupStore.groups.firstOrNull { it.first == conversation }?.second ?: conversation.take(8)
        GroupConversationScreen(conversation = conversation, name = name, onBack = { openConversation = null })
    }
}

@Composable
private fun GroupListScreen(onOpenGroup: (String) -> Unit) {
    var showCreateDialog by remember { mutableStateOf(false) }
    var showIdentityDialog by remember { mutableStateOf(false) }
    var joiningInvite by remember { mutableStateOf<GroupInvite?>(null) }
    var addingMemberTo by remember { mutableStateOf<String?>(null) }

    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("Groups", fontWeight = FontWeight.Bold)
            Row {
                TextButton(onClick = { showIdentityDialog = true }) { Text("My identity") }
                Button(onClick = { showCreateDialog = true }) { Text("Create") }
            }
        }
        Spacer(modifier = Modifier.height(8.dp))

        if (GroupStore.invites.isNotEmpty()) {
            Text("Invites", fontWeight = FontWeight.Bold, style = MaterialTheme.typography.bodySmall)
            for (invite in GroupStore.invites) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text("From ${invite.fromDeviceDebug}", modifier = Modifier.weight(1f), maxLines = 1)
                    TextButton(onClick = { joiningInvite = invite }) { Text("Join") }
                    TextButton(onClick = { GroupStore.declineInvite(invite.conversation) }) { Text("Decline") }
                }
            }
            HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))
        }

        if (GroupStore.groups.isEmpty()) {
            Text("No groups yet — create one, or accept an invite above.")
        } else {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                items(GroupStore.groups, key = { it.first }) { (conversation, name) ->
                    Row(modifier = Modifier.fillMaxWidth().padding(vertical = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text(name, fontWeight = FontWeight.Bold, modifier = Modifier.weight(1f))
                        TextButton(onClick = { addingMemberTo = conversation }) { Text("Add member") }
                        TextButton(onClick = { onOpenGroup(conversation) }) { Text("Open") }
                    }
                    HorizontalDivider()
                }
            }
        }
    }

    if (showCreateDialog) {
        CreateGroupDialog(onDismiss = { showCreateDialog = false })
    }
    if (showIdentityDialog) {
        MyIdentityDialog(onDismiss = { showIdentityDialog = false })
    }
    val invite = joiningInvite
    if (invite != null) {
        JoinGroupDialog(invite = invite, onDismiss = { joiningInvite = null })
    }
    val addTarget = addingMemberTo
    if (addTarget != null) {
        AddMemberDialog(conversation = addTarget, onDismiss = { addingMemberTo = null })
    }
}

@Composable
private fun CreateGroupDialog(onDismiss: () -> Unit) {
    var name by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Create group") },
        text = {
            Column {
                OutlinedTextField(value = name, onValueChange = { name = it }, label = { Text("Group name") })
                val currentError = error
                if (currentError != null) {
                    Text(currentError, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
                }
            }
        },
        confirmButton = {
            Button(onClick = {
                GroupStore.createGroup(name).onSuccess { onDismiss() }.onFailure { e -> error = e.message ?: "Couldn't create group" }
            }) { Text("Create") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
    )
}

/** Shows this device's own device id / account id / ticket / key
 * package — see [GroupStore.myGroupIdentity]'s own doc comment. Purely
 * informational: the person copies these out to send to a group admin
 * by whatever channel they'd naturally use (an existing 1:1 chat in
 * this app, a messaging app outside it, reading them aloud). */
@Composable
private fun MyIdentityDialog(onDismiss: () -> Unit) {
    val identity = remember { GroupStore.myGroupIdentity() }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("My identity") },
        text = {
            Column {
                Text("Share these with a group admin so they can add you:")
                Spacer(modifier = Modifier.height(8.dp))
                IdentityField("Device ID", identity.deviceId)
                IdentityField("Account ID", identity.accountId)
                IdentityField("Ticket", identity.ticket)
                IdentityField("Key package", identity.keyPackageB64)
            }
        },
        confirmButton = { TextButton(onClick = onDismiss) { Text("Close") } },
    )
}

@Composable
private fun IdentityField(label: String, value: String?) {
    Column(modifier = Modifier.padding(vertical = 4.dp)) {
        Text(label, style = MaterialTheme.typography.bodySmall, fontWeight = FontWeight.Bold)
        Text(value ?: "(not available yet)", style = MaterialTheme.typography.bodySmall, maxLines = 3)
    }
}

@Composable
private fun AddMemberDialog(conversation: String, onDismiss: () -> Unit) {
    var peerTicket by remember { mutableStateOf("") }
    var peerDeviceId by remember { mutableStateOf("") }
    var peerAccountId by remember { mutableStateOf("") }
    var keyPackageB64 by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var resultState by remember { mutableStateOf<String?>(null) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Add member") },
        text = {
            val result = resultState
            if (result != null) {
                Column {
                    Text("Send this group state back to the new member, alongside the conversation id, for them to paste into Join:")
                    Spacer(modifier = Modifier.height(8.dp))
                    Text("Conversation: $conversation", style = MaterialTheme.typography.bodySmall)
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(result, style = MaterialTheme.typography.bodySmall, maxLines = 6)
                }
            } else {
                Column {
                    Text("Paste the new member's own identity fields:")
                    OutlinedTextField(value = peerTicket, onValueChange = { peerTicket = it }, label = { Text("Their ticket") })
                    OutlinedTextField(value = peerDeviceId, onValueChange = { peerDeviceId = it }, label = { Text("Their device id") })
                    OutlinedTextField(value = peerAccountId, onValueChange = { peerAccountId = it }, label = { Text("Their account id") })
                    OutlinedTextField(value = keyPackageB64, onValueChange = { keyPackageB64 = it }, label = { Text("Their key package") })
                    val currentError = error
                    if (currentError != null) {
                        Text(currentError, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
                    }
                }
            }
        },
        confirmButton = {
            if (resultState == null) {
                Button(onClick = {
                    GroupStore.addMember(conversation, peerTicket, peerDeviceId, peerAccountId, keyPackageB64)
                        .onSuccess { state -> resultState = state }
                        .onFailure { e -> error = e.message ?: "Couldn't add member" }
                }) { Text("Add") }
            } else {
                Button(onClick = onDismiss) { Text("Done") }
            }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text(if (resultState == null) "Cancel" else "Close") } },
    )
}

@Composable
private fun JoinGroupDialog(invite: GroupInvite, onDismiss: () -> Unit) {
    var name by remember { mutableStateOf("") }
    var groupStateB64 by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Join group") },
        text = {
            Column {
                Text("Paste the group state the admin sent you:")
                OutlinedTextField(value = name, onValueChange = { name = it }, label = { Text("Group name (for you)") })
                OutlinedTextField(value = groupStateB64, onValueChange = { groupStateB64 = it }, label = { Text("Group state") })
                val currentError = error
                if (currentError != null) {
                    Text(currentError, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
                }
            }
        },
        confirmButton = {
            Button(onClick = {
                GroupStore.joinGroup(invite.conversation, groupStateB64, name)
                    .onSuccess { onDismiss() }
                    .onFailure { e -> error = e.message ?: "Couldn't join group" }
            }) { Text("Join") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
    )
}

@Composable
private fun GroupConversationScreen(conversation: String, name: String, onBack: () -> Unit) {
    var draft by remember { mutableStateOf("") }
    var sendError by remember { mutableStateOf<String?>(null) }
    val messages = GroupStore.messagesFor(conversation)

    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            TextButton(onClick = onBack) { Text("< Back") }
            Text(name, fontWeight = FontWeight.Bold, modifier = Modifier.padding(start = 8.dp))
        }
        HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

        LazyColumn(modifier = Modifier.weight(1f).fillMaxWidth()) {
            items(messages) { message ->
                Box(modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp)) {
                    Text(
                        text = (if (message.fromMe) "You: " else "") + message.text,
                        modifier = Modifier.align(if (message.fromMe) Alignment.CenterEnd else Alignment.CenterStart),
                    )
                }
            }
        }

        val currentError = sendError
        if (currentError != null) {
            Text(currentError, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
        }

        Row(modifier = Modifier.fillMaxWidth().padding(top = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            OutlinedTextField(value = draft, onValueChange = { draft = it }, modifier = Modifier.weight(1f), placeholder = { Text("Message") })
            Button(onClick = {
                val text = draft
                if (text.isNotBlank()) {
                    GroupStore.sendText(conversation, text)
                        .onSuccess { draft = ""; sendError = null }
                        .onFailure { e -> sendError = e.message ?: "Send failed" }
                }
            }, modifier = Modifier.padding(start = 8.dp)) {
                Text("Send")
            }
        }
    }
}
