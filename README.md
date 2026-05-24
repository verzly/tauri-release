# verzly/tauri-release

`verzly/tauri-release` is a release orchestrator for Tauri applications. It builds final desktop and mobile artifacts in disposable Podman/Docker sessions, then copies only publishable files into `dist/<version>/`.

It is designed for projects where `src-tauri/target`, `src-tauri/gen/android/app/build`, Gradle output, frontend dependencies, platform SDK output, and other generated files should not keep growing inside the source tree.

It provides an integrated release flow with built-in support for:

- **Linux desktop bundles** through a host-aware Podman/Docker strategy
- **Android APK/AAB builds** through a containerized Android SDK/NDK strategy
- **Windows, macOS, and iOS targets** through the same Podman/Docker orchestration model
- **Native-host preference** when the current host already matches the target platform
- **Artifact whitelisting** so only installers, packages, signatures, metadata, and checksums are kept
- **SHA-256 checksums** for collected release files
- **Release manifest generation** for predictable publishing

Build inside a disposable workspace, keep only executable artifacts, and let the container session remove everything else.

- [How it works](#how-it-works)
  - [Build strategy](#build-strategy)
  - [Container builds](#container-builds)
  - [Native host builds](#native-host-builds)
  - [Cleanup](#cleanup)
- [Get started](#get-started)
  - [Install](#install)
  - [Create config](#create-config)
  - [Upgrade](#upgrade)
- [Usage](#usage)
  - [Build Linux and Android](#build-linux-and-android)
  - [Build Windows, macOS, and iOS](#build-windows-macos-and-ios)
  - [Monorepo projects](#monorepo-projects)
  - [Android APK and AAB](#android-apk-and-aab)
  - [Artifact collection](#artifact-collection)
  - [Checksums](#checksums)
  - [Cache](#cache)
  - [Configuration](#configuration)
  - [Container images](#container-images)
- [Known issues](#known-issues)
- [Contributing](#contributing)

Read on to understand why `verzly/tauri-release` exists and how build isolation works. Or jump straight to [Get started](#get-started) for installation and first build steps.

## How it works

Tauri release builds can create a large amount of intermediate output. Desktop builds use Cargo, frontend package managers, and Tauri bundlers. Android builds also generate a Gradle project under `src-tauri/gen/android`, compile native Rust libraries, package APK/AAB files, and keep Gradle intermediates.

`verzly/tauri-release` avoids polluting the working tree by running expensive builds in an isolated workspace:

```text
host project  ->  mounted read-only at /src
container     ->  copies /src into /work/project
build output  ->  collected into /out
host dist     ->  receives only whitelisted files
```

The source project is treated as input. The release directory is treated as output. Everything else is disposable.

### Build strategy

Each platform has a `strategy` setting:

```toml
strategy = "auto"       # prefer native host when host matches, otherwise container
strategy = "container"  # always use Podman/Docker
strategy = "host-only"  # always run the command on the host
strategy = "disabled"   # reject the build target
```

`auto` is the recommended default for desktop-style targets. It respects the current host:

| Target | Linux host | Windows host | macOS host |
|---|---|---|---|
| Linux | host | container | container |
| Windows | container | host | container |
| macOS | container | container | host |
| iOS | container | container | host |
| Android | container by default | container by default | container by default |

This means a Windows host can build Windows artifacts natively without unnecessary Windows cross-compiling through Podman, and a macOS host can build macOS/iOS artifacts natively. On other hosts, the same targets go through the configured container image.

### Container builds

All targets can be executed through Podman/Docker when their strategy resolves to `container`.

Linux and Android are the primary portable container targets. Windows, macOS, and iOS also have container slots and image settings so your release infrastructure can provide the required cross-build or remote SDK environment without writing artifacts into the source tree.

The container can reuse a shared cache directory, but generated project files and temporary build outputs stay inside the release session unless they match the artifact whitelist.

### Native host builds

When the current host matches the selected target and the strategy is `auto`, `tauri-release` runs the configured command on the host and then applies the same artifact collection step.

This avoids unnecessary cross-compiling when it is not needed:

```text
Windows host -> Windows Tauri build runs on host
macOS host   -> macOS/iOS Tauri build runs on host
Linux host   -> Linux Tauri build runs on host
```

You can still force containers with `strategy = "container"`.

### Cleanup

The project keeps only explicit release artifacts.

By default, collected file types include:

```text
AppImage, deb, rpm, exe, msi, zip, dmg, apk, aab, ipa, sig, json
```

Special release metadata files such as `latest.json` and `release.json` are also kept. Everything else is treated as temporary build output.

## Get started

> [!IMPORTANT]
> `verzly/tauri-release` requires Rust and a container engine for containerized targets.
>
> Windows, macOS, and iOS can be container-orchestrated, but the image still has to contain a working platform toolchain for your release flow. With `strategy = "auto"`, matching native hosts are used directly.

### Install

Install stable releases directly from the Git repository. The recommended target is the moving `latest` tag, which is maintained by the release-tag workflow.

```sh
cargo install --git https://github.com/verzly/tauri-release --tag latest --force
```

Install a specific release tag when you need a reproducible tool version:

```sh
cargo install --git https://github.com/verzly/tauri-release --tag v0.1.5 --force
```

Development branch installation is only recommended when contributing or testing unreleased changes:

```sh
cargo install --git https://github.com/verzly/tauri-release --branch main --force
```

For a local checkout during development:

```sh
cargo install --path . --force
```

After installation, both command styles are available:

```sh
tauri-release --help
cargo tauri-release --help
```

### Create config

From the root of a Tauri project, create a config file and inspect the plan:

```sh
tauri-release init
tauri-release plan --linux --android
```

Then run the build:

```sh
tauri-release build --linux --android --apk --aab --android-target aarch64
```

The release files will be written to:

```text
dist/<version>/
```

Overwrite an existing config:

```sh
tauri-release init --force
```

Use a custom config path:

```sh
tauri-release init release/tauri-release.toml
```

### Upgrade

Upgrade to the latest stable tag:

```sh
cargo install --git https://github.com/verzly/tauri-release --tag latest --force
```

Upgrade or pin to a specific release tag:

```sh
cargo install --git https://github.com/verzly/tauri-release --tag v0.1.5 --force
```

Only use the development branch if you intentionally want unreleased changes:

```sh
cargo install --git https://github.com/verzly/tauri-release --branch main --force
```

## Usage

After installing the tool, run it from a Tauri app root, or pass `--project-dir` for monorepos.

### Build Linux and Android

```sh
# Build Linux desktop bundles only
tauri-release build --linux

# Build Android only
tauri-release build --android --apk --aab --android-target aarch64

# Build both in one release output
tauri-release build --linux --android --apk --aab --android-target aarch64
```

Preview the resolved release plan without starting containers:

```sh
tauri-release plan --linux --android --apk --aab
```

### Build Windows, macOS, and iOS

The same CLI targets are available for platform-specific releases:

```sh
tauri-release build --windows
tauri-release build --macos
tauri-release build --ios
```

With `strategy = "auto"`, these use the native host when it matches the target. Otherwise they use the configured container image:

```toml
[windows]
enabled = true
strategy = "auto"
image = "ghcr.io/your-org/tauri-release-windows:latest"

[macos]
enabled = true
strategy = "auto"
image = "ghcr.io/your-org/tauri-release-macos:latest"

[ios]
enabled = true
strategy = "auto"
image = "ghcr.io/your-org/tauri-release-ios:latest"
```

Force Podman/Docker even on a matching host:

```toml
strategy = "container"
```

Force native host execution only:

```toml
strategy = "host-only"
```

### Monorepo projects

For repositories where desktop and mobile apps live under `apps/*`, run each app separately:

```text
repo/
  apps/
    desktop/
      package.json
      src-tauri/
    mobile/
      package.json
      src-tauri/
```

```sh
tauri-release build --project-dir apps/desktop --linux
tauri-release build --project-dir apps/mobile --android --apk --aab --android-target aarch64
```

You can also keep separate config files in `examples/` or in your own release directory:

```sh
tauri-release build --config examples/nutrino-desktop.toml
tauri-release build --config examples/nutrino-mobile.toml
```

### Android APK and AAB

Build APK only:

```sh
tauri-release build --android --apk --android-target aarch64
```

Build AAB only:

```sh
tauri-release build --android --aab --android-target aarch64
```

Build both APK and AAB:

```sh
tauri-release build --android --apk --aab --android-target aarch64
```

### Artifact collection

Artifacts are collected by extension and explicit file names:

```toml
[artifacts]
include_extensions = ["AppImage", "deb", "rpm", "exe", "msi", "zip", "dmg", "apk", "aab", "ipa", "sig", "json"]
include_files = ["latest.json", "release.json"]
keep_relative_paths = false
```

### Checksums

SHA-256 files are generated next to collected artifacts by default:

```toml
[output]
sha256 = true
```

### Cache

The default cache directory is platform-specific under the user cache directory. Override it per project:

```toml
[container]
cache_dir = ".cache/tauri-release"
```

Or per command:

```sh
TAURI_RELEASE_CACHE_DIR=/tmp/tauri-release-cache tauri-release build --android
```

### Configuration

Generate the example config:

```sh
tauri-release init
```

The source example lives under:

```text
examples/tauri-release.toml
```

### Container images

Templates are included under `templates/`:

```text
templates/Containerfile.linux
templates/Containerfile.android
templates/Containerfile.windows
templates/Containerfile.macos
templates/Containerfile.ios
```

The default image names are placeholders. Replace them in `tauri-release.toml` with project-owned images that pin the exact Rust, Node, Tauri, SDK, signing, and bundling toolchain used by your release flow.

## Known issues

Windows, macOS, and iOS container builds are only as reliable as the toolchain inside the configured image. `tauri-release` orchestrates the release session, cache, output, and cleanup. It does not magically provide Apple SDKs, Windows signing certificates, or proprietary build tools.

Android builds still require the generated Tauri Android project internally. The generated Gradle output is kept inside the disposable workspace unless it matches the artifact whitelist.

## Contributing

Keep changes small, reproducible, and release-focused. Prefer explicit platform strategy, deterministic output paths, and artifact whitelists over implicit build side effects.

Release tags are managed by GitHub Actions. On `main`, the workflow reads `Cargo.toml` `package.version`, creates the immutable `vX.Y.Z` tag when it does not exist, and moves the `latest` tag to the same commit.

If the version tag already exists on another commit, the workflow fails instead of moving it. Bump `Cargo.toml` before publishing another release commit for the same project.

## License

This project is licensed under AGPL-3.0-or-later. See [LICENSE](LICENSE) for details.
