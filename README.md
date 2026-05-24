# verzly/tauri-release

`verzly/tauri-release` is a release orchestrator for Tauri applications. It builds final desktop and mobile artifacts in disposable Podman/Docker build sessions, then copies only the files that should be published into `dist/<version>/`.

It is designed for projects where `src-tauri/target`, `src-tauri/gen/android/app/build`, Gradle outputs, frontend dependencies, Android intermediates, and other generated files should not keep growing inside the source tree.

It provides an integrated release flow with built-in support for:
- **Linux desktop bundles** through containerized Tauri builds
- **Android APK/AAB builds** through containerized Android SDK/NDK builds
- **Windows, macOS, and iOS release slots** for native host or CI runner builds
- **Artifact whitelisting** so only installers, packages, signatures, metadata, and checksums are kept
- **SHA-256 checksums** for every collected release file
- **Release manifest generation** for predictable release publishing

Build inside a disposable workspace, keep only the executable artifacts, and let the container session remove everything else.

- [How it works](#how-it-works)
  - [Container builds](#container-builds)
  - [Native host builds](#native-host-builds)
  - [Cleanup](#cleanup)
- [Get started](#get-started)
  - [Install](#get-started)
  - [Create config](#create-config)
  - [Upgrade](#up-to-date)
- [Usage](#usage)
  - [Build Linux and Android](#build-linux-and-android)
  - [Monorepo projects](#monorepo-projects)
  - [Android APK and AAB](#android-apk-and-aab)
  - [Artifact collection](#artifact-collection)
  - [Checksums](#checksums)
  - [Cache](#cache)
  - [Configuration](#configuration)
  - [Container images](#container-images)
- [Known issues](#known-issues)
- [Contributing](#contributing)

Read on to understand why `verzly/tauri-release` exists and how the build isolation works. Or jump straight to [Get started](#get-started) for installation and first build steps.

## How it works

Tauri release builds can create a large amount of intermediate output. Desktop builds use Cargo, frontend package managers, and Tauri bundlers. Android builds also generate a Gradle project under `src-tauri/gen/android`, compile native Rust libraries, package APK/AAB files, and keep Gradle intermediates.

`verzly/tauri-release` avoids polluting the working tree by running the expensive build in an isolated workspace:

```text
host project  ->  mounted read-only at /src
container     ->  copies /src into /work/project
build output  ->  collected into /out
host dist     ->  receives only whitelisted files
```

The source project is treated as input. The release directory is treated as output. Everything else is disposable.

### Container builds

Linux and Android are the primary container targets.

For Linux, the container image contains the Rust, Node, package-manager, and system dependencies needed by Tauri desktop builds.

For Android, the container image contains the Android SDK, Android NDK, JDK, Rust Android targets, Node, the selected package manager, and Tauri CLI support.

The container can reuse a shared cache directory, but generated project files and temporary build outputs stay inside the release session unless they match the artifact whitelist.

### Native host builds

Windows, macOS, and iOS are represented in the project structure, but they are host-native release targets by default.

This is intentional. Tauri Windows installers, macOS bundles, signing, notarization, and iOS builds often depend on platform-native SDKs and signing tools. `verzly/tauri-release` gives these targets a consistent release plan and artifact collection path without hiding platform constraints behind fragile cross-build assumptions.

Recommended target strategy:

| Target | Default strategy | Notes |
|---|---|---|
| Linux desktop | Container | AppImage, deb, rpm, or other configured Tauri bundles |
| Android | Container | APK/AAB with Android SDK/NDK inside the image |
| Windows | Windows host/runner | MSI, NSIS, EXE, signing |
| macOS | macOS host/runner | app, dmg, signing, notarization |
| iOS | macOS host/runner | Xcode/iOS toolchain required |

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
> Linux and Android builds are intended to run through Podman or Docker. Windows, macOS, and iOS targets require suitable native hosts or CI runners if you enable them.

Install from a local checkout during development:

```sh
cargo install --path . --force
```

After installation, both command styles are available:

```sh
tauri-release --help
cargo tauri-release --help
```

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

### Create config

Generate a starter config:

```sh
tauri-release init
```

Overwrite an existing config:

```sh
tauri-release init --force
```

Use a custom config path:

```sh
tauri-release init release/tauri-release.toml
```

### Up-to-date

When the tool is installed from a local checkout, reinstall after pulling updates:

```sh
cargo install --path . --force
```

If the project is published later, install or update it through Cargo in the same way as other Rust binaries:

```sh
cargo install tauri-release
cargo install --force tauri-release
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

If no subcommand is given, `build` is used as the default command:

```sh
tauri-release --linux --android --apk
```

Preview the resolved release plan without starting containers:

```sh
tauri-release plan --linux --android --apk --aab
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

You can also keep separate config files:

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

Build multiple Android targets:

```sh
tauri-release build --android --apk --android-target aarch64 --android-target armv7
```

Split Android packages per ABI:

```sh
tauri-release build --android --apk --split-per-abi
```

### Artifact collection

Collected release files are copied into the output directory and normalized. By default, `verzly/tauri-release` keeps executable packages, installers, update metadata, signatures, and JSON release metadata.

The default artifact extensions are:

```text
AppImage, deb, rpm, exe, msi, zip, dmg, apk, aab, ipa, sig, json
```

The default special files are:

```text
latest.json
release.json
```

Configure this in `tauri-release.toml`:

```toml
[artifacts]
include_extensions = ["AppImage", "deb", "rpm", "apk", "aab", "sig", "json"]
include_files = ["latest.json", "release.json"]
keep_relative_paths = false
```

### Checksums

Every collected file receives a `.sha256` file when checksum generation is enabled.

Verify on Linux:

```sh
sha256sum -c <filename>.sha256
```

Verify on macOS:

```sh
shasum -a 256 -c <filename>.sha256
```

Verify on Windows PowerShell:

```pwsh
(Get-FileHash <filename> -Algorithm SHA256).Hash -eq (Get-Content <filename>.sha256).Split(" ")[0]
```

### Cache

Clear the shared cache:

```sh
tauri-release clean-cache
```

Use a custom cache directory:

```sh
tauri-release clean-cache --cache-dir /path/to/cache
```

Or override it for a build:

```sh
TAURI_RELEASE_CACHE_DIR=/path/to/cache tauri-release build --android
```

> [!TIP]
> Keep caches outside the source tree if you want the repository to stay small. Keep them inside a known CI cache directory if you want faster repeat builds.

### Configuration

A minimal mobile config:

```toml
[project]
project_dir = "apps/mobile"
package_manager = "pnpm"

[container]
engine = "podman"
android_image = "localhost/tauri-release-android:latest"

[output]
clean = true
sha256 = true
manifest = true

[android]
enabled = true
apk = true
aab = true
targets = ["aarch64"]
```

A minimal desktop Linux config:

```toml
[project]
project_dir = "apps/desktop"
package_manager = "pnpm"

[container]
engine = "podman"
linux_image = "localhost/tauri-release-linux:latest"

[linux]
enabled = true
bundles = ["appimage", "deb", "rpm"]
```

Common command-line overrides:

```sh
# Choose project directory
tauri-release build --project-dir apps/mobile --android

# Choose output directory
tauri-release build --output dist/releases/0.1.0 --linux

# Use Docker instead of Podman where supported
tauri-release build --engine docker --linux

# Pass extra arguments to the Tauri CLI
tauri-release build --linux --tauri-arg=--verbose
```

### Container images

Build the included starter images:

```sh
podman build -f templates/Containerfile.linux -t localhost/tauri-release-linux:latest .
podman build -f templates/Containerfile.android -t localhost/tauri-release-android:latest .
```

Then reference them from `tauri-release.toml`:

```toml
[container]
linux_image = "localhost/tauri-release-linux:latest"
android_image = "localhost/tauri-release-android:latest"
```

## Known issues

- Windows, macOS, and iOS targets are intentionally host-native by default.
- Android release signing still depends on your Tauri/Android project configuration.
- Container images are starter templates. Production projects should pin tool versions, Android SDK versions, Node versions, and Rust toolchains.
- The source project should not rely on uncommitted generated files unless those files are also available inside the container build context.

## Contributing

If you want to contribute to `verzly/tauri-release` or test local changes, install the local binary:

```sh
cargo install --path . --force
```

Run the validation script:

```sh
scripts/validate-project.sh
```

Test against a real Tauri project:

```sh
tauri-release plan --project-dir /path/to/app --linux --android
tauri-release build --project-dir /path/to/app --linux --android --apk --verbose
```

Keep release behavior explicit: build in a disposable workspace, collect only whitelisted artifacts, and avoid writing generated build output back into the source tree.
