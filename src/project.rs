use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::PackageManager;

#[derive(Clone, Debug)]
pub struct ProjectMetadata {
    pub display_name: String,
    pub version: String,
    pub package_manager: PackageManager,
}

#[derive(Deserialize)]
struct CargoToml {
    package: Option<CargoPackage>,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: Option<String>,
    version: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TauriConfig {
    product_name: Option<String>,
    version: Option<String>,
}

#[derive(Deserialize)]
struct PackageJson {
    name: Option<String>,
    version: Option<String>,
}

pub fn detect_metadata(root: &Path, project_dir: &Path, override_name: Option<String>, override_version: Option<String>, override_pm: Option<PackageManager>) -> Result<ProjectMetadata> {
    let app_dir = root.join(project_dir);
    let src_tauri = app_dir.join("src-tauri");

    let package_manager = override_pm.unwrap_or_else(|| detect_package_manager(&app_dir));
    let package_json = read_package_json(&app_dir).unwrap_or(None);
    let cargo_toml = read_cargo_toml(&src_tauri.join("Cargo.toml")).unwrap_or(None);
    let tauri_config = read_tauri_config(&src_tauri).unwrap_or(None);

    let display_name = override_name
        .or_else(|| tauri_config.as_ref().and_then(|config| config.product_name.clone()))
        .or_else(|| package_json.as_ref().and_then(|package| package.name.clone()))
        .or_else(|| cargo_toml.as_ref().and_then(|cargo| cargo.package.as_ref()).and_then(|pkg| pkg.name.clone()))
        .unwrap_or_else(|| "tauri-app".to_string());

    let version = override_version
        .or_else(|| tauri_config.as_ref().and_then(|config| config.version.clone()))
        .or_else(|| package_json.as_ref().and_then(|package| package.version.clone()))
        .or_else(|| cargo_toml.as_ref().and_then(|cargo| cargo.package.as_ref()).and_then(|pkg| pkg.version.clone()))
        .unwrap_or_else(|| "0.0.0".to_string());

    Ok(ProjectMetadata {
        display_name,
        version,
        package_manager,
    })
}

fn detect_package_manager(app_dir: &Path) -> PackageManager {
    if app_dir.join("pnpm-lock.yaml").exists() || app_dir.join("pnpm-workspace.yaml").exists() {
        PackageManager::Pnpm
    } else if app_dir.join("package-lock.json").exists() {
        PackageManager::Npm
    } else if app_dir.join("yarn.lock").exists() {
        PackageManager::Yarn
    } else if app_dir.join("bun.lockb").exists() || app_dir.join("bun.lock").exists() {
        PackageManager::Bun
    } else {
        PackageManager::Pnpm
    }
}

fn read_package_json(app_dir: &Path) -> Result<Option<PackageJson>> {
    let path = app_dir.join("package.json");
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    Ok(Some(serde_json::from_str(&content)?))
}

fn read_cargo_toml(path: &Path) -> Result<Option<CargoToml>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    Ok(Some(toml::from_str(&content)?))
}

fn read_tauri_config(src_tauri: &Path) -> Result<Option<TauriConfig>> {
    for name in ["tauri.conf.json", "tauri.conf.json5"] {
        let path = src_tauri.join(name);
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            if let Ok(config) = serde_json::from_str::<TauriConfig>(&content) {
                return Ok(Some(config));
            }
        }
    }

    Ok(None)
}
