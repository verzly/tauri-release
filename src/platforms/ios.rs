use anyhow::Result;
use std::fs;

use crate::config::HostStrategy;
use crate::context::BuildContext;
use crate::host;

pub fn build(ctx: &BuildContext) -> Result<()> {
    match ctx.config.ios.strategy {
        HostStrategy::Disabled => {
            anyhow::bail!("iOS build is disabled. Enable [ios] strategy = 'host-only' in tauri-release.toml.");
        }
        HostStrategy::HostOnly => {
            if !cfg!(target_os = "macos") {
                anyhow::bail!("iOS Tauri build requires a macOS host/runner with Xcode.");
            }
            let command = ctx.config.ios.command.clone().unwrap_or_else(|| "pnpm tauri ios build --ci".into());
            host::run_shell(ctx, "ios", &command)?;
            copy_host_artifacts(ctx, "ios")
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
