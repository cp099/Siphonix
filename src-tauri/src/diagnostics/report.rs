use serde::{Deserialize, Serialize};
use super::event::DiagnosticEvent;
use super::health::SystemHealth;
use crate::runtime::RuntimeStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub app_name: String,
    pub app_version: String,
    pub platform: String,
    pub architecture: String,
    pub generated_at: String,
    pub system_health: SystemHealth,
    pub runtime_status: RuntimeStatus,
    pub total_jobs_count: u32,
    pub total_library_items_count: u32,
    pub recent_events: Vec<DiagnosticEvent>,
}

pub struct DiagnosticReportGenerator;

impl DiagnosticReportGenerator {
    pub fn generate(
        system_health: SystemHealth,
        runtime_status: RuntimeStatus,
        total_jobs_count: u32,
        total_library_items_count: u32,
        events: Vec<DiagnosticEvent>,
    ) -> DiagnosticReport {
        // Guarantee all events in the report are sanitized
        let sanitized_events = events
            .into_iter()
            .map(|mut ev| {
                ev.message = DiagnosticEvent::sanitize(&ev.message);
                ev
            })
            .collect();

        DiagnosticReport {
            app_name: "Siphonix".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            system_health,
            runtime_status,
            total_jobs_count,
            total_library_items_count,
            recent_events: sanitized_events,
        }
    }
}
