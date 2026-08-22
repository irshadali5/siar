//! Leaf components (plan.md §94: granular, each subscribing only to the
//! state slice it actually renders).

use crate::app::{base64_decode, LocalIdentity};
use crate::state::AppState;
use dioxus::prelude::*;
use siar_domain::{AccountId, DeviceId, MediaType, MessageText};
use siar_messaging::PeerTicket;
use siar_ui_state::{AddMemberInput, AppCommand, AttachmentPreview, ConversationKind};

#[component]
pub fn NetworkBadge() -> Element {
    let state = use_context::<AppState>();
    let overall = state.network.read().overall();
    let label = match overall {
        Some(siar_ui_state::NetworkState::Online) => "online",
        Some(siar_ui_state::NetworkState::Offline) => "offline",
        None => "connecting…",
    };
    rsx! {
        div { class: "network-badge", "{label}" }
    }
}

#[component]
pub fn ConversationList() -> Element {
    let state = use_context::<AppState>();
    // Clone the summaries out of the read guard *before* building rsx!
    // — `for summary in summaries { ... onclick: move |_| ... }` puts
    // each `summary` inside a `'static`-bound closure (Dioxus event
    // handlers own their captured data), which a borrow tied to
    // `state.conversations.read()`'s guard can't satisfy: that guard's
    // lifetime ends with this function, not with the closures it'd be
    // captured into. `ConversationSummary` already derives `Clone` for
    // exactly this kind of read-then-release-the-borrow use.
    let summaries: Vec<siar_ui_state::ConversationSummary> = state.conversations.read().ordered().to_vec();

    rsx! {
        ul { class: "conversation-list",
            for summary in summaries {
                li {
                    key: "{summary.id}",
                    onclick: move |_| state.dispatch(AppCommand::OpenConversation(summary.id)),
                    span {
                        class: if summary.kind == ConversationKind::Group { "conversation-kind group" } else { "conversation-kind direct" },
                        "{conversation_kind_icon(summary.kind)}"
                    }
                    span { class: "name", "{summary.display_name}" }
                    if summary.unread_count > 0 {
                        span { class: "unread-badge", "{summary.unread_count}" }
                    }
                    if let Some(preview) = &summary.last_message_preview {
                        p { class: "preview", "{preview}" }
                    }
                }
            }
        }
    }
}

fn conversation_kind_icon(kind: ConversationKind) -> &'static str {
    match kind {
        ConversationKind::Group => "👥",
        ConversationKind::Direct => "👤",
    }
}

#[component]
pub fn MessageTimeline() -> Element {
    let state = use_context::<AppState>();
    let timeline = state.timeline.read();

    rsx! {
        div { class: "timeline",
            if timeline.has_more_history() {
                button { class: "load-more", "Load earlier messages" }
            }
            for entry in timeline.visible() {
                div {
                    key: "{entry.message_id}",
                    class: if entry.from_me { "bubble mine" } else { "bubble theirs" },
                    match &entry.content {
                        siar_domain::MessageContent::Text(text) => rsx! { p { "{text.as_str()}" } },
                        siar_domain::MessageContent::Attachment(reference) => rsx! {
                            AttachmentBubble { message_id: entry.message_id, reference: reference.clone() }
                        },
                    }
                    span { class: "delivery-state", "{delivery_state_label(entry.delivery_state)}" }
                }
            }
        }
    }
}

/// Renders one `MessageContent::Attachment` — a still-image reference
/// gets decoded and shown inline (closing the gap `siar-media-image`'s
/// own crate doc comment left open: a fully-built codec layer nothing
/// in the UI ever called), anything else stays the honest byte-count
/// placeholder this always showed, since `siar-media-image` doesn't
/// decode audio/video and never claims to (codecs2.md's hard line
/// between still images and realtime media).
#[component]
fn AttachmentBubble(message_id: siar_domain::MessageId, reference: siar_domain::AttachmentReference) -> Element {
    let state = use_context::<AppState>();
    let is_previewable_image =
        matches!(reference.media_type, MediaType::ImagePng | MediaType::ImageJpeg | MediaType::ImageWebp);

    if !is_previewable_image {
        return rsx! {
            p { class: "attachment-placeholder",
                "[attachment, {reference.encrypted_size.bytes()} bytes]"
            }
        };
    }

    let preview = state.attachment_previews.read().get(message_id);
    match preview {
        AttachmentPreview::NotRequested => {
            // Dispatched from render rather than an explicit "load
            // image" button. `AppState::dispatch` is async (it sends
            // over an mpsc channel to `command_loop`, which is what
            // actually calls `set_loading`), so there's a real — if
            // narrow — window where more than one render of this same
            // still-`NotRequested` state could fire a duplicate
            // dispatch before the first one lands; `command_loop`
            // handling `LoadAttachmentPreview` idempotently (a second
            // fetch just overwrites the same `Ready`/`Failed` result)
            // is what makes that harmless rather than actually
            // preventing it. A real fix would need a synchronous
            // "already requested" flag set at dispatch time, not
            // attempted here.
            state.dispatch(AppCommand::LoadAttachmentPreview { message_id, reference: reference.clone() });
            rsx! {
                p { class: "attachment-placeholder",
                    "[loading image, {reference.encrypted_size.bytes()} bytes]"
                }
            }
        }
        AttachmentPreview::Loading => rsx! {
            p { class: "attachment-placeholder", "[loading image…]" }
        },
        AttachmentPreview::Ready { jpeg_bytes, width, height } => rsx! {
            img {
                class: "attachment-image",
                src: "{jpeg_data_uri(&jpeg_bytes)}",
                width: "{width}",
                height: "{height}",
            }
        },
        AttachmentPreview::Failed { reason } => rsx! {
            p { class: "attachment-placeholder attachment-failed", "[couldn't load image: {reason}]" }
        },
    }
}

/// Base64 `data:` URI for an already-encoded JPEG preview — the
/// simplest way to hand Dioxus's desktop webview an in-memory image
/// with no temp file and no custom asset scheme to wire up, at the
/// small cost of ~33% base64 overhead on an already-downscaled preview
/// (`generate_preview` caps the long side at
/// `siar_media_image::PREVIEW_MAX_DIMENSION`, so this stays well under
/// any URI-length concern in practice).
fn jpeg_data_uri(jpeg_bytes: &[u8]) -> String {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(jpeg_bytes);
    format!("data:image/jpeg;base64,{encoded}")
}

fn delivery_state_label(state: siar_domain::DeliveryState) -> String {
    use siar_domain::DeliveryState::*;
    match state {
        Local => "•".to_string(),
        Queued => "sending…".to_string(),
        Sending => "sending…".to_string(),
        Sent => "sent".to_string(),
        // next.md §62: carried-by-mesh is explicitly NOT delivery — must
        // read distinctly from "delivered" so the UI never shows a
        // misleading double-check for a bundle that only left this device.
        CarriedByPeers { copies } => format!("carried by {copies} peer(s)…"),
        Delivered => "delivered".to_string(),
        Read => "read".to_string(),
        Failed => "failed — retrying".to_string(),
        Expired => "expired".to_string(),
    }
}

#[component]
pub fn Composer(active_peer: Signal<Option<PeerTicket>>) -> Element {
    let mut state = use_context::<AppState>();
    let has_peer = active_peer.read().is_some();

    rsx! {
        div { class: "composer",
            input {
                value: "{state.composer.read().draft()}",
                disabled: !has_peer,
                placeholder: if has_peer { "Message…" } else { "Pair with a peer first" },
                oninput: move |evt| state.composer.write().set_draft(evt.value()),
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter {
                        submit(state);
                    }
                },
            }
            button {
                disabled: !has_peer || state.composer.read().is_empty(),
                onclick: move |_| submit(state),
                "Send"
            }
        }
    }
}

fn submit(mut state: AppState) {
    let result = state.composer.write().try_submit();
    match result {
        Ok(text) => {
            // Phase-3 stand-in (see `ActivePeer`'s doc comment in app.rs):
            // one conversation per process run, so a fresh
            // `ConversationId` per send is wrong for a real multi-chat
            // UI, but is what the current single-peer scope actually
            // needs — the conversation/contacts tables are what turn
            // this into a real per-peer conversation lookup.
            let conversation = siar_domain::ConversationId::new();
            state.dispatch(AppCommand::SendMessage { conversation, text });
        }
        Err(e) => tracing::warn!(error = %e, "message text failed validation"),
    }
}

#[component]
pub fn PeerPairing(active_peer: Signal<Option<PeerTicket>>) -> Element {
    let mut input = use_signal(String::new);

    rsx! {
        div { class: "peer-pairing",
            input {
                value: "{input}",
                placeholder: "paste their ticket",
                oninput: move |evt| input.set(evt.value()),
            }
            button {
                onclick: move |_| {
                    match PeerTicket::decode(&input.read()) {
                        Ok(peer) => active_peer.set(Some(peer)),
                        Err(e) => tracing::warn!(error = %e, "invalid peer ticket"),
                    }
                },
                "Pair"
            }
        }
    }
}

/// Saves the currently-pasted ticket as a persistent contact — a
/// second, deliberately separate step from `PeerPairing`'s "Pair"
/// button, not folded into it, because pairing (set as the active 1:1
/// peer for this session) and saving (persist to disk for every future
/// session) are different actions with different inputs: saving needs
/// the peer's account/device id too, which a bare ticket doesn't
/// carry (see `PeerTicket`'s own fields — no id, just endpoint/keys),
/// so this form asks for what pairing alone doesn't need.
#[component]
pub fn SaveContactForm() -> Element {
    let state = use_context::<AppState>();
    let mut display_name = use_signal(String::new);
    let mut account_text = use_signal(String::new);
    let mut device_text = use_signal(String::new);
    let mut ticket_text = use_signal(String::new);

    rsx! {
        div { class: "save-contact-form",
            input { value: "{display_name}", placeholder: "name", oninput: move |evt| display_name.set(evt.value()) }
            input { value: "{account_text}", placeholder: "their account id", oninput: move |evt| account_text.set(evt.value()) }
            input { value: "{device_text}", placeholder: "their device id", oninput: move |evt| device_text.set(evt.value()) }
            input { value: "{ticket_text}", placeholder: "their ticket", oninput: move |evt| ticket_text.set(evt.value()) }
            button {
                onclick: move |_| {
                    let account: Result<AccountId, _> = account_text.read().trim().parse();
                    let device: Result<DeviceId, _> = device_text.read().trim().parse();
                    match (account, device) {
                        (Ok(account_id), Ok(device_id)) => {
                            state.dispatch(AppCommand::SaveContact {
                                device_id,
                                account_id,
                                display_name: display_name.read().clone(),
                                ticket_text: ticket_text.read().clone(),
                                key_package_b64: None,
                            });
                            display_name.set(String::new());
                            account_text.set(String::new());
                            device_text.set(String::new());
                            ticket_text.set(String::new());
                        }
                        _ => tracing::warn!("save-contact form dropped — check the account id and device id fields"),
                    }
                },
                "Save as contact"
            }
        }
    }
}

/// The persistent contact list itself — "Use" sets a saved contact as
/// the active 1:1 peer without re-pasting its ticket (the gap this
/// whole pass closes); "Remove" deletes it from disk.
#[component]
pub fn ContactList(active_peer: Signal<Option<PeerTicket>>) -> Element {
    let state = use_context::<AppState>();
    let contacts = state.contacts.read().ordered().to_vec();

    rsx! {
        div { class: "contact-list",
            h4 { "Saved contacts" }
            SaveContactForm {}
            ul {
                for contact in contacts {
                    li {
                        key: "{contact.device_id}",
                        span { class: "name", "{contact.display_name}" }
                        button {
                            onclick: move |_| {
                                match PeerTicket::decode(&contact.ticket_text) {
                                    Ok(peer) => active_peer.set(Some(peer)),
                                    Err(e) => tracing::warn!(error = %e, "saved contact has an invalid ticket"),
                                }
                            },
                            "Use"
                        }
                        button {
                            onclick: move |_| state.dispatch(AppCommand::RemoveContact { device_id: contact.device_id }),
                            "Remove"
                        }
                    }
                }
            }
        }
    }
}

/// Group messaging UI (the gap this whole pass closes): create a
/// group, publish/display this device's key package so an admin
/// elsewhere can add it, add a member to a group this device admins,
/// accept/decline an incoming invite, and send/view group messages.
///
/// Deliberately its own panel rather than folded into `ConversationList`/
/// `Composer` — those two stay 1:1-only (see `ActivePeer`'s doc comment
/// in app.rs on why "one active peer" is still the whole 1:1 model);
/// unifying group and direct conversations behind one selected-item/
/// composer pair is real follow-up work, not attempted here alongside
/// getting group actions reachable at all for the first time.
#[component]
pub fn GroupPanel(my_key_package_text: Signal<String>) -> Element {
    let state = use_context::<AppState>();
    let local_identity = use_context::<Signal<Option<LocalIdentity>>>();
    let mut selected_group = use_signal(|| None::<siar_domain::ConversationId>);

    let groups = state.groups.read().ordered().to_vec();
    let invites = state.pending_invites.read().pending().to_vec();

    rsx! {
        div { class: "group-panel",
            h3 { "Groups" }

            details { class: "my-key-package",
                summary { "Your key package (share this to be added to a group)" }
                p { class: "key-package-text", "{my_key_package_text}" }
            }

            button {
                disabled: local_identity.read().is_none(),
                onclick: move |_| {
                    if let Some(identity) = *local_identity.read() {
                        state.dispatch(AppCommand::CreateGroup { founder: identity.account });
                    }
                },
                "Create group"
            }

            if !invites.is_empty() {
                div { class: "pending-invites",
                    h4 { "Invites" }
                    for invite in invites {
                        InviteBanner { invite: invite.clone() }
                    }
                }
            }

            ul { class: "group-list",
                for group in groups {
                    li {
                        key: "{group.conversation_id}",
                        class: if selected_group.read().as_ref() == Some(&group.conversation_id) { "selected" } else { "" },
                        onclick: move |_| selected_group.set(Some(group.conversation_id)),
                        span { class: "name", "{group.display_label}" }
                        span { class: "member-count", "{group.member_count} member(s)" }
                        if group.is_admin {
                            span { class: "admin-badge", "admin" }
                        }
                    }
                }
            }

            if let Some(conversation) = *selected_group.read() {
                GroupDetail { conversation }
            }
        }
    }
}

/// One pending `GroupMlsWelcome`. The paste box is the honest surface
/// of a real backend limitation — see `PendingInviteState`'s own doc
/// comment — not a UI shortcut being hidden from the user: accepting
/// requires the founder's `GroupState`, which currently only travels
/// out-of-band (the same way `apps/cli`'s `join-group <state>` argument
/// already needs it).
#[component]
fn InviteBanner(invite: siar_ui_state::PendingInvite) -> Element {
    let mut state = use_context::<AppState>();
    let conversation = invite.conversation_id;
    let mut state_input = use_signal(|| invite.state_input.clone());

    rsx! {
        div { class: "invite-banner",
            p { "Group invite from device {invite.from_device.fmt_short()}" }
            textarea {
                value: "{state_input}",
                placeholder: "paste the group state the admin sent you out-of-band",
                oninput: move |evt| state_input.set(evt.value()),
            }
            button {
                onclick: move |_| {
                    state.pending_invites.write().set_state_input(conversation, state_input.read().clone());
                    state.dispatch(AppCommand::AcceptGroupInvite { conversation });
                },
                "Accept"
            }
            button {
                onclick: move |_| state.dispatch(AppCommand::DeclineGroupInvite { conversation }),
                "Decline"
            }
        }
    }
}

/// The selected group's add-member form, message history slice, and
/// composer. Message history isn't filtered from `state.timeline` by
/// conversation yet — same Phase-3 single-active-conversation
/// simplification `MessageTimeline` already has for 1:1 chats (see
/// that component's own lack of per-conversation filtering); flagged
/// here rather than silently inherited, since a multi-group user would
/// notice this sooner than a single 1:1 user would.
#[component]
fn GroupDetail(conversation: siar_domain::ConversationId) -> Element {
    let state = use_context::<AppState>();
    let mut draft = use_signal(String::new);
    let mut peer_account_text = use_signal(String::new);
    let mut peer_device_text = use_signal(String::new);
    let mut peer_ticket_text = use_signal(String::new);
    let mut key_package_text = use_signal(String::new);

    rsx! {
        div { class: "group-detail",
            h4 { "Add member" }
            div { class: "add-member-form",
                input { value: "{peer_account_text}", placeholder: "their account id", oninput: move |evt| peer_account_text.set(evt.value()) }
                input { value: "{peer_device_text}", placeholder: "their device id", oninput: move |evt| peer_device_text.set(evt.value()) }
                input { value: "{peer_ticket_text}", placeholder: "their ticket", oninput: move |evt| peer_ticket_text.set(evt.value()) }
                textarea { value: "{key_package_text}", placeholder: "their key package (base64)", oninput: move |evt| key_package_text.set(evt.value()) }
                button {
                    onclick: move |_| submit_add_member(state, conversation, peer_account_text, peer_device_text, peer_ticket_text, key_package_text),
                    "Add"
                }
            }

            div { class: "group-composer",
                input {
                    value: "{draft}",
                    placeholder: "Message the group…",
                    oninput: move |evt| draft.set(evt.value()),
                    onkeydown: move |evt| {
                        if evt.key() == Key::Enter {
                            submit_group_message(state, conversation, draft);
                        }
                    },
                }
                button {
                    onclick: move |_| submit_group_message(state, conversation, draft),
                    "Send"
                }
            }
        }
    }
}

/// Free function, not an inline closure, so it can be called from both
/// `onclick` and `onkeydown` without the two call sites forcing
/// conflicting inferred parameter types on a shared closure — same
/// reason this file's original `Composer`/`submit` pair (above) is
/// already split this way.
fn submit_group_message(state: AppState, conversation: siar_domain::ConversationId, mut draft: Signal<String>) {
    let text = draft.read().clone();
    match MessageText::parse(text) {
        Ok(text) => {
            draft.set(String::new());
            state.dispatch(AppCommand::SendGroupMessage { conversation, text });
        }
        Err(e) => tracing::warn!(error = %e, "group message text failed validation"),
    }
}

fn submit_add_member(
    state: AppState,
    conversation: siar_domain::ConversationId,
    mut peer_account_text: Signal<String>,
    mut peer_device_text: Signal<String>,
    mut peer_ticket_text: Signal<String>,
    mut key_package_text: Signal<String>,
) {
    let account: Result<AccountId, _> = peer_account_text.read().trim().parse();
    let device: Result<DeviceId, _> = peer_device_text.read().trim().parse();
    let key_package_bytes = base64_decode(&key_package_text.read());

    match (account, device, key_package_bytes) {
        (Ok(peer_account), Ok(peer_device), Ok(key_package_bytes)) => {
            state.dispatch(AppCommand::AddGroupMember {
                conversation,
                new_member: peer_account,
                input: AddMemberInput {
                    peer_account,
                    peer_device,
                    peer_ticket_text: peer_ticket_text.read().clone(),
                    key_package_bytes,
                },
            });
            peer_account_text.set(String::new());
            peer_device_text.set(String::new());
            peer_ticket_text.set(String::new());
            key_package_text.set(String::new());
        }
        _ => tracing::warn!("add-member form dropped — check account id, device id, and key package fields"),
    }
}
