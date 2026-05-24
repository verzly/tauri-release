use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::{ContainerEngine, PackageManager};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ReleaseConfig {
    pub project: ProjectConfig,
    pub container: ContainerConfig,
    pub output: OutputConfig,
    pub artifacts: ArtifactConfig,
    pub linux: LinuxConfig,
    pub android: AndroidConfig,
    pub windows: HostPlatformConfig,
    pub macos: HostPlatformConfig,
    pub ios: HostPlatformConfig,
}

impl Default for ReleaseConfig {
    fn default() -> Self {
        Self {
            project: ProjectConfig::default(),
            container: ContainerConfig::default(),
            output: OutputConfig::default(),
            artifacts: ArtifactConfig::default(),
            linux: LinuxConfig::default(),
            android: AndroidConfig::default(),
            windows: HostPlatformConfig::windows_default(),
            macos: HostPlatformConfig::macos_default(),
            ios: HostPlatformConfig::ios_default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub project_dir: PathBuf,
    pub package_manager: Option<PackageManagerSerde>,
    pub install_command: Option<String>,
    pub app_name: Option<String>,
    pub version: Option<String>,
    pub env: BTreeMap<String, String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project_dir: PathBuf::from("."),
            package_manager: None,
            install_command: None,
            app_name: None,
            version: None,
            env: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ContainerConfig {
    pub engine: ContainerEngineSerde,
    pub cache_dir: Option<PathBuf>,
    pub linux_image: String,
    pub android_image: String,
    pub userns_keep_id: bool,
    pub network: Option<String>,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            engine: ContainerEngineSerde::Podman,
            cache_dir: None,
            linux_image: "ghcr.io/your-org/tauri-release-linux:latest".to_string(),
            android_image: "ghcr.io/your-org/tauri-release-android:latest".to_string(),
            userns_keep_id: true,
            network: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct OutputConfig {
    pub dir: Option<PathBuf>,
    pub clean: bool,
    pub sha256: bool,
    pub manifest: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            dir: None,
            clean: true,
            sha256: true,
            manifest: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ArtifactConfig {
    pub include_extensions: Vec<String>,
    pub include_files: Vec<String>,
    pub keep_relative_paths: bool,
}

impl Default for ArtifactConfig {
    fn default() -> Self {
        Self {
            include_extensions: vec![
                "AppImage".into(),
                "deb".into(),
                "rpm".into(),
                "exe".into(),
                "msi".into(),
                "zip".into(),
                "dmg".into(),
                "apk".into(),
                "aab".into(),
                "ipa".into(),
                "sig".into(),
                "json".into(),
            ],
            include_files: vec!["latest.json".into(), "release.json".into()],
            keep_relative_paths: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LinuxConfig {
    pub enabled: bool,
    pub image: Option<String>,
    pub bundles: Vec<String>,
    pub command: Option<String>,
}

impl Default for LinuxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            image: None,
            bundles: vec!["appimage".into(), "deb".into(), "rpm".into()],
            command: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct AndroidConfig {
    pub enabled: bool,
    pub image: Option<String>,
    pub apk: bool,
    pub aab: bool,
    pub split_per_abi: bool,
    pub targets: Vec<String>,
    pub command: Option<String>,
}

impl Default for AndroidConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            image: None,
            apk: true,
            aab: true,
            split_per_abi: false,
            targets: vec!["aarch64".into()],
            command: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HostPlatformConfig {
    pub enabled: bool,
    pub strategy: HostStrategy,
    pub command: Option<String>,
}

impl HostPlatformConfig {
    pub fn windows_default() -> Self {
        Self {
            enabled: false,
            strategy: HostStrategy::HostOnly,
            command: Some("pnpm tauri build".into()),
        }
    }

    pub fn macos_default() -> Self {
        Self {
            enabled: false,
            strategy: HostStrategy::HostOnly,
            command: Some("pnpm tauri build".into()),
        }
    }

    pub fn ios_default() -> Self {
        Self {
            enabled: false,
            strategy: HostStrategy::HostOnly,
            command: Some("pnpm tauri ios build --ci".into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum HostStrategy {
    Disabled,
    HostOnly,
}

impl Default for HostPlatformConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: HostStrategy::Disabled,
            command: None,
        }
    }
}

impl Default for HostStrategy {
    fn default() -> Self {
        HostStrategy::Disabled
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ContainerEngineSerde {
    Podman,
    Docker,
}

impl Default for ContainerEngineSerde {
    fn default() -> Self {
        ContainerEngineSerde::Podman
    }
}

impl From<ContainerEngineSerde> for ContainerEngine {
    fn from(value: ContainerEngineSerde) -> Self {
        match value {
            ContainerEngineSerde::Podman => ContainerEngine::Podman,
            ContainerEngineSerde::Docker => ContainerEngine::Docker,
        }
    }
}

impl From<ContainerEngine> for ContainerEngineSerde {
    fn from(value: ContainerEngine) -> Self {
        match value {
            ContainerEngine::Podman => ContainerEngineSerde::Podman,
            ContainerEngine::Docker => ContainerEngineSerde::Docker,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PackageManagerSerde {
    Pnpm,
    Npm,
    Yarn,
    Bun,
    Cargo,
}

impl From<PackageManagerSerde> for PackageManager {
    fn from(value: PackageManagerSerde) -> Self {
        match value {
            PackageManagerSerde::Pnpm => PackageManager::Pnpm,
            PackageManagerSerde::Npm => PackageManager::Npm,
            PackageManagerSerde::Yarn => PackageManager::Yarn,
            PackageManagerSerde::Bun => PackageManager::Bun,
            PackageManagerSerde::Cargo => PackageManager::Cargo,
        }
    }
}

impl From<PackageManager> for PackageManagerSerde {
    fn from(value: PackageManager) -> Self {
        match value {
            PackageManager::Pnpm => PackageManagerSerde::Pnpm,
            PackageManager::Npm => PackageManagerSerde::Npm,
            PackageManager::Yarn => PackageManagerSerde::Yarn,
            PackageManager::Bun => PackageManagerSerde::Bun,
            PackageManager::Cargo => PackageManagerSerde::Cargo,
        }
    }
}

pub fn load_config(path: Option<&Path>) -> Result<ReleaseConfig> {
    let path = match path {
        Some(path) => path.to_path_buf(),
        None => PathBuf::from("tauri-release.toml"),
    };

    if !path.exists() {
        return Ok(ReleaseConfig::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let config: ReleaseConfig = toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(config)
}

pub fn write_example_config(path: &Path, force: bool) -> Result<()> {
    if path.exists() && !force {
        anyhow::bail!("{} already exists. Use --force to overwrite it.", path.display());
    }

    fs::write(path, include_str!("../tauri-release.example.toml"))
        .with_context(|| format!("failed to write {}", path.display()))
}
