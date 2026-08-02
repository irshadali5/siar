//! The right-hand chat pane: header with peer name/relay hint, scrolling
//! message list (text bubbles + file bubbles), and a composer with a
//! paperclip button for sending files. WhatsApp-style: own messages
//! right-aligned filled, peer messages left-aligned neutral, delivery
//! ticks on own messages.

use crate::sidebar::Avatar;
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub enum BubbleKind {
    Own { acked: bool },
    Peer,
    System,
}

/// Where an incoming file stands relative to actually being on disk. A
/// sent file (our own) is always conceptually "Done" at the original path
/// it was picked from, since there's nothing to fetch.
#[derive(Clone, PartialEq)]
pub enum FileState {
    /// Announced, not yet fetched — shows a Download button.
    Idle,
    Downloading,
    /// Saved locally at this path.
    Done(String),
    Failed(String),
}

#[derive(Clone, PartialEq)]
pub enum BubbleContent {
    Text(String),
    File {
        /// Stable id used to route `on_download_file`/`on_open_file` back
        /// to the right bubble — the blob's hex BLAKE3 hash, which is
        /// already unique per file content.
        hash: String,
        name: String,
        size_bytes: u64,
        state: FileState,
        /// Whether a download control makes sense at all for this bubble
        /// — `false` for our own sent files (nothing to fetch) and for
        /// history-reconstructed room files where we no longer know who
        /// to fetch from (see `ui::mod`'s history-reload comment).
        fetchable: bool,
    },
}

#[derive(Clone, PartialEq)]
pub struct BubbleData {
    pub kind: BubbleKind,
    pub sender: String,
    pub content: BubbleContent,
    pub time_label: String,
    /// The wire envelope id, when known — `None` for system bubbles and
    /// for history rows written before this column existed (see
    /// `Store::open`'s migration comment). Reaction/edit/delete actions
    /// are only offered in the UI when this is `Some`.
    pub id: Option<u64>,
    /// `(sender_id hex, emoji)` per reactor.
    pub reactions: Vec<(String, String)>,
    pub edited: bool,
    pub deleted: bool,
    /// Only meaningful on `Own` bubbles: has the peer's `read_watermarks`
    /// entry for this conversation caught up to this message's
    /// timestamp? Computed in `bubbles_for`, not stored on
    /// `StoredBubble` — it's a comparison against a single per-
    /// conversation watermark, not per-message state.
    pub read: bool,
    /// `Some((sender_label, snippet))` when this message replies to
    /// another — resolved locally by `bubbles_for` (see that fn's doc),
    /// so `sender_label` is empty and `snippet` reads as an
    /// unavailable-placeholder when the original isn't in this
    /// conversation's currently-loaded history. `None` for an ordinary,
    /// non-reply message.
    pub reply_preview: Option<(String, String)>,
}

#[component]
pub fn ChatPane(
    title: String,
    subtitle: String,
    messages: Vec<BubbleData>,
    compose: Signal<String>,
    on_send: EventHandler<()>,
    on_attach_file: EventHandler<()>,
    on_back: EventHandler<()>,
    on_typing: EventHandler<()>,
    on_download_file: EventHandler<String>,
    on_open_info: EventHandler<()>,
    /// Add/remove an emoji reaction: `(target envelope id, emoji)`. Also
    /// used to *remove* your own existing reaction — the caller (parent)
    /// decides add-vs-remove by checking whether you've already reacted
    /// with that emoji, this component just reports "you tapped this
    /// emoji on this message".
    on_react: EventHandler<(u64, String)>,
    /// Starts editing: `(target envelope id, current text)` — the parent
    /// pre-fills the composer and remembers which message is being
    /// edited; this component has no edit-mode state of its own.
    on_edit_start: EventHandler<(u64, String)>,
    on_delete: EventHandler<u64>,
    /// Starts a reply: `(target envelope id, sender label, snippet)` —
    /// same shape `bubbles_for` already resolved for `reply_preview`, so
    /// there's no second lookup needed here; this just hands it back to
    /// the parent to stash in `replying_to`.
    on_reply_start: EventHandler<(u64, String, String)>,
    /// Right-click on a bubble: `(x, y, target_id, is_own, deleted,
    /// sender_label, snippet)` — same shape `ContextMenuKind::Bubble`
    /// wants, captured here since this is where all of it is already at
    /// hand per-bubble, so the parent (`lib.rs`) doesn't need a second
    /// lookup to populate the menu.
    on_bubble_context_menu: EventHandler<(f64, f64, u64, bool, bool, String, String)>,
    /// `Some((sender_label, snippet))` while the composer has a reply
    /// queued — rendered as a dismissable bar above the composer.
    #[props(default = None)]
    pending_reply: Option<(String, String)>,
    on_cancel_reply: EventHandler<()>,
    /// `None` for room conversations (no 1:1 voice calling there); `Some`
    /// for DMs, shows a call button in the header.
    on_call: Option<EventHandler<()>>,
    /// Same idea as `on_call`, but for a video call (AV1 — see
    /// `net::calls::video`). A separate handler/button rather than a
    /// toggle on the voice one, since which kind of call to place has to
    /// be decided before dialing (it's part of the invite), not switched
    /// mid-ring.
    #[props(default = None)]
    on_video_call: Option<EventHandler<()>>,
    /// `None` for rooms (no single picture) or a DM contact with no
    /// avatar set/fetched yet. See `ui::sidebar::Avatar`.
    #[props(default = None)]
    avatar_hash: Option<String>,
    images: Signal<HashMap<String, String>>,
) -> Element {
    rsx! {
        div { class: "chat-pane",
            div { class: "chat-header",
                button { class: "icon back-button", onclick: move |_| on_back.call(()), "←" }
                Avatar { hash: avatar_hash, label: title.clone(), images }
                div { style: "flex: 1;",
                    div { style: "font-weight:600;", "{title}" }
                    div { style: "font-size:12px; color:var(--text-muted);", "{subtitle}" }
                }
                if let Some(on_call) = on_call {
                    button {
                        class: "icon",
                        title: "Voice call",
                        onclick: move |_| on_call.call(()),
                        "📞"
                    }
                }
                if let Some(on_video_call) = on_video_call {
                    button {
                        class: "icon",
                        title: "Video call",
                        onclick: move |_| on_video_call.call(()),
                        "🎥"
                    }
                }
                button {
                    class: "icon",
                    title: "Conversation info",
                    onclick: move |_| on_open_info.call(()),
                    "ⓘ"
                }
            }
            div { class: "chat-messages",
                for msg in messages.iter() {
                    {
                        let row_class = match msg.kind {
                            BubbleKind::Own { .. } => "bubble-row own",
                            BubbleKind::Peer => "bubble-row",
                            BubbleKind::System => "bubble-row",
                        };
                        let bubble_class = match msg.kind {
                            BubbleKind::Own { .. } => "bubble own",
                            BubbleKind::Peer => "bubble peer",
                            BubbleKind::System => "bubble system",
                        };
                        rsx! {
                            div { class: "{row_class}",
                                {
                                    // Precomputed here, as owned values, rather than
                                    // captured inside `oncontextmenu` directly — `msg`
                                    // is `&BubbleData`, borrowed from `messages.iter()`,
                                    // and Dioxus event handlers must be `'static`; a
                                    // `move` closure that captures `msg` itself (rather
                                    // than values cloned out of it beforehand) doesn't
                                    // outlive this render call, which is exactly the
                                    // "temporary value dropped while borrowed" this
                                    // produced.
                                    let ctx_bubble_id = msg.id;
                                    let ctx_is_own = matches!(msg.kind, BubbleKind::Own { .. });
                                    let ctx_is_system = matches!(msg.kind, BubbleKind::System);
                                    let ctx_deleted = msg.deleted;
                                    let ctx_sender_label = if ctx_is_own { "You".to_string() } else { msg.sender.clone() };
                                    let ctx_snippet = match &msg.content {
                                        BubbleContent::Text(t) => t.chars().take(60).collect::<String>(),
                                        BubbleContent::File { name, .. } => format!("📎 {name}"),
                                    };
                                    rsx! {
                                        div {
                                            class: "{bubble_class}",
                                            oncontextmenu: move |e| {
                                                e.prevent_default();
                                                let Some(target_id) = ctx_bubble_id else { return };
                                                if ctx_is_system {
                                                    return;
                                                }
                                                let coords = e.data.client_coordinates();
                                                on_bubble_context_menu.call((
                                                    coords.x, coords.y, target_id, ctx_is_own,
                                                    ctx_deleted, ctx_sender_label.clone(), ctx_snippet.clone(),
                                                ));
                                            },
                                            if !matches!(msg.kind, BubbleKind::Own { .. } | BubbleKind::System) {
                                                div { class: "bubble-sender", "{msg.sender}" }
                                    }
                                    if let Some((reply_sender, reply_snippet)) = &msg.reply_preview {
                                        div { class: "bubble-reply-quote",
                                            if !reply_sender.is_empty() {
                                                div { class: "bubble-reply-quote-sender", "{reply_sender}" }
                                            }
                                            div { class: "bubble-reply-quote-text", "{reply_snippet}" }
                                        }
                                    }
                                    match &msg.content {
                                        BubbleContent::Text(t) => rsx! {
                                            div {
                                                style: if msg.deleted { "color: var(--text-muted); font-style: italic;" } else { "" },
                                                "{t}"
                                            }
                                        },
                                        BubbleContent::File { hash, name, size_bytes, state, fetchable } => rsx! {
                                            div { class: "file-bubble",
                                                div { class: "file-icon", "📎" }
                                                div {
                                                    div { class: "file-name", "{name}" }
                                                    div { class: "file-size", "{format_size(*size_bytes)}" }
                                                }
                                                match state {
                                                    FileState::Idle if *fetchable => {
                                                        let hash = hash.clone();
                                                        rsx! {
                                                            button {
                                                                class: "secondary",
                                                                onclick: move |_| on_download_file.call(hash.clone()),
                                                                "Download"
                                                            }
                                                        }
                                                    }
                                                    FileState::Idle => rsx! {
                                                        span { style: "color: var(--text-muted); font-size:12px;", "sent" }
                                                    },
                                                    FileState::Downloading => rsx! {
                                                        span { style: "color: var(--text-muted); font-size:12px;", "downloading…" }
                                                    },
                                                    FileState::Done(path) => rsx! {
                                                        span { style: "color: var(--accent-strong); font-size:12px;", title: "{path}", "✓ saved" }
                                                    },
                                                    FileState::Failed(err) => {
                                                        let hash = hash.clone();
                                                        rsx! {
                                                            button {
                                                                class: "secondary",
                                                                title: "{err}",
                                                                onclick: move |_| on_download_file.call(hash.clone()),
                                                                "Retry"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                    }
                                    // Reactions/edit/delete only make sense for a
                                    // message we can actually address on the wire
                                    // (a real envelope id) and that hasn't already
                                    // been tombstoned — see `BubbleData::id`'s doc.
                                    if let (Some(target_id), false) = (msg.id, msg.deleted) {
                                        if !msg.reactions.is_empty() {
                                            div { class: "bubble-reactions",
                                                {
                                                    let mut counts: Vec<(String, usize)> = Vec::new();
                                                    for (_, emoji) in &msg.reactions {
                                                        if let Some(entry) = counts.iter_mut().find(|(e, _)| e == emoji) {
                                                            entry.1 += 1;
                                                        } else {
                                                            counts.push((emoji.clone(), 1));
                                                        }
                                                    }
                                                    rsx! {
                                                        for (emoji, count) in counts {
                                                            span {
                                                                class: "reaction-badge",
                                                                onclick: move |_| on_react.call((target_id, emoji.clone())),
                                                                "{emoji} {count}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "bubble-actions",
                                            for emoji in ["👍", "❤️", "😂", "😮"] {
                                                span {
                                                    class: "bubble-action-emoji",
                                                    onclick: move |_| on_react.call((target_id, emoji.to_string())),
                                                    "{emoji}"
                                                }
                                            }
                                            {
                                                let reply_sender = if matches!(msg.kind, BubbleKind::Own { .. }) {
                                                    "You".to_string()
                                                } else {
                                                    msg.sender.clone()
                                                };
                                                let snippet = match &msg.content {
                                                    BubbleContent::Text(t) => t.chars().take(60).collect::<String>(),
                                                    BubbleContent::File { name, .. } => format!("📎 {name}"),
                                                };
                                                rsx! {
                                                    span {
                                                        class: "bubble-action-text",
                                                        onclick: move |_| on_reply_start.call((target_id, reply_sender.clone(), snippet.clone())),
                                                        "Reply"
                                                    }
                                                }
                                            }
                                            if let BubbleKind::Own { .. } = msg.kind {
                                                if let BubbleContent::Text(t) = &msg.content {
                                                    {
                                                        let t = t.clone();
                                                        rsx! {
                                                            span {
                                                                class: "bubble-action-text",
                                                                onclick: move |_| on_edit_start.call((target_id, t.clone())),
                                                                "Edit"
                                                            }
                                                        }
                                                    }
                                                }
                                                span {
                                                    class: "bubble-action-text",
                                                    onclick: move |_| on_delete.call(target_id),
                                                    "Delete"
                                                }
                                            }
                                        }
                                    }
                                    div { class: "bubble-meta",
                                        "{msg.time_label}"
                                        if msg.edited {
                                            span { style: "color: var(--text-muted); font-style: italic;", " (edited)" }
                                        }
                                        if let BubbleKind::Own { acked } = msg.kind {
                                            {
                                                let glyph = if acked { "✓✓" } else { "✓" };
                                                let style = if msg.read {
                                                    "color: var(--accent-strong); font-weight: 700;"
                                                } else if acked {
                                                    "color: var(--accent-strong);"
                                                } else {
                                                    ""
                                                };
                                                rsx! { span { style: "{style}", " {glyph}" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            }
            }
            if let Some((reply_sender, reply_snippet)) = &pending_reply {
                div { class: "composer-reply-bar",
                    div { style: "flex:1; min-width:0;",
                        div { class: "bubble-reply-quote-sender", "Replying to {reply_sender}" }
                        div { class: "bubble-reply-quote-text", "{reply_snippet}" }
                    }
                    button { class: "icon", onclick: move |_| on_cancel_reply.call(()), "✕" }
                }
            }
            div { class: "composer",
                button { class: "icon", onclick: move |_| on_attach_file.call(()), "📎" }
                textarea {
                    class: "composer-input",
                    placeholder: "Type a message  ·  Shift+Enter for a new line",
                    rows: "1",
                    value: "{compose}",
                    oninput: move |e| {
                        compose.set(e.value());
                        on_typing.call(());
                    },
                    onkeydown: move |e| {
                        // Enter sends; Shift+Enter (or Ctrl/Cmd+Enter, for anyone
                        // used to that convention instead) inserts a newline like
                        // every other messaging app — plain `Key::Enter` used to
                        // send unconditionally, which made multi-line messages
                        // impossible to compose at all.
                        if e.key() == Key::Enter && !e.modifiers().shift() {
                            e.prevent_default();
                            on_send.call(());
                        }
                    },
                }
                button { onclick: move |_| on_send.call(()), "Send" }
            }
        }
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{size:.1} {}", UNITS[unit])
}
