use anyhow::Result;
use std::fs;

use crate::config::HostStrategy;
use crate::context::BuildContext;
use crate::host;

pub fn build(ctx: &BuildContext) -> Result<()> {
    match ctx.config.macos.strategy {
        HostStrategy::Disabled => {
            anyhow::bail!("macOS build is disabled. Enable [macos] strategy = 'host-only' in tauri-release.toml.");
        }
        HostStrategy::HostOnly => {
            if !cfg!(target_os = "macos") {
                anyhow::bail!("macOS Tauri bundles require a macOS host/runner for signing, DMG and notarization.");
            }
            let command = ctx.config.macos.command.clone().unwrap_or_else(|| "pnpm tauri build".into());
            host::run_shell(ctx, "macos", &command)?;
            copy_host_artifacts(ctx, "macos")
        }
    }
}

fn copy_host_artifacts(ctx: &BuildContext, platform: &str) -> Result<()> {
    let source = ctx.root_dir.join(&ctx.project_dir);
    let out = ctx.platform_output_dir(platform);
    fs::create_dir_all(&out)?;
    crate::artifacts::copy_matching_files(&source, &out, &ctx.config.artifacts)?;
    Ok(())
}
