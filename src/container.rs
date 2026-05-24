use anyhow::{Context, Result};
use colored::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::context::BuildContext;
use crate::util::{path_for_container, shell_quote};

#[derive(Clone, Debug)]
pub struct ContainerRun {
    pub name: String,
    pub image: String,
    pub platform: String,
    pub script: String,
    pub env: BTreeMap<String, String>,
}

impl ContainerRun {
    pub fn new(ctx: &BuildContext, platform: &str, image: impl Into<String>, script: impl Into<String>) -> Self {
        Self {
            name: format!("{}-{}", ctx.session_prefix, platform),
            image: image.into(),
            platform: platform.to_string(),
            script: script.into(),
            env: BTreeMap::new(),
        }
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

pub fn run(ctx: &BuildContext, spec: ContainerRun) -> Result<()> {
    let platform_out = ctx.platform_output_dir(&spec.platform);
    fs::create_dir_all(&platform_out)?;
    fs::create_dir_all(&ctx.cache_dir)?;

    let root = path_for_container(&ctx.root_dir);
    let out = path_for_container(&platform_out);
    let cache = path_for_container(&ctx.cache_dir.join(&spec.platform));
    fs::create_dir_all(ctx.cache_dir.join(&spec.platform))?;

    let mut args: Vec<String> = vec![
        "run".into(),
        "--rm".into(),
        "--name".into(),
        spec.name.clone(),
    ];

    if ctx.config.container.userns_keep_id && ctx.engine.executable() == "podman" {
        args.push("--userns=keep-id".into());
    }

    if let Some(network) = &ctx.config.container.network {
        args.push("--network".into());
        args.push(network.clone());
    }

    args.extend([
        "-v".into(),
        format!("{}:/src:ro,z", root),
        "-v".into(),
        format!("{}:/out:z", out),
        "-v".into(),
        format!("{}:/cache:z", cache),
        "-e".into(),
        "CI=true".into(),
        "-e".into(),
        "CARGO_HOME=/cache/cargo-home".into(),
        "-e".into(),
        "CARGO_TARGET_DIR=/cache/cargo-target".into(),
        "-e".into(),
        "GRADLE_USER_HOME=/cache/gradle".into(),
        "-e".into(),
        "PNPM_STORE_DIR=/cache/pnpm-store".into(),
    ]);

    for (key, value) in &ctx.config.project.env {
        args.push("-e".into());
        args.push(format!("{}={}", key, value));
    }

    for (key, value) in &spec.env {
        args.push("-e".into());
        args.push(format!("{}={}", key, value));
    }

    args.extend([
        "-w".into(),
        "/work".into(),
        spec.image.clone(),
        "bash".into(),
        "-lc".into(),
        wrap_script(ctx, &spec.script),
    ]);

    if ctx.args.dry_run {
        println!("{} {} {}", "dry-run".yellow(), ctx.engine.executable(), args.join(" "));
        return Ok(());
    }

    println!("{} {}", "Container".cyan().bold(), spec.name);
    if ctx.args.verbose {
        println!("{} {}", "image:".dimmed(), spec.image);
    }

    let mut command = Command::new(ctx.engine.executable());
    command.args(&args);

    if ctx.args.verbose {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    } else {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

    let status = command
        .status()
        .with_context(|| format!("failed to execute {}", ctx.engine.executable()))?;

    if !status.success() {
        anyhow::bail!("container build failed for {}", spec.platform);
    }

    Ok(())
}

fn wrap_script(ctx: &BuildContext, body: &str) -> String {
    let project_dir = path_for_container(&ctx.project_dir);
    let install = ctx
        .config
        .project
        .install_command
        .clone()
        .unwrap_or_else(|| ctx.metadata.package_manager.install_command().to_string());

    format!(
        r#"
set -euo pipefail
mkdir -p /work/project
# Copy source into disposable container workspace. The host project stays read-only.
tar -C /src -cf - . | tar -C /work/project -xf -
cd /work/project/{project_dir}
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_HOME="${{CARGO_HOME:-/cache/cargo-home}}"
export CARGO_TARGET_DIR="${{CARGO_TARGET_DIR:-/cache/cargo-target}}"
export GRADLE_USER_HOME="${{GRADLE_USER_HOME:-/cache/gradle}}"
export PNPM_STORE_DIR="${{PNPM_STORE_DIR:-/cache/pnpm-store}}"
{install}
{body}
"#,
        project_dir = shell_quote(&project_dir),
        install = install,
        body = body,
    )
}

pub fn copy_artifact_script(platform: &str) -> String {
    let platform = shell_quote(platform);
    format!(
        r#"
mkdir -p /out
find . -type f \( \
  -name '*.AppImage' -o -name '*.deb' -o -name '*.rpm' -o -name '*.exe' -o -name '*.msi' \
  -o -name '*.dmg' -o -name '*.zip' -o -name '*.apk' -o -name '*.aab' -o -name '*.ipa' \
  -o -name '*.sig' -o -name 'latest.json' -o -name 'release.json' \
\) -print0 | while IFS= read -r -d '' file; do
  base="$(basename "$file")"
  cp -f "$file" "/out/$base"
  echo "artifact:{platform}:$base"
done
"#,
        platform = platform
    )
}

pub fn ensure_image_hint(image: &str, template_path: &Path) {
    if image.starts_with("ghcr.io/your-org/") {
        println!(
            "{} image '{}' is a placeholder. Build it from {} or override it in tauri-release.toml.",
            "hint:".yellow(),
            image,
            template_path.display()
        );
    }
}
