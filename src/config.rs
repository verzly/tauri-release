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
    pub linux: DesktopPlatformConfig,
    pub android: AndroidConfig,
    pub windows: DesktopPlatformConfig,
    pub macos: DesktopPlatformConfig,
    pub ios: IosConfig,
}

impl Default for ReleaseConfig {
    fn default() -> Self {
        Self {
            project: ProjectConfig::default(),
            container: ContainerConfig::default(),
            output: OutputConfig::default(),
            artifacts: ArtifactConfig::default(),
            linux: DesktopPlatformConfig::linux_default(),
            android: AndroidConfig::default(),
            windows: DesktopPlatformConfig::windows_default(),
            macos: DesktopPlatformConfig::macos_default(),
            ios: IosConfig::default(),
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
    pub windows_image: String,
    pub macos_image: String,
    pub ios_image: String,
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
            windows_image: "ghcr.io/your-org/tauri-release-windows:latest".to_string(),
            macos_image: "ghcr.io/your-org/tauri-release-macos:latest".to_string(),
            ios_image: "ghcr.io/your-org/tauri-release-ios:latest".to_string(),
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
    pub allow_empty: bool,
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
            ],
            include_files: vec!["latest.json".into(), "release.json".into(), "release-manifest.json".into()],
            keep_relative_paths: false,
            allow_empty: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DesktopPlatformConfig {
    pub enabled: bool,
    pub strategy: BuildStrategy,
    pub image: Option<String>,
    pub bundles: Vec<String>,
    pub targets: Vec<String>,
    pub command: Option<String>,
}

impl DesktopPlatformConfig {
    pub fn linux_default() -> Self {
        Self {
            enabled: true,
            strategy: BuildStrategy::Auto,
            image: None,
            bundles: vec!["appimage".into(), "deb".into(), "rpm".into()],
            targets: Vec::new(),
            command: None,
        }
    }

    pub fn windows_default() -> Self {
        Self {
            enabled: false,
            strategy: BuildStrategy::Auto,
            image: None,
            bundles: vec!["nsis".into(), "msi".into()],
            targets: vec!["x86_64-pc-windows-msvc".into()],
            command: None,
        }
    }

    pub fn macos_default() -> Self {
        Self {
            enabled: false,
            strategy: BuildStrategy::Auto,
            image: None,
            bundles: vec!["dmg".into(), "app".into()],
            targets: vec!["universal-apple-darwin".into()],
            command: None,
        }
    }
}

impl Default for DesktopPlatformConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: BuildStrategy::Disabled,
            image: None,
            bundles: Vec::new(),
            targets: Vec::new(),
            command: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct AndroidConfig {
    pub enabled: bool,
    pub strategy: BuildStrategy,
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
            strategy: BuildStrategy::Container,
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
pub struct IosConfig {
    pub enabled: bool,
    pub strategy: BuildStrategy,
    pub image: Option<String>,
    pub targets: Vec<String>,
    pub command: Option<String>,
}

impl Default for IosConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: BuildStrategy::Auto,
            image: None,
            targets: Vec::new(),
            command: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum BuildStrategy {
    Disabled,
    Auto,
    Container,
    HostOnly,
}

impl Default for BuildStrategy {
    fn default() -> Self {
        BuildStrategy::Disabled
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

    fs::write(path, include_str!("../examples/tauri-release.toml"))
        .with_context(|| format!("failed to write {}", path.display()))
}
