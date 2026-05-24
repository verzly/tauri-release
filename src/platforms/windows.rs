use anyhow::Result;
use std::fs;

use crate::config::HostStrategy;
use crate::context::BuildContext;
use crate::host;

pub fn build(ctx: &BuildContext) -> Result<()> {
    match ctx.config.windows.strategy {
        HostStrategy::Disabled => {
            anyhow::bail!("windows build is disabled. Enable [windows] strategy = 'host-only' in tauri-release.toml.");
        }
        HostStrategy::HostOnly => {
            if !cfg!(windows) {
                anyhow::bail!("windows Tauri installer build requires a Windows host/runner. This project intentionally does not pretend Linux Podman can reliably build MSI/NSIS artifacts.");
            }
            let command = ctx.config.windows.command.clone().unwrap_or_else(|| "pnpm tauri build".into());
            host::run_shell(ctx, "windows", &command)?;
            copy_host_artifacts(ctx, "windows")
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
