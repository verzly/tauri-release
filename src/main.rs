mod artifacts;
mod checksums;
mod cli;
mod config;
mod context;
mod container;
mod host;
mod platforms;
mod project;
mod util;

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use std::env;
use std::ffi::OsString;
use std::process::{Command, Stdio};

use crate::cli::{Cli, Commands};
use crate::context::BuildContext;

fn normalized_args() -> Vec<OsString> {
    env::args_os().collect()
}

fn install_ctrlc_handler(session_prefix: String) -> Result<()> {
    ctrlc::set_handler(move || {
        eprintln!("\n{}", "Aborting. Cleaning up running release containers...".red().bold());
        let _ = Command::new("podman")
            .args(["ps", "-q", "--filter", &format!("name={}", session_prefix)])
            .output()
            .map(|output| {
                let ids = String::from_utf8_lossy(&output.stdout);
                for id in ids.lines().filter(|line| !line.trim().is_empty()) {
                    let _ = Command::new("podman")
                        .args(["kill", id])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
            });
        std::process::exit(130);
    })
    .context("failed to install Ctrl-C handler")
}

fn run_build(ctx: &BuildContext) -> Result<()> {
    let selected = ctx.selected_platforms();
    if selected.is_empty() {
        anyhow::bail!("no release target selected");
    }

    println!(
        "{} {} -> {}",
        "Starting Tauri release".green().bold(),
        ctx.metadata.display_name.cyan(),
        ctx.output_dir.display().to_string().yellow()
    );

    ctx.prepare_output_dir()?;

    if selected.linux {
        platforms::linux::build(ctx)?;
    }
    if selected.android {
        platforms::android::build(ctx)?;
    }
    if selected.windows {
        platforms::windows::build(ctx)?;
    }
    if selected.macos {
        platforms::macos::build(ctx)?;
    }
    if selected.ios {
        platforms::ios::build(ctx)?;
    }

    artifacts::normalize_artifacts(ctx)?;

    if ctx.config.output.sha256 {
        checksums::generate_checksums(&ctx.output_dir)?;
    }

    if ctx.config.output.manifest {
        artifacts::write_manifest(ctx)?;
    }

    println!("{}", "Release completed.".green().bold());
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse_from(normalized_args());

    match &cli.command {
        Some(Commands::Init(args)) => {
            config::write_example_config(&args.path, args.force)?;
            println!("{} {}", "Created".green(), args.path.display());
            Ok(())
        }
        Some(Commands::CleanCache(args)) => {
            let cache_dir = args.cache_dir.clone().unwrap_or_else(util::default_cache_dir);
            if cache_dir.exists() {
                std::fs::remove_dir_all(&cache_dir)
                    .with_context(|| format!("failed to remove {}", cache_dir.display()))?;
                println!("{} {}", "Removed cache".green(), cache_dir.display());
            } else {
                println!("{} {}", "No cache found at".yellow(), cache_dir.display());
            }
            Ok(())
        }
        Some(Commands::Plan(args)) => {
            let ctx = BuildContext::from_sources(args.clone())?;
            install_ctrlc_handler(ctx.session_prefix.clone())?;
            ctx.print_plan();
            Ok(())
        }
        Some(Commands::Build(args)) => {
            let ctx = BuildContext::from_sources(args.clone())?;
            install_ctrlc_handler(ctx.session_prefix.clone())?;
            run_build(&ctx)
        }
        None => {
            let ctx = BuildContext::from_sources(cli.build.clone())?;
            install_ctrlc_handler(ctx.session_prefix.clone())?;
            run_build(&ctx)
        }
    }
}
