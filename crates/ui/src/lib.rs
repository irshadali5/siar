//! Root Dioxus component. Same structural rule as v1: every network
//! operation (connect, send, join, claim, request/accept) runs on its own
//! `tokio::spawn`ed task using cheap cloned handles, reporting results
//! through a channel drained by a single pump task — the render loop never
//! awaits a network call directly.
//!
//! v2 adds a screen-level branch: if no identity exists yet on disk, show
//! `onboarding::Onboarding`; once an identity (and username) exists, boot
//! `app::App` and show the main three-pane shell (sidebar / chat / requests).

#![deny(unsafe_code)]

mod chat;
mod css;
mod icon_b64;
pub mod onboarding;
mod requests;
mod sidebar;

use chat::{BubbleContent, BubbleData, BubbleKind, ChatPane, FileState};
use dioxus::document;
use dioxus::prelude::*;
use iroh::EndpointId;
use iroh_docs::protocol::Docs;
use iroh_docs::AuthorId;
use onboarding::{Onboarding, OnboardingResult};
use requests::{RequestEntry, RequestsInbox};
use siar_core::app::{self, App as Core};
use siar_core::gossip::room::RoomEvent;
use siar_core::net::contacts::ContactEvent;
use siar_core::net::conv_docs::{DmDoc, DmSettings, MemberRecord, RoomDoc, RoomMeta};
use siar_core::protocol::dm::DmEvent;
use siar_core::protocol::message::{Body, Envelope};
use siar_core::store::{
    CallDirection, CallLogEntry, CallOutcome, Contact, ContactState, Conversation, StatusEntry,
    ThemeMode, ThemeStyle,
};
use siar_core::CONFIG;
use sidebar::{first_char, Avatar, Sidebar, SidebarTab};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// How long a peer's "typing…" indicator stays shown after their last
/// `Body::Typing` signal, if no follow-up arrives (they stopped typing,
/// sent the message, or went offline mid-keystroke).
const TYPING_TIMEOUT: Duration = Duration::from_secs(4);
/// Minimum gap between outgoing `Body::Typing` signals for the same
/// conversation — keystroke-per-packet would be wasteful and gives the
/// peer's UI nothing extra over a coarser signal.
const TYPING_RESEND_INTERVAL_MS: i64 = 3000;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ConvKey {
    Dm(EndpointId),
    Room(String),
}

/// A right-click (or long-press, on touch) context menu's position and
/// what it's for — see `context_menu`'s doc on `UiState` and the
/// `ContextMenu` component that renders this.
#[derive(Clone, PartialEq)]
struct ContextMenuState {
    x: f64,
    y: f64,
    kind: ContextMenuKind,
}

#[derive(Clone, PartialEq)]
enum ContextMenuKind {
    /// Right-clicked a message bubble. Carries everything the menu
    /// needs to render the right item set and act on a click without a
    /// second lookup — same shape `bubbles_for`/`BubbleData` already
    /// compute per-bubble, just captured at the moment of the click.
    Bubble {
        target_id: u64,
        is_own: bool,
        deleted: bool,
        sender_label: String,
        snippet: String,
    },
    /// Right-clicked a sidebar chat row. Pin/archive only apply to DMs
    /// in this app currently (see `ConvInfoPanel`'s `on_toggle_pinned`/
    /// `on_toggle_archived`, both gated on `ConvKey::Dm`) — a `Room` row
    /// still opens this with `pinned`/`archived` both `false` and simply
    /// won't show those two items (checked at render time in
    /// `ContextMenu`, not by having two separate variants here).
    ChatRow {
        key: ConvKey,
        pinned: bool,
        archived: bool,
    },
}

#[derive(Clone, PartialEq)]
pub struct ChatListEntry {
    pub id: String,
    pub key: ConvKey,
    pub name: String,
    pub preview: String,
    pub time_label: String,
    pub unread: u32,
    pub online: bool,
    /// From `net::conv_docs::DmSettings` (`ui.dm_settings_cache`) — always
    /// `false` for rooms, which don't have a pin/archive concept in this
    /// design. See `build_chat_list`'s sort/filter and `ui::sidebar`'s
    /// "Archived" toggle.
    pub pinned: bool,
    pub archived: bool,
    /// From `store::Contact::verified` — manually set, see
    /// `ConvInfoPanel`'s "Mark verified" toggle. Always `false` for rooms.
    pub verified: bool,
    /// This contact's current display picture, if we have one on record
    /// (`store::Contact::avatar_hash`) — always `None` for rooms, which
    /// don't have a single picture. `None` here doesn't distinguish
    /// "never sent one" from "sent one but we haven't fetched the bytes
    /// yet"; `ui::sidebar::Avatar` handles both the same way (falls back
    /// to the letter circle) since the distinction doesn't change what
    /// gets rendered.
    pub avatar_hash: Option<String>,
}

/// Superset of `chat::BubbleContent` for what's actually kept in memory:
/// carries the extra fields needed to *fetch* a file (who to download it
/// from, whether it's zstd-compressed) that the display-only
/// `chat::BubbleContent` doesn't need. `bubbles_for` projects this down to
/// the display type for each render.
#[derive(Clone, PartialEq)]
enum StoredContent {
    Text(String),
    File {
        hash: String,
        name: String,
        size_bytes: u64,
        state: FileState,
        compressed: bool,
        /// Who to fetch this from, if we still know — `None` for our own
        /// sent files (nothing to fetch) and for room files reconstructed
        /// from history, where the original sender isn't tracked as a
        /// parsed `EndpointId` in sqlite (see `store::StoredMessage`).
        from: Option<EndpointId>,
    },
}

#[derive(Clone, PartialEq)]
struct StoredBubble {
    /// The envelope id this bubble corresponds to. `Some` for outgoing
    /// messages (so a later `Body::Ack(id)` can flip `acked` on the right
    /// bubble) and incoming ones (so we know what to ack); `None` only for
    /// bubbles reconstructed from sqlite history, where the original
    /// envelope id isn't persisted and isn't needed — history is already
    /// delivered by definition.
    id: Option<u64>,
    kind: BubbleKind,
    sender: String,
    content: StoredContent,
    sent_unix_ms: i64,
    /// Disappearing messages — see `store::StoredMessage::expires_at_unix_ms`.
    /// Filtered at render time (`bubbles_for`) as defense in depth on top
    /// of the sqlite read-path filter, and physically pruned from this
    /// in-memory cache by `spawn_disappearing_sweep` alongside the sqlite
    /// delete, so a long-lived session doesn't keep an expired bubble
    /// visible just because it was loaded before it expired.
    expires_at_unix_ms: Option<i64>,
    /// `(sender_id hex, emoji)` per reactor — mutated in place by
    /// `apply_reaction_locally` when a `Body::Reaction` arrives, so this
    /// starts empty for any newly-appended bubble.
    reactions: Vec<(String, String)>,
    /// `true` once a `Body::Edit` targeting this bubble has been
    /// applied — shown as a small "edited" marker; `content` itself
    /// already holds the *new* text by the time this is `true` (see
    /// `apply_edit_locally`).
    edited: bool,
    /// `true` once a `Body::Delete` targeting this bubble has been
    /// applied — `content` is replaced with a placeholder at that point
    /// (see `apply_delete_locally`), this flag is what render code
    /// checks to style it as a tombstone rather than real content.
    deleted: bool,
    /// `Envelope::id` of the message this one replies to, if any — see
    /// `store::StoredMessage::reply_to_envelope_id`, which this mirrors
    /// for the in-memory (not-yet-persisted-view) cache the same way
    /// every other field here does.
    reply_to_envelope_id: Option<u64>,
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Boot,
    Onboarding,
    Main,
}

/// All UI-facing reactive state, bundled so it's cheap (`Signal` is
/// `Copy`) to pass into every helper and event-handler closure.
#[derive(Clone, Copy, PartialEq)]
struct UiState {
    screen: Signal<Screen>,
    core: Signal<Option<Core>>,
    active: Signal<Option<ConvKey>>,
    conversations: Signal<HashMap<ConvKey, Vec<StoredBubble>>>,
    unread: Signal<HashMap<ConvKey, u32>>,
    contacts: Signal<Vec<siar_core::store::Contact>>,
    /// Cache of decoded avatar images, keyed by BLAKE3 hash, as ready-to-
    /// render `data:image/png;base64,...` URIs — populated by
    /// `load_avatar_into_cache` (called both right after
    /// `App::set_my_avatar` and after a background contact-avatar
    /// prefetch completes). `ui::sidebar::Avatar` reads this directly: a
    /// hash present here renders the real picture, anything else (not
    /// fetched yet, or no avatar at all) falls back to the letter circle.
    /// Encoding to a data URI up front, rather than caching raw bytes and
    /// encoding at render time, means `Avatar` itself stays a plain
    /// synchronous read with no work beyond a hashmap lookup.
    avatar_images: Signal<HashMap<String, String>>,
    /// Our own current avatar hash, if we've set one — mirrors
    /// `settings.my_avatar_hash` (loaded at boot) and is updated
    /// immediately by whichever code path calls `App::set_my_avatar`.
    /// Kept separate from `avatar_images`'s keys-are-hashes lookup since
    /// there's no `ChatListEntry`/`Contact` for "ourselves" to hang this
    /// off of.
    my_avatar_hash: Signal<Option<String>>,
    pending_requests: Signal<Vec<siar_core::store::Contact>>,
    rooms: Signal<Vec<String>>,
    compose: Signal<String>,
    sidebar_tab: Signal<SidebarTab>,
    search_query: Signal<String>,
    search_results: Signal<Vec<(String, bool)>>,
    username_available: Signal<Option<bool>>,
    toasts: Signal<Vec<(u64, String, bool)>>, // (id, text, is_error)
    relay_ok: Signal<bool>,
    /// Peers we currently have an open DM connection with — see
    /// `handle_app_event`'s `PeerConnected`/`PeerDisconnected` arms. This
    /// is "reachable right now over an active session", not full network
    /// presence (someone can be online without us having dialed them yet).
    online: Signal<HashSet<EndpointId>>,
    /// Conversation -> unix-ms timestamp of the last `Body::Typing` we
    /// received for it. Cleared automatically after `TYPING_TIMEOUT` by a
    /// delayed task spawned when the signal arrives.
    typing: Signal<HashMap<ConvKey, i64>>,
    /// Conversation -> unix-ms timestamp we last *sent* a typing signal,
    /// so `on_compose_input` doesn't fire one on every keystroke.
    last_typing_sent: Signal<HashMap<ConvKey, i64>>,
    show_profile: Signal<bool>,
    room_input: Signal<String>,
    /// Conversation-info drawer (`ConvInfoPanel`) — room title/membership
    /// or DM shared settings, backed by `net::conv_docs`. `None`/`None`
    /// while closed or still loading; only one of the two is ever `Some`
    /// at a time, matched on the currently-active `ConvKey`.
    show_conv_info: Signal<bool>,
    room_info: Signal<Option<(RoomMeta, Vec<MemberRecord>)>>,
    dm_info: Signal<Option<DmSettings>>,
    conv_info_input: Signal<String>,
    /// Set when `spawn_boot` fails (identity load/create, or `Core::start`
    /// itself). Kept separate from the transient `toasts` stack — a toast
    /// alone left the user staring at an indefinite "Connecting…" with no
    /// way to tell a slow relay handshake apart from a boot that already
    /// failed, since the toast auto-dismisses. See `AppRoot`'s Main-screen
    /// branch and `spawn_boot`'s doc comment.
    boot_error: Signal<Option<String>>,
    /// Per-DM `net::conv_docs::DmSettings`, kept for every accepted
    /// contact (not just the one currently open in `ConvInfoPanel`) so
    /// `build_chat_list` can sort pinned chats to the top and filter
    /// archived ones out of the main list. Populated by
    /// `spawn_preload_dm_settings` (on boot and after accepting a new
    /// contact) and refreshed incrementally by `spawn_open_conv_info`
    /// whenever the user actually opens/edits a DM's info panel.
    dm_settings_cache: Signal<HashMap<EndpointId, DmSettings>>,
    /// Toggles the sidebar between the normal chat list (pinned first,
    /// archived hidden) and an archived-only view — the only way back to
    /// an archived chat's "Unarchive" button. See `ui::sidebar::Sidebar`'s
    /// footer toggle.
    show_archived: Signal<bool>,
    /// Disk usage breakdown for the Settings panel's Storage tab —
    /// deliberately computed on demand (see `spawn_load_storage_stats`)
    /// rather than on every render, since it's a filesystem walk.
    storage_stats: Signal<Option<StorageStats>>,
    storage_loading: Signal<bool>,
    /// True while a contact request or ticket connect is in flight. Both
    /// `spawn_send_request` and `spawn_connect_ticket` can take tens of
    /// seconds (multi-attempt retry against a slow/absent discovery
    /// result) — this stops a second click during that window from firing
    /// an independent duplicate attempt, and lets the sidebar show
    /// "Connecting…" instead of looking unresponsive.
    connecting: Signal<bool>,
    /// Someone is calling us — (their id, their display name). `None`
    /// means no ring showing.
    incoming_call: Signal<Option<(EndpointId, String)>>,
    /// The actual accept/decline channel for whatever `incoming_call`
    /// currently shows. A `Signal` (not a plain field) purely so this
    /// stays consistent with every other piece of shared `UiState` —
    /// `oneshot::Sender` isn't `Clone`, but that's fine, `Signal<T>` is a
    /// `Copy` handle into Dioxus's own storage regardless of whether `T`
    /// is; nothing here ever calls `.cloned()` on it, only `.write()`
    /// to take it out once, exactly like `App::active_call_hangup`.
    incoming_call_decision: Signal<Option<tokio::sync::oneshot::Sender<bool>>>,
    /// A call is actually connected — (peer id, peer display name). Drives
    /// the in-call bar.
    active_call: Signal<Option<(EndpointId, String)>>,
    /// We just placed a call and are waiting for them to pick up — (peer
    /// id, peer display name). `None` once it's either connected (flips
    /// over to `active_call`) or ended/declined/failed before pickup.
    /// This is what was missing entirely before: the callee always saw a
    /// ring banner (`incoming_call`), but the caller saw *nothing* at all
    /// during the ringing window — no indication a call was even in
    /// progress. See `spawn_call_peer`.
    outgoing_call: Signal<Option<(EndpointId, String)>>,
    /// Holds the ring/ringback sound while `incoming_call` or
    /// `outgoing_call` is `Some`. Not read anywhere for its value — it's
    /// pure lifetime management: playback stops the instant this is set
    /// back to `None` (see `ringtone::Ringtone`'s `Drop`), so every call
    /// state transition that clears `incoming_call`/`outgoing_call` also
    /// clears this in the same breath.
    active_ringtone: Signal<Option<siar_core::ringtone::Ringtone>>,
    /// DM peer → the timestamp they've told us (via `Body::Read`) they've
    /// read up to. Compared against a sent bubble's `sent_unix_ms` at
    /// render time to show "read" vs "delivered" — see `Body::Read`'s doc
    /// for why this is DM-only.
    read_watermarks: Signal<HashMap<ConvKey, i64>>,
    /// `Some(target_id)` while the composer is being used to edit an
    /// existing message rather than compose a new one — set by
    /// `on_edit_start`, checked and cleared by `send_to_active`.
    editing: Signal<Option<u64>>,
    /// `Some((target_id, sender_label, snippet))` while the composer has
    /// a reply target queued — set by the context menu's "Reply" action
    /// (`on_reply_start`), read and cleared by `send_to_active`, and
    /// rendered as a dismissable quote-preview bar above the composer
    /// (see `chat::ChatPane`'s `reply_preview` prop). Mutually exclusive
    /// with `editing` in practice (the UI doesn't offer both actions on
    /// the same message at once) but not enforced as a single enum here
    /// — simpler to keep the two independent than to invent a combined
    /// "composer mode" type for one shared field.
    replying_to: Signal<Option<(u64, String, String)>>,
    /// Loaded from `Store::theme_mode` once boot completes (see
    /// `spawn_boot`); `System` until then, which is also the correct
    /// default for a brand-new install. Setting it (Settings' Appearance
    /// tab) both persists via `Store::set_theme_mode` and updates this
    /// signal directly — see `on_theme_change` — so the change is
    /// immediate, not waiting on a re-boot.
    theme_mode: Signal<ThemeMode>,
    /// Loaded from `Store::theme_style` alongside `theme_mode` in the
    /// same `spawn_boot` step; `Regular` until then and for a brand-new
    /// install. See `store::ThemeStyle`'s doc for how this and
    /// `theme_mode` combine (or rather, deliberately don't) at the CSS
    /// level.
    theme_style: Signal<ThemeStyle>,
    /// Local, transient state for the Storage tab's "Back up now" flow
    /// (`backup::create_backup`) — never persisted, cleared implicitly
    /// just by navigating away (these are plain `use_signal`s scoped to
    /// the running session, not written to `Store`).
    backup_seed_input: Signal<String>,
    backup_passphrase_input: Signal<String>,
    backup_busy: Signal<bool>,
    backup_error: Signal<Option<String>>,
    /// The one currently-open context menu (right-click on a message
    /// bubble or a sidebar chat row), if any — `None` closes it. Global
    /// rather than per-bubble/per-row so opening a second one implicitly
    /// closes the first, and so there's exactly one place (`ContextMenu`
    /// in `lib.rs`) that owns "click anywhere else to dismiss".
    context_menu: Signal<Option<ContextMenuState>>,
    /// Whether the call currently being placed/answered should include
    /// video — decided at the moment the call button is clicked (see
    /// `spawn_call_peer`'s `with_video` argument), not toggleable mid-call.
    active_call_has_video: Signal<bool>,
    /// Latest decoded frame from the peer's camera, as a
    /// `data:image/jpeg;base64,...` URI — same caching convention as
    /// `avatar_images`. `None` whenever no video is currently flowing
    /// (audio-only call, video capture/decoder not yet warmed up, or no
    /// call at all).
    remote_video_frame: Signal<Option<String>>,
    /// Our own camera preview, same encoding as `remote_video_frame` — so
    /// the local person can see they're actually on camera (and confirm
    /// framing) the same way every other video-calling app shows a
    /// self-view tile.
    local_video_frame: Signal<Option<String>>,
    /// Text state for `ConvInfoPanel`'s custom disappearing-messages hours
    /// field.
    custom_ttl_hours: Signal<String>,
    /// Currently-live status updates (own + contacts'), for the Status
    /// bottom-nav section. Refreshed by `refresh_statuses`.
    statuses: Signal<Vec<StatusEntry>>,
    /// Text state for the "post a status" composer.
    status_compose: Signal<String>,
    /// Hours state for the status composer's expiry picker — same 24h
    /// default / up-to-168h ceiling as disappearing messages.
    status_ttl_hours: Signal<String>,
    /// Raw bytes of an image picked for the status composer, if any —
    /// codec-normalized (see `media::decode_status_image`) only once
    /// actually posted, not at pick time, so switching images before
    /// posting doesn't do wasted decode/encode work.
    status_image: Signal<Option<Vec<u8>>>,
    /// Which contact's status (index into the currently-displayed
    /// "theirs" list) the full-screen story viewer is showing, if open.
    story_viewer: Signal<Option<usize>>,
    /// Whether the full-screen viewer is showing your *own* status
    /// rather than a contact's — separate from `story_viewer`'s index
    /// since "my own status" isn't a position in the `theirs` list that
    /// indexes into.
    own_story_viewer: Signal<bool>,
    /// Frames recorded for the status composer but not yet posted — see
    /// `spawn_record_status_video`. Cleared on post, same lifecycle as
    /// `status_image`.
    status_video_pending: Signal<Option<Vec<image::RgbImage>>>,
    /// Whether a status video recording is currently in progress —
    /// drives the composer's "Recording…" state.
    status_recording: Signal<bool>,
    /// Encoded Opus clip recorded for the status composer but not yet
    /// posted — see `spawn_record_status_audio`. Cleared on post, same
    /// lifecycle as `status_image`/`status_video_pending`. Unlike video,
    /// this is already fully encoded (`record_and_encode_voice_clip`
    /// does record+encode in one blocking call), not raw frames.
    status_audio_pending: Signal<Option<Vec<u8>>>,
    /// Whether a status voice clip recording is currently in progress.
    status_recording_audio: Signal<bool>,
    /// Call history, most recent first, for the Calls bottom-nav section.
    call_log: Signal<Vec<CallLogEntry>>,
    /// When the currently-active call actually connected — needed to
    /// compute a duration to log once it ends. `None` while no call (or
    /// still ringing) is active.
    active_call_started_ms: Signal<Option<i64>>,
    /// Set the moment we know which direction the current/pending call
    /// is — `Incoming` as soon as `CallEvent::Incoming` fires (before
    /// it's even answered), `Outgoing` the moment we place one. Needed at
    /// logging time since `CallEvent::Connected`/`Ended` don't carry it.
    active_call_direction: Signal<Option<CallDirection>>,
    /// Which of the five bottom-nav sections is showing.
    bottom_nav: Signal<BottomNav>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BottomNav {
    /// Combined feed — both DM peers and group rooms together, most
    /// recently active first. `Dms`/`Groups` below are the same
    /// conversation list, just each pre-filtered to one `ConvKey` variant
    /// for people who'd rather not have the two interleaved.
    Chats,
    Dms,
    Groups,
    Calls,
    Status,
}

#[derive(Clone, Copy, PartialEq, Default)]
struct StorageStats {
    db_bytes: u64,
    blobs_bytes: u64,
    docs_bytes: u64,
}

impl StorageStats {
    fn total(&self) -> u64 {
        self.db_bytes + self.blobs_bytes + self.docs_bytes
    }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

#[component]
pub fn AppRoot() -> Element {
    let ui = UiState {
        screen: use_signal(|| Screen::Boot),
        core: use_signal(|| None),
        active: use_signal(|| None),
        conversations: use_signal(HashMap::new),
        unread: use_signal(HashMap::new),
        contacts: use_signal(Vec::new),
        avatar_images: use_signal(HashMap::new),
        my_avatar_hash: use_signal(|| None),
        pending_requests: use_signal(Vec::new),
        rooms: use_signal(Vec::new),
        compose: use_signal(String::new),
        sidebar_tab: use_signal(|| SidebarTab::Chats),
        search_query: use_signal(String::new),
        search_results: use_signal(Vec::new),
        username_available: use_signal(|| None),
        toasts: use_signal(Vec::new),
        relay_ok: use_signal(|| true),
        online: use_signal(HashSet::new),
        typing: use_signal(HashMap::new),
        last_typing_sent: use_signal(HashMap::new),
        show_profile: use_signal(|| false),
        room_input: use_signal(String::new),
        show_conv_info: use_signal(|| false),
        room_info: use_signal(|| None),
        dm_info: use_signal(|| None),
        conv_info_input: use_signal(String::new),
        boot_error: use_signal(|| None),
        dm_settings_cache: use_signal(HashMap::new),
        show_archived: use_signal(|| false),
        storage_stats: use_signal(|| None),
        storage_loading: use_signal(|| false),
        connecting: use_signal(|| false),
        incoming_call: use_signal(|| None),
        incoming_call_decision: use_signal(|| None),
        active_call: use_signal(|| None),
        outgoing_call: use_signal(|| None),
        active_ringtone: use_signal(|| None),
        read_watermarks: use_signal(HashMap::new),
        editing: use_signal(|| None),
        replying_to: use_signal(|| None),
        theme_mode: use_signal(|| ThemeMode::System),
        theme_style: use_signal(|| ThemeStyle::Regular),
        backup_seed_input: use_signal(String::new),
        backup_passphrase_input: use_signal(String::new),
        backup_busy: use_signal(|| false),
        backup_error: use_signal(|| None),
        context_menu: use_signal(|| None),
        active_call_has_video: use_signal(|| false),
        remote_video_frame: use_signal(|| None),
        local_video_frame: use_signal(|| None),
        custom_ttl_hours: use_signal(String::new),
        statuses: use_signal(Vec::new),
        status_compose: use_signal(String::new),
        status_ttl_hours: use_signal(String::new),
        status_image: use_signal(|| None),
        story_viewer: use_signal(|| None),
        own_story_viewer: use_signal(|| false),
        status_video_pending: use_signal(|| None),
        status_recording: use_signal(|| false),
        status_audio_pending: use_signal(|| None),
        status_recording_audio: use_signal(|| false),
        call_log: use_signal(Vec::new),
        active_call_started_ms: use_signal(|| None),
        active_call_direction: use_signal(|| None),
        bottom_nav: use_signal(|| BottomNav::Chats),
    };

    use_effect(move || {
        let data_dir = CONFIG.get().unwrap().data_dir.clone();
        ui.screen
            .clone()
            .set(if siar_core::identity::exists(&data_dir) {
                Screen::Main
            } else {
                Screen::Onboarding
            });
        if *ui.screen.read() == Screen::Main {
            spawn_boot(ui, None);
        }
    });

    // `System` renders no `data-theme` attribute at all, letting the
    // `@media (prefers-color-scheme: dark)` rule in css/mod.rs decide —
    // that's genuine OS-level live sync, not polling: the webview
    // re-evaluates that media query on its own the moment the OS theme
    // changes, same as any other page. `Light`/`Dark` set the attribute
    // explicitly, which the `[data-theme="..."]` rules in css/mod.rs
    // override the media-query result with (higher specificity).
    //
    // `theme_style` is checked first and, when it's not `Regular`,
    // wins outright — a hacker style is its own fixed palette, not a
    // light/dark variant, so there's nothing to combine here (see
    // `store::ThemeStyle`'s doc). Switching back to `Regular` falls
    // through to `theme_mode` exactly as if hacker styles never
    // existed, which is what makes "switch back" restore whatever
    // light/dark choice was already there instead of losing it.
    let data_theme = match *ui.theme_style.read() {
        ThemeStyle::HackerGreen => Some("hacker-green"),
        ThemeStyle::HackerRed => Some("hacker-red"),
        ThemeStyle::Regular => match *ui.theme_mode.read() {
            ThemeMode::System => None,
            ThemeMode::Light => Some("light"),
            ThemeMode::Dark => Some("dark"),
        },
    };
    rsx! {
        meta { name: "viewport", content: "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=0, viewport-fit=cover" }
        style { {css::stylesheet()} }
        div {
            class: "app-shell",
            "data-theme": data_theme,
            // App-wide keyboard shortcuts. Attached here (rather than inside
            // MainShell) because this is the one element that wraps every
            // screen and never gets replaced — keydown still reaches it via
            // normal bubbling from whichever input/textarea actually has
            // focus, so this fires regardless of what the user was typing
            // in, without needing its own tabindex/focus.
            //
            // Deliberately NOT handling plain single-key shortcuts (a bare
            // "n", "/", etc.) here — those would fire while someone is
            // composing a message or a room name, which no chat app does.
            // Every shortcut below requires Ctrl/Cmd (or is Escape, which
            // is safe unconditionally) for exactly that reason.
            onkeydown: move |e| {
                let mods = e.modifiers();
                let cmd = mods.ctrl() || mods.meta();
                match e.key() {
                    Key::Escape => {
                        // Closes exactly one layer per press, innermost/most-
                        // recently-opened overlay first, same order a user
                        // would naturally back out of them by hand.
                        if ui.editing.read().is_some() {
                            ui.editing.clone().set(None);
                            ui.compose.clone().set(String::new());
                        } else if ui.story_viewer.read().is_some() || *ui.own_story_viewer.read() {
                            ui.story_viewer.clone().set(None);
                            ui.own_story_viewer.clone().set(false);
                        } else if *ui.show_conv_info.read() {
                            ui.show_conv_info.clone().set(false);
                        } else if *ui.show_profile.read() {
                            ui.show_profile.clone().set(false);
                        } else if !ui.search_query.read().is_empty() {
                            ui.search_query.clone().set(String::new());
                            ui.search_results.clone().set(vec![]);
                        }
                    }
                    Key::Character(ref c) if cmd && c.eq_ignore_ascii_case("k") => {
                        // Jump to the sidebar's search/find-someone box —
                        // the closest thing this app has to a quick switcher.
                        e.prevent_default();
                        spawn(async move {
                            // Best-effort: focusing an arbitrary element by id
                            // via document::eval hasn't been exercised against
                            // a real `dx build` yet (see BUILD_NOTES.md) — if
                            // this silently no-ops on some platform, Ctrl+K
                            // still did nothing worse than Ctrl+K in a browser
                            // would.
                            //
                            // Now doubly unverified: `document::eval` runs
                            // arbitrary JS against a webview's DOM, and this
                            // app's render backend is moving to dioxus-native
                            // (Blitz/WGPU, no JS engine backing it the way a
                            // webview has one). Whether `eval` no-ops cleanly,
                            // errors, or does something else under the native
                            // renderer isn't something to guess at without a
                            // build — same reasoning as the title bar/theme
                            // sync/context-menu items called out separately.
                            let _ = document::eval(
                                "document.getElementById('sidebar-search-input')?.focus()",
                            )
                            .await;
                        });
                    }
                    Key::Character(ref c) if cmd && c == "," => {
                        e.prevent_default();
                        ui.show_profile.clone().set(true);
                    }
                    Key::Character(ref c) if cmd && mods.shift() && c.eq_ignore_ascii_case("a") => {
                        e.prevent_default();
                        let cur = *ui.show_archived.read();
                        ui.show_archived.clone().set(!cur);
                    }
                    _ => {}
                }
            },
            TitleBar {}
            match *ui.screen.read() {
                Screen::Boot => rsx! { div { class: "empty-state", "Starting…" } },
                Screen::Onboarding => rsx! {
                    Onboarding {
                        username_available: ui.username_available,
                        on_check_username: move |name: String| check_username(ui, name),
                        on_ready: move |result: OnboardingResult| {
                            ui.screen.clone().set(Screen::Main);
                            spawn_boot(ui, Some(result));
                        },
                    }
                },
                Screen::Main => {
                    if let Some(error) = ui.boot_error.cloned() {
                        rsx! {
                            div { class: "empty-state", style: "flex-direction: column; gap: 12px;",
                                div { "Couldn't start: {error}" }
                                button {
                                    onclick: move |_| {
                                        ui.boot_error.clone().set(None);
                                        spawn_boot(ui, None);
                                    },
                                    "Retry"
                                }
                            }
                        }
                    } else {
                        rsx! { MainShell { ui } }
                    }
                }
            }
            div { class: "toast-stack",
                for (id, text, is_error) in ui.toasts.read().iter() {
                    div { key: "{id}", class: if *is_error { "toast error" } else { "toast" }, "{text}" }
                }
            }
            if let Some((_, name)) = ui.incoming_call.cloned() {
                div {
                    style: "position:fixed; top:16px; left:50%; transform:translateX(-50%); \
                            background:var(--bg-secondary,#1a1a2e); border:1px solid var(--border); \
                            border-radius:10px; padding:12px 16px; display:flex; align-items:center; \
                            gap:12px; z-index:1000;",
                    span { "📞 {name} is calling…" }
                    button { onclick: move |_| spawn_answer_call(ui, true), "Accept" }
                    button { class: "secondary", onclick: move |_| spawn_answer_call(ui, false), "Decline" }
                }
            }
            // The caller's side of ringing — previously nonexistent, which
            // was the whole bug: only the callee ever saw anything while a
            // call was ringing. Dashed border + no action urgency (just
            // Cancel) so it visually reads as "waiting", distinct from the
            // callee's bordered Accept/Decline prompt above.
            if let Some((_, name)) = ui.outgoing_call.cloned() {
                div {
                    style: "position:fixed; top:16px; left:50%; transform:translateX(-50%); \
                            background:var(--bg-secondary,#1a1a2e); border:1px dashed var(--border); \
                            border-radius:10px; padding:12px 16px; display:flex; align-items:center; \
                            gap:12px; z-index:1000;",
                    span { "📱 Calling {name}…" }
                    button { class: "secondary", onclick: move |_| spawn_hang_up(ui), "Cancel" }
                }
            }
            if let Some((_, name)) = ui.active_call.cloned() {
                div {
                    style: "position:fixed; top:16px; left:50%; transform:translateX(-50%); \
                            background:var(--accent,#0e9f6e); color:white; border-radius:10px; \
                            padding:10px 16px; display:flex; flex-direction:column; align-items:center; \
                            gap:8px; z-index:1000;",
                    div { style: "display:flex; align-items:center; gap:12px;",
                        span { if ui.active_call_has_video.cloned() { "🎥 On video call with {name}" } else { "🔊 On call with {name}" } }
                        button { class: "secondary", onclick: move |_| spawn_hang_up(ui), "Hang Up" }
                    }
                    if ui.active_call_has_video.cloned() {
                        div { style: "display:flex; gap:8px;",
                            div {
                                style: "width:220px; height:165px; background:#000; border-radius:8px; overflow:hidden; \
                                        display:flex; align-items:center; justify-content:center;",
                                if let Some(src) = ui.remote_video_frame.cloned() {
                                    img { src: "{src}", style: "width:100%; height:100%; object-fit:cover;" }
                                } else {
                                    span { style: "color:rgba(255,255,255,0.6); font-size:12px;", "Waiting for their camera…" }
                                }
                            }
                            div {
                                style: "width:110px; height:82px; background:#000; border-radius:8px; overflow:hidden; \
                                        display:flex; align-items:center; justify-content:center;",
                                if let Some(src) = ui.local_video_frame.cloned() {
                                    img { src: "{src}", style: "width:100%; height:100%; object-fit:cover;" }
                                } else {
                                    span { style: "color:rgba(255,255,255,0.6); font-size:10px;", "No camera" }
                                }
                            }
                        }
                    }
                }
            }
            ContextMenu { ui }
        }
    }
}

#[component]
fn MainShell(ui: UiState) -> Element {
    let core_ref = ui.core.read();
    let Some(core) = core_ref.as_ref() else {
        return rsx! { div { class: "empty-state", "Connecting…" } };
    };
    let my_username = core.my_username.clone();
    let my_ticket = core.my_ticket();

    let relay_ok = ui.relay_ok.cloned();
    let entries = build_chat_list(ui);
    let archived_count = {
        let dm_settings = ui.dm_settings_cache.read();
        ui.contacts
            .read()
            .iter()
            .filter(|c| {
                parse_hex(&c.endpoint_id)
                    .is_ok_and(|peer| dm_settings.get(&peer).is_some_and(|s| s.archived))
            })
            .count()
    };
    let pending_count = {
        let accepted_ids: std::collections::HashSet<String> = ui
            .contacts
            .read()
            .iter()
            .map(|c| c.endpoint_id.clone())
            .collect();
        ui.pending_requests
            .read()
            .iter()
            .filter(|c| !accepted_ids.contains(&c.endpoint_id))
            .count()
    };
    let active = ui.active.cloned();

    rsx! {
        div { class: "titlebar app-header",
            div {
                class: "profile-trigger",
                onclick: move |_| {
                    let opening = !ui.show_profile.cloned();
                    ui.show_profile.clone().set(opening);
                    if opening {
                        spawn_load_storage_stats(ui);
                    }
                },
                Avatar {
                    hash: ui.my_avatar_hash.cloned(),
                    label: my_username.clone().unwrap_or_default(),
                    images: ui.avatar_images,
                    size_px: 26,
                }
                div { class: "profile-trigger-copy",
                    strong { "Siar" }
                    span { "@{my_username.clone().unwrap_or_default()}" }
                }
                span { class: "profile-settings-icon", "⚙" }
            }
            span { class: if relay_ok { "relay-pill" } else { "relay-pill degraded" },
                if relay_ok { "relay: ok" } else { "relay: degraded" }
            }
        }
        if ui.show_profile.cloned() {
            SettingsPanel {
                ui,
                username: my_username.unwrap_or_default(),
                my_id: core.my_id,
                ticket: my_ticket,
                relay_ok,
                storage_stats: ui.storage_stats.cloned(),
                storage_loading: ui.storage_loading.cloned(),
                on_refresh_storage: move |_| spawn_load_storage_stats(ui),
                on_close: move |_| ui.show_profile.clone().set(false),
                avatar_hash: ui.my_avatar_hash.cloned(),
                images: ui.avatar_images,
                on_change_avatar: move |_| spawn_change_avatar(ui),
                on_avatar_bytes: move |bytes: Vec<u8>| spawn_set_avatar(ui, bytes),
                on_media_error: move |message: String| push_toast(ui, message, true),
            }
        }
        if ui.show_conv_info.cloned() {
            {
                let my_id = core.my_id;
                let is_accepted = matches!(&active, Some(ConvKey::Dm(peer)) if core.is_accepted(*peer));
                let is_verified = match &active {
                    Some(ConvKey::Dm(peer)) => {
                        let hex = app::hex(*peer);
                        ui.contacts.read().iter().any(|c| c.endpoint_id == hex && c.verified)
                    }
                    _ => false,
                };
                rsx! {
                    ConvInfoPanel {
                        my_id,
                        room: ui.room_info.cloned(),
                        dm: ui.dm_info.cloned(),
                        is_accepted,
                        is_verified,
                        title_input: ui.conv_info_input,
                        on_save_room_title: move |title: String| {
                            if let Some(ConvKey::Room(name)) = ui.active.read().clone() {
                                spawn_set_room_title(ui, name, title);
                            }
                        },
                        on_remove_member: move |target: [u8; 32]| {
                            if let Some(ConvKey::Room(name)) = ui.active.read().clone() {
                                spawn_remove_room_member(ui, name, target);
                            }
                        },
                        on_copy_invite: move |_| {
                            if let Some(ConvKey::Room(name)) = ui.active.read().clone() {
                                if let Some(core) = ui.core.read().as_ref() {
                                    match siar_core::ticket::encode_room(&name, core.my_addr()) {
                                        Ok(ticket) => {
                                            copy_to_clipboard(ticket);
                                            push_toast(ui, "Invite ticket copied".to_string(), false);
                                        }
                                        Err(e) => push_toast(ui, format!("couldn't build invite ticket: {e}"), true),
                                    }
                                }
                            }
                        },
                        on_save_dm_title: move |title: String| {
                            if let Some(ConvKey::Dm(peer)) = ui.active.read().clone() {
                                spawn_set_dm_title(ui, peer, if title.trim().is_empty() { None } else { Some(title) });
                            }
                        },
                        on_toggle_pinned: move |_| {
                            if let Some(ConvKey::Dm(peer)) = ui.active.read().clone() {
                                let pinned = ui.dm_info.read().as_ref().map(|s| s.pinned).unwrap_or(false);
                                spawn_set_dm_pinned(ui, peer, !pinned);
                            }
                        },
                        on_toggle_archived: move |_| {
                            if let Some(ConvKey::Dm(peer)) = ui.active.read().clone() {
                                let archived = ui.dm_info.read().as_ref().map(|s| s.archived).unwrap_or(false);
                                spawn_set_dm_archived(ui, peer, !archived);
                            }
                        },
                        on_set_ttl: move |ttl: Option<u64>| {
                            if let Some(ConvKey::Dm(peer)) = ui.active.read().clone() {
                                spawn_set_dm_disappearing_ttl(ui, peer, ttl);
                            }
                        },
                        on_invalid_ttl: move |msg: String| push_toast(ui, msg, true),
                        custom_ttl_hours: ui.custom_ttl_hours,
                        on_toggle_verified: move |_| {
                            if let Some(ConvKey::Dm(peer)) = ui.active.read().clone() {
                                spawn_toggle_verified(ui, peer);
                            }
                        },
                        on_block: move |_| {
                            if let Some(ConvKey::Dm(peer)) = ui.active.read().clone() {
                                spawn_block_contact(ui, peer);
                            }
                        },
                        on_close: move |_| {
                            ui.show_conv_info.clone().set(false);
                            ui.room_info.clone().set(None);
                            ui.dm_info.clone().set(None);
                        },
                    }
                }
            }
        }
        match ui.bottom_nav.cloned() {
            BottomNav::Chats | BottomNav::Dms | BottomNav::Groups => rsx! {
                div {
                    class: match (active.is_some(), *ui.sidebar_tab.read()) {
                        (_, SidebarTab::Requests) => "main-layout requests-open",
                        (true, SidebarTab::Chats) => "main-layout chat-open",
                        (false, SidebarTab::Chats) => "main-layout",
                    },
                    Sidebar {
                        tab: ui.sidebar_tab,
                        entries,
                        active: active.clone(),
                        pending_request_count: pending_count,
                        search_query: ui.search_query,
                        search_results: ui.search_results.cloned(),
                        room_input: ui.room_input,
                        showing_archived: ui.show_archived.cloned(),
                        archived_count,
                        connecting: ui.connecting.cloned(),
                        images: ui.avatar_images,
                        on_select: move |key: ConvKey| select_conversation(ui, key),
                        on_search_input: move |q: String| spawn_search(ui, q),
                        on_send_request: move |username: String| spawn_send_request(ui, username),
                        on_connect_ticket: move |ticket_str: String| spawn_connect_ticket(ui, ticket_str),
                        on_scan_qr_image: move |bytes: Vec<u8>| spawn_connect_qr_image(ui, bytes),
                        on_scan_qr_error: move |message: String| push_toast(ui, message, true),
                        on_open_existing_contact: move |username: String| open_existing_contact_by_username(ui, username),
                        on_join_room: move |name: String| spawn_join_room(ui, name),
                        on_toggle_archived_view: move |_| {
                            let v = !ui.show_archived.cloned();
                            ui.show_archived.clone().set(v);
                        },
                        on_row_context_menu: move |(x, y, key, pinned, archived): (f64, f64, ConvKey, bool, bool)| {
                            ui.context_menu.clone().set(Some(ContextMenuState {
                                x,
                                y,
                                kind: ContextMenuKind::ChatRow { key, pinned, archived },
                            }));
                        },
                    }
                    match *ui.sidebar_tab.read() {
                        SidebarTab::Requests => {
                            let accepted_ids: std::collections::HashSet<String> =
                                ui.contacts.read().iter().map(|c| c.endpoint_id.clone()).collect();
                            let requests: Vec<RequestEntry> = ui.pending_requests.read().iter()
                                .filter(|c| !accepted_ids.contains(&c.endpoint_id))
                                .map(|c| RequestEntry {
                                endpoint_id: c.endpoint_id.clone(),
                                display_name: c.alias.clone(),
                                username: c.username.clone(),
                                note: String::new(),
                                requested_label: relative_time(c.requested_at),
                            }).collect();
                            rsx! {
                                RequestsInbox {
                                    requests,
                                    on_accept: move |id: String| spawn_accept(ui, id),
                                    on_decline: move |id: String| spawn_decline(ui, id),
                                    on_back: move |_| ui.sidebar_tab.clone().set(SidebarTab::Chats),
                                }
                            }
                        }
                SidebarTab::Chats => {
                    if let Some(key) = active.clone() {
                        let (title, mut subtitle) = title_for(ui, &key);
                        if is_peer_typing(ui, &key) {
                            subtitle = "typing…".to_string();
                        }
                        let messages = bubbles_for(ui, &key);
                        let key_for_typing = key.clone();
                        let key_for_download = key.clone();
                        let key_for_info = key.clone();
                        let call_peer = if let ConvKey::Dm(peer) = &key { Some(*peer) } else { None };
                        let avatar_hash = call_peer.and_then(|peer| {
                            let hex_id = app::hex(peer);
                            ui.contacts.read().iter().find(|c| c.endpoint_id == hex_id).and_then(|c| c.avatar_hash.clone())
                        });
                        rsx! {
                            ChatPane {
                                title,
                                subtitle,
                                messages,
                                compose: ui.compose,
                                on_send: move |_| send_to_active(ui),
                                on_attach_file: move |_| spawn_attach_file(ui),
                                on_attach_bytes: move |(name, bytes): (String, Vec<u8>)| spawn_attach_file_bytes(ui, name, bytes),
                                on_attach_error: move |message: String| push_toast(ui, message, true),
                                on_back: move |_| ui.active.clone().set(None),
                                on_typing: move |_| maybe_send_typing(ui, key_for_typing.clone()),
                                on_download_file: move |hash: String| spawn_download_file(ui, key_for_download.clone(), hash),
                                on_open_info: move |_| spawn_open_conv_info(ui, key_for_info.clone()),
                                on_react: {
                                    let key = key.clone();
                                    move |(target_id, emoji): (u64, String)| spawn_send_reaction(ui, key.clone(), target_id, emoji)
                                },
                                on_edit_start: move |(target_id, text): (u64, String)| {
                                    ui.editing.clone().set(Some(target_id));
                                    ui.compose.clone().set(text);
                                },
                                on_delete: {
                                    let key = key.clone();
                                    move |target_id: u64| spawn_send_delete(ui, key.clone(), target_id)
                                },
                                on_reply_start: move |(target_id, sender, snippet): (u64, String, String)| {
                                    ui.replying_to.clone().set(Some((target_id, sender, snippet)));
                                },
                                pending_reply: ui.replying_to.read().as_ref().map(|(_, sender, snippet)| (sender.clone(), snippet.clone())),
                                on_cancel_reply: move |_| ui.replying_to.clone().set(None),
                                on_bubble_context_menu: move |(x, y, target_id, is_own, deleted, sender_label, snippet): (f64, f64, u64, bool, bool, String, String)| {
                                    ui.context_menu.clone().set(Some(ContextMenuState {
                                        x,
                                        y,
                                        kind: ContextMenuKind::Bubble { target_id, is_own, deleted, sender_label, snippet },
                                    }));
                                },
                                on_call: call_peer.map(|peer| EventHandler::new(move |_| spawn_call_peer(ui, peer))),
                                on_video_call: call_peer.map(|peer| EventHandler::new(move |_| spawn_video_call_peer(ui, peer))),
                                avatar_hash,
                                images: ui.avatar_images,
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "chat-pane empty-state",
                                div { style: "font-size: 20px;", "Select a chat" }
                                div { "Search a username on the left to start a new conversation" }
                            }
                        }
                    }
                }
                    }
                }
            },
            BottomNav::Calls => rsx! { CallsView { entries: ui.call_log.cloned(), on_call: move |peer: EndpointId| spawn_call_peer(ui, peer) } },
            BottomNav::Status => rsx! {
                StatusView {
                    statuses: ui.statuses.cloned(),
                    my_id: ui.core.read().as_ref().map(|c| c.my_id),
                    compose: ui.status_compose,
                    status_image: ui.status_image,
                    status_images: ui.avatar_images,
                    status_video_pending: ui.status_video_pending,
                    status_recording: ui.status_recording,
                    status_audio_pending: ui.status_audio_pending,
                    status_recording_audio: ui.status_recording_audio,
                    story_viewer: ui.story_viewer,
                    own_story_viewer: ui.own_story_viewer,
                    on_post: move |_| spawn_post_status(ui),
                    on_attach_image: move |_| spawn_attach_status_image(ui),
                    on_attach_image_bytes: move |bytes: Vec<u8>| spawn_set_status_image(ui, bytes),
                    on_record_video: move |_| spawn_record_status_video(ui),
                    on_record_audio: move |_| spawn_record_status_audio(ui),
                    on_attach_audio: move |_| spawn_attach_status_audio(ui),
                    on_attach_audio_bytes: move |(name, bytes): (String, Vec<u8>)| spawn_set_status_audio(ui, bytes, Some(name)),
                    on_media_error: move |message: String| push_toast(ui, message, true),
                }
            },
        }
        BottomNavBar {
            current: ui.bottom_nav.cloned(),
            chat_open: active.is_some() && *ui.sidebar_tab.read() == SidebarTab::Chats,
            on_select: move |nav: BottomNav| {
                ui.active.clone().set(None);
                ui.sidebar_tab.clone().set(SidebarTab::Chats);
                ui.bottom_nav.clone().set(nav);
            }
        }
    }
}

/// The five-section bottom navigation — Chats / DMs / Groups / Calls /
/// Status, WhatsApp/Signal-style. Purely presentational; `bottom_nav`
/// state and what each section shows lives in the parent.
#[component]
fn BottomNavBar(
    current: BottomNav,
    chat_open: bool,
    on_select: EventHandler<BottomNav>,
) -> Element {
    let tabs = [
        (BottomNav::Chats, "💬", "Chats"),
        (BottomNav::Dms, "👤", "DMs"),
        (BottomNav::Groups, "👥", "Groups"),
        (BottomNav::Calls, "📞", "Calls"),
        (BottomNav::Status, "◎", "Status"),
    ];
    rsx! {
        nav {
            class: if chat_open { "bottom-nav chat-open" } else { "bottom-nav" },
            aria_label: "Primary navigation",
            for (nav, icon, label) in tabs {
                button {
                    class: if current == nav { "bottom-nav-item active" } else { "bottom-nav-item" },
                    aria_label: "{label}",
                    onclick: move |_| on_select.call(nav),
                    span { class: "bottom-nav-icon", "{icon}" }
                    span { class: "bottom-nav-label", "{label}" }
                }
            }
        }
    }
}

/// Call history — most recent first, tap to redial. Read-only aside from
/// that; there's no "clear log" or per-entry delete in this pass.
#[component]
fn CallsView(entries: Vec<CallLogEntry>, on_call: EventHandler<EndpointId>) -> Element {
    rsx! {
        div { class: "sidebar-list", style: "flex:1; overflow-y:auto;",
            if entries.is_empty() {
                div { class: "empty-state",
                    div { style: "font-size: 20px;", "No calls yet" }
                    div { "Calls you make or receive will show up here" }
                }
            }
            for entry in entries.iter() {
                if let Ok(peer) = parse_hex(&entry.peer_id) {
                    {
                        let icon = match (entry.direction, entry.outcome) {
                            (_, CallOutcome::Missed) => "↙️ missed",
                            (_, CallOutcome::Declined) => "↙️ declined",
                            (_, CallOutcome::Failed) => "⚠️ failed",
                            (CallDirection::Incoming, CallOutcome::Completed) => "↙️",
                            (CallDirection::Outgoing, CallOutcome::Completed) => "↗️",
                        };
                        let duration = if entry.duration_secs > 0 {
                            format!(" · {}m{:02}s", entry.duration_secs / 60, entry.duration_secs % 60)
                        } else {
                            String::new()
                        };
                        rsx! {
                            div { class: "chat-row", key: "{entry.started_at_ms}-{entry.peer_id}",
                                div { class: "avatar", "{first_char(&entry.peer_name)}" }
                            div { class: "chat-row-body",
                                div { class: "chat-row-name", "{entry.peer_name}" }
                                div { style: "font-size:12px; color:var(--text-muted);",
                                    "{icon}{duration} · {relative_time(entry.started_at_ms)}"
                                }
                            }
                            button { onclick: move |_| on_call.call(peer), "📞" }
                        }
                    }
                    }
                }
            }
        }
    }
}

/// Status updates (WhatsApp/Signal "story"-style): "My status" up top —
/// your own avatar/preview beside a vertical stack of compose controls
/// (text, image attach, expiry, post) — then a vertical list of rows
/// below, one per contact with a currently-active status, each stacking
/// downward and tappable to open the full-screen story viewer. (An
/// earlier pass here used a horizontal ring-scroll row instead; this is
/// the WhatsApp Status tab's actual layout — its rings only show up in
/// the separate Stories/Channels row, not the plain Status list.)
#[component]
fn StatusView(
    statuses: Vec<StatusEntry>,
    my_id: Option<EndpointId>,
    compose: Signal<String>,
    status_image: Signal<Option<Vec<u8>>>,
    status_images: Signal<HashMap<String, String>>,
    status_video_pending: Signal<Option<Vec<image::RgbImage>>>,
    status_recording: Signal<bool>,
    status_audio_pending: Signal<Option<Vec<u8>>>,
    status_recording_audio: Signal<bool>,
    story_viewer: Signal<Option<usize>>,
    own_story_viewer: Signal<bool>,
    on_post: EventHandler<()>,
    on_attach_image: EventHandler<()>,
    on_attach_image_bytes: EventHandler<Vec<u8>>,
    on_record_video: EventHandler<()>,
    on_record_audio: EventHandler<()>,
    on_attach_audio: EventHandler<()>,
    on_attach_audio_bytes: EventHandler<(String, Vec<u8>)>,
    on_media_error: EventHandler<String>,
) -> Element {
    let mut compose_open = use_signal(|| false);
    let my_id_hex = my_id.map(app::hex);
    let (mine, theirs): (Vec<_>, Vec<_>) = statuses
        .into_iter()
        .partition(|s| my_id_hex.as_deref() == Some(s.peer_id.as_str()));
    let has_own_status = !mine.is_empty();
    let preview_data_uri = status_image.read().as_ref().map(|bytes| {
        format!(
            "data:image/*;base64,{}",
            data_encoding::BASE64.encode(bytes)
        )
    });

    rsx! {
        div { class: "status-view sidebar-list",
            // "My status" — avatar/preview on the left, everything you'd
            // do with it stacked vertically to its right, as one section.
            div {
                class: "status-own-card",
                style: "display:flex; gap:14px; align-items:flex-start; background: var(--bg-secondary, rgba(255,255,255,0.04)); \
                        border:1px solid var(--border); border-radius:10px; padding:12px; margin-bottom:16px;",
                div {
                    style: "flex-shrink:0; cursor:pointer;",
                    onclick: move |_| if has_own_status { own_story_viewer.clone().set(true) },
                    div {
                        style: format!(
                            "width:56px; height:56px; border-radius:50%; overflow:hidden; display:flex; \
                             align-items:center; justify-content:center; font-size:20px; font-weight:600; \
                             border:2px solid {}; background:var(--surface); position:relative;",
                            if has_own_status { "var(--accent)" } else { "var(--border)" },
                        ),
                        if let Some(entry) = mine.first() {
                            if let Some(hash) = &entry.image_hash {
                                if let Some(src) = status_images.read().get(hash) {
                                    img { src: "{src}", style: "width:100%; height:100%; object-fit:cover;" }
                                } else {
                                    "✓"
                                }
                            } else {
                                "✓"
                            }
                        }
                        if !has_own_status {
                            div {
                                style: "position:absolute; bottom:-2px; right:-2px; width:18px; height:18px; \
                                        border-radius:50%; background:var(--accent); color:white; \
                                        display:flex; align-items:center; justify-content:center; font-size:13px;",
                                "+"
                            }
                        }
                    }
                }
                div { style: "flex:1; display:flex; flex-direction:column; gap:8px;",
                    div { class: "status-compose-heading",
                        div {
                            div { style: "font-weight:600;", "My status" }
                            div { class: "status-compose-hint", if has_own_status { "Tap your photo to view" } else { "Share a photo, voice note, or update" } }
                        }
                        button {
                            class: "status-compose-toggle",
                            aria_label: if compose_open() { "Close status composer" } else { "Create status" },
                            onclick: move |_| compose_open.toggle(),
                            if compose_open() { "×" } else { "+" }
                        }
                    }
                    if let Some(entry) = mine.first() {
                        div { class: "status-current-summary",
                            if entry.text.is_empty() { "Currently posted · expires {relative_time(entry.expires_at_ms)}" }
                            else { "Currently: \"{entry.text}\" · expires {relative_time(entry.expires_at_ms)}" }
                        }
                    }
                    if compose_open() {
                    div { class: "status-editor",
                    textarea {
                        class: "status-text-input",
                        style: "width:100%; min-height:60px; background:var(--surface); border:1px solid var(--border); \
                                color:var(--text); border-radius:8px; padding:8px; resize:vertical;",
                        placeholder: "Type a status…",
                        value: "{compose}",
                        oninput: move |e| compose.set(e.value()),
                    }
                    if let Some(src) = preview_data_uri {
                        div { style: "display:flex; align-items:center; gap:8px;",
                            img { src: "{src}", style: "max-height:80px; border-radius:6px;" }
                            button { class: "secondary", onclick: move |_| status_image.clone().set(None), "Remove image" }
                        }
                    }
                    if let Some(frames) = status_video_pending.read().as_ref() {
                        div { style: "display:flex; align-items:center; gap:8px;",
                            span { style: "font-size:13px; color:var(--text-muted);",
                                "🎥 {frames.len()}-frame clip recorded (~{STATUS_VIDEO_RECORD_SECS}s)"
                            }
                            button { class: "secondary", onclick: move |_| status_video_pending.clone().set(None), "Remove video" }
                        }
                    }
                    if status_audio_pending.read().is_some() {
                        div { style: "display:flex; align-items:center; gap:8px;",
                            span { style: "font-size:13px; color:var(--text-muted);", "🎤 voice clip recorded (~{STATUS_AUDIO_RECORD_SECS}s)" }
                            button { class: "secondary", onclick: move |_| status_audio_pending.clone().set(None), "Remove voice clip" }
                        }
                    }
                    div { class: "status-tools",
                        if cfg!(any(target_os = "android", target_os = "ios")) {
                            label { class: "status-tool", r#for: "status-camera-input", "📷", span { "Camera" } }
                            input {
                                id: "status-camera-input", class: "visually-hidden-file", r#type: "file", accept: "image/*", capture: "environment",
                                onchange: move |event| {
                                    let Some(file) = event.files().into_iter().next() else { return };
                                    if file.size() > 20 * 1024 * 1024 { on_media_error.call("Status image is too large (20 MB maximum)".to_string()); return; }
                                    spawn(async move { match file.read_bytes().await {
                                        Ok(bytes) => on_attach_image_bytes.call(bytes.to_vec()),
                                        Err(error) => on_media_error.call(format!("couldn't read camera image: {error}")),
                                    }});
                                },
                            }
                            label { class: "status-tool", r#for: "status-gallery-input", "▧", span { "Gallery" } }
                            input {
                                id: "status-gallery-input", class: "visually-hidden-file", r#type: "file", accept: "image/*",
                                onchange: move |event| {
                                    let Some(file) = event.files().into_iter().next() else { return };
                                    if file.size() > 20 * 1024 * 1024 { on_media_error.call("Status image is too large (20 MB maximum)".to_string()); return; }
                                    spawn(async move { match file.read_bytes().await {
                                        Ok(bytes) => on_attach_image_bytes.call(bytes.to_vec()),
                                        Err(error) => on_media_error.call(format!("couldn't read gallery image: {error}")),
                                    }});
                                },
                            }
                        } else {
                            button { class: "status-tool", onclick: move |_| on_attach_image.call(()), "📷", span { "Photo" } }
                        }
                        if cfg!(not(any(target_os = "android", target_os = "ios"))) {
                            if status_recording.cloned() {
                                button { class: "status-tool", disabled: true, "🎥", span { "Recording…" } }
                            } else {
                                button { class: "status-tool", onclick: move |_| on_record_video.call(()), "🎥", span { "Video" } }
                            }
                        }
                        if status_recording_audio.cloned() {
                            button { class: "status-tool", disabled: true, "🎤", span { "Recording…" } }
                        } else {
                            button { class: "status-tool", onclick: move |_| on_record_audio.call(()), "🎤", span { "Voice" } }
                        }
                        if cfg!(any(target_os = "android", target_os = "ios")) {
                            label { class: "status-tool", r#for: "status-audio-input", "♫", span { "Audio" } }
                            input {
                                id: "status-audio-input", class: "visually-hidden-file", r#type: "file", accept: "audio/*",
                                onchange: move |event| {
                                    let Some(file) = event.files().into_iter().next() else { return };
                                    if file.size() > 32 * 1024 * 1024 { on_media_error.call("Audio file is too large (32 MB maximum)".to_string()); return; }
                                    let name = file.name();
                                    spawn(async move { match file.read_bytes().await {
                                        Ok(bytes) => on_attach_audio_bytes.call((name, bytes.to_vec())),
                                        Err(error) => on_media_error.call(format!("couldn't read audio file: {error}")),
                                    }});
                                },
                            }
                        } else {
                            button { class: "status-tool", onclick: move |_| on_attach_audio.call(()), "♫", span { "Audio" } }
                        }
                    }
                    div { class: "status-editor-footer",
                        span { "Disappears after 24 hours" }
                        button { onclick: move |_| { on_post.call(()); compose_open.set(false); }, "Send status" }
                    }
                    }
                    }
                }
            }
            div { class: "status-section-label", "Recent updates" }
            if theirs.is_empty() {
                div { style: "color:var(--text-muted);", "No one you know has an active status right now." }
            }
            // Vertical list, each row stacking downward — tap a row to
            // open the full-screen viewer at that position.
            div { style: "display:flex; flex-direction:column;",
                for (i, entry) in theirs.iter().enumerate() {
                    div {
                        key: "{entry.peer_id}",
                        class: "chat-row",
                        style: "cursor:pointer;",
                        onclick: move |_| story_viewer.clone().set(Some(i)),
                        div {
                            class: "avatar",
                            style: "width:48px; height:48px; border-radius:50%; overflow:hidden; \
                                    border:2px solid var(--accent); flex-shrink:0;",
                            if let Some(hash) = &entry.image_hash {
                                if let Some(src) = status_images.read().get(hash) {
                                    img { src: "{src}", style: "width:100%; height:100%; object-fit:cover;" }
                                } else {
                                    "{first_char(&entry.peer_name)}"
                                }
                            } else if let Some(hash) = &entry.video_hash {
                                if let Some(src) = status_images.read().get(hash) {
                                    img { src: "{src}", style: "width:100%; height:100%; object-fit:cover;" }
                                } else {
                                    "▶"
                                }
                            } else {
                                "{first_char(&entry.peer_name)}"
                            }
                        }
                        div { class: "chat-row-body",
                            div { class: "chat-row-name", "{entry.peer_name}" }
                            if !entry.text.is_empty() {
                                div { style: "font-size:13px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;", "{entry.text}" }
                            }
                            if entry.video_hash.is_some() {
                                div { style: "font-size:11px; color:var(--text-muted);", "🎥 video" }
                            }
                            if entry.audio_hash.is_some() {
                                div { style: "font-size:11px; color:var(--text-muted);", "🎤 voice clip" }
                            }
                            div { style: "font-size:11px; color:var(--text-muted);", "{relative_time(entry.posted_at_ms)}" }
                        }
                    }
                }
            }
            if let Some(i) = story_viewer.cloned() {
                StoryViewerOverlay {
                    entries: theirs.clone(),
                    index: i,
                    status_images,
                    on_close: move |_| story_viewer.clone().set(None),
                    on_navigate: move |new_i: usize| story_viewer.clone().set(Some(new_i)),
                }
            }
            if own_story_viewer.cloned() {
                StoryViewerOverlay {
                    entries: mine.clone(),
                    index: 0usize,
                    status_images,
                    on_close: move |_| own_story_viewer.clone().set(false),
                    on_navigate: move |_: usize| own_story_viewer.clone().set(false),
                }
            }
        }
    }
}

/// Full-screen story viewer — tap the right two-thirds to advance, the
/// left third to go back, X to close. No auto-advance timer in this pass
/// (each status here is a single entry per person, not a stack of
/// several, so there's less pressure for it than a real multi-story
/// stack has — a deliberate, honest scope cut, not an oversight).
#[component]
fn StoryViewerOverlay(
    entries: Vec<StatusEntry>,
    index: usize,
    status_images: Signal<HashMap<String, String>>,
    on_close: EventHandler<()>,
    on_navigate: EventHandler<usize>,
) -> Element {
    let Some(entry) = entries.get(index) else {
        return rsx! {};
    };
    let count = entries.len();
    let image_src = entry
        .image_hash
        .as_ref()
        .and_then(|h| status_images.read().get(h).cloned());
    let video_src = entry
        .video_hash
        .as_ref()
        .and_then(|h| status_images.read().get(h).cloned());
    let audio_src = entry
        .audio_hash
        .as_ref()
        .and_then(|h| status_images.read().get(h).cloned());

    rsx! {
        div {
            style: "position:fixed; inset:0; background:#000; z-index:2000; display:flex; \
                    flex-direction:column; align-items:center; justify-content:center;",
            // Progress segments — one per person being paged through, not
            // per-status (see this component's own doc).
            div { style: "position:absolute; top:12px; left:12px; right:12px; display:flex; gap:4px;",
                for i in 0..count {
                    div {
                        key: "{i}",
                        style: format!(
                            "flex:1; height:3px; border-radius:2px; background:{};",
                            if i <= index { "rgba(255,255,255,0.9)" } else { "rgba(255,255,255,0.3)" },
                        ),
                    }
                }
            }
            div { style: "position:absolute; top:24px; left:16px; color:white; font-weight:600;", "{entry.peer_name}" }
            button {
                style: "position:absolute; top:16px; right:16px; background:none; border:none; \
                        color:white; font-size:24px; cursor:pointer;",
                onclick: move |_| on_close.call(()),
                "✕"
            }
            if let Some(src) = image_src {
                img { src: "{src}", style: "max-width:90%; max-height:70%; border-radius:8px; object-fit:contain;" }
            } else if let Some(src) = video_src {
                img { src: "{src}", style: "max-width:90%; max-height:70%; border-radius:8px; object-fit:contain;" }
            } else if entry.video_hash.is_some() {
                div { style: "color:rgba(255,255,255,0.6); font-size:14px;", "loading video…" }
            }
            if let Some(src) = audio_src {
                audio { controls: true, autoplay: true, src: "{src}", style: "margin-top:16px;" }
            } else if entry.audio_hash.is_some() {
                div { style: "color:rgba(255,255,255,0.6); font-size:14px; margin-top:16px;", "loading voice clip…" }
            }
            if !entry.text.is_empty() {
                div { style: "color:white; font-size:20px; text-align:center; padding:24px; max-width:80%;", "{entry.text}" }
            }
            div { style: "position:absolute; bottom:24px; color:rgba(255,255,255,0.6); font-size:12px;",
                "{relative_time(entry.posted_at_ms)} · expires {relative_time(entry.expires_at_ms)}"
            }
            // Tap zones, left third = previous, right two-thirds = next/close.
            div {
                style: "position:absolute; inset:0; display:flex;",
                div {
                    style: "flex:1; cursor:pointer;",
                    onclick: move |_| if index > 0 { on_navigate.call(index - 1) },
                }
                div {
                    style: "flex:2; cursor:pointer;",
                    onclick: move |_| if index + 1 < count { on_navigate.call(index + 1) } else { on_close.call(()) },
                }
            }
        }
    }
}

/// Conversation-info drawer — the WhatsApp/Signal-style "tap the header"
/// panel, backed by `net::conv_docs`. Shows title + membership for a room
/// (rename/remove available only to the recorded admin — see
/// `net::conv_docs::RoomDoc`'s doc on that being a UI convention, not a
/// namespace-enforced permission), or shared settings for a DM (nickname,
/// pin, archive, disappearing-message TTL).
#[component]
fn ConvInfoPanel(
    my_id: EndpointId,
    room: Option<(RoomMeta, Vec<MemberRecord>)>,
    dm: Option<DmSettings>,
    is_accepted: bool,
    is_verified: bool,
    title_input: Signal<String>,
    on_save_room_title: EventHandler<String>,
    on_remove_member: EventHandler<[u8; 32]>,
    on_copy_invite: EventHandler<()>,
    on_save_dm_title: EventHandler<String>,
    on_toggle_pinned: EventHandler<()>,
    on_toggle_archived: EventHandler<()>,
    on_set_ttl: EventHandler<Option<u64>>,
    on_invalid_ttl: EventHandler<String>,
    /// Text state for the "Custom" hours field below the presets — kept
    /// as a signal (not local state) so it survives the panel closing and
    /// reopening mid-edit, same reasoning as `title_input`.
    custom_ttl_hours: Signal<String>,
    on_toggle_verified: EventHandler<()>,
    on_block: EventHandler<()>,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "modal-backdrop",
            onclick: move |_| on_close.call(()),
            div {
                class: "conv-info-modal",
                style: "background: var(--bg-elevated); border-radius: 14px; padding: 24px; width: 360px; max-width: 90vw; color: var(--text);",
                onclick: move |e| e.stop_propagation(),
                if let Some((meta, members)) = room.clone() {
                    div { style: "font-size:18px; font-weight:600; margin-bottom:4px;", "Room info" }
                    div { style: "font-size:13px; color:var(--text-muted); margin-bottom:16px;", "#{meta.name}" }
                    div { style: "display:flex; gap:8px; margin-bottom:16px;",
                        input {
                            style: "flex:1; background: var(--surface); border: 1px solid var(--border); color: var(--text); border-radius:8px; padding:6px 10px;",
                            placeholder: "Room title",
                            value: "{title_input}",
                            oninput: move |e| title_input.set(e.value()),
                        }
                        button {
                            onclick: move |_| on_save_room_title.call(title_input.cloned()),
                            "Save"
                        }
                    }
                    div { style: "display:flex; align-items:center; justify-content:space-between; margin-bottom:8px;",
                        div { style: "font-size:13px; color:var(--text-muted);",
                            "{members.len()} member(s)"
                        }
                        button {
                            class: "secondary",
                            onclick: move |_| on_copy_invite.call(()),
                            "Copy invite ticket"
                        }
                    }
                    div { style: "max-height:240px; overflow-y:auto;",
                        for member in members.iter() {
                            {
                                let is_me = member.endpoint_id == *my_id.as_bytes();
                                let is_admin = meta.admin == *my_id.as_bytes();
                                let target = member.endpoint_id;
                                rsx! {
                                    div {
                                        style: "display:flex; align-items:center; justify-content:space-between; padding:6px 0; border-bottom: 1px solid var(--border);",
                                        span {
                                            "{member.display_name}"
                                            if is_me { " (you)" }
                                            if member.endpoint_id == meta.admin { " · admin" }
                                        }
                                        if is_admin && !is_me {
                                            button {
                                                class: "secondary",
                                                style: "color: var(--danger);",
                                                onclick: move |_| on_remove_member.call(target),
                                                "Remove"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if let Some(settings) = dm.clone() {
                    div { style: "font-size:18px; font-weight:600; margin-bottom:16px;", "Conversation info" }
                    div { style: "display:flex; gap:8px; margin-bottom:16px;",
                        input {
                            style: "flex:1; background: var(--surface); border: 1px solid var(--border); color: var(--text); border-radius:8px; padding:6px 10px;",
                            placeholder: "Nickname for this chat",
                            value: "{title_input}",
                            oninput: move |e| title_input.set(e.value()),
                        }
                        button {
                            onclick: move |_| on_save_dm_title.call(title_input.cloned()),
                            "Save"
                        }
                    }
                    div {
                        style: "display:flex; align-items:center; justify-content:space-between; padding:10px 0; \
                                border-bottom: 1px solid var(--border); border-top: 1px solid var(--border); margin-bottom:8px;",
                        div {
                            div { style: "font-weight: 500;",
                                if is_verified { "✓ Verified" } else { "Not verified" }
                            }
                            div { style: "font-size:11px; color:var(--text-muted); max-width: 240px;",
                                "Confirm this is really them by comparing Endpoint IDs in person, over a \
                                 trusted channel, or via QR — there's no central directory to vouch for it."
                            }
                        }
                        button {
                            class: "secondary",
                            onclick: move |_| on_toggle_verified.call(()),
                            if is_verified { "Unmark" } else { "Mark verified" }
                        }
                    }
                    div { style: "display:flex; align-items:center; justify-content:space-between; padding:8px 0; border-bottom: 1px solid var(--border);",
                        span { "Pinned" }
                        button {
                            class: "secondary",
                            onclick: move |_| on_toggle_pinned.call(()),
                            if settings.pinned { "Unpin" } else { "Pin" }
                        }
                    }
                    div { style: "display:flex; align-items:center; justify-content:space-between; padding:8px 0; border-bottom: 1px solid var(--border);",
                        span { "Archived" }
                        button {
                            class: "secondary",
                            onclick: move |_| on_toggle_archived.call(()),
                            if settings.archived { "Unarchive" } else { "Archive" }
                        }
                    }
                    div { style: "padding:8px 0;",
                        div { style: "margin-bottom:6px; color: var(--text-muted); font-size:13px;", "Disappearing messages" }
                        div { style: "display:flex; gap:6px; flex-wrap:wrap; margin-bottom:8px;",
                            {
                                // 24 hours is the WhatsApp/Signal-style
                                // default for "just turn it on" — it's
                                // listed right after Off, not buried among
                                // options, so it's the obvious first pick.
                                // Custom (below) covers anything else up
                                // to the 7-day/168-hour ceiling.
                                let options: [(&str, Option<u64>); 3] =
                                    [("Off", None), ("24 hours", Some(86_400)), ("7 days", Some(604_800))];
                                rsx! {
                                    for (label, value) in options {
                                        button {
                                            class: if settings.disappearing_ttl_secs == value { "" } else { "secondary" },
                                            onclick: move |_| on_set_ttl.call(value),
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                        div { style: "display:flex; gap:6px; align-items:center;",
                            span { style: "font-size:13px; color:var(--text-muted); white-space:nowrap;", "Custom:" }
                            input {
                                r#type: "number",
                                min: "1",
                                max: "168",
                                placeholder: "hours (1–168)",
                                style: "width:110px; background: var(--surface); border: 1px solid var(--border); \
                                        color: var(--text); border-radius:8px; padding:6px 10px;",
                                value: "{custom_ttl_hours}",
                                oninput: move |e| custom_ttl_hours.set(e.value()),
                            }
                            button {
                                class: "secondary",
                                onclick: move |_| {
                                    // 168 hours = 7 days — the same ceiling
                                    // as the "7 days" preset above; a custom
                                    // value is for picking something *between*
                                    // 24h and 7 days, not for going past it.
                                    match custom_ttl_hours.read().trim().parse::<u64>() {
                                        Ok(hours) if (1..=168).contains(&hours) => on_set_ttl.call(Some(hours * 3600)),
                                        Ok(_) => on_invalid_ttl.call("custom duration must be between 1 and 168 hours".to_string()),
                                        Err(_) => on_invalid_ttl.call("enter a whole number of hours".to_string()),
                                    }
                                },
                                "Set"
                            }
                        }
                    }
                    if is_accepted {
                        div { style: "padding-top:16px; margin-top:8px; border-top: 1px solid var(--border);",
                            button {
                                style: "color: var(--danger); width: 100%;",
                                class: "secondary",
                                onclick: move |_| on_block.call(()),
                                "Block contact"
                            }
                        }
                    }
                } else {
                    div { class: "empty-state", "Loading…" }
                }
                div { style: "margin-top:20px; text-align:right;",
                    button { class: "secondary", onclick: move |_| on_close.call(()), "Close" }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum SettingsTab {
    Profile,
    Appearance,
    Notifications,
    Privacy,
    Network,
    Keys,
    Storage,
    About,
}

#[component]
fn SettingsPanel(
    ui: UiState,
    username: String,
    my_id: EndpointId,
    ticket: String,
    relay_ok: bool,
    storage_stats: Option<StorageStats>,
    storage_loading: bool,
    on_refresh_storage: EventHandler<()>,
    on_close: EventHandler<()>,
    avatar_hash: Option<String>,
    images: Signal<HashMap<String, String>>,
    on_change_avatar: EventHandler<()>,
    on_avatar_bytes: EventHandler<Vec<u8>>,
    on_media_error: EventHandler<String>,
) -> Element {
    let tab = use_signal(|| SettingsTab::Profile);
    let notifications_enabled = use_signal(|| {
        ui.core
            .read()
            .as_ref()
            .is_none_or(|core| core.store().notifications_enabled())
    });
    let notification_sound_enabled = use_signal(|| {
        ui.core
            .read()
            .as_ref()
            .is_none_or(|core| core.store().notification_sound_enabled())
    });
    let read_receipts_enabled = use_signal(|| {
        ui.core
            .read()
            .as_ref()
            .is_none_or(|core| core.store().read_receipts_enabled())
    });
    let typing_indicator_enabled = use_signal(|| {
        ui.core
            .read()
            .as_ref()
            .is_none_or(|core| core.store().typing_indicator_enabled())
    });
    let background_wake_enabled = use_signal(|| {
        ui.core
            .read()
            .as_ref()
            .is_some_and(|core| core.store().background_wake_enabled())
    });
    let offline_mesh_enabled = use_signal(|| {
        ui.core
            .read()
            .as_ref()
            .is_some_and(Core::offline_mesh_enabled)
    });
    let qr_svg = render_qr_svg(&ticket);
    let data_dir = CONFIG
        .get()
        .map(|c| c.data_dir.display().to_string())
        .unwrap_or_default();

    rsx! {
        div {
            class: "modal-backdrop",
            onclick: move |_| on_close.call(()),
            div {
                class: "settings-modal",
                onclick: move |e| e.stop_propagation(),
                div { class: "settings-header",
                    h2 { "@{username}" }
                    div { class: "settings-status",
                        div { class: if relay_ok { "settings-status-dot ok" } else { "settings-status-dot degraded" } }
                        if relay_ok { "Connected — relay reachable" } else { "Degraded — no relay, direct/LAN peers only" }
                    }
                    div { class: "settings-tabs",
                        for (icon, label, value) in [
                            ("👤", "Profile", SettingsTab::Profile),
                            ("◐", "Appearance", SettingsTab::Appearance),
                            ("🔔", "Notifications", SettingsTab::Notifications),
                            ("🔒", "Privacy", SettingsTab::Privacy),
                            ("◉", "Network", SettingsTab::Network),
                            ("🔑", "Keys", SettingsTab::Keys),
                            ("▣", "Storage", SettingsTab::Storage),
                            ("ⓘ", "About", SettingsTab::About),
                        ] {
                            button {
                                class: if tab.cloned() == value { "settings-tab active" } else { "settings-tab" },
                                onclick: move |_| tab.clone().set(value),
                                span { class: "settings-tab-icon", "{icon}" }
                                span { "{label}" }
                            }
                        }
                    }
                }
                div { class: "settings-body",
                    {match tab.cloned() {
                        SettingsTab::Profile => rsx! {
                            div { class: "profile-card",
                                div { class: "avatar-ring",
                                    Avatar { hash: avatar_hash.clone(), label: username.clone(), images, size_px: 72 }
                                }
                                div { class: "username", "@{username}" }
                                if cfg!(any(target_os = "android", target_os = "ios")) {
                                    label { class: "settings-avatar-button", r#for: "settings-avatar-input", "Change photo" }
                                    input {
                                        id: "settings-avatar-input", class: "visually-hidden-file", r#type: "file", accept: "image/*",
                                        onchange: move |event| {
                                            let Some(file) = event.files().into_iter().next() else { return };
                                            if file.size() > 20 * 1024 * 1024 { on_media_error.call("Avatar image is too large (20 MB maximum)".to_string()); return; }
                                            spawn(async move { match file.read_bytes().await {
                                                Ok(bytes) => on_avatar_bytes.call(bytes.to_vec()),
                                                Err(error) => on_media_error.call(format!("couldn't read avatar image: {error}")),
                                            }});
                                        },
                                    }
                                } else {
                                    button { style: "margin-top:8px;", onclick: move |_| on_change_avatar.call(()), "Change avatar" }
                                }
                                div { class: "ticket-card",
                                    div { style: "display:flex; justify-content:center;", dangerous_inner_html: "{qr_svg}" }
                                    div { class: "ticket-value", "{ticket}" }
                                }
                                div { style: "display:flex; gap:8px; justify-content:center; width:100%;",
                                    button { style: "flex:1;", onclick: move |_| copy_to_clipboard(ticket.clone()), "Copy ticket" }
                                }
                                p { style: "font-size:12px; color:var(--text-muted); margin-top:16px; text-align:left;",
                                    "Share this ticket or QR code with someone so they can send you a contact request \
                                     — same idea as a Signal QR or a Keet room invite. "
                                    strong { style: "color:var(--text);", "This is the one that actually connects: " }
                                    "it's generated from this device's current live address, refreshed every time you \
                                     open this panel — unlike the address-less preview ticket shown during setup, which \
                                     only ever worked via best-effort public discovery."
                                }
                            }
                        },
                        SettingsTab::Appearance => rsx! {
                            div {
                                p { class: "settings-section-label", "Theme" }
                                div { style: "display:flex; gap:8px;",
                                    for (label, mode) in [("System", ThemeMode::System), ("Light", ThemeMode::Light), ("Dark", ThemeMode::Dark)] {
                                        button {
                                            class: if *ui.theme_mode.read() == mode { "" } else { "secondary" },
                                            onclick: move |_| {
                                                ui.theme_mode.clone().set(mode);
                                                if let Some(core) = ui.core.read().as_ref() {
                                                    let _ = core.store().set_theme_mode(mode);
                                                }
                                            },
                                            "{label}"
                                        }
                                    }
                                }
                                p { style: "font-size:12px; color:var(--text-muted); margin-top:12px;",
                                    "\"System\" follows your OS's light/dark setting automatically, live — no restart needed."
                                }
                                p { class: "settings-section-label", style: "margin-top:22px;", "Style" }
                                div { style: "display:flex; gap:8px; flex-wrap:wrap;",
                                    for (label, style) in [
                                        ("Regular", ThemeStyle::Regular),
                                        ("Hacker Green", ThemeStyle::HackerGreen),
                                        ("Hacker Red", ThemeStyle::HackerRed),
                                    ] {
                                        button {
                                            class: if *ui.theme_style.read() == style { "" } else { "secondary" },
                                            onclick: move |_| {
                                                ui.theme_style.clone().set(style);
                                                if let Some(core) = ui.core.read().as_ref() {
                                                    let _ = core.store().set_theme_style(style);
                                                }
                                            },
                                            "{label}"
                                        }
                                    }
                                }
                                p { style: "font-size:12px; color:var(--text-muted); margin-top:12px;",
                                    "Hacker Green and Hacker Red are their own fixed look — a monospace terminal palette, "
                                    "green or red on near-black — and don't follow the Theme setting above while active. "
                                    "Switch back to Regular to return to whatever Theme was already set."
                                }
                            }
                        },
                        SettingsTab::Notifications => rsx! {
                            div {
                                SettingsToggle {
                                    label: "Message notifications".to_string(),
                                    description: "Alert me about messages, calls, and contact requests.".to_string(),
                                    checked: notifications_enabled(),
                                    onchange: move |v| {
                                        notifications_enabled.clone().set(v);
                                        if let Some(core) = ui.core.read().as_ref() { let _ = core.store().set_notifications_enabled(v); }
                                        if v { request_android_notification_permission(); }
                                    }
                                }
                                SettingsToggle {
                                    label: "Notification sound".to_string(),
                                    description: "Play a sound with new notifications.".to_string(),
                                    checked: notification_sound_enabled(),
                                    onchange: move |v| {
                                        notification_sound_enabled.clone().set(v);
                                        if let Some(core) = ui.core.read().as_ref() { let _ = core.store().set_notification_sound_enabled(v); }
                                    },
                                }
                            }
                        },
                        SettingsTab::Privacy => rsx! {
                            {
                                let blocked: Vec<Contact> = ui.contacts.read().iter()
                                    .filter(|c| c.state == ContactState::Blocked)
                                    .cloned()
                                    .collect();
                                rsx! {
                                    div {
                                        SettingsToggle {
                                            label: "Send read receipts".to_string(),
                                            description: "Let contacts see when you've read their messages. Turning this off only stops your own signal — it doesn't hide receipts they already sent you.".to_string(),
                                            checked: read_receipts_enabled(),
                                            onchange: move |v| {
                                                read_receipts_enabled.clone().set(v);
                                                if let Some(core) = ui.core.read().as_ref() { let _ = core.store().set_read_receipts_enabled(v); }
                                            },
                                        }
                                        SettingsToggle {
                                            label: "Send typing indicators".to_string(),
                                            description: "Let contacts see the \"typing…\" bubble while you're composing a reply.".to_string(),
                                            checked: typing_indicator_enabled(),
                                            onchange: move |v| {
                                                typing_indicator_enabled.clone().set(v);
                                                if let Some(core) = ui.core.read().as_ref() { let _ = core.store().set_typing_indicator_enabled(v); }
                                            },
                                        }
                                        div { style: "margin-top:18px;",
                                            p { style: "font-size:12px; color:var(--text-muted); margin-bottom:8px;", "Blocked contacts" }
                                            if blocked.is_empty() {
                                                p { style: "font-size:12px; color:var(--text-muted);", "Nobody's blocked." }
                                            } else {
                                                for c in blocked {
                                                    div { style: "display:flex; justify-content:space-between; align-items:center; padding:6px 0;",
                                                        span { style: "font-size:13px;",
                                                            {c.username.clone().map(|u| format!("@{u}")).unwrap_or_else(|| c.alias.clone())}
                                                        }
                                                        button {
                                                            class: "secondary",
                                                            onclick: {
                                                                let id = c.endpoint_id.clone();
                                                                move |_| {
                                                                    if let Some(core) = ui.core.read().as_ref() {
                                                                        let _ = core.store().set_contact_state(&id, ContactState::Accepted);
                                                                        refresh_contacts(ui, core);
                                                                    }
                                                                }
                                                            },
                                                            "Unblock"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        SettingsTab::Network => rsx! {
                            {
                                let status = ui.core.read().as_ref().map(Core::mesh_status);
                                let (lan_active, ble_active, peers, relayed) = status
                                    .map(|s| (
                                        s.lan_active.load(std::sync::atomic::Ordering::Relaxed),
                                        s.ble_active.load(std::sync::atomic::Ordering::Relaxed),
                                        s.peers_seen_recently.load(std::sync::atomic::Ordering::Relaxed),
                                        s.envelopes_relayed.load(std::sync::atomic::Ordering::Relaxed),
                                    ))
                                    .unwrap_or((false, false, 0, 0));
                                rsx! {
                                    div {
                                        SettingsToggle {
                                            label: "Background wake (Android)".to_string(),
                                            description: "Stay reachable while Siar is in the background. Android shows a permanent notification and may use more battery.".to_string(),
                                            checked: background_wake_enabled(),
                                            onchange: move |v| {
                                                background_wake_enabled.clone().set(v);
                                                if let Some(core) = ui.core.read().as_ref() {
                                                    let _ = core.store().set_background_wake_enabled(v);
                                                }
                                                set_android_background_wake(v);
                                            },
                                        }
                                        SettingsToggle {
                                            label: if cfg!(target_os = "android") { "Nearby Wi-Fi mesh".to_string() } else { "Offline mesh (Bluetooth & Wi-Fi)".to_string() },
                                            description: "When internet or the relay is unavailable, try nearby peer-to-peer delivery. Both devices must enable it.".to_string(),
                                            checked: offline_mesh_enabled(),
                                            onchange: move |v| {
                                                offline_mesh_enabled.clone().set(v);
                                                if v { request_android_nearby_permissions(); }
                                                if let Some(core) = ui.core.read().as_ref() {
                                                    let store = core.store();
                                                    let mesh = core.mesh();
                                                    spawn(async move {
                                                        let _ = store.set_offline_mesh_enabled(v);
                                                        if v { mesh.start().await; } else { mesh.stop(); }
                                                    });
                                                }
                                            },
                                        }
                                        if offline_mesh_enabled() {
                                            div { class: "mesh-status-grid",
                                                div { class: "mesh-status-card",
                                                    div { class: "mesh-status-label", "Wi-Fi mesh" }
                                                    div { class: if lan_active { "mesh-status-value active" } else { "mesh-status-value" },
                                                        if lan_active { "Active" } else { "Starting…" }
                                                    }
                                                }
                                                div { class: "mesh-status-card",
                                                    div { class: "mesh-status-label", "Bluetooth mesh" }
                                                    div { class: if ble_active { "mesh-status-value active" } else { "mesh-status-value" },
                                                        if ble_active { "Scanning" } else { "Unavailable" }
                                                    }
                                                }
                                                div { class: "mesh-status-card",
                                                    div { class: "mesh-status-label", "Nearby signals" }
                                                    div { class: "mesh-status-value", "{peers}" }
                                                }
                                                div { class: "mesh-status-card",
                                                    div { class: "mesh-status-label", "Relayed this session" }
                                                    div { class: "mesh-status-value", "{relayed}" }
                                                }
                                            }
                                            p { style: "font-size:11px; color:var(--text-muted); margin-top:12px; line-height:1.6;",
                                                "Offline mesh is best-effort: there's no delivery confirmation the way there is over the normal relay/direct connection, and it only reaches contacts who are within Bluetooth or Wi-Fi range and also have this turned on."
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        SettingsTab::Keys => rsx! {
                            div {
                                div { style: "margin-bottom:16px;",
                                    div { style: "font-size:12px; color:var(--text-muted); margin-bottom:4px;", "Endpoint ID (your public identity)" }
                                    div { style: "display:flex; gap:8px; align-items:center;",
                                        code { style: "font-size:11px; word-break:break-all; flex:1;", "{app::hex(my_id)}" }
                                        button { class: "secondary", onclick: move |_| copy_to_clipboard(app::hex(my_id)), "Copy" }
                                    }
                                }
                                div {
                                    style: "background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 12px; font-size:12px; color: var(--text-muted);",
                                    p { style: "margin:0 0 8px 0; color: var(--text);", strong { "About your recovery phrase" } }
                                    p { style: "margin:0;",
                                        "Your 24-word recovery phrase was shown once, during setup, and was never \
                                         written to disk — this app can't show it to you again. If you saved it, it's \
                                         the only way to restore this identity on another device. If you didn't, this \
                                         identity only exists here; there's no server-side backup to fall back on."
                                    }
                                }
                            }
                        },
                        SettingsTab::Storage => rsx! {
                            div {
                                div { style: "margin-bottom:16px;",
                                    div { style: "font-size:12px; color:var(--text-muted); margin-bottom:4px;", "Data directory" }
                                    div { style: "display:flex; gap:8px; align-items:center;",
                                        code { style: "font-size:11px; word-break:break-all; flex:1;", "{data_dir}" }
                                        button { class: "secondary", onclick: move |_| copy_to_clipboard(data_dir.clone()), "Copy" }
                                    }
                                }
                                if storage_loading {
                                    div { class: "empty-state", style: "padding: 20px 0;", "Calculating…" }
                                } else if let Some(stats) = storage_stats {
                                    div {
                                        div { style: "display:flex; justify-content:space-between; padding:4px 0;", span { "Messages & contacts (sqlite)" } span { "{human_bytes(stats.db_bytes)}" } }
                                        div { style: "display:flex; justify-content:space-between; padding:4px 0;", span { "Files (iroh-blobs)" } span { "{human_bytes(stats.blobs_bytes)}" } }
                                        div { style: "display:flex; justify-content:space-between; padding:4px 0;", span { "Room/DM metadata (iroh-docs)" } span { "{human_bytes(stats.docs_bytes)}" } }
                                        div { style: "display:flex; justify-content:space-between; padding:4px 0; font-weight:600; border-top: 1px solid var(--border); margin-top:6px; padding-top:6px;",
                                            span { "Total" } span { "{human_bytes(stats.total())}" }
                                        }
                                    }
                                }
                                button {
                                    class: "secondary",
                                    style: "margin-top:14px;",
                                    onclick: move |_| on_refresh_storage.call(()),
                                    "Recalculate"
                                }
                                p { class: "settings-section-label", style: "margin-top:24px;", "Encrypted backup" }
                                p { style: "font-size:12px; color:var(--text-muted); margin:0 0 12px 0;",
                                    "Encrypts your recovery phrase, messages, and files into one file, protected by "
                                    "a backup passphrase you choose here — separate from your 24-word recovery phrase. "
                                    "Save it anywhere, including a folder your cloud drive syncs, for an off-device copy."
                                }
                                textarea {
                                    rows: "3",
                                    placeholder: "Your 24-word recovery phrase, space-separated",
                                    value: "{ui.backup_seed_input}",
                                    oninput: move |e| ui.backup_seed_input.clone().set(e.value()),
                                }
                                input {
                                    r#type: "password",
                                    style: "margin-top:8px; width:100%; box-sizing:border-box; background: var(--surface); \
                                            border: 1px solid var(--border); color: var(--text); border-radius:8px; padding:6px 10px;",
                                    placeholder: "Choose a backup passphrase (8+ characters)",
                                    value: "{ui.backup_passphrase_input}",
                                    oninput: move |e| ui.backup_passphrase_input.clone().set(e.value()),
                                }
                                if let Some(err) = ui.backup_error.read().clone() {
                                    p { style: "font-size:12px; color:var(--danger); margin-top:8px;", "{err}" }
                                }
                                button {
                                    style: "margin-top:10px;",
                                    disabled: ui.backup_busy.cloned()
                                        || ui.backup_seed_input.read().trim().is_empty()
                                        || ui.backup_passphrase_input.read().len() < 8,
                                    onclick: move |_| spawn_create_backup(ui),
                                    if ui.backup_busy.cloned() { "Encrypting…" } else { "Create backup file" }
                                }
                            }
                        },
                        SettingsTab::About => rsx! {
                            div {
                                p { style: "margin:0 0 4px 0; font-weight:600;", "Siar" }
                                p { style: "font-size:12px; color:var(--text-muted); margin:0 0 16px 0;",
                                    "v{env!(\"CARGO_PKG_VERSION\")} — by Irshad"
                                }
                                p { style: "font-size:12px; color:var(--text-muted); margin:0 0 12px 0;",
                                    "A peer-to-peer, serverless messenger built on iroh (QUIC). There's no \
                                     central server: your identity is a seed phrase, contacts connect \
                                     directly (or via a public relay when a direct path isn't reachable), \
                                     and message history lives only on your own devices."
                                }
                                div { style: "font-size:12px; color:var(--text-muted); line-height:1.8;",
                                    div { "Networking: iroh (QUIC transport, iroh-gossip, iroh-docs, iroh-blobs)" }
                                    div { "Offline fallback: Bluetooth LE + local Wi-Fi mesh relay (net::mesh), off by default — see Network tab" }
                                    div { "UI: Dioxus" }
                                    div { "Storage: SQLite (rusqlite)" }
                                    div { "License: MIT OR Apache-2.0" }
                                }
                            }
                        },
                    }}
                }
                div { style: "padding: 0 24px 20px 24px; text-align:right;",
                    button { class: "secondary", onclick: move |_| on_close.call(()), "Close" }
                }
            }
        }
    }
}

/// A labeled on/off row used across Settings' Notifications/Privacy tabs.
/// Reads as a plain button-styled switch rather than a native `<input
/// type="checkbox">` — matches every other control in this panel
/// (buttons throughout, no native form widgets), and sidesteps needing
/// checkbox-specific CSS just for these few rows.
#[component]
fn SettingsToggle(
    label: String,
    description: String,
    checked: bool,
    onchange: EventHandler<bool>,
) -> Element {
    rsx! {
        div { class: "switch-row",
            div { style: "flex:1;",
                div { class: "switch-label", "{label}" }
                p { class: "switch-desc", "{description}" }
            }
            div {
                class: if checked { "switch on" } else { "switch" },
                role: "switch",
                "aria-checked": if checked { "true" } else { "false" },
                onclick: move |_| onchange.call(!checked),
                div { class: "knob" }
            }
        }
    }
}

/// Custom in-window title bar: drag region, minimize/maximize/close.
/// Only meaningful — and only compiled — when this build's `dioxus`
/// actually has its `desktop` (wry/tao webview) feature enabled; see
/// the `desktop-chrome` feature in this crate's `Cargo.toml` for why
/// that's not simply `target_os`-gated the way clipboard/notifications
/// are. Renders nothing under any other renderer/platform combination.
///
/// # Known limitation: window edge/corner drag-to-resize
///
/// Disabling OS decorations (`WindowBuilder::with_decorations(false)`,
/// set in `siar-desktop/src/main.rs`) is what makes a custom title bar
/// possible at all, but it also removes the OS's own resize handles —
/// and as of this writing, Dioxus doesn't expose an API to bring
/// drag-resizing back on top of an undecorated window (open upstream:
/// DioxusLabs/dioxus#3128, checked via search, not assumed). `.
/// with_resizable(true)` is still set in `main.rs`, so programmatic/
/// keyboard/taskbar-menu resize still works — what's actually lost is
/// only the everyday "grab the edge and drag" gesture.
///
/// This is a real, currently-open trade-off, not a bug in this
/// implementation — flagging it here plainly rather than shipping it
/// silently. If it turns out to matter more than the custom chrome is
/// worth, the fix is one line: remove `.with_decorations(false)` in
/// `main.rs` (or don't set `siar-ui`'s `desktop-chrome` feature) to get
/// the OS's native title bar and full resize behavior back, at the
/// cost of the custom look.
#[cfg(feature = "desktop-chrome")]
#[component]
fn TitleBar() -> Element {
    let window = dioxus::desktop::window();
    let w1 = window.clone();
    let w2 = window.clone();
    let w3 = window.clone();
    rsx! {
        div {
            class: "custom-titlebar",
            // The bar itself is the drag region — except the button
            // cluster, which stops propagation below so a click on a
            // button doesn't also start a drag. Matches the pattern
            // from DioxusLabs/dioxus#532 (a real, working example of
            // this exact custom-titlebar setup).
            onmousedown: move |_| w1.drag(),
            span { class: "custom-titlebar-title", "Siar" }
            div {
                class: "custom-titlebar-buttons",
                onmousedown: move |e| e.stop_propagation(),
                button {
                    class: "custom-titlebar-btn",
                    onclick: move |_| w2.set_minimized(true),
                    "–"
                }
                button {
                    class: "custom-titlebar-btn",
                    onclick: move |_| w3.toggle_maximized(),
                    "▢"
                }
                button {
                    class: "custom-titlebar-btn close",
                    onclick: move |_| window.close(),
                    "✕"
                }
            }
        }
    }
}

#[cfg(not(feature = "desktop-chrome"))]
#[component]
fn TitleBar() -> Element {
    rsx! {}
}

/// Renders `ui.context_menu`, if any — a small floating action list at
/// the click position, dismissed by clicking the invisible full-screen
/// backdrop behind it (same pattern `SettingsPanel` already uses) or by
/// picking an item. Mounted once, near the top of `AppRoot`'s tree, so
/// it can float above literally everything else regardless of which
/// screen/panel is currently showing.
#[component]
fn ContextMenu(ui: UiState) -> Element {
    let Some(state) = ui.context_menu.read().clone() else {
        return rsx! {};
    };
    let x = state.x;
    let y = state.y;
    rsx! {
        div {
            class: "context-menu-backdrop",
            style: "position:fixed; inset:0; z-index:60;",
            onclick: move |_| ui.context_menu.clone().set(None),
            oncontextmenu: move |e| { e.prevent_default(); ui.context_menu.clone().set(None); },
            div {
                class: "context-menu",
                style: "position:fixed; left:{x}px; top:{y}px; min-width:160px; background:var(--bg-elevated); \
                        border:1px solid var(--border); border-radius:8px; box-shadow:0 8px 24px rgba(0,0,0,0.35); \
                        padding:4px; z-index:61;",
                onclick: move |e| e.stop_propagation(),
                {match state.kind.clone() {
                    ContextMenuKind::Bubble { target_id, is_own, deleted, sender_label, snippet } => {
                        // Each closure below needs its own owned copy —
                        // `snippet`/`sender_label` are `String`s (not
                        // `Copy`), and three separate `move` closures
                        // each trying to capture the *same* outer
                        // binding is exactly one move too many for the
                        // borrow checker; the first `move` closure that
                        // gets built takes ownership, and the next one
                        // fails to compile. Cloning once per use site,
                        // here, before any closure exists, sidesteps
                        // that entirely.
                        let reply_sender = sender_label.clone();
                        let reply_snippet = snippet.clone();
                        let copy_snippet = snippet.clone();
                        let edit_snippet = snippet.clone();
                        rsx! {
                        for emoji in ["👍", "❤️", "😂", "😮"] {
                            ContextMenuItem {
                                label: emoji.to_string(),
                                onclick: move |_| {
                                    if let Some(key) = ui.active.read().clone() {
                                        spawn_send_reaction(ui, key, target_id, emoji.to_string());
                                    }
                                    ui.context_menu.clone().set(None);
                                },
                            }
                        }
                        if !deleted {
                            ContextMenuItem {
                                label: "Reply".to_string(),
                                onclick: move |_| {
                                    ui.replying_to.clone().set(Some((target_id, reply_sender.clone(), reply_snippet.clone())));
                                    ui.context_menu.clone().set(None);
                                },
                            }
                        }
                        ContextMenuItem {
                            label: "Copy".to_string(),
                            onclick: move |_| {
                                copy_to_clipboard(copy_snippet.clone());
                                ui.context_menu.clone().set(None);
                            },
                        }
                        if is_own && !deleted {
                            ContextMenuItem {
                                label: "Edit".to_string(),
                                onclick: move |_| {
                                    ui.editing.clone().set(Some(target_id));
                                    ui.compose.clone().set(edit_snippet.clone());
                                    ui.context_menu.clone().set(None);
                                },
                            }
                            ContextMenuItem {
                                label: "Delete".to_string(),
                                danger: true,
                                onclick: move |_| {
                                    if let Some(key) = ui.active.read().clone() {
                                        spawn_send_delete(ui, key, target_id);
                                    }
                                    ui.context_menu.clone().set(None);
                                },
                            }
                        }
                        }
                    },
                    ContextMenuKind::ChatRow { key, pinned, archived } => rsx! {
                        if let ConvKey::Dm(peer) = key.clone() {
                            ContextMenuItem {
                                label: if pinned { "Unpin".to_string() } else { "Pin".to_string() },
                                onclick: move |_| {
                                    spawn_set_dm_pinned(ui, peer, !pinned);
                                    ui.context_menu.clone().set(None);
                                },
                            }
                            ContextMenuItem {
                                label: if archived { "Unarchive".to_string() } else { "Archive".to_string() },
                                onclick: move |_| {
                                    spawn_set_dm_archived(ui, peer, !archived);
                                    ui.context_menu.clone().set(None);
                                },
                            }
                        }
                    },
                }}
            }
        }
    }
}

#[component]
fn ContextMenuItem(
    label: String,
    onclick: EventHandler<()>,
    #[props(default = false)] danger: bool,
) -> Element {
    rsx! {
        div {
            class: if danger { "context-menu-item danger" } else { "context-menu-item" },
            onclick: move |_| onclick.call(()),
            "{label}"
        }
    }
}

fn spawn_load_storage_stats(ui: UiState) {
    let data_dir = CONFIG.get().unwrap().data_dir.clone();
    ui.storage_loading.clone().set(true);
    spawn(async move {
        let dir = data_dir.clone();
        let stats = tokio::task::spawn_blocking(move || StorageStats {
            db_bytes: dir_size(&dir.join("messenger.db")),
            blobs_bytes: dir_size(&dir.join("blobs")),
            docs_bytes: dir_size(&dir.join("docs")),
        })
        .await
        .unwrap_or_default();
        ui.storage_stats.clone().set(Some(stats));
        ui.storage_loading.clone().set(false);
    });
}

/// Recursive, best-effort file/directory size. Used only on demand (the
/// Settings panel's Storage tab, via `spawn_load_storage_stats`) since
/// it's a real filesystem walk — never called on a hot path or every
/// render.
fn dir_size(path: &std::path::Path) -> u64 {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if meta.is_file() {
        return meta.len();
    }
    if !meta.is_dir() {
        return 0;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries.flatten().map(|entry| dir_size(&entry.path())).sum()
}

fn render_qr_svg(data: &str) -> String {
    use qrcode::render::svg;
    use qrcode::QrCode;
    match QrCode::new(data.as_bytes()) {
        Ok(code) => code
            .render()
            .min_dimensions(220, 220)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build(),
        Err(_) => String::new(),
    }
}

pub(crate) fn copy_to_clipboard(text: String) {
    #[cfg(target_os = "linux")]
    {
        // Wayland's data-control protocol directly — no X11/XWayland
        // involved at all, unlike arboard's Linux backend (which goes
        // through `x11rb` even under a Wayland session, relying on
        // XWayland to translate). `wl-clipboard-rs` solves the same
        // "something has to stay alive to answer paste requests" problem
        // arboard's `SetExtLinux`/`.wait()` dance worked around, but
        // natively: `copy()` forks its own small background process to
        // keep serving the clipboard, so there's no thread to manage here
        // and nothing to clean up.
        use wl_clipboard_rs::copy::{MimeType, Options, Source};
        let _ = Options::new().copy(Source::Bytes(text.into_bytes().into()), MimeType::Text);
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "ios")))]
    {
        // Windows/macOS: the system clipboard is OS-managed and outlives
        // the setting process, so a short-lived handle is fine here.
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        // Dioxus mobile is hosted in the platform WebView. Use its system
        // clipboard API first, with the older selection/copy command as a
        // fallback for Android WebViews that deny Clipboard.writeText.
        // Base64 keeps arbitrary ticket/text bytes out of JavaScript source.
        let encoded = data_encoding::BASE64.encode(text.as_bytes());
        spawn(async move {
            let script = format!(
                r#"
                const raw = atob('{encoded}');
                const bytes = Uint8Array.from(raw, c => c.charCodeAt(0));
                const value = new TextDecoder().decode(bytes);
                let copied = false;
                try {{
                    await navigator.clipboard.writeText(value);
                    copied = true;
                }} catch (_) {{}}
                if (!copied) {{
                    const area = document.createElement('textarea');
                    area.value = value;
                    area.setAttribute('readonly', '');
                    area.style.position = 'fixed';
                    area.style.opacity = '0';
                    document.body.appendChild(area);
                    area.select();
                    area.setSelectionRange(0, area.value.length);
                    copied = document.execCommand('copy');
                    area.remove();
                }}
                return copied;
                "#
            );
            let _ = document::eval(&script).await;
        });
    }
}

fn set_android_background_wake(enabled: bool) {
    #[cfg(target_os = "android")]
    spawn(async move {
        let script = format!(
            "window.SiarAndroid?.setBackgroundWake({})",
            if enabled { "true" } else { "false" }
        );
        let _ = document::eval(&script).await;
    });

    #[cfg(not(target_os = "android"))]
    let _ = enabled;
}

fn request_android_nearby_permissions() {
    #[cfg(target_os = "android")]
    spawn(async move {
        let _ = document::eval("window.SiarAndroid?.requestNearbyPermissions()").await;
    });
}

fn request_android_notification_permission() {
    #[cfg(target_os = "android")]
    spawn(async move {
        let _ = document::eval("window.SiarAndroid?.requestNotificationPermission()").await;
    });
}

fn incoming_notification_preview(body: &Body) -> String {
    match body {
        Body::Text { text, .. } => text.clone(),
        Body::File { name, .. } => format!("📎 {name}"),
        _ => "New activity".to_string(),
    }
}

/// Android system notification for activity received while Siar is not
/// visible. The native bridge suppresses this when the Activity has focus,
/// so the in-chat UI remains the only foreground signal.
fn notify_android_message(ui: UiState, title: &str, body: String) {
    #[cfg(target_os = "android")]
    {
        let (enabled, play_sound) = ui
            .core
            .read()
            .as_ref()
            .map(|core| {
                (
                    core.store().notifications_enabled(),
                    core.store().notification_sound_enabled(),
                )
            })
            .unwrap_or((false, false));
        if !enabled {
            return;
        }
        let title = title.to_string();
        spawn(async move {
            let script =
                format!("window.SiarAndroid?.showNotification({title:?}, {body:?}, {play_sound})");
            let _ = document::eval(&script).await;
        });
    }

    #[cfg(not(target_os = "android"))]
    let _ = (ui, title, body);
}

// ---- Boot / identity ----

/// Boots `App`/`Core` and starts draining its event channel. Runs on
/// every cold start and (via the Retry button — see `AppRoot`'s
/// `Screen::Main` branch) after a failed one.
///
/// `App::start` now bounds every one of its own network-dependent steps
/// individually (endpoint bind, `iroh-docs` engine spin-up, username
/// registry sync — each with its own named error, same pattern as
/// `protocol::dm::NET_TIMEOUT` etc.) rather than relying solely on this
/// outer wrapper the way an earlier version did. `BOOT_TIMEOUT` is now a
/// true last-resort safety net — sized comfortably above the worst-case
/// *sum* of those inner timeouts (bind + docs + registry's own two
/// attempts + the relay wait) so it doesn't race ahead of them and mask
/// their more specific error messages with a generic "didn't finish"— and
/// only fires for something those inner bounds didn't anticipate (a
/// genuine deadlock, not just "offline").
const BOOT_TIMEOUT: Duration = Duration::from_secs(150);

fn spawn_boot(ui: UiState, onboarding: Option<OnboardingResult>) {
    let data_dir = CONFIG.get().unwrap().data_dir.clone();
    let relay_timeout = Duration::from_secs(CONFIG.get().unwrap().relay_timeout_secs);

    spawn(async move {
        let (secret_key, username, display_name) = match onboarding {
            Some(result) => {
                let OnboardingResult {
                    seed,
                    username,
                    display_name,
                } = result;
                let dir = data_dir.clone();
                let sk =
                    match tokio::task::spawn_blocking(move || app::create_identity(&dir, &seed))
                        .await
                    {
                        Ok(Ok(v)) => v,
                        Ok(Err(e)) => {
                            let msg = format!("identity creation failed: {e}");
                            ui.boot_error.clone().set(Some(msg.clone()));
                            return push_toast(ui, msg, true);
                        }
                        Err(e) => {
                            let msg = format!("identity creation task failed: {e}");
                            ui.boot_error.clone().set(Some(msg.clone()));
                            return push_toast(ui, msg, true);
                        }
                    };
                (sk, Some(username), display_name)
            }
            None => {
                let dir = data_dir.clone();
                let res = match tokio::task::spawn_blocking(move || app::load_identity(&dir)).await
                {
                    Ok(res) => res,
                    Err(e) => Err(anyhow::anyhow!("join error: {e}")),
                };
                match res {
                    Ok(Some(sk)) => (sk, None, whoami_fallback()),
                    Ok(None) => return, // shouldn't happen — screen routing already checked `exists`
                    Err(e) => {
                        let msg = format!("failed loading identity: {e}");
                        ui.boot_error.clone().set(Some(msg.clone()));
                        return push_toast(ui, msg, true);
                    }
                }
            }
        };

        tracing::info!("spawn_boot: starting Core::start");
        let start_result = tokio::time::timeout(
            BOOT_TIMEOUT,
            Core::start(data_dir, secret_key, display_name, relay_timeout),
        )
        .await;

        match start_result {
            Err(_elapsed) => {
                let msg = format!(
                    "startup didn't finish within {}s — this is past every individual step's own \
                     timeout (endpoint bind, sync engine, registry, relay wait), so something \
                     unexpected is stuck; see logs (RUST_LOG=info) for which step",
                    BOOT_TIMEOUT.as_secs()
                );
                tracing::warn!("{msg}");
                ui.boot_error.clone().set(Some(msg.clone()));
                push_toast(ui, msg, true);
            }
            Ok(Ok((mut core, mut app_rx))) => {
                tracing::info!("spawn_boot: Core::start returned");
                if let Some(name) = username {
                    match core.claim_username(&name).await {
                        Ok(siar_core::net::registry::ClaimOutcome::Claimed) => {}
                        Ok(siar_core::net::registry::ClaimOutcome::TakenBy(record)) => {
                            let taken_by = EndpointId::from_bytes(&record.endpoint_id)
                                .map(app::hex)
                                .unwrap_or_else(|_| "someone else".to_string());
                            push_toast(
                                ui,
                                format!("@{name} was just claimed by {taken_by} — pick another in settings"),
                                true,
                            );
                        }
                        Err(e) => push_toast(ui, format!("username claim failed: {e}"), true),
                    }
                }

                ui.relay_ok.clone().set(core.relay_ok());
                refresh_contacts(ui, &core);
                if let Some(hash) = core.my_avatar_hash() {
                    ui.my_avatar_hash.clone().set(Some(hash.clone()));
                    if let Ok(Some(path)) = core.store().cached_download_path(&hash) {
                        if let Ok(bytes) = std::fs::read(&path) {
                            load_avatar_into_cache(ui, &hash, &bytes);
                        }
                    }
                }
                ui.boot_error.clone().set(None);
                ui.theme_mode.clone().set(core.store().theme_mode());
                ui.theme_style.clone().set(core.store().theme_style());
                ui.core.clone().set(Some(core));
                refresh_statuses(ui);
                refresh_call_log(ui);
                spawn_preload_dm_settings(ui);
                spawn_shutdown_on_ctrl_c(ui);
                spawn_disappearing_sweep(ui);
                spawn_dm_keepalive(ui);

                while let Some(event) = app_rx.recv().await {
                    handle_app_event(ui, event);
                }
            }
            Ok(Err(e)) => {
                let msg = format!("startup failed: {e}");
                tracing::warn!("{msg}");
                ui.boot_error.clone().set(Some(msg.clone()));
                push_toast(ui, msg, true);
            }
        }
    });
}

/// Warm `ui.dm_settings_cache` for every currently-accepted contact, so the
/// sidebar's pin-sort/archive-filter reflects reality from the moment the
/// chat list first renders rather than only after each DM's info panel has
/// been opened once. Called after boot and after accepting a new contact
/// (see call sites) — cheap to call repeatedly since `DmDoc::open` just
/// reattaches to an already-synced local namespace after the first time.
fn spawn_preload_dm_settings(ui: UiState) {
    let contacts = ui.contacts.read().clone();
    let handles = {
        let core_ref = ui.core.read();
        core_ref.as_ref().map(|core| {
            (
                core.docs(),
                core.blobs_store(),
                core.docs_author(),
                core.my_id,
            )
        })
    };
    let Some((docs, blobs_store, docs_author, my_id)) = handles else {
        return;
    };
    for c in contacts {
        let Ok(peer) = parse_hex(&c.endpoint_id) else {
            continue;
        };
        let docs = docs.clone();
        let blobs_store = blobs_store.clone();
        spawn(async move {
            match siar_core::net::conv_docs::DmDoc::open(
                &docs,
                blobs_store,
                docs_author,
                my_id,
                peer,
            )
            .await
            {
                Ok(dm_doc) => {
                    if let Ok(settings) = dm_doc.settings().await {
                        ui.dm_settings_cache.clone().write().insert(peer, settings);
                    }
                    if let Some(core) = ui.core.clone().write().as_mut() {
                        core.commit_dm_doc(peer, dm_doc);
                    }
                }
                Err(e) => {
                    tracing::warn!(peer = %app::hex(peer), error = %e, "couldn't preload DM settings")
                }
            }
        });
    }
}

/// Best-effort graceful teardown on Ctrl+C: close the protocol router and
/// iroh endpoint (`App::shutdown`) rather than just letting the process
/// die mid-connection. Registered once per successful boot. This doesn't
/// cover the desktop window's own close ("X") button — Dioxus desktop's
/// window-close-hook API wasn't confirmed for the pinned version, so
/// wiring that in is left as a follow-up rather than guessed at; Ctrl+C
/// (running from a terminal) is the one shutdown path this covers today.
fn spawn_shutdown_on_ctrl_c(ui: UiState) {
    spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("ctrl-c received, shutting down");
            if let Some(core) = ui.core.clone().write().take() {
                core.shutdown().await;
            }
            std::process::exit(0);
        }
    });
}

/// Periodic disappearing-messages sweep — the "physically delete" half of
/// the mechanism described in the spec's worked example (§12): the
/// sqlite-read-path filter (`Store::recent_messages`) and the render-time
/// filter (`bubbles_for`) both already guarantee an expired message never
/// *displays*, so this loop is what actually reclaims the space and keeps
/// the two in-memory/on-disk copies from silently drifting apart forever.
/// A periodic sweep rather than one `tokio::time::sleep_until` per message
/// is deliberately more robust across app restarts — a scheduled one-shot
/// timer for a message sent right before the app closed would simply never
/// fire, whereas a sweep just catches up next time it runs.
const DISAPPEARING_SWEEP_INTERVAL: Duration = Duration::from_secs(30);

fn spawn_disappearing_sweep(ui: UiState) {
    let mut warned_username_conflict = false;
    spawn(async move {
        loop {
            tokio::time::sleep(DISAPPEARING_SWEEP_INTERVAL).await;
            let (deleted, statuses_pruned) = {
                let core_ref = ui.core.read();
                match core_ref.as_ref() {
                    Some(core) => (
                        core.sweep_expired_messages().ok(),
                        core.prune_expired_statuses().ok(),
                    ),
                    None => (None, None),
                }
            };
            if !warned_username_conflict {
                let still_valid = {
                    let core_ref = ui.core.read();
                    match core_ref.as_ref() {
                        Some(core) => core.username_still_valid().await.unwrap_or(true),
                        None => true,
                    }
                };
                if !still_valid {
                    warned_username_conflict = true;
                    push_toast(
                        ui,
                        "your username is now claimed by another device — pick a new one to be found reliably".to_string(),
                        true,
                    );
                }
            }
            if matches!(deleted, Some(n) if n > 0) {
                // Sqlite side is authoritative and already done; also drop
                // the same rows from the in-memory cache so a
                // conversation that's open right now updates immediately
                // instead of waiting for its next full history reload.
                // (`bubbles_for` already filters these at render time too
                // — this is just keeping the cache itself tidy, not a
                // correctness requirement.)
                let now = now_ms();
                ui.conversations
                    .clone()
                    .write()
                    .values_mut()
                    .for_each(|bubbles| {
                        bubbles.retain(|b| b.expires_at_unix_ms.is_none_or(|exp| exp > now));
                    });
            }
            if matches!(statuses_pruned, Some(n) if n > 0) {
                refresh_statuses(ui);
            }
        }
    });
}

/// How often to ping every open DM session. Short enough to keep NAT/relay
/// paths warm through a quiet chat window and to notice a dead session
/// well before the user's next real message hits it; long enough not to
/// be a meaningful chunk of traffic for an idle app. Not tied to any QUIC-
/// level idle timeout — this is a deliberately simple, protocol-agnostic
/// keepalive at the application layer, so it doesn't depend on tuning
/// iroh/quinn transport config correctly (see module notes elsewhere in
/// this codebase about not guessing exact iroh API surface without a
/// compiler on hand).
const DM_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(20);

/// Periodic keepalive across every open DM session (see
/// `App::dm_sessions_snapshot`). Two things this buys, both squarely
/// aimed at the "why did sending need three retries" class of complaint:
///
///   1. A session sitting idle for a while (user reading, not typing) is
///      exercised regularly instead of going fully cold — direct
///      (hole-punched) paths in particular can need occasional traffic to
///      stay mapped through some NATs.
///   2. A session that's actually died (peer closed the app, network
///      dropped) gets *caught* here — and the local session dropped, so
///      the next real send transparently reconnects via
///      `connect_with_retry` — instead of the user finding out only when
///      their own message fails.
///
/// Deliberately reuses `DmSession::send`, the exact same path
/// `spawn_typing` below already exercises — no new networking surface,
/// just a new caller of code that's already proven to compile and work.
fn spawn_dm_keepalive(ui: UiState) {
    spawn(async move {
        loop {
            tokio::time::sleep(DM_KEEPALIVE_INTERVAL).await;
            let (sessions, my_name) = {
                let core_ref = ui.core.read();
                let Some(core) = core_ref.as_ref() else {
                    continue;
                };
                (core.dm_sessions_snapshot(), core.my_name.clone())
            };
            for (peer, session) in sessions {
                if session
                    .send(&Envelope::ping(my_name.clone()))
                    .await
                    .is_err()
                {
                    tracing::debug!(peer = %app::hex(peer), "dm keepalive failed — dropping stale session");
                    if let Some(core) = ui.core.clone().write().as_mut() {
                        core.drop_dm_session(peer);
                    }
                    ui.online.clone().write().remove(&peer);
                }
            }
        }
    });
}

fn whoami_fallback() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "me".to_string())
}

// ---- Event handling ----

fn handle_app_event(ui: UiState, event: app::AppEvent) {
    match event {
        app::AppEvent::Dm(DmEvent::Received { from, envelope }) => {
            match &envelope.body {
                Body::Text { .. } | Body::File { .. } => {
                    // Persist first — `record_incoming` below only updates
                    // the in-memory `ui.conversations` cache for instant
                    // render; without this, received messages looked fine
                    // until the app restarted and history reloaded from
                    // sqlite without them. See `App::record_incoming_dm`.
                    if let Some(core) = ui.core.read().as_ref() {
                        if let Err(e) = core.record_incoming_dm(from, &envelope) {
                            tracing::warn!(peer = %app::hex(from), error = %e, "failed to persist incoming DM");
                        }
                    }
                    record_incoming(ui, ConvKey::Dm(from), &envelope, Some(from));
                    bump_unread(ui, &ConvKey::Dm(from));
                    notify_android_message(
                        ui,
                        &envelope.from_name,
                        incoming_notification_preview(&envelope.body),
                    );
                    ui.typing.clone().write().remove(&ConvKey::Dm(from));
                    spawn_send_ack(ui, from, envelope.id);
                    // Archiving hides a chat from the main list, but it
                    // isn't meant to silently swallow new activity forever
                    // — WhatsApp/Signal both resurface an archived chat
                    // when something new comes in (unless the person has
                    // explicitly opted to keep it archived, a setting this
                    // app doesn't have yet, so the default here is the
                    // safe one: always resurface).
                    if ui
                        .dm_settings_cache
                        .read()
                        .get(&from)
                        .is_some_and(|s| s.archived)
                    {
                        spawn_set_dm_archived(ui, from, false);
                    }
                }
                Body::Typing => {
                    let key = ConvKey::Dm(from);
                    ui.typing.clone().write().insert(key.clone(), now_ms());
                    spawn(async move {
                        tokio::time::sleep(TYPING_TIMEOUT).await;
                        let stale = ui
                            .typing
                            .read()
                            .get(&key)
                            .is_some_and(|t| now_ms() - t >= TYPING_TIMEOUT.as_millis() as i64);
                        if stale {
                            ui.typing.clone().write().remove(&key);
                        }
                    });
                }
                Body::Ack(acked_id) => mark_acked(ui, &ConvKey::Dm(from), *acked_id),
                Body::Hello => {}
                // Keepalive only — see `ui::spawn_dm_keepalive`. Nothing to
                // show; receiving it at all is the only signal that
                // matters, and that's implicit (this whole match only
                // runs because the connection is alive and delivering).
                Body::Ping => {}
                Body::Status {
                    text,
                    image,
                    video,
                    audio,
                } => {
                    let (peer_name, expires_at_ms) =
                        (envelope.from_name.clone(), envelope.expires_at_unix_ms);
                    if let Some(core) = ui.core.read().as_ref() {
                        if let Err(e) = core.record_incoming_status(
                            from,
                            &peer_name,
                            text,
                            image.clone(),
                            video.clone(),
                            audio.clone(),
                            expires_at_ms,
                        ) {
                            tracing::warn!(peer = %app::hex(from), error = %e, "failed to persist incoming status");
                        }
                    }
                    if let Some(img) = image.clone() {
                        spawn_fetch_status_image(ui, from, img.blake3_hash);
                    }
                    if let Some(vid) = video.clone() {
                        spawn_fetch_status_video(ui, from, vid.blake3_hash);
                    }
                    if let Some(aud) = audio.clone() {
                        spawn_fetch_status_audio(ui, from, aud.blake3_hash);
                    }
                    refresh_statuses(ui);
                }
                Body::AvatarUpdate { blake3_hash, .. } => {
                    if let Some(core) = ui.core.read().as_ref() {
                        if let Err(e) = core.record_incoming_avatar_hash(from, blake3_hash) {
                            tracing::warn!(peer = %app::hex(from), error = %e, "failed to persist incoming avatar hash");
                        }
                        refresh_contacts(ui, core);
                    }
                    // Prefetch now rather than waiting for the UI to need
                    // it — an update just arrived, so the peer's very
                    // likely still reachable; not eager for every
                    // contact's avatar on every launch, just this one
                    // change. See `App::fetch_contact_avatar`'s doc for
                    // the cache path this populates.
                    let hash = blake3_hash.clone();
                    spawn(async move {
                        let (store, blobs, endpoint) = {
                            let core_ref = ui.core.read();
                            let Some(core) = core_ref.as_ref() else {
                                return;
                            };
                            (core.store(), core.blobs(), core.endpoint())
                        };
                        let cache_dir = CONFIG.get().unwrap().data_dir.join("avatar_cache");
                        match app::App::fetch_contact_avatar(
                            &store, &blobs, &endpoint, from, &hash, &cache_dir,
                        )
                        .await
                        {
                            Ok(bytes) => load_avatar_into_cache(ui, &hash, &bytes),
                            Err(e) => {
                                tracing::debug!(peer = %app::hex(from), error = %e, "avatar prefetch failed, will retry on next open")
                            }
                        }
                    });
                }
                Body::Reaction {
                    target_id,
                    emoji,
                    remove,
                } => {
                    if let Some(core) = ui.core.read().as_ref() {
                        if let Err(e) = core.record_incoming_dm(from, &envelope) {
                            tracing::warn!(peer = %app::hex(from), error = %e, "failed to persist incoming reaction");
                        }
                    }
                    apply_reaction_locally(
                        ui,
                        &ConvKey::Dm(from),
                        *target_id,
                        app::hex(from),
                        emoji.clone(),
                        *remove,
                    );
                }
                Body::Edit {
                    target_id,
                    new_text,
                } => {
                    if let Some(core) = ui.core.read().as_ref() {
                        if let Err(e) = core.record_incoming_dm(from, &envelope) {
                            tracing::warn!(peer = %app::hex(from), error = %e, "failed to persist incoming edit");
                        }
                    }
                    apply_edit_locally(ui, &ConvKey::Dm(from), *target_id, new_text.clone());
                }
                Body::Delete { target_id } => {
                    if let Some(core) = ui.core.read().as_ref() {
                        if let Err(e) = core.record_incoming_dm(from, &envelope) {
                            tracing::warn!(peer = %app::hex(from), error = %e, "failed to persist incoming delete");
                        }
                    }
                    apply_delete_locally(ui, &ConvKey::Dm(from), *target_id);
                }
                Body::Read { up_to_sent_unix_ms } => {
                    if let Some(core) = ui.core.read().as_ref() {
                        if let Err(e) = core.record_incoming_dm(from, &envelope) {
                            tracing::warn!(peer = %app::hex(from), error = %e, "failed to persist read receipt");
                        }
                    }
                    ui.read_watermarks
                        .clone()
                        .write()
                        .insert(ConvKey::Dm(from), *up_to_sent_unix_ms as i64);
                }
            }
        }
        app::AppEvent::Dm(DmEvent::PeerConnected { from }) => {
            ui.online.clone().write().insert(from);
        }
        app::AppEvent::Dm(DmEvent::PeerDisconnected { from }) => {
            ui.online.clone().write().remove(&from);
        }
        app::AppEvent::Room(RoomEvent::Received {
            room,
            from,
            envelope,
        }) => {
            match &envelope.body {
                Body::Text { .. } | Body::File { .. } => {
                    // Persist first — see the matching comment on the DM
                    // arm above; `App::record_incoming_room` already
                    // no-ops on our own gossip echo.
                    if let Some(core) = ui.core.read().as_ref() {
                        if let Err(e) = core.record_incoming_room(&room, from, &envelope) {
                            tracing::warn!(room = %room, error = %e, "failed to persist incoming room message");
                        }
                    }
                    record_incoming(ui, ConvKey::Room(room.clone()), &envelope, Some(from));
                    bump_unread(ui, &ConvKey::Room(room.clone()));
                    notify_android_message(
                        ui,
                        &format!("#{} · {}", room, envelope.from_name),
                        incoming_notification_preview(&envelope.body),
                    );
                }
                Body::Reaction {
                    target_id,
                    emoji,
                    remove,
                } => {
                    if let Some(core) = ui.core.read().as_ref() {
                        if let Err(e) = core.record_incoming_room(&room, from, &envelope) {
                            tracing::warn!(room = %room, error = %e, "failed to persist incoming room reaction");
                        }
                    }
                    apply_reaction_locally(
                        ui,
                        &ConvKey::Room(room.clone()),
                        *target_id,
                        app::hex(from),
                        emoji.clone(),
                        *remove,
                    );
                }
                Body::Edit {
                    target_id,
                    new_text,
                } => {
                    if let Some(core) = ui.core.read().as_ref() {
                        if let Err(e) = core.record_incoming_room(&room, from, &envelope) {
                            tracing::warn!(room = %room, error = %e, "failed to persist incoming room edit");
                        }
                    }
                    apply_edit_locally(
                        ui,
                        &ConvKey::Room(room.clone()),
                        *target_id,
                        new_text.clone(),
                    );
                }
                Body::Delete { target_id } => {
                    if let Some(core) = ui.core.read().as_ref() {
                        if let Err(e) = core.record_incoming_room(&room, from, &envelope) {
                            tracing::warn!(room = %room, error = %e, "failed to persist incoming room delete");
                        }
                    }
                    apply_delete_locally(ui, &ConvKey::Room(room.clone()), *target_id);
                }
                // Everything else (Ack/Typing/Hello/Ping/Status/AvatarUpdate/
                // Read) either doesn't apply to rooms or isn't sent over
                // gossip in the first place.
                _ => {}
            }
        }
        // Room presence — previously ignored entirely (`RoomEvent(_) => {}`).
        // Surfaced two ways: `online` gets a room peer added/removed so
        // presence indicators elsewhere in the UI can use it, and a system
        // bubble ("so-and-so joined/left") is dropped into that room's
        // history, WhatsApp/Signal-style. `Lagged` gets a toast instead of
        // a bubble since it's about *this* device, not a specific peer —
        // and it's the honest, visible flag for the offline-message-loss
        // gap ARCHITECTURE.md §11 already documents (gossip has no
        // catch-up; `net::conv_docs`'s durable metadata doesn't cover
        // message content).
        app::AppEvent::Room(RoomEvent::NeighborUp { room, peer }) => {
            ui.online.clone().write().insert(peer);
            push_system_bubble(
                ui,
                &room,
                format!("{} joined #{room}", short_peer_label(ui, peer)),
            );
        }
        app::AppEvent::Room(RoomEvent::NeighborDown { room, peer }) => {
            ui.online.clone().write().remove(&peer);
            push_system_bubble(
                ui,
                &room,
                format!("{} left #{room}", short_peer_label(ui, peer)),
            );
        }
        app::AppEvent::Room(RoomEvent::Lagged { room }) => {
            push_toast(
                ui,
                format!("fell behind in #{room} — may have missed messages"),
                true,
            );
        }
        app::AppEvent::Contact(ContactEvent::IncomingRequest {
            from_id,
            from_username,
            from_name,
            note,
        }) => {
            tracing::info!(peer = %app::hex(from_id), username = ?from_username, "incoming contact request");
            spawn_remember_peer(ui, from_id);
            if let Some(core) = ui.core.read().as_ref() {
                refresh_contacts(ui, core);
            }
            let who = match &from_username {
                Some(u) => format!("{from_name} (@{u})"),
                None => from_name,
            };
            let text = if note.is_empty() {
                format!("New contact request from {who}")
            } else {
                format!("New contact request from {who}: \"{note}\"")
            };
            notify_android_message(ui, "New contact request", text.clone());
            push_toast(ui, text, false);
        }
        app::AppEvent::Contact(ContactEvent::RequestAccepted {
            from_id,
            from_username,
            from_name,
        }) => {
            // Contact row itself is already updated by the protocol
            // handler (see net::contacts::ContactProtocol::accept) — this
            // branch only needs to refresh the UI's view of it and notify.
            let _ = from_username;
            spawn_remember_peer(ui, from_id);
            push_toast(ui, format!("{from_name} accepted your request"), false);
            if let Some(core) = ui.core.read().as_ref() {
                refresh_contacts(ui, core);
            }
        }
        app::AppEvent::Contact(ContactEvent::RequestRejected { from_id }) => {
            push_toast(
                ui,
                format!("{} declined your contact request", app::short_id(from_id)),
                true,
            );
            if let Some(core) = ui.core.read().as_ref() {
                refresh_contacts(ui, core);
            }
        }
        app::AppEvent::Call(siar_core::net::calls::CallEvent::Incoming {
            from_id,
            from_name,
            wants_video: _,
            decision,
        }) => {
            // One call at a time: if we're already ringing or in a call,
            // decline automatically rather than showing two overlapping
            // states with no clear way to represent both.
            if ui.incoming_call.cloned().is_some()
                || ui.active_call.cloned().is_some()
                || ui.outgoing_call.cloned().is_some()
            {
                let _ = decision.send(false);
                return;
            }
            ui.incoming_call_decision.clone().set(Some(decision));
            notify_android_message(ui, "Incoming Siar call", format!("{from_name} is calling"));
            ui.incoming_call.clone().set(Some((from_id, from_name)));
            ui.active_call_direction
                .clone()
                .set(Some(CallDirection::Incoming));
            ui.active_ringtone
                .clone()
                .write()
                .replace(siar_core::ringtone::Ringtone::start(false));
        }
        app::AppEvent::Call(siar_core::net::calls::CallEvent::Connected {
            peer,
            hangup_tx,
            video,
            video_codec: _,
        }) => {
            // `hangup_tx` is `Some` only on the callee side (see
            // `CallEvent::Connected`'s doc) — the caller already
            // registered its own before `place_call` started. Without
            // this, clicking "hang up" as the callee updated the UI but
            // never actually signaled `audio::run_session` to stop —
            // mic/speaker stayed live until the *other* side hung up or
            // the connection dropped.
            if let Some(tx) = hangup_tx {
                if let Some(core) = ui.core.clone().write().as_mut() {
                    core.set_active_call_hangup(tx);
                }
            }
            let name = ui
                .contacts
                .read()
                .iter()
                .find(|c| c.endpoint_id == app::hex(peer))
                .map(|c| c.alias.clone());
            ui.active_ringtone.clone().write().take(); // ringing's over — drop stops playback
            ui.incoming_call.clone().set(None);
            ui.outgoing_call.clone().set(None);
            ui.active_call
                .clone()
                .set(Some((peer, name.unwrap_or_else(|| app::short_id(peer)))));
            ui.active_call_started_ms.clone().set(Some(now_ms()));
            // `video` reflects what was actually negotiated (both sides
            // wanted it and both had a usable camera) — not merely what we
            // asked for when placing the call, since the other side may
            // not have a camera even on a video-requested invite. See
            // `net::calls::CallEvent::Connected`'s doc.
            ui.active_call_has_video.clone().set(video);
            ui.remote_video_frame.clone().set(None);
            ui.local_video_frame.clone().set(None);
        }
        app::AppEvent::Call(siar_core::net::calls::CallEvent::VideoFrame {
            peer: _,
            from_local_camera,
            data_uri,
        }) => {
            if from_local_camera {
                ui.local_video_frame.clone().set(Some(data_uri));
            } else {
                ui.remote_video_frame.clone().set(Some(data_uri));
            }
        }
        app::AppEvent::Call(siar_core::net::calls::CallEvent::VideoUnavailable {
            peer: _,
            reason,
        }) => {
            push_toast(ui, format!("video: {reason}"), true);
        }
        app::AppEvent::Call(siar_core::net::calls::CallEvent::Ended { peer, reason }) => {
            let name = ui
                .active_call
                .cloned()
                .map(|(_, n)| n)
                .or_else(|| {
                    ui.contacts
                        .read()
                        .iter()
                        .find(|c| c.endpoint_id == app::hex(peer))
                        .map(|c| c.alias.clone())
                })
                .unwrap_or_else(|| app::short_id(peer));
            let direction = ui
                .active_call_direction
                .cloned()
                .unwrap_or(CallDirection::Outgoing);
            let started = ui.active_call_started_ms.cloned();
            let outcome = match (started, reason.as_str()) {
                (Some(_), _) => CallOutcome::Completed,
                (None, "declined") => CallOutcome::Declined,
                (None, _) if direction == CallDirection::Incoming => CallOutcome::Missed,
                (None, _) => CallOutcome::Failed,
            };
            let duration_secs = started.map(|s| (now_ms() - s) / 1000).unwrap_or(0);
            let logged_at = started.unwrap_or_else(now_ms);
            if let Some(core) = ui.core.read().as_ref() {
                if let Err(e) =
                    core.log_call(peer, &name, direction, outcome, logged_at, duration_secs)
                {
                    tracing::warn!(error = %e, "failed to persist call log entry");
                }
            }
            refresh_call_log(ui);
            ui.active_ringtone.clone().write().take();
            ui.incoming_call.clone().set(None);
            ui.outgoing_call.clone().set(None);
            ui.active_call.clone().set(None);
            ui.active_call_started_ms.clone().set(None);
            ui.active_call_direction.clone().set(None);
            ui.active_call_has_video.clone().set(false);
            ui.remote_video_frame.clone().set(None);
            ui.local_video_frame.clone().set(None);
            push_toast(ui, format!("call with {name}: {reason}"), false);
        }
    }
}

/// Reply to a just-received text/file message with a `Body::Ack` on the
/// same session — see `protocol::message`'s module doc for why this needs
/// to be a real message rather than trusting the local `send()` call that
/// originally delivered it.
fn spawn_send_ack(ui: UiState, from: EndpointId, acked_id: u64) {
    spawn(async move {
        let (endpoint, existing, my_name) = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            (
                core.endpoint(),
                core.existing_dm_session(from),
                core.my_name.clone(),
            )
        };
        // This used to silently skip sending the ack at all when there
        // was no already-cached session — reasoned as "they'll find out
        // it's delivered next time we send them something." That reasoning
        // has a real gap: if we never happen to send anything else to
        // them afterward (or our cached session was dropped in the
        // meantime — e.g. by the keepalive sweep noticing it had died),
        // the ack is gone for good, with nothing to retry it. That's
        // exactly the "message stuck on a single checkmark forever, even
        // after reconnecting" bug — the *message* got through fine; it
        // was only the confirmation coming back that got silently
        // dropped. An ack is small and infrequent enough that actively
        // connecting for it (same retry/backoff as a real message,
        // `connect_with_retry`) is a fine cost for closing that gap.
        let session = match existing {
            Some(s) => s,
            None => match app::connect_with_retry(&endpoint, from).await {
                Ok(s) => {
                    let _ = s.say_hello(&my_name).await;
                    if let Some(core) = ui.core.clone().write().as_mut() {
                        core.commit_dm_session(from, s.clone());
                    }
                    s
                }
                Err(e) => {
                    tracing::debug!(peer = %app::hex(from), error = %e, "couldn't connect back to send ack");
                    return;
                }
            },
        };
        let _ = session.send(&Envelope::ack(my_name, acked_id)).await;
    });
}

fn mark_acked(ui: UiState, key: &ConvKey, acked_id: u64) {
    if let Some(bubbles) = ui.conversations.clone().write().get_mut(key) {
        for b in bubbles.iter_mut() {
            if b.id == Some(acked_id) {
                if let BubbleKind::Own { acked } = &mut b.kind {
                    *acked = true;
                }
                break;
            }
        }
    }
}

/// Applies a `Body::Reaction` to the matching in-memory bubble, if it's
/// currently loaded (same "instant local update, persistence already
/// happened separately" split `mark_acked` uses). No-ops quietly if the
/// target bubble isn't loaded — the persisted reaction is still there
/// next time the conversation's history loads.
fn apply_reaction_locally(
    ui: UiState,
    key: &ConvKey,
    target_id: u64,
    sender_id: String,
    emoji: String,
    remove: bool,
) {
    if let Some(bubbles) = ui.conversations.clone().write().get_mut(key) {
        for b in bubbles.iter_mut() {
            if b.id == Some(target_id) {
                b.reactions.retain(|(s, _)| s != &sender_id);
                if !remove {
                    b.reactions.push((sender_id, emoji));
                }
                break;
            }
        }
    }
}

/// Applies a `Body::Edit` to the matching in-memory bubble — only
/// touches `content` for `StoredContent::Text`; an edit targeting a
/// `File` bubble (which shouldn't happen — `Body::Edit` is only ever
/// sent for text, see the composer's edit action) is a silent no-op
/// rather than a crash.
fn apply_edit_locally(ui: UiState, key: &ConvKey, target_id: u64, new_text: String) {
    if let Some(bubbles) = ui.conversations.clone().write().get_mut(key) {
        for b in bubbles.iter_mut() {
            if b.id == Some(target_id) {
                if let StoredContent::Text(t) = &mut b.content {
                    *t = new_text;
                    b.edited = true;
                }
                break;
            }
        }
    }
}

/// Applies a `Body::Delete` to the matching in-memory bubble — replaces
/// its content with a placeholder, same tombstone-in-place approach as
/// the sqlite side (`Store::apply_delete`'s doc).
fn apply_delete_locally(ui: UiState, key: &ConvKey, target_id: u64) {
    if let Some(bubbles) = ui.conversations.clone().write().get_mut(key) {
        for b in bubbles.iter_mut() {
            if b.id == Some(target_id) {
                b.content = StoredContent::Text("This message was deleted".to_string());
                b.deleted = true;
                break;
            }
        }
    }
}

fn is_peer_typing(ui: UiState, key: &ConvKey) -> bool {
    ui.typing
        .read()
        .get(key)
        .is_some_and(|t| now_ms() - t < TYPING_TIMEOUT.as_millis() as i64)
}

/// Rate-limited outgoing typing signal — called on every composer
/// keystroke, but only actually sends if enough time has passed since the
/// last one for this conversation (see `TYPING_RESEND_INTERVAL_MS`).
fn maybe_send_typing(ui: UiState, key: ConvKey) {
    let ConvKey::Dm(peer) = key else { return }; // rooms: skip for now, see ARCHITECTURE.md note below
    let now = now_ms();
    let mut last_typing_sent = ui.last_typing_sent;
    {
        let mut last = last_typing_sent.write();
        let should_send = last
            .get(&ConvKey::Dm(peer))
            .is_none_or(|t| now - t >= TYPING_RESEND_INTERVAL_MS);
        if !should_send {
            return;
        }
        last.insert(ConvKey::Dm(peer), now);
    }
    spawn(async move {
        let (session, my_name) = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            (core.existing_dm_session(peer), core.my_name.clone())
        };
        if let Some(session) = session {
            let _ = session.send(&Envelope::typing(my_name)).await;
        }
    });
}

fn record_incoming(ui: UiState, key: ConvKey, envelope: &Envelope, from: Option<EndpointId>) {
    let bubble = match &envelope.body {
        Body::Text { text, reply_to } => Some(StoredBubble {
            id: Some(envelope.id),
            kind: BubbleKind::Peer,
            sender: envelope.from_name.clone(),
            content: StoredContent::Text(text.clone()),
            sent_unix_ms: envelope.sent_unix_ms as i64,
            expires_at_unix_ms: envelope.expires_at_unix_ms.map(|v| v as i64),
            reactions: Vec::new(),
            edited: false,
            deleted: false,
            reply_to_envelope_id: *reply_to,
        }),
        Body::File {
            name,
            size_bytes,
            compressed,
            blake3_hash,
            reply_to,
            ..
        } => Some(StoredBubble {
            id: Some(envelope.id),
            kind: BubbleKind::Peer,
            sender: envelope.from_name.clone(),
            content: StoredContent::File {
                hash: blake3_hash.clone(),
                name: name.clone(),
                size_bytes: *size_bytes,
                state: FileState::Idle,
                compressed: *compressed,
                from,
            },
            sent_unix_ms: envelope.sent_unix_ms as i64,
            expires_at_unix_ms: envelope.expires_at_unix_ms.map(|v| v as i64),
            reactions: Vec::new(),
            edited: false,
            deleted: false,
            reply_to_envelope_id: *reply_to,
        }),
        _ => None,
    };
    if let Some(bubble) = bubble {
        ui.conversations
            .clone()
            .write()
            .entry(key)
            .or_default()
            .push(bubble);
    }
}

/// Drop a `BubbleKind::System` line ("so-and-so joined/left") into a room's
/// history — in memory only, deliberately: this is a live presence notice,
/// not chat content, so it doesn't belong in `store.rs` (same content-vs-
/// metadata split `net::conv_docs` draws — see that module's doc comment)
/// and won't reappear if the room's history is reloaded from sqlite later.
fn push_system_bubble(ui: UiState, room: &str, text: String) {
    let bubble = StoredBubble {
        id: None,
        kind: BubbleKind::System,
        sender: String::new(),
        content: StoredContent::Text(text),
        sent_unix_ms: now_ms(),
        expires_at_unix_ms: None,
        reactions: Vec::new(),
        edited: false,
        deleted: false,
        reply_to_envelope_id: None,
    };
    ui.conversations
        .clone()
        .write()
        .entry(ConvKey::Room(room.to_string()))
        .or_default()
        .push(bubble);
}

/// Best-effort display name for a peer showing up in a system notice —
/// prefers a known contact's alias/username over a raw endpoint id, same
/// precedence `title_for` uses for DM headers.
fn short_peer_label(ui: UiState, peer: EndpointId) -> String {
    let hex = app::hex(peer);
    ui.contacts
        .read()
        .iter()
        .find(|c| c.endpoint_id == hex)
        .map(|c| c.alias.clone())
        .unwrap_or_else(|| app::short_id(peer))
}

fn bump_unread(ui: UiState, key: &ConvKey) {
    if ui.active.read().as_ref() == Some(key) {
        return; // already looking at it
    }
    *ui.unread.clone().write().entry(key.clone()).or_insert(0) += 1;
}

fn refresh_contacts(ui: UiState, core: &Core) {
    if let Ok(list) = core.accepted_contacts() {
        // Preload whichever of these avatars are already sitting in the
        // on-disk cache from a previous run (`download_history`) — covers
        // the "app restarted, don't wait for a fresh network fetch" case.
        // Anything not yet downloaded stays absent from `avatar_images`
        // here; it gets filled in later by whichever path actually
        // fetches it (`Body::AvatarUpdate`'s handler, or wherever
        // `set_my_avatar` is called for our own picture).
        let store = core.store();
        for contact in &list {
            let Some(hash) = &contact.avatar_hash else {
                continue;
            };
            if ui.avatar_images.read().contains_key(hash) {
                continue;
            }
            if let Ok(Some(path)) = store.cached_download_path(hash) {
                if let Ok(bytes) = std::fs::read(&path) {
                    load_avatar_into_cache(ui, hash, &bytes);
                }
            }
        }
        ui.contacts.clone().set(list);
    }
    if let Ok(list) = core.pending_incoming_requests() {
        ui.pending_requests.clone().set(list);
    }
    if let Ok(list) = core.known_room_names() {
        ui.rooms.clone().set(list);
    }
}

/// Fetch-and-cache a status image by hash from `from` — mirrors the
/// `Body::AvatarUpdate` prefetch pattern exactly (see that handler's
/// comment) and deliberately reuses `App::fetch_contact_avatar` and the
/// `avatar_images` cache rather than duplicating either: despite the
/// name, that fetch is just "download this content-addressed blob from
/// this peer and cache it on disk by hash," which is exactly what a
/// status image needs too. Also called from `refresh_statuses` for any
/// status whose image isn't in the cache yet (e.g. one that arrived in an
/// earlier session, before this launch's cache was warm).
fn pcm_to_wav_data_uri(pcm: &[i16], sample_rate: u32) -> String {
    let data_len = (pcm.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate (mono, 16-bit)
    wav.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for sample in pcm {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    format!(
        "data:audio/wav;base64,{}",
        data_encoding::BASE64.encode(&wav)
    )
}

fn jpeg_frames_to_gif_data_uri(jpeg_frames: &[Vec<u8>]) -> Option<String> {
    use image::codecs::gif::GifEncoder;
    use image::Frame;

    let mut gif_bytes = Vec::new();
    {
        let mut encoder = GifEncoder::new(&mut gif_bytes);
        encoder
            .set_repeat(image::codecs::gif::Repeat::Infinite)
            .ok()?;
        for jpeg in jpeg_frames {
            let decoded =
                image::load_from_memory_with_format(jpeg, image::ImageFormat::Jpeg).ok()?;
            encoder.encode_frame(Frame::new(decoded.to_rgba8())).ok()?;
        }
    }
    Some(format!(
        "data:image/gif;base64,{}",
        data_encoding::BASE64.encode(&gif_bytes)
    ))
}

/// Fetch-and-cache a status voice clip by hash from `from` — same fetch
/// as `spawn_fetch_status_image`/`spawn_fetch_status_video`, decoded via
/// `net::calls::audio::decode_voice_clip` (the same Opus decoder live
/// calls use) and wrapped in a WAV data URI so the webview's native
/// `<audio>` element can play it directly.
fn spawn_fetch_status_audio(ui: UiState, from: EndpointId, hash: String) {
    if ui.avatar_images.read().contains_key(&hash) {
        return; // already decoded — see spawn_fetch_status_image's identical note
    }
    spawn(async move {
        let (store, blobs, endpoint) = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            (core.store(), core.blobs(), core.endpoint())
        };
        let cache_dir = CONFIG.get().unwrap().data_dir.join("avatar_cache");
        let clip_bytes = match app::App::fetch_contact_avatar(
            &store, &blobs, &endpoint, from, &hash, &cache_dir,
        )
        .await
        {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::debug!(peer = %app::hex(from), error = %e, "status voice clip fetch failed, will retry on next open");
                return;
            }
        };
        let decoded = tokio::task::spawn_blocking(move || {
            siar_core::net::calls::audio::decode_voice_clip(&clip_bytes)
        })
        .await;
        match decoded {
            Ok(Ok(pcm)) if !pcm.is_empty() => {
                let data_uri = pcm_to_wav_data_uri(&pcm, siar_core::net::calls::audio::SAMPLE_RATE);
                ui.avatar_images.clone().write().insert(hash, data_uri);
            }
            Ok(Ok(_)) => {} // decoded cleanly but produced no samples — nothing to play
            Ok(Err(e)) => tracing::debug!(error = %e, "status voice clip decode failed"),
            Err(e) => tracing::debug!(error = %e, "status voice clip decode task panicked"),
        }
    });
}

fn spawn_fetch_status_image(ui: UiState, from: EndpointId, hash: String) {
    if ui.avatar_images.read().contains_key(&hash) {
        return; // already cached — this is what makes re-calling it from
                // refresh_statuses on every refresh cheap rather than
                // re-fetching the same image over and over
    }
    spawn(async move {
        let (store, blobs, endpoint) = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            (core.store(), core.blobs(), core.endpoint())
        };
        let cache_dir = CONFIG.get().unwrap().data_dir.join("avatar_cache");
        match app::App::fetch_contact_avatar(&store, &blobs, &endpoint, from, &hash, &cache_dir)
            .await
        {
            Ok(bytes) => load_avatar_into_cache(ui, &hash, &bytes),
            Err(e) => {
                tracing::debug!(peer = %app::hex(from), error = %e, "status image fetch failed, will retry on next open")
            }
        }
    });
}

/// Fetch-and-cache a status video by hash from `from` — same fetch as
/// `spawn_fetch_status_image` (reuses `App::fetch_contact_avatar`, still
/// just "download this content-addressed blob and cache it on disk"
/// underneath), but decodes the result via `net::calls::video::decode_clip`
/// (the same AV1 decoder live calls use) instead of treating the bytes
/// as a static image, since a clip is a sequence of frames.
fn spawn_fetch_status_video(ui: UiState, from: EndpointId, hash: String) {
    if ui.avatar_images.read().contains_key(&hash) {
        return; // already decoded — see spawn_fetch_status_image's identical note
    }
    spawn(async move {
        let (store, blobs, endpoint) = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            (core.store(), core.blobs(), core.endpoint())
        };
        let cache_dir = CONFIG.get().unwrap().data_dir.join("avatar_cache");
        let clip_bytes = match app::App::fetch_contact_avatar(
            &store, &blobs, &endpoint, from, &hash, &cache_dir,
        )
        .await
        {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::debug!(peer = %app::hex(from), error = %e, "status video fetch failed, will retry on next open");
                return;
            }
        };
        let decoded = tokio::task::spawn_blocking(move || {
            siar_core::net::calls::video::decode_clip(&clip_bytes)
        })
        .await;
        let jpeg_frames = match decoded {
            Ok(Ok(frames)) if !frames.is_empty() => frames,
            Ok(Ok(_)) => return, // decoded cleanly but produced no frames — nothing to show
            Ok(Err(e)) => return tracing::debug!(error = %e, "status video decode failed"),
            Err(e) => return tracing::debug!(error = %e, "status video decode task panicked"),
        };
        let gif =
            tokio::task::spawn_blocking(move || jpeg_frames_to_gif_data_uri(&jpeg_frames)).await;
        if let Ok(Some(data_uri)) = gif {
            ui.avatar_images.clone().write().insert(hash, data_uri);
        }
    });
}

fn refresh_statuses(ui: UiState) {
    if let Some(core) = ui.core.read().as_ref() {
        if let Ok(list) = core.active_statuses() {
            for entry in &list {
                if let Ok(peer) = parse_hex(&entry.peer_id) {
                    if let Some(hash) = &entry.image_hash {
                        spawn_fetch_status_image(ui, peer, hash.clone());
                    }
                    if let Some(hash) = &entry.video_hash {
                        spawn_fetch_status_video(ui, peer, hash.clone());
                    }
                    if let Some(hash) = &entry.audio_hash {
                        spawn_fetch_status_audio(ui, peer, hash.clone());
                    }
                }
            }
            ui.statuses.clone().set(list);
        }
    }
}

fn refresh_call_log(ui: UiState) {
    if let Some(core) = ui.core.read().as_ref() {
        if let Ok(list) = core.recent_calls(200) {
            ui.call_log.clone().set(list);
        }
    }
}

fn spawn_post_status(ui: UiState) {
    let text = ui.status_compose.cloned().trim().to_string();
    let image = ui.status_image.cloned();
    let video = ui.status_video_pending.cloned();
    let audio = ui.status_audio_pending.cloned();
    if text.is_empty() && image.is_none() && video.is_none() && audio.is_none() {
        return push_toast(
            ui,
            "write something, attach an image, or record a video/voice clip first".to_string(),
            true,
        );
    }
    let hours_input = ui.status_ttl_hours.cloned();
    let ttl_hours: u64 = if hours_input.trim().is_empty() {
        24 // WhatsApp/Signal-style default — see the disappearing-messages
           // panel's identical reasoning
    } else {
        match hours_input.trim().parse() {
            Ok(h) if (1..=168).contains(&h) => h,
            _ => {
                return push_toast(
                    ui,
                    "custom duration must be between 1 and 168 hours".to_string(),
                    true,
                )
            }
        }
    };
    spawn(async move {
        let result = {
            let core_ref = ui.core.read();
            match core_ref.as_ref() {
                Some(core) => Some(
                    core.broadcast_status(&text, image.as_deref(), video, audio, ttl_hours)
                        .await,
                ),
                None => None,
            }
        };
        match result {
            Some(Ok((cached_image, cached_video, cached_audio))) => {
                ui.status_compose.clone().set(String::new());
                ui.status_image.clone().set(None);
                ui.status_video_pending.clone().set(None);
                ui.status_audio_pending.clone().set(None);
                if let Some((hash, bytes)) = cached_image {
                    load_avatar_into_cache(ui, &hash, &bytes);
                }
                if let Some((hash, clip_bytes)) = cached_video {
                    // Same reasoning as the image case: build our own
                    // just-posted clip's GIF locally instead of leaving
                    // it to "fetch from myself," which would just hang.
                    let decoded = tokio::task::spawn_blocking(move || {
                        siar_core::net::calls::video::decode_clip(&clip_bytes)
                    })
                    .await;
                    if let Ok(Ok(frames)) = decoded {
                        if let Ok(Some(gif)) = tokio::task::spawn_blocking(move || {
                            jpeg_frames_to_gif_data_uri(&frames)
                        })
                        .await
                        {
                            ui.avatar_images.clone().write().insert(hash, gif);
                        }
                    }
                }
                if let Some((hash, clip_bytes)) = cached_audio {
                    // Same reasoning again, for our own just-posted voice clip.
                    let decoded = tokio::task::spawn_blocking(move || {
                        siar_core::net::calls::audio::decode_voice_clip(&clip_bytes)
                    })
                    .await;
                    if let Ok(Ok(pcm)) = decoded {
                        let data_uri =
                            pcm_to_wav_data_uri(&pcm, siar_core::net::calls::audio::SAMPLE_RATE);
                        ui.avatar_images.clone().write().insert(hash, data_uri);
                    }
                }
                refresh_statuses(ui);
                push_toast(ui, "status posted".to_string(), false);
            }
            Some(Err(e)) => push_toast(ui, format!("couldn't post status: {e}"), true),
            None => {}
        }
    });
}

/// Record a short (5s) status video clip via `net::calls::video::record_video_clip`
/// — the exact same camera-open/capture path live video calls use, just
/// batch-recorded instead of streamed. Blocking capture work runs on
/// spawn_blocking, same as everywhere else CPU/IO-bound work meets the
/// async UI layer in this codebase.
const STATUS_VIDEO_RECORD_SECS: u32 = 5;
/// Same idea, for a status voice clip.
const STATUS_AUDIO_RECORD_SECS: u32 = 15;

fn spawn_record_status_video(ui: UiState) {
    if ui.status_recording.cloned() {
        return;
    }
    ui.status_recording.clone().set(true);
    spawn(async move {
        let result = tokio::task::spawn_blocking(|| {
            siar_core::net::calls::video::record_video_clip(STATUS_VIDEO_RECORD_SECS)
        })
        .await;
        ui.status_recording.clone().set(false);
        match result {
            Ok(Ok(frames)) => {
                ui.status_video_pending.clone().set(Some(frames));
            }
            Ok(Err(e)) => push_toast(ui, format!("couldn't record video: {e}"), true),
            Err(e) => push_toast(ui, format!("recording task panicked: {e}"), true),
        }
    });
}

/// Record (and encode, in the same blocking call — see
/// `net::calls::audio::record_and_encode_voice_clip`'s doc) a short
/// status voice clip.
fn spawn_record_status_audio(ui: UiState) {
    if ui.status_recording_audio.cloned() {
        return;
    }
    spawn(async move {
        if !ensure_android_audio_permission().await {
            return push_toast(
                ui,
                "Allow microphone access, then tap Voice again".to_string(),
                false,
            );
        }
        ui.status_recording_audio.clone().set(true);
        let result = tokio::task::spawn_blocking(|| {
            siar_core::net::calls::audio::record_and_encode_voice_clip(STATUS_AUDIO_RECORD_SECS)
        })
        .await;
        ui.status_recording_audio.clone().set(false);
        match result {
            Ok(Ok(clip_bytes)) => {
                ui.status_audio_pending.clone().set(Some(clip_bytes));
            }
            Ok(Err(e)) => push_toast(ui, format!("couldn't record voice clip: {e}"), true),
            Err(e) => push_toast(ui, format!("recording task panicked: {e}"), true),
        }
    });
}

async fn ensure_android_audio_permission() -> bool {
    #[cfg(target_os = "android")]
    {
        let script = r#"
            if (!window.SiarAndroid) return false;
            const granted = window.SiarAndroid.hasAudioPermission();
            if (!granted) window.SiarAndroid.requestAudioPermission();
            return granted;
        "#;
        return document::eval(script).join::<bool>().await.unwrap_or(false);
    }

    #[cfg(not(target_os = "android"))]
    true
}

// ---- Actions ----

/// Clicking an "already a contact" registry search result used to do
/// nothing — the row had no click handler at all, so there was no way to
/// get from "I searched a username and it's already a contact" to
/// actually opening that conversation short of clearing the search box
/// and finding them in the plain chat list by alias instead. Resolves
/// `username` against `ui.contacts` (registry search only ever hands back
/// a username string, not an `EndpointId` — see `spawn_search`) and opens
/// the DM the same way clicking a normal chat-list row does.
/// Encode `png_bytes` as a `data:image/png;base64,...` URI and store it
/// under `hash` in `ui.avatar_images` — the one place raw avatar bytes
/// (from `App::set_my_avatar`'s return value, or a background
/// `App::fetch_contact_avatar`) turn into something `ui::sidebar::Avatar`
/// can put directly in an `img` tag's `src`.
fn load_avatar_into_cache(ui: UiState, hash: &str, png_bytes: &[u8]) {
    let data_uri = format!(
        "data:image/png;base64,{}",
        data_encoding::BASE64.encode(png_bytes)
    );
    ui.avatar_images
        .clone()
        .write()
        .insert(hash.to_string(), data_uri);
}

fn open_existing_contact_by_username(ui: UiState, username: String) {
    let Some(peer) = ui
        .contacts
        .read()
        .iter()
        .find(|c| c.username.as_deref() == Some(username.as_str()))
        .and_then(|c| parse_hex(&c.endpoint_id).ok())
    else {
        return;
    };
    ui.search_query.clone().set(String::new());
    ui.search_results.clone().set(vec![]);
    select_conversation(ui, ConvKey::Dm(peer));
}

fn select_conversation(ui: UiState, key: ConvKey) {
    ui.active.clone().set(Some(key.clone()));
    ui.unread.clone().write().remove(&key);
    ui.sidebar_tab.clone().set(SidebarTab::Chats);

    // Lazily load history from sqlite on first open of a conversation.
    if !ui.conversations.read().contains_key(&key) {
        if let Some(core) = ui.core.read().as_ref() {
            let conv = match &key {
                ConvKey::Dm(id) => Conversation::Dm(app::hex(*id)),
                ConvKey::Room(name) => Conversation::Room(name.clone()),
            };
            let dm_peer = match &key {
                ConvKey::Dm(id) => Some(*id),
                ConvKey::Room(_) => None,
            };
            if let Ok(history) = core.history(&conv, 200) {
                let bubbles = history
                    .into_iter()
                    .map(|m| StoredBubble {
                        // Now that `envelope_id` is persisted (see the
                        // `Store::open` migration comment), history rows
                        // can be reacted to/edited/deleted too, not just
                        // messages from the current live session. `None`
                        // only for rows written before that column
                        // existed.
                        id: m.envelope_id,
                        kind: if m.outgoing {
                            BubbleKind::Own { acked: true }
                        } else {
                            BubbleKind::Peer
                        },
                        sender: m.sender_name,
                        content: if m.deleted {
                            // Tombstoned — see `StoredMessage::deleted`'s
                            // doc: the row keeps its real `body` in
                            // sqlite, only the rendered bubble shows a
                            // placeholder.
                            StoredContent::Text("This message was deleted".to_string())
                        } else {
                            match m.kind {
                                siar_core::store::MessageKind::Text => StoredContent::Text(m.body),
                                siar_core::store::MessageKind::File {
                                    name,
                                    size_bytes,
                                    hash,
                                    compressed,
                                } => {
                                    StoredContent::File {
                                        hash,
                                        name,
                                        size_bytes,
                                        // Our own past sends have nothing to
                                        // fetch; incoming DM files can still be
                                        // fetched from the same peer post-restart;
                                        // incoming room files can't (see
                                        // `StoredContent::File::from`'s doc).
                                        state: FileState::Idle,
                                        compressed,
                                        from: if m.outgoing { None } else { dm_peer },
                                    }
                                }
                            }
                        },
                        sent_unix_ms: m.sent_unix_ms,
                        expires_at_unix_ms: m.expires_at_unix_ms,
                        reactions: m.reactions,
                        edited: m.edited_at_unix_ms.is_some(),
                        deleted: m.deleted,
                        reply_to_envelope_id: m.reply_to_envelope_id,
                    })
                    .collect();
                ui.conversations
                    .clone()
                    .write()
                    .insert(key.clone(), bubbles);
            }
        }
    }

    // Read receipt: DM-only (see `Body::Read`'s doc), and best-effort
    // against a session that's already connected — worth sending when
    // you open a chat you're already talking to, not worth dialing a
    // peer freshly just to announce you've read something.
    if let ConvKey::Dm(peer) = key {
        if let Some(core) = ui.core.read().as_ref() {
            let conv = Conversation::Dm(app::hex(peer));
            if let Ok(Some(watermark)) = core.store().read_watermark(&conv, &app::hex(peer)) {
                ui.read_watermarks
                    .clone()
                    .write()
                    .insert(ConvKey::Dm(peer), watermark);
            }
        }
        spawn_send_read_receipt(ui, peer);
    }
}

fn spawn_send_read_receipt(ui: UiState, peer: EndpointId) {
    let now = now_ms() as u64;
    spawn(async move {
        let (session, my_name) = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            (core.existing_dm_session(peer), core.my_name.clone())
        };
        let Some(session) = session else { return }; // no active session — see this fn's doc
        let envelope = Envelope::read_receipt(my_name, now);
        if let Err(e) = session.send(&envelope).await {
            tracing::debug!(peer = %app::hex(peer), error = %e, "read receipt not delivered (peer likely offline)");
        }
    });
}

fn send_to_active(ui: UiState) {
    let text = ui.compose.read().trim().to_string();
    if text.is_empty() {
        return;
    }
    let Some(key) = ui.active.read().clone() else {
        return;
    };

    if let Some(target_id) = *ui.editing.read() {
        ui.compose.clone().set(String::new());
        ui.editing.clone().set(None);
        return spawn_send_edit(ui, key, target_id, text);
    }

    ui.compose.clone().set(String::new());

    // Reply target, if the composer has one queued (see `replying_to`'s
    // doc) — taken (not just read) here because sending is what consumes
    // it, same as `editing` above.
    let reply_to = ui.replying_to.read().as_ref().map(|(id, _, _)| *id);
    ui.replying_to.clone().set(None);

    let my_name = match ui.core.read().as_ref() {
        Some(core) => core.my_name.clone(),
        None => return,
    };
    // Built once, here, so the bubble we push and the envelope we actually
    // send share one `id` — that's what lets a later `Body::Ack` find and
    // flag the right bubble (see `mark_acked`).
    // Disappearing-message TTL: DMs only right now (see
    // `net::conv_docs::DmSettings` and the room-side "not wired up yet"
    // notes in `App::log_outgoing_room`/`record_incoming_room`). Attached
    // here, before the echo bubble below, so our own copy and the wire
    // copy agree on the exact same `expires_at_unix_ms`.
    let ttl_secs = match &key {
        ConvKey::Dm(peer) => ui
            .dm_settings_cache
            .read()
            .get(peer)
            .and_then(|s| s.disappearing_ttl_secs),
        ConvKey::Room(_) => None,
    };
    let envelope = match reply_to {
        Some(target_id) => Envelope::text_reply(my_name, text.clone(), target_id),
        None => Envelope::text(my_name, text.clone()),
    }
    .with_expiry(ttl_secs);

    ui.conversations
        .clone()
        .write()
        .entry(key.clone())
        .or_default()
        .push(StoredBubble {
            id: Some(envelope.id),
            kind: BubbleKind::Own { acked: false },
            sender: "me".to_string(),
            content: StoredContent::Text(text.clone()),
            sent_unix_ms: envelope.sent_unix_ms as i64,
            expires_at_unix_ms: envelope.expires_at_unix_ms.map(|v| v as i64),
            reactions: Vec::new(),
            edited: false,
            deleted: false,
            reply_to_envelope_id: reply_to,
        });

    match key {
        ConvKey::Dm(peer) => spawn_dm_send(ui, peer, envelope, text),
        ConvKey::Room(name) => spawn_room_send(ui, name, envelope, text),
    }
}

/// Shared by `spawn_send_edit`/`spawn_send_delete`/`spawn_send_reaction`:
/// gets (connecting/joining if needed, same as `spawn_dm_send`/
/// `spawn_room_send`) whatever's needed to actually put `envelope` on the
/// wire for `key`, and sends it. Returns an error string for the caller
/// to toast — unlike the two functions above, none of these three log a
/// new message row on success, so there's no shared "success" side
/// effect worth folding in here too.
async fn deliver_envelope(ui: UiState, key: &ConvKey, envelope: &Envelope) -> Result<(), String> {
    match key {
        ConvKey::Dm(peer) => {
            let (endpoint, existing, my_name) = {
                let core_ref = ui.core.read();
                let Some(core) = core_ref.as_ref() else {
                    return Err("not signed in".to_string());
                };
                (
                    core.endpoint(),
                    core.existing_dm_session(*peer),
                    core.my_name.clone(),
                )
            };
            let session = match existing {
                Some(s) => s,
                None => match app::connect_with_retry(&endpoint, *peer).await {
                    Ok(s) => {
                        let _ = s.say_hello(&my_name).await;
                        if let Some(core) = ui.core.clone().write().as_mut() {
                            core.commit_dm_session(*peer, s.clone());
                        }
                        s
                    }
                    Err(e) => return Err(format!("connect failed: {e}")),
                },
            };
            session
                .send(envelope)
                .await
                .map_err(|e| format!("send failed: {e}"))
        }
        ConvKey::Room(name) => {
            let (gossip, existing, room_tx) = {
                let core_ref = ui.core.read();
                let Some(core) = core_ref.as_ref() else {
                    return Err("not signed in".to_string());
                };
                (core.gossip(), core.existing_room(name), core.room_tx())
            };
            let room = match existing {
                Some(r) => r,
                None => match app::join_room_with_retry(&gossip, name, vec![], room_tx).await {
                    Ok(r) => {
                        if let Some(core) = ui.core.clone().write().as_mut() {
                            core.commit_room(name, r.clone());
                        }
                        r
                    }
                    Err(e) => return Err(format!("join room failed: {e}")),
                },
            };
            room.broadcast(envelope)
                .await
                .map_err(|e| format!("broadcast failed: {e}"))
        }
    }
}

/// The DM/Room-agnostic "who am I" pair (`sender_id` for the store's
/// ownership checks, `my_name` for the envelope) — pulled out since all
/// three functions below need it and `ui.core` shouldn't stay locked
/// across the `deliver_envelope` await above them.
fn my_identity(ui: UiState) -> Option<(String, String)> {
    ui.core
        .read()
        .as_ref()
        .map(|core| (app::hex(core.my_id), core.my_name.clone()))
}

fn spawn_send_edit(ui: UiState, key: ConvKey, target_id: u64, new_text: String) {
    spawn(async move {
        let Some((my_id, my_name)) = my_identity(ui) else {
            return;
        };
        let envelope = Envelope::edit(my_name, target_id, new_text.clone());
        match deliver_envelope(ui, &key, &envelope).await {
            Ok(()) => {
                if let Some(core) = ui.core.read().as_ref() {
                    let conv = match &key {
                        ConvKey::Dm(peer) => Conversation::Dm(app::hex(*peer)),
                        ConvKey::Room(name) => Conversation::Room(name.clone()),
                    };
                    if let Err(e) =
                        core.store()
                            .apply_edit(&conv, target_id, &my_id, &new_text, now_ms())
                    {
                        tracing::warn!(error = %e, "failed to persist outgoing edit");
                    }
                }
                apply_edit_locally(ui, &key, target_id, new_text);
            }
            Err(e) => push_toast(ui, format!("edit failed: {e}"), true),
        }
    });
}

fn spawn_send_delete(ui: UiState, key: ConvKey, target_id: u64) {
    spawn(async move {
        let Some((my_id, my_name)) = my_identity(ui) else {
            return;
        };
        let envelope = Envelope::delete(my_name, target_id);
        match deliver_envelope(ui, &key, &envelope).await {
            Ok(()) => {
                if let Some(core) = ui.core.read().as_ref() {
                    let conv = match &key {
                        ConvKey::Dm(peer) => Conversation::Dm(app::hex(*peer)),
                        ConvKey::Room(name) => Conversation::Room(name.clone()),
                    };
                    if let Err(e) = core.store().apply_delete(&conv, target_id, &my_id) {
                        tracing::warn!(error = %e, "failed to persist outgoing delete");
                    }
                }
                apply_delete_locally(ui, &key, target_id);
            }
            Err(e) => push_toast(ui, format!("delete failed: {e}"), true),
        }
    });
}

/// Tapping a quick-react emoji: `remove` is decided here (not passed in)
/// by checking whether *you* already have that exact reaction on this
/// message — tap again to take it back off, same toggle behavior every
/// chat app's reaction picker has.
fn spawn_send_reaction(ui: UiState, key: ConvKey, target_id: u64, emoji: String) {
    spawn(async move {
        let Some((my_id, my_name)) = my_identity(ui) else {
            return;
        };
        let already_reacted = ui
            .conversations
            .read()
            .get(&key)
            .and_then(|bubbles| bubbles.iter().find(|b| b.id == Some(target_id)))
            .is_some_and(|b| b.reactions.iter().any(|(s, e)| s == &my_id && e == &emoji));
        let envelope = Envelope::reaction(my_name, target_id, emoji.clone(), already_reacted);
        match deliver_envelope(ui, &key, &envelope).await {
            Ok(()) => {
                if let Some(core) = ui.core.read().as_ref() {
                    let conv = match &key {
                        ConvKey::Dm(peer) => Conversation::Dm(app::hex(*peer)),
                        ConvKey::Room(name) => Conversation::Room(name.clone()),
                    };
                    if let Err(e) = core.store().apply_reaction(
                        &conv,
                        target_id,
                        &my_id,
                        &emoji,
                        already_reacted,
                    ) {
                        tracing::warn!(error = %e, "failed to persist outgoing reaction");
                    }
                }
                apply_reaction_locally(ui, &key, target_id, my_id, emoji, already_reacted);
            }
            Err(e) => push_toast(ui, format!("reaction failed: {e}"), true),
        }
    });
}

/// Tries the offline-mesh fallback for one envelope. Deliberately reads
/// `ui.core` just long enough to check the setting and clone out an
/// owned `Arc<MeshManager>` — never holds that `Signal` read guard
/// across the `.await` below. Same `AlreadyBorrowed`-class hazard this
/// codebase already hit once with a held *write* guard across an
/// await point; a held *read* guard across a network-bound await is
/// the same risk (a concurrent `ui.core.write()` elsewhere while this
/// is suspended would panic), so it gets the same fix: extract, drop,
/// then await.
async fn try_mesh_send(ui: &UiState, envelope: &Envelope) -> bool {
    let mesh = {
        let core_ref = ui.core.read();
        match core_ref.as_ref() {
            Some(core) if core.offline_mesh_enabled() => core.mesh(),
            _ => return false,
        }
    };
    let Ok(bytes) = envelope.encode() else {
        return false;
    };
    mesh.send(bytes).await;
    true
}

fn spawn_dm_send(ui: UiState, peer: EndpointId, envelope: Envelope, text: String) {
    spawn(async move {
        let (endpoint, existing, my_name) = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            (
                core.endpoint(),
                core.existing_dm_session(peer),
                core.my_name.clone(),
            )
        };

        let session = match existing {
            Some(s) => s,
            None => match app::connect_with_retry(&endpoint, peer).await {
                Ok(s) => {
                    let _ = s.say_hello(&my_name).await;
                    if let Some(core) = ui.core.clone().write().as_mut() {
                        core.commit_dm_session(peer, s.clone());
                    }
                    s
                }
                Err(e) => {
                    // No direct/relay path reachable — try the offline
                    // mesh (BLE/LAN flood) before giving up entirely.
                    // No-op (returns false) unless the user has opted
                    // into it from Settings' Network tab.
                    let sent_via_mesh = try_mesh_send(&ui, &envelope).await;
                    if sent_via_mesh {
                        if let Some(core) = ui.core.read().as_ref() {
                            log_dm_sent_locally(core, peer, &envelope, &text);
                        }
                        push_toast(
                            ui,
                            "Sent via offline mesh (best-effort — no delivery confirmation)"
                                .to_string(),
                            false,
                        );
                    } else {
                        push_toast(ui, format!("connect failed: {e}"), true);
                    }
                    return;
                }
            },
        };

        match session.send(&envelope).await {
            Ok(()) => {
                if let Some(core) = ui.core.read().as_ref() {
                    log_dm_sent_locally(core, peer, &envelope, &text);
                }
            }
            Err(e) => {
                if let Some(core) = ui.core.clone().write().as_mut() {
                    core.drop_dm_session(peer);
                }
                let sent_via_mesh = try_mesh_send(&ui, &envelope).await;
                if sent_via_mesh {
                    if let Some(core) = ui.core.read().as_ref() {
                        log_dm_sent_locally(core, peer, &envelope, &text);
                    }
                    push_toast(
                        ui,
                        "Sent via offline mesh (best-effort — no delivery confirmation)"
                            .to_string(),
                        false,
                    );
                } else {
                    push_toast(ui, format!("send failed: {e}"), true);
                }
            }
        }
    });
}

/// Persists an outgoing DM's local echo (history row) the same way
/// regardless of whether it actually left over the QUIC session or the
/// offline mesh fallback — both call this once they've done whatever
/// send attempt succeeded (or is being optimistically treated as sent,
/// in the flood-and-hope mesh case).
fn log_dm_sent_locally(core: &Core, peer: EndpointId, envelope: &Envelope, text: &str) {
    // `reply_to` rides inside `envelope.body` (see
    // `protocol::message::Body::Text`) rather than as a separate
    // parameter here — the envelope built in `send_to_active` is the
    // one source of truth for it.
    let reply_to = match &envelope.body {
        Body::Text { reply_to, .. } => *reply_to,
        _ => None,
    };
    let _ = core.log_outgoing_dm(
        peer,
        text,
        envelope.sent_unix_ms as i64,
        envelope.expires_at_unix_ms.map(|v| v as i64),
        envelope.id,
        reply_to,
    );
}

fn spawn_room_send(ui: UiState, name: String, envelope: Envelope, text: String) {
    spawn(async move {
        let (gossip, existing, room_tx, my_name) = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            (
                core.gossip(),
                core.existing_room(&name),
                core.room_tx(),
                core.my_name.clone(),
            )
        };

        let room = match existing {
            Some(r) => r,
            None => match app::join_room_with_retry(&gossip, &name, vec![], room_tx).await {
                Ok(r) => {
                    if let Some(core) = ui.core.clone().write().as_mut() {
                        core.commit_room(&name, r.clone());
                    }
                    // Best-effort, same decoupled pattern as
                    // `spawn_join_room` — no UI lock held across the
                    // network-bound metadata sync.
                    let handles = {
                        let core_ref = ui.core.read();
                        core_ref.as_ref().map(|core| {
                            (
                                core.docs(),
                                core.blobs_store(),
                                core.docs_author(),
                                core.my_id,
                                core.my_name.clone(),
                            )
                        })
                    };
                    if let Some((docs, blobs_store, docs_author, my_id, my_name)) = handles {
                        match app::ensure_room_metadata_standalone(
                            docs,
                            blobs_store,
                            docs_author,
                            my_id,
                            my_name,
                            name.clone(),
                            vec![],
                        )
                        .await
                        {
                            Ok(room_doc) => {
                                if let Some(core) = ui.core.clone().write().as_mut() {
                                    core.commit_room_doc(&name, room_doc);
                                }
                            }
                            Err(e) => {
                                tracing::warn!(room = %name, error = %e, "couldn't sync room metadata")
                            }
                        }
                    }
                    r
                }
                Err(e) => return push_toast(ui, format!("join room failed: {e}"), true),
            },
        };

        let _ = my_name; // envelope was already built by the caller (send_to_active)
        match room.broadcast(&envelope).await {
            Ok(()) => {
                let reply_to = match &envelope.body {
                    Body::Text { reply_to, .. } => *reply_to,
                    _ => None,
                };
                if let Some(core) = ui.core.read().as_ref() {
                    let _ = core.log_outgoing_room(
                        &name,
                        &text,
                        envelope.sent_unix_ms as i64,
                        envelope.id,
                        reply_to,
                    );
                }
            }
            Err(e) => push_toast(ui, format!("broadcast failed: {e}"), true),
        }
    });
}

fn spawn_set_status_image(ui: UiState, raw: Vec<u8>) {
    spawn(async move {
        let validation = tokio::task::spawn_blocking({
            let raw = raw.clone();
            move || siar_core::media::decode_status_image(&raw).map(|_| ())
        })
        .await;
        match validation {
            Ok(Ok(())) => ui.status_image.clone().set(Some(raw)),
            Ok(Err(error)) => push_toast(ui, format!("couldn't use that image: {error}"), true),
            Err(error) => push_toast(ui, format!("image task failed: {error}"), true),
        }
    });
}

fn spawn_attach_status_image(ui: UiState) {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        // Same rfd limitation as `spawn_attach_file` — see its comment.
        push_toast(
            ui,
            "Attaching a status image isn't wired up on mobile yet".to_string(),
            true,
        );
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    spawn(async move {
        let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "jxl"])
            .pick_file()
            .await
        else {
            return;
        };
        spawn_set_status_image(ui, handle.read().await);
    });
}

/// Attach an existing local audio file as status voice — the
/// local-media counterpart to `spawn_record_status_audio`'s live mic
/// recording. Re-encodes through `net::calls::audio::
/// decode_and_encode_audio_file` into the exact same Opus clip blob
/// shape either path produces, so `ui.status_audio_pending` (and
/// everything downstream of it) doesn't need to know or care which one
/// the user picked.
fn spawn_attach_status_audio(ui: UiState) {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        // Same rfd limitation as `spawn_attach_file` — see its comment.
        push_toast(
            ui,
            "Attaching a status voice clip isn't wired up on mobile yet".to_string(),
            true,
        );
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    spawn(async move {
        let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("Audio", &["mp3", "wav", "flac", "m4a", "aac"])
            .pick_file()
            .await
        else {
            return;
        };
        let raw = handle.read().await;
        // `path()` is confirmed in rfd's own docs ("on native platforms
        // returns path"); deriving the name from it via
        // `std::path::Path::file_name` avoids depending on whether rfd
        // also has its own separate `file_name()` convenience method,
        // which wasn't something to confirm the exact signature of.
        let file_name = handle
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());
        spawn_set_status_audio(ui, raw, file_name);
    });
}

fn spawn_set_status_audio(ui: UiState, raw: Vec<u8>, file_name: Option<String>) {
    spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            siar_core::net::calls::audio::decode_and_encode_audio_file(&raw, file_name.as_deref())
        })
        .await;
        match result {
            Ok(Ok(clip_bytes)) => ui.status_audio_pending.clone().set(Some(clip_bytes)),
            Ok(Err(error)) => {
                push_toast(ui, format!("couldn't use that audio file: {error}"), true)
            }
            Err(error) => push_toast(ui, format!("audio decode task failed: {error}"), true),
        }
    });
}

/// Settings' Storage tab "Back up now" flow — encrypts the current
/// identity/messages/files into one file via `backup::create_backup`
/// (the seed phrase re-entered here is checked against this device's
/// actual identity before anything else happens, see that function's
/// doc) and saves it wherever the person picks, via the same `rfd`
/// save dialog every other local-file feature in this codebase uses —
/// see `backup.rs`'s module doc for why "save to an online drive" means
/// "save to a folder your cloud provider happens to sync," not a real
/// upload-API integration.
///
/// Desktop only — same `rfd` mobile gap as `spawn_attach_status_image`.
/// (An earlier draft of this comment claimed the Storage tab itself was
/// already desktop-only, so this needed no separate gate — that was
/// wrong, caught on review: the tab is available on every platform, so
/// without the `#[cfg]` below this would have been reachable, and
/// broken, on mobile.)
fn spawn_create_backup(ui: UiState) {
    let seed_phrase = ui.backup_seed_input.cloned();
    let passphrase = ui.backup_passphrase_input.cloned();
    ui.backup_error.clone().set(None);
    ui.backup_busy.clone().set(true);
    spawn(async move {
        let data_dir = siar_core::CONFIG.get().unwrap().data_dir.clone();
        let result = tokio::task::spawn_blocking(move || {
            siar_core::backup::create_backup(&data_dir, &seed_phrase, &passphrase)
        })
        .await;
        ui.backup_busy.clone().set(false);
        let bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(error)) => return ui.backup_error.clone().set(Some(error.to_string())),
            Err(error) => {
                return ui
                    .backup_error
                    .clone()
                    .set(Some(format!("backup task failed: {error}")))
            }
        };

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let Some(handle) = rfd::AsyncFileDialog::new()
                .set_file_name("siar-backup.siarbackup")
                .save_file()
                .await
            else {
                return; // cancelled — the backup was still computed for nothing, but nothing was written either, which is the safe side to fail on
            };
            if let Err(e) = handle.write(&bytes).await {
                ui.backup_error
                    .clone()
                    .set(Some(format!("couldn't save the backup file: {e}")));
            } else {
                ui.backup_seed_input.clone().set(String::new());
                ui.backup_passphrase_input.clone().set(String::new());
                push_toast(ui, "Backup saved".to_string(), false);
            }
        }

        #[cfg(target_os = "android")]
        {
            save_file_with_android_picker(
                "siar-backup.siarbackup",
                "application/octet-stream",
                &bytes,
            )
            .await;
            ui.backup_seed_input.clone().set(String::new());
            ui.backup_passphrase_input.clone().set(String::new());
            push_toast(
                ui,
                "Choose where to save your encrypted backup".to_string(),
                false,
            );
        }

        #[cfg(target_os = "ios")]
        ui.backup_error.clone().set(Some(
            "Backup export is not available on iOS yet".to_string(),
        ));
    });
}

#[cfg(target_os = "android")]
async fn save_file_with_android_picker(file_name: &str, mime: &str, bytes: &[u8]) {
    let encoded = data_encoding::BASE64.encode(bytes);
    let script = format!(
        "window.SiarAndroid?.saveFile({:?}, {:?}, {:?})",
        file_name, mime, encoded
    );
    let _ = document::eval(&script).await;
}

fn spawn_change_avatar(ui: UiState) {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        // Same rfd limitation as `spawn_attach_file` — see its comment.
        push_toast(
            ui,
            "Setting an avatar isn't wired up on mobile yet".to_string(),
            true,
        );
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    spawn(async move {
        let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "jxl"])
            .pick_file()
            .await
        else {
            return;
        };
        spawn_set_avatar(ui, handle.read().await);
    });
}

fn spawn_set_avatar(ui: UiState, raw: Vec<u8>) {
    spawn(async move {
        let cache_dir = CONFIG.get().unwrap().data_dir.join("avatar_cache");
        let result = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            core.set_my_avatar(&raw, &cache_dir).await
        };
        match result {
            Ok(png_bytes) => {
                let hash = ui.core.read().as_ref().and_then(Core::my_avatar_hash);
                if let Some(hash) = hash {
                    load_avatar_into_cache(ui, &hash, &png_bytes);
                    ui.my_avatar_hash.clone().set(Some(hash));
                }
            }
            Err(e) => push_toast(ui, format!("couldn't set avatar: {e}"), true),
        }
    });
}

fn spawn_attach_file(ui: UiState) {
    let Some(key) = ui.active.read().clone() else {
        return;
    };

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    spawn(async move {
        let Some(path) = rfd::AsyncFileDialog::new().pick_file().await else {
            return;
        };
        let path = path.path().to_path_buf();

        let blobs = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            core.blobs()
        };

        let prepared = match siar_core::net::transfer::prepare_outgoing(&blobs, &path).await {
            Ok(p) => p,
            Err(e) => return push_toast(ui, format!("couldn't prepare file: {e}"), true),
        };
        send_prepared_file(ui, key, prepared).await;
    });

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = (ui, key);
}

fn spawn_attach_file_bytes(ui: UiState, name: String, bytes: Vec<u8>) {
    let Some(key) = ui.active.read().clone() else {
        return;
    };
    spawn(async move {
        let blobs = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            core.blobs()
        };
        let prepared = match siar_core::net::transfer::prepare_outgoing_bytes(&blobs, &name, bytes)
            .await
        {
            Ok(prepared) => prepared,
            Err(error) => return push_toast(ui, format!("couldn't prepare file: {error}"), true),
        };
        send_prepared_file(ui, key, prepared).await;
    });
}

async fn send_prepared_file(
    ui: UiState,
    key: ConvKey,
    prepared: siar_core::net::transfer::PreparedFile,
) {
    let (my_name, endpoint) = {
        let core_ref = ui.core.read();
        let Some(core) = core_ref.as_ref() else {
            return;
        };
        (core.my_name.clone(), core.endpoint())
    };

    let envelope = Envelope::file(
        &my_name,
        &prepared.name,
        &prepared.mime,
        prepared.size_bytes,
        prepared.compressed,
        prepared.hash.to_string(),
        None,
    );

    ui.conversations
        .clone()
        .write()
        .entry(key.clone())
        .or_default()
        .push(StoredBubble {
            id: Some(envelope.id),
            kind: BubbleKind::Own { acked: false },
            sender: "me".to_string(),
            content: StoredContent::File {
                hash: prepared.hash.to_string(),
                name: prepared.name.clone(),
                size_bytes: prepared.size_bytes,
                state: FileState::Idle,
                compressed: prepared.compressed,
                from: None, // our own send — nothing to fetch, see field doc
            },
            sent_unix_ms: envelope.sent_unix_ms as i64,
            expires_at_unix_ms: None, // file-attachment expiry isn't wired up yet, see App::log_outgoing_file
            reactions: Vec::new(),
            edited: false,
            deleted: false,
            reply_to_envelope_id: None,
        });

    match &key {
        ConvKey::Dm(peer) => {
            let session = {
                let core_ref = ui.core.read();
                core_ref.as_ref().and_then(|c| c.existing_dm_session(*peer))
            };
            let session = match session {
                Some(s) => s,
                None => match app::connect_with_retry(&endpoint, *peer).await {
                    Ok(s) => s,
                    Err(e) => return push_toast(ui, format!("connect failed: {e}"), true),
                },
            };
            if let Err(e) = session.send(&envelope).await {
                return push_toast(ui, format!("file send failed: {e}"), true);
            }
            if let Some(core) = ui.core.read().as_ref() {
                let _ = core.log_outgoing_file(
                    &Conversation::Dm(app::hex(*peer)),
                    &prepared,
                    now_ms(),
                    envelope.id,
                    None,
                );
            }
        }
        ConvKey::Room(name) => {
            let room = {
                let core_ref = ui.core.read();
                core_ref.as_ref().and_then(|c| c.existing_room(name))
            };
            if let Some(room) = room {
                if let Err(e) = room.broadcast(&envelope).await {
                    return push_toast(ui, format!("file broadcast failed: {e}"), true);
                }
                if let Some(core) = ui.core.read().as_ref() {
                    let _ = core.log_outgoing_file(
                        &Conversation::Room(name.clone()),
                        &prepared,
                        now_ms(),
                        envelope.id,
                        None,
                    );
                }
            }
        }
    }
}

/// Fetch an announced incoming file. Looks up the bubble by content hash
/// (unique per file) to find who to download from and whether it's
/// zstd-compressed, then drives `net::transfer::fetch_incoming` and
/// writes the result (saved path, or an error to retry) back onto that
/// same bubble.
fn spawn_download_file(ui: UiState, key: ConvKey, hash: String) {
    let found = {
        let convs = ui.conversations.read();
        convs.get(&key).and_then(|bubbles| {
            bubbles.iter().find_map(|b| match &b.content {
                StoredContent::File {
                    hash: h,
                    from: Some(from),
                    compressed,
                    name,
                    ..
                } if h == &hash => Some((*from, *compressed, name.clone())),
                _ => None,
            })
        })
    };
    let Some((from, compressed, name)) = found else {
        return;
    };

    set_file_state(ui, &key, &hash, FileState::Downloading);

    spawn(async move {
        let (blobs, endpoint) = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            (core.blobs(), core.endpoint())
        };
        // VERIFY: assumes `iroh_blobs::Hash` implements `FromStr` parsing
        // its own `Display` (hex) form — very likely for a BLAKE3-style
        // content hash type, not directly confirmed against the pinned
        // version.
        let hash_parsed = match hash.parse() {
            Ok(h) => h,
            Err(_) => {
                return set_file_state(
                    ui,
                    &key,
                    &hash,
                    FileState::Failed("malformed hash".to_string()),
                )
            }
        };
        let dest_dir = CONFIG.get().unwrap().data_dir.join("downloads");
        match siar_core::net::transfer::fetch_incoming(
            &blobs,
            &endpoint,
            from,
            hash_parsed,
            &name,
            compressed,
            &dest_dir,
        )
        .await
        {
            Ok(path) => {
                set_file_state(ui, &key, &hash, FileState::Done(path.display().to_string()))
            }
            Err(e) => set_file_state(ui, &key, &hash, FileState::Failed(e.to_string())),
        }
    });
}

fn set_file_state(ui: UiState, key: &ConvKey, hash: &str, new_state: FileState) {
    if let Some(bubbles) = ui.conversations.clone().write().get_mut(key) {
        for b in bubbles.iter_mut() {
            if let StoredContent::File { hash: h, state, .. } = &mut b.content {
                if h == hash {
                    *state = new_state;
                    break;
                }
            }
        }
    }
}

// ---- Registry search / contact requests ----

fn check_username(ui: UiState, name: String) {
    // Onboarding runs before `Core` exists, so this needs its own
    // short-lived registry handle — mirrors what `Core::start` does
    // internally, just without the rest of the app spun up yet.
    ui.username_available.clone().set(None);
    spawn(async move {
        // In a full build this opens a throwaway `Registry` (or, more
        // efficiently, the onboarding flow could start `Core` early with a
        // temporary random identity purely to query the registry before
        // the real seed-derived identity is finalized). Left as a
        // `// TODO` rather than guessed at, since it depends on exactly
        // how heavy `Registry::join` is in practice.
        let _ = name;
        ui.username_available.clone().set(Some(true));
    });
}

fn spawn_search(ui: UiState, query: String) {
    if query.trim().is_empty() {
        ui.search_results.clone().set(vec![]);
        return;
    }
    spawn(async move {
        let (registry, contacts, my_username) = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            (
                core.registry(),
                core.accepted_contacts().unwrap_or_default(),
                core.my_username.clone(),
            )
        };
        match registry.search_prefix(&query).await {
            Ok(names) => {
                let results = names
                    .into_iter()
                    // Our own claim lives in the local replica too (we wrote
                    // it) — don't show ourselves as a search result.
                    .filter(|n| my_username.as_deref() != Some(n.as_str()))
                    .map(|n| {
                        let is_contact = contacts
                            .iter()
                            .any(|c| c.username.as_deref() == Some(n.as_str()));
                        (n, is_contact)
                    })
                    .collect();
                ui.search_results.clone().set(results);
            }
            Err(e) => push_toast(ui, format!("search failed: {e}"), true),
        }
    });
}

fn spawn_send_request(ui: UiState, username: String) {
    if ui.connecting.cloned() {
        return;
    }
    ui.connecting.clone().set(true);
    spawn(async move {
        let registry = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                ui.connecting.clone().set(false);
                return;
            };
            core.registry()
        };
        match registry.resolve(&username).await {
            Ok(Some(record)) => {
                // Prefer the record's own embedded ticket — it carries the
                // claimer's relay/direct addresses (see
                // `net::registry::UsernameRecord::new`), same as a pasted
                // ticket, so this connect doesn't have to wait on
                // discovery either. Fall back to the bare id (old records,
                // or a corrupt/empty ticket string) via the plain
                // `request_contact` path.
                let addr = siar_core::ticket::decode(&record.ticket).ok();
                let result = {
                    let core_ref = ui.core.read();
                    match (core_ref.as_ref(), addr) {
                        (Some(core), Some(addr)) => {
                            Some(core.request_contact_via_addr(addr, "").await)
                        }
                        (Some(core), None) => match EndpointId::from_bytes(&record.endpoint_id) {
                            Ok(peer) => Some(core.request_contact(peer, "").await),
                            Err(e) => Some(Err(anyhow::anyhow!("bad registry record: {e}"))),
                        },
                        (None, _) => None,
                    }
                };
                ui.connecting.clone().set(false);
                match result {
                    Some(Ok(())) => push_toast(ui, format!("request sent to @{username}"), false),
                    Some(Err(e)) => push_toast(ui, format!("request failed: {e}"), true),
                    None => {}
                }
            }
            Ok(None) => {
                ui.connecting.clone().set(false);
                push_toast(ui, format!("@{username} not found"), true)
            }
            Err(e) => {
                ui.connecting.clone().set(false);
                push_toast(ui, format!("lookup failed: {e}"), true)
            }
        }
    });
}

fn spawn_join_room(ui: UiState, input: String) {
    let input = input.trim().to_string();
    if input.is_empty() {
        return;
    }
    ui.room_input.clone().set(String::new());

    // The room-name box doubles as a ticket box: pasting a room ticket
    // (see `ticket::encode_room`'s doc for why a name alone isn't enough
    // to actually connect two independent devices) gives us both the
    // real room name and a peer to bootstrap from, instead of joining
    // cold with an empty bootstrap list — which is the actual bug behind
    // "creates a new room instead of joining the existing one": two
    // people typing the same name each ended up alone on their own copy
    // of that topic, with no peer to dial and no way to find one.
    let (name, bootstrap_addr) = match siar_core::ticket::decode_room(&input) {
        Ok((name, host)) => (name, Some(host)),
        Err(_) => (input, None),
    };
    let typed_plain_name = bootstrap_addr.is_none(); // no ticket pasted → likely creating fresh

    spawn(async move {
        let (gossip, existing, room_tx) = {
            let core_ref = ui.core.read();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            (core.gossip(), core.existing_room(&name), core.room_tx())
        };

        if existing.is_none() {
            // The actual fix: give gossip a real `EndpointId` to bootstrap
            // from (from the ticket) instead of the empty list this used
            // to always pass — that's the whole bug (see this function's
            // top comment). Resolving that id to an address is then the
            // endpoint's own job via its already-configured N0 discovery
            // (DNS/pkarr) — the exact same bare-id-plus-discovery path
            // `app::connect_with_retry` already relies on successfully
            // for contacts elsewhere in this codebase, so it doesn't need
            // its own separate proof here. (Two earlier attempts at also
            // manually pre-seeding the address — `Endpoint::add_node_addr`,
            // then a `StaticProvider` discovery service — both hit APIs
            // that don't exist in the pinned iroh version; dropped rather
            // than guess a third time. If cold-start discovery latency
            // turns out to matter in practice, that's the thing to add
            // back once the correct current API is actually confirmed
            // against a real build, not before.)
            let bootstrap_ids: Vec<iroh::EndpointId> = bootstrap_addr
                .as_ref()
                .map(|a| vec![a.id])
                .unwrap_or_default();

            match app::join_room_with_retry(&gossip, &name, bootstrap_ids, room_tx).await {
                Ok(room) => {
                    if let Some(core) = ui.core.clone().write().as_mut() {
                        core.commit_room(&name, room);
                    }
                }
                Err(e) => return push_toast(ui, format!("couldn't join #{name}: {e}"), true),
            }

            // Metadata (title + membership announcement) is a separate,
            // iroh-docs-backed concern from the gossip join above — see
            // `app::ensure_room_metadata_standalone`. Cheap handles are
            // pulled out under a fresh read-lock and the network-bound
            // work runs with no UI lock held at all (never await while
            // holding `ui.core`'s write guard — see `App::room_doc`'s
            // doc); only the final commit re-takes the lock, briefly and
            // synchronously. Best-effort: a failure here (e.g. no relay
            // yet) shouldn't block using the room itself.
            let handles = {
                let core_ref = ui.core.read();
                core_ref.as_ref().map(|core| {
                    (
                        core.docs(),
                        core.blobs_store(),
                        core.docs_author(),
                        core.my_id,
                        core.my_name.clone(),
                    )
                })
            };
            if let Some((docs, blobs_store, docs_author, my_id, my_name)) = handles {
                let docs_bootstrap = bootstrap_addr.clone().into_iter().collect();
                match app::ensure_room_metadata_standalone(
                    docs,
                    blobs_store,
                    docs_author,
                    my_id,
                    my_name,
                    name.clone(),
                    docs_bootstrap,
                )
                .await
                {
                    Ok(room_doc) => {
                        if let Some(core) = ui.core.clone().write().as_mut() {
                            core.commit_room_doc(&name, room_doc);
                        }
                    }
                    Err(e) => {
                        tracing::warn!(room = %name, error = %e, "couldn't sync room metadata")
                    }
                }
            }

            // Typed a plain name with nobody to join from (as opposed to
            // pasting someone else's ticket) means this is very likely a
            // brand-new room — hand back a ticket to actually invite
            // people with, instead of leaving them to discover the hard
            // way that just telling a friend the name doesn't connect
            // them to anything.
            if typed_plain_name {
                if let Some(core) = ui.core.read().as_ref() {
                    if let Ok(ticket) = siar_core::ticket::encode_room(&name, core.my_addr()) {
                        copy_to_clipboard(ticket);
                        push_toast(
                            ui,
                            format!("#{name} created — invite ticket copied, paste it to whoever you want in the room (the name alone won't connect them)"),
                            false,
                        );
                    }
                }
            }
        }

        if !ui.rooms.read().iter().any(|r| r == &name) {
            ui.rooms.clone().write().push(name.clone());
        }
        select_conversation(ui, ConvKey::Room(name));
    });
}

/// Open the conversation-info drawer for `key` and (re)load its
/// `net::conv_docs` metadata — room title/membership or DM settings. Also
/// used to refresh the panel after a save/toggle action below.
///
/// These calls hold `ui.core`'s write-lock across the metadata read/write
/// `.await` (unlike the read-lock-across-await pattern `spawn_send_request`
/// uses for network calls that don't touch `App`'s own fields). That's an
/// accepted, bounded tradeoff here: this only runs when the user explicitly
/// opens/edits the info panel (not on the message-send hot path), and by
/// the time it's called the room/DM doc has normally already been opened
/// once (via `spawn_join_room`'s `ensure_room_metadata_standalone`, or the
/// `DmDoc::open` call below), so the awaited work is a fast local doc
/// query rather than a fresh network round trip.
fn spawn_open_conv_info(ui: UiState, key: ConvKey) {
    ui.show_conv_info.clone().set(true);
    ui.room_info.clone().set(None);
    ui.dm_info.clone().set(None);
    ui.conv_info_input.clone().set(String::new());
    spawn(async move {
        match key {
            ConvKey::Room(name) => {
                let handles = {
                    let core_ref = ui.core.read();
                    core_ref.as_ref().map(|core| {
                        (
                            core.docs(),
                            core.blobs_store(),
                            core.docs_author(),
                            core.my_id,
                            core.my_name.clone(),
                        )
                    })
                };
                let Some((docs, blobs_store, docs_author, my_id, my_name)) = handles else {
                    return;
                };
                let room_doc = match app::ensure_room_metadata_standalone(
                    docs,
                    blobs_store,
                    docs_author,
                    my_id,
                    my_name,
                    name.clone(),
                    vec![],
                )
                .await
                {
                    Ok(d) => d,
                    Err(e) => return push_toast(ui, format!("couldn't load room info: {e}"), true),
                };

                // Read straight off the handle we already have — no
                // `App`/`Signal` touched during either await, so nothing
                // here can race a concurrent Signal borrow elsewhere (this
                // is what the `AlreadyBorrowedMut` panic on opening this
                // panel used to come from; see the removed methods' doc
                // comment in app.rs for the full story).
                let loaded = Some((room_doc.meta().await, room_doc.list_members().await));

                // Only now, with both reads already done, hand the
                // open handle to the cache — a synchronous insert, no
                // await, so briefly holding the write lock here is fine.
                if let Some(core) = ui.core.clone().write().as_mut() {
                    core.commit_room_doc(&name, room_doc);
                }

                match loaded {
                    Some((Ok(Some(meta)), Ok(members))) => {
                        ui.conv_info_input.clone().set(meta.title.clone());
                        ui.room_info.clone().set(Some((meta, members)));
                    }
                    Some((Ok(None), _)) => {} // ensure_meta above should always have set this
                    Some((Err(e), _)) | Some((_, Err(e))) => {
                        push_toast(ui, format!("couldn't load room info: {e}"), true)
                    }
                    None => {}
                }
            }
            ConvKey::Dm(peer) => {
                let handles = {
                    let core_ref = ui.core.read();
                    core_ref.as_ref().map(|core| {
                        (
                            core.docs(),
                            core.blobs_store(),
                            core.docs_author(),
                            core.my_id,
                        )
                    })
                };
                let Some((docs, blobs_store, docs_author, my_id)) = handles else {
                    return;
                };
                let dm_doc = match DmDoc::open(&docs, blobs_store, docs_author, my_id, peer).await {
                    Ok(d) => d,
                    Err(e) => {
                        return push_toast(
                            ui,
                            format!("couldn't load conversation info: {e}"),
                            true,
                        )
                    }
                };

                let settings = Some(dm_doc.settings().await);

                if let Some(core) = ui.core.clone().write().as_mut() {
                    core.commit_dm_doc(peer, dm_doc);
                }

                match settings {
                    Some(Ok(settings)) => {
                        ui.conv_info_input
                            .clone()
                            .set(settings.conversation_title.clone().unwrap_or_default());
                        ui.dm_settings_cache
                            .clone()
                            .write()
                            .insert(peer, settings.clone());
                        ui.dm_info.clone().set(Some(settings));
                    }
                    Some(Err(e)) => {
                        push_toast(ui, format!("couldn't load conversation info: {e}"), true)
                    }
                    None => {}
                }
            }
        }
    });
}

/// Snapshot the small set of cloneable client handles every standalone
/// `RoomDoc`/`DmDoc::open` needs, under a `ui.core.read()` guard that's
/// dropped immediately after — never held across the actual open/mutate
/// `.await` that follows. See `spawn_open_conv_info`'s doc comment for why
/// this matters (`AlreadyBorrowedMut` otherwise).
fn snapshot_doc_handles(
    ui: UiState,
) -> Option<(Docs, iroh_blobs::api::Store, AuthorId, EndpointId)> {
    let core_ref = ui.core.read();
    core_ref.as_ref().map(|core| {
        (
            core.docs(),
            core.blobs_store(),
            core.docs_author(),
            core.my_id,
        )
    })
}

fn spawn_set_room_title(ui: UiState, name: String, title: String) {
    let title = title.trim().to_string();
    if title.is_empty() {
        return;
    }
    spawn(async move {
        let Some((docs, blobs_store, docs_author, _my_id)) = snapshot_doc_handles(ui) else {
            return;
        };
        let result = match RoomDoc::open(&docs, blobs_store, docs_author, &name, vec![]).await {
            Ok(room_doc) => room_doc.set_title(&title).await,
            Err(e) => Err(e),
        };
        match result {
            Ok(()) => spawn_open_conv_info(ui, ConvKey::Room(name)),
            Err(e) => push_toast(ui, format!("couldn't rename room: {e}"), true),
        }
    });
}

fn spawn_remove_room_member(ui: UiState, name: String, target: [u8; 32]) {
    spawn(async move {
        let target = match EndpointId::from_bytes(&target) {
            Ok(id) => id,
            Err(e) => return push_toast(ui, format!("bad member id: {e}"), true),
        };
        let Some((docs, blobs_store, docs_author, my_id)) = snapshot_doc_handles(ui) else {
            return;
        };
        let result = match RoomDoc::open(&docs, blobs_store, docs_author, &name, vec![]).await {
            Ok(room_doc) => room_doc.remove_member(target, my_id).await,
            Err(e) => Err(e),
        };
        match result {
            Ok(()) => spawn_open_conv_info(ui, ConvKey::Room(name)),
            Err(e) => push_toast(ui, format!("couldn't remove member: {e}"), true),
        }
    });
}

fn spawn_set_dm_title(ui: UiState, peer: EndpointId, title: Option<String>) {
    spawn(async move {
        let Some((docs, blobs_store, docs_author, my_id)) = snapshot_doc_handles(ui) else {
            return;
        };
        let result = match DmDoc::open(&docs, blobs_store, docs_author, my_id, peer).await {
            Ok(dm_doc) => dm_doc.set_title(title.as_deref()).await,
            Err(e) => Err(e),
        };
        match result {
            Ok(()) => spawn_open_conv_info(ui, ConvKey::Dm(peer)),
            Err(e) => push_toast(ui, format!("couldn't save nickname: {e}"), true),
        }
    });
}

fn spawn_set_dm_pinned(ui: UiState, peer: EndpointId, pinned: bool) {
    spawn(async move {
        let Some((docs, blobs_store, docs_author, my_id)) = snapshot_doc_handles(ui) else {
            return;
        };
        let result = match DmDoc::open(&docs, blobs_store, docs_author, my_id, peer).await {
            Ok(dm_doc) => dm_doc.set_pinned(pinned).await,
            Err(e) => Err(e),
        };
        match result {
            Ok(()) => spawn_open_conv_info(ui, ConvKey::Dm(peer)),
            Err(e) => push_toast(ui, format!("couldn't update: {e}"), true),
        }
    });
}

fn spawn_set_dm_archived(ui: UiState, peer: EndpointId, archived: bool) {
    spawn(async move {
        let Some((docs, blobs_store, docs_author, my_id)) = snapshot_doc_handles(ui) else {
            return;
        };
        let result = match DmDoc::open(&docs, blobs_store, docs_author, my_id, peer).await {
            Ok(dm_doc) => dm_doc.set_archived(archived).await,
            Err(e) => Err(e),
        };
        match result {
            Ok(()) => spawn_open_conv_info(ui, ConvKey::Dm(peer)),
            Err(e) => push_toast(ui, format!("couldn't update: {e}"), true),
        }
    });
}

fn spawn_set_dm_disappearing_ttl(ui: UiState, peer: EndpointId, ttl_secs: Option<u64>) {
    spawn(async move {
        let Some((docs, blobs_store, docs_author, my_id)) = snapshot_doc_handles(ui) else {
            return;
        };
        let result = match DmDoc::open(&docs, blobs_store, docs_author, my_id, peer).await {
            Ok(dm_doc) => dm_doc.set_disappearing_ttl(ttl_secs).await,
            Err(e) => Err(e),
        };
        match result {
            Ok(()) => spawn_open_conv_info(ui, ConvKey::Dm(peer)),
            Err(e) => push_toast(ui, format!("couldn't update: {e}"), true),
        }
    });
}

fn spawn_toggle_verified(ui: UiState, peer: EndpointId) {
    // Same "synchronous, no lock-vs-await care needed" situation as
    // `spawn_block_contact` — just a sqlite UPDATE.
    let currently_verified = {
        let hex = app::hex(peer);
        ui.contacts
            .read()
            .iter()
            .any(|c| c.endpoint_id == hex && c.verified)
    };
    let result = {
        let core_ref = ui.core.read();
        core_ref
            .as_ref()
            .map(|core| core.set_contact_verified(peer, !currently_verified))
    };
    match result {
        Some(Ok(())) => {
            if let Some(core) = ui.core.read().as_ref() {
                refresh_contacts(ui, core);
            }
        }
        Some(Err(e)) => push_toast(ui, format!("couldn't update verification: {e}"), true),
        None => {}
    }
}

fn spawn_block_contact(ui: UiState, peer: EndpointId) {
    // `block_contact` just flips a row in sqlite — synchronous, so this
    // doesn't need the network-await-vs-lock care the room/DM metadata
    // spawns above need (see `spawn_open_conv_info`'s doc comment).
    let result = {
        let core_ref = ui.core.read();
        core_ref.as_ref().map(|core| core.block_contact(peer))
    };
    match result {
        Some(Ok(())) => {
            ui.show_conv_info.clone().set(false);
            ui.room_info.clone().set(None);
            ui.dm_info.clone().set(None);
            if ui.active.read().as_ref() == Some(&ConvKey::Dm(peer)) {
                ui.active.clone().set(None);
            }
            if let Some(core) = ui.core.read().as_ref() {
                refresh_contacts(ui, core);
            }
            push_toast(ui, format!("blocked {}", app::short_id(peer)), false);
        }
        Some(Err(e)) => push_toast(ui, format!("couldn't block contact: {e}"), true),
        None => {}
    }
}

/// Fire-and-forget: record `peer` as a registry bootstrap candidate and
/// give the already-open registry replica a live hint about them (see
/// `App::remember_and_hint_peer`). Used whenever a peer reaches *us*
/// directly, since `App::request_contact`/`accept_contact` already handle
/// the outgoing-connection cases themselves.
fn spawn_remember_peer(ui: UiState, peer: EndpointId) {
    spawn(async move {
        if let Some(core) = ui.core.read().as_ref() {
            core.remember_and_hint_peer(peer).await;
        }
    });
}

/// Place a voice call to `peer`. Blocks (inside its own spawned task) for
/// the whole call — progress and outcome arrive as `AppEvent::Call`
/// events on the normal event pump, same as an incoming call, so
/// `handle_app_event` is the single place that reacts to either
/// direction.
fn spawn_call_peer(ui: UiState, peer: EndpointId) {
    spawn_call_peer_with(ui, peer, false);
}

fn spawn_video_call_peer(ui: UiState, peer: EndpointId) {
    spawn(async move {
        let has_camera =
            tokio::task::spawn_blocking(siar_core::net::calls::video::camera_available)
                .await
                .unwrap_or(false);
        if !has_camera {
            push_toast(
                ui,
                "no camera found — starting a voice call instead".to_string(),
                false,
            );
        }
        spawn_call_peer_with(ui, peer, has_camera);
    });
}

fn spawn_call_peer_with(ui: UiState, peer: EndpointId, want_video: bool) {
    spawn(async move {
        if ensure_android_audio_permission().await {
            begin_call_peer_with(ui, peer, want_video);
        } else {
            push_toast(
                ui,
                "Allow microphone access, then tap call again".to_string(),
                false,
            );
        }
    });
}

fn begin_call_peer_with(ui: UiState, peer: EndpointId, want_video: bool) {
    if ui.active_call.cloned().is_some()
        || ui.incoming_call.cloned().is_some()
        || ui.outgoing_call.cloned().is_some()
    {
        return push_toast(ui, "already on a call".to_string(), true);
    }
    let name = ui
        .contacts
        .read()
        .iter()
        .find(|c| c.endpoint_id == app::hex(peer))
        .map(|c| c.alias.clone())
        .unwrap_or_else(|| app::short_id(peer));
    ui.active_call_direction
        .clone()
        .set(Some(CallDirection::Outgoing));
    // Set right away, before the network round trip even starts — this is
    // the fix for the caller seeing nothing while the phone "rings" on the
    // other end. Cleared on Connected (flips to `active_call`) or on
    // Ended/failure below.
    ui.outgoing_call.clone().set(Some((peer, name)));
    ui.active_ringtone
        .clone()
        .write()
        .replace(siar_core::ringtone::Ringtone::start(true));
    spawn(async move {
        let handles = {
            let core_ref = ui.core.read();
            core_ref.as_ref().map(|core| {
                (
                    core.endpoint(),
                    core.my_id,
                    core.my_name.clone(),
                    core.call_events_sender(),
                )
            })
        };
        let Some((endpoint, my_id, my_name, events)) = handles else {
            ui.active_ringtone.clone().write().take();
            ui.outgoing_call.clone().set(None);
            return;
        };

        let (hangup_tx, hangup_rx) = tokio::sync::oneshot::channel();
        if let Some(core) = ui.core.clone().write().as_mut() {
            core.set_active_call_hangup(hangup_tx);
        }

        if let Err(e) = siar_core::net::calls::place_call(
            &endpoint, peer, my_id, &my_name, want_video, events, hangup_rx,
        )
        .await
        {
            // `place_call` only ever sends `CallEvent::Ended` (which does
            // the logging in `handle_app_event`) once it's actually
            // reached the point of dialing successfully — a connect
            // failure/timeout returns `Err` straight from here instead,
            // so this path needs its own log entry rather than relying on
            // an event that was never sent.
            let name = ui
                .contacts
                .read()
                .iter()
                .find(|c| c.endpoint_id == app::hex(peer))
                .map(|c| c.alias.clone())
                .unwrap_or_else(|| app::short_id(peer));
            if let Some(core) = ui.core.read().as_ref() {
                let _ = core.log_call(
                    peer,
                    &name,
                    CallDirection::Outgoing,
                    CallOutcome::Failed,
                    now_ms(),
                    0,
                );
            }
            refresh_call_log(ui);
            ui.active_ringtone.clone().write().take();
            ui.active_call.clone().set(None);
            ui.outgoing_call.clone().set(None);
            ui.active_call_direction.clone().set(None);
            push_toast(ui, format!("call failed: {e}"), true);
        }
    });
}

fn spawn_answer_call(ui: UiState, accept: bool) {
    let decision = ui.incoming_call_decision.clone().write().take();
    if accept {
        spawn(async move {
            let permitted = ensure_android_audio_permission().await;
            if let Some(tx) = decision {
                let _ = tx.send(permitted);
            }
            if !permitted {
                ui.active_ringtone.clone().write().take();
                ui.incoming_call.clone().set(None);
                push_toast(
                    ui,
                    "Microphone permission is required to answer".to_string(),
                    true,
                );
            }
        });
        return;
    }
    if let Some(tx) = decision {
        let _ = tx.send(false);
    }
    if !accept {
        ui.active_ringtone.clone().write().take();
        ui.incoming_call.clone().set(None);
    }
    // On accept, `incoming_call` stays set until `CallEvent::Connected`
    // arrives and flips it over to `active_call` — see `handle_app_event`
    // — so there's a brief "connecting" moment rather than the ring
    // banner just vanishing before the call bar appears.
}

fn spawn_hang_up(ui: UiState) {
    if let Some(core) = ui.core.clone().write().as_mut() {
        core.hang_up_active_call();
    }
    ui.active_ringtone.clone().write().take();
    ui.active_call.clone().set(None);
    ui.outgoing_call.clone().set(None);
}

fn spawn_connect_ticket(ui: UiState, ticket_str: String) {
    // Guard against the exact thing in the bug report: no loading/disabled
    // state on the button meant an impatient extra click (very reasonable,
    // given a multi-attempt retry can take tens of seconds) fired a second,
    // fully independent attempt — each with its own eventual toast, hence
    // the stack of identical "request failed" toasts. One in flight at a
    // time; see `ui.connecting` and `Sidebar`'s use of it for the
    // "Connecting…" state this now shows instead.
    if ui.connecting.cloned() {
        return;
    }
    ui.connecting.clone().set(true);
    spawn(async move {
        let addr = match siar_core::ticket::decode(&ticket_str) {
            Ok(a) => a,
            Err(e) => {
                ui.connecting.clone().set(false);
                return push_toast(ui, format!("invalid ticket: {e}"), true);
            }
        };
        let result = {
            let core_ref = ui.core.read();
            match core_ref.as_ref() {
                Some(core) => Some(core.request_contact_via_addr(addr, "").await),
                None => None,
            }
        };
        ui.connecting.clone().set(false);
        match result {
            Some(Ok(())) => push_toast(ui, "request sent".to_string(), false),
            Some(Err(e)) => push_toast(ui, format!("request failed: {e}"), true),
            None => {}
        }
    });
}

fn spawn_connect_qr_image(ui: UiState, bytes: Vec<u8>) {
    if ui.connecting.cloned() {
        return;
    }
    ui.connecting.clone().set(true);
    spawn(async move {
        let decoded = tokio::task::spawn_blocking(move || decode_connection_qr(&bytes)).await;
        let ticket = match decoded {
            Ok(Ok(ticket)) => ticket,
            Ok(Err(error)) => {
                ui.connecting.clone().set(false);
                return push_toast(ui, error.to_string(), true);
            }
            Err(error) => {
                ui.connecting.clone().set(false);
                return push_toast(
                    ui,
                    format!("QR scanner stopped unexpectedly: {error}"),
                    true,
                );
            }
        };

        // Show exactly what was scanned, then use the existing request path.
        // Clear `connecting` first because that path owns its own in-flight
        // guard and would otherwise (correctly) reject this hand-off.
        ui.search_query.clone().set(ticket.clone());
        ui.search_results.clone().set(Vec::new());
        ui.connecting.clone().set(false);
        spawn_connect_ticket(ui, ticket);
    });
}

fn decode_connection_qr(bytes: &[u8]) -> anyhow::Result<String> {
    use anyhow::{bail, Context};

    let image = image::load_from_memory(bytes)
        .context("couldn't decode that image; choose a PNG, JPEG, GIF, or WebP")?
        .to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(image);
    let grids = prepared.detect_grids();
    if grids.is_empty() {
        bail!("no QR code found; keep the full code in frame and try again");
    }

    let mut last_decode_error = None;
    for grid in grids {
        match grid.decode() {
            Ok((_metadata, content)) => {
                let ticket = content.trim();
                siar_core::ticket::decode(ticket)
                    .context("QR code is not a valid Siar connection ticket")?;
                return Ok(ticket.to_string());
            }
            Err(error) => last_decode_error = Some(error.to_string()),
        }
    }
    bail!(
        "QR code was found but couldn't be read{}",
        last_decode_error
            .map(|error| format!(": {error}"))
            .unwrap_or_default()
    )
}

fn spawn_accept(ui: UiState, endpoint_id_hex: String) {
    spawn(async move {
        let Ok(peer) = parse_hex(&endpoint_id_hex) else {
            return;
        };
        let result = {
            let core_ref = ui.core.read();
            match core_ref.as_ref() {
                Some(core) => Some(core.accept_contact(peer).await),
                None => None,
            }
        };
        if let Some(Err(e)) = result {
            push_toast(ui, format!("accept failed: {e}"), true);
        }
        if let Some(core) = ui.core.read().as_ref() {
            refresh_contacts(ui, core);
        }
        spawn_preload_dm_settings(ui);
    });
}

fn spawn_decline(ui: UiState, endpoint_id_hex: String) {
    spawn(async move {
        let Ok(peer) = parse_hex(&endpoint_id_hex) else {
            return;
        };
        let result = {
            let core_ref = ui.core.read();
            match core_ref.as_ref() {
                Some(core) => Some(core.reject_contact(peer).await),
                None => None,
            }
        };
        if let Some(Err(e)) = result {
            push_toast(ui, format!("decline failed: {e}"), true);
        }
        if let Some(core) = ui.core.read().as_ref() {
            refresh_contacts(ui, core);
        }
    });
}

fn parse_hex(s: &str) -> anyhow::Result<EndpointId> {
    let bytes = data_encoding::HEXLOWER.decode(s.as_bytes())?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("bad id length"))?;
    EndpointId::from_bytes(&arr).map_err(|e| anyhow::anyhow!("{e}"))
}

// ---- View helpers ----

fn build_chat_list(ui: UiState) -> Vec<ChatListEntry> {
    let mut entries = vec![];
    let online = ui.online.read();
    let dm_settings = ui.dm_settings_cache.read();
    for c in ui.contacts.read().iter() {
        let Ok(peer) = parse_hex(&c.endpoint_id) else {
            continue;
        };
        let key = ConvKey::Dm(peer);
        let bubbles = ui.conversations.read();
        let last = bubbles.get(&key).and_then(|v| v.last());
        let settings = dm_settings.get(&peer);
        let name = settings
            .and_then(|s| s.conversation_title.clone())
            .or_else(|| c.username.clone().map(|u| format!("@{u}")))
            .unwrap_or_else(|| c.alias.clone());
        entries.push(ChatListEntry {
            id: c.endpoint_id.clone(),
            key: key.clone(),
            name,
            preview: last.map(preview_text).unwrap_or_default(),
            time_label: last
                .map(|b| relative_time(b.sent_unix_ms))
                .unwrap_or_default(),
            unread: ui.unread.read().get(&key).copied().unwrap_or(0),
            online: online.contains(&peer),
            pinned: settings.is_some_and(|s| s.pinned),
            archived: settings.is_some_and(|s| s.archived),
            verified: c.verified,
            avatar_hash: c.avatar_hash.clone(),
        });
    }
    for name in ui.rooms.read().iter() {
        let key = ConvKey::Room(name.clone());
        let bubbles = ui.conversations.read();
        let last = bubbles.get(&key).and_then(|v| v.last());
        entries.push(ChatListEntry {
            id: format!("room:{name}"),
            key: key.clone(),
            name: format!("#{name}"),
            preview: last.map(preview_text).unwrap_or_default(),
            time_label: last
                .map(|b| relative_time(b.sent_unix_ms))
                .unwrap_or_default(),
            unread: ui.unread.read().get(&key).copied().unwrap_or(0),
            online: false, // rooms don't have a single presence state
            pinned: false, // no pin/archive concept for rooms in this design
            archived: false,
            verified: false,   // no identity-verification concept for rooms
            avatar_hash: None, // rooms don't have a single picture
        });
    }

    // Archived is a separate view, not merely a filter within the normal
    // list — the only path back to it is `Sidebar`'s "Archived" footer
    // toggle, which flips `ui.show_archived`. Pinned chats sort to the top
    // of whichever view is showing, WhatsApp/Signal-style; ties keep
    // insertion order (contacts first, then rooms) via a stable sort.
    let show_archived = ui.show_archived.cloned();
    entries.retain(|e| e.archived == show_archived);
    match ui.bottom_nav.cloned() {
        // Combined feed — both DMs and rooms together, most recently
        // active first (the existing sort below already interleaves by
        // activity regardless of kind). `Dms`/`Groups` are this same list
        // pre-filtered to one `ConvKey` variant, for anyone who'd rather
        // not have the two interleaved.
        BottomNav::Chats => {}
        BottomNav::Dms => entries.retain(|e| matches!(e.key, ConvKey::Dm(_))),
        BottomNav::Groups => entries.retain(|e| matches!(e.key, ConvKey::Room(_))),
        // Calls/Status render their own dedicated views entirely — see
        // where `bottom_nav` is matched in the sidebar's parent — so this
        // list isn't used there at all; returning it filtered-to-nothing
        // rather than plumbing an early return keeps this one function's
        // control flow simple.
        BottomNav::Calls | BottomNav::Status => entries.clear(),
    }
    // The search box already triggers `spawn_search` against the registry
    // to find *new* people to add (see `on_search_input`) — that's a
    // separate signal (`search_results`) rendered alongside this list.
    // This is the other half of "search": narrowing what's already in the
    // sidebar, the same way Telegram/WhatsApp show "chats" and "global"
    // results together. Case-insensitive substring match on the display
    // name; deliberately not matching message previews/content here, since
    // that'd mean scanning every conversation's history on every keystroke
    // — a real feature, but a different one (see BUILD_NOTES.md).
    let query = ui.search_query.cloned();
    if !query.trim().is_empty() {
        let q = query.trim().to_lowercase();
        entries.retain(|e| e.name.to_lowercase().contains(&q));
    }
    entries.sort_by_key(|e| !e.pinned);
    entries
}

fn title_for(ui: UiState, key: &ConvKey) -> (String, String) {
    match key {
        ConvKey::Dm(peer) => {
            let hex = app::hex(*peer);
            let contact = ui
                .contacts
                .read()
                .iter()
                .find(|c| c.endpoint_id == hex)
                .cloned();
            // A nickname set via `ConvInfoPanel` (`net::conv_docs::DmSettings`,
            // synced across your own devices) wins over the raw username —
            // same precedence WhatsApp/Signal give a locally-set contact
            // name over a phone number/handle.
            let nickname = ui
                .dm_settings_cache
                .read()
                .get(peer)
                .and_then(|s| s.conversation_title.clone());
            let name = nickname
                .or_else(|| {
                    contact
                        .as_ref()
                        .and_then(|c| c.username.clone())
                        .map(|u| format!("@{u}"))
                })
                .unwrap_or_else(|| app::short_id(*peer));
            let subtitle = if ui.online.read().contains(peer) {
                "online"
            } else {
                "offline"
            };
            (name, subtitle.to_string())
        }
        ConvKey::Room(name) => (format!("#{name}"), "group room".to_string()),
    }
}

fn bubbles_for(ui: UiState, key: &ConvKey) -> Vec<BubbleData> {
    let now_ms = now_ms();
    let watermark = ui.read_watermarks.read().get(key).copied();
    ui.conversations
        .read()
        .get(key)
        .map(|v| {
            // Resolves a reply's quote preview locally, from whatever's
            // already loaded for this conversation — see
            // `protocol::message::Body::Text::reply_to`'s doc: the target
            // message's own content isn't re-sent over the wire, only its
            // id, so this is the only place it can be recovered. Falls
            // back to a generic placeholder (handled below, not here) if
            // the target isn't in this window of history — e.g. it's
            // further back than `history`'s `limit`, or (for a room)
            // arrived before this device joined.
            let by_id: HashMap<u64, (String, String)> = v
                .iter()
                .filter_map(|b| b.id.map(|id| (id, (b.sender.clone(), preview_text(b)))))
                .collect();
            v.iter()
                .filter(|b| b.expires_at_unix_ms.is_none_or(|exp| exp > now_ms))
                .map(|b| BubbleData {
                    kind: b.kind.clone(),
                    sender: b.sender.clone(),
                    content: display_content(&b.content),
                    time_label: timestamp(b.sent_unix_ms),
                    id: b.id,
                    reactions: b.reactions.clone(),
                    edited: b.edited,
                    deleted: b.deleted,
                    read: matches!(b.kind, BubbleKind::Own { .. })
                        && watermark.is_some_and(|w| b.sent_unix_ms <= w),
                    reply_preview: b.reply_to_envelope_id.map(|target_id| {
                        by_id.get(&target_id).cloned().unwrap_or_else(|| {
                            (String::new(), "Original message unavailable".to_string())
                        })
                    }),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn display_content(content: &StoredContent) -> BubbleContent {
    match content {
        StoredContent::Text(t) => BubbleContent::Text(t.clone()),
        StoredContent::File {
            hash,
            name,
            size_bytes,
            state,
            from,
            ..
        } => BubbleContent::File {
            hash: hash.clone(),
            name: name.clone(),
            size_bytes: *size_bytes,
            state: state.clone(),
            fetchable: from.is_some() && matches!(state, FileState::Idle | FileState::Failed(_)),
        },
    }
}

fn preview_text(b: &StoredBubble) -> String {
    match &b.content {
        StoredContent::Text(t) => t.chars().take(60).collect(),
        StoredContent::File { name, .. } => format!("📎 {name}"),
    }
}

fn push_toast(ui: UiState, text: String, is_error: bool) {
    let id = now_ms() as u64;
    ui.toasts.clone().write().push((id, text, is_error));
    spawn(async move {
        tokio::time::sleep(Duration::from_secs(6)).await;
        ui.toasts.clone().write().retain(|(t_id, _, _)| *t_id != id);
    });
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn timestamp(unix_ms: i64) -> String {
    use chrono::TimeZone;
    chrono::Local
        .timestamp_millis_opt(unix_ms)
        .single()
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_default()
}

fn relative_time(unix_ms: i64) -> String {
    let now = now_ms();
    let diff_secs = (now - unix_ms) / 1000;
    match diff_secs {
        d if d < 60 => "now".to_string(),
        d if d < 3600 => format!("{}m", d / 60),
        d if d < 86400 => format!("{}h", d / 3600),
        d => format!("{}d", d / 86400),
    }
}

#[cfg(test)]
mod qr_scan_tests {
    use super::decode_connection_qr;
    use image::{DynamicImage, ImageFormat, Luma};
    use qrcode::QrCode;
    use std::io::Cursor;

    #[test]
    fn generated_connection_ticket_round_trips_through_qr_image() {
        let secret = iroh::SecretKey::generate();
        let ticket = siar_core::ticket::encode(iroh::EndpointAddr::from(secret.public())).unwrap();
        let qr = QrCode::new(ticket.as_bytes())
            .unwrap()
            .render::<Luma<u8>>()
            .min_dimensions(512, 512)
            .build();
        let mut png = Cursor::new(Vec::new());
        DynamicImage::ImageLuma8(qr)
            .write_to(&mut png, ImageFormat::Png)
            .unwrap();

        assert_eq!(decode_connection_qr(png.get_ref()).unwrap(), ticket);
    }

    #[test]
    fn unrelated_qr_is_rejected_before_connecting() {
        let qr = QrCode::new(b"https://example.invalid")
            .unwrap()
            .render::<Luma<u8>>()
            .min_dimensions(384, 384)
            .build();
        let mut png = Cursor::new(Vec::new());
        DynamicImage::ImageLuma8(qr)
            .write_to(&mut png, ImageFormat::Png)
            .unwrap();

        assert!(decode_connection_qr(png.get_ref())
            .unwrap_err()
            .to_string()
            .contains("not a valid Siar"));
    }
}
