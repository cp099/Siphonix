use rand::Rng;
use std::time::Duration;

pub struct BackoffPolicy {
    pub base_secs: u64,
    pub max_secs: u64,
    pub max_retries: u32,
}

impl Default for BackoffPolicy {
    fn default() -> Self {
        Self {
            base_secs: 15,
            max_secs: 300,
            max_retries: 5,
        }
    }
}

impl BackoffPolicy {
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        let exp_factor = 2u64.saturating_pow(attempt.saturating_sub(1));
        let raw_secs = self.base_secs.saturating_mul(exp_factor);

        // Add 1 to 5 seconds of randomized jitter
        let mut rng = rand::thread_rng();
        let jitter: u64 = rng.gen_range(1..=5);

        let total_secs = raw_secs.saturating_add(jitter).min(self.max_secs);
        Duration::from_secs(total_secs)
    }

    pub fn is_retryable(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_calculation() {
        let policy = BackoffPolicy::default();
        let delay1 = policy.calculate_delay(1);
        let delay2 = policy.calculate_delay(2);

        assert!(delay1.as_secs() >= 16 && delay1.as_secs() <= 20); // 15 + (1..5)
        assert!(delay2.as_secs() >= 31 && delay2.as_secs() <= 35); // 30 + (1..5)
    }

    #[test]
    fn test_is_retryable() {
        let policy = BackoffPolicy::default();
        assert!(policy.is_retryable(0));
        assert!(policy.is_retryable(4));
        assert!(!policy.is_retryable(5));
    }
}
