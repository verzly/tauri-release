use anyhow::Result;
use std::path::Path;

use crate::config::BuildStrategy;
use crate::container::{self, ContainerRun};
use crate::context::BuildContext;
use crate::host;
use crate::platforms::common;

pub fn build(ctx: &BuildContext) -> Result<()> {
    match host::resolve_strategy(ctx.config.windows.strategy, "windows") {
        BuildStrategy::Disabled => anyhow::bail!("Windows build is disabled."),
        BuildStrategy::Auto => unreachable!("auto strategy must be resolved before build"),
        BuildStrategy::HostOnly => build_host(ctx),
        BuildStrategy::Container => build_container(ctx),
    }
}

fn command(ctx: &BuildContext) -> String {
    ctx.config.windows.command.clone().unwrap_or_else(|| {
        format!(
            "{} build{}{}{}",
            ctx.metadata.package_manager.tauri_command(),
            common::target_arg(&ctx.config.windows.targets),
            common::bundles_arg(&ctx.config.windows.bundles),
            common::extra_tauri_args(ctx)
        )
    })
}

fn build_container(ctx: &BuildContext) -> Result<()> {
    let image = ctx
        .config
        .windows
        .image
        .clone()
        .unwrap_or_else(|| ctx.config.container.windows_image.clone());
    container::ensure_image_hint(&image, Path::new("templates/Containerfile.windows"));

    let script = format!(
        r#"
{command}
{copy}
"#,
        command = command(ctx),
        copy = container::copy_artifact_script("windows")
    );

    container::run(ctx, ContainerRun::new(ctx, "windows", image, script))
}

fn build_host(ctx: &BuildContext) -> Result<()> {
    host::run_shell(ctx, "windows", &command(ctx))?;
    common::copy_host_artifacts(ctx, "windows")
}
