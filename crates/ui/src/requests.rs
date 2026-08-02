//! Requests inbox — the "someone wants to connect" list. Shown when the
//! sidebar's Requests tab is active. Each incoming request must be
//! explicitly Accepted before any DM ALPN session can open with that peer
//! (see `net::contacts`).

use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct RequestEntry {
    pub endpoint_id: String,
    pub display_name: String,
    pub username: Option<String>,
    pub note: String,
    pub requested_label: String,
}

#[component]
pub fn RequestsInbox(
    requests: Vec<RequestEntry>,
    on_accept: EventHandler<String>,
    on_decline: EventHandler<String>,
    on_back: EventHandler<()>,
) -> Element {
    rsx! {
        section { class: "requests-pane sidebar-list",
            div { class: "requests-mobile-header",
                button { class: "icon back-button", onclick: move |_| on_back.call(()), "←" }
                div {
                    strong { "Requests" }
                    span { "People who want to connect" }
                }
            }
            if requests.is_empty() {
                div { class: "empty-state requests-empty",
                    div { class: "empty-state-icon", "✓" }
                    strong { "You're all caught up" }
                    span { "New contact requests will appear here." }
                }
            }
            for req in requests.iter() {
                {
                    let id_accept = req.endpoint_id.clone();
                    let id_decline = req.endpoint_id.clone();
                    rsx! {
                        div { class: "request-card", key: "{req.endpoint_id}",
                            div { style: "display:flex; align-items:center; gap:10px;",
                                div { class: "avatar", "{req.display_name.chars().next().unwrap_or('?')}" }
                                div {
                                    div { style: "font-weight:600;",
                                        "{req.display_name}"
                                        if let Some(u) = &req.username {
                                            span { style: "color: var(--text-muted); font-weight:400;", " @{u}" }
                                        }
                                        span { style: "color: var(--text-muted); font-weight:400; font-size:12px;", "  ·  {req.requested_label}" }
                                    }
                                    if !req.note.is_empty() {
                                        div { style: "color: var(--text-muted); font-size: 13px;", "“{req.note}”" }
                                    }
                                }
                            }
                            div { class: "request-actions",
                                button { onclick: move |_| on_accept.call(id_accept.clone()), "Accept" }
                                button { class: "secondary", onclick: move |_| on_decline.call(id_decline.clone()), "Decline" }
                            }
                        }
                    }
                }
            }
        }
    }
}
