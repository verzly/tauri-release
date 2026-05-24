use anyhow::Result;
use std::path::Path;

use crate::container::{self, ContainerRun};
use crate::context::BuildContext;

pub fn build(ctx: &BuildContext) -> Result<()> {
    let image = ctx
        .config
        .linux
        .image
        .clone()
        .unwrap_or_else(|| ctx.config.container.linux_image.clone());

    container::ensure_image_hint(&image, Path::new("templates/Containerfile.linux"));

    let command = ctx.config.linux.command.clone().unwrap_or_else(|| {
        let bundles = if ctx.config.linux.bundles.is_empty() {
            String::new()
        } else {
            format!(" --bundles {}", ctx.config.linux.bundles.join(","))
        };
        format!(
            "{} build{} {}",
            ctx.metadata.package_manager.tauri_command(),
            bundles,
            ctx.args.tauri_args.join(" ")
        )
    });

    let script = format!(
        r#"
{command}
{copy}
"#,
        command = command,
        copy = container::copy_artifact_script("linux")
    );

    container::run(ctx, ContainerRun::new(ctx, "linux", image, script))
}
