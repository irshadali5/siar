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
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/**
 * Top-level switcher between the contact list and one open conversation
 * — a plain nullable-selection state, not `androidx.navigation`
 * (`app/build.gradle.kts` has no `navigation-compose` dependency, and
 * a two-screen app doesn't need a back stack library; [BackHandler],
 * from the already-present `activity-compose` dependency, covers the
 * system back button for the one screen that needs it).
 */
@Composable
fun MessengerScreen() {
    var openContact by remember { mutableStateOf<Contact?>(null) }
    val contact = openContact
    if (contact == null) {
        ContactListScreen(onOpenContact = { openContact = it })
    } else {
        BackHandler { openContact = null }
        ConversationScreen(contact = contact, onBack = { openContact = null })
    }
}

@Composable
private fun ContactListScreen(onOpenContact: (Contact) -> Unit) {
    var showAddDialog by remember { mutableStateOf(false) }
    val myTicket = ChatStore.myTicket.value

    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("Contacts", fontWeight = FontWeight.Bold)
            Button(onClick = { showAddDialog = true }) { Text("Add") }
        }
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = if (myTicket == null) "Your ticket: connecting…" else "Your ticket: $myTicket",
            style = MaterialTheme.typography.bodySmall,
        )
        Spacer(modifier = Modifier.height(12.dp))

        if (ChatStore.contacts.isEmpty()) {
            Text("No contacts yet — tap Add and paste a peer's ticket to start a conversation.")
        } else {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                items(ChatStore.contacts, key = { it.endpointKey }) { contact ->
                    ContactRow(contact = contact, onClick = { onOpenContact(contact) })
                    HorizontalDivider()
                }
            }
        }
    }

    if (showAddDialog) {
        AddContactDialog(
            onDismiss = { showAddDialog = false },
            onAdd = { nickname, ticket ->
                ChatStore.addContact(nickname, ticket)
                showAddDialog = false
            },
        )
    }
}

@Composable
private fun ContactRow(contact: Contact, onClick: () -> Unit) {
    val lastMessage = ChatStore.messagesFor(contact.endpointKey).lastOrNull()
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 12.dp),
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(contact.nickname, fontWeight = FontWeight.Bold)
            if (lastMessage != null) {
                Text(
                    text = (if (lastMessage.fromMe) "You: " else "") + lastMessage.text,
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                )
            }
        }
        TextButton(onClick = onClick) { Text("Open") }
    }
}

@Composable
private fun AddContactDialog(onDismiss: () -> Unit, onAdd: (nickname: String, ticket: String) -> Unit) {
    var nickname by remember { mutableStateOf("") }
    var ticket by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Add contact") },
        text = {
            Column {
                OutlinedTextField(value = nickname, onValueChange = { nickname = it }, label = { Text("Name") })
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = ticket,
                    onValueChange = { ticket = it },
                    label = { Text("Peer ticket") },
                    placeholder = { Text("Paste the ticket they shared") },
                )
                val currentError = error
                if (currentError != null) {
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(currentError, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
                }
            }
        },
        confirmButton = {
            Button(onClick = {
                if (ticket.isBlank()) {
                    error = "Paste a ticket first"
                } else {
                    val result = ChatStore.addContact(nickname, ticket)
                    result.onSuccess { onAdd(nickname, ticket) }
                    result.onFailure { e -> error = e.message ?: "Couldn't add that ticket" }
                }
            }) { Text("Add") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
    )
}

@Composable
private fun ConversationScreen(contact: Contact, onBack: () -> Unit) {
    var draft by remember { mutableStateOf("") }
    var sendError by remember { mutableStateOf<String?>(null) }
    val messages = ChatStore.messagesFor(contact.endpointKey)

    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            TextButton(onClick = onBack) { Text("< Back") }
            Text(contact.nickname, fontWeight = FontWeight.Bold, modifier = Modifier.padding(start = 8.dp))
        }
        HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

        LazyColumn(modifier = Modifier.weight(1f).fillMaxWidth()) {
            items(messages) { message -> MessageBubble(message) }
        }

        val currentError = sendError
        if (currentError != null) {
            Text(currentError, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
        }

        Row(modifier = Modifier.fillMaxWidth().padding(top = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            OutlinedTextField(
                value = draft,
                onValueChange = { draft = it },
                modifier = Modifier.weight(1f),
                placeholder = { Text("Message") },
            )
            Button(onClick = {
                val text = draft
                if (text.isNotBlank()) {
                    // Real-looking peer ticket placeholders (see
                    // `ChatStore.recordIncoming`'s own comment) can't
                    // actually be sent to — the native `send_text`
                    // call will fail with a real decode error, and
                    // that error is what surfaces here rather than a
                    // generic message, since it's genuinely the
                    // accurate explanation.
                    val result = ChatStore.sendText(contact, text)
                    result.onSuccess { draft = ""; sendError = null }
                    result.onFailure { e -> sendError = e.message ?: "Send failed" }
                }
            }, modifier = Modifier.padding(start = 8.dp)) {
                Text("Send")
            }
        }
    }
}

@Composable
private fun MessageBubble(message: ChatMessage) {
    val alignment = if (message.fromMe) Alignment.CenterEnd else Alignment.CenterStart
    Box(modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp)) {
        Column(
            modifier = Modifier.align(alignment),
        ) {
            Text(message.text)
            Text(
                text = timeLabel(message.timestampMillis) + if (message.anon) " · anon" else "",
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}

private val timeFormat = SimpleDateFormat("HH:mm", Locale.getDefault())

private fun timeLabel(millis: Long): String = timeFormat.format(Date(millis))
