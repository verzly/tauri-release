use anyhow::{Context, Result};
use colored::*;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::ArtifactConfig;
use crate::context::BuildContext;

#[derive(Serialize)]
struct ReleaseManifest {
    app: String,
    version: String,
    artifacts: Vec<ManifestArtifact>,
}

#[derive(Serialize)]
struct ManifestArtifact {
    platform: String,
    file: String,
    size_bytes: u64,
}

pub fn copy_matching_files(source: &Path, out: &Path, config: &ArtifactConfig) -> Result<()> {
    fs::create_dir_all(out)?;
    for entry in WalkDir::new(source).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() || !is_release_artifact(path, config) {
            continue;
        }

        // Avoid recursively copying files that already live in the output dir.
        if path.starts_with(out) {
            continue;
        }

        let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("artifact");
        let dest = out.join(name);
        fs::copy(path, &dest).with_context(|| format!("failed to copy {}", path.display()))?;
    }
    Ok(())
}

pub fn normalize_artifacts(ctx: &BuildContext) -> Result<()> {
    let selected = ctx.selected_platforms();
    let mut platforms = Vec::new();
    if selected.linux { platforms.push("linux"); }
    if selected.android { platforms.push("android"); }
    if selected.windows { platforms.push("windows"); }
    if selected.macos { platforms.push("macos"); }
    if selected.ios { platforms.push("ios"); }

    for platform in platforms {
        let dir = ctx.platform_output_dir(platform);
        if !dir.exists() {
            continue;
        }

        let mut count = 0usize;
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && is_release_artifact(&path, &ctx.config.artifacts) {
                count += 1;
            }
        }

        println!("{} {} artifact(s) in {}", "Collected".green(), count, dir.display());
    }

    Ok(())
}

pub fn write_manifest(ctx: &BuildContext) -> Result<()> {
    let mut artifacts = Vec::new();
    for entry in WalkDir::new(&ctx.output_dir).min_depth(1).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) == Some("sha256") {
            continue;
        }

        let rel = path.strip_prefix(&ctx.output_dir).unwrap_or(path);
        let platform = rel.components().next()
            .and_then(|component| component.as_os_str().to_str())
            .unwrap_or("unknown")
            .to_string();

        artifacts.push(ManifestArtifact {
            platform,
            file: rel.to_string_lossy().replace('\\', "/"),
            size_bytes: fs::metadata(path)?.len(),
        });
    }

    artifacts.sort_by(|a, b| a.file.cmp(&b.file));

    let manifest = ReleaseManifest {
        app: ctx.metadata.display_name.clone(),
        version: ctx.metadata.version.clone(),
        artifacts,
    };

    let path = ctx.output_dir.join("release-manifest.json");
    fs::write(&path, serde_json::to_string_pretty(&manifest)? + "\n")?;
    println!("{} {}", "Wrote".green(), path.display());
    Ok(())
}

fn is_release_artifact(path: &Path, config: &ArtifactConfig) -> bool {
    let filename = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
    if config.include_files.iter().any(|item| item == filename) {
        return true;
    }

    let lower_name = filename.to_ascii_lowercase();
    config.include_extensions.iter().any(|ext| {
        let ext = ext.trim_start_matches('.').to_ascii_lowercase();
        lower_name.ends_with(&format!(".{}", ext)) || lower_name.ends_with(&format!("-{}", ext))
    })
}
