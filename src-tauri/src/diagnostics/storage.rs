use std::collections::VecDeque;
use std::sync::RwLock;
use super::event::{DiagnosticEvent, DiagnosticSeverity};

const MAX_EVENT_BUFFER: usize = 200;

pub struct DiagnosticStorage {
    events: RwLock<VecDeque<DiagnosticEvent>>,
}

impl DiagnosticStorage {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(VecDeque::with_capacity(MAX_EVENT_BUFFER)),
        }
    }

    pub fn push(&self, event: DiagnosticEvent) {
        if let Ok(mut lock) = self.events.write() {
            if lock.len() >= MAX_EVENT_BUFFER {
                lock.pop_front();
            }
            lock.push_back(event);
        }
    }

    pub fn get_recent(&self, limit: usize) -> Vec<DiagnosticEvent> {
        if let Ok(lock) = self.events.read() {
            lock.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_recent_by_severity(&self, min_severity: DiagnosticSeverity) -> Vec<DiagnosticEvent> {
        if let Ok(lock) = self.events.read() {
            lock.iter()
                .filter(|ev| Self::severity_value(&ev.severity) >= Self::severity_value(&min_severity))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    fn severity_value(sev: &DiagnosticSeverity) -> u8 {
        match sev {
            DiagnosticSeverity::Debug => 1,
            DiagnosticSeverity::Info => 2,
            DiagnosticSeverity::Warn => 3,
            DiagnosticSeverity::Error => 4,
            DiagnosticSeverity::Critical => 5,
        }
    }
}
