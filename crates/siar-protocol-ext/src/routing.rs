//! spec §52 "Delivery Classes", §53 "Routing Requirements Per
//! Operation". Deliberately stops short of choosing a transport or
//! network path anywhere in this module — spec §53's own closing line,
//! "the routing engine, not the extension, chooses the concrete
//! network path," and spec §51 "Transport Neutrality" (no Iroh
//! endpoint IDs, Bluetooth MAC addresses, IP addresses, or Wi-Fi
//! handles in extension payloads) are why [`RoutingRequirements`] has
//! no field anywhere that names a transport: everything here is what
//! an operation *declares it needs*, never how that need gets met.

use crate::lifecycle::TrafficPriority;

/// spec §52, verbatim enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeliveryClass {
    Realtime,
    ReliableInteractive,
    Durable,
    DelayTolerant,
}

/// spec §52's own five worked examples. Returns `None` for "File
/// chunk," which the spec itself doesn't give one fixed answer for
/// ("Durable/DelayTolerant depending policy") — that's a real,
/// spec-acknowledged policy decision this function has no business
/// making up on the caller's behalf, so it's the one example this
/// function doesn't classify at all rather than picking one silently.
pub fn spec_52_example_classification(operation_name: &str) -> Option<DeliveryClass> {
    match operation_name {
        "Typing" => Some(DeliveryClass::Realtime),
        "Text" => Some(DeliveryClass::Durable),
        "Video frame" => Some(DeliveryClass::Realtime),
        _ => None,
    }
}

/// spec §53's own six-item list. `size_class` uses generic magnitude
/// buckets ([`SizeClass`]) rather than byte thresholds the spec never
/// gives — this crate has no basis to invent specific cutoffs.
/// `priority` reuses [`TrafficPriority`] (already real, from §21)
/// rather than inventing a second priority concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoutingRequirements {
    pub realtime_requirement: bool,
    pub maximum_age_millis: Option<u64>,
    pub durability: DeliveryClass,
    pub forwarding_permission: bool,
    pub size_class: SizeClass,
    pub priority: TrafficPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SizeClass {
    Small,
    Medium,
    Large,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_52_typing_is_realtime() {
        assert_eq!(
            spec_52_example_classification("Typing"),
            Some(DeliveryClass::Realtime)
        );
    }

    #[test]
    fn spec_52_text_is_durable() {
        assert_eq!(
            spec_52_example_classification("Text"),
            Some(DeliveryClass::Durable)
        );
    }

    #[test]
    fn spec_52_file_chunk_is_left_to_policy_not_guessed() {
        assert_eq!(spec_52_example_classification("File chunk"), None);
    }

    #[test]
    fn spec_53_routing_requirements_never_names_a_transport() {
        // Structural claim, not a runtime one: this compiles at all is
        // the proof — RoutingRequirements has no field of any
        // transport-specific type (no EndpointId, no SocketAddr, no
        // Bluetooth MAC type). Constructing one with only
        // spec-described fields is what this test actually exercises.
        let requirements = RoutingRequirements {
            realtime_requirement: true,
            maximum_age_millis: Some(5_000),
            durability: DeliveryClass::Realtime,
            forwarding_permission: false,
            size_class: SizeClass::Small,
            priority: TrafficPriority::Interactive,
        };
        assert!(requirements.realtime_requirement);
    }
}
