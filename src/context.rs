use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::cli::{BuildArgs, ContainerEngine, PackageManager};
use crate::config::{self, ReleaseConfig};
use crate::project::{self, ProjectMetadata};
use crate::util;

#[derive(Clone, Copy, Debug, Default)]
pub struct SelectedPlatforms {
    pub linux: bool,
    pub android: bool,
    pub windows: bool,
    pub macos: bool,
    pub ios: bool,
}

impl SelectedPlatforms {
    pub fn is_empty(self) -> bool {
        !self.linux && !self.android && !self.windows && !self.macos && !self.ios
    }
}

#[derive(Clone, Debug)]
pub struct BuildContext {
    pub root_dir: PathBuf,
    pub project_dir: PathBuf,
    pub output_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub engine: ContainerEngine,
    pub args: BuildArgs,
    pub config: ReleaseConfig,
    pub metadata: ProjectMetadata,
    pub session_prefix: String,
}

impl BuildContext {
    pub fn from_sources(args: BuildArgs) -> Result<Self> {
        let root_dir = std::env::current_dir().context("failed to determine current directory")?;
        let mut config = config::load_config(args.config.as_deref())?;

        if let Some(project_dir) = args.project_dir.clone() {
            config.project.project_dir = project_dir;
        }
        if let Some(package_manager) = args.package_manager {
            config.project.package_manager = Some(package_manager.into());
        }
        if let Some(engine) = args.engine {
            config.container.engine = engine.into();
        }
        if let Some(output) = args.output.clone() {
            config.output.dir = Some(output);
        }
        if args.keep_output {
            config.output.clean = false;
        }
        if args.apk {
            config.android.apk = true;
        }
        if args.aab {
            config.android.aab = true;
        }
        if args.split_per_abi {
            config.android.split_per_abi = true;
        }
        if !args.android_targets.is_empty() {
            config.android.targets = args.android_targets.clone();
        }

        let package_manager: Option<PackageManager> = config.project.package_manager.map(Into::into);
        let metadata = project::detect_metadata(
            &root_dir,
            &config.project.project_dir,
            config.project.app_name.clone(),
            config.project.version.clone(),
            package_manager,
        )?;

        let output_dir = match config.output.dir.clone() {
            Some(dir) => util::absolutize(dir)?,
            None => root_dir.join("dist").join(&metadata.version),
        };

        let cache_dir = match config.container.cache_dir.clone() {
            Some(dir) => util::absolutize(dir)?,
            None => util::default_cache_dir(),
        };

        Ok(Self {
            root_dir,
            project_dir: config.project.project_dir.clone(),
            output_dir,
            cache_dir,
            engine: config.container.engine.into(),
            args,
            config,
            metadata,
            session_prefix: util::now_session_id("tauri-release"),
        })
    }

    pub fn selected_platforms(&self) -> SelectedPlatforms {
        let explicit = self.args.linux || self.args.android || self.args.windows || self.args.macos || self.args.ios;
        if self.args.all || !explicit {
            return SelectedPlatforms {
                linux: self.config.linux.enabled,
                android: self.config.android.enabled,
                windows: self.config.windows.enabled,
                macos: self.config.macos.enabled,
                ios: self.config.ios.enabled,
            };
        }

        SelectedPlatforms {
            linux: self.args.linux,
            android: self.args.android,
            windows: self.args.windows,
            macos: self.args.macos,
            ios: self.args.ios,
        }
    }


    pub fn selected_platform_names(&self) -> Vec<&'static str> {
        let selected = self.selected_platforms();
        let mut platforms = Vec::new();
        if selected.linux {
            platforms.push("linux");
        }
        if selected.android {
            platforms.push("android");
        }
        if selected.windows {
            platforms.push("windows");
        }
        if selected.macos {
            platforms.push("macos");
        }
        if selected.ios {
            platforms.push("ios");
        }

        platforms
    }

    pub fn prepare_output_dir(&self) -> Result<()> {
        if self.output_dir.exists() && self.config.output.clean {
            fs::remove_dir_all(&self.output_dir)
                .with_context(|| format!("failed to clean {}", self.output_dir.display()))?;
        }

        fs::create_dir_all(&self.output_dir)
            .with_context(|| format!("failed to create {}", self.output_dir.display()))?;
        fs::create_dir_all(&self.cache_dir)
            .with_context(|| format!("failed to create {}", self.cache_dir.display()))?;
        Ok(())
    }

    pub fn platform_output_dir(&self, platform: &str) -> PathBuf {
        self.output_dir.join(platform)
    }

    pub fn print_plan(&self) {
        let selected = self.selected_platforms();
        println!("{}", "Resolved Tauri release plan".green().bold());
        println!("  app:            {}", self.metadata.display_name.cyan());
        println!("  version:        {}", self.metadata.version.cyan());
        println!("  root:           {}", self.root_dir.display());
        println!("  project_dir:    {}", self.project_dir.display());
        println!("  output:         {}", self.output_dir.display());
        println!("  cache:          {}", self.cache_dir.display());
        println!("  engine:         {:?}", self.engine);
        println!("  package:        {:?}", self.metadata.package_manager);
        println!("  host:           {}", util::current_host_platform());
        println!("  targets:");
        println!("    linux:        {} ({:?} -> {:?})", selected.linux, self.config.linux.strategy, crate::host::resolve_strategy(self.config.linux.strategy, "linux"));
        println!("    android:      {} ({:?} -> {:?})", selected.android, self.config.android.strategy, crate::host::resolve_strategy(self.config.android.strategy, "android"));
        println!("    windows:      {} ({:?} -> {:?})", selected.windows, self.config.windows.strategy, crate::host::resolve_strategy(self.config.windows.strategy, "windows"));
        println!("    macos:        {} ({:?} -> {:?})", selected.macos, self.config.macos.strategy, crate::host::resolve_strategy(self.config.macos.strategy, "macos"));
        println!("    ios:          {} ({:?} -> {:?})", selected.ios, self.config.ios.strategy, crate::host::resolve_strategy(self.config.ios.strategy, "ios"));
        println!("  android:");
        println!("    apk:          {}", self.config.android.apk);
        println!("    aab:          {}", self.config.android.aab);
        println!("    split_per_abi:{}", self.config.android.split_per_abi);
        println!("    targets:      {}", self.config.android.targets.join(", "));
    }
}
