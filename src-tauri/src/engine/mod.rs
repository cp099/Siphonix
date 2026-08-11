pub mod builder;
pub mod detector;
pub mod error;
pub mod info;
pub mod manager;
pub mod options;
pub mod playlist;
pub mod registry;
pub mod runner;

pub use builder::DownloadRequest;
pub use detector::{EngineDetector, EngineStatus};
pub use info::{inspect_url, VideoInfo};
pub use manager::DownloadManager;
pub use options::{DownloadOptions, DownloadPreset};
pub use playlist::{PlaylistEntry, PlaylistInfo, PlaylistInspector};

