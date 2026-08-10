use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooldownStatus {
    pub active: bool,
    pub remaining_secs: u64,
}

pub struct CooldownManager {
    rate_limit_events: Vec<Instant>,
    cooldown_until: Option<Instant>,
    window_duration: Duration,
    cooldown_duration: Duration,
    threshold: usize,
}

impl Default for CooldownManager {
    fn default() -> Self {
        Self {
            rate_limit_events: Vec::new(),
            cooldown_until: None,
            window_duration: Duration::from_secs(60),
            cooldown_duration: Duration::from_secs(120),
            threshold: 2,
        }
    }
}

impl CooldownManager {
    pub fn new(threshold: usize, window_secs: u64, cooldown_secs: u64) -> Self {
        Self {
            rate_limit_events: Vec::new(),
            cooldown_until: None,
            window_duration: Duration::from_secs(window_secs),
            cooldown_duration: Duration::from_secs(cooldown_secs),
            threshold,
        }
    }

    /// Record a rate-limit event. Returns true if queue-wide cooldown is triggered.
    pub fn record_rate_limit(&mut self) -> bool {
        let now = Instant::now();
        self.rate_limit_events.push(now);

        // Prune old events outside window
        let cutoff = now.checked_sub(self.window_duration).unwrap_or(now);
        self.rate_limit_events.retain(|&ts| ts >= cutoff);

        if self.rate_limit_events.len() >= self.threshold {
            self.cooldown_until = Some(now + self.cooldown_duration);
            true
        } else {
            false
        }
    }

    pub fn is_cooldown_active(&self) -> bool {
        if let Some(until) = self.cooldown_until {
            Instant::now() < until
        } else {
            false
        }
    }

    pub fn remaining_secs(&self) -> u64 {
        if let Some(until) = self.cooldown_until {
            let now = Instant::now();
            if now < until {
                return (until - now).as_secs();
            }
        }
        0
    }

    /// End current cooldown immediately without resetting rate-limit history.
    pub fn force_resume(&mut self) {
        self.cooldown_until = None;
    }

    pub fn get_status(&self) -> CooldownStatus {
        CooldownStatus {
            active: self.is_cooldown_active(),
            remaining_secs: self.remaining_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cooldown_trigger() {
        let mut mgr = CooldownManager::new(2, 60, 120);
        assert!(!mgr.record_rate_limit()); // 1st event
        assert!(mgr.record_rate_limit());  // 2nd event -> triggers cooldown

        assert!(mgr.is_cooldown_active());
        assert!(mgr.remaining_secs() > 0);
    }

    #[test]
    fn test_force_resume_preserves_rate_limit_history() {
        let mut mgr = CooldownManager::new(2, 60, 120);
        mgr.record_rate_limit();
        mgr.record_rate_limit();
        assert!(mgr.is_cooldown_active());

        // Manual resume ends current cooldown
        mgr.force_resume();
        assert!(!mgr.is_cooldown_active());

        // Rate-limit history remains preserved! Recording 1 new rate limit immediately triggers another cooldown
        assert!(mgr.record_rate_limit());
        assert!(mgr.is_cooldown_active());
    }
}
