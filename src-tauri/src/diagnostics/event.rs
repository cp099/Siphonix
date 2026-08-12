use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DiagnosticSeverity {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl DiagnosticSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Debug => "DEBUG",
            DiagnosticSeverity::Info => "INFO",
            DiagnosticSeverity::Warn => "WARN",
            DiagnosticSeverity::Error => "ERROR",
            DiagnosticSeverity::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub severity: DiagnosticSeverity,
    pub subsystem: String, // "runtime" | "queue" | "download" | "database" | "library" | "system"
    pub event_type: String,
    pub job_id: Option<String>,
    pub engine_info: Option<String>,
    pub message: String,
    pub context: Option<serde_json::Value>,
}

impl DiagnosticEvent {
    pub fn new(
        severity: DiagnosticSeverity,
        subsystem: impl Into<String>,
        event_type: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: format!("diag-{}-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0), std::process::id()),
            timestamp: Utc::now(),
            severity,
            subsystem: subsystem.into(),
            event_type: event_type.into(),
            job_id: None,
            engine_info: None,
            message: Self::sanitize(&message.into()),
            context: None,
        }
    }

    pub fn with_job_id(mut self, job_id: impl Into<String>) -> Self {
        self.job_id = Some(job_id.into());
        self
    }

    pub fn with_engine_info(mut self, engine_info: impl Into<String>) -> Self {
        self.engine_info = Some(engine_info.into());
        self
    }

    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    /// Sanitizes sensitive information from log messages (strips cookies, auth tokens, passwords, credentials, sensitive URL params)
    pub fn sanitize(input: &str) -> String {
        let mut s = input.to_string();

        let sensitive_keys = ["cookie", "authorization", "auth", "token", "password", "secret", "bearer", "session"];
        for key in sensitive_keys {
            let lower = s.to_lowercase();
            if let Some(pos) = lower.find(key) {
                // Find end of token/word or value
                let rest = &s[pos..];
                let end_pos = rest.find(|c: char| c.is_whitespace() || c == '&' || c == ';').unwrap_or(rest.len());
                let target = &rest[..end_pos];
                s = s.replace(target, "[REDACTED]");
            }
        }

        s
    }
}
