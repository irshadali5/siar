//! §14 "Device Capability Set" — a minimal, initial set. The spec's own
//! §200 "Recommended Initial Implementation" doesn't enumerate specific
//! capability names, so these five are a reasonable starting set
//! covering operations this same spec discusses elsewhere (§25
//! revocation, §15 linking, ordinary messaging, group membership via
//! `siar-messaging::GroupService`) — extend, don't treat as exhaustive.

use serde::{Deserialize, Serialize};

/// A plain bitset (`u32`), not an open string set — Part 01's §9
/// "Strong Protocol Identifiers" gives the same reasoning against
/// arbitrary strings for hot authorization checks, which a per-device
/// capability check is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCapabilitySet(pub u32);

impl DeviceCapabilitySet {
    pub const SEND_MESSAGE: Self = Self(1 << 0);
    pub const RECEIVE_MESSAGE: Self = Self(1 << 1);
    pub const LINK_NEW_DEVICE: Self = Self(1 << 2);
    pub const REVOKE_DEVICE: Self = Self(1 << 3);
    pub const MANAGE_GROUPS: Self = Self(1 << 4);
    /// §64's own remaining three named capabilities, added this round
    /// — extending, not replacing, the original five (this module's
    /// own doc comment already invited that).
    pub const ROTATE_ACCOUNT_STATE: Self = Self(1 << 5);
    pub const SYNC_HISTORY: Self = Self(1 << 6);
    pub const RELAY: Self = Self(1 << 7);

    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(
        Self::SEND_MESSAGE.0
            | Self::RECEIVE_MESSAGE.0
            | Self::LINK_NEW_DEVICE.0
            | Self::REVOKE_DEVICE.0
            | Self::MANAGE_GROUPS.0
            | Self::ROTATE_ACCOUNT_STATE.0
            | Self::SYNC_HISTORY.0
            | Self::RELAY.0,
    );

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}
