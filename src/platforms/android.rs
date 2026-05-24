use anyhow::Result;
use std::path::Path;

use crate::config::BuildStrategy;
use crate::container::{self, ContainerRun};
use crate::context::BuildContext;
use crate::host;
use crate::platforms::common;

pub fn build(ctx: &BuildContext) -> Result<()> {
    match host::resolve_strategy(ctx.config.android.strategy, "android") {
        BuildStrategy::Disabled => anyhow::bail!("Android build is disabled."),
        BuildStrategy::Auto => unreachable!("auto strategy must be resolved before build"),
        BuildStrategy::HostOnly => build_host(ctx),
        BuildStrategy::Container => build_container(ctx),
    }
}

fn command(ctx: &BuildContext) -> String {
    ctx.config.android.command.clone().unwrap_or_else(|| {
        let mut parts = vec![
            ctx.metadata.package_manager.tauri_command().to_string(),
            "android".into(),
            "build".into(),
            "--ci".into(),
        ];

        if ctx.config.android.apk {
            parts.push("--apk".into());
        }
        if ctx.config.android.aab {
            parts.push("--aab".into());
        }
        if ctx.config.android.split_per_abi {
            parts.push("--split-per-abi".into());
        }
        if !ctx.config.android.targets.is_empty() {
            parts.push("--target".into());
            parts.extend(ctx.config.android.targets.iter().cloned());
        }
        parts.extend(ctx.args.tauri_args.iter().cloned());
        parts.join(" ")
    })
}

fn build_container(ctx: &BuildContext) -> Result<()> {
    let image = ctx
        .config
        .android
        .image
        .clone()
        .unwrap_or_else(|| ctx.config.container.android_image.clone());

    container::ensure_image_hint(&image, Path::new("templates/Containerfile.android"));

    let script = format!(
        r#"
export ANDROID_HOME="${{ANDROID_HOME:-/opt/android-sdk}}"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export NDK_HOME="${{NDK_HOME:-$ANDROID_HOME/ndk/${{ANDROID_NDK_VERSION:-27.0.11718014}}}}"
export PATH="$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin:$PATH"
{command}
{copy}
"#,
        command = command(ctx),
        copy = container::copy_artifact_script("android")
    );

    container::run(ctx, ContainerRun::new(ctx, "android", image, script))
}

fn build_host(ctx: &BuildContext) -> Result<()> {
    host::run_shell(ctx, "android", &command(ctx))?;
    common::copy_host_artifacts(ctx, "android")
}
