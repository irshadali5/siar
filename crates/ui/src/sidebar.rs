//! Left-hand sidebar: a WhatsApp/Signal-style chat list (avatar, name,
//! last-message preview, timestamp, unread badge), a search box that
//! doubles as "find someone new" (searches the local registry view; if the
//! text matches nobody yet, it's offered as a ticket to paste instead), and
//! a "Requests" tab showing the incoming-request inbox with a badge count.

use crate::{ChatListEntry, ConvKey};
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub enum SidebarTab {
    Chats,
    Requests,
}

#[component]
pub fn Sidebar(
    tab: Signal<SidebarTab>,
    entries: Vec<ChatListEntry>,
    active: Option<ConvKey>,
    pending_request_count: usize,
    search_query: Signal<String>,
    search_results: Vec<(String, bool)>, // (username, already a contact)
    room_input: Signal<String>,
    /// Whether the list currently showing is the archived-only view (see
    /// `ui::mod`'s `ui.show_archived`) rather than the normal chat list.
    showing_archived: bool,
    /// How many DMs are archived right now, regardless of which view is
    /// showing — used for the footer toggle's badge.
    archived_count: usize,
    /// True while a request/ticket connect is in flight — disables the
    /// Request/Connect buttons and swaps their label, so an impatient
    /// extra click (reasonable given this can take tens of seconds)
    /// doesn't fire an independent duplicate attempt. See `ui.connecting`.
    connecting: bool,
    /// Cache of decoded avatar images, keyed by hash — see `ui::mod`'s
    /// `avatar_images` and the `Avatar` component below.
    images: Signal<HashMap<String, String>>,
    on_select: EventHandler<ConvKey>,
    on_search_input: EventHandler<String>,
    on_send_request: EventHandler<String>,
    on_connect_ticket: EventHandler<String>,
    /// Decode and validate one camera/gallery image, then connect using
    /// the ticket carried by its QR code. The parent owns decoding so the
    /// same validation/toast path is shared with pasted tickets.
    on_scan_qr_image: EventHandler<Vec<u8>>,
    on_scan_qr_error: EventHandler<String>,
    /// Fired when someone clicks a search result that's already a
    /// contact — takes the username, since that's all a registry search
    /// result carries (see `spawn_search`); resolving it to the actual
    /// `EndpointId` and opening the conversation happens on the caller
    /// side, which has `ui.contacts` to look it up in.
    on_open_existing_contact: EventHandler<String>,
    on_join_room: EventHandler<String>,
    on_toggle_archived_view: EventHandler<()>,
    /// Right-click on a chat row: `(x, y, key, pinned, archived)` — same
    /// shape `ContextMenuKind::ChatRow` wants.
    on_row_context_menu: EventHandler<(f64, f64, ConvKey, bool, bool)>,
) -> Element {
    let mut show_qr_actions = use_signal(|| false);
    rsx! {
        div { class: "sidebar",
            div { class: "sidebar-search",
                div { class: "sidebar-search-row",
                    input {
                        id: "sidebar-search-input",
                        placeholder: "Search username or paste a ticket…",
                        value: "{search_query}",
                        oninput: move |e| {
                            search_query.set(e.value().clone());
                            on_search_input.call(e.value());
                        },
                    }
                    button {
                        class: "qr-scan-trigger secondary",
                        r#type: "button",
                        title: "Scan connection QR code",
                        aria_label: "Scan connection QR code",
                        onclick: move |_| show_qr_actions.toggle(),
                        "⌗"
                    }
                }
                if show_qr_actions() {
                    div { class: "qr-scan-actions", role: "group", aria_label: "Scan connection QR code",
                        div { class: "qr-scan-copy",
                            strong { "Connect from QR" }
                            span { "Take a clear photo or choose an existing image." }
                        }
                        label { class: "qr-source-button", r#for: "qr-camera-input",
                            span { aria_hidden: "true", "📷" }
                            "Camera"
                        }
                        input {
                            id: "qr-camera-input",
                            class: "visually-hidden-file",
                            r#type: "file",
                            accept: "image/*",
                            capture: "environment",
                            onchange: move |event| {
                                let Some(file) = event.files().into_iter().next() else { return };
                                if file.size() > 20 * 1024 * 1024 {
                                    on_scan_qr_error.call("QR image is too large (20 MB maximum)".to_string());
                                    return;
                                }
                                show_qr_actions.set(false);
                                spawn(async move {
                                    match file.read_bytes().await {
                                        Ok(bytes) => on_scan_qr_image.call(bytes.to_vec()),
                                        Err(error) => on_scan_qr_error.call(format!("couldn't read camera image: {error}")),
                                    }
                                });
                            },
                        }
                        label { class: "qr-source-button secondary", r#for: "qr-gallery-input",
                            span { aria_hidden: "true", "▧" }
                            "Gallery"
                        }
                        input {
                            id: "qr-gallery-input",
                            class: "visually-hidden-file",
                            r#type: "file",
                            accept: "image/png,image/jpeg,image/gif,image/webp,image/*",
                            onchange: move |event| {
                                let Some(file) = event.files().into_iter().next() else { return };
                                if file.size() > 20 * 1024 * 1024 {
                                    on_scan_qr_error.call("QR image is too large (20 MB maximum)".to_string());
                                    return;
                                }
                                show_qr_actions.set(false);
                                spawn(async move {
                                    match file.read_bytes().await {
                                        Ok(bytes) => on_scan_qr_image.call(bytes.to_vec()),
                                        Err(error) => on_scan_qr_error.call(format!("couldn't read gallery image: {error}")),
                                    }
                                });
                            },
                        }
                        button {
                            class: "qr-source-close secondary",
                            r#type: "button",
                            aria_label: "Close QR options",
                            onclick: move |_| show_qr_actions.set(false),
                            "×"
                        }
                    }
                }
            }
            div { class: "sidebar-tabs",
                div {
                    class: if tab() == SidebarTab::Chats { "sidebar-tab active" } else { "sidebar-tab" },
                    onclick: move |_| tab.set(SidebarTab::Chats),
                    "Chats"
                }
                div {
                    class: if tab() == SidebarTab::Requests { "sidebar-tab active" } else { "sidebar-tab" },
                    onclick: move |_| tab.set(SidebarTab::Requests),
                    "Requests"
                    if pending_request_count > 0 {
                        span { class: "requests-badge", "{pending_request_count}" }
                    }
                }
            }

            if !search_query.read().is_empty() {
                {
                    let q = search_query.read().trim().to_string();
                    let looks_like_ticket = q.starts_with("mtkt1");
                    rsx! {
                        div { class: "sidebar-list",
                            if looks_like_ticket {
                                // A ticket never matches a username search (different
                                // format entirely) — surface the ticket action
                                // immediately instead of making the person wait for
                                // an empty registry result first.
                                div { style: "padding: 14px;",
                                    div { style: "color: var(--text-muted); font-size: 13px; margin-bottom:8px;",
                                        "That looks like a connection ticket."
                                    }
                                    button {
                                        style: "width:100%;",
                                        disabled: connecting,
                                        onclick: {
                                            let q = q.clone();
                                            move |_| on_connect_ticket.call(q.clone())
                                        },
                                        if connecting { "Connecting…" } else { "Connect via ticket" }
                                    }
                                }
                            } else {
                                for (name, is_contact) in search_results.iter() {
                                    div {
                                        class: "chat-row",
                                        key: "{name}",
                                        style: if *is_contact { "cursor:pointer;" },
                                        onclick: {
                                            let name = name.clone();
                                            let is_contact = *is_contact;
                                            move |_| {
                                                if is_contact {
                                                    on_open_existing_contact.call(name.clone());
                                                }
                                            }
                                        },
                                        div { class: "avatar", "{first_char(name)}" }
                                        div { class: "chat-row-body",
                                            div { class: "chat-row-name", "{name}" }
                                        }
                                        if *is_contact {
                                            span { style: "color: var(--text-muted); font-size:12px;", "already a contact" }
                                        } else {
                                            button {
                                                disabled: connecting,
                                                onclick: {
                                                    let name = name.clone();
                                                    move |evt| {
                                                        // The row itself has no click behavior
                                                        // for a non-contact (only the button
                                                        // does), but stop propagation anyway in
                                                        // case a future row-level onclick is
                                                        // added here later — the button's own
                                                        // action should always win over the
                                                        // row's.
                                                        evt.stop_propagation();
                                                        on_send_request.call(name.clone());
                                                    }
                                                },
                                                if connecting { "…" } else { "Request" }
                                            }
                                        }
                                    }
                                }
                                if search_results.is_empty() {
                                    div { style: "padding: 14px; color: var(--text-muted); font-size: 13px;",
                                        p { style: "margin:0 0 8px 0;",
                                            "No username match yet. Usernames spread across the network as people connect — "
                                            "a freshly claimed one can take a little while to reach you, especially on first launch."
                                        }
                                        p { style: "margin:0 0 8px 0;",
                                            "Have a ticket instead? Paste it here — tickets connect instantly, no waiting."
                                        }
                                        button {
                                            class: "secondary",
                                            style: "width:100%;",
                                            disabled: connecting,
                                            onclick: {
                                                let q = q.clone();
                                                move |_| on_connect_ticket.call(q.clone())
                                            },
                                            if connecting { "Connecting…" } else { "Connect via ticket" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                if showing_archived {
                    div {
                        style: "padding:8px 14px; font-size:12px; color:var(--text-muted); \
                                display:flex; justify-content:space-between; align-items:center;",
                        span { "Archived chats" }
                        button { class: "secondary", onclick: move |_| on_toggle_archived_view.call(()), "Back" }
                    }
                }
                div { class: "sidebar-list",
                    for entry in entries.iter() {
                        {
                            let key = entry.key.clone();
                            let ctx_key = entry.key.clone();
                            let is_active = active.as_ref() == Some(&entry.key);
                            let (pinned, archived) = (entry.pinned, entry.archived);
                            rsx! {
                                div {
                                    class: if is_active { "chat-row active" } else { "chat-row" },
                                    key: "{entry.id}",
                                    onclick: move |_| on_select.call(key.clone()),
                                    oncontextmenu: move |e| {
                                        e.prevent_default();
                                        let coords = e.data.client_coordinates();
                                        on_row_context_menu.call((coords.x, coords.y, ctx_key.clone(), pinned, archived));
                                    },
                                    div { style: "position: relative;",
                                        Avatar { hash: entry.avatar_hash.clone(), label: entry.name.clone(), images }
                                        if entry.online {
                                            span {
                                                style: "position:absolute; bottom:0; right:0; width:11px; height:11px; \
                                                        border-radius:50%; background: var(--accent-strong); \
                                                        border: 2px solid var(--sidebar-bg);",
                                            }
                                        }
                                    }
                                    div { class: "chat-row-body",
                                        div { class: "chat-row-top",
                                            span { class: "chat-row-name",
                                                if entry.pinned { "📌 " }
                                                if entry.verified { "✓ " }
                                                "{entry.name}"
                                            }
                                            span { class: "chat-row-time", "{entry.time_label}" }
                                        }
                                        div { class: "chat-row-preview", "{entry.preview}" }
                                    }
                                    if entry.unread > 0 {
                                        span { class: "unread-badge", "{entry.unread}" }
                                    }
                                }
                            }
                        }
                    }
                    if entries.is_empty() {
                        div { style: "padding: 14px; color: var(--text-muted); font-size: 13px;",
                            if showing_archived { "No archived chats." } else { "No chats yet." }
                        }
                    }
                }
                if !showing_archived {
                    div { style: "display:flex; gap:6px; padding:10px; border-top: 1px solid var(--border);",
                        input {
                            placeholder: "Room name (new) or paste an invite ticket…",
                            value: "{room_input}",
                            oninput: move |e| room_input.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    on_join_room.call(room_input.cloned());
                                }
                            },
                        }
                        button { onclick: move |_| on_join_room.call(room_input.cloned()), "Join" }
                    }
                    if archived_count > 0 {
                        div {
                            style: "padding:10px 14px; font-size:12px; color:var(--text-muted); \
                                    cursor:pointer; border-top: 1px solid var(--border);",
                            onclick: move |_| on_toggle_archived_view.call(()),
                            "Archived ({archived_count})"
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn first_char(s: &str) -> String {
    s.chars().next().map(|c| c.to_string()).unwrap_or_default()
}

/// One avatar, wherever it's shown — sidebar rows, chat header, contact
/// info panel, request list. `hash` is looked up in `images` (this
/// component's caller passes `ui.avatar_images`, populated by
/// `load_avatar_into_cache` in `ui::mod`); anything not found there — no
/// avatar ever set, or set but not fetched/decoded yet — falls back to
/// the same letter-circle placeholder this app always rendered, so
/// there's no "broken image" state to handle.
#[component]
pub(crate) fn Avatar(
    hash: Option<String>,
    label: String,
    images: Signal<HashMap<String, String>>,
    /// Pixel diameter override for contexts that aren't the default
    /// `.avatar` CSS size (e.g. the titlebar's smaller one) — `None` uses
    /// whatever `.avatar` already specifies.
    #[props(default = None)]
    size_px: Option<u32>,
) -> Element {
    let cached = hash.as_ref().and_then(|h| images.read().get(h).cloned());
    let size_style = size_px
        .map(|px| {
            format!(
                "width:{px}px; height:{px}px; font-size:{}px;",
                px * 50 / 100
            )
        })
        .unwrap_or_default();
    rsx! {
        match cached {
            Some(data_uri) => rsx! {
                img {
                    class: "avatar",
                    style: "object-fit: cover; {size_style}",
                    src: "{data_uri}",
                    alt: "{label}",
                }
            },
            None => rsx! { div { class: "avatar", style: "{size_style}", "{first_char(&label)}" } },
        }
    }
}
