#![forbid(unsafe_code)]

//! siar-ui-state: the view-model layer between Dioxus and `siar-messaging`
//! (plan.md §52–55).
//!
//! Deliberately has zero Dioxus dependency. Everything here is plain
//! Rust structs/enums that a Dioxus signal can wrap — which means this
//! whole crate is unit-testable without a UI toolkit or a system webview,
//! unlike `apps/desktop` (which needs both and can't run headless in any
//! sandbox, not just this one).
//!
//! Rule (plan.md §86): the dependency direction is
//! `Dioxus components -> ViewModels (this crate) -> AppCommand/AppEvent
//! -> MessagingCore`. This crate depends on `siar-domain` only — never on
//! `siar-messaging`, `siar-storage`, or `siar-transport` directly, so it
//! stays reusable if a future frontend replaces Dioxus.

mod attachment_preview;
mod command;
mod composer;
mod contact_list;
mod conversation_list;
mod group_list;
mod network;
mod security_center;
mod security_event;
mod timeline;

pub use attachment_preview::{AttachmentPreview, AttachmentPreviewState};
pub use command::{AppCommand, AppEvent};
pub use composer::ComposerState;
pub use contact_list::{ContactListState, SavedContact};
pub use conversation_list::{ConversationKind, ConversationListState, ConversationSummary};
pub use group_list::{
    AddMemberInput, GroupListState, GroupSummary, PendingInvite, PendingInviteState,
};
pub use network::{NetworkState, NetworkStatus, PeerReachability};
pub use security_center::{
    DeviceKind, DeviceListState, DeviceSecurityFlag, DeviceSecurityView, DeviceTrustState,
    SecurityHealth, SecurityOverview,
};
pub use security_event::{
    SecurityEvent, SecurityEventKind, SecurityEventSeverity, SecurityEventState,
};
pub use timeline::{TimelineState, TimelineWindow};
