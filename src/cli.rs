use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "tauri-release")]
#[command(bin_name = "tauri-release")]
#[command(author, version, about = "Containerized release builder for Tauri artifacts")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub build: BuildArgs,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build release artifacts. This is also the default command.
    Build(BuildArgs),
    /// Print the resolved release plan without running containers.
    Plan(BuildArgs),
    /// Create a tauri-release.toml example config.
    Init(InitArgs),
    /// Remove shared tool/cache directories used by tauri-release.
    #[command(name = "clean-cache")]
    CleanCache(CleanCacheArgs),
}

#[derive(Args, Clone, Debug, Default)]
pub struct BuildArgs {
    /// Path to tauri-release.toml.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Tauri app root. For monorepos this is usually apps/desktop or apps/mobile.
    #[arg(long)]
    pub project_dir: Option<PathBuf>,

    /// Output directory. Defaults to dist/<version>.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Container engine to use.
    #[arg(long, value_enum)]
    pub engine: Option<ContainerEngine>,

    /// Package manager used by the Tauri frontend.
    #[arg(long, value_enum)]
    pub package_manager: Option<PackageManager>,

    /// Build Linux desktop bundles. Auto strategy uses the native host on Linux, otherwise a container.
    #[arg(long, default_value_t = false)]
    pub linux: bool,

    /// Build Android APK/AAB artifacts. Defaults to a container strategy.
    #[arg(long, default_value_t = false)]
    pub android: bool,

    /// Build Windows artifacts. Auto strategy uses a Windows host on Windows, otherwise a container.
    #[arg(long, default_value_t = false)]
    pub windows: bool,

    /// Build macOS artifacts. Auto strategy uses a macOS host on macOS, otherwise a container.
    #[arg(long, default_value_t = false)]
    pub macos: bool,

    /// Build iOS artifacts. Auto strategy uses a macOS host, otherwise a container.
    #[arg(long, default_value_t = false)]
    pub ios: bool,

    /// Build all supported targets from config.
    #[arg(long, default_value_t = false)]
    pub all: bool,

    /// Build APKs when Android is selected.
    #[arg(long, default_value_t = false)]
    pub apk: bool,

    /// Build AABs when Android is selected.
    #[arg(long, default_value_t = false)]
    pub aab: bool,

    /// Split Android APK/AAB per ABI.
    #[arg(long, default_value_t = false)]
    pub split_per_abi: bool,

    /// Android targets, for example: aarch64 armv7 i686 x86_64.
    #[arg(long = "android-target")]
    pub android_targets: Vec<String>,

    /// Extra argument passed to the Tauri CLI. Repeatable.
    #[arg(long = "tauri-arg")]
    pub tauri_args: Vec<String>,

    /// Do not remove the output directory before building.
    #[arg(long, default_value_t = false)]
    pub keep_output: bool,

    /// Print commands without executing them.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Keep temporary container work directories where possible.
    #[arg(long, default_value_t = false)]
    pub keep_workdir: bool,

    /// Show container and build command output.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Config file path to create.
    #[arg(default_value = "tauri-release.toml")]
    pub path: PathBuf,

    /// Overwrite an existing config file.
    #[arg(short, long, default_value_t = false)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct CleanCacheArgs {
    /// Cache directory to remove. Defaults to the tauri-release cache root.
    #[arg(long)]
    pub cache_dir: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum, Eq, PartialEq)]
pub enum ContainerEngine {
    Podman,
    Docker,
}

impl ContainerEngine {
    pub fn executable(self) -> &'static str {
        match self {
            ContainerEngine::Podman => "podman",
            ContainerEngine::Docker => "docker",
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum, Eq, PartialEq)]
pub enum PackageManager {
    Pnpm,
    Npm,
    Yarn,
    Bun,
    Cargo,
}

impl PackageManager {
    pub fn tauri_command(self) -> &'static str {
        match self {
            PackageManager::Pnpm => "pnpm tauri",
            PackageManager::Npm => "npm run tauri --",
            PackageManager::Yarn => "yarn tauri",
            PackageManager::Bun => "bun tauri",
            PackageManager::Cargo => "cargo tauri",
        }
    }

    pub fn install_command(self) -> &'static str {
        match self {
            PackageManager::Pnpm => "corepack enable && pnpm install --frozen-lockfile",
            PackageManager::Npm => "npm ci",
            PackageManager::Yarn => "corepack enable && yarn install --immutable",
            PackageManager::Bun => "bun install --frozen-lockfile",
            PackageManager::Cargo => "cargo fetch",
        }
    }
}
