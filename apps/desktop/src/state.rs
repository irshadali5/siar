//! `AppState` (plan.md §53): one `Signal` per state slice, not one big
//! struct — so a component reading only `composer` doesn't re-render
//! when `timeline` changes (plan.md §94).
//!
//! `Signal<T>` handles are always `Copy` regardless of `T`, which is why
//! `AppState` itself can derive `Copy` even though it holds an
//! `mpsc::Sender` — that's the mechanism Dioxus context providers rely on
//! to let every component get its own cheap handle to the same shared
//! state.

use dioxus::prelude::*;
use siar_ui_state::{
    AppCommand, AttachmentPreviewState, ComposerState, ContactListState, ConversationListState, GroupListState,
    NetworkStatus, PendingInviteState, TimelineState,
};
use tokio::sync::mpsc;

#[derive(Clone, Copy)]
pub struct AppState {
    pub conversations: Signal<ConversationListState>,
    pub timeline: Signal<TimelineState>,
    pub composer: Signal<ComposerState>,
    pub network: Signal<NetworkStatus>,
    /// Group-UI state slices (plan.md §94's granular-signal rule
    /// applied here too): a component rendering only the group list
    /// doesn't re-render on every keystroke in the composer, and a
    /// component rendering only the invite banner doesn't re-render on
    /// every new group message.
    pub groups: Signal<GroupListState>,
    pub pending_invites: Signal<PendingInviteState>,
    /// The persistent contact book (see `main.rs`'s `resolve_data_paths`
    /// doc comment for what "persistent" fixed) — its own slice for the
    /// same granular-re-render reason as `groups`/`pending_invites`.
    pub contacts: Signal<ContactListState>,
    /// One `AttachmentPreview` per message that's ever asked to load an
    /// image (see that type's doc comment) — its own slice for the same
    /// granular-re-render reason as `groups`/`pending_invites`/`contacts`.
    pub attachment_previews: Signal<AttachmentPreviewState>,
    pub command_tx: Signal<Option<mpsc::Sender<AppCommand>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            conversations: Signal::new(ConversationListState::new()),
            timeline: Signal::new(TimelineState::new()),
            composer: Signal::new(ComposerState::new()),
            network: Signal::new(NetworkStatus::new()),
            groups: Signal::new(GroupListState::new()),
            pending_invites: Signal::new(PendingInviteState::new()),
            contacts: Signal::new(ContactListState::new()),
            attachment_previews: Signal::new(AttachmentPreviewState::new()),
            command_tx: Signal::new(None),
        }
    }

    /// Sends a command to the background messaging task (see
    /// `main.rs`'s module docs) if it's finished starting up; silently
    /// drops the command otherwise, which only happens in the brief
    /// window before `bootstrap_messaging` completes — a component
    /// disabling its send button while `command_tx` is `None` is the
    /// real fix, this is just the safety net under that.
    pub fn dispatch(&self, command: AppCommand) {
        if let Some(tx) = self.command_tx.read().as_ref() {
            let tx = tx.clone();
            spawn(async move {
                let _ = tx.send(command).await;
            });
        } else {
            tracing::warn!("dropped command sent before messaging bootstrap finished");
        }
    }
}
