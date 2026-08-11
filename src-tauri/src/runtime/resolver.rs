use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use super::paths::RuntimePaths;
use super::version::{EngineType, ParsedVersion, VersionChecker};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineSource {
    Managed,
    System,
    DevOverride,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedEngine {
    pub engine_type: EngineType,
    pub path: Option<PathBuf>,
    pub source: EngineSource,
    pub version: Option<String>,
    pub compatible: bool,
    pub error: Option<String>,
}

pub struct EngineResolver {
    paths: RuntimePaths,
    is_production: bool,
}

impl EngineResolver {
    pub fn new(paths: RuntimePaths, force_production: Option<bool>) -> Self {
        let is_production = force_production.unwrap_or(!cfg!(debug_assertions));
        Self {
            paths,
            is_production,
        }
    }

    pub async fn resolve(&self, engine_type: EngineType) -> ResolvedEngine {
        if self.is_production {
            self.resolve_production(engine_type).await
        } else {
            self.resolve_development(engine_type).await
        }
    }

    async fn resolve_production(&self, engine_type: EngineType) -> ResolvedEngine {
        // Priority 1: Siphonix-managed runtime directory
        let managed_binary = match engine_type {
            EngineType::YtDlp => self.paths.managed_ytdlp_binary(),
            EngineType::Ffmpeg => self.paths.managed_ffmpeg_binary(),
        };

        if managed_binary.exists() && managed_binary.is_file() {
            if let Ok(ver) = VersionChecker::detect_and_verify(&managed_binary, engine_type).await {
                let compatible = VersionChecker::is_compatible(&ver, engine_type);
                return ResolvedEngine {
                    engine_type,
                    path: Some(managed_binary),
                    source: EngineSource::Managed,
                    version: Some(ver.raw),
                    compatible,
                    error: if compatible { None } else { Some("Outdated version".to_string()) },
                };
            }
        }

        // Priority 2: Validated system installation (trusted locations only)
        if let Some((system_binary, ver)) = self.find_system_binary(engine_type).await {
            let compatible = VersionChecker::is_compatible(&ver, engine_type);
            return ResolvedEngine {
                engine_type,
                path: Some(system_binary),
                source: EngineSource::System,
                version: Some(ver.raw),
                compatible,
                error: if compatible { None } else { Some("Outdated version".to_string()) },
            };
        }

        ResolvedEngine {
            engine_type,
            path: None,
            source: EngineSource::None,
            version: None,
            compatible: false,
            error: Some(format!("No compatible {} binary found in managed directory or system installation", engine_type.name())),
        }
    }

    async fn resolve_development(&self, engine_type: EngineType) -> ResolvedEngine {
        let env_var = match engine_type {
            EngineType::YtDlp => "SIPHONIX_YTDLP_PATH",
            EngineType::Ffmpeg => "SIPHONIX_FFMPEG_PATH",
        };

        // Priority 1: Dev environment variable override
        if let Ok(env_val) = std::env::var(env_var) {
            let p = PathBuf::from(env_val);
            if p.exists() && p.is_file() {
                if let Ok(ver) = VersionChecker::detect_and_verify(&p, engine_type).await {
                    let compatible = VersionChecker::is_compatible(&ver, engine_type);
                    return ResolvedEngine {
                        engine_type,
                        path: Some(p),
                        source: EngineSource::DevOverride,
                        version: Some(ver.raw),
                        compatible,
                        error: None,
                    };
                }
            }
        }

        // Priority 2: System installation
        if let Some((system_binary, ver)) = self.find_system_binary(engine_type).await {
            let compatible = VersionChecker::is_compatible(&ver, engine_type);
            return ResolvedEngine {
                engine_type,
                path: Some(system_binary),
                source: EngineSource::System,
                version: Some(ver.raw),
                compatible,
                error: None,
            };
        }

        // Priority 3: Managed runtime
        let managed_binary = match engine_type {
            EngineType::YtDlp => self.paths.managed_ytdlp_binary(),
            EngineType::Ffmpeg => self.paths.managed_ffmpeg_binary(),
        };

        if managed_binary.exists() && managed_binary.is_file() {
            if let Ok(ver) = VersionChecker::detect_and_verify(&managed_binary, engine_type).await {
                let compatible = VersionChecker::is_compatible(&ver, engine_type);
                return ResolvedEngine {
                    engine_type,
                    path: Some(managed_binary),
                    source: EngineSource::Managed,
                    version: Some(ver.raw),
                    compatible,
                    error: None,
                };
            }
        }

        ResolvedEngine {
            engine_type,
            path: None,
            source: EngineSource::None,
            version: None,
            compatible: false,
            error: Some(format!("No {} binary found", engine_type.name())),
        }
    }

    async fn find_system_binary(&self, engine_type: EngineType) -> Option<(PathBuf, ParsedVersion)> {
        let binary_name = engine_type.name();

        let mut candidates = Vec::new();

        // 1. Standard system PATH search
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let p = dir.join(binary_name);
                if p.is_file() {
                    candidates.push(p);
                }
                if cfg!(target_os = "windows") {
                    let p_exe = dir.join(format!("{}.exe", binary_name));
                    if p_exe.is_file() {
                        candidates.push(p_exe);
                    }
                }
            }
        }

        // 2. Known platform directories
        let platform_paths = if cfg!(target_os = "windows") {
            vec![
                format!("C:\\Program Files\\Siphonix\\runtime\\{}\\{}.exe", binary_name, binary_name),
                format!("C:\\ProgramData\\chocolatey\\bin\\{}.exe", binary_name),
            ]
        } else {
            vec![
                format!("/opt/homebrew/bin/{}", binary_name),
                format!("/usr/local/bin/{}", binary_name),
                format!("/usr/bin/{}", binary_name),
            ]
        };

        for p_str in platform_paths {
            let p = PathBuf::from(p_str);
            if p.is_file() {
                candidates.push(p);
            }
        }

        for candidate in candidates {
            if Self::is_trusted_system_location(&candidate) {
                if let Ok(ver) = VersionChecker::detect_and_verify(&candidate, engine_type).await {
                    return Some((candidate, ver));
                }
            }
        }

        None
    }

    pub fn is_trusted_system_location(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let p_str = path.to_string_lossy();

        // Reject suspicious / temp directories
        if p_str.contains("/tmp/") || p_str.contains("\\Temp\\") || p_str.contains("/var/tmp/") {
            return false;
        }

        true
    }
}
