//! §88 "Path Acquisition", §90 "Discovery Budget".

use crate::platform::{DeviceState, ThermalState};
use crate::types::Priority;

/// §90: "Routing should not trigger unlimited scans. Use: discovery
/// budget, cooldown, priority-based escalation." A sliding-window
/// admission counter with a cooldown once exhausted — real state a
/// caller owns and persists across calls (this crate has no clock or
/// storage of its own; see its top doc comment on scope), not a
/// stateless heuristic like most of this crate's other functions,
/// because §90's own "budget" and "cooldown" are inherently about
/// behavior *over time*, which a pure function of a single call's
/// inputs can't express.
pub struct DiscoveryBudget {
    max_attempts_per_window: u32,
    window_millis: u64,
    cooldown_after_exhaustion_millis: u64,
    attempt_timestamps_millis: Vec<u64>,
    exhausted_until_millis: Option<u64>,
}

impl DiscoveryBudget {
    pub fn new(
        max_attempts_per_window: u32,
        window_millis: u64,
        cooldown_after_exhaustion_millis: u64,
    ) -> Self {
        Self {
            max_attempts_per_window,
            window_millis,
            cooldown_after_exhaustion_millis,
            attempt_timestamps_millis: Vec::new(),
            exhausted_until_millis: None,
        }
    }

    /// §90's own worked example, transcribed as concrete numbers:
    /// "normal message: no aggressive BLE scan" is a tight budget (a
    /// couple of attempts, long window, long cooldown); "SOS:
    /// aggressive discovery allowed" is a generous one. `Priority`
    /// (not `DeliveryClass`) is the right axis here — §90's own
    /// example contrasts an ordinary message against an SOS
    /// specifically by urgency, the same signal
    /// [`crate::privacy::justifies_expensive_setup`] (§52) already
    /// keys its own Critical-priority override on.
    pub fn for_priority(priority: Priority) -> Self {
        match priority {
            Priority::Critical => Self::new(20, 60_000, 5_000),
            Priority::High => Self::new(5, 60_000, 15_000),
            Priority::Normal => Self::new(2, 60_000, 30_000),
            Priority::Low => Self::new(1, 120_000, 60_000),
            Priority::Background => Self::new(1, 300_000, 120_000),
        }
    }

    /// Returns `true` and records the attempt if the budget currently
    /// allows one; `false` (recording nothing) otherwise. Exhausting
    /// the window starts a cooldown — the window alone re-admitting
    /// the instant the oldest timestamp ages out would let a caller
    /// retry in a tight loop right at the window boundary, which is
    /// exactly the "unlimited scans" shape §90 warns against; the
    /// cooldown is what actually stops that.
    pub fn try_admit(&mut self, now_millis: u64) -> bool {
        if let Some(until) = self.exhausted_until_millis {
            if now_millis < until {
                return false;
            }
            self.exhausted_until_millis = None;
        }

        self.attempt_timestamps_millis
            .retain(|&t| now_millis.saturating_sub(t) < self.window_millis);

        if self.attempt_timestamps_millis.len() as u32 >= self.max_attempts_per_window {
            self.exhausted_until_millis = Some(now_millis + self.cooldown_after_exhaustion_millis);
            return false;
        }

        self.attempt_timestamps_millis.push(now_millis);
        true
    }
}

/// §88: "Active acquisition must be policy-limited because it costs
/// power/time." Combines [`DiscoveryBudget`] with §85's device state:
/// even a budget-admitted attempt is refused during
/// [`ThermalState::Critical`] — a hardware safety margin, not a user
/// preference, so it has no priority-based override (an SOS shouldn't
/// force active radio scanning if the device is about to thermally
/// shut down; that would make the emergency worse, not better).
/// `battery_saver`, by contrast, *is* a user preference, and
/// `Priority::Critical` does override it — the same asymmetry
/// [`crate::privacy::justifies_expensive_setup`] already treats
/// Critical priority as able to override cost/setup preferences but
/// never safety constraints.
pub fn discovery_permitted(
    priority: Priority,
    device: Option<&DeviceState>,
    budget: &mut DiscoveryBudget,
    now_millis: u64,
) -> bool {
    if let Some(device) = device {
        if device.thermal_state == Some(ThermalState::Critical) {
            return false;
        }
        if device.battery_saver == Some(true) && priority != Priority::Critical {
            return false;
        }
    }
    budget.try_admit(now_millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_normal_priority_budget_is_much_tighter_than_a_critical_ones() {
        let normal = DiscoveryBudget::for_priority(Priority::Normal);
        let critical = DiscoveryBudget::for_priority(Priority::Critical);
        assert!(critical.max_attempts_per_window > normal.max_attempts_per_window);
    }

    #[test]
    fn admits_up_to_the_window_limit_then_denies() {
        let mut budget = DiscoveryBudget::new(2, 60_000, 30_000);
        assert!(budget.try_admit(0));
        assert!(budget.try_admit(1_000));
        assert!(!budget.try_admit(2_000)); // third attempt within the window
    }

    #[test]
    fn exhaustion_starts_a_cooldown_that_outlasts_the_window() {
        let mut budget = DiscoveryBudget::new(1, 10_000, 20_000);
        assert!(budget.try_admit(0));
        // Second attempt well within the 10s window hits the cap and
        // triggers a 20s cooldown from *this* moment.
        assert!(!budget.try_admit(500));
        // Still within that cooldown, even though the original
        // window itself would have re-opened by now.
        assert!(!budget.try_admit(15_000));
        // Cooldown (20s from t=500) has now elapsed.
        assert!(budget.try_admit(20_501));
    }

    #[test]
    fn thermal_critical_blocks_discovery_even_for_critical_priority() {
        let mut budget = DiscoveryBudget::for_priority(Priority::Critical);
        let device = DeviceState {
            thermal_state: Some(ThermalState::Critical),
            ..Default::default()
        };
        assert!(!discovery_permitted(
            Priority::Critical,
            Some(&device),
            &mut budget,
            0
        ));
    }

    #[test]
    fn battery_saver_blocks_normal_priority_but_not_critical() {
        let device = DeviceState {
            battery_saver: Some(true),
            ..Default::default()
        };
        let mut normal_budget = DiscoveryBudget::for_priority(Priority::Normal);
        assert!(!discovery_permitted(
            Priority::Normal,
            Some(&device),
            &mut normal_budget,
            0
        ));

        let mut critical_budget = DiscoveryBudget::for_priority(Priority::Critical);
        assert!(discovery_permitted(
            Priority::Critical,
            Some(&device),
            &mut critical_budget,
            0
        ));
    }

    #[test]
    fn no_device_state_at_all_still_respects_the_budget() {
        let mut budget = DiscoveryBudget::new(1, 60_000, 30_000);
        assert!(discovery_permitted(Priority::Normal, None, &mut budget, 0));
        assert!(!discovery_permitted(
            Priority::Normal,
            None,
            &mut budget,
            100
        ));
    }
}
