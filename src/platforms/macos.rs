use anyhow::Result;
use std::path::Path;

use crate::config::BuildStrategy;
use crate::container::{self, ContainerRun};
use crate::context::BuildContext;
use crate::host;
use crate::platforms::common;

pub fn build(ctx: &BuildContext) -> Result<()> {
    match host::resolve_strategy(ctx.config.macos.strategy, "macos") {
        BuildStrategy::Disabled => anyhow::bail!("macOS build is disabled."),
        BuildStrategy::Auto => unreachable!("auto strategy must be resolved before build"),
        BuildStrategy::HostOnly => build_host(ctx),
        BuildStrategy::Container => build_container(ctx),
    }
}

fn command(ctx: &BuildContext) -> String {
    ctx.config.macos.command.clone().unwrap_or_else(|| {
        format!(
            "{} build{}{}{}",
            ctx.metadata.package_manager.tauri_command(),
            common::target_arg(&ctx.config.macos.targets),
            common::bundles_arg(&ctx.config.macos.bundles),
            common::extra_tauri_args(ctx)
        )
    })
}

fn build_container(ctx: &BuildContext) -> Result<()> {
    let image = ctx
        .config
        .macos
        .image
        .clone()
        .unwrap_or_else(|| ctx.config.container.macos_image.clone());
    container::ensure_image_hint(&image, Path::new("templates/Containerfile.macos"));

    let script = format!(
        r#"
{command}
{copy}
"#,
        command = command(ctx),
        copy = container::copy_artifact_script("macos")
    );

    container::run(ctx, ContainerRun::new(ctx, "macos", image, script))
}

fn build_host(ctx: &BuildContext) -> Result<()> {
    host::run_shell(ctx, "macos", &command(ctx))?;
    common::copy_host_artifacts(ctx, "macos")
}
