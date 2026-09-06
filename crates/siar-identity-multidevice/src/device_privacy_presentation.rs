//! §145 "Privacy-Preserving Device Names": "friendly names may stay
//! local... remote peers do not need to know 'Irshad's Bedroom PC'
//! unless user explicitly shares it. Use generic capability
//! descriptors by default."

use crate::capability::DeviceCapabilitySet;

/// §145's own default — computed FROM a [`DeviceCapabilitySet`], never
/// from [`crate::discovery_privacy::PrivateDeviceMetadata::friendly_name`],
/// so there is no code path from "what capabilities this device has"
/// to "the literal string a human named it." A caller who only ever
/// calls this function cannot leak a friendly name by accident,
/// because this function never receives one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericDeviceDescriptor {
    MessagingDevice,
    FullCapabilityDevice,
    LimitedDevice,
}

impl GenericDeviceDescriptor {
    pub fn from_capabilities(capabilities: DeviceCapabilitySet) -> Self {
        let full_messaging_and_management = DeviceCapabilitySet::SEND_MESSAGE
            .union(DeviceCapabilitySet::RECEIVE_MESSAGE)
            .union(DeviceCapabilitySet::LINK_NEW_DEVICE)
            .union(DeviceCapabilitySet::MANAGE_GROUPS);

        if capabilities.contains(full_messaging_and_management) {
            GenericDeviceDescriptor::FullCapabilityDevice
        } else if capabilities.contains(DeviceCapabilitySet::SEND_MESSAGE)
            || capabilities.contains(DeviceCapabilitySet::RECEIVE_MESSAGE)
        {
            GenericDeviceDescriptor::MessagingDevice
        } else {
            GenericDeviceDescriptor::LimitedDevice
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            GenericDeviceDescriptor::MessagingDevice => "Messaging device",
            GenericDeviceDescriptor::FullCapabilityDevice => "Full-capability device",
            GenericDeviceDescriptor::LimitedDevice => "Limited device",
        }
    }
}

/// §145's "unless user explicitly shares it" made structural: the
/// ONLY way to produce a value that carries a real friendly name is
/// this function, called with an explicit `true` — there is no
/// default, no `From`/`Into` conversion, and no other constructor
/// anywhere that turns a
/// [`crate::discovery_privacy::PrivateDeviceMetadata`] into something
/// nameable to a remote peer. Passing `false` returns the generic
/// descriptor instead, so a caller who wires this up once still gets
/// §145's default behavior for every device unless a user actually
/// opts in per device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteDeviceLabel {
    Generic(GenericDeviceDescriptor),
    ExplicitlyShared(String),
}

pub fn remote_device_label(
    capabilities: DeviceCapabilitySet,
    friendly_name: &str,
    user_explicitly_shared: bool,
) -> RemoteDeviceLabel {
    if user_explicitly_shared {
        RemoteDeviceLabel::ExplicitlyShared(friendly_name.to_string())
    } else {
        RemoteDeviceLabel::Generic(GenericDeviceDescriptor::from_capabilities(capabilities))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_messaging_only_device_gets_the_messaging_descriptor() {
        let descriptor =
            GenericDeviceDescriptor::from_capabilities(DeviceCapabilitySet::SEND_MESSAGE);
        assert_eq!(descriptor, GenericDeviceDescriptor::MessagingDevice);
    }

    #[test]
    fn a_device_with_no_relevant_capability_is_limited() {
        let descriptor = GenericDeviceDescriptor::from_capabilities(DeviceCapabilitySet(0));
        assert_eq!(descriptor, GenericDeviceDescriptor::LimitedDevice);
    }

    #[test]
    fn default_behavior_never_reveals_the_friendly_name() {
        let label = remote_device_label(
            DeviceCapabilitySet::SEND_MESSAGE,
            "Irshad's Bedroom PC",
            false,
        );
        assert_eq!(
            label,
            RemoteDeviceLabel::Generic(GenericDeviceDescriptor::MessagingDevice)
        );
    }

    #[test]
    fn explicit_opt_in_is_the_only_way_to_reveal_the_friendly_name() {
        let label = remote_device_label(
            DeviceCapabilitySet::SEND_MESSAGE,
            "Irshad's Bedroom PC",
            true,
        );
        assert_eq!(
            label,
            RemoteDeviceLabel::ExplicitlyShared("Irshad's Bedroom PC".to_string())
        );
    }
}
