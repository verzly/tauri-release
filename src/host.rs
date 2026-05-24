use anyhow::{Context, Result};
use std::fs;
use std::process::{Command, Stdio};

use crate::context::BuildContext;

pub fn run_shell(ctx: &BuildContext, platform: &str, command_line: &str) -> Result<()> {
    let out = ctx.platform_output_dir(platform);
    fs::create_dir_all(&out)?;

    if ctx.args.dry_run {
        println!("dry-run host:{} -> {}", platform, command_line);
        return Ok(());
    }

    let mut command = if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command_line]);
        cmd
    } else {
        let mut cmd = Command::new("bash");
        cmd.args(["-lc", command_line]);
        cmd
    };

    command.current_dir(ctx.root_dir.join(&ctx.project_dir));
    command.env("TAURI_RELEASE_OUT", &out);
    command.env("CARGO_TARGET_DIR", ctx.cache_dir.join(platform).join("cargo-target"));

    for (key, value) in &ctx.config.project.env {
        command.env(key, value);
    }

    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let status = command
        .status()
        .with_context(|| format!("failed to execute host command for {}", platform))?;

    if !status.success() {
        anyhow::bail!("host build failed for {}", platform);
    }

    Ok(())
}
