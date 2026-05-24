use anyhow::Result;
use std::path::Path;

use crate::config::BuildStrategy;
use crate::container::{self, ContainerRun};
use crate::context::BuildContext;
use crate::host;
use crate::platforms::common;

pub fn build(ctx: &BuildContext) -> Result<()> {
    match host::resolve_strategy(ctx.config.ios.strategy, "ios") {
        BuildStrategy::Disabled => anyhow::bail!("iOS build is disabled."),
        BuildStrategy::Auto => unreachable!("auto strategy must be resolved before build"),
        BuildStrategy::HostOnly => build_host(ctx),
        BuildStrategy::Container => build_container(ctx),
    }
}

fn command(ctx: &BuildContext) -> String {
    ctx.config.ios.command.clone().unwrap_or_else(|| {
        let targets = if ctx.config.ios.targets.is_empty() {
            String::new()
        } else {
            format!(" --target {}", ctx.config.ios.targets.join(" "))
        };
        format!(
            "{} ios build --ci{}{}",
            ctx.metadata.package_manager.tauri_command(),
            targets,
            common::extra_tauri_args(ctx)
        )
    })
}

fn build_container(ctx: &BuildContext) -> Result<()> {
    let image = ctx
        .config
        .ios
        .image
        .clone()
        .unwrap_or_else(|| ctx.config.container.ios_image.clone());
    container::ensure_image_hint(&image, Path::new("templates/Containerfile.ios"));

    let script = format!(
        r#"
{command}
{copy}
"#,
        command = command(ctx),
        copy = container::copy_artifact_script("ios")
    );

    container::run(ctx, ContainerRun::new(ctx, "ios", image, script))
}

fn build_host(ctx: &BuildContext) -> Result<()> {
    host::run_shell(ctx, "ios", &command(ctx))?;
    common::copy_host_artifacts(ctx, "ios")
}
