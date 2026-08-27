//! Energy-aware discovery modes — next.md §64–67: "Continuous BLE
//! scanning / Wi-Fi discovery / Wi-Fi Direct can consume significant
//! battery. Therefore use modes."

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryMode {
    Normal,
    NearbyActive,
    Emergency,
    BatterySaver,
}

/// How aggressively a radio should scan/advertise. Deliberately
/// qualitative tiers rather than an invented specific duty-cycle
/// percentage or interval — next.md §65–66 only ever describes these in
/// qualitative terms ("low duty cycle," "on-demand," "frequent,"
/// "active"), and turning that into a fabricated precise number here
/// would be presenting a guess as a measured constant, the same trap
/// `siar_routing::score::route_score`'s doc comment already calls out
/// for its own weights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScanIntensity {
    Off,
    OnDemand,
    Low,
    Active,
}

/// The knobs next.md §65–66 actually names per mode. Not every radio
/// this doc discusses has a settings surface *this* granular yet in
/// this workspace (Bluetooth Classic doesn't have a crate at all) —
/// this only covers what §65/§66 explicitly call out: BLE, Wi-Fi
/// Direct, and DTN.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoveryModeSettings {
    pub ble_scan: ScanIntensity,
    pub wifi_direct_scan: ScanIntensity,
    pub dtn_enabled: bool,
    /// next.md §66: "SOS replication: high priority" is called out
    /// specifically under Emergency mode, distinct from DTN merely
    /// being enabled at all.
    pub sos_replication_prioritized: bool,
}

/// next.md §65 (Normal) and §66 (Emergency) give explicit settings;
/// `NearbyActive` and `BatterySaver` aren't spelled out with the same
/// level of detail in the doc, so these two are a reasonable
/// interpolation between the two documented endpoints, not a direct
/// transcription — flagged here rather than presented as equally
/// doc-sourced as `Normal`/`Emergency`.
pub fn settings_for(mode: DiscoveryMode) -> DiscoveryModeSettings {
    match mode {
        DiscoveryMode::Normal => DiscoveryModeSettings {
            ble_scan: ScanIntensity::Low,
            wifi_direct_scan: ScanIntensity::OnDemand,
            dtn_enabled: true, // §65: "DTN: opportunistic" — enabled, just not prioritized
            sos_replication_prioritized: false,
        },
        DiscoveryMode::Emergency => DiscoveryModeSettings {
            ble_scan: ScanIntensity::Active,
            wifi_direct_scan: ScanIntensity::Active,
            dtn_enabled: true,
            sos_replication_prioritized: true,
        },
        DiscoveryMode::NearbyActive => DiscoveryModeSettings {
            ble_scan: ScanIntensity::Active,
            wifi_direct_scan: ScanIntensity::Low,
            dtn_enabled: true,
            sos_replication_prioritized: false,
        },
        DiscoveryMode::BatterySaver => DiscoveryModeSettings {
            ble_scan: ScanIntensity::Off,
            wifi_direct_scan: ScanIntensity::Off,
            dtn_enabled: false,
            sos_replication_prioritized: false,
        },
    }
}

/// next.md §67: "Advertise coarse state... Do NOT publish exact
/// battery percentage unnecessarily."
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayCapacity {
    Unavailable,
    Low,
    Normal,
    High,
}

/// The actual coarsening step §67 asks for: an exact percentage goes in
/// (from the OS battery API — real device access this crate has no
/// dependency for), a 4-way tier comes out, and the exact number is
/// never carried any further than this one function call. Thresholds
/// (5% / 20% / 60%) are this crate's own reasonable choice — the doc
/// doesn't specify exact cutoffs, only that it should be coarse.
pub fn relay_capacity_for_battery_percent(percent: u8) -> RelayCapacity {
    match percent {
        0..=4 => RelayCapacity::Unavailable,
        5..=19 => RelayCapacity::Low,
        20..=59 => RelayCapacity::Normal,
        _ => RelayCapacity::High,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emergency_mode_scans_more_aggressively_than_normal() {
        let normal = settings_for(DiscoveryMode::Normal);
        let emergency = settings_for(DiscoveryMode::Emergency);
        assert!(emergency.ble_scan > normal.ble_scan);
        assert!(emergency.sos_replication_prioritized && !normal.sos_replication_prioritized);
    }

    #[test]
    fn battery_saver_turns_discovery_off() {
        let settings = settings_for(DiscoveryMode::BatterySaver);
        assert_eq!(settings.ble_scan, ScanIntensity::Off);
        assert!(!settings.dtn_enabled);
    }

    #[test]
    fn battery_percent_coarsens_into_four_tiers() {
        assert_eq!(
            relay_capacity_for_battery_percent(2),
            RelayCapacity::Unavailable
        );
        assert_eq!(relay_capacity_for_battery_percent(10), RelayCapacity::Low);
        assert_eq!(
            relay_capacity_for_battery_percent(45),
            RelayCapacity::Normal
        );
        assert_eq!(relay_capacity_for_battery_percent(90), RelayCapacity::High);
    }

    #[test]
    fn battery_tier_boundaries_are_exact() {
        assert_eq!(
            relay_capacity_for_battery_percent(4),
            RelayCapacity::Unavailable
        );
        assert_eq!(relay_capacity_for_battery_percent(5), RelayCapacity::Low);
        assert_eq!(relay_capacity_for_battery_percent(19), RelayCapacity::Low);
        assert_eq!(
            relay_capacity_for_battery_percent(20),
            RelayCapacity::Normal
        );
        assert_eq!(
            relay_capacity_for_battery_percent(59),
            RelayCapacity::Normal
        );
        assert_eq!(relay_capacity_for_battery_percent(60), RelayCapacity::High);
    }
}
