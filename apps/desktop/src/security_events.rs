//! Security Events Screen (ui-ux-15 §40-47).
//!
//! ⚠️ NOT compiled or tested — same caveat as every other
//! `apps/desktop` file in this series. `siar-ui-state`'s
//! `SecurityEventState`/`SecurityEventKind`/`SecurityEventSeverity`
//! *are* real and compile+test-verified; only this rendering code is
//! unverified. Build locally (`cargo build -p siar-desktop`) and paste
//! back errors.
//!
//! **Rewritten this round**: the previous version of this file matched
//! an earlier, narrower `SecurityEventKind` (3 variants,
//! `Notice`/`StrongWarning` severity) that has since been widened to
//! match this spec's own §41/§42 literally (11 variants,
//! `Info`/`Warning`/`Critical`). This version matches the current
//! `siar-ui-state` API — the old variant names (`NewDeviceLinked`,
//! `RootIdentityChanged`) no longer exist, so the previous version of
//! this file would no longer compile even setting aside the
//! iroh/rustc-version issue.

#![allow(dead_code)]

use crate::state::AppState;
use dioxus::prelude::*;
use siar_ui_state::{SecurityEvent, SecurityEventFilter, SecurityEventKind, SecurityEventSeverity};

/// §47: interrupts on unresolved `Critical` events specifically —
/// narrower than "every unresolved event," matching
/// `SecurityEventState::unresolved_critical_events`'s own reasoning.
#[component]
pub fn CriticalSecurityWarningBanner() -> Element {
    let mut state = use_context::<AppState>();
    let critical: Vec<SecurityEvent> = state
        .security_events
        .read()
        .unresolved_critical_events()
        .into_iter()
        .cloned()
        .collect();

    if critical.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "critical-security-warning-overlay",
            for event in critical {
                div { class: "critical-security-warning-banner",
                    key: "{event.id:?}",
                    span { class: "warning-icon", "⚠" }
                    p { class: "warning-text", "{event_headline(event.kind)}" }
                    button {
                        class: "acknowledge-button",
                        onclick: move |_| state.security_events.write().resolve(event.id),
                        "I understand"
                    }
                }
            }
        }
    }
}

/// §43's filterable list screen. `filter` is owned by whatever parent
/// component tracks the active tab — kept as a plain prop rather than
/// this component owning tab-selection state itself, matching
/// `components.rs`'s established pattern of leaf components rendering
/// exactly the slice they're given.
#[component]
pub fn SecurityEventsScreen(filter: SecurityEventFilter) -> Element {
    let mut state = use_context::<AppState>();
    let events: Vec<SecurityEvent> = state
        .security_events
        .read()
        .filtered(filter)
        .into_iter()
        .cloned()
        .collect();

    rsx! {
        ul { class: "security-event-list",
            for event in events {
                // §44: title / time / severity / resolved-unresolved /
                // related device or contact.
                li {
                    key: "{event.id:?}",
                    class: "security-event-row severity-{severity_class(event.severity)}",
                    span { class: "event-title", "{event_headline(event.kind)}" }
                    span { class: "event-severity-badge", "{severity_label(event.severity)}" }
                    if event.resolved {
                        span { class: "event-resolved-badge", "Resolved" }
                    } else {
                        button {
                            class: "resolve-button",
                            onclick: move |_| state.security_events.write().resolve(event.id),
                            "Mark resolved"
                        }
                    }
                }
            }
        }
    }
}

/// §44's "title" field. §46: "avoid raw crypto errors" — every arm here
/// is plain, non-technical language, never a propagated error message
/// or a raw type/field name.
fn event_headline(kind: SecurityEventKind) -> &'static str {
    match kind {
        SecurityEventKind::DeviceLinked => "A new device was linked to your account.",
        SecurityEventKind::DeviceRevoked => "A device was removed from your account.",
        SecurityEventKind::DeviceLinkDenied => "A device linking attempt was denied.",
        SecurityEventKind::IdentityChanged => {
            "A contact's identity has changed. Re-verify before continuing."
        }
        SecurityEventKind::VerificationFailed => "Identity verification failed.",
        SecurityEventKind::RecoveryConfigured => "Account recovery was set up.",
        SecurityEventKind::RecoveryChanged => "Your account recovery settings changed.",
        SecurityEventKind::BackupFailed => "A backup attempt failed.",
        SecurityEventKind::KeyRotation => "Your security keys were rotated.",
        SecurityEventKind::SuspiciousAuthorization => {
            "A suspicious authorization attempt was blocked."
        }
        SecurityEventKind::SecurityPolicyChanged => "Your organization's security policy changed.",
        SecurityEventKind::KeyExpiryActionRequired => "A security key requires renewal action.",
    }
}

fn severity_label(severity: SecurityEventSeverity) -> &'static str {
    match severity {
        SecurityEventSeverity::Info => "Info",
        SecurityEventSeverity::Warning => "Warning",
        SecurityEventSeverity::Critical => "Critical",
    }
}

fn severity_class(severity: SecurityEventSeverity) -> &'static str {
    match severity {
        SecurityEventSeverity::Info => "info",
        SecurityEventSeverity::Warning => "warning",
        SecurityEventSeverity::Critical => "critical",
    }
}
