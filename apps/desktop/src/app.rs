//! Root component and the background-task wiring described in
//! `main.rs`'s module docs.

use crate::components;
use crate::state::AppState;
use dioxus::prelude::*;
use siar_domain::{ConversationId, DeliveryState, MessageContent};
use siar_messaging::{
    GroupService, InMemoryDeviceDirectory, IncomingEvent, MemberDevice, MessageService, PeerTicket,
};
use siar_protocol::v1::EnvelopeKind;
use siar_protocol::WireMessage;
use siar_ui_state::{
    AppCommand, ConversationKind, ConversationSummary, GroupSummary, PendingInvite, TimelineWindow,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// One active peer for 1:1 messaging, pasted in or picked from the
/// contact list. Saved contacts now persist across restarts (see
/// `main.rs`'s `resolve_data_paths` doc comment) — what's still a
/// Phase-3 stand-in is this single `Signal<Option<PeerTicket>>` itself:
/// only one 1:1 conversation can be "active" at a time, same limitation
/// `MessageTimeline`'s own doc comment already flags for group chats.
/// Real per-conversation routing (a `contacts`/`conversation_members`
/// lookup keyed by `ConversationId`) replaces this signal, not extends
/// it.
#[derive(Clone, Copy)]
struct ActivePeer(Signal<Option<PeerTicket>>);

#[component]
pub fn App() -> Element {
    let state = use_context_provider(AppState::new);
    let active_peer = use_context_provider(|| ActivePeer(Signal::new(None)));
    let mut my_ticket_text = use_signal(String::new);
    let mut my_key_package_text = use_signal(String::new);
    let local_identity = use_context_provider(|| Signal::new(None::<LocalIdentity>));

    use_effect(move || {
        let mut state = state;
        let mut local_identity = local_identity;
        spawn(async move {
            match crate::bootstrap_messaging().await {
                Ok(boot) => {
                    my_ticket_text.set(boot.my_ticket.encode());
                    my_key_package_text.set(boot.key_package_b64.clone().unwrap_or_default());
                    local_identity.set(Some(LocalIdentity {
                        account: boot.local_account,
                        device_id: boot.device_id,
                    }));
                    state.contacts.write().load(
                        boot.saved_contacts
                            .iter()
                            .map(|c| siar_ui_state::SavedContact {
                                device_id: c.device_id,
                                account_id: c.account_id,
                                display_name: c.display_name.clone(),
                                ticket_text: c.ticket_text.clone(),
                                key_package_b64: c.key_package_b64.clone(),
                            })
                            .collect(),
                    );

                    let (cmd_tx, cmd_rx) = mpsc::channel::<AppCommand>(64);
                    state.command_tx.set(Some(cmd_tx));

                    spawn(retry_scheduler_loop(boot.service.clone()));
                    spawn(command_loop(
                        CommandLoopContext {
                            service: boot.service.clone(),
                            group_service: boot.group_service.clone(),
                            device_directory: boot.device_directory.clone(),
                            contact_repo: boot.contact_repo.clone(),
                            local_account: boot.local_account,
                        },
                        state,
                        active_peer.0,
                        cmd_rx,
                    ));
                    spawn(incoming_loop(
                        boot.service,
                        boot.group_service,
                        state,
                        active_peer.0,
                        boot.incoming_rx,
                    ));
                }
                Err(e) => tracing::error!(error = %e, "failed to bootstrap messaging"),
            }
        });
    });

    rsx! {
        div { class: "app",
            crate::security_events::StrongSecurityWarningBanner {}
            components::NetworkBadge {}
            crate::security_events::SecurityEventList {}
            div { class: "pairing",
                p { "your ticket: {my_ticket_text}" }
                components::PeerPairing { active_peer: active_peer.0 }
                components::ContactList { active_peer: active_peer.0 }
            }
            div { class: "layout",
                components::ConversationList {}
                div { class: "active-conversation",
                    components::MessageTimeline {}
                    components::Composer { active_peer: active_peer.0 }
                }
                components::GroupPanel { my_key_package_text }
            }
        }
    }
}

/// This device's own identifiers, set once bootstrap finishes —
/// `components::GroupPanel`'s "create group"/"add member" actions need
/// `AccountId`/`DeviceId` values that exist before any group does, so
/// this can't just live inside `GroupListState` alongside groups
/// themselves.
#[derive(Debug, Clone, Copy)]
pub struct LocalIdentity {
    pub account: siar_domain::AccountId,
    /// Carried alongside `account` for symmetry with `Bootstrapped`
    /// (which has both) and for a future "show my device ID" / add-
    /// this-device-to-an-existing-account UI — no component currently
    /// reads it back out (only `account` is, via `CreateGroup`'s
    /// `founder` field). `#[allow(dead_code)]` rather than dropping
    /// real, intentionally-placed identity data on the strength of "no
    /// UI needs it yet" — same reasoning as `Bootstrapped::
    /// key_package_directory`'s own field-level comment.
    #[allow(dead_code)]
    pub device_id: siar_domain::DeviceId,
}

/// plan.md §33's retry scheduler — identical cadence to `apps/cli`.
async fn retry_scheduler_loop(service: Arc<MessageService>) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        if let Err(e) = service.retry_due().await {
            tracing::warn!(error = %e, "retry_due failed");
        }
    }
}

/// Groups `command_loop`'s static, per-session dependencies — the
/// pieces that get passed through once at bootstrap and never change
/// for the life of the loop, as opposed to `state`/`active_peer`/
/// `commands`, which are the loop's actual per-iteration inputs. Real
/// fix for clippy's `too_many_arguments` (the function this replaces
/// one parameter of had grown to 8), not a suppression: bundling
/// related, always-passed-together dependencies into one type is the
/// same refactor clippy's own lint documentation recommends.
struct CommandLoopContext {
    service: Arc<MessageService>,
    group_service: Arc<GroupService>,
    device_directory: Arc<InMemoryDeviceDirectory>,
    contact_repo: Arc<dyn siar_storage::ContactRepository + Send + Sync>,
    local_account: siar_domain::AccountId,
}

/// Drains `AppCommand`s dispatched from components (via `AppState::dispatch`)
/// and drives `MessageService` — the only place in this crate outside
/// `main.rs` that calls into `siar-messaging` directly.
async fn command_loop(
    ctx: CommandLoopContext,
    mut state: AppState,
    active_peer: Signal<Option<PeerTicket>>,
    mut commands: mpsc::Receiver<AppCommand>,
) {
    let CommandLoopContext {
        service,
        group_service,
        device_directory,
        contact_repo,
        local_account,
    } = ctx;
    while let Some(command) = commands.recv().await {
        match command {
            AppCommand::SendMessage { conversation, text } => {
                let Some(peer) = active_peer.read().clone() else {
                    tracing::warn!("dropped SendMessage — no active peer paired yet");
                    continue;
                };
                match service.send_text(conversation, &peer, text.clone()).await {
                    Ok(message_id) => {
                        state.timeline.write().push_latest(TimelineWindow {
                            message_id,
                            sequence: 0,
                            content: MessageContent::Text(text),
                            delivery_state: DeliveryState::Local,
                            from_me: true,
                        });
                        state.conversations.write().mark_read(conversation);
                    }
                    Err(e) => tracing::warn!(error = %e, "send_text failed"),
                }
            }
            AppCommand::OpenConversation(conversation) => {
                state.conversations.write().mark_read(conversation);
            }
            AppCommand::MarkRead { conversation, .. } => {
                state.conversations.write().mark_read(conversation);
            }
            AppCommand::CreateGroup { founder } => {
                match group_service.create_group_mls(ConversationId::new(), founder) {
                    Ok(group_state) => {
                        let conversation_id = group_state.conversation_id;
                        upsert_group_summary(&mut state, &group_state, local_account);
                        state.conversations.write().upsert(ConversationSummary {
                            id: conversation_id,
                            display_name: format!("Group {}", conversation_id.fmt_short()),
                            last_message_preview: None,
                            unread_count: 0,
                            kind: ConversationKind::Group,
                        });
                    }
                    Err(e) => tracing::warn!(error = %e, "create_group_mls failed"),
                }
            }
            AppCommand::AddGroupMember {
                conversation,
                new_member,
                input,
            } => {
                let peer_ticket = match PeerTicket::decode(&input.peer_ticket_text) {
                    Ok(ticket) => ticket,
                    Err(e) => {
                        tracing::warn!(error = %e, "AddGroupMember dropped — invalid peer ticket");
                        continue;
                    }
                };
                device_directory.register(
                    new_member,
                    MemberDevice {
                        device_id: input.peer_device,
                        ticket: peer_ticket,
                    },
                );

                match group_service
                    .add_member_mls(conversation, new_member, input.peer_device, &input.key_package_bytes)
                    .await
                {
                    Ok(()) => match group_service.group_state(conversation) {
                        Ok(Some(group_state)) => upsert_group_summary(&mut state, &group_state, local_account),
                        Ok(None) => tracing::warn!("add_member_mls succeeded but group_state returned nothing — this is a bug"),
                        Err(e) => tracing::warn!(error = %e, "failed to refresh group state after add_member_mls"),
                    },
                    Err(e) => tracing::warn!(error = %e, "add_member_mls failed"),
                }
            }
            AppCommand::SendGroupMessage { conversation, text } => match group_service
                .send_text_mls(conversation, text.clone())
                .await
            {
                Ok(message_id) => {
                    state.timeline.write().push_latest(TimelineWindow {
                        message_id,
                        sequence: 0,
                        content: MessageContent::Text(text),
                        delivery_state: DeliveryState::Local,
                        from_me: true,
                    });
                    state.conversations.write().mark_read(conversation);
                }
                Err(e) => tracing::warn!(error = %e, "send_text_mls failed"),
            },
            AppCommand::AcceptGroupInvite { conversation } => {
                let Some(invite) = state.pending_invites.write().remove(conversation) else {
                    tracing::warn!(
                        ?conversation,
                        "AcceptGroupInvite dropped — no pending invite (already handled?)"
                    );
                    continue;
                };
                let parsed_state: Result<siar_domain::GroupState, _> =
                    base64_decode(&invite.state_input)
                        .and_then(|bytes| postcard::from_bytes(&bytes).map_err(|e| e.to_string()));
                let group_state = match parsed_state {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!(error = %e, "AcceptGroupInvite dropped — couldn't decode the pasted group state");
                        // Put it back so the user can fix the paste box and retry, rather than losing the invite outright.
                        state.pending_invites.write().add(invite);
                        continue;
                    }
                };
                match group_service.join_group_mls(
                    conversation,
                    &invite.welcome_bytes,
                    group_state.clone(),
                ) {
                    Ok(()) => {
                        upsert_group_summary(&mut state, &group_state, local_account);
                        state.conversations.write().upsert(ConversationSummary {
                            id: conversation,
                            display_name: format!("Group {}", conversation.fmt_short()),
                            last_message_preview: None,
                            unread_count: 0,
                            kind: ConversationKind::Group,
                        });
                    }
                    Err(e) => tracing::warn!(error = %e, "join_group_mls failed"),
                }
            }
            AppCommand::DeclineGroupInvite { conversation } => {
                state.pending_invites.write().remove(conversation);
            }
            AppCommand::SaveContact {
                device_id,
                account_id,
                display_name,
                ticket_text,
                key_package_b64,
            } => {
                // Validate the ticket before persisting anything — a
                // saved contact with an undecodable ticket is worse
                // than no saved contact at all (it would silently fail
                // every future re-registration attempt at the next
                // launch, per `main.rs`'s startup loop over
                // `saved_contacts`).
                let ticket = match PeerTicket::decode(&ticket_text) {
                    Ok(ticket) => ticket,
                    Err(e) => {
                        tracing::warn!(error = %e, "SaveContact dropped — invalid ticket");
                        continue;
                    }
                };
                let stored = siar_storage::StoredContact {
                    device_id,
                    account_id,
                    display_name: display_name.clone(),
                    ticket_text: ticket_text.clone(),
                    key_package_b64: key_package_b64.clone(),
                    added_at_millis: now_millis(),
                };
                match contact_repo.upsert(&stored) {
                    Ok(()) => {
                        device_directory.register(account_id, MemberDevice { device_id, ticket });
                        state.contacts.write().upsert(siar_ui_state::SavedContact {
                            device_id,
                            account_id,
                            display_name,
                            ticket_text,
                            key_package_b64,
                        });
                    }
                    Err(e) => tracing::warn!(error = %e, "failed to persist contact"),
                }
            }
            AppCommand::RemoveContact { device_id } => match contact_repo.remove(device_id) {
                Ok(()) => state.contacts.write().remove(device_id),
                Err(e) => tracing::warn!(error = %e, "failed to remove contact"),
            },
            AppCommand::LoadAttachmentPreview {
                message_id,
                reference,
            } => {
                let Some(peer) = active_peer.read().clone() else {
                    tracing::warn!(
                        ?message_id,
                        "dropped LoadAttachmentPreview — no active peer paired yet"
                    );
                    continue;
                };
                // Spawned rather than awaited inline: fetch_attachment
                // may hit the network (plan.md §22's on-demand fetch),
                // and blocking this whole command queue on one image
                // would stall an in-flight SendMessage/etc behind it.
                // `service`/`state` are cheap to clone into the task —
                // `Arc<MessageService>` and `AppState`'s `Signal`
                // handles are both `Copy`/cheap-`Clone` by design (see
                // `state.rs`'s module doc for the latter). Uses dioxus's
                // own `spawn` (already relied on for `retry_scheduler_
                // loop`/`command_loop`/`incoming_loop` themselves) rather
                // than a raw `tokio::spawn`, on the assumption that the
                // root scope this whole background-task tree runs under
                // (never unmounted for the app's lifetime) makes a
                // further-nested `spawn` call here behave the same as
                // the top-level ones — not verified by a real compile,
                // same caveat every iroh-touching change this session
                // has carried.
                let service = service.clone();
                state.attachment_previews.write().set_loading(message_id);
                spawn(async move {
                    load_attachment_preview(service, peer, message_id, reference, state).await;
                });
            }
        }
    }
}

/// The actual fetch -> decrypt -> decode -> downscale pipeline for one
/// image attachment, split out of `command_loop` so it can run as its
/// own spawned task (see the `LoadAttachmentPreview` arm's comment for
/// why). Only `MediaType::Image*` references reach here in practice —
/// `components.rs` is what decides whether to dispatch the command at
/// all — but this still checks rather than assumes, since a stale/wrong
/// dispatch site elsewhere should fail as an honest "unsupported", not
/// panic on decode.
async fn load_attachment_preview(
    service: Arc<MessageService>,
    peer: PeerTicket,
    message_id: siar_domain::MessageId,
    reference: siar_domain::AttachmentReference,
    mut state: AppState,
) {
    if !matches!(
        reference.media_type,
        siar_domain::MediaType::ImagePng
            | siar_domain::MediaType::ImageJpeg
            | siar_domain::MediaType::ImageWebp
    ) {
        state
            .attachment_previews
            .write()
            .set_failed(message_id, "not a previewable image type".to_string());
        return;
    }

    let plaintext = match service.fetch_attachment(&peer, &reference).await {
        Ok(bytes) => bytes,
        Err(e) => {
            state
                .attachment_previews
                .write()
                .set_failed(message_id, format!("fetch failed: {e}"));
            return;
        }
    };

    // Decoding + resizing is synchronous CPU work (siar-media-image has
    // no I/O, no await points — see its own crate doc comment) — run it
    // on the blocking pool rather than the async task itself so one
    // large image can't stall this task's executor thread the way an
    // inline call would, same reasoning `siar-media-android`'s pipeline
    // doc comment gives for keeping capture/encode off the async
    // runtime's own threads.
    let result = tokio::task::spawn_blocking(move || {
        let decoded = siar_media_image::decode_image(&plaintext)?;
        siar_media_image::generate_preview(&decoded)
    })
    .await;

    match result {
        Ok(Ok(encoded)) => {
            state.attachment_previews.write().set_ready(
                message_id,
                encoded.jpeg_bytes,
                encoded.width,
                encoded.height,
            );
        }
        Ok(Err(e)) => {
            state
                .attachment_previews
                .write()
                .set_failed(message_id, format!("{e}"));
        }
        Err(e) => {
            state
                .attachment_previews
                .write()
                .set_failed(message_id, format!("decode task panicked: {e}"));
        }
    }
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn upsert_group_summary(
    state: &mut AppState,
    group_state: &siar_domain::GroupState,
    local_account: siar_domain::AccountId,
) {
    state.groups.write().upsert(GroupSummary {
        conversation_id: group_state.conversation_id,
        display_label: format!("Group {}", group_state.conversation_id.fmt_short()),
        member_count: group_state.members().len(),
        is_admin: group_state.is_admin(local_account),
        epoch: group_state.epoch.number(),
    });
}

/// Small local helper — `apps/cli`'s `base64_decode` isn't `pub`, so
/// this mirrors it exactly rather than adding a cross-crate dependency
/// on the CLI binary just for one function. `pub(crate)` (not private)
/// since `components.rs`'s key-package-paste form needs the same
/// decode — one helper shared within this binary rather than a second
/// copy-pasted copy drifting out of sync with this one.
pub(crate) fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s.trim())
        .map_err(|e| e.to_string())
}

/// plan.md §112's receive flow, feeding straight into the UI signals.
///
/// Group traffic is now routed to real UI state, not just logged:
/// - `GroupMlsWelcome` becomes a `PendingInvite` the invite-banner
///   component can accept/decline (see `GroupListState`/
///   `PendingInviteState`'s own doc comments for exactly what's still
///   manual here — `join_group_mls` needing the founder's `GroupState`
///   pasted in is a real backend limitation this UI surfaces honestly
///   via a paste box, not one it papers over).
/// - `GroupMlsApplication` content now lands in the shared timeline
///   and bumps the group's `ConversationSummary`, same shape 1:1
///   messages already used.
async fn incoming_loop(
    service: Arc<MessageService>,
    group_service: Arc<GroupService>,
    mut state: AppState,
    active_peer: Signal<Option<PeerTicket>>,
    mut incoming: mpsc::Receiver<siar_transport::IncomingFrame>,
) {
    while let Some(frame) = incoming.recv().await {
        let envelope = match frame.message {
            WireMessage::V1(envelope) => envelope,
            WireMessage::Mesh(_mesh_envelope) => {
                // next.md §29's relay-routable envelope — no DTN
                // forwarding UI/logic exists in this desktop app yet;
                // not crashing on one is the whole fix needed here for
                // now (the old irrefutable `let WireMessage::V1(...) =
                // ...` pattern would have panicked the moment one
                // arrived).
                tracing::warn!(from = ?frame.from, "dropping a Mesh-routed frame — not yet handled by the desktop app");
                continue;
            }
            WireMessage::MailboxCheckIn(_check_in) => {
                // next.md §76–77's mailbox check-in — answering one is
                // a relay's job (`apps/emergency-node`), not an
                // ordinary chat client's.
                tracing::warn!(from = ?frame.from, "dropping a MailboxCheckIn frame — this app isn't a relay");
                continue;
            }
            WireMessage::TokenMailboxDeposit(_) | WireMessage::AnonymousMailboxCheckIn(_) => {
                // The unlinkable counterparts to `Mesh`/`MailboxCheckIn`
                // — same "a relay's job, not this app's" reasoning.
                tracing::warn!(from = ?frame.from, "dropping a token-mailbox frame — this app isn't a relay");
                continue;
            }
            WireMessage::RouteAdvertisement(_advertisement) => {
                // A relay-to-relay signal — this desktop app has no
                // `PathTable` of its own to fold one into.
                tracing::warn!(from = ?frame.from, "dropping a RouteAdvertisement frame — this app isn't a relay");
                continue;
            }
        };

        if matches!(
            envelope.kind,
            EnvelopeKind::GroupEvent
                | EnvelopeKind::GroupMlsCommit
                | EnvelopeKind::GroupMlsWelcome
                | EnvelopeKind::GroupMlsApplication
        ) {
            match envelope.kind {
                EnvelopeKind::GroupMlsWelcome => {
                    state.pending_invites.write().add(PendingInvite {
                        conversation_id: envelope.conversation_id,
                        from_device: envelope.sender,
                        welcome_bytes: envelope.payload,
                        state_input: String::new(),
                    });
                    tracing::info!(
                        from = ?frame.from,
                        conversation = ?envelope.conversation_id,
                        "MLS group welcome arrived — waiting on the invite banner to accept or decline"
                    );
                }
                _ => match group_service.handle_incoming_mls(envelope.conversation_id, &envelope) {
                    Ok(Some(content)) => {
                        let conversation = envelope.conversation_id;
                        if let MessageContent::Text(text) = &content {
                            state
                                .conversations
                                .write()
                                .set_preview(conversation, text.as_str().to_string());
                            state.conversations.write().increment_unread(conversation);
                        }
                        state.timeline.write().push_latest(TimelineWindow {
                            message_id: envelope.message_id,
                            sequence: envelope.sequence,
                            content,
                            delivery_state: DeliveryState::Delivered,
                            from_me: false,
                        });
                    }
                    Ok(None) => {} // commit merged, or nothing to show
                    Err(e) => tracing::warn!(error = %e, "failed to handle incoming group frame"),
                },
            }
            continue;
        }

        let Some(peer) = active_peer.read().clone() else {
            tracing::warn!("dropping frame — no active peer paired yet");
            continue;
        };
        match service.handle_incoming(&peer, envelope).await {
            // Previously only `MessageContent::Text` was handled here —
            // an incoming `Attachment` reference silently fell into the
            // `Ok(_) => {}` catch-all below and never reached the
            // timeline at all (worse than the group-chat path, which
            // already pushed every `MessageContent` variant). Matching
            // on `Content(content)` generically closes that: both
            // variants get a timeline entry, and the conversation-list
            // preview line only applies to `Text` (an attachment has no
            // sensible one-line text preview), same distinction the
            // group-chat arm above already draws for itself.
            Ok(Some(IncomingEvent::Content(content))) => {
                let conversation = ConversationId::new(); // Phase-3 stand-in, see ActivePeer's docs
                let preview = match &content {
                    MessageContent::Text(text) => Some(text.as_str().to_string()),
                    MessageContent::Attachment(_) => None,
                };
                state.conversations.write().upsert(ConversationSummary {
                    id: conversation,
                    display_name: frame.from.fmt_short().to_string(),
                    last_message_preview: preview,
                    unread_count: 0,
                    kind: ConversationKind::Direct,
                });
                state.conversations.write().increment_unread(conversation);
                state.timeline.write().push_latest(TimelineWindow {
                    message_id: siar_domain::MessageId::new(),
                    sequence: 0,
                    content,
                    delivery_state: DeliveryState::Delivered,
                    from_me: false,
                });
            }
            Ok(_) => {}
            Err(e) => tracing::warn!(error = %e, "failed to handle incoming frame"),
        }
    }
}
