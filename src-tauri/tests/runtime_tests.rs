use std::path::PathBuf;
use tempfile::tempdir;

use siphonix_lib::runtime::{
    EngineResolver, EngineSource, EngineStatusState, EngineType, HealthEvaluator,
    ParsedVersion, ResolvedEngine, RuntimePaths, VersionChecker,
};

#[tokio::test]
async fn test_runtime_path_is_within_managed_directory() {
    let tmp = tempdir().unwrap();
    let paths = RuntimePaths::new(tmp.path());

    let ytdlp_path = paths.managed_ytdlp_binary();
    let ffmpeg_path = paths.managed_ffmpeg_binary();

    assert!(paths.is_within_managed_directory(&ytdlp_path));
    assert!(paths.is_within_managed_directory(&ffmpeg_path));
    assert!(!paths.is_within_managed_directory(&PathBuf::from("/usr/bin/yt-dlp")));
}

#[test]
fn test_parse_ytdlp_version() {
    let stdout = "2024.03.10\n";
    let ver = VersionChecker::parse_ytdlp_version(stdout, stdout).unwrap();
    assert_eq!(ver.major, 2024);
    assert_eq!(ver.minor, 3);
    assert_eq!(ver.patch, 10);
    assert_eq!(ver.raw, "2024.03.10");

    let valid = VersionChecker::is_compatible(&ver, EngineType::YtDlp);
    assert!(valid);
}

#[test]
fn test_parse_ffmpeg_version() {
    let stdout = "ffmpeg version 6.1.1 Copyright (c) 2000-2023 the FFmpeg developers\n";
    let ver = VersionChecker::parse_ffmpeg_version(stdout, stdout).unwrap();
    assert_eq!(ver.major, 6);
    assert_eq!(ver.minor, 1);
    assert_eq!(ver.patch, 1);

    let valid = VersionChecker::is_compatible(&ver, EngineType::Ffmpeg);
    assert!(valid);
}

#[test]
fn test_invalid_version_output() {
    let stdout = "malicious output string\n";
    let err = VersionChecker::parse_ytdlp_version(stdout, stdout);
    assert!(err.is_err());
}

#[test]
fn test_engine_identity_validation() {
    let stdout = "some random tool v1.0\n";
    let err = VersionChecker::parse_ffmpeg_version(stdout, stdout);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("Identity validation failed"));
}

#[test]
fn test_supported_and_outdated_version() {
    let outdated_ytdlp = ParsedVersion {
        raw: "2021.01.01".to_string(),
        major: 2021,
        minor: 1,
        patch: 1,
    };
    assert!(!VersionChecker::is_compatible(&outdated_ytdlp, EngineType::YtDlp));

    let modern_ytdlp = ParsedVersion {
        raw: "2024.03.10".to_string(),
        major: 2024,
        minor: 3,
        patch: 10,
    };
    assert!(VersionChecker::is_compatible(&modern_ytdlp, EngineType::YtDlp));
}

#[tokio::test]
async fn test_managed_runtime_preferred() {
    let tmp = tempdir().unwrap();
    let paths = RuntimePaths::new(tmp.path());

    std::fs::create_dir_all(paths.managed_ytdlp_dir()).unwrap();

    let fake_binary = paths.managed_ytdlp_binary();
    #[cfg(unix)]
    {
        std::fs::write(&fake_binary, "#!/bin/sh\necho '2026.01.01'\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&fake_binary, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    #[cfg(windows)]
    {
        std::fs::write(&fake_binary, "@echo 2026.01.01\n").unwrap();
    }

    let resolver = EngineResolver::new(paths, Some(true));
    let res = resolver.resolve(EngineType::YtDlp).await;

    assert_eq!(res.source, EngineSource::Managed);
    assert!(res.compatible);
    assert_eq!(res.version.unwrap(), "2026.01.01");
}

#[tokio::test]
async fn test_reject_untrusted_production_path() {
    assert!(!EngineResolver::is_trusted_system_location(&PathBuf::from("/tmp/malware/yt-dlp")));
    assert!(!EngineResolver::is_trusted_system_location(&PathBuf::from("C:\\Temp\\bad.exe")));
}

#[tokio::test]
async fn test_missing_engine_detection() {
    let tmp = tempdir().unwrap();
    let paths = RuntimePaths::new(tmp.path());
    let resolver = EngineResolver::new(paths, Some(true));

    let yt_res = ResolvedEngine {
        engine_type: EngineType::YtDlp,
        path: None,
        source: EngineSource::None,
        version: None,
        compatible: false,
        error: Some("Missing".to_string()),
    };

    let ff_res = ResolvedEngine {
        engine_type: EngineType::Ffmpeg,
        path: None,
        source: EngineSource::None,
        version: None,
        compatible: false,
        error: Some("Missing".to_string()),
    };

    let status = HealthEvaluator::evaluate(&yt_res, &ff_res);

    assert!(!status.ready);
    assert_eq!(status.yt_dlp.status, EngineStatusState::Missing);
    assert_eq!(status.ffmpeg.status, EngineStatusState::Missing);
}

#[tokio::test]
async fn test_healthy_runtime() {
    let yt_res = ResolvedEngine {
        engine_type: EngineType::YtDlp,
        path: Some(PathBuf::from("/usr/local/bin/yt-dlp")),
        source: EngineSource::System,
        version: Some("2026.01.01".to_string()),
        compatible: true,
        error: None,
    };

    let ff_res = ResolvedEngine {
        engine_type: EngineType::Ffmpeg,
        path: Some(PathBuf::from("/usr/local/bin/ffmpeg")),
        source: EngineSource::System,
        version: Some("6.1.1".to_string()),
        compatible: true,
        error: None,
    };

    let status = HealthEvaluator::evaluate(&yt_res, &ff_res);

    assert!(status.ready);
    assert_eq!(status.yt_dlp.status, EngineStatusState::Ready);
    assert_eq!(status.ffmpeg.status, EngineStatusState::Ready);
}

#[tokio::test]
async fn test_development_override() {
    let tmp = tempdir().unwrap();
    let paths = RuntimePaths::new(tmp.path());

    let override_binary = tmp.path().join("my_dev_ytdlp");
    #[cfg(unix)]
    {
        std::fs::write(&override_binary, "#!/bin/sh\necho '2026.02.02'\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&override_binary, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    #[cfg(windows)]
    {
        std::fs::write(&override_binary, "@echo 2026.02.02\n").unwrap();
    }

    std::env::set_var("SIPHONIX_YTDLP_PATH", override_binary.to_string_lossy().to_string());

    let resolver = EngineResolver::new(paths, Some(false));
    let res = resolver.resolve(EngineType::YtDlp).await;

    assert_eq!(res.source, EngineSource::DevOverride);
    assert_eq!(res.version.unwrap(), "2026.02.02");

    std::env::remove_var("SIPHONIX_YTDLP_PATH");
}
