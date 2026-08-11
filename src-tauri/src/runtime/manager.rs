use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::engine::detector::EnginePaths;
use super::health::{HealthEvaluator, RuntimeStatus};
use super::paths::RuntimePaths;
use super::resolver::EngineResolver;
use super::version::EngineType;

#[derive(Clone)]
pub struct EngineManager {
    resolver: Arc<EngineResolver>,
    status_cache: Arc<RwLock<Option<RuntimeStatus>>>,
}

impl EngineManager {
    pub fn new(app_data_dir: &Path, force_production: Option<bool>) -> Self {
        let paths = RuntimePaths::new(app_data_dir);
        let resolver = Arc::new(EngineResolver::new(paths, force_production));

        Self {
            resolver,
            status_cache: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn refresh_status(&self) -> RuntimeStatus {
        let yt = self.resolver.resolve(EngineType::YtDlp).await;
        let ff = self.resolver.resolve(EngineType::Ffmpeg).await;

        let status = HealthEvaluator::evaluate(&yt, &ff);

        // Secure structured logging (never log auth, cookies, or private URLs)
        println!(
            "[EngineManager] Runtime Status Evaluated: ready={}, yt-dlp={:?}, ffmpeg={:?}",
            status.ready, yt.source, ff.source
        );

        let mut cache = self.status_cache.write().await;
        *cache = Some(status.clone());
        status
    }

    pub async fn get_status(&self) -> RuntimeStatus {
        let cache = self.status_cache.read().await;
        if let Some(ref s) = *cache {
            s.clone()
        } else {
            drop(cache);
            self.refresh_status().await
        }
    }

    pub async fn get_engine_paths(&self) -> Option<EnginePaths> {
        let yt = self.resolver.resolve(EngineType::YtDlp).await;
        let ff = self.resolver.resolve(EngineType::Ffmpeg).await;

        if let (Some(yt_path), Some(ff_path)) = (yt.path, ff.path) {
            Some(EnginePaths {
                yt_dlp: yt_path,
                ffmpeg: ff_path,
            })
        } else {
            None
        }
    }
}
