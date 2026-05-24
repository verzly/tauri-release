use anyhow::{bail, Context, Result};
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

pub fn copy_matching_files(source: &Path, out: &Path, config: &ArtifactConfig) -> Result<usize> {
    fs::create_dir_all(out)?;

    let mut copied = 0usize;
    for entry in WalkDir::new(source).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() || !is_release_artifact(path, config) {
            continue;
        }

        // Avoid recursively copying files that already live in the output dir.
        if is_inside(path, out) {
            continue;
        }

        let relative = path.strip_prefix(source).unwrap_or(path);
        let dest = destination_for(path, relative, out, config)?;

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        if dest.exists() {
            bail!(
                "artifact collision: {} would overwrite {}. Enable artifacts.keep_relative_paths or make artifact names unique.",
                path.display(),
                dest.display()
            );
        }

        fs::copy(path, &dest).with_context(|| {
            format!(
                "failed to copy release artifact {} to {}",
                path.display(),
                dest.display()
            )
        })?;
        copied += 1;
    }

    Ok(copied)
}

pub fn normalize_artifacts(ctx: &BuildContext) -> Result<()> {
    for platform in ctx.selected_platform_names() {
        let count = ensure_platform_artifacts(ctx, platform)?;
        println!(
            "{} {} artifact(s) in {}",
            "Collected".green(),
            count,
            ctx.platform_output_dir(platform).display()
        );
    }

    Ok(())
}

pub fn ensure_platform_artifacts(ctx: &BuildContext, platform: &str) -> Result<usize> {
    let dir = ctx.platform_output_dir(platform);
    let count = if dir.exists() {
        count_release_artifacts(&dir, &ctx.config.artifacts)?
    } else {
        0
    };

    if count == 0 && !ctx.config.artifacts.allow_empty {
        bail!(
            "no release artifacts were collected for {}. Check the build output or set artifacts.allow_empty = true intentionally.",
            platform
        );
    }

    Ok(count)
}

pub fn count_release_artifacts(dir: &Path, config: &ArtifactConfig) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }

    let mut count = 0usize;
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && is_release_artifact(path, config) {
            count += 1;
        }
    }

    Ok(count)
}

pub fn write_manifest(ctx: &BuildContext) -> Result<()> {
    let mut artifacts = Vec::new();
    for entry in WalkDir::new(&ctx.output_dir).min_depth(1).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) == Some("sha256") {
            continue;
        }

        let rel = path.strip_prefix(&ctx.output_dir).unwrap_or(path);
        let platform = rel
            .components()
            .next()
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

pub fn is_release_artifact(path: &Path, config: &ArtifactConfig) -> bool {
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

fn destination_for(
    path: &Path,
    relative: &Path,
    out: &Path,
    config: &ArtifactConfig,
) -> Result<PathBuf> {
    if config.keep_relative_paths {
        return Ok(out.join(relative));
    }

    let name = path
        .file_name()
        .with_context(|| format!("release artifact has no file name: {}", path.display()))?;
    Ok(out.join(name))
}

fn is_inside(path: &Path, parent: &Path) -> bool {
    let Ok(path) = path.canonicalize() else {
        return false;
    };
    let Ok(parent) = parent.canonicalize() else {
        return false;
    };

    path.starts_with(parent)
}
