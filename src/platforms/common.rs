use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::context::BuildContext;
use crate::util::shell_quote;

pub fn copy_host_artifacts(ctx: &BuildContext, platform: &str) -> Result<()> {
    let out = ctx.platform_output_dir(platform);
    fs::create_dir_all(&out)?;

    let sources = host_artifact_sources(ctx, platform);
    let mut copied = 0usize;
    for source in &sources {
        if !source.exists() {
            continue;
        }
        copied += crate::artifacts::copy_matching_files(source, &out, &ctx.config.artifacts)?;
    }

    let existing = crate::artifacts::count_release_artifacts(&out, &ctx.config.artifacts)?;
    if existing == 0 && copied == 0 && !ctx.config.artifacts.allow_empty {
        let searched = sources
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        anyhow::bail!(
            "no release artifacts were collected for {}. Searched: {}. Check the build output or set artifacts.allow_empty = true intentionally.",
            platform,
            searched
        );
    }

    Ok(())
}

fn host_artifact_sources(ctx: &BuildContext, platform: &str) -> Vec<PathBuf> {
    let project_dir = ctx.root_dir.join(&ctx.project_dir);
    let cargo_target_dir = ctx.cache_dir.join(platform).join("cargo-target");
    let mut sources = Vec::new();

    add_tauri_bundle_sources(&mut sources, &cargo_target_dir);
    add_tauri_bundle_sources(&mut sources, &project_dir.join("src-tauri").join("target"));

    if matches!(platform, "android") {
        add_source(&mut sources, project_dir.join("src-tauri").join("gen").join("android").join("app").join("build").join("outputs"));
        add_source(&mut sources, project_dir.join("dist").join("android"));
    }

    if matches!(platform, "ios") {
        add_source(&mut sources, project_dir.join("src-tauri").join("gen").join("apple").join("build"));
    }

    sources
}

fn add_tauri_bundle_sources(sources: &mut Vec<PathBuf>, target_dir: &Path) {
    add_source(sources, target_dir.join("release").join("bundle"));

    let Ok(entries) = fs::read_dir(target_dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            add_source(sources, path.join("release").join("bundle"));
        }
    }
}

fn add_source(sources: &mut Vec<PathBuf>, source: PathBuf) {
    if sources.iter().any(|item| item == &source) {
        return;
    }

    sources.push(source);
}

pub fn bundles_arg(bundles: &[String]) -> String {
    if bundles.is_empty() {
        String::new()
    } else {
        format!(" --bundles {}", shell_quote(&bundles.join(",")))
    }
}

pub fn target_arg(targets: &[String]) -> String {
    if targets.is_empty() {
        String::new()
    } else {
        format!(
            " --target {}",
            targets.iter().map(|target| shell_quote(target)).collect::<Vec<_>>().join(" ")
        )
    }
}

pub fn extra_tauri_args(ctx: &BuildContext) -> String {
    if ctx.args.tauri_args.is_empty() {
        String::new()
    } else {
        format!(" {}", shell_args(ctx.args.tauri_args.iter().map(String::as_str)))
    }
}

pub fn shell_args<'a>(args: impl IntoIterator<Item = &'a str>) -> String {
    args.into_iter().map(shell_quote).collect::<Vec<_>>().join(" ")
}
