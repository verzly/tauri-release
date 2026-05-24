use anyhow::{Context, Result};
use colored::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::config::ArtifactConfig;
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

    let mut args: Vec<String> = vec!["run".into()];

    if !ctx.args.keep_workdir {
        args.push("--rm".into());
    }

    args.extend(["--name".into(), spec.name.clone()]);

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

    if ctx.args.keep_workdir {
        println!(
            "{} kept container {} for inspection",
            "hint:".yellow(),
            spec.name
        );
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

pub fn copy_artifact_script(platform: &str, config: &ArtifactConfig) -> String {
    let extensions = config
        .include_extensions
        .iter()
        .map(|ext| shell_quote(&ext.trim_start_matches('.').to_ascii_lowercase()))
        .collect::<Vec<_>>()
        .join(" ");
    let files = config
        .include_files
        .iter()
        .map(|file| shell_quote(file))
        .collect::<Vec<_>>()
        .join(" ");
    let keep_relative_paths = if config.keep_relative_paths { "true" } else { "false" };
    let allow_empty = if config.allow_empty { "true" } else { "false" };

    format!(
        r#"
mkdir -p /out
ARTIFACT_PLATFORM={platform}
ARTIFACT_KEEP_RELATIVE_PATHS={keep_relative_paths}
ARTIFACT_ALLOW_EMPTY={allow_empty}
ARTIFACT_EXTENSIONS=({extensions})
ARTIFACT_FILES=({files})

is_release_artifact() {{
  local file="$1"
  local base
  base="$(basename "$file")"

  local allowed_file
  for allowed_file in "${{ARTIFACT_FILES[@]}}"; do
    if [ "$base" = "$allowed_file" ]; then
      return 0
    fi
  done

  local lower_base
  lower_base="$(printf '%s' "$base" | tr '[:upper:]' '[:lower:]')"

  local ext
  for ext in "${{ARTIFACT_EXTENSIONS[@]}}"; do
    case "$lower_base" in
      *."$ext"|*-"$ext") return 0 ;;
    esac
  done

  return 1
}}

copy_count=0
ARTIFACT_SOURCES=()

add_source() {{
  local source="$1"
  if [ ! -d "$source" ]; then
    return 0
  fi

  local existing
  for existing in "${{ARTIFACT_SOURCES[@]}}"; do
    if [ "$existing" = "$source" ]; then
      return 0
    fi
  done

  ARTIFACT_SOURCES+=("$source")
}}

add_tauri_bundle_sources() {{
  local target_dir="$1"
  add_source "$target_dir/release/bundle"

  if [ -d "$target_dir" ]; then
    local candidate
    for candidate in "$target_dir"/*/release/bundle; do
      add_source "$candidate"
    done
  fi
}}

if [ -n "${{CARGO_TARGET_DIR:-}}" ]; then
  add_tauri_bundle_sources "$CARGO_TARGET_DIR"
fi

add_tauri_bundle_sources "src-tauri/target"

if [ "$ARTIFACT_PLATFORM" = "android" ]; then
  add_source "src-tauri/gen/android/app/build/outputs"
fi

if [ "$ARTIFACT_PLATFORM" = "ios" ]; then
  add_source "src-tauri/gen/apple/build"
fi

copy_from_source() {{
  local source="$1"
  while IFS= read -r -d '' file; do
    if ! is_release_artifact "$file"; then
      continue
    fi

    rel="${{file#${{source}}/}}"
    rel="${{rel#./}}"
    if [ "$ARTIFACT_KEEP_RELATIVE_PATHS" = "true" ]; then
      dest="/out/$rel"
    else
      dest="/out/$(basename "$file")"
    fi

    if [ -e "$dest" ]; then
      echo "::error::Artifact collision: $file would overwrite $dest. Enable artifacts.keep_relative_paths or make artifact names unique."
      exit 1
    fi

    mkdir -p "$(dirname "$dest")"
    cp -f "$file" "$dest"
    copy_count=$((copy_count + 1))
    echo "artifact:$ARTIFACT_PLATFORM:${{dest#/out/}}"
  done < <(find "$source" -type f -print0)
}}

for source in "${{ARTIFACT_SOURCES[@]}}"; do
  copy_from_source "$source"
done

existing_count=0
while IFS= read -r -d '' existing_file; do
  if is_release_artifact "$existing_file"; then
    existing_count=$((existing_count + 1))
  fi
done < <(find /out -type f -print0)

if [ "$existing_count" -eq 0 ] && [ "$copy_count" -eq 0 ] && [ "$ARTIFACT_ALLOW_EMPTY" != "true" ]; then
  searched="${{ARTIFACT_SOURCES[*]:-none}}"
  echo "::error::No release artifacts were collected for $ARTIFACT_PLATFORM. Searched: $searched"
  exit 1
fi"#,
        platform = shell_quote(platform),
        keep_relative_paths = keep_relative_paths,
        allow_empty = allow_empty,
        extensions = extensions,
        files = files,
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
