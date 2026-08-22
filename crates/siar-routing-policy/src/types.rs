//! §5 "Main Abstractions" through §14 "Route Health".

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// §7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryClass {
    Realtime,
    Interactive,
    Reliable,
    Bulk,
    DelayTolerant,
}

/// §8. "Priority affects scheduling but does not override hard
/// constraints" — enforced in [`crate::scoring`], not here; this is
/// just the scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Background,
    Low,
    Normal,
    High,
    Critical,
}

/// §10. Every transport this workspace already has a bridge for
/// (`siar-transport`'s `SiarEndpoint` for Iroh direct/relay,
/// `apps/android`'s four transport-jni crates for the rest) gets a
/// variant here — this enum doesn't invent new transports, it names
/// the ones already real elsewhere in this workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportKind {
    IrohDirect,
    IrohRelay,
    LocalLan,
    WifiDirect,
    WifiAware,
    BluetoothClassic,
    BluetoothLe,
    MeshRelay,
    Dtn,
}

/// §11. "This prevents impossible route choices" — checked as a hard
/// constraint in [`crate::scoring::eliminate_hard_constraint_violations`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathCapabilities {
    pub reliable_stream: bool,
    pub datagram: bool,
    pub large_files: bool,
    pub realtime_media: bool,
    pub peer_discovery: bool,
    pub store_and_forward: bool,
    pub metered: bool,
}

/// §14. "Health is derived from: recent failures, timeouts, connection
/// churn, path changes, transport errors" — that derivation isn't
/// implemented here (it needs live transport feedback this crate
/// doesn't have a wire connection to; see this crate's own top doc
/// comment on scope); this is the value shape a caller updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteHealth {
    Healthy,
    Degraded,
    Suspect,
    Unreachable,
    Unknown,
}

/// §16. `Group(ConversationId)`, not `Group(GroupId)` as the spec's own
/// example names it — this workspace has no separate `GroupId` type;
/// `siar_domain::ConversationId` is already what
/// `siar-messaging::GroupService` addresses groups by (confirmed
/// against that crate's real `create_group_mls`/`send_text_mls`
/// signatures, not guessed), so reusing it here keeps this crate
/// consistent with the rest of the workspace instead of introducing a
/// second, redundant identifier for the same thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Destination {
    Account(siar_domain::AccountId),
    Device(siar_domain::DeviceId),
    Group(siar_domain::ConversationId),
}

/// §9's own field — a newtype so a `PathId` can never be silently
/// compared against an unrelated `Uuid`-backed id, same reasoning
/// `siar_domain::ids`' newtype macro doc comment already gives for
/// every id in that module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PathId(Uuid);

impl PathId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PathId {
    fn default() -> Self {
        Self::new()
    }
}
