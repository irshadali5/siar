//! Newtype identifiers (plan.md §120).
//!
//! Every ID is a distinct type over the same underlying `Uuid` so that
//! passing a `DeviceId` where a `ConversationId` is expected is a compile
//! error, not a runtime bug, even though the wire representation is
//! identical.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! newtype_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[repr(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Generate a new random identifier.
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Wrap an existing UUID (e.g. one read back from storage).
            pub const fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }

            /// First 8 hex characters of the UUID — enough to
            /// distinguish devices/messages/etc at a glance in logs and
            /// CLI/UI output without printing a full 36-character UUID
            /// every time. Not guaranteed globally unique on its own
            /// (a UUID's full 128 bits are what guarantees that); this
            /// is a display convenience, not an identifier — nothing
            /// should compare two `fmt_short()` outputs for equality
            /// instead of comparing the underlying value.
            pub fn fmt_short(&self) -> String {
                let full = self.0.simple().to_string();
                full[..8].to_string()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        // Added while wiring the desktop group UI's add-member form,
        // which needs to parse a user-pasted account/device id string —
        // `apps/cli` has worked around this gap in every call site that
        // needed it (its own `parse_uuid` helper, confirmed via source:
        // "ID newtypes have no FromStr" was a known, deliberately
        // unfixed limitation up to this point) rather than the type
        // itself supporting it. Fixing it here closes the gap for every
        // ID type and every future call site at once, instead of adding
        // yet another local `Uuid::parse_str` wrapper in this crate too.
        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }
    };
}

newtype_id!(AccountId);
newtype_id!(DeviceId);
newtype_id!(ConversationId);
newtype_id!(MessageId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_not_interchangeable_at_the_type_level() {
        // This test exists to document the invariant, not to exercise
        // runtime behavior: the following would not compile if uncommented,
        // which is the point of the newtype wrappers.
        //
        // fn wants_conversation(_id: ConversationId) {}
        // wants_conversation(DeviceId::new()); // <- compile error

        let a = AccountId::new();
        let b = AccountId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn round_trips_through_postcard() {
        let id = MessageId::new();
        let bytes = postcard::to_allocvec(&id).unwrap();
        let back: MessageId = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn fmt_short_is_the_first_8_hex_chars_of_the_full_uuid() {
        let id = DeviceId::new();
        let short = id.fmt_short();
        assert_eq!(short.len(), 8);
        assert!(id.to_string().replace('-', "").starts_with(&short));
    }

    #[test]
    fn from_str_round_trips_through_display() {
        let id = AccountId::new();
        let parsed: AccountId = id.to_string().parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn from_str_rejects_garbage() {
        assert!("not-a-uuid".parse::<AccountId>().is_err());
    }
}
