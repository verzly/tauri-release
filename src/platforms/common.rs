use anyhow::Result;
use std::fs;
use std::path::PathBuf;

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

    if copied == 0 && !ctx.config.artifacts.allow_empty {
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
    let mut sources = vec![ctx.root_dir.join(&ctx.project_dir)];
    let cargo_target_dir = ctx.cache_dir.join(platform).join("cargo-target");
    if !sources.iter().any(|source| source == &cargo_target_dir) {
        sources.push(cargo_target_dir);
    }
    sources
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
