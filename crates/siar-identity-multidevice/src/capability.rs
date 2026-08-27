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

    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(
        Self::SEND_MESSAGE.0
            | Self::RECEIVE_MESSAGE.0
            | Self::LINK_NEW_DEVICE.0
            | Self::REVOKE_DEVICE.0
            | Self::MANAGE_GROUPS.0,
    );

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}
