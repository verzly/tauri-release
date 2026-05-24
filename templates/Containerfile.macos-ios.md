# macOS and iOS strategy

macOS and iOS builds require macOS hosts. This includes Xcode, code signing, notarization, and iOS IPA generation.

Use `strategy = "host-only"` on a macOS runner and point `command` to the exact release command your app needs:

```toml
[macos]
enabled = true
strategy = "host-only"
command = "pnpm tauri build"

[ios]
enabled = true
strategy = "host-only"
command = "pnpm tauri ios build --ci"
```
