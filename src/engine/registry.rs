use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct ProcessRegistry {
    handles: Arc<Mutex<HashMap<String, Child>>>,
}

impl ProcessRegistry {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register(&self, job_id: String, child: Child) {
        let mut map = self.handles.lock().await;
        map.insert(job_id, child);
    }

    pub async fn unregister(&self, job_id: &str) -> Option<Child> {
        let mut map = self.handles.lock().await;
        map.remove(job_id)
    }

    pub async fn kill(&self, job_id: &str) -> bool {
        let mut map = self.handles.lock().await;
        if let Some(mut child) = map.remove(job_id) {
            let _ = child.start_kill();
            true
        } else {
            false
        }
    }
}
