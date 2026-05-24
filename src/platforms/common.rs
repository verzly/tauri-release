use anyhow::Result;
use std::fs;

use crate::context::BuildContext;

pub fn copy_host_artifacts(ctx: &BuildContext, platform: &str) -> Result<()> {
    let source = ctx.root_dir.join(&ctx.project_dir);
    let out = ctx.platform_output_dir(platform);
    fs::create_dir_all(&out)?;
    crate::artifacts::copy_matching_files(&source, &out, &ctx.config.artifacts)?;
    Ok(())
}

pub fn bundles_arg(bundles: &[String]) -> String {
    if bundles.is_empty() {
        String::new()
    } else {
        format!(" --bundles {}", bundles.join(","))
    }
}

pub fn target_arg(targets: &[String]) -> String {
    if targets.is_empty() {
        String::new()
    } else {
        format!(" --target {}", targets.join(" "))
    }
}

pub fn extra_tauri_args(ctx: &BuildContext) -> String {
    if ctx.args.tauri_args.is_empty() {
        String::new()
    } else {
        format!(" {}", ctx.args.tauri_args.join(" "))
    }
}
