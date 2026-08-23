pub mod error;
pub mod event;
pub mod health;
pub mod logger;
pub mod manager;
pub mod recovery;
pub mod report;
pub mod storage;

pub use error::{ClassifiedDiagnosticFailure, DiagnosticErrorClassifier};
pub use event::{DiagnosticEvent, DiagnosticSeverity};
pub use health::{SubsystemHealth, SystemHealth, SystemHealthEvaluator, SystemHealthStatus};
pub use logger::DiagnosticLogger;
pub use manager::DiagnosticsManager;
pub use recovery::{RecoveryManager, RecoveryResult};
pub use report::{DiagnosticReport, DiagnosticReportGenerator};
pub use storage::DiagnosticStorage;
