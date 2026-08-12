use serde::{Deserialize, Serialize};
use crate::runtime::RuntimeStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemHealthStatus {
    Healthy,
    Degraded,
    ActionRequired,
}

impl SystemHealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemHealthStatus::Healthy => "HEALTHY",
            SystemHealthStatus::Degraded => "DEGRADED",
            SystemHealthStatus::ActionRequired => "ACTION_REQUIRED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemHealth {
    pub name: String,
    pub status: SystemHealthStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: SystemHealthStatus,
    pub runtime: SubsystemHealth,
    pub database: SubsystemHealth,
    pub queue: SubsystemHealth,
    pub library: SubsystemHealth,
    pub active_issues_count: u32,
}

pub struct SystemHealthEvaluator;

impl SystemHealthEvaluator {
    pub fn evaluate(
        runtime_status: &RuntimeStatus,
        db_healthy: bool,
        failed_jobs_count: u32,
        missing_library_files_count: u32,
    ) -> SystemHealth {
        let runtime_subsystem = if !runtime_status.ready {
            SubsystemHealth {
                name: "Runtime Engine".to_string(),
                status: SystemHealthStatus::ActionRequired,
                message: "yt-dlp or FFmpeg executable is missing or incompatible.".to_string(),
            }
        } else {
            SubsystemHealth {
                name: "Runtime Engine".to_string(),
                status: SystemHealthStatus::Healthy,
                message: "yt-dlp and FFmpeg executables are verified and ready.".to_string(),
            }
        };

        let db_subsystem = if !db_healthy {
            SubsystemHealth {
                name: "SQLite Database".to_string(),
                status: SystemHealthStatus::ActionRequired,
                message: "Database connection or migration failed.".to_string(),
            }
        } else {
            SubsystemHealth {
                name: "SQLite Database".to_string(),
                status: SystemHealthStatus::Healthy,
                message: "Database initialized and schema verified.".to_string(),
            }
        };

        let queue_subsystem = if failed_jobs_count > 0 {
            SubsystemHealth {
                name: "Download Queue".to_string(),
                status: SystemHealthStatus::Degraded,
                message: format!("Queue active with {} failed job(s).", failed_jobs_count),
            }
        } else {
            SubsystemHealth {
                name: "Download Queue".to_string(),
                status: SystemHealthStatus::Healthy,
                message: "Queue operational with 0 failures.".to_string(),
            }
        };

        let library_subsystem = if missing_library_files_count > 0 {
            SubsystemHealth {
                name: "Media Library".to_string(),
                status: SystemHealthStatus::Degraded,
                message: format!("{} library file(s) missing from disk.", missing_library_files_count),
            }
        } else {
            SubsystemHealth {
                name: "Media Library".to_string(),
                status: SystemHealthStatus::Healthy,
                message: "All library records verified on disk.".to_string(),
            }
        };

        let mut issues = 0;
        if runtime_subsystem.status != SystemHealthStatus::Healthy { issues += 1; }
        if db_subsystem.status != SystemHealthStatus::Healthy { issues += 1; }
        if queue_subsystem.status != SystemHealthStatus::Healthy { issues += 1; }
        if library_subsystem.status != SystemHealthStatus::Healthy { issues += 1; }

        let overall_status = if runtime_subsystem.status == SystemHealthStatus::ActionRequired
            || db_subsystem.status == SystemHealthStatus::ActionRequired
        {
            SystemHealthStatus::ActionRequired
        } else if issues > 0 {
            SystemHealthStatus::Degraded
        } else {
            SystemHealthStatus::Healthy
        };

        SystemHealth {
            overall_status,
            runtime: runtime_subsystem,
            database: db_subsystem,
            queue: queue_subsystem,
            library: library_subsystem,
            active_issues_count: issues,
        }
    }
}
