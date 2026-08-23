use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum JobState {
    QUEUED,
    PREPARING,
    DOWNLOADING,
    PROCESSING,
    PAUSED,
    RETRYING,
    COOLDOWN,
    COMPLETED,
    FAILED,
    CANCELLED,
    NEEDS_ATTENTION,
}

impl JobState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QUEUED => "QUEUED",
            Self::PREPARING => "PREPARING",
            Self::DOWNLOADING => "DOWNLOADING",
            Self::PROCESSING => "PROCESSING",
            Self::PAUSED => "PAUSED",
            Self::RETRYING => "RETRYING",
            Self::COOLDOWN => "COOLDOWN",
            Self::COMPLETED => "COMPLETED",
            Self::FAILED => "FAILED",
            Self::CANCELLED => "CANCELLED",
            Self::NEEDS_ATTENTION => "NEEDS_ATTENTION",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "PREPARING" => Self::PREPARING,
            "DOWNLOADING" => Self::DOWNLOADING,
            "PROCESSING" => Self::PROCESSING,
            "PAUSED" => Self::PAUSED,
            "RETRYING" => Self::RETRYING,
            "COOLDOWN" => Self::COOLDOWN,
            "COMPLETED" => Self::COMPLETED,
            "FAILED" => Self::FAILED,
            "CANCELLED" => Self::CANCELLED,
            "NEEDS_ATTENTION" => Self::NEEDS_ATTENTION,
            _ => Self::QUEUED,
        }
    }

    pub fn can_transition_to(&self, target: Self) -> bool {
        matches!(
            (self, target),
            (Self::QUEUED, Self::PREPARING)
                | (Self::QUEUED, Self::PAUSED)
                | (Self::QUEUED, Self::CANCELLED)
                | (Self::PREPARING, Self::DOWNLOADING)
                | (Self::PREPARING, Self::FAILED)
                | (Self::PREPARING, Self::CANCELLED)
                | (Self::DOWNLOADING, Self::PROCESSING)
                | (Self::DOWNLOADING, Self::COMPLETED)
                | (Self::DOWNLOADING, Self::RETRYING)
                | (Self::DOWNLOADING, Self::FAILED)
                | (Self::DOWNLOADING, Self::CANCELLED)
                | (Self::PROCESSING, Self::COMPLETED)
                | (Self::PROCESSING, Self::FAILED)
                | (Self::PROCESSING, Self::CANCELLED)
                | (Self::RETRYING, Self::QUEUED)
                | (Self::RETRYING, Self::COOLDOWN)
                | (Self::RETRYING, Self::FAILED)
                | (Self::RETRYING, Self::CANCELLED)
                | (Self::COOLDOWN, Self::QUEUED)
                | (Self::COOLDOWN, Self::FAILED)
                | (Self::COOLDOWN, Self::CANCELLED)
                | (Self::PAUSED, Self::QUEUED)
                | (Self::PAUSED, Self::CANCELLED)
                | (Self::FAILED, Self::QUEUED)
                | (Self::CANCELLED, Self::QUEUED)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_state_transitions() {
        assert!(JobState::QUEUED.can_transition_to(JobState::PREPARING));
        assert!(JobState::PREPARING.can_transition_to(JobState::DOWNLOADING));
        assert!(JobState::DOWNLOADING.can_transition_to(JobState::COMPLETED));
        assert!(!JobState::COMPLETED.can_transition_to(JobState::DOWNLOADING));
    }
}
