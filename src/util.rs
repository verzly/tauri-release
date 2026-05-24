use std::env;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn default_cache_dir() -> PathBuf {
    if let Ok(value) = env::var("TAURI_RELEASE_CACHE_DIR") {
        return PathBuf::from(value);
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".cache").join("tauri-release");
    }

    if let Ok(profile) = env::var("USERPROFILE") {
        return PathBuf::from(profile).join(".cache").join("tauri-release");
    }

    PathBuf::from(".cache").join("tauri-release")
}

pub fn absolutize(path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let path = path.as_ref();
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

pub fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return if cfg!(windows) { "\"\"".into() } else { "''".into() };
    }

    if value.chars().all(is_shell_safe_char) {
        return value.to_string();
    }

    if cfg!(windows) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn is_shell_safe_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | '.' | '/' | ':' | '=' | ',' | '+' | '@' | '%')
}

pub fn now_session_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{}-{}-{}", prefix, std::process::id(), millis)
}

pub fn path_for_container(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn current_host_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

pub fn host_matches_platform(platform: &str) -> bool {
    match platform {
        "windows" => cfg!(target_os = "windows"),
        "macos" | "ios" => cfg!(target_os = "macos"),
        "linux" => cfg!(target_os = "linux"),
        _ => false,
    }
}
