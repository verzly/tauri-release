# Windows strategy

Tauri Windows installers should be built on a Windows runner/host when you need reliable MSI/NSIS output, signing, and native toolchain behavior.

This project intentionally ships Windows as a `host-only` strategy instead of pretending that Linux Podman can reliably produce all Windows Tauri bundle types.

Recommended setup:

```powershell
cargo install --path . --force
tauri-release --windows --output dist\0.1.0
```

The output copy step keeps only whitelisted release artifacts such as `.exe`, `.msi`, `.sig`, `.json`, and `.sha256`.
